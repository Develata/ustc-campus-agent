//! Deterministic public-evidence projection (M71-v8n §5). Algorithm order:
//! coalesce by exact `(authority, source_id, subject)` key → mandatory groups
//! (group-level, three clauses, no conflict-peer clause) → overflow check on
//! GROUP count → 8-slot selection over groups → reference remap.
//! `selection_rule_version = 2`. `mandatory_count` and `omitted_count` count
//! GROUPS, not raw assessments.
//!
//! Conflict-before-projection is enforced by the service before this module is
//! reached: an unresolved/incomparable material conflict returns the top-level
//! `Conflict` outcome and no projection is constructed.

use std::collections::{BTreeMap, BTreeSet};

use crate::artifact::Prerequisite;
use crate::evidence::{
    AffairsAuthority, AffairsEvidenceAssessment, AuthoritySubject, ProcedureEvidenceContext,
};
use crate::public_view::{
    ProjectionMetadata, PublicEvidenceAssessmentView, PublicEvidenceView, SELECTION_RULE_VERSION,
};
use crate::value::SourceId;

pub(crate) type AssessmentIndex = u8;
pub(crate) type GroupIndex = u8;

/// Coalescing key: the exact `(authority, source_id, subject)` triple. `Ord`
/// is tier-then-source-then-subject ascending, which fixes the coalesce
/// emission order and thus `GroupIndex`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct EvidenceGroupKey {
    pub(crate) authority: AffairsAuthority,
    pub(crate) source_id: SourceId,
    pub(crate) subject: AuthoritySubject,
}

/// One coalesced evidence group (internal; not a public wire type). Carries
/// the group key, ascending member indices, the public representative, and the
/// earliest `reviewed_at`/`last_verified_at` across members.
#[derive(Debug, Clone)]
pub(crate) struct EvidenceGroup {
    pub(crate) key: EvidenceGroupKey,
    pub(crate) member_indices: Vec<AssessmentIndex>,
    pub(crate) representative: PublicEvidenceAssessmentView,
    pub(crate) reviewed_at: time::OffsetDateTime,
    pub(crate) last_verified_at: time::OffsetDateTime,
}

impl EvidenceGroup {
    /// Lowest canonical member index in the group (unique per group; the
    /// tiebreak of last resort in the selection sort).
    pub(crate) fn lowest_member_index(&self) -> AssessmentIndex {
        self.member_indices[0]
    }
}

/// Coalesces canonical assessments by exact `(authority, source_id, subject)`
/// key. Each group representative uses the earliest `reviewed_at` and
/// `last_verified_at` across members. Groups are returned in ascending
/// `EvidenceGroupKey` order, fixing `GroupIndex`.
pub(crate) fn coalesce_by_key(assessments: &[AffairsEvidenceAssessment]) -> Vec<EvidenceGroup> {
    let mut buckets: BTreeMap<EvidenceGroupKey, Vec<AssessmentIndex>> = BTreeMap::new();
    for (index, assessment) in assessments.iter().enumerate() {
        let key = EvidenceGroupKey {
            authority: assessment.authority(),
            source_id: assessment.source_id().clone(),
            subject: assessment.subject(),
        };
        buckets
            .entry(key)
            .or_default()
            .push(AssessmentIndex::try_from(index).expect("at most 16 assessments"));
    }

    let mut groups = Vec::with_capacity(buckets.len());
    for (key, mut member_indices) in buckets {
        member_indices.sort_unstable();
        let earliest_reviewed = member_indices
            .iter()
            .map(|&i| assessments[usize::from(i)].reviewed_at())
            .min()
            .expect("non-empty group");
        let earliest_verified = member_indices
            .iter()
            .map(|&i| assessments[usize::from(i)].last_verified_at())
            .min()
            .expect("non-empty group");
        let representative = PublicEvidenceAssessmentView::new(
            key.authority,
            key.subject,
            key.source_id.clone(),
            earliest_reviewed,
            earliest_verified,
        );
        groups.push(EvidenceGroup {
            key,
            member_indices,
            representative,
            reviewed_at: earliest_reviewed,
            last_verified_at: earliest_verified,
        });
    }
    groups
}

/// Builds the canonical-index → group-index map (deterministic).
pub(crate) fn assessment_to_group(
    groups: &[EvidenceGroup],
) -> BTreeMap<AssessmentIndex, GroupIndex> {
    let mut map = BTreeMap::new();
    for (g, group) in groups.iter().enumerate() {
        for &member in &group.member_indices {
            map.insert(member, GroupIndex::try_from(g).expect("at most 16 groups"));
        }
    }
    map
}

