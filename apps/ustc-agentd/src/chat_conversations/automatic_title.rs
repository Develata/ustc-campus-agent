//! Title evidence belongs to the first turn's existing atomic reservation and terminal result.
use super::{ConversationError, TurnPhase};
use crate::conversation_title::{
    current_date, dated_title, normalize_topic, valid_date, valid_title,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct AutomaticTitle {
    date: String,
    fallback: String,
    generated: Option<String>,
}
impl AutomaticTitle {
    pub(super) fn first_message(message: &str, retained_date: Option<String>) -> Self {
        let date = retained_date.unwrap_or_else(current_date);
        let fallback = dated_title(&date, message);
        Self {
            date,
            fallback,
            generated: None,
        }
    }
    pub(super) fn date(&self) -> &str {
        &self.date
    }
    pub(super) fn title(&self) -> &str {
        self.generated.as_deref().unwrap_or(&self.fallback)
    }
    pub(super) fn accept_topic(&mut self, topic: String) {
        if let Some(topic) = normalize_topic(&topic) {
            self.generated = Some(dated_title(&self.date, &topic));
        }
    }
    pub(super) fn validate(
        &self,
        message: &str,
        phase: TurnPhase,
    ) -> Result<&str, ConversationError> {
        if !valid_date(&self.date)
            || !valid_title(&self.fallback)
            || self.fallback != dated_title(&self.date, message)
        {
            return Err(ConversationError::Unavailable);
        }
        if let Some(generated) = &self.generated
            && (phase != TurnPhase::Completed
                || !valid_title(generated)
                || generated.get(..6) != Some(self.date.as_str()))
        {
            return Err(ConversationError::Unavailable);
        }
        Ok(self.title())
    }
}
