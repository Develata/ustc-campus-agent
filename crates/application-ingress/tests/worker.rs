#![allow(clippy::unwrap_used)]

mod common;

use std::sync::Arc;

use affairs_navigator::m60_fixture::M60FixtureAdapter;
use affairs_navigator::{
    AffairsAuthority, AuthorityComparison, AuthoritySubject, ConflictKind, EvidenceConflictState,
    FixedClock, InMemoryAffairsRepository,
};
use ustc_campus_agent_application_ingress::{FileRecordStore, M10Service, RecordState};
use ustc_campus_agent_client_protocol::{
    ClientErrorDto, ClientResponseDto, M71OutcomeDto, WireErrorClassDto,
};
use ustc_campus_agent_core::request_context::{
    AdmissionPortError, AdmissionPortKind, CapabilityDisposition, DescriptorSnapshotError,
    IdempotencyError, IdempotencyReservation, OperationSnapshot, PolicyCurrentnessFact,
    PolicyResolution,
};

use common::{
    FailingM71Port, FakePorts, M71FixturePort, assessment, at, build_artifact, cap_issuer, rev,
    seed_repo, submit_request, submit_request_authenticated, t, temp_path,
};

fn make_service(m71: &dyn affairs_navigator::M71AffairsGetPort) -> M10Service<'_> {
    let store = FileRecordStore::open(temp_path()).unwrap();
    M10Service::new(store, cap_issuer(), m71, "operator:fixture")
}

// ---------------------------------------------------------------------------
// M71 six-outcome tests
// ---------------------------------------------------------------------------

#[test]
fn m71_not_found_returns_accepted_terminal() {
    let repo = InMemoryAffairsRepository::new();
    let m60 = M60FixtureAdapter::new("verifier:fixture", 1).unwrap();
    let clock = FixedClock::new(t(200));
    let m71 = M71FixturePort::new(&repo, &m60, &clock);
    let service = make_service(&m71);

    let mut ports = FakePorts::public_admitted();
    let request = submit_request("proc:missing");
    let response = service.submit(&request, &mut ports, 1_000_000);

    match response {
        ClientResponseDto::Accepted { terminal, .. } => {
            assert!(matches!(terminal.outcome(), M71OutcomeDto::NotFound { .. }));
        }
        _ => panic!("expected Accepted, got {response:?}"),
    }
}

#[test]
fn m71_archived_returns_accepted_terminal() {
    let mut repo = InMemoryAffairsRepository::new();
    let pid = affairs_navigator::ProcedureId::parse("proc:archived").unwrap();
    let state = affairs_navigator::ProcedurePublicationState::archived(pid.clone(), t(50));
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
        )],
    );
    repo.seed(artifact, state).unwrap();
    let m60 = M60FixtureAdapter::new("verifier:fixture", 1).unwrap();
    let clock = FixedClock::new(t(200));
    let m71 = M71FixturePort::new(&repo, &m60, &clock);
    let service = make_service(&m71);

    let mut ports = FakePorts::public_admitted();
    let request = submit_request("proc:archived");
    let response = service.submit(&request, &mut ports, 1_000_000);

    match response {
        ClientResponseDto::Accepted { terminal, .. } => {
            assert!(matches!(terminal.outcome(), M71OutcomeDto::Archived { .. }));
        }
        _ => panic!("expected Accepted"),
    }
}

#[test]
fn m71_not_yet_known_returns_accepted_terminal() {
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
        )],
    );
    let repo = seed_repo(artifact);
    let m60 = M60FixtureAdapter::new("verifier:fixture", 1).unwrap();
    let clock = FixedClock::new(t(200));
    let m71 = M71FixturePort::new(&repo, &m60, &clock);
    let service = make_service(&m71);

    let mut ports = FakePorts::public_admitted();
    let request = submit_request("proc:future");
    let response = service.submit(&request, &mut ports, 1_000_000);

    match response {
        ClientResponseDto::Accepted { terminal, .. } => {
            assert!(matches!(
                terminal.outcome(),
                M71OutcomeDto::NotYetKnown { .. }
            ));
        }
        _ => panic!("expected Accepted"),
    }
}

