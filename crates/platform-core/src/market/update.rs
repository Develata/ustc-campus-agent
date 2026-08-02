//! Pure package-update U1 declarations, immutable plan, classifier, evidence and commands.

use crate::identity::{TenantId, UserId};
use crate::invocation::{
    CapabilityClass, CapabilityId, CatalogComponentRevision, CatalogPackageRevision,
    CatalogRevision, ComponentId, ComponentKind, ExecutionIdentity, GrantSnapshotId, GrantState,
    GrantVersion, InstallationId, InstallationRevision, InvocationPolicySnapshot, Sha256Digest,
    SourcePolicyIdentity,
};
use crate::market::capability::{
    CapabilityDefinition, CapabilityPolicyChange, CapabilityRegistry, CapabilityRegistryRevision,
    CapabilityStatus, ScopeKind, compare_capability_definitions,
};
use crate::market::grant::{
    CurrentInstallationGrantSet, GrantCommand, GrantCommandId, GrantCommandOutcome,
    GrantCommandReceipt, GrantEvent, GrantEventKind, GrantEventSequence, GrantInvalidationReason,
    GrantReplayError, GrantRepository, GrantRepositoryError, GrantSnapshot,
    InMemoryGrantRepository, replay as grant_replay,
};
use crate::market::installation::{
    ConfigurationRevision, InMemoryInstallationRepository, InstallationCommand,
    InstallationCommandId, InstallationCommandOutcome, InstallationCommandReceipt,
    InstallationEvent, InstallationEventKind, InstallationEventSequence, InstallationPackagePin,
    InstallationReplayError, InstallationRepository, InstallationRepositoryError,
    InstallationSnapshot, ManagedInstallationState, replay as installation_replay,
};
use crate::market::{
    CatalogReadModel, ComponentDeclaration, PackageTier, ValidatedPackageManifest,
};
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt;

const UPDATE_ID_PREFIX: &str = "update:";
const COMMAND_ID_PREFIX: &str = "update-cmd:";
const APPROVAL_ID_PREFIX: &str = "update-approval:";
const EVIDENCE_ID_PREFIX: &str = "update-evidence:";
const REVISION_PREFIX: &str = "update-revision:";

const PLAN_DOMAIN: &[u8] = b"market-package-update-plan/v0\0";
const APPROVAL_EVIDENCE_DOMAIN: &[u8] = b"market-update-approval-evidence/v0\0";
const READINESS_EVIDENCE_DOMAIN: &[u8] = b"market-update-readiness-evidence/v0\0";
const CONFIRMATION_EVIDENCE_DOMAIN: &[u8] = b"market-update-confirmation-evidence/v0\0";
const ROLLBACK_EVIDENCE_DOMAIN: &[u8] = b"market-update-rollback-readiness-evidence/v0\0";
const COMPONENT_AUTHORITY_DOMAIN: &[u8] = b"market-update-component-authority/v0\0";
const CAPABILITY_AUTHORITY_DOMAIN: &[u8] = b"market-update-capability-authority/v0\0";

/// Construction failures are category-only and contain no rejected source payload.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpdateConstructionError {
    InvalidUpdateId,
    InvalidCommandId,
    InvalidApprovalId,
    InvalidEvidenceId,
    InvalidUpdateRevision,
    InvalidEventSequence,
    InstallationTerminal,
    PackageIdentityMismatch,
    TargetEqualsRollback,
    TargetUnpublishedOrRevoked,
    TargetCapabilityMissingOrInactive,
    ForbiddenAdministrativeCapability,
    DuplicateComponentOrCapability,
    AuthorityClassificationIncomplete,
    ApprovalEvidenceIncoherent,
    ReadinessEvidenceIncoherent,
    ConfirmationEvidenceIncoherent,
    RollbackEvidenceIncoherent,
}

impl fmt::Display for UpdateConstructionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "package update value rejected: {self:?}")
    }
}
impl Error for UpdateConstructionError {}

/// Stable decision failures for later U2 state checks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpdateDecisionError {
    AggregateMissing,
    AggregateAlreadyPresent,
    ActiveUpdateConflict,
    ApprovalAlreadyConsumed,
    InstallationMissing,
    InstallationTerminal,
    UpdateIdentityMismatch,
    UpdateRevisionMismatch,
    InstallationRevisionMismatch,
    InstallationPinMismatch,
    ConfigurationChanged,
    InstallationMustBeDisabled,
    PlanMismatch,
    AuthorityClassificationMismatch,
    CatalogAuthorityChanged,
    ApprovalMissingOrMismatch,
    ReadinessMissingOrMismatch,
    ActiveGrantBindingMismatch,
    ConfirmationEvidenceMismatch,
    RollbackUnavailable,
    RollbackEvidenceMismatch,
    CoupledInstallationEventMismatch,
    GrantSetConflict,
    IllegalTransition,
    SequenceOverflow,
}
impl fmt::Display for UpdateDecisionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "package update command rejected: {self:?}")
    }
}
impl Error for UpdateDecisionError {}

