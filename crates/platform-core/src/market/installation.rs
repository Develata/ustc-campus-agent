//! Pure managed-installation aggregate for `market-installation/v0`.
//!
//! This module owns the M20-B3-s1 in-memory authority model only. It does not mint grants,
//! does not issue production enable evidence, does not touch a resolver, and does not open a
//! database, network, framework checkpoint or secret store.

use crate::identity::{TenantId, UserId};
use crate::invocation::{
    CatalogRevision, ComponentId, ComponentKind, ComponentVersion, ExecutionIdentity,
    InstallationId, InstallationRevision, InstallationState as ResolverInstallationState,
    InstalledComponentIdentity, PackageId, PackageVersion, PluginInstallationSnapshot,
    Sha256Digest,
};
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt;

const COMMAND_ID_PREFIX: &str = "cmd:";
const SECRET_REF_PREFIX: &str = "secret-ref:";
const MAX_COMMAND_ID_TAIL_BYTES: usize = 124;
const MAX_SECRET_REF_TAIL_BYTES: usize = 118;
const MAX_CONFIGURATION_KEY_BYTES: usize = 64;
const MAX_NON_SECRET_TEXT_BYTES: usize = 4096;
const MAX_CONFIGURATION_ENTRIES: usize = 128;

const CONFIGURATION_DOMAIN: &[u8] = b"market-installation-configuration/v0\0";
const ENABLE_EVIDENCE_DOMAIN: &[u8] = b"market-installation-enable-evidence/v0\0";

/// Construction failure for checked managed-installation values.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallationConstructionError {
    InvalidValue,
    TooManyConfigurationEntries,
    DuplicateConfigurationKey,
    CrossTenantSecretRef,
    CrossTenantConfiguration,
    DuplicateComponentId,
    EmptyComponentSet,
    SequenceZero,
    CounterOverflow,
}

impl fmt::Display for InstallationConstructionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "installation value rejected: {self:?}")
    }
}

impl Error for InstallationConstructionError {}

/// Stable domain rejection classes for a syntactically valid installation command.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallationDecisionError {
    AggregateMissing,
    AggregateAlreadyPresent,
    TerminalState,
    RevisionMismatch,
    TenantMismatch,
    IllegalTransition,
    ConfigureWhileEnabled,
    EnableEvidenceMismatch,
    SequenceOverflow,
}

impl fmt::Display for InstallationDecisionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "installation command rejected: {self:?}")
    }
}

impl Error for InstallationDecisionError {}

/// Stable replay rejection classes. No variant contains source payloads.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallationReplayError {
    InitialEventNotInstalled,
    SequenceGap,
    SequenceDuplicate,
    SequenceOverflow,
    DuplicateCommandId,
    PostTerminalEvent,
    IllegalTransition,
    PostRevisionMismatch,
    RedundantFieldMismatch,
}

impl fmt::Display for InstallationReplayError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "installation event replay rejected: {self:?}")
    }
}

impl Error for InstallationReplayError {}

/// One bounded idempotency key for the managed-installation command ledger.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InstallationCommandId(String);

impl InstallationCommandId {
    pub fn parse(value: impl Into<String>) -> Result<Self, InstallationConstructionError> {
        let value = value.into();
        let Some(tail) = value.strip_prefix(COMMAND_ID_PREFIX) else {
            return Err(InstallationConstructionError::InvalidValue);
        };
        if !tail.is_empty()
            && tail.len() <= MAX_COMMAND_ID_TAIL_BYTES
            && tail.bytes().all(is_ascii_identity_tail_byte)
        {
            Ok(Self(value))
        } else {
            Err(InstallationConstructionError::InvalidValue)
        }
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for InstallationCommandId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_tuple("InstallationCommandId")
            .field(&self.0)
            .finish()
    }
}

/// Monotone persisted event sequence. The first event is sequence 1.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InstallationEventSequence(u64);

impl InstallationEventSequence {
    pub fn new(value: u64) -> Result<Self, InstallationConstructionError> {
        if value == 0 {
            Err(InstallationConstructionError::SequenceZero)
        } else {
            Ok(Self(value))
        }
    }

    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }

    fn next(self) -> Result<Self, InstallationDecisionError> {
        self.0
            .checked_add(1)
            .ok_or(InstallationDecisionError::SequenceOverflow)
            .and_then(|value| {
                Self::new(value).map_err(|_| InstallationDecisionError::SequenceOverflow)
            })
    }
}

/// Monotone configuration revision. The initial installed configuration has revision 1.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ConfigurationRevision(u64);

impl ConfigurationRevision {
    pub fn new(value: u64) -> Result<Self, InstallationConstructionError> {
        if value == 0 {
            Err(InstallationConstructionError::SequenceZero)
        } else {
            Ok(Self(value))
        }
    }

    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }

    fn next(self) -> Result<Self, InstallationDecisionError> {
        self.0
            .checked_add(1)
            .ok_or(InstallationDecisionError::SequenceOverflow)
            .and_then(|value| {
                Self::new(value).map_err(|_| InstallationDecisionError::SequenceOverflow)
            })
    }
}

/// One ASCII configuration key.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ConfigurationKey(String);

impl ConfigurationKey {
    pub fn parse(value: impl Into<String>) -> Result<Self, InstallationConstructionError> {
        let value = value.into();
        let mut bytes = value.bytes();
        let Some(first) = bytes.next() else {
            return Err(InstallationConstructionError::InvalidValue);
        };
        if value.len() <= MAX_CONFIGURATION_KEY_BYTES
            && first.is_ascii_alphabetic()
            && bytes.all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'.' | b'-'))
        {
            Ok(Self(value))
        } else {
            Err(InstallationConstructionError::InvalidValue)
        }
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// One non-secret UTF-8 text value. Debug output deliberately redacts the payload.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NonSecretText(String);

impl NonSecretText {
    pub fn parse(value: impl Into<String>) -> Result<Self, InstallationConstructionError> {
        let value = value.into();
        if !value.is_empty()
            && value.len() <= MAX_NON_SECRET_TEXT_BYTES
            && !value.chars().any(char::is_control)
        {
            Ok(Self(value))
        } else {
            Err(InstallationConstructionError::InvalidValue)
        }
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for NonSecretText {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("NonSecretText(<redacted>)")
    }
}

/// Opaque tenant-scoped secret-reference id. It is not a digest of secret material.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SecretRefId(String);

impl SecretRefId {
    pub fn parse(value: impl Into<String>) -> Result<Self, InstallationConstructionError> {
        let value = value.into();
        let Some(tail) = value.strip_prefix(SECRET_REF_PREFIX) else {
            return Err(InstallationConstructionError::InvalidValue);
        };
        if !tail.is_empty()
            && tail.len() <= MAX_SECRET_REF_TAIL_BYTES
            && tail.bytes().all(is_ascii_identity_tail_byte)
        {
            Ok(Self(value))
        } else {
            Err(InstallationConstructionError::InvalidValue)
        }
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for SecretRefId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("SecretRefId(<redacted>)")
    }
}

/// Tenant-scoped pointer to secret material managed outside M20.
#[derive(Clone, PartialEq, Eq)]
pub struct SecretRef {
    tenant_id: TenantId,
    id: SecretRefId,
}

impl SecretRef {
    pub fn new(
        tenant_id: TenantId,
        id: SecretRefId,
    ) -> Result<Self, InstallationConstructionError> {
        Ok(Self { tenant_id, id })
    }

