//! Typed, immutable, fail-closed capability registry domain for M20-B2.
//!
//! Authority lives in the closed Rust algebra and the validated read model. This
//! module never creates grants, installations, invocation snapshots, update plans
//! or runtime side effects. A registry definition alone does not create
//! authority; it is an input that downstream grant/invocation code may consume.
//!
//! Reuses the nominal [`crate::invocation::CapabilityId`],
//! [`crate::invocation::CapabilityClass`], [`crate::invocation::ConfirmationPolicy`]
//! and [`crate::invocation::Sha256Digest`] types. Invocation must not import this
//! module; the dependency direction is one-way.

use crate::invocation::{CapabilityClass, CapabilityId, ConfirmationPolicy, Sha256Digest};
use serde::Deserialize;
use std::collections::BTreeSet;
use std::error::Error;
use std::fmt;

const MAX_SOURCE_BYTES: usize = 1_048_576;
const MAX_CAPABILITY_ID_BYTES: usize = 128;
const MAX_REGISTRY_REVISION_SUFFIX: usize = 108;

const SCHEMA_VERSION: &str = "capability-registry/v1";
const REGISTRY_REVISION_PREFIX: &str = "capability-registry:";

const DEFINITION_DOMAIN: &[u8] = b"capability-definition/v0\0";
const REGISTRY_DOMAIN: &[u8] = b"capability-registry/v0\0";

/// Effect class carried by a capability definition.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EffectClass {
    Read,
    Write,
    Destructive,
    Linkout,
    Diagnostic,
}

/// Data class carried by a capability definition.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataClass {
    PublicCampusFact,
    TenantPrivateFact,
    UserProfile,
    Credential,
    Administrative,
}

/// Scope kind carried by a capability definition.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScopeKind {
    CampusPublic,
    TenantPrivateUser,
    OperatorAdministrative,
}

/// Auto-grant disposition stored on a capability definition.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AutoGrantDisposition {
    Never,
    FirstPartyDefaultOnly,
}

/// Lifecycle status of a capability definition.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapabilityStatus {
    Active,
    Deprecated,
    Revoked,
}

/// Derived risk class. Never stored in registry JSON or accepted from package
/// authors; it is a pure projection of `(EffectClass, DataClass)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RiskClass {
    Low,
    Medium,
    High,
    Critical,
}

/// Pure policy-change classification for one capability across two registries.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapabilityPolicyChange {
    Unchanged,
    Narrowed,
    ExpansionRequiresReapproval,
    RemovedOrRevoked,
}

/// Private-field registry revision with the checked grammar
/// `capability-registry:[A-Za-z0-9._:-]{1,108}`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CapabilityRegistryRevision(String);

/// Validation failure for a [`CapabilityRegistryRevision`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CapabilityRegistryRevisionError;

impl fmt::Display for CapabilityRegistryRevisionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("invalid capability registry revision")
    }
}

impl Error for CapabilityRegistryRevisionError {}

impl CapabilityRegistryRevision {
    /// Parse a registry revision against the checked grammar.
    pub fn parse(value: impl Into<String>) -> Result<Self, CapabilityRegistryRevisionError> {
        let value = value.into();
        if is_valid_registry_revision(&value) {
            Ok(Self(value))
        } else {
            Err(CapabilityRegistryRevisionError)
        }
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Immutable, validated capability definition. Constructible only by the
/// registry loader; all authority-bearing fields are private.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapabilityDefinition {
    id: CapabilityId,
    effect_class: EffectClass,
    data_class: DataClass,
    scope_kind: ScopeKind,
    auto_grant: AutoGrantDisposition,
    confirmation_default: ConfirmationPolicy,
    status: CapabilityStatus,
    definition_digest: Sha256Digest,
}

impl CapabilityDefinition {
    #[must_use]
    pub const fn id(&self) -> &CapabilityId {
        &self.id
    }

    #[must_use]
    pub const fn effect_class(&self) -> EffectClass {
        self.effect_class
    }

    #[must_use]
    pub const fn data_class(&self) -> DataClass {
        self.data_class
    }

    #[must_use]
    pub const fn scope_kind(&self) -> ScopeKind {
        self.scope_kind
    }

