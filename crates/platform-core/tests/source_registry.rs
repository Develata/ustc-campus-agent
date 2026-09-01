//! `source-import/v1` M60-B1 acceptance evidence for the source registry.
//!
//! Bound row: `SRC-001`. Covers the strict nominal value grammars and Serde
//! surfaces, media-type boundaries, the six-field retrieval policy, the exact
//! v1 proposal state and revision, every legal lifecycle transition, CAS on
//! every post-proposal mutation family, terminal revocation, stale rejection,
//! revision overflow, duplicate ID/URL rejection, retrieval gating, failed
//! operation atomicity, and the proposed-only USTC candidate family.
//!
//! Synthetic fixtures use `https://example.invalid/...` only; no concrete USTC
//! source is approved and no network path exists.

use std::error::Error;

use serde::Deserialize;
use serde::de::IntoDeserializer;
use serde::de::value::{BytesDeserializer, Error as SerdeValueError, StringDeserializer};

use ustc_campus_agent_core::SourceAuthority;
use ustc_campus_agent_core::source_registry::{
    PublicIpPolicyVersion, SourceAuthorityRevision, SourceDefinition, SourceDefinitionBody,
    SourceId, SourceMediaType, SourceOwner, SourceRegistry, SourceRegistryError,
    SourceRetrievalPolicy, SourceRetrievalProtocolVersion, SourceReviewEvidenceId,
    SourceReviewReceipt, SourceReviewerId, SourceStatus, SourceStatusEvidenceId, SourceStatusKind,
    SourceTransitionCommand, SourceUrl, SourceValueErrorKind,
};

// ---------------------------------------------------------------------------
// Bounds (mirror the contract's fixed ceilings).
// ---------------------------------------------------------------------------

const MAX_ID_BYTES: usize = 128;
const MAX_OWNER_BYTES: usize = 128;
const MAX_URL_BYTES: usize = 2048;
const MAX_MIN_INTERVAL: u32 = 604_800;
const MAX_MAX_BYTES: u32 = 1_048_576;
const MAX_ELAPSED: u32 = 60;

// ---------------------------------------------------------------------------
// Serde deserializer helpers (owned-string and bytes entry points).
// ---------------------------------------------------------------------------

fn owned_deserializer(value: String) -> StringDeserializer<SerdeValueError> {
    value.into_deserializer()
}

fn bytes_deserializer(value: &[u8]) -> BytesDeserializer<'_, SerdeValueError> {
    BytesDeserializer::new(value)
}

// ---------------------------------------------------------------------------
// §3 SourceId-family grammar: edges and precedence.
// ---------------------------------------------------------------------------

fn source_id_valid_values() -> Vec<String> {
    let mut exact_max = String::from("a");
    exact_max.push_str(&"b".repeat(MAX_ID_BYTES - 2));
    exact_max.push('c');
    assert_eq!(exact_max.len(), MAX_ID_BYTES);

    vec![
        "a".to_owned(),
        "7".to_owned(),
        "aa".to_owned(),
        "abc".to_owned(),
        "ustc:teach-calendar-fall-2025".to_owned(),
        "a.b:c-d_e".to_owned(),
        "a..__::--b".to_owned(),
        exact_max,
    ]
}

fn source_id_invalid_values() -> Vec<(String, SourceValueErrorKind)> {
    let too_long = "a".repeat(MAX_ID_BYTES + 1);
    let too_long_bad_start = format!("-{}", "a".repeat(MAX_ID_BYTES));
    let too_long_bad_end = format!("{}-", "a".repeat(MAX_ID_BYTES));

    vec![
        (String::new(), SourceValueErrorKind::Empty),
        (
            too_long,
            SourceValueErrorKind::TooLong {
                max_bytes: MAX_ID_BYTES,
            },
        ),
        (
            too_long_bad_start,
            SourceValueErrorKind::TooLong {
                max_bytes: MAX_ID_BYTES,
            },
        ),
        (
            too_long_bad_end,
            SourceValueErrorKind::TooLong {
                max_bytes: MAX_ID_BYTES,
            },
        ),
        ("-abc".to_owned(), SourceValueErrorKind::InvalidStart),
        ("Aabc".to_owned(), SourceValueErrorKind::InvalidStart),
        (" abc".to_owned(), SourceValueErrorKind::InvalidStart),
        ("!abc".to_owned(), SourceValueErrorKind::InvalidStart),
        ("é".to_owned(), SourceValueErrorKind::InvalidStart),
        (
            "a b".to_owned(),
            SourceValueErrorKind::InvalidCharacter { byte_index: 1 },
        ),
        (
            "aBc".to_owned(),
            SourceValueErrorKind::InvalidCharacter { byte_index: 1 },
        ),
        (
            "a/b".to_owned(),
            SourceValueErrorKind::InvalidCharacter { byte_index: 1 },
        ),
        (
            "a@b".to_owned(),
            SourceValueErrorKind::InvalidCharacter { byte_index: 1 },
        ),
        (
            "aé".to_owned(),
            SourceValueErrorKind::InvalidCharacter { byte_index: 1 },
        ),
        (
            "a b-".to_owned(),
            SourceValueErrorKind::InvalidCharacter { byte_index: 1 },
        ),
        ("abc-".to_owned(), SourceValueErrorKind::InvalidEnd),
        ("abc.".to_owned(), SourceValueErrorKind::InvalidEnd),
        ("abc_".to_owned(), SourceValueErrorKind::InvalidEnd),
        ("abc:".to_owned(), SourceValueErrorKind::InvalidEnd),
        ("abcA".to_owned(), SourceValueErrorKind::InvalidEnd),
        ("a-".to_owned(), SourceValueErrorKind::InvalidEnd),
    ]
}

macro_rules! assert_id_family_accepts {
    ($kind:ty) => {{
        let kind_name = stringify!($kind);

        for value in source_id_valid_values() {
            let parsed = <$kind>::parse(value.clone())
                .unwrap_or_else(|_| panic!("{kind_name} must accept a canonical value"));
            assert_eq!(
                parsed.as_str(),
                value,
                "{kind_name} must retain exact bytes"
            );

            let from_string = <$kind>::try_from(value.clone())
                .unwrap_or_else(|_| panic!("{kind_name} TryFrom<String> must accept"));
            let from_str_ref = <$kind>::try_from(value.as_str())
                .unwrap_or_else(|_| panic!("{kind_name} TryFrom<&str> must accept"));
            let from_str = value
                .parse::<$kind>()
                .unwrap_or_else(|_| panic!("{kind_name} FromStr must accept"));
            assert_eq!(from_string, parsed);
            assert_eq!(from_str_ref, parsed);
            assert_eq!(from_str, parsed);

            let encoded = serde_json::to_string(&parsed).expect("serialize");
            assert_eq!(
                encoded,
                serde_json::to_string(&value).expect("serialize"),
                "{kind_name} must serialize as one JSON string"
            );
            let decoded: $kind = serde_json::from_str(&encoded).expect("deserialize");
            assert_eq!(decoded, parsed, "{kind_name} Serde must round-trip");

            let from_owned = <$kind>::deserialize(owned_deserializer(value.clone()))
                .unwrap_or_else(|_| panic!("{kind_name} owned-string Serde must accept"));
            assert_eq!(from_owned, parsed);
            let from_bytes = <$kind>::deserialize(bytes_deserializer(value.as_bytes()))
                .unwrap_or_else(|_| panic!("{kind_name} bytes Serde must accept"));
            assert_eq!(from_bytes, parsed);
        }
    }};
}

macro_rules! assert_id_family_rejects {
    ($kind:ty) => {{
        let kind_name = stringify!($kind);

        for (value, expected) in source_id_invalid_values() {
            let error = <$kind>::parse(value.clone())
                .expect_err("{kind_name} must reject a non-canonical value");
            assert_eq!(error.value_kind(), kind_name);
            assert_eq!(
                error.kind(),
                expected,
                "{kind_name} precedence drift for {}-byte input",
                value.len()
            );

            assert!(
                <$kind>::try_from(value.clone()).is_err(),
                "{kind_name} TryFrom<String> must reject"
            );
            assert!(
                <$kind>::try_from(value.as_str()).is_err(),
                "{kind_name} TryFrom<&str> must reject"
            );
            assert!(
                value.parse::<$kind>().is_err(),
                "{kind_name} FromStr must reject"
            );

            let encoded = serde_json::to_string(&value).expect("serialize");
            assert!(
                serde_json::from_str::<$kind>(&encoded).is_err(),
                "{kind_name} Serde must reject"
            );
            let owned_error = <$kind>::deserialize(owned_deserializer(value.clone()))
                .expect_err("{kind_name} owned-string Serde must reject");
            assert_eq!(
                owned_error.to_string(),
                error.to_string(),
                "{kind_name} owned-string Serde must report the checked constructor error"
            );
            let bytes_error = <$kind>::deserialize(bytes_deserializer(value.as_bytes()))
                .expect_err("{kind_name} bytes Serde must reject");
            assert_eq!(
                bytes_error.to_string(),
                error.to_string(),
                "{kind_name} bytes Serde must report the checked constructor error"
            );
        }
    }};
}

