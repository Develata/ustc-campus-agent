use affairs_navigator::{AffairsGetQuery, GetProcedureError, M71AffairsGetReceipt, ProcedureId};
use time::OffsetDateTime;
use ustc_campus_agent_client_protocol::{
    ActorIntentDto, AdmittedActorDto, AffairsGetPayloadDto, ClientErrorDto, ClientResponseDto,
    DispatchCapsuleBodyV2, EchoPayloadDto, FrozenPrerequisitesDto, M10WireErrorDto, RedactionDto,
    RetryabilityDto, SubmitAffairsGetDto, UnixMillis, ViewerAuthorizationDto, WireErrorClassDto,
    WireText, affairs_get_payload_digest,
};
use ustc_campus_agent_core::identity::{CorrelationId, RequestId, SessionId};
use ustc_campus_agent_core::request_context::{
    ActorReference, AdmissionPorts, ClientProvenance, M00AdmissionResult, M00AdmittedActor,
    OperationId, OperationSnapshot, PayloadDigest, PublicScope, RequestAdmissionCoordinator,
};

use crate::capability::{CapabilityIssuer, constant_time_eq};
use crate::m00_projection::project_rejection;
use crate::m71_projection::project_receipt;
use crate::persistence::{
    ClaimOutcome, CompleteOutcome, FileRecordStore, InsertOutcome, RecordState, StoredReadPolicy,
    StoredRecord,
};

pub trait M10AdmissionPorts: AdmissionPorts {
    fn staged_operation(&self) -> OperationSnapshot;
}

/// M10-owned application seam for an admitted Affairs invocation.
///
/// The caller must pass the exact M00-admitted actor. Implementations may
/// narrow authority through Market/Agent/ToolGateway before delegating to M71;
/// M10 never fabricates a direct M71 fallback after this port denies or fails.
pub trait AffairsInvocationPort: Send + Sync {
    fn invoke(
        &self,
        actor: &M00AdmittedActor,
        query: &AffairsGetQuery,
    ) -> Result<M71AffairsGetReceipt, AffairsInvocationError>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AffairsInvocationError {
    Downstream(GetProcedureError),
    Denied,
    Unavailable,
    Internal,
}

pub struct M10Service<'a> {
    store: FileRecordStore,
    capabilities: CapabilityIssuer,
    affairs: &'a dyn AffairsInvocationPort,
    operator_grant_id: WireText,
}

impl<'a> M10Service<'a> {
    pub fn new(
        store: FileRecordStore,
        capabilities: CapabilityIssuer,
        affairs: &'a dyn AffairsInvocationPort,
        operator_grant_id: WireText,
    ) -> Self {
        Self {
            store,
            capabilities,
            affairs,
            operator_grant_id,
        }
    }

    pub fn submit<P: M10AdmissionPorts>(
        &self,
        request: &SubmitAffairsGetDto,
        ports: &mut P,
        now_ms: i64,
    ) -> ClientResponseDto {
        let expected_digest = match affairs_get_payload_digest(&request.procedure_id, request.as_of)
        {
            Ok(value) => value,
            Err(_) => return malformed_command_error(),
        };
        if !constant_time_eq(
            request.payload_digest.as_str().as_bytes(),
            expected_digest.as_str().as_bytes(),
        ) {
            return malformed_command_error();
        }
        let command = match build_command(request) {
            Ok(value) => value,
            Err(error) => return internal_error(error),
        };
        let staged = ports.staged_operation();
        let staged_identity = staged.snapshot_identity().clone();
        match RequestAdmissionCoordinator.admit(&command, ports) {
            M00AdmissionResult::Rejected(rejection)
            | M00AdmissionResult::PriorRejected(rejection) => ClientResponseDto::Error {
                error: project_rejection(&rejection),
            },
            M00AdmissionResult::Incomplete(value) => ClientResponseDto::Incomplete {
                command_id: wire(value.command_id().as_str()),
                retry_not_before: UnixMillis::new(
                    i64::try_from(value.retry_not_before().as_unix_millis()).unwrap_or(i64::MAX),
                ),
            },
            M00AdmissionResult::Admitted { disposition, .. }
            | M00AdmissionResult::PriorAdmitted(disposition) => {
                if disposition.descriptor_snapshot_id() != &staged_identity {
                    return internal_error("descriptor identity drift");
                }
                self.process_admitted(request, &disposition, now_ms)
            }
        }
    }

