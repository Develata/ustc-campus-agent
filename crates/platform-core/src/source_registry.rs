//! The pure `source-import/v0` M60-B1 source-registry kernel.
//!
//! Owns the typed boundary between a reviewed source catalog and later retrieval,
//! parsing, revision and baseline adapters (M60-B2 onward). It defines stable
//! source identity, owner, authority class and exact canonical URL; proposed
//! versus approved review state; retrieval-budget metadata whose limits later
//! adapters must enforce; and the in-memory registry that admits, approves and
//! looks up definitions.
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

// ---------------------------------------------------------------------------
// Value-error taxonomy (§7).
// ---------------------------------------------------------------------------

/// Which grammar rule rejected a candidate `source-import/v0` value.
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
    /// `SourceOwner`: the value began or ended with whitespace.
    OwnerBoundaryWhitespace,
    /// `SourceOwner`: the value contained a control character.
    OwnerControlCharacter {
        /// Zero-based index of the first offending byte within the rejected UTF-8 bytes.
        byte_index: usize,
    },
    /// `SourceDefinition::proposed`: the authority was `ModelInference`.
    ///
    /// An explanation class cannot become a source definition or approval
    /// candidate. The variant carries no payload: the rejected authority is a
    /// public enum variant, not caller-supplied text, and naming it would
    /// duplicate the Rust type system rather than protect a secret.
    NonSourceAuthority,
}

/// Why one `source-import/v0` construction failed.
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
const fn value_error(value_kind: &'static str, kind: SourceValueErrorKind) -> SourceValueError {
    SourceValueError { value_kind, kind }
}

// ---------------------------------------------------------------------------
// `SourceId`-family grammar (§3.1, §3.3). Shared by `SourceId`,
// `SourceReviewerId` and `SourceReviewEvidenceId`: identical byte grammar and
// bound, nominally distinct types.
// ---------------------------------------------------------------------------

/// Boundary bytes of a `SourceId`-family value are lowercase ASCII alphanumeric.
const fn is_source_id_boundary(byte: u8) -> bool {
    byte.is_ascii_lowercase() || byte.is_ascii_digit()
}

/// Interior bytes of a `SourceId`-family value add the four delimiters.
const fn is_source_id_interior(byte: u8) -> bool {
    byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'-' | b'.' | b'_' | b':')
}

