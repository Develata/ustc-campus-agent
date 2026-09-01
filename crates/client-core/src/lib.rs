//! M80-owned framework-neutral client core.
//!
//! `client-core` consumes the M10-owned [`ustc-campus-agent-client-protocol`]
//! wire carriers and owns:
//!
//! - checked construction of public/authenticated `affairs.get` submission and
//!   authorized lookup intent ([`public_affairs_get`], [`authenticated_affairs_get`],
//!   [`lookup_by_capability`], [`lookup_as_owner`], [`lookup_as_operator`]);
//! - a real bounded loopback transport over the M10 length-delimited framing
//!   API with connect/read/write timeouts and stable transport failure states
//!   ([`transport`]);
//! - an exhaustive reducer mapping every M10 response/error and every M71
//!   outcome/lineage/freshness/redaction variant to a typed client state and
//!   stable exit class, with no wildcard arm that could hide a future variant
//!   ([`reducer`]);
//! - canonical deterministic JSON rendering of the reduced result.
//!
//! Boundary (taskbook §3.3 and `client-shell/v2.1` §14): this crate depends
//! only on the M10 wire carrier plus `serde`/`serde_json`. It has no
//! dependency on platform-core, affairs-navigator, application-ingress, M60,
//! agentd, a file store, the operator CLI, any domain repository, or any
//! server type — directly or transitively. The client captures typed intent
//! and renders/reduces server-owned state; it owns no calculation,
//! authorization, existence decision, retry truth, mutation, or persistence.
//!
//! ```compile_fail
//! // client-core must not depend on platform-core. This doctest fails to
//! // compile if any such dependency is introduced, proving source isolation.
//! use ustc_campus_agent_core::PRODUCT_NAME;
//! let _ = PRODUCT_NAME;
//! ```

#![forbid(unsafe_code)]
#![cfg_attr(test, allow(clippy::unwrap_used))]

pub mod reducer;
pub mod transport;

pub use reducer::{
    ClientResult, ClientState, ErrorClass, ExitClass, FreshnessClass, LineageClass, OutcomeClass,
    ReasonClass, RedactionClass, TerminalKind, exit_class, reduce_response,
    reduce_transport_failure, render_result,
};
pub use transport::{Endpoint, TransportError, send_intent};
pub use ustc_campus_agent_client_protocol::{
    ActorIntentDto, CapabilityListDto, ClientIntentDto, ClientProtocolMajor, ClientProvenanceDto,
    ClientResponseDto, ProtocolCompatibilityDto, RedactionDto, ServerInfoDto, SubmitAffairsGetDto,
    UnixMillis, ViewerAuthorizationDto, WireText, WireValueError,
};

use serde::{Deserialize, Serialize};

/// Error returned by intent constructors when a checked wire value is rejected.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IntentError(pub WireValueError);

impl std::fmt::Display for IntentError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(formatter)
    }
}

impl std::error::Error for IntentError {}

fn text(value: impl Into<String>) -> Result<WireText, IntentError> {
    WireText::parse(value).map_err(IntentError)
}

/// Builds a checked client provenance envelope from build/target/protocol
/// identity strings.
///
/// # Errors
/// Returns [`IntentError`] if any field fails [`WireText::parse`].
pub fn provenance(
    build: impl Into<String>,
    target: impl Into<String>,
    protocol: impl Into<String>,
) -> Result<ClientProvenanceDto, IntentError> {
    Ok(ClientProvenanceDto {
        build: text(build)?,
        target: text(target)?,
        protocol: text(protocol)?,
    })
}

