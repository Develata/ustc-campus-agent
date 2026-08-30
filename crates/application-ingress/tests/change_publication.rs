#![allow(clippy::unwrap_used)]

mod common;

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use ustc_campus_agent_application_ingress::{
    ChangePublicationApplicationError, ChangePublicationApplicationPort, ChangePublicationCommand,
    ChangePublicationEvidenceError, ChangePublicationOutcome, M10ChangePublicationService,
    change_publication_payload_digest,
};
use ustc_campus_agent_change_radar::{
    AcceptedObservation, BoardId, BoardPolicy, ChangeEventId, ChangeRadarService,
    ChangeReviewReceipt, ChangeReviewReceiptId, InMemoryChangeRadarRepository, NormalizedFacts,
    ObservationOutcome, PublishedChangeEvent, SemanticField, SemanticValue,
};
use ustc_campus_agent_core::control_evidence::{
    ControlEvidenceAppendOutcome, ControlEvidenceAppendPort, ControlEvidenceJournalError,
    ControlEvidenceKey, ControlEvidenceReadPort, PlatformControlEvent,
};
use ustc_campus_agent_core::identity::{CommandId, UserId};
use ustc_campus_agent_core::request_context::{
    ActorReference, CapabilityDisposition, ClientProvenance, M00AdmittedActor, OperationSnapshot,
    PayloadDigest, PublicScope,
};
use ustc_campus_agent_core::source_registry::{
    SourceId, SourceReviewEvidenceId, SourceReviewerId, SourceUrl,
};
use ustc_campus_agent_core::source_revision::{
    EffectiveInterval, NormalizedSnapshotId, ParserIdentity, RawSnapshotId, RevisionSha256,
    RevisionTimestamp, SourceRevision, SourceRevisionHealth,
};

use common::{Descriptor, FakePorts};

#[derive(Clone)]
struct OrderedEvidence {
    events: Arc<Mutex<BTreeMap<ControlEvidenceKey, PlatformControlEvent>>>,
    order: Arc<Mutex<Vec<&'static str>>>,
    failure: Option<ControlEvidenceJournalError>,
}

impl OrderedEvidence {
    fn successful(order: Arc<Mutex<Vec<&'static str>>>) -> Self {
        Self {
            events: Arc::new(Mutex::new(BTreeMap::new())),
            order,
            failure: None,
        }
    }

    fn failing(order: Arc<Mutex<Vec<&'static str>>>) -> Self {
        Self {
            events: Arc::new(Mutex::new(BTreeMap::new())),
            order,
            failure: Some(ControlEvidenceJournalError::Unavailable),
        }
    }
}

impl ControlEvidenceReadPort for OrderedEvidence {
    fn load_control_event(
        &mut self,
        key: &ControlEvidenceKey,
    ) -> Result<Option<PlatformControlEvent>, ControlEvidenceJournalError> {
        Ok(self.events.lock().unwrap().get(key).cloned())
    }
}

impl ControlEvidenceAppendPort for OrderedEvidence {
    fn append_once(
        &mut self,
        event: &PlatformControlEvent,
    ) -> Result<ControlEvidenceAppendOutcome, ControlEvidenceJournalError> {
        self.order.lock().unwrap().push("evidence");
        if let Some(error) = self.failure {
            return Err(error);
        }
        let mut events = self.events.lock().unwrap();
        match events.get(&event.key()) {
            Some(existing) if existing == event => Ok(ControlEvidenceAppendOutcome::AlreadySame),
            Some(_) => Ok(ControlEvidenceAppendOutcome::Conflict),
            None => {
                events.insert(event.key(), event.clone());
                Ok(ControlEvidenceAppendOutcome::Appended)
            }
        }
    }
}

struct DenyingPublication {
    order: Arc<Mutex<Vec<&'static str>>>,
}

impl ChangePublicationApplicationPort for DenyingPublication {
    fn publish(
        &mut self,
        _command_id: &CommandId,
        _actor: &M00AdmittedActor,
        _event_id: &ChangeEventId,
        _review_receipt_id: &str,
        _reviewed_at: RevisionTimestamp,
        _published_at: RevisionTimestamp,
    ) -> Result<PublishedChangeEvent, ChangePublicationApplicationError> {
        self.order.lock().unwrap().push("publication");
        Err(ChangePublicationApplicationError::Denied)
    }
}

fn semantic_field(value: &str) -> SemanticField {
    SemanticField::parse(value).unwrap()
}

fn normalized_facts(value: &str) -> NormalizedFacts {
    NormalizedFacts::try_from_iter([(
        semantic_field("registration.deadline"),
        SemanticValue::parse(value).unwrap(),
    )])
    .unwrap()
}

