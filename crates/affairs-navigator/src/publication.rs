//! Administrator-reviewed procedure publication over an exact M60
//! `DemoReviewed` source revision.
//!
//! This module is a bounded publication foundation. It deliberately does not
//! authenticate administrators, retrieve sources, persist to a production
//! store, or expose a Web/Agent entrypoint. M00/M10 composition owns actor
//! admission; M60 owns source/revision truth; this module owns M71 draft,
//! review and atomic publication semantics.

use std::collections::BTreeMap;

use sha2::{Digest, Sha256 as Sha256Hasher};
use time::OffsetDateTime;
use ustc_campus_agent_core::source_revision::{
    SourceRevision, SourceRevisionHealth, SourceRevisionId, SourceRevisionProvenance,
};

use crate::artifact::{
    BoardPolicy, Contact, Deadline, EntryPoint, Prerequisite, ProcedureArtifact,
    ProcedurePublicationState, ProcedureStep,
};
use crate::evidence::{
    AffairsEvidenceAssessment, AuthorityComparison, AuthorityDerivation, AuthoritySubject,
    ConflictKind, EvidenceConflictState, M60RevisionRef, ProcedureEvidenceContext, Sha256,
    UncertaintyState, ValidityHorizon,
};
use crate::m60_port::{
    M60EvidencePortError, M60EvidenceUnverifiedReason, M60RetainedEvidenceRequest,
    M60VerifiedEvidenceSet,
};
use crate::repository::{AffairsRepository, AffairsRepositoryReadError};
use crate::value::{
    ActorRef, AffairsValueError, ArtifactId, AudienceTag, EffectiveInterval, ProcedureId,
    ProcedurePublicationReceiptId, ProcedureReviewId, SourceId, Title,
};

const DEFAULT_MAX_PROCEDURES: usize = 256;
const DEFAULT_MAX_ARTIFACTS: usize = 4096;

/// One exact, validated procedure draft imported from a current M60-owned
/// `DemoReviewed` revision. It has no administrator approval or publication
/// authority.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcedureDraft {
    draft_digest: Sha256,
    source_revision: SourceRevision,
    procedure_id: ProcedureId,
    title: Title,
    audience_tags: Vec<AudienceTag>,
    board_policy: BoardPolicy,
    prerequisites: Vec<Prerequisite>,
    ordered_steps: Vec<ProcedureStep>,
    deadlines: Vec<Deadline>,
    effective_interval: Option<EffectiveInterval>,
    entry_points: Vec<EntryPoint>,
    contacts: Vec<Contact>,
    evidence: ProcedureEvidenceContext,
}

impl ProcedureDraft {
    /// Imports and validates one structured draft from an exact current
    /// `DemoReviewed` M60 revision.
    ///
    /// The M71 evidence assessments must all reference the exact imported
    /// revision. Unresolved conflict or any non-`None` uncertainty is rejected
    /// before an administrator can approve the draft.
    #[allow(clippy::too_many_arguments)]
    pub fn from_demo_reviewed(
        source_revision: SourceRevision,
        procedure_id: ProcedureId,
        title: Title,
        audience_tags: Vec<AudienceTag>,
        board_policy: BoardPolicy,
        prerequisites: Vec<Prerequisite>,
        ordered_steps: Vec<ProcedureStep>,
        deadlines: Vec<Deadline>,
        effective_interval: Option<EffectiveInterval>,
        entry_points: Vec<EntryPoint>,
        contacts: Vec<Contact>,
        evidence: ProcedureEvidenceContext,
    ) -> Result<Self, ProcedureDraftError> {
        match source_revision.provenance() {
            SourceRevisionProvenance::DemoReviewed { .. } => {}
            _ => return Err(ProcedureDraftError::SourceNotDemoReviewed),
        }
        if evidence.uncertainty_state() != UncertaintyState::None
            || evidence.conflict_state() == EvidenceConflictState::UnresolvedConflict
            || evidence.authority_comparison() == AuthorityComparison::Incomparable
        {
            return Err(ProcedureDraftError::EvidenceNotPublishable);
        }

        let expected_ref = m60_ref_from_source_revision(&source_revision)?;
        if evidence
            .evidence_assessments()
            .iter()
            .any(|assessment| assessment.revision_ref() != &expected_ref)
        {
            return Err(ProcedureDraftError::EvidenceRevisionMismatch);
        }

        // Reuse the canonical artifact validator without exposing a provisional
        // artifact. The checked draft stores the exact same bounded fields.
        ProcedureArtifact::new(
            ArtifactId::parse("artifact:validation:placeholder")?,
            procedure_id.clone(),
            title.clone(),
            audience_tags.clone(),
            board_policy.clone(),
            prerequisites.clone(),
            ordered_steps.clone(),
            deadlines.clone(),
            effective_interval,
            entry_points.clone(),
            contacts.clone(),
            evidence.clone(),
            evidence.known_at(),
        )?;

        let draft_digest = derive_draft_digest(
            &source_revision,
            &procedure_id,
            &title,
            &audience_tags,
            &board_policy,
            &prerequisites,
            &ordered_steps,
            &deadlines,
            effective_interval,
            &entry_points,
            &contacts,
            &evidence,
        );
        Ok(Self {
            draft_digest,
            source_revision,
            procedure_id,
            title,
            audience_tags,
            board_policy,
            prerequisites,
            ordered_steps,
            deadlines,
            effective_interval,
            entry_points,
            contacts,
            evidence,
        })
    }

    #[must_use]
    pub fn draft_digest(&self) -> &Sha256 {
        &self.draft_digest
    }

    #[must_use]
    pub fn source_revision(&self) -> &SourceRevision {
        &self.source_revision
    }

    #[must_use]
    pub fn procedure_id(&self) -> &ProcedureId {
        &self.procedure_id
    }

    #[must_use]
    pub fn evidence(&self) -> &ProcedureEvidenceContext {
        &self.evidence
    }

    #[allow(clippy::type_complexity)]
    fn into_artifact_parts(
        self,
    ) -> (
        ProcedureId,
        Title,
        Vec<AudienceTag>,
        BoardPolicy,
        Vec<Prerequisite>,
        Vec<ProcedureStep>,
        Vec<Deadline>,
        Option<EffectiveInterval>,
        Vec<EntryPoint>,
        Vec<Contact>,
        ProcedureEvidenceContext,
    ) {
        (
            self.procedure_id,
            self.title,
            self.audience_tags,
            self.board_policy,
            self.prerequisites,
            self.ordered_steps,
            self.deadlines,
            self.effective_interval,
            self.entry_points,
            self.contacts,
            self.evidence,
        )
    }
}

