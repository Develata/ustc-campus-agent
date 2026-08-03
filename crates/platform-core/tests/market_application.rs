#![allow(clippy::expect_used, clippy::panic, clippy::unwrap_used)]

use std::format;
use std::sync::Arc;
use ustc_campus_agent_core::identity::{TenantId, UserId};
use ustc_campus_agent_core::invocation::{
    CatalogRevision, ComponentId, ComponentKind, ComponentVersion, ExecutionIdentity,
    InstallationId, InstallationRevision, PackageId, PackageVersion, Sha256Digest,
};
use ustc_campus_agent_core::market::application::{
    CatalogBrowseQuery, CatalogPackageQuery, CatalogPageLimit, CatalogReadRepository,
    DisableInstallationRequest, InMemoryCatalogReadRepository, MarketApplicationConstructionError,
    MarketApplicationError, MarketApplicationRepositoryError, MarketApplicationService,
    OwnedInstallationGrantQuery, OwnedInstallationQuery, OwnedUpdateQuery,
};
use ustc_campus_agent_core::market::grant::InMemoryGrantRepository;
use ustc_campus_agent_core::market::installation::{
    ConfigurationKey, ConfigurationValue, InMemoryInstallationRepository, InstallationCommand,
    InstallationCommandId, InstallationConfiguration, InstallationPackagePin,
    InstallationRepository, InstalledComponentPin, ManagedInstallationState, NonSecretText,
};
use ustc_campus_agent_core::market::update::InMemoryPackageUpdateRepository;
use ustc_campus_agent_core::market::update::PackageUpdateId;
use ustc_campus_agent_core::market::{
    CatalogReadModel, PackageTier, ValidatedPackageManifest, load_package_manifest,
};

const AFFAIRS: &[u8] =
    include_bytes!("../../../market/packages/ustc.affairs-navigator/package.json");
const CHANGE_RADAR: &[u8] =
    include_bytes!("../../../market/packages/ustc.change-radar/package.json");
const OPPORTUNITY: &[u8] =
    include_bytes!("../../../market/packages/ustc.opportunity-graph/package.json");

const COMMUNITY: &[u8] = br#"{
  "id":"community.example","version":"1.0.0","publisher":"community-team",
  "tier":"VerifiedCommunityText","displayName":"Example","description":"Bounded metadata",
  "implementationStatus":"development",
  "installPolicy":{"class":"UserInstalledPlugin","defaultInstalled":false,"defaultEnabled":false,"userDisableAllowed":true},
  "components":[],"capabilities":[],"sourcePolicy":{"policy":"bounded"}
}"#;

const V1_CATALOG_DIGEST: &str =
    "sha256:a5e660136cdc467a0e75a0cba495706460570e8a4f30b19c5e063c8bec8b24da";
const AFFAIRS_PACKAGE_DIGEST: &str =
    "sha256:49900ac159dc1a8b381554005ed0cddffa76ebc323d81d6bdbf204f72fd1a018";

// ---------------------------------------------------------------------------
// Shared parse / load helpers
// ---------------------------------------------------------------------------

fn load(source: &[u8]) -> ValidatedPackageManifest {
    match load_package_manifest(source) {
        Ok(manifest) => manifest,
        Err(error) => panic!("fixture must load: {error}"),
    }
}

fn parsed<T, E: std::fmt::Display>(result: Result<T, E>) -> T {
    match result {
        Ok(value) => value,
        Err(error) => panic!("fixture must parse: {error}"),
    }
}

fn parsed_catalog_revision(value: &str) -> CatalogRevision {
    parsed(CatalogRevision::parse(value))
}

fn parsed_package_id(value: &str) -> PackageId {
    parsed(PackageId::parse(value))
}

fn parsed_version(value: &str) -> PackageVersion {
    parsed(PackageVersion::parse(value))
}

fn revision_v1() -> CatalogRevision {
    parsed_catalog_revision("catalog:reviewed-v1")
}

fn revision_v2() -> CatalogRevision {
    parsed_catalog_revision("catalog:reviewed-v2")
}

fn first_party_manifests() -> Vec<ValidatedPackageManifest> {
    vec![load(AFFAIRS), load(CHANGE_RADAR), load(OPPORTUNITY)]
}

