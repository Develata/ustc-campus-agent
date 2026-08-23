use serde::{Deserialize, Deserializer, Serialize, de};

use crate::value::{UnixMillis, WireText};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RetryabilityDto {
    Retryable,
    RetryableAfterChange,
    NotRetryable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WireErrorClassDto {
    IdempotencyStoreUnavailable,
    ConflictingEnvelope,
    DescriptorSnapshotAbsent,
    DescriptorSnapshotMismatch,
    PolicyDenied,
    PolicyExpired,
    SessionNotFound,
    SessionIdMismatch,
    SessionNotAdmitted,
    CapabilityMissing,
    CapabilityDisabled,
    CapabilityRevoked,
    InfrastructurePortUnavailable,
    MalformedCommand,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum EchoPayloadDto {
    None,
    Operation {
        operation_id: WireText,
    },
    Envelope {
        operation_id: WireText,
        idempotency_key: WireText,
    },
    SnapshotMismatch {
        command_operation_id: WireText,
        snapshot_operation_id: WireText,
    },
    PolicyDenied {
        operation_id: WireText,
        permission_class: WireText,
    },
    PolicyExpired {
        operation_id: WireText,
        policy_snapshot_id: WireText,
    },
    SessionId {
        requested_session_id: WireText,
    },
    SessionMismatch {
        requested_session_id: WireText,
        loaded_session_id: WireText,
    },
    SessionNotAdmitted {
        requested_session_id: WireText,
        observed_at: UnixMillis,
    },
    Capability {
        operation_id: WireText,
        actor_kind: WireText,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct M10WireErrorDto {
    pub class: WireErrorClassDto,
    pub retryability: RetryabilityDto,
    pub wire_code: WireText,
    pub echo: EchoPayloadDto,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct UncheckedM10WireErrorDto {
    class: WireErrorClassDto,
    retryability: RetryabilityDto,
    wire_code: WireText,
    echo: EchoPayloadDto,
}

impl M10WireErrorDto {
    pub fn try_new(
        class: WireErrorClassDto,
        retryability: RetryabilityDto,
        wire_code: WireText,
        echo: EchoPayloadDto,
    ) -> Result<Self, WireErrorValidationError> {
        let (expected_retry, expected_code) = expected_relation(class);
        if retryability != expected_retry || wire_code.as_str() != expected_code {
            return Err(WireErrorValidationError);
        }
        if !echo_matches_class(class, &echo) {
            return Err(WireErrorValidationError);
        }
        Ok(Self {
            class,
            retryability,
            wire_code,
            echo,
        })
    }
}

impl<'de> Deserialize<'de> for M10WireErrorDto {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = UncheckedM10WireErrorDto::deserialize(deserializer)?;
        Self::try_new(raw.class, raw.retryability, raw.wire_code, raw.echo)
            .map_err(de::Error::custom)
    }
}

fn expected_relation(class: WireErrorClassDto) -> (RetryabilityDto, &'static str) {
    use RetryabilityDto::{NotRetryable, Retryable, RetryableAfterChange};
    use WireErrorClassDto::*;
    match class {
        IdempotencyStoreUnavailable => (Retryable, "idempotency_store_unavailable"),
        ConflictingEnvelope => (RetryableAfterChange, "conflicting_envelope"),
        DescriptorSnapshotAbsent => (NotRetryable, "descriptor_snapshot_absent"),
        DescriptorSnapshotMismatch => (NotRetryable, "descriptor_snapshot_mismatch"),
        PolicyDenied => (NotRetryable, "policy_denied"),
        PolicyExpired => (RetryableAfterChange, "policy_expired"),
        SessionNotFound => (RetryableAfterChange, "session_not_found"),
        SessionIdMismatch => (NotRetryable, "session_id_mismatch"),
        SessionNotAdmitted => (RetryableAfterChange, "session_not_admitted"),
        CapabilityMissing => (RetryableAfterChange, "capability_missing"),
        CapabilityDisabled => (RetryableAfterChange, "capability_disabled"),
        CapabilityRevoked => (NotRetryable, "capability_revoked"),
        InfrastructurePortUnavailable => (Retryable, "infrastructure_port_unavailable"),
        MalformedCommand => (RetryableAfterChange, "malformed_command"),
    }
}

fn echo_matches_class(class: WireErrorClassDto, echo: &EchoPayloadDto) -> bool {
    use WireErrorClassDto::*;
    match class {
        IdempotencyStoreUnavailable | DescriptorSnapshotAbsent | InfrastructurePortUnavailable => {
            matches!(echo, EchoPayloadDto::Operation { .. })
        }
        ConflictingEnvelope => matches!(echo, EchoPayloadDto::Envelope { .. }),
        DescriptorSnapshotMismatch => matches!(echo, EchoPayloadDto::SnapshotMismatch { .. }),
        PolicyDenied => matches!(echo, EchoPayloadDto::PolicyDenied { .. }),
        PolicyExpired => matches!(echo, EchoPayloadDto::PolicyExpired { .. }),
        SessionNotFound => matches!(echo, EchoPayloadDto::SessionId { .. }),
        SessionIdMismatch => matches!(echo, EchoPayloadDto::SessionMismatch { .. }),
        SessionNotAdmitted => matches!(echo, EchoPayloadDto::SessionNotAdmitted { .. }),
        CapabilityMissing | CapabilityDisabled | CapabilityRevoked => {
            matches!(echo, EchoPayloadDto::Capability { .. })
        }
        MalformedCommand => {
            matches!(
                echo,
                EchoPayloadDto::None | EchoPayloadDto::Operation { .. }
            )
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WireErrorValidationError;

impl std::fmt::Display for WireErrorValidationError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("M10 wire error relation is invalid")
    }
}
impl std::error::Error for WireErrorValidationError {}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ClientErrorDto {
    Admission {
        error: M10WireErrorDto,
    },
    InternalInvariant {
        wire_code: WireText,
    },
    Infrastructure {
        retryable: bool,
        wire_code: WireText,
    },
}