// ---------------------------------------------------------------------------
// Fixtures.
// ---------------------------------------------------------------------------

fn id(value: &str) -> SourceId {
    SourceId::parse(value).expect("fixture source id")
}

fn owner(value: &str) -> SourceOwner {
    SourceOwner::parse(value).expect("fixture owner")
}

fn url(value: &str) -> SourceUrl {
    SourceUrl::parse(value).expect("fixture url")
}

fn media(value: &str) -> SourceMediaType {
    SourceMediaType::parse(value).expect("fixture media type")
}

fn policy_with_media(expected_media: SourceMediaType) -> SourceRetrievalPolicy {
    SourceRetrievalPolicy::new(
        21_600,
        131_072,
        60,
        expected_media,
        SourceRetrievalProtocolVersion::V0StrictHttpsIpv4Http11_20260809,
        PublicIpPolicyVersion::V0Ipv4Only20260809,
    )
    .expect("fixture policy")
}

fn fixture_policy() -> SourceRetrievalPolicy {
    policy_with_media(media("text/html"))
}

fn receipt_with_reviewer(reviewer: &str) -> SourceReviewReceipt {
    SourceReviewReceipt::new(
        SourceReviewerId::parse(reviewer).expect("fixture reviewer"),
        SourceReviewEvidenceId::parse("evidence:review").expect("fixture"),
        SourceReviewEvidenceId::parse("evidence:permission").expect("fixture"),
        SourceReviewEvidenceId::parse("evidence:rate").expect("fixture"),
        SourceReviewEvidenceId::parse("evidence:fixture").expect("fixture"),
    )
}

fn fixture_receipt() -> SourceReviewReceipt {
    receipt_with_reviewer("reviewer:operator")
}

fn other_receipt() -> SourceReviewReceipt {
    receipt_with_reviewer("reviewer:different")
}

fn status_evidence(value: &str) -> SourceStatusEvidenceId {
    SourceStatusEvidenceId::new(String::from(value)).expect("fixture status evidence")
}

fn fixture_evidence() -> SourceStatusEvidenceId {
    status_evidence("evidence:status")
}

fn current_revision(registry: &SourceRegistry, source_id: &SourceId) -> SourceAuthorityRevision {
    registry
        .get(source_id)
        .expect("definition present")
        .authority_revision()
}

fn proposed_definition(source_id: &str, source_url: &str) -> SourceDefinition {
    SourceDefinition::proposed(
        id(source_id),
        owner("Example Campus Office"),
        url(source_url),
        SourceAuthority::ReviewedOfficialSource,
        fixture_policy(),
    )
    .expect("fixture definition")
}

fn fixture_definition() -> SourceDefinition {
    proposed_definition("example:source", "https://example.invalid/calendar/19081")
}

fn body(owner_label: &str, source_url: &str, authority: SourceAuthority) -> SourceDefinitionBody {
    SourceDefinitionBody::new(
        owner(owner_label),
        url(source_url),
        authority,
        fixture_policy(),
    )
    .expect("fixture body")
}

// ---------------------------------------------------------------------------
// 1. Strict nominal values: grammar, checked constructors, Serde, exact bytes.
// ---------------------------------------------------------------------------

#[test]
fn source_registry_parses_strict_nominal_values() {
    assert_id_family_accepts!(SourceId);
    assert_id_family_accepts!(SourceReviewerId);
    assert_id_family_accepts!(SourceReviewEvidenceId);

    let valid_owners = [
        "a",
        "USTC Affairs Office",
        "中国科学技术大学教务处",
        "www.teach.ustc.edu.cn",
        &{
            let mut value = String::from("x");
            value.push_str(&"y".repeat(MAX_OWNER_BYTES - 2));
            value.push('z');
            value
        },
    ];
    for value in valid_owners {
        let parsed = owner(value);
        assert_eq!(
            parsed.as_str(),
            value,
            "SourceOwner must preserve text exactly"
        );
        let encoded = serde_json::to_string(&parsed).expect("serialize");
        let decoded: SourceOwner = serde_json::from_str(&encoded).expect("deserialize");
        assert_eq!(decoded, parsed, "SourceOwner Serde must round-trip");
    }

    let valid_urls = [
        "https://www.ustc.edu.cn/",
        "https://example.invalid/calendar/19081.html",
        "https://example.invalid/category/calendar",
        "https://example.invalid/a",
        "https://example.invalid/a-b_c.d~e",
        "https://example.invalid/%41%42%43",
        "https://sub.domain.example.invalid/path/to/resource",
        "https://123.example.invalid/foo",
    ];
    for value in valid_urls {
        let parsed = url(value);
        assert_eq!(parsed.as_str(), value, "SourceUrl must preserve exactly");
        let encoded = serde_json::to_string(&parsed).expect("serialize");
        let decoded: SourceUrl = serde_json::from_str(&encoded).expect("deserialize");
        assert_eq!(decoded, parsed, "SourceUrl Serde must round-trip");
    }
    let maximum_dns_host = format!(
        "{}.{}.{}.{}",
        "a".repeat(63),
        "b".repeat(63),
        "c".repeat(63),
        "d".repeat(61)
    );
    assert_eq!(maximum_dns_host.len(), 253);
    let maximum_dns_url = format!("https://{maximum_dns_host}/data");
    assert_eq!(url(&maximum_dns_url).as_str(), maximum_dns_url);

    let status_evidence = SourceStatusEvidenceId::new(String::from("ustc:status-evidence-1"))
        .expect("status evidence id");
    assert_eq!(status_evidence.as_str(), "ustc:status-evidence-1");
    assert_eq!(status_evidence.into_inner(), "ustc:status-evidence-1");

    let left = SourceStatusEvidenceId::new(String::from("ustc:evidence-a")).expect("fixture");
    let right = SourceStatusEvidenceId::new(String::from("ustc:evidence-b")).expect("fixture");
    let left_clone = left.clone();
    assert_eq!(left, left_clone, "SourceStatusEvidenceId must be Clone");
    assert!(left < right, "SourceStatusEvidenceId must be Ord");
    assert_ne!(left, right);

    // Nominal kinds are distinct types with no cross-kind conversion: the same
    // text parses under every kind but the values are never interchangeable.
    let source_id = id("ustc:evidence-a");
    assert_eq!(source_id.as_str(), left.as_str());
}

// ---------------------------------------------------------------------------
// 2. Invalid nominal values: exact error kinds, Serde rejection, no echo.
// ---------------------------------------------------------------------------

