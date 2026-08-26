#![allow(clippy::unwrap_used)]

//! Integration tests for the M80 client-core reducer and loopback transport.
//!
//! Coverage:
//! - every [`ClientResponseDto`] variant (5);
//! - every [`M71OutcomeDto`] (6) with its valid lineage pairings;
//! - every [`FreshnessDto`] (2), [`CannotVerifyReasonDto`] (4), [`M71LineageDto`] (3);
//! - every [`RedactionDto`] (3) carried by `Available`;
//! - every [`ClientErrorDto`] (3) and [`WireErrorClassDto`] (14) mapped to an exit class;
//! - one real loopback TCP framing round-trip through `send_intent`;
//! - transport connect-refused → `Unavailable`;
//! - `Unavailable` equivalence: server-typed denial and transport failure reduce
//!   to the same state shape and exit code (no existence side channel).

use std::io::Write;
use std::net::TcpListener;
use std::thread;
use std::time::Duration;

use ustc_campus_agent_client_core::wire::{
    ActorIntentDto, AdmittedActorDto, AffairsGetPayloadDto, ClientIntentDto, ClientProvenanceDto,
    ClientResponseDto, ConflictDetailDto, DispatchCapsuleBodyV2, EchoPayloadDto, FreshnessDto,
    FrozenPrerequisitesDto, M10WireErrorDto, M71LineageDto, M71OutcomeDto, M71TerminalDto,
    ProcedureViewDto, RedactionDto, RetryabilityDto, UnixMillis, WireErrorClassDto, read_frame,
    write_frame,
};
use ustc_campus_agent_client_core::{
    ClientState, Endpoint, Origin, TransportError, WireText, exit_class, provenance,
    reduce_response, reduce_transport_failure, render_result, send_intent,
};

fn tx(value: &str) -> WireText {
    WireText::parse(value).unwrap()
}

fn ms(value: i64) -> UnixMillis {
    UnixMillis::new(value)
}

fn prov() -> ClientProvenanceDto {
    provenance("ustc-agent/test", "cli", "client-protocol/v0.1").unwrap()
}

fn minimal_view() -> ProcedureViewDto {
    ProcedureViewDto {
        procedure_id: tx("proc-1"),
        artifact_id: tx("art-1"),
        title: tx("Title"),
        audience_tags: vec![tx("students")],
        board_id: tx("board-1"),
        board_policy_version: 1,
        prerequisites: vec![],
        ordered_steps: vec![],
        deadlines: vec![],
        effective_interval: None,
        entry_points: vec![],
        contacts: vec![],
        evidence: ustc_campus_agent_client_core::wire::EvidenceViewDto {
            valid_interval: ustc_campus_agent_client_core::wire::ValidityHorizonDto::Unknown,
            observed_at: ms(1000),
            known_at: ms(1000),
            reviewed_at: ms(1000),
            last_verified_at: ms(1000),
            assessments: vec![],
            projection: ustc_campus_agent_client_core::wire::ProjectionMetadataDto::Complete,
        },
        lookup_path: ustc_campus_agent_client_core::wire::LookupPathDto::ExactId,
        conflict_state: ustc_campus_agent_client_core::wire::ConflictStateDto::Resolved,
        uncertainty_state: tx("none"),
    }
}

fn terminal(outcome: M71OutcomeDto, lineage: M71LineageDto) -> M71TerminalDto {
    M71TerminalDto::try_new(outcome, lineage).unwrap()
}

fn accepted(terminal: M71TerminalDto, capability: Option<&str>) -> ClientResponseDto {
    ClientResponseDto::Accepted {
        command_id: tx("cmd-1"),
        terminal: Box::new(terminal),
        public_capability: capability.map(tx),
    }
}

fn available(terminal: M71TerminalDto, redaction: RedactionDto) -> ClientResponseDto {
    ClientResponseDto::Available {
        command_id: tx("cmd-1"),
        terminal: Box::new(terminal),
        redaction,
    }
}

