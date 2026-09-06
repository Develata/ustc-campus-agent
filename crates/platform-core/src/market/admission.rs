//! LC017 trusted-composition admission for one-component reviewed packages.
//!
//! Callers supply authenticated identities and reviewed catalog/registry inputs, and
//! serialize repository reads and writes under their existing lifecycle transaction.
//! Checked readiness proves input consistency, not parser/probe provenance. Neither
//! browser booleans nor model output may enter this boundary as admission authority.

use super::{
    ValidatedPackageManifest,
    capability::{CapabilityRegistry, CapabilityStatus, ScopeKind},
    configuration_binding::ComponentConfigurationBinding,
    configuration_catalog::ValidatedPackageConfiguration,
    grant::*,
    installation::*,
};
use crate::{
    identity::{TenantId, UserId},
    invocation::{
        CapabilityId, CatalogToolDefinition, ComponentKind, ConfirmationPolicy, GrantSnapshotId,
        GrantState, InstallationId, InstallationRevision, Sha256Digest,
    },
};
use std::{collections::BTreeSet, fmt};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdmissionError {
    UnsupportedPackage,
    PackageMismatch,
    InstallationMissing,
    OwnerMismatch,
    RevisionMismatch,
    InvalidState,
    InvalidConfiguration,
    InvalidReadiness,
    InvalidInventory,
    InventoryNotReviewed,
    InvalidCapability,
    MissingActiveGrant,
    InvalidGrant,
    RepositoryFailure,
    CommandRejected,
}
impl fmt::Display for AdmissionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "market admission rejected: {self:?}")
    }
}
impl std::error::Error for AdmissionError {}

/// Private checked readiness; inputs must come from trusted parser/probe composition.
#[derive(Clone)]
pub struct ComponentReadiness {
    binding: ComponentConfigurationBinding,
    configuration_digest: Sha256Digest,
    capabilities: BTreeSet<CapabilityId>,
    digest: Sha256Digest,
}
impl fmt::Debug for ComponentReadiness {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ComponentReadiness").finish_non_exhaustive()
    }
}
impl ComponentReadiness {
    pub fn skill(
        binding: &ComponentConfigurationBinding,
        configuration: &InstallationConfiguration,
        verified_artifact_digest: &Sha256Digest,
    ) -> Result<Self, AdmissionError> {
        let pin = single_component(binding, configuration)?;
        if pin.kind() != ComponentKind::SkillComponent || pin.digest() != verified_artifact_digest {
            return Err(AdmissionError::InvalidReadiness);
        }
        let mut bytes = readiness_bytes(binding, configuration);
        encode("skill", &mut bytes);
        encode(verified_artifact_digest.as_str(), &mut bytes);
        Ok(Self {
            binding: binding.clone(),
            configuration_digest: configuration.digest().clone(),
            capabilities: BTreeSet::new(),
            digest: Sha256Digest::from_bytes(&bytes),
        })
    }
    pub fn mcp(
        binding: &ComponentConfigurationBinding,
        configuration: &InstallationConfiguration,
        tools: &[CatalogToolDefinition],
        reviewed_inventory_digest: &Sha256Digest,
    ) -> Result<Self, AdmissionError> {
        let digest = mcp_inventory_digest(binding, configuration, tools)?;
        if &digest != reviewed_inventory_digest {
            return Err(AdmissionError::InventoryNotReviewed);
        }
        Ok(Self {
            binding: binding.clone(),
            configuration_digest: configuration.digest().clone(),
            capabilities: tools
                .iter()
                .map(|tool| tool.capability_id.clone())
                .collect(),
            digest,
        })
    }
    #[must_use]
    pub fn digest(&self) -> &Sha256Digest {
        &self.digest
    }
}

