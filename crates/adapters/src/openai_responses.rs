//! OpenAI-compatible Responses API peer for the bounded M30 chat port.

use reqwest::{Client, Url};
use serde_json::{Value, json};
use std::error::Error;
use std::fmt;
use std::time::Duration;
use ustc_campus_agent_runtime::chat::{
    MAX_CHAT_ANSWER_BYTES, MAX_CHAT_MESSAGE_BYTES, MAX_CHAT_OUTPUT_TOKENS,
    MAX_CHAT_TOOL_ARGUMENT_BYTES, MAX_CHAT_TOOL_OUTPUT_BYTES, ModelInvocationError,
    ModelInvocationFuture, ModelInvocationPort, ModelInvocationRequest, ModelInvocationResponse,
    ModelOutput, ModelResponseStatus, ModelToolCall, USTC_AFFAIRS_LOOKUP_TOOL,
    USTC_COURSE_ADVICE_TOOL,
};

/// Required environment variable containing the fixed provider origin.
pub const MODEL_BASE_URL_ENV: &str = "USTC_AGENT_MODEL_BASE_URL";
/// Required environment variable containing the process-local provider credential.
pub const MODEL_API_KEY_ENV: &str = "USTC_AGENT_MODEL_API_KEY";
/// Required environment variable containing the exact model identifier.
pub const MODEL_ENV: &str = "USTC_AGENT_MODEL";
/// Optional bounded provider timeout environment variable.
pub const MODEL_TIMEOUT_SECS_ENV: &str = "USTC_AGENT_MODEL_TIMEOUT_SECS";
/// Default provider timeout when [`MODEL_TIMEOUT_SECS_ENV`] is absent.
pub const DEFAULT_MODEL_TIMEOUT_SECS: u64 = 30;
/// Maximum buffered successful Responses API body.
pub const MAX_PROVIDER_RESPONSE_BYTES: usize = 1_048_576;

const MAX_BASE_URL_BYTES: usize = 2_048;
const MAX_API_KEY_BYTES: usize = 8_192;
const MAX_MODEL_BYTES: usize = 1_024;
const MAX_METADATA_BYTES: usize = 1_024;

#[derive(Clone)]
struct ApiKey(String);

impl fmt::Debug for ApiKey {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("ApiKey(<redacted>)")
    }
}

/// Validated server-environment configuration for the bounded adapter.
#[derive(Clone)]
pub struct OpenAiResponsesConfig {
    endpoint: Url,
    api_key: ApiKey,
    model: String,
    timeout_secs: u64,
}

impl OpenAiResponsesConfig {
    /// Validate explicit configuration values without reading process environment.
    pub fn new(
        base_url: impl Into<String>,
        api_key: impl Into<String>,
        model: impl Into<String>,
        timeout_secs: Option<u64>,
    ) -> Result<Self, OpenAiResponsesConfigError> {
        let base_url = base_url.into();
        let api_key = api_key.into();
        let model = model.into();
        let timeout_secs = timeout_secs.unwrap_or(DEFAULT_MODEL_TIMEOUT_SECS);

        let endpoint = validate_and_join_base_url(&base_url)?;
        if api_key.is_empty()
            || api_key.len() > MAX_API_KEY_BYTES
            || api_key.chars().any(char::is_control)
        {
            return Err(OpenAiResponsesConfigError::InvalidApiKey);
        }
        if model.is_empty()
            || model.len() > MAX_MODEL_BYTES
            || model.trim() != model
            || model.chars().any(char::is_control)
        {
            return Err(OpenAiResponsesConfigError::InvalidModel);
        }
        if !(1..=120).contains(&timeout_secs) {
            return Err(OpenAiResponsesConfigError::InvalidTimeout);
        }

        Ok(Self {
            endpoint,
            api_key: ApiKey(api_key),
            model,
            timeout_secs,
        })
    }

