//! Bounded, deterministic `source-retrieval/v0` policy evidence.
//!
//! This module performs no I/O, reads no clock, opens no transport, and mints
//! no retrieval authority. It transforms an approved [`RetrievalSubject`] and
//! caller-supplied synthetic observations through a linear pure-policy chain.
//!
//! Phase outputs cannot be constructed by callers:
//!
//! ```compile_fail
//! use ustc_campus_agent_core::source_retrieval::ValidatedFetchCandidate;
//!
//! let candidate = ValidatedFetchCandidate {};
//! ```
//!
//! Linear phase outputs cannot be cloned:
//!
//! ```compile_fail
//! use ustc_campus_agent_core::source_retrieval::ResolvedRetrievalCandidate;
//!
//! fn duplicate(value: ResolvedRetrievalCandidate) -> ResolvedRetrievalCandidate {
//!     value.clone()
//! }
//! ```
//!
//! No effect-authority API is present:
//!
//! ```compile_fail
//! use ustc_campus_agent_core::source_retrieval::BoundedFetch;
//! ```

use std::collections::BTreeSet;
use std::error::Error;
use std::fmt;
use std::net::{Ipv4Addr, SocketAddr};

use crate::identity::CommandId;
use crate::source_registry::{
    PublicIpPolicyVersion, RetrievalSubject, SourceAuthorityRevision, SourceId, SourceMediaType,
    SourceRetrievalProtocolVersion, SourceValueError, SourceValueErrorKind, classify_source_id,
    value_error,
};

const MAX_DNS_NAME_BYTES: usize = 253;
const MAX_DNS_LABEL_BYTES: usize = 63;
const MAX_RAW_CNAME_COUNT: usize = 64;
const MAX_RAW_ADDRESS_COUNT: usize = 64;
const MAX_POLICY_CNAME_DEPTH: usize = 8;
const MAX_POLICY_ADDRESS_COUNT: usize = 16;
const MAX_RESPONSE_HEAD_BYTES: usize = 32_768;
const MAX_HEADER_FIELDS: usize = 128;
const MAX_HEADER_NAME_BYTES: usize = 64;
const MAX_HEADER_VALUE_BYTES: usize = 8_192;
const MAX_CONTENT_PARAMETERS: usize = 16;
const MAX_CONTENT_PARAMETER_BYTES: usize = 64;
const MAX_BODY_OBSERVATION_BYTES: usize = 1_048_577;
const MAX_TRANSPORT_WIRE_BYTES: u64 = 1_114_112;
const MAX_TRANSPORT_ELAPSED_MILLISECONDS: u64 = 60_000;
const WIRE_OVERHEAD_BYTES: u64 = 65_536;
const MAX_CHUNK_COUNT: u32 = 4_096;
const MAX_CHUNK_LINE_BYTES: u16 = 128;
const MAX_CHUNK_SIZE_DIGITS: u16 = 16;

macro_rules! retrieval_id {
    ($(#[$attribute:meta])* $name:ident) => {
        $(#[$attribute])*
        #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name {
            value: String,
        }

        impl $name {
            /// Constructs one checked nominal value over the B1 ID grammar.
            ///
            /// # Errors
            /// Returns a payload-free [`SourceValueError`] when the value is not canonical.
            pub fn new(value: String) -> Result<Self, SourceValueError> {
                classify_source_id(&value)
                    .map(|()| Self { value })
                    .map_err(|kind| value_error(stringify!($name), kind))
            }

            /// Returns the exact canonical text.
            #[must_use]
            pub fn as_str(&self) -> &str {
                &self.value
            }

            /// Consumes the value and returns its exact canonical text.
            #[must_use]
            pub fn into_inner(self) -> String {
                self.value
            }
        }
    };
}

retrieval_id! {
    /// Identity of one retrieval attempt; shape does not imply admission.
    RetrievalAttemptId
}
retrieval_id! {
    /// Identity of one rate-override request; shape does not imply authority.
    RateOverrideId
}
retrieval_id! {
    /// Reference to rate-override evidence retained elsewhere.
    RetrievalOverrideEvidenceId
}
retrieval_id! {
    /// Operator identity carried by synthetic override facts.
    SourceOperatorId
}

/// Caller-supplied epoch seconds used only by deterministic policy methods.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RetrievalEpochSeconds {
    value: u64,
}

impl RetrievalEpochSeconds {
    /// Wraps a complete `u64` epoch domain without reading a clock.
    #[must_use]
    pub const fn from_unix_seconds(value: u64) -> Self {
        Self { value }
    }

    /// Returns the caller-supplied scalar.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.value
    }
}

/// Checked lowercase ASCII DNS name with payload-redacted `Debug`.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RetrievalDnsName {
    value: String,
}

impl RetrievalDnsName {
    /// Parses the exact v0 DNS grammar.
    ///
    /// # Errors
    /// Returns a payload-free [`SourceValueError`] for invalid DNS text.
    pub fn parse(value: &str) -> Result<Self, SourceValueError> {
        if !dns_name_is_canonical(value) {
            return Err(value_error(
                "RetrievalDnsName",
                SourceValueErrorKind::InvalidHost,
            ));
        }
        Ok(Self {
            value: value.to_owned(),
        })
    }

    fn as_str(&self) -> &str {
        &self.value
    }
}

impl fmt::Debug for RetrievalDnsName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("RetrievalDnsName(<redacted>)")
    }
}

fn dns_name_is_canonical(value: &str) -> bool {
    let bytes = value.as_bytes();
    if !(3..=MAX_DNS_NAME_BYTES).contains(&bytes.len()) || !bytes.is_ascii() || value.ends_with('.')
    {
        return false;
    }
    let mut labels = 0usize;
    for label in value.split('.') {
        labels += 1;
        let bytes = label.as_bytes();
        if !(1..=MAX_DNS_LABEL_BYTES).contains(&bytes.len()) {
            return false;
        }
        let Some((&first, rest)) = bytes.split_first() else {
            return false;
        };
        let last = *bytes.last().expect("nonempty label");
        if !dns_edge(first) || !dns_edge(last) {
            return false;
        }
        if rest.iter().any(|byte| !dns_edge(*byte) && *byte != b'-') {
            return false;
        }
    }
    labels >= 2
}

const fn dns_edge(byte: u8) -> bool {
    byte.is_ascii_lowercase() || byte.is_ascii_digit()
}

