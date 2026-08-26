#![allow(clippy::unwrap_used)]

//! Public projection safety: no raw revision IDs, digests, actor references,
//! or journal bytes appear in the public projection's Debug output.

mod common;

use affairs_navigator::*;
use common::*;

/// The Debug output of a `PublicProcedureView` must NOT contain raw revision
/// IDs (`rev:...`), raw digests (`sha256:...`), actor references
/// (`actor:...`), or normalized/raw digest fragments.
#[test]
fn public_view_debug_has_no_raw_revision_or_digest() {
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
        "proc:safe",
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
    let service = service::AffairsGetService::new(&repo, &m60, &clock);
    let query = outcome::AffairsGetQuery::new(
        value::ProcedureId::parse("proc:safe").unwrap(),
        Some(t(200)),
    );
    let receipt = service.execute(&query).unwrap();
    if let outcome::GetProcedureOutcome::Found { view, .. } = receipt.outcome() {
        let debug = format!("{view:?}");
        // Raw revision IDs use the `rev:source:idx` pattern from fixtures.
        assert!(
            !debug.contains("rev:"),
            "public view Debug leaked revision ID: {debug}"
        );
        // Raw digests use the `sha256:` prefix.
        assert!(
            !debug.contains("sha256:"),
            "public view Debug leaked digest: {debug}"
        );
        // Actor references use the `actor:` prefix.
        assert!(
            !debug.contains("actor:"),
            "public view Debug leaked actor ref: {debug}"
        );
        // The 64-char hex digest fragments used in fixtures.
        let all_zeros = "0".repeat(64);
        let all_ones = "1".repeat(64);
        assert!(
            !debug.contains(&all_zeros),
            "public view Debug leaked raw digest hex: {debug}"
        );
        assert!(
            !debug.contains(&all_ones),
            "public view Debug leaked normalized digest hex: {debug}"
        );
    } else {
        panic!("expected Found");
    }
}

/// The Debug output of `PublicEvidenceAssessmentView` exposes only `source_id`
/// and review/verification times — NOT `revision_id`, `raw_digest`,
/// `normalized_digest`, or `assessed_by`.
#[test]
fn public_evidence_assessment_view_debug_is_safe() {
    let a = assessment(
        evidence::AffairsAuthority::OfficialBulletin,
        "src:safe",
        evidence::AuthoritySubject::ProcedureTitle,
        100,
        100,
        None,
        None,
    );
    let artifact = build_artifact(
        "proc:assess",
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
    m60.store(rev("src:safe", 0, None, None));
    let clock = clock::FixedClock::new(t(200));
    let service = service::AffairsGetService::new(&repo, &m60, &clock);
    let query = outcome::AffairsGetQuery::new(
        value::ProcedureId::parse("proc:assess").unwrap(),
        Some(t(200)),
    );
    let receipt = service.execute(&query).unwrap();
    if let outcome::GetProcedureOutcome::Found { view, .. } = receipt.outcome() {
        for assess in view.evidence().evidence_assessments() {
            let debug = format!("{assess:?}");
            assert!(
                !debug.contains("rev:"),
                "assessment Debug leaked revision ID: {debug}"
            );
            assert!(
                !debug.contains("sha256:"),
                "assessment Debug leaked digest: {debug}"
            );
            assert!(
                !debug.contains("actor:"),
                "assessment Debug leaked actor: {debug}"
            );
            // source_id IS safe to expose
            assert!(
                debug.contains("src:safe"),
                "assessment Debug should contain source_id: {debug}"
            );
        }
    } else {
        panic!("expected Found");
    }
}

/// `ConflictDetail` carries safe peer artifact references (`ArtifactId`), not
/// raw revision IDs or digests.
#[test]
fn conflict_detail_debug_is_safe() {
    let evidence = evidence::ProcedureEvidenceContext::new(
        evidence::ValidityHorizon::Unknown,
        t(0),
        t(50),
        t(0),
        t(150),
        vec![assessment(
            evidence::AffairsAuthority::OfficialBulletin,
            "s1",
            evidence::AuthoritySubject::ProcedureTitle,
            100,
            100,
            None,
            None,
        )],
        evidence::EvidenceConflictState::UnresolvedConflict,
        evidence::AuthorityComparison::Incomparable,
        evidence::UncertaintyState::None,
        Some(evidence::ConflictKind::OverlapIncompatible),
        vec![value::ArtifactId::parse("artifact:peer:v1").unwrap()],
    )
    .unwrap();
    let artifact = build_artifact_full(
        "proc:conflict",
        50,
        150,
        evidence::EvidenceConflictState::UnresolvedConflict,
        evidence::AuthorityComparison::Incomparable,
        Some(evidence::ConflictKind::OverlapIncompatible),
        100,
        200,
        evidence.evidence_assessments().to_vec(),
        Vec::new(),
    );
    let mut repo = repository::InMemoryAffairsRepository::new();
    seed_current(&mut repo, artifact);
    let mut m60 = m60_fixture::M60FixtureAdapter::new("verifier:fixture", 1).unwrap();
    m60.store(rev("s1", 0, None, None));
    let clock = clock::FixedClock::new(t(200));
    let service = service::AffairsGetService::new(&repo, &m60, &clock);
    let query = outcome::AffairsGetQuery::new(
        value::ProcedureId::parse("proc:conflict").unwrap(),
        Some(t(200)),
    );
    let receipt = service.execute(&query).unwrap();
    if let outcome::GetProcedureOutcome::Conflict { conflict, .. } = receipt.outcome() {
        let debug = format!("{conflict:?}");
        assert!(
            !debug.contains("rev:"),
            "conflict Debug leaked revision ID: {debug}"
        );
        assert!(
            !debug.contains("sha256:"),
            "conflict Debug leaked digest: {debug}"
        );
        assert!(
            debug.contains("overlapping but incompatible peer facts"),
            "conflict Debug should contain safe description: {debug}"
        );
    } else {
        panic!("expected Conflict");
    }
}

/// The `M71AffairsGetReceipt` Debug output carries the sealed lineage but not
/// raw M60 revision bytes.
#[test]
fn receipt_debug_has_no_raw_m60_bytes() {
    let repo = repository::InMemoryAffairsRepository::new();
    let m60 = m60_fixture::M60FixtureAdapter::new("verifier:fixture", 1).unwrap();
    let clock = clock::FixedClock::new(t(200));
    let service = service::AffairsGetService::new(&repo, &m60, &clock);
    let query =
        outcome::AffairsGetQuery::new(value::ProcedureId::parse("p").unwrap(), Some(t(200)));
    let receipt = service.execute(&query).unwrap();
    let debug = format!("{receipt:?}");
    assert!(
        !debug.contains("rev:"),
        "receipt Debug leaked revision ID: {debug}"
    );
    assert!(
        !debug.contains("sha256:"),
        "receipt Debug leaked digest: {debug}"
    );
}
