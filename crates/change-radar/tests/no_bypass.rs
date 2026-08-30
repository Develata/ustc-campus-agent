#![allow(clippy::unwrap_used)]

use std::fs;
use std::path::PathBuf;

fn crate_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn change_radar_domain_has_no_client_web_storage_or_executor_dependency() {
    let root = crate_root();
    let manifest = fs::read_to_string(root.join("Cargo.toml")).expect("manifest");
    for forbidden in [
        "axum",
        "serde",
        "client-protocol",
        "client-core",
        "application-ingress",
        "agent-runtime",
        "agent-tool-protocol",
    ] {
        assert!(
            !manifest.contains(forbidden),
            "domain manifest must not depend on {forbidden}"
        );
    }
    for path in ["src/lib.rs", "src/publication.rs"] {
        let source = fs::read_to_string(root.join(path)).expect("source");
        for forbidden in [
            "std::fs",
            "std::net",
            "axum::",
            "serde::",
            "ustc_campus_agent_client",
            "ustc_campus_agent_application_ingress",
            "PluginExecutor",
        ] {
            assert!(
                !source.contains(forbidden),
                "{path} must not reach {forbidden}"
            );
        }
    }
}
