use super::*;
use crate::{
    invocation::{
        CatalogRevision, ToolId, UnvalidatedSchemaNodeV0, UnvalidatedToolInputSchemaV0,
        ValidatedToolInputSchemaV0,
    },
    market::{
        capability::load_capability_registry,
        configuration_catalog::load_package_configuration,
        configuration_schema::{ConfigurationFieldSchema, ConfigurationSchema},
        load_package_manifest,
    },
};
use serde_json::{Value, json};

struct Fixture {
    tenant: TenantId,
    user: UserId,
    package: ValidatedPackageManifest,
    configuration: ValidatedPackageConfiguration,
    registry: CapabilityRegistry,
    values: InstallationConfiguration,
    installations: InMemoryInstallationRepository,
    grants: InMemoryGrantRepository,
    snapshot: InstallationSnapshot,
}
impl Fixture {
    fn new(kind: &str, count: usize) -> Self {
        let tenant = TenantId::parse("tenant:admission").expect("tenant");
        let user = UserId::parse("user:owner").expect("user");
        let mut raw: Value = serde_json::from_slice(include_bytes!(
            "../../../../../market/packages/ustc.simple-calendar/package.json"
        ))
        .expect("manifest");
        raw["capabilities"] = json!(["campus.public_rules.read"]);
        raw["components"] = json!(
            (0..count)
                .map(|index| json!({"type": kind,"path":format!("skills/member-{index}/SKILL.md")}))
                .collect::<Vec<_>>()
        );
        let package = load_package_manifest(&serde_json::to_vec(&raw).expect("manifest bytes"))
            .expect("checked manifest");
        let schema = ConfigurationSchema::new(vec![ConfigurationFieldSchema::boolean(
            ConfigurationKey::parse("enabled").expect("key"),
            true,
        )])
        .expect("schema");
        let sidecar = json!({"schemaVersion":"package-component-configuration/v1", "packageId":package.package_id().as_str(), "packageVersion":package.package_version().as_str(), "packageDigest":package.package_digest().as_str(), "componentSetDigest":package.component_declaration_set_digest().as_str(), "capabilityManifestDigest":package.capability_manifest_digest().as_str(), "components":(0..count).map(|index| json!({"path":format!("skills/member-{index}/SKILL.md"), "type":kind, "mode":null, "componentId":format!("component:member-{index}"), "componentVersion":"1", "componentDigest":Sha256Digest::from_bytes(b"reviewed artifact").as_str(), "executionIdentity":"execution:reviewed", "schemaDigest":schema.digest().as_str(), "fields":[{"key":"enabled", "kind":"boolean", "required":true}]})).collect::<Vec<_>>()});
        let configuration = load_package_configuration(
            &serde_json::to_vec(&sidecar).expect("sidecar"),
            &package,
            &CatalogRevision::parse("catalog:reviewed").expect("revision"),
        )
        .expect("bindings");
        let registry = load_capability_registry(include_bytes!(
            "../../../../../market/capabilities/registry.json"
        ))
        .expect("registry");
        let values = InstallationConfiguration::new(
            &tenant,
            vec![(
                ConfigurationKey::parse("enabled").expect("key"),
                ConfigurationValue::Boolean(true),
            )],
        )
        .expect("configuration");
        let command = InstallationCommand::install(
            InstallationCommandId::parse("cmd:install").expect("command"),
            InstallationId::parse("installation:admission").expect("id"),
            tenant.clone(),
            user.clone(),
            configuration.package_pin().clone(),
            values.clone(),
        )
        .expect("install");
        let mut installations = InMemoryInstallationRepository::new();
        let receipt = installations.execute(command).expect("install receipt");
        let InstallationCommandOutcome::Accepted { snapshot, .. } = receipt.outcome() else {
            panic!("accepted install");
        };
        Self {
            tenant,
            user,
            package,
            configuration,
            registry,
            values,
            installations,
            grants: InMemoryGrantRepository::new(),
            snapshot: snapshot.clone(),
        }
    }
    fn readiness(&self) -> ComponentReadiness {
        ComponentReadiness::skill(
            self.configuration
                .bindings()
                .values()
                .next()
                .expect("binding"),
            &self.values,
            &Sha256Digest::from_bytes(b"reviewed artifact"),
        )
        .expect("readiness")
    }
    fn grant_request(&self) -> GrantAdmissionRequest {
        GrantAdmissionRequest {
            installation_id: self.snapshot.installation_id().clone(),
            expected_revision: self.snapshot.revision().clone(),
            command_id: GrantCommandId::parse("grant-cmd:approve").expect("id"),
            approval_id: GrantApprovalId::parse("grant-approval:review").expect("id"),
            snapshot_id: GrantSnapshotId::parse("grant:admission").expect("id"),
            capability_id: CapabilityId::parse("campus.public_rules.read").expect("id"),
            confirmation_policy: ConfirmationPolicy::Allow,
        }
    }
    fn enable_request(&self) -> EnableAdmissionRequest {
        EnableAdmissionRequest {
            installation_id: self.snapshot.installation_id().clone(),
            expected_revision: self.snapshot.revision().clone(),
            command_id: InstallationCommandId::parse("cmd:enable").expect("id"),
        }
    }
    fn issue(&mut self) {
        let request = self.grant_request();
        let service = MarketAdmissionService::new(
            &self.tenant,
            &self.user,
            &self.package,
            &self.configuration,
            &self.registry,
        )
        .expect("service");
        let receipt = service
            .issue_grant(&self.installations, &mut self.grants, request)
            .expect("grant receipt");
        assert!(matches!(
            receipt.outcome(),
            GrantCommandOutcome::Accepted { .. }
        ));
    }
}

