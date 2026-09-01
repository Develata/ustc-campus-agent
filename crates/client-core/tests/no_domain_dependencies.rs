#![allow(clippy::unwrap_used)]

//! Static dependency-confinement and reducer-exhaustiveness proofs.
//!
//! These tests are sabotage guards: they read manifests and reducer source via
//! `include_str!` and fail if a forbidden dependency, a wildcard match arm, or
//! a non-allowlisted crate appears. They prove the M80 boundary
//! (`client-shell/v2.1` §14, taskbook §3.3) without running `cargo tree`.

/// client-core manifest (this crate).
const CLIENT_CORE_CARGO: &str = include_str!("../Cargo.toml");
/// M10 wire manifest (the only domain-adjacent dependency).
const CLIENT_PROTOCOL_CARGO: &str = include_str!("../../client-protocol/Cargo.toml");
/// ustc-agent manifest (the peer CLI that consumes client-core).
const USTC_AGENT_CARGO: &str = include_str!("../../../apps/ustc-agent/Cargo.toml");
/// Reducer source — scanned for wildcard match arms.
const REDUCER_SRC: &str = include_str!("../src/reducer.rs");

/// Crates that must never appear as a direct dependency of client-core or
/// ustc-agent, and never as a direct dependency of client-protocol (which is
/// the only crate client-core depends on besides serde/serde_json).
const FORBIDDEN_DEPS: &[&str] = &[
    "ustc-campus-agent-core",
    "ustc-campus-agent-runtime",
    "platform-core",
    "affairs-navigator",
    "application-ingress",
    "agent-runtime",
    "agent-tool-protocol",
    "ustc-agentd",
    "ustc-agentctl",
    "adapters",
    "course-planning",
    "time",
    "semver",
    "tokio",
    "reqwest",
    "hyper",
    "clap",
];

/// The only crates client-core may depend on directly.
const CLIENT_CORE_ALLOWED: &[&str] = &["ustc-campus-agent-client-protocol", "serde", "serde_json"];

/// The only crates client-protocol may depend on directly. `sha2` is the
/// M10-owned deterministic payload-digest primitive; it is not a domain,
/// transport, runtime, or outer-shell dependency.
const CLIENT_PROTOCOL_ALLOWED: &[&str] = &["serde", "serde_json", "sha2"];

/// The only crates ustc-agent may depend on directly.
const USTC_AGENT_ALLOWED: &[&str] = &["ustc-campus-agent-client-core", "serde", "serde_json"];

fn is_dep_section(section: &str) -> bool {
    // Matches [dependencies], [dev-dependencies], [build-dependencies],
    // [target.<cfg>.dependencies], [target.<cfg>.dev-dependencies],
    // [target.<cfg>.build-dependencies], and their table forms
    // [dependencies.foo], [dev-dependencies.bar], etc.
    section.contains("dependencies")
}

fn dep_lines(manifest: &str) -> Vec<String> {
    let mut in_deps = false;
    let mut out = Vec::new();
    for line in manifest.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            in_deps = is_dep_section(trimmed);
            if in_deps {
                // Handle table form [dependencies.foo] → dep name is "foo".
                let inner = trimmed.trim_start_matches('[').trim_end_matches(']');
                if let Some(last_dot) = inner.rfind('.') {
                    let after_dot = &inner[last_dot + 1..];
                    if !["dependencies", "dev-dependencies", "build-dependencies"]
                        .contains(&after_dot)
                    {
                        out.push(after_dot.to_string());
                    }
                }
            }
            continue;
        }
        if in_deps && !trimmed.is_empty() && !trimmed.starts_with('#') {
            let key = trimmed.split('=').next().unwrap_or("").trim();
            // `serde.workspace = true` → dep name is `serde` (before first `.`);
            // `foo = { path = ... }` → dep name is `foo`.
            let name = key.split('.').next().unwrap_or("").trim().to_string();
            if !name.is_empty() {
                out.push(name);
            }
        }
    }
    out
}