    #[must_use]
    pub const fn auto_grant(&self) -> AutoGrantDisposition {
        self.auto_grant
    }

    #[must_use]
    pub const fn confirmation_default(&self) -> ConfirmationPolicy {
        self.confirmation_default
    }

    #[must_use]
    pub const fn status(&self) -> CapabilityStatus {
        self.status
    }

    #[must_use]
    pub const fn definition_digest(&self) -> &Sha256Digest {
        &self.definition_digest
    }

    /// Derived risk class. The arms below cover the six load-admitted
    /// `(effect, data)` pairs exactly; the remaining recognized vocabulary is
    /// rejected by the loader, so the catch-all is unreachable for a constructed
    /// definition and `Critical` is kept only as the conservative bound so the
    /// match stays total without a production panic.
    #[must_use]
    pub const fn risk_class(&self) -> RiskClass {
        match (self.effect_class, self.data_class) {
            (EffectClass::Read, DataClass::PublicCampusFact) => RiskClass::Low,
            (EffectClass::Linkout, DataClass::PublicCampusFact) => RiskClass::Low,
            (EffectClass::Read, DataClass::TenantPrivateFact) => RiskClass::Medium,
            (EffectClass::Read, DataClass::UserProfile) => RiskClass::High,
            (EffectClass::Write, DataClass::TenantPrivateFact) => RiskClass::High,
            (EffectClass::Write, DataClass::UserProfile) => RiskClass::High,
            _ => RiskClass::Critical,
        }
    }

    /// Exact compatibility projection into the invocation `CapabilityClass`.
    #[must_use]
    pub const fn compatibility_class(&self) -> Option<CapabilityClass> {
        match (self.effect_class, self.data_class) {
            (EffectClass::Read, DataClass::PublicCampusFact) => Some(CapabilityClass::PublicRead),
            (EffectClass::Linkout, DataClass::PublicCampusFact) => {
                Some(CapabilityClass::PublicLinkout)
            }
            (EffectClass::Read, DataClass::TenantPrivateFact | DataClass::UserProfile) => {
                Some(CapabilityClass::TenantPrivateRead)
            }
            (EffectClass::Write, DataClass::TenantPrivateFact | DataClass::UserProfile) => {
                Some(CapabilityClass::TenantPrivateWrite)
            }
            _ => None,
        }
    }

    /// Exact first-party default auto-grant candidacy predicate.
    #[must_use]
    pub const fn is_first_party_default_auto_grant_candidate(&self) -> bool {
        matches!(self.auto_grant, AutoGrantDisposition::FirstPartyDefaultOnly)
            && matches!(self.status, CapabilityStatus::Active)
            && matches!(self.scope_kind, ScopeKind::CampusPublic)
            && matches!(self.confirmation_default, ConfirmationPolicy::Allow)
            && matches!(
                (self.effect_class, self.data_class),
                (
                    EffectClass::Read | EffectClass::Linkout,
                    DataClass::PublicCampusFact
                )
            )
    }
}

/// Immutable, validated capability registry read model. Definitions are stored
/// sorted bytewise by capability id; construction is loader-checked.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapabilityRegistry {
    revision: CapabilityRegistryRevision,
    definitions: Vec<CapabilityDefinition>,
    registry_digest: Sha256Digest,
}

impl CapabilityRegistry {
    #[must_use]
    pub const fn registry_revision(&self) -> &CapabilityRegistryRevision {
        &self.revision
    }

    #[must_use]
    pub fn definitions(&self) -> &[CapabilityDefinition] {
        &self.definitions
    }

    #[must_use]
    pub const fn registry_digest(&self) -> &Sha256Digest {
        &self.registry_digest
    }

    /// Exact bytewise-id lookup in `O(log n)`.
    #[must_use]
    pub fn find(&self, id: &CapabilityId) -> Option<&CapabilityDefinition> {
        self.definitions
            .binary_search_by(|definition| {
                definition
                    .id()
                    .as_str()
                    .as_bytes()
                    .cmp(id.as_str().as_bytes())
            })
            .ok()
            .and_then(|index| self.definitions.get(index))
    }
}

