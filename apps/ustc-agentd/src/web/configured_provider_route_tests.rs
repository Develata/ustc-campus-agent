#![allow(clippy::expect_used, clippy::unwrap_used)]

use std::fs;
use std::io::{Read as _, Write as _};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use serde_json::{Value, json};

use super::web_router_with_provider;
use crate::AffairsComposition;
use crate::chat_provider::ChatProvider;

static COUNTER: AtomicU64 = AtomicU64::new(0);

fn workspace() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root")
        .to_path_buf()
}

struct TestEnvironment {
    root: PathBuf,
}

impl TestEnvironment {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "ustc-agent-configured-provider-route-{}-{}",
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::SeqCst)
        ));
        fs::create_dir(&root).expect("create route-test directory");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt as _;
            fs::set_permissions(&root, fs::Permissions::from_mode(0o700))
                .expect("secure route-test directory");
        }
        Self { root }
    }

    fn path(&self, name: &str) -> PathBuf {
        self.root.join(name)
    }

    fn key_file(&self) -> PathBuf {
        let path = self.path("provider.key");
        fs::write(&path, b"route-test-secret\n").expect("write provider key");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt as _;
            fs::set_permissions(&path, fs::Permissions::from_mode(0o600))
                .expect("secure provider key");
        }
        path
    }

    fn composition(&self) -> AffairsComposition {
        AffairsComposition::open(
            &workspace().join("fixtures/affairs/proc-011-reviewed.json"),
            &self.path("records.json"),
            &self.path("idempotency.json"),
            &self.path("sessions.json"),
        )
        .expect("open bounded Affairs composition")
    }
}

impl Drop for TestEnvironment {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn spawn_provider_peer(expected_key: String) -> (String, thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind provider peer");
    let address = listener.local_addr().expect("provider peer address");
    let handle = thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept provider request");
        stream
            .set_read_timeout(Some(Duration::from_secs(5)))
            .expect("set provider read timeout");

        let mut request = Vec::new();
        let mut header_end = None;
        let mut expected_length = None;
        let mut buffer = [0_u8; 4096];
        loop {
            let read = stream.read(&mut buffer).expect("read provider request");
            assert_ne!(read, 0, "provider request ended before complete body");
            request.extend_from_slice(&buffer[..read]);
            assert!(
                request.len() <= 64 * 1024,
                "provider request exceeded test bound"
            );

            if header_end.is_none() {
                header_end = request
                    .windows(4)
                    .position(|window| window == b"\r\n\r\n")
                    .map(|position| position + 4);
                if let Some(end) = header_end {
                    let head = String::from_utf8_lossy(&request[..end]);
                    expected_length = head.lines().find_map(|line| {
                        let (name, value) = line.split_once(':')?;
                        name.eq_ignore_ascii_case("content-length")
                            .then(|| value.trim().parse::<usize>().expect("valid content length"))
                    });
                }
            }
            if let (Some(end), Some(length)) = (header_end, expected_length)
                && request.len() >= end + length
            {
                break;
            }
        }

        let end = header_end.expect("provider request headers");
        let head = String::from_utf8_lossy(&request[..end]);
        assert!(head.starts_with("POST /v1/chat/completions HTTP/1.1\r\n"));
        assert!(
            head.to_ascii_lowercase()
                .contains(&format!("authorization: bearer {expected_key}").to_ascii_lowercase())
        );
        let wire: Value = serde_json::from_slice(&request[end..]).expect("decode provider request");
        assert_eq!(wire["model"], "configured-route-model");
        assert_eq!(wire["stream"], false);
        assert_eq!(wire["parallel_tool_calls"], false);
        assert_eq!(wire["tool_choice"], "auto");
        assert_eq!(wire["tools"].as_array().expect("tool definitions").len(), 3);
        assert_eq!(
            wire["messages"]
                .as_array()
                .expect("complete provider messages")
                .last()
                .and_then(|message| message["content"].as_str()),
            Some("你好，请正常回答。")
        );

        let body = json!({
            "choices": [{
                "finish_reason": "stop",
                "message": {
                    "role": "assistant",
                    "content": "这是 configured provider 的完整路由回答。",
                    "tool_calls": []
                }
            }],
            "usage": {"prompt_tokens": 11, "completion_tokens": 4}
        })
        .to_string();
        write!(
            stream,
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            body.len(),
            body
        )
        .expect("write provider response");
    });
    (format!("http://{address}/v1"), handle)
}