#[test]
fn source_registry_rejects_invalid_nominal_values() {
    assert_id_family_rejects!(SourceId);
    assert_id_family_rejects!(SourceReviewerId);
    assert_id_family_rejects!(SourceReviewEvidenceId);

    // `SourceStatusEvidenceId::new` enforces the same grammar.
    for (value, expected) in source_id_invalid_values() {
        let error = SourceStatusEvidenceId::new(value).expect_err("must reject");
        assert_eq!(error.value_kind(), "SourceStatusEvidenceId");
        assert_eq!(error.kind(), expected);
    }

    let empty_max_owner = "x".repeat(MAX_OWNER_BYTES + 1);
    let owner_cases: Vec<(String, SourceValueErrorKind)> = vec![
        (String::new(), SourceValueErrorKind::Empty),
        (
            empty_max_owner,
            SourceValueErrorKind::TooLong {
                max_bytes: MAX_OWNER_BYTES,
            },
        ),
        (
            " leading".to_owned(),
            SourceValueErrorKind::OwnerBoundaryWhitespace,
        ),
        (
            "trailing ".to_owned(),
            SourceValueErrorKind::OwnerBoundaryWhitespace,
        ),
        (
            "\tleading".to_owned(),
            SourceValueErrorKind::OwnerBoundaryWhitespace,
        ),
        (
            "trailing\n".to_owned(),
            SourceValueErrorKind::OwnerBoundaryWhitespace,
        ),
        (
            "a\u{0}b".to_owned(),
            SourceValueErrorKind::OwnerControlCharacter { byte_index: 1 },
        ),
        (
            "a\u{7f}b".to_owned(),
            SourceValueErrorKind::OwnerControlCharacter { byte_index: 1 },
        ),
        (
            "a\u{9f}b".to_owned(),
            SourceValueErrorKind::OwnerControlCharacter { byte_index: 1 },
        ),
    ];
    for (value, expected) in owner_cases {
        let error = SourceOwner::parse(value.clone()).expect_err("SourceOwner must reject");
        assert_eq!(error.value_kind(), "SourceOwner");
        assert_eq!(error.kind(), expected);
    }

    let too_long_url = format!("https://example.invalid/{}", "a".repeat(MAX_URL_BYTES));
    let oversized_dns_host = format!(
        "{}.{}.{}.{}",
        "a".repeat(63),
        "b".repeat(63),
        "c".repeat(63),
        "d".repeat(63)
    );
    assert_eq!(oversized_dns_host.len(), 255);
    let url_cases: Vec<(String, SourceValueErrorKind)> = vec![
        (String::new(), SourceValueErrorKind::Empty),
        (
            too_long_url,
            SourceValueErrorKind::TooLong {
                max_bytes: MAX_URL_BYTES,
            },
        ),
        (
            "http://example.invalid/".to_owned(),
            SourceValueErrorKind::InvalidScheme,
        ),
        (
            "HTTPS://example.invalid/".to_owned(),
            SourceValueErrorKind::InvalidScheme,
        ),
        (
            "ftp://example.invalid/".to_owned(),
            SourceValueErrorKind::InvalidScheme,
        ),
        (
            "https:/example.invalid/".to_owned(),
            SourceValueErrorKind::InvalidScheme,
        ),
        (
            "https://example.invalid".to_owned(),
            SourceValueErrorKind::InvalidPath,
        ),
        (
            "https://localhost/".to_owned(),
            SourceValueErrorKind::InvalidHost,
        ),
        (
            format!("https://{oversized_dns_host}/data"),
            SourceValueErrorKind::InvalidHost,
        ),
        (
            "https://example.invalid:8080/".to_owned(),
            SourceValueErrorKind::InvalidHost,
        ),
        (
            "https://user@example.invalid/".to_owned(),
            SourceValueErrorKind::InvalidHost,
        ),
        (
            "https://example.invalid/?q=1".to_owned(),
            SourceValueErrorKind::InvalidPath,
        ),
        (
            "https://example.invalid/#frag".to_owned(),
            SourceValueErrorKind::InvalidPath,
        ),
        (
            "https://Example.invalid/".to_owned(),
            SourceValueErrorKind::InvalidHost,
        ),
        (
            "https://192.168.0.1/".to_owned(),
            SourceValueErrorKind::InvalidHost,
        ),
        (
            "https://example.invalid./".to_owned(),
            SourceValueErrorKind::InvalidHost,
        ),
        (
            "https://-bad.invalid/".to_owned(),
            SourceValueErrorKind::InvalidHost,
        ),
        (
            "https://example.invalid//".to_owned(),
            SourceValueErrorKind::InvalidPath,
        ),
        (
            "https://example.invalid/./".to_owned(),
            SourceValueErrorKind::InvalidPath,
        ),
        (
            "https://example.invalid/../".to_owned(),
            SourceValueErrorKind::InvalidPath,
        ),
        (
            "https://example.invalid/a//b".to_owned(),
            SourceValueErrorKind::InvalidPath,
        ),
        (
            "https://example.invalid/a/.".to_owned(),
            SourceValueErrorKind::InvalidPath,
        ),
        (
            "https://example.invalid/a/..".to_owned(),
            SourceValueErrorKind::InvalidPath,
        ),
        (
            "https://example.invalid/a ".to_owned(),
            SourceValueErrorKind::InvalidPath,
        ),
        (
            "https://example.invalid/%".to_owned(),
            SourceValueErrorKind::InvalidPath,
        ),
        (
            "https://example.invalid/%4".to_owned(),
            SourceValueErrorKind::InvalidPath,
        ),
        (
            "https://example.invalid/%4G".to_owned(),
            SourceValueErrorKind::InvalidPath,
        ),
        (
            "https://example.invalid/%4g".to_owned(),
            SourceValueErrorKind::InvalidPath,
        ),
    ];
    for (value, expected) in url_cases {
        let error = SourceUrl::parse(value.clone()).expect_err("SourceUrl must reject");
        assert_eq!(error.value_kind(), "SourceUrl");
        assert_eq!(error.kind(), expected);
    }

    let Err(host_error) = SourceUrl::parse("https://exämple.invalid/") else {
        panic!("must reject a non-ASCII host");
    };
    assert_eq!(host_error.kind(), SourceValueErrorKind::InvalidHost);
    let Err(path_error) = SourceUrl::parse("https://example.invalid/café") else {
        panic!("must reject a non-ASCII path");
    };
    assert_eq!(path_error.kind(), SourceValueErrorKind::InvalidPath);

    // No-echo: Display, Debug and the error chain never retain rejected input.
    let secret = "super-secret-value";
    let Err(id_error) = SourceId::parse(format!("{secret}/")) else {
        panic!("must reject");
    };
    let display = format!("{id_error}");
    let debug = format!("{id_error:?}");
    assert!(!display.contains(secret), "Display leaked: {display}");
    assert!(!debug.contains(secret), "Debug leaked: {debug}");
    assert!(id_error.source().is_none(), "source chain leaked");

    let Err(owner_error) = SourceOwner::parse(format!(" {secret}")) else {
        panic!("must reject");
    };
    assert!(
        !format!("{owner_error}").contains(secret),
        "Owner Display leaked"
    );
    assert!(
        !format!("{owner_error:?}").contains(secret),
        "Owner Debug leaked"
    );

    let Err(url_error) = SourceUrl::parse(format!("https://example.invalid/{secret}?")) else {
        panic!("must reject");
    };
    assert!(
        !format!("{url_error}").contains(secret),
        "URL Display leaked"
    );
    assert!(
        !format!("{url_error:?}").contains(secret),
        "URL Debug leaked"
    );
}

// ---------------------------------------------------------------------------
// 3. Valid media-type boundaries.
// ---------------------------------------------------------------------------

#[test]
fn source_registry_accepts_valid_media_type_boundaries() {
    let min_shape = "a/b";
    let parsed_min = media(min_shape);
    assert_eq!(
        parsed_min,
        media("a/b"),
        "minimum three-byte media type must parse"
    );

    let typical = [
        "text/html",
        "application/json",
        "text/plain",
        "text/0123456789",
        "text/a!#$%&'+-.^_`|~",
        "application/vnd.example.payload+json",
    ];
    for value in typical {
        let parsed = media(value);
        let policy = policy_with_media(parsed.clone());
        assert_eq!(
            *policy.expected_media_type(),
            media(value),
            "policy must embed the exact media type"
        );
    }

    // Both components at their 64-byte ceiling: total 129 bytes.
    let boundary = format!("{}/{}", "a".repeat(64), "b".repeat(64));
    assert_eq!(boundary.len(), 129);
    let parsed_boundary = SourceMediaType::parse(&boundary)
        .unwrap_or_else(|_| panic!("64/64 boundary media type must parse"));
    let policy = policy_with_media(parsed_boundary);
    assert_eq!(
        *policy.expected_media_type(),
        SourceMediaType::parse(&boundary).expect("reparse"),
        "boundary media type must embed exactly"
    );

    // All-digit subtype is a valid token.
    assert_eq!(media("text/1234"), media("text/1234"));
}

// ---------------------------------------------------------------------------
// 4. Invalid media-type boundaries.
// ---------------------------------------------------------------------------

#[test]
fn source_registry_rejects_invalid_media_type_boundaries() {
    let invalid: Vec<String> = vec![
        String::new(),
        "text".to_owned(),
        "/html".to_owned(),
        "text/".to_owned(),
        "text//html".to_owned(),
        "text/html/extra".to_owned(),
        "TEXT/HTML".to_owned(),
        "text/HTML".to_owned(),
        "Text/Html".to_owned(),
        "text/*".to_owned(),
        "*/*".to_owned(),
        "text/html ".to_owned(),
        " text/html".to_owned(),
        "text\thtml".to_owned(),
        "text/html;charset=utf-8".to_owned(),
        "text/ht ml".to_owned(),
        "text/héllo".to_owned(),
        format!("{}/b", "a".repeat(65)),
        format!("a/{}", "b".repeat(65)),
    ];
    for value in &invalid {
        let error = SourceMediaType::parse(value).expect_err("must reject");
        assert_eq!(error.value_kind(), "SourceMediaType");
        assert_eq!(error.kind(), SourceValueErrorKind::InvalidMediaType);
    }

    // No echo: Display and Debug never retain the rejected text.
    let secret = "super secret media";
    let Err(error) = SourceMediaType::parse(&format!("text/{secret}")) else {
        panic!("must reject");
    };
    let display = format!("{error}");
    let debug = format!("{error:?}");
    assert!(!display.contains(secret), "Display leaked: {display}");
    assert!(!debug.contains(secret), "Debug leaked: {debug}");
}

// ---------------------------------------------------------------------------
// 5. Proposal: exact v1 state and revision; policy bounds and precedence;
//    ModelInference rejection; proposed-only USTC candidate family.
// ---------------------------------------------------------------------------

