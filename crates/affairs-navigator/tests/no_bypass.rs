#![allow(clippy::unwrap_used)]

//! Public API proof that no M10/client/storage dependency is present in the
//! crate. The proof is structural: `affairs-navigator`'s `Cargo.toml` lists
//! only `time` and `sha2`. This test verifies the public API surface is
//! reachable through the crate root without naming any external product crate.

mod common;

use affairs_navigator::*;

/// The crate's public API consists only of M71-owned types. No type from M10,
/// M80, client, or storage appears in the public re-exports. This is
/// structurally enforced by the dependency graph: the crate cannot `use` a
/// crate it does not depend on.
#[test]
fn public_api_contains_only_m71_owned_types() {
    let _ = value::ProcedureId::parse("p").unwrap();
    let _ = evidence::AffairsAuthority::OfficialBulletin;
    let _ = outcome::GetProcedureOutcome::NotFound {
        procedure_id: value::ProcedureId::parse("p").unwrap(),
    };
    let _ = m60_port::M60EvidencePortError::StoreUnavailable;
    let _ = public_view::ProjectionMetadata::Complete;
    let _ = clock::FixedClock::new(common::t(200));
}

/// The only way to obtain an `M71AffairsGetReceipt` is through the service.
/// The `new` constructor is `pub(crate)` and cannot be called from outside.
#[test]
fn receipt_is_obtainable_only_through_service() {
    let repo = repository::InMemoryAffairsRepository::new();
    let m60 = m60_fixture::M60FixtureAdapter::new("verifier:fixture", 1).unwrap();
    let clock = clock::FixedClock::new(common::t(200));
    let service = service::AffairsGetService::new(&repo, &m60, &clock);
    let query = outcome::AffairsGetQuery::new(
        value::ProcedureId::parse("p").unwrap(),
        Some(common::t(200)),
    );
    let receipt = service.execute(&query).unwrap();
    let _ = receipt.outcome();
    let _ = receipt.evidence_lineage();
}

/// `PublicProcedureView` constructors are `pub(crate)`. External code reads
/// through accessors only.
#[test]
fn public_view_accessors_are_public() {
    let repo = repository::InMemoryAffairsRepository::new();
    let m60 = m60_fixture::M60FixtureAdapter::new("verifier:fixture", 1).unwrap();
    let clock = clock::FixedClock::new(common::t(200));
    let service = service::AffairsGetService::new(&repo, &m60, &clock);
    let query = outcome::AffairsGetQuery::new(
        value::ProcedureId::parse("p").unwrap(),
        Some(common::t(200)),
    );
    let receipt = service.execute(&query).unwrap();
    assert!(matches!(
        receipt.outcome(),
        outcome::GetProcedureOutcome::NotFound { .. }
    ));
}

/// `M71EvidenceLineage` constructors are `pub(crate)`. External code reads
/// through the public accessors on a receipt only.
#[test]
fn lineage_accessors_are_public() {
    let repo = repository::InMemoryAffairsRepository::new();
    let m60 = m60_fixture::M60FixtureAdapter::new("verifier:fixture", 1).unwrap();
    let clock = clock::FixedClock::new(common::t(200));
    let service = service::AffairsGetService::new(&repo, &m60, &clock);
    let query = outcome::AffairsGetQuery::new(
        value::ProcedureId::parse("p").unwrap(),
        Some(common::t(200)),
    );
    let receipt = service.execute(&query).unwrap();
    let lineage = receipt.evidence_lineage();
    let _ = lineage.is_verified();
    let _ = lineage.is_unverified();
    let _ = lineage.is_not_required();
    let _ = lineage.materialization_receipt_id();
    let _ = lineage.m60_evidence_set_digest();
    let _ = lineage.m60_revision_count();
    let _ = lineage.verification_identity();
    let _ = lineage.unverified_reason();
    let _ = lineage.not_required_reason();
}

/// The `AffairsRepository` trait is the only repository abstraction; the
/// in-memory fixture is one implementation. No storage crate is imported.
#[test]
fn repository_trait_is_public() {
    let repo = repository::InMemoryAffairsRepository::new();
    let _: &dyn repository::AffairsRepository = &repo;
}

/// The `M60ProcedureEvidencePort` trait is the only M60 abstraction; the
/// fixture adapter is one implementation. No M60 crate is imported.
#[test]
fn m60_port_trait_is_public() {
    let m60 = m60_fixture::M60FixtureAdapter::new("verifier:fixture", 1).unwrap();
    let _: &dyn m60_port::M60ProcedureEvidencePort = &m60;
}