fn verified_lineage() -> M71LineageDto {
    M71LineageDto::Verified {
        materialization_receipt_id: tx("mr-1"),
        evidence_set_digest: tx("digest-1"),
        revision_count: 1,
        verifier_id: tx("verifier-1"),
        verified_at: ms(2000),
        evidence_contract_version: 1,
    }
}

fn unverified_lineage(reason: &str) -> M71LineageDto {
    M71LineageDto::Unverified {
        materialization_receipt_id: tx("mr-1"),
        reason: tx(reason),
    }
}

fn not_required_lineage(reason: &str) -> M71LineageDto {
    M71LineageDto::NotRequired {
        materialization_receipt_id: tx("mr-1"),
        reason: tx(reason),
    }
}

fn found(freshness: FreshnessDto) -> M71OutcomeDto {
    M71OutcomeDto::Found {
        view: Box::new(minimal_view()),
        freshness,
        as_of: ms(3000),
    }
}

#[test]
fn reduces_accepted_found_fresh() {
    let t = terminal(found(FreshnessDto::Fresh), verified_lineage());
    let state = reduce_response(accepted(t, Some("cap-1")));
    let rendered = render_result(&state, Origin::Server, &prov());
    assert_eq!(exit_class(&state).code(), 0);
    assert!(rendered.contains("\"outcome_class\":\"found\""));
    assert!(rendered.contains("\"lineage_class\":\"verified\""));
    assert!(rendered.contains("\"freshness_class\":\"fresh\""));
    assert!(rendered.contains("\"public_capability\":\"cap-1\""));
    assert!(
        rendered.contains("\"terminal\":{\"outcome\":"),
        "canonical result must embed the full M71 terminal: {rendered}"
    );
}

#[test]
fn reduces_accepted_found_stale() {
    let t = terminal(
        found(FreshnessDto::Stale {
            last_verified_at: ms(1000),
            max_fresh_age_seconds: 3600,
            max_presentable_age_seconds: 86400,
        }),
        verified_lineage(),
    );
    let state = reduce_response(accepted(t, None));
    assert_eq!(exit_class(&state).code(), 0);
    let rendered = render_result(&state, Origin::Server, &prov());
    assert!(rendered.contains("\"freshness_class\":\"stale\""));
}

#[test]
fn reduces_not_yet_known() {
    let t = terminal(
        M71OutcomeDto::NotYetKnown {
            procedure_id: tx("proc-1"),
            known_at: ms(5000),
            as_of: ms(3000),
            cutoff_source: ustc_campus_agent_client_core::wire::CutoffSourceDto::CallerProvided,
        },
        not_required_lineage("known_after_cutoff"),
    );
    let state = reduce_response(accepted(t, None));
    assert_eq!(exit_class(&state).code(), 0);
    assert!(
        render_result(&state, Origin::Server, &prov())
            .contains("\"outcome_class\":\"not_yet_known\"")
    );
}

#[test]
fn reduces_archived() {
    let t = terminal(
        M71OutcomeDto::Archived {
            procedure_id: tx("proc-1"),
            archived_at: ms(2000),
        },
        not_required_lineage("archived_without_current_artifact"),
    );
    let state = reduce_response(accepted(t, None));
    assert_eq!(exit_class(&state).code(), 0);
    assert!(
        render_result(&state, Origin::Server, &prov()).contains("\"outcome_class\":\"archived\"")
    );
}

#[test]
fn reduces_not_found() {
    let t = terminal(
        M71OutcomeDto::NotFound {
            procedure_id: tx("proc-1"),
        },
        not_required_lineage("no_visible_artifact"),
    );
    let state = reduce_response(accepted(t, None));
    assert_eq!(exit_class(&state).code(), 0);
    assert!(
        render_result(&state, Origin::Server, &prov()).contains("\"outcome_class\":\"not_found\"")
    );
}

