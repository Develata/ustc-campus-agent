//! The pure, replayable `platform-session/v0` lifecycle kernel owned by `M00-B2 session-domain`.
//!
//! This module owns immutable open scope, the resolved idle/absolute/credential deadline algebra,
//! the open/refresh/expire/revoke transition table, expected-revision event ordering and
//! deterministic decision, evolution and replay. It reads no clock, mints no identifier, verifies
//! no credential, computes no digest, persists nothing and calls no adapter: every instant,
//! identity and evidence value arrives from the caller and is either accepted verbatim or
//! rejected.
//!
//! A structurally valid [`SessionCredentialEvidence`] is a claim from a trusted `M00`
//! authentication-adapter boundary, never proof that a credential was authenticated. A
//! successfully decided [`SessionEvent`] is not proof that anything was appended: `M00-B4` owns
//! the journal that makes `expected_revision` durable.
//!
//! Rejected input may itself be credential material, so no error, `Display` or `Debug` surface
//! produced here retains or echoes it, and the credential-evidence digest is redacted at the one
//! type that holds it.

use std::error::Error;
use std::fmt;

use serde::de;
use serde::{Deserialize, Deserializer, Serialize};

use crate::identity::{SessionId, TenantId, UserId};

/// Maximum encoded length, in UTF-8 bytes, of an [`AuthAdapterId`].
const MAX_ADAPTER_ID_BYTES: usize = 128;

/// The only admitted digest algorithm prefix, lowercase and exact.
const DIGEST_PREFIX: &str = "sha256:";

/// Exactly how many lowercase hexadecimal digits follow [`DIGEST_PREFIX`].
const DIGEST_HEX_DIGITS: usize = 64;

/// What a `Debug` rendering shows in place of credential-evidence digest bytes.
const REDACTED_DIGEST: &str = "<redacted>";

/// Which value rule rejected a candidate `platform-session/v0` value.
///
/// Each variant carries a fixed bound or a byte offset only. No variant carries the rejected
/// input, a fragment of it, or the offending byte itself.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionValueErrorKind {
    /// The candidate had zero bytes.
    Empty,
    /// The candidate exceeded the fixed encoded-length bound.
    TooLong {
        /// The fixed maximum encoded length, in UTF-8 bytes.
        max_bytes: usize,
    },
    /// The first byte was not ASCII alphanumeric.
    InvalidStart,
    /// An interior byte was neither ASCII alphanumeric nor one of `.`, `_`, `:`, `-`.
    InvalidCharacter {
        /// Zero-based index of the first offending byte within the rejected UTF-8 bytes.
        byte_index: usize,
    },
    /// The final byte was not ASCII alphanumeric.
    InvalidEnd,
    /// The candidate was not exactly `sha256:` followed by 64 lowercase hexadecimal digits.
    ///
    /// Payload-free on purpose: the value has one fixed shape, so a positional index would
    /// describe secret-derived text without adding a usable distinction.
    MalformedDigest,
    /// A [`SessionDuration`] of zero milliseconds.
    ZeroDuration,
    /// `credential_not_after` was present and not strictly later than `authenticated_at`.
    CredentialWindowNotAfterAuthentication,
}

/// Why one `platform-session/v0` construction failed.
///
/// The error names the Rust value kind that rejected the input and the rule that rejected it. It
/// deliberately has no `source`, so no rejected input can be reached by walking the error chain,
/// and no public constructor, because it arises only from a rejecting validator inside this
/// module.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SessionValueError {
    value_kind: &'static str,
    kind: SessionValueErrorKind,
}

impl SessionValueError {
    /// Returns the Rust type name of the value kind that rejected the input.
    #[must_use]
    pub const fn value_kind(&self) -> &'static str {
        self.value_kind
    }

    /// Returns the rule that rejected the input.
    #[must_use]
    pub const fn kind(&self) -> SessionValueErrorKind {
        self.kind
    }
}

impl fmt::Display for SessionValueError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value_kind = self.value_kind;
        match self.kind {
            SessionValueErrorKind::Empty => {
                write!(formatter, "{value_kind} rejected: value is empty")
            }
            SessionValueErrorKind::TooLong { max_bytes } => write!(
                formatter,
                "{value_kind} rejected: encoded length exceeds {max_bytes} bytes"
            ),
            SessionValueErrorKind::InvalidStart => write!(
                formatter,
                "{value_kind} rejected: first byte is not ASCII alphanumeric"
            ),
            SessionValueErrorKind::InvalidCharacter { byte_index } => write!(
                formatter,
                "{value_kind} rejected: byte {byte_index} is not permitted"
            ),
            SessionValueErrorKind::InvalidEnd => write!(
                formatter,
                "{value_kind} rejected: final byte is not ASCII alphanumeric"
            ),
            SessionValueErrorKind::MalformedDigest => {
                write!(
                    formatter,
                    "{value_kind} rejected: digest shape is malformed"
                )
            }
            SessionValueErrorKind::ZeroDuration => {
                write!(formatter, "{value_kind} rejected: duration is zero")
            }
            SessionValueErrorKind::CredentialWindowNotAfterAuthentication => write!(
                formatter,
                "{value_kind} rejected: credential deadline is not after authentication"
            ),
        }
    }
}

impl Error for SessionValueError {}

/// Builds the one error shape this module reports from a rejecting validator.
const fn value_error(value_kind: &'static str, kind: SessionValueErrorKind) -> SessionValueError {
    SessionValueError { value_kind, kind }
}

/// Boundary bytes of an [`AuthAdapterId`] are ASCII alphanumeric only.
const fn is_adapter_boundary_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric()
}

/// Interior bytes of an [`AuthAdapterId`] add the four canonical delimiters.
const fn is_adapter_interior_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b':')
}

/// Applies the [`AuthAdapterId`] grammar in the exact precedence frozen by
/// `platform-session/v0` §2.2 and §9.1.
///
/// Precondition: `value` is well-formed UTF-8, guaranteed by `&str`.
/// Postcondition: `Ok(())` exactly when `value` matches
/// `^[A-Za-z0-9](?:[-A-Za-z0-9._:]{0,126}[A-Za-z0-9])?$`.
/// Invariant: exactly one left-to-right pass over the interior, and no allocation.
///
/// This grammar is owned by `platform-session/v0`, not borrowed from `platform-identity/v0`. The
/// two are deliberately byte-identical so an operator reads one identifier shape across `M00`,
/// but neither document is authority for the other and each is bound to its own implementation by
/// its own carriers.
fn classify_adapter_id(value: &str) -> Result<(), SessionValueErrorKind> {
    let bytes = value.as_bytes();
    let Some((&first, after_first)) = bytes.split_first() else {
        return Err(SessionValueErrorKind::Empty);
    };
    if bytes.len() > MAX_ADAPTER_ID_BYTES {
        return Err(SessionValueErrorKind::TooLong {
            max_bytes: MAX_ADAPTER_ID_BYTES,
        });
    }
    if !is_adapter_boundary_byte(first) {
        return Err(SessionValueErrorKind::InvalidStart);
    }
    // A one-byte value is fully decided by the first-byte rule; the interior range is then
    // empty and there is no separate final byte.
    let Some((&last, interior)) = after_first.split_last() else {
        return Ok(());
    };
    for (offset, &byte) in interior.iter().enumerate() {
        if !is_adapter_interior_byte(byte) {
            return Err(SessionValueErrorKind::InvalidCharacter {
                byte_index: offset + 1,
            });
        }
    }
    if !is_adapter_boundary_byte(last) {
        return Err(SessionValueErrorKind::InvalidEnd);
    }
    Ok(())
}

/// Applies the [`CredentialEvidenceDigest`] shape frozen by `platform-session/v0` §2.2.
///
/// Postcondition: `Ok(())` exactly when `value` matches `^sha256:[0-9a-f]{64}$`. Uppercase
/// hexadecimal, a bare digest with no prefix, another algorithm prefix and any other length are
/// rejected; there is no normalization, lower-casing or prefix-insertion path.
fn classify_digest(value: &str) -> Result<(), SessionValueErrorKind> {
    let Some(hex) = value.strip_prefix(DIGEST_PREFIX) else {
        return Err(SessionValueErrorKind::MalformedDigest);
    };
    if hex.len() != DIGEST_HEX_DIGITS {
        return Err(SessionValueErrorKind::MalformedDigest);
    }
    if !hex
        .as_bytes()
        .iter()
        .all(|byte| matches!(byte, b'0'..=b'9' | b'a'..=b'f'))
    {
        return Err(SessionValueErrorKind::MalformedDigest);
    }
    Ok(())
}

/// A non-negative count of Unix-epoch milliseconds observed by an adapter.
///
/// The domain never reads wall-clock time itself; a command carries one adapter-observed instant.
/// Time is compared only as an integer ordering: no timezone, locale, leap second or formatted
/// timestamp enters the state machine.
///
/// The constructor is total, and that is a statement about representation rather than
/// admissibility — every `u64` denotes an instant, while §3 still rejects one that is stale,
/// non-monotone or arithmetically out of range for the transition being decided.
///
/// The private backing field cannot be filled directly:
///
/// ```compile_fail
/// use ustc_campus_agent_core::session::SessionInstant;
///
/// let instant = SessionInstant { millis: 1 };
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SessionInstant {
    millis: u64,
}