    /// Load and validate the exact environment names frozen by the MVP taskbook.
    pub fn from_env() -> Result<Self, OpenAiResponsesConfigError> {
        let base_url = std::env::var(MODEL_BASE_URL_ENV)
            .map_err(|_| OpenAiResponsesConfigError::MissingBaseUrl)?;
        let api_key = std::env::var(MODEL_API_KEY_ENV)
            .map_err(|_| OpenAiResponsesConfigError::MissingApiKey)?;
        let model =
            std::env::var(MODEL_ENV).map_err(|_| OpenAiResponsesConfigError::MissingModel)?;
        let timeout_secs = match std::env::var(MODEL_TIMEOUT_SECS_ENV) {
            Ok(value) => Some(
                value
                    .parse::<u64>()
                    .map_err(|_| OpenAiResponsesConfigError::InvalidTimeout)?,
            ),
            Err(std::env::VarError::NotPresent) => None,
            Err(std::env::VarError::NotUnicode(_)) => {
                return Err(OpenAiResponsesConfigError::InvalidTimeout);
            }
        };
        Self::new(base_url, api_key, model, timeout_secs)
    }

    /// Exact configured model identifier.
    #[must_use]
    pub fn model(&self) -> &str {
        &self.model
    }

    /// Validated bounded timeout in seconds.
    #[must_use]
    pub const fn timeout_secs(&self) -> u64 {
        self.timeout_secs
    }
}

impl fmt::Debug for OpenAiResponsesConfig {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("OpenAiResponsesConfig")
            .field("endpoint", &"<redacted>")
            .field("api_key", &self.api_key)
            .field("model", &"<redacted>")
            .field("timeout_secs", &self.timeout_secs)
            .finish()
    }
}

/// Stable, payload-free configuration errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpenAiResponsesConfigError {
    /// [`MODEL_BASE_URL_ENV`] is absent or non-Unicode.
    MissingBaseUrl,
    /// [`MODEL_API_KEY_ENV`] is absent or non-Unicode.
    MissingApiKey,
    /// [`MODEL_ENV`] is absent or non-Unicode.
    MissingModel,
    /// Base URL is not an admitted absolute origin.
    InvalidBaseUrl,
    /// API key is empty, oversized, or contains control characters.
    InvalidApiKey,
    /// Model identity is empty, oversized, padded, or contains control characters.
    InvalidModel,
    /// Timeout is not an integer in `1..=120` seconds.
    InvalidTimeout,
}

impl fmt::Display for OpenAiResponsesConfigError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::MissingBaseUrl => "model base URL configuration is missing",
            Self::MissingApiKey => "model API key configuration is missing",
            Self::MissingModel => "model identifier configuration is missing",
            Self::InvalidBaseUrl => "model base URL configuration is invalid",
            Self::InvalidApiKey => "model API key configuration is invalid",
            Self::InvalidModel => "model identifier configuration is invalid",
            Self::InvalidTimeout => "model timeout configuration is invalid",
        })
    }
}

impl Error for OpenAiResponsesConfigError {}

/// Stable adapter-construction failure with no reqwest, URL, model, or secret payload.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpenAiResponsesAdapterError {
    /// The redirect-refusing bounded HTTP client could not be initialized.
    ClientInitialization,
}

impl fmt::Display for OpenAiResponsesAdapterError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("model provider client initialization failed")
    }
}

impl Error for OpenAiResponsesAdapterError {}

/// Replaceable M50 peer for one fixed OpenAI-compatible Responses API origin and model.
#[derive(Clone)]
pub struct OpenAiResponsesAdapter {
    config: OpenAiResponsesConfig,
    client: Client,
}

impl OpenAiResponsesAdapter {
    /// Construct a redirect-refusing client with the validated bounded timeout.
    pub fn new(config: OpenAiResponsesConfig) -> Result<Self, OpenAiResponsesAdapterError> {
        let client = Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .no_proxy()
            .timeout(Duration::from_secs(config.timeout_secs))
            .build()
            .map_err(|_| OpenAiResponsesAdapterError::ClientInitialization)?;
        Ok(Self { config, client })
    }

