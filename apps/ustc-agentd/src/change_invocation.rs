//! Bounded production-path ChangeRadar invocation spine.
//!
//! This composition-owned adapter is the only place that sees M00 actor
//! identity, M20 current authority, M30 run state, M40 protocol routing and the
//! owning ChangeRadar query service. It contains no direct repository fallback.

use std::collections::BTreeSet;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use ustc_agent_tool_protocol::{
    AgentToolCall, AgentToolOutcome, AgentToolResult, CanonicalArgumentNodeV0,
    CanonicalArgumentValueV0, ProviderToolCallId as ProtocolProviderToolCallId, Sha256Digest,
    StableToolCode, UnvalidatedArgumentValueV0,
};
use ustc_campus_agent_application_ingress::{
    ChangeFeedInvocationError, ChangeFeedInvocationOutcome, ChangeFeedInvocationPort,
};
use ustc_campus_agent_change_radar::{
    BoardFeedPolicy, BoardId, ChangeFeedQueryError, ChangeFeedQueryService,
    InMemoryChangeRadarRepository,
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
    InMemoryInvocationAuthorityRepository, InvocationAuthorityService, InvocationRecheckError,
    ProjectionAssemblyError,
};
use ustc_campus_agent_core::request_context::M00AdmittedActor;
use ustc_campus_agent_runtime::{
    AgentRun, Decision, EffectIntent, EffectOutcome, EffectReceipt, RUN_SPEC_SCHEMA_VERSION,
    RunBudgets, RunCommand, RunSpec, ToolCallProposal,
};

#[derive(Clone, Default)]
pub(crate) struct ChangeInvocationCounters {
    intents: Arc<AtomicU64>,
    executions: Arc<AtomicU64>,
    receipts: Arc<AtomicU64>,
}

impl ChangeInvocationCounters {
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

pub(crate) struct ChangeInvocationSpine<'a> {
    repository: &'a InMemoryChangeRadarRepository,
    policy: &'a BoardFeedPolicy,
    market_enabled: bool,
    market_grant_active: bool,
    source_evidence_digest: &'a str,
    counters: ChangeInvocationCounters,
}

impl<'a> ChangeInvocationSpine<'a> {
    pub(crate) fn new(
        repository: &'a InMemoryChangeRadarRepository,
        policy: &'a BoardFeedPolicy,
        market_enabled: bool,
        market_grant_active: bool,
        source_evidence_digest: &'a str,
        counters: ChangeInvocationCounters,
    ) -> Self {
        Self {
            repository,
            policy,
            market_enabled,
            market_grant_active,
            source_evidence_digest,
            counters,
        }
    }
}