/// Maximal authority tier present across the evidence assessments.
fn maximal_tier_of(assessments: &[AffairsEvidenceAssessment]) -> u8 {
    assessments
        .iter()
        .map(|a| a.authority().tier())
        .max()
        .unwrap_or(0)
}

/// Whether canonical assessment `i` is material to `valid_interval` derivation:
/// it is in the maximal-tier set and supplies at least one effective bound.
fn is_material_to_valid_interval(
    i: AssessmentIndex,
    assessments: &[AffairsEvidenceAssessment],
    maximal_tier: u8,
) -> bool {
    let a = &assessments[usize::from(i)];
    a.authority().tier() == maximal_tier
        && (a.effective_from().is_some() || a.effective_to().is_some())
}

/// Computes the mandatory group set (group-level, three clauses; no
/// conflict-peer clause). Returned in ascending `GroupIndex` order.
pub(crate) fn mandatory_groups(
    groups: &[EvidenceGroup],
    assessments: &[AffairsEvidenceAssessment],
    prerequisites: &[Prerequisite],
    a2g: &BTreeMap<AssessmentIndex, GroupIndex>,
) -> BTreeSet<GroupIndex> {
    let maximal_tier = maximal_tier_of(assessments);
    let referenced: BTreeSet<GroupIndex> = prerequisites
        .iter()
        .filter_map(|p| p.m60_revision_ref())
        .filter_map(|rev| assessments.iter().position(|a| a.revision_ref() == rev))
        .filter_map(|i| {
            a2g.get(&(AssessmentIndex::try_from(i).expect("<=16")))
                .copied()
        })
        .collect();

    let mut mandatory = BTreeSet::new();
    for (g, group) in groups.iter().enumerate() {
        let g = GroupIndex::try_from(g).expect("at most 16 groups");
        let is_maximal = group.key.authority.tier() == maximal_tier;
        let is_material = group
            .member_indices
            .iter()
            .any(|&i| is_material_to_valid_interval(i, assessments, maximal_tier));
        let is_referenced = referenced.contains(&g);
        if is_maximal || is_material || is_referenced {
            mandatory.insert(g);
        }
    }
    mandatory
}

/// Selection sort comparator (total, deterministic). Ascending lexicographic
/// over: `authority_tier` DESC, `source_id` ASC, `subject` ASC, `reviewed_at`
/// ASC, `last_verified_at` ASC, `lowest_member_index` ASC. Total because
/// `lowest_member_index` is unique per group.
fn compare_for_fill(a: &EvidenceGroup, b: &EvidenceGroup) -> std::cmp::Ordering {
    use std::cmp::Ordering;
    let mut ord = b.key.authority.tier().cmp(&a.key.authority.tier());
    if ord == Ordering::Equal {
        ord = a.key.source_id.as_str().cmp(b.key.source_id.as_str());
    }
    if ord == Ordering::Equal {
        ord = a.key.subject.cmp(&b.key.subject);
    }
    if ord == Ordering::Equal {
        ord = a.reviewed_at.cmp(&b.reviewed_at);
    }
    if ord == Ordering::Equal {
        ord = a.last_verified_at.cmp(&b.last_verified_at);
    }
    if ord == Ordering::Equal {
        ord = a.lowest_member_index().cmp(&b.lowest_member_index());
    }
    ord
}

/// Outcome of the projection.
pub(crate) enum ProjectionOutcome {
    Overflow {
        mandatory_count: u8,
    },
    Projected {
        evidence: PublicEvidenceView,
        prerequisites: Vec<crate::public_view::PublicPrerequisiteView>,
    },
}