impl SessionInstant {
    /// Wraps a Unix-epoch millisecond count.
    #[must_use]
    pub const fn from_unix_millis(millis: u64) -> Self {
        Self { millis }
    }

    /// Returns the Unix-epoch millisecond count.
    #[must_use]
    pub const fn as_unix_millis(&self) -> u64 {
        self.millis
    }
}

/// A non-zero millisecond duration used to resolve a session deadline.
///
/// Session-policy and configuration loading own deployment ceilings; this pure kernel still
/// rejects zero and arithmetic overflow. A duration is not a retry count, token lifetime or UI
/// timeout.
///
/// There is no unchecked constructor:
///
/// ```compile_fail
/// use ustc_campus_agent_core::session::SessionDuration;
///
/// let duration = SessionDuration::new(0);
/// ```
///
/// A zero duration cannot arrive through Serde either, because deserialization delegates to the
/// one checked constructor:
///
/// ```
/// use ustc_campus_agent_core::session::SessionDuration;
///
/// assert!(serde_json::from_str::<SessionDuration>("0").is_err());
/// assert!(serde_json::from_str::<SessionDuration>("1").is_ok());
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(transparent)]
pub struct SessionDuration {
    millis: u64,
}

impl SessionDuration {
    /// Parses one non-zero millisecond duration.
    ///
    /// This is the single validator; the `Deserialize` implementation delegates here.
    ///
    /// # Errors
    ///
    /// Returns [`SessionValueError`] with [`SessionValueErrorKind::ZeroDuration`] when `millis`
    /// is zero.
    pub const fn from_millis(millis: u64) -> Result<Self, SessionValueError> {
        if millis == 0 {
            return Err(value_error(
                "SessionDuration",
                SessionValueErrorKind::ZeroDuration,
            ));
        }
        Ok(Self { millis })
    }

    /// Returns the millisecond count.
    #[must_use]
    pub const fn as_millis(&self) -> u64 {
        self.millis
    }
}

impl<'de> Deserialize<'de> for SessionDuration {
    /// Deserializes the canonical `u64`, then applies the one checked constructor.
    ///
    /// A hand-written `Visitor` is deliberately not used: every implemented `visit_*` method is an
    /// independent construction path. Deferring to `u64`'s own `Deserialize` leaves exactly one
    /// construction path in this implementation, whatever entry point the deserializer chooses.
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let millis = u64::deserialize(deserializer)?;
        Self::from_millis(millis).map_err(de::Error::custom)
    }
}

/// A bounded, opaque authentication-adapter identity.
///
/// The spelling carries no trust level, provider semantics or authorization result, and this is
/// not one of the six `platform-identity/v0` kinds: it does not widen them and must not be
/// converted to or from one.
///
/// One identity kind cannot be produced from this value:
///
/// ```compile_fail
/// use ustc_campus_agent_core::identity::TenantId;
/// use ustc_campus_agent_core::session::AuthAdapterId;
///
/// fn widen(adapter: AuthAdapterId) -> TenantId {
///     TenantId::from(adapter)
/// }
/// ```
///
/// The backing string cannot be mutated or reached mutably:
///
/// ```compile_fail
/// use ustc_campus_agent_core::session::AuthAdapterId;
///
/// fn rewrite(adapter: &mut AuthAdapterId) {
///     adapter.as_mut_str().make_ascii_uppercase();
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(transparent)]
pub struct AuthAdapterId {
    value: String,
}

impl AuthAdapterId {
    /// Parses one canonical `AuthAdapterId`.
    ///
    /// This is the single validator. Every other construction and deserialization path on this
    /// type delegates here, so all of them share one grammar and one error precedence.
    ///
    /// # Errors
    ///
    /// Returns [`SessionValueError`] when `value` does not match the `platform-session/v0` §2.2
    /// grammar. The error names this kind and the failing rule and never contains the rejected
    /// input.
    pub fn parse(value: impl Into<String>) -> Result<Self, SessionValueError> {
        let value = value.into();
        match classify_adapter_id(&value) {
            Ok(()) => Ok(Self { value }),
            Err(kind) => Err(value_error("AuthAdapterId", kind)),
        }
    }

    /// Returns the exact canonical bytes, with case and delimiters preserved.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.value
    }
}

impl<'de> Deserialize<'de> for AuthAdapterId {
    /// Deserializes the canonical string, then applies the one checked constructor.
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::parse(value).map_err(de::Error::custom)
    }
}

/// A non-invertible fingerprint of already-admitted credential evidence, exactly `sha256:`
/// followed by 64 lowercase hexadecimal digits.
///
/// It is not a credential, bearer token, password hash, refresh token or authorization result, and
/// this module never computes one — it validates shape and stores the value.
///
/// The producer carries an obligation this module cannot check and that `platform-session/v0`
/// §2.2 states normatively: the digest MUST be domain-separated and taken over adapter-side
/// material that is not raw credential text. This value is pinned into an immutable event and
/// preserved across replay, so a digest over a low-entropy password would be an offline-attackable
/// hash embedded permanently in audit evidence.
///
/// `Debug` is hand-written and redacts, at the one type that *is* the digest, so every holder
/// inherits the redaction rather than each having to remember it:
///
/// ```
/// use ustc_campus_agent_core::session::CredentialEvidenceDigest;
///
/// let digest = CredentialEvidenceDigest::parse(
///     "sha256:0000000000000000000000000000000000000000000000000000000000000abc",
/// )
/// .expect("fixture");
/// assert_eq!(format!("{digest:?}"), "CredentialEvidenceDigest(<redacted>)");
/// assert!(!format!("{digest:?}").contains("abc"));
/// ```
///
/// There is no second, unchecked construction path:
///
/// ```compile_fail
/// use ustc_campus_agent_core::session::CredentialEvidenceDigest;
///
/// let digest = CredentialEvidenceDigest::new("sha256:deadbeef");
/// ```
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(transparent)]
pub struct CredentialEvidenceDigest {
    value: String,
}

impl CredentialEvidenceDigest {
    /// Parses one canonical `CredentialEvidenceDigest`.
    ///
    /// # Errors
    ///
    /// Returns [`SessionValueError`] with [`SessionValueErrorKind::MalformedDigest`] when `value`
    /// is not exactly `sha256:` followed by 64 lowercase hexadecimal digits.
    pub fn parse(value: impl Into<String>) -> Result<Self, SessionValueError> {
        let value = value.into();
        match classify_digest(&value) {
            Ok(()) => Ok(Self { value }),
            Err(kind) => Err(value_error("CredentialEvidenceDigest", kind)),
        }
    }

    /// Returns the exact canonical digest text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.value
    }
}

impl fmt::Debug for CredentialEvidenceDigest {
    /// Renders a fixed redaction token instead of the digest bytes.
    ///
    /// `Display` is deliberately not implemented at all, so there is no second rendering path to
    /// keep in step.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "CredentialEvidenceDigest({REDACTED_DIGEST})")
    }
}

impl<'de> Deserialize<'de> for CredentialEvidenceDigest {
    /// Deserializes the canonical string, then applies the one checked constructor.
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::parse(value).map_err(de::Error::custom)
    }
}

/// Deserializes a `credential_not_after` field that must be *present*.
///
/// Serde's derived decode fills a **missing** `Option` field with `None` instead of failing,
/// including inside a shadow struct. On this field that default is a downgrade by omission: a
/// payload that simply omits it would decode as "this credential never expires", deleting the
/// `Credential` term from §3's `min(...)` and disarming the already-expired-evidence open check.
/// Naming a `deserialize_with` function is what makes a missing field an error, while an explicit
/// `null` still decodes as "no credential deadline".
fn deserialize_present_credential_deadline<'de, D>(
    deserializer: D,
) -> Result<Option<SessionInstant>, D::Error>
where
    D: Deserializer<'de>,
{
    Option::<SessionInstant>::deserialize(deserializer)
}

/// Bounded provenance of one admitted authentication, with no raw credential retained.
///
/// Structural validity is not authentication. A successfully constructed or deserialized value is
/// a claim from a trusted `M00` authentication-adapter or application boundary; it is never
/// sufficient evidence at an untrusted `M10`/transport boundary. There is no
/// raw-credential-to-evidence conversion here, and nothing in this module hashes credential text.
///
/// Raw credential text cannot be handed to this type in place of a digest:
///
/// ```compile_fail
/// use ustc_campus_agent_core::session::SessionCredentialEvidence;
///
/// let evidence = SessionCredentialEvidence::from_raw_credential("hunter2");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SessionCredentialEvidence {
    tenant_id: TenantId,
    user_id: UserId,
    auth_adapter_id: AuthAdapterId,
    evidence_digest: CredentialEvidenceDigest,
    authenticated_at: SessionInstant,
    credential_not_after: Option<SessionInstant>,
}