#[test]
fn source_registry_proposes_with_exact_v1_state_and_revision() {
    // Policy ceilings and declaration-order precedence.
    let protocol = SourceRetrievalProtocolVersion::V0StrictHttpsIpv4Http11_20260809;
    let ip_policy = PublicIpPolicyVersion::V0Ipv4Only20260809;
    let text_html = media("text/html");

    let error = SourceRetrievalPolicy::new(0, 1, 1, text_html.clone(), protocol, ip_policy)
        .expect_err("zero minimum interval");
    assert_eq!(error.kind(), SourceValueErrorKind::ZeroMinimumInterval);
    assert_eq!(error.value_kind(), "SourceRetrievalPolicy");

    let error = SourceRetrievalPolicy::new(
        MAX_MIN_INTERVAL + 1,
        1,
        1,
        text_html.clone(),
        protocol,
        ip_policy,
    )
    .expect_err("too large minimum interval");
    assert_eq!(
        error.kind(),
        SourceValueErrorKind::MinimumIntervalTooLarge {
            max_seconds: MAX_MIN_INTERVAL
        }
    );

    let error = SourceRetrievalPolicy::new(1, 0, 1, text_html.clone(), protocol, ip_policy)
        .expect_err("zero maximum response bytes");
    assert_eq!(error.kind(), SourceValueErrorKind::ZeroMaximumResponseBytes);

    let error = SourceRetrievalPolicy::new(
        1,
        MAX_MAX_BYTES + 1,
        1,
        text_html.clone(),
        protocol,
        ip_policy,
    )
    .expect_err("too large maximum response bytes");
    assert_eq!(
        error.kind(),
        SourceValueErrorKind::MaximumResponseBytesTooLarge {
            max_bytes: MAX_MAX_BYTES
        }
    );

    let error = SourceRetrievalPolicy::new(1, 1, 0, text_html.clone(), protocol, ip_policy)
        .expect_err("zero maximum elapsed seconds");
    assert_eq!(
        error.kind(),
        SourceValueErrorKind::ZeroMaximumElapsedSeconds
    );

    let error = SourceRetrievalPolicy::new(
        1,
        1,
        MAX_ELAPSED + 1,
        text_html.clone(),
        protocol,
        ip_policy,
    )
    .expect_err("too large maximum elapsed seconds");
    assert_eq!(
        error.kind(),
        SourceValueErrorKind::MaximumElapsedSecondsTooLarge {
            max_seconds: MAX_ELAPSED
        }
    );

    let error = SourceRetrievalPolicy::new(0, 0, 0, text_html.clone(), protocol, ip_policy)
        .expect_err("all zero");
    assert_eq!(
        error.kind(),
        SourceValueErrorKind::ZeroMinimumInterval,
        "minimum interval is checked first in declaration order"
    );

    let error = SourceRetrievalPolicy::new(1, 0, 0, text_html.clone(), protocol, ip_policy)
        .expect_err("zero maximum bytes and elapsed");
    assert_eq!(
        error.kind(),
        SourceValueErrorKind::ZeroMaximumResponseBytes,
        "maximum response bytes is checked before maximum elapsed seconds"
    );

    let error =
        SourceRetrievalPolicy::new(MAX_MIN_INTERVAL + 1, 0, 0, text_html, protocol, ip_policy)
            .expect_err("too large minimum with zero maximum");
    assert_eq!(
        error.kind(),
        SourceValueErrorKind::MinimumIntervalTooLarge {
            max_seconds: MAX_MIN_INTERVAL
        }
    );

    // Valid six-field policy: every accessor returns the exact constructed value.
    let policy = policy_with_media(media("application/json"));
    assert_eq!(policy.minimum_interval_seconds(), 21_600);
    assert_eq!(policy.maximum_response_bytes(), 131_072);
    assert_eq!(policy.maximum_elapsed_seconds(), 60);
    assert_eq!(*policy.expected_media_type(), media("application/json"));
    assert_eq!(
        policy.protocol_version(),
        SourceRetrievalProtocolVersion::V0StrictHttpsIpv4Http11_20260809
    );
    assert_eq!(
        policy.public_ip_policy_version(),
        PublicIpPolicyVersion::V0Ipv4Only20260809
    );

    // Proposal: revision 1, Proposed with no revision evidence.
    let definition = fixture_definition();
    assert_eq!(definition.authority_revision().get(), 1);
    assert_eq!(definition.status().kind(), SourceStatusKind::Proposed);
    assert!(matches!(
        definition.status(),
        SourceStatus::Proposed {
            revision_evidence: None
        }
    ));
    assert!(definition.prior_approval().is_none());
    assert_eq!(definition.source_id().as_str(), "example:source");
    assert_eq!(definition.owner().as_str(), "Example Campus Office");
    assert_eq!(
        definition.url().as_str(),
        "https://example.invalid/calendar/19081"
    );
    assert_eq!(
        definition.authority(),
        SourceAuthority::ReviewedOfficialSource
    );
    assert_eq!(definition.retrieval_policy(), &fixture_policy());

    // ModelInference is rejected by both constructors.
    let body_error = SourceDefinitionBody::new(
        owner("Example Campus Office"),
        url("https://example.invalid/a"),
        SourceAuthority::ModelInference,
        fixture_policy(),
    )
    .expect_err("body must reject ModelInference");
    assert_eq!(body_error.value_kind(), "SourceDefinitionBody");
    assert_eq!(body_error.kind(), SourceValueErrorKind::NonSourceAuthority);

    let definition_error = SourceDefinition::proposed(
        id("a"),
        owner("Example Campus Office"),
        url("https://example.invalid/a"),
        SourceAuthority::ModelInference,
        fixture_policy(),
    )
    .expect_err("proposed must reject ModelInference");
    assert_eq!(definition_error.value_kind(), "SourceDefinition");
    assert_eq!(
        definition_error.kind(),
        SourceValueErrorKind::NonSourceAuthority
    );

    // The concrete USTC candidate family stays Proposed only: synthetic URLs,
    // no approval, no retrieval.
    let mut registry = SourceRegistry::new();
    assert!(registry.is_empty());
    let candidate_2025 = proposed_definition(
        "ustc-teach-calendar-fall-2025",
        "https://example.invalid/calendar/19081",
    );
    let candidate_id_2025 = candidate_2025.source_id().clone();
    let candidate_2026 = proposed_definition(
        "ustc-teach-calendar-fall-2026",
        "https://example.invalid/calendar/20135",
    );
    let candidate_id_2026 = candidate_2026.source_id().clone();
    registry.propose(candidate_2025).expect("propose 2025");
    registry.propose(candidate_2026).expect("propose 2026");
    assert_eq!(registry.len(), 2);

    for candidate_id in [&candidate_id_2025, &candidate_id_2026] {
        let stored = registry.get(candidate_id).expect("candidate present");
        assert_eq!(stored.status().kind(), SourceStatusKind::Proposed);
        assert!(matches!(
            stored.status(),
            SourceStatus::Proposed {
                revision_evidence: None
            }
        ));
        assert_eq!(stored.authority_revision().get(), 1);
        let approved_error = registry
            .approved(candidate_id)
            .expect_err("candidates are never approved in this suite");
        assert_eq!(
            approved_error,
            SourceRegistryError::SourceNotRetrievable {
                source_id: candidate_id.clone(),
                status: SourceStatusKind::Proposed
            }
        );
    }
}

// ---------------------------------------------------------------------------
// 6. Approval with CAS and the retrieval gate.
// ---------------------------------------------------------------------------

#[test]
fn source_registry_approves_with_cas_and_retrieval_gate() {
    let mut registry = SourceRegistry::new();
    let definition = fixture_definition();
    let source_id = definition.source_id().clone();
    registry.propose(definition).expect("propose");

    // Proposed: not approved, not retrievable.
    let approved_error = registry
        .approved(&source_id)
        .expect_err("proposed must not be approved");
    assert_eq!(
        approved_error,
        SourceRegistryError::SourceNotRetrievable {
            source_id: source_id.clone(),
            status: SourceStatusKind::Proposed
        }
    );
    let subject_error = registry
        .retrieval_subject(&source_id)
        .expect_err("proposed must not be retrievable");
    assert_eq!(
        subject_error,
        SourceRegistryError::SourceNotRetrievable {
            source_id: source_id.clone(),
            status: SourceStatusKind::Proposed
        }
    );

    // Approval under exact CAS: revision 1 -> 2.
    let revision = current_revision(&registry, &source_id);
    assert_eq!(revision.get(), 1);
    let approved = registry
        .approve(&source_id, revision, fixture_receipt())
        .expect("approve");
    assert_eq!(approved.status().kind(), SourceStatusKind::Approved);
    assert_eq!(approved.authority_revision().get(), 2);
    match approved.status() {
        SourceStatus::Approved { receipt } => {
            assert_eq!(receipt.reviewer().as_str(), "reviewer:operator");
        }
        _ => panic!("must be approved"),
    }
    assert!(approved.prior_approval().is_some());

    // A second approve is SourceAlreadyApproved and preserves the first receipt.
    let error = registry
        .approve(
            &source_id,
            current_revision(&registry, &source_id),
            other_receipt(),
        )
        .expect_err("second approve must fail");
    assert_eq!(
        error,
        SourceRegistryError::SourceAlreadyApproved {
            source_id: source_id.clone()
        }
    );
    let stored = registry.get(&source_id).expect("present");
    assert_eq!(stored.authority_revision().get(), 2);
    match stored.status() {
        SourceStatus::Approved { receipt } => {
            assert_eq!(
                receipt.reviewer().as_str(),
                "reviewer:operator",
                "first receipt must be preserved"
            );
        }
        _ => panic!("must still be approved"),
    }

    // The retrieval subject is now available and carries the current revision.
    let subject = registry.retrieval_subject(&source_id).expect("subject");
    assert_eq!(subject.source_id(), &source_id);
    assert_eq!(
        subject.source_url().as_str(),
        "https://example.invalid/calendar/19081"
    );
    assert_eq!(subject.source_authority_revision().get(), 2);
    assert_eq!(
        subject.source_retrieval_policy(),
        registry
            .get(&source_id)
            .expect("present")
            .retrieval_policy()
    );
    let approved_ref = registry.approved(&source_id).expect("approved");
    assert_eq!(approved_ref.authority_revision().get(), 2);
}

