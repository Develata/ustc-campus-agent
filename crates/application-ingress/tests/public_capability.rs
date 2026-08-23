#![allow(clippy::unwrap_used)]

mod common;

use std::collections::BTreeMap;

use base64::Engine as _;
use ustc_campus_agent_application_ingress::{CapabilityError, CapabilityIssuer};

fn issuer() -> CapabilityIssuer {
    let mut keys = BTreeMap::new();
    keys.insert(1u16, [0xabu8; 32]);
    CapabilityIssuer::new(keys, 1).unwrap()
}

#[test]
fn issuer_rejects_zero_current_version() {
    let mut keys = BTreeMap::new();
    keys.insert(0u16, [0u8; 32]);
    let result = CapabilityIssuer::new(keys, 0);
    assert!(matches!(result, Err(CapabilityError::UnknownKeyVersion)));
}

#[test]
fn issuer_rejects_missing_current_version_key() {
    let mut keys = BTreeMap::new();
    keys.insert(1u16, [0u8; 32]);
    let result = CapabilityIssuer::new(keys, 2);
    assert!(matches!(result, Err(CapabilityError::UnknownKeyVersion)));
}

#[test]
fn mint_and_verify_roundtrip() {
    let issuer = issuer();
    let (bearer, stored) = issuer.mint("cmd:001", "digest:abc").unwrap();
    assert!(!bearer.as_str().is_empty());
    assert_eq!(stored.key_version(), 1);
    assert!(!stored.digest_hex().is_empty());
    assert!(issuer.verify(&stored, bearer.as_str()));
}

#[test]
fn reproduce_recovers_same_bearer() {
    let issuer = issuer();
    let (bearer, stored) = issuer.mint("cmd:001", "digest:abc").unwrap();
    let reproduced = issuer.reproduce(&stored, "cmd:001", "digest:abc").unwrap();
    assert_eq!(reproduced, bearer);
}

#[test]
fn reproduce_rejects_wrong_command_id() {
    let issuer = issuer();
    let (_bearer, stored) = issuer.mint("cmd:001", "digest:abc").unwrap();
    let result = issuer.reproduce(&stored, "cmd:WRONG", "digest:abc");
    assert!(matches!(result, Err(CapabilityError::StoredDigestMismatch)));
}

#[test]
fn reproduce_rejects_wrong_capsule_digest() {
    let issuer = issuer();
    let (_bearer, stored) = issuer.mint("cmd:001", "digest:abc").unwrap();
    let result = issuer.reproduce(&stored, "cmd:001", "digest:WRONG");
    assert!(matches!(result, Err(CapabilityError::StoredDigestMismatch)));
}

#[test]
fn verify_rejects_absent_capability() {
    let issuer = issuer();
    let (_bearer, stored) = issuer.mint("cmd:001", "digest:abc").unwrap();
    assert!(!issuer.verify(&stored, ""));
}

#[test]
fn verify_rejects_wrong_bearer() {
    let issuer = issuer();
    let (_bearer, stored) = issuer.mint("cmd:001", "digest:abc").unwrap();
    assert!(!issuer.verify(&stored, "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"));
}

#[test]
fn verify_rejects_truncated_bearer() {
    let issuer = issuer();
    let (_bearer, stored) = issuer.mint("cmd:001", "digest:abc").unwrap();
    assert!(!issuer.verify(&stored, "AAAA"));
}

#[test]
fn verify_rejects_all_zero_bearer() {
    let issuer = issuer();
    let (_bearer, stored) = issuer.mint("cmd:001", "digest:abc").unwrap();
    let zero_bearer = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode([0u8; 32]);
    assert!(!issuer.verify(&stored, &zero_bearer));
}

#[test]
fn verify_rejects_random_bearer() {
    let issuer = issuer();
    let (_bearer, stored) = issuer.mint("cmd:001", "digest:abc").unwrap();
    let random_bearer = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode([0xffu8; 32]);
    assert!(!issuer.verify(&stored, &random_bearer));
}

#[test]
fn verify_rejects_bearer_from_different_command() {
    let issuer = issuer();
    let (bearer_a, _stored_a) = issuer.mint("cmd:A", "digest:a").unwrap();
    let (_bearer_b, stored_b) = issuer.mint("cmd:B", "digest:b").unwrap();
    assert!(!issuer.verify(&stored_b, bearer_a.as_str()));
}

