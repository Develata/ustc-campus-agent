//! Owner-scoped durable transcripts around the bounded Chat application loop.
mod automatic_title;
mod management;
mod persistence;
pub(crate) use management::{ConversationManageIntentDto, ConversationManageResultDto};

use std::path::PathBuf;
use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use ustc_campus_agent_core::identity::{TenantId, UserId};

use crate::agent_chat::{
    CHAT_REQUEST_SCHEMA_V2, CHAT_REQUEST_SCHEMA_V3, ChatError, ChatInputMessageDto, ChatInputRole,
    ChatRequestDto, ChatResponseDto, OpportunityContextDto, PromptCustomizationFieldDto,
    validate_chat_request,
};

const MAX_CONVERSATIONS: usize = 50;
const MAX_TURNS: usize = 100;
const MAX_RUNNING: usize = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ConversationError {
    InvalidIntent,
    NotFound,
    RequestConflict,
    RevisionConflict,
    InProgress,
    Capacity,
    Unavailable,
    InvalidChat(ChatError),
}
impl ConversationError {
    pub(crate) fn code(self) -> &'static str {
        match self {
            Self::InvalidIntent => "invalid_conversation_intent",
            Self::NotFound => "conversation_not_found",
            Self::RequestConflict => "conversation_request_conflict",
            Self::RevisionConflict => "conversation_revision_conflict",
            Self::InProgress => "conversation_in_progress",
            Self::Capacity => "conversation_capacity_exceeded",
            Self::Unavailable => "conversation_store_unavailable",
            Self::InvalidChat(error) => error.code(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ConversationTurnIntentDto {
    pub(crate) schema: String,
    #[serde(
        default,
        skip_serializing_if = "crate::model_catalog::ModelSelectionFieldDto::is_absent"
    )]
    pub(crate) model_id: crate::model_catalog::ModelSelectionFieldDto,
    pub(crate) request_id: String,
    pub(crate) expected_revision: u64,
    pub(crate) message: String,
    #[serde(default)]
    pub(crate) opportunity_context: Option<OpportunityContextDto>,
    #[serde(default)]
    pub(crate) prompt_customization: PromptCustomizationFieldDto,
}
#[derive(Debug, Clone, Serialize)]
pub(crate) struct ConversationSummaryDto {
    pub(crate) id: String,
    pub(crate) title: String,
    pub(crate) revision: u64,
    pub(crate) turn_count: usize,
}
#[derive(Debug, Clone, Serialize)]
pub(crate) struct ConversationListDto {
    pub(crate) schema: &'static str,
    pub(crate) conversations: Vec<ConversationSummaryDto>,
}
#[derive(Debug, Clone, Serialize)]
pub(crate) struct ConversationDto {
    pub(crate) schema: &'static str,
    pub(crate) id: String,
    pub(crate) title: String,
    pub(crate) revision: u64,
    pub(crate) turns: Vec<ConversationTurnDto>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ConversationTurnDto {
    pub(crate) request_id: String,
    pub(crate) user: String,
    pub(crate) phase: TurnPhase,
    pub(crate) response: Option<serde_json::Value>,
    pub(crate) error: Option<String>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum TurnPhase {
    Running,
    Completed,
    Failed,
    Interrupted,
}
#[derive(Debug, Clone, Serialize)]
pub(crate) struct ConversationTurnResultDto {
    pub(crate) schema: &'static str,
    pub(crate) conversation_id: String,
    pub(crate) revision: u64,
    pub(crate) turn: ConversationTurnDto,
}
pub(crate) enum BeginTurn {
    New {
        run_id: String,
        request: ChatRequestDto,
        title_request: Option<String>,
    },
    Replay(ConversationTurnResultDto),
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct StoredTurn {
    view: ConversationTurnDto,
    digest: String,
    profile: Option<String>,
    result_revision: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    automatic_title: Option<automatic_title::AutomaticTitle>,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct StoredConversation {
    tenant: String,
    user: String,
    create_request: String,
    id: String,
    title: String,
    revision: u64,
    turns: Vec<StoredTurn>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    management: Vec<management::StoredManagement>,
    #[serde(default, skip_serializing_if = "is_false")]
    deleted: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    explicit_title: bool,
}
fn is_false(value: &bool) -> bool {
    !value
}
impl StoredConversation {
    fn owned_by(&self, tenant: &TenantId, user: &UserId) -> bool {
        self.tenant == tenant.as_str() && self.user == user.as_str()
    }
    fn view(&self) -> ConversationDto {
        ConversationDto {
            schema: "chat-conversation/v1",
            id: self.id.clone(),
            title: self.title.clone(),
            revision: self.revision,
            turns: self.turns.iter().map(|turn| turn.view.clone()).collect(),
        }
    }
    fn result(&self, turn: &StoredTurn) -> ConversationTurnResultDto {
        ConversationTurnResultDto {
            schema: "chat-conversation-turn-result/v1",
            conversation_id: self.id.clone(),
            revision: turn.result_revision.unwrap_or(self.revision),
            turn: turn.view.clone(),
        }
    }
}
#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct State {
    version: u32,
    conversations: Vec<StoredConversation>,
}
struct Inner {
    state: State,
    disk: persistence::Disk,
    poisoned: bool,
}
pub(crate) struct ConversationStore {
    inner: Mutex<Inner>,
}
impl ConversationStore {
    pub(crate) fn open(path: PathBuf) -> Result<Self, ConversationError> {
        let (disk, mut state) = persistence::Disk::open(path)?;
        validate_state(&state)?;
        let mut recovered = false;
        for conversation in &mut state.conversations {
            for turn in &mut conversation.turns {
                if turn.view.phase == TurnPhase::Running {
                    conversation.revision = conversation
                        .revision
                        .checked_add(1)
                        .ok_or(ConversationError::Unavailable)?;
                    turn.view.phase = TurnPhase::Interrupted;
                    turn.view.error = Some("conversation_interrupted".to_owned());
                    turn.result_revision = Some(conversation.revision);
                    recovered = true;
                }
            }
        }
        if recovered {
            disk.save(&state)?;
        }
        Ok(Self {
            inner: Mutex::new(Inner {
                state,
                disk,
                poisoned: false,
            }),
        })
    }
    pub(crate) fn list(
        &self,
        tenant: &TenantId,
        user: &UserId,
    ) -> Result<ConversationListDto, ConversationError> {
        let inner = self
            .inner
            .lock()
            .map_err(|_| ConversationError::Unavailable)?;
        if inner.poisoned {
            return Err(ConversationError::Unavailable);
        }
        Ok(ConversationListDto {
            schema: "chat-conversation-list/v1",
            conversations: inner
                .state
                .conversations
                .iter()
                .filter(|c| c.owned_by(tenant, user) && !c.deleted)
                .rev()
                .map(|c| ConversationSummaryDto {
                    id: c.id.clone(),
                    title: c.title.clone(),
                    revision: c.revision,
                    turn_count: c.turns.len(),
                })
                .collect(),
        })
    }
    pub(crate) fn get(
        &self,
        tenant: &TenantId,
        user: &UserId,
        id: &str,
    ) -> Result<ConversationDto, ConversationError> {
        let inner = self
            .inner
            .lock()
            .map_err(|_| ConversationError::Unavailable)?;
        if inner.poisoned {
            return Err(ConversationError::Unavailable);
        }
        inner
            .state
            .conversations
            .iter()
            .find(|c| c.id == id && c.owned_by(tenant, user) && !c.deleted)
            .map(StoredConversation::view)
            .ok_or(ConversationError::NotFound)
    }
    /// Narrow owner-admitted read for activity polling; never clones old history.
    pub(crate) fn current_turn(
        &self,
        tenant: &TenantId,
        user: &UserId,
        id: &str,
    ) -> Result<Option<ConversationTurnDto>, ConversationError> {
        let inner = self
            .inner
            .lock()
            .map_err(|_| ConversationError::Unavailable)?;
        if inner.poisoned {
            return Err(ConversationError::Unavailable);
        }
        inner
            .state
            .conversations
            .iter()
            .find(|c| c.id == id && c.owned_by(tenant, user) && !c.deleted)
            .map(|c| c.turns.last().map(|turn| turn.view.clone()))
            .ok_or(ConversationError::NotFound)
    }

    pub(crate) fn create(
        &self,
        tenant: &TenantId,
        user: &UserId,
        request_id: &str,
    ) -> Result<ConversationDto, ConversationError> {
        if !valid_request_id(request_id) {
            return Err(ConversationError::InvalidIntent);
        }
        let mut inner = self
            .inner
            .lock()
            .map_err(|_| ConversationError::Unavailable)?;
        if inner.poisoned {
            return Err(ConversationError::Unavailable);
        }
        if let Some(existing) = inner
            .state
            .conversations
            .iter()
            .find(|c| c.owned_by(tenant, user) && c.create_request == request_id)
        {
            if existing.deleted {
                return Err(ConversationError::NotFound);
            }
            return Ok(existing.view());
        }
        if inner
            .state
            .conversations
            .iter()
            .filter(|c| c.owned_by(tenant, user) && !c.deleted)
            .count()
            >= MAX_CONVERSATIONS
            || inner.state.conversations.len() >= 1000
        {
            return Err(ConversationError::Capacity);
        }
        let conversation = StoredConversation {
            tenant: tenant.as_str().to_owned(),
            user: user.as_str().to_owned(),
            create_request: request_id.to_owned(),
            id: persistence::random_id()?,
            title: "新对话".to_owned(),
            revision: 0,
            turns: Vec::new(),
            management: Vec::new(),
            deleted: false,
            explicit_title: false,
        };
        let view = conversation.view();
        let mut next = inner.state.clone();
        next.conversations.push(conversation);
        inner.commit(next)?;
        Ok(view)
    }
    #[cfg(test)]
    pub(crate) fn begin(
        &self,
        tenant: &TenantId,
        user: &UserId,
        id: &str,
        intent: ConversationTurnIntentDto,
        confirmed: bool,
    ) -> Result<BeginTurn, ConversationError> {
        self.begin_admitted(tenant, user, id, intent, confirmed, |_| Ok(()))
    }
    pub(crate) fn begin_admitted(
        &self,
        tenant: &TenantId,
        user: &UserId,
        id: &str,
        intent: ConversationTurnIntentDto,
        confirmed: bool,
        admit_new: impl FnOnce(&ChatRequestDto) -> Result<(), ConversationError>,
    ) -> Result<BeginTurn, ConversationError> {
        let explicit_model = match intent.schema.as_str() {
            "chat-conversation-turn/v1" => false,
            "chat-conversation-turn/v2" => true,
            _ => return Err(ConversationError::InvalidIntent),
        };
        if intent.model_id.selected(explicit_model).is_none()
            || !valid_request_id(&intent.request_id)
        {
            return Err(ConversationError::InvalidIntent);
        }
        let mut request = ChatRequestDto {
            schema: if explicit_model {
                CHAT_REQUEST_SCHEMA_V3
            } else {
                CHAT_REQUEST_SCHEMA_V2
            }
            .to_owned(),
            model_id: intent.model_id.clone(),
            messages: vec![ChatInputMessageDto {
                role: ChatInputRole::User,
                content: intent.message.clone(),
            }],
            opportunity_context: intent.opportunity_context.clone(),
            prompt_customization: intent.prompt_customization.clone(),
        };
        validate_chat_request(request.clone(), confirmed)
            .map_err(ConversationError::InvalidChat)?;
        let digest = format!(
            "{:x}",
            Sha256::digest(
                serde_json::to_vec(&(&intent, confirmed))
                    .map_err(|_| ConversationError::InvalidIntent)?
            )
        );
        let profile = intent
            .opportunity_context
            .as_ref()
            .map(|c| c.profile_snapshot_id.clone());
        let mut inner = self
            .inner
            .lock()
            .map_err(|_| ConversationError::Unavailable)?;
        if inner.poisoned {
            return Err(ConversationError::Unavailable);
        }
        let index = inner
            .state
            .conversations
            .iter()
            .position(|c| c.id == id && c.owned_by(tenant, user))
            .ok_or(ConversationError::NotFound)?;
        let current = &inner.state.conversations[index];
        if current.deleted {
            return Err(ConversationError::NotFound);
        }
        if current
            .management
            .iter()
            .any(|receipt| receipt.request_id() == intent.request_id)
        {
            return Err(ConversationError::RequestConflict);
        }
        if let Some(turn) = current
            .turns
            .iter()
            .find(|t| t.view.request_id == intent.request_id)
        {
            if turn.digest != digest {
                return Err(ConversationError::RequestConflict);
            }
            if turn.view.phase == TurnPhase::Running {
                return Err(ConversationError::InProgress);
            }
            return Ok(BeginTurn::Replay(current.result(turn)));
        }
        if current
            .turns
            .iter()
            .any(|t| t.view.phase == TurnPhase::Running)
        {
            return Err(ConversationError::InProgress);
        }
        if current.revision != intent.expected_revision {
            return Err(ConversationError::RevisionConflict);
        }
        if current.turns.len() >= MAX_TURNS
            || inner
                .state
                .conversations
                .iter()
                .flat_map(|c| &c.turns)
                .filter(|t| t.view.phase == TurnPhase::Running)
                .count()
                >= MAX_RUNNING
        {
            return Err(ConversationError::Capacity);
        }
        request.messages = history(current, &intent.message, profile.as_deref());
        validate_chat_request(request.clone(), confirmed)
            .map_err(ConversationError::InvalidChat)?;
        // Exact terminal replay has already returned; model availability gates only new effects.
        admit_new(&request)?;
        let run_id = format!("chat-run:{}:{}", current.id, &digest[..24]);
        let mut next = inner.state.clone();
        let conversation = &mut next.conversations[index];
        conversation.revision = conversation
            .revision
            .checked_add(1)
            .ok_or(ConversationError::Capacity)?;
        let title_request = (conversation.turns.is_empty() && !conversation.explicit_title)
            .then(|| intent.message.clone());
        let automatic_title = title_request
            .as_deref()
            .map(automatic_title::AutomaticTitle::first_message);
        if let Some(title) = &automatic_title {
            conversation.title = title.title().to_owned();
        }
        conversation.turns.push(StoredTurn {
            view: ConversationTurnDto {
                request_id: intent.request_id,
                user: intent.message,
                phase: TurnPhase::Running,
                response: None,
                error: None,
            },
            digest,
            profile,
            result_revision: None,
            automatic_title,
        });
        inner.commit(next)?;
        Ok(BeginTurn::New {
            run_id,
            request,
            title_request,
        })
    }
    #[cfg(test)]
    pub(crate) fn finish(
        &self,
        tenant: &TenantId,
        user: &UserId,
        id: &str,
        request_id: &str,
        result: Result<ChatResponseDto, ChatError>,
    ) -> Result<ConversationTurnResultDto, ConversationError> {
        self.finish_with_title(tenant, user, id, request_id, result, None)
    }
    pub(crate) fn finish_with_title(
        &self,
        tenant: &TenantId,
        user: &UserId,
        id: &str,
        request_id: &str,
        result: Result<ChatResponseDto, ChatError>,
        generated_title: Option<String>,
    ) -> Result<ConversationTurnResultDto, ConversationError> {
        let mut inner = self
            .inner
            .lock()
            .map_err(|_| ConversationError::Unavailable)?;
        if inner.poisoned {
            return Err(ConversationError::Unavailable);
        }
        let index = inner
            .state
            .conversations
            .iter()
            .position(|c| c.id == id && c.owned_by(tenant, user))
            .ok_or(ConversationError::NotFound)?;
        let turn_index = inner.state.conversations[index]
            .turns
            .iter()
            .position(|t| t.view.request_id == request_id)
            .ok_or(ConversationError::NotFound)?;
        let current = &inner.state.conversations[index];
        if current.deleted {
            return Err(ConversationError::NotFound);
        }
        if current.turns[turn_index].view.phase != TurnPhase::Running {
            return Ok(current.result(&current.turns[turn_index]));
        }
        let mut next = inner.state.clone();
        let conversation = &mut next.conversations[index];
        conversation.revision = conversation
            .revision
            .checked_add(1)
            .ok_or(ConversationError::Capacity)?;
        let turn = &mut conversation.turns[turn_index];
        match result {
            Ok(response) => {
                if response.answer.len() > 16 * 1024 {
                    inner.poisoned = true;
                    return Err(ConversationError::Unavailable);
                }
                turn.view.phase = TurnPhase::Completed;
                let value =
                    serde_json::to_value(response).map_err(|_| ConversationError::Unavailable)?;
                if !valid_response(&value) {
                    inner.poisoned = true;
                    return Err(ConversationError::Unavailable);
                }
                turn.view.response = Some(value);
            }
            Err(error) => {
                turn.view.phase = TurnPhase::Failed;
                turn.view.error = Some(error.code().to_owned());
            }
        }
        turn.result_revision = Some(conversation.revision);
        if turn_index == 0
            && !conversation.explicit_title
            && turn.view.phase == TurnPhase::Completed
            && let (Some(title), Some(topic)) = (&mut turn.automatic_title, generated_title)
        {
            title.accept_topic(topic);
            conversation.title = title.title().to_owned();
        }
        let view = conversation.result(&conversation.turns[turn_index]);
        if let Err(error) = inner.commit(next) {
            inner.poisoned = true;
            return Err(error);
        }
        Ok(view)
    }
}
impl Inner {
    fn commit(&mut self, next: State) -> Result<(), ConversationError> {
        match self.disk.save(&next) {
            Ok(()) => {
                self.state = next;
                Ok(())
            }
            Err(error) => {
                if error != ConversationError::Capacity {
                    self.poisoned = true;
                }
                Err(error)
            }
        }
    }
}
fn valid_request_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'-' | b'_'))
}
fn history(
    conversation: &StoredConversation,
    message: &str,
    profile: Option<&str>,
) -> Vec<ChatInputMessageDto> {
    let mut reversed = vec![ChatInputMessageDto {
        role: ChatInputRole::User,
        content: message.to_owned(),
    }];
    let mut bytes = message.len();
    for turn in conversation.turns.iter().rev() {
        if turn.view.phase != TurnPhase::Completed
            || (turn.profile.is_some() && turn.profile.as_deref() != profile)
        {
            break;
        }
        let Some(answer) = turn
            .view
            .response
            .as_ref()
            .and_then(|v| v.get("answer"))
            .and_then(serde_json::Value::as_str)
        else {
            break;
        };
        if answer.trim().is_empty()
            || answer.len() > 4096
            || reversed.len() + 2 > 12
            || bytes + answer.len() + turn.view.user.len() > 12 * 1024
        {
            break;
        }
        bytes += answer.len() + turn.view.user.len();
        reversed.push(ChatInputMessageDto {
            role: ChatInputRole::Assistant,
            content: answer.to_owned(),
        });
        reversed.push(ChatInputMessageDto {
            role: ChatInputRole::User,
            content: turn.view.user.clone(),
        });
    }
    reversed.reverse();
    reversed
}
fn validate_state(state: &State) -> Result<(), ConversationError> {
    use std::collections::{BTreeMap, BTreeSet};
    if state.version != 1
        || state.conversations.len() > 1000
        || state
            .conversations
            .iter()
            .flat_map(|c| &c.turns)
            .filter(|t| t.view.phase == TurnPhase::Running)
            .count()
            > MAX_RUNNING
    {
        return Err(ConversationError::Unavailable);
    }
    let mut ids = BTreeSet::new();
    let mut creates = BTreeSet::new();
    let mut owners = BTreeMap::new();
    for c in &state.conversations {
        if TenantId::parse(&c.tenant).is_err()
            || UserId::parse(&c.user).is_err()
            || c.id.len() != 64
            || !c.id.bytes().all(|b| b.is_ascii_hexdigit())
            || !ids.insert(&c.id)
            || !valid_request_id(&c.create_request)
            || !creates.insert((&c.tenant, &c.user, &c.create_request))
            || c.title.len() > 192
            || c.turns.len() > MAX_TURNS
        {
            return Err(ConversationError::Unavailable);
        }
        let count = owners.entry((&c.tenant, &c.user)).or_insert(0usize);
        if !c.deleted {
            *count += 1;
        }
        if *count > MAX_CONVERSATIONS {
            return Err(ConversationError::Unavailable);
        }
        let mut requests = BTreeSet::new();
        let mut revision = 0;
        let mut management = management::Replay::new(c)?;
        for (index, t) in c.turns.iter().enumerate() {
            if !valid_request_id(&t.view.request_id)
                || !requests.insert(&t.view.request_id)
                || t.view.user.trim().is_empty()
                || t.view.user.len() > 4096
                || t.view.user.contains('\0')
                || t.digest.len() != 64
                || !t.digest.bytes().all(|b| b.is_ascii_hexdigit())
                || t.profile
                    .as_ref()
                    .is_some_and(|p| p.trim().is_empty() || p.len() > 4096 || p.contains('\0'))
            {
                return Err(ConversationError::Unavailable);
            }
            management.apply_pending(c, &mut revision)?;
            management.begin_turn(index, t)?;
            revision += 1;
            match t.view.phase {
                TurnPhase::Running => {
                    if index + 1 != c.turns.len()
                        || t.view.response.is_some()
                        || t.view.error.is_some()
                        || t.result_revision.is_some()
                    {
                        return Err(ConversationError::Unavailable);
                    }
                }
                TurnPhase::Completed => {
                    revision += 1;
                    if t.view.error.is_some()
                        || !t.view.response.as_ref().is_some_and(valid_response)
                        || t.result_revision != Some(revision)
                    {
                        return Err(ConversationError::Unavailable);
                    }
                }
                TurnPhase::Failed | TurnPhase::Interrupted => {
                    revision += 1;
                    if t.view.response.is_some()
                        || !t
                            .view
                            .error
                            .as_ref()
                            .is_some_and(|e| valid_error(e, t.view.phase))
                        || t.result_revision != Some(revision)
                    {
                        return Err(ConversationError::Unavailable);
                    }
                }
            }
        }
        management.finish(c, &mut revision)?;
        if c.revision != revision {
            return Err(ConversationError::Unavailable);
        }
    }
    Ok(())
}
fn closed_object(v: &serde_json::Value, keys: &[&str]) -> bool {
    v.as_object().is_some_and(|object| {
        object.len() == keys.len() && keys.iter().all(|key| object.contains_key(*key))
    })
}
fn bounded_string(v: Option<&serde_json::Value>, max: usize) -> bool {
    v.and_then(serde_json::Value::as_str)
        .is_some_and(|s| !s.trim().is_empty() && s.len() <= max && !s.contains('\0'))
}
fn valid_response(v: &serde_json::Value) -> bool {
    closed_object(
        v,
        &[
            "schema",
            "run_id",
            "answer",
            "provider",
            "tool_trace",
            "usage",
        ],
    ) && v.get("schema").and_then(serde_json::Value::as_str) == Some("ustc-agent-chat-response/v1")
        && v.get("run_id")
            .and_then(serde_json::Value::as_str)
            .is_some_and(|id| {
                id.starts_with("chat-run:")
                    && id.len() > 9
                    && id.len() <= 128
                    && id
                        .bytes()
                        .all(|b| b.is_ascii_alphanumeric() || matches!(b, b':' | b'-' | b'_'))
            })
        && bounded_string(v.get("answer"), 16 * 1024)
        && v.get("provider").is_some_and(|p| {
            closed_object(p, &["mode", "model"])
                && matches!(
                    p.get("mode").and_then(serde_json::Value::as_str),
                    Some("mock" | "openai-compatible" | "local-chat")
                )
                && bounded_string(p.get("model"), 256)
        })
        && v.get("usage").is_some_and(|u| {
            closed_object(u, &["input_tokens", "output_tokens"])
                && u.get("input_tokens")
                    .and_then(serde_json::Value::as_u64)
                    .is_some()
                && u.get("output_tokens")
                    .and_then(serde_json::Value::as_u64)
                    .is_some()
        })
        && v.get("tool_trace")
            .and_then(serde_json::Value::as_array)
            .is_some_and(|a| {
                a.len() <= 4
                    && a.iter().all(|t| {
                        closed_object(t, &["call_id", "tool", "status"])
                            && bounded_string(t.get("call_id"), 256)
                            && matches!(
                                t.get("tool").and_then(serde_json::Value::as_str),
                                Some(
                                    "affairs_navigator_get"
                                        | "change_radar_get"
                                        | "opportunity_graph_plan_current_profile"
                                        | "simple_calendar_items"
                                        | "plugin_tool"
                                )
                            )
                            && matches!(
                                t.get("status").and_then(serde_json::Value::as_str),
                                Some("succeeded" | "denied" | "failed")
                            )
                    })
            })
}
fn valid_error(error: &str, phase: TurnPhase) -> bool {
    if phase == TurnPhase::Interrupted {
        return error == "conversation_interrupted";
    }
    [
        ChatError::InvalidChatRequest,
        ChatError::ProviderNotConfigured,
        ChatError::ProviderUnauthorized,
        ChatError::ProviderRateLimited,
        ChatError::ProviderTimeout,
        ChatError::ProviderUnavailable,
        ChatError::ProviderProtocolError,
        ChatError::ContextBudgetExceeded,
        ChatError::ToolCallRejected,
        ChatError::ToolResultTooLarge,
        ChatError::ToolBudgetExhausted,
        ChatError::TurnBudgetExhausted,
        ChatError::OpportunityConfirmationRequired,
        ChatError::CompositionUnavailable,
        ChatError::Internal,
    ]
    .iter()
    .any(|e| e.code() == error)
}

#[cfg(test)]
mod tests;
