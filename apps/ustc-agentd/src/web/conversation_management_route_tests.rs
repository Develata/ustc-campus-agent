//! CONVERSATION-MANAGE-001 real HTTP lifecycle with one controlled in-flight model.
use super::*;
use serde_json::Value;
use std::{
    fs,
    os::unix::fs::PermissionsExt,
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
        let path = std::env::temp_dir().join(format!(
            "uca-conversation-management-route-{}-{nonce}",
            std::process::id()
        ));
        fs::create_dir(&path).expect("directory");
        fs::set_permissions(&path, fs::Permissions::from_mode(0o700)).expect("private");
        Self(path)
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
        .expect("listen");
    let address = listener.local_addr().expect("address");
    let task = tokio::spawn(async move {
        axum::serve(listener, router).await.expect("serve");
    });
    (format!("http://{address}"), task)
}
fn rename(id: &str, revision: u64, title: &str) -> Value {
    json!({"schema":"chat-conversation-manage/v1","request_id":id,"expected_revision":revision,"action":{"kind":"rename","title":title}})
}
fn delete(id: &str, revision: u64) -> Value {
    json!({"schema":"chat-conversation-manage/v1","request_id":id,"expected_revision":revision,"action":{"kind":"delete"}})
}
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn conversation_management_http_running_conflict_delete_and_restart_replays_are_bounded() {
    let fixture = Fixture::new();
    let key = fixture.0.join("synthetic.key");
    fs::write(&key, "synthetic-management-key").expect("key");
    fs::set_permissions(&key, fs::Permissions::from_mode(0o600)).expect("private key");
    let calls = Arc::new(AtomicUsize::new(0));
    let reached = Arc::new(tokio::sync::Notify::new());
    let release = Arc::new(tokio::sync::Notify::new());
    let observed = Arc::clone(&calls);
    let requested = Arc::clone(&reached);
    let released = Arc::clone(&release);
    let (peer_base,peer)=serve(Router::new().route("/v1/chat/completions",post(move |Json(wire):Json<Value>| {let observed=Arc::clone(&observed);let requested=Arc::clone(&requested);let released=Arc::clone(&released);async move {assert_eq!(wire["model"],"synthetic-management");observed.fetch_add(1,Ordering::SeqCst);requested.notify_one();released.notified().await;Json(json!({"choices":[{"finish_reason":"stop","message":{"role":"assistant","content":"Private retained answer"}}]}))}}))).await;
    let provider = ChatProvider::local_chat(
        &format!("{peer_base}/v1"),
        "synthetic-management",
        &key,
        10000,
        131072,
    )
    .expect("provider");
    let (base, server) = serve(web_router_with_provider(
        Arc::new(Mutex::new(fixture.composition())),
        provider,
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
        .timeout(std::time::Duration::from_secs(15))
        .default_headers(headers)
        .build()
        .expect("client");
    let create = json!({"schema":"chat-conversation-create/v1","request_id":"create"});
    let created: Value = client
        .post(format!("{base}/api/v1/agent/conversations"))
        .json(&create)
        .send()
        .await
        .expect("create")
        .json()
        .await
        .expect("created");
    let id = created["id"].as_str().expect("id");
    let url = format!("{base}/api/v1/agent/conversations/{id}/manage");
    assert!(
        !reqwest::Client::new()
            .post(&url)
            .json(&rename("no-major", 0, "Name"))
            .send()
            .await
            .expect("no protocol")
            .status()
            .is_success()
    );
    assert_eq!(
        client
            .post(&url)
            .header("origin", "https://evil.invalid")
            .json(&rename("origin", 0, "Name"))
            .send()
            .await
            .expect("foreign origin")
            .status(),
        reqwest::StatusCode::FORBIDDEN
    );
    let mut smuggled = delete("bad", 0);
    smuggled["action"]["title"] = json!("extra");
    assert_eq!(
        client
            .post(&url)
            .json(&smuggled)
            .send()
            .await
            .expect("closed delete")
            .status(),
        reqwest::StatusCode::BAD_REQUEST
    );
    let first = rename("rename-first", 0, "  Explicit title  ");
    let first_receipt: Value = client
        .post(&url)
        .json(&first)
        .send()
        .await
        .expect("rename")
        .json()
        .await
        .expect("receipt");
    assert_eq!(first_receipt["title"], "Explicit title");
    assert_eq!(first_receipt["revision"], 1);
    assert_eq!(
        client
            .post(&url)
            .json(&delete("rename-first", 0))
            .send()
            .await
            .expect("cross-action conflict")
            .status(),
        reqwest::StatusCode::CONFLICT
    );
    let turns_url = format!("{base}/api/v1/agent/conversations/{id}/turns");
    let turn = json!({"schema":"chat-conversation-turn/v2","model_id":"default","request_id":"turn","expected_revision":1,"message":"This must not replace my title"});
    let sending_client = client.clone();
    let sending_url = turns_url.clone();
    let sending_turn = turn.clone();
    let in_flight = tokio::spawn(async move {
        sending_client
            .post(sending_url)
            .json(&sending_turn)
            .send()
            .await
            .expect("turn")
            .json::<Value>()
            .await
            .expect("turn result")
    });
    tokio::time::timeout(std::time::Duration::from_secs(5), reached.notified())
        .await
        .expect("provider running");
    for action in [rename("busy", 2, "Busy name"), delete("busy", 2)] {
        assert_eq!(
            client
                .post(&url)
                .json(&action)
                .send()
                .await
                .expect("busy request")
                .status(),
            reqwest::StatusCode::CONFLICT
        );
    }
    release.notify_one();
    let turn_result = in_flight.await.expect("completed turn");
    assert_eq!(turn_result["revision"], 3);
    let current: Value = client
        .get(format!("{base}/api/v1/agent/conversations/{id}"))
        .send()
        .await
        .expect("detail")
        .json()
        .await
        .expect("detail body");
    assert_eq!(current["title"], "Explicit title");
    assert_eq!(
        client
            .post(&url)
            .json(&rename("stale", 1, "Name"))
            .send()
            .await
            .expect("stale request")
            .status(),
        reqwest::StatusCode::CONFLICT
    );
    assert_eq!(
        client
            .post(&url)
            .json(&delete("turn", 3))
            .send()
            .await
            .expect("turn namespace conflict")
            .status(),
        reqwest::StatusCode::CONFLICT
    );
    let deletion = delete("delete", 3);
    let deleted: Value = client
        .post(&url)
        .json(&deletion)
        .send()
        .await
        .expect("delete")
        .json()
        .await
        .expect("delete receipt");
    assert_eq!(deleted["deleted"], true);
    assert_eq!(deleted["revision"], 4);
    for suffix in ["", "/activity"] {
        assert_eq!(
            client
                .get(format!("{base}/api/v1/agent/conversations/{id}{suffix}"))
                .send()
                .await
                .expect("deleted read")
                .status(),
            reqwest::StatusCode::NOT_FOUND
        );
    }
    assert_eq!(
        client
            .post(&turns_url)
            .json(&turn)
            .send()
            .await
            .expect("old turn")
            .status(),
        reqwest::StatusCode::NOT_FOUND
    );
    assert_eq!(
        client
            .post(format!("{base}/api/v1/agent/conversations"))
            .json(&create)
            .send()
            .await
            .expect("old create")
            .status(),
        reqwest::StatusCode::NOT_FOUND
    );
    let historical: Value = client
        .post(&url)
        .json(&first)
        .send()
        .await
        .expect("old rename replay")
        .json()
        .await
        .expect("old receipt");
    assert_eq!(historical, first_receipt);
    let list: Value = client
        .get(format!("{base}/api/v1/agent/conversations"))
        .send()
        .await
        .expect("list")
        .json()
        .await
        .expect("list body");
    assert_eq!(list["conversations"], json!([]));
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    server.abort();
    let _ = server.await;
    let (base, server) = serve(web_router_with_provider(
        Arc::new(Mutex::new(fixture.composition())),
        ChatProvider::deterministic_mock(),
    ))
    .await;
    let url = format!("{base}/api/v1/agent/conversations/{id}/manage");
    let replay: Value = client
        .post(&url)
        .json(&deletion)
        .send()
        .await
        .expect("restart replay")
        .json()
        .await
        .expect("replay body");
    assert_eq!(replay, deleted);
    let historical: Value = client
        .post(&url)
        .json(&first)
        .send()
        .await
        .expect("restart historical rename")
        .json()
        .await
        .expect("historical body");
    assert_eq!(historical, first_receipt);
    assert_eq!(
        client
            .get(format!("{base}/api/v1/agent/conversations/{id}"))
            .send()
            .await
            .expect("still gone")
            .status(),
        reqwest::StatusCode::NOT_FOUND
    );
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    server.abort();
    peer.abort();
    let _ = tokio::join!(server, peer);
}
