//! Canonical domain constants and authority invariants for USTC Campus Agent.
//!
//! This crate is intentionally small at repository initialization time. It exists so
//! executable surfaces can depend on one shared source of truth instead of copying
//! product identifiers, package ids, and source-authority ordering.

pub mod identity;
pub mod invocation;
pub mod market;
pub mod session;
pub mod source_registry;

/// Human-facing product name.
pub const PRODUCT_NAME: &str = "USTC Campus Agent";

/// First-party plugin id for USTC Affairs Navigator.
pub const AFFAIRS_NAVIGATOR_PLUGIN_ID: &str = "ustc.affairs-navigator";

/// First-party plugin id for USTC ChangeRadar.
pub const CHANGE_RADAR_PLUGIN_ID: &str = "ustc.change-radar";

/// First-party plugin id for Campus Opportunity Graph.
pub const OPPORTUNITY_GRAPH_PLUGIN_ID: &str = "ustc.opportunity-graph";

/// Bounded offline spike currently implemented inside Opportunity Graph.
pub const COURSE_PLANNING_SLICE: &str = "course-planning";

/// Source authority order for Course Planning facts.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
#[serde(rename_all = "snake_case")]
pub enum SourceAuthority {
    /// Model-only inference. It may explain but cannot establish material facts.
    ModelInference = 0,
    /// Community subjective signal such as iCourse review aggregates.
    CommunitySignal = 1,
    /// Public secondary mirrors such as iCourse program pages.
    #[serde(rename = "icourse_mirror")]
    ICourseMirror = 2,
    /// Reviewed official USTC/department notice outside the catalog.
    ReviewedOfficialSource = 3,
    /// Approved snapshot or future official API from the USTC catalog service.
    OfficialCatalogSnapshot = 4,
}

impl SourceAuthority {
    /// Returns true when `self` may override `other` during conflict resolution.
    #[must_use]
    pub const fn outranks(self, other: Self) -> bool {
        (self as u8) > (other as u8)
    }
}

/// Minimal package identity for market contracts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PluginIdentity {
    /// Reverse-DNS-style package id.
    pub id: &'static str,
    /// SemVer package version.
    pub version: &'static str,
}

/// Current Affairs Navigator package identity.
pub const AFFAIRS_NAVIGATOR_IDENTITY: PluginIdentity = PluginIdentity {
    id: AFFAIRS_NAVIGATOR_PLUGIN_ID,
    version: "0.1.0",
};

/// Current ChangeRadar package identity.
pub const CHANGE_RADAR_IDENTITY: PluginIdentity = PluginIdentity {
    id: CHANGE_RADAR_PLUGIN_ID,
    version: "0.1.0",
};

/// Current Opportunity Graph package identity.
pub const OPPORTUNITY_GRAPH_IDENTITY: PluginIdentity = PluginIdentity {
    id: OPPORTUNITY_GRAPH_PLUGIN_ID,
    version: "0.1.0",
};

/// Exact, stable identities of the three default first-party Plugins.
///
/// This is product/catalog display order, not implementation order. The latter starts with the
/// ChangeRadar source/revision/diff foundation and is governed by ADR-0006.
pub const DEFAULT_FIRST_PARTY_PLUGIN_IDENTITIES: [PluginIdentity; 3] = [
    AFFAIRS_NAVIGATOR_IDENTITY,
    CHANGE_RADAR_IDENTITY,
    OPPORTUNITY_GRAPH_IDENTITY,
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn official_catalog_outranks_community_sources() {
        assert!(
            SourceAuthority::OfficialCatalogSnapshot.outranks(SourceAuthority::CommunitySignal)
        );
        assert!(SourceAuthority::ICourseMirror.outranks(SourceAuthority::ModelInference));
        assert!(
            !SourceAuthority::CommunitySignal.outranks(SourceAuthority::ReviewedOfficialSource)
        );
    }

    #[test]
    fn default_first_party_plugin_identities_are_stable() {
        assert_eq!(PRODUCT_NAME, "USTC Campus Agent");
        assert_eq!(
            DEFAULT_FIRST_PARTY_PLUGIN_IDENTITIES,
            [
                PluginIdentity {
                    id: "ustc.affairs-navigator",
                    version: "0.1.0",
                },
                PluginIdentity {
                    id: "ustc.change-radar",
                    version: "0.1.0",
                },
                PluginIdentity {
                    id: "ustc.opportunity-graph",
                    version: "0.1.0",
                },
            ]
        );
        assert_eq!(COURSE_PLANNING_SLICE, "course-planning");
    }

    #[test]
    fn manifest_identities_match_rust_authority() {
        let manifests = [
            (
                AFFAIRS_NAVIGATOR_IDENTITY,
                include_str!("../../../market/packages/ustc.affairs-navigator/package.json"),
            ),
            (
                CHANGE_RADAR_IDENTITY,
                include_str!("../../../market/packages/ustc.change-radar/package.json"),
            ),
            (
                OPPORTUNITY_GRAPH_IDENTITY,
                include_str!("../../../market/packages/ustc.opportunity-graph/package.json"),
            ),
        ];

        for (identity, source) in manifests {
            let Ok(manifest) = serde_json::from_str::<serde_json::Value>(source) else {
                panic!("first-party manifest must be valid JSON");
            };
            assert_eq!(
                manifest.get("id").and_then(serde_json::Value::as_str),
                Some(identity.id)
            );
            assert_eq!(
                manifest.get("version").and_then(serde_json::Value::as_str),
                Some(identity.version)
            );
        }
    }
}
