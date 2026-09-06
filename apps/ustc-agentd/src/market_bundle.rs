//! Fixed reviewed catalog assembly shared by browsing and future installation admission.

use ustc_campus_agent_core::invocation::{CatalogRevision, Sha256Digest};
use ustc_campus_agent_core::market::configuration_catalog::{
    ValidatedPackageConfiguration, load_package_configuration,
};
use ustc_campus_agent_core::market::{CatalogReadModel, load_package_manifest};

pub(crate) const MANIFESTS: [(&str, &[u8]); 4] = [
    (
        "market/packages/ustc.affairs-navigator/package.json",
        include_bytes!("../../../market/packages/ustc.affairs-navigator/package.json"),
    ),
    (
        "market/packages/ustc.change-radar/package.json",
        include_bytes!("../../../market/packages/ustc.change-radar/package.json"),
    ),
    (
        "market/packages/ustc.opportunity-graph/package.json",
        include_bytes!("../../../market/packages/ustc.opportunity-graph/package.json"),
    ),
    (
        "market/packages/ustc.simple-calendar/package.json",
        include_bytes!("../../../market/packages/ustc.simple-calendar/package.json"),
    ),
];
const CONFIGURATIONS: [(&str, &[u8]); 1] = [(
    "market/packages/ustc.simple-calendar/configuration.json",
    include_bytes!("../../../market/packages/ustc.simple-calendar/configuration.json"),
)];
const MAX_SOURCE_BYTES: usize = 1_048_576;
const MAX_BUNDLE_BYTES: usize = 16 * MAX_SOURCE_BYTES;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BundleUnavailable;

pub(crate) struct MarketBundle {
    catalog: CatalogReadModel,
    // Held with the catalog so a future installer cannot select independent schema bytes.
    #[allow(dead_code)]
    configurations: Vec<ValidatedPackageConfiguration>,
}
impl MarketBundle {
    pub(crate) fn bundled() -> Result<Self, BundleUnavailable> {
        Self::from_sources(&MANIFESTS, &CONFIGURATIONS)
    }

    pub(crate) fn catalog(&self) -> &CatalogReadModel {
        &self.catalog
    }

    /// Internal exact lookup. No user-provided schema or source selector is accepted.
    /// Installation application admission is a separate, not-yet-exposed caller.
    #[allow(dead_code)]
    pub(crate) fn configuration(
        &self,
        package: &str,
        version: &str,
    ) -> Option<&ValidatedPackageConfiguration> {
        self.configurations.iter().find(|configuration| {
            let pin = configuration.package_pin();
            pin.package_id().as_str() == package && pin.package_version().as_str() == version
        })
    }

