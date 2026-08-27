//! M71 evidence algebra: the equal-contract `M60RevisionRef` fake carrier,
//! product-local authority, evidence assessments, the bitemporal evidence
//! context, and the deterministic `valid_interval` derivation.
//!
//! D8 split (M71-v8n §8.2): `M60RevisionRef` carries M60-owned revision
//! identity only (source_id grammar, opaque revision label, observed_at,
//! digests). It does NOT carry product-local authority. `AffairsAuthority` and
//! `AffairsAuthorityAssessment` are product-local and live on the assessment,
//! not on the revision carrier. When M60 v1 is implemented, `M60RevisionRef`
//! swaps to the accepted M60 `SourceRevision` shape; the M71 public algebra is
//! designed so this is a carrier swap, not a semantic change.

use time::OffsetDateTime;

use crate::value::{
    ActorRef, AffairsValueError, AffairsValueErrorKind, ArtifactId, SourceId, value_error,
};

/// Which kind of unresolved material conflict peer sources are in. Carried by
/// `ProcedureEvidenceContext` when `conflict_state == UnresolvedConflict` or
/// `authority_comparison == Incomparable`; the service projects it into the
/// public `ConflictDetail` with a closed `&'static str` description.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ConflictKind {
    DirectContradiction,
    OverlapIncompatible,
    AuthorityConflict,
}

/// Maximum conflict peer evidence references carried on a conflict detail.
pub const MAX_CONFLICT_EVIDENCE_REFS: usize = 16;

/// Returns the closed `&'static str` description for a conflict kind. Never
/// echoes rejected input.
#[must_use]
pub const fn conflict_description(kind: ConflictKind) -> &'static str {
    match kind {
        ConflictKind::DirectContradiction => "direct contradiction between peer sources",
        ConflictKind::OverlapIncompatible => "overlapping but incompatible peer facts",
        ConflictKind::AuthorityConflict => "peer authorities could not be reconciled",
    }
}

// ---------------------------------------------------------------------------
// `Sha256` — lowercase `sha256:` + 64 hex. M60-owned digest grammar, carried
// by the equal-contract fake. Never appears in the public projection.
// ---------------------------------------------------------------------------

const SHA256_PREFIX: &str = "sha256:";
const SHA256_HEX_LEN: usize = 64;

/// One SHA-256 digest value: exactly `sha256:` followed by 64 lowercase hex
/// digits. Carried by `M60RevisionRef` (raw/normalized digest) and by the
/// retained-evidence verified set digest. Never appears in the public
/// projection.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Sha256 {
    value: String,
}

impl Sha256 {
    /// Builds one checked digest.
    ///
    /// # Errors
    ///
    /// Returns [`AffairsValueError`] when `value` is not exactly `sha256:`
    /// followed by 64 lowercase hexadecimal digits.
    pub fn new(value: impl Into<String>) -> Result<Self, AffairsValueError> {
        let value = value.into();
        let Some(hex) = value.strip_prefix(SHA256_PREFIX) else {
            return Err(value_error("Sha256", AffairsValueErrorKind::InvalidStart));
        };
        if hex.len() != SHA256_HEX_LEN {
            return Err(value_error(
                "Sha256",
                AffairsValueErrorKind::TooLong {
                    max_bytes: SHA256_PREFIX.len() + SHA256_HEX_LEN,
                },
            ));
        }
        for (index, byte) in hex.bytes().enumerate() {
            if !byte.is_ascii_digit() && !(b'a'..=b'f').contains(&byte) {
                return Err(value_error(
                    "Sha256",
                    AffairsValueErrorKind::InvalidCharacter {
                        byte_index: SHA256_PREFIX.len() + index,
                    },
                ));
            }
        }
        Ok(Self { value })
    }

    /// Builds the canonical lowercase digest value from raw SHA-256 bytes.
    pub(crate) fn from_bytes(bytes: [u8; 32]) -> Self {
        const HEX: &[u8; 16] = b"0123456789abcdef";
        let mut value = String::with_capacity(SHA256_PREFIX.len() + SHA256_HEX_LEN);
        value.push_str(SHA256_PREFIX);
        for byte in bytes {
            value.push(char::from(HEX[usize::from(byte >> 4)]));
            value.push(char::from(HEX[usize::from(byte & 0x0f)]));
        }
        Self { value }
    }

