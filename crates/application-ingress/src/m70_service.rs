use ustc_campus_agent_change_radar::{BoardId, ChangeFeedQueryError, ChangeFeedReceipt};
use ustc_campus_agent_client_protocol::{
    ActorIntentDto, ClientErrorDto, ClientResponseDto, EchoPayloadDto, M10WireErrorDto,
    M70ChangeFeedOutcomeDto, M70ChangeFeedTerminalDto, RetryabilityDto, SubmitChangeFeedDto,
    WireErrorClassDto, WireText, change_feed_payload_digest,
};
use ustc_campus_agent_core::identity::{CorrelationId, RequestId, SessionId};
use ustc_campus_agent_core::request_context::{
    ActorReference, ClientProvenance, M00AdmissionResult, M00AdmittedActor, OperationId,
    PayloadDigest, PublicScope, RequestAdmissionCoordinator,
};

use crate::capability::constant_time_eq;
use crate::m00_projection::project_rejection;
use crate::m70_projection::project_change_feed;
use crate::service::M10AdmissionPorts;

pub trait ChangeFeedInvocationPort: Send + Sync {
    fn invoke(
        &self,
        actor: &M00AdmittedActor,
        board_id: &BoardId,
    ) -> Result<ChangeFeedInvocationOutcome, ChangeFeedInvocationError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChangeFeedInvocationOutcome {
    Found(ChangeFeedReceipt),
    NotFound(BoardId),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChangeFeedInvocationError {
    Downstream(ChangeFeedQueryError),
    Denied,
    Unavailable,
    Internal,
}

pub struct M10ChangeFeedService<'a> {
    change_feed: &'a dyn ChangeFeedInvocationPort,
}

impl<'a> M10ChangeFeedService<'a> {
    #[must_use]
    pub const fn new(change_feed: &'a dyn ChangeFeedInvocationPort) -> Self {
        Self { change_feed }
    }

    pub fn submit<P: M10AdmissionPorts>(
        &self,
        request: &SubmitChangeFeedDto,
        ports: &mut P,
    ) -> ClientResponseDto {
        let expected_digest = match change_feed_payload_digest(&request.board_id) {
            Ok(value) => value,
            Err(_) => return malformed_command_error(),
        };
        if !constant_time_eq(
            request.payload_digest.as_str().as_bytes(),
            expected_digest.as_str().as_bytes(),
        ) {
            return malformed_command_error();
        }
        let board_id = match BoardId::parse(request.board_id.as_str()) {
            Ok(value) => value,
            Err(_) => return malformed_command_error(),
        };
        let command = match build_command(request) {
            Ok(value) => value,
            Err(_) => return internal_error("change_request_context"),
        };
        let staged = ports.staged_operation();
        let staged_identity = staged.snapshot_identity().clone();
        let disposition = match RequestAdmissionCoordinator.admit(&command, ports) {
            M00AdmissionResult::Rejected(rejection)
            | M00AdmissionResult::PriorRejected(rejection) => {
                return ClientResponseDto::Error {
                    error: project_rejection(&rejection),
                };
            }
            M00AdmissionResult::Incomplete(value) => {
                return ClientResponseDto::Incomplete {
                    command_id: wire(value.command_id().as_str()),
                    retry_not_before: ustc_campus_agent_client_protocol::UnixMillis::new(
                        i64::try_from(value.retry_not_before().as_unix_millis())
                            .unwrap_or(i64::MAX),
                    ),
                };
            }
            M00AdmissionResult::Admitted { disposition, .. }
            | M00AdmissionResult::PriorAdmitted(disposition) => disposition,
        };
        if disposition.descriptor_snapshot_id() != &staged_identity {
            return internal_error("change_descriptor_identity_drift");
        }
        let outcome = match self
            .change_feed
            .invoke(disposition.admitted_actor(), &board_id)
        {
            Ok(value) => value,
            Err(error) => return map_invocation_error(error),
        };
        let terminal = match outcome {
            ChangeFeedInvocationOutcome::Found(receipt) => match project_change_feed(&receipt) {
                Ok(value) => value,
                Err(_) => return internal_error("change_projection_invariant"),
            },
            ChangeFeedInvocationOutcome::NotFound(board_id) => {
                M70ChangeFeedTerminalDto::new(M70ChangeFeedOutcomeDto::NotFound {
                    board_id: wire(board_id.as_str()),
                })
            }
        };
        ClientResponseDto::ChangeFeedAccepted {
            command_id: wire(disposition.command_id().as_str()),
            terminal: Box::new(terminal),
        }
    }
}

fn build_command(
    request: &SubmitChangeFeedDto,
) -> Result<ustc_campus_agent_core::request_context::BuildRequestContextCommand, &'static str> {
    let request_id = RequestId::parse(request.request_id.as_str()).map_err(|_| "request id")?;
    let correlation_id =
        CorrelationId::parse(request.correlation_id.as_str()).map_err(|_| "correlation id")?;
    let actor_reference = match &request.actor {
        ActorIntentDto::Public => ActorReference::Anonymous { scope: PublicScope },
        ActorIntentDto::Authenticated { session_id } => ActorReference::Authenticated {
            session_id: SessionId::parse(session_id.as_str()).map_err(|_| "session id")?,
        },
    };
    let causation_id = request
        .causation_id
        .as_ref()
        .map(|value| {
            ustc_campus_agent_core::request_context::CausationId::parse(value.as_str())
                .map_err(|_| "causation id")
        })
        .transpose()?;
    let idempotency_key = request
        .idempotency_key
        .as_ref()
        .map(|value| {
            ustc_campus_agent_core::request_context::IdempotencyKey::parse(value.as_str())
                .map_err(|_| "idempotency key")
        })
        .transpose()?;
    let provenance = ClientProvenance::new(
        request.provenance.build.as_str(),
        request.provenance.target.as_str(),
        request.provenance.protocol.as_str(),
    )
    .map_err(|_| "client provenance")?;
    let digest =
        PayloadDigest::parse(request.payload_digest.as_str()).map_err(|_| "payload digest")?;
    let operation_id = OperationId::parse("change.list").map_err(|_| "operation id")?;
    Ok(
        ustc_campus_agent_core::request_context::BuildRequestContextCommand::new(
            request_id,
            operation_id,
            actor_reference,
            correlation_id,
            causation_id,
            idempotency_key,
            provenance,
            digest,
        ),
    )
}

