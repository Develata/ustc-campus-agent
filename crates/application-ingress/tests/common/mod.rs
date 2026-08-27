#![allow(dead_code)]
#![allow(clippy::unwrap_used)]

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

static COUNTER: AtomicU64 = AtomicU64::new(0);

pub fn temp_path() -> PathBuf {
    let id = COUNTER.fetch_add(1, Ordering::SeqCst);
    let path = std::env::temp_dir()
        .join(format!("m10-test-{}-{}", std::process::id(), id))
        .join("store.json");
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("create temp dir");
    }
    path
}

// ---------------------------------------------------------------------------
// Client-protocol fixture builders
// ---------------------------------------------------------------------------

use ustc_campus_agent_client_protocol::{
    AdmittedActorDto, AffairsGetPayloadDto, DispatchCapsuleBodyV2, FrozenPrerequisitesDto,
    M71LineageDto, M71OutcomeDto, M71TerminalDto, UnixMillis, WireText, affairs_get_payload_digest,
};

pub fn authenticated_capsule(command_id: &str) -> DispatchCapsuleBodyV2 {
    DispatchCapsuleBodyV2::try_new(
        WireText::parse(command_id).unwrap(),
        WireText::parse("corr:fixture").unwrap(),
        AdmittedActorDto::Authenticated {
            tenant_id: WireText::parse("tenant:fixture").unwrap(),
            user_id: WireText::parse("user:fixture").unwrap(),
            session_id: WireText::parse("session:fixture").unwrap(),
        },
        AffairsGetPayloadDto {
            procedure_id: WireText::parse("proc:fixture").unwrap(),
            as_of: None,
        },
        WireText::parse(
            "descriptor:v0:1:0000000000000000000000000000000000000000000000000000000000000000",
        )
        .unwrap(),
        WireText::parse("0000000000000000000000000000000000000000000000000000000000000000")
            .unwrap(),
        1,
        FrozenPrerequisitesDto {
            policy_snapshot_id: WireText::parse("policy:fixture:v1").unwrap(),
            observed_at: UnixMillis::new(1_700_000_000_000),
            session_id: Some(WireText::parse("session:fixture").unwrap()),
            admitted_operation_id: WireText::parse("affairs.get").unwrap(),
        },
    )
    .unwrap()
}

pub fn public_capsule(command_id: &str) -> DispatchCapsuleBodyV2 {
    DispatchCapsuleBodyV2::try_new(
        WireText::parse(command_id).unwrap(),
        WireText::parse("corr:fixture").unwrap(),
        AdmittedActorDto::Public,
        AffairsGetPayloadDto {
            procedure_id: WireText::parse("proc:fixture").unwrap(),
            as_of: None,
        },
        WireText::parse(
            "descriptor:v0:1:0000000000000000000000000000000000000000000000000000000000000000",
        )
        .unwrap(),
        WireText::parse("0000000000000000000000000000000000000000000000000000000000000000")
            .unwrap(),
        1,
        FrozenPrerequisitesDto {
            policy_snapshot_id: WireText::parse("policy:fixture:v1").unwrap(),
            observed_at: UnixMillis::new(1_700_000_000_000),
            session_id: None,
            admitted_operation_id: WireText::parse("affairs.get").unwrap(),
        },
    )
    .unwrap()
}

pub fn not_found_terminal() -> M71TerminalDto {
    M71TerminalDto::try_new(
        M71OutcomeDto::NotFound {
            procedure_id: WireText::parse("proc:missing").unwrap(),
        },
        M71LineageDto::NotRequired {
            materialization_receipt_id: WireText::parse("receipt:002").unwrap(),
            reason: WireText::parse("no_visible_artifact").unwrap(),
        },
    )
    .unwrap()
}

pub fn digest_of(capsule: &ustc_campus_agent_client_protocol::DispatchCapsuleBodyV2) -> String {
    ustc_campus_agent_application_ingress::capsule_digest(capsule).expect("digest")
}

