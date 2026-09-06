use super::*;
use std::sync::atomic::{AtomicU64, Ordering};
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "uca-calendar-proposals-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::SeqCst)
        ));
        fs::create_dir(&path).expect("isolated fixture");
        Self(path)
    }
    fn path(&self) -> PathBuf {
        self.0.join("calendar.json")
    }
    fn store(&self) -> CalendarStore {
        CalendarStore::open(self.path()).expect("open fixture")
    }
    fn replace(&self, bytes: &[u8]) {
        fs::write(self.path(), bytes).expect("fixture bytes");
        #[cfg(unix)]
        fs::set_permissions(self.path(), fs::Permissions::from_mode(0o600))
            .expect("private fixture");
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
const OWNER: &str = "tenant:demo/user:alice";
fn record(title: &str) -> CalendarMutation {
    CalendarMutation::Record {
        title: title.to_owned(),
        scheduled_for: Some("2026-09-10T09:00:00+08:00".to_owned()),
    }
}
#[test]
fn proposals_read_v1_without_rewrite_and_migrate_on_first_mutation() {
    let f = Fixture::new();
    let original = br#"{"schema":"ustc-simple-calendar-store/v1","next_id":2,"items":[{"id":"calendar:item:1","title":"legacy","scheduled_for":null,"created_at_unix_secs":1}]}"#;
    f.replace(original);
    let mut store = f.store();
    assert_eq!(store.items().expect("items").len(), 1);
    assert!(store.proposals(OWNER).expect("list").is_empty());
    assert_eq!(fs::read(f.path()).expect("unchanged"), original);
    let p = store
        .propose(OWNER, "request-1", record("new"), 100)
        .expect("propose");
    assert_eq!(p.base_revision, 0);
    assert_eq!(store.items().expect("no item effect").len(), 1);
    let bytes = fs::read(f.path()).expect("v2");
    assert!(String::from_utf8_lossy(&bytes).contains(storage::V2));
    drop(store);
    let mut reopened = f.store();
    assert_eq!(reopened.proposals(OWNER).expect("persisted"), vec![p]);
    assert_eq!(
        reopened.items().expect("legacy retained")[0].title,
        "legacy"
    );
}
#[test]
fn proposals_confirm_is_atomic_exact_and_replays_after_restart() {
    let f = Fixture::new();
    let mut store = f.store();
    let p = store
        .propose(OWNER, "same-key", record("dated"), 100)
        .expect("propose");
    let bytes = fs::read(f.path()).expect("pending");
    assert_eq!(
        store
            .propose(OWNER, "same-key", record("dated"), 999)
            .expect("exact retry"),
        p
    );
    assert_eq!(
        store.propose(OWNER, "same-key", record("changed"), 100),
        Err(CalendarError::ProposalConflict)
    );
    assert_eq!(fs::read(f.path()).expect("unchanged"), bytes);
    assert!(store.items().expect("pending no effects").is_empty());
    assert!(
        store
            .proposals("tenant:other/user:alice")
            .expect("scope")
            .is_empty()
    );
    assert_eq!(
        store.confirm_proposal("tenant:other/user:alice", &p.id, 101),
        Err(CalendarError::ProposalNotFound)
    );
    let applied = store.confirm_proposal(OWNER, &p.id, 101).expect("confirm");
    assert_eq!(applied.status, CalendarProposalStatus::Applied);
    assert_eq!(
        store.items().expect("applied"),
        std::slice::from_ref(applied.result.as_ref().expect("receipt"))
    );
    let bytes = fs::read(f.path()).expect("applied bytes");
    drop(store);
    let mut reopened = f.store();
    assert_eq!(
        reopened
            .confirm_proposal(OWNER, &p.id, u64::MAX)
            .expect("historical receipt"),
        applied
    );
    assert_eq!(
        reopened
            .propose(OWNER, "same-key", record("dated"), 10000)
            .expect("original proposal"),
        applied
    );
    assert_eq!(fs::read(f.path()).expect("no new write"), bytes);
    assert_eq!(reopened.state.item_revision, 1);
}
#[test]
fn proposals_update_delete_and_legacy_writes_obey_global_revision() {
    let f = Fixture::new();
    let mut store = f.store();
    let item = store
        .record("old", Some("2026-09-10T09:00:00+08:00"))
        .expect("legacy record");
    assert_eq!(store.state.item_revision, 1);
    let mutation = CalendarMutation::Update {
        item_id: item.id.clone(),
        title: "replacement".to_owned(),
        scheduled_for: None,
    };
    let update = store
        .propose(OWNER, "update", mutation, 100)
        .expect("update proposal");
    assert_eq!(update.before, Some(item.clone()));
    let applied = store
        .confirm_proposal(OWNER, &update.id, 101)
        .expect("update");
    assert_eq!(applied.result.as_ref().expect("result").scheduled_for, None);
    assert_eq!(
        applied
            .result
            .as_ref()
            .expect("result")
            .created_at_unix_secs,
        item.created_at_unix_secs
    );
    let stale = store
        .propose(
            OWNER,
            "stale",
            CalendarMutation::Delete {
                item_id: item.id.clone(),
            },
            102,
        )
        .expect("stale proposal");
    let extra = store.record("unrelated", None).expect("legacy mutation");
    assert_eq!(
        store.confirm_proposal(OWNER, &stale.id, 103),
        Err(CalendarError::ProposalConflict)
    );
    let stale_after_delete = store
        .propose(OWNER, "stale-delete", record("next"), 104)
        .expect("pending");
    store.delete(&extra.id).expect("legacy delete");
    assert_eq!(
        store.confirm_proposal(OWNER, &stale_after_delete.id, 105),
        Err(CalendarError::ProposalConflict)
    );
    let delete = store
        .propose(
            OWNER,
            "delete",
            CalendarMutation::Delete {
                item_id: item.id.clone(),
            },
            106,
        )
        .expect("delete proposal");
    let removed = store
        .confirm_proposal(OWNER, &delete.id, 107)
        .expect("delete");
    assert_eq!(removed.result, delete.before);
    assert!(store.items().expect("deleted").is_empty());
    assert_eq!(
        store
            .confirm_proposal(OWNER, &update.id, 108)
            .expect("old update receipt"),
        applied
    );
    drop(store);
    assert!(f.store().items().expect("reopen").is_empty());
}
#[test]
fn proposals_expiry_cancellation_and_clock_checks_preserve_items() {
    let f = Fixture::new();
    let mut store = f.store();
    let p = store
        .propose(OWNER, "expiry", record("dated"), 100)
        .expect("proposal");
    assert_eq!(p.expires_at_unix_secs, 1900);
    assert_eq!(
        store.confirm_proposal(OWNER, &p.id, 99),
        Err(CalendarError::ClockUnavailable)
    );
    assert_eq!(
        store.confirm_proposal(OWNER, &p.id, 1900),
        Err(CalendarError::ProposalExpired)
    );
    let cancelled = store
        .cancel_proposal(OWNER, &p.id, 1901)
        .expect("expired cancellation");
    assert_eq!(cancelled.status, CalendarProposalStatus::Cancelled);
    assert_eq!(
        store
            .cancel_proposal(OWNER, &p.id, 1902)
            .expect("cancel retry"),
        cancelled
    );
    assert_eq!(
        store.confirm_proposal(OWNER, &p.id, 1902),
        Err(CalendarError::ProposalConflict)
    );
    assert!(store.items().expect("no effects").is_empty());
    drop(store);
    assert_eq!(f.store().proposals(OWNER).expect("reopen"), vec![cancelled]);
}
#[test]
fn proposals_capacity_keeps_reserved_terminal_space_and_never_evicts() {
    let f = Fixture::new();
    let mut store = f.store();
    let title = "\\".repeat(MAX_TITLE_BYTES);
    let mut all = Vec::new();
    for i in 0..MAX_PROPOSALS {
        all.push(
            store
                .propose(OWNER, &format!("request-{i}"), record(&title), 100)
                .expect("bounded pending"),
        );
    }
    assert_eq!(
        store.propose(OWNER, "overflow", record("x"), 100),
        Err(CalendarError::ProposalLimitExceeded)
    );
    let applied = store
        .confirm_proposal(OWNER, &all[0].id, 101)
        .expect("reserved applied receipt");
    for p in &all[1..] {
        store
            .cancel_proposal(OWNER, &p.id, 102)
            .expect("reserved cancellation");
    }
    assert_eq!(
        store.proposals(OWNER).expect("all receipts").len(),
        MAX_PROPOSALS
    );
    assert_eq!(
        store
            .confirm_proposal(OWNER, &all[0].id, 10000)
            .expect("terminal retry at capacity"),
        applied
    );
    assert_eq!(
        store.propose(OWNER, "still-full", record("x"), 100),
        Err(CalendarError::ProposalLimitExceeded)
    );
    drop(store);
    assert_eq!(
        f.store().proposals(OWNER).expect("reopen").len(),
        MAX_PROPOSALS
    );
}
#[test]
fn proposals_reject_item_capacity_before_persisting_a_pending_record() {
    let f = Fixture::new();
    let mut store = f.store();
    for _ in 0..MAX_ITEMS {
        store.record("x", None).expect("bounded item");
    }
    let bytes = fs::read(f.path()).expect("full item store");
    assert_eq!(
        store.propose(OWNER, "full", record("x"), 100),
        Err(CalendarError::ItemLimitExceeded)
    );
    assert_eq!(fs::read(f.path()).expect("unchanged"), bytes);
    assert!(
        store
            .proposals(OWNER)
            .expect("no phantom proposal")
            .is_empty()
    );
}
#[test]
fn proposals_uncertain_confirmation_reconciles_without_second_effect() {
    let f = Fixture::new();
    let mut store = f.store();
    let p = store
        .propose(OWNER, "uncertain", record("x"), 100)
        .expect("proposal");
    store.fail_next_parent_sync_after_rename();
    store.fail_next_post_rename_readback();
    assert_eq!(
        store.confirm_proposal(OWNER, &p.id, 101),
        Err(CalendarError::PersistenceUnavailable)
    );
    let published = fs::read(f.path()).expect("published atomic state");
    store.fail_next_parent_sync_after_rename();
    assert_eq!(
        store.confirm_proposal(OWNER, &p.id, 102),
        Err(CalendarError::PersistenceUnavailable)
    );
    let receipt = store
        .confirm_proposal(OWNER, &p.id, 103)
        .expect("exact reconcile");
    assert_eq!(receipt.status, CalendarProposalStatus::Applied);
    assert_eq!(store.items().expect("one item").len(), 1);
    assert_eq!(fs::read(f.path()).expect("not repeated"), published);
    drop(store);
    assert_eq!(
        f.store()
            .confirm_proposal(OWNER, &p.id, 10000)
            .expect("restart receipt"),
        receipt
    );
}
#[test]
fn proposals_known_prewrite_failure_rolls_back_item_receipt_and_revision() {
    let f = Fixture::new();
    let mut store = f.store();
    let p = store
        .propose(OWNER, "fail", record("x"), 100)
        .expect("pending");
    let original_path = store.path.clone();
    store.path = f.0.join("absent-parent/calendar.json");
    assert_eq!(
        store.confirm_proposal(OWNER, &p.id, 101),
        Err(CalendarError::InvalidPath)
    );
    store.path = original_path;
    assert_eq!(store.state.item_revision, 0);
    assert!(store.items().expect("no item").is_empty());
    assert_eq!(
        store.proposals(OWNER).expect("pending retained"),
        vec![p.clone()]
    );
    assert_eq!(
        store
            .confirm_proposal(OWNER, &p.id, 102)
            .expect("retry")
            .status,
        CalendarProposalStatus::Applied
    );
}
#[test]
fn proposals_closed_wire_and_redundant_evidence_reject_tampering() {
    let f = Fixture::new();
    let mut store = f.store();
    let p = store
        .propose(OWNER, "proof", record("x"), 100)
        .expect("pending");
    store.confirm_proposal(OWNER, &p.id, 101).expect("apply");
    let raw: serde_json::Value =
        serde_json::from_slice(&fs::read(f.path()).expect("bytes")).expect("JSON");
    drop(store);
    for (path, replacement) in [
        ("/proposals/0/expires_at_unix_secs", serde_json::json!(999)),
        ("/proposals/0/base_revision", serde_json::json!(2)),
        ("/proposals/0/status", serde_json::json!("pending")),
        ("/proposals/0/result/title", serde_json::json!("forged")),
        ("/proposals/0/unknown", serde_json::json!(true)),
    ] {
        let mut changed = raw.clone();
        if path.ends_with("unknown") {
            changed["proposals"][0]["unknown"] = replacement;
        } else {
            *changed.pointer_mut(path).expect("known field") = replacement;
        }
        f.replace(&serde_json::to_vec(&changed).expect("JSON"));
        assert!(
            matches!(
                CalendarStore::open(f.path()),
                Err(CalendarError::InvalidStore)
            ),
            "{path}"
        );
    }
    let text = serde_json::to_string(&raw).expect("wire");
    let duplicate = text.replacen("\"action\":", "\"action\":\"delete\",\"action\":", 1);
    f.replace(duplicate.as_bytes());
    assert!(matches!(
        CalendarStore::open(f.path()),
        Err(CalendarError::InvalidStore)
    ));
    assert!(
        serde_json::from_str::<CalendarMutation>(
            r#"{"action":"update","item_id":"calendar:item:1","title":"x"}"#
        )
        .is_err()
    );
    assert!(
        serde_json::from_str::<CalendarMutation>(
            r#"{"action":"delete","item_id":"calendar:item:1","title":"x"}"#
        )
        .is_err()
    );
}

#[test]
fn proposals_require_bounded_subject_keys_and_explicit_timezone_before_writing() {
    let f = Fixture::new();
    let mut store = f.store();
    for time in [
        "tomorrow",
        "2026-09-10T09:00:00",
        "2026-02-30T09:00:00+08:00",
    ] {
        assert_eq!(
            store.propose(
                OWNER,
                "invalid-time",
                CalendarMutation::Record {
                    title: "x".to_owned(),
                    scheduled_for: Some(time.to_owned()),
                },
                100
            ),
            Err(CalendarError::InvalidScheduledFor)
        );
    }
    assert_eq!(
        store.propose("", "request", record("x"), 100),
        Err(CalendarError::InvalidProposal)
    );
    assert_eq!(
        store.propose(OWNER, &"x".repeat(257), record("x"), 100),
        Err(CalendarError::InvalidProposal)
    );
    assert_eq!(
        store.propose(OWNER, "overflow", record("x"), u64::MAX),
        Err(CalendarError::ClockUnavailable)
    );
    assert!(
        !f.path().exists(),
        "invalid input reserves no proposal or file"
    );
    let proposal = store
        .propose(OWNER, "valid", record("x"), 100)
        .expect("valid date");
    assert_eq!(
        store.cancel_proposal("tenant:demo/user:bob", &proposal.id, 101),
        Err(CalendarError::ProposalNotFound)
    );
    let applied = store
        .confirm_proposal(OWNER, &proposal.id, 1899)
        .expect("before expiry");
    assert_eq!(
        store.cancel_proposal(OWNER, &proposal.id, 1901),
        Err(CalendarError::ProposalConflict)
    );
    assert_eq!(
        applied.result.expect("receipt").scheduled_for.as_deref(),
        Some("2026-09-10T09:00:00+08:00")
    );
}