#[test]
fn reduces_conflict() {
    let t = terminal(
        M71OutcomeDto::Conflict {
            procedure_id: tx("proc-1"),
            conflict: ConflictDetailDto {
                conflict_kind: tx("contradiction"),
                description: tx("desc"),
                evidence_refs: vec![],
            },
        },
        verified_lineage(),
    );
    let state = reduce_response(accepted(t, None));
    assert_eq!(exit_class(&state).code(), 0);
    assert!(
        render_result(&state, Origin::Server, &prov()).contains("\"outcome_class\":\"conflict\"")
    );
}

#[test]
fn reduces_cannot_verify_all_four_reasons() {
    let cases = [
        (
            M71OutcomeDto::CannotVerify {
                procedure_id: tx("proc-1"),
                reason:
                    ustc_campus_agent_client_core::wire::CannotVerifyReasonDto::SourceRevisionUnverified,
            },
            unverified_lineage("missing_revision"),
            "source_revision_unverified",
        ),
        (
            M71OutcomeDto::CannotVerify {
                procedure_id: tx("proc-1"),
                reason:
                    ustc_campus_agent_client_core::wire::CannotVerifyReasonDto::EffectiveIntervalMissing,
            },
            unverified_lineage("effective_interval_missing"),
            "effective_interval_missing",
        ),
        (
            M71OutcomeDto::CannotVerify {
                procedure_id: tx("proc-1"),
                reason:
                    ustc_campus_agent_client_core::wire::CannotVerifyReasonDto::LastVerifiedStaleBeyondPolicy,
            },
            verified_lineage(),
            "last_verified_stale_beyond_policy",
        ),
        (
            M71OutcomeDto::CannotVerify {
                procedure_id: tx("proc-1"),
                reason:
                    ustc_campus_agent_client_core::wire::CannotVerifyReasonDto::PublicEvidenceProjectionOverflow {
                        mandatory_count: 9,
                    },
            },
            verified_lineage(),
            "public_evidence_projection_overflow",
        ),
    ];
    for (outcome, lineage, expected_reason) in cases {
        let t = terminal(outcome, lineage);
        let state = reduce_response(accepted(t, None));
        assert_eq!(exit_class(&state).code(), 0);
        let rendered = render_result(&state, Origin::Server, &prov());
        assert!(
            rendered.contains(&format!("\"reason_class\":\"{expected_reason}\"")),
            "expected reason_class {expected_reason} in {rendered}"
        );
    }
}

#[test]
fn reduces_available_all_redactions() {
    let t = terminal(found(FreshnessDto::Fresh), verified_lineage());
    for (redaction, label) in [
        (RedactionDto::Public, "public"),
        (RedactionDto::AuthenticatedOwner, "authenticated_owner"),
        (RedactionDto::Operator, "operator"),
    ] {
        let state = reduce_response(available(t.clone(), redaction));
        assert_eq!(exit_class(&state).code(), 0);
        let rendered = render_result(&state, Origin::Server, &prov());
        assert!(rendered.contains(&format!("\"redaction\":\"{label}\"")));
    }
}

#[test]
fn reduces_incomplete() {
    let state = reduce_response(ClientResponseDto::Incomplete {
        command_id: tx("cmd-1"),
        retry_not_before: ms(9000),
    });
    assert_eq!(exit_class(&state).code(), 8);
    let rendered = render_result(&state, Origin::Server, &prov());
    assert!(rendered.contains("\"kind\":\"incomplete\""));
    assert!(rendered.contains("\"retry_not_before\":9000"));
}

#[test]
fn reduces_unavailable() {
    let state = reduce_response(ClientResponseDto::Unavailable);
    assert_eq!(exit_class(&state).code(), 6);
    let rendered = render_result(&state, Origin::Server, &prov());
    assert!(rendered.contains("\"kind\":\"unavailable\""));
}

#[test]
fn reduces_internal_invariant_error() {
    let state = reduce_response(ClientResponseDto::Error {
        error: ustc_campus_agent_client_core::wire::ClientErrorDto::InternalInvariant {
            wire_code: tx("internal_invariant"),
        },
    });
    assert_eq!(exit_class(&state).code(), 9);
}