/// Deterministic inventory to persist and show before exact user review. No authority.
pub fn mcp_inventory_digest(
    binding: &ComponentConfigurationBinding,
    configuration: &InstallationConfiguration,
    tools: &[CatalogToolDefinition],
) -> Result<Sha256Digest, AdmissionError> {
    if single_component(binding, configuration)?.kind() != ComponentKind::McpServerComponent
        || tools.is_empty()
        || tools.len() > 64
    {
        return Err(AdmissionError::InvalidInventory);
    }
    let mut names = BTreeSet::new();
    let mut ids = BTreeSet::new();
    let mut sorted = Vec::with_capacity(tools.len());
    for tool in tools {
        let schema = tool
            .input_schema
            .as_ref()
            .ok_or(AdmissionError::InvalidInventory)?;
        if !ustc_agent_tool_protocol::is_valid_tool_name(&tool.model_visible_name)
            || tool.description.trim().is_empty()
            || tool.description.len() > 4096
            || !names.insert(tool.model_visible_name.as_str())
            || !ids.insert(&tool.id)
            || schema.digest() != &tool.claimed_input_schema_digest
        {
            return Err(AdmissionError::InvalidInventory);
        }
        sorted.push(tool);
    }
    sorted.sort_by(|left, right| left.id.cmp(&right.id));
    let mut bytes = readiness_bytes(binding, configuration);
    encode("mcp-inventory/v1", &mut bytes);
    bytes.extend_from_slice(&(sorted.len() as u64).to_be_bytes());
    for tool in sorted {
        for value in [
            tool.id.as_str(),
            &tool.model_visible_name,
            &tool.description,
            tool.capability_id.as_str(),
            tool.claimed_input_schema_digest.as_str(),
        ] {
            encode(value, &mut bytes);
        }
    }
    Ok(Sha256Digest::from_bytes(&bytes))
}

fn single_component<'a>(
    binding: &'a ComponentConfigurationBinding,
    configuration: &InstallationConfiguration,
) -> Result<&'a InstalledComponentPin, AdmissionError> {
    let [component] = binding.package_pin().components() else {
        return Err(AdmissionError::UnsupportedPackage);
    };
    binding
        .validate(binding.package_pin(), configuration)
        .map_err(|_| AdmissionError::InvalidConfiguration)?;
    Ok(component)
}
fn readiness_bytes(
    binding: &ComponentConfigurationBinding,
    configuration: &InstallationConfiguration,
) -> Vec<u8> {
    let pin = binding.package_pin();
    let mut bytes = b"market-component-readiness/v1\0".to_vec();
    for value in [
        pin.catalog_revision().as_str(),
        pin.package_id().as_str(),
        &pin.package_version().as_str(),
        pin.package_digest().as_str(),
        pin.component_set_digest().as_str(),
        pin.capability_manifest_digest().as_str(),
        binding.component_id().as_str(),
        binding.schema().digest().as_str(),
        configuration.digest().as_str(),
    ] {
        encode(value, &mut bytes);
    }
    for component in pin.components() {
        for value in [
            component.component_id().as_str(),
            component.version().as_str(),
            component.digest().as_str(),
            component.execution_identity().as_str(),
        ] {
            encode(value, &mut bytes);
        }
    }
    bytes
}
fn encode(value: &str, bytes: &mut Vec<u8>) {
    bytes.extend_from_slice(&(value.len() as u64).to_be_bytes());
    bytes.extend_from_slice(value.as_bytes());
}

/// Typed intent from an explicit authenticated approval flow, never model output.
pub struct GrantAdmissionRequest {
    pub installation_id: InstallationId,
    pub expected_revision: InstallationRevision,
    pub command_id: GrantCommandId,
    pub approval_id: GrantApprovalId,
    pub snapshot_id: GrantSnapshotId,
    pub capability_id: CapabilityId,
    pub confirmation_policy: ConfirmationPolicy,
}
pub struct EnableAdmissionRequest {
    pub installation_id: InstallationId,
    pub expected_revision: InstallationRevision,
    pub command_id: InstallationCommandId,
}