#[tokio::test]
async fn configured_provider_serves_one_complete_http_chat_route() {
    let environment = TestEnvironment::new();
    let key_file = environment.key_file();
    let expected_key = fs::read_to_string(&key_file)
        .expect("read provider key")
        .trim()
        .to_owned();
    let (provider_base_url, provider_peer) = spawn_provider_peer(expected_key);
    let provider = ChatProvider::openai_compatible_for_test(
        &provider_base_url,
        "configured-route-model",
        &key_file,
        5_000,
    )
    .expect("configure loopback provider");
    let router =
        web_router_with_provider(Arc::new(Mutex::new(environment.composition())), provider);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind chat route");
    let address = listener.local_addr().expect("chat route address");
    let server = tokio::spawn(async move {
        axum::serve(listener, router)
            .await
            .expect("serve chat route");
    });

    let response = reqwest::Client::new()
        .post(format!("http://{address}/api/v1/agent/chat"))
        .json(&json!({
            "schema": "ustc-agent-chat-request/v1",
            "messages": [{"role": "user", "content": "你好，请正常回答。"}],
            "opportunity_context": null
        }))
        .send()
        .await
        .expect("send configured-provider chat request");
    assert_eq!(response.status(), reqwest::StatusCode::OK);
    assert_eq!(
        response
            .headers()
            .get(reqwest::header::CACHE_CONTROL)
            .and_then(|value| value.to_str().ok()),
        Some("no-store")
    );
    let payload: Value = response.json().await.expect("decode chat response");
    assert_eq!(payload["schema"], "ustc-agent-chat-response/v1");
    assert_eq!(
        payload["answer"],
        "这是 configured provider 的完整路由回答。"
    );
    assert_eq!(payload["provider"]["mode"], "openai-compatible");
    assert_eq!(payload["provider"]["model"], "configured-route-model");
    assert_eq!(payload["usage"]["input_tokens"], 11);
    assert_eq!(payload["usage"]["output_tokens"], 4);
    assert_eq!(payload["tool_trace"], json!([]));
    assert!(
        payload["run_id"]
            .as_str()
            .is_some_and(|run_id| run_id.starts_with("chat-run:"))
    );

    provider_peer.join().expect("provider peer completed");
    server.abort();
    let _ = server.await;
}