#[test]
fn explicit_grant_then_enable_uses_original_repositories_and_disable_denies() {
    let mut fixture = Fixture::new("SkillComponent", 1);
    fixture.issue();
    let readiness = fixture.readiness();
    let request = fixture.enable_request();
    let service = MarketAdmissionService::new(
        &fixture.tenant,
        &fixture.user,
        &fixture.package,
        &fixture.configuration,
        &fixture.registry,
    )
    .expect("service");
    let receipt = service
        .enable(
            &mut fixture.installations,
            &fixture.grants,
            request,
            &readiness,
        )
        .expect("enable receipt");
    let InstallationCommandOutcome::Accepted { snapshot, .. } = receipt.outcome() else {
        panic!("accepted enable");
    };
    assert_eq!(snapshot.state(), ManagedInstallationState::Enabled);
    let disable = InstallationCommand::disable(
        InstallationCommandId::parse("cmd:disable").expect("id"),
        snapshot.installation_id().clone(),
        snapshot.revision().clone(),
    )
    .expect("disable");
    let receipt = fixture
        .installations
        .execute(disable)
        .expect("disable receipt");
    let InstallationCommandOutcome::Accepted { snapshot, .. } = receipt.outcome() else {
        panic!("accepted disable");
    };
    assert_eq!(snapshot.state(), ManagedInstallationState::Disabled);
    assert_eq!(
        snapshot.to_resolver_snapshot().expect("snapshot").state,
        crate::invocation::InstallationState::Disabled
    );
}