#[test]
fn m71_found_returns_accepted_terminal() {
    let a = assessment(
        AffairsAuthority::OfficialBulletin,
        "s1",
        AuthoritySubject::ProcedureTitle,
    );
    let artifact = build_artifact(
        "proc:found",
        50,
        150,
        EvidenceConflictState::NoKnownConflict,
        AuthorityComparison::Equivalent,
        None,
        100,
        200,
        vec![a],
    );
    let repo = seed_repo(artifact);
    let mut m60 = M60FixtureAdapter::new("verifier:fixture", 1).unwrap();
    m60.store(rev("s1", 0, None, None));
    let clock = FixedClock::new(t(200));
    let m71 = M71FixturePort::new(&repo, &m60, &clock);
    let service = make_service(&m71);

    let mut ports = FakePorts::public_admitted();
    let request = submit_request("proc:found");
    let response = service.submit(&request, &mut ports, 1_000_000);

    match response {
        ClientResponseDto::Accepted { terminal, .. } => {
            assert!(matches!(terminal.outcome(), M71OutcomeDto::Found { .. }));
        }
        _ => panic!("expected Accepted"),
    }
}

#[test]
fn m71_conflict_returns_accepted_terminal() {
    let a = assessment(
        AffairsAuthority::OfficialBulletin,
        "s1",
        AuthoritySubject::ProcedureTitle,
    );
    let artifact = build_artifact(
        "proc:conflict",
        50,
        150,
        EvidenceConflictState::UnresolvedConflict,
        AuthorityComparison::Incomparable,
        Some(ConflictKind::DirectContradiction),
        100,
        200,
        vec![a],
    );
    let repo = seed_repo(artifact);
    let mut m60 = M60FixtureAdapter::new("verifier:fixture", 1).unwrap();
    m60.store(rev("s1", 0, None, None));
    let clock = FixedClock::new(t(200));
    let m71 = M71FixturePort::new(&repo, &m60, &clock);
    let service = make_service(&m71);

    let mut ports = FakePorts::public_admitted();
    let request = submit_request("proc:conflict");
    let response = service.submit(&request, &mut ports, 1_000_000);

    match response {
        ClientResponseDto::Accepted { terminal, .. } => {
            assert!(matches!(terminal.outcome(), M71OutcomeDto::Conflict { .. }));
        }
        _ => panic!("expected Accepted"),
    }
}

#[test]
fn m71_cannot_verify_stale_beyond_policy_returns_accepted_terminal() {
    let a = assessment(
        AffairsAuthority::OfficialBulletin,
        "s1",
        AuthoritySubject::ProcedureTitle,
    );
    let artifact = build_artifact(
        "proc:beyond",
        50,
        50,
        EvidenceConflictState::NoKnownConflict,
        AuthorityComparison::Equivalent,
        None,
        10,
        20,
        vec![a],
    );
    let repo = seed_repo(artifact);
    let mut m60 = M60FixtureAdapter::new("verifier:fixture", 1).unwrap();
    m60.store(rev("s1", 0, None, None));
    let clock = FixedClock::new(t(200));
    let m71 = M71FixturePort::new(&repo, &m60, &clock);
    let service = make_service(&m71);

    let mut ports = FakePorts::public_admitted();
    let request = submit_request("proc:beyond");
    let response = service.submit(&request, &mut ports, 1_000_000);

    match response {
        ClientResponseDto::Accepted { terminal, .. } => {
            assert!(matches!(
                terminal.outcome(),
                M71OutcomeDto::CannotVerify { .. }
            ));
        }
        _ => panic!("expected Accepted"),
    }
}

#[test]
fn m71_cannot_verify_source_revision_unverified_returns_accepted_terminal() {
    let a = assessment(
        AffairsAuthority::OfficialBulletin,
        "s1",
        AuthoritySubject::ProcedureTitle,
    );
    let artifact = build_artifact(
        "proc:unverified",
        50,
        150,
        EvidenceConflictState::NoKnownConflict,
        AuthorityComparison::Equivalent,
        None,
        100,
        200,
        vec![a],
    );
    let repo = seed_repo(artifact);
    let m60 = M60FixtureAdapter::new("verifier:fixture", 1).unwrap();
    let clock = FixedClock::new(t(200));
    let m71 = M71FixturePort::new(&repo, &m60, &clock);
    let service = make_service(&m71);

    let mut ports = FakePorts::public_admitted();
    let request = submit_request("proc:unverified");
    let response = service.submit(&request, &mut ports, 1_000_000);

    match response {
        ClientResponseDto::Accepted { terminal, .. } => {
            assert!(matches!(
                terminal.outcome(),
                M71OutcomeDto::CannotVerify { .. }
            ));
        }
        _ => panic!("expected Accepted"),
    }
}

