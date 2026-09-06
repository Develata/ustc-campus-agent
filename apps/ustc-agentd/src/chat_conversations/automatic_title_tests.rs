use super::*;
use crate::conversation_title::{dated_title, valid_title};
use serde_json::{Value, json};
fn title(f: &Fixture, store: &ConversationStore, id: &str) -> String {
    store.get(&f.tenant, &f.user, id).expect("view").title
}
fn rename(request: &str, revision: u64, name: &str) -> ConversationManageIntentDto {
    serde_json::from_value(json!({"schema":"chat-conversation-manage/v1","request_id":request,"expected_revision":revision,"action":{"kind":"rename","title":name}})).expect("rename")
}

#[test]
fn conversation_title_date_is_reserved_and_generation_finishes_atomically_once() {
    let f = Fixture::new();
    let store = f.store();
    let c = f.create(&store);
    let first = intent("first", 0, "请帮助整理日历提醒");
    let BeginTurn::New { title_request, .. } = store
        .begin(&f.tenant, &f.user, &c.id, first.clone(), false)
        .expect("begin")
    else {
        panic!("new")
    };
    assert_eq!(title_request.as_deref(), Some(first.message.as_str()));
    let fallback = title(&f, &store, &c.id);
    assert!(valid_title(&fallback));
    let disk: Value =
        serde_json::from_slice(&fs::read(f.path()).expect("reserved state")).expect("JSON");
    assert_eq!(
        disk["conversations"][0]["turns"][0]["automatic_title"]["fallback"],
        fallback
    );
    assert_eq!(disk["conversations"][0]["revision"], 1);
    // Simulate an admission from a previous campus date without changing the host clock.
    {
        let mut inner = store.inner.lock().expect("state");
        let mut value = serde_json::to_value(&inner.state).expect("state");
        value["conversations"][0]["created_date"] = json!("240229");
        value["conversations"][0]["turns"][0]["automatic_title"]["date"] = json!("240229");
        value["conversations"][0]["turns"][0]["automatic_title"]["fallback"] =
            json!(dated_title("240229", &first.message));
        value["conversations"][0]["title"] = json!(dated_title("240229", &first.message));
        let next: State = serde_json::from_value(value).expect("pinned date state");
        validate_state(&next).expect("valid date evidence");
        inner.commit(next).expect("pinned admission");
    }
    let completed = store
        .finish_with_title(
            &f.tenant,
            &f.user,
            &c.id,
            "first",
            Ok(response("answer")),
            Some("“启动日历”".into()),
        )
        .expect("atomic terminal");
    assert_eq!(completed.revision, 2);
    assert_eq!(title(&f, &store, &c.id), "240229|启动日历");
    let before = fs::read(f.path()).expect("terminal bytes");
    let replay = store
        .finish_with_title(
            &f.tenant,
            &f.user,
            &c.id,
            "first",
            Ok(response("different")),
            Some("覆盖旧名".into()),
        )
        .expect("finish replay");
    assert_eq!(
        serde_json::to_value(completed).expect("completed"),
        serde_json::to_value(replay).expect("replay")
    );
    assert_eq!(fs::read(f.path()).expect("no replay write"), before);
    assert!(matches!(
        store.begin(&f.tenant, &f.user, &c.id, first, false),
        Ok(BeginTurn::Replay(_))
    ));
    let BeginTurn::New { title_request, .. } = store
        .begin(
            &f.tenant,
            &f.user,
            &c.id,
            intent("second", 2, "不同话题"),
            false,
        )
        .expect("second")
    else {
        panic!("new")
    };
    assert!(title_request.is_none());
    store
        .finish_with_title(
            &f.tenant,
            &f.user,
            &c.id,
            "second",
            Ok(response("second")),
            Some("后续覆盖".into()),
        )
        .expect("second finish");
    assert_eq!(title(&f, &store, &c.id), "240229|启动日历");
    store
        .manage(
            &f.tenant,
            &f.user,
            &c.id,
            rename("manual", 4, "我的手工标题"),
        )
        .expect("manual rename after generated title");
    drop(store);
    let store = f.store();
    assert_eq!(title(&f, &store, &c.id), "我的手工标题");
    assert_eq!(
        store.get(&f.tenant, &f.user, &c.id).expect("view").revision,
        5
    );
}