#[test]
fn missing_revoked_grant_owner_revision_and_configuration_drift_fail_closed() {
    let mut fixture = Fixture::new("SkillComponent", 1);
    let readiness = fixture.readiness();
    let request = fixture.enable_request();
    let service = MarketAdmissionService::new(
        &fixture.tenant,
        &fixture.user,
        &fixture.package,
        &fixture.configuration,
        &fixture.registry,
    )
    .expect("service");
    assert_eq!(
        service
            .enable(
                &mut fixture.installations,
                &fixture.grants,
                request,
                &readiness
            )
            .expect_err("missing grant"),
        AdmissionError::MissingActiveGrant
    );
    let other = UserId::parse("user:other").expect("user");
    let request = fixture.grant_request();
    let service = MarketAdmissionService::new(
        &fixture.tenant,
        &other,
        &fixture.package,
        &fixture.configuration,
        &fixture.registry,
    )
    .expect("service");
    assert_eq!(
        service
            .issue_grant(&fixture.installations, &mut fixture.grants, request)
            .expect_err("owner"),
        AdmissionError::OwnerMismatch
    );
    fixture.issue();
    let grant = fixture
        .grants
        .load_exact(&GrantSnapshotId::parse("grant:admission").expect("id"))
        .expect("repository")
        .expect("grant");
    fixture
        .grants
        .execute(
            GrantCommand::revoke(
                GrantCommandId::parse("grant-cmd:revoke").expect("id"),
                grant.snapshot_id().clone(),
                grant.version().clone(),
            )
            .expect("revoke"),
        )
        .expect("receipt");
    let mut request = fixture.enable_request();
    request.expected_revision =
        InstallationRevision::parse("installation-revision:999").expect("revision");
    let service = MarketAdmissionService::new(
        &fixture.tenant,
        &fixture.user,
        &fixture.package,
        &fixture.configuration,
        &fixture.registry,
    )
    .expect("service");
    assert_eq!(
        service
            .enable(
                &mut fixture.installations,
                &fixture.grants,
                request,
                &readiness
            )
            .expect_err("stale revision"),
        AdmissionError::RevisionMismatch
    );
    let request = fixture.enable_request();
    let service = MarketAdmissionService::new(
        &fixture.tenant,
        &fixture.user,
        &fixture.package,
        &fixture.configuration,
        &fixture.registry,
    )
    .expect("service");
    assert_eq!(
        service
            .enable(
                &mut fixture.installations,
                &fixture.grants,
                request,
                &readiness
            )
            .expect_err("revoked"),
        AdmissionError::MissingActiveGrant
    );
    let bad = InstallationConfiguration::new(&fixture.tenant, vec![]).expect("empty config");
    assert_eq!(
        ComponentReadiness::skill(
            fixture
                .configuration
                .bindings()
                .values()
                .next()
                .expect("binding"),
            &bad,
            &Sha256Digest::from_bytes(b"reviewed artifact")
        )
        .expect_err("missing required"),
        AdmissionError::InvalidConfiguration
    );
}

#[test]
fn multiple_components_require_complete_exact_artifact_readiness() {
    let fixture = Fixture::new("SkillComponent", 2);
    assert!(
        MarketAdmissionService::new(
            &fixture.tenant,
            &fixture.user,
            &fixture.package,
            &fixture.configuration,
            &fixture.registry
        )
        .is_ok()
    );
    let members: Vec<_> = fixture
        .configuration
        .bindings()
        .values()
        .map(|binding| {
            ComponentReadiness::skill(
                binding,
                &fixture.values,
                &Sha256Digest::from_bytes(b"reviewed artifact"),
            )
            .expect("member")
        })
        .collect();
    assert!(ComponentReadiness::package(vec![members[0].clone()]).is_err());
    assert!(ComponentReadiness::package(vec![members[0].clone(), members[0].clone()]).is_err());
    assert!(ComponentReadiness::package(members).is_ok());
    let fixture = Fixture::new("SkillComponent", 1);
    assert_eq!(
        ComponentReadiness::skill(
            fixture
                .configuration
                .bindings()
                .values()
                .next()
                .expect("binding"),
            &fixture.values,
            &Sha256Digest::from_bytes(b"wrong")
        )
        .expect_err("artifact"),
        AdmissionError::InvalidReadiness
    );
    assert!(!format!("{:?}", fixture.readiness()).contains("reviewed"));
}

