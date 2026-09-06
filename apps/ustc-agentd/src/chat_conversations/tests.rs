use super::*;
use crate::agent_chat::ChatUsageDto;
use crate::chat_provider::ProviderIdentity;
use std::fs;
use std::os::unix::fs::PermissionsExt;

struct Fixture {
    directory: PathBuf,
    tenant: TenantId,
    user: UserId,
}
impl Fixture {
    fn new() -> Self {
        let directory = std::env::temp_dir().join(format!(
            "uca-conversations-{}",
            persistence::random_id().expect("random")
        ));
        fs::create_dir(&directory).expect("mkdir");
        fs::set_permissions(&directory, fs::Permissions::from_mode(0o700)).expect("chmod");
        Self {
            directory,
            tenant: TenantId::parse("tenant:test").expect("tenant"),
            user: UserId::parse("user:alice").expect("user"),
        }
    }
    fn path(&self) -> PathBuf {
        self.directory.join("conversations.json")
    }
    fn store(&self) -> ConversationStore {
        ConversationStore::open(self.path()).expect("open")
    }
    fn create(&self, store: &ConversationStore) -> ConversationDto {
        store
            .create(&self.tenant, &self.user, "create-1")
            .expect("create")
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.directory);
    }
}
fn intent(request: &str, revision: u64, message: &str) -> ConversationTurnIntentDto {
    ConversationTurnIntentDto {
        schema: "chat-conversation-turn/v1".to_owned(),
        model_id: crate::model_catalog::ModelSelectionFieldDto::Absent,
        request_id: request.to_owned(),
        expected_revision: revision,
        message: message.to_owned(),
        opportunity_context: None,
        prompt_customization: PromptCustomizationFieldDto::Absent,
    }
}
fn response(answer: &str) -> ChatResponseDto {
    ChatResponseDto {
        schema: "ustc-agent-chat-response/v1",
        run_id: "chat-run:test".to_owned(),
        answer: answer.to_owned(),
        provider: ProviderIdentity {
            mode: "mock".to_owned(),
            model: "test".to_owned(),
        },
        tool_trace: Vec::new(),
        usage: ChatUsageDto::default(),
    }
}
#[test]
fn persistence_owner_isolation_and_create_retry() {
    let f = Fixture::new();
    let store = f.store();
    let c = f.create(&store);
    assert_eq!(f.create(&store).id, c.id);
    let bob = UserId::parse("user:bob").expect("bob");
    assert!(
        store
            .list(&f.tenant, &bob)
            .expect("list")
            .conversations
            .is_empty()
    );
    assert!(matches!(
        store.get(&f.tenant, &bob, &c.id),
        Err(ConversationError::NotFound)
    ));
    let other = TenantId::parse("tenant:other").expect("tenant");
    assert!(matches!(
        store.get(&other, &f.user, &c.id),
        Err(ConversationError::NotFound)
    ));
    drop(store);
    assert_eq!(
        f.store().get(&f.tenant, &f.user, &c.id).expect("reopen").id,
        c.id
    );
}
#[test]
fn duplicate_never_reexecutes_and_replays_original_revision() {
    let f = Fixture::new();
    let store = f.store();
    let c = f.create(&store);
    let i = intent("turn-1", 0, "hello");
    assert!(matches!(
        store.begin(&f.tenant, &f.user, &c.id, i.clone(), false),
        Ok(BeginTurn::New { .. })
    ));
    assert!(matches!(
        store.begin(&f.tenant, &f.user, &c.id, i.clone(), false),
        Err(ConversationError::InProgress)
    ));
    let mut different = i.clone();
    different.message = "changed".to_owned();
    assert!(matches!(
        store.begin(&f.tenant, &f.user, &c.id, different, false),
        Err(ConversationError::RequestConflict)
    ));
    store
        .finish(&f.tenant, &f.user, &c.id, "turn-1", Ok(response("answer")))
        .expect("finish");
    store
        .begin(
            &f.tenant,
            &f.user,
            &c.id,
            intent("turn-2", 2, "next"),
            false,
        )
        .expect("next");
    let BeginTurn::Replay(replay) = store
        .begin(&f.tenant, &f.user, &c.id, i, false)
        .expect("replay")
    else {
        panic!("must replay")
    };
    assert_eq!(replay.revision, 2);
    assert_eq!(replay.turn.phase, TurnPhase::Completed);
}
#[test]
fn restart_marks_running_interrupted_and_preserves_exact_retry() {
    let f = Fixture::new();
    let store = f.store();
    let c = f.create(&store);
    let i = intent("turn-1", 0, "write calendar");
    store
        .begin(&f.tenant, &f.user, &c.id, i.clone(), false)
        .expect("begin");
    drop(store);
    let store = f.store();
    let saved = store.get(&f.tenant, &f.user, &c.id).expect("saved");
    assert_eq!(saved.revision, 2);
    assert_eq!(saved.turns[0].phase, TurnPhase::Interrupted);
    assert!(matches!(
        store.begin(&f.tenant, &f.user, &c.id, i, false),
        Ok(BeginTurn::Replay(_))
    ));
    assert!(matches!(
        store.begin(
            &f.tenant,
            &f.user,
            &c.id,
            intent("turn-2", 2, "new intent"),
            false
        ),
        Ok(BeginTurn::New { .. })
    ));
}
#[test]
fn validation_and_revision_conflict_reserve_nothing() {
    let f = Fixture::new();
    let store = f.store();
    let c = f.create(&store);
    assert!(matches!(
        store.begin(&f.tenant, &f.user, &c.id, intent("bad", 0, ""), false),
        Err(ConversationError::InvalidChat(_))
    ));
    assert!(matches!(
        store.begin(
            &f.tenant,
            &f.user,
            &c.id,
            intent("stale", 9, "hello"),
            false
        ),
        Err(ConversationError::RevisionConflict)
    ));
    assert!(
        store
            .get(&f.tenant, &f.user, &c.id)
            .expect("view")
            .turns
            .is_empty()
    );
}
#[test]
fn history_is_server_owned_and_stops_at_profile_or_size_boundary() {
    let f = Fixture::new();
    let store = f.store();
    let c = f.create(&store);
    let mut i = intent("one", 0, "private");
    i.opportunity_context = Some(OpportunityContextDto {
        profile_snapshot_id: "profile:alice".to_owned(),
    });
    store
        .begin(&f.tenant, &f.user, &c.id, i, true)
        .expect("begin");
    store
        .finish(
            &f.tenant,
            &f.user,
            &c.id,
            "one",
            Ok(response("private answer")),
        )
        .expect("finish");
    let BeginTurn::New { request, .. } = store
        .begin(&f.tenant, &f.user, &c.id, intent("two", 2, "public"), false)
        .expect("begin")
    else {
        panic!("new")
    };
    assert_eq!(request.messages.len(), 1);
    store
        .finish(
            &f.tenant,
            &f.user,
            &c.id,
            "two",
            Ok(response(&"a".repeat(4097))),
        )
        .expect("finish");
    let BeginTurn::New { request, .. } = store
        .begin(&f.tenant, &f.user, &c.id, intent("three", 4, "next"), false)
        .expect("begin")
    else {
        panic!("new")
    };
    assert_eq!(request.messages.len(), 1);
}
#[test]
fn continuation_keeps_pairs_without_replaying_preferences() {
    let f = Fixture::new();
    let store = f.store();
    let c = f.create(&store);
    let mut i = intent("one", 0, "first");
    i.prompt_customization =
        PromptCustomizationFieldDto::Value(crate::agent_chat::PromptCustomizationDto {
            text: "concise".to_owned(),
        });
    store
        .begin(&f.tenant, &f.user, &c.id, i, false)
        .expect("begin");
    store
        .finish(&f.tenant, &f.user, &c.id, "one", Ok(response("answer")))
        .expect("finish");
    let BeginTurn::New { request, .. } = store
        .begin(&f.tenant, &f.user, &c.id, intent("two", 2, "second"), false)
        .expect("begin")
    else {
        panic!("new")
    };
    assert_eq!(
        request
            .messages
            .iter()
            .map(|m| m.content.as_str())
            .collect::<Vec<_>>(),
        vec!["first", "answer", "second"]
    );
    assert!(matches!(
        request.prompt_customization,
        PromptCustomizationFieldDto::Absent
    ));
}
#[test]
fn exclusive_writer_and_corrupt_state_fail_closed() {
    let f = Fixture::new();
    let store = f.store();
    assert!(matches!(
        ConversationStore::open(f.path()),
        Err(ConversationError::Unavailable)
    ));
    drop(store);
    fs::write(f.path(), b"{invalid").expect("corrupt");
    assert!(matches!(
        ConversationStore::open(f.path()),
        Err(ConversationError::Unavailable)
    ));
    assert_eq!(fs::read(f.path()).expect("read"), b"{invalid");
}
#[test]
fn uncertain_write_poison_blocks_all_further_operations() {
    let f = Fixture::new();
    let store = f.store();
    let c = f.create(&store);
    store
        .begin(&f.tenant, &f.user, &c.id, intent("one", 0, "hello"), false)
        .expect("begin");
    fs::set_permissions(f.path(), fs::Permissions::from_mode(0o644)).expect("tamper");
    assert!(matches!(
        store.finish(&f.tenant, &f.user, &c.id, "one", Ok(response("answer"))),
        Err(ConversationError::Unavailable)
    ));
    fs::set_permissions(f.path(), fs::Permissions::from_mode(0o600)).expect("repair mode");
    assert!(matches!(
        store.create(&f.tenant, &f.user, "new"),
        Err(ConversationError::Unavailable)
    ));
    assert!(matches!(
        store.list(&f.tenant, &f.user),
        Err(ConversationError::Unavailable)
    ));
}
#[test]
fn simultaneous_begin_only_reserves_once() {
    let f = Fixture::new();
    let store = std::sync::Arc::new(f.store());
    let c = f.create(&store);
    std::thread::scope(|scope| {
        let tasks = (0..8)
            .map(|_| {
                scope.spawn(|| {
                    store.begin(&f.tenant, &f.user, &c.id, intent("one", 0, "hello"), false)
                })
            })
            .collect::<Vec<_>>();
        let mut starts = 0;
        for task in tasks {
            match task.join().expect("join") {
                Ok(BeginTurn::New { .. }) => starts += 1,
                Err(ConversationError::InProgress) => {}
                _ => panic!("unexpected"),
            }
        }
        assert_eq!(starts, 1);
    });
}

