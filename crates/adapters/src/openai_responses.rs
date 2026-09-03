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

/// Stable bootstrap failure for the optional loopback provider. The variants
/// deliberately retain neither configuration values nor transport internals.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpenAiResponsesProviderError {
    /// Some required variables were absent or one value was invalid.
    Configuration,
    /// The bounded HTTP client could not be initialized.
    ClientInitialization,
}

impl fmt::Display for OpenAiResponsesProviderError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Configuration => "model provider configuration is invalid",
            Self::ClientInitialization => "model provider client initialization failed",
        })
    }
}

impl Error for OpenAiResponsesProviderError {}

/// Replaceable M50 peer for one fixed OpenAI-compatible Responses API origin and model.
#[derive(Clone)]
pub struct OpenAiResponsesAdapter {
    config: OpenAiResponsesConfig,
    client: Client,
}

impl OpenAiResponsesAdapter {
    /// Build the optional server-environment provider. When all four model
    /// variables are absent, chat is unavailable while the existing three-
    /// Plugin Web demo remains usable. Any partial configuration fails closed.
    pub fn from_env() -> Result<Option<Self>, OpenAiResponsesProviderError> {
        let base_present = std::env::var_os(MODEL_BASE_URL_ENV).is_some();
        let key_present = std::env::var_os(MODEL_API_KEY_ENV).is_some();
        let model_present = std::env::var_os(MODEL_ENV).is_some();
        let timeout_present = std::env::var_os(MODEL_TIMEOUT_SECS_ENV).is_some();
        if !base_present && !key_present && !model_present && !timeout_present {
            return Ok(None);
        }
        if !(base_present && key_present && model_present) {
            return Err(OpenAiResponsesProviderError::Configuration);
        }
        let config = OpenAiResponsesConfig::from_env()
            .map_err(|_| OpenAiResponsesProviderError::Configuration)?;
        Self::new(config)
            .map(Some)
            .map_err(|_| OpenAiResponsesProviderError::ClientInitialization)
    }

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
        let response = parse_response(&bytes)?;
        if response.model != self.config.model {
            return Err(ModelInvocationError::MalformedResponse);
        }
        Ok(response)
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

/// Product-facing name used by the loopback composition. The concrete peer
/// remains replaceable and owns no platform authority.
pub type OpenAiResponsesProvider = OpenAiResponsesAdapter;

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

mod wire;
use wire::{map_transport_error, parse_response, serialize_request};

#[cfg(test)]
mod tests;
