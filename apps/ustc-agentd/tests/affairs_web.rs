#![allow(clippy::unwrap_used)]

//! Real loopback HTTP/Web smoke for the bounded reviewed-procedure slice.

use std::fs;
use std::io::{BufRead, Read, Write};
use std::net::TcpStream;
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

impl WebServer {
    fn start() -> Self {
        Self::start_with_plugins(true, true)
    }

    fn start_affairs_only() -> Self {
        Self::start_with_plugins(false, false)
    }

    fn start_with_plugins(include_change: bool, include_opportunity: bool) -> Self {
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
        let store = temp_dir.join("records.json");
        let idempotency = temp_dir.join("idempotency.json");
        let opportunity_profile_store = temp_dir.join("opportunity-profiles.json");

        let mut command = Command::new(env!("CARGO_BIN_EXE_ustc-agentd"));
        command.args([
            "serve-web",
            "--bind",
            "127.0.0.1:0",
            "--fixture",
            fixture.to_str().expect("fixture path utf8"),
        ]);
        if include_change {
            command.args([
                "--change-fixture",
                change_fixture.to_str().expect("change fixture path utf8"),
            ]);
        }
        if include_opportunity {
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
        let endpoint = endpoint.expect("web server did not publish endpoint");
        Self {
            child,
            endpoint,
            temp_dir,
        }
    }

    fn get(&self, path: &str) -> HttpResponse {
        let mut stream = TcpStream::connect(&self.endpoint).expect("connect web server");
        stream
            .set_read_timeout(Some(Duration::from_secs(10)))
            .expect("set read timeout");
        write!(
            stream,
            "GET {path} HTTP/1.1\r\nHost: {}\r\nAccept: application/json,text/html,*/*\r\nConnection: close\r\n\r\n",
            self.endpoint
        )
        .expect("write HTTP request");
        stream.flush().expect("flush HTTP request");
        let mut bytes = Vec::new();
        stream.read_to_end(&mut bytes).expect("read HTTP response");
        HttpResponse::parse(&bytes)
    }

    fn post_json(&self, path: &str, body: &Value) -> HttpResponse {
        let body = body.to_string();
        let mut stream = TcpStream::connect(&self.endpoint).expect("connect web server");
        stream
            .set_read_timeout(Some(Duration::from_secs(10)))
            .expect("set read timeout");
        write!(
            stream,
            "POST {path} HTTP/1.1\r\nHost: {}\r\nAccept: application/json\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
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
    let server = WebServer::start();
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
    let profile_body = json!({
        "consent": true,
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
    });

    let denied = server.post_json(
        "/api/v1/opportunity/profiles",
        &json!({
            "consent": false,
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
        &json!({"confirm_delete": true}),
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
    assert!(page.body.contains("科大办事导航"));
    assert!(page.body.contains("/assets/app.js"));
    assert!(page.body.contains("办理条件"));
    assert!(page.body.contains("时间边界"));
    assert!(page.body.contains("证据集摘要"));
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
    assert!(script.body.contains("textContent"));
    assert!(script.body.contains("syncProcedurePreview"));
    assert!(script.body.contains("renderChangeFeed"));
    assert!(script.body.contains("loadChangeFeed"));
    assert!(script.body.contains("createOpportunityProfile"));
    assert!(script.body.contains("renderOpportunityPlan"));
    assert!(script.body.contains("deleteOpportunityProfile"));

    let health = server.get("/healthz");
    assert!(health.status.contains(" 200 "), "{}", health.status);
    let value: Value = serde_json::from_str(&health.body).expect("health JSON");
    assert_eq!(value["schema"], "ustc-agentd-health/v1");
    assert_eq!(value["status"], "ok");
}
