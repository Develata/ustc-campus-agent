#![allow(clippy::expect_used, clippy::unwrap_used)]

use std::fs;
use std::io::{Read as _, Write as _};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use serde_json::{Value, json};

use super::web_router_with_provider;
use crate::AffairsComposition;
use crate::chat_provider::ChatProvider;
use crate::local_access::deterministic_access_for_tests;

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
    let router = web_router_with_provider(
        Arc::new(Mutex::new(environment.composition())),
        provider,
        deterministic_access_for_tests(),
    );
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind chat route");
    let address = listener.local_addr().expect("chat route address");
    let server = tokio::spawn(async move {
        axum::serve(listener, router)
            .await
            .expect("serve chat route");
    });

    let client = reqwest::Client::new();
    let chat_body = json!({
        "schema": "ustc-agent-chat-request/v1",
        "messages": [{"role": "user", "content": "你好，请正常回答。"}],
        "opportunity_context": null
    });
    let unauthorized = client
        .post(format!("http://{address}/api/v1/agent/chat"))
        .json(&chat_body)
        .send()
        .await
        .expect("send unauthenticated chat request");
    assert_eq!(unauthorized.status(), reqwest::StatusCode::UNAUTHORIZED);

    for path in [
        "/api/v1/demo/administrator/affairs/publication",
        "/api/v1/demo/administrator/changes/publication",
    ] {
        let public_status = client
            .get(format!("http://{address}{path}"))
            .send()
            .await
            .expect("send anonymous publication status request");
        assert!(
            matches!(
                public_status.status(),
                reqwest::StatusCode::OK | reqwest::StatusCode::SERVICE_UNAVAILABLE
            ),
            "anonymous publication status must reach the domain handler"
        );
    }

    let unauthorized_publication = client
        .post(format!(
            "http://{address}/api/v1/demo/administrator/affairs/publication"
        ))
        .header("x-ustc-agent-administrator-demo", "confirm-v1")
        .json(&json!({}))
        .send()
        .await
        .expect("send unauthenticated publication request");
    assert_eq!(
        unauthorized_publication.status(),
        reqwest::StatusCode::UNAUTHORIZED
    );

    let login = client
        .post(format!("http://{address}/api/v1/auth/login"))
        .json(&json!({
            "schema": "ustc-local-access-login/v1",
            "username": "admin",
            "password": "correct horse battery staple"
        }))
        .send()
        .await
        .expect("send login request");
    assert_eq!(login.status(), reqwest::StatusCode::OK);
    let set_cookie = login
        .headers()
        .get(reqwest::header::SET_COOKIE)
        .and_then(|value| value.to_str().ok())
        .expect("login sets one session cookie");
    for required in [
        "uca_session=",
        "Path=/",
        "HttpOnly",
        "SameSite=Strict",
        "Max-Age=43200",
    ] {
        assert!(
            set_cookie.contains(required),
            "missing cookie attribute: {required}"
        );
    }
    assert!(!set_cookie.contains("Secure"));
    let session_cookie = set_cookie
        .split(';')
        .next()
        .expect("login sets one session cookie")
        .to_owned();

    let response = client
        .post(format!("http://{address}/api/v1/agent/chat"))
        .header(reqwest::header::COOKIE, session_cookie)
        .json(&chat_body)
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
