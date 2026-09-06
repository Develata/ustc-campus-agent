//! Application composition of a reserved dialogue turn and the existing finite run.
//! The spawned operation outlives a disconnected HTTP waiter; recovery never redispatches it.

use std::path::PathBuf;
use std::sync::Arc;

use ustc_campus_agent_core::identity::{TenantId, UserId};

use crate::agent_chat::ChatError;
use crate::agent_chat::run_bounded_chat_with_observer;
use crate::chat_activity::{ActivityRegistry, ChatActivityDto};
use crate::chat_conversations::{
    BeginTurn, ConversationDto, ConversationError, ConversationListDto,
    ConversationManageIntentDto, ConversationManageResultDto, ConversationStore,
    ConversationTurnIntentDto, ConversationTurnResultDto,
};
#[cfg(test)]
use crate::chat_provider::ChatProvider;
use crate::chat_tools::ChatToolExecutor;
use crate::model_catalog::ModelCatalog;

#[derive(Clone)]
pub(crate) struct ConversationApplication {
    store: Arc<ConversationStore>,
    models: ModelCatalog,
    activity: Arc<ActivityRegistry>,
}

impl ConversationApplication {
    pub(crate) fn open(
        path: PathBuf,
        models: impl Into<ModelCatalog>,
    ) -> Result<Self, ConversationError> {
        Ok(Self {
            store: Arc::new(ConversationStore::open(path)?),
            models: models.into(),
            activity: Arc::new(ActivityRegistry::default()),
        })
    }

    pub(crate) fn root_prompt(
        &self,
        tenant: &TenantId,
        user: &UserId,
    ) -> Result<crate::chat_conversations::RootPromptDto, ConversationError> {
        self.store.root_prompt(tenant, user)
    }

    pub(crate) fn update_root_prompt(
        &self,
        tenant: &TenantId,
        user: &UserId,
        intent: crate::chat_conversations::RootPromptUpdateDto,
    ) -> Result<crate::chat_conversations::RootPromptDto, ConversationError> {
        self.store.update_root_prompt(tenant, user, intent)
    }

    pub(crate) fn list(
        &self,
        tenant: &TenantId,
        user: &UserId,
    ) -> Result<ConversationListDto, ConversationError> {
        self.store.list(tenant, user)
    }

    pub(crate) fn create(
        &self,
        tenant: &TenantId,
        user: &UserId,
        request_id: &str,
    ) -> Result<ConversationDto, ConversationError> {
        self.store.create(tenant, user, request_id)
    }

    pub(crate) fn get(
        &self,
        tenant: &TenantId,
        user: &UserId,
        id: &str,
    ) -> Result<ConversationDto, ConversationError> {
        self.store.get(tenant, user, id)
    }

    pub(crate) fn manage(
        &self,
        tenant: &TenantId,
        user: &UserId,
        id: &str,
        intent: ConversationManageIntentDto,
    ) -> Result<ConversationManageResultDto, ConversationError> {
        self.store.manage(tenant, user, id, intent)
    }

    pub(crate) fn activity(
        &self,
        tenant: &TenantId,
        user: &UserId,
        id: &str,
    ) -> Result<ChatActivityDto, ConversationError> {
        let turn = self.store.current_turn(tenant, user, id)?;
        Ok(self.activity.project(id, turn.as_ref()))
    }

