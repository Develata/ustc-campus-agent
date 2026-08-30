use serde::{Deserialize, Deserializer, Serialize, de};

use crate::{UnixMillis, WireText};

pub const MAX_CHANGE_FEED_ENTRIES: usize = 128;
pub const MAX_CHANGED_FIELDS_PER_ENTRY: usize = 64;
pub const MAX_ATOM_DOCUMENT_BYTES: usize = 1_048_576;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChangeFeedFieldDto {
    field: WireText,
    before: Option<WireText>,
    after: Option<WireText>,
}

impl ChangeFeedFieldDto {
    #[must_use]
    pub const fn new(field: WireText, before: Option<WireText>, after: Option<WireText>) -> Self {
        Self {
            field,
            before,
            after,
        }
    }

    #[must_use]
    pub const fn field(&self) -> &WireText {
        &self.field
    }

    #[must_use]
    pub const fn before(&self) -> Option<&WireText> {
        self.before.as_ref()
    }

    #[must_use]
    pub const fn after(&self) -> Option<&WireText> {
        self.after.as_ref()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChangeSourceHealthDto {
    Current,
    Stale,
    Conflicting,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChangeFeedEntryDto {
    event_id: WireText,
    stable_guid: WireText,
    affected_scope: WireText,
    changed_fields: Vec<ChangeFeedFieldDto>,
    effective_from: Option<UnixMillis>,
    effective_to: Option<UnixMillis>,
    observed_at: UnixMillis,
    published_at: UnixMillis,
    source_health: ChangeSourceHealthDto,
    source_id: WireText,
    source_url: WireText,
    old_revision_id: WireText,
    old_raw_sha256: WireText,
    old_normalized_sha256: WireText,
    old_source_reviewer: WireText,
    old_source_review_evidence: WireText,
    new_revision_id: WireText,
    new_raw_sha256: WireText,
    new_normalized_sha256: WireText,
    new_source_reviewer: WireText,
    new_source_review_evidence: WireText,
    evidence_set_digest: WireText,
}

impl ChangeFeedEntryDto {
    #[allow(clippy::too_many_arguments)]
    pub fn try_new(
        event_id: WireText,
        stable_guid: WireText,
        affected_scope: WireText,
        changed_fields: Vec<ChangeFeedFieldDto>,
        effective_from: Option<UnixMillis>,
        effective_to: Option<UnixMillis>,
        observed_at: UnixMillis,
        published_at: UnixMillis,
        source_health: ChangeSourceHealthDto,
        source_id: WireText,
        source_url: WireText,
        old_revision_id: WireText,
        old_raw_sha256: WireText,
        old_normalized_sha256: WireText,
        old_source_reviewer: WireText,
        old_source_review_evidence: WireText,
        new_revision_id: WireText,
        new_raw_sha256: WireText,
        new_normalized_sha256: WireText,
        new_source_reviewer: WireText,
        new_source_review_evidence: WireText,
        evidence_set_digest: WireText,
    ) -> Result<Self, ChangeFeedWireError> {
        if changed_fields.is_empty() || changed_fields.len() > MAX_CHANGED_FIELDS_PER_ENTRY {
            return Err(ChangeFeedWireError::ChangedFieldCount);
        }
        Ok(Self {
            event_id,
            stable_guid,
            affected_scope,
            changed_fields,
            effective_from,
            effective_to,
            observed_at,
            published_at,
            source_health,
            source_id,
            source_url,
            old_revision_id,
            old_raw_sha256,
            old_normalized_sha256,
            old_source_reviewer,
            old_source_review_evidence,
            new_revision_id,
            new_raw_sha256,
            new_normalized_sha256,
            new_source_reviewer,
            new_source_review_evidence,
            evidence_set_digest,
        })
    }

    #[must_use]
    pub const fn event_id(&self) -> &WireText {
        &self.event_id
    }

    #[must_use]
    pub const fn stable_guid(&self) -> &WireText {
        &self.stable_guid
    }

    #[must_use]
    pub const fn affected_scope(&self) -> &WireText {
        &self.affected_scope
    }

    #[must_use]
    pub fn changed_fields(&self) -> &[ChangeFeedFieldDto] {
        &self.changed_fields
    }

    #[must_use]
    pub const fn source_id(&self) -> &WireText {
        &self.source_id
    }

    #[must_use]
    pub const fn source_url(&self) -> &WireText {
        &self.source_url
    }

    #[must_use]
    pub const fn observed_at(&self) -> UnixMillis {
        self.observed_at
    }

    #[must_use]
    pub const fn published_at(&self) -> UnixMillis {
        self.published_at
    }

    #[must_use]
    pub const fn source_health(&self) -> ChangeSourceHealthDto {
        self.source_health
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ChangeFeedViewDto {
    board_id: WireText,
    board_policy_revision: u64,
    feed_id: WireText,
    title: WireText,
    author_name: WireText,
    public_base_url: WireText,
    atom: String,
    entries: Vec<ChangeFeedEntryDto>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct UncheckedChangeFeedViewDto {
    board_id: WireText,
    board_policy_revision: u64,
    feed_id: WireText,
    title: WireText,
    author_name: WireText,
    public_base_url: WireText,
    atom: String,
    entries: Vec<ChangeFeedEntryDto>,
}

impl ChangeFeedViewDto {
    #[allow(clippy::too_many_arguments)]
    pub fn try_new(
        board_id: WireText,
        board_policy_revision: u64,
        feed_id: WireText,
        title: WireText,
        author_name: WireText,
        public_base_url: WireText,
        atom: String,
        entries: Vec<ChangeFeedEntryDto>,
    ) -> Result<Self, ChangeFeedWireError> {
        if board_policy_revision == 0 {
            return Err(ChangeFeedWireError::PolicyRevisionZero);
        }
        if atom.is_empty()
            || atom.len() > MAX_ATOM_DOCUMENT_BYTES
            || atom.chars().any(|value| value == '\0')
        {
            return Err(ChangeFeedWireError::AtomDocument);
        }
        if entries.len() > MAX_CHANGE_FEED_ENTRIES {
            return Err(ChangeFeedWireError::EntryCount);
        }
        if entries.iter().any(|entry| {
            entry.changed_fields.is_empty()
                || entry.changed_fields.len() > MAX_CHANGED_FIELDS_PER_ENTRY
        }) {
            return Err(ChangeFeedWireError::ChangedFieldCount);
        }
        Ok(Self {
            board_id,
            board_policy_revision,
            feed_id,
            title,
            author_name,
            public_base_url,
            atom,
            entries,
        })
    }

    #[must_use]
    pub const fn board_id(&self) -> &WireText {
        &self.board_id
    }

    #[must_use]
    pub const fn board_policy_revision(&self) -> u64 {
        self.board_policy_revision
    }

    #[must_use]
    pub const fn feed_id(&self) -> &WireText {
        &self.feed_id
    }

    #[must_use]
    pub const fn title(&self) -> &WireText {
        &self.title
    }

    #[must_use]
    pub const fn author_name(&self) -> &WireText {
        &self.author_name
    }

    #[must_use]
    pub const fn public_base_url(&self) -> &WireText {
        &self.public_base_url
    }

    #[must_use]
    pub fn atom(&self) -> &str {
        &self.atom
    }

    #[must_use]
    pub fn entries(&self) -> &[ChangeFeedEntryDto] {
        &self.entries
    }
}

impl<'de> Deserialize<'de> for ChangeFeedViewDto {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = UncheckedChangeFeedViewDto::deserialize(deserializer)?;
        Self::try_new(
            value.board_id,
            value.board_policy_revision,
            value.feed_id,
            value.title,
            value.author_name,
            value.public_base_url,
            value.atom,
            value.entries,
        )
        .map_err(de::Error::custom)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum M70ChangeFeedOutcomeDto {
    Found { view: Box<ChangeFeedViewDto> },
    NotFound { board_id: WireText },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct M70ChangeFeedTerminalDto {
    outcome: M70ChangeFeedOutcomeDto,
}

impl M70ChangeFeedTerminalDto {
    #[must_use]
    pub const fn new(outcome: M70ChangeFeedOutcomeDto) -> Self {
        Self { outcome }
    }

    #[must_use]
    pub const fn outcome(&self) -> &M70ChangeFeedOutcomeDto {
        &self.outcome
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChangeFeedWireError {
    PolicyRevisionZero,
    ChangedFieldCount,
    EntryCount,
    AtomDocument,
}

impl std::fmt::Display for ChangeFeedWireError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::PolicyRevisionZero => "change feed policy revision is zero",
            Self::ChangedFieldCount => "change feed changed-field count is invalid",
            Self::EntryCount => "change feed entry count exceeds the bound",
            Self::AtomDocument => "change feed Atom document is invalid or exceeds the bound",
        })
    }
}

impl std::error::Error for ChangeFeedWireError {}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use serde_json::json;

    use super::*;

    fn entry(changed_fields: serde_json::Value) -> serde_json::Value {
        json!({
            "event_id": "change:sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "stable_guid": "urn:ustc-campus-agent:change:fixture",
            "affected_scope": "scope",
            "changed_fields": changed_fields,
            "effective_from": 1,
            "effective_to": 2,
            "observed_at": 3,
            "published_at": 4,
            "source_health": "current",
            "source_id": "src:fixture",
            "source_url": "https://example.test/source",
            "old_revision_id": "revision:old",
            "old_raw_sha256": "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "old_normalized_sha256": "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
            "old_source_reviewer": "reviewer:old",
            "old_source_review_evidence": "evidence:old",
            "new_revision_id": "revision:new",
            "new_raw_sha256": "sha256:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc",
            "new_normalized_sha256": "sha256:dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd",
            "new_source_reviewer": "reviewer:new",
            "new_source_review_evidence": "evidence:new",
            "evidence_set_digest": "sha256:eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee"
        })
    }

    fn view(atom: String, entries: serde_json::Value) -> serde_json::Value {
        json!({
            "board_id": "board:fixture",
            "board_policy_revision": 1,
            "feed_id": "urn:feed:fixture",
            "title": "Fixture feed",
            "author_name": "Fixture author",
            "public_base_url": "https://example.test/feed",
            "atom": atom,
            "entries": entries
        })
    }

    #[test]
    fn valid_view_round_trips_through_checked_deserialization() {
        let changed = json!([{"field":"deadline","before":"old","after":"new"}]);
        let value = view("<feed/>".to_owned(), json!([entry(changed)]));
        let parsed: ChangeFeedViewDto = serde_json::from_value(value).unwrap();
        assert_eq!(parsed.entries().len(), 1);
        assert_eq!(parsed.entries()[0].changed_fields().len(), 1);
    }

    #[test]
    fn unbounded_atom_and_empty_changed_fields_fail_closed() {
        let oversized = view(
            "x".repeat(MAX_ATOM_DOCUMENT_BYTES + 1),
            json!([entry(
                json!([{"field":"deadline","before":"old","after":"new"}])
            )]),
        );
        assert!(serde_json::from_value::<ChangeFeedViewDto>(oversized).is_err());

        let empty_changes = view("<feed/>".to_owned(), json!([entry(json!([]))]));
        assert!(serde_json::from_value::<ChangeFeedViewDto>(empty_changes).is_err());
    }

    #[test]
    fn zero_policy_revision_fails_closed() {
        let mut value = view("<feed/>".to_owned(), json!([]));
        value["board_policy_revision"] = json!(0);
        assert!(serde_json::from_value::<ChangeFeedViewDto>(value).is_err());
    }
}
