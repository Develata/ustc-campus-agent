//! Bounded loading of exact package-owned component configuration declarations.
//!
//! Consistency is checked here; reviewed provenance belongs to the composition owner.
//! No source I/O, installation mutation, secret resolution or execution is performed.

use super::ValidatedPackageManifest;
use super::configuration_binding::ComponentConfigurationBinding;
use super::configuration_schema::{ConfigurationFieldSchema, ConfigurationSchema};
use super::installation::{ConfigurationKey, InstallationPackagePin, InstalledComponentPin};
use crate::invocation::{
    CatalogRevision, ComponentId, ComponentKind, ComponentVersion, ExecutionIdentity, Sha256Digest,
};
use serde::Deserialize;
use serde::de::{self, MapAccess, SeqAccess, Visitor};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt;

const MAX_SOURCE_BYTES: usize = 1_048_576;
const MAX_COMPONENTS: usize = 64;
const MAX_FIELDS: usize = 128;
const SCHEMA_VERSION: &str = "package-component-configuration/v1";

/// Stable categories only; rejected source and identifiers are never retained.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigurationCatalogError {
    SourceTooLarge,
    JsonRejected,
    VersionRejected,
    PackageMismatch,
    MemberMismatch,
    PinMismatch,
    DuplicateComponent,
    InvalidSchema,
    SchemaDigestMismatch,
}

impl fmt::Display for ConfigurationCatalogError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "package configuration rejected: {self:?}")
    }
}

impl Error for ConfigurationCatalogError {}

/// Immutable complete configuration bindings for one exact package pin.
#[derive(Clone, PartialEq, Eq)]
pub struct ValidatedPackageConfiguration {
    package_pin: InstallationPackagePin,
    bindings: BTreeMap<ComponentId, ComponentConfigurationBinding>,
}

impl ValidatedPackageConfiguration {
    #[must_use]
    pub const fn package_pin(&self) -> &InstallationPackagePin {
        &self.package_pin
    }

    #[must_use]
    pub fn binding(&self, component: &ComponentId) -> Option<&ComponentConfigurationBinding> {
        self.bindings.get(component)
    }

    #[must_use]
    pub const fn bindings(&self) -> &BTreeMap<ComponentId, ComponentConfigurationBinding> {
        &self.bindings
    }
}

impl fmt::Debug for ValidatedPackageConfiguration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ValidatedPackageConfiguration")
            .field("component_count", &self.bindings.len())
            .finish_non_exhaustive()
    }
}