    async fn invoke_inner(
        &self,
        request: ModelInvocationRequest,
    ) -> Result<ModelInvocationResponse, ModelInvocationError> {
        let body = serialize_request(&self.config.model, request)?;
        let response = self
            .client
            .post(self.config.endpoint.clone())
            .bearer_auth(&self.config.api_key.0)
            .json(&body)
            .send()
            .await
            .map_err(map_transport_error)?;

        if !response.status().is_success() {
            return Err(ModelInvocationError::Rejected);
        }
        if response
            .content_length()
            .is_some_and(|length| length > MAX_PROVIDER_RESPONSE_BYTES as u64)
        {
            return Err(ModelInvocationError::MalformedResponse);
        }

        let mut response = response;
        let mut bytes = Vec::new();
        while let Some(chunk) = response.chunk().await.map_err(map_transport_error)? {
            let Some(next_len) = bytes.len().checked_add(chunk.len()) else {
                return Err(ModelInvocationError::MalformedResponse);
            };
            if next_len > MAX_PROVIDER_RESPONSE_BYTES {
                return Err(ModelInvocationError::MalformedResponse);
            }
            bytes.extend_from_slice(&chunk);
        }
        parse_response(&bytes)
    }
}

impl fmt::Debug for OpenAiResponsesAdapter {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("OpenAiResponsesAdapter")
            .finish_non_exhaustive()
    }
}

impl ModelInvocationPort for OpenAiResponsesAdapter {
    fn invoke(&self, request: ModelInvocationRequest) -> ModelInvocationFuture<'_> {
        Box::pin(async move { self.invoke_inner(request).await })
    }
}

fn validate_and_join_base_url(base_url: &str) -> Result<Url, OpenAiResponsesConfigError> {
    if base_url.is_empty()
        || base_url.len() > MAX_BASE_URL_BYTES
        || base_url.trim() != base_url
        || base_url.chars().any(char::is_control)
        || authority_contains_userinfo(base_url)
    {
        return Err(OpenAiResponsesConfigError::InvalidBaseUrl);
    }
    let base = Url::parse(base_url).map_err(|_| OpenAiResponsesConfigError::InvalidBaseUrl)?;
    if base.cannot_be_a_base()
        || base.host_str().is_none()
        || !base.username().is_empty()
        || base.password().is_some()
        || base.query().is_some()
        || base.fragment().is_some()
        || base.path() != "/"
    {
        return Err(OpenAiResponsesConfigError::InvalidBaseUrl);
    }

    match base.scheme() {
        "https" => {}
        "http" if is_exact_test_loopback(base.host_str()) => {}
        _ => return Err(OpenAiResponsesConfigError::InvalidBaseUrl),
    }

    base.join("/v1/responses")
        .map_err(|_| OpenAiResponsesConfigError::InvalidBaseUrl)
}

fn authority_contains_userinfo(value: &str) -> bool {
    let Some((_, after_scheme)) = value.split_once("://") else {
        return false;
    };
    after_scheme
        .split(['/', '?', '#'])
        .next()
        .is_some_and(|authority| authority.contains('@'))
}

fn is_exact_test_loopback(host: Option<&str>) -> bool {
    matches!(host, Some("127.0.0.1" | "::1" | "[::1]"))
}

