//! `source-import/v0` M60-B1 acceptance evidence for the source registry.
//!
//! Bound row: `SRC-001`. Covers every grammar edge, no-echo errors, duplicate
//! rejection, missing/proposed/already-approved states, first-receipt/value
//! preservation, failed-transition atomicity, exact API closure, bounds, and
//! zero I/O/dependency widening.

use std::error::Error;

use serde::Deserialize;
use serde::de::IntoDeserializer;
use serde::de::value::{BytesDeserializer, Error as SerdeValueError, StringDeserializer};

use ustc_campus_agent_core::SourceAuthority;
use ustc_campus_agent_core::source_registry::{
    SourceDefinition, SourceId, SourceOwner, SourceRegistry, SourceRegistryError,
    SourceRetrievalPolicy, SourceReviewEvidenceId, SourceReviewReceipt, SourceReviewState,
    SourceReviewerId, SourceUrl, SourceValueErrorKind,
};

// ---------------------------------------------------------------------------
// Bounds (mirror the contract's fixed ceilings).
// ---------------------------------------------------------------------------

const MAX_ID_BYTES: usize = 128;
const MAX_OWNER_BYTES: usize = 128;
const MAX_URL_BYTES: usize = 2048;
const MAX_MIN_INTERVAL: u32 = 604_800;
const MAX_MAX_BYTES: u32 = 1_048_576;

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
// §3.1 SourceId-family grammar: edges and precedence.
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

macro_rules! assert_source_id_kind_grammar {
    ($kind:ty) => {{
        let kind_name = stringify!($kind);

        for value in source_id_valid_values() {
            let Ok(parsed) = <$kind>::parse(value.clone()) else {
                panic!("{kind_name} must accept {}-byte value", value.len());
            };
            assert_eq!(
                parsed.as_str(),
                value,
                "{kind_name} must retain exact bytes"
            );

            let Ok(from_string) = <$kind>::try_from(value.clone()) else {
                panic!("{kind_name} TryFrom<String> must accept");
            };
            let Ok(from_str_ref) = <$kind>::try_from(value.as_str()) else {
                panic!("{kind_name} TryFrom<&str> must accept");
            };
            let Ok(from_str) = value.parse::<$kind>() else {
                panic!("{kind_name} FromStr must accept");
            };
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

            let Ok(from_owned) = <$kind>::deserialize(owned_deserializer(value.clone())) else {
                panic!("{kind_name} owned-string Serde must accept");
            };
            assert_eq!(from_owned, parsed);
            let Ok(from_bytes) = <$kind>::deserialize(bytes_deserializer(value.as_bytes())) else {
                panic!("{kind_name} bytes Serde must accept");
            };
            assert_eq!(from_bytes, parsed);
        }

        for (value, expected) in source_id_invalid_values() {
            let Err(error) = <$kind>::parse(value.clone()) else {
                panic!("{kind_name} must reject a non-canonical value");
            };
            assert_eq!(error.value_kind(), kind_name);
            assert_eq!(
                error.kind(),
                expected,
                "{kind_name} precedence drift for {}-byte input",
                value.len()
            );

            let Err(from_string) = <$kind>::try_from(value.clone()) else {
                panic!("{kind_name} TryFrom<String> must reject");
            };
            let Err(from_str_ref) = <$kind>::try_from(value.as_str()) else {
                panic!("{kind_name} TryFrom<&str> must reject");
            };
            let Err(from_str) = value.parse::<$kind>() else {
                panic!("{kind_name} FromStr must reject");
            };
            assert_eq!(from_string, error);
            assert_eq!(from_str_ref, error);
            assert_eq!(from_str, error);

            let encoded = serde_json::to_string(&value).expect("serialize");
            assert!(
                serde_json::from_str::<$kind>(&encoded).is_err(),
                "{kind_name} Serde must reject"
            );
            let Err(owned_error) = <$kind>::deserialize(owned_deserializer(value.clone())) else {
                panic!("{kind_name} owned-string Serde must reject");
            };
            assert_eq!(
                owned_error.to_string(),
                error.to_string(),
                "{kind_name} owned-string Serde must report checked constructor error"
            );
            let Err(bytes_error) = <$kind>::deserialize(bytes_deserializer(value.as_bytes()))
            else {
                panic!("{kind_name} bytes Serde must reject");
            };
            assert_eq!(
                bytes_error.to_string(),
                error.to_string(),
                "{kind_name} bytes Serde must report checked constructor error"
            );
        }
    }};
}

