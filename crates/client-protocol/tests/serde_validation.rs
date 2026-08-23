#![allow(clippy::unwrap_used)]

//! Serde validation: deny-unknown-fields, checked constructors, malformed aggregate decode
//! mutations.
//!
//! Every wire DTO rejects unknown fields, empty wire text, control characters, and oversize
//! input. The checked constructors (`WireText::parse`, `DispatchCapsuleBodyV2::try_new`,
//! `M10WireErrorDto::try_new`, `M71TerminalDto::try_new`) enforce cross-field invariants that
//! the derived Serde decode alone cannot.

use ustc_campus_agent_client_protocol::*;

#[test]
fn wire_text_rejects_empty() {
    assert!(WireText::parse("").is_err());
}

#[test]
fn wire_text_rejects_control_characters() {
    assert!(WireText::parse("hello\u{0000}").is_err());
    assert!(WireText::parse("hello\n").is_err());
}

#[test]
fn wire_text_rejects_oversize() {
    let big = "a".repeat(WireText::MAX_BYTES + 1);
    assert!(WireText::parse(&big).is_err());
}

#[test]
fn wire_text_accepts_normal() {
    assert!(WireText::parse("affairs.get").is_ok());
}

#[test]
fn wire_text_debug_redacts_value() {
    let value = WireText::parse("secret-bearer-token").unwrap();
    let debug = format!("{value:?}");
    assert!(!debug.contains("secret-bearer-token"));
    assert!(debug.contains("redacted"));
}

#[test]
fn wire_text_round_trips_through_serde() {
    let value = WireText::parse("proc:fixture:v1").unwrap();
    let json = serde_json::to_string(&value).unwrap();
    assert_eq!(json, "\"proc:fixture:v1\"");
    let back: WireText = serde_json::from_str(&json).unwrap();
    assert_eq!(value, back);
}

#[test]
fn wire_text_serde_rejects_empty_string() {
    assert!(serde_json::from_str::<WireText>("\"\"").is_err());
}

#[test]
fn unix_millis_round_trips() {
    let value = UnixMillis::new(1_700_000_000_000);
    let json = serde_json::to_string(&value).unwrap();
    assert_eq!(json, "1700000000000");
    let back: UnixMillis = serde_json::from_str(&json).unwrap();
    assert_eq!(value, back);
}

// ---------------------------------------------------------------------------
// Capsule validation
// ---------------------------------------------------------------------------

fn valid_capsule_components() -> (
    WireText,
    WireText,
    AdmittedActorDto,
    AffairsGetPayloadDto,
    WireText,
    WireText,
    u64,
    FrozenPrerequisitesDto,
) {
    (
        WireText::parse("cmd:fixture:001").unwrap(),
        WireText::parse("corr:fixture:001").unwrap(),
        AdmittedActorDto::Public,
        AffairsGetPayloadDto {
            procedure_id: WireText::parse("proc:fixture").unwrap(),
            as_of: Some(UnixMillis::new(1_700_000_000_000)),
        },
        WireText::parse(
            "descriptor:v0:1:0000000000000000000000000000000000000000000000000000000000000000",
        )
        .unwrap(),
        WireText::parse("0000000000000000000000000000000000000000000000000000000000000000")
            .unwrap(),
        1,
        FrozenPrerequisitesDto {
            policy_snapshot_id: WireText::parse("policy:fixture:v1").unwrap(),
            observed_at: UnixMillis::new(1_700_000_000_000),
            session_id: None,
            admitted_operation_id: WireText::parse("affairs.get").unwrap(),
        },
    )
}

#[test]
fn capsule_try_new_succeeds_for_valid_public() {
    let (cmd, corr, actor, payload, desc_id, desc_digest, desc_ver, frozen) =
        valid_capsule_components();
    assert!(
        DispatchCapsuleBodyV2::try_new(
            cmd,
            corr,
            actor,
            payload,
            desc_id,
            desc_digest,
            desc_ver,
            frozen
        )
        .is_ok()
    );
}

