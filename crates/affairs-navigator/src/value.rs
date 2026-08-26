//! Checked M71 nominal value types and the shared `AffairsValueError`.
//!
//! Every public nominal type in this module owns a private backing field and a
//! single checked constructor. There is no `Default`, no unchecked `new` that
//! skips validation, and no `Serialize`/`Deserialize`: the M71 public algebra
//! is a canonical domain carrier with no Serde (`docs/contracts/` M71-v8n
//! §11.1). M10 owns the wire DTO and converts exactly once through the
//! conversion-ready accessors exposed by the M71 application service.
//!
//! The M71 ID grammar is `^[a-z0-9](?:[-a-z0-9._:]{0,126}[a-z0-9])?$` over
//! `1..=128` ASCII bytes, identical in shape to the M60 `SourceId` grammar but
//! nominally distinct: M71 owns its own value types and does not import M60
//! source-registry types (D8 split — `M60RevisionRef` is an equal-contract
//! M71-owned fake carrier).

use std::error::Error;
use std::fmt;
use std::str::FromStr;

/// Maximum encoded length, in bytes, of an M71 ID-grammar value.
pub(crate) const MAX_ID_BYTES: usize = 128;

/// Maximum UTF-8 byte length of a `Title`.
pub(crate) const MAX_TITLE_BYTES: usize = 256;

/// Maximum UTF-8 byte length of an `AudienceTag`.
pub(crate) const MAX_AUDIENCE_TAG_BYTES: usize = 64;

/// Maximum UTF-8 byte length of a `Prerequisite.condition` / step instruction.
pub(crate) const MAX_INSTRUCTION_BYTES: usize = 4096;

/// Maximum UTF-8 byte length of a `Deadline.label` / `EntryPoint.label`.
pub(crate) const MAX_LABEL_BYTES: usize = 256;

/// Maximum UTF-8 byte length of a `Contact.name`.
pub(crate) const MAX_NAME_BYTES: usize = 128;

/// Maximum UTF-8 byte length of a `Contact.channel`.
pub(crate) const MAX_CHANNEL_BYTES: usize = 64;

/// Which M71 value rule rejected a candidate.
///
/// No variant carries the rejected input, a fragment of it, or the offending
/// byte. The error is `Copy` so it never retains rejected text on the stack or
/// heap.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AffairsValueErrorKind {
    /// The candidate had zero bytes.
    Empty,
    /// The candidate exceeded the fixed encoded-length bound.
    TooLong { max_bytes: usize },
    /// The first byte was not admitted by the value-specific boundary rule.
    InvalidStart,
    /// An interior byte was not admitted by the value-specific grammar.
    InvalidCharacter { byte_index: usize },
    /// The final byte was not admitted by the value-specific boundary rule.
    InvalidEnd,
    /// A URL value did not begin with `http://` or `https://`.
    InvalidScheme,
    /// A URL value had an empty host.
    InvalidHost,
    /// A cross-field invariant failed (for example `from > to`).
    InvalidRange,
}

/// Why one M71 nominal value construction failed.
///
/// The error names the Rust value kind that rejected the input and the grammar
/// rule that rejected it. It has no `source`, so no rejected input can be
/// reached by walking the error chain.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AffairsValueError {
    value_kind: &'static str,
    kind: AffairsValueErrorKind,
}

impl AffairsValueError {
    /// Returns the Rust type name of the value kind that rejected the input.
    #[must_use]
    pub const fn value_kind(&self) -> &'static str {
        self.value_kind
    }

    /// Returns the grammar rule that rejected the input.
    #[must_use]
    pub const fn kind(&self) -> AffairsValueErrorKind {
        self.kind
    }
}

impl fmt::Display for AffairsValueError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value_kind = self.value_kind;
        match self.kind {
            AffairsValueErrorKind::Empty => {
                write!(formatter, "{value_kind} rejected: value is empty")
            }
            AffairsValueErrorKind::TooLong { max_bytes } => {
                write!(
                    formatter,
                    "{value_kind} rejected: encoded length exceeds {max_bytes} bytes"
                )
            }
            AffairsValueErrorKind::InvalidStart => {
                write!(
                    formatter,
                    "{value_kind} rejected: first byte is not admitted"
                )
            }
            AffairsValueErrorKind::InvalidCharacter { byte_index } => {
                write!(
                    formatter,
                    "{value_kind} rejected: byte {byte_index} is not admitted"
                )
            }
            AffairsValueErrorKind::InvalidEnd => {
                write!(
                    formatter,
                    "{value_kind} rejected: final byte is not admitted"
                )
            }
            AffairsValueErrorKind::InvalidScheme => {
                write!(formatter, "{value_kind} rejected: scheme is not http(s)://")
            }
            AffairsValueErrorKind::InvalidHost => {
                write!(formatter, "{value_kind} rejected: host is empty")
            }
            AffairsValueErrorKind::InvalidRange => {
                write!(formatter, "{value_kind} rejected: range invariant violated")
            }
        }
    }
}

