use crate::repository::{
    OpportunityProfileRepository, OpportunityRepositoryError, ProfileLookup, TenantProfileRecord,
};
use crate::value::{
    AcademicProfileInput, AuthenticatedPrincipal, ConsentGrant, ConsentId, DeletionReceipt,
    OpportunityValueError, ProfileSnapshotId, hash_parts, mint_consent_id,
    mint_profile_snapshot_id,
};
use crate::{CourseQualification, ReviewedOpportunityCatalog};
use serde::Serialize;
use std::error::Error;
use std::fmt;
use time::OffsetDateTime;
use ustc_campus_agent_core::source_revision::{
    SourceRevision, SourceRevisionHealth, SourceRevisionId,
};
use ustc_campus_agent_course_planning::{PlanResult, PlanningConfig, PlanningError, plan_fixture};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OpportunitySourcePortError {
    Unavailable,
    Corrupted,
}

pub trait M60OpportunityPort: Send + Sync {
    fn revision_health(
        &self,
        revision: &SourceRevision,
    ) -> Result<SourceRevisionHealth, OpportunitySourcePortError>;
}

#[derive(Debug)]
#[non_exhaustive]
pub enum OpportunityProfileError {
    Value(OpportunityValueError),
    Repository(OpportunityRepositoryError),
    AccessDenied,
    MissingProfile,
    ProfileDeleted,
    DeleteBeforeConsent,
}

impl fmt::Display for OpportunityProfileError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "Opportunity profile operation rejected: {self:?}"
        )
    }
}

impl Error for OpportunityProfileError {}

impl From<OpportunityValueError> for OpportunityProfileError {
    fn from(error: OpportunityValueError) -> Self {
        Self::Value(error)
    }
}

impl From<OpportunityRepositoryError> for OpportunityProfileError {
    fn from(error: OpportunityRepositoryError) -> Self {
        Self::Repository(error)
    }
}

pub struct OpportunityProfileService<'a, R: OpportunityProfileRepository> {
    repository: &'a mut R,
}

impl<'a, R: OpportunityProfileRepository> OpportunityProfileService<'a, R> {
    pub fn new(repository: &'a mut R) -> Self {
        Self { repository }
    }

    pub fn create_profile(
        &mut self,
        principal: AuthenticatedPrincipal,
        consent: ConsentGrant,
        profile: AcademicProfileInput,
    ) -> Result<TenantProfileRecord, OpportunityProfileError> {
        let consent_id = mint_consent_id(&principal, &consent)?;
        let profile_snapshot_id = mint_profile_snapshot_id(&principal, &consent_id, &profile)?;
        let record = TenantProfileRecord::new(
            principal,
            profile_snapshot_id,
            consent_id,
            consent.purpose(),
            consent.fields().clone(),
            consent.consented_at(),
            profile,
        );
        self.repository.create(record.clone())?;
        Ok(record)
    }

    pub fn view_profile(
        &self,
        principal: &AuthenticatedPrincipal,
        profile_snapshot_id: &ProfileSnapshotId,
    ) -> Result<TenantProfileRecord, OpportunityProfileError> {
        match self.repository.lookup(principal, profile_snapshot_id)? {
            ProfileLookup::Active(record) => Ok(record),
            ProfileLookup::Deleted(_) => Err(OpportunityProfileError::ProfileDeleted),
            ProfileLookup::AccessDenied => Err(OpportunityProfileError::AccessDenied),
            ProfileLookup::Missing => Err(OpportunityProfileError::MissingProfile),
        }
    }

    pub fn revoke_consent_and_delete_profile(
        &mut self,
        principal: &AuthenticatedPrincipal,
        profile_snapshot_id: &ProfileSnapshotId,
        revoked_at: OffsetDateTime,
    ) -> Result<DeletionReceipt, OpportunityProfileError> {
        let record = match self.repository.lookup(principal, profile_snapshot_id)? {
            ProfileLookup::Active(record) => record,
            ProfileLookup::Deleted(tombstone) => {
                return Ok(tombstone.deletion_receipt().clone());
            }
            ProfileLookup::AccessDenied => return Err(OpportunityProfileError::AccessDenied),
            ProfileLookup::Missing => return Err(OpportunityProfileError::MissingProfile),
        };
        if revoked_at < record.consented_at() {
            return Err(OpportunityProfileError::DeleteBeforeConsent);
        }
        let receipt = DeletionReceipt::mint(
            principal,
            profile_snapshot_id.clone(),
            record.consent_id().clone(),
            revoked_at,
        )?;
        self.repository
            .delete(principal, profile_snapshot_id, receipt)
            .map_err(Into::into)
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub enum OpportunityPlanningError {
    Repository(OpportunityRepositoryError),
    AccessDenied,
    MissingProfile,
    ProfileDeleted,
    SourceNotCurrent(SourceRevisionHealth),
    SourceUnavailable(OpportunitySourcePortError),
    InvalidPlanningBounds,
    InvalidProfileFacts,
    PlanningFailed,
    ResultSerializationFailed,
}

impl fmt::Display for OpportunityPlanningError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "Opportunity planning rejected: {self:?}")
    }
}

