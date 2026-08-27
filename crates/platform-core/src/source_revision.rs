//! Immutable accepted source-revision values for the bounded M60 evidence path.
//!
//! This module owns value shape only. It performs no retrieval, parsing, storage,
//! baseline advancement, publication, clock reads, or model calls. A
//! `DemoReviewed` revision is an honestly labelled administrator-reviewed
//! snapshot, not a live or official-source claim.

use std::error::Error;
use std::fmt;

use sha2::{Digest, Sha256};

use crate::source_registry::{SourceId, SourceReviewEvidenceId, SourceReviewerId, SourceUrl};

const MAX_ID_BYTES: usize = 128;

fn valid_id(value: &str) -> bool {
    let bytes = value.as_bytes();
    if bytes.is_empty() || bytes.len() > MAX_ID_BYTES {
        return false;
    }
    let boundary = |byte: u8| byte.is_ascii_lowercase() || byte.is_ascii_digit();
    if !boundary(bytes[0]) || !boundary(bytes[bytes.len() - 1]) {
        return false;
    }
    bytes.iter().all(|byte| {
        byte.is_ascii_lowercase()
            || byte.is_ascii_digit()
            || matches!(byte, b'-' | b'.' | b'_' | b':')
    })
}

macro_rules! revision_id {
    ($(#[$attribute:meta])* $name:ident) => {
        $(#[$attribute])*
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(String);

        impl $name {
            /// Parses one bounded canonical identity.
            pub fn parse(value: impl Into<String>) -> Result<Self, SourceRevisionError> {
                let value = value.into();
                if !valid_id(&value) {
                    return Err(SourceRevisionError::InvalidIdentity {
                        kind: stringify!($name),
                    });
                }
                Ok(Self(value))
            }

            #[must_use]
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str(&self.0)
            }
        }
    };
}

revision_id! {
    /// Stable identity of one immutable raw snapshot.
    RawSnapshotId
}
revision_id! {
    /// Stable identity of one immutable normalized snapshot.
    NormalizedSnapshotId
}
revision_id! {
    /// Stable identity of one immutable source revision.
    SourceRevisionId
}
revision_id! {
    /// Exact deterministic parser identity and version.
    ParserIdentity
}

/// One exact lowercase `sha256:` digest.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RevisionSha256(String);

impl RevisionSha256 {
    /// Constructs the canonical lowercase identity for exact SHA-256 bytes.
    #[must_use]
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        const HEX: &[u8; 16] = b"0123456789abcdef";

        let mut value = String::with_capacity(71);
        value.push_str("sha256:");
        for byte in bytes {
            value.push(char::from(HEX[usize::from(byte >> 4)]));
            value.push(char::from(HEX[usize::from(byte & 0x0f)]));
        }
        Self(value)
    }

    /// Parses `sha256:` followed by exactly 64 lowercase hexadecimal digits.
    pub fn parse(value: impl Into<String>) -> Result<Self, SourceRevisionError> {
        let value = value.into();
        let Some(hex) = value.strip_prefix("sha256:") else {
            return Err(SourceRevisionError::InvalidDigest);
        };
        if hex.len() != 64
            || !hex
                .bytes()
                .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
        {
            return Err(SourceRevisionError::InvalidDigest);
        }
        Ok(Self(value))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for RevisionSha256 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

/// One Unix-second evidence timestamp.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RevisionTimestamp(i64);

impl RevisionTimestamp {
    #[must_use]
    pub const fn from_unix_seconds(value: i64) -> Self {
        Self(value)
    }

    #[must_use]
    pub const fn unix_seconds(self) -> i64 {
        self.0
    }
}

/// Optional source-asserted semantic validity interval.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EffectiveInterval {
    from: Option<RevisionTimestamp>,
    to: Option<RevisionTimestamp>,
}

impl EffectiveInterval {
    /// Constructs an interval. When both bounds exist, `from <= to` is required.
    pub fn new(
        from: Option<RevisionTimestamp>,
        to: Option<RevisionTimestamp>,
    ) -> Result<Self, SourceRevisionError> {
        if matches!((from, to), (Some(start), Some(end)) if start > end) {
            return Err(SourceRevisionError::InvalidEffectiveInterval);
        }
        Ok(Self { from, to })
    }

    #[must_use]
    pub const fn from(self) -> Option<RevisionTimestamp> {
        self.from
    }

    #[must_use]
    pub const fn to(self) -> Option<RevisionTimestamp> {
        self.to
    }
}

/// Review provenance for an accepted immutable revision.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SourceRevisionProvenance {
    /// A non-personal snapshot explicitly reviewed for the demo. This is not a
    /// live-source, official-publication, or legal-permission claim.
    DemoReviewed {
        reviewer: SourceReviewerId,
        evidence: SourceReviewEvidenceId,
    },
}

