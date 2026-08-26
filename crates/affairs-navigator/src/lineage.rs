//! Sealed M71 evidence-lineage receipt (M71-v8n §4). The M71 application
//! service owns this carrier and is its sole constructor. The closed enum has
//! exactly three variants — `Verified`, `Unverified`, `NotRequired` — and the
//! outcome/lineage pairing table (§4.2) is exhaustive: no other pairing is
//! constructible.
//!
//! Raw M60 revision IDs and raw/normalized digests never enter this carrier.
//! The `m60_evidence_set_digest` is the digest of the ordered retained-
//! reference set, not any individual raw source digest.

use crate::evidence::Sha256;
use crate::m60_port::{
    M60EvidenceUnverifiedReason, M60VerificationIdentity, M60VerifiedEvidenceSet,
};
use crate::value::MaterializationReceiptId;

/// Why the M71 lookup ladder terminated without source-evidence verification.
/// Each reason pairs with exactly one top-level outcome (§4.2):
///
/// | Reason | Outcome |
/// |---|---|
/// | `NoVisibleArtifact` | `NotFound` |
/// | `ArchivedWithoutCurrentArtifact` | `Archived` |
/// | `KnownAfterCutoff` | `NotYetKnown` |
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum EvidenceNotRequiredReason {
    NoVisibleArtifact,
    ArchivedWithoutCurrentArtifact,
    KnownAfterCutoff,
}

/// The sealed evidence-lineage receipt. Constructors are `pub(crate)` so only
/// the M71 application service can build one; external code reads through the
/// public accessors. M10 performs an exhaustive checked conversion to its own
/// DTO and cannot turn `Unverified` or `NotRequired` into `Verified`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum M71EvidenceLineage {
    Verified {
        materialization_receipt_id: MaterializationReceiptId,
        m60_evidence_set_digest: Sha256,
        m60_revision_count: u8,
        verification_identity: M60VerificationIdentity,
    },
    Unverified {
        materialization_receipt_id: MaterializationReceiptId,
        reason: M60EvidenceUnverifiedReason,
    },
    NotRequired {
        materialization_receipt_id: MaterializationReceiptId,
        reason: EvidenceNotRequiredReason,
    },
}

impl M71EvidenceLineage {
    /// Builds a `Verified` lineage from a successful M60 verification. M71
    /// checks digest grammar, count equality, nonzero contract version, and
    /// verification time before calling this.
    pub(crate) fn verified(
        materialization_receipt_id: MaterializationReceiptId,
        verified_set: &M60VerifiedEvidenceSet,
    ) -> Self {
        Self::Verified {
            materialization_receipt_id,
            m60_evidence_set_digest: verified_set.evidence_set_digest().clone(),
            m60_revision_count: verified_set.revision_count(),
            verification_identity: verified_set.verification_identity().clone(),
        }
    }

    /// Builds an `Unverified` lineage from a failed M60 verification. The M71
    /// service carries the M60 unverified reason without fabricating a digest
    /// or identity.
    pub(crate) fn unverified(
        materialization_receipt_id: MaterializationReceiptId,
        reason: M60EvidenceUnverifiedReason,
    ) -> Self {
        Self::Unverified {
            materialization_receipt_id,
            reason,
        }
    }

    /// Builds a `NotRequired` lineage. Used by `NotFound`, `Archived`, and
    /// `NotYetKnown` — outcomes that terminate before source-evidence
    /// verification and do not issue an empty M60 request.
    pub(crate) fn not_required(
        materialization_receipt_id: MaterializationReceiptId,
        reason: EvidenceNotRequiredReason,
    ) -> Self {
        Self::NotRequired {
            materialization_receipt_id,
            reason,
        }
    }

    /// Returns the materialization receipt ID sealing this receipt.
    #[must_use]
    pub fn materialization_receipt_id(&self) -> &MaterializationReceiptId {
        match self {
            Self::Verified {
                materialization_receipt_id,
                ..
            }
            | Self::Unverified {
                materialization_receipt_id,
                ..
            }
            | Self::NotRequired {
                materialization_receipt_id,
                ..
            } => materialization_receipt_id,
        }
    }

    /// Returns `true` iff this lineage is `Verified`.
    #[must_use]
    pub const fn is_verified(&self) -> bool {
        matches!(self, Self::Verified { .. })
    }

    /// Returns `true` iff this lineage is `Unverified`.
    #[must_use]
    pub const fn is_unverified(&self) -> bool {
        matches!(self, Self::Unverified { .. })
    }

    /// Returns `true` iff this lineage is `NotRequired`.
    #[must_use]
    pub const fn is_not_required(&self) -> bool {
        matches!(self, Self::NotRequired { .. })
    }

    /// Returns the M60 evidence set digest. `None` unless `Verified`.
    #[must_use]
    pub fn m60_evidence_set_digest(&self) -> Option<&Sha256> {
        match self {
            Self::Verified {
                m60_evidence_set_digest,
                ..
            } => Some(m60_evidence_set_digest),
            _ => None,
        }
    }

    /// Returns the M60 revision count. `None` unless `Verified`.
    #[must_use]
    pub const fn m60_revision_count(&self) -> Option<u8> {
        match self {
            Self::Verified {
                m60_revision_count, ..
            } => Some(*m60_revision_count),
            _ => None,
        }
    }

    /// Returns the M60 verification identity. `None` unless `Verified`.
    #[must_use]
    pub fn verification_identity(&self) -> Option<&M60VerificationIdentity> {
        match self {
            Self::Verified {
                verification_identity,
                ..
            } => Some(verification_identity),
            _ => None,
        }
    }

    /// Returns the unverified reason. `None` unless `Unverified`.
    #[must_use]
    pub const fn unverified_reason(&self) -> Option<M60EvidenceUnverifiedReason> {
        match self {
            Self::Unverified { reason, .. } => Some(*reason),
            _ => None,
        }
    }

    /// Returns the not-required reason. `None` unless `NotRequired`.
    #[must_use]
    pub const fn not_required_reason(&self) -> Option<EvidenceNotRequiredReason> {
        match self {
            Self::NotRequired { reason, .. } => Some(*reason),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn not_required_accessors() {
        let receipt_id = MaterializationReceiptId::parse("receipt:1").unwrap();
        let lineage = M71EvidenceLineage::not_required(
            receipt_id.clone(),
            EvidenceNotRequiredReason::NoVisibleArtifact,
        );
        assert!(lineage.is_not_required());
        assert!(!lineage.is_verified());
        assert!(!lineage.is_unverified());
        assert_eq!(
            lineage.not_required_reason(),
            Some(EvidenceNotRequiredReason::NoVisibleArtifact)
        );
        assert!(lineage.m60_revision_count().is_none());
        assert_eq!(lineage.materialization_receipt_id().as_str(), "receipt:1");
    }
}
