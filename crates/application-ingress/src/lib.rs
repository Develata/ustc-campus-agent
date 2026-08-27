//! M10 server-side admission, reconciliation, durable state and Affairs conversion.
//!
//! The crate owns an admitted-actor-aware [`AffairsInvocationPort`]. The concrete
//! Market/Agent/ToolGateway/M71 composition remains in `ustc-agentd`; M10 has no
//! M20/M30/M40/M60 dependency and M80/client code depends only on the wire protocol.

#![forbid(unsafe_code)]

pub mod capability;
pub mod m00_projection;
pub mod m71_projection;
pub mod persistence;
pub mod service;

pub use capability::{CapabilityError, CapabilityIssuer, StoredPublicAuthorization};
pub use persistence::{
    FileRecordStore, RecordState, StoreError, StoredReadPolicy, StoredRecord, capsule_digest,
};
pub use service::{AffairsInvocationError, AffairsInvocationPort, M10AdmissionPorts, M10Service};