#[test]
fn m71_cannot_verify_effective_interval_missing_returns_accepted_terminal() {
    let a = assessment(
        AffairsAuthority::OfficialBulletin,
        "s1",
        AuthoritySubject::ProcedureTitle,
    );
    let artifact = build_artifact(
        "proc:effmiss",
        50,
        150,
        EvidenceConflictState::NoKnownConflict,
        AuthorityComparison::Equivalent,
        None,
        100,
        200,
        vec![a],
    );
    let repo = seed_repo(artifact);
    let mut m60 = M60FixtureAdapter::new("verifier:fixture", 1).unwrap();
    m60.store(rev("s1", 0, None, None));
    m60.require_effective_interval(true);
    let clock = FixedClock::new(t(200));
    let m71 = M71FixturePort::new(&repo, &m60, &clock);
    let service = make_service(&m71);

    let mut ports = FakePorts::public_admitted();
    let request = submit_request("proc:effmiss");
    let response = service.submit(&request, &mut ports, 1_000_000);

    match response {
        ClientResponseDto::Accepted { terminal, .. } => {
            assert!(matches!(
                terminal.outcome(),
                M71OutcomeDto::CannotVerify { .. }
            ));
        }
        _ => panic!("expected Accepted"),
    }
}

// ---------------------------------------------------------------------------
// M00 5-arm tests: Incomplete and infrastructure failure
// ---------------------------------------------------------------------------

#[test]
fn m00_incomplete_returns_incomplete_response() {
    let m71 = FailingM71Port;
    let service = make_service(&m71);

    let mut ports = FakePorts::public_admitted();
    ports.reservation = Ok(IdempotencyReservation::InFlight(common::reservation_token(
        1,
    )));
    let request = submit_request("proc:test");
    let response = service.submit(&request, &mut ports, 1_000_000);

    match response {
        ClientResponseDto::Incomplete { .. } => {}
        _ => panic!("expected Incomplete, got {response:?}"),
    }
}

#[test]
fn m60_infrastructure_failure_returns_infra_error_and_abandons() {
    let m71 = FailingM71Port;
    let service = make_service(&m71);

    let mut ports = FakePorts::public_admitted();
    let request = submit_request("proc:fixture");
    let response = service.submit(&request, &mut ports, 1_000_000);

    match response {
        ClientResponseDto::Error {
            error: ClientErrorDto::Infrastructure { retryable, .. },
        } => {
            assert!(retryable, "M60StoreUnavailable should be retryable");
        }
        _ => panic!("expected Infrastructure error, got {response:?}"),
    }

    let record = service
        .store()
        .get("command:fixture")
        .unwrap()
        .expect("record should exist after abandon");
    assert!(
        matches!(record.state, RecordState::Pending { .. }),
        "record should be Pending after abandon, got {:?}",
        record.state
    );
}

// ---------------------------------------------------------------------------
// M00 14-row rejection projection
// ---------------------------------------------------------------------------

fn assert_admission_error(
    response: &ClientResponseDto,
    expected_class: WireErrorClassDto,
    name: &str,
) {
    match response {
        ClientResponseDto::Error {
            error: ClientErrorDto::Admission { error },
        } => {
            assert_eq!(
                error.class, expected_class,
                "{name}: wrong wire error class"
            );
        }
        _ => panic!("{name}: expected Admission error, got {response:?}"),
    }
}

#[test]
fn rejection_idempotency_store_unavailable() {
    let m71 = FailingM71Port;
    let service = make_service(&m71);
    let mut ports = FakePorts::public_admitted();
    ports.reservation = Err(IdempotencyError::StoreUnavailable);
    let response = service.submit(&submit_request("proc:x"), &mut ports, 1_000_000);
    assert_admission_error(
        &response,
        WireErrorClassDto::IdempotencyStoreUnavailable,
        "idempotency_store_unavailable",
    );
}

#[test]
fn rejection_conflicting_envelope() {
    let m71 = FailingM71Port;
    let service = make_service(&m71);
    let mut ports = FakePorts::public_admitted();
    ports.reservation = Err(IdempotencyError::ConflictingEnvelope {
        idempotency_key: common::idem_key(),
    });
    let response = service.submit(&submit_request("proc:x"), &mut ports, 1_000_000);
    assert_admission_error(
        &response,
        WireErrorClassDto::ConflictingEnvelope,
        "conflicting_envelope",
    );
}