/// Non-authority request to use one override-evidence reference.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RetrievalRateOverrideRequest {
    override_id: RateOverrideId,
    evidence_id: RetrievalOverrideEvidenceId,
}

impl RetrievalRateOverrideRequest {
    /// Constructs a total value after nominal inputs were checked.
    #[must_use]
    pub fn new(override_id: RateOverrideId, evidence_id: RetrievalOverrideEvidenceId) -> Self {
        Self {
            override_id,
            evidence_id,
        }
    }
}

/// Synthetic, non-authority facts used by pure rate evaluation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RetrievalOverrideFacts {
    evidence_id: RetrievalOverrideEvidenceId,
    override_id: RateOverrideId,
    attempt_id: RetrievalAttemptId,
    operator: SourceOperatorId,
    source_id: SourceId,
    authority_revision: SourceAuthorityRevision,
    issued_at: RetrievalEpochSeconds,
    not_after: RetrievalEpochSeconds,
}

impl RetrievalOverrideFacts {
    /// Constructs facts whose validity window is ordered.
    ///
    /// # Errors
    /// Returns `InvalidOverrideWindow` iff `issued_at > not_after`.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        evidence_id: RetrievalOverrideEvidenceId,
        override_id: RateOverrideId,
        attempt_id: RetrievalAttemptId,
        operator: SourceOperatorId,
        source_id: SourceId,
        authority_revision: SourceAuthorityRevision,
        issued_at: RetrievalEpochSeconds,
        not_after: RetrievalEpochSeconds,
    ) -> Result<Self, SourceValueError> {
        if issued_at > not_after {
            return Err(value_error(
                "RetrievalOverrideFacts",
                SourceValueErrorKind::InvalidOverrideWindow,
            ));
        }
        Ok(Self {
            evidence_id,
            override_id,
            attempt_id,
            operator,
            source_id,
            authority_revision,
            issued_at,
            not_after,
        })
    }
}

/// Complete caller command for pure candidate derivation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RetrievalAttemptCommand {
    command_id: CommandId,
    attempt_id: RetrievalAttemptId,
    source_id: SourceId,
    expected_authority_revision: SourceAuthorityRevision,
    override_request: Option<RetrievalRateOverrideRequest>,
}

impl RetrievalAttemptCommand {
    /// Constructs a total command after nominal inputs were checked.
    #[must_use]
    pub fn new(
        command_id: CommandId,
        attempt_id: RetrievalAttemptId,
        source_id: SourceId,
        expected_authority_revision: SourceAuthorityRevision,
        override_request: Option<RetrievalRateOverrideRequest>,
    ) -> Self {
        Self {
            command_id,
            attempt_id,
            source_id,
            expected_authority_revision,
            override_request,
        }
    }
}

/// Pure rate decision; neither variant consumes evidence or grants authority.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RetrievalRateDecision {
    /// No rate override was required.
    Allowed,
    /// An exact synthetic override was accepted by the pure decision table.
    AllowedWithOverride(RateOverrideId),
}

/// Version classification from the strict status-line parser.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HttpVersionClass {
    Http10,
    Http11,
    Http2,
    Http3,
    Other,
}

/// Response body framing selected by strict response-head authorization.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RetrievalBodyFraming {
    ContentLength(u64),
    Chunked,
    CloseDelimited,
}

/// Exact multiplicity projection for one observed header name.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ObservedHeaderValue {
    Missing,
    One(String),
    Repeated,
}

/// Parsed, non-authority response-head observation.
#[derive(Clone, PartialEq, Eq)]
pub struct ResponseHeadObservation {
    version: HttpVersionClass,
    status_code: u16,
    headers: Vec<(String, String)>,
}

impl fmt::Debug for ResponseHeadObservation {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ResponseHeadObservation")
            .field("version", &self.version)
            .field("status_code", &self.status_code)
            .field("header_count", &self.headers.len())
            .finish()
    }
}

/// Shape-only DNS observation returned by a future transport.
pub struct DnsTransportObservation {
    queried_host: String,
    cname_chain: Vec<String>,
    complete_addresses: Vec<Ipv4Addr>,
}

impl fmt::Debug for DnsTransportObservation {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("DnsTransportObservation")
            .field("queried_host", &"<redacted>")
            .field("cname_count", &self.cname_chain.len())
            .field("address_count", &self.complete_addresses.len())
            .finish()
    }
}

impl DnsTransportObservation {
    /// Constructs only the globally bounded raw representation.
    ///
    /// # Errors
    /// Returns only `ObservationShapeRejected` when a representation bound fails.
    pub fn new(
        queried_host: String,
        cname_chain: Vec<String>,
        complete_addresses: Vec<Ipv4Addr>,
    ) -> Result<Self, SourceTransportError> {
        if !(1..=MAX_DNS_NAME_BYTES).contains(&queried_host.len())
            || cname_chain.len() > MAX_RAW_CNAME_COUNT
            || cname_chain
                .iter()
                .any(|name| !(1..=MAX_DNS_NAME_BYTES).contains(&name.len()))
            || !(1..=MAX_RAW_ADDRESS_COUNT).contains(&complete_addresses.len())
        {
            return Err(SourceTransportError::ObservationShapeRejected);
        }
        Ok(Self {
            queried_host,
            cname_chain,
            complete_addresses,
        })
    }

    #[must_use]
    pub fn queried_host(&self) -> &str {
        &self.queried_host
    }

    #[must_use]
    pub fn cname_chain(&self) -> &[String] {
        &self.cname_chain
    }

    #[must_use]
    pub fn complete_addresses(&self) -> &[Ipv4Addr] {
        &self.complete_addresses
    }

    #[must_use]
    pub fn into_parts(self) -> (String, Vec<String>, Vec<Ipv4Addr>) {
        (self.queried_host, self.cname_chain, self.complete_addresses)
    }
}

/// Shape-only body observation. Policy is applied only by [`RetrievalPolicy::finish_body`].
pub struct BodyObservation {
    bytes: Vec<u8>,
    wire_bytes_after_headers: u64,
    chunk_count: u32,
    max_chunk_line_bytes: u16,
    saw_chunk_extension: bool,
    trailer_field_count: u16,
    framing_complete: bool,
    elapsed_milliseconds: u64,
}

