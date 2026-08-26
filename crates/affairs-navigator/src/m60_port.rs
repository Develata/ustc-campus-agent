//! M71-owned retained-evidence port implemented by M60 (M71-v8n §8.2 / TD-2).
//!
//! The M71 application service is the sole caller. M60 owns retained
//! source/revision truth and implements the adapter. The port verifies
//! retained accepted/equal-contract revision evidence only — never arbitrary
//! live fetch, source discovery, publication, or baseline advancement.

use time::OffsetDateTime;

use crate::evidence::{M60RevisionRef, Sha256};
use crate::value::{
    AffairsValueError, AffairsValueErrorKind, ProcedureId, classify_id, value_error,
};

/// Minimum retained revision refs in one verification request.
pub const MIN_REVISION_REFS: usize = 1;
/// Maximum retained revision refs in one verification request.
pub const MAX_REVISION_REFS: usize = 16;

// ---------------------------------------------------------------------------
// Request
// ---------------------------------------------------------------------------

/// Nonempty ordered retained-evidence verification request. The M71 service
/// constructs this from its internal canonical artifact refs; neither M10 nor
/// the client can supply or observe those refs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct M60RetainedEvidenceRequest {
    procedure_id: ProcedureId,
    as_of: OffsetDateTime,
    revision_refs: Vec<M60RevisionRef>,
}

impl M60RetainedEvidenceRequest {
    /// Builds one checked request. `revision_refs` MUST be `1..=16`.
    ///
    /// # Errors
    ///
    /// Returns [`AffairsValueError`] when `revision_refs` is empty or exceeds
    /// 16.
    pub fn new(
        procedure_id: ProcedureId,
        as_of: OffsetDateTime,
        revision_refs: Vec<M60RevisionRef>,
    ) -> Result<Self, AffairsValueError> {
        let count = revision_refs.len();
        if count < MIN_REVISION_REFS {
            return Err(value_error(
                "M60RetainedEvidenceRequest",
                AffairsValueErrorKind::Empty,
            ));
        }
        if count > MAX_REVISION_REFS {
            return Err(value_error(
                "M60RetainedEvidenceRequest",
                AffairsValueErrorKind::TooLong {
                    max_bytes: MAX_REVISION_REFS,
                },
            ));
        }
        Ok(Self {
            procedure_id,
            as_of,
            revision_refs,
        })
    }

    #[must_use]
    pub fn procedure_id(&self) -> &ProcedureId {
        &self.procedure_id
    }

    #[must_use]
    pub fn as_of(&self) -> OffsetDateTime {
        self.as_of
    }

    #[must_use]
    pub fn revision_refs(&self) -> &[M60RevisionRef] {
        &self.revision_refs
    }
}

// ---------------------------------------------------------------------------
// Verification identity
// ---------------------------------------------------------------------------

/// M60-owned verification identity. Carried back through the sealed M71
/// lineage receipt; M10 creates its own DTO only after M71 seals the receipt.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct M60VerificationIdentity {
    verifier_id: String,
    verified_at: OffsetDateTime,
    evidence_contract_version: u16,
}

impl M60VerificationIdentity {
    /// Builds one checked verification identity.
    ///
    /// # Errors
    ///
    /// Returns [`AffairsValueError`] when `verifier_id` is empty, exceeds 128
    /// bytes, or is not M60 ID grammar, or when `evidence_contract_version`
    /// is zero.
    pub fn new(
        verifier_id: String,
        verified_at: OffsetDateTime,
        evidence_contract_version: u16,
    ) -> Result<Self, AffairsValueError> {
        if let Err(kind) = classify_id(&verifier_id) {
            return Err(value_error("M60VerificationIdentity", kind));
        }
        if evidence_contract_version == 0 {
            return Err(value_error(
                "M60VerificationIdentity",
                AffairsValueErrorKind::Empty,
            ));
        }
        Ok(Self {
            verifier_id,
            verified_at,
            evidence_contract_version,
        })
    }

    #[must_use]
    pub fn verifier_id(&self) -> &str {
        &self.verifier_id
    }

    #[must_use]
    pub fn verified_at(&self) -> OffsetDateTime {
        self.verified_at
    }

    #[must_use]
    pub const fn evidence_contract_version(&self) -> u16 {
        self.evidence_contract_version
    }
}

// ---------------------------------------------------------------------------
// Verified evidence set
// ---------------------------------------------------------------------------

/// The verified retained-evidence set returned by M60. `evidence_set_digest`
/// is the digest of the ordered retained-reference set, not any individual raw
/// source digest.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct M60VerifiedEvidenceSet {
    evidence_set_digest: Sha256,
    revision_count: u8,
    verification_identity: M60VerificationIdentity,
}

impl M60VerifiedEvidenceSet {
    /// Builds one checked verified evidence set.
    ///
    /// # Errors
    ///
    /// Returns [`AffairsValueError`] when `revision_count` is zero.
    pub fn new(
        evidence_set_digest: Sha256,
        revision_count: u8,
        verification_identity: M60VerificationIdentity,
    ) -> Result<Self, AffairsValueError> {
        if revision_count == 0 {
            return Err(value_error(
                "M60VerifiedEvidenceSet",
                AffairsValueErrorKind::Empty,
            ));
        }
        Ok(Self {
            evidence_set_digest,
            revision_count,
            verification_identity,
        })
    }