#[test]
fn capsule_rejects_descriptor_version_zero() {
    let (cmd, corr, actor, payload, desc_id, desc_digest, _, frozen) = valid_capsule_components();
    assert!(matches!(
        DispatchCapsuleBodyV2::try_new(cmd, corr, actor, payload, desc_id, desc_digest, 0, frozen),
        Err(CapsuleValidationError::DescriptorVersionZero)
    ));
}

#[test]
fn capsule_rejects_operation_mismatch() {
    let (cmd, corr, actor, payload, desc_id, desc_digest, desc_ver, _) = valid_capsule_components();
    let frozen = FrozenPrerequisitesDto {
        policy_snapshot_id: WireText::parse("policy:fixture:v1").unwrap(),
        observed_at: UnixMillis::new(1_700_000_000_000),
        session_id: None,
        admitted_operation_id: WireText::parse("not.affairs.get").unwrap(),
    };
    assert!(matches!(
        DispatchCapsuleBodyV2::try_new(
            cmd,
            corr,
            actor,
            payload,
            desc_id,
            desc_digest,
            desc_ver,
            frozen
        ),
        Err(CapsuleValidationError::OperationMismatch)
    ));
}

#[test]
fn capsule_rejects_actor_session_mismatch_public_with_session() {
    let (cmd, corr, _, payload, desc_id, desc_digest, desc_ver, _) = valid_capsule_components();
    let frozen = FrozenPrerequisitesDto {
        policy_snapshot_id: WireText::parse("policy:fixture:v1").unwrap(),
        observed_at: UnixMillis::new(1_700_000_000_000),
        session_id: Some(WireText::parse("session:fixture").unwrap()),
        admitted_operation_id: WireText::parse("affairs.get").unwrap(),
    };
    assert!(matches!(
        DispatchCapsuleBodyV2::try_new(
            cmd,
            corr,
            AdmittedActorDto::Public,
            payload,
            desc_id,
            desc_digest,
            desc_ver,
            frozen
        ),
        Err(CapsuleValidationError::ActorSessionMismatch)
    ));
}

#[test]
fn capsule_rejects_actor_session_mismatch_authenticated_without_session() {
    let (cmd, corr, _, payload, desc_id, desc_digest, desc_ver, _) = valid_capsule_components();
    let frozen = FrozenPrerequisitesDto {
        policy_snapshot_id: WireText::parse("policy:fixture:v1").unwrap(),
        observed_at: UnixMillis::new(1_700_000_000_000),
        session_id: None,
        admitted_operation_id: WireText::parse("affairs.get").unwrap(),
    };
    let actor = AdmittedActorDto::Authenticated {
        tenant_id: WireText::parse("tenant:fixture").unwrap(),
        user_id: WireText::parse("user:fixture").unwrap(),
        session_id: WireText::parse("session:fixture").unwrap(),
    };
    assert!(matches!(
        DispatchCapsuleBodyV2::try_new(
            cmd,
            corr,
            actor,
            payload,
            desc_id,
            desc_digest,
            desc_ver,
            frozen
        ),
        Err(CapsuleValidationError::ActorSessionMismatch)
    ));
}

#[test]
fn capsule_round_trips_through_serde() {
    let (cmd, corr, actor, payload, desc_id, desc_digest, desc_ver, frozen) =
        valid_capsule_components();
    let capsule = DispatchCapsuleBodyV2::try_new(
        cmd,
        corr,
        actor,
        payload,
        desc_id,
        desc_digest,
        desc_ver,
        frozen,
    )
    .unwrap();
    let json = serde_json::to_string(&capsule).unwrap();
    let back: DispatchCapsuleBodyV2 = serde_json::from_str(&json).unwrap();
    assert_eq!(capsule, back);
}