#[test]
fn reduces_infrastructure_error_retryable_and_not() {
    let retryable = reduce_response(ClientResponseDto::Error {
        error: ustc_campus_agent_client_core::wire::ClientErrorDto::Infrastructure {
            retryable: true,
            wire_code: tx("infra_transient"),
        },
    });
    assert_eq!(exit_class(&retryable).code(), 6);

    let permanent = reduce_response(ClientResponseDto::Error {
        error: ustc_campus_agent_client_core::wire::ClientErrorDto::Infrastructure {
            retryable: false,
            wire_code: tx("infra_hard"),
        },
    });
    assert_eq!(exit_class(&permanent).code(), 9);
}

#[test]
fn reduces_every_admission_error_class_to_correct_exit() {
    use WireErrorClassDto::*;
    // (class, expected_exit_code)
    let cases: [(WireErrorClassDto, i32); 14] = [
        (IdempotencyStoreUnavailable, 6),
        (ConflictingEnvelope, 7),
        (DescriptorSnapshotAbsent, 7),
        (DescriptorSnapshotMismatch, 7),
        (PolicyDenied, 4),
        (PolicyExpired, 7),
        (SessionNotFound, 3),
        (SessionIdMismatch, 3),
        (SessionNotAdmitted, 3),
        (CapabilityMissing, 4),
        (CapabilityDisabled, 4),
        (CapabilityRevoked, 4),
        (InfrastructurePortUnavailable, 6),
        (MalformedCommand, 2),
    ];
    for (class, expected) in cases {
        let error = wire_error_for(class);
        let state = reduce_response(ClientResponseDto::Error {
            error: ustc_campus_agent_client_core::wire::ClientErrorDto::Admission { error },
        });
        let actual = exit_class(&state).code();
        assert_eq!(
            actual, expected,
            "class {class:?} should map to exit {expected}, got {actual}"
        );
    }
}

fn wire_error_for(class: WireErrorClassDto) -> M10WireErrorDto {
    use RetryabilityDto::{NotRetryable, Retryable, RetryableAfterChange};
    use WireErrorClassDto::*;
    let (retryability, code) = match class {
        IdempotencyStoreUnavailable => (Retryable, "idempotency_store_unavailable"),
        ConflictingEnvelope => (RetryableAfterChange, "conflicting_envelope"),
        DescriptorSnapshotAbsent => (NotRetryable, "descriptor_snapshot_absent"),
        DescriptorSnapshotMismatch => (NotRetryable, "descriptor_snapshot_mismatch"),
        PolicyDenied => (NotRetryable, "policy_denied"),
        PolicyExpired => (RetryableAfterChange, "policy_expired"),
        SessionNotFound => (RetryableAfterChange, "session_not_found"),
        SessionIdMismatch => (NotRetryable, "session_id_mismatch"),
        SessionNotAdmitted => (RetryableAfterChange, "session_not_admitted"),
        CapabilityMissing => (RetryableAfterChange, "capability_missing"),
        CapabilityDisabled => (RetryableAfterChange, "capability_disabled"),
        CapabilityRevoked => (NotRetryable, "capability_revoked"),
        InfrastructurePortUnavailable => (Retryable, "infrastructure_port_unavailable"),
        MalformedCommand => (RetryableAfterChange, "malformed_command"),
    };
    let echo = match class {
        IdempotencyStoreUnavailable | DescriptorSnapshotAbsent | InfrastructurePortUnavailable => {
            EchoPayloadDto::Operation {
                operation_id: tx("affairs.get"),
            }
        }
        ConflictingEnvelope => EchoPayloadDto::Envelope {
            operation_id: tx("affairs.get"),
            idempotency_key: tx("idem-1"),
        },
        DescriptorSnapshotMismatch => EchoPayloadDto::SnapshotMismatch {
            command_operation_id: tx("affairs.get"),
            snapshot_operation_id: tx("affairs.get"),
        },
        PolicyDenied => EchoPayloadDto::PolicyDenied {
            operation_id: tx("affairs.get"),
            permission_class: tx("public-read"),
        },
        PolicyExpired => EchoPayloadDto::PolicyExpired {
            operation_id: tx("affairs.get"),
            policy_snapshot_id: tx("policy-1"),
        },
        SessionNotFound => EchoPayloadDto::SessionId {
            requested_session_id: tx("session-1"),
        },
        SessionIdMismatch => EchoPayloadDto::SessionMismatch {
            requested_session_id: tx("session-1"),
            loaded_session_id: tx("session-2"),
        },
        SessionNotAdmitted => EchoPayloadDto::SessionNotAdmitted {
            requested_session_id: tx("session-1"),
            observed_at: ms(1_000),
        },
        CapabilityMissing | CapabilityDisabled | CapabilityRevoked => EchoPayloadDto::Capability {
            operation_id: tx("affairs.get"),
            actor_kind: tx("public"),
        },
        MalformedCommand => EchoPayloadDto::None,
    };
    M10WireErrorDto::try_new(class, retryability, tx(code), echo).unwrap()
}

