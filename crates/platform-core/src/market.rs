//! Typed package-manifest validation and deterministic catalog metadata for M20-B1-1.

pub mod capability;

use crate::invocation::{
    CapabilityId, CatalogRevision, ComponentKind, PackageId, PackageVersion, Sha256Digest,
};
use serde::Deserialize;
use serde::de::{self, MapAccess, Visitor};
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt;

const MAX_SOURCE_BYTES: usize = 1_048_576;
const MAX_PACKAGE_ID_BYTES: usize = 256;
const MAX_PUBLISHER_BYTES: usize = 128;
const MAX_DISPLAY_NAME_BYTES: usize = 256;
const MAX_DESCRIPTION_BYTES: usize = 4_096;
const MAX_COMPONENTS: usize = 64;
const MAX_COMPONENT_PATH_BYTES: usize = 512;
const MAX_COMPONENT_MODE_BYTES: usize = 64;
const MAX_CAPABILITIES: usize = 64;
const MAX_SOURCE_POLICY_ENTRIES: usize = 32;
const MAX_SOURCE_POLICY_VALUE_BYTES: usize = 4_096;

const PACKAGE_DOMAIN: &[u8] = b"market-package-manifest/v0\0";
const COMPONENTS_DOMAIN: &[u8] = b"market-component-declarations/v0\0";
const CAPABILITIES_DOMAIN: &[u8] = b"market-capability-manifest/v0\0";
const SOURCE_POLICY_DOMAIN: &[u8] = b"market-source-policy/v0\0";
const CATALOG_DOMAIN: &[u8] = b"market-catalog-read-model/v0\0";

/// Stable manifest field categories. Rejected source text is never retained.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackageField {
    Source,
    PackageId,
    PackageVersion,
    Publisher,
    DisplayName,
    Description,
    Tier,
    InstallPolicy,
    Components,
    ComponentPath,
    ComponentMode,
    Capabilities,
    SourcePolicy,
    SourcePolicyKey,
    SourcePolicyValue,
    ImplementationStatus,
}

/// Stable semantic rejection classes for a syntactically decoded manifest.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackageValidationErrorKind {
    Empty,
    TooLong,
    TooMany,
    InvalidFormat,
    Duplicate,
    Inconsistent,
}

/// One semantic package-manifest validation failure without source-derived diagnostics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PackageValidationError {
    field: PackageField,
    kind: PackageValidationErrorKind,
}

impl PackageValidationError {
    #[must_use]
    pub const fn field(&self) -> PackageField {
        self.field
    }

    #[must_use]
    pub const fn kind(&self) -> PackageValidationErrorKind {
        self.kind
    }
}

impl fmt::Display for PackageValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "package manifest validation failed: {:?}/{:?}",
            self.field, self.kind
        )
    }
}

impl Error for PackageValidationError {}

/// Failure before a validated immutable manifest exists.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackageLoadError {
    SourceTooLarge,
    JsonRejected,
    InvalidManifest(PackageValidationError),
}

impl fmt::Display for PackageLoadError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SourceTooLarge => {
                formatter.write_str("package manifest source exceeds its bound")
            }
            Self::JsonRejected => formatter.write_str("package manifest JSON was rejected"),
            Self::InvalidManifest(error) => error.fmt(formatter),
        }
    }
}

impl Error for PackageLoadError {}

/// Reviewed package tier declared by the manifest.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackageTier {
    FirstParty,
    VerifiedCommunityText,
    VerifiedRemoteMcp,
}

/// Honest implementation claim carried by package metadata.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImplementationStatus {
    Planned,
    Development,
    Implemented,
}

/// Catalog install-policy class. This is policy input, never installation state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallPolicyClass {
    FirstPartySystemPlugin,
    UserInstalledPlugin,
}

/// Immutable validated install-policy declaration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InstallPolicy {
    class: InstallPolicyClass,
    default_installed: bool,
    default_enabled: bool,
    user_disable_allowed: bool,
}

impl InstallPolicy {
    #[must_use]
    pub const fn class(&self) -> InstallPolicyClass {
        self.class
    }

