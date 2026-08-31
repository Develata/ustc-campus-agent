use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use ustc_campus_agent_application_ingress::{
    ChangePublicationApplicationError, ChangePublicationApplicationPort,
};
use ustc_campus_agent_change_radar::{
    BoardFeedPolicy, ChangeEventId, ChangePublicationRepository, ChangePublicationService,
    ChangeReviewReceipt, M60ChangePublicationPort, PublishedChangeEvent, SemanticChangeCandidate,
};
use ustc_campus_agent_core::identity::{CommandId, SessionId, TenantId, UserId};
use ustc_campus_agent_core::request_context::M00AdmittedActor;
use ustc_campus_agent_core::source_revision::RevisionTimestamp;

#[derive(Clone, Default)]
pub(crate) struct ChangePublicationCounters {
    applications: Arc<AtomicU64>,
}

impl ChangePublicationCounters {
    pub(crate) fn applications(&self) -> u64 {
        self.applications.load(Ordering::SeqCst)
    }
}

pub(crate) struct FixtureChangePublicationPort<'a, R> {
    repository: &'a mut R,
    m60: &'a dyn M60ChangePublicationPort,
    candidate: &'a SemanticChangeCandidate,
    review: &'a ChangeReviewReceipt,
    feed_policy: &'a BoardFeedPolicy,
    published_at: RevisionTimestamp,
    expected_tenant_id: &'a TenantId,
    expected_user_id: &'a UserId,
    expected_session_id: &'a SessionId,
    counters: ChangePublicationCounters,
}

impl<'a, R> FixtureChangePublicationPort<'a, R>
where
    R: ChangePublicationRepository + Send,
{
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        repository: &'a mut R,
        m60: &'a dyn M60ChangePublicationPort,
        candidate: &'a SemanticChangeCandidate,
        review: &'a ChangeReviewReceipt,
        feed_policy: &'a BoardFeedPolicy,
        published_at: RevisionTimestamp,
        expected_tenant_id: &'a TenantId,
        expected_user_id: &'a UserId,
        expected_session_id: &'a SessionId,
        counters: ChangePublicationCounters,
    ) -> Self {
        Self {
            repository,
            m60,
            candidate,
            review,
            feed_policy,
            published_at,
            expected_tenant_id,
            expected_user_id,
            expected_session_id,
            counters,
        }
    }
}

impl<R> ChangePublicationApplicationPort for FixtureChangePublicationPort<'_, R>
where
    R: ChangePublicationRepository + Send,
{
    fn publish(
        &mut self,
        _command_id: &CommandId,
        actor: &M00AdmittedActor,
        event_id: &ChangeEventId,
        review_receipt_id: &str,
        reviewed_at: RevisionTimestamp,
        published_at: RevisionTimestamp,
    ) -> Result<PublishedChangeEvent, ChangePublicationApplicationError> {
        self.counters.applications.fetch_add(1, Ordering::SeqCst);
        let M00AdmittedActor::Authenticated(identities) = actor else {
            return Err(ChangePublicationApplicationError::Denied);
        };
        if identities.tenant_id() != self.expected_tenant_id
            || identities.user_id() != self.expected_user_id
            || identities.session_id() != self.expected_session_id
            || event_id != self.candidate.event_id()
            || review_receipt_id != self.review.receipt_id().as_str()
            || reviewed_at != self.review.reviewed_at()
            || published_at != self.published_at
        {
            return Err(ChangePublicationApplicationError::Denied);
        }
        let mut publication =
            ChangePublicationService::new(self.repository, self.m60, self.feed_policy.clone());
        publication
            .record_review(self.review.clone())
            .map_err(ChangePublicationApplicationError::Downstream)?;
        publication
            .publish(event_id, published_at)
            .map_err(ChangePublicationApplicationError::Downstream)
    }
}
