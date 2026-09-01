//! Exhaustive reducer mapping M10 wire responses and transport outcomes to a
//! typed client state and stable exit class.
//!
//! # Exhaustiveness contract
//!
//! Every [`ClientResponseDto`], [`ClientErrorDto`], [`M71OutcomeDto`],
//! [`M71LineageDto`], [`FreshnessDto`], [`CannotVerifyReasonDto`],
//! [`WireErrorClassDto`] and [`RedactionDto`] variant is matched by an explicit
//! arm. There is no underscore wildcard arm anywhere in this module: adding a
//! new variant to any matched wire enum is a compile error, never a silent
//! fall-through. The `no_domain_dependencies` test suite pins this invariant by
//! scanning the source for wildcard arms.

use serde::Serialize;

use ustc_campus_agent_client_protocol::{
    CannotVerifyReasonDto, CapabilityListDto, ClientErrorDto, ClientProtocolMajor,
    ClientResponseDto, FreshnessDto, M10WireErrorDto, M71LineageDto, M71OutcomeDto, M71TerminalDto,
    ProtocolCompatibilityDto, RedactionDto, RetryabilityDto, ServerInfoDto, UnixMillis,
    WireErrorClassDto, WireText,
};

use crate::{Origin, RESULT_SCHEMA};

/// Stable CLI exit class. Codes follow `cli/v2.1` §7.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ExitClass {
    Success,
    Usage,
    Authentication,
    Policy,
    Compatibility,
    Unavailable,
    Conflict,
    OutcomeUnknown,
    Protocol,
}

impl ExitClass {
    #[must_use]
    pub const fn code(self) -> i32 {
        match self {
            ExitClass::Success => 0,
            ExitClass::Usage => 2,
            ExitClass::Authentication => 3,
            ExitClass::Policy => 4,
            ExitClass::Compatibility => 5,
            ExitClass::Unavailable => 6,
            ExitClass::Conflict => 7,
            ExitClass::OutcomeUnknown => 8,
            ExitClass::Protocol => 9,
        }
    }
}

/// Reduced error subclass carried inside [`ClientState::Error`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorClass {
    Usage,
    Authentication,
    Policy,
    Compatibility,
    Unavailable,
    Conflict,
    Protocol,
}

/// Client-facing projection of the M71 outcome kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OutcomeClass {
    Found,
    NotYetKnown,
    Archived,
    NotFound,
    Conflict,
    CannotVerify,
}

/// Client-facing projection of the M71 lineage kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LineageClass {
    Verified,
    Unverified,
    NotRequired,
}

/// Client-facing projection of the Found freshness kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FreshnessClass {
    Fresh,
    Stale,
}

/// Client-facing projection of the CannotVerify reason kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ReasonClass {
    SourceRevisionUnverified,
    EffectiveIntervalMissing,
    LastVerifiedStaleBeyondPolicy,
    PublicEvidenceProjectionOverflow,
}

/// Client-facing projection of the Available redaction kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RedactionClass {
    Public,
    AuthenticatedOwner,
    Operator,
}

/// How a terminal response was delivered: `Accepted` issues a response-only
/// public capability; `Available` carries the redaction class of the lookup.
///
/// `Debug` is implemented manually to redact the capability bearer. Only
/// `Some`/`None` (was a capability issued?) is surfaced as class info; the
/// plaintext value is never formatted. This owns the no-capability-in-Debug
/// contract at the M80 layer rather than relying on `WireText`'s redaction.
#[derive(Clone, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum TerminalKind {
    Accepted { public_capability: Option<WireText> },
    Available { redaction: RedactionClass },
}

impl std::fmt::Debug for TerminalKind {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TerminalKind::Accepted { public_capability } => {
                let redacted = match public_capability {
                    Some(_) => "Some(<redacted>)",
                    None => "None",
                };
                formatter
                    .debug_struct("Accepted")
                    .field("public_capability", &redacted)
                    .finish()
            }
            TerminalKind::Available { redaction } => formatter
                .debug_struct("Available")
                .field("redaction", redaction)
                .finish(),
        }
    }
}