/// Failure before a validated capability registry exists.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapabilityRegistryLoadError {
    SourceTooLarge,
    JsonRejected,
    InvalidSchemaVersion,
    InvalidRegistryRevision,
    InvalidCapabilityId,
    DuplicateCapabilityId,
    ForbiddenCombination,
    IncoherentDefinition,
}

impl fmt::Display for CapabilityRegistryLoadError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SourceTooLarge => {
                formatter.write_str("capability registry source exceeds its bound")
            }
            Self::JsonRejected => formatter.write_str("capability registry JSON was rejected"),
            Self::InvalidSchemaVersion => {
                formatter.write_str("capability registry schema version was rejected")
            }
            Self::InvalidRegistryRevision => {
                formatter.write_str("capability registry revision was rejected")
            }
            Self::InvalidCapabilityId => {
                formatter.write_str("capability registry identifier was rejected")
            }
            Self::DuplicateCapabilityId => {
                formatter.write_str("capability registry contains a duplicate identifier")
            }
            Self::ForbiddenCombination => {
                formatter.write_str("capability registry combination is forbidden")
            }
            Self::IncoherentDefinition => {
                formatter.write_str("capability registry definition is incoherent")
            }
        }
    }
}

impl Error for CapabilityRegistryLoadError {}

/// Decode and validate one untrusted capability registry without creating grant
/// or invocation authority.
///
/// # Errors
///
/// Returns a [`CapabilityRegistryLoadError`] with a stable category for every
/// rejected source. `Debug`/`Display` of the error never expose source fragments
/// or rejected values.
pub fn load_capability_registry(
    source: &[u8],
) -> Result<CapabilityRegistry, CapabilityRegistryLoadError> {
    if source.len() > MAX_SOURCE_BYTES {
        return Err(CapabilityRegistryLoadError::SourceTooLarge);
    }
    let raw = serde_json::from_slice::<RawRegistry>(source)
        .map_err(|_| CapabilityRegistryLoadError::JsonRejected)?;
    if raw.schema_version != SCHEMA_VERSION {
        return Err(CapabilityRegistryLoadError::InvalidSchemaVersion);
    }
    let revision = CapabilityRegistryRevision::parse(raw.registry_revision)
        .map_err(|_| CapabilityRegistryLoadError::InvalidRegistryRevision)?;

    let mut definitions = Vec::with_capacity(raw.capabilities.len());
    let mut seen = BTreeSet::new();
    for raw_capability in raw.capabilities {
        if !is_valid_registry_capability_id(&raw_capability.id) {
            return Err(CapabilityRegistryLoadError::InvalidCapabilityId);
        }
        let id = CapabilityId::parse(raw_capability.id)
            .map_err(|_| CapabilityRegistryLoadError::InvalidCapabilityId)?;
        if !seen.insert(id.clone()) {
            return Err(CapabilityRegistryLoadError::DuplicateCapabilityId);
        }
        let effect = map_effect_class(raw_capability.effect_class);
        let data = map_data_class(raw_capability.data_class);
        let scope = map_scope_kind(raw_capability.scope_kind);
        let auto_grant = map_auto_grant(raw_capability.auto_grant);
        let confirmation = map_confirmation_policy(raw_capability.confirmation_default);
        let status = map_capability_status(raw_capability.status);
        validate_coherence(effect, data, scope, auto_grant, confirmation, status)?;
        let definition_digest =
            compute_definition_digest(&id, effect, data, scope, auto_grant, confirmation, status);
        definitions.push(CapabilityDefinition {
            id,
            effect_class: effect,
            data_class: data,
            scope_kind: scope,
            auto_grant,
            confirmation_default: confirmation,
            status,
            definition_digest,
        });
    }

    definitions.sort_by(|left, right| {
        left.id()
            .as_str()
            .as_bytes()
            .cmp(right.id().as_str().as_bytes())
    });
    let registry_digest = compute_registry_digest(&revision, &definitions);
    Ok(CapabilityRegistry {
        revision,
        definitions,
        registry_digest,
    })
}

