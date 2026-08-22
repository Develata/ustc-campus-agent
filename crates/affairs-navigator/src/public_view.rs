//! Public projection types of the M71 `affairs.get` result (M71-v8n §9.2).
//!
//! These are canonical domain carriers with NO Serde (§11.1). M10 owns the
//! wire DTO and converts exactly once through the conversion-ready accessors.
//! The public projection exposes `source_id` and review times only — never raw
//! revision IDs, digests, actor references, or journal bytes. Constructors are
//! `pub(crate)` so only the M71 application service / projection algorithm can
//! build a view; external code reads through the public accessors.

use time::OffsetDateTime;

use crate::artifact::{Contact, Deadline, EntryPoint, ProcedureStep};
use crate::evidence::{
    AffairsAuthority, AuthoritySubject, ConflictKind, UncertaintyState, ValidityHorizon,
    conflict_description,
};
use crate::value::{
    ArtifactId, AudienceTag, BoardId, BoardPolicyVersion, EffectiveInterval, ProcedureId, SourceId,
    Title,
};

/// Which lookup path produced the result. v0 emits only `ExactId`;
/// `StructuredSearch` and `Fallback` are deferred but the enum is closed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LookupPath {
    ExactId,
    StructuredSearch,
    Fallback,
}

/// Public conflict state projected onto a `Found` view. `Unresolved` is
/// reachable only on the `Conflict` outcome path (the view is not built when
/// the ladder routes to `Conflict`); a `Found` view always carries `Resolved`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ConflictState {
    Resolved,
    Unresolved { detail: ConflictDetail },
}

/// Safe conflict detail. `description` is a closed `&'static str` selected from
/// `conflict_description`; it never echoes rejected input. `evidence_refs` are
/// safe peer artifact references, `0..=16`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ConflictDetail {
    conflict_kind: ConflictKind,
    description: &'static str,
    evidence_refs: Vec<ArtifactId>,
}

impl ConflictDetail {
    /// Builds one conflict detail. `evidence_refs` MUST be `0..=16`; the
    /// description is fixed by the conflict kind.
    pub(crate) fn new(conflict_kind: ConflictKind, evidence_refs: Vec<ArtifactId>) -> Self {
        Self {
            conflict_kind,
            description: conflict_description(conflict_kind),
            evidence_refs,
        }
    }

    #[must_use]
    pub const fn conflict_kind(&self) -> ConflictKind {
        self.conflict_kind
    }

    #[must_use]
    pub const fn description(&self) -> &'static str {
        self.description
    }

    #[must_use]
    pub fn evidence_refs(&self) -> &[ArtifactId] {
        &self.evidence_refs
    }
}

/// Projection completeness metadata. MANDATORY on every `PublicEvidenceView`
/// (R-M71-8): silently dropping it is the only prohibition. `Found` carrying
/// `Truncated` is valid.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ProjectionMetadata {
    Complete,
    Truncated {
        omitted_count: u8,
        selection_rule_version: u8,
    },
}

pub(crate) const SELECTION_RULE_VERSION: u8 = 2;

/// Freshness computed from `evidence.last_verified_at` and the board policy
/// bounds (D3). `Stale` does NOT silently substitute an unreviewed candidate.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Freshness {
    Fresh,
    Stale {
        last_verified_at: OffsetDateTime,
        max_fresh_age_seconds: u32,
        max_presentable_age_seconds: u32,
    },
}

/// Who supplied the `as_of` cutoff.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CutoffSource {
    CallerProvided,
    SystemNow,
}

/// Safe cutoff metadata returned with `NotYetKnown`. Carries only the cutoff
/// source; `as_of` is a separate field on the outcome.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CutoffMetadata {
    cutoff_source: CutoffSource,
}

impl CutoffMetadata {
    pub(crate) const fn new(cutoff_source: CutoffSource) -> Self {
        Self { cutoff_source }
    }

    #[must_use]
    pub const fn cutoff_source(&self) -> CutoffSource {
        self.cutoff_source
    }
}

/// Safe public evidence assessment view — one per selected GROUP. Exposes
/// `source_id` and review/verification times only. NOT included: `revision_id`,
/// `raw_digest`, `normalized_digest`, `assessed_by`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PublicEvidenceAssessmentView {
    authority: AffairsAuthority,
    subject: AuthoritySubject,
    source_id: SourceId,
    reviewed_at: OffsetDateTime,
    last_verified_at: OffsetDateTime,
}

impl PublicEvidenceAssessmentView {
    pub(crate) fn new(
        authority: AffairsAuthority,
        subject: AuthoritySubject,
        source_id: SourceId,
        reviewed_at: OffsetDateTime,
        last_verified_at: OffsetDateTime,
    ) -> Self {
        Self {
            authority,
            subject,
            source_id,
            reviewed_at,
            last_verified_at,
        }
    }

    #[must_use]
    pub const fn authority(&self) -> AffairsAuthority {
        self.authority
    }

    #[must_use]
    pub const fn subject(&self) -> AuthoritySubject {
        self.subject
    }

    #[must_use]
    pub fn source_id(&self) -> &SourceId {
        &self.source_id
    }

    #[must_use]
    pub fn reviewed_at(&self) -> OffsetDateTime {
        self.reviewed_at
    }

    #[must_use]
    pub fn last_verified_at(&self) -> OffsetDateTime {
        self.last_verified_at
    }
}

/// Safe public prerequisite view. Distinct from canonical `Prerequisite`:
/// `m60_revision_ref` is redacted and replaced by `source_subject`, which is
/// `Some` iff the referenced member's GROUP is selected (R-M71-7).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PublicPrerequisiteView {
    condition: String,
    source_subject: Option<AuthoritySubject>,
}