#[test]
fn capsule_serde_rejects_unknown_field() {
    let (cmd, corr, actor, payload, desc_id, desc_digest, desc_ver, frozen) =
        valid_capsule_components();
    let capsule = DispatchCapsuleBodyV2::try_new(
        cmd,
        corr,
        actor,
        payload,
        desc_id,
        desc_digest,
        desc_ver,
        frozen,
    )
    .unwrap();
    let mut json = serde_json::to_value(&capsule).unwrap();
    let obj = json.as_object_mut().unwrap();
    obj.insert("unknown_field".to_owned(), serde_json::Value::Null);
    let json_str = serde_json::to_string(&json).unwrap();
    assert!(serde_json::from_str::<DispatchCapsuleBodyV2>(&json_str).is_err());
}

#[test]
fn capsule_serde_rejects_wrong_schema_version() {
    let (cmd, corr, actor, payload, desc_id, desc_digest, desc_ver, frozen) =
        valid_capsule_components();
    let capsule = DispatchCapsuleBodyV2::try_new(
        cmd,
        corr,
        actor,
        payload,
        desc_id,
        desc_digest,
        desc_ver,
        frozen,
    )
    .unwrap();
    let mut json = serde_json::to_value(&capsule).unwrap();
    json["schema_version"] = serde_json::Value::from(99u8);
    let json_str = serde_json::to_string(&json).unwrap();
    assert!(serde_json::from_str::<DispatchCapsuleBodyV2>(&json_str).is_err());
}

// ---------------------------------------------------------------------------
// M10 wire error validation
// ---------------------------------------------------------------------------

#[test]
fn wire_error_try_new_rejects_wrong_retryability() {
    let result = M10WireErrorDto::try_new(
        WireErrorClassDto::IdempotencyStoreUnavailable,
        RetryabilityDto::NotRetryable, // wrong — should be Retryable
        WireText::parse("idempotency_store_unavailable").unwrap(),
        EchoPayloadDto::None,
    );
    assert!(result.is_err());
}

#[test]
fn wire_error_try_new_rejects_wrong_code() {
    let result = M10WireErrorDto::try_new(
        WireErrorClassDto::IdempotencyStoreUnavailable,
        RetryabilityDto::Retryable,
        WireText::parse("wrong_code").unwrap(),
        EchoPayloadDto::None,
    );
    assert!(result.is_err());
}

#[test]
fn wire_error_round_trips_through_serde() {
    let error = M10WireErrorDto::try_new(
        WireErrorClassDto::ConflictingEnvelope,
        RetryabilityDto::RetryableAfterChange,
        WireText::parse("conflicting_envelope").unwrap(),
        EchoPayloadDto::Envelope {
            operation_id: WireText::parse("affairs.get").unwrap(),
            idempotency_key: WireText::parse("idem:fixture").unwrap(),
        },
    )
    .unwrap();
    let json = serde_json::to_string(&error).unwrap();
    let back: M10WireErrorDto = serde_json::from_str(&json).unwrap();
    assert_eq!(error, back);
}

#[test]
fn wire_error_serde_rejects_unknown_field() {
    let error = M10WireErrorDto::try_new(
        WireErrorClassDto::PolicyDenied,
        RetryabilityDto::NotRetryable,
        WireText::parse("policy_denied").unwrap(),
        EchoPayloadDto::PolicyDenied {
            operation_id: WireText::parse("affairs.get").unwrap(),
            permission_class: WireText::parse("public_read").unwrap(),
        },
    )
    .unwrap();
    let mut json = serde_json::to_value(&error).unwrap();
    let obj = json.as_object_mut().unwrap();
    obj.insert("unknown".to_owned(), serde_json::Value::Null);
    let json_str = serde_json::to_string(&json).unwrap();
    assert!(serde_json::from_str::<M10WireErrorDto>(&json_str).is_err());
}

