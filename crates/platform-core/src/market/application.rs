//! M20-B7-A1 application façade — catalog, installation, current-grants,
//! package-update and disable verticals over existing M20-B1/B3/B4/B6 owner
//! ports.
//!
//! This module owns one framework-neutral façade. It selects no latest version,
//! infers no owner, creates no installation, issues/replaces no grant, mints no
//! approval or evidence, applies no update and serializes no wire DTO. The
//! façade exposes [`MarketApplicationService::browse_catalog`],
//! [`MarketApplicationService::package_detail`],
//! [`MarketApplicationService::installation`],
//! [`MarketApplicationService::current_grants`],
//! [`MarketApplicationService::package_update`] and
//! [`MarketApplicationService::disable_installation`].
//!
//! Safe views expose only reviewed typed metadata, declaration/digest fields,
//! installation/update pin and state/revision fields, and grant
//! identity/binding/state/version plus the safe capability definition/scope/
//! confirmation-policy fields. Raw source-policy maps, execution identities,
//! approval IDs/evidence, consumed-approval indexes, configuration entries,
//! secret carriers, private update evidence and history remain excluded.

use crate::identity::{TenantId, UserId};
use crate::invocation::{
    CapabilityId, CatalogRevision, ComponentId, ComponentKind, ComponentVersion,
    ConfirmationPolicy, GrantSnapshotId, GrantState, GrantVersion, InstallationId,
    InstallationRevision, PackageId, PackageVersion, Sha256Digest,
};
use crate::market::capability::CapabilityDefinition;
use crate::market::grant::{GrantRepository, GrantRepositoryError, GrantScope};
use crate::market::installation::{
    ConfigurationRevision, InstallationCommand, InstallationCommandId, InstallationCommandOutcome,
    InstallationDecisionError, InstallationRepository, InstallationRepositoryError,
    InstallationSnapshot, InstalledComponentPin, ManagedInstallationState,
};
use crate::market::update::{
    PackageUpdateId, PackageUpdateRepository, PackageUpdateSnapshot, UpdateChangeClass,
    UpdateRepositoryError, UpdateRevision, UpdateState,
};
use crate::market::{
    CatalogReadModel, ComponentDeclaration, ImplementationStatus, InstallPolicy, PackageTier,
    ValidatedPackageManifest,
};
use std::collections::BTreeMap;
use std::fmt;
use std::sync::Arc;

const MAX_CATALOG_REVISIONS: usize = 64;
const MIN_PAGE_LIMIT: u16 = 1;
const MAX_PAGE_LIMIT: u16 = 100;

/// Construction failure for checked catalog query/page-limit values.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarketApplicationConstructionError {
    PageLimitOutOfRange,
    UnboundContinuationOffset,
}

/// Repository-level failure for catalog read-model loading.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarketApplicationRepositoryError {
    Unavailable,
    EmptyCatalogHistory,
    TooManyCatalogRevisions,
    DuplicateCatalogRevision,
    CurrentCatalogMissing,
    CorruptCatalog,
}

/// Application-level failure for catalog, installation, grant, update and
/// disable operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarketApplicationError {
    NotFound,
    Conflict,
    LifecycleDenied,
    RepositoryUnavailable,
    CorruptAuthority,
}

/// Bounded page limit for anonymous catalog browsing.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CatalogPageLimit(u16);

impl CatalogPageLimit {
    /// Constructs a page limit accepting only `1..=100`.
    pub fn new(value: u16) -> Result<Self, MarketApplicationConstructionError> {
        if (MIN_PAGE_LIMIT..=MAX_PAGE_LIMIT).contains(&value) {
            Ok(Self(value))
        } else {
            Err(MarketApplicationConstructionError::PageLimitOutOfRange)
        }
    }

    #[must_use]
    pub const fn get(&self) -> u16 {
        self.0
    }
}

impl fmt::Debug for CatalogPageLimit {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_tuple("CatalogPageLimit")
            .field(&self.0)
            .finish()
    }
}

/// Checked anonymous catalog browse query.
///
/// Offset `0` may select the current revision. Any nonzero continuation offset
/// requires an exact [`CatalogRevision`]; paging never drifts across revisions.
#[derive(Clone, PartialEq, Eq)]
pub struct CatalogBrowseQuery {
    catalog_revision: Option<CatalogRevision>,
    offset: u32,
    limit: CatalogPageLimit,
}

impl CatalogBrowseQuery {
    /// Constructs a checked browse query.
    pub fn new(
        revision: Option<CatalogRevision>,
        offset: u32,
        limit: CatalogPageLimit,
    ) -> Result<Self, MarketApplicationConstructionError> {
        if offset != 0 && revision.is_none() {
            return Err(MarketApplicationConstructionError::UnboundContinuationOffset);
        }
        Ok(Self {
            catalog_revision: revision,
            offset,
            limit,
        })
    }

    #[must_use]
    pub fn catalog_revision(&self) -> Option<&CatalogRevision> {
        self.catalog_revision.as_ref()
    }

    #[must_use]
    pub const fn offset(&self) -> u32 {
        self.offset
    }

    #[must_use]
    pub const fn limit(&self) -> CatalogPageLimit {
        self.limit
    }
}

impl fmt::Debug for CatalogBrowseQuery {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("CatalogBrowseQuery")
            .field("catalog_revision", &self.catalog_revision)
            .field("offset", &self.offset)
            .field("limit", &self.limit)
            .finish()
    }
}

/// Checked exact package detail query.
#[derive(Clone, PartialEq, Eq)]
pub struct CatalogPackageQuery {
    catalog_revision: Option<CatalogRevision>,
    package_id: PackageId,
    package_version: PackageVersion,
}

impl CatalogPackageQuery {
    /// Constructs a package detail query. No latest/fuzzy/same-name fallback is applied.
    pub fn new(
        revision: Option<CatalogRevision>,
        package_id: PackageId,
        package_version: PackageVersion,
    ) -> Self {
        Self {
            catalog_revision: revision,
            package_id,
            package_version,
        }
    }

    #[must_use]
    pub fn catalog_revision(&self) -> Option<&CatalogRevision> {
        self.catalog_revision.as_ref()
    }

    #[must_use]
    pub fn package_id(&self) -> &PackageId {
        &self.package_id
    }

    #[must_use]
    pub fn package_version(&self) -> &PackageVersion {
        &self.package_version
    }
}

impl fmt::Debug for CatalogPackageQuery {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("CatalogPackageQuery")
            .field("catalog_revision", &self.catalog_revision)
            .field("package_id", &self.package_id)
            .field("package_version", &self.package_version)
            .finish()
    }
}

/// Checked owner-scoped installation read query.
#[derive(Clone, PartialEq, Eq)]
pub struct OwnedInstallationQuery {
    tenant_id: TenantId,
    user_id: UserId,
    installation_id: InstallationId,
}

impl OwnedInstallationQuery {
    #[must_use]
    pub fn new(tenant_id: TenantId, user_id: UserId, installation_id: InstallationId) -> Self {
        Self {
            tenant_id,
            user_id,
            installation_id,
        }
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
}

impl fmt::Debug for OwnedInstallationQuery {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("OwnedInstallationQuery")
            .field("tenant_id", &self.tenant_id)
            .field("user_id", &self.user_id)
            .field("installation_id", &self.installation_id)
            .finish()
    }
}

/// Checked owner-scoped current-grants read query. The exact installation
/// revision is part of the query; a mismatch maps to [`MarketApplicationError::Conflict`].
#[derive(Clone, PartialEq, Eq)]
pub struct OwnedInstallationGrantQuery {
    tenant_id: TenantId,
    user_id: UserId,
    installation_id: InstallationId,
    expected_installation_revision: InstallationRevision,
}

impl OwnedInstallationGrantQuery {
    #[must_use]
    pub fn new(
        tenant_id: TenantId,
        user_id: UserId,
        installation_id: InstallationId,
        expected_installation_revision: InstallationRevision,
    ) -> Self {
        Self {
            tenant_id,
            user_id,
            installation_id,
            expected_installation_revision,
        }
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
}

impl fmt::Debug for OwnedInstallationGrantQuery {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("OwnedInstallationGrantQuery")
            .field("tenant_id", &self.tenant_id)
            .field("user_id", &self.user_id)
            .field("installation_id", &self.installation_id)
            .field(
                "expected_installation_revision",
                &self.expected_installation_revision,
            )
            .finish()
    }
}

/// Checked owner-scoped package-update read query.
#[derive(Clone, PartialEq, Eq)]
pub struct OwnedUpdateQuery {
    tenant_id: TenantId,
    user_id: UserId,
    update_id: PackageUpdateId,
}

impl OwnedUpdateQuery {
    #[must_use]
    pub fn new(tenant_id: TenantId, user_id: UserId, update_id: PackageUpdateId) -> Self {
        Self {
            tenant_id,
            user_id,
            update_id,
        }
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
    pub const fn update_id(&self) -> &PackageUpdateId {
        &self.update_id
    }
}

impl fmt::Debug for OwnedUpdateQuery {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("OwnedUpdateQuery")
            .field("tenant_id", &self.tenant_id)
            .field("user_id", &self.user_id)
            .field("update_id", &self.update_id)
            .finish()
    }
}

/// Checked owner-scoped installation disable request.
#[derive(Clone, PartialEq, Eq)]
pub struct DisableInstallationRequest {
    command_id: InstallationCommandId,
    tenant_id: TenantId,
    user_id: UserId,
    installation_id: InstallationId,
    expected_revision: InstallationRevision,
}

impl DisableInstallationRequest {
    #[must_use]
    pub fn new(
        command_id: InstallationCommandId,
        tenant_id: TenantId,
        user_id: UserId,
        installation_id: InstallationId,
        expected_revision: InstallationRevision,
    ) -> Self {
        Self {
            command_id,
            tenant_id,
            user_id,
            installation_id,
            expected_revision,
        }
    }

