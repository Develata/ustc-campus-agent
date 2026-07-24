//! Framework-neutral value objects and sealed envelopes for `agent-tool-protocol/v0`.
//!
//! This crate owns provider-facing schemas, canonical arguments, frozen Agent views,
//! correlated calls, and typed results. It owns no package, grant, policy, executor,
//! Agent state-machine, transport, or framework authority.

mod canonical;

pub use canonical::{
    ArgumentConstructionError, CanonicalArgumentNodeV0, CanonicalArgumentValueV0, InvalidValue,
    SchemaConstructionError, Sha256Digest, UnvalidatedArgumentValueV0, UnvalidatedSchemaNodeV0,
    UnvalidatedToolInputSchemaV0, ValidatedSchemaNodeV0, ValidatedToolInputSchemaV0,
    is_valid_tool_name,
};

use std::collections::BTreeSet;
use std::error::Error;
use std::fmt;

/// Exact logical protocol major used by these in-process value objects.
pub const AGENT_TOOL_PROTOCOL_VERSION: &str = "agent-tool-protocol/v0";
const MAX_TOOLSET_ENTRIES: usize = 256;
const MAX_DESCRIPTION_BYTES: usize = 4_096;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProtocolConstructionError {
    InvalidIdentity(&'static str),
    InvalidToolName,
    DescriptionLimitExceeded,
    ToolsetLimitExceeded,
    DuplicateToolName,
    DuplicateRoute,
    UnknownTool,
}

impl fmt::Display for ProtocolConstructionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "tool protocol construction failed: {self:?}")
    }
}

impl Error for ProtocolConstructionError {}

fn is_valid_protocol_identity(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 256
        && value.chars().all(|c| !c.is_control() && !c.is_whitespace())
}

fn digest_strings<'a>(domain: &[u8], values: impl IntoIterator<Item = &'a str>) -> Sha256Digest {
    let mut bytes = Vec::from(domain);
    for value in values {
        encode_string(value, &mut bytes);
    }
    Sha256Digest::from_bytes(&bytes)
}

fn encode_string(value: &str, output: &mut Vec<u8>) {
    encode_count(value.len(), output);
    output.extend_from_slice(value.as_bytes());
}

fn encode_count(value: usize, output: &mut Vec<u8>) {
    output.extend_from_slice(&(value as u64).to_be_bytes());
}

macro_rules! protocol_identity {
    ($name:ident, $field:literal) => {
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(String);

        impl $name {
            pub fn parse(value: impl Into<String>) -> Result<Self, ProtocolConstructionError> {
                let value = value.into();
                if is_valid_protocol_identity(&value) {
                    Ok(Self(value))
                } else {
                    Err(ProtocolConstructionError::InvalidIdentity($field))
                }
            }

            #[must_use]
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }
    };
}

protocol_identity!(ProtocolRunId, "run_id");
protocol_identity!(ProtocolTurnId, "turn_id");
protocol_identity!(ProjectionSnapshotId, "projection_snapshot_id");
protocol_identity!(ToolRouteRef, "tool_route_ref");
protocol_identity!(ProviderToolCallId, "provider_tool_call_id");
protocol_identity!(StableToolCode, "stable_tool_code");

/// Provider-visible definition. Route and Plugin authority are deliberately absent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentToolDefinition {
    model_visible_name: String,
    description: String,
    input_schema: ValidatedToolInputSchemaV0,
    provider_definition_digest: Sha256Digest,
}

impl AgentToolDefinition {
    pub fn new(
        model_visible_name: impl Into<String>,
        description: impl Into<String>,
        input_schema: ValidatedToolInputSchemaV0,
    ) -> Result<Self, ProtocolConstructionError> {
        let model_visible_name = model_visible_name.into();
        if !is_valid_tool_name(&model_visible_name) {
            return Err(ProtocolConstructionError::InvalidToolName);
        }
        let description = description.into();
        if description.len() > MAX_DESCRIPTION_BYTES {
            return Err(ProtocolConstructionError::DescriptionLimitExceeded);
        }
        let provider_definition_digest = digest_strings(
            b"provider-tool-definition/v0\0",
            [
                model_visible_name.as_str(),
                description.as_str(),
                input_schema.digest().as_str(),
            ],
        );
        Ok(Self {
            model_visible_name,
            description,
            input_schema,
            provider_definition_digest,
        })
    }