#[test]
fn key_rotation_produces_different_bearer() {
    let issuer_v1 = {
        let mut keys = BTreeMap::new();
        keys.insert(1u16, [0x01u8; 32]);
        CapabilityIssuer::new(keys, 1).unwrap()
    };
    let issuer_v2 = {
        let mut keys = BTreeMap::new();
        keys.insert(1u16, [0x01u8; 32]);
        keys.insert(2u16, [0x02u8; 32]);
        CapabilityIssuer::new(keys, 2).unwrap()
    };
    let (bearer_v1, _) = issuer_v1.mint("cmd:001", "digest:abc").unwrap();
    let (bearer_v2, _) = issuer_v2.mint("cmd:001", "digest:abc").unwrap();
    assert_ne!(bearer_v1.as_str(), bearer_v2.as_str());
}

#[test]
fn reproduce_with_old_version_works() {
    let issuer_v1 = {
        let mut keys = BTreeMap::new();
        keys.insert(1u16, [0x01u8; 32]);
        CapabilityIssuer::new(keys, 1).unwrap()
    };
    let (bearer_v1, stored_v1) = issuer_v1.mint("cmd:001", "digest:abc").unwrap();
    let issuer_v2 = {
        let mut keys = BTreeMap::new();
        keys.insert(1u16, [0x01u8; 32]);
        keys.insert(2u16, [0x02u8; 32]);
        CapabilityIssuer::new(keys, 2).unwrap()
    };
    let reproduced = issuer_v2
        .reproduce(&stored_v1, "cmd:001", "digest:abc")
        .unwrap();
    assert_eq!(reproduced, bearer_v1);
}

#[test]
fn reproduce_missing_old_version_fails() {
    let issuer_v1 = {
        let mut keys = BTreeMap::new();
        keys.insert(1u16, [0x01u8; 32]);
        CapabilityIssuer::new(keys, 1).unwrap()
    };
    let (_bearer_v1, stored_v1) = issuer_v1.mint("cmd:001", "digest:abc").unwrap();
    let issuer_v2 = {
        let mut keys = BTreeMap::new();
        keys.insert(2u16, [0x02u8; 32]);
        CapabilityIssuer::new(keys, 2).unwrap()
    };
    let result = issuer_v2.reproduce(&stored_v1, "cmd:001", "digest:abc");
    assert!(matches!(result, Err(CapabilityError::UnknownKeyVersion)));
}

#[test]
fn different_inputs_produce_different_bearers() {
    let issuer = issuer();
    let (bearer_a, _) = issuer.mint("cmd:001", "digest:a").unwrap();
    let (bearer_b, _) = issuer.mint("cmd:001", "digest:b").unwrap();
    let (bearer_c, _) = issuer.mint("cmd:002", "digest:a").unwrap();
    assert_ne!(bearer_a, bearer_b);
    assert_ne!(bearer_a, bearer_c);
    assert_ne!(bearer_b, bearer_c);
}

#[test]
fn same_inputs_produce_identical_bearer() {
    let issuer = issuer();
    let (bearer_a, stored_a) = issuer.mint("cmd:001", "digest:abc").unwrap();
    let (bearer_b, stored_b) = issuer.mint("cmd:001", "digest:abc").unwrap();
    assert_eq!(bearer_a, bearer_b);
    assert_eq!(stored_a, stored_b);
}

#[test]
fn mint_rejects_oversized_command_id() {
    let issuer = issuer();
    let huge_command = "x".repeat(70000);
    let result = issuer.mint(&huge_command, "digest:abc");
    assert!(matches!(result, Err(CapabilityError::CommandTooLong)));
}

#[test]
fn constant_time_eq_equal_slices() {
    use ustc_campus_agent_application_ingress::capability::constant_time_eq;
    assert!(constant_time_eq(b"hello", b"hello"));
}

#[test]
fn constant_time_eq_unequal_slices() {
    use ustc_campus_agent_application_ingress::capability::constant_time_eq;
    assert!(!constant_time_eq(b"hello", b"world"));
}

#[test]
fn constant_time_eq_different_lengths() {
    use ustc_campus_agent_application_ingress::capability::constant_time_eq;
    assert!(!constant_time_eq(b"hello", b"hell"));
    assert!(!constant_time_eq(b"hell", b"hello"));
}

#[test]
fn constant_time_eq_empty_slices() {
    use ustc_campus_agent_application_ingress::capability::constant_time_eq;
    assert!(constant_time_eq(b"", b""));
}