/// Applies the `SourceId`-family grammar in the exact precedence frozen by
/// `source-import/v0` §7: empty; length; then first byte; then interior
/// left-to-right; then final byte.
///
/// Postcondition: `Ok(())` exactly when `value` matches
/// `^[a-z0-9](?:[-a-z0-9._:]{0,126}[a-z0-9])?$` and has `1..=128` bytes.
/// Invariant: exactly one left-to-right pass over the interior, and no
/// allocation.
fn classify_source_id(value: &str) -> Result<(), SourceValueErrorKind> {
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
// `SourceOwner` grammar (§3.2). 1..=128 UTF-8 bytes, rejects leading/trailing
// whitespace and every control character, preserves accepted text exactly.
// ---------------------------------------------------------------------------

/// Applies the `SourceOwner` grammar in the exact precedence frozen by §7:
/// empty; length; then boundary whitespace; then control characters left to
/// right.
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

/// Applies the `SourceUrl` grammar in the exact precedence frozen by §7:
/// empty; length; scheme; host; then path.
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
        // A NAMED-FIELD struct, deliberately not a tuple struct. See the
        // identity module for the full rationale: a tuple struct's constructor
        // is a VALUE that can be bound, aliased, passed and returned before it
        // is ever called, so counting construction expressions is not a
        // closure. A named-field struct has no constructor function item at
        // all, leaving a struct literal as the only way to produce one, and a
        // struct literal is syntax that cannot be bound.
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
            /// `source-import/v0` grammar for this kind. The error names this
            /// kind and the failing rule and never contains the rejected input.
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
// `SourceRetrievalPolicy` (§4). Both fields non-zero, bounded operator
// ceilings consumed by M60-B2.
// ---------------------------------------------------------------------------

/// Retrieval-budget metadata whose limits later adapters must enforce.
///
/// Both fields are non-zero. `minimum_interval_seconds <= 604_800`;
/// `maximum_response_bytes <= 1_048_576`. These are operator ceilings consumed
/// by M60-B2, not evidence that an adapter enforced them.
///
/// A policy cannot be defaulted into existence:
///
/// ```compile_fail
/// use ustc_campus_agent_core::source_registry::SourceRetrievalPolicy;
///
/// let policy = SourceRetrievalPolicy::default();
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceRetrievalPolicy {
    minimum_interval_seconds: u32,
    maximum_response_bytes: u32,
}

impl SourceRetrievalPolicy {
    /// Builds a retrieval policy, checking both bounds.
    ///
    /// Precedence (§7): `minimum_interval_seconds` is checked before
    /// `maximum_response_bytes`. A zero minimum is reported before a too-large
    /// minimum, and a zero maximum before a too-large maximum.
    ///
    /// # Errors
    ///
    /// Returns [`SourceValueError`] with
    /// [`SourceValueErrorKind::ZeroMinimumInterval`] when
    /// `minimum_interval_seconds` is zero, or
    /// [`SourceValueErrorKind::MinimumIntervalTooLarge`] when it exceeds
    /// `604_800`; then the equivalent pair for `maximum_response_bytes` against
    /// `1_048_576`.
    pub fn new(
        minimum_interval_seconds: u32,
        maximum_response_bytes: u32,
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
        Ok(Self {
            minimum_interval_seconds,
            maximum_response_bytes,
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
// `SourceReviewState` (§4). Bounded B1 review-admission state only: not the
// complete operational `SourceStatus`.
// ---------------------------------------------------------------------------

/// The bounded B1 review-admission state of one source definition.
///
/// This is not the blueprint's complete operational `SourceStatus`: `Suspended`
/// and `Revoked`, with their evidence-bearing transitions, must be accepted
/// before any live M60-B2 retrieval adapter may consume an approved definition.
/// B1 exposes no retrieval, so this deferral cannot leave a fetch path active.
///
/// Every new definition starts as `Proposed`. Approval is an explicit registry
/// transition that consumes a complete receipt. There is no constructor for an
/// already-approved definition and no implicit approval based on host suffix,
/// owner text, authority rank or fixture presence.
///
/// The state cannot be defaulted into existence:
///
/// ```compile_fail
/// use ustc_campus_agent_core::source_registry::SourceReviewState;
///
/// let state = SourceReviewState::default();
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SourceReviewState {
    /// The definition has been proposed but not yet approved.
    Proposed,
    /// The definition has been approved against a complete review receipt.
    Approved {
        /// The receipt that authorized this approval.
        receipt: SourceReviewReceipt,
    },
}

// ---------------------------------------------------------------------------
// `SourceDefinition` (§4). The exact B1 aggregate.
// ---------------------------------------------------------------------------

/// One source definition: identity, owner, URL, authority, retrieval policy
/// and review state.
///
/// The only public constructor is [`SourceDefinition::proposed`], which is
/// fallible only because `SourceAuthority::ModelInference` is an explanation
/// class, not a source, and must be rejected as `NonSourceAuthority`. Every
/// other field has already passed its owning validator. No constructor takes
/// `SourceReviewState`; no `approved`, `from_parts`, builder, `TryFrom` or
/// Serde path may bypass the registry approval transition.
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
    review_state: SourceReviewState,
}

impl SourceDefinition {
    /// Builds a proposed source definition.
    ///
    /// This is the only definition constructor. It is fallible only because
    /// [`SourceAuthority::ModelInference`] is an explanation class, not a
    /// source, and must be rejected as
    /// [`SourceValueErrorKind::NonSourceAuthority`]; every other field has
    /// already passed its owning validator. The definition starts as
    /// [`SourceReviewState::Proposed`]; approval is an explicit registry
    /// transition.
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
            review_state: SourceReviewState::Proposed,
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
    pub fn authority(&self) -> SourceAuthority {
        self.authority
    }

    /// Returns the retrieval-budget policy.
    #[must_use]
    pub fn retrieval_policy(&self) -> SourceRetrievalPolicy {
        self.retrieval_policy
    }

    /// Returns the review-admission state.
    #[must_use]
    pub fn review_state(&self) -> &SourceReviewState {
        &self.review_state
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
/// A failed operation leaves the whole registry byte-for-byte/structurally
/// unchanged.
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
    /// `approve` or `approved` rejected a missing `SourceId`.
    SourceNotFound {
        /// The source ID that was not present.
        source_id: SourceId,
    },
    /// `approved` rejected a `Proposed` entry.
    SourceNotApproved {
        /// The source ID whose definition is still `Proposed`.
        source_id: SourceId,
    },
    /// `approve` rejected an already-approved ID and preserved the first
    /// receipt.
    SourceAlreadyApproved {
        /// The source ID that was already `Approved`.
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
            Self::SourceNotApproved { source_id } => {
                write!(formatter, "source not approved: {source_id}")
            }
            Self::SourceAlreadyApproved { source_id } => {
                write!(formatter, "source already approved: {source_id}")
            }
        }
    }
}

impl Error for SourceRegistryError {}

// ---------------------------------------------------------------------------
// `SourceRegistry` (§5). Pure in-memory `BTreeMap<SourceId, SourceDefinition>`
// with no `Default` and one `new()` constructor.
// ---------------------------------------------------------------------------

/// A pure in-memory source registry.
///
/// Backed by a `BTreeMap<SourceId, SourceDefinition>`. It owns six operations
/// only: `propose`, `approve`, `get`, `approved`, `len` and `is_empty`. No
/// operation performs I/O, reads time, computes a digest or infers review from
/// source text. Failed operations leave the whole registry byte-for-byte /
/// structurally unchanged.
///
/// The registry cannot be defaulted into existence:
///
/// ```compile_fail
/// use ustc_campus_agent_core::source_registry::SourceRegistry;
///
/// let registry = SourceRegistry::default();
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
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

    /// Admits one proposed source definition.
    ///
    /// Rejects a duplicate `SourceId` or duplicate canonical `SourceUrl`
    /// without replacing the first definition. A failed operation leaves the
    /// registry unchanged.
    ///
    /// # Errors
    ///
    /// Returns [`SourceRegistryError::DuplicateSource`] when `definition`'s
    /// `source_id` is already present, or [`SourceRegistryError::DuplicateUrl`]
    /// when its `url` is already present under any other `SourceId`.
    pub fn propose(&mut self, definition: SourceDefinition) -> Result<(), SourceRegistryError> {
        if self.definitions.contains_key(definition.source_id()) {
            return Err(SourceRegistryError::DuplicateSource {
                source_id: definition.source_id().clone(),
            });
        }
        for existing in self.definitions.values() {
            if existing.url() == definition.url() {
                return Err(SourceRegistryError::DuplicateUrl {
                    url: definition.url().clone(),
                });
            }
        }
        self.definitions
            .insert(definition.source_id().clone(), definition);
        Ok(())
    }

    /// Approves one proposed source definition against a complete receipt.
    ///
    /// Rejects a missing ID, an already-approved ID (preserving the first
    /// receipt), and leaves the registry unchanged on any failure.
    ///
    /// # Errors
    ///
    /// Returns [`SourceRegistryError::SourceNotFound`] when `source_id` is not
    /// present, or [`SourceRegistryError::SourceAlreadyApproved`] when it is
    /// already `Approved` (the first receipt is preserved).
    pub fn approve(
        &mut self,
        source_id: &SourceId,
        review_receipt: SourceReviewReceipt,
    ) -> Result<(), SourceRegistryError> {
        let Some(definition) = self.definitions.get_mut(source_id) else {
            return Err(SourceRegistryError::SourceNotFound {
                source_id: source_id.clone(),
            });
        };
        if matches!(definition.review_state, SourceReviewState::Approved { .. }) {
            return Err(SourceRegistryError::SourceAlreadyApproved {
                source_id: source_id.clone(),
            });
        }
        definition.review_state = SourceReviewState::Approved {
            receipt: review_receipt,
        };
        Ok(())
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
    /// present, or [`SourceRegistryError::SourceNotApproved`] when its
    /// definition is still `Proposed`.
    pub fn approved(&self, source_id: &SourceId) -> Result<&SourceDefinition, SourceRegistryError> {
        let Some(definition) = self.definitions.get(source_id) else {
            return Err(SourceRegistryError::SourceNotFound {
                source_id: source_id.clone(),
            });
        };
        if matches!(definition.review_state, SourceReviewState::Proposed) {
            return Err(SourceRegistryError::SourceNotApproved {
                source_id: source_id.clone(),
            });
        }
        Ok(definition)
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
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A shape-valid `SourceId` for fixtures.
    fn id(value: &str) -> SourceId {
        SourceId::parse(value).expect("fixture source id")
    }

    /// A shape-valid `SourceOwner` for fixtures.
    fn owner(value: &str) -> SourceOwner {
        SourceOwner::parse(value).expect("fixture owner")
    }

    /// A shape-valid `SourceUrl` for fixtures.
    fn url(value: &str) -> SourceUrl {
        SourceUrl::parse(value).expect("fixture url")
    }

    /// A shape-valid `SourceRetrievalPolicy` for fixtures.
    fn policy() -> SourceRetrievalPolicy {
        SourceRetrievalPolicy::new(21_600, 131_072).expect("fixture policy")
    }

    /// A shape-valid `SourceReviewReceipt` for fixtures.
    fn receipt() -> SourceReviewReceipt {
        SourceReviewReceipt::new(
            SourceReviewerId::parse("reviewer:operator").expect("fixture"),
            SourceReviewEvidenceId::parse("evidence:review").expect("fixture"),
            SourceReviewEvidenceId::parse("evidence:permission").expect("fixture"),
            SourceReviewEvidenceId::parse("evidence:rate").expect("fixture"),
            SourceReviewEvidenceId::parse("evidence:fixture").expect("fixture"),
        )
    }

    #[test]
    fn propose_then_approve_round_trips() {
        let mut registry = SourceRegistry::new();
        assert!(registry.is_empty());
        let definition = SourceDefinition::proposed(
            id("example:source"),
            owner("Example Source Office"),
            url("https://example.com/calendar"),
            SourceAuthority::ReviewedOfficialSource,
            policy(),
        )
        .expect("fixture definition");
        registry.propose(definition.clone()).expect("propose");
        assert_eq!(registry.len(), 1);
        assert!(matches!(
            registry
                .get(&id("example:source"))
                .expect("present")
                .review_state(),
            SourceReviewState::Proposed
        ));
        assert!(matches!(
            registry.approved(&id("example:source")),
            Err(SourceRegistryError::SourceNotApproved { .. })
        ));
        registry
            .approve(&id("example:source"), receipt())
            .expect("approve");
        let approved = registry.approved(&id("example:source")).expect("approved");
        assert!(matches!(
            approved.review_state(),
            SourceReviewState::Approved { .. }
        ));
    }
}
