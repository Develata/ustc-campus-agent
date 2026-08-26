#![allow(clippy::unwrap_used)]
#![allow(dead_code)]

//! Shared fixtures for M71 integration tests.

use std::sync::atomic::{AtomicU32, Ordering};

use affairs_navigator::*;
use time::OffsetDateTime;

pub fn t(secs: i64) -> OffsetDateTime {
    OffsetDateTime::from_unix_timestamp(secs).unwrap()
}

pub fn digest(c: char) -> evidence::Sha256 {
    evidence::Sha256::new(format!(
        "sha256:{}",
        std::iter::repeat_n(c, 64).collect::<String>()
    ))
    .unwrap()
}

pub fn rev(
    source: &str,
    idx: usize,
    from: Option<i64>,
    to: Option<i64>,
) -> evidence::M60RevisionRef {
    evidence::M60RevisionRef::new(
        value::SourceId::parse(source).unwrap(),
        format!("rev:{source}:{idx}"),
        t(0),
        None,
        from.map(t),
        to.map(t),
        digest('0'),
        digest('1'),
    )
    .unwrap()
}

pub fn assessment(
    authority: evidence::AffairsAuthority,
    source: &str,
    subject: evidence::AuthoritySubject,
    reviewed: i64,
    verified: i64,
    from: Option<i64>,
    to: Option<i64>,
) -> evidence::AffairsEvidenceAssessment {
    let r = rev(source, 0, from, to);
    let a = evidence::AffairsAuthorityAssessment::new(
        authority,
        subject,
        evidence::AuthorityDerivation::Direct,
        t(0),
        value::ActorRef::parse("actor:fixture").unwrap(),
    );
    evidence::AffairsEvidenceAssessment::new(r, a, t(reviewed), t(verified))
}

#[allow(clippy::too_many_arguments)]
pub fn build_artifact(
    procedure_id: &str,
    known_at: i64,
    last_verified_at: i64,
    conflict_state: evidence::EvidenceConflictState,
    authority_comparison: evidence::AuthorityComparison,
    conflict_kind: Option<evidence::ConflictKind>,
    max_fresh: u32,
    max_presentable: u32,
    assessments: Vec<evidence::AffairsEvidenceAssessment>,
) -> artifact::ProcedureArtifact {
    build_artifact_full(
        procedure_id,
        known_at,
        last_verified_at,
        conflict_state,
        authority_comparison,
        conflict_kind,
        max_fresh,
        max_presentable,
        assessments,
        Vec::new(),
    )
}

