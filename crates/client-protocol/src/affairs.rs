use serde::{Deserialize, Deserializer, Serialize, de};

use crate::value::{UnixMillis, WireText};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProcedureViewDto {
    pub procedure_id: WireText,
    pub artifact_id: WireText,
    pub title: WireText,
    pub audience_tags: Vec<WireText>,
    pub board_id: WireText,
    pub board_policy_version: u64,
    pub prerequisites: Vec<PrerequisiteDto>,
    pub ordered_steps: Vec<StepDto>,
    pub deadlines: Vec<DeadlineDto>,
    pub effective_interval: Option<IntervalDto>,
    pub entry_points: Vec<EntryPointDto>,
    pub contacts: Vec<ContactDto>,
    pub evidence: EvidenceViewDto,
    pub lookup_path: LookupPathDto,
    pub conflict_state: ConflictStateDto,
    pub uncertainty_state: WireText,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PrerequisiteDto {
    pub condition: WireText,
    pub source_subject: Option<WireText>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StepDto {
    pub ordinal: u32,
    pub instruction: WireText,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DeadlineDto {
    pub label: WireText,
    pub kind: WireText,
    pub at: Option<UnixMillis>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IntervalDto {
    pub from: Option<UnixMillis>,
    pub to: Option<UnixMillis>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EntryPointDto {
    pub label: WireText,
    pub url: Option<WireText>,
    pub contact_ref: WireText,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContactDto {
    pub contact_ref: WireText,
    pub name: WireText,
    pub channel: WireText,
    pub source_id: WireText,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EvidenceViewDto {
    pub valid_interval: ValidityHorizonDto,
    pub observed_at: UnixMillis,
    pub known_at: UnixMillis,
    pub reviewed_at: UnixMillis,
    pub last_verified_at: UnixMillis,
    pub assessments: Vec<EvidenceAssessmentDto>,
    pub projection: ProjectionMetadataDto,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ValidityHorizonDto {
    Unknown,
    KnownPoint {
        at: UnixMillis,
    },
    KnownInterval {
        from: Option<UnixMillis>,
        to: Option<UnixMillis>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EvidenceAssessmentDto {
    pub authority: WireText,
    pub subject: WireText,
    pub source_id: WireText,
    pub reviewed_at: UnixMillis,
    pub last_verified_at: UnixMillis,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ProjectionMetadataDto {
    Complete,
    Truncated {
        omitted_count: u8,
        selection_rule_version: u8,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LookupPathDto {
    ExactId,
    StructuredSearch,
    Fallback,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ConflictStateDto {
    Resolved,
    Unresolved { detail: ConflictDetailDto },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConflictDetailDto {
    pub conflict_kind: WireText,
    pub description: WireText,
    pub evidence_refs: Vec<WireText>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum FreshnessDto {
    Fresh,
    Stale {
        last_verified_at: UnixMillis,
        max_fresh_age_seconds: u32,
        max_presentable_age_seconds: u32,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CutoffSourceDto {
    CallerProvided,
    SystemNow,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum CannotVerifyReasonDto {
    SourceRevisionUnverified,
    EffectiveIntervalMissing,
    LastVerifiedStaleBeyondPolicy,
    PublicEvidenceProjectionOverflow { mandatory_count: u8 },
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum M71OutcomeDto {
    Found {
        view: Box<ProcedureViewDto>,
        freshness: FreshnessDto,
        as_of: UnixMillis,
    },
    NotYetKnown {
        procedure_id: WireText,
        known_at: UnixMillis,
        as_of: UnixMillis,
        cutoff_source: CutoffSourceDto,
    },
    Archived {
        procedure_id: WireText,
        archived_at: UnixMillis,
    },
    NotFound {
        procedure_id: WireText,
    },
    Conflict {
        procedure_id: WireText,
        conflict: ConflictDetailDto,
    },
    CannotVerify {
        procedure_id: WireText,
        reason: CannotVerifyReasonDto,
    },
}

impl std::fmt::Debug for M71OutcomeDto {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Found { .. } => formatter
                .debug_struct("M71OutcomeDto")
                .field("kind", &"found")
                .field("view", &"[REDACTED]")
                .field("freshness", &"[REDACTED]")
                .field("as_of", &"[REDACTED]")
                .finish(),
            Self::NotYetKnown { .. } => formatter
                .debug_struct("M71OutcomeDto")
                .field("kind", &"not_yet_known")
                .field("procedure_id", &"[REDACTED]")
                .field("known_at", &"[REDACTED]")
                .field("as_of", &"[REDACTED]")
                .field("cutoff_source", &"[REDACTED]")
                .finish(),
            Self::Archived { .. } => formatter
                .debug_struct("M71OutcomeDto")
                .field("kind", &"archived")
                .field("procedure_id", &"[REDACTED]")
                .field("archived_at", &"[REDACTED]")
                .finish(),
            Self::NotFound { .. } => formatter
                .debug_struct("M71OutcomeDto")
                .field("kind", &"not_found")
                .field("procedure_id", &"[REDACTED]")
                .finish(),
            Self::Conflict { .. } => formatter
                .debug_struct("M71OutcomeDto")
                .field("kind", &"conflict")
                .field("procedure_id", &"[REDACTED]")
                .field("conflict", &"[REDACTED]")
                .finish(),
            Self::CannotVerify { .. } => formatter
                .debug_struct("M71OutcomeDto")
                .field("kind", &"cannot_verify")
                .field("procedure_id", &"[REDACTED]")
                .field("reason", &"[REDACTED]")
                .finish(),
        }
    }
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum M71LineageDto {
    Verified {
        materialization_receipt_id: WireText,
        evidence_set_digest: WireText,
        revision_count: u8,
        verifier_id: WireText,
        verified_at: UnixMillis,
        evidence_contract_version: u16,
    },
    Unverified {
        materialization_receipt_id: WireText,
        reason: WireText,
    },
    NotRequired {
        materialization_receipt_id: WireText,
        reason: WireText,
    },
}

impl std::fmt::Debug for M71LineageDto {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Verified { .. } => formatter
                .debug_struct("M71LineageDto")
                .field("kind", &"verified")
                .field("materialization_receipt_id", &"[REDACTED]")
                .field("evidence_set_digest", &"[REDACTED]")
                .field("revision_count", &"[REDACTED]")
                .field("verifier_id", &"[REDACTED]")
                .field("verified_at", &"[REDACTED]")
                .field("evidence_contract_version", &"[REDACTED]")
                .finish(),
            Self::Unverified { .. } => formatter
                .debug_struct("M71LineageDto")
                .field("kind", &"unverified")
                .field("materialization_receipt_id", &"[REDACTED]")
                .field("reason", &"[REDACTED]")
                .finish(),
            Self::NotRequired { .. } => formatter
                .debug_struct("M71LineageDto")
                .field("kind", &"not_required")
                .field("materialization_receipt_id", &"[REDACTED]")
                .field("reason", &"[REDACTED]")
                .finish(),
        }
    }
}

#[derive(Clone, PartialEq, Eq, Serialize)]
pub struct M71TerminalDto {
    outcome: M71OutcomeDto,
    lineage: M71LineageDto,
}

impl std::fmt::Debug for M71TerminalDto {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("M71TerminalDto")
            .field("outcome", &"[REDACTED]")
            .field("lineage", &"[REDACTED]")
            .finish()
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct UncheckedM71TerminalDto {
    outcome: M71OutcomeDto,
    lineage: M71LineageDto,
}

impl M71TerminalDto {
    pub fn try_new(
        outcome: M71OutcomeDto,
        lineage: M71LineageDto,
    ) -> Result<Self, M71PairingError> {
        let valid = match (&outcome, &lineage) {
            (M71OutcomeDto::Found { .. }, M71LineageDto::Verified { .. })
            | (M71OutcomeDto::Conflict { .. }, M71LineageDto::Verified { .. }) => true,
            (
                M71OutcomeDto::CannotVerify {
                    reason:
                        CannotVerifyReasonDto::LastVerifiedStaleBeyondPolicy
                        | CannotVerifyReasonDto::PublicEvidenceProjectionOverflow { .. },
                    ..
                },
                M71LineageDto::Verified { .. },
            ) => true,
            (
                M71OutcomeDto::CannotVerify {
                    reason:
                        CannotVerifyReasonDto::SourceRevisionUnverified
                        | CannotVerifyReasonDto::EffectiveIntervalMissing,
                    ..
                },
                M71LineageDto::Unverified { reason, .. },
            ) if is_valid_unverified_reason(reason.as_str()) => true,
            (M71OutcomeDto::NotFound { .. }, M71LineageDto::NotRequired { reason, .. })
                if reason.as_str() == "no_visible_artifact" =>
            {
                true
            }
            (M71OutcomeDto::Archived { .. }, M71LineageDto::NotRequired { reason, .. })
                if reason.as_str() == "archived_without_current_artifact" =>
            {
                true
            }
            (M71OutcomeDto::NotYetKnown { .. }, M71LineageDto::NotRequired { reason, .. })
                if reason.as_str() == "known_after_cutoff" =>
            {
                true
            }
            _ => false,
        };
        if !valid {
            return Err(M71PairingError);
        }
        Ok(Self { outcome, lineage })
    }

    #[must_use]
    pub fn outcome(&self) -> &M71OutcomeDto {
        &self.outcome
    }
    #[must_use]
    pub fn lineage(&self) -> &M71LineageDto {
        &self.lineage
    }
}

impl<'de> Deserialize<'de> for M71TerminalDto {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = UncheckedM71TerminalDto::deserialize(deserializer)?;
        Self::try_new(raw.outcome, raw.lineage).map_err(de::Error::custom)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct M71PairingError;

impl std::fmt::Display for M71PairingError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("M71 outcome/lineage pairing is invalid")
    }
}

impl std::error::Error for M71PairingError {}

fn is_valid_unverified_reason(value: &str) -> bool {
    matches!(
        value,
        "missing_revision"
            | "digest_mismatch"
            | "revoked_or_unaccepted"
            | "effective_interval_missing"
    )
}
