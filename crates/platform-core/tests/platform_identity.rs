//! `platform-identity/v0` acceptance evidence for `M00-B1 identity-types`.
//!
//! Bound rows: `AUTH-011`, `AUTH-012`, `AUTH-014`, `AUTH-015`, `AUTH-016`.

use std::any::TypeId;
use std::collections::hash_map::DefaultHasher;
use std::error::Error;
use std::hash::{Hash, Hasher};

use serde::Deserialize;
use serde::de::IntoDeserializer;
use serde::de::value::{BytesDeserializer, Error as SerdeValueError, StringDeserializer};
use ustc_campus_agent_core::identity::{
    CommandId, CorrelationId, IdentityValueError, IdentityValueErrorKind, RequestId, SessionId,
    TenantId, UserId,
};
use ustc_campus_agent_core::invocation;

const MAX_BYTES: usize = 128;

/// Values that every ID kind must accept verbatim.
fn valid_values() -> Vec<String> {
    let mut exact_max = String::from("a");
    exact_max.push_str(&"b".repeat(MAX_BYTES - 2));
    exact_max.push('c');
    assert_eq!(exact_max.len(), MAX_BYTES);

    vec![
        // Single byte, both alphanumeric classes.
        "a".to_owned(),
        "7".to_owned(),
        // Two bytes: the interior range is empty, so both boundary rules must accept.
        "aa".to_owned(),
        "0Z".to_owned(),
        // Mixed case is significant and preserved.
        "AbC".to_owned(),
        "abc".to_owned(),
        // All four interior delimiters.
        "Tenant.Alpha_Beta:Gamma-01".to_owned(),
        // Repeated interior delimiters are legal and carry no meaning.
        "a..__::--b".to_owned(),
        // Opaque generator shapes named by the contract.
        "01ARZ3NDEKTSV4RRFFQ69G5FAV".to_owned(),
        "0192f3c4d5e677889900aabbccddeeff".to_owned(),
        // Exactly at the bound.
        exact_max,
    ]
}

/// Rejected values paired with the exact rule that must fire, proving precedence.
fn invalid_values() -> Vec<(String, IdentityValueErrorKind)> {
    let too_long = "a".repeat(MAX_BYTES + 1);
    // One byte over the bound, measured in UTF-8 bytes rather than characters. Both lengths are
    // DERIVED from the bound: hand-written counts here were co-mutated with a wrong implementation
    // and kept agreeing with it, so the only thing asserted is the byte length and the fact that
    // the character count is strictly smaller.
    let multibyte_tail = "é".repeat(MAX_BYTES / 2);
    let too_long_multibyte = format!(
        "{}{}",
        "a".repeat(MAX_BYTES + 1 - multibyte_tail.len()),
        multibyte_tail
    );
    assert_eq!(too_long_multibyte.len(), MAX_BYTES + 1);
    assert!(too_long_multibyte.chars().count() < too_long_multibyte.len());
    // Length outranks an otherwise-fatal first byte, interior byte and final byte.
    let too_long_bad_start = format!("-{}", "a".repeat(MAX_BYTES));
    let too_long_bad_interior = format!("a {}", "b".repeat(MAX_BYTES - 1));
    let too_long_bad_end = format!("{}-", "a".repeat(MAX_BYTES));

    vec![
        // 1. empty outranks everything.
        (String::new(), IdentityValueErrorKind::Empty),
        // 2. byte length outranks first-byte, interior and final rules.
        (
            too_long,
            IdentityValueErrorKind::TooLong {
                max_bytes: MAX_BYTES,
            },
        ),
        (
            too_long_multibyte,
            IdentityValueErrorKind::TooLong {
                max_bytes: MAX_BYTES,
            },
        ),
        (
            too_long_bad_start,
            IdentityValueErrorKind::TooLong {
                max_bytes: MAX_BYTES,
            },
        ),
        (
            too_long_bad_interior,
            IdentityValueErrorKind::TooLong {
                max_bytes: MAX_BYTES,
            },
        ),
        (
            too_long_bad_end,
            IdentityValueErrorKind::TooLong {
                max_bytes: MAX_BYTES,
            },
        ),
        // 3. invalid first byte, including every delimiter and whitespace.
        ("-abc".to_owned(), IdentityValueErrorKind::InvalidStart),
        (".abc".to_owned(), IdentityValueErrorKind::InvalidStart),
        ("_abc".to_owned(), IdentityValueErrorKind::InvalidStart),
        (":abc".to_owned(), IdentityValueErrorKind::InvalidStart),
        (" abc".to_owned(), IdentityValueErrorKind::InvalidStart),
        ("!abc".to_owned(), IdentityValueErrorKind::InvalidStart),
        // A one-byte value is decided entirely by the first-byte rule.
        ("-".to_owned(), IdentityValueErrorKind::InvalidStart),
        (".".to_owned(), IdentityValueErrorKind::InvalidStart),
        (" ".to_owned(), IdentityValueErrorKind::InvalidStart),
        ("é".to_owned(), IdentityValueErrorKind::InvalidStart),
        // 4. first invalid interior byte, scanned left to right.
        (
            "a b".to_owned(),
            IdentityValueErrorKind::InvalidCharacter { byte_index: 1 },
        ),
        (
            "a\tb".to_owned(),
            IdentityValueErrorKind::InvalidCharacter { byte_index: 1 },
        ),
        (
            "a\nb".to_owned(),
            IdentityValueErrorKind::InvalidCharacter { byte_index: 1 },
        ),
        (
            "a\u{0}b".to_owned(),
            IdentityValueErrorKind::InvalidCharacter { byte_index: 1 },
        ),
        (
            "a\u{7f}b".to_owned(),
            IdentityValueErrorKind::InvalidCharacter { byte_index: 1 },
        ),
        (
            "a/b".to_owned(),
            IdentityValueErrorKind::InvalidCharacter { byte_index: 1 },
        ),
        (
            "a+b".to_owned(),
            IdentityValueErrorKind::InvalidCharacter { byte_index: 1 },
        ),
        (
            "a@b".to_owned(),
            IdentityValueErrorKind::InvalidCharacter { byte_index: 1 },
        ),
        (
            "a,b".to_owned(),
            IdentityValueErrorKind::InvalidCharacter { byte_index: 1 },
        ),
        (
            "a%b".to_owned(),
            IdentityValueErrorKind::InvalidCharacter { byte_index: 1 },
        ),
        (
            "a\\b".to_owned(),
            IdentityValueErrorKind::InvalidCharacter { byte_index: 1 },
        ),
        // The index is a byte offset: a long legal ASCII prefix shifts it exactly.
        (
            format!("{} b", "a".repeat(50)),
            IdentityValueErrorKind::InvalidCharacter { byte_index: 50 },
        ),
        // A multibyte offender reports its FIRST byte, not its last byte.
        (
            "ab€cd".to_owned(),
            IdentityValueErrorKind::InvalidCharacter { byte_index: 2 },
        ),
        (
            "ab\u{1d11e}cd".to_owned(),
            IdentityValueErrorKind::InvalidCharacter { byte_index: 2 },
        ),
        // Interior violations outrank an also-invalid final byte.
        (
            "a b-".to_owned(),
            IdentityValueErrorKind::InvalidCharacter { byte_index: 1 },
        ),
        (
            "a%b!".to_owned(),
            IdentityValueErrorKind::InvalidCharacter { byte_index: 1 },
        ),
        // A multibyte suffix is reached through the interior range first.
        (
            "aé".to_owned(),
            IdentityValueErrorKind::InvalidCharacter { byte_index: 1 },
        ),
        // 5. invalid final byte, including legal interior delimiters at the end.
        ("abc-".to_owned(), IdentityValueErrorKind::InvalidEnd),
        ("abc.".to_owned(), IdentityValueErrorKind::InvalidEnd),
        ("abc_".to_owned(), IdentityValueErrorKind::InvalidEnd),
        ("abc:".to_owned(), IdentityValueErrorKind::InvalidEnd),
        ("abc ".to_owned(), IdentityValueErrorKind::InvalidEnd),
        ("abc!".to_owned(), IdentityValueErrorKind::InvalidEnd),
        // Two-byte boundary: the interior range bytes[1..1] is empty, so the final rule fires.
        ("a-".to_owned(), IdentityValueErrorKind::InvalidEnd),
        ("a.".to_owned(), IdentityValueErrorKind::InvalidEnd),
    ]
}

/// A deserializer that OWNS its string, so it drives `visit_string` rather than `visit_str`.
///
/// `serde_json::from_str` only ever reaches the borrowed arm, so without this the owned entry
/// point of every kind is unproven and could construct a value without validating it.
fn owned_deserializer(value: String) -> StringDeserializer<SerdeValueError> {
    value.into_deserializer()
}

/// A deserializer that supplies BYTES, so it drives an entry point no string deserializer does.
///
/// Runtime evidence is a secondary control here, not the closure: the structural rule is that
/// the value has exactly one construction site. This exercises a third, independent entry point
/// so that the structural claim is also observed behaving correctly at run time.
fn bytes_deserializer(value: &[u8]) -> BytesDeserializer<'_, SerdeValueError> {
    BytesDeserializer::new(value)
}

/// Runs every stable construction class of one ID kind against the shared corpus.
///
/// Deliberately expanded per kind rather than run once against a representative type: the six
/// kinds are nominally distinct and each must be proven through each path.
macro_rules! assert_kind_enforces_grammar {
    ($kind:ty) => {{
        let kind_name = stringify!($kind);

        for value in valid_values() {
            let Ok(parsed) = <$kind>::parse(value.clone()) else {
                panic!(
                    "{kind_name} must accept {}-byte canonical value",
                    value.len()
                );
            };
            assert_eq!(
                parsed.as_str(),
                value,
                "{kind_name} must retain exact bytes"
            );

            let Ok(from_string) = <$kind>::try_from(value.clone()) else {
                panic!("{kind_name} TryFrom<String> must accept a canonical value");
            };
            let Ok(from_str_ref) = <$kind>::try_from(value.as_str()) else {
                panic!("{kind_name} TryFrom<&str> must accept a canonical value");
            };
            let Ok(from_str) = value.parse::<$kind>() else {
                panic!("{kind_name} FromStr must accept a canonical value");
            };
            assert_eq!(from_string, parsed);
            assert_eq!(from_str_ref, parsed);
            assert_eq!(from_str, parsed);

            let encoded = serde_json::to_string(&parsed).expect("serialize");
            assert_eq!(
                encoded,
                serde_json::to_string(&value).expect("serialize"),
                "{kind_name} must serialize as exactly one JSON string"
            );
            let decoded: $kind = serde_json::from_str(&encoded).expect("deserialize");
            assert_eq!(decoded, parsed, "{kind_name} Serde must round-trip exactly");

            // `from_str` drives the BORROWED visitor arm only. A deserializer that owns its
            // string calls `visit_string` instead, which is a second entry into the same type
            // and must be proven separately.
            let Ok(from_owned) = <$kind>::deserialize(owned_deserializer(value.clone())) else {
                panic!("{kind_name} owned-string Serde must accept a canonical value");
            };
            assert_eq!(
                from_owned, parsed,
                "{kind_name} owned-string Serde must produce the same value"
            );
            let Ok(from_bytes) = <$kind>::deserialize(bytes_deserializer(value.as_bytes())) else {
                panic!("{kind_name} bytes Serde must accept a canonical value");
            };
            assert_eq!(
                from_bytes, parsed,
                "{kind_name} bytes Serde must produce the same value"
            );
        }

        for (value, expected) in invalid_values() {
            let Err(error) = <$kind>::parse(value.clone()) else {
                panic!("{kind_name} must reject a non-canonical value");
            };
            assert_eq!(error.value_kind(), kind_name);
            assert_eq!(
                error.kind(),
                expected,
                "{kind_name} precedence drift for a {}-byte input",
                value.len()
            );

            let Err(from_string) = <$kind>::try_from(value.clone()) else {
                panic!("{kind_name} TryFrom<String> must reject a non-canonical value");
            };
            let Err(from_str_ref) = <$kind>::try_from(value.as_str()) else {
                panic!("{kind_name} TryFrom<&str> must reject a non-canonical value");
            };
            let Err(from_str) = value.parse::<$kind>() else {
                panic!("{kind_name} FromStr must reject a non-canonical value");
            };
            assert_eq!(from_string, error);
            assert_eq!(from_str_ref, error);
            assert_eq!(from_str, error);

            // Serde cannot bypass the constructor with an unchecked field decode.
            let encoded = serde_json::to_string(&value).expect("serialize");
            assert!(
                serde_json::from_str::<$kind>(&encoded).is_err(),
                "{kind_name} Serde must reject a non-canonical string"
            );
            // …and neither can the owned-string arm, which `from_str` never reaches.
            let Err(owned_error) = <$kind>::deserialize(owned_deserializer(value.clone())) else {
                panic!("{kind_name} owned-string Serde must reject a non-canonical value");
            };
            // Rendered identically to the checked constructor's own error, so the owned arm
            // inherits `parse`'s deterministic error class AND its non-echo guarantee by
            // identity rather than by a second, weaker assertion about its text.
            assert_eq!(
                owned_error.to_string(),
                error.to_string(),
                "{kind_name} owned-string Serde must report the checked constructor's error"
            );
            // The bytes entry point is the one Reproduction B used; it validates too.
            let Err(bytes_error) = <$kind>::deserialize(bytes_deserializer(value.as_bytes()))
            else {
                panic!("{kind_name} bytes Serde must reject a non-canonical value");
            };
            assert_eq!(
                bytes_error.to_string(),
                error.to_string(),
                "{kind_name} bytes Serde must report the checked constructor's error"
            );
        }
    }};
}

/// `AUTH-011`
#[test]
fn identity_values_enforce_canonical_bounds_and_errors() {
    assert_assertion_macros_bite();
    // Asserted from a second bound test as well, so ignoring AUTH-012 alone cannot silence it.
    assert_bound_test_envelope_is_active();
    // FIRST, deliberately. The behavioural oracles below judge the implementation against
    // corpora and tables that are themselves mutable; if the grammar has been moved off the
    // accepted contract, that is the finding, and it should be the one reported rather than
    // whichever hand-written corpus value happens to fail first.
    assert_grammar_semantics_match_the_contract();
    // Same reason, for the one field whose EFFECTIVE use a declared carrier cannot pin.
    assert_effective_max_byte_bound_is_contract_bound();
    assert_kind_enforces_grammar!(TenantId);
    assert_kind_enforces_grammar!(UserId);
    assert_kind_enforces_grammar!(SessionId);
    assert_kind_enforces_grammar!(RequestId);
    assert_kind_enforces_grammar!(CommandId);
    assert_kind_enforces_grammar!(CorrelationId);

    // The fixed bound is reported, never a caller-derived length.
    let Err(error) = TenantId::parse("a".repeat(MAX_BYTES + 40)) else {
        panic!("over-long value must be rejected");
    };
    assert_eq!(
        error.kind(),
        IdentityValueErrorKind::TooLong {
            max_bytes: MAX_BYTES
        }
    );

    assert_grammar_is_exhaustive_over_bytes();
}

/// Drives EVERY one of the 256 byte values through each grammar position.
///
/// The structural rules elsewhere in this suite compare function bodies after comments and
/// literal payloads are stripped, so they pin control flow and token shape but not the bytes
/// inside a literal: swapping `b':'` for `b'?'` inside `is_interior_byte` leaves the frozen body
/// unchanged. A hand-picked corpus does not close that either — it proves only that the values
/// someone thought of behave, which is the same finite-corpus weakness the contract rejects for
/// Serde entry points.
///
/// This closes it by exhausting the alphabet the grammar is defined on. Each byte is judged by
/// an oracle written independently of `classify` — a direct transcription of
/// `^[A-Za-z0-9](?:[-A-Za-z0-9._:]{0,126}[A-Za-z0-9])?$` as membership tests rather than as a
/// single left-to-right pass — so the two must agree for every byte in every position.
fn assert_grammar_is_exhaustive_over_bytes() {
    // The frozen delimiter set, spelled here and nowhere else in this function.
    let admitted_interior: [u8; 4] = *b"-._:";
    let boundary = |byte: u8| byte.is_ascii_alphanumeric();
    let interior = |byte: u8| byte.is_ascii_alphanumeric() || admitted_interior.contains(&byte);

    for byte in 0_u8..=u8::MAX {
        // Only ASCII bytes stand alone in well-formed UTF-8; the multibyte path is proven by the
        // dedicated non-ASCII cases in `invalid_values`.
        if !byte.is_ascii() {
            continue;
        }
        let character = char::from(byte);

        // Position 1: a one-byte value is decided entirely by the first-byte rule.
        let single = String::from(character);
        assert_eq!(
            TenantId::parse(single.clone()).is_ok(),
            boundary(byte),
            "one-byte grammar diverged on byte {byte}"
        );
        if !boundary(byte) {
            let Err(error) = TenantId::parse(single) else {
                panic!("byte {byte} must be rejected as a one-byte value");
            };
            assert_eq!(
                error.kind(),
                IdentityValueErrorKind::InvalidStart,
                "one-byte rejection class diverged on byte {byte}"
            );
        }

        // Position 2: the interior range, which admits the four delimiters as well.
        let inner = format!("a{character}b");
        assert_eq!(
            TenantId::parse(inner.clone()).is_ok(),
            interior(byte),
            "interior grammar diverged on byte {byte}"
        );
        if !interior(byte) {
            let Err(error) = TenantId::parse(inner) else {
                panic!("byte {byte} must be rejected in the interior");
            };
            assert_eq!(
                error.kind(),
                IdentityValueErrorKind::InvalidCharacter { byte_index: 1 },
                "interior rejection class diverged on byte {byte}"
            );
        }

        // Position 3: the final byte, where a legal interior delimiter must still be rejected.
        let trailing = format!("a{character}");
        assert_eq!(
            TenantId::parse(trailing.clone()).is_ok(),
            boundary(byte),
            "final-byte grammar diverged on byte {byte}"
        );
        if !boundary(byte) {
            let Err(error) = TenantId::parse(trailing) else {
                panic!("byte {byte} must be rejected as a final byte");
            };
            assert_eq!(
                error.kind(),
                IdentityValueErrorKind::InvalidEnd,
                "final-byte rejection class diverged on byte {byte}"
            );
        }

        // Position 4: the leading byte of a longer value, where the first-byte rule still
        // outranks an otherwise legal remainder.
        let leading = format!("{character}ab");
        assert_eq!(
            TenantId::parse(leading).is_ok(),
            boundary(byte),
            "leading-byte grammar diverged on byte {byte}"
        );
    }
}

fn hash_of<T: Hash>(value: &T) -> u64 {
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
}

/// The complete admitted public surface of the identity module, sorted.
///
/// This is an allowlist, not a blacklist of bad spellings: any added public function, trait
/// implementation, derive, alias or re-export changes the computed surface and fails. One
/// forbidden spelling such as `new` being absent is not evidence that every unchecked
/// constructor is absent, so the surface is frozen wholesale instead.
const ADMITTED_PUBLIC_SURFACE: [&str; 21] = [
    "derive Debug, Clone, Copy, PartialEq, Eq",
    "derive Debug, Clone, Copy, PartialEq, Eq",
    "derive Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash",
    "impl $name",
    "impl Deserialize<'de> for $name",
    "impl Error for IdentityValueError",
    "impl FromStr for $name",
    "impl IdentityValueError",
    "impl Serialize for $name",
    "impl TryFrom<&str> for $name",
    "impl TryFrom<String> for $name",
    "impl fmt::Display for $name",
    "impl fmt::Display for IdentityValueError",
    // The single admitted `impl Trait` argument position, on the checked constructor.
    "impl-arg Into<String>",
    "pub const fn kind",
    "pub const fn value_kind",
    "pub enum IdentityValueErrorKind",
    "pub fn as_str",
    "pub fn parse",
    "pub struct $name",
    "pub struct IdentityValueError",
];

const RUST_ITEM_KEYWORDS: [&str; 10] = [
    "fn", "struct", "enum", "union", "trait", "type", "mod", "use", "static", "const",
];

fn is_ident_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_'
}

/// Returns the leading identifier of `text`, allowing a `$` macro metavariable prefix.
fn leading_ident(text: &str) -> &str {
    let bytes = text.as_bytes();
    let mut end = 0;
    if end < bytes.len() && bytes[end] == b'$' {
        end += 1;
    }
    while end < bytes.len() && is_ident_byte(bytes[end]) {
        end += 1;
    }
    &text[..end]
}

/// Byte offsets of every `word`-boundary occurrence of `needle` in `line`.
fn token_positions(line: &str, needle: &str) -> Vec<usize> {
    let bytes = line.as_bytes();
    let mut positions = Vec::new();
    let mut cursor = 0;
    while let Some(found) = line[cursor..].find(needle) {
        let at = cursor + found;
        let after = at + needle.len();
        let before_ok = at == 0 || !is_ident_byte(bytes[at - 1]);
        let after_ok = after >= bytes.len() || !is_ident_byte(bytes[after]);
        if before_ok && after_ok {
            positions.push(at);
        }
        cursor = after;
    }
    positions
}

/// Classifies one `pub` declaration, keeping its qualifiers.
///
/// Returns `None` when the declaration is not recognised at all, which the caller must treat
/// as a failure: an allowlist is only complete if every `pub` token is accounted for.
fn classify_pub(rest: &str) -> Option<String> {
    let mut cursor = rest.trim_start();
    if cursor.starts_with('(') {
        // Restricted visibility such as `pub(crate)` is not admitted in this module.
        return None;
    }
    let mut qualifiers: Vec<String> = Vec::new();
    loop {
        let word = leading_ident(cursor);
        let after = cursor[word.len()..].trim_start();
        match word {
            "const" | "async" | "unsafe" => {
                // `const` is a qualifier only when a function follows it.
                let next = leading_ident(after);
                if word == "const" && !matches!(next, "fn" | "async" | "unsafe" | "extern") {
                    break;
                }
                qualifiers.push(word.to_owned());
                cursor = after;
            }
            "extern" => {
                if let Some(tail) = after.strip_prefix('"') {
                    let (abi, remainder) = tail.split_once('"')?;
                    qualifiers.push(format!("extern \"{abi}\""));
                    cursor = remainder.trim_start();
                } else {
                    qualifiers.push("extern".to_owned());
                    cursor = after;
                }
            }
            _ => break,
        }
    }
    let keyword = leading_ident(cursor);
    if !RUST_ITEM_KEYWORDS.contains(&keyword) {
        return None;
    }
    let name = leading_ident(cursor[keyword.len()..].trim_start());
    let mut entry = String::from("pub");
    for qualifier in &qualifiers {
        entry.push(' ');
        entry.push_str(qualifier);
    }
    entry.push(' ');
    entry.push_str(keyword);
    if !name.is_empty() {
        entry.push(' ');
        entry.push_str(name);
    }
    Some(entry)
}

