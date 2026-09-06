//! Conversation metadata commands share the transcript's atomic persistence owner.
use super::*;

const MAX_RECEIPTS: usize = 128;
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ConversationManageIntentDto {
    pub(crate) schema: String,
    pub(crate) request_id: String,
    pub(crate) expected_revision: u64,
    pub(crate) action: ConversationManageActionDto,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub(crate) enum ConversationManageActionDto {
    Rename { title: String },
    Delete {},
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ConversationManageResultDto {
    pub(crate) schema: String,
    pub(crate) conversation_id: String,
    pub(crate) request_id: String,
    pub(crate) revision: u64,
    pub(crate) title: String,
    pub(crate) deleted: bool,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct StoredManagement {
    intent: ConversationManageIntentDto,
    result: ConversationManageResultDto,
}
impl StoredManagement {
    pub(super) fn request_id(&self) -> &str {
        &self.intent.request_id
    }
}
fn checked_title(raw: &str) -> Result<&str, ConversationError> {
    let title = raw.trim();
    if title.is_empty() || raw.len() > 192 || raw.chars().any(char::is_control) {
        return Err(ConversationError::InvalidIntent);
    }
    Ok(title)
}
impl ConversationManageIntentDto {
    fn validate(&self) -> Result<(), ConversationError> {
        if self.schema != "chat-conversation-manage/v1" || !valid_request_id(&self.request_id) {
            return Err(ConversationError::InvalidIntent);
        }
        if let ConversationManageActionDto::Rename { title } = &self.action {
            checked_title(title)?;
        }
        Ok(())
    }
}
impl ConversationStore {
    pub(crate) fn manage(
        &self,
        tenant: &TenantId,
        user: &UserId,
        id: &str,
        intent: ConversationManageIntentDto,
    ) -> Result<ConversationManageResultDto, ConversationError> {
        intent.validate()?;
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
        if let Some(receipt) = current
            .management
            .iter()
            .find(|receipt| receipt.intent.request_id == intent.request_id)
        {
            return if receipt.intent == intent {
                Ok(receipt.result.clone())
            } else {
                Err(ConversationError::RequestConflict)
            };
        }
        if current.deleted {
            return Err(ConversationError::NotFound);
        }
        if current
            .turns
            .iter()
            .any(|turn| turn.view.request_id == intent.request_id)
        {
            return Err(ConversationError::RequestConflict);
        }
        if current
            .turns
            .iter()
            .any(|turn| turn.view.phase == TurnPhase::Running)
        {
            return Err(ConversationError::InProgress);
        }
        if current.revision != intent.expected_revision {
            return Err(ConversationError::RevisionConflict);
        }
        if current.management.len() >= MAX_RECEIPTS
            || (matches!(intent.action, ConversationManageActionDto::Rename { .. })
                && current.management.len() >= MAX_RECEIPTS - 1)
        {
            return Err(ConversationError::Capacity);
        }
        let mut next = inner.state.clone();
        let conversation = &mut next.conversations[index];
        conversation.revision = conversation
            .revision
            .checked_add(1)
            .ok_or(ConversationError::Capacity)?;
        match &intent.action {
            ConversationManageActionDto::Rename { title } => {
                conversation.title = checked_title(title)?.to_owned();
                conversation.explicit_title = true;
            }
            ConversationManageActionDto::Delete {} => conversation.deleted = true,
        }
        let result = ConversationManageResultDto {
            schema: "chat-conversation-manage-result/v1".into(),
            conversation_id: conversation.id.clone(),
            request_id: intent.request_id.clone(),
            revision: conversation.revision,
            title: conversation.title.clone(),
            deleted: conversation.deleted,
        };
        conversation.management.push(StoredManagement {
            intent,
            result: result.clone(),
        });
        inner.commit(next)?;
        Ok(result)
    }
}

/// Replay management events between unchanged two-revision turn transitions.
/// No management event can split a running turn or modify a historical turn receipt.
pub(super) struct Replay {
    next: usize,
    title: String,
    explicit_title: bool,
    deleted: bool,
    running: bool,
    automatic_title: bool,
}
impl Replay {
    pub(super) fn new(c: &StoredConversation) -> Result<Self, ConversationError> {
        if c.management.len() > MAX_RECEIPTS
            || c.management
                .iter()
                .filter(|receipt| {
                    matches!(
                        receipt.intent.action,
                        ConversationManageActionDto::Rename { .. }
                    )
                })
                .count()
                >= MAX_RECEIPTS
        {
            return Err(ConversationError::Unavailable);
        }
        let mut requests = std::collections::BTreeSet::new();
        for receipt in &c.management {
            receipt
                .intent
                .validate()
                .map_err(|_| ConversationError::Unavailable)?;
            if !requests.insert(receipt.request_id())
                || c.turns
                    .iter()
                    .any(|turn| turn.view.request_id == receipt.request_id())
            {
                return Err(ConversationError::Unavailable);
            }
        }
        Ok(Self {
            next: 0,
            title: "新对话".into(),
            explicit_title: false,
            deleted: false,
            running: false,
            automatic_title: false,
        })
    }
    pub(super) fn apply_pending(
        &mut self,
        c: &StoredConversation,
        revision: &mut u64,
    ) -> Result<(), ConversationError> {
        while let Some(receipt) = c.management.get(self.next) {
            if receipt.intent.expected_revision > *revision {
                break;
            }
            if self.running || self.deleted || receipt.intent.expected_revision != *revision {
                return Err(ConversationError::Unavailable);
            }
            *revision = revision
                .checked_add(1)
                .ok_or(ConversationError::Unavailable)?;
            match &receipt.intent.action {
                ConversationManageActionDto::Rename { title } => {
                    self.title = checked_title(title)
                        .map_err(|_| ConversationError::Unavailable)?
                        .into();
                    self.explicit_title = true;
                }
                ConversationManageActionDto::Delete {} => self.deleted = true,
            }
            let expected = ConversationManageResultDto {
                schema: "chat-conversation-manage-result/v1".into(),
                conversation_id: c.id.clone(),
                request_id: receipt.intent.request_id.clone(),
                revision: *revision,
                title: self.title.clone(),
                deleted: self.deleted,
            };
            if receipt.result != expected {
                return Err(ConversationError::Unavailable);
            }
            self.next += 1;
        }
        Ok(())
    }
    pub(super) fn begin_turn(
        &mut self,
        index: usize,
        turn: &StoredTurn,
    ) -> Result<(), ConversationError> {
        if self.deleted || self.running {
            return Err(ConversationError::Unavailable);
        }
        if let Some(title) = &turn.automatic_title {
            if index != 0 || self.explicit_title {
                return Err(ConversationError::Unavailable);
            }
            self.title = title.validate(&turn.view.user, turn.view.phase)?.to_owned();
            self.automatic_title = true;
        } else if index == 0 && !self.explicit_title {
            self.title = turn.view.user.chars().take(48).collect();
        }
        self.running = turn.view.phase == TurnPhase::Running;
        Ok(())
    }
    pub(super) fn finish(
        mut self,
        c: &StoredConversation,
        revision: &mut u64,
    ) -> Result<(), ConversationError> {
        self.apply_pending(c, revision)?;
        if self.next != c.management.len()
            || c.deleted != self.deleted
            || c.explicit_title != self.explicit_title
            || ((!c.management.is_empty() || self.automatic_title) && c.title != self.title)
        {
            return Err(ConversationError::Unavailable);
        }
        Ok(())
    }
}

#[cfg(all(test, unix))]
#[path = "conversation_management_tests.rs"]
mod conversation_management_tests;
