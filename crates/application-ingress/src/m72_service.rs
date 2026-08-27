use ustc_campus_agent_client_protocol::{
    ActorIntentDto, ClientErrorDto, ClientResponseDto, EchoPayloadDto, M10WireErrorDto,
    OpportunityCommandDto, OpportunityRejectionDto, OpportunitySourceHealthDto, RetryabilityDto,
    SubmitOpportunityDto, UnixMillis, WireErrorClassDto, WireText, opportunity_payload_digest,
};
use ustc_campus_agent_core::identity::{CorrelationId, RequestId, SessionId};
use ustc_campus_agent_core::request_context::{
    ActorReference, ClientProvenance, M00AdmissionResult, M00AdmittedActor, OperationId,
    PayloadDigest, PublicScope, RequestAdmissionCoordinator,
};
use ustc_campus_agent_core::source_revision::SourceRevisionHealth;
use ustc_campus_agent_opportunity_graph::{
    DeletionReceipt, OpportunityPlanReceipt, OpportunityPlanningError, OpportunityProfileError,
    OpportunityRepositoryError, TenantProfileRecord,
};

use crate::capability::constant_time_eq;
use crate::m00_projection::project_rejection;
use crate::m72_projection::{
    M72ProjectionError, terminal_plan_generated, terminal_profile_created,
    terminal_profile_deleted, terminal_profile_found,
};
use crate::service::M10AdmissionPorts;

pub trait OpportunityInvocationPort: Send + Sync {
    fn invoke(
        &self,
        actor: &M00AdmittedActor,
        command: &OpportunityCommandDto,
    ) -> Result<OpportunityInvocationOutcome, OpportunityInvocationError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OpportunityInvocationOutcome {
    ProfileCreated(TenantProfileRecord),
    ProfileFound(TenantProfileRecord),
    PlanGenerated(OpportunityPlanReceipt),
    ProfileDeleted(DeletionReceipt),
}

#[derive(Debug)]
pub enum OpportunityInvocationError {
    Profile(OpportunityProfileError),
    Planning(OpportunityPlanningError),
    Denied,
    Unavailable,
    OutcomeUnknown,
    Internal,
}

pub struct M10OpportunityService<'a> {
    opportunity: &'a dyn OpportunityInvocationPort,
}

impl<'a> M10OpportunityService<'a> {
    #[must_use]
    pub const fn new(opportunity: &'a dyn OpportunityInvocationPort) -> Self {
        Self { opportunity }
    }