/// Why a source-grounded procedure draft could not be admitted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcedureDraftError {
    SourceNotDemoReviewed,
    EvidenceNotPublishable,
    EvidenceRevisionMismatch,
    InvalidValue(AffairsValueError),
}

impl From<AffairsValueError> for ProcedureDraftError {
    fn from(error: AffairsValueError) -> Self {
        Self::InvalidValue(error)
    }
}

/// One administrator approval bound to an exact draft digest. Construction
/// does not authenticate or authorize the actor; M00/M10 must do that before
/// invoking the publication application service.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcedureReviewApproval {
    review_id: ProcedureReviewId,
    draft_digest: Sha256,
    reviewer: ActorRef,
    reviewed_at: OffsetDateTime,
}

impl ProcedureReviewApproval {
    #[must_use]
    pub fn new(draft_digest: Sha256, reviewer: ActorRef, reviewed_at: OffsetDateTime) -> Self {
        let review_id = derive_review_id(&draft_digest, &reviewer, reviewed_at);
        Self {
            review_id,
            draft_digest,
            reviewer,
            reviewed_at,
        }
    }

    #[must_use]
    pub fn review_id(&self) -> &ProcedureReviewId {
        &self.review_id
    }

    #[must_use]
    pub fn draft_digest(&self) -> &Sha256 {
        &self.draft_digest
    }

    #[must_use]
    pub fn reviewer(&self) -> &ActorRef {
        &self.reviewer
    }

    #[must_use]
    pub fn reviewed_at(&self) -> OffsetDateTime {
        self.reviewed_at
    }
}

/// Stable successful publication receipt. It carries safe identity and
/// verification summary, not raw source bytes or M00 authorization state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcedurePublicationReceipt {
    receipt_id: ProcedurePublicationReceiptId,
    procedure_id: ProcedureId,
    artifact_id: ArtifactId,
    draft_digest: Sha256,
    review_id: ProcedureReviewId,
    reviewer: ActorRef,
    expected_publication_revision: Option<u64>,
    publication_revision: u64,
    reviewed_at: OffsetDateTime,
    published_at: OffsetDateTime,
    m60_evidence_set_digest: Sha256,
    m60_revision_count: u8,
}

impl ProcedurePublicationReceipt {
    #[must_use]
    pub fn receipt_id(&self) -> &ProcedurePublicationReceiptId {
        &self.receipt_id
    }

    #[must_use]
    pub fn procedure_id(&self) -> &ProcedureId {
        &self.procedure_id
    }

    #[must_use]
    pub fn artifact_id(&self) -> &ArtifactId {
        &self.artifact_id
    }

    #[must_use]
    pub fn draft_digest(&self) -> &Sha256 {
        &self.draft_digest
    }

    #[must_use]
    pub fn review_id(&self) -> &ProcedureReviewId {
        &self.review_id
    }

    #[must_use]
    pub fn reviewer(&self) -> &ActorRef {
        &self.reviewer
    }

    #[must_use]
    pub const fn expected_publication_revision(&self) -> Option<u64> {
        self.expected_publication_revision
    }

    #[must_use]
    pub const fn publication_revision(&self) -> u64 {
        self.publication_revision
    }

    #[must_use]
    pub fn reviewed_at(&self) -> OffsetDateTime {
        self.reviewed_at
    }

    #[must_use]
    pub fn published_at(&self) -> OffsetDateTime {
        self.published_at
    }

    #[must_use]
    pub fn m60_evidence_set_digest(&self) -> &Sha256 {
        &self.m60_evidence_set_digest
    }

    #[must_use]
    pub const fn m60_revision_count(&self) -> u8 {
        self.m60_revision_count
    }

    fn matches_replay(
        &self,
        draft: &ProcedureDraft,
        approval: &ProcedureReviewApproval,
        expected_publication_revision: Option<u64>,
        publication_revision: u64,
        published_at: OffsetDateTime,
    ) -> bool {
        self.procedure_id() == draft.procedure_id()
            && self.draft_digest() == draft.draft_digest()
            && self.review_id() == approval.review_id()
            && self.reviewer() == approval.reviewer()
            && self.expected_publication_revision() == expected_publication_revision
            && self.publication_revision() == publication_revision
            && self.reviewed_at() == approval.reviewed_at()
            && self.published_at() == published_at
    }
}

/// Repository mutation minted only by [`ProcedurePublicationService`]. Public
/// repository adapters can inspect and persist the command but cannot create
/// one or bypass the service's review/M60 checks.
#[derive(Debug, Clone)]
pub struct ProcedurePublicationCommit {
    expected_publication_revision: Option<u64>,
    artifact: ProcedureArtifact,
    state: ProcedurePublicationState,
    receipt: ProcedurePublicationReceipt,
}

impl ProcedurePublicationCommit {
    fn try_new(
        expected_publication_revision: Option<u64>,
        artifact: ProcedureArtifact,
        state: ProcedurePublicationState,
        receipt: ProcedurePublicationReceipt,
    ) -> Result<Self, ProcedurePublicationRepositoryError> {
        let next = expected_publication_revision
            .unwrap_or(0)
            .checked_add(1)
            .ok_or(ProcedurePublicationRepositoryError::RevisionOverflow)?;
        if state.publication_revision() != next
            || receipt.expected_publication_revision() != expected_publication_revision
            || receipt.publication_revision() != next
            || artifact.procedure_id() != state.procedure_id()
            || artifact.procedure_id() != receipt.procedure_id()
            || state.current_artifact_id() != Some(artifact.artifact_id())
            || artifact.artifact_id() != receipt.artifact_id()
        {
            return Err(ProcedurePublicationRepositoryError::InvalidCommit);
        }
        Ok(Self {
            expected_publication_revision,
            artifact,
            state,
            receipt,
        })
    }

    #[must_use]
    pub const fn expected_publication_revision(&self) -> Option<u64> {
        self.expected_publication_revision
    }

    #[must_use]
    pub fn artifact(&self) -> &ProcedureArtifact {
        &self.artifact
    }

    #[must_use]
    pub fn state(&self) -> &ProcedurePublicationState {
        &self.state
    }

    #[must_use]
    pub fn receipt(&self) -> &ProcedurePublicationReceipt {
        &self.receipt
    }

    #[must_use]
    pub fn into_parts(
        self,
    ) -> (
        Option<u64>,
        ProcedureArtifact,
        ProcedurePublicationState,
        ProcedurePublicationReceipt,
    ) {
        (
            self.expected_publication_revision,
            self.artifact,
            self.state,
            self.receipt,
        )
    }
}

