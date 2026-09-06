//! Reviewed, owner-scoped MCP Streamable HTTP adapter (M51-EXEC-001).
//!
//! Configuration and activation are operator/application inputs, never client grants.
//! The caller MUST independently recheck current M20 installation/grant authority
//! and M40 effect ordering immediately before `call_tool`. This adapter owns only
//! endpoint, session, complete inventory and reviewed snapshot state.
mod endpoint;
mod schema;
use schema::{compile_schema, validate_arguments, validate_output};
#[cfg(test)]
mod tests;
mod transport;

use reqwest::Url;
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
    time::Duration,
};
use ustc_campus_agent_core::invocation::{Sha256Digest, ValidatedToolInputSchemaV0};

pub const PROTOCOL_VERSION: &str = "2025-11-25";

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum EndpointPolicy {
    #[default]
    PublicHttps,
    /// Operator-owned local integration only; not selectable through user input.
    LoopbackDevelopment,
}

/// Server-owned immutable binding inputs; intentionally neither serializable nor Debug.
/// Bearer material comes from a separate trusted secret lookup, never platform login.
pub struct BindingConfig {
    pub binding_id: String,
    pub owner_id: String,
    pub installation_id: String,
    pub component_id: String,
    pub endpoint: String,
    pub endpoint_policy: EndpointPolicy,
    pub bearer_token: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Limits {
    pub request_timeout: Duration,
    pub max_response_bytes: usize,
    pub max_pages: usize,
    pub max_tools: usize,
    pub max_schema_bytes: usize,
    pub max_inventory_bytes: usize,
    pub max_sse_events: usize,
}
impl Default for Limits {
    fn default() -> Self {
        Self {
            request_timeout: Duration::from_secs(15),
            max_response_bytes: 1024 * 1024,
            max_pages: 16,
            max_tools: 64,
            max_schema_bytes: 64 * 1024,
            max_inventory_bytes: 256 * 1024,
            max_sse_events: 128,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum McpError {
    InvalidBinding,
    EndpointDenied,
    Authentication,
    Transport,
    Timeout,
    Protocol,
    RemoteError,
    SessionExpired,
    LimitExceeded,
    InvalidSchema,
    InvalidArguments,
    NotActive,
    SnapshotMismatch,
    SchemaDrift,
    UnknownTool,
    UnsupportedContent,
    Retired,
}
impl fmt::Display for McpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "MCP adapter rejected: {self:?}")
    }
}
impl std::error::Error for McpError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BindingState {
    EndpointValidated,
    ToolsDiscovered,
    Active,
    Quarantined,
    Retired,
}

#[derive(Debug, Clone)]
pub struct DiscoveredTool {
    name: String,
    description: String,
    input_schema: Value,
    compiled_schema: ValidatedToolInputSchemaV0,
    output_schema: Option<Value>,
}
impl DiscoveredTool {
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn description(&self) -> &str {
        &self.description
    }
    pub fn input_schema(&self) -> &Value {
        &self.input_schema
    }
    pub fn compiled_schema(&self) -> &ValidatedToolInputSchemaV0 {
        &self.compiled_schema
    }
    pub fn output_schema(&self) -> Option<&Value> {
        self.output_schema.as_ref()
    }
}

#[derive(Debug, Clone)]
pub struct ToolInventory {
    digest: Sha256Digest,
    tools: Vec<DiscoveredTool>,
}
impl ToolInventory {
    pub fn digest(&self) -> &Sha256Digest {
        &self.digest
    }
    pub fn tools(&self) -> &[DiscoveredTool] {
        &self.tools
    }
}

/// Untrusted data, never policy, grant, instruction or a dereferenced resource.
#[derive(Debug, Clone, PartialEq)]
pub struct McpToolResult {
    pub text: Vec<String>,
    pub structured_content: Option<Value>,
    pub is_error: bool,
}

/// One instance owns one exact binding and protocol session. No cross-owner pool.
pub struct McpClient {
    config: BindingConfig,
    endpoint: Url,
    limits: Limits,
    state: BindingState,
    session_id: Option<String>,
    initialized: bool,
    server_identity: Option<Value>,
    next_id: u64,
    inventory: Option<ToolInventory>,
    reviewed_digest: Option<Sha256Digest>,
    reviewable: bool,
}
impl McpClient {
    pub fn new(config: BindingConfig, limits: Limits) -> Result<Self, McpError> {
        for id in [
            &config.binding_id,
            &config.owner_id,
            &config.installation_id,
            &config.component_id,
        ] {
            if id.is_empty()
                || id.len() > 256
                || id.chars().any(|c| c.is_control() || c.is_whitespace())
            {
                return Err(McpError::InvalidBinding);
            }
        }
        let defaults = Limits::default();
        if limits.request_timeout.is_zero()
            || limits.request_timeout > Duration::from_secs(60)
            || limits.max_response_bytes == 0
            || limits.max_response_bytes > defaults.max_response_bytes
            || limits.max_pages == 0
            || limits.max_pages > defaults.max_pages
            || limits.max_tools == 0
            || limits.max_tools > defaults.max_tools
            || limits.max_schema_bytes == 0
            || limits.max_schema_bytes > defaults.max_schema_bytes
            || limits.max_inventory_bytes == 0
            || limits.max_inventory_bytes > defaults.max_inventory_bytes
            || limits.max_sse_events == 0
            || limits.max_sse_events > defaults.max_sse_events
        {
            return Err(McpError::InvalidBinding);
        }
        if config.bearer_token.as_ref().is_some_and(|token| {
            token.is_empty()
                || token.len() > 4096
                || !token.bytes().all(|b| (0x21..=0x7e).contains(&b))
        }) {
            return Err(McpError::InvalidBinding);
        }
        let endpoint = endpoint::validate_url(&config.endpoint, config.endpoint_policy)?;
        Ok(Self {
            config,
            endpoint,
            limits,
            state: BindingState::EndpointValidated,
            session_id: None,
            initialized: false,
            server_identity: None,
            next_id: 0,
            inventory: None,
            reviewed_digest: None,
            reviewable: false,
        })
    }
    pub fn state(&self) -> BindingState {
        self.state
    }
    pub fn binding_id(&self) -> &str {
        &self.config.binding_id
    }
    pub fn owner_id(&self) -> &str {
        &self.config.owner_id
    }
    pub fn installation_id(&self) -> &str {
        &self.config.installation_id
    }
    pub fn component_id(&self) -> &str {
        &self.config.component_id
    }
    pub fn inventory(&self) -> Option<&ToolInventory> {
        self.inventory.as_ref()
    }

