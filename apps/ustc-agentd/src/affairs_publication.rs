use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use affairs_navigator::{
    ActorRef, ProcedureDraft, ProcedureId, ProcedurePublicationRepository,
    ProcedurePublicationService, ProcedureReviewApproval,
};
use time::OffsetDateTime;
use ustc_campus_agent_application_ingress::{
    AffairsPublicationApplicationError, AffairsPublicationApplicationPort,
};
use ustc_campus_agent_core::identity::{CommandId, SessionId, TenantId, UserId};
use ustc_campus_agent_core::request_context::M00AdmittedActor;

use crate::affairs_fixture::CountingM60Port;

#[derive(Clone, Default)]
pub(crate) struct AffairsPublicationCounters {
    applications: Arc<AtomicU64>,
}

impl AffairsPublicationCounters {
    pub(crate) fn applications(&self) -> u64 {
        self.applications.load(Ordering::SeqCst)
    }
}

pub(crate) struct FixtureAffairsPublicationPort<'a> {
    repository: &'a mut dyn ProcedurePublicationRepository,
    m60: &'a CountingM60Port,
    draft: &'a ProcedureDraft,
    reviewer: &'a ActorRef,
    reviewed_at: OffsetDateTime,
    published_at: OffsetDateTime,
    expected_tenant_id: &'a TenantId,
    expected_user_id: &'a UserId,
    expected_session_id: &'a SessionId,
    counters: AffairsPublicationCounters,
}

impl<'a> FixtureAffairsPublicationPort<'a> {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        repository: &'a mut dyn ProcedurePublicationRepository,
        m60: &'a CountingM60Port,
        draft: &'a ProcedureDraft,
        reviewer: &'a ActorRef,
        reviewed_at: OffsetDateTime,
        published_at: OffsetDateTime,
        expected_tenant_id: &'a TenantId,
        expected_user_id: &'a UserId,
        expected_session_id: &'a SessionId,
        counters: AffairsPublicationCounters,
    ) -> Self {
        Self {
            repository,
            m60,
            draft,
            reviewer,
            reviewed_at,
            published_at,
            expected_tenant_id,
            expected_user_id,
            expected_session_id,
            counters,
        }
    }
}

impl AffairsPublicationApplicationPort for FixtureAffairsPublicationPort<'_> {
    fn publish(
        &mut self,
        _command_id: &CommandId,
        actor: &M00AdmittedActor,
        procedure_id: &ProcedureId,
        expected_publication_revision: Option<u64>,
    ) -> Result<affairs_navigator::ProcedurePublicationReceipt, AffairsPublicationApplicationError>
    {
        self.counters.applications.fetch_add(1, Ordering::SeqCst);
        let M00AdmittedActor::Authenticated(identities) = actor else {
            return Err(AffairsPublicationApplicationError::Denied);
        };
        let current_revision = self.repository.publication_revision(procedure_id);
        if identities.tenant_id() != self.expected_tenant_id
            || identities.user_id() != self.expected_user_id
            || identities.session_id() != self.expected_session_id
            || procedure_id != self.draft.procedure_id()
            || expected_publication_revision != Some(1)
            || !matches!(current_revision, Some(1) | Some(2))
        {
            return Err(AffairsPublicationApplicationError::Denied);
        }
        let approval = ProcedureReviewApproval::new(
            self.draft.draft_digest().clone(),
            self.reviewer.clone(),
            self.reviewed_at,
        );
        ProcedurePublicationService::new(self.repository, self.m60)
            .publish(
                self.draft.clone(),
                approval,
                self.published_at,
                expected_publication_revision,
            )
            .map_err(AffairsPublicationApplicationError::Downstream)
    }
}
