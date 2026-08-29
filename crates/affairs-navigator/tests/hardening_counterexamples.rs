#![allow(clippy::unwrap_used)]

//! Adversarial hardening counterexamples for the M71 candidate. Every test in
//! this file names a defect that survived the original 82-test suite; each is
//! written against the REQUIRED (fail-closed / deterministic) behavior so it
//! fails before the root-cause repair and passes after it.

mod common;

use affairs_navigator::*;
use common::*;

/// Defect 1 (state pairing): `ProcedureEvidenceContext::new` accepted a
/// declared `valid_interval` that disagrees with the intervals derivable from
/// its own evidence assessments. The assessments own the fact
/// (`derive_valid_interval`, M71-v8n §8.2); a disagreeing declared horizon is
/// an illegal pairing and must fail closed.
#[test]
fn declared_valid_interval_must_equal_derived_horizon() {
    let no_interval = assessment(
        evidence::AffairsAuthority::OfficialBulletin,
        "s1",
        evidence::AuthoritySubject::ProcedureTitle,
        100,
        100,
        None,
        None,
    );
    let with_interval = assessment(
        evidence::AffairsAuthority::OfficialBulletin,
        "s1",
        evidence::AuthoritySubject::ProcedureTitle,
        100,
        100,
        Some(10),
        Some(20),
    );

    // Declared KnownInterval with no supporting effective intervals.
    let declared_known = evidence::ProcedureEvidenceContext::new(
        evidence::ValidityHorizon::KnownInterval {
            effective_from: t(10),
            effective_to: t(20),
        },
        t(0),
        t(0),
        t(0),
        t(0),
        vec![no_interval.clone()],
        evidence::EvidenceConflictState::NoKnownConflict,
        evidence::AuthorityComparison::Equivalent,
        evidence::UncertaintyState::None,
        None,
        Vec::new(),
    );
    assert!(
        declared_known.is_err(),
        "declared KnownInterval without supporting intervals must be rejected"
    );

    // Declared Unknown while the assessments derive a KnownInterval.
    let declared_unknown = evidence::ProcedureEvidenceContext::new(
        evidence::ValidityHorizon::Unknown,
        t(0),
        t(0),
        t(0),
        t(0),
        vec![with_interval],
        evidence::EvidenceConflictState::NoKnownConflict,
        evidence::AuthorityComparison::Equivalent,
        evidence::UncertaintyState::None,
        None,
        Vec::new(),
    );
    assert!(
        declared_unknown.is_err(),
        "declared Unknown with derivable KnownInterval must be rejected"
    );

    // An agreeing pair still constructs.
    let agreeing = evidence::ProcedureEvidenceContext::new(
        evidence::derive_valid_interval(std::slice::from_ref(&no_interval)),
        t(0),
        t(0),
        t(0),
        t(0),
        vec![no_interval],
        evidence::EvidenceConflictState::NoKnownConflict,
        evidence::AuthorityComparison::Equivalent,
        evidence::UncertaintyState::None,
        None,
        Vec::new(),
    );
    assert!(agreeing.is_ok());
}

