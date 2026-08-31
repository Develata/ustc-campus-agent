//! Deterministic ChangeRadar semantic-diff and reviewed-feed kernel.
//!
//! The crate consumes immutable M60 `SourceRevision` values. It owns one board
//! policy, typed normalized facts, deterministic semantic comparison and an
//! atomic baseline/candidate/review/publication repository boundary. It also
//! renders deterministic Atom from approved events. It performs no retrieval,
//! parsing, M00 authorization, durable storage, model call, or Web I/O.

#![forbid(unsafe_code)]

mod publication;
mod query;

pub use publication::*;
pub use query::*;

use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt;

use sha2::{Digest, Sha256};
use ustc_campus_agent_core::source_registry::SourceId;
use ustc_campus_agent_core::source_revision::{
    EffectiveInterval, RevisionSha256, RevisionTimestamp, SourceRevision, SourceRevisionHealth,
    SourceRevisionId,
};

const MAX_FIELD_BYTES: usize = 64;
const MAX_VALUE_BYTES: usize = 512;
const MAX_FIELDS: usize = 64;
const DEFAULT_MAX_BASELINES: usize = 256;
const DEFAULT_MAX_CANDIDATES: usize = 4_096;

fn valid_token(value: &str, max: usize) -> bool {
    let bytes = value.as_bytes();
    !bytes.is_empty()
        && bytes.len() <= max
        && (bytes[0].is_ascii_lowercase() || bytes[0].is_ascii_digit())
        && (bytes[bytes.len() - 1].is_ascii_lowercase() || bytes[bytes.len() - 1].is_ascii_digit())
        && bytes.iter().all(|byte| {
            byte.is_ascii_lowercase()
                || byte.is_ascii_digit()
                || matches!(byte, b'-' | b'.' | b'_' | b':')
        })
}

/// One board-scoped semantic field name.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SemanticField(String);

