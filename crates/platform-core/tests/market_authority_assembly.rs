use std::collections::BTreeSet;
use ustc_campus_agent_core::invocation::*;
use ustc_campus_agent_core::market::authority::{
    AuthorityRepositoryError, InMemoryInvocationAuthorityRepository,
    InvocationAuthorityReadTransaction, InvocationAuthorityRepository, InvocationAuthorityService,
    InvocationRecheckError, ProjectionAssemblyError,
};

macro_rules! parsed {
    ($kind:ty, $value:expr) => {{
        match <$kind>::parse($value) {
            Ok(value) => value,
            Err(error) => panic!("fixture value must parse: {error}"),
        }
    }};
}

fn digest(byte: u8) -> Sha256Digest {
    parsed!(
        Sha256Digest,
        format!("sha256:{}", format!("{byte:02x}").repeat(32))
    )
}

#[derive(Clone)]
struct Fixture {
    request: ToolProjectionRequest,
    target: InvocationTarget,
    catalog: CatalogPackageRevision,
    installation: PluginInstallationSnapshot,
    grant: CapabilityGrantSnapshot,
    policy: InvocationPolicySnapshot,
}

fn fixture() -> Fixture {
    let tenant_id = parsed!(TenantId, "tenant:synthetic");
    let user_id = parsed!(UserId, "user:synthetic");
    let installation_id = parsed!(InstallationId, "installation:synthetic");
    let package_id = parsed!(PackageId, "synthetic.resolver");
    let package_version = parsed!(PackageVersion, "1.2.3");
    let component_id = parsed!(ComponentId, "component:resolver");
    let component_version = parsed!(ComponentVersion, "component-version:1");
    let execution_identity = parsed!(ExecutionIdentity, "native:synthetic-resolver");
    let tool_id = parsed!(ToolId, "tool:search");
    let capability_id = parsed!(CapabilityId, "campus.public_rules.read");
    let object_scope = parsed!(ObjectScope, "scope:campus-public");
    let input_schema = ValidatedToolInputSchemaV0::try_from(UnvalidatedToolInputSchemaV0 {
        dialect: "tool-input-schema/v0".to_owned(),
        root: UnvalidatedSchemaNodeV0::Object {
            properties: vec![
                (
                    "query".to_owned(),
                    UnvalidatedSchemaNodeV0::String { enum_values: None },
                ),
                ("count".to_owned(), UnvalidatedSchemaNodeV0::Integer),
            ],
            required: vec!["query".to_owned()],
        },
    })
    .expect("valid fixture schema");
    let target = InvocationTarget {
        installation_id: installation_id.clone(),
        package_id: package_id.clone(),
        package_version: package_version.clone(),
        component_id: component_id.clone(),
        tool_id: tool_id.clone(),
        capability_id: capability_id.clone(),
        object_scope: object_scope.clone(),
    };
    let source_policy = SourcePolicyIdentity {
        id: parsed!(SourcePolicyId, "source-policy:synthetic-v1"),
        digest: digest(0x42),
    };
    let capability_manifest_digest = digest(0x43);
    Fixture {
        request: ToolProjectionRequest {
            tenant_id: tenant_id.clone(),
            user_id: user_id.clone(),
            run_id: parsed!(RunId, "run:synthetic-1"),
            turn_id: parsed!(TurnId, "turn:1"),
            activation_allowlist: None,
        },
        target: target.clone(),
        catalog: CatalogPackageRevision {
            catalog_revision: parsed!(CatalogRevision, "catalog:synthetic-v4"),
            package_id: package_id.clone(),
            package_version: package_version.clone(),
            package_digest: digest(0x44),
            runnable: true,
            revoked: false,
            capability_manifest_digest: capability_manifest_digest.clone(),
            source_policy: Some(source_policy.clone()),
            component: Some(CatalogComponentRevision {
                id: component_id.clone(),
                kind: ComponentKind::NativeRustComponent,
                version: component_version.clone(),
                digest: digest(0x45),
                execution_identity: execution_identity.clone(),
                declared_capabilities: BTreeSet::from([capability_id.clone()]),
                tool: Some(CatalogToolDefinition {
                    id: tool_id,
                    model_visible_name: "course_search".to_owned(),
                    description: "Search the synthetic course catalog".to_owned(),
                    capability_id: capability_id.clone(),
                    claimed_input_schema_digest: input_schema.digest().clone(),
                    input_schema: Some(input_schema),
                }),
            }),
        },
        installation: PluginInstallationSnapshot {
            id: installation_id.clone(),
            tenant_id: tenant_id.clone(),
            user_id: user_id.clone(),
            package_id,
            package_version,
            package_digest: digest(0x44),
            component: InstalledComponentIdentity {
                id: component_id,
                version: component_version,
                digest: digest(0x45),
                execution_identity: execution_identity.clone(),
            },
            state: InstallationState::Enabled,
            revision: parsed!(InstallationRevision, "installation-revision:7"),
        },
        grant: CapabilityGrantSnapshot {
            snapshot_id: parsed!(GrantSnapshotId, "grant:synthetic-v3"),
            version: parsed!(GrantVersion, "grant-version:3"),
            tenant_id,
            user_id,
            installation_id,
            capability_id: capability_id.clone(),
            object_scope,
            confirmation_policy: ConfirmationPolicy::Allow,
            capability_manifest_digest,
            state: GrantState::Active,
        },
        policy: InvocationPolicySnapshot {
            snapshot_id: parsed!(PolicySnapshotId, "policy:synthetic-v9"),
            revision: parsed!(PolicyRevision, "policy-revision:9"),
            capability_id,
            capability_class: Some(CapabilityClass::PublicRead),
            admitted_execution_identity: Some(execution_identity),
            admitted_source_policy: Some(source_policy),
            emergency_blocked: false,
        },
    }
}

