#![allow(clippy::unwrap_used)]

mod common;

use affairs_navigator::*;
use common::t;
use ustc_campus_agent_core::source_registry::{
    SourceId as M60SourceId, SourceReviewEvidenceId, SourceReviewerId, SourceUrl,
};
use ustc_campus_agent_core::source_revision::{
    EffectiveInterval as M60EffectiveInterval, NormalizedSnapshotId, ParserIdentity, RawSnapshotId,
    RevisionSha256, RevisionTimestamp, SourceRevision, SourceRevisionHealth,
};

fn source_revision(number: u8, url_suffix: &str) -> SourceRevision {
    let mut raw = [0_u8; 32];
    raw[31] = number;
    let mut normalized = [1_u8; 32];
    normalized[31] = number;
    SourceRevision::demo_reviewed(
        M60SourceId::parse("source:demo:affairs").unwrap(),
        SourceUrl::parse(format!("https://demo.example/{url_suffix}")).unwrap(),
        RawSnapshotId::parse(format!("raw:affairs:{number}")).unwrap(),
        RevisionSha256::from_bytes(raw),
        NormalizedSnapshotId::parse(format!("normalized:affairs:{number}")).unwrap(),
        RevisionSha256::from_bytes(normalized),
        ParserIdentity::parse("parser:affairs:v1").unwrap(),
        RevisionTimestamp::from_unix_seconds(i64::from(number)),
        None,
        M60EffectiveInterval::new(None, None).unwrap(),
        SourceReviewerId::parse("reviewer:demo:admin").unwrap(),
        SourceReviewEvidenceId::parse("review-evidence:demo:affairs").unwrap(),
    )
}

fn local_ref(revision: &SourceRevision) -> M60RevisionRef {
    M60RevisionRef::new(
        SourceId::parse(revision.source_id().as_str()).unwrap(),
        revision.revision_id().as_str().to_owned(),
        t(revision.observed_at().unix_seconds()),
        revision.published_at().map(|time| t(time.unix_seconds())),
        revision
            .effective_interval()
            .from()
            .map(|time| t(time.unix_seconds())),
        revision
            .effective_interval()
            .to()
            .map(|time| t(time.unix_seconds())),
        Sha256::new(revision.raw_sha256().as_str()).unwrap(),
        Sha256::new(revision.normalized_sha256().as_str()).unwrap(),
    )
    .unwrap()
}

fn draft_candidate(
    revision: SourceRevision,
    uncertainty: UncertaintyState,
) -> Result<ProcedureDraft, ProcedureDraftError> {
    let revision_ref = local_ref(&revision);
    let authority = AffairsAuthorityAssessment::new(
        AffairsAuthority::OfficialBulletin,
        AuthoritySubject::ProcedureSteps,
        AuthorityDerivation::Direct,
        t(10),
        ActorRef::parse("actor:demo:source-reviewer").unwrap(),
    );
    let assessment = AffairsEvidenceAssessment::new(revision_ref, authority, t(10), t(20));
    let evidence = ProcedureEvidenceContext::new(
        ValidityHorizon::Unknown,
        t(1),
        t(20),
        t(10),
        t(20),
        vec![assessment],
        EvidenceConflictState::NoKnownConflict,
        AuthorityComparison::Equivalent,
        uncertainty,
        None,
        Vec::new(),
    )
    .unwrap();
    let board_policy = BoardPolicy::new(
        BoardId::parse("board:demo:affairs").unwrap(),
        BoardPolicyVersion::new(1).unwrap(),
        86_400,
        604_800,
    )
    .unwrap();
    let contact = Contact::new(
        ContactRef::parse("contact:demo:desk").unwrap(),
        ContactName::new("Demo Affairs Desk").unwrap(),
        ContactChannel::new("web").unwrap(),
        SourceId::parse("source:demo:affairs").unwrap(),
    );
    let entry = EntryPoint::new(
        EntryPointLabel::new("Demo portal").unwrap(),
        Some(Url::new("https://demo.example/affairs").unwrap()),
        ContactRef::parse("contact:demo:desk").unwrap(),
    );
    ProcedureDraft::from_demo_reviewed(
        revision,
        ProcedureId::parse("procedure:demo:registration").unwrap(),
        Title::new("Demo registration procedure").unwrap(),
        vec![AudienceTag::new("students").unwrap()],
        board_policy,
        Vec::new(),
        vec![ProcedureStep::new(
            0,
            Instruction::new("Submit the reviewed demo form").unwrap(),
        )],
        Vec::new(),
        None,
        vec![entry],
        vec![contact],
        evidence,
    )
}

