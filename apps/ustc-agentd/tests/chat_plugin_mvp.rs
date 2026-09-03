#![allow(clippy::unwrap_used)]

//! Deterministic loopback proof for the bounded chat/provider/two-Plugin MVP.

use std::fs;
use std::io::{BufRead, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc;
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use serde_json::{Value, json};

static COUNTER: AtomicU64 = AtomicU64::new(0);
const PROTOCOL_HEADER: &str = "X-USTC-Client-Protocol-Major: 1";

struct FakeProvider {
    origin: String,
    handle: Option<JoinHandle<Vec<Value>>>,
}

impl FakeProvider {
    fn scripted_with_rejection(responses: Vec<Value>, rejected_index: Option<usize>) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind fake provider");
        let address = listener.local_addr().expect("fake provider address");
        let handle = thread::spawn(move || {
            let mut requests = Vec::new();
            for (index, response) in responses.into_iter().enumerate() {
                let (mut stream, _) = listener.accept().expect("accept provider request");
                stream
                    .set_read_timeout(Some(Duration::from_secs(10)))
                    .expect("provider read timeout");
                let request = read_http_request_json(&mut stream);
                let response_body = response.to_string();
                let status = if rejected_index == Some(index) {
                    "500 Internal Server Error"
                } else {
                    "200 OK"
                };
                write!(
                    stream,
                    "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    response_body.len(),
                    response_body
                )
                .expect("write provider response");
                stream.flush().expect("flush provider response");
                requests.push(request);
            }
            requests
        });
        Self {
            origin: format!("http://{address}"),
            handle: Some(handle),
        }
    }

    fn finish(mut self) -> Vec<Value> {
        self.handle
            .take()
            .expect("provider handle")
            .join()
            .expect("fake provider thread")
    }
}

struct WebServer {
    child: Child,
    endpoint: String,
    temp_dir: PathBuf,
    profile_store: PathBuf,
}

impl WebServer {
    fn start(provider_origin: &str) -> Self {
        let workspace = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(|path| path.parent())
            .expect("workspace root")
            .to_path_buf();
        let suffix = COUNTER.fetch_add(1, Ordering::SeqCst);
        let temp_dir = std::env::temp_dir().join(format!(
            "ustc-agentd-chat-plugin-mvp-{}-{suffix}",
            std::process::id()
        ));
        fs::create_dir_all(&temp_dir).expect("create test state directory");
        fs::set_permissions(&temp_dir, fs::Permissions::from_mode(0o700))
            .expect("secure test state directory");

        let profile_store = temp_dir.join("opportunity-profiles.json");
        let mut child = Command::new(env!("CARGO_BIN_EXE_ustc-agentd"))
            .args([
                "serve-web",
                "--bind",
                "127.0.0.1:0",
                "--fixture",
                workspace
                    .join("fixtures/affairs/proc-011-reviewed.json")
                    .to_str()
                    .expect("affairs fixture path"),
                "--change-fixture",
                workspace
                    .join("fixtures/change-radar/academic-calendar-demo-reviewed.json")
                    .to_str()
                    .expect("change fixture path"),
                "--opportunity-fixture",
                workspace
                    .join("fixtures/opportunity-graph/course-planning-demo-reviewed.json")
                    .to_str()
                    .expect("opportunity fixture path"),
                "--opportunity-catalog",
                workspace
                    .join("market/fixtures/course-planning/minimal-v0.json")
                    .to_str()
                    .expect("course catalog path"),
                "--opportunity-profile-store",
                profile_store.to_str().expect("profile store path"),
                "--store",
                temp_dir
                    .join("affairs-records.json")
                    .to_str()
                    .expect("record store path"),
                "--idempotency",
                temp_dir
                    .join("affairs-idempotency.json")
                    .to_str()
                    .expect("idempotency path"),
                "--session-store",
                temp_dir
                    .join("m00-sessions.json")
                    .to_str()
                    .expect("session store path"),
            ])
            .env("USTC_AGENT_MODEL_BASE_URL", provider_origin)
            .env("USTC_AGENT_MODEL_API_KEY", "synthetic-test-key")
            .env("USTC_AGENT_MODEL", "fake-model")
            .env("USTC_AGENT_MODEL_TIMEOUT_SECS", "5")
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .expect("spawn web server");

        let stdout = child.stdout.take().expect("web stdout");
        let (sender, receiver) = mpsc::channel();
        thread::spawn(move || {
            for line in std::io::BufReader::new(stdout).lines() {
                let Ok(line) = line else { break };
                if sender.send(line).is_err() {
                    break;
                }
            }
        });
        let deadline = Instant::now() + Duration::from_secs(30);
        let endpoint = loop {
            let remaining = deadline.saturating_duration_since(Instant::now());
            match receiver.recv_timeout(remaining) {
                Ok(line) => {
                    if let Some(value) = line.strip_prefix("web listening http://") {
                        break value.trim().to_owned();
                    }
                }
                Err(_) => {
                    let _ = child.kill();
                    let _ = child.wait();
                    panic!("web server did not publish endpoint");
                }
            }
        };
        Self {
            child,
            endpoint,
            temp_dir,
            profile_store,
        }
    }

