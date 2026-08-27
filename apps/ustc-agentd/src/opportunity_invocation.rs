//! Bounded Opportunity Graph Market/Harness/ToolGateway/owning-executor spine.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};

use serde::Deserialize;
use time::OffsetDateTime;
use ustc_agent_tool_protocol::{
    AgentToolCall, AgentToolOutcome, AgentToolResult, CanonicalArgumentNodeV0,
    CanonicalArgumentValueV0, ProviderToolCallId as ProtocolProviderToolCallId, Sha256Digest,
    StableToolCode, UnvalidatedArgumentValueV0,
};
use ustc_campus_agent_application_ingress::{
    OpportunityInvocationError, OpportunityInvocationOutcome, OpportunityInvocationPort,
};
use ustc_campus_agent_client_protocol::{
    OpportunityCommandDto, OpportunityConsentFieldDto, opportunity_payload_digest,
};
use ustc_campus_agent_core::identity::{TenantId, UserId};
use ustc_campus_agent_core::invocation::{
    CapabilityClass, CapabilityGrantSnapshot, CapabilityId, CatalogComponentRevision,
    CatalogPackageRevision, CatalogRevision, CatalogToolDefinition, ComponentId, ComponentKind,
    ComponentVersion, ConfirmationPolicy, ExecutionIdentity, GrantSnapshotId, GrantState,
    GrantVersion, InstallationId, InstallationRevision, InstallationState,
    InstalledComponentIdentity, InvocationPolicySnapshot, InvocationTarget, ObjectScope, PackageId,
    PackageVersion, PluginInstallationSnapshot, PolicyRevision, PolicySnapshotId, ProposedToolCall,
    ProviderToolCallId as CoreProviderToolCallId, RunId, Sha256Digest as CoreSha256Digest,
    SourcePolicyId, SourcePolicyIdentity, ToolId, ToolProjectionRequest, TurnId,
    UnvalidatedSchemaNodeV0, UnvalidatedToolInputSchemaV0, ValidatedToolInputSchemaV0,
};
use ustc_campus_agent_core::market::authority::{
    AuthorityRepositoryError, InMemoryInvocationAuthorityRepository, InvocationAuthorityRepository,
    InvocationAuthorityService, InvocationRecheckError, ProjectionAssemblyError,
};
use ustc_campus_agent_core::market::load_package_manifest;
use ustc_campus_agent_core::request_context::M00AdmittedActor;
use ustc_campus_agent_course_planning::PlanningConfig;
use ustc_campus_agent_opportunity_graph::{
    AcademicProfileInput, AuthenticatedPrincipal, ConsentField, ConsentGrant, ConsentPurpose,
    OpportunityPlanningService, OpportunityProfileService, ProfileSnapshotId,
    ReviewedOpportunityCatalog,
};
use ustc_campus_agent_runtime::{
    AgentRun, Decision, EffectIntent, EffectOutcome, EffectReceipt, RUN_SPEC_SCHEMA_VERSION,
    RunBudgets, RunCommand, RunSpec, ToolCallProposal,
};

use crate::opportunity_fixture::{
    CountingOpportunitySourcePort, OpportunityAuthorityMutationMode, OpportunityToolFailureMode,
};
use crate::opportunity_persistence::DurableOpportunityProfileRepository;

const OPPORTUNITY_PACKAGE_MANIFEST: &[u8] =
    include_bytes!("../../../market/packages/ustc.opportunity-graph/package.json");
const OPPORTUNITY_NATIVE_COMPONENT_DESCRIPTOR: &[u8] = include_bytes!(
    "../../../market/packages/ustc.opportunity-graph/components/native-rust-component.json"
);

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct NativeRustComponentDescriptor {
    schema_version: String,
    package_id: String,
    package_version: String,
    crate_path: String,
    composition_owner: String,
    operations: Vec<String>,
}

fn validate_native_component_descriptor(source: &[u8]) -> Result<(), OpportunityInvocationError> {
    let descriptor = serde_json::from_slice::<NativeRustComponentDescriptor>(source)
        .map_err(|_| OpportunityInvocationError::Internal)?;
    if descriptor.schema_version != "native-rust-component/v1"
        || descriptor.package_id != "ustc.opportunity-graph"
        || descriptor.package_version != "0.1.0"
        || descriptor.crate_path != "crates/opportunity-graph"
        || descriptor.composition_owner != "apps/ustc-agentd"
        || descriptor.operations
            != [
                "profile.academic.create",
                "profile.academic.view",
                "planner.generate",
                "profile.academic.revoke_delete",
            ]
    {
        return Err(OpportunityInvocationError::Internal);
    }
    Ok(())
}

struct BorrowedAuthorityRepository<'a>(&'a InMemoryInvocationAuthorityRepository);