    #[must_use]
    pub fn submit<P: M10AdmissionPorts>(
        &self,
        request: &SubmitOpportunityDto,
        ports: &mut P,
    ) -> ClientResponseDto {
        let operation_id = request.command.operation_id();
        if request.command.validate().is_err() {
            return malformed_command_error(operation_id);
        }
        let expected_digest = match opportunity_payload_digest(&request.command) {
            Ok(value) => value,
            Err(_) => return malformed_command_error(operation_id),
        };
        if !constant_time_eq(
            request.payload_digest.as_str().as_bytes(),
            expected_digest.as_str().as_bytes(),
        ) {
            return malformed_command_error(operation_id);
        }
        let command = match build_command(request) {
            Ok(value) => value,
            Err(_) => return malformed_command_error(operation_id),
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
                    retry_not_before: UnixMillis::new(
                        i64::try_from(value.retry_not_before().as_unix_millis())
                            .unwrap_or(i64::MAX),
                    ),
                };
            }
            M00AdmissionResult::Admitted { disposition, .. }
            | M00AdmissionResult::PriorAdmitted(disposition) => disposition,
        };
        if disposition.descriptor_snapshot_id() != &staged_identity {
            return internal_error("opportunity_descriptor_identity_drift");
        }
        if !matches!(
            disposition.admitted_actor(),
            M00AdmittedActor::Authenticated(_)
        ) {
            return rejected(
                disposition.command_id().as_str(),
                operation_id,
                OpportunityRejectionDto::AuthenticationRequired,
            );
        }
        let outcome = match self
            .opportunity
            .invoke(disposition.admitted_actor(), &request.command)
        {
            Ok(value) => value,
            Err(OpportunityInvocationError::OutcomeUnknown) => {
                return ClientResponseDto::Incomplete {
                    command_id: wire(disposition.command_id().as_str()),
                    retry_not_before: UnixMillis::new(0),
                };
            }
            Err(error) => {
                return map_invocation_error(
                    disposition.command_id().as_str(),
                    operation_id,
                    error,
                );
            }
        };
        let terminal = match outcome {
            OpportunityInvocationOutcome::ProfileCreated(record) => {
                terminal_profile_created(&record)
            }
            OpportunityInvocationOutcome::ProfileFound(record) => terminal_profile_found(&record),
            OpportunityInvocationOutcome::PlanGenerated(receipt) => {
                terminal_plan_generated(&receipt)
            }
            OpportunityInvocationOutcome::ProfileDeleted(receipt) => {
                terminal_profile_deleted(&receipt)
            }
        };
        let terminal = match terminal {
            Ok(value) => value,
            Err(M72ProjectionError::WireText) => {
                return internal_error("opportunity_projection_wire_text");
            }
            Err(M72ProjectionError::Count) => {
                return internal_error("opportunity_projection_count");
            }
            Err(M72ProjectionError::Timestamp) => {
                return internal_error("opportunity_projection_timestamp");
            }
        };
        ClientResponseDto::OpportunityAccepted {
            command_id: wire(disposition.command_id().as_str()),
            terminal: Box::new(terminal),
        }
    }
}

