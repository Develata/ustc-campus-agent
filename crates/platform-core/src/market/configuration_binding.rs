//! Exact package/component coherence for checked configuration schemas.
//!
//! Binding proves consistency of caller-supplied inputs, never reviewed provenance,
//! installation state, tenant authority or permission to execute a component.

use super::configuration_schema::{ConfigurationSchema, ConfigurationValidationError};
use super::installation::{InstallationConfiguration, InstallationPackagePin};
use crate::invocation::{ComponentId, Sha256Digest};
use std::error::Error;
use std::fmt;

/// Stable failure categories; no submitted identifiers, values or secret references.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigurationBindingError {
    UnknownComponent,
    SchemaDigestMismatch,
    PackagePinMismatch,
    ComponentPinMismatch,
    InvalidConfiguration(ConfigurationValidationError),
}

impl fmt::Display for ConfigurationBindingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "configuration binding rejected: {self:?}")
    }
}

impl Error for ConfigurationBindingError {}

/// Immutable constraints associated with one member of an exact package revision.
///
/// This value is not admission evidence. Production composition must obtain and
/// hold it from a reviewed declaration rather than accept it from the browser.
#[derive(Clone, PartialEq, Eq)]
pub struct ComponentConfigurationBinding {
    package_pin: InstallationPackagePin,
    component_id: ComponentId,
    schema: ConfigurationSchema,
}

impl ComponentConfigurationBinding {
    /// Check membership before checking the caller-supplied expected schema digest.
    pub fn new(
        package_pin: InstallationPackagePin,
        component_id: ComponentId,
        expected_schema_digest: Sha256Digest,
        schema: ConfigurationSchema,
    ) -> Result<Self, ConfigurationBindingError> {
        if !package_pin
            .components()
            .iter()
            .any(|component| component.component_id() == &component_id)
        {
            return Err(ConfigurationBindingError::UnknownComponent);
        }
        if schema.digest() != &expected_schema_digest {
            return Err(ConfigurationBindingError::SchemaDigestMismatch);
        }
        Ok(Self {
            package_pin,
            component_id,
            schema,
        })
    }

    #[must_use]
    pub const fn package_pin(&self) -> &InstallationPackagePin {
        &self.package_pin
    }

    #[must_use]
    pub const fn component_id(&self) -> &ComponentId {
        &self.component_id
    }

    #[must_use]
    pub const fn schema(&self) -> &ConfigurationSchema {
        &self.schema
    }

    /// Check package metadata, then all members, then values, without mutation.
    pub fn validate(
        &self,
        current_package_pin: &InstallationPackagePin,
        configuration: &InstallationConfiguration,
    ) -> Result<(), ConfigurationBindingError> {
        let expected = &self.package_pin;
        let current = current_package_pin;
        if expected.catalog_revision() != current.catalog_revision()
            || expected.package_id() != current.package_id()
            || expected.package_version() != current.package_version()
            || expected.package_digest() != current.package_digest()
            || expected.component_set_digest() != current.component_set_digest()
            || expected.capability_manifest_digest() != current.capability_manifest_digest()
        {
            return Err(ConfigurationBindingError::PackagePinMismatch);
        }
        if expected.components() != current.components() {
            return Err(ConfigurationBindingError::ComponentPinMismatch);
        }
        self.schema
            .validate(configuration)
            .map_err(ConfigurationBindingError::InvalidConfiguration)
    }
}