#[tokio::test]
async fn configured_provider_duplicate_calendar_record_persists_only_one_item() {
    let environment = TestEnvironment::new();
    let key_file = environment.key_file();
    let turns = Arc::new(AtomicU64::new(0));
    let peer_turns = Arc::clone(&turns);
    let peer_router = axum::Router::new().route(
        "/v1/chat/completions",
        axum::routing::post(
            move |headers: axum::http::HeaderMap, axum::Json(wire): axum::Json<Value>| {
                let turns = Arc::clone(&peer_turns);
                async move {
                    assert_eq!(headers["authorization"], "Bearer route-test-secret");
                    assert_eq!(wire["model"], "configured-route-model");
                    assert_eq!(wire["parallel_tool_calls"], false);
                    let turn = turns.fetch_add(1, Ordering::SeqCst);
                    let message = match turn {
                        0 => {
                            assert_eq!(
                                wire["messages"].as_array().unwrap().last().unwrap()["content"],
                                "记录事项：提交开题报告"
                            );
                            json!({
                                "finish_reason": "tool_calls",
                                "message": {
                                    "role": "assistant",
                                    "content": null,
                                    "tool_calls": (["provider-first", "provider-repeat"].map(|id| json!({
                                        "id": id,
                                        "type": "function",
                                        "function": {
                                            "name": "simple_calendar_items",
                                            "arguments": json!({
                                                "action": "record",
                                                "title": "提交开题报告"
                                            }).to_string()
                                        }
                                    })))
                                }
                            })
                        }
                        1 => {
                            let results = wire["messages"]
                                .as_array()
                                .unwrap()
                                .iter()
                                .filter(|message| message["role"] == "tool")
                                .collect::<Vec<_>>();
                            assert_eq!(results.len(), 2);
                            assert_eq!(results[0]["tool_call_id"], "provider-first");
                            assert_eq!(results[1]["tool_call_id"], "provider-repeat");
                            let first: Value = serde_json::from_str(results[0]["content"].as_str().unwrap()).unwrap();
                            let repeat: Value = serde_json::from_str(results[1]["content"].as_str().unwrap()).unwrap();
                            assert_eq!(first["status"], "succeeded");
                            assert_eq!(repeat["status"], "denied");
                            assert_eq!(repeat["data"]["code"], "calendar_mutation_intent_consumed");
                            json!({
                                "finish_reason": "stop",
                                "message": {"role": "assistant", "content": "已记录一条事项；重复调用被拒绝。"}
                            })
                        }
                        _ => panic!("provider exceeded the two-response test sequence"),
                    };
                    axum::Json(json!({"choices": [message]}))
                }
            },
        ),
    )
    .layer(axum::extract::DefaultBodyLimit::max(64 * 1024));
    let peer_listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let peer_address = peer_listener.local_addr().unwrap();
    let peer = tokio::spawn(async move {
        axum::serve(peer_listener, peer_router).await.unwrap();
    });
    let provider = ChatProvider::openai_compatible_for_test(
        &format!("http://{peer_address}/v1"),
        "configured-route-model",
        &key_file,
        5_000,
    )
    .unwrap();
    let composition = Arc::new(Mutex::new(environment.composition()));
    let router = web_router_with_provider(Arc::clone(&composition), provider);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let server = tokio::spawn(async move {
        axum::serve(listener, router).await.unwrap();
    });
    let response = reqwest::Client::builder()
        .no_proxy()
        .timeout(Duration::from_secs(15))
        .build()
        .unwrap()
        .post(format!("http://{address}/api/v1/agent/chat"))
        .json(&json!({
            "schema": "ustc-agent-chat-request/v1",
            "messages": [{"role": "user", "content": "记录事项：提交开题报告"}]
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), reqwest::StatusCode::OK);
    let payload: Value = response.json().await.unwrap();
    assert_eq!(
        payload["tool_trace"],
        json!([
            {"call_id": "call-1", "tool": "simple_calendar_items", "status": "succeeded"},
            {"call_id": "call-2", "tool": "simple_calendar_items", "status": "denied"}
        ])
    );
    assert_eq!(turns.load(Ordering::SeqCst), 2);
    let items = composition.lock().unwrap().calendar_items().unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].id, "calendar:item:1");
    assert_eq!(items[0].title, "提交开题报告");

    server.abort();
    let _ = server.await;
    peer.abort();
    let _ = peer.await;
    drop(composition);
    let mut reopened = environment.composition();
    assert_eq!(reopened.calendar_items().unwrap(), items);
}