#[test]
fn client_core_dependencies_are_allowlisted() {
    let deps = dep_lines(CLIENT_CORE_CARGO);
    assert!(!deps.is_empty(), "client-core must declare dependencies");
    for dep in &deps {
        assert!(
            CLIENT_CORE_ALLOWED.contains(&dep.as_str()),
            "client-core declares non-allowlisted dependency `{dep}`"
        );
    }
    for forbidden in FORBIDDEN_DEPS {
        assert!(
            !deps.iter().any(|d| d == forbidden),
            "client-core declares forbidden dependency `{forbidden}`"
        );
    }
}

#[test]
fn client_protocol_dependencies_are_allowlisted() {
    let deps = dep_lines(CLIENT_PROTOCOL_CARGO);
    assert!(
        !deps.is_empty(),
        "client-protocol must declare dependencies"
    );
    for dep in &deps {
        assert!(
            CLIENT_PROTOCOL_ALLOWED.contains(&dep.as_str()),
            "client-protocol declares non-allowlisted dependency `{dep}`"
        );
    }
    for forbidden in FORBIDDEN_DEPS {
        assert!(
            !deps.iter().any(|d| d == forbidden),
            "client-protocol declares forbidden dependency `{forbidden}`"
        );
    }
}

#[test]
fn ustc_agent_dependencies_are_allowlisted() {
    let deps = dep_lines(USTC_AGENT_CARGO);
    assert!(!deps.is_empty(), "ustc-agent must declare dependencies");
    for dep in &deps {
        assert!(
            USTC_AGENT_ALLOWED.contains(&dep.as_str()),
            "ustc-agent declares non-allowlisted dependency `{dep}`"
        );
    }
    for forbidden in FORBIDDEN_DEPS {
        assert!(
            !deps.iter().any(|d| d == forbidden),
            "ustc-agent declares forbidden dependency `{forbidden}`"
        );
    }
}

#[test]
fn client_core_first_party_closure_is_confined() {
    // client-core -> {client-protocol, serde, serde_json}
    // client-protocol -> {serde, serde_json, sha2}
    // This source-level proof confines the workspace-visible/direct boundary
    // and rejects every listed domain/runtime dependency. Cargo's resolved
    // graph and the repository contract checker own crates.io source/version
    // admission; this test does not pretend to enumerate all registry
    // transitives of serde or sha2.
    let core_deps = dep_lines(CLIENT_CORE_CARGO);
    let proto_deps = dep_lines(CLIENT_PROTOCOL_CARGO);
    let mut closure = core_deps.clone();
    closure.extend(proto_deps.iter().cloned());
    for forbidden in FORBIDDEN_DEPS {
        assert!(
            !closure.iter().any(|d| d == forbidden),
            "transitive closure contains forbidden dependency `{forbidden}`"
        );
    }
}

#[test]
fn reducer_has_no_wildcard_match_arms() {
    // A `_ =>` arm in the reducer would silently catch any future wire variant
    // and hide it from the exhaustive class projection. This is the
    // sabotage/mutation guard for reducer exhaustiveness: if anyone adds a
    // `_ =>` wildcard to reducer.rs, this test fails.
    assert!(
        !REDUCER_SRC.contains("_ =>"),
        "reducer.rs must not contain a `_ =>` wildcard match arm; \
         every wire variant must be matched explicitly"
    );
}

#[test]
fn reducer_matches_every_response_variant() {
    // Source-level proof that each ClientResponseDto variant is named in an
    // explicit arm of reduce_response.
    for variant in [
        "ClientResponseDto::ServerInfo",
        "ClientResponseDto::Capabilities",
        "ClientResponseDto::Compatibility",
        "ClientResponseDto::Accepted",
        "ClientResponseDto::ChangeFeedAccepted",
        "ClientResponseDto::OpportunityAccepted",
        "ClientResponseDto::OpportunityRejected",
        "ClientResponseDto::Available",
        "ClientResponseDto::Incomplete",
        "ClientResponseDto::Unavailable",
        "ClientResponseDto::Error",
    ] {
        assert!(
            REDUCER_SRC.contains(variant),
            "reducer.rs must explicitly match {variant}"
        );
    }
}