impl Error for AffairsValueError {}

pub(crate) const fn value_error(
    value_kind: &'static str,
    kind: AffairsValueErrorKind,
) -> AffairsValueError {
    AffairsValueError { value_kind, kind }
}

/// Boundary bytes of an M71 ID-grammar value are lowercase ASCII alphanumeric.
const fn is_id_boundary(byte: u8) -> bool {
    byte.is_ascii_lowercase() || byte.is_ascii_digit()
}

/// Interior bytes of an M71 ID-grammar value add the four delimiters.
const fn is_id_interior(byte: u8) -> bool {
    byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'-' | b'.' | b'_' | b':')
}

/// Applies the M71 ID grammar: `1..=128` ASCII bytes matching
/// `^[a-z0-9](?:[-a-z0-9._:]{0,126}[a-z0-9])?$`.
pub(crate) fn classify_id(value: &str) -> Result<(), AffairsValueErrorKind> {
    let bytes = value.as_bytes();
    let Some((&first, after_first)) = bytes.split_first() else {
        return Err(AffairsValueErrorKind::Empty);
    };
    if bytes.len() > MAX_ID_BYTES {
        return Err(AffairsValueErrorKind::TooLong {
            max_bytes: MAX_ID_BYTES,
        });
    }
    if !is_id_boundary(first) {
        return Err(AffairsValueErrorKind::InvalidStart);
    }
    let Some((&last, interior)) = after_first.split_last() else {
        return Ok(());
    };
    for (offset, &byte) in interior.iter().enumerate() {
        if !is_id_interior(byte) {
            return Err(AffairsValueErrorKind::InvalidCharacter {
                byte_index: offset + 1,
            });
        }
    }
    if !is_id_boundary(last) {
        return Err(AffairsValueErrorKind::InvalidEnd);
    }
    Ok(())
}

/// Applies a UTF-8 byte-bound grammar: `1..=max_bytes` UTF-8 bytes, non-empty.
pub(crate) fn classify_utf8(value: &str, max_bytes: usize) -> Result<(), AffairsValueErrorKind> {
    if value.is_empty() {
        return Err(AffairsValueErrorKind::Empty);
    }
    if value.len() > max_bytes {
        return Err(AffairsValueErrorKind::TooLong { max_bytes });
    }
    Ok(())
}

