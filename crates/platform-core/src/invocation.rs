//! Pure, deterministic invocation authority for `invocation-resolution/v0`.

use sha2::{Digest as _, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt;

const SCHEMA_DOMAIN: &[u8] = b"tool-input-schema/v0\0";
const ARGUMENT_DOMAIN: &[u8] = b"tool-arguments/v0\0";
const MAX_DEPTH: usize = 8;
const MAX_NODES: usize = 256;
const MAX_OBJECT_MEMBERS: usize = 64;
const MAX_SCHEMA_BYTES: usize = 65_536;
const MAX_ARGUMENT_BYTES: usize = 65_536;

/// A validated lowercase SHA-256 identity.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Sha256Digest(String);

impl Sha256Digest {
    fn from_bytes(bytes: &[u8]) -> Self {
        let digest = Sha256::digest(bytes);
        let mut rendered = String::with_capacity(71);
        rendered.push_str("sha256:");
        for byte in digest {
            use fmt::Write as _;
            let _ = write!(rendered, "{byte:02x}");
        }
        Self(rendered)
    }

    /// Parse an exact lowercase `sha256:<64 hex>` digest.
    pub fn parse(value: impl Into<String>) -> Result<Self, InvalidValue> {
        let value = value.into();
        let valid = value.strip_prefix("sha256:").is_some_and(|hex| {
            hex.len() == 64
                && hex
                    .bytes()
                    .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        });
        if valid {
            Ok(Self(value))
        } else {
            Err(InvalidValue::Digest)
        }
    }

    /// Canonical string representation.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Validation failure for an owned canonical value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InvalidValue {
    /// Digest spelling was not canonical.
    Digest,
    /// Identity was empty, too long, or contained whitespace/control characters.
    Identity,
    /// A model-visible/property name did not match the v0 grammar.
    Name,
}

impl fmt::Display for InvalidValue {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "invalid canonical value: {self:?}")
    }
}

impl Error for InvalidValue {}

/// Source-order-preserving input-schema node constructed by a future loader.
#[derive(Debug, Clone, PartialEq)]
pub enum UnvalidatedSchemaNodeV0 {
    /// Closed object with ordered declarations.
    Object {
        /// Property declarations in source order; duplicates remain observable.
        properties: Vec<(String, UnvalidatedSchemaNodeV0)>,
        /// Required names in source order; duplicates remain observable.
        required: Vec<String>,
    },
    /// String, optionally constrained to an exact enum.
    String { enum_values: Option<Vec<String>> },
    /// Signed integral value.
    Integer,
    /// Finite binary64 value.
    Number,
    /// Boolean value.
    Boolean,
    /// Homogeneous array.
    Array { items: Box<UnvalidatedSchemaNodeV0> },
}

/// Loader-produced schema with an exact dialect identity.
#[derive(Debug, Clone, PartialEq)]
pub struct UnvalidatedToolInputSchemaV0 {
    /// Exact dialect string.
    pub dialect: String,
    /// Root schema node.
    pub root: UnvalidatedSchemaNodeV0,
}

/// Canonically ordered, bounded schema node.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidatedSchemaNodeV0 {
    /// Closed object.
    Object {
        /// Properties in bytewise UTF-8 name order.
        properties: BTreeMap<String, ValidatedSchemaNodeV0>,
        /// Required names in bytewise UTF-8 order.
        required: BTreeSet<String>,
    },
    /// Exact string enum, if present.
    String {
        enum_values: Option<BTreeSet<String>>,
    },
    /// Signed integral value.
    Integer,
    /// Finite binary64 value.
    Number,
    /// Boolean value.
    Boolean,
    /// Homogeneous array.
    Array { items: Box<ValidatedSchemaNodeV0> },
}

/// Typed schema-construction failures.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SchemaConstructionError {
    /// Dialect is not `tool-input-schema/v0`.
    SchemaDialectUnsupported,
    /// Structure, names, duplicates, or required subset is malformed.
    SchemaMalformed,
    /// A depth/count/string/canonical-byte limit was exceeded.
    SchemaLimitExceeded,
}

impl fmt::Display for SchemaConstructionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "schema construction failed: {self:?}")
    }
}

impl Error for SchemaConstructionError {}

/// Validated immutable v0 tool-input schema.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatedToolInputSchemaV0 {
    root: ValidatedSchemaNodeV0,
    canonical_bytes: Vec<u8>,
    digest: Sha256Digest,
}

impl ValidatedToolInputSchemaV0 {
    /// Canonical bytes including the v0 domain separator.
    #[must_use]
    pub fn canonical_bytes(&self) -> &[u8] {
        &self.canonical_bytes
    }

    /// Canonical input-schema digest.
    #[must_use]
    pub const fn digest(&self) -> &Sha256Digest {
        &self.digest
    }

    /// Validated root node.
    #[must_use]
    pub const fn root(&self) -> &ValidatedSchemaNodeV0 {
        &self.root
    }
}

impl TryFrom<UnvalidatedToolInputSchemaV0> for ValidatedToolInputSchemaV0 {
    type Error = SchemaConstructionError;

