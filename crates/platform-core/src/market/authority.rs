//! Transaction-current invocation authority assembly for `market-lifecycle/v0`.
//!
//! This module owns bounded M20-B5 semantic evidence only. It does not issue grants or enable
//! evidence, persist production authority, execute tools, create effect intents or perform I/O.

use crate::identity::{TenantId, UserId};
use crate::invocation::{
    AuthorizedInvocation, CapabilityClass, CapabilityGrantSnapshot, CapabilityId,
    CatalogPackageRevision, ComponentKind, CurrentDenyState, GrantSnapshotId, GrantState,
    InstallationId, InstallationState, InvocationAuthorityCandidate, InvocationAuthorizationError,
    InvocationPolicySnapshot, InvocationResolver, InvocationTarget, ObjectScope,
    PluginInstallationSnapshot, ProjectionResolutionError, ProposedToolCall, ResolvedInvocation,
    ToolProjectionRequest, ToolProjectionSnapshot, authorize_call, preflight_projected_call,
};
use std::cell::Cell;
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AuthorityReadRevision(u64);

impl AuthorityReadRevision {
    pub fn try_from_counter(counter: u64) -> Result<Self, AuthorityRepositoryError> {
        if counter == 0 {
            return Err(AuthorityRepositoryError::CorruptAuthority);
        }
        Ok(Self(counter))
    }

    #[must_use]
    pub const fn counter(self) -> u64 {
        self.0
    }
}

impl fmt::Debug for AuthorityReadRevision {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_tuple("AuthorityReadRevision")
            .field(&self.0)
            .finish()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthorityRepositoryError {
    TransactionConflict,
    RevisionExhausted,
    CorruptAuthority,
    PolicyMissing,
    CurrentGrantMissing,
}

impl fmt::Display for AuthorityRepositoryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "invocation authority repository denied: {self:?}"
        )
    }
}

impl Error for AuthorityRepositoryError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectionAssemblyError {
    Repository(AuthorityRepositoryError),
    Resolution(ProjectionResolutionError),
}

impl fmt::Display for ProjectionAssemblyError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "invocation projection assembly denied: {self:?}")
    }
}

impl Error for ProjectionAssemblyError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Repository(error) => Some(error),
            Self::Resolution(error) => Some(error),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InvocationRecheckError {
    Repository(AuthorityRepositoryError),
    Authorization(InvocationAuthorizationError),
}

impl fmt::Display for InvocationRecheckError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "invocation current-authority recheck denied: {self:?}"
        )
    }
}

impl Error for InvocationRecheckError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Repository(error) => Some(error),
            Self::Authorization(error) => Some(error),
        }
    }
}

pub trait InvocationAuthorityReadTransaction {
    fn revision(&self) -> &AuthorityReadRevision;

    fn load_catalog_for_target(
        &self,
        target: &InvocationTarget,
    ) -> Result<Option<CatalogPackageRevision>, AuthorityRepositoryError>;

    fn load_installation(
        &self,
        installation_id: &InstallationId,
    ) -> Result<Option<PluginInstallationSnapshot>, AuthorityRepositoryError>;

    fn load_current_grant(
        &self,
        tenant_id: &TenantId,
        user_id: &UserId,
        target: &InvocationTarget,
    ) -> Result<Option<CapabilityGrantSnapshot>, AuthorityRepositoryError>;

    fn load_exact_grant(
        &self,
        snapshot_id: &GrantSnapshotId,
    ) -> Result<Option<CapabilityGrantSnapshot>, AuthorityRepositoryError>;

    fn load_policy(
        &self,
        capability_id: &CapabilityId,
    ) -> Result<Option<InvocationPolicySnapshot>, AuthorityRepositoryError>;

    fn verify_precondition(self) -> Result<(), AuthorityRepositoryError>;
}

pub trait InvocationAuthorityRepository {
    type ReadTransaction<'a>: InvocationAuthorityReadTransaction
    where
        Self: 'a;