    pub(crate) fn from_sources(
        manifests: &[(&str, &[u8])],
        configurations: &[(&str, &[u8])],
    ) -> Result<Self, BundleUnavailable> {
        if manifests.len() > 64 || configurations.len() > 64 {
            return Err(BundleUnavailable);
        }
        let mut total = 0usize;
        let mut sources = Vec::new();
        for (role, entries) in [("manifest", manifests), ("configuration", configurations)] {
            for &(path, bytes) in entries {
                if path.len() > 512
                    || path.starts_with('/')
                    || path.contains('\\')
                    || path
                        .split('/')
                        .any(|part| part.is_empty() || part == "." || part == "..")
                    || bytes.len() > MAX_SOURCE_BYTES
                {
                    return Err(BundleUnavailable);
                }
                total = total.checked_add(bytes.len()).ok_or(BundleUnavailable)?;
                if total > MAX_BUNDLE_BYTES {
                    return Err(BundleUnavailable);
                }
                sources.push((role, path, bytes));
            }
        }
        sources.sort_unstable_by_key(|(role, path, _)| (*role, *path));
        if sources
            .windows(2)
            .any(|pair| (pair[0].0, pair[0].1) == (pair[1].0, pair[1].1))
        {
            return Err(BundleUnavailable);
        }
        let packages = manifests
            .iter()
            .map(|(path, source)| {
                let package = load_package_manifest(source).map_err(|_| BundleUnavailable)?;
                if *path
                    != format!(
                        "market/packages/{}/package.json",
                        package.package_id().as_str()
                    )
                {
                    return Err(BundleUnavailable);
                }
                Ok(package)
            })
            .collect::<Result<Vec<_>, _>>()?;
        let mut encoded = b"market-bundled-sources/v2\0".to_vec();
        encoded.extend_from_slice(&(sources.len() as u64).to_be_bytes());
        for (role, path, bytes) in sources {
            for part in [role.as_bytes(), path.as_bytes(), bytes] {
                encoded.extend_from_slice(&(part.len() as u64).to_be_bytes());
                encoded.extend_from_slice(part);
            }
        }
        let digest = Sha256Digest::from_bytes(&encoded);
        let revision = CatalogRevision::parse(format!("catalog:{}", &digest.as_str()[7..]))
            .map_err(|_| BundleUnavailable)?;
        let catalog =
            CatalogReadModel::new(revision.clone(), packages).map_err(|_| BundleUnavailable)?;
        let mut loaded = Vec::new();
        for &(path, source) in configurations {
            let package = catalog
                .packages()
                .iter()
                .find(|package| {
                    path == format!(
                        "market/packages/{}/configuration.json",
                        package.package_id().as_str()
                    )
                })
                .ok_or(BundleUnavailable)?;
            loaded.push(
                load_package_configuration(source, package, &revision)
                    .map_err(|_| BundleUnavailable)?,
            );
        }
        Ok(Self {
            catalog,
            configurations: loaded,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn market_bundle_shares_exact_configuration_with_catalog_without_execution_readiness() {
        let bundle = MarketBundle::bundled().expect("valid reviewed bundle fixture");
        let config = bundle
            .configuration("ustc.simple-calendar", "0.1.0")
            .expect("valid reviewed bundle fixture");
        assert_eq!(
            config.package_pin().catalog_revision(),
            bundle.catalog().catalog_revision()
        );
        assert_eq!(config.package_pin().components().len(), 1);
        assert!(
            bundle
                .configuration("ustc.simple-calendar", "0.2.0")
                .is_none()
        );
        assert!(
            bundle
                .configuration("ustc.affairs-navigator", "0.1.0")
                .is_none()
        );
        assert_eq!(bundle.catalog().packages().len(), 4);
    }

    #[test]
    fn market_bundle_revision_binds_sidecar_bytes_and_preserves_order_independence() {
        let bundle = MarketBundle::bundled().expect("valid reviewed bundle fixture");
        let changed = [CONFIGURATIONS[0].1, b" "].concat();
        let other = MarketBundle::from_sources(&MANIFESTS, &[(CONFIGURATIONS[0].0, &changed)])
            .expect("valid reviewed bundle fixture");
        assert_ne!(
            bundle.catalog().catalog_revision(),
            other.catalog().catalog_revision()
        );
        assert_ne!(
            bundle.catalog().catalog_digest(),
            other.catalog().catalog_digest()
        );
        assert_ne!(
            bundle
                .configuration("ustc.simple-calendar", "0.1.0")
                .expect("valid reviewed bundle fixture")
                .package_pin()
                .catalog_revision(),
            other
                .configuration("ustc.simple-calendar", "0.1.0")
                .expect("valid reviewed bundle fixture")
                .package_pin()
                .catalog_revision()
        );
        let mut reverse = MANIFESTS;
        reverse.reverse();
        assert_eq!(
            bundle.catalog(),
            MarketBundle::from_sources(&reverse, &CONFIGURATIONS)
                .expect("valid reviewed bundle fixture")
                .catalog()
        );
    }

    #[test]
    fn market_bundle_invalid_included_source_rejects_whole_snapshot() {
        for sidecars in [
            vec![(CONFIGURATIONS[0].0, b"invalid".as_slice())],
            vec![CONFIGURATIONS[0], CONFIGURATIONS[0]],
            vec![(
                "market/packages/unknown/configuration.json",
                CONFIGURATIONS[0].1,
            )],
        ] {
            assert!(MarketBundle::from_sources(&MANIFESTS, &sidecars).is_err());
        }
        assert!(MarketBundle::from_sources(&[MANIFESTS[0], MANIFESTS[0]], &[]).is_err());
        assert!(MarketBundle::from_sources(&vec![MANIFESTS[0]; 65], &[]).is_err());
        assert!(MarketBundle::from_sources(&[(MANIFESTS[0].0, b"invalid")], &[]).is_err());
        let absent =
            MarketBundle::from_sources(&MANIFESTS, &[]).expect("valid reviewed bundle fixture");
        assert!(
            absent
                .configuration("ustc.simple-calendar", "0.1.0")
                .is_none()
        );
    }

    #[test]
    fn market_bundle_source_budget_and_paths_are_bounded_before_publication() {
        let mut exact = CONFIGURATIONS[0].1.to_vec();
        exact.resize(MAX_SOURCE_BYTES, b' ');
        assert!(MarketBundle::from_sources(&MANIFESTS, &[(CONFIGURATIONS[0].0, &exact)]).is_ok());
        exact.push(b' ');
        assert!(MarketBundle::from_sources(&MANIFESTS, &[(CONFIGURATIONS[0].0, &exact)]).is_err());
        assert!(MarketBundle::from_sources(&MANIFESTS, &vec![CONFIGURATIONS[0]; 65]).is_err());
        for path in [
            "/market/packages/ustc.simple-calendar/configuration.json",
            "market/../configuration.json",
            "market//configuration.json",
            "market\\configuration.json",
        ] {
            assert!(
                MarketBundle::from_sources(&MANIFESTS, &[(path, CONFIGURATIONS[0].1)]).is_err()
            );
        }
        let sources = (0..17)
            .map(|index| {
                let mut package: serde_json::Value =
                    serde_json::from_slice(MANIFESTS[3].1).expect("valid reviewed bundle fixture");
                let id = format!("ustc.bound-{index}");
                package["id"] = serde_json::json!(id);
                let mut bytes =
                    serde_json::to_vec(&package).expect("valid reviewed bundle fixture");
                bytes.resize(MAX_SOURCE_BYTES, b' ');
                (format!("market/packages/{id}/package.json"), bytes)
            })
            .collect::<Vec<_>>();
        let refs = sources
            .iter()
            .map(|(path, bytes)| (path.as_str(), bytes.as_slice()))
            .collect::<Vec<_>>();
        assert!(MarketBundle::from_sources(&refs[..16], &[]).is_ok());
        assert!(MarketBundle::from_sources(&refs, &[]).is_err());
    }
}