impl InvocationAuthorityRepository for BorrowedAuthorityRepository<'_> {
    type ReadTransaction<'a>
        = <InMemoryInvocationAuthorityRepository as InvocationAuthorityRepository>::ReadTransaction<
        'a,
    >
    where
        Self: 'a;

    fn begin_read(&self) -> Result<Self::ReadTransaction<'_>, AuthorityRepositoryError> {
        self.0.begin_read()
    }
}

#[derive(Clone, Default)]
pub(crate) struct OpportunityInvocationCounters {
    intents: Arc<AtomicU64>,
    executions: Arc<AtomicU64>,
    receipts: Arc<AtomicU64>,
}

impl OpportunityInvocationCounters {
    pub(crate) fn intents(&self) -> u64 {
        self.intents.load(Ordering::SeqCst)
    }

    pub(crate) fn executions(&self) -> u64 {
        self.executions.load(Ordering::SeqCst)
    }

    pub(crate) fn receipts(&self) -> u64 {
        self.receipts.load(Ordering::SeqCst)
    }
}

pub(crate) struct OpportunityInvocationSpine<'a> {
    repository: &'a Mutex<DurableOpportunityProfileRepository>,
    source: &'a CountingOpportunitySourcePort,
    catalog: &'a ReviewedOpportunityCatalog,
    authority: &'a OpportunityAuthorityStore,
    tool_failure: OpportunityToolFailureMode,
    counters: OpportunityInvocationCounters,
}

impl<'a> OpportunityInvocationSpine<'a> {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        repository: &'a Mutex<DurableOpportunityProfileRepository>,
        source: &'a CountingOpportunitySourcePort,
        catalog: &'a ReviewedOpportunityCatalog,
        authority: &'a OpportunityAuthorityStore,
        tool_failure: OpportunityToolFailureMode,
        counters: OpportunityInvocationCounters,
    ) -> Self {
        Self {
            repository,
            source,
            catalog,
            authority,
            tool_failure,
            counters,
        }
    }
}