#[test]
fn source_registry_reproposes_cloned_authority_as_fresh_proposed_definition() {
    let mut authority_registry = SourceRegistry::new();
    let source_id = id("example:source");
    authority_registry
        .propose(fixture_definition())
        .expect("propose under authority registry");
    authority_registry
        .approve(
            &source_id,
            current_revision(&authority_registry, &source_id),
            fixture_receipt(),
        )
        .expect("approve under authority registry");

    let approved_snapshot = authority_registry
        .get(&source_id)
        .expect("approved source")
        .clone();
    assert_eq!(
        approved_snapshot.status().kind(),
        SourceStatusKind::Approved
    );
    assert_eq!(approved_snapshot.authority_revision().get(), 2);
    let expected_owner = approved_snapshot.owner().clone();
    let expected_url = approved_snapshot.url().clone();
    let expected_authority = approved_snapshot.authority();
    let expected_policy = approved_snapshot.retrieval_policy().clone();

    let mut receiving_registry = SourceRegistry::new();
    receiving_registry
        .propose(approved_snapshot)
        .expect("reproposal canonicalizes lifecycle authority");

    let admitted = receiving_registry.get(&source_id).expect("admitted source");
    assert_eq!(admitted.authority_revision().get(), 1);
    assert_eq!(admitted.source_id(), &source_id);
    assert_eq!(admitted.owner(), &expected_owner);
    assert_eq!(admitted.url(), &expected_url);
    assert_eq!(admitted.authority(), expected_authority);
    assert_eq!(admitted.retrieval_policy(), &expected_policy);
    assert_eq!(
        admitted.status(),
        &SourceStatus::Proposed {
            revision_evidence: None
        }
    );
    assert!(admitted.prior_approval().is_none());
    assert_eq!(
        receiving_registry
            .retrieval_subject(&source_id)
            .expect_err("reproposal must not import retrieval authority"),
        SourceRegistryError::SourceNotRetrievable {
            source_id: source_id.clone(),
            status: SourceStatusKind::Proposed
        }
    );

    let original = authority_registry.get(&source_id).expect("original source");
    assert_eq!(original.authority_revision().get(), 2);
    assert_eq!(original.status().kind(), SourceStatusKind::Approved);
}

// ---------------------------------------------------------------------------
// 7. Suspension with CAS preserves the approval receipt.
// ---------------------------------------------------------------------------

#[test]
fn source_registry_suspends_with_cas_and_preserves_approval() {
    let mut registry = SourceRegistry::new();
    registry.propose(fixture_definition()).expect("propose");
    let source_id = id("example:source");

    registry
        .approve(
            &source_id,
            current_revision(&registry, &source_id),
            fixture_receipt(),
        )
        .expect("approve");
    let revision = current_revision(&registry, &source_id);
    assert_eq!(revision.get(), 2);

    let suspended = registry
        .suspend(&source_id, revision, status_evidence("evidence:suspend"))
        .expect("suspend");
    assert_eq!(suspended.status().kind(), SourceStatusKind::Suspended);
    assert_eq!(suspended.authority_revision().get(), 3);
    match suspended.status() {
        SourceStatus::Suspended { approval, evidence } => {
            assert_eq!(
                approval.reviewer().as_str(),
                "reviewer:operator",
                "suspension must preserve the complete approval receipt"
            );
            assert_eq!(evidence.as_str(), "evidence:suspend");
        }
        _ => panic!("must be suspended"),
    }
    assert!(suspended.prior_approval().is_some());

    // Suspended blocks retrieval and approval.
    let subject_error = registry
        .retrieval_subject(&source_id)
        .expect_err("suspended must not be retrievable");
    assert_eq!(
        subject_error,
        SourceRegistryError::SourceNotRetrievable {
            source_id: source_id.clone(),
            status: SourceStatusKind::Suspended
        }
    );
    let approved_error = registry
        .approved(&source_id)
        .expect_err("suspended is not approved");
    assert_eq!(
        approved_error,
        SourceRegistryError::SourceNotRetrievable {
            source_id: source_id.clone(),
            status: SourceStatusKind::Suspended
        }
    );
    assert_eq!(registry.len(), 1);
}

// ---------------------------------------------------------------------------
// 8. Reinstatement with CAS consumes a complete new receipt.
// ---------------------------------------------------------------------------

#[test]
fn source_registry_reapproves_with_cas_and_new_receipt() {
    let mut registry = SourceRegistry::new();
    registry.propose(fixture_definition()).expect("propose");
    let source_id = id("example:source");

    registry
        .approve(
            &source_id,
            current_revision(&registry, &source_id),
            fixture_receipt(),
        )
        .expect("approve");
    assert_eq!(current_revision(&registry, &source_id).get(), 2);
    registry
        .suspend(
            &source_id,
            current_revision(&registry, &source_id),
            fixture_evidence(),
        )
        .expect("suspend");
    assert_eq!(current_revision(&registry, &source_id).get(), 3);

    let revision = current_revision(&registry, &source_id);
    let reinstated = registry
        .reinstate(&source_id, revision, other_receipt())
        .expect("reinstate");
    assert_eq!(reinstated.status().kind(), SourceStatusKind::Approved);
    assert_eq!(reinstated.authority_revision().get(), 4);
    match reinstated.status() {
        SourceStatus::Approved { receipt } => {
            assert_eq!(
                receipt.reviewer().as_str(),
                "reviewer:different",
                "reinstatement must consume the complete new receipt"
            );
        }
        _ => panic!("must be approved"),
    }
    assert!(reinstated.prior_approval().is_some());

    let subject = registry.retrieval_subject(&source_id).expect("subject");
    assert_eq!(subject.source_authority_revision().get(), 4);
    let approved = registry.approved(&source_id).expect("approved");
    assert_eq!(approved.authority_revision().get(), 4);
}

// ---------------------------------------------------------------------------
// 9. Revision from every allowed state resets to Proposed with evidence.
// ---------------------------------------------------------------------------

#[test]
fn source_registry_revises_from_allowed_states_and_resets_to_proposed() {
    // From Proposed; reusing the same source's current URL is not a duplicate.
    let mut registry = SourceRegistry::new();
    registry.propose(fixture_definition()).expect("propose");
    let source_id = id("example:source");

    let revised = registry
        .revise(
            &source_id,
            current_revision(&registry, &source_id),
            body(
                "New Campus Office",
                "https://example.invalid/calendar/19081",
                SourceAuthority::CommunitySignal,
            ),
            status_evidence("evidence:revise-1"),
        )
        .expect("revise from proposed");
    assert_eq!(revised.status().kind(), SourceStatusKind::Proposed);
    match revised.status() {
        SourceStatus::Proposed { revision_evidence } => {
            assert_eq!(
                revision_evidence.as_ref().expect("evidence").as_str(),
                "evidence:revise-1"
            );
        }
        _ => panic!("must be proposed"),
    }
    assert_eq!(revised.authority_revision().get(), 2);
    assert_eq!(revised.owner().as_str(), "New Campus Office");
    assert_eq!(revised.authority(), SourceAuthority::CommunitySignal);
    assert_eq!(
        revised.url().as_str(),
        "https://example.invalid/calendar/19081",
        "same-source URL reuse must be allowed"
    );
    assert_eq!(revised.source_id().as_str(), "example:source");
    assert!(revised.prior_approval().is_none());

    // A later revise to a fresh URL also succeeds from Proposed.
    let revised = registry
        .revise(
            &source_id,
            current_revision(&registry, &source_id),
            body(
                "New Campus Office",
                "https://example.invalid/calendar/20135",
                SourceAuthority::CommunitySignal,
            ),
            status_evidence("evidence:revise-2"),
        )
        .expect("revise to fresh url");
    assert_eq!(
        revised.url().as_str(),
        "https://example.invalid/calendar/20135"
    );
    assert_eq!(revised.authority_revision().get(), 3);

    // From Approved: the approval is dropped, status returns to Proposed.
    let mut registry = SourceRegistry::new();
    registry
        .propose(proposed_definition(
            "example:approved",
            "https://example.invalid/a",
        ))
        .expect("propose");
    let approved_id = id("example:approved");
    registry
        .approve(
            &approved_id,
            current_revision(&registry, &approved_id),
            fixture_receipt(),
        )
        .expect("approve");
    let revised = registry
        .revise(
            &approved_id,
            current_revision(&registry, &approved_id),
            body(
                "Other Office",
                "https://example.invalid/b",
                SourceAuthority::ReviewedOfficialSource,
            ),
            status_evidence("evidence:revise-3"),
        )
        .expect("revise from approved");
    assert_eq!(revised.status().kind(), SourceStatusKind::Proposed);
    assert_eq!(revised.authority_revision().get(), 3);
    assert!(
        revised.prior_approval().is_none(),
        "revise must drop the approval"
    );

    // From Suspended.
    let mut registry = SourceRegistry::new();
    registry
        .propose(proposed_definition(
            "example:suspended",
            "https://example.invalid/c",
        ))
        .expect("propose");
    let suspended_id = id("example:suspended");
    registry
        .approve(
            &suspended_id,
            current_revision(&registry, &suspended_id),
            fixture_receipt(),
        )
        .expect("approve");
    registry
        .suspend(
            &suspended_id,
            current_revision(&registry, &suspended_id),
            fixture_evidence(),
        )
        .expect("suspend");
    let revised = registry
        .revise(
            &suspended_id,
            current_revision(&registry, &suspended_id),
            body(
                "Third Office",
                "https://example.invalid/d",
                SourceAuthority::ReviewedOfficialSource,
            ),
            status_evidence("evidence:revise-4"),
        )
        .expect("revise from suspended");
    assert_eq!(revised.status().kind(), SourceStatusKind::Proposed);
    assert_eq!(revised.authority_revision().get(), 4);
}