/// Reduced, client-owned state. This is the canonical render shape: it carries
/// the exhaustive class projection plus the full validated terminal for
/// automation that needs the complete server-owned view.
///
/// `Debug` is implemented manually to delegate to [`TerminalKind`]'s redacted
/// `Debug`, ensuring the capability bearer nested in the `Terminal` variant is
/// never formatted in plaintext.
#[derive(Clone, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ClientState {
    ServerInfo {
        info: ServerInfoDto,
    },
    Capabilities {
        capabilities: CapabilityListDto,
    },
    UpgradeRequired {
        client_major: ClientProtocolMajor,
        minimum_client_major: ClientProtocolMajor,
        server_major: ClientProtocolMajor,
    },
    IncompatibleProtocol {
        client_major: Option<ClientProtocolMajor>,
        supported_majors: [ClientProtocolMajor; 1],
        server_major: ClientProtocolMajor,
    },
    Terminal {
        command_id: WireText,
        terminal_kind: TerminalKind,
        outcome_class: OutcomeClass,
        lineage_class: LineageClass,
        freshness_class: Option<FreshnessClass>,
        reason_class: Option<ReasonClass>,
        #[serde(skip_serializing_if = "Option::is_none", default)]
        terminal: Option<Box<M71TerminalDto>>,
    },
    Incomplete {
        command_id: Option<WireText>,
        retry_not_before: Option<UnixMillis>,
    },
    Unavailable,
    Error {
        error_class: ErrorClass,
        wire_code: WireText,
        retryability: RetryabilityDto,
    },
}

impl std::fmt::Debug for ClientState {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ClientState::ServerInfo { info } => formatter
                .debug_struct("ServerInfo")
                .field("info", info)
                .finish(),
            ClientState::Capabilities { capabilities } => formatter
                .debug_struct("Capabilities")
                .field("capabilities", capabilities)
                .finish(),
            ClientState::UpgradeRequired {
                client_major,
                minimum_client_major,
                server_major,
            } => formatter
                .debug_struct("UpgradeRequired")
                .field("client_major", client_major)
                .field("minimum_client_major", minimum_client_major)
                .field("server_major", server_major)
                .finish(),
            ClientState::IncompatibleProtocol {
                client_major,
                supported_majors,
                server_major,
            } => formatter
                .debug_struct("IncompatibleProtocol")
                .field("client_major", client_major)
                .field("supported_majors", supported_majors)
                .field("server_major", server_major)
                .finish(),
            ClientState::Terminal {
                command_id,
                terminal_kind,
                outcome_class,
                lineage_class,
                freshness_class,
                reason_class,
                terminal,
            } => {
                let mut s = formatter.debug_struct("Terminal");
                s.field("command_id", command_id);
                s.field("terminal_kind", terminal_kind);
                s.field("outcome_class", outcome_class);
                s.field("lineage_class", lineage_class);
                s.field("freshness_class", freshness_class);
                s.field("reason_class", reason_class);
                s.field("terminal", terminal);
                s.finish()
            }
            ClientState::Incomplete {
                command_id,
                retry_not_before,
            } => {
                let mut s = formatter.debug_struct("Incomplete");
                s.field("command_id", command_id);
                s.field("retry_not_before", retry_not_before);
                s.finish()
            }
            ClientState::Unavailable => formatter.write_str("Unavailable"),
            ClientState::Error {
                error_class,
                wire_code,
                retryability,
            } => {
                let mut s = formatter.debug_struct("Error");
                s.field("error_class", error_class);
                s.field("wire_code", wire_code);
                s.field("retryability", retryability);
                s.finish()
            }
        }
    }
}

/// Canonical `ustc-client-result/v1` envelope serialized to stdout.
#[derive(Debug, Clone, Serialize)]
pub struct ClientResult<'a> {
    schema: &'a str,
    exit_class: ExitClass,
    exit_code: i32,
    origin: Origin,
    state: &'a ClientState,
    provenance: &'a ustc_campus_agent_client_protocol::ClientProvenanceDto,
}

/// Stable client-originated wire codes for transport failures. These are
/// distinct from any server wire code and are always paired with
/// [`Origin::Transport`].
const TRANSPORT_MALFORMED_CODE: &str = "client.transport.malformed_frame";
const UNEXPECTED_PRODUCT_RESPONSE_CODE: &str = "client.protocol.unexpected_product_response";

