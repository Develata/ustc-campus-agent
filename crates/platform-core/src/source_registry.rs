//! The pure `source-import/v1` M60-B1 source-registry kernel.
//!
//! Owns the typed boundary between a reviewed source catalog and later
//! retrieval, parsing, revision and baseline adapters (M60-B2 onward). It
//! defines stable source identity, owner, authority class and exact canonical
//! URL; one operational `SourceStatus` (`Proposed | Approved | Suspended |
//! Revoked`); a non-zero monotone `SourceAuthorityRevision` guarded by
//! exact-revision compare-and-swap on every post-proposal lifecycle mutation;
//! the six-field retrieval-policy value surface; the sealed `RetrievalSubject`
//! projection available only from current `Approved` state; and the in-memory
//! registry that admits, transitions and looks up definitions.
//!
//! A syntactically valid source definition or review receipt proves shape only.
//! It does not prove that the URL is safe now, that permission exists, that an
//! operator actually reviewed evidence, or that retrieval may occur. An
//! application boundary may admit an approved definition only after it
//! authenticates and authorizes the reviewer and binds real evidence.
//! Model-proposed URLs always enter `Proposed`; no model/tool call can
//! construct immediate fetch authority.
//!
//! This module performs no I/O, reads no clock, computes no digest, resolves no
//! DNS, follows no redirect, persists nothing and infers review from no source
//! text. Rejected input may itself be sensitive, so no error variant, `Display`,
//! `Debug` or source chain produced here retains or echoes it — except the
//! catalog-public `SourceId` and `SourceUrl`, which the contract explicitly
//! permits `SourceRegistryError` to render because source IDs and canonical
//! URLs are public catalog references, not secrets.

use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;
use std::str::FromStr;

use serde::de;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::SourceAuthority;

// ---------------------------------------------------------------------------
// Fixed bounds (single source of truth for every classifier).
// ---------------------------------------------------------------------------

/// Maximum encoded length, in bytes, of a `SourceId`-family value.
const MAX_SOURCE_ID_BYTES: usize = 128;

/// Maximum encoded length, in UTF-8 bytes, of a `SourceOwner` value.
const MAX_SOURCE_OWNER_BYTES: usize = 128;

/// Maximum encoded length, in ASCII bytes, of a `SourceUrl` value.
const MAX_SOURCE_URL_BYTES: usize = 2048;

/// Maximum number of bytes per DNS label inside a `SourceUrl` host.
const MAX_DNS_LABEL_BYTES: usize = 63;

/// Ceiling for `SourceRetrievalPolicy::minimum_interval_seconds`.
const MAX_MINIMUM_INTERVAL_SECONDS: u32 = 604_800;

/// Ceiling for `SourceRetrievalPolicy::maximum_response_bytes`.
const MAX_MAXIMUM_RESPONSE_BYTES: u32 = 1_048_576;

/// Ceiling for `SourceRetrievalPolicy::maximum_elapsed_seconds`.
const MAX_MAXIMUM_ELAPSED_SECONDS: u32 = 60;

/// Maximum bytes per `type` or `subtype` component of a `SourceMediaType`.
const MAX_MEDIA_TYPE_COMPONENT_BYTES: usize = 64;

/// Maximum total encoded length, in bytes, of a `SourceMediaType` value.
const MAX_MEDIA_TYPE_BYTES: usize = 129;

// ---------------------------------------------------------------------------
// Value-error taxonomy (§8).
// ---------------------------------------------------------------------------

/// Which grammar rule rejected a candidate `source-import/v1` value.
///
/// Each variant carries a fixed bound or a byte offset only. No variant carries
/// the rejected input, a fragment of it, or the offending byte itself. The one
/// exception is `NonSourceAuthority`, which names an enum variant by its Rust
/// spelling rather than by an offending value — `ModelInference` is a public
/// authority class, not caller-supplied secret text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceValueErrorKind {
    /// The candidate had zero bytes.
    Empty,
    /// The candidate exceeded the fixed encoded-length bound.
    TooLong {
        /// The fixed maximum encoded length, in bytes.
        max_bytes: usize,
    },
    /// The first byte was not admitted by the value-specific boundary rule.
    InvalidStart,
    /// An interior byte was not admitted by the value-specific grammar.
    InvalidCharacter {
        /// Zero-based index of the first offending byte within the rejected bytes.
        byte_index: usize,
    },
    /// The final byte was not admitted by the value-specific boundary rule.
    InvalidEnd,
    /// `SourceUrl`: the scheme was not exactly lowercase `https://`.
    InvalidScheme,
    /// `SourceUrl`: the host was not a lowercase DNS text with at least two labels.
    InvalidHost,
    /// `SourceUrl`: the path was not admitted by the constrained path grammar.
    InvalidPath,
    /// `SourceRetrievalPolicy`: `minimum_interval_seconds` was zero.
    ZeroMinimumInterval,
    /// `SourceRetrievalPolicy`: `minimum_interval_seconds` exceeded the ceiling.
    MinimumIntervalTooLarge {
        /// The fixed ceiling, in seconds.
        max_seconds: u32,
    },
    /// `SourceRetrievalPolicy`: `maximum_response_bytes` was zero.
    ZeroMaximumResponseBytes,
    /// `SourceRetrievalPolicy`: `maximum_response_bytes` exceeded the ceiling.
    MaximumResponseBytesTooLarge {
        /// The fixed ceiling, in bytes.
        max_bytes: u32,
    },
    /// `SourceRetrievalPolicy`: `maximum_elapsed_seconds` was zero.
    ZeroMaximumElapsedSeconds,
    /// `SourceRetrievalPolicy`: `maximum_elapsed_seconds` exceeded the ceiling.
    MaximumElapsedSecondsTooLarge {
        /// The fixed ceiling, in seconds.
        max_seconds: u32,
    },
    /// `SourceMediaType`: the value was not an admitted lowercase
    /// `type/subtype` token pair.
    InvalidMediaType,
    /// Reserved operator override-window evidence rule for later M60 slices.
    ///
    /// `source-import/v1` adds the variant to the closed taxonomy; no v1
    /// constructor emits it, because override evidence is an M60-B2 transport
    /// decision, not a registry value.
    InvalidOverrideWindow,
    /// `SourceOwner`: the value began or ended with whitespace.
    OwnerBoundaryWhitespace,
    /// `SourceOwner`: the value contained a control character.
    OwnerControlCharacter {
        /// Zero-based index of the first offending byte within the rejected UTF-8 bytes.
        byte_index: usize,
    },
    /// `SourceDefinition` construction: the authority was `ModelInference`.
    ///
    /// An explanation class cannot become a source definition or approval
    /// candidate. The variant carries no payload: the rejected authority is a
    /// public enum variant, not caller-supplied text, and naming it would
    /// duplicate the Rust type system rather than protect a secret.
    NonSourceAuthority,
}

/// Why one `source-import/v1` construction failed.
///
/// The error names the Rust value kind that rejected the input and the grammar
/// rule that rejected it. It deliberately has no `source`, so no rejected input
/// can be reached by walking the error chain. It is `Copy` so it can be returned
/// by value without retaining the rejected text on the stack or heap.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceValueError {
    value_kind: &'static str,
    kind: SourceValueErrorKind,
}

impl SourceValueError {
    /// Returns the Rust type name of the value kind that rejected the input.
    #[must_use]
    pub const fn value_kind(&self) -> &'static str {
        self.value_kind
    }

    /// Returns the grammar rule that rejected the input.
    #[must_use]
    pub const fn kind(&self) -> SourceValueErrorKind {
        self.kind
    }
}

impl fmt::Display for SourceValueError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value_kind = self.value_kind;
        match self.kind {
            SourceValueErrorKind::Empty => {
                write!(formatter, "{value_kind} rejected: value is empty")
            }
            SourceValueErrorKind::TooLong { max_bytes } => {
                write!(
                    formatter,
                    "{value_kind} rejected: encoded length exceeds {max_bytes} bytes"
                )
            }
            SourceValueErrorKind::InvalidStart => {
                write!(
                    formatter,
                    "{value_kind} rejected: first byte is not admitted"
                )
            }
            SourceValueErrorKind::InvalidCharacter { byte_index } => {
                write!(
                    formatter,
                    "{value_kind} rejected: byte {byte_index} is not admitted"
                )
            }
            SourceValueErrorKind::InvalidEnd => {
                write!(
                    formatter,
                    "{value_kind} rejected: final byte is not admitted"
                )
            }
            SourceValueErrorKind::InvalidScheme => {
                write!(formatter, "{value_kind} rejected: scheme is not https://")
            }
            SourceValueErrorKind::InvalidHost => {
                write!(
                    formatter,
                    "{value_kind} rejected: host is not an admitted DNS name"
                )
            }
            SourceValueErrorKind::InvalidPath => {
                write!(formatter, "{value_kind} rejected: path is not admitted")
            }
            SourceValueErrorKind::ZeroMinimumInterval => {
                write!(formatter, "{value_kind} rejected: minimum interval is zero")
            }
            SourceValueErrorKind::MinimumIntervalTooLarge { max_seconds } => write!(
                formatter,
                "{value_kind} rejected: minimum interval exceeds {max_seconds} seconds"
            ),
            SourceValueErrorKind::ZeroMaximumResponseBytes => {
                write!(
                    formatter,
                    "{value_kind} rejected: maximum response bytes is zero"
                )
            }
            SourceValueErrorKind::MaximumResponseBytesTooLarge { max_bytes } => write!(
                formatter,
                "{value_kind} rejected: maximum response bytes exceeds {max_bytes} bytes"
            ),
            SourceValueErrorKind::ZeroMaximumElapsedSeconds => {
                write!(
                    formatter,
                    "{value_kind} rejected: maximum elapsed seconds is zero"
                )
            }
            SourceValueErrorKind::MaximumElapsedSecondsTooLarge { max_seconds } => write!(
                formatter,
                "{value_kind} rejected: maximum elapsed seconds exceeds {max_seconds} seconds"
            ),
            SourceValueErrorKind::InvalidMediaType => {
                write!(
                    formatter,
                    "{value_kind} rejected: media type is not an admitted lowercase token pair"
                )
            }
            SourceValueErrorKind::InvalidOverrideWindow => {
                write!(
                    formatter,
                    "{value_kind} rejected: override window is not admitted"
                )
            }
            SourceValueErrorKind::OwnerBoundaryWhitespace => {
                write!(
                    formatter,
                    "{value_kind} rejected: owner has boundary whitespace"
                )
            }
            SourceValueErrorKind::OwnerControlCharacter { byte_index } => {
                write!(
                    formatter,
                    "{value_kind} rejected: owner byte {byte_index} is a control character"
                )
            }
            SourceValueErrorKind::NonSourceAuthority => {
                write!(
                    formatter,
                    "{value_kind} rejected: authority is an explanation class, not a source"
                )
            }
        }
    }
}