/// Defect 2 (determinism): permuting the order of equivalent evidence
/// assessments produced a different `m60_evidence_set_digest` (the M60 request
/// carried the raw artifact order), so equivalent evidence inputs yielded
/// non-byte-identical receipts. The request must be canonical.
#[test]
fn assessment_permutation_yields_byte_identical_receipt() {
    // Assessments whose revision refs carry DISTINCT digests, so the ordered
    // evidence-set digest can distinguish input order.
    fn assessment_with_digest(
        authority: evidence::AffairsAuthority,
        source: &str,
        subject: evidence::AuthoritySubject,
        digest_char: char,
    ) -> evidence::AffairsEvidenceAssessment {
        let raw = evidence::Sha256::new(format!(
            "sha256:{}",
            std::iter::repeat_n(digest_char, 64).collect::<String>()
        ))
        .unwrap();
        let rev_ref = evidence::M60RevisionRef::new(
            value::SourceId::parse(source).unwrap(),
            format!("rev:{source}"),
            t(0),
            None,
            None,
            None,
            raw.clone(),
            raw,
        )
        .unwrap();
        let auth = evidence::AffairsAuthorityAssessment::new(
            authority,
            subject,
            evidence::AuthorityDerivation::Direct,
            t(0),
            value::ActorRef::parse("actor:fixture").unwrap(),
        );
        evidence::AffairsEvidenceAssessment::new(rev_ref, auth, t(100), t(100))
    }

    fn run_with_order(reversed: bool) -> service::M71AffairsGetReceipt {
        let a1 = assessment_with_digest(
            evidence::AffairsAuthority::OfficialBulletin,
            "s1",
            evidence::AuthoritySubject::ProcedureTitle,
            'a',
        );
        let a2 = assessment_with_digest(
            evidence::AffairsAuthority::DepartmentNotice,
            "s2",
            evidence::AuthoritySubject::ProcedureSteps,
            'b',
        );
        let assessments = if reversed { vec![a2, a1] } else { vec![a1, a2] };
        let mut m60 = m60_fixture::M60FixtureAdapter::new("verifier:fixture", 1).unwrap();
        for assess in &assessments {
            m60.store(assess.revision_ref().clone());
        }
        let artifact = build_artifact(
            "proc:perm",
            50,
            150,
            evidence::EvidenceConflictState::NoKnownConflict,
            evidence::AuthorityComparison::Equivalent,
            None,
            100,
            200,
            assessments,
        );
        let mut repo = repository::InMemoryAffairsRepository::new();
        seed_current(&mut repo, artifact);
        let clock = clock::FixedClock::new(t(200));
        let service = service::AffairsGetService::new(&repo, &m60, &clock);
        let query = outcome::AffairsGetQuery::new(
            value::ProcedureId::parse("proc:perm").unwrap(),
            Some(t(200)),
        );
        service.execute(&query).unwrap()
    }

    let r1 = run_with_order(false);
    let r2 = run_with_order(true);
    assert!(matches!(
        r1.outcome(),
        outcome::GetProcedureOutcome::Found { .. }
    ));
    assert_eq!(
        r1, r2,
        "permuted equivalent evidence must yield identical receipts"
    );
    assert_eq!(format!("{r1:?}"), format!("{r2:?}"));
    assert_eq!(
        r1.evidence_lineage().m60_evidence_set_digest(),
        r2.evidence_lineage().m60_evidence_set_digest()
    );
}

/// Defect 3 (determinism of replayable reads): a caller-provided `as_of` is a
/// replayable deterministic read, but the `NotFound` and unverified
/// `CannotVerify` paths derived their materialization receipt ID from
/// `clock.now()` instead of the effective cutoff, so the same replayable query
/// produced different receipt IDs as the wall clock moved.
#[test]
fn caller_provided_as_of_receipt_is_clock_independent() {
    fn not_found_with_clock(clock_at: i64) -> service::M71AffairsGetReceipt {
        let repo = repository::InMemoryAffairsRepository::new();
        let m60 = m60_fixture::M60FixtureAdapter::new("verifier:fixture", 1).unwrap();
        let clock = clock::FixedClock::new(t(clock_at));
        let service = service::AffairsGetService::new(&repo, &m60, &clock);
        let query = outcome::AffairsGetQuery::new(
            value::ProcedureId::parse("proc:missing").unwrap(),
            Some(t(100)),
        );
        service.execute(&query).unwrap()
    }

    let r1 = not_found_with_clock(200);
    let r2 = not_found_with_clock(999);
    assert_eq!(
        r1, r2,
        "replayable caller-provided as_of must not depend on the wall clock"
    );

    fn unverified_with_clock(clock_at: i64) -> service::M71AffairsGetReceipt {
        let artifact = build_artifact(
            "proc:unverified-clock",
            50,
            150,
            evidence::EvidenceConflictState::NoKnownConflict,
            evidence::AuthorityComparison::Equivalent,
            None,
            100,
            200,
            vec![assessment(
                evidence::AffairsAuthority::OfficialBulletin,
                "s1",
                evidence::AuthoritySubject::ProcedureTitle,
                100,
                100,
                None,
                None,
            )],
        );
        let mut repo = repository::InMemoryAffairsRepository::new();
        seed_current(&mut repo, artifact);
        // Empty M60 store → MissingRevision → Unverified lineage.
        let m60 = m60_fixture::M60FixtureAdapter::new("verifier:fixture", 1).unwrap();
        let clock = clock::FixedClock::new(t(clock_at));
        let service = service::AffairsGetService::new(&repo, &m60, &clock);
        let query = outcome::AffairsGetQuery::new(
            value::ProcedureId::parse("proc:unverified-clock").unwrap(),
            Some(t(100)),
        );
        service.execute(&query).unwrap()
    }

    let u1 = unverified_with_clock(200);
    let u2 = unverified_with_clock(999);
    assert_eq!(
        u1, u2,
        "replayable caller-provided as_of must not depend on the wall clock"
    );
}

