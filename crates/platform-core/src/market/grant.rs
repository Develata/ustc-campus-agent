//! Explicit reviewed capability-grant authority for `market-grant/v0`.

use crate::identity::{TenantId, UserId};
use crate::invocation::{
    CapabilityGrantSnapshot, CapabilityId, CatalogRevision, ConfirmationPolicy, GrantSnapshotId,
    GrantState, GrantVersion, InstallationId, InstallationRevision, ObjectScope, PackageId,
    PackageVersion, Sha256Digest,
};
use crate::market::ValidatedPackageManifest;
use crate::market::capability::{
    AutoGrantDisposition, CapabilityDefinition, CapabilityPolicyChange, CapabilityRegistry,
    CapabilityRegistryRevision, CapabilityStatus, DataClass, EffectClass, ScopeKind,
    compare_capability_definitions,
};
use crate::market::installation::{InstallationSnapshot, ManagedInstallationState};
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt;

const EVIDENCE_DOMAIN: &[u8] = b"market-grant-admission-evidence/v0\0";
const SCOPE_DOMAIN: &[u8] = b"market-grant-tenant-user-scope/v0\0";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GrantConstructionError {
    InvalidCommandId,
    InvalidApprovalId,
    InvalidSnapshotId,
    InvalidGrantVersion,
    InvalidEventSequence,
    ScopeConstructionFailed,
    CrossTenantScope,
    InstallationTerminal,
    PackageBindingMismatch,
    CapabilityNotDeclared,
    CapabilityMissing,
    CapabilityInactive,
    ScopeKindMismatch,
    ConfirmationPolicyTooPermissive,
    ForbiddenAdministrativeScope,
    EvidenceIncoherent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GrantDecisionError {
    AggregateMissing,
    AggregateAlreadyPresent,
    AuthorityConflict,
    ApprovalAlreadyConsumed,
    SnapshotIdMismatch,
    TerminalState,
    VersionMismatch,
    AdmissionEvidenceMismatch,
    ScopeChangeRequiresNewGrant,
    IllegalTransition,
    SequenceOverflow,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GrantReplayError {
    InitialEventNotIssued,
    SequenceGap,
    SequenceDuplicate,
    SequenceOverflow,
    DuplicateCommandId,
    DuplicateApprovalId,
    PostTerminalEvent,
    IllegalTransition,
    VersionMismatch,
    SnapshotIdentityMismatch,
    AuthorityBindingMismatch,
    AdmissionEvidenceMismatch,
}

macro_rules! category_error {
    ($kind:ty, $prefix:literal) => {
        impl fmt::Display for $kind {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, concat!($prefix, ": {:?}"), self)
            }
        }
        impl Error for $kind {}
    };
}
category_error!(GrantConstructionError, "grant value rejected");
category_error!(GrantDecisionError, "grant command rejected");
category_error!(GrantReplayError, "grant event replay rejected");

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GrantCommandId(String);

