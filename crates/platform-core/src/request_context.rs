//! Accepted `platform-request-context/v0` admission kernel.
//!
//! The coordinator is the only constructor for authority-bearing request contexts.  Ports return
//! transaction-current observations; no value returned by a port is authority until this module
//! validates and combines it.  Durable adapters persist the projection DTOs below, never live
//! context/actor values.

use serde::{Deserialize, Deserializer, Serialize, Serializer, de};
use std::error::Error;
use std::fmt;
use std::num::NonZeroU64;
use std::sync::Arc;
use ustc_agent_tool_protocol::Sha256Digest;

use crate::identity::{CommandId, CorrelationId, RequestId, SessionId, TenantId, UserId};
use crate::session::{SessionInstant, SessionSnapshot};

const MAX_IDENTITY_BYTES: usize = 128;
const MAX_PROVENANCE_BYTES: usize = 128;

/// Construction/parsing failure for a request-context value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RequestContextValueErrorKind {
    Empty,
    TooLong,
    InvalidGrammar,
    InvalidDigest,
    ZeroFencingToken,
    ActorSessionMismatch,
    OperationMismatch,
}

/// Checked-value failure with a stable field name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RequestContextValueError {
    field: &'static str,
    kind: RequestContextValueErrorKind,
}

impl RequestContextValueError {
    const fn new(field: &'static str, kind: RequestContextValueErrorKind) -> Self {
        Self { field, kind }
    }

    #[must_use]
    pub const fn field(&self) -> &'static str {
        self.field
    }

    #[must_use]
    pub const fn kind(&self) -> RequestContextValueErrorKind {
        self.kind
    }
}

impl fmt::Display for RequestContextValueError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "invalid {}: {:?}", self.field, self.kind)
    }
}

impl Error for RequestContextValueError {}

fn valid_identity(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_IDENTITY_BYTES
        && value.bytes().all(|byte| {
            byte.is_ascii_alphanumeric() || matches!(byte, b':' | b'_' | b'.' | b'-' | b'/')
        })
}

macro_rules! checked_identity {
    ($name:ident, $field:literal) => {
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
        #[serde(transparent)]
        pub struct $name(String);

        impl $name {
            pub fn parse(value: impl Into<String>) -> Result<Self, RequestContextValueError> {
                let value = value.into();
                let kind = if value.is_empty() {
                    Some(RequestContextValueErrorKind::Empty)
                } else if value.len() > MAX_IDENTITY_BYTES {
                    Some(RequestContextValueErrorKind::TooLong)
                } else if !valid_identity(&value) {
                    Some(RequestContextValueErrorKind::InvalidGrammar)
                } else {
                    None
                };
                match kind {
                    Some(kind) => Err(RequestContextValueError::new($field, kind)),
                    None => Ok(Self(value)),
                }
            }

            #[must_use]
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                let value = String::deserialize(deserializer)?;
                Self::parse(value).map_err(de::Error::custom)
            }
        }
    };
}

checked_identity!(OperationId, "operation_id");
checked_identity!(CausationId, "causation_id");
checked_identity!(IdempotencyKey, "idempotency_key");
checked_identity!(PlatformPolicySnapshotId, "platform_policy_snapshot_id");
checked_identity!(SchemaIdentity, "schema_identity");
checked_identity!(DecoderIdentity, "decoder_identity");
checked_identity!(DispatcherIdentity, "dispatcher_identity");
checked_identity!(AdapterIdentity, "adapter_identity");

/// Exact lowercase SHA-256 hex without a prefix.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(transparent)]
pub struct SchemaDigest(String);

impl SchemaDigest {
    pub fn parse(value: impl Into<String>) -> Result<Self, RequestContextValueError> {
        let value = value.into();
        let valid = value.len() == 64
            && value
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte));
        if valid {
            Ok(Self(value))
        } else {
            Err(RequestContextValueError::new(
                "schema_digest",
                RequestContextValueErrorKind::InvalidDigest,
            ))
        }
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl<'de> Deserialize<'de> for SchemaDigest {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Self::parse(String::deserialize(deserializer)?).map_err(de::Error::custom)
    }
}

/// Canonical digest of the request payload.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(transparent)]
pub struct PayloadDigest(SchemaDigest);

impl PayloadDigest {
    pub fn parse(value: impl Into<String>) -> Result<Self, RequestContextValueError> {
        SchemaDigest::parse(value).map(Self)
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl<'de> Deserialize<'de> for PayloadDigest {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        SchemaDigest::deserialize(deserializer).map(Self)
    }
}

/// M00-owned immutable descriptor identity; M10 chooses canonical digest/version inputs.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DescriptorSnapshotId {
    rendered: String,
    content_digest: SchemaDigest,
    snapshot_version: u64,
}

impl DescriptorSnapshotId {
    pub fn from_canonical_identity(
        content_digest: &SchemaDigest,
        snapshot_version: u64,
    ) -> Result<Self, RequestContextValueError> {
        if snapshot_version == 0 {
            return Err(RequestContextValueError::new(
                "descriptor_snapshot_id",
                RequestContextValueErrorKind::InvalidGrammar,
            ));
        }
        Ok(Self {
            rendered: format!(
                "descriptor:v0:{snapshot_version}:{}",
                content_digest.as_str()
            ),
            content_digest: content_digest.clone(),
            snapshot_version,
        })
    }

    pub fn parse(value: impl Into<String>) -> Result<Self, RequestContextValueError> {
        let value = value.into();
        let mut parts = value.split(':');
        let prefix_ok =
            matches!(parts.next(), Some("descriptor")) && matches!(parts.next(), Some("v0"));
        let version = parts.next().and_then(|part| part.parse::<u64>().ok());
        let digest = parts.next().and_then(|part| SchemaDigest::parse(part).ok());
        if prefix_ok
            && parts.next().is_none()
            && let (Some(version), Some(digest)) = (version.filter(|value| *value > 0), digest)
        {
            return Ok(Self {
                rendered: value,
                content_digest: digest,
                snapshot_version: version,
            });
        }
        Err(RequestContextValueError::new(
            "descriptor_snapshot_id",
            RequestContextValueErrorKind::InvalidGrammar,
        ))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.rendered
    }
    #[must_use]
    pub const fn content_digest(&self) -> &SchemaDigest {
        &self.content_digest
    }
    #[must_use]
    pub const fn snapshot_version(&self) -> u64 {
        self.snapshot_version
    }
}

impl Serialize for DescriptorSnapshotId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for DescriptorSnapshotId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Self::parse(String::deserialize(deserializer)?).map_err(de::Error::custom)
    }
}

/// Closed M00 permission classes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PermissionClass {
    PublicRead,
    PublicLinkout,
    TenantPrivateRead,
    TenantPrivateWrite,
}

/// Closed M00 effect classes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EffectClass {
    Read,
    LinkOut,
    TenantLocalMutation,
}

const fn permission_effect_coherent_v0(permission: PermissionClass, effect: EffectClass) -> bool {
    matches!(
        (permission, effect),
        (PermissionClass::PublicRead, EffectClass::Read)
            | (PermissionClass::PublicLinkout, EffectClass::LinkOut)
            | (PermissionClass::TenantPrivateRead, EffectClass::Read)
            | (
                PermissionClass::TenantPrivateWrite,
                EffectClass::TenantLocalMutation
            )
    )
}

/// Closed actor-kind projection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActorKind {
    Public,
    Authenticated,
}

/// Closed source port kind retained for diagnosis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AdmissionPortKind {
    Clock,
    Descriptor,
    Session,
    Policy,
    Capability,
    Idempotency,
}

/// Checked ordered adapter allowlist.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AdapterAllowlist(Vec<AdapterIdentity>);

impl AdapterAllowlist {
    pub fn try_from_iter(
        values: impl IntoIterator<Item = AdapterIdentity>,
    ) -> Result<Self, RequestContextValueError> {
        let mut values: Vec<_> = values.into_iter().collect();
        values.sort();
        values.dedup();
        if values.is_empty() || values.len() > 64 {
            return Err(RequestContextValueError::new(
                "adapter_allowlist",
                RequestContextValueErrorKind::InvalidGrammar,
            ));
        }
        Ok(Self(values))
    }

    #[must_use]
    pub fn as_slice(&self) -> &[AdapterIdentity] {
        &self.0
    }
}

impl<'de> Deserialize<'de> for AdapterAllowlist {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Self::try_from_iter(Vec::<AdapterIdentity>::deserialize(deserializer)?)
            .map_err(de::Error::custom)
    }
}