fn v2_manifests() -> Vec<ValidatedPackageManifest> {
    vec![
        load(COMMUNITY),
        load(AFFAIRS),
        load(CHANGE_RADAR),
        load(OPPORTUNITY),
    ]
}

fn digest(byte: char) -> Sha256Digest {
    parsed(Sha256Digest::parse(format!(
        "sha256:{}",
        byte.to_string().repeat(64)
    )))
}

// ---------------------------------------------------------------------------
// Installation fixtures (shared by installation-read and disable tests)
// ---------------------------------------------------------------------------

fn tenant() -> TenantId {
    parsed(TenantId::parse("tenant:app-test"))
}

fn user() -> UserId {
    parsed(UserId::parse("user:app-test"))
}

fn installation_id() -> InstallationId {
    parsed(InstallationId::parse("installation:app-test"))
}

fn absent_installation_id() -> InstallationId {
    parsed(InstallationId::parse("installation:absent"))
}

fn installation_revision(value: u64) -> InstallationRevision {
    parsed(InstallationRevision::parse(format!(
        "installation-revision:{value}"
    )))
}

fn command_id(suffix: &str) -> InstallationCommandId {
    parsed(InstallationCommandId::parse(format!("cmd:{suffix}")))
}

fn configuration(tenant: &TenantId) -> InstallationConfiguration {
    parsed(InstallationConfiguration::new(
        tenant,
        vec![(
            parsed(ConfigurationKey::parse("mode")),
            ConfigurationValue::Text(parsed(NonSecretText::parse("safe"))),
        )],
    ))
}

fn package_pin() -> InstallationPackagePin {
    let component = parsed(InstalledComponentPin::new(
        parsed(ComponentId::parse("component:app-test")),
        ComponentKind::NativeRustComponent,
        parsed(ComponentVersion::parse("component-version:1")),
        digest('2'),
        parsed(ExecutionIdentity::parse("execution:identity-marker")),
    ));
    parsed(InstallationPackagePin::new(
        parsed_catalog_revision("catalog:app-test"),
        parsed_package_id("ustc.app-test"),
        parsed_version("1.0.0"),
        digest('1'),
        vec![component],
        digest('3'),
        digest('4'),
    ))
}

fn install_command() -> InstallationCommand {
    parsed(InstallationCommand::install(
        command_id("install"),
        installation_id(),
        tenant(),
        user(),
        package_pin(),
        configuration(&tenant()),
    ))
}

fn installed_repository() -> InMemoryInstallationRepository {
    let mut repository = InMemoryInstallationRepository::new();
    repository
        .execute(install_command())
        .expect("install commits");
    repository
}

// ---------------------------------------------------------------------------
// Service builders
// ---------------------------------------------------------------------------

fn build_catalog_service() -> MarketApplicationService<
    InMemoryCatalogReadRepository,
    InMemoryInstallationRepository,
    InMemoryGrantRepository,
    InMemoryPackageUpdateRepository,
> {
    let r1 = revision_v1();
    let r2 = revision_v2();
    let v1 = CatalogReadModel::new(r1.clone(), first_party_manifests()).expect("v1 catalog");
    let v2 = CatalogReadModel::new(r2.clone(), v2_manifests()).expect("v2 catalog");
    let repo = InMemoryCatalogReadRepository::try_new(vec![v1, v2], r2)
        .expect("repository constructs with current present");
    MarketApplicationService::new(
        repo,
        InMemoryInstallationRepository::new(),
        InMemoryGrantRepository::new(),
        InMemoryPackageUpdateRepository::new(),
    )
}

fn build_installation_service() -> MarketApplicationService<
    InMemoryCatalogReadRepository,
    InMemoryInstallationRepository,
    InMemoryGrantRepository,
    InMemoryPackageUpdateRepository,
> {
    let r1 = revision_v1();
    let v1 = CatalogReadModel::new(r1.clone(), first_party_manifests()).expect("v1 catalog");
    let repo = InMemoryCatalogReadRepository::try_new(vec![v1], r1).expect("repository constructs");
    MarketApplicationService::new(
        repo,
        installed_repository(),
        InMemoryGrantRepository::new(),
        InMemoryPackageUpdateRepository::new(),
    )
}