/// Pure policy-change comparison for one capability id across two validated
/// registries.
#[must_use]
pub fn compare_capability_policy(
    old: &CapabilityRegistry,
    new: &CapabilityRegistry,
    id: &CapabilityId,
) -> CapabilityPolicyChange {
    let old_definition = old.find(id);
    let new_definition = new.find(id);
    match (old_definition, new_definition) {
        (None, None) => CapabilityPolicyChange::Unchanged,
        (None, Some(_)) => CapabilityPolicyChange::ExpansionRequiresReapproval,
        (Some(_), None) => CapabilityPolicyChange::RemovedOrRevoked,
        (Some(old_definition), Some(new_definition)) => {
            if matches!(new_definition.status(), CapabilityStatus::Revoked) {
                CapabilityPolicyChange::RemovedOrRevoked
            } else if old_definition.definition_digest() == new_definition.definition_digest() {
                CapabilityPolicyChange::Unchanged
            } else {
                classify_policy_change(old_definition, new_definition)
            }
        }
    }
}

fn classify_policy_change(
    old: &CapabilityDefinition,
    new: &CapabilityDefinition,
) -> CapabilityPolicyChange {
    if matches!(
        old.status(),
        CapabilityStatus::Deprecated | CapabilityStatus::Revoked
    ) && matches!(new.status(), CapabilityStatus::Active)
    {
        return CapabilityPolicyChange::ExpansionRequiresReapproval;
    }
    if matches!(old.status(), CapabilityStatus::Active)
        && matches!(new.status(), CapabilityStatus::Deprecated)
    {
        return CapabilityPolicyChange::Narrowed;
    }
    let auto_expanded = matches!(old.auto_grant(), AutoGrantDisposition::Never)
        && matches!(
            new.auto_grant(),
            AutoGrantDisposition::FirstPartyDefaultOnly
        );
    let confirmation_expanded = matches!(old.confirmation_default(), ConfirmationPolicy::Ask)
        && matches!(new.confirmation_default(), ConfirmationPolicy::Allow);
    let axis_changed = old.effect_class() != new.effect_class()
        || old.data_class() != new.data_class()
        || old.scope_kind() != new.scope_kind();
    if auto_expanded || confirmation_expanded || axis_changed {
        return CapabilityPolicyChange::ExpansionRequiresReapproval;
    }
    let auto_narrowed = matches!(
        old.auto_grant(),
        AutoGrantDisposition::FirstPartyDefaultOnly
    ) && matches!(new.auto_grant(), AutoGrantDisposition::Never);
    let confirmation_narrowed = matches!(old.confirmation_default(), ConfirmationPolicy::Allow)
        && matches!(new.confirmation_default(), ConfirmationPolicy::Ask);
    if auto_narrowed || confirmation_narrowed {
        return CapabilityPolicyChange::Narrowed;
    }
    CapabilityPolicyChange::ExpansionRequiresReapproval
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct RawRegistry {
    schema_version: String,
    registry_revision: String,
    capabilities: Vec<RawCapability>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct RawCapability {
    id: String,
    effect_class: RawEffectClass,
    data_class: RawDataClass,
    scope_kind: RawScopeKind,
    auto_grant: RawAutoGrantDisposition,
    confirmation_default: RawConfirmationPolicy,
    status: RawCapabilityStatus,
}

#[derive(Deserialize)]
enum RawEffectClass {
    Read,
    Write,
    Destructive,
    Linkout,
    Diagnostic,
}

#[derive(Deserialize)]
enum RawDataClass {
    PublicCampusFact,
    TenantPrivateFact,
    UserProfile,
    Credential,
    Administrative,
}

#[derive(Deserialize)]
enum RawScopeKind {
    CampusPublic,
    TenantPrivateUser,
    OperatorAdministrative,
}

#[derive(Deserialize)]
enum RawAutoGrantDisposition {
    Never,
    FirstPartyDefaultOnly,
}

#[derive(Deserialize)]
enum RawConfirmationPolicy {
    Allow,
    Ask,
}

#[derive(Deserialize)]
enum RawCapabilityStatus {
    Active,
    Deprecated,
    Revoked,
}

fn map_effect_class(raw: RawEffectClass) -> EffectClass {
    match raw {
        RawEffectClass::Read => EffectClass::Read,
        RawEffectClass::Write => EffectClass::Write,
        RawEffectClass::Destructive => EffectClass::Destructive,
        RawEffectClass::Linkout => EffectClass::Linkout,
        RawEffectClass::Diagnostic => EffectClass::Diagnostic,
    }
}

fn map_data_class(raw: RawDataClass) -> DataClass {
    match raw {
        RawDataClass::PublicCampusFact => DataClass::PublicCampusFact,
        RawDataClass::TenantPrivateFact => DataClass::TenantPrivateFact,
        RawDataClass::UserProfile => DataClass::UserProfile,
        RawDataClass::Credential => DataClass::Credential,
        RawDataClass::Administrative => DataClass::Administrative,
    }
}

fn map_scope_kind(raw: RawScopeKind) -> ScopeKind {
    match raw {
        RawScopeKind::CampusPublic => ScopeKind::CampusPublic,
        RawScopeKind::TenantPrivateUser => ScopeKind::TenantPrivateUser,
        RawScopeKind::OperatorAdministrative => ScopeKind::OperatorAdministrative,
    }
}

fn map_auto_grant(raw: RawAutoGrantDisposition) -> AutoGrantDisposition {
    match raw {
        RawAutoGrantDisposition::Never => AutoGrantDisposition::Never,
        RawAutoGrantDisposition::FirstPartyDefaultOnly => {
            AutoGrantDisposition::FirstPartyDefaultOnly
        }
    }
}

fn map_confirmation_policy(raw: RawConfirmationPolicy) -> ConfirmationPolicy {
    match raw {
        RawConfirmationPolicy::Allow => ConfirmationPolicy::Allow,
        RawConfirmationPolicy::Ask => ConfirmationPolicy::Ask,
    }
}

fn map_capability_status(raw: RawCapabilityStatus) -> CapabilityStatus {
    match raw {
        RawCapabilityStatus::Active => CapabilityStatus::Active,
        RawCapabilityStatus::Deprecated => CapabilityStatus::Deprecated,
        RawCapabilityStatus::Revoked => CapabilityStatus::Revoked,
    }
}

fn validate_coherence(
    effect: EffectClass,
    data: DataClass,
    scope: ScopeKind,
    auto_grant: AutoGrantDisposition,
    confirmation: ConfirmationPolicy,
    status: CapabilityStatus,
) -> Result<(), CapabilityRegistryLoadError> {
    // Admitted `(effect, data)` pairs reject forbidden MVP vocabulary
    // (Destructive, Diagnostic, Credential, Administrative) and unlisted pairs.
    if !is_admitted_pair(effect, data) {
        return Err(CapabilityRegistryLoadError::ForbiddenCombination);
    }
    // OperatorAdministrative scope is recognized vocabulary but forbidden in MVP.
    if matches!(scope, ScopeKind::OperatorAdministrative) {
        return Err(CapabilityRegistryLoadError::ForbiddenCombination);
    }
    let expected_scope = match data {
        DataClass::PublicCampusFact => ScopeKind::CampusPublic,
        DataClass::TenantPrivateFact | DataClass::UserProfile => ScopeKind::TenantPrivateUser,
        _ => return Err(CapabilityRegistryLoadError::ForbiddenCombination),
    };
    if scope != expected_scope {
        return Err(CapabilityRegistryLoadError::IncoherentDefinition);
    }
    let expected_confirmation = match scope {
        ScopeKind::CampusPublic => ConfirmationPolicy::Allow,
        ScopeKind::TenantPrivateUser => ConfirmationPolicy::Ask,
        ScopeKind::OperatorAdministrative => {
            return Err(CapabilityRegistryLoadError::ForbiddenCombination);
        }
    };
    if confirmation != expected_confirmation {
        return Err(CapabilityRegistryLoadError::IncoherentDefinition);
    }
    let is_public_active_readlike = matches!(status, CapabilityStatus::Active)
        && matches!(scope, ScopeKind::CampusPublic)
        && matches!(confirmation, ConfirmationPolicy::Allow)
        && matches!(
            (effect, data),
            (
                EffectClass::Read | EffectClass::Linkout,
                DataClass::PublicCampusFact
            )
        );
    let expected_auto_grant = if is_public_active_readlike {
        AutoGrantDisposition::FirstPartyDefaultOnly
    } else {
        AutoGrantDisposition::Never
    };
    if auto_grant != expected_auto_grant {
        return Err(CapabilityRegistryLoadError::IncoherentDefinition);
    }
    Ok(())
}

fn is_admitted_pair(effect: EffectClass, data: DataClass) -> bool {
    matches!(
        (effect, data),
        (EffectClass::Read, DataClass::PublicCampusFact)
            | (EffectClass::Linkout, DataClass::PublicCampusFact)
            | (EffectClass::Read, DataClass::TenantPrivateFact)
            | (EffectClass::Read, DataClass::UserProfile)
            | (EffectClass::Write, DataClass::TenantPrivateFact)
            | (EffectClass::Write, DataClass::UserProfile)
    )
}

fn is_valid_registry_revision(value: &str) -> bool {
    let Some(rest) = value.strip_prefix(REGISTRY_REVISION_PREFIX) else {
        return false;
    };
    let bytes = rest.as_bytes();
    if bytes.is_empty() || bytes.len() > MAX_REGISTRY_REVISION_SUFFIX {
        return false;
    }
    bytes
        .iter()
        .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b':' | b'-'))
}