impl OpportunityInvocationPort for OpportunityInvocationSpine<'_> {
    fn invoke(
        &self,
        actor: &M00AdmittedActor,
        command: &OpportunityCommandDto,
    ) -> Result<OpportunityInvocationOutcome, OpportunityInvocationError> {
        let command_digest = opportunity_payload_digest(command)
            .map_err(|_| OpportunityInvocationError::Internal)?;
        let metadata = tool_metadata(command);
        let request = projection_request(actor, &metadata)?;
        let authority_entry = self.authority.entry(&metadata)?;
        let projection = {
            let entry = authority_entry
                .lock()
                .map_err(|_| OpportunityInvocationError::Unavailable)?;
            InvocationAuthorityService::new(BorrowedAuthorityRepository(&entry.repository))
                .resolve_projection(request, vec![entry.target.clone()])
                .map_err(map_projection_error)?
        };
        let toolset = projection
            .agent_toolset_view()
            .map_err(|_| OpportunityInvocationError::Internal)?;
        let arguments =
            CanonicalArgumentValueV0::try_from(UnvalidatedArgumentValueV0::Object(vec![
                (
                    "operation_id".to_owned(),
                    UnvalidatedArgumentValueV0::String(command.operation_id().to_owned()),
                ),
                (
                    "payload_digest".to_owned(),
                    UnvalidatedArgumentValueV0::String(command_digest.as_str().to_owned()),
                ),
            ]))
            .map_err(|_| OpportunityInvocationError::Internal)?;
        let call = toolset
            .bind_call(
                ProtocolProviderToolCallId::parse(format!(
                    "provider-call:opportunity:{}",
                    metadata.suffix
                ))
                .map_err(|_| OpportunityInvocationError::Internal)?,
                metadata.model_visible_name,
                arguments,
            )
            .map_err(|_| OpportunityInvocationError::Internal)?;
        let entry = projection
            .entries()
            .first()
            .ok_or(OpportunityInvocationError::Internal)?;
        let mut run = AgentRun::new(run_spec(&projection, entry))
            .map_err(|_| OpportunityInvocationError::Internal)?;
        append(&mut run, RunCommand::Prepare)?;
        append(&mut run, RunCommand::StartHarnessTurn)?;
        let proposed = ProposedToolCall {
            provider_tool_call_id: CoreProviderToolCallId::parse(
                call.provider_tool_call_id().as_str().to_owned(),
            )
            .map_err(|_| OpportunityInvocationError::Internal)?,
            model_visible_name: call.model_visible_name().to_owned(),
            dispatch_key: call.route_ref().as_str().to_owned(),
            arguments: call.arguments().clone(),
            claimed_argument_digest: call.arguments().digest().clone(),
        };
        let authorized = {
            let mut entry = authority_entry
                .lock()
                .map_err(|_| OpportunityInvocationError::Unavailable)?;
            self.authority.mutate_after_projection(&mut entry)?;
            InvocationAuthorityService::new(BorrowedAuthorityRepository(&entry.repository))
                .recheck_invocation(&projection, proposed)
                .map_err(map_recheck_error)?
        };
        append(
            &mut run,
            RunCommand::ProposeToolCall(ToolCallProposal::from(&call)),
        )?;
        verify_authorized_arguments(
            authorized.arguments().root(),
            command.operation_id(),
            command_digest.as_str(),
        )?;
        if self.tool_failure == OpportunityToolFailureMode::BeforeExecution {
            return Err(OpportunityInvocationError::Unavailable);
        }
        let effect_suffix = &command_digest.as_str()[..12];
        let intent = EffectIntent {
            effect_id: format!("effect:opportunity:{}:{effect_suffix}", metadata.suffix),
            idempotency_key: format!(
                "effect-idem:opportunity:{}:{effect_suffix}",
                metadata.suffix
            ),
            call_id: call.provider_tool_call_id().as_str().to_owned(),
            tool_name: call.model_visible_name().to_owned(),
            arguments_digest: call.arguments().digest().as_str().to_owned(),
            capability_id: authorized.entry().capability_id().as_str().to_owned(),
            grant_snapshot_id: authorized.entry().grant_snapshot_id().as_str().to_owned(),
            tool_schema_set_digest: projection.tool_schema_set_digest().as_str().to_owned(),
        };
        append(&mut run, RunCommand::ApproveToolCall(intent.clone()))?;
        self.counters.intents.fetch_add(1, Ordering::SeqCst);

        let outcome = self.execute(actor, command);
        self.counters.executions.fetch_add(1, Ordering::SeqCst);
        let outcome = match outcome {
            Ok(value) => value,
            Err(error) => {
                record_failed_attempt(
                    &mut run,
                    &self.counters,
                    &call,
                    &intent,
                    stable_error_code(&error),
                )?;
                return Err(error);
            }
        };
        if self.tool_failure == OpportunityToolFailureMode::OutcomePersistenceUnavailable {
            return Err(OpportunityInvocationError::OutcomeUnknown);
        }
        let output_identity = match &outcome {
            OpportunityInvocationOutcome::ProfileCreated(record)
            | OpportunityInvocationOutcome::ProfileFound(record) => {
                record.profile_snapshot_id().as_str()
            }
            OpportunityInvocationOutcome::PlanGenerated(receipt) => receipt.receipt_id(),
            OpportunityInvocationOutcome::ProfileDeleted(receipt) => receipt.receipt_id().as_str(),
        };
        let output_digest = Sha256Digest::from_bytes(
            format!(
                "opportunity-result/v1\0{}\0{output_identity}",
                command.operation_id()
            )
            .as_bytes(),
        );
        let result = AgentToolResult::from_call(
            &call,
            AgentToolOutcome::Succeeded {
                output_digest: output_digest.clone(),
            },
        );
        let AgentToolOutcome::Succeeded { output_digest } = result.outcome() else {
            return Err(OpportunityInvocationError::Internal);
        };
        append(
            &mut run,
            RunCommand::RecordEffectReceipt(EffectReceipt {
                effect_id: intent.effect_id,
                idempotency_key: intent.idempotency_key,
                outcome: EffectOutcome::Succeeded {
                    output_digest: output_digest.as_str().to_owned(),
                },
            }),
        )?;
        self.counters.receipts.fetch_add(1, Ordering::SeqCst);
        append(&mut run, RunCommand::StartHarnessTurn)?;
        append(
            &mut run,
            RunCommand::Complete {
                output_digest: output_digest.as_str().to_owned(),
            },
        )?;
        Ok(outcome)
    }
}

