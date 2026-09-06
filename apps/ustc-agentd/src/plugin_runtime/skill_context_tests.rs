//! Skill declaration limits must remain usable through the admitted Agent tool path.
use super::*;
use crate::chat_tools::ChatToolStatus;
use serde_json::{Value, json};
use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::Path,
    sync::atomic::{AtomicU64, Ordering},
};
use ustc_campus_agent_core::market::{
    configuration_schema::ConfigurationSchema, load_package_manifest,
};

static NEXT: AtomicU64 = AtomicU64::new(0);

#[tokio::test]
async fn skill_context_short_entry_path_error_is_distinct_from_artifact_drift() {
    let fixture = Fixture::new();
    let runtime = PluginRuntime::with_packages(
        fixture.0.join("state/authority.bin"),
        vec![RuntimePackage::bundled_skill().expect("bundled Skill")],
    )
    .expect("runtime");
    let session = enable(&runtime).await;
    let name = session.bindings.keys().next().expect("Skill tool");
    let output = runtime
        .execute_frozen(&session, name, json!({"resource":"SKILL.md"}))
        .await;
    assert_eq!(output.status(), ChatToolStatus::Failed);
    let result: Value =
        serde_json::from_str(&output.serialize_for_provider().expect("provider result"))
            .expect("JSON");
    assert_eq!(result["data"]["code"], "plugin_invalid_arguments");
    let journal = serde_json::to_value(&runtime.state.lock().await.authority.runs)
        .expect("journal")
        .to_string();
    assert!(
        journal.contains("effect_intent_persisted") && journal.contains("plugin_execution_failed")
    );
    let RuntimeComponent::Skill { source } = &runtime.packages[0].component else {
        panic!("Skill source")
    };
    for args in [
        json!({"resource":"SKILL.md"}),
        json!({"resource":""}),
        json!({"resource":null}),
        json!({"offset":-1}),
        json!({"offset":1.5}),
        json!({"offset":null}),
        json!({"offset":65537}),
        json!({"extra":true}),
    ] {
        assert_eq!(
            skill_context::read(source, &args).err(),
            Some(PluginError::InvalidRequest),
            "only omission selects the exact entry; invalid arguments stay invalid"
        );
    }
    let output = runtime.execute_frozen(&session, name, json!({})).await;
    assert_eq!(output.status(), ChatToolStatus::Succeeded);
    let result: Value =
        serde_json::from_str(&output.serialize_for_provider().expect("entry result"))
            .expect("entry JSON");
    assert_eq!(result["data"]["resource"], source.skill_path());
    assert_eq!(
        result["data"]["text"],
        source.read(source.skill_path()).expect("verified entry")
    );
    assert_eq!(result["data"]["offset"], 0);
    assert!(result["data"]["next_offset"].is_null());
}

struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "uca-skill-context-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&root).expect("valid Skill fixture");
        fs::set_permissions(&root, fs::Permissions::from_mode(0o700)).expect("valid Skill fixture");
        Self(root)
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        if self.0.parent() == Some(std::env::temp_dir().as_path())
            && self
                .0
                .file_name()
                .expect("valid Skill fixture")
                .to_string_lossy()
                .starts_with("uca-skill-context-")
        {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
}

