//! CALENDAR-PROPOSAL-001 real HTTP and controlled-model integration.
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
            "uca-calendar-proposal-route-{}-{nonce}",
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

fn proposal(request: &str, mutation: Value) -> Value {
    json!({"schema":"calendar-proposal/v1","request_id":request,"mutation":mutation})
}
async fn snapshot(client: &reqwest::Client, base: &str) -> Value {
    client
        .get(format!("{base}/api/v1/calendar/proposals"))
        .send()
        .await
        .expect("get")
        .error_for_status()
        .expect("list status")
        .json()
        .await
        .expect("list JSON")
}
async fn post_json(client: &reqwest::Client, url: &str, value: Value) -> Value {
    client
        .post(url)
        .json(&value)
        .send()
        .await
        .expect("post")
        .error_for_status()
        .expect("post status")
        .json()
        .await
        .expect("JSON")
}
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn calendar_proposal_http_exact_confirmation_update_conflict_and_restart() {
    let fixture = Fixture::new();
    let router = web_router_with_provider(
        Arc::new(Mutex::new(fixture.composition())),
        ChatProvider::deterministic_mock(),
    );
    let (base, server) = serve(router).await;
    let client = reqwest::Client::builder()
        .no_proxy()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .expect("client");
    let draft = proposal(
        "draft-one",
        json!({"action":"record","title":"申请材料","scheduled_for":"2026-09-09T14:00:00+08:00"}),
    );
    let proposed = post_json(
        &client,
        &format!("{base}/api/v1/calendar/proposals"),
        draft.clone(),
    )
    .await;
    assert_eq!(proposed["proposal"]["status"], "pending");
    assert_eq!(
        snapshot(&client, &base).await["items"]
            .as_array()
            .expect("controlled Calendar fixture")
            .len(),
        0
    );
    assert_eq!(
        post_json(&client, &format!("{base}/api/v1/calendar/proposals"), draft).await,
        proposed
    );
    let id = proposed["proposal"]["id"]
        .as_str()
        .expect("controlled Calendar fixture");
    let confirm_url = format!("{base}/api/v1/calendar/proposals/{id}/confirm");
    let invalid = client
        .post(&confirm_url)
        .json(&json!({"schema":"calendar-proposal-confirm/v1","title":"forged"}))
        .send()
        .await
        .expect("controlled Calendar fixture");
    assert_eq!(invalid.status(), StatusCode::BAD_REQUEST);
    let foreign = client
        .post(&confirm_url)
        .header("Origin", "http://untrusted.example")
        .json(&json!({"schema":"calendar-proposal-confirm/v1"}))
        .send()
        .await
        .expect("controlled Calendar fixture");
    assert_eq!(foreign.status(), StatusCode::FORBIDDEN);
    assert_eq!(
        snapshot(&client, &base).await["items"]
            .as_array()
            .expect("controlled Calendar fixture")
            .len(),
        0
    );
    let confirmed = post_json(
        &client,
        &confirm_url,
        json!({"schema":"calendar-proposal-confirm/v1"}),
    )
    .await;
    assert_eq!(confirmed["proposal"]["status"], "applied");
    assert_eq!(
        confirmed["proposal"]["result"]["scheduled_for"],
        "2026-09-09T14:00:00+08:00"
    );
    let item_id = confirmed["proposal"]["result"]["id"]
        .as_str()
        .expect("controlled Calendar fixture");
    let edit = post_json(&client,&format!("{base}/api/v1/calendar/proposals"),proposal("edit",json!({"action":"update","item_id":item_id,"title":"改到下午四点","scheduled_for":"2026-09-09T16:00:00+08:00"}))).await;
    let stale = post_json(
        &client,
        &format!("{base}/api/v1/calendar/proposals"),
        proposal("stale-delete", json!({"action":"delete","item_id":item_id})),
    )
    .await;
    let edit_id = edit["proposal"]["id"]
        .as_str()
        .expect("controlled Calendar fixture");
    post_json(
        &client,
        &format!("{base}/api/v1/calendar/proposals/{edit_id}/confirm"),
        json!({"schema":"calendar-proposal-confirm/v1"}),
    )
    .await;
    let stale_id = stale["proposal"]["id"]
        .as_str()
        .expect("controlled Calendar fixture");
    let conflict = client
        .post(format!(
            "{base}/api/v1/calendar/proposals/{stale_id}/confirm"
        ))
        .json(&json!({"schema":"calendar-proposal-confirm/v1"}))
        .send()
        .await
        .expect("controlled Calendar fixture");
    assert_eq!(conflict.status(), StatusCode::CONFLICT);
    post_json(
        &client,
        &format!("{base}/api/v1/calendar/proposals/{stale_id}/cancel"),
        json!({"schema":"calendar-proposal-cancel/v1"}),
    )
    .await;
    server.abort();
    let _ = server.await;
    let (base, server) = serve(web_router_with_provider(
        Arc::new(Mutex::new(fixture.composition())),
        ChatProvider::deterministic_mock(),
    ))
    .await;
    let replay = post_json(
        &client,
        &format!("{base}/api/v1/calendar/proposals/{id}/confirm"),
        json!({"schema":"calendar-proposal-confirm/v1"}),
    )
    .await;
    assert_eq!(
        replay, confirmed,
        "original effect receipt survives later edits and restart"
    );
    let saved = snapshot(&client, &base).await;
    assert_eq!(
        saved["items"]
            .as_array()
            .expect("controlled Calendar fixture")
            .len(),
        1
    );
    assert_eq!(saved["items"][0]["title"], "改到下午四点");
    server.abort();
    let _ = server.await;
}
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn calendar_proposal_real_model_tool_is_pending_until_separate_user_confirmation() {
    let fixture = Fixture::new();
    let key = fixture.0.join("synthetic.key");
    fs::write(&key, "synthetic-key").expect("controlled Calendar fixture");
    fs::set_permissions(&key, fs::Permissions::from_mode(0o600))
        .expect("controlled Calendar fixture");
    let (peer_base,peer)=serve(Router::new().route("/v1/chat/completions",post(|Json(wire):Json<Value>|async move {
        if wire["messages"].as_array().expect("controlled Calendar fixture").iter().any(|m|m["role"]=="tool") {
            Json(json!({"choices":[{"finish_reason":"stop","message":{"role":"assistant","content":"请在日历面板核对提案后确认；尚未写入，也没有提醒。"}}]}))
        } else {
            let tool=wire["tools"].as_array().expect("controlled Calendar fixture").iter().find(|t|t["function"]["name"]=="simple_calendar_items").expect("controlled Calendar fixture");
            assert!(tool["function"]["parameters"]["properties"]["action"]["enum"].as_array().expect("controlled Calendar fixture").contains(&json!("propose")));
            Json(json!({"choices":[{"finish_reason":"tool_calls","message":{"role":"assistant","content":null,"tool_calls":[{"id":"calendar-plan","type":"function","function":{"name":"simple_calendar_items","arguments":json!({"action":"propose","mutation":{"action":"record","title":"材料办理","scheduled_for":"2026-09-09T14:00:00+08:00"}}).to_string()}}]}}]}))
        }
    }))).await;
    let provider = ChatProvider::openai_compatible_for_test(
        &format!("{peer_base}/v1"),
        "synthetic-calendar",
        &key,
        10000,
    )
    .expect("controlled Calendar fixture");
    let (base, server) = serve(web_router_with_provider(
        Arc::new(Mutex::new(fixture.composition())),
        provider,
    ))
    .await;
    let client = reqwest::Client::builder()
        .no_proxy()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .expect("controlled Calendar fixture");
    let response=post_json(&client,&format!("{base}/api/v1/agent/chat"),json!({"schema":"ustc-agent-chat-request/v1","messages":[{"role":"user","content":"帮我安排9月9日下午两点办理材料"}]})).await;
    assert_eq!(response["tool_trace"][0]["status"], "succeeded");
    let saved = snapshot(&client, &base).await;
    assert!(
        saved["items"]
            .as_array()
            .expect("controlled Calendar fixture")
            .is_empty()
    );
    assert_eq!(
        saved["proposals"]
            .as_array()
            .expect("controlled Calendar fixture")
            .len(),
        1
    );
    assert_eq!(saved["proposals"][0]["status"], "pending");
    let id = saved["proposals"][0]["id"]
        .as_str()
        .expect("controlled Calendar fixture");
    post_json(
        &client,
        &format!("{base}/api/v1/calendar/proposals/{id}/confirm"),
        json!({"schema":"calendar-proposal-confirm/v1"}),
    )
    .await;
    assert_eq!(
        snapshot(&client, &base).await["items"]
            .as_array()
            .expect("controlled Calendar fixture")
            .len(),
        1
    );
    server.abort();
    peer.abort();
    let _ = server.await;
    let _ = peer.await;
}