#[test]
fn unavailable_equivalence_server_and_transport() {
    let server_state = reduce_response(ClientResponseDto::Unavailable);
    let transport_state = reduce_transport_failure(TransportError::Unavailable);
    assert_eq!(exit_class(&server_state).code(), 6);
    assert_eq!(exit_class(&transport_state).code(), 6);
    let server_rendered = render_result(&server_state, Origin::Server, &prov());
    let transport_rendered = render_result(&transport_state, Origin::Transport, &prov());
    assert_eq!(
        server_rendered.replace("\"origin\":\"server\"", "\"origin\":\"X\""),
        transport_rendered.replace("\"origin\":\"transport\"", "\"origin\":\"X\""),
        "server-typed and transport Unavailable must differ only in origin"
    );
}

#[test]
fn no_existence_side_channel_three_unavailable_inputs_identical() {
    let state = reduce_response(ClientResponseDto::Unavailable);
    let rendered_a = render_result(&state, Origin::Server, &prov());
    let state_b = reduce_response(ClientResponseDto::Unavailable);
    let rendered_b = render_result(&state_b, Origin::Server, &prov());
    assert_eq!(rendered_a, rendered_b);
}

#[test]
fn transport_connect_refused_reduces_to_unavailable() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    drop(listener);
    let endpoint = Endpoint::parse(addr.to_string()).unwrap();
    let intent = public_intent();
    let result = send_intent(&endpoint, Duration::from_secs(2), &intent);
    assert!(
        matches!(result, Err(TransportError::Unavailable)),
        "connect to a closed port must be Unavailable, got {result:?}"
    );
    let state = reduce_transport_failure(result.unwrap_err());
    assert_eq!(exit_class(&state).code(), 6);
}

#[test]
fn write_failure_source_classified_as_outcome_unknown() {
    let src = include_str!("../src/transport.rs");
    let body = src
        .split("fn write_failure")
        .nth(1)
        .and_then(|s| s.split("fn read_failure").next())
        .unwrap_or("");
    assert!(
        body.contains("TransportError::OutcomeUnknown"),
        "write_failure must classify as OutcomeUnknown: {body}"
    );
    assert!(
        !body.contains("TransportError::Unavailable"),
        "write_failure must NOT classify as Unavailable (bytes may have been sent): {body}"
    );
}

#[test]
fn endpoint_rejects_non_loopback_through_public_api() {
    assert!(Endpoint::parse("8.8.8.8:8080").is_err());
    assert!(Endpoint::parse("10.0.0.1:8080").is_err());
    assert!(Endpoint::parse("localhost:8080").is_err());
    assert!(Endpoint::parse("[::1]:8080").is_ok());
    assert!(Endpoint::parse("127.0.0.1:8080").is_ok());
}

