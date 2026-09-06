//! Read-only M10 catalog projection. No lifecycle or execution authority.
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MarketPackageSummaryDto {
    pub package_id: String,
    pub version: String,
    pub publisher: String,
    pub tier: String,
    pub display_name: String,
    pub description: Option<String>,
    pub implementation_status: String,
    pub package_digest: String,
    pub component_count: usize,
    pub requested_capabilities: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MarketCatalogDto {
    pub schema: String,
    pub catalog_revision: String,
    pub catalog_digest: String,
    pub management_available: bool,
    pub packages: Vec<MarketPackageSummaryDto>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MarketComponentDto {
    pub kind: String,
    pub path: String,
    pub mode: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MarketSourcePolicyDto {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MarketInstallPolicyDto {
    pub class: String,
    pub default_installed: bool,
    pub default_enabled: bool,
    pub user_disable_allowed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MarketPackageDto {
    pub schema: String,
    pub catalog_revision: String,
    pub catalog_digest: String,
    pub management_available: bool,
    pub package: MarketPackageSummaryDto,
    pub components: Vec<MarketComponentDto>,
    pub source_policy: Vec<MarketSourcePolicyDto>,
    pub install_policy: MarketInstallPolicyDto,
}