    #[must_use]
    pub const fn tenant_id(&self) -> &TenantId {
        &self.tenant_id
    }

    #[must_use]
    pub const fn id(&self) -> &SecretRefId {
        &self.id
    }
}

impl fmt::Debug for SecretRef {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SecretRef")
            .field("tenant_id", &self.tenant_id)
            .field("id", &"<redacted>")
            .finish()
    }
}

/// One typed configuration value. Text and secret-reference payloads are redacted in Debug.
#[derive(Clone, PartialEq, Eq)]
pub enum ConfigurationValue {
    Text(NonSecretText),
    Integer(i64),
    Boolean(bool),
    Secret(SecretRef),
}

impl fmt::Debug for ConfigurationValue {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Text(_) => formatter.write_str("Text(<redacted>)"),
            Self::Integer(value) => formatter.debug_tuple("Integer").field(value).finish(),
            Self::Boolean(value) => formatter.debug_tuple("Boolean").field(value).finish(),
            Self::Secret(_) => formatter.write_str("Secret(<redacted>)"),
        }
    }
}

/// Immutable tenant-scoped configuration with deterministic canonical digest.
#[derive(Clone, PartialEq, Eq)]
pub struct InstallationConfiguration {
    tenant_id: TenantId,
    entries: BTreeMap<ConfigurationKey, ConfigurationValue>,
    digest: Sha256Digest,
}

impl InstallationConfiguration {
    pub fn new(
        tenant_id: &TenantId,
        entries: Vec<(ConfigurationKey, ConfigurationValue)>,
    ) -> Result<Self, InstallationConstructionError> {
        if entries.len() > MAX_CONFIGURATION_ENTRIES {
            return Err(InstallationConstructionError::TooManyConfigurationEntries);
        }
        let mut canonical_entries = BTreeMap::new();
        for (key, value) in entries {
            match &value {
                ConfigurationValue::Secret(secret) if secret.tenant_id() != tenant_id => {
                    return Err(InstallationConstructionError::CrossTenantSecretRef);
                }
                _ => {}
            }
            if canonical_entries.insert(key, value).is_some() {
                return Err(InstallationConstructionError::DuplicateConfigurationKey);
            }
        }
        let digest = digest_configuration(tenant_id, &canonical_entries);
        Ok(Self {
            tenant_id: tenant_id.clone(),
            entries: canonical_entries,
            digest,
        })
    }

    const fn tenant_id(&self) -> &TenantId {
        &self.tenant_id
    }

    #[must_use]
    pub const fn entries(&self) -> &BTreeMap<ConfigurationKey, ConfigurationValue> {
        &self.entries
    }

    #[must_use]
    pub const fn digest(&self) -> &Sha256Digest {
        &self.digest
    }
}

impl fmt::Debug for InstallationConfiguration {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("InstallationConfiguration")
            .field("tenant_id", &self.tenant_id)
            .field("entry_count", &self.entries.len())
            .field("digest", &self.digest)
            .finish()
    }
}

/// One exact installed component pin.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstalledComponentPin {
    component_id: ComponentId,
    kind: ComponentKind,
    version: ComponentVersion,
    digest: Sha256Digest,
    execution_identity: ExecutionIdentity,
}

impl InstalledComponentPin {
    pub fn new(
        component_id: ComponentId,
        kind: ComponentKind,
        version: ComponentVersion,
        digest: Sha256Digest,
        execution_identity: ExecutionIdentity,
    ) -> Result<Self, InstallationConstructionError> {
        Ok(Self {
            component_id,
            kind,
            version,
            digest,
            execution_identity,
        })
    }

    #[must_use]
    pub const fn component_id(&self) -> &ComponentId {
        &self.component_id
    }

    #[must_use]
    pub const fn kind(&self) -> ComponentKind {
        self.kind
    }

    #[must_use]
    pub const fn version(&self) -> &ComponentVersion {
        &self.version
    }

    #[must_use]
    pub const fn digest(&self) -> &Sha256Digest {
        &self.digest
    }

    #[must_use]
    pub const fn execution_identity(&self) -> &ExecutionIdentity {
        &self.execution_identity
    }

    #[must_use]
    pub fn to_installed_identity(&self) -> InstalledComponentIdentity {
        InstalledComponentIdentity {
            id: self.component_id.clone(),
            version: self.version.clone(),
            digest: self.digest.clone(),
            execution_identity: self.execution_identity.clone(),
        }
    }
}

/// Exact package, component-set, configuration-independent installation pins.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallationPackagePin {
    catalog_revision: CatalogRevision,
    package_id: PackageId,
    package_version: PackageVersion,
    package_digest: Sha256Digest,
    components: Vec<InstalledComponentPin>,
    component_set_digest: Sha256Digest,
    capability_manifest_digest: Sha256Digest,
}

impl InstallationPackagePin {
    pub fn new(
        catalog_revision: CatalogRevision,
        package_id: PackageId,
        package_version: PackageVersion,
        package_digest: Sha256Digest,
        mut components: Vec<InstalledComponentPin>,
        component_set_digest: Sha256Digest,
        capability_manifest_digest: Sha256Digest,
    ) -> Result<Self, InstallationConstructionError> {
        if components.is_empty() {
            return Err(InstallationConstructionError::EmptyComponentSet);
        }
        components.sort_by(|left, right| left.component_id().cmp(right.component_id()));
        let mut ids = BTreeSet::new();
        for component in &components {
            if !ids.insert(component.component_id().clone()) {
                return Err(InstallationConstructionError::DuplicateComponentId);
            }
        }
        Ok(Self {
            catalog_revision,
            package_id,
            package_version,
            package_digest,
            components,
            component_set_digest,
            capability_manifest_digest,
        })
    }

    #[must_use]
    pub const fn catalog_revision(&self) -> &CatalogRevision {
        &self.catalog_revision
    }

    #[must_use]
    pub const fn package_id(&self) -> &PackageId {
        &self.package_id
    }

    #[must_use]
    pub const fn package_version(&self) -> &PackageVersion {
        &self.package_version
    }

    #[must_use]
    pub const fn package_digest(&self) -> &Sha256Digest {
        &self.package_digest
    }

    #[must_use]
    pub fn components(&self) -> &[InstalledComponentPin] {
        &self.components
    }

    #[must_use]
    pub const fn component_set_digest(&self) -> &Sha256Digest {
        &self.component_set_digest
    }

    #[must_use]
    pub const fn capability_manifest_digest(&self) -> &Sha256Digest {
        &self.capability_manifest_digest
    }
}

/// Managed installation lifecycle distinct from the legacy resolver state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ManagedInstallationState {
    InstalledDisabled,
    Enabled,
    Disabled,
    Revoked,
    Uninstalled,
}

impl ManagedInstallationState {
    const fn is_terminal(self) -> bool {
        matches!(self, Self::Revoked | Self::Uninstalled)
    }
}