impl SemanticField {
    pub fn parse(value: impl Into<String>) -> Result<Self, ChangeRadarError> {
        let value = value.into();
        if !valid_token(&value, MAX_FIELD_BYTES) {
            return Err(ChangeRadarError::InvalidSemanticField);
        }
        Ok(Self(value))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// One bounded semantic value. Accepted bytes are preserved exactly.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SemanticValue(String);

impl SemanticValue {
    pub fn parse(value: impl Into<String>) -> Result<Self, ChangeRadarError> {
        let value = value.into();
        if value.is_empty()
            || value.len() > MAX_VALUE_BYTES
            || value.trim() != value
            || value.chars().any(char::is_control)
        {
            return Err(ChangeRadarError::InvalidSemanticValue);
        }
        Ok(Self(value))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Deterministically ordered normalized facts for one exact revision.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NormalizedFacts(BTreeMap<SemanticField, SemanticValue>);

impl NormalizedFacts {
    pub fn try_from_iter<I>(values: I) -> Result<Self, ChangeRadarError>
    where
        I: IntoIterator<Item = (SemanticField, SemanticValue)>,
    {
        let mut fields = BTreeMap::new();
        for (field, value) in values {
            if fields.insert(field, value).is_some() {
                return Err(ChangeRadarError::DuplicateSemanticField);
            }
            if fields.len() > MAX_FIELDS {
                return Err(ChangeRadarError::TooManySemanticFields);
            }
        }
        if fields.is_empty() {
            return Err(ChangeRadarError::EmptyNormalizedFacts);
        }
        Ok(Self(fields))
    }

    #[must_use]
    pub fn get(&self, field: &SemanticField) -> Option<&SemanticValue> {
        self.0.get(field)
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Iterates the canonical field order without exposing mutable storage.
    pub fn iter(&self) -> impl Iterator<Item = (&SemanticField, &SemanticValue)> {
        self.0.iter()
    }

    /// Computes a stable digest over a length-prefixed canonical encoding.
    #[must_use]
    pub fn sha256(&self) -> RevisionSha256 {
        let mut hasher = Sha256::new();
        hasher.update(b"change-radar-normalized-facts/v0\0");
        for (field, value) in &self.0 {
            update_part(&mut hasher, field.as_str().as_bytes());
            update_part(&mut hasher, value.as_str().as_bytes());
        }
        let digest: [u8; 32] = hasher.finalize().into();
        RevisionSha256::from_bytes(digest)
    }
}

fn update_part(hasher: &mut Sha256, bytes: &[u8]) {
    hasher.update(u64::try_from(bytes.len()).unwrap_or(u64::MAX).to_be_bytes());
    hasher.update(bytes);
}

/// A revision and its exact normalized facts admitted to ChangeRadar.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AcceptedObservation {
    revision: SourceRevision,
    facts: NormalizedFacts,
    health: SourceRevisionHealth,
}

impl AcceptedObservation {
    pub fn new(
        revision: SourceRevision,
        facts: NormalizedFacts,
        health: SourceRevisionHealth,
    ) -> Result<Self, ChangeRadarError> {
        if &facts.sha256() != revision.normalized_sha256() {
            return Err(ChangeRadarError::NormalizedDigestMismatch);
        }
        Ok(Self {
            revision,
            facts,
            health,
        })
    }

    #[must_use]
    pub fn revision(&self) -> &SourceRevision {
        &self.revision
    }
    #[must_use]
    pub fn facts(&self) -> &NormalizedFacts {
        &self.facts
    }
    #[must_use]
    pub const fn health(&self) -> SourceRevisionHealth {
        self.health
    }
}

/// Stable board identity.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BoardId(String);

impl BoardId {
    pub fn parse(value: impl Into<String>) -> Result<Self, ChangeRadarError> {
        let value = value.into();
        if !valid_token(&value, 128) {
            return Err(ChangeRadarError::InvalidBoardPolicy);
        }
        Ok(Self(value))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// One bounded board policy for semantic comparison.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoardPolicy {
    board_id: BoardId,
    source_id: SourceId,
    revision: u64,
    tracked_fields: BTreeSet<SemanticField>,
    affected_scope: String,
}

impl BoardPolicy {
    pub fn new(
        board_id: BoardId,
        source_id: SourceId,
        revision: u64,
        tracked_fields: impl IntoIterator<Item = SemanticField>,
        affected_scope: impl Into<String>,
    ) -> Result<Self, ChangeRadarError> {
        let tracked_fields: BTreeSet<_> = tracked_fields.into_iter().collect();
        let affected_scope = affected_scope.into();
        if revision == 0
            || tracked_fields.is_empty()
            || tracked_fields.len() > MAX_FIELDS
            || affected_scope.is_empty()
            || affected_scope.len() > 128
            || affected_scope.chars().any(char::is_control)
        {
            return Err(ChangeRadarError::InvalidBoardPolicy);
        }
        Ok(Self {
            board_id,
            source_id,
            revision,
            tracked_fields,
            affected_scope,
        })
    }

    #[must_use]
    pub fn board_id(&self) -> &BoardId {
        &self.board_id
    }
    #[must_use]
    pub fn source_id(&self) -> &SourceId {
        &self.source_id
    }
    #[must_use]
    pub const fn revision(&self) -> u64 {
        self.revision
    }
    #[must_use]
    pub fn affected_scope(&self) -> &str {
        &self.affected_scope
    }

    /// Iterates the canonical tracked-field order without exposing mutable policy state.
    pub fn tracked_fields(&self) -> impl Iterator<Item = &SemanticField> {
        self.tracked_fields.iter()
    }
}

/// One field-level semantic before/after value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChangedField {
    field: SemanticField,
    before: Option<SemanticValue>,
    after: Option<SemanticValue>,
}

impl ChangedField {
    #[must_use]
    pub fn field(&self) -> &SemanticField {
        &self.field
    }
    #[must_use]
    pub fn before(&self) -> Option<&SemanticValue> {
        self.before.as_ref()
    }
    #[must_use]
    pub fn after(&self) -> Option<&SemanticValue> {
        self.after.as_ref()
    }
}

/// Stable deterministic identity of one proposed semantic change.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ChangeEventId(String);

impl ChangeEventId {
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Proposed semantic change. It has no administrator publication authority.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemanticChangeCandidate {
    event_id: ChangeEventId,
    board_id: BoardId,
    board_policy_revision: u64,
    source_id: SourceId,
    old_revision: SourceRevision,
    new_revision: SourceRevision,
    health: SourceRevisionHealth,
    changed_fields: Vec<ChangedField>,
    affected_scope: String,
}

impl SemanticChangeCandidate {
    #[must_use]
    pub fn event_id(&self) -> &ChangeEventId {
        &self.event_id
    }
    #[must_use]
    pub fn board_id(&self) -> &BoardId {
        &self.board_id
    }
    #[must_use]
    pub const fn board_policy_revision(&self) -> u64 {
        self.board_policy_revision
    }
    #[must_use]
    pub fn source_id(&self) -> &SourceId {
        &self.source_id
    }
    #[must_use]
    pub fn old_revision(&self) -> &SourceRevision {
        &self.old_revision
    }
    #[must_use]
    pub fn new_revision(&self) -> &SourceRevision {
        &self.new_revision
    }
    #[must_use]
    pub const fn health(&self) -> SourceRevisionHealth {
        self.health
    }
    #[must_use]
    pub fn changed_fields(&self) -> &[ChangedField] {
        &self.changed_fields
    }
    #[must_use]
    pub fn affected_scope(&self) -> &str {
        &self.affected_scope
    }
    #[must_use]
    pub const fn observed_at(&self) -> RevisionTimestamp {
        self.new_revision.observed_at()
    }
    #[must_use]
    pub const fn published_at(&self) -> Option<RevisionTimestamp> {
        self.new_revision.published_at()
    }
    #[must_use]
    pub const fn effective_interval(&self) -> EffectiveInterval {
        self.new_revision.effective_interval()
    }
}

/// Stable product result for one observation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ObservationOutcome {
    BaselineEstablished {
        revision_id: SourceRevisionId,
    },
    SemanticChange(Box<SemanticChangeCandidate>),
    NoSemanticChange {
        old_revision_id: SourceRevisionId,
        new_revision_id: SourceRevisionId,
    },
    DuplicateRevision {
        revision_id: SourceRevisionId,
    },
    OutOfOrderRevision {
        revision_id: SourceRevisionId,
    },
    StaleRevision {
        revision_id: SourceRevisionId,
    },
    ConflictingRevision {
        revision_id: SourceRevisionId,
    },
    SourceUnavailable {
        source_id: SourceId,
    },
}

/// Service-minted atomic repository command.
///
/// Fields are private so callers cannot bypass health and lineage checks while
/// repository adapters retain read access through the accessors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChangeRadarCommit {
    expected_baseline: Option<SourceRevisionId>,
    next: AcceptedObservation,
    candidate: Option<SemanticChangeCandidate>,
}

impl ChangeRadarCommit {
    fn try_new(
        expected_baseline: Option<SourceRevisionId>,
        next: AcceptedObservation,
        candidate: Option<SemanticChangeCandidate>,
    ) -> Result<Self, ChangeRadarRepositoryError> {
        if next.health() != SourceRevisionHealth::Current {
            return Err(ChangeRadarRepositoryError::InvalidCommit);
        }
        if let Some(change) = candidate.as_ref() {
            let Some(expected) = expected_baseline.as_ref() else {
                return Err(ChangeRadarRepositoryError::InvalidCommit);
            };
            if change.old_revision().revision_id() != expected
                || change.new_revision() != next.revision()
                || change.source_id() != next.revision().source_id()
                || change.health() != next.health()
            {
                return Err(ChangeRadarRepositoryError::InvalidCommit);
            }
        }
        Ok(Self {
            expected_baseline,
            next,
            candidate,
        })
    }

    #[must_use]
    pub fn expected_baseline(&self) -> Option<&SourceRevisionId> {
        self.expected_baseline.as_ref()
    }

    #[must_use]
    pub fn next(&self) -> &AcceptedObservation {
        &self.next
    }

    #[must_use]
    pub fn candidate(&self) -> Option<&SemanticChangeCandidate> {
        self.candidate.as_ref()
    }

    #[must_use]
    pub fn into_parts(
        self,
    ) -> (
        Option<SourceRevisionId>,
        AcceptedObservation,
        Option<SemanticChangeCandidate>,
    ) {
        (self.expected_baseline, self.next, self.candidate)
    }
}

/// Atomic baseline and candidate persistence boundary.
pub trait ChangeRadarRepository {
    fn baseline(
        &self,
        source_id: &SourceId,
    ) -> Result<Option<AcceptedObservation>, ChangeRadarRepositoryError>;

    fn apply(&mut self, commit: ChangeRadarCommit) -> Result<(), ChangeRadarRepositoryError>;
}

/// Deterministic bounded in-memory repository with preflight-then-publish updates.
#[derive(Debug, Clone)]
pub struct InMemoryChangeRadarRepository {
    baselines: BTreeMap<SourceId, AcceptedObservation>,
    pub(crate) candidates: BTreeMap<ChangeEventId, SemanticChangeCandidate>,
    pub(crate) reviews: BTreeMap<ChangeEventId, ChangeReviewReceipt>,
    pub(crate) publications: BTreeMap<ChangeEventId, PublishedChangeEvent>,
    pub(crate) feed_guids: BTreeMap<StableFeedGuid, ChangeEventId>,
    max_baselines: usize,
    pub(crate) max_candidates: usize,
    fail_next_apply: bool,
    pub(crate) fail_next_review: bool,
    pub(crate) fail_next_publication: bool,
}

impl Default for InMemoryChangeRadarRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl InMemoryChangeRadarRepository {
    #[must_use]
    pub fn new() -> Self {
        Self {
            baselines: BTreeMap::new(),
            candidates: BTreeMap::new(),
            reviews: BTreeMap::new(),
            publications: BTreeMap::new(),
            feed_guids: BTreeMap::new(),
            max_baselines: DEFAULT_MAX_BASELINES,
            max_candidates: DEFAULT_MAX_CANDIDATES,
            fail_next_apply: false,
            fail_next_review: false,
            fail_next_publication: false,
        }
    }

    /// Constructs explicit non-zero bounds for synthetic or deployment-specific adapters.
    pub fn with_limits(
        max_baselines: usize,
        max_candidates: usize,
    ) -> Result<Self, ChangeRadarRepositoryError> {
        if max_baselines == 0 || max_candidates == 0 {
            return Err(ChangeRadarRepositoryError::InvalidCapacity);
        }
        Ok(Self {
            baselines: BTreeMap::new(),
            candidates: BTreeMap::new(),
            reviews: BTreeMap::new(),
            publications: BTreeMap::new(),
            feed_guids: BTreeMap::new(),
            max_baselines,
            max_candidates,
            fail_next_apply: false,
            fail_next_review: false,
            fail_next_publication: false,
        })
    }

    pub fn inject_next_apply_failure(&mut self) {
        self.fail_next_apply = true;
    }

    #[must_use]
    pub fn candidate_count(&self) -> usize {
        self.candidates.len()
    }

    #[must_use]
    pub fn current_revision_id(&self, source_id: &SourceId) -> Option<&SourceRevisionId> {
        self.baselines
            .get(source_id)
            .map(|observation| observation.revision().revision_id())
    }
}

impl ChangeRadarRepository for InMemoryChangeRadarRepository {
    fn baseline(
        &self,
        source_id: &SourceId,
    ) -> Result<Option<AcceptedObservation>, ChangeRadarRepositoryError> {
        Ok(self.baselines.get(source_id).cloned())
    }

    fn apply(&mut self, commit: ChangeRadarCommit) -> Result<(), ChangeRadarRepositoryError> {
        if self.fail_next_apply {
            self.fail_next_apply = false;
            return Err(ChangeRadarRepositoryError::InjectedPersistenceFailure);
        }
        let (expected_baseline, next, candidate) = commit.into_parts();
        let source_id = next.revision().source_id();
        let actual = self
            .baselines
            .get(source_id)
            .map(|value| value.revision().revision_id());
        if actual != expected_baseline.as_ref() {
            return Err(ChangeRadarRepositoryError::BaselineConflict);
        }
        if actual.is_none() && self.baselines.len() >= self.max_baselines {
            return Err(ChangeRadarRepositoryError::BaselineCapacityExceeded);
        }
        if let Some(candidate) = candidate.as_ref() {
            if let Some(existing) = self.candidates.get(candidate.event_id()) {
                if existing != candidate {
                    return Err(ChangeRadarRepositoryError::CandidateIdentityConflict);
                }
            } else if self.candidates.len() >= self.max_candidates {
                return Err(ChangeRadarRepositoryError::CandidateCapacityExceeded);
            }
        }
        self.baselines.insert(source_id.clone(), next);
        if let Some(candidate) = candidate {
            let event_id = candidate.event_id().clone();
            self.candidates.entry(event_id).or_insert(candidate);
        }
        Ok(())
    }
}

/// Product service that derives a candidate before one atomic repository update.
pub struct ChangeRadarService<R> {
    policy: BoardPolicy,
    repository: R,
}

impl<R> ChangeRadarService<R> {
    #[must_use]
    pub fn new(policy: BoardPolicy, repository: R) -> Self {
        Self { policy, repository }
    }

    #[must_use]
    pub fn repository(&self) -> &R {
        &self.repository
    }

    #[must_use]
    pub fn repository_mut(&mut self) -> &mut R {
        &mut self.repository
    }

    #[must_use]
    pub fn into_repository(self) -> R {
        self.repository
    }
}

impl<R: ChangeRadarRepository> ChangeRadarService<R> {
    pub fn observe_unavailable(
        &self,
        source_id: SourceId,
    ) -> Result<ObservationOutcome, ChangeRadarError> {
        if &source_id != self.policy.source_id() {
            return Err(ChangeRadarError::SourceNotAllowed);
        }
        Ok(ObservationOutcome::SourceUnavailable { source_id })
    }

    pub fn observe(
        &mut self,
        next: AcceptedObservation,
    ) -> Result<ObservationOutcome, ChangeRadarError> {
        if next.revision().source_id() != self.policy.source_id() {
            return Err(ChangeRadarError::SourceNotAllowed);
        }
        let revision_id = next.revision().revision_id().clone();
        match next.health() {
            SourceRevisionHealth::Stale => {
                return Ok(ObservationOutcome::StaleRevision { revision_id });
            }
            SourceRevisionHealth::Conflicting => {
                return Ok(ObservationOutcome::ConflictingRevision { revision_id });
            }
            SourceRevisionHealth::Current => {}
        }

        let source_id = next.revision().source_id().clone();
        let previous = self
            .repository
            .baseline(&source_id)
            .map_err(ChangeRadarError::Repository)?;
        let Some(previous) = previous else {
            let commit = ChangeRadarCommit::try_new(None, next, None)
                .map_err(ChangeRadarError::Repository)?;
            self.repository
                .apply(commit)
                .map_err(ChangeRadarError::Repository)?;
            return Ok(ObservationOutcome::BaselineEstablished { revision_id });
        };

        if previous.revision().revision_id() == next.revision().revision_id() {
            if previous == next {
                return Ok(ObservationOutcome::DuplicateRevision { revision_id });
            }
            return Err(ChangeRadarError::RevisionIdentityConflict);
        }
        if next.revision().observed_at() <= previous.revision().observed_at() {
            return Ok(ObservationOutcome::OutOfOrderRevision { revision_id });
        }

        let changed_fields = semantic_diff(&self.policy, previous.facts(), next.facts());
        let expected = previous.revision().revision_id().clone();
        if changed_fields.is_empty() {
            let commit = ChangeRadarCommit::try_new(Some(expected.clone()), next, None)
                .map_err(ChangeRadarError::Repository)?;
            self.repository
                .apply(commit)
                .map_err(ChangeRadarError::Repository)?;
            return Ok(ObservationOutcome::NoSemanticChange {
                old_revision_id: expected,
                new_revision_id: revision_id,
            });
        }

        let next_for_apply = next.clone();
        let candidate = build_candidate(&self.policy, previous, next, changed_fields);
        let commit = ChangeRadarCommit::try_new(
            Some(candidate.old_revision().revision_id().clone()),
            next_for_apply,
            Some(candidate.clone()),
        )
        .map_err(ChangeRadarError::Repository)?;
        self.repository
            .apply(commit)
            .map_err(ChangeRadarError::Repository)?;
        Ok(ObservationOutcome::SemanticChange(Box::new(candidate)))
    }
}

fn semantic_diff(
    policy: &BoardPolicy,
    before: &NormalizedFacts,
    after: &NormalizedFacts,
) -> Vec<ChangedField> {
    policy
        .tracked_fields
        .iter()
        .filter_map(|field| {
            let old = before.get(field);
            let new = after.get(field);
            (old != new).then(|| ChangedField {
                field: field.clone(),
                before: old.cloned(),
                after: new.cloned(),
            })
        })
        .collect()
}

fn build_candidate(
    policy: &BoardPolicy,
    previous: AcceptedObservation,
    next: AcceptedObservation,
    changed_fields: Vec<ChangedField>,
) -> SemanticChangeCandidate {
    let health = next.health;
    let mut hasher = Sha256::new();
    hasher.update(b"change-radar-event/v1\0");
    update_part(&mut hasher, policy.board_id.as_str().as_bytes());
    update_part(&mut hasher, policy.source_id.as_str().as_bytes());
    hasher.update(policy.revision.to_be_bytes());
    update_part(&mut hasher, policy.affected_scope.as_bytes());
    for field in &policy.tracked_fields {
        update_part(&mut hasher, field.as_str().as_bytes());
    }
    update_part(
        &mut hasher,
        previous.revision().source_id().as_str().as_bytes(),
    );
    update_part(
        &mut hasher,
        previous.revision().revision_id().as_str().as_bytes(),
    );
    update_part(
        &mut hasher,
        next.revision().revision_id().as_str().as_bytes(),
    );
    for change in &changed_fields {
        update_part(&mut hasher, change.field.as_str().as_bytes());
        update_part(
            &mut hasher,
            change
                .before
                .as_ref()
                .map_or(b"", |value| value.as_str().as_bytes()),
        );
        update_part(
            &mut hasher,
            change
                .after
                .as_ref()
                .map_or(b"", |value| value.as_str().as_bytes()),
        );
    }
    let event_id = ChangeEventId(format!("change:{:x}", hasher.finalize()));
    SemanticChangeCandidate {
        event_id,
        board_id: policy.board_id.clone(),
        board_policy_revision: policy.revision,
        source_id: previous.revision().source_id().clone(),
        old_revision: previous.revision,
        new_revision: next.revision,
        health,
        changed_fields,
        affected_scope: policy.affected_scope.clone(),
    }
}

/// Repository failure. No variant carries source/profile content.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChangeRadarRepositoryError {
    InvalidCommit,
    InvalidCapacity,
    BaselineConflict,
    BaselineCapacityExceeded,
    CandidateIdentityConflict,
    CandidateCapacityExceeded,
    InjectedPersistenceFailure,
    Unavailable,
}

impl fmt::Display for ChangeRadarRepositoryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "change-radar repository failure: {self:?}")
    }
}
impl Error for ChangeRadarRepositoryError {}