/// M20 validates current authority and executes through the original repositories.
pub struct MarketAdmissionService<'a> {
    tenant: &'a TenantId,
    user: &'a UserId,
    package: &'a ValidatedPackageManifest,
    configuration: &'a ValidatedPackageConfiguration,
    registry: &'a CapabilityRegistry,
}
impl<'a> MarketAdmissionService<'a> {
    pub fn new(
        tenant: &'a TenantId,
        user: &'a UserId,
        package: &'a ValidatedPackageManifest,
        configuration: &'a ValidatedPackageConfiguration,
        registry: &'a CapabilityRegistry,
    ) -> Result<Self, AdmissionError> {
        let pin = configuration.package_pin();
        if package.components().len() != 1
            || pin.components().len() != 1
            || configuration.bindings().len() != 1
        {
            return Err(AdmissionError::UnsupportedPackage);
        }
        if package.package_id() != pin.package_id()
            || package.package_version() != pin.package_version()
            || package.package_digest() != pin.package_digest()
            || package.component_declaration_set_digest() != pin.component_set_digest()
            || package.capability_manifest_digest() != pin.capability_manifest_digest()
        {
            return Err(AdmissionError::PackageMismatch);
        }
        let service = Self {
            tenant,
            user,
            package,
            configuration,
            registry,
        };
        for capability in package.capabilities() {
            service.scope(capability)?;
        }
        Ok(service)
    }

    pub fn issue_grant(
        &self,
        installations: &impl InstallationRepository,
        grants: &mut impl GrantRepository,
        request: GrantAdmissionRequest,
    ) -> Result<GrantCommandReceipt, AdmissionError> {
        let installation = self.current(
            installations,
            &request.installation_id,
            &request.expected_revision,
        )?;
        let scope = self.scope(&request.capability_id)?;
        let evidence = GrantAdmissionEvidence::from_authority_bindings(
            request.snapshot_id,
            request.approval_id,
            &installation,
            self.package,
            request.capability_id,
            scope,
            request.confirmation_policy,
            self.registry,
        )
        .map_err(|_| AdmissionError::InvalidGrant)?;
        let command = GrantCommand::issue(request.command_id, evidence)
            .map_err(|_| AdmissionError::InvalidGrant)?;
        grants
            .execute(command)
            .map_err(|_| AdmissionError::RepositoryFailure)
    }

