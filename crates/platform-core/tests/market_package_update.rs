#![allow(clippy::expect_used, clippy::panic, clippy::unwrap_used)]

use std::collections::BTreeSet;
use ustc_campus_agent_core::identity::{TenantId, UserId};
use ustc_campus_agent_core::invocation::{
    CapabilityClass, CapabilityId, CatalogComponentRevision, CatalogPackageRevision,
    CatalogRevision, ComponentId, ComponentKind, ComponentVersion, ExecutionIdentity,
    InstallationId, InstallationRevision, InvocationPolicySnapshot, PolicyRevision,
    PolicySnapshotId, Sha256Digest, SourcePolicyId, SourcePolicyIdentity,
};
use ustc_campus_agent_core::market::capability::{CapabilityRegistry, load_capability_registry};
use ustc_campus_agent_core::market::installation::{
    ConfigurationKey, ConfigurationValue, InMemoryInstallationRepository, InstallationCommand,
    InstallationCommandId, InstallationConfiguration, InstallationPackagePin,
    InstallationRepository, InstalledComponentPin, NonSecretText,
};
use ustc_campus_agent_core::market::update::{
    InMemoryPackageUpdateRepository, PackageUpdateId, PackageUpdateRepository, UpdateChangeClass,
    UpdateCommand, UpdateCommandId, UpdateCommandOutcome, UpdateConstructionError,
    UpdateDecisionError, UpdateEventKind, UpdateEventSequence, UpdateRepositoryError,
    UpdateRevision, replay,
};
use ustc_campus_agent_core::market::{
    CatalogReadModel, ValidatedPackageManifest, load_package_manifest,
};

macro_rules! parsed {
    ($kind:ty, $value:expr) => {{
        match <$kind>::parse($value) {
            Ok(value) => value,
            Err(error) => panic!("fixture value must parse: {error}"),
        }
    }};
}

fn digest(byte: char) -> Sha256Digest {
    parsed!(
        Sha256Digest,
        format!("sha256:{}", byte.to_string().repeat(64))
    )
}

fn tenant(value: &str) -> TenantId {
    parsed!(TenantId, value)
}

fn user(value: &str) -> UserId {
    parsed!(UserId, value)
}

fn installation(value: &str) -> InstallationId {
    parsed!(InstallationId, value)
}

fn catalog_revision(value: &str) -> CatalogRevision {
    parsed!(CatalogRevision, value)
}

fn component_id(value: &str) -> ComponentId {
    parsed!(ComponentId, value)
}

fn component_version(value: &str) -> ComponentVersion {
    parsed!(ComponentVersion, value)
}

fn execution_identity(value: &str) -> ExecutionIdentity {
    parsed!(ExecutionIdentity, value)
}

fn capability_id(value: &str) -> CapabilityId {
    parsed!(CapabilityId, value)
}

fn source_policy_id(value: &str) -> SourcePolicyId {
    parsed!(SourcePolicyId, value)
}

fn update_id(value: &str) -> PackageUpdateId {
    PackageUpdateId::parse(value).expect("valid update id")
}

fn update_command_id(value: &str) -> UpdateCommandId {
    UpdateCommandId::parse(value).expect("valid update command id")
}

fn update_revision(value: &str) -> UpdateRevision {
    UpdateRevision::parse(value).expect("valid update revision")
}

fn installation_command_id(value: &str) -> InstallationCommandId {
    parsed!(InstallationCommandId, value)
}

fn policy_snapshot_id(value: &str) -> PolicySnapshotId {
    parsed!(PolicySnapshotId, value)
}

fn policy_revision(value: &str) -> PolicyRevision {
    parsed!(PolicyRevision, value)
}

fn manifest_source(version: &str, publisher: &str, display: &str) -> String {
    format!(
        r#"{{
          "id":"public.update-package",
          "version":"{version}",
          "publisher":"{publisher}",
          "tier":"VerifiedCommunityText",
          "displayName":"{display}",
          "implementationStatus":"implemented",
          "installPolicy":{{"class":"UserInstalledPlugin","defaultInstalled":false,"defaultEnabled":false,"userDisableAllowed":true}},
          "components":[{{"type":"McpServerComponent","path":"components/update.json","mode":"remote"}}],
          "capabilities":["campus.public_rules.read"],
          "sourcePolicy":{{"officialSources":"reviewed-public-only"}}
        }}"#
    )
}

