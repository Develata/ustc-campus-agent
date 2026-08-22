//! Canonical procedure content types, the reviewed `ProcedureArtifact`, and
//! the in-memory publication-state shape. The artifact is the canonical
//! carrier; the public projection (`PublicProcedureView`) is built from it by
//! the application service.

use time::OffsetDateTime;

use crate::evidence::{M60RevisionRef, ProcedureEvidenceContext};
use crate::value::{
    AffairsValueError, AffairsValueErrorKind, ArtifactId, AudienceTag, BoardId, BoardPolicyVersion,
    ContactChannel, ContactName, ContactRef, DeadlineLabel, EffectiveInterval, EntryPointLabel,
    Instruction, PrerequisiteCondition, ProcedureId, SourceId, Title, Url, value_error,
};

// ---------------------------------------------------------------------------
// Content value types (M71-v8n §9.2 / M71-H5 bounds).
// ---------------------------------------------------------------------------

/// One prerequisite condition with an optional revision reference that MUST
/// resolve in the parent evidence assessments when present.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Prerequisite {
    condition: PrerequisiteCondition,
    m60_revision_ref: Option<M60RevisionRef>,
}

impl Prerequisite {
    /// Builds one prerequisite. The `condition` is checked by
    /// `PrerequisiteCondition::new`; the `m60_revision_ref` cross-field
    /// resolution against the parent evidence is validated by
    /// `ProcedureArtifact::new`.
    #[must_use]
    pub fn new(condition: PrerequisiteCondition, m60_revision_ref: Option<M60RevisionRef>) -> Self {
        Self {
            condition,
            m60_revision_ref,
        }
    }

    #[must_use]
    pub fn condition(&self) -> &PrerequisiteCondition {
        &self.condition
    }

    /// Returns the revision reference. Internal only; redacted in the public
    /// projection (the public view exposes only `source_subject`).
    #[must_use]
    pub fn m60_revision_ref(&self) -> Option<&M60RevisionRef> {
        self.m60_revision_ref.as_ref()
    }
}

/// One ordered procedure step.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ProcedureStep {
    step_index: u32,
    instruction: Instruction,
}

impl ProcedureStep {
    /// Builds one step from already-checked fields.
    #[must_use]
    pub fn new(step_index: u32, instruction: Instruction) -> Self {
        Self {
            step_index,
            instruction,
        }
    }

    #[must_use]
    pub const fn step_index(&self) -> u32 {
        self.step_index
    }

    #[must_use]
    pub fn instruction(&self) -> &Instruction {
        &self.instruction
    }
}

/// Deadline kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DeadlineKind {
    Hard,
    Soft,
}

/// One deadline.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Deadline {
    label: DeadlineLabel,
    kind: DeadlineKind,
    at: OffsetDateTime,
}

impl Deadline {
    /// Builds one deadline from already-checked fields. The cross-field
    /// invariant `at ∈ [effective_interval.from, effective_interval.to]` when
    /// the parent effective interval is present is validated by
    /// `ProcedureArtifact::new`.
    #[must_use]
    pub fn new(label: DeadlineLabel, kind: DeadlineKind, at: OffsetDateTime) -> Self {
        Self { label, kind, at }
    }

    #[must_use]
    pub fn label(&self) -> &DeadlineLabel {
        &self.label
    }

    #[must_use]
    pub const fn kind(&self) -> DeadlineKind {
        self.kind
    }

    #[must_use]
    pub fn at(&self) -> OffsetDateTime {
        self.at
    }
}

/// One entry point with an optional URL and a contact reference that MUST
/// resolve in the parent contacts.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct EntryPoint {
    label: EntryPointLabel,
    url: Option<Url>,
    contact_ref: ContactRef,
}

impl EntryPoint {
    /// Builds one entry point. The `contact_ref` cross-field resolution is
    /// validated by `ProcedureArtifact::new`.
    #[must_use]
    pub fn new(label: EntryPointLabel, url: Option<Url>, contact_ref: ContactRef) -> Self {
        Self {
            label,
            url,
            contact_ref,
        }
    }

    #[must_use]
    pub fn label(&self) -> &EntryPointLabel {
        &self.label
    }

    #[must_use]
    pub fn url(&self) -> Option<&Url> {
        self.url.as_ref()
    }

    #[must_use]
    pub fn contact_ref(&self) -> &ContactRef {
        &self.contact_ref
    }
}

/// One contact. `role` uses the M71 ID grammar and is the resolution target for
/// `EntryPoint.contact_ref`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Contact {
    role: ContactRef,
    name: ContactName,
    channel: ContactChannel,
    value_ref: SourceId,
}

impl Contact {
    /// Builds one contact from already-checked fields.
    #[must_use]
    pub fn new(
        role: ContactRef,
        name: ContactName,
        channel: ContactChannel,
        value_ref: SourceId,
    ) -> Self {
        Self {
            role,
            name,
            channel,
            value_ref,
        }
    }