    fn begin_read(&self) -> Result<Self::ReadTransaction<'_>, AuthorityRepositoryError>;
}

pub struct InvocationAuthorityService<R> {
    repository: R,
}

impl<R> InvocationAuthorityService<R> {
    #[must_use]
    pub const fn new(repository: R) -> Self {
        Self { repository }
    }

    #[must_use]
    pub fn into_repository(self) -> R {
        self.repository
    }
}

impl<R: InvocationAuthorityRepository> InvocationAuthorityService<R> {
    pub fn resolve_projection(
        &self,
        request: ToolProjectionRequest,
        targets: Vec<InvocationTarget>,
    ) -> Result<ToolProjectionSnapshot, ProjectionAssemblyError> {
        let mut unique_targets = BTreeSet::new();
        if targets
            .iter()
            .any(|target| !unique_targets.insert(target.clone()))
        {
            return Err(ProjectionAssemblyError::Repository(
                AuthorityRepositoryError::CorruptAuthority,
            ));
        }

        let transaction = self
            .repository
            .begin_read()
            .map_err(ProjectionAssemblyError::Repository)?;
        let mut candidates = Vec::with_capacity(targets.len());
        for target in targets {
            let catalog = transaction
                .load_catalog_for_target(&target)
                .map_err(ProjectionAssemblyError::Repository)?;
            let installation = transaction
                .load_installation(&target.installation_id)
                .map_err(ProjectionAssemblyError::Repository)?;
            let grant = transaction
                .load_current_grant(&request.tenant_id, &request.user_id, &target)
                .map_err(ProjectionAssemblyError::Repository)?;
            let policy = transaction
                .load_policy(&target.capability_id)
                .map_err(ProjectionAssemblyError::Repository)?
                .ok_or(ProjectionAssemblyError::Repository(
                    AuthorityRepositoryError::PolicyMissing,
                ))?;
            candidates.push(InvocationAuthorityCandidate {
                target,
                catalog,
                installation,
                grant,
                policy,
            });
        }

        let projection = InvocationResolver::resolve_projection(request, candidates)
            .map_err(ProjectionAssemblyError::Resolution)?;
        transaction
            .verify_precondition()
            .map_err(ProjectionAssemblyError::Repository)?;
        Ok(projection)
    }

