use std::error::Error;
use std::fmt;

use sha2::{Digest, Sha256};
use time::OffsetDateTime;
use ustc_campus_agent_core::identity::UserId;
use ustc_campus_agent_core::source_registry::SourceId;
use ustc_campus_agent_core::source_revision::{
    RevisionSha256, RevisionTimestamp, SourceRevision, SourceRevisionHealth, SourceRevisionId,
    SourceRevisionProvenance,
};

use crate::{
    BoardId, ChangeEventId, InMemoryChangeRadarRepository, SemanticChangeCandidate, update_part,
};

const MAX_FEED_TITLE_BYTES: usize = 128;
const MAX_FEED_AUTHOR_BYTES: usize = 128;
const MAX_FEED_URL_BYTES: usize = 512;

fn digest_id(prefix: &str, domain: &[u8], parts: &[&[u8]]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(domain);
    for part in parts {
        update_part(&mut hasher, part);
    }
    format!("{prefix}{:x}", hasher.finalize())
}

/// Stable identity of one administrator review receipt.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ChangeReviewReceiptId(String);

impl ChangeReviewReceiptId {
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Stable identity of one atomic publication receipt.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ChangePublicationReceiptId(String);

impl ChangePublicationReceiptId {
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Stable Atom entry identity. It is derived only from the semantic event ID.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StableFeedGuid(String);

impl StableFeedGuid {
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Bounded administrator rejection reasons. Free-form source/profile content is
/// deliberately excluded from the durable review taxonomy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChangeRejectionReason {
    InsufficientEvidence,
    IncorrectAffectedScope,
    Superseded,
}

/// One terminal administrator review decision for an exact candidate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChangeReviewDecision {
    Approved,
    Rejected(ChangeRejectionReason),
}

/// Digest-bound administrator review. Constructing this value records a
/// decision; M00 authentication/authorization remains a composition concern.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChangeReviewReceipt {
    receipt_id: ChangeReviewReceiptId,
    event_id: ChangeEventId,
    board_id: BoardId,
    board_policy_revision: u64,
    source_id: SourceId,
    old_revision_id: SourceRevisionId,
    new_revision_id: SourceRevisionId,
    reviewer: UserId,
    reviewed_at: RevisionTimestamp,
    decision: ChangeReviewDecision,
}

impl ChangeReviewReceipt {
    pub fn approve(
        candidate: &SemanticChangeCandidate,
        reviewer: UserId,
        reviewed_at: RevisionTimestamp,
    ) -> Result<Self, ChangePublicationError> {
        Self::new(
            candidate,
            reviewer,
            reviewed_at,
            ChangeReviewDecision::Approved,
        )
    }

    pub fn reject(
        candidate: &SemanticChangeCandidate,
        reviewer: UserId,
        reviewed_at: RevisionTimestamp,
        reason: ChangeRejectionReason,
    ) -> Result<Self, ChangePublicationError> {
        Self::new(
            candidate,
            reviewer,
            reviewed_at,
            ChangeReviewDecision::Rejected(reason),
        )
    }

    fn new(
        candidate: &SemanticChangeCandidate,
        reviewer: UserId,
        reviewed_at: RevisionTimestamp,
        decision: ChangeReviewDecision,
    ) -> Result<Self, ChangePublicationError> {
        validate_candidate_atom_projection(candidate)?;
        validate_atom_timestamp(reviewed_at)?;
        if reviewed_at < candidate.observed_at() {
            return Err(ChangePublicationError::ReviewBeforeObservation);
        }
        let decision_bytes: &[u8] = match decision {
            ChangeReviewDecision::Approved => b"approved",
            ChangeReviewDecision::Rejected(ChangeRejectionReason::InsufficientEvidence) => {
                b"rejected:insufficient-evidence"
            }
            ChangeReviewDecision::Rejected(ChangeRejectionReason::IncorrectAffectedScope) => {
                b"rejected:incorrect-affected-scope"
            }
            ChangeReviewDecision::Rejected(ChangeRejectionReason::Superseded) => {
                b"rejected:superseded"
            }
        };
        let reviewed_bytes = reviewed_at.unix_seconds().to_be_bytes();
        let revision_bytes = candidate.board_policy_revision().to_be_bytes();
        let receipt_id = ChangeReviewReceiptId(digest_id(
            "change-review:",
            b"change-radar-review/v1\0",
            &[
                candidate.event_id().as_str().as_bytes(),
                candidate.board_id().as_str().as_bytes(),
                &revision_bytes,
                candidate.source_id().as_str().as_bytes(),
                candidate.old_revision().revision_id().as_str().as_bytes(),
                candidate.new_revision().revision_id().as_str().as_bytes(),
                reviewer.as_str().as_bytes(),
                &reviewed_bytes,
                decision_bytes,
            ],
        ));
        Ok(Self {
            receipt_id,
            event_id: candidate.event_id().clone(),
            board_id: candidate.board_id().clone(),
            board_policy_revision: candidate.board_policy_revision(),
            source_id: candidate.source_id().clone(),
            old_revision_id: candidate.old_revision().revision_id().clone(),
            new_revision_id: candidate.new_revision().revision_id().clone(),
            reviewer,
            reviewed_at,
            decision,
        })
    }

