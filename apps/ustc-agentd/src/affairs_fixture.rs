//! Bounded fixture module for the agentd composition root. Loads M71/M60
//! fixture data, M00 descriptor/session/policy facts and capability keys from
//! a durable JSON file, and provides a file-backed idempotency store. This is
//! explicitly fixture evidence for the bounded product-path slice, not
//! accepted production implementation.

#![allow(clippy::too_many_arguments)]

use std::collections::BTreeMap;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::os::unix::fs::{MetadataExt, OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use affairs_navigator::m60_fixture::M60FixtureAdapter;
use affairs_navigator::{
    ActorRef, AffairsAuthority, AffairsAuthorityAssessment, AffairsEvidenceAssessment, AudienceTag,
    AuthorityComparison, AuthorityDerivation, AuthoritySubject, BoardId, BoardPolicy,
    BoardPolicyVersion, ConflictKind, Contact as ArtifactContact, ContactChannel, ContactName,
    ContactRef, EntryPoint, EntryPointLabel, EvidenceConflictState, FixedClock,
    InMemoryPublishedAffairsRepository, Instruction, M60EvidencePortError,
    M60ProcedureEvidencePort, M60ProcedurePublicationOutcome, M60ProcedurePublicationPort,
    M60RetainedEvidenceOutcome, M60RetainedEvidenceRequest, Prerequisite, PrerequisiteCondition,
    ProcedureDraft, ProcedureEvidenceContext, ProcedureId, ProcedurePublicationReceipt,
    ProcedurePublicationRepository, ProcedurePublicationService, ProcedureReviewApproval,
    ProcedureStep, SourceId, Title, UncertaintyState, Url, ValidityHorizon,
    m60_ref_from_source_revision,
};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

const MAX_IDEMPOTENCY_STORE_BYTES: u64 = 16 * 1024 * 1024;
const IDEMPOTENCY_TEMP_ATTEMPTS: u32 = 16;
use ustc_campus_agent_application_ingress::{CapabilityIssuer, M10AdmissionPorts};
use ustc_campus_agent_client_protocol::WireText;
use ustc_campus_agent_core::identity::{CommandId, SessionId, TenantId, UserId};
use ustc_campus_agent_core::request_context::{
    ActorKind, AdapterAllowlist, AdapterIdentity, AdmissionPortError, AdmissionPortKind,
    AdmissionPorts, CapabilityDisposition, DecoderIdentity, DescriptorSnapshotError,
    DescriptorSnapshotId, DispatcherIdentity, EffectClass, EnvelopeHash, FinalAdmissionDisposition,
    FinalizeIdempotencyOutcome, IdempotencyError, IdempotencyKey, IdempotencyReservation,
    IdempotencyReservationToken, OperationDescriptorProjection, OperationId, OperationSnapshot,
    PermissionClass, PersistedPriorDispositionDto, PlatformPolicySnapshotId, PolicyCurrentnessFact,
    PolicyResolution, SchemaDigest, SchemaIdentity,
};
use ustc_campus_agent_core::session::{
    AuthAdapterId, CredentialEvidenceDigest, OpenSession, SessionCommand,
    SessionCredentialEvidence, SessionDuration, SessionEvent, SessionInstant, SessionPolicy,
    SessionSnapshot, decide, evolve,
};
use ustc_campus_agent_core::session_port::SessionHistoryReadPort;
use ustc_campus_agent_core::source_registry::{
    SourceId as M60SourceId, SourceReviewEvidenceId, SourceReviewerId, SourceUrl as M60SourceUrl,
};
use ustc_campus_agent_core::source_revision::{
    EffectiveInterval as M60EffectiveInterval, NormalizedSnapshotId, ParserIdentity, RawSnapshotId,
    RevisionSha256, RevisionTimestamp, SourceRevision,
};

use crate::affairs_invocation::AffairsInvocationCounters;
use crate::affairs_persistence::{
    DurablePublishedAffairsRepository, recovery_anchor_and_commit_from_receipt,
};
use crate::m00_session::DurableCurrentSessionStore;

// ---------------------------------------------------------------------------
// Counting M60 port (owner-private call-count instrumentation)
// ---------------------------------------------------------------------------

pub(crate) struct CountingM60Port {
    inner: M60FixtureAdapter,
    call_count: Arc<AtomicU64>,
}

impl CountingM60Port {
    pub(crate) fn new(inner: M60FixtureAdapter, call_count: Arc<AtomicU64>) -> Self {
        Self { inner, call_count }
    }
}

impl M60ProcedureEvidencePort for CountingM60Port {
    fn verify_retained(
        &self,
        request: &M60RetainedEvidenceRequest,
    ) -> Result<M60RetainedEvidenceOutcome, M60EvidencePortError> {
        self.call_count.fetch_add(1, Ordering::SeqCst);
        self.inner.verify_retained(request)
    }
}

impl M60ProcedurePublicationPort for CountingM60Port {
    fn verify_publication(
        &self,
        revision: &SourceRevision,
        request: &M60RetainedEvidenceRequest,
    ) -> Result<M60ProcedurePublicationOutcome, M60EvidencePortError> {
        self.call_count.fetch_add(1, Ordering::SeqCst);
        self.inner.verify_publication(revision, request)
    }
}

// ---------------------------------------------------------------------------
// Fixture DTO (deserialized from JSON)
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct FixtureEntryPointDto {
    label: String,
    url: Option<String>,
    contact_ref: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct FixtureContactDto {
    contact_ref: String,
    name: String,
    channel: String,
    source_id: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct AffairsFixtureDto {
    procedure_id: String,
    title: String,
    #[serde(default)]
    audience_tags: Vec<String>,
    #[serde(default)]
    prerequisites: Vec<String>,
    #[serde(default)]
    steps: Vec<String>,
    #[serde(default)]
    entry_points: Vec<FixtureEntryPointDto>,
    #[serde(default)]
    contacts: Vec<FixtureContactDto>,
    known_at_secs: i64,
    observed_at_secs: i64,
    reviewed_at_secs: i64,
    published_at_secs: i64,
    last_verified_at_secs: i64,
    max_fresh_seconds: u32,
    max_presentable_seconds: u32,
    source_id: String,
    source_url: String,
    raw_snapshot_id: String,
    raw_digest: String,
    normalized_snapshot_id: String,
    normalized_digest: String,
    parser_identity: String,
    source_published_at_secs: Option<i64>,
    source_reviewer: String,
    source_review_evidence: String,
    publication_reviewer: String,
    publication_administrator_tenant_id: Option<String>,
    publication_administrator_user_id: Option<String>,
    publication_administrator_session_id: Option<String>,
    market_enabled: Option<bool>,
    market_grant_active: Option<bool>,
    verifier_id: String,
    evidence_contract_version: u16,
    clock_unix_seconds: i64,
    now_ms: u64,
    session_id: String,
    tenant_id: String,
    user_id: String,
    auth_adapter_id: String,
    credential_evidence_digest: String,
    authenticated_at_ms: u64,
    opened_at_ms: u64,
    idle_timeout_ms: u64,
    absolute_timeout_ms: u64,
    operator_grant_id: String,
    capability_key_hex: String,
    capability_key_version: u16,
    schema_digest: String,
    descriptor_snapshot_version: u64,
    policy_snapshot_id: String,
    idempotency_deadline_ms: u64,
    m60_failure_mode: Option<String>,
    m60_require_effective_interval: Option<bool>,
    conflict_state: Option<String>,
    authority_comparison: Option<String>,
    conflict_kind: Option<String>,
}

// ---------------------------------------------------------------------------
// Loaded fixture — holds built M71/M60/M00 objects
// ---------------------------------------------------------------------------

pub(crate) struct AffairsFixture {
    pub(crate) repo: DurablePublishedAffairsRepository,
    pub(crate) publication_receipt: ProcedurePublicationReceipt,
    pub(crate) publication_draft: ProcedureDraft,
    pub(crate) publication_reviewer: ActorRef,
    pub(crate) publication_reviewed_at: OffsetDateTime,
    pub(crate) publication_published_at: OffsetDateTime,
    pub(crate) publication_descriptor: OperationSnapshot,
    pub(crate) publication_administrator_tenant_id: TenantId,
    pub(crate) publication_administrator_user_id: UserId,
    pub(crate) publication_administrator_session_id: SessionId,
    pub(crate) source_evidence_digest: String,
    pub(crate) market_enabled: bool,
    pub(crate) market_grant_active: bool,
    pub(crate) invocation_counters: AffairsInvocationCounters,
    pub(crate) m60: CountingM60Port,
    pub(crate) m60_call_count: Arc<AtomicU64>,
    pub(crate) clock: FixedClock,
    pub(crate) session: SessionSnapshot,
    pub(crate) session_events: Vec<SessionEvent>,
    pub(crate) capabilities: CapabilityIssuer,
    pub(crate) descriptor: OperationSnapshot,
    pub(crate) policy_snapshot_id: PlatformPolicySnapshotId,
    pub(crate) now: SessionInstant,
    pub(crate) operator_grant_id: WireText,
    pub(crate) idempotency_deadline_ms: u64,
}

impl AffairsFixture {
    pub(crate) fn load(
        path: &Path,
        publication_path: &Path,
        allow_fresh_publication_bootstrap: bool,
    ) -> Result<Self, String> {
        let bytes = fs::read(path).map_err(|e| format!("fixture read failed: {e}"))?;
        let dto: AffairsFixtureDto =
            serde_json::from_slice(&bytes).map_err(|e| format!("fixture parse failed: {e}"))?;
        Self::build(dto, publication_path, allow_fresh_publication_bootstrap)
    }

    fn build(
        dto: AffairsFixtureDto,
        publication_path: &Path,
        allow_fresh_publication_bootstrap: bool,
    ) -> Result<Self, String> {
        let known_at = OffsetDateTime::from_unix_timestamp(dto.known_at_secs)
            .map_err(|e| format!("known_at_secs invalid: {e}"))?;
        let observed_at = OffsetDateTime::from_unix_timestamp(dto.observed_at_secs)
            .map_err(|e| format!("observed_at_secs invalid: {e}"))?;
        let reviewed_at = OffsetDateTime::from_unix_timestamp(dto.reviewed_at_secs)
            .map_err(|e| format!("reviewed_at_secs invalid: {e}"))?;
        let published_at = OffsetDateTime::from_unix_timestamp(dto.published_at_secs)
            .map_err(|e| format!("published_at_secs invalid: {e}"))?;
        let last_verified_at = OffsetDateTime::from_unix_timestamp(dto.last_verified_at_secs)
            .map_err(|e| format!("last_verified_at_secs invalid: {e}"))?;

        // -- Exact M60-owned DemoReviewed revision and equal-contract ref --
        let source_revision = SourceRevision::demo_reviewed(
            M60SourceId::parse(&dto.source_id)
                .map_err(|e| format!("M60 source_id invalid: {e}"))?,
            M60SourceUrl::parse(&dto.source_url).map_err(|e| format!("source_url invalid: {e}"))?,
            RawSnapshotId::parse(&dto.raw_snapshot_id)
                .map_err(|e| format!("raw_snapshot_id invalid: {e}"))?,
            RevisionSha256::parse(&dto.raw_digest)
                .map_err(|e| format!("raw_digest invalid: {e}"))?,
            NormalizedSnapshotId::parse(&dto.normalized_snapshot_id)
                .map_err(|e| format!("normalized_snapshot_id invalid: {e}"))?,
            RevisionSha256::parse(&dto.normalized_digest)
                .map_err(|e| format!("normalized_digest invalid: {e}"))?,
            ParserIdentity::parse(&dto.parser_identity)
                .map_err(|e| format!("parser_identity invalid: {e}"))?,
            RevisionTimestamp::from_unix_seconds(dto.observed_at_secs),
            dto.source_published_at_secs
                .map(RevisionTimestamp::from_unix_seconds),
            M60EffectiveInterval::new(None, None)
                .map_err(|e| format!("source effective interval invalid: {e}"))?,
            SourceReviewerId::parse(&dto.source_reviewer)
                .map_err(|e| format!("source_reviewer invalid: {e}"))?,
            SourceReviewEvidenceId::parse(&dto.source_review_evidence)
                .map_err(|e| format!("source_review_evidence invalid: {e}"))?,
        );
        let revision_ref = m60_ref_from_source_revision(&source_revision)
            .map_err(|e| format!("source revision projection invalid: {e}"))?;

        // Publication uses a healthy M60 port. Query-only failure injection is
        // applied only after the reviewed artifact has committed.
        let query_failure_mode = dto
            .m60_failure_mode
            .as_deref()
            .map(|mode| match mode {
                "store_unavailable" => Ok(M60EvidencePortError::StoreUnavailable),
                "store_corrupted" => Ok(M60EvidencePortError::StoreCorrupted),
                other => Err(format!("unknown m60_failure_mode: {other}")),
            })
            .transpose()?;
        let mut m60 = M60FixtureAdapter::new(&dto.verifier_id, dto.evidence_contract_version)
            .map_err(|e| format!("m60 adapter invalid: {e}"))?;
        m60.store(revision_ref.clone());

        // -- Evidence assessment --
        let prerequisite_revision_ref = revision_ref.clone();
        let authority_assessment = AffairsAuthorityAssessment::new(
            AffairsAuthority::OfficialBulletin,
            AuthoritySubject::ProcedureTitle,
            AuthorityDerivation::Direct,
            reviewed_at,
            ActorRef::parse("actor:fixture").map_err(|e| format!("actor_ref invalid: {e}"))?,
        );
        let assessment = AffairsEvidenceAssessment::new(
            revision_ref,
            authority_assessment,
            reviewed_at,
            last_verified_at,
        );

        // -- Evidence context --
        let conflict_state = match dto.conflict_state.as_deref() {
            None | Some("no_known_conflict") => EvidenceConflictState::NoKnownConflict,
            Some("unresolved_conflict") => EvidenceConflictState::UnresolvedConflict,
            Some(other) => return Err(format!("unknown conflict_state: {other}")),
        };
        let authority_comparison = match dto.authority_comparison.as_deref() {
            None | Some("equivalent") => AuthorityComparison::Equivalent,
            Some("incomparable") => AuthorityComparison::Incomparable,
            Some(other) => return Err(format!("unknown authority_comparison: {other}")),
        };
        let conflict_kind = match dto.conflict_kind.as_deref() {
            None => None,
            Some("direct_contradiction") => Some(ConflictKind::DirectContradiction),
            Some("overlap_incompatible") => Some(ConflictKind::OverlapIncompatible),
            Some("authority_conflict") => Some(ConflictKind::AuthorityConflict),
            Some(other) => return Err(format!("unknown conflict_kind: {other}")),
        };
        let evidence = ProcedureEvidenceContext::new(
            ValidityHorizon::Unknown,
            observed_at,
            known_at,
            reviewed_at,
            last_verified_at,
            vec![assessment],
            conflict_state,
            authority_comparison,
            UncertaintyState::None,
            conflict_kind,
            Vec::new(),
        )
        .map_err(|e| format!("evidence context invalid: {e}"))?;

        // -- Board policy --
        let board_policy = BoardPolicy::new(
            BoardId::parse("board:fixture").map_err(|e| format!("board_id invalid: {e}"))?,
            BoardPolicyVersion::new(1).map_err(|e| format!("board_policy_version invalid: {e}"))?,
            dto.max_fresh_seconds,
            dto.max_presentable_seconds,
        )
        .map_err(|e| format!("board_policy invalid: {e}"))?;

        // -- Structured publication candidate --
        let procedure_id = ProcedureId::parse(&dto.procedure_id)
            .map_err(|e| format!("procedure_id invalid: {e}"))?;
        let title = Title::new(&dto.title).map_err(|e| format!("title invalid: {e}"))?;
        let audience_tags = if dto.audience_tags.is_empty() {
            vec![AudienceTag::new("students").map_err(|e| format!("audience_tag invalid: {e}"))?]
        } else {
            dto.audience_tags
                .iter()
                .map(|value| {
                    AudienceTag::new(value).map_err(|e| format!("audience_tag invalid: {e}"))
                })
                .collect::<Result<Vec<_>, _>>()?
        };
        let prerequisites = dto
            .prerequisites
            .iter()
            .map(|value| {
                let condition = PrerequisiteCondition::new(value)
                    .map_err(|e| format!("prerequisite invalid: {e}"))?;
                Ok(Prerequisite::new(
                    condition,
                    Some(prerequisite_revision_ref.clone()),
                ))
            })
            .collect::<Result<Vec<_>, String>>()?;
        let steps = if dto.steps.is_empty() {
            vec![ProcedureStep::new(
                0,
                Instruction::new("Do step 1").map_err(|e| format!("instruction invalid: {e}"))?,
            )]
        } else {
            dto.steps
                .iter()
                .enumerate()
                .map(|(index, value)| {
                    let ordinal =
                        u32::try_from(index).map_err(|_| "too many procedure steps".to_owned())?;
                    let instruction =
                        Instruction::new(value).map_err(|e| format!("instruction invalid: {e}"))?;
                    Ok(ProcedureStep::new(ordinal, instruction))
                })
                .collect::<Result<Vec<_>, String>>()?
        };
        let contacts = if dto.contacts.is_empty() {
            vec![ArtifactContact::new(
                ContactRef::parse("contact:desk")
                    .map_err(|e| format!("contact_ref invalid: {e}"))?,
                ContactName::new("Desk").map_err(|e| format!("contact_name invalid: {e}"))?,
                ContactChannel::new("email")
                    .map_err(|e| format!("contact_channel invalid: {e}"))?,
                SourceId::parse("src:desk").map_err(|e| format!("contact_source invalid: {e}"))?,
            )]
        } else {
            dto.contacts
                .iter()
                .map(|value| {
                    Ok(ArtifactContact::new(
                        ContactRef::parse(&value.contact_ref)
                            .map_err(|e| format!("contact_ref invalid: {e}"))?,
                        ContactName::new(&value.name)
                            .map_err(|e| format!("contact_name invalid: {e}"))?,
                        ContactChannel::new(&value.channel)
                            .map_err(|e| format!("contact_channel invalid: {e}"))?,
                        SourceId::parse(&value.source_id)
                            .map_err(|e| format!("contact_source invalid: {e}"))?,
                    ))
                })
                .collect::<Result<Vec<_>, String>>()?
        };
        let entry_points = if dto.entry_points.is_empty() {
            vec![EntryPoint::new(
                EntryPointLabel::new("Portal").map_err(|e| format!("entry_label invalid: {e}"))?,
                Some(
                    Url::new("https://example.com")
                        .map_err(|e| format!("entry_url invalid: {e}"))?,
                ),
                ContactRef::parse("contact:desk").map_err(|e| format!("entry_ref invalid: {e}"))?,
            )]
        } else {
            dto.entry_points
                .iter()
                .map(|value| {
                    let url = value
                        .url
                        .as_deref()
                        .map(Url::new)
                        .transpose()
                        .map_err(|e| format!("entry_url invalid: {e}"))?;
                    Ok(EntryPoint::new(
                        EntryPointLabel::new(&value.label)
                            .map_err(|e| format!("entry_label invalid: {e}"))?,
                        url,
                        ContactRef::parse(&value.contact_ref)
                            .map_err(|e| format!("entry_ref invalid: {e}"))?,
                    ))
                })
                .collect::<Result<Vec<_>, String>>()?
        };
        let draft = ProcedureDraft::from_demo_reviewed(
            source_revision,
            procedure_id,
            title,
            audience_tags,
            board_policy,
            prerequisites,
            steps,
            Vec::new(),
            None,
            entry_points,
            contacts,
            evidence,
        )
        .map_err(|e| format!("procedure draft invalid: {e:?}"))?;
        let publication_draft = draft.clone();
        let publication_reviewer = ActorRef::parse(&dto.publication_reviewer)
            .map_err(|e| format!("publication_reviewer invalid: {e}"))?;
        let approval = ProcedureReviewApproval::new(
            draft.draft_digest().clone(),
            publication_reviewer.clone(),
            reviewed_at,
        );
        let mut bootstrap_repo = InMemoryPublishedAffairsRepository::new();
        let mut publication_receipt = ProcedurePublicationService::new(&mut bootstrap_repo, &m60)
            .publish(draft, approval, published_at, None)
            .map_err(|e| format!("reviewed publication failed: {e:?}"))?;
        let source_evidence_digest = publication_receipt
            .m60_evidence_set_digest()
            .as_str()
            .to_owned();

        if let Some(error) = query_failure_mode {
            m60.set_failure_mode(Some(error));
        }
        if let Some(require) = dto.m60_require_effective_interval {
            m60.require_effective_interval(require);
        }

        // -- Clock --
        let clock = FixedClock::new(
            OffsetDateTime::from_unix_timestamp(dto.clock_unix_seconds)
                .map_err(|e| format!("clock_unix_seconds invalid: {e}"))?,
        );

        // -- Session snapshot --
        let tenant_id =
            TenantId::parse(&dto.tenant_id).map_err(|e| format!("tenant_id invalid: {e}"))?;
        let user_id = UserId::parse(&dto.user_id).map_err(|e| format!("user_id invalid: {e}"))?;
        let session_id =
            SessionId::parse(&dto.session_id).map_err(|e| format!("session_id invalid: {e}"))?;
        let publication_administrator_tenant_id = TenantId::parse(
            dto.publication_administrator_tenant_id
                .as_deref()
                .unwrap_or(&dto.tenant_id),
        )
        .map_err(|e| format!("publication administrator tenant invalid: {e}"))?;
        let publication_administrator_user_id = UserId::parse(
            dto.publication_administrator_user_id
                .as_deref()
                .unwrap_or(&dto.user_id),
        )
        .map_err(|e| format!("publication administrator user invalid: {e}"))?;
        let publication_administrator_session_id = SessionId::parse(
            dto.publication_administrator_session_id
                .as_deref()
                .unwrap_or(&dto.session_id),
        )
        .map_err(|e| format!("publication administrator session invalid: {e}"))?;
        let auth_adapter_id = AuthAdapterId::parse(&dto.auth_adapter_id)
            .map_err(|e| format!("auth_adapter_id invalid: {e}"))?;
        let credential_digest = CredentialEvidenceDigest::parse(&dto.credential_evidence_digest)
            .map_err(|e| format!("credential_evidence_digest invalid: {e}"))?;
        let authenticated_at = SessionInstant::from_unix_millis(dto.authenticated_at_ms);
        let opened_at = SessionInstant::from_unix_millis(dto.opened_at_ms);
        let idle_timeout = SessionDuration::from_millis(dto.idle_timeout_ms)
            .map_err(|e| format!("idle_timeout_ms invalid: {e}"))?;
        let absolute_timeout = SessionDuration::from_millis(dto.absolute_timeout_ms)
            .map_err(|e| format!("absolute_timeout_ms invalid: {e}"))?;
        let cred_evidence = SessionCredentialEvidence::new(
            tenant_id,
            user_id,
            auth_adapter_id,
            credential_digest,
            authenticated_at,
            None,
        )
        .map_err(|e| format!("credential evidence invalid: {e}"))?;
        let policy = SessionPolicy::new(idle_timeout, absolute_timeout);
        let open_command = SessionCommand::Open(OpenSession::new(
            session_id,
            cred_evidence,
            policy,
            opened_at,
            0,
        ));
        let open_event =
            decide(None, &open_command).map_err(|e| format!("session decide failed: {e}"))?;
        let session =
            evolve(None, &open_event).map_err(|e| format!("session evolve failed: {e}"))?;
        let session_events = vec![open_event];

        // -- Capability issuer --
        let key_bytes = decode_hex_key(&dto.capability_key_hex)?;
        let mut keys = BTreeMap::new();
        keys.insert(dto.capability_key_version, key_bytes);
        let capabilities = CapabilityIssuer::new(keys, dto.capability_key_version)
            .map_err(|e| format!("capability issuer invalid: {e:?}"))?;

        // -- Descriptor --
        let schema_digest = SchemaDigest::parse(&dto.schema_digest)
            .map_err(|e| format!("schema_digest invalid: {e}"))?;
        let descriptor_snapshot_id = DescriptorSnapshotId::from_canonical_identity(
            &schema_digest,
            dto.descriptor_snapshot_version,
        )
        .map_err(|e| format!("descriptor_snapshot_id invalid: {e}"))?;
        let descriptor = FixtureDescriptor {
            operation_id: OperationId::parse("affairs.get")
                .map_err(|e| format!("operation_id invalid: {e}"))?,
            schema_identity: SchemaIdentity::parse("schema:fixture")
                .map_err(|e| format!("schema_identity invalid: {e}"))?,
            schema_digest,
            permission_class: PermissionClass::PublicRead,
            effect_class: EffectClass::Read,
            decoder_identity: DecoderIdentity::parse("decoder:fixture")
                .map_err(|e| format!("decoder_identity invalid: {e}"))?,
            dispatcher_identity: DispatcherIdentity::parse("dispatcher:fixture")
                .map_err(|e| format!("dispatcher_identity invalid: {e}"))?,
            adapter_allowlist: AdapterAllowlist::try_from_iter([AdapterIdentity::parse(
                "adapter:fixture",
            )
            .map_err(|e| format!("adapter_identity invalid: {e}"))?])
            .map_err(|e| format!("adapter_allowlist invalid: {e:?}"))?,
            snapshot_identity: descriptor_snapshot_id,
        };
        let descriptor_snapshot: OperationSnapshot = Arc::new(descriptor);
        let publication_schema_digest = SchemaDigest::parse("b".repeat(64))
            .map_err(|e| format!("publication schema_digest invalid: {e}"))?;
        let publication_snapshot_id = DescriptorSnapshotId::from_canonical_identity(
            &publication_schema_digest,
            dto.descriptor_snapshot_version
                .checked_add(1)
                .ok_or_else(|| "publication descriptor version overflow".to_owned())?,
        )
        .map_err(|e| format!("publication descriptor_snapshot_id invalid: {e}"))?;
        let publication_descriptor: OperationSnapshot = Arc::new(FixtureDescriptor {
            operation_id: OperationId::parse("affairs.publish")
                .map_err(|e| format!("publication operation_id invalid: {e}"))?,
            schema_identity: SchemaIdentity::parse("schema:fixture-affairs-publication")
                .map_err(|e| format!("publication schema_identity invalid: {e}"))?,
            schema_digest: publication_schema_digest,
            permission_class: PermissionClass::TenantPrivateWrite,
            effect_class: EffectClass::TenantLocalMutation,
            decoder_identity: DecoderIdentity::parse("decoder:fixture-affairs-publication")
                .map_err(|e| format!("publication decoder_identity invalid: {e}"))?,
            dispatcher_identity: DispatcherIdentity::parse(
                "dispatcher:fixture-affairs-publication",
            )
            .map_err(|e| format!("publication dispatcher_identity invalid: {e}"))?,
            adapter_allowlist: AdapterAllowlist::try_from_iter([AdapterIdentity::parse(
                "adapter:fixture-affairs-publication",
            )
            .map_err(|e| format!("publication adapter_identity invalid: {e}"))?])
            .map_err(|e| format!("publication adapter_allowlist invalid: {e:?}"))?,
            snapshot_identity: publication_snapshot_id,
        });

        // -- Policy snapshot ID --
        let policy_snapshot_id = PlatformPolicySnapshotId::parse(&dto.policy_snapshot_id)
            .map_err(|e| format!("policy_snapshot_id invalid: {e}"))?;

        // -- Now --
        let now = SessionInstant::from_unix_millis(dto.now_ms);

        let operator_grant_id = WireText::parse(&dto.operator_grant_id)
            .map_err(|e| format!("operator_grant_id invalid: {e}"))?;

        let m60_call_count = Arc::new(AtomicU64::new(0));
        let m60 = CountingM60Port::new(m60, m60_call_count.clone());

        let (recovery_anchor, initial_commit) =
            recovery_anchor_and_commit_from_receipt(&publication_draft, &publication_receipt)?;
        let mut repo = DurablePublishedAffairsRepository::open(
            publication_path,
            publication_draft.clone(),
            recovery_anchor,
            allow_fresh_publication_bootstrap,
        )?;
        if repo.record_count() == 0 {
            repo.apply_publication(initial_commit)
                .map_err(|error| format!("initial durable publication failed: {error:?}"))?;
        }
        if repo
            .find_publication_replay(publication_receipt.receipt_id())
            .map_err(|error| format!("initial durable publication lookup failed: {error:?}"))?
            != Some(publication_receipt.clone())
        {
            return Err("durable publication state lost the fixture revision-1 receipt".to_owned());
        }
        publication_receipt = repo
            .latest_receipt()
            .map_err(|error| format!("latest durable publication lookup failed: {error:?}"))?
            .ok_or_else(|| "durable publication state has no latest receipt".to_owned())?;

        Ok(Self {
            repo,
            publication_receipt,
            publication_draft,
            publication_reviewer,
            publication_reviewed_at: reviewed_at,
            publication_published_at: published_at,
            publication_descriptor,
            publication_administrator_tenant_id,
            publication_administrator_user_id,
            publication_administrator_session_id,
            source_evidence_digest,
            market_enabled: dto.market_enabled.unwrap_or(true),
            market_grant_active: dto.market_grant_active.unwrap_or(true),
            invocation_counters: AffairsInvocationCounters::default(),
            m60,
            m60_call_count,
            clock,
            session,
            session_events,
            capabilities,
            descriptor: descriptor_snapshot,
            policy_snapshot_id,
            now,
            operator_grant_id,
            idempotency_deadline_ms: dto.idempotency_deadline_ms,
        })
    }
}

// ---------------------------------------------------------------------------
// Hex helpers
// ---------------------------------------------------------------------------

fn decode_hex_key(hex: &str) -> Result<[u8; 32], String> {
    let bytes = hex.as_bytes();
    if bytes.len() != 64 {
        return Err(format!(
            "capability_key_hex must be 64 hex chars, got {}",
            bytes.len()
        ));
    }
    let mut key = [0u8; 32];
    for (i, byte) in key.iter_mut().enumerate() {
        let hi = hex_digit(bytes[i * 2])?;
        let lo = hex_digit(bytes[i * 2 + 1])?;
        *byte = (hi << 4) | lo;
    }
    Ok(key)
}

fn hex_digit(byte: u8) -> Result<u8, String> {
    match byte {
        b'0'..=b'9' => Ok(byte - b'0'),
        b'a'..=b'f' => Ok(byte - b'a' + 10),
        b'A'..=b'F' => Ok(byte - b'A' + 10),
        _ => Err(format!("invalid hex digit: {}", char::from(byte))),
    }
}

// ---------------------------------------------------------------------------
// Fixture descriptor — fixed OperationDescriptorProjection
// ---------------------------------------------------------------------------

pub(crate) struct FixtureDescriptor {
    pub(crate) operation_id: OperationId,
    pub(crate) schema_identity: SchemaIdentity,
    pub(crate) schema_digest: SchemaDigest,
    pub(crate) permission_class: PermissionClass,
    pub(crate) effect_class: EffectClass,
    pub(crate) decoder_identity: DecoderIdentity,
    pub(crate) dispatcher_identity: DispatcherIdentity,
    pub(crate) adapter_allowlist: AdapterAllowlist,
    pub(crate) snapshot_identity: DescriptorSnapshotId,
}

impl OperationDescriptorProjection for FixtureDescriptor {
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

// ---------------------------------------------------------------------------
// Durable idempotency store — file-backed, Arc<Mutex>, atomic replace
// ---------------------------------------------------------------------------

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct IdempotencyEntry {
    reservation_version: u64,
    fencing_token: u64,
    deadline_ms: u64,
    in_flight: bool,
    disposition: Option<PersistedPriorDispositionDto>,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct IdempotencyState {
    schema_version: u8,
    entries: BTreeMap<String, IdempotencyEntry>,
    key_index: BTreeMap<String, String>,
    #[serde(default)]
    next_unkeyed_sequence: u64,
}

impl Default for IdempotencyState {
    fn default() -> Self {
        Self {
            schema_version: 1,
            entries: BTreeMap::new(),
            key_index: BTreeMap::new(),
            next_unkeyed_sequence: 0,
        }
    }
}

fn validate_idempotency_state(state: &IdempotencyState) -> Result<(), String> {
    for (key, entry) in &state.entries {
        CommandId::parse(key).map_err(|e| format!("invalid entry key `{key}`: {e}"))?;
        if entry.reservation_version == 0 {
            return Err(format!("entry `{key}` has zero reservation_version"));
        }
        if entry.fencing_token == 0 {
            return Err(format!("entry `{key}` has zero fencing_token"));
        }
        if entry.deadline_ms == 0 {
            return Err(format!("entry `{key}` has zero deadline_ms"));
        }
        let has_disposition = entry.disposition.is_some();
        if entry.in_flight && has_disposition {
            return Err(format!(
                "entry `{key}` has in_flight=true with a terminal disposition"
            ));
        }
        if !entry.in_flight && !has_disposition {
            return Err(format!(
                "entry `{key}` has in_flight=false without a terminal disposition"
            ));
        }
    }
    for (key, value) in &state.key_index {
        IdempotencyKey::parse(key).map_err(|e| format!("invalid key_index key `{key}`: {e}"))?;
        CommandId::parse(value)
            .map_err(|e| format!("invalid key_index value `{value}` for key `{key}`: {e}"))?;
        if !state.entries.contains_key(value) {
            return Err(format!(
                "key_index key `{key}` references absent entry `{value}`"
            ));
        }
    }
    Ok(())
}

fn idempotency_current_uid() -> Result<u32, String> {
    crate::unix_identity::effective_uid()
        .map_err(|error| format!("idempotency current uid failed: {error}"))
}

fn ensure_secure_idempotency_parent(path: &Path) -> Result<(), String> {
    crate::durable_path::ensure_secure_parent(path, true)
        .map_err(|error| format!("idempotency parent: {error}"))
}

fn read_existing_idempotency_state(path: &Path) -> Result<Option<Vec<u8>>, String> {
    crate::durable_path::ensure_secure_parent(path, false)
        .map_err(|error| format!("idempotency parent: {error}"))?;
    let path_metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(format!("idempotency metadata failed: {error}")),
    };
    if !path_metadata.file_type().is_file() {
        return Err("idempotency state path is not a regular file".to_owned());
    }
    if path_metadata.permissions().mode() & 0o7777 != 0o600
        || path_metadata.uid() != idempotency_current_uid()?
        || path_metadata.nlink() != 1
    {
        return Err("idempotency state mode must be 0600".to_owned());
    }
    if path_metadata.len() > MAX_IDEMPOTENCY_STORE_BYTES {
        return Err("idempotency state exceeds byte cap".to_owned());
    }

    let mut file = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW)
        .open(path)
        .map_err(|error| format!("idempotency open failed: {error}"))?;
    let opened_metadata = file
        .metadata()
        .map_err(|error| format!("idempotency opened metadata failed: {error}"))?;
    if !opened_metadata.file_type().is_file()
        || opened_metadata.dev() != path_metadata.dev()
        || opened_metadata.ino() != path_metadata.ino()
        || opened_metadata.permissions().mode() & 0o7777 != 0o600
        || opened_metadata.uid() != idempotency_current_uid()?
        || opened_metadata.nlink() != 1
        || opened_metadata.len() > MAX_IDEMPOTENCY_STORE_BYTES
    {
        return Err("idempotency state identity or mode changed during open".to_owned());
    }

    let mut bytes = Vec::new();
    std::io::Read::by_ref(&mut file)
        .take(MAX_IDEMPOTENCY_STORE_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| format!("idempotency read failed: {error}"))?;
    if bytes.len() as u64 > MAX_IDEMPOTENCY_STORE_BYTES {
        return Err("idempotency state exceeds byte cap".to_owned());
    }
    Ok(Some(bytes))
}

#[derive(Clone)]
pub(crate) struct DurableIdempotencyStore {
    path: PathBuf,
    state: Arc<Mutex<IdempotencyState>>,
    now_ms: u64,
    deadline_duration_ms: u64,
    fail_next_parent_sync_after_rename: Arc<Mutex<bool>>,
}

impl DurableIdempotencyStore {
    pub(crate) fn open(
        path: &Path,
        now_ms: u64,
        deadline_duration_ms: u64,
    ) -> Result<Self, String> {
        if deadline_duration_ms == 0 {
            return Err("idempotency_deadline_ms must be nonzero".to_owned());
        }
        ensure_secure_idempotency_parent(path)?;
        let state = if let Some(bytes) = read_existing_idempotency_state(path)? {
            let state: IdempotencyState = serde_json::from_slice(&bytes)
                .map_err(|e| format!("idempotency parse failed: {e}"))?;
            if state.schema_version != 1 {
                return Err(format!(
                    "idempotency schema_version mismatch: expected 1, got {}",
                    state.schema_version
                ));
            }
            validate_idempotency_state(&state)?;
            let canonical = serde_json::to_vec(&state)
                .map_err(|error| format!("idempotency encode failed: {error}"))?;
            if canonical != bytes {
                return Err("idempotency state is not canonical JSON".to_owned());
            }
            state
        } else {
            IdempotencyState::default()
        };
        Ok(Self {
            path: path.to_owned(),
            state: Arc::new(Mutex::new(state)),
            now_ms,
            deadline_duration_ms,
            fail_next_parent_sync_after_rename: Arc::new(Mutex::new(false)),
        })
    }

    pub(crate) fn open_for_state_set(
        path: &Path,
        now_ms: u64,
        deadline_duration_ms: u64,
        bootstrap_is_fresh: bool,
    ) -> Result<Self, String> {
        let existed = match fs::symlink_metadata(path) {
            Ok(_) => true,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => false,
            Err(error) => return Err(format!("idempotency state metadata failed: {error}")),
        };
        if !existed && !bootstrap_is_fresh {
            return Err("idempotency state missing from non-fresh state set".to_owned());
        }
        let store = Self::open(path, now_ms, deadline_duration_ms)?;
        if !existed {
            store.persist(&IdempotencyState::default())?;
        }
        Ok(store)
    }

    fn persist(&self, state: &IdempotencyState) -> Result<(), String> {
        ensure_secure_idempotency_parent(&self.path)?;
        let bytes =
            serde_json::to_vec(state).map_err(|e| format!("idempotency encode failed: {e}"))?;
        if bytes.len() as u64 > MAX_IDEMPOTENCY_STORE_BYTES {
            return Err("idempotency state exceeds byte cap".to_owned());
        }
        let parent = self
            .path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
            .ok_or_else(|| "idempotency state path has no parent".to_owned())?;
        let file_name = self
            .path
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| "idempotency state path has no UTF-8 file name".to_owned())?;
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| "system time precedes Unix epoch".to_owned())?
            .as_nanos();
        let mut temporary = None;
        let mut opened = None;
        for attempt in 0..IDEMPOTENCY_TEMP_ATTEMPTS {
            let candidate = parent.join(format!(
                ".{file_name}.{}.{}.{}.tmp",
                std::process::id(),
                nonce,
                attempt
            ));
            match OpenOptions::new()
                .write(true)
                .create_new(true)
                .mode(0o600)
                .custom_flags(libc::O_NOFOLLOW)
                .open(&candidate)
            {
                Ok(file) => {
                    temporary = Some(candidate);
                    opened = Some(file);
                    break;
                }
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(error) => {
                    return Err(format!("idempotency temp create failed: {error}"));
                }
            }
        }
        let temporary =
            temporary.ok_or_else(|| "idempotency temp create attempts exhausted".to_owned())?;
        let mut opened =
            opened.ok_or_else(|| "idempotency temp create attempts exhausted".to_owned())?;
        let mut temporary_created = true;
        let mut renamed = false;
        let result = (|| {
            opened
                .set_permissions(fs::Permissions::from_mode(0o600))
                .map_err(|e| format!("idempotency temp chmod failed: {e}"))?;
            opened
                .write_all(&bytes)
                .map_err(|e| format!("idempotency write failed: {e}"))?;
            opened
                .sync_all()
                .map_err(|e| format!("idempotency sync failed: {e}"))?;
            drop(opened);
            fs::rename(&temporary, &self.path)
                .map_err(|e| format!("idempotency rename failed: {e}"))?;
            renamed = true;
            temporary_created = false;
            if let Some(parent) = self
                .path
                .parent()
                .filter(|parent| !parent.as_os_str().is_empty())
            {
                let injected = {
                    let mut failure = self
                        .fail_next_parent_sync_after_rename
                        .lock()
                        .map_err(|_| "idempotency failure injection poisoned".to_owned())?;
                    let injected = *failure;
                    *failure = false;
                    injected
                };
                let parent_sync = if injected {
                    Err(std::io::Error::other(
                        "injected idempotency parent sync failure",
                    ))
                } else {
                    File::open(parent).and_then(|directory| directory.sync_all())
                };
                parent_sync.map_err(|e| format!("idempotency parent sync failed: {e}"))?;
            }
            Ok(())
        })();
        if result.is_err() {
            if renamed
                && matches!(read_existing_idempotency_state(&self.path), Ok(Some(actual)) if actual == bytes)
            {
                return Ok(());
            }
            if temporary_created {
                let _ = fs::remove_file(&temporary);
            }
        }
        result
    }

    #[cfg(test)]
    fn fail_next_parent_sync_after_rename_for_test(&self) {
        *self
            .fail_next_parent_sync_after_rename
            .lock()
            .expect("idempotency failure injection lock") = true;
    }

    pub(crate) fn reserve_or_retrieve(
        &self,
        key: Option<&IdempotencyKey>,
        envelope_hash: &EnvelopeHash,
    ) -> Result<IdempotencyReservation, IdempotencyError> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| IdempotencyError::StoreUnavailable)?;

        let mut next_unkeyed_sequence = None;
        let command_id = if key.is_some() {
            CommandId::parse(envelope_hash.as_str())
                .map_err(|_| IdempotencyError::StoreUnavailable)?
        } else {
            let mut sequence = state.next_unkeyed_sequence;
            loop {
                sequence = sequence
                    .checked_add(1)
                    .ok_or(IdempotencyError::StoreUnavailable)?;
                let candidate =
                    CommandId::parse(format!("{}-{sequence:016x}", envelope_hash.as_str()))
                        .map_err(|_| IdempotencyError::StoreUnavailable)?;
                if !state.entries.contains_key(candidate.as_str()) {
                    next_unkeyed_sequence = Some(sequence);
                    break candidate;
                }
            }
        };

        // Check idempotency key conflict
        if let Some(key) = key
            && let Some(existing) = state.key_index.get(key.as_str())
            && existing != command_id.as_str()
        {
            return Err(IdempotencyError::ConflictingEnvelope {
                idempotency_key: key.clone(),
            });
        }

        // Look up existing entry
        if let Some(entry) = state.entries.get(command_id.as_str()) {
            // Has terminal disposition → prior identical (read-only)
            if let Some(disposition) = &entry.disposition {
                return Ok(IdempotencyReservation::PriorIdentical(disposition.clone()));
            }
            // In flight and not expired (read-only)
            if entry.deadline_ms >= self.now_ms {
                let token = IdempotencyReservationToken::from_store_observation(
                    command_id.clone(),
                    entry.reservation_version,
                    entry.fencing_token,
                    SessionInstant::from_unix_millis(entry.deadline_ms),
                )
                .map_err(|_| IdempotencyError::StoreUnavailable)?;
                return Ok(IdempotencyReservation::InFlight(token));
            }
            // Expired — reclaim: clone, mutate clone, persist, publish
            let new_fencing = entry
                .fencing_token
                .checked_add(1)
                .ok_or(IdempotencyError::StoreUnavailable)?;
            let new_version = entry
                .reservation_version
                .checked_add(1)
                .ok_or(IdempotencyError::StoreUnavailable)?;
            let new_deadline = self
                .now_ms
                .checked_add(self.deadline_duration_ms)
                .ok_or(IdempotencyError::StoreUnavailable)?;
            let mut candidate = state.clone();
            let candidate_entry = candidate
                .entries
                .get_mut(command_id.as_str())
                .ok_or(IdempotencyError::StoreUnavailable)?;
            candidate_entry.fencing_token = new_fencing;
            candidate_entry.reservation_version = new_version;
            candidate_entry.deadline_ms = new_deadline;
            if let Some(key) = key {
                candidate
                    .key_index
                    .insert(key.as_str().to_owned(), command_id.as_str().to_owned());
            }
            self.persist(&candidate)
                .map_err(|_| IdempotencyError::StoreUnavailable)?;
            *state = candidate;
            let token = IdempotencyReservationToken::from_store_observation(
                command_id.clone(),
                new_version,
                new_fencing,
                SessionInstant::from_unix_millis(new_deadline),
            )
            .map_err(|_| IdempotencyError::StoreUnavailable)?;
            return Ok(IdempotencyReservation::Reclaimed(token));
        }

        // Create new entry: clone, mutate clone, persist, publish
        let fencing_token: u64 = 1;
        let reservation_version: u64 = 1;
        let deadline = self
            .now_ms
            .checked_add(self.deadline_duration_ms)
            .ok_or(IdempotencyError::StoreUnavailable)?;
        let mut candidate = state.clone();
        if let Some(sequence) = next_unkeyed_sequence {
            candidate.next_unkeyed_sequence = sequence;
        }
        candidate.entries.insert(
            command_id.as_str().to_owned(),
            IdempotencyEntry {
                reservation_version,
                fencing_token,
                deadline_ms: deadline,
                in_flight: true,
                disposition: None,
            },
        );
        if let Some(key) = key {
            candidate
                .key_index
                .insert(key.as_str().to_owned(), command_id.as_str().to_owned());
        }
        self.persist(&candidate)
            .map_err(|_| IdempotencyError::StoreUnavailable)?;
        *state = candidate;

        let token = IdempotencyReservationToken::from_store_observation(
            command_id.clone(),
            reservation_version,
            fencing_token,
            SessionInstant::from_unix_millis(deadline),
        )
        .map_err(|_| IdempotencyError::StoreUnavailable)?;
        Ok(IdempotencyReservation::New(token))
    }

    pub(crate) fn finalize(
        &self,
        token: &IdempotencyReservationToken,
        disposition: &FinalAdmissionDisposition,
    ) -> Result<FinalizeIdempotencyOutcome, IdempotencyError> {
        self.finalize_with_projection(token, disposition.to_persisted_projection())
    }

    fn finalize_with_projection(
        &self,
        token: &IdempotencyReservationToken,
        projection: PersistedPriorDispositionDto,
    ) -> Result<FinalizeIdempotencyOutcome, IdempotencyError> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| IdempotencyError::StoreUnavailable)?;

        let Some(entry) = state.entries.get(token.command_id().as_str()) else {
            return Ok(FinalizeIdempotencyOutcome::LostReservation(token.clone()));
        };

        if entry.fencing_token != token.fencing_token().get()
            || entry.reservation_version != token.reservation_version()
        {
            return Ok(FinalizeIdempotencyOutcome::LostReservation(token.clone()));
        }

        if let Some(existing) = &entry.disposition {
            if &projection == existing {
                return Ok(FinalizeIdempotencyOutcome::AlreadySame(existing.clone()));
            }
            return Ok(FinalizeIdempotencyOutcome::LostReservation(token.clone()));
        }

        let mut candidate = state.clone();
        let candidate_entry = candidate
            .entries
            .get_mut(token.command_id().as_str())
            .ok_or(IdempotencyError::StoreUnavailable)?;
        candidate_entry.disposition = Some(projection);
        candidate_entry.in_flight = false;
        self.persist(&candidate)
            .map_err(|_| IdempotencyError::StoreUnavailable)?;
        *state = candidate;

        Ok(FinalizeIdempotencyOutcome::Committed)
    }
}