// ---------------------------------------------------------------------------
// 10. Revise to another source's URL is rejected atomically.
// ---------------------------------------------------------------------------

#[test]
fn source_registry_rejects_revise_url_collision_atomically() {
    let mut registry = SourceRegistry::new();
    registry
        .propose(proposed_definition(
            "example:one",
            "https://example.invalid/one",
        ))
        .expect("propose one");
    let one_id = id("example:one");
    registry
        .approve(
            &one_id,
            current_revision(&registry, &one_id),
            fixture_receipt(),
        )
        .expect("approve one");
    registry
        .propose(proposed_definition(
            "example:two",
            "https://example.invalid/two",
        ))
        .expect("propose two");
    let two_id = id("example:two");
    let two_revision = current_revision(&registry, &two_id);
    assert_eq!(two_revision.get(), 1);

    // Revise two onto one's URL: rejected without mutation.
    let error = registry
        .revise(
            &two_id,
            two_revision,
            body(
                "Two Owner",
                "https://example.invalid/one",
                SourceAuthority::ReviewedOfficialSource,
            ),
            fixture_evidence(),
        )
        .expect_err("URL collision must be rejected");
    assert_eq!(
        error,
        SourceRegistryError::DuplicateUrl {
            url: url("https://example.invalid/one")
        }
    );
    let two = registry.get(&two_id).expect("two present");
    assert_eq!(two.url().as_str(), "https://example.invalid/two");
    assert_eq!(two.owner().as_str(), "Example Campus Office");
    assert_eq!(two.authority_revision().get(), 1);
    assert_eq!(two.status().kind(), SourceStatusKind::Proposed);
    assert!(matches!(
        two.status(),
        SourceStatus::Proposed {
            revision_evidence: None
        }
    ));
    assert_eq!(registry.len(), 2);

    // The same expected revision is still valid: the failed attempt consumed
    // no revision.
    let revised = registry
        .revise(
            &two_id,
            two_revision,
            body(
                "Two Owner",
                "https://example.invalid/three",
                SourceAuthority::ReviewedOfficialSource,
            ),
            fixture_evidence(),
        )
        .expect("revise after the failed collision");
    assert_eq!(revised.url().as_str(), "https://example.invalid/three");
    assert_eq!(revised.authority_revision().get(), 2);

    // Collision against a proposed source's URL is rejected as well.
    registry
        .propose(proposed_definition(
            "example:four",
            "https://example.invalid/four",
        ))
        .expect("propose four");
    let error = registry
        .revise(
            &two_id,
            current_revision(&registry, &two_id),
            body(
                "Two Owner",
                "https://example.invalid/four",
                SourceAuthority::ReviewedOfficialSource,
            ),
            fixture_evidence(),
        )
        .expect_err("collision with a proposed source must be rejected");
    assert_eq!(
        error,
        SourceRegistryError::DuplicateUrl {
            url: url("https://example.invalid/four")
        }
    );
    let two = registry.get(&two_id).expect("two present");
    assert_eq!(two.url().as_str(), "https://example.invalid/three");
    assert_eq!(two.authority_revision().get(), 2);
}

// ---------------------------------------------------------------------------
// 11. Revocation from every allowed state; terminal behavior.
// ---------------------------------------------------------------------------

#[test]
fn source_registry_revokes_from_allowed_states_and_is_terminal() {
    // From Proposed: prior approval is None.
    let mut registry = SourceRegistry::new();
    registry.propose(fixture_definition()).expect("propose");
    let source_id = id("example:source");
    registry
        .revoke(
            &source_id,
            current_revision(&registry, &source_id),
            status_evidence("evidence:revoke"),
        )
        .expect("revoke from proposed");
    let revoked = registry.get(&source_id).expect("present");
    match revoked.status() {
        SourceStatus::Revoked {
            prior_approval,
            evidence,
        } => {
            assert!(
                prior_approval.is_none(),
                "proposed revocation carries no approval"
            );
            assert_eq!(evidence.as_str(), "evidence:revoke");
        }
        _ => panic!("must be revoked"),
    }
    assert_eq!(revoked.authority_revision().get(), 2);

    // Terminal: every command from Revoked is rejected and mutates nothing.
    let revision = current_revision(&registry, &source_id);
    let error = registry
        .approve(&source_id, revision, fixture_receipt())
        .expect_err("terminal approve");
    assert_eq!(
        error,
        SourceRegistryError::IllegalTransition {
            status: SourceStatusKind::Revoked,
            command: SourceTransitionCommand::Approve
        }
    );
    let error = registry
        .revise(
            &source_id,
            revision,
            body(
                "Owner",
                "https://example.invalid/terminal",
                SourceAuthority::CommunitySignal,
            ),
            fixture_evidence(),
        )
        .expect_err("terminal revise");
    assert_eq!(
        error,
        SourceRegistryError::IllegalTransition {
            status: SourceStatusKind::Revoked,
            command: SourceTransitionCommand::Revise
        }
    );
    let error = registry
        .suspend(&source_id, revision, fixture_evidence())
        .expect_err("terminal suspend");
    assert_eq!(
        error,
        SourceRegistryError::IllegalTransition {
            status: SourceStatusKind::Revoked,
            command: SourceTransitionCommand::Suspend
        }
    );
    let error = registry
        .reinstate(&source_id, revision, fixture_receipt())
        .expect_err("terminal reinstate");
    assert_eq!(
        error,
        SourceRegistryError::IllegalTransition {
            status: SourceStatusKind::Revoked,
            command: SourceTransitionCommand::Reinstate
        }
    );
    let error = registry
        .revoke(&source_id, revision, fixture_evidence())
        .expect_err("terminal re-revoke");
    assert_eq!(
        error,
        SourceRegistryError::IllegalTransition {
            status: SourceStatusKind::Revoked,
            command: SourceTransitionCommand::Revoke
        }
    );
    let stored = registry.get(&source_id).expect("present");
    assert_eq!(stored.authority_revision().get(), 2);
    assert_eq!(stored.status().kind(), SourceStatusKind::Revoked);
    assert_eq!(registry.len(), 1);

    // From Approved: prior approval is Some(current receipt).
    let mut registry = SourceRegistry::new();
    registry
        .propose(proposed_definition(
            "example:approved",
            "https://example.invalid/a",
        ))
        .expect("propose");
    let approved_id = id("example:approved");
    registry
        .approve(
            &approved_id,
            current_revision(&registry, &approved_id),
            fixture_receipt(),
        )
        .expect("approve");
    registry
        .revoke(
            &approved_id,
            current_revision(&registry, &approved_id),
            fixture_evidence(),
        )
        .expect("revoke from approved");
    let revoked = registry.get(&approved_id).expect("present");
    match revoked.status() {
        SourceStatus::Revoked { prior_approval, .. } => {
            let receipt = prior_approval
                .as_ref()
                .expect("approved revocation keeps receipt");
            assert_eq!(receipt.reviewer().as_str(), "reviewer:operator");
        }
        _ => panic!("must be revoked"),
    }
    assert_eq!(revoked.authority_revision().get(), 3);

    // From Suspended: prior approval is Some(preserved approval).
    let mut registry = SourceRegistry::new();
    registry
        .propose(proposed_definition(
            "example:suspended",
            "https://example.invalid/c",
        ))
        .expect("propose");
    let suspended_id = id("example:suspended");
    registry
        .approve(
            &suspended_id,
            current_revision(&registry, &suspended_id),
            fixture_receipt(),
        )
        .expect("approve");
    registry
        .suspend(
            &suspended_id,
            current_revision(&registry, &suspended_id),
            fixture_evidence(),
        )
        .expect("suspend");
    registry
        .revoke(
            &suspended_id,
            current_revision(&registry, &suspended_id),
            fixture_evidence(),
        )
        .expect("revoke from suspended");
    let revoked = registry.get(&suspended_id).expect("present");
    match revoked.status() {
        SourceStatus::Revoked { prior_approval, .. } => {
            let receipt = prior_approval
                .as_ref()
                .expect("suspended revocation keeps approval");
            assert_eq!(receipt.reviewer().as_str(), "reviewer:operator");
        }
        _ => panic!("must be revoked"),
    }
    assert_eq!(revoked.authority_revision().get(), 4);

    // Revoked is not retrievable and not approved.
    let subject_error = registry
        .retrieval_subject(&suspended_id)
        .expect_err("revoked is not retrievable");
    assert_eq!(
        subject_error,
        SourceRegistryError::SourceNotRetrievable {
            source_id: suspended_id.clone(),
            status: SourceStatusKind::Revoked
        }
    );
    let approved_error = registry
        .approved(&suspended_id)
        .expect_err("revoked is not approved");
    assert_eq!(
        approved_error,
        SourceRegistryError::SourceNotRetrievable {
            source_id: suspended_id.clone(),
            status: SourceStatusKind::Revoked
        }
    );
}