    pub fn enable(
        &self,
        installations: &mut impl InstallationRepository,
        grants: &impl GrantRepository,
        request: EnableAdmissionRequest,
        readiness: &ComponentReadiness,
    ) -> Result<InstallationCommandReceipt, AdmissionError> {
        let installation = self.current(
            installations,
            &request.installation_id,
            &request.expected_revision,
        )?;
        if !matches!(
            installation.state(),
            ManagedInstallationState::InstalledDisabled | ManagedInstallationState::Disabled
        ) {
            return Err(AdmissionError::InvalidState);
        }
        let binding = self
            .configuration
            .bindings()
            .values()
            .next()
            .ok_or(AdmissionError::UnsupportedPackage)?;
        if &readiness.binding != binding
            || readiness.configuration_digest != *installation.configuration().digest()
        {
            return Err(AdmissionError::InvalidReadiness);
        }
        for capability in &readiness.capabilities {
            self.scope(capability)?;
        }
        let set = grants
            .load_current_for_installation(
                self.tenant,
                self.user,
                &request.installation_id,
                &request.expected_revision,
            )
            .map_err(|_| AdmissionError::RepositoryFailure)?;
        if !set.is_canonical()
            || set.tenant_id() != self.tenant
            || set.user_id() != self.user
            || set.installation_id() != &request.installation_id
            || set.observed_installation_revision() != &request.expected_revision
        {
            return Err(AdmissionError::InvalidGrant);
        }
        for capability in self.package.capabilities() {
            let scope = self.scope(capability)?;
            let definition = self
                .registry
                .find(capability)
                .ok_or(AdmissionError::InvalidCapability)?;
            let mut matching = set
                .grants()
                .iter()
                .filter(|grant| grant.capability_id() == capability && grant.scope() == &scope);
            let grant = matching.next().ok_or(AdmissionError::MissingActiveGrant)?;
            if matching.next().is_some()
                || grant.state() != GrantState::Active
                || grant.tenant_id() != self.tenant
                || grant.user_id() != self.user
                || grant.installation_id() != installation.installation_id()
                || grant.installation_revision() != installation.revision()
                || grant.catalog_revision() != installation.package_pin().catalog_revision()
                || grant.package_id() != self.package.package_id()
                || grant.package_version() != self.package.package_version()
                || grant.package_digest() != self.package.package_digest()
                || grant.capability_manifest_digest() != self.package.capability_manifest_digest()
                || grant.capability_registry_revision() != self.registry.registry_revision()
                || grant.capability_definition_digest() != definition.definition_digest()
                || grant.capability_definition() != definition
                || (grant.confirmation_policy() == ConfirmationPolicy::Allow
                    && definition.confirmation_default() == ConfirmationPolicy::Ask)
            {
                return Err(AdmissionError::InvalidGrant);
            }
        }
        let pin = installation.package_pin();
        let evidence = EnablePreconditionEvidence::from_authority_bindings(
            request.installation_id.clone(),
            request.expected_revision.clone(),
            pin.package_digest().clone(),
            pin.component_set_digest().clone(),
            installation.configuration().digest().clone(),
            pin.capability_manifest_digest().clone(),
            set.grant_set_digest().clone(),
            readiness.digest().clone(),
        )
        .map_err(|_| AdmissionError::InvalidReadiness)?;
        let command = InstallationCommand::enable(
            request.command_id,
            request.installation_id,
            request.expected_revision,
            evidence,
        )
        .map_err(|_| AdmissionError::CommandRejected)?;
        installations
            .execute(command)
            .map_err(|_| AdmissionError::RepositoryFailure)
    }

    fn current(
        &self,
        installations: &impl InstallationRepository,
        id: &InstallationId,
        revision: &InstallationRevision,
    ) -> Result<InstallationSnapshot, AdmissionError> {
        let installation = installations
            .load_exact(id)
            .map_err(|_| AdmissionError::RepositoryFailure)?
            .ok_or(AdmissionError::InstallationMissing)?;
        if installation.tenant_id() != self.tenant || installation.user_id() != self.user {
            return Err(AdmissionError::OwnerMismatch);
        }
        if installation.revision() != revision {
            return Err(AdmissionError::RevisionMismatch);
        }
        if matches!(
            installation.state(),
            ManagedInstallationState::Revoked | ManagedInstallationState::Uninstalled
        ) {
            return Err(AdmissionError::InvalidState);
        }
        if installation.package_pin() != self.configuration.package_pin() {
            return Err(AdmissionError::PackageMismatch);
        }
        for binding in self.configuration.bindings().values() {
            binding
                .validate(installation.package_pin(), installation.configuration())
                .map_err(|_| AdmissionError::InvalidConfiguration)?;
        }
        Ok(installation)
    }
    fn scope(&self, capability: &CapabilityId) -> Result<GrantScope, AdmissionError> {
        if !self.package.capabilities().contains(capability) {
            return Err(AdmissionError::InvalidCapability);
        }
        let definition = self
            .registry
            .find(capability)
            .ok_or(AdmissionError::InvalidCapability)?;
        if definition.status() != CapabilityStatus::Active {
            return Err(AdmissionError::InvalidCapability);
        }
        match definition.scope_kind() {
            ScopeKind::CampusPublic => GrantScope::campus_public(),
            ScopeKind::TenantPrivateUser => {
                GrantScope::tenant_private_user(self.tenant.clone(), self.user.clone())
            }
            ScopeKind::OperatorAdministrative => return Err(AdmissionError::InvalidCapability),
        }
        .map_err(|_| AdmissionError::InvalidCapability)
    }
}

#[cfg(test)]
mod tests;