fn static_text(value: &'static str) -> WireText {
    // Static transport codes are non-empty, ASCII, and well under the bound, so
    // parsing is a documented invariant.
    WireText::parse(value).expect("static transport code is valid wire text")
}

/// Reduces a server-owned [`ClientResponseDto`] to a typed [`ClientState`].
#[must_use]
pub fn reduce_response(response: ClientResponseDto) -> ClientState {
    match response {
        ClientResponseDto::ServerInfo { info } => ClientState::ServerInfo { info },
        ClientResponseDto::Capabilities { capabilities } => {
            ClientState::Capabilities { capabilities }
        }
        ClientResponseDto::Compatibility { compatibility } => match compatibility {
            ProtocolCompatibilityDto::UpgradeRequired {
                client_major,
                minimum_client_major,
                server_major,
            } => ClientState::UpgradeRequired {
                client_major,
                minimum_client_major,
                server_major,
            },
            ProtocolCompatibilityDto::IncompatibleProtocol {
                client_major,
                supported_majors,
                server_major,
            } => ClientState::IncompatibleProtocol {
                client_major,
                supported_majors,
                server_major,
            },
        },
        ClientResponseDto::Accepted {
            command_id,
            terminal,
            public_capability,
        } => {
            let (outcome_class, lineage_class, freshness_class, reason_class) =
                classify_terminal(&terminal);
            ClientState::Terminal {
                command_id,
                terminal_kind: TerminalKind::Accepted { public_capability },
                outcome_class,
                lineage_class,
                freshness_class,
                reason_class,
                terminal: Some(terminal),
            }
        }
        ClientResponseDto::ChangeFeedAccepted { .. }
        | ClientResponseDto::OpportunityAccepted { .. }
        | ClientResponseDto::OpportunityRejected { .. } => ClientState::Error {
            error_class: ErrorClass::Protocol,
            wire_code: static_text(UNEXPECTED_PRODUCT_RESPONSE_CODE),
            retryability: RetryabilityDto::NotRetryable,
        },
        ClientResponseDto::Available {
            command_id,
            terminal,
            redaction,
        } => {
            let (outcome_class, lineage_class, freshness_class, reason_class) =
                classify_terminal(&terminal);
            ClientState::Terminal {
                command_id,
                terminal_kind: TerminalKind::Available {
                    redaction: classify_redaction(redaction),
                },
                outcome_class,
                lineage_class,
                freshness_class,
                reason_class,
                terminal: Some(terminal),
            }
        }
        ClientResponseDto::Incomplete {
            command_id,
            retry_not_before,
        } => ClientState::Incomplete {
            command_id: Some(command_id),
            retry_not_before: Some(retry_not_before),
        },
        ClientResponseDto::Unavailable => ClientState::Unavailable,
        ClientResponseDto::Error { error } => {
            let (error_class, wire_code, retryability) = classify_error(error);
            ClientState::Error {
                error_class,
                wire_code,
                retryability,
            }
        }
    }
}

/// Reduces a transport failure to a stable [`ClientState`].
///
/// `Unavailable` (connect/early-write failure) maps to the same opaque
/// [`ClientState::Unavailable`] as a server-typed denial, preserving the
/// no-existence-side-channel property. `OutcomeUnknown` (response timeout after
/// the request was sent) maps to [`ClientState::Incomplete`] with no server
/// retry hint. `Malformed` maps to a protocol error with a stable client-side
/// wire code.
#[must_use]
pub fn reduce_transport_failure(error: crate::TransportError) -> ClientState {
    match error {
        crate::TransportError::Unavailable => ClientState::Unavailable,
        crate::TransportError::OutcomeUnknown => ClientState::Incomplete {
            command_id: None,
            retry_not_before: None,
        },
        crate::TransportError::Malformed => ClientState::Error {
            error_class: ErrorClass::Protocol,
            wire_code: static_text(TRANSPORT_MALFORMED_CODE),
            retryability: RetryabilityDto::NotRetryable,
        },
    }
}