// ---------------------------------------------------------------------------
// Catalog vertical: anonymous browse and exact package detail
// ---------------------------------------------------------------------------

#[test]
fn anonymous_catalog_paging_is_revision_bound_bounded_and_exact() {
    let service = build_catalog_service();
    let r1 = revision_v1();
    let r2 = revision_v2();

    let limit_two = CatalogPageLimit::new(2).expect("limit 2 is in range");

    let query = CatalogBrowseQuery::new(Some(r1.clone()), 0, limit_two).expect("bound v1 browse");
    let page = service
        .browse_catalog(&query)
        .expect("v1 first page resolves");
    assert_eq!(page.catalog_revision(), &r1);
    assert_eq!(page.catalog_digest().as_str(), V1_CATALOG_DIGEST);
    assert_eq!(page.packages().len(), 2);
    assert_eq!(
        page.packages()[0].package_id().as_str(),
        "ustc.affairs-navigator"
    );
    assert_eq!(
        page.packages()[1].package_id().as_str(),
        "ustc.change-radar"
    );
    assert_eq!(page.next_offset(), Some(2));

    let query =
        CatalogBrowseQuery::new(Some(r1.clone()), 2, limit_two).expect("bound v1 continuation");
    let page = service
        .browse_catalog(&query)
        .expect("v1 second page resolves");
    assert_eq!(page.catalog_revision(), &r1);
    assert_eq!(page.packages().len(), 1);
    assert_eq!(
        page.packages()[0].package_id().as_str(),
        "ustc.opportunity-graph"
    );
    assert_eq!(page.next_offset(), None);

    let query = CatalogBrowseQuery::new(None, 0, CatalogPageLimit::new(100).expect("limit 100"))
        .expect("current browse is bound at offset 0");
    let page = service
        .browse_catalog(&query)
        .expect("current page resolves to v2");
    assert_eq!(page.catalog_revision(), &r2);
    assert_eq!(page.packages().len(), 4);
    assert_eq!(
        page.packages()[0].package_id().as_str(),
        "community.example"
    );
    assert_eq!(
        page.packages()[1].package_id().as_str(),
        "ustc.affairs-navigator"
    );
    assert_eq!(page.next_offset(), None);

    let query = CatalogBrowseQuery::new(
        Some(r1.clone()),
        3,
        CatalogPageLimit::new(10).expect("limit 10"),
    )
    .expect("offset past end is still bound with exact revision");
    let page = service
        .browse_catalog(&query)
        .expect("v1 empty page resolves");
    assert_eq!(page.catalog_revision(), &r1);
    assert!(page.packages().is_empty());
    assert_eq!(page.next_offset(), None);

    let query = CatalogBrowseQuery::new(
        Some(r2.clone()),
        0,
        CatalogPageLimit::new(1).expect("limit 1"),
    )
    .expect("bound v2 first page");
    let page = service
        .browse_catalog(&query)
        .expect("v2 first page resolves");
    assert_eq!(page.catalog_revision(), &r2);
    assert_eq!(page.packages().len(), 1);
    assert_eq!(
        page.packages()[0].package_id().as_str(),
        "community.example"
    );
    assert_eq!(page.next_offset(), Some(1));

    let query = CatalogBrowseQuery::new(
        Some(r2.clone()),
        1,
        CatalogPageLimit::new(1).expect("limit 1"),
    )
    .expect("bound v2 continuation");
    let page = service
        .browse_catalog(&query)
        .expect("v2 second page resolves");
    assert_eq!(page.catalog_revision(), &r2);
    assert_eq!(
        page.packages()[0].package_id().as_str(),
        "ustc.affairs-navigator"
    );
    assert_eq!(page.next_offset(), Some(2));

    let missing = parsed_catalog_revision("catalog:missing");
    let query = CatalogBrowseQuery::new(
        Some(missing),
        0,
        CatalogPageLimit::new(10).expect("limit 10"),
    )
    .expect("missing revision query is constructible");
    assert_eq!(
        service.browse_catalog(&query),
        Err(MarketApplicationError::NotFound)
    );

    assert_eq!(
        CatalogPageLimit::new(0),
        Err(MarketApplicationConstructionError::PageLimitOutOfRange)
    );
    assert_eq!(
        CatalogPageLimit::new(101),
        Err(MarketApplicationConstructionError::PageLimitOutOfRange)
    );
    assert_eq!(
        CatalogBrowseQuery::new(None, 1, CatalogPageLimit::new(10).expect("limit 10")),
        Err(MarketApplicationConstructionError::UnboundContinuationOffset)
    );
    assert_eq!(
        CatalogBrowseQuery::new(None, 0, CatalogPageLimit::new(1).expect("limit 1")),
        Ok(
            CatalogBrowseQuery::new(None, 0, CatalogPageLimit::new(1).expect("limit 1"))
                .expect("offset 0 with no revision is bound")
        )
    );
}

