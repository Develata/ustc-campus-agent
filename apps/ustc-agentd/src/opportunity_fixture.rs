//! DemoReviewed Opportunity catalog and M60/admission fixture.

use std::fs;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use serde::Deserialize;
use sha2::{Digest, Sha256};
use ustc_campus_agent_client_protocol::OpportunityCommandDto;
use ustc_campus_agent_core::request_context::{
    AdapterAllowlist, AdapterIdentity, DecoderIdentity, DescriptorSnapshotId, DispatcherIdentity,
    EffectClass, OperationId, OperationSnapshot, PermissionClass, SchemaDigest, SchemaIdentity,
};
use ustc_campus_agent_core::source_registry::{
    SourceId, SourceReviewEvidenceId, SourceReviewerId, SourceUrl,
};
use ustc_campus_agent_core::source_revision::{
    EffectiveInterval, NormalizedSnapshotId, ParserIdentity, RawSnapshotId, RevisionSha256,
    RevisionTimestamp, SourceRevision, SourceRevisionHealth, SourceRevisionId,
};
use ustc_campus_agent_course_planning::CoursePlanningFixture;
use ustc_campus_agent_opportunity_graph::{
    M60OpportunityPort, OpportunitySourcePortError, ReviewedOpportunityCatalog,
};

use crate::affairs_fixture::FixtureDescriptor;
use crate::opportunity_use_case::OpportunityApplicationCounters;