/// Generates one named-field nominal ID-grammar value with a private backing
/// string, a single checked `parse` constructor, `as_str`, `Display`,
/// `TryFrom<String>`, `TryFrom<&str>`, `FromStr`, and `Clone + Debug + Eq + Ord
/// + Hash`. A named-field struct (not a tuple struct) leaves a struct literal
/// as the only construction path; the constructor is not a bindable function
/// item that could be aliased before validation.
#[expect(clippy::doc_lazy_continuation, reason = "multi-line prose, not a list")]
macro_rules! id_value {
    ($(#[$attribute:meta])* $name:ident) => {
        $(#[$attribute])*
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name {
            value: String,
        }

        impl $name {
            /// Parses one canonical value against the M71 ID grammar.
            ///
            /// # Errors
            ///
            /// Returns [`AffairsValueError`] when `value` does not match the
            /// M71 ID grammar. The error names this kind and the failing rule
            /// and never contains the rejected input.
            pub fn parse(value: impl Into<String>) -> Result<Self, AffairsValueError> {
                let value = value.into();
                match classify_id(&value) {
                    Ok(()) => Ok(Self { value }),
                    Err(kind) => Err(value_error(stringify!($name), kind)),
                }
            }

            /// Returns the exact canonical bytes.
            #[must_use]
            pub fn as_str(&self) -> &str {
                &self.value
            }
        }

        impl TryFrom<String> for $name {
            type Error = AffairsValueError;
            fn try_from(value: String) -> Result<Self, Self::Error> {
                Self::parse(value)
            }
        }

        impl TryFrom<&str> for $name {
            type Error = AffairsValueError;
            fn try_from(value: &str) -> Result<Self, Self::Error> {
                Self::parse(value)
            }
        }

        impl FromStr for $name {
            type Err = AffairsValueError;
            fn from_str(value: &str) -> Result<Self, Self::Err> {
                Self::parse(value)
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str(&self.value)
            }
        }
    };
}

id_value! {
    /// One stable procedure identity. Proves no presence, freshness or
    /// publication state; the repository owns that authority.
    ProcedureId
}
id_value! {
    /// One stable artifact identity. Proves no supersession state; the
    /// publication state owns that authority.
    ArtifactId
}
id_value! {
    /// One stable board identity scoping a procedure and its policy.
    BoardId
}
id_value! {
    /// One M60 source identity carried by an equal-contract `M60RevisionRef`.
    /// Opaque catalog label; not a canonical M60 `SourceRevisionId`.
    SourceId
}
id_value! {
    /// One contact reference resolved within a parent contact list. Uses the
    /// M71 ID grammar and is nominally distinct from a `Contact.role`.
    ContactRef
}
id_value! {
    /// One M71 materialization receipt identity sealing an evidence-lineage
    /// receipt. Opaque; never carries raw M60 revision bytes.
    MaterializationReceiptId
}
id_value! {
    /// One opaque actor reference (M00-owned in production; an equal-contract
    /// M71-owned fixture carrier in this spike). Never appears in the public
    /// projection.
    ActorRef
}

/// Generates one named-field UTF-8 bounded text value with a private backing
/// string and a single checked `new` constructor over a byte bound.
macro_rules! text_value {
    ($(#[$attribute:meta])* $name:ident, $max:expr) => {
        $(#[$attribute])*
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name {
            value: String,
        }

        impl $name {
            /// Builds one bounded UTF-8 text value.
            ///
            /// # Errors
            ///
            /// Returns [`AffairsValueError`] when `value` is empty or exceeds
            /// the fixed byte bound.
            pub fn new(value: impl Into<String>) -> Result<Self, AffairsValueError> {
                let value = value.into();
                match classify_utf8(&value, $max) {
                    Ok(()) => Ok(Self { value }),
                    Err(kind) => Err(value_error(stringify!($name), kind)),
                }
            }

            /// Returns the exact text.
            #[must_use]
            pub fn as_str(&self) -> &str {
                &self.value
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str(&self.value)
            }
        }
    };
}

text_value! {
    /// One procedure title. `1..=256` UTF-8 bytes.
    Title,
    MAX_TITLE_BYTES
}
text_value! {
    /// One audience tag. `1..=64` UTF-8 bytes (EXACT mirror of the canonical
    /// bound; M10 MUST NOT widen it to 128).
    AudienceTag,
    MAX_AUDIENCE_TAG_BYTES
}
text_value! {
    /// One prerequisite condition. `1..=4096` UTF-8 bytes.
    PrerequisiteCondition,
    MAX_INSTRUCTION_BYTES
}
text_value! {
    /// One procedure step instruction. `1..=4096` UTF-8 bytes.
    Instruction,
    MAX_INSTRUCTION_BYTES
}
text_value! {
    /// One deadline label. `1..=256` UTF-8 bytes.
    DeadlineLabel,
    MAX_LABEL_BYTES
}
text_value! {
    /// One entry-point label. `1..=256` UTF-8 bytes.
    EntryPointLabel,
    MAX_LABEL_BYTES
}
text_value! {
    /// One contact display name. `1..=128` UTF-8 bytes.
    ContactName,
    MAX_NAME_BYTES
}
text_value! {
    /// One contact channel label. `1..=64` UTF-8 bytes.
    ContactChannel,
    MAX_CHANNEL_BYTES
}

/// One checked absolute URL. Intentionally narrower than a general URL library:
/// the scheme is exactly `http://` or `https://` and the host is non-empty. No
/// decoding, IDNA, or normalization occurs. This is a fixture-grade bound; the
/// M71 v0 algebra exercises the projection, not URL fetch authority.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Url {
    value: String,
}

impl Url {
    /// Builds one checked absolute URL.
    ///
    /// # Errors
    ///
    /// Returns [`AffairsValueError`] when `value` is empty, lacks an
    /// `http(s)://` scheme, or has an empty host.
    pub fn new(value: impl Into<String>) -> Result<Self, AffairsValueError> {
        let value = value.into();
        if value.is_empty() {
            return Err(value_error("Url", AffairsValueErrorKind::Empty));
        }
        if value.len() > 2048 {
            return Err(value_error(
                "Url",
                AffairsValueErrorKind::TooLong { max_bytes: 2048 },
            ));
        }
        let after_scheme = value
            .strip_prefix("https://")
            .or_else(|| value.strip_prefix("http://"))
            .ok_or_else(|| value_error("Url", AffairsValueErrorKind::InvalidScheme))?;
        let host = after_scheme.split(['/', '?', '#']).next().unwrap_or("");
        if host.is_empty() {
            return Err(value_error("Url", AffairsValueErrorKind::InvalidHost));
        }
        Ok(Self { value })
    }

    /// Returns the exact URL text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.value
    }
}

impl fmt::Display for Url {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.value)
    }
}

