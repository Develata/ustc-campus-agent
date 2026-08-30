//! Executable evidence for `platform-control-evidence/v0` (`AUTH-022`).

use std::collections::BTreeMap;
use std::sync::Arc;

use ustc_campus_agent_core::control_evidence::*;
use ustc_campus_agent_core::identity::{
    CommandId, CorrelationId, RequestId, SessionId, TenantId, UserId,
};
use ustc_campus_agent_core::request_context::*;
use ustc_campus_agent_core::session::{
    AuthAdapterId, CredentialEvidenceDigest, EventDerivedField, ExpireSession, OpenSession,
    RefreshSession, RevokeSession, SessionCommand, SessionCredentialEvidence, SessionDomainError,
    SessionDuration, SessionEvent, SessionInstant, SessionPolicy, SessionSnapshot, SessionStatus,
    decide, evolve,
};
use ustc_campus_agent_core::session_port::SessionRepositoryError;

const DIGEST: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const CREDENTIAL_CANARY: &str =
    "sha256:credential-canary-aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

fn operation(value: &str) -> OperationId {
    OperationId::parse(value).expect("fixture operation")
}
fn policy_id() -> PlatformPolicySnapshotId {
    PlatformPolicySnapshotId::parse("policy:fixture").expect("fixture policy")
}
fn request_id(value: &str) -> RequestId {
    RequestId::parse(value).expect("fixture request")
}
fn command_id(value: &str) -> CommandId {
    CommandId::parse(value).expect("fixture command")
}
fn correlation_id(value: &str) -> CorrelationId {
    CorrelationId::parse(value).expect("fixture correlation")
}
fn idempotency_key(value: &str) -> IdempotencyKey {
    IdempotencyKey::parse(value).expect("fixture key")
}
fn tenant() -> TenantId {
    TenantId::parse("tenant:fixture").expect("fixture tenant")
}
fn user() -> UserId {
    UserId::parse("user:fixture").expect("fixture user")
}
fn session() -> SessionId {
    SessionId::parse("session:fixture").expect("fixture session")
}
fn at(value: u64) -> SessionInstant {
    SessionInstant::from_unix_millis(value)
}
fn schema_digest() -> SchemaDigest {
    SchemaDigest::parse(DIGEST).expect("fixture digest")
}
fn descriptor_id() -> DescriptorSnapshotId {
    DescriptorSnapshotId::from_canonical_identity(&schema_digest(), 7).expect("fixture descriptor")
}
fn reservation_token() -> IdempotencyReservationToken {
    IdempotencyReservationToken::from_store_observation(
        command_id("command:fixture"),
        3,
        1,
        at(1_100),
    )
    .expect("fixture token")
}

#[derive(Clone)]
struct Descriptor {
    operation_id: OperationId,
    schema_identity: SchemaIdentity,
    schema_digest: SchemaDigest,
    permission_class: PermissionClass,
    effect_class: EffectClass,
    decoder_identity: DecoderIdentity,
    dispatcher_identity: DispatcherIdentity,
    adapter_allowlist: AdapterAllowlist,
    snapshot_identity: DescriptorSnapshotId,
}

impl Descriptor {
    fn new(permission_class: PermissionClass) -> Self {
        let effect_class = match permission_class {
            PermissionClass::PublicRead | PermissionClass::TenantPrivateRead => EffectClass::Read,
            PermissionClass::PublicLinkout => EffectClass::LinkOut,
            PermissionClass::TenantPrivateWrite => EffectClass::TenantLocalMutation,
        };
        Self {
            operation_id: operation("affairs.publish"),
            schema_identity: SchemaIdentity::parse("schema:fixture").expect("fixture"),
            schema_digest: schema_digest(),
            permission_class,
            effect_class,
            decoder_identity: DecoderIdentity::parse("decoder:fixture").expect("fixture"),
            dispatcher_identity: DispatcherIdentity::parse("dispatcher:fixture").expect("fixture"),
            adapter_allowlist: AdapterAllowlist::try_from_iter([AdapterIdentity::parse(
                "adapter:fixture",
            )
            .expect("fixture")])
            .expect("fixture"),
            snapshot_identity: descriptor_id(),
        }
    }
}