    /// Authorizes one statically composed first-party application use case from
    /// transaction-current Market package, installation, grant, and policy state.
    ///
    /// This path deliberately produces no tool projection, provider call,
    /// `AgentRun`, `ToolGateway` route, or `PluginExecutor` request. The target's
    /// final identity field is only the repository's exact operation locator for
    /// the bounded M20 carrier; it never becomes an Agent-facing tool identity.
    pub fn authorize_static_application_use_case(
        &self,
        tenant_id: &TenantId,
        user_id: &UserId,
        target: &InvocationTarget,
        capability_class: CapabilityClass,
    ) -> Result<(), InvocationRecheckError> {
        let transaction = self
            .repository
            .begin_read()
            .map_err(InvocationRecheckError::Repository)?;
        let catalog = transaction
            .load_catalog_for_target(target)
            .map_err(InvocationRecheckError::Repository)?
            .ok_or(InvocationRecheckError::Authorization(
                InvocationAuthorizationError::CatalogRevoked,
            ))?;
        let installation = transaction
            .load_installation(&target.installation_id)
            .map_err(InvocationRecheckError::Repository)?
            .ok_or(InvocationRecheckError::Authorization(
                InvocationAuthorizationError::InstallationMissing,
            ))?;
        let grant = transaction
            .load_current_grant(tenant_id, user_id, target)
            .map_err(InvocationRecheckError::Repository)?
            .ok_or(InvocationRecheckError::Repository(
                AuthorityRepositoryError::CurrentGrantMissing,
            ))?;
        let policy = transaction
            .load_policy(&target.capability_id)
            .map_err(InvocationRecheckError::Repository)?
            .ok_or(InvocationRecheckError::Repository(
                AuthorityRepositoryError::PolicyMissing,
            ))?;

        let deny = |error| InvocationRecheckError::Authorization(error);
        if policy.emergency_blocked {
            return Err(deny(InvocationAuthorizationError::EmergencyBlocked));
        }
        if catalog.revoked || !catalog.runnable {
            return Err(deny(InvocationAuthorizationError::CatalogRevoked));
        }
        let component = catalog
            .component
            .as_ref()
            .ok_or_else(|| deny(InvocationAuthorizationError::AuthorityConflict))?;
        if catalog.package_id != target.package_id
            || catalog.package_version != target.package_version
            || component.id != target.component_id
            || component.kind != ComponentKind::DeclarativeResourcePack
            || component.tool.is_some()
            || !component
                .declared_capabilities
                .contains(&target.capability_id)
            || policy.capability_id != target.capability_id
            || policy.capability_class != Some(capability_class)
            || policy.admitted_execution_identity.is_some()
            || policy.admitted_source_policy != catalog.source_policy
        {
            return Err(deny(InvocationAuthorizationError::AuthorityConflict));
        }
        if &installation.tenant_id != tenant_id
            || &installation.user_id != user_id
            || &grant.tenant_id != tenant_id
            || &grant.user_id != user_id
        {
            return Err(deny(
                InvocationAuthorizationError::TenantOrUserScopeMismatch,
            ));
        }
        match installation.state {
            InstallationState::Enabled => {}
            InstallationState::Disabled => {
                return Err(deny(InvocationAuthorizationError::InstallationDisabled));
            }
            InstallationState::Revoked => {
                return Err(deny(InvocationAuthorizationError::InstallationRevoked));
            }
        }
        if installation.id != target.installation_id
            || installation.package_id != catalog.package_id
            || installation.package_version != catalog.package_version
            || installation.package_digest != catalog.package_digest
            || installation.component.id != component.id
            || installation.component.version != component.version
            || installation.component.digest != component.digest
            || installation.component.execution_identity != component.execution_identity
        {
            return Err(deny(
                InvocationAuthorizationError::InstallationRevisionMismatch,
            ));
        }
        match grant.state {
            GrantState::Active => {}
            GrantState::Stale => {
                return Err(deny(InvocationAuthorizationError::GrantStale));
            }
            GrantState::Expired => {
                return Err(deny(InvocationAuthorizationError::GrantExpired));
            }
            GrantState::Revoked => {
                return Err(deny(InvocationAuthorizationError::GrantRevoked));
            }
        }
        if grant.installation_id != installation.id
            || grant.capability_id != target.capability_id
            || grant.object_scope != target.object_scope
            || grant.capability_manifest_digest != catalog.capability_manifest_digest
        {
            return Err(deny(InvocationAuthorizationError::GrantScopeMismatch));
        }

        transaction
            .verify_precondition()
            .map_err(InvocationRecheckError::Repository)
    }