// ---------------------------------------------------------------------------
// Fixture ports — AdmissionPorts + M10AdmissionPorts
// ---------------------------------------------------------------------------

pub(crate) struct FixturePorts {
    store: DurableIdempotencyStore,
    descriptor: OperationSnapshot,
    now: SessionInstant,
    policy_snapshot_id: PlatformPolicySnapshotId,
    sessions: DurableCurrentSessionStore,
    capability: CapabilityDisposition,
}

impl FixturePorts {
    pub(crate) fn new(
        store: DurableIdempotencyStore,
        descriptor: OperationSnapshot,
        now: SessionInstant,
        policy_snapshot_id: PlatformPolicySnapshotId,
        sessions: DurableCurrentSessionStore,
    ) -> Self {
        Self {
            store,
            descriptor,
            now,
            policy_snapshot_id,
            sessions,
            capability: CapabilityDisposition::Enabled,
        }
    }

    pub(crate) fn with_capability(mut self, capability: CapabilityDisposition) -> Self {
        self.capability = capability;
        self
    }
}

impl AdmissionPorts for FixturePorts {
    fn reserve_or_retrieve_idempotency(
        &mut self,
        key: Option<&IdempotencyKey>,
        envelope_hash: &EnvelopeHash,
    ) -> Result<IdempotencyReservation, IdempotencyError> {
        self.store.reserve_or_retrieve(key, envelope_hash)
    }

