#![allow(clippy::unwrap_used)]

use std::sync::atomic::{AtomicU8, AtomicUsize, Ordering};

use quick_xml::events::{BytesStart, Event};
use quick_xml::reader::Reader;
use ustc_campus_agent_change_radar::{
    AcceptedObservation, BoardFeedPolicy, BoardId, BoardPolicy, ChangePublicationError,
    ChangePublicationRepositoryError, ChangePublicationService, ChangeRadarService,
    ChangeRejectionReason, ChangeReviewReceipt, InMemoryChangeRadarRepository,
    M60ChangePublicationOutcome, M60ChangePublicationPort, M60ChangePublicationPortError,
    M60VerifiedChangeEvidence, NormalizedFacts, ObservationOutcome, SemanticChangeCandidate,
    SemanticField, SemanticValue, render_atom,
};
use ustc_campus_agent_core::identity::UserId;
use ustc_campus_agent_core::source_registry::{
    SourceId, SourceReviewEvidenceId, SourceReviewerId, SourceUrl,
};
use ustc_campus_agent_core::source_revision::{
    EffectiveInterval, NormalizedSnapshotId, ParserIdentity, RawSnapshotId, RevisionSha256,
    RevisionTimestamp, SourceRevision, SourceRevisionHealth,
};

const MODE_CURRENT: u8 = 0;
const MODE_STALE: u8 = 1;
const MODE_CONFLICTING: u8 = 2;
const MODE_UNVERIFIED: u8 = 3;
const MODE_UNAVAILABLE: u8 = 4;
const MODE_CORRUPTED: u8 = 5;
const MODE_MISMATCH: u8 = 6;

#[derive(Default)]
struct AtomEntryShape {
    id: usize,
    title: usize,
    updated: usize,
    link: usize,
    summary: usize,
}

fn attribute(element: &BytesStart<'_>, name: &str) -> Option<String> {
    element
        .attributes()
        .with_checks(true)
        .map(|attribute| attribute.expect("valid XML attribute"))
        .find(|attribute| attribute.key.as_ref() == name)
        .map(|attribute| attribute.value.as_ref().to_owned())
}

fn assert_atom_1_0_document(xml: &str, expected_entries: usize) {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);
    let mut stack: Vec<String> = Vec::new();
    let mut feed_id = 0;
    let mut feed_title = 0;
    let mut feed_updated = 0;
    let mut feed_author = 0;
    let mut feed_author_name = 0;
    let mut feed_self_link = 0;
    let mut entry = None::<AtomEntryShape>;
    let mut entry_count = 0;

    loop {
        match reader.read_event().expect("well-formed Atom XML") {
            Event::Start(element) => {
                let name = element.name().as_ref().to_owned();
                let parent = stack.last().map(String::as_str);
                if stack.is_empty() {
                    assert_eq!(name, "feed");
                    assert_eq!(
                        attribute(&element, "xmlns").as_deref(),
                        Some("http://www.w3.org/2005/Atom")
                    );
                } else if parent == Some("feed") {
                    match name.as_str() {
                        "id" => feed_id += 1,
                        "title" => feed_title += 1,
                        "updated" => feed_updated += 1,
                        "author" => feed_author += 1,
                        "entry" => entry = Some(AtomEntryShape::default()),
                        _ => {}
                    }
                } else if parent == Some("author") && name == "name" && entry.is_none() {
                    feed_author_name += 1;
                } else if parent == Some("entry") {
                    let shape = entry.as_mut().expect("entry shape");
                    match name.as_str() {
                        "id" => shape.id += 1,
                        "title" => shape.title += 1,
                        "updated" => shape.updated += 1,
                        "link" => shape.link += 1,
                        "summary" => shape.summary += 1,
                        _ => {}
                    }
                }
                stack.push(name);
            }
            Event::Empty(element) => {
                let name = element.name();
                assert_eq!(name.as_ref(), "link", "only Atom links are empty");
                match stack.last().map(String::as_str) {
                    Some("feed") => {
                        if attribute(&element, "rel").as_deref() == Some("self")
                            && attribute(&element, "href").is_some()
                        {
                            feed_self_link += 1;
                        }
                    }
                    Some("entry") => {
                        assert!(attribute(&element, "href").is_some());
                        entry.as_mut().expect("entry shape").link += 1;
                    }
                    other => panic!("unexpected empty link parent: {other:?}"),
                }
            }
            Event::End(element) => {
                let name = element.name().as_ref().to_owned();
                assert_eq!(stack.pop().as_deref(), Some(name.as_str()));
                if name == "entry" {
                    let shape = entry.take().expect("entry shape");
                    assert_eq!(shape.id, 1);
                    assert_eq!(shape.title, 1);
                    assert_eq!(shape.updated, 1);
                    assert_eq!(shape.link, 1);
                    assert_eq!(shape.summary, 1);
                    entry_count += 1;
                }
            }
            Event::Eof => break,
            Event::Decl(_) | Event::Text(_) | Event::GeneralRef(_) => {}
            other => panic!("unexpected Atom event: {other:?}"),
        }
    }
    assert!(stack.is_empty());
    assert_eq!(feed_id, 1);
    assert_eq!(feed_title, 1);
    assert_eq!(feed_updated, 1);
    assert_eq!(feed_author, 1);
    assert_eq!(feed_author_name, 1);
    assert_eq!(feed_self_link, 1);
    assert_eq!(entry_count, expected_entries);
}