    /// Connection testing does initialization and complete discovery only.
    pub async fn discover(&mut self) -> Result<ToolInventory, McpError> {
        if self.state == BindingState::Retired {
            return Err(McpError::Retired);
        }
        let was_initialized = self.initialized;
        self.reviewable = false;
        self.state = BindingState::Quarantined;
        let result = self.discover_inner().await;
        if result.is_err() {
            // A failed initial discovery cannot retain a remotely allocated session.
            // Preserve the original failure and permit an explicit new discovery.
            if !was_initialized {
                let _ = self.close().await;
            }
            self.state = BindingState::Quarantined;
        }
        result
    }

    async fn discover_inner(&mut self) -> Result<ToolInventory, McpError> {
        if !self.initialized {
            self.session_id = None;
            let init = self
                .rpc(
                    "initialize",
                    json!({"protocolVersion":PROTOCOL_VERSION,
                "capabilities":{}, "clientInfo":{"name":"ustc-campus-agent", "version":"0.1.0"}}),
                )
                .await?;
            if init.get("protocolVersion").and_then(Value::as_str) != Some(PROTOCOL_VERSION)
                || !init
                    .get("capabilities")
                    .and_then(|v| v.get("tools"))
                    .is_some_and(Value::is_object)
            {
                return Err(McpError::Protocol);
            }
            let server = init
                .get("serverInfo")
                .filter(|v| v.is_object())
                .ok_or(McpError::Protocol)?;
            for key in ["name", "version"] {
                bounded_string(server.get(key), 256)?;
            }
            self.server_identity = Some(server.clone());
            self.initialized_notification().await?;
            self.initialized = true;
        }
        let mut cursor = None;
        let mut seen_cursors = BTreeSet::new();
        let mut tools = BTreeMap::new();
        let mut raw_tools = BTreeMap::new();
        let mut size = 0usize;
        let mut complete = false;
        for _ in 0..self.limits.max_pages {
            let params = cursor
                .as_ref()
                .map_or_else(|| json!({}), |c| json!({"cursor":c}));
            let page = self.rpc("tools/list", params).await?;
            let entries = page
                .get("tools")
                .and_then(Value::as_array)
                .ok_or(McpError::Protocol)?;
            for raw in entries {
                size += serde_json::to_vec(raw)
                    .map_err(|_| McpError::Protocol)?
                    .len();
                if size > self.limits.max_inventory_bytes || tools.len() >= self.limits.max_tools {
                    return Err(McpError::LimitExceeded);
                }
                let tool = self.parse_tool(raw)?;
                if tools.contains_key(tool.name()) {
                    return Err(McpError::Protocol);
                }
                raw_tools.insert(tool.name.clone(), canonical_json(raw));
                tools.insert(tool.name.clone(), tool);
            }
            match page.get("nextCursor") {
                None => {
                    complete = true;
                    break;
                }
                Some(value) => {
                    let next = bounded_string(Some(value), 1024)?.to_owned();
                    if !seen_cursors.insert(next.clone()) {
                        return Err(McpError::Protocol);
                    }
                    cursor = Some(next);
                }
            }
        }
        if !complete {
            return Err(McpError::LimitExceeded);
        }
        let snapshot = json!({"protocol":PROTOCOL_VERSION,"server":self.server_identity,
            "tools":raw_tools.into_values().collect::<Vec<_>>()});
        let digest = Sha256Digest::from_bytes(
            &serde_json::to_vec(&canonical_json(&snapshot)).map_err(|_| McpError::Protocol)?,
        );
        let drift = self
            .reviewed_digest
            .as_ref()
            .is_some_and(|reviewed| reviewed != &digest);
        let inventory = ToolInventory {
            digest,
            tools: tools.into_values().collect(),
        };
        self.inventory = Some(inventory.clone());
        self.reviewable = true;
        if drift {
            self.state = BindingState::Quarantined;
            return Err(McpError::SchemaDrift);
        }
        self.state = BindingState::ToolsDiscovered;
        Ok(inventory)
    }