#[test]
fn reducer_matches_every_outcome_variant() {
    for variant in [
        "M71OutcomeDto::Found",
        "M71OutcomeDto::NotYetKnown",
        "M71OutcomeDto::Archived",
        "M71OutcomeDto::NotFound",
        "M71OutcomeDto::Conflict",
        "M71OutcomeDto::CannotVerify",
    ] {
        assert!(
            REDUCER_SRC.contains(variant),
            "reducer.rs must explicitly match {variant}"
        );
    }
}

#[test]
fn reducer_matches_every_lineage_variant() {
    for variant in [
        "M71LineageDto::Verified",
        "M71LineageDto::Unverified",
        "M71LineageDto::NotRequired",
    ] {
        assert!(
            REDUCER_SRC.contains(variant),
            "reducer.rs must explicitly match {variant}"
        );
    }
}

#[test]
fn reducer_matches_every_wire_error_class() {
    for variant in [
        "WireErrorClassDto::IdempotencyStoreUnavailable",
        "WireErrorClassDto::ConflictingEnvelope",
        "WireErrorClassDto::DescriptorSnapshotAbsent",
        "WireErrorClassDto::DescriptorSnapshotMismatch",
        "WireErrorClassDto::PolicyDenied",
        "WireErrorClassDto::PolicyExpired",
        "WireErrorClassDto::SessionNotFound",
        "WireErrorClassDto::SessionIdMismatch",
        "WireErrorClassDto::SessionNotAdmitted",
        "WireErrorClassDto::CapabilityMissing",
        "WireErrorClassDto::CapabilityDisabled",
        "WireErrorClassDto::CapabilityRevoked",
        "WireErrorClassDto::InfrastructurePortUnavailable",
        "WireErrorClassDto::MalformedCommand",
    ] {
        assert!(
            REDUCER_SRC.contains(variant),
            "reducer.rs must explicitly match {variant}"
        );
    }
}

#[test]
fn reducer_matches_every_redaction_variant() {
    for variant in [
        "RedactionDto::Public",
        "RedactionDto::AuthenticatedOwner",
        "RedactionDto::Operator",
    ] {
        assert!(
            REDUCER_SRC.contains(variant),
            "reducer.rs must explicitly match {variant}"
        );
    }
}

#[test]
fn reducer_matches_every_freshness_and_reason_variant() {
    for variant in [
        "FreshnessDto::Fresh",
        "FreshnessDto::Stale",
        "CannotVerifyReasonDto::SourceRevisionUnverified",
        "CannotVerifyReasonDto::EffectiveIntervalMissing",
        "CannotVerifyReasonDto::LastVerifiedStaleBeyondPolicy",
        "CannotVerifyReasonDto::PublicEvidenceProjectionOverflow",
    ] {
        assert!(
            REDUCER_SRC.contains(variant),
            "reducer.rs must explicitly match {variant}"
        );
    }
}

#[test]
fn no_print_or_debug_logging_in_client_core_source() {
    // client-core is a library: it returns typed values and never prints. The
    // CLI owns stdout. This guard fails if anyone introduces a print/log macro
    // into the reducer, transport, or intent constructors, which is the
    // channel through which a capability/capsule/secret could leak.
    let lib_src = include_str!("../src/lib.rs");
    let transport_src = include_str!("../src/transport.rs");
    for src in [REDUCER_SRC, transport_src, lib_src] {
        for macro_name in [
            "eprintln!",
            "println!",
            "dbg!",
            "print!",
            "tracing::",
            "log::",
        ] {
            assert!(
                !src.contains(macro_name),
                "client-core source must not use `{macro_name}`; the CLI owns stdout and no \
                 capability/capsule/secret may be logged"
            );
        }
    }
}

#[test]
fn client_protocol_reexports_match_wire_surface() {
    // The wire crate re-exports its modules via `pub use <module>::*`. Confirm
    // the module set is exactly the expected M10 wire surface so client-core
    // cannot accidentally depend on a hidden server module.
    let proto_lib = include_str!("../../client-protocol/src/lib.rs");
    for module in ["affairs", "capsule", "error", "transport", "value"] {
        assert!(
            proto_lib.contains(&format!("pub mod {module};")),
            "client-protocol must expose module `{module}`"
        );
    }
}

