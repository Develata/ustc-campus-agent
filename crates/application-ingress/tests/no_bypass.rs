//! No-bypass proof: application-ingress has no M60 port/fixture/repository dependency,
//! no client-core dependency, and respects the M71 import allowlist.
//!
//! The structural proof is the Cargo dependency graph (application-ingress depends only on
//! client-protocol, platform-core, and affairs-navigator). These tests verify that the source
//! code does not reach forbidden affairs-navigator internals (M60 port, M60 fixture, repository,
//! service constructor) even though they are re-exported and importable.

const SERVICE: &str = include_str!("../src/service.rs");
const M71_PROJECTION: &str = include_str!("../src/m71_projection.rs");
const M00_PROJECTION: &str = include_str!("../src/m00_projection.rs");
const PERSISTENCE: &str = include_str!("../src/persistence.rs");
const CAPABILITY: &str = include_str!("../src/capability.rs");
const LIB: &str = include_str!("../src/lib.rs");
const CLIENT_CAPSULE: &str = include_str!("../../client-protocol/src/capsule.rs");
const CLIENT_TRANSPORT: &str = include_str!("../../client-protocol/src/transport.rs");

const ALL_SOURCES: [&str; 6] = [
    SERVICE,
    M71_PROJECTION,
    M00_PROJECTION,
    PERSISTENCE,
    CAPABILITY,
    LIB,
];