/// Stable replay failures for later U2 event replay.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpdateReplayError {
    NonStagedInitialEvent,
    SequenceMismatch,
    DuplicateCommandId,
    DuplicateApprovalId,
    PostTerminalEvent,
    IllegalTransition,
    RevisionMismatch,
    IdentityMismatch,
    PlanMismatch,
    EvidenceMismatch,
    SubordinateReferenceMismatch,
    SequenceOverflow,
}
impl fmt::Display for UpdateReplayError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "package update event replay rejected: {self:?}")
    }
}
impl Error for UpdateReplayError {}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PackageUpdateId(String);
impl PackageUpdateId {
    pub fn parse(value: impl Into<String>) -> Result<Self, UpdateConstructionError> {
        checked_prefixed(value.into(), UPDATE_ID_PREFIX, 121)
            .map(Self)
            .ok_or(UpdateConstructionError::InvalidUpdateId)
    }
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
impl fmt::Debug for PackageUpdateId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("PackageUpdateId(<redacted>)")
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UpdateCommandId(String);
impl UpdateCommandId {
    pub fn parse(value: impl Into<String>) -> Result<Self, UpdateConstructionError> {
        checked_prefixed(value.into(), COMMAND_ID_PREFIX, 117)
            .map(Self)
            .ok_or(UpdateConstructionError::InvalidCommandId)
    }
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
impl fmt::Debug for UpdateCommandId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("UpdateCommandId(<redacted>)")
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UpdateApprovalId(String);
impl UpdateApprovalId {
    pub fn parse(value: impl Into<String>) -> Result<Self, UpdateConstructionError> {
        checked_prefixed(value.into(), APPROVAL_ID_PREFIX, 112)
            .map(Self)
            .ok_or(UpdateConstructionError::InvalidApprovalId)
    }
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
impl fmt::Debug for UpdateApprovalId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("UpdateApprovalId(<redacted>)")
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UpdateEvidenceId(String);
impl UpdateEvidenceId {
    pub fn parse(value: impl Into<String>) -> Result<Self, UpdateConstructionError> {
        checked_prefixed(value.into(), EVIDENCE_ID_PREFIX, 112)
            .map(Self)
            .ok_or(UpdateConstructionError::InvalidEvidenceId)
    }
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
impl fmt::Debug for UpdateEvidenceId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("UpdateEvidenceId(<redacted>)")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UpdateEventSequence(u64);
impl UpdateEventSequence {
    pub fn new(value: u64) -> Result<Self, UpdateConstructionError> {
        if value == 0 {
            Err(UpdateConstructionError::InvalidEventSequence)
        } else {
            Ok(Self(value))
        }
    }
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
    pub fn next(self) -> Result<Self, UpdateConstructionError> {
        self.0
            .checked_add(1)
            .ok_or(UpdateConstructionError::InvalidEventSequence)
            .and_then(Self::new)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UpdateRevision(String);
impl UpdateRevision {
    pub fn parse(value: impl Into<String>) -> Result<Self, UpdateConstructionError> {
        let value = value.into();
        let Some(rest) = value.strip_prefix(REVISION_PREFIX) else {
            return Err(UpdateConstructionError::InvalidUpdateRevision);
        };
        if is_nonzero_decimal(rest) {
            Ok(Self(value))
        } else {
            Err(UpdateConstructionError::InvalidUpdateRevision)
        }
    }
    pub fn for_sequence(sequence: UpdateEventSequence) -> Result<Self, UpdateConstructionError> {
        Self::parse(format!("{REVISION_PREFIX}{}", sequence.get()))
    }
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpdateState {
    Staged,
    Ready,
    AppliedPendingConfirmation,
    Confirmed,
    RolledBack,
    Cancelled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpdateChangeClass {
    Unchanged,
    Narrowed,
    ReapprovalRequired,
}

#[derive(Clone, PartialEq, Eq)]
struct PlanPackageAuthority {
    catalog_revision: CatalogRevision,
    catalog_digest: Sha256Digest,
    package: ValidatedPackageManifest,
    source_policy: SourcePolicyIdentity,
    publication_components: Vec<CatalogComponentRevision>,
    component_authority_digest: Sha256Digest,
    registry_revision: CapabilityRegistryRevision,
    registry_digest: Sha256Digest,
    capability_definitions: Vec<CapabilityDefinition>,
    capability_authority_digest: Sha256Digest,
}
impl fmt::Debug for PlanPackageAuthority {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("PlanPackageAuthority")
            .field("catalog", &"<redacted>")
            .field("package", &"<redacted>")
            .field("component_count", &self.publication_components.len())
            .field("capability_count", &self.capability_definitions.len())
            .field("registry", &"<redacted>")
            .finish()
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct PackageUpdatePlan {
    update_id: PackageUpdateId,
    tenant_id: TenantId,
    user_id: UserId,
    installation_id: InstallationId,
    staged_installation_revision: InstallationRevision,
    staged_configuration_revision: ConfigurationRevision,
    staged_configuration_digest: Sha256Digest,
    rollback_pin: InstallationPackagePin,
    target_pin: InstallationPackagePin,
    rollback: PlanPackageAuthority,
    target: PlanPackageAuthority,
    change_class: UpdateChangeClass,
    plan_digest: Sha256Digest,
}

impl PackageUpdatePlan {
    #[allow(clippy::too_many_arguments)]
    fn compute(
        update_id: PackageUpdateId,
        installation: &InstallationSnapshot,
        target_pin: InstallationPackagePin,
        rollback_catalog: &CatalogReadModel,
        rollback_publications: &[CatalogPackageRevision],
        target_catalog: &CatalogReadModel,
        target_publications: &[CatalogPackageRevision],
        rollback_registry: &CapabilityRegistry,
        target_registry: &CapabilityRegistry,
    ) -> Result<Self, UpdateConstructionError> {
        let rollback_pin = installation.package_pin().clone();
        if rollback_pin.package_id() != target_pin.package_id() {
            return Err(UpdateConstructionError::PackageIdentityMismatch);
        }
        if rollback_pin == target_pin
            || (rollback_pin.package_version() == target_pin.package_version()
                && rollback_pin.package_digest() == target_pin.package_digest())
        {
            return Err(UpdateConstructionError::TargetEqualsRollback);
        }
        let rollback = validate_package_authority(
            &rollback_pin,
            rollback_catalog,
            rollback_publications,
            rollback_registry,
            false,
        )?;
        let target = validate_package_authority(
            &target_pin,
            target_catalog,
            target_publications,
            target_registry,
            true,
        )?;
        if rollback.package.package_id() != target.package.package_id() {
            return Err(UpdateConstructionError::PackageIdentityMismatch);
        }
        let change_class = classify_update(&rollback, &target)?;
        let mut plan = Self {
            update_id,
            tenant_id: installation.tenant_id().clone(),
            user_id: installation.user_id().clone(),
            installation_id: installation.installation_id().clone(),
            staged_installation_revision: installation.revision().clone(),
            staged_configuration_revision: installation.configuration_revision(),
            staged_configuration_digest: installation.configuration().digest().clone(),
            rollback_pin,
            target_pin,
            rollback,
            target,
            change_class,
            plan_digest: Sha256Digest::from_bytes(&[]),
        };
        plan.plan_digest = digest_plan(&plan);
        Ok(plan)
    }

    #[must_use]
    pub const fn update_id(&self) -> &PackageUpdateId {
        &self.update_id
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
    pub const fn staged_installation_revision(&self) -> &InstallationRevision {
        &self.staged_installation_revision
    }
    #[must_use]
    pub const fn staged_configuration_revision(&self) -> ConfigurationRevision {
        self.staged_configuration_revision
    }
    #[must_use]
    pub const fn staged_configuration_digest(&self) -> &Sha256Digest {
        &self.staged_configuration_digest
    }
    #[must_use]
    pub const fn rollback_pin(&self) -> &InstallationPackagePin {
        &self.rollback_pin
    }
    #[must_use]
    pub const fn target_pin(&self) -> &InstallationPackagePin {
        &self.target_pin
    }
    #[must_use]
    pub const fn rollback_package(&self) -> &ValidatedPackageManifest {
        &self.rollback.package
    }
    #[must_use]
    pub const fn target_package(&self) -> &ValidatedPackageManifest {
        &self.target.package
    }
    #[must_use]
    pub fn rollback_components(&self) -> &[CatalogComponentRevision] {
        &self.rollback.publication_components
    }
    #[must_use]
    pub fn target_components(&self) -> &[CatalogComponentRevision] {
        &self.target.publication_components
    }
    #[must_use]
    pub const fn rollback_source_policy_identity(&self) -> &SourcePolicyIdentity {
        &self.rollback.source_policy
    }
    #[must_use]
    pub const fn target_source_policy_identity(&self) -> &SourcePolicyIdentity {
        &self.target.source_policy
    }
    #[must_use]
    pub fn rollback_component_declarations(&self) -> &[ComponentDeclaration] {
        self.rollback.package.components()
    }
    #[must_use]
    pub fn target_component_declarations(&self) -> &[ComponentDeclaration] {
        self.target.package.components()
    }
    #[must_use]
    pub fn rollback_capability_definitions(&self) -> &[CapabilityDefinition] {
        &self.rollback.capability_definitions
    }
    #[must_use]
    pub fn target_capability_definitions(&self) -> &[CapabilityDefinition] {
        &self.target.capability_definitions
    }
    #[must_use]
    pub const fn rollback_catalog_revision(&self) -> &CatalogRevision {
        &self.rollback.catalog_revision
    }
    #[must_use]
    pub const fn target_catalog_revision(&self) -> &CatalogRevision {
        &self.target.catalog_revision
    }
    #[must_use]
    pub const fn rollback_catalog_digest(&self) -> &Sha256Digest {
        &self.rollback.catalog_digest
    }
    #[must_use]
    pub const fn target_catalog_digest(&self) -> &Sha256Digest {
        &self.target.catalog_digest
    }
    #[must_use]
    pub const fn rollback_registry_revision(&self) -> &CapabilityRegistryRevision {
        &self.rollback.registry_revision
    }
    #[must_use]
    pub const fn target_registry_revision(&self) -> &CapabilityRegistryRevision {
        &self.target.registry_revision
    }
    #[must_use]
    pub const fn rollback_registry_digest(&self) -> &Sha256Digest {
        &self.rollback.registry_digest
    }
    #[must_use]
    pub const fn target_registry_digest(&self) -> &Sha256Digest {
        &self.target.registry_digest
    }
    #[must_use]
    pub const fn rollback_component_authority_digest(&self) -> &Sha256Digest {
        &self.rollback.component_authority_digest
    }
    #[must_use]
    pub const fn target_component_authority_digest(&self) -> &Sha256Digest {
        &self.target.component_authority_digest
    }
    #[must_use]
    pub const fn rollback_capability_authority_digest(&self) -> &Sha256Digest {
        &self.rollback.capability_authority_digest
    }
    #[must_use]
    pub const fn target_capability_authority_digest(&self) -> &Sha256Digest {
        &self.target.capability_authority_digest
    }
    #[must_use]
    pub const fn change_class(&self) -> UpdateChangeClass {
        self.change_class
    }
    #[must_use]
    pub const fn plan_digest(&self) -> &Sha256Digest {
        &self.plan_digest
    }
}

impl fmt::Debug for PackageUpdatePlan {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("PackageUpdatePlan")
            .field("identity", &"<redacted>")
            .field("staged_installation_revision", &"<redacted>")
            .field("change_class", &self.change_class)
            .field("plan_digest", &"<redacted>")
            .field("authority", &"<redacted>")
            .finish()
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct UpdateApprovalEvidence {
    approval_id: UpdateApprovalId,
    plan_digest: Sha256Digest,
    change_class: UpdateChangeClass,
    staged_installation_revision: InstallationRevision,
    staged_configuration_digest: Sha256Digest,
    approval_evidence_digest: Sha256Digest,
    evidence_digest: Sha256Digest,
}
impl UpdateApprovalEvidence {
    #[allow(dead_code)]
    pub(in crate::market) fn from_plan(
        approval_id: UpdateApprovalId,
        plan: &PackageUpdatePlan,
        approval_evidence_digest: Sha256Digest,
    ) -> Result<Self, UpdateConstructionError> {
        let mut value = Self {
            approval_id,
            plan_digest: plan.plan_digest().clone(),
            change_class: plan.change_class(),
            staged_installation_revision: plan.staged_installation_revision().clone(),
            staged_configuration_digest: plan.staged_configuration_digest().clone(),
            approval_evidence_digest,
            evidence_digest: Sha256Digest::from_bytes(&[]),
        };
        value.evidence_digest = digest_update_approval_evidence(&value);
        Ok(value)
    }
    #[must_use]
    pub const fn approval_id(&self) -> &UpdateApprovalId {
        &self.approval_id
    }
    #[must_use]
    pub const fn plan_digest(&self) -> &Sha256Digest {
        &self.plan_digest
    }
    #[must_use]
    pub const fn change_class(&self) -> UpdateChangeClass {
        self.change_class
    }
    #[must_use]
    pub const fn staged_installation_revision(&self) -> &InstallationRevision {
        &self.staged_installation_revision
    }
    #[must_use]
    pub const fn staged_configuration_digest(&self) -> &Sha256Digest {
        &self.staged_configuration_digest
    }
    #[must_use]
    pub const fn approval_evidence_digest(&self) -> &Sha256Digest {
        &self.approval_evidence_digest
    }
    #[must_use]
    pub const fn evidence_digest(&self) -> &Sha256Digest {
        &self.evidence_digest
    }
}
impl fmt::Debug for UpdateApprovalEvidence {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("UpdateApprovalEvidence")
            .field("approval_id", &"<redacted>")
            .field("plan_digest", &"<redacted>")
            .field("change_class", &self.change_class)
            .field("evidence_digest", &"<redacted>")
            .finish()
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct UpdateReadinessEvidence {
    evidence_id: UpdateEvidenceId,
    plan_digest: Sha256Digest,
    target_package_digest: Sha256Digest,
    rollback_package_digest: Sha256Digest,
    target_component_authority_digest: Sha256Digest,
    rollback_component_authority_digest: Sha256Digest,
    staged_installation_revision: InstallationRevision,
    staged_configuration_digest: Sha256Digest,
    verified_target_artifact_set_digest: Sha256Digest,
    verified_rollback_artifact_set_digest: Sha256Digest,
    target_configuration_admission_snapshot_digest: Sha256Digest,
    target_source_execution_policy_admission_snapshot_digest: Sha256Digest,
    target_catalog_revision: CatalogRevision,
    rollback_catalog_revision: CatalogRevision,
    target_registry_revision: CapabilityRegistryRevision,
    rollback_registry_revision: CapabilityRegistryRevision,
    evidence_digest: Sha256Digest,
}
impl UpdateReadinessEvidence {
    #[allow(dead_code)]
    #[allow(clippy::too_many_arguments)]
    pub(in crate::market) fn from_plan(
        evidence_id: UpdateEvidenceId,
        plan: &PackageUpdatePlan,
        verified_target_artifact_set_digest: Sha256Digest,
        verified_rollback_artifact_set_digest: Sha256Digest,
        target_configuration_admission_snapshot_digest: Sha256Digest,
        target_source_execution_policy_admission_snapshot_digest: Sha256Digest,
    ) -> Result<Self, UpdateConstructionError> {
        let mut value = Self {
            evidence_id,
            plan_digest: plan.plan_digest().clone(),
            target_package_digest: plan.target_pin().package_digest().clone(),
            rollback_package_digest: plan.rollback_pin().package_digest().clone(),
            target_component_authority_digest: plan.target_component_authority_digest().clone(),
            rollback_component_authority_digest: plan.rollback_component_authority_digest().clone(),
            staged_installation_revision: plan.staged_installation_revision().clone(),
            staged_configuration_digest: plan.staged_configuration_digest().clone(),
            verified_target_artifact_set_digest,
            verified_rollback_artifact_set_digest,
            target_configuration_admission_snapshot_digest,
            target_source_execution_policy_admission_snapshot_digest,
            target_catalog_revision: plan.target_catalog_revision().clone(),
            rollback_catalog_revision: plan.rollback_catalog_revision().clone(),
            target_registry_revision: plan.target_registry_revision().clone(),
            rollback_registry_revision: plan.rollback_registry_revision().clone(),
            evidence_digest: Sha256Digest::from_bytes(&[]),
        };
        value.evidence_digest = digest_update_readiness_evidence(&value);
        Ok(value)
    }
    #[must_use]
    pub const fn evidence_id(&self) -> &UpdateEvidenceId {
        &self.evidence_id
    }
    #[must_use]
    pub const fn plan_digest(&self) -> &Sha256Digest {
        &self.plan_digest
    }
    #[must_use]
    pub const fn target_package_digest(&self) -> &Sha256Digest {
        &self.target_package_digest
    }
    #[must_use]
    pub const fn rollback_package_digest(&self) -> &Sha256Digest {
        &self.rollback_package_digest
    }
    #[must_use]
    pub const fn target_component_authority_digest(&self) -> &Sha256Digest {
        &self.target_component_authority_digest
    }
    #[must_use]
    pub const fn rollback_component_authority_digest(&self) -> &Sha256Digest {
        &self.rollback_component_authority_digest
    }
    #[must_use]
    pub const fn staged_installation_revision(&self) -> &InstallationRevision {
        &self.staged_installation_revision
    }
    #[must_use]
    pub const fn staged_configuration_digest(&self) -> &Sha256Digest {
        &self.staged_configuration_digest
    }
    #[must_use]
    pub const fn verified_target_artifact_set_digest(&self) -> &Sha256Digest {
        &self.verified_target_artifact_set_digest
    }
    #[must_use]
    pub const fn verified_rollback_artifact_set_digest(&self) -> &Sha256Digest {
        &self.verified_rollback_artifact_set_digest
    }
    #[must_use]
    pub const fn target_configuration_admission_snapshot_digest(&self) -> &Sha256Digest {
        &self.target_configuration_admission_snapshot_digest
    }
    #[must_use]
    pub const fn target_source_execution_policy_admission_snapshot_digest(&self) -> &Sha256Digest {
        &self.target_source_execution_policy_admission_snapshot_digest
    }
    #[must_use]
    pub const fn target_catalog_revision(&self) -> &CatalogRevision {
        &self.target_catalog_revision
    }
    #[must_use]
    pub const fn rollback_catalog_revision(&self) -> &CatalogRevision {
        &self.rollback_catalog_revision
    }
    #[must_use]
    pub const fn target_registry_revision(&self) -> &CapabilityRegistryRevision {
        &self.target_registry_revision
    }
    #[must_use]
    pub const fn rollback_registry_revision(&self) -> &CapabilityRegistryRevision {
        &self.rollback_registry_revision
    }
    #[must_use]
    pub const fn evidence_digest(&self) -> &Sha256Digest {
        &self.evidence_digest
    }
}
impl fmt::Debug for UpdateReadinessEvidence {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("UpdateReadinessEvidence")
            .field("evidence_id", &"<redacted>")
            .field("plan_digest", &"<redacted>")
            .field("evidence_digest", &"<redacted>")
            .field("authority", &"<redacted>")
            .finish()
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct UpdateConfirmationEvidence {
    evidence_id: UpdateEvidenceId,
    update_id: PackageUpdateId,
    expected_update_revision: UpdateRevision,
    applied_event_digest: Sha256Digest,
    installation_id: InstallationId,
    installation_revision: InstallationRevision,
    target_pin_digest: Sha256Digest,
    installation_state_digest: Sha256Digest,
    evidence_digest: Sha256Digest,
}
impl UpdateConfirmationEvidence {
    #[allow(dead_code)]
    #[allow(clippy::too_many_arguments)]
    pub(in crate::market) fn from_bindings(
        evidence_id: UpdateEvidenceId,
        update_id: PackageUpdateId,
        expected_update_revision: UpdateRevision,
        applied_event_digest: Sha256Digest,
        installation_id: InstallationId,
        installation_revision: InstallationRevision,
        target_pin_digest: Sha256Digest,
        installation_state_digest: Sha256Digest,
    ) -> Result<Self, UpdateConstructionError> {
        let mut value = Self {
            evidence_id,
            update_id,
            expected_update_revision,
            applied_event_digest,
            installation_id,
            installation_revision,
            target_pin_digest,
            installation_state_digest,
            evidence_digest: Sha256Digest::from_bytes(&[]),
        };
        value.evidence_digest = digest_update_confirmation_evidence(&value);
        Ok(value)
    }
    #[must_use]
    pub const fn evidence_id(&self) -> &UpdateEvidenceId {
        &self.evidence_id
    }
    #[must_use]
    pub const fn update_id(&self) -> &PackageUpdateId {
        &self.update_id
    }
    #[must_use]
    pub const fn expected_update_revision(&self) -> &UpdateRevision {
        &self.expected_update_revision
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
    pub const fn applied_event_digest(&self) -> &Sha256Digest {
        &self.applied_event_digest
    }
    #[must_use]
    pub const fn target_pin_digest(&self) -> &Sha256Digest {
        &self.target_pin_digest
    }
    #[must_use]
    pub const fn installation_state_digest(&self) -> &Sha256Digest {
        &self.installation_state_digest
    }
    #[must_use]
    pub const fn evidence_digest(&self) -> &Sha256Digest {
        &self.evidence_digest
    }
}
impl fmt::Debug for UpdateConfirmationEvidence {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("UpdateConfirmationEvidence")
            .field("evidence_id", &"<redacted>")
            .field("update_id", &"<redacted>")
            .field("evidence_digest", &"<redacted>")
            .finish()
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct RollbackReadinessEvidence {
    evidence_id: UpdateEvidenceId,
    update_id: PackageUpdateId,
    expected_update_revision: UpdateRevision,
    rollback_pin_digest: Sha256Digest,
    current_target_installation_revision: InstallationRevision,
    current_configuration_revision: ConfigurationRevision,
    current_configuration_digest: Sha256Digest,
    verified_rollback_artifact_set_digest: Sha256Digest,
    rollback_admission_snapshot_digest: Sha256Digest,
    evidence_digest: Sha256Digest,
}
impl RollbackReadinessEvidence {
    #[allow(dead_code)]
    #[allow(clippy::too_many_arguments)]
    pub(in crate::market) fn from_bindings(
        evidence_id: UpdateEvidenceId,
        update_id: PackageUpdateId,
        expected_update_revision: UpdateRevision,
        rollback_pin_digest: Sha256Digest,
        current_target_installation_revision: InstallationRevision,
        current_configuration_revision: ConfigurationRevision,
        current_configuration_digest: Sha256Digest,
        verified_rollback_artifact_set_digest: Sha256Digest,
        rollback_admission_snapshot_digest: Sha256Digest,
    ) -> Result<Self, UpdateConstructionError> {
        let mut value = Self {
            evidence_id,
            update_id,
            expected_update_revision,
            rollback_pin_digest,
            current_target_installation_revision,
            current_configuration_revision,
            current_configuration_digest,
            verified_rollback_artifact_set_digest,
            rollback_admission_snapshot_digest,
            evidence_digest: Sha256Digest::from_bytes(&[]),
        };
        value.evidence_digest = digest_rollback_readiness_evidence(&value);
        Ok(value)
    }
    #[must_use]
    pub const fn evidence_id(&self) -> &UpdateEvidenceId {
        &self.evidence_id
    }
    #[must_use]
    pub const fn update_id(&self) -> &PackageUpdateId {
        &self.update_id
    }
    #[must_use]
    pub const fn expected_update_revision(&self) -> &UpdateRevision {
        &self.expected_update_revision
    }
    #[must_use]
    pub const fn current_target_installation_revision(&self) -> &InstallationRevision {
        &self.current_target_installation_revision
    }
    #[must_use]
    pub const fn rollback_pin_digest(&self) -> &Sha256Digest {
        &self.rollback_pin_digest
    }
    #[must_use]
    pub const fn current_configuration_revision(&self) -> ConfigurationRevision {
        self.current_configuration_revision
    }
    #[must_use]
    pub const fn current_configuration_digest(&self) -> &Sha256Digest {
        &self.current_configuration_digest
    }
    #[must_use]
    pub const fn verified_rollback_artifact_set_digest(&self) -> &Sha256Digest {
        &self.verified_rollback_artifact_set_digest
    }
    #[must_use]
    pub const fn rollback_admission_snapshot_digest(&self) -> &Sha256Digest {
        &self.rollback_admission_snapshot_digest
    }
    #[must_use]
    pub const fn evidence_digest(&self) -> &Sha256Digest {
        &self.evidence_digest
    }
}
impl fmt::Debug for RollbackReadinessEvidence {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RollbackReadinessEvidence")
            .field("evidence_id", &"<redacted>")
            .field("update_id", &"<redacted>")
            .field("evidence_digest", &"<redacted>")
            .field("authority", &"<redacted>")
            .finish()
    }
}

#[allow(clippy::large_enum_variant)]
#[derive(Clone, PartialEq, Eq)]
enum UpdateCommandAction {
    Stage {
        plan: PackageUpdatePlan,
    },
    RecordApproval {
        expected_update_revision: UpdateRevision,
        approval: UpdateApprovalEvidence,
        readiness: UpdateReadinessEvidence,
    },
    Apply {
        expected_update_revision: UpdateRevision,
        expected_installation_revision: InstallationRevision,
    },
    ConfirmAppliedUpdate {
        expected_update_revision: UpdateRevision,
        expected_installation_revision: InstallationRevision,
        evidence: UpdateConfirmationEvidence,
    },
    Rollback {
        expected_update_revision: UpdateRevision,
        expected_installation_revision: InstallationRevision,
        evidence: RollbackReadinessEvidence,
    },
    Cancel {
        expected_update_revision: UpdateRevision,
    },
    CancelAfterTerminalInstallation {
        expected_update_revision: UpdateRevision,
        expected_terminal_installation_revision: InstallationRevision,
    },
}
impl fmt::Debug for UpdateCommandAction {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Stage { plan: _ } => formatter
                .debug_struct("Stage")
                .field("plan_digest", &"<redacted>")
                .finish(),
            Self::RecordApproval { .. } => formatter
                .debug_struct("RecordApproval")
                .field("expected_update_revision", &"<redacted>")
                .field("authority", &"<redacted>")
                .finish(),
            Self::Apply {
                expected_update_revision: _,
                expected_installation_revision: _,
            } => formatter
                .debug_struct("Apply")
                .field("expected_update_revision", &"<redacted>")
                .field("expected_installation_revision", &"<redacted>")
                .finish(),
            Self::ConfirmAppliedUpdate { .. } => formatter
                .debug_struct("ConfirmAppliedUpdate")
                .field("expected_update_revision", &"<redacted>")
                .field("expected_installation_revision", &"<redacted>")
                .finish(),
            Self::Rollback { .. } => formatter
                .debug_struct("Rollback")
                .field("expected_update_revision", &"<redacted>")
                .field("expected_installation_revision", &"<redacted>")
                .finish(),
            Self::Cancel {
                expected_update_revision: _,
            } => formatter
                .debug_struct("Cancel")
                .field("expected_update_revision", &"<redacted>")
                .finish(),
            Self::CancelAfterTerminalInstallation {
                expected_update_revision: _,
                expected_terminal_installation_revision: _,
            } => formatter
                .debug_struct("CancelAfterTerminalInstallation")
                .field("expected_update_revision", &"<redacted>")
                .field("expected_terminal_installation_revision", &"<redacted>")
                .finish(),
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct UpdateCommand {
    command_id: UpdateCommandId,
    update_id: PackageUpdateId,
    action: UpdateCommandAction,
}
impl UpdateCommand {
    #[allow(clippy::too_many_arguments)]
    pub fn stage(
        command_id: UpdateCommandId,
        update_id: PackageUpdateId,
        installation: &InstallationSnapshot,
        target_pin: InstallationPackagePin,
        rollback_catalog: &CatalogReadModel,
        rollback_publications: &[CatalogPackageRevision],
        target_catalog: &CatalogReadModel,
        target_publications: &[CatalogPackageRevision],
        rollback_registry: &CapabilityRegistry,
        target_registry: &CapabilityRegistry,
    ) -> Result<Self, UpdateConstructionError> {
        let plan = PackageUpdatePlan::compute(
            update_id.clone(),
            installation,
            target_pin,
            rollback_catalog,
            rollback_publications,
            target_catalog,
            target_publications,
            rollback_registry,
            target_registry,
        )?;
        Ok(Self {
            command_id,
            update_id,
            action: UpdateCommandAction::Stage { plan },
        })
    }

    pub fn record_approval(
        command_id: UpdateCommandId,
        update_id: PackageUpdateId,
        expected_update_revision: UpdateRevision,
        approval: UpdateApprovalEvidence,
        readiness: UpdateReadinessEvidence,
    ) -> Result<Self, UpdateConstructionError> {
        verify_update_approval_evidence(&approval)?;
        verify_update_readiness_evidence(&readiness)?;
        if approval.plan_digest() != readiness.plan_digest()
            || approval.staged_installation_revision() != readiness.staged_installation_revision()
            || approval.staged_configuration_digest() != readiness.staged_configuration_digest()
        {
            return Err(UpdateConstructionError::ApprovalEvidenceIncoherent);
        }
        Ok(Self {
            command_id,
            update_id,
            action: UpdateCommandAction::RecordApproval {
                expected_update_revision,
                approval,
                readiness,
            },
        })
    }

    pub fn apply(
        command_id: UpdateCommandId,
        update_id: PackageUpdateId,
        expected_update_revision: UpdateRevision,
        expected_installation_revision: InstallationRevision,
    ) -> Result<Self, UpdateConstructionError> {
        Ok(Self {
            command_id,
            update_id,
            action: UpdateCommandAction::Apply {
                expected_update_revision,
                expected_installation_revision,
            },
        })
    }

    pub fn confirm_applied_update(
        command_id: UpdateCommandId,
        update_id: PackageUpdateId,
        expected_update_revision: UpdateRevision,
        expected_installation_revision: InstallationRevision,
        evidence: UpdateConfirmationEvidence,
    ) -> Result<Self, UpdateConstructionError> {
        verify_update_confirmation_evidence(&evidence)?;
        if evidence.update_id() != &update_id
            || evidence.expected_update_revision() != &expected_update_revision
            || evidence.installation_revision() != &expected_installation_revision
        {
            return Err(UpdateConstructionError::ApprovalEvidenceIncoherent);
        }
        Ok(Self {
            command_id,
            update_id,
            action: UpdateCommandAction::ConfirmAppliedUpdate {
                expected_update_revision,
                expected_installation_revision,
                evidence,
            },
        })
    }

    pub fn rollback(
        command_id: UpdateCommandId,
        update_id: PackageUpdateId,
        expected_update_revision: UpdateRevision,
        expected_installation_revision: InstallationRevision,
        evidence: RollbackReadinessEvidence,
    ) -> Result<Self, UpdateConstructionError> {
        verify_rollback_readiness_evidence(&evidence)?;
        if evidence.update_id() != &update_id
            || evidence.expected_update_revision() != &expected_update_revision
            || evidence.current_target_installation_revision() != &expected_installation_revision
        {
            return Err(UpdateConstructionError::ApprovalEvidenceIncoherent);
        }
        Ok(Self {
            command_id,
            update_id,
            action: UpdateCommandAction::Rollback {
                expected_update_revision,
                expected_installation_revision,
                evidence,
            },
        })
    }

    pub fn cancel(
        command_id: UpdateCommandId,
        update_id: PackageUpdateId,
        expected_update_revision: UpdateRevision,
    ) -> Result<Self, UpdateConstructionError> {
        Ok(Self {
            command_id,
            update_id,
            action: UpdateCommandAction::Cancel {
                expected_update_revision,
            },
        })
    }

    pub fn cancel_after_terminal_installation(
        command_id: UpdateCommandId,
        update_id: PackageUpdateId,
        expected_update_revision: UpdateRevision,
        expected_terminal_installation_revision: InstallationRevision,
    ) -> Result<Self, UpdateConstructionError> {
        Ok(Self {
            command_id,
            update_id,
            action: UpdateCommandAction::CancelAfterTerminalInstallation {
                expected_update_revision,
                expected_terminal_installation_revision,
            },
        })
    }

    #[must_use]
    pub const fn command_id(&self) -> &UpdateCommandId {
        &self.command_id
    }
    #[must_use]
    pub const fn update_id(&self) -> &PackageUpdateId {
        &self.update_id
    }
}
impl fmt::Debug for UpdateCommand {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("UpdateCommand")
            .field("command_id", &self.command_id)
            .field("update_id", &self.update_id)
            .field("action", &self.action)
            .finish()
    }
}

#[derive(Clone, PartialEq, Eq)]
struct InstallationEventReference {
    installation_id: InstallationId,
    sequence: InstallationEventSequence,
    post_revision: InstallationRevision,
    command_id: InstallationCommandId,
    kind: InstallationEventKind,
    event_digest: Sha256Digest,
}
impl InstallationEventReference {
    fn from_event(
        installation_id: &InstallationId,
        event: &InstallationEvent,
        expected_kind: InstallationEventKind,
    ) -> Result<Self, UpdateDecisionError> {
        if event.kind() != expected_kind {
            return Err(UpdateDecisionError::CoupledInstallationEventMismatch);
        }
        Ok(Self {
            installation_id: installation_id.clone(),
            sequence: event.sequence(),
            post_revision: event.post_revision().clone(),
            command_id: event.command_id().clone(),
            kind: expected_kind,
            event_digest: event.canonical_coupling_digest(),
        })
    }
    fn matches_event(&self, installation_id: &InstallationId, event: &InstallationEvent) -> bool {
        self.installation_id == *installation_id
            && self.sequence == event.sequence()
            && self.post_revision == *event.post_revision()
            && self.command_id == *event.command_id()
            && self.kind == event.kind()
            && self.event_digest == event.canonical_coupling_digest()
    }
}
impl fmt::Debug for InstallationEventReference {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("InstallationEventReference(<authority-redacted>)")
    }
}

#[derive(Clone, PartialEq, Eq)]
struct GrantEventReference {
    snapshot_id: GrantSnapshotId,
    sequence: GrantEventSequence,
    post_version: GrantVersion,
    command_id: GrantCommandId,
    kind: GrantEventKind,
    event_digest: Sha256Digest,
}
impl GrantEventReference {
    fn from_event(
        event: &GrantEvent,
        expected_kind: GrantEventKind,
    ) -> Result<Self, UpdateDecisionError> {
        if event.kind() != expected_kind
            || event.invalidation_reason() != Some(GrantInvalidationReason::InstallationChanged)
        {
            return Err(UpdateDecisionError::GrantSetConflict);
        }
        Ok(Self {
            snapshot_id: event.snapshot_id().clone(),
            sequence: event.sequence(),
            post_version: event.post_version().clone(),
            command_id: event.command_id().clone(),
            kind: expected_kind,
            event_digest: event.canonical_coupling_digest(),
        })
    }
    #[allow(dead_code)]
    fn matches_event(&self, event: &GrantEvent) -> bool {
        self.snapshot_id == *event.snapshot_id()
            && self.sequence == event.sequence()
            && self.post_version == *event.post_version()
            && self.command_id == *event.command_id()
            && self.kind == event.kind()
            && self.event_digest == event.canonical_coupling_digest()
    }
}
impl fmt::Debug for GrantEventReference {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("GrantEventReference(<authority-redacted>)")
    }
}

#[derive(Clone, PartialEq, Eq)]
struct AuthorityCarrierBinding {
    target_catalog: CatalogReadModel,
    rollback_catalog: CatalogReadModel,
    target_publications: Vec<CatalogPackageRevision>,
    rollback_publications: Vec<CatalogPackageRevision>,
    target_registry: CapabilityRegistry,
    rollback_registry: CapabilityRegistry,
    policies: Vec<InvocationPolicySnapshot>,
    policy_digest: Sha256Digest,
}
impl AuthorityCarrierBinding {
    #[cfg(test)]
    fn from_parts(
        target_catalog: &CatalogReadModel,
        rollback_catalog: &CatalogReadModel,
        target_registry: &CapabilityRegistry,
        rollback_registry: &CapabilityRegistry,
        policies: &[InvocationPolicySnapshot],
        plan: &PackageUpdatePlan,
    ) -> Result<Self, UpdateConstructionError> {
        let policies = canonical_policy_snapshots(policies, plan)?;
        Ok(Self::from_complete_parts(
            target_catalog,
            &synthetic_publications_for_plan_authority(&plan.target),
            rollback_catalog,
            &synthetic_publications_for_plan_authority(&plan.rollback),
            target_registry,
            rollback_registry,
            &policies,
        ))
    }

    #[allow(clippy::too_many_arguments)]
    fn from_complete_parts(
        target_catalog: &CatalogReadModel,
        target_publications: &[CatalogPackageRevision],
        rollback_catalog: &CatalogReadModel,
        rollback_publications: &[CatalogPackageRevision],
        target_registry: &CapabilityRegistry,
        rollback_registry: &CapabilityRegistry,
        policies: &[InvocationPolicySnapshot],
    ) -> Self {
        let mut policies = policies.to_vec();
        policies.sort_by_key(policy_sort_key);
        let policy_digest = digest_policy_snapshots(&policies);
        Self {
            target_catalog: target_catalog.clone(),
            rollback_catalog: rollback_catalog.clone(),
            target_publications: target_publications.to_vec(),
            rollback_publications: rollback_publications.to_vec(),
            target_registry: target_registry.clone(),
            rollback_registry: rollback_registry.clone(),
            policies,
            policy_digest,
        }
    }

    fn plan_match_error(&self, plan: &PackageUpdatePlan) -> Option<UpdateDecisionError> {
        if self.target_catalog.catalog_revision() != plan.target_catalog_revision()
            || self.target_catalog.catalog_digest() != plan.target_catalog_digest()
            || self.rollback_catalog.catalog_revision() != plan.rollback_catalog_revision()
            || self.rollback_catalog.catalog_digest() != plan.rollback_catalog_digest()
            || self.target_registry.registry_revision() != plan.target_registry_revision()
            || self.target_registry.registry_digest() != plan.target_registry_digest()
            || self.rollback_registry.registry_revision() != plan.rollback_registry_revision()
            || self.rollback_registry.registry_digest() != plan.rollback_registry_digest()
        {
            return Some(UpdateDecisionError::CatalogAuthorityChanged);
        }
        let target = match validate_package_authority(
            plan.target_pin(),
            &self.target_catalog,
            &self.target_publications,
            &self.target_registry,
            true,
        ) {
            Ok(authority) => authority,
            Err(_) => return Some(UpdateDecisionError::AuthorityClassificationMismatch),
        };
        let rollback = match validate_package_authority(
            plan.rollback_pin(),
            &self.rollback_catalog,
            &self.rollback_publications,
            &self.rollback_registry,
            false,
        ) {
            Ok(authority) => authority,
            Err(_) => return Some(UpdateDecisionError::AuthorityClassificationMismatch),
        };
        let policies = match canonical_policy_snapshots(&self.policies, plan) {
            Ok(policies) if policies == self.policies => policies,
            _ => return Some(UpdateDecisionError::AuthorityClassificationMismatch),
        };
        let class = match classify_update(&rollback, &target) {
            Ok(class) => class,
            Err(_) => return Some(UpdateDecisionError::AuthorityClassificationMismatch),
        };
        if class != plan.change_class() {
            return Some(UpdateDecisionError::AuthorityClassificationMismatch);
        }
        if target != plan.target || rollback != plan.rollback {
            return Some(UpdateDecisionError::CatalogAuthorityChanged);
        }
        if digest_policy_snapshots(&policies) != self.policy_digest {
            return Some(UpdateDecisionError::CatalogAuthorityChanged);
        }
        None
    }

    fn matches_plan(&self, plan: &PackageUpdatePlan) -> bool {
        self.plan_match_error(plan).is_none()
    }

    fn matches_readiness(
        &self,
        plan: &PackageUpdatePlan,
        readiness: &UpdateReadinessEvidence,
    ) -> Result<(), UpdateDecisionError> {
        if let Some(error) = self.plan_match_error(plan) {
            return Err(error);
        }
        if readiness.plan_digest() == plan.plan_digest()
            && readiness.target_package_digest() == plan.target_pin().package_digest()
            && readiness.rollback_package_digest() == plan.rollback_pin().package_digest()
            && readiness.target_component_authority_digest()
                == plan.target_component_authority_digest()
            && readiness.rollback_component_authority_digest()
                == plan.rollback_component_authority_digest()
            && readiness.target_catalog_revision() == plan.target_catalog_revision()
            && readiness.rollback_catalog_revision() == plan.rollback_catalog_revision()
            && readiness.target_registry_revision() == plan.target_registry_revision()
            && readiness.rollback_registry_revision() == plan.rollback_registry_revision()
            && readiness.target_source_execution_policy_admission_snapshot_digest()
                == &self.policy_digest
        {
            Ok(())
        } else {
            Err(UpdateDecisionError::CatalogAuthorityChanged)
        }
    }

    fn matches_rollback_admission(
        &self,
        plan: &PackageUpdatePlan,
        evidence: &RollbackReadinessEvidence,
    ) -> Result<(), UpdateDecisionError> {
        if let Some(error) = self.plan_match_error(plan) {
            return Err(error);
        }
        if evidence.rollback_admission_snapshot_digest() == &self.policy_digest {
            Ok(())
        } else {
            Err(UpdateDecisionError::CatalogAuthorityChanged)
        }
    }
}

#[allow(clippy::large_enum_variant)]
#[derive(Clone, PartialEq, Eq)]
enum UpdateDecisionContextKind {
    Stage {
        installation: InstallationSnapshot,
        plan: PackageUpdatePlan,
        authority: AuthorityCarrierBinding,
    },
    RecordApproval {
        installation: InstallationSnapshot,
        authority: AuthorityCarrierBinding,
    },
    Apply {
        installation: InstallationSnapshot,
        authority: AuthorityCarrierBinding,
        current_grants: CurrentInstallationGrantSet,
        installation_event: InstallationEventReference,
        grant_events: Vec<GrantEventReference>,
    },
    ConfirmApplied {
        installation: InstallationSnapshot,
    },
    Rollback {
        installation: InstallationSnapshot,
        authority: AuthorityCarrierBinding,
        current_grants: CurrentInstallationGrantSet,
        installation_event: InstallationEventReference,
        grant_events: Vec<GrantEventReference>,
    },
    Cancel,
    CancelAfterTerminalInstallation {
        installation: InstallationSnapshot,
    },
}

/// Sealed decision authority supplied only by the market owner.
///
/// Restricted helpers are intentionally unavailable to external callers:
///
/// ```compile_fail
/// use ustc_campus_agent_core::market::update::UpdateDecisionContext;
///
/// let _forged = UpdateDecisionContext::for_cancel();
/// ```
#[derive(Clone, PartialEq, Eq)]
pub struct UpdateDecisionContext {
    kind: UpdateDecisionContextKind,
}
impl fmt::Debug for UpdateDecisionContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("UpdateDecisionContext(<authority-redacted>)")
    }
}
impl UpdateDecisionContext {
    #[allow(clippy::too_many_arguments)]
    pub(in crate::market) fn for_stage(
        command: &UpdateCommand,
        installation: &InstallationSnapshot,
        target_pin: InstallationPackagePin,
        rollback_catalog: &CatalogReadModel,
        rollback_publications: &[CatalogPackageRevision],
        target_catalog: &CatalogReadModel,
        target_publications: &[CatalogPackageRevision],
        rollback_registry: &CapabilityRegistry,
        target_registry: &CapabilityRegistry,
        policies: &[InvocationPolicySnapshot],
    ) -> Result<Self, UpdateConstructionError> {
        let plan = PackageUpdatePlan::compute(
            command.update_id.clone(),
            installation,
            target_pin,
            rollback_catalog,
            rollback_publications,
            target_catalog,
            target_publications,
            rollback_registry,
            target_registry,
        )?;
        let policies = canonical_policy_snapshots(policies, &plan)?;
        Ok(Self {
            kind: UpdateDecisionContextKind::Stage {
                installation: installation.clone(),
                authority: AuthorityCarrierBinding::from_complete_parts(
                    target_catalog,
                    target_publications,
                    rollback_catalog,
                    rollback_publications,
                    target_registry,
                    rollback_registry,
                    &policies,
                ),
                plan,
            },
        })
    }
    #[allow(clippy::too_many_arguments)]
    pub(in crate::market) fn for_record_approval(
        installation: &InstallationSnapshot,
        plan: &PackageUpdatePlan,
        target_catalog: &CatalogReadModel,
        target_publications: &[CatalogPackageRevision],
        rollback_catalog: &CatalogReadModel,
        rollback_publications: &[CatalogPackageRevision],
        target_registry: &CapabilityRegistry,
        rollback_registry: &CapabilityRegistry,
        policies: &[InvocationPolicySnapshot],
    ) -> Result<Self, UpdateDecisionError> {
        let policies = canonical_policy_snapshots(policies, plan)
            .map_err(|_| UpdateDecisionError::CatalogAuthorityChanged)?;
        Ok(Self {
            kind: UpdateDecisionContextKind::RecordApproval {
                installation: installation.clone(),
                authority: AuthorityCarrierBinding::from_complete_parts(
                    target_catalog,
                    target_publications,
                    rollback_catalog,
                    rollback_publications,
                    target_registry,
                    rollback_registry,
                    &policies,
                ),
            },
        })
    }
    #[cfg(test)]
    fn for_record_approval_for_test(
        installation: &InstallationSnapshot,
        plan: &PackageUpdatePlan,
        target_catalog: &CatalogReadModel,
        rollback_catalog: &CatalogReadModel,
        target_registry: &CapabilityRegistry,
        rollback_registry: &CapabilityRegistry,
        policies: &[InvocationPolicySnapshot],
    ) -> Result<Self, UpdateDecisionError> {
        let target_publications = synthetic_publications_for_plan_authority(&plan.target);
        let rollback_publications = synthetic_publications_for_plan_authority(&plan.rollback);
        Self::for_record_approval(
            installation,
            plan,
            target_catalog,
            &target_publications,
            rollback_catalog,
            &rollback_publications,
            target_registry,
            rollback_registry,
            policies,
        )
    }
    #[allow(clippy::too_many_arguments)]
    pub(in crate::market) fn for_apply(
        installation: &InstallationSnapshot,
        plan: &PackageUpdatePlan,
        target_catalog: &CatalogReadModel,
        target_publications: &[CatalogPackageRevision],
        rollback_catalog: &CatalogReadModel,
        rollback_publications: &[CatalogPackageRevision],
        target_registry: &CapabilityRegistry,
        rollback_registry: &CapabilityRegistry,
        policies: &[InvocationPolicySnapshot],
        current_grants: CurrentInstallationGrantSet,
        installation_event: &InstallationEvent,
        grant_events: &[GrantEvent],
    ) -> Result<Self, UpdateDecisionError> {
        let policies = canonical_policy_snapshots(policies, plan)
            .map_err(|_| UpdateDecisionError::CatalogAuthorityChanged)?;
        let refs = grant_events
            .iter()
            .map(|e| GrantEventReference::from_event(e, GrantEventKind::MarkedStale))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self {
            kind: UpdateDecisionContextKind::Apply {
                installation: installation.clone(),
                authority: AuthorityCarrierBinding::from_complete_parts(
                    target_catalog,
                    target_publications,
                    rollback_catalog,
                    rollback_publications,
                    target_registry,
                    rollback_registry,
                    &policies,
                ),
                current_grants,
                installation_event: InstallationEventReference::from_event(
                    installation.installation_id(),
                    installation_event,
                    InstallationEventKind::PackageUpdated,
                )?,
                grant_events: refs,
            },
        })
    }
    #[allow(clippy::too_many_arguments)]
    #[cfg(test)]
    fn for_apply_for_test(
        installation: &InstallationSnapshot,
        plan: &PackageUpdatePlan,
        target_catalog: &CatalogReadModel,
        rollback_catalog: &CatalogReadModel,
        target_registry: &CapabilityRegistry,
        rollback_registry: &CapabilityRegistry,
        policies: &[InvocationPolicySnapshot],
        current_grants: CurrentInstallationGrantSet,
        installation_event: &InstallationEvent,
        grant_events: &[GrantEvent],
    ) -> Result<Self, UpdateDecisionError> {
        let target_publications = synthetic_publications_for_plan_authority(&plan.target);
        let rollback_publications = synthetic_publications_for_plan_authority(&plan.rollback);
        Self::for_apply(
            installation,
            plan,
            target_catalog,
            &target_publications,
            rollback_catalog,
            &rollback_publications,
            target_registry,
            rollback_registry,
            policies,
            current_grants,
            installation_event,
            grant_events,
        )
    }
    pub(in crate::market) fn for_confirm_applied(installation: &InstallationSnapshot) -> Self {
        Self {
            kind: UpdateDecisionContextKind::ConfirmApplied {
                installation: installation.clone(),
            },
        }
    }
    #[allow(clippy::too_many_arguments)]
    pub(in crate::market) fn for_rollback(
        installation: &InstallationSnapshot,
        plan: &PackageUpdatePlan,
        target_catalog: &CatalogReadModel,
        target_publications: &[CatalogPackageRevision],
        rollback_catalog: &CatalogReadModel,
        rollback_publications: &[CatalogPackageRevision],
        target_registry: &CapabilityRegistry,
        rollback_registry: &CapabilityRegistry,
        policies: &[InvocationPolicySnapshot],
        current_grants: CurrentInstallationGrantSet,
        installation_event: &InstallationEvent,
        grant_events: &[GrantEvent],
    ) -> Result<Self, UpdateDecisionError> {
        let policies = canonical_policy_snapshots(policies, plan)
            .map_err(|_| UpdateDecisionError::CatalogAuthorityChanged)?;
        let refs = grant_events
            .iter()
            .map(|e| GrantEventReference::from_event(e, GrantEventKind::MarkedStale))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self {
            kind: UpdateDecisionContextKind::Rollback {
                installation: installation.clone(),
                authority: AuthorityCarrierBinding::from_complete_parts(
                    target_catalog,
                    target_publications,
                    rollback_catalog,
                    rollback_publications,
                    target_registry,
                    rollback_registry,
                    &policies,
                ),
                current_grants,
                installation_event: InstallationEventReference::from_event(
                    installation.installation_id(),
                    installation_event,
                    InstallationEventKind::PackageRolledBack,
                )?,
                grant_events: refs,
            },
        })
    }
    #[allow(clippy::too_many_arguments)]
    #[cfg(test)]
    fn for_rollback_for_test(
        installation: &InstallationSnapshot,
        plan: &PackageUpdatePlan,
        target_catalog: &CatalogReadModel,
        rollback_catalog: &CatalogReadModel,
        target_registry: &CapabilityRegistry,
        rollback_registry: &CapabilityRegistry,
        policies: &[InvocationPolicySnapshot],
        current_grants: CurrentInstallationGrantSet,
        installation_event: &InstallationEvent,
        grant_events: &[GrantEvent],
    ) -> Result<Self, UpdateDecisionError> {
        let target_publications = synthetic_publications_for_plan_authority(&plan.target);
        let rollback_publications = synthetic_publications_for_plan_authority(&plan.rollback);
        Self::for_rollback(
            installation,
            plan,
            target_catalog,
            &target_publications,
            rollback_catalog,
            &rollback_publications,
            target_registry,
            rollback_registry,
            policies,
            current_grants,
            installation_event,
            grant_events,
        )
    }
    pub(in crate::market) fn for_cancel() -> Self {
        Self {
            kind: UpdateDecisionContextKind::Cancel,
        }
    }
    pub(in crate::market) fn for_cancel_after_terminal_installation(
        installation: &InstallationSnapshot,
    ) -> Self {
        Self {
            kind: UpdateDecisionContextKind::CancelAfterTerminalInstallation {
                installation: installation.clone(),
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpdateEventKind {
    Staged,
    ApprovalRecorded,
    Applied,
    Confirmed,
    RolledBack,
    Cancelled,
}

#[allow(clippy::large_enum_variant)]
#[derive(Clone, PartialEq, Eq)]
enum UpdateEventPayload {
    Staged {
        plan: PackageUpdatePlan,
    },
    ApprovalRecorded {
        approval: UpdateApprovalEvidence,
        readiness: UpdateReadinessEvidence,
    },
    Applied {
        prior_installation_revision: InstallationRevision,
        applied_installation_revision: InstallationRevision,
        target_pin_digest: Sha256Digest,
        installation_event: InstallationEventReference,
        grant_events: Vec<GrantEventReference>,
        grant_set_digest: Sha256Digest,
    },
    Confirmed {
        evidence: UpdateConfirmationEvidence,
    },
    RolledBack {
        prior_installation_revision: InstallationRevision,
        rolled_back_installation_revision: InstallationRevision,
        rollback_pin_digest: Sha256Digest,
        evidence: RollbackReadinessEvidence,
        installation_event: InstallationEventReference,
        grant_events: Vec<GrantEventReference>,
        grant_set_digest: Sha256Digest,
    },
    Cancelled {
        terminal_installation_revision: Option<InstallationRevision>,
    },
}
impl UpdateEventPayload {
    const fn kind(&self) -> UpdateEventKind {
        match self {
            Self::Staged { .. } => UpdateEventKind::Staged,
            Self::ApprovalRecorded { .. } => UpdateEventKind::ApprovalRecorded,
            Self::Applied { .. } => UpdateEventKind::Applied,
            Self::Confirmed { .. } => UpdateEventKind::Confirmed,
            Self::RolledBack { .. } => UpdateEventKind::RolledBack,
            Self::Cancelled { .. } => UpdateEventKind::Cancelled,
        }
    }
}
impl fmt::Debug for UpdateEventPayload {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("UpdateEventPayload")
            .field("kind", &self.kind())
            .field("authority", &"<redacted>")
            .finish()
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct UpdateEvent {
    sequence: UpdateEventSequence,
    post_revision: UpdateRevision,
    command_id: UpdateCommandId,
    update_id: PackageUpdateId,
    payload: UpdateEventPayload,
    event_digest: Sha256Digest,
}
impl UpdateEvent {
    #[must_use]
    pub const fn sequence(&self) -> UpdateEventSequence {
        self.sequence
    }
    #[must_use]
    pub const fn post_revision(&self) -> &UpdateRevision {
        &self.post_revision
    }
    #[must_use]
    pub const fn command_id(&self) -> &UpdateCommandId {
        &self.command_id
    }
    #[must_use]
    pub const fn update_id(&self) -> &PackageUpdateId {
        &self.update_id
    }
    #[must_use]
    pub const fn kind(&self) -> UpdateEventKind {
        self.payload.kind()
    }
    #[must_use]
    pub const fn event_digest(&self) -> &Sha256Digest {
        &self.event_digest
    }
}
impl fmt::Debug for UpdateEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("UpdateEvent")
            .field("sequence", &self.sequence)
            .field("post_revision", &"<redacted>")
            .field("command_id", &"<redacted>")
            .field("update_id", &"<redacted>")
            .field("kind", &self.kind())
            .field("authority", &"<redacted>")
            .finish()
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct PackageUpdateAggregate {
    update_id: PackageUpdateId,
    installation_id: InstallationId,
    tenant_id: TenantId,
    user_id: UserId,
    plan: PackageUpdatePlan,
    state: UpdateState,
    revision: UpdateRevision,
    last_sequence: UpdateEventSequence,
    approval: Option<UpdateApprovalEvidence>,
    readiness: Option<UpdateReadinessEvidence>,
    applied_installation_revision: Option<InstallationRevision>,
    applied_event_digest: Option<Sha256Digest>,
    confirmation: Option<UpdateConfirmationEvidence>,
    rollback: Option<RollbackReadinessEvidence>,
}
pub type PackageUpdateSnapshot = PackageUpdateAggregate;
impl PackageUpdateAggregate {
    #[must_use]
    pub const fn update_id(&self) -> &PackageUpdateId {
        &self.update_id
    }
    #[must_use]
    pub const fn installation_id(&self) -> &InstallationId {
        &self.installation_id
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
    pub const fn plan(&self) -> &PackageUpdatePlan {
        &self.plan
    }
    #[must_use]
    pub const fn state(&self) -> UpdateState {
        self.state
    }
    #[must_use]
    pub const fn revision(&self) -> &UpdateRevision {
        &self.revision
    }
    #[must_use]
    pub const fn last_sequence(&self) -> UpdateEventSequence {
        self.last_sequence
    }
    #[must_use]
    pub const fn approval(&self) -> Option<&UpdateApprovalEvidence> {
        self.approval.as_ref()
    }
    #[must_use]
    pub const fn readiness(&self) -> Option<&UpdateReadinessEvidence> {
        self.readiness.as_ref()
    }
    #[must_use]
    pub const fn applied_installation_revision(&self) -> Option<&InstallationRevision> {
        self.applied_installation_revision.as_ref()
    }
    #[must_use]
    pub const fn confirmation(&self) -> Option<&UpdateConfirmationEvidence> {
        self.confirmation.as_ref()
    }
    #[must_use]
    pub const fn rollback_evidence(&self) -> Option<&RollbackReadinessEvidence> {
        self.rollback.as_ref()
    }
}
impl fmt::Debug for PackageUpdateAggregate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("PackageUpdateAggregate(<authority-redacted>)")
    }
}

pub fn decide(
    current: Option<&PackageUpdateAggregate>,
    context: &UpdateDecisionContext,
    command: &UpdateCommand,
) -> Result<UpdateEvent, UpdateDecisionError> {
    match (&command.action, &context.kind) {
        (
            UpdateCommandAction::Stage { plan },
            UpdateDecisionContextKind::Stage {
                installation,
                plan: ctx_plan,
                authority,
            },
        ) => {
            if current.is_some() {
                return Err(UpdateDecisionError::AggregateAlreadyPresent);
            }
            if plan != ctx_plan {
                return Err(UpdateDecisionError::PlanMismatch);
            }
            if !authority.matches_plan(plan) {
                return Err(UpdateDecisionError::CatalogAuthorityChanged);
            }
            if installation.state() == ManagedInstallationState::Revoked
                || installation.state() == ManagedInstallationState::Uninstalled
            {
                return Err(UpdateDecisionError::InstallationTerminal);
            }
            make_update_event(
                UpdateEventSequence::new(1).map_err(|_| UpdateDecisionError::SequenceOverflow)?,
                command,
                UpdateEventPayload::Staged { plan: plan.clone() },
            )
        }
        (
            UpdateCommandAction::RecordApproval {
                expected_update_revision,
                approval,
                readiness,
            },
            UpdateDecisionContextKind::RecordApproval {
                installation,
                authority,
            },
        ) => {
            let agg = require_update_current(current, command, expected_update_revision)?;
            if agg.state != UpdateState::Staged {
                return Err(UpdateDecisionError::IllegalTransition);
            }
            require_current_installation(
                installation,
                agg,
                agg.plan.staged_installation_revision(),
                agg.plan.rollback_pin(),
                true,
            )?;
            authority.matches_readiness(&agg.plan, readiness)?;
            if approval.plan_digest() != agg.plan.plan_digest()
                || approval.change_class() != agg.plan.change_class()
                || approval.staged_installation_revision()
                    != agg.plan.staged_installation_revision()
                || approval.staged_configuration_digest() != agg.plan.staged_configuration_digest()
                || readiness.plan_digest() != agg.plan.plan_digest()
                || readiness.staged_installation_revision()
                    != agg.plan.staged_installation_revision()
                || readiness.staged_configuration_digest() != agg.plan.staged_configuration_digest()
            {
                return Err(UpdateDecisionError::ApprovalMissingOrMismatch);
            }
            make_update_event(
                agg.last_sequence
                    .next()
                    .map_err(|_| UpdateDecisionError::SequenceOverflow)?,
                command,
                UpdateEventPayload::ApprovalRecorded {
                    approval: approval.clone(),
                    readiness: readiness.clone(),
                },
            )
        }
        (
            UpdateCommandAction::Apply {
                expected_installation_revision,
                ..
            },
            UpdateDecisionContextKind::Apply {
                installation,
                authority,
                current_grants,
                installation_event,
                grant_events,
            },
        ) => {
            let agg = validate_apply_admission(current, command, installation, authority)?;
            validate_grants(current_grants, installation, grant_events)?;
            if installation_event.kind != InstallationEventKind::PackageUpdated
                || installation_event.installation_id != *agg.plan.installation_id()
            {
                return Err(UpdateDecisionError::CoupledInstallationEventMismatch);
            }
            make_update_event(
                agg.last_sequence
                    .next()
                    .map_err(|_| UpdateDecisionError::SequenceOverflow)?,
                command,
                UpdateEventPayload::Applied {
                    prior_installation_revision: expected_installation_revision.clone(),
                    applied_installation_revision: installation_event.post_revision.clone(),
                    target_pin_digest: digest_pin_value(agg.plan.target_pin()),
                    installation_event: installation_event.clone(),
                    grant_events: grant_events.clone(),
                    grant_set_digest: current_grants.grant_set_digest().clone(),
                },
            )
        }
        (
            UpdateCommandAction::ConfirmAppliedUpdate {
                expected_update_revision,
                expected_installation_revision,
                evidence,
            },
            UpdateDecisionContextKind::ConfirmApplied { installation },
        ) => {
            let agg = require_update_current(current, command, expected_update_revision)?;
            if agg.state != UpdateState::AppliedPendingConfirmation {
                return Err(UpdateDecisionError::IllegalTransition);
            }
            require_current_installation(
                installation,
                agg,
                expected_installation_revision,
                agg.plan.target_pin(),
                false,
            )?;
            if evidence.update_id() != agg.update_id()
                || evidence.expected_update_revision() != expected_update_revision
                || evidence.applied_event_digest()
                    != agg
                        .applied_event_digest
                        .as_ref()
                        .ok_or(UpdateDecisionError::ConfirmationEvidenceMismatch)?
                || evidence.installation_id() != agg.installation_id()
                || evidence.installation_revision() != expected_installation_revision
                || evidence.target_pin_digest() != &digest_pin_value(agg.plan.target_pin())
                || evidence.installation_state_digest()
                    != &digest_installation_state_binding(installation)
            {
                return Err(UpdateDecisionError::ConfirmationEvidenceMismatch);
            }
            make_update_event(
                agg.last_sequence
                    .next()
                    .map_err(|_| UpdateDecisionError::SequenceOverflow)?,
                command,
                UpdateEventPayload::Confirmed {
                    evidence: evidence.clone(),
                },
            )
        }
        (
            UpdateCommandAction::Rollback {
                expected_installation_revision,
                evidence,
                ..
            },
            UpdateDecisionContextKind::Rollback {
                installation,
                authority,
                current_grants,
                installation_event,
                grant_events,
            },
        ) => {
            let agg = validate_rollback_admission(current, command, installation, authority)?;
            validate_grants(current_grants, installation, grant_events)?;
            if installation_event.kind != InstallationEventKind::PackageRolledBack
                || installation_event.installation_id != *agg.plan.installation_id()
            {
                return Err(UpdateDecisionError::CoupledInstallationEventMismatch);
            }
            make_update_event(
                agg.last_sequence
                    .next()
                    .map_err(|_| UpdateDecisionError::SequenceOverflow)?,
                command,
                UpdateEventPayload::RolledBack {
                    prior_installation_revision: expected_installation_revision.clone(),
                    rolled_back_installation_revision: installation_event.post_revision.clone(),
                    rollback_pin_digest: digest_pin_value(agg.plan.rollback_pin()),
                    evidence: evidence.clone(),
                    installation_event: installation_event.clone(),
                    grant_events: grant_events.clone(),
                    grant_set_digest: current_grants.grant_set_digest().clone(),
                },
            )
        }
        (
            UpdateCommandAction::Cancel {
                expected_update_revision,
            },
            UpdateDecisionContextKind::Cancel,
        ) => {
            let agg = require_update_current(current, command, expected_update_revision)?;
            if !matches!(agg.state, UpdateState::Staged | UpdateState::Ready) {
                return Err(UpdateDecisionError::IllegalTransition);
            }
            make_update_event(
                agg.last_sequence
                    .next()
                    .map_err(|_| UpdateDecisionError::SequenceOverflow)?,
                command,
                UpdateEventPayload::Cancelled {
                    terminal_installation_revision: None,
                },
            )
        }
        (
            UpdateCommandAction::CancelAfterTerminalInstallation {
                expected_update_revision,
                expected_terminal_installation_revision,
            },
            UpdateDecisionContextKind::CancelAfterTerminalInstallation { installation },
        ) => {
            let agg = require_update_current(current, command, expected_update_revision)?;
            if agg.state != UpdateState::AppliedPendingConfirmation {
                return Err(UpdateDecisionError::IllegalTransition);
            }
            if installation.installation_id() != agg.installation_id()
                || installation.revision() != expected_terminal_installation_revision
            {
                return Err(UpdateDecisionError::InstallationRevisionMismatch);
            }
            if !matches!(
                installation.state(),
                ManagedInstallationState::Revoked | ManagedInstallationState::Uninstalled
            ) {
                return Err(UpdateDecisionError::InstallationTerminal);
            }
            make_update_event(
                agg.last_sequence
                    .next()
                    .map_err(|_| UpdateDecisionError::SequenceOverflow)?,
                command,
                UpdateEventPayload::Cancelled {
                    terminal_installation_revision: Some(
                        expected_terminal_installation_revision.clone(),
                    ),
                },
            )
        }
        _ => Err(UpdateDecisionError::IllegalTransition),
    }
}

pub fn evolve(
    current: Option<PackageUpdateAggregate>,
    event: &UpdateEvent,
) -> Result<PackageUpdateAggregate, UpdateReplayError> {
    verify_update_event_digest(event)?;
    match (&current, &event.payload) {
        (None, UpdateEventPayload::Staged { plan }) => {
            if event.sequence.get() != 1 {
                return Err(UpdateReplayError::SequenceMismatch);
            }
            require_event_revision(event.sequence, event.post_revision())?;
            if event.update_id() != plan.update_id() || digest_plan(plan) != *plan.plan_digest() {
                return Err(UpdateReplayError::PlanMismatch);
            }
            Ok(PackageUpdateAggregate {
                update_id: event.update_id.clone(),
                installation_id: plan.installation_id().clone(),
                tenant_id: plan.tenant_id().clone(),
                user_id: plan.user_id().clone(),
                plan: plan.clone(),
                state: UpdateState::Staged,
                revision: event.post_revision.clone(),
                last_sequence: event.sequence,
                approval: None,
                readiness: None,
                applied_installation_revision: None,
                applied_event_digest: None,
                confirmation: None,
                rollback: None,
            })
        }
        (Some(agg), payload) => {
            if agg.update_id != event.update_id {
                return Err(UpdateReplayError::IdentityMismatch);
            }
            if agg.state.is_terminal() {
                return Err(UpdateReplayError::PostTerminalEvent);
            }
            if event.sequence
                != agg
                    .last_sequence
                    .next()
                    .map_err(|_| UpdateReplayError::SequenceOverflow)?
            {
                return Err(UpdateReplayError::SequenceMismatch);
            }
            require_event_revision(event.sequence, event.post_revision())?;
            let mut next = agg.clone();
            next.revision = event.post_revision.clone();
            next.last_sequence = event.sequence;
            match payload {
                UpdateEventPayload::ApprovalRecorded {
                    approval,
                    readiness,
                } if agg.state == UpdateState::Staged => {
                    if agg.approval.is_some() || agg.readiness.is_some() {
                        return Err(UpdateReplayError::IllegalTransition);
                    }
                    verify_update_approval_evidence(approval)
                        .map_err(|_| UpdateReplayError::EvidenceMismatch)?;
                    verify_update_readiness_evidence(readiness)
                        .map_err(|_| UpdateReplayError::EvidenceMismatch)?;
                    if approval.plan_digest() != agg.plan.plan_digest()
                        || approval.change_class() != agg.plan.change_class()
                        || approval.staged_installation_revision()
                            != agg.plan.staged_installation_revision()
                        || approval.staged_configuration_digest()
                            != agg.plan.staged_configuration_digest()
                        || readiness.plan_digest() != agg.plan.plan_digest()
                        || readiness.target_package_digest()
                            != agg.plan.target_pin().package_digest()
                        || readiness.rollback_package_digest()
                            != agg.plan.rollback_pin().package_digest()
                        || readiness.target_component_authority_digest()
                            != agg.plan.target_component_authority_digest()
                        || readiness.rollback_component_authority_digest()
                            != agg.plan.rollback_component_authority_digest()
                        || readiness.staged_installation_revision()
                            != agg.plan.staged_installation_revision()
                        || readiness.staged_configuration_digest()
                            != agg.plan.staged_configuration_digest()
                    {
                        return Err(UpdateReplayError::EvidenceMismatch);
                    }
                    next.state = UpdateState::Ready;
                    next.approval = Some(approval.clone());
                    next.readiness = Some(readiness.clone());
                }
                UpdateEventPayload::Applied {
                    prior_installation_revision,
                    applied_installation_revision,
                    target_pin_digest,
                    installation_event,
                    grant_events,
                    grant_set_digest,
                } if agg.state == UpdateState::Ready => {
                    if prior_installation_revision == applied_installation_revision
                        || target_pin_digest != &digest_pin_value(agg.plan.target_pin())
                        || installation_event.kind != InstallationEventKind::PackageUpdated
                        || installation_event.installation_id != *agg.installation_id()
                        || installation_event.post_revision != *applied_installation_revision
                        || grant_set_digest == &Sha256Digest::from_bytes(&[])
                    {
                        return Err(UpdateReplayError::SubordinateReferenceMismatch);
                    }
                    validate_active_grant_refs(&agg.plan, grant_events)
                        .map_err(|_| UpdateReplayError::SubordinateReferenceMismatch)?;
                    next.state = UpdateState::AppliedPendingConfirmation;
                    next.applied_installation_revision =
                        Some(applied_installation_revision.clone());
                    next.applied_event_digest = Some(event.event_digest().clone());
                }
                UpdateEventPayload::Confirmed { evidence }
                    if agg.state == UpdateState::AppliedPendingConfirmation =>
                {
                    verify_update_confirmation_evidence(evidence)
                        .map_err(|_| UpdateReplayError::EvidenceMismatch)?;
                    if evidence.update_id() != agg.update_id()
                        || evidence.expected_update_revision() != agg.revision()
                        || Some(evidence.applied_event_digest())
                            != agg.applied_event_digest.as_ref()
                        || evidence.installation_id() != agg.installation_id()
                        || Some(evidence.installation_revision())
                            != agg.applied_installation_revision.as_ref()
                        || evidence.target_pin_digest() != &digest_pin_value(agg.plan.target_pin())
                    {
                        return Err(UpdateReplayError::EvidenceMismatch);
                    }
                    next.state = UpdateState::Confirmed;
                    next.confirmation = Some(evidence.clone());
                }
                UpdateEventPayload::RolledBack {
                    prior_installation_revision,
                    rolled_back_installation_revision,
                    rollback_pin_digest,
                    evidence,
                    installation_event,
                    grant_events,
                    grant_set_digest,
                } if agg.state == UpdateState::AppliedPendingConfirmation => {
                    verify_rollback_readiness_evidence(evidence)
                        .map_err(|_| UpdateReplayError::EvidenceMismatch)?;
                    if evidence.update_id() != agg.update_id()
                        || evidence.expected_update_revision() != agg.revision()
                        || rollback_pin_digest != &digest_pin_value(agg.plan.rollback_pin())
                        || evidence.rollback_pin_digest() != rollback_pin_digest
                        || Some(prior_installation_revision)
                            != agg.applied_installation_revision.as_ref()
                        || evidence.current_target_installation_revision()
                            != prior_installation_revision
                        || installation_event.kind != InstallationEventKind::PackageRolledBack
                        || installation_event.installation_id != *agg.installation_id()
                        || installation_event.post_revision != *rolled_back_installation_revision
                        || grant_set_digest == &Sha256Digest::from_bytes(&[])
                    {
                        return Err(UpdateReplayError::SubordinateReferenceMismatch);
                    }
                    validate_active_grant_refs(&agg.plan, grant_events)
                        .map_err(|_| UpdateReplayError::SubordinateReferenceMismatch)?;
                    next.state = UpdateState::RolledBack;
                    next.rollback = Some(evidence.clone());
                }
                UpdateEventPayload::Cancelled {
                    terminal_installation_revision,
                } if matches!(
                    agg.state,
                    UpdateState::Staged
                        | UpdateState::Ready
                        | UpdateState::AppliedPendingConfirmation
                ) =>
                {
                    if agg.state == UpdateState::AppliedPendingConfirmation
                        && terminal_installation_revision.is_none()
                    {
                        return Err(UpdateReplayError::IllegalTransition);
                    }
                    next.state = UpdateState::Cancelled;
                }
                _ => return Err(UpdateReplayError::IllegalTransition),
            }
            Ok(next)
        }
        (None, _) => Err(UpdateReplayError::NonStagedInitialEvent),
    }
}

pub fn replay<'a>(
    events: impl IntoIterator<Item = &'a UpdateEvent>,
) -> Result<Option<PackageUpdateAggregate>, UpdateReplayError> {
    let mut current = None;
    let mut seen_commands = BTreeSet::new();
    let mut seen_approvals = BTreeSet::new();
    for event in events {
        if !seen_commands.insert(event.command_id.clone()) {
            return Err(UpdateReplayError::DuplicateCommandId);
        }
        if matches!(
            &event.payload,
            UpdateEventPayload::ApprovalRecorded { approval, .. }
                if !seen_approvals.insert(approval.approval_id().clone())
        ) {
            return Err(UpdateReplayError::DuplicateApprovalId);
        }
        current = Some(evolve(current, event)?);
    }
    Ok(current)
}

#[allow(clippy::large_enum_variant)]
#[derive(Clone, PartialEq, Eq)]
pub enum UpdateCommandOutcome {
    Accepted {
        event: UpdateEvent,
        snapshot: PackageUpdateSnapshot,
    },
    Rejected {
        error: UpdateDecisionError,
    },
}

impl fmt::Debug for UpdateCommandOutcome {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("UpdateCommandOutcome(<authority-redacted>)")
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct UpdateCommandReceipt {
    command: UpdateCommand,
    outcome: UpdateCommandOutcome,
    witness: UpdateReceiptWitness,
    subordinate_installation_receipt: Option<InstallationCommandReceipt>,
    subordinate_grant_receipts: Vec<GrantCommandReceipt>,
}

#[derive(Clone, PartialEq, Eq)]
enum UpdateReceiptWitness {
    Decision(Box<UpdateDecisionContext>),
    MissingAggregate,
    RepositoryPreflight {
        snapshot: Box<PackageUpdateSnapshot>,
    },
    CoupledDecisionPreflight {
        snapshot: Box<PackageUpdateSnapshot>,
        installation: Box<InstallationSnapshot>,
        authority: Box<AuthorityCarrierBinding>,
    },
    ApprovalAlreadyConsumed {
        prior_update_id: PackageUpdateId,
        prior_evidence_digest: Sha256Digest,
    },
    ActiveSlotConflict {
        installation_id: InstallationId,
        conflicting_update_id: PackageUpdateId,
        conflicting_state: UpdateState,
    },
}

impl UpdateCommandReceipt {
    #[must_use]
    pub const fn command(&self) -> &UpdateCommand {
        &self.command
    }

    #[must_use]
    pub const fn outcome(&self) -> &UpdateCommandOutcome {
        &self.outcome
    }
}

impl fmt::Debug for UpdateCommandReceipt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("UpdateCommandReceipt(<authority-redacted>)")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UpdateRepositoryError {
    CommandConflict,
    TransactionConflict,
    InjectedPersistenceFailure,
    CorruptUpdateHistory(UpdateReplayError),
    CorruptInstallationHistory(InstallationReplayError),
    CorruptGrantHistory(GrantReplayError),
    CorruptCurrentUpdateIndex,
    CorruptGrantIndex,
    CorruptGrantSet,
    DecisionRejected(UpdateDecisionError),
}

impl fmt::Display for UpdateRepositoryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "package update repository rejected operation: {self:?}"
        )
    }
}
impl Error for UpdateRepositoryError {}

pub trait PackageUpdateRepository {
    fn execute(
        &mut self,
        command: UpdateCommand,
    ) -> Result<UpdateCommandReceipt, UpdateRepositoryError>;

    fn load_exact(
        &self,
        id: &PackageUpdateId,
    ) -> Result<Option<PackageUpdateSnapshot>, UpdateRepositoryError>;

    fn event_history(
        &self,
        id: &PackageUpdateId,
    ) -> Result<Vec<UpdateEvent>, UpdateRepositoryError>;
}

#[derive(Clone, PartialEq, Eq)]
struct UpdateLedgerEntry {
    command: UpdateCommand,
    receipt: UpdateCommandReceipt,
}

fn select_unique_catalog_by_revision<'a>(
    models: &'a [CatalogReadModel],
    revision: &CatalogRevision,
) -> Result<&'a CatalogReadModel, UpdateRepositoryError> {
    let mut matches = models
        .iter()
        .filter(|model| model.catalog_revision() == revision);
    let Some(first) = matches.next() else {
        return Err(UpdateRepositoryError::TransactionConflict);
    };
    if matches.next().is_some() {
        return Err(UpdateRepositoryError::TransactionConflict);
    }
    Ok(first)
}

fn select_unique_registry_by_revision<'a>(
    registries: &'a [CapabilityRegistry],
    revision: &CapabilityRegistryRevision,
) -> Result<&'a CapabilityRegistry, UpdateRepositoryError> {
    let mut matches = registries
        .iter()
        .filter(|registry| registry.registry_revision() == revision);
    let Some(first) = matches.next() else {
        return Err(UpdateRepositoryError::TransactionConflict);
    };
    if matches.next().is_some() {
        return Err(UpdateRepositoryError::TransactionConflict);
    }
    Ok(first)
}

#[derive(Clone)]
pub struct InMemoryPackageUpdateRepository {
    aggregates: BTreeMap<PackageUpdateId, PackageUpdateAggregate>,
    events: BTreeMap<PackageUpdateId, Vec<UpdateEvent>>,
    command_ledger: BTreeMap<UpdateCommandId, UpdateLedgerEntry>,
    consumed_approvals: BTreeMap<UpdateApprovalId, (PackageUpdateId, Sha256Digest)>,
    current_update_slots: BTreeMap<InstallationId, PackageUpdateId>,
    catalog_read_models: Vec<CatalogReadModel>,
    catalog_publications: Vec<CatalogPackageRevision>,
    capability_registries: Vec<CapabilityRegistry>,
    policy_snapshots: Vec<InvocationPolicySnapshot>,
    installation_repository: InMemoryInstallationRepository,
    grant_repository: InMemoryGrantRepository,
    fail_next_commit: bool,
}

impl fmt::Debug for InMemoryPackageUpdateRepository {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("InMemoryPackageUpdateRepository(<authority-redacted>)")
    }
}

impl InMemoryPackageUpdateRepository {
    #[must_use]
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            aggregates: BTreeMap::new(),
            events: BTreeMap::new(),
            command_ledger: BTreeMap::new(),
            consumed_approvals: BTreeMap::new(),
            current_update_slots: BTreeMap::new(),
            catalog_read_models: Vec::new(),
            catalog_publications: Vec::new(),
            capability_registries: Vec::new(),
            policy_snapshots: Vec::new(),
            installation_repository: InMemoryInstallationRepository::new(),
            grant_repository: InMemoryGrantRepository::new(),
            fail_next_commit: false,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn try_from_authority_histories(
        catalog_read_models: Vec<CatalogReadModel>,
        catalog_publications: Vec<CatalogPackageRevision>,
        capability_registries: Vec<CapabilityRegistry>,
        policy_snapshots: Vec<InvocationPolicySnapshot>,
        installation_histories: Vec<(InstallationId, Vec<InstallationEvent>)>,
        installation_ledger_receipts: Vec<(
            InstallationCommandReceipt,
            Option<InstallationSnapshot>,
        )>,
        grant_histories: Vec<(GrantSnapshotId, Vec<GrantEvent>)>,
        grant_ledger_receipts: Vec<(GrantCommandReceipt, Option<GrantSnapshot>)>,
        update_histories: Vec<(PackageUpdateId, Vec<UpdateEvent>)>,
        update_ledger_receipts: Vec<(UpdateCommandReceipt, Option<PackageUpdateSnapshot>)>,
    ) -> Result<Self, UpdateRepositoryError> {
        let owner_installation_couplings =
            collect_owner_installation_couplings(&installation_histories)?;
        let owner_grant_couplings = collect_owner_grant_couplings(&grant_histories)?;
        let installation_repository =
            InMemoryInstallationRepository::try_from_histories_and_receipts(
                installation_histories,
                installation_ledger_receipts,
            )
            .map_err(map_installation_repository_error)?;
        let grant_repository = InMemoryGrantRepository::try_from_histories_and_receipts(
            grant_histories,
            grant_ledger_receipts,
        )
        .map_err(map_grant_repository_error)?;

        let mut repository = Self {
            catalog_read_models,
            catalog_publications,
            capability_registries,
            policy_snapshots,
            installation_repository,
            grant_repository,
            ..Self::new()
        };
        let mut reachable_prefixes: BTreeMap<PackageUpdateId, Vec<Option<PackageUpdateSnapshot>>> =
            BTreeMap::new();
        let mut accepted_events_by_command: BTreeMap<
            UpdateCommandId,
            (
                PackageUpdateId,
                UpdateEvent,
                Option<PackageUpdateSnapshot>,
                PackageUpdateSnapshot,
            ),
        > = BTreeMap::new();
        let mut referenced_installation_couplings = BTreeSet::new();
        let mut referenced_grant_couplings = BTreeSet::new();

        for (update_id, events) in update_histories {
            if repository.events.contains_key(&update_id) {
                return Err(UpdateRepositoryError::CorruptCurrentUpdateIndex);
            }
            let mut current = None;
            let mut prefixes = Vec::with_capacity(events.len().saturating_add(1));
            let mut stored_events = Vec::with_capacity(events.len());
            prefixes.push(None);
            for event in events {
                if event.update_id() != &update_id {
                    return Err(UpdateRepositoryError::CorruptUpdateHistory(
                        UpdateReplayError::IdentityMismatch,
                    ));
                }
                let pre_snapshot = current.clone();
                let snapshot =
                    evolve(current, &event).map_err(UpdateRepositoryError::CorruptUpdateHistory)?;
                repository.verify_subordinate_references(&event, pre_snapshot.as_ref())?;
                collect_update_coupling_references(
                    &event,
                    &mut referenced_installation_couplings,
                    &mut referenced_grant_couplings,
                )?;
                if accepted_events_by_command
                    .insert(
                        event.command_id().clone(),
                        (
                            update_id.clone(),
                            event.clone(),
                            pre_snapshot,
                            snapshot.clone(),
                        ),
                    )
                    .is_some()
                {
                    return Err(UpdateRepositoryError::CommandConflict);
                }
                if let UpdateEventPayload::ApprovalRecorded { approval, .. } = &event.payload {
                    repository.note_consumed_approval(update_id.clone(), approval)?;
                }
                repository.apply_current_update_slot_event(&event, &snapshot)?;
                prefixes.push(Some(snapshot.clone()));
                current = Some(snapshot);
                stored_events.push(event);
            }
            if let Some(snapshot) = current {
                repository.aggregates.insert(update_id.clone(), snapshot);
            }
            repository.events.insert(update_id.clone(), stored_events);
            reachable_prefixes.insert(update_id, prefixes);
        }

        let mut ledger_current: BTreeMap<PackageUpdateId, Option<PackageUpdateSnapshot>> =
            BTreeMap::new();
        let mut ledger_current_slots: BTreeMap<InstallationId, PackageUpdateId> = BTreeMap::new();
        let mut ledger_consumed_approvals: BTreeMap<
            UpdateApprovalId,
            (PackageUpdateId, Sha256Digest),
        > = BTreeMap::new();
        for (receipt, observed_pre_snapshot) in update_ledger_receipts {
            repository.verify_receipt_owner_context_bindings(&receipt)?;
            validate_update_receipt_against_histories(
                &receipt,
                &observed_pre_snapshot,
                &mut ledger_current,
                &mut ledger_current_slots,
                &mut ledger_consumed_approvals,
                &mut accepted_events_by_command,
            )?;
            let command = receipt.command.clone();
            if repository
                .command_ledger
                .insert(
                    command.command_id().clone(),
                    UpdateLedgerEntry {
                        command,
                        receipt: receipt.clone(),
                    },
                )
                .is_some()
            {
                return Err(UpdateRepositoryError::CommandConflict);
            }
        }
        let ledger_aggregates: BTreeMap<_, _> = ledger_current
            .into_iter()
            .filter_map(|(id, snapshot)| snapshot.map(|snapshot| (id, snapshot)))
            .collect();
        if !accepted_events_by_command.is_empty()
            || ledger_aggregates != repository.aggregates
            || ledger_current_slots != repository.current_update_slots
            || ledger_consumed_approvals != repository.consumed_approvals
            || referenced_installation_couplings != owner_installation_couplings
            || referenced_grant_couplings != owner_grant_couplings
        {
            return Err(UpdateRepositoryError::CorruptCurrentUpdateIndex);
        }
        Ok(repository)
    }