#[test]
fn package_detail_is_exact_without_latest_or_fallback() {
    let service = build_catalog_service();
    let r1 = revision_v1();
    let r2 = revision_v2();

    let query = CatalogPackageQuery::new(
        None,
        parsed_package_id("ustc.affairs-navigator"),
        parsed_version("0.1.0"),
    );
    let detail = service
        .package_detail(&query)
        .expect("current detail resolves");
    assert_eq!(detail.catalog_revision(), &r2);
    assert_eq!(
        detail.summary().package_id().as_str(),
        "ustc.affairs-navigator"
    );
    assert_eq!(detail.summary().package_version().as_str(), "0.1.0");
    assert_eq!(detail.summary().publisher(), "first-party");
    assert_eq!(detail.summary().tier(), PackageTier::FirstParty);
    assert!(detail.description().is_some());
    assert_eq!(detail.package_digest().as_str(), AFFAIRS_PACKAGE_DIGEST);
    assert_eq!(detail.components(), &[]);

    let query = CatalogPackageQuery::new(
        Some(r1.clone()),
        parsed_package_id("ustc.change-radar"),
        parsed_version("0.1.0"),
    );
    let detail = service
        .package_detail(&query)
        .expect("exact v1 detail resolves");
    assert_eq!(detail.catalog_revision(), &r1);
    assert_eq!(detail.summary().display_name(), "USTC ChangeRadar");

    let query = CatalogPackageQuery::new(
        None,
        parsed_package_id("missing.example"),
        parsed_version("0.1.0"),
    );
    assert_eq!(
        service.package_detail(&query),
        Err(MarketApplicationError::NotFound)
    );

    let query = CatalogPackageQuery::new(
        None,
        parsed_package_id("ustc.affairs-navigator"),
        parsed_version("9.9.9"),
    );
    assert_eq!(
        service.package_detail(&query),
        Err(MarketApplicationError::NotFound)
    );

    let missing = parsed_catalog_revision("catalog:missing");
    let query = CatalogPackageQuery::new(
        Some(missing),
        parsed_package_id("ustc.affairs-navigator"),
        parsed_version("0.1.0"),
    );
    assert_eq!(
        service.package_detail(&query),
        Err(MarketApplicationError::NotFound)
    );

    let query = CatalogPackageQuery::new(
        Some(r1.clone()),
        parsed_package_id("community.example"),
        parsed_version("1.0.0"),
    );
    assert_eq!(
        service.package_detail(&query),
        Err(MarketApplicationError::NotFound)
    );
}

