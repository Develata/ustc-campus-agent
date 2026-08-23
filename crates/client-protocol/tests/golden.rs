#![allow(clippy::unwrap_used)]

//! Golden wire-format tests: lock canonical JSON for request, capsule, terminal, and error.
//!
//! These goldens pin the exact field ordering, tag names, and structure that crosses the wire.
//! Any change to the JSON shape is a wire-format change and MUST be intentional.

use WireText as T;
use ustc_campus_agent_client_protocol::*;

fn proc_view() -> ProcedureViewDto {
    ProcedureViewDto {
        procedure_id: T::parse("proc:scholarship-2025").unwrap(),
        artifact_id: T::parse("art:001").unwrap(),
        title: T::parse("Scholarship Application").unwrap(),
        audience_tags: vec![T::parse("undergraduate").unwrap()],
        board_id: T::parse("board:main").unwrap(),
        board_policy_version: 3,
        prerequisites: vec![PrerequisiteDto {
            condition: T::parse("enrolled").unwrap(),
            source_subject: Some(T::parse("registrar").unwrap()),
        }],
        ordered_steps: vec![StepDto {
            ordinal: 1,
            instruction: T::parse("Submit form").unwrap(),
        }],
        deadlines: vec![DeadlineDto {
            label: T::parse("final").unwrap(),
            kind: T::parse("hard").unwrap(),
            at: Some(UnixMillis::new(1_700_000_000_000)),
        }],
        effective_interval: Some(IntervalDto {
            from: Some(UnixMillis::new(1_700_000_000_000)),
            to: Some(UnixMillis::new(1_800_000_000_000)),
        }),
        entry_points: vec![EntryPointDto {
            label: T::parse("portal").unwrap(),
            url: Some(T::parse("https://example.ustc.edu.cn").unwrap()),
            contact_ref: T::parse("contact:office").unwrap(),
        }],
        contacts: vec![ContactDto {
            contact_ref: T::parse("contact:office").unwrap(),
            name: T::parse("Financial Aid Office").unwrap(),
            channel: T::parse("email").unwrap(),
            source_id: T::parse("src:001").unwrap(),
        }],
        evidence: EvidenceViewDto {
            valid_interval: ValidityHorizonDto::KnownInterval {
                from: Some(UnixMillis::new(1_700_000_000_000)),
                to: Some(UnixMillis::new(1_800_000_000_000)),
            },
            observed_at: UnixMillis::new(1_700_000_000_000),
            known_at: UnixMillis::new(1_700_000_000_000),
            reviewed_at: UnixMillis::new(1_700_000_000_000),
            last_verified_at: UnixMillis::new(1_700_000_000_000),
            assessments: vec![EvidenceAssessmentDto {
                authority: T::parse("registrar").unwrap(),
                subject: T::parse("scholarship").unwrap(),
                source_id: T::parse("src:001").unwrap(),
                reviewed_at: UnixMillis::new(1_700_000_000_000),
                last_verified_at: UnixMillis::new(1_700_000_000_000),
            }],
            projection: ProjectionMetadataDto::Complete,
        },
        lookup_path: LookupPathDto::ExactId,
        conflict_state: ConflictStateDto::Resolved,
        uncertainty_state: T::parse("none").unwrap(),
    }
}

#[test]
fn golden_submit_request_json() {
    let request = SubmitAffairsGetDto {
        request_id: T::parse("req:fixture").unwrap(),
        correlation_id: T::parse("corr:fixture").unwrap(),
        causation_id: None,
        idempotency_key: Some(T::parse("idem:fixture").unwrap()),
        actor: ActorIntentDto::Public,
        provenance: ClientProvenanceDto {
            build: T::parse("test").unwrap(),
            target: T::parse("test").unwrap(),
            protocol: T::parse("v2").unwrap(),
        },
        payload_digest: T::parse(
            "0000000000000000000000000000000000000000000000000000000000000000",
        )
        .unwrap(),
        procedure_id: T::parse("proc:fixture").unwrap(),
        as_of: None,
    };
    let json = serde_json::to_string(&request).unwrap();
    assert_eq!(
        json,
        r#"{"request_id":"req:fixture","correlation_id":"corr:fixture","causation_id":null,"idempotency_key":"idem:fixture","actor":{"kind":"public"},"provenance":{"build":"test","target":"test","protocol":"v2"},"payload_digest":"0000000000000000000000000000000000000000000000000000000000000000","procedure_id":"proc:fixture","as_of":null}"#
    );
}

