//! Real application lifecycle against original M20 ledgers and durable M30 evidence.
use super::*;
use crate::chat_tools::{ChatToolCatalog, ChatToolStatus};
use serde_json::{Value, json};
use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

static NEXT: AtomicU64 = AtomicU64::new(0);
struct TempRoot(PathBuf);
impl TempRoot {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "uca-plugin-runtime-test-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).expect("new owned fixture directory");
        fs::set_permissions(&path, fs::Permissions::from_mode(0o700))
            .expect("private fixture directory");
        Self(path)
    }
    fn state(&self) -> PathBuf {
        self.0.join("state/authority.bin")
    }
}
impl Drop for TempRoot {
    fn drop(&mut self) {
        let allowed = std::env::temp_dir().canonicalize().expect("temp root");
        if self.0.parent().and_then(|p| p.canonicalize().ok()).as_ref() == Some(&allowed)
            && self.0.file_name().is_some_and(|name| {
                name.to_string_lossy()
                    .starts_with("uca-plugin-runtime-test-")
            })
        {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
}
fn owner(label: &str) -> (TenantId, UserId) {
    (
        TenantId::parse(format!("tenant:{label}")).expect("tenant"),
        UserId::parse(format!("user:{label}")).expect("user"),
    )
}
fn skill_runtime(path: PathBuf) -> PluginRuntime {
    PluginRuntime::with_packages(
        path,
        vec![RuntimePackage::bundled_skill().expect("bundled skill")],
    )
    .expect("runtime")
}
async fn command(
    runtime: &PluginRuntime,
    owner: &(TenantId, UserId),
    request_id: &str,
    intent: Value,
) -> Result<PluginCommandResultDto, PluginError> {
    runtime
        .command(
            &owner.0,
            &owner.1,
            serde_json::from_value(
                json!({"schema":"plugin-command/v1","request_id":request_id,"intent":intent}),
            )
            .expect("command shape"),
        )
        .await
}
async fn install(runtime: &PluginRuntime, owner: &(TenantId, UserId)) -> PluginCommandResultDto {
    let package = &runtime.packages[0];
    let pin = package.configuration.package_pin();
    command(runtime,owner,"install",json!({"action":"install","package_id":pin.package_id().as_str(),"version":pin.package_version().as_str(),"catalog_revision":pin.catalog_revision().as_str(),"package_digest":pin.package_digest().as_str()})).await.expect("install")
}
async fn snapshot(runtime: &PluginRuntime, owner: &(TenantId, UserId)) -> PluginInstallationDto {
    runtime
        .list(&owner.0, &owner.1)
        .await
        .expect("list")
        .packages
        .remove(0)
        .installation
        .expect("installation")
}
async fn probe(
    runtime: &PluginRuntime,
    owner: &(TenantId, UserId),
    installation: &PluginInstallationDto,
) -> Result<PluginProbeResultDto, PluginError> {
    runtime
        .probe(
            &owner.0,
            &owner.1,
            PluginProbeDto {
                schema: "plugin-probe/v1".into(),
                installation_id: installation.id.clone(),
                expected_revision: installation.revision.clone(),
            },
        )
        .await
}
async fn configure(
    runtime: &PluginRuntime,
    owner: &(TenantId, UserId),
    values: Value,
) -> PluginInstallationDto {
    let installed = snapshot(runtime, owner).await;
    let response=command(runtime,owner,"configure",json!({"action":"configure","installation_id":installed.id,"expected_revision":installed.revision,"values":values})).await.expect("configure");
    assert!(response.accepted);
    snapshot(runtime, owner).await
}
async fn grant_enable(
    runtime: &PluginRuntime,
    owner: &(TenantId, UserId),
    installed: &PluginInstallationDto,
    probe: &PluginProbeResultDto,
) -> PluginInstallationDto {
    let grant=command(runtime,owner,"grant",json!({"action":"grant","installation_id":installed.id,"expected_revision":installed.revision,"capability":"campus.public_rules.read"})).await.expect("grant");
    assert!(grant.accepted);
    let enabled=command(runtime,owner,"enable",json!({"action":"enable","installation_id":installed.id,"expected_revision":installed.revision,"readiness_digest":probe.readiness_digest})).await.expect("enable");
    assert!(enabled.accepted);
    snapshot(runtime, owner).await
}
async fn projected_name(runtime: &PluginRuntime, owner: &(TenantId, UserId)) -> String {
    let mut catalog = ChatToolCatalog::without_opportunity();
    let definitions = runtime
        .definitions(&owner.0, &owner.1)
        .await
        .expect("current definitions");
    assert_eq!(definitions.len(), 1);
    catalog
        .register_dynamic(definitions)
        .expect("neutral catalog");
    catalog
        .definitions()
        .into_iter()
        .find(|d| d.name.starts_with("plugin_"))
        .expect("plugin definition")
        .name
}
async fn assert_journal(runtime: &PluginRuntime, count: usize) {
    let state = runtime.state.lock().await;
    assert_eq!(state.authority.runs.len(), count);
    for run in &state.authority.runs {
        run.validate().expect("M30 exact replay");
        let value = serde_json::to_value(run).expect("journal value");
        let events = value["events"].as_array().expect("events");
        let intent = events
            .iter()
            .position(|event| event["kind"]["type"] == "effect_intent_persisted")
            .expect("intent");
        let receipt = events
            .iter()
            .position(|event| event["kind"]["type"] == "effect_receipt_persisted")
            .expect("receipt");
        assert!(intent < receipt);
        assert!(
            ["completed", "failed"].contains(
                &events.last().expect("terminal")["kind"]["type"]
                    .as_str()
                    .expect("kind")
            )
        );
        assert!(
            !value.to_string().contains("学生竞赛"),
            "private payloads stay outside evidence journal"
        );
    }
}

#[tokio::test]
async fn skill_lifecycle_uses_current_authority_and_survives_restart() {
    let temp = TempRoot::new();
    let owned = owner("a");
    let runtime = skill_runtime(temp.state());
    assert!(
        runtime
            .definitions(&owned.0, &owned.1)
            .await
            .expect("empty")
            .is_empty()
    );
    assert!(install(&runtime, &owned).await.accepted);
    let installed = configure(&runtime, &owned, json!({})).await;
    let discovered = probe(&runtime, &owned, &installed)
        .await
        .expect("skill probe");
    assert_eq!(discovered.kind, "skill");
    assert_eq!(discovered.tools.len(), 1);
    assert_eq!(command(&runtime,&owned,"premature-enable",json!({"action":"enable","installation_id":installed.id,"expected_revision":installed.revision,"readiness_digest":discovered.readiness_digest})).await.err(),Some(PluginError::Denied));
    let enabled = grant_enable(&runtime, &owned, &installed, &discovered).await;
    assert_eq!(enabled.state, "enabled");
    let name = projected_name(&runtime, &owned).await;
    let output = runtime
        .execute(
            &owned.0,
            &owned.1,
            &name,
            json!({"resource":"skills/campus-guide/SKILL.md"}),
        )
        .await;
    assert_eq!(output.status(), ChatToolStatus::Succeeded);
    let payload: Value =
        serde_json::from_str(&output.serialize_for_provider().expect("bounded result"))
            .expect("result");
    assert_eq!(payload["trust"], "untrusted_data");
    assert_eq!(payload["data"]["instruction_authority"], "none");
    assert_journal(&runtime, 1).await;
    drop(runtime);
    let runtime = skill_runtime(temp.state());
    assert_eq!(projected_name(&runtime, &owned).await, name);
    assert_journal(&runtime, 1).await;
    assert_eq!(
        runtime
            .execute(
                &owned.0,
                &owned.1,
                &name,
                json!({"resource":"skills/campus-guide/SKILL.md"})
            )
            .await
            .status(),
        ChatToolStatus::Succeeded
    );
    assert_journal(&runtime, 2).await;
    assert!(command(&runtime,&owned,"disable",json!({"action":"disable","installation_id":enabled.id,"expected_revision":enabled.revision})).await.expect("disable").accepted);
    assert!(
        runtime
            .definitions(&owned.0, &owned.1)
            .await
            .expect("disabled defs")
            .is_empty()
    );
    assert_eq!(
        runtime
            .execute(
                &owned.0,
                &owned.1,
                &name,
                json!({"resource":"skills/campus-guide/SKILL.md"})
            )
            .await
            .status(),
        ChatToolStatus::Denied
    );
    assert_journal(&runtime, 2).await;
}

#[tokio::test]
async fn owner_isolation_revocation_and_stale_intents_are_closed() {
    let temp = TempRoot::new();
    let owned = owner("a");
    let other = owner("b");
    let runtime = skill_runtime(temp.state());
    install(&runtime, &owned).await;
    let installed = snapshot(&runtime, &owned).await;
    let discovered = probe(&runtime, &owned, &installed).await.expect("probe");
    let enabled = grant_enable(&runtime, &owned, &installed, &discovered).await;
    let name = projected_name(&runtime, &owned).await;
    assert!(
        runtime
            .list(&other.0, &other.1)
            .await
            .expect("other list")
            .packages[0]
            .installation
            .is_none()
    );
    assert!(
        runtime
            .definitions(&other.0, &other.1)
            .await
            .expect("other definitions")
            .is_empty()
    );
    assert_eq!(
        probe(&runtime, &other, &enabled).await.err(),
        Some(PluginError::NotFound)
    );
    assert_eq!(command(&runtime,&other,"foreign-disable",json!({"action":"disable","installation_id":enabled.id,"expected_revision":enabled.revision})).await.err(),Some(PluginError::NotFound));
    assert_eq!(
        runtime
            .execute(
                &other.0,
                &other.1,
                &name,
                json!({"resource":"skills/campus-guide/SKILL.md"})
            )
            .await
            .status(),
        ChatToolStatus::Denied
    );
    assert_eq!(command(&runtime,&owned,"stale-disable",json!({"action":"disable","installation_id":enabled.id,"expected_revision":installed.revision})).await.err(),Some(PluginError::Conflict));
    assert!(command(&runtime,&owned,"revoke",json!({"action":"revoke","installation_id":enabled.id,"expected_revision":enabled.revision})).await.expect("revoke").accepted);
    assert!(
        runtime
            .definitions(&owned.0, &owned.1)
            .await
            .expect("revoked definitions")
            .is_empty()
    );
    assert_eq!(
        runtime
            .execute(
                &owned.0,
                &owned.1,
                &name,
                json!({"resource":"skills/campus-guide/SKILL.md"})
            )
            .await
            .status(),
        ChatToolStatus::Denied
    );
    assert_journal(&runtime, 0).await;
}

#[tokio::test]
async fn commands_retry_exactly_across_restart_and_changed_payload_conflicts() {
    let temp = TempRoot::new();
    let owned = owner("a");
    let runtime = skill_runtime(temp.state());
    let first = install(&runtime, &owned).await;
    let repeated = install(&runtime, &owned).await;
    assert!(repeated.accepted && repeated.replayed);
    assert_eq!(first.revision, repeated.revision);
    let pin = runtime.packages[0].configuration.package_pin();
    assert_eq!(command(&runtime,&owned,"install",json!({"action":"install","package_id":pin.package_id().as_str(),"version":"99.0.0","catalog_revision":pin.catalog_revision().as_str(),"package_digest":pin.package_digest().as_str()})).await.err(),Some(PluginError::Conflict));
    let installed = snapshot(&runtime, &owned).await;
    let intent = json!({"action":"grant","installation_id":installed.id,"expected_revision":installed.revision,"capability":"campus.public_rules.read"});
    assert!(
        command(&runtime, &owned, "grant", intent.clone())
            .await
            .expect("grant")
            .accepted
    );
    assert!(
        command(&runtime, &owned, "grant", intent.clone())
            .await
            .expect("grant replay")
            .replayed
    );
    let mut changed = intent.clone();
    changed["capability"] = json!("user.own_calendar_items.write");
    assert_eq!(
        command(&runtime, &owned, "grant", changed).await.err(),
        Some(PluginError::Conflict)
    );
    drop(runtime);
    let runtime = skill_runtime(temp.state());
    assert!(install(&runtime, &owned).await.replayed);
    assert!(
        command(&runtime, &owned, "grant", intent)
            .await
            .expect("durable replay")
            .replayed
    );
}

#[tokio::test]
async fn corrupt_missing_unsafe_or_concurrently_open_state_never_resets_authority() {
    for case in ["corrupt", "missing", "unsafe"] {
        let temp = TempRoot::new();
        let runtime = skill_runtime(temp.state());
        assert_eq!(
            PluginRuntime::with_packages(
                temp.state(),
                vec![RuntimePackage::bundled_skill().expect("skill")]
            )
            .err(),
            Some(PluginError::Unavailable)
        );
        drop(runtime);
        match case {
            "corrupt" => {
                fs::write(temp.state(), b"not authoritative state").expect("corrupt owned fixture")
            }
            "missing" => fs::remove_file(temp.state()).expect("remove owned fixture state"),
            _ => fs::set_permissions(temp.state(), fs::Permissions::from_mode(0o644))
                .expect("unsafe fixture"),
        }
        assert_eq!(
            PluginRuntime::with_packages(
                temp.state(),
                vec![RuntimePackage::bundled_skill().expect("skill")]
            )
            .err(),
            Some(PluginError::Unavailable),
            "{case}"
        );
    }
    let temp = TempRoot::new();
    let runtime = skill_runtime(temp.state());
    let owned = owner("a");
    fs::set_permissions(temp.state(), fs::Permissions::from_mode(0o644)).expect("unsafe fixture");
    let pin = runtime.packages[0].configuration.package_pin();
    assert_eq!(command(&runtime,&owned,"install",json!({"action":"install","package_id":pin.package_id().as_str(),"version":pin.package_version().as_str(),"catalog_revision":pin.catalog_revision().as_str(),"package_digest":pin.package_digest().as_str()})).await.err(),Some(PluginError::Unavailable));
    assert!(runtime.state.lock().await.poisoned);
    assert_eq!(
        runtime.list(&owned.0, &owned.1).await.err(),
        Some(PluginError::Unavailable)
    );
}

fn write_mcp_package(root: &Path) -> RuntimePackage {
    use ustc_campus_agent_core::market::{
        configuration_schema::{ConfigurationFieldSchema, ConfigurationSchema},
        load_package_manifest,
    };
    fs::create_dir(root).expect("new operator fixture package");
    let mut manifest: Value = serde_json::from_slice(include_bytes!(
        "../../../../market/packages/ustc.campus-guide/package.json"
    ))
    .expect("base manifest");
    manifest["id"] = json!("synthetic.mcp-read");
    manifest["displayName"] = json!("Synthetic MCP fixture");
    manifest["components"] = json!([{"type":"McpServerComponent","path":"runtime.json"}]);
    let manifest = serde_json::to_vec(&manifest).expect("manifest bytes");
    let checked = load_package_manifest(&manifest).expect("manifest");
    let runtime=serde_json::to_vec(&json!({"schemaVersion":"plugin-runtime/v1","kind":"mcp","endpointKey":"endpoint","endpointPolicy":"loopback_development","tools":[{"name":"campus_read","capabilityId":"campus.public_rules.read"}]})).expect("runtime bytes");
    let schema = ConfigurationSchema::new(vec![
        ConfigurationFieldSchema::text(
            ConfigurationKey::parse("endpoint").expect("key"),
            true,
            2048,
        )
        .expect("field"),
    ])
    .expect("schema");
    let configuration = json!({"schemaVersion":"package-component-configuration/v1","packageId":checked.package_id().as_str(),"packageVersion":checked.package_version().as_str(),"packageDigest":checked.package_digest().as_str(),"componentSetDigest":checked.component_declaration_set_digest().as_str(),"capabilityManifestDigest":checked.capability_manifest_digest().as_str(),"components":[{"path":"runtime.json","type":"McpServerComponent","mode":null,"componentId":"component:mcp","componentVersion":"1","componentDigest":Sha256Digest::from_bytes(&runtime).as_str(),"executionIdentity":"execution:mcp","schemaDigest":schema.digest().as_str(),"fields":[{"key":"endpoint","kind":"text","required":true,"maxUtf8Bytes":2048}]}]});
    for (name, bytes) in [
        ("package.json", manifest),
        (
            "configuration.json",
            serde_json::to_vec(&configuration).expect("configuration bytes"),
        ),
        ("runtime.json", runtime),
    ] {
        fs::write(root.join(name), bytes).expect("operator-selected fixture file");
    }
    RuntimePackage::load(root).expect("real operator package loader")
}

struct PeerControl {
    stop: std::sync::atomic::AtomicBool,
    drift: std::sync::atomic::AtomicBool,
    expire_next: std::sync::atomic::AtomicBool,
    business_fault: std::sync::atomic::AtomicUsize,
    calls: std::sync::atomic::AtomicUsize,
    observed: std::sync::Mutex<Vec<String>>,
    intent_before_call: std::sync::atomic::AtomicBool,
    session_limit: std::sync::atomic::AtomicUsize,
    fail_delete: std::sync::atomic::AtomicBool,
    unbound_tool: std::sync::atomic::AtomicBool,
    nul_description: std::sync::atomic::AtomicBool,
}
struct McpPeer {
    endpoint: String,
    control: Arc<PeerControl>,
    worker: Option<std::thread::JoinHandle<()>>,
}
impl McpPeer {
    fn start(authority_path: PathBuf) -> Self {
        use std::{
            io::{Read, Write},
            net::TcpListener,
            time::Duration,
        };
        let listener = TcpListener::bind("127.0.0.1:0").expect("controlled MCP endpoint");
        listener.set_nonblocking(true).expect("nonblocking");
        let endpoint = format!("http://{}/mcp", listener.local_addr().expect("address"));
        let control = Arc::new(PeerControl {
            stop: false.into(),
            drift: false.into(),
            expire_next: false.into(),
            business_fault: 0.into(),
            calls: 0.into(),
            observed: std::sync::Mutex::new(Vec::new()),
            intent_before_call: true.into(),
            session_limit: usize::MAX.into(),
            fail_delete: false.into(),
            unbound_tool: false.into(),
            nul_description: false.into(),
        });
        let watched = Arc::clone(&control);
        let worker = std::thread::spawn(move || {
            let mut session = 0usize;
            let mut active_sessions = std::collections::BTreeSet::new();
            while !watched.stop.load(Ordering::SeqCst) {
                let mut stream = match listener.accept() {
                    Ok((stream, _)) => stream,
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        std::thread::sleep(Duration::from_millis(2));
                        continue;
                    }
                    Err(_) => panic!("controlled MCP accept"),
                };
                stream
                    .set_read_timeout(Some(Duration::from_secs(3)))
                    .expect("read timeout");
                let mut request = Vec::new();
                let mut buffer = [0u8; 4096];
                let end = loop {
                    let count = stream.read(&mut buffer).expect("headers");
                    assert!(count > 0);
                    request.extend_from_slice(&buffer[..count]);
                    if let Some(end) = request.windows(4).position(|w| w == b"\r\n\r\n") {
                        break end + 4;
                    }
                    assert!(request.len() < 16384);
                };
                let headers = String::from_utf8_lossy(&request[..end]).to_ascii_lowercase();
                let length = headers
                    .lines()
                    .find_map(|line| line.strip_prefix("content-length:"))
                    .unwrap_or("0")
                    .trim()
                    .parse::<usize>()
                    .expect("content length");
                while request.len() < end + length {
                    let count = stream.read(&mut buffer).expect("body");
                    assert!(count > 0);
                    request.extend_from_slice(&buffer[..count]);
                }
                if headers.starts_with("delete ") {
                    watched
                        .observed
                        .lock()
                        .expect("observed")
                        .push("DELETE".to_owned());
                    let status = if watched.fail_delete.load(Ordering::SeqCst) {
                        500
                    } else {
                        let session = headers
                            .lines()
                            .find_map(|line| line.strip_prefix("mcp-session-id:"))
                            .expect("session header")
                            .trim();
                        assert!(
                            active_sessions.remove(session),
                            "close belongs to one active session"
                        );
                        204
                    };
                    write!(stream, "HTTP/1.1 {status} Synthetic\r\nContent-Length: 0\r\nConnection: close\r\n\r\n").expect("close response");
                    continue;
                }
                let request: Value =
                    serde_json::from_slice(&request[end..end + length]).expect("MCP JSON");
                let method = request["method"].as_str().expect("method");
                watched
                    .observed
                    .lock()
                    .expect("observed")
                    .push(method.to_owned());
                let mut status = 200;
                let mut extra = String::new();
                let result = match method {
                    "initialize" => {
                        assert!(!headers.contains("mcp-session-id"));
                        if active_sessions.len() >= watched.session_limit.load(Ordering::SeqCst) {
                            status = 503;
                            Value::Null
                        } else {
                            session += 1;
                            active_sessions.insert(format!("synthetic-session-{session}"));
                            extra = format!("MCP-Session-Id: synthetic-session-{session}\r\n");
                            json!({"protocolVersion":"2025-11-25","capabilities":{"tools":{}},"serverInfo":{"name":"synthetic-mcp","version":"1"}})
                        }
                    }
                    "notifications/initialized" => {
                        status = 202;
                        Value::Null
                    }
                    "tools/list" => {
                        json!({"tools":[{"name":if watched.unbound_tool.load(Ordering::SeqCst){"unbound_read"}else{"campus_read"},"description":if watched.drift.load(Ordering::SeqCst){"Changed synthetic definition"}else if watched.nul_description.load(Ordering::SeqCst){"Synthetic\0 public read"}else{"Synthetic public read"},"inputSchema":{"type":"object","properties":if watched.drift.load(Ordering::SeqCst){json!({"query":{"type":"string"},"mode":{"type":"string"}})}else{json!({"query":{"type":"string"}})},"required":["query"],"additionalProperties":false}}]})
                    }
                    "tools/call" => {
                        assert!(
                            headers
                                .contains(&format!("mcp-session-id: synthetic-session-{session}"))
                        );
                        assert_eq!(request["params"]["name"], "campus_read");
                        watched.calls.fetch_add(1, Ordering::SeqCst);
                        let disk = fs::read(&authority_path)
                            .expect("intent durable before network execution");
                        let text = String::from_utf8_lossy(&disk);
                        let pending = text.matches("effect_intent_persisted").count()
                            > text.matches("effect_receipt_persisted").count();
                        watched
                            .intent_before_call
                            .fetch_and(pending, Ordering::SeqCst);
                        match watched.business_fault.swap(0, Ordering::SeqCst) {
                            1 => {
                                let body = "data: {\"jsonrpc\":\"2.0\",\"method\":\"notifications/tools/list_changed\"}\n\n";
                                write!(stream, "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len()).expect("drift notification");
                                continue;
                            }
                            2 => {
                                status = 503;
                            }
                            3 => {
                                let body = json!({"jsonrpc":"2.0","id":request["id"],"result":{"isError":true,"content":[{"type":"text","text":"Synthetic business rejection"}]}}).to_string();
                                write!(stream, "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len()).expect("business error");
                                continue;
                            }
                            _ => {}
                        }
                        if watched.expire_next.swap(false, Ordering::SeqCst) {
                            status = 404;
                            Value::Null
                        } else {
                            json!({"content":[{"type":"text","text":"Synthetic MCP completed"}],"structuredContent":{"synthetic":true}})
                        }
                    }
                    _ => panic!("unexpected MCP method"),
                };
                let body = if status == 202 || status == 404 {
                    String::new()
                } else {
                    json!({"jsonrpc":"2.0","id":request["id"],"result":result}).to_string()
                };
                let response = format!(
                    "HTTP/1.1 {status} Synthetic\r\nContent-Type: application/json\r\nContent-Length: {}\r\n{extra}Connection: close\r\n\r\n{body}",
                    body.len()
                );
                stream
                    .write_all(response.as_bytes())
                    .expect("controlled MCP response");
            }
        });
        Self {
            endpoint,
            control,
            worker: Some(worker),
        }
    }
}
impl Drop for McpPeer {
    fn drop(&mut self) {
        self.control.stop.store(true, Ordering::SeqCst);
        if let Some(worker) = self.worker.take()
            && let Err(error) = worker.join()
            && !std::thread::panicking()
        {
            std::panic::resume_unwind(error);
        }
    }
}

#[tokio::test]
async fn mcp_loaded_package_executes_after_current_grant_with_durable_intent_and_receipt() {
    let temp = TempRoot::new();
    let owned = owner("mcp");
    let other = owner("other");
    let package = write_mcp_package(&temp.0.join("package"));
    let peer = McpPeer::start(temp.state());
    let runtime =
        PluginRuntime::with_packages(temp.state(), vec![package.clone()]).expect("runtime");
    assert!(install(&runtime, &owned).await.accepted);
    let installed = configure(&runtime, &owned, json!({"endpoint":peer.endpoint})).await;
    let discovered = probe(&runtime, &owned, &installed)
        .await
        .expect("real MCP discovery");
    assert_eq!(discovered.kind, "mcp");
    assert_eq!(discovered.tools[0].name, "campus_read");
    assert_eq!(
        peer.control.calls.load(Ordering::SeqCst),
        0,
        "connection probe executes no business tool"
    );
    grant_enable(&runtime, &owned, &installed, &discovered).await;
    let name = projected_name(&runtime, &owned).await;
    assert_eq!(
        runtime
            .execute(&other.0, &other.1, &name, json!({"query":"synthetic"}))
            .await
            .status(),
        ChatToolStatus::Denied
    );
    assert_eq!(
        runtime
            .execute(&owned.0, &owned.1, &name, json!({"query":9}))
            .await
            .status(),
        ChatToolStatus::Denied
    );
    assert_eq!(
        peer.control.calls.load(Ordering::SeqCst),
        0,
        "M20 rejects invalid args before intent/I/O"
    );
    assert_eq!(
        runtime
            .execute(&owned.0, &owned.1, &name, json!({"query":"synthetic"}))
            .await
            .status(),
        ChatToolStatus::Succeeded
    );
    assert!(peer.control.intent_before_call.load(Ordering::SeqCst));
    assert_journal(&runtime, 1).await;
    drop(runtime);
    let runtime = PluginRuntime::with_packages(temp.state(), vec![package]).expect("restart");
    assert_eq!(projected_name(&runtime, &owned).await, name);
    assert_eq!(
        runtime
            .execute(&owned.0, &owned.1, &name, json!({"query":"after restart"}))
            .await
            .status(),
        ChatToolStatus::Succeeded
    );
    assert_eq!(peer.control.calls.load(Ordering::SeqCst), 2);
    assert_journal(&runtime, 2).await;
    assert!(peer.control.intent_before_call.load(Ordering::SeqCst));
}

#[tokio::test]
async fn mcp_session_failure_never_retries_and_explicit_rediscovery_restores_same_review() {
    let temp = TempRoot::new();
    let owned = owner("mcp");
    let package = write_mcp_package(&temp.0.join("package"));
    let peer = McpPeer::start(temp.state());
    let runtime = PluginRuntime::with_packages(temp.state(), vec![package]).expect("runtime");
    install(&runtime, &owned).await;
    let installed = configure(&runtime, &owned, json!({"endpoint":peer.endpoint})).await;
    let discovered = probe(&runtime, &owned, &installed).await.expect("probe");
    let enabled = grant_enable(&runtime, &owned, &installed, &discovered).await;
    let name = projected_name(&runtime, &owned).await;
    peer.control.expire_next.store(true, Ordering::SeqCst);
    let expired = runtime
        .execute(&owned.0, &owned.1, &name, json!({"query":"expires"}))
        .await;
    assert_eq!(expired.status(), ChatToolStatus::Denied);
    let result: Value = serde_json::from_str(
        &expired
            .serialize_for_provider()
            .expect("safe expiry result"),
    )
    .expect("JSON");
    assert_eq!(result["data"]["code"], "plugin_review_required");
    assert_eq!(peer.control.calls.load(Ordering::SeqCst), 1);
    assert_journal(&runtime, 1).await;
    assert!(
        runtime
            .definitions(&owned.0, &owned.1)
            .await
            .expect("blocked")
            .is_empty()
    );
    assert_eq!(
        runtime
            .execute(&owned.0, &owned.1, &name, json!({"query":"must not retry"}))
            .await
            .status(),
        ChatToolStatus::Denied
    );
    assert_eq!(peer.control.calls.load(Ordering::SeqCst), 1);
    let reconnected = probe(&runtime, &owned, &enabled)
        .await
        .expect("explicit rediscovery");
    assert_eq!(reconnected.readiness_digest, discovered.readiness_digest);
    assert_eq!(projected_name(&runtime, &owned).await, name);
    assert_eq!(
        runtime
            .execute(
                &owned.0,
                &owned.1,
                &name,
                json!({"query":"new explicit call"})
            )
            .await
            .status(),
        ChatToolStatus::Succeeded
    );
    assert_eq!(peer.control.calls.load(Ordering::SeqCst), 2);
    assert_journal(&runtime, 2).await;
}

#[tokio::test]
async fn mcp_inventory_drift_after_restart_cannot_inherit_prior_enable_review() {
    let temp = TempRoot::new();
    let owned = owner("mcp");
    let package = write_mcp_package(&temp.0.join("package"));
    let peer = McpPeer::start(temp.state());
    let runtime =
        PluginRuntime::with_packages(temp.state(), vec![package.clone()]).expect("runtime");
    install(&runtime, &owned).await;
    let installed = configure(&runtime, &owned, json!({"endpoint":peer.endpoint})).await;
    let discovered = probe(&runtime, &owned, &installed).await.expect("probe");
    let enabled = grant_enable(&runtime, &owned, &installed, &discovered).await;
    let name = projected_name(&runtime, &owned).await;
    drop(runtime);
    peer.control.drift.store(true, Ordering::SeqCst);
    let runtime = PluginRuntime::with_packages(temp.state(), vec![package]).expect("restart");
    assert!(
        runtime
            .definitions(&owned.0, &owned.1)
            .await
            .expect("drift blocked")
            .is_empty()
    );
    assert_eq!(
        runtime
            .execute(&owned.0, &owned.1, &name, json!({"query":"old review"}))
            .await
            .status(),
        ChatToolStatus::Denied
    );
    let changed = probe(&runtime, &owned, &enabled)
        .await
        .expect("changed inventory surfaced for new review");
    assert_ne!(changed.readiness_digest, discovered.readiness_digest);
    assert!(
        runtime
            .definitions(&owned.0, &owned.1)
            .await
            .expect("still blocked")
            .is_empty()
    );
    assert_eq!(peer.control.calls.load(Ordering::SeqCst), 0);
    assert_journal(&runtime, 0).await;
}

#[tokio::test]
async fn current_m20_grant_revocation_blocks_a_previously_projected_tool() {
    let temp = TempRoot::new();
    let owned = owner("revocation");
    let runtime = skill_runtime(temp.state());
    install(&runtime, &owned).await;
    let installed = snapshot(&runtime, &owned).await;
    let discovered = probe(&runtime, &owned, &installed).await.expect("probe");
    let enabled = grant_enable(&runtime, &owned, &installed, &discovered).await;
    let name = projected_name(&runtime, &owned).await;
    let frozen = runtime
        .session(&owned.0, &owned.1)
        .await
        .expect("projection before revoke");
    {
        let mut state = runtime.state.lock().await;
        let id = InstallationId::parse(enabled.id).expect("id");
        let capability = CapabilityId::parse("campus.public_rules.read").expect("capability");
        let scope = GrantScope::campus_public().expect("scope");
        let grant = state
            .authority
            .grants
            .load_current_for_authority(&owned.0, &owned.1, &id, &capability, &scope)
            .expect("read grant")
            .expect("grant");
        let command = GrantCommand::revoke(
            GrantCommandId::parse("grant-cmd:explicit-test-revoke").expect("command id"),
            grant.snapshot_id().clone(),
            grant.version().clone(),
        )
        .expect("M20 revoke command");
        let mut next = state.authority.clone();
        let receipt = next.grants.execute(command).expect("M20 revoke");
        assert!(matches!(
            receipt.outcome(),
            GrantCommandOutcome::Accepted { .. }
        ));
        state.commit(next).expect("durable revocation");
    }
    assert!(
        runtime
            .definitions(&owned.0, &owned.1)
            .await
            .expect("revoked definitions")
            .is_empty()
    );
    assert_eq!(
        runtime
            .execute_frozen(
                &frozen,
                &name,
                json!({"resource":"skills/campus-guide/SKILL.md"})
            )
            .await
            .status(),
        ChatToolStatus::Denied
    );
    assert_journal(&runtime, 0).await;
}

#[tokio::test]
async fn request_id_cannot_cross_installation_and_grant_receipt_namespaces() {
    let temp = TempRoot::new();
    let owned = owner("a");
    let runtime = skill_runtime(temp.state());
    install(&runtime, &owned).await;
    let installed = snapshot(&runtime, &owned).await;
    assert_eq!(command(&runtime,&owned,"install",json!({"action":"grant","installation_id":installed.id,"expected_revision":installed.revision,"capability":"campus.public_rules.read"})).await.err(),Some(PluginError::Conflict));
    assert!(command(&runtime,&owned,"grant-only",json!({"action":"grant","installation_id":installed.id,"expected_revision":installed.revision,"capability":"campus.public_rules.read"})).await.expect("grant").accepted);
    assert_eq!(command(&runtime,&owned,"grant-only",json!({"action":"disable","installation_id":installed.id,"expected_revision":installed.revision})).await.err(),Some(PluginError::Conflict));
}

async fn activate_index(
    runtime: &PluginRuntime,
    owned: &(TenantId, UserId),
    index: usize,
    values: Value,
) -> String {
    let package = &runtime.packages[index];
    let pin = package.configuration.package_pin();
    let installed=command(runtime,owned,&format!("install-{index}"),json!({"action":"install","package_id":pin.package_id().as_str(),"version":pin.package_version().as_str(),"catalog_revision":pin.catalog_revision().as_str(),"package_digest":pin.package_digest().as_str()})).await.expect("install");
    assert!(installed.accepted);
    let configured=command(runtime,owned,&format!("configure-{index}"),json!({"action":"configure","installation_id":installed.installation_id,"expected_revision":installed.revision,"values":values})).await.expect("configure");
    assert!(configured.accepted);
    let revision = configured.revision.expect("revision");
    let id = configured.installation_id;
    let discovered = runtime
        .probe(
            &owned.0,
            &owned.1,
            PluginProbeDto {
                schema: "plugin-probe/v1".into(),
                installation_id: id.clone(),
                expected_revision: revision.clone(),
            },
        )
        .await
        .expect("probe");
    assert!(command(runtime,owned,&format!("grant-{index}"),json!({"action":"grant","installation_id":id,"expected_revision":revision,"capability":"campus.public_rules.read"})).await.expect("grant").accepted);
    assert!(command(runtime,owned,&format!("enable-{index}"),json!({"action":"enable","installation_id":id,"expected_revision":revision,"readiness_digest":discovered.readiness_digest})).await.expect("enable").accepted);
    let wire = match &package.component {
        RuntimeComponent::Skill { .. } => "skill_read",
        RuntimeComponent::Mcp { .. } => "campus_read",
    };
    tool_name(&InstallationId::parse(id).expect("id"), wire)
}

fn read_provider_request(stream: &mut std::net::TcpStream) -> Value {
    use std::io::Read;
    stream
        .set_read_timeout(Some(std::time::Duration::from_secs(5)))
        .expect("provider fixture timeout");
    let mut bytes = Vec::new();
    let mut buffer = [0u8; 4096];
    let end = loop {
        let count = stream.read(&mut buffer).expect("provider request");
        assert!(count > 0);
        bytes.extend_from_slice(&buffer[..count]);
        if let Some(index) = bytes.windows(4).position(|w| w == b"\r\n\r\n") {
            break index + 4;
        }
    };
    let headers = String::from_utf8_lossy(&bytes[..end]).to_ascii_lowercase();
    let length = headers
        .lines()
        .find_map(|line| line.strip_prefix("content-length:"))
        .expect("length")
        .trim()
        .parse::<usize>()
        .expect("numeric length");
    while bytes.len() < end + length {
        let count = stream.read(&mut buffer).expect("body");
        assert!(count > 0);
        bytes.extend_from_slice(&buffer[..count]);
    }
    serde_json::from_slice(&bytes[end..end + length]).expect("provider JSON")
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn actual_bounded_agent_executes_skill_and_mcp_through_current_m20_and_m30() {
    use crate::{
        agent_chat::{CHAT_REQUEST_SCHEMA, run_bounded_chat},
        chat_provider::ChatProvider,
        chat_tools::{ChatToolExecution, ChatToolExecutor, ChatToolRequest},
    };
    use std::{io::Write, net::TcpListener};
    let temp = TempRoot::new();
    let owned = owner("agent");
    let mcp = write_mcp_package(&temp.0.join("package"));
    let peer = McpPeer::start(temp.state());
    let runtime = PluginRuntime::with_packages(
        temp.state(),
        vec![RuntimePackage::bundled_skill().expect("skill"), mcp],
    )
    .expect("runtime");
    let skill_name = activate_index(&runtime, &owned, 0, json!({})).await;
    let mcp_name = activate_index(&runtime, &owned, 1, json!({"endpoint":peer.endpoint})).await;
    struct Executor {
        runtime: PluginRuntime,
        session: PluginToolSession,
    }
    impl ChatToolExecutor for Executor {
        fn definitions(&self) -> Vec<ChatDynamicToolDefinition> {
            self.session.definitions()
        }
        async fn execute(&mut self, request: ChatToolRequest) -> ChatToolExecution {
            match request {
                ChatToolRequest::Plugin {
                    tool_name,
                    arguments,
                } => {
                    self.runtime
                        .execute_frozen(&self.session, &tool_name, arguments)
                        .await
                }
                _ => panic!("fixture only calls installed tools"),
            }
        }
    }
    let session = runtime
        .session(&owned.0, &owned.1)
        .await
        .expect("frozen definitions");
    assert_eq!(session.definitions().len(), 2);
    let listener = TcpListener::bind("127.0.0.1:0").expect("controlled model endpoint");
    let address = listener.local_addr().expect("provider address");
    let provider_thread = std::thread::spawn(move || {
        let (mut first, _) = listener.accept().expect("first provider turn");
        let request = read_provider_request(&mut first);
        assert_eq!(
            request["tools"].as_array().expect("complete toolset").len(),
            5
        );
        assert!(
            !request.to_string().contains("http://127.0.0.1"),
            "endpoint routing remains gateway private"
        );
        let result = json!({"choices":[{"finish_reason":"tool_calls","message":{"role":"assistant","content":null,"tool_calls":[
            {"id":"synthetic-skill","type":"function","function":{"name":skill_name,"arguments":json!({"resource":"skills/campus-guide/SKILL.md"}).to_string()}},
            {"id":"synthetic-mcp","type":"function","function":{"name":mcp_name,"arguments":json!({"query":"synthetic"}).to_string()}}
        ]}}]});
        let body = result.to_string();
        write!(first,"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",body.len()).expect("tool proposal response");
        drop(first);
        let (mut second, _) = listener.accept().expect("second provider turn");
        let request = read_provider_request(&mut second);
        let results = request["messages"]
            .as_array()
            .expect("messages")
            .iter()
            .filter(|message| message["role"] == "tool")
            .collect::<Vec<_>>();
        assert_eq!(results.len(), 2);
        for result in &results {
            let content: Value =
                serde_json::from_str(result["content"].as_str().expect("tool content"))
                    .expect("tool JSON");
            assert_eq!(content["trust"], "untrusted_data");
            assert_eq!(content["status"], "succeeded");
        }
        let body=json!({"choices":[{"finish_reason":"stop","message":{"role":"assistant","content":"Both synthetic installed tools completed."}}]}).to_string();
        write!(second,"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",body.len()).expect("final response");
    });
    let key = temp.0.join("synthetic-provider-key");
    fs::write(&key, b"synthetic-provider-token").expect("synthetic credential");
    fs::set_permissions(&key, fs::Permissions::from_mode(0o600)).expect("private key fixture");
    let provider = ChatProvider::openai_compatible_for_test(
        &format!("http://{address}/v1"),
        "synthetic-model",
        &key,
        5000,
    )
    .expect("provider");
    let mut executor = Executor {
        runtime: runtime.clone(),
        session,
    };
    let request=serde_json::from_value(json!({"schema":CHAT_REQUEST_SCHEMA,"messages":[{"role":"user","content":"Use both installed synthetic tools."}]})).expect("chat request");
    let response = run_bounded_chat(
        "chat-run:installed-plugin-integration".into(),
        request,
        false,
        &provider,
        &mut executor,
    )
    .await
    .expect("actual bounded Agent run");
    assert_eq!(response.answer, "Both synthetic installed tools completed.");
    assert_eq!(response.tool_trace.len(), 2);
    assert!(
        response
            .tool_trace
            .iter()
            .all(|trace| trace.tool == "plugin_tool" && trace.status == ChatToolStatus::Succeeded)
    );
    provider_thread.join().expect("provider fixture completed");
    assert_journal(&runtime, 2).await;
    assert_eq!(peer.control.calls.load(Ordering::SeqCst), 1);
    assert!(peer.control.intent_before_call.load(Ordering::SeqCst));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn provider_inflight_projection_cannot_follow_reconfigured_endpoint_schema_or_grant() {
    use crate::{
        agent_chat::{CHAT_REQUEST_SCHEMA, run_bounded_chat},
        chat_provider::ChatProvider,
        chat_tools::{ChatToolExecution, ChatToolExecutor, ChatToolRequest},
    };
    use std::{io::Write, net::TcpListener, time::Duration};
    let temp = TempRoot::new();
    let owned = owner("frozen");
    let old_peer = McpPeer::start(temp.state());
    let new_peer = McpPeer::start(temp.state());
    new_peer.control.drift.store(true, Ordering::SeqCst);
    let runtime = PluginRuntime::with_packages(
        temp.state(),
        vec![write_mcp_package(&temp.0.join("package"))],
    )
    .expect("runtime");
    let name = activate_index(&runtime, &owned, 0, json!({"endpoint":old_peer.endpoint})).await;
    let session = runtime
        .session(&owned.0, &owned.1)
        .await
        .expect("original frozen projection");
    let original_binding = session.bindings.get(&name).expect("binding").clone();
    struct Executor {
        runtime: PluginRuntime,
        session: PluginToolSession,
    }
    impl ChatToolExecutor for Executor {
        fn definitions(&self) -> Vec<ChatDynamicToolDefinition> {
            self.session.definitions()
        }
        async fn execute(&mut self, request: ChatToolRequest) -> ChatToolExecution {
            let ChatToolRequest::Plugin {
                tool_name,
                arguments,
            } = request
            else {
                panic!("plugin expected")
            };
            self.runtime
                .execute_frozen(&self.session, &tool_name, arguments)
                .await
        }
    }
    let listener = TcpListener::bind("127.0.0.1:0").expect("controlled provider");
    let address = listener.local_addr().expect("provider address");
    let (projected_tx, projected_rx) = tokio::sync::oneshot::channel();
    let (changed_tx, changed_rx) = std::sync::mpsc::channel();
    let proposed_name = name.clone();
    let provider_thread = std::thread::spawn(move || {
        let (mut first, _) = listener.accept().expect("projection request");
        let request = read_provider_request(&mut first);
        let definition = request["tools"]
            .as_array()
            .expect("tools")
            .iter()
            .find(|tool| tool["function"]["name"] == proposed_name)
            .expect("projected tool");
        assert!(
            definition["function"]["parameters"]["properties"]
                .get("mode")
                .is_none()
        );
        projected_tx.send(()).expect("projection signal");
        changed_rx
            .recv_timeout(Duration::from_secs(10))
            .expect("configuration changed while provider waited");
        let body = json!({"choices":[{"finish_reason":"tool_calls","message":{"role":"assistant","content":null,"tool_calls":[{"id":"synthetic-stale-proposal","type":"function","function":{"name":proposed_name,"arguments":json!({"query":"stale-projection"}).to_string()}}]}}]}).to_string();
        write!(first,"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",body.len()).expect("stale proposal");
        drop(first);
        let (mut second, _) = listener.accept().expect("result request");
        let request = read_provider_request(&mut second);
        let result = request["messages"]
            .as_array()
            .expect("messages")
            .iter()
            .find(|message| message["role"] == "tool")
            .expect("denied result");
        let content: Value = serde_json::from_str(result["content"].as_str().expect("content"))
            .expect("result JSON");
        assert_eq!(content["status"], "denied");
        let body = json!({"choices":[{"finish_reason":"stop","message":{"role":"assistant","content":"The old proposal was denied."}}]}).to_string();
        write!(second,"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",body.len()).expect("final response");
    });
    let key = temp.0.join("synthetic-provider-key");
    fs::write(&key, b"synthetic-provider-token").expect("fixture credential");
    fs::set_permissions(&key, fs::Permissions::from_mode(0o600)).expect("fixture mode");
    let provider = ChatProvider::openai_compatible_for_test(
        &format!("http://{address}/v1"),
        "synthetic-model",
        &key,
        15000,
    )
    .expect("provider");
    let mut executor = Executor {
        runtime: runtime.clone(),
        session,
    };
    let request = serde_json::from_value(json!({"schema":CHAT_REQUEST_SCHEMA,"messages":[{"role":"user","content":"Read the synthetic campus information."}]})).expect("request");
    let run = run_bounded_chat(
        "chat-run:frozen-plugin-projection".into(),
        request,
        false,
        &provider,
        &mut executor,
    );
    let mutate = async {
        projected_rx
            .await
            .expect("model has the original definition");
        let old = snapshot(&runtime, &owned).await;
        assert!(command(&runtime,&owned,"freeze-disable",json!({"action":"disable","installation_id":old.id,"expected_revision":old.revision})).await.expect("disable").accepted);
        let disabled = snapshot(&runtime, &owned).await;
        assert!(command(&runtime,&owned,"freeze-configure",json!({"action":"configure","installation_id":disabled.id,"expected_revision":disabled.revision,"values":{"endpoint":new_peer.endpoint}})).await.expect("configure new endpoint").accepted);
        let configured = snapshot(&runtime, &owned).await;
        let changed = probe(&runtime, &owned, &configured)
            .await
            .expect("new endpoint and schema probe");
        assert!(command(&runtime,&owned,"freeze-grant",json!({"action":"grant","installation_id":configured.id,"expected_revision":configured.revision,"capability":"campus.public_rules.read"})).await.expect("new grant").accepted);
        assert!(command(&runtime,&owned,"freeze-enable",json!({"action":"enable","installation_id":configured.id,"expected_revision":configured.revision,"readiness_digest":changed.readiness_digest})).await.expect("new review enable").accepted);
        let fresh = runtime
            .session(&owned.0, &owned.1)
            .await
            .expect("new projection");
        let changed_binding = fresh.bindings.get(&name).expect("same visible name");
        assert_ne!(
            original_binding.installation_revision,
            changed_binding.installation_revision
        );
        assert_ne!(
            original_binding.grant_snapshot_id,
            changed_binding.grant_snapshot_id
        );
        assert_ne!(
            original_binding.readiness_digest,
            changed_binding.readiness_digest
        );
        assert_ne!(
            original_binding.tool.claimed_input_schema_digest,
            changed_binding.tool.claimed_input_schema_digest
        );
        let counts = (
            old_peer.control.observed.lock().expect("requests").len(),
            new_peer.control.observed.lock().expect("requests").len(),
        );
        changed_tx.send(()).expect("release provider");
        (fresh, counts)
    };
    let (response, (fresh, counts)) = tokio::join!(run, mutate);
    let response = response.expect("bounded chat returns denied tool result");
    assert_eq!(response.tool_trace.len(), 1);
    assert_eq!(response.tool_trace[0].status, ChatToolStatus::Denied);
    assert_eq!(response.tool_trace[0].tool, "plugin_tool");
    provider_thread.join().expect("provider fixture");
    assert_eq!(
        old_peer.control.observed.lock().expect("requests").len(),
        counts.0
    );
    assert_eq!(
        new_peer.control.observed.lock().expect("requests").len(),
        counts.1
    );
    assert_eq!(old_peer.control.calls.load(Ordering::SeqCst), 0);
    assert_eq!(new_peer.control.calls.load(Ordering::SeqCst), 0);
    assert_journal(&runtime, 0).await;
    assert_eq!(
        runtime
            .execute_frozen(
                &fresh,
                &name,
                json!({"query":"new-projection","mode":"current"})
            )
            .await
            .status(),
        ChatToolStatus::Succeeded
    );
    assert_eq!(new_peer.control.calls.load(Ordering::SeqCst), 1);
    assert_journal(&runtime, 1).await;
}

#[tokio::test]
async fn frozen_session_rejects_grant_replacement_and_foreign_runtime_without_effects() {
    let temp = TempRoot::new();
    let other_temp = TempRoot::new();
    let owned = owner("grant-freeze");
    let runtime = skill_runtime(temp.state());
    let other_runtime = skill_runtime(other_temp.state());
    let name = activate_index(&runtime, &owned, 0, json!({})).await;
    let session = runtime
        .session(&owned.0, &owned.1)
        .await
        .expect("original projection");
    let foreign = owner("foreign");
    let foreign_session = runtime
        .session(&foreign.0, &foreign.1)
        .await
        .expect("foreign empty projection");
    let args = json!({"resource":"skills/campus-guide/SKILL.md"});
    assert_eq!(
        runtime
            .execute_frozen(&foreign_session, &name, args.clone())
            .await
            .status(),
        ChatToolStatus::Denied
    );
    assert_eq!(
        other_runtime
            .execute_frozen(&session, &name, args.clone())
            .await
            .status(),
        ChatToolStatus::Denied
    );
    assert_journal(&other_runtime, 0).await;
    let installed = snapshot(&runtime, &owned).await;
    assert!(command(&runtime,&owned,"fresh-grant-same-installation",json!({"action":"grant","installation_id":installed.id,"expected_revision":installed.revision,"capability":"campus.public_rules.read"})).await.expect("replace grant").accepted);
    let fresh = runtime
        .session(&owned.0, &owned.1)
        .await
        .expect("fresh projection");
    let previous = session.bindings.get(&name).expect("original binding");
    let current = fresh.bindings.get(&name).expect("current binding");
    assert_eq!(
        previous.installation_revision,
        current.installation_revision
    );
    assert_eq!(previous.readiness_digest, current.readiness_digest);
    assert_eq!(previous.tool, current.tool);
    assert_ne!(previous.grant_snapshot_id, current.grant_snapshot_id);
    assert_eq!(
        runtime
            .execute_frozen(&session, &name, args.clone())
            .await
            .status(),
        ChatToolStatus::Denied
    );
    assert_journal(&runtime, 0).await;
    assert_eq!(
        runtime.execute_frozen(&fresh, &name, args).await.status(),
        ChatToolStatus::Succeeded
    );
    assert_journal(&runtime, 1).await;
}

#[path = "tests/retirement.rs"]
mod retirement;

mod skill_recovery;

#[path = "tests/mcp_recovery.rs"]
mod mcp_recovery;

mod mixed_import;

mod update_lifecycle;