    #[must_use]
    pub const fn command_id(&self) -> &InstallationCommandId {
        &self.command_id
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
    pub const fn expected_revision(&self) -> &InstallationRevision {
        &self.expected_revision
    }
}

impl fmt::Debug for DisableInstallationRequest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("DisableInstallationRequest")
            .field("command_id", &self.command_id)
            .field("tenant_id", &self.tenant_id)
            .field("user_id", &self.user_id)
            .field("installation_id", &self.installation_id)
            .field("expected_revision", &self.expected_revision)
            .finish()
    }
}

/// Safe anonymous package summary. Raw source-policy maps are excluded.
#[derive(Clone, PartialEq, Eq)]
pub struct MarketPackageSummary {
    package_id: PackageId,
    package_version: PackageVersion,
    publisher: String,
    tier: PackageTier,
    display_name: String,
    implementation_status: ImplementationStatus,
    install_policy: InstallPolicy,
}

impl MarketPackageSummary {
    fn from_manifest(manifest: &ValidatedPackageManifest) -> Self {
        Self {
            package_id: manifest.package_id().clone(),
            package_version: manifest.package_version().clone(),
            publisher: manifest.publisher().to_owned(),
            tier: manifest.tier(),
            display_name: manifest.display_name().to_owned(),
            implementation_status: manifest.implementation_status(),
            install_policy: *manifest.install_policy(),
        }
    }

    #[must_use]
    pub fn package_id(&self) -> &PackageId {
        &self.package_id
    }

    #[must_use]
    pub fn package_version(&self) -> &PackageVersion {
        &self.package_version
    }

    #[must_use]
    pub fn publisher(&self) -> &str {
        &self.publisher
    }

    #[must_use]
    pub const fn tier(&self) -> PackageTier {
        self.tier
    }

    #[must_use]
    pub fn display_name(&self) -> &str {
        &self.display_name
    }

    #[must_use]
    pub const fn implementation_status(&self) -> ImplementationStatus {
        self.implementation_status
    }

    #[must_use]
    pub const fn install_policy(&self) -> &InstallPolicy {
        &self.install_policy
    }
}

impl fmt::Debug for MarketPackageSummary {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("MarketPackageSummary")
            .field("package_id", &self.package_id)
            .field("package_version", &self.package_version)
            .field("publisher", &self.publisher)
            .field("tier", &self.tier)
            .field("display_name", &self.display_name)
            .field("implementation_status", &self.implementation_status)
            .field("install_policy", &self.install_policy)
            .finish()
    }
}

/// Safe anonymous package detail. Raw source-policy maps and execution identities are excluded.
#[derive(Clone, PartialEq, Eq)]
pub struct MarketPackageDetail {
    catalog_revision: CatalogRevision,
    catalog_digest: Sha256Digest,
    summary: MarketPackageSummary,
    description: Option<String>,
    components: Vec<ComponentDeclaration>,
    capabilities: Vec<CapabilityId>,
    package_digest: Sha256Digest,
    component_declaration_set_digest: Sha256Digest,
    capability_manifest_digest: Sha256Digest,
    source_policy_digest: Sha256Digest,
}

impl MarketPackageDetail {
    fn from_manifest(model: &CatalogReadModel, manifest: &ValidatedPackageManifest) -> Self {
        Self {
            catalog_revision: model.catalog_revision().clone(),
            catalog_digest: model.catalog_digest().clone(),
            summary: MarketPackageSummary::from_manifest(manifest),
            description: manifest.description().map(str::to_owned),
            components: manifest.components().to_vec(),
            capabilities: manifest.capabilities().to_vec(),
            package_digest: manifest.package_digest().clone(),
            component_declaration_set_digest: manifest.component_declaration_set_digest().clone(),
            capability_manifest_digest: manifest.capability_manifest_digest().clone(),
            source_policy_digest: manifest.source_policy_digest().clone(),
        }
    }

    #[must_use]
    pub fn catalog_revision(&self) -> &CatalogRevision {
        &self.catalog_revision
    }

    #[must_use]
    pub fn catalog_digest(&self) -> &Sha256Digest {
        &self.catalog_digest
    }

    #[must_use]
    pub fn summary(&self) -> &MarketPackageSummary {
        &self.summary
    }

    #[must_use]
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    #[must_use]
    pub fn components(&self) -> &[ComponentDeclaration] {
        &self.components
    }

    #[must_use]
    pub fn capabilities(&self) -> &[CapabilityId] {
        &self.capabilities
    }

    #[must_use]
    pub fn package_digest(&self) -> &Sha256Digest {
        &self.package_digest
    }

    #[must_use]
    pub fn component_declaration_set_digest(&self) -> &Sha256Digest {
        &self.component_declaration_set_digest
    }

    #[must_use]
    pub fn capability_manifest_digest(&self) -> &Sha256Digest {
        &self.capability_manifest_digest
    }

    #[must_use]
    pub fn source_policy_digest(&self) -> &Sha256Digest {
        &self.source_policy_digest
    }
}

impl fmt::Debug for MarketPackageDetail {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("MarketPackageDetail")
            .field("catalog_revision", &self.catalog_revision)
            .field("catalog_digest", &self.catalog_digest)
            .field("summary", &self.summary)
            .field("description", &self.description)
            .field("components", &self.components)
            .field("capabilities", &self.capabilities)
            .field("package_digest", &self.package_digest)
            .field(
                "component_declaration_set_digest",
                &self.component_declaration_set_digest,
            )
            .field(
                "capability_manifest_digest",
                &self.capability_manifest_digest,
            )
            .field("source_policy_digest", &self.source_policy_digest)
            .finish()
    }
}

/// One bounded anonymous catalog page. Paging preserves canonical
/// `(package_id, package_version)` order and never continues across revisions.
#[derive(Clone, PartialEq, Eq)]
pub struct MarketCatalogPage {
    catalog_revision: CatalogRevision,
    catalog_digest: Sha256Digest,
    packages: Vec<MarketPackageSummary>,
    next_offset: Option<u32>,
}

impl MarketCatalogPage {
    #[must_use]
    pub fn catalog_revision(&self) -> &CatalogRevision {
        &self.catalog_revision
    }

    #[must_use]
    pub fn catalog_digest(&self) -> &Sha256Digest {
        &self.catalog_digest
    }

    #[must_use]
    pub fn packages(&self) -> &[MarketPackageSummary] {
        &self.packages
    }

    #[must_use]
    pub const fn next_offset(&self) -> Option<u32> {
        self.next_offset
    }
}

impl fmt::Debug for MarketCatalogPage {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("MarketCatalogPage")
            .field("catalog_revision", &self.catalog_revision)
            .field("catalog_digest", &self.catalog_digest)
            .field("package_count", &self.packages.len())
            .field("next_offset", &self.next_offset)
            .finish()
    }
}

/// Safe installed component view. Execution identities are excluded.
#[derive(Clone, PartialEq, Eq)]
pub struct MarketInstalledComponentView {
    component_id: ComponentId,
    kind: ComponentKind,
    version: ComponentVersion,
    digest: Sha256Digest,
}

impl MarketInstalledComponentView {
    fn from_component(component: &InstalledComponentPin) -> Self {
        Self {
            component_id: component.component_id().clone(),
            kind: component.kind(),
            version: component.version().clone(),
            digest: component.digest().clone(),
        }
    }

    #[must_use]
    pub const fn component_id(&self) -> &ComponentId {
        &self.component_id
    }

    #[must_use]
    pub const fn kind(&self) -> ComponentKind {
        self.kind
    }

    #[must_use]
    pub const fn version(&self) -> &ComponentVersion {
        &self.version
    }

    #[must_use]
    pub const fn digest(&self) -> &Sha256Digest {
        &self.digest
    }
}

impl fmt::Debug for MarketInstalledComponentView {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("MarketInstalledComponentView")
            .field("component_id", &self.component_id)
            .field("kind", &self.kind)
            .field("version", &self.version)
            .field("digest", &self.digest)
            .finish()
    }
}

/// Safe installation/update package pin view. Execution identities are excluded
/// from every nested component.
#[derive(Clone, PartialEq, Eq)]
pub struct MarketPackagePinView {
    catalog_revision: CatalogRevision,
    package_id: PackageId,
    package_version: PackageVersion,
    package_digest: Sha256Digest,
    components: Vec<MarketInstalledComponentView>,
    component_set_digest: Sha256Digest,
    capability_manifest_digest: Sha256Digest,
}

impl MarketPackagePinView {
    fn from_pin(pin: &crate::market::installation::InstallationPackagePin) -> Self {
        let components: Vec<MarketInstalledComponentView> = pin
            .components()
            .iter()
            .map(MarketInstalledComponentView::from_component)
            .collect();
        Self {
            catalog_revision: pin.catalog_revision().clone(),
            package_id: pin.package_id().clone(),
            package_version: pin.package_version().clone(),
            package_digest: pin.package_digest().clone(),
            components,
            component_set_digest: pin.component_set_digest().clone(),
            capability_manifest_digest: pin.capability_manifest_digest().clone(),
        }
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
    pub fn components(&self) -> &[MarketInstalledComponentView] {
        &self.components
    }

    #[must_use]
    pub const fn component_set_digest(&self) -> &Sha256Digest {
        &self.component_set_digest
    }

    #[must_use]
    pub const fn capability_manifest_digest(&self) -> &Sha256Digest {
        &self.capability_manifest_digest
    }
}

impl fmt::Debug for MarketPackagePinView {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("MarketPackagePinView")
            .field("catalog_revision", &self.catalog_revision)
            .field("package_id", &self.package_id)
            .field("package_version", &self.package_version)
            .field("package_digest", &self.package_digest)
            .field("components", &self.components)
            .field("component_set_digest", &self.component_set_digest)
            .field(
                "capability_manifest_digest",
                &self.capability_manifest_digest,
            )
            .finish()
    }
}

/// Safe owned installation view. Configuration entries, `NonSecretText`,
/// `SecretRef` and execution identities are excluded.
#[derive(Clone, PartialEq, Eq)]
pub struct MarketInstallationView {
    installation_id: InstallationId,
    package_pin: MarketPackagePinView,
    state: ManagedInstallationState,
    revision: InstallationRevision,
    configuration_revision: ConfigurationRevision,
    configuration_digest: Sha256Digest,
}

