//! Compose current M20 resolution with durable M30 intent/receipt and a bounded adapter.
use super::*;
use crate::chat_tools::{ChatToolExecution, canonical_arguments};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use ustc_campus_agent_adapters::mcp::{BindingState, McpError};
use ustc_campus_agent_core::market::authority::{
    InMemoryInvocationAuthorityRepository, InvocationAuthorityService,
};
use ustc_campus_agent_runtime::{
    AgentRun, Decision, EffectIntent, EffectOutcome, EffectReceipt, RUN_SPEC_SCHEMA_VERSION,
    RunBudgets, RunCommand, RunEvent, RunSpec, ToolCallProposal,
};
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct JournalRun {
    spec: RunSpec,
    events: Vec<RunEvent>,
}
impl JournalRun {
    pub(super) fn validate(&self) -> Result<(), PluginError> {
        if self.events.len() > 16 {
            return Err(PluginError::Capacity);
        }
        AgentRun::replay(self.spec.clone(), &self.events).map_err(|_| PluginError::Unavailable)?;
        Ok(())
    }
}
impl PluginRuntime {
    #[cfg(test)]
    pub(super) async fn execute(
        &self,
        tenant: &TenantId,
        user: &UserId,
        name: &str,
        arguments: Value,
    ) -> ChatToolExecution {
        match self.session(tenant, user).await {
            Ok(session) => self.execute_frozen(&session, name, arguments).await,
            Err(_) => ChatToolExecution::denied(json!({"code":"plugin_permission_denied"})),
        }
    }
    pub(crate) async fn execute_frozen(
        &self,
        session: &PluginToolSession,
        name: &str,
        arguments: Value,
    ) -> ChatToolExecution {
        match self.invoke(session, name, arguments).await {
            Ok(value) => ChatToolExecution::succeeded(value),
            Err(PluginError::Denied | PluginError::NotFound) => {
                ChatToolExecution::denied(json!({"code":"plugin_permission_denied"}))
            }
            Err(PluginError::NotReady) => {
                ChatToolExecution::denied(json!({"code":"plugin_review_required"}))
            }
            Err(PluginError::Capacity) => ChatToolExecution::failed(json!({
                "code":"plugin_capacity_exceeded",
                "message":"The plugin runtime reached a configured capacity limit. Do not automatically retry. Ask the operator to inspect capacity and retained execution records; restarting does not release retained evidence."
            })),
            Err(PluginError::InvalidRequest) => ChatToolExecution::failed(json!({
                "code":"plugin_invalid_arguments",
                "message":"Check the tool schema and description, then correct the arguments within the remaining tool budget. This error does not require a new permission grant."
            })),
            Err(_) => ChatToolExecution::failed(json!({"code":"plugin_execution_unavailable"})),
        }
    }
    async fn invoke(
        &self,
        session: &PluginToolSession,
        name: &str,
        arguments: Value,
    ) -> Result<Value, PluginError> {
        if !session.runtime.ptr_eq(&Arc::downgrade(&self.state)) {
            return Err(PluginError::Denied);
        }
        let frozen = session.bindings.get(name).ok_or(PluginError::Denied)?;
        let mut state = self.state.lock().await;
        let current = state.owned(&session.tenant, &session.user, &frozen.installation_id)?;
        // Reject stale model proposals before rediscovery, adapter I/O or effect intent.
        // A new installation revision or grant must get a new model projection.
        if current.state() != ManagedInstallationState::Enabled
            || current.revision() != &frozen.installation_revision
        {
            return Err(PluginError::Denied);
        }
        let probe = state
            .probes
            .get(&frozen.installation_id)
            .ok_or(PluginError::NotReady)?;
        if probe.readiness.digest() != &frozen.readiness_digest
            || !probe.tools.iter().any(|tool| tool == &frozen.tool)
        {
            return Err(PluginError::Denied);
        }
        let grant = self.current_grant(&state.authority, &current, &frozen.tool.capability_id)?;
        if grant.snapshot_id() != &frozen.grant_snapshot_id
            || grant.version() != &frozen.grant_version
        {
            return Err(PluginError::Denied);
        }
        self.ensure_active(&mut state, &session.tenant, &session.user, &current)
            .await?;
        let tool = &frozen.tool;
        let package = self.package(
            current.package_pin().package_id().as_str(),
            &current.package_pin().package_version().as_str(),
        )?;
        let argument =
            canonical_arguments(&arguments.to_string()).map_err(|_| PluginError::InvalidRequest)?;
        let (mut run, mut record, intent) = prepare(
            &current,
            &grant,
            package,
            &frozen.component_id,
            tool,
            argument,
        )?;
        if state.authority.runs.len() >= 1024 {
            return Err(PluginError::Capacity);
        }
        let mut next = state.authority.clone();
        next.runs.push(record.clone());
        state.commit(next)?;
        // This mutex is the owner transaction: disable/revoke linearizes before or after this admitted call.
        let outcome = match package
            .component(&frozen.component_id)
            .ok_or(PluginError::Denied)?
        {
            RuntimeComponent::Skill { source } => super::skill_context::read(source, &arguments),
            RuntimeComponent::Mcp { .. } => {
                let probe = state
                    .probes
                    .get_mut(current.installation_id())
                    .ok_or(PluginError::NotReady)?;
                let probe = if probe.component_id == frozen.component_id {
                    probe
                } else {
                    probe
                        .additional
                        .get_mut(&frozen.component_id)
                        .ok_or(PluginError::NotReady)?
                };
                let wire = probe
                    .wire_names
                    .get(name)
                    .ok_or(PluginError::NotReady)?
                    .clone();
                let digest = probe
                    .transport_digest
                    .clone()
                    .ok_or(PluginError::NotReady)?;
                let client = probe.client.as_mut().ok_or(PluginError::NotReady)?;
                match client.call_tool(&digest, &wire, &arguments).await {
                    Ok(result) if !result.is_error => Ok(json!({
                        "kind":"mcp_result",
                        "text":result.text,
                        "structured_content":result.structured_content
                    })),
                    Ok(_) => Err(PluginError::Unavailable),
                    Err(error) => Err(mcp_failure(error, client.state())),
                }
            }
        };
        let outcome = outcome.and_then(|value| {
            if serde_json::to_vec(&value)
                .map_err(|_| PluginError::Unavailable)?
                .len()
                > 60 * 1024
            {
                return Err(PluginError::Capacity);
            }
            Ok(value)
        });
        let effect_outcome = match &outcome {
            Ok(value) => EffectOutcome::Succeeded {
                output_digest: Sha256Digest::from_bytes(value.to_string().as_bytes())
                    .as_str()
                    .to_owned(),
            },
            Err(_) => EffectOutcome::Failed {
                error_code: "plugin_execution_failed".to_owned(),
            },
        };
        append(
            &mut run,
            &mut record,
            RunCommand::RecordEffectReceipt(EffectReceipt {
                effect_id: intent.effect_id,
                idempotency_key: intent.idempotency_key,
                outcome: effect_outcome.clone(),
            }),
        )?;
        match effect_outcome {
            EffectOutcome::Succeeded { output_digest } => {
                append(&mut run, &mut record, RunCommand::StartHarnessTurn)?;
                append(
                    &mut run,
                    &mut record,
                    RunCommand::Complete { output_digest },
                )?;
            }
            EffectOutcome::Failed { error_code } => {
                append(&mut run, &mut record, RunCommand::Fail { error_code })?
            }
        }
        let mut next = state.authority.clone();
        let last = next.runs.last_mut().ok_or(PluginError::Unavailable)?;
        *last = record;
        state.commit(next)?;
        outcome
    }
}
fn mcp_failure(error: McpError, state: BindingState) -> PluginError {
    if error == McpError::InvalidArguments {
        return PluginError::InvalidRequest;
    }
    // M51 owns invalidation. Project its resulting state, not a duplicate list
    // of protocol errors that happen to quarantine today's transport.
    if state != BindingState::Active {
        PluginError::NotReady
    } else {
        PluginError::Unavailable
    }
}
fn prepare(
    current: &InstallationSnapshot,
    grant: &GrantSnapshot,
    package: &RuntimePackage,
    component_id: &ComponentId,
    tool: &CatalogToolDefinition,
    arguments: CanonicalArgumentValueV0,
) -> Result<(AgentRun, JournalRun, EffectIntent), PluginError> {
    let pin = current.package_pin();
    let component = pin
        .components()
        .iter()
        .find(|component| component.component_id() == component_id)
        .ok_or(PluginError::Unsupported)?;
    let source = SourcePolicyIdentity {
        id: SourcePolicyId::parse("source-policy:reviewed-package")
            .map_err(|_| PluginError::Unavailable)?,
        digest: package.manifest.source_policy_digest().clone(),
    };
    let target = InvocationTarget {
        installation_id: current.installation_id().clone(),
        package_id: pin.package_id().clone(),
        package_version: pin.package_version().clone(),
        component_id: component.component_id().clone(),
        tool_id: tool.id.clone(),
        capability_id: tool.capability_id.clone(),
        object_scope: grant.scope().object_scope().clone(),
    };
    let catalog = CatalogPackageRevision {
        catalog_revision: pin.catalog_revision().clone(),
        package_id: pin.package_id().clone(),
        package_version: pin.package_version().clone(),
        package_digest: pin.package_digest().clone(),
        runnable: true,
        revoked: false,
        capability_manifest_digest: pin.capability_manifest_digest().clone(),
        source_policy: Some(source.clone()),
        component: Some(CatalogComponentRevision {
            id: component.component_id().clone(),
            kind: component.kind(),
            version: component.version().clone(),
            digest: component.digest().clone(),
            execution_identity: component.execution_identity().clone(),
            declared_capabilities: package.manifest.capabilities().iter().cloned().collect(),
            tool: Some(tool.clone()),
        }),
    };
    let installation = current
        .to_resolver_snapshot_for_component(component_id)
        .ok_or(PluginError::Denied)?;
    let grant = grant.to_resolver_snapshot();
    let policy = InvocationPolicySnapshot {
        snapshot_id: PolicySnapshotId::parse("policy-snapshot:package-public-read-v1")
            .map_err(|_| PluginError::Unavailable)?,
        revision: PolicyRevision::parse("policy-revision:1")
            .map_err(|_| PluginError::Unavailable)?,
        capability_id: tool.capability_id.clone(),
        capability_class: Some(CapabilityClass::PublicRead),
        admitted_execution_identity: Some(component.execution_identity().clone()),
        admitted_source_policy: Some(source),
        emergency_blocked: false,
    };
    let repository = InMemoryInvocationAuthorityRepository::try_new(
        vec![(target.clone(), catalog)],
        vec![installation],
        vec![grant.clone()],
        vec![grant.snapshot_id],
        vec![policy],
    )
    .map_err(|_| PluginError::Unavailable)?;
    let service = InvocationAuthorityService::new(repository);
    let nonce = random_nonce()?;
    let run_id = format!("run:{nonce}");
    let projection = service
        .resolve_projection(
            ToolProjectionRequest {
                tenant_id: current.tenant_id().clone(),
                user_id: current.user_id().clone(),
                run_id: RunId::parse(run_id.clone()).map_err(|_| PluginError::Unavailable)?,
                turn_id: TurnId::parse(format!("turn:{nonce}"))
                    .map_err(|_| PluginError::Unavailable)?,
                activation_allowlist: None,
            },
            vec![target],
        )
        .map_err(|_| PluginError::Denied)?;
    let call = projection
        .agent_toolset_view()
        .map_err(|_| PluginError::Unavailable)?
        .bind_call(
            ustc_agent_tool_protocol::ProviderToolCallId::parse(format!("provider-call:{nonce}"))
                .map_err(|_| PluginError::Unavailable)?,
            &tool.model_visible_name,
            arguments,
        )
        .map_err(|_| PluginError::Denied)?;
    let authorized = service
        .recheck_invocation(
            &projection,
            ProposedToolCall {
                provider_tool_call_id: ProviderToolCallId::parse(
                    call.provider_tool_call_id().as_str().to_owned(),
                )
                .map_err(|_| PluginError::Unavailable)?,
                model_visible_name: call.model_visible_name().to_owned(),
                dispatch_key: call.route_ref().as_str().to_owned(),
                arguments: call.arguments().clone(),
                claimed_argument_digest: call.arguments().digest().clone(),
            },
        )
        .map_err(|_| PluginError::Denied)?;
    let entry = authorized.entry();
    let spec = RunSpec {
        schema_version: RUN_SPEC_SCHEMA_VERSION.to_owned(),
        run_id,
        tenant_id: current.tenant_id().as_str().to_owned(),
        installation_id: current.installation_id().as_str().to_owned(),
        package_id: pin.package_id().as_str().to_owned(),
        package_version: pin.package_version().as_str(),
        component_id: component.component_id().as_str().to_owned(),
        provider_profile_id: "provider:plugin-execution".to_owned(),
        grant_snapshot_id: entry.grant_snapshot_id().as_str().to_owned(),
        tool_schema_set_digest: projection.tool_schema_set_digest().as_str().to_owned(),
        budgets: RunBudgets {
            max_turns: 2,
            max_tool_calls: 1,
            max_input_tokens: 1,
            max_output_tokens: 1,
            max_cost_microunits: 1,
            max_retries: 1,
            max_elapsed_ms: 30000,
        },
    };
    let mut run = AgentRun::new(spec.clone()).map_err(|_| PluginError::Unavailable)?;
    let mut record = JournalRun {
        spec,
        events: Vec::new(),
    };
    append(&mut run, &mut record, RunCommand::Prepare)?;
    append(&mut run, &mut record, RunCommand::StartHarnessTurn)?;
    append(
        &mut run,
        &mut record,
        RunCommand::ProposeToolCall(ToolCallProposal::from(&call)),
    )?;
    let intent = EffectIntent {
        tool_schema_set_digest: projection.tool_schema_set_digest().as_str().to_owned(),
        effect_id: format!("effect:{nonce}"),
        idempotency_key: format!("effect-idem:{nonce}"),
        call_id: call.provider_tool_call_id().as_str().to_owned(),
        tool_name: call.model_visible_name().to_owned(),
        arguments_digest: call.arguments().digest().as_str().to_owned(),
        capability_id: entry.capability_id().as_str().to_owned(),
        grant_snapshot_id: entry.grant_snapshot_id().as_str().to_owned(),
    };
    append(
        &mut run,
        &mut record,
        RunCommand::ApproveToolCall(intent.clone()),
    )?;
    Ok((run, record, intent))
}
fn append(
    run: &mut AgentRun,
    record: &mut JournalRun,
    command: RunCommand,
) -> Result<(), PluginError> {
    match run.decide(command).map_err(|_| PluginError::Unavailable)? {
        Decision::Append(event) => {
            run.apply(event.clone())
                .map_err(|_| PluginError::Unavailable)?;
            record.events.push(event);
            Ok(())
        }
        Decision::AlreadyApplied => Ok(()),
    }
}
fn random_nonce() -> Result<String, PluginError> {
    use std::io::Read;
    let mut bytes = [0u8; 24];
    std::fs::File::open("/dev/urandom")
        .and_then(|mut f| f.read_exact(&mut bytes))
        .map_err(|_| PluginError::Unavailable)?;
    Ok(bytes.iter().map(|byte| format!("{byte:02x}")).collect())
}