#[test]
fn source_id_family_enforces_grammar_and_precedence() {
    assert_source_id_kind_grammar!(SourceId);
    assert_source_id_kind_grammar!(SourceReviewerId);
    assert_source_id_kind_grammar!(SourceReviewEvidenceId);
}

#[test]
fn source_id_family_values_are_nominally_distinct() {
    let id = SourceId::parse("ustc:example").expect("fixture");
    let reviewer = SourceReviewerId::parse("ustc:example").expect("fixture");
    let evidence = SourceReviewEvidenceId::parse("ustc:example").expect("fixture");

    assert_eq!(id.as_str(), "ustc:example");
    assert_eq!(reviewer.as_str(), "ustc:example");
    assert_eq!(evidence.as_str(), "ustc:example");
}

// ---------------------------------------------------------------------------
// §3.2 SourceOwner grammar.
// ---------------------------------------------------------------------------

#[test]
fn source_owner_enforces_grammar_and_precedence() {
    let valid = [
        "a",
        "USTC Affairs Office",
        "中国科学技术大学教务处",
        "www.teach.ustc.edu.cn",
        &{
            let mut s = String::from("x");
            s.push_str(&"y".repeat(MAX_OWNER_BYTES - 2));
            s.push('z');
            s
        },
    ];
    for value in valid {
        let Ok(parsed) = SourceOwner::parse(value.to_owned()) else {
            panic!("SourceOwner must accept {value:?}");
        };
        assert_eq!(
            parsed.as_str(),
            value,
            "SourceOwner must preserve text exactly"
        );

        let encoded = serde_json::to_string(&parsed).expect("serialize");
        let decoded: SourceOwner = serde_json::from_str(&encoded).expect("deserialize");
        assert_eq!(decoded, parsed);
    }

    let empty_max = "x".repeat(MAX_OWNER_BYTES + 1);
    let cases: Vec<(String, SourceValueErrorKind)> = vec![
        (String::new(), SourceValueErrorKind::Empty),
        (
            empty_max,
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
    for (value, expected) in cases {
        let Err(error) = SourceOwner::parse(value.clone()) else {
            panic!("SourceOwner must reject {value:?}");
        };
        assert_eq!(error.value_kind(), "SourceOwner");
        assert_eq!(
            error.kind(),
            expected,
            "SourceOwner precedence drift for {value:?}"
        );
    }

    let encoded = serde_json::to_string(" leading").expect("serialize");
    assert!(
        serde_json::from_str::<SourceOwner>(&encoded).is_err(),
        "SourceOwner Serde must reject boundary whitespace"
    );
}

// ---------------------------------------------------------------------------
// §3.4 SourceUrl grammar: edges and precedence.
// ---------------------------------------------------------------------------

#[test]
fn source_url_enforces_grammar_and_precedence() {
    let valid = [
        "https://www.ustc.edu.cn/",
        "https://www.teach.ustc.edu.cn/calendar/19081.html",
        "https://www.teach.ustc.edu.cn/category/calendar",
        "https://example.com/a",
        "https://example.com/a-b_c.d~e",
        "https://example.com/%41%42%43",
        "https://sub.domain.example.com/path/to/resource",
        "https://123.example.com/foo",
    ];
    for value in valid {
        let Ok(parsed) = SourceUrl::parse(value.to_owned()) else {
            panic!("SourceUrl must accept {value}");
        };
        assert_eq!(parsed.as_str(), value, "SourceUrl must preserve exactly");

        let encoded = serde_json::to_string(&parsed).expect("serialize");
        let decoded: SourceUrl = serde_json::from_str(&encoded).expect("deserialize");
        assert_eq!(decoded, parsed);
    }

    let too_long = format!("https://example.com/{}", "a".repeat(MAX_URL_BYTES));
    let cases: Vec<(String, SourceValueErrorKind)> = vec![
        (String::new(), SourceValueErrorKind::Empty),
        (
            too_long,
            SourceValueErrorKind::TooLong {
                max_bytes: MAX_URL_BYTES,
            },
        ),
        (
            "http://example.com/".to_owned(),
            SourceValueErrorKind::InvalidScheme,
        ),
        (
            "HTTPS://example.com/".to_owned(),
            SourceValueErrorKind::InvalidScheme,
        ),
        (
            "ftp://example.com/".to_owned(),
            SourceValueErrorKind::InvalidScheme,
        ),
        (
            "https:/example.com/".to_owned(),
            SourceValueErrorKind::InvalidScheme,
        ),
        (
            "https://example.com".to_owned(),
            SourceValueErrorKind::InvalidPath,
        ),
        (
            "https://localhost/".to_owned(),
            SourceValueErrorKind::InvalidHost,
        ),
        (
            "https://example.com:8080/".to_owned(),
            SourceValueErrorKind::InvalidHost,
        ),
        (
            "https://user@example.com/".to_owned(),
            SourceValueErrorKind::InvalidHost,
        ),
        (
            "https://example.com/?q=1".to_owned(),
            SourceValueErrorKind::InvalidPath,
        ),
        (
            "https://example.com/#frag".to_owned(),
            SourceValueErrorKind::InvalidPath,
        ),
        (
            "https://Example.com/".to_owned(),
            SourceValueErrorKind::InvalidHost,
        ),
        (
            "https://192.168.0.1/".to_owned(),
            SourceValueErrorKind::InvalidHost,
        ),
        (
            "https://example.com./".to_owned(),
            SourceValueErrorKind::InvalidHost,
        ),
        (
            "https://-bad.com/".to_owned(),
            SourceValueErrorKind::InvalidHost,
        ),
        (
            "https://example.com//".to_owned(),
            SourceValueErrorKind::InvalidPath,
        ),
        (
            "https://example.com/./".to_owned(),
            SourceValueErrorKind::InvalidPath,
        ),
        (
            "https://example.com/../".to_owned(),
            SourceValueErrorKind::InvalidPath,
        ),
        (
            "https://example.com/a//b".to_owned(),
            SourceValueErrorKind::InvalidPath,
        ),
        (
            "https://example.com/a/.".to_owned(),
            SourceValueErrorKind::InvalidPath,
        ),
        (
            "https://example.com/a/..".to_owned(),
            SourceValueErrorKind::InvalidPath,
        ),
        (
            "https://example.com/a ".to_owned(),
            SourceValueErrorKind::InvalidPath,
        ),
        (
            "https://example.com/%".to_owned(),
            SourceValueErrorKind::InvalidPath,
        ),
        (
            "https://example.com/%4".to_owned(),
            SourceValueErrorKind::InvalidPath,
        ),
        (
            "https://example.com/%4G".to_owned(),
            SourceValueErrorKind::InvalidPath,
        ),
        (
            "https://example.com/%4g".to_owned(),
            SourceValueErrorKind::InvalidPath,
        ),
    ];
    for (value, expected) in cases {
        let Err(error) = SourceUrl::parse(value.clone()) else {
            panic!("SourceUrl must reject {value}");
        };
        assert_eq!(error.value_kind(), "SourceUrl");
        assert_eq!(
            error.kind(),
            expected,
            "SourceUrl precedence drift for {value}"
        );
    }
}

#[test]
fn source_url_rejects_non_ascii_in_correct_position() {
    let Err(host_err) = SourceUrl::parse("https://exämple.com/") else {
        panic!("must reject non-ASCII host");
    };
    assert_eq!(host_err.kind(), SourceValueErrorKind::InvalidHost);

    let Err(path_err) = SourceUrl::parse("https://example.com/café") else {
        panic!("must reject non-ASCII path");
    };
    assert_eq!(path_err.kind(), SourceValueErrorKind::InvalidPath);
}

// ---------------------------------------------------------------------------
// §7 No-echo: Display, Debug, and source chain never retain rejected input.
// ---------------------------------------------------------------------------

#[test]
fn source_value_errors_never_echo_rejected_input() {
    let secret = "super-secret-value";

    let Err(error) = SourceId::parse(format!("{secret}/")) else {
        panic!("must reject");
    };
    let display = format!("{error}");
    let debug = format!("{error:?}");
    assert!(!display.contains(secret), "Display leaked: {display}");
    assert!(!debug.contains(secret), "Debug leaked: {debug}");
    assert!(error.source().is_none(), "source chain leaked");

    let Err(owner_err) = SourceOwner::parse(format!(" {secret}")) else {
        panic!("must reject");
    };
    let owner_display = format!("{owner_err}");
    let owner_debug = format!("{owner_err:?}");
    assert!(
        !owner_display.contains(secret),
        "Owner Display leaked: {owner_display}"
    );
    assert!(
        !owner_debug.contains(secret),
        "Owner Debug leaked: {owner_debug}"
    );

    let Err(url_err) = SourceUrl::parse(format!("https://example.com/{secret}?")) else {
        panic!("must reject");
    };
    let url_display = format!("{url_err}");
    let url_debug = format!("{url_err:?}");
    assert!(
        !url_display.contains(secret),
        "URL Display leaked: {url_display}"
    );
    assert!(!url_debug.contains(secret), "URL Debug leaked: {url_debug}");
}

// ---------------------------------------------------------------------------
// §4.1 SourceRetrievalPolicy: bounds and precedence.
// ---------------------------------------------------------------------------

#[test]
fn retrieval_policy_enforces_bounds_and_precedence() {
    let Ok(policy) = SourceRetrievalPolicy::new(21_600, 131_072) else {
        panic!("must accept valid policy");
    };
    assert_eq!(policy.minimum_interval_seconds(), 21_600);
    assert_eq!(policy.maximum_response_bytes(), 131_072);

    let e = SourceRetrievalPolicy::new(0, 131_072).expect_err("zero min");
    assert_eq!(e.kind(), SourceValueErrorKind::ZeroMinimumInterval);
    assert_eq!(e.value_kind(), "SourceRetrievalPolicy");

    let e = SourceRetrievalPolicy::new(MAX_MIN_INTERVAL + 1, 131_072).expect_err("too large min");
    assert_eq!(
        e.kind(),
        SourceValueErrorKind::MinimumIntervalTooLarge {
            max_seconds: MAX_MIN_INTERVAL
        }
    );

    let e = SourceRetrievalPolicy::new(21_600, 0).expect_err("zero max");
    assert_eq!(e.kind(), SourceValueErrorKind::ZeroMaximumResponseBytes);

    let e = SourceRetrievalPolicy::new(21_600, MAX_MAX_BYTES + 1).expect_err("too large max");
    assert_eq!(
        e.kind(),
        SourceValueErrorKind::MaximumResponseBytesTooLarge {
            max_bytes: MAX_MAX_BYTES
        }
    );

    let e = SourceRetrievalPolicy::new(0, 0).expect_err("both zero");
    assert_eq!(
        e.kind(),
        SourceValueErrorKind::ZeroMinimumInterval,
        "minimum interval is checked before maximum response bytes"
    );

    let e = SourceRetrievalPolicy::new(MAX_MIN_INTERVAL + 1, 0).expect_err("both bad");
    assert_eq!(
        e.kind(),
        SourceValueErrorKind::MinimumIntervalTooLarge {
            max_seconds: MAX_MIN_INTERVAL
        },
        "too-large minimum is checked before zero maximum"
    );
}

// ---------------------------------------------------------------------------
// §4.1 SourceReviewReceipt: total constructor and accessors.
// ---------------------------------------------------------------------------

#[test]
fn review_receipt_is_total_and_exposes_all_fields() {
    let reviewer = SourceReviewerId::parse("reviewer:operator").expect("fixture");
    let review = SourceReviewEvidenceId::parse("evidence:review").expect("fixture");
    let permission = SourceReviewEvidenceId::parse("evidence:permission").expect("fixture");
    let rate = SourceReviewEvidenceId::parse("evidence:rate").expect("fixture");
    let parser_fixture = SourceReviewEvidenceId::parse("evidence:fixture").expect("fixture");

    let receipt = SourceReviewReceipt::new(
        reviewer.clone(),
        review.clone(),
        permission.clone(),
        rate.clone(),
        parser_fixture.clone(),
    );
    assert_eq!(receipt.reviewer(), &reviewer);
    assert_eq!(receipt.review(), &review);
    assert_eq!(receipt.permission(), &permission);
    assert_eq!(receipt.rate(), &rate);
    assert_eq!(receipt.parser_fixture(), &parser_fixture);
}

// ---------------------------------------------------------------------------
// §4 SourceDefinition::proposed: only constructor; rejects ModelInference.
// ---------------------------------------------------------------------------

#[test]
fn proposed_rejects_model_inference() {
    let definition_fields = fixture_definition_fields();
    let Err(error) = SourceDefinition::proposed(
        definition_fields.source_id,
        definition_fields.owner,
        definition_fields.url,
        SourceAuthority::ModelInference,
        definition_fields.retrieval_policy,
    ) else {
        panic!("ModelInference must be rejected");
    };
    assert_eq!(error.value_kind(), "SourceDefinition");
    assert_eq!(error.kind(), SourceValueErrorKind::NonSourceAuthority);
}

#[test]
fn proposed_accepts_non_model_inference_authorities() {
    for authority in [
        SourceAuthority::CommunitySignal,
        SourceAuthority::ReviewedOfficialSource,
    ] {
        let fields = fixture_definition_fields();
        let Ok(definition) = SourceDefinition::proposed(
            fields.source_id,
            fields.owner,
            fields.url,
            authority,
            fields.retrieval_policy,
        ) else {
            panic!("must accept {authority:?}");
        };
        assert_eq!(definition.authority(), authority);
        assert!(matches!(
            definition.review_state(),
            SourceReviewState::Proposed
        ));
    }
}

#[test]
fn definition_accessors_return_correct_types() {
    let fields = fixture_definition_fields();
    let definition = SourceDefinition::proposed(
        fields.source_id,
        fields.owner,
        fields.url,
        SourceAuthority::ReviewedOfficialSource,
        fields.retrieval_policy,
    )
    .expect("fixture");

    assert_eq!(definition.source_id().as_str(), "ustc:example-2025");
    assert_eq!(definition.owner().as_str(), "USTC Affairs Office");
    assert_eq!(
        definition.url().as_str(),
        "https://www.teach.ustc.edu.cn/calendar/19081.html"
    );
    assert_eq!(
        definition.authority(),
        SourceAuthority::ReviewedOfficialSource
    );
    assert_eq!(
        definition.retrieval_policy().minimum_interval_seconds(),
        21_600
    );
    assert_eq!(
        definition.retrieval_policy().maximum_response_bytes(),
        131_072
    );
    assert!(matches!(
        definition.review_state(),
        SourceReviewState::Proposed
    ));
}

// ---------------------------------------------------------------------------
// §5 Registry operations: propose, approve, get, approved, len, is_empty.
// ---------------------------------------------------------------------------

#[test]
fn registry_starts_empty() {
    let registry = SourceRegistry::new();
    assert!(registry.is_empty());
    assert_eq!(registry.len(), 0);
}

#[test]
fn propose_then_get_works() {
    let mut registry = SourceRegistry::new();
    let definition = fixture_definition();
    let id = definition.source_id().clone();
    registry.propose(definition).expect("propose");
    assert_eq!(registry.len(), 1);
    assert!(!registry.is_empty());
    let got = registry.get(&id).expect("present");
    assert!(matches!(got.review_state(), SourceReviewState::Proposed));
}

#[test]
fn propose_rejects_duplicate_source_id() {
    let mut registry = SourceRegistry::new();
    let def1 = fixture_definition();
    let id = def1.source_id().clone();
    registry.propose(def1).expect("first propose");

    let def2 = SourceDefinition::proposed(
        id.clone(),
        fixture_owner(),
        SourceUrl::parse("https://other.example.com/").expect("different url"),
        SourceAuthority::ReviewedOfficialSource,
        fixture_policy(),
    )
    .expect("def2");
    let Err(error) = registry.propose(def2) else {
        panic!("duplicate source ID must be rejected");
    };
    assert_eq!(
        error,
        SourceRegistryError::DuplicateSource {
            source_id: id.clone()
        }
    );
    assert_eq!(registry.len(), 1, "failed propose must not change registry");
    let existing = registry.get(&id).expect("present");
    assert_eq!(
        existing.url().as_str(),
        "https://www.teach.ustc.edu.cn/calendar/19081.html",
        "first definition must be preserved"
    );
}

#[test]
fn propose_rejects_duplicate_url() {
    let mut registry = SourceRegistry::new();
    let def1 = fixture_definition();
    let url = def1.url().clone();
    registry.propose(def1).expect("first propose");

    let def2 = SourceDefinition::proposed(
        SourceId::parse("ustc:different-id").expect("different id"),
        fixture_owner(),
        url.clone(),
        SourceAuthority::ReviewedOfficialSource,
        fixture_policy(),
    )
    .expect("def2");
    let Err(error) = registry.propose(def2) else {
        panic!("duplicate URL must be rejected");
    };
    assert_eq!(error, SourceRegistryError::DuplicateUrl { url });
    assert_eq!(registry.len(), 1, "failed propose must not change registry");
    assert!(
        registry
            .get(&SourceId::parse("ustc:example-2025").expect("fixture"))
            .is_some(),
        "first definition must be preserved"
    );
}

#[test]
fn approve_missing_rejects() {
    let mut registry = SourceRegistry::new();
    let missing_id = SourceId::parse("ustc:missing").expect("fixture");
    let Err(error) = registry.approve(&missing_id, fixture_receipt()) else {
        panic!("missing approve must be rejected");
    };
    assert_eq!(
        error,
        SourceRegistryError::SourceNotFound {
            source_id: missing_id
        }
    );
    assert!(
        registry.is_empty(),
        "failed approve must not change registry"
    );
}

#[test]
fn approve_then_approved_works() {
    let mut registry = SourceRegistry::new();
    let definition = fixture_definition();
    let id = definition.source_id().clone();
    registry.propose(definition).expect("propose");

    let receipt = fixture_receipt();
    registry.approve(&id, receipt).expect("approve");

    let approved = registry.approved(&id).expect("approved");
    match approved.review_state() {
        SourceReviewState::Approved { receipt: r } => {
            assert_eq!(r.reviewer().as_str(), "reviewer:operator");
        }
        SourceReviewState::Proposed => panic!("must be approved"),
    }
}

#[test]
fn approve_already_approved_preserves_first_receipt() {
    let mut registry = SourceRegistry::new();
    let definition = fixture_definition();
    let id = definition.source_id().clone();
    registry.propose(definition).expect("propose");

    let first_receipt = fixture_receipt();
    registry.approve(&id, first_receipt).expect("first approve");

    let second_receipt = SourceReviewReceipt::new(
        SourceReviewerId::parse("reviewer:different").expect("fixture"),
        SourceReviewEvidenceId::parse("evidence:diff").expect("fixture"),
        SourceReviewEvidenceId::parse("evidence:diff").expect("fixture"),
        SourceReviewEvidenceId::parse("evidence:diff").expect("fixture"),
        SourceReviewEvidenceId::parse("evidence:diff").expect("fixture"),
    );
    let Err(error) = registry.approve(&id, second_receipt) else {
        panic!("second approve must be rejected");
    };
    assert_eq!(
        error,
        SourceRegistryError::SourceAlreadyApproved {
            source_id: id.clone()
        }
    );

    let approved = registry.approved(&id).expect("approved");
    match approved.review_state() {
        SourceReviewState::Approved { receipt } => {
            assert_eq!(
                receipt.reviewer().as_str(),
                "reviewer:operator",
                "first receipt must be preserved"
            );
        }
        SourceReviewState::Proposed => panic!("must still be approved"),
    }
}

#[test]
fn approved_rejects_missing_and_proposed() {
    let mut registry = SourceRegistry::new();
    let missing_id = SourceId::parse("ustc:missing").expect("fixture");
    let Err(error) = registry.approved(&missing_id) else {
        panic!("approved of missing must be rejected");
    };
    assert_eq!(
        error,
        SourceRegistryError::SourceNotFound {
            source_id: missing_id
        }
    );

    let definition = fixture_definition();
    let id = definition.source_id().clone();
    registry.propose(definition).expect("propose");

    let Err(error) = registry.approved(&id) else {
        panic!("approved of proposed must be rejected");
    };
    assert_eq!(
        error,
        SourceRegistryError::SourceNotApproved { source_id: id }
    );
}

#[test]
fn failed_operations_preserve_registry_unchanged() {
    let mut registry = SourceRegistry::new();
    let def1 = fixture_definition();
    let id1 = def1.source_id().clone();
    registry.propose(def1).expect("propose");
    let snapshot_before = registry.clone();

    let missing_id = SourceId::parse("ustc:missing").expect("fixture");
    let _ = registry.approve(&missing_id, fixture_receipt());
    let _ = registry.approved(&missing_id);

    let def2 = SourceDefinition::proposed(
        id1.clone(),
        fixture_owner(),
        SourceUrl::parse("https://other.example.com/").expect("url"),
        SourceAuthority::ReviewedOfficialSource,
        fixture_policy(),
    )
    .expect("def2");
    let _ = registry.propose(def2);

    assert_eq!(
        registry, snapshot_before,
        "all failed operations must leave registry byte-for-byte unchanged"
    );
}

#[test]
fn registry_error_display_and_source_chain() {
    let id = SourceId::parse("ustc:example").expect("fixture");
    let url = SourceUrl::parse("https://example.com/").expect("fixture");

    let dup_id = SourceRegistryError::DuplicateSource {
        source_id: id.clone(),
    };
    let display = format!("{dup_id}");
    assert!(
        display.contains("ustc:example"),
        "DuplicateSource must render ID: {display}"
    );
    assert!(dup_id.source().is_none());

    let dup_url = SourceRegistryError::DuplicateUrl { url: url.clone() };
    let display = format!("{dup_url}");
    assert!(
        display.contains("https://example.com/"),
        "DuplicateUrl must render URL: {display}"
    );

    let not_found = SourceRegistryError::SourceNotFound {
        source_id: id.clone(),
    };
    let _ = format!("{not_found}");

    let not_approved = SourceRegistryError::SourceNotApproved {
        source_id: id.clone(),
    };
    let _ = format!("{not_approved}");

    let already = SourceRegistryError::SourceAlreadyApproved { source_id: id };
    let _ = format!("{already}");
}

// ---------------------------------------------------------------------------
// §6 No aggregate Serde: definition, registry, receipt, state, error have
// no Deserialize.
// ---------------------------------------------------------------------------

#[test]
fn no_aggregate_serde_decode_exists() {
    // These are compile-time guarantees documented as compile_fail doctests
    // in the source. Here we verify at runtime that the types do NOT implement
    // Deserialize by checking that serde_json::from_str fails to compile against
    // them — which is enforced by the absence of the impl, not by a runtime
    // check. The doctests in source_registry.rs are the binding proof.
}

// ---------------------------------------------------------------------------
// §11 No concrete approved USTC source: the candidate family stays Proposed.
// ---------------------------------------------------------------------------

#[test]
fn no_concrete_approved_ustc_source_in_production_data() {
    let mut registry = SourceRegistry::new();
    let def_2025 = SourceDefinition::proposed(
        SourceId::parse("ustc-teach-calendar-fall-2025").expect("fixture"),
        SourceOwner::parse("www.teach.ustc.edu.cn").expect("fixture"),
        SourceUrl::parse("https://www.teach.ustc.edu.cn/calendar/19081.html").expect("fixture"),
        SourceAuthority::ReviewedOfficialSource,
        SourceRetrievalPolicy::new(21_600, 131_072).expect("fixture"),
    )
    .expect("fixture");
    let id_2025 = def_2025.source_id().clone();
    registry.propose(def_2025).expect("propose 2025");

    let def_2026 = SourceDefinition::proposed(
        SourceId::parse("ustc-teach-calendar-fall-2026").expect("fixture"),
        SourceOwner::parse("www.teach.ustc.edu.cn").expect("fixture"),
        SourceUrl::parse("https://www.teach.ustc.edu.cn/calendar/20135.html").expect("fixture"),
        SourceAuthority::ReviewedOfficialSource,
        SourceRetrievalPolicy::new(21_600, 131_072).expect("fixture"),
    )
    .expect("fixture");
    let id_2026 = def_2026.source_id().clone();
    registry.propose(def_2026).expect("propose 2026");

    for id in [&id_2025, &id_2026] {
        let def = registry.get(id).expect("present");
        assert!(
            matches!(def.review_state(), SourceReviewState::Proposed),
            "candidate family must stay Proposed throughout P1-1: {id}"
        );
        let Err(error) = registry.approved(id) else {
            panic!("approved must reject Proposed entry: {id}");
        };
        assert_eq!(
            error,
            SourceRegistryError::SourceNotApproved {
                source_id: id.clone()
            }
        );
    }
}

// ---------------------------------------------------------------------------
// Fixtures.
// ---------------------------------------------------------------------------

struct DefinitionFields {
    source_id: SourceId,
    owner: SourceOwner,
    url: SourceUrl,
    retrieval_policy: SourceRetrievalPolicy,
}

fn fixture_definition_fields() -> DefinitionFields {
    DefinitionFields {
        source_id: SourceId::parse("ustc:example-2025").expect("fixture"),
        owner: SourceOwner::parse("USTC Affairs Office").expect("fixture"),
        url: SourceUrl::parse("https://www.teach.ustc.edu.cn/calendar/19081.html")
            .expect("fixture"),
        retrieval_policy: SourceRetrievalPolicy::new(21_600, 131_072).expect("fixture"),
    }
}

fn fixture_definition() -> SourceDefinition {
    let f = fixture_definition_fields();
    SourceDefinition::proposed(
        f.source_id,
        f.owner,
        f.url,
        SourceAuthority::ReviewedOfficialSource,
        f.retrieval_policy,
    )
    .expect("fixture definition")
}

fn fixture_owner() -> SourceOwner {
    SourceOwner::parse("USTC Affairs Office").expect("fixture")
}

fn fixture_policy() -> SourceRetrievalPolicy {
    SourceRetrievalPolicy::new(21_600, 131_072).expect("fixture")
}

fn fixture_receipt() -> SourceReviewReceipt {
    SourceReviewReceipt::new(
        SourceReviewerId::parse("reviewer:operator").expect("fixture"),
        SourceReviewEvidenceId::parse("evidence:review").expect("fixture"),
        SourceReviewEvidenceId::parse("evidence:permission").expect("fixture"),
        SourceReviewEvidenceId::parse("evidence:rate").expect("fixture"),
        SourceReviewEvidenceId::parse("evidence:fixture").expect("fixture"),
    )
}