#[test]
fn missing_initialized_state_cannot_reset_duplicate_evidence() {
    let f = Fixture::new();
    let store = f.store();
    f.create(&store);
    drop(store);
    fs::remove_file(f.path()).expect("simulate missing state");
    assert!(matches!(
        ConversationStore::open(f.path()),
        Err(ConversationError::Unavailable)
    ));
    assert!(!f.path().exists());
}

#[test]
fn altered_completed_response_and_unknown_error_fail_closed_on_reopen() {
    for unknown_error in [false, true] {
        let f = Fixture::new();
        let store = f.store();
        let c = f.create(&store);
        store
            .begin(&f.tenant, &f.user, &c.id, intent("one", 0, "hello"), false)
            .expect("begin");
        store
            .finish(&f.tenant, &f.user, &c.id, "one", Ok(response("answer")))
            .expect("finish");
        drop(store);
        let mut state: serde_json::Value =
            serde_json::from_slice(&fs::read(f.path()).expect("read")).expect("state");
        let turn = &mut state["conversations"][0]["turns"][0]["view"];
        if unknown_error {
            turn["phase"] = "failed".into();
            turn["response"] = serde_json::Value::Null;
            turn["error"] = "invented_error".into();
        } else {
            turn["response"]["unexpected_authority"] = true.into();
        }
        fs::write(f.path(), serde_json::to_vec(&state).expect("json")).expect("tamper");
        assert!(matches!(
            ConversationStore::open(f.path()),
            Err(ConversationError::Unavailable)
        ));
    }
}
#[test]
fn worst_case_terminal_json_fits_reserved_space() {
    let value = serde_json::json!({"schema":"ustc-agent-chat-response/v1","run_id":"chat-run:".to_owned()+&"x".repeat(110),"answer":"\u{1}".repeat(16*1024),"provider":{"mode":"openai-compatible","model":"\u{1}".repeat(256)},"tool_trace":(0..4).map(|_|serde_json::json!({"call_id":"\u{1}".repeat(256),"tool":"opportunity_graph_plan_current_profile","status":"succeeded"})).collect::<Vec<_>>(),"usage":{"input_tokens":u64::MAX,"output_tokens":u64::MAX}});
    assert!(valid_response(&value));
    assert!(serde_json::to_vec(&value).expect("json").len() + 1024 < 128 * 1024);
}

