#![allow(clippy::unwrap_used)]

//! Focused tests for the M71 application port seam (`M71AffairsGetPort`).
//!
//! Proves:
//! - the port delegates to `AffairsGetService::execute` and returns
//!   byte-identical receipts across all six outcomes (delegation + exact query
//!   forwarding);
//! - the sealed receipt is reachable through its public accessors only;
//! - infrastructure errors are forwarded unchanged;
//! - the port surface exposes no repository/M60/raw-evidence handle (trait
//!   object erasure + compile-time signature pin);
//! - the port and service are `Send + Sync`;
//! - the receipt constructor stays private, so callers cannot inject prebuilt
//!   receipts (the compile_fail doctest on the trait lives in
//!   `src/application_port.rs`).

mod common;

use affairs_navigator::*;
use common::{assessment, build_artifact, rev, seed_current, t};

/// Compares the port method against a direct `execute` call on the same
/// service. Equality proves delegation, exact query forwarding, and sealed
/// receipt reachability in one assertion.
fn assert_port_delegates(
    repo: &InMemoryAffairsRepository,
    m60: &dyn M60ProcedureEvidencePort,
    clock: &FixedClock,
    query: &AffairsGetQuery,
) {
    let service = AffairsGetService::new(repo, m60, clock);
    let direct = service.execute(query);
    let via_port = M71AffairsGetPort::affairs_get(&service, query);
    assert_eq!(
        direct, via_port,
        "port.affairs_get must delegate to service.execute byte-identically"
    );
}

// ---------------------------------------------------------------------------
// Delegation across all six outcomes
// ---------------------------------------------------------------------------

#[test]
fn port_delegates_for_not_found() {
    let repo = InMemoryAffairsRepository::new();
    let m60 = m60_fixture::M60FixtureAdapter::new("verifier:fixture", 1).unwrap();
    let clock = FixedClock::new(t(200));
    let query = AffairsGetQuery::new(ProcedureId::parse("proc:missing").unwrap(), Some(t(200)));
    assert_port_delegates(&repo, &m60, &clock, &query);

    // Sealed receipt reachability: the port-returned receipt exposes only the
    // public outcome/lineage accessors.
    let service = AffairsGetService::new(&repo, &m60, &clock);
    let receipt = M71AffairsGetPort::affairs_get(&service, &query).unwrap();
    assert!(matches!(
        receipt.outcome(),
        GetProcedureOutcome::NotFound { .. }
    ));
    assert!(receipt.evidence_lineage().is_not_required());
}

#[test]
fn port_delegates_for_archived() {
    let pid = ProcedureId::parse("proc:archived").unwrap();
    let artifact = build_artifact(
        "proc:archived",
        0,
        0,
        EvidenceConflictState::NoKnownConflict,
        AuthorityComparison::Equivalent,
        None,
        100,
        200,
        vec![assessment(
            AffairsAuthority::OfficialBulletin,
            "s1",
            AuthoritySubject::ProcedureTitle,
            100,
            100,
            None,
            None,
        )],
    );
    let mut repo = InMemoryAffairsRepository::new();
    let state = ProcedurePublicationState::archived(pid.clone(), t(50));
    repo.seed(artifact, state).unwrap();
    let m60 = m60_fixture::M60FixtureAdapter::new("verifier:fixture", 1).unwrap();
    let clock = FixedClock::new(t(200));
    let query = AffairsGetQuery::new(pid, Some(t(200)));
    assert_port_delegates(&repo, &m60, &clock, &query);
}

#[test]
fn port_delegates_for_not_yet_known() {
    let artifact = build_artifact(
        "proc:future",
        300,
        100,
        EvidenceConflictState::NoKnownConflict,
        AuthorityComparison::Equivalent,
        None,
        100,
        200,
        vec![assessment(
            AffairsAuthority::OfficialBulletin,
            "s1",
            AuthoritySubject::ProcedureTitle,
            100,
            100,
            None,
            None,
        )],
    );
    let mut repo = InMemoryAffairsRepository::new();
    seed_current(&mut repo, artifact);
    let m60 = m60_fixture::M60FixtureAdapter::new("verifier:fixture", 1).unwrap();
    let clock = FixedClock::new(t(200));
    let query = AffairsGetQuery::new(ProcedureId::parse("proc:future").unwrap(), Some(t(200)));
    assert_port_delegates(&repo, &m60, &clock, &query);
}