impl SessionCredentialEvidence {
    /// Builds credential-evidence provenance, checking the one invariant that spans two fields.
    ///
    /// # Errors
    ///
    /// Returns [`SessionValueError`] with
    /// [`SessionValueErrorKind::CredentialWindowNotAfterAuthentication`] when
    /// `credential_not_after` is present and not strictly later than `authenticated_at`.
    pub fn new(
        tenant_id: TenantId,
        user_id: UserId,
        auth_adapter_id: AuthAdapterId,
        evidence_digest: CredentialEvidenceDigest,
        authenticated_at: SessionInstant,
        credential_not_after: Option<SessionInstant>,
    ) -> Result<Self, SessionValueError> {
        if let Some(not_after) = credential_not_after
            && not_after.millis <= authenticated_at.millis
        {
            return Err(value_error(
                "SessionCredentialEvidence",
                SessionValueErrorKind::CredentialWindowNotAfterAuthentication,
            ));
        }
        Ok(Self {
            tenant_id,
            user_id,
            auth_adapter_id,
            evidence_digest,
            authenticated_at,
            credential_not_after,
        })
    }

    /// Returns the tenant this evidence was admitted for.
    #[must_use]
    pub const fn tenant_id(&self) -> &TenantId {
        &self.tenant_id
    }

    /// Returns the platform user subject this evidence was admitted for.
    #[must_use]
    pub const fn user_id(&self) -> &UserId {
        &self.user_id
    }

    /// Returns the authentication adapter that produced this evidence.
    #[must_use]
    pub const fn auth_adapter_id(&self) -> &AuthAdapterId {
        &self.auth_adapter_id
    }

    /// Returns the non-invertible fingerprint of the admitted evidence.
    #[must_use]
    pub const fn evidence_digest(&self) -> &CredentialEvidenceDigest {
        &self.evidence_digest
    }

    /// Returns when the adapter observed the authentication.
    #[must_use]
    pub const fn authenticated_at(&self) -> SessionInstant {
        self.authenticated_at
    }

    /// Returns the credential deadline, when the credential carries one.
    #[must_use]
    pub const fn credential_not_after(&self) -> Option<SessionInstant> {
        self.credential_not_after
    }
}

impl<'de> Deserialize<'de> for SessionCredentialEvidence {
    /// Decodes a private shadow struct and hands it to the checked constructor.
    ///
    /// The derived field-by-field decode is insufficient here and is forbidden: the credential
    /// window is a relation between two fields, so it belongs to neither of them, and
    /// `deny_unknown_fields` plus per-field delegation would still admit a value no constructor
    /// would have built.
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Shadow {
            tenant_id: TenantId,
            user_id: UserId,
            auth_adapter_id: AuthAdapterId,
            evidence_digest: CredentialEvidenceDigest,
            authenticated_at: SessionInstant,
            #[serde(deserialize_with = "deserialize_present_credential_deadline")]
            credential_not_after: Option<SessionInstant>,
        }

        let shadow = Shadow::deserialize(deserializer)?;
        Self::new(
            shadow.tenant_id,
            shadow.user_id,
            shadow.auth_adapter_id,
            shadow.evidence_digest,
            shadow.authenticated_at,
            shadow.credential_not_after,
        )
        .map_err(de::Error::custom)
    }
}

/// The idle and hard-expiry durations resolved and pinned when a session opens.
///
/// Refresh never reloads policy and never changes either duration. `idle_timeout` may equal or
/// exceed `absolute_timeout`: such a policy is well-formed and simply means the idle candidate
/// never binds, so every refresh returns [`SessionDomainError::NoEffectiveRefresh`] — an ordinary
/// steady state, not a liveness failure.
///
/// A session cannot be defaulted into existence:
///
/// ```compile_fail
/// use ustc_campus_agent_core::session::SessionPolicy;
///
/// let policy = SessionPolicy::default();
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SessionPolicy {
    idle_timeout: SessionDuration,
    absolute_timeout: SessionDuration,
}

impl SessionPolicy {
    /// Pins both resolved durations.
    ///
    /// Total: both fields are already non-zero by type and the two durations carry no relation to
    /// each other, so there is nothing left to reject.
    #[must_use]
    pub const fn new(idle_timeout: SessionDuration, absolute_timeout: SessionDuration) -> Self {
        Self {
            idle_timeout,
            absolute_timeout,
        }
    }

    /// Returns the resolved idle timeout.
    #[must_use]
    pub const fn idle_timeout(&self) -> SessionDuration {
        self.idle_timeout
    }

    /// Returns the resolved policy-absolute timeout.
    #[must_use]
    pub const fn absolute_timeout(&self) -> SessionDuration {
        self.absolute_timeout
    }
}

/// Which deadline made a session expire, resolved with the tie precedence
/// `Credential > Absolute > Idle`.
///
/// The classification does not change authority: every cause blocks all new request contexts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionExpiryCause {
    /// The credential deadline equalled the effective deadline.
    Credential,
    /// The policy-absolute deadline equalled the effective deadline.
    Absolute,
    /// Neither of the above; the idle candidate bound.
    Idle,
}

/// The lifecycle state of one session.
///
/// This is a public enum, so a caller may name [`SessionStatus::Active`] or assemble
/// [`SessionStatus::Expired`] from instants and a cause it already holds. That is not a hole and
/// no part of this module claims otherwise: a status carries no authority by itself. What is
/// closed is the *snapshot* — a caller-built status cannot be injected into, substituted inside or
/// read back out of a [`SessionSnapshot`] as that snapshot's own status, because the snapshot's
/// fields are private, it has no public constructor, no `Deserialize` and no setter, and
/// [`evolve`] is its only producer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionStatus {
    /// The session may still admit operations, subject to [`SessionSnapshot::admits_at`].
    Active,
    /// The session lost validity at its effective deadline. Terminal.
    Expired {
        /// The effective deadline at which validity was lost.
        expired_at: SessionInstant,
        /// When expiry was detected and persisted, at or after `expired_at`.
        observed_at: SessionInstant,
        /// Which deadline bound, under the §3 tie precedence.
        cause: SessionExpiryCause,
    },
    /// The session was revoked before its effective deadline. Terminal.
    Revoked {
        /// When revocation was observed.
        revoked_at: SessionInstant,
    },
}

/// The immutable read model of one session at one revision.
///
/// All fields are private and read-only. Tenant, user, session, adapter, evidence digest,
/// authentication time, credential deadline, policy durations and policy-absolute expiry are
/// immutable after open. `revision` is the last applied event sequence and starts at `1` after
/// `SessionOpened`.
///
/// A snapshot has no public constructor and no `Deserialize`: it arises only from validated
/// evolution and replay.
///
/// A caller cannot substitute its own status:
///
/// ```compile_fail
/// use ustc_campus_agent_core::session::{SessionSnapshot, SessionStatus};
///
/// fn forge(snapshot: &mut SessionSnapshot) {
///     snapshot.status = SessionStatus::Active;
/// }
/// ```
///
/// …nor set its revision or deadlines through a generic setter:
///
/// ```compile_fail
/// use ustc_campus_agent_core::session::SessionSnapshot;
///
/// fn bump(snapshot: &mut SessionSnapshot) {
///     snapshot.set_revision(0);
/// }
/// ```
///
/// …nor default one into an active state:
///
/// ```compile_fail
/// use ustc_campus_agent_core::session::SessionSnapshot;
///
/// let snapshot = SessionSnapshot::default();
/// ```
///
/// …nor decode one from a transport payload, because it has no `Deserialize`:
///
/// ```compile_fail
/// use ustc_campus_agent_core::session::SessionSnapshot;
///
/// let snapshot: SessionSnapshot = serde_json::from_str("{}").expect("no Deserialize exists");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SessionSnapshot {
    session_id: SessionId,
    tenant_id: TenantId,
    user_id: UserId,
    auth_adapter_id: AuthAdapterId,
    evidence_digest: CredentialEvidenceDigest,
    authenticated_at: SessionInstant,
    credential_not_after: Option<SessionInstant>,
    opened_at: SessionInstant,
    last_transition_at: SessionInstant,
    idle_timeout: SessionDuration,
    absolute_timeout: SessionDuration,
    effective_expires_at: SessionInstant,
    absolute_expires_at: SessionInstant,
    status: SessionStatus,
    revision: u64,
}

impl SessionSnapshot {
    /// Returns the session identity pinned at open.
    #[must_use]
    pub const fn session_id(&self) -> &SessionId {
        &self.session_id
    }

    /// Returns the tenant pinned at open.
    #[must_use]
    pub const fn tenant_id(&self) -> &TenantId {
        &self.tenant_id
    }

    /// Returns the user pinned at open.
    #[must_use]
    pub const fn user_id(&self) -> &UserId {
        &self.user_id
    }