    #[must_use]
    pub const fn default_installed(&self) -> bool {
        self.default_installed
    }

    #[must_use]
    pub const fn default_enabled(&self) -> bool {
        self.default_enabled
    }

    #[must_use]
    pub const fn user_disable_allowed(&self) -> bool {
        self.user_disable_allowed
    }
}

/// One schema-level component declaration. It is not artifact or execution evidence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentDeclaration {
    kind: ComponentKind,
    path: String,
    mode: Option<String>,
}

impl ComponentDeclaration {
    #[must_use]
    pub const fn kind(&self) -> ComponentKind {
        self.kind
    }

    #[must_use]
    pub fn path(&self) -> &str {
        &self.path
    }

    #[must_use]
    pub fn mode(&self) -> Option<&str> {
        self.mode.as_deref()
    }
}

/// Immutable, schema-valid package declaration. It is not publication or runtime authority.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatedPackageManifest {
    package_id: PackageId,
    package_version: PackageVersion,
    publisher: String,
    tier: PackageTier,
    display_name: String,
    description: Option<String>,
    implementation_status: ImplementationStatus,
    install_policy: InstallPolicy,
    components: Vec<ComponentDeclaration>,
    capabilities: Vec<CapabilityId>,
    source_policy: BTreeMap<String, String>,
    package_digest: Sha256Digest,
    component_declaration_set_digest: Sha256Digest,
    capability_manifest_digest: Sha256Digest,
    source_policy_digest: Sha256Digest,
}

impl ValidatedPackageManifest {
    #[must_use]
    pub const fn package_id(&self) -> &PackageId {
        &self.package_id
    }

    #[must_use]
    pub const fn package_version(&self) -> &PackageVersion {
        &self.package_version
    }

    #[must_use]
    pub fn publisher(&self) -> &str {
        &self.publisher
    }

    #[must_use]
    pub const fn tier(&self) -> PackageTier {
        self.tier
    }

    #[must_use]
    pub fn display_name(&self) -> &str {
        &self.display_name
    }

    #[must_use]
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    #[must_use]
    pub const fn implementation_status(&self) -> ImplementationStatus {
        self.implementation_status
    }

    #[must_use]
    pub const fn install_policy(&self) -> &InstallPolicy {
        &self.install_policy
    }

    #[must_use]
    pub fn components(&self) -> &[ComponentDeclaration] {
        &self.components
    }

    #[must_use]
    pub fn capabilities(&self) -> &[CapabilityId] {
        &self.capabilities
    }

    #[must_use]
    pub const fn source_policy(&self) -> &BTreeMap<String, String> {
        &self.source_policy
    }

    #[must_use]
    pub const fn package_digest(&self) -> &Sha256Digest {
        &self.package_digest
    }

    #[must_use]
    pub const fn component_declaration_set_digest(&self) -> &Sha256Digest {
        &self.component_declaration_set_digest
    }

    #[must_use]
    pub const fn capability_manifest_digest(&self) -> &Sha256Digest {
        &self.capability_manifest_digest
    }

    #[must_use]
    pub const fn source_policy_digest(&self) -> &Sha256Digest {
        &self.source_policy_digest
    }
}

/// Deterministic anonymous metadata projection for one reviewed catalog revision.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CatalogReadModel {
    catalog_revision: CatalogRevision,
    packages: Vec<ValidatedPackageManifest>,
    catalog_digest: Sha256Digest,
}

/// Catalog read-model construction failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CatalogReadModelError {
    DuplicatePackageRevision,
}

impl fmt::Display for CatalogReadModelError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("catalog read model contains a duplicate package revision")
    }
}

impl Error for CatalogReadModelError {}