#[test]
fn model_selection_schema_presence_and_legacy_digest_remain_stable() {
    let f = Fixture::new();
    let store = f.store();
    let c = f.create(&store);
    let legacy = intent("turn-1", 0, "hello");
    let bytes = serde_json::to_vec(&(&legacy, false)).expect("legacy encoding");
    assert_eq!(
        String::from_utf8(bytes.clone()).expect("UTF8"),
        r#"[{"schema":"chat-conversation-turn/v1","request_id":"turn-1","expected_revision":0,"message":"hello","opportunity_context":null,"prompt_customization":"Absent"},false]"#
    );
    assert_eq!(
        format!("{:x}", Sha256::digest(&bytes)),
        "fa31c6a88ee37314d02384b8fa35aa1e2a7b4a2c6189fde453d6122fd08a4209"
    );
    for value in [
        serde_json::json!({"schema":"chat-conversation-turn/v1","request_id":"bad","expected_revision":0,"message":"hello","model_id":"default"}),
        serde_json::json!({"schema":"chat-conversation-turn/v2","request_id":"bad","expected_revision":0,"message":"hello"}),
        serde_json::json!({"schema":"chat-conversation-turn/v2","request_id":"bad","expected_revision":0,"message":"hello","model_id":"invalid id"}),
    ] {
        let dto = serde_json::from_value(value).expect("typed envelope");
        assert!(matches!(
            store.begin(&f.tenant, &f.user, &c.id, dto, false),
            Err(ConversationError::InvalidIntent)
        ));
    }
    for value in [serde_json::Value::Null, serde_json::json!(12)] {
        assert!(serde_json::from_value::<ConversationTurnIntentDto>(serde_json::json!({"schema":"chat-conversation-turn/v2","request_id":"bad","expected_revision":0,"message":"hello","model_id":value})).is_err());
    }
    assert_eq!(
        store
            .get(&f.tenant, &f.user, &c.id)
            .expect("unchanged")
            .revision,
        0
    );
}