/// Immutable request-scoped operation descriptor supplied by M10/B5.
pub trait OperationDescriptorProjection: Send + Sync {
    fn operation_id(&self) -> &OperationId;
    fn schema_identity(&self) -> &SchemaIdentity;
    fn schema_digest(&self) -> &SchemaDigest;
    fn permission_class(&self) -> PermissionClass;
    fn effect_class(&self) -> EffectClass;
    fn decoder_identity(&self) -> &DecoderIdentity;
    fn dispatcher_identity(&self) -> &DispatcherIdentity;
    fn adapter_allowlist(&self) -> &AdapterAllowlist;
    fn snapshot_identity(&self) -> &DescriptorSnapshotId;
}

pub type OperationSnapshot = Arc<dyn OperationDescriptorProjection + Send + Sync>;

/// Nominal public scope; it is not a reserved tenant identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct PublicScope;

/// Untrusted actor reference accepted from M10 ingress.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ActorReference {
    Anonymous { scope: PublicScope },
    Authenticated { session_id: SessionId },
}

impl ActorReference {
    #[must_use]
    pub const fn kind(&self) -> ActorKind {
        match self {
            Self::Anonymous { .. } => ActorKind::Public,
            Self::Authenticated { .. } => ActorKind::Authenticated,
        }
    }

    #[must_use]
    pub const fn session_id(&self) -> Option<&SessionId> {
        match self {
            Self::Anonymous { .. } => None,
            Self::Authenticated { session_id } => Some(session_id),
        }
    }
}

/// Exact identities admitted from one matched session snapshot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdmittedIdentities {
    tenant_id: TenantId,
    user_id: UserId,
    session_id: SessionId,
}

impl AdmittedIdentities {
    fn from_snapshot(snapshot: &SessionSnapshot) -> Self {
        Self {
            tenant_id: snapshot.tenant_id().clone(),
            user_id: snapshot.user_id().clone(),
            session_id: snapshot.session_id().clone(),
        }
    }
    #[must_use]
    pub const fn tenant_id(&self) -> &TenantId {
        &self.tenant_id
    }
    #[must_use]
    pub const fn user_id(&self) -> &UserId {
        &self.user_id
    }
    #[must_use]
    pub const fn session_id(&self) -> &SessionId {
        &self.session_id
    }
}

/// Closed admitted actor sum. Public contains no synthetic identity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum M00AdmittedActor {
    Public,
    Authenticated(AdmittedIdentities),
}

impl M00AdmittedActor {
    #[must_use]
    pub const fn kind(&self) -> ActorKind {
        match self {
            Self::Public => ActorKind::Public,
            Self::Authenticated(_) => ActorKind::Authenticated,
        }
    }
    #[must_use]
    pub const fn identities(&self) -> Option<&AdmittedIdentities> {
        match self {
            Self::Public => None,
            Self::Authenticated(value) => Some(value),
        }
    }
}

/// Checked non-authoritative client provenance.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ClientProvenance {
    build: String,
    target: String,
    protocol: String,
}

impl ClientProvenance {
    pub fn new(
        build: impl Into<String>,
        target: impl Into<String>,
        protocol: impl Into<String>,
    ) -> Result<Self, RequestContextValueError> {
        let build = build.into();
        let target = target.into();
        let protocol = protocol.into();
        if [&build, &target, &protocol].iter().all(|value| {
            !value.is_empty()
                && value.len() <= MAX_PROVENANCE_BYTES
                && value.bytes().all(|byte| !byte.is_ascii_control())
        }) {
            Ok(Self {
                build,
                target,
                protocol,
            })
        } else {
            Err(RequestContextValueError::new(
                "client_provenance",
                RequestContextValueErrorKind::InvalidGrammar,
            ))
        }
    }

    #[must_use]
    pub fn build(&self) -> &str {
        &self.build
    }

    #[must_use]
    pub fn target(&self) -> &str {
        &self.target
    }

    #[must_use]
    pub fn protocol(&self) -> &str {
        &self.protocol
    }
}

/// Checked untrusted command. Authority fields are deliberately absent.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct BuildRequestContextCommand {
    request_id: RequestId,
    operation_id: OperationId,
    actor_reference: ActorReference,
    correlation_id: CorrelationId,
    causation_id: Option<CausationId>,
    idempotency_key: Option<IdempotencyKey>,
    client_provenance: ClientProvenance,
    payload_digest: PayloadDigest,
}

impl BuildRequestContextCommand {
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn new(
        request_id: RequestId,
        operation_id: OperationId,
        actor_reference: ActorReference,
        correlation_id: CorrelationId,
        causation_id: Option<CausationId>,
        idempotency_key: Option<IdempotencyKey>,
        client_provenance: ClientProvenance,
        payload_digest: PayloadDigest,
    ) -> Self {
        Self {
            request_id,
            operation_id,
            actor_reference,
            correlation_id,
            causation_id,
            idempotency_key,
            client_provenance,
            payload_digest,
        }
    }
    #[must_use]
    pub const fn request_id(&self) -> &RequestId {
        &self.request_id
    }
    #[must_use]
    pub const fn operation_id(&self) -> &OperationId {
        &self.operation_id
    }
    #[must_use]
    pub const fn actor_reference(&self) -> &ActorReference {
        &self.actor_reference
    }
    #[must_use]
    pub const fn correlation_id(&self) -> &CorrelationId {
        &self.correlation_id
    }
    #[must_use]
    pub const fn causation_id(&self) -> Option<&CausationId> {
        self.causation_id.as_ref()
    }
    #[must_use]
    pub const fn idempotency_key(&self) -> Option<&IdempotencyKey> {
        self.idempotency_key.as_ref()
    }
    #[must_use]
    pub const fn client_provenance(&self) -> &ClientProvenance {
        &self.client_provenance
    }
    #[must_use]
    pub const fn payload_digest(&self) -> &PayloadDigest {
        &self.payload_digest
    }
}

/// Snapshot of operation facts admitted for this request; no public constructor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdmittedOperation {
    operation_id: OperationId,
    schema_identity: SchemaIdentity,
    schema_digest: SchemaDigest,
    permission_class: PermissionClass,
    effect_class: EffectClass,
    descriptor_snapshot_id: DescriptorSnapshotId,
}

impl AdmittedOperation {
    fn from_snapshot(
        snapshot: &dyn OperationDescriptorProjection,
        permission_class: PermissionClass,
        effect_class: EffectClass,
    ) -> Self {
        Self {
            operation_id: snapshot.operation_id().clone(),
            schema_identity: snapshot.schema_identity().clone(),
            schema_digest: snapshot.schema_digest().clone(),
            permission_class,
            effect_class,
            descriptor_snapshot_id: snapshot.snapshot_identity().clone(),
        }
    }

    #[must_use]
    pub const fn operation_id(&self) -> &OperationId {
        &self.operation_id
    }
    #[must_use]
    pub const fn schema_identity(&self) -> &SchemaIdentity {
        &self.schema_identity
    }
    #[must_use]
    pub const fn schema_digest(&self) -> &SchemaDigest {
        &self.schema_digest
    }
    #[must_use]
    pub const fn permission_class(&self) -> PermissionClass {
        self.permission_class
    }
    #[must_use]
    pub const fn effect_class(&self) -> EffectClass {
        self.effect_class
    }
    #[must_use]
    pub const fn descriptor_snapshot_id(&self) -> &DescriptorSnapshotId {
        &self.descriptor_snapshot_id
    }
}

/// Currentness fact from the configured policy port.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PolicyCurrentnessFact {
    Current,
    Stale,
}

/// Policy resolution returned by the configured port.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PolicyResolution {
    snapshot_id: PlatformPolicySnapshotId,
    currentness: PolicyCurrentnessFact,
}

impl PolicyResolution {
    #[must_use]
    pub const fn new(
        snapshot_id: PlatformPolicySnapshotId,
        currentness: PolicyCurrentnessFact,
    ) -> Self {
        Self {
            snapshot_id,
            currentness,
        }
    }
    #[must_use]
    pub const fn snapshot_id(&self) -> &PlatformPolicySnapshotId {
        &self.snapshot_id
    }
    #[must_use]
    pub const fn currentness(&self) -> PolicyCurrentnessFact {
        self.currentness
    }
}

/// Current capability/grant disposition.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapabilityDisposition {
    Enabled,
    Missing,
    Disabled,
    Revoked,
}