    pub fn lookup(&self, command_id: &str, viewer: &ViewerAuthorizationDto) -> ClientResponseDto {
        let Ok(Some(record)) = self.store.get(command_id) else {
            return ClientResponseDto::Unavailable;
        };
        let redaction = match (&record.read_policy, viewer) {
            (
                StoredReadPolicy::Public { authorization },
                ViewerAuthorizationDto::PublicCapability { capability },
            ) if self.capabilities.verify(authorization, capability.as_str()) => {
                RedactionDto::Public
            }
            (
                StoredReadPolicy::Authenticated { tenant_id, user_id },
                ViewerAuthorizationDto::AuthenticatedOwner {
                    tenant_id: presented_tenant,
                    user_id: presented_user,
                },
            ) if constant_time_eq(tenant_id.as_bytes(), presented_tenant.as_str().as_bytes())
                && constant_time_eq(user_id.as_bytes(), presented_user.as_str().as_bytes()) =>
            {
                RedactionDto::AuthenticatedOwner
            }
            (_, ViewerAuthorizationDto::Operator { grant_id })
                if constant_time_eq(
                    self.operator_grant_id.as_str().as_bytes(),
                    grant_id.as_str().as_bytes(),
                ) =>
            {
                RedactionDto::Operator
            }
            _ => return ClientResponseDto::Unavailable,
        };
        match &record.state {
            RecordState::Terminal { terminal, .. } => ClientResponseDto::Available {
                command_id: wire(command_id),
                terminal: terminal.clone(),
                redaction,
            },
            RecordState::Pending { .. } | RecordState::Claimed { .. } => {
                ClientResponseDto::Incomplete {
                    command_id: wire(command_id),
                    retry_not_before: UnixMillis::new(0),
                }
            }
        }
    }

    #[must_use]
    pub fn capabilities(&self) -> &CapabilityIssuer {
        &self.capabilities
    }