fn serialize_request(
    model: &str,
    request: ModelInvocationRequest,
) -> Result<Value, ModelInvocationError> {
    match request {
        ModelInvocationRequest::Initial {
            developer_instruction,
            user_message,
            tools,
            max_output_tokens,
        } => {
            validate_output_tokens(max_output_tokens)?;
            if developer_instruction.trim().is_empty()
                || developer_instruction.len() > MAX_CHAT_ANSWER_BYTES
                || user_message.trim().is_empty()
                || user_message.len() > MAX_CHAT_MESSAGE_BYTES
                || tools.len() != 2
            {
                return Err(ModelInvocationError::InvalidRequest);
            }

            let mut saw_affairs = false;
            let mut saw_course = false;
            let mut serialized_tools = Vec::with_capacity(tools.len());
            for tool in tools {
                if !tool.strict || tool.description.trim().is_empty() {
                    return Err(ModelInvocationError::InvalidRequest);
                }
                match tool.name.as_str() {
                    USTC_AFFAIRS_LOOKUP_TOOL if !saw_affairs => saw_affairs = true,
                    USTC_COURSE_ADVICE_TOOL if !saw_course => saw_course = true,
                    _ => return Err(ModelInvocationError::InvalidRequest),
                }
                let parameters = serde_json::from_str::<Value>(&tool.parameters_json)
                    .map_err(|_| ModelInvocationError::InvalidRequest)?;
                if !parameters.is_object() {
                    return Err(ModelInvocationError::InvalidRequest);
                }
                serialized_tools.push(json!({
                    "type": "function",
                    "name": tool.name,
                    "description": tool.description,
                    "parameters": parameters,
                    "strict": true,
                }));
            }
            if !saw_affairs || !saw_course {
                return Err(ModelInvocationError::InvalidRequest);
            }

            Ok(json!({
                "model": model,
                "store": false,
                "parallel_tool_calls": false,
                "max_output_tokens": max_output_tokens,
                "input": [
                    {
                        "role": "developer",
                        "content": [{"type": "input_text", "text": developer_instruction}],
                    },
                    {
                        "role": "user",
                        "content": [{"type": "input_text", "text": user_message}],
                    },
                ],
                "tools": serialized_tools,
            }))
        }
        ModelInvocationRequest::ToolContinuation {
            previous_response_id,
            tool_output,
            max_output_tokens,
        } => {
            validate_output_tokens(max_output_tokens)?;
            if !valid_metadata(&previous_response_id)
                || !valid_metadata(&tool_output.call_id)
                || !matches!(
                    tool_output.name.as_str(),
                    USTC_AFFAIRS_LOOKUP_TOOL | USTC_COURSE_ADVICE_TOOL
                )
                || tool_output.output_json.trim().is_empty()
                || tool_output.output_json.len() > MAX_CHAT_TOOL_OUTPUT_BYTES
            {
                return Err(ModelInvocationError::InvalidRequest);
            }
            Ok(json!({
                "model": model,
                "store": false,
                "parallel_tool_calls": false,
                "max_output_tokens": max_output_tokens,
                "previous_response_id": previous_response_id,
                "input": [{
                    "type": "function_call_output",
                    "call_id": tool_output.call_id,
                    "output": tool_output.output_json,
                }],
            }))
        }
    }
}

fn validate_output_tokens(max_output_tokens: u32) -> Result<(), ModelInvocationError> {
    if (1..=MAX_CHAT_OUTPUT_TOKENS).contains(&max_output_tokens) {
        Ok(())
    } else {
        Err(ModelInvocationError::InvalidRequest)
    }
}

fn parse_response(bytes: &[u8]) -> Result<ModelInvocationResponse, ModelInvocationError> {
    let value = serde_json::from_slice::<Value>(bytes)
        .map_err(|_| ModelInvocationError::MalformedResponse)?;
    let object = value
        .as_object()
        .ok_or(ModelInvocationError::MalformedResponse)?;
    if object.get("status").and_then(Value::as_str) != Some("completed") {
        return Err(ModelInvocationError::MalformedResponse);
    }
    let response_id = required_metadata(object.get("id"))?;
    let model = required_metadata(object.get("model"))?;
    let outputs = object
        .get("output")
        .and_then(Value::as_array)
        .ok_or(ModelInvocationError::MalformedResponse)?;
    let [output] = outputs.as_slice() else {
        return Err(ModelInvocationError::MalformedResponse);
    };
    let output = output
        .as_object()
        .ok_or(ModelInvocationError::MalformedResponse)?;

    let normalized = match output.get("type").and_then(Value::as_str) {
        Some("message") => parse_message_output(output)?,
        Some("function_call") => parse_function_call_output(output)?,
        _ => return Err(ModelInvocationError::MalformedResponse),
    };

    Ok(ModelInvocationResponse {
        response_id,
        model,
        status: ModelResponseStatus::Completed,
        outputs: vec![normalized],
    })
}