impl fmt::Debug for BodyObservation {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("BodyObservation")
            .field("body_bytes", &self.bytes.len())
            .field("wire_bytes_after_headers", &self.wire_bytes_after_headers)
            .field("chunk_count", &self.chunk_count)
            .field("framing_complete", &self.framing_complete)
            .field("elapsed_milliseconds", &self.elapsed_milliseconds)
            .finish()
    }
}

impl BodyObservation {
    /// Constructs the exact bounded body representation without applying policy.
    ///
    /// # Errors
    /// Returns only `ObservationShapeRejected` for vectors larger than 1,048,577 bytes.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        bytes: Vec<u8>,
        wire_bytes_after_headers: u64,
        chunk_count: u32,
        max_chunk_line_bytes: u16,
        saw_chunk_extension: bool,
        trailer_field_count: u16,
        framing_complete: bool,
        elapsed_milliseconds: u64,
    ) -> Result<Self, SourceTransportError> {
        if bytes.len() > MAX_BODY_OBSERVATION_BYTES {
            return Err(SourceTransportError::ObservationShapeRejected);
        }
        Ok(Self {
            bytes,
            wire_bytes_after_headers,
            chunk_count,
            max_chunk_line_bytes,
            saw_chunk_extension,
            trailer_field_count,
            framing_complete,
            elapsed_milliseconds,
        })
    }
}

/// Exact serialized request bytes; no mutation or arbitrary header surface exists.
#[derive(PartialEq, Eq)]
pub struct SerializedRetrievalRequest {
    bytes: Vec<u8>,
}

impl fmt::Debug for SerializedRetrievalRequest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SerializedRetrievalRequest")
            .field("bytes", &self.bytes.len())
            .finish()
    }
}

impl SerializedRetrievalRequest {
    /// Returns the exact immutable wire bytes.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }
}

/// Shape-only raw transport success. It carries no domain verdict.
pub struct RetrievalTransportSuccess {
    response_head_bytes: Vec<u8>,
    body_bytes: Vec<u8>,
    wire_bytes_after_headers: u64,
    peer_socket_addr: SocketAddr,
    dns_transport_observation: DnsTransportObservation,
    framing: RetrievalBodyFraming,
    elapsed_milliseconds: u64,
}

impl fmt::Debug for RetrievalTransportSuccess {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RetrievalTransportSuccess")
            .field("response_head_bytes", &self.response_head_bytes.len())
            .field("body_bytes", &self.body_bytes.len())
            .field("wire_bytes_after_headers", &self.wire_bytes_after_headers)
            .field(
                "peer_family",
                &if self.peer_socket_addr.is_ipv4() {
                    "ipv4"
                } else {
                    "ipv6"
                },
            )
            .field("framing", &self.framing)
            .field("elapsed_milliseconds", &self.elapsed_milliseconds)
            .finish()
    }
}

/// Consuming projection of a transport success.
pub struct RetrievalTransportSuccessParts {
    pub response_head_bytes: Vec<u8>,
    pub body_bytes: Vec<u8>,
    pub wire_bytes_after_headers: u64,
    pub peer_socket_addr: SocketAddr,
    pub dns_transport_observation: DnsTransportObservation,
    pub framing: RetrievalBodyFraming,
    pub elapsed_milliseconds: u64,
}

impl fmt::Debug for RetrievalTransportSuccessParts {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RetrievalTransportSuccessParts")
            .field("response_head_bytes", &self.response_head_bytes.len())
            .field("body_bytes", &self.body_bytes.len())
            .field("wire_bytes_after_headers", &self.wire_bytes_after_headers)
            .field(
                "peer_family",
                &if self.peer_socket_addr.is_ipv4() {
                    "ipv4"
                } else {
                    "ipv6"
                },
            )
            .field("dns_transport_observation", &self.dns_transport_observation)
            .field("framing", &self.framing)
            .field("elapsed_milliseconds", &self.elapsed_milliseconds)
            .finish()
    }
}

impl RetrievalTransportSuccess {
    /// Constructs only the globally bounded raw success representation.
    ///
    /// # Errors
    /// Returns only `ObservationShapeRejected` when a representation bound fails.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        response_head_bytes: Vec<u8>,
        body_bytes: Vec<u8>,
        wire_bytes_after_headers: u64,
        peer_socket_addr: SocketAddr,
        dns_transport_observation: DnsTransportObservation,
        framing: RetrievalBodyFraming,
        elapsed_milliseconds: u64,
    ) -> Result<Self, SourceTransportError> {
        if response_head_bytes.len() > MAX_RESPONSE_HEAD_BYTES
            || body_bytes.len() > MAX_BODY_OBSERVATION_BYTES
            || wire_bytes_after_headers > MAX_TRANSPORT_WIRE_BYTES
            || !peer_socket_addr.is_ipv4()
            || elapsed_milliseconds > MAX_TRANSPORT_ELAPSED_MILLISECONDS
        {
            return Err(SourceTransportError::ObservationShapeRejected);
        }
        Ok(Self {
            response_head_bytes,
            body_bytes,
            wire_bytes_after_headers,
            peer_socket_addr,
            dns_transport_observation,
            framing,
            elapsed_milliseconds,
        })
    }

    #[must_use]
    pub fn response_head_bytes(&self) -> &[u8] {
        &self.response_head_bytes
    }

    #[must_use]
    pub fn body_bytes(&self) -> &[u8] {
        &self.body_bytes
    }

    #[must_use]
    pub const fn wire_bytes_after_headers(&self) -> u64 {
        self.wire_bytes_after_headers
    }

    #[must_use]
    pub const fn peer_socket_addr(&self) -> SocketAddr {
        self.peer_socket_addr
    }

    #[must_use]
    pub const fn dns_transport_observation(&self) -> &DnsTransportObservation {
        &self.dns_transport_observation
    }

    #[must_use]
    pub const fn framing(&self) -> RetrievalBodyFraming {
        self.framing
    }

    #[must_use]
    pub const fn elapsed_milliseconds(&self) -> u64 {
        self.elapsed_milliseconds
    }

    #[must_use]
    pub fn into_parts(self) -> RetrievalTransportSuccessParts {
        RetrievalTransportSuccessParts {
            response_head_bytes: self.response_head_bytes,
            body_bytes: self.body_bytes,
            wire_bytes_after_headers: self.wire_bytes_after_headers,
            peer_socket_addr: self.peer_socket_addr,
            dns_transport_observation: self.dns_transport_observation,
            framing: self.framing,
            elapsed_milliseconds: self.elapsed_milliseconds,
        }
    }
}