#[test]
fn digest_hex_is_64_chars() {
    let issuer = issuer();
    let (_bearer, stored) = issuer.mint("cmd:001", "digest:abc").unwrap();
    assert_eq!(stored.digest_hex().len(), 64);
    assert!(stored.digest_hex().chars().all(|c| c.is_ascii_hexdigit()));
}

#[test]
fn bearer_is_url_safe_base64_of_32_bytes() {
    let issuer = issuer();
    let (bearer, _) = issuer.mint("cmd:001", "digest:abc").unwrap();
    let decoded = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(bearer.as_str())
        .expect("bearer must be valid base64");
    assert_eq!(decoded.len(), 32);
}

#[test]
fn lookup_with_public_capability_succeeds() {
    use affairs_navigator::{
        FixedClock, InMemoryAffairsRepository, m60_fixture::M60FixtureAdapter,
    };
    use common::{FakePorts, M71FixturePort, submit_request, t};
    use ustc_campus_agent_application_ingress::{FileRecordStore, M10Service};
    use ustc_campus_agent_client_protocol::{ClientResponseDto, ViewerAuthorizationDto};

    let store = FileRecordStore::open(common::temp_path()).unwrap();
    let cap_issuer = common::cap_issuer();
    let repo = InMemoryAffairsRepository::new();
    let m60 = M60FixtureAdapter::new("verifier:fixture", 1).unwrap();
    let clock = FixedClock::new(t(200));
    let m71 = M71FixturePort::new(&repo, &m60, &clock);
    let service = M10Service::new(
        store,
        cap_issuer,
        &m71,
        ustc_campus_agent_client_protocol::WireText::parse("operator:fixture").unwrap(),
    );

    let mut ports = FakePorts::public_admitted();
    let request = submit_request("proc:missing");
    let response = service.submit(&request, &mut ports, 1_000_000);

    let bearer = match response {
        ClientResponseDto::Accepted {
            public_capability, ..
        } => public_capability.expect("public actor must receive capability"),
        _ => panic!("expected Accepted, got {response:?}"),
    };

    let response = service.lookup(
        "command:fixture",
        &ViewerAuthorizationDto::PublicCapability { capability: bearer },
    );
    match response {
        ClientResponseDto::Available { redaction, .. } => {
            assert!(matches!(
                redaction,
                ustc_campus_agent_client_protocol::RedactionDto::Public
            ));
        }
        _ => panic!("expected Available, got {response:?}"),
    }
}

#[test]
fn lookup_with_wrong_capability_returns_unavailable() {
    use affairs_navigator::{
        FixedClock, InMemoryAffairsRepository, m60_fixture::M60FixtureAdapter,
    };
    use common::{FakePorts, M71FixturePort, submit_request, t};
    use ustc_campus_agent_application_ingress::{FileRecordStore, M10Service};
    use ustc_campus_agent_client_protocol::{ClientResponseDto, ViewerAuthorizationDto};

    let store = FileRecordStore::open(common::temp_path()).unwrap();
    let cap_issuer = common::cap_issuer();
    let repo = InMemoryAffairsRepository::new();
    let m60 = M60FixtureAdapter::new("verifier:fixture", 1).unwrap();
    let clock = FixedClock::new(t(200));
    let m71 = M71FixturePort::new(&repo, &m60, &clock);
    let service = M10Service::new(
        store,
        cap_issuer,
        &m71,
        ustc_campus_agent_client_protocol::WireText::parse("operator:fixture").unwrap(),
    );

    let mut ports = FakePorts::public_admitted();
    let request = submit_request("proc:missing");
    let _response = service.submit(&request, &mut ports, 1_000_000);

    let response = service.lookup(
        "command:fixture",
        &ViewerAuthorizationDto::PublicCapability {
            capability: ustc_campus_agent_client_protocol::WireText::parse("wrong-capability")
                .unwrap(),
        },
    );
    assert!(matches!(response, ClientResponseDto::Unavailable));
}

#[test]
fn b3_empty_operator_grant_id_rejected_by_wiretext_parse() {
    assert!(ustc_campus_agent_client_protocol::WireText::parse("").is_err());
}

#[test]
fn b3_control_char_operator_grant_id_rejected_by_wiretext_parse() {
    assert!(ustc_campus_agent_client_protocol::WireText::parse("operator\0bad").is_err());
}

#[test]
fn b3_oversize_operator_grant_id_rejected_by_wiretext_parse() {
    let oversize = "x".repeat(ustc_campus_agent_client_protocol::WireText::MAX_BYTES + 1);
    assert!(ustc_campus_agent_client_protocol::WireText::parse(oversize).is_err());
}