/// Returns the leading type path of `text`, including balanced generic arguments.
fn leading_type_path(text: &str) -> Option<String> {
    let trimmed = text.trim_start();
    let path = leading_ident(trimmed);
    if path.is_empty() {
        return None;
    }
    let mut end = path.len();
    // Path segments such as `crate::identity::TenantId`.
    while trimmed[end..].starts_with("::") {
        let segment = leading_ident(&trimmed[end + 2..]);
        if segment.is_empty() {
            break;
        }
        end += 2 + segment.len();
    }
    let mut rendered = trimmed[..end].to_owned();
    let rest = &trimmed[end..];
    if rest.starts_with('<') {
        let mut depth = 0;
        for (index, character) in rest.char_indices() {
            match character {
                '<' => depth += 1,
                '>' => {
                    depth -= 1;
                    if depth == 0 {
                        rendered.push_str(
                            &rest[..index + 1]
                                .split_whitespace()
                                .collect::<Vec<_>>()
                                .join(" "),
                        );
                        break;
                    }
                }
                _ => {}
            }
        }
    }
    Some(rendered)
}

/// Returns the implemented self type of every `impl` token, whatever its line position.
fn impl_self_types(code: &str) -> Vec<String> {
    let mut targets = Vec::new();
    for (index, _) in code.match_indices("impl") {
        let bytes = code.as_bytes();
        let before_ok = index == 0 || !is_ident_byte(bytes[index - 1]);
        let after = index + 4;
        let after_ok = after >= bytes.len() || !is_ident_byte(bytes[after]);
        if !(before_ok && after_ok) {
            continue;
        }
        // Scan to the brace that opens the block, ignoring generic and parameter nesting.
        let (mut angle, mut paren) = (0i32, 0i32);
        let mut end = None;
        for (offset, character) in code[after..].char_indices() {
            match character {
                '<' => angle += 1,
                '>' => angle -= 1,
                '(' => paren += 1,
                ')' => paren -= 1,
                ';' => break,
                '{' if angle <= 0 && paren <= 0 => {
                    end = Some(after + offset);
                    break;
                }
                _ => {}
            }
        }
        if let Some(end) = end {
            targets.push(impl_self_type(&code[after..end]));
        }
    }
    targets
}

/// Returns the implemented self type of an impl header, ignoring generics and `where`.
fn impl_self_type(header: &str) -> String {
    let mut normalized = header.split_whitespace().collect::<Vec<_>>().join(" ");
    if normalized.starts_with('<') {
        let mut depth = 0;
        let mut cut = None;
        for (index, character) in normalized.char_indices() {
            match character {
                '<' => depth += 1,
                '>' => {
                    depth -= 1;
                    if depth == 0 {
                        cut = Some(index);
                        break;
                    }
                }
                _ => {}
            }
        }
        if let Some(index) = cut {
            normalized = normalized[index + 1..].trim().to_owned();
        }
    }
    // A `where` clause follows the self type and must not be folded into it.
    if let Some(at) = normalized.find(" where ") {
        normalized = normalized[..at].to_owned();
    }
    if normalized.ends_with(" where") {
        normalized = normalized[..normalized.len() - 6].to_owned();
    }
    let target = normalized
        .rsplit(" for ")
        .next()
        .unwrap_or(&normalized)
        .trim()
        .trim_end_matches(',')
        .trim();
    target.to_owned()
}

fn impl_header(joined: &str) -> String {
    let after = joined.strip_prefix("impl").unwrap_or(joined).trim_start();
    // Drop the generic parameter list introduced immediately after `impl`.
    let after = if after.starts_with('<') {
        let mut depth = 0;
        let mut cut = None;
        for (index, character) in after.char_indices() {
            match character {
                '<' => depth += 1,
                '>' => {
                    depth -= 1;
                    if depth == 0 {
                        cut = Some(index);
                        break;
                    }
                }
                _ => {}
            }
        }
        match cut {
            Some(index) => after[index + 1..].trim_start(),
            None => after,
        }
    } else {
        after
    };
    let head = after.split('{').next().unwrap_or_default();
    head.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Computes the sorted public-surface fingerprint of already-stripped Rust code.
///
/// Every `pub` and `impl` token is accounted for. Anything the grammar below does not
/// recognise is emitted as an `UNCLASSIFIED-*` entry so it can never silently vanish from
/// the fingerprint and pass the comparison.
fn public_surface(code: &str) -> Vec<String> {
    let mut surface = Vec::new();
    // Derives are extracted over the whole source, whitespace-tolerant between `#`, `[`,
    // `derive` and `(`, because a derive synthesizes a trait impl that no `use`/`type`/`impl`
    // scan ever sees, and `# [derive(Copy)]` derives exactly as `#[derive(Copy)]` does.
    for body in derive_bodies(code) {
        surface.push(format!("derive {body}"));
    }
    let lines: Vec<&str> = code.lines().collect();
    let mut index = 0;
    while index < lines.len() {
        let line = lines[index].trim();

        for at in token_positions(lines[index], "pub") {
            match classify_pub(&lines[index][at + 3..]) {
                Some(entry) => surface.push(entry),
                None => surface.push(format!("UNCLASSIFIED-PUB {line}")),
            }
        }

        if line.starts_with("impl") && token_positions(line, "impl").first() == Some(&0) {
            let mut joined = line.to_owned();
            while !joined.contains('{') && index + 1 < lines.len() {
                index += 1;
                joined.push(' ');
                joined.push_str(lines[index].trim());
            }
            surface.push(format!("impl {}", impl_header(&joined)));
        } else {
            for at in token_positions(lines[index], "impl") {
                // A non-line-start `impl` is NOT skipped. Any positional heuristic can be
                // defeated by a decoy: `mod m { fn decoy() {} impl AsRef<str> for TenantId
                // { .. } }` puts a `fn` earlier on the same line while the `impl` is a real
                // item. Fingerprint it instead, so only exact known argument positions pass.
                match leading_type_path(&lines[index][at + 4..]) {
                    Some(argument) => surface.push(format!("impl-arg {argument}")),
                    None => surface.push(format!("UNCLASSIFIED-IMPL {line}")),
                }
            }
        }
        index += 1;
    }
    surface.sort();
    surface
}

/// Accounts for EVERY attribute in already-stripped Rust code, in source order.
///
/// Returns `(is_inner, normalized name, normalized body)` per attribute, plus any attribute that
/// could not be terminated.
///
/// An attribute's NAME is an ordinary identifier, and Rust accepts an identifier written as a
/// raw identifier: `#[r#derive(Default)]` derives exactly as `#[derive(Default)]` does,
/// `#[r#ignore]` suppresses a test, `#[r#default]` picks an enum default — and none of them
/// contains the substring a literal scan looks for. `#`, `!` and `[` may also be separated by
/// whitespace, which is what a comment strips to. The name is therefore normalized and the
/// punctuation matched tolerantly, so callers can account for the whole attribute set instead of
/// screening for spellings somebody had to predict.
fn attributes(code: &str) -> (Vec<(bool, String, String)>, Vec<String>) {
    let bytes = code.as_bytes();
    let mut found = Vec::new();
    let mut unterminated = Vec::new();
    for (at, _) in code.match_indices('#') {
        // `r#derive` is a raw identifier, not an attribute head: its `#` has an ident byte
        // before it and no `[` after it.
        if at > 0 && is_ident_byte(bytes[at - 1]) {
            continue;
        }
        let rest = code[at + 1..].trim_start();
        let (inner, rest) = match rest.strip_prefix('!') {
            Some(tail) => (true, tail.trim_start()),
            None => (false, rest),
        };
        if !rest.starts_with('[') {
            continue;
        }
        let bracket = code.len() - rest.len();
        let Some(group) = balanced_group(&code[bracket..], '[', ']') else {
            unterminated.push(code[at..].chars().take(24).collect());
            continue;
        };
        let body = group[1..group.len() - 1]
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");
        let head = body.strip_prefix("r#").unwrap_or(&body);
        found.push((inner, leading_ident(head).to_owned(), body));
    }
    (found, unterminated)
}

/// Names that construct the governed newtype through its private form, and the keywords that
/// mean a given occurrence is a declaration or an impl header rather than a construction.
/// The private field is private to the MODULE, not to the macro expansion, so the concrete kind
/// names construct exactly as `$name` and `Self` do: a bare `fn f() -> TenantId { TenantId(s) }`
/// beside the generator bypasses `parse` while naming neither placeholder. Counting construction
/// sites is only a closure if it counts every spelling of the constructor.
const CONSTRUCTION_FORMS: [&str; 8] = [
    "$name",
    "Self",
    "TenantId",
    "UserId",
    "SessionId",
    "RequestId",
    "CommandId",
    "CorrelationId",
];
const NON_CONSTRUCTION_KEYWORDS: [&str; 9] = [
    "struct", "impl", "enum", "union", "trait", "for", "dyn", "as", "type",
];

/// Returns every expression that builds the governed newtype through its private form.
///
/// A newtype with a private field can only be produced by its own tuple/struct-literal syntax
/// inside the defining module, so counting THOSE expressions counts every construction path
/// there is — an extra visitor arm, an early return inside a helper, a branch, or a trait impl
/// nobody has thought of. Whichever entry point a deserializer picks it must reach one of these.
fn newtype_constructions(code: &str) -> Vec<String> {
    let bytes = code.as_bytes();
    let mut found = Vec::new();
    for form in CONSTRUCTION_FORMS {
        for at in code.match_indices(form).map(|(index, _)| index) {
            if at > 0 && is_ident_byte(bytes[at - 1]) {
                continue;
            }
            let after = code[at + form.len()..].trim_start();
            if !(after.starts_with('(') || after.starts_with('{')) {
                continue;
            }
            let head = code[..at].trim_end();
            // `Foo::$name(` is an associated call or an enum variant, not this newtype's ctor.
            if head.ends_with("::") {
                continue;
            }
            let keyword = {
                let bytes = head.as_bytes();
                let mut start = head.len();
                while start > 0 && is_ident_byte(bytes[start - 1]) {
                    start -= 1;
                }
                &head[start..]
            };
            if NON_CONSTRUCTION_KEYWORDS.contains(&keyword) {
                continue;
            }
            // Canonical: the name joined to its delimiter with no whitespace. `Self {` and
            // `Self{` are the SAME construction, so the normalization must collapse the gap
            // rather than preserve it — otherwise the admitted list is a list of spellings
            // again, and the two carriers can normalize the same source differently.
            let delimiter = code.len() - after.len();
            found.push(format!("{form}{}", &code[delimiter..=delimiter]));
        }
    }
    found
}

/// Returns the sorted, deduplicated normalized attribute names.
fn attribute_names(found: &[(bool, String, String)]) -> Vec<String> {
    let mut names: Vec<String> = found.iter().map(|(_, name, _)| name.clone()).collect();
    names.sort();
    names.dedup();
    names
}

/// Returns the normalized argument list of every derive attribute, in source order.
///
/// Built on the shared attribute parser, so a derive reached by any equivalent spelling —
/// spaced, comment-split or raw-identifier — is one carrier here. A derive synthesizes a trait
/// implementation that appears nowhere as text, so no `use`/`type`/`impl` accounting can see it.
fn derive_bodies(code: &str) -> Vec<String> {
    let (found, _) = attributes(code);
    found
        .iter()
        .filter(|(_, name, _)| name == "derive")
        .map(|(_, _, body)| match (body.find('('), body.rfind(')')) {
            (Some(open), Some(close)) if close > open => body[open + 1..close]
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" "),
            _ => String::new(),
        })
        .collect()
}

/// Item keywords whose declarations decide which files are compiled and which names exist.
///
/// `extern` is governed as an item rather than as a forbidden substring: `extern crate self as
/// x;` re-roots the crate under a second public name, and a comment may sit between its two
/// keywords, so only token-level accounting can see it.
const RUST_GOVERNED_ITEM_KEYWORDS: [&str; 4] = ["extern", "mod", "use", "type"];

/// Rust keywords, which can never name a macro — so `if !(a || b)` is not an `if!` invocation.
const RUST_KEYWORDS: [&str; 40] = [
    "as", "async", "await", "break", "const", "continue", "crate", "dyn", "else", "enum", "extern",
    "false", "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub",
    "ref", "return", "self", "static", "struct", "super", "trait", "true", "type", "union",
    "unsafe", "use", "where", "while", "yield", "Self",
];

/// True when `tokens` appear consecutively in `code`, separated only by whitespace.
///
/// An attribute or a macro call is a token sequence, not a string: `# /*x*/ [ path = …]` and
/// `include /*x*/ !("f.rs")` are the same items as `#[path = …]` and `include!("f.rs")`. Two
/// adjacent identifier tokens must still be separated, so the single identifier `externcrate`
/// does not match `extern` `crate`.
fn contains_token_sequence(code: &str, tokens: &[&str]) -> bool {
    let bytes = code.as_bytes();
    let mut cursor = 0;
    while let Some(found) = code[cursor..].find(tokens[0]) {
        let at = cursor + found;
        cursor = at + tokens[0].len();
        let head = tokens[0].as_bytes();
        if is_ident_byte(head[0]) && at > 0 && is_ident_byte(bytes[at - 1]) {
            continue;
        }
        let mut index = cursor;
        let mut previous = head[head.len() - 1];
        let mut matched = true;
        for token in &tokens[1..] {
            let start = index;
            while index < bytes.len() && bytes[index].is_ascii_whitespace() {
                index += 1;
            }
            let separated = index > start;
            let next = token.as_bytes();
            if !code[index..].starts_with(token)
                || (is_ident_byte(previous) && is_ident_byte(next[0]) && !separated)
            {
                matched = false;
                break;
            }
            index += token.len();
            previous = next[next.len() - 1];
        }
        if matched
            && (!is_ident_byte(previous) || index >= bytes.len() || !is_ident_byte(bytes[index]))
        {
            return true;
        }
    }
    false
}

/// Returns the start of the item at `at`, stepping back over its visibility.
fn visibility_start(code: &str, at: usize) -> usize {
    let head = code[..at].trim_end();
    let bytes = head.as_bytes();
    let mut end = head.len();
    if end > 0 && bytes[end - 1] == b')' {
        // Restricted visibility such as `pub(crate)` sits between the keyword and `pub`.
        let mut depth = 0i32;
        let mut index = end;
        let mut opened = None;
        while index > 0 {
            index -= 1;
            match bytes[index] {
                b')' => depth += 1,
                b'(' => {
                    depth -= 1;
                    if depth == 0 {
                        opened = Some(index);
                        break;
                    }
                }
                _ => {}
            }
        }
        match opened {
            Some(index) => end = code[..index].trim_end().len(),
            None => return at,
        }
    }
    let head = &code[..end];
    if head.ends_with("pub") && (head.len() == 3 || !is_ident_byte(head.as_bytes()[head.len() - 4]))
    {
        return head.len() - 3;
    }
    at
}

/// Returns the start of the attribute envelope immediately preceding `start`.
///
/// Walks backwards over balanced `#[ ... ]` and `#![ ... ]` groups, so a wrapped attribute is
/// still attached to the item it decorates.
fn attribute_envelope_start(code: &str, start: usize) -> usize {
    let mut cursor = start;
    loop {
        let head = code[..cursor].trim_end();
        if !head.ends_with(']') {
            return cursor;
        }
        let bytes = head.as_bytes();
        let mut depth = 0i32;
        let mut index = head.len();
        let mut opened = None;
        while index > 0 {
            index -= 1;
            match bytes[index] {
                b']' => depth += 1,
                b'[' => {
                    depth -= 1;
                    if depth == 0 {
                        opened = Some(index);
                        break;
                    }
                }
                _ => {}
            }
        }
        let Some(bracket) = opened else {
            return cursor;
        };
        if bracket == 0 {
            return cursor;
        }
        let mut opener = bracket - 1;
        if bytes[opener] == b'!' {
            if opener == 0 || bytes[opener - 1] != b'#' {
                return cursor;
            }
            opener -= 1;
        } else if bytes[opener] != b'#' {
            return cursor;
        }
        cursor = opener;
    }
}

/// Accounts for EVERY `mod`, `use` and `type` item, in source order, with its attribute
/// envelope and visibility.
///
/// An attribute on an admitted module changes the fingerprint, so
/// `#[path = "identity_hidden.txt"] pub mod identity;` cannot keep the admitted name while
/// Cargo compiles a different file. A use tree is one fingerprint whatever its spelling, so
/// grouped, nested, `self`-rooted and unqualified re-exports are all either listed verbatim in
/// the allowlist or rejected. Removing an admitted item fails too.
fn item_declarations(code: &str) -> (Vec<String>, Vec<String>) {
    let bytes = code.as_bytes();
    let mut tokens: Vec<(usize, &str)> = Vec::new();
    for keyword in RUST_GOVERNED_ITEM_KEYWORDS {
        for at in token_positions(code, keyword) {
            tokens.push((at, keyword));
        }
    }
    tokens.sort_unstable();
    let mut items = Vec::new();
    let mut unterminated = Vec::new();
    for (at, keyword) in tokens {
        // A path segment, a field access, or a raw identifier such as `r#type` — none is an
        // item keyword.
        let before = code[..at].trim_end();
        if before.ends_with(':') || before.ends_with('.') || code[..at].ends_with('#') {
            continue;
        }
        let item_start = attribute_envelope_start(code, visibility_start(code, at));
        let inline_module = keyword == "mod";
        let mut depth = 0i32;
        let mut terminated = None;
        for (index, byte) in bytes.iter().enumerate().skip(at + keyword.len()) {
            let byte = *byte;
            if inline_module && depth == 0 && byte == b'{' {
                terminated = Some((index, byte));
                break;
            }
            match byte {
                b'(' | b'[' | b'{' => depth += 1,
                b')' | b']' | b'}' => depth = (depth - 1).max(0),
                b';' if depth == 0 => {
                    terminated = Some((index, byte));
                    break;
                }
                _ => {}
            }
        }
        let Some((end, terminator)) = terminated else {
            unterminated.push(format!("{keyword} <unterminated>"));
            continue;
        };
        let mut header = code[item_start..end]
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");
        if terminator == b';' {
            header.push(';');
        }
        items.push(header);
    }
    (items, unterminated)
}

/// Returns the text between an `impl` token ending at `start` and its opening brace.
///
/// Spans line breaks and ignores generic and parameter nesting, exactly like
/// `_rust_impl_header` in `scripts/check_repo_contracts.py`.
fn impl_block_header(code: &str, start: usize) -> Option<&str> {
    let (mut angle, mut paren) = (0i32, 0i32);
    for (index, character) in code[start..].char_indices() {
        match character {
            '<' => angle += 1,
            '>' => angle -= 1,
            '(' => paren += 1,
            ')' => paren -= 1,
            ';' => return None,
            '{' if angle <= 0 && paren <= 0 => return Some(&code[start..start + index]),
            _ => {}
        }
    }
    None
}

/// Accounts for EVERY `impl` token as either an item or an argument-position `impl Trait`.
///
/// Token-based rather than line-based, so a second `impl` later on a line that already starts
/// with one is still recorded — the line-based split would drop it. Freezing this surface as an
/// allowlist rather than screening it for the governed kind names is what catches a blanket
/// `impl<T> Extension for T`, which names no kind and covers all six.
fn impl_declarations(code: &str) -> Vec<String> {
    let mut declarations = Vec::new();
    for at in token_positions(code, "impl") {
        let line_start = code[..at].rfind('\n').map_or(0, |index| index + 1);
        if !code[line_start..at].trim().is_empty() {
            match leading_type_path(&code[at + 4..]) {
                Some(argument) => declarations.push(format!("impl-arg {argument}")),
                None => declarations.push("UNCLASSIFIED-IMPL <no type path>".to_owned()),
            }
            continue;
        }
        match impl_block_header(code, at + 4).map(impl_header) {
            Some(header) if !header.is_empty() => declarations.push(format!("impl {header}")),
            Some(_) => declarations.push("UNCLASSIFIED-IMPL <empty>".to_owned()),
            None => declarations.push("UNCLASSIFIED-IMPL <no block>".to_owned()),
        }
    }
    declarations.sort();
    declarations
}

/// Returns the names of every `macro_rules!` definition.
fn macro_definitions(code: &str) -> Vec<String> {
    let mut names = Vec::new();
    for at in token_positions(code, "macro_rules") {
        let rest = code[at + "macro_rules".len()..].trim_start();
        let Some(rest) = rest.strip_prefix('!') else {
            continue;
        };
        let name = leading_ident(rest.trim_start());
        if !name.is_empty() {
            names.push(name.to_owned());
        }
    }
    names.sort();
    names
}

/// Consumes one balanced `open..close` group starting at char index `start` in `chars`.
///
/// Returns the index just past the closing delimiter, or `None` when the group never closes.
fn consume_group(chars: &[char], start: usize, open: char, close: char) -> Option<usize> {
    let mut depth = 0i32;
    let mut index = start;
    while index < chars.len() {
        if chars[index] == open {
            depth += 1;
        } else if chars[index] == close {
            depth -= 1;
            if depth == 0 {
                return Some(index + 1);
            }
        }
        index += 1;
    }
    None
}