#[test]
fn saved_plugin_trace_and_real_model_identity_finish_and_reopen() {
    let f = Fixture::new();
    let store = f.store();
    let c = f.create(&store);
    let mut selected = intent("plugin-turn", 0, "read installed skill");
    selected.schema = "chat-conversation-turn/v2".into();
    selected.model_id =
        crate::model_catalog::ModelSelectionFieldDto::Value("synthetic.model".into());
    assert!(matches!(
        store.begin(&f.tenant, &f.user, &c.id, selected.clone(), false),
        Ok(BeginTurn::New { .. })
    ));
    let mut result = response("Synthetic plugin result");
    result.provider = ProviderIdentity {
        mode: "openai-compatible".into(),
        model: "synthetic/provider-model:2026".into(),
    };
    result.tool_trace.push(crate::agent_chat::ChatToolTraceDto {
        call_id: "synthetic-tool".into(),
        tool: "plugin_tool".into(),
        status: crate::chat_tools::ChatToolStatus::Succeeded,
    });
    let original = store
        .finish(&f.tenant, &f.user, &c.id, &selected.request_id, Ok(result))
        .expect("plugin result persisted");
    drop(store);
    let store = f.store();
    let Ok(BeginTurn::Replay(replay)) =
        store.begin_admitted(&f.tenant, &f.user, &c.id, selected.clone(), false, |_| {
            panic!("terminal replay must bypass today's catalog")
        })
    else {
        panic!("replay expected")
    };
    assert_eq!(
        serde_json::to_value(original).expect("original"),
        serde_json::to_value(replay).expect("replay")
    );
    selected.model_id = crate::model_catalog::ModelSelectionFieldDto::Value("removed.model".into());
    assert!(matches!(
        store.begin_admitted(&f.tenant, &f.user, &c.id, selected, false, |_| panic!(
            "conflict before selection"
        )),
        Err(ConversationError::RequestConflict)
    ));
}

#[path = "automatic_title_tests.rs"]
mod automatic_title_tests;