    pub fn recheck_invocation(
        &self,
        projection: &ToolProjectionSnapshot,
        call: ProposedToolCall,
    ) -> Result<AuthorizedInvocation, InvocationRecheckError> {
        let entry = preflight_projected_call(projection, &call)
            .map_err(InvocationRecheckError::Authorization)?;
        let target = invocation_target_from_entry(entry);
        let transaction = self
            .repository
            .begin_read()
            .map_err(InvocationRecheckError::Repository)?;
        let catalog = transaction
            .load_catalog_for_target(&target)
            .map_err(InvocationRecheckError::Repository)?;
        let installation = transaction
            .load_installation(entry.installation_id())
            .map_err(InvocationRecheckError::Repository)?;
        let grant = transaction
            .load_exact_grant(entry.grant_snapshot_id())
            .map_err(InvocationRecheckError::Repository)?
            .ok_or(InvocationRecheckError::Repository(
                AuthorityRepositoryError::CurrentGrantMissing,
            ))?;
        let policy = transaction
            .load_policy(entry.capability_id())
            .map_err(InvocationRecheckError::Repository)?
            .ok_or(InvocationRecheckError::Repository(
                AuthorityRepositoryError::PolicyMissing,
            ))?;
        let current = CurrentDenyState {
            tenant_id: entry.tenant_id().clone(),
            user_id: entry.user_id().clone(),
            catalog_revoked: !catalog_supports_entry(catalog.as_ref(), entry),
            installation,
            grant,
            policy,
        };
        let authorized = authorize_call(projection, current, call)
            .map_err(InvocationRecheckError::Authorization)?;
        transaction
            .verify_precondition()
            .map_err(InvocationRecheckError::Repository)?;
        Ok(authorized)
    }
}

fn invocation_target_from_entry(entry: &ResolvedInvocation) -> InvocationTarget {
    InvocationTarget {
        installation_id: entry.installation_id().clone(),
        package_id: entry.package_id().clone(),
        package_version: entry.package_version().clone(),
        component_id: entry.component_id().clone(),
        tool_id: entry.tool_id().clone(),
        capability_id: entry.capability_id().clone(),
        object_scope: entry.object_scope().clone(),
    }
}