#[test]
fn conversation_title_manual_name_before_first_turn_never_gets_generation_metadata() {
    let f = Fixture::new();
    let store = f.store();
    let c = f.create(&store);
    let command = rename("manual", 0, "手工保留名称");
    let receipt = store
        .manage(&f.tenant, &f.user, &c.id, command.clone())
        .expect("manual");
    let BeginTurn::New { title_request, .. } = store
        .begin(
            &f.tenant,
            &f.user,
            &c.id,
            intent("first", 1, "用户消息"),
            false,
        )
        .expect("begin")
    else {
        panic!("new")
    };
    assert!(title_request.is_none());
    store
        .finish_with_title(
            &f.tenant,
            &f.user,
            &c.id,
            "first",
            Ok(response("answer")),
            Some("模型候选".into()),
        )
        .expect("finish");
    assert_eq!(title(&f, &store, &c.id), "手工保留名称");
    let disk: Value = serde_json::from_slice(&fs::read(f.path()).expect("disk")).expect("JSON");
    assert!(
        disk["conversations"][0]["turns"][0]
            .get("automatic_title")
            .is_none()
    );
    drop(store);
    let store = f.store();
    assert_eq!(
        store
            .manage(&f.tenant, &f.user, &c.id, command)
            .expect("manual replay"),
        receipt
    );
    assert_eq!(title(&f, &store, &c.id), "手工保留名称");
}

#[test]
fn conversation_title_invalid_generation_failure_and_restart_keep_the_dated_fallback() {
    let f = Fixture::new();
    let store = f.store();
    for (index, candidate) in [
        Some("无效|分隔".into()),
        Some("换\n行".into()),
        Some("长".repeat(25)),
        Some("".into()),
        None,
    ]
    .into_iter()
    .enumerate()
    {
        let c = store
            .create(&f.tenant, &f.user, &format!("create-{index}"))
            .expect("create");
        store
            .begin(
                &f.tenant,
                &f.user,
                &c.id,
                intent("first", 0, "fallback 用户消息"),
                false,
            )
            .expect("begin");
        let before = title(&f, &store, &c.id);
        store
            .finish_with_title(
                &f.tenant,
                &f.user,
                &c.id,
                "first",
                Ok(response("answer")),
                candidate,
            )
            .expect("invalid title does not fail chat");
        assert_eq!(title(&f, &store, &c.id), before);
    }
    let failed = store.create(&f.tenant, &f.user, "failed").expect("create");
    store
        .begin(
            &f.tenant,
            &f.user,
            &failed.id,
            intent("first", 0, "失败请求"),
            false,
        )
        .expect("begin");
    let failed_title = title(&f, &store, &failed.id);
    store
        .finish_with_title(
            &f.tenant,
            &f.user,
            &failed.id,
            "first",
            Err(ChatError::ProviderTimeout),
            Some("不可使用".into()),
        )
        .expect("failed receipt");
    assert_eq!(title(&f, &store, &failed.id), failed_title);
    let pending = store.create(&f.tenant, &f.user, "pending").expect("create");
    let pending_intent = intent("first", 0, "跨重启请求");
    store
        .begin(
            &f.tenant,
            &f.user,
            &pending.id,
            pending_intent.clone(),
            false,
        )
        .expect("begin");
    let pending_title = title(&f, &store, &pending.id);
    drop(store);
    let store = f.store();
    assert_eq!(title(&f, &store, &failed.id), failed_title);
    assert_eq!(title(&f, &store, &pending.id), pending_title);
    let Ok(BeginTurn::Replay(result)) =
        store.begin(&f.tenant, &f.user, &pending.id, pending_intent, false)
    else {
        panic!("interrupted replay")
    };
    assert_eq!(result.turn.phase, TurnPhase::Interrupted);
}

