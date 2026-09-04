#![allow(clippy::unwrap_used)]

//! Real loopback HTTP/Web smoke for the bounded reviewed-procedure slice.

use std::fs;
use std::io::{BufRead, Read, Write};
use std::net::TcpStream;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc;
use std::time::{Duration, Instant};

use serde_json::{Value, json};
use sha2::{Digest, Sha256};

static COUNTER: AtomicU64 = AtomicU64::new(0);

struct WebServer {
    child: Child,
    endpoint: String,
    temp_dir: PathBuf,
}

#[derive(Clone, Copy, Debug)]
enum MarketState {
    Active,
    Disabled,
    Revoked,
    Omitted,
}

#[derive(Clone, Copy)]
enum PluginTarget {
    Affairs,
    Change,
    Opportunity,
}

impl MarketState {
    fn enabled(self) -> bool {
        !matches!(self, Self::Disabled | Self::Omitted)
    }

    fn grant_active(self) -> bool {
        !matches!(self, Self::Revoked | Self::Omitted)
    }
}

fn fixture_with_market_state(
    source: &std::path::Path,
    destination: &std::path::Path,
    state: MarketState,
    copy_change_evidence: bool,
) -> PathBuf {
    if matches!(state, MarketState::Active) {
        return source.to_path_buf();
    }
    assert!(!matches!(state, MarketState::Omitted));

    let mut value: Value = serde_json::from_slice(&fs::read(source).expect("read fixture"))
        .expect("parse fixture JSON");
    value["market_enabled"] = Value::Bool(state.enabled());
    value["market_grant_active"] = Value::Bool(state.grant_active());

    if copy_change_evidence {
        let source_parent = source.parent().expect("change fixture parent");
        let destination_parent = destination.parent().expect("temporary fixture parent");
        for revision in ["old_revision", "new_revision"] {
            for field in ["raw_path", "normalized_path"] {
                let relative = value[revision][field]
                    .as_str()
                    .expect("change evidence path");
                let target = destination_parent.join(relative);
                fs::create_dir_all(target.parent().expect("change evidence target parent"))
                    .expect("create change evidence target parent");
                fs::copy(source_parent.join(relative), &target)
                    .expect("copy change evidence into isolated fixture tree");
            }
        }
    }

    fs::write(
        destination,
        serde_json::to_vec_pretty(&value).expect("encode fixture JSON"),
    )
    .expect("write market-state fixture");
    destination.to_path_buf()
}

impl WebServer {
    fn start() -> Self {
        Self::start_with_market_states(
            MarketState::Active,
            MarketState::Active,
            MarketState::Active,
        )
    }

    fn start_affairs_only() -> Self {
        Self::start_with_market_states(
            MarketState::Active,
            MarketState::Omitted,
            MarketState::Omitted,
        )
    }