/// Descriptor-port failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DescriptorSnapshotError {
    Absent,
    PortUnavailable,
}

/// Non-descriptor admission-port failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdmissionPortError {
    Unavailable(AdmissionPortKind),
}

/// Opaque current store observation with nonzero fencing authority.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct IdempotencyReservationToken {
    command_id: CommandId,
    reservation_version: u64,
    fencing_token: NonZeroU64,
    deadline: SessionInstant,
}

impl IdempotencyReservationToken {
    pub fn from_store_observation(
        command_id: CommandId,
        reservation_version: u64,
        fencing_token: u64,
        deadline: SessionInstant,
    ) -> Result<Self, RequestContextValueError> {
        let fencing_token = NonZeroU64::new(fencing_token).ok_or_else(|| {
            RequestContextValueError::new(
                "idempotency_reservation_token",
                RequestContextValueErrorKind::ZeroFencingToken,
            )
        })?;
        Ok(Self {
            command_id,
            reservation_version,
            fencing_token,
            deadline,
        })
    }
    #[must_use]
    pub const fn command_id(&self) -> &CommandId {
        &self.command_id
    }
    #[must_use]
    pub const fn reservation_version(&self) -> u64 {
        self.reservation_version
    }
    #[must_use]
    pub const fn fencing_token(&self) -> NonZeroU64 {
        self.fencing_token
    }
    #[must_use]
    pub const fn deadline(&self) -> SessionInstant {
        self.deadline
    }
}

/// Durable admitted actor projection.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum PersistedAdmittedActorDto {
    Public,
    Authenticated {
        tenant_id: TenantId,
        user_id: UserId,
        session_id: SessionId,
    },
}

#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum UncheckedPersistedAdmittedActorDto {
    Public,
    Authenticated {
        tenant_id: TenantId,
        user_id: UserId,
        session_id: SessionId,
    },
}

impl<'de> Deserialize<'de> for PersistedAdmittedActorDto {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(
            match UncheckedPersistedAdmittedActorDto::deserialize(deserializer)? {
                UncheckedPersistedAdmittedActorDto::Public => Self::Public,
                UncheckedPersistedAdmittedActorDto::Authenticated {
                    tenant_id,
                    user_id,
                    session_id,
                } => Self::Authenticated {
                    tenant_id,
                    user_id,
                    session_id,
                },
            },
        )
    }
}

/// Durable frozen prerequisite projection.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PersistedFrozenPrerequisitesDto {
    policy_snapshot_id: PlatformPolicySnapshotId,
    observed_at: SessionInstant,
    session_id: Option<SessionId>,
    admitted_operation_id: OperationId,
}

impl PersistedFrozenPrerequisitesDto {
    #[must_use]
    pub fn from_parts(
        policy_snapshot_id: PlatformPolicySnapshotId,
        observed_at: SessionInstant,
        session_id: Option<SessionId>,
        admitted_operation_id: OperationId,
    ) -> Self {
        Self {
            policy_snapshot_id,
            observed_at,
            session_id,
            admitted_operation_id,
        }
    }
    #[must_use]
    pub const fn policy_snapshot_id(&self) -> &PlatformPolicySnapshotId {
        &self.policy_snapshot_id
    }
    #[must_use]
    pub const fn observed_at(&self) -> SessionInstant {
        self.observed_at
    }
    #[must_use]
    pub const fn session_id(&self) -> Option<&SessionId> {
        self.session_id.as_ref()
    }
    #[must_use]
    pub const fn admitted_operation_id(&self) -> &OperationId {
        &self.admitted_operation_id
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct UncheckedPersistedFrozenPrerequisitesDto {
    policy_snapshot_id: PlatformPolicySnapshotId,
    observed_at: SessionInstant,
    session_id: Option<SessionId>,
    admitted_operation_id: OperationId,
}

impl<'de> Deserialize<'de> for PersistedFrozenPrerequisitesDto {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = UncheckedPersistedFrozenPrerequisitesDto::deserialize(deserializer)?;
        Ok(Self::from_parts(
            value.policy_snapshot_id,
            value.observed_at,
            value.session_id,
            value.admitted_operation_id,
        ))
    }
}

/// Durable admitted disposition projection.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PersistedAdmittedDispositionDto {
    command_id: CommandId,
    correlation_id: CorrelationId,
    descriptor_snapshot_id: DescriptorSnapshotId,
    admitted_actor: PersistedAdmittedActorDto,
    frozen_prerequisites: PersistedFrozenPrerequisitesDto,
}

impl PersistedAdmittedDispositionDto {
    pub fn try_from_parts(
        command_id: CommandId,
        correlation_id: CorrelationId,
        descriptor_snapshot_id: DescriptorSnapshotId,
        admitted_actor: PersistedAdmittedActorDto,
        frozen_prerequisites: PersistedFrozenPrerequisitesDto,
    ) -> Result<Self, RequestContextValueError> {
        let coherent = match (&admitted_actor, frozen_prerequisites.session_id()) {
            (PersistedAdmittedActorDto::Public, None) => true,
            (PersistedAdmittedActorDto::Authenticated { session_id, .. }, Some(frozen)) => {
                session_id == frozen
            }
            _ => false,
        };
        if !coherent {
            return Err(RequestContextValueError::new(
                "persisted_admitted_actor",
                RequestContextValueErrorKind::ActorSessionMismatch,
            ));
        }
        Ok(Self {
            command_id,
            correlation_id,
            descriptor_snapshot_id,
            admitted_actor,
            frozen_prerequisites,
        })
    }
    #[must_use]
    pub const fn command_id(&self) -> &CommandId {
        &self.command_id
    }
    #[must_use]
    pub const fn correlation_id(&self) -> &CorrelationId {
        &self.correlation_id
    }
    #[must_use]
    pub const fn descriptor_snapshot_id(&self) -> &DescriptorSnapshotId {
        &self.descriptor_snapshot_id
    }
    #[must_use]
    pub const fn admitted_actor(&self) -> &PersistedAdmittedActorDto {
        &self.admitted_actor
    }
    #[must_use]
    pub const fn frozen_prerequisites(&self) -> &PersistedFrozenPrerequisitesDto {
        &self.frozen_prerequisites
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct UncheckedPersistedAdmittedDispositionDto {
    command_id: CommandId,
    correlation_id: CorrelationId,
    descriptor_snapshot_id: DescriptorSnapshotId,
    admitted_actor: PersistedAdmittedActorDto,
    frozen_prerequisites: PersistedFrozenPrerequisitesDto,
}

impl<'de> Deserialize<'de> for PersistedAdmittedDispositionDto {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = UncheckedPersistedAdmittedDispositionDto::deserialize(deserializer)?;
        Self::try_from_parts(
            value.command_id,
            value.correlation_id,
            value.descriptor_snapshot_id,
            value.admitted_actor,
            value.frozen_prerequisites,
        )
        .map_err(de::Error::custom)
    }
}

/// Closed wire-independent rejection class.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AdmissionRejectionClass {
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

/// Payload-bearing closed rejection projection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AdmissionRejectionProjection {
    IdempotencyStoreUnavailable {
        operation_id: OperationId,
    },
    ConflictingEnvelope {
        operation_id: OperationId,
        idempotency_key: IdempotencyKey,
    },
    DescriptorSnapshotAbsent {
        operation_id: OperationId,
    },
    DescriptorSnapshotMismatch {
        command_operation_id: OperationId,
        snapshot_operation_id: OperationId,
    },
    PolicyDenied {
        operation_id: OperationId,
        permission_class: PermissionClass,
    },
    PolicyExpired {
        operation_id: OperationId,
        policy_snapshot_id: PlatformPolicySnapshotId,
    },
    SessionNotFound {
        requested_session_id: SessionId,
    },
    SessionIdMismatch {
        requested_session_id: SessionId,
        loaded_session_id: SessionId,
    },
    SessionNotAdmitted {
        requested_session_id: SessionId,
        observed_at: SessionInstant,
    },
    CapabilityMissing {
        operation_id: OperationId,
        actor_kind: ActorKind,
    },
    CapabilityDisabled {
        operation_id: OperationId,
        actor_kind: ActorKind,
    },
    CapabilityRevoked {
        operation_id: OperationId,
        actor_kind: ActorKind,
    },
    InfrastructurePortUnavailable {
        operation_id: OperationId,
        port: AdmissionPortKind,
    },
    MalformedCommand {
        operation_id: Option<OperationId>,
    },
}