    pub fn fail_next_commit_for_test(&mut self) {
        self.fail_next_commit = true;
    }

    fn note_consumed_approval(
        &mut self,
        update_id: PackageUpdateId,
        approval: &UpdateApprovalEvidence,
    ) -> Result<(), UpdateRepositoryError> {
        match self.consumed_approvals.get(approval.approval_id()) {
            None => {
                self.consumed_approvals.insert(
                    approval.approval_id().clone(),
                    (update_id, approval.evidence_digest().clone()),
                );
                Ok(())
            }
            Some((existing_update, existing_digest))
                if *existing_update == update_id
                    && existing_digest == approval.evidence_digest() =>
            {
                Ok(())
            }
            Some(_) => Err(UpdateRepositoryError::CorruptCurrentUpdateIndex),
        }
    }

    fn publications_for_pin(&self, pin: &InstallationPackagePin) -> Vec<CatalogPackageRevision> {
        self.catalog_publications
            .iter()
            .filter(|publication| {
                &publication.catalog_revision == pin.catalog_revision()
                    && &publication.package_id == pin.package_id()
                    && &publication.package_version == pin.package_version()
            })
            .cloned()
            .collect()
    }

    fn policies_for_plan(
        &self,
        plan: &PackageUpdatePlan,
    ) -> Result<Vec<InvocationPolicySnapshot>, UpdateRepositoryError> {
        let required = required_policy_bindings(plan);
        let mut policies: Vec<_> = self
            .policy_snapshots
            .iter()
            .filter(|policy| {
                policy_required_key(policy).is_some_and(|key| required.contains_key(&key))
            })
            .cloned()
            .collect();
        policies.sort_by_key(policy_sort_key);
        Ok(policies)
    }

    fn stage_context(
        &self,
        command: &UpdateCommand,
    ) -> Result<UpdateDecisionContext, UpdateRepositoryError> {
        let UpdateCommandAction::Stage { plan } = &command.action else {
            return Err(UpdateRepositoryError::DecisionRejected(
                UpdateDecisionError::IllegalTransition,
            ));
        };
        let installation = self
            .installation_repository
            .load_exact(plan.installation_id())
            .map_err(map_installation_repository_error)?
            .ok_or(UpdateRepositoryError::DecisionRejected(
                UpdateDecisionError::InstallationMissing,
            ))?;
        UpdateDecisionContext::for_stage(
            command,
            &installation,
            plan.target_pin().clone(),
            select_unique_catalog_by_revision(
                &self.catalog_read_models,
                plan.rollback_catalog_revision(),
            )?,
            &self.publications_for_pin(plan.rollback_pin()),
            select_unique_catalog_by_revision(
                &self.catalog_read_models,
                plan.target_catalog_revision(),
            )?,
            &self.publications_for_pin(plan.target_pin()),
            select_unique_registry_by_revision(
                &self.capability_registries,
                plan.rollback_registry_revision(),
            )?,
            select_unique_registry_by_revision(
                &self.capability_registries,
                plan.target_registry_revision(),
            )?,
            &self.policies_for_plan(plan)?,
        )
        .map_err(|_| UpdateRepositoryError::DecisionRejected(UpdateDecisionError::PlanMismatch))
    }

    fn decision_context_for_simple_command(
        &self,
        current: &PackageUpdateAggregate,
        command: &UpdateCommand,
    ) -> Result<UpdateDecisionContext, UpdateRepositoryError> {
        match &command.action {
            UpdateCommandAction::RecordApproval { approval, .. } => {
                if self
                    .consumed_approvals
                    .get(approval.approval_id())
                    .is_some_and(|(id, digest)| {
                        id != current.update_id() || digest != approval.evidence_digest()
                    })
                {
                    return Err(UpdateRepositoryError::DecisionRejected(
                        UpdateDecisionError::ApprovalAlreadyConsumed,
                    ));
                }
                let installation = self
                    .installation_repository
                    .load_exact(current.installation_id())
                    .map_err(map_installation_repository_error)?
                    .ok_or(UpdateRepositoryError::DecisionRejected(
                        UpdateDecisionError::InstallationMissing,
                    ))?;
                let plan = current.plan();
                let target_publications = self.publications_for_pin(plan.target_pin());
                let rollback_publications = self.publications_for_pin(plan.rollback_pin());
                let policies = self.policies_for_plan(plan)?;
                UpdateDecisionContext::for_record_approval(
                    &installation,
                    plan,
                    select_unique_catalog_by_revision(
                        &self.catalog_read_models,
                        plan.target_catalog_revision(),
                    )?,
                    &target_publications,
                    select_unique_catalog_by_revision(
                        &self.catalog_read_models,
                        plan.rollback_catalog_revision(),
                    )?,
                    &rollback_publications,
                    select_unique_registry_by_revision(
                        &self.capability_registries,
                        plan.target_registry_revision(),
                    )?,
                    select_unique_registry_by_revision(
                        &self.capability_registries,
                        plan.rollback_registry_revision(),
                    )?,
                    &policies,
                )
                .map_err(UpdateRepositoryError::DecisionRejected)
            }
            UpdateCommandAction::ConfirmAppliedUpdate { .. } => {
                let installation = self
                    .installation_repository
                    .load_exact(current.installation_id())
                    .map_err(map_installation_repository_error)?
                    .ok_or(UpdateRepositoryError::DecisionRejected(
                        UpdateDecisionError::InstallationMissing,
                    ))?;
                Ok(UpdateDecisionContext::for_confirm_applied(&installation))
            }
            UpdateCommandAction::Cancel { .. } => Ok(UpdateDecisionContext::for_cancel()),
            UpdateCommandAction::CancelAfterTerminalInstallation { .. } => {
                let installation = self
                    .installation_repository
                    .load_exact(current.installation_id())
                    .map_err(map_installation_repository_error)?
                    .ok_or(UpdateRepositoryError::DecisionRejected(
                        UpdateDecisionError::InstallationMissing,
                    ))?;
                Ok(UpdateDecisionContext::for_cancel_after_terminal_installation(&installation))
            }
            _ => Err(UpdateRepositoryError::DecisionRejected(
                UpdateDecisionError::IllegalTransition,
            )),
        }
    }

    fn prepare_apply_or_rollback(
        &self,
        current: &PackageUpdateAggregate,
        command: &UpdateCommand,
    ) -> Result<PreparedCoupledDecision, UpdateRepositoryError> {
        let rollback = matches!(command.action, UpdateCommandAction::Rollback { .. });
        let mut installation_repository = self.installation_repository.clone();
        let mut grant_repository = self.grant_repository.clone();
        let installation = installation_repository
            .load_exact(current.installation_id())
            .map_err(map_installation_repository_error)?
            .ok_or(UpdateRepositoryError::DecisionRejected(
                UpdateDecisionError::InstallationMissing,
            ))?;
        let command_digest = digest_command_identity(command.command_id(), current.update_id());
        let install_command_id = InstallationCommandId::parse(format!(
            "cmd:update-{}",
            command_digest.as_str().trim_start_matches("sha256:")
        ))
        .map_err(|_| UpdateRepositoryError::CorruptCurrentUpdateIndex)?;
        let installation_command = if rollback {
            InstallationCommand::package_rolled_back(
                install_command_id,
                installation.installation_id().clone(),
                installation.revision().clone(),
                current.plan().plan_digest().clone(),
                current.plan().rollback_pin().clone(),
            )
        } else {
            InstallationCommand::package_updated(
                install_command_id,
                installation.installation_id().clone(),
                installation.revision().clone(),
                current.plan().plan_digest().clone(),
                current.plan().target_pin().clone(),
            )
        }
        .map_err(|_| UpdateRepositoryError::CorruptCurrentUpdateIndex)?;
        let installation_receipt = installation_repository
            .execute(installation_command)
            .map_err(map_installation_repository_error)?;
        let installation_event = match installation_receipt.outcome() {
            InstallationCommandOutcome::Accepted { event, .. } => event.clone(),
            InstallationCommandOutcome::Rejected { .. } => {
                return Err(UpdateRepositoryError::DecisionRejected(
                    UpdateDecisionError::CoupledInstallationEventMismatch,
                ));
            }
        };
        let current_grants = grant_repository
            .load_current_for_installation(
                installation.tenant_id(),
                installation.user_id(),
                installation.installation_id(),
                installation.revision(),
            )
            .map_err(map_grant_repository_error)?;
        let mut grant_receipts = Vec::new();
        let mut grant_events = Vec::new();
        for grant in current_grants.grants() {
            if grant.state() != GrantState::Active {
                continue;
            }
            let grant_command_id = GrantCommandId::parse(format!(
                "grant-cmd:update-{}",
                digest_grant_command_identity(command.command_id(), grant.snapshot_id())
                    .as_str()
                    .trim_start_matches("sha256:")
            ))
            .map_err(|_| UpdateRepositoryError::CorruptCurrentUpdateIndex)?;
            let grant_command = GrantCommand::mark_stale(
                grant_command_id,
                grant.snapshot_id().clone(),
                grant.version().clone(),
                GrantInvalidationReason::InstallationChanged,
            )
            .map_err(|_| UpdateRepositoryError::CorruptCurrentUpdateIndex)?;
            let receipt = grant_repository
                .execute(grant_command)
                .map_err(map_grant_repository_error)?;
            match receipt.outcome() {
                GrantCommandOutcome::Accepted { event, .. } => grant_events.push(event.clone()),
                GrantCommandOutcome::Rejected { .. } => {
                    return Err(UpdateRepositoryError::DecisionRejected(
                        UpdateDecisionError::GrantSetConflict,
                    ));
                }
            }
            grant_receipts.push(receipt);
        }
        let plan = current.plan();
        let target_publications = self.publications_for_pin(plan.target_pin());
        let rollback_publications = self.publications_for_pin(plan.rollback_pin());
        let policies = self.policies_for_plan(plan)?;
        let target_catalog = select_unique_catalog_by_revision(
            &self.catalog_read_models,
            plan.target_catalog_revision(),
        )?;
        let rollback_catalog = select_unique_catalog_by_revision(
            &self.catalog_read_models,
            plan.rollback_catalog_revision(),
        )?;
        let target_registry = select_unique_registry_by_revision(
            &self.capability_registries,
            plan.target_registry_revision(),
        )?;
        let rollback_registry = select_unique_registry_by_revision(
            &self.capability_registries,
            plan.rollback_registry_revision(),
        )?;
        let context = if rollback {
            UpdateDecisionContext::for_rollback(
                &installation,
                plan,
                target_catalog,
                &target_publications,
                rollback_catalog,
                &rollback_publications,
                target_registry,
                rollback_registry,
                &policies,
                current_grants,
                &installation_event,
                &grant_events,
            )
        } else {
            UpdateDecisionContext::for_apply(
                &installation,
                plan,
                target_catalog,
                &target_publications,
                rollback_catalog,
                &rollback_publications,
                target_registry,
                rollback_registry,
                &policies,
                current_grants,
                &installation_event,
                &grant_events,
            )
        }
        .map_err(UpdateRepositoryError::DecisionRejected)?;
        Ok(PreparedCoupledDecision {
            context,
            installation_repository,
            grant_repository,
            installation_receipt: Some(installation_receipt),
            grant_receipts,
        })
    }

    fn authority_binding_for_plan(
        &self,
        plan: &PackageUpdatePlan,
    ) -> Result<AuthorityCarrierBinding, UpdateRepositoryError> {
        let target_publications = self.publications_for_pin(plan.target_pin());
        let rollback_publications = self.publications_for_pin(plan.rollback_pin());
        let policies = self.policies_for_plan(plan)?;
        Ok(AuthorityCarrierBinding::from_complete_parts(
            select_unique_catalog_by_revision(
                &self.catalog_read_models,
                plan.target_catalog_revision(),
            )?,
            &target_publications,
            select_unique_catalog_by_revision(
                &self.catalog_read_models,
                plan.rollback_catalog_revision(),
            )?,
            &rollback_publications,
            select_unique_registry_by_revision(
                &self.capability_registries,
                plan.target_registry_revision(),
            )?,
            select_unique_registry_by_revision(
                &self.capability_registries,
                plan.rollback_registry_revision(),
            )?,
            &policies,
        ))
    }

    fn coupled_admission_carriers(
        &self,
        current: &PackageUpdateAggregate,
    ) -> Result<(InstallationSnapshot, AuthorityCarrierBinding), UpdateRepositoryError> {
        let installation = self
            .installation_repository
            .load_exact(current.installation_id())
            .map_err(map_installation_repository_error)?
            .ok_or(UpdateRepositoryError::DecisionRejected(
                UpdateDecisionError::InstallationMissing,
            ))?;
        let authority = self.authority_binding_for_plan(current.plan())?;
        Ok((installation, authority))
    }

    fn persist_rejected_receipt(
        &mut self,
        command: UpdateCommand,
        error: UpdateDecisionError,
        witness: UpdateReceiptWitness,
    ) -> Result<UpdateCommandReceipt, UpdateRepositoryError> {
        if self.fail_next_commit {
            self.fail_next_commit = false;
            return Err(UpdateRepositoryError::InjectedPersistenceFailure);
        }
        let receipt = UpdateCommandReceipt {
            command: command.clone(),
            outcome: UpdateCommandOutcome::Rejected { error },
            witness,
            subordinate_installation_receipt: None,
            subordinate_grant_receipts: Vec::new(),
        };
        self.command_ledger.insert(
            command.command_id().clone(),
            UpdateLedgerEntry {
                command,
                receipt: receipt.clone(),
            },
        );
        Ok(receipt)
    }