    /// Returns the authentication adapter pinned at open.
    #[must_use]
    pub const fn auth_adapter_id(&self) -> &AuthAdapterId {
        &self.auth_adapter_id
    }

    /// Returns the credential-evidence digest pinned at open.
    #[must_use]
    pub const fn evidence_digest(&self) -> &CredentialEvidenceDigest {
        &self.evidence_digest
    }

    /// Returns when the adapter observed the authentication.
    #[must_use]
    pub const fn authenticated_at(&self) -> SessionInstant {
        self.authenticated_at
    }

    /// Returns the credential deadline pinned at open, when the credential carries one.
    #[must_use]
    pub const fn credential_not_after(&self) -> Option<SessionInstant> {
        self.credential_not_after
    }

    /// Returns when the session opened.
    #[must_use]
    pub const fn opened_at(&self) -> SessionInstant {
        self.opened_at
    }

    /// Returns when the last applied event was observed.
    #[must_use]
    pub const fn last_transition_at(&self) -> SessionInstant {
        self.last_transition_at
    }

    /// Returns the idle timeout pinned at open.
    #[must_use]
    pub const fn idle_timeout(&self) -> SessionDuration {
        self.idle_timeout
    }

    /// Returns the policy-absolute timeout pinned at open.
    #[must_use]
    pub const fn absolute_timeout(&self) -> SessionDuration {
        self.absolute_timeout
    }

    /// Returns the current effective expiry deadline.
    ///
    /// This is **not** a validity predicate. A revoked session keeps this field unchanged, so
    /// `observed_at < effective_expires_at()` reads as "not yet expired" for exactly the
    /// revocation case. Use [`SessionSnapshot::admits_at`] instead.
    #[must_use]
    pub const fn effective_expires_at(&self) -> SessionInstant {
        self.effective_expires_at
    }

    /// Returns the policy-absolute deadline pinned at open.
    #[must_use]
    pub const fn absolute_expires_at(&self) -> SessionInstant {
        self.absolute_expires_at
    }

    /// Returns the lifecycle status.
    #[must_use]
    pub const fn status(&self) -> SessionStatus {
        self.status
    }

    /// Returns the last applied event sequence.
    #[must_use]
    pub const fn revision(&self) -> u64 {
        self.revision
    }

    /// Answers the one sanctioned validity question: may this snapshot admit an operation observed
    /// at `observed_at`?
    ///
    /// The question is frozen as **current admission**, never historical validity. It does not
    /// answer "was this session valid at instant `t`?", and this module offers no method that
    /// does; a later audit answers that by replaying the event sequence, which is the only place
    /// the past is authoritative.
    ///
    /// It is `true` only when all three of `status == Active`,
    /// `observed_at >= last_transition_at` and `observed_at < effective_expires_at` hold. The
    /// middle conjunct makes the read model fail closed on stale time, so it is never more
    /// permissive than the decide path it guards: an instant the aggregate has already moved past
    /// is not evidence of present admission.
    ///
    /// The conjunct never makes a live session unreachable. While a session is `Active`,
    /// `effective_expires_at > last_transition_at` is an invariant, so the admitting window
    /// `[last_transition_at, effective_expires_at)` is non-empty: `admits_at` is `true` at exactly
    /// `last_transition_at` and `false` at exactly `effective_expires_at`.
    #[must_use]
    pub const fn admits_at(&self, observed_at: SessionInstant) -> bool {
        matches!(self.status, SessionStatus::Active)
            && observed_at.millis >= self.last_transition_at.millis
            && observed_at.millis < self.effective_expires_at.millis
    }
}

/// Opens one session from admitted credential evidence and a resolved policy.
///
/// The private fields cannot be filled or rewritten directly:
///
/// ```compile_fail
/// use ustc_campus_agent_core::session::{OpenSession, SessionInstant};
///
/// fn restamp(command: &mut OpenSession) {
///     command.observed_at = SessionInstant::from_unix_millis(0);
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct OpenSession {
    session_id: SessionId,
    credential_evidence: SessionCredentialEvidence,
    policy: SessionPolicy,
    observed_at: SessionInstant,
    expected_revision: u64,
}

impl OpenSession {
    /// Builds an open command.
    ///
    /// Total on stated grounds. §3's open failures are all computable from these fields, so a
    /// checked constructor *could* reject them, and it must not: §7 puts `SessionAlreadyExists`
    /// and `RevisionMismatch` ahead of time ordering, credential expiry and overflow, while a
    /// constructor necessarily runs before any of them — so it would report the wrong one of two
    /// co-occurring faults. It would also split one failure across both §9 taxonomies.
    #[must_use]
    pub const fn new(
        session_id: SessionId,
        credential_evidence: SessionCredentialEvidence,
        policy: SessionPolicy,
        observed_at: SessionInstant,
        expected_revision: u64,
    ) -> Self {
        Self {
            session_id,
            credential_evidence,
            policy,
            observed_at,
            expected_revision,
        }
    }

    /// Returns the session identity to open.
    #[must_use]
    pub const fn session_id(&self) -> &SessionId {
        &self.session_id
    }

    /// Returns the admitted credential evidence.
    #[must_use]
    pub const fn credential_evidence(&self) -> &SessionCredentialEvidence {
        &self.credential_evidence
    }

    /// Returns the resolved session policy.
    #[must_use]
    pub const fn policy(&self) -> SessionPolicy {
        self.policy
    }

    /// Returns the adapter-observed open instant.
    #[must_use]
    pub const fn observed_at(&self) -> SessionInstant {
        self.observed_at
    }

    /// Returns the optimistic-concurrency revision claim.
    #[must_use]
    pub const fn expected_revision(&self) -> u64 {
        self.expected_revision
    }
}

/// Extends an active session's idle expiry, and nothing else.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RefreshSession {
    session_id: SessionId,
    observed_at: SessionInstant,
    expected_revision: u64,
}

impl RefreshSession {
    /// Builds a refresh command.
    ///
    /// Total: this command carries only a session identity, an instant and a revision claim, and
    /// every question about them is a fact about the *aggregate*, which a constructor holding one
    /// command cannot see.
    #[must_use]
    pub const fn new(
        session_id: SessionId,
        observed_at: SessionInstant,
        expected_revision: u64,
    ) -> Self {
        Self {
            session_id,
            observed_at,
            expected_revision,
        }
    }

    /// Returns the session identity to refresh.
    #[must_use]
    pub const fn session_id(&self) -> &SessionId {
        &self.session_id
    }

    /// Returns the adapter-observed instant.
    #[must_use]
    pub const fn observed_at(&self) -> SessionInstant {
        self.observed_at
    }

    /// Returns the optimistic-concurrency revision claim.
    #[must_use]
    pub const fn expected_revision(&self) -> u64 {
        self.expected_revision
    }
}

/// Records that an active session reached its effective deadline.
///
/// `M00`-internal: it is issued only through the future `M00-B4` session application/port path, is
/// never decoded directly from `M10`, and does not expand the `M00` blueprint's public input list.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ExpireSession {
    session_id: SessionId,
    observed_at: SessionInstant,
    expected_revision: u64,
}

impl ExpireSession {
    /// Builds an expire command. Total, on [`RefreshSession::new`]'s grounds.
    #[must_use]
    pub const fn new(
        session_id: SessionId,
        observed_at: SessionInstant,
        expected_revision: u64,
    ) -> Self {
        Self {
            session_id,
            observed_at,
            expected_revision,
        }
    }

    /// Returns the session identity to expire.
    #[must_use]
    pub const fn session_id(&self) -> &SessionId {
        &self.session_id
    }

    /// Returns the adapter-observed instant.
    #[must_use]
    pub const fn observed_at(&self) -> SessionInstant {
        self.observed_at
    }

    /// Returns the optimistic-concurrency revision claim.
    #[must_use]
    pub const fn expected_revision(&self) -> u64 {
        self.expected_revision
    }
}

/// Revokes an active session before its effective deadline.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RevokeSession {
    session_id: SessionId,
    observed_at: SessionInstant,
    expected_revision: u64,
}

impl RevokeSession {
    /// Builds a revoke command. Total, on [`RefreshSession::new`]'s grounds.
    #[must_use]
    pub const fn new(
        session_id: SessionId,
        observed_at: SessionInstant,
        expected_revision: u64,
    ) -> Self {
        Self {
            session_id,
            observed_at,
            expected_revision,
        }
    }

    /// Returns the session identity to revoke.
    #[must_use]
    pub const fn session_id(&self) -> &SessionId {
        &self.session_id
    }

    /// Returns the adapter-observed instant.
    #[must_use]
    pub const fn observed_at(&self) -> SessionInstant {
        self.observed_at
    }

    /// Returns the optimistic-concurrency revision claim.
    #[must_use]
    pub const fn expected_revision(&self) -> u64 {
        self.expected_revision
    }
}

