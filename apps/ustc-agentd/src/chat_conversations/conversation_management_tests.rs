use super::*;
use serde_json::{Value, json};
use std::{fs, os::unix::fs::PermissionsExt};

struct Fixture {
    root: PathBuf,
    owner: (TenantId, UserId),
}
impl Fixture {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "uca-conversation-management-{}",
            persistence::random_id().expect("nonce")
        ));
        fs::create_dir(&root).expect("directory");
        fs::set_permissions(&root, fs::Permissions::from_mode(0o700)).expect("private directory");
        Self {
            root,
            owner: (
                TenantId::parse("tenant:manage").expect("tenant"),
                UserId::parse("user:alice").expect("user"),
            ),
        }
    }
    fn path(&self) -> PathBuf {
        self.root.join("conversations.json")
    }
    fn store(&self) -> ConversationStore {
        ConversationStore::open(self.path()).expect("store")
    }
    fn create(&self, store: &ConversationStore) -> ConversationDto {
        store
            .create(&self.owner.0, &self.owner.1, "create")
            .expect("create")
    }
    fn manage(
        &self,
        store: &ConversationStore,
        id: &str,
        intent: ConversationManageIntentDto,
    ) -> Result<ConversationManageResultDto, ConversationError> {
        store.manage(&self.owner.0, &self.owner.1, id, intent)
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}
fn rename(id: &str, revision: u64, title: &str) -> ConversationManageIntentDto {
    ConversationManageIntentDto {
        schema: "chat-conversation-manage/v1".into(),
        request_id: id.into(),
        expected_revision: revision,
        action: ConversationManageActionDto::Rename {
            title: title.into(),
        },
    }
}
fn delete(id: &str, revision: u64) -> ConversationManageIntentDto {
    ConversationManageIntentDto {
        schema: "chat-conversation-manage/v1".into(),
        request_id: id.into(),
        expected_revision: revision,
        action: ConversationManageActionDto::Delete {},
    }
}
fn turn(id: &str, revision: u64) -> ConversationTurnIntentDto {
    serde_json::from_value(json!({"schema":"chat-conversation-turn/v1","request_id":id,"expected_revision":revision,"message":"Original automatic title"})).expect("turn intent")
}
fn finish(
    f: &Fixture,
    store: &ConversationStore,
    id: &str,
    request_id: &str,
) -> ConversationTurnResultDto {
    store
        .finish(
            &f.owner.0,
            &f.owner.1,
            id,
            request_id,
            Ok(ChatResponseDto {
                schema: "ustc-agent-chat-response/v1",
                run_id: "chat-run:management-test".into(),
                answer: "Retained answer".into(),
                provider: crate::chat_provider::ChatProvider::deterministic_mock().identity(),
                tool_trace: Vec::new(),
                usage: crate::agent_chat::ChatUsageDto::default(),
            }),
        )
        .expect("finish")
}
#[test]
fn rename_interleaved_turns_delete_and_exact_receipts_survive_restart_without_resurrection() {
    let f = Fixture::new();
    let store = f.store();
    let c = f.create(&store);
    let first = rename("rename-first", 0, "  Explicit first title  ");
    let first_receipt = f
        .manage(&store, &c.id, first.clone())
        .expect("rename empty");
    assert_eq!(first_receipt.title, "Explicit first title");
    assert_eq!(first_receipt.revision, 1);
    let original_turn = turn("turn-first", 1);
    assert!(matches!(
        store.begin(&f.owner.0, &f.owner.1, &c.id, original_turn.clone(), false),
        Ok(BeginTurn::New { .. })
    ));
    let first_result = finish(&f, &store, &c.id, "turn-first");
    assert_eq!(first_result.revision, 3);
    assert_eq!(
        store
            .get(&f.owner.0, &f.owner.1, &c.id)
            .expect("view")
            .title,
        "Explicit first title"
    );
    let second = rename("rename-second", 3, "Second title");
    assert_eq!(
        f.manage(&store, &c.id, second)
            .expect("rename between turns")
            .revision,
        4
    );
    assert!(matches!(
        store.begin(&f.owner.0, &f.owner.1, &c.id, turn("turn-second", 4), false),
        Ok(BeginTurn::New { .. })
    ));
    finish(&f, &store, &c.id, "turn-second");
    let Ok(BeginTurn::Replay(replay)) =
        store.begin(&f.owner.0, &f.owner.1, &c.id, original_turn.clone(), false)
    else {
        panic!("first turn replay")
    };
    assert_eq!(
        serde_json::to_value(replay).expect("replay"),
        serde_json::to_value(first_result).expect("original")
    );
    drop(store);
    let store = f.store();
    let deletion = delete("delete", 6);
    let deleted = f.manage(&store, &c.id, deletion.clone()).expect("delete");
    assert_eq!(deleted.revision, 7);
    assert!(deleted.deleted);
    assert_eq!(
        f.manage(&store, &c.id, first.clone())
            .expect("historical rename receipt"),
        first_receipt
    );
    for result in [
        store.get(&f.owner.0, &f.owner.1, &c.id),
        store.create(&f.owner.0, &f.owner.1, "create"),
    ] {
        assert!(matches!(result, Err(ConversationError::NotFound)));
    }
    assert!(matches!(
        store.current_turn(&f.owner.0, &f.owner.1, &c.id),
        Err(ConversationError::NotFound)
    ));
    assert!(matches!(
        store.begin_admitted(
            &f.owner.0,
            &f.owner.1,
            &c.id,
            original_turn,
            false,
            |_| panic!("deleted must never select model")
        ),
        Err(ConversationError::NotFound)
    ));
    assert!(
        store
            .list(&f.owner.0, &f.owner.1)
            .expect("hidden list")
            .conversations
            .is_empty()
    );
    assert_eq!(
        f.manage(
            &store,
            &c.id,
            rename("rename-first", 0, "Explicit first title")
        )
        .expect_err("raw submitted title changed"),
        ConversationError::RequestConflict
    );
    assert_eq!(
        f.manage(&store, &c.id, rename("new-management", 7, "Resurrect"))
            .expect_err("terminal deletion"),
        ConversationError::NotFound
    );
    let bytes = fs::read(f.path()).expect("retained bytes");
    assert!(String::from_utf8_lossy(&bytes).contains("Retained answer"));
    drop(store);
    let store = f.store();
    assert_eq!(
        f.manage(&store, &c.id, deletion).expect("delete replay"),
        deleted
    );
    assert_eq!(
        f.manage(&store, &c.id, first).expect("old rename replay"),
        first_receipt
    );
    assert!(
        store
            .list(&f.owner.0, &f.owner.1)
            .expect("still hidden")
            .conversations
            .is_empty()
    );
    assert_eq!(fs::read(f.path()).expect("read only replays"), bytes);
}
#[test]
fn owner_validation_revision_busy_and_request_namespaces_fail_without_writes() {
    let f = Fixture::new();
    let store = f.store();
    let c = f.create(&store);
    let before = fs::read(f.path()).expect("before");
    for title in [
        "".into(),
        " ".into(),
        "x\ny".into(),
        "x".repeat(193),
        "中".repeat(65),
    ] {
        assert_eq!(
            f.manage(&store, &c.id, rename("invalid", 0, &title))
                .expect_err("title"),
            ConversationError::InvalidIntent
        );
    }
    let other = UserId::parse("user:bob").expect("other");
    assert!(matches!(
        store.manage(&f.owner.0, &other, &c.id, delete("foreign", 0)),
        Err(ConversationError::NotFound)
    ));
    let other_tenant = TenantId::parse("tenant:other").expect("other");
    assert!(matches!(
        store.manage(&other_tenant, &f.owner.1, &c.id, delete("foreign", 0)),
        Err(ConversationError::NotFound)
    ));
    assert_eq!(
        f.manage(&store, &c.id, rename("stale", 1, "Name"))
            .expect_err("stale"),
        ConversationError::RevisionConflict
    );
    assert_eq!(fs::read(f.path()).expect("no writes"), before);
    let management = rename("management-id", 0, &"中".repeat(64));
    f.manage(&store, &c.id, management.clone())
        .expect("192-byte boundary");
    assert!(matches!(
        store.begin(
            &f.owner.0,
            &f.owner.1,
            &c.id,
            turn("management-id", 1),
            false
        ),
        Err(ConversationError::RequestConflict)
    ));
    assert_eq!(
        f.manage(&store, &c.id, delete("management-id", 0))
            .expect_err("cross action"),
        ConversationError::RequestConflict
    );
    store
        .begin(&f.owner.0, &f.owner.1, &c.id, turn("turn-id", 1), false)
        .expect("running");
    let before = fs::read(f.path()).expect("before busy");
    for intent in [rename("busy", 2, "Name"), delete("busy", 2)] {
        assert_eq!(
            f.manage(&store, &c.id, intent).expect_err("busy"),
            ConversationError::InProgress
        );
    }
    assert_eq!(
        f.manage(&store, &c.id, management)
            .expect("exact old receipt during busy")
            .revision,
        1
    );
    assert_eq!(fs::read(f.path()).expect("busy no write"), before);
    finish(&f, &store, &c.id, "turn-id");
    assert_eq!(
        f.manage(&store, &c.id, delete("turn-id", 3))
            .expect_err("turn namespace"),
        ConversationError::RequestConflict
    );
    for raw in [
        r#"{"schema":"chat-conversation-manage/v1","request_id":"r","expected_revision":0,"action":{"kind":"delete","title":"smuggled"}}"#,
        r#"{"schema":"chat-conversation-manage/v1","request_id":"r","expected_revision":0,"action":{"kind":"rename","title":"a","title":"b"}}"#,
        r#"{"schema":"chat-conversation-manage/v1","request_id":"r","request_id":"x","expected_revision":0,"action":{"kind":"delete"}}"#,
        r#"{"schema":"chat-conversation-manage/v1","request_id":"r","expected_revision":0,"action":{"kind":"restore"}}"#,
    ] {
        assert!(serde_json::from_str::<ConversationManageIntentDto>(raw).is_err());
    }
}
#[test]
fn visible_limit_excludes_deletions_but_restart_preserves_physical_retention() {
    let f = Fixture::new();
    let store = f.store();
    let mut ids = Vec::new();
    for i in 0..50 {
        ids.push(
            store
                .create(&f.owner.0, &f.owner.1, &format!("create-{i}"))
                .expect("visible slot")
                .id,
        );
    }
    assert!(matches!(
        store.create(&f.owner.0, &f.owner.1, "overflow"),
        Err(ConversationError::Capacity)
    ));
    f.manage(&store, &ids[0], delete("delete", 0))
        .expect("free visible slot");
    store
        .create(&f.owner.0, &f.owner.1, "replacement")
        .expect("replacement");
    assert_eq!(
        store
            .list(&f.owner.0, &f.owner.1)
            .expect("visible")
            .conversations
            .len(),
        50
    );
    drop(store);
    let store = f.store();
    assert_eq!(
        store
            .list(&f.owner.0, &f.owner.1)
            .expect("reopened visible")
            .conversations
            .len(),
        50
    );
    assert_eq!(
        store.inner.lock().expect("state").state.conversations.len(),
        51
    );
    assert!(matches!(
        store.create(&f.owner.0, &f.owner.1, "create-0"),
        Err(ConversationError::NotFound)
    ));
}
#[test]
fn management_capacity_keeps_the_final_delete_slot_and_exact_receipts() {
    let f = Fixture::new();
    let store = f.store();
    let c = f.create(&store);
    for revision in 0..127 {
        f.manage(
            &store,
            &c.id,
            rename(&format!("rename-{revision}"), revision, "Title"),
        )
        .expect("rename slot");
    }
    assert_eq!(
        f.manage(&store, &c.id, rename("one-too-many", 127, "Title"))
            .expect_err("reserve delete slot"),
        ConversationError::Capacity
    );
    let result = f
        .manage(&store, &c.id, delete("last-delete", 127))
        .expect("last slot delete");
    assert_eq!(result.revision, 128);
    assert_eq!(
        f.manage(&store, &c.id, rename("rename-0", 0, "Title"))
            .expect("retry without new slot")
            .revision,
        1
    );
    drop(store);
    let store = f.store();
    assert_eq!(
        f.manage(&store, &c.id, delete("last-delete", 127))
            .expect("reopen delete receipt"),
        result
    );
}
#[test]
fn unused_legacy_store_stays_byte_identical_and_uncertain_management_save_poisons() {
    let f = Fixture::new();
    let store = f.store();
    let c = f.create(&store);
    let original = fs::read(f.path()).expect("legacy bytes");
    assert!(!String::from_utf8_lossy(&original).contains("management"));
    drop(store);
    let store = f.store();
    assert_eq!(fs::read(f.path()).expect("unchanged open"), original);
    fs::set_permissions(f.path(), fs::Permissions::from_mode(0o644))
        .expect("simulate unsafe replacement");
    assert_eq!(
        f.manage(&store, &c.id, rename("rename", 0, "Name"))
            .expect_err("save failure"),
        ConversationError::Unavailable
    );
    fs::set_permissions(f.path(), fs::Permissions::from_mode(0o600)).expect("restore fixture mode");
    assert_eq!(
        fs::read(f.path()).expect("unacknowledged state unchanged"),
        original
    );
    assert!(matches!(
        store.get(&f.owner.0, &f.owner.1, &c.id),
        Err(ConversationError::Unavailable)
    ));
}
#[test]
fn malformed_management_revisions_receipts_and_tombstones_fail_closed_on_reopen() {
    let f = Fixture::new();
    let store = f.store();
    let c = f.create(&store);
    f.manage(&store, &c.id, rename("before", 0, "Before"))
        .expect("rename");
    store
        .begin(&f.owner.0, &f.owner.1, &c.id, turn("turn", 1), false)
        .expect("begin");
    finish(&f, &store, &c.id, "turn");
    f.manage(&store, &c.id, rename("after", 3, "After"))
        .expect("rename");
    f.manage(&store, &c.id, delete("delete", 4))
        .expect("delete");
    drop(store);
    let original: Value =
        serde_json::from_slice(&fs::read(f.path()).expect("state")).expect("JSON");
    for pointer in [
        "/conversations/0/management/1/intent/expected_revision",
        "/conversations/0/management/1/result/revision",
        "/conversations/0/turns/0/result_revision",
        "/conversations/0/revision",
    ] {
        let mut altered = original.clone();
        *altered.pointer_mut(pointer).expect("field") = json!(2);
        fs::write(f.path(), altered.to_string()).expect("corrupt fixture");
        assert!(ConversationStore::open(f.path()).is_err(), "{pointer}");
    }
    for (pointer, value) in [
        ("/conversations/0/deleted", json!(false)),
        ("/conversations/0/explicit_title", json!(false)),
        ("/conversations/0/title", json!("forged")),
        ("/conversations/0/management/2/result/deleted", json!(false)),
        (
            "/conversations/0/management/0/result/title",
            json!("forged"),
        ),
        (
            "/conversations/0/management/1/intent/request_id",
            json!("turn"),
        ),
        (
            "/conversations/0/management/2/intent/action",
            json!({"kind":"rename","title":"After"}),
        ),
    ] {
        let mut altered = original.clone();
        *altered.pointer_mut(pointer).expect("field") = value;
        fs::write(f.path(), altered.to_string()).expect("corrupt fixture");
        assert!(ConversationStore::open(f.path()).is_err(), "{pointer}");
    }
    fs::write(f.path(), original.to_string()).expect("restore");
    let store = f.store();
    assert!(
        store
            .list(&f.owner.0, &f.owner.1)
            .expect("valid reopened tombstone")
            .conversations
            .is_empty()
    );
}