    fn try_from(value: UnvalidatedToolInputSchemaV0) -> Result<Self, Self::Error> {
        if value.dialect != "tool-input-schema/v0" {
            return Err(SchemaConstructionError::SchemaDialectUnsupported);
        }
        if !matches!(value.root, UnvalidatedSchemaNodeV0::Object { .. }) {
            return Err(SchemaConstructionError::SchemaMalformed);
        }
        let mut nodes = 0;
        let root = validate_schema_node(value.root, 1, &mut nodes)?;
        let mut canonical_bytes = SCHEMA_DOMAIN.to_vec();
        encode_schema_node(&root, &mut canonical_bytes);
        if canonical_bytes.len() > MAX_SCHEMA_BYTES {
            return Err(SchemaConstructionError::SchemaLimitExceeded);
        }
        let digest = Sha256Digest::from_bytes(&canonical_bytes);
        Ok(Self {
            root,
            canonical_bytes,
            digest,
        })
    }
}

fn validate_schema_node(
    node: UnvalidatedSchemaNodeV0,
    depth: usize,
    nodes: &mut usize,
) -> Result<ValidatedSchemaNodeV0, SchemaConstructionError> {
    if depth > MAX_DEPTH {
        return Err(SchemaConstructionError::SchemaLimitExceeded);
    }
    *nodes += 1;
    if *nodes > MAX_NODES {
        return Err(SchemaConstructionError::SchemaLimitExceeded);
    }
    match node {
        UnvalidatedSchemaNodeV0::Object {
            properties,
            required,
        } => {
            if properties.len() > MAX_OBJECT_MEMBERS {
                return Err(SchemaConstructionError::SchemaLimitExceeded);
            }
            let mut validated = BTreeMap::new();
            for (name, child) in properties {
                if !is_valid_name(&name) || validated.contains_key(&name) {
                    return Err(SchemaConstructionError::SchemaMalformed);
                }
                let child = validate_schema_node(child, depth + 1, nodes)?;
                validated.insert(name, child);
            }
            let mut required_set = BTreeSet::new();
            for name in required {
                if !is_valid_name(&name)
                    || !validated.contains_key(&name)
                    || !required_set.insert(name)
                {
                    return Err(SchemaConstructionError::SchemaMalformed);
                }
            }
            Ok(ValidatedSchemaNodeV0::Object {
                properties: validated,
                required: required_set,
            })
        }
        UnvalidatedSchemaNodeV0::String { enum_values } => {
            let enum_values = match enum_values {
                None => None,
                Some(values) => {
                    if values.is_empty() || values.len() > 64 {
                        return Err(SchemaConstructionError::SchemaLimitExceeded);
                    }
                    let mut unique = BTreeSet::new();
                    for value in values {
                        if value.is_empty() || value.len() > 256 {
                            return Err(SchemaConstructionError::SchemaLimitExceeded);
                        }
                        if !unique.insert(value) {
                            return Err(SchemaConstructionError::SchemaMalformed);
                        }
                    }
                    Some(unique)
                }
            };
            Ok(ValidatedSchemaNodeV0::String { enum_values })
        }
        UnvalidatedSchemaNodeV0::Integer => Ok(ValidatedSchemaNodeV0::Integer),
        UnvalidatedSchemaNodeV0::Number => Ok(ValidatedSchemaNodeV0::Number),
        UnvalidatedSchemaNodeV0::Boolean => Ok(ValidatedSchemaNodeV0::Boolean),
        UnvalidatedSchemaNodeV0::Array { items } => Ok(ValidatedSchemaNodeV0::Array {
            items: Box::new(validate_schema_node(*items, depth + 1, nodes)?),
        }),
    }
}

fn encode_schema_node(node: &ValidatedSchemaNodeV0, output: &mut Vec<u8>) {
    match node {
        ValidatedSchemaNodeV0::Object {
            properties,
            required,
        } => {
            output.push(0x01);
            encode_count(properties.len(), output);
            for (name, child) in properties {
                encode_string(name, output);
                encode_schema_node(child, output);
            }
            encode_count(required.len(), output);
            for name in required {
                encode_string(name, output);
            }
        }
        ValidatedSchemaNodeV0::String { enum_values } => {
            output.push(0x02);
            match enum_values {
                None => output.push(0),
                Some(values) => {
                    output.push(1);
                    encode_count(values.len(), output);
                    for value in values {
                        encode_string(value, output);
                    }
                }
            }
        }
        ValidatedSchemaNodeV0::Integer => output.push(0x03),
        ValidatedSchemaNodeV0::Number => output.push(0x04),
        ValidatedSchemaNodeV0::Boolean => output.push(0x05),
        ValidatedSchemaNodeV0::Array { items } => {
            output.push(0x06);
            encode_schema_node(items, output);
        }
    }
}

fn encode_count(count: usize, output: &mut Vec<u8>) {
    output.extend_from_slice(&(count as u64).to_be_bytes());
}

fn encode_string(value: &str, output: &mut Vec<u8>) {
    encode_count(value.len(), output);
    output.extend_from_slice(value.as_bytes());
}

fn is_valid_name(value: &str) -> bool {
    if value.is_empty() || value.len() > 64 {
        return false;
    }
    let mut bytes = value.bytes();
    let Some(first) = bytes.next() else {
        return false;
    };
    (first.is_ascii_alphabetic() || first == b'_')
        && bytes.all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'.' | b'-'))
}

/// Source-order-preserving parsed argument value. Numeric variants retain their token text.
#[derive(Debug, Clone, PartialEq)]
pub enum UnvalidatedArgumentValueV0 {
    Null,
    Boolean(bool),
    Integer(String),
    Number(String),
    String(String),
    Array(Vec<UnvalidatedArgumentValueV0>),
    Object(Vec<(String, UnvalidatedArgumentValueV0)>),
}

