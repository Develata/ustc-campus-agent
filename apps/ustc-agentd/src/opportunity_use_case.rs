//! Statically composed Opportunity Graph application use case.
//!
//! The four tenant-private operations are admitted by M00/M10, authorized from
//! transaction-current M20 package/installation/grant state, and then dispatched
//! directly to the owning M72 repository/source/planner services. This module
//! deliberately creates no Agent run, provider call, ToolGateway route, effect
//! intent/receipt, or PluginExecutor request.

use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use time::OffsetDateTime;
use ustc_campus_agent_application_ingress::{
    OpportunityInvocationError, OpportunityInvocationOutcome, OpportunityInvocationPort,
};
use ustc_campus_agent_client_protocol::{OpportunityCommandDto, OpportunityConsentFieldDto};
use ustc_campus_agent_core::request_context::M00AdmittedActor;
use ustc_campus_agent_course_planning::PlanningConfig;
use ustc_campus_agent_opportunity_graph::{
    AcademicProfileInput, AuthenticatedPrincipal, ConsentField, ConsentGrant, ConsentPurpose,
    OpportunityPlanningService, OpportunityProfileService, ProfileSnapshotId,
    ReviewedOpportunityCatalog,
};

use crate::opportunity_authority::{
    OpportunityMarketAuthorityStore, operation_metadata_for_command,
};
use crate::opportunity_fixture::{
    CountingOpportunitySourcePort, OpportunityApplicationFailureMode,
};
use crate::opportunity_persistence::DurableOpportunityProfileRepository;

#[derive(Clone, Default)]
pub(crate) struct OpportunityApplicationCounters {
    authorizations: Arc<AtomicU64>,
    dispatches: Arc<AtomicU64>,
    terminals: Arc<AtomicU64>,
}

impl OpportunityApplicationCounters {
    pub(crate) fn authorizations(&self) -> u64 {
        self.authorizations.load(Ordering::SeqCst)
    }

    pub(crate) fn dispatches(&self) -> u64 {
        self.dispatches.load(Ordering::SeqCst)
    }

    pub(crate) fn terminals(&self) -> u64 {
        self.terminals.load(Ordering::SeqCst)
    }
}

pub(crate) struct OpportunityApplicationUseCase<'a> {
    repository: &'a Mutex<DurableOpportunityProfileRepository>,
    source: &'a CountingOpportunitySourcePort,
    catalog: &'a ReviewedOpportunityCatalog,
    authority: &'a OpportunityMarketAuthorityStore,
    failure: OpportunityApplicationFailureMode,
    counters: OpportunityApplicationCounters,
}