impl OpportunityInvocationSpine<'_> {
    fn execute(
        &self,
        actor: &M00AdmittedActor,
        command: &OpportunityCommandDto,
    ) -> Result<OpportunityInvocationOutcome, OpportunityInvocationError> {
        let principal = authenticated_principal(actor)?;
        let mut repository = self
            .repository
            .lock()
            .map_err(|_| OpportunityInvocationError::Unavailable)?;
        match command {
            OpportunityCommandDto::CreateProfile {
                consent_purpose,
                consent_fields,
                consented_at,
                completed_courses,
                min_credits,
                max_credits,
                preference_weights,
            } => {
                if consent_purpose.as_str() != "opportunity_planning" {
                    return Err(OpportunityInvocationError::Internal);
                }
                let fields = consent_fields
                    .iter()
                    .map(|field| match field {
                        OpportunityConsentFieldDto::CompletedCourses => {
                            ConsentField::CompletedCourses
                        }
                        OpportunityConsentFieldDto::CreditBounds => ConsentField::CreditBounds,
                        OpportunityConsentFieldDto::PreferenceWeights => {
                            ConsentField::PreferenceWeights
                        }
                    })
                    .collect::<Vec<_>>();
                let consent = ConsentGrant::new(
                    ConsentPurpose::OpportunityPlanning,
                    fields,
                    unix_millis(*consented_at)?,
                )
                .map_err(|error| {
                    OpportunityInvocationError::Profile(
                        ustc_campus_agent_opportunity_graph::OpportunityProfileError::Value(error),
                    )
                })?;
                let preferences = preference_weights
                    .iter()
                    .map(|preference| {
                        (
                            preference.course_code.as_str().to_owned(),
                            preference.weight,
                        )
                    })
                    .collect::<BTreeMap<_, _>>();
                let profile = AcademicProfileInput::new(
                    completed_courses
                        .iter()
                        .map(|course| course.as_str().to_owned())
                        .collect(),
                    *min_credits,
                    *max_credits,
                    preferences,
                )
                .map_err(|error| {
                    OpportunityInvocationError::Profile(
                        ustc_campus_agent_opportunity_graph::OpportunityProfileError::Value(error),
                    )
                })?;
                OpportunityProfileService::new(&mut *repository)
                    .create_profile(principal, consent, profile)
                    .map(OpportunityInvocationOutcome::ProfileCreated)
                    .map_err(OpportunityInvocationError::Profile)
            }
            OpportunityCommandDto::ViewProfile {
                profile_snapshot_id,
            } => {
                let profile_snapshot_id = ProfileSnapshotId::parse(profile_snapshot_id.as_str())
                    .map_err(|_| OpportunityInvocationError::Internal)?;
                OpportunityProfileService::new(&mut *repository)
                    .view_profile(&principal, &profile_snapshot_id)
                    .map(OpportunityInvocationOutcome::ProfileFound)
                    .map_err(OpportunityInvocationError::Profile)
            }
            OpportunityCommandDto::GeneratePlan {
                profile_snapshot_id,
                max_results,
                beam_width,
            } => {
                let profile_snapshot_id = ProfileSnapshotId::parse(profile_snapshot_id.as_str())
                    .map_err(|_| OpportunityInvocationError::Internal)?;
                OpportunityPlanningService::new(&*repository, self.source, self.catalog)
                    .plan(
                        &principal,
                        &profile_snapshot_id,
                        PlanningConfig {
                            max_results: usize::from(*max_results),
                            beam_width: usize::from(*beam_width),
                        },
                    )
                    .map(OpportunityInvocationOutcome::PlanGenerated)
                    .map_err(OpportunityInvocationError::Planning)
            }
            OpportunityCommandDto::RevokeConsentAndDeleteProfile {
                profile_snapshot_id,
                revoked_at,
            } => {
                let profile_snapshot_id = ProfileSnapshotId::parse(profile_snapshot_id.as_str())
                    .map_err(|_| OpportunityInvocationError::Internal)?;
                OpportunityProfileService::new(&mut *repository)
                    .revoke_consent_and_delete_profile(
                        &principal,
                        &profile_snapshot_id,
                        unix_millis(*revoked_at)?,
                    )
                    .map(OpportunityInvocationOutcome::ProfileDeleted)
                    .map_err(OpportunityInvocationError::Profile)
            }
        }
    }
}

#[derive(Clone, Copy)]
struct ToolMetadata {
    operation_id: &'static str,
    suffix: &'static str,
    tool_id: &'static str,
    model_visible_name: &'static str,
    capability_id: &'static str,
    capability_class: CapabilityClass,
}

const PROFILE_CREATE_METADATA: ToolMetadata = ToolMetadata {
    operation_id: "profile.academic.create",
    suffix: "profile-create",
    tool_id: "tool:opportunity-profile-create",
    model_visible_name: "ustc_opportunity_profile_create",
    capability_id: "user.own_academic_snapshot.write",
    capability_class: CapabilityClass::TenantPrivateWrite,
};
const PROFILE_VIEW_METADATA: ToolMetadata = ToolMetadata {
    operation_id: "profile.academic.view",
    suffix: "profile-view",
    tool_id: "tool:opportunity-profile-view",
    model_visible_name: "ustc_opportunity_profile_view",
    capability_id: "user.own_academic_snapshot.read",
    capability_class: CapabilityClass::TenantPrivateRead,
};
const PLAN_GENERATE_METADATA: ToolMetadata = ToolMetadata {
    operation_id: "planner.generate",
    suffix: "plan-generate",
    tool_id: "tool:opportunity-plan-generate",
    model_visible_name: "ustc_opportunity_plan_generate",
    capability_id: "user.own_plan_draft.write",
    capability_class: CapabilityClass::TenantPrivateWrite,
};
const PROFILE_DELETE_METADATA: ToolMetadata = ToolMetadata {
    operation_id: "profile.academic.revoke_delete",
    suffix: "profile-delete",
    tool_id: "tool:opportunity-profile-delete",
    model_visible_name: "ustc_opportunity_profile_delete",
    capability_id: "user.own_academic_snapshot.write",
    capability_class: CapabilityClass::TenantPrivateWrite,
};
const ALL_TOOL_METADATA: [ToolMetadata; 4] = [
    PROFILE_CREATE_METADATA,
    PROFILE_VIEW_METADATA,
    PLAN_GENERATE_METADATA,
    PROFILE_DELETE_METADATA,
];