fn repository(
    fixture: &Fixture,
    catalog: bool,
    installation: bool,
    grant: bool,
    policy: bool,
) -> InMemoryInvocationAuthorityRepository {
    let catalog_records = if catalog {
        vec![(fixture.target.clone(), fixture.catalog.clone())]
    } else {
        Vec::new()
    };
    let installations = if installation {
        vec![fixture.installation.clone()]
    } else {
        Vec::new()
    };
    let grants = if grant {
        vec![fixture.grant.clone()]
    } else {
        Vec::new()
    };
    let current_grant_snapshot_ids = if grant {
        vec![fixture.grant.snapshot_id.clone()]
    } else {
        Vec::new()
    };
    let policies = if policy {
        vec![fixture.policy.clone()]
    } else {
        Vec::new()
    };
    InMemoryInvocationAuthorityRepository::try_new(
        catalog_records,
        installations,
        grants,
        current_grant_snapshot_ids,
        policies,
    )
    .expect("coherent synthetic authority repository")
}

fn static_application_fixture() -> Fixture {
    let mut fixture = fixture();
    let component = fixture
        .catalog
        .component
        .as_mut()
        .expect("fixture component");
    component.kind = ComponentKind::DeclarativeResourcePack;
    component.tool = None;
    fixture.policy.capability_class = Some(CapabilityClass::TenantPrivateWrite);
    fixture.policy.admitted_execution_identity = None;
    fixture
}

fn authorize_static_application(fixture: &Fixture) -> Result<(), InvocationRecheckError> {
    InvocationAuthorityService::new(repository(fixture, true, true, true, true))
        .authorize_static_application_use_case(
            &fixture.request.tenant_id,
            &fixture.request.user_id,
            &fixture.target,
            CapabilityClass::TenantPrivateWrite,
            InvocationConfirmation::Confirmed,
        )
}

fn candidate(fixture: &Fixture) -> InvocationAuthorityCandidate {
    InvocationAuthorityCandidate {
        target: fixture.target.clone(),
        catalog: Some(fixture.catalog.clone()),
        installation: Some(fixture.installation.clone()),
        grant: Some(fixture.grant.clone()),
        policy: fixture.policy.clone(),
    }
}

fn valid_projection(fixture: &Fixture) -> ToolProjectionSnapshot {
    InvocationResolver::resolve_projection(fixture.request.clone(), vec![candidate(fixture)])
        .expect("valid synthetic projection")
}

fn valid_call(projection: &ToolProjectionSnapshot) -> ProposedToolCall {
    let arguments = CanonicalArgumentValueV0::try_from(UnvalidatedArgumentValueV0::Object(vec![(
        "query".to_owned(),
        UnvalidatedArgumentValueV0::String("analysis".to_owned()),
    )]))
    .expect("valid fixture arguments");
    ProposedToolCall {
        provider_tool_call_id: parsed!(ProviderToolCallId, "provider-call:authority-01"),
        model_visible_name: projection.entries()[0].model_visible_name().to_owned(),
        dispatch_key: projection.entries()[0].dispatch_key().to_owned(),
        claimed_argument_digest: arguments.digest().clone(),
        arguments,
    }
}