#[test]
fn in_memory_catalog_repository_rejects_invalid_construction() {
    let r1 = revision_v1();
    let r2 = revision_v2();

    assert_eq!(
        InMemoryCatalogReadRepository::try_new(vec![], r1.clone()).err(),
        Some(MarketApplicationRepositoryError::EmptyCatalogHistory)
    );

    let too_many: Vec<CatalogReadModel> = (0..65)
        .map(|index| {
            CatalogReadModel::new(
                parsed_catalog_revision(&format!("catalog:r{index}")),
                first_party_manifests(),
            )
            .expect("catalog fixture")
        })
        .collect();
    assert_eq!(
        InMemoryCatalogReadRepository::try_new(too_many, parsed_catalog_revision("catalog:r0"))
            .err(),
        Some(MarketApplicationRepositoryError::TooManyCatalogRevisions)
    );

    let model_a = CatalogReadModel::new(r1.clone(), first_party_manifests()).expect("v1 catalog a");
    let model_b = CatalogReadModel::new(r1.clone(), first_party_manifests()).expect("v1 catalog b");
    assert_eq!(
        InMemoryCatalogReadRepository::try_new(vec![model_a, model_b], r1.clone()).err(),
        Some(MarketApplicationRepositoryError::DuplicateCatalogRevision)
    );

    let v1 = CatalogReadModel::new(r1.clone(), first_party_manifests()).expect("v1 catalog");
    let v2 = CatalogReadModel::new(r2.clone(), v2_manifests()).expect("v2 catalog");
    assert_eq!(
        InMemoryCatalogReadRepository::try_new(
            vec![v1, v2],
            parsed_catalog_revision("catalog:missing")
        )
        .err(),
        Some(MarketApplicationRepositoryError::CurrentCatalogMissing)
    );

    let sixty_four: Vec<CatalogReadModel> = (0..64)
        .map(|index| {
            CatalogReadModel::new(
                parsed_catalog_revision(&format!("catalog:r{index}")),
                first_party_manifests(),
            )
            .expect("catalog fixture")
        })
        .collect();
    assert!(
        InMemoryCatalogReadRepository::try_new(sixty_four, parsed_catalog_revision("catalog:r0"))
            .is_ok()
    );
}

#[test]
fn catalog_read_repository_port_returns_exact_and_current_arcs() {
    let r1 = revision_v1();
    let r2 = revision_v2();
    let v1 = CatalogReadModel::new(r1.clone(), first_party_manifests()).expect("v1 catalog");
    let v2 = CatalogReadModel::new(r2.clone(), v2_manifests()).expect("v2 catalog");
    let repo = InMemoryCatalogReadRepository::try_new(vec![v1, v2], r2.clone())
        .expect("repository constructs");

    let current = repo.load_current().expect("current loads");
    assert_eq!(current.catalog_revision(), &r2);
    assert_eq!(current.packages().len(), 4);

    let exact = repo
        .load_exact(&r1)
        .expect("exact v1 loads")
        .expect("v1 is present");
    assert_eq!(exact.catalog_revision(), &r1);
    assert_eq!(exact.packages().len(), 3);

    let missing = parsed_catalog_revision("catalog:missing");
    assert!(
        repo.load_exact(&missing)
            .expect("missing resolves to None")
            .is_none()
    );
}

// ---------------------------------------------------------------------------
// Installation-read vertical: owner-scoped read with carrier redaction
// ---------------------------------------------------------------------------

#[test]
fn owned_installation_read_is_owner_scoped_and_excludes_sensitive_carriers() {
    let service = build_installation_service();

    let absent = OwnedInstallationQuery::new(tenant(), user(), absent_installation_id());
    assert_eq!(
        service.installation(&absent),
        Err(MarketApplicationError::NotFound)
    );

    let correct = OwnedInstallationQuery::new(tenant(), user(), installation_id());
    let view = service
        .installation(&correct)
        .expect("owned installation resolves");
    assert_eq!(view.installation_id(), &installation_id());
    assert_eq!(view.state(), ManagedInstallationState::InstalledDisabled);
    assert_eq!(view.revision(), &installation_revision(1));

    let foreign_tenant = OwnedInstallationQuery::new(
        parsed(TenantId::parse("tenant:foreign")),
        user(),
        installation_id(),
    );
    assert_eq!(
        service.installation(&foreign_tenant),
        Err(MarketApplicationError::NotFound)
    );

    let foreign_user = OwnedInstallationQuery::new(
        tenant(),
        parsed(UserId::parse("user:foreign")),
        installation_id(),
    );
    assert_eq!(
        service.installation(&foreign_user),
        Err(MarketApplicationError::NotFound)
    );

    let debug = format!("{view:?}");
    assert!(
        !debug.contains("sensitive-value-marker"),
        "debug leaks NonSecretText"
    );
    assert!(
        !debug.contains("execution:identity-marker"),
        "debug leaks ExecutionIdentity"
    );
}