/// Sealed root for recovering publications belonging to one exact reviewed
/// draft. It can only be minted from a sealed service/recovery receipt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcedurePublicationRecoveryAnchor {
    source_revision_id: SourceRevisionId,
    draft_digest: Sha256,
    reviewer: ActorRef,
    reviewed_at: OffsetDateTime,
    published_at: OffsetDateTime,
    m60_evidence_set_digest: Sha256,
    m60_revision_count: u8,
}

impl ProcedurePublicationRecoveryAnchor {
    pub fn from_receipt(
        draft: &ProcedureDraft,
        receipt: &ProcedurePublicationReceipt,
    ) -> Result<Self, ProcedurePublicationRepositoryError> {
        let anchor = Self {
            source_revision_id: draft.source_revision().revision_id().clone(),
            draft_digest: receipt.draft_digest().clone(),
            reviewer: receipt.reviewer().clone(),
            reviewed_at: receipt.reviewed_at(),
            published_at: receipt.published_at(),
            m60_evidence_set_digest: receipt.m60_evidence_set_digest().clone(),
            m60_revision_count: receipt.m60_revision_count(),
        };
        let (_, reconstructed) = ProcedurePublicationRecoveryRecord::try_recover(
            draft,
            &anchor,
            draft.source_revision().revision_id().clone(),
            receipt.draft_digest().clone(),
            receipt.review_id().clone(),
            receipt.reviewer().clone(),
            receipt.reviewed_at(),
            receipt.receipt_id().clone(),
            receipt.artifact_id().clone(),
            receipt.expected_publication_revision(),
            receipt.publication_revision(),
            receipt.published_at(),
            receipt.m60_evidence_set_digest().clone(),
            receipt.m60_revision_count(),
        )?;
        if reconstructed.receipt() != receipt {
            return Err(ProcedurePublicationRepositoryError::StoredPublicationCorrupted);
        }
        Ok(anchor)
    }
}

/// Checked durable representation of one already-committed publication.
///
/// This carrier contains no M00 authority and cannot publish arbitrary state.
/// Recovery always rebinds it to one exact checked [`ProcedureDraft`] and
/// recomputes every deterministic review, receipt, artifact and revision
/// identity before returning a sealed [`ProcedurePublicationCommit`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcedurePublicationRecoveryRecord {
    source_revision_id: SourceRevisionId,
    draft_digest: Sha256,
    review_id: ProcedureReviewId,
    reviewer: ActorRef,
    reviewed_at: OffsetDateTime,
    receipt_id: ProcedurePublicationReceiptId,
    artifact_id: ArtifactId,
    expected_publication_revision: Option<u64>,
    publication_revision: u64,
    published_at: OffsetDateTime,
    m60_evidence_set_digest: Sha256,
    m60_revision_count: u8,
}

impl ProcedurePublicationRecoveryRecord {
    /// Projects one service-minted commit into a checked durable record.
    ///
    /// The supplied draft must reconstruct the exact artifact, state and
    /// receipt already sealed in `commit`; otherwise no record is returned.
    pub fn from_commit(
        commit: &ProcedurePublicationCommit,
        draft: &ProcedureDraft,
    ) -> Result<Self, ProcedurePublicationRepositoryError> {
        let receipt = commit.receipt();
        let anchor = ProcedurePublicationRecoveryAnchor::from_receipt(draft, receipt)?;
        let (record, reconstructed) = Self::try_recover(
            draft,
            &anchor,
            draft.source_revision().revision_id().clone(),
            receipt.draft_digest().clone(),
            receipt.review_id().clone(),
            receipt.reviewer().clone(),
            receipt.reviewed_at(),
            receipt.receipt_id().clone(),
            receipt.artifact_id().clone(),
            receipt.expected_publication_revision(),
            receipt.publication_revision(),
            receipt.published_at(),
            receipt.m60_evidence_set_digest().clone(),
            receipt.m60_revision_count(),
        )?;
        if reconstructed.expected_publication_revision() != commit.expected_publication_revision()
            || reconstructed.artifact() != commit.artifact()
            || reconstructed.state() != commit.state()
            || reconstructed.receipt() != commit.receipt()
        {
            return Err(ProcedurePublicationRepositoryError::StoredPublicationCorrupted);
        }
        Ok(record)
    }

