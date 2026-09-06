//! Pure, package-owned constraints over checked installation configuration values.
//!
//! This module neither installs packages nor admits components or grants execution authority.
//! A caller must bind the schema digest to the exact reviewed package/component revision.

use super::installation::{ConfigurationKey, ConfigurationValue, InstallationConfiguration};
use crate::invocation::Sha256Digest;
use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;

const MAX_FIELDS: usize = 128;
const MAX_TEXT_BYTES: usize = 4096;
const SCHEMA_DOMAIN: &[u8] = b"market-installation-configuration-schema/v0\0";

/// Stable construction failures, without configuration names or values.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigurationSchemaError {
    TooManyFields,
    DuplicateField,
    InvalidTextBound,
    InvalidIntegerRange,
}

impl fmt::Display for ConfigurationSchemaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "configuration schema rejected: {self:?}")
    }
}

impl Error for ConfigurationSchemaError {}

/// Stable validation failures, without rejected values or secret references.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigurationValidationError {
    UnknownField,
    MissingRequiredField,
    TypeMismatch,
    OutOfRange,
}

impl fmt::Display for ConfigurationValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "configuration rejected: {self:?}")
    }
}

impl Error for ConfigurationValidationError {}

/// Read-only type projection; it cannot construct an unchecked field rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigurationFieldKind {
    Text,
    Integer,
    Boolean,
    SecretRef,
}

#[derive(Clone, PartialEq, Eq)]
enum FieldRule {
    Text { max_utf8_bytes: usize },
    Integer { min: i64, max: i64 },
    Boolean,
    SecretRef,
}

/// An immutable, checked field declaration. It carries no default or runtime value.
#[derive(Clone, PartialEq, Eq)]
pub struct ConfigurationFieldSchema {
    key: ConfigurationKey,
    required: bool,
    rule: FieldRule,
}

impl ConfigurationFieldSchema {
    pub fn text(
        key: ConfigurationKey,
        required: bool,
        max_utf8_bytes: usize,
    ) -> Result<Self, ConfigurationSchemaError> {
        if !(1..=MAX_TEXT_BYTES).contains(&max_utf8_bytes) {
            return Err(ConfigurationSchemaError::InvalidTextBound);
        }
        Ok(Self {
            key,
            required,
            rule: FieldRule::Text { max_utf8_bytes },
        })
    }

    pub fn integer(
        key: ConfigurationKey,
        required: bool,
        min: i64,
        max: i64,
    ) -> Result<Self, ConfigurationSchemaError> {
        if min > max {
            return Err(ConfigurationSchemaError::InvalidIntegerRange);
        }
        Ok(Self {
            key,
            required,
            rule: FieldRule::Integer { min, max },
        })
    }

    #[must_use]
    pub const fn boolean(key: ConfigurationKey, required: bool) -> Self {
        Self {
            key,
            required,
            rule: FieldRule::Boolean,
        }
    }

    #[must_use]
    pub const fn secret_ref(key: ConfigurationKey, required: bool) -> Self {
        Self {
            key,
            required,
            rule: FieldRule::SecretRef,
        }
    }

    #[must_use]
    pub const fn key(&self) -> &ConfigurationKey {
        &self.key
    }

    #[must_use]
    pub const fn required(&self) -> bool {
        self.required
    }

    #[must_use]
    pub const fn kind(&self) -> ConfigurationFieldKind {
        match self.rule {
            FieldRule::Text { .. } => ConfigurationFieldKind::Text,
            FieldRule::Integer { .. } => ConfigurationFieldKind::Integer,
            FieldRule::Boolean => ConfigurationFieldKind::Boolean,
            FieldRule::SecretRef => ConfigurationFieldKind::SecretRef,
        }
    }

    #[must_use]
    pub const fn max_utf8_bytes(&self) -> Option<usize> {
        match self.rule {
            FieldRule::Text { max_utf8_bytes } => Some(max_utf8_bytes),
            _ => None,
        }
    }

    #[must_use]
    pub const fn integer_bounds(&self) -> Option<(i64, i64)> {
        match self.rule {
            FieldRule::Integer { min, max } => Some((min, max)),
            _ => None,
        }
    }

