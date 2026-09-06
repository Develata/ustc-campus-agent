//! Thin package lifecycle intent and view carriers; domain decisions belong to M20.
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PluginCommandDto {
    pub schema: String,
    pub request_id: String,
    pub intent: PluginIntentDto,
}
#[derive(Deserialize)]
#[serde(tag = "action", rename_all = "snake_case", deny_unknown_fields)]
pub enum PluginIntentDto {
    Install {
        package_id: String,
        version: String,
        catalog_revision: String,
        package_digest: String,
    },
    Configure {
        installation_id: String,
        expected_revision: String,
        #[serde(deserialize_with = "unique_values")]
        values: BTreeMap<String, PluginValueDto>,
    },
    Grant {
        installation_id: String,
        expected_revision: String,
        capability: String,
    },
    Enable {
        installation_id: String,
        expected_revision: String,
        readiness_digest: String,
    },
    Disable {
        installation_id: String,
        expected_revision: String,
    },
    Revoke {
        installation_id: String,
        expected_revision: String,
    },
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum PluginValueDto {
    Text(String),
    Integer(i64),
    Boolean(bool),
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PluginProbeDto {
    pub schema: String,
    pub installation_id: String,
    pub expected_revision: String,
}
#[derive(Serialize)]
pub struct PluginProbeResultDto {
    pub schema: &'static str,
    pub installation_id: String,
    pub revision: String,
    pub readiness_digest: String,
    pub tools: Vec<PluginToolDto>,
    pub kind: String,
}
#[derive(Serialize)]
pub struct PluginToolDto {
    pub name: String,
    pub description: String,
    pub capability: String,
}
#[derive(Serialize)]
pub struct PluginLifecycleDto {
    pub schema: &'static str,
    pub packages: Vec<PluginManagedPackageDto>,
    pub public_read_only: bool,
    pub updates: Vec<PluginUpdateViewDto>,
}
#[derive(Serialize)]
pub struct PluginManagedPackageDto {
    pub available: bool,
    pub package_id: String,
    pub version: String,
    pub name: String,
    pub description: String,
    pub catalog_revision: String,
    pub package_digest: String,
    pub kind: String,
    pub capabilities: Vec<String>,
    pub fields: Vec<PluginConfigurationFieldDto>,
    pub installation: Option<PluginInstallationDto>,
}
#[derive(Serialize)]
pub struct PluginConfigurationFieldDto {
    pub key: String,
    pub kind: String,
    pub required: bool,
    pub max_bytes: Option<usize>,
    pub integer_bounds: Option<(i64, i64)>,
}
#[derive(Serialize)]
pub struct PluginInstallationDto {
    pub id: String,
    pub revision: String,
    pub state: String,
    pub values: BTreeMap<String, PluginValueDto>,
    pub active_capabilities: Vec<String>,
}
#[derive(Serialize)]
pub struct PluginCommandResultDto {
    pub schema: &'static str,
    pub accepted: bool,
    pub installation_id: String,
    pub revision: Option<String>,
    pub state: Option<String>,
    pub replayed: bool,
}

fn unique_values<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<BTreeMap<String, PluginValueDto>, D::Error> {
    struct Values;
    impl<'de> serde::de::Visitor<'de> for Values {
        type Value = BTreeMap<String, PluginValueDto>;
        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str("a bounded configuration object with unique fields")
        }
        fn visit_map<M: serde::de::MapAccess<'de>>(
            self,
            mut map: M,
        ) -> Result<Self::Value, M::Error> {
            let mut values = BTreeMap::new();
            while let Some((key, value)) = map.next_entry::<String, PluginValueDto>()? {
                if values.len() >= 128 || values.insert(key, value).is_some() {
                    return Err(serde::de::Error::custom("invalid configuration fields"));
                }
            }
            Ok(values)
        }
    }
    deserializer.deserialize_map(Values)
}

/// Inert inputs for a review packet. This does not admit a catalog revision.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PluginImportPreviewDto {
    pub schema: String,
    pub package_id: String,
    pub version: String,
    pub display_name: String,
    pub source: String,
    pub skill: Option<String>,
    pub mcp: Option<PluginImportMcpDto>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PluginImportMcpDto {
    pub endpoint: String,
    pub tools: BTreeMap<String, String>,
}
#[derive(Serialize)]
pub struct PluginImportReviewDto {
    pub schema: &'static str,
    pub review_digest: String,
    pub files: BTreeMap<String, String>,
    pub configuration_values: BTreeMap<String, PluginValueDto>,
    pub warnings: Vec<String>,
    pub admitted: bool,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PluginUpdateDto {
    pub schema: String,
    pub request_id: String,
    pub intent: PluginUpdateIntentDto,
}
#[derive(Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case", deny_unknown_fields)]
pub enum PluginUpdateIntentDto {
    Preview {
        installation_id: String,
        expected_revision: String,
        target_version: String,
    },
    Apply {
        installation_id: String,
        expected_revision: String,
        target_version: String,
        update_id: String,
        plan_digest: String,
        target_readiness: String,
        rollback_readiness: String,
    },
    ReviewRollback {
        installation_id: String,
        expected_revision: String,
        update_id: String,
    },
    Rollback {
        installation_id: String,
        expected_revision: String,
        update_id: String,
        rollback_readiness: String,
    },
    Confirm {
        installation_id: String,
        expected_revision: String,
        update_id: String,
    },
}
#[derive(Serialize)]
pub struct PluginUpdateViewDto {
    pub schema: &'static str,
    pub update_id: String,
    pub installation_id: String,
    pub update_revision: String,
    pub installation_revision: String,
    pub state: String,
    pub rollback_version: String,
    pub target_version: String,
    pub plan_digest: String,
    pub change_class: String,
    pub replayed: bool,
    pub target_readiness: Option<String>,
    pub rollback_readiness: Option<String>,
}