/// Defect 4 (bounds/fail-closed): `M60VerificationIdentity::new` documented
/// M60 ID-grammar enforcement for `verifier_id` but only checked emptiness and
/// length, admitting arbitrary text into the public Verified lineage.
#[test]
fn verification_identity_rejects_non_id_grammar_verifier() {
    for bad in [
        "",
        "Verfier With Spaces",
        "-leading-dash",
        "trailing-dash-",
        "bad@char",
    ] {
        let result = m60_port::M60VerificationIdentity::new(bad.to_owned(), t(0), 1);
        assert!(result.is_err(), "verifier_id {bad:?} must be rejected");
    }
    assert!(m60_port::M60VerificationIdentity::new("verifier:fixture".to_owned(), t(0), 1).is_ok());
}

/// Defect 5 (repository truth / fail closed): the service trusted the
/// repository trait to return the current artifact of the queried procedure.
/// An incoherent repository adapter made the service return a `Found` view for
/// a different procedure than the one queried. The service must fail closed
/// with `InternalInconsistent`.
#[test]
fn service_fails_closed_on_incoherent_repository_pairing() {
    struct RogueRepo {
        state: artifact::ProcedurePublicationState,
        artifact: artifact::ProcedureArtifact,
    }
    impl repository::AffairsRepository for RogueRepo {
        fn find_current_artifact(
            &self,
            _procedure_id: &value::ProcedureId,
        ) -> Result<Option<artifact::ProcedureArtifact>, repository::AffairsRepositoryReadError>
        {
            Ok(Some(self.artifact.clone()))
        }
        fn find_publication_state(
            &self,
            _procedure_id: &value::ProcedureId,
        ) -> Result<
            Option<artifact::ProcedurePublicationState>,
            repository::AffairsRepositoryReadError,
        > {
            Ok(Some(self.state.clone()))
        }
    }

    let other_artifact = build_artifact_full_with_id(
        "artifact:other:v1",
        "proc:other",
        50,
        150,
        evidence::EvidenceConflictState::NoKnownConflict,
        evidence::AuthorityComparison::Equivalent,
        None,
        100,
        200,
        vec![assessment(
            evidence::AffairsAuthority::OfficialBulletin,
            "s1",
            evidence::AuthoritySubject::ProcedureTitle,
            100,
            100,
            None,
            None,
        )],
        Vec::new(),
    );
    let rogue = RogueRepo {
        state: artifact::ProcedurePublicationState::current(
            value::ProcedureId::parse("proc:queried").unwrap(),
            value::ArtifactId::parse("artifact:other:v1").unwrap(),
        ),
        artifact: other_artifact,
    };

    let mut m60 = m60_fixture::M60FixtureAdapter::new("verifier:fixture", 1).unwrap();
    m60.store(rev("s1", 0, None, None));
    let clock = clock::FixedClock::new(t(200));
    let service = service::AffairsGetService::new(&rogue, &m60, &clock);
    let query = outcome::AffairsGetQuery::new(
        value::ProcedureId::parse("proc:queried").unwrap(),
        Some(t(200)),
    );
    let result = service.execute(&query);
    assert!(
        matches!(
            result,
            Err(outcome::GetProcedureError::InternalInconsistent)
        ),
        "incoherent repository pairing must fail closed, got {result:?}"
    );
}