/// Public transport-only failure family; all variants are payload-free.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SourceTransportError {
    DnsUnavailable,
    ConnectFailed,
    TlsFailed,
    WriteFailed,
    ReadFailed,
    EofFramingFailure,
    ExecutionDeadline,
    TransportCancelled,
    ObservationShapeRejected,
}

impl fmt::Display for SourceTransportError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::DnsUnavailable => "DNS transport unavailable",
            Self::ConnectFailed => "transport connection failed",
            Self::TlsFailed => "TLS transport failed",
            Self::WriteFailed => "transport write failed",
            Self::ReadFailed => "transport read failed",
            Self::EofFramingFailure => "transport framing ended incompletely",
            Self::ExecutionDeadline => "transport deadline elapsed",
            Self::TransportCancelled => "transport cancelled",
            Self::ObservationShapeRejected => "transport observation shape rejected",
        })
    }
}

impl Error for SourceTransportError {}

/// Complete closed `source-retrieval/v0` policy error algebra.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RetrievalPolicyError {
    AttemptIdConflict,
    CommandIdConflict,
    RetrievalProtocolVersionMismatch,
    AttemptSourceMismatch,
    ValidatedCandidateMismatch,
    MissingAttempt,
    AttemptCompletionConflict,
    MissingOrTerminalSession,
    RequestContextMismatch,
    OperatorPolicyUnavailable,
    UnauthorizedSourceOperator,
    SourceNotRetrievable,
    StaleSourceAuthorityRevision,
    ClockUnavailable,
    ClockRegression,
    OverrideEvidenceUnavailable,
    InvalidRateOverride,
    RateOverrideAlreadyConsumed,
    RateLimitNotElapsed,
    LeaseUnavailable,
    LeaseTimeOverflow,
    LeaseExpired,
    InvalidStartAuthorization,
    StartAuthorizationAlreadyConsumed,
    AdmissionStoreUnavailable,
    PublicIpPolicyVersionMismatch,
    DnsAliasViolation,
    DnsAnswerCountViolation,
    UnsupportedAddressFamily,
    NonPublicAddress,
    PeerAddressMismatch,
    MalformedResponseHead,
    UnexpectedHttpVersion,
    InterimResponseDenied,
    RedirectDenied,
    UnexpectedStatus,
    HeaderLimitExceeded,
    InvalidContentType,
    UnexpectedContentType,
    UnsupportedContentEncoding,
    UnsupportedTransferCoding,
    AmbiguousFraming,
    DeclaredBodyTooLarge,
    ChunkLimitExceeded,
    TrailerDenied,
    WireLimitExceeded,
    BodyLimitExceeded,
    DeadlineExceeded,
}

impl fmt::Display for RetrievalPolicyError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "retrieval policy rejected: {self:?}")
    }
}

impl Error for RetrievalPolicyError {}

/// First pure phase output. It is cloneable only for independent pure branches.
pub struct RetrievalPlanCandidate {
    command_id: CommandId,
    attempt_id: RetrievalAttemptId,
    source_id: SourceId,
    authority_revision: SourceAuthorityRevision,
    canonical_host: RetrievalDnsName,
    serialized_request: SerializedRetrievalRequest,
    expected_media_type: SourceMediaType,
    minimum_interval_seconds: u32,
    maximum_response_bytes: u32,
    maximum_elapsed_seconds: u32,
    protocol_version: SourceRetrievalProtocolVersion,
    public_ip_policy_version: PublicIpPolicyVersion,
    override_request: Option<RetrievalRateOverrideRequest>,
}

impl Clone for RetrievalPlanCandidate {
    fn clone(&self) -> Self {
        Self {
            command_id: self.command_id.clone(),
            attempt_id: self.attempt_id.clone(),
            source_id: self.source_id.clone(),
            authority_revision: self.authority_revision,
            canonical_host: self.canonical_host.clone(),
            serialized_request: SerializedRetrievalRequest {
                bytes: self.serialized_request.bytes.clone(),
            },
            expected_media_type: self.expected_media_type.clone(),
            minimum_interval_seconds: self.minimum_interval_seconds,
            maximum_response_bytes: self.maximum_response_bytes,
            maximum_elapsed_seconds: self.maximum_elapsed_seconds,
            protocol_version: self.protocol_version,
            public_ip_policy_version: self.public_ip_policy_version,
            override_request: self.override_request.clone(),
        }
    }
}

macro_rules! redacted_phase_debug {
    ($name:ident) => {
        impl fmt::Debug for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str(concat!(stringify!($name), "(<redacted>)"))
            }
        }
    };
}

redacted_phase_debug!(RetrievalPlanCandidate);

/// Candidate after M60 independently admitted the full DNS observation.
pub struct ResolvedRetrievalCandidate {
    candidate: RetrievalPlanCandidate,
    admitted_addresses: Vec<Ipv4Addr>,
    selected_peer: Ipv4Addr,
}
redacted_phase_debug!(ResolvedRetrievalCandidate);

/// Candidate after exact selected-peer binding.
pub struct PeerBoundRetrievalCandidate {
    candidate: RetrievalPlanCandidate,
    peer: SocketAddr,
}
redacted_phase_debug!(PeerBoundRetrievalCandidate);

/// Candidate after strict response-head and framing authorization.
pub struct BodyAdmissionCandidate {
    candidate: RetrievalPlanCandidate,
    peer: SocketAddr,
    framing: RetrievalBodyFraming,
    trailer_declared: bool,
}
redacted_phase_debug!(BodyAdmissionCandidate);

/// Final bounded synthetic candidate; it is not `BoundedFetch` or effect authority.
///
/// Its retained fields are intentionally opaque in this B2 slice; a future separately
/// admitted owner may consume them, but no current public accessor or conversion exists.
#[allow(dead_code)]
pub struct ValidatedFetchCandidate {
    candidate: RetrievalPlanCandidate,
    peer: SocketAddr,
    body: Vec<u8>,
}
redacted_phase_debug!(ValidatedFetchCandidate);

/// Stateless pure retrieval-policy namespace.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RetrievalPolicy;