    fn request_scoped_operation(&mut self) -> Result<OperationSnapshot, DescriptorSnapshotError> {
        Ok(Arc::clone(&self.descriptor))
    }

    fn now(&mut self) -> Result<SessionInstant, AdmissionPortError> {
        Ok(self.now)
    }

    fn resolve_policy(
        &mut self,
        _operation_id: &OperationId,
        _observed_at: SessionInstant,
    ) -> Result<PolicyResolution, AdmissionPortError> {
        Ok(PolicyResolution::new(
            self.policy_snapshot_id.clone(),
            PolicyCurrentnessFact::Current,
        ))
    }

    fn load_session(
        &mut self,
        session_id: &SessionId,
    ) -> Result<Option<SessionSnapshot>, AdmissionPortError> {
        self.sessions
            .load_history(session_id)
            .map(|history| history.map(|retained| retained.snapshot().clone()))
            .map_err(|_| AdmissionPortError::Unavailable(AdmissionPortKind::Session))
    }

    fn check_capability(
        &mut self,
        _operation_id: &OperationId,
        _actor_kind: ActorKind,
        _observed_at: SessionInstant,
    ) -> Result<CapabilityDisposition, AdmissionPortError> {
        Ok(self.capability)
    }

    fn finalize_idempotency(
        &mut self,
        token: &IdempotencyReservationToken,
        disposition: &FinalAdmissionDisposition,
    ) -> Result<FinalizeIdempotencyOutcome, IdempotencyError> {
        self.store.finalize(token, disposition)
    }
}