fn is_valid_registry_capability_id(value: &str) -> bool {
    let bytes = value.as_bytes();
    if bytes.is_empty() || bytes.len() > MAX_CAPABILITY_ID_BYTES || !value.is_ascii() {
        return false;
    }
    let mut segments = bytes.split(|&byte| byte == b'.');
    let first = match segments.next() {
        Some(segment) => segment,
        None => return false,
    };
    let mut first_bytes = first.iter();
    match first_bytes.next() {
        Some(&byte) if byte.is_ascii_lowercase() => {}
        _ => return false,
    }
    if !first_bytes.all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit()) {
        return false;
    }
    let mut count = 0usize;
    for segment in segments {
        count += 1;
        if count > 7 {
            return false;
        }
        if segment.is_empty() {
            return false;
        }
        if !segment.iter().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'_' | b'-')
        }) {
            return false;
        }
    }
    count >= 1
}

fn compute_definition_digest(
    id: &CapabilityId,
    effect: EffectClass,
    data: DataClass,
    scope: ScopeKind,
    auto_grant: AutoGrantDisposition,
    confirmation: ConfirmationPolicy,
    status: CapabilityStatus,
) -> Sha256Digest {
    let mut bytes = DEFINITION_DOMAIN.to_vec();
    encode_string(id.as_str(), &mut bytes);
    bytes.push(effect_class_tag(effect));
    bytes.push(data_class_tag(data));
    bytes.push(scope_kind_tag(scope));
    bytes.push(auto_grant_tag(auto_grant));
    bytes.push(confirmation_policy_tag(confirmation));
    bytes.push(capability_status_tag(status));
    Sha256Digest::from_bytes(&bytes)
}

