#![allow(clippy::unwrap_used)]

//! Projection boundary tests at 1/8/9/16 groups.

mod common;

use affairs_navigator::*;
use common::*;
use time::OffsetDateTime;

const ALL_SUBJECTS: [evidence::AuthoritySubject; 8] = [
    evidence::AuthoritySubject::ProcedureTitle,
    evidence::AuthoritySubject::ProcedureSteps,
    evidence::AuthoritySubject::ProcedureDeadlines,
    evidence::AuthoritySubject::ProcedureEffectiveInterval,
    evidence::AuthoritySubject::ProcedureEntryPoints,
    evidence::AuthoritySubject::ProcedureContacts,
    evidence::AuthoritySubject::ProcedurePrerequisites,
    evidence::AuthoritySubject::ProcedureEvidence,
];

fn run_found(
    assessments: Vec<evidence::AffairsEvidenceAssessment>,
) -> outcome::GetProcedureOutcome {
    run_found_with_prerequisites(assessments, Vec::new())
}

fn run_found_with_prerequisites(
    assessments: Vec<evidence::AffairsEvidenceAssessment>,
    prerequisites: Vec<artifact::Prerequisite>,
) -> outcome::GetProcedureOutcome {
    let artifact = build_artifact_full(
        "proc:proj",
        50,
        150,
        evidence::EvidenceConflictState::NoKnownConflict,
        evidence::AuthorityComparison::Equivalent,
        None,
        100,
        200,
        assessments,
        prerequisites,
    );
    let mut repo = repository::InMemoryAffairsRepository::new();
    seed_current(&mut repo, artifact.clone());
    let mut m60 = m60_fixture::M60FixtureAdapter::new("verifier:fixture", 1).unwrap();
    // Store every revision ref the artifact's evidence carries so M60
    // verification succeeds regardless of source naming.
    for assess in artifact.evidence().evidence_assessments() {
        m60.store(assess.revision_ref().clone());
    }
    let clock = clock::FixedClock::new(t(200));
    let service = service::AffairsGetService::new(&repo, &m60, &clock);
    let query = outcome::AffairsGetQuery::new(
        value::ProcedureId::parse("proc:proj").unwrap(),
        Some(t(200)),
    );
    match service.execute(&query).unwrap().outcome().clone() {
        o @ outcome::GetProcedureOutcome::Found { .. } => o,
        other => panic!("expected Found, got {other:?}"),
    }
}

fn one_group_assessment(source: &str) -> evidence::AffairsEvidenceAssessment {
    assessment(
        evidence::AffairsAuthority::OfficialBulletin,
        source,
        evidence::AuthoritySubject::ProcedureTitle,
        100,
        100,
        None,
        None,
    )
}

#[test]
fn one_group_complete() {
    let outcome = run_found(vec![one_group_assessment("s1")]);
    if let outcome::GetProcedureOutcome::Found { view, .. } = outcome {
        assert_eq!(view.evidence().evidence_assessments().len(), 1);
        assert_eq!(
            view.evidence().projection(),
            public_view::ProjectionMetadata::Complete
        );
    }
}

#[test]
fn mandatory_groups_preserve_ascending_group_index_before_fill_order() {
    // The coalesced BTreeMap orders the lower-tier group first. It is mandatory
    // because a surviving prerequisite references it; the OfficialBulletin
    // group is mandatory because it is maximal-tier. The fill comparator would
    // reverse these two, but M71-v8n requires mandatory GroupIndex order.
    let low = assessment(
        evidence::AffairsAuthority::ReviewedCommunitySummary,
        "low1",
        evidence::AuthoritySubject::ProcedureTitle,
        100,
        100,
        None,
        None,
    );
    let low_ref = low.revision_ref().clone();
    let high = assessment(
        evidence::AffairsAuthority::OfficialBulletin,
        "high1",
        evidence::AuthoritySubject::ProcedureSteps,
        100,
        100,
        None,
        None,
    );
    let prerequisite = artifact::Prerequisite::new(
        value::PrerequisiteCondition::new("Bring the referenced document").unwrap(),
        Some(low_ref),
    );

    let outcome = run_found_with_prerequisites(vec![low, high], vec![prerequisite]);
    if let outcome::GetProcedureOutcome::Found { view, .. } = outcome {
        let authorities: Vec<_> = view
            .evidence()
            .evidence_assessments()
            .iter()
            .map(public_view::PublicEvidenceAssessmentView::authority)
            .collect();
        assert_eq!(
            authorities,
            vec![
                evidence::AffairsAuthority::ReviewedCommunitySummary,
                evidence::AffairsAuthority::OfficialBulletin,
            ]
        );
    }
}

