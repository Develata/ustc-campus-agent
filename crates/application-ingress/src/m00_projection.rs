use ustc_campus_agent_client_protocol::{
    ClientErrorDto, EchoPayloadDto, M10WireErrorDto, RetryabilityDto, UnixMillis,
    WireErrorClassDto, WireText,
};
use ustc_campus_agent_core::request_context::{
    ActorKind, AdmissionRejectionProjection, PermissionClass, RequestContextRejection,
};

pub fn project_rejection(rejection: &RequestContextRejection) -> ClientErrorDto {
    let (class, retryability, code, echo) = match rejection.projection() {
        AdmissionRejectionProjection::IdempotencyStoreUnavailable { operation_id } => (
            WireErrorClassDto::IdempotencyStoreUnavailable,
            RetryabilityDto::Retryable,
            "idempotency_store_unavailable",
            operation_echo(operation_id.as_str()),
        ),
        AdmissionRejectionProjection::ConflictingEnvelope {
            operation_id,
            idempotency_key,
        } => (
            WireErrorClassDto::ConflictingEnvelope,
            RetryabilityDto::RetryableAfterChange,
            "conflicting_envelope",
            EchoPayloadDto::Envelope {
                operation_id: wire(operation_id.as_str()),
                idempotency_key: wire(idempotency_key.as_str()),
            },
        ),
        AdmissionRejectionProjection::DescriptorSnapshotAbsent { operation_id } => (
            WireErrorClassDto::DescriptorSnapshotAbsent,
            RetryabilityDto::NotRetryable,
            "descriptor_snapshot_absent",
            operation_echo(operation_id.as_str()),
        ),
        AdmissionRejectionProjection::DescriptorSnapshotMismatch {
            command_operation_id,
            snapshot_operation_id,
        } => (
            WireErrorClassDto::DescriptorSnapshotMismatch,
            RetryabilityDto::NotRetryable,
            "descriptor_snapshot_mismatch",
            EchoPayloadDto::SnapshotMismatch {
                command_operation_id: wire(command_operation_id.as_str()),
                snapshot_operation_id: wire(snapshot_operation_id.as_str()),
            },
        ),
        AdmissionRejectionProjection::PolicyDenied {
            operation_id,
            permission_class,
        } => (
            WireErrorClassDto::PolicyDenied,
            RetryabilityDto::NotRetryable,
            "policy_denied",
            EchoPayloadDto::PolicyDenied {
                operation_id: wire(operation_id.as_str()),
                permission_class: wire(permission_class_text(*permission_class)),
            },
        ),
        AdmissionRejectionProjection::PolicyExpired {
            operation_id,
            policy_snapshot_id,
        } => (
            WireErrorClassDto::PolicyExpired,
            RetryabilityDto::RetryableAfterChange,
            "policy_expired",
            EchoPayloadDto::PolicyExpired {
                operation_id: wire(operation_id.as_str()),
                policy_snapshot_id: wire(policy_snapshot_id.as_str()),
            },
        ),
        AdmissionRejectionProjection::SessionNotFound {
            requested_session_id,
        } => (
            WireErrorClassDto::SessionNotFound,
            RetryabilityDto::RetryableAfterChange,
            "session_not_found",
            EchoPayloadDto::SessionId {
                requested_session_id: wire(requested_session_id.as_str()),
            },
        ),
        AdmissionRejectionProjection::SessionIdMismatch {
            requested_session_id,
            loaded_session_id,
        } => (
            WireErrorClassDto::SessionIdMismatch,
            RetryabilityDto::NotRetryable,
            "session_id_mismatch",
            EchoPayloadDto::SessionMismatch {
                requested_session_id: wire(requested_session_id.as_str()),
                loaded_session_id: wire(loaded_session_id.as_str()),
            },
        ),
        AdmissionRejectionProjection::SessionNotAdmitted {
            requested_session_id,
            observed_at,
        } => (
            WireErrorClassDto::SessionNotAdmitted,
            RetryabilityDto::RetryableAfterChange,
            "session_not_admitted",
            EchoPayloadDto::SessionNotAdmitted {
                requested_session_id: wire(requested_session_id.as_str()),
                observed_at: UnixMillis::new(
                    i64::try_from(observed_at.as_unix_millis()).unwrap_or(i64::MAX),
                ),
            },
        ),
        AdmissionRejectionProjection::CapabilityMissing {
            operation_id,
            actor_kind,
        } => (
            WireErrorClassDto::CapabilityMissing,
            RetryabilityDto::RetryableAfterChange,
            "capability_missing",
            capability_echo(operation_id.as_str(), *actor_kind),
        ),
        AdmissionRejectionProjection::CapabilityDisabled {
            operation_id,
            actor_kind,
        } => (
            WireErrorClassDto::CapabilityDisabled,
            RetryabilityDto::RetryableAfterChange,
            "capability_disabled",
            capability_echo(operation_id.as_str(), *actor_kind),
        ),
        AdmissionRejectionProjection::CapabilityRevoked {
            operation_id,
            actor_kind,
        } => (
            WireErrorClassDto::CapabilityRevoked,
            RetryabilityDto::NotRetryable,
            "capability_revoked",
            capability_echo(operation_id.as_str(), *actor_kind),
        ),
        AdmissionRejectionProjection::InfrastructurePortUnavailable { operation_id, .. } => (
            WireErrorClassDto::InfrastructurePortUnavailable,
            RetryabilityDto::Retryable,
            "infrastructure_port_unavailable",
            operation_echo(operation_id.as_str()),
        ),
        AdmissionRejectionProjection::MalformedCommand { operation_id } => (
            WireErrorClassDto::MalformedCommand,
            RetryabilityDto::RetryableAfterChange,
            "malformed_command",
            operation_id
                .as_ref()
                .map(|value| operation_echo(value.as_str()))
                .unwrap_or(EchoPayloadDto::None),
        ),
    };
    let error = match M10WireErrorDto::try_new(class, retryability, wire(code), echo) {
        Ok(error) => error,
        Err(_) => {
            return ClientErrorDto::InternalInvariant {
                wire_code: wire("m00_projection_invariant"),
            };
        }
    };
    ClientErrorDto::Admission { error }
}

fn wire(value: &str) -> WireText {
    WireText::parse(value).unwrap_or_else(|_| WireText::fallback())
}

fn operation_echo(value: &str) -> EchoPayloadDto {
    EchoPayloadDto::Operation {
        operation_id: wire(value),
    }
}

fn capability_echo(operation_id: &str, actor_kind: ActorKind) -> EchoPayloadDto {
    EchoPayloadDto::Capability {
        operation_id: wire(operation_id),
        actor_kind: wire(match actor_kind {
            ActorKind::Public => "public",
            ActorKind::Authenticated => "authenticated",
        }),
    }
}

fn permission_class_text(value: PermissionClass) -> &'static str {
    match value {
        PermissionClass::PublicRead => "public_read",
        PermissionClass::PublicLinkout => "public_linkout",
        PermissionClass::TenantPrivateRead => "tenant_private_read",
        PermissionClass::TenantPrivateWrite => "tenant_private_write",
    }
}