/// Decode all-or-nothing, then reuse the checked manifest, pin and schema owners.
pub fn load_package_configuration(
    source: &[u8],
    package: &ValidatedPackageManifest,
    catalog_revision: &CatalogRevision,
) -> Result<ValidatedPackageConfiguration, ConfigurationCatalogError> {
    use ConfigurationCatalogError as E;
    if source.len() > MAX_SOURCE_BYTES {
        return Err(E::SourceTooLarge);
    }
    // Value's ordinary deserializer overwrites duplicate keys. Reject them before
    // decoding the closed carriers, including internally tagged field variants.
    let unique: UniqueValue = serde_json::from_slice(source).map_err(|_| E::JsonRejected)?;
    let raw: RawPackage = serde_json::from_value(unique.0).map_err(|_| E::JsonRejected)?;
    if raw.schema_version != SCHEMA_VERSION {
        return Err(E::VersionRejected);
    }
    if raw.package_id != package.package_id().as_str()
        || raw.package_version != package.package_version().as_str()
        || raw.package_digest != package.package_digest().as_str()
        || raw.component_set_digest != package.component_declaration_set_digest().as_str()
        || raw.capability_manifest_digest != package.capability_manifest_digest().as_str()
    {
        return Err(E::PackageMismatch);
    }
    if !(1..=MAX_COMPONENTS).contains(&raw.components.len())
        || raw.components.len() != package.components().len()
    {
        return Err(E::MemberMismatch);
    }
    let mut paths = BTreeSet::new();
    let mut ids = BTreeSet::new();
    let mut pins = Vec::with_capacity(raw.components.len());
    let mut schemas = Vec::with_capacity(raw.components.len());
    for component in raw.components {
        let id = ComponentId::parse(component.component_id).map_err(|_| E::PinMismatch)?;
        if !paths.insert(component.path.clone()) || !ids.insert(id.clone()) {
            return Err(E::DuplicateComponent);
        }
        let declaration = package
            .components()
            .iter()
            .find(|declaration| declaration.path() == component.path)
            .ok_or(E::MemberMismatch)?;
        let mode = match &component.mode {
            Value::Null => None,
            Value::String(value) => Some(value.as_str()),
            _ => return Err(E::JsonRejected),
        };
        if declaration.kind() != component.kind.checked() || declaration.mode() != mode {
            return Err(E::MemberMismatch);
        }
        let pin = InstalledComponentPin::new(
            id.clone(),
            declaration.kind(),
            ComponentVersion::parse(component.component_version).map_err(|_| E::PinMismatch)?,
            Sha256Digest::parse(component.component_digest).map_err(|_| E::PinMismatch)?,
            ExecutionIdentity::parse(component.execution_identity).map_err(|_| E::PinMismatch)?,
        )
        .map_err(|_| E::PinMismatch)?;
        if component.fields.len() > MAX_FIELDS {
            return Err(E::InvalidSchema);
        }
        let fields = component
            .fields
            .into_iter()
            .map(RawField::check)
            .collect::<Result<Vec<_>, _>>()?;
        let schema = ConfigurationSchema::new(fields).map_err(|_| E::InvalidSchema)?;
        let digest = Sha256Digest::parse(component.schema_digest).map_err(|_| E::InvalidSchema)?;
        if schema.digest() != &digest {
            return Err(E::SchemaDigestMismatch);
        }
        pins.push(pin);
        schemas.push((id, digest, schema));
    }
    let package_pin = InstallationPackagePin::new(
        catalog_revision.clone(),
        package.package_id().clone(),
        package.package_version().clone(),
        package.package_digest().clone(),
        pins,
        package.component_declaration_set_digest().clone(),
        package.capability_manifest_digest().clone(),
    )
    .map_err(|_| E::PinMismatch)?;
    let mut bindings = BTreeMap::new();
    for (id, digest, schema) in schemas {
        let binding =
            ComponentConfigurationBinding::new(package_pin.clone(), id.clone(), digest, schema)
                .map_err(|_| E::SchemaDigestMismatch)?;
        bindings.insert(id, binding);
    }
    Ok(ValidatedPackageConfiguration {
        package_pin,
        bindings,
    })
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct RawPackage {
    schema_version: String,
    package_id: String,
    package_version: String,
    package_digest: String,
    component_set_digest: String,
    capability_manifest_digest: String,
    components: Vec<RawComponent>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct RawComponent {
    path: String,
    #[serde(rename = "type")]
    kind: RawKind,
    mode: Value,
    component_id: String,
    component_version: String,
    component_digest: String,
    execution_identity: String,
    schema_digest: String,
    fields: Vec<RawField>,
}

#[derive(Deserialize)]
enum RawKind {
    SkillComponent,
    DeclarativeResourcePack,
    McpServerComponent,
    NativeRustComponent,
}

impl RawKind {
    const fn checked(&self) -> ComponentKind {
        match self {
            Self::SkillComponent => ComponentKind::SkillComponent,
            Self::DeclarativeResourcePack => ComponentKind::DeclarativeResourcePack,
            Self::McpServerComponent => ComponentKind::McpServerComponent,
            Self::NativeRustComponent => ComponentKind::NativeRustComponent,
        }
    }
}

#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum RawField {
    Text {
        key: String,
        required: bool,
        #[serde(rename = "maxUtf8Bytes")]
        max_utf8_bytes: usize,
    },
    Integer {
        key: String,
        required: bool,
        min: i64,
        max: i64,
    },
    Boolean {
        key: String,
        required: bool,
    },
    SecretRef {
        key: String,
        required: bool,
    },
}

impl RawField {
    fn check(self) -> Result<ConfigurationFieldSchema, ConfigurationCatalogError> {
        use ConfigurationCatalogError::InvalidSchema;
        let key = |value| ConfigurationKey::parse(value).map_err(|_| InvalidSchema);
        match self {
            Self::Text {
                key: name,
                required,
                max_utf8_bytes,
            } => ConfigurationFieldSchema::text(key(name)?, required, max_utf8_bytes)
                .map_err(|_| InvalidSchema),
            Self::Integer {
                key: name,
                required,
                min,
                max,
            } => ConfigurationFieldSchema::integer(key(name)?, required, min, max)
                .map_err(|_| InvalidSchema),
            Self::Boolean {
                key: name,
                required,
            } => Ok(ConfigurationFieldSchema::boolean(key(name)?, required)),
            Self::SecretRef {
                key: name,
                required,
            } => Ok(ConfigurationFieldSchema::secret_ref(key(name)?, required)),
        }
    }
}

// Private decoding intermediate. Recursive serde_json decoding retains its default
// depth bound; the outer byte cap bounds allocations even for malformed carriers.
struct UniqueValue(Value);

impl<'de> Deserialize<'de> for UniqueValue {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct UniqueVisitor;
        impl<'de> Visitor<'de> for UniqueVisitor {
            type Value = UniqueValue;
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("JSON with unique object fields")
            }
            fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
                let mut fields = serde_json::Map::new();
                while let Some(key) = map.next_key::<String>()? {
                    if fields.contains_key(&key) {
                        return Err(de::Error::custom("duplicate field"));
                    }
                    fields.insert(key, map.next_value::<UniqueValue>()?.0);
                }
                Ok(UniqueValue(Value::Object(fields)))
            }
            fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
                let mut values = Vec::new();
                while let Some(value) = seq.next_element::<UniqueValue>()? {
                    values.push(value.0);
                }
                Ok(UniqueValue(Value::Array(values)))
            }
            fn visit_bool<E: de::Error>(self, value: bool) -> Result<Self::Value, E> {
                Ok(UniqueValue(Value::Bool(value)))
            }
            fn visit_i64<E: de::Error>(self, value: i64) -> Result<Self::Value, E> {
                Ok(UniqueValue(Value::Number(value.into())))
            }
            fn visit_u64<E: de::Error>(self, value: u64) -> Result<Self::Value, E> {
                Ok(UniqueValue(Value::Number(value.into())))
            }
            fn visit_f64<E: de::Error>(self, value: f64) -> Result<Self::Value, E> {
                serde_json::Number::from_f64(value)
                    .map(|n| UniqueValue(Value::Number(n)))
                    .ok_or_else(|| E::custom("invalid number"))
            }
            fn visit_str<E: de::Error>(self, value: &str) -> Result<Self::Value, E> {
                Ok(UniqueValue(Value::String(value.to_owned())))
            }
            fn visit_string<E: de::Error>(self, value: String) -> Result<Self::Value, E> {
                Ok(UniqueValue(Value::String(value)))
            }
            fn visit_unit<E: de::Error>(self) -> Result<Self::Value, E> {
                Ok(UniqueValue(Value::Null))
            }
        }
        deserializer.deserialize_any(UniqueVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::market::load_package_manifest;
    use serde_json::json;

    const MANIFEST: &[u8] =
        include_bytes!("../../../../market/packages/ustc.simple-calendar/package.json");
    const SIDECAR: &[u8] =
        include_bytes!("../../../../market/packages/ustc.simple-calendar/configuration.json");

    fn revision() -> CatalogRevision {
        CatalogRevision::parse("catalog:configuration-tests").expect("test revision")
    }

    fn fixture(count: usize) -> (ValidatedPackageManifest, Value) {
        let mut manifest: Value = serde_json::from_slice(MANIFEST).expect("manifest JSON");
        if count == 0 {
            manifest["implementationStatus"] = json!("planned");
        }
        manifest["components"] = Value::Array((0..count).map(|index| json!({
            "type": "NativeRustComponent", "path": format!("components/member-{index}.rs"),
        })).collect());
        let package =
            load_package_manifest(&serde_json::to_vec(&manifest).expect("manifest bytes"))
                .expect("checked manifest");
        let schema = ConfigurationSchema::new(vec![]).expect("empty schema");
        let sidecar = json!({
            "schemaVersion": SCHEMA_VERSION,
            "packageId": package.package_id().as_str(),
            "packageVersion": package.package_version().as_str(),
            "packageDigest": package.package_digest().as_str(),
            "componentSetDigest": package.component_declaration_set_digest().as_str(),
            "capabilityManifestDigest": package.capability_manifest_digest().as_str(),
            "components": (0..count).map(|index| json!({
                "type": "NativeRustComponent", "path": format!("components/member-{index}.rs"),
                "mode": null, "componentId": format!("component:member-{index}"),
                "componentVersion": "component-version:1",
                "componentDigest": Sha256Digest::from_bytes(b"synthetic component").as_str(),
                "executionIdentity": "execution:synthetic",
                "schemaDigest": schema.digest().as_str(), "fields": []
            })).collect::<Vec<_>>()
        });
        (package, sidecar)
    }

    fn load(
        package: &ValidatedPackageManifest,
        source: &Value,
    ) -> Result<ValidatedPackageConfiguration, ConfigurationCatalogError> {
        load_package_configuration(
            &serde_json::to_vec(source).expect("source JSON"),
            package,
            &revision(),
        )
    }

    fn replace_fields(source: &mut Value, fields: Value) {
        let raw: Vec<RawField> = serde_json::from_value(fields.clone()).expect("raw fields");
        let checked = raw
            .into_iter()
            .map(RawField::check)
            .collect::<Result<Vec<_>, _>>()
            .expect("checked fields");
        let schema = ConfigurationSchema::new(checked).expect("checked schema");
        source["components"][0]["fields"] = fields;
        source["components"][0]["schemaDigest"] = json!(schema.digest().as_str());
    }

    #[test]
    fn reviewed_simple_calendar_loads_explicit_empty_schema() {
        let package = load_package_manifest(MANIFEST).expect("reviewed manifest");
        let checked =
            load_package_configuration(SIDECAR, &package, &revision()).expect("reviewed sidecar");
        assert_eq!(checked.bindings().len(), 1);
        let member = &checked.package_pin().components()[0];
        let binding = checked
            .binding(member.component_id())
            .expect("complete binding");
        assert!(binding.schema().fields().is_empty());
        assert_eq!(binding.package_pin(), checked.package_pin());
        assert!(
            checked
                .binding(&ComponentId::parse("component:missing").expect("id"))
                .is_none()
        );
    }

    #[test]
    fn all_field_kinds_and_inclusive_integer_extremes_are_checked() {
        let (package, mut source) = fixture(1);
        replace_fields(
            &mut source,
            json!([
                {"key":"label", "required":true, "kind":"text", "maxUtf8Bytes":4096},
                {"key":"count", "required":false, "kind":"integer", "min":i64::MIN, "max":i64::MAX},
                {"key":"enabled", "required":true, "kind":"boolean"},
                {"key":"credential", "required":false, "kind":"secret_ref"}
            ]),
        );
        let checked = load(&package, &source).expect("all kinds");
        let binding = checked.bindings().values().next().expect("binding");
        assert_eq!(binding.schema().fields().len(), 4);
        assert_eq!(
            binding.schema().fields()[&ConfigurationKey::parse("count").expect("key")]
                .integer_bounds(),
            Some((i64::MIN, i64::MAX))
        );
    }

    #[test]
    fn every_package_pin_dimension_must_match_the_checked_manifest() {
        let (package, source) = fixture(1);
        for field in [
            "packageId",
            "packageVersion",
            "packageDigest",
            "componentSetDigest",
            "capabilityManifestDigest",
        ] {
            let mut changed = source.clone();
            changed[field] = json!("substituted");
            assert_eq!(
                load(&package, &changed),
                Err(ConfigurationCatalogError::PackageMismatch),
                "{field}"
            );
        }
        let mut changed = source;
        changed["schemaVersion"] = json!("package-component-configuration/v2");
        assert_eq!(
            load(&package, &changed),
            Err(ConfigurationCatalogError::VersionRejected)
        );
    }

    #[test]
    fn complete_membership_path_kind_mode_and_unique_ids_are_required() {
        let (package, source) = fixture(2);
        for (field, value) in [
            ("path", "components/unknown.rs"),
            ("type", "SkillComponent"),
            ("mode", "unexpected"),
        ] {
            let mut changed = source.clone();
            changed["components"][0][field] = json!(value);
            assert_eq!(
                load(&package, &changed),
                Err(ConfigurationCatalogError::MemberMismatch),
                "{field}"
            );
        }
        for field in ["path", "componentId"] {
            let mut changed = source.clone();
            changed["components"][1][field] = changed["components"][0][field].clone();
            assert_eq!(
                load(&package, &changed),
                Err(ConfigurationCatalogError::DuplicateComponent),
                "{field}"
            );
        }
        let mut missing = source.clone();
        missing["components"].as_array_mut().expect("array").pop();
        assert_eq!(
            load(&package, &missing),
            Err(ConfigurationCatalogError::MemberMismatch)
        );
        let mut extra = source.clone();
        extra["components"]
            .as_array_mut()
            .expect("array")
            .push(source["components"][0].clone());
        assert_eq!(
            load(&package, &extra),
            Err(ConfigurationCatalogError::MemberMismatch)
        );
        let (empty_package, empty) = fixture(0);
        assert_eq!(
            load(&empty_package, &empty),
            Err(ConfigurationCatalogError::MemberMismatch)
        );
    }

    #[test]
    fn checked_component_pin_parsers_reject_malformed_values() {
        let (package, source) = fixture(1);
        for field in [
            "componentId",
            "componentVersion",
            "componentDigest",
            "executionIdentity",
        ] {
            let mut changed = source.clone();
            changed["components"][0][field] = json!("");
            assert_eq!(
                load(&package, &changed),
                Err(ConfigurationCatalogError::PinMismatch),
                "{field}"
            );
        }
        let mut changed = source;
        changed["components"][0]["executionIdentity"] = json!({"kind":"native"});
        assert_eq!(
            load(&package, &changed),
            Err(ConfigurationCatalogError::JsonRejected)
        );
    }

    #[test]
    fn duplicate_json_fields_are_rejected_at_every_object_depth() {
        let (package, mut source) = fixture(1);
        replace_fields(
            &mut source,
            json!([{"key":"flag", "required":true, "kind":"boolean"}]),
        );
        let original = serde_json::to_string(&source).expect("JSON");
        for (needle, replacement) in [
            (
                "\"schemaVersion\":",
                "\"schemaVersion\":\"bad\",\"schemaVersion\":",
            ),
            ("\"path\":", "\"path\":\"bad\",\"path\":"),
            ("\"kind\":", "\"kind\":\"text\",\"kind\":"),
            ("\"key\":", "\"key\":\"other\",\"key\":"),
            ("\"required\":", "\"required\":false,\"required\":"),
        ] {
            let changed = original.replacen(needle, replacement, 1);
            assert_ne!(original, changed);
            assert_eq!(
                load_package_configuration(changed.as_bytes(), &package, &revision()),
                Err(ConfigurationCatalogError::JsonRejected),
                "{needle}"
            );
        }
        let escaped = original.replacen("\"kind\":", "\"k\\u0069nd\":\"boolean\",\"kind\":", 1);
        assert_eq!(
            load_package_configuration(escaped.as_bytes(), &package, &revision()),
            Err(ConfigurationCatalogError::JsonRejected)
        );
    }

    #[test]
    fn unknown_null_missing_and_inapplicable_fields_fail_closed() {
        let (package, mut source) = fixture(1);
        replace_fields(
            &mut source,
            json!([{"key":"flag", "required":true, "kind":"boolean"}]),
        );
        for path in ["", "/components/0", "/components/0/fields/0"] {
            let object = source
                .pointer(path)
                .expect("object")
                .as_object()
                .expect("map");
            for key in object.keys() {
                if key == "mode" {
                    continue;
                }
                let mut changed = source.clone();
                changed.pointer_mut(path).expect("object")[key] = Value::Null;
                assert_eq!(
                    load(&package, &changed),
                    Err(ConfigurationCatalogError::JsonRejected),
                    "null {path}/{key}"
                );
                let mut missing = source.clone();
                missing
                    .pointer_mut(path)
                    .expect("object")
                    .as_object_mut()
                    .expect("map")
                    .remove(key);
                assert_eq!(
                    load(&package, &missing),
                    Err(ConfigurationCatalogError::JsonRejected),
                    "missing {path}/{key}"
                );
            }
            let mut unknown = source.clone();
            unknown.pointer_mut(path).expect("object")["unknown"] = json!("hidden source");
            assert_eq!(
                load(&package, &unknown),
                Err(ConfigurationCatalogError::JsonRejected)
            );
        }
        let mut missing_mode = source.clone();
        missing_mode["components"][0]
            .as_object_mut()
            .expect("component")
            .remove("mode");
        assert_eq!(
            load(&package, &missing_mode),
            Err(ConfigurationCatalogError::JsonRejected)
        );
        for field in ["min", "max", "maxUtf8Bytes", "default", "value", "secret"] {
            let mut changed = source.clone();
            changed["components"][0]["fields"][0][field] = json!(1);
            assert_eq!(
                load(&package, &changed),
                Err(ConfigurationCatalogError::JsonRejected),
                "{field}"
            );
        }
    }

    #[test]
    fn schema_substitution_duplicate_keys_and_invalid_constraints_are_rejected() {
        let (package, source) = fixture(1);
        let mut substituted = source.clone();
        substituted["components"][0]["schemaDigest"] =
            json!(Sha256Digest::from_bytes(b"other schema").as_str());
        assert_eq!(
            load(&package, &substituted),
            Err(ConfigurationCatalogError::SchemaDigestMismatch)
        );
        for fields in [
            json!([{"key":"flag","required":true,"kind":"boolean"},{"key":"flag","required":false,"kind":"secret_ref"}]),
            json!([{"key":"","required":true,"kind":"boolean"}]),
            json!([{"key":"label","required":true,"kind":"text","maxUtf8Bytes":0}]),
            json!([{"key":"label","required":true,"kind":"text","maxUtf8Bytes":4097}]),
            json!([{"key":"count","required":true,"kind":"integer","min":2,"max":1}]),
        ] {
            let mut changed = source.clone();
            changed["components"][0]["fields"] = fields;
            assert_eq!(
                load(&package, &changed),
                Err(ConfigurationCatalogError::InvalidSchema)
            );
        }
        for fields in [
            json!([{"key":"count","required":true,"kind":"integer","min":0.5,"max":2}]),
            json!([{"key":"count","required":true,"kind":"integer","min":0,"max":18446744073709551615u64}]),
            json!([{"key":"label","required":true,"kind":"text","maxUtf8Bytes":null}]),
        ] {
            let mut changed = source.clone();
            changed["components"][0]["fields"] = fields;
            assert_eq!(
                load(&package, &changed),
                Err(ConfigurationCatalogError::JsonRejected)
            );
        }
    }

    #[test]
    fn byte_component_and_field_limits_are_inclusive() {
        let (package, source) = fixture(64);
        assert_eq!(
            load(&package, &source)
                .expect("64 components")
                .bindings()
                .len(),
            64
        );
        let mut excessive = source;
        let component = excessive["components"][0].clone();
        excessive["components"]
            .as_array_mut()
            .expect("components")
            .push(component);
        assert_eq!(
            load(&package, &excessive),
            Err(ConfigurationCatalogError::MemberMismatch)
        );
        let (package, mut source) = fixture(1);
        let fields = (0..128)
            .map(|index| json!({"key":format!("field{index}"), "required":false, "kind":"boolean"}))
            .collect::<Vec<_>>();
        replace_fields(&mut source, json!(fields));
        assert!(load(&package, &source).is_ok());
        source["components"][0]["fields"]
            .as_array_mut()
            .expect("fields")
            .push(json!({"key":"extra","required":false,"kind":"boolean"}));
        assert_eq!(
            load(&package, &source),
            Err(ConfigurationCatalogError::InvalidSchema)
        );
        let (package, source) = fixture(1);
        let mut bytes = serde_json::to_vec(&source).expect("JSON");
        bytes.resize(MAX_SOURCE_BYTES, b' ');
        assert!(load_package_configuration(&bytes, &package, &revision()).is_ok());
        bytes.push(b' ');
        assert_eq!(
            load_package_configuration(&bytes, &package, &revision()),
            Err(ConfigurationCatalogError::SourceTooLarge)
        );
    }

    #[test]
    fn component_and_field_order_do_not_change_the_checked_result() {
        let (package, mut source) = fixture(3);
        replace_fields(
            &mut source,
            json!([
                {"key":"b", "required":false, "kind":"secret_ref"},
                {"key":"a", "required":true, "kind":"boolean"}
            ]),
        );
        let checked = load(&package, &source).expect("original");
        source["components"][0]["fields"]
            .as_array_mut()
            .expect("fields")
            .reverse();
        source["components"]
            .as_array_mut()
            .expect("components")
            .reverse();
        assert_eq!(load(&package, &source), Ok(checked));
    }

    #[test]
    fn diagnostics_are_redacted_and_malformed_sources_never_return_partial_output() {
        let (package, mut source) = fixture(2);
        source["components"][1]["executionIdentity"] = json!("hidden secret\n");
        let error = load(&package, &source).expect_err("invalid last member");
        let (_, valid) = fixture(2);
        let checked = load(&package, &valid).expect("valid");
        let diagnostics = format!("{error:?} {error} {checked:?}");
        for hidden in [
            "hidden",
            "secret",
            "execution:synthetic",
            "component:member",
            "ustc.simple-calendar",
        ] {
            assert!(!diagnostics.contains(hidden));
        }
        for bytes in [b"{} trailing".as_slice(), b"\xff", b"[]", b"null"] {
            assert_eq!(
                load_package_configuration(bytes, &package, &revision()),
                Err(ConfigurationCatalogError::JsonRejected)
            );
        }
        let deep = format!("{}null{}", "[".repeat(200), "]".repeat(200));
        assert_eq!(
            load_package_configuration(deep.as_bytes(), &package, &revision()),
            Err(ConfigurationCatalogError::JsonRejected)
        );
    }
}