/// Returns the stable exit class for a reduced state.
#[must_use]
pub fn exit_class(state: &ClientState) -> ExitClass {
    match state {
        ClientState::ServerInfo { .. } | ClientState::Capabilities { .. } => ExitClass::Success,
        ClientState::UpgradeRequired { .. } | ClientState::IncompatibleProtocol { .. } => {
            ExitClass::Compatibility
        }
        ClientState::Terminal { .. } => ExitClass::Success,
        ClientState::Incomplete { .. } => ExitClass::OutcomeUnknown,
        ClientState::Unavailable => ExitClass::Unavailable,
        ClientState::Error { error_class, .. } => match error_class {
            ErrorClass::Usage => ExitClass::Usage,
            ErrorClass::Authentication => ExitClass::Authentication,
            ErrorClass::Policy => ExitClass::Policy,
            ErrorClass::Compatibility => ExitClass::Compatibility,
            ErrorClass::Unavailable => ExitClass::Unavailable,
            ErrorClass::Conflict => ExitClass::Conflict,
            ErrorClass::Protocol => ExitClass::Protocol,
        },
    }
}

/// Renders the canonical deterministic `ustc-client-result/v1` JSON envelope.
///
/// Field order is fixed by the struct definition; all nested enums use
/// `tag = "kind"` / `rename_all = "snake_case"`, so the output is stable
/// regardless of map iteration order.
#[must_use]
pub fn render_result(
    state: &ClientState,
    origin: Origin,
    provenance: &ustc_campus_agent_client_protocol::ClientProvenanceDto,
) -> String {
    let class = exit_class(state);
    let result = ClientResult {
        schema: RESULT_SCHEMA,
        exit_class: class,
        exit_code: class.code(),
        origin,
        state,
        provenance,
    };
    serde_json::to_string(&result).expect("canonical client result is serializable")
}

fn classify_terminal(
    terminal: &M71TerminalDto,
) -> (
    OutcomeClass,
    LineageClass,
    Option<FreshnessClass>,
    Option<ReasonClass>,
) {
    let outcome = terminal.outcome();
    let lineage = terminal.lineage();
    let lineage_class = match lineage {
        M71LineageDto::Verified { .. } => LineageClass::Verified,
        M71LineageDto::Unverified { .. } => LineageClass::Unverified,
        M71LineageDto::NotRequired { .. } => LineageClass::NotRequired,
    };
    let (outcome_class, freshness_class, reason_class) = match outcome {
        M71OutcomeDto::Found { freshness, .. } => (
            OutcomeClass::Found,
            Some(classify_freshness(freshness)),
            None,
        ),
        M71OutcomeDto::NotYetKnown { .. } => (OutcomeClass::NotYetKnown, None, None),
        M71OutcomeDto::Archived { .. } => (OutcomeClass::Archived, None, None),
        M71OutcomeDto::NotFound { .. } => (OutcomeClass::NotFound, None, None),
        M71OutcomeDto::Conflict { .. } => (OutcomeClass::Conflict, None, None),
        M71OutcomeDto::CannotVerify { reason, .. } => (
            OutcomeClass::CannotVerify,
            None,
            Some(classify_reason(reason)),
        ),
    };
    (outcome_class, lineage_class, freshness_class, reason_class)
}

fn classify_freshness(freshness: &FreshnessDto) -> FreshnessClass {
    match freshness {
        FreshnessDto::Fresh => FreshnessClass::Fresh,
        FreshnessDto::Stale { .. } => FreshnessClass::Stale,
    }
}

fn classify_reason(reason: &CannotVerifyReasonDto) -> ReasonClass {
    match reason {
        CannotVerifyReasonDto::SourceRevisionUnverified => ReasonClass::SourceRevisionUnverified,
        CannotVerifyReasonDto::EffectiveIntervalMissing => ReasonClass::EffectiveIntervalMissing,
        CannotVerifyReasonDto::LastVerifiedStaleBeyondPolicy => {
            ReasonClass::LastVerifiedStaleBeyondPolicy
        }
        CannotVerifyReasonDto::PublicEvidenceProjectionOverflow { .. } => {
            ReasonClass::PublicEvidenceProjectionOverflow
        }
    }
}