/// Normalizes a char slice to single-spaced tokens, matching the Python carrier's join.
fn normalize_whitespace(chars: &[char]) -> String {
    chars
        .iter()
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn arm_delimiter(opener: char) -> Option<char> {
    match opener {
        '(' => Some(')'),
        '[' => Some(']'),
        '{' => Some('}'),
        _ => None,
    }
}

/// Returns the matcher of each arm of a macro body, in order.
///
/// Each arm is one balanced matcher group, `=>`, one balanced transcriber group and an optional
/// `;`. A malformed or unterminated arm becomes a sentinel matcher rather than being dropped, so
/// the caller fails closed on it.
fn macro_arm_matchers(body: &str) -> Vec<String> {
    let chars: Vec<char> = body.chars().collect();
    let n = chars.len();
    let mut matchers = Vec::new();
    let mut cursor = 0usize;
    loop {
        while cursor < n && chars[cursor].is_whitespace() {
            cursor += 1;
        }
        if cursor >= n {
            break;
        }
        let opener = chars[cursor];
        let Some(closer) = arm_delimiter(opener) else {
            matchers.push(format!("<unparsed:{opener}>"));
            break;
        };
        let Some(next) = consume_group(&chars, cursor, opener, closer) else {
            matchers.push("<unterminated-matcher>".to_owned());
            break;
        };
        let matcher = normalize_whitespace(&chars[cursor..next]);
        cursor = next;
        while cursor < n && chars[cursor].is_whitespace() {
            cursor += 1;
        }
        if !(cursor + 1 < n && chars[cursor] == '=' && chars[cursor + 1] == '>') {
            matchers.push(format!("{matcher} <no-arrow>"));
            break;
        }
        cursor += 2;
        while cursor < n && chars[cursor].is_whitespace() {
            cursor += 1;
        }
        let transcriber = chars.get(cursor).copied().and_then(arm_delimiter);
        let Some(transcriber_close) = transcriber else {
            matchers.push(format!("{matcher} <no-body>"));
            break;
        };
        let Some(next) = consume_group(&chars, cursor, chars[cursor], transcriber_close) else {
            matchers.push(format!("{matcher} <unterminated-body>"));
            break;
        };
        cursor = next;
        matchers.push(matcher);
        while cursor < n && chars[cursor].is_whitespace() {
            cursor += 1;
        }
        if cursor < n && chars[cursor] == ';' {
            cursor += 1;
        }
    }
    matchers
}

/// Returns `(name, [arm matcher, ...])` for every `macro_rules!` definition, in source order.
///
/// Rust selects the FIRST arm whose matcher matches, so an earlier catch-all arm intercepts
/// every existing call while the true arm below it stays present but unread. Pinning a name and
/// the presence of one matcher line cannot see that; accounting for the complete arm-matcher
/// list can.
fn macro_arms(code: &str) -> Vec<(String, Vec<String>)> {
    let mut definitions = Vec::new();
    for at in token_positions(code, "macro_rules") {
        // A comment strips to a space, so `macro_rules /*x*/ ! name` is the same definition as
        // `macro_rules!name`; tolerate whitespace between the keyword and the `!`.
        let Some(after_bang) = code[at + "macro_rules".len()..]
            .trim_start()
            .strip_prefix('!')
        else {
            continue;
        };
        let after = after_bang.trim_start();
        let name = leading_ident(after);
        if name.is_empty() {
            continue;
        }
        let rest = &after[name.len()..];
        let Some(brace) = rest.find('{') else {
            definitions.push((
                name.to_owned(),
                vec!["<unterminated-definition>".to_owned()],
            ));
            continue;
        };
        let Some(body) = balanced_group(&rest[brace..], '{', '}') else {
            definitions.push((
                name.to_owned(),
                vec!["<unterminated-definition>".to_owned()],
            ));
            continue;
        };
        let inner = &body[1..body.len() - 1];
        definitions.push((name.to_owned(), macro_arm_matchers(inner)));
    }
    definitions
}

/// Returns the brace-matched body of one `macro_rules! <name>`, including the outer braces.
fn macro_body<'a>(code: &'a str, name: &str) -> Option<&'a str> {
    for at in token_positions(code, "macro_rules") {
        let after_bang = code[at + "macro_rules".len()..]
            .trim_start()
            .strip_prefix('!')?;
        let after = after_bang.trim_start();
        if leading_ident(after) == name {
            let rest = &after[name.len()..];
            let brace = rest.find('{')?;
            return balanced_group(&rest[brace..], '{', '}');
        }
    }
    None
}

/// Returns the brace-matched body of `fn <name>`, tolerating a generic parameter list.
///
/// Anchoring on `name(` would return `None` for every generic function and silently turn a body
/// pin into a vacuous check, so `fn visit_string<E>(..)` is found as readily as `fn name(..)`.
fn function_body<'a>(code: &'a str, name: &str) -> Option<&'a str> {
    let needle = format!("fn {name}");
    let mut cursor = 0;
    while let Some(found) = code[cursor..].find(&needle) {
        let at = cursor + found;
        cursor = at + needle.len();
        // A word boundary on both sides, so `visit_str` does not match `visit_string`.
        if at > 0 && is_ident_byte(code.as_bytes()[at - 1]) {
            continue;
        }
        let rest = code[cursor..].trim_start();
        if is_ident_byte(rest.as_bytes().first().copied().unwrap_or(b' ')) {
            continue;
        }
        // Skip a generic parameter list before the argument list.
        let after_generics = if rest.starts_with('<') {
            match balanced_group(rest, '<', '>') {
                Some(group) => rest[group.len()..].trim_start(),
                None => continue,
            }
        } else {
            rest
        };
        if !after_generics.starts_with('(') {
            continue;
        }
        let tail_at = code.len() - after_generics.len();
        let brace = code[tail_at..].find('{')?;
        return balanced_group(&code[tail_at + brace..], '{', '}');
    }
    None
}

/// Reported in place of a name when a `fn` declaration's name is not an ASCII identifier.
const UNNAMED_FUNCTION: &str = "<unnamed>";

/// Accounts for EVERY `fn` declaration of already-stripped Rust code, in source order.
///
/// Returns `(name, normalized body)` pairs plus the name of any declaration whose body could not
/// be resolved. A body is its brace-matched block with whitespace collapsed, or `";"` for a
/// declaration that has none.
///
/// Pinning function NAMES freezes the module's shape but says nothing about what each function
/// does, and a containment check says nothing about what else it does — a branch that still
/// contains the admitted call can return before ever reaching it. The mirror of the repository
/// checker's `rust_functions`.
fn functions(code: &str) -> (Vec<(String, String)>, Vec<String>) {
    let mut found = Vec::new();
    let mut unresolved = Vec::new();
    for at in token_positions(code, "fn") {
        // `r#fn` is an ordinary identifier, not the item keyword — the same exclusion the item
        // scan makes for `r#type`.
        if code[..at].ends_with('#') {
            continue;
        }
        let mut tail = code[at + 2..].trim_start();
        let name = leading_ident(tail).to_owned();
        if name.is_empty() {
            // `fn(u8) -> u8` is a function POINTER TYPE, not a declaration: it carries no name.
            if tail.starts_with('(') {
                continue;
            }
            // Anything else is a declaration this scan cannot classify — rustc accepts non-ASCII
            // identifiers and this lexer is deliberately ASCII-only. Skipping it would drop the
            // declaration from the inventory silently, so it fails closed instead.
            unresolved.push(UNNAMED_FUNCTION.to_owned());
            continue;
        }
        tail = tail[name.len()..].trim_start();
        if tail.starts_with('<') {
            match balanced_group(tail, '<', '>') {
                Some(group) => tail = tail[group.len()..].trim_start(),
                None => {
                    unresolved.push(name);
                    continue;
                }
            }
        }
        // The parameter list is consumed as a BALANCED group rather than scanned past, because a
        // const-generic array length such as `x: [u8; { N }]` puts a brace inside it.
        match balanced_group(tail, '(', ')') {
            Some(group) => tail = &tail[group.len()..],
            None => {
                unresolved.push(name);
                continue;
            }
        }
        // A return type and a `where` clause may carry `(`, `[` and `<`, never `{` or `;`.
        let Some(opener) = tail.find(['{', ';']) else {
            unresolved.push(name);
            continue;
        };
        if tail.as_bytes()[opener] == b';' {
            found.push((name, ";".to_owned()));
            continue;
        }
        match balanced_group(&tail[opener..], '{', '}') {
            Some(group) => {
                found.push((name, group.split_whitespace().collect::<Vec<_>>().join(" ")));
            }
            None => unresolved.push(name),
        }
    }
    (found, unresolved)
}

/// Returns the balanced `open..close` group that must begin at the start of `text`, or `None`.
fn balanced_group(text: &str, open: char, close: char) -> Option<&str> {
    if !text.starts_with(open) {
        return None;
    }
    let mut depth = 0i32;
    for (index, character) in text.char_indices() {
        if character == open {
            depth += 1;
        } else if character == close {
            depth -= 1;
            if depth == 0 {
                return Some(&text[..index + character.len_utf8()]);
            }
        }
    }
    None
}

/// Returns `(name, argument)` for every macro invocation plus any unterminated name.
///
/// A macro is the one item category that can implement a trait for a governed type without
/// naming it in a `use`, a `type` or an `impl` header: the definition sees `$t` and the
/// invocation site sees only a macro call. An unterminated invocation is reported rather than
/// dropped, so the caller accounts for exactly as many macros as the source contains.
fn macro_invocation_arguments(code: &str) -> (Vec<(String, String)>, Vec<String>) {
    let bytes = code.as_bytes();
    let mut invocations = Vec::new();
    let mut unterminated = Vec::new();
    for (index, byte) in bytes.iter().enumerate() {
        if *byte != b'!' {
            continue;
        }
        let head = code[..index].trim_end();
        let head_bytes = head.as_bytes();
        let mut start = head.len();
        while start > 0 && is_ident_byte(head_bytes[start - 1]) {
            start -= 1;
        }
        if start == head.len() {
            continue;
        }
        let name = &head[start..];
        if name == "macro_rules" || RUST_KEYWORDS.contains(&name) {
            continue;
        }
        let tail = &code[index + 1..];
        let offset = index + 1 + (tail.len() - tail.trim_start().len());
        let Some(open) = bytes.get(offset).copied() else {
            continue;
        };
        let close = match open {
            b'(' => b')',
            b'[' => b']',
            b'{' => b'}',
            _ => continue,
        };
        let mut depth = 0i32;
        let mut closed = false;
        for cursor in offset..bytes.len() {
            if bytes[cursor] == open {
                depth += 1;
            } else if bytes[cursor] == close {
                depth -= 1;
                if depth == 0 {
                    invocations.push((
                        name.to_owned(),
                        code[offset + 1..cursor]
                            .split_whitespace()
                            .collect::<Vec<_>>()
                            .join(" "),
                    ));
                    closed = true;
                    break;
                }
            }
        }
        if !closed {
            unterminated.push(name.to_owned());
        }
    }
    (invocations, unterminated)
}

/// Returns the sorted, deduplicated macro invocation names of `code`.
fn macro_invocation_names(invocations: &[(String, String)]) -> Vec<String> {
    let mut names: Vec<String> = invocations.iter().map(|(name, _)| name.clone()).collect();
    names.sort();
    names.dedup();
    names
}

/// Freezes the admitted public surface of the identity module.
///
/// Bound to `AUTH-012` so the registered binding itself observes the negative-space guard
/// rather than delegating it to a checker the binding never runs.
fn assert_public_surface_is_frozen() {
    let code = strip_comments_and_literals(IDENTITY_SOURCE);
    assert!(
        !code.contains("#[macro_export]"),
        "identity module exported its private value generator"
    );

    // Total accounting over attributes by NORMALIZED name. An attribute name is an ordinary
    // identifier, so `#[r#derive(Default)]` is the same attribute as `#[derive(Default)]` and no
    // spelling blacklist can enumerate the equivalents; an unadmitted name is drift whatever it
    // is called. Derive ARGUMENTS are pinned by the public-surface allowlist below, because a
    // derive is the one attribute that adds public API without adding any text to scan.
    let (identity_attributes, unterminated_attributes) = attributes(&code);
    assert!(
        unterminated_attributes.is_empty(),
        "unterminated attribute in the identity module: {unterminated_attributes:?}"
    );
    assert_eq!(
        attribute_names(&identity_attributes),
        ADMITTED_IDENTITY_ATTRIBUTE_NAMES
            .iter()
            .map(|name| (*name).to_owned())
            .collect::<Vec<_>>(),
        "identity module attribute names drifted"
    );

    // THE closure for every construction path, Serde's included. Naming Serde's entry points
    // cannot close the class: each implemented `visit_*` method is an independent construction
    // path, and a branch inside a helper that still contains the parse call is another. But a
    // private field can only be filled by this module's own tuple/struct-literal syntax, so
    // requiring exactly ONE such expression — inside the checked constructor — leaves an extra
    // arm, an early return or a future trait impl with nowhere to build the value.
    let mut declared_functions: Vec<String> = Vec::new();
    for at in token_positions(&code, "fn") {
        let name = leading_ident(code[at + 2..].trim_start());
        if !name.is_empty() {
            declared_functions.push(name.to_owned());
        }
    }
    declared_functions.sort();
    let mut admitted_names = ADMITTED_IDENTITY_FUNCTION_BODIES
        .iter()
        .map(|(name, _)| (*name).to_owned())
        .collect::<Vec<_>>();
    admitted_names.sort();
    assert_eq!(
        declared_functions, admitted_names,
        "identity module function inventory drifted"
    );
    // …and their exact BODIES, because a name inventory says nothing about what a function does
    // and a containment check says nothing about what else it does.
    let (declared_bodies, unresolved_bodies) = functions(&code);
    assert!(
        unresolved_bodies.is_empty(),
        "identity module function body unreadable: {unresolved_bodies:?}"
    );
    assert_eq!(
        declared_bodies,
        ADMITTED_IDENTITY_FUNCTION_BODIES
            .iter()
            .map(|(name, body)| ((*name).to_owned(), (*body).to_owned()))
            .collect::<Vec<_>>(),
        "identity module function body drifted"
    );
    assert_eq!(
        newtype_constructions(&code),
        ADMITTED_CONSTRUCTIONS
            .iter()
            .map(|form| (*form).to_owned())
            .collect::<Vec<_>>(),
        "identity value constructed outside the checked constructor"
    );
    let Some(constructor_body) = function_body(&code, "parse") else {
        panic!("identity module lost its checked constructor");
    };
    assert!(
        !newtype_constructions(constructor_body).is_empty(),
        "identity value construction does not live in the checked constructor"
    );
    // The two bodies the contract rests on, by exact equality and named separately so a failure
    // says which invariant broke rather than only that the module drifted.
    for (label, function, expected) in [
        (
            "checked constructor",
            "parse",
            ADMITTED_IDENTITY_FUNCTION_BODIES[PARSE_BODY_INDEX].1,
        ),
        (
            "Deserialize",
            "deserialize",
            ADMITTED_IDENTITY_FUNCTION_BODIES[DESERIALIZE_BODY_INDEX].1,
        ),
    ] {
        let Some(body) = function_body(&code, function) else {
            panic!("identity module {label} body unreadable");
        };
        assert_eq!(
            body.split_whitespace().collect::<Vec<_>>().join(" "),
            expected,
            "identity module {label} body is not the frozen one"
        );
    }
    // A named-field struct has no constructor function item, so `let ctor = $name;` does not
    // compile and a struct literal — syntax, not a value — is the only way to produce one.
    assert!(
        code.split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
            .contains(IDENTITY_STRUCT_DECLARATION),
        "identity value representation drifted from {IDENTITY_STRUCT_DECLARATION}"
    );
    for at in token_positions(&code, "struct") {
        let tail = code[at + 6..].trim_start();
        let name = leading_ident(tail);
        let mut after = tail[name.len()..].trim_start();
        // A generic parameter list sits between the name and the field list, so `struct S<T>(T);`
        // is a tuple struct too. The repository checker skips it; not skipping it here would make
        // the two carriers disagree on exactly the declaration this rule exists to reject.
        if after.starts_with('<')
            && let Some(group) = balanced_group(after, '<', '>')
        {
            after = after[group.len()..].trim_start();
        }
        assert!(
            !after.starts_with('('),
            "identity module must declare no constructor function item: {name} is a tuple struct"
        );
    }
    // A hand-written visitor is what reopened this class twice; the module carries none, so
    // there is no per-method arm set for evidence to keep enumerating.
    for carrier in ["Visitor", "visit_", "deserialize_any"] {
        assert!(
            !code.contains(carrier),
            "identity module must not hand-write a Serde visitor: {carrier} is forbidden"
        );
    }

    // A single-file textual allowlist proves nothing if the file can splice in another file
    // or grow a submodule that no scan reads.
    for (carrier, tokens) in [
        ("include!", &["include", "!"][..]),
        ("include_str!", &["include_str", "!"][..]),
        ("include_bytes!", &["include_bytes", "!"][..]),
        ("#[path", &["#", "[", "path"][..]),
    ] {
        assert!(
            !contains_token_sequence(&code, tokens),
            "identity module must not splice external source: {carrier}"
        );
    }
    assert!(
        !contains_token_sequence(&code, &["#", "!", "["]),
        "identity module must not carry an inner attribute"
    );
    // Which files Cargo compiles into the crate is decided by non-inline `mod` declarations,
    // not by a file extension. Pinning the declarations pins the compiled set semantically, so
    // no attribute spelling — `#[path]`, `#[cfg_attr(all(), path = "x.txt")]` — can introduce
    // a module the scan never reads.
    //
    // Pinning module NAMES is still not the same as pinning module SOURCES, and pinning a
    // re-export by the spelling `crate::identity` is not the same as accounting for the use
    // tree that contains it. Both are settled by the exact item allowlist below.
    for (label, source, modules, items, macros, admits_kind_macro_arguments) in [
        (
            "identity.rs",
            IDENTITY_SOURCE,
            &[] as &[&str],
            &ADMITTED_IDENTITY_ITEMS as &[&str],
            &["identity_value"] as &[&str],
            true,
        ),
        (
            "invocation.rs",
            INVOCATION_SOURCE,
            &[] as &[&str],
            &ADMITTED_INVOCATION_ITEMS as &[&str],
            &["authority_id"] as &[&str],
            false,
        ),
        (
            "market.rs",
            MARKET_SOURCE,
            &["authority", "capability", "grant", "installation", "update"] as &[&str],
            &ADMITTED_MARKET_ITEMS as &[&str],
            &[] as &[&str],
            false,
        ),
        (
            "lib.rs",
            LIB_SOURCE,
            &["identity", "invocation", "market", "session"] as &[&str],
            &ADMITTED_LIB_ITEMS as &[&str],
            &[] as &[&str],
            false,
        ),
        (
            "session.rs",
            SESSION_SOURCE,
            &[] as &[&str],
            &ADMITTED_SESSION_ITEMS as &[&str],
            &[] as &[&str],
            false,
        ),
    ] {
        let governed = strip_comments_and_literals(source);
        assert!(
            !governed.contains("cfg_attr"),
            "platform-core source must not carry cfg_attr: {label}"
        );
        // Second, independent carrier alongside the `extern` item accounting below. Matched as
        // a token sequence, because a comment between the two keywords is a separator: the
        // stripper turns `extern/**/crate` into `extern crate`, never `externcrate`.
        assert!(
            !contains_token_sequence(&governed, &["extern", "crate"]),
            "platform-core source must not carry `extern crate`: {label}"
        );
        assert!(
            !contains_token_sequence(&governed, &["#", "!", "["]),
            "platform-core source must not carry an inner attribute: {label}"
        );
        let mut declared: Vec<String> = Vec::new();
        for statement in governed.split(';') {
            let normalized = statement.split_whitespace().collect::<Vec<_>>().join(" ");
            let tail = normalized
                .strip_suffix(|_: char| false)
                .unwrap_or(&normalized);
            if let Some(name) = tail.rsplit("mod ").next()
                && tail.contains("mod ")
                && !name.is_empty()
                && name
                    .chars()
                    .all(|character| character.is_ascii_alphanumeric() || character == '_')
            {
                declared.push(name.to_owned());
            }
        }
        declared.sort();
        assert_eq!(
            declared,
            modules
                .iter()
                .map(|name| (*name).to_owned())
                .collect::<Vec<_>>(),
            "platform-core module declarations drifted in {label}"
        );

        let (declarations, unterminated) = item_declarations(&governed);
        assert!(
            unterminated.is_empty(),
            "unterminated platform-core item declaration in {label}: {unterminated:?}"
        );
        assert_eq!(
            declarations,
            items
                .iter()
                .map(|item| (*item).to_owned())
                .collect::<Vec<_>>(),
            "platform-core item declarations drifted in {label}"
        );
        let definitions = macro_definitions(&governed);
        assert_eq!(
            definitions,
            macros
                .iter()
                .map(|name| (*name).to_owned())
                .collect::<Vec<_>>(),
            "platform-core macro definitions drifted in {label}"
        );
        for definition in &definitions {
            assert!(
                !RUST_SHADOWABLE_MACRO_NAMES.contains(&definition.as_str()),
                "platform-core source redefines the standard {definition}! macro: {label}"
            );
        }
        let (invocations, unterminated_macros) = macro_invocation_arguments(&governed);
        assert!(
            unterminated_macros.is_empty(),
            "unterminated platform-core macro invocation in {label}: {unterminated_macros:?}"
        );
        // Exhaustive by label, NOT by fallthrough default. A `_ =>` arm here would silently
        // check an unregistered source against lib.rs's list instead of failing closed, which is
        // the opposite of what a total-accounting guard is for.
        let admitted_invocations: &[&str] = match label {
            "identity.rs" => &ADMITTED_IDENTITY_MACRO_INVOCATIONS,
            "invocation.rs" => &ADMITTED_INVOCATION_MACRO_INVOCATIONS,
            "market.rs" => &ADMITTED_MARKET_MACRO_INVOCATIONS,
            "lib.rs" => &ADMITTED_LIB_MACRO_INVOCATIONS,
            "session.rs" => &ADMITTED_SESSION_MACRO_INVOCATIONS,
            other => panic!("ungoverned platform-core source macro invocations: {other}"),
        };
        assert_eq!(
            macro_invocation_names(&invocations),
            admitted_invocations
                .iter()
                .map(|name| (*name).to_owned())
                .collect::<Vec<_>>(),
            "platform-core macro invocations drifted in {label}"
        );
        if !admits_kind_macro_arguments {
            for (name, argument) in &invocations {
                for kind in ADMITTED_IDENTITY_KINDS {
                    assert!(
                        token_positions(argument, kind).is_empty(),
                        "identity kind passed to a macro outside the identity module: \
                         {label}: {name}!({argument})"
                    );
                }
            }
        }
    }

    // Cargo, not Rust, decides which files become which target. `[lib] path`,
    // `[package] build`, `[[bin]]`, `[[example]]`, `[[bench]]` and `[[test]]` each name a
    // source file no Rust scan reads, and `[[test]]` can rename or unharness the bound
    // acceptance test. The manifest is pinned by exact key sets, not screened for keys.
    let entries = manifest_entries(MANIFEST_SOURCE);
    let mut tables: Vec<String> = entries
        .iter()
        .map(|(table, _)| table.clone())
        .filter(|table| !table.is_empty())
        .collect();
    tables.sort();
    tables.dedup();
    assert_eq!(
        tables,
        ADMITTED_MANIFEST_TABLES
            .iter()
            .map(|name| (*name).to_owned())
            .collect::<Vec<_>>(),
        "platform-core manifest tables drifted"
    );
    for (table, admitted) in [
        ("package", &ADMITTED_MANIFEST_PACKAGE_KEYS[..]),
        ("dependencies", &ADMITTED_MANIFEST_DEPENDENCIES[..]),
        ("dev-dependencies", &ADMITTED_MANIFEST_DEV_DEPENDENCIES[..]),
    ] {
        assert_eq!(
            manifest_keys(&entries, table),
            admitted
                .iter()
                .map(|name| (*name).to_owned())
                .collect::<Vec<_>>(),
            "platform-core [{table}] drifted"
        );
    }
    assert_eq!(
        manifest_keys(&entries, "lib"),
        vec!["path".to_owned()],
        "platform-core [lib] keys drifted"
    );
    // The one target path that decides which file becomes the library root.
    let lib_path = MANIFEST_SOURCE
        .lines()
        .skip_while(|line| line.trim() != "[lib]")
        .find_map(|line| line.trim().strip_prefix("path"))
        .and_then(|rest| rest.trim().strip_prefix('='))
        .map(|value| value.split_whitespace().next().unwrap_or_default());
    assert_eq!(
        lib_path,
        Some(ADMITTED_MANIFEST_LIB_PATH),
        "platform-core [lib] path drifted"
    );

    // A dependency NAME allowlist says nothing about what the name resolves to:
    // `hex = { path = "../fake-hex" }` keeps the admitted name while Cargo compiles a different
    // crate, and every Rust scan still reads `hex::encode`. Specifications are compared value
    // for value, and the resolved identity is read from the committed lockfile.
    for (table, admitted) in [
        ("dependencies", &ADMITTED_DEPENDENCY_SPECS[..]),
        ("dev-dependencies", &ADMITTED_DEV_DEPENDENCY_SPECS[..]),
    ] {
        assert_eq!(
            manifest_specifications(MANIFEST_SOURCE, table),
            admitted
                .iter()
                .map(|spec| (*spec).to_owned())
                .collect::<Vec<_>>(),
            "platform-core [{table}] specifications drifted"
        );
    }
    for (name, expected) in RESOLVED_DEPENDENCY_SOURCES {
        assert_eq!(
            locked_package_sources(LOCKFILE_SOURCE, name),
            vec![expected.to_owned()],
            "governed dependency {name} resolved to an unexpected source"
        );
    }

    // The frozen surface belongs to the value kinds, not to one file. Rust's orphan rule does
    // not stop a sibling module in the same crate from adding a second inherent impl.
    for (label, sibling) in [
        ("invocation.rs", INVOCATION_SOURCE),
        ("market.rs", MARKET_SOURCE),
        ("lib.rs", LIB_SOURCE),
        ("session.rs", SESSION_SOURCE),
    ] {
        let sibling_code = strip_comments_and_literals(sibling);
        for (carrier, tokens) in [
            ("include!", &["include", "!"][..]),
            ("#[path", &["#", "[", "path"][..]),
        ] {
            assert!(
                !contains_token_sequence(&sibling_code, tokens),
                "platform-core source must not splice external source: {label}: {carrier}"
            );
        }
        // Total attribute-name accounting applies to EVERY governed source, not only the
        // identity module: a `#[r#doc(hidden)]` or any other unadmitted attribute smuggled into
        // a sibling is the same class this rule exists to close.
        let (sibling_attributes, sibling_unterminated) = attributes(&sibling_code);
        assert!(
            sibling_unterminated.is_empty(),
            "unterminated attribute in {label}: {sibling_unterminated:?}"
        );
        let admitted_attribute_names = ADMITTED_SIBLING_ATTRIBUTE_NAMES
            .iter()
            .find(|(name, _)| *name == label)
            .map(|(_, names)| *names)
            .unwrap_or_else(|| panic!("ungoverned platform-core sibling: {label}"));
        assert_eq!(
            attribute_names(&sibling_attributes),
            admitted_attribute_names
                .iter()
                .map(|name| (*name).to_owned())
                .collect::<Vec<_>>(),
            "platform-core attribute names drifted in {label}"
        );
        // Every `use`/`type` binding of an admitted kind is rejected, private ones included,
        // rather than resolved. A local alias does not change Rust's self type, so
        // `use crate::identity::TenantId as Tenant; impl AsRef<str> for Tenant { .. }` is a
        // real implementation for the governed type while every textual comparison sees
        // `Tenant`. Refusing to create the alias removes the thing that would need resolving.
        for statement in sibling_code.split(';') {
            let normalized = statement.split_whitespace().collect::<Vec<_>>().join(" ");
            let at = ["pub use ", "pub type ", "use ", "type "]
                .iter()
                .filter_map(|keyword| {
                    if normalized.starts_with(keyword) {
                        Some(0)
                    } else {
                        normalized
                            .find(&format!(" {keyword}"))
                            .map(|index| index + 1)
                    }
                })
                .min();
            let Some(at) = at else {
                continue;
            };
            let declaration = format!("{};", &normalized[at..]);
            let mentions_kind = ADMITTED_IDENTITY_KINDS
                .iter()
                .any(|kind| declaration.contains(kind));
            // A whole-module re-export (`pub use crate::identity as identity_alias;`) names no
            // kind yet publishes every one of them under a second path.
            let mentions_module = ["crate::identity", "self::identity", "super::identity"]
                .iter()
                .any(|path| declaration.contains(path));
            // Enumerated by exact file name together with exact normalized text, never a
            // prefix, regex or predicate over `crate::identity::` — a pattern would re-open the
            // alias class this rule exists to close.
            let admitted = ADMITTED_CROSS_FILE_IDENTITY_BINDINGS
                .iter()
                .any(|(file, text)| *file == label && declaration == *text);
            assert!(
                !(mentions_kind || mentions_module) || admitted,
                "identity value alias or import outside the identity module: {label}: {declaration}"
            );
        }
        // Every `impl` token, whatever its line position: the argument-position fingerprint
        // records only the leading type path, which would drop the `for <Target>` of a real
        // block hidden mid-line behind a decoy `fn`.
        for target in impl_self_types(&sibling_code) {
            let simple = target.rsplit("::").next().unwrap_or(&target);
            assert!(
                !ADMITTED_IDENTITY_KINDS.contains(&simple),
                "identity value implementation outside the identity module: {label}: {target}"
            );
        }
        // …and the sibling implementation surface is an allowlist, not a kind blacklist: a
        // blanket `impl<T> Extension for T` names no governed kind yet covers all six.
        // Exhaustive by label, for the same reason as the macro-invocation lookup above.
        let admitted_impls: &[&str] = match label {
            "invocation.rs" => &ADMITTED_INVOCATION_IMPLS,
            "market.rs" => &ADMITTED_MARKET_IMPLS,
            "lib.rs" => &ADMITTED_LIB_IMPLS,
            "session.rs" => &ADMITTED_SESSION_IMPLS,
            other => panic!("ungoverned platform-core sibling implementations: {other}"),
        };
        assert_eq!(
            impl_declarations(&sibling_code),
            admitted_impls
                .iter()
                .map(|entry| (*entry).to_owned())
                .collect::<Vec<_>>(),
            "platform-core sibling implementation surface drifted in {label}"
        );
    }

    // The generator macro and its invocations are frozen too. Widening the matcher to accept
    // an `$extra:item` fragment and forwarding a trait implementation through an existing
    // invocation adds real public API without adding any new macro definition.
    let definitions: Vec<&str> = code
        .lines()
        .map(str::trim)
        .filter(|line| line.starts_with("macro_rules!"))
        .collect();
    assert_eq!(
        definitions,
        vec!["macro_rules! identity_value {"],
        "identity module macro definitions drifted"
    );
    assert!(
        code.lines()
            .any(|line| line.split_whitespace().collect::<Vec<_>>().join(" ")
                == "($(#[$attribute:meta])* $name:ident) => {"),
        "identity value generator matcher drifted from the frozen grammar"
    );
    for body in generator_invocation_bodies(&code) {
        let argument = body.trim();
        assert!(
            !argument.is_empty()
                && argument
                    .chars()
                    .all(|character| character.is_ascii_alphanumeric() || character == '_'),
            "identity value generator invocation must pass exactly one kind name: {argument}"
        );
    }

    assert_eq!(
        public_surface(&code),
        ADMITTED_PUBLIC_SURFACE.to_vec(),
        "identity module public surface drifted from the admitted allowlist"
    );
}

