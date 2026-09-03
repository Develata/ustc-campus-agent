//! plan_ref: docs/plan/modules/60-model-provider-integration.md#121-bounded-chat--first-party-plugin-implementation-slice
//! Provider-neutral chat request, response, port, and failure contracts.

use std::error::Error;
use std::fmt;
use std::future::Future;
use std::pin::Pin;

/// One user message admitted to the bounded chat engine.
#[derive(Clone, PartialEq, Eq)]
pub struct ChatRequest {
    /// Caller-supplied UTF-8 message. The engine trims and validates it before provider I/O.
    pub message: String,
}

impl fmt::Debug for ChatRequest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ChatRequest")
            .field("message", &"<redacted>")
            .finish()
    }
}

/// Successful terminal result of one bounded stateless chat request.
#[derive(Clone, PartialEq, Eq)]
pub struct ChatResult {
    /// Non-empty trimmed assistant text.
    pub answer: String,
    /// Exact model identifier reported by the provider adapter.
    pub model: String,
    /// Supported tool names successfully executed during this request, in execution order.
    pub used_tools: Vec<String>,
    /// True only after one supported tool executed successfully and its output reached the provider.
    pub grounded: bool,
}

impl fmt::Debug for ChatResult {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ChatResult")
            .field("answer", &"<redacted>")
            .field("model", &self.model)
            .field("used_tools", &self.used_tools)
            .field("grounded", &self.grounded)
            .finish()
    }
}

/// One strict provider-visible function definition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelToolDefinition {
    /// Exact model-visible name.
    pub name: String,
    /// Provider-visible description.
    pub description: String,
    /// JSON object schema encoded as JSON text.
    pub parameters_json: String,
    /// Whether the provider must enforce strict schema adherence.
    pub strict: bool,
}

/// Complete owned input for one provider invocation.
#[derive(Clone, PartialEq, Eq)]
pub enum ModelInvocationRequest {
    /// First and only user-message request.
    Initial {
        /// One host-owned developer instruction.
        developer_instruction: String,
        /// One trimmed user message.
        user_message: String,
        /// Exact frozen tool definitions.
        tools: Vec<ModelToolDefinition>,
        /// Provider output-token ceiling.
        max_output_tokens: u32,
    },
    /// Single allowed continuation after successful tool execution.
    ToolContinuation {
        /// Exact provider response ID retained for local correlation only.
        prior_response_id: String,
        /// Replayed host-owned developer instruction for a stateless continuation.
        developer_instruction: String,
        /// Replayed trimmed user message for a stateless continuation.
        user_message: String,
        /// Exact validated tool call returned by the first provider response.
        tool_call: ModelToolCall,
        /// Correlated successful tool output.
        tool_output: ModelToolOutput,
        /// Exact frozen tool definitions replayed without provider-side state.
        tools: Vec<ModelToolDefinition>,
        /// Provider output-token ceiling.
        max_output_tokens: u32,
    },
}

impl fmt::Debug for ModelInvocationRequest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Initial {
                tools,
                max_output_tokens,
                ..
            } => formatter
                .debug_struct("ModelInvocationRequest::Initial")
                .field("developer_instruction", &"<redacted>")
                .field("user_message", &"<redacted>")
                .field("tools", tools)
                .field("max_output_tokens", max_output_tokens)
                .finish(),
            Self::ToolContinuation {
                prior_response_id,
                tool_call,
                tool_output,
                tools,
                max_output_tokens,
                ..
            } => formatter
                .debug_struct("ModelInvocationRequest::ToolContinuation")
                .field("prior_response_id", prior_response_id)
                .field("developer_instruction", &"<redacted>")
                .field("user_message", &"<redacted>")
                .field("tool_call", tool_call)
                .field("tool_output", tool_output)
                .field("tools", tools)
                .field("max_output_tokens", max_output_tokens)
                .finish(),
        }
    }
}