#[test]
fn physical_capacity_counts_tombstones_and_running_management_overlap_is_invalid() {
    let f = Fixture::new();
    let store = f.store();
    let c = f.create(&store);
    let template = store.inner.lock().expect("state").state.conversations[0].clone();
    f.manage(&store, &c.id, delete("delete", 0))
        .expect("tombstone");
    {
        let mut inner = store.inner.lock().expect("state");
        let mut next = inner.state.clone();
        for index in 1..1000 {
            let mut row = template.clone();
            row.id = format!("{index:064x}");
            row.tenant = format!("tenant:quota-{}", index / 49);
            row.create_request = format!("create-{index}");
            next.conversations.push(row);
        }
        validate_state(&next).expect("valid 1000 physical retained entries");
        inner.commit(next).expect("seed exact physical bound");
    }
    assert!(matches!(
        store.create(&f.owner.0, &f.owner.1, "another"),
        Err(ConversationError::Capacity)
    ));
    drop(store);
    let store = f.store();
    assert_eq!(
        store.inner.lock().expect("state").state.conversations.len(),
        1000
    );
    drop(store);
    let f = Fixture::new();
    let store = f.store();
    let c = f.create(&store);
    f.manage(&store, &c.id, rename("before", 0, "Before"))
        .expect("rename");
    store
        .begin(&f.owner.0, &f.owner.1, &c.id, turn("running", 1), false)
        .expect("running");
    let mut bad = store.inner.lock().expect("state").state.clone();
    let row = &mut bad.conversations[0];
    row.management.push(StoredManagement {
        intent: delete("impossible-delete", 2),
        result: ConversationManageResultDto {
            schema: "chat-conversation-manage-result/v1".into(),
            conversation_id: c.id.clone(),
            request_id: "impossible-delete".into(),
            revision: 3,
            title: "Before".into(),
            deleted: true,
        },
    });
    row.revision = 3;
    row.deleted = true;
    assert!(validate_state(&bad).is_err());
    drop(store);
    let store = f.store();
    let current = store
        .get(&f.owner.0, &f.owner.1, &c.id)
        .expect("legitimate running recovery");
    assert_eq!(current.title, "Before");
    assert_eq!(current.revision, 3);
    assert_eq!(current.turns[0].phase, TurnPhase::Interrupted);
}
