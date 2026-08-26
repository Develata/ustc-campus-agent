#![allow(clippy::unwrap_used)]

//! Exhaustive outcome/lineage pairing table (M71-v8n §4.2). Every allowed pair
//! is constructible and reachable; every mismatched pair is rejected.

mod common;

use affairs_navigator::*;
use common::*;
use time::OffsetDateTime;

fn run(
    repo: &repository::InMemoryAffairsRepository,
    m60: &dyn m60_port::M60ProcedureEvidencePort,
    clock: &dyn clock::AffairsClock,
    procedure_id: &str,
    as_of: Option<OffsetDateTime>,
) -> service::M71AffairsGetReceipt {
    let service = service::AffairsGetService::new(repo, m60, clock);
    let query =
        outcome::AffairsGetQuery::new(value::ProcedureId::parse(procedure_id).unwrap(), as_of);
    service.execute(&query).unwrap()
}

// ---------------------------------------------------------------------------
// Allowed pairs (§4.2)
// ---------------------------------------------------------------------------

#[test]
fn found_pairs_with_verified() {
    let a = assessment(
        evidence::AffairsAuthority::OfficialBulletin,
        "s1",
        evidence::AuthoritySubject::ProcedureTitle,
        100,
        100,
        None,
        None,
    );
    let artifact = build_artifact(
        "proc:found",
        50,
        150,
        evidence::EvidenceConflictState::NoKnownConflict,
        evidence::AuthorityComparison::Equivalent,
        None,
        100,
        200,
        vec![a],
    );
    let mut repo = repository::InMemoryAffairsRepository::new();
    seed_current(&mut repo, artifact);
    let mut m60 = m60_fixture::M60FixtureAdapter::new("verifier:fixture", 1).unwrap();
    m60.store(rev("s1", 0, None, None));
    let clock = clock::FixedClock::new(t(200));
    let receipt = run(&repo, &m60, &clock, "proc:found", Some(t(200)));
    assert!(matches!(
        receipt.outcome(),
        outcome::GetProcedureOutcome::Found { .. }
    ));
    assert!(receipt.evidence_lineage().is_verified());
}

#[test]
fn conflict_pairs_with_verified() {
    let a = assessment(
        evidence::AffairsAuthority::OfficialBulletin,
        "s1",
        evidence::AuthoritySubject::ProcedureTitle,
        100,
        100,
        None,
        None,
    );
    let artifact = build_artifact(
        "proc:conflict",
        50,
        150,
        evidence::EvidenceConflictState::UnresolvedConflict,
        evidence::AuthorityComparison::Incomparable,
        Some(evidence::ConflictKind::DirectContradiction),
        100,
        200,
        vec![a],
    );
    let mut repo = repository::InMemoryAffairsRepository::new();
    seed_current(&mut repo, artifact);
    let mut m60 = m60_fixture::M60FixtureAdapter::new("verifier:fixture", 1).unwrap();
    m60.store(rev("s1", 0, None, None));
    let clock = clock::FixedClock::new(t(200));
    let receipt = run(&repo, &m60, &clock, "proc:conflict", Some(t(200)));
    assert!(matches!(
        receipt.outcome(),
        outcome::GetProcedureOutcome::Conflict { .. }
    ));
    assert!(receipt.evidence_lineage().is_verified());
}

#[test]
fn overflow_pairs_with_verified() {
    let subjects = [
        evidence::AuthoritySubject::ProcedureTitle,
        evidence::AuthoritySubject::ProcedureSteps,
        evidence::AuthoritySubject::ProcedureDeadlines,
        evidence::AuthoritySubject::ProcedureEffectiveInterval,
        evidence::AuthoritySubject::ProcedureEntryPoints,
        evidence::AuthoritySubject::ProcedureContacts,
        evidence::AuthoritySubject::ProcedurePrerequisites,
        evidence::AuthoritySubject::ProcedureEvidence,
    ];
    let assessments: Vec<_> = subjects
        .iter()
        .enumerate()
        .map(|(i, &subject)| {
            assessment(
                evidence::AffairsAuthority::OfficialBulletin,
                &format!("s{i}"),
                subject,
                100,
                100,
                None,
                None,
            )
        })
        .collect();
    let assessments_with_extra = assessments
        .into_iter()
        .chain(std::iter::once(assessment(
            evidence::AffairsAuthority::OfficialBulletin,
            "s8",
            evidence::AuthoritySubject::ProcedureTitle,
            100,
            100,
            None,
            None,
        )))
        .collect();
    let artifact = build_artifact(
        "proc:overflow",
        50,
        150,
        evidence::EvidenceConflictState::NoKnownConflict,
        evidence::AuthorityComparison::Equivalent,
        None,
        100,
        200,
        assessments_with_extra,
    );
    let mut repo = repository::InMemoryAffairsRepository::new();
    seed_current(&mut repo, artifact);
    let mut m60 = m60_fixture::M60FixtureAdapter::new("verifier:fixture", 1).unwrap();
    for i in 0..9 {
        m60.store(rev(&format!("s{i}"), 0, None, None));
    }
    let clock = clock::FixedClock::new(t(200));
    let receipt = run(&repo, &m60, &clock, "proc:overflow", Some(t(200)));
    if let outcome::GetProcedureOutcome::CannotVerify { reason, .. } = receipt.outcome() {
        assert!(matches!(
            reason,
            outcome::CannotVerifyReason::PublicEvidenceProjectionOverflow { mandatory_count: 9 }
        ));
    } else {
        panic!("expected CannotVerify overflow");
    }
    assert!(receipt.evidence_lineage().is_verified());
}