impl M10AdmissionPorts for FixturePorts {
    fn staged_operation(&self) -> OperationSnapshot {
        Arc::clone(&self.descriptor)
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;
    use std::fs;
    use std::io;
    use std::os::unix::fs::{PermissionsExt, symlink};
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};

    static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

    fn test_dir(label: &str) -> PathBuf {
        let id = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
        let dir = std::env::temp_dir().join(format!(
            "agentd-fixture-test-{}-{id}-{label}",
            std::process::id()
        ));
        fs::create_dir_all(&dir).unwrap();
        fs::set_permissions(&dir, fs::Permissions::from_mode(0o700)).unwrap();
        dir
    }

    fn valid_state_json() -> String {
        let cmd = "abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789";
        let value = serde_json::json!({
            "schema_version": 1,
            "entries": {
                cmd: {
                    "reservation_version": 1,
                    "fencing_token": 1,
                    "deadline_ms": 1000000002000_u64,
                    "in_flight": false,
                    "disposition": {
                        "kind": "rejected",
                        "value": {
                            "kind": "malformed_command",
                            "operation_id": null
                        }
                    }
                }
            },
            "key_index": {
                "idem:valid": cmd
            }
        });
        let state: IdempotencyState =
            serde_json::from_value(value).expect("valid idempotency fixture");
        serde_json::to_string(&state).expect("canonical idempotency fixture")
    }

