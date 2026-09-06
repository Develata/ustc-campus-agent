//! Local operator preparation; all canonical digests come from existing M20 owners.
use std::path::Path;
#[cfg(unix)]
mod directory;
#[cfg(unix)]
use serde_json::json;
#[cfg(unix)]
use ustc_campus_agent_adapters::skills::ParsedSkill;
#[cfg(unix)]
use ustc_campus_agent_core::{
    invocation::{
        CatalogRevision, ComponentId, ComponentKind, ComponentVersion, ExecutionIdentity,
        Sha256Digest,
    },
    market::{
        configuration_catalog::load_package_configuration,
        configuration_schema::{ConfigurationFieldSchema, ConfigurationSchema},
        installation::ConfigurationKey,
        load_package_manifest,
    },
};

pub(super) fn run(args: &[String]) -> Result<(), String> {
    let [flag, path] = args else {
        return Err("expected --package-dir PATH".to_owned());
    };
    if flag != "--package-dir" {
        return Err("expected --package-dir PATH".to_owned());
    }
    prepare(Path::new(path))?;
    println!("configuration.json created; review the package before installation");
    Ok(())
}
#[cfg(unix)]
fn prepare(root: &Path) -> Result<(), String> {
    let directory = directory::PackageDirectory::open(root)?;
    let manifest_bytes = directory.read("package.json", 1024 * 1024)?;
    let manifest =
        load_package_manifest(&manifest_bytes).map_err(|_| "package declaration rejected")?;
    let [component] = manifest.components() else {
        return Err("exactly one component is required".to_owned());
    };
    let source = directory.read(
        component.path(),
        if component.kind() == ComponentKind::SkillComponent {
            64 * 1024
        } else {
            1024 * 1024
        },
    )?;
    let (kind, fields, raw_fields, suffix) = match component.kind() {
        ComponentKind::SkillComponent => {
            let name = Path::new(component.path())
                .parent()
                .and_then(Path::file_name)
                .and_then(|v| v.to_str())
                .ok_or("skill directory unavailable")?;
            ParsedSkill::parse(name, &source).map_err(|_| "SKILL.md rejected")?;
            ("SkillComponent", Vec::new(), Vec::new(), "skill")
        }
        ComponentKind::McpServerComponent => {
            if component.path() != "runtime.json" {
                return Err("MCP component must declare runtime.json".to_owned());
            }
            let runtime: serde_json::Value =
                serde_json::from_slice(&source).map_err(|_| "runtime declaration rejected")?;
            let endpoint = runtime
                .get("endpointKey")
                .and_then(serde_json::Value::as_str)
                .ok_or("runtime endpointKey missing")?;
            let field = ConfigurationFieldSchema::text(
                ConfigurationKey::parse(endpoint).map_err(|_| "invalid endpoint field")?,
                true,
                2048,
            )
            .map_err(|_| "invalid endpoint schema")?;
            (
                "McpServerComponent",
                vec![field],
                vec![json!({"kind":"text","key":endpoint,"required":true,"maxUtf8Bytes":2048})],
                "mcp",
            )
        }
        _ => return Err("only package-owned MCP or Skill components are supported".to_owned()),
    };
    let schema = ConfigurationSchema::new(fields).map_err(|_| "configuration schema rejected")?;
    let id = ComponentId::parse(format!(
        "component:{}/{suffix}",
        manifest.package_id().as_str()
    ))
    .map_err(|_| "component id rejected")?;
    let version = ComponentVersion::parse(manifest.package_version().as_str())
        .map_err(|_| "component version rejected")?;
    let execution = ExecutionIdentity::parse(format!(
        "execution:{}/{suffix}/v1",
        manifest.package_id().as_str()
    ))
    .map_err(|_| "execution identity rejected")?;
    let body = json!({"schemaVersion":"package-component-configuration/v1","packageId":manifest.package_id().as_str(),"packageVersion":manifest.package_version().as_str(),
        "packageDigest":manifest.package_digest().as_str(),"componentSetDigest":manifest.component_declaration_set_digest().as_str(),"capabilityManifestDigest":manifest.capability_manifest_digest().as_str(),
        "components":[{"path":component.path(),"type":kind,"mode":component.mode(),"componentId":id.as_str(),"componentVersion":version.as_str(),"componentDigest":Sha256Digest::from_bytes(&source).as_str(),
            "executionIdentity":execution.as_str(),"schemaDigest":schema.digest().as_str(),"fields":raw_fields}]});
    let bytes = serde_json::to_vec_pretty(&body).map_err(|_| "configuration encoding failed")?;
    let revision = CatalogRevision::parse("catalog:operator-configuration-preparation-v1")
        .map_err(|_| "invalid preparation revision")?;
    load_package_configuration(&bytes, &manifest, &revision)
        .map_err(|_| "configuration roundtrip rejected")?;
    directory.create_sidecar(&bytes)
}
#[cfg(not(unix))]
fn prepare(_: &Path) -> Result<(), String> {
    Err(
        "component configuration preparation is unsupported on this platform; use a Unix host"
            .to_owned(),
    )
}
#[cfg(all(test, unix))]
mod tests;
