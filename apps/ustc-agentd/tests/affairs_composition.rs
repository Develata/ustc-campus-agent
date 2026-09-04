#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use std::fs;
use std::os::unix::fs::{PermissionsExt, symlink};
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

use affairs_navigator::{ProcedurePublicationError, ProcedurePublicationRepositoryError};
use serde_json::{Value, json};
use ustc_agentd::AffairsComposition;
use ustc_campus_agent_application_ingress::{
    AffairsPublicationApplicationError, AffairsPublicationEvidenceError, AffairsPublicationOutcome,
};
use ustc_campus_agent_client_protocol::{
    ActorIntentDto, ClientErrorDto, ClientProvenanceDto, ClientResponseDto, M71OutcomeDto,
    RedactionDto, SubmitAffairsGetDto, UnixMillis, ViewerAuthorizationDto, WireErrorClassDto,
    WireText, affairs_get_payload_digest,
};
use ustc_campus_agent_core::request_context::CapabilityDisposition;

static COUNTER: AtomicU64 = AtomicU64::new(0);

fn temp_dir() -> PathBuf {
    let id = COUNTER.fetch_add(1, Ordering::SeqCst);
    let dir = std::env::temp_dir().join(format!("agentd-comp-{}-{id}", std::process::id()));
    fs::create_dir_all(&dir).expect("create temp dir");
    fs::set_permissions(&dir, fs::Permissions::from_mode(0o700)).expect("secure temp dir");
    dir
}

fn write_private_state(path: &std::path::Path, bytes: impl AsRef<[u8]>) {
    fs::write(path, bytes).expect("write private state");
    fs::set_permissions(path, fs::Permissions::from_mode(0o600)).expect("set private state mode");
}

