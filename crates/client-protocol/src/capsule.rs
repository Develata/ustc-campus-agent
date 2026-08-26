use serde::{Deserialize, Deserializer, Serialize, de};

use crate::value::{UnixMillis, WireText};

pub const CAPSULE_SCHEMA_VERSION: u8 = 2;

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum AdmittedActorDto {
    Public,
    Authenticated {
        tenant_id: WireText,
        user_id: WireText,
        session_id: WireText,
    },
}

impl std::fmt::Debug for AdmittedActorDto {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Public => formatter
                .debug_struct("AdmittedActorDto")
                .field("kind", &"public")
                .finish(),
            Self::Authenticated { .. } => formatter
                .debug_struct("AdmittedActorDto")
                .field("kind", &"authenticated")
                .field("tenant_id", &"[REDACTED]")
                .field("user_id", &"[REDACTED]")
                .field("session_id", &"[REDACTED]")
                .finish(),
        }
    }
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FrozenPrerequisitesDto {
    pub policy_snapshot_id: WireText,
    pub observed_at: UnixMillis,
    pub session_id: Option<WireText>,
    pub admitted_operation_id: WireText,
}

impl std::fmt::Debug for FrozenPrerequisitesDto {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("FrozenPrerequisitesDto")
            .field("policy_snapshot_id", &"[REDACTED]")
            .field("observed_at", &"[REDACTED]")
            .field("session_id", &"[REDACTED]")
            .field("admitted_operation_id", &"[REDACTED]")
            .finish()
    }
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AffairsGetPayloadDto {
    pub procedure_id: WireText,
    pub as_of: Option<UnixMillis>,
}

impl std::fmt::Debug for AffairsGetPayloadDto {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("AffairsGetPayloadDto")
            .field("procedure_id", &"[REDACTED]")
            .field("as_of", &"[REDACTED]")
            .finish()
    }
}

#[derive(Clone, PartialEq, Eq, Serialize)]
pub struct DispatchCapsuleBodyV2 {
    schema_version: u8,
    command_id: WireText,
    correlation_id: WireText,
    dispatch_identity: WireText,
    admitted_actor: AdmittedActorDto,
    affairs_get: AffairsGetPayloadDto,
    descriptor_snapshot_id: WireText,
    descriptor_content_digest: WireText,
    descriptor_snapshot_version: u64,
    frozen_prerequisites: FrozenPrerequisitesDto,
}

impl std::fmt::Debug for DispatchCapsuleBodyV2 {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("DispatchCapsuleBodyV2")
            .field("schema_version", &"[REDACTED]")
            .field("command_id", &"[REDACTED]")
            .field("correlation_id", &"[REDACTED]")
            .field("dispatch_identity", &"[REDACTED]")
            .field("admitted_actor", &"[REDACTED]")
            .field("affairs_get", &"[REDACTED]")
            .field("descriptor_snapshot_id", &"[REDACTED]")
            .field("descriptor_content_digest", &"[REDACTED]")
            .field("descriptor_snapshot_version", &"[REDACTED]")
            .field("frozen_prerequisites", &"[REDACTED]")
            .finish()
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct UncheckedDispatchCapsuleBodyV2 {
    schema_version: u8,
    command_id: WireText,
    correlation_id: WireText,
    dispatch_identity: WireText,
    admitted_actor: AdmittedActorDto,
    affairs_get: AffairsGetPayloadDto,
    descriptor_snapshot_id: WireText,
    descriptor_content_digest: WireText,
    descriptor_snapshot_version: u64,
    frozen_prerequisites: FrozenPrerequisitesDto,
}

impl DispatchCapsuleBodyV2 {
    #[allow(clippy::too_many_arguments)]
    pub fn try_new(
        command_id: WireText,
        correlation_id: WireText,
        admitted_actor: AdmittedActorDto,
        affairs_get: AffairsGetPayloadDto,
        descriptor_snapshot_id: WireText,
        descriptor_content_digest: WireText,
        descriptor_snapshot_version: u64,
        frozen_prerequisites: FrozenPrerequisitesDto,
    ) -> Result<Self, CapsuleValidationError> {
        if descriptor_snapshot_version == 0 {
            return Err(CapsuleValidationError::DescriptorVersionZero);
        }
        if frozen_prerequisites.admitted_operation_id.as_str() != "affairs.get" {
            return Err(CapsuleValidationError::OperationMismatch);
        }
        match (&admitted_actor, &frozen_prerequisites.session_id) {
            (AdmittedActorDto::Public, None) => {}
            (AdmittedActorDto::Authenticated { session_id, .. }, Some(frozen_session))
                if session_id == frozen_session => {}
            _ => return Err(CapsuleValidationError::ActorSessionMismatch),
        }
        let dispatch_identity = WireText::parse(format!("dispatch:v2:{}", command_id.as_str()))
            .map_err(|_| CapsuleValidationError::DispatchIdentityMismatch)?;
        Ok(Self {
            schema_version: CAPSULE_SCHEMA_VERSION,
            command_id,
            correlation_id,
            dispatch_identity,
            admitted_actor,
            affairs_get,
            descriptor_snapshot_id,
            descriptor_content_digest,
            descriptor_snapshot_version,
            frozen_prerequisites,
        })
    }

