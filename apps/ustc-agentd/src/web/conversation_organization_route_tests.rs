//! CONVERSATION-ORGANIZE-001 real HTTP and restart proof.
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
            "uca-organization-route-{}-{nonce}",
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

async fn request(
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
fn command(id: &str, revision: u64, action: Value) -> Value {
    json!({"schema":"chat-conversation-manage/v2","request_id":id,"expected_revision":revision,"action":action})
}
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn conversation_organization_http_rename_pin_group_order_replay_and_restart() {
    let fixture = Fixture::new();
    let (base, server) = serve(web_router_with_provider(
        Arc::new(Mutex::new(fixture.composition())),
        ChatProvider::deterministic_mock(),
    ))
    .await;
    let client = reqwest::Client::builder()
        .no_proxy()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .expect("client");
    let endpoint = format!("{base}/api/v1/agent/conversations");
    let mut conversations = Vec::new();
    for n in 0..3 {
        conversations.push(request(&client,reqwest::Method::POST,&endpoint,Some(json!({"schema":"chat-conversation-create/v1","request_id":format!("organization-{n}")}))).await);
    }
    let ids: Vec<String> = conversations
        .iter()
        .map(|c| c["id"].as_str().expect("id").to_owned())
        .collect();
    let listing = request(&client, reqwest::Method::GET, &endpoint, None).await;
    assert_eq!(
        listing["conversations"]
            .as_array()
            .expect("list")
            .iter()
            .map(|c| c["id"].as_str().expect("id"))
            .collect::<Vec<_>>(),
        vec![ids[2].as_str(), ids[1].as_str(), ids[0].as_str()]
    );
    let manage = format!("{endpoint}/{}/manage", ids[0]);
    let date = conversations[0]["organization"]["date"]
        .as_str()
        .expect("server date")
        .to_owned();
    let rename = command(
        "rename-topic",
        0,
        json!({"kind":"rename","title":"  校园安排  "}),
    );
    let named = request(
        &client,
        reqwest::Method::POST,
        &manage,
        Some(rename.clone()),
    )
    .await;
    assert_eq!(named["schema"], "chat-conversation-manage-result/v2");
    assert_eq!(named["title"], format!("{date}|校园安排"));
    for (title, n) in [("250101|改日期", 0), ("a|b", 1), ("\u{202e}hide", 2)] {
        let response = client
            .post(&manage)
            .header("X-USTC-Client-Protocol-Major", "1")
            .json(&command(
                &format!("invalid-{n}"),
                1,
                json!({"kind":"rename","title":title}),
            ))
            .send()
            .await
            .expect("invalid rename");
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }
    let pinned = request(
        &client,
        reqwest::Method::POST,
        &manage,
        Some(command("pin-one", 1, json!({"kind":"pin","pinned":true}))),
    )
    .await;
    assert_eq!(pinned["title"], named["title"]);
    assert_eq!(pinned["organization"]["pinned"], true);
    assert_eq!(pinned["organization"]["date"], date);
    let grouped = request(
        &client,
        reqwest::Method::POST,
        &manage,
        Some(command(
            "group-one",
            2,
            json!({"kind":"group","group":"  学业  "}),
        )),
    )
    .await;
    assert_eq!(grouped["organization"]["group"], "学业");
    let list = request(&client, reqwest::Method::GET, &endpoint, None).await;
    assert_eq!(list["conversations"][0]["id"], ids[0]);
    assert_eq!(
        request(
            &client,
            reqwest::Method::POST,
            &manage,
            Some(rename.clone())
        )
        .await,
        named,
        "old rename replay stays exact after pin/group"
    );
    server.abort();
    let _ = server.await;
    let (base, server) = serve(web_router_with_provider(
        Arc::new(Mutex::new(fixture.composition())),
        ChatProvider::deterministic_mock(),
    ))
    .await;
    let endpoint = format!("{base}/api/v1/agent/conversations");
    let manage = format!("{endpoint}/{}/manage", ids[0]);
    let detail = request(
        &client,
        reqwest::Method::GET,
        &format!("{endpoint}/{}", ids[0]),
        None,
    )
    .await;
    assert_eq!(detail["organization"], grouped["organization"]);
    assert_eq!(
        request(&client, reqwest::Method::POST, &manage, Some(rename)).await,
        named
    );
    request(
        &client,
        reqwest::Method::POST,
        &manage,
        Some(command("unpin", 3, json!({"kind":"pin","pinned":false}))),
    )
    .await;
    let ungrouped = request(
        &client,
        reqwest::Method::POST,
        &manage,
        Some(command("ungroup", 4, json!({"kind":"group","group":null}))),
    )
    .await;
    assert_eq!(ungrouped["organization"]["date"], date);
    assert_eq!(ungrouped["organization"]["group"], Value::Null);
    let list = request(&client, reqwest::Method::GET, &endpoint, None).await;
    assert_eq!(
        list["conversations"]
            .as_array()
            .expect("list")
            .iter()
            .map(|c| c["id"].as_str().expect("id"))
            .collect::<Vec<_>>(),
        vec![ids[2].as_str(), ids[1].as_str(), ids[0].as_str()],
        "metadata changes do not bump same-date order"
    );
    server.abort();
    let _ = server.await;
}