fn current_state(fixture: &Fixture) -> CurrentDenyState {
    CurrentDenyState {
        tenant_id: fixture.request.tenant_id.clone(),
        user_id: fixture.request.user_id.clone(),
        catalog_revoked: false,
        installation: Some(fixture.installation.clone()),
        grant: fixture.grant.clone(),
        policy: fixture.policy.clone(),
    }
}

#[test]
fn projection_and_recheck_assemble_separate_carriers_under_one_verified_revision() {
    let fixture = fixture();
    let expected_projection = valid_projection(&fixture);
    let service = InvocationAuthorityService::new(repository(&fixture, true, true, true, true));
    let projection = service
        .resolve_projection(fixture.request.clone(), vec![fixture.target.clone()])
        .expect("assembled projection");
    assert_eq!(projection, expected_projection);

    let call = valid_call(&projection);
    let expected = authorize_call(&projection, current_state(&fixture), call.clone())
        .expect("direct adopted recheck");
    let actual = service
        .recheck_invocation(&projection, call)
        .expect("assembled adopted recheck");
    assert_eq!(actual, expected);
}

#[test]
fn static_application_authority_accepts_only_a_non_tool_resource_carrier() {
    let static_fixture = static_application_fixture();
    authorize_static_application(&static_fixture).expect("static application authority");

    let mut tool_projected = static_fixture.clone();
    tool_projected
        .catalog
        .component
        .as_mut()
        .expect("fixture component")
        .tool = fixture().catalog.component.expect("tool component").tool;
    assert_eq!(
        authorize_static_application(&tool_projected),
        Err(InvocationRecheckError::Authorization(
            InvocationAuthorizationError::AuthorityConflict
        ))
    );

    let mut execution_admitted = static_fixture.clone();
    execution_admitted.policy.admitted_execution_identity = Some(
        execution_admitted
            .installation
            .component
            .execution_identity
            .clone(),
    );
    assert_eq!(
        authorize_static_application(&execution_admitted),
        Err(InvocationRecheckError::Authorization(
            InvocationAuthorizationError::AuthorityConflict
        ))
    );

    let service =
        InvocationAuthorityService::new(repository(&static_fixture, true, true, true, true));
    assert_eq!(
        service.authorize_static_application_use_case(
            &static_fixture.request.tenant_id,
            &static_fixture.request.user_id,
            &static_fixture.target,
            CapabilityClass::TenantPrivateRead,
            InvocationConfirmation::Confirmed,
        ),
        Err(InvocationRecheckError::Authorization(
            InvocationAuthorizationError::AuthorityConflict
        ))
    );
}

#[test]
fn static_application_ask_grant_requires_confirmation() {
    let mut fixture = static_application_fixture();
    fixture.grant.confirmation_policy = ConfirmationPolicy::Ask;
    let service = InvocationAuthorityService::new(repository(&fixture, true, true, true, true));
    assert_eq!(
        service.authorize_static_application_use_case(
            &fixture.request.tenant_id,
            &fixture.request.user_id,
            &fixture.target,
            CapabilityClass::TenantPrivateWrite,
            InvocationConfirmation::NotConfirmed,
        ),
        Err(InvocationRecheckError::Authorization(
            InvocationAuthorizationError::ConfirmationRequired
        ))
    );
    service
        .authorize_static_application_use_case(
            &fixture.request.tenant_id,
            &fixture.request.user_id,
            &fixture.target,
            CapabilityClass::TenantPrivateWrite,
            InvocationConfirmation::Confirmed,
        )
        .expect("confirmed Ask grant authorizes");
}