/// Returns the brace-balanced argument of every `identity_value!` invocation.
fn generator_invocation_bodies(code: &str) -> Vec<String> {
    let bytes = code.as_bytes();
    let mut bodies = Vec::new();
    let mut cursor = 0;
    while let Some(found) = code[cursor..].find("identity_value!") {
        let mut index = cursor + found + "identity_value!".len();
        while index < bytes.len() && bytes[index].is_ascii_whitespace() {
            index += 1;
        }
        if index < bytes.len() && bytes[index] == b'{' {
            let mut depth = 0;
            let start = index;
            while index < bytes.len() {
                match bytes[index] {
                    b'{' => depth += 1,
                    b'}' => {
                        depth -= 1;
                        if depth == 0 {
                            break;
                        }
                    }
                    _ => {}
                }
                index += 1;
            }
            if index < bytes.len() {
                bodies.push(code[start + 1..index].to_owned());
            }
        }
        cursor = cursor + found + "identity_value!".len();
    }
    bodies
}

const TEST_SOURCE: &str = include_str!("platform_identity.rs");

/// The exact attributes every bound acceptance test may carry.
const ADMITTED_TEST_ATTRIBUTES: [&str; 1] = ["#[test]"];

/// Every attribute NAME admitted in the identity module, normalized (`r#` removed) and sorted.
///
/// `$attribute` is the generator's `#[$attribute]` / `#[$attribute:meta]` forwarding of a
/// caller-supplied attribute; it names a metavariable rather than a fixed attribute.
const ADMITTED_IDENTITY_ATTRIBUTE_NAMES: [&str; 4] = ["$attribute", "derive", "doc", "must_use"];

/// The bound test file carries exactly one attribute name. `#[r#ignore]` normalizes to `ignore`
/// and is rejected here even though it contains no `#[ignore]` substring.
const ADMITTED_TEST_ATTRIBUTE_NAMES: [&str; 1] = ["test"];

/// The one admitted construction of the governed newtype: `Ok(Self { value })` inside `parse`.
const ADMITTED_CONSTRUCTIONS: [&str; 1] = ["Self{"];

/// The frozen representation, and the reason it is not a tuple struct.
///
/// A tuple struct's constructor is a VALUE, not only a syntax: `let ctor = $name; ctor(text)`
/// fills the private field while writing neither `$name(` nor `Self(` at the construction site,
/// so it satisfies every construction count there is. That value cannot be scanned away — it can
/// be bound, aliased, passed as an argument or returned before it is ever called. A named-field
/// struct has no constructor function item at all, leaving a struct literal as the only way to
/// produce one, and a struct literal is syntax that cannot be bound.
const IDENTITY_STRUCT_DECLARATION: &str = "pub struct $name { value: String, }";

/// Every function the identity module may declare, WITH ITS EXACT BODY, in source order.
///
/// A name inventory freezes the module's shape but says nothing about what each function does,
/// and a containment check says nothing about what else it does: an early return above the
/// admitted call keeps it satisfied and never reaches it, and one construction site inside
/// `parse` is still one site if the branch guarding it is inverted. Bodies are therefore
/// accounted for exactly.
///
/// LIMIT, stated rather than implied: bodies are compared after comments and literal PAYLOADS
/// are stripped, so this pins control flow and token shape, not the bytes inside a literal.
/// That residue is closed by `assert_grammar_is_exhaustive_over_bytes`, which drives all 256
/// byte values through each grammar position instead of a hand-picked corpus.
const ADMITTED_IDENTITY_FUNCTION_BODIES: [(&str, &str); 14] = [
    ("value_kind", "{ self.value_kind }"),
    ("kind", "{ self.kind }"),
    (
        "fmt",
        "{ let value_kind = self.value_kind; match self.kind { \
         IdentityValueErrorKind::Empty => { write!(formatter, ) } \
         IdentityValueErrorKind::TooLong { max_bytes } => write!( formatter, ), \
         IdentityValueErrorKind::InvalidStart => write!( formatter, ), \
         IdentityValueErrorKind::InvalidCharacter { byte_index } => write!( formatter, ), \
         IdentityValueErrorKind::InvalidEnd => write!( formatter, ), } }",
    ),
    ("is_boundary_byte", "{ byte.is_ascii_alphanumeric() }"),
    (
        "is_interior_byte",
        "{ byte.is_ascii_alphanumeric() || matches!(byte, | | | ) }",
    ),
    (
        "classify",
        "{ let bytes = value.as_bytes(); \
         let Some((&first, after_first)) = bytes.split_first() else { \
         return Err(IdentityValueErrorKind::Empty); }; \
         if bytes.len() > MAX_IDENTITY_BYTES { return Err(IdentityValueErrorKind::TooLong { \
         max_bytes: MAX_IDENTITY_BYTES, }); } \
         if !is_boundary_byte(first) { return Err(IdentityValueErrorKind::InvalidStart); } \
         let Some((&last, interior)) = after_first.split_last() else { return Ok(()); }; \
         for (offset, &byte) in interior.iter().enumerate() { if !is_interior_byte(byte) { \
         return Err(IdentityValueErrorKind::InvalidCharacter { byte_index: offset + 1, }); } } \
         if !is_boundary_byte(last) { return Err(IdentityValueErrorKind::InvalidEnd); } Ok(()) }",
    ),
    (
        "parse",
        "{ let value = value.into(); match classify(&value) { Ok(()) => Ok(Self { value }), \
         Err(kind) => Err(IdentityValueError { value_kind: stringify!($name), kind, }), } }",
    ),
    ("as_str", "{ &self.value }"),
    ("try_from", "{ Self::parse(value) }"),
    ("try_from", "{ Self::parse(value) }"),
    ("from_str", "{ Self::parse(value) }"),
    ("fmt", "{ formatter.write_str(&self.value) }"),
    ("serialize", "{ serializer.serialize_str(&self.value) }"),
    (
        "deserialize",
        "{ let value = String::deserialize(deserializer)?; \
         $name::parse(value).map_err(de::Error::custom) }",
    ),
];

/// Index of the checked constructor and of `Deserialize` inside the body table above, so the two
/// bodies the whole contract rests on can be named in their own assertion messages.
const PARSE_BODY_INDEX: usize = 6;
const DESERIALIZE_BODY_INDEX: usize = 13;

/// Admitted attribute names of each governed sibling, mirroring the repository checker.
///
/// The rule belongs to every governed source, not only the identity module: an unadmitted
/// attribute in a sibling is the same carrier reached one file over.
const ADMITTED_SIBLING_ATTRIBUTE_NAMES: [(&str, &[&str]); 4] = [
    ("invocation.rs", &["derive", "must_use"]),
    ("market.rs", &["derive", "must_use", "serde"]),
    ("lib.rs", &["cfg", "derive", "must_use", "serde", "test"]),
    (
        "session.rs",
        &["cfg", "derive", "must_use", "serde", "test"],
    ),
];

/// The only macro this file defines. Pinned because a definition rebinds every call site that
/// uses its name, while the invocation-name allowlist sees no change at all.
const ADMITTED_TEST_MACROS: [&str; 1] = ["assert_kind_enforces_grammar"];

/// The admitted helper's complete arm-matcher list: exactly one `($kind:ty)` arm. Rust reads the
/// first matching arm, so an earlier `($ignored:expr)` arm would intercept every call while the
/// real oracle below stays present but unread; a name-plus-one-line check cannot see that.
const HELPER_MACRO_NAME: &str = "assert_kind_enforces_grammar";
const HELPER_ARM_MATCHERS: [&str; 1] = ["($kind:ty)"];
/// The load-bearing checks the sole arm must still carry, so its body cannot be gutted to a
/// no-op while production is arbitrarily wrong.
const HELPER_BODY_CARRIERS: [&str; 4] = [
    "<$kind>::parse",
    "error.value_kind()",
    "error.kind()",
    "serde_json::from_str",
];

/// The complete `use`/`type`/`mod` item allowlist of this bound test file. A block-local
/// `use std::assert as assert_eq;` rebinds `assert_eq!` for its scope without a `macro_rules!`
/// or a changed invocation name, so only total item accounting sees it.
const ADMITTED_TEST_ITEMS: [&str; 9] = [
    "use std::any::TypeId;",
    "use std::collections::hash_map::DefaultHasher;",
    "use std::error::Error;",
    "use std::hash::{Hash, Hasher};",
    // The owned-string deserializer that reaches `visit_string`, which `from_str` never does.
    "use serde::Deserialize;",
    "use serde::de::IntoDeserializer;",
    "use serde::de::value::{BytesDeserializer, Error as SerdeValueError, StringDeserializer};",
    "use ustc_campus_agent_core::identity::{ CommandId, CorrelationId, IdentityValueError, \
     IdentityValueErrorKind, RequestId, SessionId, TenantId, UserId, };",
    "use ustc_campus_agent_core::invocation;",
];

/// Macros whose meaning this suite's evidence depends on. None may be redefined anywhere.
const RUST_SHADOWABLE_MACRO_NAMES: [&str; 17] = [
    "assert",
    "assert_eq",
    "assert_ne",
    "debug_assert",
    "debug_assert_eq",
    "debug_assert_ne",
    "matches",
    "panic",
    "unreachable",
    "write",
    "writeln",
    "format",
    "concat",
    "stringify",
    "include_str",
    "include_bytes",
    "vec",
];

/// The cross-language differential corpus, shared byte-for-byte with the repository checker.
const LEXICAL_CORPUS: &str = include_str!("../../../scripts/tests/data/rust_lexical_corpus.json");

/// Proves this file's lexer agrees with `scripts/check_repo_contracts.py` on every corpus case.
///
/// The same rules are implemented twice. Two implementations that are only claimed to agree
/// diverge silently; two implementations compared against one committed expectation set cannot.
/// The corpus is deliberately adversarial — comment-split keywords, byte-char literals, raw
/// identifiers, nested use trees, restricted visibility, non-ASCII identifiers — because every
/// one of those classes has produced a real divergence in this file's history.
fn assert_lexer_matches_the_shared_corpus() {
    let Ok(payload) = serde_json::from_str::<serde_json::Value>(LEXICAL_CORPUS) else {
        panic!("lexical corpus must be valid JSON");
    };
    let Some(cases) = payload.get("cases").and_then(serde_json::Value::as_array) else {
        panic!("lexical corpus must carry a cases array");
    };
    assert!(
        cases.len() >= 50,
        "lexical corpus collapsed: {}",
        cases.len()
    );

    let strings = |value: Option<&serde_json::Value>| -> Vec<String> {
        value
            .and_then(serde_json::Value::as_array)
            .map(|entries| {
                entries
                    .iter()
                    .filter_map(|entry| entry.as_str().map(str::to_owned))
                    .collect()
            })
            .unwrap_or_default()
    };

    for case in cases {
        let Some(source) = case.get("source").and_then(serde_json::Value::as_str) else {
            panic!("lexical corpus case is missing its source");
        };
        let stripped = strip_comments_and_literals(source);
        assert_eq!(
            Some(stripped.as_str()),
            case.get("stripped").and_then(serde_json::Value::as_str),
            "stripper diverged from the shared corpus on {source:?}"
        );
        // The literal-PRESERVING mode is a second lexer output, so it is compared too: the
        // grammar's semantics are read through it, and an unchecked mode is an unchecked carrier.
        assert_eq!(
            Some(strip_comments_only(source).as_str()),
            case.get("stripped_literals")
                .and_then(serde_json::Value::as_str),
            "literal-preserving stripper diverged from the shared corpus on {source:?}"
        );
        let (items, item_unterminated) = item_declarations(&stripped);
        assert_eq!(
            items,
            strings(case.get("items")),
            "item declarations diverged on {source:?}"
        );
        assert_eq!(
            item_unterminated,
            strings(case.get("item_unterminated")),
            "unterminated items diverged on {source:?}"
        );
        let declarations = impl_declarations(&stripped);
        let unclassified = declarations
            .iter()
            .any(|entry| entry.starts_with("UNCLASSIFIED-IMPL"));
        assert_eq!(
            declarations
                .iter()
                .filter(|entry| !entry.starts_with("UNCLASSIFIED-IMPL"))
                .cloned()
                .collect::<Vec<_>>(),
            strings(case.get("impls")),
            "impl declarations diverged on {source:?}"
        );
        assert_eq!(
            Some(unclassified),
            case.get("impl_unclassified")
                .and_then(serde_json::Value::as_bool),
            "impl classification diverged on {source:?}"
        );
        assert_eq!(
            macro_definitions(&stripped),
            strings(case.get("macro_definitions")),
            "macro definitions diverged on {source:?}"
        );
        let (invocations, mut unterminated_macros) = macro_invocation_arguments(&stripped);
        assert_eq!(
            invocations
                .iter()
                .map(|(name, argument)| format!("{name}!({argument})"))
                .collect::<Vec<_>>(),
            strings(case.get("macro_invocations")),
            "macro invocations diverged on {source:?}"
        );
        unterminated_macros.sort();
        assert_eq!(
            unterminated_macros,
            strings(case.get("macro_unterminated")),
            "unterminated macros diverged on {source:?}"
        );
        let arms = macro_arms(&stripped);
        let arms_json = serde_json::to_value(&arms).expect("serialize arms");
        assert_eq!(
            Some(&arms_json),
            case.get("macro_arms"),
            "macro arms diverged on {source:?}"
        );
        assert_eq!(
            derive_bodies(&stripped),
            strings(case.get("derives")),
            "derive bodies diverged on {source:?}"
        );
        let (found, unterminated_attributes) = attributes(&stripped);
        let attributes_json = serde_json::to_value(&found).expect("serialize attributes");
        assert_eq!(
            Some(&attributes_json),
            case.get("attributes"),
            "attributes diverged on {source:?}"
        );
        assert_eq!(
            unterminated_attributes,
            strings(case.get("attribute_unterminated")),
            "unterminated attributes diverged on {source:?}"
        );
        let (declared, unresolved) = functions(&stripped);
        let functions_json = serde_json::to_value(&declared).expect("serialize functions");
        assert_eq!(
            Some(&functions_json),
            case.get("functions"),
            "function bodies diverged on {source:?}"
        );
        assert_eq!(
            unresolved,
            strings(case.get("function_unresolved")),
            "unresolved functions diverged on {source:?}"
        );
        assert_eq!(
            newtype_constructions(&stripped),
            strings(case.get("constructions")),
            "newtype constructions diverged on {source:?}"
        );
        assert_eq!(
            string_literals(source),
            strings(case.get("string_literals")),
            "string literals diverged on {source:?}"
        );
    }
}