fn parse_message_output(
    output: &serde_json::Map<String, Value>,
) -> Result<ModelOutput, ModelInvocationError> {
    if output.get("role").and_then(Value::as_str) != Some("assistant")
        || output.get("status").and_then(Value::as_str) != Some("completed")
    {
        return Err(ModelInvocationError::MalformedResponse);
    }
    let content = output
        .get("content")
        .and_then(Value::as_array)
        .ok_or(ModelInvocationError::MalformedResponse)?;
    let [part] = content.as_slice() else {
        return Err(ModelInvocationError::MalformedResponse);
    };
    let part = part
        .as_object()
        .ok_or(ModelInvocationError::MalformedResponse)?;
    if part.get("type").and_then(Value::as_str) != Some("output_text") {
        return Err(ModelInvocationError::MalformedResponse);
    }
    let text = part
        .get("text")
        .and_then(Value::as_str)
        .ok_or(ModelInvocationError::MalformedResponse)?;
    if text.trim().is_empty() || text.len() > MAX_CHAT_ANSWER_BYTES {
        return Err(ModelInvocationError::MalformedResponse);
    }
    Ok(ModelOutput::Text(text.to_owned()))
}

fn parse_function_call_output(
    output: &serde_json::Map<String, Value>,
) -> Result<ModelOutput, ModelInvocationError> {
    if output.get("status").and_then(Value::as_str) != Some("completed") {
        return Err(ModelInvocationError::MalformedResponse);
    }
    let call_id = required_metadata(output.get("call_id"))?;
    let name = required_metadata(output.get("name"))?;
    let arguments_json = output
        .get("arguments")
        .and_then(Value::as_str)
        .ok_or(ModelInvocationError::MalformedResponse)?;
    if arguments_json.is_empty() || arguments_json.len() > MAX_CHAT_TOOL_ARGUMENT_BYTES {
        return Err(ModelInvocationError::MalformedResponse);
    }
    let arguments = serde_json::from_str::<Value>(arguments_json)
        .map_err(|_| ModelInvocationError::MalformedResponse)?;
    if !arguments.is_object() {
        return Err(ModelInvocationError::MalformedResponse);
    }
    Ok(ModelOutput::ToolCall(ModelToolCall {
        call_id,
        name,
        arguments_json: arguments_json.to_owned(),
    }))
}

fn required_metadata(value: Option<&Value>) -> Result<String, ModelInvocationError> {
    let value = value
        .and_then(Value::as_str)
        .ok_or(ModelInvocationError::MalformedResponse)?;
    if valid_metadata(value) {
        Ok(value.to_owned())
    } else {
        Err(ModelInvocationError::MalformedResponse)
    }
}

fn valid_metadata(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_METADATA_BYTES
        && value.trim() == value
        && !value.chars().any(char::is_control)
}