    /// Reconstructs a service-equivalent sealed commit from one persisted
    /// record and an exact checked draft, without consulting mutable M60 state.
    #[allow(clippy::too_many_arguments)]
    pub fn try_recover(
        draft: &ProcedureDraft,
        anchor: &ProcedurePublicationRecoveryAnchor,
        source_revision_id: SourceRevisionId,
        draft_digest: Sha256,
        review_id: ProcedureReviewId,
        reviewer: ActorRef,
        reviewed_at: OffsetDateTime,
        receipt_id: ProcedurePublicationReceiptId,
        artifact_id: ArtifactId,
        expected_publication_revision: Option<u64>,
        publication_revision: u64,
        published_at: OffsetDateTime,
        m60_evidence_set_digest: Sha256,
        m60_revision_count: u8,
    ) -> Result<(Self, ProcedurePublicationCommit), ProcedurePublicationRepositoryError> {
        if source_revision_id != anchor.source_revision_id
            || draft_digest != anchor.draft_digest
            || reviewer != anchor.reviewer
            || reviewed_at != anchor.reviewed_at
            || published_at != anchor.published_at
            || m60_evidence_set_digest != anchor.m60_evidence_set_digest
            || m60_revision_count != anchor.m60_revision_count
            || &source_revision_id != draft.source_revision().revision_id()
            || &draft_digest != draft.draft_digest()
            || m60_revision_count == 0
            || usize::from(m60_revision_count) != draft.evidence().evidence_assessments().len()
            || reviewed_at < draft.evidence().known_at()
            || reviewed_at < draft.evidence().last_verified_at()
            || published_at < reviewed_at
        {
            return Err(ProcedurePublicationRepositoryError::StoredPublicationCorrupted);
        }
        let next_revision = expected_publication_revision
            .unwrap_or(0)
            .checked_add(1)
            .ok_or(ProcedurePublicationRepositoryError::StoredPublicationCorrupted)?;
        if publication_revision != next_revision {
            return Err(ProcedurePublicationRepositoryError::StoredPublicationCorrupted);
        }

        let approval =
            ProcedureReviewApproval::new(draft_digest.clone(), reviewer.clone(), reviewed_at);
        if approval.review_id() != &review_id {
            return Err(ProcedurePublicationRepositoryError::StoredPublicationCorrupted);
        }
        let expected_receipt_id = derive_publication_receipt_id(
            &draft_digest,
            &review_id,
            expected_publication_revision,
            publication_revision,
            published_at,
        );
        if expected_receipt_id != receipt_id {
            return Err(ProcedurePublicationRepositoryError::StoredPublicationCorrupted);
        }
        let expected_artifact_id = derive_artifact_id(
            &draft_digest,
            &review_id,
            &m60_evidence_set_digest,
            publication_revision,
            published_at,
        );
        if expected_artifact_id != artifact_id {
            return Err(ProcedurePublicationRepositoryError::StoredPublicationCorrupted);
        }

        let (
            procedure_id,
            title,
            audience_tags,
            board_policy,
            prerequisites,
            ordered_steps,
            deadlines,
            effective_interval,
            entry_points,
            contacts,
            evidence,
        ) = draft.clone().into_artifact_parts();
        let artifact = ProcedureArtifact::new(
            artifact_id.clone(),
            procedure_id.clone(),
            title,
            audience_tags,
            board_policy,
            prerequisites,
            ordered_steps,
            deadlines,
            effective_interval,
            entry_points,
            contacts,
            evidence,
            published_at,
        )
        .map_err(|_| ProcedurePublicationRepositoryError::StoredPublicationCorrupted)?;
        let state = ProcedurePublicationState::published(
            procedure_id.clone(),
            artifact_id.clone(),
            publication_revision,
        );
        let receipt = ProcedurePublicationReceipt {
            receipt_id: receipt_id.clone(),
            procedure_id,
            artifact_id: artifact_id.clone(),
            draft_digest: draft_digest.clone(),
            review_id: review_id.clone(),
            reviewer: reviewer.clone(),
            expected_publication_revision,
            publication_revision,
            reviewed_at,
            published_at,
            m60_evidence_set_digest: m60_evidence_set_digest.clone(),
            m60_revision_count,
        };
        let commit = ProcedurePublicationCommit::try_new(
            expected_publication_revision,
            artifact,
            state,
            receipt,
        )?;
        let record = Self {
            source_revision_id,
            draft_digest,
            review_id,
            reviewer,
            reviewed_at,
            receipt_id,
            artifact_id,
            expected_publication_revision,
            publication_revision,
            published_at,
            m60_evidence_set_digest,
            m60_revision_count,
        };
        Ok((record, commit))
    }

    #[must_use]
    pub fn source_revision_id(&self) -> &SourceRevisionId {
        &self.source_revision_id
    }

    #[must_use]
    pub fn draft_digest(&self) -> &Sha256 {
        &self.draft_digest
    }

    #[must_use]
    pub fn review_id(&self) -> &ProcedureReviewId {
        &self.review_id
    }

    #[must_use]
    pub fn reviewer(&self) -> &ActorRef {
        &self.reviewer
    }

    #[must_use]
    pub fn reviewed_at(&self) -> OffsetDateTime {
        self.reviewed_at
    }

    #[must_use]
    pub fn receipt_id(&self) -> &ProcedurePublicationReceiptId {
        &self.receipt_id
    }

    #[must_use]
    pub fn artifact_id(&self) -> &ArtifactId {
        &self.artifact_id
    }

    #[must_use]
    pub const fn expected_publication_revision(&self) -> Option<u64> {
        self.expected_publication_revision
    }

    #[must_use]
    pub const fn publication_revision(&self) -> u64 {
        self.publication_revision
    }

    #[must_use]
    pub fn published_at(&self) -> OffsetDateTime {
        self.published_at
    }

    #[must_use]
    pub fn m60_evidence_set_digest(&self) -> &Sha256 {
        &self.m60_evidence_set_digest
    }

    #[must_use]
    pub const fn m60_revision_count(&self) -> u8 {
        self.m60_revision_count
    }
}

/// Atomic publication repository port.
pub trait ProcedurePublicationRepository: AffairsRepository + Send + Sync {
    fn publication_revision(&self, procedure_id: &ProcedureId) -> Option<u64>;
    fn find_publication_replay(
        &self,
        receipt_id: &ProcedurePublicationReceiptId,
    ) -> Result<Option<ProcedurePublicationReceipt>, ProcedurePublicationRepositoryError>;
    fn apply_publication(
        &mut self,
        commit: ProcedurePublicationCommit,
    ) -> Result<(), ProcedurePublicationRepositoryError>;
}

/// One M60-owned publication decision made against a single transaction-current
/// source/evidence view. M71 cannot combine a caller-supplied health flag with a
/// separate retained-evidence result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum M60ProcedurePublicationOutcome {
    CurrentVerified(M60VerifiedEvidenceSet),
    SourceNotCurrent(SourceRevisionHealth),
    Unverified(M60EvidenceUnverifiedReason),
}

/// M60 transaction-current publication decision port. Production adapters must
/// derive health and retained-evidence verification from one coherent read.
pub trait M60ProcedurePublicationPort: Send + Sync {
    fn verify_publication(
        &self,
        revision: &SourceRevision,
        request: &M60RetainedEvidenceRequest,
    ) -> Result<M60ProcedurePublicationOutcome, M60EvidencePortError>;
}

/// Stable repository failure taxonomy. No variant echoes caller data.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcedurePublicationRepositoryError {
    InvalidCapacity,
    InvalidCommit,
    RevisionOverflow,
    PublicationConflict,
    ArtifactIdentityConflict,
    ReceiptIdentityConflict,
    StoredPublicationCorrupted,
    ProcedureCapacityExceeded,
    ArtifactCapacityExceeded,
    PersistenceUnavailable,
    PersistenceLimitExceeded,
    FailureInjected,
}

/// Explicitly bounded in-memory implementation used by the publication
/// foundation and integration tests. Durable storage remains an adapter task.
#[derive(Debug)]
pub struct InMemoryPublishedAffairsRepository {
    artifacts: BTreeMap<ArtifactId, ProcedureArtifact>,
    publication_states: BTreeMap<ProcedureId, ProcedurePublicationState>,
    receipts: BTreeMap<ProcedurePublicationReceiptId, ProcedurePublicationReceipt>,
    max_procedures: usize,
    max_artifacts: usize,
    fail_next_publication: bool,
}

