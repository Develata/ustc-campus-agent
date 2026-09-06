//! Real asynchronous stop and restart-recovery tests.
use super::*;
use crate::agent_chat::{ChatActivityObserver, ChatActivityTool};
use crate::chat_activity::{ActivityKind, ActivityStatus, ActivityStepDto, ChatProgress};
use crate::chat_conversations::TurnPhase;
use crate::chat_tools::{ChatToolExecution, ChatToolRequest};
use std::os::unix::fs::PermissionsExt;
use std::sync::atomic::{AtomicBool, Ordering};

struct PendingExecutor {
    entered: Arc<tokio::sync::Notify>,
    dropped: Arc<AtomicBool>,
}
impl Drop for PendingExecutor {
    fn drop(&mut self) {
        self.dropped.store(true, Ordering::SeqCst);
    }
}
impl ChatToolExecutor for PendingExecutor {
    async fn execute(&mut self, _: ChatToolRequest) -> ChatToolExecution {
        self.entered.notify_one();
        std::future::pending().await
    }
}
fn fixture() -> (
    std::path::PathBuf,
    ConversationApplication,
    (TenantId, UserId),
) {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("controlled execution fixture")
        .as_nanos();
    let root = std::env::temp_dir().join(format!("uca-execution-{}-{nonce}", std::process::id()));
    std::fs::create_dir(&root).expect("controlled execution fixture");
    std::fs::set_permissions(&root, std::fs::Permissions::from_mode(0o700))
        .expect("controlled execution fixture");
    let app = ConversationApplication::open(
        root.join("conversations.json"),
        ChatProvider::deterministic_mock(),
    )
    .expect("controlled execution fixture");
    (
        root,
        app,
        (
            TenantId::parse("tenant:execution").expect("controlled execution fixture"),
            UserId::parse("user:alice").expect("controlled execution fixture"),
        ),
    )
}
fn intent(request: &str, revision: u64) -> ConversationTurnIntentDto {
    serde_json::from_value(serde_json::json!({"schema":"chat-conversation-turn/v1","request_id":request,"expected_revision":revision,"message":"列出日历事项"})).expect("controlled execution fixture")
}
#[tokio::test]
async fn cancellation_drops_pending_executor_is_owner_scoped_and_frozen_to_request() {
    let (root, app, owner) = fixture();
    let conversation = app
        .create(&owner.0, &owner.1, "create")
        .expect("controlled execution fixture");
    let entered = Arc::new(tokio::sync::Notify::new());
    let dropped = Arc::new(AtomicBool::new(false));
    let task_app = app.clone();
    let task_owner = owner.clone();
    let id = conversation.id.clone();
    let executor = PendingExecutor {
        entered: entered.clone(),
        dropped: dropped.clone(),
    };
    let task = tokio::spawn(async move {
        task_app
            .submit(task_owner, id, intent("first", 0), false, executor)
            .await
    });
    tokio::time::timeout(std::time::Duration::from_secs(3), entered.notified())
        .await
        .expect("controlled execution fixture");
    assert!(matches!(
        app.cancel(
            &owner.0,
            &UserId::parse("user:bob").expect("controlled execution fixture"),
            &conversation.id,
            "first"
        ),
        Err(ConversationError::NotFound)
    ));
    assert!(matches!(
        app.cancel(&owner.0, &owner.1, &conversation.id, "stale"),
        Err(ConversationError::RequestConflict)
    ));
    app.cancel(&owner.0, &owner.1, &conversation.id, "first")
        .expect("controlled execution fixture");
    let result = tokio::time::timeout(std::time::Duration::from_secs(3), task)
        .await
        .expect("controlled execution fixture")
        .expect("controlled execution fixture")
        .expect("controlled execution fixture");
    assert_eq!(result.turn.error.as_deref(), Some("chat_cancelled"));
    assert!(dropped.load(Ordering::SeqCst));
    let replay = app
        .submit(
            owner.clone(),
            conversation.id.clone(),
            intent("first", 0),
            false,
            |_| panic!("cancelled replay must not execute"),
        )
        .await
        .expect("controlled execution fixture");
    assert_eq!(replay.turn.error, result.turn.error);
    let completed = app
        .submit(
            owner.clone(),
            conversation.id.clone(),
            intent("second", result.revision),
            false,
            |_| ChatToolExecution::succeeded(serde_json::json!({"items":[]})),
        )
        .await
        .expect("controlled execution fixture");
    assert_eq!(completed.turn.phase, TurnPhase::Completed);
    assert!(matches!(
        app.cancel(&owner.0, &owner.1, &conversation.id, "first"),
        Err(ConversationError::RequestConflict)
    ));
    drop(app);
    let reopened = ConversationApplication::open(
        root.join("conversations.json"),
        ChatProvider::deterministic_mock(),
    )
    .expect("controlled execution fixture");
    assert_eq!(
        reopened
            .get(&owner.0, &owner.1, &conversation.id)
            .expect("controlled execution fixture")
            .turns[0]
            .error
            .as_deref(),
        Some("chat_cancelled")
    );
    drop(reopened);
    std::fs::remove_dir_all(root).expect("controlled execution fixture");
}
#[test]
fn checkpoint_reopen_retains_completed_results_without_reexecution() {
    let (root, app, owner) = fixture();
    let conversation = app
        .create(&owner.0, &owner.1, "create")
        .expect("controlled execution fixture");
    app.store
        .begin_admitted(
            &owner.0,
            &owner.1,
            &conversation.id,
            intent("first", 0),
            false,
            |_| Ok(()),
        )
        .expect("controlled execution fixture");
    let progress = ChatProgress {
        steps: vec![ActivityStepDto {
            id: "call-1".into(),
            kind: ActivityKind::Tool,
            tool: Some(ChatActivityTool::CalendarItems),
            status: ActivityStatus::Succeeded,
        }],
        partial_answer: "partial server response".into(),
        tool_results: vec![
            serde_json::json!({"schema":"ustc-agent-chat-tool-result/v1","trust":"untrusted_data","status":"succeeded","data":{"receipt_id":"receipt:synthetic"}}),
        ],
    };
    app.store
        .checkpoint(&owner.0, &owner.1, &conversation.id, "first", &progress)
        .expect("controlled execution fixture");
    drop(app);
    let app = ConversationApplication::open(
        root.join("conversations.json"),
        ChatProvider::deterministic_mock(),
    )
    .expect("controlled execution fixture");
    assert_eq!(
        app.get(&owner.0, &owner.1, &conversation.id)
            .expect("controlled execution fixture")
            .turns[0]
            .phase,
        TurnPhase::Interrupted
    );
    let activity = app
        .activity(&owner.0, &owner.1, &conversation.id)
        .expect("controlled execution fixture");
    assert_eq!(activity.steps[0].status, ActivityStatus::Succeeded);
    assert_eq!(activity.partial_answer, "partial server response");
    assert_eq!(
        app.store
            .progress(&owner.0, &owner.1, &conversation.id, "first")
            .expect("controlled execution fixture")
            .expect("controlled execution fixture")
            .tool_results,
        progress.tool_results
    );
    drop(app);
    std::fs::remove_dir_all(root).expect("controlled execution fixture");
}
#[test]
fn cancellation_flag_blocks_new_calls_even_for_ready_futures() {
    let registry = Arc::new(ActivityRegistry::default());
    let guard = registry.register("c", "r");
    assert!(guard.check_cancelled().is_ok());
    assert!(!registry.cancel("c", "other"));
    assert!(registry.cancel("c", "r"));
    assert_eq!(guard.check_cancelled(), Err(ChatError::Cancelled));
}