#[test]
fn static_application_denial_precedes_final_transaction_verification() {
    let mut revoked = static_application_fixture();
    revoked.grant.state = GrantState::Revoked;
    let revoked_repository = repository(&revoked, true, true, true, true);
    revoked_repository.fail_next_precondition_for_testing();
    let service = InvocationAuthorityService::new(revoked_repository);
    assert_eq!(
        service.authorize_static_application_use_case(
            &revoked.request.tenant_id,
            &revoked.request.user_id,
            &revoked.target,
            CapabilityClass::TenantPrivateWrite,
            InvocationConfirmation::Confirmed,
        ),
        Err(InvocationRecheckError::Authorization(
            InvocationAuthorizationError::GrantRevoked
        ))
    );

    let fixture = static_application_fixture();
    let repository = repository(&fixture, true, true, true, true);
    repository.fail_next_precondition_for_testing();
    let service = InvocationAuthorityService::new(repository);
    assert_eq!(
        service.authorize_static_application_use_case(
            &fixture.request.tenant_id,
            &fixture.request.user_id,
            &fixture.target,
            CapabilityClass::TenantPrivateWrite,
            InvocationConfirmation::Confirmed,
        ),
        Err(InvocationRecheckError::Repository(
            AuthorityRepositoryError::TransactionConflict
        ))
    );
}

#[test]
fn projection_missing_catalog_installation_or_grant_keeps_exact_resolver_denial() {
    let fixture = fixture();
    for (catalog, installation, grant, expected) in [
        (false, true, true, ProjectionResolutionError::PackageMissing),
        (
            true,
            false,
            true,
            ProjectionResolutionError::InstallationMissing,
        ),
        (
            true,
            true,
            false,
            ProjectionResolutionError::CapabilityNotGranted,
        ),
    ] {
        let service = InvocationAuthorityService::new(repository(
            &fixture,
            catalog,
            installation,
            grant,
            true,
        ));
        assert_eq!(
            service.resolve_projection(fixture.request.clone(), vec![fixture.target.clone()]),
            Err(ProjectionAssemblyError::Resolution(expected))
        );
    }

    let service = InvocationAuthorityService::new(repository(&fixture, true, true, true, false));
    assert_eq!(
        service.resolve_projection(fixture.request.clone(), vec![fixture.target.clone()]),
        Err(ProjectionAssemblyError::Repository(
            AuthorityRepositoryError::PolicyMissing
        ))
    );
}

#[test]
fn post_success_transaction_conflict_returns_no_projection_or_authorized_invocation() {
    let fixture = fixture();
    let projection_repository = repository(&fixture, true, true, true, true);
    projection_repository.fail_next_precondition_for_testing();
    let projection_service = InvocationAuthorityService::new(projection_repository);
    assert_eq!(
        projection_service
            .resolve_projection(fixture.request.clone(), vec![fixture.target.clone()]),
        Err(ProjectionAssemblyError::Repository(
            AuthorityRepositoryError::TransactionConflict
        ))
    );

    let projection = valid_projection(&fixture);
    let call_repository = repository(&fixture, true, true, true, true);
    call_repository.fail_next_precondition_for_testing();
    let call_service = InvocationAuthorityService::new(call_repository);
    assert_eq!(
        call_service.recheck_invocation(&projection, valid_call(&projection)),
        Err(InvocationRecheckError::Repository(
            AuthorityRepositoryError::TransactionConflict
        ))
    );
}

#[test]
fn resolver_or_recheck_denial_precedes_pending_verification_conflict() {
    let fixture = fixture();
    let projection_repository = repository(&fixture, false, true, true, true);
    projection_repository.fail_next_precondition_for_testing();
    let projection_service = InvocationAuthorityService::new(projection_repository);
    assert_eq!(
        projection_service
            .resolve_projection(fixture.request.clone(), vec![fixture.target.clone()]),
        Err(ProjectionAssemblyError::Resolution(
            ProjectionResolutionError::PackageMissing
        ))
    );

    let projection = valid_projection(&fixture);
    let mut emergency = fixture.clone();
    emergency.policy.emergency_blocked = true;
    let recheck_repository = repository(&emergency, true, true, true, true);
    recheck_repository.fail_next_precondition_for_testing();
    let recheck_service = InvocationAuthorityService::new(recheck_repository);
    assert_eq!(
        recheck_service.recheck_invocation(&projection, valid_call(&projection)),
        Err(InvocationRecheckError::Authorization(
            InvocationAuthorizationError::EmergencyBlocked
        ))
    );
}

struct FailingRepository;
struct UnreachableTransaction;

impl InvocationAuthorityReadTransaction for UnreachableTransaction {
    fn revision(&self) -> &ustc_campus_agent_core::market::authority::AuthorityReadRevision {
        unreachable!("begin_read always fails")
    }

    fn load_catalog_for_target(
        &self,
        _: &InvocationTarget,
    ) -> Result<Option<CatalogPackageRevision>, AuthorityRepositoryError> {
        unreachable!("begin_read always fails")
    }

