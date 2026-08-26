#![allow(clippy::unwrap_used)]

//! End-to-end product-path tests for `ustc-agent` ↔ `ustc-agentd`.
//!
//! These tests spawn a real `ustc-agentd` server on `127.0.0.1:0` and invoke
//! the real `ustc-agent` CLI binary as a subprocess. They prove the full wire
//! path: M80 CLI → loopback TCP → M10 ingress → M71 affairs-navigator → M60
//! fixture → durable record store → M80 reducer → canonical JSON envelope.
//!
//! The wire-only dependency guard test verifies at compile time that
//! `ustc-agent`'s `Cargo.toml` contains no server-side or operator-side
//! dependencies.

use std::fs;
use std::io::{BufRead, Write};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc;
use std::time::{Duration, Instant};

use serde_json::Value;

use ustc_campus_agent_client_core::wire::{WireText, affairs_get_payload_digest};

// ---------------------------------------------------------------------------
// Static fixture (mirrors composition test)
// ---------------------------------------------------------------------------

fn base_fixture() -> Value {
    serde_json::json!({
        "procedure_id": "proc:fixture",
        "artifact_id": "artifact:fixture:v1",
        "title": "Fixture procedure",
        "known_at_secs": 50,
        "last_verified_at_secs": 150,
        "max_fresh_seconds": 100,
        "max_presentable_seconds": 200,
        "source_id": "src:fixture",
        "revision_id": "rev:fixture:0",
        "raw_digest": "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "normalized_digest": "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
        "verifier_id": "verifier:fixture",
        "evidence_contract_version": 1,
        "clock_unix_seconds": 200,
        "now_ms": 1000000001000_u64,
        "session_id": "session:fixture",
        "tenant_id": "tenant:fixture",
        "user_id": "user:fixture",
        "auth_adapter_id": "fixture.adapter",
        "credential_evidence_digest": "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "authenticated_at_ms": 1000000000000_u64,
        "opened_at_ms": 1000000000000_u64,
        "idle_timeout_ms": 3600000_u64,
        "absolute_timeout_ms": 86400000_u64,
        "operator_grant_id": "operator:fixture",
        "capability_key_hex": "abababababababababababababababababababababababababababababababab",
        "capability_key_version": 1,
        "schema_digest": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "descriptor_snapshot_version": 1,
        "policy_snapshot_id": "policy:fixture:v1",
        "idempotency_deadline_ms": 30000_u64
    })
}

// ---------------------------------------------------------------------------
// Temp helpers
// ---------------------------------------------------------------------------

static COUNTER: AtomicU64 = AtomicU64::new(0);

fn temp_dir(label: &str) -> PathBuf {
    let id = COUNTER.fetch_add(1, Ordering::SeqCst);
    let dir = std::env::temp_dir().join(format!("agentd-e2e-{}-{id}-{label}", std::process::id()));
    fs::create_dir_all(&dir).expect("create temp dir");
    dir
}

// ---------------------------------------------------------------------------
// Binary discovery
// ---------------------------------------------------------------------------

fn agent_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_ustc-agent"))
}

fn agentd_bin() -> PathBuf {
    let candidate = std::env::current_exe()
        .expect("current_exe")
        .parent()
        .expect("deps dir")
        .parent()
        .expect("target dir")
        .join("ustc-agentd");
    if candidate.exists() {
        return candidate;
    }
    // Build fallback: ustc-agentd may not have been built yet.
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir.parent().expect("workspace root");
    let status = Command::new("cargo")
        .args(["build", "-p", "ustc-agentd", "--all-features", "--locked"])
        .current_dir(workspace_root)
        .status()
        .expect("cargo build ustc-agentd");
    assert!(status.success(), "cargo build -p ustc-agentd failed");
    assert!(
        candidate.exists(),
        "ustc-agentd binary not found at {candidate:?}"
    );
    candidate
}

// ---------------------------------------------------------------------------
// Server environment
// ---------------------------------------------------------------------------

struct ServerEnv {
    _dir: PathBuf,
    fixture: PathBuf,
    store: PathBuf,
    idempotency: PathBuf,
    child: Child,
    endpoint: String,
}