    #[must_use]
    pub fn role(&self) -> &ContactRef {
        &self.role
    }

    #[must_use]
    pub fn name(&self) -> &ContactName {
        &self.name
    }

    #[must_use]
    pub fn channel(&self) -> &ContactChannel {
        &self.channel
    }

    #[must_use]
    pub fn value_ref(&self) -> &SourceId {
        &self.value_ref
    }
}

/// One board policy scoping a procedure. Owns the freshness bounds consumed by
/// the lookup ladder; contains no source credentials or model keys.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct BoardPolicy {
    board_id: BoardId,
    policy_version: BoardPolicyVersion,
    max_fresh_age_seconds: u32,
    max_presentable_age_seconds: u32,
}

impl BoardPolicy {
    /// Builds one board policy.
    ///
    /// # Errors
    ///
    /// Returns [`AffairsValueError`] when `max_fresh_age_seconds` is zero,
    /// `max_presentable_age_seconds` is zero, or
    /// `max_fresh_age_seconds > max_presentable_age_seconds`.
    pub fn new(
        board_id: BoardId,
        policy_version: BoardPolicyVersion,
        max_fresh_age_seconds: u32,
        max_presentable_age_seconds: u32,
    ) -> Result<Self, AffairsValueError> {
        if max_fresh_age_seconds == 0 || max_presentable_age_seconds == 0 {
            return Err(value_error("BoardPolicy", AffairsValueErrorKind::Empty));
        }
        if max_fresh_age_seconds > max_presentable_age_seconds {
            return Err(value_error(
                "BoardPolicy",
                AffairsValueErrorKind::InvalidRange,
            ));
        }
        Ok(Self {
            board_id,
            policy_version,
            max_fresh_age_seconds,
            max_presentable_age_seconds,
        })
    }

    #[must_use]
    pub fn board_id(&self) -> &BoardId {
        &self.board_id
    }

    #[must_use]
    pub fn policy_version(&self) -> BoardPolicyVersion {
        self.policy_version
    }

    #[must_use]
    pub const fn max_fresh_age_seconds(&self) -> u32 {
        self.max_fresh_age_seconds
    }

    #[must_use]
    pub const fn max_presentable_age_seconds(&self) -> u32 {
        self.max_presentable_age_seconds
    }
}

// ---------------------------------------------------------------------------
// Collection bounds (M71-v8n §9.2 / M71-H5).
// ---------------------------------------------------------------------------

pub const MIN_AUDIENCE_TAGS: usize = 1;
pub const MAX_AUDIENCE_TAGS: usize = 32;
pub const MAX_PREREQUISITES: usize = 32;
pub const MIN_ORDERED_STEPS: usize = 1;
pub const MAX_ORDERED_STEPS: usize = 64;
pub const MAX_DEADLINES: usize = 16;
pub const MAX_ENTRY_POINTS: usize = 16;
pub const MAX_CONTACTS: usize = 32;

// ---------------------------------------------------------------------------
// `ProcedureArtifact` — the reviewed canonical value.
// ---------------------------------------------------------------------------

/// One reviewed canonical procedure artifact. Carries all public content plus
/// the canonical evidence context. The public projection is built from it by
/// the application service; internal journal/actor/revision bytes never enter
/// the projection.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ProcedureArtifact {
    artifact_id: ArtifactId,
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
    published_at: OffsetDateTime,
}