/// `ustc-agent-runtime` (M00 runtime) is not a dependency of application-ingress.
#[cfg(doctest)]
#[doc = "```compile_fail
use ustc_agent_runtime;
```"]
const _NO_AGENT_RUNTIME_DEPENDENCY: () = ();

/// `ustc-campus-agent-client-core` (M80) is not a dependency of application-ingress.
#[cfg(doctest)]
#[doc = "```compile_fail
use ustc_campus_agent_client_core;
```"]
const _NO_CLIENT_CORE_DEPENDENCY: () = ();

/// `adapters` is not a dependency of application-ingress.
#[cfg(doctest)]
#[doc = "```compile_fail
use adapters;
```"]
const _NO_ADAPTERS_DEPENDENCY: () = ();

/// `FileRecordStore::claim` is pub(crate) — not callable from downstream.
#[cfg(doctest)]
#[doc = "```compile_fail
use ustc_campus_agent_application_ingress::FileRecordStore;
fn _probe(store: &FileRecordStore) { let _ = store.claim(\"x\", 0, 1); }
```"]
const _NO_PUBLIC_CLAIM: () = ();

/// `FileRecordStore::insert_admitted_once` is pub(crate) — not callable from downstream.
#[cfg(doctest)]
#[doc = "```compile_fail
use ustc_campus_agent_application_ingress::FileRecordStore;
fn _probe(store: &FileRecordStore) { let _ = store.insert_admitted_once; }
```"]
const _NO_PUBLIC_INSERT: () = ();

/// `FileRecordStore::complete` is pub(crate) — not callable from downstream.
#[cfg(doctest)]
#[doc = "```compile_fail
use ustc_campus_agent_application_ingress::FileRecordStore;
fn _probe(store: &FileRecordStore) { let _ = store.complete; }
```"]
const _NO_PUBLIC_COMPLETE: () = ();

/// `ClaimToken` is pub(crate) — not constructible or importable from downstream.
#[cfg(doctest)]
#[doc = "```compile_fail
use ustc_campus_agent_application_ingress::ClaimToken;
```"]
const _NO_PUBLIC_CLAIM_TOKEN: () = ();

/// `M10Service::store` is removed — downstream must not reach the FileRecordStore
/// directly; only the service's typed response surfaces are public.
#[cfg(doctest)]
#[doc = "```compile_fail
use ustc_campus_agent_application_ingress::M10Service;
fn _probe<'a, P>(svc: &M10Service<'a>) { let _ = svc.store(); }
```"]
const _NO_PUBLIC_STORE_ACCESSOR: () = ();

/// `FileRecordStore::get` is pub(crate) — not callable from downstream.
#[cfg(doctest)]
#[doc = "```compile_fail
use ustc_campus_agent_application_ingress::FileRecordStore;
fn _probe(store: &FileRecordStore) { let _ = store.get(\"x\"); }
```"]
const _NO_PUBLIC_GET: () = ();

#[test]
fn no_m60_port_trait_imported() {
    for src in &ALL_SOURCES {
        assert!(
            !src.contains("M60ProcedureEvidencePort"),
            "application-ingress must not import M60ProcedureEvidencePort"
        );
    }
}

#[test]
fn no_m60_retained_evidence_types_imported() {
    for src in &ALL_SOURCES {
        assert!(
            !src.contains("M60RetainedEvidenceRequest")
                && !src.contains("M60RetainedEvidenceOutcome"),
            "application-ingress must not import M60 retained-evidence types"
        );
    }
}

#[test]
fn no_m60_verification_types_imported() {
    for src in &ALL_SOURCES {
        assert!(
            !src.contains("M60VerificationIdentity") && !src.contains("M60VerifiedEvidenceSet"),
            "application-ingress must not import M60 verification types"
        );
    }
}

#[test]
fn no_m60_evidence_port_error_imported() {
    for src in &ALL_SOURCES {
        assert!(
            !src.contains("M60EvidencePortError"),
            "application-ingress must not import M60EvidencePortError"
        );
    }
}

#[test]
fn no_m60_fixture_adapter_imported() {
    for src in &ALL_SOURCES {
        assert!(
            !src.contains("M60FixtureAdapter"),
            "application-ingress must not import M60FixtureAdapter"
        );
    }
}

#[test]
fn no_repository_types_imported() {
    for src in &ALL_SOURCES {
        assert!(
            !src.contains("InMemoryAffairsRepository")
                && !src.contains("AffairsRepository")
                && !src.contains("RepositorySeedError"),
            "application-ingress must not import affairs-navigator repository types"
        );
    }
}

#[test]
fn no_affairs_get_service_imported() {
    for src in &ALL_SOURCES {
        assert!(
            !src.contains("AffairsGetService"),
            "application-ingress must not import AffairsGetService (only M71AffairsGetPort and M71AffairsGetReceipt)"
        );
    }
}

#[test]
fn no_forbidden_module_paths() {
    for src in &ALL_SOURCES {
        assert!(
            !src.contains("affairs_navigator::m60_port"),
            "application-ingress must not import from affairs_navigator::m60_port module"
        );
        assert!(
            !src.contains("affairs_navigator::m60_fixture"),
            "application-ingress must not import from affairs_navigator::m60_fixture module"
        );
        assert!(
            !src.contains("affairs_navigator::repository"),
            "application-ingress must not import from affairs_navigator::repository module"
        );
        assert!(
            !src.contains("affairs_navigator::service"),
            "application-ingress must not import from affairs_navigator::service module"
        );
    }
}

#[test]
fn m71_evidence_unverified_reason_is_allowed() {
    assert!(
        M71_PROJECTION.contains("M60EvidenceUnverifiedReason"),
        "M60EvidenceUnverifiedReason is a sealed lineage accessor type required by exhaustive conversion"
    );
}

#[test]
fn permitted_m71_imports_present() {
    assert!(
        SERVICE.contains("M71AffairsGetPort"),
        "service must import M71AffairsGetPort"
    );
    assert!(
        M71_PROJECTION.contains("M71AffairsGetReceipt"),
        "m71_projection must import M71AffairsGetReceipt"
    );
    assert!(
        M71_PROJECTION.contains("GetProcedureOutcome"),
        "m71_projection must import GetProcedureOutcome"
    );
    assert!(
        SERVICE.contains("GetProcedureError"),
        "service must import GetProcedureError"
    );
}

#[test]
fn no_production_panic_paths() {
    let persistence_prod = PERSISTENCE
        .split("#[cfg(test)]")
        .next()
        .unwrap_or(PERSISTENCE);
    let sources = [
        ("service.rs", SERVICE),
        ("persistence.rs", persistence_prod),
        ("m00_projection.rs", M00_PROJECTION),
        ("m71_projection.rs", M71_PROJECTION),
        ("capability.rs", CAPABILITY),
        ("lib.rs", LIB),
    ];
    for (name, src) in &sources {
        assert!(
            !src.contains(".unwrap()"),
            "{name}: production code must not contain .unwrap()"
        );
        assert!(
            !src.contains(".expect("),
            "{name}: production code must not contain .expect()"
        );
        assert!(
            !src.contains("panic!"),
            "{name}: production code must not contain panic!"
        );
        assert!(
            !src.contains("unreachable!"),
            "{name}: production code must not contain unreachable!"
        );
        assert!(
            !src.contains("todo!"),
            "{name}: production code must not contain todo!"
        );
        assert!(
            !src.contains("unimplemented!"),
            "{name}: production code must not contain unimplemented!"
        );
    }
}

#[test]
fn secret_bearing_carriers_cannot_regain_derived_debug() {
    fn assert_manual_debug(source: &str, type_name: &str) {
        let declaration = [
            "pub struct ",
            "pub enum ",
            "pub(crate) struct ",
            "pub(crate) enum ",
        ]
        .into_iter()
        .map(|prefix| format!("{prefix}{type_name}"))
        .find_map(|needle| source.find(&needle))
        .unwrap_or_else(|| panic!("missing secret-bearing type {type_name}"));
        let window_start = declaration.saturating_sub(256);
        let attribute_window = &source[window_start..declaration];
        assert!(
            !attribute_window.contains("derive(Debug"),
            "{type_name} must not derive plaintext Debug"
        );
        assert!(
            source.contains(&format!("impl std::fmt::Debug for {type_name}")),
            "{type_name} must retain a manual redacted Debug implementation"
        );
    }

    for type_name in [
        "AdmittedActorDto",
        "FrozenPrerequisitesDto",
        "AffairsGetPayloadDto",
        "DispatchCapsuleBodyV2",
    ] {
        assert_manual_debug(CLIENT_CAPSULE, type_name);
    }
    for type_name in [
        "ActorIntentDto",
        "ViewerAuthorizationDto",
        "ClientIntentDto",
        "ClientResponseDto",
    ] {
        assert_manual_debug(CLIENT_TRANSPORT, type_name);
    }
    assert_manual_debug(CAPABILITY, "StoredPublicAuthorization");
    for type_name in [
        "StoredReadPolicy",
        "CompletionReceipt",
        "RecordState",
        "StoredRecord",
    ] {
        assert_manual_debug(PERSISTENCE, type_name);
    }

    let claim = PERSISTENCE
        .find("pub(crate) struct ClaimToken")
        .expect("ClaimToken must exist");
    let claim_window = &PERSISTENCE[claim.saturating_sub(256)..claim];
    assert!(
        !claim_window.contains("derive(Debug"),
        "ClaimToken must remain non-Debug because it carries lease/fence authority"
    );
}