/// The exact command set of `platform-session/v0`.
///
/// There is deliberately no generic `SetState`, `Touch`, `Patch`, `Restore`, `Unexpire` or
/// `Unrevoke` operation.
///
/// Commands implement `Serialize` but **not** `Deserialize`. Replay reads events, never commands,
/// and with no `Deserialize` there is no way to decode an [`OpenSession`] from a transport payload
/// at all — which is what turns "untrusted callers must not invoke `OpenSession` with
/// self-asserted evidence" from a rule someone must remember into one the compiler enforces:
///
/// ```compile_fail
/// use ustc_campus_agent_core::session::SessionCommand;
///
/// let command: SessionCommand = serde_json::from_str("{}").expect("no Deserialize exists");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionCommand {
    /// Open a new session.
    Open(OpenSession),
    /// Extend an active session's idle expiry.
    Refresh(RefreshSession),
    /// Record that an active session reached its effective deadline.
    Expire(ExpireSession),
    /// Revoke an active session.
    Revoke(RevokeSession),
}

impl SessionCommand {
    /// Returns the session identity this command names.
    #[must_use]
    pub const fn session_id(&self) -> &SessionId {
        match self {
            Self::Open(command) => command.session_id(),
            Self::Refresh(command) => command.session_id(),
            Self::Expire(command) => command.session_id(),
            Self::Revoke(command) => command.session_id(),
        }
    }

    /// Returns the adapter-observed instant this command carries.
    #[must_use]
    pub const fn observed_at(&self) -> SessionInstant {
        match self {
            Self::Open(command) => command.observed_at(),
            Self::Refresh(command) => command.observed_at(),
            Self::Expire(command) => command.observed_at(),
            Self::Revoke(command) => command.observed_at(),
        }
    }

    /// Returns the optimistic-concurrency revision claim this command carries.
    #[must_use]
    pub const fn expected_revision(&self) -> u64 {
        match self {
            Self::Open(command) => command.expected_revision(),
            Self::Refresh(command) => command.expected_revision(),
            Self::Expire(command) => command.expected_revision(),
            Self::Revoke(command) => command.expected_revision(),
        }
    }
}

/// One session opened with immutable scope.
///
/// `opened_at` is exactly the open command's `observed_at`, and the evolved snapshot initializes
/// `last_transition_at` to that same instant.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SessionOpened {
    sequence: u64,
    session_id: SessionId,
    credential_evidence: SessionCredentialEvidence,
    policy: SessionPolicy,
    opened_at: SessionInstant,
}

impl SessionOpened {
    /// Builds an opened event.
    ///
    /// Total, and deliberately so even though this event carries exactly the fields §3's four open
    /// conditions are computed from. A checked constructor would answer the open-invariant
    /// question ahead of [`evolve`]'s sequence check, and it would remove from the input space
    /// every event that could exercise evolution's obligation to re-derive open invariants from
    /// the persisted event — leaving [`SessionDomainError::InvalidTimeOrder`],
    /// [`SessionDomainError::CredentialEvidenceExpired`] and
    /// [`SessionDomainError::DeadlineOverflow`] dead on the evolve path.
    #[must_use]
    pub const fn new(
        sequence: u64,
        session_id: SessionId,
        credential_evidence: SessionCredentialEvidence,
        policy: SessionPolicy,
        opened_at: SessionInstant,
    ) -> Self {
        Self {
            sequence,
            session_id,
            credential_evidence,
            policy,
            opened_at,
        }
    }

    /// Returns the event sequence.
    #[must_use]
    pub const fn sequence(&self) -> u64 {
        self.sequence
    }

    /// Returns the session identity opened.
    #[must_use]
    pub const fn session_id(&self) -> &SessionId {
        &self.session_id
    }

    /// Returns the admitted credential evidence pinned by this event.
    #[must_use]
    pub const fn credential_evidence(&self) -> &SessionCredentialEvidence {
        &self.credential_evidence
    }

    /// Returns the resolved policy pinned by this event.
    #[must_use]
    pub const fn policy(&self) -> SessionPolicy {
        self.policy
    }

    /// Returns when the session opened.
    #[must_use]
    pub const fn opened_at(&self) -> SessionInstant {
        self.opened_at
    }
}

/// One session's idle expiry extended.
///
/// `effective_expires_at` is a redundant verification field: evolution recomputes it from prior
/// state and rejects a mismatch.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SessionRefreshed {
    sequence: u64,
    session_id: SessionId,
    observed_at: SessionInstant,
    effective_expires_at: SessionInstant,
}

impl SessionRefreshed {
    /// Builds a refreshed event.
    ///
    /// Total: its one derived field is correct only against a deadline the event cannot see, so
    /// comparing it to the instant beside it would be decidable without being meaningful.
    #[must_use]
    pub const fn new(
        sequence: u64,
        session_id: SessionId,
        observed_at: SessionInstant,
        effective_expires_at: SessionInstant,
    ) -> Self {
        Self {
            sequence,
            session_id,
            observed_at,
            effective_expires_at,
        }
    }

    /// Returns the event sequence.
    #[must_use]
    pub const fn sequence(&self) -> u64 {
        self.sequence
    }

    /// Returns the session identity refreshed.
    #[must_use]
    pub const fn session_id(&self) -> &SessionId {
        &self.session_id
    }

    /// Returns when the refresh was observed.
    #[must_use]
    pub const fn observed_at(&self) -> SessionInstant {
        self.observed_at
    }

    /// Returns the recomputed effective deadline this event claims.
    #[must_use]
    pub const fn effective_expires_at(&self) -> SessionInstant {
        self.effective_expires_at
    }
}

/// One session expired at its effective deadline.
///
/// `expired_at` is the effective deadline at which the session became invalid, while `observed_at`
/// is when expiry was detected and persisted; a late observation must not rewrite historical
/// validity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SessionExpired {
    sequence: u64,
    session_id: SessionId,
    observed_at: SessionInstant,
    expired_at: SessionInstant,
    cause: SessionExpiryCause,
}

impl SessionExpired {
    /// Builds an expired event.
    ///
    /// Total, and that is a decision rather than an omission. `observed_at >= expired_at` is
    /// perfectly decidable here, but it is not *meaningful*: `expired_at` is a derived field whose
    /// correctness means "exactly equals the aggregate's pre-existing effective deadline", which
    /// an event holding two caller-supplied instants does not know. Checking it would enforce
    /// agreement between two numbers that may both be forged while removing from the input space
    /// exactly the adversarial event [`evolve`]'s first apply guard has to be shown rejecting.
    #[must_use]
    pub const fn new(
        sequence: u64,
        session_id: SessionId,
        observed_at: SessionInstant,
        expired_at: SessionInstant,
        cause: SessionExpiryCause,
    ) -> Self {
        Self {
            sequence,
            session_id,
            observed_at,
            expired_at,
            cause,
        }
    }

    /// Returns the event sequence.
    #[must_use]
    pub const fn sequence(&self) -> u64 {
        self.sequence
    }

    /// Returns the session identity that expired.
    #[must_use]
    pub const fn session_id(&self) -> &SessionId {
        &self.session_id
    }

    /// Returns when expiry was detected.
    #[must_use]
    pub const fn observed_at(&self) -> SessionInstant {
        self.observed_at
    }

    /// Returns the effective deadline this event claims validity was lost at.
    #[must_use]
    pub const fn expired_at(&self) -> SessionInstant {
        self.expired_at
    }

    /// Returns the expiry cause this event claims.
    #[must_use]
    pub const fn cause(&self) -> SessionExpiryCause {
        self.cause
    }
}

/// One session revoked before its effective deadline.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SessionRevoked {
    sequence: u64,
    session_id: SessionId,
    observed_at: SessionInstant,
}

impl SessionRevoked {
    /// Builds a revoked event. Total: it carries no derived field at all.
    #[must_use]
    pub const fn new(sequence: u64, session_id: SessionId, observed_at: SessionInstant) -> Self {
        Self {
            sequence,
            session_id,
            observed_at,
        }
    }

    /// Returns the event sequence.
    #[must_use]
    pub const fn sequence(&self) -> u64 {
        self.sequence
    }

    /// Returns the session identity revoked.
    #[must_use]
    pub const fn session_id(&self) -> &SessionId {
        &self.session_id
    }

    /// Returns when revocation was observed.
    #[must_use]
    pub const fn observed_at(&self) -> SessionInstant {
        self.observed_at
    }
}

/// The exact immutable event set of `platform-session/v0`.
///
/// Events retain only bounded provenance: never raw credentials, secret values, cookies,
/// authorization headers, provider payloads, arbitrary reason strings or client-supplied metadata.
///
/// Events implement both `Serialize` and `Deserialize`, because replay reads them back. That is
/// not an *unchecked* decode: every field is a `u64` or a nominal value whose own `Deserialize`
/// delegates to its checked constructor, and `deny_unknown_fields` closes the unknown-field half.
/// What remains — sequence order, cross-session identity, derived-field agreement, event time
/// inside the guard window — is an aggregate question that [`evolve`] answers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionEvent {
    /// A session was opened.
    Opened(SessionOpened),
    /// A session's idle expiry was extended.
    Refreshed(SessionRefreshed),
    /// A session expired at its effective deadline.
    Expired(SessionExpired),
    /// A session was revoked.
    Revoked(SessionRevoked),
}