#[test]
fn port_delegates_for_found() {
    let artifact = build_artifact(
        "proc:found",
        50,
        150,
        EvidenceConflictState::NoKnownConflict,
        AuthorityComparison::Equivalent,
        None,
        100,
        200,
        vec![assessment(
            AffairsAuthority::OfficialBulletin,
            "s1",
            AuthoritySubject::ProcedureTitle,
            100,
            100,
            None,
            None,
        )],
    );
    let mut repo = InMemoryAffairsRepository::new();
    seed_current(&mut repo, artifact);
    let mut m60 = m60_fixture::M60FixtureAdapter::new("verifier:fixture", 1).unwrap();
    m60.store(rev("s1", 0, None, None));
    let clock = FixedClock::new(t(200));
    let query = AffairsGetQuery::new(ProcedureId::parse("proc:found").unwrap(), Some(t(200)));
    assert_port_delegates(&repo, &m60, &clock, &query);

    // Sealed receipt reachability for the Verified pairing.
    let service = AffairsGetService::new(&repo, &m60, &clock);
    let receipt = M71AffairsGetPort::affairs_get(&service, &query).unwrap();
    assert!(matches!(
        receipt.outcome(),
        GetProcedureOutcome::Found { .. }
    ));
    assert!(receipt.evidence_lineage().is_verified());
}

#[test]
fn port_delegates_for_conflict() {
    let artifact = build_artifact(
        "proc:conflict",
        50,
        150,
        EvidenceConflictState::UnresolvedConflict,
        AuthorityComparison::Incomparable,
        Some(ConflictKind::DirectContradiction),
        100,
        200,
        vec![assessment(
            AffairsAuthority::OfficialBulletin,
            "s1",
            AuthoritySubject::ProcedureTitle,
            100,
            100,
            None,
            None,
        )],
    );
    let mut repo = InMemoryAffairsRepository::new();
    seed_current(&mut repo, artifact);
    let mut m60 = m60_fixture::M60FixtureAdapter::new("verifier:fixture", 1).unwrap();
    m60.store(rev("s1", 0, None, None));
    let clock = FixedClock::new(t(200));
    let query = AffairsGetQuery::new(ProcedureId::parse("proc:conflict").unwrap(), Some(t(200)));
    assert_port_delegates(&repo, &m60, &clock, &query);
}

#[test]
fn port_delegates_for_cannot_verify_stale_beyond_policy() {
    let artifact = build_artifact(
        "proc:beyond",
        50,
        50,
        EvidenceConflictState::NoKnownConflict,
        AuthorityComparison::Equivalent,
        None,
        10,
        20,
        vec![assessment(
            AffairsAuthority::OfficialBulletin,
            "s1",
            AuthoritySubject::ProcedureTitle,
            100,
            100,
            None,
            None,
        )],
    );
    let mut repo = InMemoryAffairsRepository::new();
    seed_current(&mut repo, artifact);
    let mut m60 = m60_fixture::M60FixtureAdapter::new("verifier:fixture", 1).unwrap();
    m60.store(rev("s1", 0, None, None));
    let clock = FixedClock::new(t(200));
    let query = AffairsGetQuery::new(ProcedureId::parse("proc:beyond").unwrap(), Some(t(200)));
    assert_port_delegates(&repo, &m60, &clock, &query);
}

// ---------------------------------------------------------------------------
// Error forwarding
// ---------------------------------------------------------------------------

#[test]
fn port_forwards_infrastructure_error() {
    let artifact = build_artifact(
        "proc:infra",
        50,
        150,
        EvidenceConflictState::NoKnownConflict,
        AuthorityComparison::Equivalent,
        None,
        100,
        200,
        vec![assessment(
            AffairsAuthority::OfficialBulletin,
            "s1",
            AuthoritySubject::ProcedureTitle,
            100,
            100,
            None,
            None,
        )],
    );
    let mut repo = InMemoryAffairsRepository::new();
    seed_current(&mut repo, artifact);
    let mut m60 = m60_fixture::M60FixtureAdapter::new("verifier:fixture", 1).unwrap();
    m60.store(rev("s1", 0, None, None));
    m60.set_failure_mode(Some(M60EvidencePortError::StoreUnavailable));
    let clock = FixedClock::new(t(200));
    let query = AffairsGetQuery::new(ProcedureId::parse("proc:infra").unwrap(), Some(t(200)));
    let service = AffairsGetService::new(&repo, &m60, &clock);

    let direct = service.execute(&query);
    let via_port = M71AffairsGetPort::affairs_get(&service, &query);

    assert_eq!(direct, via_port);
    assert!(matches!(
        via_port,
        Err(GetProcedureError::M60StoreUnavailable)
    ));
}