/// Canonical argument tree; finite numbers are represented by normalized binary64 bits.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CanonicalArgumentNodeV0 {
    Null,
    Boolean(bool),
    Integer(i64),
    Number(u64),
    String(String),
    Array(Vec<CanonicalArgumentNodeV0>),
    Object(BTreeMap<String, CanonicalArgumentNodeV0>),
}

/// Typed canonical-argument construction failures.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArgumentConstructionError {
    ArgumentDuplicateKey,
    ArgumentInvalidName,
    ArgumentNumberOutOfRange,
    ArgumentLimitExceeded,
}

impl fmt::Display for ArgumentConstructionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "argument construction failed: {self:?}")
    }
}

impl Error for ArgumentConstructionError {}

/// Bounded canonical argument value and its exact digest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalArgumentValueV0 {
    root: CanonicalArgumentNodeV0,
    canonical_bytes: Vec<u8>,
    digest: Sha256Digest,
}

impl CanonicalArgumentValueV0 {
    #[must_use]
    pub fn canonical_bytes(&self) -> &[u8] {
        &self.canonical_bytes
    }

    #[must_use]
    pub const fn digest(&self) -> &Sha256Digest {
        &self.digest
    }

    #[must_use]
    pub const fn root(&self) -> &CanonicalArgumentNodeV0 {
        &self.root
    }
}

impl TryFrom<UnvalidatedArgumentValueV0> for CanonicalArgumentValueV0 {
    type Error = ArgumentConstructionError;

    fn try_from(value: UnvalidatedArgumentValueV0) -> Result<Self, Self::Error> {
        let mut nodes = 0;
        let root = validate_argument_node(value, 1, &mut nodes)?;
        let mut canonical_bytes = ARGUMENT_DOMAIN.to_vec();
        encode_argument_node(&root, &mut canonical_bytes);
        if canonical_bytes.len() > MAX_ARGUMENT_BYTES {
            return Err(ArgumentConstructionError::ArgumentLimitExceeded);
        }
        let digest = Sha256Digest::from_bytes(&canonical_bytes);
        Ok(Self {
            root,
            canonical_bytes,
            digest,
        })
    }
}

fn validate_argument_node(
    node: UnvalidatedArgumentValueV0,
    depth: usize,
    nodes: &mut usize,
) -> Result<CanonicalArgumentNodeV0, ArgumentConstructionError> {
    if depth > MAX_DEPTH {
        return Err(ArgumentConstructionError::ArgumentLimitExceeded);
    }
    *nodes += 1;
    if *nodes > MAX_NODES {
        return Err(ArgumentConstructionError::ArgumentLimitExceeded);
    }
    match node {
        UnvalidatedArgumentValueV0::Null => Ok(CanonicalArgumentNodeV0::Null),
        UnvalidatedArgumentValueV0::Boolean(value) => Ok(CanonicalArgumentNodeV0::Boolean(value)),
        UnvalidatedArgumentValueV0::Integer(token) => token
            .parse::<i64>()
            .map(CanonicalArgumentNodeV0::Integer)
            .map_err(|_| ArgumentConstructionError::ArgumentNumberOutOfRange),
        UnvalidatedArgumentValueV0::Number(token) => {
            let number = token
                .parse::<f64>()
                .map_err(|_| ArgumentConstructionError::ArgumentNumberOutOfRange)?;
            if !number.is_finite() {
                return Err(ArgumentConstructionError::ArgumentNumberOutOfRange);
            }
            let bits = if number == 0.0 { 0 } else { number.to_bits() };
            Ok(CanonicalArgumentNodeV0::Number(bits))
        }
        UnvalidatedArgumentValueV0::String(value) => {
            if value.len() > 4_096 {
                Err(ArgumentConstructionError::ArgumentLimitExceeded)
            } else {
                Ok(CanonicalArgumentNodeV0::String(value))
            }
        }
        UnvalidatedArgumentValueV0::Array(values) => {
            if values.len() > 256 {
                return Err(ArgumentConstructionError::ArgumentLimitExceeded);
            }
            values
                .into_iter()
                .map(|value| validate_argument_node(value, depth + 1, nodes))
                .collect::<Result<Vec<_>, _>>()
                .map(CanonicalArgumentNodeV0::Array)
        }
        UnvalidatedArgumentValueV0::Object(members) => {
            if members.len() > MAX_OBJECT_MEMBERS {
                return Err(ArgumentConstructionError::ArgumentLimitExceeded);
            }
            let mut object = BTreeMap::new();
            for (name, value) in members {
                if !is_valid_name(&name) {
                    return Err(ArgumentConstructionError::ArgumentInvalidName);
                }
                if object.contains_key(&name) {
                    return Err(ArgumentConstructionError::ArgumentDuplicateKey);
                }
                let value = validate_argument_node(value, depth + 1, nodes)?;
                object.insert(name, value);
            }
            Ok(CanonicalArgumentNodeV0::Object(object))
        }
    }
}