    /// Returns the exact canonical digest text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.value
    }
}

impl std::fmt::Display for Sha256 {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.value)
    }
}

// ---------------------------------------------------------------------------
// `M60RevisionRef` — equal-contract M71-owned fake (D8). M60-owned: source_id
// grammar, revision_id opacity, observed_at, digests. Does NOT carry
// product-local authority.
// ---------------------------------------------------------------------------

/// One equal-contract M60 revision reference. This is the v0 fake carrier that
/// lets the M71 algebra be exercised before M60 v1 exists; it does NOT mint
/// canonical M60 revision IDs and swaps to M60 v1 `SourceRevision` when
/// implemented.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct M60RevisionRef {
    source_id: SourceId,
    revision_id: String,
    observed_at: OffsetDateTime,
    published_at: Option<OffsetDateTime>,
    effective_from: Option<OffsetDateTime>,
    effective_to: Option<OffsetDateTime>,
    raw_digest: Sha256,
    normalized_digest: Sha256,
}

impl M60RevisionRef {
    /// Builds one checked revision reference.
    ///
    /// # Errors
    ///
    /// Returns [`AffairsValueError`] when `revision_id` is empty or exceeds 128
    /// bytes.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        source_id: SourceId,
        revision_id: String,
        observed_at: OffsetDateTime,
        published_at: Option<OffsetDateTime>,
        effective_from: Option<OffsetDateTime>,
        effective_to: Option<OffsetDateTime>,
        raw_digest: Sha256,
        normalized_digest: Sha256,
    ) -> Result<Self, AffairsValueError> {
        if revision_id.is_empty() {
            return Err(value_error("M60RevisionRef", AffairsValueErrorKind::Empty));
        }
        if revision_id.len() > 128 {
            return Err(value_error(
                "M60RevisionRef",
                AffairsValueErrorKind::TooLong { max_bytes: 128 },
            ));
        }
        Ok(Self {
            source_id,
            revision_id,
            observed_at,
            published_at,
            effective_from,
            effective_to,
            raw_digest,
            normalized_digest,
        })
    }

    #[must_use]
    pub fn source_id(&self) -> &SourceId {
        &self.source_id
    }

    /// Returns the opaque revision label. NOT a canonical M60 revision ID.
    #[must_use]
    pub fn revision_id(&self) -> &str {
        &self.revision_id
    }

    #[must_use]
    pub fn observed_at(&self) -> OffsetDateTime {
        self.observed_at
    }

    #[must_use]
    pub fn published_at(&self) -> Option<OffsetDateTime> {
        self.published_at
    }

    #[must_use]
    pub fn effective_from(&self) -> Option<OffsetDateTime> {
        self.effective_from
    }

    #[must_use]
    pub fn effective_to(&self) -> Option<OffsetDateTime> {
        self.effective_to
    }

    /// Returns the raw source digest. Internal only; never enters the public
    /// projection.
    #[must_use]
    pub fn raw_digest(&self) -> &Sha256 {
        &self.raw_digest
    }

    /// Returns the normalized source digest. Internal only.
    #[must_use]
    pub fn normalized_digest(&self) -> &Sha256 {
        &self.normalized_digest
    }
}

// ---------------------------------------------------------------------------
// Product-local authority (§6.3). Four-tier total order; `Ord` is tier-ascending
// so `OfficialBulletin` is the maximal value. This is NOT the generic M60
// `SourceAuthority`.
// ---------------------------------------------------------------------------

/// Product-local affairs authority tier. Refined inside M71's bounded type from
/// the generic M60 `SourceAuthority`; the generic authority remains
/// incomparable across product-specific variants it does not name.
///
/// `Ord` is by tier ascending: `ReviewedCommunitySummary` < … <
/// `OfficialBulletin`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AffairsAuthority {
    /// Tier 1. Reviewed community summary.
    ReviewedCommunitySummary,
    /// Tier 2. Student affairs office.
    StudentAffairsOffice,
    /// Tier 3. Department notice.
    DepartmentNotice,
    /// Tier 4. Official bulletin (maximal tier).
    OfficialBulletin,
}

impl AffairsAuthority {
    /// Returns the fixed authority tier: `OfficialBulletin=4`,
    /// `DepartmentNotice=3`, `StudentAffairsOffice=2`,
    /// `ReviewedCommunitySummary=1`.
    #[must_use]
    pub const fn tier(self) -> u8 {
        match self {
            Self::OfficialBulletin => 4,
            Self::DepartmentNotice => 3,
            Self::StudentAffairsOffice => 2,
            Self::ReviewedCommunitySummary => 1,
        }
    }
}