fn tool() -> CatalogToolDefinition {
    let schema = ValidatedToolInputSchemaV0::try_from(UnvalidatedToolInputSchemaV0 {
        dialect: "tool-input-schema/v0".into(),
        root: UnvalidatedSchemaNodeV0::Object {
            properties: Vec::new(),
            required: Vec::new(),
        },
    })
    .expect("schema");
    CatalogToolDefinition {
        id: ToolId::parse("tool:read").expect("id"),
        model_visible_name: "campus_read".into(),
        description: "Read campus facts".into(),
        capability_id: CapabilityId::parse("campus.public_rules.read").expect("capability"),
        claimed_input_schema_digest: schema.digest().clone(),
        input_schema: Some(schema),
    }
}
#[test]
fn mcp_inventory_is_reviewed_complete_bound_and_order_independent() {
    let fixture = Fixture::new("McpServerComponent", 1);
    let binding = fixture
        .configuration
        .bindings()
        .values()
        .next()
        .expect("binding");
    let first = tool();
    let mut second = tool();
    second.id = ToolId::parse("tool:second").expect("id");
    second.model_visible_name = "campus_second".into();
    let digest = mcp_inventory_digest(binding, &fixture.values, &[first.clone(), second.clone()])
        .expect("inventory");
    assert_eq!(
        mcp_inventory_digest(binding, &fixture.values, &[second.clone(), first.clone()])
            .expect("reordered"),
        digest
    );
    assert_eq!(
        ComponentReadiness::mcp(binding, &fixture.values, &[first.clone(), second], &digest)
            .expect("reviewed")
            .digest(),
        &digest
    );
    assert_eq!(
        ComponentReadiness::mcp(
            binding,
            &fixture.values,
            std::slice::from_ref(&first),
            &digest
        )
        .expect_err("inventory drift"),
        AdmissionError::InventoryNotReviewed
    );
    assert_eq!(
        mcp_inventory_digest(binding, &fixture.values, &[first.clone(), first.clone()])
            .expect_err("duplicate"),
        AdmissionError::InvalidInventory
    );
    let mut invalid = first.clone();
    invalid.claimed_input_schema_digest = Sha256Digest::from_bytes(b"wrong");
    assert_eq!(
        mcp_inventory_digest(binding, &fixture.values, &[invalid]).expect_err("schema mismatch"),
        AdmissionError::InvalidInventory
    );
    let changed = InstallationConfiguration::new(
        &fixture.tenant,
        vec![(
            ConfigurationKey::parse("enabled").expect("key"),
            ConfigurationValue::Boolean(false),
        )],
    )
    .expect("values");
    assert_eq!(
        ComponentReadiness::mcp(binding, &changed, &[first], &digest)
            .expect_err("configuration drift"),
        AdmissionError::InventoryNotReviewed
    );
}

#[test]
fn configure_invalidates_old_readiness_and_old_grant_without_enabling() {
    let mut fixture = Fixture::new("SkillComponent", 1);
    fixture.issue();
    let readiness = fixture.readiness();
    let changed = InstallationConfiguration::new(
        &fixture.tenant,
        vec![(
            ConfigurationKey::parse("enabled").expect("key"),
            ConfigurationValue::Boolean(false),
        )],
    )
    .expect("configuration");
    let command = InstallationCommand::configure(
        InstallationCommandId::parse("cmd:configure").expect("command"),
        fixture.snapshot.installation_id().clone(),
        fixture.snapshot.revision().clone(),
        changed.clone(),
    )
    .expect("configure");
    let receipt = fixture
        .installations
        .execute(command)
        .expect("configuration receipt");
    let InstallationCommandOutcome::Accepted { snapshot, .. } = receipt.outcome() else {
        panic!("accepted configuration");
    };
    fixture.snapshot = snapshot.clone();
    let request = fixture.enable_request();
    let service = MarketAdmissionService::new(
        &fixture.tenant,
        &fixture.user,
        &fixture.package,
        &fixture.configuration,
        &fixture.registry,
    )
    .expect("service");
    assert_eq!(
        service
            .enable(
                &mut fixture.installations,
                &fixture.grants,
                request,
                &readiness
            )
            .expect_err("old readiness"),
        AdmissionError::InvalidReadiness
    );
    fixture.values = changed;
    let readiness = fixture.readiness();
    let request = fixture.enable_request();
    let service = MarketAdmissionService::new(
        &fixture.tenant,
        &fixture.user,
        &fixture.package,
        &fixture.configuration,
        &fixture.registry,
    )
    .expect("service");
    assert_eq!(
        service
            .enable(
                &mut fixture.installations,
                &fixture.grants,
                request,
                &readiness
            )
            .expect_err("old grant"),
        AdmissionError::InvalidGrant
    );
    assert_eq!(
        fixture
            .installations
            .load_exact(fixture.snapshot.installation_id())
            .expect("load")
            .expect("installed")
            .state(),
        ManagedInstallationState::InstalledDisabled
    );
}