fn observation(number: u8, observed_at: i64, value: &str) -> AcceptedObservation {
    let facts = normalized_facts(value);
    let revision = SourceRevision::demo_reviewed(
        SourceId::parse("source:fixture:calendar").unwrap(),
        SourceUrl::parse("https://example.test/calendar").unwrap(),
        RawSnapshotId::parse(format!("raw:fixture:{number}")).unwrap(),
        RevisionSha256::parse(format!(
            "sha256:{}",
            char::from(b'a' + number).to_string().repeat(64)
        ))
        .unwrap(),
        NormalizedSnapshotId::parse(format!("normalized:fixture:{number}")).unwrap(),
        facts.sha256(),
        ParserIdentity::parse("parser:fixture:v1").unwrap(),
        RevisionTimestamp::from_unix_seconds(observed_at),
        Some(RevisionTimestamp::from_unix_seconds(observed_at - 10)),
        EffectiveInterval::new(
            Some(RevisionTimestamp::from_unix_seconds(observed_at + 100)),
            None,
        )
        .unwrap(),
        SourceReviewerId::parse("reviewer:fixture").unwrap(),
        SourceReviewEvidenceId::parse(format!("evidence:fixture:{number}")).unwrap(),
    );
    AcceptedObservation::new(revision, facts, SourceRevisionHealth::Current).unwrap()
}

fn reviewed_identity() -> (ChangeEventId, ChangeReviewReceiptId, RevisionTimestamp) {
    let policy = BoardPolicy::new(
        BoardId::parse("board:fixture:calendar").unwrap(),
        SourceId::parse("source:fixture:calendar").unwrap(),
        1,
        [semantic_field("registration.deadline")],
        "all_students",
    )
    .unwrap();
    let mut radar = ChangeRadarService::new(policy, InMemoryChangeRadarRepository::new());
    radar.observe(observation(1, 100, "2026-09-01")).unwrap();
    let ObservationOutcome::SemanticChange(candidate) =
        radar.observe(observation(2, 200, "2026-09-03")).unwrap()
    else {
        panic!("semantic change")
    };
    let reviewed_at = RevisionTimestamp::from_unix_seconds(250);
    let review = ChangeReviewReceipt::approve(
        &candidate,
        UserId::parse("user:fixture").unwrap(),
        reviewed_at,
    )
    .unwrap();
    (
        candidate.event_id().clone(),
        review.receipt_id().clone(),
        reviewed_at,
    )
}

fn command_with_digest(
    actor_reference: ActorReference,
    digest_override: Option<PayloadDigest>,
) -> ChangePublicationCommand {
    let (event_id, review_receipt_id, reviewed_at) = reviewed_identity();
    let published_at = RevisionTimestamp::from_unix_seconds(300);
    let digest = digest_override.unwrap_or_else(|| {
        change_publication_payload_digest(
            &event_id,
            review_receipt_id.as_str(),
            reviewed_at,
            published_at,
        )
    });
    ChangePublicationCommand::new(
        common::request_id(),
        actor_reference,
        common::correlation_id(),
        None,
        Some(common::idem_key()),
        ClientProvenance::new("build:fixture", "linux", "m10:change-publication-v1").unwrap(),
        digest,
        event_id,
        &review_receipt_id,
        reviewed_at,
        published_at,
    )
}

fn publication_ports(authenticated: bool) -> FakePorts {
    let descriptor: OperationSnapshot = Arc::new(Descriptor::change_publication());
    let mut ports = if authenticated {
        FakePorts::authenticated_admitted("session:fixture")
    } else {
        FakePorts::public_admitted()
    };
    ports.descriptor = Ok(Arc::clone(&descriptor));
    ports.staged = descriptor;
    ports
}

#[test]
fn evidence_append_precedes_direct_m70_publication_port() {
    let order = Arc::new(Mutex::new(Vec::new()));
    let mut evidence = OrderedEvidence::successful(Arc::clone(&order));
    let mut publication = DenyingPublication {
        order: Arc::clone(&order),
    };
    let mut service = M10ChangePublicationService::new(&mut publication, &mut evidence);
    let mut ports = publication_ports(true);
    let request = command_with_digest(
        ActorReference::Authenticated {
            session_id: common::session("session:fixture"),
        },
        None,
    );

    let outcome = service.submit(&request, &mut ports);

    assert_eq!(
        outcome,
        ChangePublicationOutcome::PublicationRejected(ChangePublicationApplicationError::Denied)
    );
    assert_eq!(*order.lock().unwrap(), vec!["evidence", "publication"]);
    assert_eq!(evidence.events.lock().unwrap().len(), 1);
}

#[test]
fn public_actor_is_rejected_before_evidence_or_publication() {
    let order = Arc::new(Mutex::new(Vec::new()));
    let mut evidence = OrderedEvidence::successful(Arc::clone(&order));
    let mut publication = DenyingPublication {
        order: Arc::clone(&order),
    };
    let mut service = M10ChangePublicationService::new(&mut publication, &mut evidence);
    let mut ports = publication_ports(false);
    let request = command_with_digest(ActorReference::Anonymous { scope: PublicScope }, None);

    let outcome = service.submit(&request, &mut ports);

    assert!(matches!(outcome, ChangePublicationOutcome::Rejected(_)));
    assert!(order.lock().unwrap().is_empty());
    assert!(evidence.events.lock().unwrap().is_empty());
}