impl Default for InMemoryPublishedAffairsRepository {
    fn default() -> Self {
        Self {
            artifacts: BTreeMap::new(),
            publication_states: BTreeMap::new(),
            receipts: BTreeMap::new(),
            max_procedures: DEFAULT_MAX_PROCEDURES,
            max_artifacts: DEFAULT_MAX_ARTIFACTS,
            fail_next_publication: false,
        }
    }
}

impl InMemoryPublishedAffairsRepository {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_limits(
        max_procedures: usize,
        max_artifacts: usize,
    ) -> Result<Self, ProcedurePublicationRepositoryError> {
        if max_procedures == 0 || max_artifacts == 0 {
            return Err(ProcedurePublicationRepositoryError::InvalidCapacity);
        }
        Ok(Self {
            max_procedures,
            max_artifacts,
            ..Self::default()
        })
    }

    pub fn fail_next_publication(&mut self) {
        self.fail_next_publication = true;
    }

    #[must_use]
    pub fn artifact_count(&self) -> usize {
        self.artifacts.len()
    }

    #[must_use]
    pub fn receipt_count(&self) -> usize {
        self.receipts.len()
    }
}

impl AffairsRepository for InMemoryPublishedAffairsRepository {
    fn find_current_artifact(
        &self,
        procedure_id: &ProcedureId,
    ) -> Result<Option<ProcedureArtifact>, AffairsRepositoryReadError> {
        let Some(state) = self.publication_states.get(procedure_id) else {
            return Ok(None);
        };
        let Some(artifact_id) = state.current_artifact_id() else {
            return Ok(None);
        };
        Ok(self.artifacts.get(artifact_id).cloned())
    }

    fn find_publication_state(
        &self,
        procedure_id: &ProcedureId,
    ) -> Result<Option<ProcedurePublicationState>, AffairsRepositoryReadError> {
        Ok(self.publication_states.get(procedure_id).cloned())
    }
}

impl ProcedurePublicationRepository for InMemoryPublishedAffairsRepository {
    fn publication_revision(&self, procedure_id: &ProcedureId) -> Option<u64> {
        self.publication_states
            .get(procedure_id)
            .map(ProcedurePublicationState::publication_revision)
    }

    fn find_publication_replay(
        &self,
        receipt_id: &ProcedurePublicationReceiptId,
    ) -> Result<Option<ProcedurePublicationReceipt>, ProcedurePublicationRepositoryError> {
        let Some(receipt) = self.receipts.get(receipt_id) else {
            return Ok(None);
        };
        let Some(artifact) = self.artifacts.get(receipt.artifact_id()) else {
            return Err(ProcedurePublicationRepositoryError::StoredPublicationCorrupted);
        };
        if artifact.procedure_id() != receipt.procedure_id()
            || artifact.artifact_id() != receipt.artifact_id()
            || artifact.published_at() != receipt.published_at()
        {
            return Err(ProcedurePublicationRepositoryError::StoredPublicationCorrupted);
        }
        Ok(Some(receipt.clone()))
    }

    fn apply_publication(
        &mut self,
        commit: ProcedurePublicationCommit,
    ) -> Result<(), ProcedurePublicationRepositoryError> {
        if let Some(existing) = self.receipts.get(commit.receipt().receipt_id()) {
            let artifact_matches = self
                .artifacts
                .get(commit.artifact().artifact_id())
                .is_some_and(|artifact| artifact == commit.artifact());
            return if existing == commit.receipt() && artifact_matches {
                Ok(())
            } else {
                Err(ProcedurePublicationRepositoryError::ReceiptIdentityConflict)
            };
        }

        let actual_revision = self
            .publication_states
            .get(commit.artifact().procedure_id())
            .map(ProcedurePublicationState::publication_revision);
        if actual_revision != commit.expected_publication_revision() {
            return Err(ProcedurePublicationRepositoryError::PublicationConflict);
        }
        if self
            .artifacts
            .get(commit.artifact().artifact_id())
            .is_some_and(|artifact| artifact != commit.artifact())
        {
            return Err(ProcedurePublicationRepositoryError::ArtifactIdentityConflict);
        }
        if !self
            .publication_states
            .contains_key(commit.artifact().procedure_id())
            && self.publication_states.len() >= self.max_procedures
        {
            return Err(ProcedurePublicationRepositoryError::ProcedureCapacityExceeded);
        }
        if !self.artifacts.contains_key(commit.artifact().artifact_id())
            && self.artifacts.len() >= self.max_artifacts
        {
            return Err(ProcedurePublicationRepositoryError::ArtifactCapacityExceeded);
        }
        if self.fail_next_publication {
            self.fail_next_publication = false;
            return Err(ProcedurePublicationRepositoryError::FailureInjected);
        }

        let (_, artifact, state, receipt) = commit.into_parts();
        self.artifacts
            .insert(artifact.artifact_id().clone(), artifact);
        self.publication_states
            .insert(state.procedure_id().clone(), state);
        self.receipts.insert(receipt.receipt_id().clone(), receipt);
        Ok(())
    }
}

/// Publication application service. It verifies retained M60 evidence at
/// publish time, enforces administrator-review chronology, mints one sealed
/// commit and returns only after atomic repository success.
pub struct ProcedurePublicationService<'a> {
    repository: &'a mut dyn ProcedurePublicationRepository,
    m60: &'a dyn M60ProcedurePublicationPort,
}

impl<'a> ProcedurePublicationService<'a> {
    #[must_use]
    pub fn new(
        repository: &'a mut dyn ProcedurePublicationRepository,
        m60: &'a dyn M60ProcedurePublicationPort,
    ) -> Self {
        Self { repository, m60 }
    }