impl MarketInstallationView {
    #[must_use]
    pub const fn installation_id(&self) -> &InstallationId {
        &self.installation_id
    }

    #[must_use]
    pub const fn package_pin(&self) -> &MarketPackagePinView {
        &self.package_pin
    }

    #[must_use]
    pub const fn state(&self) -> ManagedInstallationState {
        self.state
    }

    #[must_use]
    pub const fn revision(&self) -> &InstallationRevision {
        &self.revision
    }

    #[must_use]
    pub const fn configuration_revision(&self) -> ConfigurationRevision {
        self.configuration_revision
    }

    #[must_use]
    pub const fn configuration_digest(&self) -> &Sha256Digest {
        &self.configuration_digest
    }
}

impl fmt::Debug for MarketInstallationView {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("MarketInstallationView")
            .field("installation_id", &self.installation_id)
            .field("package_pin", &self.package_pin)
            .field("state", &self.state)
            .field("revision", &self.revision)
            .field("configuration_revision", &self.configuration_revision)
            .field("configuration_digest", &self.configuration_digest)
            .finish()
    }
}

/// Safe current grant view. Approval IDs/evidence, consumed-approval indexes
/// and history are excluded; only grant identity/binding/state/version and the
/// safe capability definition/scope/confirmation policy fields are exposed.
#[derive(Clone, PartialEq, Eq)]
pub struct MarketGrantView {
    snapshot_id: GrantSnapshotId,
    installation_id: InstallationId,
    installation_revision: InstallationRevision,
    catalog_revision: CatalogRevision,
    package_id: PackageId,
    package_version: PackageVersion,
    package_digest: Sha256Digest,
    capability_id: CapabilityId,
    capability_definition: CapabilityDefinition,
    scope: GrantScope,
    confirmation_policy: ConfirmationPolicy,
    state: GrantState,
    version: GrantVersion,
}

impl MarketGrantView {
    fn from_snapshot(snapshot: &crate::market::grant::GrantSnapshot) -> Self {
        Self {
            snapshot_id: snapshot.snapshot_id().clone(),
            installation_id: snapshot.installation_id().clone(),
            installation_revision: snapshot.installation_revision().clone(),
            catalog_revision: snapshot.catalog_revision().clone(),
            package_id: snapshot.package_id().clone(),
            package_version: snapshot.package_version().clone(),
            package_digest: snapshot.package_digest().clone(),
            capability_id: snapshot.capability_id().clone(),
            capability_definition: snapshot.capability_definition().clone(),
            scope: snapshot.scope().clone(),
            confirmation_policy: snapshot.confirmation_policy(),
            state: snapshot.state(),
            version: snapshot.version().clone(),
        }
    }

    #[must_use]
    pub const fn snapshot_id(&self) -> &GrantSnapshotId {
        &self.snapshot_id
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
    pub const fn capability_definition(&self) -> &CapabilityDefinition {
        &self.capability_definition
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
    pub const fn state(&self) -> GrantState {
        self.state
    }

    #[must_use]
    pub const fn version(&self) -> &GrantVersion {
        &self.version
    }
}

impl fmt::Debug for MarketGrantView {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("MarketGrantView")
            .field("snapshot_id", &self.snapshot_id)
            .field("installation_id", &self.installation_id)
            .field("installation_revision", &self.installation_revision)
            .field("catalog_revision", &self.catalog_revision)
            .field("package_id", &self.package_id)
            .field("package_version", &self.package_version)
            .field("package_digest", &self.package_digest)
            .field("capability_id", &self.capability_id)
            .field("capability_definition", &self.capability_definition)
            .field("scope", &self.scope)
            .field("confirmation_policy", &self.confirmation_policy)
            .field("state", &self.state)
            .field("version", &self.version)
            .finish()
    }
}

/// One complete canonically sorted current nonterminal grant set for an exact
/// installation/revision. Revoked history is absent.
#[derive(Clone, PartialEq, Eq)]
pub struct MarketGrantPage {
    installation_id: InstallationId,
    observed_installation_revision: InstallationRevision,
    grants: Vec<MarketGrantView>,
}

impl MarketGrantPage {
    #[must_use]
    pub const fn installation_id(&self) -> &InstallationId {
        &self.installation_id
    }

    #[must_use]
    pub const fn observed_installation_revision(&self) -> &InstallationRevision {
        &self.observed_installation_revision
    }

    #[must_use]
    pub fn grants(&self) -> &[MarketGrantView] {
        &self.grants
    }
}

impl fmt::Debug for MarketGrantPage {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("MarketGrantPage")
            .field("installation_id", &self.installation_id)
            .field(
                "observed_installation_revision",
                &self.observed_installation_revision,
            )
            .field("grant_count", &self.grants.len())
            .finish()
    }
}

/// Safe package-update view.
///
/// Readiness/confirmation/rollback evidence, policies, private routes, history,
/// execution identities, configuration entries, raw source maps and secret
/// carriers are all excluded. Only update identity, installation binding,
/// rollback/target pins, change class, state, revision and optional applied
/// installation revision are exposed.
#[derive(Clone, PartialEq, Eq)]
pub struct MarketUpdateView {
    update_id: PackageUpdateId,
    installation_id: InstallationId,
    rollback_pin: MarketPackagePinView,
    target_pin: MarketPackagePinView,
    change_class: UpdateChangeClass,
    state: UpdateState,
    revision: UpdateRevision,
    applied_installation_revision: Option<InstallationRevision>,
}

impl MarketUpdateView {
    #[must_use]
    pub const fn update_id(&self) -> &PackageUpdateId {
        &self.update_id
    }

    #[must_use]
    pub const fn installation_id(&self) -> &InstallationId {
        &self.installation_id
    }

    #[must_use]
    pub const fn rollback_pin(&self) -> &MarketPackagePinView {
        &self.rollback_pin
    }

    #[must_use]
    pub const fn target_pin(&self) -> &MarketPackagePinView {
        &self.target_pin
    }

    #[must_use]
    pub const fn change_class(&self) -> &UpdateChangeClass {
        &self.change_class
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
    pub fn applied_installation_revision(&self) -> Option<&InstallationRevision> {
        self.applied_installation_revision.as_ref()
    }
}

impl fmt::Debug for MarketUpdateView {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("MarketUpdateView")
            .field("update_id", &self.update_id)
            .field("installation_id", &self.installation_id)
            .field("rollback_pin", &self.rollback_pin)
            .field("target_pin", &self.target_pin)
            .field("change_class", &self.change_class)
            .field("state", &self.state)
            .field("revision", &self.revision)
            .field(
                "applied_installation_revision",
                &self.applied_installation_revision,
            )
            .finish()
    }
}

/// Safe disable receipt view. It is a historical command disposition, not a
/// current-state claim.
#[derive(Clone, PartialEq, Eq)]
pub struct DisableInstallationReceiptView {
    command_id: InstallationCommandId,
    installation_id: InstallationId,
    post_state: ManagedInstallationState,
    post_revision: InstallationRevision,
}

impl DisableInstallationReceiptView {
    #[must_use]
    pub const fn command_id(&self) -> &InstallationCommandId {
        &self.command_id
    }

    #[must_use]
    pub const fn installation_id(&self) -> &InstallationId {
        &self.installation_id
    }

    #[must_use]
    pub const fn post_state(&self) -> ManagedInstallationState {
        self.post_state
    }

    #[must_use]
    pub const fn post_revision(&self) -> &InstallationRevision {
        &self.post_revision
    }
}

impl fmt::Debug for DisableInstallationReceiptView {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("DisableInstallationReceiptView")
            .field("command_id", &self.command_id)
            .field("installation_id", &self.installation_id)
            .field("post_state", &self.post_state)
            .field("post_revision", &self.post_revision)
            .finish()
    }
}

/// Semantic repository port for anonymous catalog read models.
pub trait CatalogReadRepository {
    /// Loads the current catalog revision.
    fn load_current(&self) -> Result<Arc<CatalogReadModel>, MarketApplicationRepositoryError>;

    /// Loads one exact catalog revision, or `None` if absent.
    fn load_exact(
        &self,
        revision: &CatalogRevision,
    ) -> Result<Option<Arc<CatalogReadModel>>, MarketApplicationRepositoryError>;
}

/// Deterministic in-memory catalog read repository.
///
/// `try_new` is its only public inherent method: it accepts `1..=64` exact
/// immutable catalog revisions, rejects duplicate/current-missing histories,
/// and exposes no mutator.
#[derive(Clone)]
pub struct InMemoryCatalogReadRepository {
    revisions: BTreeMap<CatalogRevision, Arc<CatalogReadModel>>,
    current: Arc<CatalogReadModel>,
}

impl fmt::Debug for InMemoryCatalogReadRepository {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("InMemoryCatalogReadRepository(<authority-redacted>)")
    }
}

impl InMemoryCatalogReadRepository {
    /// Constructs a checked in-memory catalog history.
    pub fn try_new(
        revisions: Vec<CatalogReadModel>,
        current_revision: CatalogRevision,
    ) -> Result<Self, MarketApplicationRepositoryError> {
        if revisions.is_empty() {
            return Err(MarketApplicationRepositoryError::EmptyCatalogHistory);
        }
        if revisions.len() > MAX_CATALOG_REVISIONS {
            return Err(MarketApplicationRepositoryError::TooManyCatalogRevisions);
        }
        let mut map: BTreeMap<CatalogRevision, Arc<CatalogReadModel>> = BTreeMap::new();
        for model in revisions {
            let revision = model.catalog_revision().clone();
            if map.insert(revision, Arc::new(model)).is_some() {
                return Err(MarketApplicationRepositoryError::DuplicateCatalogRevision);
            }
        }
        let current = map
            .get(&current_revision)
            .cloned()
            .ok_or(MarketApplicationRepositoryError::CurrentCatalogMissing)?;
        Ok(Self {
            revisions: map,
            current,
        })
    }
}