fn map_invocation_error(error: ChangeFeedInvocationError) -> ClientResponseDto {
    match error {
        ChangeFeedInvocationError::Downstream(ChangeFeedQueryError::Repository(_))
        | ChangeFeedInvocationError::Unavailable => {
            infrastructure_error("change_invocation_unavailable")
        }
        ChangeFeedInvocationError::Downstream(ChangeFeedQueryError::Projection)
        | ChangeFeedInvocationError::Internal => internal_error("change_invocation_internal"),
        ChangeFeedInvocationError::Denied => invocation_denied_error(),
    }
}

fn invocation_denied_error() -> ClientResponseDto {
    let error = match M10WireErrorDto::try_new(
        WireErrorClassDto::PolicyDenied,
        RetryabilityDto::NotRetryable,
        wire("policy_denied"),
        EchoPayloadDto::PolicyDenied {
            operation_id: wire("change.list"),
            permission_class: wire("public_read"),
        },
    ) {
        Ok(value) => value,
        Err(_) => return internal_error("change_invocation_denial_projection"),
    };
    ClientResponseDto::Error {
        error: ClientErrorDto::Admission { error },
    }
}

fn infrastructure_error(code: &str) -> ClientResponseDto {
    ClientResponseDto::Error {
        error: ClientErrorDto::Infrastructure {
            retryable: code.ends_with("retry") || code.contains("unavailable"),
            wire_code: wire(code),
        },
    }
}

fn internal_error(code: &str) -> ClientResponseDto {
    ClientResponseDto::Error {
        error: ClientErrorDto::InternalInvariant {
            wire_code: wire(code),
        },
    }
}

fn malformed_command_error() -> ClientResponseDto {
    let error = match M10WireErrorDto::try_new(
        WireErrorClassDto::MalformedCommand,
        RetryabilityDto::RetryableAfterChange,
        wire("malformed_command"),
        EchoPayloadDto::Operation {
            operation_id: wire("change.list"),
        },
    ) {
        Ok(value) => value,
        Err(_) => return internal_error("change_malformed_command_projection"),
    };
    ClientResponseDto::Error {
        error: ClientErrorDto::Admission { error },
    }
}

fn wire(value: &str) -> WireText {
    WireText::parse(value).unwrap_or_else(|_| WireText::fallback())
}