    #[must_use]
    pub fn receipt_id(&self) -> &ChangeReviewReceiptId {
        &self.receipt_id
    }
    #[must_use]
    pub fn event_id(&self) -> &ChangeEventId {
        &self.event_id
    }
    #[must_use]
    pub fn reviewer(&self) -> &UserId {
        &self.reviewer
    }
    #[must_use]
    pub const fn reviewed_at(&self) -> RevisionTimestamp {
        self.reviewed_at
    }
    #[must_use]
    pub const fn decision(&self) -> ChangeReviewDecision {
        self.decision
    }

    fn matches_candidate(&self, candidate: &SemanticChangeCandidate) -> bool {
        self.event_id == *candidate.event_id()
            && self.board_id == *candidate.board_id()
            && self.board_policy_revision == candidate.board_policy_revision()
            && self.source_id == *candidate.source_id()
            && self.old_revision_id == *candidate.old_revision().revision_id()
            && self.new_revision_id == *candidate.new_revision().revision_id()
    }
}

/// Feed-specific policy. Feed presentation is versioned separately but must be
/// pinned to the same board and semantic-policy revision as the candidate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoardFeedPolicy {
    board_id: BoardId,
    board_policy_revision: u64,
    title: String,
    author_name: String,
    public_base_url: String,
}

impl BoardFeedPolicy {
    pub fn new(
        board_id: BoardId,
        board_policy_revision: u64,
        title: impl Into<String>,
        author_name: impl Into<String>,
        public_base_url: impl Into<String>,
    ) -> Result<Self, ChangePublicationError> {
        let title = title.into();
        let author_name = author_name.into();
        let public_base_url = public_base_url.into();
        if board_policy_revision == 0
            || title.is_empty()
            || title.len() > MAX_FEED_TITLE_BYTES
            || title.trim() != title
            || title.chars().any(char::is_control)
            || author_name.is_empty()
            || author_name.len() > MAX_FEED_AUTHOR_BYTES
            || author_name.trim() != author_name
            || author_name.chars().any(char::is_control)
            || public_base_url.len() > MAX_FEED_URL_BYTES
            || !public_base_url.starts_with("https://")
            || public_base_url.ends_with('/')
            || public_base_url.contains('?')
            || public_base_url.contains('#')
            || public_base_url.chars().any(char::is_whitespace)
        {
            return Err(ChangePublicationError::InvalidFeedPolicy);
        }
        Ok(Self {
            board_id,
            board_policy_revision,
            title,
            author_name,
            public_base_url,
        })
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
    pub fn title(&self) -> &str {
        &self.title
    }
    #[must_use]
    pub fn author_name(&self) -> &str {
        &self.author_name
    }
    #[must_use]
    pub fn public_base_url(&self) -> &str {
        &self.public_base_url
    }
    #[must_use]
    pub fn feed_id(&self) -> String {
        format!(
            "urn:ustc-campus-agent:change-board:{}",
            self.board_id.as_str()
        )
    }

    fn has_same_pinned_identity(&self, other: &Self) -> bool {
        self.board_id == other.board_id && self.board_policy_revision == other.board_policy_revision
    }
}

/// Evidence result for the exact old/new revision pair. Construction alone is
/// non-authoritative: the publication service accepts it only as the result of
/// its composition-selected trusted M60 port and rechecks every identity field.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct M60VerifiedChangeEvidence {
    source_id: SourceId,
    old_revision_id: SourceRevisionId,
    new_revision_id: SourceRevisionId,
    evidence_set_digest: RevisionSha256,
}