impl AdmissionRejectionProjection {
    #[must_use]
    pub const fn class(&self) -> AdmissionRejectionClass {
        match self {
            Self::IdempotencyStoreUnavailable { .. } => {
                AdmissionRejectionClass::IdempotencyStoreUnavailable
            }
            Self::ConflictingEnvelope { .. } => AdmissionRejectionClass::ConflictingEnvelope,
            Self::DescriptorSnapshotAbsent { .. } => {
                AdmissionRejectionClass::DescriptorSnapshotAbsent
            }
            Self::DescriptorSnapshotMismatch { .. } => {
                AdmissionRejectionClass::DescriptorSnapshotMismatch
            }
            Self::PolicyDenied { .. } => AdmissionRejectionClass::PolicyDenied,
            Self::PolicyExpired { .. } => AdmissionRejectionClass::PolicyExpired,
            Self::SessionNotFound { .. } => AdmissionRejectionClass::SessionNotFound,
            Self::SessionIdMismatch { .. } => AdmissionRejectionClass::SessionIdMismatch,
            Self::SessionNotAdmitted { .. } => AdmissionRejectionClass::SessionNotAdmitted,
            Self::CapabilityMissing { .. } => AdmissionRejectionClass::CapabilityMissing,
            Self::CapabilityDisabled { .. } => AdmissionRejectionClass::CapabilityDisabled,
            Self::CapabilityRevoked { .. } => AdmissionRejectionClass::CapabilityRevoked,
            Self::InfrastructurePortUnavailable { .. } => {
                AdmissionRejectionClass::InfrastructurePortUnavailable
            }
            Self::MalformedCommand { .. } => AdmissionRejectionClass::MalformedCommand,
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum RequestContextDiagnosticSource {
    Coordinator,
    Admission,
    Port(AdmissionPortKind),
    RestoredPriorDisposition,
    MalformedCommand,
}

/// Sole M00-owned rejection carrier. Public construction is intentionally absent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RequestContextRejection {
    projection: AdmissionRejectionProjection,
    diagnostic_source: RequestContextDiagnosticSource,
}

impl RequestContextRejection {
    fn new(
        projection: AdmissionRejectionProjection,
        diagnostic_source: RequestContextDiagnosticSource,
    ) -> Self {
        Self {
            projection,
            diagnostic_source,
        }
    }
    #[must_use]
    pub const fn projection(&self) -> &AdmissionRejectionProjection {
        &self.projection
    }
    #[must_use]
    pub const fn class(&self) -> AdmissionRejectionClass {
        self.projection.class()
    }
    #[must_use]
    #[allow(dead_code)]
    pub(crate) const fn diagnostic_source(&self) -> &RequestContextDiagnosticSource {
        &self.diagnostic_source
    }
}

/// Durable rejection projection. It is data only; promotion creates a fresh in-memory projection.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum PersistedAdmissionRejectionDto {
    IdempotencyStoreUnavailable {
        operation_id: OperationId,
    },
    ConflictingEnvelope {
        operation_id: OperationId,
        idempotency_key: IdempotencyKey,
    },
    DescriptorSnapshotAbsent {
        operation_id: OperationId,
    },
    DescriptorSnapshotMismatch {
        command_operation_id: OperationId,
        snapshot_operation_id: OperationId,
    },
    PolicyDenied {
        operation_id: OperationId,
        permission_class: PermissionClass,
    },
    PolicyExpired {
        operation_id: OperationId,
        policy_snapshot_id: PlatformPolicySnapshotId,
    },
    SessionNotFound {
        requested_session_id: SessionId,
    },
    SessionIdMismatch {
        requested_session_id: SessionId,
        loaded_session_id: SessionId,
    },
    SessionNotAdmitted {
        requested_session_id: SessionId,
        observed_at: SessionInstant,
    },
    CapabilityMissing {
        operation_id: OperationId,
        actor_kind: ActorKind,
    },
    CapabilityDisabled {
        operation_id: OperationId,
        actor_kind: ActorKind,
    },
    CapabilityRevoked {
        operation_id: OperationId,
        actor_kind: ActorKind,
    },
    InfrastructurePortUnavailable {
        operation_id: OperationId,
        port: AdmissionPortKind,
    },
    MalformedCommand {
        operation_id: Option<OperationId>,
    },
}

impl PersistedAdmissionRejectionDto {
    #[must_use]
    pub fn from_projection(value: &AdmissionRejectionProjection) -> Self {
        match value {
            AdmissionRejectionProjection::IdempotencyStoreUnavailable { operation_id } => {
                Self::IdempotencyStoreUnavailable {
                    operation_id: operation_id.clone(),
                }
            }
            AdmissionRejectionProjection::ConflictingEnvelope {
                operation_id,
                idempotency_key,
            } => Self::ConflictingEnvelope {
                operation_id: operation_id.clone(),
                idempotency_key: idempotency_key.clone(),
            },
            AdmissionRejectionProjection::DescriptorSnapshotAbsent { operation_id } => {
                Self::DescriptorSnapshotAbsent {
                    operation_id: operation_id.clone(),
                }
            }
            AdmissionRejectionProjection::DescriptorSnapshotMismatch {
                command_operation_id,
                snapshot_operation_id,
            } => Self::DescriptorSnapshotMismatch {
                command_operation_id: command_operation_id.clone(),
                snapshot_operation_id: snapshot_operation_id.clone(),
            },
            AdmissionRejectionProjection::PolicyDenied {
                operation_id,
                permission_class,
            } => Self::PolicyDenied {
                operation_id: operation_id.clone(),
                permission_class: *permission_class,
            },
            AdmissionRejectionProjection::PolicyExpired {
                operation_id,
                policy_snapshot_id,
            } => Self::PolicyExpired {
                operation_id: operation_id.clone(),
                policy_snapshot_id: policy_snapshot_id.clone(),
            },
            AdmissionRejectionProjection::SessionNotFound {
                requested_session_id,
            } => Self::SessionNotFound {
                requested_session_id: requested_session_id.clone(),
            },
            AdmissionRejectionProjection::SessionIdMismatch {
                requested_session_id,
                loaded_session_id,
            } => Self::SessionIdMismatch {
                requested_session_id: requested_session_id.clone(),
                loaded_session_id: loaded_session_id.clone(),
            },
            AdmissionRejectionProjection::SessionNotAdmitted {
                requested_session_id,
                observed_at,
            } => Self::SessionNotAdmitted {
                requested_session_id: requested_session_id.clone(),
                observed_at: *observed_at,
            },
            AdmissionRejectionProjection::CapabilityMissing {
                operation_id,
                actor_kind,
            } => Self::CapabilityMissing {
                operation_id: operation_id.clone(),
                actor_kind: *actor_kind,
            },
            AdmissionRejectionProjection::CapabilityDisabled {
                operation_id,
                actor_kind,
            } => Self::CapabilityDisabled {
                operation_id: operation_id.clone(),
                actor_kind: *actor_kind,
            },
            AdmissionRejectionProjection::CapabilityRevoked {
                operation_id,
                actor_kind,
            } => Self::CapabilityRevoked {
                operation_id: operation_id.clone(),
                actor_kind: *actor_kind,
            },
            AdmissionRejectionProjection::InfrastructurePortUnavailable { operation_id, port } => {
                Self::InfrastructurePortUnavailable {
                    operation_id: operation_id.clone(),
                    port: *port,
                }
            }
            AdmissionRejectionProjection::MalformedCommand { operation_id } => {
                Self::MalformedCommand {
                    operation_id: operation_id.clone(),
                }
            }
        }
    }