impl Error for OpportunityPlanningError {}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum OpportunityPlanDecision {
    Planned { result: PlanResult },
    NoFeasiblePlan,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PlanStaleness {
    Current,
    SourceChanged,
    ProfileChanged,
    SourceAndProfileChanged,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct OpportunityPlanReceipt {
    receipt_id: String,
    source_revision_id: String,
    profile_snapshot_id: ProfileSnapshotId,
    consent_id: ConsentId,
    qualifications: Vec<CourseQualification>,
    decision: OpportunityPlanDecision,
    has_uncertainty: bool,
}

impl OpportunityPlanReceipt {
    #[must_use]
    pub fn receipt_id(&self) -> &str {
        &self.receipt_id
    }

    #[must_use]
    pub fn source_revision_id(&self) -> &str {
        &self.source_revision_id
    }

    #[must_use]
    pub const fn profile_snapshot_id(&self) -> &ProfileSnapshotId {
        &self.profile_snapshot_id
    }

    #[must_use]
    pub const fn consent_id(&self) -> &ConsentId {
        &self.consent_id
    }

    #[must_use]
    pub fn qualifications(&self) -> &[CourseQualification] {
        &self.qualifications
    }

    #[must_use]
    pub const fn decision(&self) -> &OpportunityPlanDecision {
        &self.decision
    }

    #[must_use]
    pub const fn has_uncertainty(&self) -> bool {
        self.has_uncertainty
    }

    #[must_use]
    pub fn staleness(
        &self,
        current_source_revision_id: &SourceRevisionId,
        current_profile_snapshot_id: &ProfileSnapshotId,
    ) -> PlanStaleness {
        match (
            self.source_revision_id != current_source_revision_id.as_str(),
            &self.profile_snapshot_id != current_profile_snapshot_id,
        ) {
            (false, false) => PlanStaleness::Current,
            (true, false) => PlanStaleness::SourceChanged,
            (false, true) => PlanStaleness::ProfileChanged,
            (true, true) => PlanStaleness::SourceAndProfileChanged,
        }
    }
}

pub const MAX_OPPORTUNITY_PLAN_RESULTS: usize = 8;
pub const MAX_OPPORTUNITY_BEAM_WIDTH: usize = 4_096;

pub struct OpportunityPlanningService<'a, R: OpportunityProfileRepository, S: M60OpportunityPort> {
    repository: &'a R,
    source: &'a S,
    catalog: &'a ReviewedOpportunityCatalog,
}

impl<'a, R: OpportunityProfileRepository, S: M60OpportunityPort>
    OpportunityPlanningService<'a, R, S>
{
    pub fn new(repository: &'a R, source: &'a S, catalog: &'a ReviewedOpportunityCatalog) -> Self {
        Self {
            repository,
            source,
            catalog,
        }
    }

    pub fn plan(
        &self,
        principal: &AuthenticatedPrincipal,
        profile_snapshot_id: &ProfileSnapshotId,
        config: PlanningConfig,
    ) -> Result<OpportunityPlanReceipt, OpportunityPlanningError> {
        if config.max_results == 0
            || config.max_results > MAX_OPPORTUNITY_PLAN_RESULTS
            || config.beam_width == 0
            || config.beam_width > MAX_OPPORTUNITY_BEAM_WIDTH
        {
            return Err(OpportunityPlanningError::InvalidPlanningBounds);
        }
        let record = match self
            .repository
            .lookup(principal, profile_snapshot_id)
            .map_err(OpportunityPlanningError::Repository)?
        {
            ProfileLookup::Active(record) => record,
            ProfileLookup::Deleted(_) => {
                return Err(OpportunityPlanningError::ProfileDeleted);
            }
            ProfileLookup::AccessDenied => return Err(OpportunityPlanningError::AccessDenied),
            ProfileLookup::Missing => return Err(OpportunityPlanningError::MissingProfile),
        };

        match self
            .source
            .revision_health(self.catalog.source_revision())
            .map_err(OpportunityPlanningError::SourceUnavailable)?
        {
            SourceRevisionHealth::Current => {}
            health => return Err(OpportunityPlanningError::SourceNotCurrent(health)),
        }

        if !self.catalog.profile_facts_are_known(record.profile()) {
            return Err(OpportunityPlanningError::InvalidProfileFacts);
        }
        let qualifications = self.catalog.qualifications(record.profile());
        let fixture = self.catalog.planning_fixture(record.profile());
        let decision = match plan_fixture(&fixture, config) {
            Ok(result) => OpportunityPlanDecision::Planned { result },
            Err(PlanningError::NoFeasiblePlan) => OpportunityPlanDecision::NoFeasiblePlan,
            Err(PlanningError::InvalidFixture { .. }) => {
                return Err(OpportunityPlanningError::InvalidProfileFacts);
            }
            Err(PlanningError::ReadFixture { .. } | PlanningError::DecodeFixture { .. }) => {
                return Err(OpportunityPlanningError::PlanningFailed);
            }
        };
        let decision_bytes = serde_json::to_vec(&decision)
            .map_err(|_| OpportunityPlanningError::ResultSerializationFailed)?;
        let qualification_bytes = serde_json::to_vec(&qualifications)
            .map_err(|_| OpportunityPlanningError::ResultSerializationFailed)?;
        let receipt_id = format!(
            "opportunity-plan:sha256:{}",
            hash_parts(
                b"opportunity-plan-receipt/v1\0",
                &[
                    self.catalog
                        .source_revision()
                        .revision_id()
                        .as_str()
                        .as_bytes(),
                    profile_snapshot_id.as_str().as_bytes(),
                    record.consent_id().as_str().as_bytes(),
                    &qualification_bytes,
                    &decision_bytes,
                ],
            )
        );
        let has_uncertainty = matches!(
            &decision,
            OpportunityPlanDecision::Planned { result } if !result.warnings.is_empty()
        );
        Ok(OpportunityPlanReceipt {
            receipt_id,
            source_revision_id: self
                .catalog
                .source_revision()
                .revision_id()
                .as_str()
                .to_owned(),
            profile_snapshot_id: profile_snapshot_id.clone(),
            consent_id: record.consent_id().clone(),
            qualifications,
            decision,
            has_uncertainty,
        })
    }
}
