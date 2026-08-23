#![allow(clippy::unwrap_used)]

mod common;

use affairs_navigator::{FixedClock, InMemoryAffairsRepository, m60_fixture::M60FixtureAdapter};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use ustc_campus_agent_application_ingress::{FileRecordStore, RecordState, StoreError};
use ustc_campus_agent_client_protocol::{ClientResponseDto, ViewerAuthorizationDto, WireText};

use common::{FakePorts, M71FixturePort, cap_issuer, submit_request, t, temp_path};

fn make_m71_fixture() -> (InMemoryAffairsRepository, M60FixtureAdapter, FixedClock) {
    let repo = InMemoryAffairsRepository::new();
    let m60 = M60FixtureAdapter::new("verifier:fixture", 1).unwrap();
    let clock = FixedClock::new(t(200));
    (repo, m60, clock)
}

// ---------------------------------------------------------------------------
// Public API recovery: reopen persists terminal state across store instances
// ---------------------------------------------------------------------------

#[test]
fn reopen_persists_terminal_state_across_store_instances() {
    let path = temp_path();
    let procedure_id = "proc:missing";
    {
        let store = FileRecordStore::open(path.clone()).unwrap();
        let (repo, m60, clock) = make_m71_fixture();
        let m71 = M71FixturePort::new(&repo, &m60, &clock);
        let service = ustc_campus_agent_application_ingress::M10Service::new(
            store,
            cap_issuer(),
            &m71,
            ustc_campus_agent_client_protocol::WireText::parse("operator:fixture").unwrap(),
        );
        let mut ports = FakePorts::public_admitted();
        let request = submit_request(procedure_id);
        let response = service.submit(&request, &mut ports, 1_000_000);
        assert!(
            matches!(
                response,
                ustc_campus_agent_client_protocol::ClientResponseDto::Accepted { .. }
            ),
            "submit must succeed"
        );
    }
    {
        let store = FileRecordStore::open(path).unwrap();
        let record = store
            .test_get("command:fixture")
            .unwrap()
            .expect("record must persist across reopen");
        assert!(
            matches!(record.state, RecordState::Terminal { .. }),
            "record must be Terminal after reopen, got {:?}",
            record.state
        );
    }
}

#[test]
fn get_returns_none_for_missing() {
    let store = FileRecordStore::open(temp_path()).unwrap();
    assert!(store.test_get("nonexistent").unwrap().is_none());
}

// ---------------------------------------------------------------------------
// R4: JSON mutation tests — validate_state / validate_record reject corruption
// ---------------------------------------------------------------------------

/// Seed a valid store with one terminal record via M10Service::submit, then
/// return the file path and the parsed JSON value for mutation.
fn seed_valid_store() -> (std::path::PathBuf, serde_json::Value) {
    let path = temp_path();
    {
        let store = FileRecordStore::open(path.clone()).unwrap();
        let (repo, m60, clock) = make_m71_fixture();
        let m71 = M71FixturePort::new(&repo, &m60, &clock);
        let service = ustc_campus_agent_application_ingress::M10Service::new(
            store,
            cap_issuer(),
            &m71,
            ustc_campus_agent_client_protocol::WireText::parse("operator:fixture").unwrap(),
        );
        let mut ports = FakePorts::public_admitted();
        let request = submit_request("proc:missing");
        let _response = service.submit(&request, &mut ports, 1_000_000);
    }
    let bytes = std::fs::read(&path).unwrap();
    let value: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    (path, value)
}

/// Write the mutated JSON back to the path and attempt to reopen.
fn reopen_expect_invariant(path: &std::path::Path, value: serde_json::Value) {
    let bytes = serde_json::to_vec(&value).unwrap();
    std::fs::write(path, &bytes).unwrap();
    let result = FileRecordStore::open(path);
    match result {
        Err(StoreError::Invariant) | Err(StoreError::Corrupted(_)) => {}
        Err(other) => panic!("expected Invariant or Corrupted, got {other}"),
        Ok(_) => panic!("expected rejection, but store opened successfully"),
    }
}

#[test]
fn r4_wrong_schema_version_rejected() {
    let (path, mut value) = seed_valid_store();
    value["schema_version"] = serde_json::json!(2);
    reopen_expect_invariant(&path, value);
}

#[test]
fn r4_pending_wrong_version_nonzero_highest_fencing_rejected() {
    let (path, mut value) = seed_valid_store();
    let record = &mut value["records"]["command:fixture"];
    record["state"] = serde_json::json!({
        "kind": "pending",
        "version": 0,
        "highest_fencing": 5,
    });
    reopen_expect_invariant(&path, value);
}

#[test]
fn r4_pending_version_below_highest_fencing_rejected() {
    let (path, mut value) = seed_valid_store();
    let record = &mut value["records"]["command:fixture"];
    record["state"] = serde_json::json!({
        "kind": "pending",
        "version": 2,
        "highest_fencing": 5,
    });
    reopen_expect_invariant(&path, value);
}