fn encode_argument_node(node: &CanonicalArgumentNodeV0, output: &mut Vec<u8>) {
    match node {
        CanonicalArgumentNodeV0::Null => output.push(0x00),
        CanonicalArgumentNodeV0::Boolean(value) => {
            output.push(0x01);
            output.push(u8::from(*value));
        }
        CanonicalArgumentNodeV0::Integer(value) => {
            output.push(0x02);
            output.extend_from_slice(&value.to_be_bytes());
        }
        CanonicalArgumentNodeV0::Number(bits) => {
            output.push(0x03);
            output.extend_from_slice(&bits.to_be_bytes());
        }
        CanonicalArgumentNodeV0::String(value) => {
            output.push(0x04);
            encode_string(value, output);
        }
        CanonicalArgumentNodeV0::Array(values) => {
            output.push(0x05);
            encode_count(values.len(), output);
            for value in values {
                encode_argument_node(value, output);
            }
        }
        CanonicalArgumentNodeV0::Object(members) => {
            output.push(0x06);
            encode_count(members.len(), output);
            for (name, value) in members {
                encode_string(name, output);
                encode_argument_node(value, output);
            }
        }
    }
}

macro_rules! authority_id {
    ($name:ident) => {
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(String);
        impl $name {
            pub fn parse(value: impl Into<String>) -> Result<Self, InvalidValue> {
                let value = value.into();
                if is_valid_identity(&value) {
                    Ok(Self(value))
                } else {
                    Err(InvalidValue::Identity)
                }
            }
            #[must_use]
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }
    };
}

authority_id!(TenantId);
authority_id!(UserId);
authority_id!(RunId);
authority_id!(TurnId);
authority_id!(InstallationId);
authority_id!(PackageId);
authority_id!(ComponentId);
authority_id!(ComponentVersion);
authority_id!(ExecutionIdentity);
authority_id!(ToolId);
authority_id!(CapabilityId);
authority_id!(ObjectScope);
authority_id!(CatalogRevision);
authority_id!(InstallationRevision);
authority_id!(GrantSnapshotId);
authority_id!(GrantVersion);
authority_id!(PolicySnapshotId);
authority_id!(PolicyRevision);
authority_id!(SourcePolicyId);
authority_id!(ProviderToolCallId);