/// M60-owned current health decision for one accepted revision.
///
/// Availability without a revision is represented by a product's typed
/// unavailable outcome rather than forged revision evidence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceRevisionHealth {
    Current,
    Stale,
    Conflicting,
}

/// One immutable, evidence-bound source revision accepted for a bounded product path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceRevision {
    revision_id: SourceRevisionId,
    source_id: SourceId,
    source_url: SourceUrl,
    raw_snapshot_id: RawSnapshotId,
    raw_sha256: RevisionSha256,
    normalized_snapshot_id: NormalizedSnapshotId,
    normalized_sha256: RevisionSha256,
    parser_identity: ParserIdentity,
    observed_at: RevisionTimestamp,
    published_at: Option<RevisionTimestamp>,
    effective_interval: EffectiveInterval,
    provenance: SourceRevisionProvenance,
}

impl SourceRevision {
    /// Constructs one honestly labelled immutable `DemoReviewed` revision.
    #[allow(clippy::too_many_arguments)]
    pub fn demo_reviewed(
        source_id: SourceId,
        source_url: SourceUrl,
        raw_snapshot_id: RawSnapshotId,
        raw_sha256: RevisionSha256,
        normalized_snapshot_id: NormalizedSnapshotId,
        normalized_sha256: RevisionSha256,
        parser_identity: ParserIdentity,
        observed_at: RevisionTimestamp,
        published_at: Option<RevisionTimestamp>,
        effective_interval: EffectiveInterval,
        reviewer: SourceReviewerId,
        review_evidence: SourceReviewEvidenceId,
    ) -> Self {
        let revision_id = derive_demo_revision_id(
            &source_id,
            &source_url,
            &raw_snapshot_id,
            &raw_sha256,
            &normalized_snapshot_id,
            &normalized_sha256,
            &parser_identity,
            observed_at,
            published_at,
            effective_interval,
            &reviewer,
            &review_evidence,
        );
        Self {
            revision_id,
            source_id,
            source_url,
            raw_snapshot_id,
            raw_sha256,
            normalized_snapshot_id,
            normalized_sha256,
            parser_identity,
            observed_at,
            published_at,
            effective_interval,
            provenance: SourceRevisionProvenance::DemoReviewed {
                reviewer,
                evidence: review_evidence,
            },
        }
    }

    #[must_use]
    pub fn revision_id(&self) -> &SourceRevisionId {
        &self.revision_id
    }
    #[must_use]
    pub fn source_id(&self) -> &SourceId {
        &self.source_id
    }
    #[must_use]
    pub fn source_url(&self) -> &SourceUrl {
        &self.source_url
    }
    #[must_use]
    pub fn raw_snapshot_id(&self) -> &RawSnapshotId {
        &self.raw_snapshot_id
    }
    #[must_use]
    pub fn raw_sha256(&self) -> &RevisionSha256 {
        &self.raw_sha256
    }
    #[must_use]
    pub fn normalized_snapshot_id(&self) -> &NormalizedSnapshotId {
        &self.normalized_snapshot_id
    }
    #[must_use]
    pub fn normalized_sha256(&self) -> &RevisionSha256 {
        &self.normalized_sha256
    }
    #[must_use]
    pub fn parser_identity(&self) -> &ParserIdentity {
        &self.parser_identity
    }
    #[must_use]
    pub const fn observed_at(&self) -> RevisionTimestamp {
        self.observed_at
    }
    #[must_use]
    pub const fn published_at(&self) -> Option<RevisionTimestamp> {
        self.published_at
    }
    #[must_use]
    pub const fn effective_interval(&self) -> EffectiveInterval {
        self.effective_interval
    }
    #[must_use]
    pub fn provenance(&self) -> &SourceRevisionProvenance {
        &self.provenance
    }
}