// ---------------------------------------------------------------------------
// 12. Unsupported transitions are rejected atomically.
// ---------------------------------------------------------------------------

#[test]
fn source_registry_rejects_illegal_transitions_atomically() {
    let mut registry = SourceRegistry::new();
    registry.propose(fixture_definition()).expect("propose");
    let source_id = id("example:source");
    let revision = current_revision(&registry, &source_id);
    assert_eq!(revision.get(), 1);

    // suspend from Proposed.
    let error = registry
        .suspend(&source_id, revision, fixture_evidence())
        .expect_err("suspend from proposed");
    assert_eq!(
        error,
        SourceRegistryError::IllegalTransition {
            status: SourceStatusKind::Proposed,
            command: SourceTransitionCommand::Suspend
        }
    );

    // reinstate from Proposed.
    let error = registry
        .reinstate(&source_id, revision, fixture_receipt())
        .expect_err("reinstate from proposed");
    assert_eq!(
        error,
        SourceRegistryError::IllegalTransition {
            status: SourceStatusKind::Proposed,
            command: SourceTransitionCommand::Reinstate
        }
    );

    // Failed mutations changed nothing.
    let stored = registry.get(&source_id).expect("present");
    assert_eq!(stored.authority_revision().get(), 1);
    assert_eq!(stored.status().kind(), SourceStatusKind::Proposed);

    // approve on Approved is SourceAlreadyApproved, not IllegalTransition.
    registry
        .approve(&source_id, revision, fixture_receipt())
        .expect("approve");
    let revision = current_revision(&registry, &source_id);
    let error = registry
        .approve(&source_id, revision, other_receipt())
        .expect_err("approve on approved");
    assert_eq!(
        error,
        SourceRegistryError::SourceAlreadyApproved {
            source_id: source_id.clone()
        }
    );

    // reinstate from Approved.
    let error = registry
        .reinstate(&source_id, revision, other_receipt())
        .expect_err("reinstate from approved");
    assert_eq!(
        error,
        SourceRegistryError::IllegalTransition {
            status: SourceStatusKind::Approved,
            command: SourceTransitionCommand::Reinstate
        }
    );
    let stored = registry.get(&source_id).expect("present");
    assert_eq!(stored.authority_revision().get(), 2);
    assert_eq!(stored.status().kind(), SourceStatusKind::Approved);

    // suspend from Suspended.
    registry
        .suspend(&source_id, revision, fixture_evidence())
        .expect("suspend");
    let revision = current_revision(&registry, &source_id);
    assert_eq!(revision.get(), 3);
    let error = registry
        .suspend(&source_id, revision, fixture_evidence())
        .expect_err("suspend from suspended");
    assert_eq!(
        error,
        SourceRegistryError::IllegalTransition {
            status: SourceStatusKind::Suspended,
            command: SourceTransitionCommand::Suspend
        }
    );

    // approve from Suspended.
    let error = registry
        .approve(&source_id, revision, fixture_receipt())
        .expect_err("approve from suspended");
    assert_eq!(
        error,
        SourceRegistryError::IllegalTransition {
            status: SourceStatusKind::Suspended,
            command: SourceTransitionCommand::Approve
        }
    );

    // Failed mutations changed nothing.
    let stored = registry.get(&source_id).expect("present");
    assert_eq!(stored.authority_revision().get(), 3);
    assert_eq!(stored.status().kind(), SourceStatusKind::Suspended);
    assert_eq!(registry.len(), 1);
}

// ---------------------------------------------------------------------------
// 13. Stale expected revision rejects every mutation family atomically.
// ---------------------------------------------------------------------------

#[test]
fn source_registry_rejects_stale_revision_atomically() {
    let mut registry = SourceRegistry::new();

    // alpha walks to Approved at revision 2.
    registry
        .propose(proposed_definition(
            "example:alpha",
            "https://example.invalid/alpha",
        ))
        .expect("propose alpha");
    let alpha = id("example:alpha");
    let alpha_stale = current_revision(&registry, &alpha);
    assert_eq!(alpha_stale.get(), 1);
    registry
        .approve(&alpha, alpha_stale, fixture_receipt())
        .expect("approve alpha");

    // beta stays Proposed at revision 1.
    registry
        .propose(proposed_definition(
            "example:beta",
            "https://example.invalid/beta",
        ))
        .expect("propose beta");
    let beta = id("example:beta");
    let beta_revision = current_revision(&registry, &beta);
    assert_eq!(beta_revision.get(), 1);

    // approve family: beta is Proposed (approve is legal) and a foreign
    // current revision is stale.
    let foreign_revision = current_revision(&registry, &alpha);
    let error = registry
        .approve(&beta, foreign_revision, fixture_receipt())
        .expect_err("stale approve");
    assert_eq!(
        error,
        SourceRegistryError::StaleAuthorityRevision {
            expected: foreign_revision,
            actual: beta_revision
        }
    );

    // revise family: alpha is Approved (revise is legal), revision 1 is stale.
    let error = registry
        .revise(
            &alpha,
            alpha_stale,
            body(
                "Alpha Owner",
                "https://example.invalid/stale",
                SourceAuthority::CommunitySignal,
            ),
            fixture_evidence(),
        )
        .expect_err("stale revise");
    assert_eq!(
        error,
        SourceRegistryError::StaleAuthorityRevision {
            expected: alpha_stale,
            actual: current_revision(&registry, &alpha)
        }
    );

    // suspend family.
    let error = registry
        .suspend(&alpha, alpha_stale, fixture_evidence())
        .expect_err("stale suspend");
    assert_eq!(
        error,
        SourceRegistryError::StaleAuthorityRevision {
            expected: alpha_stale,
            actual: current_revision(&registry, &alpha)
        }
    );

    // revoke family.
    let error = registry
        .revoke(&alpha, alpha_stale, fixture_evidence())
        .expect_err("stale revoke");
    assert_eq!(
        error,
        SourceRegistryError::StaleAuthorityRevision {
            expected: alpha_stale,
            actual: current_revision(&registry, &alpha)
        }
    );

    // reinstate family: gamma walks to Suspended; its pre-suspension revision
    // is stale for reinstate.
    registry
        .propose(proposed_definition(
            "example:gamma",
            "https://example.invalid/gamma",
        ))
        .expect("propose gamma");
    let gamma = id("example:gamma");
    registry
        .approve(
            &gamma,
            current_revision(&registry, &gamma),
            fixture_receipt(),
        )
        .expect("approve gamma");
    let gamma_stale = current_revision(&registry, &gamma);
    assert_eq!(gamma_stale.get(), 2);
    registry
        .suspend(&gamma, gamma_stale, fixture_evidence())
        .expect("suspend gamma");
    let error = registry
        .reinstate(&gamma, gamma_stale, fixture_receipt())
        .expect_err("stale reinstate");
    assert_eq!(
        error,
        SourceRegistryError::StaleAuthorityRevision {
            expected: gamma_stale,
            actual: current_revision(&registry, &gamma)
        }
    );

    // Atomicity: every source keeps its exact revision, status and length.
    let alpha_stored = registry.get(&alpha).expect("alpha present");
    assert_eq!(alpha_stored.authority_revision().get(), 2);
    assert_eq!(alpha_stored.status().kind(), SourceStatusKind::Approved);
    let beta_stored = registry.get(&beta).expect("beta present");
    assert_eq!(beta_stored.authority_revision().get(), 1);
    assert_eq!(beta_stored.status().kind(), SourceStatusKind::Proposed);
    assert!(matches!(
        beta_stored.status(),
        SourceStatus::Proposed {
            revision_evidence: None
        }
    ));
    let gamma_stored = registry.get(&gamma).expect("gamma present");
    assert_eq!(gamma_stored.authority_revision().get(), 3);
    assert_eq!(gamma_stored.status().kind(), SourceStatusKind::Suspended);
    assert_eq!(registry.len(), 3);
}

