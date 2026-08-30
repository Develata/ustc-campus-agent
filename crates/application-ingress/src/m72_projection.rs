use ustc_campus_agent_client_protocol::{
    M72OpportunityTerminalDto, OpportunityConsentFieldDto, OpportunityDeletionViewDto,
    OpportunityFactEvidenceDto, OpportunityPlanCandidateDto, OpportunityPlanDecisionDto,
    OpportunityPlanViewDto, OpportunityProfileViewDto, OpportunityQualificationBlockerDto,
    OpportunityQualificationDto, OpportunityRequirementCreditDto, UnixMillis, WireText,
};
use ustc_campus_agent_course_planning::{
    ConflictStatus, CoursePlanningAuthority, FactProvenance, PlanCandidate,
};
use ustc_campus_agent_opportunity_graph::{
    ConsentField, ConsentPurpose, DeletionReceipt, OpportunityPlanDecision, OpportunityPlanReceipt,
    QualificationBlocker, TenantProfileRecord,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum M72ProjectionError {
    WireText,
    Count,
    Timestamp,
}

pub fn project_profile(
    record: &TenantProfileRecord,
) -> Result<OpportunityProfileViewDto, M72ProjectionError> {
    let snapshot = record.profile().snapshot();
    let completed_course_count =
        u16::try_from(snapshot.completed_courses.len()).map_err(|_| M72ProjectionError::Count)?;
    let preference_count =
        u16::try_from(snapshot.preference_weights.len()).map_err(|_| M72ProjectionError::Count)?;
    let mut consent_fields = record
        .consent_fields()
        .iter()
        .map(|field| match field {
            ConsentField::CompletedCourses => OpportunityConsentFieldDto::CompletedCourses,
            ConsentField::CreditBounds => OpportunityConsentFieldDto::CreditBounds,
            ConsentField::PreferenceWeights => OpportunityConsentFieldDto::PreferenceWeights,
        })
        .collect::<Vec<_>>();
    consent_fields.sort();
    let consent_purpose = match record.consent_purpose() {
        ConsentPurpose::OpportunityPlanning => "opportunity_planning",
    };
    Ok(OpportunityProfileViewDto {
        profile_snapshot_id: wire(record.profile_snapshot_id().as_str())?,
        consent_id: wire(record.consent_id().as_str())?,
        consent_purpose: wire(consent_purpose)?,
        consent_fields,
        consented_at: timestamp(record.consented_at())?,
        completed_course_count,
        min_credits: snapshot.min_credits,
        max_credits: snapshot.max_credits,
        preference_count,
    })
}

pub fn project_deletion(
    receipt: &DeletionReceipt,
) -> Result<OpportunityDeletionViewDto, M72ProjectionError> {
    let deleted_at_millis = i64::try_from(receipt.deleted_at_unix_nanos() / 1_000_000)
        .map_err(|_| M72ProjectionError::Timestamp)?;
    Ok(OpportunityDeletionViewDto {
        deletion_receipt_id: wire(receipt.receipt_id().as_str())?,
        profile_snapshot_id: wire(receipt.profile_snapshot_id().as_str())?,
        consent_id: wire(receipt.consent_id().as_str())?,
        deleted_at: UnixMillis::new(deleted_at_millis),
    })
}

pub fn project_plan(
    receipt: &OpportunityPlanReceipt,
) -> Result<OpportunityPlanViewDto, M72ProjectionError> {
    let qualifications = receipt
        .qualifications()
        .iter()
        .map(|qualification| {
            let blockers = qualification
                .blockers()
                .iter()
                .map(project_blocker)
                .collect::<Result<Vec<_>, _>>()?;
            Ok(OpportunityQualificationDto {
                course_code: wire(qualification.course_code())?,
                source_id: wire(qualification.source_id())?,
                source_revision_id: wire(qualification.source_revision_id())?,
                eligible: qualification.eligible(),
                blockers,
            })
        })
        .collect::<Result<Vec<_>, M72ProjectionError>>()?;
    let decision = match receipt.decision() {
        OpportunityPlanDecision::NoFeasiblePlan => OpportunityPlanDecisionDto::NoFeasiblePlan,
        OpportunityPlanDecision::Planned { result } => {
            let candidates = result
                .candidates
                .iter()
                .map(project_candidate)
                .collect::<Result<Vec<_>, _>>()?;
            let hard_constraint_violations = u32::try_from(result.hard_constraint_violations)
                .map_err(|_| M72ProjectionError::Count)?;
            let warnings = result
                .warnings
                .iter()
                .map(|value| wire(value))
                .collect::<Result<Vec<_>, _>>()?;
            OpportunityPlanDecisionDto::Planned {
                candidates,
                hard_constraint_violations,
                warnings,
            }
        }
    };
    Ok(OpportunityPlanViewDto {
        receipt_id: wire(receipt.receipt_id())?,
        source_revision_id: wire(receipt.source_revision_id())?,
        profile_snapshot_id: wire(receipt.profile_snapshot_id().as_str())?,
        consent_id: wire(receipt.consent_id().as_str())?,
        qualifications,
        decision,
        has_uncertainty: receipt.has_uncertainty(),
    })
}

pub fn terminal_profile_created(
    record: &TenantProfileRecord,
) -> Result<M72OpportunityTerminalDto, M72ProjectionError> {
    Ok(M72OpportunityTerminalDto::ProfileCreated {
        profile: project_profile(record)?,
    })
}

pub fn terminal_profile_found(
    record: &TenantProfileRecord,
) -> Result<M72OpportunityTerminalDto, M72ProjectionError> {
    Ok(M72OpportunityTerminalDto::ProfileFound {
        profile: project_profile(record)?,
    })
}

pub fn terminal_plan_generated(
    receipt: &OpportunityPlanReceipt,
) -> Result<M72OpportunityTerminalDto, M72ProjectionError> {
    Ok(M72OpportunityTerminalDto::PlanGenerated {
        plan: Box::new(project_plan(receipt)?),
    })
}

pub fn terminal_profile_deleted(
    receipt: &DeletionReceipt,
) -> Result<M72OpportunityTerminalDto, M72ProjectionError> {
    Ok(M72OpportunityTerminalDto::ProfileDeleted {
        deletion: project_deletion(receipt)?,
    })
}

fn project_blocker(
    blocker: &QualificationBlocker,
) -> Result<OpportunityQualificationBlockerDto, M72ProjectionError> {
    Ok(match blocker {
        QualificationBlocker::Unavailable => OpportunityQualificationBlockerDto::Unavailable,
        QualificationBlocker::UnresolvedIdentity => {
            OpportunityQualificationBlockerDto::UnresolvedIdentity
        }
        QualificationBlocker::ConflictingFact => {
            OpportunityQualificationBlockerDto::ConflictingFact
        }
        QualificationBlocker::MissingPrerequisite { course_code } => {
            OpportunityQualificationBlockerDto::MissingPrerequisite {
                course_code: wire(course_code)?,
            }
        }
    })
}

fn project_candidate(
    candidate: &PlanCandidate,
) -> Result<OpportunityPlanCandidateDto, M72ProjectionError> {
    let course_codes = candidate
        .course_codes
        .iter()
        .map(|value| wire(value))
        .collect::<Result<Vec<_>, _>>()?;
    let requirement_credits = candidate
        .requirement_credits
        .iter()
        .map(|(requirement_id, credits)| {
            Ok(OpportunityRequirementCreditDto {
                requirement_id: wire(requirement_id)?,
                credits: *credits,
            })
        })
        .collect::<Result<Vec<_>, M72ProjectionError>>()?;
    let hard_constraint_violations = candidate
        .hard_constraint_violations
        .iter()
        .map(|value| wire(value))
        .collect::<Result<Vec<_>, _>>()?;
    let rationale = candidate
        .rationale
        .iter()
        .map(|value| wire(value))
        .collect::<Result<Vec<_>, _>>()?;
    let provenance = candidate
        .provenance
        .iter()
        .map(project_evidence)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(OpportunityPlanCandidateDto {
        course_codes,
        total_credits: candidate.total_credits,
        requirement_credits,
        soft_score: candidate.soft_score,
        hard_constraint_violations,
        rationale,
        provenance,
    })
}

fn project_evidence(
    evidence: &FactProvenance,
) -> Result<OpportunityFactEvidenceDto, M72ProjectionError> {
    Ok(OpportunityFactEvidenceDto {
        fact: wire(&evidence.fact)?,
        source_id: wire(&evidence.source_id)?,
        revision: wire(&evidence.revision)?,
        authority: wire(match evidence.authority {
            CoursePlanningAuthority::ModelInference => "model_inference",
            CoursePlanningAuthority::CommunitySignal => "community_signal",
            CoursePlanningAuthority::ICourseMirror => "icourse_mirror",
            CoursePlanningAuthority::ReviewedOfficialSource => "reviewed_official_source",
            CoursePlanningAuthority::OfficialCatalogSnapshot => "official_catalog_snapshot",
        })?,
        retrieved_at: wire(&evidence.retrieved_at)?,
        effective_time: evidence.effective_time.as_deref().map(wire).transpose()?,
        conflict_status: wire(match evidence.conflict_status {
            ConflictStatus::NoKnownConflict => "no_known_conflict",
            ConflictStatus::ResolvedByAuthority => "resolved_by_authority",
            ConflictStatus::EquivalentSources => "equivalent_sources",
        })?,
    })
}

fn timestamp(value: time::OffsetDateTime) -> Result<UnixMillis, M72ProjectionError> {
    let millis = value.unix_timestamp_nanos() / 1_000_000;
    i64::try_from(millis)
        .map(UnixMillis::new)
        .map_err(|_| M72ProjectionError::Timestamp)
}

fn wire(value: &str) -> Result<WireText, M72ProjectionError> {
    WireText::parse(value).map_err(|_| M72ProjectionError::WireText)
}