    fn coupled_preflight_error(
        current: &PackageUpdateAggregate,
        command: &UpdateCommand,
    ) -> Option<UpdateDecisionError> {
        match &command.action {
            UpdateCommandAction::Apply {
                expected_update_revision,
                ..
            } => {
                if current.revision() != expected_update_revision {
                    Some(UpdateDecisionError::UpdateRevisionMismatch)
                } else if current.state() != UpdateState::Ready {
                    Some(UpdateDecisionError::IllegalTransition)
                } else {
                    None
                }
            }
            UpdateCommandAction::Rollback {
                expected_update_revision,
                ..
            } => {
                if current.revision() != expected_update_revision {
                    Some(UpdateDecisionError::UpdateRevisionMismatch)
                } else if current.state() != UpdateState::AppliedPendingConfirmation {
                    Some(UpdateDecisionError::RollbackUnavailable)
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn verify_current_update_slots(&self) -> Result<(), UpdateRepositoryError> {
        let mut expected = BTreeMap::new();
        for aggregate in self.aggregates.values() {
            if aggregate.state().is_terminal() {
                continue;
            }
            if expected
                .insert(
                    aggregate.installation_id().clone(),
                    aggregate.update_id().clone(),
                )
                .is_some()
            {
                return Err(UpdateRepositoryError::CorruptCurrentUpdateIndex);
            }
        }
        if expected == self.current_update_slots {
            Ok(())
        } else {
            Err(UpdateRepositoryError::CorruptCurrentUpdateIndex)
        }
    }

    fn apply_current_update_slot_event(
        &mut self,
        event: &UpdateEvent,
        snapshot: &PackageUpdateSnapshot,
    ) -> Result<(), UpdateRepositoryError> {
        match event.kind() {
            UpdateEventKind::Staged => {
                if self
                    .current_update_slots
                    .insert(
                        snapshot.installation_id().clone(),
                        snapshot.update_id().clone(),
                    )
                    .is_some()
                {
                    return Err(UpdateRepositoryError::CorruptCurrentUpdateIndex);
                }
            }
            UpdateEventKind::Confirmed
            | UpdateEventKind::RolledBack
            | UpdateEventKind::Cancelled => {
                if self
                    .current_update_slots
                    .remove(snapshot.installation_id())
                    .as_ref()
                    != Some(snapshot.update_id())
                {
                    return Err(UpdateRepositoryError::CorruptCurrentUpdateIndex);
                }
            }
            UpdateEventKind::ApprovalRecorded | UpdateEventKind::Applied => {}
        }
        Ok(())
    }

    fn verify_subordinate_references(
        &self,
        event: &UpdateEvent,
        pre_snapshot: Option<&PackageUpdateSnapshot>,
    ) -> Result<(), UpdateRepositoryError> {
        let (
            installation_event,
            grant_events,
            prior_installation_revision,
            expected_kind,
            prior_pin,
            next_pin,
        ) = match &event.payload {
            UpdateEventPayload::Applied {
                installation_event,
                grant_events,
                prior_installation_revision,
                ..
            } => {
                let plan = pre_snapshot
                    .ok_or(UpdateRepositoryError::CorruptCurrentUpdateIndex)?
                    .plan();
                (
                    installation_event,
                    grant_events,
                    prior_installation_revision,
                    InstallationEventKind::PackageUpdated,
                    plan.rollback_pin(),
                    plan.target_pin(),
                )
            }
            UpdateEventPayload::RolledBack {
                installation_event,
                grant_events,
                prior_installation_revision,
                ..
            } => {
                let plan = pre_snapshot
                    .ok_or(UpdateRepositoryError::CorruptCurrentUpdateIndex)?
                    .plan();
                (
                    installation_event,
                    grant_events,
                    prior_installation_revision,
                    InstallationEventKind::PackageRolledBack,
                    plan.target_pin(),
                    plan.rollback_pin(),
                )
            }
            _ => return Ok(()),
        };
        let plan = pre_snapshot
            .ok_or(UpdateRepositoryError::CorruptCurrentUpdateIndex)?
            .plan();
        let installation_events = self
            .installation_repository
            .event_history(&installation_event.installation_id)
            .map_err(map_installation_repository_error)?;
        let owner_installation_event = installation_events.iter().find(|candidate| {
            installation_event.matches_event(&installation_event.installation_id, candidate)
        });
        if owner_installation_event.is_none_or(|candidate| {
            !candidate.matches_package_pin_change(
                expected_kind,
                &installation_event.installation_id,
                prior_installation_revision,
                plan.plan_digest(),
                prior_pin,
                next_pin,
            )
        }) {
            return Err(UpdateRepositoryError::CorruptCurrentUpdateIndex);
        }
        for reference in grant_events {
            let events = self
                .grant_repository
                .event_history(&reference.snapshot_id)
                .map_err(map_grant_repository_error)?;
            if !events
                .iter()
                .any(|candidate| reference.matches_event(candidate))
            {
                return Err(UpdateRepositoryError::CorruptCurrentUpdateIndex);
            }
        }
        Ok(())
    }

    fn verify_receipt_owner_context_bindings(
        &self,
        receipt: &UpdateCommandReceipt,
    ) -> Result<(), UpdateRepositoryError> {
        if let (
            UpdateCommandOutcome::Rejected { .. },
            UpdateReceiptWitness::CoupledDecisionPreflight {
                snapshot,
                installation,
                authority,
            },
        ) = (&receipt.outcome, &receipt.witness)
        {
            let installation_events = self
                .installation_repository
                .event_history(installation.installation_id())
                .map_err(map_installation_repository_error)?;
            let installation_prefix_len = usize::try_from(installation.last_sequence().get())
                .map_err(|_| UpdateRepositoryError::CorruptCurrentUpdateIndex)?;
            let expected_authority = self
                .authority_binding_for_plan(snapshot.plan())
                .map_err(|_| UpdateRepositoryError::CorruptCurrentUpdateIndex)?;
            if installation_prefix_len > installation_events.len()
                || installation_replay(installation_events[..installation_prefix_len].iter())
                    .map_err(UpdateRepositoryError::CorruptInstallationHistory)?
                    .as_ref()
                    != Some(installation.as_ref())
                || expected_authority != **authority
            {
                return Err(UpdateRepositoryError::CorruptCurrentUpdateIndex);
            }
            return Ok(());
        }
        let UpdateCommandOutcome::Accepted { .. } = &receipt.outcome else {
            return Ok(());
        };
        let UpdateReceiptWitness::Decision(context) = &receipt.witness else {
            return Ok(());
        };
        let (installation, current_grants, installation_event) = match &context.kind {
            UpdateDecisionContextKind::Apply {
                installation,
                current_grants,
                installation_event,
                ..
            }
            | UpdateDecisionContextKind::Rollback {
                installation,
                current_grants,
                installation_event,
                ..
            } => (installation, current_grants, installation_event),
            _ => return Ok(()),
        };
        if !current_grants.is_canonical() {
            return Err(UpdateRepositoryError::CorruptCurrentUpdateIndex);
        }

        let installation_events = self
            .installation_repository
            .event_history(installation.installation_id())
            .map_err(map_installation_repository_error)?;
        let installation_prefix_len = installation_event
            .sequence
            .get()
            .checked_sub(1)
            .and_then(|value| usize::try_from(value).ok())
            .ok_or(UpdateRepositoryError::CorruptCurrentUpdateIndex)?;
        if installation_prefix_len > installation_events.len()
            || installation_replay(installation_events[..installation_prefix_len].iter())
                .map_err(UpdateRepositoryError::CorruptInstallationHistory)?
                .as_ref()
                != Some(installation)
        {
            return Err(UpdateRepositoryError::CorruptCurrentUpdateIndex);
        }

        let mut context_grant_ids = BTreeSet::new();
        for grant in current_grants.grants() {
            if !context_grant_ids.insert(grant.snapshot_id().clone()) {
                return Err(UpdateRepositoryError::CorruptCurrentUpdateIndex);
            }
            let grant_events = self
                .grant_repository
                .event_history(grant.snapshot_id())
                .map_err(map_grant_repository_error)?;
            let grant_prefix_len = usize::try_from(grant.last_sequence().get())
                .map_err(|_| UpdateRepositoryError::CorruptCurrentUpdateIndex)?;
            if grant_prefix_len > grant_events.len()
                || grant_replay(grant_events[..grant_prefix_len].iter())
                    .map_err(UpdateRepositoryError::CorruptGrantHistory)?
                    .as_ref()
                    != Some(grant)
            {
                return Err(UpdateRepositoryError::CorruptCurrentUpdateIndex);
            }
        }

        let final_grants = self
            .grant_repository
            .load_current_for_installation(
                installation.tenant_id(),
                installation.user_id(),
                installation.installation_id(),
                installation.revision(),
            )
            .map_err(map_grant_repository_error)?;
        if final_grants.grants().iter().any(|grant| {
            grant.state() == GrantState::Active
                && grant.installation_revision() == installation.revision()
                && !context_grant_ids.contains(grant.snapshot_id())
        }) {
            return Err(UpdateRepositoryError::CorruptCurrentUpdateIndex);
        }
        Ok(())
    }
}

struct PreparedCoupledDecision {
    context: UpdateDecisionContext,
    installation_repository: InMemoryInstallationRepository,
    grant_repository: InMemoryGrantRepository,
    installation_receipt: Option<InstallationCommandReceipt>,
    grant_receipts: Vec<GrantCommandReceipt>,
}

type InstallationCouplingKey = (String, u64, u8, String);
type GrantCouplingKey = (String, u64, String);

fn installation_coupling_kind(kind: InstallationEventKind) -> Option<u8> {
    match kind {
        InstallationEventKind::PackageUpdated => Some(0),
        InstallationEventKind::PackageRolledBack => Some(1),
        _ => None,
    }
}

fn collect_owner_installation_couplings(
    histories: &[(InstallationId, Vec<InstallationEvent>)],
) -> Result<BTreeSet<InstallationCouplingKey>, UpdateRepositoryError> {
    let mut keys = BTreeSet::new();
    for (installation_id, events) in histories {
        for event in events {
            let Some(kind) = installation_coupling_kind(event.kind()) else {
                continue;
            };
            let key = (
                installation_id.as_str().to_owned(),
                event.sequence().get(),
                kind,
                event.canonical_coupling_digest().as_str().to_owned(),
            );
            if !keys.insert(key) {
                return Err(UpdateRepositoryError::CorruptCurrentUpdateIndex);
            }
        }
    }
    Ok(keys)
}

fn collect_owner_grant_couplings(
    histories: &[(GrantSnapshotId, Vec<GrantEvent>)],
) -> Result<BTreeSet<GrantCouplingKey>, UpdateRepositoryError> {
    let mut keys = BTreeSet::new();
    for (snapshot_id, events) in histories {
        for event in events {
            if event.kind() != GrantEventKind::MarkedStale
                || event.invalidation_reason() != Some(GrantInvalidationReason::InstallationChanged)
                || !event.command_id().as_str().starts_with("grant-cmd:update-")
            {
                continue;
            }
            let key = (
                snapshot_id.as_str().to_owned(),
                event.sequence().get(),
                event.canonical_coupling_digest().as_str().to_owned(),
            );
            if !keys.insert(key) {
                return Err(UpdateRepositoryError::CorruptCurrentUpdateIndex);
            }
        }
    }
    Ok(keys)
}

fn collect_update_coupling_references(
    event: &UpdateEvent,
    installation_keys: &mut BTreeSet<InstallationCouplingKey>,
    grant_keys: &mut BTreeSet<GrantCouplingKey>,
) -> Result<(), UpdateRepositoryError> {
    let (installation_event, grant_events) = match &event.payload {
        UpdateEventPayload::Applied {
            installation_event,
            grant_events,
            ..
        }
        | UpdateEventPayload::RolledBack {
            installation_event,
            grant_events,
            ..
        } => (installation_event, grant_events),
        _ => return Ok(()),
    };
    let Some(kind) = installation_coupling_kind(installation_event.kind) else {
        return Err(UpdateRepositoryError::CorruptCurrentUpdateIndex);
    };
    let installation_key = (
        installation_event.installation_id.as_str().to_owned(),
        installation_event.sequence.get(),
        kind,
        installation_event.event_digest.as_str().to_owned(),
    );
    if !installation_keys.insert(installation_key) {
        return Err(UpdateRepositoryError::CorruptCurrentUpdateIndex);
    }
    for reference in grant_events {
        if reference.kind != GrantEventKind::MarkedStale {
            return Err(UpdateRepositoryError::CorruptCurrentUpdateIndex);
        }
        let grant_key = (
            reference.snapshot_id.as_str().to_owned(),
            reference.sequence.get(),
            reference.event_digest.as_str().to_owned(),
        );
        if !grant_keys.insert(grant_key) {
            return Err(UpdateRepositoryError::CorruptCurrentUpdateIndex);
        }
    }
    Ok(())
}

fn map_installation_repository_error(error: InstallationRepositoryError) -> UpdateRepositoryError {
    match error {
        InstallationRepositoryError::CommandConflict => UpdateRepositoryError::CommandConflict,
        InstallationRepositoryError::InjectedPersistenceFailure => {
            UpdateRepositoryError::InjectedPersistenceFailure
        }
        InstallationRepositoryError::CorruptEventHistory(error) => {
            UpdateRepositoryError::CorruptInstallationHistory(error)
        }
        InstallationRepositoryError::CorruptCommandLedger => {
            UpdateRepositoryError::CorruptCurrentUpdateIndex
        }
        InstallationRepositoryError::DecisionRejected(_) => {
            UpdateRepositoryError::TransactionConflict
        }
    }
}

fn map_grant_repository_error(error: GrantRepositoryError) -> UpdateRepositoryError {
    match error {
        GrantRepositoryError::CommandConflict => UpdateRepositoryError::CommandConflict,
        GrantRepositoryError::InjectedPersistenceFailure => {
            UpdateRepositoryError::InjectedPersistenceFailure
        }
        GrantRepositoryError::CorruptEventHistory(error) => {
            UpdateRepositoryError::CorruptGrantHistory(error)
        }
        GrantRepositoryError::CorruptAuthorityIndex => UpdateRepositoryError::CorruptGrantIndex,
        GrantRepositoryError::DecisionRejected(_) => UpdateRepositoryError::CorruptGrantSet,
    }
}

impl PackageUpdateRepository for InMemoryPackageUpdateRepository {
    fn execute(
        &mut self,
        command: UpdateCommand,
    ) -> Result<UpdateCommandReceipt, UpdateRepositoryError> {
        if let Some(entry) = self.command_ledger.get(command.command_id()) {
            return if entry.command == command {
                Ok(entry.receipt.clone())
            } else {
                Err(UpdateRepositoryError::CommandConflict)
            };
        }

        self.verify_current_update_slots()?;
        let current = self.aggregates.get(command.update_id()).cloned();
        if matches!(command.action, UpdateCommandAction::Stage { .. })
            && command_stage_installation_id(&command).is_some_and(|id| {
                self.current_update_slots.get(id).is_some_and(|slot| {
                    Some(slot) != current.as_ref().map(PackageUpdateAggregate::update_id)
                })
            })
        {
            if self.fail_next_commit {
                self.fail_next_commit = false;
                return Err(UpdateRepositoryError::InjectedPersistenceFailure);
            }
            let conflict_installation_id = command_stage_installation_id(&command)
                .expect("stage conflict has installation id")
                .clone();
            let receipt = UpdateCommandReceipt {
                command: command.clone(),
                outcome: UpdateCommandOutcome::Rejected {
                    error: UpdateDecisionError::ActiveUpdateConflict,
                },
                witness: UpdateReceiptWitness::ActiveSlotConflict {
                    installation_id: conflict_installation_id.clone(),
                    conflicting_update_id: self
                        .current_update_slots
                        .get(&conflict_installation_id)
                        .expect("stage conflict has slot")
                        .clone(),
                    conflicting_state: self
                        .aggregates
                        .get(
                            self.current_update_slots
                                .get(&conflict_installation_id)
                                .expect("stage conflict has slot"),
                        )
                        .map_or(UpdateState::Staged, PackageUpdateAggregate::state),
                },
                subordinate_installation_receipt: None,
                subordinate_grant_receipts: Vec::new(),
            };
            self.command_ledger.insert(
                command.command_id().clone(),
                UpdateLedgerEntry {
                    command,
                    receipt: receipt.clone(),
                },
            );
            return Ok(receipt);
        }

        let mut prepared_coupled = None;
        let context_result = match &command.action {
            UpdateCommandAction::Stage { .. } => self.stage_context(&command),
            UpdateCommandAction::Apply { .. } | UpdateCommandAction::Rollback { .. } => {
                let Some(current_ref) = current.as_ref() else {
                    return self.persist_rejected_receipt(
                        command,
                        UpdateDecisionError::AggregateMissing,
                        UpdateReceiptWitness::MissingAggregate,
                    );
                };
                if let Some(error) = Self::coupled_preflight_error(current_ref, &command) {
                    return self.persist_rejected_receipt(
                        command,
                        error,
                        UpdateReceiptWitness::RepositoryPreflight {
                            snapshot: Box::new(current_ref.clone()),
                        },
                    );
                }
                let (installation, authority) = self.coupled_admission_carriers(current_ref)?;
                if let Err(error) =
                    validate_coupled_admission(current_ref, &command, &installation, &authority)
                {
                    return self.persist_rejected_receipt(
                        command,
                        error,
                        UpdateReceiptWitness::CoupledDecisionPreflight {
                            snapshot: Box::new(current_ref.clone()),
                            installation: Box::new(installation),
                            authority: Box::new(authority),
                        },
                    );
                }
                self.prepare_apply_or_rollback(current_ref, &command)
                    .map(|prepared| {
                        let context = prepared.context.clone();
                        prepared_coupled = Some(prepared);
                        context
                    })
            }
            _ => {
                let Some(current_ref) = current.as_ref() else {
                    return self.persist_rejected_receipt(
                        command,
                        UpdateDecisionError::AggregateMissing,
                        UpdateReceiptWitness::MissingAggregate,
                    );
                };
                let consumed_approval = match &command.action {
                    UpdateCommandAction::RecordApproval { approval, .. } => {
                        self.consumed_approvals.get(approval.approval_id()).cloned()
                    }
                    _ => None,
                };
                if let Some((prior_update_id, prior_evidence_digest)) = consumed_approval {
                    return self.persist_rejected_receipt(
                        command,
                        UpdateDecisionError::ApprovalAlreadyConsumed,
                        UpdateReceiptWitness::ApprovalAlreadyConsumed {
                            prior_update_id,
                            prior_evidence_digest,
                        },
                    );
                }
                self.decision_context_for_simple_command(current_ref, &command)
            }
        };

        let decision = match &context_result {
            Ok(context) => decide(current.as_ref(), context, &command),
            Err(UpdateRepositoryError::DecisionRejected(error)) => Err(*error),
            Err(error) => return Err(error.clone()),
        };
        let prepared_witness = match &context_result {
            Ok(context) => UpdateReceiptWitness::Decision(Box::new(context.clone())),
            Err(UpdateRepositoryError::DecisionRejected(UpdateDecisionError::AggregateMissing)) => {
                UpdateReceiptWitness::MissingAggregate
            }
            Err(_) => return Err(UpdateRepositoryError::TransactionConflict),
        };
        let prepared_outcome = match decision {
            Ok(event) => {
                let snapshot = evolve(current.clone(), &event)
                    .map_err(UpdateRepositoryError::CorruptUpdateHistory)?;
                UpdateCommandOutcome::Accepted { event, snapshot }
            }
            Err(error) => UpdateCommandOutcome::Rejected { error },
        };
        if self.fail_next_commit {
            self.fail_next_commit = false;
            return Err(UpdateRepositoryError::InjectedPersistenceFailure);
        }
        let receipt = UpdateCommandReceipt {
            command: command.clone(),
            outcome: prepared_outcome.clone(),
            witness: prepared_witness,
            subordinate_installation_receipt: prepared_coupled
                .as_ref()
                .and_then(|prepared| prepared.installation_receipt.clone()),
            subordinate_grant_receipts: prepared_coupled
                .as_ref()
                .map_or_else(Vec::new, |prepared| prepared.grant_receipts.clone()),
        };
        if let UpdateCommandOutcome::Accepted {
            event, snapshot, ..
        } = &prepared_outcome
        {
            if let Some(prepared) = prepared_coupled {
                self.installation_repository = prepared.installation_repository;
                self.grant_repository = prepared.grant_repository;
            }
            if let UpdateEventPayload::ApprovalRecorded { approval, .. } = &event.payload {
                self.note_consumed_approval(snapshot.update_id().clone(), approval)?;
            }
            self.apply_current_update_slot_event(event, snapshot)?;
            self.events
                .entry(event.update_id().clone())
                .or_default()
                .push(event.clone());
            self.aggregates
                .insert(snapshot.update_id().clone(), snapshot.clone());
        }
        self.command_ledger.insert(
            command.command_id().clone(),
            UpdateLedgerEntry {
                command,
                receipt: receipt.clone(),
            },
        );
        Ok(receipt)
    }

    fn load_exact(
        &self,
        id: &PackageUpdateId,
    ) -> Result<Option<PackageUpdateSnapshot>, UpdateRepositoryError> {
        self.verify_current_update_slots()?;
        let events = self.events.get(id).cloned().unwrap_or_default();
        if events.is_empty() {
            return Ok(None);
        }
        let mut replayed = None;
        for event in &events {
            self.verify_subordinate_references(event, replayed.as_ref())?;
            replayed =
                Some(evolve(replayed, event).map_err(UpdateRepositoryError::CorruptUpdateHistory)?);
        }
        if replayed.as_ref() != self.aggregates.get(id) {
            return Err(UpdateRepositoryError::CorruptCurrentUpdateIndex);
        }
        Ok(replayed)
    }

    fn event_history(
        &self,
        id: &PackageUpdateId,
    ) -> Result<Vec<UpdateEvent>, UpdateRepositoryError> {
        Ok(self.events.get(id).cloned().unwrap_or_default())
    }
}

fn validate_update_receipt_against_histories(
    receipt: &UpdateCommandReceipt,
    observed_pre_snapshot: &Option<PackageUpdateSnapshot>,
    ledger_current: &mut BTreeMap<PackageUpdateId, Option<PackageUpdateSnapshot>>,
    ledger_current_slots: &mut BTreeMap<InstallationId, PackageUpdateId>,
    ledger_consumed_approvals: &mut BTreeMap<UpdateApprovalId, (PackageUpdateId, Sha256Digest)>,
    accepted_events_by_command: &mut BTreeMap<
        UpdateCommandId,
        (
            PackageUpdateId,
            UpdateEvent,
            Option<PackageUpdateSnapshot>,
            PackageUpdateSnapshot,
        ),
    >,
) -> Result<(), UpdateRepositoryError> {
    let command = receipt.command();
    if observed_pre_snapshot
        .as_ref()
        .is_some_and(|snapshot| snapshot.update_id() != command.update_id())
    {
        return Err(UpdateRepositoryError::CorruptCurrentUpdateIndex);
    }
    let exact_prefix = ledger_current
        .entry(command.update_id().clone())
        .or_insert(None);
    if exact_prefix != observed_pre_snapshot {
        return Err(UpdateRepositoryError::CorruptCurrentUpdateIndex);
    }
    match receipt.outcome() {
        UpdateCommandOutcome::Accepted { event, snapshot } => {
            let UpdateReceiptWitness::Decision(context) = &receipt.witness else {
                return Err(UpdateRepositoryError::CorruptCurrentUpdateIndex);
            };
            if decide(observed_pre_snapshot.as_ref(), context, command).as_ref() != Ok(event) {
                return Err(UpdateRepositoryError::CorruptCurrentUpdateIndex);
            }
            let evolved = evolve(observed_pre_snapshot.clone(), event)
                .map_err(UpdateRepositoryError::CorruptUpdateHistory)?;
            if &evolved != snapshot {
                return Err(UpdateRepositoryError::CorruptCurrentUpdateIndex);
            }
            let Some((history_id, history_event, history_pre, history_snapshot)) =
                accepted_events_by_command.remove(command.command_id())
            else {
                return Err(UpdateRepositoryError::CorruptCurrentUpdateIndex);
            };
            if &history_id != command.update_id()
                || &history_event != event
                || &history_pre != observed_pre_snapshot
                || &history_snapshot != snapshot
            {
                return Err(UpdateRepositoryError::CorruptCurrentUpdateIndex);
            }
            *exact_prefix = Some(snapshot.clone());
            match &event.payload {
                UpdateEventPayload::Staged { .. } => {
                    if ledger_current_slots
                        .insert(
                            snapshot.installation_id().clone(),
                            snapshot.update_id().clone(),
                        )
                        .is_some()
                    {
                        return Err(UpdateRepositoryError::CorruptCurrentUpdateIndex);
                    }
                }
                UpdateEventPayload::ApprovalRecorded { approval, .. } => {
                    if ledger_consumed_approvals
                        .insert(
                            approval.approval_id().clone(),
                            (
                                snapshot.update_id().clone(),
                                approval.evidence_digest().clone(),
                            ),
                        )
                        .is_some()
                    {
                        return Err(UpdateRepositoryError::CorruptCurrentUpdateIndex);
                    }
                }
                UpdateEventPayload::Confirmed { .. }
                | UpdateEventPayload::RolledBack { .. }
                | UpdateEventPayload::Cancelled { .. } => {
                    if ledger_current_slots
                        .remove(snapshot.installation_id())
                        .as_ref()
                        != Some(snapshot.update_id())
                    {
                        return Err(UpdateRepositoryError::CorruptCurrentUpdateIndex);
                    }
                }
                UpdateEventPayload::Applied { .. } => {}
            }
        }
        UpdateCommandOutcome::Rejected { error } => {
            if accepted_events_by_command.contains_key(command.command_id()) {
                return Err(UpdateRepositoryError::CorruptCurrentUpdateIndex);
            }
            match &receipt.witness {
                UpdateReceiptWitness::Decision(context) => {
                    if decide(observed_pre_snapshot.as_ref(), context, command)
                        .err()
                        .as_ref()
                        != Some(error)
                    {
                        return Err(UpdateRepositoryError::CorruptCurrentUpdateIndex);
                    }
                }
                UpdateReceiptWitness::MissingAggregate => {
                    if observed_pre_snapshot.is_some()
                        || *error != UpdateDecisionError::AggregateMissing
                    {
                        return Err(UpdateRepositoryError::CorruptCurrentUpdateIndex);
                    }
                }
                UpdateReceiptWitness::RepositoryPreflight { snapshot } => {
                    if observed_pre_snapshot.as_ref() != Some(snapshot.as_ref())
                        || InMemoryPackageUpdateRepository::coupled_preflight_error(
                            snapshot, command,
                        )
                        .as_ref()
                            != Some(error)
                    {
                        return Err(UpdateRepositoryError::CorruptCurrentUpdateIndex);
                    }
                }
                UpdateReceiptWitness::CoupledDecisionPreflight {
                    snapshot,
                    installation,
                    authority,
                } => {
                    if observed_pre_snapshot.as_ref() != Some(snapshot.as_ref())
                        || validate_coupled_admission(snapshot, command, installation, authority)
                            .err()
                            .as_ref()
                            != Some(error)
                    {
                        return Err(UpdateRepositoryError::CorruptCurrentUpdateIndex);
                    }
                }
                UpdateReceiptWitness::ApprovalAlreadyConsumed {
                    prior_update_id,
                    prior_evidence_digest,
                } => {
                    let UpdateCommandAction::RecordApproval { approval, .. } = &command.action
                    else {
                        return Err(UpdateRepositoryError::CorruptCurrentUpdateIndex);
                    };
                    if *error != UpdateDecisionError::ApprovalAlreadyConsumed
                        || ledger_consumed_approvals.get(approval.approval_id())
                            != Some(&(prior_update_id.clone(), prior_evidence_digest.clone()))
                    {
                        return Err(UpdateRepositoryError::CorruptCurrentUpdateIndex);
                    }
                }
                UpdateReceiptWitness::ActiveSlotConflict {
                    installation_id,
                    conflicting_update_id,
                    conflicting_state,
                } => {
                    let conflicting_snapshot = ledger_current
                        .get(conflicting_update_id)
                        .and_then(Option::as_ref);
                    if *error != UpdateDecisionError::ActiveUpdateConflict
                        || command_stage_installation_id(command) != Some(installation_id)
                        || conflicting_update_id == command.update_id()
                        || conflicting_state.is_terminal()
                        || ledger_current_slots.get(installation_id) != Some(conflicting_update_id)
                        || conflicting_snapshot.is_none_or(|snapshot| {
                            snapshot.installation_id() != installation_id
                                || snapshot.state() != *conflicting_state
                        })
                    {
                        return Err(UpdateRepositoryError::CorruptCurrentUpdateIndex);
                    }
                }
            }
        }
    }
    Ok(())
}

fn command_stage_installation_id(command: &UpdateCommand) -> Option<&InstallationId> {
    match &command.action {
        UpdateCommandAction::Stage { plan } => Some(plan.installation_id()),
        _ => None,
    }
}

fn digest_command_identity(
    command_id: &UpdateCommandId,
    update_id: &PackageUpdateId,
) -> Sha256Digest {
    let mut bytes = b"market-update-subordinate-installation-command/v0\0".to_vec();
    encode_string(command_id.as_str(), &mut bytes);
    encode_string(update_id.as_str(), &mut bytes);
    Sha256Digest::from_bytes(&bytes)
}

fn digest_grant_command_identity(
    command_id: &UpdateCommandId,
    snapshot_id: &GrantSnapshotId,
) -> Sha256Digest {
    let mut bytes = b"market-update-subordinate-grant-command/v0\0".to_vec();
    encode_string(command_id.as_str(), &mut bytes);
    encode_string(snapshot_id.as_str(), &mut bytes);
    Sha256Digest::from_bytes(&bytes)
}

impl UpdateState {
    const fn is_terminal(self) -> bool {
        matches!(self, Self::Confirmed | Self::RolledBack | Self::Cancelled)
    }
}

fn require_update_current<'a>(
    current: Option<&'a PackageUpdateAggregate>,
    command: &UpdateCommand,
    expected_revision: &UpdateRevision,
) -> Result<&'a PackageUpdateAggregate, UpdateDecisionError> {
    let agg = current.ok_or(UpdateDecisionError::AggregateMissing)?;
    if agg.update_id() != command.update_id() {
        return Err(UpdateDecisionError::UpdateIdentityMismatch);
    }
    if agg.revision() != expected_revision {
        return Err(UpdateDecisionError::UpdateRevisionMismatch);
    }
    Ok(agg)
}

fn validate_apply_admission<'a>(
    current: Option<&'a PackageUpdateAggregate>,
    command: &UpdateCommand,
    installation: &InstallationSnapshot,
    authority: &AuthorityCarrierBinding,
) -> Result<&'a PackageUpdateAggregate, UpdateDecisionError> {
    let UpdateCommandAction::Apply {
        expected_update_revision,
        expected_installation_revision,
    } = &command.action
    else {
        return Err(UpdateDecisionError::IllegalTransition);
    };
    let agg = require_update_current(current, command, expected_update_revision)?;
    if agg.state != UpdateState::Ready {
        return Err(UpdateDecisionError::IllegalTransition);
    }
    require_current_installation(
        installation,
        agg,
        expected_installation_revision,
        agg.plan.rollback_pin(),
        true,
    )?;
    let readiness = agg
        .readiness()
        .ok_or(UpdateDecisionError::ReadinessMissingOrMismatch)?;
    authority.matches_readiness(&agg.plan, readiness)?;
    if !matches!(
        installation.state(),
        ManagedInstallationState::InstalledDisabled | ManagedInstallationState::Disabled
    ) {
        return Err(UpdateDecisionError::InstallationMustBeDisabled);
    }
    Ok(agg)
}

fn validate_rollback_admission<'a>(
    current: Option<&'a PackageUpdateAggregate>,
    command: &UpdateCommand,
    installation: &InstallationSnapshot,
    authority: &AuthorityCarrierBinding,
) -> Result<&'a PackageUpdateAggregate, UpdateDecisionError> {
    let UpdateCommandAction::Rollback {
        expected_update_revision,
        expected_installation_revision,
        evidence,
    } = &command.action
    else {
        return Err(UpdateDecisionError::IllegalTransition);
    };
    let agg = require_update_current(current, command, expected_update_revision)?;
    if agg.state != UpdateState::AppliedPendingConfirmation {
        return Err(UpdateDecisionError::RollbackUnavailable);
    }
    require_current_installation(
        installation,
        agg,
        expected_installation_revision,
        agg.plan.target_pin(),
        true,
    )?;
    authority.matches_rollback_admission(&agg.plan, evidence)?;
    if !matches!(
        installation.state(),
        ManagedInstallationState::InstalledDisabled | ManagedInstallationState::Disabled
    ) {
        return Err(UpdateDecisionError::InstallationMustBeDisabled);
    }
    if evidence.update_id() != agg.update_id()
        || evidence.expected_update_revision() != expected_update_revision
        || evidence.current_target_installation_revision() != expected_installation_revision
        || evidence.rollback_pin_digest() != &digest_pin_value(agg.plan.rollback_pin())
    {
        return Err(UpdateDecisionError::RollbackEvidenceMismatch);
    }
    Ok(agg)
}

fn validate_coupled_admission(
    current: &PackageUpdateAggregate,
    command: &UpdateCommand,
    installation: &InstallationSnapshot,
    authority: &AuthorityCarrierBinding,
) -> Result<(), UpdateDecisionError> {
    match &command.action {
        UpdateCommandAction::Apply { .. } => {
            validate_apply_admission(Some(current), command, installation, authority)?;
        }
        UpdateCommandAction::Rollback { .. } => {
            validate_rollback_admission(Some(current), command, installation, authority)?;
        }
        _ => return Err(UpdateDecisionError::IllegalTransition),
    }
    Ok(())
}

fn require_current_installation(
    installation: &InstallationSnapshot,
    agg: &PackageUpdateAggregate,
    expected_revision: &InstallationRevision,
    expected_pin: &InstallationPackagePin,
    require_config: bool,
) -> Result<(), UpdateDecisionError> {
    if installation.installation_id() != agg.installation_id()
        || installation.tenant_id() != agg.tenant_id()
        || installation.user_id() != agg.user_id()
    {
        return Err(UpdateDecisionError::UpdateIdentityMismatch);
    }
    if installation.revision() != expected_revision {
        return Err(UpdateDecisionError::InstallationRevisionMismatch);
    }
    if installation.package_pin() != expected_pin {
        return Err(UpdateDecisionError::InstallationPinMismatch);
    }
    if matches!(
        installation.state(),
        ManagedInstallationState::Revoked | ManagedInstallationState::Uninstalled
    ) {
        return Err(UpdateDecisionError::InstallationTerminal);
    }
    if require_config
        && (installation.configuration_revision() != agg.plan.staged_configuration_revision()
            || installation.configuration().digest() != agg.plan.staged_configuration_digest())
    {
        return Err(UpdateDecisionError::ConfigurationChanged);
    }
    Ok(())
}
fn validate_grants(
    current_grants: &CurrentInstallationGrantSet,
    installation: &InstallationSnapshot,
    events: &[GrantEventReference],
) -> Result<(), UpdateDecisionError> {
    if current_grants.tenant_id() != installation.tenant_id()
        || current_grants.user_id() != installation.user_id()
        || current_grants.installation_id() != installation.installation_id()
        || current_grants.observed_installation_revision() != installation.revision()
    {
        return Err(UpdateDecisionError::GrantSetConflict);
    }
    let mut active_refs = BTreeSet::new();
    for event in events {
        if event.kind != GrantEventKind::MarkedStale
            || !active_refs.insert(event.snapshot_id.clone())
        {
            return Err(UpdateDecisionError::ActiveGrantBindingMismatch);
        }
    }
    let mut active = 0usize;
    for grant in current_grants.grants() {
        if grant.tenant_id() != installation.tenant_id()
            || grant.user_id() != installation.user_id()
            || grant.installation_id() != installation.installation_id()
        {
            return Err(UpdateDecisionError::GrantSetConflict);
        }
        match grant.state() {
            GrantState::Active => {
                active += 1;
                if grant.installation_revision() != installation.revision()
                    || grant.package_id() != installation.package_pin().package_id()
                    || grant.package_version() != installation.package_pin().package_version()
                    || grant.package_digest() != installation.package_pin().package_digest()
                    || grant.capability_manifest_digest()
                        != installation.package_pin().capability_manifest_digest()
                {
                    return Err(UpdateDecisionError::ActiveGrantBindingMismatch);
                }
                let Some(reference) = events
                    .iter()
                    .find(|event| event.snapshot_id == *grant.snapshot_id())
                else {
                    return Err(UpdateDecisionError::ActiveGrantBindingMismatch);
                };
                if reference.sequence.get() != grant.last_sequence().get().saturating_add(1) {
                    return Err(UpdateDecisionError::ActiveGrantBindingMismatch);
                }
            }
            GrantState::Stale | GrantState::Expired => {
                if active_refs.contains(grant.snapshot_id()) {
                    return Err(UpdateDecisionError::ActiveGrantBindingMismatch);
                }
            }
            GrantState::Revoked => return Err(UpdateDecisionError::GrantSetConflict),
        }
    }
    if events.len() != active {
        return Err(UpdateDecisionError::GrantSetConflict);
    }
    Ok(())
}
fn validate_active_grant_refs(
    _plan: &PackageUpdatePlan,
    refs: &[GrantEventReference],
) -> Result<(), ()> {
    let mut seen = BTreeSet::new();
    for r in refs {
        if r.kind != GrantEventKind::MarkedStale || !seen.insert(r.snapshot_id.clone()) {
            return Err(());
        }
    }
    Ok(())
}

fn canonical_policy_snapshots(
    policies: &[InvocationPolicySnapshot],
    plan: &PackageUpdatePlan,
) -> Result<Vec<InvocationPolicySnapshot>, UpdateConstructionError> {
    let required = required_policy_bindings(plan);
    let mut canonical = policies.to_vec();
    canonical.sort_by_key(policy_sort_key);
    let mut seen_rows = BTreeSet::new();
    let mut seen_required = BTreeSet::new();
    for policy in &canonical {
        let row = policy_sort_key(policy);
        if !seen_rows.insert(row) || policy.emergency_blocked {
            return Err(UpdateConstructionError::AuthorityClassificationIncomplete);
        }
        let binding = policy_required_key(policy)
            .ok_or(UpdateConstructionError::AuthorityClassificationIncomplete)?;
        if !required.contains_key(&binding) || !seen_required.insert(binding.clone()) {
            return Err(UpdateConstructionError::AuthorityClassificationIncomplete);
        }
        let expected_class = required
            .get(&binding)
            .ok_or(UpdateConstructionError::AuthorityClassificationIncomplete)?;
        if policy.capability_class != *expected_class {
            return Err(UpdateConstructionError::AuthorityClassificationIncomplete);
        }
    }
    if seen_required.len() != required.len() {
        return Err(UpdateConstructionError::AuthorityClassificationIncomplete);
    }
    Ok(canonical)
}

type PolicyBindingKey = (String, String, String, String);

fn required_policy_bindings(
    plan: &PackageUpdatePlan,
) -> BTreeMap<PolicyBindingKey, Option<CapabilityClass>> {
    let mut required = BTreeMap::new();
    collect_required_policy_bindings(&plan.rollback, &mut required);
    collect_required_policy_bindings(&plan.target, &mut required);
    required
}

fn collect_required_policy_bindings(
    authority: &PlanPackageAuthority,
    required: &mut BTreeMap<PolicyBindingKey, Option<CapabilityClass>>,
) {
    for component in &authority.publication_components {
        for capability in &component.declared_capabilities {
            if !authority.package.capabilities().contains(capability) {
                continue;
            }
            let class = authority
                .capability_definitions
                .iter()
                .find(|definition| definition.id() == capability)
                .and_then(CapabilityDefinition::compatibility_class);
            required.insert(
                (
                    capability.as_str().to_owned(),
                    component.execution_identity.as_str().to_owned(),
                    authority.source_policy.id.as_str().to_owned(),
                    authority.source_policy.digest.as_str().to_owned(),
                ),
                class,
            );
        }
    }
}

fn policy_required_key(policy: &InvocationPolicySnapshot) -> Option<PolicyBindingKey> {
    let source = policy.admitted_source_policy.as_ref()?;
    Some((
        policy.capability_id.as_str().to_owned(),
        policy
            .admitted_execution_identity
            .as_ref()?
            .as_str()
            .to_owned(),
        source.id.as_str().to_owned(),
        source.digest.as_str().to_owned(),
    ))
}

fn policy_sort_key(
    policy: &InvocationPolicySnapshot,
) -> (String, String, String, String, u8, String, String, bool) {
    (
        policy.capability_id.as_str().to_owned(),
        policy
            .admitted_execution_identity
            .as_ref()
            .map_or_else(String::new, |identity| identity.as_str().to_owned()),
        policy
            .admitted_source_policy
            .as_ref()
            .map_or_else(String::new, |source| source.id.as_str().to_owned()),
        policy
            .admitted_source_policy
            .as_ref()
            .map_or_else(String::new, |source| source.digest.as_str().to_owned()),
        policy.capability_class.map_or(0, capability_class_tag),
        policy.snapshot_id.as_str().to_owned(),
        policy.revision.as_str().to_owned(),
        policy.emergency_blocked,
    )
}

fn make_update_event(
    sequence: UpdateEventSequence,
    command: &UpdateCommand,
    payload: UpdateEventPayload,
) -> Result<UpdateEvent, UpdateDecisionError> {
    let post_revision = UpdateRevision::for_sequence(sequence)
        .map_err(|_| UpdateDecisionError::SequenceOverflow)?;
    let mut event = UpdateEvent {
        sequence,
        post_revision,
        command_id: command.command_id.clone(),
        update_id: command.update_id.clone(),
        payload,
        event_digest: Sha256Digest::from_bytes(&[]),
    };
    event.event_digest = digest_update_event(&event);
    Ok(event)
}
fn verify_update_event_digest(event: &UpdateEvent) -> Result<(), UpdateReplayError> {
    if digest_update_event(event) == event.event_digest {
        Ok(())
    } else {
        Err(UpdateReplayError::EvidenceMismatch)
    }
}
fn require_event_revision(
    sequence: UpdateEventSequence,
    revision: &UpdateRevision,
) -> Result<(), UpdateReplayError> {
    if &UpdateRevision::for_sequence(sequence).map_err(|_| UpdateReplayError::SequenceOverflow)?
        == revision
    {
        Ok(())
    } else {
        Err(UpdateReplayError::RevisionMismatch)
    }
}

fn digest_pin_value(pin: &InstallationPackagePin) -> Sha256Digest {
    let mut bytes = b"market-update-pin-binding/v0\0".to_vec();
    encode_pin(pin, &mut bytes);
    Sha256Digest::from_bytes(&bytes)
}
fn digest_installation_state_binding(installation: &InstallationSnapshot) -> Sha256Digest {
    let mut bytes = b"market-update-installation-state-binding/v0\0".to_vec();
    encode_string(installation.installation_id().as_str(), &mut bytes);
    encode_string(installation.revision().as_str(), &mut bytes);
    encode_string(installation.tenant_id().as_str(), &mut bytes);
    encode_string(installation.user_id().as_str(), &mut bytes);
    encode_pin(installation.package_pin(), &mut bytes);
    encode_count(
        installation.configuration_revision().get() as usize,
        &mut bytes,
    );
    encode_string(installation.configuration().digest().as_str(), &mut bytes);
    bytes.push(match installation.state() {
        ManagedInstallationState::InstalledDisabled => 1,
        ManagedInstallationState::Disabled => 2,
        ManagedInstallationState::Enabled => 3,
        ManagedInstallationState::Revoked => 4,
        ManagedInstallationState::Uninstalled => 5,
    });
    Sha256Digest::from_bytes(&bytes)
}
fn digest_policy_snapshots(policies: &[InvocationPolicySnapshot]) -> Sha256Digest {
    let mut bytes = b"market-update-policy-bindings/v0\0".to_vec();
    encode_count(policies.len(), &mut bytes);
    for policy in policies {
        encode_policy_snapshot(policy, &mut bytes);
    }
    Sha256Digest::from_bytes(&bytes)
}

fn encode_policy_snapshot(policy: &InvocationPolicySnapshot, bytes: &mut Vec<u8>) {
    encode_string(policy.capability_id.as_str(), bytes);
    match &policy.admitted_execution_identity {
        Some(exec) => {
            bytes.push(1);
            encode_string(exec.as_str(), bytes);
        }
        None => bytes.push(0),
    }
    match &policy.admitted_source_policy {
        Some(source) => {
            bytes.push(1);
            encode_string(source.id.as_str(), bytes);
            encode_string(source.digest.as_str(), bytes);
        }
        None => bytes.push(0),
    }
    match policy.capability_class {
        Some(class) => {
            bytes.push(1);
            bytes.push(capability_class_tag(class));
        }
        None => bytes.push(0),
    }
    encode_string(policy.snapshot_id.as_str(), bytes);
    encode_string(policy.revision.as_str(), bytes);
    bytes.push(u8::from(policy.emergency_blocked));
}
fn digest_update_event(event: &UpdateEvent) -> Sha256Digest {
    let mut bytes = b"market-update-event/v0\0".to_vec();
    encode_count(event.sequence.get() as usize, &mut bytes);
    encode_string(event.post_revision.as_str(), &mut bytes);
    encode_string(event.command_id.as_str(), &mut bytes);
    encode_string(event.update_id.as_str(), &mut bytes);
    bytes.push(update_event_kind_tag(event.kind()));
    encode_update_event_payload(&event.payload, &mut bytes);
    Sha256Digest::from_bytes(&bytes)
}
fn encode_update_event_payload(payload: &UpdateEventPayload, bytes: &mut Vec<u8>) {
    match payload {
        UpdateEventPayload::Staged { plan } => encode_string(plan.plan_digest().as_str(), bytes),
        UpdateEventPayload::ApprovalRecorded {
            approval,
            readiness,
        } => {
            encode_string(approval.approval_id().as_str(), bytes);
            encode_string(approval.evidence_digest().as_str(), bytes);
            encode_string(readiness.evidence_id().as_str(), bytes);
            encode_string(readiness.evidence_digest().as_str(), bytes);
        }
        UpdateEventPayload::Applied {
            prior_installation_revision,
            applied_installation_revision,
            target_pin_digest,
            installation_event,
            grant_events,
            grant_set_digest,
        } => {
            encode_string(prior_installation_revision.as_str(), bytes);
            encode_string(applied_installation_revision.as_str(), bytes);
            encode_string(target_pin_digest.as_str(), bytes);
            encode_installation_event_reference(installation_event, bytes);
            encode_grant_event_references(grant_events, bytes);
            encode_string(grant_set_digest.as_str(), bytes);
        }
        UpdateEventPayload::Confirmed { evidence } => {
            encode_string(evidence.evidence_digest().as_str(), bytes)
        }
        UpdateEventPayload::RolledBack {
            prior_installation_revision,
            rolled_back_installation_revision,
            rollback_pin_digest,
            evidence,
            installation_event,
            grant_events,
            grant_set_digest,
        } => {
            encode_string(prior_installation_revision.as_str(), bytes);
            encode_string(rolled_back_installation_revision.as_str(), bytes);
            encode_string(rollback_pin_digest.as_str(), bytes);
            encode_string(evidence.evidence_digest().as_str(), bytes);
            encode_installation_event_reference(installation_event, bytes);
            encode_grant_event_references(grant_events, bytes);
            encode_string(grant_set_digest.as_str(), bytes);
        }
        UpdateEventPayload::Cancelled {
            terminal_installation_revision,
        } => match terminal_installation_revision {
            Some(rev) => {
                bytes.push(1);
                encode_string(rev.as_str(), bytes);
            }
            None => bytes.push(0),
        },
    }
}
fn encode_installation_event_reference(
    reference: &InstallationEventReference,
    bytes: &mut Vec<u8>,
) {
    encode_string(reference.installation_id.as_str(), bytes);
    encode_count(reference.sequence.get() as usize, bytes);
    encode_string(reference.post_revision.as_str(), bytes);
    encode_string(reference.command_id.as_str(), bytes);
    bytes.push(match reference.kind {
        InstallationEventKind::PackageUpdated => 1,
        InstallationEventKind::PackageRolledBack => 2,
        _ => 0,
    });
    encode_string(reference.event_digest.as_str(), bytes);
}
fn encode_grant_event_references(references: &[GrantEventReference], bytes: &mut Vec<u8>) {
    encode_count(references.len(), bytes);
    for reference in references {
        encode_string(reference.snapshot_id.as_str(), bytes);
        encode_count(reference.sequence.get() as usize, bytes);
        encode_string(reference.post_version.as_str(), bytes);
        encode_string(reference.command_id.as_str(), bytes);
        bytes.push(match reference.kind {
            GrantEventKind::MarkedStale => 1,
            _ => 0,
        });
        encode_string(reference.event_digest.as_str(), bytes);
    }
}
const fn update_event_kind_tag(kind: UpdateEventKind) -> u8 {
    match kind {
        UpdateEventKind::Staged => 1,
        UpdateEventKind::ApprovalRecorded => 2,
        UpdateEventKind::Applied => 3,
        UpdateEventKind::Confirmed => 4,
        UpdateEventKind::RolledBack => 5,
        UpdateEventKind::Cancelled => 6,
    }
}

fn checked_prefixed(value: String, prefix: &str, max_tail: usize) -> Option<String> {
    let tail = value.strip_prefix(prefix)?;
    if !tail.is_empty()
        && tail.len() <= max_tail
        && tail
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b':' | b'-'))
    {
        Some(value)
    } else {
        None
    }
}
fn is_nonzero_decimal(value: &str) -> bool {
    let Some(first) = value.as_bytes().first() else {
        return false;
    };
    *first >= b'1' && *first <= b'9' && value.bytes().all(|byte| byte.is_ascii_digit())
}

