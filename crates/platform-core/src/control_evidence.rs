//! Stable, redacted, data-only M00 control evidence.
//!
//! These projections carry no authentication, authorization, repository-currentness, publication,
//! or durable-append authority. Decoding a value proves only that its closed data shape is valid.
//!
//! An evidence value cannot be converted into request authority:
//!
//! ```compile_fail
//! use ustc_campus_agent_core::control_evidence::PlatformControlEvent;
//! use ustc_campus_agent_core::request_context::PlatformRequestContext;
//!
//! fn promote(event: PlatformControlEvent) -> PlatformRequestContext {
//!     event.into()
//! }
//! ```
//!
//! Evidence cannot be defaulted into existence:
//!
//! ```compile_fail
//! use ustc_campus_agent_core::control_evidence::PlatformControlEvent;
//!
//! let event = PlatformControlEvent::default();
//! ```
//!
//! Lower-level diagnostic text has no constructor:
//!
//! ```compile_fail
//! use ustc_campus_agent_core::control_evidence::PlatformControlError;
//!
//! let error = PlatformControlError::from("credential-canary");
//! ```
//!
//! The error code field is private:
//!
//! ```compile_fail
//! use ustc_campus_agent_core::control_evidence::{
//!     PlatformControlError, PlatformControlErrorCode,
//! };
//!
//! let error = PlatformControlError {
//!     code: PlatformControlErrorCode::MalformedExternalInput,
//! };
//! ```

use serde::{Deserialize, Serialize};

use crate::identity::{CommandId, CorrelationId, RequestId, SessionId, TenantId, UserId};
use crate::request_context::{
    AdmissionRejectionClass, CausationId, DescriptorSnapshotId, EffectClass, M00AdmittedActor,
    OperationId, PermissionClass, PlatformPolicySnapshotId, PlatformRequestContext,
    RequestContextRejection,
};
use crate::session::{
    AuthAdapterId, SessionDomainError, SessionEvent, SessionExpiryCause, SessionInstant,
};
use crate::session_port::SessionRepositoryError;

/// Redacted actor data retained in one admitted-request evidence event.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum PlatformControlActor {
    /// A public request has no synthetic tenant, user, or session identity.
    Public,
    /// Exact identities retained from an admitted session snapshot.
    Authenticated {
        tenant_id: TenantId,
        user_id: UserId,
        session_id: SessionId,
    },
}

/// Stable event classification for external control evidence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlatformControlEventKind {
    SessionOpened,
    SessionRefreshed,
    SessionExpired,
    SessionRevoked,
    RequestAdmitted,
}

/// Stable deduplication identity; constructing a key grants no authority.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ControlEvidenceKey {
    Session {
        session_id: SessionId,
        sequence: u64,
    },
    Request {
        command_id: CommandId,
    },
}

/// Stable, redacted, data-only control evidence.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum PlatformControlEvent {
    SessionOpened {
        session_id: SessionId,
        sequence: u64,
        tenant_id: TenantId,
        user_id: UserId,
        auth_adapter_id: AuthAdapterId,
        opened_at: SessionInstant,
    },
    SessionRefreshed {
        session_id: SessionId,
        sequence: u64,
        refreshed_at: SessionInstant,
        effective_expires_at: SessionInstant,
    },
    SessionExpired {
        session_id: SessionId,
        sequence: u64,
        expired_at: SessionInstant,
        observed_at: SessionInstant,
        cause: SessionExpiryCause,
    },
    SessionRevoked {
        session_id: SessionId,
        sequence: u64,
        revoked_at: SessionInstant,
    },
    RequestAdmitted {
        request_id: RequestId,
        command_id: CommandId,
        correlation_id: CorrelationId,
        causation_id: Option<CausationId>,
        actor: PlatformControlActor,
        operation_id: OperationId,
        descriptor_snapshot_id: DescriptorSnapshotId,
        permission_class: PermissionClass,
        effect_class: EffectClass,
        policy_snapshot_id: PlatformPolicySnapshotId,
        observed_at: SessionInstant,
    },
}