/// Non-public evidence that future authority composition minted an exact enable precondition.
#[derive(Clone, PartialEq, Eq)]
pub struct EnablePreconditionEvidence {
    installation_id: InstallationId,
    expected_installation_revision: InstallationRevision,
    package_digest: Sha256Digest,
    component_set_digest: Sha256Digest,
    configuration_digest: Sha256Digest,
    capability_manifest_digest: Sha256Digest,
    grant_set_snapshot_digest: Sha256Digest,
    policy_admission_snapshot_digest: Sha256Digest,
    evidence_digest: Sha256Digest,
}

impl EnablePreconditionEvidence {
    #[allow(clippy::too_many_arguments, dead_code)]
    pub(in crate::market) fn from_authority_bindings(
        installation_id: InstallationId,
        expected_installation_revision: InstallationRevision,
        package_digest: Sha256Digest,
        component_set_digest: Sha256Digest,
        configuration_digest: Sha256Digest,
        capability_manifest_digest: Sha256Digest,
        grant_set_snapshot_digest: Sha256Digest,
        policy_admission_snapshot_digest: Sha256Digest,
    ) -> Result<Self, InstallationConstructionError> {
        let evidence_digest = digest_enable_evidence(
            &installation_id,
            &expected_installation_revision,
            &package_digest,
            &component_set_digest,
            &configuration_digest,
            &capability_manifest_digest,
            &grant_set_snapshot_digest,
            &policy_admission_snapshot_digest,
        );
        Ok(Self {
            installation_id,
            expected_installation_revision,
            package_digest,
            component_set_digest,
            configuration_digest,
            capability_manifest_digest,
            grant_set_snapshot_digest,
            policy_admission_snapshot_digest,
            evidence_digest,
        })
    }

    #[must_use]
    pub const fn installation_id(&self) -> &InstallationId {
        &self.installation_id
    }

    #[must_use]
    pub const fn expected_installation_revision(&self) -> &InstallationRevision {
        &self.expected_installation_revision
    }

    #[must_use]
    pub const fn package_digest(&self) -> &Sha256Digest {
        &self.package_digest
    }

    #[must_use]
    pub const fn component_set_digest(&self) -> &Sha256Digest {
        &self.component_set_digest
    }

    #[must_use]
    pub const fn configuration_digest(&self) -> &Sha256Digest {
        &self.configuration_digest
    }

    #[must_use]
    pub const fn capability_manifest_digest(&self) -> &Sha256Digest {
        &self.capability_manifest_digest
    }

    #[must_use]
    pub const fn grant_set_snapshot_digest(&self) -> &Sha256Digest {
        &self.grant_set_snapshot_digest
    }

    #[must_use]
    pub const fn policy_admission_snapshot_digest(&self) -> &Sha256Digest {
        &self.policy_admission_snapshot_digest
    }

    #[must_use]
    pub const fn evidence_digest(&self) -> &Sha256Digest {
        &self.evidence_digest
    }
}

impl fmt::Debug for EnablePreconditionEvidence {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("EnablePreconditionEvidence")
            .field("installation_id", &self.installation_id)
            .field(
                "expected_installation_revision",
                &self.expected_installation_revision,
            )
            .field("evidence_digest", &self.evidence_digest)
            .finish()
    }
}

#[derive(Clone, PartialEq, Eq)]
enum InstallationCommandAction {
    Install {
        tenant_id: TenantId,
        user_id: UserId,
        package_pin: InstallationPackagePin,
        configuration: InstallationConfiguration,
    },
    Configure {
        expected_revision: InstallationRevision,
        configuration: InstallationConfiguration,
    },
    Enable {
        expected_revision: InstallationRevision,
        evidence: EnablePreconditionEvidence,
    },
    Disable {
        expected_revision: InstallationRevision,
    },
    Revoke {
        expected_revision: InstallationRevision,
    },
    Uninstall {
        expected_revision: InstallationRevision,
    },
}

impl fmt::Debug for InstallationCommandAction {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Install { .. } => formatter.write_str("Install { <redacted> }"),
            Self::Configure {
                expected_revision, ..
            } => formatter
                .debug_struct("Configure")
                .field("expected_revision", expected_revision)
                .finish(),
            Self::Enable {
                expected_revision, ..
            } => formatter
                .debug_struct("Enable")
                .field("expected_revision", expected_revision)
                .finish(),
            Self::Disable { expected_revision } => formatter
                .debug_struct("Disable")
                .field("expected_revision", expected_revision)
                .finish(),
            Self::Revoke { expected_revision } => formatter
                .debug_struct("Revoke")
                .field("expected_revision", expected_revision)
                .finish(),
            Self::Uninstall { expected_revision } => formatter
                .debug_struct("Uninstall")
                .field("expected_revision", expected_revision)
                .finish(),
        }
    }
}

/// One checked installation command. Fields and action constructors are closed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallationCommand {
    command_id: InstallationCommandId,
    installation_id: InstallationId,
    action: InstallationCommandAction,
}

impl InstallationCommand {
    pub fn install(
        command_id: InstallationCommandId,
        installation_id: InstallationId,
        tenant_id: TenantId,
        user_id: UserId,
        package_pin: InstallationPackagePin,
        configuration: InstallationConfiguration,
    ) -> Result<Self, InstallationConstructionError> {
        if configuration.tenant_id() != &tenant_id {
            return Err(InstallationConstructionError::CrossTenantConfiguration);
        }
        Ok(Self {
            command_id,
            installation_id,
            action: InstallationCommandAction::Install {
                tenant_id,
                user_id,
                package_pin,
                configuration,
            },
        })
    }

    pub fn configure(
        command_id: InstallationCommandId,
        installation_id: InstallationId,
        expected_revision: InstallationRevision,
        configuration: InstallationConfiguration,
    ) -> Result<Self, InstallationConstructionError> {
        Ok(Self {
            command_id,
            installation_id,
            action: InstallationCommandAction::Configure {
                expected_revision,
                configuration,
            },
        })
    }

    pub fn enable(
        command_id: InstallationCommandId,
        installation_id: InstallationId,
        expected_revision: InstallationRevision,
        evidence: EnablePreconditionEvidence,
    ) -> Result<Self, InstallationConstructionError> {
        Ok(Self {
            command_id,
            installation_id,
            action: InstallationCommandAction::Enable {
                expected_revision,
                evidence,
            },
        })
    }

    pub fn disable(
        command_id: InstallationCommandId,
        installation_id: InstallationId,
        expected_revision: InstallationRevision,
    ) -> Result<Self, InstallationConstructionError> {
        Ok(Self {
            command_id,
            installation_id,
            action: InstallationCommandAction::Disable { expected_revision },
        })
    }

    pub fn revoke(
        command_id: InstallationCommandId,
        installation_id: InstallationId,
        expected_revision: InstallationRevision,
    ) -> Result<Self, InstallationConstructionError> {
        Ok(Self {
            command_id,
            installation_id,
            action: InstallationCommandAction::Revoke { expected_revision },
        })
    }

    pub fn uninstall(
        command_id: InstallationCommandId,
        installation_id: InstallationId,
        expected_revision: InstallationRevision,
    ) -> Result<Self, InstallationConstructionError> {
        Ok(Self {
            command_id,
            installation_id,
            action: InstallationCommandAction::Uninstall { expected_revision },
        })
    }