#[allow(clippy::too_many_arguments)]
fn derive_demo_revision_id(
    source_id: &SourceId,
    source_url: &SourceUrl,
    raw_snapshot_id: &RawSnapshotId,
    raw_sha256: &RevisionSha256,
    normalized_snapshot_id: &NormalizedSnapshotId,
    normalized_sha256: &RevisionSha256,
    parser_identity: &ParserIdentity,
    observed_at: RevisionTimestamp,
    published_at: Option<RevisionTimestamp>,
    effective_interval: EffectiveInterval,
    reviewer: &SourceReviewerId,
    review_evidence: &SourceReviewEvidenceId,
) -> SourceRevisionId {
    let mut hasher = Sha256::new();
    hasher.update(b"source-revision/demo-reviewed/v1\0");
    for value in [
        source_id.as_str(),
        source_url.as_str(),
        raw_snapshot_id.as_str(),
        raw_sha256.as_str(),
        normalized_snapshot_id.as_str(),
        normalized_sha256.as_str(),
        parser_identity.as_str(),
    ] {
        let bytes = value.as_bytes();
        hasher.update(u64::try_from(bytes.len()).unwrap_or(u64::MAX).to_be_bytes());
        hasher.update(bytes);
    }
    hasher.update(observed_at.unix_seconds().to_be_bytes());
    update_optional_timestamp(&mut hasher, published_at);
    update_optional_timestamp(&mut hasher, effective_interval.from());
    update_optional_timestamp(&mut hasher, effective_interval.to());
    for value in [reviewer.as_str(), review_evidence.as_str()] {
        let bytes = value.as_bytes();
        hasher.update(u64::try_from(bytes.len()).unwrap_or(u64::MAX).to_be_bytes());
        hasher.update(bytes);
    }
    SourceRevisionId(format!("revision:sha256:{:x}", hasher.finalize()))
}

fn update_optional_timestamp(hasher: &mut Sha256, timestamp: Option<RevisionTimestamp>) {
    match timestamp {
        Some(value) => {
            hasher.update([1]);
            hasher.update(value.unix_seconds().to_be_bytes());
        }
        None => hasher.update([0]),
    }
}

/// Deterministic construction failure for source-revision values.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceRevisionError {
    InvalidIdentity { kind: &'static str },
    InvalidDigest,
    InvalidEffectiveInterval,
}

impl fmt::Display for SourceRevisionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidIdentity { kind } => write!(formatter, "invalid {kind} identity"),
            Self::InvalidDigest => formatter.write_str("invalid source-revision sha256 digest"),
            Self::InvalidEffectiveInterval => {
                formatter.write_str("source-revision effective interval is reversed")
            }
        }
    }
}

impl Error for SourceRevisionError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_and_digest_values_fail_closed() {
        assert!(SourceRevisionId::parse("revision:demo:1").is_ok());
        assert!(SourceRevisionId::parse("Revision:Demo:1").is_err());
        assert!(RevisionSha256::parse(format!("sha256:{}", "a".repeat(64))).is_ok());
        assert!(RevisionSha256::parse(format!("sha256:{}", "A".repeat(64))).is_err());
    }

    #[test]
    fn reversed_effective_interval_is_rejected() {
        assert!(
            EffectiveInterval::new(
                Some(RevisionTimestamp::from_unix_seconds(2)),
                Some(RevisionTimestamp::from_unix_seconds(1)),
            )
            .is_err()
        );
    }
}