#[test]
fn stale_beyond_policy_pairs_with_verified() {
    let a = assessment(
        evidence::AffairsAuthority::OfficialBulletin,
        "s1",
        evidence::AuthoritySubject::ProcedureTitle,
        100,
        50,
        None,
        None,
    );
    let artifact = build_artifact(
        "proc:stale",
        50,
        50,
        evidence::EvidenceConflictState::NoKnownConflict,
        evidence::AuthorityComparison::Equivalent,
        None,
        10,
        20,
        vec![a],
    );
    let mut repo = repository::InMemoryAffairsRepository::new();
    seed_current(&mut repo, artifact);
    let mut m60 = m60_fixture::M60FixtureAdapter::new("verifier:fixture", 1).unwrap();
    m60.store(rev("s1", 0, None, None));
    let clock = clock::FixedClock::new(t(200));
    let receipt = run(&repo, &m60, &clock, "proc:stale", Some(t(200)));
    if let outcome::GetProcedureOutcome::CannotVerify { reason, .. } = receipt.outcome() {
        assert_eq!(
            *reason,
            outcome::CannotVerifyReason::LastVerifiedStaleBeyondPolicy
        );
    } else {
        panic!("expected CannotVerify");
    }
    assert!(receipt.evidence_lineage().is_verified());
}

#[test]
fn source_revision_unverified_pairs_with_unverified() {
    let a = assessment(
        evidence::AffairsAuthority::OfficialBulletin,
        "s1",
        evidence::AuthoritySubject::ProcedureTitle,
        100,
        100,
        None,
        None,
    );
    let artifact = build_artifact(
        "proc:unverified",
        50,
        150,
        evidence::EvidenceConflictState::NoKnownConflict,
        evidence::AuthorityComparison::Equivalent,
        None,
        100,
        200,
        vec![a],
    );
    let mut repo = repository::InMemoryAffairsRepository::new();
    seed_current(&mut repo, artifact);
    let m60 = m60_fixture::M60FixtureAdapter::new("verifier:fixture", 1).unwrap();
    let clock = clock::FixedClock::new(t(200));
    let receipt = run(&repo, &m60, &clock, "proc:unverified", Some(t(200)));
    if let outcome::GetProcedureOutcome::CannotVerify { reason, .. } = receipt.outcome() {
        assert_eq!(
            *reason,
            outcome::CannotVerifyReason::SourceRevisionUnverified
        );
    } else {
        panic!("expected CannotVerify");
    }
    assert!(receipt.evidence_lineage().is_unverified());
}

#[test]
fn effective_interval_missing_pairs_with_unverified() {
    let a = assessment(
        evidence::AffairsAuthority::OfficialBulletin,
        "s1",
        evidence::AuthoritySubject::ProcedureTitle,
        100,
        100,
        None,
        None,
    );
    let artifact = build_artifact(
        "proc:effmiss",
        50,
        150,
        evidence::EvidenceConflictState::NoKnownConflict,
        evidence::AuthorityComparison::Equivalent,
        None,
        100,
        200,
        vec![a],
    );
    let mut repo = repository::InMemoryAffairsRepository::new();
    seed_current(&mut repo, artifact);
    let mut m60 = m60_fixture::M60FixtureAdapter::new("verifier:fixture", 1).unwrap();
    m60.store(rev("s1", 0, None, None));
    m60.require_effective_interval(true);
    let clock = clock::FixedClock::new(t(200));
    let receipt = run(&repo, &m60, &clock, "proc:effmiss", Some(t(200)));
    if let outcome::GetProcedureOutcome::CannotVerify { reason, .. } = receipt.outcome() {
        assert_eq!(
            *reason,
            outcome::CannotVerifyReason::EffectiveIntervalMissing
        );
    } else {
        panic!("expected CannotVerify");
    }
    assert!(receipt.evidence_lineage().is_unverified());
}