fn build_command(
    request: &SubmitOpportunityDto,
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
    let operation_id =
        OperationId::parse(request.command.operation_id()).map_err(|_| "operation id")?;
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

fn map_invocation_error(
    command_id: &str,
    operation_id: &str,
    error: OpportunityInvocationError,
) -> ClientResponseDto {
    match error {
        OpportunityInvocationError::Denied => invocation_denied_error(operation_id),
        OpportunityInvocationError::Unavailable => {
            infrastructure_error("opportunity_invocation_unavailable")
        }
        OpportunityInvocationError::OutcomeUnknown => {
            internal_error("opportunity_outcome_unknown_unreachable")
        }
        OpportunityInvocationError::Internal => internal_error("opportunity_invocation_internal"),
        OpportunityInvocationError::Profile(error) => {
            map_profile_error(command_id, operation_id, error)
        }
        OpportunityInvocationError::Planning(error) => {
            map_planning_error(command_id, operation_id, error)
        }
    }
}

fn map_profile_error(
    command_id: &str,
    operation_id: &str,
    error: OpportunityProfileError,
) -> ClientResponseDto {
    let rejection = match error {
        OpportunityProfileError::AccessDenied => OpportunityRejectionDto::AccessDenied,
        OpportunityProfileError::MissingProfile => OpportunityRejectionDto::MissingProfile,
        OpportunityProfileError::ProfileDeleted => OpportunityRejectionDto::ProfileDeleted,
        OpportunityProfileError::DeleteBeforeConsent => {
            OpportunityRejectionDto::DeleteBeforeConsent
        }
        OpportunityProfileError::Repository(
            OpportunityRepositoryError::PrincipalAlreadyHasProfile,
        ) => OpportunityRejectionDto::ProfileAlreadyExists,
        OpportunityProfileError::Repository(OpportunityRepositoryError::Unavailable)
        | OpportunityProfileError::Repository(OpportunityRepositoryError::CapacityExceeded) => {
            return infrastructure_error("opportunity_repository_unavailable");
        }
        OpportunityProfileError::Value(_) => return malformed_command_error(operation_id),
        OpportunityProfileError::Repository(_) => {
            return internal_error("opportunity_repository_invariant");
        }
        _ => return internal_error("opportunity_profile_non_exhaustive"),
    };
    rejected(command_id, operation_id, rejection)
}

fn map_planning_error(
    command_id: &str,
    operation_id: &str,
    error: OpportunityPlanningError,
) -> ClientResponseDto {
    let rejection = match error {
        OpportunityPlanningError::AccessDenied => OpportunityRejectionDto::AccessDenied,
        OpportunityPlanningError::MissingProfile => OpportunityRejectionDto::MissingProfile,
        OpportunityPlanningError::ProfileDeleted => OpportunityRejectionDto::ProfileDeleted,
        OpportunityPlanningError::SourceNotCurrent(SourceRevisionHealth::Stale) => {
            OpportunityRejectionDto::SourceNotCurrent {
                health: OpportunitySourceHealthDto::Stale,
            }
        }
        OpportunityPlanningError::SourceNotCurrent(SourceRevisionHealth::Conflicting) => {
            OpportunityRejectionDto::SourceNotCurrent {
                health: OpportunitySourceHealthDto::Conflicting,
            }
        }
        OpportunityPlanningError::SourceNotCurrent(SourceRevisionHealth::Current) => {
            return internal_error("opportunity_current_source_rejected");
        }
        OpportunityPlanningError::SourceUnavailable(_) => {
            OpportunityRejectionDto::SourceUnavailable
        }
        OpportunityPlanningError::InvalidPlanningBounds => {
            return malformed_command_error(operation_id);
        }
        OpportunityPlanningError::InvalidProfileFacts => {
            OpportunityRejectionDto::InvalidProfileFacts
        }
        OpportunityPlanningError::Repository(OpportunityRepositoryError::Unavailable)
        | OpportunityPlanningError::Repository(OpportunityRepositoryError::CapacityExceeded) => {
            return infrastructure_error("opportunity_repository_unavailable");
        }
        OpportunityPlanningError::Repository(_) => {
            return internal_error("opportunity_repository_invariant");
        }
        OpportunityPlanningError::PlanningFailed
        | OpportunityPlanningError::ResultSerializationFailed => {
            return internal_error("opportunity_planning_invariant");
        }
        _ => return internal_error("opportunity_planning_non_exhaustive"),
    };
    rejected(command_id, operation_id, rejection)
}

fn rejected(
    command_id: &str,
    operation_id: &str,
    rejection: OpportunityRejectionDto,
) -> ClientResponseDto {
    ClientResponseDto::OpportunityRejected {
        command_id: wire(command_id),
        operation_id: wire(operation_id),
        rejection,
    }
}

fn invocation_denied_error(operation_id: &str) -> ClientResponseDto {
    let error = match M10WireErrorDto::try_new(
        WireErrorClassDto::PolicyDenied,
        RetryabilityDto::NotRetryable,
        wire("policy_denied"),
        EchoPayloadDto::PolicyDenied {
            operation_id: wire(operation_id),
            permission_class: wire(permission_class(operation_id)),
        },
    ) {
        Ok(value) => value,
        Err(_) => return internal_error("opportunity_invocation_denial_projection"),
    };
    ClientResponseDto::Error {
        error: ClientErrorDto::Admission { error },
    }
}

fn permission_class(operation_id: &str) -> &'static str {
    match operation_id {
        "profile.academic.view" => "tenant_private_read",
        _ => "tenant_private_write",
    }
}

fn infrastructure_error(code: &str) -> ClientResponseDto {
    ClientResponseDto::Error {
        error: ClientErrorDto::Infrastructure {
            retryable: true,
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

fn malformed_command_error(operation_id: &str) -> ClientResponseDto {
    let error = match M10WireErrorDto::try_new(
        WireErrorClassDto::MalformedCommand,
        RetryabilityDto::RetryableAfterChange,
        wire("malformed_command"),
        EchoPayloadDto::Operation {
            operation_id: wire(operation_id),
        },
    ) {
        Ok(value) => value,
        Err(_) => return internal_error("opportunity_malformed_command_projection"),
    };
    ClientResponseDto::Error {
        error: ClientErrorDto::Admission { error },
    }
}

fn wire(value: &str) -> WireText {
    WireText::parse(value).unwrap_or_else(|_| WireText::fallback())
}
