use std::collections::BTreeMap;

use ustc_campus_agent_core::identity::{SessionId, TenantId, UserId};
use ustc_campus_agent_core::session::{
    AuthAdapterId, CredentialEvidenceDigest, ExpireSession, OpenSession, RefreshSession,
    RevokeSession, SessionCommand, SessionCredentialEvidence, SessionDuration, SessionEvent,
    SessionInstant, SessionPolicy, SessionRefreshed, SessionRevoked, SessionSnapshot, decide,
    evolve,
};
use ustc_campus_agent_core::session_port::{
    CredentialEvidencePort, CredentialEvidencePortError, SecretRef, SessionAppendOutcome,
    SessionClockError, SessionClockPort, SessionHistory, SessionHistoryAppendPort,
    SessionHistoryReadPort, SessionRepositoryError,
};

const MAX_EVENTS: usize = 4;
const DIGEST: &str = "sha256:00000000000000000000000000000000000000000000000000000000cafebabe";

fn parsed<T>(value: &str) -> T
where
    T: ParseFixture,
{
    T::parse_fixture(value)
}

trait ParseFixture: Sized {
    fn parse_fixture(value: &str) -> Self;
}

macro_rules! parse_fixture {
    ($kind:ty) => {
        impl ParseFixture for $kind {
            fn parse_fixture(value: &str) -> Self {
                <$kind>::parse(value).expect("synthetic fixture must parse")
            }
        }
    };
}

parse_fixture!(SessionId);
parse_fixture!(TenantId);
parse_fixture!(UserId);
parse_fixture!(AuthAdapterId);
parse_fixture!(CredentialEvidenceDigest);

fn session() -> SessionId {
    parsed("session:example")
}

fn other_session() -> SessionId {
    parsed("session:other")
}

fn at(value: u64) -> SessionInstant {
    SessionInstant::from_unix_millis(value)
}

fn policy() -> SessionPolicy {
    SessionPolicy::new(
        SessionDuration::from_millis(1_000).expect("fixture"),
        SessionDuration::from_millis(10_000).expect("fixture"),
    )
}

fn evidence() -> SessionCredentialEvidence {
    SessionCredentialEvidence::new(
        parsed("tenant:example"),
        parsed("user:example"),
        parsed("ustc.cas"),
        parsed(DIGEST),
        at(1_000),
        None,
    )
    .expect("fixture")
}

fn open_command(session_id: SessionId) -> SessionCommand {
    SessionCommand::Open(OpenSession::new(
        session_id,
        evidence(),
        policy(),
        at(1_000),
        0,
    ))
}

fn refresh_command(session_id: SessionId, observed_at: u64, revision: u64) -> SessionCommand {
    SessionCommand::Refresh(RefreshSession::new(session_id, at(observed_at), revision))
}

fn revoke_command(session_id: SessionId, observed_at: u64, revision: u64) -> SessionCommand {
    SessionCommand::Revoke(RevokeSession::new(session_id, at(observed_at), revision))
}

fn expire_command(session_id: SessionId, observed_at: u64, revision: u64) -> SessionCommand {
    SessionCommand::Expire(ExpireSession::new(session_id, at(observed_at), revision))
}

fn decided(state: Option<&SessionSnapshot>, command: &SessionCommand) -> SessionEvent {
    decide(state, command).expect("fixture command must decide")
}

fn valid_events(count: usize) -> Vec<SessionEvent> {
    assert!((1..=MAX_EVENTS).contains(&count));
    let mut events = Vec::new();
    let mut state = None;
    let open = decided(state.as_ref(), &open_command(session()));
    state = Some(evolve(state.as_ref(), &open).expect("open applies"));
    events.push(open);
    for index in 1..count {
        let command = refresh_command(
            session(),
            1_000 + u64::try_from(index).expect("small fixture") * 100,
            u64::try_from(index).expect("small fixture"),
        );
        let event = decided(state.as_ref(), &command);
        state = Some(evolve(state.as_ref(), &event).expect("refresh applies"));
        events.push(event);
    }
    events
}

#[derive(Default)]
struct FakeRepository {
    retained: BTreeMap<SessionId, Vec<SessionEvent>>,
    unavailable: bool,
    loads: u64,
    appends: u64,
}

