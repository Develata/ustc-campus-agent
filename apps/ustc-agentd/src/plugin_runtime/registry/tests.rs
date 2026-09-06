use super::*;
use serde_json::{Value, json};
use ustc_campus_agent_core::market::configuration_schema::{
    ConfigurationFieldSchema, ConfigurationSchema,
};

#[test]
fn bundled_skill_is_optional_checked_and_read_only() {
    let package = RuntimePackage::bundled_skill().expect("checked bundled package");
    assert_eq!(package.manifest.package_id().as_str(), "ustc.campus-guide");
    assert_eq!(package.manifest.components().len(), 1);
    let RuntimeComponent::Skill { source } = package.component else {
        panic!("skill");
    };
    assert_eq!(source.metadata().name, "campus-guide");
    assert!(!source.metadata().description.is_empty());
    assert_eq!(
        source.resource_paths(),
        vec!["skills/campus-guide/SKILL.md"]
    );
    assert!(
        source
            .read(source.skill_path())
            .expect("skill text")
            .contains("学生竞赛")
    );
    assert_eq!(
        source.verified_artifact_digest().expect("verified bytes"),
        *package.configuration.package_pin().components()[0].digest()
    );
    assert_eq!(
        source.read("/etc/passwd").expect_err("unlisted"),
        RuntimeRegistryError::InvalidDeclaration
    );
}

fn mcp_sources(runtime: Value) -> (Vec<u8>, Vec<u8>, Vec<u8>) {
    let mut manifest: Value = serde_json::from_slice(BUNDLED_MANIFEST).expect("manifest");
    manifest["components"] = json!([{"type":"McpServerComponent","path":"runtime.json"}]);
    let manifest = serde_json::to_vec(&manifest).expect("bytes");
    let package = load_package_manifest(&manifest).expect("checked manifest");
    let runtime = serde_json::to_vec(&runtime).expect("runtime bytes");
    let schema = ConfigurationSchema::new(vec![
        ConfigurationFieldSchema::text(
            ConfigurationKey::parse("endpoint").expect("key"),
            true,
            2048,
        )
        .expect("field"),
    ])
    .expect("schema");
    let configuration = json!({"schemaVersion":"package-component-configuration/v1","packageId":package.package_id().as_str(),"packageVersion":package.package_version().as_str(),"packageDigest":package.package_digest().as_str(),"componentSetDigest":package.component_declaration_set_digest().as_str(),"capabilityManifestDigest":package.capability_manifest_digest().as_str(),"components":[{"path":"runtime.json","type":"McpServerComponent","mode":null,"componentId":"component:mcp","componentVersion":"1","componentDigest":Sha256Digest::from_bytes(&runtime).as_str(),"executionIdentity":"execution:mcp","schemaDigest":schema.digest().as_str(),"fields":[{"key":"endpoint","kind":"text","required":true,"maxUtf8Bytes":2048}]}]});
    (
        manifest,
        serde_json::to_vec(&configuration).expect("configuration bytes"),
        runtime,
    )
}
fn runtime() -> Value {
    json!({"schemaVersion":"plugin-runtime/v1","kind":"mcp","endpointKey":"endpoint","tools":[{"name":"campus_read","capabilityId":"campus.public_rules.read"}]})
}
fn from_mcp(raw: Value) -> Result<RuntimePackage, RuntimeRegistryError> {
    let (manifest, configuration, runtime) = mcp_sources(raw);
    RuntimePackage::from_sources(
        &manifest,
        &configuration,
        &runtime,
        SkillStorage::Directory(PathBuf::from("/reviewed/operator/package")),
    )
}
#[test]
fn mcp_configuration_is_closed_capability_mapped_and_public_https_by_default() {
    let package = from_mcp(runtime()).expect("mcp declaration");
    let RuntimeComponent::Mcp {
        endpoint_key,
        tools,
        endpoint_policy,
        bearer_file,
        credential_endpoint,
    } = package.component
    else {
        panic!("mcp");
    };
    assert_eq!(endpoint_key.as_str(), "endpoint");
    assert_eq!(tools["campus_read"].as_str(), "campus.public_rules.read");
    assert_eq!(endpoint_policy, EndpointPolicy::PublicHttps);
    assert!(bearer_file.is_none());
    assert!(credential_endpoint.is_none());
    let mut explicit = runtime();
    explicit["endpointPolicy"] = json!("loopback_development");
    let package = from_mcp(explicit).expect("explicit operator loopback");
    assert!(matches!(
        package.component,
        RuntimeComponent::Mcp {
            endpoint_policy: EndpointPolicy::LoopbackDevelopment,
            ..
        }
    ));
    for changed in ["url", "mapping", "duplicates", "field"] {
        let mut raw = runtime();
        match changed {
            "url" => raw["endpoint"] = json!("http://127.0.0.1/arbitrary"),
            "mapping" => raw["tools"][0]["capabilityId"] = json!("user.own_calendar_items.write"),
            "duplicates" => {
                raw["tools"] = json!([{"name":"same","capabilityId":"campus.public_rules.read"},{"name":"same","capabilityId":"campus.public_rules.read"}])
            }
            _ => raw["endpointKey"] = json!("undeclared"),
        }
        assert!(
            matches!(from_mcp(raw), Err(RuntimeRegistryError::InvalidDeclaration)),
            "{changed}"
        );
    }
}
#[test]
fn mcp_runtime_bytes_are_artifact_bound_and_duplicate_keys_reject() {
    let (manifest, configuration, runtime) = mcp_sources(runtime());
    let mut changed = runtime.clone();
    changed.push(b' ');
    assert!(matches!(
        RuntimePackage::from_sources(&manifest, &configuration, &changed, SkillStorage::Bundled),
        Err(RuntimeRegistryError::ArtifactMismatch)
    ));
    let duplicate =
        String::from_utf8(runtime)
            .expect("UTF-8")
            .replacen('{', "{\"kind\":\"mcp\",", 1);
    assert!(matches!(
        RuntimePackage::from_sources(
            &manifest,
            &configuration,
            duplicate.as_bytes(),
            SkillStorage::Bundled
        ),
        Err(RuntimeRegistryError::InvalidDeclaration)
    ));
}

