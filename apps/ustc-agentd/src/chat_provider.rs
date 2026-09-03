//! Bounded deterministic and OpenAI-compatible providers for the loopback Chat slice.
//!
//! Provider origin, model and credential are process configuration. Browser requests cannot
//! override them. Raw credentials and upstream bodies are never retained in public errors.

use std::fs::OpenOptions;
use std::io::Read;
use std::net::IpAddr;
use std::path::{Path, PathBuf};
use std::time::Duration;

use reqwest::redirect::Policy;
use reqwest::{Client, StatusCode, Url};
use serde::{Deserialize, Serialize};
use serde_json::Value;

const DEFAULT_TIMEOUT_MS: u64 = 15_000;
const MIN_TIMEOUT_MS: u64 = 100;
const MAX_TIMEOUT_MS: u64 = 120_000;
const MAX_BASE_URL_BYTES: usize = 2_048;
const MAX_MODEL_BYTES: usize = 256;
const MAX_KEY_FILE_PATH_BYTES: usize = 4_096;
const MAX_API_KEY_BYTES: usize = 8_192;
const MAX_RESPONSE_BYTES: usize = 256 * 1024;
const MAX_PROVIDER_MESSAGES: usize = 24;
const MAX_PROVIDER_TOOLS: usize = 3;
const MAX_PROVIDER_TEXT_BYTES: usize = 64 * 1024;
const MAX_TOOL_CALL_ID_BYTES: usize = 256;
const MAX_TOOL_NAME_BYTES: usize = 128;
const MAX_TOOL_ARGUMENT_BYTES: usize = 4 * 1024;
const MOCK_MODEL: &str = "deterministic-mock-v1";
const AFFAIRS_TOOL: &str = "affairs_navigator_get";
const CHANGE_TOOL: &str = "change_radar_get";
const OPPORTUNITY_TOOL: &str = "opportunity_graph_plan_current_profile";