impl Error for SourceValueError {}

/// Builds the one error shape this module reports from a rejecting validator.
pub(crate) const fn value_error(
    value_kind: &'static str,
    kind: SourceValueErrorKind,
) -> SourceValueError {
    SourceValueError { value_kind, kind }
}

// ---------------------------------------------------------------------------
// `SourceId`-family grammar (§3). Shared by `SourceId`, `SourceReviewerId`,
// `SourceReviewEvidenceId` and `SourceStatusEvidenceId`: identical byte
// grammar and bound, nominally distinct types.
// ---------------------------------------------------------------------------

/// Boundary bytes of a `SourceId`-family value are lowercase ASCII alphanumeric.
const fn is_source_id_boundary(byte: u8) -> bool {
    byte.is_ascii_lowercase() || byte.is_ascii_digit()
}

/// Interior bytes of a `SourceId`-family value add the four delimiters.
const fn is_source_id_interior(byte: u8) -> bool {
    byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'-' | b'.' | b'_' | b':')
}

/// Applies the `SourceId`-family grammar in the exact precedence frozen by the
/// accepted contract: empty; length; then first byte; then interior
/// left-to-right; then final byte.
///
/// Postcondition: `Ok(())` exactly when `value` matches
/// `^[a-z0-9](?:[-a-z0-9._:]{0,126}[a-z0-9])?$` and has `1..=128` bytes.
/// Invariant: exactly one left-to-right pass over the interior, and no
/// allocation.
pub(crate) fn classify_source_id(value: &str) -> Result<(), SourceValueErrorKind> {
    let bytes = value.as_bytes();
    let Some((&first, after_first)) = bytes.split_first() else {
        return Err(SourceValueErrorKind::Empty);
    };
    if bytes.len() > MAX_SOURCE_ID_BYTES {
        return Err(SourceValueErrorKind::TooLong {
            max_bytes: MAX_SOURCE_ID_BYTES,
        });
    }
    if !is_source_id_boundary(first) {
        return Err(SourceValueErrorKind::InvalidStart);
    }
    // A one-byte value is fully decided by the first-byte rule; the interior
    // range is then empty and there is no separate final byte.
    let Some((&last, interior)) = after_first.split_last() else {
        return Ok(());
    };
    for (offset, &byte) in interior.iter().enumerate() {
        if !is_source_id_interior(byte) {
            return Err(SourceValueErrorKind::InvalidCharacter {
                byte_index: offset + 1,
            });
        }
    }
    if !is_source_id_boundary(last) {
        return Err(SourceValueErrorKind::InvalidEnd);
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// `SourceOwner` grammar (§3.3). 1..=128 UTF-8 bytes, rejects leading/trailing
// whitespace and every control character, preserves accepted text exactly.
// ---------------------------------------------------------------------------

/// Applies the `SourceOwner` grammar in the exact precedence frozen by the
/// accepted contract: empty; length; then boundary whitespace; then control
/// characters left to right.
///
/// `char::is_whitespace` matches Unicode whitespace, which is wider than ASCII
/// — a deliberate choice, because an owner label that begins or ends with a
/// non-breaking space or an ideographic space is no less a boundary violation
/// than one wrapped in ASCII spaces. `char::is_control` matches every Unicode
/// control code, including the C0 range (`\0`..`\u{1f}` and `\u{7f}`) and the
/// C1 range (`\u{80}`..`\u{9f}`).
///
/// Postcondition: `Ok(())` exactly when `value` has `1..=128` UTF-8 bytes, no
/// leading or trailing whitespace, and no control character anywhere.
fn classify_source_owner(value: &str) -> Result<(), SourceValueErrorKind> {
    if value.is_empty() {
        return Err(SourceValueErrorKind::Empty);
    }
    if value.len() > MAX_SOURCE_OWNER_BYTES {
        return Err(SourceValueErrorKind::TooLong {
            max_bytes: MAX_SOURCE_OWNER_BYTES,
        });
    }
    // Boundary whitespace is checked before control characters: a value that is
    // entirely whitespace is a boundary violation first, and a leading space
    // ahead of a control character is a boundary violation first.
    if value.starts_with(char::is_whitespace) || value.ends_with(char::is_whitespace) {
        return Err(SourceValueErrorKind::OwnerBoundaryWhitespace);
    }
    for (offset, character) in value.char_indices() {
        if character.is_control() {
            return Err(SourceValueErrorKind::OwnerControlCharacter { byte_index: offset });
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// `SourceUrl` grammar (§3.4). Intentionally narrower than a general URL
// library: exactly one canonical public-HTTPS shape.
// ---------------------------------------------------------------------------

/// Returns the bytes after the `https://` prefix, or `None` if the prefix is
/// absent or the scheme is not exactly lowercase.
fn strip_https_scheme(value: &str) -> Option<&str> {
    value.strip_prefix("https://")
}

/// True when `label` is an all-numeric label, the shape of one IPv4-octet.
///
/// A DNS label may legitimately consist entirely of digits (`123.example.com`
/// is a valid name), so a single all-numeric label is not itself an IP literal.
/// The IPv4 shape is a host whose EVERY label is all-numeric; the caller checks
/// that aggregate condition.
fn is_all_numeric_label(label: &[u8]) -> bool {
    !label.is_empty() && label.iter().all(|byte| byte.is_ascii_digit())
}

/// Classifies one DNS label: `1..=63` ASCII bytes, begins and ends alphanumeric,
/// interior alphanumeric or `-`. `localhost`, empty labels and a trailing dot
/// are forbidden at the caller, not here.
fn classify_dns_label(label: &[u8]) -> bool {
    if label.is_empty() || label.len() > MAX_DNS_LABEL_BYTES {
        return false;
    }
    let first = label[0];
    let last = label[label.len() - 1];
    if !first.is_ascii_alphanumeric() || !last.is_ascii_alphanumeric() {
        return false;
    }
    label
        .iter()
        .all(|byte| byte.is_ascii_alphanumeric() || *byte == b'-')
}

/// True when `host` is a lowercase DNS name with at least two labels, each
/// admitted by `classify_dns_label`. A host whose every label is all-numeric
/// is the bare-IPv4 shape and is rejected. `localhost` is rejected by the
/// single-label rule.
fn is_admitted_host(host: &str) -> bool {
    if host.is_empty() {
        return false;
    }
    // A trailing dot is forbidden.
    if host.ends_with('.') {
        return false;
    }
    let labels: Vec<&str> = host.split('.').collect();
    if labels.len() < 2 {
        return false;
    }
    let mut all_numeric = true;
    for label in &labels {
        let bytes = label.as_bytes();
        // Each label must be lowercase ASCII. `is_ascii_alphanumeric` admits
        // uppercase too, so reject any uppercase byte first; this also covers
        // non-ASCII, which is neither lowercase nor alphanumeric-ASCII.
        if bytes
            .iter()
            .any(|byte| byte.is_ascii_uppercase() || !byte.is_ascii())
        {
            return false;
        }
        if !classify_dns_label(bytes) {
            return false;
        }
        if !is_all_numeric_label(bytes) {
            all_numeric = false;
        }
    }
    !all_numeric
}

/// True when `byte` is admitted by the path grammar: ASCII alphanumeric or one
/// of `-._~`, or `%` introducing an uppercase percent triplet.
const fn is_admitted_path_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~' | b'/' | b'%')
}

/// Classifies the path: begins with `/`, no empty/`.`/`..` segment, bytes
/// limited to ASCII alphanumeric plus `-._~` and uppercase percent triplets.
///
/// The percent-triplet rule is checked byte-by-byte: a `%` must be followed by
/// two uppercase hex digits. Lowercase hex, a bare `%`, or `%` at the end of
/// the string is rejected. No decoding, slash folding, dot-segment removal or
/// percent-case rewriting occurs.
fn classify_path(path: &str) -> bool {
    if !path.starts_with('/') {
        return false;
    }
    let bytes = path.as_bytes();
    let mut index = 0usize;
    while index < bytes.len() {
        let byte = bytes[index];
        if byte == b'%' {
            // An uppercase percent triplet: `%` followed by two uppercase hex
            // digits. Lowercase hex is rejected so percent-case rewriting
            // cannot smuggle a value through.
            if index + 2 >= bytes.len() {
                return false;
            }
            let hi = bytes[index + 1];
            let lo = bytes[index + 2];
            if !classify_percent_hex(hi) || !classify_percent_hex(lo) {
                return false;
            }
            index += 3;
            continue;
        }
        if !is_admitted_path_byte(byte) {
            return false;
        }
        index += 1;
    }
    // Segment grammar: after the leading `/`, segments are separated by `/`.
    // The root path `/` (empty remainder) has zero segments and is admitted.
    // Any empty segment — from `//` or a trailing `/` — is forbidden, as are
    // `.` and `..`.
    let after_leading = &path[1..];
    if after_leading.is_empty() {
        return true;
    }
    for segment in after_leading.split('/') {
        if segment.is_empty() {
            return false;
        }
        if segment == "." || segment == ".." {
            return false;
        }
    }
    true
}

/// True when `byte` is an uppercase hexadecimal digit admitted inside a
/// percent triplet.
fn classify_percent_hex(byte: u8) -> bool {
    byte.is_ascii_digit() || (b'A'..=b'F').contains(&byte)
}

/// Applies the `SourceUrl` grammar in the exact precedence frozen by the
/// accepted contract: empty; length; scheme; host; then path.
///
/// Postcondition: `Ok(())` exactly when `value` is exactly
/// `https://<host>/<path>` with `host` admitted by `is_admitted_host` and
/// `path` admitted by `classify_path`, total length `1..=2048` ASCII bytes.
fn classify_source_url(value: &str) -> Result<(), SourceValueErrorKind> {
    if value.is_empty() {
        return Err(SourceValueErrorKind::Empty);
    }
    if value.len() > MAX_SOURCE_URL_BYTES {
        return Err(SourceValueErrorKind::TooLong {
            max_bytes: MAX_SOURCE_URL_BYTES,
        });
    }
    let Some(after_scheme) = strip_https_scheme(value) else {
        return Err(SourceValueErrorKind::InvalidScheme);
    };
    // The host runs to the first `/`. A URL with no `/` after the host has no
    // path and is rejected: the contract requires the path to begin with `/`.
    let Some((host, _)) = after_scheme.split_once('/') else {
        return Err(SourceValueErrorKind::InvalidPath);
    };
    // Userinfo, password, explicit port, query and fragment are forbidden.
    // `@` in the host section is userinfo; `:` is an explicit port; `?` is a
    // query; `#` is a fragment. Any of these is a host violation under the
    // constrained grammar.
    if host.contains(['@', ':', '?', '#']) {
        return Err(SourceValueErrorKind::InvalidHost);
    }
    if !is_admitted_host(host) {
        return Err(SourceValueErrorKind::InvalidHost);
    }
    // `after_scheme[host.len()..]` starts at the `/` that `split_once` found,
    // so it is the full path including the leading `/`.
    let full_path = &after_scheme[host.len()..];
    if !classify_path(full_path) {
        return Err(SourceValueErrorKind::InvalidPath);
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// `SourceMediaType` grammar (§3). Lowercase ASCII `type/subtype`, each side
// `1..=64` bytes, RFC token bytes only, no whitespace, parameter, wildcard or
// structured fallback; total `3..=129` bytes.
// ---------------------------------------------------------------------------

/// True when `byte` is an admitted media-type token byte.
///
/// The set is the RFC token byte set minus the wildcard `*`: the contract
/// forbids wildcard media types, and the simplest deterministic boundary is to
/// exclude the byte entirely rather than re-derive `*/*` special cases.
const fn is_media_token_byte(byte: u8) -> bool {
    byte.is_ascii_lowercase()
        || byte.is_ascii_digit()
        || matches!(
            byte,
            b'!' | b'#'
                | b'$'
                | b'%'
                | b'&'
                | b'\''
                | b'+'
                | b'-'
                | b'.'
                | b'^'
                | b'_'
                | b'`'
                | b'|'
                | b'~'
        )
}

/// Applies the `SourceMediaType` grammar: total length; then exactly one `/`
/// splitting a non-empty lowercase token `type` and `subtype`, each `1..=64`
/// bytes.
///
/// Postcondition: `Ok(())` exactly when `value` is
/// `<type>/<subtype>` with both components admitted by `is_media_token_byte`,
/// total length `3..=129` bytes.
fn classify_media_type(value: &str) -> Result<(), SourceValueErrorKind> {
    if value.len() < 3 || value.len() > MAX_MEDIA_TYPE_BYTES {
        return Err(SourceValueErrorKind::InvalidMediaType);
    }
    let Some((media_type, media_subtype)) = value.split_once('/') else {
        return Err(SourceValueErrorKind::InvalidMediaType);
    };
    // Exactly one `/`: a second slash is a parameter-or-fallback shape.
    if media_subtype.contains('/') {
        return Err(SourceValueErrorKind::InvalidMediaType);
    }
    for component in [media_type, media_subtype] {
        if component.is_empty() || component.len() > MAX_MEDIA_TYPE_COMPONENT_BYTES {
            return Err(SourceValueErrorKind::InvalidMediaType);
        }
        for &byte in component.as_bytes() {
            if !is_media_token_byte(byte) {
                return Err(SourceValueErrorKind::InvalidMediaType);
            }
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Nominal string value generator (§3). Defines one named-field struct with a
// private backing string, one checked `parse`, `as_str`, `Display`,
// `TryFrom<String>`, `TryFrom<&str>`, `FromStr`, validating Serde decode and
// exact Serde string encode. The generator is private: the five kinds below
// are the whole public surface, and no downstream crate may mint a sixth.
// ---------------------------------------------------------------------------

macro_rules! source_value {
    ($(#[$attribute:meta])* $name:ident, $classifier:ident) => {
        $(#[$attribute])*
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
        // A NAMED-FIELD struct, deliberately not a tuple struct. A tuple
        // struct's constructor is a VALUE that can be bound, aliased, passed
        // and returned before it is ever called, so counting construction
        // expressions is not a closure. A named-field struct has no constructor
        // function item at all, leaving a struct literal as the only way to
        // produce one, and a struct literal is syntax that cannot be bound.
        pub struct $name {
            value: String,
        }

        impl $name {
            #[doc = concat!("Parses one canonical `", stringify!($name), "`.")]
            ///
            /// This is the single validator. Every other construction and
            /// deserialization path on this type delegates here, so all of them
            /// share one grammar and one error precedence.
            ///
            /// # Errors
            ///
            /// Returns [`SourceValueError`] when `value` does not match the
            /// accepted grammar for this kind. The error names this kind and
            /// the failing rule and never contains the rejected input.
            pub fn parse(value: impl Into<String>) -> Result<Self, SourceValueError> {
                let value = value.into();
                match $classifier(&value) {
                    Ok(()) => Ok(Self { value }),
                    Err(kind) => Err(SourceValueError {
                        value_kind: stringify!($name),
                        kind,
                    }),
                }
            }

            /// Returns the exact canonical bytes, with case and delimiters preserved.
            #[must_use]
            pub fn as_str(&self) -> &str {
                &self.value
            }
        }

        impl TryFrom<String> for $name {
            type Error = SourceValueError;

            fn try_from(value: String) -> Result<Self, Self::Error> {
                Self::parse(value)
            }
        }

        impl TryFrom<&str> for $name {
            type Error = SourceValueError;

            fn try_from(value: &str) -> Result<Self, Self::Error> {
                Self::parse(value)
            }
        }

        impl FromStr for $name {
            type Err = SourceValueError;

            fn from_str(value: &str) -> Result<Self, Self::Err> {
                Self::parse(value)
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str(&self.value)
            }
        }

        impl Serialize for $name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                serializer.serialize_str(&self.value)
            }
        }

        impl<'de> Deserialize<'de> for $name {
            /// Deserializes the canonical string, then applies the one checked
            /// constructor.
            ///
            /// A hand-written `Visitor` is deliberately NOT used. Every
            /// implemented `visit_*` method is an independent construction
            /// path, so a visitor has to be enumerated and each arm proven to
            /// validate — and the next unenumerated arm (`visit_bytes`,
            /// `visit_borrowed_str`, …) reopens the hole. Deferring to
            /// `String`'s own `Deserialize` leaves exactly one construction
            /// path in this impl: whatever entry point the deserializer
            /// chooses, it produces a `String` that this line then hands to
            /// `parse`.
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                let value = String::deserialize(deserializer)?;
                $name::parse(value).map_err(de::Error::custom)
            }
        }
    };
}

source_value! {
    /// One stable source identity.
    ///
    /// It proves no permission, retrieval authority, baseline membership or
    /// product readiness. Prefixes and delimiters carry no authority meaning.
    ///
    /// The private backing field cannot be constructed directly:
    ///
    /// ```compile_fail
    /// use ustc_campus_agent_core::source_registry::SourceId;
    ///
    /// let id = SourceId { value: String::from("ustc:example") };
    /// ```
    ///
    /// A default identity value does not exist:
    ///
    /// ```compile_fail
    /// use ustc_campus_agent_core::source_registry::SourceId;
    ///
    /// let id = SourceId::default();
    /// ```
    ///
    /// There is no unchecked constructor:
    ///
    /// ```compile_fail
    /// use ustc_campus_agent_core::source_registry::SourceId;
    ///
    /// let id = SourceId::new("ustc:example");
    /// ```
    SourceId,
    classify_source_id
}

source_value! {
    /// One human/governance owner label.
    ///
    /// It is never interpreted as an account, role or permission. It accepts
    /// `1..=128` UTF-8 bytes, rejects leading/trailing whitespace and every
    /// control character, and preserves accepted text exactly.
    ///
    /// The backing string cannot be mutated:
    ///
    /// ```compile_fail
    /// use ustc_campus_agent_core::source_registry::SourceOwner;
    ///
    /// fn rewrite(owner: &mut SourceOwner) {
    ///     owner.as_mut_str().make_ascii_uppercase();
    /// }
    /// ```
    SourceOwner,
    classify_source_owner
}

source_value! {
    /// One exact canonical public-HTTPS URL.
    ///
    /// It is intentionally narrower than a general URL library: scheme is
    /// exactly `https://`; userinfo, password, explicit port, query and
    /// fragment are forbidden; the host is lowercase DNS text with at least two
    /// labels; IP literals, `localhost`, empty labels and a trailing dot are
    /// forbidden; the path begins with `/`, contains no empty/`.`/`..` segment
    /// and is limited to ASCII alphanumeric plus `-._~` and uppercase percent
    /// triplets. No decoding, IDNA normalization, slash folding, dot-segment
    /// removal or percent-case rewriting occurs.
    ///
    /// This constrained value is an exact reviewed endpoint identity, not
    /// permission to fetch arbitrary URLs. M60-B2 still owns DNS/IP, redirect,
    /// content-type, timeout, size and transport enforcement.
    SourceUrl,
    classify_source_url
}

source_value! {
    /// One authenticated reviewer identity supplied by an application boundary.
    ///
    /// It uses the same byte grammar and bound as [`SourceId`] but is nominally
    /// distinct. It proves no reviewer authorization; the application boundary
    /// must authenticate and authorize the reviewer before approval.
    SourceReviewerId,
    classify_source_id
}

source_value! {
    /// One opaque reference to evidence retained by an owning operator or
    /// governance surface.
    ///
    /// It uses the same byte grammar and bound as [`SourceId`] but is nominally
    /// distinct. It neither proves reviewer authorization nor contains the
    /// evidence; it is a reference the operator binds at the application
    /// boundary.
    SourceReviewEvidenceId,
    classify_source_id
}

// ---------------------------------------------------------------------------
// `SourceStatusEvidenceId` (§3). Accepted v1 nominal identifier over the
// `SourceId` grammar with a deliberately reduced trait surface.
// ---------------------------------------------------------------------------

/// One opaque reference to transition evidence retained by an owning operator
/// or governance surface.
///
/// It uses the same byte grammar and bound as [`SourceId`] but is nominally
/// distinct, and it intentionally exposes fewer traits than the five nominal
/// string values: no `Serde`, no `Display`, no `TryFrom`, no `FromStr`, no
/// `Default` and no unchecked constructor. It is never interpreted as a
/// credential, does not contain the evidence and is not self-proving
/// authorization.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SourceStatusEvidenceId {
    value: String,
}

impl SourceStatusEvidenceId {
    /// Builds one status-evidence reference over the `SourceId` grammar.
    ///
    /// # Errors
    ///
    /// Returns [`SourceValueError`] when `value` does not match the
    /// `SourceId`-family grammar. The error names this kind and the failing
    /// rule and never contains the rejected input.
    pub fn new(value: String) -> Result<Self, SourceValueError> {
        match classify_source_id(&value) {
            Ok(()) => Ok(Self { value }),
            Err(kind) => Err(SourceValueError {
                value_kind: "SourceStatusEvidenceId",
                kind,
            }),
        }
    }

    /// Returns the exact canonical bytes, with case and delimiters preserved.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.value
    }

    /// Consumes the identifier and returns the canonical bytes.
    #[must_use]
    pub fn into_inner(self) -> String {
        self.value
    }
}

// ---------------------------------------------------------------------------
// `SourceAuthorityRevision` (§3.1). Non-zero monotone generation counter.
// ---------------------------------------------------------------------------

/// The single current-authority generation for one stable [`SourceId`].
///
/// It is non-zero: initial `propose` — the contract-defined constructor
/// exception — takes no expected revision and initializes revision `1`. Every
/// post-proposal lifecycle mutation (`revise`, `approve`, `suspend`,
/// `reinstate`, `revoke`) requires an exact expected-revision compare-and-swap
/// and increments with checked arithmetic; `u64` overflow is
/// [`SourceRegistryError::RevisionExhausted`]. There is no peer
/// definition/status revision and no reset while the `SourceId` exists.
///
/// The field is private and there is no public constructor, so a caller can
/// only obtain a revision value from a definition or registry:
///
/// ```compile_fail
/// use ustc_campus_agent_core::source_registry::SourceAuthorityRevision;
///
/// let revision = SourceAuthorityRevision { revision: 1 };
/// ```
///
/// A defaulted revision does not exist:
///
/// ```compile_fail
/// use ustc_campus_agent_core::source_registry::SourceAuthorityRevision;
///
/// let revision = SourceAuthorityRevision::default();
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SourceAuthorityRevision {
    revision: u64,
}

impl SourceAuthorityRevision {
    /// Returns the current generation.
    #[must_use]
    pub const fn get(&self) -> u64 {
        self.revision
    }

    /// Returns the next generation, or `None` on `u64` overflow.
    ///
    /// Private: callers never construct revisions; the registry owns the
    /// checked increment on every successful post-proposal mutation.
    fn increment(self) -> Option<Self> {
        let revision = self.revision.checked_add(1)?;
        Some(Self { revision })
    }
}

// ---------------------------------------------------------------------------
// Closed policy-version enums (§4). Exactly one accepted variant each.
// ---------------------------------------------------------------------------

/// The closed retrieval protocol-version inventory.
///
/// Exactly one accepted variant; there is no compatibility, default or caller
/// escape from the closed set.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceRetrievalProtocolVersion {
    /// Strict HTTPS-over-IPv4 with HTTP/1.1 framing.
    V0StrictHttpsIpv4Http11_20260809,
}

/// The closed public-IP policy-version inventory.
///
/// Exactly one accepted variant; there is no compatibility, default or caller
/// escape from the closed set.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PublicIpPolicyVersion {
    /// Public IPv4 addresses only.
    V0Ipv4Only20260809,
}

// ---------------------------------------------------------------------------
// `SourceRetrievalPolicy` (§4). Six bounded operator ceilings consumed by
// `source-retrieval/v0`.
// ---------------------------------------------------------------------------

/// Retrieval-policy metadata whose limits later adapters must enforce.
///
/// Exactly six fields. The three `u32` ceilings are non-zero and bounded;
/// `expected_media_type` is a checked [`SourceMediaType`]; `protocol_version`
/// and `public_ip_policy_version` are the closed one-variant enums. These are
/// operator ceilings consumed by `source-retrieval/v0`, not evidence that an
/// adapter enforced them.
///
/// A policy cannot be defaulted into existence:
///
/// ```compile_fail
/// use ustc_campus_agent_core::source_registry::SourceRetrievalPolicy;
///
/// let policy = SourceRetrievalPolicy::default();
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceRetrievalPolicy {
    minimum_interval_seconds: u32,
    maximum_response_bytes: u32,
    maximum_elapsed_seconds: u32,
    expected_media_type: SourceMediaType,
    protocol_version: SourceRetrievalProtocolVersion,
    public_ip_policy_version: PublicIpPolicyVersion,
}

impl SourceRetrievalPolicy {
    /// Builds a retrieval policy, checking the three `u32` ceilings.
    ///
    /// Precedence: policy fields in declaration order. A zero field is
    /// reported before a too-large field, and an earlier field is reported
    /// before a later one.
    ///
    /// # Errors
    ///
    /// Returns [`SourceValueError`] with
    /// [`SourceValueErrorKind::ZeroMinimumInterval`] when
    /// `minimum_interval_seconds` is zero, or
    /// [`SourceValueErrorKind::MinimumIntervalTooLarge`] when it exceeds
    /// `604_800`; then the equivalent pair for `maximum_response_bytes` against
    /// `1_048_576`; then the equivalent pair for `maximum_elapsed_seconds`
    /// against `60`.
    pub fn new(
        minimum_interval_seconds: u32,
        maximum_response_bytes: u32,
        maximum_elapsed_seconds: u32,
        expected_media_type: SourceMediaType,
        protocol_version: SourceRetrievalProtocolVersion,
        public_ip_policy_version: PublicIpPolicyVersion,
    ) -> Result<Self, SourceValueError> {
        if minimum_interval_seconds == 0 {
            return Err(value_error(
                "SourceRetrievalPolicy",
                SourceValueErrorKind::ZeroMinimumInterval,
            ));
        }
        if minimum_interval_seconds > MAX_MINIMUM_INTERVAL_SECONDS {
            return Err(value_error(
                "SourceRetrievalPolicy",
                SourceValueErrorKind::MinimumIntervalTooLarge {
                    max_seconds: MAX_MINIMUM_INTERVAL_SECONDS,
                },
            ));
        }
        if maximum_response_bytes == 0 {
            return Err(value_error(
                "SourceRetrievalPolicy",
                SourceValueErrorKind::ZeroMaximumResponseBytes,
            ));
        }
        if maximum_response_bytes > MAX_MAXIMUM_RESPONSE_BYTES {
            return Err(value_error(
                "SourceRetrievalPolicy",
                SourceValueErrorKind::MaximumResponseBytesTooLarge {
                    max_bytes: MAX_MAXIMUM_RESPONSE_BYTES,
                },
            ));
        }
        if maximum_elapsed_seconds == 0 {
            return Err(value_error(
                "SourceRetrievalPolicy",
                SourceValueErrorKind::ZeroMaximumElapsedSeconds,
            ));
        }
        if maximum_elapsed_seconds > MAX_MAXIMUM_ELAPSED_SECONDS {
            return Err(value_error(
                "SourceRetrievalPolicy",
                SourceValueErrorKind::MaximumElapsedSecondsTooLarge {
                    max_seconds: MAX_MAXIMUM_ELAPSED_SECONDS,
                },
            ));
        }
        Ok(Self {
            minimum_interval_seconds,
            maximum_response_bytes,
            maximum_elapsed_seconds,
            expected_media_type,
            protocol_version,
            public_ip_policy_version,
        })
    }

    /// Returns the minimum interval between retrievals, in seconds.
    #[must_use]
    pub const fn minimum_interval_seconds(&self) -> u32 {
        self.minimum_interval_seconds
    }

    /// Returns the maximum response body size, in bytes.
    #[must_use]
    pub const fn maximum_response_bytes(&self) -> u32 {
        self.maximum_response_bytes
    }

    /// Returns the maximum elapsed retrieval time, in seconds.
    #[must_use]
    pub const fn maximum_elapsed_seconds(&self) -> u32 {
        self.maximum_elapsed_seconds
    }

    /// Returns the expected media type of the response body.
    #[must_use]
    pub const fn expected_media_type(&self) -> &SourceMediaType {
        &self.expected_media_type
    }

    /// Returns the closed retrieval protocol version.
    #[must_use]
    pub const fn protocol_version(&self) -> SourceRetrievalProtocolVersion {
        self.protocol_version
    }

    /// Returns the closed public-IP policy version.
    #[must_use]
    pub const fn public_ip_policy_version(&self) -> PublicIpPolicyVersion {
        self.public_ip_policy_version
    }
}

// ---------------------------------------------------------------------------
// `SourceMediaType` (§3). Checked `parse`, no `Display`, `TryFrom`, `FromStr`
// or `Serde`; read-only use through policy.
// ---------------------------------------------------------------------------

/// The expected media-type value of one approved source.
///
/// Grammar: lowercase ASCII `type/subtype`, each side `1..=64` bytes, RFC
/// token bytes only, no whitespace, parameter, wildcard or structured
/// fallback; total `3..=129` bytes. There is no `Display`, `TryFrom`,
/// `FromStr` or `Serde`; the only checked constructor is [`SourceMediaType::parse`]
/// and the only read access is through [`SourceRetrievalPolicy::expected_media_type`].
///
/// There is no `FromStr` path:
///
/// ```compile_fail
/// use ustc_campus_agent_core::source_registry::SourceMediaType;
///
/// let media: SourceMediaType = "text/html".parse().expect("parse");
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceMediaType {
    value: String,
}

impl SourceMediaType {
    /// Parses one canonical media type.
    ///
    /// This is the single validator for the type.
    ///
    /// # Errors
    ///
    /// Returns [`SourceValueError`] with
    /// [`SourceValueErrorKind::InvalidMediaType`] when `value` is not an
    /// admitted lowercase `type/subtype` token pair. The error never contains
    /// the rejected input.
    pub fn parse(value: &str) -> Result<Self, SourceValueError> {
        match classify_media_type(value) {
            Ok(()) => Ok(Self {
                value: value.to_owned(),
            }),
            Err(kind) => Err(value_error("SourceMediaType", kind)),
        }
    }

    /// Returns the canonical media-type bytes to sibling policy modules.
    pub(crate) fn as_bytes(&self) -> &[u8] {
        self.value.as_bytes()
    }
}

// ---------------------------------------------------------------------------
// `SourceReviewReceipt` (§4). One reviewer identity and exactly four evidence
// references. No boolean shortcuts, no optional field.
// ---------------------------------------------------------------------------

/// Provenance of one operator review: one reviewer identity and exactly four
/// evidence references.
///
/// A structurally complete receipt can still be false if a caller forged the
/// references; reviewer authentication and authorization are application
/// boundary obligations. The receipt has no boolean shortcuts and no optional
/// field.
///
/// The receipt cannot be defaulted into existence:
///
/// ```compile_fail
/// use ustc_campus_agent_core::source_registry::SourceReviewReceipt;
///
/// let receipt = SourceReviewReceipt::default();
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceReviewReceipt {
    reviewer: SourceReviewerId,
    review: SourceReviewEvidenceId,
    permission: SourceReviewEvidenceId,
    rate: SourceReviewEvidenceId,
    parser_fixture: SourceReviewEvidenceId,
}

impl SourceReviewReceipt {
    /// Builds a review receipt.
    ///
    /// Total: the reviewer and evidence references have already passed their
    /// nominal validators, so there is nothing left to reject.
    #[must_use]
    pub fn new(
        reviewer: SourceReviewerId,
        review: SourceReviewEvidenceId,
        permission: SourceReviewEvidenceId,
        rate: SourceReviewEvidenceId,
        parser_fixture: SourceReviewEvidenceId,
    ) -> Self {
        Self {
            reviewer,
            review,
            permission,
            rate,
            parser_fixture,
        }
    }

    /// Returns the authenticated reviewer identity.
    #[must_use]
    pub fn reviewer(&self) -> &SourceReviewerId {
        &self.reviewer
    }

    /// Returns the evidence reference for the review act.
    #[must_use]
    pub fn review(&self) -> &SourceReviewEvidenceId {
        &self.review
    }

    /// Returns the evidence reference for permission to retrieve.
    #[must_use]
    pub fn permission(&self) -> &SourceReviewEvidenceId {
        &self.permission
    }

    /// Returns the evidence reference for the rate limit.
    #[must_use]
    pub fn rate(&self) -> &SourceReviewEvidenceId {
        &self.rate
    }

    /// Returns the evidence reference for the parser fixture.
    #[must_use]
    pub fn parser_fixture(&self) -> &SourceReviewEvidenceId {
        &self.parser_fixture
    }
}

// ---------------------------------------------------------------------------
// Operational status (§4).
// ---------------------------------------------------------------------------

/// The evidence-free closed kind of one [`SourceStatus`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceStatusKind {
    /// The definition is proposed and not yet approved.
    Proposed,
    /// The definition is currently approved.
    Approved,
    /// New retrieval is blocked while historical evidence is preserved.
    Suspended,
    /// Terminal revocation; no further transition exists.
    Revoked,
}

/// The closed command inventory of the post-proposal lifecycle.
///
/// Initial `propose` is the creation exception and is not a transition
/// command; it takes no expected revision and initializes revision `1`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceTransitionCommand {
    /// Replace the full body and return to `Proposed`.
    Revise,
    /// Approve against a complete review receipt.
    Approve,
    /// Suspend an approved definition, preserving its approval receipt.
    Suspend,
    /// Reinstate a suspended definition against a complete new receipt.
    Reinstate,
    /// Revoke terminally.
    Revoke,
}

/// The operational state of one source definition.
///
/// Every new definition starts as [`SourceStatus::Proposed`] with no
/// revision evidence. Approval, suspension, reinstatement and revocation are
/// explicit registry transitions; no constructor takes a `SourceStatus`, so no
/// caller-built status can enter a definition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SourceStatus {
    /// Proposed. Carries optional reference evidence for the proposal.
    Proposed {
        /// Opaque reference to proposal evidence retained by the operator.
        revision_evidence: Option<SourceStatusEvidenceId>,
    },
    /// Approved against a complete review receipt.
    Approved {
        /// The receipt that authorized this approval.
        receipt: SourceReviewReceipt,
    },
    /// Suspended: new retrieval is blocked while the approval receipt is
    /// preserved.
    Suspended {
        /// The preserved approval receipt.
        approval: SourceReviewReceipt,
        /// Opaque reference to the suspension evidence.
        evidence: SourceStatusEvidenceId,
    },
    /// Terminal revocation.
    Revoked {
        /// The prior approval receipt, when revocation ended an approval or a
        /// suspension that carried one.
        prior_approval: Option<SourceReviewReceipt>,
        /// Opaque reference to the revocation evidence.
        evidence: SourceStatusEvidenceId,
    },
}

impl SourceStatus {
    /// Returns the evidence-free closed kind of this status.
    #[must_use]
    pub const fn kind(&self) -> SourceStatusKind {
        match self {
            Self::Proposed { .. } => SourceStatusKind::Proposed,
            Self::Approved { .. } => SourceStatusKind::Approved,
            Self::Suspended { .. } => SourceStatusKind::Suspended,
            Self::Revoked { .. } => SourceStatusKind::Revoked,
        }
    }
}

// ---------------------------------------------------------------------------
// `SourceDefinitionBody` (§4). The full replacement body for `revise`.
// ---------------------------------------------------------------------------

/// The full non-identity body of one source definition: owner, URL, authority
/// and retrieval policy.
///
/// [`SourceDefinitionBody::new`] is the only full replacement-body
/// constructor. `revise` atomically replaces a definition's body with one of
/// these values while preserving its `SourceId`.
///
/// A body cannot be defaulted into existence:
///
/// ```compile_fail
/// use ustc_campus_agent_core::source_registry::SourceDefinitionBody;
///
/// let body = SourceDefinitionBody::default();
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceDefinitionBody {
    owner: SourceOwner,
    url: SourceUrl,
    authority: SourceAuthority,
    retrieval_policy: SourceRetrievalPolicy,
}

impl SourceDefinitionBody {
    /// Builds a full replacement body.
    ///
    /// # Errors
    ///
    /// Returns [`SourceValueError`] with
    /// [`SourceValueErrorKind::NonSourceAuthority`] when `authority` is
    /// [`SourceAuthority::ModelInference`].
    pub fn new(
        owner: SourceOwner,
        url: SourceUrl,
        authority: SourceAuthority,
        retrieval_policy: SourceRetrievalPolicy,
    ) -> Result<Self, SourceValueError> {
        if matches!(authority, SourceAuthority::ModelInference) {
            return Err(value_error(
                "SourceDefinitionBody",
                SourceValueErrorKind::NonSourceAuthority,
            ));
        }
        Ok(Self {
            owner,
            url,
            authority,
            retrieval_policy,
        })
    }

    /// Returns the human/governance owner label.
    #[must_use]
    pub fn owner(&self) -> &SourceOwner {
        &self.owner
    }

    /// Returns the exact canonical URL.
    #[must_use]
    pub fn url(&self) -> &SourceUrl {
        &self.url
    }

    /// Returns the source authority class.
    #[must_use]
    pub const fn authority(&self) -> SourceAuthority {
        self.authority
    }

    /// Returns the retrieval-policy value.
    #[must_use]
    pub fn retrieval_policy(&self) -> &SourceRetrievalPolicy {
        &self.retrieval_policy
    }
}

// ---------------------------------------------------------------------------
// `SourceDefinition` (§4). The exact v1 aggregate.
// ---------------------------------------------------------------------------

/// One source definition: identity, owner, URL, authority, retrieval policy,
/// authority revision and operational status.
///
/// The only public constructor is [`SourceDefinition::proposed`], which is
/// fallible only because [`SourceAuthority::ModelInference`] is an explanation
/// class, not a source, and must be rejected as
/// [`SourceValueErrorKind::NonSourceAuthority`]. It always produces revision
/// `1` and `Proposed` state with no revision evidence. No constructor takes a
/// `SourceStatus`; no `approved`, `from_parts`, builder, `TryFrom` or `Serde`
/// path may bypass the registry approval transition.
///
/// A definition cannot be defaulted into existence:
///
/// ```compile_fail
/// use ustc_campus_agent_core::source_registry::SourceDefinition;
///
/// let definition = SourceDefinition::default();
/// ```
///
/// …nor decoded from a transport payload, because it has no `Deserialize`:
///
/// ```compile_fail
/// use ustc_campus_agent_core::source_registry::SourceDefinition;
///
/// let definition: SourceDefinition = serde_json::from_str("{}").expect("no Deserialize exists");
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceDefinition {
    source_id: SourceId,
    owner: SourceOwner,
    url: SourceUrl,
    authority: SourceAuthority,
    retrieval_policy: SourceRetrievalPolicy,
    authority_revision: SourceAuthorityRevision,
    status: SourceStatus,
}

impl SourceDefinition {
    /// Builds a proposed source definition at revision `1`.
    ///
    /// This is the only definition constructor. It is fallible only because
    /// [`SourceAuthority::ModelInference`] is an explanation class, not a
    /// source, and must be rejected as
    /// [`SourceValueErrorKind::NonSourceAuthority`]; every other field has
    /// already passed its owning validator. The definition starts as
    /// [`SourceStatus::Proposed`] with no revision evidence; approval and every
    /// later transition are explicit registry mutations guarded by
    /// compare-and-swap.
    ///
    /// # Errors
    ///
    /// Returns [`SourceValueError`] with
    /// [`SourceValueErrorKind::NonSourceAuthority`] when `authority` is
    /// [`SourceAuthority::ModelInference`].
    pub fn proposed(
        source_id: SourceId,
        owner: SourceOwner,
        url: SourceUrl,
        authority: SourceAuthority,
        retrieval_policy: SourceRetrievalPolicy,
    ) -> Result<Self, SourceValueError> {
        if matches!(authority, SourceAuthority::ModelInference) {
            return Err(value_error(
                "SourceDefinition",
                SourceValueErrorKind::NonSourceAuthority,
            ));
        }
        Ok(Self {
            source_id,
            owner,
            url,
            authority,
            retrieval_policy,
            authority_revision: SourceAuthorityRevision { revision: 1 },
            status: SourceStatus::Proposed {
                revision_evidence: None,
            },
        })
    }

    /// Returns the source identity.
    #[must_use]
    pub fn source_id(&self) -> &SourceId {
        &self.source_id
    }

    /// Returns the human/governance owner label.
    #[must_use]
    pub fn owner(&self) -> &SourceOwner {
        &self.owner
    }

    /// Returns the exact canonical URL.
    #[must_use]
    pub fn url(&self) -> &SourceUrl {
        &self.url
    }

    /// Returns the source authority class.
    #[must_use]
    pub const fn authority(&self) -> SourceAuthority {
        self.authority
    }

    /// Returns the retrieval-policy value.
    #[must_use]
    pub fn retrieval_policy(&self) -> &SourceRetrievalPolicy {
        &self.retrieval_policy
    }

    /// Returns the current-authority generation.
    #[must_use]
    pub const fn authority_revision(&self) -> SourceAuthorityRevision {
        self.authority_revision
    }

    /// Returns the operational status.
    #[must_use]
    pub const fn status(&self) -> &SourceStatus {
        &self.status
    }

    /// Returns the prior approval receipt carried by the current status, if
    /// any.
    #[must_use]
    pub fn prior_approval(&self) -> Option<&SourceReviewReceipt> {
        match &self.status {
            SourceStatus::Proposed { .. } => None,
            SourceStatus::Approved { receipt } => Some(receipt),
            SourceStatus::Suspended { approval, .. } => Some(approval),
            SourceStatus::Revoked { prior_approval, .. } => prior_approval.as_ref(),
        }
    }
}

// ---------------------------------------------------------------------------
// `RetrievalSubject` (§6). Sealed owned snapshot from current `Approved` state.
// ---------------------------------------------------------------------------

/// A sealed owned snapshot of one approved source, available only from current
/// [`SourceStatusKind::Approved`] state.
///
/// Fields are private, accessors are read-only, and there is no public
/// unchecked constructor, `Serde` or authority-bearing conversion from
/// [`SourceDefinition`]. A subject is a policy input to `source-retrieval/v0`,
/// not final effect authority; M60-B3 must later re-check the same source ID
/// and authority revision atomically before any network effect.
///
/// There is no public construction path:
///
/// ```compile_fail
/// use ustc_campus_agent_core::source_registry::{RetrievalSubject, SourceId, SourceUrl};
///
/// fn build(source_id: SourceId, source_url: SourceUrl) -> RetrievalSubject {
///     RetrievalSubject { source_id, source_url }
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetrievalSubject {
    source_id: SourceId,
    source_url: SourceUrl,
    source_authority_revision: SourceAuthorityRevision,
    source_retrieval_policy: SourceRetrievalPolicy,
}

impl RetrievalSubject {
    /// Returns the approved source identity.
    #[must_use]
    pub fn source_id(&self) -> &SourceId {
        &self.source_id
    }

    /// Returns the approved canonical URL.
    #[must_use]
    pub fn source_url(&self) -> &SourceUrl {
        &self.source_url
    }

    /// Returns the authority generation this snapshot was taken at.
    #[must_use]
    pub const fn source_authority_revision(&self) -> SourceAuthorityRevision {
        self.source_authority_revision
    }

    /// Returns the approved retrieval policy.
    #[must_use]
    pub fn source_retrieval_policy(&self) -> &SourceRetrievalPolicy {
        &self.source_retrieval_policy
    }
}

// ---------------------------------------------------------------------------
// `SourceRegistryError` (§5).
// ---------------------------------------------------------------------------

/// Why one registry operation failed.
///
/// Variants may carry the `SourceId` or `SourceUrl` that named the failed
/// operation, because source IDs and canonical URLs are public catalog
/// references, not secrets. No variant carries rejected owner text, evidence
/// IDs or a review receipt.
///
/// A failed operation leaves the whole registry structurally unchanged.
///
/// The error cannot be defaulted into existence:
///
/// ```compile_fail
/// use ustc_campus_agent_core::source_registry::SourceRegistryError;
///
/// let error = SourceRegistryError::default();
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SourceRegistryError {
    /// `propose` rejected a duplicate `SourceId` without replacing the first.
    DuplicateSource {
        /// The source ID that was already present.
        source_id: SourceId,
    },
    /// `propose` rejected a duplicate canonical `SourceUrl` without replacing
    /// the first definition.
    DuplicateUrl {
        /// The canonical URL that was already present.
        url: SourceUrl,
    },
    /// A mutation or read rejected a missing `SourceId`.
    SourceNotFound {
        /// The source ID that was not present.
        source_id: SourceId,
    },
    /// A retrievability-gated operation rejected a non-`Approved` entry.
    SourceNotRetrievable {
        /// The source ID whose definition is not `Approved`.
        source_id: SourceId,
        /// The evidence-free kind of the current status.
        status: SourceStatusKind,
    },
    /// `approve` rejected an already-approved ID and preserved the first
    /// receipt.
    SourceAlreadyApproved {
        /// The source ID that was already `Approved`.
        source_id: SourceId,
    },
    /// A mutation rejected an expected revision that does not match the
    /// current authority revision.
    StaleAuthorityRevision {
        /// The caller-supplied expected revision.
        expected: SourceAuthorityRevision,
        /// The current authority revision.
        actual: SourceAuthorityRevision,
    },
    /// A mutation rejected a state/command pair the transition matrix does not
    /// admit.
    IllegalTransition {
        /// The evidence-free kind of the current status.
        status: SourceStatusKind,
        /// The rejected transition command.
        command: SourceTransitionCommand,
    },
    /// A mutation rejected because the authority revision is exhausted at
    /// `u64::MAX`.
    RevisionExhausted {
        /// The source ID whose revision cannot increment further.
        source_id: SourceId,
    },
}

impl fmt::Display for SourceRegistryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateSource { source_id } => {
                write!(formatter, "source already registered: {source_id}")
            }
            Self::DuplicateUrl { url } => {
                write!(formatter, "source URL already registered: {url}")
            }
            Self::SourceNotFound { source_id } => {
                write!(formatter, "source not found: {source_id}")
            }
            Self::SourceNotRetrievable { source_id, status } => {
                write!(
                    formatter,
                    "source is not retrievable in state {status:?}: {source_id}"
                )
            }
            Self::SourceAlreadyApproved { source_id } => {
                write!(formatter, "source already approved: {source_id}")
            }
            Self::StaleAuthorityRevision { expected, actual } => {
                write!(
                    formatter,
                    "stale authority revision: expected {}, actual {}",
                    expected.get(),
                    actual.get()
                )
            }
            Self::IllegalTransition { status, command } => {
                write!(
                    formatter,
                    "illegal transition: {status:?} does not admit {command:?}"
                )
            }
            Self::RevisionExhausted { source_id } => {
                write!(
                    formatter,
                    "source authority revision exhausted: {source_id}"
                )
            }
        }
    }
}