    #[must_use]
    pub const fn command_id(&self) -> &InstallationCommandId {
        &self.command_id
    }

    #[must_use]
    pub const fn installation_id(&self) -> &InstallationId {
        &self.installation_id
    }
}

/// Public event kind classifier. Payloads remain sealed inside [`InstallationEvent`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallationEventKind {
    Installed,
    Configured,
    Enabled,
    Disabled,
    Revoked,
    Uninstalled,
}

#[derive(Clone, PartialEq, Eq)]
enum InstallationEventPayload {
    Installed {
        installation_id: InstallationId,
        tenant_id: TenantId,
        user_id: UserId,
        package_pin: InstallationPackagePin,
        configuration: InstallationConfiguration,
        configuration_revision: ConfigurationRevision,
    },
    Configured {
        installation_id: InstallationId,
        configuration: InstallationConfiguration,
        configuration_revision: ConfigurationRevision,
    },
    Enabled {
        installation_id: InstallationId,
        evidence: EnablePreconditionEvidence,
    },
    Disabled {
        installation_id: InstallationId,
    },
    Revoked {
        installation_id: InstallationId,
    },
    Uninstalled {
        installation_id: InstallationId,
    },
}

impl InstallationEventPayload {
    const fn kind(&self) -> InstallationEventKind {
        match self {
            Self::Installed { .. } => InstallationEventKind::Installed,
            Self::Configured { .. } => InstallationEventKind::Configured,
            Self::Enabled { .. } => InstallationEventKind::Enabled,
            Self::Disabled { .. } => InstallationEventKind::Disabled,
            Self::Revoked { .. } => InstallationEventKind::Revoked,
            Self::Uninstalled { .. } => InstallationEventKind::Uninstalled,
        }
    }

    fn installation_id(&self) -> &InstallationId {
        match self {
            Self::Installed {
                installation_id, ..
            }
            | Self::Configured {
                installation_id, ..
            }
            | Self::Enabled {
                installation_id, ..
            }
            | Self::Disabled { installation_id }
            | Self::Revoked { installation_id }
            | Self::Uninstalled { installation_id } => installation_id,
        }
    }
}

impl fmt::Debug for InstallationEventPayload {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("InstallationEventPayload")
            .field("kind", &self.kind())
            .field("installation_id", self.installation_id())
            .finish()
    }
}

/// One persisted installation event envelope.
#[derive(Clone, PartialEq, Eq)]
pub struct InstallationEvent {
    sequence: InstallationEventSequence,
    post_revision: InstallationRevision,
    command_id: InstallationCommandId,
    payload: InstallationEventPayload,
}

impl InstallationEvent {
    #[must_use]
    pub const fn sequence(&self) -> InstallationEventSequence {
        self.sequence
    }

    #[must_use]
    pub const fn post_revision(&self) -> &InstallationRevision {
        &self.post_revision
    }

    #[must_use]
    pub const fn command_id(&self) -> &InstallationCommandId {
        &self.command_id
    }

    #[must_use]
    pub const fn kind(&self) -> InstallationEventKind {
        self.payload.kind()
    }
}

impl fmt::Debug for InstallationEvent {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("InstallationEvent")
            .field("sequence", &self.sequence)
            .field("post_revision", &self.post_revision)
            .field("command_id", &self.command_id)
            .field("kind", &self.kind())
            .finish()
    }
}

/// Current aggregate snapshot. The same type is returned by the semantic fake.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallationAggregate {
    installation_id: InstallationId,
    tenant_id: TenantId,
    user_id: UserId,
    package_pin: InstallationPackagePin,
    configuration: InstallationConfiguration,
    configuration_revision: ConfigurationRevision,
    state: ManagedInstallationState,
    revision: InstallationRevision,
    last_sequence: InstallationEventSequence,
}

pub type InstallationSnapshot = InstallationAggregate;