impl GrantCommandId {
    pub fn parse(value: impl Into<String>) -> Result<Self, GrantConstructionError> {
        checked_prefixed(value.into(), "grant-cmd:", 118)
            .map(Self)
            .ok_or(GrantConstructionError::InvalidCommandId)
    }
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for GrantCommandId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("GrantCommandId(<redacted>)")
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GrantApprovalId(String);

impl GrantApprovalId {
    pub fn parse(value: impl Into<String>) -> Result<Self, GrantConstructionError> {
        checked_prefixed(value.into(), "grant-approval:", 113)
            .map(Self)
            .ok_or(GrantConstructionError::InvalidApprovalId)
    }
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for GrantApprovalId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("GrantApprovalId(<redacted>)")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GrantEventSequence(u64);

impl GrantEventSequence {
    pub fn new(value: u64) -> Result<Self, GrantConstructionError> {
        (value != 0)
            .then_some(Self(value))
            .ok_or(GrantConstructionError::InvalidEventSequence)
    }
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
    fn next(self) -> Result<Self, GrantDecisionError> {
        self.0
            .checked_add(1)
            .map(Self)
            .ok_or(GrantDecisionError::SequenceOverflow)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GrantScope {
    kind: ScopeKind,
    object_scope: ObjectScope,
    tenant_id: Option<TenantId>,
    user_id: Option<UserId>,
}

impl GrantScope {
    pub fn campus_public() -> Result<Self, GrantConstructionError> {
        let object_scope = ObjectScope::parse("scope:campus-public")
            .map_err(|_| GrantConstructionError::ScopeConstructionFailed)?;
        Ok(Self {
            kind: ScopeKind::CampusPublic,
            object_scope,
            tenant_id: None,
            user_id: None,
        })
    }

    pub fn tenant_private_user(
        tenant_id: TenantId,
        user_id: UserId,
    ) -> Result<Self, GrantConstructionError> {
        let mut bytes = SCOPE_DOMAIN.to_vec();
        encode_string(tenant_id.as_str(), &mut bytes);
        encode_string(user_id.as_str(), &mut bytes);
        let digest = Sha256Digest::from_bytes(&bytes);
        let object_scope = ObjectScope::parse(format!("scope:tenant-user:{}", digest.as_str()))
            .map_err(|_| GrantConstructionError::ScopeConstructionFailed)?;
        Ok(Self {
            kind: ScopeKind::TenantPrivateUser,
            object_scope,
            tenant_id: Some(tenant_id),
            user_id: Some(user_id),
        })
    }

    #[must_use]
    pub const fn scope_kind(&self) -> ScopeKind {
        self.kind
    }
    #[must_use]
    pub const fn object_scope(&self) -> &ObjectScope {
        &self.object_scope
    }
    #[must_use]
    pub const fn tenant_id(&self) -> Option<&TenantId> {
        self.tenant_id.as_ref()
    }
    #[must_use]
    pub const fn user_id(&self) -> Option<&UserId> {
        self.user_id.as_ref()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GrantInvalidationReason {
    CapabilityManifestChanged,
    CapabilityDefinitionChanged,
    InstallationChanged,
    PolicyChanged,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GrantChangeClass {
    Unchanged,
    Narrowed,
    ReapprovalRequired,
}

#[derive(Clone, PartialEq, Eq)]
pub struct GrantAdmissionEvidence {
    snapshot_id: GrantSnapshotId,
    approval_id: GrantApprovalId,
    tenant_id: TenantId,
    user_id: UserId,
    installation_id: InstallationId,
    expected_installation_revision: InstallationRevision,
    catalog_revision: CatalogRevision,
    package_id: PackageId,
    package_version: PackageVersion,
    package_digest: Sha256Digest,
    capability_id: CapabilityId,
    scope: GrantScope,
    confirmation_policy: ConfirmationPolicy,
    capability_manifest_digest: Sha256Digest,
    capability_registry_revision: CapabilityRegistryRevision,
    capability_definition: CapabilityDefinition,
    capability_definition_digest: Sha256Digest,
    evidence_digest: Sha256Digest,
}

impl fmt::Debug for GrantAdmissionEvidence {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("GrantAdmissionEvidence(<authority-redacted>)")
    }
}

impl GrantAdmissionEvidence {
    #[allow(dead_code)]
    #[allow(clippy::too_many_arguments)]
    pub(in crate::market) fn from_authority_bindings(
        snapshot_id: GrantSnapshotId,
        approval_id: GrantApprovalId,
        installation: &InstallationSnapshot,
        package: &ValidatedPackageManifest,
        capability_id: CapabilityId,
        scope: GrantScope,
        confirmation_policy: ConfirmationPolicy,
        registry: &CapabilityRegistry,
    ) -> Result<Self, GrantConstructionError> {
        validate_snapshot_id(&snapshot_id)?;
        if matches!(
            installation.state(),
            ManagedInstallationState::Revoked | ManagedInstallationState::Uninstalled
        ) {
            return Err(GrantConstructionError::InstallationTerminal);
        }
        let pin = installation.package_pin();
        if pin.package_id() != package.package_id()
            || pin.package_version() != package.package_version()
            || pin.package_digest() != package.package_digest()
            || pin.capability_manifest_digest() != package.capability_manifest_digest()
        {
            return Err(GrantConstructionError::PackageBindingMismatch);
        }
        if !package.capabilities().contains(&capability_id) {
            return Err(GrantConstructionError::CapabilityNotDeclared);
        }
        let definition = registry
            .find(&capability_id)
            .ok_or(GrantConstructionError::CapabilityMissing)?;
        if definition.status() != CapabilityStatus::Active {
            return Err(GrantConstructionError::CapabilityInactive);
        }
        if definition.scope_kind() == ScopeKind::OperatorAdministrative {
            return Err(GrantConstructionError::ForbiddenAdministrativeScope);
        }
        if definition.scope_kind() != scope.scope_kind() {
            return Err(GrantConstructionError::ScopeKindMismatch);
        }
        if scope.scope_kind() == ScopeKind::TenantPrivateUser
            && (scope.tenant_id() != Some(installation.tenant_id())
                || scope.user_id() != Some(installation.user_id()))
        {
            return Err(GrantConstructionError::CrossTenantScope);
        }
        if confirmation_policy == ConfirmationPolicy::Allow
            && definition.confirmation_default() == ConfirmationPolicy::Ask
        {
            return Err(GrantConstructionError::ConfirmationPolicyTooPermissive);
        }
        let mut evidence = Self {
            snapshot_id,
            approval_id,
            tenant_id: installation.tenant_id().clone(),
            user_id: installation.user_id().clone(),
            installation_id: installation.installation_id().clone(),
            expected_installation_revision: installation.revision().clone(),
            catalog_revision: pin.catalog_revision().clone(),
            package_id: pin.package_id().clone(),
            package_version: pin.package_version().clone(),
            package_digest: pin.package_digest().clone(),
            capability_id,
            scope,
            confirmation_policy,
            capability_manifest_digest: pin.capability_manifest_digest().clone(),
            capability_registry_revision: registry.registry_revision().clone(),
            capability_definition: definition.clone(),
            capability_definition_digest: definition.definition_digest().clone(),
            evidence_digest: Sha256Digest::from_bytes(&[]),
        };
        evidence.evidence_digest = digest_evidence(&evidence);
        Ok(evidence)
    }

    #[must_use]
    pub const fn snapshot_id(&self) -> &GrantSnapshotId {
        &self.snapshot_id
    }
    #[must_use]
    pub const fn approval_id(&self) -> &GrantApprovalId {
        &self.approval_id
    }
    #[must_use]
    pub const fn tenant_id(&self) -> &TenantId {
        &self.tenant_id
    }
    #[must_use]
    pub const fn user_id(&self) -> &UserId {
        &self.user_id
    }
    #[must_use]
    pub const fn installation_id(&self) -> &InstallationId {
        &self.installation_id
    }
    #[must_use]
    pub const fn expected_installation_revision(&self) -> &InstallationRevision {
        &self.expected_installation_revision
    }
    #[must_use]
    pub const fn catalog_revision(&self) -> &CatalogRevision {
        &self.catalog_revision
    }
    #[must_use]
    pub const fn package_id(&self) -> &PackageId {
        &self.package_id
    }
    #[must_use]
    pub const fn package_version(&self) -> &PackageVersion {
        &self.package_version
    }
    #[must_use]
    pub const fn package_digest(&self) -> &Sha256Digest {
        &self.package_digest
    }
    #[must_use]
    pub const fn capability_id(&self) -> &CapabilityId {
        &self.capability_id
    }
    #[must_use]
    pub const fn scope(&self) -> &GrantScope {
        &self.scope
    }
    #[must_use]
    pub const fn confirmation_policy(&self) -> ConfirmationPolicy {
        self.confirmation_policy
    }
    #[must_use]
    pub const fn capability_manifest_digest(&self) -> &Sha256Digest {
        &self.capability_manifest_digest
    }
    #[must_use]
    pub const fn capability_registry_revision(&self) -> &CapabilityRegistryRevision {
        &self.capability_registry_revision
    }
    #[must_use]
    pub const fn capability_definition(&self) -> &CapabilityDefinition {
        &self.capability_definition
    }
    #[must_use]
    pub const fn capability_definition_digest(&self) -> &Sha256Digest {
        &self.capability_definition_digest
    }
    #[must_use]
    pub const fn evidence_digest(&self) -> &Sha256Digest {
        &self.evidence_digest
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum GrantCommandAction {
    Issue(GrantAdmissionEvidence),
    Replace {
        expected_version: GrantVersion,
        evidence: GrantAdmissionEvidence,
    },
    MarkStale {
        expected_version: GrantVersion,
        reason: GrantInvalidationReason,
    },
    Expire {
        expected_version: GrantVersion,
    },
    Revoke {
        expected_version: GrantVersion,
    },
}

#[derive(Clone, PartialEq, Eq)]
pub struct GrantCommand {
    command_id: GrantCommandId,
    snapshot_id: GrantSnapshotId,
    action: GrantCommandAction,
}

impl fmt::Debug for GrantCommand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("GrantCommand(<authority-redacted>)")
    }
}

impl GrantCommand {
    pub fn issue(
        command_id: GrantCommandId,
        evidence: GrantAdmissionEvidence,
    ) -> Result<Self, GrantConstructionError> {
        validate_snapshot_id(evidence.snapshot_id())?;
        Ok(Self {
            command_id,
            snapshot_id: evidence.snapshot_id.clone(),
            action: GrantCommandAction::Issue(evidence),
        })
    }
    pub fn replace(
        command_id: GrantCommandId,
        expected_version: GrantVersion,
        evidence: GrantAdmissionEvidence,
    ) -> Result<Self, GrantConstructionError> {
        validate_grant_version(&expected_version)?;
        validate_snapshot_id(evidence.snapshot_id())?;
        Ok(Self {
            command_id,
            snapshot_id: evidence.snapshot_id.clone(),
            action: GrantCommandAction::Replace {
                expected_version,
                evidence,
            },
        })
    }
    pub fn mark_stale(
        command_id: GrantCommandId,
        snapshot_id: GrantSnapshotId,
        expected_version: GrantVersion,
        reason: GrantInvalidationReason,
    ) -> Result<Self, GrantConstructionError> {
        validate_snapshot_id(&snapshot_id)?;
        validate_grant_version(&expected_version)?;
        Ok(Self {
            command_id,
            snapshot_id,
            action: GrantCommandAction::MarkStale {
                expected_version,
                reason,
            },
        })
    }
    pub fn expire(
        command_id: GrantCommandId,
        snapshot_id: GrantSnapshotId,
        expected_version: GrantVersion,
    ) -> Result<Self, GrantConstructionError> {
        validate_snapshot_id(&snapshot_id)?;
        validate_grant_version(&expected_version)?;
        Ok(Self {
            command_id,
            snapshot_id,
            action: GrantCommandAction::Expire { expected_version },
        })
    }
    pub fn revoke(
        command_id: GrantCommandId,
        snapshot_id: GrantSnapshotId,
        expected_version: GrantVersion,
    ) -> Result<Self, GrantConstructionError> {
        validate_snapshot_id(&snapshot_id)?;
        validate_grant_version(&expected_version)?;
        Ok(Self {
            command_id,
            snapshot_id,
            action: GrantCommandAction::Revoke { expected_version },
        })
    }
    #[must_use]
    pub const fn command_id(&self) -> &GrantCommandId {
        &self.command_id
    }
    #[must_use]
    pub const fn snapshot_id(&self) -> &GrantSnapshotId {
        &self.snapshot_id
    }
    fn evidence(&self) -> Option<&GrantAdmissionEvidence> {
        match &self.action {
            GrantCommandAction::Issue(v) | GrantCommandAction::Replace { evidence: v, .. } => {
                Some(v)
            }
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GrantEventKind {
    Issued,
    Replaced,
    MarkedStale,
    Expired,
    Revoked,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum GrantEventPayload {
    Issued(GrantAdmissionEvidence),
    Replaced {
        evidence: GrantAdmissionEvidence,
        change_class: GrantChangeClass,
    },
    MarkedStale(GrantInvalidationReason),
    Expired,
    Revoked,
}

#[derive(Clone, PartialEq, Eq)]
pub struct GrantEvent {
    sequence: GrantEventSequence,
    post_version: GrantVersion,
    command_id: GrantCommandId,
    snapshot_id: GrantSnapshotId,
    payload: GrantEventPayload,
}

impl fmt::Debug for GrantEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("GrantEvent(<authority-redacted>)")
    }
}

impl GrantEvent {
    #[must_use]
    pub const fn sequence(&self) -> GrantEventSequence {
        self.sequence
    }
    #[must_use]
    pub const fn post_version(&self) -> &GrantVersion {
        &self.post_version
    }
    #[must_use]
    pub const fn command_id(&self) -> &GrantCommandId {
        &self.command_id
    }
    #[must_use]
    pub const fn snapshot_id(&self) -> &GrantSnapshotId {
        &self.snapshot_id
    }
    #[must_use]
    pub fn kind(&self) -> GrantEventKind {
        match &self.payload {
            GrantEventPayload::Issued(_) => GrantEventKind::Issued,
            GrantEventPayload::Replaced { .. } => GrantEventKind::Replaced,
            GrantEventPayload::MarkedStale(_) => GrantEventKind::MarkedStale,
            GrantEventPayload::Expired => GrantEventKind::Expired,
            GrantEventPayload::Revoked => GrantEventKind::Revoked,
        }
    }
    #[must_use]
    pub fn change_class(&self) -> Option<GrantChangeClass> {
        if let GrantEventPayload::Replaced { change_class, .. } = &self.payload {
            Some(*change_class)
        } else {
            None
        }
    }
    #[must_use]
    pub fn invalidation_reason(&self) -> Option<GrantInvalidationReason> {
        if let GrantEventPayload::MarkedStale(reason) = &self.payload {
            Some(*reason)
        } else {
            None
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct GrantAggregate {
    snapshot_id: GrantSnapshotId,
    tenant_id: TenantId,
    user_id: UserId,
    installation_id: InstallationId,
    installation_revision: InstallationRevision,
    catalog_revision: CatalogRevision,
    package_id: PackageId,
    package_version: PackageVersion,
    package_digest: Sha256Digest,
    capability_id: CapabilityId,
    scope: GrantScope,
    confirmation_policy: ConfirmationPolicy,
    capability_manifest_digest: Sha256Digest,
    capability_registry_revision: CapabilityRegistryRevision,
    capability_definition: CapabilityDefinition,
    capability_definition_digest: Sha256Digest,
    last_approval_id: GrantApprovalId,
    state: GrantState,
    version: GrantVersion,
    last_sequence: GrantEventSequence,
}

impl fmt::Debug for GrantAggregate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("GrantAggregate(<authority-redacted>)")
    }
}

pub type GrantSnapshot = GrantAggregate;

impl GrantAggregate {
    #[must_use]
    pub const fn snapshot_id(&self) -> &GrantSnapshotId {
        &self.snapshot_id
    }
    #[must_use]
    pub const fn tenant_id(&self) -> &TenantId {
        &self.tenant_id
    }
    #[must_use]
    pub const fn user_id(&self) -> &UserId {
        &self.user_id
    }
    #[must_use]
    pub const fn installation_id(&self) -> &InstallationId {
        &self.installation_id
    }
    #[must_use]
    pub const fn installation_revision(&self) -> &InstallationRevision {
        &self.installation_revision
    }
    #[must_use]
    pub const fn catalog_revision(&self) -> &CatalogRevision {
        &self.catalog_revision
    }
    #[must_use]
    pub const fn package_id(&self) -> &PackageId {
        &self.package_id
    }
    #[must_use]
    pub const fn package_version(&self) -> &PackageVersion {
        &self.package_version
    }
    #[must_use]
    pub const fn package_digest(&self) -> &Sha256Digest {
        &self.package_digest
    }
    #[must_use]
    pub const fn capability_id(&self) -> &CapabilityId {
        &self.capability_id
    }
    #[must_use]
    pub const fn scope(&self) -> &GrantScope {
        &self.scope
    }
    #[must_use]
    pub const fn confirmation_policy(&self) -> ConfirmationPolicy {
        self.confirmation_policy
    }
    #[must_use]
    pub const fn capability_manifest_digest(&self) -> &Sha256Digest {
        &self.capability_manifest_digest
    }
    #[must_use]
    pub const fn capability_registry_revision(&self) -> &CapabilityRegistryRevision {
        &self.capability_registry_revision
    }
    #[must_use]
    pub const fn capability_definition(&self) -> &CapabilityDefinition {
        &self.capability_definition
    }
    #[must_use]
    pub const fn capability_definition_digest(&self) -> &Sha256Digest {
        &self.capability_definition_digest
    }
    #[must_use]
    pub const fn last_approval_id(&self) -> &GrantApprovalId {
        &self.last_approval_id
    }
    #[must_use]
    pub const fn state(&self) -> GrantState {
        self.state
    }
    #[must_use]
    pub const fn version(&self) -> &GrantVersion {
        &self.version
    }
    #[must_use]
    pub const fn last_sequence(&self) -> GrantEventSequence {
        self.last_sequence
    }
    #[must_use]
    pub fn to_resolver_snapshot(&self) -> CapabilityGrantSnapshot {
        CapabilityGrantSnapshot {
            snapshot_id: self.snapshot_id.clone(),
            version: self.version.clone(),
            tenant_id: self.tenant_id.clone(),
            user_id: self.user_id.clone(),
            installation_id: self.installation_id.clone(),
            capability_id: self.capability_id.clone(),
            object_scope: self.scope.object_scope.clone(),
            confirmation_policy: self.confirmation_policy,
            capability_manifest_digest: self.capability_manifest_digest.clone(),
            state: self.state,
        }
    }
}

pub fn decide(
    current: Option<&GrantAggregate>,
    command: &GrantCommand,
) -> Result<GrantEvent, GrantDecisionError> {
    match &command.action {
        GrantCommandAction::Issue(evidence) => {
            if current.is_some() {
                return Err(GrantDecisionError::AggregateAlreadyPresent);
            }
            require_evidence_coherent(evidence)?;
            make_event(
                GrantEventSequence(1),
                command,
                GrantEventPayload::Issued(evidence.clone()),
            )
        }
        GrantCommandAction::Replace {
            expected_version,
            evidence,
        } => {
            let aggregate = require_current(current)?;
            require_target(aggregate, command)?;
            require_nonterminal(aggregate)?;
            require_version(aggregate, expected_version)?;
            require_evidence_coherent(evidence)?;
            if evidence.snapshot_id != aggregate.snapshot_id
                || evidence.tenant_id != aggregate.tenant_id
                || evidence.user_id != aggregate.user_id
                || evidence.installation_id != aggregate.installation_id
                || evidence.capability_id != aggregate.capability_id
            {
                return Err(GrantDecisionError::AdmissionEvidenceMismatch);
            }
            if evidence.scope != aggregate.scope {
                return Err(GrantDecisionError::ScopeChangeRequiresNewGrant);
            }
            if evidence.approval_id == aggregate.last_approval_id {
                return Err(GrantDecisionError::AdmissionEvidenceMismatch);
            }
            let change_class = classify_change(aggregate, evidence);
            make_event(
                aggregate.last_sequence.next()?,
                command,
                GrantEventPayload::Replaced {
                    evidence: evidence.clone(),
                    change_class,
                },
            )
        }
        GrantCommandAction::MarkStale {
            expected_version,
            reason,
        } => {
            let aggregate = require_current(current)?;
            require_target(aggregate, command)?;
            require_nonterminal(aggregate)?;
            require_version(aggregate, expected_version)?;
            if aggregate.state != GrantState::Active {
                return Err(GrantDecisionError::IllegalTransition);
            }
            make_event(
                aggregate.last_sequence.next()?,
                command,
                GrantEventPayload::MarkedStale(*reason),
            )
        }
        GrantCommandAction::Expire { expected_version } => {
            let aggregate = require_current(current)?;
            require_target(aggregate, command)?;
            require_nonterminal(aggregate)?;
            require_version(aggregate, expected_version)?;
            if !matches!(aggregate.state, GrantState::Active | GrantState::Stale) {
                return Err(GrantDecisionError::IllegalTransition);
            }
            make_event(
                aggregate.last_sequence.next()?,
                command,
                GrantEventPayload::Expired,
            )
        }
        GrantCommandAction::Revoke { expected_version } => {
            let aggregate = require_current(current)?;
            require_target(aggregate, command)?;
            require_nonterminal(aggregate)?;
            require_version(aggregate, expected_version)?;
            make_event(
                aggregate.last_sequence.next()?,
                command,
                GrantEventPayload::Revoked,
            )
        }
    }
}

pub fn evolve(
    current: Option<GrantAggregate>,
    event: &GrantEvent,
) -> Result<GrantAggregate, GrantReplayError> {
    verify_event_envelope(current.as_ref(), event)?;
    verify_event_reachable(current.as_ref(), event)?;
    match (current, &event.payload) {
        (None, GrantEventPayload::Issued(evidence)) => {
            verify_evidence(evidence)?;
            Ok(aggregate_from_evidence(evidence, GrantState::Active, event))
        }
        (None, _) => Err(GrantReplayError::InitialEventNotIssued),
        (Some(_), GrantEventPayload::Issued(_)) => Err(GrantReplayError::IllegalTransition),
        (
            Some(mut aggregate),
            GrantEventPayload::Replaced {
                evidence,
                change_class,
            },
        ) => {
            if aggregate.state == GrantState::Revoked {
                return Err(GrantReplayError::PostTerminalEvent);
            }
            verify_replacement(&aggregate, evidence, *change_class)?;
            apply_evidence(&mut aggregate, evidence);
            finish_transition(&mut aggregate, GrantState::Active, event);
            Ok(aggregate)
        }
        (Some(mut aggregate), GrantEventPayload::MarkedStale(_)) => {
            if aggregate.state == GrantState::Revoked {
                return Err(GrantReplayError::PostTerminalEvent);
            }
            if aggregate.state != GrantState::Active {
                return Err(GrantReplayError::IllegalTransition);
            }
            finish_transition(&mut aggregate, GrantState::Stale, event);
            Ok(aggregate)
        }
        (Some(mut aggregate), GrantEventPayload::Expired) => {
            if aggregate.state == GrantState::Revoked {
                return Err(GrantReplayError::PostTerminalEvent);
            }
            if !matches!(aggregate.state, GrantState::Active | GrantState::Stale) {
                return Err(GrantReplayError::IllegalTransition);
            }
            finish_transition(&mut aggregate, GrantState::Expired, event);
            Ok(aggregate)
        }
        (Some(mut aggregate), GrantEventPayload::Revoked) => {
            if aggregate.state == GrantState::Revoked {
                return Err(GrantReplayError::PostTerminalEvent);
            }
            finish_transition(&mut aggregate, GrantState::Revoked, event);
            Ok(aggregate)
        }
    }
}

pub fn replay<'a>(
    events: impl IntoIterator<Item = &'a GrantEvent>,
) -> Result<Option<GrantAggregate>, GrantReplayError> {
    let mut aggregate = None;
    let mut commands = BTreeSet::new();
    let mut approvals = BTreeSet::new();
    for event in events {
        if !commands.insert(event.command_id.clone()) {
            return Err(GrantReplayError::DuplicateCommandId);
        }
        if event_evidence(event)
            .is_some_and(|evidence| !approvals.insert(evidence.approval_id.clone()))
        {
            return Err(GrantReplayError::DuplicateApprovalId);
        }
        aggregate = Some(evolve(aggregate, event)?);
    }
    Ok(aggregate)
}

#[allow(clippy::large_enum_variant)]
#[derive(Clone, PartialEq, Eq)]
pub enum GrantCommandOutcome {
    Accepted {
        event: GrantEvent,
        snapshot: GrantSnapshot,
    },
    Rejected {
        error: GrantDecisionError,
    },
}

impl fmt::Debug for GrantCommandOutcome {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("GrantCommandOutcome(<authority-redacted>)")
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct GrantCommandReceipt {
    command_id: GrantCommandId,
    snapshot_id: GrantSnapshotId,
    outcome: GrantCommandOutcome,
}

impl fmt::Debug for GrantCommandReceipt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("GrantCommandReceipt(<authority-redacted>)")
    }
}

impl GrantCommandReceipt {
    #[must_use]
    pub const fn command_id(&self) -> &GrantCommandId {
        &self.command_id
    }
    #[must_use]
    pub const fn snapshot_id(&self) -> &GrantSnapshotId {
        &self.snapshot_id
    }
    #[must_use]
    pub const fn outcome(&self) -> &GrantCommandOutcome {
        &self.outcome
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GrantRepositoryError {
    CommandConflict,
    InjectedPersistenceFailure,
    CorruptEventHistory(GrantReplayError),
    CorruptAuthorityIndex,
    DecisionRejected(GrantDecisionError),
}
impl fmt::Display for GrantRepositoryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("grant repository operation rejected")
    }
}
impl Error for GrantRepositoryError {}

pub trait GrantRepository {
    fn execute(
        &mut self,
        command: GrantCommand,
    ) -> Result<GrantCommandReceipt, GrantRepositoryError>;
    fn load_exact(
        &self,
        id: &GrantSnapshotId,
    ) -> Result<Option<GrantSnapshot>, GrantRepositoryError>;
    fn load_current_for_authority(
        &self,
        tenant_id: &TenantId,
        user_id: &UserId,
        installation_id: &InstallationId,
        capability_id: &CapabilityId,
        scope: &GrantScope,
    ) -> Result<Option<GrantSnapshot>, GrantRepositoryError>;
    fn event_history(&self, id: &GrantSnapshotId) -> Result<Vec<GrantEvent>, GrantRepositoryError>;
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct AuthorityKey {
    tenant_id: TenantId,
    user_id: UserId,
    installation_id: InstallationId,
    capability_id: CapabilityId,
    object_scope: ObjectScope,
}
#[derive(Debug, Clone, PartialEq, Eq)]
struct LedgerEntry {
    command: GrantCommand,
    receipt: GrantCommandReceipt,
}

pub struct InMemoryGrantRepository {
    aggregates: BTreeMap<GrantSnapshotId, GrantAggregate>,
    events: BTreeMap<GrantSnapshotId, Vec<GrantEvent>>,
    command_ledger: BTreeMap<GrantCommandId, LedgerEntry>,
    consumed_approvals: BTreeMap<GrantApprovalId, (GrantSnapshotId, Sha256Digest)>,
    current_authority: BTreeMap<AuthorityKey, BTreeSet<GrantSnapshotId>>,
    fail_next_commit: bool,
}

impl fmt::Debug for InMemoryGrantRepository {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("InMemoryGrantRepository(<authority-redacted>)")
    }
}

impl InMemoryGrantRepository {
    #[must_use]
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            aggregates: BTreeMap::new(),
            events: BTreeMap::new(),
            command_ledger: BTreeMap::new(),
            consumed_approvals: BTreeMap::new(),
            current_authority: BTreeMap::new(),
            fail_next_commit: false,
        }
    }
    pub fn fail_next_commit_for_testing(&mut self) {
        self.fail_next_commit = true;
    }
}

impl GrantRepository for InMemoryGrantRepository {
    fn execute(
        &mut self,
        command: GrantCommand,
    ) -> Result<GrantCommandReceipt, GrantRepositoryError> {
        if let Some(entry) = self.command_ledger.get(command.command_id()) {
            return if entry.command == command {
                Ok(entry.receipt.clone())
            } else {
                Err(GrantRepositoryError::CommandConflict)
            };
        }
        let mut repository_rejection = None;
        if let Some(evidence) = command.evidence() {
            if self.consumed_approvals.contains_key(evidence.approval_id()) {
                repository_rejection = Some(GrantDecisionError::ApprovalAlreadyConsumed);
            }
            if repository_rejection.is_none()
                && matches!(command.action, GrantCommandAction::Issue(_))
            {
                let key = authority_key(evidence);
                if self
                    .current_authority
                    .get(&key)
                    .is_some_and(|ids| !ids.is_empty())
                {
                    repository_rejection = Some(GrantDecisionError::AuthorityConflict);
                }
            }
        }
        let current = self.aggregates.get(command.snapshot_id());
        let decision = repository_rejection.map_or_else(|| decide(current, &command), Err);
        let prepared = match decision {
            Ok(event) => {
                let snapshot = evolve(current.cloned(), &event)
                    .map_err(GrantRepositoryError::CorruptEventHistory)?;
                GrantCommandOutcome::Accepted { event, snapshot }
            }
            Err(error) => GrantCommandOutcome::Rejected { error },
        };
        if self.fail_next_commit {
            self.fail_next_commit = false;
            return Err(GrantRepositoryError::InjectedPersistenceFailure);
        }
        let receipt = GrantCommandReceipt {
            command_id: command.command_id.clone(),
            snapshot_id: command.snapshot_id.clone(),
            outcome: prepared.clone(),
        };
        if let GrantCommandOutcome::Accepted { event, snapshot } = prepared {
            if let Some(evidence) = event_evidence(&event) {
                self.consumed_approvals.insert(
                    evidence.approval_id.clone(),
                    (
                        evidence.snapshot_id.clone(),
                        evidence.evidence_digest.clone(),
                    ),
                );
            }
            let key = authority_key_from_aggregate(&snapshot);
            if snapshot.state == GrantState::Revoked {
                if let Some(ids) = self.current_authority.get_mut(&key) {
                    ids.remove(snapshot.snapshot_id());
                    if ids.is_empty() {
                        self.current_authority.remove(&key);
                    }
                }
            } else {
                self.current_authority
                    .entry(key)
                    .or_default()
                    .insert(snapshot.snapshot_id.clone());
            }
            self.aggregates
                .insert(snapshot.snapshot_id.clone(), snapshot);
            self.events
                .entry(event.snapshot_id.clone())
                .or_default()
                .push(event);
        }
        self.command_ledger.insert(
            command.command_id.clone(),
            LedgerEntry {
                command,
                receipt: receipt.clone(),
            },
        );
        Ok(receipt)
    }
    fn load_exact(
        &self,
        id: &GrantSnapshotId,
    ) -> Result<Option<GrantSnapshot>, GrantRepositoryError> {
        Ok(self.aggregates.get(id).cloned())
    }
    fn load_current_for_authority(
        &self,
        tenant_id: &TenantId,
        user_id: &UserId,
        installation_id: &InstallationId,
        capability_id: &CapabilityId,
        scope: &GrantScope,
    ) -> Result<Option<GrantSnapshot>, GrantRepositoryError> {
        let key = AuthorityKey {
            tenant_id: tenant_id.clone(),
            user_id: user_id.clone(),
            installation_id: installation_id.clone(),
            capability_id: capability_id.clone(),
            object_scope: scope.object_scope().clone(),
        };
        let Some(ids) = self.current_authority.get(&key) else {
            return Ok(None);
        };
        if ids.len() != 1 {
            return Err(GrantRepositoryError::CorruptAuthorityIndex);
        }
        let Some(id) = ids.first() else {
            return Err(GrantRepositoryError::CorruptAuthorityIndex);
        };
        self.aggregates
            .get(id)
            .cloned()
            .ok_or(GrantRepositoryError::CorruptAuthorityIndex)
            .map(Some)
    }
    fn event_history(&self, id: &GrantSnapshotId) -> Result<Vec<GrantEvent>, GrantRepositoryError> {
        Ok(self.events.get(id).cloned().unwrap_or_default())
    }
}

fn checked_prefixed(value: String, prefix: &str, max_tail: usize) -> Option<String> {
    let tail = value.strip_prefix(prefix)?;
    (!tail.is_empty()
        && tail.len() <= max_tail
        && tail
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_' | b':' | b'-')))
    .then_some(value)
}
fn validate_snapshot_id(id: &GrantSnapshotId) -> Result<(), GrantConstructionError> {
    checked_prefixed(id.as_str().to_owned(), "grant:", 122)
        .map(drop)
        .ok_or(GrantConstructionError::InvalidSnapshotId)
}
fn validate_grant_version(version: &GrantVersion) -> Result<(), GrantConstructionError> {
    let Some(tail) = version.as_str().strip_prefix("grant-version:") else {
        return Err(GrantConstructionError::InvalidGrantVersion);
    };
    if tail.is_empty()
        || tail.starts_with('0')
        || !tail.bytes().all(|b| b.is_ascii_digit())
        || tail.parse::<u64>().ok().filter(|v| *v != 0).is_none()
    {
        Err(GrantConstructionError::InvalidGrantVersion)
    } else {
        Ok(())
    }
}
fn version_for(sequence: GrantEventSequence) -> Result<GrantVersion, GrantReplayError> {
    GrantVersion::parse(format!("grant-version:{}", sequence.get()))
        .map_err(|_| GrantReplayError::SequenceOverflow)
}
fn make_event(
    sequence: GrantEventSequence,
    command: &GrantCommand,
    payload: GrantEventPayload,
) -> Result<GrantEvent, GrantDecisionError> {
    let post_version = version_for(sequence).map_err(|_| GrantDecisionError::SequenceOverflow)?;
    Ok(GrantEvent {
        sequence,
        post_version,
        command_id: command.command_id.clone(),
        snapshot_id: command.snapshot_id.clone(),
        payload,
    })
}
fn require_current(
    current: Option<&GrantAggregate>,
) -> Result<&GrantAggregate, GrantDecisionError> {
    current.ok_or(GrantDecisionError::AggregateMissing)
}
fn require_target(
    current: &GrantAggregate,
    command: &GrantCommand,
) -> Result<(), GrantDecisionError> {
    if current.snapshot_id == command.snapshot_id {
        Ok(())
    } else {
        Err(GrantDecisionError::SnapshotIdMismatch)
    }
}
fn require_nonterminal(current: &GrantAggregate) -> Result<(), GrantDecisionError> {
    if current.state == GrantState::Revoked {
        Err(GrantDecisionError::TerminalState)
    } else {
        Ok(())
    }
}
fn require_version(
    current: &GrantAggregate,
    expected: &GrantVersion,
) -> Result<(), GrantDecisionError> {
    if &current.version == expected {
        Ok(())
    } else {
        Err(GrantDecisionError::VersionMismatch)
    }
}
fn require_evidence_coherent(evidence: &GrantAdmissionEvidence) -> Result<(), GrantDecisionError> {
    verify_evidence(evidence).map_err(|_| GrantDecisionError::AdmissionEvidenceMismatch)
}
fn verify_evidence(e: &GrantAdmissionEvidence) -> Result<(), GrantReplayError> {
    validate_snapshot_id(&e.snapshot_id)
        .map_err(|_| GrantReplayError::AdmissionEvidenceMismatch)?;
    GrantApprovalId::parse(e.approval_id.as_str())
        .map_err(|_| GrantReplayError::AdmissionEvidenceMismatch)?;
    let scope_is_coherent = match e.scope.scope_kind() {
        ScopeKind::CampusPublic => {
            e.scope.tenant_id().is_none()
                && e.scope.user_id().is_none()
                && GrantScope::campus_public().is_ok_and(|scope| scope == e.scope)
        }
        ScopeKind::TenantPrivateUser => {
            e.scope.tenant_id() == Some(&e.tenant_id)
                && e.scope.user_id() == Some(&e.user_id)
                && GrantScope::tenant_private_user(e.tenant_id.clone(), e.user_id.clone())
                    .is_ok_and(|scope| scope == e.scope)
        }
        ScopeKind::OperatorAdministrative => false,
    };
    if e.capability_definition.id() != &e.capability_id
        || e.capability_definition.definition_digest() != &e.capability_definition_digest
        || e.capability_definition.status() != CapabilityStatus::Active
        || e.capability_definition.scope_kind() == ScopeKind::OperatorAdministrative
        || e.capability_definition.scope_kind() != e.scope.scope_kind()
        || !scope_is_coherent
        || (e.confirmation_policy == ConfirmationPolicy::Allow
            && e.capability_definition.confirmation_default() == ConfirmationPolicy::Ask)
        || digest_evidence(e) != e.evidence_digest
    {
        Err(GrantReplayError::AdmissionEvidenceMismatch)
    } else {
        Ok(())
    }
}
fn classify_change(old: &GrantAggregate, new: &GrantAdmissionEvidence) -> GrantChangeClass {
    let policy = compare_capability_definitions(
        Some(&old.capability_definition),
        Some(&new.capability_definition),
    );
    let confirmation_narrowed = old.confirmation_policy == ConfirmationPolicy::Allow
        && new.confirmation_policy == ConfirmationPolicy::Ask;
    let confirmation_widened = old.confirmation_policy == ConfirmationPolicy::Ask
        && new.confirmation_policy == ConfirmationPolicy::Allow;
    if confirmation_widened
        || old.capability_manifest_digest != new.capability_manifest_digest
        || matches!(
            policy,
            CapabilityPolicyChange::ExpansionRequiresReapproval
                | CapabilityPolicyChange::RemovedOrRevoked
        )
    {
        GrantChangeClass::ReapprovalRequired
    } else if confirmation_narrowed || policy == CapabilityPolicyChange::Narrowed {
        GrantChangeClass::Narrowed
    } else {
        GrantChangeClass::Unchanged
    }
}
fn verify_replacement(
    old: &GrantAggregate,
    evidence: &GrantAdmissionEvidence,
    class: GrantChangeClass,
) -> Result<(), GrantReplayError> {
    verify_evidence(evidence)?;
    if old.snapshot_id != evidence.snapshot_id
        || old.tenant_id != evidence.tenant_id
        || old.user_id != evidence.user_id
        || old.installation_id != evidence.installation_id
        || old.capability_id != evidence.capability_id
        || old.scope != evidence.scope
    {
        return Err(GrantReplayError::AuthorityBindingMismatch);
    }
    if old.last_approval_id == evidence.approval_id || classify_change(old, evidence) != class {
        return Err(GrantReplayError::AdmissionEvidenceMismatch);
    }
    Ok(())
}
fn verify_event_envelope(
    current: Option<&GrantAggregate>,
    event: &GrantEvent,
) -> Result<(), GrantReplayError> {
    validate_snapshot_id(&event.snapshot_id)
        .map_err(|_| GrantReplayError::SnapshotIdentityMismatch)?;
    let expected = match current {
        None => 1,
        Some(v) => v
            .last_sequence
            .get()
            .checked_add(1)
            .ok_or(GrantReplayError::SequenceOverflow)?,
    };
    if event.sequence.get() < expected {
        return Err(GrantReplayError::SequenceDuplicate);
    }
    if event.sequence.get() > expected {
        return Err(GrantReplayError::SequenceGap);
    }
    if event.post_version != version_for(event.sequence)? {
        return Err(GrantReplayError::VersionMismatch);
    }
    if current.is_some_and(|value| value.snapshot_id != event.snapshot_id) {
        return Err(GrantReplayError::SnapshotIdentityMismatch);
    }
    Ok(())
}
fn verify_event_reachable(
    current: Option<&GrantAggregate>,
    event: &GrantEvent,
) -> Result<(), GrantReplayError> {
    if let Some(evidence) = event_evidence(event) {
        verify_evidence(evidence)?;
        if evidence.snapshot_id != event.snapshot_id {
            return Err(GrantReplayError::SnapshotIdentityMismatch);
        }
    }
    let command = match &event.payload {
        GrantEventPayload::Issued(evidence) => {
            GrantCommand::issue(event.command_id.clone(), evidence.clone())
        }
        GrantEventPayload::Replaced { evidence, .. } => GrantCommand::replace(
            event.command_id.clone(),
            current
                .ok_or(GrantReplayError::InitialEventNotIssued)?
                .version
                .clone(),
            evidence.clone(),
        ),
        GrantEventPayload::MarkedStale(reason) => GrantCommand::mark_stale(
            event.command_id.clone(),
            event.snapshot_id.clone(),
            current
                .ok_or(GrantReplayError::InitialEventNotIssued)?
                .version
                .clone(),
            *reason,
        ),
        GrantEventPayload::Expired => GrantCommand::expire(
            event.command_id.clone(),
            event.snapshot_id.clone(),
            current
                .ok_or(GrantReplayError::InitialEventNotIssued)?
                .version
                .clone(),
        ),
        GrantEventPayload::Revoked => GrantCommand::revoke(
            event.command_id.clone(),
            event.snapshot_id.clone(),
            current
                .ok_or(GrantReplayError::InitialEventNotIssued)?
                .version
                .clone(),
        ),
    }
    .map_err(|error| match error {
        GrantConstructionError::InvalidSnapshotId => GrantReplayError::SnapshotIdentityMismatch,
        _ => GrantReplayError::AdmissionEvidenceMismatch,
    })?;
    let decided = decide(current, &command).map_err(map_decision_to_replay)?;
    if decided == *event {
        Ok(())
    } else {
        Err(GrantReplayError::AdmissionEvidenceMismatch)
    }
}
fn map_decision_to_replay(error: GrantDecisionError) -> GrantReplayError {
    match error {
        GrantDecisionError::SnapshotIdMismatch => GrantReplayError::SnapshotIdentityMismatch,
        GrantDecisionError::VersionMismatch => GrantReplayError::VersionMismatch,
        GrantDecisionError::TerminalState => GrantReplayError::PostTerminalEvent,
        GrantDecisionError::AggregateMissing => GrantReplayError::InitialEventNotIssued,
        GrantDecisionError::SequenceOverflow => GrantReplayError::SequenceOverflow,
        GrantDecisionError::AdmissionEvidenceMismatch
        | GrantDecisionError::ScopeChangeRequiresNewGrant => {
            GrantReplayError::AdmissionEvidenceMismatch
        }
        GrantDecisionError::AggregateAlreadyPresent
        | GrantDecisionError::AuthorityConflict
        | GrantDecisionError::ApprovalAlreadyConsumed
        | GrantDecisionError::IllegalTransition => GrantReplayError::IllegalTransition,
    }
}
fn aggregate_from_evidence(
    e: &GrantAdmissionEvidence,
    state: GrantState,
    event: &GrantEvent,
) -> GrantAggregate {
    GrantAggregate {
        snapshot_id: e.snapshot_id.clone(),
        tenant_id: e.tenant_id.clone(),
        user_id: e.user_id.clone(),
        installation_id: e.installation_id.clone(),
        installation_revision: e.expected_installation_revision.clone(),
        catalog_revision: e.catalog_revision.clone(),
        package_id: e.package_id.clone(),
        package_version: e.package_version.clone(),
        package_digest: e.package_digest.clone(),
        capability_id: e.capability_id.clone(),
        scope: e.scope.clone(),
        confirmation_policy: e.confirmation_policy,
        capability_manifest_digest: e.capability_manifest_digest.clone(),
        capability_registry_revision: e.capability_registry_revision.clone(),
        capability_definition: e.capability_definition.clone(),
        capability_definition_digest: e.capability_definition_digest.clone(),
        last_approval_id: e.approval_id.clone(),
        state,
        version: event.post_version.clone(),
        last_sequence: event.sequence,
    }
}
fn apply_evidence(a: &mut GrantAggregate, e: &GrantAdmissionEvidence) {
    a.installation_revision = e.expected_installation_revision.clone();
    a.catalog_revision = e.catalog_revision.clone();
    a.package_id = e.package_id.clone();
    a.package_version = e.package_version.clone();
    a.package_digest = e.package_digest.clone();
    a.confirmation_policy = e.confirmation_policy;
    a.capability_manifest_digest = e.capability_manifest_digest.clone();
    a.capability_registry_revision = e.capability_registry_revision.clone();
    a.capability_definition = e.capability_definition.clone();
    a.capability_definition_digest = e.capability_definition_digest.clone();
    a.last_approval_id = e.approval_id.clone();
}
fn finish_transition(a: &mut GrantAggregate, state: GrantState, event: &GrantEvent) {
    a.state = state;
    a.version = event.post_version.clone();
    a.last_sequence = event.sequence;
}
fn event_evidence(event: &GrantEvent) -> Option<&GrantAdmissionEvidence> {
    match &event.payload {
        GrantEventPayload::Issued(e) | GrantEventPayload::Replaced { evidence: e, .. } => Some(e),
        _ => None,
    }
}
fn authority_key(e: &GrantAdmissionEvidence) -> AuthorityKey {
    AuthorityKey {
        tenant_id: e.tenant_id.clone(),
        user_id: e.user_id.clone(),
        installation_id: e.installation_id.clone(),
        capability_id: e.capability_id.clone(),
        object_scope: e.scope.object_scope().clone(),
    }
}
fn authority_key_from_aggregate(a: &GrantAggregate) -> AuthorityKey {
    AuthorityKey {
        tenant_id: a.tenant_id.clone(),
        user_id: a.user_id.clone(),
        installation_id: a.installation_id.clone(),
        capability_id: a.capability_id.clone(),
        object_scope: a.scope.object_scope().clone(),
    }
}
fn encode_string(value: &str, out: &mut Vec<u8>) {
    out.extend_from_slice(&(value.len() as u64).to_be_bytes());
    out.extend_from_slice(value.as_bytes());
}
fn encode_tag(tag: u8, out: &mut Vec<u8>) {
    out.extend_from_slice(&1_u64.to_be_bytes());
    out.push(tag);
}
fn digest_evidence(e: &GrantAdmissionEvidence) -> Sha256Digest {
    let mut out = EVIDENCE_DOMAIN.to_vec();
    let package_version = e.package_version.as_str();
    for value in [
        e.snapshot_id.as_str(),
        e.approval_id.as_str(),
        e.tenant_id.as_str(),
        e.user_id.as_str(),
        e.installation_id.as_str(),
        e.expected_installation_revision.as_str(),
        e.catalog_revision.as_str(),
        e.package_id.as_str(),
        &package_version,
        e.package_digest.as_str(),
        e.capability_id.as_str(),
        e.scope.object_scope.as_str(),
        e.capability_manifest_digest.as_str(),
        e.capability_registry_revision.as_str(),
        e.capability_definition_digest.as_str(),
    ] {
        encode_string(value, &mut out);
    }
    encode_tag(
        match e.confirmation_policy {
            ConfirmationPolicy::Allow => 0,
            ConfirmationPolicy::Ask => 1,
        },
        &mut out,
    );
    encode_tag(
        match e.capability_definition.effect_class() {
            EffectClass::Read => 0,
            EffectClass::Write => 1,
            EffectClass::Destructive => 2,
            EffectClass::Linkout => 3,
            EffectClass::Diagnostic => 4,
        },
        &mut out,
    );
    encode_tag(
        match e.capability_definition.data_class() {
            DataClass::PublicCampusFact => 0,
            DataClass::TenantPrivateFact => 1,
            DataClass::UserProfile => 2,
            DataClass::Credential => 3,
            DataClass::Administrative => 4,
        },
        &mut out,
    );
    encode_tag(
        match e.capability_definition.scope_kind() {
            ScopeKind::CampusPublic => 0,
            ScopeKind::TenantPrivateUser => 1,
            ScopeKind::OperatorAdministrative => 2,
        },
        &mut out,
    );
    encode_tag(
        match e.capability_definition.auto_grant() {
            AutoGrantDisposition::Never => 0,
            AutoGrantDisposition::FirstPartyDefaultOnly => 1,
        },
        &mut out,
    );
    encode_tag(
        match e.capability_definition.confirmation_default() {
            ConfirmationPolicy::Allow => 0,
            ConfirmationPolicy::Ask => 1,
        },
        &mut out,
    );
    encode_tag(
        match e.capability_definition.status() {
            CapabilityStatus::Active => 0,
            CapabilityStatus::Deprecated => 1,
            CapabilityStatus::Revoked => 2,
        },
        &mut out,
    );
    Sha256Digest::from_bytes(&out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::invocation::{ComponentId, ComponentKind, ComponentVersion, ExecutionIdentity};
    use crate::market::capability::load_capability_registry;
    use crate::market::installation::{
        InstallationCommand, InstallationCommandId, InstallationConfiguration,
        InstallationPackagePin, InstalledComponentPin,
    };
    use crate::market::load_package_manifest;

    const PACKAGE: &[u8] = br#"{
      "id":"ustc.grant-tests","version":"1.0.0","publisher":"first-party",
      "tier":"FirstParty","displayName":"Grant Tests","description":"Typed fixture",
      "implementationStatus":"development",
      "installPolicy":{"class":"FirstPartySystemPlugin","defaultInstalled":true,"defaultEnabled":true,"userDisableAllowed":true},
      "components":[],
      "capabilities":["campus.public_plan.read","user.own_academic_snapshot.read"],
      "sourcePolicy":{"fixture":"bounded"}
    }"#;
    const REGISTRY: &[u8] = include_bytes!("../../../../market/capabilities/registry.json");

    macro_rules! parsed {
        ($kind:ty, $value:expr) => {
            <$kind>::parse($value).expect("typed fixture value")
        };
    }

    struct Fixture {
        package: ValidatedPackageManifest,
        registry: CapabilityRegistry,
        installation: InstallationSnapshot,
        tenant: TenantId,
        user: UserId,
        capability: CapabilityId,
        scope: GrantScope,
    }

    impl Fixture {
        fn new() -> Self {
            let package = load_package_manifest(PACKAGE).expect("reviewed package");
            let registry = load_capability_registry(REGISTRY).expect("reviewed registry");
            let tenant = parsed!(TenantId, "tenant:grant-tests");
            let user = parsed!(UserId, "user:grant-tests");
            let component = InstalledComponentPin::new(
                parsed!(ComponentId, "component:grant-tests"),
                ComponentKind::NativeRustComponent,
                parsed!(ComponentVersion, "component-version:1"),
                Sha256Digest::from_bytes(b"component"),
                parsed!(ExecutionIdentity, "native:grant-tests"),
            )
            .expect("component");
            let pin = InstallationPackagePin::new(
                parsed!(CatalogRevision, "catalog:grant-tests"),
                package.package_id().clone(),
                package.package_version().clone(),
                package.package_digest().clone(),
                vec![component],
                Sha256Digest::from_bytes(b"components"),
                package.capability_manifest_digest().clone(),
            )
            .expect("package pin");
            let installation_id = parsed!(InstallationId, "installation:grant-tests");
            let configuration =
                InstallationConfiguration::new(&tenant, Vec::new()).expect("configuration");
            let command = InstallationCommand::install(
                parsed!(InstallationCommandId, "cmd:grant-tests-install"),
                installation_id,
                tenant.clone(),
                user.clone(),
                pin,
                configuration,
            )
            .expect("install command");
            let event = crate::market::installation::decide(None, &command).expect("install");
            let installation =
                crate::market::installation::evolve(None, &event).expect("installation snapshot");
            let capability = parsed!(CapabilityId, "campus.public_plan.read");
            let scope = GrantScope::campus_public().expect("scope");
            Self {
                package,
                registry,
                installation,
                tenant,
                user,
                capability,
                scope,
            }
        }

        fn evidence(&self, snapshot: &str, approval: &str) -> GrantAdmissionEvidence {
            GrantAdmissionEvidence::from_authority_bindings(
                parsed!(GrantSnapshotId, snapshot),
                parsed!(GrantApprovalId, approval),
                &self.installation,
                &self.package,
                self.capability.clone(),
                self.scope.clone(),
                ConfirmationPolicy::Allow,
                &self.registry,
            )
            .expect("admission evidence")
        }

        fn issue(&self, snapshot: &str, approval: &str, command: &str) -> GrantCommand {
            GrantCommand::issue(
                parsed!(GrantCommandId, command),
                self.evidence(snapshot, approval),
            )
            .expect("issue command")
        }

        fn issued(&self) -> (GrantEvent, GrantAggregate) {
            let command = self.issue(
                "grant:primary",
                "grant-approval:primary",
                "grant-cmd:issue-primary",
            );
            let event = decide(None, &command).expect("issue decision");
            let aggregate = evolve(None, &event).expect("issue evolution");
            (event, aggregate)
        }
    }

    fn accepted(receipt: &GrantCommandReceipt) -> (&GrantEvent, &GrantSnapshot) {
        match receipt.outcome() {
            GrantCommandOutcome::Accepted { event, snapshot } => (event, snapshot),
            GrantCommandOutcome::Rejected { error } => panic!("unexpected rejection: {error}"),
        }
    }

    fn rejected(receipt: &GrantCommandReceipt) -> GrantDecisionError {
        match receipt.outcome() {
            GrantCommandOutcome::Rejected { error } => *error,
            GrantCommandOutcome::Accepted { .. } => panic!("unexpected acceptance"),
        }
    }

    #[test]
    fn admission_evidence_requires_actual_nonterminal_installation_manifest_and_registry() {
        let fixture = Fixture::new();
        let evidence = fixture.evidence("grant:admitted", "grant-approval:admitted");
        assert_eq!(
            evidence.installation_id(),
            fixture.installation.installation_id()
        );
        assert_eq!(evidence.package_digest(), fixture.package.package_digest());
        assert_eq!(evidence.capability_definition().id(), &fixture.capability);
        assert_eq!(&digest_evidence(&evidence), evidence.evidence_digest());
        let revoke = InstallationCommand::revoke(
            parsed!(InstallationCommandId, "cmd:grant-tests-revoke"),
            fixture.installation.installation_id().clone(),
            fixture.installation.revision().clone(),
        )
        .expect("revoke installation");
        let event = crate::market::installation::decide(Some(&fixture.installation), &revoke)
            .expect("revoke decision");
        let terminal =
            crate::market::installation::evolve(Some(fixture.installation.clone()), &event)
                .expect("terminal installation");
        assert_eq!(terminal.state(), ManagedInstallationState::Revoked);
        assert_eq!(
            GrantAdmissionEvidence::from_authority_bindings(
                parsed!(GrantSnapshotId, "grant:terminal"),
                parsed!(GrantApprovalId, "grant-approval:terminal"),
                &terminal,
                &fixture.package,
                fixture.capability.clone(),
                fixture.scope.clone(),
                ConfirmationPolicy::Allow,
                &fixture.registry,
            ),
            Err(GrantConstructionError::InstallationTerminal)
        );
    }
    #[test]
    fn admission_evidence_rejects_package_capability_scope_confirmation_and_tenant_mismatch() {
        let fixture = Fixture::new();
        let other_package = load_package_manifest(include_bytes!(
            "../../../../market/packages/ustc.change-radar/package.json"
        ))
        .expect("other package");
        let base = |package: &ValidatedPackageManifest,
                    capability: CapabilityId,
                    scope: GrantScope,
                    confirmation| {
            GrantAdmissionEvidence::from_authority_bindings(
                parsed!(GrantSnapshotId, "grant:bad"),
                parsed!(GrantApprovalId, "grant-approval:bad"),
                &fixture.installation,
                package,
                capability,
                scope,
                confirmation,
                &fixture.registry,
            )
        };
        assert_eq!(
            base(
                &other_package,
                fixture.capability.clone(),
                fixture.scope.clone(),
                ConfirmationPolicy::Allow
            ),
            Err(GrantConstructionError::PackageBindingMismatch)
        );
        assert_eq!(
            base(
                &fixture.package,
                parsed!(CapabilityId, "campus.public_rules.read"),
                fixture.scope.clone(),
                ConfirmationPolicy::Allow
            ),
            Err(GrantConstructionError::CapabilityNotDeclared)
        );
        let private_capability = parsed!(CapabilityId, "user.own_academic_snapshot.read");
        let foreign_scope = GrantScope::tenant_private_user(
            parsed!(TenantId, "tenant:foreign"),
            fixture.user.clone(),
        )
        .expect("foreign scope");
        assert_eq!(
            base(
                &fixture.package,
                private_capability.clone(),
                fixture.scope.clone(),
                ConfirmationPolicy::Ask
            ),
            Err(GrantConstructionError::ScopeKindMismatch)
        );
        assert_eq!(
            base(
                &fixture.package,
                private_capability.clone(),
                foreign_scope,
                ConfirmationPolicy::Ask
            ),
            Err(GrantConstructionError::CrossTenantScope)
        );
        let own_scope =
            GrantScope::tenant_private_user(fixture.tenant.clone(), fixture.user.clone())
                .expect("own scope");
        assert_eq!(
            base(
                &fixture.package,
                private_capability,
                own_scope,
                ConfirmationPolicy::Allow
            ),
            Err(GrantConstructionError::ConfirmationPolicyTooPermissive)
        );
    }
    #[test]
    fn issue_decide_evolve_and_resolver_projection_are_exact() {
        let fixture = Fixture::new();
        let (event, aggregate) = fixture.issued();
        assert_eq!(event.kind(), GrantEventKind::Issued);
        assert_eq!(aggregate.state(), GrantState::Active);
        assert_eq!(replay([&event]), Ok(Some(aggregate.clone())));
        let projected = aggregate.to_resolver_snapshot();
        assert_eq!(&projected.snapshot_id, aggregate.snapshot_id());
        assert_eq!(projected.state, GrantState::Active);
        assert_eq!(&projected.object_scope, fixture.scope.object_scope());
    }
    #[test]
    fn replace_requires_fresh_approval_same_scope_and_computed_change_class() {
        let fixture = Fixture::new();
        let (_, aggregate) = fixture.issued();
        let same_approval = GrantCommand::replace(
            parsed!(GrantCommandId, "grant-cmd:replace-same-approval"),
            aggregate.version().clone(),
            fixture.evidence("grant:primary", "grant-approval:primary"),
        )
        .expect("replace command");
        assert_eq!(
            decide(Some(&aggregate), &same_approval),
            Err(GrantDecisionError::AdmissionEvidenceMismatch)
        );
        let replacement = GrantCommand::replace(
            parsed!(GrantCommandId, "grant-cmd:replace-fresh"),
            aggregate.version().clone(),
            fixture.evidence("grant:primary", "grant-approval:fresh"),
        )
        .expect("replace command");
        let event = decide(Some(&aggregate), &replacement).expect("replacement");
        assert_eq!(event.change_class(), Some(GrantChangeClass::Unchanged));
        let narrowed_evidence = GrantAdmissionEvidence::from_authority_bindings(
            parsed!(GrantSnapshotId, "grant:primary"),
            parsed!(GrantApprovalId, "grant-approval:narrowed"),
            &fixture.installation,
            &fixture.package,
            fixture.capability.clone(),
            fixture.scope.clone(),
            ConfirmationPolicy::Ask,
            &fixture.registry,
        )
        .expect("narrowed evidence");
        let narrowed = GrantCommand::replace(
            parsed!(GrantCommandId, "grant-cmd:replace-narrowed"),
            aggregate.version().clone(),
            narrowed_evidence,
        )
        .expect("narrowed command");
        assert_eq!(
            decide(Some(&aggregate), &narrowed)
                .expect("narrowed replacement")
                .change_class(),
            Some(GrantChangeClass::Narrowed)
        );
        let private_registry = load_capability_registry(
            br#"{
          "schemaVersion":"capability-registry/v1",
          "registryRevision":"capability-registry:private-scope",
          "capabilities":[{
            "id":"campus.public_plan.read","effectClass":"Read","dataClass":"UserProfile",
            "scopeKind":"TenantPrivateUser","autoGrant":"Never",
            "confirmationDefault":"Ask","status":"Active"
          }]
        }"#,
        )
        .expect("private registry");
        let private_scope =
            GrantScope::tenant_private_user(fixture.tenant.clone(), fixture.user.clone())
                .expect("private scope");
        let changed_scope = GrantAdmissionEvidence::from_authority_bindings(
            parsed!(GrantSnapshotId, "grant:primary"),
            parsed!(GrantApprovalId, "grant-approval:scope"),
            &fixture.installation,
            &fixture.package,
            fixture.capability.clone(),
            private_scope,
            ConfirmationPolicy::Ask,
            &private_registry,
        )
        .expect("scope-change evidence");
        let command = GrantCommand::replace(
            parsed!(GrantCommandId, "grant-cmd:replace-scope"),
            aggregate.version().clone(),
            changed_scope,
        )
        .expect("replace command");
        assert_eq!(
            decide(Some(&aggregate), &command),
            Err(GrantDecisionError::ScopeChangeRequiresNewGrant)
        );
    }
    #[test]
    fn stale_expire_revoke_and_terminal_transitions_fail_closed() {
        let fixture = Fixture::new();
        let (_, mut aggregate) = fixture.issued();
        for (index, state) in [GrantState::Stale, GrantState::Expired, GrantState::Revoked]
            .into_iter()
            .enumerate()
        {
            let command = match state {
                GrantState::Stale => GrantCommand::mark_stale(
                    parsed!(GrantCommandId, "grant-cmd:stale"),
                    aggregate.snapshot_id().clone(),
                    aggregate.version().clone(),
                    GrantInvalidationReason::PolicyChanged,
                ),
                GrantState::Expired => GrantCommand::expire(
                    parsed!(GrantCommandId, "grant-cmd:expire"),
                    aggregate.snapshot_id().clone(),
                    aggregate.version().clone(),
                ),
                GrantState::Revoked => GrantCommand::revoke(
                    parsed!(GrantCommandId, "grant-cmd:revoke"),
                    aggregate.snapshot_id().clone(),
                    aggregate.version().clone(),
                ),
                GrantState::Active => unreachable!(),
            }
            .expect("transition command");
            let event = decide(Some(&aggregate), &command).expect("legal transition");
            aggregate = evolve(Some(aggregate), &event).expect("legal evolution");
            assert_eq!(aggregate.state(), state, "transition {index}");
        }
        let terminal = GrantCommand::revoke(
            parsed!(GrantCommandId, "grant-cmd:terminal"),
            aggregate.snapshot_id().clone(),
            aggregate.version().clone(),
        )
        .expect("command");
        assert_eq!(
            decide(Some(&aggregate), &terminal),
            Err(GrantDecisionError::TerminalState)
        );
    }
    #[test]
    fn version_sequence_identity_and_evidence_mismatch_fail_closed() {
        let fixture = Fixture::new();
        let (event, aggregate) = fixture.issued();
        let mut corrupt = event.clone();
        corrupt.post_version = parsed!(GrantVersion, "grant-version:2");
        assert_eq!(
            evolve(None, &corrupt),
            Err(GrantReplayError::VersionMismatch)
        );
        corrupt = event.clone();
        corrupt.snapshot_id = parsed!(GrantSnapshotId, "grant:other");
        assert_eq!(
            evolve(None, &corrupt),
            Err(GrantReplayError::SnapshotIdentityMismatch)
        );
        corrupt = event;
        if let GrantEventPayload::Issued(evidence) = &mut corrupt.payload {
            evidence.package_digest = Sha256Digest::from_bytes(b"forged");
        }
        assert_eq!(
            evolve(None, &corrupt),
            Err(GrantReplayError::AdmissionEvidenceMismatch)
        );
        assert_eq!(aggregate.last_sequence().get(), 1);
    }
    #[test]
    fn replay_reconstructs_and_rejects_gap_duplicate_reorder_overflow_and_forgery() {
        let fixture = Fixture::new();
        let (issued, aggregate) = fixture.issued();
        let stale_command = GrantCommand::mark_stale(
            parsed!(GrantCommandId, "grant-cmd:stale-replay"),
            aggregate.snapshot_id().clone(),
            aggregate.version().clone(),
            GrantInvalidationReason::PolicyChanged,
        )
        .expect("stale");
        let stale = decide(Some(&aggregate), &stale_command).expect("stale event");
        assert_eq!(
            replay([&issued, &stale])
                .expect("replay")
                .expect("snapshot")
                .state(),
            GrantState::Stale
        );
        assert_eq!(
            replay([&issued, &issued]),
            Err(GrantReplayError::DuplicateCommandId)
        );
        let mut duplicate_approval = stale.clone();
        duplicate_approval.payload = GrantEventPayload::Replaced {
            evidence: fixture.evidence("grant:primary", "grant-approval:primary"),
            change_class: GrantChangeClass::Unchanged,
        };
        assert_eq!(
            replay([&issued, &duplicate_approval]),
            Err(GrantReplayError::DuplicateApprovalId)
        );
        assert!(matches!(
            replay([&stale, &issued]),
            Err(GrantReplayError::SequenceGap | GrantReplayError::InitialEventNotIssued)
        ));
        let mut gap = stale.clone();
        gap.sequence = GrantEventSequence(3);
        gap.post_version = parsed!(GrantVersion, "grant-version:3");
        assert_eq!(replay([&issued, &gap]), Err(GrantReplayError::SequenceGap));
        let mut duplicate = stale.clone();
        duplicate.sequence = GrantEventSequence(1);
        duplicate.post_version = parsed!(GrantVersion, "grant-version:1");
        assert_eq!(
            replay([&issued, &duplicate]),
            Err(GrantReplayError::SequenceDuplicate)
        );
        let mut bad_snapshot = issued.clone();
        bad_snapshot.snapshot_id = parsed!(GrantSnapshotId, "snapshot:bad");
        assert_eq!(
            replay([&bad_snapshot]),
            Err(GrantReplayError::SnapshotIdentityMismatch)
        );
        let mut bad_evidence_snapshot = issued.clone();
        if let GrantEventPayload::Issued(evidence) = &mut bad_evidence_snapshot.payload {
            evidence.snapshot_id = parsed!(GrantSnapshotId, "snapshot:bad-evidence");
            evidence.evidence_digest = digest_evidence(evidence);
        }
        assert_eq!(
            replay([&bad_evidence_snapshot]),
            Err(GrantReplayError::AdmissionEvidenceMismatch)
        );
        let private_capability = parsed!(CapabilityId, "user.own_academic_snapshot.read");
        let private_scope =
            GrantScope::tenant_private_user(fixture.tenant.clone(), fixture.user.clone())
                .expect("private scope");
        let mut private_evidence = GrantAdmissionEvidence::from_authority_bindings(
            parsed!(GrantSnapshotId, "grant:private"),
            parsed!(GrantApprovalId, "grant-approval:private"),
            &fixture.installation,
            &fixture.package,
            private_capability,
            private_scope,
            ConfirmationPolicy::Ask,
            &fixture.registry,
        )
        .expect("private evidence");
        private_evidence.confirmation_policy = ConfirmationPolicy::Allow;
        private_evidence.evidence_digest = digest_evidence(&private_evidence);
        let mut permissive = issued.clone();
        permissive.snapshot_id = private_evidence.snapshot_id.clone();
        permissive.payload = GrantEventPayload::Issued(private_evidence);
        assert_eq!(
            replay([&permissive]),
            Err(GrantReplayError::AdmissionEvidenceMismatch)
        );
        let mut invalid_approval = issued.clone();
        if let GrantEventPayload::Issued(evidence) = &mut invalid_approval.payload {
            evidence.approval_id.0 = "approval:invalid".to_owned();
            evidence.evidence_digest = digest_evidence(evidence);
        }
        assert_eq!(
            replay([&invalid_approval]),
            Err(GrantReplayError::AdmissionEvidenceMismatch)
        );
        let mut foreign_scope = issued.clone();
        if let GrantEventPayload::Issued(evidence) = &mut foreign_scope.payload {
            evidence.scope = GrantScope::tenant_private_user(
                parsed!(TenantId, "tenant:foreign"),
                fixture.user.clone(),
            )
            .expect("foreign scope");
            evidence.evidence_digest = digest_evidence(evidence);
        }
        assert_eq!(
            replay([&foreign_scope]),
            Err(GrantReplayError::AdmissionEvidenceMismatch)
        );
        let mut overflow = stale;
        overflow.sequence = GrantEventSequence(u64::MAX);
        overflow.post_version = parsed!(GrantVersion, "grant-version:18446744073709551615");
        let mut prior = aggregate;
        prior.last_sequence = GrantEventSequence(u64::MAX);
        prior.version = parsed!(GrantVersion, "grant-version:18446744073709551615");
        assert_eq!(
            evolve(Some(prior), &overflow),
            Err(GrantReplayError::SequenceOverflow)
        );
    }
    #[test]
    fn repository_exact_retry_and_command_conflict_are_idempotent() {
        let fixture = Fixture::new();
        let mut repository = InMemoryGrantRepository::new();
        let command = fixture.issue("grant:retry", "grant-approval:retry", "grant-cmd:retry");
        let first = repository.execute(command.clone()).expect("first");
        assert_eq!(repository.execute(command).expect("retry"), first);
        let conflict = fixture.issue("grant:other", "grant-approval:other", "grant-cmd:retry");
        assert_eq!(
            repository.execute(conflict),
            Err(GrantRepositoryError::CommandConflict)
        );
        assert_eq!(
            repository
                .event_history(first.snapshot_id())
                .expect("history")
                .len(),
            1
        );
    }
    #[test]
    fn repository_persists_domain_rejection_receipts() {
        let mut repository = InMemoryGrantRepository::new();
        let missing = GrantCommand::revoke(
            parsed!(GrantCommandId, "grant-cmd:missing"),
            parsed!(GrantSnapshotId, "grant:missing"),
            parsed!(GrantVersion, "grant-version:1"),
        )
        .expect("command");
        let receipt = repository
            .execute(missing.clone())
            .expect("rejection receipt");
        assert_eq!(rejected(&receipt), GrantDecisionError::AggregateMissing);
        assert_eq!(repository.execute(missing).expect("exact retry"), receipt);
        assert_eq!(
            repository.load_exact(receipt.snapshot_id()).expect("load"),
            None
        );
    }
    #[test]
    fn repository_approval_consumption_is_global_and_acceptance_only() {
        let fixture = Fixture::new();
        let mut repository = InMemoryGrantRepository::new();
        repository
            .execute(fixture.issue("grant:first", "grant-approval:shared", "grant-cmd:first"))
            .expect("first");
        let reused = repository
            .execute(fixture.issue("grant:second", "grant-approval:shared", "grant-cmd:second"))
            .expect("receipt");
        assert_eq!(
            rejected(&reused),
            GrantDecisionError::ApprovalAlreadyConsumed
        );
        assert_eq!(repository.consumed_approvals.len(), 1);
        assert_eq!(repository.current_authority.len(), 1);
    }
    #[test]
    fn repository_authority_conflict_receipt_survives_later_revoke() {
        let fixture = Fixture::new();
        let mut repository = InMemoryGrantRepository::new();
        let first = repository
            .execute(fixture.issue(
                "grant:first",
                "grant-approval:first",
                "grant-cmd:first-authority",
            ))
            .expect("first");
        let conflict_command = fixture.issue(
            "grant:second",
            "grant-approval:second",
            "grant-cmd:conflict",
        );
        let conflict = repository
            .execute(conflict_command.clone())
            .expect("conflict receipt");
        assert_eq!(rejected(&conflict), GrantDecisionError::AuthorityConflict);
        let snapshot = accepted(&first).1;
        let revoke = GrantCommand::revoke(
            parsed!(GrantCommandId, "grant-cmd:release"),
            snapshot.snapshot_id().clone(),
            snapshot.version().clone(),
        )
        .expect("revoke");
        repository.execute(revoke).expect("release");
        assert_eq!(
            repository.execute(conflict_command).expect("retry"),
            conflict
        );
    }
    #[test]
    fn repository_persistence_failure_is_atomic_across_all_indexes() {
        let mut repository = InMemoryGrantRepository::new();
        let fixture = Fixture::new();
        let command = fixture.issue("grant:atomic", "grant-approval:atomic", "grant-cmd:atomic");
        repository.fail_next_commit_for_testing();
        assert_eq!(
            repository.execute(command.clone()),
            Err(GrantRepositoryError::InjectedPersistenceFailure)
        );
        assert!(repository.aggregates.is_empty() && repository.events.is_empty());
        assert!(repository.command_ledger.is_empty() && repository.consumed_approvals.is_empty());
        assert!(repository.current_authority.is_empty());
        assert!(matches!(
            repository.execute(command).expect("retry").outcome(),
            GrantCommandOutcome::Accepted { .. }
        ));
    }
    #[test]
    fn current_authority_index_retains_stale_expired_and_releases_revoked() {
        let fixture = Fixture::new();
        let mut repository = InMemoryGrantRepository::new();
        let issue = repository
            .execute(fixture.issue("grant:index", "grant-approval:index", "grant-cmd:index"))
            .expect("issue");
        let mut snapshot = accepted(&issue).1.clone();
        for command in [
            GrantCommand::mark_stale(
                parsed!(GrantCommandId, "grant-cmd:index-stale"),
                snapshot.snapshot_id().clone(),
                snapshot.version().clone(),
                GrantInvalidationReason::PolicyChanged,
            )
            .expect("stale"),
            GrantCommand::expire(
                parsed!(GrantCommandId, "grant-cmd:index-expire"),
                snapshot.snapshot_id().clone(),
                parsed!(GrantVersion, "grant-version:2"),
            )
            .expect("expire"),
        ] {
            snapshot = accepted(&repository.execute(command).expect("transition"))
                .1
                .clone();
            assert!(
                repository
                    .load_current_for_authority(
                        &fixture.tenant,
                        &fixture.user,
                        fixture.installation.installation_id(),
                        &fixture.capability,
                        &fixture.scope
                    )
                    .expect("current")
                    .is_some()
            );
        }
        let revoke = GrantCommand::revoke(
            parsed!(GrantCommandId, "grant-cmd:index-revoke"),
            snapshot.snapshot_id().clone(),
            snapshot.version().clone(),
        )
        .expect("revoke");
        repository.execute(revoke).expect("revoke");
        assert_eq!(
            repository
                .load_current_for_authority(
                    &fixture.tenant,
                    &fixture.user,
                    fixture.installation.installation_id(),
                    &fixture.capability,
                    &fixture.scope
                )
                .expect("current"),
            None
        );
    }
    #[test]
    fn resolver_projection_is_pure_and_denial_side() {
        let fixture = Fixture::new();
        let (_, aggregate) = fixture.issued();
        let active = aggregate.to_resolver_snapshot();
        assert_eq!(active, aggregate.to_resolver_snapshot());
        let stale_command = GrantCommand::mark_stale(
            parsed!(GrantCommandId, "grant-cmd:projection-stale"),
            aggregate.snapshot_id().clone(),
            aggregate.version().clone(),
            GrantInvalidationReason::PolicyChanged,
        )
        .expect("stale");
        let stale_event = decide(Some(&aggregate), &stale_command).expect("stale");
        let stale = evolve(Some(aggregate), &stale_event).expect("stale aggregate");
        assert_eq!(stale.to_resolver_snapshot().state, GrantState::Stale);
        assert_eq!(active.state, GrantState::Active);
    }
    #[test]
    fn errors_events_and_receipts_contain_no_raw_secret_or_arbitrary_payload() {
        let fixture = Fixture::new();
        let command = fixture.issue(
            "grant:raw-secret",
            "grant-approval:raw-secret",
            "grant-cmd:raw-secret",
        );
        let command_rendered = format!("{command:?}");
        let event = decide(None, &command).expect("event");
        let aggregate = evolve(None, &event).expect("aggregate");
        let mut repository = InMemoryGrantRepository::new();
        let receipt = repository.execute(command).expect("receipt");
        for rendered in [
            command_rendered,
            format!("{event:?}"),
            format!("{aggregate:?}"),
            format!("{:?}", receipt.outcome()),
            format!("{receipt:?}"),
            format!("{repository:?}"),
            format!(
                "{:?}",
                fixture.evidence("grant:evidence", "grant-approval:raw-secret")
            ),
            GrantReplayError::AdmissionEvidenceMismatch.to_string(),
            GrantRepositoryError::DecisionRejected(GrantDecisionError::AdmissionEvidenceMismatch)
                .to_string(),
        ] {
            assert!(!rendered.contains("raw-secret"));
            assert!(!rendered.contains("arbitrary-payload"));
        }
    }
}