    #[must_use]
    pub fn evidence_set_digest(&self) -> &Sha256 {
        &self.evidence_set_digest
    }

    #[must_use]
    pub const fn revision_count(&self) -> u8 {
        self.revision_count
    }

    #[must_use]
    pub fn verification_identity(&self) -> &M60VerificationIdentity {
        &self.verification_identity
    }
}

// ---------------------------------------------------------------------------
// Unverified reason
// ---------------------------------------------------------------------------

/// Why M60 could not verify the retained evidence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum M60EvidenceUnverifiedReason {
    MissingRevision,
    DigestMismatch,
    RevokedOrUnaccepted,
    EffectiveIntervalMissing,
}

// ---------------------------------------------------------------------------
// Outcome and error
// ---------------------------------------------------------------------------

/// M60 verification outcome.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum M60RetainedEvidenceOutcome {
    Verified(M60VerifiedEvidenceSet),
    Unverified(M60EvidenceUnverifiedReason),
}

/// M60 port infrastructure error. Never becomes a public `NotFound` or
/// unverified semantic outcome; the M71 service maps these to
/// `GetProcedureError`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum M60EvidencePortError {
    StoreUnavailable,
    StoreCorrupted,
}

// ---------------------------------------------------------------------------
// Port trait
// ---------------------------------------------------------------------------

/// The M71-owned retained-evidence port. M60 owns retained source/revision
/// truth and implements the adapter. The M71 application service is the sole
/// caller.
pub trait M60ProcedureEvidencePort: Send + Sync {
    /// Verifies the retained accepted/equal-contract revision evidence for one
    /// request.
    ///
    /// # Errors
    ///
    /// Returns [`M60EvidencePortError`] on infrastructure failure
    /// (`StoreUnavailable` / `StoreCorrupted`). These remain M71
    /// infrastructure errors and are never public `NotFound` or an unverified
    /// semantic outcome.
    fn verify_retained(
        &self,
        request: &M60RetainedEvidenceRequest,
    ) -> Result<M60RetainedEvidenceOutcome, M60EvidencePortError>;
}

/// Helper suppressed from public API: M60 fixture adapters compute the
/// ordered-set digest from the normalized digests of the retained refs.
pub(crate) fn compute_evidence_set_digest(refs: &[M60RevisionRef]) -> Sha256 {
    use sha2::{Digest, Sha256 as Sha256Hasher};
    let mut hasher = Sha256Hasher::new();
    for rev in refs {
        hasher.update(rev.normalized_digest().as_str().as_bytes());
        hasher.update(b"\n");
    }
    let digest = hasher.finalize();
    let hex: String = digest.iter().map(|b| format!("{b:02x}")).collect();
    Sha256::new(format!("sha256:{hex}")).expect("sha256 output is always valid grammar")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::evidence::M60RevisionRef;
    use crate::value::SourceId;

    fn rev(source: &str) -> M60RevisionRef {
        M60RevisionRef::new(
            SourceId::parse(source).unwrap(),
            format!("rev:{source}"),
            OffsetDateTime::from_unix_timestamp(0).unwrap(),
            None,
            None,
            None,
            Sha256::new("sha256:0000000000000000000000000000000000000000000000000000000000000000")
                .unwrap(),
            Sha256::new("sha256:1111111111111111111111111111111111111111111111111111111111111111")
                .unwrap(),
        )
        .unwrap()
    }

    #[test]
    fn request_rejects_empty_and_overflow() {
        assert!(
            M60RetainedEvidenceRequest::new(
                ProcedureId::parse("p1").unwrap(),
                OffsetDateTime::from_unix_timestamp(0).unwrap(),
                Vec::new(),
            )
            .is_err()
        );

        let refs: Vec<_> = (0..17).map(|i| rev(&format!("s{i}"))).collect();
        assert!(
            M60RetainedEvidenceRequest::new(
                ProcedureId::parse("p1").unwrap(),
                OffsetDateTime::from_unix_timestamp(0).unwrap(),
                refs,
            )
            .is_err()
        );

        let refs: Vec<_> = (0..16).map(|i| rev(&format!("s{i}"))).collect();
        assert!(
            M60RetainedEvidenceRequest::new(
                ProcedureId::parse("p1").unwrap(),
                OffsetDateTime::from_unix_timestamp(0).unwrap(),
                refs,
            )
            .is_ok()
        );
    }

    #[test]
    fn verification_identity_rejects_zero_version() {
        assert!(
            M60VerificationIdentity::new(
                "verifier:1".to_owned(),
                OffsetDateTime::from_unix_timestamp(0).unwrap(),
                0,
            )
            .is_err()
        );
        assert!(
            M60VerificationIdentity::new(
                "verifier:1".to_owned(),
                OffsetDateTime::from_unix_timestamp(0).unwrap(),
                1,
            )
            .is_ok()
        );
    }

    #[test]
    fn evidence_set_digest_is_deterministic() {
        let refs = vec![rev("s1"), rev("s2")];
        let d1 = compute_evidence_set_digest(&refs);
        let d2 = compute_evidence_set_digest(&refs);
        assert_eq!(d1.as_str(), d2.as_str());
        assert!(d1.as_str().starts_with("sha256:"));
        assert_eq!(d1.as_str().len(), "sha256:".len() + 64);
    }
}