impl M60VerifiedChangeEvidence {
    #[must_use]
    pub fn for_revisions(old: &SourceRevision, new: &SourceRevision) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(b"m60-change-evidence/v1\0");
        update_part(&mut hasher, old.source_id().as_str().as_bytes());
        update_part(&mut hasher, old.revision_id().as_str().as_bytes());
        update_part(&mut hasher, old.raw_sha256().as_str().as_bytes());
        update_part(&mut hasher, old.normalized_sha256().as_str().as_bytes());
        update_part(&mut hasher, new.revision_id().as_str().as_bytes());
        update_part(&mut hasher, new.raw_sha256().as_str().as_bytes());
        update_part(&mut hasher, new.normalized_sha256().as_str().as_bytes());
        Self {
            source_id: old.source_id().clone(),
            old_revision_id: old.revision_id().clone(),
            new_revision_id: new.revision_id().clone(),
            evidence_set_digest: RevisionSha256::from_bytes(hasher.finalize().into()),
        }
    }

    #[must_use]
    pub fn evidence_set_digest(&self) -> &RevisionSha256 {
        &self.evidence_set_digest
    }

    fn matches_candidate(&self, candidate: &SemanticChangeCandidate) -> bool {
        self.source_id == *candidate.source_id()
            && self.old_revision_id == *candidate.old_revision().revision_id()
            && self.new_revision_id == *candidate.new_revision().revision_id()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum M60ChangePublicationOutcome {
    CurrentVerified(M60VerifiedChangeEvidence),
    SourceNotCurrent(SourceRevisionHealth),
    EvidenceUnverified,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum M60ChangePublicationPortError {
    StoreUnavailable,
    StoreCorrupted,
}

/// One coherent transaction-current M60 publication decision. Implementations
/// must verify retained old/new evidence and current source health together.
pub trait M60ChangePublicationPort: Send + Sync {
    fn verify_publication(
        &self,
        old_revision: &SourceRevision,
        new_revision: &SourceRevision,
    ) -> Result<M60ChangePublicationOutcome, M60ChangePublicationPortError>;
}

/// Canonical approved and published change. The nested candidate retains exact
/// changed fields, effective/observed time and source provenance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublishedChangeEvent {
    candidate: SemanticChangeCandidate,
    review: ChangeReviewReceipt,
    feed_policy: BoardFeedPolicy,
    verified_evidence: M60VerifiedChangeEvidence,
    published_at: RevisionTimestamp,
    stable_guid: StableFeedGuid,
    receipt_id: ChangePublicationReceiptId,
}

impl PublishedChangeEvent {
    fn new(
        candidate: SemanticChangeCandidate,
        review: ChangeReviewReceipt,
        feed_policy: BoardFeedPolicy,
        verified_evidence: M60VerifiedChangeEvidence,
        published_at: RevisionTimestamp,
    ) -> Result<Self, ChangePublicationError> {
        if !review.matches_candidate(&candidate)
            || review.decision() != ChangeReviewDecision::Approved
        {
            return Err(ChangePublicationError::ReviewMismatch);
        }
        if feed_policy.board_id() != candidate.board_id()
            || feed_policy.board_policy_revision() != candidate.board_policy_revision()
        {
            return Err(ChangePublicationError::FeedPolicyMismatch);
        }
        if !verified_evidence.matches_candidate(&candidate) {
            return Err(ChangePublicationError::M60EvidenceMismatch);
        }
        if published_at < review.reviewed_at() {
            return Err(ChangePublicationError::PublishBeforeReview);
        }
        let stable_guid = StableFeedGuid(format!(
            "urn:ustc-campus-agent:{}",
            candidate.event_id().as_str()
        ));
        let published_bytes = published_at.unix_seconds().to_be_bytes();
        let policy_bytes = feed_policy.board_policy_revision().to_be_bytes();
        let receipt_id = ChangePublicationReceiptId(digest_id(
            "change-publication:",
            b"change-radar-publication/v1\0",
            &[
                candidate.event_id().as_str().as_bytes(),
                review.receipt_id().as_str().as_bytes(),
                feed_policy.board_id().as_str().as_bytes(),
                &policy_bytes,
                feed_policy.title().as_bytes(),
                feed_policy.author_name().as_bytes(),
                feed_policy.public_base_url().as_bytes(),
                verified_evidence.evidence_set_digest().as_str().as_bytes(),
                &published_bytes,
            ],
        ));
        Ok(Self {
            candidate,
            review,
            feed_policy,
            verified_evidence,
            published_at,
            stable_guid,
            receipt_id,
        })
    }

    #[must_use]
    pub fn event_id(&self) -> &ChangeEventId {
        self.candidate.event_id()
    }
    #[must_use]
    pub fn candidate(&self) -> &SemanticChangeCandidate {
        &self.candidate
    }
    #[must_use]
    pub fn review(&self) -> &ChangeReviewReceipt {
        &self.review
    }
    #[must_use]
    pub fn feed_policy(&self) -> &BoardFeedPolicy {
        &self.feed_policy
    }
    #[must_use]
    pub fn evidence_set_digest(&self) -> &RevisionSha256 {
        self.verified_evidence.evidence_set_digest()
    }
    #[must_use]
    pub const fn published_at(&self) -> RevisionTimestamp {
        self.published_at
    }
    #[must_use]
    pub fn stable_guid(&self) -> &StableFeedGuid {
        &self.stable_guid
    }
    #[must_use]
    pub fn receipt_id(&self) -> &ChangePublicationReceiptId {
        &self.receipt_id
    }
}

/// Service-minted review commit. Its private fields prevent repository callers
/// from attaching a decision to a different candidate.
pub struct ChangeReviewCommit {
    candidate: SemanticChangeCandidate,
    review: ChangeReviewReceipt,
}

impl ChangeReviewCommit {
    fn new(
        candidate: SemanticChangeCandidate,
        review: ChangeReviewReceipt,
    ) -> Result<Self, ChangePublicationError> {
        if !review.matches_candidate(&candidate) {
            return Err(ChangePublicationError::ReviewMismatch);
        }
        Ok(Self { candidate, review })
    }

    #[must_use]
    pub fn candidate(&self) -> &SemanticChangeCandidate {
        &self.candidate
    }

    #[must_use]
    pub fn review(&self) -> &ChangeReviewReceipt {
        &self.review
    }
}

/// Service-minted atomic publication commit.
pub struct ChangePublicationCommit {
    publication: PublishedChangeEvent,
}

impl ChangePublicationCommit {
    fn new(publication: PublishedChangeEvent) -> Self {
        Self { publication }
    }

    #[must_use]
    pub fn publication(&self) -> &PublishedChangeEvent {
        &self.publication
    }
}

/// Storage-neutral projection of one persisted review. It carries no decision
/// authority: checked recovery recomputes the receipt from the exact candidate
/// and rejects every identity mismatch before exposing a sealed commit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChangeReviewRecoveryProjection {
    reviewer: UserId,
    reviewed_at: RevisionTimestamp,
    decision: ChangeReviewDecision,
    expected_receipt_id: String,
}