    pub fn publish(
        &mut self,
        draft: ProcedureDraft,
        approval: ProcedureReviewApproval,
        published_at: OffsetDateTime,
        expected_publication_revision: Option<u64>,
    ) -> Result<ProcedurePublicationReceipt, ProcedurePublicationError> {
        if approval.draft_digest() != draft.draft_digest() {
            return Err(ProcedurePublicationError::ApprovalMismatch);
        }
        if approval.reviewed_at() < draft.evidence().known_at()
            || approval.reviewed_at() < draft.evidence().last_verified_at()
        {
            return Err(ProcedurePublicationError::ReviewBeforeKnown);
        }
        if published_at < approval.reviewed_at() {
            return Err(ProcedurePublicationError::PublishBeforeReview);
        }
        let next_revision = expected_publication_revision
            .unwrap_or(0)
            .checked_add(1)
            .ok_or(ProcedurePublicationError::PublicationRevisionOverflow)?;
        let revision_refs = draft
            .evidence()
            .evidence_assessments()
            .iter()
            .map(|assessment| assessment.revision_ref().clone())
            .collect();
        let request = M60RetainedEvidenceRequest::new(
            draft.procedure_id().clone(),
            published_at,
            revision_refs,
        )?;
        let draft_digest = draft.draft_digest().clone();
        let receipt_id = derive_publication_receipt_id(
            &draft_digest,
            approval.review_id(),
            expected_publication_revision,
            next_revision,
            published_at,
        );

        // A committed command is a permanent idempotency tombstone. Exact
        // replay returns it before consulting mutable source health/revocation;
        // an uncommitted retry still requires a fresh M60 decision.
        if let Some(existing) = self.repository.find_publication_replay(&receipt_id)? {
            return if existing.matches_replay(
                &draft,
                &approval,
                expected_publication_revision,
                next_revision,
                published_at,
            ) {
                Ok(existing)
            } else {
                Err(ProcedurePublicationError::Repository(
                    ProcedurePublicationRepositoryError::ReceiptIdentityConflict,
                ))
            };
        }

        let verified = match self
            .m60
            .verify_publication(draft.source_revision(), &request)
        {
            Ok(M60ProcedurePublicationOutcome::CurrentVerified(verified)) => verified,
            Ok(M60ProcedurePublicationOutcome::SourceNotCurrent(health)) => {
                return Err(ProcedurePublicationError::SourceNotCurrent(health));
            }
            Ok(M60ProcedurePublicationOutcome::Unverified(reason)) => {
                return Err(ProcedurePublicationError::M60Unverified(reason));
            }
            Err(M60EvidencePortError::StoreUnavailable) => {
                return Err(ProcedurePublicationError::M60StoreUnavailable);
            }
            Err(M60EvidencePortError::StoreCorrupted) => {
                return Err(ProcedurePublicationError::M60StoreCorrupted);
            }
        };

        let artifact_id = derive_artifact_id(
            &draft_digest,
            approval.review_id(),
            verified.evidence_set_digest(),
            next_revision,
            published_at,
        );
        let (
            procedure_id,
            title,
            audience_tags,
            board_policy,
            prerequisites,
            ordered_steps,
            deadlines,
            effective_interval,
            entry_points,
            contacts,
            evidence,
        ) = draft.into_artifact_parts();
        let artifact = ProcedureArtifact::new(
            artifact_id.clone(),
            procedure_id.clone(),
            title,
            audience_tags,
            board_policy,
            prerequisites,
            ordered_steps,
            deadlines,
            effective_interval,
            entry_points,
            contacts,
            evidence,
            published_at,
        )?;
        let state = ProcedurePublicationState::published(
            procedure_id.clone(),
            artifact_id.clone(),
            next_revision,
        );
        let receipt = ProcedurePublicationReceipt {
            receipt_id,
            procedure_id,
            artifact_id,
            draft_digest,
            review_id: approval.review_id().clone(),
            reviewer: approval.reviewer().clone(),
            expected_publication_revision,
            publication_revision: next_revision,
            reviewed_at: approval.reviewed_at(),
            published_at,
            m60_evidence_set_digest: verified.evidence_set_digest().clone(),
            m60_revision_count: verified.revision_count(),
        };

        let commit = ProcedurePublicationCommit::try_new(
            expected_publication_revision,
            artifact,
            state,
            receipt.clone(),
        )?;
        self.repository.apply_publication(commit)?;
        Ok(receipt)
    }
}

/// Stable publication failure taxonomy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcedurePublicationError {
    ApprovalMismatch,
    ReviewBeforeKnown,
    PublishBeforeReview,
    PublicationRevisionOverflow,
    SourceNotCurrent(SourceRevisionHealth),
    InvalidValue(AffairsValueError),
    M60Unverified(M60EvidenceUnverifiedReason),
    M60StoreUnavailable,
    M60StoreCorrupted,
    Repository(ProcedurePublicationRepositoryError),
}

impl From<AffairsValueError> for ProcedurePublicationError {
    fn from(error: AffairsValueError) -> Self {
        Self::InvalidValue(error)
    }
}

impl From<ProcedurePublicationRepositoryError> for ProcedurePublicationError {
    fn from(error: ProcedurePublicationRepositoryError) -> Self {
        Self::Repository(error)
    }
}

/// Projects one exact M60-owned [`SourceRevision`] into the bounded retained
/// evidence reference consumed by M71. This conversion is shared by the draft
/// validator and composition adapters so canonical revision identity and time
/// range checks cannot drift.
pub fn m60_ref_from_source_revision(
    revision: &SourceRevision,
) -> Result<M60RevisionRef, AffairsValueError> {
    M60RevisionRef::new(
        SourceId::parse(revision.source_id().as_str())?,
        revision.revision_id().as_str().to_owned(),
        OffsetDateTime::from_unix_timestamp(revision.observed_at().unix_seconds()).map_err(
            |_| {
                crate::value::value_error(
                    "SourceRevision",
                    crate::value::AffairsValueErrorKind::InvalidRange,
                )
            },
        )?,
        revision
            .published_at()
            .map(|time| OffsetDateTime::from_unix_timestamp(time.unix_seconds()))
            .transpose()
            .map_err(|_| {
                crate::value::value_error(
                    "SourceRevision",
                    crate::value::AffairsValueErrorKind::InvalidRange,
                )
            })?,
        revision
            .effective_interval()
            .from()
            .map(|timestamp| OffsetDateTime::from_unix_timestamp(timestamp.unix_seconds()))
            .transpose()
            .map_err(|_| {
                crate::value::value_error(
                    "SourceRevision",
                    crate::value::AffairsValueErrorKind::InvalidRange,
                )
            })?,
        revision
            .effective_interval()
            .to()
            .map(|timestamp| OffsetDateTime::from_unix_timestamp(timestamp.unix_seconds()))
            .transpose()
            .map_err(|_| {
                crate::value::value_error(
                    "SourceRevision",
                    crate::value::AffairsValueErrorKind::InvalidRange,
                )
            })?,
        Sha256::new(revision.raw_sha256().as_str())?,
        Sha256::new(revision.normalized_sha256().as_str())?,
    )
}

fn update_part(hasher: &mut Sha256Hasher, bytes: &[u8]) {
    let length = u64::try_from(bytes.len()).unwrap_or(u64::MAX);
    hasher.update(length.to_be_bytes());
    hasher.update(bytes);
}

fn update_count(hasher: &mut Sha256Hasher, count: usize) {
    update_part(
        hasher,
        &u64::try_from(count).unwrap_or(u64::MAX).to_be_bytes(),
    );
}