fn validate_package_authority(
    pin: &InstallationPackagePin,
    catalog: &CatalogReadModel,
    publications: &[CatalogPackageRevision],
    registry: &CapabilityRegistry,
    target: bool,
) -> Result<PlanPackageAuthority, UpdateConstructionError> {
    if pin.catalog_revision() != catalog.catalog_revision() {
        return Err(UpdateConstructionError::AuthorityClassificationIncomplete);
    }
    let package = catalog
        .find(pin.package_id(), pin.package_version())
        .ok_or(UpdateConstructionError::TargetUnpublishedOrRevoked)?;
    if package.package_digest() != pin.package_digest()
        || package.capability_manifest_digest() != pin.capability_manifest_digest()
    {
        return Err(UpdateConstructionError::AuthorityClassificationIncomplete);
    }
    let (source_policy, publication_components) =
        validate_publications(pin, package, publications, target)?;
    let capability_definitions = validate_capabilities(package, registry, target)?;
    let component_authority_digest = digest_component_authority(&publication_components);
    let capability_authority_digest = digest_capability_authority(&capability_definitions);
    Ok(PlanPackageAuthority {
        catalog_revision: catalog.catalog_revision().clone(),
        catalog_digest: catalog.catalog_digest().clone(),
        package: package.clone(),
        source_policy,
        publication_components,
        component_authority_digest,
        registry_revision: registry.registry_revision().clone(),
        registry_digest: registry.registry_digest().clone(),
        capability_definitions,
        capability_authority_digest,
    })
}

fn validate_publications(
    pin: &InstallationPackagePin,
    package: &ValidatedPackageManifest,
    publications: &[CatalogPackageRevision],
    target: bool,
) -> Result<(SourcePolicyIdentity, Vec<CatalogComponentRevision>), UpdateConstructionError> {
    let mut seen = BTreeSet::new();
    let mut by_id = BTreeMap::new();
    let mut source_policy = None;
    for publication in publications {
        if &publication.catalog_revision != pin.catalog_revision()
            || &publication.package_id != pin.package_id()
            || &publication.package_version != pin.package_version()
            || &publication.package_digest != pin.package_digest()
            || publication.capability_manifest_digest != *pin.capability_manifest_digest()
        {
            return Err(UpdateConstructionError::AuthorityClassificationIncomplete);
        }
        if !publication.runnable {
            return Err(UpdateConstructionError::TargetUnpublishedOrRevoked);
        }
        if publication.revoked {
            return Err(UpdateConstructionError::TargetUnpublishedOrRevoked);
        }
        let Some(policy) = publication.source_policy.as_ref() else {
            return Err(UpdateConstructionError::AuthorityClassificationIncomplete);
        };
        if policy.digest != *package.source_policy_digest() {
            return Err(UpdateConstructionError::AuthorityClassificationIncomplete);
        }
        if source_policy
            .as_ref()
            .is_some_and(|seen: &SourcePolicyIdentity| seen != policy)
        {
            return Err(UpdateConstructionError::AuthorityClassificationIncomplete);
        }
        source_policy = Some(policy.clone());
        let component = publication
            .component
            .as_ref()
            .ok_or(UpdateConstructionError::TargetUnpublishedOrRevoked)?;
        if !seen.insert(component.id.clone()) {
            return Err(UpdateConstructionError::DuplicateComponentOrCapability);
        }
        by_id.insert(component.id.clone(), component.clone());
    }
    if by_id.len() != pin.components().len() || package.components().len() != pin.components().len()
    {
        return Err(UpdateConstructionError::TargetUnpublishedOrRevoked);
    }
    let mut components = Vec::with_capacity(pin.components().len());
    for pinned in pin.components() {
        let component = by_id
            .remove(pinned.component_id())
            .ok_or(UpdateConstructionError::TargetUnpublishedOrRevoked)?;
        if component.kind != pinned.kind()
            || component.version != *pinned.version()
            || component.digest != *pinned.digest()
            || component.execution_identity != *pinned.execution_identity()
        {
            return Err(UpdateConstructionError::AuthorityClassificationIncomplete);
        }
        if target {
            for capability in package.capabilities() {
                if !component.declared_capabilities.contains(capability) {
                    return Err(UpdateConstructionError::AuthorityClassificationIncomplete);
                }
            }
        }
        components.push(component);
    }
    if !by_id.is_empty() {
        return Err(UpdateConstructionError::DuplicateComponentOrCapability);
    }
    components.sort_by(|left, right| left.id.cmp(&right.id));
    let source_policy = source_policy.ok_or(UpdateConstructionError::TargetUnpublishedOrRevoked)?;
    Ok((source_policy, components))
}

fn validate_capabilities(
    package: &ValidatedPackageManifest,
    registry: &CapabilityRegistry,
    target: bool,
) -> Result<Vec<CapabilityDefinition>, UpdateConstructionError> {
    let mut seen = BTreeSet::new();
    let mut definitions = Vec::with_capacity(package.capabilities().len());
    for capability in package.capabilities() {
        if !seen.insert(capability.clone()) {
            return Err(UpdateConstructionError::DuplicateComponentOrCapability);
        }
        let Some(definition) = registry.find(capability) else {
            if target {
                return Err(UpdateConstructionError::TargetCapabilityMissingOrInactive);
            }
            continue;
        };
        if target && definition.status() != CapabilityStatus::Active {
            return Err(UpdateConstructionError::TargetCapabilityMissingOrInactive);
        }
        if target && definition.scope_kind() == ScopeKind::OperatorAdministrative {
            return Err(UpdateConstructionError::ForbiddenAdministrativeCapability);
        }
        definitions.push(definition.clone());
    }
    definitions.sort_by(|left, right| left.id().cmp(right.id()));
    Ok(definitions)
}

#[cfg(test)]
fn synthetic_publications_for_plan_authority(
    authority: &PlanPackageAuthority,
) -> Vec<CatalogPackageRevision> {
    authority
        .publication_components
        .iter()
        .cloned()
        .map(|component| CatalogPackageRevision {
            catalog_revision: authority.catalog_revision.clone(),
            package_id: authority.package.package_id().clone(),
            package_version: authority.package.package_version().clone(),
            package_digest: authority.package.package_digest().clone(),
            runnable: true,
            revoked: false,
            capability_manifest_digest: authority.package.capability_manifest_digest().clone(),
            source_policy: Some(authority.source_policy.clone()),
            component: Some(component),
        })
        .collect()
}

fn classify_update(
    old: &PlanPackageAuthority,
    new: &PlanPackageAuthority,
) -> Result<UpdateChangeClass, UpdateConstructionError> {
    let old_caps: BTreeSet<CapabilityId> = old.package.capabilities().iter().cloned().collect();
    let new_caps: BTreeSet<CapabilityId> = new.package.capabilities().iter().cloned().collect();
    let old_components = component_class_map(&old.publication_components);
    let new_components = component_class_map(&new.publication_components);

    let capability_added = new_caps.difference(&old_caps).next().is_some();
    let capability_removed = old_caps.difference(&new_caps).next().is_some();
    let component_added = new_components
        .keys()
        .any(|id| !old_components.contains_key(id));
    let component_removed = old_components
        .keys()
        .any(|id| !new_components.contains_key(id));
    let source_policy_changed = old.source_policy != new.source_policy;
    let publisher_or_tier_changed = old.package.publisher() != new.package.publisher()
        || old.package.tier() != new.package.tier();
    let component_expanded = old_components.iter().any(|(id, old_component)| {
        new_components.get(id).is_some_and(|new_component| {
            old_component.kind != new_component.kind
                || old_component.execution_identity != new_component.execution_identity
        })
    });
    let mut narrowed_policy = false;
    let mut expanded_policy = false;
    for capability in old_caps.intersection(&new_caps) {
        let old_definition = old
            .capability_definitions
            .iter()
            .find(|definition| definition.id() == capability);
        let new_definition = new
            .capability_definitions
            .iter()
            .find(|definition| definition.id() == capability);
        match compare_capability_definitions(old_definition, new_definition) {
            CapabilityPolicyChange::ExpansionRequiresReapproval => expanded_policy = true,
            CapabilityPolicyChange::Narrowed | CapabilityPolicyChange::RemovedOrRevoked => {
                narrowed_policy = true
            }
            CapabilityPolicyChange::Unchanged => {}
        }
        if old_definition.is_none() || new_definition.is_none() {
            expanded_policy = true;
        }
    }
    if capability_added
        || expanded_policy
        || source_policy_changed
        || publisher_or_tier_changed
        || component_expanded
        || component_added
    {
        return Ok(UpdateChangeClass::ReapprovalRequired);
    }
    if capability_removed || component_removed || narrowed_policy {
        return Ok(UpdateChangeClass::Narrowed);
    }
    Ok(UpdateChangeClass::Unchanged)
}

#[derive(Clone, PartialEq, Eq)]
struct ComponentClassAnchor {
    kind: ComponentKind,
    execution_identity: ExecutionIdentity,
}
fn component_class_map(
    components: &[CatalogComponentRevision],
) -> BTreeMap<ComponentId, ComponentClassAnchor> {
    components
        .iter()
        .map(|component| {
            (
                component.id.clone(),
                ComponentClassAnchor {
                    kind: component.kind,
                    execution_identity: component.execution_identity.clone(),
                },
            )
        })
        .collect()
}

fn verify_update_approval_evidence(
    value: &UpdateApprovalEvidence,
) -> Result<(), UpdateConstructionError> {
    UpdateApprovalId::parse(value.approval_id.as_str())?;
    if digest_update_approval_evidence(value) == value.evidence_digest {
        Ok(())
    } else {
        Err(UpdateConstructionError::ApprovalEvidenceIncoherent)
    }
}
fn verify_update_readiness_evidence(
    value: &UpdateReadinessEvidence,
) -> Result<(), UpdateConstructionError> {
    UpdateEvidenceId::parse(value.evidence_id.as_str())?;
    if digest_update_readiness_evidence(value) == value.evidence_digest {
        Ok(())
    } else {
        Err(UpdateConstructionError::ReadinessEvidenceIncoherent)
    }
}
fn verify_update_confirmation_evidence(
    value: &UpdateConfirmationEvidence,
) -> Result<(), UpdateConstructionError> {
    UpdateEvidenceId::parse(value.evidence_id.as_str())?;
    PackageUpdateId::parse(value.update_id.as_str())?;
    UpdateRevision::parse(value.expected_update_revision.as_str())?;
    if digest_update_confirmation_evidence(value) == value.evidence_digest {
        Ok(())
    } else {
        Err(UpdateConstructionError::ConfirmationEvidenceIncoherent)
    }
}
fn verify_rollback_readiness_evidence(
    value: &RollbackReadinessEvidence,
) -> Result<(), UpdateConstructionError> {
    UpdateEvidenceId::parse(value.evidence_id.as_str())?;
    PackageUpdateId::parse(value.update_id.as_str())?;
    UpdateRevision::parse(value.expected_update_revision.as_str())?;
    if digest_rollback_readiness_evidence(value) == value.evidence_digest {
        Ok(())
    } else {
        Err(UpdateConstructionError::RollbackEvidenceIncoherent)
    }
}

fn digest_component_authority(components: &[CatalogComponentRevision]) -> Sha256Digest {
    let mut bytes = COMPONENT_AUTHORITY_DOMAIN.to_vec();
    encode_count(components.len(), &mut bytes);
    for component in components {
        encode_component(component, &mut bytes, true);
    }
    Sha256Digest::from_bytes(&bytes)
}
fn digest_capability_authority(definitions: &[CapabilityDefinition]) -> Sha256Digest {
    let mut bytes = CAPABILITY_AUTHORITY_DOMAIN.to_vec();
    encode_count(definitions.len(), &mut bytes);
    for definition in definitions {
        encode_string(definition.id().as_str(), &mut bytes);
        encode_string(definition.definition_digest().as_str(), &mut bytes);
    }
    Sha256Digest::from_bytes(&bytes)
}
fn digest_plan(plan: &PackageUpdatePlan) -> Sha256Digest {
    let mut bytes = PLAN_DOMAIN.to_vec();
    encode_string(plan.update_id.as_str(), &mut bytes);
    encode_string(plan.tenant_id.as_str(), &mut bytes);
    encode_string(plan.user_id.as_str(), &mut bytes);
    encode_string(plan.installation_id.as_str(), &mut bytes);
    encode_string(plan.staged_installation_revision.as_str(), &mut bytes);
    encode_count(
        plan.staged_configuration_revision.get() as usize,
        &mut bytes,
    );
    encode_string(plan.staged_configuration_digest.as_str(), &mut bytes);
    encode_pin(&plan.rollback_pin, &mut bytes);
    encode_pin(&plan.target_pin, &mut bytes);
    encode_plan_authority(&plan.rollback, &mut bytes);
    encode_plan_authority(&plan.target, &mut bytes);
    bytes.push(change_class_tag(plan.change_class));
    Sha256Digest::from_bytes(&bytes)
}
fn encode_plan_authority(authority: &PlanPackageAuthority, bytes: &mut Vec<u8>) {
    encode_string(authority.catalog_revision.as_str(), bytes);
    encode_string(authority.catalog_digest.as_str(), bytes);
    encode_package(&authority.package, bytes);
    encode_string(authority.source_policy.id.as_str(), bytes);
    encode_string(authority.source_policy.digest.as_str(), bytes);
    encode_string(authority.component_authority_digest.as_str(), bytes);
    encode_string(authority.registry_revision.as_str(), bytes);
    encode_string(authority.registry_digest.as_str(), bytes);
    encode_string(authority.capability_authority_digest.as_str(), bytes);
}
fn digest_update_approval_evidence(value: &UpdateApprovalEvidence) -> Sha256Digest {
    let mut bytes = APPROVAL_EVIDENCE_DOMAIN.to_vec();
    encode_string(value.approval_id.as_str(), &mut bytes);
    encode_string(value.plan_digest.as_str(), &mut bytes);
    bytes.push(change_class_tag(value.change_class));
    encode_string(value.staged_installation_revision.as_str(), &mut bytes);
    encode_string(value.staged_configuration_digest.as_str(), &mut bytes);
    encode_string(value.approval_evidence_digest.as_str(), &mut bytes);
    Sha256Digest::from_bytes(&bytes)
}
fn digest_update_readiness_evidence(value: &UpdateReadinessEvidence) -> Sha256Digest {
    let mut bytes = READINESS_EVIDENCE_DOMAIN.to_vec();
    encode_string(value.evidence_id.as_str(), &mut bytes);
    encode_string(value.plan_digest.as_str(), &mut bytes);
    encode_string(value.target_package_digest.as_str(), &mut bytes);
    encode_string(value.rollback_package_digest.as_str(), &mut bytes);
    encode_string(value.target_component_authority_digest.as_str(), &mut bytes);
    encode_string(
        value.rollback_component_authority_digest.as_str(),
        &mut bytes,
    );
    encode_string(value.staged_installation_revision.as_str(), &mut bytes);
    encode_string(value.staged_configuration_digest.as_str(), &mut bytes);
    encode_string(
        value.verified_target_artifact_set_digest.as_str(),
        &mut bytes,
    );
    encode_string(
        value.verified_rollback_artifact_set_digest.as_str(),
        &mut bytes,
    );
    encode_string(
        value
            .target_configuration_admission_snapshot_digest
            .as_str(),
        &mut bytes,
    );
    encode_string(
        value
            .target_source_execution_policy_admission_snapshot_digest
            .as_str(),
        &mut bytes,
    );
    encode_string(value.target_catalog_revision.as_str(), &mut bytes);
    encode_string(value.rollback_catalog_revision.as_str(), &mut bytes);
    encode_string(value.target_registry_revision.as_str(), &mut bytes);
    encode_string(value.rollback_registry_revision.as_str(), &mut bytes);
    Sha256Digest::from_bytes(&bytes)
}
fn digest_update_confirmation_evidence(value: &UpdateConfirmationEvidence) -> Sha256Digest {
    let mut bytes = CONFIRMATION_EVIDENCE_DOMAIN.to_vec();
    encode_string(value.evidence_id.as_str(), &mut bytes);
    encode_string(value.update_id.as_str(), &mut bytes);
    encode_string(value.expected_update_revision.as_str(), &mut bytes);
    encode_string(value.applied_event_digest.as_str(), &mut bytes);
    encode_string(value.installation_id.as_str(), &mut bytes);
    encode_string(value.installation_revision.as_str(), &mut bytes);
    encode_string(value.target_pin_digest.as_str(), &mut bytes);
    encode_string(value.installation_state_digest.as_str(), &mut bytes);
    Sha256Digest::from_bytes(&bytes)
}
fn digest_rollback_readiness_evidence(value: &RollbackReadinessEvidence) -> Sha256Digest {
    let mut bytes = ROLLBACK_EVIDENCE_DOMAIN.to_vec();
    encode_string(value.evidence_id.as_str(), &mut bytes);
    encode_string(value.update_id.as_str(), &mut bytes);
    encode_string(value.expected_update_revision.as_str(), &mut bytes);
    encode_string(value.rollback_pin_digest.as_str(), &mut bytes);
    encode_string(
        value.current_target_installation_revision.as_str(),
        &mut bytes,
    );
    encode_count(
        value.current_configuration_revision.get() as usize,
        &mut bytes,
    );
    encode_string(value.current_configuration_digest.as_str(), &mut bytes);
    encode_string(
        value.verified_rollback_artifact_set_digest.as_str(),
        &mut bytes,
    );
    encode_string(
        value.rollback_admission_snapshot_digest.as_str(),
        &mut bytes,
    );
    Sha256Digest::from_bytes(&bytes)
}