/// Which subject an authority is asserting. `Ord` is the frozen discriminant
/// order from M71-v8n §9.2.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AuthoritySubject {
    ProcedureTitle,
    ProcedureSteps,
    ProcedureDeadlines,
    ProcedureEffectiveInterval,
    ProcedureEntryPoints,
    ProcedureContacts,
    ProcedurePrerequisites,
    ProcedureEvidence,
}

/// How an authority fact was derived from the source revision.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AuthorityDerivation {
    Direct,
    Extracted,
    InferredRejected,
}

/// Product-local authority assessment pairing an `M60RevisionRef` with an
/// `AffairsAuthority` tier + subject and derivation/verification facts. D8
/// split: this is NOT carried on the M60 revision carrier.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct AffairsAuthorityAssessment {
    authority: AffairsAuthority,
    subject: AuthoritySubject,
    derivation: AuthorityDerivation,
    assessed_at: OffsetDateTime,
    assessed_by: ActorRef,
}

impl AffairsAuthorityAssessment {
    /// Builds one authority assessment from already-checked fields.
    #[must_use]
    pub fn new(
        authority: AffairsAuthority,
        subject: AuthoritySubject,
        derivation: AuthorityDerivation,
        assessed_at: OffsetDateTime,
        assessed_by: ActorRef,
    ) -> Self {
        Self {
            authority,
            subject,
            derivation,
            assessed_at,
            assessed_by,
        }
    }

    #[must_use]
    pub const fn authority(&self) -> AffairsAuthority {
        self.authority
    }

    #[must_use]
    pub const fn subject(&self) -> AuthoritySubject {
        self.subject
    }

    #[must_use]
    pub const fn derivation(&self) -> AuthorityDerivation {
        self.derivation
    }

    #[must_use]
    pub fn assessed_at(&self) -> OffsetDateTime {
        self.assessed_at
    }

    /// Returns the assessing actor reference. Internal only; never enters the
    /// public projection.
    #[must_use]
    pub fn assessed_by(&self) -> &ActorRef {
        &self.assessed_by
    }
}

/// The v0 evidence unit. Wraps an `M60RevisionRef` with product-local authority
/// assessment + review/verification facts. Canonical raw assessments are
/// `1..=16` per the non-empty evidence invariant.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct AffairsEvidenceAssessment {
    revision_ref: M60RevisionRef,
    authority_assessment: AffairsAuthorityAssessment,
    reviewed_at: OffsetDateTime,
    last_verified_at: OffsetDateTime,
}

impl AffairsEvidenceAssessment {
    /// Builds one evidence assessment from already-checked fields.
    #[must_use]
    pub fn new(
        revision_ref: M60RevisionRef,
        authority_assessment: AffairsAuthorityAssessment,
        reviewed_at: OffsetDateTime,
        last_verified_at: OffsetDateTime,
    ) -> Self {
        Self {
            revision_ref,
            authority_assessment,
            reviewed_at,
            last_verified_at,
        }
    }

    /// Returns the M60 revision reference. Internal only; never enters the
    /// public projection.
    #[must_use]
    pub fn revision_ref(&self) -> &M60RevisionRef {
        &self.revision_ref
    }

    #[must_use]
    pub fn authority_assessment(&self) -> &AffairsAuthorityAssessment {
        &self.authority_assessment
    }

    #[must_use]
    pub const fn authority(&self) -> AffairsAuthority {
        self.authority_assessment.authority
    }

    #[must_use]
    pub const fn subject(&self) -> AuthoritySubject {
        self.authority_assessment.subject
    }

    #[must_use]
    pub fn source_id(&self) -> &SourceId {
        self.revision_ref.source_id()
    }

    #[must_use]
    pub fn reviewed_at(&self) -> OffsetDateTime {
        self.reviewed_at
    }

    #[must_use]
    pub fn last_verified_at(&self) -> OffsetDateTime {
        self.last_verified_at
    }

    #[must_use]
    pub fn effective_from(&self) -> Option<OffsetDateTime> {
        self.revision_ref.effective_from()
    }