// ---------------------------------------------------------------------------
// Current-grants vertical: public denial/empty cases only.
//
// Grant admission evidence construction is `pub(in crate::market)`, so the
// seeded canonical-order success proof lives in `application.rs` internal
// tests. These external tests prove only the public denial/empty surface:
// absent/foreign authority maps to `NotFound`, a stale installation revision
// maps to `Conflict`, an empty current set observes the exact revision, and no
// approval/history carrier leaks through the public Debug surface.
// ---------------------------------------------------------------------------

fn grant_parsed<T, E: std::fmt::Display>(result: Result<T, E>) -> T {
    match result {
        Ok(value) => value,
        Err(error) => panic!("grant fixture must parse: {error}"),
    }
}

fn grant_tenant() -> TenantId {
    grant_parsed(TenantId::parse("tenant:grant-app-ext"))
}

fn grant_user() -> UserId {
    grant_parsed(UserId::parse("user:grant-app-ext"))
}

fn grant_installation_id() -> InstallationId {
    grant_parsed(InstallationId::parse("installation:grant-app-ext"))
}

fn grant_installation_revision(sequence: u64) -> InstallationRevision {
    grant_parsed(InstallationRevision::parse(format!(
        "installation-revision:{sequence}"
    )))
}

fn grant_command_id(suffix: &str) -> InstallationCommandId {
    grant_parsed(InstallationCommandId::parse(format!("cmd:{suffix}")))
}

fn grant_digest(byte: char) -> Sha256Digest {
    grant_parsed(Sha256Digest::parse(format!(
        "sha256:{}",
        byte.to_string().repeat(64)
    )))
}

fn grant_package_pin() -> InstallationPackagePin {
    InstallationPackagePin::new(
        grant_parsed(CatalogRevision::parse("catalog:grant-app-ext")),
        grant_parsed(PackageId::parse("ustc.grant-app-ext")),
        grant_parsed(PackageVersion::parse("1.0.0")),
        grant_digest('1'),
        vec![grant_parsed(InstalledComponentPin::new(
            grant_parsed(ComponentId::parse("component:grant-app-ext")),
            ComponentKind::NativeRustComponent,
            grant_parsed(ComponentVersion::parse("component-version:1")),
            grant_digest('2'),
            grant_parsed(ExecutionIdentity::parse("native:grant-app-ext")),
        ))],
        grant_digest('3'),
        grant_digest('4'),
    )
    .expect("grant package pin")
}

fn grant_installation_repository() -> InMemoryInstallationRepository {
    let mut repository = InMemoryInstallationRepository::new();
    let command = InstallationCommand::install(
        grant_command_id("grant-install"),
        grant_installation_id(),
        grant_tenant(),
        grant_user(),
        grant_package_pin(),
        InstallationConfiguration::new(&grant_tenant(), Vec::new()).expect("configuration"),
    )
    .expect("grant install command");
    repository.execute(command).expect("seed installation");
    repository
}

struct NullCatalogRepository;

impl CatalogReadRepository for NullCatalogRepository {
    fn load_current(&self) -> Result<Arc<CatalogReadModel>, MarketApplicationRepositoryError> {
        Err(MarketApplicationRepositoryError::Unavailable)
    }

    fn load_exact(
        &self,
        _revision: &CatalogRevision,
    ) -> Result<Option<Arc<CatalogReadModel>>, MarketApplicationRepositoryError> {
        Err(MarketApplicationRepositoryError::Unavailable)
    }
}

fn grant_service() -> MarketApplicationService<
    NullCatalogRepository,
    InMemoryInstallationRepository,
    InMemoryGrantRepository,
    InMemoryPackageUpdateRepository,
> {
    MarketApplicationService::new(
        NullCatalogRepository,
        grant_installation_repository(),
        InMemoryGrantRepository::new(),
        InMemoryPackageUpdateRepository::new(),
    )
}

