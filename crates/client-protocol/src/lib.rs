//! M10-owned wire-only client protocol.
//!
//! This crate has no dependency on platform authority, M70/M71 domain crates,
//! M60, server persistence, or a client implementation. Server-side conversion
//! lives in `application-ingress`; M80 reduction lives in `client-core`.

#![forbid(unsafe_code)]

pub mod affairs;
pub mod capsule;
pub mod change;
pub mod digest;
pub mod error;
pub mod opportunity;
pub mod transport;
pub mod value;

pub use affairs::*;
pub use capsule::*;
pub use change::*;
pub use digest::*;
pub use error::*;
pub use opportunity::*;
pub use transport::*;
pub use value::*;