impl fmt::Debug for ComponentConfigurationBinding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ComponentConfigurationBinding")
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use super::super::configuration_schema::ConfigurationFieldSchema;
    use super::super::installation::{ConfigurationKey, ConfigurationValue, InstalledComponentPin};
    use super::*;
    use crate::identity::TenantId;
    use crate::invocation::{
        CatalogRevision, ComponentKind, ComponentVersion, ExecutionIdentity, PackageId,
        PackageVersion,
    };

    fn id(value: &str) -> ComponentId {
        ComponentId::parse(value).expect("fixture component")
    }

    fn component(changed: &str) -> InstalledComponentPin {
        InstalledComponentPin::new(
            id(if changed == "id" {
                "component:other"
            } else {
                "component:primary"
            }),
            if changed == "kind" {
                ComponentKind::SkillComponent
            } else {
                ComponentKind::McpServerComponent
            },
            ComponentVersion::parse(if changed == "version" {
                "component-version:2"
            } else {
                "component-version:1"
            })
            .expect("fixture version"),
            Sha256Digest::from_bytes(if changed == "digest" {
                b"changed"
            } else {
                b"component"
            }),
            ExecutionIdentity::parse(if changed == "execution" {
                "execution:other"
            } else {
                "execution:primary"
            })
            .expect("fixture execution"),
        )
        .expect("fixture component pin")
    }

    fn package(changed: &str, components: Vec<InstalledComponentPin>) -> InstallationPackagePin {
        InstallationPackagePin::new(
            CatalogRevision::parse(if changed == "catalog" {
                "catalog:2"
            } else {
                "catalog:1"
            })
            .expect("fixture catalog"),
            PackageId::parse(if changed == "id" {
                "ustc.other"
            } else {
                "ustc.configuration-test"
            })
            .expect("fixture package"),
            PackageVersion::parse(if changed == "version" {
                "2.0.0"
            } else {
                "1.0.0"
            })
            .expect("fixture version"),
            Sha256Digest::from_bytes(if changed == "digest" {
                b"changed"
            } else {
                b"package"
            }),
            components,
            Sha256Digest::from_bytes(if changed == "component-set" {
                b"changed"
            } else {
                b"component-set"
            }),
            Sha256Digest::from_bytes(if changed == "capability-manifest" {
                b"changed"
            } else {
                b"capability-manifest"
            }),
        )
        .expect("fixture package pin")
    }

    fn schema(required: bool) -> ConfigurationSchema {
        ConfigurationSchema::new(vec![ConfigurationFieldSchema::boolean(
            ConfigurationKey::parse("enabled").expect("fixture key"),
            required,
        )])
        .expect("fixture schema")
    }

    fn configuration(value: Option<ConfigurationValue>) -> InstallationConfiguration {
        InstallationConfiguration::new(
            &TenantId::parse("tenant:configuration-test").expect("fixture tenant"),
            value
                .into_iter()
                .map(|value| {
                    (
                        ConfigurationKey::parse("enabled").expect("fixture key"),
                        value,
                    )
                })
                .collect(),
        )
        .expect("fixture configuration")
    }

    fn binding(pin: InstallationPackagePin) -> ComponentConfigurationBinding {
        let schema = schema(true);
        ComponentConfigurationBinding::new(
            pin,
            id("component:primary"),
            schema.digest().clone(),
            schema,
        )
        .expect("fixture binding")
    }

    #[test]
    fn constructor_rejects_absent_component_before_substituted_schema() {
        let pin = package("", vec![component("")]);
        assert_eq!(
            ComponentConfigurationBinding::new(
                pin.clone(),
                id("component:missing"),
                schema(false).digest().clone(),
                schema(true)
            ),
            Err(ConfigurationBindingError::UnknownComponent)
        );
        assert_eq!(
            ComponentConfigurationBinding::new(
                pin,
                id("component:primary"),
                schema(false).digest().clone(),
                schema(true)
            ),
            Err(ConfigurationBindingError::SchemaDigestMismatch)
        );
    }

    #[test]
    fn matching_pins_validate_without_changing_inputs() {
        let pin = package("", vec![component("")]);
        let bound = binding(pin.clone());
        let values = configuration(Some(ConfigurationValue::Boolean(false)));
        let before = values.clone();
        assert_eq!(bound.validate(&pin, &values), Ok(()));
        assert_eq!(values, before);
        assert_eq!(bound.package_pin(), &pin);
        assert_eq!(bound.component_id(), &id("component:primary"));
        assert_eq!(bound.schema().digest(), schema(true).digest());
    }

    #[test]
    fn every_package_metadata_drift_precedes_member_and_value_validation() {
        let bound = binding(package("", vec![component("")]));
        let missing_required = configuration(None);
        for changed in [
            "catalog",
            "id",
            "version",
            "digest",
            "component-set",
            "capability-manifest",
        ] {
            let pin = package(changed, vec![component("execution")]);
            assert_eq!(
                bound.validate(&pin, &missing_required),
                Err(ConfigurationBindingError::PackagePinMismatch),
                "{changed}"
            );
        }
    }

    #[test]
    fn complete_member_pin_is_checked_even_with_unchanged_claimed_set_digest() {
        let bound = binding(package("", vec![component("")]));
        for changed in ["id", "kind", "version", "digest", "execution"] {
            let pin = package("", vec![component(changed)]);
            assert_eq!(
                bound.validate(&pin, &configuration(None)),
                Err(ConfigurationBindingError::ComponentPinMismatch),
                "{changed}"
            );
        }
    }

    #[test]
    fn canonical_order_is_accepted_but_sibling_add_remove_or_change_is_rejected() {
        let primary = component("");
        let sibling = component("id");
        let bound = binding(package("", vec![primary.clone(), sibling.clone()]));
        let values = configuration(Some(ConfigurationValue::Boolean(true)));
        assert_eq!(
            bound.validate(
                &package("", vec![sibling.clone(), primary.clone()]),
                &values
            ),
            Ok(())
        );
        assert_eq!(
            bound.validate(&package("", vec![primary.clone()]), &values),
            Err(ConfigurationBindingError::ComponentPinMismatch)
        );
        let changed_sibling = InstalledComponentPin::new(
            sibling.component_id().clone(),
            sibling.kind(),
            sibling.version().clone(),
            Sha256Digest::from_bytes(b"changed-sibling"),
            sibling.execution_identity().clone(),
        )
        .expect("changed sibling");
        assert_eq!(
            bound.validate(
                &package("", vec![primary.clone(), changed_sibling]),
                &values
            ),
            Err(ConfigurationBindingError::ComponentPinMismatch)
        );
        let single = binding(package("", vec![primary.clone()]));
        assert_eq!(
            single.validate(&package("", vec![primary, sibling]), &values),
            Err(ConfigurationBindingError::ComponentPinMismatch)
        );
    }

    #[test]
    fn schema_errors_remain_typed_and_diagnostics_do_not_disclose_identifiers() {
        let pin = package("", vec![component("")]);
        let bound = binding(pin.clone());
        assert_eq!(
            bound.validate(&pin, &configuration(None)),
            Err(ConfigurationBindingError::InvalidConfiguration(
                ConfigurationValidationError::MissingRequiredField
            ))
        );
        let error = bound
            .validate(&pin, &configuration(Some(ConfigurationValue::Integer(42))))
            .expect_err("wrong type");
        assert_eq!(
            error,
            ConfigurationBindingError::InvalidConfiguration(
                ConfigurationValidationError::TypeMismatch
            )
        );
        let diagnostics = format!("{bound:?} {error:?} {error}");
        for hidden in [
            "ustc.configuration-test",
            "component:primary",
            "execution:primary",
            "enabled",
            "42",
            "tenant:configuration-test",
        ] {
            assert!(
                !diagnostics.contains(hidden),
                "diagnostic disclosed {hidden}"
            );
        }
    }
}