fn catalog_supports_entry(
    catalog: Option<&CatalogPackageRevision>,
    entry: &ResolvedInvocation,
) -> bool {
    let Some(catalog) = catalog else {
        return false;
    };
    if catalog.revoked
        || !catalog.runnable
        || &catalog.package_id != entry.package_id()
        || &catalog.package_version != entry.package_version()
        || &catalog.package_digest != entry.package_digest()
        || &catalog.capability_manifest_digest != entry.capability_manifest_digest()
        || catalog.source_policy.as_ref() != Some(entry.source_policy())
    {
        return false;
    }
    let Some(component) = catalog.component.as_ref() else {
        return false;
    };
    if &component.id != entry.component_id()
        || &component.version != entry.component_version()
        || component.kind != entry.component_kind()
        || &component.digest != entry.component_digest()
        || &component.execution_identity != entry.execution_identity()
        || !component
            .declared_capabilities
            .contains(entry.capability_id())
    {
        return false;
    }
    let Some(tool) = component.tool.as_ref() else {
        return false;
    };
    &tool.id == entry.tool_id()
        && tool.model_visible_name == entry.model_visible_name()
        && &tool.capability_id == entry.capability_id()
        && tool.input_schema.as_ref() == Some(entry.input_schema())
        && &tool.claimed_input_schema_digest == entry.input_schema().digest()
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
struct CurrentGrantKey {
    tenant_id: TenantId,
    user_id: UserId,
    installation_id: InstallationId,
    capability_id: CapabilityId,
    object_scope: ObjectScope,
}

impl CurrentGrantKey {
    fn from_snapshot(snapshot: &CapabilityGrantSnapshot) -> Self {
        Self {
            tenant_id: snapshot.tenant_id.clone(),
            user_id: snapshot.user_id.clone(),
            installation_id: snapshot.installation_id.clone(),
            capability_id: snapshot.capability_id.clone(),
            object_scope: snapshot.object_scope.clone(),
        }
    }

    fn from_request_target(
        tenant_id: &TenantId,
        user_id: &UserId,
        target: &InvocationTarget,
    ) -> Self {
        Self {
            tenant_id: tenant_id.clone(),
            user_id: user_id.clone(),
            installation_id: target.installation_id.clone(),
            capability_id: target.capability_id.clone(),
            object_scope: target.object_scope.clone(),
        }
    }
}

#[derive(Clone)]
struct InMemoryAuthorityState {
    catalogs: BTreeMap<InvocationTarget, CatalogPackageRevision>,
    installations: BTreeMap<InstallationId, PluginInstallationSnapshot>,
    grants: BTreeMap<GrantSnapshotId, CapabilityGrantSnapshot>,
    current_grants: BTreeMap<CurrentGrantKey, GrantSnapshotId>,
    policies: BTreeMap<CapabilityId, InvocationPolicySnapshot>,
}

pub struct InMemoryInvocationAuthorityRepository {
    revision: Cell<u64>,
    state: InMemoryAuthorityState,
    fail_next_precondition: Cell<bool>,
}

impl InMemoryInvocationAuthorityRepository {
    pub fn try_new(
        catalog_records: Vec<(InvocationTarget, CatalogPackageRevision)>,
        installations: Vec<PluginInstallationSnapshot>,
        grants: Vec<CapabilityGrantSnapshot>,
        current_grant_snapshot_ids: Vec<GrantSnapshotId>,
        policies: Vec<InvocationPolicySnapshot>,
    ) -> Result<Self, AuthorityRepositoryError> {
        let mut catalogs = BTreeMap::new();
        for (target, catalog) in catalog_records {
            if catalogs.insert(target, catalog).is_some() {
                return Err(AuthorityRepositoryError::CorruptAuthority);
            }
        }
        let mut installation_map = BTreeMap::new();
        for installation in installations {
            if installation_map
                .insert(installation.id.clone(), installation)
                .is_some()
            {
                return Err(AuthorityRepositoryError::CorruptAuthority);
            }
        }
        let mut grant_map = BTreeMap::new();
        for grant in grants {
            if grant_map.insert(grant.snapshot_id.clone(), grant).is_some() {
                return Err(AuthorityRepositoryError::CorruptAuthority);
            }
        }
        let mut current_grants = BTreeMap::new();
        for snapshot_id in current_grant_snapshot_ids {
            let grant = grant_map
                .get(&snapshot_id)
                .ok_or(AuthorityRepositoryError::CorruptAuthority)?;
            if current_grants
                .insert(CurrentGrantKey::from_snapshot(grant), snapshot_id)
                .is_some()
            {
                return Err(AuthorityRepositoryError::CorruptAuthority);
            }
        }
        let mut policy_map = BTreeMap::new();
        for policy in policies {
            if policy_map
                .insert(policy.capability_id.clone(), policy)
                .is_some()
            {
                return Err(AuthorityRepositoryError::CorruptAuthority);
            }
        }
        Ok(Self {
            revision: Cell::new(1),
            state: InMemoryAuthorityState {
                catalogs,
                installations: installation_map,
                grants: grant_map,
                current_grants,
                policies: policy_map,
            },
            fail_next_precondition: Cell::new(false),
        })
    }

    pub fn fail_next_precondition_for_testing(&self) {
        self.fail_next_precondition.set(true);
    }

    pub fn replace_catalog_for_testing(
        &mut self,
        target: InvocationTarget,
        catalog: Option<CatalogPackageRevision>,
    ) -> Result<(), AuthorityRepositoryError> {
        self.advance_revision()?;
        match catalog {
            Some(catalog) => {
                self.state.catalogs.insert(target, catalog);
            }
            None => {
                self.state.catalogs.remove(&target);
            }
        }
        Ok(())
    }

    pub fn replace_installation_for_testing(
        &mut self,
        installation_id: InstallationId,
        installation: Option<PluginInstallationSnapshot>,
    ) -> Result<(), AuthorityRepositoryError> {
        self.advance_revision()?;
        match installation {
            Some(installation) => {
                self.state
                    .installations
                    .insert(installation_id, installation);
            }
            None => {
                self.state.installations.remove(&installation_id);
            }
        }
        Ok(())
    }

    pub fn replace_grant_for_testing(
        &mut self,
        grant: CapabilityGrantSnapshot,
        make_current: bool,
    ) -> Result<(), AuthorityRepositoryError> {
        self.advance_revision()?;
        let snapshot_id = grant.snapshot_id.clone();
        let was_current = self
            .state
            .current_grants
            .values()
            .any(|current_id| current_id == &snapshot_id);
        self.state
            .current_grants
            .retain(|_, current_id| current_id != &snapshot_id);
        let key = CurrentGrantKey::from_snapshot(&grant);
        self.state.grants.insert(snapshot_id.clone(), grant);
        if was_current || make_current {
            self.state.current_grants.insert(key, snapshot_id);
        }
        Ok(())
    }

    pub fn replace_policy_for_testing(
        &mut self,
        policy: InvocationPolicySnapshot,
    ) -> Result<(), AuthorityRepositoryError> {
        self.advance_revision()?;
        self.state
            .policies
            .insert(policy.capability_id.clone(), policy);
        Ok(())
    }

    fn advance_revision(&self) -> Result<(), AuthorityRepositoryError> {
        let next = self
            .revision
            .get()
            .checked_add(1)
            .ok_or(AuthorityRepositoryError::RevisionExhausted)?;
        self.revision.set(next);
        Ok(())
    }
}

impl fmt::Debug for InMemoryInvocationAuthorityRepository {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("InMemoryInvocationAuthorityRepository")
            .field("revision", &self.revision.get())
            .field("catalog_records", &self.state.catalogs.len())
            .field("installation_records", &self.state.installations.len())
            .field("grant_records", &self.state.grants.len())
            .field("current_grant_records", &self.state.current_grants.len())
            .field("policy_records", &self.state.policies.len())
            .field("authority_payload", &"[REDACTED]")
            .finish()
    }
}

