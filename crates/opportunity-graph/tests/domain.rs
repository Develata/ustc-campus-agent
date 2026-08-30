#![allow(clippy::unwrap_used)]

use std::collections::{BTreeMap, BTreeSet};
use std::sync::atomic::{AtomicUsize, Ordering};
use time::OffsetDateTime;
use ustc_campus_agent_core::identity::{TenantId, UserId};
use ustc_campus_agent_core::source_registry::{
    SourceId, SourceReviewEvidenceId, SourceReviewerId, SourceUrl,
};
use ustc_campus_agent_core::source_revision::{
    EffectiveInterval, NormalizedSnapshotId, ParserIdentity, RawSnapshotId, RevisionSha256,
    RevisionTimestamp, SourceRevision, SourceRevisionHealth,
};
use ustc_campus_agent_course_planning::{CoursePlanningFixture, PlanningConfig};
use ustc_campus_agent_opportunity_graph::*;

fn at(seconds: i64) -> OffsetDateTime {
    OffsetDateTime::from_unix_timestamp(seconds).unwrap()
}

fn principal(tenant: &str, user: &str) -> AuthenticatedPrincipal {
    AuthenticatedPrincipal::new(
        TenantId::parse(tenant).unwrap(),
        UserId::parse(user).unwrap(),
    )
    .unwrap()
}

fn source_revision(number: u8) -> SourceRevision {
    let mut raw = [0_u8; 32];
    raw[31] = number;
    let mut normalized = [1_u8; 32];
    normalized[31] = number;
    SourceRevision::demo_reviewed(
        SourceId::parse("source:demo:opportunity-catalog").unwrap(),
        SourceUrl::parse(format!("https://demo.example/opportunities/{number}")).unwrap(),
        RawSnapshotId::parse(format!("raw:opportunity:{number}")).unwrap(),
        RevisionSha256::from_bytes(raw),
        NormalizedSnapshotId::parse(format!("normalized:opportunity:{number}")).unwrap(),
        RevisionSha256::from_bytes(normalized),
        ParserIdentity::parse("parser:opportunity:v1").unwrap(),
        RevisionTimestamp::from_unix_seconds(i64::from(number)),
        None,
        EffectiveInterval::new(None, None).unwrap(),
        SourceReviewerId::parse("reviewer:demo:opportunity").unwrap(),
        SourceReviewEvidenceId::parse("review-evidence:demo:opportunity").unwrap(),
    )
}

fn catalog_and_profile() -> (ReviewedOpportunityCatalog, AcademicProfileInput) {
    let mut fixture: CoursePlanningFixture = serde_json::from_str(include_str!(
        "../../../market/fixtures/course-planning/minimal-v0.json"
    ))
    .unwrap();
    let profile = AcademicProfileInput::new(
        fixture.profile.completed_courses.clone(),
        fixture.profile.min_credits,
        fixture.profile.max_credits,
        fixture.profile.preference_weights.clone(),
    )
    .unwrap();
    let revision = source_revision(1);
    fixture.source_revision = revision.revision_id().as_str().to_owned();
    (
        ReviewedOpportunityCatalog::from_demo_reviewed(revision, fixture).unwrap(),
        profile,
    )
}

fn consent() -> ConsentGrant {
    ConsentGrant::new(
        ConsentPurpose::OpportunityPlanning,
        [
            ConsentField::CompletedCourses,
            ConsentField::CreditBounds,
            ConsentField::PreferenceWeights,
        ],
        at(10),
    )
    .unwrap()
}

struct SourcePort {
    health: SourceRevisionHealth,
    error: Option<OpportunitySourcePortError>,
    calls: AtomicUsize,
}

impl SourcePort {
    fn current() -> Self {
        Self {
            health: SourceRevisionHealth::Current,
            error: None,
            calls: AtomicUsize::new(0),
        }
    }
}

impl M60OpportunityPort for SourcePort {
    fn revision_health(
        &self,
        _revision: &SourceRevision,
    ) -> Result<SourceRevisionHealth, OpportunitySourcePortError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        self.error.map_or(Ok(self.health), Err)
    }
}

