use sha2::{Digest, Sha256};
use ustc_campus_agent_change_radar::{
    ChangeEventId, ChangePublicationError, ChangeReviewReceiptId, PublishedChangeEvent,
};
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
use ustc_campus_agent_core::source_revision::RevisionTimestamp;

use crate::capability::constant_time_eq;
use crate::service::M10AdmissionPorts;

const PUBLICATION_OPERATION_ID: &str = "change.publish";
const PAYLOAD_DOMAIN: &[u8] = b"change-publication-payload/v1\0";

/// Internal M10 administrator command. It carries exact intent, never admitted authority.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChangePublicationCommand {
    request_id: RequestId,
    actor_reference: ActorReference,
    correlation_id: CorrelationId,
    causation_id: Option<CausationId>,
    idempotency_key: Option<IdempotencyKey>,
    provenance: ClientProvenance,
    payload_digest: PayloadDigest,
    event_id: ChangeEventId,
    review_receipt_id: String,
    reviewed_at: RevisionTimestamp,
    published_at: RevisionTimestamp,
}

impl ChangePublicationCommand {
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
        event_id: ChangeEventId,
        review_receipt_id: &ChangeReviewReceiptId,
        reviewed_at: RevisionTimestamp,
        published_at: RevisionTimestamp,
    ) -> Self {
        Self {
            request_id,
            actor_reference,
            correlation_id,
            causation_id,
            idempotency_key,
            provenance,
            payload_digest,
            event_id,
            review_receipt_id: review_receipt_id.as_str().to_owned(),
            reviewed_at,
            published_at,
        }
    }

    #[must_use]
    pub const fn event_id(&self) -> &ChangeEventId {
        &self.event_id
    }

    #[must_use]
    pub fn review_receipt_id(&self) -> &str {
        &self.review_receipt_id
    }

    #[must_use]
    pub const fn reviewed_at(&self) -> RevisionTimestamp {
        self.reviewed_at
    }

    #[must_use]
    pub const fn published_at(&self) -> RevisionTimestamp {
        self.published_at
    }
}

