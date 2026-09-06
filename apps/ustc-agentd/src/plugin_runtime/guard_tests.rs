//! Regression proofs for operator credential routing and historical installation views.
use super::*;
use serde_json::{Value, json};
use std::{
    fs,
    os::unix::fs::PermissionsExt,
    sync::atomic::{AtomicU64, Ordering},
};
use ustc_campus_agent_core::market::{
    configuration_catalog::load_package_configuration,
    configuration_schema::{ConfigurationFieldSchema, ConfigurationSchema},
    load_package_manifest,
};
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "uca-plugin-guard-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).expect("isolated fixture");
        fs::set_permissions(&path, fs::Permissions::from_mode(0o700)).expect("private fixture");
        Self(path)
    }
    fn state(&self) -> PathBuf {
        self.0.join("state/authority.bin")
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        if self.0.parent() == Some(std::env::temp_dir().as_path())
            && self
                .0
                .file_name()
                .is_some_and(|name| name.to_string_lossy().starts_with("uca-plugin-guard-"))
        {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
}
fn owner() -> (TenantId, UserId) {
    (
        TenantId::parse("tenant:guard").expect("tenant"),
        UserId::parse("user:guard").expect("user"),
    )
}
async fn command(
    runtime: &PluginRuntime,
    request_id: &str,
    intent: Value,
) -> Result<PluginCommandResultDto, PluginError> {
    let (tenant, user) = owner();
    runtime
        .command(
            &tenant,
            &user,
            serde_json::from_value(
                json!({"schema":"plugin-command/v1","request_id":request_id,"intent":intent}),
            )
            .expect("intent"),
        )
        .await
}
async fn install(runtime: &PluginRuntime) -> PluginCommandResultDto {
    let pin = runtime.packages[0].configuration.package_pin();
    command(runtime, "install", json!({"action":"install","package_id":pin.package_id().as_str(),"version":pin.package_version().as_str(),"catalog_revision":pin.catalog_revision().as_str(),"package_digest":pin.package_digest().as_str()})).await.expect("install")
}
fn mcp_package(root: &std::path::Path, endpoint: &str) -> RuntimePackage {
    fs::create_dir(root).expect("package directory");
    let mut raw: Value = serde_json::from_slice(include_bytes!(
        "../../../../market/packages/ustc.campus-guide/package.json"
    ))
    .expect("manifest");
    raw["components"] = json!([{"type":"McpServerComponent","path":"runtime.json"}]);
    let manifest_bytes = serde_json::to_vec(&raw).expect("manifest bytes");
    let manifest = load_package_manifest(&manifest_bytes).expect("checked manifest");
    let runtime = serde_json::to_vec(&json!({"schemaVersion":"plugin-runtime/v1","kind":"mcp","endpointKey":"endpoint","endpointPolicy":"loopback_development","bearerFile":root.join("missing-token"),"credentialEndpoint":endpoint,"tools":[{"name":"campus_read","capabilityId":"campus.public_rules.read"}]})).expect("runtime bytes");
    let schema = ConfigurationSchema::new(vec![
        ConfigurationFieldSchema::text(
            ConfigurationKey::parse("endpoint").expect("key"),
            true,
            2048,
        )
        .expect("field"),
    ])
    .expect("schema");
    let config = json!({"schemaVersion":"package-component-configuration/v1","packageId":manifest.package_id().as_str(),"packageVersion":manifest.package_version().as_str(),"packageDigest":manifest.package_digest().as_str(),"componentSetDigest":manifest.component_declaration_set_digest().as_str(),"capabilityManifestDigest":manifest.capability_manifest_digest().as_str(),"components":[{"path":"runtime.json","type":"McpServerComponent","mode":null,"componentId":"component:mcp","componentVersion":"1","componentDigest":Sha256Digest::from_bytes(&runtime).as_str(),"executionIdentity":"execution:mcp","schemaDigest":schema.digest().as_str(),"fields":[{"key":"endpoint","kind":"text","required":true,"maxUtf8Bytes":2048}]}]});
    fs::write(root.join("package.json"), manifest_bytes).expect("manifest");
    fs::write(
        root.join("configuration.json"),
        serde_json::to_vec(&config).expect("config"),
    )
    .expect("config file");
    fs::write(root.join("runtime.json"), runtime).expect("runtime file");
    RuntimePackage::load(root).expect("operator package")
}
#[tokio::test]
async fn credential_endpoint_drift_rejects_before_secret_read_or_network() {
    let fixture = Fixture::new();
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("controlled target");
    listener.set_nonblocking(true).expect("nonblocking target");
    let endpoint = format!(
        "http://{}/reviewed?tenant=one",
        listener.local_addr().expect("address")
    );
    let runtime = PluginRuntime::with_packages(
        fixture.state(),
        vec![mcp_package(&fixture.0.join("package"), &endpoint)],
    )
    .expect("runtime");
    let installed = install(&runtime).await;
    let mut revision = installed.revision.expect("revision");
    let (tenant, user) = owner();
    for (index, configured) in [
        endpoint.replace("127.0.0.1", "127.0.0.2"),
        endpoint.replace("/reviewed", "/attacker"),
        endpoint.replace("tenant=one", "tenant=two"),
    ]
    .into_iter()
    .enumerate()
    {
        let response = command(&runtime, &format!("configure-{index}"), json!({"action":"configure","installation_id":installed.installation_id,"expected_revision":revision,"values":{"endpoint":configured}})).await.expect("configure");
        assert!(response.accepted);
        revision = response.revision.expect("configured revision");
        let error = runtime
            .probe(
                &tenant,
                &user,
                PluginProbeDto {
                    schema: "plugin-probe/v1".into(),
                    installation_id: installed.installation_id.clone(),
                    expected_revision: revision.clone(),
                },
            )
            .await
            .err();
        assert_eq!(
            error,
            Some(PluginError::Denied),
            "missing token would instead fail Unavailable if read"
        );
        assert_eq!(
            listener.accept().err().map(|error| error.kind()),
            Some(std::io::ErrorKind::WouldBlock),
            "no initialization reached unreviewed URL"
        );
    }
    let response = command(&runtime,"configure-exact",json!({"action":"configure","installation_id":installed.installation_id,"expected_revision":revision,"values":{"endpoint":endpoint}})).await.expect("configure exact");
    let error = runtime
        .probe(
            &tenant,
            &user,
            PluginProbeDto {
                schema: "plugin-probe/v1".into(),
                installation_id: installed.installation_id,
                expected_revision: response.revision.expect("revision"),
            },
        )
        .await
        .err();
    assert_eq!(
        error,
        Some(PluginError::Unavailable),
        "the exact URL proceeds to the intentionally missing secret"
    );
    assert_eq!(
        listener.accept().err().map(|error| error.kind()),
        Some(std::io::ErrorKind::WouldBlock)
    );
}
#[tokio::test]
async fn absent_or_changed_catalog_preserves_owner_disable_and_revoke() {
    for changed in [false, true] {
        let fixture = Fixture::new();
        let package = RuntimePackage::bundled_skill().expect("skill");
        let runtime =
            PluginRuntime::with_packages(fixture.state(), vec![package.clone()]).expect("runtime");
        let installed = install(&runtime).await;
        let revision = installed.revision.expect("revision");
        let (tenant, user) = owner();
        let probe = runtime
            .probe(
                &tenant,
                &user,
                PluginProbeDto {
                    schema: "plugin-probe/v1".into(),
                    installation_id: installed.installation_id.clone(),
                    expected_revision: revision.clone(),
                },
            )
            .await
            .expect("probe");
        assert!(command(&runtime,"grant",json!({"action":"grant","installation_id":installed.installation_id,"expected_revision":revision,"capability":"campus.public_rules.read"})).await.expect("grant").accepted);
        let enabled=command(&runtime,"enable",json!({"action":"enable","installation_id":installed.installation_id,"expected_revision":revision,"readiness_digest":probe.readiness_digest})).await.expect("enable");
        assert!(enabled.accepted);
        drop(runtime);
        let mut packages = Vec::new();
        if changed {
            let mut replacement = package;
            replacement.configuration = load_package_configuration(
                include_bytes!("../../../../market/packages/ustc.campus-guide/configuration.json"),
                &replacement.manifest,
                &CatalogRevision::parse("catalog:changed-review").expect("revision"),
            )
            .expect("changed catalog");
            packages.push(replacement);
        }
        let runtime = PluginRuntime::with_packages(fixture.state(), packages).expect("reopen");
        let list = runtime.list(&tenant, &user).await.expect("historical list");
        assert_eq!(list.packages.len(), if changed { 2 } else { 1 });
        let orphan = list
            .packages
            .iter()
            .find(|package| !package.available)
            .expect("orphan");
        assert!(orphan.fields.is_empty() && orphan.capabilities.is_empty());
        assert_eq!(orphan.name, "ustc.campus-guide");
        let historical = orphan.installation.as_ref().expect("original installation");
        assert_eq!(historical.id, installed.installation_id);
        assert_eq!(historical.state, "enabled");
        let other = UserId::parse("user:other").expect("other");
        assert!(
            runtime
                .list(&tenant, &other)
                .await
                .expect("other owner")
                .packages
                .iter()
                .all(|package| package.installation.is_none())
        );
        assert!(
            runtime
                .definitions(&tenant, &user)
                .await
                .expect("definitions")
                .is_empty()
        );
        for (request_id, intent) in [
            (
                "configure-orphan",
                json!({"action":"configure","installation_id":historical.id,"expected_revision":historical.revision,"values":{}}),
            ),
            (
                "grant-orphan",
                json!({"action":"grant","installation_id":historical.id,"expected_revision":historical.revision,"capability":"campus.public_rules.read"}),
            ),
            (
                "enable-orphan",
                json!({"action":"enable","installation_id":historical.id,"expected_revision":historical.revision,"readiness_digest":probe.readiness_digest}),
            ),
        ] {
            assert!(
                command(&runtime, request_id, intent).await.is_err(),
                "unavailable package must not admit {request_id}"
            );
        }
        assert!(
            runtime
                .probe(
                    &tenant,
                    &user,
                    PluginProbeDto {
                        schema: "plugin-probe/v1".into(),
                        installation_id: historical.id.clone(),
                        expected_revision: historical.revision.clone()
                    }
                )
                .await
                .is_err()
        );
        let disabled=command(&runtime,"disable-orphan",json!({"action":"disable","installation_id":historical.id,"expected_revision":historical.revision})).await.expect("disable orphan");
        assert!(disabled.accepted);
        let revoked=command(&runtime,"revoke-orphan",json!({"action":"revoke","installation_id":historical.id,"expected_revision":disabled.revision})).await.expect("revoke orphan");
        assert!(revoked.accepted);
        assert_eq!(revoked.state.as_deref(), Some("revoked"));
    }
}