impl OperationDescriptorProjection for Descriptor {
    fn operation_id(&self) -> &OperationId {
        &self.operation_id
    }
    fn schema_identity(&self) -> &SchemaIdentity {
        &self.schema_identity
    }
    fn schema_digest(&self) -> &SchemaDigest {
        &self.schema_digest
    }
    fn permission_class(&self) -> PermissionClass {
        self.permission_class
    }
    fn effect_class(&self) -> EffectClass {
        self.effect_class
    }
    fn decoder_identity(&self) -> &DecoderIdentity {
        &self.decoder_identity
    }
    fn dispatcher_identity(&self) -> &DispatcherIdentity {
        &self.dispatcher_identity
    }
    fn adapter_allowlist(&self) -> &AdapterAllowlist {
        &self.adapter_allowlist
    }
    fn snapshot_identity(&self) -> &DescriptorSnapshotId {
        &self.snapshot_identity
    }
}

fn active_session() -> SessionSnapshot {
    let evidence = SessionCredentialEvidence::new(
        tenant(),
        user(),
        AuthAdapterId::parse("fixture.adapter").expect("fixture"),
        CredentialEvidenceDigest::parse(
            "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        )
        .expect("fixture"),
        at(1_000),
        None,
    )
    .expect("fixture evidence");
    let command = SessionCommand::Open(OpenSession::new(
        session(),
        evidence,
        SessionPolicy::new(
            SessionDuration::from_millis(100).expect("fixture"),
            SessionDuration::from_millis(1_000).expect("fixture"),
        ),
        at(1_000),
        0,
    ));
    let event = decide(None, &command).expect("fixture open");
    evolve(None, &event).expect("fixture evolve")
}

#[derive(Clone)]
struct FakePorts {
    reservation: Result<IdempotencyReservation, IdempotencyError>,
    descriptor: Result<OperationSnapshot, DescriptorSnapshotError>,
    now: Result<SessionInstant, AdmissionPortError>,
    policy: Result<PolicyResolution, AdmissionPortError>,
    loaded_session: Result<Option<SessionSnapshot>, AdmissionPortError>,
    capability: Result<CapabilityDisposition, AdmissionPortError>,
    finalize: Result<FinalizeIdempotencyOutcome, IdempotencyError>,
}

impl FakePorts {
    fn public() -> Self {
        Self {
            reservation: Ok(IdempotencyReservation::New(reservation_token())),
            descriptor: Ok(Arc::new(Descriptor::new(PermissionClass::PublicRead))),
            now: Ok(at(1_000)),
            policy: Ok(PolicyResolution::new(
                policy_id(),
                PolicyCurrentnessFact::Current,
            )),
            loaded_session: Ok(None),
            capability: Ok(CapabilityDisposition::Enabled),
            finalize: Ok(FinalizeIdempotencyOutcome::Committed),
        }
    }

    fn authenticated() -> Self {
        Self {
            descriptor: Ok(Arc::new(Descriptor::new(
                PermissionClass::TenantPrivateWrite,
            ))),
            loaded_session: Ok(Some(active_session())),
            ..Self::public()
        }
    }
}

impl AdmissionPorts for FakePorts {
    fn reserve_or_retrieve_idempotency(
        &mut self,
        _key: Option<&IdempotencyKey>,
        _envelope_hash: &EnvelopeHash,
    ) -> Result<IdempotencyReservation, IdempotencyError> {
        self.reservation.clone()
    }

    fn request_scoped_operation(&mut self) -> Result<OperationSnapshot, DescriptorSnapshotError> {
        self.descriptor.clone()
    }

    fn now(&mut self) -> Result<SessionInstant, AdmissionPortError> {
        self.now
    }

    fn resolve_policy(
        &mut self,
        _operation_id: &OperationId,
        _observed_at: SessionInstant,
    ) -> Result<PolicyResolution, AdmissionPortError> {
        self.policy.clone()
    }

    fn load_session(
        &mut self,
        _session_id: &SessionId,
    ) -> Result<Option<SessionSnapshot>, AdmissionPortError> {
        self.loaded_session.clone()
    }

    fn check_capability(
        &mut self,
        _operation_id: &OperationId,
        _actor_kind: ActorKind,
        _observed_at: SessionInstant,
    ) -> Result<CapabilityDisposition, AdmissionPortError> {
        self.capability
    }

    fn finalize_idempotency(
        &mut self,
        _token: &IdempotencyReservationToken,
        _disposition: &FinalAdmissionDisposition,
    ) -> Result<FinalizeIdempotencyOutcome, IdempotencyError> {
        self.finalize.clone()
    }
}

fn command(authenticated: bool) -> BuildRequestContextCommand {
    BuildRequestContextCommand::new(
        request_id(if authenticated {
            "request:authenticated"
        } else {
            "request:public"
        }),
        operation("affairs.publish"),
        if authenticated {
            ActorReference::Authenticated {
                session_id: session(),
            }
        } else {
            ActorReference::Anonymous { scope: PublicScope }
        },
        correlation_id("correlation:fixture"),
        authenticated.then(|| CausationId::parse("causation:fixture").expect("fixture")),
        Some(idempotency_key("idempotency:fixture")),
        ClientProvenance::new("build:fixture", "linux", "m10:v2").expect("fixture"),
        PayloadDigest::parse(DIGEST).expect("fixture"),
    )
}

fn admitted_context(authenticated: bool) -> PlatformRequestContext {
    let mut ports = if authenticated {
        FakePorts::authenticated()
    } else {
        FakePorts::public()
    };
    match RequestAdmissionCoordinator.admit(&command(authenticated), &mut ports) {
        M00AdmissionResult::Admitted { context, .. } => context,
        other => panic!("expected admission, got {other:?}"),
    }
}

fn session_events() -> [SessionEvent; 4] {
    let evidence = SessionCredentialEvidence::new(
        tenant(),
        user(),
        AuthAdapterId::parse("fixture.adapter").expect("fixture"),
        CredentialEvidenceDigest::parse(
            "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
        )
        .expect("fixture"),
        at(1_000),
        None,
    )
    .expect("fixture evidence");
    let open = SessionCommand::Open(OpenSession::new(
        session(),
        evidence,
        SessionPolicy::new(
            SessionDuration::from_millis(100).expect("fixture"),
            SessionDuration::from_millis(1_000).expect("fixture"),
        ),
        at(1_000),
        0,
    ));
    let opened = decide(None, &open).expect("open");
    let opened_snapshot = evolve(None, &opened).expect("open evolve");
    let refresh = SessionCommand::Refresh(RefreshSession::new(session(), at(1_050), 1));
    let refreshed = decide(Some(&opened_snapshot), &refresh).expect("refresh");
    let refreshed_snapshot = evolve(Some(&opened_snapshot), &refreshed).expect("refresh evolve");
    let expire = SessionCommand::Expire(ExpireSession::new(session(), at(1_150), 2));
    let expired = decide(Some(&refreshed_snapshot), &expire).expect("expire");
    let revoke = SessionCommand::Revoke(RevokeSession::new(session(), at(1_100), 2));
    let revoked = decide(Some(&refreshed_snapshot), &revoke).expect("revoke");
    [opened, refreshed, expired, revoked]
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FakeJournal {
    events: BTreeMap<ControlEvidenceKey, PlatformControlEvent>,
    unavailable: bool,
    corrupt: bool,
    max_records: usize,
    fail_commit: bool,
}

impl FakeJournal {
    fn empty(max_records: usize) -> Self {
        Self {
            events: BTreeMap::new(),
            unavailable: false,
            corrupt: false,
            max_records,
            fail_commit: false,
        }
    }
}

impl ControlEvidenceReadPort for FakeJournal {
    fn load_control_event(
        &mut self,
        key: &ControlEvidenceKey,
    ) -> Result<Option<PlatformControlEvent>, ControlEvidenceJournalError> {
        if self.unavailable {
            return Err(ControlEvidenceJournalError::Unavailable);
        }
        if self.corrupt {
            return Err(ControlEvidenceJournalError::Corrupt);
        }
        Ok(self.events.get(key).cloned())
    }
}

impl ControlEvidenceAppendPort for FakeJournal {
    fn append_once(
        &mut self,
        event: &PlatformControlEvent,
    ) -> Result<ControlEvidenceAppendOutcome, ControlEvidenceJournalError> {
        if self.unavailable {
            return Err(ControlEvidenceJournalError::Unavailable);
        }
        if self.corrupt {
            return Err(ControlEvidenceJournalError::Corrupt);
        }
        let key = event.key();
        if let Some(retained) = self.events.get(&key) {
            return Ok(if retained == event {
                ControlEvidenceAppendOutcome::AlreadySame
            } else {
                ControlEvidenceAppendOutcome::Conflict
            });
        }
        if self.events.len() >= self.max_records {
            return Err(ControlEvidenceJournalError::LimitExceeded);
        }
        let mut candidate = self.events.clone();
        if candidate.insert(key, event.clone()).is_some() || self.fail_commit {
            return Err(ControlEvidenceJournalError::InternalInvariant);
        }
        self.events = candidate;
        Ok(ControlEvidenceAppendOutcome::Appended)
    }
}

fn request_event(command: &str, request: &str, correlation: &str) -> PlatformControlEvent {
    PlatformControlEvent::RequestAdmitted {
        request_id: request_id(request),
        command_id: command_id(command),
        correlation_id: correlation_id(correlation),
        causation_id: None,
        actor: PlatformControlActor::Public,
        operation_id: operation("affairs.publish"),
        descriptor_snapshot_id: descriptor_id(),
        permission_class: PermissionClass::PublicRead,
        effect_class: EffectClass::Read,
        policy_snapshot_id: policy_id(),
        observed_at: at(1_000),
    }
}

#[test]
fn session_events_project_one_to_one_without_credential_evidence() {
    let projected: Vec<_> = session_events()
        .iter()
        .map(PlatformControlEvent::from_session_event)
        .collect();
    assert_eq!(
        projected
            .iter()
            .map(PlatformControlEvent::kind)
            .collect::<Vec<_>>(),
        vec![
            PlatformControlEventKind::SessionOpened,
            PlatformControlEventKind::SessionRefreshed,
            PlatformControlEventKind::SessionExpired,
            PlatformControlEventKind::SessionRevoked,
        ]
    );
    assert_eq!(
        projected
            .iter()
            .map(PlatformControlEvent::occurred_at)
            .collect::<Vec<_>>(),
        vec![at(1_000), at(1_050), at(1_150), at(1_100)]
    );
    for (expected_sequence, event) in [1_u64, 2, 3, 3].into_iter().zip(&projected) {
        assert_eq!(
            event.key(),
            ControlEvidenceKey::Session {
                session_id: session(),
                sequence: expected_sequence,
            }
        );
        let json = serde_json::to_string(event).expect("serialize");
        assert!(!json.contains("evidence_digest"));
        assert!(!json.contains("credential_not_after"));
        assert!(!json.contains(CREDENTIAL_CANARY));
    }
}

#[test]
fn request_admission_event_retains_only_stable_control_fields() {
    for authenticated in [false, true] {
        let context = admitted_context(authenticated);
        let event = PlatformControlEvent::from_admitted_request(&context);
        assert_eq!(event.kind(), PlatformControlEventKind::RequestAdmitted);
        assert_eq!(event.occurred_at(), at(1_000));
        assert_eq!(
            event.key(),
            ControlEvidenceKey::Request {
                command_id: command_id("command:fixture")
            }
        );
        let PlatformControlEvent::RequestAdmitted {
            actor,
            permission_class,
            effect_class,
            ..
        } = &event
        else {
            panic!("request event")
        };
        if authenticated {
            assert!(matches!(
                actor,
                PlatformControlActor::Authenticated {
                    tenant_id,
                    user_id,
                    session_id,
                } if tenant_id == &tenant() && user_id == &user() && session_id == &session()
            ));
            assert_eq!(*permission_class, PermissionClass::TenantPrivateWrite);
            assert_eq!(*effect_class, EffectClass::TenantLocalMutation);
        } else {
            assert_eq!(actor, &PlatformControlActor::Public);
            assert_eq!(*permission_class, PermissionClass::PublicRead);
            assert_eq!(*effect_class, EffectClass::Read);
        }
        let json = serde_json::to_string(&event).expect("serialize");
        for forbidden in [
            "payload_digest",
            "idempotency",
            "client_provenance",
            "operation_snapshot",
            CREDENTIAL_CANARY,
        ] {
            assert!(!json.contains(forbidden), "forbidden {forbidden}");
        }
    }
}

#[test]
fn control_errors_map_every_domain_admission_repository_and_boundary_class() {
    let lifecycle = [
        (
            SessionDomainError::CredentialEvidenceExpired,
            PlatformControlErrorCode::LifecycleCredentialEvidenceExpired,
        ),
        (
            SessionDomainError::InvalidTimeOrder,
            PlatformControlErrorCode::LifecycleInvalidTimeOrder,
        ),
        (
            SessionDomainError::DeadlineOverflow,
            PlatformControlErrorCode::LifecycleDeadlineOverflow,
        ),
        (
            SessionDomainError::SessionNotFound,
            PlatformControlErrorCode::LifecycleSessionNotFound,
        ),
        (
            SessionDomainError::SessionAlreadyExists,
            PlatformControlErrorCode::LifecycleSessionAlreadyExists,
        ),
        (
            SessionDomainError::SessionIdMismatch,
            PlatformControlErrorCode::LifecycleSessionIdMismatch,
        ),
        (
            SessionDomainError::RevisionMismatch {
                expected: 7,
                actual: 9,
            },
            PlatformControlErrorCode::LifecycleRevisionMismatch,
        ),
        (
            SessionDomainError::RevisionOverflow,
            PlatformControlErrorCode::LifecycleRevisionOverflow,
        ),
        (
            SessionDomainError::TerminalSession {
                status: SessionStatus::Revoked {
                    revoked_at: at(1_001),
                },
            },
            PlatformControlErrorCode::LifecycleTerminalSession,
        ),
        (
            SessionDomainError::NonMonotoneTime,
            PlatformControlErrorCode::LifecycleNonMonotoneTime,
        ),
        (
            SessionDomainError::SessionNotYetExpired,
            PlatformControlErrorCode::LifecycleSessionNotYetExpired,
        ),
        (
            SessionDomainError::NoEffectiveRefresh,
            PlatformControlErrorCode::LifecycleNoEffectiveRefresh,
        ),
        (
            SessionDomainError::EventSequenceMismatch {
                expected: 10,
                actual: 11,
            },
            PlatformControlErrorCode::LifecycleEventSequenceMismatch,
        ),
        (
            SessionDomainError::EventTimeOutsideValidity,
            PlatformControlErrorCode::LifecycleEventTimeOutsideValidity,
        ),
        (
            SessionDomainError::IllegalEventForState,
            PlatformControlErrorCode::LifecycleIllegalEventForState,
        ),
        (
            SessionDomainError::EventDerivedFieldMismatch {
                field: EventDerivedField::ExpiryCause,
            },
            PlatformControlErrorCode::LifecycleEventDerivedFieldMismatch,
        ),
    ];
    for (error, expected) in lifecycle {
        assert_eq!(
            PlatformControlError::from_session_domain(&error).code(),
            expected
        );
    }

    let admission = vec![
        (
            PersistedAdmissionRejectionDto::IdempotencyStoreUnavailable {
                operation_id: operation("affairs.publish"),
            },
            PlatformControlErrorCode::AdmissionIdempotencyStoreUnavailable,
        ),
        (
            PersistedAdmissionRejectionDto::ConflictingEnvelope {
                operation_id: operation("affairs.publish"),
                idempotency_key: idempotency_key("idempotency:credential-canary"),
            },
            PlatformControlErrorCode::AdmissionConflictingEnvelope,
        ),
        (
            PersistedAdmissionRejectionDto::DescriptorSnapshotAbsent {
                operation_id: operation("affairs.publish"),
            },
            PlatformControlErrorCode::AdmissionDescriptorSnapshotAbsent,
        ),
        (
            PersistedAdmissionRejectionDto::DescriptorSnapshotMismatch {
                command_operation_id: operation("affairs.publish"),
                snapshot_operation_id: operation("affairs.other"),
            },
            PlatformControlErrorCode::AdmissionDescriptorSnapshotMismatch,
        ),
        (
            PersistedAdmissionRejectionDto::PolicyDenied {
                operation_id: operation("affairs.publish"),
                permission_class: PermissionClass::TenantPrivateWrite,
            },
            PlatformControlErrorCode::AdmissionPolicyDenied,
        ),
        (
            PersistedAdmissionRejectionDto::PolicyExpired {
                operation_id: operation("affairs.publish"),
                policy_snapshot_id: policy_id(),
            },
            PlatformControlErrorCode::AdmissionPolicyExpired,
        ),
        (
            PersistedAdmissionRejectionDto::SessionNotFound {
                requested_session_id: session(),
            },
            PlatformControlErrorCode::AdmissionSessionNotFound,
        ),
        (
            PersistedAdmissionRejectionDto::SessionIdMismatch {
                requested_session_id: session(),
                loaded_session_id: SessionId::parse("session:other").expect("fixture"),
            },
            PlatformControlErrorCode::AdmissionSessionIdMismatch,
        ),
        (
            PersistedAdmissionRejectionDto::SessionNotAdmitted {
                requested_session_id: session(),
                observed_at: at(1_000),
            },
            PlatformControlErrorCode::AdmissionSessionNotAdmitted,
        ),
        (
            PersistedAdmissionRejectionDto::CapabilityMissing {
                operation_id: operation("affairs.publish"),
                actor_kind: ActorKind::Authenticated,
            },
            PlatformControlErrorCode::AdmissionCapabilityMissing,
        ),
        (
            PersistedAdmissionRejectionDto::CapabilityDisabled {
                operation_id: operation("affairs.publish"),
                actor_kind: ActorKind::Authenticated,
            },
            PlatformControlErrorCode::AdmissionCapabilityDisabled,
        ),
        (
            PersistedAdmissionRejectionDto::CapabilityRevoked {
                operation_id: operation("affairs.publish"),
                actor_kind: ActorKind::Authenticated,
            },
            PlatformControlErrorCode::AdmissionCapabilityRevoked,
        ),
        (
            PersistedAdmissionRejectionDto::InfrastructurePortUnavailable {
                operation_id: operation("affairs.publish"),
                port: AdmissionPortKind::Session,
            },
            PlatformControlErrorCode::AdmissionInfrastructurePortUnavailable,
        ),
        (
            PersistedAdmissionRejectionDto::MalformedCommand {
                operation_id: Some(operation("affairs.publish")),
            },
            PlatformControlErrorCode::AdmissionMalformedCommand,
        ),
    ];
    for (projection, expected) in admission {
        let mut ports = FakePorts::public();
        ports.reservation = Ok(IdempotencyReservation::PriorIdentical(
            PersistedPriorDispositionDto::Rejected(projection),
        ));
        let rejection = match RequestAdmissionCoordinator.admit(&command(false), &mut ports) {
            M00AdmissionResult::PriorRejected(value) => value,
            other => panic!("expected prior rejection, got {other:?}"),
        };
        let projected = PlatformControlError::from_admission_rejection(&rejection);
        assert_eq!(projected.code(), expected);
        let rendered = format!(
            "{projected:?} {}",
            serde_json::to_string(&projected).expect("json")
        );
        assert!(!rendered.contains("credential-canary"));
    }

    let repository = [
        (
            SessionRepositoryError::Unavailable,
            PlatformControlErrorCode::RepositoryUnavailable,
        ),
        (
            SessionRepositoryError::Corrupt,
            PlatformControlErrorCode::RepositoryCorrupt,
        ),
        (
            SessionRepositoryError::InvalidEvent,
            PlatformControlErrorCode::RepositoryInvalidEvent,
        ),
        (
            SessionRepositoryError::LimitExceeded,
            PlatformControlErrorCode::RepositoryLimitExceeded,
        ),
        (
            SessionRepositoryError::InternalInvariant,
            PlatformControlErrorCode::RepositoryInternalInvariant,
        ),
    ];
    for (error, expected) in repository {
        assert_eq!(
            PlatformControlError::from_session_repository(error).code(),
            expected
        );
    }
    assert_eq!(
        PlatformControlError::malformed_external_input().code(),
        PlatformControlErrorCode::MalformedExternalInput
    );
}

#[test]
fn control_event_serde_is_closed_redacted_and_data_only() {
    let event = request_event("command:serde", "request:serde", "correlation:serde");
    let json = serde_json::to_string(&event).expect("serialize");
    let decoded: PlatformControlEvent = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(decoded, event);

    let mut value = serde_json::to_value(&event).expect("value");
    value.as_object_mut().expect("object").insert(
        "credential_digest".to_owned(),
        serde_json::Value::String(CREDENTIAL_CANARY.to_owned()),
    );
    assert!(serde_json::from_value::<PlatformControlEvent>(value).is_err());
    assert!(serde_json::from_str::<PlatformControlEvent>("{\"kind\":\"unknown\"}").is_err());
    assert!(!json.contains(CREDENTIAL_CANARY));
}

#[test]
fn control_evidence_fake_is_idempotent_conflict_safe_and_atomic() {
    let first = request_event("command:dedupe", "request:first", "correlation:first");
    let conflicting = request_event("command:dedupe", "request:second", "correlation:second");
    let mut journal = FakeJournal::empty(1);
    assert_eq!(
        journal.append_once(&first),
        Ok(ControlEvidenceAppendOutcome::Appended)
    );
    let retained = journal.clone();
    assert_eq!(
        journal.append_once(&first),
        Ok(ControlEvidenceAppendOutcome::AlreadySame)
    );
    assert_eq!(journal, retained);
    assert_eq!(
        journal.append_once(&conflicting),
        Ok(ControlEvidenceAppendOutcome::Conflict)
    );
    assert_eq!(journal, retained);

    let second = request_event("command:second", "request:second", "correlation:second");
    assert_eq!(
        journal.append_once(&second),
        Err(ControlEvidenceJournalError::LimitExceeded)
    );
    assert_eq!(journal, retained);

    let mut fail = FakeJournal::empty(2);
    fail.fail_commit = true;
    let before = fail.clone();
    assert_eq!(
        fail.append_once(&first),
        Err(ControlEvidenceJournalError::InternalInvariant)
    );
    assert_eq!(fail, before);
}

#[test]
fn control_evidence_ports_distinguish_absent_unavailable_corrupt_and_limit() {
    let event = request_event("command:ports", "request:ports", "correlation:ports");
    let key = event.key();
    let mut journal = FakeJournal::empty(0);
    assert_eq!(journal.load_control_event(&key), Ok(None));
    assert_eq!(
        journal.append_once(&event),
        Err(ControlEvidenceJournalError::LimitExceeded)
    );
    assert_eq!(journal.load_control_event(&key), Ok(None));

    journal.unavailable = true;
    assert_eq!(
        journal.load_control_event(&key),
        Err(ControlEvidenceJournalError::Unavailable)
    );
    assert_eq!(
        journal.append_once(&event),
        Err(ControlEvidenceJournalError::Unavailable)
    );

    journal.unavailable = false;
    journal.corrupt = true;
    assert_eq!(
        journal.load_control_event(&key),
        Err(ControlEvidenceJournalError::Corrupt)
    );
    assert_eq!(
        journal.append_once(&event),
        Err(ControlEvidenceJournalError::Corrupt)
    );
}