/// Proves the assertion macros this suite depends on are the real ones.
///
/// A test-local `macro_rules! assert_eq` leaves every admitted `assert_eq!` invocation NAME
/// unchanged while making the assertion type-check-only, so no name allowlist can see it. This
/// checks the property rather than the spelling: the macros must evaluate their arguments, and
/// they must still fail on a false claim. Its own failure path is the path-qualified
/// `::core::panic!`, which a local `macro_rules! panic` cannot shadow.
fn assert_assertion_macros_bite() {
    let mut evaluated = 0_u32;
    assert_eq!(
        {
            evaluated += 1;
            1_u8
        },
        1_u8
    );
    assert_ne!(
        {
            evaluated += 1;
            1_u8
        },
        2_u8
    );
    assert!({
        evaluated += 1;
        true
    });
    if evaluated != 3 {
        ::core::panic!("an assertion macro did not evaluate its arguments: {evaluated} of 3");
    }
    for (label, enforced) in [
        (
            "assert_eq!",
            std::panic::catch_unwind(|| {
                assert_eq!(std::hint::black_box(1_u8), std::hint::black_box(2_u8));
            })
            .is_err(),
        ),
        (
            "assert_ne!",
            std::panic::catch_unwind(|| {
                assert_ne!(std::hint::black_box(1_u8), std::hint::black_box(1_u8));
            })
            .is_err(),
        ),
        (
            "assert!",
            std::panic::catch_unwind(|| {
                assert!(std::hint::black_box(false));
            })
            .is_err(),
        ),
    ] {
        if !enforced {
            ::core::panic!("{label} did not enforce a false claim");
        }
    }
}

const BOUND_TEST_FUNCTIONS: [&str; 5] = [
    "identity_values_enforce_canonical_bounds_and_errors",
    "identity_values_are_exact_and_nominal",
    "identity_errors_never_echo_rejected_input",
    "identity_module_has_no_generation_or_adapter_surface",
    "market_invocation_authority_uses_m00_identity_definitions",
];

/// Returns the attributes attached to `fn <name>`, parsed bracket-balanced.
///
/// Line-based collection cannot see a multiline attribute: the closing `)]` of a wrapped
/// `#[cfg_attr(...)]` does not start with `#[`, so a reverse line scan stops early.
fn attribute_block(code: &str, name: &str) -> Vec<String> {
    let needle = format!("fn {name}(");
    let Some(at) = code.find(&needle) else {
        panic!("bound test not found: {name}");
    };
    let mut attributes = Vec::new();
    let mut cursor = at;
    loop {
        let head = code[..cursor].trim_end();
        if !head.ends_with(']') {
            break;
        }
        let bytes = head.as_bytes();
        let mut depth = 0;
        let mut index = head.len();
        while index > 0 {
            index -= 1;
            match bytes[index] {
                b']' => depth += 1,
                b'[' => {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                }
                _ => {}
            }
        }
        // `#` and `[` may be separated by whitespace (a comment strips to a space), so
        // `# [ignore]` is the same attribute as `#[ignore]`; anchoring on adjacency would let a
        // spaced attribute suppress a bound test unseen.
        let prefix = head[..index].trim_end();
        if index == 0 || !prefix.ends_with('#') {
            break;
        }
        let opener = prefix.len() - 1;
        attributes.push(
            head[opener..]
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" "),
        );
        cursor = opener;
    }
    attributes.reverse();
    attributes
}

/// Proves every bound acceptance test still executes.
///
/// A named test that is ignored or conditionally excluded keeps its registered binding at exit
/// zero while contributing no evidence, so this is asserted from inside the suite as well as by
/// the repository checker.
fn assert_bound_test_envelope_is_active() {
    let code = strip_comments_and_literals(TEST_SOURCE);
    for forbidden in ["cfg_attr", "should_panic"] {
        assert!(
            !code.contains(forbidden),
            "bound acceptance tests must execute unconditionally: {forbidden} is forbidden"
        );
    }
    // Disabling attributes are rejected by NORMALIZED NAME through the shared parser, so the
    // raw spelling `#[r#ignore]` and the spaced `# [ignore]` are one rule rather than a list of
    // substrings. The exhaustive name allowlist below is what closes the unpredicted cases.
    for (_, name, body) in attributes(&code).0 {
        assert!(
            !matches!(
                name.as_str(),
                "ignore" | "cfg" | "cfg_attr" | "should_panic"
            ),
            "bound acceptance tests must execute unconditionally: #[{body}] is forbidden"
        );
    }
    // An inner attribute is the token sequence `#` `!` `[`, not the string `#![`:
    // `# /*x*/ ! [cfg(any())]` excludes this whole test crate just as `#![cfg(any())]` does, so
    // both bound commands would report "running 0 tests" at exit 0 and this guard would not run.
    assert!(
        !contains_token_sequence(&code, &["#", "!", "["]),
        "bound acceptance tests must execute unconditionally: an inner attribute is forbidden"
    );
    // Invocation names alone do not fix what an invocation MEANS. A local
    // `macro_rules! assert_eq` rebinds every admitted `assert_eq!` call site in this file.
    assert_eq!(
        macro_definitions(&code),
        ADMITTED_TEST_MACROS
            .iter()
            .map(|name| (*name).to_owned())
            .collect::<Vec<_>>(),
        "macro definitions drifted in the bound acceptance test file"
    );
    // Rust reads the FIRST matching arm, so an earlier `($ignored:expr)` arm intercepts every
    // `helper!(TenantId)` call while the real `($kind:ty)` oracle stays present but unread. The
    // complete arm-matcher list is pinned, not the presence of one line.
    let helper_arms = macro_arms(&code)
        .into_iter()
        .filter(|(name, _)| name == HELPER_MACRO_NAME)
        .map(|(_, arms)| arms)
        .collect::<Vec<_>>();
    assert_eq!(
        helper_arms,
        vec![
            HELPER_ARM_MATCHERS
                .iter()
                .map(|matcher| (*matcher).to_owned())
                .collect::<Vec<_>>()
        ],
        "admitted test helper macro arms drifted"
    );
    // One arm with the exact matcher can still be gutted to a no-op, so the grammar oracle's
    // load-bearing checks are pinned in the sole arm's body.
    let helper_body = macro_body(&code, HELPER_MACRO_NAME).expect("helper macro body");
    for carrier in HELPER_BODY_CARRIERS {
        assert!(
            helper_body.contains(carrier),
            "admitted test helper macro lost a grammar-oracle carrier: {carrier}"
        );
    }
    // Total accounting over this file's attributes by normalized name: `#[r#ignore]` normalizes
    // to `ignore` and is drift even though it contains no `#[ignore]` substring.
    let (test_attributes, unterminated_test_attributes) = attributes(&code);
    assert!(
        unterminated_test_attributes.is_empty(),
        "unterminated attribute in the bound acceptance test file: {unterminated_test_attributes:?}"
    );
    assert_eq!(
        attribute_names(&test_attributes),
        ADMITTED_TEST_ATTRIBUTE_NAMES
            .iter()
            .map(|name| (*name).to_owned())
            .collect::<Vec<_>>(),
        "bound test attribute names drifted"
    );
    // Total accounting over this file's items, the same rule the governed sources carry. A
    // block-local `use std::assert as assert_eq;` — dropped after this guard has already run —
    // rebinds `assert_eq!` for the rest of its scope while the invocation-name set is unchanged.
    let (test_items, test_item_unterminated) = item_declarations(&code);
    assert!(
        test_item_unterminated.is_empty(),
        "unterminated item in the bound acceptance test file: {test_item_unterminated:?}"
    );
    assert_eq!(
        test_items,
        ADMITTED_TEST_ITEMS
            .iter()
            .map(|item| (*item).to_owned())
            .collect::<Vec<_>>(),
        "bound test item declarations drifted"
    );
    for name in BOUND_TEST_FUNCTIONS {
        assert_eq!(
            attribute_block(&code, name),
            ADMITTED_TEST_ATTRIBUTES.to_vec(),
            "bound acceptance test {name} attribute envelope drifted"
        );
    }
}

/// `AUTH-012`
#[test]
fn identity_values_are_exact_and_nominal() {
    // Before anything else: prove the macros the rest of this suite asserts through are the
    // real ones. Every assertion below is worthless if `assert_eq!` has been rebound.
    assert_assertion_macros_bite();
    // …and that this file's lexer still agrees with the checker's, case for case.
    assert_lexer_matches_the_shared_corpus();
    // Negative space first: an added unchecked constructor, mutable-backing accessor,
    // cross-kind conversion, deferred alias or extra derive fails here, not silently.
    assert_public_surface_is_frozen();
    assert_bound_test_envelope_is_active();

    let raw = "Tenant.Alpha_Beta:Gamma-01";
    let tenant = TenantId::parse(raw).expect("canonical value");

    // Exact string, Display and JSON projections agree byte for byte.
    assert_eq!(tenant.as_str(), raw);
    assert_eq!(tenant.to_string(), raw);
    assert_eq!(
        serde_json::to_string(&tenant).expect("serialize"),
        format!("\"{raw}\"")
    );
    let round_tripped: TenantId =
        serde_json::from_str(&serde_json::to_string(&tenant).expect("serialize"))
            .expect("deserialize");
    assert_eq!(round_tripped, tenant);
    assert_eq!(round_tripped.as_str(), raw);

    // Case and delimiters are significant, never folded or rewritten.
    let upper = TenantId::parse("ABC").expect("canonical value");
    let lower = TenantId::parse("abc").expect("canonical value");
    assert_ne!(upper, lower);
    let dotted = TenantId::parse("a.b").expect("canonical value");
    let coloned = TenantId::parse("a:b").expect("canonical value");
    let doubled = TenantId::parse("a..b").expect("canonical value");
    assert_ne!(dotted, coloned);
    assert_ne!(dotted, doubled);
    assert_eq!(doubled.as_str(), "a..b");

    // Ordering is exact-byte ordering: 'A'(0x41) < 'a'(0x61), '.'(0x2E) < ':'(0x3A).
    let mut ordered = [
        coloned.clone(),
        lower.clone(),
        upper.clone(),
        dotted.clone(),
    ];
    ordered.sort();
    assert_eq!(
        ordered.iter().map(TenantId::as_str).collect::<Vec<_>>(),
        ["ABC", "a.b", "a:b", "abc"]
    );

    // Equal values hash equally; different values are distinguished by exact bytes.
    assert_eq!(
        hash_of(&TenantId::parse("a.b").expect("canonical value")),
        hash_of(&dotted)
    );
    assert_ne!(hash_of(&dotted), hash_of(&coloned));
    assert_ne!(hash_of(&upper), hash_of(&lower));

    // The six kinds are nominally distinct types, not aliases of one another.
    let kinds = [
        ("TenantId", TypeId::of::<TenantId>()),
        ("UserId", TypeId::of::<UserId>()),
        ("SessionId", TypeId::of::<SessionId>()),
        ("RequestId", TypeId::of::<RequestId>()),
        ("CommandId", TypeId::of::<CommandId>()),
        ("CorrelationId", TypeId::of::<CorrelationId>()),
    ];
    for (left_index, (left_name, left)) in kinds.iter().enumerate() {
        for (right_name, right) in kinds.iter().skip(left_index + 1) {
            assert_ne!(left, right, "{left_name} must not alias {right_name}");
        }
    }

    // Every kind reports its own value kind in construction errors.
    for (kind_name, error) in [
        (
            "TenantId",
            TenantId::parse("-").err().map(|e| e.value_kind()),
        ),
        ("UserId", UserId::parse("-").err().map(|e| e.value_kind())),
        (
            "SessionId",
            SessionId::parse("-").err().map(|e| e.value_kind()),
        ),
        (
            "RequestId",
            RequestId::parse("-").err().map(|e| e.value_kind()),
        ),
        (
            "CommandId",
            CommandId::parse("-").err().map(|e| e.value_kind()),
        ),
        (
            "CorrelationId",
            CorrelationId::parse("-").err().map(|e| e.value_kind()),
        ),
    ] {
        assert_eq!(error, Some(kind_name));
    }
}

/// Synthetic, obviously fake sentinels. No real credential or personal data appears here.
///
/// Each offending byte is `!` or `%` so its absence from any rendering is directly assertable.
fn rejected_sentinels() -> Vec<(String, IdentityValueErrorKind)> {
    vec![
        (
            "synthetic!sentinel".to_owned(),
            IdentityValueErrorKind::InvalidCharacter { byte_index: 9 },
        ),
        (
            "%leading-sentinel".to_owned(),
            IdentityValueErrorKind::InvalidStart,
        ),
        (
            "trailing-sentinel%".to_owned(),
            IdentityValueErrorKind::InvalidEnd,
        ),
        (String::new(), IdentityValueErrorKind::Empty),
        (
            format!("z{}", "y".repeat(MAX_BYTES)),
            IdentityValueErrorKind::TooLong {
                max_bytes: MAX_BYTES,
            },
        ),
    ]
}

/// Tokens of the rejected input that must never surface in a diagnostic.
fn input_fragments(value: &str) -> Vec<String> {
    value
        .split(|byte: char| !byte.is_ascii_alphanumeric())
        .filter(|token| token.len() >= 3)
        .map(str::to_owned)
        .collect()
}

fn assert_no_echo(label: &str, rendered: &str, value: &str) {
    if !value.is_empty() {
        assert!(
            !rendered.contains(value),
            "{label} echoed the complete rejected input"
        );
    }
    for fragment in input_fragments(value) {
        assert!(
            !rendered.contains(&fragment),
            "{label} echoed an input-derived fragment"
        );
    }
    for offending in ['!', '%'] {
        assert!(
            !rendered.contains(offending),
            "{label} rendered the offending byte"
        );
    }
}

/// `AUTH-014`
#[test]
fn identity_errors_never_echo_rejected_input() {
    for (value, expected) in rejected_sentinels() {
        let Err(error) = TenantId::parse(value.clone()) else {
            panic!("synthetic sentinel must be rejected");
        };

        // Value-kind and failure metadata remain available to the caller.
        assert_eq!(error.value_kind(), "TenantId");
        assert_eq!(error.kind(), expected);
        match error.kind() {
            IdentityValueErrorKind::TooLong { max_bytes } => assert_eq!(max_bytes, MAX_BYTES),
            IdentityValueErrorKind::InvalidCharacter { byte_index } => {
                assert_eq!(byte_index, 9);
                assert_eq!(value.as_bytes()[byte_index], b'!');
            }
            IdentityValueErrorKind::Empty
            | IdentityValueErrorKind::InvalidStart
            | IdentityValueErrorKind::InvalidEnd => {}
        }

        assert_no_echo("Display", &error.to_string(), &value);
        assert_no_echo("Debug", &format!("{error:?}"), &value);

        // No retained source or input payload anywhere in the error chain.
        let as_error: &dyn Error = &error;
        assert!(as_error.source().is_none());

        // A Serde rejection raised after a string was successfully decoded is equally silent.
        let encoded = serde_json::to_string(&value).expect("serialize");
        let Err(serde_error) = serde_json::from_str::<TenantId>(&encoded) else {
            panic!("Serde must reject a non-canonical string");
        };
        assert_no_echo("Serde", &serde_error.to_string(), &value);
    }

    // Every kind is equally silent, not only the representative one.
    let secret_shaped = "synthetic!sentinel";
    let rejected = "must be rejected";
    for rendered in [
        UserId::parse(secret_shaped)
            .expect_err(rejected)
            .to_string(),
        SessionId::parse(secret_shaped)
            .expect_err(rejected)
            .to_string(),
        RequestId::parse(secret_shaped)
            .expect_err(rejected)
            .to_string(),
        CommandId::parse(secret_shaped)
            .expect_err(rejected)
            .to_string(),
        CorrelationId::parse(secret_shaped)
            .expect_err(rejected)
            .to_string(),
    ] {
        assert_no_echo("Display", &rendered, secret_shaped);
    }
}

/// The accepted contract, read as data so the grammar's authority is a document outside the
/// mutable evidence set rather than agreement among carriers.
const IDENTITY_CONTRACT: &str = include_str!("../../../docs/contracts/platform-identity.md");

/// This carrier's copy of the `platform-identity/v0` semantic table.
///
/// It is NOT the authority. Every field below is cross-checked against the contract's single
/// normative regex — parsed structurally, not searched for — and against its individually
/// anchored normative-consequence lines, then used to check production and the oracle. Editing
/// this table alone fails; editing the contract too is a `v0` change under §9.
const GRAMMAR_REGEX: &str = "^[A-Za-z0-9](?:[-A-Za-z0-9._:]{0,126}[A-Za-z0-9])?$";
const GRAMMAR_MAX_BYTES: usize = 128;
const GRAMMAR_INTERIOR_EXTRA: &str = "-._:";
const GRAMMAR_BOUNDARY_PREDICATE: &str = "byte.is_ascii_alphanumeric()";
const GRAMMAR_NORMATIVE_LINES: [(usize, &str); 4] = [
    (1, "encoded length is `1..=128` bytes;"),
    (2, "the first and last byte are ASCII alphanumeric;"),
    (5, "case is significant;"),
    (
        6,
        "no trimming, Unicode normalization, case folding, delimiter rewriting or alternate \
         spelling occurs;",
    ),
];

/// The deciding carriers of the length bound's EFFECTIVE semantics.
///
/// A declared constant and a frozen body table are both mutable, and a body may legally introduce a
/// second semantic constant. So these name the positions the contract-bound value must occupy, and
/// the accounting below eliminates every place a second bound could come from.
const BOUND_FUNCTION: &str = "classify";
const BOUND_CONSTANT: &str = "MAX_IDENTITY_BYTES";
const BOUND_SUBJECT: &str = "bytes";
const BOUND_SUBJECT_BINDING: &str = "let bytes = value.as_bytes();";
const BOUND_OPERATOR: &str = ">";
const BOUND_FIELD: &str = "max_bytes";
/// Source order: the enum variant's own field type, then the single constructed value.
const BOUND_FIELD_VALUES: [&str; 2] = ["usize", BOUND_CONSTANT];
/// `byte_index: offset + 1` — the only number the deciding function may spell.
const BOUND_ADMITTED_LITERALS: [&str; 1] = ["1"];
const BOUND_FORBIDDEN_ITEM_KEYWORDS: [&str; 13] = [
    "const",
    "enum",
    "extern",
    "fn",
    "impl",
    "macro_rules",
    "mod",
    "static",
    "struct",
    "trait",
    "type",
    "union",
    "use",
];

/// The error names the rejection branch must construct, per the accepted contract's §2 table.
const BOUND_ERROR_TYPE: &str = "IdentityValueErrorKind";
const BOUND_ERROR_VARIANT: &str = "TooLong";
/// `TooLong` may be spelled only where the contract puts it: the variant, its rendering, the branch.
const BOUND_ERROR_VARIANT_SITES: usize = 3;

/// The generic corpus macro, whose pinned carriers are all substrings.
///
/// A substring is not a case that still reaches it. The runtime proof's own body is bound by the
/// always-run Python checker instead, which is the carrier a deletion inside this file cannot move.
const CORPUS_MACRO: &str = "assert_kind_enforces_grammar";
/// A skip is how every required substring survives while the rows that matter stop executing.
///
/// `?` belongs here for the same reason `continue` does. A helper returning `Result`, a caller
/// writing `let _ = helper();` and one `black_box(Err::<(),()>(()))?` leave before the proof runs
/// while spelling neither `continue` nor `return`; `break` ends a corpus loop just as quietly.
const CORPUS_MACRO_FORBIDDEN_CONTROL: [&str; 4] = ["?", "break", "continue", "return"];
/// The seeds and span of the runtime length sweep.
///
/// The 128/129 pair proves the boundary, and nothing more: an early accept keyed to some OTHER
/// over-bound length — reviewer Task 1's 200-byte literal — walks straight past it. Sweeping every
/// length to twice the bound, under two different canonical bytes, is cheap and closes that
/// behaviourally rather than only structurally.
const RUNTIME_PROOF_SEEDS: [&str; 2] = ["a", "p"];
const RUNTIME_PROOF_SWEEP: usize = 2 * GRAMMAR_MAX_BYTES;

/// The helper names whose CALL must reach the file-level function it spells.
///
/// Rust resolves a call lexically, so an item declared in the caller's OWN body binds the name
/// ahead of the module's. Round 18 proved each of these is called as a plain statement, which is a
/// statement about tokens; `fn r#assert_no_length_past_the_bound_is_accepted() {}` beside
/// `let _ = crate::assert_no_length_past_the_bound_is_accepted as fn();` keeps the real helper used
/// — so no unused-item lint fires — leaves every token those rules match exactly where they are,
/// and sends the call to a no-op. A raw identifier is the same name to Rust and a different string
/// to every textual rule, which is why the accounting below normalizes `r#` away first.
const LOAD_BEARING_HELPERS: [&str; 11] = [
    "assert_assertion_macros_bite",
    "assert_bound_test_envelope_is_active",
    "assert_classify_is_the_contract_decision_procedure",
    "assert_contract_bound_is_the_effective_runtime_limit",
    "assert_corpus_macro_cannot_skip_a_row",
    "assert_effective_max_byte_bound_is_contract_bound",
    "assert_grammar_is_exhaustive_over_bytes",
    "assert_grammar_semantics_match_the_contract",
    "assert_load_bearing_calls_reach_their_helper",
    "assert_no_length_past_the_bound_is_accepted",
    "assert_sweep_carriers_are_the_contract_extent",
];

/// The two bodies that call them, and so the only two scopes a shadow could be introduced in.
///
/// Every other load-bearing body is bound token for token by the always-run Python checker, and a
/// declaration cannot be added to a body whose whole token sequence is fixed.
const SHADOWABLE_CALLERS: [&str; 2] = [
    "identity_values_enforce_canonical_bounds_and_errors",
    "assert_effective_max_byte_bound_is_contract_bound",
];