impl RetrievalPolicy {
    /// Derives one exact, immutable, non-authority request candidate.
    pub fn derive_candidate(
        subject: &RetrievalSubject,
        command: &RetrievalAttemptCommand,
    ) -> Result<RetrievalPlanCandidate, RetrievalPolicyError> {
        let policy = subject.source_retrieval_policy();
        if policy.protocol_version()
            != SourceRetrievalProtocolVersion::V0StrictHttpsIpv4Http11_20260809
        {
            return Err(RetrievalPolicyError::RetrievalProtocolVersionMismatch);
        }
        if subject.source_id() != &command.source_id {
            return Err(RetrievalPolicyError::AttemptSourceMismatch);
        }
        if subject.source_authority_revision() != command.expected_authority_revision {
            return Err(RetrievalPolicyError::StaleSourceAuthorityRevision);
        }

        let url = subject.source_url().as_str();
        let without_scheme = url
            .strip_prefix("https://")
            .ok_or(RetrievalPolicyError::RetrievalProtocolVersionMismatch)?;
        let (host, path_tail) = without_scheme
            .split_once('/')
            .ok_or(RetrievalPolicyError::RetrievalProtocolVersionMismatch)?;
        let canonical_host = RetrievalDnsName::parse(host)
            .map_err(|_| RetrievalPolicyError::RetrievalProtocolVersionMismatch)?;
        let path = format!("/{path_tail}");
        let media = policy.expected_media_type().as_bytes();
        let mut bytes = Vec::with_capacity(path.len() + host.len() + media.len() + 96);
        bytes.extend_from_slice(b"GET ");
        bytes.extend_from_slice(path.as_bytes());
        bytes.extend_from_slice(b" HTTP/1.1\r\nHost: ");
        bytes.extend_from_slice(host.as_bytes());
        bytes.extend_from_slice(b"\r\nAccept: ");
        bytes.extend_from_slice(media);
        bytes.extend_from_slice(b"\r\nAccept-Encoding: identity\r\nConnection: close\r\n\r\n");

        Ok(RetrievalPlanCandidate {
            command_id: command.command_id.clone(),
            attempt_id: command.attempt_id.clone(),
            source_id: command.source_id.clone(),
            authority_revision: command.expected_authority_revision,
            canonical_host,
            serialized_request: SerializedRetrievalRequest { bytes },
            expected_media_type: policy.expected_media_type().clone(),
            minimum_interval_seconds: policy.minimum_interval_seconds(),
            maximum_response_bytes: policy.maximum_response_bytes(),
            maximum_elapsed_seconds: policy.maximum_elapsed_seconds(),
            protocol_version: policy.protocol_version(),
            public_ip_policy_version: policy.public_ip_policy_version(),
            override_request: command.override_request.clone(),
        })
    }

    /// Evaluates the exhaustive synthetic pure-rate decision table.
    pub fn evaluate_rate(
        candidate: &RetrievalPlanCandidate,
        now: RetrievalEpochSeconds,
        last_attempt_started_at: Option<RetrievalEpochSeconds>,
        override_facts: Option<&RetrievalOverrideFacts>,
        override_consumed: bool,
    ) -> Result<RetrievalRateDecision, RetrievalPolicyError> {
        let Some(last) = last_attempt_started_at else {
            return Ok(RetrievalRateDecision::Allowed);
        };
        let elapsed = now
            .get()
            .checked_sub(last.get())
            .ok_or(RetrievalPolicyError::ClockRegression)?;
        if elapsed >= u64::from(candidate.minimum_interval_seconds) {
            return Ok(RetrievalRateDecision::Allowed);
        }
        let request = candidate
            .override_request
            .as_ref()
            .ok_or(RetrievalPolicyError::RateLimitNotElapsed)?;
        let facts = override_facts.ok_or(RetrievalPolicyError::OverrideEvidenceUnavailable)?;
        let exact = facts.evidence_id == request.evidence_id
            && facts.override_id == request.override_id
            && facts.attempt_id == candidate.attempt_id
            && facts.source_id == candidate.source_id
            && facts.authority_revision == candidate.authority_revision
            && facts.issued_at <= now
            && now <= facts.not_after;
        if !exact {
            return Err(RetrievalPolicyError::InvalidRateOverride);
        }
        if override_consumed {
            return Err(RetrievalPolicyError::RateOverrideAlreadyConsumed);
        }
        Ok(RetrievalRateDecision::AllowedWithOverride(
            request.override_id.clone(),
        ))
    }

    /// Applies exact host/CNAME/address-count/public-IPv4 policy to raw DNS shape.
    pub fn authorize_resolution(
        candidate: RetrievalPlanCandidate,
        transport_observation: DnsTransportObservation,
    ) -> Result<ResolvedRetrievalCandidate, RetrievalPolicyError> {
        let (queried_host, cname_chain, complete_addresses) = transport_observation.into_parts();
        if queried_host != candidate.canonical_host.as_str()
            || cname_chain.len() > MAX_POLICY_CNAME_DEPTH
        {
            return Err(RetrievalPolicyError::DnsAliasViolation);
        }
        let mut seen_names = BTreeSet::new();
        seen_names.insert(candidate.canonical_host.as_str().to_owned());
        for alias in cname_chain {
            if !dns_name_is_canonical(&alias) || !seen_names.insert(alias) {
                return Err(RetrievalPolicyError::DnsAliasViolation);
            }
        }
        let admitted: BTreeSet<Ipv4Addr> = complete_addresses.into_iter().collect();
        if admitted.is_empty() || admitted.len() > MAX_POLICY_ADDRESS_COUNT {
            return Err(RetrievalPolicyError::DnsAnswerCountViolation);
        }
        if admitted.iter().any(|address| !is_public_ipv4(*address)) {
            return Err(RetrievalPolicyError::NonPublicAddress);
        }
        let admitted_addresses: Vec<_> = admitted.into_iter().collect();
        let selected_peer = admitted_addresses[0];
        Ok(ResolvedRetrievalCandidate {
            candidate,
            admitted_addresses,
            selected_peer,
        })
    }