    #[must_use]
    pub fn to_projection(&self) -> AdmissionRejectionProjection {
        match self {
            Self::IdempotencyStoreUnavailable { operation_id } => {
                AdmissionRejectionProjection::IdempotencyStoreUnavailable {
                    operation_id: operation_id.clone(),
                }
            }
            Self::ConflictingEnvelope {
                operation_id,
                idempotency_key,
            } => AdmissionRejectionProjection::ConflictingEnvelope {
                operation_id: operation_id.clone(),
                idempotency_key: idempotency_key.clone(),
            },
            Self::DescriptorSnapshotAbsent { operation_id } => {
                AdmissionRejectionProjection::DescriptorSnapshotAbsent {
                    operation_id: operation_id.clone(),
                }
            }
            Self::DescriptorSnapshotMismatch {
                command_operation_id,
                snapshot_operation_id,
            } => AdmissionRejectionProjection::DescriptorSnapshotMismatch {
                command_operation_id: command_operation_id.clone(),
                snapshot_operation_id: snapshot_operation_id.clone(),
            },
            Self::PolicyDenied {
                operation_id,
                permission_class,
            } => AdmissionRejectionProjection::PolicyDenied {
                operation_id: operation_id.clone(),
                permission_class: *permission_class,
            },
            Self::PolicyExpired {
                operation_id,
                policy_snapshot_id,
            } => AdmissionRejectionProjection::PolicyExpired {
                operation_id: operation_id.clone(),
                policy_snapshot_id: policy_snapshot_id.clone(),
            },
            Self::SessionNotFound {
                requested_session_id,
            } => AdmissionRejectionProjection::SessionNotFound {
                requested_session_id: requested_session_id.clone(),
            },
            Self::SessionIdMismatch {
                requested_session_id,
                loaded_session_id,
            } => AdmissionRejectionProjection::SessionIdMismatch {
                requested_session_id: requested_session_id.clone(),
                loaded_session_id: loaded_session_id.clone(),
            },
            Self::SessionNotAdmitted {
                requested_session_id,
                observed_at,
            } => AdmissionRejectionProjection::SessionNotAdmitted {
                requested_session_id: requested_session_id.clone(),
                observed_at: *observed_at,
            },
            Self::CapabilityMissing {
                operation_id,
                actor_kind,
            } => AdmissionRejectionProjection::CapabilityMissing {
                operation_id: operation_id.clone(),
                actor_kind: *actor_kind,
            },
            Self::CapabilityDisabled {
                operation_id,
                actor_kind,
            } => AdmissionRejectionProjection::CapabilityDisabled {
                operation_id: operation_id.clone(),
                actor_kind: *actor_kind,
            },
            Self::CapabilityRevoked {
                operation_id,
                actor_kind,
            } => AdmissionRejectionProjection::CapabilityRevoked {
                operation_id: operation_id.clone(),
                actor_kind: *actor_kind,
            },
            Self::InfrastructurePortUnavailable { operation_id, port } => {
                AdmissionRejectionProjection::InfrastructurePortUnavailable {
                    operation_id: operation_id.clone(),
                    port: *port,
                }
            }
            Self::MalformedCommand { operation_id } => {
                AdmissionRejectionProjection::MalformedCommand {
                    operation_id: operation_id.clone(),
                }
            }
        }
    }
}

#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum UncheckedPersistedAdmissionRejectionDto {
    IdempotencyStoreUnavailable {
        operation_id: OperationId,
    },
    ConflictingEnvelope {
        operation_id: OperationId,
        idempotency_key: IdempotencyKey,
    },
    DescriptorSnapshotAbsent {
        operation_id: OperationId,
    },
    DescriptorSnapshotMismatch {
        command_operation_id: OperationId,
        snapshot_operation_id: OperationId,
    },
    PolicyDenied {
        operation_id: OperationId,
        permission_class: PermissionClass,
    },
    PolicyExpired {
        operation_id: OperationId,
        policy_snapshot_id: PlatformPolicySnapshotId,
    },
    SessionNotFound {
        requested_session_id: SessionId,
    },
    SessionIdMismatch {
        requested_session_id: SessionId,
        loaded_session_id: SessionId,
    },
    SessionNotAdmitted {
        requested_session_id: SessionId,
        observed_at: SessionInstant,
    },
    CapabilityMissing {
        operation_id: OperationId,
        actor_kind: ActorKind,
    },
    CapabilityDisabled {
        operation_id: OperationId,
        actor_kind: ActorKind,
    },
    CapabilityRevoked {
        operation_id: OperationId,
        actor_kind: ActorKind,
    },
    InfrastructurePortUnavailable {
        operation_id: OperationId,
        port: AdmissionPortKind,
    },
    MalformedCommand {
        operation_id: Option<OperationId>,
    },
}

impl From<UncheckedPersistedAdmissionRejectionDto> for PersistedAdmissionRejectionDto {
    fn from(value: UncheckedPersistedAdmissionRejectionDto) -> Self {
        match value {
            UncheckedPersistedAdmissionRejectionDto::IdempotencyStoreUnavailable {
                operation_id,
            } => Self::IdempotencyStoreUnavailable { operation_id },
            UncheckedPersistedAdmissionRejectionDto::ConflictingEnvelope {
                operation_id,
                idempotency_key,
            } => Self::ConflictingEnvelope {
                operation_id,
                idempotency_key,
            },
            UncheckedPersistedAdmissionRejectionDto::DescriptorSnapshotAbsent { operation_id } => {
                Self::DescriptorSnapshotAbsent { operation_id }
            }
            UncheckedPersistedAdmissionRejectionDto::DescriptorSnapshotMismatch {
                command_operation_id,
                snapshot_operation_id,
            } => Self::DescriptorSnapshotMismatch {
                command_operation_id,
                snapshot_operation_id,
            },
            UncheckedPersistedAdmissionRejectionDto::PolicyDenied {
                operation_id,
                permission_class,
            } => Self::PolicyDenied {
                operation_id,
                permission_class,
            },
            UncheckedPersistedAdmissionRejectionDto::PolicyExpired {
                operation_id,
                policy_snapshot_id,
            } => Self::PolicyExpired {
                operation_id,
                policy_snapshot_id,
            },
            UncheckedPersistedAdmissionRejectionDto::SessionNotFound {
                requested_session_id,
            } => Self::SessionNotFound {
                requested_session_id,
            },
            UncheckedPersistedAdmissionRejectionDto::SessionIdMismatch {
                requested_session_id,
                loaded_session_id,
            } => Self::SessionIdMismatch {
                requested_session_id,
                loaded_session_id,
            },
            UncheckedPersistedAdmissionRejectionDto::SessionNotAdmitted {
                requested_session_id,
                observed_at,
            } => Self::SessionNotAdmitted {
                requested_session_id,
                observed_at,
            },
            UncheckedPersistedAdmissionRejectionDto::CapabilityMissing {
                operation_id,
                actor_kind,
            } => Self::CapabilityMissing {
                operation_id,
                actor_kind,
            },
            UncheckedPersistedAdmissionRejectionDto::CapabilityDisabled {
                operation_id,
                actor_kind,
            } => Self::CapabilityDisabled {
                operation_id,
                actor_kind,
            },
            UncheckedPersistedAdmissionRejectionDto::CapabilityRevoked {
                operation_id,
                actor_kind,
            } => Self::CapabilityRevoked {
                operation_id,
                actor_kind,
            },
            UncheckedPersistedAdmissionRejectionDto::InfrastructurePortUnavailable {
                operation_id,
                port,
            } => Self::InfrastructurePortUnavailable { operation_id, port },
            UncheckedPersistedAdmissionRejectionDto::MalformedCommand { operation_id } => {
                Self::MalformedCommand { operation_id }
            }
        }
    }
}

impl<'de> Deserialize<'de> for PersistedAdmissionRejectionDto {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(UncheckedPersistedAdmissionRejectionDto::deserialize(deserializer)?.into())
    }
}

/// Durable prior result restored by B4. The aggregate uses a private validating mirror.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum PersistedPriorDispositionDto {
    Admitted(PersistedAdmittedDispositionDto),
    Rejected(PersistedAdmissionRejectionDto),
}

impl PersistedPriorDispositionDto {
    #[must_use]
    pub const fn admitted(&self) -> Option<&PersistedAdmittedDispositionDto> {
        match self {
            Self::Admitted(value) => Some(value),
            Self::Rejected(_) => None,
        }
    }

    #[must_use]
    pub const fn rejected(&self) -> Option<&PersistedAdmissionRejectionDto> {
        match self {
            Self::Admitted(_) => None,
            Self::Rejected(value) => Some(value),
        }
    }
}

#[derive(Deserialize)]
#[serde(
    tag = "kind",
    content = "value",
    rename_all = "snake_case",
    deny_unknown_fields
)]
enum UncheckedPersistedPriorDispositionDto {
    Admitted(PersistedAdmittedDispositionDto),
    Rejected(PersistedAdmissionRejectionDto),
}

impl<'de> Deserialize<'de> for PersistedPriorDispositionDto {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(
            match UncheckedPersistedPriorDispositionDto::deserialize(deserializer)? {
                UncheckedPersistedPriorDispositionDto::Admitted(value) => Self::Admitted(value),
                UncheckedPersistedPriorDispositionDto::Rejected(value) => Self::Rejected(value),
            },
        )
    }
}