pub struct InMemoryAuthorityReadTransaction<'a> {
    revision: AuthorityReadRevision,
    state: InMemoryAuthorityState,
    current_revision: &'a Cell<u64>,
    fail_precondition: bool,
}

impl fmt::Debug for InMemoryAuthorityReadTransaction<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("InMemoryAuthorityReadTransaction")
            .field("revision", &self.revision)
            .field("authority_payload", &"[REDACTED]")
            .finish()
    }
}

impl InvocationAuthorityReadTransaction for InMemoryAuthorityReadTransaction<'_> {
    fn revision(&self) -> &AuthorityReadRevision {
        &self.revision
    }

    fn load_catalog_for_target(
        &self,
        target: &InvocationTarget,
    ) -> Result<Option<CatalogPackageRevision>, AuthorityRepositoryError> {
        Ok(self.state.catalogs.get(target).cloned())
    }

    fn load_installation(
        &self,
        installation_id: &InstallationId,
    ) -> Result<Option<PluginInstallationSnapshot>, AuthorityRepositoryError> {
        Ok(self.state.installations.get(installation_id).cloned())
    }

    fn load_current_grant(
        &self,
        tenant_id: &TenantId,
        user_id: &UserId,
        target: &InvocationTarget,
    ) -> Result<Option<CapabilityGrantSnapshot>, AuthorityRepositoryError> {
        let key = CurrentGrantKey::from_request_target(tenant_id, user_id, target);
        let Some(snapshot_id) = self.state.current_grants.get(&key) else {
            return Ok(None);
        };
        self.state
            .grants
            .get(snapshot_id)
            .cloned()
            .map(Some)
            .ok_or(AuthorityRepositoryError::CorruptAuthority)
    }

    fn load_exact_grant(
        &self,
        snapshot_id: &GrantSnapshotId,
    ) -> Result<Option<CapabilityGrantSnapshot>, AuthorityRepositoryError> {
        Ok(self.state.grants.get(snapshot_id).cloned())
    }

    fn load_policy(
        &self,
        capability_id: &CapabilityId,
    ) -> Result<Option<InvocationPolicySnapshot>, AuthorityRepositoryError> {
        Ok(self.state.policies.get(capability_id).cloned())
    }

    fn verify_precondition(self) -> Result<(), AuthorityRepositoryError> {
        if self.fail_precondition || self.current_revision.get() != self.revision.counter() {
            return Err(AuthorityRepositoryError::TransactionConflict);
        }
        Ok(())
    }
}