impl ChangeReviewRecoveryProjection {
    #[must_use]
    pub fn new(
        reviewer: UserId,
        reviewed_at: RevisionTimestamp,
        decision: ChangeReviewDecision,
        expected_receipt_id: impl Into<String>,
    ) -> Self {
        Self {
            reviewer,
            reviewed_at,
            decision,
            expected_receipt_id: expected_receipt_id.into(),
        }
    }
}

/// Storage-neutral projection of an optional persisted publication. This value
/// carries no authority by itself: only [`ChangePublicationRecoveryRecord::try_recover`]
/// can turn it into sealed repository commits, and that entry point recomputes
/// every deterministic identity from the exact candidate and review.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChangePublicationRecoveryProjection {
    feed_policy: BoardFeedPolicy,
    evidence_set_digest: RevisionSha256,
    published_at: RevisionTimestamp,
    expected_receipt_id: String,
    expected_stable_guid: String,
}

impl ChangePublicationRecoveryProjection {
    #[must_use]
    pub fn new(
        feed_policy: BoardFeedPolicy,
        evidence_set_digest: RevisionSha256,
        published_at: RevisionTimestamp,
        expected_receipt_id: impl Into<String>,
        expected_stable_guid: impl Into<String>,
    ) -> Self {
        Self {
            feed_policy,
            evidence_set_digest,
            published_at,
            expected_receipt_id: expected_receipt_id.into(),
            expected_stable_guid: expected_stable_guid.into(),
        }
    }
}

/// Checked recovery-only carrier for one exact reviewed candidate and its
/// optional publication. It performs no M60 I/O, authorizes no new review,
/// and exposes only sealed commits after all stored identities have been
/// recomputed and matched byte-for-byte.
pub struct ChangePublicationRecoveryRecord {
    review_commit: ChangeReviewCommit,
    publication_commit: Option<ChangePublicationCommit>,
}

impl ChangePublicationRecoveryRecord {
    /// Reconstructs sealed repository commits from adapter-validated persisted
    /// state. This is the sole M70 recovery entry point; normal publication must
    /// continue through [`ChangePublicationService::record_review`] and
    /// [`ChangePublicationService::publish`].
    pub fn try_recover(
        candidate: SemanticChangeCandidate,
        review: ChangeReviewRecoveryProjection,
        publication: Option<ChangePublicationRecoveryProjection>,
    ) -> Result<Self, ChangePublicationError> {
        validate_candidate_atom_projection(&candidate)?;
        validate_atom_timestamp(review.reviewed_at)?;
        let recovered_review = match review.decision {
            ChangeReviewDecision::Approved => {
                ChangeReviewReceipt::approve(&candidate, review.reviewer, review.reviewed_at)?
            }
            ChangeReviewDecision::Rejected(reason) => ChangeReviewReceipt::reject(
                &candidate,
                review.reviewer,
                review.reviewed_at,
                reason,
            )?,
        };
        if recovered_review.receipt_id().as_str() != review.expected_receipt_id {
            return Err(ChangePublicationError::RecoveryMismatch);
        }
        let review_commit = ChangeReviewCommit::new(candidate.clone(), recovered_review.clone())?;
        let publication_commit = publication
            .map(|projection| {
                let verified = M60VerifiedChangeEvidence::for_revisions(
                    candidate.old_revision(),
                    candidate.new_revision(),
                );
                if verified.evidence_set_digest() != &projection.evidence_set_digest {
                    return Err(ChangePublicationError::RecoveryMismatch);
                }
                let recovered = PublishedChangeEvent::new(
                    candidate,
                    recovered_review,
                    projection.feed_policy,
                    verified,
                    projection.published_at,
                )?;
                if recovered.receipt_id().as_str() != projection.expected_receipt_id
                    || recovered.stable_guid().as_str() != projection.expected_stable_guid
                {
                    return Err(ChangePublicationError::RecoveryMismatch);
                }
                Ok(ChangePublicationCommit::new(recovered))
            })
            .transpose()?;
        Ok(Self {
            review_commit,
            publication_commit,
        })
    }