    fn load_installation(
        &self,
        _: &InstallationId,
    ) -> Result<Option<PluginInstallationSnapshot>, AuthorityRepositoryError> {
        unreachable!("begin_read always fails")
    }

    fn load_current_grant(
        &self,
        _: &TenantId,
        _: &UserId,
        _: &InvocationTarget,
    ) -> Result<Option<CapabilityGrantSnapshot>, AuthorityRepositoryError> {
        unreachable!("begin_read always fails")
    }

    fn load_exact_grant(
        &self,
        _: &GrantSnapshotId,
    ) -> Result<Option<CapabilityGrantSnapshot>, AuthorityRepositoryError> {
        unreachable!("begin_read always fails")
    }

    fn load_policy(
        &self,
        _: &CapabilityId,
    ) -> Result<Option<InvocationPolicySnapshot>, AuthorityRepositoryError> {
        unreachable!("begin_read always fails")
    }

    fn verify_precondition(self) -> Result<(), AuthorityRepositoryError> {
        unreachable!("begin_read always fails")
    }
}

impl InvocationAuthorityRepository for FailingRepository {
    type ReadTransaction<'a> = UnreachableTransaction;

    fn begin_read(&self) -> Result<Self::ReadTransaction<'_>, AuthorityRepositoryError> {
        Err(AuthorityRepositoryError::TransactionConflict)
    }
}

#[test]
fn invalid_name_or_dispatch_preflight_precedes_repository_failure() {
    let fixture = fixture();
    let projection = valid_projection(&fixture);
    let service = InvocationAuthorityService::new(FailingRepository);

    let mut invalid = valid_call(&projection);
    invalid.model_visible_name.clear();
    assert_eq!(
        service.recheck_invocation(&projection, invalid),
        Err(InvocationRecheckError::Authorization(
            InvocationAuthorizationError::InvalidCall
        ))
    );

    let mut missing = valid_call(&projection);
    missing.model_visible_name = "not_projected".to_owned();
    assert_eq!(
        service.recheck_invocation(&projection, missing),
        Err(InvocationRecheckError::Authorization(
            InvocationAuthorizationError::ToolNotProjected
        ))
    );

    let mut wrong_dispatch = valid_call(&projection);
    wrong_dispatch.dispatch_key = "dispatch:wrong".to_owned();
    assert_eq!(
        service.recheck_invocation(&projection, wrong_dispatch),
        Err(InvocationRecheckError::Authorization(
            InvocationAuthorizationError::DispatchIdentityMismatch
        ))
    );
}

#[test]
fn disable_revoke_stale_expire_and_emergency_block_deny_current_use() {
    let fixture = fixture();
    let projection = valid_projection(&fixture);

    for (state, expected) in [
        (
            InstallationState::Disabled,
            InvocationAuthorizationError::InstallationDisabled,
        ),
        (
            InstallationState::Revoked,
            InvocationAuthorizationError::InstallationRevoked,
        ),
    ] {
        let mut narrowed = fixture.clone();
        narrowed.installation.state = state;
        let service =
            InvocationAuthorityService::new(repository(&narrowed, true, true, true, true));
        assert_eq!(
            service.recheck_invocation(&projection, valid_call(&projection)),
            Err(InvocationRecheckError::Authorization(expected))
        );
    }

    for (state, expected) in [
        (GrantState::Stale, InvocationAuthorizationError::GrantStale),
        (
            GrantState::Expired,
            InvocationAuthorizationError::GrantExpired,
        ),
        (
            GrantState::Revoked,
            InvocationAuthorizationError::GrantRevoked,
        ),
    ] {
        let mut narrowed = fixture.clone();
        narrowed.grant.state = state;
        let service =
            InvocationAuthorityService::new(repository(&narrowed, true, true, true, true));
        assert_eq!(
            service.recheck_invocation(&projection, valid_call(&projection)),
            Err(InvocationRecheckError::Authorization(expected))
        );
    }

    let mut catalog_revoked = fixture.clone();
    catalog_revoked.catalog.revoked = true;
    let service =
        InvocationAuthorityService::new(repository(&catalog_revoked, true, true, true, true));
    assert_eq!(
        service.recheck_invocation(&projection, valid_call(&projection)),
        Err(InvocationRecheckError::Authorization(
            InvocationAuthorizationError::CatalogRevoked
        ))
    );

    let mut emergency = fixture.clone();
    emergency.policy.emergency_blocked = true;
    let service = InvocationAuthorityService::new(repository(&emergency, true, true, true, true));
    assert_eq!(
        service.recheck_invocation(&projection, valid_call(&projection)),
        Err(InvocationRecheckError::Authorization(
            InvocationAuthorizationError::EmergencyBlocked
        ))
    );
}

