use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

use crate::{UnixMillis, WireText};

pub const MAX_OPPORTUNITY_COMPLETED_COURSES: usize = 64;
pub const MAX_OPPORTUNITY_PREFERENCES: usize = 64;
pub const MAX_OPPORTUNITY_RESULTS: u16 = 8;
pub const MAX_OPPORTUNITY_BEAM_WIDTH: u16 = 4_096;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OpportunityConsentFieldDto {
    CompletedCourses,
    CreditBounds,
    PreferenceWeights,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OpportunityPreferenceDto {
    pub course_code: WireText,
    pub weight: i32,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum OpportunityCommandDto {
    CreateProfile {
        consent_purpose: WireText,
        consent_fields: Vec<OpportunityConsentFieldDto>,
        consented_at: UnixMillis,
        completed_courses: Vec<WireText>,
        min_credits: u16,
        max_credits: u16,
        preference_weights: Vec<OpportunityPreferenceDto>,
    },
    ViewProfile {
        profile_snapshot_id: WireText,
    },
    GeneratePlan {
        profile_snapshot_id: WireText,
        max_results: u16,
        beam_width: u16,
    },
    RevokeConsentAndDeleteProfile {
        profile_snapshot_id: WireText,
        revoked_at: UnixMillis,
    },
}

impl std::fmt::Debug for OpportunityCommandDto {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CreateProfile {
                completed_courses,
                preference_weights,
                ..
            } => formatter
                .debug_struct("OpportunityCommandDto")
                .field("kind", &"create_profile")
                .field("completed_course_count", &completed_courses.len())
                .field("preference_count", &preference_weights.len())
                .field("private_profile", &"[REDACTED]")
                .finish(),
            Self::ViewProfile { .. } => formatter
                .debug_struct("OpportunityCommandDto")
                .field("kind", &"view_profile")
                .field("profile_snapshot_id", &"[REDACTED]")
                .finish(),
            Self::GeneratePlan { .. } => formatter
                .debug_struct("OpportunityCommandDto")
                .field("kind", &"generate_plan")
                .field("request", &"[REDACTED]")
                .finish(),
            Self::RevokeConsentAndDeleteProfile { .. } => formatter
                .debug_struct("OpportunityCommandDto")
                .field("kind", &"revoke_consent_and_delete_profile")
                .field("request", &"[REDACTED]")
                .finish(),
        }
    }
}

impl OpportunityCommandDto {
    #[must_use]
    pub const fn operation_id(&self) -> &'static str {
        match self {
            Self::CreateProfile { .. } => "profile.academic.create",
            Self::ViewProfile { .. } => "profile.academic.view",
            Self::GeneratePlan { .. } => "planner.generate",
            Self::RevokeConsentAndDeleteProfile { .. } => "profile.academic.revoke_delete",
        }
    }

    pub fn validate(&self) -> Result<(), OpportunityWireError> {
        match self {
            Self::CreateProfile {
                consent_purpose,
                consent_fields,
                completed_courses,
                min_credits,
                max_credits,
                preference_weights,
                ..
            } => {
                if consent_purpose.as_str() != "opportunity_planning" {
                    return Err(OpportunityWireError::ConsentPurpose);
                }
                let actual: BTreeSet<_> = consent_fields.iter().copied().collect();
                let expected = BTreeSet::from([
                    OpportunityConsentFieldDto::CompletedCourses,
                    OpportunityConsentFieldDto::CreditBounds,
                    OpportunityConsentFieldDto::PreferenceWeights,
                ]);
                if actual != expected || actual.len() != consent_fields.len() {
                    return Err(OpportunityWireError::ConsentFields);
                }
                if completed_courses.len() > MAX_OPPORTUNITY_COMPLETED_COURSES {
                    return Err(OpportunityWireError::CompletedCourseCount);
                }
                let courses: BTreeSet<_> = completed_courses
                    .iter()
                    .map(|value| value.as_str())
                    .collect();
                if courses.len() != completed_courses.len() {
                    return Err(OpportunityWireError::DuplicateCourse);
                }
                if *max_credits == 0 || min_credits > max_credits {
                    return Err(OpportunityWireError::CreditBounds);
                }
                if preference_weights.len() > MAX_OPPORTUNITY_PREFERENCES {
                    return Err(OpportunityWireError::PreferenceCount);
                }
                let preferences: BTreeSet<_> = preference_weights
                    .iter()
                    .map(|value| value.course_code.as_str())
                    .collect();
                if preferences.len() != preference_weights.len() {
                    return Err(OpportunityWireError::DuplicatePreference);
                }
                Ok(())
            }
            Self::ViewProfile {
                profile_snapshot_id,
            }
            | Self::RevokeConsentAndDeleteProfile {
                profile_snapshot_id,
                ..
            } => validate_profile_snapshot_id(profile_snapshot_id),
            Self::GeneratePlan {
                profile_snapshot_id,
                max_results,
                beam_width,
            } => {
                validate_profile_snapshot_id(profile_snapshot_id)?;
                if !(1..=MAX_OPPORTUNITY_RESULTS).contains(max_results) {
                    return Err(OpportunityWireError::ResultCount);
                }
                if !(1..=MAX_OPPORTUNITY_BEAM_WIDTH).contains(beam_width) {
                    return Err(OpportunityWireError::BeamWidth);
                }
                Ok(())
            }
        }
    }

    pub(crate) fn canonical_bytes(&self) -> Result<Vec<u8>, OpportunityWireError> {
        self.validate()?;
        let canonical = match self {
            Self::CreateProfile {
                consent_purpose,
                consent_fields,
                consented_at,
                completed_courses,
                min_credits,
                max_credits,
                preference_weights,
            } => {
                let mut fields = consent_fields.clone();
                fields.sort();
                let mut courses = completed_courses.clone();
                courses.sort_by(|left, right| left.as_str().cmp(right.as_str()));
                let mut preferences = preference_weights.clone();
                preferences.sort_by(|left, right| {
                    left.course_code.as_str().cmp(right.course_code.as_str())
                });
                Self::CreateProfile {
                    consent_purpose: consent_purpose.clone(),
                    consent_fields: fields,
                    consented_at: *consented_at,
                    completed_courses: courses,
                    min_credits: *min_credits,
                    max_credits: *max_credits,
                    preference_weights: preferences,
                }
            }
            value => value.clone(),
        };
        serde_json::to_vec(&canonical).map_err(|_| OpportunityWireError::Serialization)
    }
}