    #[must_use]
    pub fn into_commits(self) -> (ChangeReviewCommit, Option<ChangePublicationCommit>) {
        (self.review_commit, self.publication_commit)
    }
}

pub trait ChangePublicationRepository {
    fn find_candidate(
        &self,
        event_id: &ChangeEventId,
    ) -> Result<Option<SemanticChangeCandidate>, ChangePublicationRepositoryError>;
    fn find_review(
        &self,
        event_id: &ChangeEventId,
    ) -> Result<Option<ChangeReviewReceipt>, ChangePublicationRepositoryError>;
    fn find_publication(
        &self,
        event_id: &ChangeEventId,
    ) -> Result<Option<PublishedChangeEvent>, ChangePublicationRepositoryError>;
    fn apply_review(
        &mut self,
        commit: ChangeReviewCommit,
    ) -> Result<(), ChangePublicationRepositoryError>;
    fn apply_publication(
        &mut self,
        commit: ChangePublicationCommit,
    ) -> Result<(), ChangePublicationRepositoryError>;
    fn feed_items(
        &self,
        board_id: &BoardId,
    ) -> Result<Vec<PublishedChangeEvent>, ChangePublicationRepositoryError>;
}

impl ChangePublicationRepository for InMemoryChangeRadarRepository {
    fn find_candidate(
        &self,
        event_id: &ChangeEventId,
    ) -> Result<Option<SemanticChangeCandidate>, ChangePublicationRepositoryError> {
        Ok(self.candidates.get(event_id).cloned())
    }

    fn find_review(
        &self,
        event_id: &ChangeEventId,
    ) -> Result<Option<ChangeReviewReceipt>, ChangePublicationRepositoryError> {
        Ok(self.reviews.get(event_id).cloned())
    }

    fn find_publication(
        &self,
        event_id: &ChangeEventId,
    ) -> Result<Option<PublishedChangeEvent>, ChangePublicationRepositoryError> {
        Ok(self.publications.get(event_id).cloned())
    }

    fn apply_review(
        &mut self,
        commit: ChangeReviewCommit,
    ) -> Result<(), ChangePublicationRepositoryError> {
        let ChangeReviewCommit { candidate, review } = commit;
        let Some(stored_candidate) = self.candidates.get(candidate.event_id()) else {
            return Err(ChangePublicationRepositoryError::CandidateNotFound);
        };
        if stored_candidate != &candidate || !review.matches_candidate(&candidate) {
            return Err(ChangePublicationRepositoryError::InvalidCommit);
        }
        if let Some(existing) = self.reviews.get(candidate.event_id()) {
            return if existing == &review {
                Ok(())
            } else {
                Err(ChangePublicationRepositoryError::ReviewConflict)
            };
        }
        if self.reviews.len() >= self.max_candidates {
            return Err(ChangePublicationRepositoryError::ReviewCapacityExceeded);
        }
        if self.fail_next_review {
            self.fail_next_review = false;
            return Err(ChangePublicationRepositoryError::InjectedReviewFailure);
        }
        self.reviews.insert(candidate.event_id().clone(), review);
        Ok(())
    }

    fn apply_publication(
        &mut self,
        commit: ChangePublicationCommit,
    ) -> Result<(), ChangePublicationRepositoryError> {
        let publication = commit.publication;
        let event_id = publication.event_id();
        if let Some(existing) = self.publications.get(event_id) {
            return if existing == &publication {
                Ok(())
            } else {
                Err(ChangePublicationRepositoryError::PublicationConflict)
            };
        }
        let Some(candidate) = self.candidates.get(event_id) else {
            return Err(ChangePublicationRepositoryError::CandidateNotFound);
        };
        let Some(review) = self.reviews.get(event_id) else {
            return Err(ChangePublicationRepositoryError::ReviewNotFound);
        };
        if candidate != publication.candidate() || review != publication.review() {
            return Err(ChangePublicationRepositoryError::InvalidCommit);
        }
        if self
            .feed_guids
            .get(publication.stable_guid())
            .is_some_and(|existing_event| existing_event != event_id)
        {
            return Err(ChangePublicationRepositoryError::FeedGuidConflict);
        }
        if self.publications.len() >= self.max_candidates {
            return Err(ChangePublicationRepositoryError::PublicationCapacityExceeded);
        }
        if self.fail_next_publication {
            self.fail_next_publication = false;
            return Err(ChangePublicationRepositoryError::InjectedPublicationFailure);
        }
        self.feed_guids
            .insert(publication.stable_guid().clone(), event_id.clone());
        self.publications.insert(event_id.clone(), publication);
        Ok(())
    }