struct FixtureM60 {
    mode: AtomicU8,
    calls: AtomicUsize,
}

impl FixtureM60 {
    fn new(mode: u8) -> Self {
        Self {
            mode: AtomicU8::new(mode),
            calls: AtomicUsize::new(0),
        }
    }

    fn set_mode(&self, mode: u8) {
        self.mode.store(mode, Ordering::SeqCst);
    }

    fn calls(&self) -> usize {
        self.calls.load(Ordering::SeqCst)
    }
}

impl M60ChangePublicationPort for FixtureM60 {
    fn verify_publication(
        &self,
        old_revision: &SourceRevision,
        new_revision: &SourceRevision,
    ) -> Result<M60ChangePublicationOutcome, M60ChangePublicationPortError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        match self.mode.load(Ordering::SeqCst) {
            MODE_CURRENT => Ok(M60ChangePublicationOutcome::CurrentVerified(
                M60VerifiedChangeEvidence::for_revisions(old_revision, new_revision),
            )),
            MODE_STALE => Ok(M60ChangePublicationOutcome::SourceNotCurrent(
                SourceRevisionHealth::Stale,
            )),
            MODE_CONFLICTING => Ok(M60ChangePublicationOutcome::SourceNotCurrent(
                SourceRevisionHealth::Conflicting,
            )),
            MODE_UNVERIFIED => Ok(M60ChangePublicationOutcome::EvidenceUnverified),
            MODE_UNAVAILABLE => Err(M60ChangePublicationPortError::StoreUnavailable),
            MODE_CORRUPTED => Err(M60ChangePublicationPortError::StoreCorrupted),
            MODE_MISMATCH => {
                let foreign_facts = facts(&[("registration.deadline", "2027-01-01")]);
                let foreign = revision(
                    "source:demo:foreign",
                    4,
                    900,
                    &foreign_facts,
                    "https://example.com/foreign",
                );
                Ok(M60ChangePublicationOutcome::CurrentVerified(
                    M60VerifiedChangeEvidence::for_revisions(old_revision, &foreign),
                ))
            }
            _ => unreachable!("fixture mode"),
        }
    }
}

fn field(value: &str) -> SemanticField {
    SemanticField::parse(value).expect("field")
}

fn semantic_value(value: &str) -> SemanticValue {
    SemanticValue::parse(value).expect("value")
}

fn facts(values: &[(&str, &str)]) -> NormalizedFacts {
    NormalizedFacts::try_from_iter(
        values
            .iter()
            .map(|(name, value)| (field(name), semantic_value(value))),
    )
    .expect("facts")
}

fn revision(
    source_id: &str,
    number: u8,
    observed: i64,
    facts: &NormalizedFacts,
    source_url: &str,
) -> SourceRevision {
    SourceRevision::demo_reviewed(
        SourceId::parse(source_id).expect("source"),
        SourceUrl::parse(source_url).expect("url"),
        RawSnapshotId::parse(format!("raw:feed:{number}")).expect("raw id"),
        RevisionSha256::parse(format!(
            "sha256:{}",
            char::from(b'a' + number).to_string().repeat(64)
        ))
        .expect("raw digest"),
        NormalizedSnapshotId::parse(format!("normalized:feed:{number}")).expect("normalized id"),
        facts.sha256(),
        ParserIdentity::parse("parser:calendar:v1").expect("parser"),
        RevisionTimestamp::from_unix_seconds(observed),
        Some(RevisionTimestamp::from_unix_seconds(observed - 20)),
        EffectiveInterval::new(
            Some(RevisionTimestamp::from_unix_seconds(observed + 100)),
            None,
        )
        .expect("interval"),
        SourceReviewerId::parse("reviewer:demo").expect("source reviewer"),
        SourceReviewEvidenceId::parse(format!("evidence:feed:{number}")).expect("evidence"),
    )
}