impl InvocationAuthorityRepository for InMemoryInvocationAuthorityRepository {
    type ReadTransaction<'a> = InMemoryAuthorityReadTransaction<'a>;

    fn begin_read(&self) -> Result<Self::ReadTransaction<'_>, AuthorityRepositoryError> {
        Ok(InMemoryAuthorityReadTransaction {
            revision: AuthorityReadRevision::try_from_counter(self.revision.get())?,
            state: self.state.clone(),
            current_revision: &self.revision,
            fail_precondition: self.fail_next_precondition.replace(false),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::invocation::{
        CapabilityClass, CatalogRevision, ComponentId, GrantState, GrantVersion, ObjectScope,
        PackageId, PackageVersion, PolicyRevision, PolicySnapshotId, RunId, Sha256Digest, ToolId,
        TurnId,
    };

    macro_rules! parsed {
        ($kind:ty, $value:expr) => {{
            match <$kind>::parse($value) {
                Ok(value) => value,
                Err(error) => panic!("fixture value must parse: {error}"),
            }
        }};
    }

    fn digest(byte: char) -> Sha256Digest {
        parsed!(
            Sha256Digest,
            format!("sha256:{}", byte.to_string().repeat(64))
        )
    }

    fn grant() -> CapabilityGrantSnapshot {
        CapabilityGrantSnapshot {
            snapshot_id: parsed!(GrantSnapshotId, "grant:authority-unit"),
            version: parsed!(GrantVersion, "grant-version:1"),
            tenant_id: parsed!(TenantId, "tenant:synthetic"),
            user_id: parsed!(UserId, "user:synthetic"),
            installation_id: parsed!(InstallationId, "installation:synthetic"),
            capability_id: parsed!(CapabilityId, "campus.public_rules.read"),
            object_scope: parsed!(ObjectScope, "scope:campus-public"),
            confirmation_policy: crate::invocation::ConfirmationPolicy::Allow,
            capability_manifest_digest: digest('1'),
            state: GrantState::Active,
        }
    }

    fn policy() -> InvocationPolicySnapshot {
        InvocationPolicySnapshot {
            snapshot_id: parsed!(PolicySnapshotId, "policy:authority-unit"),
            revision: parsed!(PolicyRevision, "policy-revision:1"),
            capability_id: parsed!(CapabilityId, "campus.public_rules.read"),
            capability_class: Some(CapabilityClass::PublicRead),
            admitted_execution_identity: None,
            admitted_source_policy: None,
            emergency_blocked: false,
        }
    }

    fn target() -> InvocationTarget {
        InvocationTarget {
            installation_id: parsed!(InstallationId, "installation:synthetic"),
            package_id: parsed!(PackageId, "synthetic.resolver"),
            package_version: parsed!(PackageVersion, "1.2.3"),
            component_id: parsed!(ComponentId, "component:resolver"),
            tool_id: parsed!(ToolId, "tool:search"),
            capability_id: parsed!(CapabilityId, "campus.public_rules.read"),
            object_scope: parsed!(ObjectScope, "scope:campus-public"),
        }
    }

    fn empty_repository() -> InMemoryInvocationAuthorityRepository {
        InMemoryInvocationAuthorityRepository::try_new(
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
        )
        .expect("empty semantic fake is coherent")
    }

    #[test]
    fn repository_rejects_duplicate_keys_and_incoherent_current_index() {
        let grant = grant();
        assert_eq!(
            InMemoryInvocationAuthorityRepository::try_new(
                Vec::new(),
                Vec::new(),
                vec![grant.clone(), grant.clone()],
                Vec::new(),
                Vec::new(),
            )
            .expect_err("duplicate grant IDs must fail"),
            AuthorityRepositoryError::CorruptAuthority
        );
        let policy = policy();
        assert_eq!(
            InMemoryInvocationAuthorityRepository::try_new(
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
                vec![policy.clone(), policy],
            )
            .expect_err("duplicate policy IDs must fail"),
            AuthorityRepositoryError::CorruptAuthority
        );
        assert_eq!(
            InMemoryInvocationAuthorityRepository::try_new(
                Vec::new(),
                Vec::new(),
                Vec::new(),
                vec![grant.snapshot_id],
                Vec::new(),
            )
            .expect_err("current index must reference an exact grant"),
            AuthorityRepositoryError::CorruptAuthority
        );
    }

    #[test]
    fn transaction_loads_separate_carriers_under_one_revision() {
        let grant = grant();
        let policy = policy();
        let repository = InMemoryInvocationAuthorityRepository::try_new(
            Vec::new(),
            Vec::new(),
            vec![grant.clone()],
            vec![grant.snapshot_id.clone()],
            vec![policy.clone()],
        )
        .expect("coherent repository");
        let transaction = repository.begin_read().expect("read transaction");
        assert_eq!(transaction.revision().counter(), 1);
        assert_eq!(
            transaction
                .load_current_grant(&grant.tenant_id, &grant.user_id, &target())
                .expect("current grant read"),
            Some(grant)
        );
        assert_eq!(
            transaction
                .load_policy(&policy.capability_id)
                .expect("policy read"),
            Some(policy)
        );
        assert_eq!(
            transaction
                .load_catalog_for_target(&target())
                .expect("catalog read"),
            None
        );
        transaction
            .verify_precondition()
            .expect("stable revision must verify");
    }

    #[test]
    fn conflict_revision_exhaustion_and_debug_are_fail_closed() {
        let repository = empty_repository();
        let rendered = format!("{repository:?}");
        assert!(rendered.contains("[REDACTED]"));
        assert!(!rendered.contains("tenant:synthetic"));
        repository.fail_next_precondition_for_testing();
        assert_eq!(
            repository
                .begin_read()
                .expect("transaction")
                .verify_precondition(),
            Err(AuthorityRepositoryError::TransactionConflict)
        );

        repository.revision.set(u64::MAX);
        assert_eq!(
            repository.advance_revision(),
            Err(AuthorityRepositoryError::RevisionExhausted)
        );
    }

    #[test]
    fn adopted_empty_projection_denial_precedes_pending_conflict() {
        let repository = empty_repository();
        repository.fail_next_precondition_for_testing();
        let service = InvocationAuthorityService::new(repository);
        let request = ToolProjectionRequest {
            tenant_id: parsed!(TenantId, "tenant:synthetic"),
            user_id: parsed!(UserId, "user:synthetic"),
            run_id: parsed!(RunId, "run:authority-unit"),
            turn_id: parsed!(TurnId, "turn:authority-unit"),
            activation_allowlist: None,
        };
        let result = service.resolve_projection(request, Vec::new());
        assert_eq!(
            result,
            Err(ProjectionAssemblyError::Resolution(
                ProjectionResolutionError::InvalidRequest
            ))
        );
    }

    #[test]
    fn catalog_key_multiplicity_is_rejected_without_deriving_authority() {
        let target = target();
        let catalog = CatalogPackageRevision {
            catalog_revision: parsed!(CatalogRevision, "catalog:authority-unit"),
            package_id: target.package_id.clone(),
            package_version: target.package_version.clone(),
            package_digest: digest('2'),
            runnable: false,
            revoked: true,
            capability_manifest_digest: digest('3'),
            source_policy: None,
            component: None,
        };
        assert_eq!(
            InMemoryInvocationAuthorityRepository::try_new(
                vec![(target.clone(), catalog.clone()), (target, catalog)],
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
            )
            .expect_err("duplicate catalog target must fail"),
            AuthorityRepositoryError::CorruptAuthority
        );
    }
}