#[test]
fn consented_profile_plans_with_qualification_provenance_then_deletes() {
    let (catalog, profile) = catalog_and_profile();
    let owner = principal("tenant:alpha", "user:alice");
    let mut repository = InMemoryOpportunityProfileRepository::new(4, 8).unwrap();
    let record = OpportunityProfileService::new(&mut repository)
        .create_profile(owner.clone(), consent(), profile)
        .unwrap();

    let source = SourcePort::current();
    let receipt = OpportunityPlanningService::new(&repository, &source, &catalog)
        .plan(
            &owner,
            record.profile_snapshot_id(),
            PlanningConfig::default(),
        )
        .unwrap();
    assert_eq!(source.calls.load(Ordering::SeqCst), 1);
    let qualification_codes: BTreeSet<_> = receipt
        .qualifications()
        .iter()
        .map(CourseQualification::course_code)
        .collect();
    assert_eq!(qualification_codes.len(), receipt.qualifications().len());
    assert!(receipt.qualifications().iter().any(|item| {
        item.course_code() == "MATH2001" && item.eligible() && item.blockers().is_empty()
    }));
    let OpportunityPlanDecision::Planned { result } = receipt.decision() else {
        panic!("expected a feasible plan");
    };
    assert_eq!(result.hard_constraint_violations, 0);
    assert!(!result.candidates.is_empty());
    assert!(result.candidates.iter().all(|candidate| {
        candidate.hard_constraint_violations.is_empty() && !candidate.provenance.is_empty()
    }));
    assert_eq!(
        receipt.staleness(
            catalog.source_revision().revision_id(),
            record.profile_snapshot_id()
        ),
        PlanStaleness::Current
    );
    let encoded = serde_json::to_string(&receipt).unwrap();
    assert!(!encoded.contains("completed_courses"));
    assert!(!encoded.contains("preference_weights"));
    assert!(!format!("{record:?}").contains("MATH1001"));

    let deletion = OpportunityProfileService::new(&mut repository)
        .revoke_consent_and_delete_profile(&owner, record.profile_snapshot_id(), at(20))
        .unwrap();
    assert_eq!(repository.private_payload_count(), 0);
    assert_eq!(repository.tombstone_count(), 1);
    assert_eq!(deletion.profile_snapshot_id(), record.profile_snapshot_id());

    let replay = OpportunityProfileService::new(&mut repository)
        .revoke_consent_and_delete_profile(&owner, record.profile_snapshot_id(), at(30))
        .unwrap();
    assert_eq!(replay, deletion);
    assert!(matches!(
        OpportunityPlanningService::new(&repository, &source, &catalog).plan(
            &owner,
            record.profile_snapshot_id(),
            PlanningConfig::default()
        ),
        Err(OpportunityPlanningError::ProfileDeleted)
    ));
}

#[test]
fn wrong_tenant_is_denied_before_source_or_private_payload_read() {
    let (catalog, profile) = catalog_and_profile();
    let owner = principal("tenant:alpha", "user:alice");
    let intruder = principal("tenant:beta", "user:bob");
    let mut repository = InMemoryOpportunityProfileRepository::new(4, 8).unwrap();
    let record = OpportunityProfileService::new(&mut repository)
        .create_profile(owner, consent(), profile)
        .unwrap();
    let source = SourcePort::current();

    assert!(matches!(
        OpportunityProfileRepository::lookup(&repository, &intruder, record.profile_snapshot_id()),
        Ok(ProfileLookup::AccessDenied)
    ));
    assert!(matches!(
        OpportunityPlanningService::new(&repository, &source, &catalog).plan(
            &intruder,
            record.profile_snapshot_id(),
            PlanningConfig::default()
        ),
        Err(OpportunityPlanningError::AccessDenied)
    ));
    assert_eq!(source.calls.load(Ordering::SeqCst), 0);
    assert!(matches!(
        OpportunityProfileService::new(&mut repository)
            .view_profile(&intruder, record.profile_snapshot_id()),
        Err(OpportunityProfileError::AccessDenied)
    ));
}

#[test]
fn stale_and_unavailable_sources_fail_closed_after_profile_authorization() {
    let (catalog, profile) = catalog_and_profile();
    let owner = principal("tenant:alpha", "user:alice");
    let mut repository = InMemoryOpportunityProfileRepository::new(4, 8).unwrap();
    let record = OpportunityProfileService::new(&mut repository)
        .create_profile(owner.clone(), consent(), profile)
        .unwrap();

    let stale = SourcePort {
        health: SourceRevisionHealth::Stale,
        error: None,
        calls: AtomicUsize::new(0),
    };
    assert!(matches!(
        OpportunityPlanningService::new(&repository, &stale, &catalog).plan(
            &owner,
            record.profile_snapshot_id(),
            PlanningConfig::default()
        ),
        Err(OpportunityPlanningError::SourceNotCurrent(
            SourceRevisionHealth::Stale
        ))
    ));
    let unavailable = SourcePort {
        health: SourceRevisionHealth::Current,
        error: Some(OpportunitySourcePortError::Unavailable),
        calls: AtomicUsize::new(0),
    };
    assert!(matches!(
        OpportunityPlanningService::new(&repository, &unavailable, &catalog).plan(
            &owner,
            record.profile_snapshot_id(),
            PlanningConfig::default()
        ),
        Err(OpportunityPlanningError::SourceUnavailable(
            OpportunitySourcePortError::Unavailable
        ))
    ));
}

#[test]
fn deletion_failure_never_claims_completion_or_removes_payload() {
    let (_catalog, profile) = catalog_and_profile();
    let owner = principal("tenant:alpha", "user:alice");
    let mut repository = InMemoryOpportunityProfileRepository::new(4, 8).unwrap();
    let record = OpportunityProfileService::new(&mut repository)
        .create_profile(owner.clone(), consent(), profile)
        .unwrap();
    repository.set_failure_mode(RepositoryFailureMode::WriteUnavailable);

    assert!(matches!(
        OpportunityProfileService::new(&mut repository).revoke_consent_and_delete_profile(
            &owner,
            record.profile_snapshot_id(),
            at(20)
        ),
        Err(OpportunityProfileError::Repository(
            OpportunityRepositoryError::Unavailable
        ))
    ));
    assert_eq!(repository.private_payload_count(), 1);
    assert_eq!(repository.tombstone_count(), 0);
}

