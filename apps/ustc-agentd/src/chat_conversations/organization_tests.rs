use super::conversation_management_tests::{Fixture, finish, turn};
use super::*;
use serde_json::json;

fn intent(id: &str, revision: u64, action: serde_json::Value) -> ConversationManageIntentDto {
    serde_json::from_value(
        json!({"schema":"chat-conversation-manage/v2", "request_id":id,
        "expected_revision":revision,"action":action}),
    )
    .expect("intent")
}
#[test]
fn organization_v2_rename_pin_group_receipts_owner_revision_and_restart() {
    let f = Fixture::new();
    let store = f.store();
    let c = f.create(&store);
    let date = c.organization.date.clone().expect("creation date");
    let rename = intent("rename", 0, json!({"kind":"rename","title":"  启动日历  "}));
    let original = f.manage(&store, &c.id, rename.clone()).expect("rename");
    assert_eq!(original.title, format!("{date}|启动日历"));
    assert_eq!(original.schema, "chat-conversation-manage-result/v2");
    let foreign = UserId::parse("user:bob").expect("user");
    assert_eq!(
        store.manage(&f.owner.0, &foreign, &c.id, rename.clone()),
        Err(ConversationError::NotFound)
    );
    assert_eq!(
        f.manage(
            &store,
            &c.id,
            intent("stale", 0, json!({"kind":"pin","pinned":true}))
        ),
        Err(ConversationError::RevisionConflict)
    );
    f.manage(
        &store,
        &c.id,
        intent("pin", 1, json!({"kind":"pin","pinned":true})),
    )
    .expect("pin");
    let group = f
        .manage(
            &store,
            &c.id,
            intent("group", 2, json!({"kind":"group","group":"  学业  "})),
        )
        .expect("group");
    assert_eq!(
        group
            .organization
            .as_ref()
            .expect("organization")
            .group
            .as_deref(),
        Some("学业")
    );
    assert_eq!(group.title, original.title);
    store
        .begin(&f.owner.0, &f.owner.1, &c.id, turn("turn", 3), false)
        .expect("turn");
    assert_eq!(
        f.manage(
            &store,
            &c.id,
            intent("running", 4, json!({"kind":"pin","pinned":false}))
        ),
        Err(ConversationError::InProgress)
    );
    finish(&f, &store, &c.id, "turn");
    drop(store);
    let store = f.store();
    assert_eq!(
        f.manage(&store, &c.id, rename).expect("exact old receipt"),
        original
    );
    let current = store.get(&f.owner.0, &f.owner.1, &c.id).expect("view");
    assert_eq!(
        current.organization,
        group.organization.expect("organization")
    );
    assert_eq!(current.title, original.title);
    f.manage(
        &store,
        &c.id,
        intent("ungroup", 5, json!({"kind":"group","group":null})),
    )
    .expect("ungroup");
    f.manage(
        &store,
        &c.id,
        intent("unpin", 6, json!({"kind":"pin","pinned":false})),
    )
    .expect("unpin");
    drop(store);
    let store = f.store();
    let current = store.get(&f.owner.0, &f.owner.1, &c.id).expect("view");
    assert!(!current.organization.pinned);
    assert_eq!(current.organization.group, None);
    assert_eq!(current.organization.date, Some(date));
}
#[test]
fn organization_order_is_pin_then_date_then_creation_not_activity() {
    let f = Fixture::new();
    let store = f.store();
    let mut ids = Vec::new();
    for id in ["old", "new", "same", "unknown"] {
        ids.push(store.create(&f.owner.0, &f.owner.1, id).expect("create").id);
    }
    {
        let mut inner = store.inner.lock().expect("state");
        let mut next = inner.state.clone();
        for (row, date) in next.conversations.iter_mut().zip([
            Some("240229"),
            Some("260921"),
            Some("260921"),
            None,
        ]) {
            row.created_date = date.map(str::to_owned);
        }
        validate_state(&next).expect("fixture valid");
        inner.commit(next).expect("fixture");
    }
    let listed = || {
        store
            .list(&f.owner.0, &f.owner.1)
            .expect("list")
            .conversations
            .into_iter()
            .map(|c| c.id)
            .collect::<Vec<_>>()
    };
    assert_eq!(
        listed(),
        vec![
            ids[2].clone(),
            ids[1].clone(),
            ids[0].clone(),
            ids[3].clone()
        ]
    );
    store
        .begin(&f.owner.0, &f.owner.1, &ids[0], turn("activity", 0), false)
        .expect("activity");
    finish(&f, &store, &ids[0], "activity");
    assert_eq!(listed()[2], ids[0]);
    f.manage(
        &store,
        &ids[0],
        intent("pin", 2, json!({"kind":"pin","pinned":true})),
    )
    .expect("pin");
    assert_eq!(listed()[0], ids[0]);
    assert_eq!(
        store
            .get(&f.owner.0, &f.owner.1, &ids[0])
            .expect("date")
            .organization
            .date
            .as_deref(),
        Some("240229")
    );
    drop(store);
    f.store();
}
#[test]
fn organization_legacy_undated_read_does_not_rewrite_and_first_write_dates_once() {
    let f = Fixture::new();
    let store = f.store();
    let c = f.create(&store);
    {
        let mut inner = store.inner.lock().expect("state");
        let mut next = inner.state.clone();
        next.conversations[0].created_date = None;
        inner.commit(next).expect("legacy fixture");
    }
    drop(store);
    let bytes = std::fs::read(f.path()).expect("bytes");
    let store = f.store();
    assert_eq!(
        store
            .get(&f.owner.0, &f.owner.1, &c.id)
            .expect("legacy")
            .organization
            .date,
        None
    );
    assert_eq!(std::fs::read(f.path()).expect("read unchanged"), bytes);
    let pin = f
        .manage(
            &store,
            &c.id,
            intent("pin", 0, json!({"kind":"pin","pinned":true})),
        )
        .expect("date adoption");
    assert_eq!(pin.title, "新对话");
    let date = pin.organization.expect("metadata").date.expect("date");
    let renamed = f
        .manage(
            &store,
            &c.id,
            intent("rename", 1, json!({"kind":"rename","title":"旧话题"})),
        )
        .expect("rename");
    assert_eq!(renamed.title, format!("{date}|旧话题"));
    drop(store);
    f.store();
}
#[test]
fn organization_rejects_delimiters_format_controls_unknown_fields_and_missing_group() {
    let f = Fixture::new();
    let store = f.store();
    let c = f.create(&store);
    for topic in ["260921|x", "x\u{200b}", "x\u{202e}", "\n", ""] {
        assert_eq!(
            f.manage(
                &store,
                &c.id,
                intent("bad", 0, json!({"kind":"rename","title":topic}))
            ),
            Err(ConversationError::InvalidIntent)
        );
    }
    for action in [
        json!({"kind":"group"}),
        json!({"kind":"pin","pinned":true,"date":"260921"}),
        json!({"kind":"group","group":null,"pinned":false}),
    ] {
        assert!(serde_json::from_value::<ConversationManageIntentDto>(json!({"schema":"chat-conversation-manage/v2","request_id":"bad","expected_revision":0,"action":action})).is_err());
    }
    let mut v1 = intent("bad", 0, json!({"kind":"pin","pinned":true}));
    v1.schema = "chat-conversation-manage/v1".into();
    assert_eq!(
        f.manage(&store, &c.id, v1),
        Err(ConversationError::InvalidIntent)
    );
    let long = "课".repeat(64);
    let renamed = f
        .manage(
            &store,
            &c.id,
            intent("limit", 0, json!({"kind":"rename","title":long})),
        )
        .expect("192-byte topic accepted");
    assert_eq!(renamed.title.len(), 199);
    drop(store);
    f.store();
}
#[test]
fn organization_new_metadata_and_historical_receipt_tampering_fail_closed() {
    let f = Fixture::new();
    let store = f.store();
    let c = f.create(&store);
    f.manage(
        &store,
        &c.id,
        intent("pin", 0, json!({"kind":"pin","pinned":true})),
    )
    .expect("pin");
    let state = store.inner.lock().expect("state").state.clone();
    for mutation in 0..5 {
        let mut bad = state.clone();
        let row = &mut bad.conversations[0];
        match mutation {
            0 => row.organization.as_mut().expect("metadata").pinned = false,
            1 => row.organization.as_mut().expect("metadata").date = Some("240229".into()),
            2 => row.management[0].result.organization = None,
            3 => {
                row.management[0]
                    .result
                    .organization
                    .as_mut()
                    .expect("receipt")
                    .date = None
            }
            _ => row.management[0].result.schema = "chat-conversation-manage-result/v1".into(),
        }
        assert!(validate_state(&bad).is_err(), "tamper {mutation}");
    }
}
#[test]
fn organization_pin_and_group_reserve_last_receipt_for_delete() {
    let f = Fixture::new();
    let store = f.store();
    let c = f.create(&store);
    for index in 0..127 {
        f.manage(
            &store,
            &c.id,
            intent(
                &format!("pin-{index}"),
                index,
                json!({"kind":"pin","pinned":index%2==0}),
            ),
        )
        .expect("pin budget");
    }
    assert_eq!(
        f.manage(
            &store,
            &c.id,
            intent("full", 127, json!({"kind":"group","group":"学业"}))
        ),
        Err(ConversationError::Capacity)
    );
    f.manage(
        &store,
        &c.id,
        intent("delete", 127, json!({"kind":"delete"})),
    )
    .expect("reserved deletion");
    drop(store);
    f.store();
}