impl InstallationAggregate {
    #[must_use]
    pub const fn installation_id(&self) -> &InstallationId {
        &self.installation_id
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
    pub const fn package_pin(&self) -> &InstallationPackagePin {
        &self.package_pin
    }

    #[must_use]
    pub const fn configuration(&self) -> &InstallationConfiguration {
        &self.configuration
    }

    #[must_use]
    pub const fn configuration_revision(&self) -> ConfigurationRevision {
        self.configuration_revision
    }

    #[must_use]
    pub const fn state(&self) -> ManagedInstallationState {
        self.state
    }

    #[must_use]
    pub const fn revision(&self) -> &InstallationRevision {
        &self.revision
    }

    #[must_use]
    pub const fn last_sequence(&self) -> InstallationEventSequence {
        self.last_sequence
    }

    /// Pure deny-side projection into the legacy resolver snapshot. No grants are implied.
    #[must_use]
    pub fn to_resolver_snapshot(&self) -> Option<PluginInstallationSnapshot> {
        let state = match self.state {
            ManagedInstallationState::InstalledDisabled | ManagedInstallationState::Disabled => {
                ResolverInstallationState::Disabled
            }
            ManagedInstallationState::Enabled => ResolverInstallationState::Enabled,
            ManagedInstallationState::Revoked => ResolverInstallationState::Revoked,
            ManagedInstallationState::Uninstalled => return None,
        };
        let component = self
            .package_pin
            .components()
            .first()?
            .to_installed_identity();
        Some(PluginInstallationSnapshot {
            id: self.installation_id.clone(),
            tenant_id: self.tenant_id.clone(),
            user_id: self.user_id.clone(),
            package_id: self.package_pin.package_id().clone(),
            package_version: self.package_pin.package_version().clone(),
            package_digest: self.package_pin.package_digest().clone(),
            component,
            state,
            revision: self.revision.clone(),
        })
    }
}

/// Computes one domain event from the current aggregate and a checked command.
pub fn decide(
    current: Option<&InstallationAggregate>,
    command: &InstallationCommand,
) -> Result<InstallationEvent, InstallationDecisionError> {
    match &command.action {
        InstallationCommandAction::Install {
            tenant_id,
            user_id,
            package_pin,
            configuration,
        } => {
            if current.is_some() {
                return Err(InstallationDecisionError::AggregateAlreadyPresent);
            }
            event(
                InstallationEventSequence::new(1)
                    .map_err(|_| InstallationDecisionError::SequenceOverflow)?,
                command.command_id.clone(),
                InstallationEventPayload::Installed {
                    installation_id: command.installation_id.clone(),
                    tenant_id: tenant_id.clone(),
                    user_id: user_id.clone(),
                    package_pin: package_pin.clone(),
                    configuration: configuration.clone(),
                    configuration_revision: ConfigurationRevision::new(1)
                        .map_err(|_| InstallationDecisionError::SequenceOverflow)?,
                },
            )
        }
        InstallationCommandAction::Configure {
            expected_revision,
            configuration,
        } => {
            let aggregate = require_current(current)?;
            require_target(aggregate, command.installation_id())?;
            require_nonterminal(aggregate)?;
            require_revision(aggregate, expected_revision)?;
            require_configuration_tenant(aggregate, configuration)?;
            match aggregate.state {
                ManagedInstallationState::InstalledDisabled
                | ManagedInstallationState::Disabled => event(
                    aggregate.last_sequence.next()?,
                    command.command_id.clone(),
                    InstallationEventPayload::Configured {
                        installation_id: command.installation_id.clone(),
                        configuration: configuration.clone(),
                        configuration_revision: aggregate.configuration_revision.next()?,
                    },
                ),
                ManagedInstallationState::Enabled => {
                    Err(InstallationDecisionError::ConfigureWhileEnabled)
                }
                ManagedInstallationState::Revoked | ManagedInstallationState::Uninstalled => {
                    Err(InstallationDecisionError::TerminalState)
                }
            }
        }
        InstallationCommandAction::Enable {
            expected_revision,
            evidence,
        } => {
            let aggregate = require_current(current)?;
            require_target(aggregate, command.installation_id())?;
            require_nonterminal(aggregate)?;
            require_revision(aggregate, expected_revision)?;
            match aggregate.state {
                ManagedInstallationState::InstalledDisabled
                | ManagedInstallationState::Disabled => {
                    if !evidence_matches(aggregate, evidence, expected_revision) {
                        return Err(InstallationDecisionError::EnableEvidenceMismatch);
                    }
                    event(
                        aggregate.last_sequence.next()?,
                        command.command_id.clone(),
                        InstallationEventPayload::Enabled {
                            installation_id: command.installation_id.clone(),
                            evidence: evidence.clone(),
                        },
                    )
                }
                ManagedInstallationState::Enabled => {
                    Err(InstallationDecisionError::IllegalTransition)
                }
                ManagedInstallationState::Revoked | ManagedInstallationState::Uninstalled => {
                    Err(InstallationDecisionError::TerminalState)
                }
            }
        }
        InstallationCommandAction::Disable { expected_revision } => {
            let aggregate = require_current(current)?;
            require_target(aggregate, command.installation_id())?;
            require_nonterminal(aggregate)?;
            require_revision(aggregate, expected_revision)?;
            if aggregate.state != ManagedInstallationState::Enabled {
                return Err(InstallationDecisionError::IllegalTransition);
            }
            event(
                aggregate.last_sequence.next()?,
                command.command_id.clone(),
                InstallationEventPayload::Disabled {
                    installation_id: command.installation_id.clone(),
                },
            )
        }
        InstallationCommandAction::Revoke { expected_revision } => {
            let aggregate = require_current(current)?;
            require_target(aggregate, command.installation_id())?;
            require_nonterminal(aggregate)?;
            require_revision(aggregate, expected_revision)?;
            event(
                aggregate.last_sequence.next()?,
                command.command_id.clone(),
                InstallationEventPayload::Revoked {
                    installation_id: command.installation_id.clone(),
                },
            )
        }
        InstallationCommandAction::Uninstall { expected_revision } => {
            let aggregate = require_current(current)?;
            require_target(aggregate, command.installation_id())?;
            require_nonterminal(aggregate)?;
            require_revision(aggregate, expected_revision)?;
            event(
                aggregate.last_sequence.next()?,
                command.command_id.clone(),
                InstallationEventPayload::Uninstalled {
                    installation_id: command.installation_id.clone(),
                },
            )
        }
    }
}

/// Applies one event and independently verifies redundant post-revision authority.
pub fn evolve(
    current: Option<InstallationAggregate>,
    event: &InstallationEvent,
) -> Result<InstallationAggregate, InstallationReplayError> {
    require_next_event_sequence(current.as_ref(), event.sequence)?;
    if event.post_revision != revision_for_sequence(event.sequence)? {
        return Err(InstallationReplayError::PostRevisionMismatch);
    }
    match (&current, &event.payload) {
        (
            None,
            InstallationEventPayload::Installed {
                installation_id,
                tenant_id,
                user_id,
                package_pin,
                configuration,
                configuration_revision,
            },
        ) => {
            if configuration_revision.get() != 1 || configuration.tenant_id() != tenant_id {
                return Err(InstallationReplayError::RedundantFieldMismatch);
            }
            Ok(InstallationAggregate {
                installation_id: installation_id.clone(),
                tenant_id: tenant_id.clone(),
                user_id: user_id.clone(),
                package_pin: package_pin.clone(),
                configuration: configuration.clone(),
                configuration_revision: *configuration_revision,
                state: ManagedInstallationState::InstalledDisabled,
                revision: event.post_revision.clone(),
                last_sequence: event.sequence,
            })
        }
        (None, _) => Err(InstallationReplayError::InitialEventNotInstalled),
        (Some(_), InstallationEventPayload::Installed { .. }) => {
            Err(InstallationReplayError::IllegalTransition)
        }
        (
            Some(aggregate),
            InstallationEventPayload::Configured {
                installation_id,
                configuration,
                configuration_revision,
            },
        ) => {
            ensure_same_installation(aggregate, installation_id)?;
            ensure_configuration_tenant(aggregate, configuration)?;
            if aggregate.state.is_terminal() {
                return Err(InstallationReplayError::PostTerminalEvent);
            }
            if !matches!(
                aggregate.state,
                ManagedInstallationState::InstalledDisabled | ManagedInstallationState::Disabled
            ) {
                return Err(InstallationReplayError::IllegalTransition);
            }
            let expected_config = aggregate
                .configuration_revision
                .get()
                .checked_add(1)
                .ok_or(InstallationReplayError::SequenceOverflow)?;
            if configuration_revision.get() != expected_config {
                return Err(InstallationReplayError::RedundantFieldMismatch);
            }
            let mut next = aggregate.clone();
            next.configuration = configuration.clone();
            next.configuration_revision = *configuration_revision;
            next.revision = event.post_revision.clone();
            next.last_sequence = event.sequence;
            Ok(next)
        }
        (
            Some(aggregate),
            InstallationEventPayload::Enabled {
                installation_id,
                evidence,
            },
        ) => {
            ensure_same_installation(aggregate, installation_id)?;
            if aggregate.state.is_terminal() {
                return Err(InstallationReplayError::PostTerminalEvent);
            }
            if !matches!(
                aggregate.state,
                ManagedInstallationState::InstalledDisabled | ManagedInstallationState::Disabled
            ) || !evidence_matches(aggregate, evidence, &aggregate.revision)
            {
                return Err(InstallationReplayError::IllegalTransition);
            }
            transition(aggregate, event, ManagedInstallationState::Enabled)
        }
        (Some(aggregate), InstallationEventPayload::Disabled { installation_id }) => {
            ensure_same_installation(aggregate, installation_id)?;
            if aggregate.state.is_terminal() {
                return Err(InstallationReplayError::PostTerminalEvent);
            }
            if aggregate.state != ManagedInstallationState::Enabled {
                return Err(InstallationReplayError::IllegalTransition);
            }
            transition(aggregate, event, ManagedInstallationState::Disabled)
        }
        (Some(aggregate), InstallationEventPayload::Revoked { installation_id }) => {
            ensure_same_installation(aggregate, installation_id)?;
            if aggregate.state.is_terminal() {
                return Err(InstallationReplayError::PostTerminalEvent);
            }
            transition(aggregate, event, ManagedInstallationState::Revoked)
        }
        (Some(aggregate), InstallationEventPayload::Uninstalled { installation_id }) => {
            ensure_same_installation(aggregate, installation_id)?;
            if aggregate.state.is_terminal() {
                return Err(InstallationReplayError::PostTerminalEvent);
            }
            transition(aggregate, event, ManagedInstallationState::Uninstalled)
        }
    }
}

/// Replays a complete event history in one pass.
pub fn replay<'a>(
    events: impl IntoIterator<Item = &'a InstallationEvent>,
) -> Result<Option<InstallationAggregate>, InstallationReplayError> {
    let mut current = None;
    let mut expected_sequence = 1_u64;
    let mut command_ids = BTreeSet::new();
    for event in events {
        if current
            .as_ref()
            .is_some_and(|aggregate: &InstallationAggregate| aggregate.state.is_terminal())
        {
            return Err(InstallationReplayError::PostTerminalEvent);
        }
        if current.is_none() && event.kind() != InstallationEventKind::Installed {
            return Err(InstallationReplayError::InitialEventNotInstalled);
        }
        if event.sequence.get() == u64::MAX {
            return Err(InstallationReplayError::SequenceOverflow);
        }
        match event.sequence.get().cmp(&expected_sequence) {
            std::cmp::Ordering::Less => return Err(InstallationReplayError::SequenceDuplicate),
            std::cmp::Ordering::Greater => return Err(InstallationReplayError::SequenceGap),
            std::cmp::Ordering::Equal => {}
        }
        if !command_ids.insert(event.command_id.clone()) {
            return Err(InstallationReplayError::DuplicateCommandId);
        }
        current = Some(evolve(current, event)?);
        expected_sequence = expected_sequence
            .checked_add(1)
            .ok_or(InstallationReplayError::SequenceOverflow)?;
    }
    Ok(current)
}