fn classify_redaction(redaction: RedactionDto) -> RedactionClass {
    match redaction {
        RedactionDto::Public => RedactionClass::Public,
        RedactionDto::AuthenticatedOwner => RedactionClass::AuthenticatedOwner,
        RedactionDto::Operator => RedactionClass::Operator,
    }
}

fn classify_error(error: ClientErrorDto) -> (ErrorClass, WireText, RetryabilityDto) {
    match error {
        ClientErrorDto::Admission { error: wire_error } => {
            let M10WireErrorDto {
                class,
                retryability,
                wire_code,
                echo: _,
            } = wire_error;
            let error_class = match class {
                WireErrorClassDto::IdempotencyStoreUnavailable => ErrorClass::Unavailable,
                WireErrorClassDto::ConflictingEnvelope => ErrorClass::Conflict,
                WireErrorClassDto::DescriptorSnapshotAbsent => ErrorClass::Conflict,
                WireErrorClassDto::DescriptorSnapshotMismatch => ErrorClass::Conflict,
                WireErrorClassDto::PolicyDenied => ErrorClass::Policy,
                WireErrorClassDto::PolicyExpired => ErrorClass::Conflict,
                WireErrorClassDto::SessionNotFound => ErrorClass::Authentication,
                WireErrorClassDto::SessionIdMismatch => ErrorClass::Authentication,
                WireErrorClassDto::SessionNotAdmitted => ErrorClass::Authentication,
                WireErrorClassDto::CapabilityMissing => ErrorClass::Policy,
                WireErrorClassDto::CapabilityDisabled => ErrorClass::Policy,
                WireErrorClassDto::CapabilityRevoked => ErrorClass::Policy,
                WireErrorClassDto::InfrastructurePortUnavailable => ErrorClass::Unavailable,
                WireErrorClassDto::MalformedCommand => ErrorClass::Usage,
            };
            (error_class, wire_code, retryability)
        }
        ClientErrorDto::InternalInvariant { wire_code } => (
            ErrorClass::Protocol,
            wire_code,
            RetryabilityDto::NotRetryable,
        ),
        ClientErrorDto::Infrastructure {
            retryable,
            wire_code,
        } => {
            let error_class = if retryable {
                ErrorClass::Unavailable
            } else {
                ErrorClass::Protocol
            };
            let retryability = if retryable {
                RetryabilityDto::Retryable
            } else {
                RetryabilityDto::NotRetryable
            };
            (error_class, wire_code, retryability)
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    #[test]
    fn affairs_reducer_rejects_change_feed_terminal_as_protocol_mismatch() {
        let response = ClientResponseDto::ChangeFeedAccepted {
            command_id: WireText::parse("cmd:change").unwrap(),
            terminal: Box::new(
                ustc_campus_agent_client_protocol::M70ChangeFeedTerminalDto::new(
                    ustc_campus_agent_client_protocol::M70ChangeFeedOutcomeDto::NotFound {
                        board_id: WireText::parse("board:fixture").unwrap(),
                    },
                ),
            ),
        };
        match reduce_response(response) {
            ClientState::Error {
                error_class: ErrorClass::Protocol,
                wire_code,
                retryability: RetryabilityDto::NotRetryable,
            } => assert_eq!(wire_code.as_str(), UNEXPECTED_PRODUCT_RESPONSE_CODE),
            other => panic!("expected protocol mismatch, got {other:?}"),
        }
    }

    #[test]
    fn exit_codes_match_cli_contract() {
        assert_eq!(ExitClass::Success.code(), 0);
        assert_eq!(ExitClass::Usage.code(), 2);
        assert_eq!(ExitClass::Authentication.code(), 3);
        assert_eq!(ExitClass::Policy.code(), 4);
        assert_eq!(ExitClass::Compatibility.code(), 5);
        assert_eq!(ExitClass::Unavailable.code(), 6);
        assert_eq!(ExitClass::Conflict.code(), 7);
        assert_eq!(ExitClass::OutcomeUnknown.code(), 8);
        assert_eq!(ExitClass::Protocol.code(), 9);
    }
}
