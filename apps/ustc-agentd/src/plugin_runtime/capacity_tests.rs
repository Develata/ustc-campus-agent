//! Capacity rejection must preserve existing authority and usable tool projections.
use super::*;
use serde_json::{Value, json};
use std::{
    fs,
    os::unix::fs::PermissionsExt,
    sync::atomic::{AtomicU64, Ordering},
};
use ustc_campus_agent_core::market::{
    admission::{EnableAdmissionRequest, MarketAdmissionService, mcp_inventory_digest},
    configuration_catalog::load_package_configuration,
    configuration_schema::ConfigurationSchema,
    load_package_manifest,
};
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "uca-plugin-capacity-{}-{}",
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
        if self.0.parent() == Some(std::env::temp_dir().as_path()) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
}
fn owner(label: &str) -> (TenantId, UserId) {
    (
        TenantId::parse(format!("tenant:{label}")).expect("valid capacity fixture"),
        UserId::parse(format!("user:{label}")).expect("valid capacity fixture"),
    )
}
async fn command(
    runtime: &PluginRuntime,
    owner: &(TenantId, UserId),
    request: &str,
    intent: Value,
) -> Result<PluginCommandResultDto, PluginError> {
    runtime
        .command(
            &owner.0,
            &owner.1,
            serde_json::from_value(
                json!({"schema":"plugin-command/v1","request_id":request,"intent":intent}),
            )
            .expect("valid capacity fixture"),
        )
        .await
}
// Checked package/catalog fixture: discovery is seeded locally so the admission proof needs no network.
fn package(count: usize) -> RuntimePackage {
    let mut raw: Value = serde_json::from_slice(include_bytes!(
        "../../../../market/packages/ustc.campus-guide/package.json"
    ))
    .expect("valid capacity fixture");
    raw["id"] = json!("synthetic.capacity");
    raw["components"] = json!([{"type":"McpServerComponent","path":"runtime.json"}]);
    let manifest =
        load_package_manifest(&serde_json::to_vec(&raw).expect("valid capacity fixture"))
            .expect("valid capacity fixture");
    let schema = ConfigurationSchema::new(Vec::new()).expect("valid capacity fixture");
    let config = json!({"schemaVersion":"package-component-configuration/v1","packageId":manifest.package_id().as_str(),"packageVersion":manifest.package_version().as_str(),"packageDigest":manifest.package_digest().as_str(),"componentSetDigest":manifest.component_declaration_set_digest().as_str(),"capabilityManifestDigest":manifest.capability_manifest_digest().as_str(),"components":[{"path":"runtime.json","type":"McpServerComponent","mode":null,"componentId":"component:mcp","componentVersion":"1","componentDigest":Sha256Digest::from_bytes(b"capacity-reviewed-artifact").as_str(),"executionIdentity":"execution:mcp","schemaDigest":schema.digest().as_str(),"fields":[]}]});
    let configuration = load_package_configuration(
        &serde_json::to_vec(&config).expect("valid capacity fixture"),
        &manifest,
        &CatalogRevision::parse("catalog:capacity").expect("valid capacity fixture"),
    )
    .expect("valid capacity fixture");
    RuntimePackage {
        manifest,
        configuration,
        component: RuntimeComponent::Mcp {
            endpoint_key: ConfigurationKey::parse("endpoint").expect("valid capacity fixture"),
            tools: (0..count)
                .map(|i| {
                    (
                        format!("read_{i}"),
                        CapabilityId::parse("campus.public_rules.read")
                            .expect("valid capacity fixture"),
                    )
                })
                .collect(),
            endpoint_policy: ustc_campus_agent_adapters::mcp::EndpointPolicy::LoopbackDevelopment,
            bearer_file: None,
            credential_endpoint: None,
        },
    }
}
async fn prepare(
    runtime: &PluginRuntime,
    owner: &(TenantId, UserId),
    index: usize,
) -> (PluginInstallationDto, PluginProbeResultDto) {
    let pin = runtime.packages[index].configuration.package_pin();
    let key = format!("{index}");
    let installed=command(runtime,owner,&format!("install-{key}"),json!({"action":"install","package_id":pin.package_id().as_str(),"version":pin.package_version().as_str(),"catalog_revision":pin.catalog_revision().as_str(),"package_digest":pin.package_digest().as_str()})).await.expect("valid capacity fixture");
    assert!(installed.accepted);
    let id = InstallationId::parse(installed.installation_id).expect("valid capacity fixture");
    let probe = if let RuntimeComponent::Mcp { tools, .. } = &runtime.packages[index].component {
        let mut state = runtime.state.lock().await;
        let current = state
            .owned(&owner.0, &owner.1, &id)
            .expect("valid capacity fixture");
        let binding = runtime.packages[index]
            .configuration
            .bindings()
            .values()
            .next()
            .expect("valid capacity fixture");
        let schema = ValidatedToolInputSchemaV0::try_from(UnvalidatedToolInputSchemaV0 {
            dialect: "tool-input-schema/v0".into(),
            root: UnvalidatedSchemaNodeV0::Object {
                properties: vec![],
                required: vec![],
            },
        })
        .expect("valid capacity fixture");
        let definitions: Vec<_> = tools
            .iter()
            .map(|(name, capability)| CatalogToolDefinition {
                id: ToolId::parse(format!("tool:{name}")).expect("valid capacity fixture"),
                model_visible_name: tool_name(&id, name),
                description: "Synthetic capacity tool".into(),
                capability_id: capability.clone(),
                input_schema: Some(schema.clone()),
                claimed_input_schema_digest: schema.digest().clone(),
            })
            .collect();
        let digest = mcp_inventory_digest(binding, current.configuration(), &definitions)
            .expect("valid capacity fixture");
        let readiness =
            ComponentReadiness::mcp(binding, current.configuration(), &definitions, &digest)
                .expect("valid capacity fixture");
        let result = PluginProbeResultDto {
            schema: "plugin-probe-result/v1",
            installation_id: id.as_str().into(),
            revision: current.revision().as_str().into(),
            readiness_digest: readiness.digest().as_str().into(),
            kind: "mcp".into(),
            tools: vec![],
        };
        state.probes.insert(
            id.clone(),
            ProbedComponent {
                revision: current.revision().clone(),
                readiness,
                tools: definitions,
                wire_names: BTreeMap::new(),
                client: None,
                transport_digest: None,
            },
        );
        result
    } else {
        runtime
            .probe(
                &owner.0,
                &owner.1,
                PluginProbeDto {
                    schema: "plugin-probe/v1".into(),
                    installation_id: id.as_str().into(),
                    expected_revision: installed.revision.expect("valid capacity fixture"),
                },
            )
            .await
            .expect("valid capacity fixture")
    };
    assert!(command(runtime,owner,&format!("grant-{key}"),json!({"action":"grant","installation_id":id.as_str(),"expected_revision":probe.revision,"capability":"campus.public_rules.read"})).await.expect("valid capacity fixture").accepted);
    let view = runtime
        .list(&owner.0, &owner.1)
        .await
        .expect("valid capacity fixture")
        .packages
        .remove(index)
        .installation
        .expect("valid capacity fixture");
    (view, probe)
}
fn enable_intent(view: &PluginInstallationDto, probe: &PluginProbeResultDto) -> Value {
    json!({"action":"enable","installation_id":view.id,"expected_revision":view.revision,"readiness_digest":probe.readiness_digest})
}
#[tokio::test]
async fn aggregate_tool_capacity_rejects_enable_without_losing_existing_tools_or_receipts() {
    let f = Fixture::new();
    let runtime = PluginRuntime::with_packages(
        f.state(),
        vec![
            package(28),
            RuntimePackage::bundled_skill().expect("valid capacity fixture"),
        ],
    )
    .expect("valid capacity fixture");
    let owned = owner("capacity");
    let (first, first_probe) = prepare(&runtime, &owned, 0).await;
    let first_enable = enable_intent(&first, &first_probe);
    assert!(
        command(&runtime, &owned, "enable-first", first_enable.clone())
            .await
            .expect("valid capacity fixture")
            .accepted
    );
    assert_eq!(
        runtime
            .session(&owned.0, &owned.1)
            .await
            .expect("valid capacity fixture")
            .definitions()
            .len(),
        28
    );
    let (second, second_probe) = prepare(&runtime, &owned, 1).await;
    let before = fs::read(f.state()).expect("valid capacity fixture");
    assert_eq!(
        command(
            &runtime,
            &owned,
            "enable-overflow",
            enable_intent(&second, &second_probe)
        )
        .await
        .err(),
        Some(PluginError::Capacity)
    );
    assert_eq!(fs::read(f.state()).expect("valid capacity fixture"), before);
    assert_eq!(
        runtime
            .session(&owned.0, &owned.1)
            .await
            .expect("valid capacity fixture")
            .definitions()
            .len(),
        28
    );
    assert!(
        command(&runtime, &owned, "enable-first", first_enable.clone())
            .await
            .expect("valid capacity fixture")
            .replayed
    );
    let other = owner("other");
    let (other_view, other_probe) = prepare(&runtime, &other, 1).await;
    assert!(
        command(
            &runtime,
            &other,
            "enable-other",
            enable_intent(&other_view, &other_probe)
        )
        .await
        .expect("valid capacity fixture")
        .accepted
    );
    assert_eq!(
        runtime
            .session(&other.0, &other.1)
            .await
            .expect("valid capacity fixture")
            .definitions()
            .len(),
        1
    );
    drop(runtime);
    let runtime = PluginRuntime::with_packages(
        f.state(),
        vec![RuntimePackage::bundled_skill().expect("valid capacity fixture")],
    )
    .expect("valid capacity fixture");
    let reprobe = runtime
        .probe(
            &owned.0,
            &owned.1,
            PluginProbeDto {
                schema: "plugin-probe/v1".into(),
                installation_id: second.id.clone(),
                expected_revision: second.revision.clone(),
            },
        )
        .await
        .expect("valid capacity fixture");
    assert!(
        command(
            &runtime,
            &owned,
            "enable-overflow",
            enable_intent(&second, &reprobe)
        )
        .await
        .expect("valid capacity fixture")
        .accepted,
        "orphan pins do not reserve usable tools"
    );
    assert!(
        command(&runtime, &owned, "enable-first", first_enable)
            .await
            .expect("valid capacity fixture")
            .replayed,
        "old receipt precedes current catalog/capacity"
    );
    assert_eq!(
        runtime
            .session(&owned.0, &owned.1)
            .await
            .expect("valid capacity fixture")
            .definitions()
            .len(),
        1
    );
}
#[tokio::test]
async fn known_prewrite_capacity_does_not_poison_or_change_authority() {
    let f = Fixture::new();
    let runtime = PluginRuntime::with_packages(
        f.state(),
        vec![RuntimePackage::bundled_skill().expect("valid capacity fixture")],
    )
    .expect("valid capacity fixture");
    let owned = owner("disk-capacity");
    let (view, probe) = prepare(&runtime, &owned, 0).await;
    assert!(
        command(&runtime, &owned, "enable", enable_intent(&view, &probe))
            .await
            .expect("valid capacity fixture")
            .accepted
    );
    let session = runtime
        .session(&owned.0, &owned.1)
        .await
        .expect("valid capacity fixture");
    let result = runtime
        .execute_frozen(
            &session,
            session
                .bindings
                .keys()
                .next()
                .expect("valid capacity fixture"),
            json!({"resource":"skills/campus-guide/SKILL.md"}),
        )
        .await;
    assert_eq!(
        result.status(),
        crate::chat_tools::ChatToolStatus::Succeeded
    );
    let before = fs::read(f.state()).expect("valid capacity fixture");
    let mut state = runtime.state.lock().await;
    let mut oversized = state.authority.clone();
    oversized.runs = vec![oversized.runs[0].clone(); 1025];
    assert_eq!(state.commit(oversized), Err(PluginError::Capacity));
    assert!(
        !state.poisoned,
        "capacity is detected before filesystem writes"
    );
    assert_eq!(state.authority.runs.len(), 1);
    assert_eq!(fs::read(f.state()).expect("valid capacity fixture"), before);
    let mut oversized = state.authority.clone();
    let current = state
        .owned(
            &owned.0,
            &owned.1,
            &InstallationId::parse(view.id.clone()).expect("valid capacity fixture"),
        )
        .expect("valid capacity fixture");
    for index in 0..4096 {
        oversized
            .installations
            .execute(
                InstallationCommand::disable(
                    InstallationCommandId::parse(format!("cmd:capacity-{index}"))
                        .expect("valid capacity fixture"),
                    current.installation_id().clone(),
                    current.revision().clone(),
                )
                .expect("valid capacity fixture"),
            )
            .expect("valid capacity fixture");
    }
    assert_eq!(
        state.commit(oversized),
        Err(PluginError::Capacity),
        "codec TooLarge is a known pre-write capacity refusal"
    );
    assert!(!state.poisoned);
    assert_eq!(fs::read(f.state()).expect("valid capacity fixture"), before);
    drop(state);
    assert!(runtime.list(&owned.0, &owned.1).await.is_ok());
    let current = runtime
        .list(&owned.0, &owned.1)
        .await
        .expect("valid capacity fixture")
        .packages
        .remove(0)
        .installation
        .expect("valid capacity fixture");
    assert!(command(&runtime,&owned,"disable-after-capacity",json!({"action":"disable","installation_id":current.id,"expected_revision":current.revision})).await.expect("valid capacity fixture").accepted);
}