#[test]
fn secret_bearing_types_have_manual_debug_not_derived() {
    // TerminalKind and ClientState directly or transitively contain a
    // capability bearer (public_capability). They must implement Debug
    // manually with redaction, not derive it. This mutation guard fails if
    // anyone re-adds #[derive(Debug)] to either type.
    for type_name in ["TerminalKind", "ClientState"] {
        let needle = format!("pub enum {type_name}");
        let lines: Vec<&str> = REDUCER_SRC.lines().collect();
        let mut found_def = false;
        for (i, line) in lines.iter().enumerate() {
            if line.contains(&needle) {
                found_def = true;
                // Check preceding 3 lines for a derive attribute containing Debug.
                for prev in lines[i.saturating_sub(3)..i].iter() {
                    assert!(
                        !(prev.contains("#[derive") && prev.contains("Debug")),
                        "{type_name} must not derive Debug; it carries a capability bearer \
                         and must use a manual redacted Debug impl. Found derive at: {prev}"
                    );
                }
                break;
            }
        }
        assert!(found_def, "{type_name} definition not found in reducer.rs");
        // Confirm a manual Debug impl exists for this type.
        let impl_needle = format!("impl std::fmt::Debug for {type_name}");
        assert!(
            REDUCER_SRC.contains(&impl_needle),
            "{type_name} must have a manual `impl std::fmt::Debug`; found neither derived nor \
             manual Debug"
        );
    }
}

#[test]
fn dep_lines_catches_dev_dependencies() {
    let manifest = r#"
[package]
name = "test"

[dependencies]
serde = "1"

[dev-dependencies]
tokio = "1"
"#;
    let deps = dep_lines(manifest);
    assert!(
        deps.contains(&"tokio".to_string()),
        "dep_lines must parse [dev-dependencies]; got {deps:?}"
    );
    assert!(deps.contains(&"serde".to_string()));
}

#[test]
fn dep_lines_catches_build_dependencies() {
    let manifest = r#"
[package]
name = "test"

[dependencies]
serde = "1"

[build-dependencies]
sha2 = "0.10"
"#;
    let deps = dep_lines(manifest);
    assert!(
        deps.contains(&"sha2".to_string()),
        "dep_lines must parse [build-dependencies]; got {deps:?}"
    );
}

#[test]
fn dep_lines_catches_target_dependencies() {
    let manifest = r#"
[package]
name = "test"

[dependencies]
serde = "1"

[target.'cfg(unix)'.dependencies]
reqwest = "0.11"
"#;
    let deps = dep_lines(manifest);
    assert!(
        deps.contains(&"reqwest".to_string()),
        "dep_lines must parse [target.*.dependencies]; got {deps:?}"
    );
}

#[test]
fn dep_lines_catches_target_dev_dependencies() {
    let manifest = r#"
[package]
name = "test"

[dependencies]
serde = "1"

[target.'cfg(windows)'.dev-dependencies]
hyper = "1"
"#;
    let deps = dep_lines(manifest);
    assert!(
        deps.contains(&"hyper".to_string()),
        "dep_lines must parse [target.*.dev-dependencies]; got {deps:?}"
    );
}

#[test]
fn dep_lines_catches_table_form_dependency() {
    let manifest = r#"
[package]
name = "test"

[dependencies]
serde = "1"

[dependencies.clap]
version = "4"
features = ["derive"]
"#;
    let deps = dep_lines(manifest);
    assert!(
        deps.contains(&"clap".to_string()),
        "dep_lines must parse [dependencies.clap] table form; got {deps:?}"
    );
    assert!(deps.contains(&"serde".to_string()));
}

#[test]
fn dep_lines_does_not_collect_non_dependency_sections() {
    let manifest = r#"
[package]
name = "test"

[dependencies]
serde = "1"

[features]
default = []

[[bin]]
name = "test"
path = "src/main.rs"
"#;
    let deps = dep_lines(manifest);
    assert_eq!(deps, vec!["serde".to_string()]);
}