/// Constructs a public `affairs.get` submission intent.
///
/// The caller supplies every wire identity; `client-core` performs no
/// calculation and invents no digest. `payload_digest` is a caller-provided
/// identity propagated for server-side idempotency verification, not a value
/// this crate authoritatively computes.
///
/// # Errors
/// Returns [`IntentError`] if any field fails [`WireText::parse`].
#[allow(clippy::too_many_arguments)]
pub fn public_affairs_get(
    request_id: impl Into<String>,
    correlation_id: impl Into<String>,
    causation_id: Option<impl Into<String>>,
    idempotency_key: Option<impl Into<String>>,
    provenance: ClientProvenanceDto,
    payload_digest: impl Into<String>,
    procedure_id: impl Into<String>,
    as_of: Option<UnixMillis>,
) -> Result<ClientIntentDto, IntentError> {
    let request = SubmitAffairsGetDto {
        request_id: text(request_id)?,
        correlation_id: text(correlation_id)?,
        causation_id: causation_id.map(text).transpose()?,
        idempotency_key: idempotency_key.map(text).transpose()?,
        actor: ActorIntentDto::Public,
        provenance,
        payload_digest: text(payload_digest)?,
        procedure_id: text(procedure_id)?,
        as_of,
    };
    Ok(ClientIntentDto::SubmitAffairsGet { request })
}

/// Constructs an authenticated `affairs.get` submission intent.
///
/// # Errors
/// Returns [`IntentError`] if any field fails [`WireText::parse`].
#[allow(clippy::too_many_arguments)]
pub fn authenticated_affairs_get(
    request_id: impl Into<String>,
    correlation_id: impl Into<String>,
    causation_id: Option<impl Into<String>>,
    idempotency_key: Option<impl Into<String>>,
    provenance: ClientProvenanceDto,
    payload_digest: impl Into<String>,
    procedure_id: impl Into<String>,
    as_of: Option<UnixMillis>,
    session_id: impl Into<String>,
) -> Result<ClientIntentDto, IntentError> {
    let request = SubmitAffairsGetDto {
        request_id: text(request_id)?,
        correlation_id: text(correlation_id)?,
        causation_id: causation_id.map(text).transpose()?,
        idempotency_key: idempotency_key.map(text).transpose()?,
        actor: ActorIntentDto::Authenticated {
            session_id: text(session_id)?,
        },
        provenance,
        payload_digest: text(payload_digest)?,
        procedure_id: text(procedure_id)?,
        as_of,
    };
    Ok(ClientIntentDto::SubmitAffairsGet { request })
}

/// Constructs a lookup intent authorized by a public capability.
///
/// # Errors
/// Returns [`IntentError`] if any field fails [`WireText::parse`].
pub fn lookup_by_capability(
    command_id: impl Into<String>,
    capability: impl Into<String>,
) -> Result<ClientIntentDto, IntentError> {
    Ok(ClientIntentDto::Lookup {
        command_id: text(command_id)?,
        viewer: ViewerAuthorizationDto::PublicCapability {
            capability: text(capability)?,
        },
    })
}

/// Constructs a lookup intent authorized by the authenticated tenant/user owner.
///
/// # Errors
/// Returns [`IntentError`] if any field fails [`WireText::parse`].
pub fn lookup_as_owner(
    command_id: impl Into<String>,
    tenant_id: impl Into<String>,
    user_id: impl Into<String>,
) -> Result<ClientIntentDto, IntentError> {
    Ok(ClientIntentDto::Lookup {
        command_id: text(command_id)?,
        viewer: ViewerAuthorizationDto::AuthenticatedOwner {
            tenant_id: text(tenant_id)?,
            user_id: text(user_id)?,
        },
    })
}

/// Constructs a lookup intent authorized by a checked operator grant.
///
/// # Errors
/// Returns [`IntentError`] if any field fails [`WireText::parse`].
pub fn lookup_as_operator(
    command_id: impl Into<String>,
    grant_id: impl Into<String>,
) -> Result<ClientIntentDto, IntentError> {
    Ok(ClientIntentDto::Lookup {
        command_id: text(command_id)?,
        viewer: ViewerAuthorizationDto::Operator {
            grant_id: text(grant_id)?,
        },
    })
}

/// Schema version stamped on every canonical CLI result envelope.
pub const RESULT_SCHEMA: &str = "ustc-client-result/v1";

/// Default per-call connect/read/write budget used when the caller does not
/// supply an explicit timeout. Bounded so a silent server cannot hang an
/// automation client indefinitely.
pub const DEFAULT_CALL_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(10);

/// Re-export of the M10 wire crate for clients that want a single import root.
pub extern crate ustc_campus_agent_client_protocol as wire;

/// Marker serialized as part of the result envelope so automation can detect
/// transport-originated outcomes without inspecting private state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Origin {
    Server,
    Transport,
}