    #[must_use]
    pub fn model_visible_name(&self) -> &str {
        &self.model_visible_name
    }

    #[must_use]
    pub fn description(&self) -> &str {
        &self.description
    }

    #[must_use]
    pub const fn input_schema(&self) -> &ValidatedToolInputSchemaV0 {
        &self.input_schema
    }

    #[must_use]
    pub const fn provider_definition_digest(&self) -> &Sha256Digest {
        &self.provider_definition_digest
    }
}

/// Composition-owned pairing of one provider definition with an opaque frozen route.
#[derive(Clone, PartialEq, Eq)]
pub struct AgentTool {
    definition: AgentToolDefinition,
    route_ref: ToolRouteRef,
}

impl AgentTool {
    #[must_use]
    pub const fn new(definition: AgentToolDefinition, route_ref: ToolRouteRef) -> Self {
        Self {
            definition,
            route_ref,
        }
    }
}

/// Immutable per-turn view. Provider adapters enumerate definitions but cannot enumerate routes.
#[derive(Clone, PartialEq, Eq)]
pub struct AgentToolsetView {
    run_id: ProtocolRunId,
    turn_id: ProtocolTurnId,
    projection_snapshot_id: ProjectionSnapshotId,
    tool_definition_set_digest: Sha256Digest,
    tools: Vec<AgentTool>,
}

impl AgentToolsetView {
    pub fn new(
        run_id: ProtocolRunId,
        turn_id: ProtocolTurnId,
        projection_snapshot_id: ProjectionSnapshotId,
        mut tools: Vec<AgentTool>,
    ) -> Result<Self, ProtocolConstructionError> {
        if tools.len() > MAX_TOOLSET_ENTRIES {
            return Err(ProtocolConstructionError::ToolsetLimitExceeded);
        }
        tools.sort_by(|left, right| {
            left.definition
                .model_visible_name
                .cmp(&right.definition.model_visible_name)
        });
        let mut names = BTreeSet::new();
        let mut routes = BTreeSet::new();
        for tool in &tools {
            if !names.insert(tool.definition.model_visible_name.as_str()) {
                return Err(ProtocolConstructionError::DuplicateToolName);
            }
            if !routes.insert(&tool.route_ref) {
                return Err(ProtocolConstructionError::DuplicateRoute);
            }
        }
        let tool_definition_set_digest = digest_strings(
            b"tool-projection/v0\0",
            tools.iter().flat_map(|tool| {
                [
                    tool.route_ref.as_str(),
                    tool.definition.provider_definition_digest().as_str(),
                ]
            }),
        );
        Ok(Self {
            run_id,
            turn_id,
            projection_snapshot_id,
            tool_definition_set_digest,
            tools,
        })
    }

    #[must_use]
    pub const fn schema_version(&self) -> &'static str {
        AGENT_TOOL_PROTOCOL_VERSION
    }

    #[must_use]
    pub const fn run_id(&self) -> &ProtocolRunId {
        &self.run_id
    }

    #[must_use]
    pub const fn turn_id(&self) -> &ProtocolTurnId {
        &self.turn_id
    }

    #[must_use]
    pub const fn projection_snapshot_id(&self) -> &ProjectionSnapshotId {
        &self.projection_snapshot_id
    }

    #[must_use]
    pub const fn tool_definition_set_digest(&self) -> &Sha256Digest {
        &self.tool_definition_set_digest
    }

    pub fn definitions(&self) -> impl ExactSizeIterator<Item = &AgentToolDefinition> {
        self.tools.iter().map(|tool| &tool.definition)
    }

    /// Bind a provider-selected name to the route frozen in this exact view.
    pub fn bind_call(
        &self,
        provider_tool_call_id: ProviderToolCallId,
        model_visible_name: &str,
        arguments: CanonicalArgumentValueV0,
    ) -> Result<AgentToolCall, ProtocolConstructionError> {
        let Some(tool) = self
            .tools
            .iter()
            .find(|tool| tool.definition.model_visible_name == model_visible_name)
        else {
            return Err(ProtocolConstructionError::UnknownTool);
        };
        Ok(AgentToolCall {
            run_id: self.run_id.clone(),
            turn_id: self.turn_id.clone(),
            projection_snapshot_id: self.projection_snapshot_id.clone(),
            provider_tool_call_id,
            model_visible_name: tool.definition.model_visible_name.clone(),
            route_ref: tool.route_ref.clone(),
            arguments,
        })
    }
}