/// Returns the one fenced ```regex carrier of the contract, or `None` if there is not exactly one.
fn contract_regex_carrier(contract: &str) -> Option<String> {
    let mut found: Vec<String> = Vec::new();
    let mut rest = contract;
    while let Some(at) = rest.find("```regex\n") {
        let body = &rest[at + "```regex\n".len()..];
        let end = body.find("\n```")?;
        found.push(body[..end].trim().to_owned());
        rest = &body[end..];
    }
    if found.len() == 1 { found.pop() } else { None }
}

/// Parses the frozen grammar shape into `(boundary class, interior class, repetition bound)`.
///
/// A structural parse, not a substring comparison: a class that gained a byte or a bound that
/// moved is a different parse rather than a string that still contains the old one.
fn parse_grammar_shape(regex: &str) -> Option<(String, String, usize)> {
    let rest = regex.strip_prefix("^[")?;
    let (lead, rest) = rest.split_once(']')?;
    let rest = rest.strip_prefix("(?:[")?;
    let (interior, rest) = rest.split_once(']')?;
    let rest = rest.strip_prefix("{0,")?;
    let (bound, rest) = rest.split_once('}')?;
    let rest = rest.strip_prefix('[')?;
    let (tail, rest) = rest.split_once(']')?;
    if rest != ")?$" || lead != tail {
        return None;
    }
    Some((lead.to_owned(), interior.to_owned(), bound.parse().ok()?))
}

/// Expands a regex character-class body into its ordered members, reporting a repeated byte.
fn expand_character_class(spec: &str) -> (Vec<char>, bool) {
    let characters: Vec<char> = spec.chars().collect();
    let mut members = Vec::new();
    let mut index = 0;
    while index < characters.len() {
        let is_range = index + 2 < characters.len()
            && characters[index + 1] == '-'
            && index + 2 != characters.len() - 1 + 1;
        if is_range {
            for code in (characters[index] as u32)..=(characters[index + 2] as u32) {
                if let Some(character) = char::from_u32(code) {
                    members.push(character);
                }
            }
            index += 3;
            continue;
        }
        members.push(characters[index]);
        index += 1;
    }
    let mut unique = members.clone();
    unique.sort_unstable();
    let before = unique.len();
    unique.dedup();
    (members, unique.len() != before)
}

fn sorted_chars(text: &str) -> Vec<char> {
    let mut characters: Vec<char> = text.chars().collect();
    characters.sort_unstable();
    characters
}

/// Binds grammar SEMANTICS to the accepted contract instead of to agreement among carriers.
///
/// Every function body in this module is pinned exactly, but over code with literal payloads
/// stripped — which pins control flow and deliberately not the bytes inside a literal. Production,
/// this oracle, both corpora, the JSON fixtures and their digests could therefore be moved from
/// `:` to `?` together and every mechanical gate stayed green while `a?b` was accepted. So the
/// literal semantics are read from comment-stripped, literal-PRESERVING source and checked
/// against the contract document, which no coordinated edit of the evidence can move.
fn assert_grammar_semantics_match_the_contract() {
    let Some(carrier) = contract_regex_carrier(IDENTITY_CONTRACT) else {
        panic!("platform identity contract must carry exactly one normative regex carrier");
    };
    assert_eq!(
        carrier, GRAMMAR_REGEX,
        "platform identity grammar-contract mismatch: contract regex"
    );
    let Some((lead, interior, bound)) = parse_grammar_shape(&carrier) else {
        panic!("platform identity contract regex is not the frozen shape: {carrier}");
    };
    let (boundary_members, boundary_repeat) = expand_character_class(&lead);
    let (interior_members, interior_repeat) = expand_character_class(&interior);
    assert!(
        !boundary_repeat && !interior_repeat,
        "platform identity contract regex repeats a character-class byte"
    );
    let ascii_alphanumeric: Vec<char> = (0_u8..128)
        .map(char::from)
        .filter(char::is_ascii_alphanumeric)
        .collect();
    assert_eq!(
        sorted_chars(&boundary_members.iter().collect::<String>()),
        ascii_alphanumeric,
        "platform identity grammar-contract mismatch: boundary class"
    );
    let extras: Vec<char> = {
        let mut found: Vec<char> = interior_members
            .iter()
            .copied()
            .filter(|character| !character.is_ascii_alphanumeric())
            .collect();
        found.sort_unstable();
        found
    };
    assert_eq!(
        extras,
        sorted_chars(GRAMMAR_INTERIOR_EXTRA),
        "platform identity grammar-contract mismatch: interior delimiter set"
    );
    assert_eq!(
        bound + 2,
        GRAMMAR_MAX_BYTES,
        "platform identity grammar-contract mismatch: contract max bytes"
    );

    // Each remaining field is bound to its own anchored normative line, by list position and
    // exact text — never to a substring found anywhere in the document.
    let section = {
        let start = IDENTITY_CONTRACT
            .find("## 3. Shared identifier grammar")
            .expect("contract grammar section");
        let end = IDENTITY_CONTRACT[start..]
            .find("## 4. ")
            .expect("contract section 4");
        &IDENTITY_CONTRACT[start..start + end]
    };
    let numbered = |position: usize| -> Option<&str> {
        let prefix = format!("{position}. ");
        section
            .lines()
            .find_map(|line| line.strip_prefix(prefix.as_str()))
            .map(str::trim)
    };
    for (position, text) in GRAMMAR_NORMATIVE_LINES {
        assert_eq!(
            numbered(position),
            Some(text),
            "platform identity grammar-contract mismatch: normative line {position}"
        );
    }
    let interior_line = numbered(3).unwrap_or_default();
    assert!(
        interior_line.starts_with("interior bytes are ASCII alphanumeric or one of "),
        "platform identity grammar-contract mismatch: interior line {interior_line}"
    );
    let quoted: Vec<char> = {
        let bytes: Vec<char> = interior_line.chars().collect();
        let mut found = Vec::new();
        for window in bytes.windows(3) {
            if window[0] == '`' && window[2] == '`' {
                found.push(window[1]);
            }
        }
        found.sort_unstable();
        found.dedup();
        found
    };
    assert_eq!(
        quoted,
        sorted_chars(GRAMMAR_INTERIOR_EXTRA),
        "platform identity grammar-contract mismatch: interior normative line"
    );

    // Production and oracle literals, read with payloads PRESERVED.
    let source = strip_comments_only(IDENTITY_SOURCE);
    let test_source = strip_comments_only(TEST_SOURCE);
    let declaration = source
        .split("const MAX_IDENTITY_BYTES: usize =")
        .nth(1)
        .and_then(|rest| rest.split(';').next())
        .map(str::trim)
        .and_then(|value| value.parse::<usize>().ok());
    assert_eq!(
        declaration,
        Some(GRAMMAR_MAX_BYTES),
        "platform identity grammar-contract mismatch: production max bytes"
    );
    let Some(boundary_body) = function_body(&source, "is_boundary_byte") else {
        panic!("platform identity boundary carrier unreadable");
    };
    assert_eq!(
        boundary_body
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" "),
        format!("{{ {GRAMMAR_BOUNDARY_PREDICATE} }}"),
        "platform identity grammar-contract mismatch: production boundary class"
    );
    let Some(interior_body) = function_body(&source, "is_interior_byte") else {
        panic!("platform identity interior carrier unreadable");
    };
    assert_eq!(
        byte_literals(interior_body),
        sorted_chars(GRAMMAR_INTERIOR_EXTRA),
        "platform identity grammar-contract mismatch: production interior delimiters"
    );
    let restatements: Vec<&str> = IDENTITY_SOURCE
        .lines()
        .filter_map(|line| line.trim().strip_prefix("/// `"))
        .filter_map(|rest| rest.split('`').next())
        .filter(|candidate| candidate.starts_with("^[") && candidate.ends_with('$'))
        .collect();
    assert_eq!(
        restatements,
        vec![GRAMMAR_REGEX],
        "platform identity grammar-contract mismatch: production regex restatement"
    );
    let Some(oracle) = function_body(&test_source, "assert_grammar_is_exhaustive_over_bytes")
    else {
        panic!("platform identity exhaustive grammar oracle unreadable");
    };
    let tables: Vec<&str> = oracle
        .match_indices("*b\"")
        .filter_map(|(at, _)| oracle[at + 3..].split('"').next())
        .collect();
    assert_eq!(
        tables.len(),
        1,
        "platform identity exhaustive grammar oracle must carry exactly one delimiter table"
    );
    assert_eq!(
        sorted_chars(tables[0]),
        sorted_chars(GRAMMAR_INTERIOR_EXTRA),
        "platform identity grammar-contract mismatch: oracle interior delimiters"
    );
    assert_eq!(
        tables[0].chars().count(),
        GRAMMAR_INTERIOR_EXTRA.chars().count(),
        "platform identity grammar-contract mismatch: oracle delimiter multiplicity"
    );
    assert!(
        oracle.contains(GRAMMAR_BOUNDARY_PREDICATE),
        "platform identity exhaustive grammar oracle lost its boundary predicate"
    );
    let Some(corpus) = function_body(&test_source, "valid_values") else {
        panic!("platform identity valid corpus unreadable");
    };
    // Looked for inside the test VALUES, not anywhere in the body: a body containing
    // `String::from` contains `:` whatever its corpus says, and the first version of this check
    // passed for exactly that reason while the corpus had been drifted off the contract.
    let values = string_literals(corpus).concat();
    for delimiter in GRAMMAR_INTERIOR_EXTRA.chars() {
        assert!(
            values.contains(delimiter),
            "platform identity valid corpus does not exercise contract delimiter {delimiter}"
        );
    }

    // The runtime half, which no static carrier can stand in for: each byte the CONTRACT admits
    // must actually be accepted in interior position, and each one it does not must be rejected.
    for delimiter in GRAMMAR_INTERIOR_EXTRA.chars() {
        assert!(
            TenantId::parse(format!("a{delimiter}b")).is_ok(),
            "contract delimiter {delimiter} is rejected by the implementation"
        );
    }
    for byte in 0_u8..128 {
        let candidate = char::from(byte);
        if candidate.is_ascii_alphanumeric() || GRAMMAR_INTERIOR_EXTRA.contains(candidate) {
            continue;
        }
        assert!(
            TenantId::parse(format!("a{candidate}b")).is_err(),
            "byte {byte} is admitted in the interior but the contract does not name it"
        );
    }
}

/// Splits `code` into identifier/number runs, two-byte comparison operators and single characters.
///
/// A token sequence, not a substring search: `bytes.len()>MAX_IDENTITY_BYTES` and
/// `bytes . len ( ) > MAX_IDENTITY_BYTES` are the same sequence, and `max_bytes` is one token
/// rather than an occurrence of `bytes`.
fn rust_tokens(code: &str) -> Vec<&str> {
    let bytes = code.as_bytes();
    let mut found = Vec::new();
    let mut index = 0;
    while index < bytes.len() {
        let byte = bytes[index];
        if byte.is_ascii_alphanumeric() || byte == b'_' {
            let start = index;
            while index < bytes.len()
                && (bytes[index].is_ascii_alphanumeric() || bytes[index] == b'_')
            {
                index += 1;
            }
            found.push(&code[start..index]);
            continue;
        }
        if byte.is_ascii_whitespace() {
            index += 1;
            continue;
        }
        if matches!(byte, b'<' | b'>' | b'=' | b'!')
            && index + 1 < bytes.len()
            && bytes[index + 1] == b'='
        {
            found.push(&code[index..index + 2]);
            index += 2;
            continue;
        }
        let width = code[index..].chars().next().map_or(1, char::len_utf8);
        found.push(&code[index..index + width]);
        index += width;
    }
    found
}

fn is_rust_identifier(token: &str) -> bool {
    token.starts_with(|character: char| character.is_ascii_alphabetic() || character == '_')
}

/// Returns `receiver op operand` for every `<receiver>.len() <op> <operand>` of `tokens`.
fn length_comparisons(tokens: &[&str]) -> Vec<String> {
    let mut found = Vec::new();
    for window in tokens.windows(7) {
        if is_rust_identifier(window[0])
            && window[1] == "."
            && window[2] == "len"
            && window[3] == "("
            && window[4] == ")"
            && matches!(window[5], "<" | ">" | "<=" | ">=" | "==" | "!=")
        {
            found.push(format!("{} {} {}", window[0], window[5], window[6]));
        }
    }
    found
}

/// Counts every `.len()` call, so a measurement that is compared in some other shape still shows.
fn length_measurements(tokens: &[&str]) -> usize {
    tokens
        .windows(4)
        .filter(|window| {
            window[0] == "." && window[1] == "len" && window[2] == "(" && window[3] == ")"
        })
        .count()
}

/// Returns the operand of every `<field>: <operand>` of `tokens`, in source order.
fn field_values(tokens: &[&str], field: &str) -> Vec<String> {
    let mut found = Vec::new();
    for window in tokens.windows(3) {
        if window[0] == field && window[1] == ":" {
            found.push(window[2].to_owned());
        }
    }
    found
}

fn integer_literals(tokens: &[&str]) -> Vec<String> {
    tokens
        .iter()
        .filter(|token| token.starts_with(|character: char| character.is_ascii_digit()))
        .map(|token| (*token).to_owned())
        .collect()
}

/// Returns each `let`/`for` pattern of `tokens`: the tokens between the keyword and its `=`/`in`.
fn binding_patterns(tokens: &[&str]) -> Vec<String> {
    let mut found = Vec::new();
    let mut index = 0;
    while index < tokens.len() {
        if matches!(tokens[index], "let" | "for") {
            let mut end = index + 1;
            while end < tokens.len() && !matches!(tokens[end], "=" | "in" | ";" | "{" | "}") {
                end += 1;
            }
            found.push(tokens[index + 1..end].join(" "));
            index = end;
            continue;
        }
        index += 1;
    }
    found
}

fn pattern_binds(pattern: &str, name: &str) -> bool {
    pattern.split(' ').any(|token| token == name)
}

/// Returns every forbidden item keyword `tokens` contains, sorted.
fn declared_items(tokens: &[&str]) -> Vec<String> {
    let mut found: Vec<String> = BOUND_FORBIDDEN_ITEM_KEYWORDS
        .iter()
        .filter(|keyword| tokens.contains(*keyword))
        .map(|keyword| (*keyword).to_owned())
        .collect();
    found.sort_unstable();
    found
}

/// Returns the value of every `const <name>: usize = <digits>;` declared at brace depth zero.
///
/// Depth matters: a declaration nested in a function body, a `mod` or an `impl` is a different
/// binding from the module's own, and only the module's own is what every use resolves to. A value
/// that is not plain digits does not match at all, so `= 128 + 1` fails closed rather than reading
/// as 128.
fn module_level_usize_constants(code: &str, name: &str) -> Vec<usize> {
    let tokens = rust_tokens(code);
    let mut found = Vec::new();
    let mut depth = 0_i64;
    for index in 0..tokens.len() {
        match tokens[index] {
            "{" | "(" | "[" => depth += 1,
            "}" | ")" | "]" => depth -= 1,
            "const" if depth == 0 => {
                if let Some(window) = tokens.get(index..index + 7)
                    && window[1] == name
                    && window[2] == ":"
                    && window[3] == "usize"
                    && window[4] == "="
                    && window[6] == ";"
                    && let Ok(value) = window[5].parse::<usize>()
                {
                    found.push(value);
                }
            }
            _ => {}
        }
    }
    found
}

/// Returns the brace/paren/bracket depth *before* each token of `tokens`.
///
/// Depth is what separates a statement of the function from a statement of some block nested in it,
/// and a guard that decides from a guard that some other condition decides for.
fn token_depths(tokens: &[&str]) -> Vec<i64> {
    let mut depths = Vec::with_capacity(tokens.len());
    let mut depth = 0_i64;
    for token in tokens {
        match *token {
            "}" | ")" | "]" => depth -= 1,
            _ => {}
        }
        depths.push(depth);
        match *token {
            "{" | "(" | "[" => depth += 1,
            _ => {}
        }
    }
    depths
}

/// Drops a `,` that only separates a last element from its closing delimiter.
///
/// A trailing comma is rustfmt's business, not the language's: normalizing it keeps the admitted
/// shape below a statement about STRUCTURE rather than about a formatting policy that may change.
fn without_trailing_commas<'a>(tokens: &[&'a str]) -> Vec<&'a str> {
    let mut kept = Vec::with_capacity(tokens.len());
    for (index, token) in tokens.iter().enumerate() {
        if *token == "," && matches!(tokens.get(index + 1), Some(&("}" | ")" | "]"))) {
            continue;
        }
        kept.push(*token);
    }
    kept
}

/// Rewrites every raw identifier `r # <name>` to the plain `<name>` it denotes.
///
/// `r#foo` and `foo` are ONE name to Rust and two strings to every rule that compares tokens, so a
/// raw spelling defines a shadow no exact-token scan for the plain one can see. Both lexers split
/// it into three tokens; folding them here is what lets a single rule answer for both spellings.
fn without_raw_identifiers<'a>(tokens: &[&'a str]) -> Vec<&'a str> {
    let mut kept = Vec::with_capacity(tokens.len());
    let mut index = 0;
    while index < tokens.len() {
        if tokens[index] == "r"
            && tokens.get(index + 1) == Some(&"#")
            && tokens
                .get(index + 2)
                .is_some_and(|token| is_rust_identifier(token))
        {
            kept.push(tokens[index + 2]);
            index += 3;
            continue;
        }
        kept.push(tokens[index]);
        index += 1;
    }
    kept
}

/// The one admitted max-byte rejection statement, spelled from the CONTRACT's own names.
///
/// This is assembled here rather than copied out of `classify`, so it says what the contract
/// requires instead of agreeing with whatever the implementation currently happens to be.
fn admitted_bound_statement() -> Vec<&'static str> {
    vec![
        // if bytes.len() > MAX_IDENTITY_BYTES {
        "if",
        BOUND_SUBJECT,
        ".",
        "len",
        "(",
        ")",
        BOUND_OPERATOR,
        BOUND_CONSTANT,
        "{",
        // return Err(IdentityValueErrorKind::TooLong { max_bytes: MAX_IDENTITY_BYTES });
        "return",
        "Err",
        "(",
        BOUND_ERROR_TYPE,
        ":",
        ":",
        BOUND_ERROR_VARIANT,
        "{",
        BOUND_FIELD,
        ":",
        BOUND_CONSTANT,
        "}",
        ")",
        ";",
        "}",
    ]
}

/// Returns the start index of every occurrence of `needle` in `haystack` at depth `depth`.
fn statement_positions(
    haystack: &[&str],
    depths: &[i64],
    needle: &[&str],
    depth: i64,
) -> Vec<usize> {
    (0..haystack.len())
        .filter(|index| depths[*index] == depth)
        .filter(|index| haystack.get(*index..*index + needle.len()) == Some(needle))
        .collect()
}

/// Proves the max-byte comparison is the whole DECIDING condition, not merely a token that occurs.
///
/// Round 16 proved the comparison tuple `bytes.len() > MAX_IDENTITY_BYTES` occurs in `classify`. It
/// did not prove the comparison is what the rejection branch turns on, so a wrapper that keeps every
/// declared carrier alive while making the branch unreachable —
/// `if std::hint::black_box(false) && bytes.len() > MAX_IDENTITY_BYTES { … }` — passed the whole gate
/// chain with both body fingerprints co-mutated, and an external caller then parsed a 200-byte ID.
///
/// So the admitted statement above is bound as ONE structural unit: the exact condition between `if`
/// and its brace, the exact immediate rejection branch, at the function's own statement depth, with
/// no alternate branch. A prefix, suffix or wrapper predicate is part of the max-byte closure, not an
/// unrelated branch, and is rejected here rather than left to the body fingerprint.
fn assert_max_byte_guard_is_the_deciding_condition(body_tokens: &[&str], module_tokens: &[&str]) {
    let body = without_trailing_commas(body_tokens);
    let depths = token_depths(&body);
    let admitted = admitted_bound_statement();

    // The statement occurs exactly once, as a statement OF the deciding function: depth 1 is
    // `classify`'s own body, so a copy nested inside `if false { … }` is a different depth and does
    // not answer for it.
    let positions = statement_positions(&body, &depths, &admitted, 1);
    assert_eq!(
        positions.len(),
        1,
        "platform identity effective max-byte bound: {BOUND_FUNCTION} must contain exactly one \
         top-level {admitted:?}, found {}",
        positions.len()
    );

    // No `else`: the rejection branch is the whole decision, with no alternate outcome.
    let after = positions[0] + admitted.len();
    assert_ne!(
        body.get(after),
        Some(&"else"),
        "platform identity effective max-byte bound: {BOUND_FUNCTION} guard has an alternate branch"
    );

    // The variant is constructed nowhere else, so a second rejection path cannot report a second
    // bound while this one is disabled.
    let variants = module_tokens
        .iter()
        .filter(|token| **token == BOUND_ERROR_VARIANT)
        .count();
    assert_eq!(
        variants, BOUND_ERROR_VARIANT_SITES,
        "platform identity effective max-byte bound: {BOUND_ERROR_VARIANT} is spelled {variants} \
         times in the module, expected the variant, its rendering and the one rejection branch"
    );
}

/// Proves the generic corpus macro cannot silently stop exercising a row.
///
/// Every carrier the checker pins inside this macro is a SUBSTRING, and a substring is not a case
/// that still reaches it: a `continue` guarded by `matches!(expected, …::TooLong { .. })` keeps all
/// of them while the over-length rows stop running. The dedicated runtime proof below is the
/// authority for the bound; this keeps the broad corpus honest rather than load-bearing.
fn assert_corpus_macro_cannot_skip_a_row() {
    let code = strip_comments_and_literals(TEST_SOURCE);
    let needle = format!("macro_rules! {CORPUS_MACRO}");
    let Some(at) = code.find(&needle) else {
        panic!("platform identity runtime bound: {CORPUS_MACRO} is missing");
    };
    let rest = &code[at + needle.len()..];
    let Some(open) = rest.find('{') else {
        panic!("platform identity runtime bound: {CORPUS_MACRO} body unreadable");
    };
    let Some(body) = balanced_group(&rest[open..], '{', '}') else {
        panic!("platform identity runtime bound: {CORPUS_MACRO} body unbalanced");
    };
    let tokens = rust_tokens(body);
    for forbidden in CORPUS_MACRO_FORBIDDEN_CONTROL {
        assert!(
            !tokens.contains(&forbidden),
            "platform identity runtime bound: {CORPUS_MACRO} may not {forbidden} past a row"
        );
    }
}

