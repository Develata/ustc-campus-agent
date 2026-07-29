#![allow(clippy::expect_used, clippy::panic, clippy::unwrap_used)]

use ustc_campus_agent_core::identity::{TenantId, UserId};
use ustc_campus_agent_core::invocation::{
    CatalogRevision, ComponentId, ComponentKind, ComponentVersion, ExecutionIdentity,
    InstallationId, InstallationRevision, InstallationState as ResolverInstallationState,
    PackageId, PackageVersion, Sha256Digest,
};
use ustc_campus_agent_core::market::installation::{
    ConfigurationKey, ConfigurationRevision, ConfigurationValue, InMemoryInstallationRepository,
    InstallationCommand, InstallationCommandId, InstallationCommandOutcome,
    InstallationConfiguration, InstallationDecisionError, InstallationEvent,
    InstallationEventSequence, InstallationPackagePin, InstallationReplayError,
    InstallationRepository, InstallationRepositoryError, InstalledComponentPin,
    ManagedInstallationState, NonSecretText, SecretRef, SecretRefId, decide, evolve, replay,
};

macro_rules! parsed {
    ($kind:ty, $value:expr) => {{
        match <$kind>::parse($value) {
            Ok(value) => value,
            Err(error) => panic!("fixture value must parse: {error}"),
        }
    }};
}

fn tenant(suffix: &str) -> TenantId {
    parsed!(TenantId, format!("tenant:market-{suffix}"))
}

fn user(suffix: &str) -> UserId {
    parsed!(UserId, format!("user:market-{suffix}"))
}

fn installation(suffix: &str) -> InstallationId {
    parsed!(InstallationId, format!("installation:market-{suffix}"))
}

fn revision(sequence: u64) -> InstallationRevision {
    parsed!(
        InstallationRevision,
        format!("installation-revision:{sequence}")
    )
}

fn command_id(suffix: &str) -> InstallationCommandId {
    parsed!(InstallationCommandId, format!("cmd:{suffix}"))
}

fn config_key(value: &str) -> ConfigurationKey {
    parsed!(ConfigurationKey, value)
}

fn non_secret(value: &str) -> NonSecretText {
    parsed!(NonSecretText, value)
}

fn secret_ref_id(value: &str) -> SecretRefId {
    parsed!(SecretRefId, value)
}

fn digest(byte: char) -> Sha256Digest {
    parsed!(
        Sha256Digest,
        format!("sha256:{}", byte.to_string().repeat(64))
    )
}

fn catalog_revision(value: &str) -> CatalogRevision {
    parsed!(CatalogRevision, value)
}

fn package_id(value: &str) -> PackageId {
    parsed!(PackageId, value)
}