// ---------------------------------------------------------------------------
// 14. Exact monotone revisions across every successful mutation family.
// ---------------------------------------------------------------------------

#[test]
fn source_registry_rejects_revision_overflow_atomically() {
    // The public API cannot construct an exhausted revision, so this suite
    // binds the exact non-zero monotone +1 increments of every post-proposal
    // mutation family; the u64::MAX exhaustion case is bound by the internal
    // unit fixture in src/source_registry.rs.
    let mut registry = SourceRegistry::new();
    registry.propose(fixture_definition()).expect("propose");
    let source_id = id("example:source");

    let revision = current_revision(&registry, &source_id);
    assert_eq!(revision.get(), 1);
    let approved = registry
        .approve(&source_id, revision, fixture_receipt())
        .expect("approve");
    assert_eq!(approved.authority_revision().get(), 2);

    let revision = current_revision(&registry, &source_id);
    let suspended = registry
        .suspend(&source_id, revision, fixture_evidence())
        .expect("suspend");
    assert_eq!(suspended.authority_revision().get(), 3);

    let revision = current_revision(&registry, &source_id);
    let reinstated = registry
        .reinstate(&source_id, revision, other_receipt())
        .expect("reinstate");
    assert_eq!(reinstated.authority_revision().get(), 4);

    let revision = current_revision(&registry, &source_id);
    let revised = registry
        .revise(
            &source_id,
            revision,
            body(
                "Revised Office",
                "https://example.invalid/revised",
                SourceAuthority::CommunitySignal,
            ),
            fixture_evidence(),
        )
        .expect("revise");
    assert_eq!(revised.authority_revision().get(), 5);

    let revision = current_revision(&registry, &source_id);
    let reapproved = registry
        .approve(&source_id, revision, fixture_receipt())
        .expect("re-approve");
    assert_eq!(reapproved.authority_revision().get(), 6);

    let revision = current_revision(&registry, &source_id);
    let revoked = registry
        .revoke(&source_id, revision, fixture_evidence())
        .expect("revoke");
    assert_eq!(revoked.authority_revision().get(), 7);

    // Revisions are non-zero, strictly monotone, and never reset.
    let stored = registry.get(&source_id).expect("present");
    assert_eq!(stored.authority_revision().get(), 7);
    assert!(stored.authority_revision().get() > 0);
}

// ---------------------------------------------------------------------------
// 15. Duplicate SourceId/URL rejection, with proposal precedence.
// ---------------------------------------------------------------------------

#[test]
fn source_registry_rejects_duplicate_id_and_url_atomically() {
    let mut registry = SourceRegistry::new();
    let first = fixture_definition();
    let first_id = first.source_id().clone();
    let first_url = first.url().clone();
    registry.propose(first).expect("propose");

    // Duplicate SourceId with a different URL.
    let same_id = SourceDefinition::proposed(
        first_id.clone(),
        owner("Other Office"),
        url("https://example.invalid/other"),
        SourceAuthority::ReviewedOfficialSource,
        fixture_policy(),
    )
    .expect("fixture definition");
    let error = registry.propose(same_id).expect_err("duplicate id");
    assert_eq!(
        error,
        SourceRegistryError::DuplicateSource {
            source_id: first_id.clone()
        }
    );

    // Different SourceId with the same canonical URL.
    let same_url = proposed_definition("example:other", "https://example.invalid/calendar/19081");
    let error = registry.propose(same_url).expect_err("duplicate url");
    assert_eq!(
        error,
        SourceRegistryError::DuplicateUrl {
            url: first_url.clone()
        }
    );

    // Duplicate SourceId and URL together: DuplicateSource is checked first.
    let both = proposed_definition("example:source", "https://example.invalid/calendar/19081");
    let error = registry.propose(both).expect_err("duplicate id and url");
    assert_eq!(
        error,
        SourceRegistryError::DuplicateSource {
            source_id: first_id.clone()
        },
        "DuplicateSource is checked before DuplicateUrl"
    );

    // Atomicity: the first definition is untouched.
    assert_eq!(registry.len(), 1);
    let stored = registry.get(&first_id).expect("present");
    assert_eq!(
        stored.url().as_str(),
        "https://example.invalid/calendar/19081"
    );
    assert_eq!(stored.owner().as_str(), "Example Campus Office");
    assert_eq!(stored.authority_revision().get(), 1);
    assert_eq!(stored.status().kind(), SourceStatusKind::Proposed);
}

// ---------------------------------------------------------------------------
// 16. Retrieval exposes only the approved definition at its current revision.
// ---------------------------------------------------------------------------

#[test]
fn source_registry_exposes_only_the_approved_current_revision() {
    let mut registry = SourceRegistry::new();

    // Missing: SourceNotFound for both gates.
    let missing = id("example:missing");
    let error = registry
        .retrieval_subject(&missing)
        .expect_err("missing must not be retrievable");
    assert_eq!(
        error,
        SourceRegistryError::SourceNotFound {
            source_id: missing.clone()
        }
    );
    let error = registry
        .approved(&missing)
        .expect_err("missing must not be approved");
    assert_eq!(
        error,
        SourceRegistryError::SourceNotFound {
            source_id: missing.clone()
        }
    );

    // Proposed: gated.
    registry.propose(fixture_definition()).expect("propose");
    let source_id = id("example:source");
    let error = registry
        .retrieval_subject(&source_id)
        .expect_err("proposed must not be retrievable");
    assert_eq!(
        error,
        SourceRegistryError::SourceNotRetrievable {
            source_id: source_id.clone(),
            status: SourceStatusKind::Proposed
        }
    );
    let error = registry
        .approved(&source_id)
        .expect_err("proposed must not be approved");
    assert_eq!(
        error,
        SourceRegistryError::SourceNotRetrievable {
            source_id: source_id.clone(),
            status: SourceStatusKind::Proposed
        }
    );

    // Approved: the subject carries the current revision.
    registry
        .approve(
            &source_id,
            current_revision(&registry, &source_id),
            fixture_receipt(),
        )
        .expect("approve");
    let subject = registry.retrieval_subject(&source_id).expect("subject");
    assert_eq!(subject.source_authority_revision().get(), 2);
    assert_eq!(subject.source_id(), &source_id);
    assert_eq!(
        subject.source_url().as_str(),
        "https://example.invalid/calendar/19081"
    );
    let approved = registry.approved(&source_id).expect("approved");
    assert_eq!(approved.authority_revision().get(), 2);

    // Suspended: gated again.
    registry
        .suspend(
            &source_id,
            current_revision(&registry, &source_id),
            fixture_evidence(),
        )
        .expect("suspend");
    let error = registry
        .retrieval_subject(&source_id)
        .expect_err("suspended must not be retrievable");
    assert_eq!(
        error,
        SourceRegistryError::SourceNotRetrievable {
            source_id: source_id.clone(),
            status: SourceStatusKind::Suspended
        }
    );
    let error = registry
        .approved(&source_id)
        .expect_err("suspended must not be approved");
    assert_eq!(
        error,
        SourceRegistryError::SourceNotRetrievable {
            source_id: source_id.clone(),
            status: SourceStatusKind::Suspended
        }
    );

    // Reinstated: retrievable again with the new current revision.
    registry
        .reinstate(
            &source_id,
            current_revision(&registry, &source_id),
            fixture_receipt(),
        )
        .expect("reinstate");
    let subject = registry
        .retrieval_subject(&source_id)
        .expect("subject after reinstate");
    assert_eq!(subject.source_authority_revision().get(), 4);
    let approved = registry
        .approved(&source_id)
        .expect("approved after reinstate");
    assert_eq!(approved.authority_revision().get(), 4);

    // Revoked: gated terminally.
    registry
        .revoke(
            &source_id,
            current_revision(&registry, &source_id),
            fixture_evidence(),
        )
        .expect("revoke");
    let error = registry
        .retrieval_subject(&source_id)
        .expect_err("revoked must not be retrievable");
    assert_eq!(
        error,
        SourceRegistryError::SourceNotRetrievable {
            source_id: source_id.clone(),
            status: SourceStatusKind::Revoked
        }
    );
    let error = registry
        .approved(&source_id)
        .expect_err("revoked must not be approved");
    assert_eq!(
        error,
        SourceRegistryError::SourceNotRetrievable {
            source_id: source_id.clone(),
            status: SourceStatusKind::Revoked
        }
    );
}