    fn write_secure(path: &Path, bytes: impl AsRef<[u8]>) -> io::Result<()> {
        fs::write(path, bytes)?;
        let mut permissions = fs::metadata(path)?.permissions();
        permissions.set_mode(0o600);
        fs::set_permissions(path, permissions)
    }

    #[test]
    fn valid_persisted_state_reopens() {
        let dir = test_dir("valid-reopen");
        let path = dir.join("store.json");
        write_secure(&path, valid_state_json()).unwrap();
        let store = DurableIdempotencyStore::open(&path, 1000000001000, 30000);
        assert!(store.is_ok(), "valid state must reopen");
    }

    #[test]
    fn noncanonical_persisted_state_fails_closed() {
        let dir = test_dir("noncanonical-reopen");
        let path = dir.join("store.json");
        write_secure(&path, format!("{}\n", valid_state_json())).expect("write noncanonical state");
        let store = DurableIdempotencyStore::open(&path, 1_000_000_001_000, 30_000);
        assert!(store.is_err(), "noncanonical state must fail closed");
    }

    #[test]
    fn idempotency_parent_sync_uncertainty_reconciles_before_memory_publish() {
        let dir = test_dir("idempotency-parent-sync-reconcile");
        let path = dir.join("store.json");
        let store = DurableIdempotencyStore::open(&path, 1_000_000_001_000, 30_000)
            .expect("open idempotency store");
        let candidate: IdempotencyState =
            serde_json::from_str(&valid_state_json()).expect("valid idempotency candidate");
        store.fail_next_parent_sync_after_rename_for_test();
        store
            .persist(&candidate)
            .expect("exact read-back reconciles parent sync uncertainty");
        drop(store);

        let reopened = DurableIdempotencyStore::open(&path, 1_000_000_001_000, 30_000)
            .expect("reopen reconciled idempotency store");
        let reopened_state = reopened.state.lock().expect("reopened idempotency state");
        assert_eq!(
            serde_json::to_vec(&*reopened_state).expect("encode reopened state"),
            serde_json::to_vec(&candidate).expect("encode candidate state")
        );
    }

