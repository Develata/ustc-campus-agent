//! Consent-bound Campus Opportunity Graph domain foundation.
//!
//! Public reviewed opportunity facts and tenant-private profile payloads remain
//! separate. Planning reads one exact private snapshot only after principal and
//! consent checks, then delegates hard feasibility to the deterministic Course
//! Planning pack. This crate owns no transport, UI, Market lifecycle, source
//! retrieval, credential, or durable storage implementation.

mod catalog;
mod repository;
mod service;
mod value;

pub use catalog::{
    CatalogConstructionError, CourseQualification, QualificationBlocker, ReviewedOpportunityCatalog,
};
pub use repository::{
    InMemoryOpportunityProfileRepository, OpportunityProfileRepository, OpportunityRepositoryError,
    ProfileLookup, RepositoryFailureMode, TenantProfileRecord,
};
pub use service::{
    M60OpportunityPort, MAX_OPPORTUNITY_BEAM_WIDTH, MAX_OPPORTUNITY_PLAN_RESULTS,
    OpportunityPlanDecision, OpportunityPlanReceipt, OpportunityPlanningError,
    OpportunityPlanningService, OpportunityProfileError, OpportunityProfileService,
    OpportunitySourcePortError, PlanStaleness,
};
pub use value::{
    AcademicProfileInput, AuthenticatedPrincipal, ConsentField, ConsentGrant, ConsentId,
    ConsentPurpose, DeletionReceipt, DeletionReceiptId, OpportunityValueError, ProfileSnapshotId,
    ProfileTombstone,
};