impl ChangeFeedInvocationPort for ChangeInvocationSpine<'_> {
    fn invoke(
        &self,
        actor: &M00AdmittedActor,
        board_id: &BoardId,
    ) -> Result<ChangeFeedInvocationOutcome, ChangeFeedInvocationError> {
        let (request, target, repository) = authority(
            actor,
            self.market_enabled,
            self.market_grant_active,
            self.source_evidence_digest,
        )?;
        let authority = InvocationAuthorityService::new(repository);
        let projection = authority
            .resolve_projection(request, vec![target])
            .map_err(map_projection_error)?;
        let toolset = projection
            .agent_toolset_view()
            .map_err(|_| ChangeFeedInvocationError::Internal)?;
        let arguments =
            CanonicalArgumentValueV0::try_from(UnvalidatedArgumentValueV0::Object(vec![(
                "board_id".to_owned(),
                UnvalidatedArgumentValueV0::String(board_id.as_str().to_owned()),
            )]))
            .map_err(|_| ChangeFeedInvocationError::Internal)?;
        let call = toolset
            .bind_call(
                ProtocolProviderToolCallId::parse("provider-call:change-feed")
                    .map_err(|_| ChangeFeedInvocationError::Internal)?,
                "ustc_change_feed",
                arguments,
            )
            .map_err(|_| ChangeFeedInvocationError::Internal)?;
        let entry = projection
            .entries()
            .first()
            .ok_or(ChangeFeedInvocationError::Internal)?;
        let mut run = AgentRun::new(run_spec(&projection, entry))
            .map_err(|_| ChangeFeedInvocationError::Internal)?;
        append(&mut run, RunCommand::Prepare)?;
        append(&mut run, RunCommand::StartHarnessTurn)?;
        let proposed = ProposedToolCall {
            provider_tool_call_id: CoreProviderToolCallId::parse(
                call.provider_tool_call_id().as_str().to_owned(),
            )
            .map_err(|_| ChangeFeedInvocationError::Internal)?,
            model_visible_name: call.model_visible_name().to_owned(),
            dispatch_key: call.route_ref().as_str().to_owned(),
            arguments: call.arguments().clone(),
            claimed_argument_digest: call.arguments().digest().clone(),
        };
        let authorized = authority
            .recheck_invocation(&projection, proposed)
            .map_err(map_recheck_error)?;
        append(
            &mut run,
            RunCommand::ProposeToolCall(ToolCallProposal::from(&call)),
        )?;
        let authorized_board = match authorized.arguments().root() {
            CanonicalArgumentNodeV0::Object(values) => match values.get("board_id") {
                Some(CanonicalArgumentNodeV0::String(value)) => value,
                _ => return Err(ChangeFeedInvocationError::Internal),
            },
            _ => return Err(ChangeFeedInvocationError::Internal),
        };
        if authorized_board != board_id.as_str() {
            return Err(ChangeFeedInvocationError::Internal);
        }
        let intent = EffectIntent {
            effect_id: "effect:change-feed".to_owned(),
            idempotency_key: "effect-idem:change-feed".to_owned(),
            call_id: call.provider_tool_call_id().as_str().to_owned(),
            tool_name: call.model_visible_name().to_owned(),
            arguments_digest: call.arguments().digest().as_str().to_owned(),
            capability_id: authorized.entry().capability_id().as_str().to_owned(),
            grant_snapshot_id: authorized.entry().grant_snapshot_id().as_str().to_owned(),
            tool_schema_set_digest: projection.tool_schema_set_digest().as_str().to_owned(),
        };
        append(&mut run, RunCommand::ApproveToolCall(intent.clone()))?;
        self.counters.intents.fetch_add(1, Ordering::SeqCst);

        let outcome = if board_id == self.policy.board_id() {
            ChangeFeedQueryService::new(self.repository)
                .execute(self.policy)
                .map(ChangeFeedInvocationOutcome::Found)
        } else {
            Ok(ChangeFeedInvocationOutcome::NotFound(board_id.clone()))
        };
        self.counters.executions.fetch_add(1, Ordering::SeqCst);
        let outcome = match outcome {
            Ok(value) => value,
            Err(error) => {
                record_failed_attempt(
                    &mut run,
                    &self.counters,
                    &call,
                    &intent,
                    stable_query_error_code(&error),
                )?;
                return Err(ChangeFeedInvocationError::Downstream(error));
            }
        };
        let output_bytes = match &outcome {
            ChangeFeedInvocationOutcome::Found(receipt) => receipt.atom().as_bytes(),
            ChangeFeedInvocationOutcome::NotFound(board) => board.as_str().as_bytes(),
        };
        let mut output_fingerprint = b"change-feed-result/v1\0".to_vec();
        output_fingerprint.extend_from_slice(output_bytes);
        let output_digest = Sha256Digest::from_bytes(&output_fingerprint);
        let result = AgentToolResult::from_call(
            &call,
            AgentToolOutcome::Succeeded {
                output_digest: output_digest.clone(),
            },
        );
        let AgentToolOutcome::Succeeded { output_digest } = result.outcome() else {
            return Err(ChangeFeedInvocationError::Internal);
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

fn append(run: &mut AgentRun, command: RunCommand) -> Result<(), ChangeFeedInvocationError> {
    let Decision::Append(event) = run
        .decide(command)
        .map_err(|_| ChangeFeedInvocationError::Internal)?
    else {
        return Err(ChangeFeedInvocationError::Internal);
    };
    run.apply(event)
        .map_err(|_| ChangeFeedInvocationError::Internal)
}

fn record_failed_attempt(
    run: &mut AgentRun,
    counters: &ChangeInvocationCounters,
    call: &AgentToolCall,
    intent: &EffectIntent,
    code: &'static str,
) -> Result<(), ChangeFeedInvocationError> {
    let result = AgentToolResult::from_call(
        call,
        AgentToolOutcome::Failed {
            error_code: StableToolCode::parse(code)
                .map_err(|_| ChangeFeedInvocationError::Internal)?,
        },
    );
    let AgentToolOutcome::Failed { error_code } = result.outcome() else {
        return Err(ChangeFeedInvocationError::Internal);
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

fn map_projection_error(error: ProjectionAssemblyError) -> ChangeFeedInvocationError {
    match error {
        ProjectionAssemblyError::Resolution(_) => ChangeFeedInvocationError::Denied,
        ProjectionAssemblyError::Repository(_) => ChangeFeedInvocationError::Unavailable,
    }
}

fn map_recheck_error(error: InvocationRecheckError) -> ChangeFeedInvocationError {
    match error {
        InvocationRecheckError::Authorization(_) => ChangeFeedInvocationError::Denied,
        InvocationRecheckError::Repository(_) => ChangeFeedInvocationError::Unavailable,
    }
}

const fn stable_query_error_code(error: &ChangeFeedQueryError) -> &'static str {
    match error {
        ChangeFeedQueryError::Repository(_) => "change_feed_store_unavailable",
        ChangeFeedQueryError::Projection => "change_feed_projection_failed",
    }
}

fn authority(
    actor: &M00AdmittedActor,
    enabled: bool,
    grant_active: bool,
    source_evidence_digest: &str,
) -> Result<
    (
        ToolProjectionRequest,
        InvocationTarget,
        InMemoryInvocationAuthorityRepository,
    ),
    ChangeFeedInvocationError,
> {
    let (tenant_id, user_id) = match actor {
        M00AdmittedActor::Public => (
            TenantId::parse("tenant:campus-public")
                .map_err(|_| ChangeFeedInvocationError::Internal)?,
            UserId::parse("user:anonymous").map_err(|_| ChangeFeedInvocationError::Internal)?,
        ),
        M00AdmittedActor::Authenticated(ids) => (ids.tenant_id().clone(), ids.user_id().clone()),
    };
    authority_for_ids(
        tenant_id,
        user_id,
        enabled,
        grant_active,
        source_evidence_digest,
    )
}

fn authority_for_ids(
    tenant_id: TenantId,
    user_id: UserId,
    enabled: bool,
    grant_active: bool,
    source_evidence_digest: &str,
) -> Result<
    (
        ToolProjectionRequest,
        InvocationTarget,
        InMemoryInvocationAuthorityRepository,
    ),
    ChangeFeedInvocationError,
> {
    let installation_id = InstallationId::parse("installation:ustc-change-radar")
        .map_err(|_| ChangeFeedInvocationError::Internal)?;
    let package_id =
        PackageId::parse("ustc.change-radar").map_err(|_| ChangeFeedInvocationError::Internal)?;
    let package_version =
        PackageVersion::parse("0.1.0").map_err(|_| ChangeFeedInvocationError::Internal)?;
    let component_id = ComponentId::parse("component:change-feed")
        .map_err(|_| ChangeFeedInvocationError::Internal)?;
    let component_version = ComponentVersion::parse("component-version:1")
        .map_err(|_| ChangeFeedInvocationError::Internal)?;
    let execution_identity = ExecutionIdentity::parse("builtin:ustc-agentd-change-feed-v0")
        .map_err(|_| ChangeFeedInvocationError::Internal)?;
    let tool_id =
        ToolId::parse("tool:change-feed").map_err(|_| ChangeFeedInvocationError::Internal)?;
    let capability_id = CapabilityId::parse("campus.public_changes.read")
        .map_err(|_| ChangeFeedInvocationError::Internal)?;
    let object_scope = ObjectScope::parse("scope:campus-public")
        .map_err(|_| ChangeFeedInvocationError::Internal)?;
    let source_evidence_digest = CoreSha256Digest::parse(source_evidence_digest.to_owned())
        .map_err(|_| ChangeFeedInvocationError::Internal)?;
    let source_policy = SourcePolicyIdentity {
        id: SourcePolicyId::parse("source-policy:change-demo-reviewed-v1")
            .map_err(|_| ChangeFeedInvocationError::Internal)?,
        digest: CoreSha256Digest::from_bytes(
            format!(
                "change-source-policy/v1\0demo-reviewed\0{}",
                source_evidence_digest.as_str()
            )
            .as_bytes(),
        ),
    };
    let schema = ValidatedToolInputSchemaV0::try_from(UnvalidatedToolInputSchemaV0 {
        dialect: "tool-input-schema/v0".to_owned(),
        root: UnvalidatedSchemaNodeV0::Object {
            properties: vec![(
                "board_id".to_owned(),
                UnvalidatedSchemaNodeV0::String { enum_values: None },
            )],
            required: vec!["board_id".to_owned()],
        },
    })
    .map_err(|_| ChangeFeedInvocationError::Internal)?;
    let package_digest =
        CoreSha256Digest::from_bytes(b"package/ustc.change-radar/0.1.0/fixed-first-party");
    let component_digest = CoreSha256Digest::from_bytes(
        b"component/change-feed/component-version:1/builtin:ustc-agentd-change-feed-v0",
    );
    let capability_manifest_digest = CoreSha256Digest::from_bytes(
        b"capability/campus.public_changes.read/public-read/scope:campus-public/no-secrets/no-external-effect",
    );
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
            .map_err(|_| ChangeFeedInvocationError::Internal)?,
    };
    let grant = CapabilityGrantSnapshot {
        snapshot_id: GrantSnapshotId::parse("grant-snapshot:change-public-read")
            .map_err(|_| ChangeFeedInvocationError::Internal)?,
        version: GrantVersion::parse("grant-version:1")
            .map_err(|_| ChangeFeedInvocationError::Internal)?,
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
        snapshot_id: PolicySnapshotId::parse("policy-snapshot:change-public-read")
            .map_err(|_| ChangeFeedInvocationError::Internal)?,
        revision: PolicyRevision::parse("policy-revision:1")
            .map_err(|_| ChangeFeedInvocationError::Internal)?,
        capability_id: capability_id.clone(),
        capability_class: Some(CapabilityClass::PublicRead),
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
        catalog_revision: CatalogRevision::parse("catalog-revision:change-demo")
            .map_err(|_| ChangeFeedInvocationError::Internal)?,
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
                model_visible_name: "ustc_change_feed".to_owned(),
                description: "Read one reviewed USTC semantic-change feed".to_owned(),
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
        vec![grant.snapshot_id],
        vec![policy],
    )
    .map_err(|_| ChangeFeedInvocationError::Unavailable)?;
    Ok((
        ToolProjectionRequest {
            tenant_id,
            user_id,
            run_id: RunId::parse("run:change-feed")
                .map_err(|_| ChangeFeedInvocationError::Internal)?,
            turn_id: TurnId::parse("turn:change-feed")
                .map_err(|_| ChangeFeedInvocationError::Internal)?,
            activation_allowlist: None,
        },
        target,
        repository,
    ))
}
