//! Bounded durable observation checkpoints; never a tool replay journal.
use super::*;
use crate::chat_activity::ChatProgress;

impl ConversationStore {
    pub(crate) fn progress(
        &self,
        tenant: &TenantId,
        user: &UserId,
        id: &str,
        request_id: &str,
    ) -> Result<Option<ChatProgress>, ConversationError> {
        let inner = self
            .inner
            .lock()
            .map_err(|_| ConversationError::Unavailable)?;
        if inner.poisoned {
            return Err(ConversationError::Unavailable);
        }
        let conversation = inner
            .state
            .conversations
            .iter()
            .find(|c| c.id == id && c.owned_by(tenant, user) && !c.deleted)
            .ok_or(ConversationError::NotFound)?;
        Ok(conversation
            .turns
            .iter()
            .find(|t| t.view.request_id == request_id)
            .and_then(|t| t.progress.clone()))
    }
    pub(crate) fn checkpoint(
        &self,
        tenant: &TenantId,
        user: &UserId,
        id: &str,
        request_id: &str,
        progress: &ChatProgress,
    ) -> Result<(), ConversationError> {
        if !progress.valid() {
            return Err(ConversationError::Unavailable);
        }
        let mut inner = self
            .inner
            .lock()
            .map_err(|_| ConversationError::Unavailable)?;
        if inner.poisoned {
            return Err(ConversationError::Unavailable);
        }
        let mut next = inner.state.clone();
        let conversation = next
            .conversations
            .iter_mut()
            .find(|c| c.id == id && c.owned_by(tenant, user) && !c.deleted)
            .ok_or(ConversationError::NotFound)?;
        let turn = conversation
            .turns
            .last_mut()
            .filter(|t| t.view.request_id == request_id && t.view.phase == TurnPhase::Running)
            .ok_or(ConversationError::RequestConflict)?;
        turn.progress = Some(progress.clone());
        next.version = 3;
        inner.commit(next)
    }
}