/// Canonical call bound to one exact per-turn projection and opaque route.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentToolCall {
    run_id: ProtocolRunId,
    turn_id: ProtocolTurnId,
    projection_snapshot_id: ProjectionSnapshotId,
    provider_tool_call_id: ProviderToolCallId,
    model_visible_name: String,
    route_ref: ToolRouteRef,
    arguments: CanonicalArgumentValueV0,
}

impl AgentToolCall {
    #[must_use]
    pub const fn schema_version(&self) -> &'static str {
        AGENT_TOOL_PROTOCOL_VERSION
    }

    #[must_use]
    pub const fn run_id(&self) -> &ProtocolRunId {
        &self.run_id
    }

    #[must_use]
    pub const fn turn_id(&self) -> &ProtocolTurnId {
        &self.turn_id
    }

    #[must_use]
    pub const fn projection_snapshot_id(&self) -> &ProjectionSnapshotId {
        &self.projection_snapshot_id
    }

    #[must_use]
    pub const fn provider_tool_call_id(&self) -> &ProviderToolCallId {
        &self.provider_tool_call_id
    }

    #[must_use]
    pub fn model_visible_name(&self) -> &str {
        &self.model_visible_name
    }

    #[must_use]
    pub const fn route_ref(&self) -> &ToolRouteRef {
        &self.route_ref
    }

    #[must_use]
    pub const fn arguments(&self) -> &CanonicalArgumentValueV0 {
        &self.arguments
    }
}

/// Typed terminal result returned to the Agent loop.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AgentToolOutcome {
    Succeeded { output_digest: Sha256Digest },
    Failed { error_code: StableToolCode },
    Denied { reason_code: StableToolCode },
    Cancelled { reason_code: StableToolCode },
    TimedOut { reason_code: StableToolCode },
}

/// Correlated result for one exact call and frozen projection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentToolResult {
    run_id: ProtocolRunId,
    turn_id: ProtocolTurnId,
    projection_snapshot_id: ProjectionSnapshotId,
    provider_tool_call_id: ProviderToolCallId,
    outcome: AgentToolOutcome,
}