impl CatalogReadRepository for InMemoryCatalogReadRepository {
    fn load_current(&self) -> Result<Arc<CatalogReadModel>, MarketApplicationRepositoryError> {
        Ok(self.current.clone())
    }

    fn load_exact(
        &self,
        revision: &CatalogRevision,
    ) -> Result<Option<Arc<CatalogReadModel>>, MarketApplicationRepositoryError> {
        Ok(self.revisions.get(revision).cloned())
    }
}

/// Framework-neutral application service over owner repositories.
///
/// The catalog vertical selects no latest version, infers no owner, and mints
/// no evidence. Owner-scoped reads identify their owner revisions and do not
/// claim cross-repository atomicity. The update vertical reads one exact owned
/// update without applying it.
pub struct MarketApplicationService<C, I, G, U> {
    catalogs: C,
    installations: I,
    grants: G,
    updates: U,
}

impl<C, I, G, U> fmt::Debug for MarketApplicationService<C, I, G, U> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("MarketApplicationService(<authority-redacted>)")
    }
}

impl<C, I, G, U> MarketApplicationService<C, I, G, U>
where
    C: CatalogReadRepository,
    I: InstallationRepository,
    G: GrantRepository,
    U: PackageUpdateRepository,
{
    /// Constructs the service from its four owner repositories.
    pub fn new(catalogs: C, installations: I, grants: G, updates: U) -> Self {
        Self {
            catalogs,
            installations,
            grants,
            updates,
        }
    }

    /// Browses the catalog with exact revision binding and bounded paging.
    pub fn browse_catalog(
        &self,
        query: &CatalogBrowseQuery,
    ) -> Result<MarketCatalogPage, MarketApplicationError> {
        let model = self.resolve_catalog(query.catalog_revision())?;
        let packages = model.packages();
        let offset = usize::try_from(query.offset()).unwrap_or(usize::MAX);
        let limit = usize::from(query.limit().get());
        let start = offset.min(packages.len());
        let end = start.saturating_add(limit).min(packages.len());
        let summaries: Vec<MarketPackageSummary> = packages[start..end]
            .iter()
            .map(MarketPackageSummary::from_manifest)
            .collect();
        let next_offset = (end < packages.len()).then_some(u32::try_from(end).unwrap_or(u32::MAX));
        Ok(MarketCatalogPage {
            catalog_revision: model.catalog_revision().clone(),
            catalog_digest: model.catalog_digest().clone(),
            packages: summaries,
            next_offset,
        })
    }

    /// Reads one exact package detail. No latest/fuzzy/same-name fallback.
    pub fn package_detail(
        &self,
        query: &CatalogPackageQuery,
    ) -> Result<MarketPackageDetail, MarketApplicationError> {
        let model = self.resolve_catalog(query.catalog_revision())?;
        let manifest = model
            .find(query.package_id(), query.package_version())
            .ok_or(MarketApplicationError::NotFound)?;
        Ok(MarketPackageDetail::from_manifest(&model, manifest))
    }

    /// Reads one owned installation. Absence and owner mismatch map to `NotFound`.
    pub fn installation(
        &self,
        query: &OwnedInstallationQuery,
    ) -> Result<MarketInstallationView, MarketApplicationError> {
        let snapshot = self
            .installations
            .load_exact(query.installation_id())
            .map_err(map_installation_load_error)?
            .ok_or(MarketApplicationError::NotFound)?;
        if snapshot.tenant_id() != query.tenant_id() || snapshot.user_id() != query.user_id() {
            return Err(MarketApplicationError::NotFound);
        }
        Ok(installation_view_from_snapshot(&snapshot))
    }

    /// Reads the complete canonically sorted current nonterminal grant set for
    /// one exact installation/revision. The exact installation revision is part
    /// of the query; a mismatch maps to `Conflict`. This method issues, revokes
    /// and replaces no grant, mints no approval/evidence, and exposes no
    /// consumed approval or history carrier.
    pub fn current_grants(
        &self,
        query: &OwnedInstallationGrantQuery,
    ) -> Result<MarketGrantPage, MarketApplicationError> {
        let snapshot = self
            .installations
            .load_exact(query.installation_id())
            .map_err(map_installation_load_error)?
            .ok_or(MarketApplicationError::NotFound)?;
        if snapshot.tenant_id() != query.tenant_id() || snapshot.user_id() != query.user_id() {
            return Err(MarketApplicationError::NotFound);
        }
        if snapshot.revision() != query.expected_installation_revision() {
            return Err(MarketApplicationError::Conflict);
        }
        let set = self
            .grants
            .load_current_for_installation(
                query.tenant_id(),
                query.user_id(),
                query.installation_id(),
                query.expected_installation_revision(),
            )
            .map_err(map_grant_load_error)?;
        let grants: Vec<MarketGrantView> = set
            .grants()
            .iter()
            .map(MarketGrantView::from_snapshot)
            .collect();
        Ok(MarketGrantPage {
            installation_id: set.installation_id().clone(),
            observed_installation_revision: set.observed_installation_revision().clone(),
            grants,
        })
    }

    /// Reads one exact owned package update without applying it.
    ///
    /// Loads the exact update from [`PackageUpdateRepository::load_exact`];
    /// absence or owner mismatch maps to [`MarketApplicationError::NotFound`];
    /// repository/persistence unavailability maps to
    /// [`MarketApplicationError::RepositoryUnavailable`]; corrupt replay/index
    /// drift maps to [`MarketApplicationError::CorruptAuthority`];
    /// revision/slot conflicts surfaced by the port map to
    /// [`MarketApplicationError::Conflict`].
    pub fn package_update(
        &self,
        query: &OwnedUpdateQuery,
    ) -> Result<MarketUpdateView, MarketApplicationError> {
        let snapshot = self
            .updates
            .load_exact(query.update_id())
            .map_err(map_update_load_error)?
            .ok_or(MarketApplicationError::NotFound)?;
        if snapshot.tenant_id() != query.tenant_id() || snapshot.user_id() != query.user_id() {
            return Err(MarketApplicationError::NotFound);
        }
        Ok(update_view_from_snapshot(&snapshot))
    }

    /// Disables one owned installation via the existing owner command ledger.
    pub fn disable_installation(
        &mut self,
        request: DisableInstallationRequest,
    ) -> Result<DisableInstallationReceiptView, MarketApplicationError> {
        let snapshot = self
            .installations
            .load_exact(request.installation_id())
            .map_err(map_installation_load_error)?
            .ok_or(MarketApplicationError::NotFound)?;
        if snapshot.tenant_id() != request.tenant_id() || snapshot.user_id() != request.user_id() {
            return Err(MarketApplicationError::NotFound);
        }
        let command = InstallationCommand::disable(
            request.command_id().clone(),
            request.installation_id().clone(),
            request.expected_revision().clone(),
        )
        .map_err(|_| MarketApplicationError::CorruptAuthority)?;
        let receipt = self
            .installations
            .execute(command)
            .map_err(map_installation_execute_error)?;
        match receipt.outcome() {
            InstallationCommandOutcome::Accepted { snapshot, .. } => {
                Ok(DisableInstallationReceiptView {
                    command_id: receipt.command().command_id().clone(),
                    installation_id: snapshot.installation_id().clone(),
                    post_state: snapshot.state(),
                    post_revision: snapshot.revision().clone(),
                })
            }
            InstallationCommandOutcome::Rejected { error } => {
                Err(map_installation_decision_error(*error))
            }
        }
    }

    fn resolve_catalog(
        &self,
        revision: Option<&CatalogRevision>,
    ) -> Result<Arc<CatalogReadModel>, MarketApplicationError> {
        match revision {
            Some(rev) => self
                .catalogs
                .load_exact(rev)
                .map_err(map_repository_error)?
                .ok_or(MarketApplicationError::NotFound),
            None => self.catalogs.load_current().map_err(map_repository_error),
        }
    }
}

fn installation_view_from_snapshot(snapshot: &InstallationSnapshot) -> MarketInstallationView {
    MarketInstallationView {
        installation_id: snapshot.installation_id().clone(),
        package_pin: MarketPackagePinView::from_pin(snapshot.package_pin()),
        state: snapshot.state(),
        revision: snapshot.revision().clone(),
        configuration_revision: snapshot.configuration_revision(),
        configuration_digest: snapshot.configuration().digest().clone(),
    }
}

fn update_view_from_snapshot(snapshot: &PackageUpdateSnapshot) -> MarketUpdateView {
    let plan = snapshot.plan();
    MarketUpdateView {
        update_id: snapshot.update_id().clone(),
        installation_id: snapshot.installation_id().clone(),
        rollback_pin: MarketPackagePinView::from_pin(plan.rollback_pin()),
        target_pin: MarketPackagePinView::from_pin(plan.target_pin()),
        change_class: plan.change_class(),
        state: snapshot.state(),
        revision: snapshot.revision().clone(),
        applied_installation_revision: snapshot.applied_installation_revision().cloned(),
    }
}

fn map_repository_error(error: MarketApplicationRepositoryError) -> MarketApplicationError {
    match error {
        MarketApplicationRepositoryError::Unavailable
        | MarketApplicationRepositoryError::EmptyCatalogHistory
        | MarketApplicationRepositoryError::TooManyCatalogRevisions => {
            MarketApplicationError::RepositoryUnavailable
        }
        MarketApplicationRepositoryError::DuplicateCatalogRevision
        | MarketApplicationRepositoryError::CurrentCatalogMissing
        | MarketApplicationRepositoryError::CorruptCatalog => {
            MarketApplicationError::CorruptAuthority
        }
    }
}

fn map_installation_load_error(error: InstallationRepositoryError) -> MarketApplicationError {
    match error {
        InstallationRepositoryError::CommandConflict => MarketApplicationError::Conflict,
        InstallationRepositoryError::InjectedPersistenceFailure => {
            MarketApplicationError::RepositoryUnavailable
        }
        InstallationRepositoryError::CorruptEventHistory(_)
        | InstallationRepositoryError::CorruptCommandLedger
        | InstallationRepositoryError::DecisionRejected(_) => {
            MarketApplicationError::CorruptAuthority
        }
    }
}

