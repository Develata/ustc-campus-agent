use super::*;
fn store() -> (CalendarStore, PathBuf) {
    use std::os::unix::fs::PermissionsExt;
    let dir = std::env::temp_dir().join(format!(
        "uca-workspace-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("calendar workspace test fixture or assertion")
            .as_nanos()
    ));
    fs::create_dir(&dir).expect("calendar workspace test fixture or assertion");
    fs::set_permissions(&dir, fs::Permissions::from_mode(0o700))
        .expect("calendar workspace test fixture or assertion");
    (
        CalendarStore::open(dir.join("calendar.json"))
            .expect("calendar workspace test fixture or assertion"),
        dir,
    )
}
#[test]
fn batch_confirmation_is_atomic_scoped_and_replayed_after_restart() {
    let (mut store, dir) = store();
    let draft = CalendarDraft {
        title: "课程 A".into(),
        scheduled_for: "2026-09-07T08:00:00+08:00".into(),
    };
    let batch = store
        .propose_batch(
            "tenant/a",
            "request",
            vec![
                draft.clone(),
                CalendarDraft {
                    title: "课程 B".into(),
                    ..draft
                },
            ],
            100,
        )
        .expect("calendar workspace test fixture or assertion");
    assert!(
        store
            .items()
            .expect("calendar workspace test fixture or assertion")
            .is_empty()
    );
    assert!(
        store
            .finish_batch("tenant/b", &batch.id, true, 101)
            .is_err()
    );
    let receipt = store
        .finish_batch("tenant/a", &batch.id, true, 101)
        .expect("calendar workspace test fixture or assertion");
    assert_eq!(receipt.result.len(), 2);
    assert_eq!(
        store
            .reminders()
            .expect("calendar workspace test fixture or assertion")
            .len(),
        2
    );
    drop(store);
    let mut reopened = CalendarStore::open(dir.join("calendar.json"))
        .expect("calendar workspace test fixture or assertion");
    assert_eq!(
        reopened
            .finish_batch("tenant/a", &batch.id, true, 99999)
            .expect("calendar workspace test fixture or assertion"),
        receipt
    );
    assert_eq!(
        reopened
            .items()
            .expect("calendar workspace test fixture or assertion")
            .len(),
        2
    );
    fs::remove_dir_all(dir).expect("calendar workspace test fixture or assertion");
}
#[test]
fn reminders_deliver_once_persist_ack_and_cancel_obsolete_schedule() {
    let (mut store, dir) = store();
    let p = store
        .propose(
            "owner",
            "new",
            CalendarMutation::Record {
                title: "考试".into(),
                scheduled_for: Some("2026-09-07T08:00:00+08:00".into()),
            },
            100,
        )
        .expect("calendar workspace test fixture or assertion");
    let item = store
        .confirm_proposal("owner", &p.id, 101)
        .expect("calendar workspace test fixture or assertion")
        .result
        .expect("calendar workspace test fixture or assertion");
    let due = store
        .reminders()
        .expect("calendar workspace test fixture or assertion")[0]
        .due_unix_secs;
    assert_eq!(
        store
            .dispatch_reminders(due - 1)
            .expect("calendar workspace test fixture or assertion")[0]
            .status,
        ReminderStatus::Scheduled
    );
    let delivered = store
        .dispatch_reminders(due)
        .expect("calendar workspace test fixture or assertion");
    assert_eq!(
        store
            .dispatch_reminders(due + 100)
            .expect("calendar workspace test fixture or assertion"),
        delivered
    );
    let read = store
        .read_reminder(&delivered[0].id, due + 1)
        .expect("calendar workspace test fixture or assertion");
    assert_eq!(
        store
            .read_reminder(&read.id, due + 2)
            .expect("calendar workspace test fixture or assertion"),
        read
    );
    let update = store
        .propose(
            "owner",
            "update",
            CalendarMutation::Update {
                item_id: item.id.clone(),
                title: "新时间".into(),
                scheduled_for: Some("2026-09-08T08:00:00+08:00".into()),
            },
            due + 2,
        )
        .expect("calendar workspace test fixture or assertion");
    store
        .confirm_proposal("owner", &update.id, due + 3)
        .expect("calendar workspace test fixture or assertion");
    store
        .delete(&item.id)
        .expect("calendar workspace test fixture or assertion");
    assert_eq!(
        store
            .reminders()
            .expect("calendar workspace test fixture or assertion")[1]
            .status,
        ReminderStatus::Cancelled
    );
    drop(store);
    let mut reopened = CalendarStore::open(dir.join("calendar.json"))
        .expect("calendar workspace test fixture or assertion");
    assert_eq!(
        reopened
            .reminders()
            .expect("calendar workspace test fixture or assertion")[0],
        read
    );
    fs::remove_dir_all(dir).expect("calendar workspace test fixture or assertion");
}
#[test]
fn stale_batch_never_partially_records_items() {
    let (mut store, dir) = store();
    let b = store
        .propose_batch(
            "owner",
            "request",
            vec![CalendarDraft {
                title: "课程".into(),
                scheduled_for: "2026-09-07T08:00:00+08:00".into(),
            }],
            100,
        )
        .expect("calendar workspace test fixture or assertion");
    store
        .record("other", None)
        .expect("calendar workspace test fixture or assertion");
    assert_eq!(
        store.finish_batch("owner", &b.id, true, 101),
        Err(CalendarError::ProposalConflict)
    );
    assert_eq!(
        store
            .items()
            .expect("calendar workspace test fixture or assertion")
            .len(),
        1
    );
    assert!(
        store
            .reminders()
            .expect("calendar workspace test fixture or assertion")
            .is_empty()
    );
    fs::remove_dir_all(dir).expect("calendar workspace test fixture or assertion");
}

#[test]
fn negative_epoch_is_rejected_before_single_proposal_is_saved() {
    let (mut store, dir) = store();
    assert_eq!(
        store.propose(
            "owner",
            "request",
            CalendarMutation::Record {
                title: "invalid reminder epoch".into(),
                scheduled_for: Some("1969-12-31T23:59:59Z".into())
            },
            100
        ),
        Err(CalendarError::InvalidScheduledFor)
    );
    assert!(store.proposals("owner").expect("read proposals").is_empty());
    fs::remove_dir_all(dir).expect("cleanup");
}