#[tokio::test]
async fn legacy_overcapacity_reports_capacity_but_ungranted_tools_do_not_count() {
    let f = Fixture::new();
    let runtime = PluginRuntime::with_packages(
        f.state(),
        vec![
            package(28),
            RuntimePackage::bundled_skill().expect("valid capacity fixture"),
        ],
    )
    .expect("valid capacity fixture");
    let owned = (0..100)
        .map(|i| owner(&format!("legacy-{i}")))
        .find(|o| {
            stable_id(&[o.0.as_str(), o.1.as_str(), "synthetic.capacity", "0.1.0"])
                < stable_id(&[o.0.as_str(), o.1.as_str(), "ustc.campus-guide", "0.1.0"])
        })
        .expect("valid capacity fixture");
    let (first, first_probe) = prepare(&runtime, &owned, 0).await;
    assert!(
        command(
            &runtime,
            &owned,
            "enable-first",
            enable_intent(&first, &first_probe)
        )
        .await
        .expect("valid capacity fixture")
        .accepted
    );
    let (second, _) = prepare(&runtime, &owned, 1).await;
    let id = InstallationId::parse(second.id.clone()).expect("valid capacity fixture");
    {
        // Reproduce a pre-fix durable state through the original checked M20 issuer.
        let mut state = runtime.state.lock().await;
        let mut next = state.authority.clone();
        let package = &runtime.packages[1];
        let service = MarketAdmissionService::new(
            &owned.0,
            &owned.1,
            &package.manifest,
            &package.configuration,
            &runtime.registry,
        )
        .expect("valid capacity fixture");
        service
            .enable(
                &mut next.installations,
                &next.grants,
                EnableAdmissionRequest {
                    installation_id: id.clone(),
                    expected_revision: InstallationRevision::parse(second.revision.clone())
                        .expect("valid capacity fixture"),
                    command_id: InstallationCommandId::parse("cmd:legacy-overflow")
                        .expect("valid capacity fixture"),
                },
                &state.probes[&id].readiness,
            )
            .expect("valid capacity fixture");
        state.commit(next).expect("valid capacity fixture");
    }
    assert_eq!(
        runtime.session(&owned.0, &owned.1).await.err(),
        Some(PluginError::Capacity)
    );
    {
        let mut state = runtime.state.lock().await;
        let mut next = state.authority.clone();
        let grant = next
            .grants
            .load_current_for_authority(
                &owned.0,
                &owned.1,
                &id,
                &CapabilityId::parse("campus.public_rules.read").expect("valid capacity fixture"),
                &GrantScope::campus_public().expect("valid capacity fixture"),
            )
            .expect("valid capacity fixture")
            .expect("valid capacity fixture");
        next.grants
            .execute(
                GrantCommand::revoke(
                    GrantCommandId::parse("grant-cmd:legacy-revoke")
                        .expect("valid capacity fixture"),
                    grant.snapshot_id().clone(),
                    grant.version().clone(),
                )
                .expect("valid capacity fixture"),
            )
            .expect("valid capacity fixture");
        state.commit(next).expect("valid capacity fixture");
    }
    assert_eq!(
        runtime
            .session(&owned.0, &owned.1)
            .await
            .expect("valid capacity fixture")
            .definitions()
            .len(),
        28,
        "a denied trailing tool must not hide the admitted 28"
    );
}
