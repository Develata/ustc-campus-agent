mod support;

use support::proof_authority;
use ustc_agent_tool_protocol::{
    AgentTool, AgentToolCall, AgentToolDefinition, AgentToolOutcome, AgentToolResult,
    AgentToolsetView, CanonicalArgumentValueV0, ProjectionSnapshotId, ProtocolRunId,
    ProtocolTurnId, ProviderToolCallId as ProtocolProviderToolCallId, Sha256Digest, ToolRouteRef,
    UnvalidatedArgumentValueV0,
};
use ustc_campus_agent_core::invocation::{
    AuthorizedInvocation, CurrentDenyState, InvocationAuthorityCandidate,
    InvocationAuthorizationError, InvocationResolver, ProposedToolCall,
    ProviderToolCallId as CoreProviderToolCallId, ToolProjectionSnapshot, TurnId, authorize_call,
};
use ustc_campus_agent_runtime::ToolCallProposal;

fn current_deny_state(candidate: &InvocationAuthorityCandidate) -> CurrentDenyState {
    let Some(installation) = candidate.installation.clone() else {
        panic!("proof authority must include installation")
    };
    let Some(grant) = candidate.grant.clone() else {
        panic!("proof authority must include grant")
    };
    let Some(catalog) = candidate.catalog.as_ref() else {
        panic!("proof authority must include catalog")
    };
    CurrentDenyState {
        tenant_id: installation.tenant_id.clone(),
        user_id: installation.user_id.clone(),
        catalog_revoked: catalog.revoked,
        installation: Some(installation),
        grant,
        policy: candidate.policy.clone(),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ObservedExecution {
    installation_id: String,
    package_id: String,
    component_id: String,
    execution_identity: String,
    argument_digest: String,
}

#[derive(Default)]
struct FakePluginExecutor {
    observed: Vec<ObservedExecution>,
}

impl FakePluginExecutor {
    fn execute(&mut self, invocation: &AuthorizedInvocation) -> Sha256Digest {
        let entry = invocation.entry();
        self.observed.push(ObservedExecution {
            installation_id: entry.installation_id().as_str().to_owned(),
            package_id: entry.package_id().as_str().to_owned(),
            component_id: entry.component_id().as_str().to_owned(),
            execution_identity: entry.execution_identity().as_str().to_owned(),
            argument_digest: invocation.arguments().digest().as_str().to_owned(),
        });
        Sha256Digest::from_bytes(invocation.arguments().canonical_bytes())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum FakeGatewayError {
    ProjectionMismatch,
    InvalidProviderCallId,
    Authorization(InvocationAuthorizationError),
}

struct FakeToolGateway {
    projection: ToolProjectionSnapshot,
    current: CurrentDenyState,
    executor: FakePluginExecutor,
}

impl FakeToolGateway {
    fn new(projection: ToolProjectionSnapshot, current: CurrentDenyState) -> Self {
        Self {
            projection,
            current,
            executor: FakePluginExecutor::default(),
        }
    }

    fn execute(&mut self, call: &AgentToolCall) -> Result<AgentToolResult, FakeGatewayError> {
        if call.run_id().as_str() != self.projection.run_id().as_str()
            || call.turn_id().as_str() != self.projection.turn_id().as_str()
            || call.projection_snapshot_id().as_str() != self.projection.snapshot_id()
        {
            return Err(FakeGatewayError::ProjectionMismatch);
        }
        let provider_tool_call_id =
            CoreProviderToolCallId::parse(call.provider_tool_call_id().as_str().to_owned())
                .map_err(|_| FakeGatewayError::InvalidProviderCallId)?;
        let proposed = ProposedToolCall {
            provider_tool_call_id,
            model_visible_name: call.model_visible_name().to_owned(),
            dispatch_key: call.route_ref().as_str().to_owned(),
            arguments: call.arguments().clone(),
            claimed_argument_digest: call.arguments().digest().clone(),
        };
        let authorized = authorize_call(&self.projection, self.current.clone(), proposed)
            .map_err(FakeGatewayError::Authorization)?;
        let output_digest = self.executor.execute(&authorized);
        Ok(AgentToolResult::from_call(
            call,
            AgentToolOutcome::Succeeded { output_digest },
        ))
    }
}

fn resolved_view() -> (ToolProjectionSnapshot, CurrentDenyState, AgentToolsetView) {
    let (request, candidate) = proof_authority();
    let current = current_deny_state(&candidate);
    let projection = match InvocationResolver::resolve_projection(request, vec![candidate]) {
        Ok(projection) => projection,
        Err(error) => panic!("proof projection must resolve: {error}"),
    };
    let view = match projection.agent_toolset_view() {
        Ok(view) => view,
        Err(error) => panic!("resolved projection must map to Agent view: {error}"),
    };
    (projection, current, view)
}

fn canonical_query(value: &str) -> CanonicalArgumentValueV0 {
    match CanonicalArgumentValueV0::try_from(UnvalidatedArgumentValueV0::Object(vec![(
        "query".to_owned(),
        UnvalidatedArgumentValueV0::String(value.to_owned()),
    )])) {
        Ok(arguments) => arguments,
        Err(error) => panic!("query arguments must canonicalize: {error}"),
    }
}

fn bind_proof_call(view: &AgentToolsetView) -> AgentToolCall {
    let call_id = match ProtocolProviderToolCallId::parse("provider-call:proof") {
        Ok(call_id) => call_id,
        Err(error) => panic!("provider call id must parse: {error}"),
    };
    match view.bind_call(call_id, "proof_tool", canonical_query("calendar")) {
        Ok(call) => call,
        Err(error) => panic!("projected tool must bind: {error}"),
    }
}

fn bind_forged_call(
    projection: &ToolProjectionSnapshot,
    model_visible_name: &str,
    route_ref: &str,
    provider_call_id: &str,
) -> AgentToolCall {
    let [entry] = projection.entries() else {
        panic!("proof projection must contain exactly one entry")
    };
    let definition = match AgentToolDefinition::new(
        model_visible_name,
        entry.description(),
        entry.input_schema().clone(),
    ) {
        Ok(definition) => definition,
        Err(error) => panic!("forged proof definition must validate: {error}"),
    };
    let route = match ToolRouteRef::parse(route_ref) {
        Ok(route) => route,
        Err(error) => panic!("forged route must parse: {error}"),
    };
    let view = match AgentToolsetView::new(
        ProtocolRunId::parse(projection.run_id().as_str()).expect("run id must map"),
        ProtocolTurnId::parse(projection.turn_id().as_str()).expect("turn id must map"),
        ProjectionSnapshotId::parse(projection.snapshot_id()).expect("snapshot id must map"),
        vec![AgentTool::new(definition, route)],
    ) {
        Ok(view) => view,
        Err(error) => panic!("forged view must validate structurally: {error}"),
    };
    let call_id = match ProtocolProviderToolCallId::parse(provider_call_id) {
        Ok(call_id) => call_id,
        Err(error) => panic!("provider call id must parse: {error}"),
    };
    match view.bind_call(call_id, model_visible_name, canonical_query("calendar")) {
        Ok(call) => call,
        Err(error) => panic!("forged call must bind structurally: {error}"),
    }
}

#[test]
fn provider_view_to_gateway_to_executor_is_correlated_and_authorized() {
    let (projection, current, view) = resolved_view();
    assert_eq!(view.schema_version(), "agent-tool-protocol/v0");
    assert_eq!(view.run_id().as_str(), projection.run_id().as_str());
    assert_eq!(view.turn_id().as_str(), projection.turn_id().as_str());
    assert_eq!(
        view.projection_snapshot_id().as_str(),
        projection.snapshot_id()
    );
    assert_eq!(
        view.tool_definition_set_digest(),
        projection.tool_schema_set_digest()
    );
    let definitions = view.definitions().collect::<Vec<_>>();
    let [definition] = definitions.as_slice() else {
        panic!("proof view must contain exactly one provider definition")
    };
    assert_eq!(definition.model_visible_name(), "proof_tool");
    assert_eq!(definition.description(), "Synthetic proof tool");
    assert_eq!(
        definition.input_schema().digest(),
        projection.entries()[0].input_schema().digest()
    );
    assert_eq!(
        definition.provider_definition_digest(),
        projection.entries()[0].provider_tool_definition_digest()
    );

    let call = bind_proof_call(&view);
    let journal_proposal = ToolCallProposal::from(&call);
    assert_eq!(journal_proposal.call_id, "provider-call:proof");
    assert_eq!(journal_proposal.tool_name, "proof_tool");
    assert_eq!(
        journal_proposal.arguments_digest,
        call.arguments().digest().as_str()
    );

    let mut gateway = FakeToolGateway::new(projection, current);
    let result = match gateway.execute(&call) {
        Ok(result) => result,
        Err(error) => panic!("authorized call must execute: {error:?}"),
    };
    assert_eq!(result.schema_version(), "agent-tool-protocol/v0");
    assert_eq!(call.schema_version(), result.schema_version());
    assert_eq!(result.run_id(), call.run_id());
    assert_eq!(result.turn_id(), call.turn_id());
    assert_eq!(
        result.projection_snapshot_id(),
        call.projection_snapshot_id()
    );
    assert_eq!(result.provider_tool_call_id(), call.provider_tool_call_id());
    let [observed] = gateway.executor.observed.as_slice() else {
        panic!("executor must receive exactly one sealed invocation")
    };
    assert_eq!(observed.installation_id, "installation:proof");
    assert_eq!(observed.package_id, "synthetic.proof");
    assert_eq!(observed.component_id, "component:proof");
    assert_eq!(observed.execution_identity, "native:proof");
    assert_eq!(observed.argument_digest, call.arguments().digest().as_str());
    let AgentToolOutcome::Succeeded { output_digest } = result.outcome() else {
        panic!("fake executor must return success")
    };
    assert_eq!(
        output_digest,
        &Sha256Digest::from_bytes(call.arguments().canonical_bytes())
    );
}

#[test]
fn invalid_or_stale_calls_never_execute() {
    let (projection, mut current, view) = resolved_view();
    let unknown_call_id = match ProtocolProviderToolCallId::parse("provider-call:unknown") {
        Ok(call_id) => call_id,
        Err(error) => panic!("provider call id must parse: {error}"),
    };
    assert!(
        view.bind_call(unknown_call_id, "missing_tool", canonical_query("x"))
            .is_err()
    );
    let unknown_call = bind_forged_call(
        &projection,
        "missing_tool",
        projection.entries()[0].dispatch_key(),
        "provider-call:unknown-forged",
    );
    let mut unknown_gateway = FakeToolGateway::new(projection.clone(), current.clone());
    assert_eq!(
        unknown_gateway.execute(&unknown_call),
        Err(FakeGatewayError::Authorization(
            InvocationAuthorizationError::ToolNotProjected
        ))
    );
    assert!(unknown_gateway.executor.observed.is_empty());

    let malformed_arguments =
        match CanonicalArgumentValueV0::try_from(UnvalidatedArgumentValueV0::Object(Vec::new())) {
            Ok(arguments) => arguments,
            Err(error) => {
                panic!("empty object must canonicalize before schema validation: {error}")
            }
        };
    let malformed_call_id = match ProtocolProviderToolCallId::parse("provider-call:malformed") {
        Ok(call_id) => call_id,
        Err(error) => panic!("provider call id must parse: {error}"),
    };
    let malformed_call = match view.bind_call(malformed_call_id, "proof_tool", malformed_arguments)
    {
        Ok(call) => call,
        Err(error) => panic!("malformed schema call must bind before gateway validation: {error}"),
    };
    let mut malformed_gateway = FakeToolGateway::new(projection.clone(), current.clone());
    assert_eq!(
        malformed_gateway.execute(&malformed_call),
        Err(FakeGatewayError::Authorization(
            InvocationAuthorizationError::ArgumentsInvalid
        ))
    );
    assert!(malformed_gateway.executor.observed.is_empty());

    let route_mismatch_call = bind_forged_call(
        &projection,
        projection.entries()[0].model_visible_name(),
        "dispatch:mismatched",
        "provider-call:route-mismatch",
    );
    let mut route_mismatch_gateway = FakeToolGateway::new(projection.clone(), current.clone());
    assert_eq!(
        route_mismatch_gateway.execute(&route_mismatch_call),
        Err(FakeGatewayError::Authorization(
            InvocationAuthorizationError::DispatchIdentityMismatch
        ))
    );
    assert!(route_mismatch_gateway.executor.observed.is_empty());

    let call = bind_proof_call(&view);
    current.policy.emergency_blocked = true;
    let mut denied_gateway = FakeToolGateway::new(projection.clone(), current);
    assert_eq!(
        denied_gateway.execute(&call),
        Err(FakeGatewayError::Authorization(
            InvocationAuthorizationError::EmergencyBlocked
        ))
    );
    assert!(denied_gateway.executor.observed.is_empty());

    let (mut request, candidate) = proof_authority();
    request.turn_id = match TurnId::parse("turn:other") {
        Ok(turn_id) => turn_id,
        Err(error) => panic!("alternate turn id must parse: {error}"),
    };
    let other_current = current_deny_state(&candidate);
    let other_projection = match InvocationResolver::resolve_projection(request, vec![candidate]) {
        Ok(projection) => projection,
        Err(error) => panic!("alternate projection must resolve: {error}"),
    };
    let mut other_gateway = FakeToolGateway::new(other_projection, other_current);
    assert_eq!(
        other_gateway.execute(&call),
        Err(FakeGatewayError::ProjectionMismatch)
    );
    assert!(other_gateway.executor.observed.is_empty());
}