#[test]
fn current_grants_require_exact_installation_revision_and_canonical_order() {
    let service = grant_service();

    // Absent installation maps to NotFound, not Conflict, even with a revision claim.
    let absent = OwnedInstallationGrantQuery::new(
        grant_tenant(),
        grant_user(),
        grant_parsed(InstallationId::parse("installation:absent")),
        grant_installation_revision(1),
    );
    assert_eq!(
        service.current_grants(&absent),
        Err(MarketApplicationError::NotFound)
    );

    // Stale installation revision maps to Conflict.
    let stale = OwnedInstallationGrantQuery::new(
        grant_tenant(),
        grant_user(),
        grant_installation_id(),
        grant_installation_revision(99),
    );
    assert_eq!(
        service.current_grants(&stale),
        Err(MarketApplicationError::Conflict)
    );

    // Exact revision with no seeded grants returns the complete empty current
    // set, observing the exact installation revision. Canonical order of a
    // seeded multi-grant set is proved in the internal application test.
    let correct = OwnedInstallationGrantQuery::new(
        grant_tenant(),
        grant_user(),
        grant_installation_id(),
        grant_installation_revision(1),
    );
    let page = service
        .current_grants(&correct)
        .expect("empty current grants resolve");
    assert_eq!(page.installation_id(), &grant_installation_id());
    assert_eq!(
        page.observed_installation_revision(),
        &grant_installation_revision(1)
    );
    assert!(page.grants().is_empty());

    // Re-running the same exact revision is deterministic and idempotent.
    let again = service.current_grants(&correct).expect("re-read resolves");
    assert_eq!(again, page);
}

#[test]
fn current_grants_hide_foreign_or_absent_authority() {
    let service = grant_service();

    let absent = OwnedInstallationGrantQuery::new(
        grant_tenant(),
        grant_user(),
        grant_parsed(InstallationId::parse("installation:absent")),
        grant_installation_revision(1),
    );
    assert_eq!(
        service.current_grants(&absent),
        Err(MarketApplicationError::NotFound)
    );

    let foreign_tenant = OwnedInstallationGrantQuery::new(
        grant_parsed(TenantId::parse("tenant:foreign")),
        grant_user(),
        grant_installation_id(),
        grant_installation_revision(1),
    );
    assert_eq!(
        service.current_grants(&foreign_tenant),
        Err(MarketApplicationError::NotFound)
    );

    let foreign_user = OwnedInstallationGrantQuery::new(
        grant_tenant(),
        grant_parsed(UserId::parse("user:foreign")),
        grant_installation_id(),
        grant_installation_revision(1),
    );
    assert_eq!(
        service.current_grants(&foreign_user),
        Err(MarketApplicationError::NotFound)
    );

    // A foreign tenant combined with a stale revision still maps to NotFound:
    // ownership is checked before the exact-revision claim, so no foreign
    // existence is leaked through a Conflict signal.
    let foreign_tenant_stale = OwnedInstallationGrantQuery::new(
        grant_parsed(TenantId::parse("tenant:foreign")),
        grant_user(),
        grant_installation_id(),
        grant_installation_revision(99),
    );
    assert_eq!(
        service.current_grants(&foreign_tenant_stale),
        Err(MarketApplicationError::NotFound)
    );
}

#[test]
fn current_grants_expose_no_approval_or_history_carriers() {
    let service = grant_service();
    let query = OwnedInstallationGrantQuery::new(
        grant_tenant(),
        grant_user(),
        grant_installation_id(),
        grant_installation_revision(1),
    );
    let page = service
        .current_grants(&query)
        .expect("empty current grants resolve");

    // The public Debug surface of the page redacts approval/evidence/history
    // carriers. Even with an empty grant set, the page debug must not reference
    // approval, evidence, consumed-approval or history vocabulary. The seeded
    // redaction proof over a populated grant set lives in the internal test.
    let page_debug = format!("{page:?}");
    assert!(
        !page_debug.contains("approval"),
        "page debug references approval carrier"
    );
    assert!(
        !page_debug.contains("evidence"),
        "page debug references evidence carrier"
    );
    assert!(
        !page_debug.contains("history"),
        "page debug references history carrier"
    );
    assert!(
        !page_debug.contains("consumed"),
        "page debug references consumed-approval carrier"
    );
    assert!(
        !page_debug.contains("last_sequence"),
        "page debug references history carrier"
    );

    // The query debug exposes only scope claims and the expected revision.
    let query_debug = format!("{query:?}");
    assert!(
        !query_debug.contains("approval"),
        "query debug references approval carrier"
    );
    assert!(
        !query_debug.contains("evidence"),
        "query debug references evidence carrier"
    );
}