#[test]
fn tenant_and_full_component_pin_are_checked_before_grant_mutation() {
    let mut fixture = Fixture::new("SkillComponent", 1);
    let foreign = TenantId::parse("tenant:foreign").expect("tenant");
    let request = fixture.grant_request();
    let service = MarketAdmissionService::new(
        &foreign,
        &fixture.user,
        &fixture.package,
        &fixture.configuration,
        &fixture.registry,
    )
    .expect("service");
    assert_eq!(
        service
            .issue_grant(&fixture.installations, &mut fixture.grants, request)
            .expect_err("tenant"),
        AdmissionError::OwnerMismatch
    );
    let pin = fixture.configuration.package_pin();
    let component = &pin.components()[0];
    let changed = InstalledComponentPin::new(
        component.component_id().clone(),
        component.kind(),
        component.version().clone(),
        component.digest().clone(),
        crate::invocation::ExecutionIdentity::parse("execution:substituted").expect("identity"),
    )
    .expect("component");
    let forged = InstallationPackagePin::new(
        pin.catalog_revision().clone(),
        pin.package_id().clone(),
        pin.package_version().clone(),
        pin.package_digest().clone(),
        vec![changed],
        pin.component_set_digest().clone(),
        pin.capability_manifest_digest().clone(),
    )
    .expect("pin");
    let command = InstallationCommand::install(
        InstallationCommandId::parse("cmd:substituted").expect("command"),
        fixture.snapshot.installation_id().clone(),
        fixture.tenant.clone(),
        fixture.user.clone(),
        forged,
        fixture.values.clone(),
    )
    .expect("install");
    let mut substituted = InMemoryInstallationRepository::new();
    substituted.execute(command).expect("install receipt");
    let request = fixture.grant_request();
    let service = MarketAdmissionService::new(
        &fixture.tenant,
        &fixture.user,
        &fixture.package,
        &fixture.configuration,
        &fixture.registry,
    )
    .expect("service");
    assert_eq!(
        service
            .issue_grant(&substituted, &mut fixture.grants, request)
            .expect_err("full pin"),
        AdmissionError::PackageMismatch
    );
    assert!(
        fixture
            .grants
            .load_exact(&GrantSnapshotId::parse("grant:admission").expect("id"))
            .expect("repository")
            .is_none()
    );
}

#[test]
fn current_registry_and_mcp_capabilities_cannot_expand_manifest_authority() {
    let mut fixture = Fixture::new("McpServerComponent", 1);
    let mut extra = tool();
    extra.capability_id = CapabilityId::parse("user.own_calendar_items.write").expect("capability");
    let binding = fixture
        .configuration
        .bindings()
        .values()
        .next()
        .expect("binding");
    let digest = mcp_inventory_digest(binding, &fixture.values, std::slice::from_ref(&extra))
        .expect("syntactic inventory");
    let readiness = ComponentReadiness::mcp(binding, &fixture.values, &[extra], &digest)
        .expect("consistency only");
    let request = fixture.enable_request();
    let service = MarketAdmissionService::new(
        &fixture.tenant,
        &fixture.user,
        &fixture.package,
        &fixture.configuration,
        &fixture.registry,
    )
    .expect("service");
    assert_eq!(
        service
            .enable(
                &mut fixture.installations,
                &fixture.grants,
                request,
                &readiness
            )
            .expect_err("undeclared capability"),
        AdmissionError::InvalidCapability
    );
    let mut raw: Value = serde_json::from_slice(include_bytes!(
        "../../../../../market/capabilities/registry.json"
    ))
    .expect("registry");
    let capability = raw["capabilities"]
        .as_array_mut()
        .expect("definitions")
        .iter_mut()
        .find(|value| value["id"] == "campus.public_rules.read")
        .expect("definition");
    capability["status"] = json!("Revoked");
    capability["autoGrant"] = json!("Never");
    let registry = load_capability_registry(&serde_json::to_vec(&raw).expect("registry bytes"))
        .expect("changed registry");
    assert!(matches!(
        MarketAdmissionService::new(
            &fixture.tenant,
            &fixture.user,
            &fixture.package,
            &fixture.configuration,
            &registry
        ),
        Err(AdmissionError::InvalidCapability)
    ));
}