impl CatalogReadModel {
    pub fn new(
        catalog_revision: CatalogRevision,
        mut packages: Vec<ValidatedPackageManifest>,
    ) -> Result<Self, CatalogReadModelError> {
        packages.sort_by(compare_packages);
        if packages
            .windows(2)
            .any(|pair| same_package_revision(&pair[0], &pair[1]))
        {
            return Err(CatalogReadModelError::DuplicatePackageRevision);
        }

        let mut canonical = CATALOG_DOMAIN.to_vec();
        encode_string(catalog_revision.as_str(), &mut canonical);
        encode_count(packages.len(), &mut canonical);
        for package in &packages {
            encode_string(package.package_digest().as_str(), &mut canonical);
        }
        let catalog_digest = Sha256Digest::from_bytes(&canonical);

        Ok(Self {
            catalog_revision,
            packages,
            catalog_digest,
        })
    }

    #[must_use]
    pub const fn catalog_revision(&self) -> &CatalogRevision {
        &self.catalog_revision
    }

    #[must_use]
    pub fn packages(&self) -> &[ValidatedPackageManifest] {
        &self.packages
    }

    #[must_use]
    pub const fn catalog_digest(&self) -> &Sha256Digest {
        &self.catalog_digest
    }

    #[must_use]
    pub fn find(
        &self,
        package_id: &PackageId,
        package_version: &PackageVersion,
    ) -> Option<&ValidatedPackageManifest> {
        let version = package_version.as_str();
        self.packages
            .binary_search_by(|package| {
                package
                    .package_id()
                    .as_str()
                    .cmp(package_id.as_str())
                    .then_with(|| package.package_version().as_str().cmp(&version))
            })
            .ok()
            .and_then(|index| self.packages.get(index))
    }
}

/// Decode and validate one untrusted package manifest without creating publication authority.
pub fn load_package_manifest(source: &[u8]) -> Result<ValidatedPackageManifest, PackageLoadError> {
    if source.len() > MAX_SOURCE_BYTES {
        return Err(PackageLoadError::SourceTooLarge);
    }
    let raw = serde_json::from_slice::<RawPackageManifest>(source)
        .map_err(|_| PackageLoadError::JsonRejected)?;
    validate_manifest(raw).map_err(PackageLoadError::InvalidManifest)
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct RawPackageManifest {
    id: String,
    version: String,
    publisher: String,
    tier: RawPackageTier,
    display_name: String,
    #[serde(default, deserialize_with = "deserialize_present_string")]
    description: Option<String>,
    implementation_status: RawImplementationStatus,
    install_policy: RawInstallPolicy,
    components: Vec<RawComponentDeclaration>,
    capabilities: Vec<String>,
    source_policy: UniqueStringMap,
}

#[derive(Deserialize)]
enum RawPackageTier {
    FirstParty,
    VerifiedCommunityText,
    VerifiedRemoteMcp,
}

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
enum RawImplementationStatus {
    Planned,
    Development,
    Implemented,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct RawInstallPolicy {
    class: RawInstallPolicyClass,
    default_installed: bool,
    default_enabled: bool,
    user_disable_allowed: bool,
}

#[derive(Deserialize)]
enum RawInstallPolicyClass {
    FirstPartySystemPlugin,
    UserInstalledPlugin,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct RawComponentDeclaration {
    #[serde(rename = "type")]
    kind: RawComponentKind,
    path: String,
    #[serde(default, deserialize_with = "deserialize_present_string")]
    mode: Option<String>,
}

fn deserialize_present_string<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    String::deserialize(deserializer).map(Some)
}

#[derive(Deserialize)]
enum RawComponentKind {
    SkillComponent,
    DeclarativeResourcePack,
    McpServerComponent,
    NativeRustComponent,
}

struct UniqueStringMap(BTreeMap<String, String>);

impl<'de> Deserialize<'de> for UniqueStringMap {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct UniqueStringMapVisitor;

        impl<'de> Visitor<'de> for UniqueStringMapVisitor {
            type Value = UniqueStringMap;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("an object with unique string keys and string values")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut values = BTreeMap::new();
                while let Some((key, value)) = map.next_entry::<String, String>()? {
                    if values.insert(key, value).is_some() {
                        return Err(de::Error::custom("duplicate source-policy key"));
                    }
                }
                Ok(UniqueStringMap(values))
            }
        }

        deserializer.deserialize_map(UniqueStringMapVisitor)
    }
}

