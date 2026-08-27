use crate::AcademicProfileInput;
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt;
use ustc_campus_agent_core::source_revision::{SourceRevision, SourceRevisionProvenance};
use ustc_campus_agent_course_planning::{
    CommunitySignal, CourseOffering, CoursePlanningAuthority, CoursePlanningFixture,
    FIXTURE_SCHEMA_VERSION, IdentityStatus, RequirementGroup, SourceDescriptor,
    UserAcademicSnapshot, validate_fixture,
};

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum CatalogConstructionError {
    SourceNotDemoReviewed,
    RevisionIdentityMismatch,
    InvalidFixture(String),
}

impl fmt::Display for CatalogConstructionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "invalid Opportunity catalog: {self:?}")
    }
}

impl Error for CatalogConstructionError {}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum QualificationBlocker {
    Unavailable,
    UnresolvedIdentity,
    ConflictingFact,
    MissingPrerequisite { course_code: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CourseQualification {
    course_code: String,
    source_id: String,
    source_revision_id: String,
    eligible: bool,
    blockers: Vec<QualificationBlocker>,
}

impl CourseQualification {
    #[must_use]
    pub fn course_code(&self) -> &str {
        &self.course_code
    }

    #[must_use]
    pub fn source_id(&self) -> &str {
        &self.source_id
    }

    #[must_use]
    pub fn source_revision_id(&self) -> &str {
        &self.source_revision_id
    }

    #[must_use]
    pub const fn eligible(&self) -> bool {
        self.eligible
    }

    #[must_use]
    pub fn blockers(&self) -> &[QualificationBlocker] {
        &self.blockers
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReviewedOpportunityCatalog {
    source_revision: SourceRevision,
    sources: Vec<SourceDescriptor>,
    requirements: Vec<RequirementGroup>,
    courses: Vec<CourseOffering>,
    community_signals: Vec<CommunitySignal>,
}

impl ReviewedOpportunityCatalog {
    pub fn from_demo_reviewed(
        source_revision: SourceRevision,
        fixture: CoursePlanningFixture,
    ) -> Result<Self, CatalogConstructionError> {
        if !matches!(
            source_revision.provenance(),
            SourceRevisionProvenance::DemoReviewed { .. }
        ) {
            return Err(CatalogConstructionError::SourceNotDemoReviewed);
        }
        if fixture.source_revision != source_revision.revision_id().as_str() {
            return Err(CatalogConstructionError::RevisionIdentityMismatch);
        }
        let validation_fixture = CoursePlanningFixture {
            schema_version: FIXTURE_SCHEMA_VERSION.to_owned(),
            source_revision: fixture.source_revision.clone(),
            sources: fixture.sources.clone(),
            profile: UserAcademicSnapshot {
                completed_courses: Vec::new(),
                min_credits: 1,
                max_credits: 1,
                preference_weights: BTreeMap::new(),
            },
            requirements: fixture.requirements.clone(),
            courses: fixture.courses.clone(),
            community_signals: fixture.community_signals.clone(),
        };
        validate_fixture(&validation_fixture)
            .map_err(|error| CatalogConstructionError::InvalidFixture(error.to_string()))?;
        Ok(Self {
            source_revision,
            sources: fixture.sources,
            requirements: fixture.requirements,
            courses: fixture.courses,
            community_signals: fixture.community_signals,
        })
    }

    #[must_use]
    pub const fn source_revision(&self) -> &SourceRevision {
        &self.source_revision
    }

    pub(crate) fn planning_fixture(&self, profile: &AcademicProfileInput) -> CoursePlanningFixture {
        CoursePlanningFixture {
            schema_version: FIXTURE_SCHEMA_VERSION.to_owned(),
            source_revision: self.source_revision.revision_id().as_str().to_owned(),
            sources: self.sources.clone(),
            profile: profile.snapshot().clone(),
            requirements: self.requirements.clone(),
            courses: self.courses.clone(),
            community_signals: self.community_signals.clone(),
        }
    }

    #[must_use]
    pub fn qualifications(&self, profile: &AcademicProfileInput) -> Vec<CourseQualification> {
        let completed: BTreeSet<_> = profile
            .snapshot()
            .completed_courses
            .iter()
            .map(String::as_str)
            .collect();
        let source_revision_id = self.source_revision.revision_id().as_str().to_owned();
        let authorities: BTreeMap<_, _> = self
            .sources
            .iter()
            .map(|source| (source.id.as_str(), source.authority))
            .collect();
        let mut selected: BTreeMap<String, (CourseOffering, CoursePlanningAuthority, bool)> =
            BTreeMap::new();
        for course in &self.courses {
            let authority = authorities
                .get(course.source_id.as_str())
                .copied()
                .unwrap_or(CoursePlanningAuthority::ModelInference);
            match selected.get_mut(&course.code) {
                None => {
                    selected.insert(course.code.clone(), (course.clone(), authority, false));
                }
                Some((current, current_authority, conflicting))
                    if authority > *current_authority =>
                {
                    *current = course.clone();
                    *current_authority = authority;
                    *conflicting = false;
                }
                Some((current, current_authority, conflicting))
                    if authority == *current_authority
                        && !same_material_course(current, course) =>
                {
                    *conflicting = true;
                }
                Some(_) => {}
            }
        }
        selected
            .into_values()
            .map(|(course, _authority, conflicting)| {
                let mut blockers = Vec::new();
                if !course.available {
                    blockers.push(QualificationBlocker::Unavailable);
                }
                if course.identity_status == IdentityStatus::UnresolvedAlias {
                    blockers.push(QualificationBlocker::UnresolvedIdentity);
                }
                if conflicting {
                    blockers.push(QualificationBlocker::ConflictingFact);
                }
                for prerequisite in &course.prerequisites {
                    if !completed.contains(prerequisite.as_str()) {
                        blockers.push(QualificationBlocker::MissingPrerequisite {
                            course_code: prerequisite.clone(),
                        });
                    }
                }
                CourseQualification {
                    course_code: course.code,
                    source_id: course.source_id,
                    source_revision_id: source_revision_id.clone(),
                    eligible: blockers.is_empty(),
                    blockers,
                }
            })
            .collect()
    }
}

fn same_material_course(left: &CourseOffering, right: &CourseOffering) -> bool {
    left.code == right.code
        && left.title == right.title
        && left.credits == right.credits
        && left.prerequisites == right.prerequisites
        && left.slots == right.slots
        && left.available == right.available
        && left.identity_status == right.identity_status
}