fn is_valid_identity(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 256
        && value.chars().all(|c| !c.is_control() && !c.is_whitespace())
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PackageVersion(semver::Version);

impl PackageVersion {
    pub fn parse(value: &str) -> Result<Self, InvalidValue> {
        let version = semver::Version::parse(value).map_err(|_| InvalidValue::Identity)?;
        if version.to_string() == value {
            Ok(Self(version))
        } else {
            Err(InvalidValue::Identity)
        }
    }
    #[must_use]
    pub fn as_str(&self) -> String {
        self.0.to_string()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComponentKind {
    SkillComponent,
    DeclarativeResourcePack,
    McpServerComponent,
    NativeRustComponent,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallationState {
    Enabled,
    Disabled,
    Revoked,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GrantState {
    Active,
    Stale,
    Expired,
    Revoked,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapabilityClass {
    PublicRead,
    PublicLinkout,
    TenantPrivateRead,
    TenantPrivateWrite,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfirmationPolicy {
    Allow,
    Ask,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourcePolicyIdentity {
    pub id: SourcePolicyId,
    pub digest: Sha256Digest,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CatalogToolDefinition {
    pub id: ToolId,
    pub model_visible_name: String,
    pub description: String,
    pub capability_id: CapabilityId,
    pub input_schema: Option<ValidatedToolInputSchemaV0>,
    pub claimed_input_schema_digest: Sha256Digest,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CatalogComponentRevision {
    pub id: ComponentId,
    pub kind: ComponentKind,
    pub version: ComponentVersion,
    pub digest: Sha256Digest,
    pub execution_identity: ExecutionIdentity,
    pub declared_capabilities: BTreeSet<CapabilityId>,
    pub tool: Option<CatalogToolDefinition>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CatalogPackageRevision {
    pub catalog_revision: CatalogRevision,
    pub package_id: PackageId,
    pub package_version: PackageVersion,
    pub package_digest: Sha256Digest,
    pub runnable: bool,
    pub revoked: bool,
    pub capability_manifest_digest: Sha256Digest,
    pub source_policy: Option<SourcePolicyIdentity>,
    pub component: Option<CatalogComponentRevision>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstalledComponentIdentity {
    pub id: ComponentId,
    pub version: ComponentVersion,
    pub digest: Sha256Digest,
    pub execution_identity: ExecutionIdentity,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluginInstallationSnapshot {
    pub id: InstallationId,
    pub tenant_id: TenantId,
    pub user_id: UserId,
    pub package_id: PackageId,
    pub package_version: PackageVersion,
    pub package_digest: Sha256Digest,
    pub component: InstalledComponentIdentity,
    pub state: InstallationState,
    pub revision: InstallationRevision,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapabilityGrantSnapshot {
    pub snapshot_id: GrantSnapshotId,
    pub version: GrantVersion,
    pub tenant_id: TenantId,
    pub user_id: UserId,
    pub installation_id: InstallationId,
    pub capability_id: CapabilityId,
    pub object_scope: ObjectScope,
    pub confirmation_policy: ConfirmationPolicy,
    pub capability_manifest_digest: Sha256Digest,
    pub state: GrantState,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvocationPolicySnapshot {
    pub snapshot_id: PolicySnapshotId,
    pub revision: PolicyRevision,
    pub capability_id: CapabilityId,
    pub capability_class: Option<CapabilityClass>,
    pub admitted_execution_identity: Option<ExecutionIdentity>,
    pub admitted_source_policy: Option<SourcePolicyIdentity>,
    pub emergency_blocked: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct InvocationTarget {
    pub installation_id: InstallationId,
    pub package_id: PackageId,
    pub package_version: PackageVersion,
    pub component_id: ComponentId,
    pub tool_id: ToolId,
    pub capability_id: CapabilityId,
    pub object_scope: ObjectScope,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvocationAuthorityCandidate {
    pub target: InvocationTarget,
    pub catalog: Option<CatalogPackageRevision>,
    pub installation: Option<PluginInstallationSnapshot>,
    pub grant: Option<CapabilityGrantSnapshot>,
    pub policy: InvocationPolicySnapshot,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolProjectionRequest {
    pub tenant_id: TenantId,
    pub user_id: UserId,
    pub run_id: RunId,
    pub turn_id: TurnId,
    pub activation_allowlist: Option<BTreeSet<ToolId>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedInvocation {
    pub tenant_id: TenantId,
    pub user_id: UserId,
    pub installation_id: InstallationId,
    pub installation_revision: InstallationRevision,
    pub package_id: PackageId,
    pub package_version: PackageVersion,
    pub package_digest: Sha256Digest,
    pub catalog_revision: CatalogRevision,
    pub component_id: ComponentId,
    pub component_version: ComponentVersion,
    pub component_digest: Sha256Digest,
    pub component_kind: ComponentKind,
    pub execution_identity: ExecutionIdentity,
    pub tool_id: ToolId,
    pub model_visible_name: String,
    pub dispatch_key: String,
    pub description: String,
    pub provider_tool_definition_digest: Sha256Digest,
    pub capability_id: CapabilityId,
    pub capability_manifest_digest: Sha256Digest,
    pub grant_snapshot_id: GrantSnapshotId,
    pub grant_version: GrantVersion,
    pub object_scope: ObjectScope,
    pub confirmation_policy: ConfirmationPolicy,
    pub source_policy: SourcePolicyIdentity,
    pub input_schema: ValidatedToolInputSchemaV0,
    pub policy_snapshot_id: PolicySnapshotId,
    pub policy_revision: PolicyRevision,
    pub projection_authority_entry_digest: Sha256Digest,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolProjectionSnapshot {
    pub schema_version: &'static str,
    pub run_id: RunId,
    pub turn_id: TurnId,
    pub snapshot_id: String,
    pub entries: Vec<ResolvedInvocation>,
    pub tool_schema_set_digest: Sha256Digest,
    pub projection_authority_set_digest: Sha256Digest,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectionResolutionError {
    InvalidRequest,
    InvalidAuthoritySnapshot,
    EmergencyBlocked,
    AuthorityConflict,
    TenantOrUserScopeMismatch,
    PackageMissing,
    PackageNotRunnable,
    PackageVersionMismatch,
    PackageDigestMismatch,
    CatalogRevoked,
    InstallationMissing,
    InstallationDisabled,
    InstallationRevoked,
    InstallationRevisionMismatch,
    ComponentMissing,
    ComponentIdentityMismatch,
    ExecutionIdentityUnknown,
    ExecutionIdentityMismatch,
    ToolMissing,
    ToolIdentityMismatch,
    CapabilityUnknown,
    CapabilityNotDeclared,
    CapabilityManifestMismatch,
    CapabilityNotGranted,
    GrantStale,
    GrantExpired,
    GrantRevoked,
    GrantVersionMismatch,
    GrantScopeMismatch,
    SourcePolicyMissing,
    SourcePolicyMismatch,
    SchemaMissing,
    SchemaDigestMismatch,
    ToolNameCollision,
}

impl fmt::Display for ProjectionResolutionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "projection denied: {self:?}")
    }
}
impl Error for ProjectionResolutionError {}

pub struct InvocationResolver;

impl InvocationResolver {
    pub fn resolve_projection(
        request: ToolProjectionRequest,
        mut candidates: Vec<InvocationAuthorityCandidate>,
    ) -> Result<ToolProjectionSnapshot, ProjectionResolutionError> {
        if candidates.is_empty() {
            return Err(ProjectionResolutionError::InvalidRequest);
        }
        candidates.sort_by(|left, right| left.target.cmp(&right.target));
        if candidates
            .windows(2)
            .any(|pair| pair[0].target == pair[1].target)
        {
            return Err(ProjectionResolutionError::InvalidAuthoritySnapshot);
        }
        if let Some(allowlist) = &request.activation_allowlist {
            candidates.retain(|candidate| allowlist.contains(&candidate.target.tool_id));
            if candidates.is_empty() {
                return Err(ProjectionResolutionError::InvalidRequest);
            }
        }
        if candidates
            .iter()
            .any(|candidate| candidate.policy.emergency_blocked)
        {
            return Err(ProjectionResolutionError::EmergencyBlocked);
        }
        let authority_anchor = &candidates[0];
        if candidates.iter().skip(1).any(|candidate| {
            candidate.target.installation_id != authority_anchor.target.installation_id
                || candidate.target.package_id != authority_anchor.target.package_id
                || candidate.target.package_version != authority_anchor.target.package_version
                || candidate.target.component_id != authority_anchor.target.component_id
                || candidate.grant.as_ref().map(|grant| &grant.snapshot_id)
                    != authority_anchor
                        .grant
                        .as_ref()
                        .map(|grant| &grant.snapshot_id)
        }) {
            return Err(ProjectionResolutionError::AuthorityConflict);
        }
        let mut entries = Vec::with_capacity(candidates.len());
        for candidate in candidates {
            entries.push(resolve_candidate(&request, candidate)?);
        }
        let first = &entries[0];
        if entries.iter().any(|entry| {
            entry.tenant_id != first.tenant_id
                || entry.user_id != first.user_id
                || entry.installation_id != first.installation_id
                || entry.package_id != first.package_id
                || entry.package_version != first.package_version
                || entry.component_id != first.component_id
                || entry.grant_snapshot_id != first.grant_snapshot_id
        }) {
            return Err(ProjectionResolutionError::AuthorityConflict);
        }
        let mut names = BTreeSet::new();
        if entries
            .iter()
            .any(|entry| !names.insert(entry.model_visible_name.clone()))
        {
            return Err(ProjectionResolutionError::ToolNameCollision);
        }
        let tool_schema_set_digest = digest_strings(
            b"tool-projection/v0\0",
            entries.iter().flat_map(|entry| {
                [
                    entry.dispatch_key.as_str(),
                    entry.provider_tool_definition_digest.as_str(),
                ]
            }),
        );
        let projection_authority_set_digest = digest_strings(
            b"projection-authority-set/v0\0",
            entries
                .iter()
                .map(|entry| entry.projection_authority_entry_digest.as_str()),
        );
        let snapshot_digest = digest_strings(
            b"tool-projection-snapshot/v0\0",
            [
                request.run_id.as_str(),
                request.turn_id.as_str(),
                tool_schema_set_digest.as_str(),
                projection_authority_set_digest.as_str(),
            ],
        );
        Ok(ToolProjectionSnapshot {
            schema_version: "tool-projection/v0",
            run_id: request.run_id,
            turn_id: request.turn_id,
            snapshot_id: format!("tool-projection:{}", snapshot_digest.as_str()),
            entries,
            tool_schema_set_digest,
            projection_authority_set_digest,
        })
    }
}

fn resolve_candidate(
    request: &ToolProjectionRequest,
    candidate: InvocationAuthorityCandidate,
) -> Result<ResolvedInvocation, ProjectionResolutionError> {
    let target = candidate.target;
    if candidate.installation.as_ref().is_some_and(|installation| {
        installation.tenant_id != request.tenant_id || installation.user_id != request.user_id
    }) {
        return Err(ProjectionResolutionError::TenantOrUserScopeMismatch);
    }
    let catalog = candidate
        .catalog
        .ok_or(ProjectionResolutionError::PackageMissing)?;
    if !catalog.runnable {
        return Err(ProjectionResolutionError::PackageNotRunnable);
    }
    if catalog.package_id != target.package_id || catalog.package_version != target.package_version
    {
        return Err(ProjectionResolutionError::PackageVersionMismatch);
    }
    if let Some(installation) = candidate.installation.as_ref() {
        if installation.package_id != target.package_id
            || installation.package_version != target.package_version
        {
            return Err(ProjectionResolutionError::PackageVersionMismatch);
        }
        if installation.package_digest != catalog.package_digest {
            return Err(ProjectionResolutionError::PackageDigestMismatch);
        }
    }
    if catalog.revoked {
        return Err(ProjectionResolutionError::CatalogRevoked);
    }
    let installation = candidate
        .installation
        .ok_or(ProjectionResolutionError::InstallationMissing)?;
    match installation.state {
        InstallationState::Disabled => return Err(ProjectionResolutionError::InstallationDisabled),
        InstallationState::Revoked => return Err(ProjectionResolutionError::InstallationRevoked),
        InstallationState::Enabled => {}
    }
    if installation.id != target.installation_id {
        return Err(ProjectionResolutionError::InstallationRevisionMismatch);
    }
    let component = catalog
        .component
        .ok_or(ProjectionResolutionError::ComponentMissing)?;
    if component.id != target.component_id {
        return Err(ProjectionResolutionError::ComponentIdentityMismatch);
    }
    if installation.component.id != component.id
        || installation.component.version != component.version
        || installation.component.digest != component.digest
    {
        return Err(ProjectionResolutionError::ComponentIdentityMismatch);
    }
    let admitted_execution = candidate
        .policy
        .admitted_execution_identity
        .as_ref()
        .ok_or(ProjectionResolutionError::ExecutionIdentityUnknown)?;
    if admitted_execution != &component.execution_identity
        || installation.component.execution_identity != component.execution_identity
    {
        return Err(ProjectionResolutionError::ExecutionIdentityMismatch);
    }
    let tool = component
        .tool
        .ok_or(ProjectionResolutionError::ToolMissing)?;
    if tool.id != target.tool_id {
        return Err(ProjectionResolutionError::ToolIdentityMismatch);
    }
    if candidate.policy.capability_class.is_none()
        || candidate.policy.capability_id != target.capability_id
    {
        return Err(ProjectionResolutionError::CapabilityUnknown);
    }
    if !component
        .declared_capabilities
        .contains(&target.capability_id)
        || tool.capability_id != target.capability_id
    {
        return Err(ProjectionResolutionError::CapabilityNotDeclared);
    }
    let grant = candidate
        .grant
        .ok_or(ProjectionResolutionError::CapabilityNotGranted)?;
    if grant.capability_manifest_digest != catalog.capability_manifest_digest {
        return Err(ProjectionResolutionError::CapabilityManifestMismatch);
    }
    match grant.state {
        GrantState::Stale => return Err(ProjectionResolutionError::GrantStale),
        GrantState::Expired => return Err(ProjectionResolutionError::GrantExpired),
        GrantState::Revoked => return Err(ProjectionResolutionError::GrantRevoked),
        GrantState::Active => {}
    }
    if grant.tenant_id != request.tenant_id
        || grant.user_id != request.user_id
        || grant.installation_id != installation.id
    {
        return Err(ProjectionResolutionError::GrantVersionMismatch);
    }
    if grant.capability_id != target.capability_id || grant.object_scope != target.object_scope {
        return Err(ProjectionResolutionError::GrantScopeMismatch);
    }
    let source_policy = catalog
        .source_policy
        .ok_or(ProjectionResolutionError::SourcePolicyMissing)?;
    if candidate.policy.admitted_source_policy.as_ref() != Some(&source_policy) {
        return Err(ProjectionResolutionError::SourcePolicyMismatch);
    }
    if tool.model_visible_name.len() > 64
        || !is_valid_name(&tool.model_visible_name)
        || tool.description.len() > 4_096
    {
        return Err(ProjectionResolutionError::InvalidAuthoritySnapshot);
    }
    let input_schema = tool
        .input_schema
        .ok_or(ProjectionResolutionError::SchemaMissing)?;
    if tool.claimed_input_schema_digest != *input_schema.digest() {
        return Err(ProjectionResolutionError::SchemaDigestMismatch);
    }
    let version = target.package_version.as_str();
    let dispatch_digest = digest_strings(
        b"dispatch-identity/v0\0",
        [
            target.package_id.as_str(),
            version.as_str(),
            target.component_id.as_str(),
            target.tool_id.as_str(),
        ],
    );
    let dispatch_key = format!("dispatch:{}", dispatch_digest.as_str());
    let provider_tool_definition_digest = digest_strings(
        b"provider-tool-definition/v0\0",
        [
            tool.model_visible_name.as_str(),
            tool.description.as_str(),
            input_schema.digest().as_str(),
        ],
    );
    let package_version = target.package_version.as_str();
    let projection_authority_entry_digest = digest_strings(
        b"projection-authority-entry/v0\0",
        [
            dispatch_key.as_str(),
            provider_tool_definition_digest.as_str(),
            request.tenant_id.as_str(),
            request.user_id.as_str(),
            installation.id.as_str(),
            installation.revision.as_str(),
            target.package_id.as_str(),
            package_version.as_str(),
            catalog.package_digest.as_str(),
            catalog.catalog_revision.as_str(),
            component.id.as_str(),
            component.version.as_str(),
            component.digest.as_str(),
            component.execution_identity.as_str(),
            tool.id.as_str(),
            target.capability_id.as_str(),
            catalog.capability_manifest_digest.as_str(),
            grant.snapshot_id.as_str(),
            grant.version.as_str(),
            source_policy.id.as_str(),
            source_policy.digest.as_str(),
            candidate.policy.snapshot_id.as_str(),
            candidate.policy.revision.as_str(),
            input_schema.digest().as_str(),
        ],
    );
    Ok(ResolvedInvocation {
        tenant_id: request.tenant_id.clone(),
        user_id: request.user_id.clone(),
        installation_id: installation.id,
        installation_revision: installation.revision,
        package_id: target.package_id,
        package_version: target.package_version,
        package_digest: catalog.package_digest,
        catalog_revision: catalog.catalog_revision,
        component_id: component.id,
        component_version: component.version,
        component_digest: component.digest,
        component_kind: component.kind,
        execution_identity: component.execution_identity,
        tool_id: tool.id,
        model_visible_name: tool.model_visible_name,
        dispatch_key,
        description: tool.description,
        provider_tool_definition_digest,
        capability_id: target.capability_id,
        capability_manifest_digest: catalog.capability_manifest_digest,
        grant_snapshot_id: grant.snapshot_id,
        grant_version: grant.version,
        object_scope: grant.object_scope,
        confirmation_policy: grant.confirmation_policy,
        source_policy,
        input_schema,
        policy_snapshot_id: candidate.policy.snapshot_id,
        policy_revision: candidate.policy.revision,
        projection_authority_entry_digest,
    })
}

fn digest_strings<'a>(domain: &[u8], values: impl IntoIterator<Item = &'a str>) -> Sha256Digest {
    let mut bytes = domain.to_vec();
    for value in values {
        encode_string(value, &mut bytes);
    }
    Sha256Digest::from_bytes(&bytes)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProposedToolCall {
    pub provider_tool_call_id: ProviderToolCallId,
    pub model_visible_name: String,
    pub dispatch_key: String,
    pub arguments: CanonicalArgumentValueV0,
    pub claimed_argument_digest: Sha256Digest,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CurrentDenyState {
    pub tenant_id: TenantId,
    pub user_id: UserId,
    pub catalog_revoked: bool,
    pub installation: Option<PluginInstallationSnapshot>,
    pub grant: CapabilityGrantSnapshot,
    pub policy: InvocationPolicySnapshot,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthorizedInvocation {
    pub entry: ResolvedInvocation,
    pub provider_tool_call_id: ProviderToolCallId,
    pub arguments: CanonicalArgumentValueV0,
    pub current_installation_revision: InstallationRevision,
    pub current_grant_version: GrantVersion,
    pub current_policy_revision: PolicyRevision,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InvocationAuthorizationError {
    InvalidCall,
    ToolNotProjected,
    DispatchIdentityMismatch,
    EmergencyBlocked,
    AuthorityConflict,
    TenantOrUserScopeMismatch,
    CatalogRevoked,
    InstallationMissing,
    InstallationDisabled,
    InstallationRevoked,
    InstallationRevisionMismatch,
    GrantStale,
    GrantExpired,
    GrantRevoked,
    GrantVersionMismatch,
    GrantScopeMismatch,
    ArgumentDigestMismatch,
    ArgumentsInvalid,
}

impl fmt::Display for InvocationAuthorizationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "invocation denied: {self:?}")
    }
}
impl Error for InvocationAuthorizationError {}

pub fn authorize_call(
    projection: &ToolProjectionSnapshot,
    current: CurrentDenyState,
    call: ProposedToolCall,
) -> Result<AuthorizedInvocation, InvocationAuthorizationError> {
    if call.model_visible_name.is_empty() || call.dispatch_key.is_empty() {
        return Err(InvocationAuthorizationError::InvalidCall);
    }
    let entry = projection
        .entries
        .iter()
        .find(|entry| entry.model_visible_name == call.model_visible_name)
        .ok_or(InvocationAuthorizationError::ToolNotProjected)?;
    if entry.dispatch_key != call.dispatch_key {
        return Err(InvocationAuthorizationError::DispatchIdentityMismatch);
    }
    if current.policy.emergency_blocked {
        return Err(InvocationAuthorizationError::EmergencyBlocked);
    }
    if current.policy.snapshot_id != entry.policy_snapshot_id
        || current.policy.revision != entry.policy_revision
        || current.policy.capability_id != entry.capability_id
        || current.policy.admitted_execution_identity.as_ref() != Some(&entry.execution_identity)
        || current.policy.admitted_source_policy.as_ref() != Some(&entry.source_policy)
    {
        return Err(InvocationAuthorizationError::AuthorityConflict);
    }
    if current.tenant_id != entry.tenant_id || current.user_id != entry.user_id {
        return Err(InvocationAuthorizationError::TenantOrUserScopeMismatch);
    }
    if current.catalog_revoked {
        return Err(InvocationAuthorizationError::CatalogRevoked);
    }
    let installation = current
        .installation
        .ok_or(InvocationAuthorizationError::InstallationMissing)?;
    match installation.state {
        InstallationState::Disabled => {
            return Err(InvocationAuthorizationError::InstallationDisabled);
        }
        InstallationState::Revoked => {
            return Err(InvocationAuthorizationError::InstallationRevoked);
        }
        InstallationState::Enabled => {}
    }
    if installation.id != entry.installation_id
        || installation.revision != entry.installation_revision
        || installation.package_id != entry.package_id
        || installation.package_version != entry.package_version
        || installation.package_digest != entry.package_digest
        || installation.component.id != entry.component_id
        || installation.component.version != entry.component_version
        || installation.component.digest != entry.component_digest
        || installation.component.execution_identity != entry.execution_identity
    {
        return Err(InvocationAuthorizationError::InstallationRevisionMismatch);
    }
    match current.grant.state {
        GrantState::Stale => return Err(InvocationAuthorizationError::GrantStale),
        GrantState::Expired => return Err(InvocationAuthorizationError::GrantExpired),
        GrantState::Revoked => return Err(InvocationAuthorizationError::GrantRevoked),
        GrantState::Active => {}
    }
    if current.grant.snapshot_id != entry.grant_snapshot_id
        || current.grant.version != entry.grant_version
        || current.grant.capability_manifest_digest != entry.capability_manifest_digest
    {
        return Err(InvocationAuthorizationError::GrantVersionMismatch);
    }
    if current.grant.tenant_id != entry.tenant_id
        || current.grant.user_id != entry.user_id
        || current.grant.installation_id != entry.installation_id
        || current.grant.capability_id != entry.capability_id
        || current.grant.object_scope != entry.object_scope
    {
        return Err(InvocationAuthorizationError::GrantScopeMismatch);
    }
    if call.claimed_argument_digest != *call.arguments.digest() {
        return Err(InvocationAuthorizationError::ArgumentDigestMismatch);
    }
    if !arguments_match_schema(call.arguments.root(), entry.input_schema.root()) {
        return Err(InvocationAuthorizationError::ArgumentsInvalid);
    }
    Ok(AuthorizedInvocation {
        entry: entry.clone(),
        provider_tool_call_id: call.provider_tool_call_id,
        arguments: call.arguments,
        current_installation_revision: installation.revision,
        current_grant_version: current.grant.version,
        current_policy_revision: current.policy.revision,
    })
}

fn arguments_match_schema(
    argument: &CanonicalArgumentNodeV0,
    schema: &ValidatedSchemaNodeV0,
) -> bool {
    match (argument, schema) {
        (CanonicalArgumentNodeV0::String(value), ValidatedSchemaNodeV0::String { enum_values }) => {
            enum_values
                .as_ref()
                .is_none_or(|values| values.contains(value))
        }
        (CanonicalArgumentNodeV0::Integer(_), ValidatedSchemaNodeV0::Integer)
        | (CanonicalArgumentNodeV0::Number(_), ValidatedSchemaNodeV0::Number)
        | (CanonicalArgumentNodeV0::Boolean(_), ValidatedSchemaNodeV0::Boolean) => true,
        (CanonicalArgumentNodeV0::Array(values), ValidatedSchemaNodeV0::Array { items }) => values
            .iter()
            .all(|value| arguments_match_schema(value, items)),
        (
            CanonicalArgumentNodeV0::Object(members),
            ValidatedSchemaNodeV0::Object {
                properties,
                required,
            },
        ) => {
            members.len() <= properties.len()
                && required.iter().all(|name| members.contains_key(name))
                && members.iter().all(|(name, value)| {
                    properties
                        .get(name)
                        .is_some_and(|schema| arguments_match_schema(value, schema))
                })
        }
        _ => false,
    }
}