#[test]
fn rejection_descriptor_snapshot_absent() {
    let m71 = FailingM71Port;
    let service = make_service(&m71);
    let mut ports = FakePorts::public_admitted();
    ports.descriptor = Err(DescriptorSnapshotError::Absent);
    ports.staged = Arc::new(common::Descriptor::public_read());
    let response = service.submit(&submit_request("proc:x"), &mut ports, 1_000_000);
    assert_admission_error(
        &response,
        WireErrorClassDto::DescriptorSnapshotAbsent,
        "descriptor_snapshot_absent",
    );
}

#[test]
fn rejection_descriptor_snapshot_mismatch() {
    let m71 = FailingM71Port;
    let service = make_service(&m71);
    let wrong: OperationSnapshot = Arc::new(common::Descriptor::wrong_operation());
    let mut ports = FakePorts::public_admitted();
    ports.descriptor = Ok(Arc::clone(&wrong));
    ports.staged = wrong;
    let response = service.submit(&submit_request("proc:x"), &mut ports, 1_000_000);
    assert_admission_error(
        &response,
        WireErrorClassDto::DescriptorSnapshotMismatch,
        "descriptor_snapshot_mismatch",
    );
}

#[test]
fn rejection_policy_denied() {
    let m71 = FailingM71Port;
    let service = make_service(&m71);
    let desc: OperationSnapshot = Arc::new(common::Descriptor::tenant_private_write());
    let mut ports = FakePorts::public_admitted();
    ports.descriptor = Ok(Arc::clone(&desc));
    ports.staged = desc;
    let response = service.submit(&submit_request("proc:x"), &mut ports, 1_000_000);
    assert_admission_error(&response, WireErrorClassDto::PolicyDenied, "policy_denied");
}

#[test]
fn rejection_policy_expired() {
    let m71 = FailingM71Port;
    let service = make_service(&m71);
    let mut ports = FakePorts::public_admitted();
    ports.policy = Ok(PolicyResolution::new(
        common::policy_id(),
        PolicyCurrentnessFact::Stale,
    ));
    let response = service.submit(&submit_request("proc:x"), &mut ports, 1_000_000);
    assert_admission_error(
        &response,
        WireErrorClassDto::PolicyExpired,
        "policy_expired",
    );
}

#[test]
fn rejection_session_not_found() {
    let m71 = FailingM71Port;
    let service = make_service(&m71);
    let mut ports = FakePorts::authenticated_admitted("session:fixture");
    ports.loaded_session = Ok(None);
    let request = submit_request_authenticated("proc:x", "session:fixture");
    let response = service.submit(&request, &mut ports, 1_000_000);
    assert_admission_error(
        &response,
        WireErrorClassDto::SessionNotFound,
        "session_not_found",
    );
}

#[test]
fn rejection_session_id_mismatch() {
    let m71 = FailingM71Port;
    let service = make_service(&m71);
    let mut ports = FakePorts::authenticated_admitted("session:fixture");
    let request = submit_request_authenticated("proc:x", "session:DIFFERENT");
    let response = service.submit(&request, &mut ports, 1_000_000);
    assert_admission_error(
        &response,
        WireErrorClassDto::SessionIdMismatch,
        "session_id_mismatch",
    );
}

#[test]
fn rejection_session_not_admitted() {
    let m71 = FailingM71Port;
    let service = make_service(&m71);
    let mut ports = FakePorts::authenticated_admitted("session:fixture");
    ports.now = Ok(at(5000));
    let request = submit_request_authenticated("proc:x", "session:fixture");
    let response = service.submit(&request, &mut ports, 1_000_000);
    assert_admission_error(
        &response,
        WireErrorClassDto::SessionNotAdmitted,
        "session_not_admitted",
    );
}

#[test]
fn rejection_capability_missing() {
    let m71 = FailingM71Port;
    let service = make_service(&m71);
    let mut ports = FakePorts::public_admitted();
    ports.capability = Ok(CapabilityDisposition::Missing);
    let response = service.submit(&submit_request("proc:x"), &mut ports, 1_000_000);
    assert_admission_error(
        &response,
        WireErrorClassDto::CapabilityMissing,
        "capability_missing",
    );
}

