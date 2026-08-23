//! M10-owned wire-only client protocol.
//!
//! This crate has no dependency on platform authority, M71, M60, server persistence, or a client
//! implementation. Server-side conversion lives in `application-ingress`; M80 reduction lives in
//! `client-core`.

#![forbid(unsafe_code)]

pub mod affairs;
pub mod capsule;
pub mod error;
pub mod transport;
pub mod value;

pub use affairs::*;
pub use capsule::*;
pub use error::*;
pub use transport::*;
pub use value::*;