/// Runs the full projection on a non-conflict evidence context. Algorithm:
/// coalesce → mandatory groups → overflow check on group count → 8-slot
/// selection → reference remap.
pub(crate) fn project_public_evidence(
    ctx: &ProcedureEvidenceContext,
    prerequisites: &[Prerequisite],
) -> ProjectionOutcome {
    let assessments = ctx.evidence_assessments();
    let groups = coalesce_by_key(assessments);
    let a2g = assessment_to_group(&groups);
    let mandatory = mandatory_groups(&groups, assessments, prerequisites, &a2g);

    if mandatory.len() > 8 {
        return ProjectionOutcome::Overflow {
            mandatory_count: u8::try_from(mandatory.len()).expect("<=16 groups"),
        };
    }

    // `mandatory` is a BTreeSet over GroupIndex, so this prefix is already in
    // the contract's required ascending GroupIndex order. Only non-mandatory
    // fill groups use `compare_for_fill` (M71-v8n §5.6).
    let mut selected: Vec<GroupIndex> = mandatory.iter().copied().collect();
    let mut non_mandatory: Vec<GroupIndex> = (0..u8::try_from(groups.len()).expect("<=16"))
        .filter(|g| !mandatory.contains(g))
        .collect();
    non_mandatory
        .sort_by(|&a, &b| compare_for_fill(&groups[usize::from(a)], &groups[usize::from(b)]));
    selected.extend(non_mandatory);
    selected.truncate(8);

    let selected_set: BTreeSet<GroupIndex> = selected.iter().copied().collect();
    let omitted_count = u8::try_from(groups.len() - selected.len()).expect("<=16 groups");
    let projection = if groups.len() <= 8 {
        ProjectionMetadata::Complete
    } else {
        ProjectionMetadata::Truncated {
            omitted_count,
            selection_rule_version: SELECTION_RULE_VERSION,
        }
    };

    let evidence_assessments: Vec<PublicEvidenceAssessmentView> = selected
        .iter()
        .map(|&g| groups[usize::from(g)].representative.clone())
        .collect();

    let prereq_views = prerequisites
        .iter()
        .map(|prereq| {
            let source_subject = prereq
                .m60_revision_ref()
                .and_then(|rev| assessments.iter().position(|a| a.revision_ref() == rev))
                .and_then(|i| {
                    a2g.get(&(AssessmentIndex::try_from(i).expect("<=16")))
                        .copied()
                })
                .filter(|g| selected_set.contains(g))
                .map(|g| groups[usize::from(g)].key.subject);
            crate::public_view::PublicPrerequisiteView::new(
                prereq.condition().as_str().to_owned(),
                source_subject,
            )
        })
        .collect();

    let evidence = PublicEvidenceView::new(
        ctx.valid_interval().clone(),
        ctx.observed_at(),
        ctx.known_at(),
        ctx.reviewed_at(),
        ctx.last_verified_at(),
        evidence_assessments,
        projection,
    );
    ProjectionOutcome::Projected {
        evidence,
        prerequisites: prereq_views,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::evidence::{
        AuthorityComparison, AuthorityDerivation, EvidenceConflictState, M60RevisionRef, Sha256,
        UncertaintyState, ValidityHorizon,
    };
    use crate::value::{ActorRef, SourceId};
    use time::OffsetDateTime;

    fn t(secs: i64) -> OffsetDateTime {
        OffsetDateTime::from_unix_timestamp(secs).unwrap()
    }

    fn digest(hex_first: char) -> Sha256 {
        let s: String = std::iter::repeat_n(hex_first, 64).collect();
        Sha256::new(format!("sha256:{s}")).unwrap()
    }

    fn rev(source: &str, idx: usize, from: Option<i64>, to: Option<i64>) -> M60RevisionRef {
        M60RevisionRef::new(
            SourceId::parse(source).unwrap(),
            format!("rev:{source}:{idx}"),
            t(0),
            None,
            from.map(t),
            to.map(t),
            digest('0'),
            digest('1'),
        )
        .unwrap()
    }

    fn assessment(
        authority: AffairsAuthority,
        source: &str,
        subject: AuthoritySubject,
        reviewed: i64,
        verified: i64,
        from: Option<i64>,
        to: Option<i64>,
    ) -> AffairsEvidenceAssessment {
        let r = rev(source, 0, from, to);
        let a = crate::evidence::AffairsAuthorityAssessment::new(
            authority,
            subject,
            AuthorityDerivation::Direct,
            t(0),
            ActorRef::parse("actor:fixture").unwrap(),
        );
        AffairsEvidenceAssessment::new(r, a, t(reviewed), t(verified))
    }

    fn ctx_from(assessments: Vec<AffairsEvidenceAssessment>) -> ProcedureEvidenceContext {
        ProcedureEvidenceContext::new(
            ValidityHorizon::Unknown,
            t(0),
            t(0),
            t(0),
            t(0),
            assessments,
            EvidenceConflictState::NoKnownConflict,
            AuthorityComparison::Equivalent,
            UncertaintyState::None,
            None,
            Vec::new(),
        )
        .unwrap()
    }

    #[test]
    fn coalesce_groups_equivalent_assessments() {
        let a0 = assessment(
            AffairsAuthority::OfficialBulletin,
            "s1",
            AuthoritySubject::ProcedureTitle,
            100,
            110,
            None,
            None,
        );
        let a1 = assessment(
            AffairsAuthority::OfficialBulletin,
            "s1",
            AuthoritySubject::ProcedureTitle,
            90,
            120,
            None,
            None,
        );
        let groups = coalesce_by_key(&[a0, a1]);
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].member_indices.len(), 2);
        assert_eq!(groups[0].reviewed_at, t(90));
        assert_eq!(groups[0].last_verified_at, t(110));
    }

    #[test]
    fn nine_raw_one_group_no_false_overflow() {
        let assessments: Vec<_> = (0..9)
            .map(|i| {
                assessment(
                    AffairsAuthority::OfficialBulletin,
                    "s1",
                    AuthoritySubject::ProcedureTitle,
                    100 + i,
                    110 + i,
                    None,
                    None,
                )
            })
            .collect();
        let ctx = ctx_from(assessments);
        let prereqs: Vec<Prerequisite> = Vec::new();
        match project_public_evidence(&ctx, &prereqs) {
            ProjectionOutcome::Projected { evidence, .. } => {
                assert_eq!(evidence.evidence_assessments().len(), 1);
                assert_eq!(evidence.projection(), ProjectionMetadata::Complete);
            }
            ProjectionOutcome::Overflow { .. } => panic!("9 raw / 1 group must not overflow"),
        }
    }

    #[test]
    fn nine_distinct_mandatory_groups_overflow() {
        let subjects = [
            AuthoritySubject::ProcedureTitle,
            AuthoritySubject::ProcedureSteps,
            AuthoritySubject::ProcedureDeadlines,
            AuthoritySubject::ProcedureEffectiveInterval,
            AuthoritySubject::ProcedureEntryPoints,
            AuthoritySubject::ProcedureContacts,
            AuthoritySubject::ProcedurePrerequisites,
            AuthoritySubject::ProcedureEvidence,
        ];
        let mut assessments = Vec::new();
        for (i, &subject) in subjects.iter().enumerate() {
            assessments.push(assessment(
                AffairsAuthority::OfficialBulletin,
                &format!("s{i}"),
                subject,
                100,
                110,
                None,
                None,
            ));
        }
        assessments.push(assessment(
            AffairsAuthority::OfficialBulletin,
            "s9",
            AuthoritySubject::ProcedureTitle,
            100,
            110,
            None,
            None,
        ));
        let ctx = ctx_from(assessments);
        match project_public_evidence(&ctx, &[]) {
            ProjectionOutcome::Overflow { mandatory_count } => assert_eq!(mandatory_count, 9),
            ProjectionOutcome::Projected { .. } => {
                panic!("9 distinct mandatory groups must overflow")
            }
        }
    }

    #[test]
    fn truncated_at_nine_total_groups() {
        let subjects = [
            AuthoritySubject::ProcedureTitle,
            AuthoritySubject::ProcedureSteps,
            AuthoritySubject::ProcedureDeadlines,
            AuthoritySubject::ProcedureEffectiveInterval,
            AuthoritySubject::ProcedureEntryPoints,
            AuthoritySubject::ProcedureContacts,
            AuthoritySubject::ProcedurePrerequisites,
            AuthoritySubject::ProcedureEvidence,
        ];
        let mut assessments: Vec<_> = subjects
            .iter()
            .enumerate()
            .map(|(i, &subject)| {
                assessment(
                    AffairsAuthority::OfficialBulletin,
                    &format!("s{i}"),
                    subject,
                    100,
                    110,
                    None,
                    None,
                )
            })
            .collect();
        let non_mandatory = assessment(
            AffairsAuthority::ReviewedCommunitySummary,
            "low1",
            AuthoritySubject::ProcedureTitle,
            100,
            110,
            None,
            None,
        );
        assessments.push(non_mandatory);
        let ctx = ctx_from(assessments);
        match project_public_evidence(&ctx, &[]) {
            ProjectionOutcome::Projected { evidence, .. } => {
                assert_eq!(evidence.evidence_assessments().len(), 8);
                assert_eq!(
                    evidence.projection(),
                    ProjectionMetadata::Truncated {
                        omitted_count: 1,
                        selection_rule_version: 2,
                    }
                );
            }
            ProjectionOutcome::Overflow { .. } => {
                panic!("8 mandatory + 1 non must truncate, not overflow")
            }
        }
    }
}