fn compute_registry_digest(
    revision: &CapabilityRegistryRevision,
    definitions: &[CapabilityDefinition],
) -> Sha256Digest {
    let mut bytes = REGISTRY_DOMAIN.to_vec();
    encode_string(revision.as_str(), &mut bytes);
    encode_count(definitions.len(), &mut bytes);
    for definition in definitions {
        encode_string(definition.id().as_str(), &mut bytes);
        encode_string(definition.definition_digest().as_str(), &mut bytes);
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

// Explicit fixed `u8` enum tags for canonical hashing. Reordering input JSON
// cannot change any digest, and any authority-field change must change the
// corresponding digest.

const fn effect_class_tag(value: EffectClass) -> u8 {
    match value {
        EffectClass::Read => 1,
        EffectClass::Write => 2,
        EffectClass::Destructive => 3,
        EffectClass::Linkout => 4,
        EffectClass::Diagnostic => 5,
    }
}

const fn data_class_tag(value: DataClass) -> u8 {
    match value {
        DataClass::PublicCampusFact => 1,
        DataClass::TenantPrivateFact => 2,
        DataClass::UserProfile => 3,
        DataClass::Credential => 4,
        DataClass::Administrative => 5,
    }
}

const fn scope_kind_tag(value: ScopeKind) -> u8 {
    match value {
        ScopeKind::CampusPublic => 1,
        ScopeKind::TenantPrivateUser => 2,
        ScopeKind::OperatorAdministrative => 3,
    }
}

const fn auto_grant_tag(value: AutoGrantDisposition) -> u8 {
    match value {
        AutoGrantDisposition::Never => 1,
        AutoGrantDisposition::FirstPartyDefaultOnly => 2,
    }
}

const fn confirmation_policy_tag(value: ConfirmationPolicy) -> u8 {
    match value {
        ConfirmationPolicy::Allow => 1,
        ConfirmationPolicy::Ask => 2,
    }
}

const fn capability_status_tag(value: CapabilityStatus) -> u8 {
    match value {
        CapabilityStatus::Active => 1,
        CapabilityStatus::Deprecated => 2,
        CapabilityStatus::Revoked => 3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enum_tags_are_explicit_and_stable() {
        assert_eq!(effect_class_tag(EffectClass::Read), 1);
        assert_eq!(effect_class_tag(EffectClass::Write), 2);
        assert_eq!(effect_class_tag(EffectClass::Destructive), 3);
        assert_eq!(effect_class_tag(EffectClass::Linkout), 4);
        assert_eq!(effect_class_tag(EffectClass::Diagnostic), 5);
        assert_eq!(data_class_tag(DataClass::PublicCampusFact), 1);
        assert_eq!(data_class_tag(DataClass::TenantPrivateFact), 2);
        assert_eq!(data_class_tag(DataClass::UserProfile), 3);
        assert_eq!(scope_kind_tag(ScopeKind::CampusPublic), 1);
        assert_eq!(scope_kind_tag(ScopeKind::TenantPrivateUser), 2);
        assert_eq!(scope_kind_tag(ScopeKind::OperatorAdministrative), 3);
        assert_eq!(auto_grant_tag(AutoGrantDisposition::Never), 1);
        assert_eq!(
            auto_grant_tag(AutoGrantDisposition::FirstPartyDefaultOnly),
            2
        );
        assert_eq!(confirmation_policy_tag(ConfirmationPolicy::Allow), 1);
        assert_eq!(confirmation_policy_tag(ConfirmationPolicy::Ask), 2);
        assert_eq!(capability_status_tag(CapabilityStatus::Active), 1);
        assert_eq!(capability_status_tag(CapabilityStatus::Deprecated), 2);
        assert_eq!(capability_status_tag(CapabilityStatus::Revoked), 3);
    }

    #[test]
    fn registry_revision_grammar_is_enforced() {
        assert!(CapabilityRegistryRevision::parse("capability-registry:2026-07-29-01").is_ok());
        assert!(CapabilityRegistryRevision::parse("capability-registry:a").is_ok());
        assert!(CapabilityRegistryRevision::parse("capability-registry:a.b:c-d_e").is_ok());
        assert!(CapabilityRegistryRevision::parse("capability-registry:").is_err());
        assert!(CapabilityRegistryRevision::parse("capability-registry").is_err());
        assert!(CapabilityRegistryRevision::parse("catalog:v1").is_err());
        assert!(CapabilityRegistryRevision::parse("capability-registry:space value").is_err());
    }

    #[test]
    fn capability_id_grammar_is_enforced() {
        assert!(is_valid_registry_capability_id("campus.public_rules.read"));
        assert!(is_valid_registry_capability_id(
            "user.own_academic_snapshot.read"
        ));
        assert!(is_valid_registry_capability_id("a.b"));
        assert!(is_valid_registry_capability_id("x.y-z_w.0"));
        assert!(!is_valid_registry_capability_id("campus"));
        assert!(!is_valid_registry_capability_id("Campus.public.read"));
        assert!(!is_valid_registry_capability_id("campus..read"));
        assert!(!is_valid_registry_capability_id(".campus.read"));
        assert!(!is_valid_registry_capability_id("campus.public."));
        assert!(!is_valid_registry_capability_id("campus .public.read"));
    }
}