fn manifest(version: &str, publisher: &str, display: &str) -> ValidatedPackageManifest {
    load_package_manifest(manifest_source(version, publisher, display).as_bytes())
        .expect("manifest fixture must validate")
}

fn registry_source(revision: &str) -> String {
    format!(
        r#"{{"schemaVersion":"capability-registry/v1","registryRevision":"{revision}","capabilities":[{{"id":"campus.public_rules.read","effectClass":"Read","dataClass":"PublicCampusFact","scopeKind":"CampusPublic","autoGrant":"FirstPartyDefaultOnly","confirmationDefault":"Allow","status":"Active"}}]}}"#
    )
}

fn registry(revision: &str) -> CapabilityRegistry {
    load_capability_registry(registry_source(revision).as_bytes()).expect("registry fixture")
}

#[derive(Clone)]
struct PublicUpdateFixture {
    tenant_id: TenantId,
    user_id: UserId,
    installation_id: InstallationId,
    rollback_manifest: ValidatedPackageManifest,
    target_manifest: ValidatedPackageManifest,
    rollback_catalog: CatalogReadModel,
    target_catalog: CatalogReadModel,
    rollback_publications: Vec<CatalogPackageRevision>,
    target_publications: Vec<CatalogPackageRevision>,
    rollback_registry: CapabilityRegistry,
    target_registry: CapabilityRegistry,
    rollback_pin: InstallationPackagePin,
    target_pin: InstallationPackagePin,
    configuration: InstallationConfiguration,
}

impl PublicUpdateFixture {
    fn new_with_sentinels(prefix: &str) -> Self {
        let tenant_id = tenant(&format!("tenant:{prefix}-tenant"));
        let user_id = user(&format!("user:{prefix}-user"));
        let installation_id = installation(&format!("installation:{prefix}-installation"));
        let rollback_manifest =
            manifest("1.0.0", &format!("{prefix}-publisher"), "Public Rollback");
        let target_manifest = manifest("1.1.0", &format!("{prefix}-publisher"), "Public Target");
        let rollback_catalog = CatalogReadModel::new(
            catalog_revision(&format!("catalog:{prefix}-rollback")),
            vec![rollback_manifest.clone()],
        )
        .expect("rollback catalog");
        let target_catalog = CatalogReadModel::new(
            catalog_revision(&format!("catalog:{prefix}-target")),
            vec![target_manifest.clone()],
        )
        .expect("target catalog");
        let rollback_component = catalog_component('2');
        let target_component = catalog_component('3');
        let rollback_publications = vec![publication(
            &rollback_catalog,
            &rollback_manifest,
            rollback_component.clone(),
        )];
        let target_publications = vec![publication(
            &target_catalog,
            &target_manifest,
            target_component.clone(),
        )];
        let rollback_registry = registry(&format!("capability-registry:{prefix}-rollback"));
        let target_registry = registry(&format!("capability-registry:{prefix}-target"));
        let rollback_pin = pin(&rollback_catalog, &rollback_manifest, &rollback_component);
        let target_pin = pin(&target_catalog, &target_manifest, &target_component);
        let configuration = InstallationConfiguration::new(
            &tenant_id,
            vec![(
                parsed!(ConfigurationKey, "mode"),
                ConfigurationValue::Text(parsed!(NonSecretText, format!("{prefix}-config"))),
            )],
        )
        .expect("installation configuration");
        Self {
            tenant_id,
            user_id,
            installation_id,
            rollback_manifest,
            target_manifest,
            rollback_catalog,
            target_catalog,
            rollback_publications,
            target_publications,
            rollback_registry,
            target_registry,
            rollback_pin,
            target_pin,
            configuration,
        }
    }

    fn install_command(&self, suffix: &str) -> InstallationCommand {
        InstallationCommand::install(
            installation_command_id(&format!("cmd:{suffix}")),
            self.installation_id.clone(),
            self.tenant_id.clone(),
            self.user_id.clone(),
            self.rollback_pin.clone(),
            self.configuration.clone(),
        )
        .expect("install command")
    }