#[cfg(unix)]
#[test]
fn operator_loader_rechecks_drift_and_rejects_symlink_metadata() {
    use std::{
        fs,
        os::unix::fs::symlink,
        sync::atomic::{AtomicU64, Ordering},
    };
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let root = std::env::temp_dir().join(format!(
        "uca-runtime-registry-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&root).expect("new test directory");
    fs::create_dir_all(root.join("skills/campus-guide")).expect("skill directory");
    for (path, bytes) in [
        ("package.json", BUNDLED_MANIFEST),
        ("configuration.json", BUNDLED_CONFIGURATION),
        ("runtime.json", BUNDLED_RUNTIME),
        ("skills/campus-guide/SKILL.md", BUNDLED_SKILL),
    ] {
        fs::write(root.join(path), bytes).expect("fixture file");
    }
    let package = RuntimePackage::load(&root).expect("operator package");
    let RuntimeComponent::Skill { source } = package.component else {
        panic!("skill");
    };
    fs::write(root.join(source.skill_path()), b"changed").expect("drift");
    assert_eq!(
        source
            .verified_artifact_digest()
            .expect_err("drift rejects"),
        RuntimeRegistryError::ArtifactMismatch
    );
    fs::remove_file(root.join("runtime.json")).expect("remove owned fixture");
    symlink(root.join("package.json"), root.join("runtime.json")).expect("symlink fixture");
    assert!(matches!(
        RuntimePackage::load(&root),
        Err(RuntimeRegistryError::Unavailable)
    ));
    fs::remove_dir_all(root).expect("remove owned fixture");
}

#[test]
fn credentials_require_an_exact_operator_url_and_file_pair() {
    for missing in ["bearerFile", "credentialEndpoint"] {
        let mut raw = runtime();
        raw["bearerFile"] = json!("/reviewed/token");
        raw["credentialEndpoint"] = json!("https://mcp.example.test/rpc?tenant=one");
        raw.as_object_mut().expect("object").remove(missing);
        assert!(matches!(
            from_mcp(raw),
            Err(RuntimeRegistryError::InvalidDeclaration)
        ));
    }
    for endpoint in [
        "relative",
        "file:///secret",
        "https://user:pass@example.test/rpc",
        "https://example.test/rpc#fragment",
        "https://example.test/ bad",
    ] {
        let mut raw = runtime();
        raw["bearerFile"] = json!("/reviewed/token");
        raw["credentialEndpoint"] = json!(endpoint);
        assert!(matches!(
            from_mcp(raw),
            Err(RuntimeRegistryError::InvalidDeclaration)
        ));
    }
    let mut raw = runtime();
    raw["bearerFile"] = json!("/reviewed/token");
    raw["credentialEndpoint"] = json!("https://mcp.example.test/rpc?tenant=one");
    let package = from_mcp(raw).expect("paired operator declaration");
    assert!(
        matches!(package.component, RuntimeComponent::Mcp { credential_endpoint: Some(ref endpoint), .. } if endpoint == "https://mcp.example.test/rpc?tenant=one")
    );
}