    #[test]
    fn private_idempotency_store_rejects_unsafe_primary_and_temporary_files() {
        let dir = test_dir("secure-idempotency-store");
        let path = dir.join("store.json");
        let store = DurableIdempotencyStore::open(&path, 1000000001000, 30000)
            .expect("open absent idempotency store");
        store
            .persist(&IdempotencyState::default())
            .expect("persist secure idempotency store");
        assert_eq!(
            fs::metadata(&path)
                .expect("persisted idempotency metadata")
                .permissions()
                .mode()
                & 0o777,
            0o600
        );
        DurableIdempotencyStore::open(&path, 1000000001000, 30000)
            .expect("reopen secure idempotency store");

        let insecure = test_dir("insecure-idempotency-mode").join("store.json");
        write_secure(&insecure, valid_state_json()).expect("write valid insecure fixture");
        let mut permissions = fs::metadata(&insecure)
            .expect("insecure idempotency metadata")
            .permissions();
        permissions.set_mode(0o644);
        fs::set_permissions(&insecure, permissions).expect("set insecure idempotency mode");
        assert!(DurableIdempotencyStore::open(&insecure, 1000000001000, 30000).is_err());

        let insecure_parent = test_dir("insecure-idempotency-parent");
        fs::set_permissions(&insecure_parent, fs::Permissions::from_mode(0o755))
            .expect("set insecure idempotency parent mode");
        assert!(
            DurableIdempotencyStore::open(
                &insecure_parent.join("store.json"),
                1000000001000,
                30000,
            )
            .is_err()
        );

        let hardlink_dir = test_dir("hardlinked-idempotency-state");
        let hardlink_path = hardlink_dir.join("store.json");
        let hardlink_alias = hardlink_dir.join("store-alias.json");
        write_secure(&hardlink_path, valid_state_json()).expect("write hardlinked state");
        fs::hard_link(&hardlink_path, &hardlink_alias).expect("create idempotency hardlink");
        assert!(DurableIdempotencyStore::open(&hardlink_path, 1000000001000, 30000).is_err());

        let symlink_dir = test_dir("idempotency-primary-symlink");
        let symlink_path = symlink_dir.join("store.json");
        let sentinel = symlink_dir.join("sentinel.json");
        write_secure(&sentinel, valid_state_json()).expect("write idempotency sentinel");
        symlink(&sentinel, &symlink_path).expect("create idempotency primary symlink");
        assert!(DurableIdempotencyStore::open(&symlink_path, 1000000001000, 30000).is_err());

        let temporary_dir = test_dir("idempotency-temporary-symlink");
        let temporary_path = temporary_dir.join("store.json");
        let temporary_sentinel = temporary_dir.join("sentinel");
        fs::write(&temporary_sentinel, b"do-not-overwrite").expect("write temp sentinel");
        symlink(&temporary_sentinel, temporary_path.with_extension("tmp"))
            .expect("create idempotency temporary symlink");
        let temporary_store = DurableIdempotencyStore::open(&temporary_path, 1000000001000, 30000)
            .expect("open absent idempotency store");
        assert!(
            temporary_store
                .persist(&IdempotencyState::default())
                .is_ok()
        );
        assert_eq!(
            fs::read(&temporary_sentinel).expect("read idempotency temp sentinel"),
            b"do-not-overwrite"
        );
        assert!(
            fs::symlink_metadata(temporary_path.with_extension("tmp"))
                .expect("unrelated legacy idempotency temp symlink remains")
                .file_type()
                .is_symlink()
        );

        let oversized = test_dir("oversized-idempotency-store").join("store.json");
        let oversized_file = File::create(&oversized).expect("create oversized idempotency store");
        oversized_file
            .set_len(MAX_IDEMPOTENCY_STORE_BYTES + 1)
            .expect("size oversized idempotency store");
        let mut permissions = fs::metadata(&oversized)
            .expect("oversized idempotency metadata")
            .permissions();
        permissions.set_mode(0o600);
        fs::set_permissions(&oversized, permissions).expect("secure oversized idempotency mode");
        assert!(DurableIdempotencyStore::open(&oversized, 1000000001000, 30000).is_err());
    }