    #[must_use]
    pub fn effective_to(&self) -> Option<OffsetDateTime> {
        self.revision_ref.effective_to()
    }
}

// ---------------------------------------------------------------------------
// `ValidityHorizon` and the deterministic `valid_interval` derivation
// (M71-v8n §8.2). Never substitutes published_at/known_at/reviewed_at.
// ---------------------------------------------------------------------------

/// Fact-level `valid_at` projection derived from source-revision effective
/// intervals.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ValidityHorizon {
    KnownInterval {
        effective_from: OffsetDateTime,
        effective_to: OffsetDateTime,
    },
    KnownPoint {
        at: OffsetDateTime,
    },
    Unknown,
}

/// Derives the `valid_interval` from the maximal-tier assessment set.
///
/// Rules (M71-v8n §8.2, Hermes blocker 4):
/// 1. Select the maximal-tier assessment set (all at highest authority tier).
/// 2. If two or more in that set have DIFFERENT effective intervals → `Unknown`
///    (peer-source interval conflict cannot be resolved).
/// 3. If all agree: all `from:Some`+`to:Some` (and `from <= to`) →
///    `KnownInterval`; all `from:Some`+`to:None` → `KnownPoint { at: from }`;
///    all `from:None`+`to:Some` → `KnownPoint { at: to }`; neither → `Unknown`.
/// 4. Empty maximal-tier set → `Unknown`.
#[must_use]
pub fn derive_valid_interval(assessments: &[AffairsEvidenceAssessment]) -> ValidityHorizon {
    let Some(maximal_tier) = assessments.iter().map(|a| a.authority().tier()).max() else {
        return ValidityHorizon::Unknown;
    };
    let maximal: Vec<&AffairsEvidenceAssessment> = assessments
        .iter()
        .filter(|a| a.authority().tier() == maximal_tier)
        .collect();
    if maximal.is_empty() {
        return ValidityHorizon::Unknown;
    }
    // Peer-source interval conflict: any difference in (from, to) shape among
    // the maximal-tier set → Unknown.
    let first_from = maximal[0].effective_from();
    let first_to = maximal[0].effective_to();
    if maximal
        .iter()
        .any(|a| a.effective_from() != first_from || a.effective_to() != first_to)
    {
        return ValidityHorizon::Unknown;
    }
    match (first_from, first_to) {
        (Some(from), Some(to)) if from <= to => ValidityHorizon::KnownInterval {
            effective_from: from,
            effective_to: to,
        },
        (Some(_from), Some(_)) => ValidityHorizon::Unknown,
        (Some(at), None) => ValidityHorizon::KnownPoint { at },
        (None, Some(at)) => ValidityHorizon::KnownPoint { at },
        (None, None) => ValidityHorizon::Unknown,
    }
}

// ---------------------------------------------------------------------------
// Conflict / uncertainty / authority-comparison states.
// ---------------------------------------------------------------------------

/// Canonical evidence conflict state carried by `ProcedureEvidenceContext`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum EvidenceConflictState {
    NoKnownConflict,
    ResolvedByAuthority,
    EquivalentSources,
    UnresolvedConflict,
}

/// Policy-scoped comparison result between peer authorities. `Incomparable`
/// surfaces an unresolved peer-source conflict that the local tier order
/// cannot resolve.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AuthorityComparison {
    Higher,
    Lower,
    Equivalent,
    Incomparable,
}

/// Canonical uncertainty state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum UncertaintyState {
    None,
    Stale,
    CannotVerify,
    InsufficientEvidence,
}

// ---------------------------------------------------------------------------
// `ProcedureEvidenceContext` (§8.1).
// ---------------------------------------------------------------------------

/// Minimum canonical retained-evidence count (non-empty invariant).
pub const MIN_EVIDENCE_ASSESSMENTS: usize = 1;
/// Maximum canonical retained-evidence count.
pub const MAX_EVIDENCE_ASSESSMENTS: usize = 16;

/// The canonical evidence carrier carried by every `ProcedureArtifact`. It
/// distinguishes the bitemporal fact vocabulary from review/verification
/// metadata and binds `as_of` as a query/answer-level cutoff, not a fact-level
/// field.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ProcedureEvidenceContext {
    valid_interval: ValidityHorizon,
    observed_at: OffsetDateTime,
    known_at: OffsetDateTime,
    reviewed_at: OffsetDateTime,
    last_verified_at: OffsetDateTime,
    evidence_assessments: Vec<AffairsEvidenceAssessment>,
    conflict_state: EvidenceConflictState,
    authority_comparison: AuthorityComparison,
    uncertainty_state: UncertaintyState,
    conflict_kind: Option<ConflictKind>,
    conflict_evidence_refs: Vec<ArtifactId>,
}

