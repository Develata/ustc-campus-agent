//! Fixed bundled or explicitly operator-selected package inputs. Never request paths.
use serde::Deserialize;
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};
use ustc_campus_agent_adapters::{
    mcp::EndpointPolicy,
    skills::{DeclaredTextResource, ParsedSkill, load_declared_text_resource},
};
use ustc_campus_agent_core::{
    invocation::{CapabilityId, CatalogRevision, ComponentId, ComponentKind, Sha256Digest},
    market::{
        ValidatedPackageManifest,
        configuration_catalog::{ValidatedPackageConfiguration, load_package_configuration},
        configuration_schema::ConfigurationFieldKind,
        installation::ConfigurationKey,
        load_package_manifest,
    },
};

const MAX_METADATA_BYTES: usize = 64 * 1024;
const BUNDLED_MANIFEST: &[u8] =
    include_bytes!("../../../../market/packages/ustc.campus-guide/package.json");
const BUNDLED_CONFIGURATION: &[u8] =
    include_bytes!("../../../../market/packages/ustc.campus-guide/configuration.json");
const BUNDLED_RUNTIME: &[u8] =
    include_bytes!("../../../../market/packages/ustc.campus-guide/runtime.json");
const BUNDLED_SKILL: &[u8] =
    include_bytes!("../../../../market/packages/ustc.campus-guide/skills/campus-guide/SKILL.md");

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RuntimeRegistryError {
    Unavailable,
    Capacity,
    InvalidDeclaration,
    UnsupportedPackage,
    ArtifactMismatch,
}
impl fmt::Display for RuntimeRegistryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "plugin runtime declaration rejected: {self:?}")
    }
}
impl std::error::Error for RuntimeRegistryError {}

// This read-only Skill profile has one exact authority, independent of sibling MCP tools.
// Admission and probe share the binding so no package-wide ordering selects permission.
pub(super) fn skill_read_capability(
    manifest: &ValidatedPackageManifest,
) -> Result<CapabilityId, RuntimeRegistryError> {
    manifest
        .capabilities()
        .iter()
        .find(|capability| capability.as_str() == "campus.public_rules.read")
        .cloned()
        .ok_or(RuntimeRegistryError::InvalidDeclaration)
}

#[derive(Clone)]
pub(crate) struct RuntimePackage {
    pub manifest_source: Vec<u8>,
    pub configuration_source: Vec<u8>,
    pub manifest: ValidatedPackageManifest,
    pub configuration: ValidatedPackageConfiguration,
    pub component: RuntimeComponent,
    pub additional: BTreeMap<ComponentId, RuntimeComponent>,
}
#[derive(Clone)]
pub(crate) enum RuntimeComponent {
    Skill {
        source: SkillSource,
    },
    Mcp {
        endpoint_key: ConfigurationKey,
        tools: BTreeMap<String, CapabilityId>,
        endpoint_policy: EndpointPolicy,
        bearer_file: Option<PathBuf>,
        credential_endpoint: Option<String>,
    },
}
#[derive(Clone)]
pub(crate) struct SkillMetadata {
    pub name: String,
    pub description: String,
}
#[derive(Clone)]
enum SkillStorage {
    Bundled,
    Directory(PathBuf),
}
#[derive(Clone)]
pub(crate) struct SkillSource {
    storage: SkillStorage,
    skill_path: String,
    declarations: Vec<DeclaredTextResource>,
    metadata: SkillMetadata,
}
impl SkillSource {
    pub(crate) fn metadata(&self) -> &SkillMetadata {
        &self.metadata
    }
    pub(crate) fn skill_path(&self) -> &str {
        &self.skill_path
    }
    #[cfg(test)]
    pub(crate) fn resource_paths(&self) -> Vec<String> {
        self.declarations
            .iter()
            .map(|resource| resource.path().to_owned())
            .collect()
    }
    pub(crate) fn read(&self, requested_resource: &str) -> Result<String, RuntimeRegistryError> {
        let declaration = self
            .declarations
            .iter()
            .find(|resource| resource.path() == requested_resource)
            .ok_or(RuntimeRegistryError::InvalidDeclaration)?;
        match &self.storage {
            SkillStorage::Directory(root) => {
                load_declared_text_resource(root, &self.declarations, requested_resource)
                    .map(|resource| resource.text().to_owned())
                    .map_err(|_| RuntimeRegistryError::ArtifactMismatch)
            }
            SkillStorage::Bundled => {
                if requested_resource != self.skill_path
                    || Sha256Digest::from_bytes(BUNDLED_SKILL) != *declaration.digest()
                {
                    return Err(RuntimeRegistryError::ArtifactMismatch);
                }
                std::str::from_utf8(BUNDLED_SKILL)
                    .map(str::to_owned)
                    .map_err(|_| RuntimeRegistryError::InvalidDeclaration)
            }
        }
    }
    /// Rechecks source bytes on every readiness check; prior metadata is not a read grant.
    pub(crate) fn verified_artifact_digest(&self) -> Result<Sha256Digest, RuntimeRegistryError> {
        let text = self.read(&self.skill_path)?;
        let directory = skill_directory(&self.skill_path)?;
        ParsedSkill::parse(directory, text.as_bytes())
            .map_err(|_| RuntimeRegistryError::InvalidDeclaration)?;
        Ok(Sha256Digest::from_bytes(text.as_bytes()))
    }
}