fn package(root: &Path, resources: &[(String, String)]) -> RuntimePackage {
    let skill_path = "skills/campus-guide/SKILL.md";
    let skill = format!(
        "---\nname: campus-guide\ndescription: {}\n---\nRead the declared references.\n",
        "😀".repeat(1024)
    );
    let manifest_bytes =
        include_bytes!("../../../../market/packages/ustc.campus-guide/package.json");
    let manifest = load_package_manifest(manifest_bytes).expect("valid Skill fixture");
    let schema = ConfigurationSchema::new(vec![]).expect("valid Skill fixture");
    let digest = Sha256Digest::from_bytes(skill.as_bytes());
    let config = json!({"schemaVersion":"package-component-configuration/v1","packageId":manifest.package_id().as_str(),"packageVersion":manifest.package_version().as_str(),"packageDigest":manifest.package_digest().as_str(),"componentSetDigest":manifest.component_declaration_set_digest().as_str(),"capabilityManifestDigest":manifest.capability_manifest_digest().as_str(),"components":[{"path":skill_path,"type":"SkillComponent","mode":null,"componentId":"component:skill-context","componentVersion":"1","componentDigest":digest.as_str(),"executionIdentity":"execution:skill-context","schemaDigest":schema.digest().as_str(),"fields":[]}]});
    let mut declarations = vec![json!({"path":skill_path,"sha256":digest.as_str()})];
    for (path, text) in std::iter::once(&(skill_path.to_owned(), skill)).chain(resources.iter()) {
        let target = root.join(path);
        fs::create_dir_all(target.parent().expect("valid Skill fixture"))
            .expect("valid Skill fixture");
        fs::write(target, text).expect("valid Skill fixture");
    }
    declarations.extend(resources.iter().map(|(path, text)| json!({"path":path,"sha256":Sha256Digest::from_bytes(text.as_bytes()).as_str()})));
    fs::write(root.join("package.json"), manifest_bytes).expect("valid Skill fixture");
    fs::write(
        root.join("configuration.json"),
        serde_json::to_vec(&config).expect("valid Skill fixture"),
    )
    .expect("valid Skill fixture");
    fs::write(root.join("runtime.json"), serde_json::to_vec(&json!({"schemaVersion":"plugin-runtime/v1","kind":"skill","skillPath":skill_path,"resources":declarations})).expect("valid Skill fixture")).expect("valid Skill fixture");
    RuntimePackage::load(root).expect("standard long-description Skill and declared resources load")
}

fn owner() -> (TenantId, UserId) {
    (
        TenantId::parse("tenant:skill-context").expect("valid Skill fixture"),
        UserId::parse("user:skill-context").expect("valid Skill fixture"),
    )
}
async fn command(runtime: &PluginRuntime, request: &str, intent: Value) -> PluginCommandResultDto {
    let (tenant, user) = owner();
    runtime
        .command(
            &tenant,
            &user,
            serde_json::from_value(
                json!({"schema":"plugin-command/v1","request_id":request,"intent":intent}),
            )
            .expect("valid Skill fixture"),
        )
        .await
        .expect("valid Skill fixture")
}
async fn enable(runtime: &PluginRuntime) -> PluginToolSession {
    let (tenant, user) = owner();
    let pin = runtime.packages[0].configuration.package_pin();
    let installed = command(runtime, "install", json!({"action":"install","package_id":pin.package_id().as_str(),"version":pin.package_version().as_str(),"catalog_revision":pin.catalog_revision().as_str(),"package_digest":pin.package_digest().as_str()})).await;
    assert!(installed.accepted);
    let probe = runtime
        .probe(
            &tenant,
            &user,
            PluginProbeDto {
                schema: "plugin-probe/v1".into(),
                installation_id: installed.installation_id.clone(),
                expected_revision: installed.revision.clone().expect("valid Skill fixture"),
            },
        )
        .await
        .expect("all legal resource declarations can be projected");
    assert!(command(runtime, "grant", json!({"action":"grant","installation_id":installed.installation_id,"expected_revision":installed.revision,"capability":"campus.public_rules.read"})).await.accepted);
    assert!(command(runtime, "enable", json!({"action":"enable","installation_id":installed.installation_id,"expected_revision":installed.revision,"readiness_digest":probe.readiness_digest})).await.accepted);
    let session = runtime
        .session(&tenant, &user)
        .await
        .expect("long metadata does not discard plugin session");
    assert_eq!(session.definitions().len(), 1);
    assert!(
        session
            .bindings
            .values()
            .next()
            .expect("valid Skill fixture")
            .tool
            .description
            .len()
            <= 4096
    );
    session
}

