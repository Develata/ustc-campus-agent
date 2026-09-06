//! Read-only application query over the catalog snapshot shipped in this binary.
//! M20 owns manifest validation and catalog semantics; M10 owns the wire types.
//! This composition projects declarations only and never touches private runtime state.

use crate::market_bundle::MarketBundle;
use ustc_campus_agent_client_protocol::{
    MarketCatalogDto, MarketComponentDto, MarketInstallPolicyDto, MarketPackageDto,
    MarketPackageSummaryDto, MarketSourcePolicyDto,
};
use ustc_campus_agent_core::invocation::ComponentKind;
use ustc_campus_agent_core::market::{
    ImplementationStatus, InstallPolicyClass, PackageTier, ValidatedPackageManifest,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MarketCatalogError {
    Unavailable,
    InvalidReference,
    NotFound,
}

/// Immutable application query; it owns no runtime installation or user configuration.
pub(crate) struct MarketCatalogQuery {
    bundle: MarketBundle,
}

impl MarketCatalogQuery {
    pub(crate) fn bundled() -> Result<Self, MarketCatalogError> {
        Ok(Self {
            bundle: MarketBundle::bundled().map_err(|_| MarketCatalogError::Unavailable)?,
        })
    }

    pub(crate) fn browse(&self) -> MarketCatalogDto {
        MarketCatalogDto {
            schema: "market-catalog/v1".to_owned(),
            catalog_revision: self.bundle.catalog().catalog_revision().as_str().to_owned(),
            catalog_digest: self.bundle.catalog().catalog_digest().as_str().to_owned(),
            management_available: false,
            packages: self
                .bundle
                .catalog()
                .packages()
                .iter()
                .map(project_summary)
                .collect(),
        }
    }

    pub(crate) fn detail(
        &self,
        package_id: &str,
        version: &str,
    ) -> Result<MarketPackageDto, MarketCatalogError> {
        let package = self
            .bundle
            .catalog()
            .find_reference(package_id, version)
            .map_err(|_| MarketCatalogError::InvalidReference)?
            .ok_or(MarketCatalogError::NotFound)?;
        Ok(MarketPackageDto {
            schema: "market-package/v1".to_owned(),
            catalog_revision: self.bundle.catalog().catalog_revision().as_str().to_owned(),
            catalog_digest: self.bundle.catalog().catalog_digest().as_str().to_owned(),
            management_available: false,
            package: project_summary(package),
            components: package
                .components()
                .iter()
                .map(|component| MarketComponentDto {
                    kind: match component.kind() {
                        ComponentKind::SkillComponent => "SkillComponent",
                        ComponentKind::DeclarativeResourcePack => "DeclarativeResourcePack",
                        ComponentKind::McpServerComponent => "McpServerComponent",
                        ComponentKind::NativeRustComponent => "NativeRustComponent",
                    }
                    .to_owned(),
                    path: component.path().to_owned(),
                    mode: component.mode().map(str::to_owned),
                })
                .collect(),
            source_policy: package
                .source_policy()
                .iter()
                .map(|(key, value)| MarketSourcePolicyDto {
                    key: key.clone(),
                    value: value.clone(),
                })
                .collect(),
            install_policy: MarketInstallPolicyDto {
                class: match package.install_policy().class() {
                    InstallPolicyClass::FirstPartySystemPlugin => "FirstPartySystemPlugin",
                    InstallPolicyClass::UserInstalledPlugin => "UserInstalledPlugin",
                }
                .to_owned(),
                default_installed: package.install_policy().default_installed(),
                default_enabled: package.install_policy().default_enabled(),
                user_disable_allowed: package.install_policy().user_disable_allowed(),
            },
        })
    }
}

fn project_summary(package: &ValidatedPackageManifest) -> MarketPackageSummaryDto {
    MarketPackageSummaryDto {
        package_id: package.package_id().as_str().to_owned(),
        version: package.package_version().as_str(),
        publisher: package.publisher().to_owned(),
        tier: match package.tier() {
            PackageTier::FirstParty => "FirstParty",
            PackageTier::VerifiedCommunityText => "VerifiedCommunityText",
            PackageTier::VerifiedRemoteMcp => "VerifiedRemoteMcp",
        }
        .to_owned(),
        display_name: package.display_name().to_owned(),
        description: package.description().map(str::to_owned),
        implementation_status: match package.implementation_status() {
            ImplementationStatus::Planned => "planned",
            ImplementationStatus::Development => "development",
            ImplementationStatus::Implemented => "implemented",
        }
        .to_owned(),
        package_digest: package.package_digest().as_str().to_owned(),
        component_count: package.components().len(),
        requested_capabilities: package
            .capabilities()
            .iter()
            .map(|id| id.as_str().to_owned())
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundled_catalog_projects_real_declarations_without_installation_claims() {
        let query = MarketCatalogQuery::bundled().expect("bundled manifests");
        let catalog = query.browse();
        assert_eq!(catalog.packages.len(), 4);
        assert!(!catalog.management_available);
        for summary in &catalog.packages {
            let detail = query
                .detail(&summary.package_id, &summary.version)
                .expect("exact detail");
            assert_eq!(&detail.package, summary);
            assert_eq!(detail.catalog_digest, catalog.catalog_digest);
            assert_eq!(detail.components.len(), summary.component_count);
            assert!(!detail.source_policy.is_empty());
            assert!(!detail.management_available);
        }
        let calendar = query
            .detail("ustc.simple-calendar", "0.1.0")
            .expect("calendar");
        assert_eq!(calendar.components[0].kind, "NativeRustComponent");
        assert_eq!(calendar.package.implementation_status, "implemented");
        assert!(!calendar.install_policy.default_installed);
        let affairs = query
            .detail("ustc.affairs-navigator", "0.1.0")
            .expect("affairs");
        assert!(affairs.install_policy.default_enabled);
        assert_eq!(affairs.package.implementation_status, "planned");
        assert!(affairs.components.is_empty());
        // Metadata defaults cannot turn into installed state on the wire.
        let wire = serde_json::to_value(affairs).expect("serialize");
        assert!(wire.get("installed").is_none());
        assert!(wire.get("grants").is_none());
    }

    #[test]
    fn exact_reference_rejects_invalid_and_never_selects_another_version() {
        let query = MarketCatalogQuery::bundled().expect("catalog");
        for (id, version) in [
            ("", "0.1.0"),
            ("../../secret", "0.1.0"),
            ("NOT-A-PACKAGE", "0.1.0"),
            ("ustc.simple-calendar", "0.1.0+build"),
            ("ustc.simple-calendar", "latest"),
            ("ustc.simple-calendar", "00.1.0"),
        ] {
            assert_eq!(
                query.detail(id, version),
                Err(MarketCatalogError::InvalidReference)
            );
        }
        assert_eq!(
            query.detail("ustc.simple-calendar", "0.2.0"),
            Err(MarketCatalogError::NotFound)
        );
        assert_eq!(
            query.detail("ustc.missing", "0.1.0"),
            Err(MarketCatalogError::NotFound)
        );
    }
}