    fn post_chat(&self, body: &Value) -> HttpResponse {
        self.post_chat_with_protocol(body, true)
    }

    fn post_chat_without_protocol(&self, body: &Value) -> HttpResponse {
        self.post_chat_with_protocol(body, false)
    }

    fn post_chat_with_protocol(&self, body: &Value, include_protocol: bool) -> HttpResponse {
        let body = body.to_string();
        let protocol = if include_protocol {
            format!("{PROTOCOL_HEADER}\r\n")
        } else {
            String::new()
        };
        let mut stream = TcpStream::connect(&self.endpoint).expect("connect web server");
        stream
            .set_read_timeout(Some(Duration::from_secs(15)))
            .expect("web read timeout");
        write!(
            stream,
            "POST /api/v1/agent/chat HTTP/1.1\r\nHost: {}\r\nAccept: application/json\r\n{protocol}Content-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            self.endpoint,
            body.len(),
            body
        )
        .expect("write chat request");
        stream.flush().expect("flush chat request");
        let mut bytes = Vec::new();
        stream.read_to_end(&mut bytes).expect("read chat response");
        assert!(!bytes.is_empty(), "empty HTTP response");
        HttpResponse::parse(&bytes)
    }
}

impl Drop for WebServer {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
        let _ = fs::remove_dir_all(&self.temp_dir);
    }
}

struct HttpResponse {
    status: u16,
    body: Value,
}

impl HttpResponse {
    fn parse(bytes: &[u8]) -> Self {
        let split = bytes
            .windows(4)
            .position(|window| window == b"\r\n\r\n")
            .expect("HTTP response separator");
        let headers = std::str::from_utf8(&bytes[..split]).expect("UTF-8 response headers");
        let status = headers
            .lines()
            .next()
            .and_then(|line| line.split_ascii_whitespace().nth(1))
            .and_then(|value| value.parse::<u16>().ok())
            .expect("HTTP response status");
        let body = serde_json::from_slice(&bytes[split + 4..]).expect("JSON response body");
        Self { status, body }
    }
}