#[test]
fn disabled_or_revoked_capability_is_rejected_before_evidence_or_publication() {
    for capability in [
        CapabilityDisposition::Disabled,
        CapabilityDisposition::Revoked,
    ] {
        let order = Arc::new(Mutex::new(Vec::new()));
        let mut evidence = OrderedEvidence::successful(Arc::clone(&order));
        let mut publication = DenyingPublication {
            order: Arc::clone(&order),
        };
        let mut service = M10ChangePublicationService::new(&mut publication, &mut evidence);
        let mut ports = publication_ports(true);
        ports.capability = Ok(capability);
        let request = command_with_digest(
            ActorReference::Authenticated {
                session_id: common::session("session:fixture"),
            },
            None,
        );

        let outcome = service.submit(&request, &mut ports);

        assert!(matches!(outcome, ChangePublicationOutcome::Rejected(_)));
        assert!(order.lock().unwrap().is_empty());
    }
}

#[test]
fn missing_session_is_rejected_before_evidence_or_publication() {
    let order = Arc::new(Mutex::new(Vec::new()));
    let mut evidence = OrderedEvidence::successful(Arc::clone(&order));
    let mut publication = DenyingPublication {
        order: Arc::clone(&order),
    };
    let mut service = M10ChangePublicationService::new(&mut publication, &mut evidence);
    let mut ports = publication_ports(true);
    ports.loaded_session = Ok(None);
    let request = command_with_digest(
        ActorReference::Authenticated {
            session_id: common::session("session:fixture"),
        },
        None,
    );

    let outcome = service.submit(&request, &mut ports);

    assert!(matches!(outcome, ChangePublicationOutcome::Rejected(_)));
    assert!(order.lock().unwrap().is_empty());
}

#[test]
fn evidence_failure_is_terminal_before_publication_port() {
    let order = Arc::new(Mutex::new(Vec::new()));
    let mut evidence = OrderedEvidence::failing(Arc::clone(&order));
    let mut publication = DenyingPublication {
        order: Arc::clone(&order),
    };
    let mut service = M10ChangePublicationService::new(&mut publication, &mut evidence);
    let mut ports = publication_ports(true);
    let request = command_with_digest(
        ActorReference::Authenticated {
            session_id: common::session("session:fixture"),
        },
        None,
    );

    let outcome = service.submit(&request, &mut ports);

    assert_eq!(
        outcome,
        ChangePublicationOutcome::EvidenceRejected(ChangePublicationEvidenceError::Unavailable)
    );
    assert_eq!(*order.lock().unwrap(), vec!["evidence"]);
}

#[test]
fn malformed_payload_digest_is_rejected_before_admission_or_evidence() {
    let order = Arc::new(Mutex::new(Vec::new()));
    let mut evidence = OrderedEvidence::successful(Arc::clone(&order));
    let mut publication = DenyingPublication {
        order: Arc::clone(&order),
    };
    let mut service = M10ChangePublicationService::new(&mut publication, &mut evidence);
    let mut ports = publication_ports(true);
    let request = command_with_digest(
        ActorReference::Authenticated {
            session_id: common::session("session:fixture"),
        },
        Some(PayloadDigest::parse("b".repeat(64)).unwrap()),
    );

    let outcome = service.submit(&request, &mut ports);

    assert_eq!(outcome, ChangePublicationOutcome::MalformedCommand);
    assert!(order.lock().unwrap().is_empty());
    assert!(evidence.events.lock().unwrap().is_empty());
}

#[test]
fn reviewed_timestamp_is_part_of_the_exact_payload_binding() {
    let order = Arc::new(Mutex::new(Vec::new()));
    let mut evidence = OrderedEvidence::successful(Arc::clone(&order));
    let mut publication = DenyingPublication {
        order: Arc::clone(&order),
    };
    let mut service = M10ChangePublicationService::new(&mut publication, &mut evidence);
    let mut ports = publication_ports(true);
    let (event_id, review_receipt_id, reviewed_at) = reviewed_identity();
    let published_at = RevisionTimestamp::from_unix_seconds(300);
    let digest = change_publication_payload_digest(
        &event_id,
        review_receipt_id.as_str(),
        reviewed_at,
        published_at,
    );
    let request = ChangePublicationCommand::new(
        common::request_id(),
        ActorReference::Authenticated {
            session_id: common::session("session:fixture"),
        },
        common::correlation_id(),
        None,
        Some(common::idem_key()),
        ClientProvenance::new("build:fixture", "linux", "m10:change-publication-v1").unwrap(),
        digest,
        event_id,
        &review_receipt_id,
        RevisionTimestamp::from_unix_seconds(reviewed_at.unix_seconds() + 1),
        published_at,
    );

    let outcome = service.submit(&request, &mut ports);

    assert_eq!(outcome, ChangePublicationOutcome::MalformedCommand);
    assert!(order.lock().unwrap().is_empty());
    assert!(evidence.events.lock().unwrap().is_empty());
}