impl AgentToolResult {
    #[must_use]
    pub const fn schema_version(&self) -> &'static str {
        AGENT_TOOL_PROTOCOL_VERSION
    }

    #[must_use]
    pub fn from_call(call: &AgentToolCall, outcome: AgentToolOutcome) -> Self {
        Self {
            run_id: call.run_id.clone(),
            turn_id: call.turn_id.clone(),
            projection_snapshot_id: call.projection_snapshot_id.clone(),
            provider_tool_call_id: call.provider_tool_call_id.clone(),
            outcome,
        }
    }

    #[must_use]
    pub const fn run_id(&self) -> &ProtocolRunId {
        &self.run_id
    }

    #[must_use]
    pub const fn turn_id(&self) -> &ProtocolTurnId {
        &self.turn_id
    }

    #[must_use]
    pub const fn projection_snapshot_id(&self) -> &ProjectionSnapshotId {
        &self.projection_snapshot_id
    }

    #[must_use]
    pub const fn provider_tool_call_id(&self) -> &ProviderToolCallId {
        &self.provider_tool_call_id
    }

    #[must_use]
    pub const fn outcome(&self) -> &AgentToolOutcome {
        &self.outcome
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn schema() -> ValidatedToolInputSchemaV0 {
        ValidatedToolInputSchemaV0::try_from(UnvalidatedToolInputSchemaV0 {
            dialect: "tool-input-schema/v0".to_owned(),
            root: UnvalidatedSchemaNodeV0::Object {
                properties: vec![(
                    "query".to_owned(),
                    UnvalidatedSchemaNodeV0::String { enum_values: None },
                )],
                required: vec!["query".to_owned()],
            },
        })
        .expect("schema must validate")
    }

    fn tool(name: &str, route: &str) -> AgentTool {
        AgentTool::new(
            AgentToolDefinition::new(name, "Search reviewed campus facts", schema())
                .expect("definition must validate"),
            ToolRouteRef::parse(route).expect("route must validate"),
        )
    }

    fn view(tools: Vec<AgentTool>) -> Result<AgentToolsetView, ProtocolConstructionError> {
        AgentToolsetView::new(
            ProtocolRunId::parse("run-1")?,
            ProtocolTurnId::parse("turn-1")?,
            ProjectionSnapshotId::parse("projection-1")?,
            tools,
        )
    }

    #[test]
    fn frozen_view_sorts_definitions_and_binds_private_route() {
        let view = view(vec![
            tool("zeta.search", "route-z"),
            tool("alpha.search", "route-a"),
        ])
        .expect("view must validate");
        assert_eq!(
            view.definitions()
                .map(AgentToolDefinition::model_visible_name)
                .collect::<Vec<_>>(),
            vec!["alpha.search", "zeta.search"]
        );
        let arguments =
            CanonicalArgumentValueV0::try_from(UnvalidatedArgumentValueV0::Object(vec![(
                "query".to_owned(),
                UnvalidatedArgumentValueV0::String("calendar".to_owned()),
            )]))
            .expect("arguments must canonicalize");
        let call = view
            .bind_call(
                ProviderToolCallId::parse("provider-call-1").expect("call id"),
                "alpha.search",
                arguments,
            )
            .expect("known tool must bind");
        assert_eq!(call.route_ref().as_str(), "route-a");
    }

    #[test]
    fn duplicate_names_and_routes_fail_closed() {
        assert_eq!(
            view(vec![
                tool("campus.search", "route-a"),
                tool("campus.search", "route-b")
            ])
            .err(),
            Some(ProtocolConstructionError::DuplicateToolName)
        );
        assert_eq!(
            view(vec![
                tool("campus.search", "route-a"),
                tool("campus.open", "route-a")
            ])
            .err(),
            Some(ProtocolConstructionError::DuplicateRoute)
        );
    }

    #[test]
    fn unknown_tool_never_produces_a_call() {
        let view = view(vec![tool("campus.search", "route-a")]).expect("view");
        let arguments =
            CanonicalArgumentValueV0::try_from(UnvalidatedArgumentValueV0::Object(Vec::new()))
                .expect("arguments");
        assert_eq!(
            view.bind_call(
                ProviderToolCallId::parse("provider-call-1").expect("call id"),
                "missing.tool",
                arguments,
            )
            .err(),
            Some(ProtocolConstructionError::UnknownTool)
        );
    }

    #[test]
    fn result_is_correlated_without_plugin_identity() {
        let view = view(vec![tool("campus.search", "route-a")]).expect("view");
        let call = view
            .bind_call(
                ProviderToolCallId::parse("provider-call-1").expect("call id"),
                "campus.search",
                CanonicalArgumentValueV0::try_from(UnvalidatedArgumentValueV0::Object(Vec::new()))
                    .expect("arguments"),
            )
            .expect("call");
        let digest = Sha256Digest::from_bytes(b"output");
        let result = AgentToolResult::from_call(
            &call,
            AgentToolOutcome::Succeeded {
                output_digest: digest.clone(),
            },
        );
        assert_eq!(result.provider_tool_call_id().as_str(), "provider-call-1");
        assert_eq!(
            result.outcome(),
            &AgentToolOutcome::Succeeded {
                output_digest: digest,
            }
        );
    }
}
