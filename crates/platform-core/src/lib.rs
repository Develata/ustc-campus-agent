//! Canonical domain constants and authority invariants for USTC Campus Agent.
//!
//! This crate is intentionally small at repository initialization time. It exists so
//! executable surfaces can depend on one shared source of truth instead of copying
//! product identifiers, package ids, and source-authority ordering.

/// Human-facing product name.
pub const PRODUCT_NAME: &str = "USTC Campus Agent";

/// First-party plugin id for the flagship opportunity graph package.
pub const OPPORTUNITY_GRAPH_PLUGIN_ID: &str = "ustc.opportunity-graph";

/// First vertical slice shipped through the opportunity graph plugin.
pub const FIRST_VERTICAL_SLICE: &str = "course-planning";

/// Source authority order for Course Planning facts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SourceAuthority {
    /// Model-only inference. It may explain but cannot establish material facts.
    ModelInference = 0,
    /// Community subjective signal such as iCourse review aggregates.
    CommunitySignal = 1,
    /// Public secondary mirrors such as iCourse program pages.
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

/// Current first-party flagship package identity.
pub const OPPORTUNITY_GRAPH_IDENTITY: PluginIdentity = PluginIdentity {
    id: OPPORTUNITY_GRAPH_PLUGIN_ID,
    version: "0.1.0",
};

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
    fn flagship_identity_is_stable() {
        assert_eq!(PRODUCT_NAME, "USTC Campus Agent");
        assert_eq!(OPPORTUNITY_GRAPH_IDENTITY.id, "ustc.opportunity-graph");
        assert_eq!(FIRST_VERTICAL_SLICE, "course-planning");
    }
}
