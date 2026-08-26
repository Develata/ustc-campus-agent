use affairs_navigator::{
    AffairsAuthority, AuthoritySubject, CannotVerifyReason, ConflictDetail, ConflictKind,
    ConflictState, CutoffSource, DeadlineKind, EvidenceNotRequiredReason, Freshness,
    GetProcedureOutcome, LookupPath, M60EvidenceUnverifiedReason, M71AffairsGetReceipt,
    M71EvidenceLineage, ProjectionMetadata, PublicProcedureView, UncertaintyState, ValidityHorizon,
};
use time::OffsetDateTime;
use ustc_campus_agent_client_protocol::{
    CannotVerifyReasonDto, ConflictDetailDto, ConflictStateDto, ContactDto, CutoffSourceDto,
    DeadlineDto, EntryPointDto, EvidenceAssessmentDto, EvidenceViewDto, FreshnessDto, IntervalDto,
    LookupPathDto, M71LineageDto, M71OutcomeDto, M71PairingError, M71TerminalDto, PrerequisiteDto,
    ProcedureViewDto, ProjectionMetadataDto, StepDto, UnixMillis, ValidityHorizonDto, WireText,
};

#[derive(Debug)]
pub enum M71ProjectionError {
    InvalidWireValue,
    TimestampOutOfRange,
    Pairing(M71PairingError),
}

pub fn project_receipt(
    receipt: &M71AffairsGetReceipt,
) -> Result<M71TerminalDto, M71ProjectionError> {
    let outcome = project_outcome(receipt.outcome())?;
    let lineage = project_lineage(receipt.evidence_lineage())?;
    M71TerminalDto::try_new(outcome, lineage).map_err(M71ProjectionError::Pairing)
}

fn text(value: impl Into<String>) -> Result<WireText, M71ProjectionError> {
    WireText::parse(value).map_err(|_| M71ProjectionError::InvalidWireValue)
}

fn timestamp(value: OffsetDateTime) -> Result<UnixMillis, M71ProjectionError> {
    let millis = value.unix_timestamp_nanos() / 1_000_000;
    i64::try_from(millis)
        .map(UnixMillis::new)
        .map_err(|_| M71ProjectionError::TimestampOutOfRange)
}

fn project_outcome(value: &GetProcedureOutcome) -> Result<M71OutcomeDto, M71ProjectionError> {
    Ok(match value {
        GetProcedureOutcome::Found {
            view,
            freshness,
            as_of,
        } => M71OutcomeDto::Found {
            view: Box::new(project_view(view)?),
            freshness: project_freshness(freshness)?,
            as_of: timestamp(*as_of)?,
        },
        GetProcedureOutcome::NotYetKnown {
            procedure_id,
            known_at,
            as_of,
            cutoff_metadata,
        } => M71OutcomeDto::NotYetKnown {
            procedure_id: text(procedure_id.as_str())?,
            known_at: timestamp(*known_at)?,
            as_of: timestamp(*as_of)?,
            cutoff_source: match cutoff_metadata.cutoff_source() {
                CutoffSource::CallerProvided => CutoffSourceDto::CallerProvided,
                CutoffSource::SystemNow => CutoffSourceDto::SystemNow,
            },
        },
        GetProcedureOutcome::Archived {
            procedure_id,
            archived_at,
        } => M71OutcomeDto::Archived {
            procedure_id: text(procedure_id.as_str())?,
            archived_at: timestamp(*archived_at)?,
        },
        GetProcedureOutcome::NotFound { procedure_id } => M71OutcomeDto::NotFound {
            procedure_id: text(procedure_id.as_str())?,
        },
        GetProcedureOutcome::Conflict {
            procedure_id,
            conflict,
        } => M71OutcomeDto::Conflict {
            procedure_id: text(procedure_id.as_str())?,
            conflict: project_conflict(conflict)?,
        },
        GetProcedureOutcome::CannotVerify {
            procedure_id,
            reason,
        } => M71OutcomeDto::CannotVerify {
            procedure_id: text(procedure_id.as_str())?,
            reason: match reason {
                CannotVerifyReason::SourceRevisionUnverified => {
                    CannotVerifyReasonDto::SourceRevisionUnverified
                }
                CannotVerifyReason::EffectiveIntervalMissing => {
                    CannotVerifyReasonDto::EffectiveIntervalMissing
                }
                CannotVerifyReason::LastVerifiedStaleBeyondPolicy => {
                    CannotVerifyReasonDto::LastVerifiedStaleBeyondPolicy
                }
                CannotVerifyReason::PublicEvidenceProjectionOverflow { mandatory_count } => {
                    CannotVerifyReasonDto::PublicEvidenceProjectionOverflow {
                        mandatory_count: *mandatory_count,
                    }
                }
            },
        },
    })
}