fn observation(number: u8, observed: i64, values: &[(&str, &str)]) -> AcceptedObservation {
    let facts = facts(values);
    let revision = revision(
        "source:demo:calendar",
        number,
        observed,
        &facts,
        "https://example.com/calendar",
    );
    AcceptedObservation::new(revision, facts, SourceRevisionHealth::Current).expect("observation")
}

fn candidate_and_repository() -> (SemanticChangeCandidate, InMemoryChangeRadarRepository) {
    let policy = BoardPolicy::new(
        BoardId::parse("board:academic-calendar").expect("board"),
        SourceId::parse("source:demo:calendar").expect("source"),
        1,
        [
            field("registration.deadline"),
            field("registration.location"),
        ],
        "all_students",
    )
    .expect("policy");
    let mut service = ChangeRadarService::new(policy, InMemoryChangeRadarRepository::new());
    service
        .observe(observation(
            1,
            100,
            &[
                ("registration.deadline", "2026-09-01"),
                ("registration.location", "West & Main"),
            ],
        ))
        .expect("baseline");
    let outcome = service
        .observe(observation(
            2,
            200,
            &[
                ("registration.deadline", "2026-09-03"),
                ("registration.location", "East <Campus>"),
            ],
        ))
        .expect("candidate");
    let ObservationOutcome::SemanticChange(candidate) = outcome else {
        panic!("semantic candidate")
    };
    ((*candidate).clone(), service.into_repository())
}

fn reviewer(value: &str) -> UserId {
    UserId::parse(value).expect("reviewer")
}

fn feed_policy() -> BoardFeedPolicy {
    BoardFeedPolicy::new(
        BoardId::parse("board:academic-calendar").expect("board"),
        1,
        "Academic calendar changes",
        "USTC Campus Agent",
        "https://campus.example.test",
    )
    .expect("feed policy")
}

#[test]
fn approved_candidate_publishes_exactly_once_and_renders_deterministic_atom() {
    let (candidate, mut repository) = candidate_and_repository();
    let m60 = FixtureM60::new(MODE_CURRENT);
    let approval = ChangeReviewReceipt::approve(
        &candidate,
        reviewer("user:admin"),
        RevisionTimestamp::from_unix_seconds(250),
    )
    .expect("approval");
    let published_at = RevisionTimestamp::from_unix_seconds(300);
    let mut service = ChangePublicationService::new(&mut repository, &m60, feed_policy());
    service.record_review(approval.clone()).expect("review");
    let published = service
        .publish(candidate.event_id(), published_at)
        .expect("publication");
    let atom = service.atom_feed().expect("atom");
    assert_atom_1_0_document(&atom, 1);

    assert_eq!(published.review(), &approval);
    assert_eq!(published.candidate().changed_fields().len(), 2);
    assert!(
        published
            .stable_guid()
            .as_str()
            .contains(candidate.event_id().as_str())
    );
    assert!(atom.contains("<author>\n    <name>USTC Campus Agent</name>"));
    assert!(atom.contains("registration.deadline"));
    assert!(atom.contains("2026-09-01→2026-09-03"));
    assert!(atom.contains("West &amp; Main→East &lt;Campus&gt;"));
    assert!(atom.contains("source_url=https://example.com/calendar"));
    assert!(atom.contains("observed_at=1970-01-01T00:03:20Z"));
    assert!(atom.contains("effective_from=1970-01-01T00:05:00Z"));
    assert!(atom.contains("source_health=current"));
    assert!(atom.contains("old_provenance=DemoReviewed"));
    assert!(atom.contains("new_provenance=DemoReviewed"));
    assert!(atom.contains("old_source_reviewer=reviewer:demo"));
    assert!(atom.contains("old_source_review_evidence=evidence:feed:1"));
    assert!(atom.contains("new_source_reviewer=reviewer:demo"));
    assert!(atom.contains("new_source_review_evidence=evidence:feed:2"));
    assert!(atom.contains(candidate.old_revision().raw_sha256().as_str()));
    assert!(atom.contains(candidate.new_revision().raw_sha256().as_str()));
    assert!(atom.contains(published.evidence_set_digest().as_str()));

    let revised_presentation = BoardFeedPolicy::new(
        BoardId::parse("board:academic-calendar").expect("board"),
        1,
        "Current academic calendar feed",
        "Current USTC Publisher",
        "https://current-campus.example.test",
    )
    .expect("revised presentation");
    let revised_atom = render_atom(&revised_presentation, std::slice::from_ref(&published))
        .expect("canonical event remains projectable");
    assert_atom_1_0_document(&revised_atom, 1);
    assert!(revised_atom.contains("Current USTC Publisher"));
    assert!(revised_atom.contains("https://current-campus.example.test"));
    assert!(revised_atom.contains(published.stable_guid().as_str()));

    // Exact replay is resolved from the stored receipt before a new M60 read.
    m60.set_mode(MODE_STALE);
    let replay = service
        .publish(candidate.event_id(), published_at)
        .expect("idempotent replay");
    assert_eq!(replay, published);
    assert_eq!(m60.calls(), 1);
    assert_eq!(repository.review_count(), 1);
    assert_eq!(repository.publication_count(), 1);
}