#[test]
fn eight_groups_complete() {
    let assessments: Vec<_> = ALL_SUBJECTS
        .iter()
        .enumerate()
        .map(|(i, &subject)| {
            assessment(
                evidence::AffairsAuthority::OfficialBulletin,
                &format!("s{i}"),
                subject,
                100,
                100,
                None,
                None,
            )
        })
        .collect();
    let outcome = run_found(assessments);
    if let outcome::GetProcedureOutcome::Found { view, .. } = outcome {
        assert_eq!(view.evidence().evidence_assessments().len(), 8);
        assert_eq!(
            view.evidence().projection(),
            public_view::ProjectionMetadata::Complete
        );
    }
}

#[test]
fn nine_groups_truncated_with_one_omitted() {
    let mut assessments: Vec<_> = ALL_SUBJECTS
        .iter()
        .enumerate()
        .map(|(i, &subject)| {
            assessment(
                evidence::AffairsAuthority::OfficialBulletin,
                &format!("s{i}"),
                subject,
                100,
                100,
                None,
                None,
            )
        })
        .collect();
    // 9th group: lower tier (non-mandatory)
    assessments.push(assessment(
        evidence::AffairsAuthority::ReviewedCommunitySummary,
        "low1",
        evidence::AuthoritySubject::ProcedureTitle,
        100,
        100,
        None,
        None,
    ));
    let outcome = run_found(assessments);
    if let outcome::GetProcedureOutcome::Found { view, .. } = outcome {
        assert_eq!(view.evidence().evidence_assessments().len(), 8);
        assert_eq!(
            view.evidence().projection(),
            public_view::ProjectionMetadata::Truncated {
                omitted_count: 1,
                selection_rule_version: 2,
            }
        );
    }
}

#[test]
fn nine_raw_one_group_complete() {
    // 9 raw assessments that coalesce into 1 group
    let assessments: Vec<_> = (0..9).map(|_| one_group_assessment("s1")).collect();
    let outcome = run_found(assessments);
    if let outcome::GetProcedureOutcome::Found { view, .. } = outcome {
        assert_eq!(view.evidence().evidence_assessments().len(), 1);
        assert_eq!(
            view.evidence().projection(),
            public_view::ProjectionMetadata::Complete
        );
    }
}

#[test]
fn sixteen_groups_truncated_with_eight_omitted() {
    // 16 distinct groups: 8 mandatory (all 8 subjects at OfficialBulletin) + 8
    // non-mandatory (all 8 subjects at ReviewedCommunitySummary). 16 total > 8,
    // so truncated with omitted_count = 8.
    let mut assessments: Vec<_> = ALL_SUBJECTS
        .iter()
        .enumerate()
        .map(|(i, &subject)| {
            assessment(
                evidence::AffairsAuthority::OfficialBulletin,
                &format!("hi{i}"),
                subject,
                100,
                100,
                None,
                None,
            )
        })
        .collect();
    for (i, &subject) in ALL_SUBJECTS.iter().enumerate() {
        assessments.push(assessment(
            evidence::AffairsAuthority::ReviewedCommunitySummary,
            &format!("lo{i}"),
            subject,
            100,
            100,
            None,
            None,
        ));
    }
    let outcome = run_found(assessments);
    if let outcome::GetProcedureOutcome::Found { view, .. } = outcome {
        assert_eq!(view.evidence().evidence_assessments().len(), 8);
        assert_eq!(
            view.evidence().projection(),
            public_view::ProjectionMetadata::Truncated {
                omitted_count: 8,
                selection_rule_version: 2,
            }
        );
    }
}