fn validate_profile_snapshot_id(value: &WireText) -> Result<(), OpportunityWireError> {
    let Some(tail) = value.as_str().strip_prefix("profile-snapshot:opportunity:") else {
        return Err(OpportunityWireError::ProfileSnapshotId);
    };
    if tail.is_empty()
        || tail.len() > 128
        || !tail
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b':' | b'.' | b'_' | b'-'))
    {
        return Err(OpportunityWireError::ProfileSnapshotId);
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpportunityWireError {
    ConsentPurpose,
    ConsentFields,
    CompletedCourseCount,
    DuplicateCourse,
    CreditBounds,
    PreferenceCount,
    DuplicatePreference,
    ProfileSnapshotId,
    ResultCount,
    BeamWidth,
    Serialization,
}

impl std::fmt::Display for OpportunityWireError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::ConsentPurpose => "Opportunity consent purpose is invalid",
            Self::ConsentFields => "Opportunity consent fields are not the exact required set",
            Self::CompletedCourseCount => "Opportunity completed-course count exceeds the bound",
            Self::DuplicateCourse => "Opportunity completed courses contain a duplicate",
            Self::CreditBounds => "Opportunity credit bounds are invalid",
            Self::PreferenceCount => "Opportunity preference count exceeds the bound",
            Self::DuplicatePreference => {
                "Opportunity preference weights contain a duplicate course"
            }
            Self::ProfileSnapshotId => "Opportunity profile snapshot identity is invalid",
            Self::ResultCount => "Opportunity result count is outside the admitted bound",
            Self::BeamWidth => "Opportunity beam width is outside the admitted bound",
            Self::Serialization => "Opportunity command canonical serialization failed",
        })
    }
}