fn encode_pin(pin: &InstallationPackagePin, bytes: &mut Vec<u8>) {
    encode_string(pin.catalog_revision().as_str(), bytes);
    encode_string(pin.package_id().as_str(), bytes);
    let version = pin.package_version().as_str();
    encode_string(&version, bytes);
    encode_string(pin.package_digest().as_str(), bytes);
    encode_count(pin.components().len(), bytes);
    for component in pin.components() {
        encode_string(component.component_id().as_str(), bytes);
        bytes.push(component_kind_tag(component.kind()));
        encode_string(component.version().as_str(), bytes);
        encode_string(component.digest().as_str(), bytes);
        encode_string(component.execution_identity().as_str(), bytes);
    }
    encode_string(pin.component_set_digest().as_str(), bytes);
    encode_string(pin.capability_manifest_digest().as_str(), bytes);
}
fn encode_package(package: &ValidatedPackageManifest, bytes: &mut Vec<u8>) {
    encode_string(package.package_id().as_str(), bytes);
    let version = package.package_version().as_str();
    encode_string(&version, bytes);
    encode_string(package.publisher(), bytes);
    bytes.push(package_tier_tag(package.tier()));
    encode_string(package.package_digest().as_str(), bytes);
    encode_string(package.component_declaration_set_digest().as_str(), bytes);
    encode_string(package.capability_manifest_digest().as_str(), bytes);
    encode_string(package.source_policy_digest().as_str(), bytes);
}
fn encode_component(
    component: &CatalogComponentRevision,
    bytes: &mut Vec<u8>,
    include_artifact: bool,
) {
    encode_string(component.id.as_str(), bytes);
    bytes.push(component_kind_tag(component.kind));
    encode_string(component.version.as_str(), bytes);
    if include_artifact {
        encode_string(component.digest.as_str(), bytes);
    }
    encode_string(component.execution_identity.as_str(), bytes);
    encode_count(component.declared_capabilities.len(), bytes);
    for capability in &component.declared_capabilities {
        encode_string(capability.as_str(), bytes);
    }
}
fn encode_count(count: usize, output: &mut Vec<u8>) {
    output.extend_from_slice(&(count as u64).to_be_bytes());
}
fn encode_string(value: &str, output: &mut Vec<u8>) {
    encode_count(value.len(), output);
    output.extend_from_slice(value.as_bytes());
}
const fn package_tier_tag(value: PackageTier) -> u8 {
    match value {
        PackageTier::FirstParty => 1,
        PackageTier::VerifiedCommunityText => 2,
        PackageTier::VerifiedRemoteMcp => 3,
    }
}
const fn component_kind_tag(value: ComponentKind) -> u8 {
    match value {
        ComponentKind::SkillComponent => 1,
        ComponentKind::DeclarativeResourcePack => 2,
        ComponentKind::McpServerComponent => 3,
        ComponentKind::NativeRustComponent => 4,
    }
}
const fn capability_class_tag(value: CapabilityClass) -> u8 {
    match value {
        CapabilityClass::PublicRead => 1,
        CapabilityClass::PublicLinkout => 2,
        CapabilityClass::TenantPrivateRead => 3,
        CapabilityClass::TenantPrivateWrite => 4,
    }
}
const fn change_class_tag(value: UpdateChangeClass) -> u8 {
    match value {
        UpdateChangeClass::Unchanged => 1,
        UpdateChangeClass::Narrowed => 2,
        UpdateChangeClass::ReapprovalRequired => 3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::invocation::{
        ComponentVersion, ConfirmationPolicy, PolicyRevision, PolicySnapshotId, SourcePolicyId,
        SourcePolicyIdentity,
    };
    use crate::market::grant::{
        GrantAdmissionEvidence, GrantApprovalId, GrantRepository, GrantScope,
        InMemoryGrantRepository,
    };
    use crate::market::installation::{
        ConfigurationKey, EnablePreconditionEvidence, InstallationCommand, InstallationCommandId,
        InstallationConfiguration, InstalledComponentPin, NonSecretText,
        decide as installation_decide, evolve as installation_evolve,
    };
    use crate::market::load_package_manifest;

    macro_rules! parsed {
        ($kind:ty, $value:expr) => {{
            match <$kind>::parse($value) {
                Ok(value) => value,
                Err(error) => panic!("fixture value must parse: {error}"),
            }
        }};
    }

    fn digest(ch: char) -> Sha256Digest {
        parsed!(
            Sha256Digest,
            format!("sha256:{}", ch.to_string().repeat(64))
        )
    }
    fn cap_public() -> &'static str {
        "campus.public_rules.read"
    }
    fn cap_private() -> &'static str {
        "user.own_profile.read"
    }
    fn manifest(
        version: &str,
        publisher: &str,
        tier: &str,
        caps: &[&str],
        source_value: &str,
    ) -> ValidatedPackageManifest {
        let caps_json = caps
            .iter()
            .map(|cap| format!("\"{cap}\""))
            .collect::<Vec<_>>()
            .join(",");
        let source = format!(
            r#"{{"id":"synthetic.update","version":"{version}","publisher":"{publisher}","tier":"{tier}","displayName":"Synthetic","implementationStatus":"implemented","installPolicy":{{"class":"UserInstalledPlugin","defaultInstalled":false,"defaultEnabled":false,"userDisableAllowed":true}},"components":[{{"type":"NativeRustComponent","path":"bin/main","mode":"local"}}],"capabilities":[{caps_json}],"sourcePolicy":{{"reviewed":"{source_value}"}}}}"#
        )
        .replace('\\', "");
        load_package_manifest(source.as_bytes()).expect("manifest fixture validates")
    }
    fn manifest_two_components(
        version: &str,
        publisher: &str,
        tier: &str,
        caps: &[&str],
        source_value: &str,
    ) -> ValidatedPackageManifest {
        let caps_json = caps
            .iter()
            .map(|cap| format!("\"{cap}\""))
            .collect::<Vec<_>>()
            .join(",");
        let source = format!(
            r#"{{"id":"synthetic.update","version":"{version}","publisher":"{publisher}","tier":"{tier}","displayName":"Synthetic","implementationStatus":"implemented","installPolicy":{{"class":"UserInstalledPlugin","defaultInstalled":false,"defaultEnabled":false,"userDisableAllowed":true}},"components":[{{"type":"NativeRustComponent","path":"bin/main","mode":"local"}},{{"type":"NativeRustComponent","path":"bin/extra","mode":"local"}}],"capabilities":[{caps_json}],"sourcePolicy":{{"reviewed":"{source_value}"}}}}"#
        )
        .replace('\\', "");
        load_package_manifest(source.as_bytes()).expect("two-component manifest fixture validates")
    }
    fn registry(rev: &str, include_private: bool, private_ask: bool) -> CapabilityRegistry {
        let private_confirmation = if private_ask { "Ask" } else { "Allow" };
        let private_auto = if private_ask {
            "Never"
        } else {
            "FirstPartyDefaultOnly"
        };
        let private_scope = if private_ask {
            "TenantPrivateUser"
        } else {
            "CampusPublic"
        };
        let private_data = if private_ask {
            "UserProfile"
        } else {
            "PublicCampusFact"
        };
        let extra = if include_private {
            format!(
                r#",{{"id":"{}","effectClass":"Read","dataClass":"{private_data}","scopeKind":"{private_scope}","autoGrant":"{private_auto}","confirmationDefault":"{private_confirmation}","status":"Active"}}"#,
                cap_private()
            )
            .replace('\\', "")
        } else {
            String::new()
        };
        let source = format!(
            r#"{{"schemaVersion":"capability-registry/v1","registryRevision":"capability-registry:{rev}","capabilities":[{{"id":"{}","effectClass":"Read","dataClass":"PublicCampusFact","scopeKind":"CampusPublic","autoGrant":"FirstPartyDefaultOnly","confirmationDefault":"Allow","status":"Active"}}{extra}]}}"#,
            cap_public()
        )
        .replace('\\', "");
        crate::market::capability::load_capability_registry(source.as_bytes())
            .expect("registry fixture validates")
    }
    fn component(
        id: &str,
        version: &str,
        artifact: char,
        exec: &str,
        caps: &[&str],
    ) -> CatalogComponentRevision {
        CatalogComponentRevision {
            id: parsed!(ComponentId, id),
            kind: ComponentKind::NativeRustComponent,
            version: parsed!(ComponentVersion, version),
            digest: digest(artifact),
            execution_identity: parsed!(ExecutionIdentity, exec),
            declared_capabilities: caps.iter().map(|cap| parsed!(CapabilityId, *cap)).collect(),
            tool: None,
        }
    }
    fn pin(
        catalog: &str,
        package: &ValidatedPackageManifest,
        comp: CatalogComponentRevision,
    ) -> InstallationPackagePin {
        let installed = InstalledComponentPin::new(
            comp.id.clone(),
            comp.kind,
            comp.version.clone(),
            comp.digest.clone(),
            comp.execution_identity.clone(),
        )
        .expect("pin");
        InstallationPackagePin::new(
            parsed!(CatalogRevision, catalog),
            package.package_id().clone(),
            package.package_version().clone(),
            package.package_digest().clone(),
            vec![installed],
            digest('c'),
            package.capability_manifest_digest().clone(),
        )
        .expect("package pin")
    }
    fn publication(
        catalog: &str,
        package: &ValidatedPackageManifest,
        comp: CatalogComponentRevision,
    ) -> CatalogPackageRevision {
        CatalogPackageRevision {
            catalog_revision: parsed!(CatalogRevision, catalog),
            package_id: package.package_id().clone(),
            package_version: package.package_version().clone(),
            package_digest: package.package_digest().clone(),
            runnable: true,
            revoked: false,
            capability_manifest_digest: package.capability_manifest_digest().clone(),
            source_policy: Some(SourcePolicyIdentity {
                id: parsed!(SourcePolicyId, "source-policy:fixture"),
                digest: package.source_policy_digest().clone(),
            }),
            component: Some(comp),
        }
    }
    fn catalog(rev: &str, packages: Vec<ValidatedPackageManifest>) -> CatalogReadModel {
        CatalogReadModel::new(parsed!(CatalogRevision, rev), packages).expect("catalog")
    }
    fn installation(
        package: &ValidatedPackageManifest,
        rollback_component: CatalogComponentRevision,
    ) -> InstallationSnapshot {
        let tenant = parsed!(TenantId, "tenant:update-unit");
        let config = InstallationConfiguration::new(
            &tenant,
            vec![(
                ConfigurationKey::parse("mode").expect("fixture operation must succeed"),
                crate::market::installation::ConfigurationValue::Text(
                    NonSecretText::parse("safe").expect("fixture operation must succeed"),
                ),
            )],
        )
        .expect("fixture operation must succeed");
        let pin = pin("catalog:old", package, rollback_component);
        let command = InstallationCommand::install(
            parsed!(InstallationCommandId, "cmd:install-update-unit"),
            parsed!(InstallationId, "installation:update-unit"),
            tenant,
            parsed!(UserId, "user:update-unit"),
            pin,
            config,
        )
        .expect("fixture operation must succeed");
        let event = crate::market::installation::decide(None, &command)
            .expect("fixture operation must succeed");
        crate::market::installation::evolve(None, &event).expect("fixture operation must succeed")
    }
    fn plan_with(
        old: ValidatedPackageManifest,
        new: ValidatedPackageManifest,
        old_comp: CatalogComponentRevision,
        new_comp: CatalogComponentRevision,
        old_reg: CapabilityRegistry,
        new_reg: CapabilityRegistry,
    ) -> PackageUpdatePlan {
        let installation = installation(&old, old_comp.clone());
        let target_pin = pin("catalog:new", &new, new_comp.clone());
        match UpdateCommand::stage(
            parsed!(UpdateCommandId, "update-cmd:stage"),
            parsed!(PackageUpdateId, "update:unit"),
            &installation,
            target_pin,
            &catalog("catalog:old", vec![old.clone()]),
            &[publication("catalog:old", &old, old_comp)],
            &catalog("catalog:new", vec![new.clone()]),
            &[publication("catalog:new", &new, new_comp)],
            &old_reg,
            &new_reg,
        )
        .expect("fixture operation must succeed")
        .action
        {
            UpdateCommandAction::Stage { plan } => plan,
            _ => unreachable!(),
        }
    }

    fn stage_command_fixture(
        id: &str,
    ) -> (
        UpdateCommand,
        InstallationSnapshot,
        CatalogReadModel,
        Vec<CatalogPackageRevision>,
        CatalogReadModel,
        Vec<CatalogPackageRevision>,
        CapabilityRegistry,
        CapabilityRegistry,
    ) {
        let old = manifest(
            "1.0.0",
            "Nous",
            "VerifiedRemoteMcp",
            &[cap_public()],
            "same",
        );
        let new = manifest(
            "1.1.0",
            "Nous",
            "VerifiedRemoteMcp",
            &[cap_public()],
            "same",
        );
        let old_comp = component(
            "component:main",
            "component-version:1",
            '1',
            "exec:main",
            &[cap_public()],
        );
        let new_comp = component(
            "component:main",
            "component-version:2",
            '2',
            "exec:main",
            &[cap_public()],
        );
        let installation = installation(&old, old_comp.clone());
        let old_catalog = catalog("catalog:old", vec![old.clone()]);
        let new_catalog = catalog("catalog:new", vec![new.clone()]);
        let old_publications = vec![publication("catalog:old", &old, old_comp)];
        let new_publications = vec![publication("catalog:new", &new, new_comp.clone())];
        let old_registry = registry("old", false, true);
        let new_registry = registry("new", false, true);
        let cmd = UpdateCommand::stage(
            parsed!(UpdateCommandId, format!("update-cmd:stage-{id}")),
            parsed!(PackageUpdateId, format!("update:{id}")),
            &installation,
            pin("catalog:new", &new, new_comp),
            &old_catalog,
            &old_publications,
            &new_catalog,
            &new_publications,
            &old_registry,
            &new_registry,
        )
        .expect("fixture operation must succeed");
        (
            cmd,
            installation,
            old_catalog,
            old_publications,
            new_catalog,
            new_publications,
            old_registry,
            new_registry,
        )
    }
    fn stage_event_and_aggregate(
        id: &str,
    ) -> (
        UpdateEvent,
        PackageUpdateAggregate,
        UpdateCommand,
        InstallationSnapshot,
        CatalogReadModel,
        CatalogReadModel,
        CapabilityRegistry,
        CapabilityRegistry,
    ) {
        let (
            cmd,
            installation,
            old_catalog,
            _old_pubs,
            new_catalog,
            _new_pubs,
            old_registry,
            new_registry,
        ) = stage_command_fixture(id);
        let policies = match &cmd.action {
            UpdateCommandAction::Stage { plan } => policy_snapshots_for_plan(plan),
            _ => unreachable!(),
        };
        let ctx = match &cmd.action {
            UpdateCommandAction::Stage { plan } => UpdateDecisionContext {
                kind: UpdateDecisionContextKind::Stage {
                    installation: installation.clone(),
                    authority: AuthorityCarrierBinding::from_parts(
                        &new_catalog,
                        &old_catalog,
                        &new_registry,
                        &old_registry,
                        &policies,
                        plan,
                    )
                    .expect("fixture operation must succeed"),
                    plan: plan.clone(),
                },
            },
            _ => unreachable!(),
        };
        let event = decide(None, &ctx, &cmd).expect("fixture operation must succeed");
        let agg = evolve(None, &event).expect("fixture operation must succeed");
        (
            event,
            agg,
            cmd,
            installation,
            old_catalog,
            new_catalog,
            old_registry,
            new_registry,
        )
    }
    fn policy_snapshots_for_plan(plan: &PackageUpdatePlan) -> Vec<InvocationPolicySnapshot> {
        required_policy_bindings(plan)
            .into_iter()
            .enumerate()
            .map(
                |(idx, ((capability, execution, source_id, source_digest), class))| {
                    InvocationPolicySnapshot {
                        snapshot_id: parsed!(
                            PolicySnapshotId,
                            format!("policy-snapshot:update-{idx}")
                        ),
                        revision: parsed!(PolicyRevision, format!("policy-revision:update-{idx}")),
                        capability_id: parsed!(CapabilityId, capability),
                        capability_class: class,
                        admitted_execution_identity: Some(parsed!(ExecutionIdentity, execution)),
                        admitted_source_policy: Some(SourcePolicyIdentity {
                            id: parsed!(SourcePolicyId, source_id),
                            digest: parsed!(Sha256Digest, source_digest),
                        }),
                        emergency_blocked: false,
                    }
                },
            )
            .collect()
    }
    fn ready_aggregate(
        id: &str,
    ) -> (
        Vec<UpdateEvent>,
        PackageUpdateAggregate,
        InstallationSnapshot,
        CatalogReadModel,
        CatalogReadModel,
        CapabilityRegistry,
        CapabilityRegistry,
    ) {
        let (
            stage_event,
            staged,
            _cmd,
            installation,
            old_catalog,
            new_catalog,
            old_registry,
            new_registry,
        ) = stage_event_and_aggregate(id);
        let policies = policy_snapshots_for_plan(staged.plan());
        let approval = UpdateApprovalEvidence::from_plan(
            parsed!(UpdateApprovalId, format!("update-approval:{id}")),
            staged.plan(),
            digest('a'),
        )
        .expect("fixture operation must succeed");
        let readiness = UpdateReadinessEvidence::from_plan(
            parsed!(UpdateEvidenceId, format!("update-evidence:ready-{id}")),
            staged.plan(),
            digest('b'),
            digest('c'),
            digest('d'),
            digest_policy_snapshots(&policy_snapshots_for_plan(staged.plan())),
        )
        .expect("fixture operation must succeed");
        let cmd = UpdateCommand::record_approval(
            parsed!(UpdateCommandId, format!("update-cmd:approve-{id}")),
            staged.update_id().clone(),
            staged.revision().clone(),
            approval,
            readiness,
        )
        .expect("fixture operation must succeed");
        let ctx = UpdateDecisionContext::for_record_approval_for_test(
            &installation,
            staged.plan(),
            &new_catalog,
            &old_catalog,
            &new_registry,
            &old_registry,
            &policies,
        )
        .expect("fixture operation must succeed");
        let event = decide(Some(&staged), &ctx, &cmd).expect("fixture operation must succeed");
        let ready = evolve(Some(staged), &event).expect("fixture operation must succeed");
        (
            vec![stage_event, event],
            ready,
            installation,
            old_catalog,
            new_catalog,
            old_registry,
            new_registry,
        )
    }
    fn disabled_installation(installation: InstallationSnapshot) -> InstallationSnapshot {
        let evidence = EnablePreconditionEvidence::from_authority_bindings(
            installation.installation_id().clone(),
            installation.revision().clone(),
            installation.package_pin().package_digest().clone(),
            installation.package_pin().component_set_digest().clone(),
            installation.configuration().digest().clone(),
            installation
                .package_pin()
                .capability_manifest_digest()
                .clone(),
            digest('7'),
            digest('8'),
        )
        .expect("fixture operation must succeed");
        let enable = InstallationCommand::enable(
            parsed!(InstallationCommandId, "cmd:enable-update-unit"),
            installation.installation_id().clone(),
            installation.revision().clone(),
            evidence,
        )
        .expect("fixture operation must succeed");
        let enabled_event = installation_decide(Some(&installation), &enable)
            .expect("fixture operation must succeed");
        let enabled = installation_evolve(Some(installation), &enabled_event)
            .expect("fixture operation must succeed");
        let disable = InstallationCommand::disable(
            parsed!(InstallationCommandId, "cmd:disable-update-unit"),
            enabled.installation_id().clone(),
            enabled.revision().clone(),
        )
        .expect("fixture operation must succeed");
        let disabled_event =
            installation_decide(Some(&enabled), &disable).expect("fixture operation must succeed");
        installation_evolve(Some(enabled), &disabled_event).expect("fixture operation must succeed")
    }

    #[test]
    fn checked_update_ids_revisions_and_sequences_are_canonical() {
        assert_eq!(
            PackageUpdateId::parse("update:a.b_c-1:2")
                .expect("fixture operation must succeed")
                .as_str(),
            "update:a.b_c-1:2"
        );
        assert!(PackageUpdateId::parse("update:").is_err());
        assert!(UpdateCommandId::parse("update-cmd:bad space").is_err());
        assert!(UpdateApprovalId::parse("update-approval:x").is_ok());
        assert!(UpdateEvidenceId::parse("update-evidence:x").is_ok());
        let seq = UpdateEventSequence::new(1).expect("fixture operation must succeed");
        assert_eq!(seq.next().expect("fixture operation must succeed").get(), 2);
        assert!(UpdateEventSequence::new(0).is_err());
        assert_eq!(
            UpdateRevision::for_sequence(seq)
                .expect("fixture operation must succeed")
                .as_str(),
            "update-revision:1"
        );
        assert!(UpdateRevision::parse("update-revision:0").is_err());
        assert!(UpdateRevision::parse("update-revision:01").is_err());
    }

    #[test]
    fn update_plan_binds_exact_source_target_catalog_registry_policy_and_digests() {
        let old = manifest(
            "1.0.0",
            "Nous",
            "VerifiedRemoteMcp",
            &[cap_public()],
            "same",
        );
        let new = manifest(
            "1.1.0",
            "Nous",
            "VerifiedRemoteMcp",
            &[cap_public()],
            "same",
        );
        let old_comp = component(
            "component:main",
            "component-version:1",
            '1',
            "exec:main",
            &[cap_public()],
        );
        let new_comp = component(
            "component:main",
            "component-version:2",
            '2',
            "exec:main",
            &[cap_public()],
        );
        let old_reg = registry("old", false, true);
        let new_reg = registry("new", false, true);
        let plan = plan_with(
            old.clone(),
            new.clone(),
            old_comp,
            new_comp,
            old_reg,
            new_reg,
        );
        assert_eq!(plan.update_id().as_str(), "update:unit");
        assert_eq!(plan.tenant_id().as_str(), "tenant:update-unit");
        assert_eq!(plan.installation_id().as_str(), "installation:update-unit");
        assert_eq!(
            plan.rollback_package().package_digest(),
            old.package_digest()
        );
        assert_eq!(plan.target_package().package_digest(), new.package_digest());
        assert_eq!(plan.rollback_catalog_revision().as_str(), "catalog:old");
        assert_eq!(plan.target_catalog_revision().as_str(), "catalog:new");
        assert_eq!(
            plan.rollback_registry_revision().as_str(),
            "capability-registry:old"
        );
        assert_eq!(
            plan.target_registry_revision().as_str(),
            "capability-registry:new"
        );
        assert_eq!(plan.rollback_component_declarations().len(), 1);
        assert_eq!(plan.target_capability_definitions().len(), 1);
        assert_eq!(plan.change_class(), UpdateChangeClass::Unchanged);
        assert_ne!(
            plan.plan_digest().as_str(),
            Sha256Digest::from_bytes(&[]).as_str()
        );
    }

    #[test]
    fn change_classifier_is_computed_from_complete_typed_authority_not_caller_hint() {
        let old = manifest(
            "1.0.0",
            "Nous",
            "VerifiedRemoteMcp",
            &[cap_public()],
            "same",
        );
        let new = manifest(
            "1.1.0",
            "Nous",
            "VerifiedRemoteMcp",
            &[cap_public(), cap_private()],
            "same",
        );
        let old_comp = component(
            "component:main",
            "component-version:1",
            '1',
            "exec:main",
            &[cap_public()],
        );
        let new_comp = component(
            "component:main",
            "component-version:2",
            '2',
            "exec:main",
            &[cap_public(), cap_private()],
        );
        let plan = plan_with(
            old,
            new,
            old_comp,
            new_comp,
            registry("old", false, true),
            registry("new", true, true),
        );
        assert_eq!(plan.change_class(), UpdateChangeClass::ReapprovalRequired);
    }

    #[test]
    fn change_classifier_precedence_covers_added_expanded_source_policy_tier_component_scope_and_unknown()
     {
        let base_old = manifest(
            "1.0.0",
            "Nous",
            "VerifiedRemoteMcp",
            &[cap_public()],
            "same",
        );
        let old_comp = component(
            "component:main",
            "component-version:1",
            '1',
            "exec:main",
            &[cap_public()],
        );
        let changed_source = manifest(
            "1.1.0",
            "Nous",
            "VerifiedRemoteMcp",
            &[cap_public()],
            "changed",
        );
        let plan = plan_with(
            base_old.clone(),
            changed_source,
            old_comp.clone(),
            component(
                "component:main",
                "component-version:2",
                '2',
                "exec:main",
                &[cap_public()],
            ),
            registry("old1", false, true),
            registry("new1", false, true),
        );
        assert_eq!(plan.change_class(), UpdateChangeClass::ReapprovalRequired);
        let narrowed = manifest("1.1.0", "Nous", "VerifiedRemoteMcp", &[], "same");
        assert!(UpdateCommand::stage(parsed!(UpdateCommandId, "update-cmd:narrow"), parsed!(PackageUpdateId, "update:narrow"), &installation(&base_old, old_comp.clone()), pin("catalog:new", &narrowed, component("component:main", "component-version:2", '2', "exec:main", &[])), &catalog("catalog:old", vec![base_old.clone()]), &[publication("catalog:old", &base_old, old_comp.clone())], &catalog("catalog:new", vec![narrowed.clone()]), &[publication("catalog:new", &narrowed, component("component:main", "component-version:2", '2', "exec:main", &[]))], &registry("old2", false, true), &registry("new2", false, true)).is_ok_and(|cmd| matches!(cmd.action, UpdateCommandAction::Stage { plan } if plan.change_class() == UpdateChangeClass::Narrowed)));
        let target_with_added_component = manifest_two_components(
            "1.1.0",
            "Nous",
            "VerifiedRemoteMcp",
            &[cap_public()],
            "same",
        );
        let target_main = component(
            "component:main",
            "component-version:2",
            '2',
            "exec:main",
            &[cap_public()],
        );
        let target_extra = component(
            "component:extra",
            "component-version:1",
            '3',
            "exec:extra",
            &[cap_public()],
        );
        let target_pin = InstallationPackagePin::new(
            parsed!(CatalogRevision, "catalog:new"),
            target_with_added_component.package_id().clone(),
            target_with_added_component.package_version().clone(),
            target_with_added_component.package_digest().clone(),
            vec![
                InstalledComponentPin::new(
                    target_main.id.clone(),
                    target_main.kind,
                    target_main.version.clone(),
                    target_main.digest.clone(),
                    target_main.execution_identity.clone(),
                )
                .expect("main target pin validates"),
                InstalledComponentPin::new(
                    target_extra.id.clone(),
                    target_extra.kind,
                    target_extra.version.clone(),
                    target_extra.digest.clone(),
                    target_extra.execution_identity.clone(),
                )
                .expect("extra target pin validates"),
            ],
            digest('d'),
            target_with_added_component
                .capability_manifest_digest()
                .clone(),
        )
        .expect("two-component target pin validates");
        let added_component = UpdateCommand::stage(
            parsed!(UpdateCommandId, "update-cmd:added-component"),
            parsed!(PackageUpdateId, "update:added-component"),
            &installation(&base_old, old_comp.clone()),
            target_pin,
            &catalog("catalog:old", vec![base_old.clone()]),
            &[publication("catalog:old", &base_old, old_comp.clone())],
            &catalog("catalog:new", vec![target_with_added_component.clone()]),
            &[
                publication("catalog:new", &target_with_added_component, target_main),
                publication("catalog:new", &target_with_added_component, target_extra),
            ],
            &registry("old3", false, true),
            &registry("new3", false, true),
        )
        .expect("coherent component addition plan validates");
        assert!(matches!(
            added_component.action,
            UpdateCommandAction::Stage { plan }
                if plan.change_class() == UpdateChangeClass::ReapprovalRequired
        ));
    }

    #[test]
    fn evidence_values_bind_plan_digest_revision_configuration_and_authority_without_leaking_payloads()
     {
        let old = manifest(
            "1.0.0",
            "Nous",
            "VerifiedRemoteMcp",
            &[cap_public()],
            "same",
        );
        let new = manifest(
            "1.1.0",
            "Nous",
            "VerifiedRemoteMcp",
            &[cap_public()],
            "same",
        );
        let plan = plan_with(
            old,
            new,
            component(
                "component:main",
                "component-version:1",
                '1',
                "exec:main",
                &[cap_public()],
            ),
            component(
                "component:main",
                "component-version:2",
                '2',
                "exec:main",
                &[cap_public()],
            ),
            registry("old", false, true),
            registry("new", false, true),
        );
        let approval = UpdateApprovalEvidence::from_plan(
            parsed!(UpdateApprovalId, "update-approval:unit"),
            &plan,
            digest('a'),
        )
        .expect("fixture operation must succeed");
        let readiness = UpdateReadinessEvidence::from_plan(
            parsed!(UpdateEvidenceId, "update-evidence:ready"),
            &plan,
            digest('b'),
            digest('c'),
            digest('d'),
            digest_policy_snapshots(&policy_snapshots_for_plan(&plan)),
        )
        .expect("fixture operation must succeed");
        assert_eq!(approval.plan_digest(), plan.plan_digest());
        assert_eq!(approval.change_class(), plan.change_class());
        assert_eq!(
            approval.staged_installation_revision(),
            plan.staged_installation_revision()
        );
        assert_eq!(
            readiness.target_package_digest(),
            plan.target_pin().package_digest()
        );
        assert_eq!(
            readiness.rollback_catalog_revision(),
            plan.rollback_catalog_revision()
        );
        let cmd = UpdateCommand::record_approval(
            parsed!(UpdateCommandId, "update-cmd:approval"),
            parsed!(PackageUpdateId, "update:unit"),
            UpdateRevision::parse("update-revision:1").expect("fixture operation must succeed"),
            approval.clone(),
            readiness.clone(),
        )
        .expect("fixture operation must succeed");
        assert_eq!(cmd.update_id().as_str(), "update:unit");
        let rendered = format!("{approval:?} {readiness:?} {cmd:?} {plan:?}");
        assert!(rendered.contains("<redacted>"));
        for forbidden in [
            "tenant:update-unit",
            "user:update-unit",
            "installation:update-unit",
            "update:unit",
            "update-revision:1",
            "sha256:",
            "synthetic.update",
        ] {
            assert!(
                !rendered.contains(forbidden),
                "debug leaked {forbidden}: {rendered}"
            );
        }
        assert!(!rendered.contains("safe"));
        assert!(!rendered.contains("approval evidence raw"));
        assert_ne!(approval.evidence_digest(), readiness.evidence_digest());
    }

    #[test]
    fn state_machine_allows_only_stage_approval_apply_confirm_rollback_cancel_paths() {
        let (
            _events,
            ready,
            _installation,
            _old_catalog,
            _new_catalog,
            _old_registry,
            _new_registry,
        ) = ready_aggregate("smoke");
        assert_eq!(ready.state(), UpdateState::Ready);
        let (
            cmd2,
            installation2,
            old_catalog2,
            old_pubs2,
            new_catalog2,
            new_pubs2,
            old_registry2,
            new_registry2,
        ) = stage_command_fixture("for-stage-used");
        let (target_pin2, policies2) = match &cmd2.action {
            UpdateCommandAction::Stage { plan } => {
                (plan.target_pin().clone(), policy_snapshots_for_plan(plan))
            }
            _ => unreachable!(),
        };
        let _stage_ctx = UpdateDecisionContext::for_stage(
            &cmd2,
            &installation2,
            target_pin2,
            &old_catalog2,
            &old_pubs2,
            &new_catalog2,
            &new_pubs2,
            &old_registry2,
            &new_registry2,
            &policies2,
        )
        .expect("fixture operation must succeed");
        let cancel = UpdateCommand::cancel(
            parsed!(UpdateCommandId, "update-cmd:cancel-smoke"),
            ready.update_id().clone(),
            ready.revision().clone(),
        )
        .expect("fixture operation must succeed");
        let event = decide(Some(&ready), &UpdateDecisionContext::for_cancel(), &cancel)
            .expect("fixture operation must succeed");
        assert_eq!(event.kind(), UpdateEventKind::Cancelled);
        assert_eq!(
            evolve(Some(ready), &event)
                .expect("fixture operation must succeed")
                .state(),
            UpdateState::Cancelled
        );
    }

    #[test]
    fn stage_requires_absence_one_slot_nonterminal_installation_and_distinct_reviewed_target() {
        let (
            _event,
            agg,
            cmd,
            _installation,
            _old_catalog,
            _new_catalog,
            _old_registry,
            _new_registry,
        ) = stage_event_and_aggregate("stage-rules");
        let ctx = match &cmd.action {
            UpdateCommandAction::Stage { plan } => UpdateDecisionContext {
                kind: UpdateDecisionContextKind::Stage {
                    installation: _installation.clone(),
                    authority: AuthorityCarrierBinding::from_parts(
                        &_new_catalog,
                        &_old_catalog,
                        &_new_registry,
                        &_old_registry,
                        &policy_snapshots_for_plan(plan),
                        plan,
                    )
                    .expect("fixture operation must succeed"),
                    plan: plan.clone(),
                },
            },
            _ => unreachable!(),
        };
        assert_eq!(
            decide(Some(&agg), &ctx, &cmd).expect_err("fixture operation must fail"),
            UpdateDecisionError::AggregateAlreadyPresent
        );
        assert!(matches!(cmd.action, UpdateCommandAction::Stage { .. }));
    }

    #[test]
    fn record_approval_requires_coherent_fresh_approval_and_readiness_evidence() {
        let (
            _event,
            staged,
            _cmd,
            _installation,
            old_catalog,
            new_catalog,
            old_registry,
            new_registry,
        ) = stage_event_and_aggregate("approval-rules");
        let approval = UpdateApprovalEvidence::from_plan(
            parsed!(UpdateApprovalId, "update-approval:approval-rules"),
            staged.plan(),
            digest('a'),
        )
        .expect("fixture operation must succeed");
        let readiness = UpdateReadinessEvidence::from_plan(
            parsed!(UpdateEvidenceId, "update-evidence:approval-rules"),
            staged.plan(),
            digest('b'),
            digest('c'),
            digest('d'),
            digest_policy_snapshots(&policy_snapshots_for_plan(staged.plan())),
        )
        .expect("fixture operation must succeed");
        let cmd = UpdateCommand::record_approval(
            parsed!(UpdateCommandId, "update-cmd:approval-rules"),
            staged.update_id().clone(),
            staged.revision().clone(),
            approval,
            readiness,
        )
        .expect("fixture operation must succeed");
        let wrong_authority = UpdateDecisionContext::for_record_approval_for_test(
            &_installation,
            staged.plan(),
            &old_catalog,
            &old_catalog,
            &new_registry,
            &old_registry,
            &policy_snapshots_for_plan(staged.plan()),
        )
        .expect("fixture operation must succeed");
        assert_eq!(
            decide(Some(&staged), &wrong_authority, &cmd).expect_err("fixture operation must fail"),
            UpdateDecisionError::CatalogAuthorityChanged
        );
        let drifted_policy = InvocationPolicySnapshot {
            snapshot_id: parsed!(PolicySnapshotId, "policy-snapshot:approval-drift"),
            revision: parsed!(PolicyRevision, "policy-revision:approval-drift"),
            capability_id: parsed!(CapabilityId, cap_public()),
            capability_class: None,
            admitted_execution_identity: None,
            admitted_source_policy: None,
            emergency_blocked: true,
        };
        let policy_drift = UpdateDecisionContext::for_record_approval_for_test(
            &_installation,
            staged.plan(),
            &new_catalog,
            &old_catalog,
            &new_registry,
            &old_registry,
            &[drifted_policy],
        )
        .expect_err("fixture operation must fail");
        assert_eq!(policy_drift, UpdateDecisionError::CatalogAuthorityChanged);
        let installation_drift = UpdateDecisionContext::for_record_approval_for_test(
            &disabled_installation(_installation.clone()),
            staged.plan(),
            &new_catalog,
            &old_catalog,
            &new_registry,
            &old_registry,
            &policy_snapshots_for_plan(staged.plan()),
        )
        .expect("fixture operation must succeed");
        assert_eq!(
            decide(Some(&staged), &installation_drift, &cmd)
                .expect_err("fixture operation must fail"),
            UpdateDecisionError::InstallationRevisionMismatch
        );
        let ctx = UpdateDecisionContext::for_record_approval_for_test(
            &_installation,
            staged.plan(),
            &new_catalog,
            &old_catalog,
            &new_registry,
            &old_registry,
            &policy_snapshots_for_plan(staged.plan()),
        )
        .expect("fixture operation must succeed");
        assert_eq!(
            decide(Some(&staged), &ctx, &cmd)
                .expect("fixture operation must succeed")
                .kind(),
            UpdateEventKind::ApprovalRecorded
        );
    }

    #[test]
    fn apply_requires_ready_disabled_exact_revisions_pins_configuration_and_authority_recheck() {
        let (_events, ready, installation, old_catalog, new_catalog, old_registry, new_registry) =
            ready_aggregate("apply-rules");
        let installation = disabled_installation(installation);
        let install_cmd = InstallationCommand::package_updated(
            parsed!(InstallationCommandId, "cmd:update-apply-rules"),
            installation.installation_id().clone(),
            installation.revision().clone(),
            ready.plan().plan_digest().clone(),
            ready.plan().target_pin().clone(),
        )
        .expect("fixture operation must succeed");
        let install_event = installation_decide(Some(&installation), &install_cmd)
            .expect("fixture operation must succeed");
        let grants = InMemoryGrantRepository::new()
            .load_current_for_installation(
                installation.tenant_id(),
                installation.user_id(),
                installation.installation_id(),
                installation.revision(),
            )
            .expect("fixture operation must succeed");
        let ctx = UpdateDecisionContext::for_apply_for_test(
            &installation,
            ready.plan(),
            &new_catalog,
            &old_catalog,
            &new_registry,
            &old_registry,
            &policy_snapshots_for_plan(ready.plan()),
            grants,
            &install_event,
            &[],
        )
        .expect("fixture operation must succeed");
        let cmd = UpdateCommand::apply(
            parsed!(UpdateCommandId, "update-cmd:apply-rules"),
            ready.update_id().clone(),
            ready.revision().clone(),
            installation.revision().clone(),
        )
        .expect("fixture operation must succeed");
        let event = decide(Some(&ready), &ctx, &cmd).expect("fixture operation must succeed");
        assert_eq!(event.kind(), UpdateEventKind::Applied);
        assert_eq!(
            evolve(Some(ready), &event)
                .expect("fixture operation must succeed")
                .state(),
            UpdateState::AppliedPendingConfirmation
        );
    }

    #[test]
    fn rollback_requires_applied_pending_disabled_fresh_readiness_and_current_target_pin() {
        let (_events, ready, installation, old_catalog, new_catalog, old_registry, new_registry) =
            ready_aggregate("rollback-rules");
        let installation = disabled_installation(installation);
        let install_cmd = InstallationCommand::package_updated(
            parsed!(InstallationCommandId, "cmd:update-rollback-rules"),
            installation.installation_id().clone(),
            installation.revision().clone(),
            ready.plan().plan_digest().clone(),
            ready.plan().target_pin().clone(),
        )
        .expect("fixture operation must succeed");
        let install_event = installation_decide(Some(&installation), &install_cmd)
            .expect("fixture operation must succeed");
        let applied_installation = installation_evolve(Some(installation.clone()), &install_event)
            .expect("fixture operation must succeed");
        let grants = InMemoryGrantRepository::new()
            .load_current_for_installation(
                installation.tenant_id(),
                installation.user_id(),
                installation.installation_id(),
                installation.revision(),
            )
            .expect("fixture operation must succeed");
        let apply_ctx = UpdateDecisionContext::for_apply_for_test(
            &installation,
            ready.plan(),
            &new_catalog,
            &old_catalog,
            &new_registry,
            &old_registry,
            &policy_snapshots_for_plan(ready.plan()),
            grants,
            &install_event,
            &[],
        )
        .expect("fixture operation must succeed");
        let apply_cmd = UpdateCommand::apply(
            parsed!(UpdateCommandId, "update-cmd:apply-rollback-rules"),
            ready.update_id().clone(),
            ready.revision().clone(),
            installation.revision().clone(),
        )
        .expect("fixture operation must succeed");
        let applied_event =
            decide(Some(&ready), &apply_ctx, &apply_cmd).expect("fixture operation must succeed");
        let applied = evolve(Some(ready), &applied_event).expect("fixture operation must succeed");
        let mut forged_cancel = make_update_event(
            applied
                .last_sequence()
                .next()
                .expect("fixture operation must succeed"),
            &UpdateCommand::cancel(
                parsed!(UpdateCommandId, "update-cmd:forged-cancel-none-after-apply"),
                applied.update_id().clone(),
                applied.revision().clone(),
            )
            .expect("fixture operation must succeed"),
            UpdateEventPayload::Cancelled {
                terminal_installation_revision: None,
            },
        )
        .expect("fixture operation must succeed");
        forged_cancel.event_digest = digest_update_event(&forged_cancel);
        assert_eq!(
            evolve(Some(applied.clone()), &forged_cancel).expect_err("fixture operation must fail"),
            UpdateReplayError::IllegalTransition
        );
        let rollback_evidence = RollbackReadinessEvidence::from_bindings(
            parsed!(UpdateEvidenceId, "update-evidence:rollback-rules"),
            applied.update_id().clone(),
            applied.revision().clone(),
            digest_pin_value(applied.plan().rollback_pin()),
            applied_installation.revision().clone(),
            applied_installation.configuration_revision(),
            applied_installation.configuration().digest().clone(),
            digest('f'),
            digest_policy_snapshots(&policy_snapshots_for_plan(applied.plan())),
        )
        .expect("fixture operation must succeed");
        let rb_cmd = InstallationCommand::package_rolled_back(
            parsed!(InstallationCommandId, "cmd:rolledback-rules"),
            applied_installation.installation_id().clone(),
            applied_installation.revision().clone(),
            applied.plan().plan_digest().clone(),
            applied.plan().rollback_pin().clone(),
        )
        .expect("fixture operation must succeed");
        let rb_install_event = installation_decide(Some(&applied_installation), &rb_cmd)
            .expect("fixture operation must succeed");
        let rb_grants = InMemoryGrantRepository::new()
            .load_current_for_installation(
                applied_installation.tenant_id(),
                applied_installation.user_id(),
                applied_installation.installation_id(),
                applied_installation.revision(),
            )
            .expect("fixture operation must succeed");
        let rb_ctx = UpdateDecisionContext::for_rollback_for_test(
            &applied_installation,
            applied.plan(),
            &new_catalog,
            &old_catalog,
            &new_registry,
            &old_registry,
            &policy_snapshots_for_plan(applied.plan()),
            rb_grants,
            &rb_install_event,
            &[],
        )
        .expect("fixture operation must succeed");
        let cmd = UpdateCommand::rollback(
            parsed!(UpdateCommandId, "update-cmd:rollback-rules"),
            applied.update_id().clone(),
            applied.revision().clone(),
            applied_installation.revision().clone(),
            rollback_evidence,
        )
        .expect("fixture operation must succeed");
        assert_eq!(
            decide(Some(&applied), &rb_ctx, &cmd)
                .expect("fixture operation must succeed")
                .kind(),
            UpdateEventKind::RolledBack
        );
    }

    #[test]
    fn confirm_only_closes_applied_pending_without_runtime_health_or_grant_authority() {
        let (_events, ready, installation, old_catalog, new_catalog, old_registry, new_registry) =
            ready_aggregate("confirm-rules");
        let installation = disabled_installation(installation);
        let install_cmd = InstallationCommand::package_updated(
            parsed!(InstallationCommandId, "cmd:update-confirm-rules"),
            installation.installation_id().clone(),
            installation.revision().clone(),
            ready.plan().plan_digest().clone(),
            ready.plan().target_pin().clone(),
        )
        .expect("fixture operation must succeed");
        let install_event = installation_decide(Some(&installation), &install_cmd)
            .expect("fixture operation must succeed");
        let applied_installation = installation_evolve(Some(installation.clone()), &install_event)
            .expect("fixture operation must succeed");
        let grants = InMemoryGrantRepository::new()
            .load_current_for_installation(
                installation.tenant_id(),
                installation.user_id(),
                installation.installation_id(),
                installation.revision(),
            )
            .expect("fixture operation must succeed");
        let apply_ctx = UpdateDecisionContext::for_apply_for_test(
            &installation,
            ready.plan(),
            &new_catalog,
            &old_catalog,
            &new_registry,
            &old_registry,
            &policy_snapshots_for_plan(ready.plan()),
            grants,
            &install_event,
            &[],
        )
        .expect("fixture operation must succeed");
        let apply_cmd = UpdateCommand::apply(
            parsed!(UpdateCommandId, "update-cmd:apply-confirm-rules"),
            ready.update_id().clone(),
            ready.revision().clone(),
            installation.revision().clone(),
        )
        .expect("fixture operation must succeed");
        let applied_event =
            decide(Some(&ready), &apply_ctx, &apply_cmd).expect("fixture operation must succeed");
        let applied = evolve(Some(ready), &applied_event).expect("fixture operation must succeed");
        let evidence = UpdateConfirmationEvidence::from_bindings(
            parsed!(UpdateEvidenceId, "update-evidence:confirm-rules"),
            applied.update_id().clone(),
            applied.revision().clone(),
            applied_event.event_digest().clone(),
            applied_installation.installation_id().clone(),
            applied_installation.revision().clone(),
            digest_pin_value(applied.plan().target_pin()),
            digest_installation_state_binding(&applied_installation),
        )
        .expect("fixture operation must succeed");
        let wrong_evidence = UpdateConfirmationEvidence::from_bindings(
            parsed!(
                UpdateEvidenceId,
                "update-evidence:confirm-wrong-applied-event"
            ),
            applied.update_id().clone(),
            applied.revision().clone(),
            digest('9'),
            applied_installation.installation_id().clone(),
            applied_installation.revision().clone(),
            digest_pin_value(applied.plan().target_pin()),
            digest_installation_state_binding(&applied_installation),
        )
        .expect("fixture operation must succeed");
        let wrong_cmd = UpdateCommand::confirm_applied_update(
            parsed!(UpdateCommandId, "update-cmd:confirm-wrong-applied-event"),
            applied.update_id().clone(),
            applied.revision().clone(),
            applied_installation.revision().clone(),
            wrong_evidence,
        )
        .expect("fixture operation must succeed");
        assert_eq!(
            decide(
                Some(&applied),
                &UpdateDecisionContext::for_confirm_applied(&applied_installation),
                &wrong_cmd,
            )
            .expect_err("fixture operation must fail"),
            UpdateDecisionError::ConfirmationEvidenceMismatch
        );
        let cmd = UpdateCommand::confirm_applied_update(
            parsed!(UpdateCommandId, "update-cmd:confirm-rules"),
            applied.update_id().clone(),
            applied.revision().clone(),
            applied_installation.revision().clone(),
            evidence,
        )
        .expect("fixture operation must succeed");
        let event = decide(
            Some(&applied),
            &UpdateDecisionContext::for_confirm_applied(&applied_installation),
            &cmd,
        )
        .expect("fixture operation must succeed");
        assert_eq!(
            evolve(Some(applied), &event)
                .expect("fixture operation must succeed")
                .state(),
            UpdateState::Confirmed
        );
    }

    #[test]
    fn cancel_and_terminal_installation_reconciliation_have_exact_precedence() {
        let (
            _events,
            ready,
            _installation,
            _old_catalog,
            _new_catalog,
            _old_registry,
            _new_registry,
        ) = ready_aggregate("cancel-rules");
        let _terminal_ctx =
            UpdateDecisionContext::for_cancel_after_terminal_installation(&_installation);
        let cmd = UpdateCommand::cancel(
            parsed!(UpdateCommandId, "update-cmd:cancel-rules"),
            ready.update_id().clone(),
            ready.revision().clone(),
        )
        .expect("fixture operation must succeed");
        assert_eq!(
            decide(Some(&ready), &UpdateDecisionContext::for_cancel(), &cmd)
                .expect("fixture operation must succeed")
                .kind(),
            UpdateEventKind::Cancelled
        );
    }

    #[test]
    fn event_sequences_revisions_command_ids_and_approval_consumption_are_exact() {
        let (
            events,
            ready,
            _installation,
            _old_catalog,
            _new_catalog,
            _old_registry,
            _new_registry,
        ) = ready_aggregate("sequence-rules");
        assert_eq!(events[0].sequence().get(), 1);
        assert_eq!(events[1].sequence().get(), 2);
        assert_eq!(ready.revision().as_str(), "update-revision:2");
        assert_eq!(
            replay(events.iter())
                .expect("fixture operation must succeed")
                .expect("fixture operation must succeed")
                .state(),
            UpdateState::Ready
        );
    }

    #[test]
    fn replay_accepts_reachable_histories_and_rejects_gap_duplicate_reorder_overflow_and_post_terminal()
     {
        let (
            events,
            ready,
            _installation,
            _old_catalog,
            _new_catalog,
            _old_registry,
            _new_registry,
        ) = ready_aggregate("replay-rules");
        assert_eq!(
            replay(events.iter())
                .expect("fixture operation must succeed")
                .expect("fixture operation must succeed")
                .state(),
            UpdateState::Ready
        );
        assert_eq!(
            replay([&events[1], &events[0]]).expect_err("fixture operation must fail"),
            UpdateReplayError::NonStagedInitialEvent
        );
        let cancel = UpdateCommand::cancel(
            parsed!(UpdateCommandId, "update-cmd:cancel-replay-rules"),
            ready.update_id().clone(),
            ready.revision().clone(),
        )
        .expect("fixture operation must succeed");
        let cancel_event = decide(Some(&ready), &UpdateDecisionContext::for_cancel(), &cancel)
            .expect("fixture operation must succeed");
        let terminal = evolve(Some(ready), &cancel_event).expect("fixture operation must succeed");
        assert_eq!(
            evolve(Some(terminal), &cancel_event).expect_err("fixture operation must fail"),
            UpdateReplayError::PostTerminalEvent
        );
    }

    #[test]
    fn replay_rejects_forged_plan_change_class_evidence_revision_and_identity_bindings() {
        let (
            events,
            _ready,
            _installation,
            _old_catalog,
            _new_catalog,
            _old_registry,
            _new_registry,
        ) = ready_aggregate("forgery-rules");
        let mut forged = events[0].clone();
        forged.post_revision =
            UpdateRevision::parse("update-revision:3").expect("fixture operation must succeed");
        forged.event_digest = digest_update_event(&forged);
        assert_eq!(
            evolve(None, &forged).expect_err("fixture operation must fail"),
            UpdateReplayError::RevisionMismatch
        );
        let mut wrong_update_id = events[0].clone();
        if let UpdateEventPayload::Staged { plan } = &mut wrong_update_id.payload {
            plan.update_id = parsed!(PackageUpdateId, "update:forged-plan-id");
            plan.plan_digest = digest_plan(plan);
        }
        wrong_update_id.event_digest = digest_update_event(&wrong_update_id);
        assert_eq!(
            evolve(None, &wrong_update_id).expect_err("fixture operation must fail"),
            UpdateReplayError::PlanMismatch
        );
        let mut wrong_plan_digest = events[0].clone();
        if let UpdateEventPayload::Staged { plan } = &mut wrong_plan_digest.payload {
            plan.plan_digest = digest('9');
        }
        wrong_plan_digest.event_digest = digest_update_event(&wrong_plan_digest);
        assert_eq!(
            evolve(None, &wrong_plan_digest).expect_err("fixture operation must fail"),
            UpdateReplayError::PlanMismatch
        );
        let mut wrong_evidence = events[1].clone();
        if let UpdateEventPayload::ApprovalRecorded { approval, .. } = &mut wrong_evidence.payload {
            approval.evidence_digest = digest('8');
        }
        wrong_evidence.event_digest = digest_update_event(&wrong_evidence);
        assert_eq!(
            replay([&events[0], &wrong_evidence]).expect_err("fixture operation must fail"),
            UpdateReplayError::EvidenceMismatch
        );
    }

    fn seeded_update_repository(
        id: &str,
    ) -> (
        InMemoryPackageUpdateRepository,
        UpdateCommand,
        InstallationSnapshot,
        InstallationCommandReceipt,
    ) {
        let (
            cmd,
            installation,
            old_catalog,
            old_publications,
            new_catalog,
            new_publications,
            old_registry,
            new_registry,
        ) = stage_command_fixture(id);
        let install = InstallationCommand::install(
            parsed!(InstallationCommandId, "cmd:install-update-unit"),
            installation.installation_id().clone(),
            installation.tenant_id().clone(),
            installation.user_id().clone(),
            installation.package_pin().clone(),
            installation.configuration().clone(),
        )
        .expect("fixture install command validates");
        let mut installation_repository = InMemoryInstallationRepository::new();
        let install_receipt = installation_repository
            .execute(install)
            .expect("fixture install commits");
        let plan = match &cmd.action {
            UpdateCommandAction::Stage { plan } => plan,
            _ => unreachable!(),
        };
        let mut repository = InMemoryPackageUpdateRepository::new();
        repository.catalog_read_models = vec![old_catalog, new_catalog];
        repository.catalog_publications = old_publications
            .into_iter()
            .chain(new_publications)
            .collect();
        repository.capability_registries = vec![old_registry, new_registry];
        repository.policy_snapshots = policy_snapshots_for_plan(plan);
        repository.installation_repository = installation_repository;
        (repository, cmd, installation, install_receipt)
    }

    fn approval_command_for(id: &str, staged: &PackageUpdateSnapshot) -> UpdateCommand {
        let approval = UpdateApprovalEvidence::from_plan(
            parsed!(UpdateApprovalId, format!("update-approval:{id}")),
            staged.plan(),
            digest('a'),
        )
        .expect("fixture approval validates");
        let readiness = UpdateReadinessEvidence::from_plan(
            parsed!(UpdateEvidenceId, format!("update-evidence:ready-{id}")),
            staged.plan(),
            digest('b'),
            digest('c'),
            digest('d'),
            digest_policy_snapshots(&policy_snapshots_for_plan(staged.plan())),
        )
        .expect("fixture readiness validates");
        UpdateCommand::record_approval(
            parsed!(UpdateCommandId, format!("update-cmd:approve-{id}")),
            staged.update_id().clone(),
            staged.revision().clone(),
            approval,
            readiness,
        )
        .expect("fixture approval command validates")
    }

    fn staged_and_ready_repository(
        id: &str,
    ) -> (
        InMemoryPackageUpdateRepository,
        PackageUpdateSnapshot,
        Vec<UpdateCommandReceipt>,
        InstallationSnapshot,
        InstallationCommandReceipt,
    ) {
        let (mut repository, stage, installation, install_receipt) = seeded_update_repository(id);
        let stage_receipt = repository.execute(stage).expect("stage accepted");
        let staged = match stage_receipt.outcome() {
            UpdateCommandOutcome::Accepted { snapshot, .. } => snapshot.clone(),
            UpdateCommandOutcome::Rejected { error } => panic!("stage rejected: {error:?}"),
        };
        let approve = approval_command_for(id, &staged);
        let approve_receipt = repository.execute(approve).expect("approval accepted");
        let ready = match approve_receipt.outcome() {
            UpdateCommandOutcome::Accepted { snapshot, .. } => snapshot.clone(),
            UpdateCommandOutcome::Rejected { error } => panic!("approval rejected: {error:?}"),
        };
        (
            repository,
            ready,
            vec![stage_receipt, approve_receipt],
            installation,
            install_receipt,
        )
    }

    #[test]
    fn subordinate_installation_and_grant_event_references_are_complete_kind_checked_and_digest_bound()
     {
        let (_events, ready, installation, old_catalog, new_catalog, old_registry, new_registry) =
            ready_aggregate("refs-rules");
        let installation = disabled_installation(installation);
        let install_cmd = InstallationCommand::package_updated(
            parsed!(InstallationCommandId, "cmd:update-refs-rules"),
            installation.installation_id().clone(),
            installation.revision().clone(),
            ready.plan().plan_digest().clone(),
            ready.plan().target_pin().clone(),
        )
        .expect("fixture operation must succeed");
        let install_event = installation_decide(Some(&installation), &install_cmd)
            .expect("fixture operation must succeed");
        let reference = InstallationEventReference::from_event(
            installation.installation_id(),
            &install_event,
            InstallationEventKind::PackageUpdated,
        )
        .expect("fixture operation must succeed");
        assert!(reference.matches_event(installation.installation_id(), &install_event));
        assert_eq!(
            InstallationEventReference::from_event(
                installation.installation_id(),
                &install_event,
                InstallationEventKind::PackageRolledBack
            )
            .expect_err("fixture operation must fail"),
            UpdateDecisionError::CoupledInstallationEventMismatch
        );
        let grants = InMemoryGrantRepository::new()
            .load_current_for_installation(
                installation.tenant_id(),
                installation.user_id(),
                installation.installation_id(),
                installation.revision(),
            )
            .expect("empty complete grant set loads");
        let policies = policy_snapshots_for_plan(ready.plan());
        let context = UpdateDecisionContext::for_apply_for_test(
            &installation,
            ready.plan(),
            &new_catalog,
            &old_catalog,
            &new_registry,
            &old_registry,
            &policies,
            grants,
            &install_event,
            &[],
        )
        .expect("apply context validates");
        let command = UpdateCommand::apply(
            parsed!(UpdateCommandId, "update-cmd:apply-ref-forgery"),
            ready.update_id().clone(),
            ready.revision().clone(),
            installation.revision().clone(),
        )
        .expect("apply command validates");
        let mut forged = decide(Some(&ready), &context, &command).expect("apply decision succeeds");
        if let UpdateEventPayload::Applied {
            installation_event, ..
        } = &mut forged.payload
        {
            installation_event.post_revision = installation.revision().clone();
        } else {
            unreachable!();
        }
        forged.event_digest = digest_update_event(&forged);
        assert_eq!(
            evolve(Some(ready), &forged).expect_err("forged subordinate revision must fail"),
            UpdateReplayError::SubordinateReferenceMismatch
        );
    }

    fn disable_repository_installation(
        repository: &mut InMemoryPackageUpdateRepository,
        installation: &InstallationSnapshot,
        id: &str,
    ) -> InstallationSnapshot {
        disable_repository_installation_with_receipts(repository, installation, id).0
    }

    fn disable_repository_installation_with_receipts(
        repository: &mut InMemoryPackageUpdateRepository,
        installation: &InstallationSnapshot,
        id: &str,
    ) -> (
        InstallationSnapshot,
        Vec<(InstallationCommandReceipt, Option<InstallationSnapshot>)>,
    ) {
        let evidence = EnablePreconditionEvidence::from_authority_bindings(
            installation.installation_id().clone(),
            installation.revision().clone(),
            installation.package_pin().package_digest().clone(),
            installation.package_pin().component_set_digest().clone(),
            installation.configuration().digest().clone(),
            installation
                .package_pin()
                .capability_manifest_digest()
                .clone(),
            digest('7'),
            digest('8'),
        )
        .expect("enable evidence validates");
        let enable = InstallationCommand::enable(
            parsed!(InstallationCommandId, format!("cmd:enable-update-{id}")),
            installation.installation_id().clone(),
            installation.revision().clone(),
            evidence,
        )
        .expect("enable command validates");
        let enable_receipt = repository
            .installation_repository
            .execute(enable)
            .expect("enable commits");
        let enabled = repository
            .installation_repository
            .load_exact(installation.installation_id())
            .expect("installation loads")
            .expect("installation exists");
        let disable = InstallationCommand::disable(
            parsed!(InstallationCommandId, format!("cmd:disable-update-{id}")),
            installation.installation_id().clone(),
            enabled.revision().clone(),
        )
        .expect("disable command validates");
        let disable_receipt = repository
            .installation_repository
            .execute(disable)
            .expect("disable commits");
        let disabled = repository
            .installation_repository
            .load_exact(installation.installation_id())
            .expect("installation loads")
            .expect("installation exists");
        (
            disabled,
            vec![
                (enable_receipt, Some(installation.clone())),
                (disable_receipt, Some(enabled)),
            ],
        )
    }

    fn update_histories_for(
        repository: &InMemoryPackageUpdateRepository,
        update_id: &PackageUpdateId,
    ) -> Vec<(PackageUpdateId, Vec<UpdateEvent>)> {
        vec![(
            update_id.clone(),
            repository
                .event_history(update_id)
                .expect("update history loads"),
        )]
    }

    fn installation_histories_for(
        repository: &InMemoryPackageUpdateRepository,
        installation_id: &InstallationId,
    ) -> Vec<(InstallationId, Vec<InstallationEvent>)> {
        vec![(
            installation_id.clone(),
            repository
                .installation_repository
                .event_history(installation_id)
                .expect("installation history loads"),
        )]
    }

    fn assert_rejected(receipt: &UpdateCommandReceipt, expected: UpdateDecisionError) {
        assert_eq!(
            receipt.outcome(),
            &UpdateCommandOutcome::Rejected { error: expected }
        );
    }

    fn issue_active_grant_for_installation(
        repository: &mut InMemoryPackageUpdateRepository,
        installation: &InstallationSnapshot,
        package: &ValidatedPackageManifest,
        registry_index: usize,
        id: &str,
    ) -> (GrantCommandReceipt, GrantSnapshot) {
        let evidence = GrantAdmissionEvidence::from_authority_bindings(
            parsed!(GrantSnapshotId, format!("grant:update-{id}")),
            parsed!(GrantApprovalId, format!("grant-approval:update-{id}")),
            installation,
            package,
            parsed!(CapabilityId, cap_public()),
            GrantScope::campus_public().expect("grant scope validates"),
            ConfirmationPolicy::Allow,
            &repository.capability_registries[registry_index],
        )
        .expect("grant admission evidence validates");
        let command = GrantCommand::issue(
            parsed!(GrantCommandId, format!("grant-cmd:update-issue-{id}")),
            evidence,
        )
        .expect("grant issue command validates");
        let receipt = repository
            .grant_repository
            .execute(command)
            .expect("active grant commits");
        let snapshot = match receipt.outcome() {
            GrantCommandOutcome::Accepted { snapshot, .. } => snapshot.clone(),
            GrantCommandOutcome::Rejected { error } => panic!("grant rejected: {error:?}"),
        };
        assert_eq!(snapshot.state(), GrantState::Active);
        (receipt, snapshot)
    }

    fn approval_command_reusing_id(
        command_id: &str,
        approval_id: &UpdateApprovalId,
        staged: &PackageUpdateSnapshot,
    ) -> UpdateCommand {
        let approval =
            UpdateApprovalEvidence::from_plan(approval_id.clone(), staged.plan(), digest('a'))
                .expect("reused approval evidence validates");
        let readiness = UpdateReadinessEvidence::from_plan(
            parsed!(UpdateEvidenceId, format!("update-evidence:{command_id}")),
            staged.plan(),
            digest('b'),
            digest('c'),
            digest('d'),
            digest_policy_snapshots(&policy_snapshots_for_plan(staged.plan())),
        )
        .expect("readiness validates");
        UpdateCommand::record_approval(
            parsed!(UpdateCommandId, format!("update-cmd:{command_id}")),
            staged.update_id().clone(),
            staged.revision().clone(),
            approval,
            readiness,
        )
        .expect("approval command validates")
    }

    #[test]
    fn repository_idempotency_command_conflict_approval_conflict_and_failure_injection_are_atomic()
    {
        let (mut repository, stage, installation, _install_receipt) =
            seeded_update_repository("repo-atomic");
        let initial_install_history = repository
            .installation_repository
            .event_history(installation.installation_id())
            .expect("installation history loads");
        let initial_grant_current = repository
            .grant_repository
            .load_current_for_installation(
                installation.tenant_id(),
                installation.user_id(),
                installation.installation_id(),
                installation.revision(),
            )
            .expect("grant current loads");

        let missing_apply = UpdateCommand::apply(
            parsed!(UpdateCommandId, "update-cmd:missing-aggregate-apply"),
            parsed!(PackageUpdateId, "update:missing-aggregate"),
            parsed!(UpdateRevision, "update-revision:1"),
            installation.revision().clone(),
        )
        .expect("missing aggregate apply command validates");
        let missing_receipt = repository
            .execute(missing_apply.clone())
            .expect("missing aggregate rejection persists");
        assert_rejected(&missing_receipt, UpdateDecisionError::AggregateMissing);
        assert_eq!(
            repository
                .execute(missing_apply)
                .expect("missing aggregate retry exact"),
            missing_receipt
        );
        assert!(matches!(
            missing_receipt.witness,
            UpdateReceiptWitness::MissingAggregate
        ));

        repository.fail_next_commit_for_test();
        assert_eq!(
            repository
                .execute(stage.clone())
                .expect_err("failure injects before rejected/accepted receipt"),
            UpdateRepositoryError::InjectedPersistenceFailure
        );
        assert_eq!(
            repository.command_ledger.len(),
            1,
            "only missing receipt persisted"
        );
        assert_eq!(
            repository
                .event_history(stage.update_id())
                .expect("history loads"),
            Vec::<UpdateEvent>::new()
        );
        assert_eq!(
            repository.load_exact(stage.update_id()).expect("load"),
            None
        );
        assert_eq!(
            repository
                .installation_repository
                .event_history(installation.installation_id())
                .expect("installation history loads"),
            initial_install_history
        );
        assert_eq!(
            repository
                .grant_repository
                .load_current_for_installation(
                    installation.tenant_id(),
                    installation.user_id(),
                    installation.installation_id(),
                    installation.revision(),
                )
                .expect("grant current loads"),
            initial_grant_current
        );

        let stage_receipt = repository
            .execute(stage.clone())
            .expect("retry accepts same stage once");
        assert_eq!(
            repository
                .event_history(stage.update_id())
                .expect("history")
                .len(),
            1
        );
        assert_eq!(
            repository.execute(stage.clone()).expect("idempotent retry"),
            stage_receipt
        );
        assert_eq!(
            repository
                .event_history(stage.update_id())
                .expect("history")
                .len(),
            1
        );
        let staged = stage_receipt.outcome().accepted_snapshot_for_test();

        let conflicting_stage = UpdateCommand::stage(
            stage.command_id().clone(),
            parsed!(PackageUpdateId, "update:repo-atomic-conflict"),
            &installation,
            staged.plan().target_pin().clone(),
            &repository.catalog_read_models[0],
            &repository.catalog_publications[0..1],
            &repository.catalog_read_models[1],
            &repository.catalog_publications[1..2],
            &repository.capability_registries[0],
            &repository.capability_registries[1],
        )
        .expect("conflicting stage command validates");
        assert_eq!(
            repository
                .execute(conflicting_stage)
                .expect_err("ledger detects unequal command"),
            UpdateRepositoryError::CommandConflict
        );

        let approve = approval_command_for("repo-atomic", &staged);
        let approved = repository
            .execute(approve.clone())
            .expect("approval accepted");
        let ready = approved.outcome().accepted_snapshot_for_test();
        let approval_id = match &approve.action {
            UpdateCommandAction::RecordApproval { approval, .. } => approval.approval_id().clone(),
            _ => unreachable!(),
        };

        let (_, other_stage, _, _) = seeded_update_repository("repo-atomic-other");
        let other_stage_receipt = repository
            .execute(other_stage)
            .expect("active slot conflict persists rejected receipt");
        assert_rejected(
            &other_stage_receipt,
            UpdateDecisionError::ActiveUpdateConflict,
        );
        match &other_stage_receipt.witness {
            UpdateReceiptWitness::ActiveSlotConflict {
                installation_id,
                conflicting_update_id,
                conflicting_state,
            } => {
                assert_eq!(installation_id, installation.installation_id());
                assert_eq!(conflicting_update_id, ready.update_id());
                assert_eq!(*conflicting_state, UpdateState::Ready);
            }
            _ => panic!("active-slot rejection must carry active slot witness"),
        }
        assert_eq!(
            repository
                .execute(other_stage_receipt.command().clone())
                .expect("active conflict retry exact"),
            other_stage_receipt
        );

        let mut enabled_repository = repository.clone();
        let enable_evidence = EnablePreconditionEvidence::from_authority_bindings(
            installation.installation_id().clone(),
            installation.revision().clone(),
            installation.package_pin().package_digest().clone(),
            installation.package_pin().component_set_digest().clone(),
            installation.configuration().digest().clone(),
            installation
                .package_pin()
                .capability_manifest_digest()
                .clone(),
            digest('7'),
            digest('8'),
        )
        .expect("enabled preflight evidence validates");
        let enable = InstallationCommand::enable(
            parsed!(
                InstallationCommandId,
                "cmd:enable-update-repo-atomic-preflight"
            ),
            installation.installation_id().clone(),
            installation.revision().clone(),
            enable_evidence,
        )
        .expect("enable command validates");
        enabled_repository
            .installation_repository
            .execute(enable)
            .expect("enabled preflight owner transition commits");
        let enabled = enabled_repository
            .installation_repository
            .load_exact(installation.installation_id())
            .expect("enabled installation loads")
            .expect("enabled installation exists");
        let enabled_owner_history = enabled_repository
            .installation_repository
            .event_history(installation.installation_id())
            .expect("enabled owner history loads");
        let enabled_apply = UpdateCommand::apply(
            parsed!(UpdateCommandId, "update-cmd:apply-enabled-repo-atomic"),
            ready.update_id().clone(),
            ready.revision().clone(),
            enabled.revision().clone(),
        )
        .expect("enabled apply command validates");
        let enabled_receipt = enabled_repository
            .execute(enabled_apply.clone())
            .expect("enabled rejection persists");
        assert_rejected(
            &enabled_receipt,
            UpdateDecisionError::InstallationMustBeDisabled,
        );
        assert!(matches!(
            enabled_receipt.witness,
            UpdateReceiptWitness::CoupledDecisionPreflight { .. }
        ));
        assert_eq!(
            enabled_repository
                .installation_repository
                .event_history(installation.installation_id())
                .expect("owner history remains unchanged"),
            enabled_owner_history
        );
        assert_eq!(
            enabled_repository
                .execute(enabled_apply)
                .expect("enabled rejection retry exact"),
            enabled_receipt
        );

        let disabled =
            disable_repository_installation(&mut repository, &installation, "repo-atomic");
        let apply = UpdateCommand::apply(
            parsed!(UpdateCommandId, "update-cmd:apply-repo-atomic"),
            ready.update_id().clone(),
            ready.revision().clone(),
            disabled.revision().clone(),
        )
        .expect("apply command validates");
        let before_apply_update_history = repository
            .event_history(ready.update_id())
            .expect("history loads");
        let before_apply_install_history = repository
            .installation_repository
            .event_history(installation.installation_id())
            .expect("installation history loads");
        let before_apply_grants = repository
            .grant_repository
            .load_current_for_installation(
                installation.tenant_id(),
                installation.user_id(),
                installation.installation_id(),
                disabled.revision(),
            )
            .expect("grant snapshot loads");
        repository.fail_next_commit_for_test();
        assert_eq!(
            repository
                .execute(apply.clone())
                .expect_err("coupled failure injects atomically"),
            UpdateRepositoryError::InjectedPersistenceFailure
        );
        assert_eq!(
            repository
                .event_history(ready.update_id())
                .expect("history"),
            before_apply_update_history
        );
        assert_eq!(
            repository
                .installation_repository
                .event_history(installation.installation_id())
                .expect("installation history"),
            before_apply_install_history
        );
        assert_eq!(
            repository
                .grant_repository
                .load_current_for_installation(
                    installation.tenant_id(),
                    installation.user_id(),
                    installation.installation_id(),
                    disabled.revision(),
                )
                .expect("grant snapshot loads"),
            before_apply_grants
        );
        let applied = repository
            .execute(apply.clone())
            .expect("same apply retries after injected failure");
        assert!(
            matches!(applied.outcome(), UpdateCommandOutcome::Accepted { event, .. } if event.kind() == UpdateEventKind::Applied)
        );
        assert_eq!(
            repository.execute(apply).expect("applied retry exact"),
            applied
        );

        let (mut repository2, stage2, _installation2, _) =
            seeded_update_repository("repo-atomic-reuse");
        repository2.consumed_approvals.insert(
            approval_id.clone(),
            (
                ready.update_id().clone(),
                match &approve.action {
                    UpdateCommandAction::RecordApproval { approval, .. } => {
                        approval.evidence_digest().clone()
                    }
                    _ => unreachable!(),
                },
            ),
        );
        let staged2 = repository2
            .execute(stage2)
            .expect("second update staged")
            .outcome()
            .accepted_snapshot_for_test();
        let reused = repository2
            .execute(approval_command_reusing_id(
                "approve-reused-global-approval",
                &approval_id,
                &staged2,
            ))
            .expect("global approval reuse rejection persists");
        assert_rejected(&reused, UpdateDecisionError::ApprovalAlreadyConsumed);
        match &reused.witness {
            UpdateReceiptWitness::ApprovalAlreadyConsumed {
                prior_update_id,
                prior_evidence_digest,
            } => {
                assert_eq!(prior_update_id, ready.update_id());
                assert_eq!(
                    prior_evidence_digest,
                    match &approve.action {
                        UpdateCommandAction::RecordApproval { approval, .. } =>
                            approval.evidence_digest(),
                        _ => unreachable!(),
                    }
                );
            }
            _ => panic!("approval reuse must carry consumed prior tuple"),
        }
    }

    #[test]
    fn repository_rebuilds_current_slot_all_ledgers_consumed_approvals_and_authority_indexes() {
        {
            let (
                mut enabled_repository,
                enabled_ready,
                enabled_update_receipts,
                initial_installation,
                initial_install_receipt,
            ) = staged_and_ready_repository("repo-rebuild-enabled-preflight");
            let enable_evidence = EnablePreconditionEvidence::from_authority_bindings(
                initial_installation.installation_id().clone(),
                initial_installation.revision().clone(),
                initial_installation.package_pin().package_digest().clone(),
                initial_installation
                    .package_pin()
                    .component_set_digest()
                    .clone(),
                initial_installation.configuration().digest().clone(),
                initial_installation
                    .package_pin()
                    .capability_manifest_digest()
                    .clone(),
                digest('7'),
                digest('8'),
            )
            .expect("reseed enable evidence validates");
            let enable_command = InstallationCommand::enable(
                parsed!(InstallationCommandId, "cmd:enable-update-rebuild-preflight"),
                initial_installation.installation_id().clone(),
                initial_installation.revision().clone(),
                enable_evidence,
            )
            .expect("reseed enable command validates");
            let enable_receipt = enabled_repository
                .installation_repository
                .execute(enable_command)
                .expect("reseed owner enable commits");
            let enabled_installation = enabled_repository
                .installation_repository
                .load_exact(initial_installation.installation_id())
                .expect("reseed owner loads")
                .expect("reseed owner exists");
            let enabled_apply = UpdateCommand::apply(
                parsed!(
                    UpdateCommandId,
                    "update-cmd:apply-rebuild-enabled-preflight"
                ),
                enabled_ready.update_id().clone(),
                enabled_ready.revision().clone(),
                enabled_installation.revision().clone(),
            )
            .expect("reseed enabled apply validates");
            let enabled_rejection = enabled_repository
                .execute(enabled_apply)
                .expect("reseed enabled rejection persists");
            assert_rejected(
                &enabled_rejection,
                UpdateDecisionError::InstallationMustBeDisabled,
            );
            let complete_installation_histories = installation_histories_for(
                &enabled_repository,
                initial_installation.installation_id(),
            );
            let complete_installation_ledger = vec![
                (initial_install_receipt.clone(), None),
                (enable_receipt, Some(initial_installation.clone())),
            ];
            let complete_update_ledger = vec![
                (enabled_update_receipts[0].clone(), None),
                (
                    enabled_update_receipts[1].clone(),
                    Some(
                        enabled_update_receipts[0]
                            .outcome()
                            .accepted_snapshot_for_test(),
                    ),
                ),
                (enabled_rejection.clone(), Some(enabled_ready.clone())),
            ];
            InMemoryPackageUpdateRepository::try_from_authority_histories(
                enabled_repository.catalog_read_models.clone(),
                enabled_repository.catalog_publications.clone(),
                enabled_repository.capability_registries.clone(),
                enabled_repository.policy_snapshots.clone(),
                complete_installation_histories.clone(),
                complete_installation_ledger,
                Vec::new(),
                Vec::new(),
                update_histories_for(&enabled_repository, enabled_ready.update_id()),
                complete_update_ledger.clone(),
            )
            .expect("complete enabled preflight witness reseeds exactly");
            let mut missing_enable_history = complete_installation_histories;
            missing_enable_history[0].1.pop();
            assert_eq!(
                InMemoryPackageUpdateRepository::try_from_authority_histories(
                    enabled_repository.catalog_read_models.clone(),
                    enabled_repository.catalog_publications.clone(),
                    enabled_repository.capability_registries.clone(),
                    enabled_repository.policy_snapshots.clone(),
                    missing_enable_history,
                    vec![(initial_install_receipt, None)],
                    Vec::new(),
                    Vec::new(),
                    update_histories_for(&enabled_repository, enabled_ready.update_id()),
                    complete_update_ledger,
                )
                .expect_err("preflight witness without its owner prefix is corrupt"),
                UpdateRepositoryError::CorruptCurrentUpdateIndex
            );
        }

        let (mut repository, ready, receipts, installation, install_receipt) =
            staged_and_ready_repository("repo-rebuild");
        let rejected_stage = repository
            .execute(stage_command_fixture("repo-rebuild-conflict").0)
            .expect("active slot conflict rejected receipt persists");
        assert_rejected(&rejected_stage, UpdateDecisionError::ActiveUpdateConflict);
        let histories = update_histories_for(&repository, ready.update_id());
        let install_histories =
            installation_histories_for(&repository, installation.installation_id());
        let accepted_ledger = vec![
            (receipts[0].clone(), None),
            (
                receipts[1].clone(),
                Some(receipts[0].outcome().accepted_snapshot_for_test()),
            ),
            (rejected_stage.clone(), None),
        ];
        let mut rebuilt = InMemoryPackageUpdateRepository::try_from_authority_histories(
            repository.catalog_read_models.clone(),
            repository.catalog_publications.clone(),
            repository.capability_registries.clone(),
            repository.policy_snapshots.clone(),
            install_histories.clone(),
            vec![(install_receipt.clone(), None)],
            Vec::new(),
            Vec::new(),
            histories.clone(),
            accepted_ledger.clone(),
        )
        .expect("ordered accepted and rejected receipts reseed exact ledger");
        assert_eq!(
            rebuilt.load_exact(ready.update_id()).expect("rebuilt load"),
            Some(ready.clone())
        );
        assert_eq!(
            rebuilt
                .execute(rejected_stage.command().clone())
                .expect("rejected retry exact"),
            rejected_stage
        );

        let mut omitted_accepted = accepted_ledger.clone();
        omitted_accepted.remove(1);
        assert_eq!(
            InMemoryPackageUpdateRepository::try_from_authority_histories(
                repository.catalog_read_models.clone(),
                repository.catalog_publications.clone(),
                repository.capability_registries.clone(),
                repository.policy_snapshots.clone(),
                install_histories.clone(),
                vec![(install_receipt.clone(), None)],
                Vec::new(),
                Vec::new(),
                histories.clone(),
                omitted_accepted,
            )
            .expect_err("omitted accepted receipt corrupts ledger"),
            UpdateRepositoryError::CorruptCurrentUpdateIndex
        );
        let missing_for_duplicate = repository
            .execute(
                UpdateCommand::cancel(
                    parsed!(UpdateCommandId, "update-cmd:rebuild-missing-duplicate"),
                    parsed!(PackageUpdateId, "update:rebuild-missing-duplicate"),
                    parsed!(UpdateRevision, "update-revision:1"),
                )
                .expect("missing duplicate command validates"),
            )
            .expect("missing duplicate receipt persists");
        assert_eq!(
            InMemoryPackageUpdateRepository::try_from_authority_histories(
                repository.catalog_read_models.clone(),
                repository.catalog_publications.clone(),
                repository.capability_registries.clone(),
                repository.policy_snapshots.clone(),
                install_histories.clone(),
                vec![(install_receipt.clone(), None)],
                Vec::new(),
                Vec::new(),
                histories.clone(),
                vec![
                    (missing_for_duplicate.clone(), None),
                    (missing_for_duplicate, None)
                ],
            )
            .expect_err("duplicate rejected command conflicts in ledger"),
            UpdateRepositoryError::CommandConflict
        );
        let mut conflicting = receipts[0].clone();
        conflicting.command.update_id =
            parsed!(PackageUpdateId, "update:repo-rebuild-forged-command");
        assert_eq!(
            InMemoryPackageUpdateRepository::try_from_authority_histories(
                repository.catalog_read_models.clone(),
                repository.catalog_publications.clone(),
                repository.capability_registries.clone(),
                repository.policy_snapshots.clone(),
                install_histories.clone(),
                vec![(install_receipt.clone(), None)],
                Vec::new(),
                Vec::new(),
                histories.clone(),
                vec![(receipts[0].clone(), None), (conflicting, None)],
            )
            .expect_err("conflicting command id rejected"),
            UpdateRepositoryError::CorruptCurrentUpdateIndex
        );
        assert_eq!(
            InMemoryPackageUpdateRepository::try_from_authority_histories(
                repository.catalog_read_models.clone(),
                repository.catalog_publications.clone(),
                repository.capability_registries.clone(),
                repository.policy_snapshots.clone(),
                install_histories.clone(),
                vec![(install_receipt.clone(), None)],
                Vec::new(),
                Vec::new(),
                histories.clone(),
                vec![(receipts[1].clone(), None), (receipts[0].clone(), None)],
            )
            .expect_err("wrong observed prefix/order corrupts ledger"),
            UpdateRepositoryError::CorruptCurrentUpdateIndex
        );
        let mut forged_active = rejected_stage.clone();
        if let UpdateReceiptWitness::ActiveSlotConflict {
            conflicting_state, ..
        } = &mut forged_active.witness
        {
            *conflicting_state = UpdateState::Staged;
        }
        assert_eq!(
            InMemoryPackageUpdateRepository::try_from_authority_histories(
                repository.catalog_read_models.clone(),
                repository.catalog_publications.clone(),
                repository.capability_registries.clone(),
                repository.policy_snapshots.clone(),
                install_histories.clone(),
                vec![(install_receipt.clone(), None)],
                Vec::new(),
                Vec::new(),
                histories.clone(),
                vec![
                    (receipts[0].clone(), None),
                    (
                        receipts[1].clone(),
                        Some(receipts[0].outcome().accepted_snapshot_for_test())
                    ),
                    (forged_active, None)
                ],
            )
            .expect_err("forged active conflict state rejected"),
            UpdateRepositoryError::CorruptCurrentUpdateIndex
        );
        let mut forged_active_id = rejected_stage.clone();
        if let UpdateReceiptWitness::ActiveSlotConflict {
            conflicting_update_id,
            ..
        } = &mut forged_active_id.witness
        {
            *conflicting_update_id = rejected_stage.command().update_id().clone();
        }
        assert_eq!(
            InMemoryPackageUpdateRepository::try_from_authority_histories(
                repository.catalog_read_models.clone(),
                repository.catalog_publications.clone(),
                repository.capability_registries.clone(),
                repository.policy_snapshots.clone(),
                install_histories.clone(),
                vec![(install_receipt.clone(), None)],
                Vec::new(),
                Vec::new(),
                histories.clone(),
                vec![
                    (receipts[0].clone(), None),
                    (
                        receipts[1].clone(),
                        Some(receipts[0].outcome().accepted_snapshot_for_test())
                    ),
                    (forged_active_id, None)
                ],
            )
            .expect_err("forged active conflict id rejected"),
            UpdateRepositoryError::CorruptCurrentUpdateIndex
        );

        let mut forged_prior = rejected_stage.clone();
        forged_prior.witness = UpdateReceiptWitness::ApprovalAlreadyConsumed {
            prior_update_id: parsed!(PackageUpdateId, "update:forged-prior"),
            prior_evidence_digest: digest('0'),
        };
        assert_eq!(
            InMemoryPackageUpdateRepository::try_from_authority_histories(
                repository.catalog_read_models.clone(),
                repository.catalog_publications.clone(),
                repository.capability_registries.clone(),
                repository.policy_snapshots.clone(),
                install_histories.clone(),
                vec![(install_receipt.clone(), None)],
                Vec::new(),
                Vec::new(),
                histories.clone(),
                vec![
                    (receipts[0].clone(), None),
                    (
                        receipts[1].clone(),
                        Some(receipts[0].outcome().accepted_snapshot_for_test())
                    ),
                    (forged_prior, Some(ready.clone()))
                ],
            )
            .expect_err("forged consumed approval prior tuple rejected"),
            UpdateRepositoryError::CorruptCurrentUpdateIndex
        );

        let (disabled, disable_receipts) = disable_repository_installation_with_receipts(
            &mut repository,
            &installation,
            "repo-rebuild-coupled",
        );
        let apply = UpdateCommand::apply(
            parsed!(UpdateCommandId, "update-cmd:apply-rebuild-coupled"),
            ready.update_id().clone(),
            ready.revision().clone(),
            disabled.revision().clone(),
        )
        .expect("apply command");
        let applied_receipt = repository.execute(apply).expect("apply accepted");
        let applied = applied_receipt.outcome().accepted_snapshot_for_test();
        let mut installation_ledger = vec![(install_receipt.clone(), None)];
        installation_ledger.extend(disable_receipts);
        installation_ledger.push((
            applied_receipt
                .subordinate_installation_receipt
                .clone()
                .expect("apply owns one installation receipt"),
            Some(disabled.clone()),
        ));
        let install_histories_after_apply =
            installation_histories_for(&repository, installation.installation_id());
        let update_histories_after_apply = update_histories_for(&repository, ready.update_id());
        let mut update_ledger_through_apply = accepted_ledger.clone();
        update_ledger_through_apply.push((applied_receipt.clone(), Some(ready.clone())));
        InMemoryPackageUpdateRepository::try_from_authority_histories(
            repository.catalog_read_models.clone(),
            repository.catalog_publications.clone(),
            repository.capability_registries.clone(),
            repository.policy_snapshots.clone(),
            install_histories_after_apply.clone(),
            installation_ledger.clone(),
            Vec::new(),
            Vec::new(),
            update_histories_after_apply.clone(),
            update_ledger_through_apply.clone(),
        )
        .expect("complete Apply owner/update seeds rebuild before coupling mutation");

        let mut wrong_pin_repository = repository.clone();
        let wrong_pin_pre = wrong_pin_repository
            .installation_repository
            .load_exact(installation.installation_id())
            .expect("wrong-pin owner loads")
            .expect("wrong-pin owner exists");
        let wrong_pin_command = InstallationCommand::package_updated(
            parsed!(InstallationCommandId, "cmd:update-owner-wrong-pin"),
            wrong_pin_pre.installation_id().clone(),
            wrong_pin_pre.revision().clone(),
            digest('f'),
            ready.plan().rollback_pin().clone(),
        )
        .expect("wrong-but-valid owner command validates");
        let wrong_pin_receipt = wrong_pin_repository
            .installation_repository
            .execute(wrong_pin_command)
            .expect("wrong-but-valid owner event commits");
        let wrong_owner_event = match wrong_pin_receipt.outcome() {
            InstallationCommandOutcome::Accepted { event, .. } => event,
            InstallationCommandOutcome::Rejected { .. } => {
                panic!("wrong-but-valid owner event must be accepted")
            }
        };
        let mut forged_owner_reference_event = match applied_receipt.outcome() {
            UpdateCommandOutcome::Accepted { event, .. } => event.clone(),
            UpdateCommandOutcome::Rejected { .. } => unreachable!(),
        };
        if let UpdateEventPayload::Applied {
            prior_installation_revision,
            applied_installation_revision,
            installation_event,
            ..
        } = &mut forged_owner_reference_event.payload
        {
            *prior_installation_revision = wrong_pin_pre.revision().clone();
            *applied_installation_revision = wrong_owner_event.post_revision().clone();
            *installation_event = InstallationEventReference::from_event(
                installation.installation_id(),
                wrong_owner_event,
                InstallationEventKind::PackageUpdated,
            )
            .expect("wrong owner reference is structurally valid");
        } else {
            unreachable!();
        }
        forged_owner_reference_event.event_digest =
            digest_update_event(&forged_owner_reference_event);
        assert!(
            evolve(Some(ready.clone()), &forged_owner_reference_event).is_ok(),
            "opaque update replay alone cannot inspect owner pin payload"
        );
        assert_eq!(
            wrong_pin_repository
                .verify_subordinate_references(&forged_owner_reference_event, Some(&ready),),
            Err(UpdateRepositoryError::CorruptCurrentUpdateIndex)
        );

        let mut omitted_active_repository = repository.clone();
        let (omitted_issue_receipt, omitted_active_grant) = issue_active_grant_for_installation(
            &mut omitted_active_repository,
            &disabled,
            ready.plan().rollback_package(),
            0,
            "repo-rebuild-omitted-active",
        );
        let omitted_grant_histories = vec![(
            omitted_active_grant.snapshot_id().clone(),
            omitted_active_repository
                .grant_repository
                .event_history(omitted_active_grant.snapshot_id())
                .expect("omitted active grant history loads"),
        )];
        assert_eq!(
            InMemoryPackageUpdateRepository::try_from_authority_histories(
                repository.catalog_read_models.clone(),
                repository.catalog_publications.clone(),
                repository.capability_registries.clone(),
                repository.policy_snapshots.clone(),
                install_histories_after_apply.clone(),
                installation_ledger.clone(),
                omitted_grant_histories,
                vec![(omitted_issue_receipt, None)],
                update_histories_after_apply.clone(),
                update_ledger_through_apply.clone(),
            )
            .expect_err("active old-revision grant omitted from Apply context is corrupt"),
            UpdateRepositoryError::CorruptCurrentUpdateIndex
        );

        assert_eq!(
            InMemoryPackageUpdateRepository::try_from_authority_histories(
                repository.catalog_read_models.clone(),
                repository.catalog_publications.clone(),
                repository.capability_registries.clone(),
                repository.policy_snapshots.clone(),
                install_histories_after_apply,
                installation_ledger.clone(),
                Vec::new(),
                Vec::new(),
                histories.clone(),
                accepted_ledger.clone(),
            )
            .expect_err("unreferenced owner PackageUpdated rejected by bijection"),
            UpdateRepositoryError::CorruptCurrentUpdateIndex
        );

        let current_installation = repository
            .installation_repository
            .load_exact(installation.installation_id())
            .expect("installation loads")
            .expect("installation exists");
        let rollback_evidence = RollbackReadinessEvidence::from_bindings(
            parsed!(UpdateEvidenceId, "update-evidence:rollback-rebuild-coupled"),
            applied.update_id().clone(),
            applied.revision().clone(),
            digest_pin_value(applied.plan().rollback_pin()),
            current_installation.revision().clone(),
            current_installation.configuration_revision(),
            current_installation.configuration().digest().clone(),
            digest('f'),
            digest_policy_snapshots(&policy_snapshots_for_plan(applied.plan())),
        )
        .expect("rollback evidence validates");
        let rollback = UpdateCommand::rollback(
            parsed!(UpdateCommandId, "update-cmd:rollback-rebuild-coupled"),
            applied.update_id().clone(),
            applied.revision().clone(),
            current_installation.revision().clone(),
            rollback_evidence,
        )
        .expect("rollback command validates");
        let rollback_receipt = repository.execute(rollback).expect("rollback accepted");
        let rolled_back = rollback_receipt.outcome().accepted_snapshot_for_test();
        installation_ledger.push((
            rollback_receipt
                .subordinate_installation_receipt
                .clone()
                .expect("rollback owns one installation receipt"),
            Some(current_installation),
        ));
        let update_histories_after_rollback = update_histories_for(&repository, ready.update_id());
        let mut complete_update_ledger = update_ledger_through_apply.clone();
        complete_update_ledger.push((rollback_receipt.clone(), Some(applied.clone())));
        InMemoryPackageUpdateRepository::try_from_authority_histories(
            repository.catalog_read_models.clone(),
            repository.catalog_publications.clone(),
            repository.capability_registries.clone(),
            repository.policy_snapshots.clone(),
            installation_histories_for(&repository, installation.installation_id()),
            installation_ledger.clone(),
            Vec::new(),
            Vec::new(),
            update_histories_after_rollback.clone(),
            complete_update_ledger.clone(),
        )
        .expect("complete Rollback owner/update seeds rebuild before coupling mutation");
        assert_eq!(
            InMemoryPackageUpdateRepository::try_from_authority_histories(
                repository.catalog_read_models.clone(),
                repository.catalog_publications.clone(),
                repository.capability_registries.clone(),
                repository.policy_snapshots.clone(),
                installation_histories_for(&repository, installation.installation_id()),
                installation_ledger.clone(),
                Vec::new(),
                Vec::new(),
                update_histories_after_apply,
                update_ledger_through_apply.clone(),
            )
            .expect_err("unreferenced owner PackageRolledBack rejected by bijection"),
            UpdateRepositoryError::CorruptCurrentUpdateIndex
        );

        let rolled_back_installation = repository
            .installation_repository
            .load_exact(installation.installation_id())
            .expect("installation loads")
            .expect("installation exists");
        let (grant_issue_receipt, active_grant) = issue_active_grant_for_installation(
            &mut repository,
            &rolled_back_installation,
            rolled_back.plan().rollback_package(),
            0,
            "repo-rebuild-orphan-grant",
        );
        let stale_command = GrantCommand::mark_stale(
            parsed!(GrantCommandId, "grant-cmd:update-orphan-rebuild-coupling"),
            active_grant.snapshot_id().clone(),
            active_grant.version().clone(),
            GrantInvalidationReason::InstallationChanged,
        )
        .expect("stale command validates");
        let stale_receipt = repository
            .grant_repository
            .execute(stale_command)
            .expect("orphan stale owner event commits");
        let grant_histories = vec![(
            active_grant.snapshot_id().clone(),
            repository
                .grant_repository
                .event_history(active_grant.snapshot_id())
                .expect("grant history loads"),
        )];
        let grant_ledger = vec![
            (grant_issue_receipt, None),
            (stale_receipt, Some(active_grant)),
        ];
        InMemoryGrantRepository::try_from_histories_and_receipts(
            grant_histories.clone(),
            grant_ledger.clone(),
        )
        .expect("orphan grant history and ledger are independently self-consistent");
        assert_eq!(
            InMemoryPackageUpdateRepository::try_from_authority_histories(
                repository.catalog_read_models.clone(),
                repository.catalog_publications.clone(),
                repository.capability_registries.clone(),
                repository.policy_snapshots.clone(),
                installation_histories_for(&repository, installation.installation_id()),
                installation_ledger,
                grant_histories,
                grant_ledger,
                update_histories_after_rollback,
                complete_update_ledger,
            )
            .expect_err("unreferenced update-prefixed MarkedStale rejected by bijection"),
            UpdateRepositoryError::CorruptCurrentUpdateIndex
        );
    }

    #[test]
    fn permission_expansion_and_every_conservative_update_require_exact_approval() {
        let expanded_plan = plan_with(
            manifest(
                "1.0.0",
                "Nous",
                "VerifiedRemoteMcp",
                &[cap_public()],
                "same",
            ),
            manifest(
                "1.1.0",
                "Nous",
                "VerifiedRemoteMcp",
                &[cap_public(), cap_private()],
                "same",
            ),
            component(
                "component:main",
                "component-version:1",
                '1',
                "exec:main",
                &[cap_public()],
            ),
            component(
                "component:main",
                "component-version:2",
                '2',
                "exec:main",
                &[cap_public(), cap_private()],
            ),
            registry("permission-old", false, true),
            registry("permission-new", true, true),
        );
        assert_eq!(
            expanded_plan.change_class(),
            UpdateChangeClass::ReapprovalRequired
        );

        let (mut repository, stage, installation, _install_receipt) =
            seeded_update_repository("permission-proof");
        let staged_receipt = repository
            .execute(stage)
            .expect("stage accepts without approval");
        let staged = staged_receipt.outcome().accepted_snapshot_for_test();
        assert_eq!(staged.plan().change_class(), UpdateChangeClass::Unchanged);
        let update_history_before = repository
            .event_history(staged.update_id())
            .expect("history");
        let owner_history_before = repository
            .installation_repository
            .event_history(installation.installation_id())
            .expect("owner history");
        let apply_before_approval = UpdateCommand::apply(
            parsed!(UpdateCommandId, "update-cmd:apply-before-approval"),
            staged.update_id().clone(),
            staged.revision().clone(),
            staged.plan().staged_installation_revision().clone(),
        )
        .expect("apply command validates");
        let rejected = repository
            .execute(apply_before_approval)
            .expect("domain rejection persists");
        assert_rejected(&rejected, UpdateDecisionError::IllegalTransition);
        assert_eq!(
            repository
                .event_history(staged.update_id())
                .expect("history"),
            update_history_before
        );
        assert_eq!(
            repository
                .installation_repository
                .event_history(installation.installation_id())
                .expect("owner history"),
            owner_history_before
        );
        assert_eq!(
            repository.load_exact(staged.update_id()).expect("load"),
            Some(staged.clone())
        );

        let approve = approval_command_for("permission-proof", &staged);
        let ready_receipt = repository
            .execute(approve.clone())
            .expect("exact approval accepted");
        assert!(
            matches!(ready_receipt.outcome(), UpdateCommandOutcome::Accepted { event, snapshot } if event.kind() == UpdateEventKind::ApprovalRecorded && snapshot.state() == UpdateState::Ready)
        );
        assert_eq!(
            repository
                .execute(approve)
                .expect("same approval command idempotent"),
            ready_receipt
        );

        let (mut repository2, stage2, _, _) = seeded_update_repository("permission-reuse");
        let staged2 = repository2
            .execute(stage2)
            .expect("stage2")
            .outcome()
            .accepted_snapshot_for_test();
        repository2.consumed_approvals.insert(
            parsed!(UpdateApprovalId, "update-approval:permission-proof"),
            (staged.update_id().clone(), digest('a')),
        );
        let reused = repository2
            .execute(approval_command_reusing_id(
                "permission-reuse-approval",
                &parsed!(UpdateApprovalId, "update-approval:permission-proof"),
                &staged2,
            ))
            .expect("reuse rejection");
        assert_rejected(&reused, UpdateDecisionError::ApprovalAlreadyConsumed);
    }

    #[test]
    fn apply_and_rollback_preserve_frozen_toolsets_and_change_only_current_future_authority() {
        let (mut repository, ready, _receipts, installation, _install_receipt) =
            staged_and_ready_repository("frozen-authority");
        let frozen = installation.to_resolver_snapshot();
        let disabled =
            disable_repository_installation(&mut repository, &installation, "frozen-authority");
        let (_grant_issue_receipt, active_grant) = issue_active_grant_for_installation(
            &mut repository,
            &disabled,
            ready.plan().rollback_package(),
            0,
            "frozen-authority",
        );
        let before_apply_installation = repository
            .installation_repository
            .load_exact(installation.installation_id())
            .expect("load")
            .expect("exists");
        let before_apply_grant = repository
            .grant_repository
            .load_exact(active_grant.snapshot_id())
            .expect("grant load")
            .expect("grant exists");
        let apply = UpdateCommand::apply(
            parsed!(UpdateCommandId, "update-cmd:apply-frozen-authority"),
            ready.update_id().clone(),
            ready.revision().clone(),
            disabled.revision().clone(),
        )
        .expect("apply command validates");
        let applied_receipt = repository.execute(apply).expect("apply accepted");
        let applied = applied_receipt.outcome().accepted_snapshot_for_test();
        let current_installation = repository
            .installation_repository
            .load_exact(installation.installation_id())
            .expect("installation loads")
            .expect("installation exists");
        let stale_grant = repository
            .grant_repository
            .load_exact(active_grant.snapshot_id())
            .expect("grant loads")
            .expect("grant exists");
        assert_eq!(frozen, installation.to_resolver_snapshot());
        assert_eq!(
            before_apply_installation.configuration(),
            current_installation.configuration()
        );
        assert_eq!(
            current_installation.package_pin(),
            ready.plan().target_pin()
        );
        assert_ne!(
            before_apply_installation.package_pin(),
            current_installation.package_pin()
        );
        assert_eq!(
            current_installation.state(),
            ManagedInstallationState::Disabled
        );
        assert_eq!(before_apply_grant.state(), GrantState::Active);
        assert_eq!(stale_grant.state(), GrantState::Stale);
        assert_eq!(stale_grant.snapshot_id(), active_grant.snapshot_id());
        assert!(
            matches!(applied_receipt.outcome(), UpdateCommandOutcome::Accepted { event, .. } if matches!(&event.payload, UpdateEventPayload::Applied { grant_events, .. } if grant_events.len() == 1))
        );
        let stale_event = &repository
            .grant_repository
            .event_history(active_grant.snapshot_id())
            .expect("grant history")[1];
        assert_eq!(stale_event.kind(), GrantEventKind::MarkedStale);
        assert_eq!(
            stale_event.invalidation_reason(),
            Some(GrantInvalidationReason::InstallationChanged)
        );

        let rollback_evidence = RollbackReadinessEvidence::from_bindings(
            parsed!(
                UpdateEvidenceId,
                "update-evidence:rollback-frozen-authority"
            ),
            applied.update_id().clone(),
            applied.revision().clone(),
            digest_pin_value(applied.plan().rollback_pin()),
            current_installation.revision().clone(),
            current_installation.configuration_revision(),
            current_installation.configuration().digest().clone(),
            digest('f'),
            digest_policy_snapshots(&policy_snapshots_for_plan(applied.plan())),
        )
        .expect("rollback evidence validates");
        let rollback = UpdateCommand::rollback(
            parsed!(UpdateCommandId, "update-cmd:rollback-frozen-authority"),
            applied.update_id().clone(),
            applied.revision().clone(),
            current_installation.revision().clone(),
            rollback_evidence,
        )
        .expect("rollback command validates");
        let rolled_back = repository.execute(rollback).expect("rollback accepted");
        assert!(
            matches!(rolled_back.outcome(), UpdateCommandOutcome::Accepted { event, snapshot } if event.kind() == UpdateEventKind::RolledBack && snapshot.state() == UpdateState::RolledBack)
        );
        let after_rollback = repository
            .installation_repository
            .load_exact(installation.installation_id())
            .expect("installation loads")
            .expect("installation exists");
        assert_eq!(after_rollback.package_pin(), installation.package_pin());
        assert_eq!(after_rollback.state(), ManagedInstallationState::Disabled);
        assert_eq!(frozen, installation.to_resolver_snapshot());
        let stale_after_rollback = repository
            .grant_repository
            .load_exact(active_grant.snapshot_id())
            .expect("grant loads")
            .expect("grant exists");
        assert_eq!(stale_after_rollback.state(), GrantState::Stale);
        assert_eq!(
            stale_after_rollback.snapshot_id(),
            active_grant.snapshot_id()
        );

        let (
            mut initially_disabled_repository,
            initially_disabled_ready,
            _initial_receipts,
            initially_disabled_installation,
            _initial_install_receipt,
        ) = staged_and_ready_repository("initially-disabled-authority");
        assert_eq!(
            initially_disabled_installation.state(),
            ManagedInstallationState::InstalledDisabled
        );
        let initial_apply = UpdateCommand::apply(
            parsed!(
                UpdateCommandId,
                "update-cmd:apply-initially-disabled-authority"
            ),
            initially_disabled_ready.update_id().clone(),
            initially_disabled_ready.revision().clone(),
            initially_disabled_installation.revision().clone(),
        )
        .expect("initially disabled apply command validates");
        let initial_applied_receipt = initially_disabled_repository
            .execute(initial_apply)
            .expect("initially disabled apply accepted");
        let initial_applied = initial_applied_receipt
            .outcome()
            .accepted_snapshot_for_test();
        let initial_current_installation = initially_disabled_repository
            .installation_repository
            .load_exact(initially_disabled_installation.installation_id())
            .expect("initially disabled installation loads")
            .expect("initially disabled installation exists");
        assert_eq!(
            initial_current_installation.state(),
            ManagedInstallationState::InstalledDisabled
        );
        assert_eq!(
            initial_current_installation.package_pin(),
            initially_disabled_ready.plan().target_pin()
        );
        let initial_rollback_evidence = RollbackReadinessEvidence::from_bindings(
            parsed!(
                UpdateEvidenceId,
                "update-evidence:rollback-initially-disabled-authority"
            ),
            initial_applied.update_id().clone(),
            initial_applied.revision().clone(),
            digest_pin_value(initial_applied.plan().rollback_pin()),
            initial_current_installation.revision().clone(),
            initial_current_installation.configuration_revision(),
            initial_current_installation
                .configuration()
                .digest()
                .clone(),
            digest('f'),
            digest_policy_snapshots(&policy_snapshots_for_plan(initial_applied.plan())),
        )
        .expect("initially disabled rollback evidence validates");
        let initial_rollback = UpdateCommand::rollback(
            parsed!(
                UpdateCommandId,
                "update-cmd:rollback-initially-disabled-authority"
            ),
            initial_applied.update_id().clone(),
            initial_applied.revision().clone(),
            initial_current_installation.revision().clone(),
            initial_rollback_evidence,
        )
        .expect("initially disabled rollback command validates");
        let initial_rolled_back = initially_disabled_repository
            .execute(initial_rollback)
            .expect("initially disabled rollback accepted");
        assert!(
            matches!(initial_rolled_back.outcome(), UpdateCommandOutcome::Accepted { event, snapshot } if event.kind() == UpdateEventKind::RolledBack && snapshot.state() == UpdateState::RolledBack)
        );
        let initial_after_rollback = initially_disabled_repository
            .installation_repository
            .load_exact(initially_disabled_installation.installation_id())
            .expect("initially disabled installation loads after rollback")
            .expect("initially disabled installation exists after rollback");
        assert_eq!(
            initial_after_rollback.state(),
            ManagedInstallationState::InstalledDisabled
        );
        assert_eq!(
            initial_after_rollback.package_pin(),
            initially_disabled_installation.package_pin()
        );
    }

    #[test]
    fn public_errors_debug_display_and_authority_debug_are_category_only_and_secret_safe() {
        let sentinel_id = "sentinel-secret-update-id";
        let sentinel_digest = "9".repeat(64);
        let (mut repository, stage, installation, _install_receipt) =
            seeded_update_repository("debug-safe");
        let staged_receipt = repository.execute(stage.clone()).expect("stage");
        let staged = staged_receipt.outcome().accepted_snapshot_for_test();
        let approval = approval_command_for("debug-safe", &staged);
        let ctx = repository
            .decision_context_for_simple_command(&staged, &approval)
            .expect("context");
        let event = repository
            .event_history(staged.update_id())
            .expect("history")
            .remove(0);
        let plan = match &stage.action {
            UpdateCommandAction::Stage { plan } => plan.clone(),
            _ => unreachable!(),
        };
        let evidence = match &approval.action {
            UpdateCommandAction::RecordApproval {
                approval,
                readiness,
                ..
            } => (approval.clone(), readiness.clone()),
            _ => unreachable!(),
        };
        let confirmation = UpdateConfirmationEvidence::from_bindings(
            parsed!(
                UpdateEvidenceId,
                "update-evidence:sentinel-secret-update-id"
            ),
            parsed!(PackageUpdateId, "update:sentinel-secret-update-id"),
            parsed!(UpdateRevision, "update-revision:1"),
            parsed!(Sha256Digest, format!("sha256:{sentinel_digest}")),
            installation.installation_id().clone(),
            installation.revision().clone(),
            parsed!(Sha256Digest, format!("sha256:{sentinel_digest}")),
            parsed!(Sha256Digest, format!("sha256:{sentinel_digest}")),
        )
        .expect("confirmation evidence");
        let rollback = RollbackReadinessEvidence::from_bindings(
            parsed!(
                UpdateEvidenceId,
                "update-evidence:rollback-sentinel-secret-update-id"
            ),
            staged.update_id().clone(),
            staged.revision().clone(),
            parsed!(Sha256Digest, format!("sha256:{sentinel_digest}")),
            installation.revision().clone(),
            installation.configuration_revision(),
            installation.configuration().digest().clone(),
            parsed!(Sha256Digest, format!("sha256:{sentinel_digest}")),
            parsed!(Sha256Digest, format!("sha256:{sentinel_digest}")),
        )
        .expect("rollback evidence");
        let rendered = format!(
            "{:?} {} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?}",
            UpdateRepositoryError::CorruptCurrentUpdateIndex,
            UpdateRepositoryError::DecisionRejected(UpdateDecisionError::ApprovalMissingOrMismatch),
            PackageUpdateId::parse(format!("update:{sentinel_id}")).expect("sentinel id parses"),
            UpdateCommandId::parse(format!("update-cmd:{sentinel_id}"))
                .expect("sentinel command parses"),
            UpdateApprovalId::parse(format!("update-approval:{sentinel_id}"))
                .expect("sentinel approval parses"),
            UpdateEvidenceId::parse(format!("update-evidence:{sentinel_id}"))
                .expect("sentinel evidence parses"),
            plan,
            evidence.0,
            evidence.1,
            confirmation,
            rollback,
            ctx,
            stage.action,
            stage,
            InstallationEventReference {
                installation_id: installation.installation_id().clone(),
                sequence: InstallationEventSequence::new(1).expect("seq"),
                post_revision: installation.revision().clone(),
                command_id: parsed!(InstallationCommandId, format!("cmd:{sentinel_id}")),
                kind: InstallationEventKind::Installed,
                event_digest: parsed!(Sha256Digest, format!("sha256:{sentinel_digest}"))
            },
            GrantEventReference {
                snapshot_id: parsed!(GrantSnapshotId, format!("grant-snapshot:{sentinel_id}")),
                sequence: GrantEventSequence::new(1).expect("seq"),
                post_version: parsed!(GrantVersion, "grant-version:1"),
                command_id: parsed!(GrantCommandId, format!("grant-cmd:{sentinel_id}")),
                kind: GrantEventKind::MarkedStale,
                event_digest: parsed!(Sha256Digest, format!("sha256:{sentinel_digest}"))
            },
            event.payload,
            event,
            staged_receipt,
            InMemoryPackageUpdateRepository::new(),
            repository
                .grant_repository
                .load_current_for_installation(
                    installation.tenant_id(),
                    installation.user_id(),
                    installation.installation_id(),
                    installation.revision()
                )
                .expect("grant set"),
        );
        for marker in [
            "CorruptCurrentUpdateIndex",
            "DecisionRejected",
            "PackageUpdateId(<redacted>)",
            "UpdateCommandId(<redacted>)",
            "UpdateApprovalId(<redacted>)",
            "UpdateEvidenceId(<redacted>)",
            "PackageUpdatePlan",
            "UpdateApprovalEvidence",
            "UpdateReadinessEvidence",
            "UpdateConfirmationEvidence",
            "RollbackReadinessEvidence",
            "UpdateDecisionContext(<authority-redacted>)",
            "Stage",
            "UpdateCommand",
            "InstallationEventReference(<authority-redacted>)",
            "GrantEventReference(<authority-redacted>)",
            "UpdateEventPayload { kind:",
            "UpdateEvent { sequence:",
            "UpdateCommandReceipt(<authority-redacted>)",
            "InMemoryPackageUpdateRepository(<authority-redacted>)",
            "CurrentInstallationGrantSet(<authority-redacted>)",
        ] {
            assert!(
                rendered.contains(marker),
                "missing debug marker {marker}: {rendered}"
            );
        }
        for forbidden in [
            sentinel_id,
            &sentinel_digest,
            "tenant:update-unit",
            "user:update-unit",
            "installation:update-unit",
            "update:debug-safe",
            "safe",
            "approval evidence raw",
        ] {
            assert!(
                !rendered.contains(forbidden),
                "debug leaked {forbidden}: {rendered}"
            );
        }
    }

    trait AcceptedSnapshotForTest {
        fn accepted_snapshot_for_test(&self) -> PackageUpdateSnapshot;
    }

    impl AcceptedSnapshotForTest for UpdateCommandOutcome {
        fn accepted_snapshot_for_test(&self) -> PackageUpdateSnapshot {
            match self {
                UpdateCommandOutcome::Accepted { snapshot, .. } => snapshot.clone(),
                UpdateCommandOutcome::Rejected { error } => {
                    panic!("expected accepted snapshot, got {error:?}")
                }
            }
        }
    }
}