#[test]
fn b3_operator_grant_exact_match_succeeds_lookup() {
    use affairs_navigator::{
        FixedClock, InMemoryAffairsRepository, m60_fixture::M60FixtureAdapter,
    };
    use common::{FakePorts, M71FixturePort, submit_request, t};
    use ustc_campus_agent_application_ingress::{FileRecordStore, M10Service};
    use ustc_campus_agent_client_protocol::{ClientResponseDto, ViewerAuthorizationDto};

    let store = FileRecordStore::open(common::temp_path()).unwrap();
    let cap_issuer = common::cap_issuer();
    let repo = InMemoryAffairsRepository::new();
    let m60 = M60FixtureAdapter::new("verifier:fixture", 1).unwrap();
    let clock = FixedClock::new(t(200));
    let m71 = M71FixturePort::new(&repo, &m60, &clock);
    let grant = ustc_campus_agent_client_protocol::WireText::parse("operator:fixture").unwrap();
    let service = M10Service::new(store, cap_issuer, &m71, grant);

    let mut ports = FakePorts::public_admitted();
    let request = submit_request("proc:missing");
    let _response = service.submit(&request, &mut ports, 1_000_000);

    let response = service.lookup(
        "command:fixture",
        &ViewerAuthorizationDto::Operator {
            grant_id: ustc_campus_agent_client_protocol::WireText::parse("operator:fixture")
                .unwrap(),
        },
    );
    match response {
        ClientResponseDto::Available { redaction, .. } => {
            assert!(matches!(
                redaction,
                ustc_campus_agent_client_protocol::RedactionDto::Operator
            ));
        }
        _ => panic!("expected Available with Operator redaction, got {response:?}"),
    }
}

#[test]
fn b3_operator_grant_wrong_match_returns_unavailable() {
    use affairs_navigator::{
        FixedClock, InMemoryAffairsRepository, m60_fixture::M60FixtureAdapter,
    };
    use common::{FakePorts, M71FixturePort, submit_request, t};
    use ustc_campus_agent_application_ingress::{FileRecordStore, M10Service};
    use ustc_campus_agent_client_protocol::{ClientResponseDto, ViewerAuthorizationDto};

    let store = FileRecordStore::open(common::temp_path()).unwrap();
    let cap_issuer = common::cap_issuer();
    let repo = InMemoryAffairsRepository::new();
    let m60 = M60FixtureAdapter::new("verifier:fixture", 1).unwrap();
    let clock = FixedClock::new(t(200));
    let m71 = M71FixturePort::new(&repo, &m60, &clock);
    let grant = ustc_campus_agent_client_protocol::WireText::parse("operator:fixture").unwrap();
    let service = M10Service::new(store, cap_issuer, &m71, grant);

    let mut ports = FakePorts::public_admitted();
    let request = submit_request("proc:missing");
    let _response = service.submit(&request, &mut ports, 1_000_000);

    let response = service.lookup(
        "command:fixture",
        &ViewerAuthorizationDto::Operator {
            grant_id: ustc_campus_agent_client_protocol::WireText::parse("operator:wrong").unwrap(),
        },
    );
    assert!(matches!(response, ClientResponseDto::Unavailable));
}

#[test]
fn b3_operator_grant_never_appears_in_debug() {
    use affairs_navigator::{
        FixedClock, InMemoryAffairsRepository, m60_fixture::M60FixtureAdapter,
    };
    use common::{FakePorts, M71FixturePort, submit_request, t};
    use ustc_campus_agent_application_ingress::{FileRecordStore, M10Service};

    let store = FileRecordStore::open(common::temp_path()).unwrap();
    let cap_issuer = common::cap_issuer();
    let repo = InMemoryAffairsRepository::new();
    let m60 = M60FixtureAdapter::new("verifier:fixture", 1).unwrap();
    let clock = FixedClock::new(t(200));
    let m71 = M71FixturePort::new(&repo, &m60, &clock);
    let secret_grant = "operator:secret-canary-12345";
    let grant = ustc_campus_agent_client_protocol::WireText::parse(secret_grant).unwrap();
    let service = M10Service::new(store, cap_issuer, &m71, grant);

    let mut ports = FakePorts::public_admitted();
    let request = submit_request("proc:missing");
    let response = service.submit(&request, &mut ports, 1_000_000);

    let debug = format!("{response:?}");
    assert!(
        !debug.contains(secret_grant),
        "operator grant must not leak in ClientResponseDto Debug, got: {debug}"
    );
}