    fn validate(&self, value: &ConfigurationValue) -> Result<(), ConfigurationValidationError> {
        let within_bounds = match (&self.rule, value) {
            (FieldRule::Text { max_utf8_bytes }, ConfigurationValue::Text(text)) => {
                text.as_str().len() <= *max_utf8_bytes
            }
            (FieldRule::Integer { min, max }, ConfigurationValue::Integer(value)) => {
                (min..=max).contains(&value)
            }
            (FieldRule::Boolean, ConfigurationValue::Boolean(_))
            | (FieldRule::SecretRef, ConfigurationValue::Secret(_)) => true,
            _ => return Err(ConfigurationValidationError::TypeMismatch),
        };
        if within_bounds {
            Ok(())
        } else {
            Err(ConfigurationValidationError::OutOfRange)
        }
    }
}

impl fmt::Debug for ConfigurationFieldSchema {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ConfigurationFieldSchema")
            .field("kind", &self.kind())
            .field("required", &self.required)
            .finish_non_exhaustive()
    }
}

/// Immutable constraints; validation neither changes values nor supplies defaults.
#[derive(Clone, PartialEq, Eq)]
pub struct ConfigurationSchema {
    fields: BTreeMap<ConfigurationKey, ConfigurationFieldSchema>,
    digest: Sha256Digest,
}

impl ConfigurationSchema {
    pub fn new(fields: Vec<ConfigurationFieldSchema>) -> Result<Self, ConfigurationSchemaError> {
        if fields.len() > MAX_FIELDS {
            return Err(ConfigurationSchemaError::TooManyFields);
        }
        let mut canonical = BTreeMap::new();
        for field in fields {
            if canonical.insert(field.key.clone(), field).is_some() {
                return Err(ConfigurationSchemaError::DuplicateField);
            }
        }
        let digest = schema_digest(&canonical);
        Ok(Self {
            fields: canonical,
            digest,
        })
    }

    #[must_use]
    pub const fn fields(&self) -> &BTreeMap<ConfigurationKey, ConfigurationFieldSchema> {
        &self.fields
    }

    #[must_use]
    pub const fn digest(&self) -> &Sha256Digest {
        &self.digest
    }

    /// Unknown fields take precedence; remaining failures follow canonical key order.
    pub fn validate(
        &self,
        configuration: &InstallationConfiguration,
    ) -> Result<(), ConfigurationValidationError> {
        if configuration
            .entries()
            .keys()
            .any(|key| !self.fields.contains_key(key))
        {
            return Err(ConfigurationValidationError::UnknownField);
        }
        for (key, field) in &self.fields {
            match configuration.entries().get(key) {
                Some(value) => field.validate(value)?,
                None if field.required => {
                    return Err(ConfigurationValidationError::MissingRequiredField);
                }
                None => {}
            }
        }
        Ok(())
    }
}

impl fmt::Debug for ConfigurationSchema {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ConfigurationSchema")
            .field("field_count", &self.fields.len())
            .finish_non_exhaustive()
    }
}

fn schema_digest(fields: &BTreeMap<ConfigurationKey, ConfigurationFieldSchema>) -> Sha256Digest {
    let mut bytes = SCHEMA_DOMAIN.to_vec();
    bytes.extend_from_slice(&(fields.len() as u64).to_be_bytes());
    for (key, field) in fields {
        bytes.extend_from_slice(&(key.as_str().len() as u64).to_be_bytes());
        bytes.extend_from_slice(key.as_str().as_bytes());
        bytes.push(u8::from(field.required));
        match field.rule {
            FieldRule::Text { max_utf8_bytes } => {
                bytes.push(1);
                bytes.extend_from_slice(&(max_utf8_bytes as u64).to_be_bytes());
            }
            FieldRule::Integer { min, max } => {
                bytes.push(2);
                bytes.extend_from_slice(&min.to_be_bytes());
                bytes.extend_from_slice(&max.to_be_bytes());
            }
            FieldRule::Boolean => bytes.push(3),
            FieldRule::SecretRef => bytes.push(4),
        }
    }
    Sha256Digest::from_bytes(&bytes)
}

#[cfg(test)]
mod tests {
    use super::super::installation::{NonSecretText, SecretRef, SecretRefId};
    use super::*;
    use crate::identity::TenantId;

    fn key(value: &str) -> ConfigurationKey {
        ConfigurationKey::parse(value).expect("test key")
    }
    fn tenant() -> TenantId {
        TenantId::parse("tenant:configuration-test").expect("test tenant")
    }
    fn text(value: &str) -> ConfigurationValue {
        ConfigurationValue::Text(NonSecretText::parse(value).expect("test text"))
    }
    fn configuration(entries: Vec<(&str, ConfigurationValue)>) -> InstallationConfiguration {
        InstallationConfiguration::new(
            &tenant(),
            entries
                .into_iter()
                .map(|(name, value)| (key(name), value))
                .collect(),
        )
        .expect("test configuration")
    }
    fn schema(fields: Vec<ConfigurationFieldSchema>) -> ConfigurationSchema {
        ConfigurationSchema::new(fields).expect("test schema")
    }
    fn secret() -> ConfigurationValue {
        ConfigurationValue::Secret(
            SecretRef::new(
                tenant(),
                SecretRefId::parse("secret-ref:synthetic-reference").expect("test reference"),
            )
            .expect("test secret"),
        )
    }