    #[test]
    fn malformed_json_fails_closed() {
        let dir = test_dir("malformed-json");
        let path = dir.join("store.json");
        write_secure(&path, b"not json").unwrap();
        assert!(DurableIdempotencyStore::open(&path, 1000000001000, 30000).is_err());
    }

    #[test]
    fn wrong_schema_version_fails_closed() {
        let dir = test_dir("wrong-schema");
        let path = dir.join("store.json");
        let mut json = serde_json::from_str::<serde_json::Value>(&valid_state_json()).unwrap();
        json["schema_version"] = serde_json::json!(2);
        write_secure(&path, json.to_string()).unwrap();
        assert!(DurableIdempotencyStore::open(&path, 1000000001000, 30000).is_err());
    }

    #[test]
    fn unknown_field_fails_closed() {
        let dir = test_dir("unknown-field");
        let path = dir.join("store.json");
        let mut json = serde_json::from_str::<serde_json::Value>(&valid_state_json()).unwrap();
        json["bogus_field"] = serde_json::json!(42);
        write_secure(&path, json.to_string()).unwrap();
        assert!(DurableIdempotencyStore::open(&path, 1000000001000, 30000).is_err());
    }

    #[test]
    fn invalid_entry_key_command_grammar_fails_closed() {
        let dir = test_dir("bad-entry-key");
        let path = dir.join("store.json");
        let mut json = serde_json::from_str::<serde_json::Value>(&valid_state_json()).unwrap();
        let old_key = json["entries"]
            .as_object()
            .unwrap()
            .keys()
            .next()
            .unwrap()
            .clone();
        json["entries"]["UPPERCASE INVALID"] = json["entries"][&old_key].clone();
        json["entries"].as_object_mut().unwrap().remove(&old_key);
        json["key_index"].as_object_mut().unwrap().clear();
        write_secure(&path, json.to_string()).unwrap();
        assert!(DurableIdempotencyStore::open(&path, 1000000001000, 30000).is_err());
    }