    fn parse_tool(&self, raw: &Value) -> Result<DiscoveredTool, McpError> {
        let name = bounded_string(raw.get("name"), 128)?.to_owned();
        if !name
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"_.-".contains(&b))
        {
            return Err(McpError::Protocol);
        }
        let description = match raw.get("description") {
            None => String::new(),
            Some(Value::String(value)) if value.len() <= 4096 => value.clone(),
            _ => return Err(McpError::Protocol),
        };
        let input_schema = raw
            .get("inputSchema")
            .ok_or(McpError::InvalidSchema)?
            .clone();
        if serde_json::to_vec(&input_schema)
            .map_err(|_| McpError::InvalidSchema)?
            .len()
            > self.limits.max_schema_bytes
        {
            return Err(McpError::LimitExceeded);
        }
        let compiled_schema = compile_schema(&input_schema)?;
        let output_schema = raw.get("outputSchema").cloned();
        if let Some(schema) = &output_schema {
            if serde_json::to_vec(schema)
                .map_err(|_| McpError::InvalidSchema)?
                .len()
                > self.limits.max_schema_bytes
            {
                return Err(McpError::LimitExceeded);
            }
            compile_schema(schema)?;
        }
        Ok(DiscoveredTool {
            name,
            description,
            input_schema,
            compiled_schema,
            output_schema,
        })
    }