fn read_http_request_json(stream: &mut TcpStream) -> Value {
    let mut bytes = Vec::new();
    let mut buffer = [0_u8; 4096];
    let header_end = loop {
        let read = stream.read(&mut buffer).expect("read provider request");
        assert!(read > 0, "provider request ended before headers");
        bytes.extend_from_slice(&buffer[..read]);
        assert!(bytes.len() <= 256 * 1024, "provider request too large");
        if let Some(index) = bytes.windows(4).position(|window| window == b"\r\n\r\n") {
            break index + 4;
        }
    };
    let headers = std::str::from_utf8(&bytes[..header_end]).expect("provider request headers");
    let content_length = headers
        .lines()
        .find_map(|line| {
            let (name, value) = line.split_once(':')?;
            name.eq_ignore_ascii_case("content-length")
                .then(|| value.trim().parse::<usize>().expect("content length"))
        })
        .expect("provider content length");
    while bytes.len() < header_end + content_length {
        let read = stream.read(&mut buffer).expect("read provider body");
        assert!(read > 0, "provider body ended early");
        bytes.extend_from_slice(&buffer[..read]);
    }
    serde_json::from_slice(&bytes[header_end..header_end + content_length])
        .expect("provider request JSON")
}

fn response(id: &str, output: Value) -> Value {
    json!({
        "id": id,
        "object": "response",
        "status": "completed",
        "model": "fake-model",
        "output": [output]
    })
}

fn response_with_reasoning(id: &str, text: &str) -> Value {
    json!({
        "id": id,
        "object": "response",
        "status": "completed",
        "model": "fake-model",
        "output": [
            {"type": "reasoning", "id": format!("reasoning-{id}"), "summary": []},
            text_output(text)
        ]
    })
}

fn text_output(text: &str) -> Value {
    json!({
        "type": "message",
        "id": "message-fixture",
        "status": "completed",
        "role": "assistant",
        "content": [{"type": "output_text", "text": text, "annotations": []}]
    })
}

fn tool_output(id: &str, name: &str, arguments: Value) -> Value {
    json!({
        "type": "function_call",
        "id": format!("function-{id}"),
        "call_id": id,
        "status": "completed",
        "name": name,
        "arguments": arguments.to_string()
    })
}

fn tool_response(id: &str, name: &str, arguments: Value) -> Value {
    response(id, tool_output(&format!("call-{id}"), name, arguments))
}

fn course_arguments() -> Value {
    json!({
        "completed_courses": ["MATH1001", "MATH1002", "CS1001", "PHYS1001"],
        "min_credits": 9,
        "max_credits": 12,
        "preference_weights": [
            {"course_code": "MATH2001", "weight": 9},
            {"course_code": "MATH2003", "weight": 8},
            {"course_code": "CS2006", "weight": 7}
        ]
    })
}