#[derive(Clone)]
pub(crate) enum ChatProvider {
    DeterministicMock,
    OpenAiCompatible(OpenAiCompatibleProvider),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct ProviderIdentity {
    pub(crate) profile: String,
    pub(crate) model: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ProviderMessage {
    System {
        content: String,
    },
    User {
        content: String,
    },
    Assistant {
        content: Option<String>,
        tool_calls: Vec<ProviderToolCall>,
    },
    Tool {
        tool_call_id: String,
        content: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ProviderToolDefinition {
    pub(crate) name: String,
    pub(crate) description: String,
    pub(crate) input_schema: Value,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ProviderRequest {
    pub(crate) messages: Vec<ProviderMessage>,
    pub(crate) tools: Vec<ProviderToolDefinition>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ProviderTurn {
    pub(crate) content: Option<String>,
    pub(crate) tool_calls: Vec<ProviderToolCall>,
    pub(crate) usage: ProviderUsage,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ProviderToolCall {
    pub(crate) id: String,
    pub(crate) call_type: String,
    pub(crate) name: String,
    pub(crate) arguments: String,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct ProviderUsage {
    pub(crate) input_tokens: u64,
    pub(crate) output_tokens: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ProviderError {
    Unauthorized,
    RateLimited,
    Timeout,
    Unavailable,
    Protocol,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ProviderConfigError {
    UnknownProfile,
    MissingBaseUrl,
    InvalidBaseUrl,
    InsecureBaseUrl,
    MissingModel,
    InvalidModel,
    MissingKeyFile,
    InvalidKeyFile,
    InvalidTimeout,
    ClientUnavailable,
}

impl ChatProvider {
    pub(crate) fn from_env() -> Result<Self, ProviderConfigError> {
        let profile = std::env::var("UCA_AGENT_PROVIDER").unwrap_or_else(|_| "mock".to_owned());
        match profile.as_str() {
            "mock" => Ok(Self::deterministic_mock()),
            "openai-compatible" => {
                let base_url =
                    required_env("UCA_AGENT_BASE_URL", ProviderConfigError::MissingBaseUrl)?;
                let model = required_env("UCA_AGENT_MODEL", ProviderConfigError::MissingModel)?;
                let key_file = required_env(
                    "UCA_AGENT_API_KEY_FILE",
                    ProviderConfigError::MissingKeyFile,
                )?;
                let timeout_ms = match std::env::var("UCA_AGENT_TIMEOUT_MS") {
                    Ok(raw) => raw
                        .parse::<u64>()
                        .map_err(|_| ProviderConfigError::InvalidTimeout)?,
                    Err(std::env::VarError::NotPresent) => DEFAULT_TIMEOUT_MS,
                    Err(std::env::VarError::NotUnicode(_)) => {
                        return Err(ProviderConfigError::InvalidTimeout);
                    }
                };
                Self::openai_compatible(&base_url, &model, Path::new(&key_file), timeout_ms, false)
            }
            _ => Err(ProviderConfigError::UnknownProfile),
        }
    }

    pub(crate) const fn deterministic_mock() -> Self {
        Self::DeterministicMock
    }

    pub(crate) fn identity(&self) -> ProviderIdentity {
        match self {
            Self::DeterministicMock => ProviderIdentity {
                profile: "mock".to_owned(),
                model: MOCK_MODEL.to_owned(),
            },
            Self::OpenAiCompatible(provider) => provider.identity.clone(),
        }
    }

    pub(crate) async fn complete(
        &self,
        request: &ProviderRequest,
    ) -> Result<ProviderTurn, ProviderError> {
        validate_provider_request(request)?;
        match self {
            Self::DeterministicMock => deterministic_turn(request),
            Self::OpenAiCompatible(provider) => provider.complete(request).await,
        }
    }

    fn openai_compatible(
        base_url: &str,
        model: &str,
        key_file: &Path,
        timeout_ms: u64,
        permit_test_loopback_http: bool,
    ) -> Result<Self, ProviderConfigError> {
        let endpoint = chat_completions_endpoint(base_url, permit_test_loopback_http)?;
        let model = bounded_nonblank(model, MAX_MODEL_BYTES)
            .ok_or(ProviderConfigError::InvalidModel)?
            .to_owned();
        if timeout_ms < MIN_TIMEOUT_MS || timeout_ms > MAX_TIMEOUT_MS {
            return Err(ProviderConfigError::InvalidTimeout);
        }
        let api_key = read_api_key(key_file)?;
        let timeout = Duration::from_millis(timeout_ms);
        let client = Client::builder()
            .redirect(Policy::none())
            .connect_timeout(timeout)
            .timeout(timeout)
            .build()
            .map_err(|_| ProviderConfigError::ClientUnavailable)?;
        Ok(Self::OpenAiCompatible(OpenAiCompatibleProvider {
            client,
            endpoint,
            api_key,
            identity: ProviderIdentity {
                profile: "openai-compatible".to_owned(),
                model,
            },
        }))
    }

    #[cfg(test)]
    fn openai_compatible_for_test(
        base_url: &str,
        model: &str,
        key_file: &Path,
        timeout_ms: u64,
    ) -> Result<Self, ProviderConfigError> {
        Self::openai_compatible(base_url, model, key_file, timeout_ms, true)
    }
}

fn required_env(name: &str, missing: ProviderConfigError) -> Result<String, ProviderConfigError> {
    match std::env::var(name) {
        Ok(value) if !value.is_empty() => Ok(value),
        Ok(_) | Err(std::env::VarError::NotPresent) => Err(missing),
        Err(std::env::VarError::NotUnicode(_)) => Err(missing),
    }
}

#[derive(Clone)]
struct OpenAiCompatibleProvider {
    client: Client,
    endpoint: Url,
    api_key: SecretString,
    identity: ProviderIdentity,
}

#[derive(Clone)]
struct SecretString(String);

impl std::fmt::Debug for SecretString {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("[REDACTED]")
    }
}

impl OpenAiCompatibleProvider {
    async fn complete(&self, request: &ProviderRequest) -> Result<ProviderTurn, ProviderError> {
        let wire_request = build_wire_request(&self.identity.model, request)?;
        let response = self
            .client
            .post(self.endpoint.clone())
            .bearer_auth(&self.api_key.0)
            .json(&wire_request)
            .send()
            .await
            .map_err(map_transport_error)?;

        let status = response.status();
        if status == StatusCode::UNAUTHORIZED || status == StatusCode::FORBIDDEN {
            return Err(ProviderError::Unauthorized);
        }
        if status == StatusCode::TOO_MANY_REQUESTS {
            return Err(ProviderError::RateLimited);
        }
        if !status.is_success() {
            return Err(ProviderError::Unavailable);
        }
        if response
            .content_length()
            .is_some_and(|length| length > MAX_RESPONSE_BYTES as u64)
        {
            return Err(ProviderError::Protocol);
        }

        let mut response = response;
        let mut bytes = Vec::new();
        while let Some(chunk) = response.chunk().await.map_err(map_transport_error)? {
            let next = bytes
                .len()
                .checked_add(chunk.len())
                .ok_or(ProviderError::Protocol)?;
            if next > MAX_RESPONSE_BYTES {
                return Err(ProviderError::Protocol);
            }
            bytes.extend_from_slice(&chunk);
        }
        let wire: OpenAiResponse =
            serde_json::from_slice(&bytes).map_err(|_| ProviderError::Protocol)?;
        parse_wire_response(wire)
    }
}

fn map_transport_error(error: reqwest::Error) -> ProviderError {
    if error.is_timeout() {
        ProviderError::Timeout
    } else {
        ProviderError::Unavailable
    }
}

fn bounded_nonblank(value: &str, maximum_bytes: usize) -> Option<&str> {
    let trimmed = value.trim();
    (!trimmed.is_empty()
        && trimmed.len() <= maximum_bytes
        && !trimmed.chars().any(char::is_control))
    .then_some(trimmed)
}

fn chat_completions_endpoint(
    base_url: &str,
    permit_test_loopback_http: bool,
) -> Result<Url, ProviderConfigError> {
    if base_url.len() > MAX_BASE_URL_BYTES {
        return Err(ProviderConfigError::InvalidBaseUrl);
    }
    let mut url = Url::parse(base_url).map_err(|_| ProviderConfigError::InvalidBaseUrl)?;
    if !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
        || url.host_str().is_none()
    {
        return Err(ProviderConfigError::InvalidBaseUrl);
    }
    let secure = url.scheme() == "https";
    let test_loopback = permit_test_loopback_http
        && url.scheme() == "http"
        && url.host_str().is_some_and(is_loopback_host);
    if !secure && !test_loopback {
        return Err(ProviderConfigError::InsecureBaseUrl);
    }
    let mut path = url.path().trim_end_matches('/').to_owned();
    path.push_str("/chat/completions");
    url.set_path(&path);
    url.set_query(None);
    url.set_fragment(None);
    Ok(url)
}

fn is_loopback_host(host: &str) -> bool {
    host.eq_ignore_ascii_case("localhost")
        || host
            .parse::<IpAddr>()
            .is_ok_and(|address| address.is_loopback())
}

fn read_api_key(path: &Path) -> Result<SecretString, ProviderConfigError> {
    let path_text = path
        .to_str()
        .filter(|value| !value.is_empty() && value.len() <= MAX_KEY_FILE_PATH_BYTES)
        .ok_or(ProviderConfigError::InvalidKeyFile)?;
    let path = PathBuf::from(path_text);
    let metadata =
        std::fs::symlink_metadata(&path).map_err(|_| ProviderConfigError::InvalidKeyFile)?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(ProviderConfigError::InvalidKeyFile);
    }

    let mut options = OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(libc::O_NOFOLLOW);
    }
    let file = options
        .open(&path)
        .map_err(|_| ProviderConfigError::InvalidKeyFile)?;
    let opened = file
        .metadata()
        .map_err(|_| ProviderConfigError::InvalidKeyFile)?;
    if !opened.is_file() || opened.len() > MAX_API_KEY_BYTES as u64 {
        return Err(ProviderConfigError::InvalidKeyFile);
    }
    let mut raw = Vec::new();
    file.take((MAX_API_KEY_BYTES + 1) as u64)
        .read_to_end(&mut raw)
        .map_err(|_| ProviderConfigError::InvalidKeyFile)?;
    if raw.len() > MAX_API_KEY_BYTES {
        return Err(ProviderConfigError::InvalidKeyFile);
    }
    let value = String::from_utf8(raw).map_err(|_| ProviderConfigError::InvalidKeyFile)?;
    let value = value.trim();
    if value.is_empty() || value.len() > MAX_API_KEY_BYTES || value.chars().any(char::is_control) {
        return Err(ProviderConfigError::InvalidKeyFile);
    }
    Ok(SecretString(value.to_owned()))
}

fn validate_provider_request(request: &ProviderRequest) -> Result<(), ProviderError> {
    if request.messages.is_empty()
        || request.messages.len() > MAX_PROVIDER_MESSAGES
        || request.tools.len() > MAX_PROVIDER_TOOLS
    {
        return Err(ProviderError::Protocol);
    }
    for message in &request.messages {
        match message {
            ProviderMessage::System { content } | ProviderMessage::User { content } => {
                validate_provider_text(content)?;
            }
            ProviderMessage::Assistant {
                content,
                tool_calls,
            } => {
                if let Some(content) = content {
                    validate_provider_text(content)?;
                }
                for call in tool_calls {
                    validate_tool_call(call)?;
                }
            }
            ProviderMessage::Tool {
                tool_call_id,
                content,
            } => {
                validate_identifier(tool_call_id, MAX_TOOL_CALL_ID_BYTES)?;
                validate_provider_text(content)?;
            }
        }
    }
    for tool in &request.tools {
        validate_identifier(&tool.name, MAX_TOOL_NAME_BYTES)?;
        validate_provider_text(&tool.description)?;
        if !tool.input_schema.is_object() {
            return Err(ProviderError::Protocol);
        }
    }
    Ok(())
}

fn validate_provider_text(value: &str) -> Result<(), ProviderError> {
    if value.is_empty() || value.len() > MAX_PROVIDER_TEXT_BYTES || value.contains('\0') {
        return Err(ProviderError::Protocol);
    }
    Ok(())
}

fn validate_identifier(value: &str, maximum_bytes: usize) -> Result<(), ProviderError> {
    if value.trim().is_empty() || value.len() > maximum_bytes || value.contains('\0') {
        return Err(ProviderError::Protocol);
    }
    Ok(())
}

fn validate_tool_call(call: &ProviderToolCall) -> Result<(), ProviderError> {
    validate_identifier(&call.id, MAX_TOOL_CALL_ID_BYTES)?;
    validate_identifier(&call.name, MAX_TOOL_NAME_BYTES)?;
    if call.call_type != "function"
        || call.arguments.len() > MAX_TOOL_ARGUMENT_BYTES
        || call.arguments.contains('\0')
    {
        return Err(ProviderError::Protocol);
    }
    Ok(())
}

#[derive(Serialize)]
struct OpenAiRequest<'a> {
    model: &'a str,
    messages: Vec<OpenAiMessage<'a>>,
    tools: Vec<OpenAiTool<'a>>,
    tool_choice: &'static str,
    parallel_tool_calls: bool,
    stream: bool,
}

#[derive(Serialize)]
struct OpenAiMessage<'a> {
    role: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    content: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_calls: Option<Vec<OpenAiOutboundToolCall<'a>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_call_id: Option<&'a str>,
}

#[derive(Serialize)]
struct OpenAiOutboundToolCall<'a> {
    id: &'a str,
    r#type: &'static str,
    function: OpenAiOutboundFunction<'a>,
}

#[derive(Serialize)]
struct OpenAiOutboundFunction<'a> {
    name: &'a str,
    arguments: &'a str,
}

#[derive(Serialize)]
struct OpenAiTool<'a> {
    r#type: &'static str,
    function: OpenAiToolFunction<'a>,
}

#[derive(Serialize)]
struct OpenAiToolFunction<'a> {
    name: &'a str,
    description: &'a str,
    parameters: &'a Value,
}

fn build_wire_request<'a>(
    model: &'a str,
    request: &'a ProviderRequest,
) -> Result<OpenAiRequest<'a>, ProviderError> {
    validate_provider_request(request)?;
    let messages = request
        .messages
        .iter()
        .map(|message| match message {
            ProviderMessage::System { content } => OpenAiMessage {
                role: "system",
                content: Some(content),
                tool_calls: None,
                tool_call_id: None,
            },
            ProviderMessage::User { content } => OpenAiMessage {
                role: "user",
                content: Some(content),
                tool_calls: None,
                tool_call_id: None,
            },
            ProviderMessage::Assistant {
                content,
                tool_calls,
            } => OpenAiMessage {
                role: "assistant",
                content: content.as_deref(),
                tool_calls: (!tool_calls.is_empty()).then(|| {
                    tool_calls
                        .iter()
                        .map(|call| OpenAiOutboundToolCall {
                            id: &call.id,
                            r#type: "function",
                            function: OpenAiOutboundFunction {
                                name: &call.name,
                                arguments: &call.arguments,
                            },
                        })
                        .collect()
                }),
                tool_call_id: None,
            },
            ProviderMessage::Tool {
                tool_call_id,
                content,
            } => OpenAiMessage {
                role: "tool",
                content: Some(content),
                tool_calls: None,
                tool_call_id: Some(tool_call_id),
            },
        })
        .collect();
    let tools = request
        .tools
        .iter()
        .map(|tool| OpenAiTool {
            r#type: "function",
            function: OpenAiToolFunction {
                name: &tool.name,
                description: &tool.description,
                parameters: &tool.input_schema,
            },
        })
        .collect();
    Ok(OpenAiRequest {
        model,
        messages,
        tools,
        tool_choice: "auto",
        parallel_tool_calls: false,
        stream: false,
    })
}