fn spawn_agentd(
    fixture: &std::path::Path,
    store: &std::path::Path,
    idempotency: &std::path::Path,
) -> (Child, String) {
    let bin = agentd_bin();
    let mut child = Command::new(&bin)
        .args([
            "serve",
            "--bind",
            "127.0.0.1:0",
            "--fixture",
            fixture.to_str().unwrap(),
            "--store",
            store.to_str().unwrap(),
            "--idempotency",
            idempotency.to_str().unwrap(),
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn ustc-agentd");

    let stdout = child.stdout.take().expect("stdout");
    let (tx, rx) = mpsc::channel::<String>();
    std::thread::spawn(move || {
        let reader = std::io::BufReader::new(stdout);
        for line in reader.lines() {
            match line {
                Ok(text) => {
                    if tx.send(text).is_err() {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
    });

    let deadline = Instant::now() + Duration::from_secs(30);
    let mut endpoint: Option<String> = None;
    while Instant::now() < deadline {
        let remaining = deadline.saturating_duration_since(Instant::now());
        match rx.recv_timeout(remaining) {
            Ok(line) => {
                if let Some(addr) = line.strip_prefix("listening ") {
                    endpoint = Some(addr.trim().to_owned());
                    break;
                }
            }
            Err(mpsc::RecvTimeoutError::Timeout) => break,
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        }
    }
    let endpoint = endpoint.expect("agentd did not print listening line within 30s");
    (child, endpoint)
}

impl ServerEnv {
    fn start() -> Self {
        Self::start_with_fixture(base_fixture())
    }

    fn start_with_fixture(fixture: Value) -> Self {
        let dir = temp_dir("server");
        let fixture_path = dir.join("fixture.json");
        let store_path = dir.join("store.json");
        let idempotency_path = dir.join("idempotency.json");
        fs::write(&fixture_path, fixture.to_string()).expect("write fixture");

        let (child, endpoint) = spawn_agentd(&fixture_path, &store_path, &idempotency_path);

        Self {
            _dir: dir,
            fixture: fixture_path,
            store: store_path,
            idempotency: idempotency_path,
            child,
            endpoint,
        }
    }

    fn restart(&mut self) {
        self.child.kill().expect("kill old server");
        self.child.wait().expect("wait old server");

        let (child, endpoint) = spawn_agentd(&self.fixture, &self.store, &self.idempotency);
        self.child = child;
        self.endpoint = endpoint;
    }
}

impl Drop for ServerEnv {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

// ---------------------------------------------------------------------------
// CLI invocation helpers
// ---------------------------------------------------------------------------

struct CliOutcome {
    code: i32,
    stdout: String,
    stderr: String,
}

fn run_agent(args: &[&str]) -> CliOutcome {
    run_agent_with_stdin(args, None)
}

fn run_agent_with_stdin(args: &[&str], stdin_value: Option<&str>) -> CliOutcome {
    let mut command = Command::new(agent_bin());
    command
        .args(args)
        .stdin(if stdin_value.is_some() {
            Stdio::piped()
        } else {
            Stdio::null()
        })
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = command.spawn().expect("spawn ustc-agent");
    if let Some(value) = stdin_value {
        child
            .stdin
            .take()
            .expect("piped ustc-agent stdin")
            .write_all(value.as_bytes())
            .expect("write ustc-agent stdin");
    }
    let output = child.wait_with_output().expect("wait for ustc-agent");
    CliOutcome {
        code: output.status.code().unwrap_or(-1),
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
    }
}

fn payload_digest(procedure_id: &str) -> String {
    let pid = WireText::parse(procedure_id).expect("valid procedure_id");
    affairs_get_payload_digest(&pid, None)
        .expect("digest")
        .as_str()
        .to_owned()
}

fn parse_json_envelope(stdout: &str) -> Value {
    let trimmed = stdout.trim();
    serde_json::from_str(trimmed)
        .unwrap_or_else(|e| panic!("failed to parse JSON envelope: {e}\nstdout: {trimmed}"))
}

// ---------------------------------------------------------------------------
// Wire-only dependency guard (compile-time)
// ---------------------------------------------------------------------------

#[test]
fn wire_only_dependency_guard() {
    let cargo_toml = include_str!("../Cargo.toml");
    assert!(
        cargo_toml.contains("ustc-campus-agent-client-core"),
        "must depend on client-core"
    );
    assert!(
        !cargo_toml.contains("ustc-agentd"),
        "must NOT depend on ustc-agentd"
    );
    assert!(
        !cargo_toml.contains("platform-core"),
        "must NOT depend on platform-core"
    );
    assert!(
        !cargo_toml.contains("affairs-navigator"),
        "must NOT depend on affairs-navigator"
    );
    assert!(
        !cargo_toml.contains("application-ingress"),
        "must NOT depend on application-ingress"
    );
    assert!(
        !cargo_toml.contains("ustc-agentctl"),
        "must NOT depend on ustc-agentctl"
    );
}

// ---------------------------------------------------------------------------
// Public affairs get → Found (proves wire path)
// ---------------------------------------------------------------------------

#[test]
fn e2e_public_affairs_get_found() {
    let server = ServerEnv::start();
    let digest = payload_digest("proc:fixture");

    let outcome = run_agent(&[
        "affairs",
        "get",
        "--endpoint",
        &server.endpoint,
        "--procedure-id",
        "proc:fixture",
        "--request-id",
        "req:e2e-pub-found",
        "--correlation-id",
        "corr:e2e",
        "--payload-digest",
        &digest,
        "--idempotency-key",
        "idem:e2e-pub-found",
    ]);

    assert_eq!(
        outcome.code, 0,
        "exit code must be 0, stderr: {}",
        outcome.stderr
    );
    assert!(
        outcome.stderr.is_empty(),
        "stderr must be empty: {}",
        outcome.stderr
    );

    let json = parse_json_envelope(&outcome.stdout);
    assert_eq!(json["schema"], "ustc-client-result/v1");
    assert_eq!(json["exit_class"], "success");
    assert_eq!(json["exit_code"], 0);
    assert_eq!(json["origin"], "server");
    assert_eq!(json["state"]["kind"], "terminal");
    assert_eq!(json["state"]["outcome_class"], "found");
    assert_eq!(json["state"]["lineage_class"], "verified");
    assert_eq!(json["state"]["freshness_class"], "fresh");
    assert_eq!(json["state"]["terminal_kind"]["kind"], "accepted");
    assert!(
        json["state"]["terminal_kind"]["public_capability"].is_string(),
        "public submit must mint capability"
    );
    assert!(
        json["state"]["command_id"].is_string(),
        "command_id must be present"
    );
}

// ---------------------------------------------------------------------------
// Ordinary-user CLI rejects raw session authority in argv
// ---------------------------------------------------------------------------

#[test]
fn e2e_session_argv_is_rejected() {
    let server = ServerEnv::start();
    let digest = payload_digest("proc:fixture");
    let outcome = run_agent(&[
        "affairs",
        "get",
        "--endpoint",
        &server.endpoint,
        "--procedure-id",
        "proc:fixture",
        "--request-id",
        "req:e2e-session-argv",
        "--correlation-id",
        "corr:e2e",
        "--payload-digest",
        &digest,
        "--session-id",
        "session:fixture",
    ]);
    assert_eq!(outcome.code, 2);
    assert!(outcome.stderr.contains("unknown flag `--session-id`"));
    assert!(outcome.stdout.is_empty());
}

// ---------------------------------------------------------------------------
// Public get + lookup by capability (proves full lifecycle)
// ---------------------------------------------------------------------------

#[test]
fn e2e_public_get_then_lookup_by_capability() {
    let server = ServerEnv::start();
    let digest = payload_digest("proc:fixture");

    // Submit
    let get_outcome = run_agent(&[
        "affairs",
        "get",
        "--endpoint",
        &server.endpoint,
        "--procedure-id",
        "proc:fixture",
        "--request-id",
        "req:e2e-cap-lookup",
        "--correlation-id",
        "corr:e2e",
        "--payload-digest",
        &digest,
        "--idempotency-key",
        "idem:e2e-cap-lookup",
    ]);
    assert_eq!(get_outcome.code, 0);
    let get_json = parse_json_envelope(&get_outcome.stdout);
    let command_id = get_json["state"]["command_id"]
        .as_str()
        .expect("command_id")
        .to_owned();
    let capability = get_json["state"]["terminal_kind"]["public_capability"]
        .as_str()
        .expect("public_capability")
        .to_owned();

    // Lookup by capability
    let lookup_outcome = run_agent_with_stdin(
        &[
            "affairs",
            "lookup",
            "--endpoint",
            &server.endpoint,
            "--command-id",
            &command_id,
            "--capability-stdin",
        ],
        Some(&capability),
    );
    assert_eq!(lookup_outcome.code, 0, "stderr: {}", lookup_outcome.stderr);
    let lookup_json = parse_json_envelope(&lookup_outcome.stdout);
    assert_eq!(lookup_json["exit_code"], 0);
    assert_eq!(lookup_json["state"]["kind"], "terminal");
    assert_eq!(lookup_json["state"]["terminal_kind"]["kind"], "available");
    assert_eq!(lookup_json["state"]["terminal_kind"]["redaction"], "public");
    assert_eq!(lookup_json["state"]["outcome_class"], "found");
}

// ---------------------------------------------------------------------------
// Ordinary-user CLI rejects caller-supplied owner identity
// ---------------------------------------------------------------------------

#[test]
fn e2e_owner_identity_argv_is_rejected() {
    let outcome = run_agent(&[
        "affairs",
        "lookup",
        "--endpoint",
        "127.0.0.1:18080",
        "--command-id",
        "cmd:fixture",
        "--tenant-id",
        "tenant:fixture",
        "--user-id",
        "user:fixture",
    ]);
    assert_eq!(outcome.code, 2);
    assert!(outcome.stderr.contains("unknown flag `--tenant-id`"));
    assert!(outcome.stdout.is_empty());
}

// ---------------------------------------------------------------------------
// Ordinary-user CLI rejects operator authority
// ---------------------------------------------------------------------------

#[test]
fn e2e_operator_grant_argv_is_rejected() {
    let outcome = run_agent(&[
        "affairs",
        "lookup",
        "--endpoint",
        "127.0.0.1:18080",
        "--command-id",
        "cmd:fixture",
        "--grant-id",
        "operator:fixture",
    ]);
    assert_eq!(outcome.code, 2);
    assert!(outcome.stderr.contains("unknown flag `--grant-id`"));
    assert!(outcome.stdout.is_empty());
}

// ---------------------------------------------------------------------------
// Legacy capability argv is rejected before transport
// ---------------------------------------------------------------------------

#[test]
fn e2e_capability_argv_is_rejected() {
    let outcome = run_agent(&[
        "affairs",
        "lookup",
        "--endpoint",
        "127.0.0.1:18080",
        "--command-id",
        "cmd:fixture",
        "--capability",
        "cap:must-not-appear-in-argv",
    ]);
    assert_eq!(outcome.code, 2);
    assert!(outcome.stderr.contains("unknown flag `--capability`"));
    assert!(outcome.stdout.is_empty());
}

// ---------------------------------------------------------------------------
// Wrong capability → exit 6 (Unavailable, indistinguishable denial)
// ---------------------------------------------------------------------------

#[test]
fn e2e_wrong_capability_exit_6() {
    let server = ServerEnv::start();
    let digest = payload_digest("proc:fixture");

    // First submit to get a real command_id
    let get_outcome = run_agent(&[
        "affairs",
        "get",
        "--endpoint",
        &server.endpoint,
        "--procedure-id",
        "proc:fixture",
        "--request-id",
        "req:e2e-wrong-cap",
        "--correlation-id",
        "corr:e2e",
        "--payload-digest",
        &digest,
        "--idempotency-key",
        "idem:e2e-wrong-cap",
    ]);
    assert_eq!(get_outcome.code, 0);
    let get_json = parse_json_envelope(&get_outcome.stdout);
    let command_id = get_json["state"]["command_id"]
        .as_str()
        .expect("command_id")
        .to_owned();

    // Lookup with wrong capability
    let lookup_outcome = run_agent_with_stdin(
        &[
            "affairs",
            "lookup",
            "--endpoint",
            &server.endpoint,
            "--command-id",
            &command_id,
            "--capability-stdin",
        ],
        Some("deadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef"),
    );
    assert_eq!(
        lookup_outcome.code, 6,
        "wrong capability must exit 6, stderr: {}",
        lookup_outcome.stderr
    );
    let json = parse_json_envelope(&lookup_outcome.stdout);
    assert_eq!(json["exit_class"], "unavailable");
    assert_eq!(json["state"]["kind"], "unavailable");
}

// ---------------------------------------------------------------------------
// Capability stdin is mandatory and nonempty
// ---------------------------------------------------------------------------

#[test]
fn e2e_empty_capability_stdin_exit_2() {
    let outcome = run_agent_with_stdin(
        &[
            "affairs",
            "lookup",
            "--endpoint",
            "127.0.0.1:18080",
            "--command-id",
            "cmd:fixture",
            "--capability-stdin",
        ],
        Some("\n"),
    );
    assert_eq!(outcome.code, 2);
    assert!(outcome.stderr.contains("is empty"));
    assert!(outcome.stdout.is_empty());
}

// ---------------------------------------------------------------------------
// Capability stdin is one bounded value, not a multiline channel
// ---------------------------------------------------------------------------

#[test]
fn e2e_multiline_capability_stdin_exit_2() {
    let outcome = run_agent_with_stdin(
        &[
            "affairs",
            "lookup",
            "--endpoint",
            "127.0.0.1:18080",
            "--command-id",
            "cmd:fixture",
            "--capability-stdin",
        ],
        Some("cap:first\ncap:second\n"),
    );
    assert_eq!(outcome.code, 2);
    assert!(outcome.stderr.contains("exactly one value"));
    assert!(outcome.stdout.is_empty());
}

// ---------------------------------------------------------------------------
// Malformed CLI → exit 2 (no JSON envelope)
// ---------------------------------------------------------------------------

#[test]
fn e2e_missing_endpoint_exit_2() {
    let outcome = run_agent(&[
        "affairs",
        "get",
        "--procedure-id",
        "proc:fixture",
        "--request-id",
        "req:e2e-missing-ep",
        "--correlation-id",
        "corr:e2e",
        "--payload-digest",
        "abc",
    ]);
    assert_eq!(outcome.code, 2);
    assert!(outcome.stderr.contains("--endpoint"));
    assert!(outcome.stdout.is_empty(), "usage error must not print JSON");
}

#[test]
fn e2e_missing_payload_digest_exit_2() {
    let outcome = run_agent(&[
        "affairs",
        "get",
        "--endpoint",
        "127.0.0.1:18080",
        "--procedure-id",
        "proc:fixture",
        "--request-id",
        "req:e2e-missing-digest",
        "--correlation-id",
        "corr:e2e",
    ]);
    assert_eq!(outcome.code, 2);
    assert!(outcome.stderr.contains("--payload-digest"));
}

#[test]
fn e2e_duplicate_capability_stdin_flag_exit_2() {
    let outcome = run_agent(&[
        "affairs",
        "lookup",
        "--endpoint",
        "127.0.0.1:18080",
        "--command-id",
        "cmd1",
        "--capability-stdin",
        "--capability-stdin",
    ]);
    assert_eq!(outcome.code, 2);
    assert!(outcome.stderr.contains("duplicate flag"));
}

#[test]
fn e2e_non_loopback_endpoint_exit_2() {
    let outcome = run_agent(&[
        "affairs",
        "get",
        "--endpoint",
        "8.8.8.8:8080",
        "--procedure-id",
        "proc:fixture",
        "--request-id",
        "req:e2e-non-loopback",
        "--correlation-id",
        "corr:e2e",
        "--payload-digest",
        "abc",
    ]);
    assert_eq!(outcome.code, 2);
    assert!(outcome.stderr.contains("not loopback"));
}

// ---------------------------------------------------------------------------
// Server unavailable → exit 6 (transport-originated)
// ---------------------------------------------------------------------------

#[test]
fn e2e_server_unavailable_exit_6() {
    // Use a port that is almost certainly not listening.
    let outcome = run_agent(&[
        "affairs",
        "get",
        "--endpoint",
        "127.0.0.1:1",
        "--procedure-id",
        "proc:fixture",
        "--request-id",
        "req:e2e-no-server",
        "--correlation-id",
        "corr:e2e",
        "--payload-digest",
        "abc",
        "--timeout",
        "2",
    ]);
    assert_eq!(outcome.code, 6, "server unavailable must exit 6");
    let json = parse_json_envelope(&outcome.stdout);
    assert_eq!(json["origin"], "transport");
    assert_eq!(json["state"]["kind"], "unavailable");
}

// ---------------------------------------------------------------------------
// Response-loss recovery: restart server, retry, same terminal
// ---------------------------------------------------------------------------

#[test]
fn e2e_response_loss_recovery_restart_retry() {
    let mut server = ServerEnv::start();
    let digest = payload_digest("proc:fixture");

    // First submit
    let get1 = run_agent(&[
        "affairs",
        "get",
        "--endpoint",
        &server.endpoint,
        "--procedure-id",
        "proc:fixture",
        "--request-id",
        "req:e2e-restart",
        "--correlation-id",
        "corr:e2e",
        "--payload-digest",
        &digest,
        "--idempotency-key",
        "idem:e2e-restart",
    ]);
    assert_eq!(get1.code, 0);
    let json1 = parse_json_envelope(&get1.stdout);
    let command_id1 = json1["state"]["command_id"]
        .as_str()
        .expect("command_id")
        .to_owned();
    let outcome_class1 = json1["state"]["outcome_class"]
        .as_str()
        .expect("outcome_class")
        .to_owned();
    let capability1 = json1["state"]["terminal_kind"]["public_capability"]
        .as_str()
        .expect("public_capability")
        .to_owned();

    // Restart server on same durable files
    server.restart();

    // Retry identical request
    let get2 = run_agent(&[
        "affairs",
        "get",
        "--endpoint",
        &server.endpoint,
        "--procedure-id",
        "proc:fixture",
        "--request-id",
        "req:e2e-restart",
        "--correlation-id",
        "corr:e2e",
        "--payload-digest",
        &digest,
        "--idempotency-key",
        "idem:e2e-restart",
    ]);
    assert_eq!(
        get2.code, 0,
        "retry after restart must succeed, stderr: {}",
        get2.stderr
    );
    let json2 = parse_json_envelope(&get2.stdout);
    let command_id2 = json2["state"]["command_id"]
        .as_str()
        .expect("command_id")
        .to_owned();
    let outcome_class2 = json2["state"]["outcome_class"]
        .as_str()
        .expect("outcome_class")
        .to_owned();
    let capability2 = json2["state"]["terminal_kind"]["public_capability"]
        .as_str()
        .expect("public_capability")
        .to_owned();

    assert_eq!(
        command_id1, command_id2,
        "retry after restart must return same command_id"
    );
    assert_eq!(
        outcome_class1, outcome_class2,
        "retry after restart must return same outcome_class"
    );
    assert_eq!(
        capability1, capability2,
        "retry after restart must reproduce same public capability"
    );

    // Lookup proves durable terminal record survived restart
    let lookup = run_agent_with_stdin(
        &[
            "affairs",
            "lookup",
            "--endpoint",
            &server.endpoint,
            "--command-id",
            &command_id2,
            "--capability-stdin",
        ],
        Some(&capability2),
    );
    assert_eq!(lookup.code, 0, "durable lookup after restart must succeed");
    let lookup_json = parse_json_envelope(&lookup.stdout);
    assert_eq!(lookup_json["state"]["kind"], "terminal");
    assert_eq!(lookup_json["state"]["terminal_kind"]["kind"], "available");
    assert_eq!(lookup_json["state"]["terminal_kind"]["redaction"], "public");
}

// ---------------------------------------------------------------------------
// --version and --help
// ---------------------------------------------------------------------------

#[test]
fn e2e_version_exit_0() {
    let outcome = run_agent(&["--version"]);
    assert_eq!(outcome.code, 0);
    assert!(outcome.stdout.contains("ustc-agent"));
}

#[test]
fn e2e_help_exit_0() {
    let outcome = run_agent(&["--help"]);
    assert_eq!(outcome.code, 0);
    assert!(outcome.stdout.contains("affairs get"));
    assert!(outcome.stdout.contains("affairs lookup"));
}

// ---------------------------------------------------------------------------
// Malformed TCP frame: server rejects before handler, durable state unchanged
// ---------------------------------------------------------------------------

#[test]
fn e2e_malformed_frame_no_state_mutation() {
    use std::io::Write;
    use std::net::TcpStream;

    let server = ServerEnv::start();

    let mut stream = TcpStream::connect(&server.endpoint).expect("connect");

    let before_store = fs::read(&server.store).ok();
    let before_idem = fs::read(&server.idempotency).ok();

    let zero_length = 0u32.to_be_bytes();
    stream
        .write_all(&zero_length)
        .expect("write zero-length frame");
    stream.flush().expect("flush");

    let read_result = std::io::Read::read(&mut stream, &mut [0u8; 16]);
    assert!(
        read_result.is_err() || read_result.as_ref().is_ok_and(|n| *n == 0),
        "server must close/reject after malformed frame, got {:?}",
        read_result
    );
    drop(stream);

    let after_store = fs::read(&server.store).ok();
    let after_idem = fs::read(&server.idempotency).ok();
    assert_eq!(
        before_store, after_store,
        "store file must be unchanged after malformed frame"
    );
    assert_eq!(
        before_idem, after_idem,
        "idempotency file must be unchanged after malformed frame"
    );
}

#[test]
fn e2e_oversized_frame_no_state_mutation() {
    use std::io::Write;
    use std::net::TcpStream;

    let server = ServerEnv::start();

    let mut stream = TcpStream::connect(&server.endpoint).expect("connect");

    let before_store = fs::read(&server.store).ok();
    let before_idem = fs::read(&server.idempotency).ok();

    let oversized = u32::MAX.to_be_bytes();
    stream
        .write_all(&oversized)
        .expect("write oversized length");
    stream.flush().expect("flush");

    let read_result = std::io::Read::read(&mut stream, &mut [0u8; 16]);
    assert!(
        read_result.is_err() || read_result.as_ref().is_ok_and(|n| *n == 0),
        "server must close/reject after oversized frame, got {:?}",
        read_result
    );
    drop(stream);

    let after_store = fs::read(&server.store).ok();
    let after_idem = fs::read(&server.idempotency).ok();
    assert_eq!(
        before_store, after_store,
        "store file must be unchanged after oversized frame"
    );
    assert_eq!(
        before_idem, after_idem,
        "idempotency file must be unchanged after oversized frame"
    );
}

#[test]
fn e2e_invalid_json_frame_no_state_mutation() {
    use std::io::Write;
    use std::net::TcpStream;

    let server = ServerEnv::start();

    let mut stream = TcpStream::connect(&server.endpoint).expect("connect");

    let before_store = fs::read(&server.store).ok();
    let before_idem = fs::read(&server.idempotency).ok();

    let payload = b"not valid json";
    let length = u32::try_from(payload.len()).unwrap().to_be_bytes();
    stream.write_all(&length).expect("write length");
    stream.write_all(payload).expect("write payload");
    stream.flush().expect("flush");

    let read_result = std::io::Read::read(&mut stream, &mut [0u8; 16]);
    assert!(
        read_result.is_err() || read_result.as_ref().is_ok_and(|n| *n == 0),
        "server must close/reject after invalid JSON frame, got {:?}",
        read_result
    );
    drop(stream);

    let after_store = fs::read(&server.store).ok();
    let after_idem = fs::read(&server.idempotency).ok();
    assert_eq!(
        before_store, after_store,
        "store file must be unchanged after invalid JSON frame"
    );
    assert_eq!(
        before_idem, after_idem,
        "idempotency file must be unchanged after invalid JSON frame"
    );
}