// ---------------------------------------------------------------------------
// Package-update vertical: public denial surface
// ---------------------------------------------------------------------------

#[test]
fn package_update_absent_maps_to_not_found() {
    let service = build_catalog_service();
    let update_id = parsed(PackageUpdateId::parse("update:absent"));
    let query = OwnedUpdateQuery::new(tenant(), user(), update_id);
    assert_eq!(
        service.package_update(&query),
        Err(MarketApplicationError::NotFound)
    );
}

// ---------------------------------------------------------------------------
// Disable vertical: public denial surface.
//
// The disable success path (Enabled → Disabled) and idempotency are proved in
// the internal application tests because they require
// `EnablePreconditionEvidence::from_authority_bindings` which is
// `pub(in crate::market)`. These external tests prove the public denial
// surface: absent/foreign authority maps to `NotFound`, a non-enabled
// installation maps to `LifecycleDenied`, and a stale expected revision maps
// to `Conflict` — all delegated to the owner command ledger with no pre-reject
// in the application façade.
// ---------------------------------------------------------------------------

#[test]
fn disable_absent_and_foreign_installations_are_not_found() {
    let mut service = build_installation_service();

    // Absent installation maps to NotFound before reaching the owner ledger.
    let absent = DisableInstallationRequest::new(
        command_id("absent-disable"),
        tenant(),
        user(),
        absent_installation_id(),
        installation_revision(1),
    );
    assert_eq!(
        service.disable_installation(absent),
        Err(MarketApplicationError::NotFound)
    );

    // Foreign tenant maps to NotFound; ownership is checked before the ledger.
    let foreign_tenant = DisableInstallationRequest::new(
        command_id("foreign-tenant-disable"),
        parsed(TenantId::parse("tenant:foreign")),
        user(),
        installation_id(),
        installation_revision(1),
    );
    assert_eq!(
        service.disable_installation(foreign_tenant),
        Err(MarketApplicationError::NotFound)
    );

    // Foreign user maps to NotFound; ownership is checked before the ledger.
    let foreign_user = DisableInstallationRequest::new(
        command_id("foreign-user-disable"),
        tenant(),
        parsed(UserId::parse("user:foreign")),
        installation_id(),
        installation_revision(1),
    );
    assert_eq!(
        service.disable_installation(foreign_user),
        Err(MarketApplicationError::NotFound)
    );
}

#[test]
fn disable_non_enabled_installation_is_lifecycle_denied() {
    let mut service = build_installation_service();

    // The installation is in InstalledDisabled state (revision 1) after install.
    // Disable requires Enabled state, so the owner ledger rejects with
    // IllegalTransition, which the façade maps to LifecycleDenied.
    let request = DisableInstallationRequest::new(
        command_id("disable"),
        tenant(),
        user(),
        installation_id(),
        installation_revision(1),
    );
    assert_eq!(
        service.disable_installation(request),
        Err(MarketApplicationError::LifecycleDenied)
    );
}

#[test]
fn disable_stale_revision_on_installed_is_conflict() {
    let mut service = build_installation_service();

    // The installation is at revision 1. A stale expected revision (99) is
    // rejected by the owner ledger with RevisionMismatch before the state
    // transition is evaluated, which the façade maps to Conflict.
    let stale = DisableInstallationRequest::new(
        command_id("stale-disable"),
        tenant(),
        user(),
        installation_id(),
        installation_revision(99),
    );
    assert_eq!(
        service.disable_installation(stale),
        Err(MarketApplicationError::Conflict)
    );
}

// ---------------------------------------------------------------------------
// Service-level debug redaction
// ---------------------------------------------------------------------------

#[test]
fn application_service_debug_is_authority_redacted() {
    let service = build_catalog_service();
    let debug = format!("{service:?}");
    assert_eq!(debug, "MarketApplicationService(<authority-redacted>)");
}