/// The whole admitted body of the deciding function, as the contract's decision procedure.
///
/// §5 fixes the error precedence exactly and §3 fixes what each step tests, so the deciding
/// function has one admitted shape and this assembles it from those names.
fn admitted_classify_body() -> Vec<String> {
    let subject = BOUND_SUBJECT;
    let error = BOUND_ERROR_TYPE;
    let guard = admitted_bound_statement().join(" ");
    let steps = [
        "{".to_owned(),
        format!("let {subject} = value . as_bytes ( ) ;"),
        // §5.1 empty
        format!(
            "let Some ( ( & first , after_first ) ) = {subject} . split_first ( ) \
             else {{ return Err ( {error} : : Empty ) ; }} ;"
        ),
        // §5.2 too long — the guard bound in full above
        guard,
        // §5.3 invalid start
        format!("if ! is_boundary_byte ( first ) {{ return Err ( {error} : : InvalidStart ) ; }}"),
        // a one-byte value is decided by the first-byte rule alone
        "let Some ( ( & last , interior ) ) = after_first . split_last ( ) \
         else { return Ok ( ( ) ) ; } ;"
            .to_owned(),
        // §5.4 invalid interior byte, reported by index
        format!(
            "for ( offset , & byte ) in interior . iter ( ) . enumerate ( ) \
             {{ if ! is_interior_byte ( byte ) {{ return Err ( {error} : : InvalidCharacter \
             {{ byte_index : offset + 1 }} ) ; }} }}"
        ),
        // §5.5 invalid end
        format!("if ! is_boundary_byte ( last ) {{ return Err ( {error} : : InvalidEnd ) ; }}"),
        // otherwise canonical
        "Ok ( ( ) ) }".to_owned(),
    ];
    steps
        .join(" ")
        .split_whitespace()
        .map(str::to_owned)
        .collect()
}

/// Proves the deciding function is the contract's decision procedure and nothing else.
///
/// Binding the max-byte guard was not enough. An early accept keyed to a literal —
/// `if value == "aaa…129" { return Ok(()); }` — adds a step BEFORE the guard while leaving the
/// guard, the constant, every count and every elimination rule intact; literal payloads are
/// stripped before all of those rules, so both frozen fingerprints could be synchronized to
/// `if value == { return Ok(()); }` and the whole gate chain stayed green while a 129-byte value
/// parsed. So a step the contract does not name is refused outright, and the deciding function may
/// hold no string literal at all — it tests length and per-byte class, never a whole value.
fn assert_classify_is_the_contract_decision_procedure() {
    let source = strip_comments_and_literals(IDENTITY_SOURCE);
    let Some(body) = function_body(&source, BOUND_FUNCTION) else {
        panic!("platform identity decision procedure: {BOUND_FUNCTION} body unreadable");
    };
    assert_eq!(
        without_trailing_commas(&rust_tokens(body))
            .into_iter()
            .map(str::to_owned)
            .collect::<Vec<String>>(),
        admitted_classify_body(),
        "platform identity decision procedure: {BOUND_FUNCTION} admits exactly the steps the \
         contract names, in that order"
    );
    let Some(raw) = function_body(IDENTITY_SOURCE, BOUND_FUNCTION) else {
        panic!("platform identity decision procedure: {BOUND_FUNCTION} source unreadable");
    };
    assert_eq!(
        string_literals(raw),
        Vec::<String>::new(),
        "platform identity decision procedure: {BOUND_FUNCTION} may hold no string literal — a \
         literal there is a value-keyed branch, and payloads are stripped before every other rule"
    );
}

/// Proves NO length past the bound is accepted, not merely the first one.
///
/// The 128/129 pair below fixes the boundary and nothing else, so an early accept keyed to some
/// other over-bound value — reviewer Task 1's 200-byte literal — passes it untouched while the
/// public API returns a value the contract forbids. This sweeps every length to twice the bound
/// under two canonical seeds, so any length-keyed accept fails here regardless of what the
/// structural rules can see. Its body is bound by the always-run Python checker.
fn assert_no_length_past_the_bound_is_accepted() {
    let mut admitted = 0;
    let mut refused = 0;
    for seed in RUNTIME_PROOF_SEEDS {
        for length in 1..=RUNTIME_PROOF_SWEEP {
            let candidate = seed.repeat(length);
            let parsed = TenantId::parse(candidate.clone());
            if length <= GRAMMAR_MAX_BYTES {
                assert!(
                    parsed.is_ok(),
                    "platform identity runtime sweep: an admitted length is rejected"
                );
                admitted += 1;
            } else {
                let Err(error) = parsed else {
                    panic!("platform identity runtime sweep: an over-length value is accepted");
                };
                assert_eq!(
                    error.kind(),
                    IdentityValueErrorKind::TooLong {
                        max_bytes: GRAMMAR_MAX_BYTES
                    },
                    "platform identity runtime sweep: reported bound"
                );
                refused += 1;
            }
        }
    }
    // The sweep's own EXTENT, judged against the contract's number rather than against the two
    // carriers that drive it. Round 18 bound this body's tokens and nothing else, so emptying the
    // seeds or halving the span left every token above in place and swept nothing: the loops are
    // the claim, and these two counts are what makes the claim answerable.
    assert_eq!(
        admitted,
        2 * GRAMMAR_MAX_BYTES,
        "platform identity runtime sweep: admitted-length coverage"
    );
    assert_eq!(
        refused,
        2 * GRAMMAR_MAX_BYTES,
        "platform identity runtime sweep: over-length coverage"
    );
}

/// Proves each load-bearing call reaches the file-level helper whose name it spells.
///
/// A plain-statement call is a fact about tokens; which function it runs is a fact about NAME
/// RESOLUTION, and Rust resolves lexically. An item declared in the caller's own body binds the
/// name ahead of the module's, so
///
/// ```text
/// let _ = crate::assert_no_length_past_the_bound_is_accepted as fn();
/// fn r#assert_no_length_past_the_bound_is_accepted() {}
/// assert_no_length_past_the_bound_is_accepted();
/// ```
///
/// keeps the real helper used, leaves Round 18's statement-position rules satisfied, and calls a
/// no-op. It passed the checker, all 303 suite tests, fmt, clippy and every cargo gate.
///
/// Two facts close it, and neither is a spelling. A shadow needs a DECLARATION, so no caller may
/// declare an item at all; and a declaration must WRITE the name, so no caller may spell a
/// load-bearing name more than the once its call spends. `use x as helper;`, `let helper = ..`, a
/// closure parameter and a raw identifier are all caught by one or the other.
fn assert_load_bearing_calls_reach_their_helper() {
    let module = strip_comments_and_literals(TEST_SOURCE);
    let module_tokens = without_raw_identifiers(&rust_tokens(&module));
    for helper in LOAD_BEARING_HELPERS {
        let declared = module_tokens
            .windows(2)
            .filter(|window| window[0] == "fn" && window[1] == helper)
            .count();
        assert_eq!(
            declared, 1,
            "platform identity helper resolution: {helper} is declared {declared} times in this \
             module — every rule that reads `the` helper's body must name exactly one body"
        );
    }
    for caller in SHADOWABLE_CALLERS {
        let Some(body) = function_body(&module, caller) else {
            panic!("platform identity helper resolution: {caller} body unreadable");
        };
        let tokens = without_raw_identifiers(&rust_tokens(body));
        assert_eq!(
            declared_items(&tokens),
            Vec::<String>::new(),
            "platform identity helper resolution: {caller} declares an item — a local item binds \
             its name ahead of the module's, so a call it shadows proves nothing"
        );
        for helper in LOAD_BEARING_HELPERS {
            let spelled = tokens.iter().filter(|token| **token == helper).count();
            assert!(
                spelled <= 1,
                "platform identity helper resolution: {caller} spells {helper} {spelled} times, \
                 expected at most the one call — a second mention is how a shadow is written"
            );
        }
    }
}

/// Binds the VALUES the length sweep is driven by, not merely the names its body spells.
///
/// Round 18 froze the sweep's token sequence, which fixes the loops and leaves what they range
/// over free: `const RUNTIME_PROOF_SEEDS: [&str; 0] = [];` and `= GRAMMAR_MAX_BYTES;` each left
/// every bound token in place and swept nothing, or nothing past the bound, with all gates green.
/// So the seeds are checked for what the sweep needs of them — two distinct single-byte values the
/// grammar itself admits — and the span against the contract's number, not against a carrier.
fn assert_sweep_carriers_are_the_contract_extent() {
    assert_eq!(
        RUNTIME_PROOF_SEEDS.len(),
        2,
        "platform identity runtime sweep: seed count"
    );
    let mut seen: Vec<&str> = Vec::new();
    for seed in RUNTIME_PROOF_SEEDS {
        assert_eq!(
            seed.len(),
            1,
            "platform identity runtime sweep: a seed must be one byte, so `seed.repeat(length)` \
             sweeps lengths rather than multiples of one"
        );
        assert!(
            TenantId::parse(seed).is_ok(),
            "platform identity runtime sweep: a seed must itself be admitted, or every length \
             below the bound is refused for the seed rather than for the length"
        );
        assert!(
            !seen.contains(&seed),
            "platform identity runtime sweep: the seeds must be distinct — a repeated seed sweeps \
             one alphabet twice and claims two"
        );
        seen.push(seed);
    }
    assert_eq!(
        RUNTIME_PROOF_SWEEP,
        2 * GRAMMAR_MAX_BYTES,
        "platform identity runtime sweep: the span must reach twice the contract's bound, so the \
         over-length half of the sweep is not empty"
    );
}

/// The runtime half of the max-byte closure, driven through the PUBLIC API.
///
/// Every token of this body is bound structurally by the always-run Python checker, keyed to the
/// CONTRACT's number. That is deliberate: Round 16 pinned only that `AUTH-011` CALLS its bound
/// helper, so deleting this proof's load-bearing tail while leaving the call in place kept the gate
/// chain green. A call site is not a proof body.
fn assert_contract_bound_is_the_effective_runtime_limit() {
    let admitted = "a".repeat(GRAMMAR_MAX_BYTES);
    let refused = "a".repeat(GRAMMAR_MAX_BYTES + 1);
    let Ok(parsed) = TenantId::parse(admitted.clone()) else {
        panic!("platform identity runtime bound: the last admitted length is rejected");
    };
    assert_eq!(
        parsed.as_str(),
        admitted,
        "platform identity runtime bound: admitted value is not retained"
    );
    let Err(error) = TenantId::parse(refused) else {
        panic!("platform identity runtime bound: an over-length value is accepted");
    };
    assert_eq!(
        error.kind(),
        IdentityValueErrorKind::TooLong {
            max_bytes: GRAMMAR_MAX_BYTES
        },
        "platform identity runtime bound: reported bound"
    );
}

/// Proves the EFFECTIVE length bound resolves to the contract-bound constant.
///
/// Round 15 bound the DECLARATION `const MAX_IDENTITY_BYTES: usize = 128;` to the contract and froze
/// `classify`'s exact body. Neither closes the class, because the body fingerprint is itself one of
/// the mutable carriers: a body that declares a local `const EFFECTIVE_MAX_IDENTITY_BYTES: usize =
/// 129;`, compares and reports through it, and keeps the module constant alive as
/// `let _ = MAX_IDENTITY_BYTES;` leaves the contract, both checker tables and every declared `128`
/// in place. With the fingerprints and this suite's corpus constant co-mutated with it, the whole
/// gate chain stayed green while an external caller parsed a 129-byte ID and was told 129.
///
/// So the accounting below eliminates every place a second bound could come from, and then drives
/// the exact boundary through the public API using the CONTRACT's number rather than the corpus
/// constant any co-mutation would have moved with the implementation.
fn assert_effective_max_byte_bound_is_contract_bound() {
    // The corpus constant is not an independent authority — every length fixture derives from it.
    assert_eq!(
        MAX_BYTES, GRAMMAR_MAX_BYTES,
        "platform identity effective max-byte bound: corpus constant is not the contract bound"
    );
    let source = strip_comments_and_literals(IDENTITY_SOURCE);
    assert_eq!(
        module_level_usize_constants(&source, BOUND_CONSTANT),
        vec![GRAMMAR_MAX_BYTES],
        "platform identity effective max-byte bound: module-level {BOUND_CONSTANT}"
    );
    let Some(body) = function_body(&source, BOUND_FUNCTION) else {
        panic!("platform identity effective max-byte bound: {BOUND_FUNCTION} body unreadable");
    };
    let module_tokens = rust_tokens(&source);
    let body_tokens = rust_tokens(body);

    // Exactly two occurrences inside the deciding function, and only the declaration outside it.
    let inside = body_tokens
        .iter()
        .filter(|token| **token == BOUND_CONSTANT)
        .count();
    let total = module_tokens
        .iter()
        .filter(|token| **token == BOUND_CONSTANT)
        .count();
    assert_eq!(
        inside, 2,
        "platform identity effective max-byte bound: {BOUND_CONSTANT} occurs {inside} times in \
         {BOUND_FUNCTION}, expected the comparison and the reported bound"
    );
    assert_eq!(
        total - inside,
        1,
        "platform identity effective max-byte bound: {BOUND_CONSTANT} occurs outside \
         {BOUND_FUNCTION} other than in its declaration"
    );

    // The effective comparison and the reported bound are the contract-bound name itself, not
    // something derived from it, and the module measures length exactly once.
    let admitted = format!("{BOUND_SUBJECT} {BOUND_OPERATOR} {BOUND_CONSTANT}");
    assert_eq!(
        length_measurements(&body_tokens),
        1,
        "platform identity effective max-byte bound: {BOUND_FUNCTION} length measurements"
    );
    assert_eq!(
        length_comparisons(&body_tokens),
        vec![admitted.clone()],
        "platform identity effective max-byte bound: {BOUND_FUNCTION} length comparison"
    );
    assert_eq!(
        field_values(&body_tokens, BOUND_FIELD),
        vec![BOUND_CONSTANT.to_owned()],
        "platform identity effective max-byte bound: {BOUND_FUNCTION} reported bound"
    );

    // No item, no shadowing binding, no second number, one measured subject bound to the whole
    // candidate: there is nowhere left for a second bound to be introduced.
    assert_eq!(
        declared_items(&body_tokens),
        Vec::<String>::new(),
        "platform identity effective max-byte bound: {BOUND_FUNCTION} declares an item"
    );
    assert_eq!(
        integer_literals(&body_tokens),
        BOUND_ADMITTED_LITERALS.map(str::to_owned).to_vec(),
        "platform identity effective max-byte bound: {BOUND_FUNCTION} integer literals"
    );
    let patterns = binding_patterns(&body_tokens);
    assert!(
        !patterns
            .iter()
            .any(|pattern| pattern_binds(pattern, BOUND_CONSTANT)),
        "platform identity effective max-byte bound: {BOUND_FUNCTION} shadows {BOUND_CONSTANT}"
    );
    let subject: Vec<String> = patterns
        .into_iter()
        .filter(|pattern| pattern_binds(pattern, BOUND_SUBJECT))
        .collect();
    assert_eq!(
        subject,
        vec![BOUND_SUBJECT.to_owned()],
        "platform identity effective max-byte bound: {BOUND_FUNCTION} binds {BOUND_SUBJECT}"
    );
    assert_eq!(
        body.split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
            .matches(BOUND_SUBJECT_BINDING)
            .count(),
        1,
        "platform identity effective max-byte bound: {BOUND_FUNCTION} must measure exactly one \
         {BOUND_SUBJECT_BINDING}"
    );

    // Module-wide totals, so a helper cannot hold the comparison or construct the report.
    assert_eq!(
        length_comparisons(&module_tokens),
        vec![admitted],
        "platform identity effective max-byte bound: module length comparisons"
    );
    assert_eq!(
        field_values(&module_tokens, BOUND_FIELD),
        BOUND_FIELD_VALUES.map(str::to_owned).to_vec(),
        "platform identity effective max-byte bound: module {BOUND_FIELD} fields"
    );

    // An occurring comparison is not a deciding one: bind the whole guard and its branch.
    assert_max_byte_guard_is_the_deciding_condition(&body_tokens, &module_tokens);
    // …and a bound guard is not a bound procedure: bind every step the contract names.
    assert_classify_is_the_contract_decision_procedure();

    // …and a bound call is not a call to the bound helper: prove the names resolve outward before
    // trusting anything the calls below establish.
    assert_load_bearing_calls_reach_their_helper();

    // The runtime half no static carrier can stand in for, keyed to the CONTRACT's number.
    assert_corpus_macro_cannot_skip_a_row();
    assert_sweep_carriers_are_the_contract_extent();
    assert_contract_bound_is_the_effective_runtime_limit();
    assert_no_length_past_the_bound_is_accepted();
}

/// Returns the payload of every string literal of `code`, in source order.
fn string_literals(code: &str) -> Vec<String> {
    let bytes = code.as_bytes();
    let mut payloads = Vec::new();
    let mut index = 0;
    while index < bytes.len() {
        let mut probe = index;
        if bytes[probe] == b'b' {
            probe += 1;
        }
        if probe < bytes.len() && bytes[probe] == b'r' {
            let mut hashes = 0;
            let mut scan = probe + 1;
            while scan < bytes.len() && bytes[scan] == b'#' {
                hashes += 1;
                scan += 1;
            }
            if scan < bytes.len() && bytes[scan] == b'"' {
                let mut terminator = vec![b'"'];
                terminator.extend(std::iter::repeat_n(b'#', hashes));
                let start = scan + 1;
                let mut end = start;
                while end < bytes.len() && !bytes[end..].starts_with(&terminator) {
                    end += 1;
                }
                if end >= bytes.len() {
                    break;
                }
                payloads.push(code[start..end].to_owned());
                index = end + terminator.len();
                continue;
            }
        }
        let quote_at =
            if bytes[index] == b'b' && index + 1 < bytes.len() && bytes[index + 1] == b'"' {
                Some(index + 1)
            } else if bytes[index] == b'"' {
                Some(index)
            } else {
                None
            };
        if let Some(quote) = quote_at {
            let start = quote + 1;
            let mut end = start;
            while end < bytes.len() {
                if bytes[end] == b'\\' {
                    end += 2;
                    continue;
                }
                if bytes[end] == b'"' {
                    break;
                }
                end += 1;
            }
            payloads.push(code[start..end.min(bytes.len())].to_owned());
            index = end + 1;
            continue;
        }
        index += code[index..].chars().next().map_or(1, char::len_utf8);
    }
    payloads
}

/// Returns the sorted byte-literal payloads of `matches!(byte, b'x' | …)`, multiplicity kept.
fn byte_literals(body: &str) -> Vec<char> {
    let mut found = Vec::new();
    for (at, _) in body.match_indices("b'") {
        let rest = &body[at + 2..];
        let mut characters = rest.chars();
        if let Some(character) = characters.next()
            && characters.next() == Some('\'')
        {
            found.push(character);
        }
    }
    found.sort_unstable();
    found
}

const IDENTITY_SOURCE: &str = include_str!("../src/identity.rs");
const INVOCATION_SOURCE: &str = include_str!("../src/invocation.rs");
const MARKET_SOURCE: &str = include_str!("../src/market.rs");
const LIB_SOURCE: &str = include_str!("../src/lib.rs");
const SESSION_SOURCE: &str = include_str!("../src/session.rs");
const MANIFEST_SOURCE: &str = include_str!("../Cargo.toml");
const LOCKFILE_SOURCE: &str = include_str!("../../../Cargo.lock");

/// Exact dependency specifications, so an admitted NAME cannot be redirected to another crate.
const ADMITTED_DEPENDENCY_SPECS: [&str; 4] = [
    "semver.workspace = true",
    "serde.workspace = true",
    "serde_json.workspace = true",
    "ustc-agent-tool-protocol.workspace = true",
];
const ADMITTED_DEV_DEPENDENCY_SPECS: [&str; 1] = ["hex = \"0.4.3\""];

/// The resolved source of each direct dependency, as recorded in the committed lockfile.
///
/// An in-repo path dependency has no `source` line at all, so a redirect to a local fake changes
/// this even when the dependency name and version are preserved verbatim.
const CRATES_IO_SOURCE: &str = "registry+https://github.com/rust-lang/crates.io-index";
const RESOLVED_DEPENDENCY_SOURCES: [(&str, &str); 5] = [
    ("semver", CRATES_IO_SOURCE),
    ("serde", CRATES_IO_SOURCE),
    ("hex", CRATES_IO_SOURCE),
    ("serde_json", CRATES_IO_SOURCE),
    ("ustc-agent-tool-protocol", "<in-repo path>"),
];

/// Returns the recorded source of every `[[package]]` in `lock` with the given name.
///
/// `Cargo.lock` blocks are a flat, regular `key = "value"` shape, so this needs no TOML parser.
/// A package with no `source` line is in-repo and is reported as `<in-repo path>` rather than
/// being dropped, so a stripped source line is drift instead of an absent entry.
fn locked_package_sources(lock: &str, name: &str) -> Vec<String> {
    let mut sources = Vec::new();
    for block in lock.split("[[package]]").skip(1) {
        let block = block.split("\n[").next().unwrap_or(block);
        let field = |key: &str| -> Option<String> {
            block
                .lines()
                .map(str::trim)
                .find_map(|line| line.strip_prefix(key))
                .and_then(|rest| rest.trim().strip_prefix('='))
                .map(|value| value.trim().trim_matches('"').to_owned())
        };
        if field("name").as_deref() == Some(name) {
            sources.push(field("source").unwrap_or_else(|| "<in-repo path>".to_owned()));
        }
    }
    sources
}

/// Every `mod`/`use`/`type` item of each governed source, in source order, with its attribute
/// envelope. This is an exact allowlist, not a screen: an added item, a removed item, a
/// re-spelled use tree and an attributed module declaration are all drift.
///
/// The cost is real — an M20 change to the protocol import list must be mirrored here and in
/// `scripts/check_repo_contracts.py`. That is the price of a frozen `platform-identity/v0`
/// surface, and the failure message names the drift.
const ADMITTED_IDENTITY_ITEMS: [&str; 8] = [
    "use std::error::Error;",
    "use std::fmt;",
    "use std::str::FromStr;",
    "use serde::de;",
    "use serde::{Deserialize, Deserializer, Serialize, Serializer};",
    "type Error = IdentityValueError;",
    "type Error = IdentityValueError;",
    "type Err = IdentityValueError;",
];