impl SessionEvent {
    /// Returns the event sequence.
    #[must_use]
    pub const fn sequence(&self) -> u64 {
        match self {
            Self::Opened(event) => event.sequence(),
            Self::Refreshed(event) => event.sequence(),
            Self::Expired(event) => event.sequence(),
            Self::Revoked(event) => event.sequence(),
        }
    }

    /// Returns the session identity this event names.
    #[must_use]
    pub const fn session_id(&self) -> &SessionId {
        match self {
            Self::Opened(event) => event.session_id(),
            Self::Refreshed(event) => event.session_id(),
            Self::Expired(event) => event.session_id(),
            Self::Revoked(event) => event.session_id(),
        }
    }

    /// Returns when this event was observed.
    ///
    /// [`SessionEvent::Opened`] maps to that event's `opened_at`, which is exactly the open
    /// command's `observed_at`.
    #[must_use]
    pub const fn observed_at(&self) -> SessionInstant {
        match self {
            Self::Opened(event) => event.opened_at(),
            Self::Refreshed(event) => event.observed_at(),
            Self::Expired(event) => event.observed_at(),
            Self::Revoked(event) => event.observed_at(),
        }
    }
}

/// Which redundant verification field a persisted event forged.
///
/// Closed and non-secret: the error carries no arbitrary field names, source payloads or rendered
/// event values.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventDerivedField {
    /// `SessionRefreshed.effective_expires_at`.
    RefreshEffectiveExpiresAt,
    /// `SessionExpired.expired_at`.
    ExpiredAt,
    /// `SessionExpired.cause`.
    ExpiryCause,
}

/// Why one decision or evolution failed.
///
/// Small, `Copy` and non-echoing: variants may report a failure kind, safe expected/actual
/// revisions and a terminal status, and never credential text, a provider subject, rejected
/// secret-derived bytes, a serialized evidence value or caller-provided reason text.
///
/// A failed command and a failed event application both leave the previous snapshot unchanged and
/// produce no partial event or snapshot.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionDomainError {
    /// `credential_not_after` was present and the open instant was at or after it.
    CredentialEvidenceExpired,
    /// The open instant preceded the authentication instant.
    InvalidTimeOrder,
    /// Checked deadline arithmetic overflowed.
    DeadlineOverflow,
    /// A non-open command named an aggregate that does not exist.
    SessionNotFound,
    /// An open command named an aggregate that already exists.
    SessionAlreadyExists,
    /// The command or event named a different session than the supplied state.
    SessionIdMismatch,
    /// The optimistic-concurrency claim did not match the aggregate.
    RevisionMismatch {
        /// The caller's claim.
        expected: u64,
        /// The aggregate's truth.
        actual: u64,
    },
    /// The revision counter is exhausted at `u64::MAX`.
    RevisionOverflow,
    /// The aggregate is `Expired` or `Revoked` and cannot mutate or resurrect.
    TerminalSession {
        /// The terminal status, reported as a fact rather than accepted as an input.
        status: SessionStatus,
    },
    /// The observed instant preceded the last applied transition.
    NonMonotoneTime,
    /// An expire command was observed before the effective deadline.
    SessionNotYetExpired,
    /// A refresh recomputed a deadline that does not strictly advance the current one.
    NoEffectiveRefresh,
    /// The event's sequence was not the exact next revision.
    EventSequenceMismatch {
        /// The derived next revision.
        expected: u64,
        /// The event's own sequence.
        actual: u64,
    },
    /// The event's `observed_at` was on the wrong side of the effective deadline for its kind.
    ///
    /// Payload-free: the two instants involved are already in the caller's own snapshot and event.
    EventTimeOutsideValidity,
    /// The event/state pair is not in the §8 apply table.
    IllegalEventForState,
    /// A redundant verification field did not equal the value evolution recomputed.
    EventDerivedFieldMismatch {
        /// Which field disagreed.
        field: EventDerivedField,
    },
}

impl fmt::Display for SessionDomainError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CredentialEvidenceExpired => {
                formatter.write_str("credential evidence had already expired at open")
            }
            Self::InvalidTimeOrder => {
                formatter.write_str("open instant precedes the authentication instant")
            }
            Self::DeadlineOverflow => formatter.write_str("checked deadline arithmetic overflowed"),
            Self::SessionNotFound => formatter.write_str("session aggregate does not exist"),
            Self::SessionAlreadyExists => formatter.write_str("session aggregate already exists"),
            Self::SessionIdMismatch => {
                formatter.write_str("session identity does not match the supplied state")
            }
            Self::RevisionMismatch { expected, actual } => write!(
                formatter,
                "expected revision {expected} does not match aggregate revision {actual}"
            ),
            Self::RevisionOverflow => formatter.write_str("session revision counter is exhausted"),
            Self::TerminalSession { .. } => {
                formatter.write_str("session is terminal and cannot transition")
            }
            Self::NonMonotoneTime => {
                formatter.write_str("observed instant precedes the last transition")
            }
            Self::SessionNotYetExpired => {
                formatter.write_str("session has not reached its effective deadline")
            }
            Self::NoEffectiveRefresh => {
                formatter.write_str("refresh does not advance the effective deadline")
            }
            Self::EventSequenceMismatch { expected, actual } => write!(
                formatter,
                "expected event sequence {expected} but the event carries {actual}"
            ),
            Self::EventTimeOutsideValidity => {
                formatter.write_str("event time is outside the validity window for its kind")
            }
            Self::IllegalEventForState => {
                formatter.write_str("event is not legal for the current session state")
            }
            Self::EventDerivedFieldMismatch { .. } => {
                formatter.write_str("event derived field does not match the recomputed value")
            }
        }
    }
}

impl Error for SessionDomainError {}

/// Adds a duration to an instant, failing closed on overflow.
const fn checked_deadline(
    base: SessionInstant,
    duration: SessionDuration,
) -> Result<SessionInstant, SessionDomainError> {
    match base.millis.checked_add(duration.millis) {
        Some(millis) => Ok(SessionInstant::from_unix_millis(millis)),
        None => Err(SessionDomainError::DeadlineOverflow),
    }
}

/// Returns the effective deadline: the minimum of the idle candidate, the policy-absolute deadline
/// and the credential deadline when present.
const fn effective_deadline(
    idle_candidate: SessionInstant,
    absolute_expires_at: SessionInstant,
    credential_not_after: Option<SessionInstant>,
) -> SessionInstant {
    let mut effective = if idle_candidate.millis <= absolute_expires_at.millis {
        idle_candidate
    } else {
        absolute_expires_at
    };
    if let Some(credential) = credential_not_after
        && credential.millis < effective.millis
    {
        effective = credential;
    }
    effective
}

/// Selects the expiry cause from the deadline that equals the effective deadline, with the
/// deterministic tie precedence `Credential > Absolute > Idle`.
const fn expiry_cause(
    effective_expires_at: SessionInstant,
    absolute_expires_at: SessionInstant,
    credential_not_after: Option<SessionInstant>,
) -> SessionExpiryCause {
    if let Some(credential) = credential_not_after
        && credential.millis == effective_expires_at.millis
    {
        return SessionExpiryCause::Credential;
    }
    if absolute_expires_at.millis == effective_expires_at.millis {
        return SessionExpiryCause::Absolute;
    }
    SessionExpiryCause::Idle
}

/// The deadlines an accepted open derives, recomputed identically by [`decide`] and [`evolve`].
struct OpenDerivation {
    absolute_expires_at: SessionInstant,
    effective_expires_at: SessionInstant,
}

/// Applies §3's open conditions and derives both deadlines, in §7's open precedence.
///
/// Precondition: `evidence` and `policy` are already validated values, so no shape failure can
/// reach here.
/// Postcondition: on `Ok`, `effective_expires_at > opened_at` and
/// `absolute_expires_at > opened_at`, which is the `Active` invariant
/// `effective_expires_at > last_transition_at` at open.
fn derive_open(
    evidence: &SessionCredentialEvidence,
    policy: &SessionPolicy,
    opened_at: SessionInstant,
) -> Result<OpenDerivation, SessionDomainError> {
    if opened_at.millis < evidence.authenticated_at.millis {
        return Err(SessionDomainError::InvalidTimeOrder);
    }
    if let Some(not_after) = evidence.credential_not_after
        && opened_at.millis >= not_after.millis
    {
        return Err(SessionDomainError::CredentialEvidenceExpired);
    }
    let absolute_expires_at = checked_deadline(opened_at, policy.absolute_timeout)?;
    let idle_candidate = checked_deadline(opened_at, policy.idle_timeout)?;
    // Both durations are non-zero by type and both additions are checked, so the only way a
    // derived deadline is not strictly later than `opened_at` is the overflow already named.
    // Asserted rather than assumed, so evolution has an explicit postcondition to re-derive.
    if absolute_expires_at.millis <= opened_at.millis || idle_candidate.millis <= opened_at.millis {
        return Err(SessionDomainError::DeadlineOverflow);
    }
    Ok(OpenDerivation {
        absolute_expires_at,
        effective_expires_at: effective_deadline(
            idle_candidate,
            absolute_expires_at,
            evidence.credential_not_after,
        ),
    })
}