#[test]
fn rejection_capability_disabled() {
    let m71 = FailingM71Port;
    let service = make_service(&m71);
    let mut ports = FakePorts::public_admitted();
    ports.capability = Ok(CapabilityDisposition::Disabled);
    let response = service.submit(&submit_request("proc:x"), &mut ports, 1_000_000);
    assert_admission_error(
        &response,
        WireErrorClassDto::CapabilityDisabled,
        "capability_disabled",
    );
}

#[test]
fn rejection_capability_revoked() {
    let m71 = FailingM71Port;
    let service = make_service(&m71);
    let mut ports = FakePorts::public_admitted();
    ports.capability = Ok(CapabilityDisposition::Revoked);
    let response = service.submit(&submit_request("proc:x"), &mut ports, 1_000_000);
    assert_admission_error(
        &response,
        WireErrorClassDto::CapabilityRevoked,
        "capability_revoked",
    );
}

#[test]
fn rejection_infrastructure_port_unavailable() {
    let m71 = FailingM71Port;
    let service = make_service(&m71);
    let mut ports = FakePorts::public_admitted();
    ports.now = Err(AdmissionPortError::Unavailable(AdmissionPortKind::Clock));
    let response = service.submit(&submit_request("proc:x"), &mut ports, 1_000_000);
    assert_admission_error(
        &response,
        WireErrorClassDto::InfrastructurePortUnavailable,
        "infrastructure_port_unavailable",
    );
}

// ---------------------------------------------------------------------------
// Rejection zero-effect: no store record created
// ---------------------------------------------------------------------------

#[test]
fn rejection_creates_no_store_record() {
    let m71 = FailingM71Port;
    let service = make_service(&m71);
    let mut ports = FakePorts::public_admitted();
    ports.reservation = Err(IdempotencyError::StoreUnavailable);
    let _response = service.submit(&submit_request("proc:x"), &mut ports, 1_000_000);

    assert!(
        service.store().get("command:fixture").unwrap().is_none(),
        "rejection must not create a store record"
    );
}

// ---------------------------------------------------------------------------
// Exhaustive rejection count (14 rows in AdmissionRejectionProjection)
// ---------------------------------------------------------------------------

#[test]
fn m00_has_14_rejection_rows() {
    let rows = [
        "IdempotencyStoreUnavailable",
        "ConflictingEnvelope",
        "DescriptorSnapshotAbsent",
        "DescriptorSnapshotMismatch",
        "PolicyDenied",
        "PolicyExpired",
        "SessionNotFound",
        "SessionIdMismatch",
        "SessionNotAdmitted",
        "CapabilityMissing",
        "CapabilityDisabled",
        "CapabilityRevoked",
        "InfrastructurePortUnavailable",
        "MalformedCommand",
    ];
    assert_eq!(rows.len(), 14);
    for name in &rows {
        assert!(
            SOURCE.contains(name),
            "AdmissionRejectionProjection must have variant {name}"
        );
    }
}

// ---------------------------------------------------------------------------
// R6: post-admit validation failures abandon the claim and return InternalInvariant
// ---------------------------------------------------------------------------

/// `procedure_id` that passes WireText but fails the M71 ID grammar (uppercase
/// start byte is invalid for ProcedureId).
fn submit_request_bad_procedure_id() -> ustc_campus_agent_client_protocol::SubmitAffairsGetDto {
    ustc_campus_agent_client_protocol::SubmitAffairsGetDto {
        request_id: ustc_campus_agent_client_protocol::WireText::parse("req:fixture").unwrap(),
        correlation_id: ustc_campus_agent_client_protocol::WireText::parse("corr:fixture").unwrap(),
        causation_id: None,
        idempotency_key: Some(
            ustc_campus_agent_client_protocol::WireText::parse("idem:fixture").unwrap(),
        ),
        actor: ustc_campus_agent_client_protocol::ActorIntentDto::Public,
        provenance: ustc_campus_agent_client_protocol::ClientProvenanceDto {
            build: ustc_campus_agent_client_protocol::WireText::parse("build:fixture").unwrap(),
            target: ustc_campus_agent_client_protocol::WireText::parse("linux").unwrap(),
            protocol: ustc_campus_agent_client_protocol::WireText::parse("m10:v2").unwrap(),
        },
        payload_digest: ustc_campus_agent_client_protocol::WireText::parse(
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        )
        .unwrap(),
        procedure_id: ustc_campus_agent_client_protocol::WireText::parse("INVALID").unwrap(),
        as_of: None,
    }
}