// ---------------------------------------------------------------------------
// M71 terminal pairing validation
// ---------------------------------------------------------------------------

fn not_found_terminal() -> M71TerminalDto {
    M71TerminalDto::try_new(
        M71OutcomeDto::NotFound {
            procedure_id: WireText::parse("proc:missing").unwrap(),
        },
        M71LineageDto::NotRequired {
            materialization_receipt_id: WireText::parse("receipt:fixture").unwrap(),
            reason: WireText::parse("no_visible_artifact").unwrap(),
        },
    )
    .unwrap()
}

#[test]
fn terminal_pairing_rejects_not_found_with_verified_lineage() {
    assert!(
        M71TerminalDto::try_new(
            M71OutcomeDto::NotFound {
                procedure_id: WireText::parse("proc:missing").unwrap(),
            },
            M71LineageDto::Verified {
                materialization_receipt_id: WireText::parse("receipt:fixture").unwrap(),
                evidence_set_digest: WireText::parse(
                    "0000000000000000000000000000000000000000000000000000000000000000"
                )
                .unwrap(),
                revision_count: 1,
                verifier_id: WireText::parse("verifier:fixture").unwrap(),
                verified_at: UnixMillis::new(1_700_000_000_000),
                evidence_contract_version: 1,
            },
        )
        .is_err()
    );
}

#[test]
fn terminal_pairing_rejects_found_with_not_required_lineage() {
    assert!(
        M71TerminalDto::try_new(
            M71OutcomeDto::NotFound {
                procedure_id: WireText::parse("proc:missing").unwrap(),
            },
            M71LineageDto::NotRequired {
                materialization_receipt_id: WireText::parse("receipt:fixture").unwrap(),
                reason: WireText::parse("known_after_cutoff").unwrap(), // wrong reason for NotFound
            },
        )
        .is_err()
    );
}

#[test]
fn terminal_round_trips_through_serde() {
    let terminal = not_found_terminal();
    let json = serde_json::to_string(&terminal).unwrap();
    let back: M71TerminalDto = serde_json::from_str(&json).unwrap();
    assert_eq!(terminal, back);
}

#[test]
fn terminal_serde_rejects_unknown_field() {
    let terminal = not_found_terminal();
    let mut json = serde_json::to_value(&terminal).unwrap();
    let obj = json.as_object_mut().unwrap();
    obj.insert("unknown".to_owned(), serde_json::Value::Null);
    let json_str = serde_json::to_string(&json).unwrap();
    assert!(serde_json::from_str::<M71TerminalDto>(&json_str).is_err());
}

// ---------------------------------------------------------------------------
// Transport framing
// ---------------------------------------------------------------------------

#[test]
fn frame_round_trips() {
    let request = SubmitAffairsGetDto {
        request_id: WireText::parse("req:fixture").unwrap(),
        correlation_id: WireText::parse("corr:fixture").unwrap(),
        causation_id: None,
        idempotency_key: Some(WireText::parse("idem:fixture").unwrap()),
        actor: ActorIntentDto::Public,
        provenance: ClientProvenanceDto {
            build: WireText::parse("test").unwrap(),
            target: WireText::parse("test").unwrap(),
            protocol: WireText::parse("v2").unwrap(),
        },
        payload_digest: WireText::parse(
            "0000000000000000000000000000000000000000000000000000000000000000",
        )
        .unwrap(),
        procedure_id: WireText::parse("proc:fixture").unwrap(),
        as_of: None,
    };
    let mut buffer = Vec::new();
    write_frame(&mut buffer, &request).unwrap();
    let back: SubmitAffairsGetDto = read_frame(&buffer[..]).unwrap();
    assert_eq!(request, back);
}

#[test]
fn frame_rejects_zero_length() {
    let buffer = [0_u8; 4];
    assert!(read_frame::<SubmitAffairsGetDto>(&buffer[..]).is_err());
}