impl RuntimePackage {
    pub(crate) fn components(&self) -> Vec<(&ComponentId, &RuntimeComponent)> {
        let mut components = vec![(
            self.configuration.package_pin().components()[0].component_id(),
            &self.component,
        )];
        components.extend(self.additional.iter());
        components
    }
    pub(crate) fn component(&self, id: &ComponentId) -> Option<&RuntimeComponent> {
        self.components()
            .into_iter()
            .find(|(candidate, _)| *candidate == id)
            .map(|(_, component)| component)
    }
    pub(crate) fn bundled_skill() -> Result<Self, RuntimeRegistryError> {
        Self::from_sources(
            BUNDLED_MANIFEST,
            BUNDLED_CONFIGURATION,
            BUNDLED_RUNTIME,
            SkillStorage::Bundled,
        )
    }
    pub(crate) fn load(root: &Path) -> Result<Self, RuntimeRegistryError> {
        let manifest = read_metadata(root, "package.json")?;
        let configuration = read_metadata(root, "configuration.json")?;
        let runtime = read_metadata(root, "runtime.json")?;
        Self::from_sources(
            &manifest,
            &configuration,
            &runtime,
            SkillStorage::Directory(root.to_owned()),
        )
    }
    fn from_sources(
        manifest: &[u8],
        configuration: &[u8],
        runtime: &[u8],
        storage: SkillStorage,
    ) -> Result<Self, RuntimeRegistryError> {
        if [manifest, configuration, runtime]
            .iter()
            .any(|bytes| bytes.len() > MAX_METADATA_BYTES)
        {
            return Err(RuntimeRegistryError::Capacity);
        }
        let manifest_source = manifest.to_vec();
        let configuration_source = configuration.to_vec();
        let manifest = load_package_manifest(manifest)
            .map_err(|_| RuntimeRegistryError::InvalidDeclaration)?;
        if manifest.components().is_empty() || manifest.components().len() > 16 {
            return Err(RuntimeRegistryError::UnsupportedPackage);
        }
        let mut revision_bytes = b"plugin-runtime-reviewed-sources/v1\0".to_vec();
        for bytes in [
            manifest.package_digest().as_str().as_bytes(),
            configuration,
            runtime,
        ] {
            revision_bytes.extend_from_slice(&(bytes.len() as u64).to_be_bytes());
            revision_bytes.extend_from_slice(bytes);
        }
        let revision = CatalogRevision::parse(format!(
            "catalog:{}",
            Sha256Digest::from_bytes(&revision_bytes).as_str()
        ))
        .map_err(|_| RuntimeRegistryError::InvalidDeclaration)?;
        let configuration = load_package_configuration(configuration, &manifest, &revision)
            .map_err(|_| RuntimeRegistryError::InvalidDeclaration)?;
        let envelope: serde_json::Value = serde_json::from_slice(runtime)
            .map_err(|_| RuntimeRegistryError::InvalidDeclaration)?;
        let entries = if envelope.get("schemaVersion").and_then(|v| v.as_str())
            == Some("plugin-runtime/v2")
        {
            let raw: RawPackageRuntime = serde_json::from_slice(runtime)
                .map_err(|_| RuntimeRegistryError::InvalidDeclaration)?;
            if raw.schema_version != "plugin-runtime/v2" {
                return Err(RuntimeRegistryError::InvalidDeclaration);
            }
            raw.components
        } else {
            if manifest.components().len() != 1 {
                return Err(RuntimeRegistryError::UnsupportedPackage);
            }
            vec![RawRuntimeMember {
                component_id: configuration.package_pin().components()[0]
                    .component_id()
                    .as_str()
                    .to_owned(),
                runtime: serde_json::from_slice(runtime)
                    .map_err(|_| RuntimeRegistryError::InvalidDeclaration)?,
            }]
        };
        if entries.len() != manifest.components().len() {
            return Err(RuntimeRegistryError::InvalidDeclaration);
        }
        let first_schema = configuration
            .bindings()
            .values()
            .next()
            .ok_or(RuntimeRegistryError::InvalidDeclaration)?
            .schema();
        if configuration
            .bindings()
            .values()
            .any(|binding| binding.schema() != first_schema)
        {
            return Err(RuntimeRegistryError::UnsupportedPackage);
        }
        // The checked sidecar owns the component ID to declared path relationship.
        // Digest equality alone cannot substitute a sibling declaration.
        let configuration_document: serde_json::Value =
            serde_json::from_slice(&configuration_source)
                .map_err(|_| RuntimeRegistryError::InvalidDeclaration)?;
        let mut components = BTreeMap::new();
        for entry in entries {
            let id = ComponentId::parse(entry.component_id)
                .map_err(|_| RuntimeRegistryError::InvalidDeclaration)?;
            let pin = configuration
                .package_pin()
                .components()
                .iter()
                .find(|pin| pin.component_id() == &id)
                .ok_or(RuntimeRegistryError::InvalidDeclaration)?;
            let raw = entry.runtime;
            if raw.schema_version != "plugin-runtime/v1" {
                return Err(RuntimeRegistryError::InvalidDeclaration);
            }
            let expected_path = configuration_document["components"]
                .as_array()
                .and_then(|members| {
                    members
                        .iter()
                        .find(|member| member["componentId"].as_str() == Some(id.as_str()))
                })
                .and_then(|member| member["path"].as_str())
                .ok_or(RuntimeRegistryError::InvalidDeclaration)?;
            let declaration = manifest
                .components()
                .iter()
                .find(|declaration| declaration.path() == expected_path)
                .ok_or(RuntimeRegistryError::InvalidDeclaration)?;
            if declaration.kind() != pin.kind() {
                return Err(RuntimeRegistryError::InvalidDeclaration);
            }
            let component = match raw.kind {
                RawKind::Skill => {
                    skill_read_capability(&manifest)?;
                    if declaration.kind() != ComponentKind::SkillComponent
                        || raw.endpoint_key.is_some()
                        || raw.tools.is_some()
                        || raw.endpoint_policy.is_some()
                        || raw.bearer_file.is_some()
                        || raw.credential_endpoint.is_some()
                    {
                        return Err(RuntimeRegistryError::InvalidDeclaration);
                    }
                    let path = raw
                        .skill_path
                        .ok_or(RuntimeRegistryError::InvalidDeclaration)?;
                    if declaration.path() != path {
                        return Err(RuntimeRegistryError::InvalidDeclaration);
                    }
                    let directory = skill_directory(&path)?;
                    let resources = raw
                        .resources
                        .ok_or(RuntimeRegistryError::InvalidDeclaration)?;
                    if resources.is_empty() || resources.len() > 256 {
                        return Err(RuntimeRegistryError::Capacity);
                    }
                    let mut names = BTreeSet::new();
                    let declarations = resources
                        .into_iter()
                        .map(|resource| {
                            if !names.insert(resource.path.clone()) {
                                return Err(RuntimeRegistryError::InvalidDeclaration);
                            }
                            DeclaredTextResource::new(
                                &resource.path,
                                Sha256Digest::parse(resource.sha256)
                                    .map_err(|_| RuntimeRegistryError::InvalidDeclaration)?,
                            )
                            .map_err(|_| RuntimeRegistryError::InvalidDeclaration)
                        })
                        .collect::<Result<Vec<_>, _>>()?;
                    let artifact = declarations
                        .iter()
                        .find(|resource| resource.path() == path)
                        .ok_or(RuntimeRegistryError::InvalidDeclaration)?;
                    if artifact.digest() != pin.digest() {
                        return Err(RuntimeRegistryError::ArtifactMismatch);
                    }
                    if matches!(storage, SkillStorage::Bundled) && declarations.len() != 1 {
                        return Err(RuntimeRegistryError::UnsupportedPackage);
                    }
                    let mut source = SkillSource {
                        storage: storage.clone(),
                        skill_path: path.clone(),
                        declarations,
                        metadata: SkillMetadata {
                            name: String::new(),
                            description: String::new(),
                        },
                    };
                    let text = source.read(&path)?;
                    let parsed = ParsedSkill::parse(directory, text.as_bytes())
                        .map_err(|_| RuntimeRegistryError::InvalidDeclaration)?;
                    source.metadata = SkillMetadata {
                        name: parsed.name().to_owned(),
                        description: parsed.description().to_owned(),
                    };
                    RuntimeComponent::Skill { source }
                }
                RawKind::Mcp => {
                    if declaration.kind() != ComponentKind::McpServerComponent
                        || declaration.path() != "runtime.json"
                        || raw.skill_path.is_some()
                        || raw.resources.is_some()
                    {
                        return Err(RuntimeRegistryError::InvalidDeclaration);
                    }
                    if pin.digest() != &Sha256Digest::from_bytes(runtime) {
                        return Err(RuntimeRegistryError::ArtifactMismatch);
                    }
                    let endpoint_key = ConfigurationKey::parse(
                        raw.endpoint_key
                            .ok_or(RuntimeRegistryError::InvalidDeclaration)?,
                    )
                    .map_err(|_| RuntimeRegistryError::InvalidDeclaration)?;
                    let binding = configuration
                        .binding(pin.component_id())
                        .ok_or(RuntimeRegistryError::InvalidDeclaration)?;
                    let field = binding
                        .schema()
                        .fields()
                        .get(&endpoint_key)
                        .ok_or(RuntimeRegistryError::InvalidDeclaration)?;
                    if field.kind() != ConfigurationFieldKind::Text
                        || !field.required()
                        || field.max_utf8_bytes().is_none_or(|limit| limit > 2048)
                    {
                        return Err(RuntimeRegistryError::InvalidDeclaration);
                    }
                    let entries = raw.tools.ok_or(RuntimeRegistryError::InvalidDeclaration)?;
                    if entries.is_empty() || entries.len() > 64 {
                        return Err(RuntimeRegistryError::Capacity);
                    }
                    let mut tools = BTreeMap::new();
                    for entry in entries {
                        if entry.name.is_empty()
                            || entry.name.len() > 128
                            || !entry.name.bytes().all(|byte| {
                                byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.')
                            })
                        {
                            return Err(RuntimeRegistryError::InvalidDeclaration);
                        }
                        let capability = CapabilityId::parse(entry.capability_id)
                            .map_err(|_| RuntimeRegistryError::InvalidDeclaration)?;
                        if !manifest.capabilities().contains(&capability)
                            || tools.insert(entry.name, capability).is_some()
                        {
                            return Err(RuntimeRegistryError::InvalidDeclaration);
                        }
                    }
                    let endpoint_policy = match raw
                        .endpoint_policy
                        .unwrap_or(RawEndpointPolicy::PublicHttps)
                    {
                        RawEndpointPolicy::PublicHttps => EndpointPolicy::PublicHttps,
                        RawEndpointPolicy::LoopbackDevelopment => {
                            EndpointPolicy::LoopbackDevelopment
                        }
                    };
                    if raw.bearer_file.is_some() != raw.credential_endpoint.is_some() {
                        return Err(RuntimeRegistryError::InvalidDeclaration);
                    }
                    let credential_endpoint = raw
                        .credential_endpoint
                        .map(|endpoint| {
                            let url = reqwest::Url::parse(&endpoint)
                                .map_err(|_| RuntimeRegistryError::InvalidDeclaration)?;
                            if endpoint.len() > 2048
                                || endpoint.chars().any(char::is_whitespace)
                                || !matches!(url.scheme(), "http" | "https")
                                || url.host_str().is_none()
                                || !url.username().is_empty()
                                || url.password().is_some()
                                || url.fragment().is_some()
                            {
                                return Err(RuntimeRegistryError::InvalidDeclaration);
                            }
                            Ok(endpoint)
                        })
                        .transpose()?;
                    let bearer_file = raw
                        .bearer_file
                        .map(|path| {
                            if path.len() > 1024
                                || path.chars().any(char::is_control)
                                || !Path::new(&path).is_absolute()
                            {
                                return Err(RuntimeRegistryError::InvalidDeclaration);
                            }
                            Ok(PathBuf::from(path))
                        })
                        .transpose()?;
                    RuntimeComponent::Mcp {
                        endpoint_key,
                        tools,
                        endpoint_policy,
                        bearer_file,
                        credential_endpoint,
                    }
                }
            };
            if components.insert(id, component).is_some() {
                return Err(RuntimeRegistryError::InvalidDeclaration);
            }
        }
        let first = configuration.package_pin().components()[0].component_id();
        let component = components
            .remove(first)
            .ok_or(RuntimeRegistryError::InvalidDeclaration)?;
        Ok(Self {
            manifest_source,
            configuration_source,
            manifest,
            configuration,
            component,
            additional: components,
        })
    }
}
fn skill_directory(path: &str) -> Result<&str, RuntimeRegistryError> {
    let mut parts = path.rsplit('/');
    if parts.next() != Some("SKILL.md") {
        return Err(RuntimeRegistryError::InvalidDeclaration);
    }
    parts
        .next()
        .filter(|name| !name.is_empty())
        .ok_or(RuntimeRegistryError::InvalidDeclaration)
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct RawPackageRuntime {
    schema_version: String,
    components: Vec<RawRuntimeMember>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct RawRuntimeMember {
    component_id: String,
    runtime: RawRuntime,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct RawRuntime {
    schema_version: String,
    kind: RawKind,
    skill_path: Option<String>,
    resources: Option<Vec<RawResource>>,
    endpoint_key: Option<String>,
    tools: Option<Vec<RawTool>>,
    endpoint_policy: Option<RawEndpointPolicy>,
    bearer_file: Option<String>,
    credential_endpoint: Option<String>,
}
#[derive(Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
enum RawKind {
    Skill,
    Mcp,
}
#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
enum RawEndpointPolicy {
    PublicHttps,
    LoopbackDevelopment,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawResource {
    path: String,
    sha256: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct RawTool {
    name: String,
    capability_id: String,
}

#[cfg(unix)]
fn read_metadata(root: &Path, name: &str) -> Result<Vec<u8>, RuntimeRegistryError> {
    use rustix::fs::{Mode, OFlags, open, openat};
    use std::path::Component;
    if !root.is_absolute() {
        return Err(RuntimeRegistryError::InvalidDeclaration);
    }
    let flags = OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC;
    let mut directory =
        open("/", flags, Mode::empty()).map_err(|_| RuntimeRegistryError::Unavailable)?;
    for part in root.components() {
        match part {
            Component::RootDir => {}
            Component::Normal(part) => {
                directory = openat(&directory, part, flags, Mode::empty())
                    .map_err(|_| RuntimeRegistryError::Unavailable)?
            }
            _ => return Err(RuntimeRegistryError::InvalidDeclaration),
        }
    }
    let file = File::from(
        openat(
            &directory,
            name,
            OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::NONBLOCK | OFlags::CLOEXEC,
            Mode::empty(),
        )
        .map_err(|_| RuntimeRegistryError::Unavailable)?,
    );
    let meta = file
        .metadata()
        .map_err(|_| RuntimeRegistryError::Unavailable)?;
    if !meta.is_file() {
        return Err(RuntimeRegistryError::Unavailable);
    }
    if meta.len() > MAX_METADATA_BYTES as u64 {
        return Err(RuntimeRegistryError::Capacity);
    }
    let mut bytes = Vec::new();
    file.take((MAX_METADATA_BYTES + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(|_| RuntimeRegistryError::Unavailable)?;
    if bytes.len() > MAX_METADATA_BYTES {
        return Err(RuntimeRegistryError::Capacity);
    }
    Ok(bytes)
}
#[cfg(not(unix))]
fn read_metadata(_: &Path, _: &str) -> Result<Vec<u8>, RuntimeRegistryError> {
    Err(RuntimeRegistryError::UnsupportedPackage)
}

#[cfg(test)]
mod tests;
