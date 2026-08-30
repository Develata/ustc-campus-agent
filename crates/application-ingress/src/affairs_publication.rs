use affairs_navigator::{ProcedureId, ProcedurePublicationError, ProcedurePublicationReceipt};
use sha2::{Digest, Sha256};
use ustc_campus_agent_core::control_evidence::{
    ControlEvidenceAppendOutcome, ControlEvidenceAppendPort, ControlEvidenceJournalError,
    ControlEvidenceKey, PlatformControlActor, PlatformControlError, PlatformControlEvent,
};
use ustc_campus_agent_core::identity::{CommandId, CorrelationId, RequestId};
use ustc_campus_agent_core::request_context::{
    ActorReference, BuildRequestContextCommand, CausationId, ClientProvenance, EffectClass,
    IdempotencyKey, M00AdmissionResult, M00AdmittedActor, OperationId, PayloadDigest,
    PermissionClass, RequestAdmissionCoordinator,
};
use ustc_campus_agent_core::session::SessionInstant;

use crate::capability::constant_time_eq;
use crate::service::M10AdmissionPorts;

const PUBLICATION_OPERATION_ID: &str = "affairs.publish";
const PAYLOAD_DOMAIN: &[u8] = b"affairs-publication-payload/v1\0";

/// Internal M10 administrator command. It carries intent, never admitted authority.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AffairsPublicationCommand {
    request_id: RequestId,
    actor_reference: ActorReference,
    correlation_id: CorrelationId,
    causation_id: Option<CausationId>,
    idempotency_key: Option<IdempotencyKey>,
    provenance: ClientProvenance,
    payload_digest: PayloadDigest,
    procedure_id: ProcedureId,
    expected_publication_revision: Option<u64>,
}

impl AffairsPublicationCommand {
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn new(
        request_id: RequestId,
        actor_reference: ActorReference,
        correlation_id: CorrelationId,
        causation_id: Option<CausationId>,
        idempotency_key: Option<IdempotencyKey>,
        provenance: ClientProvenance,
        payload_digest: PayloadDigest,
        procedure_id: ProcedureId,
        expected_publication_revision: Option<u64>,
    ) -> Self {
        Self {
            request_id,
            actor_reference,
            correlation_id,
            causation_id,
            idempotency_key,
            provenance,
            payload_digest,
            procedure_id,
            expected_publication_revision,
        }
    }

    #[must_use]
    pub const fn procedure_id(&self) -> &ProcedureId {
        &self.procedure_id
    }

    #[must_use]
    pub const fn expected_publication_revision(&self) -> Option<u64> {
        self.expected_publication_revision
    }
}