    #[cfg(test)]
    pub(crate) async fn submit<E>(
        &self,
        owner: (TenantId, UserId),
        id: String,
        intent: ConversationTurnIntentDto,
        confirmed: bool,
        executor: E,
    ) -> Result<ConversationTurnResultDto, ConversationError>
    where
        E: ChatToolExecutor + Send + 'static,
    {
        self.submit_with_executor_factory(owner, id, intent, confirmed, |_| async { Ok(executor) })
            .await
    }
    pub(crate) async fn submit_with_executor_factory<E, F, Fut>(
        &self,
        owner: (TenantId, UserId),
        id: String,
        intent: ConversationTurnIntentDto,
        confirmed: bool,
        make_executor: F,
    ) -> Result<ConversationTurnResultDto, ConversationError>
    where
        E: ChatToolExecutor + Send + 'static,
        F: FnOnce(bool) -> Fut + Send + 'static,
        Fut: std::future::Future<Output = Result<E, ChatError>> + Send,
    {
        let application = self.clone();
        // Reservation and dispatch outlive a dropped HTTP waiter. Replay never selects
        // today's provider or discovers today's plugin inventory.
        tokio::spawn(async move {
            let request_id = intent.request_id.clone();
            let (tenant, user) = owner;
            let mut selected = None;
            match application.store.begin_admitted(
                &tenant,
                &user,
                &id,
                intent,
                confirmed,
                |request| {
                    let model_id = request
                        .selected_model_id()
                        .map_err(ConversationError::InvalidChat)?;
                    selected = Some(application.models.resolve(model_id).map_err(|_| {
                        ConversationError::InvalidChat(ChatError::InvalidChatRequest)
                    })?);
                    Ok(())
                },
            )? {
                BeginTurn::Replay(result) => Ok(result),
                BeginTurn::New {
                    run_id,
                    request,
                    title_request,
                } => {
                    let mut observer = application.activity.register(&id, &request_id);
                    let outcome = match (
                        selected.as_ref(),
                        make_executor(
                            selected
                                .as_ref()
                                .is_some_and(|provider| provider.tool_calling_enabled()),
                        )
                        .await,
                    ) {
                        (Some(provider), Ok(mut executor)) => {
                            run_bounded_chat_with_observer(
                                run_id,
                                request,
                                confirmed,
                                provider,
                                &mut executor,
                                &mut observer,
                            )
                            .await
                        }
                        (_, Err(error)) => Err(error),
                        (None, _) => Err(ChatError::Internal),
                    };
                    let generated_title = if outcome.is_ok() {
                        match (selected.as_ref(), title_request.as_deref()) {
                            (Some(provider), Some(message)) => {
                                crate::conversation_title::provider::generate(provider, message)
                                    .await
                            }
                            _ => None,
                        }
                    } else {
                        None
                    };
                    // Factory failure is also terminal evidence, never a stranded reservation.
                    application.store.finish_with_title(
                        &tenant,
                        &user,
                        &id,
                        &request_id,
                        outcome,
                        generated_title,
                    )
                }
            }
        })
        .await
        .map_err(|_| ConversationError::Unavailable)?
    }
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use crate::agent_chat::PromptCustomizationFieldDto;
    use crate::chat_activity::{ActivityPhase, ActivityStatus};
    use crate::chat_tools::ChatToolExecution;
    use std::os::unix::fs::PermissionsExt;

    #[tokio::test]
    async fn activity_application_owner_admission_read_only_and_reopen_projection() {
        let suffix = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("valid activity test fixture")
            .as_nanos();
        let directory =
            std::env::temp_dir().join(format!("uca-activity-{}-{suffix}", std::process::id()));
        std::fs::create_dir(&directory).expect("valid activity test fixture");
        std::fs::set_permissions(&directory, std::fs::Permissions::from_mode(0o700))
            .expect("valid activity test fixture");
        let path = directory.join("conversations.json");
        let app = ConversationApplication::open(path.clone(), ChatProvider::deterministic_mock())
            .expect("valid activity test fixture");
        let tenant = TenantId::parse("tenant:activity").expect("valid activity test fixture");
        let user = UserId::parse("user:alice").expect("valid activity test fixture");
        let other = UserId::parse("user:bob").expect("valid activity test fixture");
        let other_tenant = TenantId::parse("tenant:other").expect("valid activity test fixture");
        let conversation = app
            .create(&tenant, &user, "create")
            .expect("valid activity test fixture");
        assert_eq!(
            app.activity(&tenant, &user, &conversation.id)
                .expect("valid activity test fixture")
                .phase,
            ActivityPhase::Idle
        );
        for (t, u, id) in [
            (&tenant, &other, conversation.id.as_str()),
            (&other_tenant, &user, conversation.id.as_str()),
            (&tenant, &user, "missing"),
        ] {
            assert!(matches!(
                app.activity(t, u, id),
                Err(ConversationError::NotFound)
            ));
        }
        let intent = ConversationTurnIntentDto {
            schema: "chat-conversation-turn/v1".to_owned(),
            model_id: crate::model_catalog::ModelSelectionFieldDto::Absent,
            request_id: "request".to_owned(),
            expected_revision: 0,
            message: "列出日历事项".to_owned(),
            opportunity_context: None,
            prompt_customization: PromptCustomizationFieldDto::Absent,
        };
        app.submit(
            (tenant.clone(), user.clone()),
            conversation.id.clone(),
            intent,
            false,
            |_| ChatToolExecution::succeeded(serde_json::json!({"items":[]})),
        )
        .await
        .expect("valid activity test fixture");
        let before = std::fs::read(&path).expect("valid activity test fixture");
        let dto = app
            .activity(&tenant, &user, &conversation.id)
            .expect("valid activity test fixture");
        assert_eq!(dto.phase, ActivityPhase::Completed);
        assert_eq!(dto.sequence, 15);
        assert_eq!(dto.steps.len(), 1);
        assert_eq!(dto.steps[0].status, ActivityStatus::Succeeded);
        assert_eq!(
            std::fs::read(&path).expect("valid activity test fixture"),
            before
        );
        drop(app);
        let app = ConversationApplication::open(path, ChatProvider::deterministic_mock())
            .expect("valid activity test fixture");
        let reopened = app
            .activity(&tenant, &user, &conversation.id)
            .expect("valid activity test fixture");
        assert_eq!(reopened.steps, dto.steps);
        assert_eq!(reopened.request_id, dto.request_id);
        drop(app);
        std::fs::remove_dir_all(directory).expect("valid activity test fixture");
    }
}