    fn feed_items(
        &self,
        board_id: &BoardId,
    ) -> Result<Vec<PublishedChangeEvent>, ChangePublicationRepositoryError> {
        let mut values: Vec<_> = self
            .publications
            .values()
            .filter(|event| event.candidate().board_id() == board_id)
            .cloned()
            .collect();
        values.sort_by(|left, right| {
            right
                .published_at()
                .cmp(&left.published_at())
                .then_with(|| left.event_id().cmp(right.event_id()))
        });
        Ok(values)
    }
}

impl InMemoryChangeRadarRepository {
    pub fn inject_next_review_failure(&mut self) {
        self.fail_next_review = true;
    }

    pub fn inject_next_publication_failure(&mut self) {
        self.fail_next_publication = true;
    }

    #[must_use]
    pub fn review_count(&self) -> usize {
        self.reviews.len()
    }

    #[must_use]
    pub fn publication_count(&self) -> usize {
        self.publications.len()
    }
}

/// Administrator review and exactly-once publication service.
pub struct ChangePublicationService<'a> {
    repository: &'a mut dyn ChangePublicationRepository,
    m60: &'a dyn M60ChangePublicationPort,
    feed_policy: BoardFeedPolicy,
}

impl<'a> ChangePublicationService<'a> {
    #[must_use]
    pub fn new(
        repository: &'a mut dyn ChangePublicationRepository,
        m60: &'a dyn M60ChangePublicationPort,
        feed_policy: BoardFeedPolicy,
    ) -> Self {
        Self {
            repository,
            m60,
            feed_policy,
        }
    }

    pub fn record_review(
        &mut self,
        review: ChangeReviewReceipt,
    ) -> Result<ChangeReviewReceipt, ChangePublicationError> {
        let candidate = self
            .repository
            .find_candidate(review.event_id())
            .map_err(ChangePublicationError::Repository)?
            .ok_or(ChangePublicationError::CandidateNotFound)?;
        let commit = ChangeReviewCommit::new(candidate, review.clone())?;
        self.repository
            .apply_review(commit)
            .map_err(ChangePublicationError::Repository)?;
        Ok(review)
    }

    pub fn publish(
        &mut self,
        event_id: &ChangeEventId,
        published_at: RevisionTimestamp,
    ) -> Result<PublishedChangeEvent, ChangePublicationError> {
        let candidate = self
            .repository
            .find_candidate(event_id)
            .map_err(ChangePublicationError::Repository)?
            .ok_or(ChangePublicationError::CandidateNotFound)?;
        let review = self
            .repository
            .find_review(event_id)
            .map_err(ChangePublicationError::Repository)?
            .ok_or(ChangePublicationError::ReviewNotFound)?;
        if !review.matches_candidate(&candidate) {
            return Err(ChangePublicationError::ReviewMismatch);
        }
        if let ChangeReviewDecision::Rejected(reason) = review.decision() {
            return Err(ChangePublicationError::CandidateRejected(reason));
        }
        if self.feed_policy.board_id() != candidate.board_id()
            || self.feed_policy.board_policy_revision() != candidate.board_policy_revision()
        {
            return Err(ChangePublicationError::FeedPolicyMismatch);
        }
        validate_candidate_atom_projection(&candidate)?;
        validate_atom_timestamp(review.reviewed_at())?;
        validate_atom_timestamp(published_at)?;
        if published_at < review.reviewed_at() {
            return Err(ChangePublicationError::PublishBeforeReview);
        }

        if let Some(existing) = self
            .repository
            .find_publication(event_id)
            .map_err(ChangePublicationError::Repository)?
        {
            return if existing.review() == &review
                && existing.feed_policy() == &self.feed_policy
                && existing.published_at() == published_at
            {
                Ok(existing)
            } else {
                Err(ChangePublicationError::PublicationReplayConflict)
            };
        }

        let verified = match self
            .m60
            .verify_publication(candidate.old_revision(), candidate.new_revision())
        {
            Ok(M60ChangePublicationOutcome::CurrentVerified(verified)) => verified,
            Ok(M60ChangePublicationOutcome::SourceNotCurrent(health)) => {
                return Err(ChangePublicationError::SourceNotCurrent(health));
            }
            Ok(M60ChangePublicationOutcome::EvidenceUnverified) => {
                return Err(ChangePublicationError::M60EvidenceUnverified);
            }
            Err(M60ChangePublicationPortError::StoreUnavailable) => {
                return Err(ChangePublicationError::M60StoreUnavailable);
            }
            Err(M60ChangePublicationPortError::StoreCorrupted) => {
                return Err(ChangePublicationError::M60StoreCorrupted);
            }
        };
        if !verified.matches_candidate(&candidate) {
            return Err(ChangePublicationError::M60EvidenceMismatch);
        }
        let publication = PublishedChangeEvent::new(
            candidate,
            review,
            self.feed_policy.clone(),
            verified,
            published_at,
        )?;
        self.repository
            .apply_publication(ChangePublicationCommit::new(publication.clone()))
            .map_err(ChangePublicationError::Repository)?;
        Ok(publication)
    }

