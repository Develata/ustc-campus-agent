//! MODEL-001 actual HTTP routes and two independent controlled provider peers.
use super::*;
use serde_json::Value;
use std::os::unix::fs::PermissionsExt;
use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::AtomicUsize,
};

struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        let root =
            std::env::temp_dir().join(format!("uca-model-routes-{}-{nonce}", std::process::id()));
        fs::create_dir(&root).expect("temp directory");
        fs::set_permissions(&root, fs::Permissions::from_mode(0o700)).expect("private directory");
        Self(root)
    }
    fn composition(&self) -> AffairsComposition {
        let workspace = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .expect("workspace");
        AffairsComposition::open(
            &workspace.join("fixtures/affairs/proc-011-reviewed.json"),
            &self.0.join("records.json"),
            &self.0.join("idempotency.json"),
            &self.0.join("sessions.json"),
        )
        .expect("composition")
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
async fn serve(router: Router) -> (String, tokio::task::JoinHandle<()>) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("listener");
    let address = listener.local_addr().expect("address");
    let task = tokio::spawn(async move {
        axum::serve(listener, router)
            .await
            .expect("controlled server");
    });
    (format!("http://{address}"), task)
}
async fn peer(
    model: &'static str,
    barrier: Arc<tokio::sync::Barrier>,
) -> (String, tokio::task::JoinHandle<()>, Arc<AtomicUsize>) {
    let calls = Arc::new(AtomicUsize::new(0));
    let observed = Arc::clone(&calls);
    let router=Router::new().route("/v1/chat/completions",post(move |headers:HeaderMap,Json(wire):Json<Value>| {
        let barrier=Arc::clone(&barrier);let observed=Arc::clone(&observed);
        async move {
            assert_eq!(headers["authorization"],"Bearer synthetic-selection-secret");
            assert_eq!(wire["model"],model);
            assert!(wire.get("tools").is_none(),"local-chat keeps all tools unavailable");
            let call=observed.fetch_add(1,Ordering::SeqCst);
            if call==0 {barrier.wait().await;}
            let content=match call {
                0 => format!("Answer from {model}"),
                1 => {assert_eq!(model,"synthetic-model-a");format!("Answer from {model}")},
                2 => {
                    assert_eq!(model,"synthetic-model-a","naming uses selected A, never default/B");
                    let messages=wire["messages"].as_array().expect("naming messages");
                    assert_eq!(messages.len(),2,"isolated naming has no prior replies");
                    assert_eq!(messages[0]["role"],"system");
                    assert!(messages[0]["content"].as_str().expect("instruction").contains("话题标题"));
                    assert_eq!(messages[1],json!({"role":"user","content":"Hello synthetic model"}));
                    "模型选择验证".to_owned()
                },
                _ => panic!("exact replay must not call chat or title providers"),
            };
            Json(json!({"choices":[{"finish_reason":"stop","message":{"role":"assistant","content":content}}],"usage":{"prompt_tokens":5,"completion_tokens":4}}))
        }
    }));
    let (base, task) = serve(router).await;
    (format!("{base}/v1"), task, calls)
}
fn chat(id: &str) -> Value {
    json!({"schema":"ustc-agent-chat-request/v3","model_id":id,"messages":[{"role":"user","content":"Hello synthetic model"}]})
}
fn turn(id: &str, request: &str, revision: u64) -> Value {
    json!({"schema":"chat-conversation-turn/v2","model_id":id,"request_id":request,"expected_revision":revision,"message":"Hello synthetic model"})
}
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn selected_models_route_independently_and_removed_model_terminal_replay_is_exact() {
    let fixture = Fixture::new();
    let key = fixture.0.join("synthetic.key");
    fs::write(&key, "synthetic-selection-secret").expect("key");
    fs::set_permissions(&key, fs::Permissions::from_mode(0o600)).expect("key mode");
    let barrier = Arc::new(tokio::sync::Barrier::new(2));
    let (a, peer_a, calls_a) = peer("synthetic-model-a", Arc::clone(&barrier)).await;
    let (b, peer_b, calls_b) = peer("synthetic-model-b", barrier).await;
    let file = fixture.0.join("models.json");
    let models = json!({"schema":"uca-agent-models/v1","models":[
        {"id":"a","label":"Model A","mode":"local-chat","base_url":a,"model":"synthetic-model-a","api_key_file":key,"timeout_ms":5000,"context_limit_tokens":131072},
        {"id":"b","label":"Model B","mode":"local-chat","base_url":b,"model":"synthetic-model-b","api_key_file":key,"timeout_ms":5000,"context_limit_tokens":131072}
    ]});
    fs::write(&file, models.to_string()).expect("catalog");
    let catalog = ModelCatalog::from_file(ChatProvider::deterministic_mock(), &file)
        .expect("operator catalog");
    let (base, server) = serve(web_router_with_models(
        Arc::new(Mutex::new(fixture.composition())),
        catalog,
    ))
    .await;
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        "x-ustc-client-protocol-major",
        reqwest::header::HeaderValue::from_static("1"),
    );
    headers.insert(
        "connection",
        reqwest::header::HeaderValue::from_static("close"),
    );
    let client = reqwest::Client::builder()
        .no_proxy()
        .timeout(std::time::Duration::from_secs(10))
        .default_headers(headers)
        .build()
        .expect("client");
    assert!(
        !reqwest::Client::new()
            .get(format!("{base}/api/v1/agent/models"))
            .send()
            .await
            .expect("no protocol header")
            .status()
            .is_success()
    );
    let view: Value = client
        .get(format!("{base}/api/v1/agent/models"))
        .send()
        .await
        .expect("catalog GET")
        .json()
        .await
        .expect("catalog view");
    assert_eq!(view["models"].as_array().expect("models").len(), 3);
    let view_text = view.to_string();
    assert!(
        !view_text.contains("127.0.0.1")
            && !view_text.contains("synthetic.key")
            && !view_text.contains("synthetic-selection-secret")
    );
    for id in ["a", "b"] {
        let model = view["models"]
            .as_array()
            .expect("models")
            .iter()
            .find(|model| model["id"] == id)
            .expect("entry");
        assert_eq!(model["tool_calling"], false);
    }
    for request in [
        chat("removed"),
        {
            let mut v = chat("a");
            v["schema"] = json!("ustc-agent-chat-request/v1");
            v
        },
        {
            let mut v = chat("a");
            v["schema"] = json!("ustc-agent-chat-request/v2");
            v
        },
        {
            let mut v = chat("a");
            v.as_object_mut().expect("object").remove("model_id");
            v
        },
    ] {
        assert_eq!(
            client
                .post(format!("{base}/api/v1/agent/chat"))
                .json(&request)
                .send()
                .await
                .expect("rejected request")
                .status(),
            reqwest::StatusCode::BAD_REQUEST
        );
    }
    assert_eq!(
        calls_a.load(Ordering::SeqCst) + calls_b.load(Ordering::SeqCst),
        0
    );
    let request_a = chat("a");
    let request_b = chat("b");
    let (a_result, b_result) = tokio::join!(
        client
            .post(format!("{base}/api/v1/agent/chat"))
            .json(&request_a)
            .send(),
        client
            .post(format!("{base}/api/v1/agent/chat"))
            .json(&request_b)
            .send()
    );
    let a_result: Value = a_result.expect("A HTTP").json().await.expect("A result");
    let b_result: Value = b_result.expect("B HTTP").json().await.expect("B result");
    assert_eq!(a_result["provider"]["model"], "synthetic-model-a");
    assert_eq!(a_result["answer"], "Answer from synthetic-model-a");
    assert_eq!(b_result["provider"]["model"], "synthetic-model-b");
    let default:Value=client.post(format!("{base}/api/v1/agent/chat")).json(&json!({"schema":"ustc-agent-chat-request/v1","messages":[{"role":"user","content":"Hello"}]})).send().await.expect("legacy").json().await.expect("legacy response");
    assert_eq!(default["provider"]["mode"], "mock");
    let status: Value = client
        .get(format!("{base}/api/v1/agent/status"))
        .send()
        .await
        .expect("status")
        .json()
        .await
        .expect("status JSON");
    assert_eq!(status["provider"]["mode"], "mock");
    let created: Value = client
        .post(format!("{base}/api/v1/agent/conversations"))
        .json(
            &json!({"schema":"chat-conversation-create/v1","request_id":"model-selection-create"}),
        )
        .send()
        .await
        .expect("create")
        .json()
        .await
        .expect("created");
    let id = created["id"].as_str().expect("conversation id");
    let intent = turn("a", "selected-turn", 0);
    let url = format!("{base}/api/v1/agent/conversations/{id}/turns");
    let original: Value = client
        .post(&url)
        .json(&intent)
        .send()
        .await
        .expect("saved turn")
        .json()
        .await
        .expect("original");
    assert_eq!(
        original["turn"]["response"]["provider"]["model"],
        "synthetic-model-a"
    );
    let replay: Value = client
        .post(&url)
        .json(&intent)
        .send()
        .await
        .expect("replay")
        .json()
        .await
        .expect("replay result");
    assert_eq!(original, replay);
    assert_eq!(
        calls_a.load(Ordering::SeqCst),
        3,
        "one stateless chat, one saved chat, one title; replay adds none"
    );
    assert_eq!(calls_b.load(Ordering::SeqCst), 1);
    server.abort();
    let _ = server.await;
    fs::write(&file, r#"{"schema":"uca-agent-models/v1","models":[]}"#)
        .expect("operator removed models");
    let catalog =
        ModelCatalog::from_file(ChatProvider::deterministic_mock(), &file).expect("new catalog");
    let (base, server) = serve(web_router_with_models(
        Arc::new(Mutex::new(fixture.composition())),
        catalog,
    ))
    .await;
    let url = format!("{base}/api/v1/agent/conversations/{id}/turns");
    let replay: Value = client
        .post(&url)
        .json(&intent)
        .send()
        .await
        .expect("restart replay")
        .json()
        .await
        .expect("restart result");
    assert_eq!(original, replay);
    assert_eq!(
        client
            .post(&url)
            .json(&turn("b", "selected-turn", 0))
            .send()
            .await
            .expect("changed selection")
            .status(),
        reqwest::StatusCode::CONFLICT
    );
    assert_eq!(
        client
            .post(&url)
            .json(&turn("a", "unknown-new", 2))
            .send()
            .await
            .expect("unknown new")
            .status(),
        reqwest::StatusCode::BAD_REQUEST
    );
    let saved: Value = client
        .get(format!("{base}/api/v1/agent/conversations/{id}"))
        .send()
        .await
        .expect("history")
        .json()
        .await
        .expect("history result");
    assert!(
        saved["title"]
            .as_str()
            .expect("persisted title")
            .ends_with("|模型选择验证")
    );
    assert_eq!(saved["revision"], 2);
    assert_eq!(saved["turns"].as_array().expect("turns").len(), 1);
    assert_eq!(
        calls_a.load(Ordering::SeqCst),
        3,
        "one stateless chat, one saved chat, one title; replay adds none"
    );
    assert_eq!(calls_b.load(Ordering::SeqCst), 1);
    server.abort();
    peer_a.abort();
    peer_b.abort();
    let _ = tokio::join!(server, peer_a, peer_b);
}