/// Deterministic domain/service failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChangeRadarError {
    InvalidSemanticField,
    InvalidSemanticValue,
    DuplicateSemanticField,
    TooManySemanticFields,
    EmptyNormalizedFacts,
    InvalidBoardPolicy,
    NormalizedDigestMismatch,
    SourceNotAllowed,
    RevisionIdentityConflict,
    MissingBaseline,
    CandidateLostNormalizedFacts,
    Repository(ChangeRadarRepositoryError),
}

impl fmt::Display for ChangeRadarError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Repository(error) => error.fmt(formatter),
            other => write!(formatter, "change-radar rejected input: {other:?}"),
        }
    }
}
impl Error for ChangeRadarError {}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;
    use ustc_campus_agent_core::source_registry::{
        SourceReviewEvidenceId, SourceReviewerId, SourceUrl,
    };
    use ustc_campus_agent_core::source_revision::{
        NormalizedSnapshotId, ParserIdentity, RawSnapshotId, SourceRevisionProvenance,
    };

    fn field(value: &str) -> SemanticField {
        SemanticField::parse(value).expect("field")
    }
    fn value(value: &str) -> SemanticValue {
        SemanticValue::parse(value).expect("value")
    }
    fn facts(values: &[(&str, &str)]) -> NormalizedFacts {
        NormalizedFacts::try_from_iter(
            values
                .iter()
                .map(|(name, value_text)| (field(name), value(value_text))),
        )
        .expect("facts")
    }
    fn revision_for_source_and_url(
        source_id: &str,
        source_url: &str,
        number: u8,
        observed: i64,
        facts: &NormalizedFacts,
    ) -> SourceRevision {
        SourceRevision::demo_reviewed(
            SourceId::parse(source_id).expect("source id"),
            SourceUrl::parse(source_url).expect("source url"),
            RawSnapshotId::parse(format!("raw:demo:{number}")).expect("raw id"),
            RevisionSha256::parse(format!(
                "sha256:{}",
                char::from(b'a' + number).to_string().repeat(64)
            ))
            .expect("raw digest"),
            NormalizedSnapshotId::parse(format!("normalized:demo:{number}"))
                .expect("normalized id"),
            facts.sha256(),
            ParserIdentity::parse("parser:calendar:v1").expect("parser id"),
            RevisionTimestamp::from_unix_seconds(observed),
            Some(RevisionTimestamp::from_unix_seconds(observed - 10)),
            EffectiveInterval::new(
                Some(RevisionTimestamp::from_unix_seconds(observed + 100)),
                None,
            )
            .expect("interval"),
            SourceReviewerId::parse("reviewer:demo").expect("reviewer"),
            SourceReviewEvidenceId::parse(format!("evidence:demo:{number}")).expect("evidence"),
        )
    }
    fn revision_for_source(
        source_id: &str,
        number: u8,
        observed: i64,
        facts: &NormalizedFacts,
    ) -> SourceRevision {
        revision_for_source_and_url(
            source_id,
            &format!("https://example.com/{}", source_id.replace(':', "/")),
            number,
            observed,
            facts,
        )
    }
    fn revision(number: u8, observed: i64, facts: &NormalizedFacts) -> SourceRevision {
        revision_for_source("source:demo:calendar", number, observed, facts)
    }
    fn observation(number: u8, observed: i64, values: &[(&str, &str)]) -> AcceptedObservation {
        let facts = facts(values);
        AcceptedObservation::new(
            revision(number, observed, &facts),
            facts,
            SourceRevisionHealth::Current,
        )
        .expect("observation")
    }
    fn observation_for_source(
        source_id: &str,
        number: u8,
        observed: i64,
        values: &[(&str, &str)],
    ) -> AcceptedObservation {
        let facts = facts(values);
        AcceptedObservation::new(
            revision_for_source(source_id, number, observed, &facts),
            facts,
            SourceRevisionHealth::Current,
        )
        .expect("observation")
    }
    fn service() -> ChangeRadarService<InMemoryChangeRadarRepository> {
        let policy = BoardPolicy::new(
            BoardId::parse("board:academic-calendar").expect("board"),
            SourceId::parse("source:demo:calendar").expect("source"),
            1,
            [
                field("registration.deadline"),
                field("registration.location"),
            ],
            "all_students",
        )
        .expect("policy");
        ChangeRadarService::new(policy, InMemoryChangeRadarRepository::new())
    }

    #[test]
    fn two_demo_reviewed_revisions_produce_semantic_change() {
        let mut service = service();
        let first = observation(
            1,
            100,
            &[
                ("registration.deadline", "2026-09-01"),
                ("registration.location", "West Campus"),
            ],
        );
        let second = observation(
            2,
            200,
            &[
                ("registration.deadline", "2026-09-03"),
                ("registration.location", "West Campus"),
            ],
        );
        assert!(matches!(
            service.observe(first),
            Ok(ObservationOutcome::BaselineEstablished { .. })
        ));
        let outcome = service.observe(second).expect("semantic change");
        let ObservationOutcome::SemanticChange(candidate) = outcome else {
            panic!("expected candidate")
        };
        assert_eq!(candidate.changed_fields().len(), 1);
        assert_eq!(
            candidate.changed_fields()[0].field().as_str(),
            "registration.deadline"
        );
        assert_eq!(candidate.affected_scope(), "all_students");
        assert!(matches!(
            candidate.new_revision().provenance(),
            SourceRevisionProvenance::DemoReviewed { .. }
        ));
        assert_eq!(
            candidate.new_revision().source_url().as_str(),
            "https://example.com/source/demo/calendar"
        );
        assert_eq!(candidate.health(), SourceRevisionHealth::Current);
        assert_eq!(service.repository().candidate_count(), 1);
    }

    #[test]
    fn duplicate_and_out_of_order_revisions_create_no_change() {
        let mut service = service();
        let first = observation(1, 100, &[("registration.deadline", "2026-09-01")]);
        service.observe(first.clone()).expect("baseline");
        assert!(matches!(
            service.observe(first),
            Ok(ObservationOutcome::DuplicateRevision { .. })
        ));
        let older = observation(2, 90, &[("registration.deadline", "2026-09-03")]);
        assert!(matches!(
            service.observe(older),
            Ok(ObservationOutcome::OutOfOrderRevision { .. })
        ));
        assert_eq!(service.repository().candidate_count(), 0);
    }

    #[test]
    fn board_policy_rejects_an_unpinned_source_before_repository_access() {
        let mut service = service();
        service
            .observe(observation(
                1,
                100,
                &[("registration.deadline", "2026-09-01")],
            ))
            .expect("baseline");
        let foreign = observation_for_source(
            "source:demo:fees",
            2,
            200,
            &[("registration.deadline", "2026-09-03")],
        );
        assert!(matches!(
            service.observe(foreign),
            Err(ChangeRadarError::SourceNotAllowed)
        ));
        let foreign_source = SourceId::parse("source:demo:fees").expect("source");
        assert!(matches!(
            service.observe_unavailable(foreign_source.clone()),
            Err(ChangeRadarError::SourceNotAllowed)
        ));
        assert_eq!(service.repository().candidate_count(), 0);
        assert_eq!(
            service.repository().current_revision_id(&foreign_source),
            None
        );
    }

    #[test]
    fn no_semantic_change_advances_baseline_without_event() {
        let mut service = service();
        service
            .observe(observation(
                1,
                100,
                &[
                    ("registration.deadline", "2026-09-01"),
                    ("layout.note", "first layout"),
                ],
            ))
            .expect("baseline");
        let next = observation(
            2,
            200,
            &[
                ("registration.deadline", "2026-09-01"),
                ("layout.note", "second layout"),
            ],
        );
        assert!(matches!(
            service.observe(next),
            Ok(ObservationOutcome::NoSemanticChange { .. })
        ));
        assert_eq!(service.repository().candidate_count(), 0);
    }

    #[test]
    fn stale_conflict_and_unavailable_are_stable_non_mutating_results() {
        let mut service = service();
        let base = observation(1, 100, &[("registration.deadline", "2026-09-01")]);
        let source = base.revision().source_id().clone();
        let baseline_id = base.revision().revision_id().clone();
        service.observe(base).expect("baseline");
        for (number, health) in [
            (2, SourceRevisionHealth::Stale),
            (3, SourceRevisionHealth::Conflicting),
        ] {
            let facts = facts(&[("registration.deadline", "2026-09-03")]);
            let next = AcceptedObservation::new(revision(number, 200, &facts), facts, health)
                .expect("observation");
            let outcome = service.observe(next).expect("stable result");
            assert!(matches!(
                (health, outcome),
                (
                    SourceRevisionHealth::Stale,
                    ObservationOutcome::StaleRevision { .. }
                ) | (
                    SourceRevisionHealth::Conflicting,
                    ObservationOutcome::ConflictingRevision { .. }
                )
            ));
        }
        assert!(matches!(
            service
                .observe_unavailable(source.clone())
                .expect("unavailable"),
            ObservationOutcome::SourceUnavailable { .. }
        ));
        assert_eq!(
            service
                .repository()
                .current_revision_id(&source)
                .map(SourceRevisionId::as_str),
            Some(baseline_id.as_str())
        );
        assert_eq!(service.repository().candidate_count(), 0);
    }

    #[test]
    fn repository_commit_rejects_non_current_and_mismatched_lineage() {
        let stale_facts = facts(&[("registration.deadline", "2026-09-01")]);
        let stale = AcceptedObservation::new(
            revision(1, 100, &stale_facts),
            stale_facts,
            SourceRevisionHealth::Stale,
        )
        .expect("stale observation");
        assert!(matches!(
            ChangeRadarCommit::try_new(None, stale, None),
            Err(ChangeRadarRepositoryError::InvalidCommit)
        ));

        let mut service = service();
        service
            .observe(observation(
                1,
                100,
                &[("registration.deadline", "2026-09-01")],
            ))
            .expect("baseline");
        let ObservationOutcome::SemanticChange(candidate) = service
            .observe(observation(
                2,
                200,
                &[("registration.deadline", "2026-09-03")],
            ))
            .expect("candidate")
        else {
            panic!("candidate")
        };
        let wrong_next = observation(3, 300, &[("registration.deadline", "2026-09-05")]);
        assert!(matches!(
            ChangeRadarCommit::try_new(
                Some(candidate.old_revision().revision_id().clone()),
                wrong_next,
                Some((*candidate).clone()),
            ),
            Err(ChangeRadarRepositoryError::InvalidCommit)
        ));
    }

    #[test]
    fn persistence_failure_advances_neither_candidate_nor_baseline() {
        let mut service = service();
        let first = observation(1, 100, &[("registration.deadline", "2026-09-01")]);
        let source = first.revision().source_id().clone();
        let baseline_id = first.revision().revision_id().clone();
        service.observe(first).expect("baseline");
        service.repository_mut().inject_next_apply_failure();
        let changed = observation(2, 200, &[("registration.deadline", "2026-09-03")]);
        assert!(matches!(
            service.observe(changed),
            Err(ChangeRadarError::Repository(
                ChangeRadarRepositoryError::InjectedPersistenceFailure
            ))
        ));
        assert_eq!(
            service
                .repository()
                .current_revision_id(&source)
                .map(SourceRevisionId::as_str),
            Some(baseline_id.as_str())
        );
        assert_eq!(service.repository().candidate_count(), 0);
    }

    #[test]
    fn repository_limits_fail_before_baseline_or_candidate_mutation() {
        assert!(matches!(
            InMemoryChangeRadarRepository::with_limits(0, 1),
            Err(ChangeRadarRepositoryError::InvalidCapacity)
        ));
        let policy = BoardPolicy::new(
            BoardId::parse("board:academic-calendar").expect("board"),
            SourceId::parse("source:demo:calendar").expect("source"),
            1,
            [field("registration.deadline")],
            "all_students",
        )
        .expect("policy");
        let repository = InMemoryChangeRadarRepository::with_limits(1, 1).expect("limits");
        let mut service = ChangeRadarService::new(policy, repository);
        let source_a = SourceId::parse("source:demo:calendar").expect("source a");
        let source_b = SourceId::parse("source:demo:fees").expect("source b");

        service
            .observe(observation(
                1,
                100,
                &[("registration.deadline", "2026-09-01")],
            ))
            .expect("baseline");
        let second = observation(2, 200, &[("registration.deadline", "2026-09-03")]);
        let second_id = second.revision().revision_id().clone();
        service.observe(second).expect("first candidate");
        assert!(matches!(
            service.observe(observation(
                3,
                300,
                &[("registration.deadline", "2026-09-05")],
            )),
            Err(ChangeRadarError::Repository(
                ChangeRadarRepositoryError::CandidateCapacityExceeded
            ))
        ));
        assert_eq!(
            service
                .repository()
                .current_revision_id(&source_a)
                .map(SourceRevisionId::as_str),
            Some(second_id.as_str())
        );

        let source_b_observation = observation_for_source(
            "source:demo:fees",
            4,
            400,
            &[("registration.deadline", "2026-09-07")],
        );
        let source_b_commit =
            ChangeRadarCommit::try_new(None, source_b_observation, None).expect("commit");
        assert!(matches!(
            service.repository_mut().apply(source_b_commit),
            Err(ChangeRadarRepositoryError::BaselineCapacityExceeded)
        ));
        assert_eq!(service.repository().current_revision_id(&source_b), None);
        assert_eq!(service.repository().candidate_count(), 1);
    }

    #[test]
    fn source_revision_identity_and_candidate_evidence_bind_the_exact_reviewed_url() {
        let facts = facts(&[("registration.deadline", "2026-09-01")]);
        let west = revision_for_source_and_url(
            "source:demo:calendar",
            "https://example.com/calendar/west",
            1,
            100,
            &facts,
        );
        let east = revision_for_source_and_url(
            "source:demo:calendar",
            "https://example.com/calendar/east",
            1,
            100,
            &facts,
        );
        assert_ne!(west.revision_id(), east.revision_id());
        assert_eq!(
            east.source_url().as_str(),
            "https://example.com/calendar/east"
        );
    }

    #[test]
    fn normalized_digest_mismatch_is_rejected() {
        let first = facts(&[("registration.deadline", "2026-09-01")]);
        let different = facts(&[("registration.deadline", "2026-09-03")]);
        assert!(matches!(
            AcceptedObservation::new(
                revision(1, 100, &first),
                different,
                SourceRevisionHealth::Current,
            ),
            Err(ChangeRadarError::NormalizedDigestMismatch)
        ));
    }

    #[test]
    fn event_identity_is_deterministic_across_fact_insertion_order() {
        let first_a = observation(
            1,
            100,
            &[
                ("registration.deadline", "2026-09-01"),
                ("registration.location", "West Campus"),
            ],
        );
        let second_a = observation(
            2,
            200,
            &[
                ("registration.deadline", "2026-09-03"),
                ("registration.location", "East Campus"),
            ],
        );
        let first_b = observation(
            1,
            100,
            &[
                ("registration.location", "West Campus"),
                ("registration.deadline", "2026-09-01"),
            ],
        );
        let second_b = observation(
            2,
            200,
            &[
                ("registration.location", "East Campus"),
                ("registration.deadline", "2026-09-03"),
            ],
        );
        let mut service_a = service();
        service_a.observe(first_a).expect("baseline a");
        let ObservationOutcome::SemanticChange(candidate_a) =
            service_a.observe(second_a).expect("change a")
        else {
            panic!("candidate a")
        };
        let mut service_b = service();
        service_b.observe(first_b).expect("baseline b");
        let ObservationOutcome::SemanticChange(candidate_b) =
            service_b.observe(second_b).expect("change b")
        else {
            panic!("candidate b")
        };
        assert_eq!(candidate_a.event_id(), candidate_b.event_id());
        assert_eq!(candidate_a.changed_fields(), candidate_b.changed_fields());
    }

    #[test]
    fn event_identity_binds_the_complete_board_policy_scope() {
        let make_policy = |scope: &str| {
            BoardPolicy::new(
                BoardId::parse("board:academic-calendar").expect("board"),
                SourceId::parse("source:demo:calendar").expect("source"),
                1,
                [field("registration.deadline")],
                scope,
            )
            .expect("policy")
        };
        let first = observation(1, 100, &[("registration.deadline", "2026-09-01")]);
        let second = observation(2, 200, &[("registration.deadline", "2026-09-03")]);
        let mut all_students = ChangeRadarService::new(
            make_policy("all_students"),
            InMemoryChangeRadarRepository::new(),
        );
        let mut graduates = ChangeRadarService::new(
            make_policy("graduate_students"),
            InMemoryChangeRadarRepository::new(),
        );
        all_students.observe(first.clone()).expect("baseline all");
        graduates.observe(first).expect("baseline graduates");
        let ObservationOutcome::SemanticChange(all_candidate) =
            all_students.observe(second.clone()).expect("change all")
        else {
            panic!("all candidate")
        };
        let ObservationOutcome::SemanticChange(graduate_candidate) =
            graduates.observe(second).expect("change graduates")
        else {
            panic!("graduate candidate")
        };
        assert_ne!(all_candidate.event_id(), graduate_candidate.event_id());
    }
}