#[tokio::test]
async fn skill_context_large_inventory_long_path_and_metadata_reach_real_tool() {
    let fixture = Fixture::new();
    let mut resources: Vec<_> = (0..64)
        .map(|index| (format!("refs/{index}.md"), format!("reference {index}")))
        .collect();
    let long_path = format!(
        "refs/{}/{}/{}.md",
        "a".repeat(100),
        "b".repeat(100),
        "c".repeat(47)
    );
    assert_eq!(long_path.len(), 257);
    resources.push((long_path.clone(), "long path resource".into()));
    let package = package(&fixture.0.join("package"), &resources);
    let RuntimeComponent::Skill { source } = &package.component else {
        panic!("skill")
    };
    assert_eq!(
        source.metadata().description.chars().count(),
        1024,
        "original metadata is retained"
    );
    let runtime =
        PluginRuntime::with_packages(fixture.0.join("state/authority.bin"), vec![package])
            .expect("valid Skill fixture");
    let session = enable(&runtime).await;
    let name = session.bindings.keys().next().expect("valid Skill fixture");
    for resource in ["refs/63.md", long_path.as_str()] {
        let result = runtime
            .execute_frozen(&session, name, json!({"resource":resource}))
            .await;
        assert_eq!(result.status(), ChatToolStatus::Succeeded);
        let result: Value = serde_json::from_str(
            &result
                .serialize_for_provider()
                .expect("valid Skill fixture"),
        )
        .expect("valid Skill fixture");
        assert_eq!(result["data"]["resource"], resource);
        assert_eq!(result["data"]["offset"], 0);
        assert!(result["data"]["next_offset"].is_null());
    }
    for args in [
        json!({"resource":"refs/0.md","extra":true}),
        json!({"resource":"refs/0.md","offset":-1}),
        json!({"resource":"refs/0.md","offset":0.5}),
        json!({"resource":"refs/0.md","offset":999}),
        json!({"resource":"not-declared.md"}),
    ] {
        assert_ne!(
            runtime.execute_frozen(&session, name, args).await.status(),
            ChatToolStatus::Succeeded
        );
    }
}

#[tokio::test]
async fn skill_context_full_size_and_escaped_resources_reassemble_with_bounded_results() {
    let fixture = Fixture::new();
    let resources = vec![
        ("ascii.md".into(), "a".repeat(65536)),
        ("quotes.md".into(), "\"".repeat(65536)),
        ("escaped.md".into(), "\0".repeat(65536)),
        ("unicode.md".into(), "😀".repeat(16384)),
    ];
    let package = package(&fixture.0.join("package"), &resources);
    let runtime =
        PluginRuntime::with_packages(fixture.0.join("state/authority.bin"), vec![package])
            .expect("valid Skill fixture");
    let session = enable(&runtime).await;
    let name = session.bindings.keys().next().expect("valid Skill fixture");
    for (resource, expected) in &resources {
        let mut assembled = String::new();
        let mut offset = 0;
        loop {
            let output = runtime
                .execute_frozen(&session, name, json!({"resource":resource,"offset":offset}))
                .await;
            assert_eq!(
                output.status(),
                ChatToolStatus::Succeeded,
                "{resource} at {offset}"
            );
            let serialized = output
                .serialize_for_provider()
                .expect("outer envelope is bounded");
            assert!(serialized.len() <= 65536);
            let output: Value = serde_json::from_str(&serialized).expect("valid Skill fixture");
            let data = &output["data"];
            assert!(serde_json::to_vec(data).expect("valid Skill fixture").len() <= 60 * 1024);
            assert_eq!(data["offset"], offset);
            assert_eq!(data["total_bytes"], expected.len());
            let text = data["text"].as_str().expect("valid Skill fixture");
            assert!(text.len() <= 16 * 1024);
            assert!(!text.is_empty());
            assembled.push_str(text);
            if data["next_offset"].is_null() {
                break;
            }
            let next = data["next_offset"].as_u64().expect("valid Skill fixture") as usize;
            assert_eq!(next, offset + text.len());
            assert!(next > offset && expected.is_char_boundary(next));
            offset = next;
        }
        assert_eq!(&assembled, expected);
    }
    assert_ne!(
        runtime
            .execute_frozen(&session, name, json!({"resource":"unicode.md","offset":1}))
            .await
            .status(),
        ChatToolStatus::Succeeded
    );
}