fn update_time(hasher: &mut Sha256Hasher, time: OffsetDateTime) {
    update_part(hasher, &time.unix_timestamp_nanos().to_be_bytes());
}

fn update_optional_time(hasher: &mut Sha256Hasher, time: Option<OffsetDateTime>) {
    match time {
        Some(value) => {
            update_part(hasher, &[1]);
            update_time(hasher, value);
        }
        None => update_part(hasher, &[0]),
    }
}

#[allow(clippy::too_many_arguments)]
fn derive_draft_digest(
    source_revision: &SourceRevision,
    procedure_id: &ProcedureId,
    title: &Title,
    audience_tags: &[AudienceTag],
    board_policy: &BoardPolicy,
    prerequisites: &[Prerequisite],
    ordered_steps: &[ProcedureStep],
    deadlines: &[Deadline],
    effective_interval: Option<EffectiveInterval>,
    entry_points: &[EntryPoint],
    contacts: &[Contact],
    evidence: &ProcedureEvidenceContext,
) -> Sha256 {
    let mut hasher = Sha256Hasher::new();
    hasher.update(b"affairs-procedure-draft/v1\0");
    update_source_revision(&mut hasher, source_revision);
    update_part(&mut hasher, procedure_id.as_str().as_bytes());
    update_part(&mut hasher, title.as_str().as_bytes());
    update_count(&mut hasher, audience_tags.len());
    for tag in audience_tags {
        update_part(&mut hasher, tag.as_str().as_bytes());
    }
    update_part(&mut hasher, board_policy.board_id().as_str().as_bytes());
    update_part(
        &mut hasher,
        &board_policy.policy_version().as_u64().to_be_bytes(),
    );
    update_part(
        &mut hasher,
        &board_policy.max_fresh_age_seconds().to_be_bytes(),
    );
    update_part(
        &mut hasher,
        &board_policy.max_presentable_age_seconds().to_be_bytes(),
    );
    update_count(&mut hasher, prerequisites.len());
    for prerequisite in prerequisites {
        update_part(&mut hasher, prerequisite.condition().as_str().as_bytes());
        match prerequisite.m60_revision_ref() {
            Some(reference) => {
                update_part(&mut hasher, &[1]);
                update_m60_ref(&mut hasher, reference);
            }
            None => update_part(&mut hasher, &[0]),
        }
    }
    update_count(&mut hasher, ordered_steps.len());
    for step in ordered_steps {
        update_part(&mut hasher, &step.step_index().to_be_bytes());
        update_part(&mut hasher, step.instruction().as_str().as_bytes());
    }
    update_count(&mut hasher, deadlines.len());
    for deadline in deadlines {
        update_part(&mut hasher, deadline.label().as_str().as_bytes());
        update_part(
            &mut hasher,
            &[match deadline.kind() {
                crate::artifact::DeadlineKind::Hard => 1,
                crate::artifact::DeadlineKind::Soft => 2,
            }],
        );
        update_time(&mut hasher, deadline.at());
    }
    match effective_interval {
        Some(interval) => {
            update_part(&mut hasher, &[1]);
            update_time(&mut hasher, interval.from());
            update_time(&mut hasher, interval.to());
        }
        None => update_part(&mut hasher, &[0]),
    }
    update_count(&mut hasher, entry_points.len());
    for entry in entry_points {
        update_part(&mut hasher, entry.label().as_str().as_bytes());
        match entry.url() {
            Some(url) => {
                update_part(&mut hasher, &[1]);
                update_part(&mut hasher, url.as_str().as_bytes());
            }
            None => update_part(&mut hasher, &[0]),
        }
        update_part(&mut hasher, entry.contact_ref().as_str().as_bytes());
    }
    update_count(&mut hasher, contacts.len());
    for contact in contacts {
        update_part(&mut hasher, contact.role().as_str().as_bytes());
        update_part(&mut hasher, contact.name().as_str().as_bytes());
        update_part(&mut hasher, contact.channel().as_str().as_bytes());
        update_part(&mut hasher, contact.value_ref().as_str().as_bytes());
    }
    update_evidence(&mut hasher, evidence);
    let digest: [u8; 32] = hasher.finalize().into();
    Sha256::from_bytes(digest)
}

fn update_source_revision(hasher: &mut Sha256Hasher, revision: &SourceRevision) {
    update_part(hasher, revision.revision_id().as_str().as_bytes());
    update_part(hasher, revision.source_id().as_str().as_bytes());
    update_part(hasher, revision.source_url().as_str().as_bytes());
    update_part(hasher, revision.raw_snapshot_id().as_str().as_bytes());
    update_part(hasher, revision.raw_sha256().as_str().as_bytes());
    update_part(
        hasher,
        revision.normalized_snapshot_id().as_str().as_bytes(),
    );
    update_part(hasher, revision.normalized_sha256().as_str().as_bytes());
    update_part(hasher, revision.parser_identity().as_str().as_bytes());
    update_part(hasher, &revision.observed_at().unix_seconds().to_be_bytes());
    match revision.published_at() {
        Some(time) => {
            update_part(hasher, &[1]);
            update_part(hasher, &time.unix_seconds().to_be_bytes());
        }
        None => update_part(hasher, &[0]),
    }
    let interval = revision.effective_interval();
    match interval.from() {
        Some(time) => {
            update_part(hasher, &[1]);
            update_part(hasher, &time.unix_seconds().to_be_bytes());
        }
        None => update_part(hasher, &[0]),
    }
    match interval.to() {
        Some(time) => {
            update_part(hasher, &[1]);
            update_part(hasher, &time.unix_seconds().to_be_bytes());
        }
        None => update_part(hasher, &[0]),
    }
    match revision.provenance() {
        SourceRevisionProvenance::DemoReviewed { reviewer, evidence } => {
            update_part(hasher, &[1]);
            update_part(hasher, reviewer.as_str().as_bytes());
            update_part(hasher, evidence.as_str().as_bytes());
        }
        _ => update_part(hasher, &[0]),
    }
}

fn update_m60_ref(hasher: &mut Sha256Hasher, reference: &M60RevisionRef) {
    update_part(hasher, reference.source_id().as_str().as_bytes());
    update_part(hasher, reference.revision_id().as_bytes());
    update_time(hasher, reference.observed_at());
    update_optional_time(hasher, reference.published_at());
    update_optional_time(hasher, reference.effective_from());
    update_optional_time(hasher, reference.effective_to());
    update_part(hasher, reference.raw_digest().as_str().as_bytes());
    update_part(hasher, reference.normalized_digest().as_str().as_bytes());
}