    #[test]
    fn checked_construction_rejects_duplicates_and_out_of_bounds_declarations() {
        assert_eq!(
            ConfigurationFieldSchema::text(key("text"), true, 0),
            Err(ConfigurationSchemaError::InvalidTextBound)
        );
        assert_eq!(
            ConfigurationFieldSchema::text(key("text"), true, 4097),
            Err(ConfigurationSchemaError::InvalidTextBound)
        );
        assert!(ConfigurationFieldSchema::text(key("text"), true, 4096).is_ok());
        assert_eq!(
            ConfigurationFieldSchema::integer(key("count"), true, 1, 0),
            Err(ConfigurationSchemaError::InvalidIntegerRange)
        );
        assert_eq!(
            ConfigurationSchema::new(vec![
                ConfigurationFieldSchema::boolean(key("flag"), true),
                ConfigurationFieldSchema::secret_ref(key("flag"), false)
            ]),
            Err(ConfigurationSchemaError::DuplicateField)
        );
        let fields: Vec<_> = (0..128)
            .map(|index| ConfigurationFieldSchema::boolean(key(&format!("field{index}")), false))
            .collect();
        assert_eq!(schema(fields.clone()).fields().len(), 128);
        let mut too_many = fields;
        too_many.push(ConfigurationFieldSchema::boolean(key("extra"), false));
        assert_eq!(
            ConfigurationSchema::new(too_many),
            Err(ConfigurationSchemaError::TooManyFields)
        );
    }

    #[test]
    fn required_optional_and_unknown_fields_do_not_insert_defaults() {
        let optional = schema(vec![ConfigurationFieldSchema::boolean(key("flag"), false)]);
        let empty = configuration(vec![]);
        let before = empty.clone();
        assert_eq!(optional.validate(&empty), Ok(()));
        assert_eq!(empty, before);
        assert_eq!(schema(vec![]).validate(&empty), Ok(()));
        let required = schema(vec![ConfigurationFieldSchema::boolean(key("flag"), true)]);
        assert_eq!(
            required.validate(&empty),
            Err(ConfigurationValidationError::MissingRequiredField)
        );
        let unknown = configuration(vec![("unknown", ConfigurationValue::Boolean(true))]);
        assert_eq!(
            required.validate(&unknown),
            Err(ConfigurationValidationError::UnknownField)
        );
    }

    #[test]
    fn utf8_limit_counts_bytes_instead_of_characters() {
        let constraints = schema(vec![
            ConfigurationFieldSchema::text(key("label"), true, 6).expect("valid limit"),
        ]);
        assert_eq!(
            constraints.validate(&configuration(vec![("label", text("校园"))])),
            Ok(())
        );
        assert_eq!(
            constraints.validate(&configuration(vec![("label", text("校园a"))])),
            Err(ConfigurationValidationError::OutOfRange)
        );
    }

    #[test]
    fn integer_bounds_are_inclusive_without_arithmetic_overflow() {
        let bounded = schema(vec![
            ConfigurationFieldSchema::integer(key("count"), true, -2, 2).expect("valid range"),
        ]);
        for value in [-2, 0, 2] {
            assert_eq!(
                bounded.validate(&configuration(vec![(
                    "count",
                    ConfigurationValue::Integer(value)
                )])),
                Ok(())
            );
        }
        for value in [-3, 3, i64::MIN, i64::MAX] {
            assert_eq!(
                bounded.validate(&configuration(vec![(
                    "count",
                    ConfigurationValue::Integer(value)
                )])),
                Err(ConfigurationValidationError::OutOfRange)
            );
        }
        let full = schema(vec![
            ConfigurationFieldSchema::integer(key("count"), true, i64::MIN, i64::MAX)
                .expect("full range"),
        ]);
        for value in [i64::MIN, i64::MAX] {
            assert_eq!(
                full.validate(&configuration(vec![(
                    "count",
                    ConfigurationValue::Integer(value)
                )])),
                Ok(())
            );
        }
        let exact = schema(vec![
            ConfigurationFieldSchema::integer(key("count"), true, 7, 7).expect("single value"),
        ]);
        assert_eq!(
            exact.validate(&configuration(vec![(
                "count",
                ConfigurationValue::Integer(7)
            )])),
            Ok(())
        );
    }