#[derive(Deserialize)]
struct OpenAiResponse {
    choices: Vec<OpenAiChoice>,
    #[serde(default)]
    usage: Option<OpenAiUsage>,
}

#[derive(Deserialize)]
struct OpenAiChoice {
    message: OpenAiInboundMessage,
}

#[derive(Deserialize)]
struct OpenAiInboundMessage {
    #[serde(default)]
    content: Option<String>,
    #[serde(default)]
    tool_calls: Vec<OpenAiInboundToolCall>,
}

#[derive(Deserialize)]
struct OpenAiInboundToolCall {
    id: String,
    r#type: String,
    function: OpenAiInboundFunction,
}

#[derive(Deserialize)]
struct OpenAiInboundFunction {
    name: String,
    arguments: String,
}

#[derive(Deserialize, Default)]
struct OpenAiUsage {
    #[serde(default)]
    prompt_tokens: u64,
    #[serde(default)]
    completion_tokens: u64,
}

fn parse_wire_response(mut response: OpenAiResponse) -> Result<ProviderTurn, ProviderError> {
    if response.choices.len() != 1 {
        return Err(ProviderError::Protocol);
    }
    let choice = response.choices.pop().ok_or(ProviderError::Protocol)?;
    if choice.message.tool_calls.is_empty()
        && choice
            .message
            .content
            .as_deref()
            .is_none_or(|content| content.trim().is_empty())
    {
        return Err(ProviderError::Protocol);
    }
    if choice
        .message
        .content
        .as_ref()
        .is_some_and(|content| content.len() > MAX_PROVIDER_TEXT_BYTES || content.contains('\0'))
    {
        return Err(ProviderError::Protocol);
    }
    let mut tool_calls = Vec::with_capacity(choice.message.tool_calls.len());
    for call in choice.message.tool_calls {
        let call = ProviderToolCall {
            id: call.id,
            call_type: call.r#type,
            name: call.function.name,
            arguments: call.function.arguments,
        };
        validate_tool_call(&call)?;
        tool_calls.push(call);
    }
    let usage = response.usage.unwrap_or_default();
    Ok(ProviderTurn {
        content: choice.message.content,
        tool_calls,
        usage: ProviderUsage {
            input_tokens: usage.prompt_tokens,
            output_tokens: usage.completion_tokens,
        },
    })
}

