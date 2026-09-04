//! Durable, fail-closed ChangeRadar publication repository for the bounded demo.
//!
//! Persisted bytes are private adapter DTOs. Reopening reconstructs the exact
//! board/candidate from checked observations, then uses the single M70 recovery
//! entry point to restore sealed review/publication commits without M60 I/O.

use std::collections::BTreeMap;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::os::unix::fs::{MetadataExt, OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use ustc_campus_agent_change_radar::{
    AcceptedObservation, BoardFeedPolicy, BoardId, BoardPolicy, ChangeEventId,
    ChangePublicationCommit, ChangePublicationRecoveryProjection, ChangePublicationRecoveryRecord,
    ChangePublicationRepository, ChangePublicationRepositoryError, ChangeRadarService,
    ChangeReviewCommit, ChangeReviewDecision, ChangeReviewReceipt, ChangeReviewRecoveryProjection,
    InMemoryChangeRadarRepository, NormalizedFacts, ObservationOutcome, PublishedChangeEvent,
    SemanticChangeCandidate, SemanticField, SemanticValue,
};
use ustc_campus_agent_core::identity::UserId;
use ustc_campus_agent_core::source_registry::{
    SourceId, SourceReviewEvidenceId, SourceReviewerId, SourceUrl,
};
use ustc_campus_agent_core::source_revision::{
    EffectiveInterval, NormalizedSnapshotId, ParserIdentity, RawSnapshotId, RevisionSha256,
    RevisionTimestamp, SourceRevision, SourceRevisionHealth, SourceRevisionProvenance,
};

const STATE_SCHEMA_VERSION: u8 = 1;
const DEFAULT_MAX_BYTES: u64 = 1_048_576;
const TEMP_ATTEMPTS: usize = 16;

#[derive(Clone)]
pub(crate) struct ChangePublicationBootstrap {
    pub(crate) board_policy: BoardPolicy,
    pub(crate) old_observation: AcceptedObservation,
    pub(crate) new_observation: AcceptedObservation,
    pub(crate) candidate: SemanticChangeCandidate,
    pub(crate) review: ChangeReviewReceipt,
    pub(crate) feed_policy: BoardFeedPolicy,
    pub(crate) published_at: RevisionTimestamp,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PersistedBoardPolicy {
    board_id: String,
    source_id: String,
    revision: u64,
    tracked_fields: Vec<String>,
    affected_scope: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PersistedObservation {
    source_id: String,
    source_url: String,
    revision_id: String,
    raw_snapshot_id: String,
    raw_sha256: String,
    normalized_snapshot_id: String,
    normalized_sha256: String,
    parser_identity: String,
    observed_at_secs: i64,
    published_at_secs: Option<i64>,
    effective_from_secs: Option<i64>,
    effective_to_secs: Option<i64>,
    source_reviewer: String,
    source_review_evidence: String,
    health: String,
    facts: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PersistedBinding {
    board_policy: PersistedBoardPolicy,
    old_observation: PersistedObservation,
    new_observation: PersistedObservation,
    candidate_event_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PersistedReview {
    receipt_id: String,
    reviewer: String,
    reviewed_at_secs: i64,
    decision: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PersistedPublication {
    board_id: String,
    board_policy_revision: u64,
    feed_title: String,
    feed_author: String,
    feed_public_base_url: String,
    evidence_set_digest: String,
    published_at_secs: i64,
    stable_guid: String,
    receipt_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PersistedChangePublicationState {
    schema_version: u8,
    binding: PersistedBinding,
    review: Option<PersistedReview>,
    publication: Option<PersistedPublication>,
}

impl PersistedChangePublicationState {
    fn fresh(bootstrap: &ChangePublicationBootstrap) -> Result<Self, String> {
        Ok(Self {
            schema_version: STATE_SCHEMA_VERSION,
            binding: PersistedBinding::from_bootstrap(bootstrap)?,
            review: None,
            publication: None,
        })
    }
}

impl PersistedBinding {
    fn from_bootstrap(bootstrap: &ChangePublicationBootstrap) -> Result<Self, String> {
        Ok(Self {
            board_policy: PersistedBoardPolicy::from_policy(&bootstrap.board_policy),
            old_observation: PersistedObservation::from_observation(&bootstrap.old_observation)?,
            new_observation: PersistedObservation::from_observation(&bootstrap.new_observation)?,
            candidate_event_id: bootstrap.candidate.event_id().as_str().to_owned(),
        })
    }
}

impl PersistedBoardPolicy {
    fn from_policy(policy: &BoardPolicy) -> Self {
        Self {
            board_id: policy.board_id().as_str().to_owned(),
            source_id: policy.source_id().as_str().to_owned(),
            revision: policy.revision(),
            tracked_fields: policy
                .tracked_fields()
                .map(|field| field.as_str().to_owned())
                .collect(),
            affected_scope: policy.affected_scope().to_owned(),
        }
    }

    fn recover(&self) -> Result<BoardPolicy, String> {
        let fields = self
            .tracked_fields
            .iter()
            .map(|value| SemanticField::parse(value).map_err(error_text))
            .collect::<Result<Vec<_>, _>>()?;
        BoardPolicy::new(
            BoardId::parse(&self.board_id).map_err(error_text)?,
            SourceId::parse(&self.source_id).map_err(error_text)?,
            self.revision,
            fields,
            &self.affected_scope,
        )
        .map_err(error_text)
    }
}

impl PersistedObservation {
    fn from_observation(observation: &AcceptedObservation) -> Result<Self, String> {
        let revision = observation.revision();
        let SourceRevisionProvenance::DemoReviewed { reviewer, evidence } = revision.provenance()
        else {
            return Err("change publication persistence requires DemoReviewed evidence".to_owned());
        };
        let health = match observation.health() {
            SourceRevisionHealth::Current => "current",
            SourceRevisionHealth::Stale => "stale",
            SourceRevisionHealth::Conflicting => "conflicting",
        };
        Ok(Self {
            source_id: revision.source_id().as_str().to_owned(),
            source_url: revision.source_url().as_str().to_owned(),
            revision_id: revision.revision_id().as_str().to_owned(),
            raw_snapshot_id: revision.raw_snapshot_id().as_str().to_owned(),
            raw_sha256: revision.raw_sha256().as_str().to_owned(),
            normalized_snapshot_id: revision.normalized_snapshot_id().as_str().to_owned(),
            normalized_sha256: revision.normalized_sha256().as_str().to_owned(),
            parser_identity: revision.parser_identity().as_str().to_owned(),
            observed_at_secs: revision.observed_at().unix_seconds(),
            published_at_secs: revision.published_at().map(RevisionTimestamp::unix_seconds),
            effective_from_secs: revision
                .effective_interval()
                .from()
                .map(RevisionTimestamp::unix_seconds),
            effective_to_secs: revision
                .effective_interval()
                .to()
                .map(RevisionTimestamp::unix_seconds),
            source_reviewer: reviewer.as_str().to_owned(),
            source_review_evidence: evidence.as_str().to_owned(),
            health: health.to_owned(),
            facts: observation
                .facts()
                .iter()
                .map(|(field, value)| (field.as_str().to_owned(), value.as_str().to_owned()))
                .collect(),
        })
    }

    fn recover(&self) -> Result<AcceptedObservation, String> {
        let facts = NormalizedFacts::try_from_iter(
            self.facts
                .iter()
                .map(|(field, value)| {
                    Ok((
                        SemanticField::parse(field).map_err(error_text)?,
                        SemanticValue::parse(value).map_err(error_text)?,
                    ))
                })
                .collect::<Result<Vec<_>, String>>()?,
        )
        .map_err(error_text)?;
        let effective_interval = EffectiveInterval::new(
            self.effective_from_secs
                .map(RevisionTimestamp::from_unix_seconds),
            self.effective_to_secs
                .map(RevisionTimestamp::from_unix_seconds),
        )
        .map_err(error_text)?;
        let revision = SourceRevision::demo_reviewed(
            SourceId::parse(&self.source_id).map_err(error_text)?,
            SourceUrl::parse(&self.source_url).map_err(error_text)?,
            RawSnapshotId::parse(&self.raw_snapshot_id).map_err(error_text)?,
            RevisionSha256::parse(&self.raw_sha256).map_err(error_text)?,
            NormalizedSnapshotId::parse(&self.normalized_snapshot_id).map_err(error_text)?,
            RevisionSha256::parse(&self.normalized_sha256).map_err(error_text)?,
            ParserIdentity::parse(&self.parser_identity).map_err(error_text)?,
            RevisionTimestamp::from_unix_seconds(self.observed_at_secs),
            self.published_at_secs
                .map(RevisionTimestamp::from_unix_seconds),
            effective_interval,
            SourceReviewerId::parse(&self.source_reviewer).map_err(error_text)?,
            SourceReviewEvidenceId::parse(&self.source_review_evidence).map_err(error_text)?,
        );
        if revision.revision_id().as_str() != self.revision_id {
            return Err("persisted change revision identity is incoherent".to_owned());
        }
        let health = match self.health.as_str() {
            "current" => SourceRevisionHealth::Current,
            "stale" => SourceRevisionHealth::Stale,
            "conflicting" => SourceRevisionHealth::Conflicting,
            _ => return Err("persisted change health is invalid".to_owned()),
        };
        AcceptedObservation::new(revision, facts, health).map_err(error_text)
    }
}

impl PersistedReview {
    fn from_review(review: &ChangeReviewReceipt) -> Result<Self, String> {
        let decision = match review.decision() {
            ChangeReviewDecision::Approved => "approved",
            ChangeReviewDecision::Rejected(_) => {
                return Err("bounded ChangeRadar fixture persists only approved review".to_owned());
            }
        };
        Ok(Self {
            receipt_id: review.receipt_id().as_str().to_owned(),
            reviewer: review.reviewer().as_str().to_owned(),
            reviewed_at_secs: review.reviewed_at().unix_seconds(),
            decision: decision.to_owned(),
        })
    }

    fn recovery_projection(&self) -> Result<ChangeReviewRecoveryProjection, String> {
        let decision = match self.decision.as_str() {
            "approved" => ChangeReviewDecision::Approved,
            _ => return Err("persisted ChangeRadar review decision is invalid".to_owned()),
        };
        Ok(ChangeReviewRecoveryProjection::new(
            UserId::parse(&self.reviewer).map_err(error_text)?,
            RevisionTimestamp::from_unix_seconds(self.reviewed_at_secs),
            decision,
            &self.receipt_id,
        ))
    }
}

impl PersistedPublication {
    fn from_publication(publication: &PublishedChangeEvent) -> Self {
        Self {
            board_id: publication.feed_policy().board_id().as_str().to_owned(),
            board_policy_revision: publication.feed_policy().board_policy_revision(),
            feed_title: publication.feed_policy().title().to_owned(),
            feed_author: publication.feed_policy().author_name().to_owned(),
            feed_public_base_url: publication.feed_policy().public_base_url().to_owned(),
            evidence_set_digest: publication.evidence_set_digest().as_str().to_owned(),
            published_at_secs: publication.published_at().unix_seconds(),
            stable_guid: publication.stable_guid().as_str().to_owned(),
            receipt_id: publication.receipt_id().as_str().to_owned(),
        }
    }

    fn recovery_projection(&self) -> Result<ChangePublicationRecoveryProjection, String> {
        let feed_policy = BoardFeedPolicy::new(
            BoardId::parse(&self.board_id).map_err(error_text)?,
            self.board_policy_revision,
            &self.feed_title,
            &self.feed_author,
            &self.feed_public_base_url,
        )
        .map_err(error_text)?;
        Ok(ChangePublicationRecoveryProjection::new(
            feed_policy,
            RevisionSha256::parse(&self.evidence_set_digest).map_err(error_text)?,
            RevisionTimestamp::from_unix_seconds(self.published_at_secs),
            &self.receipt_id,
            &self.stable_guid,
        ))
    }
}

/// Composition-owned durable adapter used by both the administrator mutation
/// and ordinary public ChangeRadar query path.
pub(crate) struct DurableChangeRadarRepository {
    path: PathBuf,
    bootstrap: ChangePublicationBootstrap,
    state: PersistedChangePublicationState,
    inner: InMemoryChangeRadarRepository,
    max_bytes: u64,
    fail_persist_after_commits: Option<u8>,
    fail_parent_sync_after_commits: Option<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PersistOutcome {
    Synced,
    RenamedParentSyncUncertain,
}

fn consume_failure_injection(remaining_commits: &mut Option<u8>) -> bool {
    match *remaining_commits {
        None => false,
        Some(0) => {
            *remaining_commits = None;
            true
        }
        Some(remaining) => {
            *remaining_commits = Some(remaining - 1);
            false
        }
    }
}

impl DurableChangeRadarRepository {
    pub(crate) fn open(
        path: &Path,
        bootstrap: ChangePublicationBootstrap,
        allow_fresh_bootstrap: bool,
    ) -> Result<Self, String> {
        Self::open_with_limit(path, bootstrap, allow_fresh_bootstrap, DEFAULT_MAX_BYTES)
    }

    fn open_with_limit(
        path: &Path,
        bootstrap: ChangePublicationBootstrap,
        allow_fresh_bootstrap: bool,
        max_bytes: u64,
    ) -> Result<Self, String> {
        if max_bytes == 0 {
            return Err("change publication byte limit must be nonzero".to_owned());
        }
        validate_secure_parent(path).map_err(repository_error_text)?;
        let expected_fresh = PersistedChangePublicationState::fresh(&bootstrap)?;
        let (state, must_persist_fresh) =
            match read_existing(path, max_bytes).map_err(repository_error_text)? {
                None if allow_fresh_bootstrap => (expected_fresh, true),
                None => {
                    return Err(
                        "change publication state is missing from an existing state set".to_owned(),
                    );
                }
                Some(bytes) => {
                    let state: PersistedChangePublicationState = serde_json::from_slice(&bytes)
                        .map_err(|_| {
                            "change publication state is not valid schema-v1 JSON".to_owned()
                        })?;
                    let canonical = serde_json::to_vec(&state)
                        .map_err(|_| "change publication canonicalization failed".to_owned())?;
                    if canonical != bytes {
                        return Err("change publication state is noncanonical".to_owned());
                    }
                    (state, false)
                }
            };
        validate_state(&state, &bootstrap)?;
        let inner = rebuild_inner(&state, &bootstrap)?;
        let mut repository = Self {
            path: path.to_owned(),
            bootstrap,
            state,
            inner,
            max_bytes,
            fail_persist_after_commits: None,
            fail_parent_sync_after_commits: None,
        };
        if must_persist_fresh {
            let state = repository.state.clone();
            repository.persist(&state).map_err(repository_error_text)?;
        }
        Ok(repository)
    }

    pub(crate) fn publication_receipt_id(
        &self,
    ) -> Result<Option<&str>, ChangePublicationRepositoryError> {
        self.verify_durable_matches()?;
        Ok(self
            .state
            .publication
            .as_ref()
            .map(|value| value.receipt_id.as_str()))
    }

    pub(crate) fn review_count(&self) -> Result<usize, ChangePublicationRepositoryError> {
        self.verify_durable_matches()?;
        Ok(usize::from(self.state.review.is_some()))
    }

    pub(crate) fn publication_count(&self) -> Result<usize, ChangePublicationRepositoryError> {
        self.verify_durable_matches()?;
        Ok(usize::from(self.state.publication.is_some()))
    }

    pub(crate) fn fail_next_persist(&mut self) {
        self.fail_persist_after_commits = Some(0);
    }

    pub(crate) fn fail_publication_persist_after_review(&mut self) {
        self.fail_persist_after_commits = Some(1);
    }

    pub(crate) fn fail_next_parent_sync_after_rename(&mut self) {
        self.fail_parent_sync_after_commits = Some(0);
    }

    pub(crate) fn fail_publication_parent_sync_after_review(&mut self) {
        self.fail_parent_sync_after_commits = Some(1);
    }

    fn verify_durable_matches(&self) -> Result<(), ChangePublicationRepositoryError> {
        let expected = serde_json::to_vec(&self.state)
            .map_err(|_| ChangePublicationRepositoryError::Unavailable)?;
        match read_existing(&self.path, self.max_bytes)? {
            Some(actual) if actual == expected => Ok(()),
            _ => Err(ChangePublicationRepositoryError::Unavailable),
        }
    }

    fn persist(
        &mut self,
        state: &PersistedChangePublicationState,
    ) -> Result<PersistOutcome, ChangePublicationRepositoryError> {
        if consume_failure_injection(&mut self.fail_persist_after_commits) {
            return Err(ChangePublicationRepositoryError::Unavailable);
        }
        validate_state(state, &self.bootstrap)
            .map_err(|_| ChangePublicationRepositoryError::Unavailable)?;
        let bytes =
            serde_json::to_vec(state).map_err(|_| ChangePublicationRepositoryError::Unavailable)?;
        if u64::try_from(bytes.len()).unwrap_or(u64::MAX) > self.max_bytes {
            return Err(ChangePublicationRepositoryError::Unavailable);
        }
        validate_secure_parent(&self.path)?;
        validate_existing_destination(&self.path, self.max_bytes)?;
        let parent = direct_parent(&self.path)?;
        let mut temporary = None;
        let mut file = None;
        for _ in 0..TEMP_ATTEMPTS {
            let candidate = unpredictable_temporary(parent, &self.path)?;
            match OpenOptions::new()
                .write(true)
                .create_new(true)
                .mode(0o600)
                .custom_flags(libc::O_NOFOLLOW)
                .open(&candidate)
            {
                Ok(opened) => {
                    temporary = Some(candidate);
                    file = Some(opened);
                    break;
                }
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(_) => return Err(ChangePublicationRepositoryError::Unavailable),
            }
        }
        let temporary = temporary.ok_or(ChangePublicationRepositoryError::Unavailable)?;
        let mut file = file.ok_or(ChangePublicationRepositoryError::Unavailable)?;
        let result = (|| {
            validate_primary_metadata(
                &file
                    .metadata()
                    .map_err(|_| ChangePublicationRepositoryError::Unavailable)?,
                self.max_bytes,
            )?;
            file.write_all(&bytes)
                .map_err(|_| ChangePublicationRepositoryError::Unavailable)?;
            file.sync_all()
                .map_err(|_| ChangePublicationRepositoryError::Unavailable)?;
            drop(file);
            fs::rename(&temporary, &self.path)
                .map_err(|_| ChangePublicationRepositoryError::Unavailable)?;
            let parent_sync = if consume_failure_injection(&mut self.fail_parent_sync_after_commits)
            {
                Err(std::io::Error::other("injected parent sync failure"))
            } else {
                File::open(parent).and_then(|directory| directory.sync_all())
            };
            if parent_sync.is_ok() {
                return Ok(PersistOutcome::Synced);
            }
            match read_existing(&self.path, self.max_bytes)? {
                Some(actual) if actual == bytes => Ok(PersistOutcome::RenamedParentSyncUncertain),
                _ => Err(ChangePublicationRepositoryError::Unavailable),
            }
        })();
        if result.is_err() {
            let _ = fs::remove_file(&temporary);
        }
        result
    }
}

impl ChangePublicationRepository for DurableChangeRadarRepository {
    fn find_candidate(
        &self,
        event_id: &ChangeEventId,
    ) -> Result<Option<SemanticChangeCandidate>, ChangePublicationRepositoryError> {
        self.verify_durable_matches()?;
        self.inner.find_candidate(event_id)
    }

    fn find_review(
        &self,
        event_id: &ChangeEventId,
    ) -> Result<Option<ChangeReviewReceipt>, ChangePublicationRepositoryError> {
        self.verify_durable_matches()?;
        self.inner.find_review(event_id)
    }

    fn find_publication(
        &self,
        event_id: &ChangeEventId,
    ) -> Result<Option<PublishedChangeEvent>, ChangePublicationRepositoryError> {
        self.verify_durable_matches()?;
        self.inner.find_publication(event_id)
    }

    fn apply_review(
        &mut self,
        commit: ChangeReviewCommit,
    ) -> Result<(), ChangePublicationRepositoryError> {
        self.verify_durable_matches()?;
        if commit.candidate() != &self.bootstrap.candidate {
            return Err(ChangePublicationRepositoryError::InvalidCommit);
        }
        let persisted = PersistedReview::from_review(commit.review())
            .map_err(|_| ChangePublicationRepositoryError::InvalidCommit)?;
        let expected = PersistedReview::from_review(&self.bootstrap.review)
            .map_err(|_| ChangePublicationRepositoryError::InvalidCommit)?;
        if persisted != expected {
            return Err(ChangePublicationRepositoryError::ReviewConflict);
        }
        if let Some(existing) = self.state.review.as_ref() {
            return if existing == &persisted {
                Ok(())
            } else {
                Err(ChangePublicationRepositoryError::ReviewConflict)
            };
        }
        let mut next = self.state.clone();
        next.review = Some(persisted);
        let mut next_inner = self.inner.clone();
        next_inner.apply_review(commit)?;
        let persist = self.persist(&next)?;
        self.inner = next_inner;
        self.state = next;
        match persist {
            PersistOutcome::Synced => Ok(()),
            PersistOutcome::RenamedParentSyncUncertain => {
                Err(ChangePublicationRepositoryError::Unavailable)
            }
        }
    }

    fn apply_publication(
        &mut self,
        commit: ChangePublicationCommit,
    ) -> Result<(), ChangePublicationRepositoryError> {
        self.verify_durable_matches()?;
        let publication = commit.publication();
        if publication.candidate() != &self.bootstrap.candidate
            || publication.review() != &self.bootstrap.review
            || publication.feed_policy() != &self.bootstrap.feed_policy
            || publication.published_at() != self.bootstrap.published_at
        {
            return Err(ChangePublicationRepositoryError::InvalidCommit);
        }
        let persisted = PersistedPublication::from_publication(publication);
        if let Some(existing) = self.state.publication.as_ref() {
            return if existing == &persisted {
                Ok(())
            } else {
                Err(ChangePublicationRepositoryError::PublicationConflict)
            };
        }
        if self.state.review.is_none() {
            return Err(ChangePublicationRepositoryError::ReviewNotFound);
        }
        let mut next = self.state.clone();
        next.publication = Some(persisted);
        let mut next_inner = self.inner.clone();
        next_inner.apply_publication(commit)?;
        let persist = self.persist(&next)?;
        self.inner = next_inner;
        self.state = next;
        match persist {
            PersistOutcome::Synced => Ok(()),
            PersistOutcome::RenamedParentSyncUncertain => {
                Err(ChangePublicationRepositoryError::Unavailable)
            }
        }
    }

    fn feed_items(
        &self,
        board_id: &BoardId,
    ) -> Result<Vec<PublishedChangeEvent>, ChangePublicationRepositoryError> {
        self.verify_durable_matches()?;
        self.inner.feed_items(board_id)
    }
}

fn validate_state(
    state: &PersistedChangePublicationState,
    bootstrap: &ChangePublicationBootstrap,
) -> Result<(), String> {
    if state.schema_version != STATE_SCHEMA_VERSION {
        return Err("change publication state schema version is unsupported".to_owned());
    }
    let expected = PersistedBinding::from_bootstrap(bootstrap)?;
    if state.binding != expected {
        return Err("change publication state is bound to different fixture evidence".to_owned());
    }
    if state.publication.is_some() && state.review.is_none() {
        return Err("change publication state has a publication without review".to_owned());
    }
    if let Some(review) = state.review.as_ref() {
        let expected_review = PersistedReview::from_review(&bootstrap.review)?;
        if review != &expected_review {
            return Err("persisted ChangeRadar review conflicts with fixture authority".to_owned());
        }
    }
    if let Some(publication) = state.publication.as_ref() {
        let expected_policy = &bootstrap.feed_policy;
        if publication.board_id != expected_policy.board_id().as_str()
            || publication.board_policy_revision != expected_policy.board_policy_revision()
            || publication.feed_title != expected_policy.title()
            || publication.feed_author != expected_policy.author_name()
            || publication.feed_public_base_url != expected_policy.public_base_url()
            || publication.published_at_secs != bootstrap.published_at.unix_seconds()
        {
            return Err(
                "persisted ChangeRadar publication conflicts with fixture policy".to_owned(),
            );
        }
    }
    Ok(())
}

fn rebuild_inner(
    state: &PersistedChangePublicationState,
    bootstrap: &ChangePublicationBootstrap,
) -> Result<InMemoryChangeRadarRepository, String> {
    let policy = state.binding.board_policy.recover()?;
    let old = state.binding.old_observation.recover()?;
    let new = state.binding.new_observation.recover()?;
    if policy != bootstrap.board_policy
        || old != bootstrap.old_observation
        || new != bootstrap.new_observation
    {
        return Err("persisted ChangeRadar bootstrap values failed exact recovery".to_owned());
    }
    let mut radar = ChangeRadarService::new(policy, InMemoryChangeRadarRepository::new());
    match radar.observe(old).map_err(error_text)? {
        ObservationOutcome::BaselineEstablished { .. } => {}
        _ => return Err("persisted ChangeRadar baseline recovery was incoherent".to_owned()),
    }
    let candidate = match radar.observe(new).map_err(error_text)? {
        ObservationOutcome::SemanticChange(candidate) => *candidate,
        _ => return Err("persisted ChangeRadar candidate recovery was incoherent".to_owned()),
    };
    if candidate != bootstrap.candidate
        || candidate.event_id().as_str() != state.binding.candidate_event_id
    {
        return Err("persisted ChangeRadar candidate identity is incoherent".to_owned());
    }
    let mut inner = radar.into_repository();
    if let Some(review) = state.review.as_ref() {
        let publication = state
            .publication
            .as_ref()
            .map(PersistedPublication::recovery_projection)
            .transpose()?;
        let recovered = ChangePublicationRecoveryRecord::try_recover(
            candidate,
            review.recovery_projection()?,
            publication,
        )
        .map_err(error_text)?;
        let (review_commit, publication_commit) = recovered.into_commits();
        inner.apply_review(review_commit).map_err(error_text)?;
        if let Some(publication_commit) = publication_commit {
            inner
                .apply_publication(publication_commit)
                .map_err(error_text)?;
        }
    }
    Ok(inner)
}

fn read_existing(
    path: &Path,
    max_bytes: u64,
) -> Result<Option<Vec<u8>>, ChangePublicationRepositoryError> {
    validate_secure_parent(path)?;
    let metadata = match fs::symlink_metadata(path) {
        Ok(value) => value,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(_) => return Err(ChangePublicationRepositoryError::Unavailable),
    };
    validate_primary_metadata(&metadata, max_bytes)?;
    let mut file = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW)
        .open(path)
        .map_err(|_| ChangePublicationRepositoryError::Unavailable)?;
    let opened = file
        .metadata()
        .map_err(|_| ChangePublicationRepositoryError::Unavailable)?;
    validate_primary_metadata(&opened, max_bytes)?;
    if opened.dev() != metadata.dev() || opened.ino() != metadata.ino() {
        return Err(ChangePublicationRepositoryError::Unavailable);
    }
    let mut bytes = Vec::new();
    Read::by_ref(&mut file)
        .take(max_bytes.saturating_add(1))
        .read_to_end(&mut bytes)
        .map_err(|_| ChangePublicationRepositoryError::Unavailable)?;
    if u64::try_from(bytes.len()).unwrap_or(u64::MAX) > max_bytes {
        return Err(ChangePublicationRepositoryError::Unavailable);
    }
    Ok(Some(bytes))
}

fn validate_existing_destination(
    path: &Path,
    max_bytes: u64,
) -> Result<(), ChangePublicationRepositoryError> {
    match fs::symlink_metadata(path) {
        Ok(metadata) => validate_primary_metadata(&metadata, max_bytes),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(_) => Err(ChangePublicationRepositoryError::Unavailable),
    }
}

fn validate_primary_metadata(
    metadata: &fs::Metadata,
    max_bytes: u64,
) -> Result<(), ChangePublicationRepositoryError> {
    if !metadata.file_type().is_file()
        || metadata.permissions().mode() & 0o777 != 0o600
        || metadata.nlink() != 1
        || metadata.uid() != current_uid()?
        || metadata.len() > max_bytes
    {
        return Err(ChangePublicationRepositoryError::Unavailable);
    }
    Ok(())
}

fn validate_secure_parent(path: &Path) -> Result<(), ChangePublicationRepositoryError> {
    crate::durable_path::ensure_secure_parent(path, false)
        .map_err(|_| ChangePublicationRepositoryError::Unavailable)
}

fn direct_parent(path: &Path) -> Result<&Path, ChangePublicationRepositoryError> {
    path.parent()
        .filter(|value| !value.as_os_str().is_empty())
        .ok_or(ChangePublicationRepositoryError::Unavailable)
}

fn current_uid() -> Result<u32, ChangePublicationRepositoryError> {
    crate::unix_identity::effective_uid().map_err(|_| ChangePublicationRepositoryError::Unavailable)
}

fn unpredictable_temporary(
    parent: &Path,
    destination: &Path,
) -> Result<PathBuf, ChangePublicationRepositoryError> {
    let mut random = [0_u8; 16];
    File::open("/dev/urandom")
        .and_then(|mut file| file.read_exact(&mut random))
        .map_err(|_| ChangePublicationRepositoryError::Unavailable)?;
    let nonce: String = random.iter().map(|byte| format!("{byte:02x}")).collect();
    let name = destination
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or(ChangePublicationRepositoryError::Unavailable)?;
    Ok(parent.join(format!(".{name}.{nonce}.tmp")))
}

fn error_text(error: impl std::fmt::Display) -> String {
    error.to_string()
}

fn repository_error_text(error: ChangePublicationRepositoryError) -> String {
    format!("change publication persistence rejected: {error:?}")
}