#[test]
fn conversation_title_legacy_metadata_absence_preserves_old_names_and_bytes() {
    let f = Fixture::new();
    let store = f.store();
    let c = f.create(&store);
    let legacy = intent("first", 0, "Legacy name without a date");
    store
        .begin(&f.tenant, &f.user, &c.id, legacy.clone(), false)
        .expect("begin");
    store
        .finish(
            &f.tenant,
            &f.user,
            &c.id,
            "first",
            Ok(response("legacy answer")),
        )
        .expect("finish");
    drop(store);
    let mut value: Value =
        serde_json::from_slice(&fs::read(f.path()).expect("state")).expect("JSON");
    value["conversations"][0]["turns"][0]
        .as_object_mut()
        .expect("turn")
        .remove("automatic_title");
    value["conversations"][0]["title"] = json!(legacy.message);
    let bytes = serde_json::to_vec(&value).expect("legacy bytes");
    fs::write(f.path(), &bytes).expect("legacy fixture");
    let store = f.store();
    assert_eq!(fs::read(f.path()).expect("unchanged"), bytes);
    assert_eq!(title(&f, &store, &c.id), legacy.message);
    let BeginTurn::New { title_request, .. } = store
        .begin(
            &f.tenant,
            &f.user,
            &c.id,
            intent("second", 2, "new message"),
            false,
        )
        .expect("continuation")
    else {
        panic!("new")
    };
    assert!(title_request.is_none());
    store
        .finish_with_title(
            &f.tenant,
            &f.user,
            &c.id,
            "second",
            Ok(response("answer")),
            Some("模型不改旧名".into()),
        )
        .expect("finish");
    assert_eq!(title(&f, &store, &c.id), legacy.message);
    drop(store);
    let store = f.store();
    assert_eq!(title(&f, &store, &c.id), legacy.message);
}

#[test]
fn conversation_title_corrupt_date_fallback_phase_placement_and_current_title_fail_closed() {
    let f = Fixture::new();
    let store = f.store();
    let c = f.create(&store);
    store
        .begin(
            &f.tenant,
            &f.user,
            &c.id,
            intent("first", 0, "first message"),
            false,
        )
        .expect("begin");
    store
        .finish_with_title(
            &f.tenant,
            &f.user,
            &c.id,
            "first",
            Ok(response("answer")),
            Some("有效话题".into()),
        )
        .expect("finish");
    store
        .begin(
            &f.tenant,
            &f.user,
            &c.id,
            intent("second", 2, "second message"),
            false,
        )
        .expect("second");
    store
        .finish(&f.tenant, &f.user, &c.id, "second", Ok(response("answer")))
        .expect("finish second");
    drop(store);
    let original: Value =
        serde_json::from_slice(&fs::read(f.path()).expect("state")).expect("JSON");
    for (pointer, value) in [
        (
            "/conversations/0/turns/0/automatic_title/date",
            json!("260229"),
        ),
        (
            "/conversations/0/turns/0/automatic_title/fallback",
            json!("240229|forged"),
        ),
        (
            "/conversations/0/turns/0/automatic_title/generated",
            json!("240229|different date"),
        ),
        (
            "/conversations/0/turns/0/automatic_title/generated",
            json!("260921|bad|title"),
        ),
        ("/conversations/0/title", json!("forged current title")),
    ] {
        let mut bad = original.clone();
        *bad.pointer_mut(pointer).expect("field") = value;
        fs::write(f.path(), bad.to_string()).expect("corrupt fixture");
        assert!(ConversationStore::open(f.path()).is_err(), "{pointer}");
    }
    let mut bad = original.clone();
    bad["conversations"][0]["turns"][1]["automatic_title"] =
        bad["conversations"][0]["turns"][0]["automatic_title"].clone();
    fs::write(f.path(), bad.to_string()).expect("bad placement");
    assert!(ConversationStore::open(f.path()).is_err());
    let mut bad = original.clone();
    bad["conversations"][0]["turns"][0]["view"]["phase"] = json!("failed");
    bad["conversations"][0]["turns"][0]["view"]["response"] = Value::Null;
    bad["conversations"][0]["turns"][0]["view"]["error"] = json!("provider_timeout");
    fs::write(f.path(), bad.to_string()).expect("bad phase");
    assert!(ConversationStore::open(f.path()).is_err());
    let mut bad = original.clone();
    bad["conversations"][0]["turns"][0]["automatic_title"]["extra"] = json!(true);
    fs::write(f.path(), bad.to_string()).expect("unknown field");
    assert!(ConversationStore::open(f.path()).is_err());
    fs::write(f.path(), original.to_string()).expect("restore valid state");
    let store = f.store();
    assert!(valid_title(&title(&f, &store, &c.id)));
}
