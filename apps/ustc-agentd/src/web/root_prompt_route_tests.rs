//! ROOT-PROMPT-001 real settings HTTP and provider projection.
use super::*;
use serde_json::Value;
use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
};
struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "uca-root-prompt-route-{}-{nonce}",
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

async fn json_request(
    client: &reqwest::Client,
    method: reqwest::Method,
    url: &str,
    body: Option<Value>,
) -> Value {
    let mut request = client
        .request(method, url)
        .header("X-USTC-Client-Protocol-Major", "1");
    if let Some(body) = body {
        request = request.json(&body);
    }
    request
        .send()
        .await
        .expect("HTTP")
        .error_for_status()
        .expect("success")
        .json()
        .await
        .expect("JSON")
}
fn update(revision: u64, text: &str) -> Value {
    json!({"schema":"agent-root-prompt-update/v1","expected_revision":revision,"text":text})
}
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn root_prompt_http_admission_projection_replay_clear_and_restart() {
    let fixture = Fixture::new();
    let key = fixture.0.join("synthetic.key");
    fs::write(&key, "synthetic-key").expect("key");
    fs::set_permissions(&key, fs::Permissions::from_mode(0o600)).expect("private key");
    let seen = Arc::new(Mutex::new(Vec::<Value>::new()));
    let peer_seen = seen.clone();
    let (peer_base, peer) = serve(Router::new().route("/v1/chat/completions", post(move |Json(wire):Json<Value>| {
        let seen=peer_seen.clone(); async move {
            seen.lock().expect("requests").push(wire);
            Json(json!({"choices":[{"finish_reason":"stop","message":{"role":"assistant","content":"收到"}}]}))
        }
    }))).await;
    let provider = ChatProvider::openai_compatible_for_test(
        &format!("{peer_base}/v1"),
        "test-prompt",
        &key,
        10000,
    )
    .expect("provider");
    let (base, server) = serve(web_router_with_provider(
        Arc::new(Mutex::new(fixture.composition())),
        provider,
    ))
    .await;
    let client = reqwest::Client::builder()
        .no_proxy()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .expect("client");
    let settings = format!("{base}/api/v1/agent/root-prompt");
    assert_ne!(
        client
            .get(&settings)
            .send()
            .await
            .expect("no major")
            .status(),
        StatusCode::OK
    );
    let empty = json_request(&client, reqwest::Method::GET, &settings, None).await;
    assert_eq!(
        empty,
        json!({"schema":"agent-root-prompt/v1","revision":0,"text":""})
    );
    for (body, origin, expected) in [
        (
            json!({"schema":"agent-root-prompt-update/v1","expected_revision":0,"text":"x","user":"peer"}),
            None,
            StatusCode::BAD_REQUEST,
        ),
        (
            update(0, "x"),
            Some("https://unrelated.invalid"),
            StatusCode::FORBIDDEN,
        ),
        (update(0, &"界".repeat(3000)), None, StatusCode::BAD_REQUEST),
    ] {
        let mut req = client
            .put(&settings)
            .header("X-USTC-Client-Protocol-Major", "1")
            .json(&body);
        if let Some(origin) = origin {
            req = req.header("Origin", origin);
        }
        assert_eq!(
            req.send().await.expect("rejected update").status(),
            expected
        );
    }
    let saved = json_request(
        &client,
        reqwest::Method::PUT,
        &settings,
        Some(update(0, "你是我的学习伙伴。\n回答先给可执行步骤。")),
    )
    .await;
    assert_eq!(saved["revision"], 1);
    assert_eq!(
        json_request(
            &client,
            reqwest::Method::PUT,
            &settings,
            Some(update(0, saved["text"].as_str().expect("text")))
        )
        .await,
        saved
    );
    let conflict = client
        .put(&settings)
        .header("X-USTC-Client-Protocol-Major", "1")
        .json(&update(0, "stale overwrite"))
        .send()
        .await
        .expect("stale");
    assert_eq!(conflict.status(), StatusCode::CONFLICT);
    let conversation = json_request(
        &client,
        reqwest::Method::POST,
        &format!("{base}/api/v1/agent/conversations"),
        Some(json!({"schema":"chat-conversation-create/v1","request_id":"prompt-check"})),
    )
    .await;
    let id = conversation["id"].as_str().expect("id");
    let turn_url = format!("{base}/api/v1/agent/conversations/{id}/turns");
    let turn = json!({"schema":"chat-conversation-turn/v1","request_id":"first","expected_revision":0,"message":"怎么安排复习？","prompt_customization":{"text":"只列三点"}});
    let result = json_request(
        &client,
        reqwest::Method::POST,
        &turn_url,
        Some(turn.clone()),
    )
    .await;
    assert_eq!(result["turn"]["phase"], "completed");
    {
        let requests = seen.lock().expect("requests");
        let wire = requests
            .iter()
            .find(|v| v.get("tools").is_some())
            .expect("chat provider request");
        assert_eq!(wire["messages"][0]["role"], "system");
        let messages = wire["messages"].as_array().expect("messages");
        let personal = messages
            .iter()
            .position(|m| {
                m["content"]
                    .as_str()
                    .is_some_and(|s| s.contains("你是我的学习伙伴"))
            })
            .expect("saved setting reaches provider");
        let per_turn = messages
            .iter()
            .position(|m| {
                m["content"]
                    .as_str()
                    .is_some_and(|s| s.contains("只列三点"))
            })
            .expect("request preference");
        assert!(personal > 0 && personal < per_turn);
        assert_eq!(messages[personal]["role"], "user");
        assert!(
            !result.to_string().contains("你是我的学习伙伴"),
            "setting is not transcript metadata"
        );
    }
    let cleared = json_request(
        &client,
        reqwest::Method::PUT,
        &settings,
        Some(update(1, "")),
    )
    .await;
    assert_eq!(cleared["text"], "");
    let calls = seen.lock().expect("calls").len();
    assert_eq!(
        json_request(&client, reqwest::Method::POST, &turn_url, Some(turn)).await,
        result
    );
    assert_eq!(
        seen.lock().expect("calls").len(),
        calls,
        "replay does not rerun with changed setting"
    );
    let next = json!({"schema":"chat-conversation-turn/v1","request_id":"second","expected_revision":result["revision"],"message":"继续"});
    json_request(&client, reqwest::Method::POST, &turn_url, Some(next)).await;
    assert!(
        !seen
            .lock()
            .expect("requests")
            .last()
            .expect("new request")
            .to_string()
            .contains("你是我的学习伙伴")
    );
    let final_saved = json_request(
        &client,
        reqwest::Method::PUT,
        &settings,
        Some(update(2, "重启后仍保留的个人指令")),
    )
    .await;
    server.abort();
    let _ = server.await;
    let (base, server) = serve(web_router_with_provider(
        Arc::new(Mutex::new(fixture.composition())),
        ChatProvider::deterministic_mock(),
    ))
    .await;
    assert_eq!(
        json_request(
            &client,
            reqwest::Method::GET,
            &format!("{base}/api/v1/agent/root-prompt"),
            None
        )
        .await,
        final_saved
    );
    server.abort();
    peer.abort();
    let _ = server.await;
    let _ = peer.await;
}