/// One board policy version, a non-zero monotone counter owned by board policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BoardPolicyVersion(u64);

impl BoardPolicyVersion {
    /// Builds one board policy version. The value MUST be non-zero.
    ///
    /// # Errors
    ///
    /// Returns [`AffairsValueError`] when `version` is zero.
    pub fn new(version: u64) -> Result<Self, AffairsValueError> {
        if version == 0 {
            return Err(value_error(
                "BoardPolicyVersion",
                AffairsValueErrorKind::Empty,
            ));
        }
        Ok(Self(version))
    }

    /// Returns the raw version counter.
    #[must_use]
    pub const fn as_u64(self) -> u64 {
        self.0
    }
}

impl fmt::Display for BoardPolicyVersion {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.0)
    }
}

/// One procedure-level effective interval with `from <= to`. Distinct from the
/// evidence-derived `ValidityHorizon`: this is the procedure's declared
/// effective period, not the source-revision validity projection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct EffectiveInterval {
    from: time::OffsetDateTime,
    to: time::OffsetDateTime,
}

impl EffectiveInterval {
    /// Builds one effective interval.
    ///
    /// # Errors
    ///
    /// Returns [`AffairsValueError`] when `from > to`.
    pub fn new(
        from: time::OffsetDateTime,
        to: time::OffsetDateTime,
    ) -> Result<Self, AffairsValueError> {
        if from > to {
            return Err(value_error(
                "EffectiveInterval",
                AffairsValueErrorKind::InvalidRange,
            ));
        }
        Ok(Self { from, to })
    }

    #[must_use]
    pub const fn from(self) -> time::OffsetDateTime {
        self.from
    }

    #[must_use]
    pub const fn to(self) -> time::OffsetDateTime {
        self.to
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn id_grammar_admits_and_rejects() {
        assert!(ProcedureId::parse("proc:fixture").is_ok());
        assert!(ProcedureId::parse("a").is_ok());
        assert!(ProcedureId::parse("a".repeat(128)).is_ok());
        assert!(matches!(
            ProcedureId::parse(""),
            Err(AffairsValueError {
                kind: AffairsValueErrorKind::Empty,
                ..
            })
        ));
        assert!(matches!(
            ProcedureId::parse("a".repeat(129)),
            Err(AffairsValueError {
                kind: AffairsValueErrorKind::TooLong { .. },
                ..
            })
        ));
        assert!(matches!(
            ProcedureId::parse("-bad"),
            Err(AffairsValueError {
                kind: AffairsValueErrorKind::InvalidStart,
                ..
            })
        ));
        assert!(matches!(
            ProcedureId::parse("bad-"),
            Err(AffairsValueError {
                kind: AffairsValueErrorKind::InvalidEnd,
                ..
            })
        ));
        assert!(matches!(
            ProcedureId::parse("ba@d"),
            Err(AffairsValueError {
                kind: AffairsValueErrorKind::InvalidCharacter { .. },
                ..
            })
        ));
    }

    #[test]
    fn utf8_text_bounds_enforced() {
        assert!(Title::new("hello").is_ok());
        assert!(matches!(
            Title::new(""),
            Err(AffairsValueError {
                kind: AffairsValueErrorKind::Empty,
                ..
            })
        ));
        assert!(matches!(
            Title::new("a".repeat(257)),
            Err(AffairsValueError {
                kind: AffairsValueErrorKind::TooLong { max_bytes: 256 },
                ..
            })
        ));
        assert!(BoardPolicyVersion::new(0).is_err());
        assert!(BoardPolicyVersion::new(1).is_ok());
    }

    #[test]
    fn url_checks_scheme_and_host() {
        assert!(Url::new("https://example.com/path").is_ok());
        assert!(Url::new("http://host.example").is_ok());
        assert!(matches!(
            Url::new("ftp://example.com"),
            Err(AffairsValueError {
                kind: AffairsValueErrorKind::InvalidScheme,
                ..
            })
        ));
        assert!(matches!(
            Url::new("https:///path"),
            Err(AffairsValueError {
                kind: AffairsValueErrorKind::InvalidHost,
                ..
            })
        ));
    }
}