fn base_fixture() -> Value {
    json!({
        "procedure_id": "proc:fixture",
        "title": "Fixture procedure",
        "known_at_secs": 50,
        "observed_at_secs": 40,
        "reviewed_at_secs": 160,
        "published_at_secs": 170,
        "last_verified_at_secs": 150,
        "max_fresh_seconds": 100,
        "max_presentable_seconds": 200,
        "source_id": "src:fixture",
        "source_url": "https://demo.example/affairs/fixture",
        "raw_snapshot_id": "raw:affairs:fixture:1",
        "raw_digest": "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "normalized_snapshot_id": "normalized:affairs:fixture:1",
        "normalized_digest": "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
        "parser_identity": "parser:affairs:fixture:v1",
        "source_published_at_secs": 30,
        "source_reviewer": "reviewer:demo:source",
        "source_review_evidence": "evidence:demo:source",
        "publication_reviewer": "actor:demo:administrator",
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

struct TestEnv {
    _dir: PathBuf,
    fixture: PathBuf,
    store: PathBuf,
    idempotency: PathBuf,
    sessions: PathBuf,
}

impl TestEnv {
    fn new() -> Self {
        Self::with_fixture(base_fixture())
    }

    fn with_fixture(fixture: Value) -> Self {
        let dir = temp_dir();
        let fixture_path = dir.join("fixture.json");
        let store_path = dir.join("store.json");
        let idempotency_path = dir.join("idempotency.json");
        let sessions_path = dir.join("sessions.json");
        fs::write(&fixture_path, fixture.to_string()).expect("write fixture");
        Self {
            _dir: dir,
            fixture: fixture_path,
            store: store_path,
            idempotency: idempotency_path,
            sessions: sessions_path,
        }
    }

    fn open(&self) -> AffairsComposition {
        AffairsComposition::open(
            &self.fixture,
            &self.store,
            &self.idempotency,
            &self.sessions,
        )
        .expect("composition open")
    }

    fn publication_state(&self) -> PathBuf {
        self.idempotency.with_extension("affairs-publication.json")
    }
}

fn wire(s: &str) -> WireText {
    WireText::parse(s).expect("valid wire text")
}

fn submit_request(
    procedure_id: &str,
    actor: ActorIntentDto,
    idempotency_key: Option<&str>,
) -> SubmitAffairsGetDto {
    let procedure_id_wire = wire(procedure_id);
    let payload_digest = affairs_get_payload_digest(&procedure_id_wire, None).expect("digest");
    SubmitAffairsGetDto {
        request_id: wire("req:fixture"),
        correlation_id: wire("corr:fixture"),
        causation_id: None,
        idempotency_key: idempotency_key.map(wire),
        actor,
        provenance: ClientProvenanceDto {
            build: wire("build:fixture"),
            target: wire("linux"),
            protocol: wire("m10:v2"),
        },
        payload_digest,
        procedure_id: procedure_id_wire,
        as_of: None,
    }
}

fn submit_request_with_digest(
    procedure_id: &str,
    actor: ActorIntentDto,
    idempotency_key: Option<&str>,
    digest: &str,
) -> SubmitAffairsGetDto {
    let mut req = submit_request(procedure_id, actor, idempotency_key);
    req.payload_digest = wire(digest);
    req
}

fn submit_request_as_of(
    procedure_id: &str,
    actor: ActorIntentDto,
    idempotency_key: Option<&str>,
    as_of_millis: i64,
) -> SubmitAffairsGetDto {
    let mut request = submit_request(procedure_id, actor, idempotency_key);
    let as_of = UnixMillis::new(as_of_millis);
    request.payload_digest =
        affairs_get_payload_digest(&request.procedure_id, Some(as_of)).expect("as-of digest");
    request.as_of = Some(as_of);
    request
}

fn authenticated_actor() -> ActorIntentDto {
    ActorIntentDto::Authenticated {
        session_id: wire("session:fixture"),
    }
}

fn public_actor() -> ActorIntentDto {
    ActorIntentDto::Public
}

fn extract_accepted(
    response: &ClientResponseDto,
) -> (
    &WireText,
    &ustc_campus_agent_client_protocol::M71TerminalDto,
    &Option<WireText>,
) {
    match response {
        ClientResponseDto::Accepted {
            command_id,
            terminal,
            public_capability,
        } => (command_id, terminal, public_capability),
        _ => panic!("expected Accepted, got {response:?}"),
    }
}

// ---------------------------------------------------------------------------
// Administrator publication: M10 -> M00/evidence -> direct M71 port
// ---------------------------------------------------------------------------

#[test]
fn administrator_publication_is_ordered_idempotent_and_restart_recoverable() {
    let env = TestEnv::new();
    let mut comp = env.open();
    assert_eq!(comp.current_publication_revision(), Some(1));
    assert_eq!(comp.control_evidence_event_count(), 0);
    assert_eq!(comp.publication_application_call_count(), 0);

    let first = comp.publish_demo_as_administrator();
    let AffairsPublicationOutcome::Published(first_receipt) = &first else {
        panic!("administrator publication must succeed, got {first:?}");
    };
    assert_eq!(first_receipt.expected_publication_revision(), Some(1));
    assert_eq!(first_receipt.publication_revision(), 2);
    assert_eq!(comp.current_publication_revision(), Some(2));
    assert_eq!(comp.control_evidence_event_count(), 1);
    assert_eq!(comp.publication_application_call_count(), 1);
    assert_eq!(comp.m60_call_count(), 1);

    let retry = comp.publish_demo_as_administrator();
    let AffairsPublicationOutcome::Published(retry_receipt) = &retry else {
        panic!("exact retry must replay publication, got {retry:?}");
    };
    assert_eq!(retry_receipt.receipt_id(), first_receipt.receipt_id());
    assert_eq!(comp.current_publication_revision(), Some(2));
    assert_eq!(comp.control_evidence_event_count(), 1);
    assert_eq!(comp.publication_application_call_count(), 2);
    assert_eq!(comp.m60_call_count(), 1);

    let query = submit_request(
        "proc:fixture",
        authenticated_actor(),
        Some("idem:post-publication-query"),
    );
    let response = comp.handle_submit(&query);
    let (_, terminal, _) = extract_accepted(&response);
    assert!(matches!(terminal.outcome(), M71OutcomeDto::Found { .. }));
    assert_eq!(comp.invocation_counts(), (1, 1, 1));
    assert_eq!(comp.current_publication_revision(), Some(2));

    let first_receipt_id = first_receipt.receipt_id().clone();
    drop(comp);
    let mut reopened = env.open();
    assert_eq!(reopened.current_publication_revision(), Some(2));
    assert_eq!(reopened.control_evidence_event_count(), 1);
    assert_eq!(reopened.publication_receipt_id(), first_receipt_id.as_str());
    assert_eq!(reopened.m60_call_count(), 0);
    let recovered = reopened.publish_demo_as_administrator();
    let AffairsPublicationOutcome::Published(recovered_receipt) = recovered else {
        panic!("restart retry must recover, got {recovered:?}");
    };
    assert_eq!(recovered_receipt.receipt_id(), &first_receipt_id);
    assert_eq!(reopened.current_publication_revision(), Some(2));
    assert_eq!(reopened.control_evidence_event_count(), 1);
    assert_eq!(reopened.publication_application_call_count(), 1);
    assert_eq!(reopened.m60_call_count(), 0);

    let response = reopened.handle_submit(&submit_request_as_of(
        "proc:fixture",
        authenticated_actor(),
        Some("idem:restart-publication-query"),
        199_000,
    ));
    let (_, terminal, _) = extract_accepted(&response);
    assert!(matches!(terminal.outcome(), M71OutcomeDto::Found { .. }));
    assert_eq!(reopened.invocation_counts(), (1, 1, 1));
    assert_eq!(reopened.m60_call_count(), 1);
}

#[test]
fn proc011_process_restart_helper() {
    let Ok(phase) = std::env::var("PROC011_PROCESS_PHASE") else {
        return;
    };
    let fixture = PathBuf::from(std::env::var_os("PROC011_FIXTURE").expect("fixture env"));
    let store = PathBuf::from(std::env::var_os("PROC011_STORE").expect("store env"));
    let idempotency =
        PathBuf::from(std::env::var_os("PROC011_IDEMPOTENCY").expect("idempotency env"));
    let sessions = PathBuf::from(std::env::var_os("PROC011_SESSIONS").expect("sessions env"));
    let receipt_path = PathBuf::from(std::env::var_os("PROC011_RECEIPT").expect("receipt env"));
    let mut composition =
        AffairsComposition::open(&fixture, &store, &idempotency, &sessions).expect("child open");

    match phase.as_str() {
        "publish" => {
            assert_eq!(composition.current_publication_revision(), Some(1));
            let AffairsPublicationOutcome::Published(receipt) =
                composition.publish_demo_as_administrator()
            else {
                panic!("child publication must succeed");
            };
            assert_eq!(receipt.publication_revision(), 2);
            assert_eq!(composition.m60_call_count(), 1);
            fs::write(receipt_path, receipt.receipt_id().as_str()).expect("write publish receipt");
        }
        "recover" => {
            assert_eq!(composition.current_publication_revision(), Some(2));
            assert_eq!(composition.control_evidence_event_count(), 1);
            assert_eq!(composition.m60_call_count(), 0);
            let AffairsPublicationOutcome::Published(receipt) =
                composition.publish_demo_as_administrator()
            else {
                panic!("child retry must replay");
            };
            assert_eq!(composition.m60_call_count(), 0);
            fs::write(receipt_path, receipt.receipt_id().as_str())
                .expect("write recovered receipt");

            let response = composition.handle_submit(&submit_request_as_of(
                "proc:fixture",
                authenticated_actor(),
                Some("idem:real-process-restart-query"),
                198_000,
            ));
            let (_, terminal, _) = extract_accepted(&response);
            assert!(matches!(terminal.outcome(), M71OutcomeDto::Found { .. }));
            assert_eq!(composition.invocation_counts(), (1, 1, 1));
            assert_eq!(composition.m60_call_count(), 1);
        }
        other => panic!("unknown child phase: {other}"),
    }
}

#[test]
fn administrator_publication_survives_real_process_restart() {
    let env = TestEnv::new();
    let publish_receipt = env._dir.join("publish-receipt.txt");
    let recovered_receipt = env._dir.join("recovered-receipt.txt");
    let executable = std::env::current_exe().expect("current integration-test executable");

    let run_phase = |phase: &str, receipt_path: &PathBuf| {
        let status = Command::new(&executable)
            .arg("--exact")
            .arg("proc011_process_restart_helper")
            .arg("--nocapture")
            .env("PROC011_PROCESS_PHASE", phase)
            .env("PROC011_FIXTURE", &env.fixture)
            .env("PROC011_STORE", &env.store)
            .env("PROC011_IDEMPOTENCY", &env.idempotency)
            .env("PROC011_SESSIONS", &env.sessions)
            .env("PROC011_RECEIPT", receipt_path)
            .status()
            .expect("spawn isolated process phase");
        assert!(status.success(), "child process phase {phase} failed");
    };

    run_phase("publish", &publish_receipt);
    run_phase("recover", &recovered_receipt);
    assert_eq!(
        fs::read_to_string(publish_receipt).expect("read publish receipt"),
        fs::read_to_string(recovered_receipt).expect("read recovered receipt")
    );
}

#[test]
fn publication_parent_sync_uncertainty_reconciles_before_memory_publish() {
    let env = TestEnv::new();
    let mut comp = env.open();
    comp.fail_next_publication_parent_sync_after_rename_for_test();

    let AffairsPublicationOutcome::Published(receipt) = comp.publish_demo_as_administrator() else {
        panic!("exact read-back must reconcile a post-rename parent-sync failure");
    };
    let receipt_id = receipt.receipt_id().clone();
    assert_eq!(comp.current_publication_revision(), Some(2));
    drop(comp);

    let mut reopened = env.open();
    assert_eq!(reopened.current_publication_revision(), Some(2));
    let AffairsPublicationOutcome::Published(replayed) = reopened.publish_demo_as_administrator()
    else {
        panic!("restart retry must replay the reconciled publication");
    };
    assert_eq!(replayed.receipt_id(), &receipt_id);
    assert_eq!(reopened.m60_call_count(), 0);
}

#[test]
fn publication_persistence_failure_is_not_visible_and_restart_keeps_prior_revision() {
    let env = TestEnv::new();
    let mut comp = env.open();
    comp.fail_next_publication_persistence_for_test();

    assert_eq!(
        comp.publish_demo_as_administrator(),
        AffairsPublicationOutcome::PublicationRejected(
            AffairsPublicationApplicationError::Downstream(ProcedurePublicationError::Repository(
                ProcedurePublicationRepositoryError::FailureInjected
            ))
        )
    );
    assert_eq!(comp.current_publication_revision(), Some(1));
    assert_eq!(comp.control_evidence_event_count(), 1);
    assert_eq!(comp.publication_application_call_count(), 1);
    assert_eq!(comp.m60_call_count(), 1);
    drop(comp);

    let mut reopened = env.open();
    assert_eq!(reopened.current_publication_revision(), Some(1));
    assert_eq!(reopened.control_evidence_event_count(), 1);
    assert_eq!(reopened.m60_call_count(), 0);
    assert!(matches!(
        reopened.publish_demo_as_administrator(),
        AffairsPublicationOutcome::Published(_)
    ));
    assert_eq!(reopened.current_publication_revision(), Some(2));
    assert_eq!(reopened.control_evidence_event_count(), 1);
    assert_eq!(reopened.m60_call_count(), 1);
}

#[test]
fn publication_state_corruption_matrix_fails_open_closed() {
    let env = TestEnv::new();
    let mut comp = env.open();
    assert!(matches!(
        comp.publish_demo_as_administrator(),
        AffairsPublicationOutcome::Published(_)
    ));
    drop(comp);

    let path = env.publication_state();
    let valid = fs::read(&path).expect("read valid publication state");
    let parsed: Value = serde_json::from_slice(&valid).expect("parse valid publication state");
    let mut corruptions = vec![
        b"{ malformed".to_vec(),
        serde_json::to_vec_pretty(&parsed).expect("pretty state"),
        vec![b'x'; 1_048_577],
    ];

    let mut unknown = parsed.clone();
    unknown["unknown_field"] = json!(true);
    corruptions.push(serde_json::to_vec(&unknown).expect("unknown-field state"));

    let mut duplicate = parsed.clone();
    duplicate["records"][1] = duplicate["records"][0].clone();
    corruptions.push(serde_json::to_vec(&duplicate).expect("duplicate state"));

    let mut gap = parsed.clone();
    gap["records"][1]["expected_publication_revision"] = json!(2);
    gap["records"][1]["publication_revision"] = json!(3);
    corruptions.push(serde_json::to_vec(&gap).expect("gapped state"));

    let mut reordered = parsed;
    reordered["records"]
        .as_array_mut()
        .expect("records array")
        .swap(0, 1);
    corruptions.push(serde_json::to_vec(&reordered).expect("reordered state"));

    for bytes in corruptions {
        write_private_state(&path, bytes);
        assert!(
            AffairsComposition::open(&env.fixture, &env.store, &env.idempotency, &env.sessions)
                .is_err(),
            "every malformed, noncanonical, oversized, duplicated, gapped, or reordered state must fail open"
        );
    }
    write_private_state(&path, valid);
    assert_eq!(env.open().current_publication_revision(), Some(2));
}

#[test]
fn every_missing_base_state_member_fails_closed() {
    for member in [
        "store.json",
        "idempotency.json",
        "sessions.json",
        "idempotency.affairs-publication.json",
        "idempotency.control-evidence.json",
        "idempotency.calendar-items.json",
    ] {
        let env = TestEnv::new();
        drop(env.open());
        fs::remove_file(env._dir.join(member)).expect("remove required base state member");

        let error = match AffairsComposition::open(
            &env.fixture,
            &env.store,
            &env.idempotency,
            &env.sessions,
        ) {
            Ok(_) => panic!("missing {member} must fail closed"),
            Err(error) => error,
        };
        assert_eq!(error, "durable_state_set_incomplete", "member={member}");
    }
}

#[test]
fn publication_primary_shape_and_parent_attacks_fail_closed() {
    let symlink_env = TestEnv::new();
    let sentinel = symlink_env._dir.join("publication-sentinel");
    write_private_state(&sentinel, b"sentinel");
    symlink(&sentinel, symlink_env.publication_state()).expect("publication symlink");
    assert!(
        AffairsComposition::open(
            &symlink_env.fixture,
            &symlink_env.store,
            &symlink_env.idempotency,
            &symlink_env.sessions,
        )
        .is_err()
    );
    assert_eq!(fs::read(&sentinel).expect("sentinel remains"), b"sentinel");

    let hardlink_env = TestEnv::new();
    let hardlink_target = hardlink_env._dir.join("publication-hardlink-target");
    write_private_state(&hardlink_target, b"{}");
    fs::hard_link(&hardlink_target, hardlink_env.publication_state())
        .expect("publication hardlink");
    assert!(
        AffairsComposition::open(
            &hardlink_env.fixture,
            &hardlink_env.store,
            &hardlink_env.idempotency,
            &hardlink_env.sessions,
        )
        .is_err()
    );

    let directory_env = TestEnv::new();
    fs::create_dir(directory_env.publication_state()).expect("publication directory");
    assert!(
        AffairsComposition::open(
            &directory_env.fixture,
            &directory_env.store,
            &directory_env.idempotency,
            &directory_env.sessions,
        )
        .is_err()
    );

    let fifo_env = TestEnv::new();
    let fifo_path = fifo_env.publication_state();
    assert!(
        Command::new("mkfifo")
            .arg(&fifo_path)
            .status()
            .expect("execute mkfifo")
            .success(),
        "create publication FIFO"
    );
    assert!(
        AffairsComposition::open(
            &fifo_env.fixture,
            &fifo_env.store,
            &fifo_env.idempotency,
            &fifo_env.sessions,
        )
        .is_err()
    );

    let wrong_mode_env = TestEnv::new();
    write_private_state(&wrong_mode_env.publication_state(), b"{}");
    fs::set_permissions(
        wrong_mode_env.publication_state(),
        fs::Permissions::from_mode(0o640),
    )
    .expect("set unsafe publication mode");
    assert!(
        AffairsComposition::open(
            &wrong_mode_env.fixture,
            &wrong_mode_env.store,
            &wrong_mode_env.idempotency,
            &wrong_mode_env.sessions,
        )
        .is_err()
    );

    let unsafe_parent_env = TestEnv::new();
    fs::set_permissions(&unsafe_parent_env._dir, fs::Permissions::from_mode(0o750))
        .expect("set unsafe state parent mode");
    assert!(
        AffairsComposition::open(
            &unsafe_parent_env.fixture,
            &unsafe_parent_env.store,
            &unsafe_parent_env.idempotency,
            &unsafe_parent_env.sessions,
        )
        .is_err()
    );
}

#[test]
fn runtime_publication_state_replacement_creates_no_new_publication_authority() {
    let env = TestEnv::new();
    let mut comp = env.open();
    write_private_state(&env.publication_state(), b"{}");

    assert_eq!(comp.current_publication_revision(), None);
    let query_response = comp.handle_submit(&submit_request_as_of(
        "proc:fixture",
        public_actor(),
        Some("idem:runtime-publication-corrupt"),
        199_000,
    ));
    assert!(
        matches!(
            query_response,
            ClientResponseDto::Error {
                error: ClientErrorDto::Infrastructure {
                    retryable: false,
                    ..
                }
            }
        ),
        "durable read corruption must be infrastructure failure, never semantic NotFound: {query_response:?}"
    );
    assert_eq!(comp.invocation_counts(), (1, 1, 1));
    assert!(matches!(
        comp.publish_demo_as_administrator(),
        AffairsPublicationOutcome::PublicationRejected(AffairsPublicationApplicationError::Denied)
    ));
    assert_eq!(comp.control_evidence_event_count(), 1);
    assert_eq!(comp.publication_application_call_count(), 1);
    assert_eq!(comp.m60_call_count(), 0);
    assert!(
        AffairsComposition::open(&env.fixture, &env.store, &env.idempotency, &env.sessions)
            .is_err()
    );
}

#[test]
fn disabled_publication_is_rejected_before_evidence_or_m71() {
    let env = TestEnv::new();
    let mut comp = env.open();
    comp.set_publication_capability(CapabilityDisposition::Disabled);

    let outcome = comp.publish_demo_as_administrator();

    assert!(matches!(outcome, AffairsPublicationOutcome::Rejected(_)));
    assert_eq!(comp.control_evidence_event_count(), 0);
    assert_eq!(comp.publication_application_call_count(), 0);
    assert_eq!(comp.m60_call_count(), 0);
    assert_eq!(comp.current_publication_revision(), Some(1));
}

#[test]
fn evidence_destination_attack_blocks_before_m71() {
    let env = TestEnv::new();
    let mut comp = env.open();
    let target = env._dir.join("evidence-sentinel");
    write_private_state(&target, b"sentinel");
    let evidence_path = env.idempotency.with_extension("control-evidence.json");
    fs::remove_file(&evidence_path).expect("remove canonical evidence state");
    symlink(&target, &evidence_path).expect("install evidence symlink");

    let outcome = comp.publish_demo_as_administrator();

    assert_eq!(
        outcome,
        AffairsPublicationOutcome::EvidenceRejected(AffairsPublicationEvidenceError::Corrupt)
    );
    assert_eq!(fs::read(&target).expect("read sentinel"), b"sentinel");
    assert_eq!(comp.publication_application_call_count(), 0);
    assert_eq!(comp.m60_call_count(), 0);
    assert_eq!(comp.current_publication_revision(), Some(1));
}

#[test]
fn wrong_fixture_administrator_identity_denies_before_m60_or_repository_mutation() {
    let mut fixture = base_fixture();
    fixture["publication_administrator_user_id"] = json!("user:different-administrator");
    let env = TestEnv::with_fixture(fixture);
    let mut comp = env.open();

    let outcome = comp.publish_demo_as_administrator();

    assert!(matches!(
        outcome,
        AffairsPublicationOutcome::PublicationRejected(_)
    ));
    assert_eq!(comp.control_evidence_event_count(), 1);
    assert_eq!(comp.publication_application_call_count(), 1);
    assert_eq!(comp.m60_call_count(), 0);
    assert_eq!(comp.current_publication_revision(), Some(1));
}

// ---------------------------------------------------------------------------
// Authenticated Found with Verified lineage and exactly one M71/M60 call
// ---------------------------------------------------------------------------

#[test]
fn authenticated_found_one_m71_call() {
    let env = TestEnv::new();
    let comp = env.open();
    assert!(
        comp.publication_receipt_id()
            .starts_with("publication:sha256:"),
        "composition must mint a reviewed publication receipt before serving queries"
    );
    assert_eq!(
        comp.m60_call_count(),
        0,
        "publication verification is a separate M60 authority call, not an M71 query call"
    );

    let request = submit_request("proc:fixture", authenticated_actor(), Some("idem:auth"));
    let response = comp.handle_submit(&request);

    let (command_id, terminal, public_capability) = extract_accepted(&response);
    assert!(matches!(terminal.outcome(), M71OutcomeDto::Found { .. }));
    assert!(
        public_capability.is_none(),
        "authenticated submit must not mint public capability"
    );
    assert!(!command_id.as_str().is_empty());
    assert_eq!(
        comp.m60_call_count(),
        1,
        "exactly one M60 verify_retained call"
    );
    assert_eq!(
        comp.invocation_counts(),
        (1, 1, 1),
        "intent must precede one owning-plugin execution and one receipt"
    );
}

#[test]
fn disabled_market_installation_denies_without_plugin_execution_or_direct_fallback() {
    let mut fixture = base_fixture();
    fixture["market_enabled"] = json!(false);
    let env = TestEnv::with_fixture(fixture);
    let comp = env.open();
    let request = submit_request(
        "proc:fixture",
        authenticated_actor(),
        Some("idem:market-deny"),
    );

    let response = comp.handle_submit(&request);

    match response {
        ClientResponseDto::Error {
            error: ClientErrorDto::Admission { error },
        } => assert_eq!(error.class, WireErrorClassDto::PolicyDenied),
        _ => panic!("expected stable Market denial, got {response:?}"),
    }
    assert_eq!(comp.invocation_counts(), (0, 0, 0));
    assert_eq!(
        comp.m60_call_count(),
        0,
        "denial must not fall back to direct M71"
    );
}

#[test]
fn revoked_market_grant_denies_without_plugin_execution_or_direct_fallback() {
    let mut fixture = base_fixture();
    fixture["market_grant_active"] = json!(false);
    let env = TestEnv::with_fixture(fixture);
    let comp = env.open();
    let request = submit_request(
        "proc:fixture",
        authenticated_actor(),
        Some("idem:grant-deny"),
    );

    let response = comp.handle_submit(&request);

    match response {
        ClientResponseDto::Error {
            error: ClientErrorDto::Admission { error },
        } => assert_eq!(error.class, WireErrorClassDto::PolicyDenied),
        _ => panic!("expected stable Market grant denial, got {response:?}"),
    }
    assert_eq!(comp.invocation_counts(), (0, 0, 0));
    assert_eq!(
        comp.m60_call_count(),
        0,
        "revoked grant must not fall back to direct M71"
    );
}

// ---------------------------------------------------------------------------
// Public capability exact lookup
// ---------------------------------------------------------------------------

#[test]
fn public_capability_exact_lookup() {
    let env = TestEnv::new();
    let comp = env.open();

    let request = submit_request("proc:fixture", public_actor(), Some("idem:pub"));
    let response = comp.handle_submit(&request);
    let (command_id, _, public_capability) = extract_accepted(&response);
    let bearer = public_capability
        .as_ref()
        .expect("public submit mints bearer");

    let viewer = ViewerAuthorizationDto::PublicCapability {
        capability: bearer.clone(),
    };
    let lookup = comp.handle_lookup(command_id.as_str(), &viewer);
    match lookup {
        ClientResponseDto::Available {
            redaction: RedactionDto::Public,
            ..
        } => {}
        _ => panic!("expected Available/Public, got {lookup:?}"),
    }
}

// ---------------------------------------------------------------------------
// Indistinguishable denials: missing/wrong/truncated/all-zero/random bearer
// ---------------------------------------------------------------------------

#[test]
fn public_capability_indistinguishable_denials() {
    let env = TestEnv::new();
    let comp = env.open();

    let request = submit_request("proc:fixture", public_actor(), Some("idem:pub"));
    let response = comp.handle_submit(&request);
    let (command_id, _, _) = extract_accepted(&response);

    let wrong_bearers = [
        "deadbeef",
        "0000000000000000000000000000000000000000000000000000000000000000",
        "cafecafecafecafecafecafecafecafecafecafecafecafecafecafecafecafe",
        "wrong-bearer",
    ];
    for wrong in &wrong_bearers {
        let viewer = ViewerAuthorizationDto::PublicCapability {
            capability: wire(wrong),
        };
        let lookup = comp.handle_lookup(command_id.as_str(), &viewer);
        assert!(
            matches!(lookup, ClientResponseDto::Unavailable),
            "wrong bearer `{wrong}` must be indistinguishable Unavailable, got {lookup:?}"
        );
    }
}

// ---------------------------------------------------------------------------
// Authenticated owner lookup succeeds; wrong tenant/user fails
// ---------------------------------------------------------------------------

#[test]
fn authenticated_owner_lookup_succeeds() {
    let env = TestEnv::new();
    let comp = env.open();

    let request = submit_request("proc:fixture", authenticated_actor(), Some("idem:auth"));
    let response = comp.handle_submit(&request);
    let (command_id, _, _) = extract_accepted(&response);

    let viewer = ViewerAuthorizationDto::AuthenticatedOwner {
        tenant_id: wire("tenant:fixture"),
        user_id: wire("user:fixture"),
    };
    let lookup = comp.handle_lookup(command_id.as_str(), &viewer);
    match lookup {
        ClientResponseDto::Available {
            redaction: RedactionDto::AuthenticatedOwner,
            ..
        } => {}
        _ => panic!("expected Available/AuthenticatedOwner, got {lookup:?}"),
    }
}

#[test]
fn authenticated_owner_wrong_tenant_rejected() {
    let env = TestEnv::new();
    let comp = env.open();

    let request = submit_request("proc:fixture", authenticated_actor(), Some("idem:auth"));
    let response = comp.handle_submit(&request);
    let (command_id, _, _) = extract_accepted(&response);

    let viewer = ViewerAuthorizationDto::AuthenticatedOwner {
        tenant_id: wire("tenant:WRONG"),
        user_id: wire("user:fixture"),
    };
    let lookup = comp.handle_lookup(command_id.as_str(), &viewer);
    assert!(
        matches!(lookup, ClientResponseDto::Unavailable),
        "wrong tenant must be Unavailable, got {lookup:?}"
    );
}

#[test]
fn authenticated_owner_wrong_user_rejected() {
    let env = TestEnv::new();
    let comp = env.open();

    let request = submit_request("proc:fixture", authenticated_actor(), Some("idem:auth"));
    let response = comp.handle_submit(&request);
    let (command_id, _, _) = extract_accepted(&response);

    let viewer = ViewerAuthorizationDto::AuthenticatedOwner {
        tenant_id: wire("tenant:fixture"),
        user_id: wire("user:WRONG"),
    };
    let lookup = comp.handle_lookup(command_id.as_str(), &viewer);
    assert!(
        matches!(lookup, ClientResponseDto::Unavailable),
        "wrong user must be Unavailable, got {lookup:?}"
    );
}

// ---------------------------------------------------------------------------
// Wrong actor (wrong session_id) rejected at admission
// ---------------------------------------------------------------------------

#[test]
fn authenticated_wrong_session_rejected() {
    let env = TestEnv::new();
    let comp = env.open();

    let actor = ActorIntentDto::Authenticated {
        session_id: wire("session:WRONG"),
    };
    let request = submit_request("proc:fixture", actor, Some("idem:wrong-session"));
    let response = comp.handle_submit(&request);

    match response {
        ClientResponseDto::Error {
            error: ClientErrorDto::Admission { error },
        } => {
            assert_eq!(
                error.class,
                WireErrorClassDto::SessionNotFound,
                "wrong session must be SessionNotFound"
            );
        }
        _ => panic!("expected Admission error, got {response:?}"),
    }
    assert_eq!(
        comp.m60_call_count(),
        0,
        "rejected submit must not call M71"
    );
}

// ---------------------------------------------------------------------------
// Operator exact lookup succeeds; wrong grant fails
// ---------------------------------------------------------------------------

#[test]
fn operator_exact_lookup_succeeds() {
    let env = TestEnv::new();
    let comp = env.open();

    let request = submit_request("proc:fixture", authenticated_actor(), Some("idem:auth"));
    let response = comp.handle_submit(&request);
    let (command_id, _, _) = extract_accepted(&response);

    let viewer = ViewerAuthorizationDto::Operator {
        grant_id: wire("operator:fixture"),
    };
    let lookup = comp.handle_lookup(command_id.as_str(), &viewer);
    match lookup {
        ClientResponseDto::Available {
            redaction: RedactionDto::Operator,
            ..
        } => {}
        _ => panic!("expected Available/Operator, got {lookup:?}"),
    }
}

#[test]
fn operator_wrong_grant_rejected() {
    let env = TestEnv::new();
    let comp = env.open();

    let request = submit_request("proc:fixture", authenticated_actor(), Some("idem:auth"));
    let response = comp.handle_submit(&request);
    let (command_id, _, _) = extract_accepted(&response);

    let viewer = ViewerAuthorizationDto::Operator {
        grant_id: wire("operator:WRONG"),
    };
    let lookup = comp.handle_lookup(command_id.as_str(), &viewer);
    assert!(
        matches!(lookup, ClientResponseDto::Unavailable),
        "wrong grant must be Unavailable, got {lookup:?}"
    );
}

// ---------------------------------------------------------------------------
// M60 failure/corruption maps to typed Infrastructure error with no raw-ref leak
// ---------------------------------------------------------------------------

#[test]
fn m60_store_unavailable_maps_to_infrastructure_error() {
    let mut fixture = base_fixture();
    fixture["m60_failure_mode"] = json!("store_unavailable");
    let env = TestEnv::with_fixture(fixture);
    let comp = env.open();

    let request = submit_request("proc:fixture", public_actor(), Some("idem:m60-fail"));
    let response = comp.handle_submit(&request);

    match response {
        ClientResponseDto::Error {
            error:
                ClientErrorDto::Infrastructure {
                    retryable,
                    wire_code,
                },
        } => {
            assert!(retryable, "M60StoreUnavailable should be retryable");
            assert!(
                !wire_code.as_str().contains("rev:"),
                "wire_code must not leak raw revision refs"
            );
            assert!(
                !wire_code.as_str().contains("src:"),
                "wire_code must not leak source ids"
            );
        }
        _ => panic!("expected Infrastructure error, got {response:?}"),
    }
    assert_eq!(
        comp.invocation_counts(),
        (1, 1, 1),
        "an attempted failing Plugin call must still persist intent and failure receipt"
    );
}

#[test]
fn m60_store_corrupted_maps_to_infrastructure_error() {
    let mut fixture = base_fixture();
    fixture["m60_failure_mode"] = json!("store_corrupted");
    let env = TestEnv::with_fixture(fixture);
    let comp = env.open();

    let request = submit_request("proc:fixture", public_actor(), Some("idem:m60-corrupt"));
    let response = comp.handle_submit(&request);

    match response {
        ClientResponseDto::Error {
            error: ClientErrorDto::Infrastructure { retryable, .. },
        } => {
            assert!(!retryable, "M60StoreCorrupted should NOT be retryable");
        }
        _ => panic!("expected Infrastructure error, got {response:?}"),
    }
    assert_eq!(
        comp.invocation_counts(),
        (1, 1, 1),
        "a corrupted-source failure still requires one exact failure receipt"
    );
}

// ---------------------------------------------------------------------------
// Publication bootstrap identity and fail-closed admission
// ---------------------------------------------------------------------------

#[test]
fn source_url_changes_publication_receipt_identity() {
    let first_env = TestEnv::with_fixture(base_fixture());
    let first = first_env.open();
    let mut changed_fixture = base_fixture();
    changed_fixture["source_url"] = json!("https://demo.example/affairs/fixture-v2");
    let changed_env = TestEnv::with_fixture(changed_fixture);
    let changed = changed_env.open();

    assert_ne!(
        first.publication_receipt_id(),
        changed.publication_receipt_id(),
        "canonical source URL must remain bound into draft/review/publication identity"
    );
}

#[test]
fn unresolved_conflict_cannot_bootstrap_reviewed_publication() {
    let mut fixture = base_fixture();
    fixture["conflict_state"] = json!("unresolved_conflict");
    fixture["authority_comparison"] = json!("incomparable");
    fixture["conflict_kind"] = json!("direct_contradiction");
    let env = TestEnv::with_fixture(fixture);
    let result =
        AffairsComposition::open(&env.fixture, &env.store, &env.idempotency, &env.sessions);
    let Err(error) = result else {
        panic!("unresolved evidence must fail before repository publication");
    };
    assert!(
        error.contains("procedure draft invalid"),
        "unexpected publication rejection: {error}"
    );
}

// ---------------------------------------------------------------------------
// Malformed fixture/schema/key/operator/id grammar fails closed
// ---------------------------------------------------------------------------

#[test]
fn malformed_json_fails_closed() {
    let dir = temp_dir();
    let fixture_path = dir.join("fixture.json");
    fs::write(&fixture_path, "{ not valid json").expect("write fixture");
    let result = AffairsComposition::open(
        &fixture_path,
        &dir.join("store.json"),
        &dir.join("idempotency.json"),
        &dir.join("sessions.json"),
    );
    assert!(result.is_err(), "malformed JSON must fail closed");
}

#[test]
fn invalid_schema_digest_fails_closed() {
    let mut fixture = base_fixture();
    fixture["schema_digest"] = json!("not-hex");
    let env = TestEnv::with_fixture(fixture);
    let result =
        AffairsComposition::open(&env.fixture, &env.store, &env.idempotency, &env.sessions);
    assert!(result.is_err(), "invalid schema_digest must fail closed");
}

#[test]
fn invalid_capability_key_fails_closed() {
    let mut fixture = base_fixture();
    fixture["capability_key_hex"] = json!("too-short");
    let env = TestEnv::with_fixture(fixture);
    let result =
        AffairsComposition::open(&env.fixture, &env.store, &env.idempotency, &env.sessions);
    assert!(
        result.is_err(),
        "invalid capability_key_hex must fail closed"
    );
}

#[test]
fn empty_operator_grant_id_fails_closed() {
    let mut fixture = base_fixture();
    fixture["operator_grant_id"] = json!("");
    let env = TestEnv::with_fixture(fixture);
    let result =
        AffairsComposition::open(&env.fixture, &env.store, &env.idempotency, &env.sessions);
    assert!(result.is_err(), "empty operator_grant_id must fail closed");
}

#[test]
fn invalid_procedure_id_in_fixture_fails_closed() {
    let mut fixture = base_fixture();
    fixture["procedure_id"] = json!("INVALID-UPPERCASE");
    let env = TestEnv::with_fixture(fixture);
    let result =
        AffairsComposition::open(&env.fixture, &env.store, &env.idempotency, &env.sessions);
    assert!(result.is_err(), "invalid procedure_id must fail closed");
}

// ---------------------------------------------------------------------------
// Loopback bind policy: rejects wildcard / non-loopback / unparseable
// ---------------------------------------------------------------------------

#[test]
fn loopback_rejects_wildcard_ipv4() {
    let env = TestEnv::new();
    let comp = env.open();
    let result = comp.serve("0.0.0.0:0");
    assert!(result.is_err(), "wildcard 0.0.0.0:0 must be rejected");
    let msg = result.unwrap_err();
    assert!(
        msg.contains("loopback"),
        "error should mention loopback: {msg}"
    );
}

#[test]
fn loopback_rejects_non_loopback_ipv4() {
    let env = TestEnv::new();
    let comp = env.open();
    let result = comp.serve("8.8.8.8:8080");
    assert!(
        result.is_err(),
        "non-loopback 8.8.8.8:8080 must be rejected"
    );
}

#[test]
fn loopback_rejects_non_loopback_ipv6() {
    let env = TestEnv::new();
    let comp = env.open();
    let result = comp.serve("[2001:4860:4860::8888]:8080");
    assert!(result.is_err(), "non-loopback IPv6 must be rejected");
}

#[test]
fn loopback_rejects_unparseable() {
    let env = TestEnv::new();
    let comp = env.open();
    let result = comp.serve("not_a_valid_address");
    assert!(result.is_err(), "unparseable address must be rejected");
}

// ---------------------------------------------------------------------------
// Canonical digest mismatch executes zero M71
// ---------------------------------------------------------------------------

#[test]
fn digest_mismatch_zero_m71() {
    let env = TestEnv::new();
    let comp = env.open();

    let wrong_digest = "0000000000000000000000000000000000000000000000000000000000000000";
    let request = submit_request_with_digest(
        "proc:fixture",
        public_actor(),
        Some("idem:bad-digest"),
        wrong_digest,
    );
    let response = comp.handle_submit(&request);

    match response {
        ClientResponseDto::Error {
            error: ClientErrorDto::Admission { error },
        } => {
            assert_eq!(
                error.class,
                WireErrorClassDto::MalformedCommand,
                "digest mismatch must be MalformedCommand"
            );
        }
        _ => panic!("expected Admission/MalformedCommand, got {response:?}"),
    }
    assert_eq!(
        comp.m60_call_count(),
        0,
        "digest mismatch must not call M71"
    );
}

// ---------------------------------------------------------------------------
// Response-loss recovery: drop/reopen same terminal/capability, no second M71
// ---------------------------------------------------------------------------

#[test]
fn response_loss_recovery_same_terminal_no_second_m71() {
    let env = TestEnv::new();

    // First composition: submit and get terminal
    let comp1 = env.open();
    let request = submit_request("proc:fixture", public_actor(), Some("idem:retry"));
    let response1 = comp1.handle_submit(&request);
    let (command_id1, terminal1, bearer1) = extract_accepted(&response1);
    assert_eq!(comp1.m60_call_count(), 1, "first submit calls M71 once");
    let command_id_str = command_id1.as_str().to_owned();
    let bearer1_clone = bearer1.clone();

    // Drop composition (simulate response loss + process restart)
    drop(comp1);

    // Reopen on same durable files
    let comp2 = env.open();

    // Identical retry returns same terminal
    let response2 = comp2.handle_submit(&request);
    let (command_id2, terminal2, bearer2) = extract_accepted(&response2);

    assert_eq!(
        command_id1, command_id2,
        "retry must return same command_id"
    );
    assert_eq!(terminal1, terminal2, "retry must return same terminal");
    assert_eq!(
        bearer1_clone,
        bearer2.clone(),
        "retry must reproduce same public capability"
    );
    assert_eq!(
        comp2.m60_call_count(),
        0,
        "retry on reopened composition must not call M71 again"
    );

    // Lookup proves durable terminal record exists
    let viewer = ViewerAuthorizationDto::Operator {
        grant_id: wire("operator:fixture"),
    };
    let lookup = comp2.handle_lookup(&command_id_str, &viewer);
    assert!(
        matches!(lookup, ClientResponseDto::Available { .. }),
        "durable terminal record must be available via lookup"
    );
}

// ---------------------------------------------------------------------------
// Unkeyed submits are fresh attempts, including after process restart
// ---------------------------------------------------------------------------

#[test]
fn unkeyed_submit_re_evaluates_after_restart() {
    let env = TestEnv::new();
    let request = submit_request("proc:fixture", public_actor(), None);

    let comp1 = env.open();
    let response1 = comp1.handle_submit(&request);
    let (command_id1, _, _) = extract_accepted(&response1);
    let command_id1 = command_id1.clone();
    assert_eq!(comp1.m60_call_count(), 1, "first unkeyed submit calls M71");
    drop(comp1);

    let original_fixture = fs::read(&env.fixture).expect("read original fixture");
    let mut updated_fixture = base_fixture();
    updated_fixture["title"] = json!("Updated fixture procedure");
    fs::write(&env.fixture, updated_fixture.to_string()).expect("update fixture");

    let blocked =
        AffairsComposition::open(&env.fixture, &env.store, &env.idempotency, &env.sessions);
    let error = blocked.err().expect("draft drift must fail durable open");
    assert!(error.contains("bound to a different fixture draft"));
    fs::write(&env.fixture, original_fixture).expect("restore exact bound fixture");

    let comp2 = env.open();
    let response2 = comp2.handle_submit(&request);
    let (command_id2, _, _) = extract_accepted(&response2);

    assert_ne!(
        &command_id1, command_id2,
        "an unkeyed retry must mint a fresh command identity"
    );
    assert_eq!(
        comp2.m60_call_count(),
        1,
        "an unkeyed retry after restart must re-evaluate current M71/source state"
    );
}

// ---------------------------------------------------------------------------
// Same idempotency key with different actual payload is rejected, no extra M71
// ---------------------------------------------------------------------------

#[test]
fn same_key_different_payload_rejected_no_extra_m71() {
    let env = TestEnv::new();
    let comp = env.open();

    // First submit with proc:fixture
    let request1 = submit_request("proc:fixture", public_actor(), Some("idem:conflict"));
    let response1 = comp.handle_submit(&request1);
    assert!(matches!(response1, ClientResponseDto::Accepted { .. }));
    assert_eq!(comp.m60_call_count(), 1, "first submit calls M71 once");

    // Second submit with proc:other (different payload) but same key
    let request2 = submit_request("proc:other", public_actor(), Some("idem:conflict"));
    let response2 = comp.handle_submit(&request2);

    match response2 {
        ClientResponseDto::Error {
            error: ClientErrorDto::Admission { error },
        } => {
            assert_eq!(
                error.class,
                WireErrorClassDto::ConflictingEnvelope,
                "same key different payload must be ConflictingEnvelope"
            );
        }
        _ => panic!("expected Admission/ConflictingEnvelope, got {response2:?}"),
    }
    assert_eq!(
        comp.m60_call_count(),
        1,
        "conflicting envelope must not call M71 again"
    );
}

// ---------------------------------------------------------------------------
// Duplicate submit on same composition returns same terminal, no second M71
// ---------------------------------------------------------------------------

#[test]
fn duplicate_submit_same_terminal_no_second_m71() {
    let env = TestEnv::new();
    let comp = env.open();

    let request = submit_request("proc:fixture", public_actor(), Some("idem:dup"));
    let response1 = comp.handle_submit(&request);
    let (_, terminal1, _) = extract_accepted(&response1);
    assert_eq!(comp.m60_call_count(), 1, "first submit calls M71 once");

    let response2 = comp.handle_submit(&request);
    let (_, terminal2, _) = extract_accepted(&response2);
    assert_eq!(terminal1, terminal2, "duplicate must return same terminal");
    assert_eq!(
        comp.m60_call_count(),
        1,
        "duplicate must not call M71 again"
    );
}

// ---------------------------------------------------------------------------
// Public submit mints capability; authenticated does not
// ---------------------------------------------------------------------------

#[test]
fn public_submit_mints_capability_authenticated_does_not() {
    let env = TestEnv::new();
    let comp = env.open();

    let pub_request = submit_request("proc:fixture", public_actor(), Some("idem:pub-cap"));
    let pub_response = comp.handle_submit(&pub_request);
    let (_, _, pub_cap) = extract_accepted(&pub_response);
    assert!(pub_cap.is_some(), "public submit mints capability");

    let auth_request = submit_request("proc:fixture", authenticated_actor(), Some("idem:auth-cap"));
    let auth_response = comp.handle_submit(&auth_request);
    let (_, _, auth_cap) = extract_accepted(&auth_response);
    assert!(
        auth_cap.is_none(),
        "authenticated submit does not mint public capability"
    );
}

// ---------------------------------------------------------------------------
// Reserve persist failure leaves no phantom: read-only dir blocks persist,
// retry after restoring write succeeds with New (not InFlight)
// ---------------------------------------------------------------------------

#[test]
fn reserve_persist_failure_leaves_no_phantom() {
    use std::os::unix::fs::PermissionsExt;

    let env = TestEnv::new();
    let comp = env.open();
    let idempotency_before = fs::read(&env.idempotency).expect("read canonical empty state");

    let parent = env.idempotency.parent().unwrap();
    let original_mode = fs::metadata(parent).unwrap().permissions().mode();
    fs::set_permissions(parent, fs::Permissions::from_mode(0o555)).unwrap();

    let request = submit_request("proc:fixture", public_actor(), Some("idem:phantom"));
    let response = comp.handle_submit(&request);

    match response {
        ClientResponseDto::Error {
            error: ClientErrorDto::Admission { error },
        } => {
            assert_eq!(
                error.class,
                WireErrorClassDto::IdempotencyStoreUnavailable,
                "persist failure must map to IdempotencyStoreUnavailable, got {:?}",
                error.class
            );
        }
        _ => panic!("expected IdempotencyStoreUnavailable, got {response:?}"),
    }

    assert_eq!(
        fs::read(&env.idempotency).expect("read state after failed reserve"),
        idempotency_before,
        "persist failure must leave the canonical empty state unchanged"
    );

    fs::set_permissions(parent, fs::Permissions::from_mode(original_mode)).unwrap();

    let response2 = comp.handle_submit(&request);
    match response2 {
        ClientResponseDto::Accepted { .. } => {}
        ClientResponseDto::Incomplete { .. } => {
            panic!(
                "retry must not return Incomplete — that would mean a phantom reservation was published"
            );
        }
        _ => panic!("expected Accepted on retry, got {response2:?}"),
    }

    assert_eq!(
        comp.m60_call_count(),
        1,
        "retry after persist failure must call M71 exactly once"
    );
}

// ---------------------------------------------------------------------------
// Capability is command-scoped: bearer A cannot look up command B
// ---------------------------------------------------------------------------

fn submit_request_with_causation(
    causation_id: &str,
    procedure_id: &str,
    actor: ActorIntentDto,
    idempotency_key: Option<&str>,
) -> SubmitAffairsGetDto {
    let mut req = submit_request(procedure_id, actor, idempotency_key);
    req.causation_id = Some(wire(causation_id));
    req
}

#[test]
fn capability_denial_cross_command_bearer_rejected() {
    let env = TestEnv::new();
    let comp = env.open();

    let request_a = submit_request_with_causation(
        "corr:cmd-a",
        "proc:fixture",
        public_actor(),
        Some("idem:cmd-a"),
    );
    let response_a = comp.handle_submit(&request_a);
    let (command_id_a, _, bearer_a) = extract_accepted(&response_a);
    let bearer_a = bearer_a.clone().expect("public submit mints bearer");

    let request_b = submit_request_with_causation(
        "corr:cmd-b",
        "proc:fixture",
        public_actor(),
        Some("idem:cmd-b"),
    );
    let response_b = comp.handle_submit(&request_b);
    let (command_id_b, _, bearer_b) = extract_accepted(&response_b);
    let bearer_b = bearer_b.clone().expect("public submit mints bearer");

    assert_ne!(
        command_id_a, command_id_b,
        "different request_ids must produce different command_ids"
    );
    assert_ne!(
        bearer_a, bearer_b,
        "different command_ids must mint different bearers"
    );

    let viewer_a_on_b = ViewerAuthorizationDto::PublicCapability {
        capability: bearer_a.clone(),
    };
    let lookup = comp.handle_lookup(command_id_b.as_str(), &viewer_a_on_b);
    assert!(
        matches!(lookup, ClientResponseDto::Unavailable),
        "bearer A must not unlock command B, got {lookup:?}"
    );

    let viewer_b_on_a = ViewerAuthorizationDto::PublicCapability {
        capability: bearer_b.clone(),
    };
    let lookup = comp.handle_lookup(command_id_a.as_str(), &viewer_b_on_a);
    assert!(
        matches!(lookup, ClientResponseDto::Unavailable),
        "bearer B must not unlock command A, got {lookup:?}"
    );
}

// ---------------------------------------------------------------------------
// Absent command with valid bearer returns Unavailable (not an error)
// ---------------------------------------------------------------------------

#[test]
fn absent_command_with_valid_bearer_returns_unavailable() {
    let env = TestEnv::new();
    let comp = env.open();

    let request = submit_request("proc:fixture", public_actor(), Some("idem:absent"));
    let response = comp.handle_submit(&request);
    let (_, _, bearer) = extract_accepted(&response);
    let bearer = bearer.clone().expect("public submit mints bearer");

    let absent_command_id = "0".repeat(64);
    let viewer = ViewerAuthorizationDto::PublicCapability {
        capability: bearer.clone(),
    };
    let lookup = comp.handle_lookup(&absent_command_id, &viewer);
    assert!(
        matches!(lookup, ClientResponseDto::Unavailable),
        "absent command must return Unavailable, got {lookup:?}"
    );
}

// ---------------------------------------------------------------------------
// Empty bearer rejected at WireText::parse level
// ---------------------------------------------------------------------------

#[test]
fn empty_bearer_rejected_at_wire_text_parse() {
    let result = WireText::parse("");
    assert!(
        result.is_err(),
        "empty string must not parse as WireText, got {:?}",
        result.ok()
    );
}

// ---------------------------------------------------------------------------
// Idempotency validation fail-closed via AffairsComposition::open
// ---------------------------------------------------------------------------

fn write_idempotency(json: &str) -> TestEnv {
    let env = TestEnv::with_fixture(base_fixture());
    write_private_state(&env.idempotency, json);
    env
}

#[test]
fn idempotency_malformed_json_fails_closed() {
    let env = TestEnv::with_fixture(base_fixture());
    write_private_state(&env.idempotency, "not valid json");
    let result =
        AffairsComposition::open(&env.fixture, &env.store, &env.idempotency, &env.sessions);
    assert!(
        result.is_err(),
        "malformed idempotency JSON must fail closed"
    );
}

#[test]
fn idempotency_wrong_schema_version_fails_closed() {
    let cmd = "abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789";
    let json = json!({
        "schema_version": 2,
        "entries": {
            cmd: {
                "reservation_version": 1,
                "fencing_token": 1,
                "deadline_ms": 1000000002000_u64,
                "in_flight": false,
                "disposition": {
                    "kind": "rejected",
                    "value": { "kind": "malformed_command", "operation_id": null }
                }
            }
        },
        "key_index": { "idem:valid": cmd }
    });
    let env = write_idempotency(&json.to_string());
    let result =
        AffairsComposition::open(&env.fixture, &env.store, &env.idempotency, &env.sessions);
    assert!(result.is_err(), "wrong schema_version must fail closed");
}

#[test]
fn idempotency_zero_deadline_ms_fails_closed() {
    let cmd = "abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789";
    let json = json!({
        "schema_version": 1,
        "entries": {
            cmd: {
                "reservation_version": 1,
                "fencing_token": 1,
                "deadline_ms": 0_u64,
                "in_flight": true,
                "disposition": null
            }
        },
        "key_index": {}
    });
    let env = write_idempotency(&json.to_string());
    let result =
        AffairsComposition::open(&env.fixture, &env.store, &env.idempotency, &env.sessions);
    assert!(result.is_err(), "zero deadline_ms must fail closed");
}

#[test]
fn idempotency_zero_fencing_token_fails_closed() {
    let cmd = "abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789";
    let json = json!({
        "schema_version": 1,
        "entries": {
            cmd: {
                "reservation_version": 1,
                "fencing_token": 0_u64,
                "deadline_ms": 1000000002000_u64,
                "in_flight": true,
                "disposition": null
            }
        },
        "key_index": {}
    });
    let env = write_idempotency(&json.to_string());
    let result =
        AffairsComposition::open(&env.fixture, &env.store, &env.idempotency, &env.sessions);
    assert!(result.is_err(), "zero fencing_token must fail closed");
}

#[test]
fn idempotency_dangling_key_index_fails_closed() {
    let cmd = "abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789";
    let dangling = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
    let json = json!({
        "schema_version": 1,
        "entries": {
            cmd: {
                "reservation_version": 1,
                "fencing_token": 1,
                "deadline_ms": 1000000002000_u64,
                "in_flight": false,
                "disposition": {
                    "kind": "rejected",
                    "value": { "kind": "malformed_command", "operation_id": null }
                }
            }
        },
        "key_index": { "idem:dangling": dangling }
    });
    let env = write_idempotency(&json.to_string());
    let result =
        AffairsComposition::open(&env.fixture, &env.store, &env.idempotency, &env.sessions);
    assert!(result.is_err(), "dangling key_index must fail closed");
}

#[test]
fn idempotency_in_flight_with_disposition_fails_closed() {
    let cmd = "abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789";
    let json = json!({
        "schema_version": 1,
        "entries": {
            cmd: {
                "reservation_version": 1,
                "fencing_token": 1,
                "deadline_ms": 1000000002000_u64,
                "in_flight": true,
                "disposition": {
                    "kind": "rejected",
                    "value": { "kind": "malformed_command", "operation_id": null }
                }
            }
        },
        "key_index": { "idem:valid": cmd }
    });
    let env = write_idempotency(&json.to_string());
    let result =
        AffairsComposition::open(&env.fixture, &env.store, &env.idempotency, &env.sessions);
    assert!(
        result.is_err(),
        "in_flight with disposition must fail closed"
    );
}

// ---------------------------------------------------------------------------
// Bind loopback helper: IPv4 and IPv6 zero-port bind succeeds
// ---------------------------------------------------------------------------

#[test]
fn bind_loopback_ipv4_zero_port_succeeds() {
    use ustc_agentd::bind_loopback;
    let listener = bind_loopback("127.0.0.1:0");
    assert!(listener.is_ok(), "IPv4 loopback bind must succeed");
    let listener = listener.unwrap();
    let local = listener.local_addr().unwrap();
    assert!(local.ip().is_loopback(), "bound address must be loopback");
}

#[test]
fn bind_loopback_ipv6_zero_port_succeeds_or_env_unsupported() {
    use ustc_agentd::bind_loopback;
    let listener = bind_loopback("[::1]:0");
    match listener {
        Ok(listener) => {
            let local = listener.local_addr().unwrap();
            assert!(
                local.ip().is_loopback(),
                "bound IPv6 address must be loopback"
            );
        }
        Err(msg) => {
            let lowered = msg.to_ascii_lowercase();
            let recognized = lowered.contains("address family")
                || lowered.contains("address not available")
                || lowered.contains("not supported")
                || lowered.contains("protocol not supported")
                || lowered.contains("no such device")
                || lowered.contains("eafnosupport")
                || lowered.contains("enodev")
                || lowered.contains("eprotonosupport");
            assert!(
                recognized,
                "IPv6 bind failure must be a recognized unsupported/address-family error, got: {msg}"
            );
        }
    }
}