impl PlatformControlEvent {
    /// Projects one checked session-domain event without credential evidence.
    #[must_use]
    pub fn from_session_event(event: &SessionEvent) -> Self {
        match event {
            SessionEvent::Opened(value) => {
                let evidence = value.credential_evidence();
                Self::SessionOpened {
                    session_id: value.session_id().clone(),
                    sequence: value.sequence(),
                    tenant_id: evidence.tenant_id().clone(),
                    user_id: evidence.user_id().clone(),
                    auth_adapter_id: evidence.auth_adapter_id().clone(),
                    opened_at: value.opened_at(),
                }
            }
            SessionEvent::Refreshed(value) => Self::SessionRefreshed {
                session_id: value.session_id().clone(),
                sequence: value.sequence(),
                refreshed_at: value.observed_at(),
                effective_expires_at: value.effective_expires_at(),
            },
            SessionEvent::Expired(value) => Self::SessionExpired {
                session_id: value.session_id().clone(),
                sequence: value.sequence(),
                expired_at: value.expired_at(),
                observed_at: value.observed_at(),
                cause: value.cause(),
            },
            SessionEvent::Revoked(value) => Self::SessionRevoked {
                session_id: value.session_id().clone(),
                sequence: value.sequence(),
                revoked_at: value.observed_at(),
            },
        }
    }

    /// Projects one already-admitted request context without payload/client/runtime data.
    #[must_use]
    pub fn from_admitted_request(context: &PlatformRequestContext) -> Self {
        let actor = match context.actor() {
            M00AdmittedActor::Public => PlatformControlActor::Public,
            M00AdmittedActor::Authenticated(identities) => PlatformControlActor::Authenticated {
                tenant_id: identities.tenant_id().clone(),
                user_id: identities.user_id().clone(),
                session_id: identities.session_id().clone(),
            },
        };
        let operation = context.operation();
        Self::RequestAdmitted {
            request_id: context.request_id().clone(),
            command_id: context.command_id().clone(),
            correlation_id: context.correlation_id().clone(),
            causation_id: context.causation_id().cloned(),
            actor,
            operation_id: operation.operation_id().clone(),
            descriptor_snapshot_id: operation.descriptor_snapshot_id().clone(),
            permission_class: operation.permission_class(),
            effect_class: operation.effect_class(),
            policy_snapshot_id: context.policy_reference().clone(),
            observed_at: context.observed_at(),
        }
    }

    /// Returns the stable event class.
    #[must_use]
    pub const fn kind(&self) -> PlatformControlEventKind {
        match self {
            Self::SessionOpened { .. } => PlatformControlEventKind::SessionOpened,
            Self::SessionRefreshed { .. } => PlatformControlEventKind::SessionRefreshed,
            Self::SessionExpired { .. } => PlatformControlEventKind::SessionExpired,
            Self::SessionRevoked { .. } => PlatformControlEventKind::SessionRevoked,
            Self::RequestAdmitted { .. } => PlatformControlEventKind::RequestAdmitted,
        }
    }

    /// Returns the stable dedupe key.
    #[must_use]
    pub fn key(&self) -> ControlEvidenceKey {
        match self {
            Self::SessionOpened {
                session_id,
                sequence,
                ..
            }
            | Self::SessionRefreshed {
                session_id,
                sequence,
                ..
            }
            | Self::SessionExpired {
                session_id,
                sequence,
                ..
            }
            | Self::SessionRevoked {
                session_id,
                sequence,
                ..
            } => ControlEvidenceKey::Session {
                session_id: session_id.clone(),
                sequence: *sequence,
            },
            Self::RequestAdmitted { command_id, .. } => ControlEvidenceKey::Request {
                command_id: command_id.clone(),
            },
        }
    }

    /// Returns the transition/admission observation instant.
    #[must_use]
    pub const fn occurred_at(&self) -> SessionInstant {
        match self {
            Self::SessionOpened { opened_at, .. } => *opened_at,
            Self::SessionRefreshed { refreshed_at, .. } => *refreshed_at,
            Self::SessionExpired { observed_at, .. } => *observed_at,
            Self::SessionRevoked { revoked_at, .. } => *revoked_at,
            Self::RequestAdmitted { observed_at, .. } => *observed_at,
        }
    }
}