fn tool_metadata(command: &OpportunityCommandDto) -> ToolMetadata {
    match command {
        OpportunityCommandDto::CreateProfile { .. } => PROFILE_CREATE_METADATA,
        OpportunityCommandDto::ViewProfile { .. } => PROFILE_VIEW_METADATA,
        OpportunityCommandDto::GeneratePlan { .. } => PLAN_GENERATE_METADATA,
        OpportunityCommandDto::RevokeConsentAndDeleteProfile { .. } => PROFILE_DELETE_METADATA,
    }
}

fn verify_authorized_arguments(
    root: &CanonicalArgumentNodeV0,
    expected_operation_id: &str,
    expected_digest: &str,
) -> Result<(), OpportunityInvocationError> {
    let CanonicalArgumentNodeV0::Object(values) = root else {
        return Err(OpportunityInvocationError::Internal);
    };
    let operation = match values.get("operation_id") {
        Some(CanonicalArgumentNodeV0::String(value)) => value,
        _ => return Err(OpportunityInvocationError::Internal),
    };
    let digest = match values.get("payload_digest") {
        Some(CanonicalArgumentNodeV0::String(value)) => value,
        _ => return Err(OpportunityInvocationError::Internal),
    };
    if operation != expected_operation_id || digest != expected_digest {
        return Err(OpportunityInvocationError::Internal);
    }
    Ok(())
}

fn authenticated_principal(
    actor: &M00AdmittedActor,
) -> Result<AuthenticatedPrincipal, OpportunityInvocationError> {
    let M00AdmittedActor::Authenticated(ids) = actor else {
        return Err(OpportunityInvocationError::Denied);
    };
    AuthenticatedPrincipal::new(ids.tenant_id().clone(), ids.user_id().clone())
        .map_err(|_| OpportunityInvocationError::Internal)
}

fn unix_millis(
    value: ustc_campus_agent_client_protocol::UnixMillis,
) -> Result<OffsetDateTime, OpportunityInvocationError> {
    OffsetDateTime::from_unix_timestamp_nanos(i128::from(value.get()) * 1_000_000)
        .map_err(|_| OpportunityInvocationError::Internal)
}

fn append(run: &mut AgentRun, command: RunCommand) -> Result<(), OpportunityInvocationError> {
    let Decision::Append(event) = run
        .decide(command)
        .map_err(|_| OpportunityInvocationError::Internal)?
    else {
        return Err(OpportunityInvocationError::Internal);
    };
    run.apply(event)
        .map_err(|_| OpportunityInvocationError::Internal)
}

fn record_failed_attempt(
    run: &mut AgentRun,
    counters: &OpportunityInvocationCounters,
    call: &AgentToolCall,
    intent: &EffectIntent,
    code: &'static str,
) -> Result<(), OpportunityInvocationError> {
    let result = AgentToolResult::from_call(
        call,
        AgentToolOutcome::Failed {
            error_code: StableToolCode::parse(code)
                .map_err(|_| OpportunityInvocationError::Internal)?,
        },
    );
    let AgentToolOutcome::Failed { error_code } = result.outcome() else {
        return Err(OpportunityInvocationError::Internal);
    };
    append(
        run,
        RunCommand::RecordEffectReceipt(EffectReceipt {
            effect_id: intent.effect_id.clone(),
            idempotency_key: intent.idempotency_key.clone(),
            outcome: EffectOutcome::Failed {
                error_code: error_code.as_str().to_owned(),
            },
        }),
    )?;
    counters.receipts.fetch_add(1, Ordering::SeqCst);
    append(
        run,
        RunCommand::Fail {
            error_code: error_code.as_str().to_owned(),
        },
    )
}