    #[must_use]
    pub const fn schema_version(&self) -> u8 {
        self.schema_version
    }
    #[must_use]
    pub fn command_id(&self) -> &WireText {
        &self.command_id
    }
    #[must_use]
    pub fn correlation_id(&self) -> &WireText {
        &self.correlation_id
    }
    #[must_use]
    pub fn dispatch_identity(&self) -> &WireText {
        &self.dispatch_identity
    }
    #[must_use]
    pub fn admitted_actor(&self) -> &AdmittedActorDto {
        &self.admitted_actor
    }
    #[must_use]
    pub fn affairs_get(&self) -> &AffairsGetPayloadDto {
        &self.affairs_get
    }
    #[must_use]
    pub fn descriptor_snapshot_id(&self) -> &WireText {
        &self.descriptor_snapshot_id
    }
    #[must_use]
    pub fn descriptor_content_digest(&self) -> &WireText {
        &self.descriptor_content_digest
    }
    #[must_use]
    pub const fn descriptor_snapshot_version(&self) -> u64 {
        self.descriptor_snapshot_version
    }
    #[must_use]
    pub fn frozen_prerequisites(&self) -> &FrozenPrerequisitesDto {
        &self.frozen_prerequisites
    }
}

impl<'de> Deserialize<'de> for DispatchCapsuleBodyV2 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = UncheckedDispatchCapsuleBodyV2::deserialize(deserializer)?;
        if raw.schema_version != CAPSULE_SCHEMA_VERSION {
            return Err(de::Error::custom(CapsuleValidationError::SchemaVersion));
        }
        let value = Self::try_new(
            raw.command_id,
            raw.correlation_id,
            raw.admitted_actor,
            raw.affairs_get,
            raw.descriptor_snapshot_id,
            raw.descriptor_content_digest,
            raw.descriptor_snapshot_version,
            raw.frozen_prerequisites,
        )
        .map_err(de::Error::custom)?;
        if value.dispatch_identity != raw.dispatch_identity {
            return Err(de::Error::custom(
                CapsuleValidationError::DispatchIdentityMismatch,
            ));
        }
        Ok(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapsuleValidationError {
    SchemaVersion,
    DescriptorVersionZero,
    ActorSessionMismatch,
    OperationMismatch,
    DispatchIdentityMismatch,
}

impl std::fmt::Display for CapsuleValidationError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::SchemaVersion => "capsule schema version mismatch",
            Self::DescriptorVersionZero => "capsule descriptor version is zero",
            Self::ActorSessionMismatch => "capsule actor/session mismatch",
            Self::OperationMismatch => "capsule operation mismatch",
            Self::DispatchIdentityMismatch => "capsule dispatch identity mismatch",
        })
    }
}

impl std::error::Error for CapsuleValidationError {}
