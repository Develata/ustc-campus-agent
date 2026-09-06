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
    invocation::{CapabilityId, CatalogRevision, ComponentKind, Sha256Digest},
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

#[derive(Clone)]
pub(crate) struct RuntimePackage {
    pub manifest: ValidatedPackageManifest,
    pub configuration: ValidatedPackageConfiguration,
    pub component: RuntimeComponent,
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
        let manifest = load_package_manifest(manifest)
            .map_err(|_| RuntimeRegistryError::InvalidDeclaration)?;
        let [declaration] = manifest.components() else {
            return Err(RuntimeRegistryError::UnsupportedPackage);
        };
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
        let raw: RawRuntime = serde_json::from_slice(runtime)
            .map_err(|_| RuntimeRegistryError::InvalidDeclaration)?;
        if raw.schema_version != "plugin-runtime/v1" {
            return Err(RuntimeRegistryError::InvalidDeclaration);
        }
        let pin = &configuration.package_pin().components()[0];
        let component = match raw.kind {
            RawKind::Skill => {
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
                    storage,
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
                    RawEndpointPolicy::LoopbackDevelopment => EndpointPolicy::LoopbackDevelopment,
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
        Ok(Self {
            manifest,
            configuration,
            component,
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
#[derive(Deserialize)]
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