    /// Rechecks exact IPv4:443 selected-peer binding without opening a connection.
    pub fn authorize_peer(
        plan: ResolvedRetrievalCandidate,
        peer: SocketAddr,
    ) -> Result<PeerBoundRetrievalCandidate, RetrievalPolicyError> {
        let SocketAddr::V4(peer_v4) = peer else {
            return Err(RetrievalPolicyError::UnsupportedAddressFamily);
        };
        if !is_public_ipv4(*peer_v4.ip()) {
            return Err(RetrievalPolicyError::NonPublicAddress);
        }
        if peer_v4.port() != 443
            || *peer_v4.ip() != plan.selected_peer
            || !plan.admitted_addresses.contains(peer_v4.ip())
        {
            return Err(RetrievalPolicyError::PeerAddressMismatch);
        }
        if plan.candidate.public_ip_policy_version != PublicIpPolicyVersion::V0Ipv4Only20260809 {
            return Err(RetrievalPolicyError::PublicIpPolicyVersionMismatch);
        }
        Ok(PeerBoundRetrievalCandidate {
            candidate: plan.candidate,
            peer,
        })
    }

    /// Parses one strictly framed HTTP response head.
    pub fn parse_strict_response_head(
        raw: &[u8],
    ) -> Result<ResponseHeadObservation, RetrievalPolicyError> {
        if raw.contains(&0)
            || raw
                .windows(2)
                .any(|pair| pair == b"\r\r" || pair == b"\n\n")
            || raw.iter().enumerate().any(|(index, byte)| {
                (*byte == b'\r' && raw.get(index + 1) != Some(&b'\n'))
                    || (*byte == b'\n' && (index == 0 || raw[index - 1] != b'\r'))
            })
            || !raw.ends_with(b"\r\n\r\n")
        {
            return Err(RetrievalPolicyError::MalformedResponseHead);
        }
        let text =
            std::str::from_utf8(raw).map_err(|_| RetrievalPolicyError::MalformedResponseHead)?;
        if raw.len() > MAX_RESPONSE_HEAD_BYTES {
            return Err(RetrievalPolicyError::HeaderLimitExceeded);
        }
        let mut lines = text[..text.len() - 4].split("\r\n");
        let status_line = lines
            .next()
            .ok_or(RetrievalPolicyError::MalformedResponseHead)?;
        let mut status_parts = status_line.splitn(3, ' ');
        let version_text = status_parts
            .next()
            .ok_or(RetrievalPolicyError::MalformedResponseHead)?;
        let status_text = status_parts
            .next()
            .ok_or(RetrievalPolicyError::MalformedResponseHead)?;
        let reason = status_parts
            .next()
            .ok_or(RetrievalPolicyError::MalformedResponseHead)?;
        if status_text.len() != 3
            || !status_text.bytes().all(|byte| byte.is_ascii_digit())
            || !reason
                .bytes()
                .all(|byte| byte == b' ' || (b'!'..=b'~').contains(&byte))
        {
            return Err(RetrievalPolicyError::MalformedResponseHead);
        }
        let version = match version_text {
            "HTTP/1.0" => HttpVersionClass::Http10,
            "HTTP/1.1" => HttpVersionClass::Http11,
            "HTTP/2.0" => HttpVersionClass::Http2,
            "HTTP/3.0" => HttpVersionClass::Http3,
            value
                if value.len() == 8
                    && value.starts_with("HTTP/")
                    && value.as_bytes()[5].is_ascii_digit()
                    && value.as_bytes()[6] == b'.'
                    && value.as_bytes()[7].is_ascii_digit() =>
            {
                HttpVersionClass::Other
            }
            _ => return Err(RetrievalPolicyError::MalformedResponseHead),
        };
        let status_code = status_text
            .parse::<u16>()
            .map_err(|_| RetrievalPolicyError::MalformedResponseHead)?;
        let mut headers = Vec::new();
        for line in lines {
            if line.starts_with(' ') || line.starts_with('\t') {
                return Err(RetrievalPolicyError::MalformedResponseHead);
            }
            let Some((name, value)) = line.split_once(':') else {
                return Err(RetrievalPolicyError::MalformedResponseHead);
            };
            if name.is_empty()
                || name.len() > MAX_HEADER_NAME_BYTES
                || !name.bytes().all(is_tchar)
                || value.len() > MAX_HEADER_VALUE_BYTES
                || !value
                    .bytes()
                    .all(|byte| byte == b' ' || (b'!'..=b'~').contains(&byte))
            {
                return Err(
                    if name.len() > MAX_HEADER_NAME_BYTES || value.len() > MAX_HEADER_VALUE_BYTES {
                        RetrievalPolicyError::HeaderLimitExceeded
                    } else {
                        RetrievalPolicyError::MalformedResponseHead
                    },
                );
            }
            headers.push((
                name.to_ascii_lowercase(),
                value.trim_matches(' ').to_owned(),
            ));
            if headers.len() > MAX_HEADER_FIELDS {
                return Err(RetrievalPolicyError::HeaderLimitExceeded);
            }
        }
        Ok(ResponseHeadObservation {
            version,
            status_code,
            headers,
        })
    }