impl FakeRepository {
    fn insert_raw(&mut self, session_id: SessionId, events: Vec<SessionEvent>) {
        self.retained.insert(session_id, events);
    }
}

impl SessionHistoryReadPort for FakeRepository {
    fn load_history(
        &mut self,
        session_id: &SessionId,
    ) -> Result<Option<SessionHistory>, SessionRepositoryError> {
        self.loads += 1;
        if self.unavailable {
            return Err(SessionRepositoryError::Unavailable);
        }
        let history = self
            .retained
            .get(session_id)
            .cloned()
            .map(SessionHistory::try_from_events)
            .transpose()?;
        if history
            .as_ref()
            .is_some_and(|retained| retained.session_id() != session_id)
        {
            return Err(SessionRepositoryError::Corrupt);
        }
        Ok(history)
    }
}

impl SessionHistoryAppendPort for FakeRepository {
    fn compare_and_append(
        &mut self,
        session_id: &SessionId,
        expected_revision: Option<u64>,
        event: &SessionEvent,
    ) -> Result<SessionAppendOutcome, SessionRepositoryError> {
        self.appends += 1;
        if self.unavailable {
            return Err(SessionRepositoryError::Unavailable);
        }

        let retained_events = self.retained.get(session_id).cloned();
        let retained = retained_events
            .as_ref()
            .map(|events| SessionHistory::try_from_events(events.clone()))
            .transpose()?;

        if let Some(current) = &retained {
            if current.revision() == event.sequence() {
                let predecessor = if event.sequence() == 1 {
                    None
                } else {
                    Some(event.sequence() - 1)
                };
                if current.events().last() == Some(event) && expected_revision == predecessor {
                    return Ok(SessionAppendOutcome::AlreadySame(current.clone()));
                }
                if current.revision() == u64::MAX {
                    return Err(SessionRepositoryError::LimitExceeded);
                }
                return Ok(SessionAppendOutcome::Conflict {
                    current_revision: Some(current.revision()),
                });
            }
            if current.revision() == u64::MAX {
                return Err(SessionRepositoryError::LimitExceeded);
            }
            if current.revision() > event.sequence() {
                return Ok(SessionAppendOutcome::Conflict {
                    current_revision: Some(current.revision()),
                });
            }
        }

        let current_revision = retained.as_ref().map(SessionHistory::revision);
        match current_revision {
            None => {
                if expected_revision.is_some() {
                    return Ok(SessionAppendOutcome::Conflict {
                        current_revision: None,
                    });
                }
                if event.session_id() != session_id || event.sequence() != 1 {
                    return Err(SessionRepositoryError::InvalidEvent);
                }
            }
            Some(revision) => {
                if expected_revision != Some(revision) {
                    return Ok(SessionAppendOutcome::Conflict {
                        current_revision: Some(revision),
                    });
                }
                let next = revision
                    .checked_add(1)
                    .ok_or(SessionRepositoryError::LimitExceeded)?;
                if event.session_id() != session_id || event.sequence() != next {
                    return Err(SessionRepositoryError::InvalidEvent);
                }
            }
        }

        let mut candidate = retained_events.unwrap_or_default();
        if candidate.len() >= MAX_EVENTS {
            return Err(SessionRepositoryError::LimitExceeded);
        }
        candidate.push(event.clone());
        let history = SessionHistory::try_from_events(candidate.clone())
            .map_err(|_| SessionRepositoryError::InvalidEvent)?;
        self.retained.insert(session_id.clone(), candidate);
        Ok(SessionAppendOutcome::Appended(history))
    }
}

struct FakeClock {
    values: Vec<Result<SessionInstant, SessionClockError>>,
    calls: usize,
}

impl SessionClockPort for FakeClock {
    fn now(&mut self) -> Result<SessionInstant, SessionClockError> {
        let value = self
            .values
            .get(self.calls)
            .copied()
            .unwrap_or(Err(SessionClockError::Unavailable));
        self.calls += 1;
        value
    }
}

struct FakeCredentialEvidence {
    entries: Vec<(AuthAdapterId, SecretRef, CredentialEvidenceDigest)>,
    unavailable: bool,
    calls: u64,
}

