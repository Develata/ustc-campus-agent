#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use serde_json::{Value, json};
use ustc_agentd::AffairsComposition;
use ustc_campus_agent_application_ingress::ChangePublicationOutcome;
use ustc_campus_agent_client_protocol::{
    ActorIntentDto, ClientErrorDto, ClientProvenanceDto, ClientResponseDto,
    M70ChangeFeedOutcomeDto, SubmitAffairsGetDto, SubmitChangeFeedDto, WireErrorClassDto, WireText,
    affairs_get_payload_digest, change_feed_payload_digest,
};
use ustc_campus_agent_core::source_revision::RevisionTimestamp;

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
    fs::set_permissions(&dir, fs::Permissions::from_mode(0o700)).expect("secure temp dir");
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
    sessions: PathBuf,
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
            sessions: dir.join("sessions.json"),
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
            &self.sessions,
        )
        .expect("open shared composition")
    }

    fn change_evidence_path(&self, name: &str) -> PathBuf {
        self.dir.join("change-radar/evidence").join(name)
    }

    fn change_publication_state_path(&self) -> PathBuf {
        self.idempotency.with_extension("change-publication.json")
    }

    fn control_evidence_path(&self) -> PathBuf {
        self.idempotency.with_extension("control-evidence.json")
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
    let mut composition = env.open();
    let published = composition.publish_change_demo_as_administrator();
    assert!(matches!(published, ChangePublicationOutcome::Published(_)));
    assert_eq!(composition.change_publication_counts(), Ok((1, 1)));
    assert_eq!(composition.change_publication_m60_call_count(), 1);
    assert_eq!(composition.change_invocation_counts(), (0, 0, 0));
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
fn reviewed_candidate_is_not_public_before_administrator_publication() {
    let env = TestEnv::new();
    let composition = env.open();
    let response = composition.handle_change_submit(&change_request(
        "board:ustc:academic-calendar",
        "before-publish",
    ));
    let ClientResponseDto::ChangeFeedAccepted { terminal, .. } = response else {
        panic!("expected accepted empty feed")
    };
    let M70ChangeFeedOutcomeDto::Found { view } = terminal.outcome() else {
        panic!(
            "expected verified empty board before publication, got {:?}",
            terminal.outcome()
        )
    };
    assert!(view.entries().is_empty());
    assert!(!view.atom().contains("<entry>"));
    assert_eq!(composition.change_publication_counts(), Ok((0, 0)));
    assert_eq!(composition.change_publication_m60_call_count(), 0);
    assert_eq!(composition.change_publication_application_call_count(), 0);
}

#[test]
fn exact_publication_retry_is_stable_and_does_not_repeat_m60() {
    let env = TestEnv::new();
    let mut composition = env.open();
    let first = match composition.publish_change_demo_as_administrator() {
        ChangePublicationOutcome::Published(value) => value,
        other => panic!("expected first publication, got {other:?}"),
    };
    let second = match composition.publish_change_demo_as_administrator() {
        ChangePublicationOutcome::Published(value) => value,
        other => panic!("expected exact publication retry, got {other:?}"),
    };
    assert_eq!(first, second);
    assert_eq!(composition.change_publication_counts(), Ok((1, 1)));
    assert_eq!(composition.change_publication_m60_call_count(), 1);
    assert_eq!(composition.control_evidence_event_count(), 1);
}

#[test]
fn changed_payload_reuse_of_publication_identity_fails_before_m70() {
    let env = TestEnv::new();
    let mut composition = env.open();
    assert!(matches!(
        composition.publish_change_demo_as_administrator(),
        ChangePublicationOutcome::Published(_)
    ));
    let applications = composition.change_publication_application_call_count();
    let changed = composition
        .publish_change_demo_at_for_test(RevisionTimestamp::from_unix_seconds(9_999_999));
    assert!(
        !matches!(changed, ChangePublicationOutcome::Published(_)),
        "changed-payload identity reuse must fail closed"
    );
    assert_eq!(
        composition.change_publication_application_call_count(),
        applications,
        "identity conflict must not reach the owning M70 port"
    );
    assert_eq!(composition.change_publication_counts(), Ok((1, 1)));
    assert_eq!(composition.control_evidence_event_count(), 1);
}

#[test]
fn corrupt_m00_evidence_blocks_exact_retry_before_the_owning_port() {
    let env = TestEnv::new();
    let mut composition = env.open();
    assert!(matches!(
        composition.publish_change_demo_as_administrator(),
        ChangePublicationOutcome::Published(_)
    ));
    let applications = composition.change_publication_application_call_count();
    fs::write(env.control_evidence_path(), b"{}\n").expect("corrupt evidence");

    assert!(matches!(
        composition.publish_change_demo_as_administrator(),
        ChangePublicationOutcome::EvidenceRejected(_)
    ));
    assert_eq!(
        composition.change_publication_application_call_count(),
        applications
    );
    assert_eq!(composition.change_publication_counts(), Ok((1, 1)));
}

#[test]
fn denied_publication_never_reaches_m70_or_durable_evidence() {
    let env = TestEnv::new();
    let mut composition = env.open();
    composition.set_change_publication_capability(
        ustc_campus_agent_core::request_context::CapabilityDisposition::Disabled,
    );
    assert!(matches!(
        composition.publish_change_demo_as_administrator(),
        ChangePublicationOutcome::Rejected(_)
    ));
    assert_eq!(composition.change_publication_application_call_count(), 0);
    assert_eq!(composition.change_publication_m60_call_count(), 0);
    assert_eq!(composition.change_publication_counts(), Ok((0, 0)));
    assert_eq!(composition.control_evidence_event_count(), 0);
}

#[test]
fn persistence_failure_after_m00_evidence_is_invisible_and_retryable() {
    let env = TestEnv::new();
    let mut composition = env.open();
    composition.fail_next_change_publication_persistence_for_test();
    assert!(matches!(
        composition.publish_change_demo_as_administrator(),
        ChangePublicationOutcome::PublicationRejected(_)
    ));
    assert_eq!(composition.control_evidence_event_count(), 1);
    assert_eq!(composition.change_publication_counts(), Ok((0, 0)));
    assert_eq!(composition.change_publication_m60_call_count(), 0);

    drop(composition);
    let mut composition = env.open();
    assert!(matches!(
        composition.publish_change_demo_as_administrator(),
        ChangePublicationOutcome::Published(_)
    ));
    assert_eq!(composition.control_evidence_event_count(), 1);
    assert_eq!(composition.change_publication_counts(), Ok((1, 1)));
    assert_eq!(composition.change_publication_m60_call_count(), 1);
}

#[test]
fn publication_persistence_failure_recovers_durable_review_before_exact_retry() {
    let env = TestEnv::new();
    let mut composition = env.open();
    composition.fail_change_publication_final_persistence_for_test();
    assert!(matches!(
        composition.publish_change_demo_as_administrator(),
        ChangePublicationOutcome::PublicationRejected(_)
    ));
    assert_eq!(composition.control_evidence_event_count(), 1);
    assert_eq!(composition.change_publication_counts(), Ok((1, 0)));
    assert_eq!(composition.change_publication_m60_call_count(), 1);

    drop(composition);
    let mut composition = env.open();
    assert_eq!(composition.change_publication_counts(), Ok((1, 0)));
    assert_eq!(
        composition.change_publication_m60_call_count(),
        0,
        "restart recovery of the durable review must perform no M60 I/O"
    );
    assert!(matches!(
        composition.publish_change_demo_as_administrator(),
        ChangePublicationOutcome::Published(_)
    ));
    assert_eq!(composition.control_evidence_event_count(), 1);
    assert_eq!(composition.change_publication_counts(), Ok((1, 1)));
    assert_eq!(composition.change_publication_m60_call_count(), 1);
}

#[test]
fn post_rename_parent_sync_failure_reconciles_same_process_before_retry() {
    let env = TestEnv::new();
    let mut composition = env.open();
    composition.fail_next_change_publication_parent_sync_after_rename_for_test();
    assert!(matches!(
        composition.publish_change_demo_as_administrator(),
        ChangePublicationOutcome::PublicationRejected(_)
    ));
    assert_eq!(
        composition.change_publication_counts(),
        Ok((1, 0)),
        "post-rename failure must reconcile memory to the canonical renamed state"
    );
    assert_eq!(composition.change_publication_m60_call_count(), 0);

    assert!(matches!(
        composition.publish_change_demo_as_administrator(),
        ChangePublicationOutcome::Published(_)
    ));
    assert_eq!(composition.change_publication_counts(), Ok((1, 1)));
    assert_eq!(composition.change_publication_m60_call_count(), 1);
    assert_eq!(composition.control_evidence_event_count(), 1);
}

#[test]
fn publication_parent_sync_uncertainty_reconciles_exact_visible_commit() {
    let env = TestEnv::new();
    let mut composition = env.open();
    composition.fail_change_publication_final_parent_sync_for_test();
    assert!(matches!(
        composition.publish_change_demo_as_administrator(),
        ChangePublicationOutcome::PublicationRejected(_)
    ));
    assert_eq!(composition.change_publication_counts(), Ok((1, 1)));
    assert_eq!(composition.change_publication_m60_call_count(), 1);
    let receipt = composition
        .change_publication_receipt_id()
        .expect("read renamed publication receipt")
        .expect("renamed publication receipt")
        .to_owned();

    let ChangePublicationOutcome::Published(retry) =
        composition.publish_change_demo_as_administrator()
    else {
        panic!("exact retry must return the reconciled publication")
    };
    assert_eq!(retry.receipt_id().as_str(), receipt);
    assert_eq!(composition.change_publication_counts(), Ok((1, 1)));
    assert_eq!(
        composition.change_publication_m60_call_count(),
        1,
        "reconciled exact retry must not repeat M60"
    );
    assert_eq!(composition.control_evidence_event_count(), 1);
}

#[test]
fn restart_recovers_exact_publication_without_m60_io() {
    let env = TestEnv::new();
    let receipt = {
        let mut composition = env.open();
        let ChangePublicationOutcome::Published(publication) =
            composition.publish_change_demo_as_administrator()
        else {
            panic!("initial publication failed")
        };
        publication.receipt_id().as_str().to_owned()
    };

    let mut reopened = env.open();
    assert_eq!(reopened.change_publication_counts(), Ok((1, 1)));
    assert_eq!(
        reopened.change_publication_receipt_id(),
        Ok(Some(receipt.as_str()))
    );
    assert_eq!(reopened.change_publication_m60_call_count(), 0);
    let ChangePublicationOutcome::Published(replayed) =
        reopened.publish_change_demo_as_administrator()
    else {
        panic!("restart retry failed")
    };
    assert_eq!(replayed.receipt_id().as_str(), receipt);
    assert_eq!(reopened.change_publication_m60_call_count(), 0);
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
fn failed_fresh_change_bootstrap_rolls_back_every_state_member_and_retry_succeeds() {
    let env = TestEnv::new();
    let valid_fixture = fs::read(&env.change_fixture).expect("read valid change fixture");
    fs::write(&env.change_fixture, b"{").expect("install malformed change fixture");

    assert!(
        AffairsComposition::open_with_change(
            &env.affairs_fixture,
            &env.change_fixture,
            &env.store,
            &env.idempotency,
            &env.sessions,
        )
        .is_err()
    );
    for path in [
        env.store.clone(),
        env.idempotency.clone(),
        env.sessions.clone(),
        env.idempotency.with_extension("affairs-publication.json"),
        env.control_evidence_path(),
        env.change_publication_state_path(),
    ] {
        assert!(
            !path.exists(),
            "failed fresh bootstrap left {}",
            path.display()
        );
    }

    fs::write(&env.change_fixture, valid_fixture).expect("restore valid change fixture");
    drop(env.open());
    for path in [
        env.store.clone(),
        env.idempotency.clone(),
        env.sessions.clone(),
        env.idempotency.with_extension("affairs-publication.json"),
        env.control_evidence_path(),
        env.change_publication_state_path(),
    ] {
        assert!(path.exists(), "successful retry omitted {}", path.display());
    }
}

#[test]
fn concurrent_fresh_change_bootstraps_cannot_rollback_a_successful_peer() {
    let env = TestEnv::new();
    let malformed = env.dir.join("malformed-change.json");
    fs::write(&malformed, b"{").expect("write malformed fixture");
    let barrier = std::sync::Arc::new(std::sync::Barrier::new(3));

    let spawn_open = |change_fixture: PathBuf| {
        let barrier = std::sync::Arc::clone(&barrier);
        let affairs_fixture = env.affairs_fixture.clone();
        let store = env.store.clone();
        let idempotency = env.idempotency.clone();
        let sessions = env.sessions.clone();
        std::thread::spawn(move || {
            barrier.wait();
            AffairsComposition::open_with_change(
                &affairs_fixture,
                &change_fixture,
                &store,
                &idempotency,
                &sessions,
            )
            .is_ok()
        })
    };
    let malformed_open = spawn_open(malformed);
    let valid_open = spawn_open(env.change_fixture.clone());
    barrier.wait();

    assert!(!malformed_open.join().expect("malformed join"));
    assert!(valid_open.join().expect("valid join"));
    drop(env.open());
    for path in [
        env.store.clone(),
        env.idempotency.clone(),
        env.sessions.clone(),
        env.idempotency.with_extension("affairs-publication.json"),
        env.control_evidence_path(),
        env.change_publication_state_path(),
    ] {
        assert!(path.exists(), "successful peer lost {}", path.display());
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
fn existing_state_set_rejects_missing_or_noncanonical_change_publication_state() {
    let env = TestEnv::new();
    drop(env.open());
    let state_path = env.change_publication_state_path();
    let canonical = fs::read(&state_path).expect("read canonical change publication state");

    fs::remove_file(&state_path).expect("remove change publication state");
    assert!(
        AffairsComposition::open_with_change(
            &env.affairs_fixture,
            &env.change_fixture,
            &env.store,
            &env.idempotency,
            &env.sessions,
        )
        .is_err(),
        "an existing state set must not degrade a missing ChangeRadar member"
    );

    let value: Value = serde_json::from_slice(&canonical).expect("parse canonical state");
    fs::write(
        &state_path,
        serde_json::to_vec_pretty(&value).expect("pretty state"),
    )
    .expect("write noncanonical state");
    fs::set_permissions(&state_path, fs::Permissions::from_mode(0o600)).expect("secure state mode");
    assert!(
        AffairsComposition::open_with_change(
            &env.affairs_fixture,
            &env.change_fixture,
            &env.store,
            &env.idempotency,
            &env.sessions,
        )
        .is_err(),
        "noncanonical ChangeRadar state must fail closed"
    );
}

#[test]
fn unsafe_and_conflicting_change_publication_state_shapes_fail_closed() {
    for case in [
        "wrong-mode",
        "unsafe-parent-mode",
        "hardlink",
        "symlink",
        "directory",
        "fifo",
        "oversized",
        "duplicate-field",
        "conflicting-identity",
    ] {
        let env = TestEnv::new();
        drop(env.open());
        let state = env.change_publication_state_path();
        match case {
            "wrong-mode" => fs::set_permissions(&state, fs::Permissions::from_mode(0o644))
                .expect("set wrong mode"),
            "unsafe-parent-mode" => fs::set_permissions(
                state.parent().expect("state parent"),
                fs::Permissions::from_mode(0o755),
            )
            .expect("set unsafe parent mode"),
            "hardlink" => {
                let target = state.with_extension("hardlink-target");
                fs::rename(&state, &target).expect("move state");
                fs::hard_link(&target, &state).expect("hard-link state");
            }
            "symlink" => {
                fs::remove_file(&state).expect("remove state");
                std::os::unix::fs::symlink("/dev/null", &state).expect("symlink state");
            }
            "directory" => {
                fs::remove_file(&state).expect("remove state");
                fs::create_dir(&state).expect("directory state");
            }
            "fifo" => {
                fs::remove_file(&state).expect("remove state");
                assert!(
                    std::process::Command::new("mkfifo")
                        .arg(&state)
                        .status()
                        .expect("run mkfifo")
                        .success()
                );
            }
            "oversized" => {
                fs::write(&state, vec![b'x'; 1_048_577]).expect("oversized state");
                fs::set_permissions(&state, fs::Permissions::from_mode(0o600))
                    .expect("secure oversized mode");
            }
            "duplicate-field" => {
                fs::write(&state, br#"{"schema_version":1,"schema_version":1}"#)
                    .expect("duplicate-field state");
                fs::set_permissions(&state, fs::Permissions::from_mode(0o600))
                    .expect("secure duplicate mode");
            }
            "conflicting-identity" => {
                let mut value: Value =
                    serde_json::from_slice(&fs::read(&state).expect("read state"))
                        .expect("parse state");
                let event_id = value["binding"]["candidate_event_id"]
                    .as_str()
                    .expect("event id")
                    .to_owned();
                let mut bytes = event_id.into_bytes();
                let last = bytes.last_mut().expect("nonempty event id");
                *last = if *last == b'a' { b'b' } else { b'a' };
                value["binding"]["candidate_event_id"] =
                    Value::String(String::from_utf8(bytes).expect("ASCII event id"));
                fs::write(
                    &state,
                    serde_json::to_vec(&value).expect("canonical conflict"),
                )
                .expect("conflicting state");
            }
            _ => unreachable!(),
        }
        assert!(
            AffairsComposition::open_with_change(
                &env.affairs_fixture,
                &env.change_fixture,
                &env.store,
                &env.idempotency,
                &env.sessions,
            )
            .is_err(),
            "unsafe ChangeRadar state case {case} must fail closed"
        );
    }
}

#[test]
fn runtime_state_replacement_is_infrastructure_error_not_empty_feed() {
    let env = TestEnv::new();
    let composition = env.open();
    fs::write(env.change_publication_state_path(), b"{}").expect("replace live ChangeRadar state");
    let response = composition.handle_change_submit(&change_request(
        "board:ustc:academic-calendar",
        "runtime-corrupt",
    ));
    assert!(
        matches!(response, ClientResponseDto::Error { .. }),
        "durable corruption must not degrade into accepted NotFound"
    );
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
        &env.sessions,
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
        &env.sessions,
    );
    assert!(
        result.is_err(),
        "fixture identity must not claim a digest different from retained bytes"
    );
}