#[cfg(all(test, unix))]
mod model_selection_tests {
    use super::*;
    use crate::chat_conversations::TurnPhase;
    use crate::chat_tools::{ChatToolExecution, ChatToolRequest};
    use std::{
        fs,
        os::unix::fs::PermissionsExt,
        sync::atomic::{AtomicUsize, Ordering},
    };
    #[tokio::test]
    async fn unknown_selection_has_no_reservation_or_factory_and_factory_failure_is_terminal() {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "uca-model-application-{}-{nonce}",
            std::process::id()
        ));
        fs::create_dir(&root).expect("directory");
        fs::set_permissions(&root, fs::Permissions::from_mode(0o700)).expect("private");
        let path = root.join("conversations.json");
        let app = ConversationApplication::open(path.clone(), ChatProvider::deterministic_mock())
            .expect("application");
        let owner = (
            TenantId::parse("tenant:model-test").expect("tenant"),
            UserId::parse("user:model-test").expect("user"),
        );
        let conversation = app
            .create(&owner.0, &owner.1, "create")
            .expect("conversation");
        let calls = Arc::new(AtomicUsize::new(0));
        let intent:ConversationTurnIntentDto=serde_json::from_value(serde_json::json!({"schema":"chat-conversation-turn/v2","model_id":"default","request_id":"failure","expected_revision":0,"message":"Hello"})).expect("intent");
        let mut unknown = intent.clone();
        unknown.model_id = crate::model_catalog::ModelSelectionFieldDto::Value("unknown".into());
        let before = fs::read(&path).expect("before");
        let counted = Arc::clone(&calls);
        assert!(matches!(
            app.submit_with_executor_factory(
                owner.clone(),
                conversation.id.clone(),
                unknown,
                false,
                move |_| async move {
                    counted.fetch_add(1, Ordering::SeqCst);
                    Err::<fn(ChatToolRequest) -> ChatToolExecution, _>(
                        ChatError::CompositionUnavailable,
                    )
                }
            )
            .await,
            Err(ConversationError::InvalidChat(
                ChatError::InvalidChatRequest
            ))
        ));
        assert_eq!(fs::read(&path).expect("unchanged"), before);
        assert_eq!(calls.load(Ordering::SeqCst), 0);
        let counted = Arc::clone(&calls);
        let failure = app
            .submit_with_executor_factory(
                owner.clone(),
                conversation.id.clone(),
                intent.clone(),
                false,
                move |tools| async move {
                    assert!(tools, "mock supports tool calling");
                    counted.fetch_add(1, Ordering::SeqCst);
                    Err::<fn(ChatToolRequest) -> ChatToolExecution, _>(
                        ChatError::CompositionUnavailable,
                    )
                },
            )
            .await
            .expect("terminal failure receipt");
        assert_eq!(failure.turn.phase, TurnPhase::Failed);
        assert_eq!(
            failure.turn.error.as_deref(),
            Some("composition_unavailable")
        );
        assert_eq!(failure.revision, 2);
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        drop(app);
        let app = ConversationApplication::open(path, ChatProvider::deterministic_mock())
            .expect("reopen");
        let counted = Arc::clone(&calls);
        let replay = app
            .submit_with_executor_factory(
                owner,
                conversation.id,
                intent,
                false,
                move |_| async move {
                    counted.fetch_add(1, Ordering::SeqCst);
                    Err::<fn(ChatToolRequest) -> ChatToolExecution, _>(ChatError::Internal)
                },
            )
            .await
            .expect("failed terminal replay");
        assert_eq!(
            serde_json::to_value(failure).expect("failure"),
            serde_json::to_value(replay).expect("replay")
        );
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        drop(app);
        fs::remove_dir_all(root).expect("owned fixture cleanup");
    }
}