fn project_view(view: &PublicProcedureView) -> Result<ProcedureViewDto, M71ProjectionError> {
    let evidence = view.evidence();
    Ok(ProcedureViewDto {
        procedure_id: text(view.procedure_id().as_str())?,
        artifact_id: text(view.artifact_id().as_str())?,
        title: text(view.title().as_str())?,
        audience_tags: view
            .audience_tags()
            .iter()
            .map(|value| text(value.as_str()))
            .collect::<Result<_, _>>()?,
        board_id: text(view.board_id().as_str())?,
        board_policy_version: view.board_policy_version().as_u64(),
        prerequisites: view
            .prerequisites()
            .iter()
            .map(|value| {
                Ok(PrerequisiteDto {
                    condition: text(value.condition())?,
                    source_subject: value.source_subject().map(subject_text).transpose()?,
                })
            })
            .collect::<Result<_, M71ProjectionError>>()?,
        ordered_steps: view
            .ordered_steps()
            .iter()
            .map(|value| {
                Ok(StepDto {
                    ordinal: value.step_index(),
                    instruction: text(value.instruction().as_str())?,
                })
            })
            .collect::<Result<_, M71ProjectionError>>()?,
        deadlines: view
            .deadlines()
            .iter()
            .map(|value| {
                Ok(DeadlineDto {
                    label: text(value.label().as_str())?,
                    kind: text(match value.kind() {
                        DeadlineKind::Hard => "hard",
                        DeadlineKind::Soft => "soft",
                    })?,
                    at: Some(timestamp(value.at())?),
                })
            })
            .collect::<Result<_, M71ProjectionError>>()?,
        effective_interval: view
            .effective_interval()
            .map(|value| {
                Ok(IntervalDto {
                    from: Some(timestamp(value.from())?),
                    to: Some(timestamp(value.to())?),
                })
            })
            .transpose()?,
        entry_points: view
            .entry_points()
            .iter()
            .map(|value| {
                Ok(EntryPointDto {
                    label: text(value.label().as_str())?,
                    url: value.url().map(|url| text(url.as_str())).transpose()?,
                    contact_ref: text(value.contact_ref().as_str())?,
                })
            })
            .collect::<Result<_, M71ProjectionError>>()?,
        contacts: view
            .contacts()
            .iter()
            .map(|value| {
                Ok(ContactDto {
                    contact_ref: text(value.role().as_str())?,
                    name: text(value.name().as_str())?,
                    channel: text(value.channel().as_str())?,
                    source_id: text(value.value_ref().as_str())?,
                })
            })
            .collect::<Result<_, M71ProjectionError>>()?,
        evidence: EvidenceViewDto {
            valid_interval: match evidence.valid_interval() {
                ValidityHorizon::Unknown => ValidityHorizonDto::Unknown,
                ValidityHorizon::KnownPoint { at } => ValidityHorizonDto::KnownPoint {
                    at: timestamp(*at)?,
                },
                ValidityHorizon::KnownInterval {
                    effective_from,
                    effective_to,
                } => ValidityHorizonDto::KnownInterval {
                    from: Some(timestamp(*effective_from)?),
                    to: Some(timestamp(*effective_to)?),
                },
            },
            observed_at: timestamp(evidence.observed_at())?,
            known_at: timestamp(evidence.known_at())?,
            reviewed_at: timestamp(evidence.reviewed_at())?,
            last_verified_at: timestamp(evidence.last_verified_at())?,
            assessments: evidence
                .evidence_assessments()
                .iter()
                .map(|item| {
                    Ok(EvidenceAssessmentDto {
                        authority: authority_text(item.authority())?,
                        subject: subject_text(item.subject())?,
                        source_id: text(item.source_id().as_str())?,
                        reviewed_at: timestamp(item.reviewed_at())?,
                        last_verified_at: timestamp(item.last_verified_at())?,
                    })
                })
                .collect::<Result<_, M71ProjectionError>>()?,
            projection: match evidence.projection() {
                ProjectionMetadata::Complete => ProjectionMetadataDto::Complete,
                ProjectionMetadata::Truncated {
                    omitted_count,
                    selection_rule_version,
                } => ProjectionMetadataDto::Truncated {
                    omitted_count,
                    selection_rule_version,
                },
            },
        },
        lookup_path: match view.lookup_path() {
            LookupPath::ExactId => LookupPathDto::ExactId,
            LookupPath::StructuredSearch => LookupPathDto::StructuredSearch,
            LookupPath::Fallback => LookupPathDto::Fallback,
        },
        conflict_state: match view.conflict_state() {
            ConflictState::Resolved => ConflictStateDto::Resolved,
            ConflictState::Unresolved { detail } => ConflictStateDto::Unresolved {
                detail: project_conflict(detail)?,
            },
        },
        uncertainty_state: text(match view.uncertainty_state() {
            UncertaintyState::None => "none",
            UncertaintyState::Stale => "stale",
            UncertaintyState::CannotVerify => "cannot_verify",
            UncertaintyState::InsufficientEvidence => "insufficient_evidence",
        })?,
    })
}