#[test]
fn r6_bad_procedure_id_returns_internal_error_and_abandons() {
    let m71 = FailingM71Port;
    let service = make_service(&m71);

    let mut ports = FakePorts::public_admitted();
    let request = submit_request_bad_procedure_id();
    let response = service.submit(&request, &mut ports, 1_000_000);

    match response {
        ClientResponseDto::Error {
            error: ClientErrorDto::InternalInvariant { .. },
        } => {}
        _ => panic!("expected InternalInvariant, got {response:?}"),
    }

    let record = service
        .store()
        .get("command:fixture")
        .unwrap()
        .expect("record should exist after abandon");
    assert!(
        matches!(record.state, RecordState::Pending { .. }),
        "record should be Pending after abandon, got {:?}",
        record.state
    );
}

/// `as_of` value far exceeding `i64::MAX` nanoseconds, causing
/// `OffsetDateTime::from_unix_timestamp_nanos` to fail.
fn submit_request_bad_as_of() -> ustc_campus_agent_client_protocol::SubmitAffairsGetDto {
    ustc_campus_agent_client_protocol::SubmitAffairsGetDto {
        request_id: ustc_campus_agent_client_protocol::WireText::parse("req:fixture").unwrap(),
        correlation_id: ustc_campus_agent_client_protocol::WireText::parse("corr:fixture").unwrap(),
        causation_id: None,
        idempotency_key: Some(
            ustc_campus_agent_client_protocol::WireText::parse("idem:fixture").unwrap(),
        ),
        actor: ustc_campus_agent_client_protocol::ActorIntentDto::Public,
        provenance: ustc_campus_agent_client_protocol::ClientProvenanceDto {
            build: ustc_campus_agent_client_protocol::WireText::parse("build:fixture").unwrap(),
            target: ustc_campus_agent_client_protocol::WireText::parse("linux").unwrap(),
            protocol: ustc_campus_agent_client_protocol::WireText::parse("m10:v2").unwrap(),
        },
        payload_digest: ustc_campus_agent_client_protocol::WireText::parse(
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        )
        .unwrap(),
        procedure_id: ustc_campus_agent_client_protocol::WireText::parse("proc:fixture").unwrap(),
        as_of: Some(ustc_campus_agent_client_protocol::UnixMillis::new(i64::MAX)),
    }
}

#[test]
fn r6_bad_as_of_returns_internal_error_and_abandons() {
    let m71 = FailingM71Port;
    let service = make_service(&m71);

    let mut ports = FakePorts::public_admitted();
    let request = submit_request_bad_as_of();
    let response = service.submit(&request, &mut ports, 1_000_000);

    match response {
        ClientResponseDto::Error {
            error: ClientErrorDto::InternalInvariant { .. },
        } => {}
        _ => panic!("expected InternalInvariant, got {response:?}"),
    }

    let record = service
        .store()
        .get("command:fixture")
        .unwrap()
        .expect("record should exist after abandon");
    assert!(
        matches!(record.state, RecordState::Pending { .. }),
        "record should be Pending after abandon, got {:?}",
        record.state
    );
}

// ---------------------------------------------------------------------------
// R3: M71 infrastructure failure returns retryable Infrastructure error and abandons
// ---------------------------------------------------------------------------

#[test]
fn r3_m71_infrastructure_failure_abandons_and_returns_retryable_error() {
    let m71 = FailingM71Port;
    let service = make_service(&m71);

    let mut ports = FakePorts::public_admitted();
    let request = submit_request("proc:fixture");
    let response = service.submit(&request, &mut ports, 1_000_000);

    match response {
        ClientResponseDto::Error {
            error: ClientErrorDto::Infrastructure { retryable, .. },
        } => {
            assert!(retryable, "M60StoreUnavailable should be retryable");
        }
        _ => panic!("expected Infrastructure error, got {response:?}"),
    }

    let record = service
        .store()
        .get("command:fixture")
        .unwrap()
        .expect("record should exist after abandon");
    assert!(
        matches!(record.state, RecordState::Pending { .. }),
        "record should be Pending after M71 failure abandon, got {:?}",
        record.state
    );
}

const SOURCE: &str = include_str!("../../platform-core/src/request_context.rs");