fn package_version(value: &str) -> PackageVersion {
    parsed!(PackageVersion, value)
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

#[derive(Clone)]
struct Fixture {
    tenant_id: TenantId,
    user_id: UserId,
    installation_id: InstallationId,
    alternate_installation_id: InstallationId,
    package_pin: InstallationPackagePin,
    initial_configuration: InstallationConfiguration,
    alternate_configuration: InstallationConfiguration,
}

impl Fixture {
    fn new() -> Self {
        let tenant_id = tenant("primary");
        let user_id = user("primary");
        let installation_id = installation("primary");
        let alternate_installation_id = installation("alternate");
        let component = InstalledComponentPin::new(
            component_id("component:calendar"),
            ComponentKind::NativeRustComponent,
            component_version("component-version:7"),
            digest('2'),
            execution_identity("native:calendar-v7"),
        )
        .expect("valid component pin");
        let second_component = InstalledComponentPin::new(
            component_id("component:resource-pack"),
            ComponentKind::DeclarativeResourcePack,
            component_version("component-version:3"),
            digest('3'),
            execution_identity("resource:pack-v3"),
        )
        .expect("valid component pin");
        let package_pin = InstallationPackagePin::new(
            catalog_revision("catalog:reviewed-v7"),
            package_id("ustc.lifecycle-suite"),
            package_version("1.2.3"),
            digest('1'),
            vec![second_component.clone(), component.clone()],
            digest('4'),
            digest('5'),
        )
        .expect("valid package pin");
        let initial_configuration = InstallationConfiguration::new(
            &tenant_id,
            vec![
                (
                    config_key("apiMode"),
                    ConfigurationValue::Text(non_secret("readonly")),
                ),
                (config_key("maxItems"), ConfigurationValue::Integer(32)),
                (config_key("enabled"), ConfigurationValue::Boolean(true)),
                (
                    config_key("token"),
                    ConfigurationValue::Secret(
                        SecretRef::new(tenant_id.clone(), secret_ref_id("secret-ref:market-token"))
                            .expect("valid tenant secret ref"),
                    ),
                ),
            ],
        )
        .expect("valid initial configuration");
        let alternate_configuration = InstallationConfiguration::new(
            &tenant_id,
            vec![
                (config_key("enabled"), ConfigurationValue::Boolean(false)),
                (config_key("maxItems"), ConfigurationValue::Integer(64)),
                (
                    config_key("apiMode"),
                    ConfigurationValue::Text(non_secret("bounded")),
                ),
            ],
        )
        .expect("valid alternate configuration");
        Self {
            tenant_id,
            user_id,
            installation_id,
            alternate_installation_id,
            package_pin,
            initial_configuration,
            alternate_configuration,
        }
    }

    fn install_command(&self, id: &str) -> InstallationCommand {
        InstallationCommand::install(
            command_id(id),
            self.installation_id.clone(),
            self.tenant_id.clone(),
            self.user_id.clone(),
            self.package_pin.clone(),
            self.initial_configuration.clone(),
        )
        .expect("valid install command")
    }

    fn configure_command(&self, id: &str, expected_revision: u64) -> InstallationCommand {
        InstallationCommand::configure(
            command_id(id),
            self.installation_id.clone(),
            revision(expected_revision),
            self.alternate_configuration.clone(),
        )
        .expect("valid configure command")
    }

    fn disable_command(&self, id: &str, expected_revision: u64) -> InstallationCommand {
        InstallationCommand::disable(
            command_id(id),
            self.installation_id.clone(),
            revision(expected_revision),
        )
        .expect("valid disable command")
    }

    fn revoke_command(&self, id: &str, expected_revision: u64) -> InstallationCommand {
        InstallationCommand::revoke(
            command_id(id),
            self.installation_id.clone(),
            revision(expected_revision),
        )
        .expect("valid revoke command")
    }

    fn uninstall_command(&self, id: &str, expected_revision: u64) -> InstallationCommand {
        InstallationCommand::uninstall(
            command_id(id),
            self.installation_id.clone(),
            revision(expected_revision),
        )
        .expect("valid uninstall command")
    }

    fn installed(
        &self,
    ) -> (
        InstallationEvent,
        ustc_campus_agent_core::market::installation::InstallationAggregate,
    ) {
        let event = decide(None, &self.install_command("install")).expect("install must decide");
        let aggregate = evolve(None, &event).expect("install event must evolve");
        (event, aggregate)
    }
}

fn assert_state(
    aggregate: &ustc_campus_agent_core::market::installation::InstallationAggregate,
    state: ManagedInstallationState,
    revision_number: u64,
    config_revision: u64,
) {
    assert_eq!(aggregate.state(), state);
    assert_eq!(aggregate.revision(), &revision(revision_number));
    assert_eq!(
        aggregate.configuration_revision(),
        ConfigurationRevision::new(config_revision).unwrap()
    );
}

fn accepted_event(
    receipt: &ustc_campus_agent_core::market::installation::InstallationCommandReceipt,
) -> InstallationEvent {
    match receipt.outcome() {
        InstallationCommandOutcome::Accepted { event, snapshot } => {
            assert_eq!(snapshot.revision(), event.post_revision());
            event.clone()
        }
        InstallationCommandOutcome::Rejected { error } => {
            panic!("expected accepted receipt, got {error:?}")
        }
    }
}

fn rejected_error(
    receipt: &ustc_campus_agent_core::market::installation::InstallationCommandReceipt,
) -> InstallationDecisionError {
    match receipt.outcome() {
        InstallationCommandOutcome::Accepted { event, .. } => {
            panic!("expected rejected receipt, got event {event:?}")
        }
        InstallationCommandOutcome::Rejected { error } => *error,
    }
}

#[test]
fn configuration_values_are_canonical_bounded_and_secret_safe() {
    let fixture = Fixture::new();
    let reversed = InstallationConfiguration::new(
        &fixture.tenant_id,
        vec![
            (
                config_key("token"),
                ConfigurationValue::Secret(
                    SecretRef::new(
                        fixture.tenant_id.clone(),
                        secret_ref_id("secret-ref:market-token"),
                    )
                    .expect("valid tenant secret ref"),
                ),
            ),
            (config_key("enabled"), ConfigurationValue::Boolean(true)),
            (config_key("maxItems"), ConfigurationValue::Integer(32)),
            (
                config_key("apiMode"),
                ConfigurationValue::Text(non_secret("readonly")),
            ),
        ],
    )
    .expect("same configuration with reversed source order");

    assert_eq!(fixture.initial_configuration, reversed);
    assert_eq!(fixture.initial_configuration.digest(), reversed.digest());
    assert_eq!(
        fixture.initial_configuration.digest().as_str(),
        "sha256:895ea89495332089b2bd42bb65dd7ce9e1c26d39c1f69182ce95743345f3bf1a"
    );
    assert_eq!(
        fixture.alternate_configuration.digest().as_str(),
        "sha256:4e9999291f79c612f05cbb056ea621cedb733ee74cee072b5f7509d8ef6535ce"
    );

    assert!(ConfigurationKey::parse("1bad").is_err());
    assert!(ConfigurationKey::parse("A".repeat(65)).is_err());
    assert!(NonSecretText::parse("").is_err());
    assert!(NonSecretText::parse("contains\ncontrol").is_err());
    assert!(NonSecretText::parse("x".repeat(4097)).is_err());
    assert!(SecretRefId::parse("secret-ref:").is_err());
    assert!(SecretRefId::parse(format!("secret-ref:{}", "a".repeat(119))).is_err());

    assert_eq!(
        InstallationConfiguration::new(
            &fixture.tenant_id,
            vec![
                (config_key("dup"), ConfigurationValue::Boolean(true)),
                (config_key("dup"), ConfigurationValue::Boolean(false)),
            ],
        ),
        Err(ustc_campus_agent_core::market::installation::InstallationConstructionError::DuplicateConfigurationKey)
    );
    assert_eq!(
        InstallationConfiguration::new(
            &fixture.tenant_id,
            vec![(
                config_key("foreignSecret"),
                ConfigurationValue::Secret(
                    SecretRef::new(tenant("other"), secret_ref_id("secret-ref:foreign"))
                        .expect("valid secret ref shape"),
                ),
            )],
        ),
        Err(ustc_campus_agent_core::market::installation::InstallationConstructionError::CrossTenantSecretRef)
    );
    let other_tenant_text_only = InstallationConfiguration::new(
        &tenant("other"),
        vec![(
            config_key("apiMode"),
            ConfigurationValue::Text(non_secret("readonly")),
        )],
    )
    .expect("text-only foreign tenant configuration is a valid value for that tenant");
    assert_ne!(
        fixture.initial_configuration.digest(),
        other_tenant_text_only.digest()
    );
    assert_eq!(
        InstallationCommand::install(
            command_id("cross-tenant-install"),
            fixture.installation_id.clone(),
            fixture.tenant_id.clone(),
            fixture.user_id.clone(),
            fixture.package_pin.clone(),
            other_tenant_text_only.clone(),
        ),
        Err(ustc_campus_agent_core::market::installation::InstallationConstructionError::CrossTenantConfiguration)
    );
    assert!(
        InstallationConfiguration::new(
            &fixture.tenant_id,
            (0..129)
                .map(|index| {
                    (
                        config_key(&format!("k{index}")),
                        ConfigurationValue::Integer(index),
                    )
                })
                .collect(),
        )
        .is_err()
    );

    let debug = format!("{:?}", fixture.initial_configuration);
    let digest_display = fixture.initial_configuration.digest().as_str().to_owned();
    assert!(!debug.contains("readonly"));
    assert!(!debug.contains("secret-ref:market-token"));
    assert!(!digest_display.contains("readonly"));
    assert!(!digest_display.contains("secret-ref:market-token"));
}

#[test]
fn package_pins_are_exact_canonical_and_duplicate_safe() {
    let fixture = Fixture::new();
    assert_eq!(fixture.package_pin.components().len(), 2);
    assert!(
        fixture
            .package_pin
            .components()
            .windows(2)
            .all(|pair| pair[0].component_id() < pair[1].component_id())
    );
    assert_eq!(
        InstalledComponentPin::new(
            component_id("component:calendar"),
            ComponentKind::NativeRustComponent,
            component_version("component-version:7"),
            digest('2'),
            execution_identity("native:calendar-v7"),
        )
        .expect("valid component")
        .to_installed_identity(),
        ustc_campus_agent_core::invocation::InstalledComponentIdentity {
            id: component_id("component:calendar"),
            version: component_version("component-version:7"),
            digest: digest('2'),
            execution_identity: execution_identity("native:calendar-v7"),
        }
    );

    let duplicate = InstalledComponentPin::new(
        component_id("component:dup"),
        ComponentKind::SkillComponent,
        component_version("component-version:1"),
        digest('b'),
        execution_identity("skill:dup"),
    )
    .expect("valid component");
    assert_eq!(
        InstallationPackagePin::new(
            catalog_revision("catalog:dup"),
            package_id("ustc.duplicate"),
            package_version("1.0.0"),
            digest('c'),
            vec![duplicate.clone(), duplicate],
            digest('d'),
            digest('e'),
        ),
        Err(ustc_campus_agent_core::market::installation::InstallationConstructionError::DuplicateComponentId)
    );
}

#[test]
fn legal_install_configure_revoke_and_uninstall_transitions_are_explicit() {
    let fixture = Fixture::new();
    let (install_event, installed) = fixture.installed();
    assert_state(
        &installed,
        ManagedInstallationState::InstalledDisabled,
        1,
        1,
    );
    assert_eq!(
        install_event.sequence(),
        InstallationEventSequence::new(1).unwrap()
    );
    assert_eq!(install_event.post_revision(), &revision(1));

    let configured_event = decide(
        Some(&installed),
        &fixture.configure_command("configure-from-initial-disabled", 1),
    )
    .expect("configure from initial disabled");
    let configured = evolve(Some(installed), &configured_event).expect("configured aggregate");
    assert_state(
        &configured,
        ManagedInstallationState::InstalledDisabled,
        2,
        2,
    );
    assert_eq!(configured.configuration(), &fixture.alternate_configuration);

    let revoked_event = decide(Some(&configured), &fixture.revoke_command("revoke", 2))
        .expect("revoke from nonterminal");
    let revoked = evolve(Some(configured), &revoked_event).expect("revoked aggregate");
    assert_state(&revoked, ManagedInstallationState::Revoked, 3, 2);

    let fixture = Fixture::new();
    let (_, installed) = fixture.installed();
    let uninstalled_event =
        decide(Some(&installed), &fixture.uninstall_command("uninstall", 1)).expect("uninstall");
    let uninstalled = evolve(Some(installed), &uninstalled_event).expect("uninstalled aggregate");
    assert_state(&uninstalled, ManagedInstallationState::Uninstalled, 2, 1);
}

#[test]
fn illegal_transitions_fail_closed_with_stable_categories() {
    let fixture = Fixture::new();
    assert_eq!(
        decide(None, &fixture.configure_command("configure-absent", 1)),
        Err(InstallationDecisionError::AggregateMissing)
    );
    assert_eq!(
        decide(None, &fixture.disable_command("disable-absent", 1)),
        Err(InstallationDecisionError::AggregateMissing)
    );

    let (_, installed) = fixture.installed();
    assert_eq!(
        decide(Some(&installed), &fixture.install_command("install-again")),
        Err(InstallationDecisionError::AggregateAlreadyPresent)
    );
    assert_eq!(
        decide(
            Some(&installed),
            &fixture.disable_command("disable-initial-disabled", 1),
        ),
        Err(InstallationDecisionError::IllegalTransition)
    );
    assert_eq!(
        decide(
            Some(&installed),
            &fixture.configure_command("wrong-revision", 2),
        ),
        Err(InstallationDecisionError::RevisionMismatch)
    );
    let foreign_configuration = InstallationConfiguration::new(
        &tenant("foreign"),
        vec![(config_key("enabled"), ConfigurationValue::Boolean(false))],
    )
    .expect("foreign text-only configuration is valid for that tenant");
    let foreign_wrong_revision = InstallationCommand::configure(
        command_id("foreign-wrong-revision"),
        fixture.installation_id.clone(),
        revision(2),
        foreign_configuration.clone(),
    )
    .expect("configure command constructor does not know aggregate tenant or revision");
    assert_eq!(
        decide(Some(&installed), &foreign_wrong_revision),
        Err(InstallationDecisionError::RevisionMismatch)
    );
    let foreign_configure = InstallationCommand::configure(
        command_id("foreign-configure"),
        fixture.installation_id.clone(),
        revision(1),
        foreign_configuration.clone(),
    )
    .expect("configure command constructor does not know aggregate tenant");
    assert_eq!(
        decide(Some(&installed), &foreign_configure),
        Err(InstallationDecisionError::TenantMismatch)
    );

    let revoked = evolve(
        Some(installed),
        &decide(
            Some(&fixture.installed().1),
            &fixture.revoke_command("terminal-revoke", 1),
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(
        decide(
            Some(&revoked),
            &fixture.configure_command("configure-terminal", 2),
        ),
        Err(InstallationDecisionError::TerminalState)
    );
    let terminal_foreign_configure = InstallationCommand::configure(
        command_id("terminal-foreign-configure"),
        fixture.installation_id.clone(),
        revision(99),
        foreign_configuration,
    )
    .expect("configure command constructor does not know aggregate terminal state");
    assert_eq!(
        decide(Some(&revoked), &terminal_foreign_configure),
        Err(InstallationDecisionError::TerminalState)
    );
}

#[test]
fn absence_terminal_and_reinstall_semantics_are_distinct() {
    let fixture = Fixture::new();
    let mut repo = InMemoryInstallationRepository::new();
    assert_eq!(repo.load_exact(&fixture.installation_id).unwrap(), None);

    let uninstall_without_install = repo
        .execute(fixture.uninstall_command("uninstall-missing", 1))
        .expect("domain rejection receipt is persisted");
    assert_eq!(
        rejected_error(&uninstall_without_install),
        InstallationDecisionError::AggregateMissing
    );
    assert_eq!(repo.load_exact(&fixture.installation_id).unwrap(), None);
    assert!(
        repo.event_history(&fixture.installation_id)
            .unwrap()
            .is_empty()
    );

    repo.execute(fixture.install_command("install-primary"))
        .unwrap();
    let uninstall_receipt = repo
        .execute(fixture.uninstall_command("uninstall-primary", 1))
        .unwrap();
    let uninstall_event = accepted_event(&uninstall_receipt);
    assert_eq!(uninstall_event.post_revision(), &revision(2));
    assert_eq!(
        repo.load_exact(&fixture.installation_id)
            .unwrap()
            .unwrap()
            .state(),
        ManagedInstallationState::Uninstalled
    );
    let reinstall_same_id = repo
        .execute(fixture.install_command("reinstall-same-id"))
        .unwrap();
    assert_eq!(
        rejected_error(&reinstall_same_id),
        InstallationDecisionError::AggregateAlreadyPresent
    );

    let reinstall = InstallationCommand::install(
        command_id("install-alternate-id"),
        fixture.alternate_installation_id.clone(),
        fixture.tenant_id.clone(),
        fixture.user_id.clone(),
        fixture.package_pin.clone(),
        fixture.initial_configuration.clone(),
    )
    .expect("valid alternate install");
    let receipt = repo.execute(reinstall).unwrap();
    assert_eq!(accepted_event(&receipt).post_revision(), &revision(1));
}

#[test]
fn repository_idempotency_persists_accepted_rejected_and_global_conflicts() {
    let fixture = Fixture::new();
    let mut repo = InMemoryInstallationRepository::new();

    let install = fixture.install_command("same-accepted");
    let first = repo.execute(install.clone()).unwrap();
    let duplicate = repo.execute(install).unwrap();
    assert_eq!(first, duplicate);
    assert_eq!(
        repo.event_history(&fixture.installation_id).unwrap().len(),
        1
    );

    let rejected = fixture.configure_command("same-rejected", 99);
    let first_rejection = repo.execute(rejected.clone()).unwrap();
    assert_eq!(
        rejected_error(&first_rejection),
        InstallationDecisionError::RevisionMismatch
    );
    repo.execute(fixture.configure_command("advance-state", 1))
        .unwrap();
    let duplicate_rejection = repo.execute(rejected).unwrap();
    assert_eq!(first_rejection, duplicate_rejection);
    assert_eq!(
        repo.event_history(&fixture.installation_id).unwrap().len(),
        2
    );

    let conflicting_same_installation = fixture.configure_command("same-accepted", 2);
    assert_eq!(
        repo.execute(conflicting_same_installation).unwrap_err(),
        InstallationRepositoryError::CommandConflict
    );

    let different_installation_same_command_id = InstallationCommand::install(
        command_id("same-accepted"),
        fixture.alternate_installation_id.clone(),
        fixture.tenant_id.clone(),
        fixture.user_id.clone(),
        fixture.package_pin.clone(),
        fixture.initial_configuration.clone(),
    )
    .expect("valid install command with conflicting global id");
    assert_eq!(
        repo.execute(different_installation_same_command_id)
            .unwrap_err(),
        InstallationRepositoryError::CommandConflict
    );
}

#[test]
fn repository_failure_injection_is_atomic_and_retryable() {
    let fixture = Fixture::new();
    let mut repo = InMemoryInstallationRepository::new();
    repo.fail_next_commit_for_testing();
    let command = fixture.install_command("atomic-install");
    assert_eq!(
        repo.execute(command.clone()).unwrap_err(),
        InstallationRepositoryError::InjectedPersistenceFailure
    );
    assert_eq!(repo.load_exact(&fixture.installation_id).unwrap(), None);
    assert!(
        repo.event_history(&fixture.installation_id)
            .unwrap()
            .is_empty()
    );

    let receipt = repo.execute(command).expect("retry after atomic failure");
    assert_eq!(
        accepted_event(&receipt).sequence(),
        InstallationEventSequence::new(1).unwrap()
    );
    assert_eq!(
        repo.event_history(&fixture.installation_id).unwrap().len(),
        1
    );

    repo.fail_next_commit_for_testing();
    let bad = fixture.configure_command("atomic-rejected-config", 99);
    assert_eq!(
        repo.execute(bad.clone()).unwrap_err(),
        InstallationRepositoryError::InjectedPersistenceFailure
    );
    let retry = repo
        .execute(bad)
        .expect("rejected command retried after no receipt commit");
    assert_eq!(
        rejected_error(&retry),
        InstallationDecisionError::RevisionMismatch
    );
}

#[test]
fn replay_accepts_success_histories_and_rejects_gap_duplicate_reorder_and_command_reuse() {
    let fixture = Fixture::new();
    let install_event = decide(None, &fixture.install_command("install")).unwrap();
    let installed = evolve(None, &install_event).unwrap();
    let configure_event =
        decide(Some(&installed), &fixture.configure_command("configure", 1)).unwrap();
    let configured = evolve(Some(installed.clone()), &configure_event).unwrap();
    let revoke_event = decide(Some(&configured), &fixture.revoke_command("revoke", 2)).unwrap();
    assert_eq!(
        evolve(Some(installed.clone()), &revoke_event),
        Err(InstallationReplayError::SequenceGap)
    );
    let replayed = replay([&install_event, &configure_event, &revoke_event])
        .expect("valid history must replay")
        .unwrap();
    assert_eq!(replayed.state(), ManagedInstallationState::Revoked);

    let duplicate_sequence = vec![install_event.clone(), install_event.clone()];
    assert_eq!(
        replay(&duplicate_sequence),
        Err(InstallationReplayError::SequenceDuplicate)
    );

    let gap = vec![install_event.clone(), revoke_event.clone()];
    assert_eq!(replay(&gap), Err(InstallationReplayError::SequenceGap));

    let reorder = vec![configure_event.clone(), install_event.clone()];
    assert_eq!(
        replay(&reorder),
        Err(InstallationReplayError::InitialEventNotInstalled)
    );

    let reused_command_event = decide(
        Some(&installed),
        &InstallationCommand::configure(
            command_id("install"),
            fixture.installation_id.clone(),
            revision(1),
            fixture.alternate_configuration.clone(),
        )
        .unwrap(),
    )
    .expect("decide does not own command-ledger idempotency");
    assert_eq!(
        replay([&install_event, &reused_command_event]),
        Err(InstallationReplayError::DuplicateCommandId)
    );
}

#[test]
fn replay_rejects_impossible_initial_post_terminal_and_redundant_field_mismatches() {
    let fixture = Fixture::new();
    let install_event = decide(None, &fixture.install_command("install")).unwrap();
    let installed = evolve(None, &install_event).unwrap();
    let configure_event =
        decide(Some(&installed), &fixture.configure_command("configure", 1)).unwrap();
    assert_eq!(
        replay([&configure_event]),
        Err(InstallationReplayError::InitialEventNotInstalled)
    );

    let revoke_event = decide(
        Some(&installed),
        &fixture.revoke_command("terminal-revoke", 1),
    )
    .unwrap();
    assert_eq!(
        replay([&install_event, &revoke_event, &configure_event]),
        Err(InstallationReplayError::PostTerminalEvent)
    );

    let other = Fixture::new();
    let other_install = InstallationCommand::install(
        command_id("other-install"),
        other.alternate_installation_id.clone(),
        other.tenant_id.clone(),
        other.user_id.clone(),
        other.package_pin.clone(),
        other.initial_configuration.clone(),
    )
    .expect("valid different install command");
    let other_installed_event = decide(None, &other_install).expect("different install event");
    assert_eq!(
        replay([&install_event, &other_installed_event]),
        Err(InstallationReplayError::SequenceDuplicate)
    );

    let other_config = {
        let other_installed = evolve(None, &other_installed_event).unwrap();
        decide(
            Some(&other_installed),
            &InstallationCommand::configure(
                command_id("other-configure"),
                other.alternate_installation_id.clone(),
                revision(1),
                other.alternate_configuration.clone(),
            )
            .expect("valid other configure command"),
        )
        .expect("other configure event")
    };
    assert_eq!(
        replay([&install_event, &other_config]),
        Err(InstallationReplayError::RedundantFieldMismatch)
    );
}

#[test]
fn resolver_projection_maps_managed_states_without_grants_or_resolver_mutation() {
    let fixture = Fixture::new();
    let (_, installed) = fixture.installed();
    let disabled_snapshot = installed
        .to_resolver_snapshot()
        .expect("installed-disabled projects to disabled resolver state");
    assert_eq!(disabled_snapshot.state, ResolverInstallationState::Disabled);
    assert_eq!(disabled_snapshot.id, fixture.installation_id);
    assert_eq!(disabled_snapshot.tenant_id, fixture.tenant_id);
    assert_eq!(disabled_snapshot.user_id, fixture.user_id);
    assert_eq!(disabled_snapshot.package_digest, digest('1'));
    assert_eq!(disabled_snapshot.revision, revision(1));

    let revoked_event = decide(
        Some(&installed),
        &fixture.revoke_command("project-revoked", 1),
    )
    .unwrap();
    let revoked = evolve(Some(installed), &revoked_event).unwrap();
    assert_eq!(
        revoked.to_resolver_snapshot().unwrap().state,
        ResolverInstallationState::Revoked
    );

    let fixture = Fixture::new();
    let (_, installed) = fixture.installed();
    let uninstalled_event = decide(
        Some(&installed),
        &fixture.uninstall_command("project-uninstalled", 1),
    )
    .unwrap();
    let uninstalled = evolve(Some(installed), &uninstalled_event).unwrap();
    assert_eq!(uninstalled.to_resolver_snapshot(), None);
}

#[test]
fn event_receipt_and_error_debug_display_do_not_leak_configuration_or_secret_material() {
    let fixture = Fixture::new();
    let mut repo = InMemoryInstallationRepository::new();
    let rejected = repo
        .execute(fixture.configure_command("leak-rejected", 1))
        .expect("rejection receipt persisted");
    let accepted = repo
        .execute(fixture.install_command("leak-install"))
        .unwrap();
    let history = repo.event_history(&fixture.installation_id).unwrap();

    let sentinel_text = "readonly";
    let sentinel_secret = "secret-ref:market-token";
    for rendered in [
        format!("{rejected:?}"),
        format!("{accepted:?}"),
        format!("{:?}", history),
        format!("{}", rejected_error(&rejected)),
        format!("{:?}", rejected_error(&rejected)),
    ] {
        assert!(
            !rendered.contains(sentinel_text),
            "leaked text in {rendered}"
        );
        assert!(
            !rendered.contains(sentinel_secret),
            "leaked secret ref in {rendered}"
        );
    }
}