/// Stable redacted reason code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlatformControlErrorCode {
    LifecycleCredentialEvidenceExpired,
    LifecycleInvalidTimeOrder,
    LifecycleDeadlineOverflow,
    LifecycleSessionNotFound,
    LifecycleSessionAlreadyExists,
    LifecycleSessionIdMismatch,
    LifecycleRevisionMismatch,
    LifecycleRevisionOverflow,
    LifecycleTerminalSession,
    LifecycleNonMonotoneTime,
    LifecycleSessionNotYetExpired,
    LifecycleNoEffectiveRefresh,
    LifecycleEventSequenceMismatch,
    LifecycleEventTimeOutsideValidity,
    LifecycleIllegalEventForState,
    LifecycleEventDerivedFieldMismatch,
    AdmissionIdempotencyStoreUnavailable,
    AdmissionConflictingEnvelope,
    AdmissionDescriptorSnapshotAbsent,
    AdmissionDescriptorSnapshotMismatch,
    AdmissionPolicyDenied,
    AdmissionPolicyExpired,
    AdmissionSessionNotFound,
    AdmissionSessionIdMismatch,
    AdmissionSessionNotAdmitted,
    AdmissionCapabilityMissing,
    AdmissionCapabilityDisabled,
    AdmissionCapabilityRevoked,
    AdmissionInfrastructurePortUnavailable,
    AdmissionMalformedCommand,
    RepositoryUnavailable,
    RepositoryCorrupt,
    RepositoryInvalidEvent,
    RepositoryLimitExceeded,
    RepositoryInternalInvariant,
    MalformedExternalInput,
}

/// Stable external error containing no source diagnostic or arbitrary text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlatformControlError {
    code: PlatformControlErrorCode,
}

impl PlatformControlError {
    #[must_use]
    pub const fn from_session_domain(error: &SessionDomainError) -> Self {
        let code = match error {
            SessionDomainError::CredentialEvidenceExpired => {
                PlatformControlErrorCode::LifecycleCredentialEvidenceExpired
            }
            SessionDomainError::InvalidTimeOrder => {
                PlatformControlErrorCode::LifecycleInvalidTimeOrder
            }
            SessionDomainError::DeadlineOverflow => {
                PlatformControlErrorCode::LifecycleDeadlineOverflow
            }
            SessionDomainError::SessionNotFound => {
                PlatformControlErrorCode::LifecycleSessionNotFound
            }
            SessionDomainError::SessionAlreadyExists => {
                PlatformControlErrorCode::LifecycleSessionAlreadyExists
            }
            SessionDomainError::SessionIdMismatch => {
                PlatformControlErrorCode::LifecycleSessionIdMismatch
            }
            SessionDomainError::RevisionMismatch { .. } => {
                PlatformControlErrorCode::LifecycleRevisionMismatch
            }
            SessionDomainError::RevisionOverflow => {
                PlatformControlErrorCode::LifecycleRevisionOverflow
            }
            SessionDomainError::TerminalSession { .. } => {
                PlatformControlErrorCode::LifecycleTerminalSession
            }
            SessionDomainError::NonMonotoneTime => {
                PlatformControlErrorCode::LifecycleNonMonotoneTime
            }
            SessionDomainError::SessionNotYetExpired => {
                PlatformControlErrorCode::LifecycleSessionNotYetExpired
            }
            SessionDomainError::NoEffectiveRefresh => {
                PlatformControlErrorCode::LifecycleNoEffectiveRefresh
            }
            SessionDomainError::EventSequenceMismatch { .. } => {
                PlatformControlErrorCode::LifecycleEventSequenceMismatch
            }
            SessionDomainError::EventTimeOutsideValidity => {
                PlatformControlErrorCode::LifecycleEventTimeOutsideValidity
            }
            SessionDomainError::IllegalEventForState => {
                PlatformControlErrorCode::LifecycleIllegalEventForState
            }
            SessionDomainError::EventDerivedFieldMismatch { .. } => {
                PlatformControlErrorCode::LifecycleEventDerivedFieldMismatch
            }
        };
        Self { code }
    }

