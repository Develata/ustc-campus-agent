//! Closed M10-owned client protocol version and retained operation registry.

use serde::{Deserialize, Serialize};

mod compatibility;
mod registry;

pub use compatibility::*;
pub use registry::*;

pub const CLIENT_PROTOCOL_MAJOR_HEADER: &str = "x-ustc-client-protocol-major";
pub const CLIENT_PROTOCOL_SCHEMA: &str = "ustc-client-protocol/v1";
pub const CLIENT_RESULT_SCHEMA: &str = "ustc-client-result/v1";
pub const OPERATION_REGISTRY_REVISION: &str = "m10-affairs-first/v1";
pub const CURRENT_CLIENT_PROTOCOL_MAJOR: ClientProtocolMajor = ClientProtocolMajor::new(1);
pub const MINIMUM_CLIENT_PROTOCOL_MAJOR: ClientProtocolMajor = ClientProtocolMajor::new(1);
pub const SUPPORTED_CLIENT_PROTOCOL_MAJORS: [ClientProtocolMajor; 1] =
    [CURRENT_CLIENT_PROTOCOL_MAJOR];

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ClientProtocolMajor(u16);

impl ClientProtocolMajor {
    #[must_use]
    pub const fn new(value: u16) -> Self {
        Self(value)
    }

    #[must_use]
    pub const fn get(self) -> u16 {
        self.0
    }
}