#[test]
fn not_found_pairs_with_not_required_no_visible_artifact() {
    let repo = repository::InMemoryAffairsRepository::new();
    let m60 = m60_fixture::M60FixtureAdapter::new("verifier:fixture", 1).unwrap();
    let clock = clock::FixedClock::new(t(200));
    let receipt = run(&repo, &m60, &clock, "proc:missing", Some(t(200)));
    assert!(matches!(
        receipt.outcome(),
        outcome::GetProcedureOutcome::NotFound { .. }
    ));
    assert_eq!(
        receipt.evidence_lineage().not_required_reason(),
        Some(lineage::EvidenceNotRequiredReason::NoVisibleArtifact)
    );
}

#[test]
fn archived_pairs_with_not_required_archived() {
    let mut repo = repository::InMemoryAffairsRepository::new();
    let pid = value::ProcedureId::parse("proc:archived").unwrap();
    let state = artifact::ProcedurePublicationState::archived(pid.clone(), t(50));
    repo.seed(
        build_artifact(
            "proc:archived",
            0,
            0,
            evidence::EvidenceConflictState::NoKnownConflict,
            evidence::AuthorityComparison::Equivalent,
            None,
            100,
            200,
            vec![assessment(
                evidence::AffairsAuthority::OfficialBulletin,
                "s1",
                evidence::AuthoritySubject::ProcedureTitle,
                0,
                0,
                None,
                None,
            )],
        ),
        state,
    )
    .expect("coherent archived fixture pair");
    let m60 = m60_fixture::M60FixtureAdapter::new("verifier:fixture", 1).unwrap();
    let clock = clock::FixedClock::new(t(200));
    let receipt = run(&repo, &m60, &clock, "proc:archived", Some(t(200)));
    assert!(matches!(
        receipt.outcome(),
        outcome::GetProcedureOutcome::Archived { .. }
    ));
    assert_eq!(
        receipt.evidence_lineage().not_required_reason(),
        Some(lineage::EvidenceNotRequiredReason::ArchivedWithoutCurrentArtifact)
    );
}

#[test]
fn not_yet_known_pairs_with_not_required_known_after_cutoff() {
    let artifact = build_artifact(
        "proc:future",
        300,
        100,
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
    let m60 = m60_fixture::M60FixtureAdapter::new("verifier:fixture", 1).unwrap();
    let clock = clock::FixedClock::new(t(200));
    let receipt = run(&repo, &m60, &clock, "proc:future", Some(t(200)));
    assert!(matches!(
        receipt.outcome(),
        outcome::GetProcedureOutcome::NotYetKnown { .. }
    ));
    assert_eq!(
        receipt.evidence_lineage().not_required_reason(),
        Some(lineage::EvidenceNotRequiredReason::KnownAfterCutoff)
    );
}

// ---------------------------------------------------------------------------
// NotRequired outcomes do NOT call M60 (call count stays zero)
// ---------------------------------------------------------------------------

#[test]
fn not_found_does_not_call_m60() {
    let repo = repository::InMemoryAffairsRepository::new();
    let m60 = CountingM60Adapter::new("verifier:fixture", 1);
    let clock = clock::FixedClock::new(t(200));
    let _ = run(&repo, &m60, &clock, "proc:missing", Some(t(200)));
    assert_eq!(m60.call_count(), 0);
}

#[test]
fn archived_does_not_call_m60() {
    let mut repo = repository::InMemoryAffairsRepository::new();
    let pid = value::ProcedureId::parse("proc:archived2").unwrap();
    let state = artifact::ProcedurePublicationState::archived(pid.clone(), t(50));
    repo.seed(
        build_artifact(
            "proc:archived2",
            0,
            0,
            evidence::EvidenceConflictState::NoKnownConflict,
            evidence::AuthorityComparison::Equivalent,
            None,
            100,
            200,
            vec![assessment(
                evidence::AffairsAuthority::OfficialBulletin,
                "s1",
                evidence::AuthoritySubject::ProcedureTitle,
                0,
                0,
                None,
                None,
            )],
        ),
        state,
    )
    .expect("coherent archived fixture pair");
    let m60 = CountingM60Adapter::new("verifier:fixture", 1);
    let clock = clock::FixedClock::new(t(200));
    let _ = run(&repo, &m60, &clock, "proc:archived2", Some(t(200)));
    assert_eq!(m60.call_count(), 0);
}

#[test]
fn not_yet_known_does_not_call_m60() {
    let artifact = build_artifact(
        "proc:future2",
        300,
        100,
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
    let m60 = CountingM60Adapter::new("verifier:fixture", 1);
    let clock = clock::FixedClock::new(t(200));
    let _ = run(&repo, &m60, &clock, "proc:future2", Some(t(200)));
    assert_eq!(m60.call_count(), 0);
}