#[test]
fn historical_projection_is_immutable_after_current_authority_narrows() {
    let fixture = fixture();
    let service = InvocationAuthorityService::new(repository(&fixture, true, true, true, true));
    let projection = service
        .resolve_projection(fixture.request.clone(), vec![fixture.target.clone()])
        .expect("valid assembled projection");
    let frozen = projection.clone();
    let mut repository = service.into_repository();
    let mut disabled = fixture.installation.clone();
    disabled.state = InstallationState::Disabled;
    repository
        .replace_installation_for_testing(disabled.id.clone(), Some(disabled))
        .expect("fixture authority mutation");
    let service = InvocationAuthorityService::new(repository);
    assert_eq!(projection, frozen);
    assert_eq!(
        service.recheck_invocation(&projection, valid_call(&projection)),
        Err(InvocationRecheckError::Authorization(
            InvocationAuthorizationError::InstallationDisabled
        ))
    );
    assert_eq!(projection, frozen);
}

#[test]
fn dispatch_argument_version_scope_and_schema_errors_remain_adopted_recheck_errors() {
    let fixture = fixture();
    let projection = valid_projection(&fixture);

    let mut bad_digest = valid_call(&projection);
    bad_digest.claimed_argument_digest = digest(0xaa);
    let service = InvocationAuthorityService::new(repository(&fixture, true, true, true, true));
    assert_eq!(
        service.recheck_invocation(&projection, bad_digest),
        Err(InvocationRecheckError::Authorization(
            InvocationAuthorizationError::ArgumentDigestMismatch
        ))
    );

    let invalid_arguments =
        CanonicalArgumentValueV0::try_from(UnvalidatedArgumentValueV0::Object(vec![(
            "query".to_owned(),
            UnvalidatedArgumentValueV0::Boolean(true),
        )]))
        .expect("canonical but schema-invalid arguments");
    let mut bad_schema = valid_call(&projection);
    bad_schema.claimed_argument_digest = invalid_arguments.digest().clone();
    bad_schema.arguments = invalid_arguments;
    let service = InvocationAuthorityService::new(repository(&fixture, true, true, true, true));
    assert_eq!(
        service.recheck_invocation(&projection, bad_schema),
        Err(InvocationRecheckError::Authorization(
            InvocationAuthorizationError::ArgumentsInvalid
        ))
    );

    let mut installation_mismatch = fixture.clone();
    installation_mismatch.installation.revision =
        parsed!(InstallationRevision, "installation-revision:8");
    let service =
        InvocationAuthorityService::new(repository(&installation_mismatch, true, true, true, true));
    assert_eq!(
        service.recheck_invocation(&projection, valid_call(&projection)),
        Err(InvocationRecheckError::Authorization(
            InvocationAuthorizationError::InstallationRevisionMismatch
        ))
    );

    let mut version_mismatch = fixture.clone();
    version_mismatch.grant.version = parsed!(GrantVersion, "grant-version:4");
    let service =
        InvocationAuthorityService::new(repository(&version_mismatch, true, true, true, true));
    assert_eq!(
        service.recheck_invocation(&projection, valid_call(&projection)),
        Err(InvocationRecheckError::Authorization(
            InvocationAuthorizationError::GrantVersionMismatch
        ))
    );

    let mut scope_mismatch = fixture.clone();
    scope_mismatch.grant.object_scope = parsed!(ObjectScope, "scope:other");
    let service = InvocationAuthorityService::new(
        InMemoryInvocationAuthorityRepository::try_new(
            vec![(
                scope_mismatch.target.clone(),
                scope_mismatch.catalog.clone(),
            )],
            vec![scope_mismatch.installation.clone()],
            vec![scope_mismatch.grant.clone()],
            Vec::new(),
            vec![scope_mismatch.policy.clone()],
        )
        .expect("coherent exact-grant-only current repository"),
    );
    assert_eq!(
        service.recheck_invocation(&projection, valid_call(&projection)),
        Err(InvocationRecheckError::Authorization(
            InvocationAuthorizationError::GrantScopeMismatch
        ))
    );
}