fn map_installation_execute_error(error: InstallationRepositoryError) -> MarketApplicationError {
    match error {
        InstallationRepositoryError::CommandConflict => MarketApplicationError::Conflict,
        InstallationRepositoryError::InjectedPersistenceFailure => {
            MarketApplicationError::RepositoryUnavailable
        }
        InstallationRepositoryError::CorruptEventHistory(_)
        | InstallationRepositoryError::CorruptCommandLedger => {
            MarketApplicationError::CorruptAuthority
        }
        InstallationRepositoryError::DecisionRejected(inner) => {
            map_installation_decision_error(inner)
        }
    }
}

fn map_installation_decision_error(error: InstallationDecisionError) -> MarketApplicationError {
    match error {
        InstallationDecisionError::RevisionMismatch => MarketApplicationError::Conflict,
        InstallationDecisionError::IllegalTransition
        | InstallationDecisionError::TerminalState
        | InstallationDecisionError::ConfigureWhileEnabled => {
            MarketApplicationError::LifecycleDenied
        }
        InstallationDecisionError::AggregateMissing
        | InstallationDecisionError::AggregateAlreadyPresent
        | InstallationDecisionError::TenantMismatch
        | InstallationDecisionError::EnableEvidenceMismatch
        | InstallationDecisionError::SequenceOverflow => MarketApplicationError::CorruptAuthority,
    }
}

fn map_grant_load_error(error: GrantRepositoryError) -> MarketApplicationError {
    match error {
        GrantRepositoryError::InjectedPersistenceFailure => {
            MarketApplicationError::RepositoryUnavailable
        }
        GrantRepositoryError::CommandConflict
        | GrantRepositoryError::CorruptEventHistory(_)
        | GrantRepositoryError::CorruptAuthorityIndex
        | GrantRepositoryError::DecisionRejected(_) => MarketApplicationError::CorruptAuthority,
    }
}