#[tokio::test]
async fn saved_conversation_activity_observes_admitted_tools_without_reexecuting() {
    let environment = TestEnvironment::new();
    let key_file = environment.key_file();
    let turns = Arc::new(AtomicU64::new(0));
    let release = Arc::new(AtomicBool::new(false));
    let peer_turns = Arc::clone(&turns);
    let peer_release = Arc::clone(&release);
    let peer_router = axum::Router::new().route(
        "/v1/chat/completions",
        axum::routing::post(move |axum::Json(wire): axum::Json<Value>| {
            let turns = Arc::clone(&peer_turns);
            let release = Arc::clone(&peer_release);
            async move {
                let turn = turns.fetch_add(1, Ordering::SeqCst);
                let choice = match turn {
                    0 => json!({
                        "finish_reason": "tool_calls",
                        "message": {"role":"assistant", "content":null, "tool_calls":[{
                            "id":"private-provider-activity-call", "type":"function",
                            "function":{"name":"simple_calendar_items", "arguments":
                                json!({"action":"record", "title":"活动状态核验事项"}).to_string()}
                        }]}
                    }),
                    1 => {
                        let messages = wire["messages"].as_array().unwrap();
                        let result = messages.iter().find(|message| message["role"] == "tool").unwrap();
                        assert_eq!(result["tool_call_id"], "private-provider-activity-call");
                        let result: Value = serde_json::from_str(result["content"].as_str().unwrap()).unwrap();
                        assert_eq!(result["status"], "succeeded");
                        tokio::time::timeout(Duration::from_secs(10), async {
                            while !release.load(Ordering::SeqCst) {
                                tokio::time::sleep(Duration::from_millis(10)).await;
                            }
                        }).await.expect("test releases second provider reply");
                        json!({"finish_reason":"stop", "message":{"role":"assistant", "content":"已记录事项。"}})
                    }
                    2 => {
                        assert_eq!(wire["tools"], json!([]), "naming grants no tools");
                        let messages=wire["messages"].as_array().expect("naming messages");
                        assert_eq!(messages.len(),2,"naming excludes dialogue/tool history");
                        assert_eq!(messages[0]["role"],"system");
                        assert!(messages[0]["content"].as_str().expect("title instruction").contains("话题标题"));
                        assert_eq!(messages[1],json!({"role":"user","content":"记录事项：活动状态核验事项"}));
                        json!({"finish_reason":"stop", "message":{"role":"assistant", "content":"日历事项记录"}})
                    }
                    _ => panic!("read-only activity must not call the provider again"),
                };
                axum::Json(json!({"choices":[choice]}))
            }
        }),
    ).layer(axum::extract::DefaultBodyLimit::max(64 * 1024));
    let peer_listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let peer_address = peer_listener.local_addr().unwrap();
    let peer = tokio::spawn(async move {
        axum::serve(peer_listener, peer_router).await.unwrap();
    });
    let provider = ChatProvider::openai_compatible_for_test(
        &format!("http://{peer_address}/v1"),
        "configured-route-model",
        &key_file,
        15_000,
    )
    .unwrap();
    let composition = Arc::new(Mutex::new(environment.composition()));
    let router = web_router_with_provider(Arc::clone(&composition), provider);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let server = tokio::spawn(async move {
        axum::serve(listener, router).await.unwrap();
    });
    let client = reqwest::Client::builder()
        .no_proxy()
        .timeout(Duration::from_secs(20))
        .build()
        .unwrap();
    let collection = format!("http://{address}/api/v1/agent/conversations");
    let response = client
        .post(&collection)
        .header("X-USTC-Client-Protocol-Major", "1")
        .json(&json!({"schema":"chat-conversation-create/v1", "request_id":"activity-create"}))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), reqwest::StatusCode::OK);
    let created: Value = response.json().await.unwrap();
    let id = created["id"].as_str().unwrap();
    let activity_url = format!("{collection}/{id}/activity");
    let intent = json!({"schema":"chat-conversation-turn/v1", "request_id":"activity-turn",
        "expected_revision":created["revision"], "message":"记录事项：活动状态核验事项", "opportunity_context":null});
    let post = client
        .post(format!("{collection}/{id}/turns"))
        .header("X-USTC-Client-Protocol-Major", "1")
        .json(&intent);
    let pending = tokio::spawn(async move { post.send().await.unwrap() });
    tokio::time::timeout(Duration::from_secs(10), async {
        while turns.load(Ordering::SeqCst) < 2 {
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .expect("provider reaches gated second request");
    for (major, status) in [
        (None, reqwest::StatusCode::CONFLICT),
        (Some("0"), reqwest::StatusCode::UPGRADE_REQUIRED),
        (Some("2"), reqwest::StatusCode::CONFLICT),
    ] {
        let mut read = client.get(&activity_url);
        if let Some(major) = major {
            read = read.header("X-USTC-Client-Protocol-Major", major);
        }
        let response = read.send().await.unwrap();
        assert_eq!(response.status(), status);
        let rejected: Value = response.json().await.unwrap();
        assert!(rejected.get("steps").is_none());
    }
    let expected_steps = json!([
        {"id":"model-1", "kind":"model", "tool":null, "status":"succeeded"},
        {"id":"call-1", "kind":"tool", "tool":"simple_calendar_items", "status":"succeeded"},
        {"id":"model-2", "kind":"model", "tool":null, "status":"running"}
    ]);
    for _ in 0..3 {
        let response = client
            .get(&activity_url)
            .header("X-USTC-Client-Protocol-Major", "1")
            .send()
            .await
            .unwrap();
        assert_eq!(response.status(), reqwest::StatusCode::OK);
        assert_eq!(
            response.headers()[reqwest::header::CACHE_CONTROL],
            "no-store"
        );
        let body = response.text().await.unwrap();
        for forbidden in [
            "private-provider-activity-call",
            "活动状态核验事项",
            "arguments",
            "route-test-secret",
        ] {
            assert!(
                !body.contains(forbidden),
                "activity excludes provider/private details"
            );
        }
        let activity: Value = serde_json::from_str(&body).unwrap();
        assert_eq!(
            activity,
            json!({"schema":"chat-conversation-activity/v1", "conversation_id":id,
            "request_id":"activity-turn", "phase":"running", "sequence":5, "steps":expected_steps})
        );
        assert_eq!(turns.load(Ordering::SeqCst), 2);
        assert_eq!(
            composition.lock().unwrap().calendar_items().unwrap().len(),
            1
        );
        assert!(
            !pending.is_finished(),
            "provider remains gated while GET returns"
        );
    }
    release.store(true, Ordering::SeqCst);
    let response = pending.await.unwrap();
    assert_eq!(response.status(), reqwest::StatusCode::OK);
    let result: Value = response.json().await.unwrap();
    assert_eq!(result["turn"]["phase"], "completed");
    assert_eq!(
        result["turn"]["response"]["tool_trace"],
        json!([
            {"call_id":"call-1", "tool":"simple_calendar_items", "status":"succeeded"}
        ])
    );
    let terminal: Value = client
        .get(&activity_url)
        .header("X-USTC-Client-Protocol-Major", "1")
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(
        terminal,
        json!({"schema":"chat-conversation-activity/v1", "conversation_id":id,
        "request_id":"activity-turn", "phase":"completed", "sequence":15, "steps":[
            {"id":"call-1", "kind":"tool", "tool":"simple_calendar_items", "status":"succeeded"}
        ]})
    );
    let replay: Value = client
        .post(format!("{collection}/{id}/turns"))
        .header("X-USTC-Client-Protocol-Major", "1")
        .json(&intent)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(
        replay, result,
        "replay retains the original completed response"
    );
    let detail: Value = client
        .get(format!("{collection}/{id}"))
        .header("X-USTC-Client-Protocol-Major", "1")
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert!(detail["title"].as_str().unwrap().ends_with("|日历事项记录"));
    assert_eq!(
        turns.load(Ordering::SeqCst),
        3,
        "two chat calls and one naming call; activity and replay add none"
    );
    let items = composition.lock().unwrap().calendar_items().unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].title, "活动状态核验事项");
    server.abort();
    let _ = server.await;
    peer.abort();
    let _ = peer.await;
}
