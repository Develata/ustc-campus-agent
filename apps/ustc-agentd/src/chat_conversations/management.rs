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
    Rename {
        title: String,
    },
    Delete {},
    Pin {
        pinned: bool,
    },
    Group {
        #[serde(deserialize_with = "organization::required_nullable")]
        group: Option<String>,
    },
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) organization: Option<ConversationOrganizationDto>,
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
    fn v2(&self) -> bool {
        self.schema == "chat-conversation-manage/v2"
    }
    fn validate(&self) -> Result<(), ConversationError> {
        if !matches!(
            self.schema.as_str(),
            "chat-conversation-manage/v1" | "chat-conversation-manage/v2"
        ) || !valid_request_id(&self.request_id)
        {
            return Err(ConversationError::InvalidIntent);
        }
        match &self.action {
            ConversationManageActionDto::Rename { title } if self.v2() => {
                organization::checked_label(title, 192)?;
                if title.contains('|') {
                    return Err(ConversationError::InvalidIntent);
                }
            }
            ConversationManageActionDto::Rename { title } => {
                checked_title(title)?;
            }
            ConversationManageActionDto::Pin { .. } | ConversationManageActionDto::Group { .. }
                if !self.v2() =>
            {
                return Err(ConversationError::InvalidIntent);
            }
            ConversationManageActionDto::Group { group: Some(group) } => {
                organization::checked_label(group, 64)?;
            }
            _ => {}
        }
        Ok(())
    }
}
fn apply_action(
    intent: &ConversationManageIntentDto,
    title: &mut String,
    explicit: &mut bool,
    deleted: &mut bool,
    organization: &mut Option<ConversationOrganizationDto>,
) -> Result<(), ConversationError> {
    match &intent.action {
        ConversationManageActionDto::Rename { title: raw } => {
            *title = if intent.v2() {
                let date = organization
                    .as_ref()
                    .and_then(|o| o.date.as_deref())
                    .ok_or(ConversationError::Unavailable)?;
                format!("{date}|{}", organization::checked_label(raw, 192)?)
            } else {
                checked_title(raw)?.to_owned()
            };
            *explicit = true;
        }
        ConversationManageActionDto::Delete {} => *deleted = true,
        ConversationManageActionDto::Pin { pinned } => {
            organization
                .as_mut()
                .ok_or(ConversationError::Unavailable)?
                .pinned = *pinned;
        }
        ConversationManageActionDto::Group { group } => {
            organization
                .as_mut()
                .ok_or(ConversationError::Unavailable)?
                .group = group
                .as_deref()
                .map(|g| organization::checked_label(g, 64).map(str::to_owned))
                .transpose()?;
        }
    }
    Ok(())
}
fn result_schema(v2: bool) -> String {
    if v2 {
        "chat-conversation-manage-result/v2"
    } else {
        "chat-conversation-manage-result/v1"
    }
    .into()
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
            || (!matches!(intent.action, ConversationManageActionDto::Delete { .. })
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
        if intent.v2() {
            conversation.ensure_organization();
        }
        apply_action(
            &intent,
            &mut conversation.title,
            &mut conversation.explicit_title,
            &mut conversation.deleted,
            &mut conversation.organization,
        )?;
        let result = ConversationManageResultDto {
            schema: result_schema(intent.v2()),
            conversation_id: conversation.id.clone(),
            request_id: intent.request_id.clone(),
            revision: conversation.revision,
            title: conversation.title.clone(),
            deleted: conversation.deleted,
            organization: intent.v2().then(|| conversation.organization_view()),
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
    organization: Option<ConversationOrganizationDto>,
    initial_date: Option<String>,
}
impl Replay {
    pub(super) fn new(c: &StoredConversation) -> Result<Self, ConversationError> {
        if c.management.len() > MAX_RECEIPTS
            || c.management
                .iter()
                .filter(|receipt| {
                    !matches!(
                        receipt.intent.action,
                        ConversationManageActionDto::Delete { .. }
                    )
                })
                .count()
                >= MAX_RECEIPTS
        {
            return Err(ConversationError::Unavailable);
        }
        if let Some(organization) = &c.organization {
            organization.validate()?;
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
        // Pre-organization stores admitted an arbitrary title on an empty legacy row.
        // Its first non-rename v2 receipt preserves that original title as evidence.
        let legacy_title = c
            .created_date
            .is_none()
            .then_some(c)
            .and_then(|c| c.management.first())
            .filter(|r| {
                r.intent.v2()
                    && r.intent.expected_revision == 0
                    && !matches!(r.intent.action, ConversationManageActionDto::Rename { .. })
            })
            .map(|r| checked_title(&r.result.title))
            .transpose()
            .map_err(|_| ConversationError::Unavailable)?;
        Ok(Self {
            next: 0,
            title: legacy_title.unwrap_or("新对话").into(),
            explicit_title: false,
            deleted: false,
            running: false,
            automatic_title: false,
            organization: None,
            initial_date: c.initial_date(),
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
            if receipt.intent.v2() && self.organization.is_none() {
                let evidence = receipt
                    .result
                    .organization
                    .as_ref()
                    .ok_or(ConversationError::Unavailable)?;
                evidence.validate()?;
                let date = self
                    .initial_date
                    .clone()
                    .or_else(|| organization::title_date(&self.title))
                    .or_else(|| evidence.date.clone())
                    .ok_or(ConversationError::Unavailable)?;
                self.organization = Some(ConversationOrganizationDto {
                    date: Some(date),
                    ..Default::default()
                });
            }
            apply_action(
                &receipt.intent,
                &mut self.title,
                &mut self.explicit_title,
                &mut self.deleted,
                &mut self.organization,
            )
            .map_err(|_| ConversationError::Unavailable)?;
            let expected = ConversationManageResultDto {
                schema: result_schema(receipt.intent.v2()),
                conversation_id: c.id.clone(),
                request_id: receipt.intent.request_id.clone(),
                revision: *revision,
                title: self.title.clone(),
                deleted: self.deleted,
                organization: receipt
                    .intent
                    .v2()
                    .then(|| self.organization.clone())
                    .flatten(),
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
            if index != 0
                || self.explicit_title
                || self
                    .initial_date
                    .as_deref()
                    .is_some_and(|date| date != title.date())
                || self
                    .organization
                    .as_ref()
                    .and_then(|o| o.date.as_deref())
                    .is_some_and(|date| date != title.date())
            {
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
            || c.organization != self.organization
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

#[cfg(all(test, unix))]
#[path = "organization_tests.rs"]
mod organization_tests;