fn approval(draft: &ProcedureDraft, reviewed_at: i64) -> ProcedureReviewApproval {
    ProcedureReviewApproval::new(
        draft.draft_digest().clone(),
        ActorRef::parse("actor:demo:administrator").unwrap(),
        t(reviewed_at),
    )
}

#[test]
fn reviewed_draft_publishes_atomically_and_retry_is_idempotent() {
    let revision = source_revision(1, "registration-v1");
    let draft = draft_candidate(revision.clone(), UncertaintyState::None).unwrap();
    let review = approval(&draft, 30);
    let mut m60 = m60_fixture::M60FixtureAdapter::new("verifier:demo", 1).unwrap();
    m60.store(local_ref(&revision));
    let mut repository = InMemoryPublishedAffairsRepository::new();

    let receipt = ProcedurePublicationService::new(&mut repository, &m60)
        .publish(draft.clone(), review.clone(), t(40), None)
        .unwrap();
    let retry = ProcedurePublicationService::new(&mut repository, &m60)
        .publish(draft.clone(), review.clone(), t(40), None)
        .unwrap();
    let wrong_precondition = ProcedurePublicationService::new(&mut repository, &m60)
        .publish(draft, review, t(40), Some(0))
        .unwrap_err();

    assert_eq!(receipt, retry);
    assert_eq!(
        wrong_precondition,
        ProcedurePublicationError::Repository(
            ProcedurePublicationRepositoryError::PublicationConflict
        )
    );
    assert_eq!(receipt.publication_revision(), 1);
    assert_eq!(receipt.m60_revision_count(), 1);
    assert_eq!(repository.artifact_count(), 1);
    assert_eq!(repository.receipt_count(), 1);
    let state = repository
        .find_publication_state(receipt.procedure_id())
        .unwrap();
    assert_eq!(state.publication_revision(), 1);
    assert_eq!(state.current_artifact_id(), Some(receipt.artifact_id()));
    assert_eq!(
        repository
            .find_current_artifact(receipt.procedure_id())
            .unwrap()
            .artifact_id(),
        receipt.artifact_id()
    );
}

#[test]
fn uncertain_evidence_cannot_become_a_draft() {
    assert_eq!(
        draft_candidate(
            source_revision(1, "registration-v1"),
            UncertaintyState::Stale,
        )
        .unwrap_err(),
        ProcedureDraftError::EvidenceNotPublishable
    );
}

#[test]
fn revoked_m60_revision_fails_before_repository_mutation() {
    let revision = source_revision(1, "registration-v1");
    let revision_ref = local_ref(&revision);
    let draft = draft_candidate(revision, UncertaintyState::None).unwrap();
    let review = approval(&draft, 30);
    let mut m60 = m60_fixture::M60FixtureAdapter::new("verifier:demo", 1).unwrap();
    m60.store(revision_ref.clone());
    m60.revoke(revision_ref.source_id(), revision_ref.revision_id());
    let mut repository = InMemoryPublishedAffairsRepository::new();

    assert_eq!(
        ProcedurePublicationService::new(&mut repository, &m60)
            .publish(draft, review, t(40), None)
            .unwrap_err(),
        ProcedurePublicationError::M60Unverified(M60EvidenceUnverifiedReason::RevokedOrUnaccepted)
    );
    assert_eq!(repository.artifact_count(), 0);
    assert_eq!(repository.receipt_count(), 0);
}