    pub fn atom_feed(&self) -> Result<String, ChangePublicationError> {
        let items = self
            .repository
            .feed_items(self.feed_policy.board_id())
            .map_err(ChangePublicationError::Repository)?;
        render_atom(&self.feed_policy, &items)
    }
}

/// Deterministic Atom 1.0 renderer over canonical published events.
pub fn render_atom(
    policy: &BoardFeedPolicy,
    items: &[PublishedChangeEvent],
) -> Result<String, ChangePublicationError> {
    if items
        .iter()
        .any(|item| !item.feed_policy().has_same_pinned_identity(policy))
    {
        return Err(ChangePublicationError::FeedPolicyMismatch);
    }
    for item in items {
        validate_candidate_atom_projection(item.candidate())?;
        validate_atom_timestamp(item.review().reviewed_at())?;
        validate_atom_timestamp(item.published_at())?;
    }
    let mut ordered_items = items.to_vec();
    ordered_items.sort_by(|left, right| {
        right
            .published_at()
            .cmp(&left.published_at())
            .then_with(|| left.event_id().cmp(right.event_id()))
    });
    let updated = ordered_items
        .iter()
        .map(PublishedChangeEvent::published_at)
        .max()
        .unwrap_or_else(|| RevisionTimestamp::from_unix_seconds(0));
    let mut output = String::new();
    output.push_str("<?xml version=\"1.0\" encoding=\"utf-8\"?>\n");
    output.push_str("<feed xmlns=\"http://www.w3.org/2005/Atom\">\n");
    output.push_str("  <id>");
    output.push_str(&xml_escape(&policy.feed_id()));
    output.push_str("</id>\n  <title>");
    output.push_str(&xml_escape(policy.title()));
    output.push_str("</title>\n  <author>\n    <name>");
    output.push_str(&xml_escape(policy.author_name()));
    output.push_str("</name>\n    <uri>");
    output.push_str(&xml_escape(policy.public_base_url()));
    output.push_str("</uri>\n  </author>\n  <updated>");
    output.push_str(&format_timestamp(updated)?);
    output.push_str("</updated>\n");
    output.push_str("  <link rel=\"self\" href=\"");
    output.push_str(&xml_escape(&format!(
        "{}/feeds/{}.atom",
        policy.public_base_url(),
        policy.board_id().as_str()
    )));
    output.push_str("\"/>\n");

    for item in &ordered_items {
        let candidate = item.candidate();
        output.push_str("  <entry>\n    <id>");
        output.push_str(&xml_escape(item.stable_guid().as_str()));
        output.push_str("</id>\n    <title>");
        let fields = candidate
            .changed_fields()
            .iter()
            .map(|change| change.field().as_str())
            .collect::<Vec<_>>()
            .join(", ");
        output.push_str(&xml_escape(&format!("Changed: {fields}")));
        output.push_str("</title>\n    <updated>");
        output.push_str(&format_timestamp(item.published_at())?);
        output.push_str("</updated>\n    <link href=\"");
        output.push_str(&xml_escape(&format!(
            "{}/changes/{}",
            policy.public_base_url(),
            candidate.event_id().as_str()
        )));
        output.push_str("\"/>\n    <summary>");
        let (old_reviewer, old_review_evidence) =
            demo_reviewed_provenance(candidate.old_revision())?;
        let (new_reviewer, new_review_evidence) =
            demo_reviewed_provenance(candidate.new_revision())?;
        let mut summary = format!(
            "scope={}; source={}; source_url={}; old_revision={}; old_raw_sha256={}; old_normalized_sha256={}; new_revision={}; new_raw_sha256={}; new_normalized_sha256={}; observed_at={}; source_health={}; old_provenance=DemoReviewed; old_source_reviewer={}; old_source_review_evidence={}; new_provenance=DemoReviewed; new_source_reviewer={}; new_source_review_evidence={}; evidence_set={}",
            candidate.affected_scope(),
            candidate.source_id().as_str(),
            candidate.new_revision().source_url().as_str(),
            candidate.old_revision().revision_id().as_str(),
            candidate.old_revision().raw_sha256().as_str(),
            candidate.old_revision().normalized_sha256().as_str(),
            candidate.new_revision().revision_id().as_str(),
            candidate.new_revision().raw_sha256().as_str(),
            candidate.new_revision().normalized_sha256().as_str(),
            format_timestamp(candidate.observed_at())?,
            source_health_label(candidate.health()),
            old_reviewer,
            old_review_evidence,
            new_reviewer,
            new_review_evidence,
            item.evidence_set_digest().as_str(),
        );
        for change in candidate.changed_fields() {
            summary.push_str("; ");
            summary.push_str(change.field().as_str());
            summary.push('=');
            summary.push_str(change.before().map_or("∅", |value| value.as_str()));
            summary.push('→');
            summary.push_str(change.after().map_or("∅", |value| value.as_str()));
        }
        if let Some(from) = candidate.effective_interval().from() {
            summary.push_str(&format!("; effective_from={}", format_timestamp(from)?));
        }
        if let Some(to) = candidate.effective_interval().to() {
            summary.push_str(&format!("; effective_to={}", format_timestamp(to)?));
        }
        output.push_str(&xml_escape(&summary));
        output.push_str("</summary>\n  </entry>\n");
    }
    output.push_str("</feed>\n");
    Ok(output)
}

