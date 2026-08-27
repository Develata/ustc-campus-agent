//! Source-grounded DemoReviewed ChangeRadar fixture for the bounded composition.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Component, Path, PathBuf};

use serde::Deserialize;
use sha2::{Digest, Sha256};
use ustc_campus_agent_change_radar::{
    AcceptedObservation, BoardFeedPolicy, BoardId, BoardPolicy, ChangeFeedQueryService,
    ChangePublicationService, ChangeRadarService, ChangeReviewReceipt,
    InMemoryChangeRadarRepository, M60ChangePublicationOutcome, M60ChangePublicationPort,
    M60ChangePublicationPortError, M60VerifiedChangeEvidence, NormalizedFacts, ObservationOutcome,
    SemanticField, SemanticValue,
};
use ustc_campus_agent_core::identity::UserId;
use ustc_campus_agent_core::request_context::{
    AdapterAllowlist, AdapterIdentity, DecoderIdentity, DescriptorSnapshotId, DispatcherIdentity,
    EffectClass, OperationId, OperationSnapshot, PermissionClass, SchemaDigest, SchemaIdentity,
};
use ustc_campus_agent_core::source_registry::{
    SourceId, SourceReviewEvidenceId, SourceReviewerId, SourceUrl,
};
use ustc_campus_agent_core::source_revision::{
    EffectiveInterval, NormalizedSnapshotId, ParserIdentity, RawSnapshotId, RevisionSha256,
    RevisionTimestamp, SourceRevision, SourceRevisionHealth,
};

