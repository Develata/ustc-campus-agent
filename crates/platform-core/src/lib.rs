//! Canonical domain constants and authority invariants for USTC Campus Agent.
//!
//! This crate is intentionally small at repository initialization time. It exists so
//! executable surfaces can depend on one shared source of truth instead of copying
//! product identifiers, package ids, and source-authority ordering.

pub mod control_evidence;
pub mod identity;
pub mod invocation;
pub mod market;
pub mod request_context;
pub mod session;
pub mod session_port;
pub mod source_registry;
pub mod source_revision;

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

/// Generic source authority class for the `M60` source pipeline.
///
/// This enum carries **no product-specific ontology** (no `ICourseMirror`,
/// `OfficialCatalogSnapshot`, etc.) and defines **no semantic total order**.
/// Distinct authority classes are `Incomparable`; a product module may refine
/// them into a local total order behind its own policy/type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceAuthority {
    /// Model-only inference. It may explain but cannot establish material facts.
    /// Rejected at `SourceDefinition::proposed` admission.
    ModelInference,
    /// Community subjective signal (generic class; product modules name concrete sources).
    CommunitySignal,
    /// Reviewed official source outside a product-specific catalog (generic class).
    ReviewedOfficialSource,
}

/// Policy-scoped comparison result for generic source authority.
///
/// The generic `M60` source-registry never ranks distinct authority classes.
/// `Higher`/`Lower` are reserved for product-local policy refinements; the
/// generic `SourceAuthority::compare` returns only `Equivalent` or `Incomparable`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthorityComparison {
    Higher,
    Lower,
    Equivalent,
    Incomparable,
}

impl SourceAuthority {
    /// Compare two generic authority classes under policy-scoped comparison.
    ///
    /// Identical values are `Equivalent`; distinct classes are `Incomparable`.
    /// `Incomparable` material facts create conflict or `cannot_verify`; they are
    /// never selected by a numeric total order.
    #[must_use]
    pub const fn compare(self, other: Self) -> AuthorityComparison {
        if self as u8 == other as u8 {
            AuthorityComparison::Equivalent
        } else {
            AuthorityComparison::Incomparable
        }
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
    fn generic_authority_comparison_is_partial_not_total() {
        assert_eq!(
            SourceAuthority::ReviewedOfficialSource
                .compare(SourceAuthority::ReviewedOfficialSource),
            AuthorityComparison::Equivalent
        );
        assert_eq!(
            SourceAuthority::CommunitySignal.compare(SourceAuthority::CommunitySignal),
            AuthorityComparison::Equivalent
        );
        assert_eq!(
            SourceAuthority::ModelInference.compare(SourceAuthority::ModelInference),
            AuthorityComparison::Equivalent
        );
        assert_eq!(
            SourceAuthority::ReviewedOfficialSource.compare(SourceAuthority::CommunitySignal),
            AuthorityComparison::Incomparable
        );
        assert_eq!(
            SourceAuthority::CommunitySignal.compare(SourceAuthority::ReviewedOfficialSource),
            AuthorityComparison::Incomparable
        );
        assert_eq!(
            SourceAuthority::ModelInference.compare(SourceAuthority::CommunitySignal),
            AuthorityComparison::Incomparable
        );
        assert!(AuthorityComparison::Incomparable != AuthorityComparison::Equivalent);
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