/// Recomputes a refresh's candidate effective deadline from prior state.
fn refreshed_deadline(
    state: &SessionSnapshot,
    observed_at: SessionInstant,
) -> Result<SessionInstant, SessionDomainError> {
    let candidate = checked_deadline(observed_at, state.idle_timeout)?;
    Ok(effective_deadline(
        candidate,
        state.absolute_expires_at,
        state.credential_not_after,
    ))
}

/// Builds the expiry event an at-or-after-deadline observation produces on an active session.
fn expiry_event(
    state: &SessionSnapshot,
    observed_at: SessionInstant,
    sequence: u64,
) -> SessionEvent {
    SessionEvent::Expired(SessionExpired::new(
        sequence,
        state.session_id.clone(),
        observed_at,
        state.effective_expires_at,
        expiry_cause(
            state.effective_expires_at,
            state.absolute_expires_at,
            state.credential_not_after,
        ),
    ))
}

/// Decides one command against caller-supplied state, producing at most one immutable event.
///
/// `None` is the empty aggregate. Because the caller supplies the state rather than the domain
/// looking it up, a command naming a different session than the state it was given is a real,
/// reachable [`SessionDomainError::SessionIdMismatch`] on this path, not only on the replay path.
///
/// This function is pure: it reads no clock, resolves no credential, generates no identifier,
/// touches no store and appends nothing. `expected_revision` is validated as
/// optimistic-concurrency *intent*; the compare-and-append that would make it durable belongs to
/// `M00-B4`.
///
/// # Errors
///
/// Returns [`SessionDomainError`] in the exact precedence `platform-session/v0` §7 freezes: for a
/// non-open command on an existing aggregate, session-identity mismatch, then revision mismatch,
/// then terminal mutation, then revision exhaustion, then non-monotone time, then time-derived
/// expiry, then command-specific legality. No lower-precedence failure may hide a higher-precedence
/// one.
pub fn decide(
    state: Option<&SessionSnapshot>,
    command: &SessionCommand,
) -> Result<SessionEvent, SessionDomainError> {
    let SessionCommand::Open(open) = command else {
        return decide_existing(state, command);
    };
    // Open precedence: an existing aggregate answers before revision, time ordering, credential
    // expiry or deadline arithmetic.
    if state.is_some() {
        return Err(SessionDomainError::SessionAlreadyExists);
    }
    if open.expected_revision != 0 {
        return Err(SessionDomainError::RevisionMismatch {
            expected: open.expected_revision,
            actual: 0,
        });
    }
    derive_open(&open.credential_evidence, &open.policy, open.observed_at)?;
    Ok(SessionEvent::Opened(SessionOpened::new(
        1,
        open.session_id.clone(),
        open.credential_evidence.clone(),
        open.policy,
        open.observed_at,
    )))
}

/// Decides a refresh, expire or revoke command in §7's existing-aggregate precedence.
fn decide_existing(
    state: Option<&SessionSnapshot>,
    command: &SessionCommand,
) -> Result<SessionEvent, SessionDomainError> {
    let Some(state) = state else {
        return Err(SessionDomainError::SessionNotFound);
    };
    if command.session_id() != &state.session_id {
        return Err(SessionDomainError::SessionIdMismatch);
    }
    if command.expected_revision() != state.revision {
        return Err(SessionDomainError::RevisionMismatch {
            expected: command.expected_revision(),
            actual: state.revision,
        });
    }
    // Terminal is checked BEFORE revision exhaustion, so §6.3's flat statement holds with no
    // exception at `u64::MAX`: a terminal session will never emit another event, so reporting its
    // exhausted counter would describe the less decisive of two facts.
    if !matches!(state.status, SessionStatus::Active) {
        return Err(SessionDomainError::TerminalSession {
            status: state.status,
        });
    }
    let Some(sequence) = state.revision.checked_add(1) else {
        return Err(SessionDomainError::RevisionOverflow);
    };
    let observed_at = command.observed_at();
    if observed_at.millis < state.last_transition_at.millis {
        return Err(SessionDomainError::NonMonotoneTime);
    }
    // Time-derived expiry answers before refresh or revoke semantics, so an already expired
    // session cannot be refreshed or relabeled by a later command.
    if observed_at.millis >= state.effective_expires_at.millis {
        return Ok(expiry_event(state, observed_at, sequence));
    }
    match command {
        SessionCommand::Refresh(_) => {
            let candidate = refreshed_deadline(state, observed_at)?;
            if candidate.millis <= state.effective_expires_at.millis {
                return Err(SessionDomainError::NoEffectiveRefresh);
            }
            Ok(SessionEvent::Refreshed(SessionRefreshed::new(
                sequence,
                state.session_id.clone(),
                observed_at,
                candidate,
            )))
        }
        SessionCommand::Expire(_) => Err(SessionDomainError::SessionNotYetExpired),
        SessionCommand::Revoke(_) => Ok(SessionEvent::Revoked(SessionRevoked::new(
            sequence,
            state.session_id.clone(),
            observed_at,
        ))),
        SessionCommand::Open(_) => Err(SessionDomainError::SessionAlreadyExists),
    }
}

/// Applies one persisted event to caller-supplied state, producing the next snapshot.
///
/// Evolution never trusts a serialized snapshot or a caller-supplied derived deadline: for an
/// empty aggregate it revalidates every §2–§3 open invariant from the event and derives the
/// deadlines itself, and for an existing aggregate it recomputes each redundant verification field
/// with the same functions [`decide`] uses, so it cannot accept a persisted event that `decide`
/// could never have emitted.
///
/// Replaying the same validated `SessionOpened` plus the same ordered events reconstructs a
/// structurally equal [`SessionSnapshot`]. Replay reads no clock, reloads no policy, resolves no
/// credential, calls no adapter and writes no evidence.
///
/// # Errors
///
/// Returns [`SessionDomainError`] for a gap, a duplicate sequence, an out-of-order event, a
/// cross-session event, a forged derived field, an event time outside the guard's validity window,
/// an illegal event/state pair, or a failed open invariant. Each returns no partial snapshot.
pub fn evolve(
    state: Option<&SessionSnapshot>,
    event: &SessionEvent,
) -> Result<SessionSnapshot, SessionDomainError> {
    let current_revision = state.map_or(0, |snapshot| snapshot.revision);
    let Some(next_revision) = current_revision.checked_add(1) else {
        return Err(SessionDomainError::RevisionOverflow);
    };
    if event.sequence() != next_revision {
        return Err(SessionDomainError::EventSequenceMismatch {
            expected: next_revision,
            actual: event.sequence(),
        });
    }
    let Some(state) = state else {
        return evolve_open(event);
    };
    if event.session_id() != &state.session_id {
        return Err(SessionDomainError::SessionIdMismatch);
    }
    if event.observed_at().millis < state.last_transition_at.millis {
        return Err(SessionDomainError::NonMonotoneTime);
    }
    if !matches!(state.status, SessionStatus::Active) {
        return Err(SessionDomainError::IllegalEventForState);
    }
    match event {
        SessionEvent::Opened(_) => Err(SessionDomainError::IllegalEventForState),
        SessionEvent::Refreshed(refreshed) => evolve_refreshed(state, refreshed, next_revision),
        SessionEvent::Expired(expired) => evolve_expired(state, expired, next_revision),
        SessionEvent::Revoked(revoked) => evolve_revoked(state, revoked, next_revision),
    }
}

/// Applies the only event legal for an empty aggregate, revalidating every open invariant.
fn evolve_open(event: &SessionEvent) -> Result<SessionSnapshot, SessionDomainError> {
    let SessionEvent::Opened(opened) = event else {
        return Err(SessionDomainError::IllegalEventForState);
    };
    let derivation = derive_open(
        &opened.credential_evidence,
        &opened.policy,
        opened.opened_at,
    )?;
    let evidence = &opened.credential_evidence;
    Ok(SessionSnapshot {
        session_id: opened.session_id.clone(),
        tenant_id: evidence.tenant_id.clone(),
        user_id: evidence.user_id.clone(),
        auth_adapter_id: evidence.auth_adapter_id.clone(),
        evidence_digest: evidence.evidence_digest.clone(),
        authenticated_at: evidence.authenticated_at,
        credential_not_after: evidence.credential_not_after,
        opened_at: opened.opened_at,
        last_transition_at: opened.opened_at,
        idle_timeout: opened.policy.idle_timeout,
        absolute_timeout: opened.policy.absolute_timeout,
        effective_expires_at: derivation.effective_expires_at,
        absolute_expires_at: derivation.absolute_expires_at,
        status: SessionStatus::Active,
        revision: 1,
    })
}