#[cfg(all(test, unix))]
mod automatic_title_tests {
    use super::*;
    use crate::chat_conversations::TurnPhase;
    use crate::chat_tools::{ChatToolExecution, ChatToolRequest};
    use crate::conversation_title::provider::test_support::{Peer, answer};
    use serde_json::json;

    fn owner() -> (TenantId, UserId) {
        (
            TenantId::parse("tenant:title-test").expect("tenant"),
            UserId::parse("user:title-test").expect("user"),
        )
    }
    fn no_tools(_: ChatToolRequest) -> ChatToolExecution {
        panic!("title fixture must never execute tools")
    }
    fn intent(request_id: &str, revision: u64, message: &str) -> ConversationTurnIntentDto {
        serde_json::from_value(json!({"schema":"chat-conversation-turn/v2","model_id":"chosen","request_id":request_id,"expected_revision":revision,"message":message})).expect("intent")
    }
    fn application(peer: &Peer) -> ConversationApplication {
        let file = peer.root.join("models.json");
        std::fs::write(&file,serde_json::to_vec(&json!({"schema":"uca-agent-models/v1","models":[{"id":"chosen","label":"Chosen fixture","mode":"local-chat","base_url":peer.url,"model":"title-selected-model","api_key_file":peer.root.join("key"),"timeout_ms":10000,"context_limit_tokens":65536}]})).expect("catalog")).expect("catalog file");
        let models =
            ModelCatalog::from_file(ChatProvider::deterministic_mock(), &file).expect("catalog");
        ConversationApplication::open(peer.root.join("conversations.json"), models)
            .expect("application")
    }

    #[tokio::test]
    async fn root_prompt_reaches_controlled_provider_frozen_and_never_title_or_replay() {
        use crate::chat_conversations::RootPromptUpdateDto;
        let peer = Peer::start(vec![
            Some((200, answer("first reply"))),
            Some((200, answer("Test title"))),
            Some((200, answer("second reply"))),
        ]);
        let app = application(&peer);
        let owner = owner();
        let conversation = app.create(&owner.0, &owner.1, "create").expect("create");
        app.update_root_prompt(
            &owner.0,
            &owner.1,
            RootPromptUpdateDto {
                schema: "agent-root-prompt-update/v1".to_owned(),
                expected_revision: 0,
                text: "saved-root-old-marker".to_owned(),
            },
        )
        .expect("save original");
        let first = intent("first", 0, "hello");
        let changing_app = app.clone();
        let changing_owner = owner.clone();
        let result = app
            .submit_with_executor_factory(
                owner.clone(),
                conversation.id.clone(),
                first.clone(),
                false,
                move |_| async move {
                    changing_app
                        .update_root_prompt(
                            &changing_owner.0,
                            &changing_owner.1,
                            RootPromptUpdateDto {
                                schema: "agent-root-prompt-update/v1".to_owned(),
                                expected_revision: 1,
                                text: "saved-root-new-marker".to_owned(),
                            },
                        )
                        .expect("change after reservation");
                    Ok(no_tools as fn(ChatToolRequest) -> ChatToolExecution)
                },
            )
            .await
            .expect("first result");
        assert_eq!(result.turn.phase, TurnPhase::Completed);
        {
            let wire = peer.requests.lock().expect("requests");
            assert!(wire[0].to_string().contains("saved-root-old-marker"));
            assert!(!wire[0].to_string().contains("saved-root-new-marker"));
            assert!(
                !wire[1].to_string().contains("saved-root-"),
                "title excludes private instruction"
            );
        }
        app.submit(
            owner.clone(),
            conversation.id.clone(),
            intent("second", result.revision, "next"),
            false,
            no_tools,
        )
        .await
        .expect("second");
        {
            let wire = peer.requests.lock().expect("requests");
            assert!(wire[2].to_string().contains("saved-root-new-marker"));
            assert!(!wire[2].to_string().contains("saved-root-old-marker"));
        }
        assert!(
            !serde_json::to_string(
                &app.get(&owner.0, &owner.1, &conversation.id)
                    .expect("public")
            )
            .expect("json")
            .contains("saved-root-")
        );
        app.submit(owner, conversation.id, first, false, no_tools)
            .await
            .expect("replay");
        assert_eq!(peer.count(), 3);
    }