fn stable_error_code(error: &OpportunityInvocationError) -> &'static str {
    match error {
        OpportunityInvocationError::Profile(_) => "opportunity_profile_rejected",
        OpportunityInvocationError::Planning(_) => "opportunity_plan_rejected",
        OpportunityInvocationError::Denied => "opportunity_policy_denied",
        OpportunityInvocationError::Unavailable => "opportunity_tool_unavailable",
        OpportunityInvocationError::OutcomeUnknown => "opportunity_outcome_unknown",
        OpportunityInvocationError::Internal => "opportunity_internal_error",
    }
}

fn run_spec(
    projection: &ustc_campus_agent_core::invocation::ToolProjectionSnapshot,
    entry: &ustc_campus_agent_core::invocation::ResolvedInvocation,
) -> RunSpec {
    RunSpec {
        schema_version: RUN_SPEC_SCHEMA_VERSION.to_owned(),
        run_id: projection.run_id().as_str().to_owned(),
        tenant_id: entry.tenant_id().as_str().to_owned(),
        installation_id: entry.installation_id().as_str().to_owned(),
        package_id: entry.package_id().as_str().to_owned(),
        package_version: entry.package_version().as_str(),
        component_id: entry.component_id().as_str().to_owned(),
        provider_profile_id: "provider:bounded-no-model".to_owned(),
        grant_snapshot_id: entry.grant_snapshot_id().as_str().to_owned(),
        tool_schema_set_digest: projection.tool_schema_set_digest().as_str().to_owned(),
        budgets: RunBudgets {
            max_turns: 2,
            max_tool_calls: 1,
            max_input_tokens: 1,
            max_output_tokens: 1,
            max_cost_microunits: 1,
            max_retries: 1,
            max_elapsed_ms: 30_000,
        },
    }
}

fn map_projection_error(error: ProjectionAssemblyError) -> OpportunityInvocationError {
    match error {
        ProjectionAssemblyError::Resolution(_) => OpportunityInvocationError::Denied,
        ProjectionAssemblyError::Repository(_) => OpportunityInvocationError::Unavailable,
    }
}

fn map_recheck_error(error: InvocationRecheckError) -> OpportunityInvocationError {
    match error {
        InvocationRecheckError::Authorization(_) => OpportunityInvocationError::Denied,
        InvocationRecheckError::Repository(_) => OpportunityInvocationError::Unavailable,
    }
}

struct OpportunityAuthorityEntry {
    target: InvocationTarget,
    repository: InMemoryInvocationAuthorityRepository,
    grant: CapabilityGrantSnapshot,
}

pub(crate) struct OpportunityAuthorityStore {
    entries: BTreeMap<&'static str, Mutex<OpportunityAuthorityEntry>>,
    mutation: OpportunityAuthorityMutationMode,
}

impl OpportunityAuthorityStore {
    pub(crate) fn new(
        tenant_id: TenantId,
        user_id: UserId,
        enabled: bool,
        grant_active: bool,
        source_evidence_digest: &str,
        mutation: OpportunityAuthorityMutationMode,
    ) -> Result<Self, OpportunityInvocationError> {
        validate_native_component_descriptor(OPPORTUNITY_NATIVE_COMPONENT_DESCRIPTOR)?;
        let manifest = load_package_manifest(OPPORTUNITY_PACKAGE_MANIFEST)
            .map_err(|_| OpportunityInvocationError::Internal)?;
        if manifest.package_id().as_str() != "ustc.opportunity-graph"
            || manifest.package_version().as_str() != "0.1.0"
            || !manifest.components().iter().any(|component| {
                component.kind() == ComponentKind::NativeRustComponent
                    && component.path()
                        == "market/packages/ustc.opportunity-graph/components/native-rust-component.json"
            })
        {
            return Err(OpportunityInvocationError::Internal);
        }
        let source_evidence_digest = CoreSha256Digest::parse(source_evidence_digest.to_owned())
            .map_err(|_| OpportunityInvocationError::Internal)?;
        let combined_source_policy_digest = CoreSha256Digest::from_bytes(
            format!(
                "opportunity-current-source-policy/v1\0{}\0{}",
                manifest.source_policy_digest().as_str(),
                source_evidence_digest.as_str()
            )
            .as_bytes(),
        );

        let mut entries = BTreeMap::new();
        for metadata in ALL_TOOL_METADATA {
            let declared_capability = CapabilityId::parse(metadata.capability_id)
                .map_err(|_| OpportunityInvocationError::Internal)?;
            if !manifest.capabilities().contains(&declared_capability) {
                return Err(OpportunityInvocationError::Internal);
            }
            let (target, repository, grant) = authority_for_ids(
                tenant_id.clone(),
                user_id.clone(),
                enabled,
                grant_active,
                manifest.package_id().clone(),
                manifest.package_version().clone(),
                manifest.package_digest().clone(),
                manifest.capability_manifest_digest().clone(),
                combined_source_policy_digest.clone(),
                &metadata,
            )?;
            if entries
                .insert(
                    metadata.suffix,
                    Mutex::new(OpportunityAuthorityEntry {
                        target,
                        repository,
                        grant,
                    }),
                )
                .is_some()
            {
                return Err(OpportunityInvocationError::Internal);
            }
        }
        Ok(Self { entries, mutation })
    }