    fn installed_repository(
        &self,
    ) -> (
        InMemoryInstallationRepository,
        ustc_campus_agent_core::market::installation::InstallationCommandReceipt,
    ) {
        let mut repository = InMemoryInstallationRepository::new();
        let receipt = repository
            .execute(self.install_command("install-for-update"))
            .expect("install persisted");
        (repository, receipt)
    }

    fn installed_snapshot(
        &self,
    ) -> ustc_campus_agent_core::market::installation::InstallationSnapshot {
        let (repository, _) = self.installed_repository();
        repository
            .load_exact(&self.installation_id)
            .expect("installation query")
            .expect("installed snapshot")
    }

    fn policy_snapshots(&self) -> Vec<InvocationPolicySnapshot> {
        let source_policy = self
            .target_publications
            .first()
            .and_then(|publication| publication.source_policy.clone())
            .expect("target source policy");
        vec![InvocationPolicySnapshot {
            snapshot_id: policy_snapshot_id("policy:public-update-stage"),
            revision: policy_revision("policy-revision:1"),
            capability_id: capability_id("campus.public_rules.read"),
            capability_class: Some(CapabilityClass::PublicRead),
            admitted_execution_identity: Some(execution_identity("exec:public-update")),
            admitted_source_policy: Some(source_policy),
            emergency_blocked: false,
        }]
    }
}

fn catalog_component(byte: char) -> CatalogComponentRevision {
    let mut capabilities = BTreeSet::new();
    capabilities.insert(capability_id("campus.public_rules.read"));
    CatalogComponentRevision {
        id: component_id("component:public-update"),
        kind: ComponentKind::McpServerComponent,
        version: component_version(&format!("component-version:{byte}")),
        digest: digest(byte),
        execution_identity: execution_identity("exec:public-update"),
        declared_capabilities: capabilities,
        tool: None,
    }
}

fn publication(
    catalog: &CatalogReadModel,
    manifest: &ValidatedPackageManifest,
    component: CatalogComponentRevision,
) -> CatalogPackageRevision {
    CatalogPackageRevision {
        catalog_revision: catalog.catalog_revision().clone(),
        package_id: manifest.package_id().clone(),
        package_version: manifest.package_version().clone(),
        package_digest: manifest.package_digest().clone(),
        runnable: true,
        revoked: false,
        capability_manifest_digest: manifest.capability_manifest_digest().clone(),
        source_policy: Some(SourcePolicyIdentity {
            id: source_policy_id("source-policy:public-update"),
            digest: manifest.source_policy_digest().clone(),
        }),
        component: Some(component),
    }
}

fn pin(
    catalog: &CatalogReadModel,
    manifest: &ValidatedPackageManifest,
    component: &CatalogComponentRevision,
) -> InstallationPackagePin {
    InstallationPackagePin::new(
        catalog.catalog_revision().clone(),
        manifest.package_id().clone(),
        manifest.package_version().clone(),
        manifest.package_digest().clone(),
        vec![
            InstalledComponentPin::new(
                component.id.clone(),
                component.kind,
                component.version.clone(),
                component.digest.clone(),
                component.execution_identity.clone(),
            )
            .expect("component pin"),
        ],
        manifest.component_declaration_set_digest().clone(),
        manifest.capability_manifest_digest().clone(),
    )
    .expect("package pin")
}

fn update_repository(fixture: &PublicUpdateFixture) -> InMemoryPackageUpdateRepository {
    let (installation_repository, installation_receipt) = fixture.installed_repository();
    let history = installation_repository
        .event_history(&fixture.installation_id)
        .expect("installation history");
    InMemoryPackageUpdateRepository::try_from_authority_histories(
        vec![
            fixture.rollback_catalog.clone(),
            fixture.target_catalog.clone(),
        ],
        [
            fixture.rollback_publications.clone(),
            fixture.target_publications.clone(),
        ]
        .concat(),
        vec![
            fixture.rollback_registry.clone(),
            fixture.target_registry.clone(),
        ],
        fixture.policy_snapshots(),
        vec![(fixture.installation_id.clone(), history)],
        vec![(installation_receipt, None)],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
    )
    .expect("update repository from public histories")
}