    #[must_use]
    pub const fn from_admission_rejection(rejection: &RequestContextRejection) -> Self {
        let code = match rejection.class() {
            AdmissionRejectionClass::IdempotencyStoreUnavailable => {
                PlatformControlErrorCode::AdmissionIdempotencyStoreUnavailable
            }
            AdmissionRejectionClass::ConflictingEnvelope => {
                PlatformControlErrorCode::AdmissionConflictingEnvelope
            }
            AdmissionRejectionClass::DescriptorSnapshotAbsent => {
                PlatformControlErrorCode::AdmissionDescriptorSnapshotAbsent
            }
            AdmissionRejectionClass::DescriptorSnapshotMismatch => {
                PlatformControlErrorCode::AdmissionDescriptorSnapshotMismatch
            }
            AdmissionRejectionClass::PolicyDenied => {
                PlatformControlErrorCode::AdmissionPolicyDenied
            }
            AdmissionRejectionClass::PolicyExpired => {
                PlatformControlErrorCode::AdmissionPolicyExpired
            }
            AdmissionRejectionClass::SessionNotFound => {
                PlatformControlErrorCode::AdmissionSessionNotFound
            }
            AdmissionRejectionClass::SessionIdMismatch => {
                PlatformControlErrorCode::AdmissionSessionIdMismatch
            }
            AdmissionRejectionClass::SessionNotAdmitted => {
                PlatformControlErrorCode::AdmissionSessionNotAdmitted
            }
            AdmissionRejectionClass::CapabilityMissing => {
                PlatformControlErrorCode::AdmissionCapabilityMissing
            }
            AdmissionRejectionClass::CapabilityDisabled => {
                PlatformControlErrorCode::AdmissionCapabilityDisabled
            }
            AdmissionRejectionClass::CapabilityRevoked => {
                PlatformControlErrorCode::AdmissionCapabilityRevoked
            }
            AdmissionRejectionClass::InfrastructurePortUnavailable => {
                PlatformControlErrorCode::AdmissionInfrastructurePortUnavailable
            }
            AdmissionRejectionClass::MalformedCommand => {
                PlatformControlErrorCode::AdmissionMalformedCommand
            }
        };
        Self { code }
    }

    #[must_use]
    pub const fn from_session_repository(error: SessionRepositoryError) -> Self {
        let code = match error {
            SessionRepositoryError::Unavailable => PlatformControlErrorCode::RepositoryUnavailable,
            SessionRepositoryError::Corrupt => PlatformControlErrorCode::RepositoryCorrupt,
            SessionRepositoryError::InvalidEvent => {
                PlatformControlErrorCode::RepositoryInvalidEvent
            }
            SessionRepositoryError::LimitExceeded => {
                PlatformControlErrorCode::RepositoryLimitExceeded
            }
            SessionRepositoryError::InternalInvariant => {
                PlatformControlErrorCode::RepositoryInternalInvariant
            }
        };
        Self { code }
    }

    /// Maps all untrusted decoder diagnostics to one bounded code without accepting their text.
    #[must_use]
    pub const fn malformed_external_input() -> Self {
        Self {
            code: PlatformControlErrorCode::MalformedExternalInput,
        }
    }

    #[must_use]
    pub const fn code(&self) -> PlatformControlErrorCode {
        self.code
    }
}

/// Closed evidence-journal failure without adapter diagnostics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlEvidenceJournalError {
    Unavailable,
    Corrupt,
    LimitExceeded,
    InternalInvariant,
}

/// Exact append-once disposition.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlEvidenceAppendOutcome {
    Appended,
    AlreadySame,
    Conflict,
}

/// Least-authority evidence read port.
pub trait ControlEvidenceReadPort {
    fn load_control_event(
        &mut self,
        key: &ControlEvidenceKey,
    ) -> Result<Option<PlatformControlEvent>, ControlEvidenceJournalError>;
}

/// Append-once evidence port; no result claims product-transaction atomicity.
pub trait ControlEvidenceAppendPort: ControlEvidenceReadPort {
    fn append_once(
        &mut self,
        event: &PlatformControlEvent,
    ) -> Result<ControlEvidenceAppendOutcome, ControlEvidenceJournalError>;
}