pub fn found_terminal() -> M71TerminalDto {
    M71TerminalDto::try_new(
        M71OutcomeDto::Found {
            view: Box::new(ustc_campus_agent_client_protocol::ProcedureViewDto {
                procedure_id: WireText::parse("proc:found").unwrap(),
                artifact_id: WireText::parse("art:001").unwrap(),
                title: WireText::parse("Title").unwrap(),
                audience_tags: vec![WireText::parse("students").unwrap()],
                board_id: WireText::parse("board:main").unwrap(),
                board_policy_version: 1,
                prerequisites: Vec::new(),
                ordered_steps: vec![ustc_campus_agent_client_protocol::StepDto {
                    ordinal: 0,
                    instruction: WireText::parse("Step 1").unwrap(),
                }],
                deadlines: Vec::new(),
                effective_interval: None,
                entry_points: Vec::new(),
                contacts: Vec::new(),
                evidence: ustc_campus_agent_client_protocol::EvidenceViewDto {
                    valid_interval: ustc_campus_agent_client_protocol::ValidityHorizonDto::Unknown,
                    observed_at: UnixMillis::new(1_700_000_000_000),
                    known_at: UnixMillis::new(1_700_000_000_000),
                    reviewed_at: UnixMillis::new(1_700_000_000_000),
                    last_verified_at: UnixMillis::new(1_700_000_000_000),
                    assessments: Vec::new(),
                    projection: ustc_campus_agent_client_protocol::ProjectionMetadataDto::Complete,
                },
                lookup_path: ustc_campus_agent_client_protocol::LookupPathDto::ExactId,
                conflict_state: ustc_campus_agent_client_protocol::ConflictStateDto::Resolved,
                uncertainty_state: WireText::parse("none").unwrap(),
            }),
            freshness: ustc_campus_agent_client_protocol::FreshnessDto::Fresh,
            as_of: UnixMillis::new(1_700_000_000_000),
        },
        M71LineageDto::Verified {
            materialization_receipt_id: WireText::parse("receipt:001").unwrap(),
            evidence_set_digest: WireText::parse(
                "0000000000000000000000000000000000000000000000000000000000000000",
            )
            .unwrap(),
            revision_count: 1,
            verifier_id: WireText::parse("verifier:m00").unwrap(),
            verified_at: UnixMillis::new(1_700_000_000_000),
            evidence_contract_version: 1,
        },
    )
    .unwrap()
}

// ---------------------------------------------------------------------------
// M71 fixture helpers (adapted from affairs-navigator tests)
// ---------------------------------------------------------------------------

use affairs_navigator::m60_fixture::M60FixtureAdapter;
use affairs_navigator::{
    AffairsAuthority, AffairsAuthorityAssessment, AffairsEvidenceAssessment, AffairsGetQuery,
    AffairsGetService, AuthorityComparison, AuthorityDerivation, AuthoritySubject, BoardId,
    BoardPolicy, BoardPolicyVersion, ConflictKind, Contact as ArtifactContact, ContactChannel,
    ContactName, ContactRef, EntryPoint, EntryPointLabel, EvidenceConflictState, FixedClock,
    InMemoryAffairsRepository, Instruction, M60RevisionRef, ProcedureArtifact,
    ProcedureEvidenceContext, ProcedureId, ProcedurePublicationState, ProcedureStep, Sha256,
    SourceId, Title, UncertaintyState, Url, ValidityHorizon,
};
use time::OffsetDateTime;

pub fn t(secs: i64) -> OffsetDateTime {
    OffsetDateTime::from_unix_timestamp(secs).expect("valid epoch seconds")
}

pub fn digest_str(c: char) -> String {
    std::iter::repeat_n(c, 64).collect()
}

pub fn sha256(c: char) -> Sha256 {
    Sha256::new(format!("sha256:{}", digest_str(c))).expect("valid digest")
}

pub fn rev(source: &str, idx: usize, from: Option<i64>, to: Option<i64>) -> M60RevisionRef {
    M60RevisionRef::new(
        SourceId::parse(source).expect("valid source"),
        format!("rev:{source}:{idx}"),
        t(0),
        None,
        from.map(t),
        to.map(t),
        sha256('0'),
        sha256('1'),
    )
    .expect("valid revision ref")
}