    fn process_admitted(
        &self,
        request: &SubmitAffairsGetDto,
        disposition: &ustc_campus_agent_core::request_context::M00AdmittedDisposition,
        now_ms: i64,
    ) -> ClientResponseDto {
        let body = match build_capsule(request, disposition) {
            Ok(value) => value,
            Err(error) => return internal_error(error),
        };
        let command_id = disposition.command_id().as_str();
        let capsule_digest = match crate::persistence::capsule_digest(&body) {
            Ok(value) => value,
            Err(_) => return infrastructure_error("m10_store_unavailable"),
        };
        let (candidate_bearer, candidate_policy) = match disposition.admitted_actor() {
            M00AdmittedActor::Public => match self.capabilities.mint(command_id, &capsule_digest) {
                Ok((bearer, authorization)) => {
                    (Some(bearer), StoredReadPolicy::Public { authorization })
                }
                Err(_) => return internal_error("public capability mint failed"),
            },
            M00AdmittedActor::Authenticated(ids) => (
                None,
                StoredReadPolicy::Authenticated {
                    tenant_id: ids.tenant_id().as_str().to_owned(),
                    user_id: ids.user_id().as_str().to_owned(),
                },
            ),
        };
        let (record, response_bearer) =
            match self
                .store
                .insert_admitted_once(command_id, body, candidate_policy)
            {
                Ok(InsertOutcome::Created) => {
                    let Some(record) = self.store.get(command_id).ok().flatten() else {
                        return internal_error("m10_post_insert_record_missing");
                    };
                    (record, candidate_bearer)
                }
                Ok(InsertOutcome::Existing(record)) => {
                    let record = *record;
                    let bearer = match reproduce_bearer(&self.capabilities, command_id, &record) {
                        Ok(bearer) => bearer,
                        Err(code) => return infrastructure_error(code),
                    };
                    (record, bearer)
                }
                Ok(InsertOutcome::InvariantCorruption) => {
                    return internal_error("m10_internal_invariant");
                }
                Err(_) => return infrastructure_error("m10_store_unavailable"),
            };
        if let RecordState::Terminal { terminal, .. } = record.state {
            return ClientResponseDto::Accepted {
                command_id: wire(command_id),
                terminal,
                public_capability: response_bearer,
            };
        }
        let token = match self.store.claim(command_id, now_ms, 30_000) {
            Ok(ClaimOutcome::Claimed(token)) => token,
            Ok(ClaimOutcome::AlreadyTerminal(record)) => {
                let RecordState::Terminal { terminal, .. } = &record.state else {
                    return internal_error("terminal state mismatch");
                };
                let public_capability =
                    match reproduce_bearer(&self.capabilities, command_id, &record) {
                        Ok(cap) => cap,
                        Err(code) => return infrastructure_error(code),
                    };
                return ClientResponseDto::Accepted {
                    command_id: wire(command_id),
                    terminal: terminal.clone(),
                    public_capability,
                };
            }
            Ok(ClaimOutcome::Busy) => {
                return ClientResponseDto::Incomplete {
                    command_id: wire(command_id),
                    retry_not_before: UnixMillis::new(now_ms.saturating_add(30_000)),
                };
            }
            Ok(ClaimOutcome::Missing) => {
                return internal_error("m10_post_insert_record_missing");
            }
            Err(_) => return infrastructure_error("m10_store_unavailable"),
        };
        let procedure_id = match ProcedureId::parse(request.procedure_id.as_str()) {
            Ok(value) => value,
            Err(_) => {
                if self.store.abandon(&token).is_err() {
                    return infrastructure_error("m10_store_unavailable");
                }
                return internal_error("procedure id invalid after wire validation");
            }
        };
        let as_of = match request.as_of {
            Some(value) => {
                match OffsetDateTime::from_unix_timestamp_nanos(i128::from(value.get()) * 1_000_000)
                {
                    Ok(value) => Some(value),
                    Err(_) => {
                        if self.store.abandon(&token).is_err() {
                            return infrastructure_error("m10_store_unavailable");
                        }
                        return internal_error("as_of is out of range");
                    }
                }
            }
            None => None,
        };
        let query = AffairsGetQuery::new(procedure_id, as_of);
        let receipt = match self.affairs.invoke(disposition.admitted_actor(), &query) {
            Ok(value) => value,
            Err(error) => {
                if self.store.abandon(&token).is_err() {
                    return infrastructure_error("m10_store_unavailable");
                }
                return map_invocation_error(error);
            }
        };
        let terminal = match project_receipt(&receipt) {
            Ok(value) => value,
            Err(_) => {
                if self.store.abandon(&token).is_err() {
                    return infrastructure_error("m10_store_unavailable");
                }
                return internal_error("M71 projection invariant");
            }
        };
        match self.store.complete(&token, terminal.clone()) {
            Ok(CompleteOutcome::Completed(record))
            | Ok(CompleteOutcome::AlreadyTerminal(record)) => {
                let RecordState::Terminal { terminal, .. } = &record.state else {
                    return internal_error("terminal state mismatch");
                };
                let public_capability =
                    match reproduce_bearer(&self.capabilities, command_id, &record) {
                        Ok(cap) => cap.or(response_bearer),
                        Err(code) => return infrastructure_error(code),
                    };
                ClientResponseDto::Accepted {
                    command_id: wire(command_id),
                    terminal: terminal.clone(),
                    public_capability,
                }
            }
            Ok(CompleteOutcome::LostToWinner(record)) => {
                let RecordState::Terminal { terminal, .. } = &record.state else {
                    return internal_error("winner state mismatch");
                };
                let public_capability =
                    match reproduce_bearer(&self.capabilities, command_id, &record) {
                        Ok(cap) => cap,
                        Err(code) => return infrastructure_error(code),
                    };
                ClientResponseDto::Accepted {
                    command_id: wire(command_id),
                    terminal: terminal.clone(),
                    public_capability,
                }
            }
            Ok(CompleteOutcome::Stale) => ClientResponseDto::Incomplete {
                command_id: wire(command_id),
                retry_not_before: UnixMillis::new(now_ms.saturating_add(30_000)),
            },
            Ok(CompleteOutcome::Missing) => internal_error("m10_post_claim_record_missing"),
            Err(_) => infrastructure_error("m10_store_unavailable"),
        }
    }
}