fn stage_command(fixture: &PublicUpdateFixture, command: &str, update: &str) -> UpdateCommand {
    UpdateCommand::stage(
        update_command_id(command),
        update_id(update),
        &fixture.installed_snapshot(),
        fixture.target_pin.clone(),
        &fixture.rollback_catalog,
        &fixture.rollback_publications,
        &fixture.target_catalog,
        &fixture.target_publications,
        &fixture.rollback_registry,
        &fixture.target_registry,
    )
    .expect("stage command")
}

#[test]
fn checked_public_update_values_and_stage_surface_are_deterministic() {
    assert_eq!(
        PackageUpdateId::parse("update:deterministic")
            .unwrap()
            .as_str(),
        "update:deterministic"
    );
    assert_eq!(
        PackageUpdateId::parse("package-update:bad").unwrap_err(),
        UpdateConstructionError::InvalidUpdateId
    );
    assert_eq!(
        PackageUpdateId::parse("update:").unwrap_err(),
        UpdateConstructionError::InvalidUpdateId
    );
    assert_eq!(
        UpdateCommandId::parse("update-cmd:deterministic")
            .unwrap()
            .as_str(),
        "update-cmd:deterministic"
    );
    assert_eq!(
        UpdateCommandId::parse("cmd:bad").unwrap_err(),
        UpdateConstructionError::InvalidCommandId
    );
    assert_eq!(UpdateEventSequence::new(1).unwrap().get(), 1);
    assert_eq!(
        UpdateEventSequence::new(0).unwrap_err(),
        UpdateConstructionError::InvalidEventSequence
    );
    assert_eq!(
        UpdateRevision::parse("update-revision:7").unwrap().as_str(),
        "update-revision:7"
    );
    assert_eq!(
        UpdateRevision::parse("update-revision:0").unwrap_err(),
        UpdateConstructionError::InvalidUpdateRevision
    );
    assert_eq!(
        UpdateRevision::parse("update-revision:01").unwrap_err(),
        UpdateConstructionError::InvalidUpdateRevision
    );

    let fixture = PublicUpdateFixture::new_with_sentinels("deterministic-public");
    let first = stage_command(
        &fixture,
        "update-cmd:deterministic-stage",
        "update:deterministic-stage",
    );
    let second = stage_command(
        &fixture,
        "update-cmd:deterministic-stage",
        "update:deterministic-stage",
    );
    assert_eq!(first, second);
    assert_eq!(
        first.command_id().as_str(),
        "update-cmd:deterministic-stage"
    );
    assert_eq!(first.update_id().as_str(), "update:deterministic-stage");

    let mut repository = update_repository(&fixture);
    let receipt = repository.execute(first).expect("stage receipt");
    let (event, snapshot) = match receipt.outcome() {
        UpdateCommandOutcome::Accepted { event, snapshot } => (event, snapshot),
        UpdateCommandOutcome::Rejected { error } => panic!("stage rejected: {error:?}"),
    };
    assert_eq!(event.kind(), UpdateEventKind::Staged);
    assert_eq!(
        event.command_id().as_str(),
        "update-cmd:deterministic-stage"
    );
    assert_eq!(event.update_id().as_str(), "update:deterministic-stage");
    assert_eq!(snapshot.update_id().as_str(), "update:deterministic-stage");
    assert_eq!(snapshot.installation_id(), &fixture.installation_id);
    assert_eq!(snapshot.tenant_id(), &fixture.tenant_id);
    assert_eq!(snapshot.user_id(), &fixture.user_id);
    assert_eq!(snapshot.revision().as_str(), "update-revision:1");
    assert_eq!(snapshot.last_sequence().get(), 1);
    assert_eq!(snapshot.plan().change_class(), UpdateChangeClass::Unchanged);
    assert_eq!(snapshot.plan().update_id(), snapshot.update_id());
    assert_eq!(snapshot.plan().tenant_id(), &fixture.tenant_id);
    assert_eq!(snapshot.plan().user_id(), &fixture.user_id);
    assert_eq!(snapshot.plan().installation_id(), &fixture.installation_id);
    assert_eq!(
        snapshot.plan().staged_installation_revision(),
        &parsed!(InstallationRevision, "installation-revision:1")
    );
    assert_eq!(snapshot.plan().staged_configuration_revision().get(), 1);
    assert_eq!(
        snapshot.plan().staged_configuration_digest(),
        fixture.configuration.digest()
    );
    assert_eq!(snapshot.plan().rollback_pin(), &fixture.rollback_pin);
    assert_eq!(snapshot.plan().target_pin(), &fixture.target_pin);
    assert_eq!(
        snapshot.plan().rollback_package(),
        &fixture.rollback_manifest
    );
    assert_eq!(snapshot.plan().target_package(), &fixture.target_manifest);
    assert_eq!(snapshot.plan().rollback_components().len(), 1);
    assert_eq!(snapshot.plan().target_components().len(), 1);
    assert_eq!(snapshot.plan().rollback_component_declarations().len(), 1);
    assert_eq!(snapshot.plan().target_component_declarations().len(), 1);
    assert_eq!(snapshot.plan().rollback_capability_definitions().len(), 1);
    assert_eq!(snapshot.plan().target_capability_definitions().len(), 1);
    assert_eq!(
        snapshot.plan().rollback_catalog_revision(),
        fixture.rollback_catalog.catalog_revision()
    );
    assert_eq!(
        snapshot.plan().target_catalog_revision(),
        fixture.target_catalog.catalog_revision()
    );
    assert_eq!(
        snapshot.plan().rollback_catalog_digest(),
        fixture.rollback_catalog.catalog_digest()
    );
    assert_eq!(
        snapshot.plan().target_catalog_digest(),
        fixture.target_catalog.catalog_digest()
    );
    assert_eq!(
        snapshot.plan().rollback_registry_revision(),
        fixture.rollback_registry.registry_revision()
    );
    assert_eq!(
        snapshot.plan().target_registry_revision(),
        fixture.target_registry.registry_revision()
    );
    assert_eq!(
        snapshot.plan().rollback_registry_digest(),
        fixture.rollback_registry.registry_digest()
    );
    assert_eq!(
        snapshot.plan().target_registry_digest(),
        fixture.target_registry.registry_digest()
    );
    assert_ne!(
        snapshot.plan().plan_digest(),
        fixture.target_pin.package_digest()
    );
}