    /// The supplied digest MUST come from explicit trusted review, not server/UI data.
    pub fn activate_reviewed(&mut self, digest: &Sha256Digest) -> Result<(), McpError> {
        if self.state == BindingState::Retired {
            return Err(McpError::Retired);
        }
        if !self.initialized || !self.reviewable {
            return Err(McpError::NotActive);
        }
        if self.inventory.as_ref().is_none_or(|i| i.digest() != digest) {
            return Err(McpError::SnapshotMismatch);
        }
        self.reviewed_digest = Some(digest.clone());
        self.state = BindingState::Active;
        Ok(())
    }

    pub fn quarantine(&mut self) {
        if self.state != BindingState::Retired {
            self.state = BindingState::Quarantined;
            self.reviewable = false;
        }
    }

    /// Caller owns fresh M20 authorization and effect receipt ordering. No retry,
    /// fallback, remote instruction execution or resource dereference occurs here.
    pub async fn call_tool(
        &mut self,
        digest: &Sha256Digest,
        name: &str,
        arguments: &Value,
    ) -> Result<McpToolResult, McpError> {
        if self.state != BindingState::Active || !self.initialized {
            return Err(McpError::NotActive);
        }
        if self.reviewed_digest.as_ref() != Some(digest) {
            return Err(McpError::SnapshotMismatch);
        }
        let tool = self
            .inventory
            .as_ref()
            .and_then(|i| i.tools.iter().find(|tool| tool.name() == name))
            .ok_or(McpError::UnknownTool)?;
        validate_arguments(&tool.compiled_schema, arguments)?;
        let output_schema = tool.output_schema.clone();
        let result = self
            .rpc("tools/call", json!({"name":name, "arguments":arguments}))
            .await?;
        let parsed = parse_result(result, output_schema.as_ref());
        if parsed.is_err() {
            self.quarantine();
        }
        parsed
    }

    /// Explicit session termination; local retirement is final even if DELETE fails.
    pub async fn close(&mut self) -> Result<(), McpError> {
        self.state = BindingState::Retired;
        self.reviewable = false;
        let result = if self.session_id.is_some() {
            self.delete_session().await
        } else {
            Ok(())
        };
        self.session_id = None;
        self.initialized = false;
        self.state = BindingState::Retired;
        result
    }
}

fn bounded_string(value: Option<&Value>, max: usize) -> Result<&str, McpError> {
    value
        .and_then(Value::as_str)
        .filter(|value| {
            !value.is_empty() && value.len() <= max && !value.chars().any(char::is_control)
        })
        .ok_or(McpError::Protocol)
}

fn canonical_json(value: &Value) -> Value {
    match value {
        Value::Object(object) => {
            let sorted = object
                .iter()
                .map(|(key, value)| (key.clone(), canonical_json(value)))
                .collect::<BTreeMap<_, _>>();
            Value::Object(sorted.into_iter().collect())
        }
        Value::Array(array) => Value::Array(array.iter().map(canonical_json).collect()),
        _ => value.clone(),
    }
}

fn parse_result(result: Value, output_schema: Option<&Value>) -> Result<McpToolResult, McpError> {
    let content = result
        .get("content")
        .and_then(Value::as_array)
        .ok_or(McpError::Protocol)?;
    let mut text = Vec::with_capacity(content.len());
    for item in content {
        if item.get("type").and_then(Value::as_str) != Some("text") {
            return Err(McpError::UnsupportedContent);
        }
        text.push(
            item.get("text")
                .and_then(Value::as_str)
                .ok_or(McpError::Protocol)?
                .to_owned(),
        );
    }
    let is_error = match result.get("isError") {
        None => false,
        Some(Value::Bool(value)) => *value,
        _ => return Err(McpError::Protocol),
    };
    let structured_content = result.get("structuredContent").cloned();
    if structured_content
        .as_ref()
        .is_some_and(|value| !value.is_object())
    {
        return Err(McpError::Protocol);
    }
    if !is_error && let Some(schema) = output_schema {
        let data = structured_content.as_ref().ok_or(McpError::InvalidSchema)?;
        validate_output(schema, data).map_err(|_| McpError::InvalidSchema)?;
    }
    Ok(McpToolResult {
        text,
        structured_content,
        is_error,
    })
}