#[test]
fn direct_chat_and_two_plugin_rounds_are_reproducible_and_fail_closed() {
    let provider = FakeProvider::scripted_with_rejection(
        vec![
            response_with_reasoning("direct", "你好，我可以帮助查询校园事项与解释课程建议。"),
            tool_response(
                "affairs-call",
                "ustc_affairs_lookup",
                json!({"procedure_id": "proc:ustc:undergraduate:transcript-certificate"}),
            ),
            response(
                "affairs-final",
                text_output("成绩单办理信息来自 DemoReviewed 事项证据。"),
            ),
            tool_response("course-denied", "ustc_course_advice", course_arguments()),
            tool_response("unknown-tool", "ustc_write_enrollment", json!({})),
            tool_response("course-call", "ustc_course_advice", course_arguments()),
            response(
                "course-final",
                text_output("建议方案已生成；这是解释性建议，不会执行选课。"),
            ),
            tool_response(
                "invalid-affairs",
                "ustc_affairs_lookup",
                json!({"procedure_id": ""}),
            ),
            json!({"error": "synthetic provider rejection"}),
            tool_response(
                "second-round-first",
                "ustc_affairs_lookup",
                json!({"procedure_id": "proc:ustc:undergraduate:transcript-certificate"}),
            ),
            tool_response(
                "second-round-again",
                "ustc_course_advice",
                course_arguments(),
            ),
            response("escaped-limit", text_output("bounded input accepted")),
        ],
        Some(8),
    );
    let server = WebServer::start(&provider.origin);

    let direct = server.post_chat(&json!({
        "message": "你好",
        "course_profile_consent": false
    }));
    assert_eq!(direct.status, 200);
    assert_eq!(direct.body["grounded"], false);
    assert_eq!(direct.body["used_tools"], json!([]));
    assert_eq!(direct.body["model"], "fake-model");

    let affairs = server.post_chat(&json!({
        "message": "如何办理成绩单？",
        "course_profile_consent": false
    }));
    assert_eq!(affairs.status, 200);
    assert_eq!(affairs.body["grounded"], true);
    assert_eq!(affairs.body["used_tools"], json!(["ustc_affairs_lookup"]));

    let denied = server.post_chat(&json!({
        "message": "根据我的课程情况给建议",
        "course_profile_consent": false
    }));
    assert_eq!(denied.status, 422);
    assert_eq!(denied.body["error"], "chat_tool_denied");

    let unknown = server.post_chat(&json!({
        "message": "帮我写入教务系统",
        "course_profile_consent": true
    }));
    assert_eq!(unknown.status, 502);
    assert_eq!(unknown.body["error"], "chat_tool_error");

    let course = server.post_chat(&json!({
        "message": "我已完成数学分析等先修课，请解释下学期课程建议",
        "course_profile_consent": true
    }));
    assert_eq!(course.status, 200);
    assert_eq!(course.body["grounded"], true);
    assert_eq!(course.body["used_tools"], json!(["ustc_course_advice"]));
    assert!(
        course.body["answer"]
            .as_str()
            .is_some_and(|answer| answer.contains("不会执行选课"))
    );

    let profile_state: Value = serde_json::from_slice(
        &fs::read(&server.profile_store).expect("read persisted private profile state"),
    )
    .expect("parse persisted private profile state");
    assert_eq!(profile_state["active"].as_array().map(Vec::len), Some(0));
    assert_eq!(
        profile_state["tombstones"].as_array().map(Vec::len),
        Some(1)
    );
    let persisted = profile_state.to_string();
    for private_value in ["MATH1001", "MATH2001", "CS2006"] {
        assert!(!persisted.contains(private_value));
    }

    let invalid_json = server.post_chat(&json!({
        "message": "hello",
        "course_profile_consent": false,
        "unexpected": true
    }));
    assert_eq!(invalid_json.status, 400);
    assert_eq!(invalid_json.body["error"], "invalid_chat_json");

    let invalid_arguments = server.post_chat(&json!({
        "message": "查询空事项标识",
        "course_profile_consent": false
    }));
    assert_eq!(invalid_arguments.status, 422);
    assert_eq!(
        invalid_arguments.body["error"],
        "chat_tool_invalid_arguments"
    );

    let rejected = server.post_chat(&json!({
        "message": "触发模拟 provider 拒绝",
        "course_profile_consent": false
    }));
    assert_eq!(rejected.status, 502);
    assert_eq!(rejected.body["error"], "chat_provider_error");

    let second_round = server.post_chat(&json!({
        "message": "触发第二轮工具调用",
        "course_profile_consent": true
    }));
    assert_eq!(second_round.status, 502);
    assert_eq!(second_round.body["error"], "chat_provider_error");

    let escaped_limit = server.post_chat(&json!({
        "message": "\\".repeat(8_192),
        "course_profile_consent": false
    }));
    assert_eq!(escaped_limit.status, 200);

    let missing_protocol = server.post_chat_without_protocol(&json!({
        "message": "missing protocol",
        "course_profile_consent": false
    }));
    assert_eq!(missing_protocol.status, 409);

    let requests = provider.finish();
    assert_eq!(requests.len(), 12);
    assert!(requests.iter().all(|request| request["store"] == false));
    assert!(
        requests
            .iter()
            .all(|request| request.get("previous_response_id").is_none())
    );
    assert_eq!(requests[2]["input"].as_array().map(Vec::len), Some(4));
    assert_eq!(requests[6]["input"].as_array().map(Vec::len), Some(4));
    assert_eq!(requests[10]["input"].as_array().map(Vec::len), Some(4));
}