fn deterministic_turn(request: &ProviderRequest) -> Result<ProviderTurn, ProviderError> {
    let tool_messages = request
        .messages
        .iter()
        .filter_map(|message| match message {
            ProviderMessage::Tool { content, .. } => Some(content.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>();
    if !tool_messages.is_empty() {
        return Ok(ProviderTurn {
            content: Some(format!(
                "已按本次允许的校园工具完成查询。工具返回了 {} 份受限结果；请以其中的来源、有效时间、冲突与不确定性字段为准。",
                tool_messages.len()
            )),
            tool_calls: Vec::new(),
            usage: ProviderUsage::default(),
        });
    }

    let user = request
        .messages
        .iter()
        .rev()
        .find_map(|message| match message {
            ProviderMessage::User { content } => Some(content.as_str()),
            _ => None,
        })
        .ok_or(ProviderError::Protocol)?;
    let has_tool = |name: &str| request.tools.iter().any(|tool| tool.name == name);
    let wants_affairs = user.contains("成绩单")
        || user.contains("证明")
        || user.contains("怎么办")
        || user.to_ascii_lowercase().contains("affairs");
    let wants_change = user.contains("校历")
        || user.contains("变化")
        || user.contains("变更")
        || user.to_ascii_lowercase().contains("change");
    let wants_opportunity = user.contains("选课")
        || user.contains("课程")
        || user.contains("规划")
        || user.to_ascii_lowercase().contains("opportunity");

    let mut calls = Vec::new();
    if wants_affairs && has_tool(AFFAIRS_TOOL) {
        calls.push(mock_call(
            calls.len() + 1,
            AFFAIRS_TOOL,
            r#"{"procedure_id":"proc:ustc:undergraduate:transcript-certificate"}"#,
        ));
    }
    if wants_change && has_tool(CHANGE_TOOL) {
        calls.push(mock_call(
            calls.len() + 1,
            CHANGE_TOOL,
            r#"{"board_id":"board:ustc:academic-calendar"}"#,
        ));
    }
    if wants_opportunity && has_tool(OPPORTUNITY_TOOL) {
        calls.push(mock_call(calls.len() + 1, OPPORTUNITY_TOOL, "{}"));
    }
    if !calls.is_empty() {
        return Ok(ProviderTurn {
            content: None,
            tool_calls: calls,
            usage: ProviderUsage::default(),
        });
    }
    let answer = if wants_opportunity {
        "课程规划需要你先在 Opportunity Graph 面板明确同意并创建 synthetic profile，再为这一次请求单独勾选允许；我不会代你创建或读取私人档案。"
    } else {
        "这是离线 deterministic mock 回答。你可以询问成绩单证明、校历变更，或在逐次明确允许后询问课程规划。"
    };
    Ok(ProviderTurn {
        content: Some(answer.to_owned()),
        tool_calls: Vec::new(),
        usage: ProviderUsage::default(),
    })
}

fn mock_call(index: usize, name: &str, arguments: &str) -> ProviderToolCall {
    ProviderToolCall {
        id: format!("mock-call-{index}"),
        call_type: "function".to_owned(),
        name: name.to_owned(),
        arguments: arguments.to_owned(),
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use std::fs;
    use std::io::{Read as _, Write as _};
    use std::net::TcpListener;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::thread;

    use serde_json::json;

    use super::*;

    static COUNTER: AtomicU64 = AtomicU64::new(0);

    fn key_file() -> PathBuf {
        let path = std::env::temp_dir().join(format!(
            "ustc-agent-provider-key-{}-{}",
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::SeqCst)
        ));
        fs::write(&path, b"test-secret-value\n").unwrap();
        path
    }

    fn request(user: &str, tools: &[&str]) -> ProviderRequest {
        ProviderRequest {
            messages: vec![
                ProviderMessage::System {
                    content: "bounded system".to_owned(),
                },
                ProviderMessage::User {
                    content: user.to_owned(),
                },
            ],
            tools: tools
                .iter()
                .map(|name| ProviderToolDefinition {
                    name: (*name).to_owned(),
                    description: "bounded tool".to_owned(),
                    input_schema: json!({"type":"object"}),
                })
                .collect(),
        }
    }

    #[test]
    fn deterministic_mock_is_network_free_and_routes_exact_tools() {
        let provider = ChatProvider::deterministic_mock();
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        for (prompt, expected) in [
            ("成绩单证明怎么办", AFFAIRS_TOOL),
            ("校历最近有什么变化", CHANGE_TOOL),
            ("帮我规划课程", OPPORTUNITY_TOOL),
        ] {
            let turn = runtime
                .block_on(provider.complete(&request(
                    prompt,
                    &[AFFAIRS_TOOL, CHANGE_TOOL, OPPORTUNITY_TOOL],
                )))
                .unwrap();
            assert_eq!(turn.tool_calls.len(), 1);
            assert_eq!(turn.tool_calls[0].name, expected);
            assert_eq!(turn.usage, ProviderUsage::default());
        }
    }

    #[test]
    fn opportunity_mock_refuses_without_registered_tool() {
        let provider = ChatProvider::deterministic_mock();
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let turn = runtime
            .block_on(provider.complete(&request("帮我规划课程", &[AFFAIRS_TOOL, CHANGE_TOOL])))
            .unwrap();
        assert!(turn.tool_calls.is_empty());
        assert!(turn.content.unwrap().contains("明确同意"));
    }

    #[test]
    fn openai_request_is_nonstreaming_ordered_and_disables_parallel_tools() {
        let request = request("成绩单怎么办", &[AFFAIRS_TOOL]);
        let wire = build_wire_request("model-fixed", &request).unwrap();
        let value = serde_json::to_value(wire).unwrap();
        assert_eq!(value["model"], "model-fixed");
        assert_eq!(value["stream"], false);
        assert_eq!(value["parallel_tool_calls"], false);
        assert_eq!(value["tool_choice"], "auto");
        assert_eq!(value["messages"][0]["role"], "system");
        assert_eq!(value["messages"][1]["role"], "user");
        assert_eq!(value["tools"][0]["function"]["name"], AFFAIRS_TOOL);
    }

    #[test]
    fn provider_url_rejects_credentials_query_fragment_and_non_https() {
        for invalid in [
            "http://example.com/v1",
            "https://user@example.com/v1",
            "https://example.com/v1?q=1",
            "https://example.com/v1#fragment",
        ] {
            assert!(chat_completions_endpoint(invalid, false).is_err());
        }
        let accepted = chat_completions_endpoint("https://example.com/v1/", false).unwrap();
        assert_eq!(accepted.as_str(), "https://example.com/v1/chat/completions");
        assert!(chat_completions_endpoint("http://127.0.0.1:8123/v1", true).is_ok());
        assert!(chat_completions_endpoint("http://example.com/v1", true).is_err());
    }

    #[cfg(unix)]
    #[test]
    fn key_file_rejects_symlinks_and_secret_debug_is_redacted() {
        use std::os::unix::fs::symlink;

        let key = key_file();
        let link = key.with_extension("link");
        symlink(&key, &link).unwrap();
        assert!(matches!(
            read_api_key(&link),
            Err(ProviderConfigError::InvalidKeyFile)
        ));
        let secret = read_api_key(&key).unwrap();
        assert_eq!(format!("{secret:?}"), "[REDACTED]");
        assert!(!format!("{secret:?}").contains("test-secret-value"));
        fs::remove_file(link).unwrap();
        fs::remove_file(key).unwrap();
    }

    #[test]
    fn wire_response_maps_text_tools_usage_and_rejects_empty() {
        let text: OpenAiResponse = serde_json::from_value(json!({
            "choices":[{"message":{"content":"answer","tool_calls":[]}}],
            "usage":{"prompt_tokens":7,"completion_tokens":3}
        }))
        .unwrap();
        let turn = parse_wire_response(text).unwrap();
        assert_eq!(turn.content.as_deref(), Some("answer"));
        assert_eq!(turn.usage.input_tokens, 7);
        assert_eq!(turn.usage.output_tokens, 3);

        let tools: OpenAiResponse = serde_json::from_value(json!({
            "choices":[{"message":{"content":null,"tool_calls":[{
                "id":"call-1","type":"function","function":{"name":AFFAIRS_TOOL,"arguments":"{}"}
            }]}}]
        }))
        .unwrap();
        assert_eq!(parse_wire_response(tools).unwrap().tool_calls.len(), 1);

        let empty: OpenAiResponse = serde_json::from_value(json!({
            "choices":[{"message":{"content":" ","tool_calls":[]}}]
        }))
        .unwrap();
        assert_eq!(parse_wire_response(empty), Err(ProviderError::Protocol));
    }

    #[test]
    fn local_openai_peer_receives_bearer_and_returns_one_turn() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let server = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            stream
                .set_read_timeout(Some(Duration::from_secs(5)))
                .unwrap();
            let mut request = Vec::new();
            let mut buffer = [0_u8; 4096];
            loop {
                let read = stream.read(&mut buffer).unwrap();
                if read == 0 {
                    break;
                }
                request.extend_from_slice(&buffer[..read]);
                if request.windows(4).any(|window| window == b"\r\n\r\n") {
                    break;
                }
            }
            let request_text = String::from_utf8_lossy(&request);
            assert!(request_text.contains("POST /v1/chat/completions HTTP/1.1"));
            assert!(request_text.contains("authorization: Bearer test-secret-value"));
            let body = r#"{"choices":[{"message":{"content":"bounded answer","tool_calls":[]}}],"usage":{"prompt_tokens":2,"completion_tokens":1}}"#;
            write!(
                stream,
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body
            )
            .unwrap();
        });

        let key = key_file();
        let provider = ChatProvider::openai_compatible_for_test(
            &format!("http://{address}/v1"),
            "fixed-model",
            &key,
            5_000,
        )
        .unwrap();
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let turn = runtime
            .block_on(provider.complete(&request("hello", &[])))
            .unwrap();
        assert_eq!(turn.content.as_deref(), Some("bounded answer"));
        assert_eq!(turn.usage.input_tokens, 2);
        server.join().unwrap();
        fs::remove_file(key).unwrap();
    }
}
