//! M10 server-side admission, reconciliation, durable state and product conversion.
//!
//! The crate owns admitted-actor-aware Affairs and ChangeRadar invocation ports.
//! Concrete Market/Agent/ToolGateway/owning-Plugin composition remains in
//! `ustc-agentd`; M10 has no M20/M30/M40/M60 dependency and M80/client code
//! depends only on the wire protocol.

#![forbid(unsafe_code)]

pub mod affairs_publication;
pub mod capability;
pub mod change_publication;
pub mod m00_projection;
pub mod m70_projection;
pub mod m70_service;
pub mod m71_projection;
pub mod m72_projection;
pub mod m72_service;
pub mod persistence;
pub mod service;

pub use affairs_publication::{
    AffairsPublicationApplicationError, AffairsPublicationApplicationPort,
    AffairsPublicationCommand, AffairsPublicationEvidenceError, AffairsPublicationOutcome,
    M10AffairsPublicationService, affairs_publication_payload_digest,
};
pub use capability::{CapabilityError, CapabilityIssuer, StoredPublicAuthorization};
pub use change_publication::{
    ChangePublicationApplicationError, ChangePublicationApplicationPort, ChangePublicationCommand,
    ChangePublicationEvidenceError, ChangePublicationOutcome, M10ChangePublicationService,
    change_publication_payload_digest,
};
pub use m70_service::{
    ChangeFeedInvocationError, ChangeFeedInvocationOutcome, ChangeFeedInvocationPort,
    M10ChangeFeedService,
};
pub use m72_service::{
    M10OpportunityService, OpportunityInvocationError, OpportunityInvocationOutcome,
    OpportunityInvocationPort,
};
pub use persistence::{
    FileRecordStore, RecordState, StoreError, StoredReadPolicy, StoredRecord, capsule_digest,
};
pub use service::{AffairsInvocationError, AffairsInvocationPort, M10AdmissionPorts, M10Service};