#[test]
fn golden_capsule_json() {
    let capsule = DispatchCapsuleBodyV2::try_new(
        T::parse("cmd:fixture:001").unwrap(),
        T::parse("corr:fixture:001").unwrap(),
        AdmittedActorDto::Public,
        AffairsGetPayloadDto {
            procedure_id: T::parse("proc:fixture").unwrap(),
            as_of: Some(UnixMillis::new(1_700_000_000_000)),
        },
        T::parse(
            "descriptor:v0:1:0000000000000000000000000000000000000000000000000000000000000000",
        )
        .unwrap(),
        T::parse("0000000000000000000000000000000000000000000000000000000000000000").unwrap(),
        1,
        FrozenPrerequisitesDto {
            policy_snapshot_id: T::parse("policy:fixture:v1").unwrap(),
            observed_at: UnixMillis::new(1_700_000_000_000),
            session_id: None,
            admitted_operation_id: T::parse("affairs.get").unwrap(),
        },
    )
    .unwrap();
    let json = serde_json::to_string(&capsule).unwrap();
    assert_eq!(
        json,
        r#"{"schema_version":2,"command_id":"cmd:fixture:001","correlation_id":"corr:fixture:001","dispatch_identity":"dispatch:v2:cmd:fixture:001","admitted_actor":{"kind":"public"},"affairs_get":{"procedure_id":"proc:fixture","as_of":1700000000000},"descriptor_snapshot_id":"descriptor:v0:1:0000000000000000000000000000000000000000000000000000000000000000","descriptor_content_digest":"0000000000000000000000000000000000000000000000000000000000000000","descriptor_snapshot_version":1,"frozen_prerequisites":{"policy_snapshot_id":"policy:fixture:v1","observed_at":1700000000000,"session_id":null,"admitted_operation_id":"affairs.get"}}"#
    );
}

#[test]
fn golden_not_found_terminal_json() {
    let terminal = M71TerminalDto::try_new(
        M71OutcomeDto::NotFound {
            procedure_id: T::parse("proc:missing").unwrap(),
        },
        M71LineageDto::NotRequired {
            materialization_receipt_id: T::parse("receipt:002").unwrap(),
            reason: T::parse("no_visible_artifact").unwrap(),
        },
    )
    .unwrap();
    let json = serde_json::to_string(&terminal).unwrap();
    assert_eq!(
        json,
        r#"{"outcome":{"kind":"not_found","procedure_id":"proc:missing"},"lineage":{"kind":"not_required","materialization_receipt_id":"receipt:002","reason":"no_visible_artifact"}}"#
    );
}

#[test]
fn golden_cannot_verify_terminal_json() {
    let terminal = M71TerminalDto::try_new(
        M71OutcomeDto::CannotVerify {
            procedure_id: T::parse("proc:stale").unwrap(),
            reason: CannotVerifyReasonDto::LastVerifiedStaleBeyondPolicy,
        },
        M71LineageDto::Verified {
            materialization_receipt_id: T::parse("receipt:003").unwrap(),
            evidence_set_digest: T::parse(
                "0000000000000000000000000000000000000000000000000000000000000000",
            )
            .unwrap(),
            revision_count: 1,
            verifier_id: T::parse("verifier:m00").unwrap(),
            verified_at: UnixMillis::new(1_700_000_000_000),
            evidence_contract_version: 1,
        },
    )
    .unwrap();
    let json = serde_json::to_string(&terminal).unwrap();
    assert_eq!(
        json,
        r#"{"outcome":{"kind":"cannot_verify","procedure_id":"proc:stale","reason":{"kind":"last_verified_stale_beyond_policy"}},"lineage":{"kind":"verified","materialization_receipt_id":"receipt:003","evidence_set_digest":"0000000000000000000000000000000000000000000000000000000000000000","revision_count":1,"verifier_id":"verifier:m00","verified_at":1700000000000,"evidence_contract_version":1}}"#
    );
}