#[test]
fn approval_chronology_and_provider_failure_are_typed_and_non_mutating() {
    let revision = source_revision(1, "registration-v1");
    let draft = draft_candidate(revision.clone(), UncertaintyState::None).unwrap();
    let mut m60 = m60_fixture::M60FixtureAdapter::new("verifier:demo", 1).unwrap();
    m60.store(local_ref(&revision));
    let mut repository = InMemoryPublishedAffairsRepository::new();

    let wrong_approval = ProcedureReviewApproval::new(
        Sha256::new(format!("sha256:{}", "0".repeat(64))).unwrap(),
        ActorRef::parse("actor:demo:administrator").unwrap(),
        t(30),
    );
    assert_eq!(
        ProcedurePublicationService::new(&mut repository, &m60)
            .publish(draft.clone(), wrong_approval, t(40), None)
            .unwrap_err(),
        ProcedurePublicationError::ApprovalMismatch
    );
    assert_eq!(
        ProcedurePublicationService::new(&mut repository, &m60)
            .publish(draft.clone(), approval(&draft, 19), t(40), None)
            .unwrap_err(),
        ProcedurePublicationError::ReviewBeforeKnown
    );
    assert_eq!(
        ProcedurePublicationService::new(&mut repository, &m60)
            .publish(draft.clone(), approval(&draft, 30), t(29), None)
            .unwrap_err(),
        ProcedurePublicationError::PublishBeforeReview
    );

    m60.set_revision_health(SourceRevisionHealth::Stale);
    assert_eq!(
        ProcedurePublicationService::new(&mut repository, &m60)
            .publish(draft.clone(), approval(&draft, 30), t(40), None)
            .unwrap_err(),
        ProcedurePublicationError::SourceNotCurrent(SourceRevisionHealth::Stale)
    );
    m60.set_revision_health(SourceRevisionHealth::Current);
    m60.set_failure_mode(Some(M60EvidencePortError::StoreUnavailable));
    assert_eq!(
        ProcedurePublicationService::new(&mut repository, &m60)
            .publish(draft.clone(), approval(&draft, 30), t(40), None)
            .unwrap_err(),
        ProcedurePublicationError::M60StoreUnavailable
    );
    assert_eq!(repository.artifact_count(), 0);
    assert_eq!(repository.receipt_count(), 0);
}

#[test]
fn persistence_failure_and_cas_conflict_advance_no_state() {
    let revision = source_revision(1, "registration-v1");
    let draft = draft_candidate(revision.clone(), UncertaintyState::None).unwrap();
    let review = approval(&draft, 30);
    let first_ref = local_ref(&revision);
    let mut m60 = m60_fixture::M60FixtureAdapter::new("verifier:demo", 1).unwrap();
    m60.store(first_ref.clone());
    let mut repository = InMemoryPublishedAffairsRepository::new();
    repository.fail_next_publication();

    assert_eq!(
        ProcedurePublicationService::new(&mut repository, &m60)
            .publish(draft.clone(), review.clone(), t(40), None)
            .unwrap_err(),
        ProcedurePublicationError::Repository(ProcedurePublicationRepositoryError::FailureInjected)
    );
    assert_eq!(repository.artifact_count(), 0);
    let first = ProcedurePublicationService::new(&mut repository, &m60)
        .publish(draft.clone(), review.clone(), t(40), None)
        .unwrap();

    let second_revision = source_revision(2, "registration-v2");
    let second_draft = draft_candidate(second_revision.clone(), UncertaintyState::None).unwrap();
    let second_review = approval(&second_draft, 45);
    m60.store(local_ref(&second_revision));
    assert_eq!(
        ProcedurePublicationService::new(&mut repository, &m60)
            .publish(second_draft.clone(), second_review.clone(), t(50), Some(0),)
            .unwrap_err(),
        ProcedurePublicationError::Repository(
            ProcedurePublicationRepositoryError::PublicationConflict
        )
    );
    assert_eq!(repository.artifact_count(), 1);
    assert_eq!(
        repository.publication_revision(first.procedure_id()),
        Some(1)
    );

    let second = ProcedurePublicationService::new(&mut repository, &m60)
        .publish(second_draft, second_review, t(50), Some(1))
        .unwrap();
    assert_eq!(second.publication_revision(), 2);
    assert_ne!(first.artifact_id(), second.artifact_id());
    assert_eq!(repository.artifact_count(), 2);
    assert_eq!(repository.receipt_count(), 2);

    // A replay remains idempotent after a newer revision becomes current and
    // after the historical source revision is revoked: the immutable
    // receipt+artifact tombstone, not mutable source state or current-state
    // equality, owns an already-committed command identity.
    m60.revoke(first_ref.source_id(), first_ref.revision_id());
    let first_replay = ProcedurePublicationService::new(&mut repository, &m60)
        .publish(draft, review, t(40), None)
        .unwrap();
    assert_eq!(first_replay, first);
    assert_eq!(
        repository.publication_revision(first.procedure_id()),
        Some(2)
    );
}

#[test]
fn source_revision_url_is_bound_into_draft_and_artifact_identity() {
    let left = draft_candidate(
        source_revision(1, "registration-west"),
        UncertaintyState::None,
    )
    .unwrap();
    let right = draft_candidate(
        source_revision(1, "registration-east"),
        UncertaintyState::None,
    )
    .unwrap();
    assert_ne!(
        left.source_revision().revision_id(),
        right.source_revision().revision_id()
    );
    assert_ne!(left.draft_digest(), right.draft_digest());
}