fn validate_manifest(
    raw: RawPackageManifest,
) -> Result<ValidatedPackageManifest, PackageValidationError> {
    if !is_valid_package_id(&raw.id) {
        return Err(invalid(
            PackageField::PackageId,
            PackageValidationErrorKind::InvalidFormat,
        ));
    }
    let package_id = PackageId::parse(raw.id).map_err(|_| {
        invalid(
            PackageField::PackageId,
            PackageValidationErrorKind::InvalidFormat,
        )
    })?;
    if !is_valid_release_version(&raw.version) {
        return Err(invalid(
            PackageField::PackageVersion,
            PackageValidationErrorKind::InvalidFormat,
        ));
    }
    let package_version = PackageVersion::parse(&raw.version).map_err(|_| {
        invalid(
            PackageField::PackageVersion,
            PackageValidationErrorKind::InvalidFormat,
        )
    })?;
    if !is_valid_publisher(&raw.publisher) {
        return Err(invalid(
            PackageField::Publisher,
            classify_text(&raw.publisher, MAX_PUBLISHER_BYTES),
        ));
    }
    validate_text(
        &raw.display_name,
        MAX_DISPLAY_NAME_BYTES,
        PackageField::DisplayName,
    )?;
    if let Some(description) = &raw.description {
        validate_text(
            description,
            MAX_DESCRIPTION_BYTES,
            PackageField::Description,
        )?;
    }

    let tier = match raw.tier {
        RawPackageTier::FirstParty => PackageTier::FirstParty,
        RawPackageTier::VerifiedCommunityText => PackageTier::VerifiedCommunityText,
        RawPackageTier::VerifiedRemoteMcp => PackageTier::VerifiedRemoteMcp,
    };
    let implementation_status = match raw.implementation_status {
        RawImplementationStatus::Planned => ImplementationStatus::Planned,
        RawImplementationStatus::Development => ImplementationStatus::Development,
        RawImplementationStatus::Implemented => ImplementationStatus::Implemented,
    };
    let install_policy = validate_install_policy(raw.install_policy, tier)?;
    let components = validate_components(raw.components)?;
    match implementation_status {
        ImplementationStatus::Planned if !components.is_empty() => {
            return Err(invalid(
                PackageField::ImplementationStatus,
                PackageValidationErrorKind::Inconsistent,
            ));
        }
        ImplementationStatus::Implemented if components.is_empty() => {
            return Err(invalid(
                PackageField::ImplementationStatus,
                PackageValidationErrorKind::Inconsistent,
            ));
        }
        _ => {}
    }
    let capabilities = validate_capabilities(raw.capabilities)?;
    let source_policy = validate_source_policy(raw.source_policy.0)?;

    let component_bytes = encode_components(&components);
    let capability_bytes = encode_capabilities(&capabilities);
    let source_policy_bytes = encode_source_policy(&source_policy);
    let component_declaration_set_digest = Sha256Digest::from_bytes(&component_bytes);
    let capability_manifest_digest = Sha256Digest::from_bytes(&capability_bytes);
    let source_policy_digest = Sha256Digest::from_bytes(&source_policy_bytes);

    let mut canonical = PACKAGE_DOMAIN.to_vec();
    encode_string(package_id.as_str(), &mut canonical);
    encode_string(&package_version.as_str(), &mut canonical);
    encode_string(&raw.publisher, &mut canonical);
    canonical.push(package_tier_tag(tier));
    encode_string(&raw.display_name, &mut canonical);
    encode_optional_string(raw.description.as_deref(), &mut canonical);
    canonical.push(implementation_status_tag(implementation_status));
    canonical.push(install_policy_class_tag(install_policy.class()));
    encode_bool(install_policy.default_installed(), &mut canonical);
    encode_bool(install_policy.default_enabled(), &mut canonical);
    encode_bool(install_policy.user_disable_allowed(), &mut canonical);
    canonical.extend_from_slice(&component_bytes[COMPONENTS_DOMAIN.len()..]);
    canonical.extend_from_slice(&capability_bytes[CAPABILITIES_DOMAIN.len()..]);
    canonical.extend_from_slice(&source_policy_bytes[SOURCE_POLICY_DOMAIN.len()..]);
    let package_digest = Sha256Digest::from_bytes(&canonical);

    Ok(ValidatedPackageManifest {
        package_id,
        package_version,
        publisher: raw.publisher,
        tier,
        display_name: raw.display_name,
        description: raw.description,
        implementation_status,
        install_policy,
        components,
        capabilities,
        source_policy,
        package_digest,
        component_declaration_set_digest,
        capability_manifest_digest,
        source_policy_digest,
    })
}