#[test]
fn skill_context_terminal_offsets_digest_drift_and_unlisted_paths_remain_checked() {
    let fixture = Fixture::new();
    let root = fixture.0.join("package");
    let package = package(
        &root,
        &[
            ("empty.md".into(), String::new()),
            ("unicode.md".into(), "校园".into()),
        ],
    );
    let RuntimeComponent::Skill { source } = &package.component else {
        panic!("skill")
    };
    for args in [
        json!({"resource":"empty.md"}),
        json!({"resource":"unicode.md","offset":6}),
    ] {
        let output = skill_context::read(source, &args).expect("valid Skill fixture");
        assert_eq!(output["text"], "");
        assert!(output["next_offset"].is_null());
    }
    assert_eq!(
        skill_context::read(source, &json!({"resource":"unicode.md","offset":1})).err(),
        Some(PluginError::InvalidRequest)
    );
    fs::write(root.join("unicode.md"), "改动").expect("valid Skill fixture");
    assert_eq!(
        skill_context::read(source, &json!({"resource":"unicode.md","offset":3})).err(),
        Some(PluginError::NotReady),
        "every continuation verifies the entire file digest"
    );
    fs::rename(&root, fixture.0.join("moved-package")).expect("valid Skill fixture");
    assert_eq!(
        source.read("unlisted.md").err(),
        Some(registry::RuntimeRegistryError::InvalidDeclaration),
        "undeclared path is rejected before trying the now-missing package root"
    );
    assert_eq!(
        source.read("empty.md").err(),
        Some(registry::RuntimeRegistryError::ArtifactMismatch)
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn skill_context_bounded_chat_reads_two_pages_then_reports_partial_coverage() {
    use crate::{
        agent_chat::{CHAT_REQUEST_SCHEMA, run_bounded_chat},
        chat_provider::ChatProvider,
        chat_tools::{ChatToolExecution, ChatToolExecutor, ChatToolRequest},
    };
    use axum::{Json, Router, extract::State, routing::post};
    use std::sync::{Mutex as StdMutex, atomic::AtomicUsize};

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
                _ => panic!("controlled provider only reads the installed Skill"),
            }
        }
    }
    #[derive(Clone)]
    struct ProviderFixture {
        tool_name: String,
        calls: Arc<AtomicUsize>,
        initial_system: Arc<StdMutex<Value>>,
    }
    async fn provider(
        State(fixture): State<ProviderFixture>,
        Json(request): Json<Value>,
    ) -> Json<Value> {
        let turn = fixture.calls.fetch_add(1, Ordering::SeqCst);
        let messages = request["messages"].as_array().expect("provider messages");
        if turn == 0 {
            *fixture.initial_system.lock().expect("fixture mutex") = messages[0].clone();
            let tool = request["tools"]
                .as_array()
                .expect("tools available before budget ends")
                .iter()
                .find(|tool| tool["function"]["name"] == fixture.tool_name)
                .expect("installed Skill is offered");
            let description = tool["function"]["description"]
                .as_str()
                .expect("description");
            assert!(description.contains("partial coverage") && description.contains("budget"));
        } else {
            assert_eq!(
                messages[0],
                *fixture.initial_system.lock().expect("fixture mutex"),
                "initial authority system message stays unchanged"
            );
            let results: Vec<Value> = messages
                .iter()
                .filter(|message| message["role"] == "tool")
                .map(|message| {
                    serde_json::from_str(message["content"].as_str().expect("tool content"))
                        .expect("tool result JSON")
                })
                .collect();
            assert_eq!(results.len(), turn);
            for (index, result) in results.iter().enumerate() {
                assert_eq!(result["status"], "succeeded");
                assert_eq!(result["trust"], "untrusted_data");
                assert_eq!(result["data"]["offset"], index * 16384);
                assert_eq!(
                    result["data"]["text"].as_str().expect("page text").len(),
                    16384
                );
                assert_eq!(result["data"]["next_offset"], (index + 1) * 16384);
                assert_eq!(result["data"]["total_bytes"], 65536);
            }
        }
        assert!(turn < 3, "the immutable three-model-turn limit is retained");
        if turn == 2 {
            assert!(
                request["tools"]
                    .as_array()
                    .expect("explicit final tool list")
                    .is_empty()
            );
            assert!(
                messages
                    .iter()
                    .skip(1)
                    .any(|message| message["role"] == "system"
                        && message["content"].as_str().is_some_and(|text| text
                            .contains("partial")
                            && text.contains("next_offset"))),
                "last model turn explicitly requires partial-coverage reporting"
            );
            return Json(
                json!({"choices":[{"finish_reason":"stop","message":{"role":"assistant","content":"已读取 32KiB，属于部分材料；其余内容尚未读取，next_offset=32768。"}}]}),
            );
        }
        assert!(
            !request["tools"]
                .as_array()
                .expect("read tools available")
                .is_empty()
        );
        Json(
            json!({"choices":[{"finish_reason":"tool_calls","message":{"role":"assistant","content":null,"tool_calls":[{"id":format!("skill-page-{turn}"),"type":"function","function":{"name":fixture.tool_name,"arguments":json!({"resource":"large.md","offset":turn * 16384}).to_string()}}]}}]}),
        )
    }

    let fixture = Fixture::new();
    let package = package(
        &fixture.0.join("package"),
        &[("large.md".into(), "a".repeat(65536))],
    );
    let runtime =
        PluginRuntime::with_packages(fixture.0.join("state/authority.bin"), vec![package])
            .expect("runtime");
    let session = enable(&runtime).await;
    let calls = Arc::new(AtomicUsize::new(0));
    let state = ProviderFixture {
        tool_name: session.bindings.keys().next().expect("Skill tool").clone(),
        calls: calls.clone(),
        initial_system: Arc::new(StdMutex::new(Value::Null)),
    };
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("controlled provider listener");
    let address = listener.local_addr().expect("provider address");
    let server = tokio::spawn(async move {
        axum::serve(
            listener,
            Router::new()
                .route("/v1/chat/completions", post(provider))
                .with_state(state),
        )
        .await
        .expect("controlled provider server")
    });
    let key = fixture.0.join("provider-key");
    fs::write(&key, b"synthetic-skill-context-provider-token").expect("synthetic credential");
    fs::set_permissions(&key, fs::Permissions::from_mode(0o600))
        .expect("private synthetic credential");
    let provider = ChatProvider::openai_compatible_for_test(
        &format!("http://{address}/v1"),
        "synthetic-skill-context",
        &key,
        5000,
    )
    .expect("controlled provider");
    let request = serde_json::from_value(json!({"schema":CHAT_REQUEST_SCHEMA,"messages":[{"role":"user","content":"按当前预算阅读 large.md，并说明没有读完的部分。"}]})).expect("bounded chat request");
    let response = run_bounded_chat(
        "chat-run:skill-context-partial".into(),
        request,
        false,
        &provider,
        &mut Executor {
            runtime: runtime.clone(),
            session,
        },
    )
    .await;
    server.abort();
    let response = response.expect("bounded chat succeeds with an honest partial answer");
    assert_eq!(
        response.answer,
        "已读取 32KiB，属于部分材料；其余内容尚未读取，next_offset=32768。"
    );
    assert_eq!(calls.load(Ordering::SeqCst), 3);
    assert_eq!(response.tool_trace.len(), 2);
    assert!(
        response
            .tool_trace
            .iter()
            .all(|trace| trace.status == ChatToolStatus::Succeeded)
    );
    assert_eq!(
        runtime.state.lock().await.authority.runs.len(),
        2,
        "only two admitted resource reads occurred"
    );
}