impl ProcedureArtifact {
    /// Builds one checked artifact, enforcing every collection bound and
    /// cross-field invariant:
    /// - `audience_tags` `1..=32`, `prerequisites` `0..=32`, `ordered_steps`
    ///   `1..=64` with unique `step_index`, `deadlines` `0..=16`,
    ///   `entry_points` `0..=16`, `contacts` `0..=32`;
    /// - each `Deadline.at` within `[from, to]` when `effective_interval` is
    ///   present;
    /// - each `EntryPoint.contact_ref` resolves to a `Contact.role`;
    /// - each `Prerequisite.m60_revision_ref` resolves in `evidence`;
    /// - `evidence.known_at <= published_at` (D4).
    ///
    /// # Errors
    ///
    /// Returns [`AffairsValueError`] naming the failing rule. The artifact is
    /// the canonical carrier; a failed construction never reaches the
    /// repository.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        artifact_id: ArtifactId,
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
        published_at: OffsetDateTime,
    ) -> Result<Self, AffairsValueError> {
        check_bound(
            "ProcedureArtifact",
            "audience_tags",
            audience_tags.len(),
            MIN_AUDIENCE_TAGS,
            MAX_AUDIENCE_TAGS,
        )?;
        check_bound(
            "ProcedureArtifact",
            "prerequisites",
            prerequisites.len(),
            0,
            MAX_PREREQUISITES,
        )?;
        check_bound(
            "ProcedureArtifact",
            "ordered_steps",
            ordered_steps.len(),
            MIN_ORDERED_STEPS,
            MAX_ORDERED_STEPS,
        )?;
        check_bound(
            "ProcedureArtifact",
            "deadlines",
            deadlines.len(),
            0,
            MAX_DEADLINES,
        )?;
        check_bound(
            "ProcedureArtifact",
            "entry_points",
            entry_points.len(),
            0,
            MAX_ENTRY_POINTS,
        )?;
        check_bound(
            "ProcedureArtifact",
            "contacts",
            contacts.len(),
            0,
            MAX_CONTACTS,
        )?;

        let mut seen_step_indexes = std::collections::BTreeSet::new();
        for step in &ordered_steps {
            if !seen_step_indexes.insert(step.step_index()) {
                return Err(value_error(
                    "ProcedureArtifact",
                    AffairsValueErrorKind::InvalidRange,
                ));
            }
        }

        if let Some(interval) = effective_interval.as_ref() {
            for deadline in &deadlines {
                if deadline.at() < interval.from() || deadline.at() > interval.to() {
                    return Err(value_error(
                        "ProcedureArtifact",
                        AffairsValueErrorKind::InvalidRange,
                    ));
                }
            }
        }

        let contact_roles: std::collections::BTreeSet<&ContactRef> =
            contacts.iter().map(Contact::role).collect();
        for entry_point in &entry_points {
            if !contact_roles.contains(entry_point.contact_ref()) {
                return Err(value_error(
                    "ProcedureArtifact",
                    AffairsValueErrorKind::InvalidRange,
                ));
            }
        }

        let evidence_refs: Vec<&M60RevisionRef> = evidence
            .evidence_assessments()
            .iter()
            .map(|a| a.revision_ref())
            .collect();
        for prereq in &prerequisites {
            if let Some(rev) = prereq.m60_revision_ref()
                && !evidence_refs.contains(&rev)
            {
                return Err(value_error(
                    "ProcedureArtifact",
                    AffairsValueErrorKind::InvalidRange,
                ));
            }
        }

        if evidence.known_at() > published_at {
            return Err(value_error(
                "ProcedureArtifact",
                AffairsValueErrorKind::InvalidRange,
            ));
        }

        Ok(Self {
            artifact_id,
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
            published_at,
        })
    }

    #[must_use]
    pub fn artifact_id(&self) -> &ArtifactId {
        &self.artifact_id
    }

    #[must_use]
    pub fn procedure_id(&self) -> &ProcedureId {
        &self.procedure_id
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
    pub fn board_policy(&self) -> &BoardPolicy {
        &self.board_policy
    }

    #[must_use]
    pub fn prerequisites(&self) -> &[Prerequisite] {
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
    pub fn evidence(&self) -> &ProcedureEvidenceContext {
        &self.evidence
    }

    #[must_use]
    pub fn published_at(&self) -> OffsetDateTime {
        self.published_at
    }
}

fn check_bound(
    type_name: &'static str,
    field: &str,
    len: usize,
    min: usize,
    max: usize,
) -> Result<(), AffairsValueError> {
    if len < min {
        return Err(value_error(type_name, AffairsValueErrorKind::Empty));
    }
    if len > max {
        return Err(value_error(
            type_name,
            AffairsValueErrorKind::TooLong { max_bytes: max },
        ));
    }
    let _ = field;
    Ok(())
}

// ---------------------------------------------------------------------------
// `ProcedurePublicationState` — in-memory publication state per procedure.
// ---------------------------------------------------------------------------

/// One procedure's publication state. Owns the at-most-one-Current invariant:
/// `current_artifact_id` is `Some` iff a Current artifact exists; `None` after
/// archive or before first publication.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ProcedurePublicationState {
    procedure_id: ProcedureId,
    current_artifact_id: Option<ArtifactId>,
    archived_at: Option<OffsetDateTime>,
    publication_revision: u64,
}

impl ProcedurePublicationState {
    /// Builds one publication state for a procedure with a Current artifact.
    #[must_use]
    pub fn current(procedure_id: ProcedureId, current_artifact_id: ArtifactId) -> Self {
        Self {
            procedure_id,
            current_artifact_id: Some(current_artifact_id),
            archived_at: None,
            publication_revision: 1,
        }
    }

    /// Builds one publication state for a procedure that has been archived.
    #[must_use]
    pub fn archived(procedure_id: ProcedureId, archived_at: OffsetDateTime) -> Self {
        Self {
            procedure_id,
            current_artifact_id: None,
            archived_at: Some(archived_at),
            publication_revision: 1,
        }
    }

    #[must_use]
    pub fn procedure_id(&self) -> &ProcedureId {
        &self.procedure_id
    }

    #[must_use]
    pub fn current_artifact_id(&self) -> Option<&ArtifactId> {
        self.current_artifact_id.as_ref()
    }

    #[must_use]
    pub fn archived_at(&self) -> Option<OffsetDateTime> {
        self.archived_at
    }

    #[must_use]
    pub const fn publication_revision(&self) -> u64 {
        self.publication_revision
    }
}