    /// Applies status, media-type, coding and framing policy.
    pub fn authorize_response_head(
        plan: PeerBoundRetrievalCandidate,
        head: ResponseHeadObservation,
    ) -> Result<BodyAdmissionCandidate, RetrievalPolicyError> {
        if head.version != HttpVersionClass::Http11 {
            return Err(RetrievalPolicyError::UnexpectedHttpVersion);
        }
        if (100..200).contains(&head.status_code) {
            return Err(RetrievalPolicyError::InterimResponseDenied);
        }
        if (300..400).contains(&head.status_code) {
            return Err(RetrievalPolicyError::RedirectDenied);
        }
        if head.status_code != 200 {
            return Err(RetrievalPolicyError::UnexpectedStatus);
        }

        let content_type = one_header(&head.headers, "content-type");
        let ObservedHeaderValue::One(content_type) = content_type else {
            return Err(RetrievalPolicyError::InvalidContentType);
        };
        let essence = parse_content_type(&content_type)?;
        let expected = std::str::from_utf8(plan.candidate.expected_media_type.as_bytes())
            .expect("source media type is ASCII");
        if !essence.eq_ignore_ascii_case(expected) {
            return Err(RetrievalPolicyError::UnexpectedContentType);
        }

        match one_header(&head.headers, "content-encoding") {
            ObservedHeaderValue::Missing => {}
            ObservedHeaderValue::One(value) if value.eq_ignore_ascii_case("identity") => {}
            _ => return Err(RetrievalPolicyError::UnsupportedContentEncoding),
        }

        let transfer = one_header(&head.headers, "transfer-encoding");
        let length = one_header(&head.headers, "content-length");
        let framing = match (transfer, length) {
            (ObservedHeaderValue::One(value), _) if !value.eq_ignore_ascii_case("chunked") => {
                return Err(RetrievalPolicyError::UnsupportedTransferCoding);
            }
            (ObservedHeaderValue::Repeated, _) | (_, ObservedHeaderValue::Repeated) => {
                return Err(RetrievalPolicyError::AmbiguousFraming);
            }
            (ObservedHeaderValue::One(_), ObservedHeaderValue::One(_)) => {
                return Err(RetrievalPolicyError::AmbiguousFraming);
            }
            (ObservedHeaderValue::One(_), ObservedHeaderValue::Missing) => {
                RetrievalBodyFraming::Chunked
            }
            (ObservedHeaderValue::Missing, ObservedHeaderValue::One(value)) => {
                let declared = parse_content_length(&value)?;
                if declared > u64::from(plan.candidate.maximum_response_bytes) {
                    return Err(RetrievalPolicyError::DeclaredBodyTooLarge);
                }
                RetrievalBodyFraming::ContentLength(declared)
            }
            (ObservedHeaderValue::Missing, ObservedHeaderValue::Missing) => {
                RetrievalBodyFraming::CloseDelimited
            }
        };
        let trailer_declared = !matches!(
            one_header(&head.headers, "trailer"),
            ObservedHeaderValue::Missing
        );
        Ok(BodyAdmissionCandidate {
            candidate: plan.candidate,
            peer: plan.peer,
            framing,
            trailer_declared,
        })
    }

    /// Applies chunk, trailer, framing, wire, body and deadline policy in order.
    pub fn finish_body(
        admission: BodyAdmissionCandidate,
        body: BodyObservation,
    ) -> Result<ValidatedFetchCandidate, RetrievalPolicyError> {
        if !body.framing_complete {
            return Err(RetrievalPolicyError::AmbiguousFraming);
        }
        let content_length_mismatch = match admission.framing {
            RetrievalBodyFraming::ContentLength(declared) => declared != body.bytes.len() as u64,
            RetrievalBodyFraming::Chunked | RetrievalBodyFraming::CloseDelimited => false,
        };
        if content_length_mismatch {
            return Err(RetrievalPolicyError::AmbiguousFraming);
        }
        if matches!(admission.framing, RetrievalBodyFraming::Chunked)
            && (body.chunk_count > MAX_CHUNK_COUNT
                || body.max_chunk_line_bytes == 0
                || body.max_chunk_line_bytes > MAX_CHUNK_LINE_BYTES
                || body.max_chunk_line_bytes > MAX_CHUNK_SIZE_DIGITS
                || body.saw_chunk_extension)
        {
            return Err(RetrievalPolicyError::ChunkLimitExceeded);
        }
        if admission.trailer_declared || body.trailer_field_count != 0 {
            return Err(RetrievalPolicyError::TrailerDenied);
        }
        let wire_limit = u64::from(admission.candidate.maximum_response_bytes)
            .checked_add(WIRE_OVERHEAD_BYTES)
            .expect("bounded policy plus fixed overhead fits u64");
        if body.wire_bytes_after_headers > wire_limit {
            return Err(RetrievalPolicyError::WireLimitExceeded);
        }
        if body.bytes.len() as u64 > u64::from(admission.candidate.maximum_response_bytes) {
            return Err(RetrievalPolicyError::BodyLimitExceeded);
        }
        let deadline = u64::from(admission.candidate.maximum_elapsed_seconds) * 1_000;
        if body.elapsed_milliseconds > deadline {
            return Err(RetrievalPolicyError::DeadlineExceeded);
        }
        Ok(ValidatedFetchCandidate {
            candidate: admission.candidate,
            peer: admission.peer,
            body: body.bytes,
        })
    }
}

fn one_header(headers: &[(String, String)], name: &str) -> ObservedHeaderValue {
    let mut found = headers.iter().filter(|(candidate, _)| candidate == name);
    let Some((_, first)) = found.next() else {
        return ObservedHeaderValue::Missing;
    };
    if found.next().is_some() {
        ObservedHeaderValue::Repeated
    } else {
        ObservedHeaderValue::One(first.clone())
    }
}

fn parse_content_type(value: &str) -> Result<&str, RetrievalPolicyError> {
    let mut parts = value.split(';');
    let essence = parts
        .next()
        .ok_or(RetrievalPolicyError::InvalidContentType)?
        .trim_matches(' ');
    let Some((kind, subtype)) = essence.split_once('/') else {
        return Err(RetrievalPolicyError::InvalidContentType);
    };
    if kind.is_empty()
        || subtype.is_empty()
        || kind.len() > MAX_CONTENT_PARAMETER_BYTES
        || subtype.len() > MAX_CONTENT_PARAMETER_BYTES
        || !kind.bytes().all(is_tchar)
        || !subtype.bytes().all(is_tchar)
    {
        return Err(RetrievalPolicyError::InvalidContentType);
    }
    let mut names = BTreeSet::new();
    let mut count = 0usize;
    for parameter in parts {
        count += 1;
        if count > MAX_CONTENT_PARAMETERS {
            return Err(RetrievalPolicyError::InvalidContentType);
        }
        let parameter = parameter.trim_matches(' ');
        let Some((name, raw_value)) = parameter.split_once('=') else {
            return Err(RetrievalPolicyError::InvalidContentType);
        };
        let (value, quoted) = match raw_value
            .strip_prefix('"')
            .and_then(|inner| inner.strip_suffix('"'))
        {
            Some(value) => (value, true),
            None => (raw_value, false),
        };
        if name.is_empty()
            || value.is_empty()
            || name.len() > MAX_CONTENT_PARAMETER_BYTES
            || value.len() > MAX_CONTENT_PARAMETER_BYTES
            || !name.bytes().all(is_tchar)
            || !value
                .bytes()
                .all(|byte| is_tchar(byte) || (quoted && byte == b' '))
            || !names.insert(name.to_ascii_lowercase())
        {
            return Err(RetrievalPolicyError::InvalidContentType);
        }
    }
    Ok(essence)
}