/// Direct M10-to-owning-M70 port for the admitted administrator mutation.
pub trait ChangePublicationApplicationPort: Send {
    fn publish(
        &mut self,
        command_id: &CommandId,
        actor: &M00AdmittedActor,
        event_id: &ChangeEventId,
        review_receipt_id: &str,
        reviewed_at: RevisionTimestamp,
        published_at: RevisionTimestamp,
    ) -> Result<PublishedChangeEvent, ChangePublicationApplicationError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChangePublicationApplicationError {
    Denied,
    Unavailable,
    Downstream(ChangePublicationError),
    Internal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChangePublicationEvidenceError {
    Unavailable,
    Corrupt,
    LimitExceeded,
    InternalInvariant,
    Conflict,
    Missing,
    Incoherent,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChangePublicationOutcome {
    Published(Box<PublishedChangeEvent>),
    Rejected(PlatformControlError),
    Incomplete {
        command_id: CommandId,
        retry_not_before: SessionInstant,
    },
    MalformedCommand,
    EvidenceRejected(ChangePublicationEvidenceError),
    PublicationRejected(ChangePublicationApplicationError),
    InternalInvariant,
}

pub struct M10ChangePublicationService<'a> {
    publication: &'a mut dyn ChangePublicationApplicationPort,
    evidence: &'a mut dyn ControlEvidenceAppendPort,
}

impl<'a> M10ChangePublicationService<'a> {
    #[must_use]
    pub fn new(
        publication: &'a mut dyn ChangePublicationApplicationPort,
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
        request: &ChangePublicationCommand,
        ports: &mut P,
    ) -> ChangePublicationOutcome {
        let expected_digest = change_publication_payload_digest(
            request.event_id(),
            request.review_receipt_id(),
            request.reviewed_at(),
            request.published_at(),
        );
        if !constant_time_eq(
            request.payload_digest.as_str().as_bytes(),
            expected_digest.as_str().as_bytes(),
        ) {
            return ChangePublicationOutcome::MalformedCommand;
        }
        let command = match build_command(request) {
            Ok(value) => value,
            Err(()) => return ChangePublicationOutcome::MalformedCommand,
        };
        let staged = ports.staged_operation();
        if !publication_descriptor_is_valid(staged.as_ref()) {
            return ChangePublicationOutcome::InternalInvariant;
        }
        let staged_identity = staged.snapshot_identity().clone();

        let disposition = match RequestAdmissionCoordinator.admit(&command, ports) {
            M00AdmissionResult::Rejected(rejection)
            | M00AdmissionResult::PriorRejected(rejection) => {
                return ChangePublicationOutcome::Rejected(
                    PlatformControlError::from_admission_rejection(&rejection),
                );
            }
            M00AdmissionResult::Incomplete(value) => {
                return ChangePublicationOutcome::Incomplete {
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
                    return ChangePublicationOutcome::InternalInvariant;
                }
                let event = PlatformControlEvent::from_admitted_request(&context);
                match self.evidence.append_once(&event) {
                    Ok(ControlEvidenceAppendOutcome::Appended)
                    | Ok(ControlEvidenceAppendOutcome::AlreadySame) => {}
                    Ok(ControlEvidenceAppendOutcome::Conflict) => {
                        return ChangePublicationOutcome::EvidenceRejected(
                            ChangePublicationEvidenceError::Conflict,
                        );
                    }
                    Err(error) => {
                        return ChangePublicationOutcome::EvidenceRejected(map_journal_error(
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
                    return ChangePublicationOutcome::InternalInvariant;
                }
                let key = ControlEvidenceKey::Request {
                    command_id: disposition.command_id().clone(),
                };
                let event = match self.evidence.load_control_event(&key) {
                    Ok(Some(event)) => event,
                    Ok(None) => {
                        return ChangePublicationOutcome::EvidenceRejected(
                            ChangePublicationEvidenceError::Missing,
                        );
                    }
                    Err(error) => {
                        return ChangePublicationOutcome::EvidenceRejected(map_journal_error(
                            error,
                        ));
                    }
                };
                if !prior_event_is_coherent(&event, &disposition, &staged_identity) {
                    return ChangePublicationOutcome::EvidenceRejected(
                        ChangePublicationEvidenceError::Incoherent,
                    );
                }
                disposition
            }
        };

        match self.publication.publish(
            disposition.command_id(),
            disposition.admitted_actor(),
            request.event_id(),
            request.review_receipt_id(),
            request.reviewed_at(),
            request.published_at(),
        ) {
            Ok(receipt) => ChangePublicationOutcome::Published(Box::new(receipt)),
            Err(error) => ChangePublicationOutcome::PublicationRejected(error),
        }
    }
}

#[must_use]
pub fn change_publication_payload_digest(
    event_id: &ChangeEventId,
    review_receipt_id: &str,
    reviewed_at: RevisionTimestamp,
    published_at: RevisionTimestamp,
) -> PayloadDigest {
    let mut hasher = Sha256::new();
    hasher.update(PAYLOAD_DOMAIN);
    for bytes in [event_id.as_str().as_bytes(), review_receipt_id.as_bytes()] {
        hasher.update(u64::try_from(bytes.len()).unwrap_or(u64::MAX).to_be_bytes());
        hasher.update(bytes);
    }
    hasher.update(reviewed_at.unix_seconds().to_be_bytes());
    hasher.update(b"unpublished-or-exact-retry");
    hasher.update(published_at.unix_seconds().to_be_bytes());
    PayloadDigest::parse(format!("{:x}", hasher.finalize()))
        .expect("SHA-256 output is valid payload-digest grammar")
}

fn build_command(request: &ChangePublicationCommand) -> Result<BuildRequestContextCommand, ()> {
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

const fn map_journal_error(error: ControlEvidenceJournalError) -> ChangePublicationEvidenceError {
    match error {
        ControlEvidenceJournalError::Unavailable => ChangePublicationEvidenceError::Unavailable,
        ControlEvidenceJournalError::Corrupt => ChangePublicationEvidenceError::Corrupt,
        ControlEvidenceJournalError::LimitExceeded => ChangePublicationEvidenceError::LimitExceeded,
        ControlEvidenceJournalError::InternalInvariant => {
            ChangePublicationEvidenceError::InternalInvariant
        }
    }
}