/// Semantic repository port for installation-domain commands and snapshots.
pub trait InstallationRepository {
    fn execute(
        &mut self,
        command: InstallationCommand,
    ) -> Result<InstallationCommandReceipt, InstallationRepositoryError>;

    fn load_exact(
        &self,
        id: &InstallationId,
    ) -> Result<Option<InstallationSnapshot>, InstallationRepositoryError>;

    fn event_history(
        &self,
        id: &InstallationId,
    ) -> Result<Vec<InstallationEvent>, InstallationRepositoryError>;
}

/// Result of a persisted command execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallationCommandReceipt {
    command: InstallationCommand,
    outcome: InstallationCommandOutcome,
}

impl InstallationCommandReceipt {
    #[must_use]
    pub const fn command(&self) -> &InstallationCommand {
        &self.command
    }

    #[must_use]
    pub const fn outcome(&self) -> &InstallationCommandOutcome {
        &self.outcome
    }
}

/// Accepted or rejected command outcome stored by the command ledger.
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InstallationCommandOutcome {
    Accepted {
        event: InstallationEvent,
        snapshot: InstallationSnapshot,
    },
    Rejected {
        error: InstallationDecisionError,
    },
}

/// Repository-level failures. Domain rejections are normally persisted receipts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InstallationRepositoryError {
    CommandConflict,
    InjectedPersistenceFailure,
    CorruptEventHistory(InstallationReplayError),
    DecisionRejected(InstallationDecisionError),
}

impl fmt::Display for InstallationRepositoryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "installation repository rejected operation: {self:?}"
        )
    }
}

impl Error for InstallationRepositoryError {}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CommandLedgerEntry {
    command: InstallationCommand,
    receipt: InstallationCommandReceipt,
}

/// Deterministic semantic in-memory fake with idempotent command receipts.
#[derive(Debug, Default)]
pub struct InMemoryInstallationRepository {
    aggregates: BTreeMap<InstallationId, InstallationAggregate>,
    events: BTreeMap<InstallationId, Vec<InstallationEvent>>,
    command_ledger: BTreeMap<InstallationCommandId, CommandLedgerEntry>,
    fail_next_commit: bool,
}

impl InMemoryInstallationRepository {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// One-shot pre-commit failure injection for repository tests.
    pub fn fail_next_commit_for_testing(&mut self) {
        self.fail_next_commit = true;
    }
}

impl InstallationRepository for InMemoryInstallationRepository {
    fn execute(
        &mut self,
        command: InstallationCommand,
    ) -> Result<InstallationCommandReceipt, InstallationRepositoryError> {
        if let Some(entry) = self.command_ledger.get(command.command_id()) {
            if entry.command == command {
                return Ok(entry.receipt.clone());
            }
            return Err(InstallationRepositoryError::CommandConflict);
        }

        let current = self.aggregates.get(command.installation_id());
        let decision = decide(current, &command);
        if self.fail_next_commit {
            self.fail_next_commit = false;
            return Err(InstallationRepositoryError::InjectedPersistenceFailure);
        }

        let receipt = match decision {
            Ok(event) => {
                let snapshot = evolve(current.cloned(), &event)
                    .map_err(InstallationRepositoryError::CorruptEventHistory)?;
                self.aggregates
                    .insert(command.installation_id().clone(), snapshot.clone());
                self.events
                    .entry(command.installation_id().clone())
                    .or_default()
                    .push(event.clone());
                InstallationCommandReceipt {
                    command: command.clone(),
                    outcome: InstallationCommandOutcome::Accepted { event, snapshot },
                }
            }
            Err(error) => InstallationCommandReceipt {
                command: command.clone(),
                outcome: InstallationCommandOutcome::Rejected { error },
            },
        };
        self.command_ledger.insert(
            command.command_id().clone(),
            CommandLedgerEntry {
                command,
                receipt: receipt.clone(),
            },
        );
        Ok(receipt)
    }

    fn load_exact(
        &self,
        id: &InstallationId,
    ) -> Result<Option<InstallationSnapshot>, InstallationRepositoryError> {
        Ok(self.aggregates.get(id).cloned())
    }

    fn event_history(
        &self,
        id: &InstallationId,
    ) -> Result<Vec<InstallationEvent>, InstallationRepositoryError> {
        Ok(self.events.get(id).cloned().unwrap_or_default())
    }
}

fn require_current(
    current: Option<&InstallationAggregate>,
) -> Result<&InstallationAggregate, InstallationDecisionError> {
    current.ok_or(InstallationDecisionError::AggregateMissing)
}

fn require_target(
    aggregate: &InstallationAggregate,
    installation_id: &InstallationId,
) -> Result<(), InstallationDecisionError> {
    if aggregate.installation_id() == installation_id {
        Ok(())
    } else {
        Err(InstallationDecisionError::RevisionMismatch)
    }
}

fn require_nonterminal(aggregate: &InstallationAggregate) -> Result<(), InstallationDecisionError> {
    if aggregate.state.is_terminal() {
        Err(InstallationDecisionError::TerminalState)
    } else {
        Ok(())
    }
}

fn require_revision(
    aggregate: &InstallationAggregate,
    expected_revision: &InstallationRevision,
) -> Result<(), InstallationDecisionError> {
    if aggregate.revision() == expected_revision {
        Ok(())
    } else {
        Err(InstallationDecisionError::RevisionMismatch)
    }
}

