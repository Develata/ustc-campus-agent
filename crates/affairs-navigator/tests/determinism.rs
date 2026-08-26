#![allow(clippy::unwrap_used)]

//! Determinism: the same fixture/input twice produces byte-identical M71-owned
//! DTO-ready projection and lineage.

mod common;

use affairs_navigator::*;
use common::*;

fn run_twice(procedure_id: &str) -> (service::M71AffairsGetReceipt, service::M71AffairsGetReceipt) {
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
        procedure_id,
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
        value::ProcedureId::parse(procedure_id).unwrap(),
        Some(t(200)),
    );
    let r1 = service.execute(&query).unwrap();
    let r2 = service.execute(&query).unwrap();
    (r1, r2)
}

#[test]
fn same_input_produces_identical_receipt() {
    let (r1, r2) = run_twice("proc:deterministic");
    assert_eq!(r1, r2);
}

#[test]
fn same_input_produces_identical_debug_output() {
    let (r1, r2) = run_twice("proc:deterministic-debug");
    assert_eq!(format!("{r1:?}"), format!("{r2:?}"));
}

#[test]
fn same_input_produces_identical_outcome_and_lineage() {
    let (r1, r2) = run_twice("proc:deterministic-pair");
    assert_eq!(r1.outcome(), r2.outcome());
    assert_eq!(r1.evidence_lineage(), r2.evidence_lineage());
}

#[test]
fn materialization_receipt_id_is_deterministic() {
    let (r1, r2) = run_twice("proc:receipt-id");
    assert_eq!(
        r1.evidence_lineage().materialization_receipt_id(),
        r2.evidence_lineage().materialization_receipt_id()
    );
}

#[test]
fn not_found_receipt_is_deterministic() {
    let repo = repository::InMemoryAffairsRepository::new();
    let m60 = m60_fixture::M60FixtureAdapter::new("verifier:fixture", 1).unwrap();
    let clock = clock::FixedClock::new(t(200));
    let service = service::AffairsGetService::new(&repo, &m60, &clock);
    let query = outcome::AffairsGetQuery::new(
        value::ProcedureId::parse("proc:missing").unwrap(),
        Some(t(200)),
    );
    let r1 = service.execute(&query).unwrap();
    let r2 = service.execute(&query).unwrap();
    assert_eq!(r1, r2);
}