    #[test]
    fn invalid_key_index_key_grammar_fails_closed() {
        let dir = test_dir("bad-key-grammar");
        let path = dir.join("store.json");
        let mut json = serde_json::from_str::<serde_json::Value>(&valid_state_json()).unwrap();
        let old_key = json["key_index"]
            .as_object()
            .unwrap()
            .keys()
            .next()
            .unwrap()
            .clone();
        let val = json["key_index"][&old_key].clone();
        json["key_index"].as_object_mut().unwrap().remove(&old_key);
        json["key_index"]["UPPERCASE INVALID KEY"] = val;
        write_secure(&path, json.to_string()).unwrap();
        assert!(DurableIdempotencyStore::open(&path, 1000000001000, 30000).is_err());
    }

    #[test]
    fn zero_reservation_version_fails_closed() {
        let dir = test_dir("zero-reserv");
        let path = dir.join("store.json");
        let mut json = serde_json::from_str::<serde_json::Value>(&valid_state_json()).unwrap();
        let key = json["entries"]
            .as_object()
            .unwrap()
            .keys()
            .next()
            .unwrap()
            .clone();
        json["entries"][&key]["reservation_version"] = serde_json::json!(0);
        json["entries"][&key]["in_flight"] = serde_json::json!(true);
        json["entries"][&key]["disposition"] = serde_json::Value::Null;
        write_secure(&path, json.to_string()).unwrap();
        assert!(DurableIdempotencyStore::open(&path, 1000000001000, 30000).is_err());
    }

    #[test]
    fn zero_fencing_token_fails_closed() {
        let dir = test_dir("zero-fencing");
        let path = dir.join("store.json");
        let mut json = serde_json::from_str::<serde_json::Value>(&valid_state_json()).unwrap();
        let key = json["entries"]
            .as_object()
            .unwrap()
            .keys()
            .next()
            .unwrap()
            .clone();
        json["entries"][&key]["fencing_token"] = serde_json::json!(0);
        json["entries"][&key]["in_flight"] = serde_json::json!(true);
        json["entries"][&key]["disposition"] = serde_json::Value::Null;
        write_secure(&path, json.to_string()).unwrap();
        assert!(DurableIdempotencyStore::open(&path, 1000000001000, 30000).is_err());
    }

    #[test]
    fn zero_deadline_ms_fails_closed() {
        let dir = test_dir("zero-deadline");
        let path = dir.join("store.json");
        let mut json = serde_json::from_str::<serde_json::Value>(&valid_state_json()).unwrap();
        let key = json["entries"]
            .as_object()
            .unwrap()
            .keys()
            .next()
            .unwrap()
            .clone();
        json["entries"][&key]["deadline_ms"] = serde_json::json!(0);
        json["entries"][&key]["in_flight"] = serde_json::json!(true);
        json["entries"][&key]["disposition"] = serde_json::Value::Null;
        write_secure(&path, json.to_string()).unwrap();
        assert!(DurableIdempotencyStore::open(&path, 1000000001000, 30000).is_err());
    }

    #[test]
    fn in_flight_with_disposition_fails_closed() {
        let dir = test_dir("inflight-disp");
        let path = dir.join("store.json");
        let mut json = serde_json::from_str::<serde_json::Value>(&valid_state_json()).unwrap();
        let key = json["entries"]
            .as_object()
            .unwrap()
            .keys()
            .next()
            .unwrap()
            .clone();
        json["entries"][&key]["in_flight"] = serde_json::json!(true);
        write_secure(&path, json.to_string()).unwrap();
        assert!(DurableIdempotencyStore::open(&path, 1000000001000, 30000).is_err());
    }

    #[test]
    fn not_in_flight_without_disposition_fails_closed() {
        let dir = test_dir("noflight-nodisp");
        let path = dir.join("store.json");
        let mut json = serde_json::from_str::<serde_json::Value>(&valid_state_json()).unwrap();
        let key = json["entries"]
            .as_object()
            .unwrap()
            .keys()
            .next()
            .unwrap()
            .clone();
        json["entries"][&key]["in_flight"] = serde_json::json!(false);
        json["entries"][&key]["disposition"] = serde_json::Value::Null;
        write_secure(&path, json.to_string()).unwrap();
        assert!(DurableIdempotencyStore::open(&path, 1000000001000, 30000).is_err());
    }

    #[test]
    fn dangling_key_index_reference_fails_closed() {
        let dir = test_dir("dangling-ref");
        let path = dir.join("store.json");
        let mut json = serde_json::from_str::<serde_json::Value>(&valid_state_json()).unwrap();
        let cmd = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
        json["key_index"]["idem:dangling"] = serde_json::json!(cmd);
        write_secure(&path, json.to_string()).unwrap();
        assert!(DurableIdempotencyStore::open(&path, 1000000001000, 30000).is_err());
    }

    #[test]
    fn zero_configured_deadline_rejected_at_open() {
        let dir = test_dir("zero-config-deadline");
        let path = dir.join("store.json");
        assert!(DurableIdempotencyStore::open(&path, 1000000001000, 0).is_err());
    }

    fn in_flight_state_json() -> String {
        let cmd = "abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789";
        let value = serde_json::json!({
            "schema_version": 1,
            "entries": {
                cmd: {
                    "reservation_version": 1,
                    "fencing_token": 1,
                    "deadline_ms": 1000000002000_u64,
                    "in_flight": true,
                    "disposition": null
                }
            },
            "key_index": {
                "idem:finalize-fail": cmd
            }
        });
        let state: IdempotencyState =
            serde_json::from_value(value).expect("valid in-flight fixture");
        serde_json::to_string(&state).expect("canonical in-flight fixture")
    }

    #[test]
    fn finalize_persist_failure_leaves_state_in_flight() {
        use std::os::unix::fs::PermissionsExt;

        let dir = test_dir("persist-fail-finalize");
        let store_path = dir.join("store.json");
        write_secure(&store_path, in_flight_state_json()).unwrap();

        let store = DurableIdempotencyStore::open(&store_path, 1000000001000, 30000).unwrap();

        let command_id =
            CommandId::parse("abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789")
                .unwrap();
        let token = IdempotencyReservationToken::from_store_observation(
            command_id,
            1,
            1,
            SessionInstant::from_unix_millis(1000000002000),
        )
        .unwrap();

        let cmd_id = token.command_id().as_str().to_owned();

        let parent = store_path.parent().unwrap();
        let original_mode = fs::metadata(parent).unwrap().permissions().mode();
        fs::set_permissions(parent, fs::Permissions::from_mode(0o555)).unwrap();

        let projection = PersistedPriorDispositionDto::Rejected(
            ustc_campus_agent_core::request_context::PersistedAdmissionRejectionDto::MalformedCommand {
                operation_id: None,
            },
        );
        let result = store.finalize_with_projection(&token, projection);
        assert!(
            result.is_err(),
            "finalize persist must fail on read-only dir"
        );

        {
            let state = store.state.lock().unwrap();
            let entry = state.entries.get(&cmd_id).unwrap();
            assert!(
                entry.in_flight,
                "entry must remain in_flight after finalize persist failure"
            );
            assert!(
                entry.disposition.is_none(),
                "entry must have no disposition after finalize persist failure"
            );
        }

        fs::set_permissions(parent, fs::Permissions::from_mode(original_mode)).unwrap();

        let projection2 = PersistedPriorDispositionDto::Rejected(
            ustc_campus_agent_core::request_context::PersistedAdmissionRejectionDto::MalformedCommand {
                operation_id: None,
            },
        );
        let result = store.finalize_with_projection(&token, projection2);
        assert!(
            result.is_ok(),
            "clean finalize must succeed after removing fault"
        );
        assert!(
            matches!(result.unwrap(), FinalizeIdempotencyOutcome::Committed),
            "clean finalize must commit"
        );

        {
            let state = store.state.lock().unwrap();
            let entry = state.entries.get(&cmd_id).unwrap();
            assert!(!entry.in_flight, "entry must be committed");
            assert!(entry.disposition.is_some(), "entry must have disposition");
        }
    }
}