    fn start_with_market_states(
        affairs_state: MarketState,
        change_state: MarketState,
        opportunity_state: MarketState,
    ) -> Self {
        let workspace = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(|path| path.parent())
            .expect("workspace root")
            .to_path_buf();
        let fixture = workspace.join("fixtures/affairs/proc-011-reviewed.json");
        let change_fixture =
            workspace.join("fixtures/change-radar/academic-calendar-demo-reviewed.json");
        let opportunity_fixture =
            workspace.join("fixtures/opportunity-graph/course-planning-demo-reviewed.json");
        let opportunity_catalog = workspace.join("market/fixtures/course-planning/minimal-v0.json");
        assert!(fixture.is_file(), "reviewed fixture must exist");
        assert!(
            change_fixture.is_file(),
            "reviewed change fixture must exist"
        );
        assert!(
            opportunity_fixture.is_file(),
            "reviewed opportunity fixture must exist"
        );
        assert!(
            opportunity_catalog.is_file(),
            "opportunity catalog must exist"
        );

        let suffix = COUNTER.fetch_add(1, Ordering::SeqCst);
        let temp_dir =
            std::env::temp_dir().join(format!("ustc-agentd-web-{}-{suffix}", std::process::id()));
        fs::create_dir_all(&temp_dir).expect("create web test directory");
        fs::set_permissions(&temp_dir, fs::Permissions::from_mode(0o700))
            .expect("secure web test directory");
        let fixture = fixture_with_market_state(
            &fixture,
            &temp_dir.join("affairs-fixture.json"),
            affairs_state,
            false,
        );
        let change_fixture = (!matches!(change_state, MarketState::Omitted)).then(|| {
            fixture_with_market_state(
                &change_fixture,
                &temp_dir.join("change-fixture.json"),
                change_state,
                true,
            )
        });
        let opportunity_fixture = (!matches!(opportunity_state, MarketState::Omitted)).then(|| {
            fixture_with_market_state(
                &opportunity_fixture,
                &temp_dir.join("opportunity-fixture.json"),
                opportunity_state,
                false,
            )
        });
        let store = temp_dir.join("records.json");
        let idempotency = temp_dir.join("idempotency.json");
        let sessions = temp_dir.join("m00-sessions.json");
        let opportunity_profile_store = temp_dir.join("opportunity-profiles.json");

        let mut command = Command::new(env!("CARGO_BIN_EXE_ustc-agentd"));
        command.args([
            "serve-web",
            "--bind",
            "127.0.0.1:0",
            "--fixture",
            fixture.to_str().expect("fixture path utf8"),
        ]);
        if let Some(change_fixture) = &change_fixture {
            command.args([
                "--change-fixture",
                change_fixture.to_str().expect("change fixture path utf8"),
            ]);
        }
        if let Some(opportunity_fixture) = &opportunity_fixture {
            command.args([
                "--opportunity-fixture",
                opportunity_fixture
                    .to_str()
                    .expect("opportunity fixture path utf8"),
                "--opportunity-catalog",
                opportunity_catalog
                    .to_str()
                    .expect("opportunity catalog path utf8"),
                "--opportunity-profile-store",
                opportunity_profile_store
                    .to_str()
                    .expect("opportunity profile store path utf8"),
            ]);
        }
        let mut child = command
            .args([
                "--store",
                store.to_str().expect("store path utf8"),
                "--idempotency",
                idempotency.to_str().expect("idempotency path utf8"),
                "--session-store",
                sessions.to_str().expect("session store path utf8"),
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("spawn ustc-agentd serve-web");

        let stdout = child.stdout.take().expect("web stdout");
        let (sender, receiver) = mpsc::channel();
        std::thread::spawn(move || {
            for line in std::io::BufReader::new(stdout).lines() {
                let Ok(line) = line else { break };
                if sender.send(line).is_err() {
                    break;
                }
            }
        });

        let deadline = Instant::now() + Duration::from_secs(30);
        let mut endpoint = None;
        while Instant::now() < deadline {
            let remaining = deadline.saturating_duration_since(Instant::now());
            match receiver.recv_timeout(remaining) {
                Ok(line) => {
                    if let Some(value) = line.strip_prefix("web listening http://") {
                        endpoint = Some(value.trim().to_owned());
                        break;
                    }
                }
                Err(_) => break,
            }
        }
        let endpoint = endpoint.unwrap_or_else(|| {
            let _ = child.kill();
            let _ = child.wait();
            let mut stderr = String::new();
            if let Some(mut pipe) = child.stderr.take() {
                let _ = pipe.read_to_string(&mut stderr);
            }
            panic!(
                "web server did not publish endpoint for affairs={affairs_state:?}, change={change_state:?}, opportunity={opportunity_state:?}: {stderr}"
            );
        });
        Self {
            child,
            endpoint,
            temp_dir,
        }
    }

    fn get(&self, path: &str) -> HttpResponse {
        self.get_with_protocol_headers(path, &["1"])
    }

    fn get_without_protocol(&self, path: &str) -> HttpResponse {
        self.get_with_protocol_headers(path, &[])
    }

    fn get_with_protocol(&self, path: &str, major: &str) -> HttpResponse {
        self.get_with_protocol_headers(path, &[major])
    }

    fn get_with_protocol_headers(&self, path: &str, majors: &[&str]) -> HttpResponse {
        let mut stream = TcpStream::connect(&self.endpoint).expect("connect web server");
        stream
            .set_read_timeout(Some(Duration::from_secs(10)))
            .expect("set read timeout");
        let protocol_headers = majors
            .iter()
            .map(|major| format!("X-USTC-Client-Protocol-Major: {major}\r\n"))
            .collect::<String>();
        write!(
            stream,
            "GET {path} HTTP/1.1\r\nHost: {}\r\nAccept: application/json,text/html,*/*\r\n{protocol_headers}X-USTC-Opportunity-Confirmation: confirmed\r\nConnection: close\r\n\r\n",
            self.endpoint,
        )
        .expect("write HTTP request");
        stream.flush().expect("flush HTTP request");
        let mut bytes = Vec::new();
        stream.read_to_end(&mut bytes).expect("read HTTP response");
        HttpResponse::parse(&bytes)
    }

    fn get_admin(&self, path: &str) -> HttpResponse {
        let mut stream = TcpStream::connect(&self.endpoint).expect("connect web server");
        stream
            .set_read_timeout(Some(Duration::from_secs(10)))
            .expect("set read timeout");
        write!(
            stream,
            "GET {path} HTTP/1.1\r\nHost: {}\r\nAccept: application/json\r\nX-USTC-Agent-Administrator-Demo: confirm-v1\r\nConnection: close\r\n\r\n",
            self.endpoint
        )
        .expect("write administrator HTTP request");
        stream.flush().expect("flush administrator HTTP request");
        let mut bytes = Vec::new();
        stream
            .read_to_end(&mut bytes)
            .expect("read administrator HTTP response");
        HttpResponse::parse(&bytes)
    }

    fn post_admin_json(&self, path: &str, body: &Value) -> HttpResponse {
        let body = body.to_string();
        let mut stream = TcpStream::connect(&self.endpoint).expect("connect web server");
        stream
            .set_read_timeout(Some(Duration::from_secs(10)))
            .expect("set read timeout");
        write!(
            stream,
            "POST {path} HTTP/1.1\r\nHost: {}\r\nAccept: application/json\r\nX-USTC-Agent-Administrator-Demo: confirm-v1\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            self.endpoint,
            body.len(),
            body
        )
        .expect("write administrator HTTP request");
        stream.flush().expect("flush administrator HTTP request");
        let mut bytes = Vec::new();
        stream
            .read_to_end(&mut bytes)
            .expect("read administrator HTTP response");
        HttpResponse::parse(&bytes)
    }

    fn post_json(&self, path: &str, body: &Value) -> HttpResponse {
        let body = body.to_string();
        self.post_raw_with_confirmation(path, "application/json", &body, true)
    }

    fn post_json_without_opportunity_confirmation(&self, path: &str, body: &Value) -> HttpResponse {
        let body = body.to_string();
        self.post_raw_with_confirmation(path, "application/json", &body, false)
    }

    fn post_raw(&self, path: &str, content_type: &str, body: &str) -> HttpResponse {
        self.post_raw_with_confirmation(path, content_type, body, true)
    }

    fn post_raw_with_confirmation(
        &self,
        path: &str,
        content_type: &str,
        body: &str,
        confirmed: bool,
    ) -> HttpResponse {
        let mut stream = TcpStream::connect(&self.endpoint).expect("connect web server");
        stream
            .set_read_timeout(Some(Duration::from_secs(10)))
            .expect("set read timeout");
        let confirmation_header = if confirmed {
            "X-USTC-Opportunity-Confirmation: confirmed\r\n"
        } else {
            ""
        };
        write!(
            stream,
            "POST {path} HTTP/1.1\r\nHost: {}\r\nAccept: application/json\r\nContent-Type: {content_type}\r\n{confirmation_header}Content-Length: {}\r\nConnection: close\r\n\r\n{}",
            self.endpoint,
            body.len(),
            body
        )
        .expect("write HTTP request");
        stream.flush().expect("flush HTTP request");
        let mut bytes = Vec::new();
        stream.read_to_end(&mut bytes).expect("read HTTP response");
        HttpResponse::parse(&bytes)
    }

    fn post_json_with_authority(
        &self,
        path: &str,
        body: &Value,
        host: &str,
        origin: Option<&str>,
    ) -> HttpResponse {
        let body = body.to_string();
        let mut stream = TcpStream::connect(&self.endpoint).expect("connect web server");
        stream
            .set_read_timeout(Some(Duration::from_secs(10)))
            .expect("set read timeout");
        let origin_header = origin
            .map(|value| format!("Origin: {value}\r\n"))
            .unwrap_or_default();
        write!(
            stream,
            "POST {path} HTTP/1.1\r\nHost: {host}\r\n{origin_header}Accept: application/json\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            body.len(),
            body,
        )
        .expect("write authority HTTP request");
        stream.flush().expect("flush authority HTTP request");
        let mut bytes = Vec::new();
        stream.read_to_end(&mut bytes).expect("read HTTP response");
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
    status: String,
    headers: String,
    body: String,
}

impl HttpResponse {
    fn parse(bytes: &[u8]) -> Self {
        let text = String::from_utf8(bytes.to_vec()).expect("HTTP response utf8");
        let (head, body) = text.split_once("\r\n\r\n").expect("HTTP separator");
        let mut lines = head.lines();
        let status = lines.next().expect("HTTP status").to_owned();
        Self {
            status,
            headers: lines.collect::<Vec<_>>().join("\n").to_ascii_lowercase(),
            body: body.to_owned(),
        }
    }
}

fn valid_opportunity_profile_body() -> Value {
    json!({
        "consent": true,
        "request_id": "req:web:opportunity-create-regression",
        "correlation_id": "corr:web:opportunity-create-regression",
        "idempotency_key": "idem:web:opportunity-create-regression",
        "consented_at": 1_787_792_400_000i64,
        "completed_courses": ["MATH1001", "MATH1002", "CS1001", "PHYS1001"],
        "min_credits": 9,
        "max_credits": 12,
        "preference_weights": [
            {"course_code": "MATH2001", "weight": 9},
            {"course_code": "MATH2003", "weight": 8},
            {"course_code": "CS2006", "weight": 7},
            {"course_code": "PHYS2003", "weight": 5},
            {"course_code": "HUM2001", "weight": 4},
            {"course_code": "GEN2001", "weight": 3},
            {"course_code": "LANG2001", "weight": 2}
        ]
    })
}

#[test]
fn loopback_host_and_origin_admission_precede_chat_dispatch() {
    let server = WebServer::start();
    let body = json!({
        "schema": "ustc-agent-chat-request/v1",
        "messages": [{"role": "user", "content": "普通问题"}],
        "opportunity_context": null
    });

    let rebound = server.post_json_with_authority(
        "/api/v1/agent/chat",
        &body,
        "campus-attacker.example:8787",
        None,
    );
    assert!(rebound.status.contains(" 421 "), "{}", rebound.status);
    assert!(rebound.body.contains("invalid_loopback_host"));

    let cross_origin = server.post_json_with_authority(
        "/api/v1/agent/chat",
        &body,
        &server.endpoint,
        Some("http://campus-attacker.example"),
    );
    assert!(
        cross_origin.status.contains(" 403 "),
        "{}",
        cross_origin.status
    );
    assert!(cross_origin.body.contains("cross_origin_request_forbidden"));

    let origin = format!("http://{}", server.endpoint);
    let admitted = server.post_json_with_authority(
        "/api/v1/agent/chat",
        &body,
        &server.endpoint,
        Some(&origin),
    );
    assert!(admitted.status.contains(" 200 "), "{}", admitted.status);
}

#[test]
fn agent_chat_http_route_maps_success_and_closed_request_failures() {
    let server = WebServer::start();
    let path = "/api/v1/agent/chat";

    let success = server.post_json_without_opportunity_confirmation(
        path,
        &json!({
            "schema": "ustc-agent-chat-request/v1",
            "messages": [{"role": "user", "content": "成绩单证明怎么办"}],
            "opportunity_context": null
        }),
    );
    assert!(success.status.contains(" 200 "), "{}", success.status);
    assert!(success.headers.contains("content-type: application/json"));
    assert!(success.headers.contains("cache-control: no-store"));
    let success: Value = serde_json::from_str(&success.body).expect("chat success JSON");
    assert_eq!(success["schema"], "ustc-agent-chat-response/v1");
    assert_eq!(success["provider"]["mode"], "mock");
    assert_eq!(success["provider"]["model"], "deterministic-mock-v1");
    assert_eq!(success["tool_trace"][0]["tool"], "affairs_navigator_get");
    assert_eq!(success["tool_trace"][0]["status"], "succeeded");
    assert!(
        success["answer"]
            .as_str()
            .is_some_and(|answer| answer.contains("transcript-certificate"))
    );

    let unrelated = server.post_json_without_opportunity_confirmation(
        path,
        &json!({
            "schema": "ustc-agent-chat-request/v1",
            "messages": [{"role": "user", "content": "student affairs office hours and exchange opportunities"}],
            "opportunity_context": null
        }),
    );
    assert!(unrelated.status.contains(" 200 "), "{}", unrelated.status);
    let unrelated: Value = serde_json::from_str(&unrelated.body).expect("unrelated chat JSON");
    assert_eq!(unrelated["tool_trace"], json!([]));
    assert!(
        unrelated["answer"]
            .as_str()
            .is_some_and(|answer| answer.contains("deterministic mock"))
    );

    let mixed_without_opportunity = server.post_json_without_opportunity_confirmation(
        path,
        &json!({
            "schema": "ustc-agent-chat-request/v1",
            "messages": [{"role": "user", "content": "请查成绩单并规划课程"}],
            "opportunity_context": null
        }),
    );
    assert!(
        mixed_without_opportunity.status.contains(" 200 "),
        "{}",
        mixed_without_opportunity.status
    );
    let mixed_without_opportunity: Value =
        serde_json::from_str(&mixed_without_opportunity.body).expect("mixed chat JSON");
    assert_eq!(
        mixed_without_opportunity["tool_trace"]
            .as_array()
            .map(Vec::len),
        Some(1)
    );
    assert_eq!(
        mixed_without_opportunity["tool_trace"][0]["tool"],
        "affairs_navigator_get"
    );
    assert_eq!(
        mixed_without_opportunity["tool_trace"][0]["status"],
        "succeeded"
    );
    assert!(
        mixed_without_opportunity["answer"]
            .as_str()
            .is_some_and(|answer| answer.contains("transcript-certificate")
                && answer.contains("课程规划请求未执行"))
    );

    let change = server.post_json_without_opportunity_confirmation(
        path,
        &json!({
            "schema": "ustc-agent-chat-request/v1",
            "messages": [{"role": "user", "content": "校历有什么变更"}],
            "opportunity_context": null
        }),
    );
    assert!(change.status.contains(" 200 "), "{}", change.status);
    let change: Value = serde_json::from_str(&change.body).expect("change chat JSON");
    assert_eq!(change["tool_trace"][0]["tool"], "change_radar_get");
    assert_eq!(change["tool_trace"][0]["status"], "succeeded");
    assert!(
        change["answer"]
            .as_str()
            .is_some_and(|answer| answer.contains("academic-calendar"))
    );

    let record = server.post_json_without_opportunity_confirmation(
        path,
        &json!({
            "schema": "ustc-agent-chat-request/v1",
            "messages": [{"role": "user", "content": "记录事项：提交开题报告"}],
            "opportunity_context": null
        }),
    );
    assert!(record.status.contains(" 200 "), "{}", record.status);
    let record: Value = serde_json::from_str(&record.body).expect("calendar record JSON");
    assert_eq!(record["tool_trace"][0]["tool"], "simple_calendar_items");
    assert_eq!(record["tool_trace"][0]["status"], "succeeded");
    assert!(record["answer"].as_str().is_some_and(|answer| {
        answer.contains("calendar:item:1") && answer.contains("提交开题报告")
    }));

    let list = server.post_json_without_opportunity_confirmation(
        path,
        &json!({
            "schema": "ustc-agent-chat-request/v1",
            "messages": [{"role": "user", "content": "列出我的待办事项"}],
            "opportunity_context": null
        }),
    );
    assert!(list.status.contains(" 200 "), "{}", list.status);
    let list: Value = serde_json::from_str(&list.body).expect("calendar list JSON");
    assert_eq!(list["tool_trace"][0]["status"], "succeeded");
    assert!(
        list["answer"]
            .as_str()
            .is_some_and(|answer| answer.contains("calendar:item:1"))
    );

    let delete = server.post_json_without_opportunity_confirmation(
        path,
        &json!({
            "schema": "ustc-agent-chat-request/v1",
            "messages": [{"role": "user", "content": "删除事项 calendar:item:1"}],
            "opportunity_context": null
        }),
    );
    assert!(delete.status.contains(" 200 "), "{}", delete.status);
    let delete: Value = serde_json::from_str(&delete.body).expect("calendar delete JSON");
    assert_eq!(delete["tool_trace"][0]["status"], "succeeded");
    assert!(
        delete["answer"]
            .as_str()
            .is_some_and(|answer| answer.contains("calendar:item:1"))
    );

    let created = server.post_json(
        "/api/v1/opportunity/profiles",
        &valid_opportunity_profile_body(),
    );
    assert!(created.status.contains(" 201 "), "{}", created.status);
    let created: Value = serde_json::from_str(&created.body).expect("profile JSON");
    let profile_id = created["terminal"]["profile"]["profile_snapshot_id"]
        .as_str()
        .expect("profile id");
    let opportunity = server.post_json(
        path,
        &json!({
            "schema": "ustc-agent-chat-request/v1",
            "messages": [{"role": "user", "content": "帮我规划课程"}],
            "opportunity_context": {"profile_snapshot_id": profile_id}
        }),
    );
    assert!(
        opportunity.status.contains(" 200 "),
        "{}",
        opportunity.status
    );
    let opportunity: Value =
        serde_json::from_str(&opportunity.body).expect("opportunity chat JSON");
    assert_eq!(
        opportunity["tool_trace"][0]["tool"],
        "opportunity_graph_plan_current_profile"
    );
    assert_eq!(opportunity["tool_trace"][0]["status"], "succeeded");
    assert!(
        opportunity["answer"]
            .as_str()
            .is_some_and(|answer| answer.contains("MATH2001"))
    );

    let denied = server.post_json(
        path,
        &json!({
            "schema": "ustc-agent-chat-request/v1",
            "messages": [{"role": "user", "content": "帮我规划课程"}],
            "opportunity_context": {"profile_snapshot_id": "profile:synthetic:missing"}
        }),
    );
    assert!(denied.status.contains(" 200 "), "{}", denied.status);
    let denied: Value = serde_json::from_str(&denied.body).expect("denied chat JSON");
    assert_eq!(denied["tool_trace"][0]["status"], "denied");
    assert!(
        denied["answer"]
            .as_str()
            .is_some_and(|answer| answer.contains("拒绝"))
    );

    let malformed = server.post_raw_with_confirmation(path, "application/json", "{", false);
    assert!(malformed.status.contains(" 400 "), "{}", malformed.status);
    let malformed: Value = serde_json::from_str(&malformed.body).expect("chat error JSON");
    assert_eq!(malformed["schema"], "ustc-agent-chat-error/v1");
    assert_eq!(malformed["error"], "invalid_chat_request");

    let structured_suffix_body = json!({
        "schema": "ustc-agent-chat-request/v1",
        "messages": [{"role": "user", "content": "成绩单证明怎么办"}],
        "opportunity_context": null
    })
    .to_string();
    let structured_suffix = server.post_raw_with_confirmation(
        path,
        "application/vnd.ustc-agent+json",
        &structured_suffix_body,
        false,
    );
    assert!(
        structured_suffix.status.contains(" 400 "),
        "{}",
        structured_suffix.status
    );
    let structured_suffix: Value =
        serde_json::from_str(&structured_suffix.body).expect("chat content-type error JSON");
    assert_eq!(structured_suffix["schema"], "ustc-agent-chat-error/v1");
    assert_eq!(structured_suffix["error"], "invalid_chat_request");

    let nul_content = server.post_json_without_opportunity_confirmation(
        path,
        &json!({
            "schema": "ustc-agent-chat-request/v1",
            "messages": [{"role": "user", "content": "bad\u{0000}content"}],
            "opportunity_context": null
        }),
    );
    assert!(
        nul_content.status.contains(" 400 "),
        "{}",
        nul_content.status
    );
    let nul_content: Value = serde_json::from_str(&nul_content.body).expect("NUL chat error JSON");
    assert_eq!(nul_content["error"], "invalid_chat_request");

    let missing_confirmation = server.post_json_without_opportunity_confirmation(
        path,
        &json!({
            "schema": "ustc-agent-chat-request/v1",
            "messages": [{"role": "user", "content": "帮我规划课程"}],
            "opportunity_context": {"profile_snapshot_id": "profile:synthetic:missing"}
        }),
    );
    assert!(
        missing_confirmation.status.contains(" 403 "),
        "{}",
        missing_confirmation.status
    );
    let missing_confirmation: Value =
        serde_json::from_str(&missing_confirmation.body).expect("chat confirmation error JSON");
    assert_eq!(
        missing_confirmation["error"],
        "opportunity_confirmation_required"
    );
}

#[test]
fn client_protocol_bootstrap_and_capability_registry_are_retained() {
    let server = WebServer::start_affairs_only();

    let info = server.get_without_protocol("/api/v1/server/info");
    assert!(info.status.contains(" 200 "), "{}", info.status);
    let info: Value = serde_json::from_str(&info.body).expect("server info JSON");
    assert_eq!(info["kind"], "server_info");
    assert_eq!(info["info"]["protocol_major"], 1);
    assert_eq!(info["info"]["supported_protocol_majors"], json!([1]));
    assert_eq!(
        info["info"]["capabilities_route"],
        "/api/v1/client/capabilities"
    );

    let capabilities = server.get("/api/v1/client/capabilities");
    assert!(
        capabilities.status.contains(" 200 "),
        "{}",
        capabilities.status
    );
    let capabilities: Value =
        serde_json::from_str(&capabilities.body).expect("capability registry JSON");
    assert_eq!(capabilities["kind"], "capabilities");
    let operations = capabilities["capabilities"]["operations"]
        .as_array()
        .expect("operation array");
    assert_eq!(operations.len(), 3);
    assert_eq!(operations[0]["operation_id"], "server.info");
    assert_eq!(operations[1]["operation_id"], "capability.list");
    assert_eq!(operations[2]["operation_id"], "affairs.get");
}

#[test]
fn version_gated_affairs_rejects_old_new_missing_malformed_and_repeated_majors() {
    const PATH: &str = "/api/v1/affairs/proc%3Austc%3Aundergraduate%3Atranscript-certificate";
    let server = WebServer::start_affairs_only();

    let old = server.get_with_protocol(PATH, "0");
    assert!(old.status.contains(" 426 "), "{}", old.status);
    assert!(old.body.contains("upgrade_required"));

    let newer = server.get_with_protocol(PATH, "2");
    assert!(newer.status.contains(" 409 "), "{}", newer.status);
    assert!(newer.body.contains("incompatible_protocol"));
    assert!(newer.body.contains("\"client_major\":2"));

    for response in [
        server.get_without_protocol(PATH),
        server.get_with_protocol(PATH, "not-a-major"),
        server.get_with_protocol(PATH, "65536"),
        server.get_with_protocol(PATH, "1.0"),
        server.get_with_protocol_headers(PATH, &["1", "1"]),
    ] {
        assert!(response.status.contains(" 409 "), "{}", response.status);
        assert!(response.body.contains("incompatible_protocol"));
        assert!(response.body.contains("\"client_major\":null"));
    }

    let invalid_procedure = "/api/v1/affairs/%0A";
    let gated = server.get_without_protocol(invalid_procedure);
    assert!(gated.status.contains(" 409 "), "{}", gated.status);
    let admitted = server.get(invalid_procedure);
    assert!(admitted.status.contains(" 400 "), "{}", admitted.status);
    assert!(admitted.body.contains("invalid_procedure_id"));
}

#[test]
fn affairs_as_of_query_reaches_existing_m71_cutoff_semantics() {
    let server = WebServer::start_affairs_only();
    let response =
        server.get("/api/v1/affairs/proc%3Austc%3Aundergraduate%3Atranscript-certificate?as_of=1");
    assert!(response.status.contains(" 200 "), "{}", response.status);
    let value: Value = serde_json::from_str(&response.body).expect("typed JSON response");
    assert_eq!(value["kind"], "available");
    assert_eq!(value["terminal"]["outcome"]["kind"], "not_yet_known");
    assert_eq!(value["terminal"]["outcome"]["as_of"], 1);
}

#[test]
fn reviewed_affairs_http_path_returns_typed_found_result() {
    let server = WebServer::start();
    let response =
        server.get("/api/v1/affairs/proc%3Austc%3Aundergraduate%3Atranscript-certificate");
    assert!(response.status.contains(" 200 "), "{}", response.status);
    assert!(
        response.headers.contains("content-type: application/json"),
        "{}",
        response.headers
    );
    assert!(response.headers.contains("cache-control: no-store"));
    assert!(response.headers.contains("x-content-type-options: nosniff"));

    let value: Value = serde_json::from_str(&response.body).expect("typed JSON response");
    assert_eq!(value["kind"], "available");
    assert_eq!(value["redaction"], "public");
    assert!(
        !response.body.contains("public_capability"),
        "response-only capability field must never cross the Web boundary"
    );
    assert!(!response.body.contains("capability_key_hex"));
    assert!(!response.body.contains("rev:ustc-teach"));
    assert!(!response.body.contains("de0cf446858717898f24aebc4b31a634"));
    assert_eq!(value["terminal"]["outcome"]["kind"], "found");
    assert_eq!(
        value["terminal"]["outcome"]["view"]["procedure_id"],
        "proc:ustc:undergraduate:transcript-certificate"
    );
    assert_eq!(
        value["terminal"]["outcome"]["view"]["title"],
        "在校生办理成绩单、成绩排名证明与在读证明"
    );
    assert_eq!(
        value["terminal"]["outcome"]["view"]["ordered_steps"]
            .as_array()
            .expect("steps array")
            .len(),
        4
    );
    assert_eq!(
        value["terminal"]["outcome"]["view"]["prerequisites"]
            .as_array()
            .expect("prerequisites array")
            .len(),
        2
    );
    assert!(value["terminal"]["outcome"]["view"]["effective_interval"].is_null());
    assert_eq!(
        value["terminal"]["outcome"]["view"]["deadlines"]
            .as_array()
            .expect("deadlines array")
            .len(),
        0
    );
    assert_eq!(
        value["terminal"]["outcome"]["view"]["contacts"]
            .as_array()
            .expect("contacts array")
            .len(),
        1
    );
    assert_eq!(
        value["terminal"]["outcome"]["view"]["evidence"]["assessments"][0]["source_id"],
        "src:ustc-teach:13824"
    );
    assert_eq!(value["terminal"]["outcome"]["freshness"]["kind"], "fresh");
    assert_eq!(value["terminal"]["lineage"]["kind"], "verified");
    assert_eq!(value["terminal"]["lineage"]["revision_count"], 1);
    assert!(
        value["terminal"]["lineage"]["evidence_set_digest"]
            .as_str()
            .is_some_and(|value| value.starts_with("sha256:"))
    );
    assert!(
        value["terminal"]["lineage"]["materialization_receipt_id"]
            .as_str()
            .is_some_and(|value| !value.is_empty())
    );
}

#[test]
fn administrator_publication_http_requires_explicit_demo_confirmation_and_is_idempotent() {
    const PATH: &str = "/api/v1/demo/administrator/affairs/publication";
    let server = WebServer::start_affairs_only();

    let denied = server.get(PATH);
    assert!(denied.status.contains(" 403 "), "{}", denied.status);
    assert!(
        denied
            .body
            .contains("administrator_demo_confirmation_required")
    );

    let initial = server.get_admin(PATH);
    assert!(initial.status.contains(" 200 "), "{}", initial.status);
    let initial: Value = serde_json::from_str(&initial.body).expect("publication status JSON");
    assert_eq!(initial["schema"], "ustc-affairs-publication-status/v1");
    assert_eq!(initial["publication_revision"], 1);
    assert_eq!(initial["control_evidence_event_count"], 0);

    let unconfirmed = server.post_admin_json(PATH, &json!({"confirm_publish": false}));
    assert!(
        unconfirmed.status.contains(" 400 "),
        "{}",
        unconfirmed.status
    );
    assert!(
        unconfirmed
            .body
            .contains("explicit_publish_confirmation_required")
    );

    let published = server.post_admin_json(PATH, &json!({"confirm_publish": true}));
    assert!(
        published.status.contains(" 200 "),
        "{}: {}",
        published.status,
        published.body
    );
    let published: Value =
        serde_json::from_str(&published.body).expect("publication response JSON");
    assert_eq!(published["schema"], "ustc-affairs-publication-response/v1");
    assert_eq!(published["outcome"]["kind"], "published");
    assert_eq!(published["outcome"]["expected_publication_revision"], 1);
    assert_eq!(published["outcome"]["publication_revision"], 2);
    let receipt = published["outcome"]["receipt_id"]
        .as_str()
        .expect("publication receipt")
        .to_owned();

    let replay = server.post_admin_json(PATH, &json!({"confirm_publish": true}));
    assert!(replay.status.contains(" 200 "), "{}", replay.status);
    let replay: Value = serde_json::from_str(&replay.body).expect("replay response JSON");
    assert_eq!(replay["outcome"]["receipt_id"], receipt);
    assert_eq!(replay["outcome"]["publication_revision"], 2);

    let recovered = server.get_admin(PATH);
    let recovered: Value =
        serde_json::from_str(&recovered.body).expect("recovered publication status JSON");
    assert_eq!(recovered["publication_revision"], 2);
    assert_eq!(recovered["publication_receipt_id"], receipt);
    assert_eq!(recovered["control_evidence_event_count"], 1);

    let publication_state = server.temp_dir.join("idempotency.affairs-publication.json");
    let metadata = fs::metadata(publication_state).expect("durable publication state");
    assert_eq!(metadata.permissions().mode() & 0o777, 0o600);
}

#[test]
fn affairs_only_web_mode_keeps_affairs_available_and_change_fail_closed() {
    let server = WebServer::start_affairs_only();

    let affairs =
        server.get("/api/v1/affairs/proc%3Austc%3Aundergraduate%3Atranscript-certificate");
    assert!(affairs.status.contains(" 200 "), "{}", affairs.status);
    let affairs_value: Value = serde_json::from_str(&affairs.body).expect("affairs JSON");
    assert_eq!(affairs_value["terminal"]["outcome"]["kind"], "found");

    let change = server.get("/api/v1/changes/board%3Austc%3Aacademic-calendar");
    assert!(change.status.contains(" 503 "), "{}", change.status);
    let change_value: Value = serde_json::from_str(&change.body).expect("change JSON");
    assert_eq!(change_value["kind"], "unavailable");

    let atom = server.get("/api/v1/changes/board%3Austc%3Aacademic-calendar/atom");
    assert!(atom.status.contains(" 503 "), "{}", atom.status);
    let atom_value: Value = serde_json::from_str(&atom.body).expect("atom error JSON");
    assert_eq!(atom_value["error"], "change_feed_unavailable");

    let opportunity = server.get("/api/v1/opportunity/profiles/profile-snapshot%3Aopportunity%3Asha256%3Aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");
    assert!(
        opportunity.status.contains(" 503 "),
        "{}",
        opportunity.status
    );
    let opportunity_value: Value =
        serde_json::from_str(&opportunity.body).expect("opportunity error JSON");
    assert_eq!(opportunity_value["kind"], "unavailable");
}

#[test]
fn every_plugin_disable_and_revoke_fails_closed_without_harming_peers() {
    for (label, affairs_state, change_state, opportunity_state, blocked) in [
        (
            "affairs-disabled",
            MarketState::Disabled,
            MarketState::Active,
            MarketState::Active,
            PluginTarget::Affairs,
        ),
        (
            "affairs-revoked",
            MarketState::Revoked,
            MarketState::Active,
            MarketState::Active,
            PluginTarget::Affairs,
        ),
        (
            "change-disabled",
            MarketState::Active,
            MarketState::Disabled,
            MarketState::Active,
            PluginTarget::Change,
        ),
        (
            "change-revoked",
            MarketState::Active,
            MarketState::Revoked,
            MarketState::Active,
            PluginTarget::Change,
        ),
        (
            "opportunity-disabled",
            MarketState::Active,
            MarketState::Active,
            MarketState::Disabled,
            PluginTarget::Opportunity,
        ),
        (
            "opportunity-revoked",
            MarketState::Active,
            MarketState::Active,
            MarketState::Revoked,
            PluginTarget::Opportunity,
        ),
    ] {
        let server =
            WebServer::start_with_market_states(affairs_state, change_state, opportunity_state);
        let affairs =
            server.get("/api/v1/affairs/proc%3Austc%3Aundergraduate%3Atranscript-certificate");
        let change = server.get("/api/v1/changes/board%3Austc%3Aacademic-calendar");
        let opportunity = server.post_json(
            "/api/v1/opportunity/profiles",
            &valid_opportunity_profile_body(),
        );

        for (target, response, success_status) in [
            (PluginTarget::Affairs, &affairs, " 200 "),
            (PluginTarget::Change, &change, " 200 "),
            (PluginTarget::Opportunity, &opportunity, " 201 "),
        ] {
            let is_blocked = matches!(
                (blocked, target),
                (PluginTarget::Affairs, PluginTarget::Affairs)
                    | (PluginTarget::Change, PluginTarget::Change)
                    | (PluginTarget::Opportunity, PluginTarget::Opportunity)
            );
            if is_blocked {
                assert!(
                    response.status.contains(" 403 "),
                    "{label}: blocked plugin returned {}: {}",
                    response.status,
                    response.body
                );
                assert!(
                    response.body.contains("policy_denied"),
                    "{label}: denial was not typed: {}",
                    response.body
                );
            } else {
                assert!(
                    response.status.contains(success_status),
                    "{label}: peer plugin returned {}: {}",
                    response.status,
                    response.body
                );
            }
        }

        if matches!(blocked, PluginTarget::Opportunity) {
            let profile_id = "profile-snapshot%3Aopportunity%3Asha256%3Aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
            let view = server.get(&format!("/api/v1/opportunity/profiles/{profile_id}"));
            let plan = server.post_json(
                "/api/v1/opportunity/plans",
                &json!({
                    "profile_snapshot_id": "profile-snapshot:opportunity:sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                    "max_results": 3,
                    "beam_width": 1024,
                }),
            );
            for response in [&view, &plan] {
                assert!(
                    response.status.contains(" 403 "),
                    "{label}: denied Opportunity operation returned {}: {}",
                    response.status,
                    response.body
                );
                assert!(response.body.contains("policy_denied"));
            }

            let state_path = server.temp_dir.join("opportunity-profiles.json");
            if state_path.exists() {
                let state: Value = serde_json::from_slice(
                    &fs::read(&state_path).expect("read denied Opportunity state"),
                )
                .expect("decode denied Opportunity state");
                assert_eq!(
                    state["active"].as_array().map_or(0, Vec::len),
                    0,
                    "{label}: denied create persisted active private payload"
                );
                assert_eq!(
                    state["tombstones"].as_array().map_or(0, Vec::len),
                    0,
                    "{label}: denied create persisted a tombstone"
                );
            }
        }
    }
}

#[test]
fn unknown_affairs_http_path_returns_public_not_found_without_bearer() {
    let server = WebServer::start();
    let response = server.get("/api/v1/affairs/proc%3Austc%3Astudent%3Aunknown");
    assert!(response.status.contains(" 200 "), "{}", response.status);

    let value: Value = serde_json::from_str(&response.body).expect("typed JSON response");
    assert_eq!(value["kind"], "available");
    assert_eq!(value["redaction"], "public");
    assert_eq!(value["terminal"]["outcome"]["kind"], "not_found");
    assert_eq!(
        value["terminal"]["outcome"]["procedure_id"],
        "proc:ustc:student:unknown"
    );
    assert_eq!(value["terminal"]["lineage"]["kind"], "not_required");
    assert!(!response.body.contains("public_capability"));
    assert!(!response.body.contains("cap:fixture-public"));
}

#[test]
fn reviewed_change_radar_http_and_atom_paths_are_source_grounded() {
    const PUBLICATION_PATH: &str = "/api/v1/demo/administrator/changes/publication";
    let server = WebServer::start();
    let initial = server.get_admin(PUBLICATION_PATH);
    assert!(initial.status.contains(" 200 "), "{}", initial.status);
    let initial: Value =
        serde_json::from_str(&initial.body).expect("ChangeRadar publication status JSON");
    assert_eq!(initial["schema"], "ustc-change-publication-status/v1");
    assert_eq!(initial["review_count"], 0);
    assert_eq!(initial["publication_count"], 0);
    assert!(initial["publication_receipt_id"].is_null());

    let before = server.get("/api/v1/changes/board%3Austc%3Aacademic-calendar");
    let before: Value = serde_json::from_str(&before.body).expect("empty change JSON");
    assert_eq!(before["terminal"]["outcome"]["kind"], "found");
    assert_eq!(before["terminal"]["outcome"]["view"]["entries"], json!([]));
    assert!(
        before["terminal"]["outcome"]["view"]["atom"]
            .as_str()
            .is_some_and(|atom| !atom.contains("<entry>"))
    );

    let published = server.post_admin_json(PUBLICATION_PATH, &json!({"confirm_publish": true}));
    assert!(
        published.status.contains(" 200 "),
        "{}: {}",
        published.status,
        published.body
    );
    let published: Value =
        serde_json::from_str(&published.body).expect("ChangeRadar publication response JSON");
    assert_eq!(published["schema"], "ustc-change-publication-response/v1");
    assert_eq!(published["outcome"]["kind"], "published");

    let response = server.get("/api/v1/changes/board%3Austc%3Aacademic-calendar");
    assert!(response.status.contains(" 200 "), "{}", response.status);
    assert!(response.headers.contains("content-type: application/json"));
    assert!(response.headers.contains("cache-control: no-store"));
    let value: Value = serde_json::from_str(&response.body).expect("change JSON response");
    assert_eq!(value["kind"], "change_feed_accepted");
    assert_eq!(value["terminal"]["outcome"]["kind"], "found");
    assert_eq!(
        value["terminal"]["outcome"]["view"]["board_id"],
        "board:ustc:academic-calendar"
    );
    let entry = &value["terminal"]["outcome"]["view"]["entries"][0];
    assert_eq!(entry["source_health"], "current");
    assert_eq!(entry["source_id"], "src:ustc:academic-calendar:2026-fall");
    assert_eq!(
        entry["changed_fields"]
            .as_array()
            .expect("changed fields")
            .len(),
        2
    );
    assert!(entry["old_raw_sha256"].as_str().is_some());
    assert!(entry["new_raw_sha256"].as_str().is_some());
    assert!(entry["old_normalized_sha256"].as_str().is_some());
    assert!(entry["new_normalized_sha256"].as_str().is_some());
    assert!(entry["effective_from"].as_i64().is_some());
    assert!(entry["effective_to"].as_i64().is_some());
    assert!(entry["observed_at"].as_i64().is_some());
    assert!(entry["published_at"].as_i64().is_some());
    assert_eq!(entry["old_source_reviewer"], "reviewer:demo:change-source");
    assert_eq!(entry["new_source_reviewer"], "reviewer:demo:change-source");
    assert_eq!(
        entry["old_source_review_evidence"],
        "evidence:demo:change:r1"
    );
    assert_eq!(
        entry["new_source_review_evidence"],
        "evidence:demo:change:r2"
    );
    assert!(entry["evidence_set_digest"].as_str().is_some());
    assert!(!response.body.contains("public_capability"));

    let atom = server.get("/api/v1/changes/board%3Austc%3Aacademic-calendar/atom");
    assert!(atom.status.contains(" 200 "), "{}", atom.status);
    assert!(
        atom.headers
            .contains("content-type: application/atom+xml; charset=utf-8")
    );
    assert!(
        atom.body
            .contains("<feed xmlns=\"http://www.w3.org/2005/Atom\">")
    );
    assert!(atom.body.contains("<author>"));
    assert!(atom.body.contains("registration.deadline"));
    assert!(atom.body.contains("old_raw_sha256="));
}

#[test]
fn unknown_change_board_has_stable_json_and_atom_results() {
    let server = WebServer::start();
    let response = server.get("/api/v1/changes/board%3Austc%3Aunknown");
    assert!(response.status.contains(" 200 "), "{}", response.status);
    let value: Value = serde_json::from_str(&response.body).expect("change JSON response");
    assert_eq!(value["terminal"]["outcome"]["kind"], "not_found");
    assert_eq!(
        value["terminal"]["outcome"]["board_id"],
        "board:ustc:unknown"
    );

    let atom = server.get("/api/v1/changes/board%3Austc%3Aunknown/atom");
    assert!(atom.status.contains(" 404 "), "{}", atom.status);
    let value: Value = serde_json::from_str(&atom.body).expect("Atom error JSON");
    assert_eq!(value["error"], "change_board_not_found");
}

#[test]
fn opportunity_http_journey_requires_consent_plans_and_deletes_private_payload() {
    let server = WebServer::start();
    let profile_body = valid_opportunity_profile_body();

    for response in [
        server.post_raw(
            "/api/v1/opportunity/profiles",
            "application/json",
            "{ not valid json",
        ),
        server.post_json(
            "/api/v1/opportunity/plans",
            &json!({"max_results": 3, "beam_width": 1024}),
        ),
        server.post_raw(
            "/api/v1/opportunity/profiles/profile%3Afixture/revoke-delete",
            "text/plain",
            r#"{"confirm_delete":true}"#,
        ),
    ] {
        assert!(response.status.contains(" 400 "), "{}", response.status);
        let value: Value =
            serde_json::from_str(&response.body).expect("malformed Opportunity error JSON");
        assert_eq!(value["error"], "invalid_opportunity_json");
    }

    let denied = server.post_json(
        "/api/v1/opportunity/profiles",
        &json!({
            "consent": false,
            "request_id": "req:web:journey-denied",
            "correlation_id": "corr:web:journey-denied",
            "idempotency_key": "idem:web:journey-denied",
            "consented_at": 1_787_792_400_000i64,
            "completed_courses": ["MATH101"],
            "min_credits": 6,
            "max_credits": 8,
            "preference_weights": []
        }),
    );
    assert!(denied.status.contains(" 400 "), "{}", denied.status);
    let denied_value: Value = serde_json::from_str(&denied.body).expect("consent error JSON");
    assert_eq!(denied_value["error"], "explicit_consent_required");

    let created = server.post_json("/api/v1/opportunity/profiles", &profile_body);
    assert!(created.status.contains(" 201 "), "{}", created.status);
    assert!(created.headers.contains("cache-control: no-store"));
    let created_value: Value = serde_json::from_str(&created.body).expect("create JSON");
    assert_eq!(created_value["kind"], "opportunity_accepted");
    assert_eq!(created_value["terminal"]["kind"], "profile_created");
    let profile_id = created_value["terminal"]["profile"]["profile_snapshot_id"]
        .as_str()
        .expect("profile id")
        .to_owned();
    assert_eq!(
        created_value["terminal"]["profile"]["completed_course_count"],
        4
    );
    assert_eq!(created_value["terminal"]["profile"]["preference_count"], 7);
    assert!(!created.body.contains("MATH1001"));
    assert!(!created.body.contains("\"weight\":"));

    let encoded_profile = profile_id.replace(':', "%3A");
    let viewed = server.get(&format!("/api/v1/opportunity/profiles/{encoded_profile}"));
    assert!(viewed.status.contains(" 200 "), "{}", viewed.status);
    let viewed_value: Value = serde_json::from_str(&viewed.body).expect("view JSON");
    assert_eq!(viewed_value["terminal"]["kind"], "profile_found");

    let planned = server.post_json(
        "/api/v1/opportunity/plans",
        &json!({
            "profile_snapshot_id": profile_id,
            "max_results": 3,
            "beam_width": 1024
        }),
    );
    assert!(planned.status.contains(" 200 "), "{}", planned.status);
    let planned_value: Value = serde_json::from_str(&planned.body).expect("plan JSON");
    assert_eq!(planned_value["terminal"]["kind"], "plan_generated");
    assert_eq!(
        planned_value["terminal"]["plan"]["decision"]["kind"],
        "planned"
    );
    assert_eq!(
        planned_value["terminal"]["plan"]["decision"]["hard_constraint_violations"],
        0
    );
    assert!(
        !planned_value["terminal"]["plan"]["decision"]["candidates"]
            .as_array()
            .expect("candidates")
            .is_empty()
    );
    assert!(
        planned_value["terminal"]["plan"]["source_revision_id"]
            .as_str()
            .is_some_and(|value| value.starts_with("revision:sha256:"))
    );
    assert!(
        !planned_value["terminal"]["plan"]["qualifications"]
            .as_array()
            .expect("qualifications")
            .is_empty()
    );
    assert!(planned.body.contains("source_revision_id"));
    assert!(planned.body.contains("conflict_status"));

    let deleted = server.post_json(
        &format!("/api/v1/opportunity/profiles/{encoded_profile}/revoke-delete"),
        &json!({
            "confirm_delete": true,
            "request_id": "req:web:journey-delete",
            "correlation_id": "corr:web:journey-delete",
            "idempotency_key": "idem:web:journey-delete",
            "revoked_at": 1_787_792_500_000i64
        }),
    );
    assert!(deleted.status.contains(" 200 "), "{}", deleted.status);
    let deleted_value: Value = serde_json::from_str(&deleted.body).expect("delete JSON");
    assert_eq!(deleted_value["terminal"]["kind"], "profile_deleted");
    assert!(!deleted.body.contains("MATH1001"));
    assert!(!deleted.body.contains("\"weight\":"));

    let after_delete = server.post_json(
        "/api/v1/opportunity/plans",
        &json!({
            "profile_snapshot_id": profile_id,
            "max_results": 3,
            "beam_width": 1024
        }),
    );
    assert!(
        after_delete.status.contains(" 410 "),
        "{}",
        after_delete.status
    );
    let after_delete_value: Value =
        serde_json::from_str(&after_delete.body).expect("deleted-plan JSON");
    assert_eq!(after_delete_value["kind"], "opportunity_rejected");
    assert_eq!(after_delete_value["rejection"]["kind"], "profile_deleted");
}

#[test]
fn opportunity_http_missing_confirmation_fails_closed() {
    let server = WebServer::start();
    let response = server.post_json_without_opportunity_confirmation(
        "/api/v1/opportunity/profiles",
        &valid_opportunity_profile_body(),
    );
    assert!(response.status.contains(" 403 "), "{}", response.status);
    let body: Value = serde_json::from_str(&response.body).expect("typed confirmation denial");
    assert_eq!(body["kind"], "error");
    assert_eq!(body["error"]["error"]["class"], "policy_denied");
}

#[test]
fn create_retry_with_byte_identical_body_recovers_same_profile_receipt() {
    let server = WebServer::start();
    let body = valid_opportunity_profile_body().to_string();

    let first = server.post_raw("/api/v1/opportunity/profiles", "application/json", &body);
    assert!(
        first.status.contains(" 201 "),
        "{}: {}",
        first.status,
        first.body
    );
    let first_value: Value = serde_json::from_str(&first.body).expect("create JSON");
    assert_eq!(first_value["kind"], "opportunity_accepted");
    assert_eq!(first_value["terminal"]["kind"], "profile_created");
    let profile_id = first_value["terminal"]["profile"]["profile_snapshot_id"]
        .as_str()
        .expect("profile id")
        .to_owned();
    let consent_id = first_value["terminal"]["profile"]["consent_id"]
        .as_str()
        .expect("consent id")
        .to_owned();

    // The first response is discarded; the exact same body is resent.
    let retry = server.post_raw("/api/v1/opportunity/profiles", "application/json", &body);
    assert!(
        retry.status.contains(" 201 "),
        "byte-identical create retry must recover the committed terminal, not ProfileAlreadyExists: {}: {}",
        retry.status,
        retry.body
    );
    let retry_value: Value = serde_json::from_str(&retry.body).expect("retry create JSON");
    assert_eq!(retry_value["kind"], "opportunity_accepted");
    assert_eq!(retry_value["terminal"]["kind"], "profile_created");
    assert_eq!(
        retry_value["terminal"]["profile"]["profile_snapshot_id"].as_str(),
        Some(profile_id.as_str())
    );
    assert_eq!(
        retry_value["terminal"]["profile"]["consent_id"].as_str(),
        Some(consent_id.as_str())
    );
}

#[test]
fn delete_retry_with_byte_identical_body_recovers_same_deletion_receipt() {
    let server = WebServer::start();
    let created = server.post_json(
        "/api/v1/opportunity/profiles",
        &valid_opportunity_profile_body(),
    );
    assert!(created.status.contains(" 201 "), "{}", created.status);
    let created_value: Value = serde_json::from_str(&created.body).expect("create JSON");
    let profile_id = created_value["terminal"]["profile"]["profile_snapshot_id"]
        .as_str()
        .expect("profile id")
        .to_owned();
    let encoded_profile = profile_id.replace(':', "%3A");

    let delete_body = json!({
        "confirm_delete": true,
        "request_id": "req:web:opportunity-delete-regression",
        "correlation_id": "corr:web:opportunity-delete-regression",
        "idempotency_key": "idem:web:opportunity-delete-regression",
        "revoked_at": 1_787_792_500_000i64
    })
    .to_string();
    let path = format!("/api/v1/opportunity/profiles/{encoded_profile}/revoke-delete");

    let deleted = server.post_raw(&path, "application/json", &delete_body);
    assert!(
        deleted.status.contains(" 200 "),
        "{}: {}",
        deleted.status,
        deleted.body
    );
    let deleted_value: Value = serde_json::from_str(&deleted.body).expect("delete JSON");
    assert_eq!(deleted_value["terminal"]["kind"], "profile_deleted");
    let receipt = deleted_value["terminal"]["deletion"]["deletion_receipt_id"]
        .as_str()
        .expect("deletion receipt")
        .to_owned();

    // The first response is discarded; the exact same body is resent. It must
    // recover the committed terminal instead of conflicting with the tombstone.
    let replay = server.post_raw(&path, "application/json", &delete_body);
    assert!(
        replay.status.contains(" 200 "),
        "byte-identical delete retry must recover the committed deletion, not a tombstone conflict: {}: {}",
        replay.status,
        replay.body
    );
    let replay_value: Value = serde_json::from_str(&replay.body).expect("replay delete JSON");
    assert_eq!(replay_value["terminal"]["kind"], "profile_deleted");
    assert_eq!(
        replay_value["terminal"]["deletion"]["deletion_receipt_id"].as_str(),
        Some(receipt.as_str())
    );
}

#[test]
fn opportunity_identity_and_timestamp_malformed_bodies_are_stable_input_errors() {
    let server = WebServer::start();

    let mut empty_request_id = valid_opportunity_profile_body();
    empty_request_id["request_id"] = json!("");
    let response = server.post_json("/api/v1/opportunity/profiles", &empty_request_id);
    assert!(response.status.contains(" 400 "), "{}", response.status);
    let value: Value = serde_json::from_str(&response.body).expect("error JSON");
    assert_eq!(value["error"], "invalid_opportunity_identity");

    let mut control_character_key = valid_opportunity_profile_body();
    control_character_key["idempotency_key"] = json!("idem:web:bad\u{0007}token");
    let response = server.post_json("/api/v1/opportunity/profiles", &control_character_key);
    assert!(response.status.contains(" 400 "), "{}", response.status);
    let value: Value = serde_json::from_str(&response.body).expect("error JSON");
    assert_eq!(value["error"], "invalid_opportunity_identity");

    let mut negative_timestamp = valid_opportunity_profile_body();
    negative_timestamp["consented_at"] = json!(-1);
    let response = server.post_json("/api/v1/opportunity/profiles", &negative_timestamp);
    assert!(response.status.contains(" 400 "), "{}", response.status);
    let value: Value = serde_json::from_str(&response.body).expect("error JSON");
    assert_eq!(value["error"], "invalid_opportunity_identity");

    let mut missing_identity = valid_opportunity_profile_body();
    missing_identity
        .as_object_mut()
        .expect("create body object")
        .remove("correlation_id");
    let response = server.post_json("/api/v1/opportunity/profiles", &missing_identity);
    assert!(response.status.contains(" 400 "), "{}", response.status);
    let value: Value = serde_json::from_str(&response.body).expect("error JSON");
    assert_eq!(value["error"], "invalid_opportunity_json");

    let zero_timestamp_delete = json!({
        "confirm_delete": true,
        "request_id": "req:web:opportunity-delete-malformed",
        "correlation_id": "corr:web:opportunity-delete-malformed",
        "idempotency_key": "idem:web:opportunity-delete-malformed",
        "revoked_at": 0
    });
    let response = server.post_json(
        "/api/v1/opportunity/profiles/profile%3Afixture/revoke-delete",
        &zero_timestamp_delete,
    );
    assert!(response.status.contains(" 400 "), "{}", response.status);
    let value: Value = serde_json::from_str(&response.body).expect("error JSON");
    assert_eq!(value["error"], "invalid_opportunity_identity");
}

#[test]
fn retained_source_fixture_hashes_match_declared_evidence() {
    let workspace = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|path| path.parent())
        .expect("workspace root")
        .to_path_buf();
    let fixture_path = workspace.join("fixtures/affairs/proc-011-reviewed.json");
    let fixture: Value =
        serde_json::from_slice(&fs::read(&fixture_path).expect("read source-grounded fixture"))
            .expect("parse source-grounded fixture");

    for (relative, field) in [
        (
            "fixtures/affairs/evidence/ustc-teach-13824.reviewed.txt",
            "raw_digest",
        ),
        (
            "fixtures/affairs/evidence/ustc-teach-13824.normalized.json",
            "normalized_digest",
        ),
    ] {
        let bytes = fs::read(workspace.join(relative)).expect("read retained source evidence");
        let actual = format!("sha256:{:x}", Sha256::digest(bytes));
        assert_eq!(fixture[field], actual, "digest drift for {relative}");
    }
}

#[test]
fn embedded_web_shell_and_health_are_hardened() {
    let server = WebServer::start();
    let page = server.get("/");
    assert!(page.status.contains(" 200 "), "{}", page.status);
    assert!(page.headers.contains("content-type: text/html"));
    assert!(page.headers.contains("content-security-policy:"));
    assert!(page.headers.contains("x-frame-options: deny"));
    assert!(page.body.contains("USTC Campus Agent"));
    assert!(!page.body.contains("科大校园助手"));
    assert!(page.body.contains("先说你要做什么。"));
    for id in [
        "chat-form",
        "chat-input",
        "chat-clear",
        "chat-send",
        "chat-messages",
        "chat-opportunity-confirm",
    ] {
        assert!(page.body.contains(id), "missing bounded chat element {id}");
    }
    assert!(page.body.contains("/assets/app.js"));
    assert!(page.body.contains("办理条件"));
    assert!(page.body.contains("时间边界"));
    assert!(page.body.contains("证据集摘要"));
    assert!(page.body.contains("管理员发布 · 非生产演示"));
    assert!(page.body.contains("radar-publication-confirm"));
    assert!(page.body.contains("procedure-id-preview"));
    assert!(page.body.contains("CHANGE RADAR"));
    assert!(page.body.contains("radar-fields"));
    assert!(page.body.contains("Atom feed"));
    assert!(page.body.contains("OPPORTUNITY GRAPH"));
    assert!(page.body.contains("opportunity-consent"));
    assert!(page.body.contains("opportunity-create"));
    assert!(page.body.contains("opportunity-plan"));
    assert!(page.body.contains("opportunity-delete"));
    for id in [
        "radar-effective",
        "radar-published",
        "radar-old-raw-digest",
        "radar-old-normalized-digest",
        "radar-old-review",
        "radar-new-raw-digest",
        "radar-new-normalized-digest",
        "radar-new-review",
    ] {
        assert!(
            page.body.contains(id),
            "missing browser evidence field {id}"
        );
    }

    let script = server.get("/assets/app.js");
    assert!(script.status.contains(" 200 "), "{}", script.status);
    assert!(script.headers.contains("content-type: text/javascript"));
    assert!(!script.body.contains("innerHTML"));
    assert!(script.body.contains("ustc-change-publication-status/v1"));
    assert!(script.body.contains("publishChangeDemo"));
    assert!(script.body.contains("textContent"));
    assert!(script.body.contains("X-USTC-Client-Protocol-Major"));
    assert!(script.body.contains("syncProcedurePreview"));
    assert!(script.body.contains("createChatRequest"));
    assert!(script.body.contains("submitChat"));
    assert!(script.body.contains("/api/v1/agent/chat"));
    assert!(script.body.contains("X-USTC-Opportunity-Confirmation"));
    assert!(script.body.contains("renderChangeFeed"));
    assert!(script.body.contains("loadChangeFeed"));
    assert!(script.body.contains("createOpportunityProfile"));
    assert!(script.body.contains("renderOpportunityPlan"));
    assert!(script.body.contains("deleteOpportunityProfile"));
    assert!(script.body.contains("submitOpportunityOperation"));
    assert!(script.body.contains("opportunity-pending-create/v1"));
    assert!(script.body.contains("opportunity-pending-delete/v1"));
    assert!(script.body.contains("opportunityPendingMemory = new Map()"));
    assert!(
        script
            .body
            .contains("clearPendingOperation(\"create\", envelope)")
    );
    assert!(
        script
            .body
            .contains("clearPendingOperation(\"delete\", envelope)")
    );
    assert!(!script.body.contains("opportunity-pending-operation/v1"));
    assert!(script.body.contains("mintBoundedId"));

    let health = server.get("/healthz");
    assert!(health.status.contains(" 200 "), "{}", health.status);
    let value: Value = serde_json::from_str(&health.body).expect("health JSON");
    assert_eq!(value["schema"], "ustc-agentd-health/v1");
    assert_eq!(value["status"], "ok");
}