/// Atomic reservation/retrieval result.
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(clippy::large_enum_variant)] // Exact accepted algebra; boxing changes the public payload.
pub enum IdempotencyReservation {
    New(IdempotencyReservationToken),
    Reclaimed(IdempotencyReservationToken),
    PriorIdentical(PersistedPriorDispositionDto),
    InFlight(IdempotencyReservationToken),
}

/// Atomic reservation/finalization failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IdempotencyError {
    StoreUnavailable,
    ConflictingEnvelope { idempotency_key: IdempotencyKey },
}

/// Fenced finalization result.
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(clippy::large_enum_variant)] // Exact accepted algebra; boxing changes the public payload.
pub enum FinalizeIdempotencyOutcome {
    Committed,
    AlreadySame(PersistedPriorDispositionDto),
    LostReservation(IdempotencyReservationToken),
}

/// Opaque admission-envelope hash computed by M00.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnvelopeHash(Sha256Digest);

impl EnvelopeHash {
    #[must_use]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

/// Frozen prerequisites handed to M10/M71.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrozenPrerequisites {
    policy_snapshot_id: PlatformPolicySnapshotId,
    observed_at: SessionInstant,
    session_id: Option<SessionId>,
    admitted_operation_id: OperationId,
}

impl FrozenPrerequisites {
    #[must_use]
    pub const fn policy_snapshot_id(&self) -> &PlatformPolicySnapshotId {
        &self.policy_snapshot_id
    }
    #[must_use]
    pub const fn observed_at(&self) -> SessionInstant {
        self.observed_at
    }
    #[must_use]
    pub const fn session_id(&self) -> Option<&SessionId> {
        self.session_id.as_ref()
    }
    #[must_use]
    pub const fn admitted_operation_id(&self) -> &OperationId {
        &self.admitted_operation_id
    }
}

/// Complete scalar admitted truth. No public constructor and no Serde.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct M00AdmittedDisposition {
    command_id: CommandId,
    correlation_id: CorrelationId,
    descriptor_snapshot_id: DescriptorSnapshotId,
    admitted_actor: M00AdmittedActor,
    frozen_prerequisites: FrozenPrerequisites,
}

impl M00AdmittedDisposition {
    fn try_from_parts(
        command_id: CommandId,
        correlation_id: CorrelationId,
        descriptor_snapshot_id: DescriptorSnapshotId,
        admitted_actor: M00AdmittedActor,
        frozen_prerequisites: FrozenPrerequisites,
    ) -> Result<Self, RequestContextValueError> {
        let coherent = match (&admitted_actor, frozen_prerequisites.session_id()) {
            (M00AdmittedActor::Public, None) => true,
            (M00AdmittedActor::Authenticated(ids), Some(session_id)) => {
                ids.session_id() == session_id
            }
            _ => false,
        };
        if !coherent {
            return Err(RequestContextValueError::new(
                "admitted_actor",
                RequestContextValueErrorKind::ActorSessionMismatch,
            ));
        }
        Ok(Self {
            command_id,
            correlation_id,
            descriptor_snapshot_id,
            admitted_actor,
            frozen_prerequisites,
        })
    }

    fn from_persisted(
        value: PersistedAdmittedDispositionDto,
    ) -> Result<Self, RequestContextValueError> {
        let actor = match value.admitted_actor {
            PersistedAdmittedActorDto::Public => M00AdmittedActor::Public,
            PersistedAdmittedActorDto::Authenticated {
                tenant_id,
                user_id,
                session_id,
            } => M00AdmittedActor::Authenticated(AdmittedIdentities {
                tenant_id,
                user_id,
                session_id,
            }),
        };
        let frozen = FrozenPrerequisites {
            policy_snapshot_id: value.frozen_prerequisites.policy_snapshot_id,
            observed_at: value.frozen_prerequisites.observed_at,
            session_id: value.frozen_prerequisites.session_id,
            admitted_operation_id: value.frozen_prerequisites.admitted_operation_id,
        };
        Self::try_from_parts(
            value.command_id,
            value.correlation_id,
            value.descriptor_snapshot_id,
            actor,
            frozen,
        )
    }

    fn to_persisted_projection(&self) -> PersistedAdmittedDispositionDto {
        let admitted_actor = match &self.admitted_actor {
            M00AdmittedActor::Public => PersistedAdmittedActorDto::Public,
            M00AdmittedActor::Authenticated(ids) => PersistedAdmittedActorDto::Authenticated {
                tenant_id: ids.tenant_id().clone(),
                user_id: ids.user_id().clone(),
                session_id: ids.session_id().clone(),
            },
        };
        let frozen = PersistedFrozenPrerequisitesDto::from_parts(
            self.frozen_prerequisites.policy_snapshot_id.clone(),
            self.frozen_prerequisites.observed_at,
            self.frozen_prerequisites.session_id.clone(),
            self.frozen_prerequisites.admitted_operation_id.clone(),
        );
        PersistedAdmittedDispositionDto::try_from_parts(
            self.command_id.clone(),
            self.correlation_id.clone(),
            self.descriptor_snapshot_id.clone(),
            admitted_actor,
            frozen,
        )
        .expect("in-memory admitted disposition is coherent")
    }

    #[must_use]
    pub const fn command_id(&self) -> &CommandId {
        &self.command_id
    }
    #[must_use]
    pub const fn correlation_id(&self) -> &CorrelationId {
        &self.correlation_id
    }
    #[must_use]
    pub const fn descriptor_snapshot_id(&self) -> &DescriptorSnapshotId {
        &self.descriptor_snapshot_id
    }
    #[must_use]
    pub const fn admitted_actor(&self) -> &M00AdmittedActor {
        &self.admitted_actor
    }
    #[must_use]
    pub const fn frozen_prerequisites(&self) -> &FrozenPrerequisites {
        &self.frozen_prerequisites
    }
}

/// Typed in-flight or lost-reservation result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct M00IncompleteReservation {
    command_id: CommandId,
    retry_not_before: SessionInstant,
}

impl M00IncompleteReservation {
    fn from_token(token: &IdempotencyReservationToken) -> Self {
        Self {
            command_id: token.command_id().clone(),
            retry_not_before: token.deadline(),
        }
    }
    #[must_use]
    pub const fn command_id(&self) -> &CommandId {
        &self.command_id
    }
    #[must_use]
    pub const fn retry_not_before(&self) -> SessionInstant {
        self.retry_not_before
    }
}

/// Sealed request context. The operation snapshot is the same `Arc` checked by admission.
pub struct PlatformRequestContext {
    request_id: RequestId,
    command_id: CommandId,
    correlation_id: CorrelationId,
    causation_id: Option<CausationId>,
    actor: M00AdmittedActor,
    operation: AdmittedOperation,
    policy_reference: PlatformPolicySnapshotId,
    observed_at: SessionInstant,
    client_provenance: ClientProvenance,
    operation_snapshot: OperationSnapshot,
}

impl fmt::Debug for PlatformRequestContext {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("PlatformRequestContext")
            .field("request_id", &self.request_id)
            .field("command_id", &self.command_id)
            .field("correlation_id", &self.correlation_id)
            .field("actor", &self.actor)
            .field("operation", &self.operation)
            .field("policy_reference", &self.policy_reference)
            .field("observed_at", &self.observed_at)
            .finish_non_exhaustive()
    }
}

impl PlatformRequestContext {
    #[must_use]
    pub const fn request_id(&self) -> &RequestId {
        &self.request_id
    }
    #[must_use]
    pub const fn command_id(&self) -> &CommandId {
        &self.command_id
    }
    #[must_use]
    pub const fn correlation_id(&self) -> &CorrelationId {
        &self.correlation_id
    }
    #[must_use]
    pub const fn causation_id(&self) -> Option<&CausationId> {
        self.causation_id.as_ref()
    }
    #[must_use]
    pub const fn actor(&self) -> &M00AdmittedActor {
        &self.actor
    }
    #[must_use]
    pub const fn operation(&self) -> &AdmittedOperation {
        &self.operation
    }
    #[must_use]
    pub const fn policy_reference(&self) -> &PlatformPolicySnapshotId {
        &self.policy_reference
    }
    #[must_use]
    pub const fn observed_at(&self) -> SessionInstant {
        self.observed_at
    }
    #[must_use]
    pub const fn client_provenance(&self) -> &ClientProvenance {
        &self.client_provenance
    }
    #[must_use]
    pub fn operation_snapshot(&self) -> OperationSnapshot {
        Arc::clone(&self.operation_snapshot)
    }
}