fn validate_install_policy(
    raw: RawInstallPolicy,
    tier: PackageTier,
) -> Result<InstallPolicy, PackageValidationError> {
    let class = match raw.class {
        RawInstallPolicyClass::FirstPartySystemPlugin => InstallPolicyClass::FirstPartySystemPlugin,
        RawInstallPolicyClass::UserInstalledPlugin => InstallPolicyClass::UserInstalledPlugin,
    };
    if raw.default_enabled && !raw.default_installed {
        return Err(invalid(
            PackageField::InstallPolicy,
            PackageValidationErrorKind::Inconsistent,
        ));
    }
    let coherent = match (tier, class) {
        (PackageTier::FirstParty, InstallPolicyClass::FirstPartySystemPlugin) => {
            raw.default_installed && raw.default_enabled && raw.user_disable_allowed
        }
        (
            PackageTier::VerifiedCommunityText | PackageTier::VerifiedRemoteMcp,
            InstallPolicyClass::UserInstalledPlugin,
        ) => !raw.default_installed && !raw.default_enabled,
        _ => false,
    };
    if !coherent {
        return Err(invalid(
            PackageField::InstallPolicy,
            PackageValidationErrorKind::Inconsistent,
        ));
    }
    Ok(InstallPolicy {
        class,
        default_installed: raw.default_installed,
        default_enabled: raw.default_enabled,
        user_disable_allowed: raw.user_disable_allowed,
    })
}

fn validate_components(
    raw: Vec<RawComponentDeclaration>,
) -> Result<Vec<ComponentDeclaration>, PackageValidationError> {
    if raw.len() > MAX_COMPONENTS {
        return Err(invalid(
            PackageField::Components,
            PackageValidationErrorKind::TooMany,
        ));
    }
    let mut paths = BTreeSet::new();
    let mut components = Vec::with_capacity(raw.len());
    for declaration in raw {
        if !is_valid_component_path(&declaration.path) {
            return Err(invalid(
                PackageField::ComponentPath,
                classify_text(&declaration.path, MAX_COMPONENT_PATH_BYTES),
            ));
        }
        if !paths.insert(declaration.path.clone()) {
            return Err(invalid(
                PackageField::ComponentPath,
                PackageValidationErrorKind::Duplicate,
            ));
        }
        if let Some(mode) = declaration.mode.as_ref()
            && !is_valid_ascii_mode(mode)
        {
            return Err(invalid(
                PackageField::ComponentMode,
                classify_text(mode, MAX_COMPONENT_MODE_BYTES),
            ));
        }
        let kind = match declaration.kind {
            RawComponentKind::SkillComponent => ComponentKind::SkillComponent,
            RawComponentKind::DeclarativeResourcePack => ComponentKind::DeclarativeResourcePack,
            RawComponentKind::McpServerComponent => ComponentKind::McpServerComponent,
            RawComponentKind::NativeRustComponent => ComponentKind::NativeRustComponent,
        };
        components.push(ComponentDeclaration {
            kind,
            path: declaration.path,
            mode: declaration.mode,
        });
    }
    components.sort_by(|left, right| {
        left.path
            .cmp(&right.path)
            .then_with(|| component_kind_tag(left.kind).cmp(&component_kind_tag(right.kind)))
            .then_with(|| left.mode.cmp(&right.mode))
    });
    Ok(components)
}