/// Applies a refreshed event to an active session.
///
/// Guards run in the order §8's table lists them: the apply-time validity window, then the strict
/// advance, then exact agreement with the recomputed deadline.
fn evolve_refreshed(
    state: &SessionSnapshot,
    event: &SessionRefreshed,
    revision: u64,
) -> Result<SessionSnapshot, SessionDomainError> {
    if event.observed_at.millis >= state.effective_expires_at.millis {
        return Err(SessionDomainError::EventTimeOutsideValidity);
    }
    let recomputed = refreshed_deadline(state, event.observed_at)?;
    if recomputed.millis <= state.effective_expires_at.millis {
        return Err(SessionDomainError::NoEffectiveRefresh);
    }
    if recomputed.millis != event.effective_expires_at.millis {
        return Err(SessionDomainError::EventDerivedFieldMismatch {
            field: EventDerivedField::RefreshEffectiveExpiresAt,
        });
    }
    Ok(SessionSnapshot {
        effective_expires_at: event.effective_expires_at,
        last_transition_at: event.observed_at,
        revision,
        ..state.clone()
    })
}

/// Applies an expired event to an active session.
///
/// The three guards run in exactly this order, so two implementations report the same failure for
/// a multiply-forged event. Together they imply `observed_at >= expired_at` for every event this
/// function accepts — which is the property [`SessionExpired::new`] declines to check, obtained
/// from the aggregate that can actually establish it.
fn evolve_expired(
    state: &SessionSnapshot,
    event: &SessionExpired,
    revision: u64,
) -> Result<SessionSnapshot, SessionDomainError> {
    if event.observed_at.millis < state.effective_expires_at.millis {
        return Err(SessionDomainError::EventTimeOutsideValidity);
    }
    if event.expired_at.millis != state.effective_expires_at.millis {
        return Err(SessionDomainError::EventDerivedFieldMismatch {
            field: EventDerivedField::ExpiredAt,
        });
    }
    let cause = expiry_cause(
        state.effective_expires_at,
        state.absolute_expires_at,
        state.credential_not_after,
    );
    if event.cause != cause {
        return Err(SessionDomainError::EventDerivedFieldMismatch {
            field: EventDerivedField::ExpiryCause,
        });
    }
    Ok(SessionSnapshot {
        status: SessionStatus::Expired {
            expired_at: event.expired_at,
            observed_at: event.observed_at,
            cause,
        },
        last_transition_at: event.observed_at,
        revision,
        ..state.clone()
    })
}

/// Applies a revoked event to an active session, preserving its effective deadline.
fn evolve_revoked(
    state: &SessionSnapshot,
    event: &SessionRevoked,
    revision: u64,
) -> Result<SessionSnapshot, SessionDomainError> {
    if event.observed_at.millis >= state.effective_expires_at.millis {
        return Err(SessionDomainError::EventTimeOutsideValidity);
    }
    Ok(SessionSnapshot {
        status: SessionStatus::Revoked {
            revoked_at: event.observed_at,
        },
        last_transition_at: event.observed_at,
        revision,
        ..state.clone()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A shape-valid digest for fixtures. It fingerprints nothing and is not secret-derived.
    const FIXTURE_DIGEST: &str =
        "sha256:00000000000000000000000000000000000000000000000000000000cafebabe";

    fn session() -> SessionId {
        SessionId::parse("session:example").expect("fixture")
    }

    fn at(millis: u64) -> SessionInstant {
        SessionInstant::from_unix_millis(millis)
    }

    /// The snapshot an open at `1_000` really produces, built by the production path.
    fn opened() -> SessionSnapshot {
        let evidence = SessionCredentialEvidence::new(
            TenantId::parse("tenant:example").expect("fixture"),
            UserId::parse("user:example").expect("fixture"),
            AuthAdapterId::parse("ustc.cas").expect("fixture"),
            CredentialEvidenceDigest::parse(FIXTURE_DIGEST).expect("fixture"),
            at(1_000),
            None,
        )
        .expect("fixture");
        let policy = SessionPolicy::new(
            SessionDuration::from_millis(100).expect("fixture"),
            SessionDuration::from_millis(1_000).expect("fixture"),
        );
        let command =
            SessionCommand::Open(OpenSession::new(session(), evidence, policy, at(1_000), 0));
        let Ok(event) = decide(None, &command) else {
            panic!("the fixture open must be accepted");
        };
        let Ok(snapshot) = evolve(None, &event) else {
            panic!("the fixture open must apply");
        };
        snapshot
    }

    /// That same snapshot with only `revision` — and, where a case needs it, `status` — overridden.
    ///
    /// This is the one aggregate no public call sequence can produce: [`SessionSnapshot`] has no
    /// public constructor and no `Deserialize`, and [`evolve`] sets `revision` only to
    /// `current + 1` from a base of `1`, so reaching `u64::MAX` would take ~1.8e19 accepted
    /// evolutions. The guards at that value are nonetheless real and observable, so the fixture is
    /// built here — inside the module that owns the fields — rather than left unproven. Every
    /// other field is exactly what `evolve` derived, so no invariant is fabricated, and nothing
    /// here is reachable from outside this module: there is no public or feature-gated hook, and
    /// the production items above are untouched.
    fn at_ceiling(status: SessionStatus) -> SessionSnapshot {
        SessionSnapshot {
            status,
            revision: u64::MAX,
            ..opened()
        }
    }

    /// `AUTH-018` library leg: terminal state answers before revision exhaustion, so §6.3's flat
    /// statement holds with no exception at `u64::MAX`.
    #[test]
    fn terminal_precedence_holds_at_the_revision_ceiling() {
        for status in [
            SessionStatus::Revoked {
                revoked_at: at(1_050),
            },
            SessionStatus::Expired {
                expired_at: at(1_100),
                observed_at: at(1_100),
                cause: SessionExpiryCause::Idle,
            },
        ] {
            let terminal = at_ceiling(status);
            assert_eq!(terminal.revision(), u64::MAX);
            // Every later command, at instants that are in turn ordinary, stale and past the
            // effective deadline — so the ordering is pinned against each lower-precedence fault
            // it could compete with, not only against exhaustion.
            for observed in [at(1_060), at(1), at(9_999)] {
                for command in [
                    SessionCommand::Refresh(RefreshSession::new(session(), observed, u64::MAX)),
                    SessionCommand::Expire(ExpireSession::new(session(), observed, u64::MAX)),
                    SessionCommand::Revoke(RevokeSession::new(session(), observed, u64::MAX)),
                ] {
                    assert_eq!(
                        decide(Some(&terminal), &command),
                        Err(SessionDomainError::TerminalSession { status }),
                        "terminal state must answer before revision exhaustion"
                    );
                }
            }
        }
    }

    /// `AUTH-019` library leg: both paths increment through a checked add and fail closed at the
    /// ceiling, and the wrapped sequence `0` a wrapping increment would compute is rejected.
    #[test]
    fn revision_ceiling_fails_closed_on_decide_and_evolve() {
        let active = at_ceiling(SessionStatus::Active);
        assert_eq!(active.revision(), u64::MAX);
        // The session is otherwise live — only the counter is exhausted — so nothing but the
        // exhaustion guard can be answering below.
        assert!(active.admits_at(at(1_000)));
        assert_eq!(active.status(), SessionStatus::Active);

        // Decision: exhaustion answers, and it answers ahead of non-monotone time and of
        // time-derived expiry, which is §7's item 5 sitting above items 6 and 7.
        for observed in [at(1_000), at(1_050), at(1), at(9_999)] {
            for command in [
                SessionCommand::Refresh(RefreshSession::new(session(), observed, u64::MAX)),
                SessionCommand::Expire(ExpireSession::new(session(), observed, u64::MAX)),
                SessionCommand::Revoke(RevokeSession::new(session(), observed, u64::MAX)),
            ] {
                assert_eq!(
                    decide(Some(&active), &command),
                    Err(SessionDomainError::RevisionOverflow)
                );
            }
        }

        // Evolution: the checked increment fails BEFORE the sequence comparison, so the forged
        // wrapped `0` — precisely the sequence a wrapping increment would have derived and then
        // matched — is rejected rather than applied. A wrapping increment would accept it and
        // return a snapshot at revision `0`.
        for forged in [0_u64, 1, u64::MAX] {
            for event in [
                SessionEvent::Refreshed(SessionRefreshed::new(
                    forged,
                    session(),
                    at(1_050),
                    at(1_150),
                )),
                SessionEvent::Revoked(SessionRevoked::new(forged, session(), at(1_050))),
                SessionEvent::Expired(SessionExpired::new(
                    forged,
                    session(),
                    at(1_100),
                    at(1_100),
                    SessionExpiryCause::Idle,
                )),
            ] {
                assert_eq!(
                    evolve(Some(&active), &event),
                    Err(SessionDomainError::RevisionOverflow)
                );
            }
        }
    }
}