#[test]
fn receipt_staleness_binds_both_source_and_profile_revisions() {
    let (catalog, profile) = catalog_and_profile();
    let owner = principal("tenant:alpha", "user:alice");
    let mut repository = InMemoryOpportunityProfileRepository::new(4, 8).unwrap();
    let record = OpportunityProfileService::new(&mut repository)
        .create_profile(owner.clone(), consent(), profile)
        .unwrap();
    let source = SourcePort::current();
    let receipt = OpportunityPlanningService::new(&repository, &source, &catalog)
        .plan(
            &owner,
            record.profile_snapshot_id(),
            PlanningConfig::default(),
        )
        .unwrap();
    let newer_source = source_revision(2);
    let newer_profile = ProfileSnapshotId::parse(
        "profile-snapshot:opportunity:sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
    )
    .unwrap();

    assert_eq!(
        receipt.staleness(newer_source.revision_id(), record.profile_snapshot_id()),
        PlanStaleness::SourceChanged
    );
    assert_eq!(
        receipt.staleness(catalog.source_revision().revision_id(), &newer_profile),
        PlanStaleness::ProfileChanged
    );
    assert_eq!(
        receipt.staleness(newer_source.revision_id(), &newer_profile),
        PlanStaleness::SourceAndProfileChanged
    );
}

#[test]
fn exact_consent_field_set_is_required_and_no_feasible_plan_still_returns_qualification() {
    assert!(matches!(
        ConsentGrant::new(
            ConsentPurpose::OpportunityPlanning,
            [ConsentField::CompletedCourses],
            at(10)
        ),
        Err(OpportunityValueError::InvalidConsentFields)
    ));

    let (catalog, _) = catalog_and_profile();
    let owner = principal("tenant:alpha", "user:alice");
    let constrained = AcademicProfileInput::new(Vec::new(), 1, 1, BTreeMap::new()).unwrap();
    let mut repository = InMemoryOpportunityProfileRepository::new(4, 8).unwrap();
    let record = OpportunityProfileService::new(&mut repository)
        .create_profile(owner.clone(), consent(), constrained)
        .unwrap();
    let source = SourcePort::current();
    let receipt = OpportunityPlanningService::new(&repository, &source, &catalog)
        .plan(
            &owner,
            record.profile_snapshot_id(),
            PlanningConfig::default(),
        )
        .unwrap();
    assert!(matches!(
        receipt.decision(),
        OpportunityPlanDecision::NoFeasiblePlan
    ));
    assert!(
        receipt
            .qualifications()
            .iter()
            .any(|item| !item.eligible() && !item.blockers().is_empty())
    );
}

#[test]
fn planning_bounds_and_debug_redaction_fail_closed_before_source_access() {
    let (catalog, profile) = catalog_and_profile();
    let owner = principal("tenant:private-sentinel", "user:private-sentinel");
    let principal_debug = format!("{owner:?}");
    assert!(!principal_debug.contains("tenant:private-sentinel"));
    assert!(!principal_debug.contains("user:private-sentinel"));
    let profile_debug = format!("{profile:?}");
    assert!(!profile_debug.contains("min_credits"));
    assert!(!profile_debug.contains("max_credits"));
    assert!(!profile_debug.contains("MATH1001"));

    let mut repository = InMemoryOpportunityProfileRepository::new(4, 8).unwrap();
    let record = OpportunityProfileService::new(&mut repository)
        .create_profile(owner.clone(), consent(), profile.clone())
        .unwrap();
    let replay = OpportunityProfileService::new(&mut repository)
        .create_profile(owner.clone(), consent(), profile)
        .unwrap();
    assert_eq!(replay, record);
    assert_eq!(repository.private_payload_count(), 1);

    let source = SourcePort::current();
    assert!(matches!(
        OpportunityPlanningService::new(&repository, &source, &catalog).plan(
            &owner,
            record.profile_snapshot_id(),
            PlanningConfig {
                max_results: MAX_OPPORTUNITY_PLAN_RESULTS + 1,
                beam_width: MAX_OPPORTUNITY_BEAM_WIDTH,
            }
        ),
        Err(OpportunityPlanningError::InvalidPlanningBounds)
    ));
    assert!(matches!(
        OpportunityPlanningService::new(&repository, &source, &catalog).plan(
            &owner,
            record.profile_snapshot_id(),
            PlanningConfig {
                max_results: MAX_OPPORTUNITY_PLAN_RESULTS,
                beam_width: MAX_OPPORTUNITY_BEAM_WIDTH + 1,
            }
        ),
        Err(OpportunityPlanningError::InvalidPlanningBounds)
    ));
    assert_eq!(source.calls.load(Ordering::SeqCst), 0);
}