pub fn assessment(
    authority: AffairsAuthority,
    source: &str,
    subject: AuthoritySubject,
) -> AffairsEvidenceAssessment {
    let r = rev(source, 0, None, None);
    let a = AffairsAuthorityAssessment::new(
        authority,
        subject,
        AuthorityDerivation::Direct,
        t(0),
        affairs_navigator::ActorRef::parse("actor:fixture").expect("valid actor"),
    );
    AffairsEvidenceAssessment::new(r, a, t(100), t(100))
}

#[allow(clippy::too_many_arguments)]
pub fn build_artifact(
    procedure_id: &str,
    known_at: i64,
    last_verified_at: i64,
    conflict_state: EvidenceConflictState,
    authority_comparison: AuthorityComparison,
    conflict_kind: Option<ConflictKind>,
    max_fresh: u32,
    max_presentable: u32,
    assessments: Vec<AffairsEvidenceAssessment>,
) -> ProcedureArtifact {
    let evidence = ProcedureEvidenceContext::new(
        ValidityHorizon::Unknown,
        t(0),
        t(known_at),
        t(0),
        t(last_verified_at),
        assessments,
        conflict_state,
        authority_comparison,
        UncertaintyState::None,
        conflict_kind,
        Vec::new(),
    )
    .expect("valid evidence context");

    let board_policy = BoardPolicy::new(
        BoardId::parse("board:fixture").expect("valid board"),
        BoardPolicyVersion::new(1).expect("valid version"),
        max_fresh,
        max_presentable,
    )
    .expect("valid policy");

    let step = ProcedureStep::new(0, Instruction::new("Do step 1").expect("valid instruction"));
    let contact = ArtifactContact::new(
        ContactRef::parse("contact:desk").expect("valid ref"),
        ContactName::new("Desk").expect("valid name"),
        ContactChannel::new("email").expect("valid channel"),
        SourceId::parse("src:desk").expect("valid source"),
    );
    let entry = EntryPoint::new(
        EntryPointLabel::new("Portal").expect("valid label"),
        Url::new("https://example.com").ok(),
        ContactRef::parse("contact:desk").expect("valid ref"),
    );

    ProcedureArtifact::new(
        affairs_navigator::ArtifactId::parse("artifact:fixture:v1").expect("valid id"),
        ProcedureId::parse(procedure_id).expect("valid id"),
        Title::new("Fixture procedure").expect("valid title"),
        vec![affairs_navigator::AudienceTag::new("students").expect("valid tag")],
        board_policy,
        Vec::new(),
        vec![step],
        Vec::new(),
        None,
        vec![entry],
        vec![contact],
        evidence,
        t(known_at),
    )
    .expect("valid artifact")
}

pub fn seed_repo(artifact: ProcedureArtifact) -> InMemoryAffairsRepository {
    let mut repo = InMemoryAffairsRepository::new();
    let state = ProcedurePublicationState::current(
        artifact.procedure_id().clone(),
        artifact.artifact_id().clone(),
    );
    repo.seed(artifact, state).expect("coherent fixture pair");
    repo
}

pub fn m71_service<'a>(
    repo: &'a InMemoryAffairsRepository,
    m60: &'a M60FixtureAdapter,
    clock: &'a FixedClock,
) -> AffairsGetService<'a> {
    AffairsGetService::new(repo, m60, clock)
}

// ---------------------------------------------------------------------------
// M00 FakePorts (adapted from platform-core tests)
// ---------------------------------------------------------------------------

use std::sync::Arc;
use ustc_campus_agent_application_ingress::M10AdmissionPorts;
use ustc_campus_agent_core::identity::{
    CommandId, CorrelationId, RequestId, SessionId, TenantId, UserId,
};
use ustc_campus_agent_core::request_context::*;
use ustc_campus_agent_core::session::{
    AuthAdapterId, CredentialEvidenceDigest, OpenSession, SessionCommand,
    SessionCredentialEvidence, SessionDuration, SessionInstant, SessionPolicy, SessionSnapshot,
    decide, evolve,
};