const ADMITTED_INVOCATION_ITEMS: [&str; 6] = [
    "pub use crate::identity::{TenantId, UserId};",
    "use std::collections::BTreeSet;",
    "use std::error::Error;",
    "use std::fmt;",
    concat!(
        "use ustc_agent_tool_protocol::{ AgentTool, AgentToolDefinition, AgentToolsetView, ",
        "ProjectionSnapshotId, ProtocolConstructionError, ProtocolRunId, ProtocolTurnId, ",
        "ToolRouteRef, is_valid_tool_name, };"
    ),
    concat!(
        "pub use ustc_agent_tool_protocol::{ ArgumentConstructionError, ",
        "CanonicalArgumentNodeV0, CanonicalArgumentValueV0, InvalidValue, ",
        "SchemaConstructionError, Sha256Digest, UnvalidatedArgumentValueV0, ",
        "UnvalidatedSchemaNodeV0, UnvalidatedToolInputSchemaV0, ValidatedSchemaNodeV0, ",
        "ValidatedToolInputSchemaV0, };"
    ),
];

const ADMITTED_MARKET_ITEMS: [&str; 12] = [
    "pub mod authority;",
    "pub mod capability;",
    "pub mod grant;",
    "pub mod installation;",
    "pub mod update;",
    concat!(
        "use crate::invocation::{ CapabilityId, CatalogRevision, ComponentKind, PackageId, ",
        "PackageVersion, Sha256Digest, };"
    ),
    "use serde::Deserialize;",
    "use serde::de::{self, MapAccess, Visitor};",
    "use std::collections::{BTreeMap, BTreeSet};",
    "use std::error::Error;",
    "use std::fmt;",
    "type Value = UniqueStringMap;",
];

const ADMITTED_LIB_ITEMS: [&str; 6] = [
    "pub mod identity;",
    "pub mod invocation;",
    "pub mod market;",
    "pub mod session;",
    "#[cfg(test)] mod tests",
    "use super::*;",
];

/// The `M00-B2` session module's complete `mod`/`use`/`type` item list.
///
/// Its last entry is the enumerated cross-file identity binding, which two independent carriers
/// must both admit: this allowlist, like any other item, AND the exception below that otherwise
/// refuses every `use` naming an admitted kind or the identity module path.
const ADMITTED_SESSION_ITEMS: [&str; 7] = [
    "use std::error::Error;",
    "use std::fmt;",
    "use serde::de;",
    "use serde::{Deserialize, Deserializer, Serialize};",
    "use crate::identity::{SessionId, TenantId, UserId};",
    "#[cfg(test)] mod tests",
    "use super::*;",
];

/// The complete `impl` surface of each sibling, sorted. These are M20 items; a genuine M20
/// addition is drift that must be admitted here explicitly rather than arriving unseen.
const ADMITTED_INVOCATION_IMPLS: [&str; 12] = [
    "impl $name",
    "impl AuthorizedInvocation",
    "impl Error for InvocationAuthorizationError",
    "impl Error for ProjectionResolutionError",
    "impl InvocationResolver",
    "impl PackageVersion",
    "impl ResolvedInvocation",
    "impl ToolProjectionSnapshot",
    "impl fmt::Display for InvocationAuthorizationError",
    "impl fmt::Display for ProjectionResolutionError",
    "impl-arg Into<String>",
    "impl-arg IntoIterator<Item = &'a str>",
];

const ADMITTED_MARKET_IMPLS: [&str; 13] = [
    "impl CatalogReadModel",
    "impl ComponentDeclaration",
    "impl Deserialize<'de> for UniqueStringMap",
    "impl Error for CatalogReadModelError",
    "impl Error for PackageLoadError",
    "impl Error for PackageValidationError",
    "impl InstallPolicy",
    "impl PackageValidationError",
    "impl ValidatedPackageManifest",
    "impl Visitor<'de> for UniqueStringMapVisitor",
    "impl fmt::Display for CatalogReadModelError",
    "impl fmt::Display for PackageLoadError",
    "impl fmt::Display for PackageValidationError",
];

const ADMITTED_LIB_IMPLS: [&str; 1] = ["impl SourceAuthority"];

/// The `M00-B2` session module's complete `impl` surface, sorted. An allowlist rather than a
/// kind blacklist, because a blanket `impl<T> Extension for T` names no governed kind yet
/// covers all six.
const ADMITTED_SESSION_IMPLS: [&str; 29] = [
    "impl AuthAdapterId",
    "impl CredentialEvidenceDigest",
    "impl Deserialize<'de> for AuthAdapterId",
    "impl Deserialize<'de> for CredentialEvidenceDigest",
    "impl Deserialize<'de> for SessionCredentialEvidence",
    "impl Deserialize<'de> for SessionDuration",
    "impl Error for SessionDomainError",
    "impl Error for SessionValueError",
    "impl ExpireSession",
    "impl OpenSession",
    "impl RefreshSession",
    "impl RevokeSession",
    "impl SessionCommand",
    "impl SessionCredentialEvidence",
    "impl SessionDuration",
    "impl SessionEvent",
    "impl SessionExpired",
    "impl SessionInstant",
    "impl SessionOpened",
    "impl SessionPolicy",
    "impl SessionRefreshed",
    "impl SessionRevoked",
    "impl SessionSnapshot",
    "impl SessionValueError",
    "impl fmt::Debug for CredentialEvidenceDigest",
    "impl fmt::Display for SessionDomainError",
    "impl fmt::Display for SessionValueError",
    "impl-arg Into<String>",
    "impl-arg Into<String>",
];

/// Macro INVOCATION names of each governed source, pinned exactly. A splicing macro reached by
/// any spelling adds items from a file no scan reads, and no substring enumerates the spellings
/// of `include /* x */ !("f.rs")`. `include_str!` stays admitted in lib.rs, which legitimately
/// embeds the first-party manifests as data.
const ADMITTED_IDENTITY_MACRO_INVOCATIONS: [&str; 5] =
    ["concat", "identity_value", "matches", "stringify", "write"];
const ADMITTED_INVOCATION_MACRO_INVOCATIONS: [&str; 3] = ["authority_id", "format", "write"];
const ADMITTED_MARKET_MACRO_INVOCATIONS: [&str; 2] = ["matches", "write"];
const ADMITTED_LIB_MACRO_INVOCATIONS: [&str; 4] = ["assert", "assert_eq", "include_str", "panic"];
const ADMITTED_SESSION_MACRO_INVOCATIONS: [&str; 5] =
    ["assert", "assert_eq", "matches", "panic", "write"];

/// The Cargo target set of `platform-core`, pinned by the same key sets as
/// `check_platform_core_manifest` in `scripts/check_repo_contracts.py`.
///
/// Structural rather than a literal line diff: a `#` comment, a blank line or a reordered
/// dependency changes the file's text without changing which files Cargo compiles, and a guard
/// that rejected those would fail the frozen-identity-surface gate for an edit that has nothing
/// to do with the identity surface — and would disagree with the Python carrier.
const ADMITTED_MANIFEST_TABLES: [&str; 5] = [
    "dependencies",
    "dev-dependencies",
    "lib",
    "lints",
    "package",
];
const ADMITTED_MANIFEST_PACKAGE_KEYS: [&str; 8] = [
    "authors",
    "edition",
    "homepage",
    "license",
    "name",
    "repository",
    "rust-version",
    "version",
];
const ADMITTED_MANIFEST_DEPENDENCIES: [&str; 4] =
    ["semver", "serde", "serde_json", "ustc-agent-tool-protocol"];
const ADMITTED_MANIFEST_DEV_DEPENDENCIES: [&str; 1] = ["hex"];
const ADMITTED_MANIFEST_LIB_PATH: &str = "\"src/lib.rs\"";

/// Returns `(table, key)` for every key line of a Cargo manifest, table headers normalized to
/// their bare name and dotted keys reduced to their first segment, so `edition.workspace = true`
/// is the `edition` key exactly as `tomllib` reports it.
fn manifest_entries(manifest: &str) -> Vec<(String, String)> {
    let mut entries = Vec::new();
    let mut table = String::new();
    for line in manifest.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some(rest) = line.strip_prefix('[') {
            let header = rest.split(']').next().unwrap_or_default();
            table = header.trim_start_matches('[').trim().to_owned();
            entries.push((table.clone(), String::new()));
            continue;
        }
        let Some((key, _)) = line.split_once('=') else {
            continue;
        };
        let key = key.trim().split('.').next().unwrap_or_default().trim();
        entries.push((table.clone(), key.to_owned()));
    }
    entries
}

/// Returns the complete, whitespace-normalized specification LINE of every entry under `table`.
///
/// `manifest_keys` answers "which dependency names are declared"; this answers "and what does
/// each name resolve to", which is the question a `path`/`git` redirect changes while leaving
/// the key set identical.
fn manifest_specifications(manifest: &str, table: &str) -> Vec<String> {
    let mut specifications = Vec::new();
    let mut current = String::new();
    for line in manifest.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some(rest) = line.strip_prefix('[') {
            let header = rest.split(']').next().unwrap_or_default();
            current = header.trim_start_matches('[').trim().to_owned();
            continue;
        }
        if current == table && line.contains('=') {
            specifications.push(line.split_whitespace().collect::<Vec<_>>().join(" "));
        }
    }
    specifications
}

/// Returns the sorted, deduplicated keys declared under `table`.
fn manifest_keys(entries: &[(String, String)], table: &str) -> Vec<String> {
    let mut keys: Vec<String> = entries
        .iter()
        .filter(|(name, key)| name == table && !key.is_empty())
        .map(|(_, key)| key.clone())
        .collect();
    keys.sort();
    keys.dedup();
    keys
}

/// Every admitted cross-file binding of an identity kind, by exact file and exact normalized
/// text. Adding a row is registered surface drift that must be mirrored in
/// `scripts/check_repo_contracts.py`; it changes no accepted grammar, bound, error precedence,
/// Serde shape or nominal kind set.
const ADMITTED_CROSS_FILE_IDENTITY_BINDINGS: [(&str, &str); 2] = [
    (
        "invocation.rs",
        "pub use crate::identity::{TenantId, UserId};",
    ),
    (
        "session.rs",
        "use crate::identity::{SessionId, TenantId, UserId};",
    ),
];

/// The six kinds whose public surface `platform-identity/v0` freezes.
const ADMITTED_IDENTITY_KINDS: [&str; 6] = [
    "TenantId",
    "UserId",
    "SessionId",
    "RequestId",
    "CommandId",
    "CorrelationId",
];

/// Exactly the imports `platform-identity/v0` admits. Serde is the one admitted encoding
/// foundation; everything else is `core`/`std` value machinery.
const ALLOWED_IDENTITY_IMPORTS: [&str; 5] = [
    "use std::error::Error;",
    "use std::fmt;",
    "use std::str::FromStr;",
    "use serde::de;",
    "use serde::{Deserialize, Deserializer, Serialize, Serializer};",
];

/// Code carriers that would mean the module mints values or reached an adapter.
const FORBIDDEN_IDENTITY_CARRIERS: [&str; 26] = [
    "uuid",
    "Uuid",
    "ulid",
    "Ulid",
    "nanoid",
    "NanoId",
    "rand",
    "Rng",
    "random",
    "generate",
    "mint",
    "SystemTime",
    "Instant",
    "chrono",
    "std::time",
    "std::net",
    "TcpStream",
    "reqwest",
    "hyper",
    "std::fs",
    "std::process",
    "sqlx",
    "diesel",
    "rusqlite",
    "axum",
    "dioxus",
];

/// Replaces Rust line/block comments and ordinary, byte and raw string and char literals with
/// one space, so that only code carriers remain. Documentation prose and test sentinels must not
/// trip the scan.
///
/// Each removed span becomes a single space rather than nothing, because a comment is a token
/// SEPARATOR in Rust. Deleting it welds the neighbours together: `extern/**/crate` would become
/// the single identifier `externcrate`, invisible to every `extern crate` scan while Rust still
/// reads two keywords and compiles the item.
fn strip_comments_and_literals(source: &str) -> String {
    strip_rust_source(source, false)
}

/// The same stripper, but emitting literal SPANS verbatim.
///
/// The grammar's semantics live inside literals — the interior byte set and the length bound —
/// so a fingerprint taken over stripped literals pins control flow while leaving those bytes
/// free to drift. Every token-accounting rule keeps using the stripping mode; only the targeted
/// semantic checks read this one.
fn strip_comments_only(source: &str) -> String {
    strip_rust_source(source, true)
}

fn strip_rust_source(source: &str, keep_literals: bool) -> String {
    let bytes = source.as_bytes();
    let mut output = String::with_capacity(source.len());
    let mut index = 0;
    while index < bytes.len() {
        let rest = &bytes[index..];
        if rest.starts_with(b"//") {
            while index < bytes.len() && bytes[index] != b'\n' {
                index += 1;
            }
            output.push(' ');
            continue;
        }
        if rest.starts_with(b"/*") {
            let mut depth = 1;
            index += 2;
            while index < bytes.len() && depth > 0 {
                if bytes[index..].starts_with(b"/*") {
                    depth += 1;
                    index += 2;
                } else if bytes[index..].starts_with(b"*/") {
                    depth -= 1;
                    index += 2;
                } else {
                    index += 1;
                }
            }
            output.push(' ');
            continue;
        }
        // Raw string: optional b/br prefix, r, then N hashes, then a quote.
        let mut probe = index;
        if bytes[probe] == b'b' {
            probe += 1;
        }
        if probe < bytes.len() && bytes[probe] == b'r' {
            let mut hashes = 0;
            let mut scan = probe + 1;
            while scan < bytes.len() && bytes[scan] == b'#' {
                hashes += 1;
                scan += 1;
            }
            if scan < bytes.len() && bytes[scan] == b'"' {
                let terminator = {
                    let mut pattern = vec![b'"'];
                    pattern.extend(std::iter::repeat_n(b'#', hashes));
                    pattern
                };
                let start = index;
                index = scan + 1;
                while index < bytes.len() && !bytes[index..].starts_with(&terminator) {
                    index += 1;
                }
                index = (index + terminator.len()).min(bytes.len());
                push_span(&mut output, source, start, index, keep_literals);
                continue;
            }
        }
        // Ordinary or byte string literal.
        let quote_at =
            if bytes[index] == b'b' && index + 1 < bytes.len() && bytes[index + 1] == b'"' {
                Some(index + 1)
            } else if bytes[index] == b'"' {
                Some(index)
            } else {
                None
            };
        if let Some(quote) = quote_at {
            let start = index;
            index = quote + 1;
            while index < bytes.len() {
                if bytes[index] == b'\\' {
                    index += 2;
                    continue;
                }
                if bytes[index] == b'"' {
                    index += 1;
                    break;
                }
                index += 1;
            }
            push_span(&mut output, source, start, index, keep_literals);
            continue;
        }
        // Character literal, distinguished from a lifetime by its closing quote. The optional
        // `b` prefix is part of the literal: `b'\n'` must vanish whole, or the stray `b` is
        // left behind as an identifier the Python mirror never emits.
        let quote_start =
            if bytes[index] == b'b' && index + 1 < bytes.len() && bytes[index + 1] == b'\'' {
                Some(index + 1)
            } else if bytes[index] == b'\'' {
                Some(index)
            } else {
                None
            };
        if let Some(quote) = quote_start {
            let mut scan = quote + 1;
            if scan < bytes.len() && bytes[scan] == b'\\' {
                scan += 2;
            } else if scan < bytes.len() {
                scan += 1;
                while scan < bytes.len() && (bytes[scan] & 0b1100_0000) == 0b1000_0000 {
                    scan += 1;
                }
            }
            if scan < bytes.len() && bytes[scan] == b'\'' {
                let start = index;
                index = scan + 1;
                push_span(&mut output, source, start, index, keep_literals);
                continue;
            }
        }
        let step = source[index..].chars().next().map_or(1, char::len_utf8);
        output.push_str(&source[index..index + step]);
        index += step;
    }
    output
}

/// Emits one literal span verbatim, or the single space that keeps its token boundary.
fn push_span(output: &mut String, source: &str, start: usize, end: usize, keep: bool) {
    if keep {
        output.push_str(&source[start..end]);
    } else {
        output.push(' ');
    }
}

/// `AUTH-015`
#[test]
fn identity_module_has_no_generation_or_adapter_surface() {
    let code = strip_comments_and_literals(IDENTITY_SOURCE);

    // The stripper must actually remove prose, otherwise the scan below proves nothing.
    assert!(
        IDENTITY_SOURCE.contains("//! Canonical `platform-identity/v0`"),
        "identity module doc header is missing"
    );
    assert!(
        !code.contains("Canonical `platform-identity/v0`"),
        "comment stripping failed"
    );
    assert!(
        !code.contains("a canonical platform-identity/v0 "),
        "string-literal stripping failed"
    );
    // The six kinds are macro-generated, so the surviving code carriers are the private
    // generator and its six invocation arguments.
    assert!(
        code.contains("macro_rules! identity_value"),
        "code was over-stripped"
    );
    for kind in [
        "TenantId",
        "UserId",
        "SessionId",
        "RequestId",
        "CommandId",
        "CorrelationId",
    ] {
        assert!(code.contains(kind), "code was over-stripped: {kind}");
    }

    // Adversarial cases for the stripper itself. `identity.rs` contains no raw or byte
    // strings today, so without these the branches that handle them would be untested and a
    // carrier hidden in one could silently survive the scan below.
    let adversarial = concat!(
        "// reqwest\n",
        "/* outer /* inner SystemTime */ still uuid */\n",
        "let a = \"rand\";\n",
        "let b = b\"ulid\";\n",
        "let c = r#\"use std::fs; mint\"#;\n",
        "let d = \"escaped \\\" still inside chrono\";\n",
        "impl<'de> Deserialize<'de> for Value {}\n",
        "fn kind(&self) -> &'static str {}\n",
        "let e = matches!(byte, b'-' | b'.');\n",
    );
    let stripped = strip_comments_and_literals(adversarial);
    for hidden in [
        "reqwest",
        "SystemTime",
        "uuid",
        "rand",
        "ulid",
        "std::fs",
        "mint",
        "chrono",
    ] {
        assert!(
            !stripped.contains(hidden),
            "stripper failed open on {hidden}"
        );
    }
    // Lifetimes must not be consumed as char literals, which would eat surrounding code.
    assert!(stripped.contains("Deserialize"));
    assert!(stripped.contains("for Value"));
    assert!(stripped.contains("&'static str"));
    assert!(stripped.contains("matches!(byte"));
    assert!(!stripped.contains("b'-'"));

    // Imports are an exact allowlist, so a new dependency cannot arrive unnoticed.
    let declared: Vec<&str> = code
        .lines()
        .map(str::trim)
        .filter(|line| line.starts_with("use "))
        .collect();
    for import in &declared {
        assert!(
            ALLOWED_IDENTITY_IMPORTS.contains(import),
            "identity module declared an unadmitted import: {import}"
        );
    }
    for allowed in ALLOWED_IDENTITY_IMPORTS {
        assert!(
            declared.contains(&allowed),
            "identity module import allowlist drifted from the source: {allowed}"
        );
    }

    // No generator, clock, RNG, transport, filesystem, database or framework carrier.
    for carrier in FORBIDDEN_IDENTITY_CARRIERS {
        assert!(
            !code.contains(carrier),
            "identity module gained a forbidden carrier: {carrier}"
        );
    }

    // The single validator is the only construction entry point; nothing mints a value.
    assert_eq!(code.matches("pub fn parse(").count(), 1);
    assert!(!code.contains("pub fn new("));
    assert!(!code.contains("pub fn generate"));
    assert!(!code.contains("impl Default"));
    assert!(!code.contains("impl Deref"));
    assert!(!code.contains("#[macro_export]"));
}

/// `AUTH-016`
#[test]
fn market_invocation_authority_uses_m00_identity_definitions() {
    // Compile-level proof: the two paths name one type, so no conversion is possible or needed.
    fn tenant_is_one_type(value: TenantId) -> invocation::TenantId {
        value
    }
    fn user_is_one_type(value: UserId) -> invocation::UserId {
        value
    }
    let tenant = TenantId::parse("tenant:synthetic").expect("canonical value");
    let user = UserId::parse("user:synthetic").expect("canonical value");
    assert_eq!(
        tenant_is_one_type(tenant.clone()).as_str(),
        "tenant:synthetic"
    );
    assert_eq!(user_is_one_type(user.clone()).as_str(), "user:synthetic");

    assert_eq!(
        TypeId::of::<TenantId>(),
        TypeId::of::<invocation::TenantId>()
    );
    assert_eq!(TypeId::of::<UserId>(), TypeId::of::<invocation::UserId>());

    // The M20 policy-snapshot identity stays distinct and unmigrated.
    let policy_snapshot_type = TypeId::of::<invocation::PolicySnapshotId>();
    for (name, identity_type) in [
        ("TenantId", TypeId::of::<TenantId>()),
        ("UserId", TypeId::of::<UserId>()),
        ("SessionId", TypeId::of::<SessionId>()),
        ("RequestId", TypeId::of::<RequestId>()),
        ("CommandId", TypeId::of::<CommandId>()),
        ("CorrelationId", TypeId::of::<CorrelationId>()),
    ] {
        assert_ne!(
            policy_snapshot_type, identity_type,
            "invocation PolicySnapshotId must not alias {name}"
        );
    }

    // It also keeps its own M20 error type and 256-byte bound, which platform-identity rejects.
    let wide = "p".repeat(200);
    let policy_snapshot =
        invocation::PolicySnapshotId::parse(wide.clone()).expect("M20 bound is unchanged");
    assert_eq!(policy_snapshot.as_str(), wide);
    assert!(TenantId::parse(wide).is_err());

    // The M20 grammar remains looser at the boundary; convergence did not leak into it.
    assert!(invocation::PolicySnapshotId::parse("policy-snapshot-").is_ok());
    assert!(TenantId::parse("policy-snapshot-").is_err());

    // The M20 constructor still reports the M20 error value, not IdentityValueError.
    let m20_error: invocation::InvalidValue =
        invocation::PolicySnapshotId::parse(" ").expect_err("M20 rejects whitespace");
    assert_eq!(m20_error, invocation::InvalidValue::Identity);
    let m00_error: IdentityValueError = TenantId::parse(" ").expect_err("M00 rejects whitespace");
    assert_eq!(m00_error.value_kind(), "TenantId");
}