#[test]
fn real_loopback_framing_round_trip() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let expected_terminal = terminal(found(FreshnessDto::Fresh), verified_lineage());
    let canned_response = ClientResponseDto::Accepted {
        command_id: tx("cmd-loopback"),
        terminal: Box::new(expected_terminal.clone()),
        public_capability: Some(tx("cap-loopback")),
    };
    let server_handle = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let intent: ClientIntentDto = read_frame(&stream).unwrap();
        assert!(matches!(intent, ClientIntentDto::SubmitAffairsGet { .. }));
        write_frame(&mut stream, &canned_response).unwrap();
        stream.flush().unwrap();
    });
    let endpoint = Endpoint::parse(addr.to_string()).unwrap();
    let intent = public_intent();
    let response = send_intent(&endpoint, Duration::from_secs(5), &intent).unwrap();
    server_handle.join().unwrap();
    let state = reduce_response(response);
    assert_eq!(exit_class(&state).code(), 0);
    let rendered = render_result(&state, Origin::Server, &prov());
    assert!(rendered.contains("\"command_id\":\"cmd-loopback\""));
    assert!(rendered.contains("\"public_capability\":\"cap-loopback\""));
    assert!(rendered.contains("\"outcome_class\":\"found\""));
}

fn public_intent() -> ClientIntentDto {
    let provenance = prov();
    ustc_campus_agent_client_core::public_affairs_get(
        "req-1",
        "corr-1",
        None::<String>,
        None::<String>,
        provenance,
        "digest-1",
        "proc-1",
        None,
    )
    .unwrap()
}

#[test]
fn capsule_round_trip_through_wire() {
    let body = DispatchCapsuleBodyV2::try_new(
        tx("cmd-1"),
        tx("corr-1"),
        AdmittedActorDto::Public,
        AffairsGetPayloadDto {
            procedure_id: tx("proc-1"),
            as_of: None,
        },
        tx("snap-1"),
        tx("digest-1"),
        1,
        FrozenPrerequisitesDto {
            policy_snapshot_id: tx("pol-1"),
            observed_at: ms(1000),
            session_id: None,
            admitted_operation_id: tx("affairs.get"),
        },
    )
    .unwrap();
    let json = serde_json::to_string(&body).unwrap();
    let decoded: DispatchCapsuleBodyV2 = serde_json::from_str(&json).unwrap();
    assert_eq!(decoded.command_id(), body.command_id());
    assert_eq!(decoded.schema_version(), 2);
}

#[test]
fn authenticated_intent_carries_session() {
    let provenance = prov();
    let intent = ustc_campus_agent_client_core::authenticated_affairs_get(
        "req-1",
        "corr-1",
        None::<String>,
        None::<String>,
        provenance,
        "digest-1",
        "proc-1",
        None,
        "session-1",
    )
    .unwrap();
    match intent {
        ClientIntentDto::SubmitAffairsGet { request } => match request.actor {
            ActorIntentDto::Authenticated { session_id } => {
                assert_eq!(session_id.as_str(), "session-1");
            }
            ActorIntentDto::Public => panic!("expected authenticated actor"),
        },
        ClientIntentDto::Lookup { .. } => panic!("expected submit intent"),
    }
}

#[test]
fn lookup_intents_build_correct_viewer() {
    let by_cap = ustc_campus_agent_client_core::lookup_by_capability("cmd-1", "cap-1").unwrap();
    let as_owner =
        ustc_campus_agent_client_core::lookup_as_owner("cmd-1", "tenant-1", "user-1").unwrap();
    let as_op = ustc_campus_agent_client_core::lookup_as_operator("cmd-1", "grant-1").unwrap();
    assert!(matches!(
        by_cap,
        ClientIntentDto::Lookup {
            viewer: ustc_campus_agent_client_core::wire::ViewerAuthorizationDto::PublicCapability { .. },
            ..
        }
    ));
    assert!(matches!(
        as_owner,
        ClientIntentDto::Lookup {
            viewer:
                ustc_campus_agent_client_core::wire::ViewerAuthorizationDto::AuthenticatedOwner { .. },
            ..
        }
    ));
    assert!(matches!(
        as_op,
        ClientIntentDto::Lookup {
            viewer: ustc_campus_agent_client_core::wire::ViewerAuthorizationDto::Operator { .. },
            ..
        }
    ));
}