#[test]
fn nine_mandatory_groups_overflow() {
    // 9 mandatory groups: 8 subjects at OfficialBulletin + 1 extra
    // OfficialBulletin with a different source on ProcedureTitle. All are
    // maximal tier → all mandatory → overflow.
    let mut assessments: Vec<_> = ALL_SUBJECTS
        .iter()
        .enumerate()
        .map(|(i, &subject)| {
            assessment(
                evidence::AffairsAuthority::OfficialBulletin,
                &format!("s{i}"),
                subject,
                100,
                100,
                None,
                None,
            )
        })
        .collect();
    assessments.push(assessment(
        evidence::AffairsAuthority::OfficialBulletin,
        "s8",
        evidence::AuthoritySubject::ProcedureTitle,
        100,
        100,
        None,
        None,
    ));
    let artifact = build_artifact(
        "proc:overflow9",
        50,
        150,
        evidence::EvidenceConflictState::NoKnownConflict,
        evidence::AuthorityComparison::Equivalent,
        None,
        100,
        200,
        assessments,
    );
    let mut repo = repository::InMemoryAffairsRepository::new();
    seed_current(&mut repo, artifact);
    let mut m60 = m60_fixture::M60FixtureAdapter::new("verifier:fixture", 1).unwrap();
    for i in 0..9 {
        m60.store(rev(&format!("s{i}"), 0, None, None));
    }
    let clock = clock::FixedClock::new(t(200));
    let service = service::AffairsGetService::new(&repo, &m60, &clock);
    let query = outcome::AffairsGetQuery::new(
        value::ProcedureId::parse("proc:overflow9").unwrap(),
        Some(t(200)),
    );
    let receipt = service.execute(&query).unwrap();
    if let outcome::GetProcedureOutcome::CannotVerify { reason, .. } = receipt.outcome() {
        assert!(matches!(
            reason,
            outcome::CannotVerifyReason::PublicEvidenceProjectionOverflow { mandatory_count: 9 }
        ));
    } else {
        panic!("expected overflow CannotVerify");
    }
}

#[test]
fn selection_rule_version_is_two() {
    let outcome = run_found(vec![one_group_assessment("s1")]);
    if let outcome::GetProcedureOutcome::Found { view, .. } = outcome {
        match view.evidence().projection() {
            public_view::ProjectionMetadata::Complete => {}
            public_view::ProjectionMetadata::Truncated {
                selection_rule_version,
                ..
            } => assert_eq!(selection_rule_version, 2),
        }
    }
}

#[test]
fn coalesce_uses_earliest_reviewed_and_verified() {
    // Two assessments in the same group: different reviewed_at / verified_at.
    // Representative uses the earliest of each.
    let a1 = assessment(
        evidence::AffairsAuthority::OfficialBulletin,
        "s1",
        evidence::AuthoritySubject::ProcedureTitle,
        200,
        200,
        None,
        None,
    );
    let a2 = assessment(
        evidence::AffairsAuthority::OfficialBulletin,
        "s1",
        evidence::AuthoritySubject::ProcedureTitle,
        100,
        150,
        None,
        None,
    );
    let outcome = run_found(vec![a1, a2]);
    if let outcome::GetProcedureOutcome::Found { view, .. } = outcome {
        let rep = &view.evidence().evidence_assessments()[0];
        assert_eq!(
            rep.reviewed_at(),
            OffsetDateTime::from_unix_timestamp(100).unwrap()
        );
        assert_eq!(
            rep.last_verified_at(),
            OffsetDateTime::from_unix_timestamp(150).unwrap()
        );
    }
}