impl PublicPrerequisiteView {
    pub(crate) fn new(condition: String, source_subject: Option<AuthoritySubject>) -> Self {
        Self {
            condition,
            source_subject,
        }
    }

    #[must_use]
    pub fn condition(&self) -> &str {
        &self.condition
    }

    #[must_use]
    pub const fn source_subject(&self) -> Option<AuthoritySubject> {
        self.source_subject
    }
}

/// Safe public evidence view. `evidence_assessments` are GROUP representatives
/// (`1..=8`); `projection` is MANDATORY.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PublicEvidenceView {
    valid_interval: ValidityHorizon,
    observed_at: OffsetDateTime,
    known_at: OffsetDateTime,
    reviewed_at: OffsetDateTime,
    last_verified_at: OffsetDateTime,
    evidence_assessments: Vec<PublicEvidenceAssessmentView>,
    projection: ProjectionMetadata,
}

impl PublicEvidenceView {
    pub(crate) fn new(
        valid_interval: ValidityHorizon,
        observed_at: OffsetDateTime,
        known_at: OffsetDateTime,
        reviewed_at: OffsetDateTime,
        last_verified_at: OffsetDateTime,
        evidence_assessments: Vec<PublicEvidenceAssessmentView>,
        projection: ProjectionMetadata,
    ) -> Self {
        Self {
            valid_interval,
            observed_at,
            known_at,
            reviewed_at,
            last_verified_at,
            evidence_assessments,
            projection,
        }
    }

    #[must_use]
    pub fn valid_interval(&self) -> &ValidityHorizon {
        &self.valid_interval
    }

    #[must_use]
    pub fn observed_at(&self) -> OffsetDateTime {
        self.observed_at
    }

    #[must_use]
    pub fn known_at(&self) -> OffsetDateTime {
        self.known_at
    }

    #[must_use]
    pub fn reviewed_at(&self) -> OffsetDateTime {
        self.reviewed_at
    }

    #[must_use]
    pub fn last_verified_at(&self) -> OffsetDateTime {
        self.last_verified_at
    }

    #[must_use]
    pub fn evidence_assessments(&self) -> &[PublicEvidenceAssessmentView] {
        &self.evidence_assessments
    }

    #[must_use]
    pub const fn projection(&self) -> ProjectionMetadata {
        self.projection
    }
}

/// Safe public procedure view. Built only on the `Found` path. Internal
/// journal/actor/revision bytes are absent; `lookup_path` is always `ExactId`
/// in v0.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PublicProcedureView {
    procedure_id: ProcedureId,
    artifact_id: ArtifactId,
    title: Title,
    audience_tags: Vec<AudienceTag>,
    board_id: BoardId,
    board_policy_version: BoardPolicyVersion,
    prerequisites: Vec<PublicPrerequisiteView>,
    ordered_steps: Vec<ProcedureStep>,
    deadlines: Vec<Deadline>,
    effective_interval: Option<EffectiveInterval>,
    entry_points: Vec<EntryPoint>,
    contacts: Vec<Contact>,
    evidence: PublicEvidenceView,
    lookup_path: LookupPath,
    conflict_state: ConflictState,
    uncertainty_state: UncertaintyState,
}

impl PublicProcedureView {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        procedure_id: ProcedureId,
        artifact_id: ArtifactId,
        title: Title,
        audience_tags: Vec<AudienceTag>,
        board_id: BoardId,
        board_policy_version: BoardPolicyVersion,
        prerequisites: Vec<PublicPrerequisiteView>,
        ordered_steps: Vec<ProcedureStep>,
        deadlines: Vec<Deadline>,
        effective_interval: Option<EffectiveInterval>,
        entry_points: Vec<EntryPoint>,
        contacts: Vec<Contact>,
        evidence: PublicEvidenceView,
        conflict_state: ConflictState,
        uncertainty_state: UncertaintyState,
    ) -> Self {
        Self {
            procedure_id,
            artifact_id,
            title,
            audience_tags,
            board_id,
            board_policy_version,
            prerequisites,
            ordered_steps,
            deadlines,
            effective_interval,
            entry_points,
            contacts,
            evidence,
            lookup_path: LookupPath::ExactId,
            conflict_state,
            uncertainty_state,
        }
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
    pub fn title(&self) -> &Title {
        &self.title
    }

    #[must_use]
    pub fn audience_tags(&self) -> &[AudienceTag] {
        &self.audience_tags
    }

    #[must_use]
    pub fn board_id(&self) -> &BoardId {
        &self.board_id
    }

    #[must_use]
    pub fn board_policy_version(&self) -> BoardPolicyVersion {
        self.board_policy_version
    }

    #[must_use]
    pub fn prerequisites(&self) -> &[PublicPrerequisiteView] {
        &self.prerequisites
    }

    #[must_use]
    pub fn ordered_steps(&self) -> &[ProcedureStep] {
        &self.ordered_steps
    }

    #[must_use]
    pub fn deadlines(&self) -> &[Deadline] {
        &self.deadlines
    }

    #[must_use]
    pub fn effective_interval(&self) -> Option<&EffectiveInterval> {
        self.effective_interval.as_ref()
    }

    #[must_use]
    pub fn entry_points(&self) -> &[EntryPoint] {
        &self.entry_points
    }

    #[must_use]
    pub fn contacts(&self) -> &[Contact] {
        &self.contacts
    }

    #[must_use]
    pub fn evidence(&self) -> &PublicEvidenceView {
        &self.evidence
    }

    #[must_use]
    pub const fn lookup_path(&self) -> LookupPath {
        self.lookup_path
    }

    #[must_use]
    pub fn conflict_state(&self) -> &ConflictState {
        &self.conflict_state
    }

    #[must_use]
    pub const fn uncertainty_state(&self) -> UncertaintyState {
        self.uncertainty_state
    }
}