/// Private final truth passed to the configured fenced store.
#[derive(Debug, Clone)]
pub enum FinalAdmissionDisposition {
    Admitted(M00AdmittedDisposition),
    Rejected(RequestContextRejection),
}

impl FinalAdmissionDisposition {
    #[must_use]
    pub fn to_persisted_projection(&self) -> PersistedPriorDispositionDto {
        match self {
            Self::Admitted(value) => {
                PersistedPriorDispositionDto::Admitted(value.to_persisted_projection())
            }
            Self::Rejected(value) => PersistedPriorDispositionDto::Rejected(
                PersistedAdmissionRejectionDto::from_projection(value.projection()),
            ),
        }
    }
}

/// Admission ports. Implementations must represent one configured authority path.
pub trait AdmissionPorts {
    fn reserve_or_retrieve_idempotency(
        &mut self,
        key: Option<&IdempotencyKey>,
        envelope_hash: &EnvelopeHash,
    ) -> Result<IdempotencyReservation, IdempotencyError>;
    fn request_scoped_operation(&mut self) -> Result<OperationSnapshot, DescriptorSnapshotError>;
    fn now(&mut self) -> Result<SessionInstant, AdmissionPortError>;
    fn resolve_policy(
        &mut self,
        operation_id: &OperationId,
        observed_at: SessionInstant,
    ) -> Result<PolicyResolution, AdmissionPortError>;
    fn load_session(
        &mut self,
        session_id: &SessionId,
    ) -> Result<Option<SessionSnapshot>, AdmissionPortError>;
    fn check_capability(
        &mut self,
        operation_id: &OperationId,
        actor_kind: ActorKind,
        observed_at: SessionInstant,
    ) -> Result<CapabilityDisposition, AdmissionPortError>;
    fn finalize_idempotency(
        &mut self,
        token: &IdempotencyReservationToken,
        disposition: &FinalAdmissionDisposition,
    ) -> Result<FinalizeIdempotencyOutcome, IdempotencyError>;
}

/// Closed coordinator result consumed exhaustively by M10.
#[derive(Debug)]
#[allow(clippy::large_enum_variant)] // Exact M10-v17 match surface; boxing changes the boundary.
pub enum M00AdmissionResult {
    Admitted {
        context: PlatformRequestContext,
        disposition: M00AdmittedDisposition,
    },
    PriorAdmitted(M00AdmittedDisposition),
    Rejected(RequestContextRejection),
    PriorRejected(RequestContextRejection),
    Incomplete(M00IncompleteReservation),
}

/// Stateless authority coordinator.
pub struct RequestAdmissionCoordinator;

impl RequestAdmissionCoordinator {
    pub fn admit<P: AdmissionPorts>(
        &self,
        command: &BuildRequestContextCommand,
        ports: &mut P,
    ) -> M00AdmissionResult {
        let envelope_hash = envelope_hash(command);
        let reservation = match ports
            .reserve_or_retrieve_idempotency(command.idempotency_key(), &envelope_hash)
        {
            Ok(value) => value,
            Err(IdempotencyError::StoreUnavailable) => {
                return M00AdmissionResult::Rejected(rejection(
                    AdmissionRejectionProjection::IdempotencyStoreUnavailable {
                        operation_id: command.operation_id.clone(),
                    },
                    RequestContextDiagnosticSource::Port(AdmissionPortKind::Idempotency),
                ));
            }
            Err(IdempotencyError::ConflictingEnvelope { idempotency_key }) => {
                return M00AdmissionResult::Rejected(rejection(
                    AdmissionRejectionProjection::ConflictingEnvelope {
                        operation_id: command.operation_id.clone(),
                        idempotency_key,
                    },
                    RequestContextDiagnosticSource::Coordinator,
                ));
            }
        };
        let token = match reservation {
            IdempotencyReservation::New(token) | IdempotencyReservation::Reclaimed(token) => token,
            IdempotencyReservation::PriorIdentical(prior) => return promote_persisted_prior(prior),
            IdempotencyReservation::InFlight(token) => {
                return M00AdmissionResult::Incomplete(M00IncompleteReservation::from_token(
                    &token,
                ));
            }
        };

        let snapshot = match ports.request_scoped_operation() {
            Ok(snapshot) => snapshot,
            Err(DescriptorSnapshotError::Absent) => {
                return finalize_rejection(
                    ports,
                    &token,
                    command.operation_id(),
                    rejection(
                        AdmissionRejectionProjection::DescriptorSnapshotAbsent {
                            operation_id: command.operation_id.clone(),
                        },
                        RequestContextDiagnosticSource::Coordinator,
                    ),
                );
            }
            Err(DescriptorSnapshotError::PortUnavailable) => {
                return finalize_rejection(
                    ports,
                    &token,
                    command.operation_id(),
                    infrastructure_rejection(
                        command.operation_id.clone(),
                        AdmissionPortKind::Descriptor,
                    ),
                );
            }
        };
        if snapshot.operation_id() != command.operation_id() {
            return finalize_rejection(
                ports,
                &token,
                command.operation_id(),
                rejection(
                    AdmissionRejectionProjection::DescriptorSnapshotMismatch {
                        command_operation_id: command.operation_id.clone(),
                        snapshot_operation_id: snapshot.operation_id().clone(),
                    },
                    RequestContextDiagnosticSource::Coordinator,
                ),
            );
        }

        let permission = snapshot.permission_class();
        let effect = snapshot.effect_class();
        if !permission_effect_coherent_v0(permission, effect) {
            return finalize_rejection(
                ports,
                &token,
                command.operation_id(),
                rejection(
                    AdmissionRejectionProjection::MalformedCommand {
                        operation_id: Some(command.operation_id().clone()),
                    },
                    RequestContextDiagnosticSource::Coordinator,
                ),
            );
        }

        let observed_at = match ports.now() {
            Ok(value) => value,
            Err(AdmissionPortError::Unavailable(port)) => {
                return finalize_rejection(
                    ports,
                    &token,
                    command.operation_id(),
                    infrastructure_rejection(command.operation_id.clone(), port),
                );
            }
        };
        let actor_kind = command.actor_reference.kind();
        if actor_kind == ActorKind::Public
            && !matches!(
                permission,
                PermissionClass::PublicRead | PermissionClass::PublicLinkout
            )
        {
            return finalize_rejection(
                ports,
                &token,
                command.operation_id(),
                rejection(
                    AdmissionRejectionProjection::PolicyDenied {
                        operation_id: command.operation_id.clone(),
                        permission_class: permission,
                    },
                    RequestContextDiagnosticSource::Admission,
                ),
            );
        }

        let policy = match ports.resolve_policy(command.operation_id(), observed_at) {
            Ok(value) => value,
            Err(AdmissionPortError::Unavailable(port)) => {
                return finalize_rejection(
                    ports,
                    &token,
                    command.operation_id(),
                    infrastructure_rejection(command.operation_id.clone(), port),
                );
            }
        };
        if policy.currentness() != PolicyCurrentnessFact::Current {
            return finalize_rejection(
                ports,
                &token,
                command.operation_id(),
                rejection(
                    AdmissionRejectionProjection::PolicyExpired {
                        operation_id: command.operation_id.clone(),
                        policy_snapshot_id: policy.snapshot_id().clone(),
                    },
                    RequestContextDiagnosticSource::Admission,
                ),
            );
        }

        let (actor, session_id) = match command.actor_reference() {
            ActorReference::Anonymous { .. } => (M00AdmittedActor::Public, None),
            ActorReference::Authenticated { session_id } => {
                let loaded = match ports.load_session(session_id) {
                    Ok(value) => value,
                    Err(AdmissionPortError::Unavailable(port)) => {
                        return finalize_rejection(
                            ports,
                            &token,
                            command.operation_id(),
                            infrastructure_rejection(command.operation_id.clone(), port),
                        );
                    }
                };
                let Some(loaded) = loaded else {
                    return finalize_rejection(
                        ports,
                        &token,
                        command.operation_id(),
                        rejection(
                            AdmissionRejectionProjection::SessionNotFound {
                                requested_session_id: session_id.clone(),
                            },
                            RequestContextDiagnosticSource::Admission,
                        ),
                    );
                };
                if loaded.session_id() != session_id {
                    return finalize_rejection(
                        ports,
                        &token,
                        command.operation_id(),
                        rejection(
                            AdmissionRejectionProjection::SessionIdMismatch {
                                requested_session_id: session_id.clone(),
                                loaded_session_id: loaded.session_id().clone(),
                            },
                            RequestContextDiagnosticSource::Admission,
                        ),
                    );
                }
                if !loaded.admits_at(observed_at) {
                    return finalize_rejection(
                        ports,
                        &token,
                        command.operation_id(),
                        rejection(
                            AdmissionRejectionProjection::SessionNotAdmitted {
                                requested_session_id: session_id.clone(),
                                observed_at,
                            },
                            RequestContextDiagnosticSource::Admission,
                        ),
                    );
                }
                (
                    M00AdmittedActor::Authenticated(AdmittedIdentities::from_snapshot(&loaded)),
                    Some(session_id.clone()),
                )
            }
        };

        let capability =
            match ports.check_capability(command.operation_id(), actor_kind, observed_at) {
                Ok(value) => value,
                Err(AdmissionPortError::Unavailable(port)) => {
                    return finalize_rejection(
                        ports,
                        &token,
                        command.operation_id(),
                        infrastructure_rejection(command.operation_id.clone(), port),
                    );
                }
            };
        let capability_projection = match capability {
            CapabilityDisposition::Enabled => None,
            CapabilityDisposition::Missing => {
                Some(AdmissionRejectionProjection::CapabilityMissing {
                    operation_id: command.operation_id.clone(),
                    actor_kind,
                })
            }
            CapabilityDisposition::Disabled => {
                Some(AdmissionRejectionProjection::CapabilityDisabled {
                    operation_id: command.operation_id.clone(),
                    actor_kind,
                })
            }
            CapabilityDisposition::Revoked => {
                Some(AdmissionRejectionProjection::CapabilityRevoked {
                    operation_id: command.operation_id.clone(),
                    actor_kind,
                })
            }
        };
        if let Some(projection) = capability_projection {
            return finalize_rejection(
                ports,
                &token,
                command.operation_id(),
                rejection(projection, RequestContextDiagnosticSource::Admission),
            );
        }

        let operation = AdmittedOperation::from_snapshot(snapshot.as_ref(), permission, effect);
        let frozen = FrozenPrerequisites {
            policy_snapshot_id: policy.snapshot_id().clone(),
            observed_at,
            session_id,
            admitted_operation_id: operation.operation_id.clone(),
        };
        let disposition = M00AdmittedDisposition::try_from_parts(
            token.command_id().clone(),
            command.correlation_id.clone(),
            snapshot.snapshot_identity().clone(),
            actor.clone(),
            frozen,
        )
        .expect("coordinator produced coherent admitted disposition");
        let context = PlatformRequestContext {
            request_id: command.request_id.clone(),
            command_id: token.command_id().clone(),
            correlation_id: command.correlation_id.clone(),
            causation_id: command.causation_id.clone(),
            actor,
            operation,
            policy_reference: policy.snapshot_id().clone(),
            observed_at,
            client_provenance: command.client_provenance.clone(),
            operation_snapshot: snapshot,
        };
        let final_disposition = FinalAdmissionDisposition::Admitted(disposition.clone());
        match ports.finalize_idempotency(&token, &final_disposition) {
            Ok(FinalizeIdempotencyOutcome::Committed) => M00AdmissionResult::Admitted {
                context,
                disposition,
            },
            Ok(FinalizeIdempotencyOutcome::AlreadySame(prior)) => promote_persisted_prior(prior),
            Ok(FinalizeIdempotencyOutcome::LostReservation(lost)) => {
                M00AdmissionResult::Incomplete(M00IncompleteReservation::from_token(&lost))
            }
            Err(_) => M00AdmissionResult::Rejected(rejection(
                AdmissionRejectionProjection::IdempotencyStoreUnavailable {
                    operation_id: command.operation_id.clone(),
                },
                RequestContextDiagnosticSource::Port(AdmissionPortKind::Idempotency),
            )),
        }
    }
}