fn require_configuration_tenant(
    aggregate: &InstallationAggregate,
    configuration: &InstallationConfiguration,
) -> Result<(), InstallationDecisionError> {
    if configuration.tenant_id() == aggregate.tenant_id() {
        Ok(())
    } else {
        Err(InstallationDecisionError::TenantMismatch)
    }
}

fn event(
    sequence: InstallationEventSequence,
    command_id: InstallationCommandId,
    payload: InstallationEventPayload,
) -> Result<InstallationEvent, InstallationDecisionError> {
    let post_revision = revision_for_sequence(sequence).map_err(|error| match error {
        InstallationReplayError::SequenceOverflow => InstallationDecisionError::SequenceOverflow,
        _ => InstallationDecisionError::SequenceOverflow,
    })?;
    Ok(InstallationEvent {
        sequence,
        post_revision,
        command_id,
        payload,
    })
}

fn transition(
    aggregate: &InstallationAggregate,
    event: &InstallationEvent,
    state: ManagedInstallationState,
) -> Result<InstallationAggregate, InstallationReplayError> {
    let mut next = aggregate.clone();
    next.state = state;
    next.revision = event.post_revision.clone();
    next.last_sequence = event.sequence;
    Ok(next)
}

fn ensure_same_installation(
    aggregate: &InstallationAggregate,
    installation_id: &InstallationId,
) -> Result<(), InstallationReplayError> {
    if aggregate.installation_id() == installation_id {
        Ok(())
    } else {
        Err(InstallationReplayError::RedundantFieldMismatch)
    }
}

fn ensure_configuration_tenant(
    aggregate: &InstallationAggregate,
    configuration: &InstallationConfiguration,
) -> Result<(), InstallationReplayError> {
    if configuration.tenant_id() == aggregate.tenant_id() {
        Ok(())
    } else {
        Err(InstallationReplayError::RedundantFieldMismatch)
    }
}

fn require_next_event_sequence(
    current: Option<&InstallationAggregate>,
    sequence: InstallationEventSequence,
) -> Result<(), InstallationReplayError> {
    let expected = match current {
        None => 1_u64,
        Some(aggregate) => aggregate
            .last_sequence()
            .get()
            .checked_add(1)
            .ok_or(InstallationReplayError::SequenceOverflow)?,
    };
    match sequence.get().cmp(&expected) {
        std::cmp::Ordering::Less => Err(InstallationReplayError::SequenceDuplicate),
        std::cmp::Ordering::Greater => Err(InstallationReplayError::SequenceGap),
        std::cmp::Ordering::Equal => Ok(()),
    }
}

fn evidence_matches(
    aggregate: &InstallationAggregate,
    evidence: &EnablePreconditionEvidence,
    expected_revision: &InstallationRevision,
) -> bool {
    let expected_evidence_digest = digest_enable_evidence(
        evidence.installation_id(),
        evidence.expected_installation_revision(),
        evidence.package_digest(),
        evidence.component_set_digest(),
        evidence.configuration_digest(),
        evidence.capability_manifest_digest(),
        evidence.grant_set_snapshot_digest(),
        evidence.policy_admission_snapshot_digest(),
    );
    evidence.evidence_digest() == &expected_evidence_digest
        && evidence.installation_id() == aggregate.installation_id()
        && evidence.expected_installation_revision() == expected_revision
        && evidence.package_digest() == aggregate.package_pin().package_digest()
        && evidence.component_set_digest() == aggregate.package_pin().component_set_digest()
        && evidence.configuration_digest() == aggregate.configuration().digest()
        && evidence.capability_manifest_digest()
            == aggregate.package_pin().capability_manifest_digest()
}

fn revision_for_sequence(
    sequence: InstallationEventSequence,
) -> Result<InstallationRevision, InstallationReplayError> {
    if sequence.get() == u64::MAX {
        return Err(InstallationReplayError::SequenceOverflow);
    }
    InstallationRevision::parse(format!("installation-revision:{}", sequence.get()))
        .map_err(|_| InstallationReplayError::PostRevisionMismatch)
}

fn digest_configuration(
    tenant_id: &TenantId,
    entries: &BTreeMap<ConfigurationKey, ConfigurationValue>,
) -> Sha256Digest {
    let mut bytes = CONFIGURATION_DOMAIN.to_vec();
    encode_string(tenant_id.as_str(), &mut bytes);
    encode_count(entries.len(), &mut bytes);
    for (key, value) in entries {
        encode_string(key.as_str(), &mut bytes);
        match value {
            ConfigurationValue::Text(text) => {
                bytes.push(1);
                encode_string(text.as_str(), &mut bytes);
            }
            ConfigurationValue::Integer(value) => {
                bytes.push(2);
                bytes.extend_from_slice(&value.to_be_bytes());
            }
            ConfigurationValue::Boolean(value) => {
                bytes.push(3);
                bytes.push(u8::from(*value));
            }
            ConfigurationValue::Secret(secret) => {
                bytes.push(4);
                encode_string(secret.tenant_id().as_str(), &mut bytes);
                encode_string(secret.id().as_str(), &mut bytes);
            }
        }
    }
    Sha256Digest::from_bytes(&bytes)
}

#[allow(clippy::too_many_arguments)]
fn digest_enable_evidence(
    installation_id: &InstallationId,
    expected_installation_revision: &InstallationRevision,
    package_digest: &Sha256Digest,
    component_set_digest: &Sha256Digest,
    configuration_digest: &Sha256Digest,
    capability_manifest_digest: &Sha256Digest,
    grant_set_snapshot_digest: &Sha256Digest,
    policy_admission_snapshot_digest: &Sha256Digest,
) -> Sha256Digest {
    let mut bytes = ENABLE_EVIDENCE_DOMAIN.to_vec();
    for value in [
        installation_id.as_str(),
        expected_installation_revision.as_str(),
        package_digest.as_str(),
        component_set_digest.as_str(),
        configuration_digest.as_str(),
        capability_manifest_digest.as_str(),
        grant_set_snapshot_digest.as_str(),
        policy_admission_snapshot_digest.as_str(),
    ] {
        encode_string(value, &mut bytes);
    }
    Sha256Digest::from_bytes(&bytes)
}

fn encode_count(count: usize, output: &mut Vec<u8>) {
    output.extend_from_slice(&(count as u64).to_be_bytes());
}

fn encode_string(value: &str, output: &mut Vec<u8>) {
    encode_count(value.len(), output);
    output.extend_from_slice(value.as_bytes());
}

const fn is_ascii_identity_tail_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b':' | b'-')
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::panic, clippy::unwrap_used)]
mod tests {
    use super::*;

    fn parsed<T, E: fmt::Display>(result: Result<T, E>) -> T {
        match result {
            Ok(value) => value,
            Err(error) => panic!("fixture must parse: {error}"),
        }
    }

    fn tenant() -> TenantId {
        parsed(TenantId::parse("tenant:install-test"))
    }

    fn user() -> UserId {
        parsed(UserId::parse("user:install-test"))
    }

    fn installation_id() -> InstallationId {
        parsed(InstallationId::parse("installation:test"))
    }