fn validate_capabilities(raw: Vec<String>) -> Result<Vec<CapabilityId>, PackageValidationError> {
    if raw.len() > MAX_CAPABILITIES {
        return Err(invalid(
            PackageField::Capabilities,
            PackageValidationErrorKind::TooMany,
        ));
    }
    let mut capabilities = BTreeSet::new();
    for value in raw {
        let capability = CapabilityId::parse(value).map_err(|_| {
            invalid(
                PackageField::Capabilities,
                PackageValidationErrorKind::InvalidFormat,
            )
        })?;
        if !capabilities.insert(capability) {
            return Err(invalid(
                PackageField::Capabilities,
                PackageValidationErrorKind::Duplicate,
            ));
        }
    }
    Ok(capabilities.into_iter().collect())
}

fn validate_source_policy(
    raw: BTreeMap<String, String>,
) -> Result<BTreeMap<String, String>, PackageValidationError> {
    if raw.is_empty() {
        return Err(invalid(
            PackageField::SourcePolicy,
            PackageValidationErrorKind::Empty,
        ));
    }
    if raw.len() > MAX_SOURCE_POLICY_ENTRIES {
        return Err(invalid(
            PackageField::SourcePolicy,
            PackageValidationErrorKind::TooMany,
        ));
    }
    for (key, value) in &raw {
        if !is_valid_source_policy_key(key) {
            return Err(invalid(
                PackageField::SourcePolicyKey,
                classify_text(key, MAX_COMPONENT_MODE_BYTES),
            ));
        }
        validate_text(
            value,
            MAX_SOURCE_POLICY_VALUE_BYTES,
            PackageField::SourcePolicyValue,
        )?;
    }
    Ok(raw)
}

const fn invalid(field: PackageField, kind: PackageValidationErrorKind) -> PackageValidationError {
    PackageValidationError { field, kind }
}

fn validate_text(
    value: &str,
    maximum: usize,
    field: PackageField,
) -> Result<(), PackageValidationError> {
    let kind = classify_text(value, maximum);
    if kind == PackageValidationErrorKind::InvalidFormat
        && is_valid_control_free_text(value, maximum)
    {
        Ok(())
    } else {
        Err(invalid(field, kind))
    }
}

fn classify_text(value: &str, maximum: usize) -> PackageValidationErrorKind {
    if value.is_empty() {
        PackageValidationErrorKind::Empty
    } else if value.len() > maximum {
        PackageValidationErrorKind::TooLong
    } else {
        PackageValidationErrorKind::InvalidFormat
    }
}

fn is_valid_control_free_text(value: &str, maximum: usize) -> bool {
    !value.is_empty() && value.len() <= maximum && !value.chars().any(char::is_control)
}

fn is_valid_package_id(value: &str) -> bool {
    if value.is_empty() || value.len() > MAX_PACKAGE_ID_BYTES || !value.is_ascii() {
        return false;
    }
    let mut segments = value.split('.');
    let Some(first) = segments.next() else {
        return false;
    };
    if first.is_empty()
        || !first
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
    {
        return false;
    }
    let remaining: Vec<&str> = segments.collect();
    !remaining.is_empty()
        && remaining.iter().all(|segment| {
            !segment.is_empty()
                && segment
                    .bytes()
                    .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
        })
}

fn is_valid_release_version(value: &str) -> bool {
    let segments: Vec<&str> = value.split('.').collect();
    segments.len() == 3
        && segments.iter().all(|segment| {
            !segment.is_empty()
                && segment.bytes().all(|byte| byte.is_ascii_digit())
                && (segment == &"0" || !segment.starts_with('0'))
        })
}

fn is_valid_publisher(value: &str) -> bool {
    let Some((first, remaining)) = value.as_bytes().split_first() else {
        return false;
    };
    value.len() <= MAX_PUBLISHER_BYTES
        && value.is_ascii()
        && first.is_ascii_alphanumeric()
        && remaining
            .iter()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
}

fn is_valid_ascii_identity(value: &str, maximum: usize) -> bool {
    !value.is_empty()
        && value.len() <= maximum
        && value.is_ascii()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
}

