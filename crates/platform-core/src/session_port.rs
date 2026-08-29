//! Bounded ports around the pure `platform-session/v0` lifecycle kernel.
//!
//! This module deliberately does not authenticate credentials, read a clock, persist state, or
//! create a production session.  It owns only least-authority interfaces and a replay-derived
//! history carrier.  Concrete adapters remain outer-boundary responsibilities.
//!
//! `SessionHistory` cannot be forged or decoded as a snapshot:
//!
//! ```compile_fail
//! use ustc_campus_agent_core::session_port::SessionHistory;
//!
//! let _ = SessionHistory::default();
//! ```
//!
//! ```compile_fail
//! use ustc_campus_agent_core::session_port::SessionHistory;
//!
//! let _ = SessionHistory { events: Vec::new(), snapshot: todo!() };
//! ```
//!
//! `SecretRef` exposes neither raw bytes nor path conversion:
//!
//! ```compile_fail
//! use ustc_campus_agent_core::session_port::SecretRef;
//!
//! let secret_ref = SecretRef::parse("secret-ref:demo").expect("checked fixture");
//! let _ = secret_ref.as_bytes();
//! ```
//!
//! ```compile_fail
//! use std::path::PathBuf;
//! use ustc_campus_agent_core::session_port::SecretRef;
//!
//! let secret_ref = SecretRef::parse("secret-ref:demo").expect("checked fixture");
//! let _: PathBuf = secret_ref.into();
//! ```

use std::fmt;

use serde::{Deserialize, Deserializer, Serialize, Serializer, de};

use crate::identity::SessionId;
use crate::session::{
    AuthAdapterId, CredentialEvidenceDigest, SessionEvent, SessionInstant, SessionSnapshot, evolve,
};

const SECRET_REF_PREFIX: &str = "secret-ref:";
const SECRET_REF_MAX_SLUG_BYTES: usize = 96;

/// Logical reference consumed only by a trusted credential adapter.
///
/// The spelling is not a filesystem path and does not itself prove authentication.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SecretRef(String);

impl SecretRef {
    /// Parses the canonical `secret-ref:<slug>` spelling without normalization.
    #[must_use]
    pub fn parse(value: impl Into<String>) -> Option<Self> {
        let value = value.into();
        let slug = value.strip_prefix(SECRET_REF_PREFIX)?;
        if slug.is_empty()
            || slug.len() > SECRET_REF_MAX_SLUG_BYTES
            || !slug.is_ascii()
            || !slug.as_bytes().first().is_some_and(u8::is_ascii_lowercase)
                && !slug.as_bytes().first().is_some_and(u8::is_ascii_digit)
            || !slug.bytes().all(|byte| {
                byte.is_ascii_lowercase() || byte.is_ascii_digit() || b"._-".contains(&byte)
            })
        {
            return None;
        }
        Some(Self(value))
    }

    /// Returns the canonical logical reference spelling.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for SecretRef {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("SecretRef(<redacted>)")
    }
}

impl Serialize for SecretRef {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for SecretRef {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::parse(value).ok_or_else(|| de::Error::custom("invalid secret reference"))
    }
}

/// Closed repository failure surface. Adapter diagnostics never cross this boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionRepositoryError {
    Unavailable,
    Corrupt,
    InvalidEvent,
    LimitExceeded,
    InternalInvariant,
}

/// Result of a compare-and-append attempt.
#[derive(Clone, PartialEq, Eq)]
pub enum SessionAppendOutcome {
    Appended(SessionHistory),
    AlreadySame(SessionHistory),
    Conflict { current_revision: Option<u64> },
}

/// Closed clock failure surface.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionClockError {
    Unavailable,
}

/// Closed credential-evidence adapter failure surface.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CredentialEvidencePortError {
    Unavailable,
    UnknownSecretRef,
    InternalInvariant,
}

/// Complete event history and its replay-derived current snapshot.
///
/// There is intentionally no snapshot deserializer, mutable accessor, unchecked constructor,
/// `Default`, `Debug`, or display surface.
#[derive(Clone, PartialEq, Eq)]
pub struct SessionHistory {
    events: Vec<SessionEvent>,
    snapshot: SessionSnapshot,
}

impl SessionHistory {
    /// Replays one non-empty complete history through the production lifecycle kernel.
    ///
    /// # Errors
    ///
    /// Returns [`SessionRepositoryError::Corrupt`] when the history is empty or replay rejects any
    /// event. No partial history is returned.
    pub fn try_from_events(events: Vec<SessionEvent>) -> Result<Self, SessionRepositoryError> {
        let snapshot = replay_history(&events)?;
        Ok(Self { events, snapshot })
    }

    /// Returns the complete immutable event sequence.
    #[must_use]
    pub fn events(&self) -> &[SessionEvent] {
        &self.events
    }

    /// Returns the replay-derived current snapshot.
    #[must_use]
    pub const fn snapshot(&self) -> &SessionSnapshot {
        &self.snapshot
    }

    /// Returns the retained session identity.
    #[must_use]
    pub const fn session_id(&self) -> &SessionId {
        self.snapshot.session_id()
    }

    /// Returns the replay-derived current revision.
    #[must_use]
    pub const fn revision(&self) -> u64 {
        self.snapshot.revision()
    }
}

fn replay_history(events: &[SessionEvent]) -> Result<SessionSnapshot, SessionRepositoryError> {
    let mut current = None;
    for event in events {
        current =
            Some(evolve(current.as_ref(), event).map_err(|_| SessionRepositoryError::Corrupt)?);
    }
    current.ok_or(SessionRepositoryError::Corrupt)
}

/// Read-only session-history repository boundary.
pub trait SessionHistoryReadPort {
    fn load_history(
        &mut self,
        session_id: &SessionId,
    ) -> Result<Option<SessionHistory>, SessionRepositoryError>;
}

/// Optimistically fenced append boundary.
pub trait SessionHistoryAppendPort: SessionHistoryReadPort {
    fn compare_and_append(
        &mut self,
        session_id: &SessionId,
        expected_revision: Option<u64>,
        event: &SessionEvent,
    ) -> Result<SessionAppendOutcome, SessionRepositoryError>;
}

/// Trusted clock boundary for later session orchestration.
pub trait SessionClockPort {
    fn now(&mut self) -> Result<SessionInstant, SessionClockError>;
}

/// Fingerprints already-verified adapter evidence referenced by a logical handle.
///
/// Implementing this port does not authenticate a caller and does not turn a digest into
/// authorization.
pub trait CredentialEvidencePort {
    fn fingerprint_adapter_evidence(
        &mut self,
        auth_adapter_id: &AuthAdapterId,
        secret_ref: &SecretRef,
    ) -> Result<CredentialEvidenceDigest, CredentialEvidencePortError>;
}