impl CredentialEvidencePort for FakeCredentialEvidence {
    fn fingerprint_adapter_evidence(
        &mut self,
        auth_adapter_id: &AuthAdapterId,
        secret_ref: &SecretRef,
    ) -> Result<CredentialEvidenceDigest, CredentialEvidencePortError> {
        self.calls += 1;
        if self.unavailable {
            return Err(CredentialEvidencePortError::Unavailable);
        }
        self.entries
            .iter()
            .find(|(adapter, reference, _)| adapter == auth_adapter_id && reference == secret_ref)
            .map(|(_, _, digest)| digest.clone())
            .ok_or(CredentialEvidencePortError::UnknownSecretRef)
    }
}

#[test]
fn session_history_replays_only_complete_valid_event_sequences() {
    assert!(matches!(
        SessionHistory::try_from_events(Vec::new()),
        Err(SessionRepositoryError::Corrupt)
    ));

    let events = valid_events(3);
    let history = SessionHistory::try_from_events(events.clone()).expect("valid replay");
    assert!(history.events() == events.as_slice());
    assert!(history.session_id() == &session());
    assert!(history.revision() == 3);

    let open = decided(None, &open_command(session()));
    let opened = evolve(None, &open).expect("open applies");
    let revoke = decided(
        Some(&opened),
        &revoke_command(session(), 1_100, opened.revision()),
    );
    assert!(SessionHistory::try_from_events(vec![open.clone(), revoke]).is_ok());

    let expire = decided(
        Some(&opened),
        &expire_command(session(), 2_000, opened.revision()),
    );
    assert!(SessionHistory::try_from_events(vec![open, expire]).is_ok());

    let forged_refresh =
        SessionEvent::Refreshed(SessionRefreshed::new(2, session(), at(1_100), at(9_999)));
    assert!(matches!(
        SessionHistory::try_from_events(vec![
            decided(None, &open_command(session())),
            forged_refresh,
        ]),
        Err(SessionRepositoryError::Corrupt)
    ));

    let cross_session = decided(None, &open_command(other_session()));
    let mut invalid = events;
    invalid.push(cross_session);
    assert!(matches!(
        SessionHistory::try_from_events(invalid),
        Err(SessionRepositoryError::Corrupt)
    ));
}

#[test]
fn session_read_port_distinguishes_absent_unavailable_and_corrupt() {
    let mut repository = FakeRepository::default();
    assert!(
        repository
            .load_history(&session())
            .expect("available")
            .is_none()
    );

    repository.unavailable = true;
    assert!(matches!(
        repository.load_history(&session()),
        Err(SessionRepositoryError::Unavailable)
    ));
    repository.unavailable = false;

    repository.insert_raw(
        session(),
        vec![decided(None, &open_command(other_session()))],
    );
    assert!(matches!(
        repository.load_history(&session()),
        Err(SessionRepositoryError::Corrupt)
    ));
}

#[test]
fn session_append_fake_is_exactly_fenced_and_atomic() {
    let session_id = session();
    let mut repository = FakeRepository::default();
    let open = decided(None, &open_command(session_id.clone()));

    assert!(matches!(
        repository.compare_and_append(&session_id, Some(0), &open),
        Ok(SessionAppendOutcome::Conflict {
            current_revision: None
        })
    ));
    assert!(repository.retained.is_empty());

    let opened = match repository
        .compare_and_append(&session_id, None, &open)
        .expect("open append")
    {
        SessionAppendOutcome::Appended(history) => history,
        _ => panic!("expected append"),
    };
    assert!(opened.revision() == 1);

    assert!(matches!(
        repository
            .compare_and_append(&session_id, None, &open)
            .expect("exact retry"),
        SessionAppendOutcome::AlreadySame(_)
    ));

    let mut state = Some(opened.snapshot().clone());
    for revision in 1..MAX_EVENTS {
        let command = refresh_command(
            session_id.clone(),
            1_000 + u64::try_from(revision).expect("small fixture") * 100,
            u64::try_from(revision).expect("small fixture"),
        );
        let event = decided(state.as_ref(), &command);
        let outcome = repository
            .compare_and_append(
                &session_id,
                Some(u64::try_from(revision).expect("small fixture")),
                &event,
            )
            .expect("bounded append");
        let SessionAppendOutcome::Appended(history) = outcome else {
            panic!("expected append");
        };
        state = Some(history.snapshot().clone());
    }

    let before = repository
        .retained
        .get(&session_id)
        .cloned()
        .expect("retained");
    let fifth = decided(
        state.as_ref(),
        &refresh_command(session_id.clone(), 1_500, 4),
    );
    assert!(matches!(
        repository.compare_and_append(&session_id, Some(4), &fifth),
        Err(SessionRepositoryError::LimitExceeded)
    ));
    assert!(repository.retained.get(&session_id) == Some(&before));
}