fn is_valid_ascii_mode(value: &str) -> bool {
    is_valid_ascii_identity(value, MAX_COMPONENT_MODE_BYTES)
}

fn is_valid_component_path(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_COMPONENT_PATH_BYTES
        && value.is_ascii()
        && !value.contains('\\')
        && value.split('/').all(|segment| {
            !segment.is_empty()
                && segment != "."
                && segment != ".."
                && segment
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
        })
}

fn is_valid_source_policy_key(value: &str) -> bool {
    let mut bytes = value.bytes();
    let Some(first) = bytes.next() else {
        return false;
    };
    value.len() <= MAX_COMPONENT_MODE_BYTES
        && first.is_ascii_alphabetic()
        && bytes.all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
}

fn encode_count(count: usize, output: &mut Vec<u8>) {
    output.extend_from_slice(&(count as u64).to_be_bytes());
}

fn encode_string(value: &str, output: &mut Vec<u8>) {
    encode_count(value.len(), output);
    output.extend_from_slice(value.as_bytes());
}

fn encode_optional_string(value: Option<&str>, output: &mut Vec<u8>) {
    match value {
        Some(value) => {
            output.push(1);
            encode_string(value, output);
        }
        None => output.push(0),
    }
}

fn encode_bool(value: bool, output: &mut Vec<u8>) {
    output.push(u8::from(value));
}

fn encode_components(components: &[ComponentDeclaration]) -> Vec<u8> {
    let mut output = COMPONENTS_DOMAIN.to_vec();
    encode_count(components.len(), &mut output);
    for component in components {
        encode_string(component.path(), &mut output);
        output.push(component_kind_tag(component.kind()));
        encode_optional_string(component.mode(), &mut output);
    }
    output
}

fn encode_capabilities(capabilities: &[CapabilityId]) -> Vec<u8> {
    let mut output = CAPABILITIES_DOMAIN.to_vec();
    encode_count(capabilities.len(), &mut output);
    for capability in capabilities {
        encode_string(capability.as_str(), &mut output);
    }
    output
}

fn encode_source_policy(source_policy: &BTreeMap<String, String>) -> Vec<u8> {
    let mut output = SOURCE_POLICY_DOMAIN.to_vec();
    encode_count(source_policy.len(), &mut output);
    for (key, value) in source_policy {
        encode_string(key, &mut output);
        encode_string(value, &mut output);
    }
    output
}

const fn package_tier_tag(value: PackageTier) -> u8 {
    match value {
        PackageTier::FirstParty => 1,
        PackageTier::VerifiedCommunityText => 2,
        PackageTier::VerifiedRemoteMcp => 3,
    }
}

const fn implementation_status_tag(value: ImplementationStatus) -> u8 {
    match value {
        ImplementationStatus::Planned => 1,
        ImplementationStatus::Development => 2,
        ImplementationStatus::Implemented => 3,
    }
}

const fn install_policy_class_tag(value: InstallPolicyClass) -> u8 {
    match value {
        InstallPolicyClass::FirstPartySystemPlugin => 1,
        InstallPolicyClass::UserInstalledPlugin => 2,
    }
}

const fn component_kind_tag(value: ComponentKind) -> u8 {
    match value {
        ComponentKind::SkillComponent => 1,
        ComponentKind::DeclarativeResourcePack => 2,
        ComponentKind::McpServerComponent => 3,
        ComponentKind::NativeRustComponent => 4,
    }
}

fn compare_packages(
    left: &ValidatedPackageManifest,
    right: &ValidatedPackageManifest,
) -> std::cmp::Ordering {
    left.package_id()
        .as_str()
        .cmp(right.package_id().as_str())
        .then_with(|| {
            left.package_version()
                .as_str()
                .cmp(&right.package_version().as_str())
        })
}

fn same_package_revision(
    left: &ValidatedPackageManifest,
    right: &ValidatedPackageManifest,
) -> bool {
    left.package_id() == right.package_id() && left.package_version() == right.package_version()
}