#[allow(clippy::too_many_arguments)]
pub fn build_artifact_full(
    procedure_id: &str,
    known_at: i64,
    last_verified_at: i64,
    conflict_state: evidence::EvidenceConflictState,
    authority_comparison: evidence::AuthorityComparison,
    conflict_kind: Option<evidence::ConflictKind>,
    max_fresh: u32,
    max_presentable: u32,
    assessments: Vec<evidence::AffairsEvidenceAssessment>,
    prerequisites: Vec<artifact::Prerequisite>,
) -> artifact::ProcedureArtifact {
    build_artifact_full_with_id(
        "artifact:fixture:v1",
        procedure_id,
        known_at,
        last_verified_at,
        conflict_state,
        authority_comparison,
        conflict_kind,
        max_fresh,
        max_presentable,
        assessments,
        prerequisites,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn build_artifact_full_with_id(
    artifact_id: &str,
    procedure_id: &str,
    known_at: i64,
    last_verified_at: i64,
    conflict_state: evidence::EvidenceConflictState,
    authority_comparison: evidence::AuthorityComparison,
    conflict_kind: Option<evidence::ConflictKind>,
    max_fresh: u32,
    max_presentable: u32,
    assessments: Vec<evidence::AffairsEvidenceAssessment>,
    prerequisites: Vec<artifact::Prerequisite>,
) -> artifact::ProcedureArtifact {
    let evidence = evidence::ProcedureEvidenceContext::new(
        evidence::ValidityHorizon::Unknown,
        t(0),
        t(known_at),
        t(0),
        t(last_verified_at),
        assessments,
        conflict_state,
        authority_comparison,
        evidence::UncertaintyState::None,
        conflict_kind,
        Vec::new(),
    )
    .unwrap();

    let board_policy = artifact::BoardPolicy::new(
        value::BoardId::parse("board:fixture").unwrap(),
        value::BoardPolicyVersion::new(1).unwrap(),
        max_fresh,
        max_presentable,
    )
    .unwrap();

    let step = artifact::ProcedureStep::new(0, value::Instruction::new("Do step 1").unwrap());
    let contact = artifact::Contact::new(
        value::ContactRef::parse("contact:desk").unwrap(),
        value::ContactName::new("Desk").unwrap(),
        value::ContactChannel::new("email").unwrap(),
        value::SourceId::parse("src:desk").unwrap(),
    );
    let entry = artifact::EntryPoint::new(
        value::EntryPointLabel::new("Portal").unwrap(),
        value::Url::new("https://example.com").ok(),
        value::ContactRef::parse("contact:desk").unwrap(),
    );

    artifact::ProcedureArtifact::new(
        value::ArtifactId::parse(artifact_id).unwrap(),
        value::ProcedureId::parse(procedure_id).unwrap(),
        value::Title::new("Fixture procedure").unwrap(),
        vec![value::AudienceTag::new("students").unwrap()],
        board_policy,
        prerequisites,
        vec![step],
        Vec::new(),
        None,
        vec![entry],
        vec![contact],
        evidence,
        t(known_at),
    )
    .unwrap()
}

pub fn seed_current(
    repo: &mut repository::InMemoryAffairsRepository,
    artifact: artifact::ProcedureArtifact,
) {
    let state = artifact::ProcedurePublicationState::current(
        artifact.procedure_id().clone(),
        artifact.artifact_id().clone(),
    );
    repo.seed(artifact, state).unwrap();
}

/// Counting M60 adapter that wraps `M60FixtureAdapter` and tracks call count.
#[allow(dead_code)]
pub struct CountingM60Adapter {
    inner: m60_fixture::M60FixtureAdapter,
    call_count: AtomicU32,
}

impl CountingM60Adapter {
    #[allow(dead_code)]
    pub fn new(verifier_id: &str, version: u16) -> Self {
        Self {
            inner: m60_fixture::M60FixtureAdapter::new(verifier_id, version).unwrap(),
            call_count: AtomicU32::new(0),
        }
    }

    #[allow(dead_code)]
    pub fn store(&mut self, revision_ref: evidence::M60RevisionRef) -> &mut Self {
        self.inner.store(revision_ref);
        self
    }

    #[allow(dead_code)]
    pub fn revoke(&mut self, source_id: &value::SourceId, revision_id: &str) -> &mut Self {
        self.inner.revoke(source_id, revision_id);
        self
    }

    #[allow(dead_code)]
    pub fn require_effective_interval(&mut self, require: bool) -> &mut Self {
        self.inner.require_effective_interval(require);
        self
    }

    #[allow(dead_code)]
    pub fn set_failure_mode(&mut self, mode: Option<m60_port::M60EvidencePortError>) -> &mut Self {
        self.inner.set_failure_mode(mode);
        self
    }

    #[allow(dead_code)]
    pub fn call_count(&self) -> u32 {
        self.call_count.load(Ordering::SeqCst)
    }
}

impl m60_port::M60ProcedureEvidencePort for CountingM60Adapter {
    fn verify_retained(
        &self,
        request: &m60_port::M60RetainedEvidenceRequest,
    ) -> Result<m60_port::M60RetainedEvidenceOutcome, m60_port::M60EvidencePortError> {
        self.call_count.fetch_add(1, Ordering::SeqCst);
        self.inner.verify_retained(request)
    }
}