#[test]
fn organization_legacy_dated_title_empty_row_is_recovered_and_replayed() {
    let f = Fixture::new();
    let store = f.store();
    let c = f.create(&store);
    {
        let mut inner = store.inner.lock().expect("state");
        let mut next = inner.state.clone();
        next.conversations[0].created_date = None;
        next.conversations[0].title = "240229|旧名称".into();
        validate_state(&next).expect("admitted legacy row");
        inner.commit(next).expect("legacy fixture");
    }
    let pin = f
        .manage(
            &store,
            &c.id,
            intent("pin", 0, json!({"kind":"pin","pinned":true})),
        )
        .expect("pin");
    assert_eq!(pin.title, "240229|旧名称");
    assert_eq!(
        pin.organization.expect("metadata").date.as_deref(),
        Some("240229")
    );
    drop(store);
    let store = f.store();
    store
        .begin(&f.owner.0, &f.owner.1, &c.id, turn("first", 1), false)
        .expect("first after pin");
    finish(&f, &store, &c.id, "first");
    drop(store);
    let store = f.store();
    let renamed = f
        .manage(
            &store,
            &c.id,
            intent("rename", 3, json!({"kind":"rename","title":"新名称"})),
        )
        .expect("rename");
    assert_eq!(renamed.title, "240229|新名称");
    drop(store);
    f.store();
}
