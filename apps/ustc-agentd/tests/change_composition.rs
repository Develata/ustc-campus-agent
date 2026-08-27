#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use serde_json::{Value, json};
use ustc_agentd::AffairsComposition;
use ustc_campus_agent_client_protocol::{
    ActorIntentDto, ClientErrorDto, ClientProvenanceDto, ClientResponseDto,
    M70ChangeFeedOutcomeDto, SubmitAffairsGetDto, SubmitChangeFeedDto, WireErrorClassDto, WireText,
    affairs_get_payload_digest, change_feed_payload_digest,
};

static COUNTER: AtomicU64 = AtomicU64::new(0);

fn workspace() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace")
        .to_path_buf()
}

fn temp_dir() -> PathBuf {
    let id = COUNTER.fetch_add(1, Ordering::SeqCst);
    let dir = std::env::temp_dir().join(format!(
        "agentd-change-composition-{}-{id}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).expect("create temp dir");
    dir
}

fn wire(value: &str) -> WireText {
    WireText::parse(value).expect("wire")
}

struct TestEnv {
    dir: PathBuf,
    affairs_fixture: PathBuf,
    change_fixture: PathBuf,
    store: PathBuf,
    idempotency: PathBuf,
}

impl TestEnv {
    fn new() -> Self {
        Self::with_overrides(None, None)
    }

    fn with_change_override(key: &str, value: Value) -> Self {
        Self::with_overrides(None, Some((key, value)))
    }

    fn with_affairs_override(key: &str, value: Value) -> Self {
        Self::with_overrides(Some((key, value)), None)
    }

    fn with_overrides(
        affairs_override: Option<(&str, Value)>,
        change_override: Option<(&str, Value)>,
    ) -> Self {
        let dir = temp_dir();
        let source_affairs = workspace().join("fixtures/affairs/proc-011-reviewed.json");
        let source_change =
            workspace().join("fixtures/change-radar/academic-calendar-demo-reviewed.json");
        let affairs_fixture = dir.join("affairs.json");
        let change_root = dir.join("change-radar");
        let change_evidence = change_root.join("evidence");
        fs::create_dir_all(&change_evidence).expect("create change evidence");
        let change_fixture = change_root.join("academic-calendar-demo-reviewed.json");

        let mut affairs: Value =
            serde_json::from_slice(&fs::read(source_affairs).expect("read affairs fixture"))
                .expect("parse affairs fixture");
        if let Some((key, value)) = affairs_override {
            affairs[key] = value;
        }
        fs::write(&affairs_fixture, affairs.to_string()).expect("write affairs fixture");

        let mut change: Value =
            serde_json::from_slice(&fs::read(source_change).expect("read change fixture"))
                .expect("parse change fixture");
        if let Some((key, value)) = change_override {
            change[key] = value;
        }
        fs::write(&change_fixture, change.to_string()).expect("write change fixture");
        for name in [
            "academic-calendar-r1.reviewed.txt",
            "academic-calendar-r1.normalized.json",
            "academic-calendar-r2.reviewed.txt",
            "academic-calendar-r2.normalized.json",
        ] {
            fs::copy(
                workspace()
                    .join("fixtures/change-radar/evidence")
                    .join(name),
                change_evidence.join(name),
            )
            .expect("copy change evidence");
        }
        Self {
            store: dir.join("records.json"),
            idempotency: dir.join("idempotency.json"),
            dir,
            affairs_fixture,
            change_fixture,
        }
    }

    fn open(&self) -> AffairsComposition {
        AffairsComposition::open_with_change(
            &self.affairs_fixture,
            &self.change_fixture,
            &self.store,
            &self.idempotency,
        )
        .expect("open shared composition")
    }

    fn change_evidence_path(&self, name: &str) -> PathBuf {
        self.dir.join("change-radar/evidence").join(name)
    }
}

impl Drop for TestEnv {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.dir);
    }
}

fn change_request(board_id: &str, suffix: &str) -> SubmitChangeFeedDto {
    let board_id = wire(board_id);
    SubmitChangeFeedDto {
        request_id: wire(&format!("req:change:{suffix}")),
        correlation_id: wire(&format!("corr:change:{suffix}")),
        causation_id: None,
        idempotency_key: Some(wire(&format!("idem:change:{suffix}"))),
        actor: ActorIntentDto::Public,
        provenance: ClientProvenanceDto {
            build: wire("build:test"),
            target: wire("linux"),
            protocol: wire("m10:v2"),
        },
        payload_digest: change_feed_payload_digest(&board_id).expect("digest"),
        board_id,
    }
}

fn affairs_request() -> SubmitAffairsGetDto {
    let procedure_id = wire("proc:ustc:undergraduate:transcript-certificate");
    SubmitAffairsGetDto {
        request_id: wire("req:affairs:isolation"),
        correlation_id: wire("corr:affairs:isolation"),
        causation_id: None,
        idempotency_key: Some(wire("idem:affairs:isolation")),
        actor: ActorIntentDto::Public,
        provenance: ClientProvenanceDto {
            build: wire("build:test"),
            target: wire("linux"),
            protocol: wire("m10:v2"),
        },
        payload_digest: affairs_get_payload_digest(&procedure_id, None).expect("digest"),
        procedure_id,
        as_of: None,
    }
}

