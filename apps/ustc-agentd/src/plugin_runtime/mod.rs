//! Application owner of package lifecycle transactions and replaceable runtime caches.
//! M20 repositories alone own installations/grants; cached discoveries never grant access.
#[cfg(test)]
mod capacity_tests;
mod commands;
#[cfg(test)]
mod guard_tests;
mod import_review;
mod invocation;
mod persistence;
mod probe;
mod registry;
mod retire;
mod skill_context;
#[cfg(test)]
mod skill_context_tests;
#[cfg(test)]
mod tests;
mod updates;

use crate::chat_tools::ChatDynamicToolDefinition;
use registry::{RuntimeComponent, RuntimePackage};
use std::{collections::BTreeMap, path::PathBuf, sync::Arc};
use tokio::sync::Mutex;
use ustc_campus_agent_adapters::mcp::McpClient;
use ustc_campus_agent_client_protocol::plugins::*;
use ustc_campus_agent_core::{
    identity::{TenantId, UserId},
    invocation::*,
    market::{
        admission::ComponentReadiness,
        capability::{CapabilityRegistry, ScopeKind, load_capability_registry},
        grant::*,
        installation::*,
    },
};

const MAX_PLUGIN_TOOLS: usize = 28;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PluginError {
    InvalidRequest,
    NotFound,
    Conflict,
    Denied,
    Unsupported,
    NotReady,
    Unavailable,
    Capacity,
}
#[derive(Clone)]
struct AuthorityState {
    installations: InMemoryInstallationRepository,
    grants: InMemoryGrantRepository,
    runs: Vec<invocation::JournalRun>,
    updates: ustc_campus_agent_core::market::update::application::UpdateJournal,
}
impl Default for AuthorityState {
    fn default() -> Self {
        Self {
            installations: InMemoryInstallationRepository::new(),
            grants: InMemoryGrantRepository::new(),
            runs: Vec::new(),
            updates: Default::default(),
        }
    }
}
struct RuntimeState {
    authority: AuthorityState,
    disk: persistence::Disk,
    poisoned: bool,
    probes: BTreeMap<InstallationId, ProbedComponent>,
}
struct ProbedComponent {
    component_id: ComponentId,
    additional: BTreeMap<ComponentId, ProbedComponent>,
    tool_components: BTreeMap<String, ComponentId>,
    revision: InstallationRevision,
    readiness: ComponentReadiness,
    tools: Vec<CatalogToolDefinition>,
    wire_names: BTreeMap<String, String>,
    client: Option<McpClient>,
    transport_digest: Option<Sha256Digest>,
}
/// Gateway-owned authority snapshot. Only neutral definitions cross into the Agent.
#[derive(Clone)]
pub(crate) struct PluginToolSession {
    runtime: std::sync::Weak<Mutex<RuntimeState>>,
    tenant: TenantId,
    user: UserId,
    definitions: Vec<ChatDynamicToolDefinition>,
    bindings: BTreeMap<String, FrozenToolBinding>,
}
#[derive(Clone)]
struct FrozenToolBinding {
    installation_id: InstallationId,
    component_id: ComponentId,
    installation_revision: InstallationRevision,
    readiness_digest: Sha256Digest,
    grant_snapshot_id: GrantSnapshotId,
    grant_version: GrantVersion,
    tool: CatalogToolDefinition,
}
impl PluginToolSession {
    pub(crate) fn definitions(&self) -> Vec<ChatDynamicToolDefinition> {
        self.definitions.clone()
    }
}
#[derive(Clone)]
pub(crate) struct PluginRuntime {
    state: Arc<Mutex<RuntimeState>>,
    packages: Arc<Vec<RuntimePackage>>,
    registry: Arc<CapabilityRegistry>,
}
impl PluginRuntime {
    pub(crate) fn open(path: PathBuf) -> Result<Self, PluginError> {
        let mut packages =
            vec![RuntimePackage::bundled_skill().map_err(|_| PluginError::Unavailable)?];
        if let Some(paths) = std::env::var_os("UCA_PLUGIN_PACKAGE_DIRS") {
            for root in std::env::split_paths(&paths) {
                if packages.len() >= 16 {
                    return Err(PluginError::Capacity);
                }
                packages.push(RuntimePackage::load(&root).map_err(|_| PluginError::Unavailable)?);
            }
        }
        Self::with_packages(path, packages)
    }
    fn with_packages(path: PathBuf, packages: Vec<RuntimePackage>) -> Result<Self, PluginError> {
        let mut unique = std::collections::BTreeSet::new();
        if packages.iter().any(|p| {
            !unique.insert((
                p.manifest.package_id().clone(),
                p.manifest.package_version().clone(),
            ))
        }) {
            return Err(PluginError::InvalidRequest);
        }
        let registry = load_capability_registry(include_bytes!(
            "../../../../market/capabilities/registry.json"
        ))
        .map_err(|_| PluginError::Unavailable)?;
        let (disk, authority) = persistence::Disk::open(path)?;
        Ok(Self {
            state: Arc::new(Mutex::new(RuntimeState {
                authority,
                disk,
                poisoned: false,
                probes: BTreeMap::new(),
            })),
            packages: Arc::new(packages),
            registry: Arc::new(registry),
        })
    }
    fn package(&self, id: &str, version: &str) -> Result<&RuntimePackage, PluginError> {
        self.packages
            .iter()
            .find(|p| {
                p.manifest.package_id().as_str() == id
                    && p.manifest.package_version().as_str() == version
            })
            .ok_or(PluginError::NotFound)
    }
    pub(crate) async fn list(
        &self,
        tenant: &TenantId,
        user: &UserId,
    ) -> Result<PluginLifecycleDto, PluginError> {
        let state = self.state.lock().await;
        state.check()?;
        let installations = state.authority.installations.list_owned(tenant, user);
        let mut packages = Vec::new();
        for package in self.packages.iter() {
            let pin = package.configuration.package_pin();
            let snapshot = installations.iter().find(|s| s.package_pin() == pin);
            let installation = snapshot
                .map(|s| installation_view(&state.authority, s))
                .transpose()?;
            let binding = package
                .configuration
                .bindings()
                .values()
                .next()
                .ok_or(PluginError::Unsupported)?;
            let fields = binding
                .schema()
                .fields()
                .values()
                .map(|field| PluginConfigurationFieldDto {
                    key: field.key().as_str().to_owned(),
                    kind: format!("{:?}", field.kind()).to_lowercase(),
                    required: field.required(),
                    max_bytes: field.max_utf8_bytes(),
                    integer_bounds: field.integer_bounds(),
                })
                .collect();
            packages.push(PluginManagedPackageDto {
                available: true,
                package_id: pin.package_id().as_str().to_owned(),
                version: pin.package_version().as_str(),
                name: package.manifest.display_name().to_owned(),
                description: package.manifest.description().unwrap_or("").to_owned(),
                catalog_revision: pin.catalog_revision().as_str().to_owned(),
                package_digest: pin.package_digest().as_str().to_owned(),
                kind: if !package.additional.is_empty() {
                    "mixed"
                } else {
                    match &package.component {
                        RuntimeComponent::Skill { .. } => "skill",
                        RuntimeComponent::Mcp { .. } => "mcp",
                    }
                }
                .to_owned(),
                capabilities: package
                    .manifest
                    .capabilities()
                    .iter()
                    .map(|c| c.as_str().to_owned())
                    .collect(),
                fields,
                installation,
            });
        }
        for snapshot in &installations {
            let pin = snapshot.package_pin();
            if self
                .packages
                .iter()
                .any(|package| package.configuration.package_pin() == pin)
            {
                continue;
            }
            packages.push(PluginManagedPackageDto {
                available: false,
                package_id: pin.package_id().as_str().to_owned(),
                version: pin.package_version().as_str(),
                name: pin.package_id().as_str().to_owned(),
                description: String::new(),
                catalog_revision: pin.catalog_revision().as_str().to_owned(),
                package_digest: pin.package_digest().as_str().to_owned(),
                kind: match pin.components().first().map(|component| component.kind()) {
                    Some(ComponentKind::SkillComponent) => "skill",
                    Some(ComponentKind::McpServerComponent) => "mcp",
                    _ => "unavailable",
                }
                .to_owned(),
                capabilities: Vec::new(),
                fields: Vec::new(),
                installation: Some(installation_view(&state.authority, snapshot)?),
            });
        }
        Ok(PluginLifecycleDto {
            schema: "plugin-lifecycle/v1",
            packages,
            public_read_only: true,
            updates: state
                .authority
                .updates
                .list(
                    &state.authority.installations,
                    &state.authority.grants,
                    tenant,
                    user,
                )
                .map_err(updates::error)?
                .into_iter()
                .map(updates::view)
                .collect(),
        })
    }
}
impl RuntimeState {
    fn check(&self) -> Result<(), PluginError> {
        if self.poisoned {
            Err(PluginError::Unavailable)
        } else {
            Ok(())
        }
    }
    fn commit(&mut self, next: AuthorityState) -> Result<(), PluginError> {
        self.check()?;
        if let Err(error) = self.disk.save(&next) {
            // Capacity is proven by encoding before any filesystem operation.
            if error != PluginError::Capacity {
                self.poisoned = true;
            }
            return Err(error);
        }
        self.authority = next;
        Ok(())
    }
    fn owned(
        &self,
        tenant: &TenantId,
        user: &UserId,
        id: &InstallationId,
    ) -> Result<InstallationSnapshot, PluginError> {
        self.check()?;
        self.authority
            .installations
            .load_exact(id)
            .map_err(|_| PluginError::Unavailable)?
            .filter(|s| s.tenant_id() == tenant && s.user_id() == user)
            .ok_or(PluginError::NotFound)
    }
}
fn installation_view(
    authority: &AuthorityState,
    snapshot: &InstallationSnapshot,
) -> Result<PluginInstallationDto, PluginError> {
    let grants = authority
        .grants
        .load_current_for_installation(
            snapshot.tenant_id(),
            snapshot.user_id(),
            snapshot.installation_id(),
            snapshot.revision(),
        )
        .map_err(|_| PluginError::Unavailable)?;
    Ok(PluginInstallationDto {
        id: snapshot.installation_id().as_str().to_owned(),
        revision: snapshot.revision().as_str().to_owned(),
        state: format!("{:?}", snapshot.state()).to_lowercase(),
        values: snapshot
            .configuration()
            .entries()
            .iter()
            .filter_map(|(k, v)| {
                let value = match v {
                    ConfigurationValue::Text(t) => PluginValueDto::Text(t.as_str().to_owned()),
                    ConfigurationValue::Integer(i) => PluginValueDto::Integer(*i),
                    ConfigurationValue::Boolean(b) => PluginValueDto::Boolean(*b),
                    ConfigurationValue::Secret(_) => return None,
                };
                Some((k.as_str().to_owned(), value))
            })
            .collect(),
        active_capabilities: grants
            .grants()
            .iter()
            .filter(|g| {
                g.state() == GrantState::Active
                    && (snapshot.state() == ManagedInstallationState::Enabled
                        || g.installation_revision() == snapshot.revision())
            })
            .map(|g| g.capability_id().as_str().to_owned())
            .collect(),
    })
}
fn configuration(
    tenant: &TenantId,
    values: BTreeMap<String, PluginValueDto>,
) -> Result<InstallationConfiguration, PluginError> {
    let mut entries = Vec::new();
    for (key, value) in values {
        entries.push((
            ConfigurationKey::parse(key).map_err(|_| PluginError::InvalidRequest)?,
            match value {
                PluginValueDto::Text(t) => ConfigurationValue::Text(
                    NonSecretText::parse(t).map_err(|_| PluginError::InvalidRequest)?,
                ),
                PluginValueDto::Integer(i) => ConfigurationValue::Integer(i),
                PluginValueDto::Boolean(b) => ConfigurationValue::Boolean(b),
            },
        ));
    }
    InstallationConfiguration::new(tenant, entries).map_err(|_| PluginError::InvalidRequest)
}
fn stable_id(parts: &[&str]) -> String {
    let mut bytes = b"uca-package-application/v1\0".to_vec();
    for part in parts {
        bytes.extend_from_slice(&(part.len() as u64).to_be_bytes());
        bytes.extend_from_slice(part.as_bytes());
    }
    Sha256Digest::from_bytes(&bytes).as_str()[7..].to_owned()
}
fn tool_name(installation: &InstallationId, wire: &str) -> String {
    format!(
        "plugin_{}",
        &stable_id(&[installation.as_str(), wire])[..48]
    )
}