impl ProcedureEvidenceContext {
    /// Builds one checked evidence context. `evidence_assessments` MUST be
    /// `1..=16` (the non-empty retained-evidence invariant) and
    /// `conflict_evidence_refs` MUST be `0..=16`. Conflict intent
    /// (`UnresolvedConflict` or `Incomparable`) requires `conflict_kind`;
    /// non-conflict intent forbids both conflict kind and conflict refs. Zero
    /// retained refs fails before repository insertion. `valid_interval` MUST
    /// equal `derive_valid_interval(&evidence_assessments)` — the assessments
    /// own the fact, so a disagreeing declared horizon is an illegal pairing.
    ///
    /// # Errors
    ///
    /// Returns [`AffairsValueError`] when `evidence_assessments` is empty or
    /// exceeds 16, when `conflict_evidence_refs` exceeds 16, or when the
    /// declared `valid_interval` disagrees with the horizon derivable from
    /// `evidence_assessments`.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        valid_interval: ValidityHorizon,
        observed_at: OffsetDateTime,
        known_at: OffsetDateTime,
        reviewed_at: OffsetDateTime,
        last_verified_at: OffsetDateTime,
        evidence_assessments: Vec<AffairsEvidenceAssessment>,
        conflict_state: EvidenceConflictState,
        authority_comparison: AuthorityComparison,
        uncertainty_state: UncertaintyState,
        conflict_kind: Option<ConflictKind>,
        conflict_evidence_refs: Vec<ArtifactId>,
    ) -> Result<Self, AffairsValueError> {
        let count = evidence_assessments.len();
        if count < MIN_EVIDENCE_ASSESSMENTS {
            return Err(value_error(
                "ProcedureEvidenceContext",
                AffairsValueErrorKind::Empty,
            ));
        }
        if count > MAX_EVIDENCE_ASSESSMENTS {
            return Err(value_error(
                "ProcedureEvidenceContext",
                AffairsValueErrorKind::TooLong {
                    max_bytes: MAX_EVIDENCE_ASSESSMENTS,
                },
            ));
        }
        if conflict_evidence_refs.len() > MAX_CONFLICT_EVIDENCE_REFS {
            return Err(value_error(
                "ProcedureEvidenceContext",
                AffairsValueErrorKind::TooLong {
                    max_bytes: MAX_CONFLICT_EVIDENCE_REFS,
                },
            ));
        }
        let has_conflict_intent = conflict_state == EvidenceConflictState::UnresolvedConflict
            || authority_comparison == AuthorityComparison::Incomparable;
        if has_conflict_intent != conflict_kind.is_some()
            || (!has_conflict_intent && !conflict_evidence_refs.is_empty())
        {
            return Err(value_error(
                "ProcedureEvidenceContext",
                AffairsValueErrorKind::InvalidRange,
            ));
        }
        // The assessments own the valid-interval fact: a declared horizon that
        // disagrees with `derive_valid_interval` would be a second truth owner.
        if valid_interval != derive_valid_interval(&evidence_assessments) {
            return Err(value_error(
                "ProcedureEvidenceContext",
                AffairsValueErrorKind::InvalidRange,
            ));
        }
        Ok(Self {
            valid_interval,
            observed_at,
            known_at,
            reviewed_at,
            last_verified_at,
            evidence_assessments,
            conflict_state,
            authority_comparison,
            uncertainty_state,
            conflict_kind,
            conflict_evidence_refs,
        })
    }

    #[must_use]
    pub fn valid_interval(&self) -> &ValidityHorizon {
        &self.valid_interval
    }

    #[must_use]
    pub fn observed_at(&self) -> OffsetDateTime {
        self.observed_at
    }

    #[must_use]
    pub fn known_at(&self) -> OffsetDateTime {
        self.known_at
    }

    #[must_use]
    pub fn reviewed_at(&self) -> OffsetDateTime {
        self.reviewed_at
    }

    #[must_use]
    pub fn last_verified_at(&self) -> OffsetDateTime {
        self.last_verified_at
    }

    #[must_use]
    pub fn evidence_assessments(&self) -> &[AffairsEvidenceAssessment] {
        &self.evidence_assessments
    }

    #[must_use]
    pub const fn conflict_state(&self) -> EvidenceConflictState {
        self.conflict_state
    }

    #[must_use]
    pub const fn authority_comparison(&self) -> AuthorityComparison {
        self.authority_comparison
    }

    #[must_use]
    pub const fn uncertainty_state(&self) -> UncertaintyState {
        self.uncertainty_state
    }

    #[must_use]
    pub const fn conflict_kind(&self) -> Option<ConflictKind> {
        self.conflict_kind
    }

    #[must_use]
    pub fn conflict_evidence_refs(&self) -> &[ArtifactId] {
        &self.conflict_evidence_refs
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::SourceId;

    fn assessment(
        authority: AffairsAuthority,
        source: &str,
        subject: AuthoritySubject,
    ) -> AffairsEvidenceAssessment {
        let rev = M60RevisionRef::new(
            SourceId::parse(source).unwrap(),
            format!("rev:{source}"),
            OffsetDateTime::from_unix_timestamp(1_000_000).unwrap(),
            None,
            None,
            None,
            Sha256::new("sha256:0000000000000000000000000000000000000000000000000000000000000000")
                .unwrap(),
            Sha256::new("sha256:1111111111111111111111111111111111111111111111111111111111111111")
                .unwrap(),
        )
        .unwrap();
        let auth = AffairsAuthorityAssessment::new(
            authority,
            subject,
            AuthorityDerivation::Direct,
            OffsetDateTime::from_unix_timestamp(1_000_000).unwrap(),
            crate::value::ActorRef::parse("actor:fixture").unwrap(),
        );
        AffairsEvidenceAssessment::new(
            rev,
            auth,
            OffsetDateTime::from_unix_timestamp(1_200_000).unwrap(),
            OffsetDateTime::from_unix_timestamp(1_300_000).unwrap(),
        )
    }

    #[test]
    fn sha256_grammar() {
        assert!(
            Sha256::new("sha256:abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789")
                .is_ok()
        );
        assert!(Sha256::new("sha256:ABCDEF").is_err());
        assert!(Sha256::new("sha256:short").is_err());
        assert!(Sha256::new("xxxx:abcd").is_err());
    }

    #[test]
    fn authority_tier_order() {
        assert!(AffairsAuthority::OfficialBulletin > AffairsAuthority::DepartmentNotice);
        assert!(AffairsAuthority::DepartmentNotice > AffairsAuthority::StudentAffairsOffice);
        assert!(
            AffairsAuthority::StudentAffairsOffice > AffairsAuthority::ReviewedCommunitySummary
        );
        assert_eq!(AffairsAuthority::OfficialBulletin.tier(), 4);
    }

    #[test]
    fn derive_valid_interval_known_interval() {
        let from = OffsetDateTime::from_unix_timestamp(100).unwrap();
        let to = OffsetDateTime::from_unix_timestamp(200).unwrap();
        let mut a = assessment(
            AffairsAuthority::OfficialBulletin,
            "s1",
            AuthoritySubject::ProcedureTitle,
        );
        a.revision_ref = M60RevisionRef::new(
            SourceId::parse("s1").unwrap(),
            "rev".to_owned(),
            OffsetDateTime::from_unix_timestamp(0).unwrap(),
            None,
            Some(from),
            Some(to),
            Sha256::new("sha256:0000000000000000000000000000000000000000000000000000000000000000")
                .unwrap(),
            Sha256::new("sha256:0000000000000000000000000000000000000000000000000000000000000000")
                .unwrap(),
        )
        .unwrap();
        match derive_valid_interval(&[a]) {
            ValidityHorizon::KnownInterval {
                effective_from,
                effective_to,
            } => {
                assert_eq!(effective_from, from);
                assert_eq!(effective_to, to);
            }
            other => panic!("expected KnownInterval, got {other:?}"),
        }
    }

    #[test]
    fn evidence_context_rejects_zero_and_overflow() {
        assert!(
            ProcedureEvidenceContext::new(
                ValidityHorizon::Unknown,
                OffsetDateTime::from_unix_timestamp(0).unwrap(),
                OffsetDateTime::from_unix_timestamp(0).unwrap(),
                OffsetDateTime::from_unix_timestamp(0).unwrap(),
                OffsetDateTime::from_unix_timestamp(0).unwrap(),
                Vec::new(),
                EvidenceConflictState::NoKnownConflict,
                AuthorityComparison::Equivalent,
                UncertaintyState::None,
                None,
                Vec::new(),
            )
            .is_err()
        );
        let many: Vec<_> = (0..17)
            .map(|i| {
                assessment(
                    AffairsAuthority::OfficialBulletin,
                    &format!("s{i}"),
                    AuthoritySubject::ProcedureTitle,
                )
            })
            .collect();
        assert!(
            ProcedureEvidenceContext::new(
                ValidityHorizon::Unknown,
                OffsetDateTime::from_unix_timestamp(0).unwrap(),
                OffsetDateTime::from_unix_timestamp(0).unwrap(),
                OffsetDateTime::from_unix_timestamp(0).unwrap(),
                OffsetDateTime::from_unix_timestamp(0).unwrap(),
                many,
                EvidenceConflictState::NoKnownConflict,
                AuthorityComparison::Equivalent,
                UncertaintyState::None,
                None,
                Vec::new(),
            )
            .is_err()
        );
    }

    #[test]
    fn evidence_context_rejects_conflict_intent_without_kind() {
        let values = || {
            vec![assessment(
                AffairsAuthority::OfficialBulletin,
                "s1",
                AuthoritySubject::ProcedureTitle,
            )]
        };
        for (state, comparison) in [
            (
                EvidenceConflictState::UnresolvedConflict,
                AuthorityComparison::Equivalent,
            ),
            (
                EvidenceConflictState::NoKnownConflict,
                AuthorityComparison::Incomparable,
            ),
        ] {
            assert!(
                ProcedureEvidenceContext::new(
                    ValidityHorizon::Unknown,
                    OffsetDateTime::from_unix_timestamp(0).unwrap(),
                    OffsetDateTime::from_unix_timestamp(0).unwrap(),
                    OffsetDateTime::from_unix_timestamp(0).unwrap(),
                    OffsetDateTime::from_unix_timestamp(0).unwrap(),
                    values(),
                    state,
                    comparison,
                    UncertaintyState::None,
                    None,
                    Vec::new(),
                )
                .is_err()
            );
        }
    }

    #[test]
    fn evidence_context_rejects_conflict_metadata_without_conflict_intent() {
        let values = || {
            vec![assessment(
                AffairsAuthority::OfficialBulletin,
                "s1",
                AuthoritySubject::ProcedureTitle,
            )]
        };
        let args = [
            (Some(ConflictKind::DirectContradiction), Vec::new()),
            (
                None,
                vec![crate::value::ArtifactId::parse("artifact:peer:v1").unwrap()],
            ),
        ];
        for (kind, refs) in args {
            assert!(
                ProcedureEvidenceContext::new(
                    ValidityHorizon::Unknown,
                    OffsetDateTime::from_unix_timestamp(0).unwrap(),
                    OffsetDateTime::from_unix_timestamp(0).unwrap(),
                    OffsetDateTime::from_unix_timestamp(0).unwrap(),
                    OffsetDateTime::from_unix_timestamp(0).unwrap(),
                    values(),
                    EvidenceConflictState::NoKnownConflict,
                    AuthorityComparison::Equivalent,
                    UncertaintyState::None,
                    kind,
                    refs,
                )
                .is_err()
            );
        }
    }

    #[test]
    fn evidence_context_allows_conflict_with_kind_and_zero_refs() {
        assert!(
            ProcedureEvidenceContext::new(
                ValidityHorizon::Unknown,
                OffsetDateTime::from_unix_timestamp(0).unwrap(),
                OffsetDateTime::from_unix_timestamp(0).unwrap(),
                OffsetDateTime::from_unix_timestamp(0).unwrap(),
                OffsetDateTime::from_unix_timestamp(0).unwrap(),
                vec![assessment(
                    AffairsAuthority::OfficialBulletin,
                    "s1",
                    AuthoritySubject::ProcedureTitle,
                )],
                EvidenceConflictState::UnresolvedConflict,
                AuthorityComparison::Equivalent,
                UncertaintyState::None,
                Some(ConflictKind::DirectContradiction),
                Vec::new(),
            )
            .is_ok()
        );
    }
}