fn map_transport_error(error: reqwest::Error) -> ModelInvocationError {
    if error.is_timeout() {
        ModelInvocationError::Timeout
    } else {
        ModelInvocationError::Unavailable
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::net::{TcpListener, TcpStream};
    use std::thread::{self, JoinHandle};

    struct FakeReply {
        status: &'static str,
        body: String,
    }

    struct CapturedRequest {
        request_line: String,
        headers: String,
        body: Value,
    }

    fn response_with(output: Value) -> String {
        json!({
            "id": "resp-test",
            "object": "response",
            "status": "completed",
            "model": "model-test",
            "output": [output],
        })
        .to_string()
    }

    fn text_output(text: &str) -> Value {
        json!({
            "type": "message",
            "id": "msg-test",
            "status": "completed",
            "role": "assistant",
            "content": [{
                "type": "output_text",
                "text": text,
                "annotations": [],
            }],
        })
    }

    fn function_output() -> Value {
        json!({
            "type": "function_call",
            "id": "fc-test",
            "call_id": "call-test",
            "status": "completed",
            "name": USTC_AFFAIRS_LOOKUP_TOOL,
            "arguments": r#"{"procedure_id":"proc-011"}"#,
        })
    }

    fn initial_request() -> ModelInvocationRequest {
        ModelInvocationRequest::Initial {
            developer_instruction: "Bounded developer instruction".to_owned(),
            user_message: "Where is the procedure?".to_owned(),
            tools: vec![
                ustc_campus_agent_runtime::chat::ModelToolDefinition {
                    name: USTC_AFFAIRS_LOOKUP_TOOL.to_owned(),
                    description: "Lookup".to_owned(),
                    parameters_json: r#"{"type":"object"}"#.to_owned(),
                    strict: true,
                },
                ustc_campus_agent_runtime::chat::ModelToolDefinition {
                    name: USTC_COURSE_ADVICE_TOOL.to_owned(),
                    description: "Advice".to_owned(),
                    parameters_json: r#"{"type":"object"}"#.to_owned(),
                    strict: true,
                },
            ],
            max_output_tokens: MAX_CHAT_OUTPUT_TOKENS,
        }
    }

    fn runtime() -> tokio::runtime::Runtime {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("test runtime")
    }

    fn spawn_fake_server(replies: Vec<FakeReply>) -> (String, JoinHandle<Vec<CapturedRequest>>) {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind fake provider");
        let address = listener.local_addr().expect("fake provider address");
        let handle = thread::spawn(move || {
            let mut captured = Vec::new();
            for reply in replies {
                let (mut stream, _) = listener.accept().expect("accept provider request");
                let request = read_request(&mut stream);
                let wire = format!(
                    "HTTP/1.1 {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    reply.status,
                    reply.body.len(),
                    reply.body
                );
                stream
                    .write_all(wire.as_bytes())
                    .expect("write provider response");
                stream.flush().expect("flush provider response");
                captured.push(request);
            }
            captured
        });
        (format!("http://{address}"), handle)
    }

    fn read_request(stream: &mut TcpStream) -> CapturedRequest {
        let mut bytes = Vec::new();
        let mut buffer = [0_u8; 4_096];
        let header_end = loop {
            let read = stream.read(&mut buffer).expect("read provider request");
            assert!(read > 0, "provider request ended before headers");
            bytes.extend_from_slice(&buffer[..read]);
            assert!(bytes.len() <= 128 * 1_024, "provider request too large");
            if let Some(index) = bytes.windows(4).position(|window| window == b"\r\n\r\n") {
                break index + 4;
            }
        };
        let headers = std::str::from_utf8(&bytes[..header_end])
            .expect("UTF-8 request headers")
            .to_owned();
        let content_length = headers
            .lines()
            .find_map(|line| {
                let (name, value) = line.split_once(':')?;
                name.eq_ignore_ascii_case("content-length")
                    .then(|| value.trim().parse::<usize>().expect("content length"))
            })
            .expect("request content length");
        while bytes.len() < header_end + content_length {
            let read = stream
                .read(&mut buffer)
                .expect("read provider request body");
            assert!(read > 0, "provider request body ended early");
            bytes.extend_from_slice(&buffer[..read]);
        }
        let request_line = headers.lines().next().expect("request line").to_owned();
        let body = serde_json::from_slice(&bytes[header_end..header_end + content_length])
            .expect("JSON request body");
        CapturedRequest {
            request_line,
            headers,
            body,
        }
    }

    fn adapter(base_url: String, secret: &str) -> OpenAiResponsesAdapter {
        let config = OpenAiResponsesConfig::new(base_url, secret, "model-test", Some(5))
            .expect("valid test config");
        OpenAiResponsesAdapter::new(config).expect("test adapter")
    }

    #[test]
    fn direct_answer_uses_bounded_initial_responses_wire() {
        let (base_url, server) = spawn_fake_server(vec![FakeReply {
            status: "200 OK",
            body: response_with(text_output("Direct answer")),
        }]);
        let adapter = adapter(base_url, "synthetic-test-key");

        let response = runtime()
            .block_on(adapter.invoke(initial_request()))
            .expect("direct provider response");
        assert_eq!(
            response.outputs,
            vec![ModelOutput::Text("Direct answer".to_owned())]
        );
        assert_eq!(response.response_id, "resp-test");
        assert_eq!(response.model, "model-test");

        let captured = server.join().expect("fake provider thread");
        let [request] = captured.as_slice() else {
            panic!("one provider request expected");
        };
        assert_eq!(request.request_line, "POST /v1/responses HTTP/1.1");
        assert!(
            request
                .headers
                .to_ascii_lowercase()
                .contains("authorization: bearer synthetic-test-key")
        );
        assert_eq!(request.body["model"], "model-test");
        assert_eq!(request.body["store"], false);
        assert_eq!(request.body["parallel_tool_calls"], false);
        assert_eq!(request.body["max_output_tokens"], MAX_CHAT_OUTPUT_TOKENS);
        assert_eq!(request.body["input"].as_array().map(Vec::len), Some(2));
        assert_eq!(request.body["input"][0]["role"], "developer");
        assert_eq!(request.body["input"][1]["role"], "user");
        assert_eq!(request.body["tools"].as_array().map(Vec::len), Some(2));
        assert_eq!(request.body["tools"][0]["strict"], true);
        assert_eq!(request.body["tools"][1]["strict"], true);
    }

    #[test]
    fn tool_call_and_continuation_preserve_response_and_call_correlation() {
        let (base_url, server) = spawn_fake_server(vec![
            FakeReply {
                status: "200 OK",
                body: response_with(function_output()),
            },
            FakeReply {
                status: "200 OK",
                body: response_with(text_output("Grounded answer")),
            },
        ]);
        let adapter = adapter(base_url, "synthetic-test-key");
        let runtime = runtime();

        let first = runtime
            .block_on(adapter.invoke(initial_request()))
            .expect("function call response");
        let [ModelOutput::ToolCall(call)] = first.outputs.as_slice() else {
            panic!("one function call expected");
        };
        assert_eq!(call.call_id, "call-test");
        assert_eq!(call.name, USTC_AFFAIRS_LOOKUP_TOOL);

        let second = runtime
            .block_on(adapter.invoke(ModelInvocationRequest::ToolContinuation {
                previous_response_id: first.response_id,
                tool_output: ustc_campus_agent_runtime::chat::ModelToolOutput {
                    call_id: call.call_id.clone(),
                    name: call.name.clone(),
                    output_json: r#"{"status":"found"}"#.to_owned(),
                },
                max_output_tokens: MAX_CHAT_OUTPUT_TOKENS,
            }))
            .expect("continuation response");
        assert_eq!(
            second.outputs,
            vec![ModelOutput::Text("Grounded answer".to_owned())]
        );

        let captured = server.join().expect("fake provider thread");
        let [initial, continuation] = captured.as_slice() else {
            panic!("two provider requests expected");
        };
        assert!(initial.body.get("previous_response_id").is_none());
        assert_eq!(continuation.body["previous_response_id"], "resp-test");
        assert_eq!(
            continuation.body["input"][0]["type"],
            "function_call_output"
        );
        assert_eq!(continuation.body["input"][0]["call_id"], "call-test");
        assert_eq!(
            continuation.body["input"][0]["output"],
            r#"{"status":"found"}"#
        );
        assert!(continuation.body.get("tools").is_none());
    }

    #[test]
    fn invalid_origins_and_bounded_configuration_fail_closed() {
        for invalid in [
            "http://example.com",
            "http://localhost:8080",
            "http://127.0.0.2:8080",
            "https://user@example.com",
            "https://example.com/base",
            "https://example.com?query=1",
            "https://example.com#fragment",
            "relative.example.com",
        ] {
            assert!(matches!(
                OpenAiResponsesConfig::new(invalid, "key", "model", None),
                Err(OpenAiResponsesConfigError::InvalidBaseUrl)
            ));
        }
        assert!(OpenAiResponsesConfig::new("https://example.com", "key", "model", None).is_ok());
        assert!(OpenAiResponsesConfig::new("http://127.0.0.1:8080", "key", "model", None).is_ok());
        assert!(OpenAiResponsesConfig::new("http://[::1]:8080", "key", "model", None).is_ok());
        assert!(matches!(
            OpenAiResponsesConfig::new("https://example.com", "key", "model", Some(0)),
            Err(OpenAiResponsesConfigError::InvalidTimeout)
        ));
        assert!(matches!(
            OpenAiResponsesConfig::new("https://example.com", "key", "model", Some(121)),
            Err(OpenAiResponsesConfigError::InvalidTimeout)
        ));
        assert!(matches!(
            OpenAiResponsesConfig::new("https://example.com", "", "model", None),
            Err(OpenAiResponsesConfigError::InvalidApiKey)
        ));
        assert!(matches!(
            OpenAiResponsesConfig::new("https://example.com", "key", " model ", None),
            Err(OpenAiResponsesConfigError::InvalidModel)
        ));
    }

    #[test]
    fn malformed_or_mixed_provider_response_is_rejected() {
        let malformed = json!({
            "id": "resp-test",
            "status": "completed",
            "model": "model-test",
            "output": [text_output("answer"), function_output()],
        })
        .to_string();
        let (base_url, server) = spawn_fake_server(vec![FakeReply {
            status: "200 OK",
            body: malformed,
        }]);
        let adapter = adapter(base_url, "synthetic-test-key");
        let error = runtime()
            .block_on(adapter.invoke(initial_request()))
            .expect_err("mixed output must fail");
        assert_eq!(error, ModelInvocationError::MalformedResponse);
        let captured = server.join().expect("fake provider thread");
        assert_eq!(captured.len(), 1);
    }

    #[test]
    fn secret_url_model_and_provider_body_never_render_in_errors_or_debug() {
        let secret = "synthetic-secret-never-render";
        let provider_body = "provider-body-never-render";
        let (base_url, server) = spawn_fake_server(vec![FakeReply {
            status: "500 Internal Server Error",
            body: provider_body.to_owned(),
        }]);
        let config =
            OpenAiResponsesConfig::new(base_url.clone(), secret, "model-never-render", Some(5))
                .expect("valid config");
        let config_debug = format!("{config:?}");
        let adapter = OpenAiResponsesAdapter::new(config).expect("test adapter");
        let adapter_debug = format!("{adapter:?}");
        let error = runtime()
            .block_on(adapter.invoke(initial_request()))
            .expect_err("HTTP failure");
        let rendered = format!("{error:?} {error}");
        for forbidden in [
            secret,
            base_url.as_str(),
            "model-never-render",
            provider_body,
        ] {
            assert!(!config_debug.contains(forbidden));
            assert!(!adapter_debug.contains(forbidden));
            assert!(!rendered.contains(forbidden));
        }
        let captured = server.join().expect("fake provider thread");
        assert_eq!(captured.len(), 1);
    }
}