fn project_conflict(value: &ConflictDetail) -> Result<ConflictDetailDto, M71ProjectionError> {
    Ok(ConflictDetailDto {
        conflict_kind: text(match value.conflict_kind() {
            ConflictKind::DirectContradiction => "direct_contradiction",
            ConflictKind::OverlapIncompatible => "overlap_incompatible",
            ConflictKind::AuthorityConflict => "authority_conflict",
        })?,
        description: text(value.description())?,
        evidence_refs: value
            .evidence_refs()
            .iter()
            .map(|item| text(item.as_str()))
            .collect::<Result<_, _>>()?,
    })
}

fn project_freshness(value: &Freshness) -> Result<FreshnessDto, M71ProjectionError> {
    Ok(match value {
        Freshness::Fresh => FreshnessDto::Fresh,
        Freshness::Stale {
            last_verified_at,
            max_fresh_age_seconds,
            max_presentable_age_seconds,
        } => FreshnessDto::Stale {
            last_verified_at: timestamp(*last_verified_at)?,
            max_fresh_age_seconds: *max_fresh_age_seconds,
            max_presentable_age_seconds: *max_presentable_age_seconds,
        },
    })
}

fn project_lineage(value: &M71EvidenceLineage) -> Result<M71LineageDto, M71ProjectionError> {
    let receipt = text(value.materialization_receipt_id().as_str())?;
    Ok(match value {
        M71EvidenceLineage::Verified {
            m60_evidence_set_digest,
            m60_revision_count,
            verification_identity,
            ..
        } => M71LineageDto::Verified {
            materialization_receipt_id: receipt,
            evidence_set_digest: text(m60_evidence_set_digest.as_str())?,
            revision_count: *m60_revision_count,
            verifier_id: text(verification_identity.verifier_id())?,
            verified_at: timestamp(verification_identity.verified_at())?,
            evidence_contract_version: verification_identity.evidence_contract_version(),
        },
        M71EvidenceLineage::Unverified { reason, .. } => M71LineageDto::Unverified {
            materialization_receipt_id: receipt,
            reason: text(match reason {
                M60EvidenceUnverifiedReason::MissingRevision => "missing_revision",
                M60EvidenceUnverifiedReason::DigestMismatch => "digest_mismatch",
                M60EvidenceUnverifiedReason::RevokedOrUnaccepted => "revoked_or_unaccepted",
                M60EvidenceUnverifiedReason::EffectiveIntervalMissing => {
                    "effective_interval_missing"
                }
            })?,
        },
        M71EvidenceLineage::NotRequired { reason, .. } => M71LineageDto::NotRequired {
            materialization_receipt_id: receipt,
            reason: text(match reason {
                EvidenceNotRequiredReason::NoVisibleArtifact => "no_visible_artifact",
                EvidenceNotRequiredReason::ArchivedWithoutCurrentArtifact => {
                    "archived_without_current_artifact"
                }
                EvidenceNotRequiredReason::KnownAfterCutoff => "known_after_cutoff",
            })?,
        },
    })
}

fn authority_text(value: AffairsAuthority) -> Result<WireText, M71ProjectionError> {
    text(match value {
        AffairsAuthority::ReviewedCommunitySummary => "reviewed_community_summary",
        AffairsAuthority::StudentAffairsOffice => "student_affairs_office",
        AffairsAuthority::DepartmentNotice => "department_notice",
        AffairsAuthority::OfficialBulletin => "official_bulletin",
    })
}

fn subject_text(value: AuthoritySubject) -> Result<WireText, M71ProjectionError> {
    text(match value {
        AuthoritySubject::ProcedureTitle => "procedure_title",
        AuthoritySubject::ProcedureSteps => "procedure_steps",
        AuthoritySubject::ProcedureDeadlines => "procedure_deadlines",
        AuthoritySubject::ProcedureEffectiveInterval => "procedure_effective_interval",
        AuthoritySubject::ProcedureEntryPoints => "procedure_entry_points",
        AuthoritySubject::ProcedureContacts => "procedure_contacts",
        AuthoritySubject::ProcedurePrerequisites => "procedure_prerequisites",
        AuthoritySubject::ProcedureEvidence => "procedure_evidence",
    })
}