#[test]
fn reviewed_change_feed_crosses_m10_market_harness_gateway_and_plugin() {
    let env = TestEnv::new();
    let composition = env.open();
    let response =
        composition.handle_change_submit(&change_request("board:ustc:academic-calendar", "found"));
    let terminal = match response {
        ClientResponseDto::ChangeFeedAccepted { terminal, .. } => terminal,
        other => panic!("expected change feed accepted, got {other:?}"),
    };
    let view = match terminal.outcome() {
        M70ChangeFeedOutcomeDto::Found { view } => view,
        other => panic!("expected found, got {other:?}"),
    };
    assert_eq!(view.board_id().as_str(), "board:ustc:academic-calendar");
    assert_eq!(view.entries().len(), 1);
    let entry = &view.entries()[0];
    assert_eq!(entry.changed_fields().len(), 2);
    assert!(
        entry
            .changed_fields()
            .iter()
            .any(|field| field.field().as_str() == "registration.deadline")
    );
    assert!(
        entry
            .changed_fields()
            .iter()
            .any(|field| field.field().as_str() == "location")
    );
    assert_eq!(
        entry.source_id().as_str(),
        "src:ustc:academic-calendar:2026-fall"
    );
    assert!(entry.source_url().as_str().starts_with("https://"));
    assert!(
        view.atom()
            .contains("<feed xmlns=\"http://www.w3.org/2005/Atom\">")
    );
    assert!(view.atom().contains("old_raw_sha256="));
    assert_eq!(composition.change_invocation_counts(), (1, 1, 1));
    assert_eq!(composition.invocation_counts(), (0, 0, 0));
}

#[test]
fn unknown_board_is_stable_plugin_owned_not_found() {
    let env = TestEnv::new();
    let composition = env.open();
    let response =
        composition.handle_change_submit(&change_request("board:ustc:unknown", "unknown"));
    match response {
        ClientResponseDto::ChangeFeedAccepted { terminal, .. } => match terminal.outcome() {
            M70ChangeFeedOutcomeDto::NotFound { board_id } => {
                assert_eq!(board_id.as_str(), "board:ustc:unknown");
            }
            other => panic!("expected not found, got {other:?}"),
        },
        other => panic!("expected change feed accepted, got {other:?}"),
    }
    assert_eq!(composition.change_invocation_counts(), (1, 1, 1));
}

#[test]
fn disabled_or_revoked_change_plugin_denies_before_intent_and_executor() {
    for (key, suffix) in [
        ("market_enabled", "disabled"),
        ("market_grant_active", "revoked"),
    ] {
        let env = TestEnv::with_change_override(key, json!(false));
        let composition = env.open();
        let response = composition
            .handle_change_submit(&change_request("board:ustc:academic-calendar", suffix));
        match response {
            ClientResponseDto::Error {
                error: ClientErrorDto::Admission { error },
            } => assert_eq!(error.class, WireErrorClassDto::PolicyDenied),
            other => panic!("expected policy denied, got {other:?}"),
        }
        assert_eq!(composition.change_invocation_counts(), (0, 0, 0));
    }
}

#[test]
fn malformed_change_digest_reaches_no_plugin_execution() {
    let env = TestEnv::new();
    let composition = env.open();
    let mut request = change_request("board:ustc:academic-calendar", "bad-digest");
    request.payload_digest =
        wire("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");
    match composition.handle_change_submit(&request) {
        ClientResponseDto::Error {
            error: ClientErrorDto::Admission { error },
        } => assert_eq!(error.class, WireErrorClassDto::MalformedCommand),
        other => panic!("expected malformed command, got {other:?}"),
    }
    assert_eq!(composition.change_invocation_counts(), (0, 0, 0));
}

#[test]
fn disabling_change_does_not_disable_affairs() {
    let env = TestEnv::with_change_override("market_enabled", json!(false));
    let composition = env.open();
    assert!(matches!(
        composition.handle_submit(&affairs_request()),
        ClientResponseDto::Accepted { .. }
    ));
    assert_eq!(composition.invocation_counts(), (1, 1, 1));
    assert_eq!(composition.change_invocation_counts(), (0, 0, 0));
}

#[test]
fn disabling_affairs_does_not_disable_change() {
    let env = TestEnv::with_affairs_override("market_enabled", json!(false));
    let composition = env.open();
    assert!(matches!(
        composition.handle_change_submit(&change_request(
            "board:ustc:academic-calendar",
            "affairs-disabled"
        )),
        ClientResponseDto::ChangeFeedAccepted { .. }
    ));
    assert_eq!(composition.invocation_counts(), (0, 0, 0));
    assert_eq!(composition.change_invocation_counts(), (1, 1, 1));
}

#[test]
fn retained_change_evidence_tamper_blocks_startup() {
    let env = TestEnv::new();
    fs::write(
        env.change_evidence_path("academic-calendar-r2.reviewed.txt"),
        b"tampered",
    )
    .expect("tamper evidence");
    let result = AffairsComposition::open_with_change(
        &env.affairs_fixture,
        &env.change_fixture,
        &env.store,
        &env.idempotency,
    );
    assert!(result.is_err(), "tampered evidence must fail closed");
}

#[test]
fn declared_raw_revision_digest_must_match_retained_evidence_bytes() {
    let env = TestEnv::new();
    let mut fixture: Value =
        serde_json::from_slice(&fs::read(&env.change_fixture).expect("read copied change fixture"))
            .expect("parse copied change fixture");
    fixture["old_revision"]["raw_digest"] =
        json!("sha256:ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff");
    fs::write(&env.change_fixture, fixture.to_string()).expect("write mismatched fixture");

    let result = AffairsComposition::open_with_change(
        &env.affairs_fixture,
        &env.change_fixture,
        &env.store,
        &env.idempotency,
    );
    assert!(
        result.is_err(),
        "fixture identity must not claim a digest different from retained bytes"
    );
}
