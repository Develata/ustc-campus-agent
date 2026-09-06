//! Date and organization policy; metadata stays in the transcript persistence owner.
use super::*;
use crate::conversation_title::{current_date, valid_date};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ConversationOrganizationDto {
    #[serde(deserialize_with = "required_nullable")]
    pub(crate) date: Option<String>,
    pub(crate) pinned: bool,
    #[serde(deserialize_with = "required_nullable")]
    pub(crate) group: Option<String>,
}

pub(super) fn checked_label(raw: &str, max: usize) -> Result<&str, ConversationError> {
    let label = raw.trim();
    if label.is_empty() || raw.len() > max || raw.chars().any(forbidden) {
        return Err(ConversationError::InvalidIntent);
    }
    Ok(label)
}
fn forbidden(c: char) -> bool {
    c.is_control()
        || matches!(c,
        '\u{00ad}' | '\u{0600}'..='\u{0605}' | '\u{061c}' | '\u{06dd}' | '\u{070f}' |
        '\u{0890}'..='\u{0891}' | '\u{08e2}' | '\u{180e}' | '\u{200b}'..='\u{200f}' |
        '\u{2028}'..='\u{202e}' | '\u{2060}'..='\u{2064}' | '\u{2066}'..='\u{206f}' |
        '\u{feff}' | '\u{fff9}'..='\u{fffb}' | '\u{110bd}' | '\u{110cd}' |
        '\u{13430}'..='\u{1343f}' | '\u{1bca0}'..='\u{1bca3}' |
        '\u{1d173}'..='\u{1d17a}' | '\u{e0001}' | '\u{e0020}'..='\u{e007f}')
}
pub(super) fn title_date(title: &str) -> Option<String> {
    let (date, topic) = title.split_once('|')?;
    (valid_date(date) && checked_label(topic, 192).is_ok() && !topic.contains('|'))
        .then(|| date.to_owned())
}
impl ConversationOrganizationDto {
    pub(super) fn validate(&self) -> Result<(), ConversationError> {
        if self.date.as_deref().is_some_and(|d| !valid_date(d))
            || self
                .group
                .as_deref()
                .is_some_and(|g| checked_label(g, 64) != Ok(g))
        {
            return Err(ConversationError::Unavailable);
        }
        Ok(())
    }
}
impl StoredConversation {
    pub(super) fn initial_date(&self) -> Option<String> {
        self.created_date.clone().or_else(|| {
            self.turns
                .first()
                .and_then(|t| t.automatic_title.as_ref())
                .map(|t| t.date().to_owned())
        })
    }
    pub(super) fn organization_view(&self) -> ConversationOrganizationDto {
        self.organization
            .clone()
            .unwrap_or_else(|| ConversationOrganizationDto {
                date: self.initial_date().or_else(|| title_date(&self.title)),
                ..Default::default()
            })
    }
    pub(super) fn ensure_organization(&mut self) -> &mut ConversationOrganizationDto {
        let mut organization = self.organization_view();
        organization.date.get_or_insert_with(current_date);
        self.organization.get_or_insert(organization)
    }
}

pub(super) fn required_nullable<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<String>, D::Error> {
    Option::<String>::deserialize(deserializer)
}