#[test]
fn empty_public_repository_and_replay_are_non_authoritative() {
    let id = update_id("update:empty-public");
    assert_eq!(replay(std::iter::empty()).expect("empty replay"), None);
    let mut repository = InMemoryPackageUpdateRepository::new();
    assert_eq!(repository.load_exact(&id).expect("empty load"), None);
    assert_eq!(
        repository.event_history(&id).expect("empty history"),
        Vec::new()
    );

    let command = UpdateCommand::cancel(
        update_command_id("update-cmd:cancel-missing"),
        id.clone(),
        update_revision("update-revision:1"),
    )
    .expect("cancel command");
    let first = repository
        .execute(command.clone())
        .expect("missing aggregate rejection receipt is persisted");
    let second = repository
        .execute(command)
        .expect("missing aggregate rejection is idempotent");
    assert_eq!(first, second);
    assert_eq!(first.command().update_id(), &id);
    assert_eq!(
        first.command().command_id().as_str(),
        "update-cmd:cancel-missing"
    );
    assert_eq!(
        first.outcome(),
        &UpdateCommandOutcome::Rejected {
            error: UpdateDecisionError::AggregateMissing,
        }
    );
    assert_eq!(repository.load_exact(&id).expect("still empty"), None);
    assert_eq!(
        repository.event_history(&id).expect("still no authority"),
        Vec::new()
    );
}