impl Error for SourceRegistryError {}

// ---------------------------------------------------------------------------
// `SourceRegistry` (§5). Pure in-memory `BTreeMap<SourceId, SourceDefinition>`
// with no `Default` and one `new()` constructor. `Clone` is intentionally
// dropped from the historical v0 aggregate.
// ---------------------------------------------------------------------------

/// A pure in-memory source registry.
///
/// Backed by a `BTreeMap<SourceId, SourceDefinition>`. Initial `propose` is
/// creation: it takes no expected revision and admits the definition at
/// revision `1`. Every post-proposal lifecycle mutation — `revise`, `approve`,
/// `suspend`, `reinstate`, `revoke` — requires an exact expected-revision
/// compare-and-swap and increments the revision with checked arithmetic on
/// success. No operation performs I/O, reads time, computes a digest or infers
/// review from source text. Failed operations leave the whole registry
/// structurally unchanged.
///
/// The registry cannot be defaulted into existence:
///
/// ```compile_fail
/// use ustc_campus_agent_core::source_registry::SourceRegistry;
///
/// let registry = SourceRegistry::default();
/// ```
///
/// …nor duplicated, because the registry is one mutable aggregate and `Clone`
/// is intentionally not implemented:
///
/// ```compile_fail
/// use ustc_campus_agent_core::source_registry::SourceRegistry;
///
/// fn duplicate(registry: &SourceRegistry) -> SourceRegistry {
///     registry.clone()
/// }
/// ```
#[derive(Debug, PartialEq, Eq)]
pub struct SourceRegistry {
    definitions: BTreeMap<SourceId, SourceDefinition>,
}