#[test]
fn rejection_is_terminal_and_never_reaches_m60_or_feed() {
    let (candidate, mut repository) = candidate_and_repository();
    let m60 = FixtureM60::new(MODE_CURRENT);
    let rejection = ChangeReviewReceipt::reject(
        &candidate,
        reviewer("user:admin"),
        RevisionTimestamp::from_unix_seconds(250),
        ChangeRejectionReason::InsufficientEvidence,
    )
    .expect("rejection");
    let mut service = ChangePublicationService::new(&mut repository, &m60, feed_policy());
    service.record_review(rejection).expect("review");
    assert!(matches!(
        service.publish(
            candidate.event_id(),
            RevisionTimestamp::from_unix_seconds(300)
        ),
        Err(ChangePublicationError::CandidateRejected(
            ChangeRejectionReason::InsufficientEvidence
        ))
    ));
    assert_eq!(m60.calls(), 0);
    assert_eq!(repository.publication_count(), 0);
}

#[test]
fn transaction_current_m60_failures_are_typed_and_non_mutating() {
    for (mode, expected) in [
        (MODE_STALE, "stale"),
        (MODE_CONFLICTING, "conflicting"),
        (MODE_UNVERIFIED, "unverified"),
        (MODE_UNAVAILABLE, "unavailable"),
        (MODE_CORRUPTED, "corrupted"),
        (MODE_MISMATCH, "mismatch"),
    ] {
        let (candidate, mut repository) = candidate_and_repository();
        let m60 = FixtureM60::new(mode);
        let approval = ChangeReviewReceipt::approve(
            &candidate,
            reviewer("user:admin"),
            RevisionTimestamp::from_unix_seconds(250),
        )
        .expect("approval");
        let mut service = ChangePublicationService::new(&mut repository, &m60, feed_policy());
        service.record_review(approval).expect("review");
        let result = service.publish(
            candidate.event_id(),
            RevisionTimestamp::from_unix_seconds(300),
        );
        let matches_expected = matches!(
            (expected, result),
            (
                "stale",
                Err(ChangePublicationError::SourceNotCurrent(
                    SourceRevisionHealth::Stale
                )),
            ) | (
                "conflicting",
                Err(ChangePublicationError::SourceNotCurrent(
                    SourceRevisionHealth::Conflicting
                )),
            ) | (
                "unverified",
                Err(ChangePublicationError::M60EvidenceUnverified)
            ) | (
                "unavailable",
                Err(ChangePublicationError::M60StoreUnavailable)
            ) | ("corrupted", Err(ChangePublicationError::M60StoreCorrupted))
                | ("mismatch", Err(ChangePublicationError::M60EvidenceMismatch))
        );
        assert!(matches_expected, "unexpected M60 result for {expected}");
        assert_eq!(repository.publication_count(), 0);
    }
}

#[test]
fn review_and_publication_failures_leave_no_partial_state() {
    let (candidate, mut repository) = candidate_and_repository();
    let m60 = FixtureM60::new(MODE_CURRENT);
    let approval = ChangeReviewReceipt::approve(
        &candidate,
        reviewer("user:admin"),
        RevisionTimestamp::from_unix_seconds(250),
    )
    .expect("approval");

    repository.inject_next_review_failure();
    {
        let mut service = ChangePublicationService::new(&mut repository, &m60, feed_policy());
        assert!(matches!(
            service.record_review(approval.clone()),
            Err(ChangePublicationError::Repository(
                ChangePublicationRepositoryError::InjectedReviewFailure
            ))
        ));
    }
    assert_eq!(repository.review_count(), 0);

    {
        let mut service = ChangePublicationService::new(&mut repository, &m60, feed_policy());
        service.record_review(approval).expect("review retry");
    }
    repository.inject_next_publication_failure();
    {
        let mut service = ChangePublicationService::new(&mut repository, &m60, feed_policy());
        assert!(matches!(
            service.publish(
                candidate.event_id(),
                RevisionTimestamp::from_unix_seconds(300)
            ),
            Err(ChangePublicationError::Repository(
                ChangePublicationRepositoryError::InjectedPublicationFailure
            ))
        ));
    }
    assert_eq!(repository.review_count(), 1);
    assert_eq!(repository.publication_count(), 0);
}