/// Direct M10-to-owning-M71 application port for the admitted administrator mutation.
pub trait AffairsPublicationApplicationPort: Send {
    fn publish(
        &mut self,
        command_id: &CommandId,
        actor: &M00AdmittedActor,
        procedure_id: &ProcedureId,
        expected_publication_revision: Option<u64>,
    ) -> Result<ProcedurePublicationReceipt, AffairsPublicationApplicationError>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AffairsPublicationApplicationError {
    Denied,
    Unavailable,
    Downstream(ProcedurePublicationError),
    Internal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AffairsPublicationEvidenceError {
    Unavailable,
    Corrupt,
    LimitExceeded,
    InternalInvariant,
    Conflict,
    Missing,
    Incoherent,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AffairsPublicationOutcome {
    Published(ProcedurePublicationReceipt),
    Rejected(PlatformControlError),
    Incomplete {
        command_id: CommandId,
        retry_not_before: SessionInstant,
    },
    MalformedCommand,
    EvidenceRejected(AffairsPublicationEvidenceError),
    PublicationRejected(AffairsPublicationApplicationError),
    InternalInvariant,
}

pub struct M10AffairsPublicationService<'a> {
    publication: &'a mut dyn AffairsPublicationApplicationPort,
    evidence: &'a mut dyn ControlEvidenceAppendPort,
}

impl<'a> M10AffairsPublicationService<'a> {
    #[must_use]
    pub fn new(
        publication: &'a mut dyn AffairsPublicationApplicationPort,
        evidence: &'a mut dyn ControlEvidenceAppendPort,
    ) -> Self {
        Self {
            publication,
            evidence,
        }
    }

    #[must_use]
    pub fn submit<P: M10AdmissionPorts>(
        &mut self,
        request: &AffairsPublicationCommand,
        ports: &mut P,
    ) -> AffairsPublicationOutcome {
        let expected_digest = affairs_publication_payload_digest(
            request.procedure_id(),
            request.expected_publication_revision(),
        );
        if !constant_time_eq(
            request.payload_digest.as_str().as_bytes(),
            expected_digest.as_str().as_bytes(),
        ) {
            return AffairsPublicationOutcome::MalformedCommand;
        }
        let command = match build_command(request) {
            Ok(value) => value,
            Err(()) => return AffairsPublicationOutcome::MalformedCommand,
        };
        let staged = ports.staged_operation();
        if !publication_descriptor_is_valid(staged.as_ref()) {
            return AffairsPublicationOutcome::InternalInvariant;
        }
        let staged_identity = staged.snapshot_identity().clone();

        let disposition = match RequestAdmissionCoordinator.admit(&command, ports) {
            M00AdmissionResult::Rejected(rejection)
            | M00AdmissionResult::PriorRejected(rejection) => {
                return AffairsPublicationOutcome::Rejected(
                    PlatformControlError::from_admission_rejection(&rejection),
                );
            }
            M00AdmissionResult::Incomplete(value) => {
                return AffairsPublicationOutcome::Incomplete {
                    command_id: value.command_id().clone(),
                    retry_not_before: value.retry_not_before(),
                };
            }
            M00AdmissionResult::Admitted {
                context,
                disposition,
            } => {
                if disposition.descriptor_snapshot_id() != &staged_identity
                    || context.operation().descriptor_snapshot_id() != &staged_identity
                    || context.operation().operation_id().as_str() != PUBLICATION_OPERATION_ID
                    || context.operation().permission_class() != PermissionClass::TenantPrivateWrite
                    || context.operation().effect_class() != EffectClass::TenantLocalMutation
                {
                    return AffairsPublicationOutcome::InternalInvariant;
                }
                let event = PlatformControlEvent::from_admitted_request(&context);
                match self.evidence.append_once(&event) {
                    Ok(ControlEvidenceAppendOutcome::Appended)
                    | Ok(ControlEvidenceAppendOutcome::AlreadySame) => {}
                    Ok(ControlEvidenceAppendOutcome::Conflict) => {
                        return AffairsPublicationOutcome::EvidenceRejected(
                            AffairsPublicationEvidenceError::Conflict,
                        );
                    }
                    Err(error) => {
                        return AffairsPublicationOutcome::EvidenceRejected(map_journal_error(
                            error,
                        ));
                    }
                }
                disposition
            }
            M00AdmissionResult::PriorAdmitted(disposition) => {
                if disposition.descriptor_snapshot_id() != &staged_identity
                    || disposition
                        .frozen_prerequisites()
                        .admitted_operation_id()
                        .as_str()
                        != PUBLICATION_OPERATION_ID
                {
                    return AffairsPublicationOutcome::InternalInvariant;
                }
                let key = ControlEvidenceKey::Request {
                    command_id: disposition.command_id().clone(),
                };
                let event = match self.evidence.load_control_event(&key) {
                    Ok(Some(event)) => event,
                    Ok(None) => {
                        return AffairsPublicationOutcome::EvidenceRejected(
                            AffairsPublicationEvidenceError::Missing,
                        );
                    }
                    Err(error) => {
                        return AffairsPublicationOutcome::EvidenceRejected(map_journal_error(
                            error,
                        ));
                    }
                };
                if !prior_event_is_coherent(&event, &disposition, &staged_identity) {
                    return AffairsPublicationOutcome::EvidenceRejected(
                        AffairsPublicationEvidenceError::Incoherent,
                    );
                }
                disposition
            }
        };

        match self.publication.publish(
            disposition.command_id(),
            disposition.admitted_actor(),
            request.procedure_id(),
            request.expected_publication_revision(),
        ) {
            Ok(receipt) => AffairsPublicationOutcome::Published(receipt),
            Err(error) => AffairsPublicationOutcome::PublicationRejected(error),
        }
    }
}

#[must_use]
pub fn affairs_publication_payload_digest(
    procedure_id: &ProcedureId,
    expected_publication_revision: Option<u64>,
) -> PayloadDigest {
    let bytes = procedure_id.as_str().as_bytes();
    let mut hasher = Sha256::new();
    hasher.update(PAYLOAD_DOMAIN);
    hasher.update(u64::try_from(bytes.len()).unwrap_or(u64::MAX).to_be_bytes());
    hasher.update(bytes);
    match expected_publication_revision {
        None => hasher.update([0]),
        Some(revision) => {
            hasher.update([1]);
            hasher.update(revision.to_be_bytes());
        }
    }
    PayloadDigest::parse(format!("{:x}", hasher.finalize()))
        .expect("SHA-256 output is valid payload-digest grammar")
}

fn build_command(request: &AffairsPublicationCommand) -> Result<BuildRequestContextCommand, ()> {
    let operation_id = OperationId::parse(PUBLICATION_OPERATION_ID).map_err(|_| ())?;
    Ok(BuildRequestContextCommand::new(
        request.request_id.clone(),
        operation_id,
        request.actor_reference.clone(),
        request.correlation_id.clone(),
        request.causation_id.clone(),
        request.idempotency_key.clone(),
        request.provenance.clone(),
        request.payload_digest.clone(),
    ))
}

fn publication_descriptor_is_valid(
    descriptor: &dyn ustc_campus_agent_core::request_context::OperationDescriptorProjection,
) -> bool {
    descriptor.operation_id().as_str() == PUBLICATION_OPERATION_ID
        && descriptor.permission_class() == PermissionClass::TenantPrivateWrite
        && descriptor.effect_class() == EffectClass::TenantLocalMutation
}

fn prior_event_is_coherent(
    event: &PlatformControlEvent,
    disposition: &ustc_campus_agent_core::request_context::M00AdmittedDisposition,
    staged_identity: &ustc_campus_agent_core::request_context::DescriptorSnapshotId,
) -> bool {
    let PlatformControlEvent::RequestAdmitted {
        command_id,
        correlation_id,
        actor,
        operation_id,
        descriptor_snapshot_id,
        permission_class,
        effect_class,
        policy_snapshot_id,
        observed_at,
        ..
    } = event
    else {
        return false;
    };
    command_id == disposition.command_id()
        && correlation_id == disposition.correlation_id()
        && actor_matches(actor, disposition.admitted_actor())
        && operation_id.as_str() == PUBLICATION_OPERATION_ID
        && descriptor_snapshot_id == staged_identity
        && permission_class == &PermissionClass::TenantPrivateWrite
        && effect_class == &EffectClass::TenantLocalMutation
        && policy_snapshot_id == disposition.frozen_prerequisites().policy_snapshot_id()
        && observed_at == &disposition.frozen_prerequisites().observed_at()
}

fn actor_matches(actor: &PlatformControlActor, admitted: &M00AdmittedActor) -> bool {
    match (actor, admitted) {
        (PlatformControlActor::Public, M00AdmittedActor::Public) => true,
        (
            PlatformControlActor::Authenticated {
                tenant_id,
                user_id,
                session_id,
            },
            M00AdmittedActor::Authenticated(ids),
        ) => {
            tenant_id == ids.tenant_id()
                && user_id == ids.user_id()
                && session_id == ids.session_id()
        }
        _ => false,
    }
}

const fn map_journal_error(error: ControlEvidenceJournalError) -> AffairsPublicationEvidenceError {
    match error {
        ControlEvidenceJournalError::Unavailable => AffairsPublicationEvidenceError::Unavailable,
        ControlEvidenceJournalError::Corrupt => AffairsPublicationEvidenceError::Corrupt,
        ControlEvidenceJournalError::LimitExceeded => {
            AffairsPublicationEvidenceError::LimitExceeded
        }
        ControlEvidenceJournalError::InternalInvariant => {
            AffairsPublicationEvidenceError::InternalInvariant
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn payload_digest_distinguishes_optional_revision() {
        let procedure = ProcedureId::parse("procedure:demo").expect("fixture id");
        let absent = affairs_publication_payload_digest(&procedure, None);
        let revision_one = affairs_publication_payload_digest(&procedure, Some(1));
        let revision_two = affairs_publication_payload_digest(&procedure, Some(2));
        assert_ne!(absent, revision_one);
        assert_ne!(revision_one, revision_two);
        assert_eq!(
            revision_one,
            affairs_publication_payload_digest(&procedure, Some(1))
        );
    }
}