    fn entry(
        &self,
        metadata: &ToolMetadata,
    ) -> Result<&Mutex<OpportunityAuthorityEntry>, OpportunityInvocationError> {
        self.entries
            .get(metadata.suffix)
            .ok_or(OpportunityInvocationError::Internal)
    }

    fn mutate_after_projection(
        &self,
        entry: &mut OpportunityAuthorityEntry,
    ) -> Result<(), OpportunityInvocationError> {
        if self.mutation != OpportunityAuthorityMutationMode::RevokeGrantAfterProjection {
            return Ok(());
        }
        let mut revoked = entry.grant.clone();
        revoked.state = GrantState::Revoked;
        entry
            .repository
            .replace_grant_for_testing(revoked.clone(), true)
            .map_err(|_| OpportunityInvocationError::Unavailable)?;
        entry.grant = revoked;
        Ok(())
    }
}

fn projection_request(
    actor: &M00AdmittedActor,
    metadata: &ToolMetadata,
) -> Result<ToolProjectionRequest, OpportunityInvocationError> {
    let M00AdmittedActor::Authenticated(ids) = actor else {
        return Err(OpportunityInvocationError::Denied);
    };
    Ok(ToolProjectionRequest {
        tenant_id: ids.tenant_id().clone(),
        user_id: ids.user_id().clone(),
        run_id: RunId::parse(format!("run:opportunity:{}", metadata.suffix))
            .map_err(|_| OpportunityInvocationError::Internal)?,
        turn_id: TurnId::parse(format!("turn:opportunity:{}", metadata.suffix))
            .map_err(|_| OpportunityInvocationError::Internal)?,
        activation_allowlist: None,
    })
}

#[allow(clippy::too_many_arguments)]
fn authority_for_ids(
    tenant_id: TenantId,
    user_id: UserId,
    enabled: bool,
    grant_active: bool,
    package_id: PackageId,
    package_version: PackageVersion,
    package_digest: CoreSha256Digest,
    capability_manifest_digest: CoreSha256Digest,
    source_policy_digest: CoreSha256Digest,
    metadata: &ToolMetadata,
) -> Result<
    (
        InvocationTarget,
        InMemoryInvocationAuthorityRepository,
        CapabilityGrantSnapshot,
    ),
    OpportunityInvocationError,