#[test]
fn out_of_range_timestamps_fail_before_review_or_publication_mutation() {
    let (candidate, mut repository) = candidate_and_repository();
    let out_of_range = RevisionTimestamp::from_unix_seconds(i64::MAX);
    assert!(matches!(
        ChangeReviewReceipt::approve(&candidate, reviewer("user:admin"), out_of_range),
        Err(ChangePublicationError::TimestampOutOfRange)
    ));
    assert_eq!(repository.review_count(), 0);

    let approval = ChangeReviewReceipt::approve(
        &candidate,
        reviewer("user:admin"),
        RevisionTimestamp::from_unix_seconds(250),
    )
    .expect("approval");
    let m60 = FixtureM60::new(MODE_CURRENT);
    let mut service = ChangePublicationService::new(&mut repository, &m60, feed_policy());
    service.record_review(approval).expect("review");
    assert!(matches!(
        service.publish(candidate.event_id(), out_of_range),
        Err(ChangePublicationError::TimestampOutOfRange)
    ));
    assert!(matches!(
        service.publish(
            candidate.event_id(),
            RevisionTimestamp::from_unix_seconds(i64::MIN)
        ),
        Err(ChangePublicationError::TimestampOutOfRange)
    ));
    assert_eq!(m60.calls(), 0);
    assert_eq!(repository.review_count(), 1);
    assert_eq!(repository.publication_count(), 0);
}

#[test]
fn chronology_policy_and_replay_conflicts_fail_closed() {
    let (candidate, mut repository) = candidate_and_repository();
    assert!(matches!(
        ChangeReviewReceipt::approve(
            &candidate,
            reviewer("user:admin"),
            RevisionTimestamp::from_unix_seconds(199)
        ),
        Err(ChangePublicationError::ReviewBeforeObservation)
    ));

    let approval = ChangeReviewReceipt::approve(
        &candidate,
        reviewer("user:admin"),
        RevisionTimestamp::from_unix_seconds(250),
    )
    .expect("approval");
    let m60 = FixtureM60::new(MODE_CURRENT);
    {
        let wrong_policy = BoardFeedPolicy::new(
            BoardId::parse("board:other").expect("board"),
            1,
            "Other changes",
            "USTC Campus Agent",
            "https://campus.example.test",
        )
        .expect("policy");
        let mut service = ChangePublicationService::new(&mut repository, &m60, wrong_policy);
        service.record_review(approval.clone()).expect("review");
        assert!(matches!(
            service.publish(
                candidate.event_id(),
                RevisionTimestamp::from_unix_seconds(300)
            ),
            Err(ChangePublicationError::FeedPolicyMismatch)
        ));
    }

    let mut service = ChangePublicationService::new(&mut repository, &m60, feed_policy());
    service.record_review(approval).expect("idempotent review");
    assert!(matches!(
        service.publish(
            candidate.event_id(),
            RevisionTimestamp::from_unix_seconds(249)
        ),
        Err(ChangePublicationError::PublishBeforeReview)
    ));
    service
        .publish(
            candidate.event_id(),
            RevisionTimestamp::from_unix_seconds(300),
        )
        .expect("publication");
    assert!(matches!(
        service.publish(
            candidate.event_id(),
            RevisionTimestamp::from_unix_seconds(301)
        ),
        Err(ChangePublicationError::PublicationReplayConflict)
    ));
    drop(service);

    let changed_author = BoardFeedPolicy::new(
        BoardId::parse("board:academic-calendar").expect("board"),
        1,
        "Academic calendar changes",
        "Different Publisher",
        "https://campus.example.test",
    )
    .expect("changed author policy");
    let mut service = ChangePublicationService::new(&mut repository, &m60, changed_author);
    assert!(matches!(
        service.publish(
            candidate.event_id(),
            RevisionTimestamp::from_unix_seconds(300)
        ),
        Err(ChangePublicationError::PublicationReplayConflict)
    ));
}