    #[test]
    fn every_type_mismatch_is_rejected_and_secret_text_is_never_coerced() {
        let fields = [
            ConfigurationFieldSchema::text(key("value"), true, 4096).expect("text field"),
            ConfigurationFieldSchema::integer(key("value"), true, i64::MIN, i64::MAX)
                .expect("integer field"),
            ConfigurationFieldSchema::boolean(key("value"), true),
            ConfigurationFieldSchema::secret_ref(key("value"), true),
        ];
        let values = [
            text("sk-synthetic-not-a-real-key"),
            ConfigurationValue::Integer(12),
            ConfigurationValue::Boolean(false),
            secret(),
        ];
        for (field_index, field) in fields.into_iter().enumerate() {
            let constraints = schema(vec![field]);
            for (value_index, value) in values.iter().enumerate() {
                let input = configuration(vec![("value", value.clone())]);
                let before = input.clone();
                let expected = if field_index == value_index {
                    Ok(())
                } else {
                    Err(ConfigurationValidationError::TypeMismatch)
                };
                assert_eq!(constraints.validate(&input), expected);
                assert_eq!(input, before);
            }
        }
    }

    #[test]
    fn schema_digest_is_order_independent_and_binds_every_constraint() {
        let text_field = ConfigurationFieldSchema::text(key("a"), false, 64).expect("text field");
        let integer_field =
            ConfigurationFieldSchema::integer(key("b"), true, -4, 7).expect("integer field");
        let original = schema(vec![text_field.clone(), integer_field.clone()]);
        assert_eq!(
            original,
            schema(vec![integer_field.clone(), text_field.clone()])
        );
        let changes = [
            vec![
                ConfigurationFieldSchema::text(key("a"), true, 64).expect("field"),
                integer_field.clone(),
            ],
            vec![
                ConfigurationFieldSchema::text(key("a"), false, 65).expect("field"),
                integer_field.clone(),
            ],
            vec![
                ConfigurationFieldSchema::text(key("c"), false, 64).expect("field"),
                integer_field.clone(),
            ],
            vec![
                ConfigurationFieldSchema::boolean(key("a"), false),
                integer_field.clone(),
            ],
            vec![
                ConfigurationFieldSchema::secret_ref(key("a"), false),
                integer_field,
            ],
            vec![
                text_field.clone(),
                ConfigurationFieldSchema::integer(key("b"), true, -5, 7).expect("field"),
            ],
            vec![
                text_field.clone(),
                ConfigurationFieldSchema::integer(key("b"), true, -4, 8).expect("field"),
            ],
            vec![text_field],
        ];
        for fields in changes {
            assert_ne!(original.digest(), schema(fields).digest());
        }
        assert_ne!(
            schema(vec![ConfigurationFieldSchema::boolean(key("a"), false)]).digest(),
            schema(vec![ConfigurationFieldSchema::secret_ref(key("a"), false)]).digest()
        );
    }

    #[test]
    fn digest_has_a_frozen_cross_language_vector() {
        let constraints = schema(vec![
            ConfigurationFieldSchema::secret_ref(key("token"), false),
            ConfigurationFieldSchema::integer(key("attempts"), true, -2, 3).expect("integer field"),
            ConfigurationFieldSchema::text(key("label"), true, 6).expect("text field"),
            ConfigurationFieldSchema::boolean(key("active"), false),
        ]);
        assert_eq!(
            constraints.digest().as_str(),
            "sha256:e477cee7c35b6091a2764ca0d597db088e143a75331454878eb4f2ba9e12a50d"
        );
    }

    #[test]
    fn public_projection_is_read_only_and_debug_is_redacted() {
        let field =
            ConfigurationFieldSchema::text(key("sensitiveFieldName"), true, 4096).expect("field");
        assert_eq!(field.key(), &key("sensitiveFieldName"));
        assert!(field.required());
        assert_eq!(field.kind(), ConfigurationFieldKind::Text);
        assert_eq!(field.max_utf8_bytes(), Some(4096));
        assert_eq!(field.integer_bounds(), None);
        let constraints = schema(vec![field.clone()]);
        let debug = format!(
            "{field:?} {constraints:?} {:?} {}",
            ConfigurationValidationError::TypeMismatch,
            ConfigurationValidationError::TypeMismatch
        );
        for value in [
            "sensitiveFieldName",
            "synthetic-reference",
            "sk-synthetic",
            "4096",
        ] {
            assert!(!debug.contains(value));
        }
    }
}