const DIGEST: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const CREDENTIAL_DIGEST: &str =
    "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

pub fn operation(value: &str) -> OperationId {
    OperationId::parse(value).expect("fixture operation")
}

pub fn policy_id() -> PlatformPolicySnapshotId {
    PlatformPolicySnapshotId::parse("policy:fixture").expect("fixture policy")
}

pub fn request_id() -> RequestId {
    RequestId::parse("request:fixture").expect("fixture request")
}

pub fn command_id() -> CommandId {
    CommandId::parse("command:fixture").expect("fixture command")
}

pub fn correlation_id() -> CorrelationId {
    CorrelationId::parse("correlation:fixture").expect("fixture correlation")
}

pub fn idem_key() -> IdempotencyKey {
    IdempotencyKey::parse("idempotency:fixture").expect("fixture key")
}

pub fn tenant() -> TenantId {
    TenantId::parse("tenant:fixture").expect("fixture tenant")
}

pub fn user() -> UserId {
    UserId::parse("user:fixture").expect("fixture user")
}

pub fn session(value: &str) -> SessionId {
    SessionId::parse(value).expect("fixture session")
}

pub fn at(value: u64) -> SessionInstant {
    SessionInstant::from_unix_millis(value)
}

pub fn schema_digest() -> SchemaDigest {
    SchemaDigest::parse(DIGEST).expect("fixture digest")
}

pub fn descriptor_id() -> DescriptorSnapshotId {
    DescriptorSnapshotId::from_canonical_identity(&schema_digest(), 7)
        .expect("fixture descriptor id")
}

pub fn reservation_token(fence: u64) -> IdempotencyReservationToken {
    IdempotencyReservationToken::from_store_observation(command_id(), 3, fence, at(1_100))
        .expect("fixture token")
}

