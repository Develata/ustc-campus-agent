//! Replaceable adapter boundary.
//!
//! Adapters can talk to model providers, MCP servers, HTTP sources, Git catalogs,
//! object storage, or databases. They must not own canonical grants, approvals,
//! receipts, source revisions, or market catalog truth.

pub mod openai_responses;

use ustc_campus_agent_core::PRODUCT_NAME;

/// Returns a small health string used by binary smoke tests.
#[must_use]
pub fn adapter_health() -> String {
    format!("{PRODUCT_NAME} adapters: ok")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adapter_health_mentions_product() {
        assert!(adapter_health().contains("USTC Campus Agent"));
    }
}