#[test]
fn frame_rejects_oversize_length() {
    let mut buffer = (MAX_FRAME_BYTES as u32 + 1).to_be_bytes().to_vec();
    buffer.extend_from_slice(&[0_u8; 16]);
    assert!(read_frame::<SubmitAffairsGetDto>(&buffer[..]).is_err());
}

// ---------------------------------------------------------------------------
// Deny-unknown-fields across all tagged enums
// ---------------------------------------------------------------------------

#[test]
fn actor_intent_dto_rejects_unknown_field() {
    let json = r#"{"kind":"authenticated","session_id":"s","extra":1}"#;
    assert!(serde_json::from_str::<ActorIntentDto>(json).is_err());
}

#[test]
fn viewer_authorization_dto_rejects_unknown_field() {
    let json = r#"{"kind":"public_capability","capability":"abc","extra":1}"#;
    assert!(serde_json::from_str::<ViewerAuthorizationDto>(json).is_err());
}

#[test]
fn client_response_dto_rejects_unknown_field() {
    let json = r#"{"kind":"incomplete","command_id":"c","retry_not_before":1,"extra":1}"#;
    assert!(serde_json::from_str::<ClientResponseDto>(json).is_err());
}

#[test]
fn admitted_actor_dto_rejects_unknown_field() {
    let json =
        r#"{"kind":"authenticated","tenant_id":"t","user_id":"u","session_id":"s","extra":1}"#;
    assert!(serde_json::from_str::<AdmittedActorDto>(json).is_err());
}

#[test]
fn m71_outcome_dto_rejects_unknown_field() {
    let json = r#"{"kind":"not_found","procedure_id":"proc:x","extra":1}"#;
    assert!(serde_json::from_str::<M71OutcomeDto>(json).is_err());
}

#[test]
fn m71_lineage_dto_rejects_unknown_field() {
    let json = r#"{"kind":"not_required","materialization_receipt_id":"r","reason":"x","extra":1}"#;
    assert!(serde_json::from_str::<M71LineageDto>(json).is_err());
}

#[test]
fn client_error_dto_rejects_unknown_field() {
    let json = r#"{"kind":"internal_invariant","wire_code":"x","extra":1}"#;
    assert!(serde_json::from_str::<ClientErrorDto>(json).is_err());
}

#[test]
fn secret_bearing_wire_debug_is_transitively_redacted() {
    let secret = "cap-secret-bearer-xyz";
    let viewer = ViewerAuthorizationDto::PublicCapability {
        capability: WireText::parse(secret).unwrap(),
    };
    let intent = ClientIntentDto::Lookup {
        command_id: WireText::parse("capsule-secret-command").unwrap(),
        viewer: viewer.clone(),
    };
    let response = ClientResponseDto::Accepted {
        command_id: WireText::parse("capsule-secret-command").unwrap(),
        terminal: Box::new(not_found_terminal()),
        public_capability: Some(WireText::parse(secret).unwrap()),
    };
    let (cmd, corr, actor, payload, desc_id, desc_digest, desc_ver, frozen) =
        valid_capsule_components();
    let capsule = DispatchCapsuleBodyV2::try_new(
        cmd,
        corr,
        actor,
        payload,
        desc_id,
        desc_digest,
        desc_ver,
        frozen,
    )
    .unwrap();

    for (name, debug) in [
        ("viewer", format!("{viewer:?}")),
        ("intent", format!("{intent:?}")),
        ("response", format!("{response:?}")),
        ("capsule", format!("{capsule:?}")),
    ] {
        assert!(!debug.contains(secret), "{name} leaked bearer: {debug}");
        assert!(
            !debug.contains("cmd:fixture:001"),
            "{name} leaked capsule identity: {debug}"
        );
        assert!(
            debug.contains("REDACTED") || debug.contains("redacted"),
            "{name} did not expose an explicit redaction marker: {debug}"
        );
    }
}