#[derive(Clone)]
pub struct Descriptor {
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
    pub fn public_read() -> Self {
        Self::new("affairs.get", PermissionClass::PublicRead)
    }
    pub fn tenant_private_write() -> Self {
        Self::new("affairs.get", PermissionClass::TenantPrivateWrite)
    }
    pub fn wrong_operation() -> Self {
        Self::new("wrong.operation", PermissionClass::PublicRead)
    }
    fn new(operation_name: &str, permission_class: PermissionClass) -> Self {
        Self {
            operation_id: operation(operation_name),
            schema_identity: SchemaIdentity::parse("schema:fixture").expect("fixture"),
            schema_digest: schema_digest(),
            permission_class,
            effect_class: if matches!(permission_class, PermissionClass::PublicLinkout) {
                EffectClass::LinkOut
            } else {
                EffectClass::Read
            },
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

pub fn active_session(session_id: SessionId) -> SessionSnapshot {
    let evidence = SessionCredentialEvidence::new(
        tenant(),
        user(),
        AuthAdapterId::parse("fixture.adapter").expect("fixture"),
        CredentialEvidenceDigest::parse(CREDENTIAL_DIGEST).expect("fixture"),
        at(1_000),
        None,
    )
    .expect("fixture");
    let policy = SessionPolicy::new(
        SessionDuration::from_millis(100).expect("fixture"),
        SessionDuration::from_millis(1_000).expect("fixture"),
    );
    let command =
        SessionCommand::Open(OpenSession::new(session_id, evidence, policy, at(1_000), 0));
    let event = decide(None, &command).expect("fixture open");
    evolve(None, &event).expect("fixture evolve")
}

pub fn active_session_with_id(session_id_value: &str) -> SessionSnapshot {
    active_session(session(session_id_value))
}

#[derive(Clone)]
pub struct FakePorts {
    pub reservation: Result<IdempotencyReservation, IdempotencyError>,
    pub descriptor: Result<OperationSnapshot, DescriptorSnapshotError>,
    pub now: Result<SessionInstant, AdmissionPortError>,
    pub policy: Result<PolicyResolution, AdmissionPortError>,
    pub loaded_session: Result<Option<SessionSnapshot>, AdmissionPortError>,
    pub capability: Result<CapabilityDisposition, AdmissionPortError>,
    pub finalize: Result<FinalizeIdempotencyOutcome, IdempotencyError>,
    pub staged: OperationSnapshot,
}

impl FakePorts {
    pub fn public_admitted() -> Self {
        let desc: OperationSnapshot = Arc::new(Descriptor::public_read());
        Self {
            reservation: Ok(IdempotencyReservation::New(reservation_token(1))),
            descriptor: Ok(Arc::clone(&desc)),
            now: Ok(at(1_000)),
            policy: Ok(PolicyResolution::new(
                policy_id(),
                PolicyCurrentnessFact::Current,
            )),
            loaded_session: Ok(None),
            capability: Ok(CapabilityDisposition::Enabled),
            finalize: Ok(FinalizeIdempotencyOutcome::Committed),
            staged: desc,
        }
    }

    pub fn authenticated_admitted(session_id_value: &str) -> Self {
        let desc: OperationSnapshot = Arc::new(Descriptor::public_read());
        Self {
            reservation: Ok(IdempotencyReservation::New(reservation_token(1))),
            descriptor: Ok(Arc::clone(&desc)),
            now: Ok(at(1_000)),
            policy: Ok(PolicyResolution::new(
                policy_id(),
                PolicyCurrentnessFact::Current,
            )),
            loaded_session: Ok(Some(active_session_with_id(session_id_value))),
            capability: Ok(CapabilityDisposition::Enabled),
            finalize: Ok(FinalizeIdempotencyOutcome::Committed),
            staged: desc,
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

impl M10AdmissionPorts for FakePorts {
    fn staged_operation(&self) -> OperationSnapshot {
        Arc::clone(&self.staged)
    }
}

pub fn public_command() -> BuildRequestContextCommand {
    BuildRequestContextCommand::new(
        request_id(),
        operation("affairs.get"),
        ActorReference::Anonymous { scope: PublicScope },
        correlation_id(),
        None,
        None,
        ClientProvenance::new("build:fixture", "linux", "m10:v2").expect("fixture"),
        PayloadDigest::parse(DIGEST).expect("fixture"),
    )
}

pub fn public_command_with_key() -> BuildRequestContextCommand {
    BuildRequestContextCommand::new(
        request_id(),
        operation("affairs.get"),
        ActorReference::Anonymous { scope: PublicScope },
        correlation_id(),
        None,
        Some(idem_key()),
        ClientProvenance::new("build:fixture", "linux", "m10:v2").expect("fixture"),
        PayloadDigest::parse(DIGEST).expect("fixture"),
    )
}

pub fn authenticated_command(session_id_value: &str) -> BuildRequestContextCommand {
    BuildRequestContextCommand::new(
        request_id(),
        operation("affairs.get"),
        ActorReference::Authenticated {
            session_id: session(session_id_value),
        },
        correlation_id(),
        Some(
            ustc_campus_agent_core::request_context::CausationId::parse("causation:fixture")
                .expect("fixture"),
        ),
        Some(idem_key()),
        ClientProvenance::new("build:fixture", "linux", "m10:v2").expect("fixture"),
        PayloadDigest::parse(DIGEST).expect("fixture"),
    )
}

// ---------------------------------------------------------------------------
// M71 mock port
// ---------------------------------------------------------------------------

pub struct FailingM71Port;

impl affairs_navigator::M71AffairsGetPort for FailingM71Port {
    fn affairs_get(
        &self,
        _query: &AffairsGetQuery,
    ) -> Result<affairs_navigator::M71AffairsGetReceipt, affairs_navigator::GetProcedureError> {
        Err(affairs_navigator::GetProcedureError::M60StoreUnavailable)
    }
}

impl ustc_campus_agent_application_ingress::AffairsInvocationPort for FailingM71Port {
    fn invoke(
        &self,
        _actor: &ustc_campus_agent_core::request_context::M00AdmittedActor,
        query: &AffairsGetQuery,
    ) -> Result<
        affairs_navigator::M71AffairsGetReceipt,
        ustc_campus_agent_application_ingress::AffairsInvocationError,
    > {
        affairs_navigator::M71AffairsGetPort::affairs_get(self, query)
            .map_err(ustc_campus_agent_application_ingress::AffairsInvocationError::Downstream)
    }
}

pub struct M71FixturePort<'a> {
    service: AffairsGetService<'a>,
}

impl<'a> M71FixturePort<'a> {
    pub fn new(
        repo: &'a InMemoryAffairsRepository,
        m60: &'a M60FixtureAdapter,
        clock: &'a FixedClock,
    ) -> Self {
        Self {
            service: AffairsGetService::new(repo, m60, clock),
        }
    }
}

impl<'a> affairs_navigator::M71AffairsGetPort for M71FixturePort<'a> {
    fn affairs_get(
        &self,
        query: &AffairsGetQuery,
    ) -> Result<affairs_navigator::M71AffairsGetReceipt, affairs_navigator::GetProcedureError> {
        self.service.execute(query)
    }
}

impl ustc_campus_agent_application_ingress::AffairsInvocationPort for M71FixturePort<'_> {
    fn invoke(
        &self,
        _actor: &ustc_campus_agent_core::request_context::M00AdmittedActor,
        query: &AffairsGetQuery,
    ) -> Result<
        affairs_navigator::M71AffairsGetReceipt,
        ustc_campus_agent_application_ingress::AffairsInvocationError,
    > {
        affairs_navigator::M71AffairsGetPort::affairs_get(self, query)
            .map_err(ustc_campus_agent_application_ingress::AffairsInvocationError::Downstream)
    }
}

pub fn submit_request(
    procedure_id: &str,
) -> ustc_campus_agent_client_protocol::SubmitAffairsGetDto {
    let procedure_id_wire = WireText::parse(procedure_id).unwrap();
    let payload_digest = affairs_get_payload_digest(&procedure_id_wire, None).unwrap();
    ustc_campus_agent_client_protocol::SubmitAffairsGetDto {
        request_id: WireText::parse("req:fixture").unwrap(),
        correlation_id: WireText::parse("corr:fixture").unwrap(),
        causation_id: None,
        idempotency_key: Some(WireText::parse("idem:fixture").unwrap()),
        actor: ustc_campus_agent_client_protocol::ActorIntentDto::Public,
        provenance: ustc_campus_agent_client_protocol::ClientProvenanceDto {
            build: WireText::parse("build:fixture").unwrap(),
            target: WireText::parse("linux").unwrap(),
            protocol: WireText::parse("m10:v2").unwrap(),
        },
        payload_digest,
        procedure_id: procedure_id_wire,
        as_of: None,
    }
}

pub fn submit_request_authenticated(
    procedure_id: &str,
    session_id: &str,
) -> ustc_campus_agent_client_protocol::SubmitAffairsGetDto {
    let procedure_id_wire = WireText::parse(procedure_id).unwrap();
    let payload_digest = affairs_get_payload_digest(&procedure_id_wire, None).unwrap();
    ustc_campus_agent_client_protocol::SubmitAffairsGetDto {
        request_id: WireText::parse("req:fixture").unwrap(),
        correlation_id: WireText::parse("corr:fixture").unwrap(),
        causation_id: None,
        idempotency_key: Some(WireText::parse("idem:fixture").unwrap()),
        actor: ustc_campus_agent_client_protocol::ActorIntentDto::Authenticated {
            session_id: WireText::parse(session_id).unwrap(),
        },
        provenance: ustc_campus_agent_client_protocol::ClientProvenanceDto {
            build: WireText::parse("build:fixture").unwrap(),
            target: WireText::parse("linux").unwrap(),
            protocol: WireText::parse("m10:v2").unwrap(),
        },
        payload_digest,
        procedure_id: procedure_id_wire,
        as_of: None,
    }
}

pub fn cap_issuer() -> ustc_campus_agent_application_ingress::CapabilityIssuer {
    let mut keys = BTreeMap::new();
    keys.insert(1u16, [0xabu8; 32]);
    ustc_campus_agent_application_ingress::CapabilityIssuer::new(keys, 1).unwrap()
}