#[test]
fn session_append_fake_rejects_historical_retry_after_later_events() {
    let session_id = session();
    let mut repository = FakeRepository::default();
    let open = decided(None, &open_command(session_id.clone()));
    let opened = match repository
        .compare_and_append(&session_id, None, &open)
        .expect("open")
    {
        SessionAppendOutcome::Appended(history) => history,
        _ => panic!("expected append"),
    };
    let refresh = decided(
        Some(opened.snapshot()),
        &refresh_command(session_id.clone(), 1_100, 1),
    );
    assert!(matches!(
        repository.compare_and_append(&session_id, Some(1), &refresh),
        Ok(SessionAppendOutcome::Appended(_))
    ));
    assert!(matches!(
        repository
            .compare_and_append(&session_id, None, &open)
            .expect("historical retry is typed conflict"),
        SessionAppendOutcome::Conflict {
            current_revision: Some(2)
        }
    ));

    let before = repository
        .retained
        .get(&session_id)
        .cloned()
        .expect("retained");
    let forged = SessionEvent::Revoked(SessionRevoked::new(3, other_session(), at(1_200)));
    assert!(matches!(
        repository.compare_and_append(&session_id, Some(2), &forged),
        Err(SessionRepositoryError::InvalidEvent)
    ));
    assert!(repository.retained.get(&session_id) == Some(&before));
}

#[test]
fn session_clock_fake_is_deterministic_and_fail_closed() {
    let mut clock = FakeClock {
        values: vec![
            Ok(at(1_000)),
            Ok(at(1_001)),
            Err(SessionClockError::Unavailable),
        ],
        calls: 0,
    };
    assert!(clock.now() == Ok(at(1_000)));
    assert!(clock.now() == Ok(at(1_001)));
    assert!(clock.now() == Err(SessionClockError::Unavailable));
    assert!(clock.now() == Err(SessionClockError::Unavailable));
    assert!(clock.calls == 4);
}

#[test]
fn credential_evidence_port_uses_only_logical_secret_refs_and_redacts() {
    assert!(SecretRef::parse("secret-ref:demo.adapter").is_some());
    for invalid in [
        "",
        "secret-ref:",
        "secret-ref:Upper",
        "secret-ref:/tmp/key",
        "secret-ref:demo secret",
        "secret-ref:é",
    ] {
        assert!(SecretRef::parse(invalid).is_none(), "accepted {invalid:?}");
    }

    let reference = SecretRef::parse("secret-ref:hunter2").expect("logical ref");
    assert!(format!("{reference:?}") == "SecretRef(<redacted>)");
    assert!(serde_json::to_string(&reference).expect("serialize") == "\"secret-ref:hunter2\"");
    let decoded: SecretRef = serde_json::from_str("\"secret-ref:hunter2\"").expect("decode");
    assert!(decoded == reference);

    let adapter: AuthAdapterId = parsed("ustc.cas");
    let digest: CredentialEvidenceDigest = parsed(DIGEST);
    let mut port = FakeCredentialEvidence {
        entries: vec![(adapter.clone(), reference.clone(), digest.clone())],
        unavailable: false,
        calls: 0,
    };
    assert!(
        port.fingerprint_adapter_evidence(&adapter, &reference)
            .expect("known")
            == digest
    );
    let unknown = SecretRef::parse("secret-ref:unknown").expect("fixture");
    assert!(matches!(
        port.fingerprint_adapter_evidence(&adapter, &unknown),
        Err(CredentialEvidencePortError::UnknownSecretRef)
    ));
    port.unavailable = true;
    assert!(matches!(
        port.fingerprint_adapter_evidence(&adapter, &reference),
        Err(CredentialEvidencePortError::Unavailable)
    ));
    assert!(!format!("{:?}", CredentialEvidencePortError::UnknownSecretRef).contains("hunter2"));
    assert!(port.calls == 3);
}
