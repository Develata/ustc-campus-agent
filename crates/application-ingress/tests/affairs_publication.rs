#![allow(clippy::unwrap_used)]

mod common;

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use affairs_navigator::{ProcedureId, ProcedurePublicationReceipt};
use ustc_campus_agent_application_ingress::{
    AffairsPublicationApplicationError, AffairsPublicationApplicationPort,
    AffairsPublicationEvidenceError, AffairsPublicationOutcome, M10AffairsPublicationService,
    affairs_publication_payload_digest,
};
use ustc_campus_agent_core::control_evidence::{
    ControlEvidenceAppendOutcome, ControlEvidenceAppendPort, ControlEvidenceJournalError,
    ControlEvidenceKey, ControlEvidenceReadPort, PlatformControlEvent,
};
use ustc_campus_agent_core::identity::CommandId;
use ustc_campus_agent_core::request_context::{
    ActorReference, CapabilityDisposition, ClientProvenance, M00AdmittedActor, OperationSnapshot,
    PublicScope,
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

impl AffairsPublicationApplicationPort for DenyingPublication {
    fn publish(
        &mut self,
        _command_id: &CommandId,
        _actor: &M00AdmittedActor,
        _procedure_id: &ProcedureId,
        _expected_publication_revision: Option<u64>,
    ) -> Result<ProcedurePublicationReceipt, AffairsPublicationApplicationError> {
        self.order.lock().unwrap().push("publication");
        Err(AffairsPublicationApplicationError::Denied)
    }
}

fn command(
    actor_reference: ActorReference,
) -> ustc_campus_agent_application_ingress::AffairsPublicationCommand {
    let procedure_id = ProcedureId::parse("proc:fixture").unwrap();
    let digest = affairs_publication_payload_digest(&procedure_id, Some(1));
    ustc_campus_agent_application_ingress::AffairsPublicationCommand::new(
        common::request_id(),
        actor_reference,
        common::correlation_id(),
        None,
        Some(common::idem_key()),
        ClientProvenance::new("build:fixture", "linux", "m10:publication-v1").unwrap(),
        digest,
        procedure_id,
        Some(1),
    )
}

fn publication_ports(authenticated: bool) -> FakePorts {
    let descriptor: OperationSnapshot = Arc::new(Descriptor::affairs_publication());
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
fn evidence_append_precedes_direct_publication_port() {
    let order = Arc::new(Mutex::new(Vec::new()));
    let mut evidence = OrderedEvidence::successful(Arc::clone(&order));
    let mut publication = DenyingPublication {
        order: Arc::clone(&order),
    };
    let mut service = M10AffairsPublicationService::new(&mut publication, &mut evidence);
    let mut ports = publication_ports(true);
    let request = command(ActorReference::Authenticated {
        session_id: common::session("session:fixture"),
    });

    let outcome = service.submit(&request, &mut ports);

    assert_eq!(
        outcome,
        AffairsPublicationOutcome::PublicationRejected(AffairsPublicationApplicationError::Denied)
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
    let mut service = M10AffairsPublicationService::new(&mut publication, &mut evidence);
    let mut ports = publication_ports(false);
    let request = command(ActorReference::Anonymous { scope: PublicScope });

    let outcome = service.submit(&request, &mut ports);

    assert!(matches!(outcome, AffairsPublicationOutcome::Rejected(_)));
    assert!(order.lock().unwrap().is_empty());
    assert!(evidence.events.lock().unwrap().is_empty());
}

#[test]
fn session_failure_is_rejected_before_evidence_or_publication() {
    let order = Arc::new(Mutex::new(Vec::new()));
    let mut evidence = OrderedEvidence::successful(Arc::clone(&order));
    let mut publication = DenyingPublication {
        order: Arc::clone(&order),
    };
    let mut service = M10AffairsPublicationService::new(&mut publication, &mut evidence);
    let mut ports = publication_ports(true);
    ports.loaded_session = Ok(None);
    let request = command(ActorReference::Authenticated {
        session_id: common::session("session:fixture"),
    });

    let outcome = service.submit(&request, &mut ports);

    assert!(matches!(outcome, AffairsPublicationOutcome::Rejected(_)));
    assert!(order.lock().unwrap().is_empty());
}

#[test]
fn disabled_capability_is_rejected_before_evidence_or_publication() {
    let order = Arc::new(Mutex::new(Vec::new()));
    let mut evidence = OrderedEvidence::successful(Arc::clone(&order));
    let mut publication = DenyingPublication {
        order: Arc::clone(&order),
    };
    let mut service = M10AffairsPublicationService::new(&mut publication, &mut evidence);
    let mut ports = publication_ports(true);
    ports.capability = Ok(CapabilityDisposition::Disabled);
    let request = command(ActorReference::Authenticated {
        session_id: common::session("session:fixture"),
    });

    let outcome = service.submit(&request, &mut ports);

    assert!(matches!(outcome, AffairsPublicationOutcome::Rejected(_)));
    assert!(order.lock().unwrap().is_empty());
}

#[test]
fn evidence_failure_is_terminal_before_publication_port() {
    let order = Arc::new(Mutex::new(Vec::new()));
    let mut evidence = OrderedEvidence::failing(Arc::clone(&order));
    let mut publication = DenyingPublication {
        order: Arc::clone(&order),
    };
    let mut service = M10AffairsPublicationService::new(&mut publication, &mut evidence);
    let mut ports = publication_ports(true);
    let request = command(ActorReference::Authenticated {
        session_id: common::session("session:fixture"),
    });

    let outcome = service.submit(&request, &mut ports);

    assert_eq!(
        outcome,
        AffairsPublicationOutcome::EvidenceRejected(AffairsPublicationEvidenceError::Unavailable)
    );
    assert_eq!(*order.lock().unwrap(), vec!["evidence"]);
}