/// Provider completion state needed by the bounded engine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelResponseStatus {
    /// Provider proved the response complete.
    Completed,
    /// Any queued, running, incomplete, cancelled, or otherwise nonterminal outcome.
    NonTerminal,
}

/// One normalized provider output item.
#[derive(Clone, PartialEq, Eq)]
pub enum ModelOutput {
    /// Assistant output text.
    Text(String),
    /// One provider-proposed function call.
    ToolCall(ModelToolCall),
}

impl fmt::Debug for ModelOutput {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Text(_) => formatter.write_str("ModelOutput::Text(<redacted>)"),
            Self::ToolCall(call) => formatter
                .debug_tuple("ModelOutput::ToolCall")
                .field(call)
                .finish(),
        }
    }
}

/// Complete normalized provider response used by M30.
#[derive(Clone, PartialEq, Eq)]
pub struct ModelInvocationResponse {
    /// Provider-issued response identity used for one continuation.
    pub response_id: String,
    /// Provider-reported model identity.
    pub model: String,
    /// Terminality asserted by the adapter.
    pub status: ModelResponseStatus,
    /// Normalized output items. The engine applies stricter turn-dependent cardinality rules.
    pub outputs: Vec<ModelOutput>,
}

impl fmt::Debug for ModelInvocationResponse {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ModelInvocationResponse")
            .field("response_id", &self.response_id)
            .field("model", &self.model)
            .field("status", &self.status)
            .field("outputs", &self.outputs)
            .finish()
    }
}

/// One provider-proposed tool call before host validation and execution.
#[derive(Clone, PartialEq, Eq)]
pub struct ModelToolCall {
    /// Provider-issued call identity.
    pub call_id: String,
    /// Exact model-visible function name.
    pub name: String,
    /// Provider-supplied JSON arguments text.
    pub arguments_json: String,
}

impl fmt::Debug for ModelToolCall {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ModelToolCall")
            .field("call_id", &self.call_id)
            .field("name", &self.name)
            .field("arguments_json", &"<redacted>")
            .finish()
    }
}

/// Successful synchronous tool execution returned by the host composition.
#[derive(Clone, PartialEq, Eq)]
pub struct ToolExecutionOutput {
    /// Bounded typed output encoded for the provider as JSON text.
    pub output_json: String,
}

impl fmt::Debug for ToolExecutionOutput {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ToolExecutionOutput")
            .field("output_json", &"<redacted>")
            .finish()
    }
}

/// Correlated successful tool output sent in the sole continuation request.
#[derive(Clone, PartialEq, Eq)]
pub struct ModelToolOutput {
    /// Exact provider-issued call identity.
    pub call_id: String,
    /// Exact supported tool name that executed.
    pub name: String,
    /// Bounded typed output encoded as JSON text.
    pub output_json: String,
}

impl fmt::Debug for ModelToolOutput {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ModelToolOutput")
            .field("call_id", &self.call_id)
            .field("name", &self.name)
            .field("output_json", &"<redacted>")
            .finish()
    }
}

/// Stable provider-side failure classes. Variants intentionally carry no external payload.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelInvocationError {
    /// The owned request could not be represented safely by the peer.
    InvalidRequest,
    /// The configured provider was unavailable or transport failed.
    Unavailable,
    /// The bounded provider deadline elapsed.
    Timeout,
    /// The provider rejected the request or returned non-success HTTP status.
    Rejected,
    /// The provider response violated the admitted protocol shape.
    MalformedResponse,
}

impl fmt::Display for ModelInvocationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::InvalidRequest => "model request is invalid",
            Self::Unavailable => "model provider is unavailable",
            Self::Timeout => "model provider timed out",
            Self::Rejected => "model provider rejected the request",
            Self::MalformedResponse => "model provider returned a malformed response",
        })
    }
}

impl Error for ModelInvocationError {}