fn promote_persisted_prior(prior: PersistedPriorDispositionDto) -> M00AdmissionResult {
    match prior {
        PersistedPriorDispositionDto::Admitted(value) => {
            match M00AdmittedDisposition::from_persisted(value) {
                Ok(value) => M00AdmissionResult::PriorAdmitted(value),
                Err(_) => M00AdmissionResult::Rejected(rejection(
                    AdmissionRejectionProjection::MalformedCommand { operation_id: None },
                    RequestContextDiagnosticSource::RestoredPriorDisposition,
                )),
            }
        }
        PersistedPriorDispositionDto::Rejected(value) => {
            M00AdmissionResult::PriorRejected(rejection(
                value.to_projection(),
                RequestContextDiagnosticSource::RestoredPriorDisposition,
            ))
        }
    }
}

fn rejection(
    projection: AdmissionRejectionProjection,
    diagnostic: RequestContextDiagnosticSource,
) -> RequestContextRejection {
    RequestContextRejection::new(projection, diagnostic)
}

fn infrastructure_rejection(
    operation_id: OperationId,
    port: AdmissionPortKind,
) -> RequestContextRejection {
    rejection(
        AdmissionRejectionProjection::InfrastructurePortUnavailable { operation_id, port },
        RequestContextDiagnosticSource::Port(port),
    )
}

fn finalize_rejection<P: AdmissionPorts>(
    ports: &mut P,
    token: &IdempotencyReservationToken,
    operation_id: &OperationId,
    rejected: RequestContextRejection,
) -> M00AdmissionResult {
    let final_disposition = FinalAdmissionDisposition::Rejected(rejected.clone());
    match ports.finalize_idempotency(token, &final_disposition) {
        Ok(FinalizeIdempotencyOutcome::Committed) => M00AdmissionResult::Rejected(rejected),
        Ok(FinalizeIdempotencyOutcome::AlreadySame(prior)) => promote_persisted_prior(prior),
        Ok(FinalizeIdempotencyOutcome::LostReservation(lost)) => {
            M00AdmissionResult::Incomplete(M00IncompleteReservation::from_token(&lost))
        }
        Err(_) => M00AdmissionResult::Rejected(rejection(
            AdmissionRejectionProjection::IdempotencyStoreUnavailable {
                operation_id: operation_id.clone(),
            },
            RequestContextDiagnosticSource::Port(AdmissionPortKind::Idempotency),
        )),
    }
}

fn envelope_hash(command: &BuildRequestContextCommand) -> EnvelopeHash {
    let mut bytes = Vec::from(b"platform-request-context/v0/envelope\0".as_slice());
    encode(command.operation_id.as_str(), &mut bytes);
    match command.actor_reference() {
        ActorReference::Anonymous { .. } => bytes.push(0),
        ActorReference::Authenticated { session_id } => {
            bytes.push(1);
            encode(session_id.as_str(), &mut bytes);
        }
    }
    encode(command.payload_digest.as_str(), &mut bytes);
    if let Some(causation_id) = command.causation_id() {
        bytes.push(1);
        encode(causation_id.as_str(), &mut bytes);
    } else {
        bytes.push(0);
    }
    EnvelopeHash(Sha256Digest::from_bytes(&bytes))
}

fn encode(value: &str, output: &mut Vec<u8>) {
    output.extend_from_slice(&(value.len() as u64).to_be_bytes());
    output.extend_from_slice(value.as_bytes());
}

/// Cross-crate callers cannot construct authority facts:
///
/// ```compile_fail
/// use ustc_campus_agent_core::request_context::{AdmissionFacts, OperationId};
/// let _ = AdmissionFacts { operation_id: OperationId::parse("affairs.get").unwrap() };
/// ```
///
/// Cross-crate callers cannot construct a sealed context:
///
/// ```compile_fail
/// use ustc_campus_agent_core::request_context::PlatformRequestContext;
/// let _ = PlatformRequestContext::new();
/// ```
#[derive(Debug)]
pub struct AdmissionFacts {
    _private: (),
}

/// Authority-bearing values cannot be decoded from client JSON:
///
/// ```compile_fail
/// use ustc_campus_agent_core::request_context::PlatformRequestContext;
/// let _: PlatformRequestContext = serde_json::from_str("{}").unwrap();
/// ```
///
/// ```compile_fail
/// use ustc_campus_agent_core::request_context::M00AdmittedActor;
/// let _: M00AdmittedActor = serde_json::from_str(r#"{"kind":"public"}"#).unwrap();
/// ```
pub struct AuthorityDeserializeCompileProof;