fn build_command(
    request: &SubmitAffairsGetDto,
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
    let operation_id = OperationId::parse("affairs.get").map_err(|_| "operation id")?;
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

fn build_capsule(
    request: &SubmitAffairsGetDto,
    disposition: &ustc_campus_agent_core::request_context::M00AdmittedDisposition,
) -> Result<DispatchCapsuleBodyV2, &'static str> {
    let actor = match disposition.admitted_actor() {
        M00AdmittedActor::Public => AdmittedActorDto::Public,
        M00AdmittedActor::Authenticated(ids) => AdmittedActorDto::Authenticated {
            tenant_id: wire(ids.tenant_id().as_str()),
            user_id: wire(ids.user_id().as_str()),
            session_id: wire(ids.session_id().as_str()),
        },
    };
    let frozen = disposition.frozen_prerequisites();
    let frozen_dto = FrozenPrerequisitesDto {
        policy_snapshot_id: wire(frozen.policy_snapshot_id().as_str()),
        observed_at: UnixMillis::new(
            i64::try_from(frozen.observed_at().as_unix_millis()).map_err(|_| "observed_at")?,
        ),
        session_id: frozen.session_id().map(|value| wire(value.as_str())),
        admitted_operation_id: wire(frozen.admitted_operation_id().as_str()),
    };
    DispatchCapsuleBodyV2::try_new(
        wire(disposition.command_id().as_str()),
        wire(disposition.correlation_id().as_str()),
        actor,
        AffairsGetPayloadDto {
            procedure_id: request.procedure_id.clone(),
            as_of: request.as_of,
        },
        wire(disposition.descriptor_snapshot_id().as_str()),
        wire(
            disposition
                .descriptor_snapshot_id()
                .content_digest()
                .as_str(),
        ),
        disposition.descriptor_snapshot_id().snapshot_version(),
        frozen_dto,
    )
    .map_err(|_| "capsule validation")
}

fn reproduce_bearer(
    issuer: &CapabilityIssuer,
    command_id: &str,
    record: &StoredRecord,
) -> Result<Option<WireText>, &'static str> {
    let StoredReadPolicy::Public { authorization } = &record.read_policy else {
        return Ok(None);
    };
    issuer
        .reproduce(authorization, command_id, &record.capsule_digest)
        .map(Some)
        .map_err(|_| "m10_capability_reproduction_failed")
}

fn wire(value: &str) -> WireText {
    WireText::parse(value).unwrap_or_else(|_| WireText::fallback())
}

fn map_invocation_error(error: AffairsInvocationError) -> ClientResponseDto {
    match error {
        AffairsInvocationError::Downstream(error) => map_m71_error(error),
        AffairsInvocationError::Denied => invocation_denied_error(),
        AffairsInvocationError::Unavailable => {
            infrastructure_error("affairs_invocation_unavailable")
        }
        AffairsInvocationError::Internal => internal_error("affairs_invocation_internal"),
    }
}

fn invocation_denied_error() -> ClientResponseDto {
    let error = match M10WireErrorDto::try_new(
        WireErrorClassDto::PolicyDenied,
        RetryabilityDto::NotRetryable,
        wire("policy_denied"),
        EchoPayloadDto::PolicyDenied {
            operation_id: wire("affairs.get"),
            permission_class: wire("public_read"),
        },
    ) {
        Ok(value) => value,
        Err(_) => return internal_error("m10_invocation_denial_projection"),
    };
    ClientResponseDto::Error {
        error: ClientErrorDto::Admission { error },
    }
}

fn map_m71_error(error: GetProcedureError) -> ClientResponseDto {
    let code = match error {
        GetProcedureError::M60StoreUnavailable | GetProcedureError::PersistenceUnavailable => {
            "affairs_infrastructure_retry"
        }
        GetProcedureError::M60StoreCorrupted
        | GetProcedureError::StoreCorrupted
        | GetProcedureError::JournalCorrupted
        | GetProcedureError::InternalInconsistent => "affairs_infrastructure_failure",
    };
    infrastructure_error(code)
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
    let echo = EchoPayloadDto::Operation {
        operation_id: wire("affairs.get"),
    };
    let error = match M10WireErrorDto::try_new(
        WireErrorClassDto::MalformedCommand,
        RetryabilityDto::RetryableAfterChange,
        wire("malformed_command"),
        echo,
    ) {
        Ok(value) => value,
        Err(_) => return internal_error("m10_malformed_command_projection"),
    };
    ClientResponseDto::Error {
        error: ClientErrorDto::Admission { error },
    }
}