#[test]
fn render_result_is_deterministic_across_calls() {
    let state = reduce_response(ClientResponseDto::Unavailable);
    let a = render_result(&state, Origin::Server, &prov());
    let b = render_result(&state, Origin::Server, &prov());
    assert_eq!(a, b);
}

#[test]
fn submit_affairs_get_dto_serializes_with_all_fields() {
    let provenance = prov();
    let intent = ustc_campus_agent_client_core::public_affairs_get(
        "req-1",
        "corr-1",
        Some("caus-1"),
        Some("idem-1"),
        provenance,
        "digest-1",
        "proc-1",
        Some(ms(42)),
    )
    .unwrap();
    let json = serde_json::to_string(&intent).unwrap();
    assert!(json.contains("\"causation_id\":\"caus-1\""));
    assert!(json.contains("\"idempotency_key\":\"idem-1\""));
    assert!(json.contains("\"as_of\":42"));
    assert!(json.contains("\"kind\":\"submit_affairs_get\""));
}

#[test]
fn debug_does_not_leak_capability_bearer() {
    let terminal_dto = terminal(found(FreshnessDto::Fresh), verified_lineage());
    let response = ClientResponseDto::Accepted {
        command_id: tx("cmd-secret-test"),
        terminal: Box::new(terminal_dto),
        public_capability: Some(tx("cap-secret-bearer-xyz")),
    };
    let state = reduce_response(response);

    let state_debug = format!("{state:?}");
    assert!(
        !state_debug.contains("cap-secret-bearer-xyz"),
        "ClientState Debug must not leak capability bearer: {state_debug}"
    );

    match &state {
        ClientState::Terminal { terminal_kind, .. } => {
            let kind_debug = format!("{terminal_kind:?}");
            assert!(
                !kind_debug.contains("cap-secret-bearer-xyz"),
                "TerminalKind Debug must not leak capability bearer: {kind_debug}"
            );
            assert!(
                kind_debug.contains("Accepted") && kind_debug.contains("Some"),
                "TerminalKind Debug must preserve variant/class info: {kind_debug}"
            );
        }
        other => panic!("expected Terminal, got {other:?}"),
    }

    let response_none = ClientResponseDto::Accepted {
        command_id: tx("cmd-none-test"),
        terminal: Box::new(terminal(found(FreshnessDto::Fresh), verified_lineage())),
        public_capability: None,
    };
    let state_none = reduce_response(response_none);
    match &state_none {
        ClientState::Terminal { terminal_kind, .. } => {
            let kind_debug = format!("{terminal_kind:?}");
            assert!(
                kind_debug.contains("Accepted") && kind_debug.contains("None"),
                "TerminalKind Debug must show None when no capability: {kind_debug}"
            );
        }
        other => panic!("expected Terminal, got {other:?}"),
    }

    let available_response = ClientResponseDto::Available {
        command_id: tx("cmd-avail-test"),
        terminal: Box::new(terminal(found(FreshnessDto::Fresh), verified_lineage())),
        redaction: RedactionDto::Public,
    };
    let state_avail = reduce_response(available_response);
    match &state_avail {
        ClientState::Terminal { terminal_kind, .. } => {
            let kind_debug = format!("{terminal_kind:?}");
            assert!(
                kind_debug.contains("Available") && kind_debug.contains("Public"),
                "TerminalKind Available Debug must preserve redaction class: {kind_debug}"
            );
        }
        other => panic!("expected Terminal, got {other:?}"),
    }
}