#[test]
fn r4_claimed_highest_fencing_ne_fencing_token_rejected() {
    let (path, mut value) = seed_valid_store();
    let record = &mut value["records"]["command:fixture"];
    record["state"] = serde_json::json!({
        "kind": "claimed",
        "version": 3,
        "highest_fencing": 2,
        "fencing_token": 3,
        "lease_deadline_ms": 1_030_000,
    });
    reopen_expect_invariant(&path, value);
}

#[test]
fn r4_claimed_version_below_highest_fencing_rejected() {
    let (path, mut value) = seed_valid_store();
    let record = &mut value["records"]["command:fixture"];
    record["state"] = serde_json::json!({
        "kind": "claimed",
        "version": 2,
        "highest_fencing": 5,
        "fencing_token": 5,
        "lease_deadline_ms": 1_030_000,
    });
    reopen_expect_invariant(&path, value);
}

#[test]
fn r4_terminal_highest_fencing_ne_completion_fencing_token_rejected() {
    let (path, mut value) = seed_valid_store();
    let record = &mut value["records"]["command:fixture"];
    let terminal = record["state"]["terminal"].clone();
    let completion = record["state"]["completion"].clone();
    record["state"] = serde_json::json!({
        "kind": "terminal",
        "version": 3,
        "highest_fencing": 2,
        "terminal": terminal,
        "completion": completion,
    });
    reopen_expect_invariant(&path, value);
}

#[test]
fn r4_terminal_version_below_highest_fencing_rejected() {
    let (path, mut value) = seed_valid_store();
    let record = &mut value["records"]["command:fixture"];
    let terminal = record["state"]["terminal"].clone();
    let completion = record["state"]["completion"].clone();
    let hf = record["state"]["highest_fencing"].as_u64().unwrap();
    record["state"] = serde_json::json!({
        "kind": "terminal",
        "version": hf - 1,
        "highest_fencing": hf,
        "terminal": terminal,
        "completion": completion,
    });
    reopen_expect_invariant(&path, value);
}

#[test]
fn r4_terminal_digest_mismatch_rejected() {
    let (path, mut value) = seed_valid_store();
    let record = &mut value["records"]["command:fixture"];
    record["state"]["completion"]["terminal_digest"] =
        serde_json::json!("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb");
    reopen_expect_invariant(&path, value);
}

#[test]
fn r4_terminal_digest_uppercase_hex_rejected() {
    let (path, mut value) = seed_valid_store();
    let record = &mut value["records"]["command:fixture"];
    let terminal = record["state"]["terminal"].clone();
    let mut completion = record["state"]["completion"].clone();
    completion["terminal_digest"] =
        serde_json::json!("AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA");
    let hf = record["state"]["highest_fencing"].clone();
    let version = record["state"]["version"].clone();
    record["state"] = serde_json::json!({
        "kind": "terminal",
        "version": version,
        "highest_fencing": hf,
        "terminal": terminal,
        "completion": completion,
    });
    reopen_expect_invariant(&path, value);
}

#[test]
fn r4_capsule_digest_mismatch_rejected() {
    let (path, mut value) = seed_valid_store();
    let record = &mut value["records"]["command:fixture"];
    record["capsule_digest"] =
        serde_json::json!("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb");
    reopen_expect_invariant(&path, value);
}

#[test]
fn r4_capsule_digest_uppercase_hex_rejected() {
    let (path, mut value) = seed_valid_store();
    let record = &mut value["records"]["command:fixture"];
    record["capsule_digest"] =
        serde_json::json!("AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA");
    reopen_expect_invariant(&path, value);
}

#[test]
fn r4_public_authorization_key_version_zero_rejected() {
    let (path, mut value) = seed_valid_store();
    let record = &mut value["records"]["command:fixture"];
    record["read_policy"]["authorization"]["key_version"] = serde_json::json!(0);
    reopen_expect_invariant(&path, value);
}

#[test]
fn r4_public_authorization_digest_uppercase_hex_rejected() {
    let (path, mut value) = seed_valid_store();
    let record = &mut value["records"]["command:fixture"];
    record["read_policy"]["authorization"]["digest_hex"] =
        serde_json::json!("AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA");
    reopen_expect_invariant(&path, value);
}

#[test]
fn r4_command_id_mismatch_between_capsule_and_record_key_rejected() {
    let (path, mut value) = seed_valid_store();
    let record = value["records"]["command:fixture"].clone();
    value["records"]["command:DIFFERENT"] = record;
    value["records"]
        .as_object_mut()
        .unwrap()
        .remove("command:fixture");
    reopen_expect_invariant(&path, value);
}

#[test]
fn r4_actor_policy_mismatch_public_actor_with_authenticated_policy_rejected() {
    let (path, mut value) = seed_valid_store();
    let record = &mut value["records"]["command:fixture"];
    record["read_policy"] = serde_json::json!({
        "kind": "authenticated",
        "tenant_id": "tenant:fixture",
        "user_id": "user:fixture",
    });
    reopen_expect_invariant(&path, value);
}