fn update_evidence(hasher: &mut Sha256Hasher, evidence: &ProcedureEvidenceContext) {
    match evidence.valid_interval() {
        ValidityHorizon::KnownInterval {
            effective_from,
            effective_to,
        } => {
            update_part(hasher, &[1]);
            update_time(hasher, *effective_from);
            update_time(hasher, *effective_to);
        }
        ValidityHorizon::KnownPoint { at } => {
            update_part(hasher, &[2]);
            update_time(hasher, *at);
        }
        ValidityHorizon::Unknown => update_part(hasher, &[3]),
    }
    update_time(hasher, evidence.observed_at());
    update_time(hasher, evidence.known_at());
    update_time(hasher, evidence.reviewed_at());
    update_time(hasher, evidence.last_verified_at());
    update_count(hasher, evidence.evidence_assessments().len());
    for assessment in evidence.evidence_assessments() {
        update_assessment(hasher, assessment);
    }
    update_part(
        hasher,
        &[match evidence.conflict_state() {
            EvidenceConflictState::NoKnownConflict => 1,
            EvidenceConflictState::ResolvedByAuthority => 2,
            EvidenceConflictState::EquivalentSources => 3,
            EvidenceConflictState::UnresolvedConflict => 4,
        }],
    );
    update_part(
        hasher,
        &[match evidence.authority_comparison() {
            AuthorityComparison::Higher => 1,
            AuthorityComparison::Lower => 2,
            AuthorityComparison::Equivalent => 3,
            AuthorityComparison::Incomparable => 4,
        }],
    );
    update_part(
        hasher,
        &[match evidence.uncertainty_state() {
            UncertaintyState::None => 1,
            UncertaintyState::Stale => 2,
            UncertaintyState::CannotVerify => 3,
            UncertaintyState::InsufficientEvidence => 4,
        }],
    );
    match evidence.conflict_kind() {
        Some(kind) => {
            update_part(hasher, &[1, conflict_kind_tag(kind)]);
        }
        None => update_part(hasher, &[0]),
    }
    update_count(hasher, evidence.conflict_evidence_refs().len());
    for artifact_id in evidence.conflict_evidence_refs() {
        update_part(hasher, artifact_id.as_str().as_bytes());
    }
}

fn update_assessment(hasher: &mut Sha256Hasher, assessment: &AffairsEvidenceAssessment) {
    update_m60_ref(hasher, assessment.revision_ref());
    let authority = assessment.authority_assessment();
    update_part(hasher, &[authority.authority().tier()]);
    update_part(hasher, &[authority_subject_tag(authority.subject())]);
    update_part(
        hasher,
        &[match authority.derivation() {
            AuthorityDerivation::Direct => 1,
            AuthorityDerivation::Extracted => 2,
            AuthorityDerivation::InferredRejected => 3,
        }],
    );
    update_time(hasher, authority.assessed_at());
    update_part(hasher, authority.assessed_by().as_str().as_bytes());
    update_time(hasher, assessment.reviewed_at());
    update_time(hasher, assessment.last_verified_at());
}

const fn authority_subject_tag(subject: AuthoritySubject) -> u8 {
    match subject {
        AuthoritySubject::ProcedureTitle => 1,
        AuthoritySubject::ProcedureSteps => 2,
        AuthoritySubject::ProcedureDeadlines => 3,
        AuthoritySubject::ProcedureEffectiveInterval => 4,
        AuthoritySubject::ProcedureEntryPoints => 5,
        AuthoritySubject::ProcedureContacts => 6,
        AuthoritySubject::ProcedurePrerequisites => 7,
        AuthoritySubject::ProcedureEvidence => 8,
    }
}

const fn conflict_kind_tag(kind: ConflictKind) -> u8 {
    match kind {
        ConflictKind::DirectContradiction => 1,
        ConflictKind::OverlapIncompatible => 2,
        ConflictKind::AuthorityConflict => 3,
    }
}

fn derive_review_id(
    draft_digest: &Sha256,
    reviewer: &ActorRef,
    reviewed_at: OffsetDateTime,
) -> ProcedureReviewId {
    let mut hasher = Sha256Hasher::new();
    hasher.update(b"affairs-procedure-review/v1\0");
    update_part(&mut hasher, draft_digest.as_str().as_bytes());
    update_part(&mut hasher, reviewer.as_str().as_bytes());
    update_time(&mut hasher, reviewed_at);
    let digest = hasher.finalize();
    ProcedureReviewId::parse(format!("review:sha256:{digest:x}"))
        .expect("derived review identity is canonical")
}

fn derive_artifact_id(
    draft_digest: &Sha256,
    review_id: &ProcedureReviewId,
    evidence_set_digest: &Sha256,
    publication_revision: u64,
    published_at: OffsetDateTime,
) -> ArtifactId {
    let mut hasher = Sha256Hasher::new();
    hasher.update(b"affairs-procedure-artifact/v1\0");
    update_part(&mut hasher, draft_digest.as_str().as_bytes());
    update_part(&mut hasher, review_id.as_str().as_bytes());
    update_part(&mut hasher, evidence_set_digest.as_str().as_bytes());
    update_part(&mut hasher, &publication_revision.to_be_bytes());
    update_time(&mut hasher, published_at);
    let digest = hasher.finalize();
    ArtifactId::parse(format!("artifact:sha256:{digest:x}"))
        .expect("derived artifact identity is canonical")
}

fn derive_publication_receipt_id(
    draft_digest: &Sha256,
    review_id: &ProcedureReviewId,
    expected_publication_revision: Option<u64>,
    publication_revision: u64,
    published_at: OffsetDateTime,
) -> ProcedurePublicationReceiptId {
    let mut hasher = Sha256Hasher::new();
    hasher.update(b"affairs-publication-receipt/v2\0");
    update_part(&mut hasher, draft_digest.as_str().as_bytes());
    update_part(&mut hasher, review_id.as_str().as_bytes());
    match expected_publication_revision {
        Some(value) => {
            update_part(&mut hasher, &[1]);
            update_part(&mut hasher, &value.to_be_bytes());
        }
        None => update_part(&mut hasher, &[0]),
    }
    update_part(&mut hasher, &publication_revision.to_be_bytes());
    update_time(&mut hasher, published_at);
    let digest = hasher.finalize();
    ProcedurePublicationReceiptId::parse(format!("publication:sha256:{digest:x}"))
        .expect("derived publication receipt identity is canonical")
}