use crate::affairs_fixture::FixtureDescriptor;
use crate::change_invocation::ChangeInvocationCounters;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ChangeFixtureDto {
    board_id: String,
    board_policy_revision: u64,
    feed_title: String,
    feed_author: String,
    feed_public_base_url: String,
    source_id: String,
    source_url: String,
    parser_identity: String,
    source_reviewer: String,
    old_revision: ChangeRevisionDto,
    new_revision: ChangeRevisionDto,
    tracked_fields: Vec<String>,
    affected_scope: String,
    effective_from_secs: i64,
    effective_to_secs: i64,
    publication_reviewer: String,
    reviewed_at_secs: i64,
    published_at_secs: i64,
    market_enabled: bool,
    market_grant_active: bool,
    schema_digest: String,
    descriptor_snapshot_version: u64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ChangeRevisionDto {
    raw_path: String,
    raw_snapshot_id: String,
    raw_file_sha256: String,
    raw_digest: String,
    normalized_path: String,
    normalized_snapshot_id: String,
    normalized_file_sha256: String,
    normalized_digest: String,
    source_published_at_secs: i64,
    observed_at_secs: i64,
    source_review_evidence: String,
    facts: BTreeMap<String, String>,
}

pub(crate) struct ChangeRadarFixture {
    pub(crate) repository: InMemoryChangeRadarRepository,
    pub(crate) feed_policy: BoardFeedPolicy,
    pub(crate) source_evidence_digest: String,
    pub(crate) market_enabled: bool,
    pub(crate) market_grant_active: bool,
    pub(crate) invocation_counters: ChangeInvocationCounters,
    pub(crate) descriptor: OperationSnapshot,
}

impl ChangeRadarFixture {
    pub(crate) fn load(path: &Path) -> Result<Self, String> {
        let bytes =
            fs::read(path).map_err(|error| format!("change fixture read failed: {error}"))?;
        let dto: ChangeFixtureDto = serde_json::from_slice(&bytes)
            .map_err(|error| format!("change fixture parse failed: {error}"))?;
        let root = path
            .parent()
            .ok_or_else(|| "change fixture has no parent".to_owned())?;
        Self::build(root, dto)
    }

    fn build(root: &Path, dto: ChangeFixtureDto) -> Result<Self, String> {
        let board_id = BoardId::parse(&dto.board_id)
            .map_err(|error| format!("change board_id invalid: {error}"))?;
        let source_id = SourceId::parse(&dto.source_id)
            .map_err(|error| format!("change source_id invalid: {error}"))?;
        let parser_identity = ParserIdentity::parse(&dto.parser_identity)
            .map_err(|error| format!("change parser_identity invalid: {error}"))?;
        let source_url = SourceUrl::parse(&dto.source_url)
            .map_err(|error| format!("change source_url invalid: {error}"))?;
        let source_reviewer = SourceReviewerId::parse(&dto.source_reviewer)
            .map_err(|error| format!("change source_reviewer invalid: {error}"))?;
        let effective_interval = EffectiveInterval::new(
            Some(RevisionTimestamp::from_unix_seconds(
                dto.effective_from_secs,
            )),
            Some(RevisionTimestamp::from_unix_seconds(dto.effective_to_secs)),
        )
        .map_err(|error| format!("change effective interval invalid: {error}"))?;
        let old = build_observation(
            root,
            &dto.old_revision,
            &source_id,
            &source_url,
            &parser_identity,
            &source_reviewer,
            &effective_interval,
        )?;
        let new = build_observation(
            root,
            &dto.new_revision,
            &source_id,
            &source_url,
            &parser_identity,
            &source_reviewer,
            &effective_interval,
        )?;
        let tracked_fields = dto
            .tracked_fields
            .iter()
            .map(|value| {
                SemanticField::parse(value)
                    .map_err(|error| format!("tracked field invalid: {error}"))
            })
            .collect::<Result<Vec<_>, _>>()?;
        let board_policy = BoardPolicy::new(
            board_id.clone(),
            source_id,
            dto.board_policy_revision,
            tracked_fields,
            &dto.affected_scope,
        )
        .map_err(|error| format!("change board policy invalid: {error}"))?;
        let mut radar = ChangeRadarService::new(board_policy, InMemoryChangeRadarRepository::new());
        match radar
            .observe(old)
            .map_err(|error| format!("change baseline failed: {error}"))?
        {
            ObservationOutcome::BaselineEstablished { .. } => {}
            other => return Err(format!("change baseline unexpected: {other:?}")),
        }
        let candidate = match radar
            .observe(new)
            .map_err(|error| format!("change candidate failed: {error}"))?
        {
            ObservationOutcome::SemanticChange(candidate) => *candidate,
            other => return Err(format!("change candidate unexpected: {other:?}")),
        };
        let mut repository = radar.into_repository();
        let feed_policy = BoardFeedPolicy::new(
            board_id,
            dto.board_policy_revision,
            dto.feed_title,
            dto.feed_author,
            dto.feed_public_base_url,
        )
        .map_err(|error| format!("change feed policy invalid: {error}"))?;
        let review = ChangeReviewReceipt::approve(
            &candidate,
            UserId::parse(dto.publication_reviewer)
                .map_err(|error| format!("change publication reviewer invalid: {error}"))?,
            RevisionTimestamp::from_unix_seconds(dto.reviewed_at_secs),
        )
        .map_err(|error| format!("change review invalid: {error}"))?;
        let m60 = FixtureM60;
        let published = {
            let mut publication =
                ChangePublicationService::new(&mut repository, &m60, feed_policy.clone());
            publication
                .record_review(review)
                .map_err(|error| format!("change review persistence failed: {error}"))?;
            publication
                .publish(
                    candidate.event_id(),
                    RevisionTimestamp::from_unix_seconds(dto.published_at_secs),
                )
                .map_err(|error| format!("change publication failed: {error}"))?
        };
        let source_evidence_digest = published.evidence_set_digest().as_str().to_owned();
        let query = ChangeFeedQueryService::new(&repository)
            .execute(&feed_policy)
            .map_err(|error| format!("change feed query failed: {error}"))?;
        if query.items().len() != 1 || query.atom().is_empty() {
            return Err("change feed fixture did not publish exactly one event".to_owned());
        }
        let schema_digest = SchemaDigest::parse(&dto.schema_digest)
            .map_err(|error| format!("change schema digest invalid: {error}"))?;
        let snapshot_identity = DescriptorSnapshotId::from_canonical_identity(
            &schema_digest,
            dto.descriptor_snapshot_version,
        )
        .map_err(|error| format!("change descriptor identity invalid: {error}"))?;
        let descriptor: OperationSnapshot = std::sync::Arc::new(FixtureDescriptor {
            operation_id: OperationId::parse("change.list")
                .map_err(|error| format!("change operation invalid: {error}"))?,
            schema_identity: SchemaIdentity::parse("schema:change-feed-fixture")
                .map_err(|error| format!("change schema identity invalid: {error}"))?,
            schema_digest,
            permission_class: PermissionClass::PublicRead,
            effect_class: EffectClass::Read,
            decoder_identity: DecoderIdentity::parse("decoder:change-feed:v1")
                .map_err(|error| format!("change decoder invalid: {error}"))?,
            dispatcher_identity: DispatcherIdentity::parse("dispatcher:change-feed:v1")
                .map_err(|error| format!("change dispatcher invalid: {error}"))?,
            adapter_allowlist: AdapterAllowlist::try_from_iter(vec![
                AdapterIdentity::parse("fixture.adapter")
                    .map_err(|error| format!("change adapter invalid: {error}"))?,
            ])
            .map_err(|error| format!("change adapter allowlist invalid: {error:?}"))?,
            snapshot_identity,
        });
        Ok(Self {
            repository,
            feed_policy,
            source_evidence_digest,
            market_enabled: dto.market_enabled,
            market_grant_active: dto.market_grant_active,
            invocation_counters: ChangeInvocationCounters::default(),
            descriptor,
        })
    }
}

fn build_observation(
    root: &Path,
    dto: &ChangeRevisionDto,
    source_id: &SourceId,
    source_url: &SourceUrl,
    parser_identity: &ParserIdentity,
    source_reviewer: &SourceReviewerId,
    effective_interval: &EffectiveInterval,
) -> Result<AcceptedObservation, String> {
    let raw = read_evidence(root, &dto.raw_path)?;
    let raw_file_sha256 = sha256_hex(&raw);
    if raw_file_sha256 != dto.raw_file_sha256 {
        return Err(format!("raw evidence digest mismatch: {}", dto.raw_path));
    }
    let raw_digest = RevisionSha256::parse(&dto.raw_digest)
        .map_err(|error| format!("raw digest invalid: {error}"))?;
    let retained_raw_digest = RevisionSha256::parse(format!("sha256:{raw_file_sha256}"))
        .map_err(|error| format!("retained raw digest invalid: {error}"))?;
    if raw_digest != retained_raw_digest {
        return Err(format!(
            "raw revision digest does not match retained evidence: {}",
            dto.raw_path
        ));
    }
    let normalized = read_evidence(root, &dto.normalized_path)?;
    if sha256_hex(&normalized) != dto.normalized_file_sha256 {
        return Err(format!(
            "normalized evidence file digest mismatch: {}",
            dto.normalized_path
        ));
    }
    let retained_facts: BTreeMap<String, String> = serde_json::from_slice(&normalized)
        .map_err(|error| format!("normalized evidence parse failed: {error}"))?;
    if retained_facts != dto.facts {
        return Err(format!(
            "normalized evidence facts mismatch: {}",
            dto.normalized_path
        ));
    }
    let facts = NormalizedFacts::try_from_iter(
        dto.facts
            .iter()
            .map(|(field, value)| {
                Ok((
                    SemanticField::parse(field)
                        .map_err(|error| format!("semantic field invalid: {error}"))?,
                    SemanticValue::parse(value)
                        .map_err(|error| format!("semantic value invalid: {error}"))?,
                ))
            })
            .collect::<Result<Vec<_>, String>>()?,
    )
    .map_err(|error| format!("normalized facts invalid: {error}"))?;
    let normalized_digest = RevisionSha256::parse(&dto.normalized_digest)
        .map_err(|error| format!("normalized digest invalid: {error}"))?;
    if facts.sha256() != normalized_digest {
        return Err(format!(
            "normalized facts digest mismatch: {}",
            dto.normalized_path
        ));
    }
    let revision = SourceRevision::demo_reviewed(
        source_id.clone(),
        source_url.clone(),
        RawSnapshotId::parse(&dto.raw_snapshot_id)
            .map_err(|error| format!("raw snapshot id invalid: {error}"))?,
        raw_digest,
        NormalizedSnapshotId::parse(&dto.normalized_snapshot_id)
            .map_err(|error| format!("normalized snapshot id invalid: {error}"))?,
        normalized_digest,
        parser_identity.clone(),
        RevisionTimestamp::from_unix_seconds(dto.observed_at_secs),
        Some(RevisionTimestamp::from_unix_seconds(
            dto.source_published_at_secs,
        )),
        *effective_interval,
        source_reviewer.clone(),
        SourceReviewEvidenceId::parse(&dto.source_review_evidence)
            .map_err(|error| format!("source review evidence invalid: {error}"))?,
    );
    AcceptedObservation::new(revision, facts, SourceRevisionHealth::Current)
        .map_err(|error| format!("accepted observation invalid: {error}"))
}

fn read_evidence(root: &Path, relative: &str) -> Result<Vec<u8>, String> {
    let relative_path = PathBuf::from(relative);
    if relative_path.is_absolute()
        || relative_path
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(format!("unsafe evidence path: {relative}"));
    }
    fs::read(root.join(relative_path)).map_err(|error| format!("evidence read failed: {error}"))
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut output = String::with_capacity(64);
    for byte in digest {
        use std::fmt::Write as _;
        let _ = write!(output, "{byte:02x}");
    }
    output
}

struct FixtureM60;

impl M60ChangePublicationPort for FixtureM60 {
    fn verify_publication(
        &self,
        old_revision: &SourceRevision,
        new_revision: &SourceRevision,
    ) -> Result<M60ChangePublicationOutcome, M60ChangePublicationPortError> {
        Ok(M60ChangePublicationOutcome::CurrentVerified(
            M60VerifiedChangeEvidence::for_revisions(old_revision, new_revision),
        ))
    }
}