impl std::error::Error for OpportunityWireError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OpportunitySourceHealthDto {
    Stale,
    Conflicting,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum OpportunityRejectionDto {
    AuthenticationRequired,
    AccessDenied,
    MissingProfile,
    ProfileDeleted,
    ProfileAlreadyExists,
    DeleteBeforeConsent,
    InvalidProfileFacts,
    SourceNotCurrent { health: OpportunitySourceHealthDto },
    SourceUnavailable,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OpportunityProfileViewDto {
    pub profile_snapshot_id: WireText,
    pub consent_id: WireText,
    pub consent_purpose: WireText,
    pub consent_fields: Vec<OpportunityConsentFieldDto>,
    pub consented_at: UnixMillis,
    pub completed_course_count: u16,
    pub min_credits: u16,
    pub max_credits: u16,
    pub preference_count: u16,
}

impl std::fmt::Debug for OpportunityProfileViewDto {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("OpportunityProfileViewDto")
            .field("profile_snapshot_id", &"[REDACTED]")
            .field("consent_id", &"[REDACTED]")
            .field("private_profile", &"[REDACTED]")
            .finish()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum OpportunityQualificationBlockerDto {
    Unavailable,
    UnresolvedIdentity,
    ConflictingFact,
    MissingPrerequisite { course_code: WireText },
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OpportunityQualificationDto {
    pub course_code: WireText,
    pub source_id: WireText,
    pub source_revision_id: WireText,
    pub eligible: bool,
    pub blockers: Vec<OpportunityQualificationBlockerDto>,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OpportunityRequirementCreditDto {
    pub requirement_id: WireText,
    pub credits: u16,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OpportunityFactEvidenceDto {
    pub fact: WireText,
    pub source_id: WireText,
    pub revision: WireText,
    pub authority: WireText,
    pub retrieved_at: WireText,
    pub effective_time: Option<WireText>,
    pub conflict_status: WireText,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OpportunityPlanCandidateDto {
    pub course_codes: Vec<WireText>,
    pub total_credits: u16,
    pub requirement_credits: Vec<OpportunityRequirementCreditDto>,
    pub soft_score: i64,
    pub hard_constraint_violations: Vec<WireText>,
    pub rationale: Vec<WireText>,
    pub provenance: Vec<OpportunityFactEvidenceDto>,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum OpportunityPlanDecisionDto {
    Planned {
        candidates: Vec<OpportunityPlanCandidateDto>,
        hard_constraint_violations: u32,
        warnings: Vec<WireText>,
    },
    NoFeasiblePlan,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OpportunityPlanViewDto {
    pub receipt_id: WireText,
    pub source_revision_id: WireText,
    pub profile_snapshot_id: WireText,
    pub consent_id: WireText,
    pub qualifications: Vec<OpportunityQualificationDto>,
    pub decision: OpportunityPlanDecisionDto,
    pub has_uncertainty: bool,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OpportunityDeletionViewDto {
    pub deletion_receipt_id: WireText,
    pub profile_snapshot_id: WireText,
    pub consent_id: WireText,
    pub deleted_at: UnixMillis,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum M72OpportunityTerminalDto {
    ProfileCreated {
        profile: OpportunityProfileViewDto,
    },
    ProfileFound {
        profile: OpportunityProfileViewDto,
    },
    PlanGenerated {
        plan: Box<OpportunityPlanViewDto>,
    },
    ProfileDeleted {
        deletion: OpportunityDeletionViewDto,
    },
}

impl std::fmt::Debug for M72OpportunityTerminalDto {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("M72OpportunityTerminalDto")
            .field(
                "kind",
                &match self {
                    Self::ProfileCreated { .. } => "profile_created",
                    Self::ProfileFound { .. } => "profile_found",
                    Self::PlanGenerated { .. } => "plan_generated",
                    Self::ProfileDeleted { .. } => "profile_deleted",
                },
            )
            .field("owner_private_terminal", &"[REDACTED]")
            .finish()
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    fn text(value: &str) -> WireText {
        WireText::parse(value).unwrap()
    }

    fn create(courses: &[&str], preferences: &[(&str, i32)]) -> OpportunityCommandDto {
        OpportunityCommandDto::CreateProfile {
            consent_purpose: text("opportunity_planning"),
            consent_fields: vec![
                OpportunityConsentFieldDto::CompletedCourses,
                OpportunityConsentFieldDto::CreditBounds,
                OpportunityConsentFieldDto::PreferenceWeights,
            ],
            consented_at: UnixMillis::new(1_700_000_000_000),
            completed_courses: courses.iter().map(|value| text(value)).collect(),
            min_credits: 9,
            max_credits: 12,
            preference_weights: preferences
                .iter()
                .map(|(course_code, weight)| OpportunityPreferenceDto {
                    course_code: text(course_code),
                    weight: *weight,
                })
                .collect(),
        }
    }

    #[test]
    fn canonical_profile_command_is_order_independent() {
        let left = create(
            &["MATH1002", "MATH1001"],
            &[("MATH2003", 8), ("MATH2001", 9)],
        );
        let right = create(
            &["MATH1001", "MATH1002"],
            &[("MATH2001", 9), ("MATH2003", 8)],
        );
        assert_eq!(
            left.canonical_bytes().unwrap(),
            right.canonical_bytes().unwrap()
        );
    }

    #[test]
    fn duplicate_private_inputs_and_unbounded_plans_fail_closed() {
        assert_eq!(
            create(&["MATH1001", "MATH1001"], &[]).validate(),
            Err(OpportunityWireError::DuplicateCourse)
        );
        assert_eq!(
            create(&[], &[("MATH2001", 1), ("MATH2001", 2)]).validate(),
            Err(OpportunityWireError::DuplicatePreference)
        );
        let plan = OpportunityCommandDto::GeneratePlan {
            profile_snapshot_id: text("profile-snapshot:opportunity:fixture"),
            max_results: MAX_OPPORTUNITY_RESULTS + 1,
            beam_width: 1,
        };
        assert_eq!(plan.validate(), Err(OpportunityWireError::ResultCount));
    }

    #[test]
    fn command_debug_redacts_private_profile() {
        let debug = format!("{:?}", create(&["SECRET1001"], &[("SECRET2001", 9)]));
        assert!(!debug.contains("SECRET1001"));
        assert!(!debug.contains("SECRET2001"));
        assert!(debug.contains("REDACTED"));
    }
}