> {
    let installation_id = InstallationId::parse("installation:ustc-opportunity-graph")
        .map_err(|_| OpportunityInvocationError::Internal)?;
    let component_id = ComponentId::parse("component:opportunity-private")
        .map_err(|_| OpportunityInvocationError::Internal)?;
    let component_version = ComponentVersion::parse("component-version:1")
        .map_err(|_| OpportunityInvocationError::Internal)?;
    let execution_identity = ExecutionIdentity::parse("builtin:ustc-agentd-opportunity-v0")
        .map_err(|_| OpportunityInvocationError::Internal)?;
    let tool_id =
        ToolId::parse(metadata.tool_id).map_err(|_| OpportunityInvocationError::Internal)?;
    let capability_id = CapabilityId::parse(metadata.capability_id)
        .map_err(|_| OpportunityInvocationError::Internal)?;
    let object_scope = ObjectScope::parse("scope:tenant-private-owner")
        .map_err(|_| OpportunityInvocationError::Internal)?;
    let source_policy = SourcePolicyIdentity {
        id: SourcePolicyId::parse("source-policy:opportunity-package-manifest-v1")
            .map_err(|_| OpportunityInvocationError::Internal)?,
        digest: source_policy_digest,
    };
    let schema = ValidatedToolInputSchemaV0::try_from(UnvalidatedToolInputSchemaV0 {
        dialect: "tool-input-schema/v0".to_owned(),
        root: UnvalidatedSchemaNodeV0::Object {
            properties: vec![
                (
                    "operation_id".to_owned(),
                    UnvalidatedSchemaNodeV0::String {
                        enum_values: Some(vec![metadata.operation_id.to_owned()]),
                    },
                ),
                (
                    "payload_digest".to_owned(),
                    UnvalidatedSchemaNodeV0::String { enum_values: None },
                ),
            ],
            required: vec!["operation_id".to_owned(), "payload_digest".to_owned()],
        },
    })
    .map_err(|_| OpportunityInvocationError::Internal)?;
    let component_digest = CoreSha256Digest::from_bytes(OPPORTUNITY_NATIVE_COMPONENT_DESCRIPTOR);
    let installation = PluginInstallationSnapshot {
        id: installation_id.clone(),
        tenant_id: tenant_id.clone(),
        user_id: user_id.clone(),
        package_id: package_id.clone(),
        package_version: package_version.clone(),
        package_digest: package_digest.clone(),
        component: InstalledComponentIdentity {
            id: component_id.clone(),
            version: component_version.clone(),
            digest: component_digest.clone(),
            execution_identity: execution_identity.clone(),
        },
        state: if enabled {
            InstallationState::Enabled
        } else {
            InstallationState::Disabled
        },
        revision: InstallationRevision::parse("installation-revision:1")
            .map_err(|_| OpportunityInvocationError::Internal)?,
    };
    let grant = CapabilityGrantSnapshot {
        snapshot_id: GrantSnapshotId::parse(format!(
            "grant-snapshot:opportunity:{}",
            metadata.suffix
        ))
        .map_err(|_| OpportunityInvocationError::Internal)?,
        version: GrantVersion::parse("grant-version:1")
            .map_err(|_| OpportunityInvocationError::Internal)?,
        tenant_id: tenant_id.clone(),
        user_id: user_id.clone(),
        installation_id: installation_id.clone(),
        capability_id: capability_id.clone(),
        object_scope: object_scope.clone(),
        confirmation_policy: ConfirmationPolicy::Allow,
        capability_manifest_digest: capability_manifest_digest.clone(),
        state: if grant_active {
            GrantState::Active
        } else {
            GrantState::Revoked
        },
    };
    let policy = InvocationPolicySnapshot {
        snapshot_id: PolicySnapshotId::parse(format!(
            "policy-snapshot:opportunity:{}",
            metadata.suffix
        ))
        .map_err(|_| OpportunityInvocationError::Internal)?,
        revision: PolicyRevision::parse("policy-revision:1")
            .map_err(|_| OpportunityInvocationError::Internal)?,
        capability_id: capability_id.clone(),
        capability_class: Some(metadata.capability_class),
        admitted_execution_identity: Some(execution_identity.clone()),
        admitted_source_policy: Some(source_policy.clone()),
        emergency_blocked: false,
    };
    let target = InvocationTarget {
        installation_id: installation_id.clone(),
        package_id: package_id.clone(),
        package_version: package_version.clone(),
        component_id: component_id.clone(),
        tool_id: tool_id.clone(),
        capability_id: capability_id.clone(),
        object_scope,
    };
    let catalog = CatalogPackageRevision {
        catalog_revision: CatalogRevision::parse("catalog-revision:opportunity-demo")
            .map_err(|_| OpportunityInvocationError::Internal)?,
        package_id,
        package_version,
        package_digest,
        runnable: true,
        revoked: false,
        capability_manifest_digest,
        source_policy: Some(source_policy),
        component: Some(CatalogComponentRevision {
            id: component_id,
            kind: ComponentKind::NativeRustComponent,
            version: component_version,
            digest: component_digest,
            execution_identity,
            declared_capabilities: BTreeSet::from([capability_id.clone()]),
            tool: Some(CatalogToolDefinition {
                id: tool_id,
                model_visible_name: metadata.model_visible_name.to_owned(),
                description: "Execute one consent-bound Opportunity Graph operation".to_owned(),
                capability_id: capability_id.clone(),
                claimed_input_schema_digest: schema.digest().clone(),
                input_schema: Some(schema),
            }),
        }),
    };
    let repository = InMemoryInvocationAuthorityRepository::try_new(
        vec![(target.clone(), catalog)],
        vec![installation],
        vec![grant.clone()],
        vec![grant.snapshot_id.clone()],
        vec![policy],
    )
    .map_err(|_| OpportunityInvocationError::Unavailable)?;
    Ok((target, repository, grant))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_component_descriptor_accepts_only_exact_identity_and_operations() {
        assert!(
            validate_native_component_descriptor(OPPORTUNITY_NATIVE_COMPONENT_DESCRIPTOR).is_ok()
        );

        for (field, replacement) in [
            ("packageId", serde_json::json!("ustc.other-plugin")),
            ("compositionOwner", serde_json::json!("apps/other-daemon")),
            (
                "operations",
                serde_json::json!([
                    "profile.academic.create",
                    "profile.academic.view",
                    "planner.generate"
                ]),
            ),
        ] {
            let mut value: serde_json::Value =
                serde_json::from_slice(OPPORTUNITY_NATIVE_COMPONENT_DESCRIPTOR)
                    .expect("checked descriptor JSON");
            value[field] = replacement;
            let mutated = serde_json::to_vec(&value).expect("mutated descriptor JSON");
            assert!(matches!(
                validate_native_component_descriptor(&mutated),
                Err(OpportunityInvocationError::Internal)
            ));
        }
    }
}
