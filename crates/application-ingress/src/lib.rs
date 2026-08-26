//! M10 server-side admission, reconciliation, durable state and M71 conversion.
//!
//! The crate has exactly one M71 application dependency and no M60 dependency. Composition is
//! owned by `ustc-agentd`; M80/client code depends only on the wire protocol.

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
pub use service::{M10AdmissionPorts, M10Service};