impl<'a> OpportunityApplicationUseCase<'a> {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        repository: &'a Mutex<DurableOpportunityProfileRepository>,
        source: &'a CountingOpportunitySourcePort,
        catalog: &'a ReviewedOpportunityCatalog,
        authority: &'a OpportunityMarketAuthorityStore,
        failure: OpportunityApplicationFailureMode,
        counters: OpportunityApplicationCounters,
    ) -> Self {
        Self {
            repository,
            source,
            catalog,
            authority,
            failure,
            counters,
        }
    }

    fn execute(
        &self,
        actor: &M00AdmittedActor,
        command: &OpportunityCommandDto,
    ) -> Result<OpportunityInvocationOutcome, OpportunityInvocationError> {
        let principal = authenticated_principal(actor)?;
        let mut repository = self
            .repository
            .lock()
            .map_err(|_| OpportunityInvocationError::Unavailable)?;
        match command {
            OpportunityCommandDto::CreateProfile {
                consent_purpose,
                consent_fields,
                consented_at,
                completed_courses,
                min_credits,
                max_credits,
                preference_weights,
            } => {
                if consent_purpose.as_str() != "opportunity_planning" {
                    return Err(OpportunityInvocationError::Internal);
                }
                let fields = consent_fields
                    .iter()
                    .map(|field| match field {
                        OpportunityConsentFieldDto::CompletedCourses => {
                            ConsentField::CompletedCourses
                        }
                        OpportunityConsentFieldDto::CreditBounds => ConsentField::CreditBounds,
                        OpportunityConsentFieldDto::PreferenceWeights => {
                            ConsentField::PreferenceWeights
                        }
                    })
                    .collect::<Vec<_>>();
                let consent = ConsentGrant::new(
                    ConsentPurpose::OpportunityPlanning,
                    fields,
                    unix_millis(*consented_at)?,
                )
                .map_err(|error| {
                    OpportunityInvocationError::Profile(
                        ustc_campus_agent_opportunity_graph::OpportunityProfileError::Value(error),
                    )
                })?;
                let preferences = preference_weights
                    .iter()
                    .map(|preference| {
                        (
                            preference.course_code.as_str().to_owned(),
                            preference.weight,
                        )
                    })
                    .collect::<BTreeMap<_, _>>();
                let profile = AcademicProfileInput::new(
                    completed_courses
                        .iter()
                        .map(|course| course.as_str().to_owned())
                        .collect(),
                    *min_credits,
                    *max_credits,
                    preferences,
                )
                .map_err(|error| {
                    OpportunityInvocationError::Profile(
                        ustc_campus_agent_opportunity_graph::OpportunityProfileError::Value(error),
                    )
                })?;
                OpportunityProfileService::new(&mut *repository)
                    .create_profile(principal, consent, profile)
                    .map(OpportunityInvocationOutcome::ProfileCreated)
                    .map_err(OpportunityInvocationError::Profile)
            }
            OpportunityCommandDto::ViewProfile {
                profile_snapshot_id,
            } => {
                let profile_snapshot_id = ProfileSnapshotId::parse(profile_snapshot_id.as_str())
                    .map_err(|_| OpportunityInvocationError::Internal)?;
                OpportunityProfileService::new(&mut *repository)
                    .view_profile(&principal, &profile_snapshot_id)
                    .map(OpportunityInvocationOutcome::ProfileFound)
                    .map_err(OpportunityInvocationError::Profile)
            }
            OpportunityCommandDto::GeneratePlan {
                profile_snapshot_id,
                max_results,
                beam_width,
            } => {
                let profile_snapshot_id = ProfileSnapshotId::parse(profile_snapshot_id.as_str())
                    .map_err(|_| OpportunityInvocationError::Internal)?;
                OpportunityPlanningService::new(&*repository, self.source, self.catalog)
                    .plan(
                        &principal,
                        &profile_snapshot_id,
                        PlanningConfig {
                            max_results: usize::from(*max_results),
                            beam_width: usize::from(*beam_width),
                        },
                    )
                    .map(OpportunityInvocationOutcome::PlanGenerated)
                    .map_err(OpportunityInvocationError::Planning)
            }
            OpportunityCommandDto::RevokeConsentAndDeleteProfile {
                profile_snapshot_id,
                revoked_at,
            } => {
                let profile_snapshot_id = ProfileSnapshotId::parse(profile_snapshot_id.as_str())
                    .map_err(|_| OpportunityInvocationError::Internal)?;
                OpportunityProfileService::new(&mut *repository)
                    .revoke_consent_and_delete_profile(
                        &principal,
                        &profile_snapshot_id,
                        unix_millis(*revoked_at)?,
                    )
                    .map(OpportunityInvocationOutcome::ProfileDeleted)
                    .map_err(OpportunityInvocationError::Profile)
            }
        }
    }
}

impl OpportunityInvocationPort for OpportunityApplicationUseCase<'_> {
    fn invoke(
        &self,
        actor: &M00AdmittedActor,
        command: &OpportunityCommandDto,
    ) -> Result<OpportunityInvocationOutcome, OpportunityInvocationError> {
        let metadata = operation_metadata_for_command(command);
        self.authority.authorize(actor, &metadata)?;
        self.counters.authorizations.fetch_add(1, Ordering::SeqCst);

        if self.failure == OpportunityApplicationFailureMode::BeforeDispatch {
            return Err(OpportunityInvocationError::Unavailable);
        }

        self.counters.dispatches.fetch_add(1, Ordering::SeqCst);
        let outcome = self.execute(actor, command);
        match outcome {
            Ok(outcome) => {
                if self.failure == OpportunityApplicationFailureMode::ResponsePersistenceUnavailable
                {
                    return Err(OpportunityInvocationError::OutcomeUnknown);
                }
                self.counters.terminals.fetch_add(1, Ordering::SeqCst);
                Ok(outcome)
            }
            Err(error) => {
                self.counters.terminals.fetch_add(1, Ordering::SeqCst);
                Err(error)
            }
        }
    }
}

fn authenticated_principal(
    actor: &M00AdmittedActor,
) -> Result<AuthenticatedPrincipal, OpportunityInvocationError> {
    let M00AdmittedActor::Authenticated(ids) = actor else {
        return Err(OpportunityInvocationError::Denied);
    };
    AuthenticatedPrincipal::new(ids.tenant_id().clone(), ids.user_id().clone())
        .map_err(|_| OpportunityInvocationError::Internal)
}

fn unix_millis(
    value: ustc_campus_agent_client_protocol::UnixMillis,
) -> Result<OffsetDateTime, OpportunityInvocationError> {
    OffsetDateTime::from_unix_timestamp_nanos(i128::from(value.get()) * 1_000_000)
        .map_err(|_| OpportunityInvocationError::Internal)
}