    #[tokio::test]
    async fn automatic_title_uses_first_selected_provider_without_tools_and_replay_never_regenerates()
     {
        let peer = Peer::start(vec![
            Some((200, answer("已回答选课问题"))),
            Some((200, answer("选课准备"))),
            Some((200, answer("后续回答"))),
        ]);
        let app = application(&peer);
        let owner = owner();
        let conversation = app.create(&owner.0, &owner.1, "create").expect("create");
        let first = intent("first", 0, "如何安排下学期选课？");
        let result = app
            .submit(
                owner.clone(),
                conversation.id.clone(),
                first.clone(),
                false,
                no_tools,
            )
            .await
            .expect("first result");
        assert_eq!(result.turn.phase, TurnPhase::Completed);
        assert_eq!(peer.count(), 2);
        assert_eq!(result.revision, 2, "title shares atomic finish revision");
        let detail = app
            .get(&owner.0, &owner.1, &conversation.id)
            .expect("detail");
        assert!(detail.title.ends_with("|选课准备"));
        {
            let wire = peer.requests.lock().expect("wire");
            let naming = &wire[1];
            assert_eq!(naming["model"], "title-selected-model");
            assert!(naming.get("tools").is_none());
            assert_eq!(naming["messages"].as_array().expect("messages").len(), 2);
            assert_eq!(naming["messages"][1]["content"], first.message);
            assert!(
                naming["messages"]
                    .as_array()
                    .expect("messages")
                    .iter()
                    .all(
                        |entry| ["system", "user"].contains(&entry["role"].as_str().expect("role"))
                    )
            );
        }
        app.submit(
            owner.clone(),
            conversation.id.clone(),
            intent("second", result.revision, "继续说明"),
            false,
            no_tools,
        )
        .await
        .expect("later turn");
        assert_eq!(peer.count(), 3, "later turns do not generate another title");
        assert_eq!(
            app.get(&owner.0, &owner.1, &conversation.id)
                .expect("detail")
                .title,
            detail.title
        );
        drop(app);
        let reopened = ConversationApplication::open(
            peer.root.join("conversations.json"),
            ChatProvider::deterministic_mock(),
        )
        .expect("reopen without selected model");
        let replay = reopened
            .submit(owner, conversation.id, first, false, no_tools)
            .await
            .expect("exact stored replay");
        assert_eq!(
            serde_json::to_value(&replay.turn).expect("replay"),
            serde_json::to_value(&result.turn).expect("result")
        );
        assert_eq!(replay.revision, result.revision);
        assert_eq!(peer.count(), 3);
    }

    #[tokio::test]
    async fn automatic_title_failure_is_nonfatal_and_failed_chat_never_requests_a_title() {
        for failure in [false, true] {
            let replies = if failure {
                vec![Some((503, json!({"error":"synthetic chat failure"})))]
            } else {
                vec![
                    Some((200, answer("聊天成功"))),
                    Some((503, json!({"error":"synthetic title failure"}))),
                ]
            };
            let peer = Peer::start(replies);
            let app = application(&peer);
            let owner = owner();
            let conversation = app.create(&owner.0, &owner.1, "create").expect("create");
            let result = app
                .submit(
                    owner.clone(),
                    conversation.id.clone(),
                    intent("first", 0, "校历问题"),
                    false,
                    no_tools,
                )
                .await
                .expect("terminal result");
            assert_eq!(
                result.turn.phase,
                if failure {
                    TurnPhase::Failed
                } else {
                    TurnPhase::Completed
                }
            );
            assert_eq!(peer.count(), if failure { 1 } else { 2 });
            assert!(
                app.get(&owner.0, &owner.1, &conversation.id)
                    .expect("fallback")
                    .title
                    .ends_with("|校历问题")
            );
        }
    }

    #[tokio::test]
    async fn automatic_title_preserves_explicit_name_without_a_naming_request() {
        let peer = Peer::start(vec![Some((200, answer("正常回答")))]);
        let app = application(&peer);
        let owner = owner();
        let conversation = app.create(&owner.0, &owner.1, "create").expect("create");
        let rename:ConversationManageIntentDto=serde_json::from_value(json!({"schema":"chat-conversation-manage/v1","request_id":"rename","expected_revision":0,"action":{"kind":"rename","title":"手动名称"}})).expect("rename");
        let receipt = app
            .manage(&owner.0, &owner.1, &conversation.id, rename)
            .expect("explicit name");
        app.submit(
            owner.clone(),
            conversation.id.clone(),
            intent("first", receipt.revision, "首次提问"),
            false,
            no_tools,
        )
        .await
        .expect("turn");
        assert_eq!(peer.count(), 1);
        assert_eq!(
            app.get(&owner.0, &owner.1, &conversation.id)
                .expect("detail")
                .title,
            "手动名称"
        );
    }
}