// ---------------------------------------------------------------------------
// Port surface exposes no repository/M60/raw-evidence handle
// ---------------------------------------------------------------------------

/// A caller holding `&dyn M71AffairsGetPort` needs nothing but the query,
/// receipt, and error types. This function does not name `AffairsRepository`,
/// `M60ProcedureEvidencePort`, `M60RevisionRef`, or any raw evidence handle,
/// proving the port surface erases M71 internals.
fn call_erased_port(
    port: &dyn M71AffairsGetPort,
    query: &AffairsGetQuery,
) -> Result<M71AffairsGetReceipt, GetProcedureError> {
    port.affairs_get(query)
}

#[test]
fn port_trait_object_erases_repository_and_m60() {
    let repo = InMemoryAffairsRepository::new();
    let m60 = m60_fixture::M60FixtureAdapter::new("verifier:fixture", 1).unwrap();
    let clock = FixedClock::new(t(200));
    let service = AffairsGetService::new(&repo, &m60, &clock);
    let query = AffairsGetQuery::new(ProcedureId::parse("proc:missing").unwrap(), Some(t(200)));

    // The service is usable as an erased `&dyn M71AffairsGetPort`.
    let receipt = call_erased_port(&service, &query).unwrap();
    assert!(matches!(
        receipt.outcome(),
        GetProcedureOutcome::NotFound { .. }
    ));
}

/// Pins the trait method signature at compile time. If the signature ever
/// widens to expose a repository/M60/raw-evidence handle, this stops
/// compiling.
fn assert_port_signature_exact<P: M71AffairsGetPort + ?Sized>() {
    let _signature: fn(&P, &AffairsGetQuery) -> Result<M71AffairsGetReceipt, GetProcedureError> =
        P::affairs_get;
}

#[test]
fn port_method_signature_is_exact() {
    assert_port_signature_exact::<AffairsGetService<'static>>();
    assert_port_signature_exact::<dyn M71AffairsGetPort>();
}

// ---------------------------------------------------------------------------
// Send + Sync
// ---------------------------------------------------------------------------

#[test]
fn port_and_service_are_send_sync() {
    fn assert_send_sync<T: Send + Sync + ?Sized>() {}
    assert_send_sync::<AffairsGetService<'static>>();
    assert_send_sync::<dyn M71AffairsGetPort>();
}

// ---------------------------------------------------------------------------
// Exact query forwarding: distinct queries produce distinct receipts, and the
// port never substitutes or mutates the query.
// ---------------------------------------------------------------------------

#[test]
fn port_forwards_distinct_queries_without_substitution() {
    let repo = InMemoryAffairsRepository::new();
    let m60 = m60_fixture::M60FixtureAdapter::new("verifier:fixture", 1).unwrap();
    let clock = FixedClock::new(t(200));
    let service = AffairsGetService::new(&repo, &m60, &clock);

    let q_a = AffairsGetQuery::new(ProcedureId::parse("proc:a").unwrap(), Some(t(200)));
    let q_b = AffairsGetQuery::new(ProcedureId::parse("proc:b").unwrap(), Some(t(200)));

    let r_a = M71AffairsGetPort::affairs_get(&service, &q_a).unwrap();
    let r_b = M71AffairsGetPort::affairs_get(&service, &q_b).unwrap();

    // Both are NotFound, but for distinct procedure IDs. The port must forward
    // the exact procedure ID, not a substituted one.
    match (r_a.outcome(), r_b.outcome()) {
        (
            GetProcedureOutcome::NotFound { procedure_id: id_a },
            GetProcedureOutcome::NotFound { procedure_id: id_b },
        ) => {
            assert_eq!(id_a.as_str(), "proc:a");
            assert_eq!(id_b.as_str(), "proc:b");
            assert_ne!(id_a, id_b);
        }
        _ => panic!("expected NotFound for both queries"),
    }
}