#[test]
fn golden_wire_error_json() {
    let error = M10WireErrorDto::try_new(
        WireErrorClassDto::PolicyDenied,
        RetryabilityDto::NotRetryable,
        T::parse("policy_denied").unwrap(),
        EchoPayloadDto::PolicyDenied {
            operation_id: T::parse("affairs.get").unwrap(),
            permission_class: T::parse("public_read").unwrap(),
        },
    )
    .unwrap();
    let json = serde_json::to_string(&error).unwrap();
    assert_eq!(
        json,
        r#"{"class":"policy_denied","retryability":"not_retryable","wire_code":"policy_denied","echo":{"kind":"policy_denied","operation_id":"affairs.get","permission_class":"public_read"}}"#
    );
}

#[test]
fn golden_client_response_unavailable_json() {
    let response = ClientResponseDto::Unavailable;
    let json = serde_json::to_string(&response).unwrap();
    assert_eq!(json, r#"{"kind":"unavailable"}"#);
}

#[test]
fn golden_client_response_incomplete_json() {
    let response = ClientResponseDto::Incomplete {
        command_id: T::parse("cmd:001").unwrap(),
        retry_not_before: UnixMillis::new(1_700_000_000_000),
    };
    let json = serde_json::to_string(&response).unwrap();
    assert_eq!(
        json,
        r#"{"kind":"incomplete","command_id":"cmd:001","retry_not_before":1700000000000}"#
    );
}

#[test]
fn golden_client_response_accepted_json() {
    let terminal = M71TerminalDto::try_new(
        M71OutcomeDto::NotFound {
            procedure_id: T::parse("proc:missing").unwrap(),
        },
        M71LineageDto::NotRequired {
            materialization_receipt_id: T::parse("receipt:002").unwrap(),
            reason: T::parse("no_visible_artifact").unwrap(),
        },
    )
    .unwrap();
    let response = ClientResponseDto::Accepted {
        command_id: T::parse("cmd:001").unwrap(),
        terminal: Box::new(terminal),
        public_capability: Some(T::parse("cap:abc").unwrap()),
    };
    let json = serde_json::to_string(&response).unwrap();
    assert_eq!(
        json,
        r#"{"kind":"accepted","command_id":"cmd:001","terminal":{"outcome":{"kind":"not_found","procedure_id":"proc:missing"},"lineage":{"kind":"not_required","materialization_receipt_id":"receipt:002","reason":"no_visible_artifact"}},"public_capability":"cap:abc"}"#
    );
}

#[test]
fn golden_found_terminal_round_trips() {
    let terminal = M71TerminalDto::try_new(
        M71OutcomeDto::Found {
            view: Box::new(proc_view()),
            freshness: FreshnessDto::Fresh,
            as_of: UnixMillis::new(1_700_000_000_000),
        },
        M71LineageDto::Verified {
            materialization_receipt_id: T::parse("receipt:001").unwrap(),
            evidence_set_digest: T::parse(
                "abc0000000000000000000000000000000000000000000000000000000000000",
            )
            .unwrap(),
            revision_count: 2,
            verifier_id: T::parse("verifier:m00").unwrap(),
            verified_at: UnixMillis::new(1_700_000_000_000),
            evidence_contract_version: 1,
        },
    )
    .unwrap();
    let json = serde_json::to_string(&terminal).unwrap();
    let back: M71TerminalDto = serde_json::from_str(&json).unwrap();
    assert_eq!(terminal, back);
    assert!(json.contains(r#""kind":"found""#));
    assert!(json.contains(r#""kind":"verified""#));
    assert!(json.contains(r#""kind":"fresh""#));
    assert!(json.contains(r#""kind":"known_interval""#));
    assert!(json.contains(r#""kind":"complete""#));
    assert!(json.contains(r#""kind":"resolved""#));
    assert!(json.contains(r#""lookup_path":"exact_id""#));
}

#[test]
fn golden_archived_terminal_json() {
    let terminal = M71TerminalDto::try_new(
        M71OutcomeDto::Archived {
            procedure_id: T::parse("proc:old").unwrap(),
            archived_at: UnixMillis::new(1_600_000_000_000),
        },
        M71LineageDto::NotRequired {
            materialization_receipt_id: T::parse("receipt:004").unwrap(),
            reason: T::parse("archived_without_current_artifact").unwrap(),
        },
    )
    .unwrap();
    let json = serde_json::to_string(&terminal).unwrap();
    assert_eq!(
        json,
        r#"{"outcome":{"kind":"archived","procedure_id":"proc:old","archived_at":1600000000000},"lineage":{"kind":"not_required","materialization_receipt_id":"receipt:004","reason":"archived_without_current_artifact"}}"#
    );
}

#[test]
fn golden_not_yet_known_terminal_json() {
    let terminal = M71TerminalDto::try_new(
        M71OutcomeDto::NotYetKnown {
            procedure_id: T::parse("proc:future").unwrap(),
            known_at: UnixMillis::new(1_800_000_000_000),
            as_of: UnixMillis::new(1_700_000_000_000),
            cutoff_source: CutoffSourceDto::CallerProvided,
        },
        M71LineageDto::NotRequired {
            materialization_receipt_id: T::parse("receipt:005").unwrap(),
            reason: T::parse("known_after_cutoff").unwrap(),
        },
    )
    .unwrap();
    let json = serde_json::to_string(&terminal).unwrap();
    assert_eq!(
        json,
        r#"{"outcome":{"kind":"not_yet_known","procedure_id":"proc:future","known_at":1800000000000,"as_of":1700000000000,"cutoff_source":"caller_provided"},"lineage":{"kind":"not_required","materialization_receipt_id":"receipt:005","reason":"known_after_cutoff"}}"#
    );
}

#[test]
fn golden_conflict_terminal_json() {
    let terminal = M71TerminalDto::try_new(
        M71OutcomeDto::Conflict {
            procedure_id: T::parse("proc:conflict").unwrap(),
            conflict: ConflictDetailDto {
                conflict_kind: T::parse("duplicate").unwrap(),
                description: T::parse("two sources disagree").unwrap(),
                evidence_refs: vec![T::parse("ref:a").unwrap(), T::parse("ref:b").unwrap()],
            },
        },
        M71LineageDto::Verified {
            materialization_receipt_id: T::parse("receipt:006").unwrap(),
            evidence_set_digest: T::parse(
                "0000000000000000000000000000000000000000000000000000000000000000",
            )
            .unwrap(),
            revision_count: 1,
            verifier_id: T::parse("verifier:m00").unwrap(),
            verified_at: UnixMillis::new(1_700_000_000_000),
            evidence_contract_version: 1,
        },
    )
    .unwrap();
    let json = serde_json::to_string(&terminal).unwrap();
    assert_eq!(
        json,
        r#"{"outcome":{"kind":"conflict","procedure_id":"proc:conflict","conflict":{"conflict_kind":"duplicate","description":"two sources disagree","evidence_refs":["ref:a","ref:b"]}},"lineage":{"kind":"verified","materialization_receipt_id":"receipt:006","evidence_set_digest":"0000000000000000000000000000000000000000000000000000000000000000","revision_count":1,"verifier_id":"verifier:m00","verified_at":1700000000000,"evidence_contract_version":1}}"#
    );
}