const MAX_OPPORTUNITY_METADATA_BYTES: u64 = 65_536;
const MAX_OPPORTUNITY_CATALOG_BYTES: u64 = 1_048_576;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct OpportunityFixtureDto {
    schema_version: String,
    catalog_source_revision_marker: String,
    catalog_file_sha256: String,
    source_id: String,
    source_url: String,
    raw_snapshot_id: String,
    normalized_snapshot_id: String,
    parser_identity: String,
    observed_at_secs: i64,
    source_published_at_secs: Option<i64>,
    source_reviewer: String,
    source_review_evidence: String,
    source_health: String,
    market_enabled: bool,
    market_grant_active: bool,
    authority_change_before_authorization: String,
    application_failure: String,
    schema_digest_seed: String,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum OpportunityApplicationFailureMode {
    None,
    BeforeDispatch,
    ResponsePersistenceUnavailable,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum OpportunityAuthorityMutationMode {
    None,
    RevokeGrantBeforeAuthorization,
}

pub(crate) struct OpportunityFixture {
    pub(crate) catalog: ReviewedOpportunityCatalog,
    pub(crate) source: CountingOpportunitySourcePort,
    pub(crate) source_evidence_digest: String,
    pub(crate) market_enabled: bool,
    pub(crate) market_grant_active: bool,
    pub(crate) authority_mutation: OpportunityAuthorityMutationMode,
    pub(crate) application_failure: OpportunityApplicationFailureMode,
    pub(crate) application_counters: OpportunityApplicationCounters,
    schema_digest_seed: String,
}

impl OpportunityFixture {
    pub(crate) fn load(metadata_path: &Path, catalog_path: &Path) -> Result<Self, String> {
        let metadata_bytes = read_bounded(
            metadata_path,
            MAX_OPPORTUNITY_METADATA_BYTES,
            "opportunity fixture",
        )?;
        let dto: OpportunityFixtureDto = serde_json::from_slice(&metadata_bytes)
            .map_err(|error| format!("opportunity fixture decode: {error}"))?;
        if dto.schema_version != "opportunity-demo-reviewed/v1" {
            return Err("opportunity fixture schema version mismatch".to_owned());
        }
        let catalog_bytes = read_bounded(
            catalog_path,
            MAX_OPPORTUNITY_CATALOG_BYTES,
            "opportunity catalog",
        )?;
        let file_sha256 = sha256_hex(&catalog_bytes);
        if file_sha256 != dto.catalog_file_sha256 {
            return Err("opportunity catalog retained-byte digest mismatch".to_owned());
        }
        let revision_digest = format!("sha256:{file_sha256}");
        let source_revision = SourceRevision::demo_reviewed(
            SourceId::parse(&dto.source_id)
                .map_err(|error| format!("opportunity source id: {error}"))?,
            SourceUrl::parse(&dto.source_url)
                .map_err(|error| format!("opportunity source URL: {error}"))?,
            RawSnapshotId::parse(&dto.raw_snapshot_id)
                .map_err(|error| format!("opportunity raw snapshot id: {error}"))?,
            RevisionSha256::parse(&revision_digest)
                .map_err(|error| format!("opportunity raw digest: {error}"))?,
            NormalizedSnapshotId::parse(&dto.normalized_snapshot_id)
                .map_err(|error| format!("opportunity normalized snapshot id: {error}"))?,
            RevisionSha256::parse(&revision_digest)
                .map_err(|error| format!("opportunity normalized digest: {error}"))?,
            ParserIdentity::parse(&dto.parser_identity)
                .map_err(|error| format!("opportunity parser identity: {error}"))?,
            RevisionTimestamp::from_unix_seconds(dto.observed_at_secs),
            dto.source_published_at_secs
                .map(RevisionTimestamp::from_unix_seconds),
            EffectiveInterval::new(None, None)
                .map_err(|error| format!("opportunity effective interval: {error}"))?,
            SourceReviewerId::parse(&dto.source_reviewer)
                .map_err(|error| format!("opportunity source reviewer: {error}"))?,
            SourceReviewEvidenceId::parse(&dto.source_review_evidence)
                .map_err(|error| format!("opportunity source review evidence: {error}"))?,
        );
        let mut planning_fixture: CoursePlanningFixture = serde_json::from_slice(&catalog_bytes)
            .map_err(|error| format!("opportunity catalog decode: {error}"))?;
        if planning_fixture.source_revision != dto.catalog_source_revision_marker {
            return Err("opportunity catalog source-revision marker mismatch".to_owned());
        }
        planning_fixture.source_revision = source_revision.revision_id().as_str().to_owned();
        let catalog = ReviewedOpportunityCatalog::from_demo_reviewed(
            source_revision.clone(),
            planning_fixture,
        )
        .map_err(|error| format!("opportunity reviewed catalog: {error}"))?;
        let source_mode = match dto.source_health.as_str() {
            "current" => OpportunitySourceMode::Health(SourceRevisionHealth::Current),
            "stale" => OpportunitySourceMode::Health(SourceRevisionHealth::Stale),
            "conflicting" => OpportunitySourceMode::Health(SourceRevisionHealth::Conflicting),
            "unavailable" => OpportunitySourceMode::Error(OpportunitySourcePortError::Unavailable),
            "corrupted" => OpportunitySourceMode::Error(OpportunitySourcePortError::Corrupted),
            _ => return Err("opportunity source health mode is invalid".to_owned()),
        };
        let application_failure = match dto.application_failure.as_str() {
            "none" => OpportunityApplicationFailureMode::None,
            "before_dispatch" => OpportunityApplicationFailureMode::BeforeDispatch,
            "response_persistence_unavailable" => {
                OpportunityApplicationFailureMode::ResponsePersistenceUnavailable
            }
            _ => return Err("opportunity application failure mode is invalid".to_owned()),
        };
        let authority_mutation = match dto.authority_change_before_authorization.as_str() {
            "none" => OpportunityAuthorityMutationMode::None,
            "revoke_grant" => OpportunityAuthorityMutationMode::RevokeGrantBeforeAuthorization,
            _ => return Err("opportunity authority mutation mode is invalid".to_owned()),
        };
        let source_evidence_digest = format!(
            "sha256:{}",
            sha256_hex(
                format!(
                    "opportunity-source-evidence/v1\0{}\0{}",
                    source_revision.revision_id().as_str(),
                    dto.source_review_evidence
                )
                .as_bytes()
            )
        );
        Ok(Self {
            catalog,
            source: CountingOpportunitySourcePort {
                expected_revision_id: source_revision.revision_id().clone(),
                mode: source_mode,
                calls: Arc::new(AtomicU64::new(0)),
            },
            source_evidence_digest,
            market_enabled: dto.market_enabled,
            market_grant_active: dto.market_grant_active,
            authority_mutation,
            application_failure,
            application_counters: OpportunityApplicationCounters::default(),
            schema_digest_seed: dto.schema_digest_seed,
        })
    }

    pub(crate) fn descriptor(
        &self,
        command: &OpportunityCommandDto,
    ) -> Result<OperationSnapshot, String> {
        let operation_id = command.operation_id();
        let schema_digest = SchemaDigest::parse(sha256_hex(
            format!(
                "{}\0{}\0{}",
                self.schema_digest_seed,
                operation_id,
                match command {
                    OpportunityCommandDto::CreateProfile { .. } => "create",
                    OpportunityCommandDto::ViewProfile { .. } => "view",
                    OpportunityCommandDto::GeneratePlan { .. } => "plan",
                    OpportunityCommandDto::RevokeConsentAndDeleteProfile { .. } => "delete",
                }
            )
            .as_bytes(),
        ))
        .map_err(|error| format!("opportunity schema digest: {error}"))?;
        let snapshot_identity = DescriptorSnapshotId::from_canonical_identity(&schema_digest, 1)
            .map_err(|error| format!("opportunity descriptor identity: {error}"))?;
        let (permission_class, effect_class) = match command {
            OpportunityCommandDto::ViewProfile { .. } => {
                (PermissionClass::TenantPrivateRead, EffectClass::Read)
            }
            OpportunityCommandDto::CreateProfile { .. }
            | OpportunityCommandDto::GeneratePlan { .. }
            | OpportunityCommandDto::RevokeConsentAndDeleteProfile { .. } => (
                PermissionClass::TenantPrivateWrite,
                EffectClass::TenantLocalMutation,
            ),
        };
        Ok(Arc::new(FixtureDescriptor {
            operation_id: OperationId::parse(operation_id)
                .map_err(|error| format!("opportunity operation id: {error}"))?,
            schema_identity: SchemaIdentity::parse("schema:opportunity-private:v1")
                .map_err(|error| format!("opportunity schema identity: {error}"))?,
            schema_digest,
            permission_class,
            effect_class,
            decoder_identity: DecoderIdentity::parse("decoder:opportunity-private:v1")
                .map_err(|error| format!("opportunity decoder identity: {error}"))?,
            dispatcher_identity: DispatcherIdentity::parse("dispatcher:opportunity-private:v1")
                .map_err(|error| format!("opportunity dispatcher identity: {error}"))?,
            adapter_allowlist: AdapterAllowlist::try_from_iter([AdapterIdentity::parse(
                "adapter:ustc-agentd-opportunity-web:v1",
            )
            .map_err(|error| format!("opportunity adapter identity: {error}"))?])
            .map_err(|error| format!("opportunity adapter allowlist: {error:?}"))?,
            snapshot_identity,
        }))
    }
}

#[derive(Clone, Copy)]
enum OpportunitySourceMode {
    Health(SourceRevisionHealth),
    Error(OpportunitySourcePortError),
}

pub(crate) struct CountingOpportunitySourcePort {
    expected_revision_id: SourceRevisionId,
    mode: OpportunitySourceMode,
    calls: Arc<AtomicU64>,
}

impl CountingOpportunitySourcePort {
    pub(crate) fn calls(&self) -> u64 {
        self.calls.load(Ordering::SeqCst)
    }
}

impl M60OpportunityPort for CountingOpportunitySourcePort {
    fn revision_health(
        &self,
        revision: &SourceRevision,
    ) -> Result<SourceRevisionHealth, OpportunitySourcePortError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        if revision.revision_id() != &self.expected_revision_id {
            return Err(OpportunitySourcePortError::Corrupted);
        }
        match self.mode {
            OpportunitySourceMode::Health(health) => Ok(health),
            OpportunitySourceMode::Error(error) => Err(error),
        }
    }
}

fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn read_bounded(path: &Path, max_bytes: u64, label: &str) -> Result<Vec<u8>, String> {
    let length = fs::metadata(path)
        .map_err(|error| format!("{label} metadata: {error}"))?
        .len();
    if length > max_bytes {
        return Err(format!("{label} exceeds byte limit"));
    }
    fs::read(path).map_err(|error| format!("{label} read: {error}"))
}