#[test]
fn public_errors_and_debug_are_category_only_and_redacted() {
    let fixture = PublicUpdateFixture::new_with_sentinels("sentinel-redacted");
    let command = stage_command(
        &fixture,
        "update-cmd:sentinel-redacted-command",
        "update:sentinel-redacted-update",
    );
    let mut repository = update_repository(&fixture);
    let receipt = repository.execute(command.clone()).expect("stage receipt");
    let duplicate_receipt = repository
        .execute(command.clone())
        .expect("idempotent stage");
    assert_eq!(receipt, duplicate_receipt);
    let conflict = repository
        .execute(
            UpdateCommand::cancel(
                update_command_id("update-cmd:sentinel-redacted-conflict"),
                update_id("update:sentinel-redacted-missing"),
                update_revision("update-revision:1"),
            )
            .expect("cancel command"),
        )
        .expect("missing update rejection receipt");
    let history = repository
        .event_history(&update_id("update:sentinel-redacted-update"))
        .expect("history");

    let construction_error = PackageUpdateId::parse("update:").unwrap_err();
    let decision_error = match conflict.outcome() {
        UpdateCommandOutcome::Rejected { error } => *error,
        UpdateCommandOutcome::Accepted { .. } => panic!("expected rejection"),
    };
    let replay_error = replay(history.iter().chain(history.iter())).unwrap_err();
    let repository_error = UpdateRepositoryError::DecisionRejected(decision_error);
    let category_renderings = [
        format!("{construction_error}"),
        format!("{construction_error:?}"),
        format!("{decision_error}"),
        format!("{decision_error:?}"),
        format!("{replay_error}"),
        format!("{replay_error:?}"),
        format!("{repository_error}"),
        format!("{repository_error:?}"),
    ];
    assert!(
        category_renderings
            .iter()
            .any(|value| value.contains("InvalidUpdateId"))
    );
    assert!(
        category_renderings
            .iter()
            .any(|value| value.contains("AggregateMissing"))
    );
    assert!(
        category_renderings
            .iter()
            .any(|value| value.contains("DuplicateCommandId"))
    );
    assert!(
        category_renderings
            .iter()
            .any(|value| value.contains("DecisionRejected"))
    );

    let accepted_snapshot = match receipt.outcome() {
        UpdateCommandOutcome::Accepted { snapshot, .. } => snapshot,
        UpdateCommandOutcome::Rejected { error } => panic!("stage rejected: {error:?}"),
    };
    let authority_renderings = [
        format!("{:?}", update_id("update:sentinel-redacted-update")),
        format!(
            "{:?}",
            update_command_id("update-cmd:sentinel-redacted-command")
        ),
        format!("{:?}", command),
        format!("{:?}", accepted_snapshot.plan()),
        format!("{:?}", accepted_snapshot),
        format!("{:?}", history),
        format!("{:?}", repository),
        format!("{:?}", receipt),
        format!("{:?}", receipt.outcome()),
        format!("{:?}", conflict),
        format!("{:?}", construction_error),
        format!("{construction_error}"),
        format!("{:?}", decision_error),
        format!("{decision_error}"),
        format!("{:?}", replay_error),
        format!("{replay_error}"),
        format!("{:?}", repository_error),
        format!("{repository_error}"),
    ];
    assert!(
        authority_renderings
            .iter()
            .any(|value| value.contains("PackageUpdateId(<redacted>)"))
    );
    assert!(
        authority_renderings
            .iter()
            .any(|value| value.contains("UpdateCommandId(<redacted>)"))
    );
    assert!(
        authority_renderings
            .iter()
            .any(|value| value.contains("PackageUpdatePlan"))
    );
    assert!(
        authority_renderings
            .iter()
            .any(|value| value.contains("UpdateCommandReceipt(<authority-redacted>)"))
    );

    let sentinels = [
        "sentinel-redacted-tenant",
        "sentinel-redacted-user",
        "sentinel-redacted-installation",
        "public.update-package",
        "sentinel-redacted-update",
        "sentinel-redacted-command",
        "sentinel-redacted-config",
        "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        fixture.target_pin.package_digest().as_str(),
        fixture.target_pin.capability_manifest_digest().as_str(),
        fixture.configuration.digest().as_str(),
    ];
    for rendered in authority_renderings {
        for sentinel in sentinels {
            assert!(
                !rendered.contains(sentinel),
                "leaked sentinel {sentinel} in {rendered}"
            );
        }
    }
}