#[test]
fn r4_corrupt_json_rejected() {
    let path = temp_path();
    std::fs::write(&path, b"not valid json").unwrap();
    let result = FileRecordStore::open(&path);
    assert!(matches!(result, Err(StoreError::Corrupted(_))));
}

#[test]
fn r4_empty_file_rejected() {
    let path = temp_path();
    std::fs::write(&path, b"").unwrap();
    let result = FileRecordStore::open(&path);
    assert!(matches!(result, Err(StoreError::Corrupted(_))));
}

struct CountingM71Port<'a> {
    inner: M71FixturePort<'a>,
    count: Arc<AtomicU64>,
}

impl<'a> CountingM71Port<'a> {
    fn new(
        repo: &'a InMemoryAffairsRepository,
        m60: &'a M60FixtureAdapter,
        clock: &'a FixedClock,
        count: Arc<AtomicU64>,
    ) -> Self {
        Self {
            inner: M71FixturePort::new(repo, m60, clock),
            count,
        }
    }
}

impl<'a> affairs_navigator::M71AffairsGetPort for CountingM71Port<'a> {
    fn affairs_get(
        &self,
        query: &affairs_navigator::AffairsGetQuery,
    ) -> Result<affairs_navigator::M71AffairsGetReceipt, affairs_navigator::GetProcedureError> {
        self.count.fetch_add(1, Ordering::SeqCst);
        self.inner.affairs_get(query)
    }
}

#[test]
fn s2_response_loss_recovery_retries_to_identical_terminal_with_one_m71_call() {
    let path = temp_path();
    let repo = InMemoryAffairsRepository::new();
    let m60 = M60FixtureAdapter::new("verifier:fixture", 1).unwrap();
    let clock = FixedClock::new(t(200));
    let m71_call_count = Arc::new(AtomicU64::new(0));
    let operator_grant = WireText::parse("operator:fixture").unwrap();

    let (terminal_first, capability_first) = {
        let store = FileRecordStore::open(path.clone()).unwrap();
        let m71 = CountingM71Port::new(&repo, &m60, &clock, Arc::clone(&m71_call_count));
        let service = ustc_campus_agent_application_ingress::M10Service::new(
            store,
            cap_issuer(),
            &m71,
            operator_grant.clone(),
        );
        let mut ports = FakePorts::public_admitted();
        let request = submit_request("proc:missing");
        let response = service.submit(&request, &mut ports, 1_000_000);
        match response {
            ClientResponseDto::Accepted {
                terminal,
                public_capability,
                ..
            } => (terminal, public_capability),
            _ => panic!("expected Accepted on first submit, got {response:?}"),
        }
    };

    assert_eq!(
        m71_call_count.load(Ordering::SeqCst),
        1,
        "first submit must call M71 exactly once"
    );

    let (terminal_retry, capability_retry) = {
        let store = FileRecordStore::open(path.clone()).unwrap();
        let m71 = CountingM71Port::new(&repo, &m60, &clock, Arc::clone(&m71_call_count));
        let service = ustc_campus_agent_application_ingress::M10Service::new(
            store,
            cap_issuer(),
            &m71,
            operator_grant.clone(),
        );
        let mut ports = FakePorts::public_admitted();
        let request = submit_request("proc:missing");
        let response = service.submit(&request, &mut ports, 1_000_000);
        match response {
            ClientResponseDto::Accepted {
                terminal,
                public_capability,
                ..
            } => (terminal, public_capability),
            _ => panic!("expected Accepted on retry, got {response:?}"),
        }
    };

    assert_eq!(
        m71_call_count.load(Ordering::SeqCst),
        1,
        "retry must not call M71 — total must remain 1"
    );
    assert_eq!(
        terminal_first, terminal_retry,
        "terminal must be identical across response-loss recovery"
    );
    assert_eq!(
        capability_first, capability_retry,
        "public capability must be reproducible across response-loss recovery"
    );

    let store = FileRecordStore::open(&path).unwrap();
    let lookup_response = {
        let m71 = CountingM71Port::new(&repo, &m60, &clock, Arc::clone(&m71_call_count));
        let service = ustc_campus_agent_application_ingress::M10Service::new(
            store,
            cap_issuer(),
            &m71,
            operator_grant,
        );
        service.lookup(
            "command:fixture",
            &ViewerAuthorizationDto::Operator {
                grant_id: WireText::parse("operator:fixture").unwrap(),
            },
        )
    };
    match lookup_response {
        ClientResponseDto::Available { terminal, .. } => {
            assert_eq!(
                terminal, terminal_first,
                "lookup terminal must match the original accepted terminal"
            );
        }
        _ => panic!("expected Available on operator lookup, got {lookup_response:?}"),
    }
    assert_eq!(
        m71_call_count.load(Ordering::SeqCst),
        1,
        "lookup must not call M71"
    );

    let record_count = std::fs::read_to_string(&path)
        .map(|content| {
            let value: serde_json::Value = serde_json::from_str(&content).unwrap();
            value["records"].as_object().unwrap().len()
        })
        .unwrap();
    assert_eq!(
        record_count, 1,
        "exactly one terminal record must exist in the store"
    );
}
