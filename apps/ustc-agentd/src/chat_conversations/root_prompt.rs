//! Owner-scoped personal instructions; the conversation store is the only durable owner.
use super::{ConversationError, ConversationStore, State};
use crate::agent_chat::{MAX_ROOT_PROMPT_BYTES, normalize_personal_prompt};
use serde::{Deserialize, Serialize};
use ustc_campus_agent_core::identity::{TenantId, UserId};

const MAX_OWNERS: usize = 1000;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct RootPromptDto {
    pub(crate) schema: &'static str,
    pub(crate) revision: u64,
    pub(crate) text: String,
}
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RootPromptUpdateDto {
    pub(crate) schema: String,
    pub(crate) expected_revision: u64,
    pub(crate) text: String,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct StoredRootPrompt {
    tenant: String,
    user: String,
    revision: u64,
    pub(super) text: String,
}
impl StoredRootPrompt {
    fn view(&self) -> RootPromptDto {
        RootPromptDto {
            schema: "agent-root-prompt/v1",
            revision: self.revision,
            text: self.text.clone(),
        }
    }
}
pub(super) fn find<'a>(
    state: &'a State,
    tenant: &TenantId,
    user: &UserId,
) -> Option<&'a StoredRootPrompt> {
    state
        .root_prompts
        .as_deref()
        .unwrap_or_default()
        .iter()
        .find(|p| p.tenant == tenant.as_str() && p.user == user.as_str())
}
pub(super) fn validate(state: &State) -> Result<(), ConversationError> {
    if (state.version == 1 && state.root_prompts.is_some())
        || (state.version == 2 && state.root_prompts.is_none())
    {
        return Err(ConversationError::Unavailable);
    }
    let prompts = state.root_prompts.as_deref().unwrap_or_default();
    if prompts.len() > MAX_OWNERS {
        return Err(ConversationError::Unavailable);
    }
    let mut owners = std::collections::BTreeSet::new();
    for prompt in prompts {
        if TenantId::parse(&prompt.tenant).is_err()
            || UserId::parse(&prompt.user).is_err()
            || !owners.insert((&prompt.tenant, &prompt.user))
            || prompt.revision == 0
            || !normalize_personal_prompt(&prompt.text, MAX_ROOT_PROMPT_BYTES)
                .is_ok_and(|text| text == prompt.text)
        {
            return Err(ConversationError::Unavailable);
        }
    }
    Ok(())
}
impl ConversationStore {
    pub(crate) fn root_prompt(
        &self,
        tenant: &TenantId,
        user: &UserId,
    ) -> Result<RootPromptDto, ConversationError> {
        let inner = self
            .inner
            .lock()
            .map_err(|_| ConversationError::Unavailable)?;
        if inner.poisoned {
            return Err(ConversationError::Unavailable);
        }
        Ok(find(&inner.state, tenant, user).map_or_else(
            || RootPromptDto {
                schema: "agent-root-prompt/v1",
                revision: 0,
                text: String::new(),
            },
            StoredRootPrompt::view,
        ))
    }
    pub(crate) fn update_root_prompt(
        &self,
        tenant: &TenantId,
        user: &UserId,
        intent: RootPromptUpdateDto,
    ) -> Result<RootPromptDto, ConversationError> {
        if intent.schema != "agent-root-prompt-update/v1" {
            return Err(ConversationError::InvalidIntent);
        }
        let text = normalize_personal_prompt(&intent.text, MAX_ROOT_PROMPT_BYTES)
            .map_err(|_| ConversationError::InvalidIntent)?;
        let mut inner = self
            .inner
            .lock()
            .map_err(|_| ConversationError::Unavailable)?;
        if inner.poisoned {
            return Err(ConversationError::Unavailable);
        }
        let current = find(&inner.state, tenant, user);
        let revision = current.map_or(0, |p| p.revision);
        if revision != intent.expected_revision {
            if let Some(current) = current
                && intent.expected_revision.checked_add(1) == Some(revision)
                && current.text == text
            {
                return Ok(current.view());
            }
            return Err(ConversationError::RevisionConflict);
        }
        let revision = revision.checked_add(1).ok_or(ConversationError::Capacity)?;
        let mut next = inner.state.clone();
        let prompts = next.root_prompts.get_or_insert_with(Vec::new);
        let updated = StoredRootPrompt {
            tenant: tenant.as_str().to_owned(),
            user: user.as_str().to_owned(),
            revision,
            text,
        };
        let result = updated.view();
        if let Some(prompt) = prompts
            .iter_mut()
            .find(|p| p.tenant == tenant.as_str() && p.user == user.as_str())
        {
            *prompt = updated;
        } else {
            if prompts.len() >= MAX_OWNERS {
                return Err(ConversationError::Capacity);
            }
            prompts.push(updated);
        }
        next.version = next.version.max(2);
        inner.commit(next)?;
        Ok(result)
    }
}