fn format_timestamp(value: RevisionTimestamp) -> Result<String, ChangePublicationError> {
    let value = atom_datetime(value)?;
    Ok(format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        value.year(),
        u8::from(value.month()),
        value.day(),
        value.hour(),
        value.minute(),
        value.second(),
    ))
}

fn atom_datetime(value: RevisionTimestamp) -> Result<OffsetDateTime, ChangePublicationError> {
    let value = OffsetDateTime::from_unix_timestamp(value.unix_seconds())
        .map_err(|_| ChangePublicationError::TimestampOutOfRange)?;
    if !(0..=9999).contains(&value.year()) {
        return Err(ChangePublicationError::TimestampOutOfRange);
    }
    Ok(value)
}

fn validate_atom_timestamp(value: RevisionTimestamp) -> Result<(), ChangePublicationError> {
    atom_datetime(value).map(|_| ())
}

fn demo_reviewed_provenance(
    revision: &SourceRevision,
) -> Result<(&str, &str), ChangePublicationError> {
    match revision.provenance() {
        SourceRevisionProvenance::DemoReviewed { reviewer, evidence } => {
            Ok((reviewer.as_str(), evidence.as_str()))
        }
        _ => Err(ChangePublicationError::UnsupportedSourceProvenance),
    }
}

fn validate_candidate_atom_projection(
    candidate: &SemanticChangeCandidate,
) -> Result<(), ChangePublicationError> {
    validate_atom_timestamp(candidate.observed_at())?;
    if let Some(from) = candidate.effective_interval().from() {
        validate_atom_timestamp(from)?;
    }
    if let Some(to) = candidate.effective_interval().to() {
        validate_atom_timestamp(to)?;
    }
    demo_reviewed_provenance(candidate.old_revision())?;
    demo_reviewed_provenance(candidate.new_revision())?;
    Ok(())
}

const fn source_health_label(health: SourceRevisionHealth) -> &'static str {
    match health {
        SourceRevisionHealth::Current => "current",
        SourceRevisionHealth::Stale => "stale",
        SourceRevisionHealth::Conflicting => "conflicting",
    }
}

fn xml_escape(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for character in value.chars() {
        match character {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&apos;"),
            other => escaped.push(other),
        }
    }
    escaped
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChangePublicationRepositoryError {
    InvalidCommit,
    CandidateNotFound,
    ReviewNotFound,
    ReviewConflict,
    PublicationConflict,
    FeedGuidConflict,
    ReviewCapacityExceeded,
    PublicationCapacityExceeded,
    InjectedReviewFailure,
    InjectedPublicationFailure,
    Unavailable,
}

impl fmt::Display for ChangePublicationRepositoryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "change publication repository failure: {self:?}")
    }
}
impl Error for ChangePublicationRepositoryError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChangePublicationError {
    InvalidFeedPolicy,
    ReviewBeforeObservation,
    ReviewMismatch,
    ReviewNotFound,
    CandidateNotFound,
    CandidateRejected(ChangeRejectionReason),
    FeedPolicyMismatch,
    PublishBeforeReview,
    PublicationReplayConflict,
    RecoveryMismatch,
    SourceNotCurrent(SourceRevisionHealth),
    M60EvidenceUnverified,
    M60EvidenceMismatch,
    M60StoreUnavailable,
    M60StoreCorrupted,
    TimestampOutOfRange,
    UnsupportedSourceProvenance,
    Repository(ChangePublicationRepositoryError),
}

impl fmt::Display for ChangePublicationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Repository(error) => error.fmt(formatter),
            other => write!(formatter, "change publication rejected: {other:?}"),
        }
    }
}
impl Error for ChangePublicationError {}