fn parse_content_length(value: &str) -> Result<u64, RetrievalPolicyError> {
    if value == "0" {
        return Ok(0);
    }
    if value.is_empty()
        || value.starts_with('0')
        || !value.bytes().all(|byte| byte.is_ascii_digit())
    {
        return Err(RetrievalPolicyError::AmbiguousFraming);
    }
    value
        .parse()
        .map_err(|_| RetrievalPolicyError::DeclaredBodyTooLarge)
}

const fn is_tchar(byte: u8) -> bool {
    byte.is_ascii_alphanumeric()
        || matches!(
            byte,
            b'!' | b'#'
                | b'$'
                | b'%'
                | b'&'
                | b'\''
                | b'*'
                | b'+'
                | b'-'
                | b'.'
                | b'^'
                | b'_'
                | b'`'
                | b'|'
                | b'~'
        )
}

fn is_public_ipv4(address: Ipv4Addr) -> bool {
    let value = u32::from(address);
    ![
        (u32::from_be_bytes([0, 0, 0, 0]), 8),
        (u32::from_be_bytes([10, 0, 0, 0]), 8),
        (u32::from_be_bytes([100, 64, 0, 0]), 10),
        (u32::from_be_bytes([127, 0, 0, 0]), 8),
        (u32::from_be_bytes([169, 254, 0, 0]), 16),
        (u32::from_be_bytes([172, 16, 0, 0]), 12),
        (u32::from_be_bytes([192, 0, 0, 0]), 24),
        (u32::from_be_bytes([192, 0, 2, 0]), 24),
        (u32::from_be_bytes([192, 88, 99, 0]), 24),
        (u32::from_be_bytes([192, 168, 0, 0]), 16),
        (u32::from_be_bytes([198, 18, 0, 0]), 15),
        (u32::from_be_bytes([198, 51, 100, 0]), 24),
        (u32::from_be_bytes([203, 0, 113, 0]), 24),
        (u32::from_be_bytes([224, 0, 0, 0]), 4),
        (u32::from_be_bytes([240, 0, 0, 0]), 4),
    ]
    .into_iter()
    .any(|(network, prefix)| {
        let mask = u32::MAX.checked_shl(32 - prefix).unwrap_or(0);
        value & mask == network & mask
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn private_helpers_enforce_exact_dns_and_public_ip_boundaries() {
        assert!(dns_name_is_canonical("a.example"));
        assert!(!dns_name_is_canonical("A.example"));
        assert!(!dns_name_is_canonical("a.example."));
        assert!(is_public_ipv4(Ipv4Addr::new(8, 8, 8, 8)));
        assert!(!is_public_ipv4(Ipv4Addr::new(10, 0, 0, 1)));
        assert!(!is_public_ipv4(Ipv4Addr::new(203, 0, 113, 255)));
    }

    #[test]
    fn parser_and_header_helpers_are_non_vacuous() {
        let head = RetrievalPolicy::parse_strict_response_head(
            b"HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\n\r\n",
        )
        .expect("strict head");
        assert_eq!(head.status_code, 200);
        assert_eq!(
            one_header(&head.headers, "content-type"),
            ObservedHeaderValue::One("text/plain".to_owned())
        );
        assert_eq!(parse_content_length("0"), Ok(0));
        assert_eq!(
            parse_content_length("01"),
            Err(RetrievalPolicyError::AmbiguousFraming)
        );
        assert_eq!(
            parse_content_length("18446744073709551616"),
            Err(RetrievalPolicyError::DeclaredBodyTooLarge)
        );
    }

    #[test]
    fn exact_serialized_request_is_closed_and_byte_exact() {
        let source_id = SourceId::parse("source:wire".to_owned()).expect("source id");
        let policy = crate::source_registry::SourceRetrievalPolicy::new(
            10,
            64,
            1,
            SourceMediaType::parse("text/plain").expect("media"),
            SourceRetrievalProtocolVersion::V0StrictHttpsIpv4Http11_20260809,
            PublicIpPolicyVersion::V0Ipv4Only20260809,
        )
        .expect("policy");
        let definition = crate::source_registry::SourceDefinition::proposed(
            source_id.clone(),
            crate::source_registry::SourceOwner::parse("Synthetic Wire Fixture").expect("owner"),
            crate::source_registry::SourceUrl::parse(
                "https://example.invalid/exact/path".to_owned(),
            )
            .expect("url"),
            crate::SourceAuthority::ReviewedOfficialSource,
            policy,
        )
        .expect("definition");
        let revision = definition.authority_revision();
        let mut registry = crate::source_registry::SourceRegistry::new();
        registry.propose(definition).expect("propose fixture");
        let review = crate::source_registry::SourceReviewReceipt::new(
            crate::source_registry::SourceReviewerId::parse("reviewer:wire").expect("reviewer"),
            crate::source_registry::SourceReviewEvidenceId::parse("evidence:wire-review")
                .expect("review evidence"),
            crate::source_registry::SourceReviewEvidenceId::parse("evidence:wire-permission")
                .expect("permission evidence"),
            crate::source_registry::SourceReviewEvidenceId::parse("evidence:wire-rate")
                .expect("rate evidence"),
            crate::source_registry::SourceReviewEvidenceId::parse("evidence:wire-fixture")
                .expect("fixture evidence"),
        );
        registry
            .approve(&source_id, revision, review)
            .expect("approve synthetic fixture only");
        let subject = registry
            .retrieval_subject(&source_id)
            .expect("retrieval subject");
        let command = RetrievalAttemptCommand::new(
            CommandId::parse("command:wire").expect("command"),
            RetrievalAttemptId::new("attempt:wire".to_owned()).expect("attempt"),
            source_id,
            subject.source_authority_revision(),
            None,
        );
        let candidate =
            RetrievalPolicy::derive_candidate(&subject, &command).expect("wire candidate");
        let bytes = candidate.serialized_request.as_bytes();
        assert_eq!(
            bytes,
            b"GET /exact/path HTTP/1.1\r\nHost: example.invalid\r\nAccept: text/plain\r\nAccept-Encoding: identity\r\nConnection: close\r\n\r\n"
        );
        assert!(
            !bytes
                .windows(b"User-Agent".len())
                .any(|window| window == b"User-Agent")
        );
    }
}