/// Boxed, sendable provider future returned by [`ModelInvocationPort`].
pub type ModelInvocationFuture<'a> = Pin<
    Box<dyn Future<Output = Result<ModelInvocationResponse, ModelInvocationError>> + Send + 'a>,
>;

/// M30-owned provider-neutral model invocation port implemented by M50 peers.
pub trait ModelInvocationPort: Send + Sync {
    /// Invoke exactly one complete owned request without retry or fallback.
    fn invoke(&self, request: ModelInvocationRequest) -> ModelInvocationFuture<'_>;
}

/// Stable synchronous tool failure classes. Variants intentionally carry no payload.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChatToolError {
    /// The named tool arguments did not satisfy its exact host-owned schema or bounds.
    MalformedArguments,
    /// Current permission, consent, or authority denied execution.
    Denied,
    /// The owning Plugin/application path was unavailable.
    Unavailable,
    /// The owning Plugin/application path returned an explicit failure.
    Failed,
}

impl fmt::Display for ChatToolError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::MalformedArguments => "tool arguments are malformed",
            Self::Denied => "tool execution was denied",
            Self::Unavailable => "tool execution is unavailable",
            Self::Failed => "tool execution failed",
        })
    }
}

impl Error for ChatToolError {}

/// M30-owned synchronous boundary to the composition-owned supported tool paths.
pub trait ChatToolPort: Send + Sync {
    /// Validate and execute one supported call. The port must return only typed bounded output.
    fn execute(&self, call: &ModelToolCall) -> Result<ToolExecutionOutput, ChatToolError>;
}

/// Stable safe failures from the bounded engine. No variant can retain external content.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChatError {
    /// The trimmed user message was empty or exceeded [`MAX_CHAT_MESSAGE_BYTES`].
    InvalidMessage,
    /// The provider port rejected the locally assembled request.
    InvalidProviderRequest,
    /// Provider transport or service was unavailable.
    ProviderUnavailable,
    /// Provider invocation exceeded its deadline.
    ProviderTimeout,
    /// Provider explicitly rejected the request.
    ProviderRejected,
    /// Provider output or metadata violated the admitted shape.
    MalformedProviderResponse,
    /// A provider response was not terminal.
    NonTerminalProviderResponse,
    /// A provider response mixed outputs or returned an invalid output count.
    InvalidProviderOutcome,
    /// The provider proposed a malformed call envelope.
    MalformedToolCall,
    /// The provider proposed a tool outside the two frozen names.
    UnknownTool,
    /// Tool arguments did not satisfy the host-owned schema.
    MalformedToolArguments,
    /// Current authority or consent denied the tool.
    ToolDenied,
    /// The owning tool path was unavailable or failed.
    ToolFailed,
    /// A successful tool returned empty or oversized output.
    InvalidToolOutput,
    /// The continuation proposed any second tool round.
    SecondToolRound,
}

impl fmt::Display for ChatError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::InvalidMessage => "chat message is invalid",
            Self::InvalidProviderRequest => "chat provider request is invalid",
            Self::ProviderUnavailable => "chat provider is unavailable",
            Self::ProviderTimeout => "chat provider timed out",
            Self::ProviderRejected => "chat provider rejected the request",
            Self::MalformedProviderResponse => "chat provider response is malformed",
            Self::NonTerminalProviderResponse => "chat provider response is not terminal",
            Self::InvalidProviderOutcome => "chat provider outcome is invalid",
            Self::MalformedToolCall => "chat tool call is malformed",
            Self::UnknownTool => "chat tool is unknown",
            Self::MalformedToolArguments => "chat tool arguments are malformed",
            Self::ToolDenied => "chat tool execution was denied",
            Self::ToolFailed => "chat tool execution failed",
            Self::InvalidToolOutput => "chat tool output is invalid",
            Self::SecondToolRound => "chat tool round limit was exceeded",
        })
    }
}

impl Error for ChatError {}