impl SourceRegistry {
    /// Builds an empty registry.
    #[must_use]
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            definitions: BTreeMap::new(),
        }
    }

    /// Admits one proposed source definition at revision `1`.
    ///
    /// This is the creation exception: it takes no expected revision, because
    /// no caller-supplied revision exists before a `SourceId` is admitted. The
    /// registry always canonicalizes the consumed definition to fresh
    /// `Proposed { revision_evidence: None }` at revision `1`; cloning a
    /// definition from another registry never imports its lifecycle authority.
    ///
    /// # Errors
    ///
    /// Returns [`SourceRegistryError::DuplicateSource`] when `definition`'s
    /// `source_id` is already present, or
    /// [`SourceRegistryError::DuplicateUrl`] when its `url` is already present
    /// under any other `SourceId`. Duplicate `SourceId` is checked before
    /// duplicate `SourceUrl`. A failed operation leaves the registry
    /// unchanged.
    pub fn propose(&mut self, mut definition: SourceDefinition) -> Result<(), SourceRegistryError> {
        if self.definitions.contains_key(definition.source_id()) {
            return Err(SourceRegistryError::DuplicateSource {
                source_id: definition.source_id().clone(),
            });
        }
        if self.url_owner(definition.url()).is_some() {
            return Err(SourceRegistryError::DuplicateUrl {
                url: definition.url().clone(),
            });
        }
        definition.authority_revision = SourceAuthorityRevision { revision: 1 };
        definition.status = SourceStatus::Proposed {
            revision_evidence: None,
        };
        self.definitions
            .insert(definition.source_id().clone(), definition);
        Ok(())
    }

    /// Replaces the full body of one definition and returns it to `Proposed`.
    ///
    /// The `SourceId` is preserved; owner, URL, authority and retrieval policy
    /// are replaced as one atomic body; the transition records `Some(evidence)`
    /// and increments the authority revision. Reusing the same source's
    /// current URL is not a duplicate; a canonical URL already owned by
    /// another source is rejected without mutation.
    ///
    /// # Errors
    ///
    /// Returns [`SourceRegistryError::SourceNotFound`] when `source_id` is not
    /// present, [`SourceRegistryError::StaleAuthorityRevision`] when
    /// `expected` does not match the current authority revision,
    /// [`SourceRegistryError::IllegalTransition`] from `Revoked`,
    /// [`SourceRegistryError::DuplicateUrl`] when the replacement URL is
    /// already owned by another source, or
    /// [`SourceRegistryError::RevisionExhausted`] when the revision cannot
    /// increment.
    pub fn revise(
        &mut self,
        source_id: &SourceId,
        expected: SourceAuthorityRevision,
        replacement: SourceDefinitionBody,
        evidence: SourceStatusEvidenceId,
    ) -> Result<&SourceDefinition, SourceRegistryError> {
        let current = self.current(source_id)?;
        if current.authority_revision != expected {
            return Err(SourceRegistryError::StaleAuthorityRevision {
                expected,
                actual: current.authority_revision,
            });
        }
        if matches!(current.status, SourceStatus::Revoked { .. }) {
            return Err(SourceRegistryError::IllegalTransition {
                status: current.status.kind(),
                command: SourceTransitionCommand::Revise,
            });
        }
        if let Some(owner) = self.url_owner(replacement.url())
            && owner != source_id
        {
            return Err(SourceRegistryError::DuplicateUrl {
                url: replacement.url().clone(),
            });
        }
        let Some(new_revision) = current.authority_revision.increment() else {
            return Err(SourceRegistryError::RevisionExhausted {
                source_id: source_id.clone(),
            });
        };
        let definition = self.definitions.get_mut(source_id).expect("verified above");
        definition.owner = replacement.owner;
        definition.url = replacement.url;
        definition.authority = replacement.authority;
        definition.retrieval_policy = replacement.retrieval_policy;
        definition.authority_revision = new_revision;
        definition.status = SourceStatus::Proposed {
            revision_evidence: Some(evidence),
        };
        Ok(definition)
    }

    /// Approves one proposed definition against a complete review receipt.
    ///
    /// # Errors
    ///
    /// Returns [`SourceRegistryError::SourceNotFound`] when `source_id` is not
    /// present, [`SourceRegistryError::StaleAuthorityRevision`] when
    /// `expected` does not match, [`SourceRegistryError::SourceAlreadyApproved`]
    /// when the definition is already `Approved` (the first receipt is
    /// preserved), [`SourceRegistryError::IllegalTransition`] from `Suspended`
    /// or `Revoked`, or [`SourceRegistryError::RevisionExhausted`].
    pub fn approve(
        &mut self,
        source_id: &SourceId,
        expected: SourceAuthorityRevision,
        receipt: SourceReviewReceipt,
    ) -> Result<&SourceDefinition, SourceRegistryError> {
        let current = self.current(source_id)?;
        if current.authority_revision != expected {
            return Err(SourceRegistryError::StaleAuthorityRevision {
                expected,
                actual: current.authority_revision,
            });
        }
        let status_kind = current.status.kind();
        match &current.status {
            SourceStatus::Proposed { .. } => {}
            SourceStatus::Approved { .. } => {
                return Err(SourceRegistryError::SourceAlreadyApproved {
                    source_id: source_id.clone(),
                });
            }
            SourceStatus::Suspended { .. } | SourceStatus::Revoked { .. } => {
                return Err(SourceRegistryError::IllegalTransition {
                    status: status_kind,
                    command: SourceTransitionCommand::Approve,
                });
            }
        }
        let Some(new_revision) = current.authority_revision.increment() else {
            return Err(SourceRegistryError::RevisionExhausted {
                source_id: source_id.clone(),
            });
        };
        let definition = self.definitions.get_mut(source_id).expect("verified above");
        definition.authority_revision = new_revision;
        definition.status = SourceStatus::Approved { receipt };
        Ok(definition)
    }

    /// Suspends one approved definition while preserving its approval receipt.
    ///
    /// # Errors
    ///
    /// Returns [`SourceRegistryError::SourceNotFound`],
    /// [`SourceRegistryError::StaleAuthorityRevision`],
    /// [`SourceRegistryError::IllegalTransition`] from any state other than
    /// `Approved`, or [`SourceRegistryError::RevisionExhausted`].
    pub fn suspend(
        &mut self,
        source_id: &SourceId,
        expected: SourceAuthorityRevision,
        evidence: SourceStatusEvidenceId,
    ) -> Result<&SourceDefinition, SourceRegistryError> {
        let current = self.current(source_id)?;
        if current.authority_revision != expected {
            return Err(SourceRegistryError::StaleAuthorityRevision {
                expected,
                actual: current.authority_revision,
            });
        }
        let status_kind = current.status.kind();
        let approval = match &current.status {
            SourceStatus::Approved { receipt } => receipt.clone(),
            SourceStatus::Proposed { .. }
            | SourceStatus::Suspended { .. }
            | SourceStatus::Revoked { .. } => {
                return Err(SourceRegistryError::IllegalTransition {
                    status: status_kind,
                    command: SourceTransitionCommand::Suspend,
                });
            }
        };
        let Some(new_revision) = current.authority_revision.increment() else {
            return Err(SourceRegistryError::RevisionExhausted {
                source_id: source_id.clone(),
            });
        };
        let definition = self.definitions.get_mut(source_id).expect("verified above");
        definition.authority_revision = new_revision;
        definition.status = SourceStatus::Suspended { approval, evidence };
        Ok(definition)
    }

    /// Reinstates one suspended definition against a complete new receipt.
    ///
    /// The preserved approval receipt is consumed and replaced by the new one.
    ///
    /// # Errors
    ///
    /// Returns [`SourceRegistryError::SourceNotFound`],
    /// [`SourceRegistryError::StaleAuthorityRevision`],
    /// [`SourceRegistryError::IllegalTransition`] from any state other than
    /// `Suspended`, or [`SourceRegistryError::RevisionExhausted`].
    pub fn reinstate(
        &mut self,
        source_id: &SourceId,
        expected: SourceAuthorityRevision,
        receipt: SourceReviewReceipt,
    ) -> Result<&SourceDefinition, SourceRegistryError> {
        let current = self.current(source_id)?;
        if current.authority_revision != expected {
            return Err(SourceRegistryError::StaleAuthorityRevision {
                expected,
                actual: current.authority_revision,
            });
        }
        let status_kind = current.status.kind();
        if !matches!(current.status, SourceStatus::Suspended { .. }) {
            return Err(SourceRegistryError::IllegalTransition {
                status: status_kind,
                command: SourceTransitionCommand::Reinstate,
            });
        }
        let Some(new_revision) = current.authority_revision.increment() else {
            return Err(SourceRegistryError::RevisionExhausted {
                source_id: source_id.clone(),
            });
        };
        let definition = self.definitions.get_mut(source_id).expect("verified above");
        definition.authority_revision = new_revision;
        definition.status = SourceStatus::Approved { receipt };
        Ok(definition)
    }

    /// Revokes one definition terminally.
    ///
    /// Revocation preserves `Some(prior_approval)` when an approval exists —
    /// from `Approved` the current receipt, from `Suspended` the preserved
    /// approval — and carries `None` from `Proposed`. `Revoked` is terminal.
    ///
    /// # Errors
    ///
    /// Returns [`SourceRegistryError::SourceNotFound`],
    /// [`SourceRegistryError::StaleAuthorityRevision`],
    /// [`SourceRegistryError::IllegalTransition`] from `Revoked`, or
    /// [`SourceRegistryError::RevisionExhausted`].
    pub fn revoke(
        &mut self,
        source_id: &SourceId,
        expected: SourceAuthorityRevision,
        evidence: SourceStatusEvidenceId,
    ) -> Result<&SourceDefinition, SourceRegistryError> {
        let current = self.current(source_id)?;
        if current.authority_revision != expected {
            return Err(SourceRegistryError::StaleAuthorityRevision {
                expected,
                actual: current.authority_revision,
            });
        }
        let status_kind = current.status.kind();
        let prior_approval = match &current.status {
            SourceStatus::Proposed { .. } => None,
            SourceStatus::Approved { receipt } => Some(receipt.clone()),
            SourceStatus::Suspended { approval, .. } => Some(approval.clone()),
            SourceStatus::Revoked { .. } => {
                return Err(SourceRegistryError::IllegalTransition {
                    status: status_kind,
                    command: SourceTransitionCommand::Revoke,
                });
            }
        };
        let Some(new_revision) = current.authority_revision.increment() else {
            return Err(SourceRegistryError::RevisionExhausted {
                source_id: source_id.clone(),
            });
        };
        let definition = self.definitions.get_mut(source_id).expect("verified above");
        definition.authority_revision = new_revision;
        definition.status = SourceStatus::Revoked {
            prior_approval,
            evidence,
        };
        Ok(definition)
    }

    /// Returns the definition for `source_id`, if present.
    #[must_use]
    pub fn get(&self, source_id: &SourceId) -> Option<&SourceDefinition> {
        self.definitions.get(source_id)
    }

    /// Returns the approved definition for `source_id`.
    ///
    /// # Errors
    ///
    /// Returns [`SourceRegistryError::SourceNotFound`] when `source_id` is not
    /// present, or [`SourceRegistryError::SourceNotRetrievable`] with the
    /// current status kind when the definition is not `Approved`.
    pub fn approved(&self, source_id: &SourceId) -> Result<&SourceDefinition, SourceRegistryError> {
        let definition = self.current(source_id)?;
        let SourceStatus::Approved { .. } = &definition.status else {
            return Err(SourceRegistryError::SourceNotRetrievable {
                source_id: source_id.clone(),
                status: definition.status.kind(),
            });
        };
        Ok(definition)
    }

    /// Returns a sealed owned snapshot of one approved source.
    ///
    /// # Errors
    ///
    /// Returns [`SourceRegistryError::SourceNotFound`] when `source_id` is not
    /// present, or [`SourceRegistryError::SourceNotRetrievable`] with the
    /// current status kind when the definition is not `Approved`.
    pub fn retrieval_subject(
        &self,
        source_id: &SourceId,
    ) -> Result<RetrievalSubject, SourceRegistryError> {
        let definition = self.current(source_id)?;
        let SourceStatus::Approved { .. } = &definition.status else {
            return Err(SourceRegistryError::SourceNotRetrievable {
                source_id: source_id.clone(),
                status: definition.status.kind(),
            });
        };
        Ok(RetrievalSubject {
            source_id: definition.source_id.clone(),
            source_url: definition.url.clone(),
            source_authority_revision: definition.authority_revision,
            source_retrieval_policy: definition.retrieval_policy.clone(),
        })
    }

    /// Returns the number of registered definitions.
    #[must_use]
    pub fn len(&self) -> usize {
        self.definitions.len()
    }

    /// Returns `true` when no definitions are registered.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.definitions.is_empty()
    }

    /// Returns the definition for `source_id`, or the `SourceNotFound` error.
    fn current(&self, source_id: &SourceId) -> Result<&SourceDefinition, SourceRegistryError> {
        self.definitions
            .get(source_id)
            .ok_or_else(|| SourceRegistryError::SourceNotFound {
                source_id: source_id.clone(),
            })
    }

    /// Returns the `SourceId` that currently owns `url`, if any.
    fn url_owner(&self, url: &SourceUrl) -> Option<&SourceId> {
        self.definitions
            .values()
            .find(|definition| definition.url() == url)
            .map(SourceDefinition::source_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXHAUSTED_REVISION: u64 = u64::MAX;

    /// A shape-valid media type for fixtures.
    fn media() -> SourceMediaType {
        SourceMediaType::parse("text/html").expect("fixture media type")
    }

    /// A shape-valid six-field retrieval policy for fixtures.
    fn policy() -> SourceRetrievalPolicy {
        SourceRetrievalPolicy::new(
            1,
            1,
            1,
            media(),
            SourceRetrievalProtocolVersion::V0StrictHttpsIpv4Http11_20260809,
            PublicIpPolicyVersion::V0Ipv4Only20260809,
        )
        .expect("fixture policy")
    }

    /// A shape-valid review receipt for fixtures.
    fn receipt() -> SourceReviewReceipt {
        SourceReviewReceipt::new(
            SourceReviewerId::parse("reviewer:operator").expect("fixture"),
            SourceReviewEvidenceId::parse("evidence:review").expect("fixture"),
            SourceReviewEvidenceId::parse("evidence:permission").expect("fixture"),
            SourceReviewEvidenceId::parse("evidence:rate").expect("fixture"),
            SourceReviewEvidenceId::parse("evidence:fixture").expect("fixture"),
        )
    }

    /// A shape-valid status-evidence reference for fixtures.
    fn evidence() -> SourceStatusEvidenceId {
        SourceStatusEvidenceId::new(String::from("evidence:status")).expect("fixture")
    }

    /// Inserts one proposed definition and forces its authority revision.
    ///
    /// Module-internal only: the public API correctly cannot construct an
    /// arbitrary revision, which is exactly why the exhaustion case must be
    /// exercised here.
    fn registry_with_revision(revision: u64) -> (SourceRegistry, SourceId) {
        let mut registry = SourceRegistry::new();
        let definition = SourceDefinition::proposed(
            SourceId::parse("example:source").expect("fixture id"),
            SourceOwner::parse("Example Office").expect("fixture owner"),
            SourceUrl::parse("https://example.invalid/calendar").expect("fixture url"),
            crate::SourceAuthority::ReviewedOfficialSource,
            policy(),
        )
        .expect("fixture definition");
        let source_id = definition.source_id().clone();
        registry.propose(definition).expect("propose");
        if revision != 1 {
            let stored = registry.definitions.get_mut(&source_id).expect("present");
            stored.authority_revision = SourceAuthorityRevision { revision };
        }
        (registry, source_id)
    }

    /// Forces the definition's status inside the module-internal registry.
    fn force_status(registry: &mut SourceRegistry, source_id: &SourceId, status: SourceStatus) {
        let stored = registry.definitions.get_mut(source_id).expect("present");
        stored.status = status;
    }

    fn current_revision(
        registry: &SourceRegistry,
        source_id: &SourceId,
    ) -> SourceAuthorityRevision {
        registry
            .get(source_id)
            .expect("present")
            .authority_revision()
    }

    #[test]
    fn revision_overflow_is_revision_exhausted_without_mutation() {
        // approve, revise and revoke from `Proposed` at `u64::MAX`.
        let (mut registry, source_id) = registry_with_revision(EXHAUSTED_REVISION);
        let expected = current_revision(&registry, &source_id);
        let error = registry
            .approve(&source_id, expected, receipt())
            .expect_err("approve must exhaust");
        assert_eq!(
            error,
            SourceRegistryError::RevisionExhausted {
                source_id: source_id.clone()
            }
        );

        let error = registry
            .revise(
                &source_id,
                expected,
                SourceDefinitionBody::new(
                    SourceOwner::parse("Example Office").expect("fixture owner"),
                    SourceUrl::parse("https://example.invalid/other").expect("fixture url"),
                    crate::SourceAuthority::ReviewedOfficialSource,
                    policy(),
                )
                .expect("fixture body"),
                evidence(),
            )
            .expect_err("revise must exhaust");
        assert_eq!(
            error,
            SourceRegistryError::RevisionExhausted {
                source_id: source_id.clone()
            }
        );

        let error = registry
            .revoke(&source_id, expected, evidence())
            .expect_err("revoke must exhaust");
        assert_eq!(
            error,
            SourceRegistryError::RevisionExhausted {
                source_id: source_id.clone()
            }
        );

        let stored = registry.get(&source_id).expect("present");
        assert_eq!(stored.authority_revision().get(), EXHAUSTED_REVISION);
        assert_eq!(stored.status().kind(), SourceStatusKind::Proposed);

        // suspend from `Approved` at `u64::MAX`.
        let (mut registry, source_id) = registry_with_revision(EXHAUSTED_REVISION);
        force_status(
            &mut registry,
            &source_id,
            SourceStatus::Approved { receipt: receipt() },
        );
        let expected = current_revision(&registry, &source_id);
        let error = registry
            .suspend(&source_id, expected, evidence())
            .expect_err("suspend must exhaust");
        assert_eq!(
            error,
            SourceRegistryError::RevisionExhausted {
                source_id: source_id.clone()
            }
        );
        let stored = registry.get(&source_id).expect("present");
        assert_eq!(stored.authority_revision().get(), EXHAUSTED_REVISION);
        assert_eq!(stored.status().kind(), SourceStatusKind::Approved);

        // reinstate from `Suspended` at `u64::MAX`.
        let (mut registry, source_id) = registry_with_revision(EXHAUSTED_REVISION);
        force_status(
            &mut registry,
            &source_id,
            SourceStatus::Suspended {
                approval: receipt(),
                evidence: evidence(),
            },
        );
        let expected = current_revision(&registry, &source_id);
        let error = registry
            .reinstate(&source_id, expected, receipt())
            .expect_err("reinstate must exhaust");
        assert_eq!(
            error,
            SourceRegistryError::RevisionExhausted {
                source_id: source_id.clone()
            }
        );
        let stored = registry.get(&source_id).expect("present");
        assert_eq!(stored.authority_revision().get(), EXHAUSTED_REVISION);
        assert_eq!(stored.status().kind(), SourceStatusKind::Suspended);
    }
}