fn map_update_load_error(error: UpdateRepositoryError) -> MarketApplicationError {
    match error {
        UpdateRepositoryError::CommandConflict | UpdateRepositoryError::TransactionConflict => {
            MarketApplicationError::Conflict
        }
        UpdateRepositoryError::InjectedPersistenceFailure => {
            MarketApplicationError::RepositoryUnavailable
        }
        UpdateRepositoryError::CorruptUpdateHistory(_)
        | UpdateRepositoryError::CorruptInstallationHistory(_)
        | UpdateRepositoryError::CorruptGrantHistory(_)
        | UpdateRepositoryError::CorruptCurrentUpdateIndex
        | UpdateRepositoryError::CorruptGrantIndex
        | UpdateRepositoryError::CorruptGrantSet
        | UpdateRepositoryError::DecisionRejected(_) => MarketApplicationError::CorruptAuthority,
    }
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::panic, clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::invocation::{
        CapabilityClass, CapabilityId, CatalogComponentRevision, CatalogPackageRevision,
        ComponentId, ComponentKind, ComponentVersion, ExecutionIdentity, InvocationPolicySnapshot,
        PolicyRevision, PolicySnapshotId, SourcePolicyId,
    };
    use crate::market::capability::{CapabilityRegistry, load_capability_registry};
    use crate::market::grant::{
        GrantAdmissionEvidence, GrantApprovalId, GrantCommand, GrantCommandId, GrantRepository,
        GrantScope, InMemoryGrantRepository,
    };
    use crate::market::installation::{
        ConfigurationKey, ConfigurationValue, EnablePreconditionEvidence,
        InMemoryInstallationRepository, InstallationCommand, InstallationCommandId,
        InstallationConfiguration, InstallationPackagePin, InstallationRepository, NonSecretText,
        SecretRef, SecretRefId,
    };
    use crate::market::load_package_manifest;
    use crate::market::update::{
        InMemoryPackageUpdateRepository, UpdateCommand, UpdateCommandId, UpdateDecisionContext,
        decide as update_decide, evolve as update_evolve,
    };
    use std::collections::{BTreeMap, BTreeSet};
    use std::format;

    fn parsed<T, E: fmt::Display>(result: Result<T, E>) -> T {
        match result {
            Ok(value) => value,
            Err(error) => panic!("fixture must parse: {error}"),
        }
    }

    fn tenant() -> TenantId {
        parsed(TenantId::parse("tenant:app-test"))
    }

    fn user() -> UserId {
        parsed(UserId::parse("user:app-test"))
    }

    fn installation_id() -> InstallationId {
        parsed(InstallationId::parse("installation:app-test"))
    }

    fn absent_installation_id() -> InstallationId {
        parsed(InstallationId::parse("installation:absent"))
    }

    fn revision(value: u64) -> InstallationRevision {
        parsed(InstallationRevision::parse(format!(
            "installation-revision:{value}"
        )))
    }

    fn command_id(suffix: &str) -> InstallationCommandId {
        parsed(InstallationCommandId::parse(format!("cmd:{suffix}")))
    }

    fn digest(byte: char) -> Sha256Digest {
        parsed(Sha256Digest::parse(format!(
            "sha256:{}",
            byte.to_string().repeat(64)
        )))
    }

    const EXECUTION_MARKER: &str = "execution:identity-marker";
    const TEXT_MARKER: &str = "sensitive-value-marker";
    const SECRET_MARKER: &str = "secret-ref:secret-marker";

    fn configuration(tenant: &TenantId) -> InstallationConfiguration {
        InstallationConfiguration::new(
            tenant,
            vec![
                (
                    parsed(ConfigurationKey::parse("mode")),
                    ConfigurationValue::Text(parsed(NonSecretText::parse(TEXT_MARKER))),
                ),
                (
                    parsed(ConfigurationKey::parse("token")),
                    ConfigurationValue::Secret(parsed(SecretRef::new(
                        tenant.clone(),
                        parsed(SecretRefId::parse(SECRET_MARKER)),
                    ))),
                ),
            ],
        )
        .unwrap()
    }

    fn package_pin() -> InstallationPackagePin {
        InstallationPackagePin::new(
            parsed(CatalogRevision::parse("catalog:app-test")),
            parsed(PackageId::parse("ustc.app-test")),
            parsed(PackageVersion::parse("1.0.0")),
            digest('1'),
            vec![parsed(InstalledComponentPin::new(
                parsed(ComponentId::parse("component:app-test")),
                ComponentKind::NativeRustComponent,
                parsed(ComponentVersion::parse("component-version:1")),
                digest('2'),
                parsed(ExecutionIdentity::parse(EXECUTION_MARKER)),
            ))],
            digest('3'),
            digest('4'),
        )
        .unwrap()
    }

    fn evidence(sequence: u64, config: &InstallationConfiguration) -> EnablePreconditionEvidence {
        EnablePreconditionEvidence::from_authority_bindings(
            installation_id(),
            revision(sequence),
            package_pin().package_digest().clone(),
            package_pin().component_set_digest().clone(),
            config.digest().clone(),
            package_pin().capability_manifest_digest().clone(),
            digest('5'),
            digest('6'),
        )
        .unwrap()
    }

    fn install_command(suffix: &str) -> InstallationCommand {
        InstallationCommand::install(
            command_id(suffix),
            installation_id(),
            tenant(),
            user(),
            package_pin(),
            configuration(&tenant()),
        )
        .unwrap()
    }

    fn enable_command(suffix: &str, config: &InstallationConfiguration) -> InstallationCommand {
        InstallationCommand::enable(
            command_id(suffix),
            installation_id(),
            revision(1),
            evidence(1, config),
        )
        .unwrap()
    }

    struct NullCatalogRepository;

    impl CatalogReadRepository for NullCatalogRepository {
        fn load_current(&self) -> Result<Arc<CatalogReadModel>, MarketApplicationRepositoryError> {
            Err(MarketApplicationRepositoryError::Unavailable)
        }

        fn load_exact(
            &self,
            _revision: &CatalogRevision,
        ) -> Result<Option<Arc<CatalogReadModel>>, MarketApplicationRepositoryError> {
            Err(MarketApplicationRepositoryError::Unavailable)
        }
    }

    fn service_with(
        repository: InMemoryInstallationRepository,
    ) -> MarketApplicationService<
        NullCatalogRepository,
        InMemoryInstallationRepository,
        InMemoryGrantRepository,
        InMemoryPackageUpdateRepository,
    > {
        MarketApplicationService::new(
            NullCatalogRepository,
            repository,
            InMemoryGrantRepository::new(),
            InMemoryPackageUpdateRepository::new(),
        )
    }

    fn installed_repository() -> InMemoryInstallationRepository {
        let mut repository = InMemoryInstallationRepository::new();
        repository.execute(install_command("install")).unwrap();
        repository
    }

    fn enabled_repository() -> InMemoryInstallationRepository {
        let mut repository = installed_repository();
        let config = configuration(&tenant());
        repository
            .execute(enable_command("enable", &config))
            .unwrap();
        repository
    }

    #[test]
    fn owned_installation_absence_and_mismatch_are_not_found() {
        assert_eq!(
            map_installation_load_error(InstallationRepositoryError::CommandConflict),
            MarketApplicationError::Conflict
        );
        let service = service_with(installed_repository());

        let absent = OwnedInstallationQuery::new(tenant(), user(), absent_installation_id());
        assert_eq!(
            service.installation(&absent),
            Err(MarketApplicationError::NotFound)
        );

        let correct = OwnedInstallationQuery::new(tenant(), user(), installation_id());
        let view = service.installation(&correct).unwrap();
        assert_eq!(view.installation_id(), &installation_id());
        assert_eq!(view.state(), ManagedInstallationState::InstalledDisabled);
        assert_eq!(view.revision(), &revision(1));

        let foreign_tenant = OwnedInstallationQuery::new(
            parsed(TenantId::parse("tenant:foreign")),
            user(),
            installation_id(),
        );
        assert_eq!(
            service.installation(&foreign_tenant),
            Err(MarketApplicationError::NotFound)
        );

        let foreign_user = OwnedInstallationQuery::new(
            tenant(),
            parsed(UserId::parse("user:foreign")),
            installation_id(),
        );
        assert_eq!(
            service.installation(&foreign_user),
            Err(MarketApplicationError::NotFound)
        );
    }

    #[test]
    fn disable_preserves_owner_ledger_first_idempotency_and_maps_one_event() {
        let mut service = service_with(enabled_repository());

        let request = DisableInstallationRequest::new(
            command_id("disable"),
            tenant(),
            user(),
            installation_id(),
            revision(2),
        );
        let first = service.disable_installation(request.clone()).unwrap();
        assert_eq!(first.command_id(), &command_id("disable"));
        assert_eq!(first.installation_id(), &installation_id());
        assert_eq!(first.post_state(), ManagedInstallationState::Disabled);
        assert_eq!(first.post_revision(), &revision(3));

        let query = OwnedInstallationQuery::new(tenant(), user(), installation_id());
        let view = service.installation(&query).unwrap();
        assert_eq!(view.state(), ManagedInstallationState::Disabled);
        assert_eq!(view.revision(), &revision(3));

        let history_before = service
            .installations
            .event_history(&installation_id())
            .expect("event history loads after first disable");
        let history_len_before = history_before.len();

        let retry = service.disable_installation(request.clone()).unwrap();
        assert_eq!(retry, first);

        let history_after = service
            .installations
            .event_history(&installation_id())
            .expect("event history loads after typed-equal retry");
        assert_eq!(
            history_after.len(),
            history_len_before,
            "typed-equal retry must not append a second event to the owner ledger"
        );

        let view_after_retry = service.installation(&query).unwrap();
        assert_eq!(view_after_retry.revision(), &revision(3));
    }

    #[test]
    fn disable_stale_new_command_and_conflicting_reuse_are_conflict() {
        let mut service = service_with(enabled_repository());

        let stale = DisableInstallationRequest::new(
            command_id("stale"),
            tenant(),
            user(),
            installation_id(),
            revision(99),
        );
        assert_eq!(
            service.disable_installation(stale),
            Err(MarketApplicationError::Conflict)
        );

        let correct = DisableInstallationRequest::new(
            command_id("correct"),
            tenant(),
            user(),
            installation_id(),
            revision(2),
        );
        let accepted = service.disable_installation(correct.clone()).unwrap();
        assert_eq!(accepted.post_state(), ManagedInstallationState::Disabled);

        let conflicting_reuse = DisableInstallationRequest::new(
            command_id("correct"),
            tenant(),
            user(),
            installation_id(),
            revision(99),
        );
        assert_eq!(
            service.disable_installation(conflicting_reuse),
            Err(MarketApplicationError::Conflict)
        );
    }

    #[test]
    fn installation_and_receipt_debug_surfaces_exclude_sensitive_carriers() {
        let mut service = service_with(enabled_repository());

        let query = OwnedInstallationQuery::new(tenant(), user(), installation_id());
        let view = service.installation(&query).unwrap();
        let debug = format!("{view:?}");
        assert!(!debug.contains(TEXT_MARKER), "debug leaks NonSecretText");
        assert!(!debug.contains(SECRET_MARKER), "debug leaks SecretRefId");
        assert!(
            !debug.contains(EXECUTION_MARKER),
            "debug leaks ExecutionIdentity"
        );

        let request = DisableInstallationRequest::new(
            command_id("debug-disable"),
            tenant(),
            user(),
            installation_id(),
            revision(2),
        );
        let receipt = service.disable_installation(request).unwrap();
        let receipt_debug = format!("{receipt:?}");
        assert!(
            !receipt_debug.contains(TEXT_MARKER),
            "receipt debug leaks NonSecretText"
        );
        assert!(
            !receipt_debug.contains(SECRET_MARKER),
            "receipt debug leaks SecretRefId"
        );
        assert!(
            !receipt_debug.contains(EXECUTION_MARKER),
            "receipt debug leaks ExecutionIdentity"
        );
    }

    // Current-grants vertical fixtures. Grant admission evidence construction is
    // `pub(in crate::market)`, so the seeded canonical-order proof lives here as
    // an internal application test; external tests cover only the public
    // denial/empty surface.

    const GRANT_PACKAGE: &[u8] = br#"{
      "id":"ustc.grant-app","version":"1.0.0","publisher":"first-party",
      "tier":"FirstParty","displayName":"Grant App","description":"Typed fixture",
      "implementationStatus":"development",
      "installPolicy":{"class":"FirstPartySystemPlugin","defaultInstalled":true,"defaultEnabled":true,"userDisableAllowed":true},
      "components":[],
      "capabilities":["campus.public_plan.read","user.own_academic_snapshot.read"],
      "sourcePolicy":{"fixture":"bounded"}
    }"#;
    const GRANT_REGISTRY: &[u8] = include_bytes!("../../../../market/capabilities/registry.json");

    fn grant_tenant() -> TenantId {
        parsed(TenantId::parse("tenant:grant-app"))
    }

    fn grant_user() -> UserId {
        parsed(UserId::parse("user:grant-app"))
    }

    fn grant_installation_id() -> InstallationId {
        parsed(InstallationId::parse("installation:grant-app"))
    }

    fn grant_snapshot_id(suffix: &str) -> GrantSnapshotId {
        parsed(GrantSnapshotId::parse(format!("grant:{suffix}")))
    }

    fn grant_approval_id(suffix: &str) -> GrantApprovalId {
        parsed(GrantApprovalId::parse(format!("grant-approval:{suffix}")))
    }

    fn grant_command_id(suffix: &str) -> GrantCommandId {
        parsed(GrantCommandId::parse(format!("grant-cmd:{suffix}")))
    }

    fn grant_package() -> ValidatedPackageManifest {
        load_package_manifest(GRANT_PACKAGE).expect("grant package fixture")
    }

    fn grant_registry() -> CapabilityRegistry {
        load_capability_registry(GRANT_REGISTRY).expect("grant registry fixture")
    }

    fn grant_package_pin(package: &ValidatedPackageManifest) -> InstallationPackagePin {
        InstallationPackagePin::new(
            parsed(CatalogRevision::parse("catalog:grant-app")),
            package.package_id().clone(),
            package.package_version().clone(),
            package.package_digest().clone(),
            vec![parsed(InstalledComponentPin::new(
                parsed(ComponentId::parse("component:grant-app")),
                ComponentKind::NativeRustComponent,
                parsed(ComponentVersion::parse("component-version:1")),
                Sha256Digest::from_bytes(b"component-grant-app"),
                parsed(ExecutionIdentity::parse("native:grant-app")),
            ))],
            Sha256Digest::from_bytes(b"components-grant-app"),
            package.capability_manifest_digest().clone(),
        )
        .expect("grant package pin")
    }

    fn grant_install_command(package: &ValidatedPackageManifest) -> InstallationCommand {
        InstallationCommand::install(
            command_id("grant-install"),
            grant_installation_id(),
            grant_tenant(),
            grant_user(),
            grant_package_pin(package),
            InstallationConfiguration::new(&grant_tenant(), Vec::new()).unwrap(),
        )
        .expect("grant install command")
    }

    #[allow(clippy::too_many_arguments)]
    fn grant_admission_evidence(
        installation: &InstallationSnapshot,
        package: &ValidatedPackageManifest,
        registry: &CapabilityRegistry,
        capability: CapabilityId,
        scope: GrantScope,
        confirmation_policy: ConfirmationPolicy,
        snapshot_suffix: &str,
        approval_suffix: &str,
    ) -> GrantAdmissionEvidence {
        GrantAdmissionEvidence::from_authority_bindings(
            grant_snapshot_id(snapshot_suffix),
            grant_approval_id(approval_suffix),
            installation,
            package,
            capability,
            scope,
            confirmation_policy,
            registry,
        )
        .expect("grant admission evidence")
    }

    fn seeded_grant_service() -> MarketApplicationService<
        NullCatalogRepository,
        InMemoryInstallationRepository,
        InMemoryGrantRepository,
        InMemoryPackageUpdateRepository,
    > {
        let package = grant_package();
        let registry = grant_registry();
        let mut installations = InMemoryInstallationRepository::new();
        installations
            .execute(grant_install_command(&package))
            .expect("seed installation");
        let installation = installations
            .load_exact(&grant_installation_id())
            .expect("load seeded installation")
            .expect("seeded installation present");

        let campus_capability = parsed(CapabilityId::parse("campus.public_plan.read"));
        let campus_scope = GrantScope::campus_public().expect("campus scope");
        let campus_evidence = grant_admission_evidence(
            &installation,
            &package,
            &registry,
            campus_capability,
            campus_scope,
            ConfirmationPolicy::Allow,
            "campus-public",
            "campus-public",
        );
        let user_capability = parsed(CapabilityId::parse("user.own_academic_snapshot.read"));
        let user_scope =
            GrantScope::tenant_private_user(grant_tenant(), grant_user()).expect("user scope");
        let user_evidence = grant_admission_evidence(
            &installation,
            &package,
            &registry,
            user_capability,
            user_scope,
            ConfirmationPolicy::Ask,
            "user-private",
            "user-private",
        );

        let mut grants = InMemoryGrantRepository::new();
        grants
            .execute(
                GrantCommand::issue(grant_command_id("issue-campus"), campus_evidence)
                    .expect("campus issue command"),
            )
            .expect("seed campus grant");
        grants
            .execute(
                GrantCommand::issue(grant_command_id("issue-user"), user_evidence)
                    .expect("user issue command"),
            )
            .expect("seed user grant");

        MarketApplicationService::new(
            NullCatalogRepository,
            installations,
            grants,
            InMemoryPackageUpdateRepository::new(),
        )
    }

    #[test]
    fn current_grants_require_exact_installation_revision_and_canonical_order() {
        let service = seeded_grant_service();
        let installation_revision = revision(1);

        let stale = OwnedInstallationGrantQuery::new(
            grant_tenant(),
            grant_user(),
            grant_installation_id(),
            revision(99),
        );
        assert_eq!(
            service.current_grants(&stale),
            Err(MarketApplicationError::Conflict)
        );

        let correct = OwnedInstallationGrantQuery::new(
            grant_tenant(),
            grant_user(),
            grant_installation_id(),
            installation_revision.clone(),
        );
        let page = service
            .current_grants(&correct)
            .expect("current grants resolve");
        assert_eq!(page.installation_id(), &grant_installation_id());
        assert_eq!(
            page.observed_installation_revision(),
            &installation_revision
        );
        assert_eq!(page.grants().len(), 2);

        // Canonical order is the repository's authority-key order: campus public
        // (`campus.public_plan.read`) sorts before tenant-private user
        // (`user.own_academic_snapshot.read`) bytewise on capability id.
        assert_eq!(
            page.grants()[0].capability_id().as_str(),
            "campus.public_plan.read"
        );
        assert_eq!(page.grants()[0].state(), GrantState::Active);
        assert_eq!(
            page.grants()[1].capability_id().as_str(),
            "user.own_academic_snapshot.read"
        );
        assert_eq!(
            page.grants()[1].confirmation_policy(),
            ConfirmationPolicy::Ask
        );

        assert_eq!(page.grants()[0].installation_id(), &grant_installation_id());
        assert_eq!(
            page.grants()[1].installation_revision(),
            &installation_revision
        );
        assert_eq!(page.grants()[0].package_id().as_str(), "ustc.grant-app");
    }

    #[test]
    fn current_grants_hide_foreign_or_absent_authority() {
        let service = seeded_grant_service();

        let absent = OwnedInstallationGrantQuery::new(
            grant_tenant(),
            grant_user(),
            parsed(InstallationId::parse("installation:absent")),
            revision(1),
        );
        assert_eq!(
            service.current_grants(&absent),
            Err(MarketApplicationError::NotFound)
        );

        let foreign_tenant = OwnedInstallationGrantQuery::new(
            parsed(TenantId::parse("tenant:foreign")),
            grant_user(),
            grant_installation_id(),
            revision(1),
        );
        assert_eq!(
            service.current_grants(&foreign_tenant),
            Err(MarketApplicationError::NotFound)
        );

        let foreign_user = OwnedInstallationGrantQuery::new(
            grant_tenant(),
            parsed(UserId::parse("user:foreign")),
            grant_installation_id(),
            revision(1),
        );
        assert_eq!(
            service.current_grants(&foreign_user),
            Err(MarketApplicationError::NotFound)
        );
    }

    #[test]
    fn current_grants_expose_no_approval_or_history_carriers() {
        let service = seeded_grant_service();
        let query = OwnedInstallationGrantQuery::new(
            grant_tenant(),
            grant_user(),
            grant_installation_id(),
            revision(1),
        );
        let page = service
            .current_grants(&query)
            .expect("current grants resolve");
        let campus_approval = "grant-approval:campus-public";
        let user_approval = "grant-approval:user-private";

        for grant in page.grants() {
            let debug = format!("{grant:?}");
            assert!(
                !debug.contains(campus_approval),
                "grant debug leaks campus approval id"
            );
            assert!(
                !debug.contains(user_approval),
                "grant debug leaks user approval id"
            );
            assert!(
                !debug.contains("approval"),
                "grant debug references approval carrier"
            );
            assert!(
                !debug.contains("last_sequence"),
                "grant debug references history carrier"
            );
            assert!(
                !debug.contains("consumed"),
                "grant debug references consumed-approval carrier"
            );
        }

        let page_debug = format!("{page:?}");
        assert!(
            !page_debug.contains(campus_approval),
            "page debug leaks campus approval id"
        );
        assert!(
            !page_debug.contains(user_approval),
            "page debug leaks user approval id"
        );
    }

    #[test]
    fn current_grants_empty_page_observes_exact_revision() {
        let mut installations = InMemoryInstallationRepository::new();
        installations
            .execute(grant_install_command(&grant_package()))
            .expect("seed installation");
        let service = MarketApplicationService::new(
            NullCatalogRepository,
            installations,
            InMemoryGrantRepository::new(),
            InMemoryPackageUpdateRepository::new(),
        );
        let query = OwnedInstallationGrantQuery::new(
            grant_tenant(),
            grant_user(),
            grant_installation_id(),
            revision(1),
        );
        let page = service
            .current_grants(&query)
            .expect("empty grants resolve");
        assert!(page.grants().is_empty());
        assert_eq!(page.installation_id(), &grant_installation_id());
        assert_eq!(page.observed_installation_revision(), &revision(1));
    }

    // Package-update vertical fixtures. Update staging requires
    // `pub(in crate::market)` decision context construction, so the seeded
    // success proof lives here as an internal application test; external tests
    // cover only the public denial surface.

    const EXEC_MARKER: &str = "exec:private-marker";
    const SOURCE_POLICY_MARKER: &str = "source-policy:fixture";

    fn cap_public() -> &'static str {
        "campus.public_rules.read"
    }

    fn manifest(version: &str, source_value: &str) -> ValidatedPackageManifest {
        let source = format!(
            r#"{{"id":"synthetic.update","version":"{version}","publisher":"Nous","tier":"VerifiedRemoteMcp","displayName":"Synthetic","implementationStatus":"implemented","installPolicy":{{"class":"UserInstalledPlugin","defaultInstalled":false,"defaultEnabled":false,"userDisableAllowed":true}},"components":[{{"type":"NativeRustComponent","path":"bin/main","mode":"local"}}],"capabilities":["{cap}"],"sourcePolicy":{{"reviewed":"{src}"}}}}"#,
            cap = cap_public(),
            src = source_value,
        )
        .replace('\\', "");
        load_package_manifest(source.as_bytes()).expect("manifest fixture validates")
    }

    fn registry(rev: &str) -> CapabilityRegistry {
        let source = format!(
            r#"{{"schemaVersion":"capability-registry/v1","registryRevision":"capability-registry:{rev}","capabilities":[{{"id":"{cap}","effectClass":"Read","dataClass":"PublicCampusFact","scopeKind":"CampusPublic","autoGrant":"FirstPartyDefaultOnly","confirmationDefault":"Allow","status":"Active"}}]}}"#,
            cap = cap_public(),
        )
        .replace('\\', "");
        load_capability_registry(source.as_bytes()).expect("registry fixture validates")
    }

    fn component(version: &str, artifact: char) -> CatalogComponentRevision {
        CatalogComponentRevision {
            id: parsed(ComponentId::parse("component:main")),
            kind: ComponentKind::NativeRustComponent,
            version: parsed(ComponentVersion::parse(version)),
            digest: digest(artifact),
            execution_identity: parsed(ExecutionIdentity::parse(EXEC_MARKER)),
            declared_capabilities: BTreeSet::from([parsed(CapabilityId::parse(cap_public()))]),
            tool: None,
        }
    }

    fn pin(
        catalog: &str,
        package: &ValidatedPackageManifest,
        comp: CatalogComponentRevision,
    ) -> InstallationPackagePin {
        parsed(InstallationPackagePin::new(
            parsed(CatalogRevision::parse(catalog)),
            package.package_id().clone(),
            package.package_version().clone(),
            package.package_digest().clone(),
            vec![parsed(InstalledComponentPin::new(
                comp.id,
                comp.kind,
                comp.version,
                comp.digest,
                comp.execution_identity,
            ))],
            digest('c'),
            package.capability_manifest_digest().clone(),
        ))
    }

    fn publication(
        catalog: &str,
        package: &ValidatedPackageManifest,
        comp: CatalogComponentRevision,
    ) -> CatalogPackageRevision {
        CatalogPackageRevision {
            catalog_revision: parsed(CatalogRevision::parse(catalog)),
            package_id: package.package_id().clone(),
            package_version: package.package_version().clone(),
            package_digest: package.package_digest().clone(),
            runnable: true,
            revoked: false,
            capability_manifest_digest: package.capability_manifest_digest().clone(),
            source_policy: Some(crate::invocation::SourcePolicyIdentity {
                id: parsed(SourcePolicyId::parse(SOURCE_POLICY_MARKER)),
                digest: package.source_policy_digest().clone(),
            }),
            component: Some(comp),
        }
    }

    fn catalog(rev: &str, packages: Vec<ValidatedPackageManifest>) -> CatalogReadModel {
        parsed(CatalogReadModel::new(
            parsed(CatalogRevision::parse(rev)),
            packages,
        ))
    }

    fn installation_snapshot(
        package: &ValidatedPackageManifest,
        rollback_component: CatalogComponentRevision,
    ) -> InstallationSnapshot {
        let tenant = parsed(TenantId::parse("tenant:update-app"));
        let user = parsed(UserId::parse("user:update-app"));
        let installation_id = parsed(InstallationId::parse("installation:update-app"));
        let config = parsed(InstallationConfiguration::new(
            &tenant,
            vec![(
                parsed(ConfigurationKey::parse("mode")),
                ConfigurationValue::Text(parsed(NonSecretText::parse("safe"))),
            )],
        ));
        let package_pin = pin("catalog:old", package, rollback_component);
        let command = parsed(InstallationCommand::install(
            parsed(InstallationCommandId::parse("cmd:install-app")),
            installation_id,
            tenant,
            user,
            package_pin,
            config,
        ));
        let event =
            crate::market::installation::decide(None, &command).expect("install event validates");
        crate::market::installation::evolve(None, &event).expect("install snapshot evolves")
    }

    /// Computes the exact policy snapshots required by the rollback and target
    /// authorities, mirroring the private `required_policy_bindings` logic so
    /// the fixture can call `UpdateDecisionContext::for_stage`.
    #[allow(clippy::too_many_arguments)]
    fn policy_snapshots_for_inputs(
        rollback_pin: &InstallationPackagePin,
        rollback_catalog: &CatalogReadModel,
        rollback_publications: &[CatalogPackageRevision],
        rollback_registry: &CapabilityRegistry,
        target_pin: &InstallationPackagePin,
        target_catalog: &CatalogReadModel,
        target_publications: &[CatalogPackageRevision],
        target_registry: &CapabilityRegistry,
    ) -> Vec<InvocationPolicySnapshot> {
        let mut bindings: BTreeMap<(String, String, String, String), Option<CapabilityClass>> =
            BTreeMap::new();
        for (pin, catalog, publications, registry) in [
            (
                rollback_pin,
                rollback_catalog,
                rollback_publications,
                rollback_registry,
            ),
            (
                target_pin,
                target_catalog,
                target_publications,
                target_registry,
            ),
        ] {
            let Some(package) = catalog.find(pin.package_id(), pin.package_version()) else {
                continue;
            };
            for publication in publications {
                let Some(comp) = publication.component.as_ref() else {
                    continue;
                };
                for cap in &comp.declared_capabilities {
                    if !package.capabilities().contains(cap) {
                        continue;
                    }
                    let class = registry
                        .definitions()
                        .iter()
                        .find(|def| def.id() == cap)
                        .and_then(
                            crate::market::capability::CapabilityDefinition::compatibility_class,
                        );
                    if let Some(source) = publication.source_policy.as_ref() {
                        bindings.insert(
                            (
                                cap.as_str().to_owned(),
                                comp.execution_identity.as_str().to_owned(),
                                source.id.as_str().to_owned(),
                                source.digest.as_str().to_owned(),
                            ),
                            class,
                        );
                    }
                }
            }
        }
        bindings
            .into_iter()
            .enumerate()
            .map(
                |(idx, ((cap, exec, src_id, src_digest), class))| InvocationPolicySnapshot {
                    snapshot_id: parsed(PolicySnapshotId::parse(format!(
                        "policy-snapshot:update-{idx}"
                    ))),
                    revision: parsed(PolicyRevision::parse(format!(
                        "policy-revision:update-{idx}"
                    ))),
                    capability_id: parsed(CapabilityId::parse(cap)),
                    capability_class: class,
                    admitted_execution_identity: Some(parsed(ExecutionIdentity::parse(exec))),
                    admitted_source_policy: Some(crate::invocation::SourcePolicyIdentity {
                        id: parsed(SourcePolicyId::parse(src_id)),
                        digest: parsed(Sha256Digest::parse(src_digest)),
                    }),
                    emergency_blocked: false,
                },
            )
            .collect()
    }

    fn staged_snapshot() -> PackageUpdateSnapshot {
        let old = manifest("1.0.0", "same");
        let new = manifest("1.1.0", "same");
        let old_comp = component("component-version:1", '1');
        let new_comp = component("component-version:2", '2');
        let installation = installation_snapshot(&old, old_comp.clone());
        let old_catalog = catalog("catalog:old", vec![old.clone()]);
        let new_catalog = catalog("catalog:new", vec![new.clone()]);
        let old_publications = vec![publication("catalog:old", &old, old_comp)];
        let new_publications = vec![publication("catalog:new", &new, new_comp.clone())];
        let old_registry = registry("old");
        let new_registry = registry("new");
        let target_pin = pin("catalog:new", &new, new_comp);
        let command = parsed(UpdateCommand::stage(
            parsed(UpdateCommandId::parse("update-cmd:stage-app")),
            parsed(PackageUpdateId::parse("update:app-test")),
            &installation,
            target_pin.clone(),
            &old_catalog,
            &old_publications,
            &new_catalog,
            &new_publications,
            &old_registry,
            &new_registry,
        ));
        let policies = policy_snapshots_for_inputs(
            installation.package_pin(),
            &old_catalog,
            &old_publications,
            &old_registry,
            &target_pin,
            &new_catalog,
            &new_publications,
            &new_registry,
        );
        let context = parsed(UpdateDecisionContext::for_stage(
            &command,
            &installation,
            target_pin,
            &old_catalog,
            &old_publications,
            &new_catalog,
            &new_publications,
            &old_registry,
            &new_registry,
            &policies,
        ));
        let event = update_decide(None, &context, &command).expect("stage event validates");
        update_evolve(None, &event).expect("staged snapshot evolves")
    }

    fn update_tenant() -> TenantId {
        parsed(TenantId::parse("tenant:update-app"))
    }

    fn update_user() -> UserId {
        parsed(UserId::parse("user:update-app"))
    }

    fn update_id() -> PackageUpdateId {
        parsed(PackageUpdateId::parse("update:app-test"))
    }

    fn absent_update_id() -> PackageUpdateId {
        parsed(PackageUpdateId::parse("update:absent"))
    }

    /// Test-only fake update repository that returns one pre-built snapshot.
    struct FakeUpdateRepository {
        snapshot: Option<PackageUpdateSnapshot>,
    }

    impl PackageUpdateRepository for FakeUpdateRepository {
        fn execute(
            &mut self,
            _command: UpdateCommand,
        ) -> Result<crate::market::update::UpdateCommandReceipt, UpdateRepositoryError> {
            unreachable!("package_update read vertical does not execute commands")
        }

        fn load_exact(
            &self,
            id: &PackageUpdateId,
        ) -> Result<Option<PackageUpdateSnapshot>, UpdateRepositoryError> {
            match &self.snapshot {
                Some(snapshot) if snapshot.update_id() == id => Ok(Some(snapshot.clone())),
                _ => Ok(None),
            }
        }

        fn event_history(
            &self,
            _id: &PackageUpdateId,
        ) -> Result<Vec<crate::market::update::UpdateEvent>, UpdateRepositoryError> {
            Ok(Vec::new())
        }
    }

    fn service_with_update(
        snapshot: Option<PackageUpdateSnapshot>,
    ) -> MarketApplicationService<
        NullCatalogRepository,
        InMemoryInstallationRepository,
        InMemoryGrantRepository,
        FakeUpdateRepository,
    > {
        MarketApplicationService::new(
            NullCatalogRepository,
            InMemoryInstallationRepository::new(),
            InMemoryGrantRepository::new(),
            FakeUpdateRepository { snapshot },
        )
    }

    #[test]
    fn package_update_read_is_exact_owner_scoped_and_safe() {
        let snapshot = staged_snapshot();
        let service = service_with_update(Some(snapshot));

        let query = OwnedUpdateQuery::new(update_tenant(), update_user(), update_id());
        let view = service
            .package_update(&query)
            .expect("owned update resolves");

        assert_eq!(view.update_id(), &update_id());
        assert_eq!(
            view.installation_id(),
            &parsed(InstallationId::parse("installation:update-app"))
        );
        assert_eq!(view.state(), UpdateState::Staged);
        assert_eq!(
            view.revision(),
            &parsed(UpdateRevision::parse("update-revision:1"))
        );
        assert!(view.applied_installation_revision().is_none());

        let rollback = view.rollback_pin();
        assert_eq!(
            rollback.catalog_revision(),
            &parsed(CatalogRevision::parse("catalog:old"))
        );
        assert_eq!(rollback.package_version().as_str(), "1.0.0");
        assert_eq!(rollback.components().len(), 1);
        assert_eq!(
            rollback.components()[0].version().as_str(),
            "component-version:1"
        );

        let target = view.target_pin();
        assert_eq!(
            target.catalog_revision(),
            &parsed(CatalogRevision::parse("catalog:new"))
        );
        assert_eq!(target.package_version().as_str(), "1.1.0");
        assert_eq!(target.components().len(), 1);
        assert_eq!(
            target.components()[0].version().as_str(),
            "component-version:2"
        );

        assert_eq!(view.change_class(), &UpdateChangeClass::Unchanged);
    }

    #[test]
    fn package_update_absent_and_foreign_are_not_found() {
        let snapshot = staged_snapshot();
        let service = service_with_update(Some(snapshot));

        let absent = OwnedUpdateQuery::new(update_tenant(), update_user(), absent_update_id());
        assert_eq!(
            service.package_update(&absent),
            Err(MarketApplicationError::NotFound)
        );

        let correct = OwnedUpdateQuery::new(update_tenant(), update_user(), update_id());
        assert!(service.package_update(&correct).is_ok());

        let foreign_tenant = OwnedUpdateQuery::new(
            parsed(TenantId::parse("tenant:foreign")),
            update_user(),
            update_id(),
        );
        assert_eq!(
            service.package_update(&foreign_tenant),
            Err(MarketApplicationError::NotFound)
        );

        let foreign_user = OwnedUpdateQuery::new(
            update_tenant(),
            parsed(UserId::parse("user:foreign")),
            update_id(),
        );
        assert_eq!(
            service.package_update(&foreign_user),
            Err(MarketApplicationError::NotFound)
        );

        let empty_service = service_with_update(None);
        assert_eq!(
            empty_service.package_update(&correct),
            Err(MarketApplicationError::NotFound)
        );
    }

    #[test]
    fn package_update_debug_and_views_exclude_private_evidence() {
        let snapshot = staged_snapshot();
        let service = service_with_update(Some(snapshot));

        let query = OwnedUpdateQuery::new(update_tenant(), update_user(), update_id());
        let view = service.package_update(&query).expect("view resolves");
        let debug = format!("{view:?}");

        assert!(
            !debug.contains(EXEC_MARKER),
            "debug leaks ExecutionIdentity"
        );
        assert!(
            !debug.contains(SOURCE_POLICY_MARKER),
            "debug leaks SourcePolicyId"
        );
        assert!(
            !debug.contains("plan_digest"),
            "debug leaks plan digest field"
        );
        assert!(!debug.contains("approval"), "debug leaks approval evidence");
        assert!(
            !debug.contains("readiness"),
            "debug leaks readiness evidence"
        );
        assert!(
            !debug.contains("confirmation"),
            "debug leaks confirmation evidence"
        );
        assert!(
            !debug.contains("rollback_evidence"),
            "debug leaks rollback evidence"
        );

        let rollback_debug = format!("{:?}", view.rollback_pin());
        assert!(
            !rollback_debug.contains(EXEC_MARKER),
            "pin debug leaks ExecutionIdentity"
        );
        let target_debug = format!("{:?}", view.target_pin());
        assert!(
            !target_debug.contains(EXEC_MARKER),
            "pin debug leaks ExecutionIdentity"
        );
    }
}