    fn revision(value: u64) -> InstallationRevision {
        parsed(InstallationRevision::parse(format!(
            "installation-revision:{value}"
        )))
    }

    fn command_id(value: &str) -> InstallationCommandId {
        parsed(InstallationCommandId::parse(format!("cmd:{value}")))
    }

    fn digest(byte: char) -> Sha256Digest {
        parsed(Sha256Digest::parse(format!(
            "sha256:{}",
            byte.to_string().repeat(64)
        )))
    }

    fn configuration() -> InstallationConfiguration {
        let tenant = tenant();
        InstallationConfiguration::new(
            &tenant,
            vec![(
                parsed(ConfigurationKey::parse("mode")),
                ConfigurationValue::Text(parsed(NonSecretText::parse("readonly"))),
            )],
        )
        .unwrap()
    }

    fn package_pin() -> InstallationPackagePin {
        InstallationPackagePin::new(
            parsed(CatalogRevision::parse("catalog:test")),
            parsed(PackageId::parse("ustc.installation-test")),
            parsed(PackageVersion::parse("1.0.0")),
            digest('1'),
            vec![
                InstalledComponentPin::new(
                    parsed(ComponentId::parse("component:test")),
                    ComponentKind::NativeRustComponent,
                    parsed(ComponentVersion::parse("component-version:1")),
                    digest('2'),
                    parsed(ExecutionIdentity::parse("execution:test")),
                )
                .unwrap(),
            ],
            digest('3'),
            digest('4'),
        )
        .unwrap()
    }

    fn install_command(id: &str) -> InstallationCommand {
        InstallationCommand::install(
            command_id(id),
            installation_id(),
            tenant(),
            user(),
            package_pin(),
            configuration(),
        )
        .unwrap()
    }

    fn evidence(revision: u64, config: &InstallationConfiguration) -> EnablePreconditionEvidence {
        EnablePreconditionEvidence::from_authority_bindings(
            installation_id(),
            revision_for_sequence(InstallationEventSequence::new(revision).unwrap()).unwrap(),
            package_pin().package_digest().clone(),
            package_pin().component_set_digest().clone(),
            config.digest().clone(),
            package_pin().capability_manifest_digest().clone(),
            digest('5'),
            digest('6'),
        )
        .unwrap()
    }

    #[test]
    fn private_enable_evidence_is_checked_against_every_authority_binding() {
        let install = decide(None, &install_command("install")).unwrap();
        let aggregate = evolve(None, &install).unwrap();
        let valid_evidence = evidence(1, aggregate.configuration());
        let enable = InstallationCommand::enable(
            command_id("enable"),
            installation_id(),
            revision(1),
            valid_evidence.clone(),
        )
        .unwrap();
        let enabled_event =
            decide(Some(&aggregate), &enable).expect("valid private evidence enables");
        let enabled = evolve(Some(aggregate.clone()), &enabled_event).expect("enable evolves");
        assert_eq!(enabled.state(), ManagedInstallationState::Enabled);

        #[derive(Clone, Copy)]
        enum EvidenceFault {
            InstallationId,
            InstallationRevision,
            PackageDigest,
            ComponentSetDigest,
            ConfigurationDigest,
            CapabilityManifestDigest,
            GrantSetDigest,
            PolicyDigest,
        }

        for fault in [
            EvidenceFault::InstallationId,
            EvidenceFault::InstallationRevision,
            EvidenceFault::PackageDigest,
            EvidenceFault::ComponentSetDigest,
            EvidenceFault::ConfigurationDigest,
            EvidenceFault::CapabilityManifestDigest,
            EvidenceFault::GrantSetDigest,
            EvidenceFault::PolicyDigest,
        ] {
            let mut bad = valid_evidence.clone();
            match fault {
                EvidenceFault::InstallationId => {
                    bad.installation_id = parsed(InstallationId::parse("installation:other"));
                }
                EvidenceFault::InstallationRevision => {
                    bad.expected_installation_revision = revision(99)
                }
                EvidenceFault::PackageDigest => bad.package_digest = digest('9'),
                EvidenceFault::ComponentSetDigest => bad.component_set_digest = digest('9'),
                EvidenceFault::ConfigurationDigest => bad.configuration_digest = digest('9'),
                EvidenceFault::CapabilityManifestDigest => {
                    bad.capability_manifest_digest = digest('9')
                }
                EvidenceFault::GrantSetDigest => bad.grant_set_snapshot_digest = digest('9'),
                EvidenceFault::PolicyDigest => bad.policy_admission_snapshot_digest = digest('9'),
            }
            let command = InstallationCommand::enable(
                command_id("enable-mismatch"),
                installation_id(),
                revision(1),
                bad,
            )
            .unwrap();
            assert_eq!(
                decide(Some(&aggregate), &command),
                Err(InstallationDecisionError::EnableEvidenceMismatch)
            );
        }

        let disable =
            InstallationCommand::disable(command_id("disable"), installation_id(), revision(2))
                .unwrap();
        let disable_event =
            decide(Some(&enabled), &disable).expect("enabled installation disables");
        let disabled = evolve(Some(enabled), &disable_event).expect("disable evolves");
        assert_eq!(disabled.state(), ManagedInstallationState::Disabled);
    }

    #[test]
    fn replay_rejects_sequence_overflow_and_post_revision_forgery_without_public_event_mutator() {
        let mut event = decide(None, &install_command("install-forgery")).unwrap();
        event.post_revision = revision(99);
        assert_eq!(
            evolve(None, &event),
            Err(InstallationReplayError::PostRevisionMismatch)
        );

        let mut overflow = decide(None, &install_command("install-overflow")).unwrap();
        overflow.sequence = InstallationEventSequence::new(u64::MAX).unwrap();
        assert_eq!(
            replay([&overflow]),
            Err(InstallationReplayError::SequenceOverflow)
        );
    }

    #[test]
    fn replay_rejects_forged_installed_configuration_authority() {
        let mut event = decide(None, &install_command("install-forged-config-revision")).unwrap();
        if let InstallationEventPayload::Installed {
            configuration_revision,
            ..
        } = &mut event.payload
        {
            *configuration_revision = ConfigurationRevision::new(2).unwrap();
        } else {
            panic!("fixture must produce an installed event");
        }
        assert_eq!(
            evolve(None, &event),
            Err(InstallationReplayError::RedundantFieldMismatch)
        );

        let mut event = decide(None, &install_command("install-forged-config-tenant")).unwrap();
        let foreign_tenant = parsed(TenantId::parse("tenant:other-install-test"));
        let foreign_configuration = InstallationConfiguration::new(
            &foreign_tenant,
            vec![(
                parsed(ConfigurationKey::parse("mode")),
                ConfigurationValue::Text(parsed(NonSecretText::parse("readonly"))),
            )],
        )
        .unwrap();
        if let InstallationEventPayload::Installed { configuration, .. } = &mut event.payload {
            *configuration = foreign_configuration;
        } else {
            panic!("fixture must produce an installed event");
        }
        assert_eq!(
            evolve(None, &event),
            Err(InstallationReplayError::RedundantFieldMismatch)
        );
    }
}
