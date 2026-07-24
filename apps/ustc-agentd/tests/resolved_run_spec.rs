use std::collections::BTreeSet;
use ustc_campus_agent_core::invocation::*;
use ustc_campus_agent_runtime::{AgentRun, RUN_SPEC_SCHEMA_VERSION, RunBudgets, RunSpec};

#[path = "../../../crates/platform-core/tests/support/invocation_fixture.rs"]
mod invocation_fixture;

use invocation_fixture::{
    FixtureApi, FixtureExpected, FixtureExpectedName, FixturePrecedence, FixtureRecipe,
    InvocationFixture,
};

macro_rules! parsed {
    ($kind:ty, $value:expr) => {{
        match <$kind>::parse($value) {
            Ok(value) => value,
            Err(error) => panic!("synthetic fixture must parse: {error}"),
        }
    }};
}

fn digest(byte: char) -> Sha256Digest {
    parsed!(
        Sha256Digest,
        format!("sha256:{}", byte.to_string().repeat(64))
    )
}

fn authority() -> (ToolProjectionRequest, InvocationAuthorityCandidate) {
    let tenant = parsed!(TenantId, "tenant:proof");
    let user = parsed!(UserId, "user:proof");
    let installation_id = parsed!(InstallationId, "installation:proof");
    let package_id = parsed!(PackageId, "synthetic.proof");
    let version = parsed!(PackageVersion, "1.0.0");
    let component_id = parsed!(ComponentId, "component:proof");
    let component_version = parsed!(ComponentVersion, "component-version:1");
    let execution = parsed!(ExecutionIdentity, "native:proof");
    let tool_id = parsed!(ToolId, "tool:proof");
    let capability = parsed!(CapabilityId, "campus.public_rules.read");
    let scope = parsed!(ObjectScope, "scope:public");
    let source = SourcePolicyIdentity {
        id: parsed!(SourcePolicyId, "source-policy:proof"),
        digest: digest('5'),
    };
    let schema = match ValidatedToolInputSchemaV0::try_from(UnvalidatedToolInputSchemaV0 {
        dialect: "tool-input-schema/v0".to_owned(),
        root: UnvalidatedSchemaNodeV0::Object {
            properties: vec![(
                "query".to_owned(),
                UnvalidatedSchemaNodeV0::String { enum_values: None },
            )],
            required: vec!["query".to_owned()],
        },
    }) {
        Ok(schema) => schema,
        Err(error) => panic!("proof schema must validate: {error}"),
    };
    let installed_component = InstalledComponentIdentity {
        id: component_id.clone(),
        version: component_version.clone(),
        digest: digest('2'),
        execution_identity: execution.clone(),
    };
    let installation = PluginInstallationSnapshot {
        id: installation_id.clone(),
        tenant_id: tenant.clone(),
        user_id: user.clone(),
        package_id: package_id.clone(),
        package_version: version.clone(),
        package_digest: digest('1'),
        component: installed_component,
        state: InstallationState::Enabled,
        revision: parsed!(InstallationRevision, "installation-revision:1"),
    };
    let grant = CapabilityGrantSnapshot {
        snapshot_id: parsed!(GrantSnapshotId, "grant:proof"),
        version: parsed!(GrantVersion, "grant-version:1"),
        tenant_id: tenant.clone(),
        user_id: user.clone(),
        installation_id: installation_id.clone(),
        capability_id: capability.clone(),
        object_scope: scope.clone(),
        confirmation_policy: ConfirmationPolicy::Allow,
        capability_manifest_digest: digest('3'),
        state: GrantState::Active,
    };
    let policy = InvocationPolicySnapshot {
        snapshot_id: parsed!(PolicySnapshotId, "policy:proof"),
        revision: parsed!(PolicyRevision, "policy-revision:1"),
        capability_id: capability.clone(),
        capability_class: Some(CapabilityClass::PublicRead),
        admitted_execution_identity: Some(execution.clone()),
        admitted_source_policy: Some(source.clone()),
        emergency_blocked: false,
    };
    let target = InvocationTarget {
        installation_id,
        package_id: package_id.clone(),
        package_version: version.clone(),
        component_id: component_id.clone(),
        tool_id: tool_id.clone(),
        capability_id: capability.clone(),
        object_scope: scope,
    };
    let component = CatalogComponentRevision {
        id: component_id,
        kind: ComponentKind::NativeRustComponent,
        version: component_version,
        digest: digest('2'),
        execution_identity: execution,
        declared_capabilities: BTreeSet::from([capability.clone()]),
        tool: Some(CatalogToolDefinition {
            id: tool_id,
            model_visible_name: "proof_tool".to_owned(),
            description: "Synthetic proof tool".to_owned(),
            capability_id: capability,
            claimed_input_schema_digest: schema.digest().clone(),
            input_schema: Some(schema),
        }),
    };
    (
        ToolProjectionRequest {
            tenant_id: tenant,
            user_id: user,
            run_id: parsed!(RunId, "run:proof"),
            turn_id: parsed!(TurnId, "turn:proof"),
            activation_allowlist: None,
        },
        InvocationAuthorityCandidate {
            target,
            catalog: Some(CatalogPackageRevision {
                catalog_revision: parsed!(CatalogRevision, "catalog:proof"),
                package_id,
                package_version: version,
                package_digest: digest('1'),
                runnable: true,
                revoked: false,
                capability_manifest_digest: digest('3'),
                source_policy: Some(source),
                component: Some(component),
            }),
            installation: Some(installation),
            grant: Some(grant),
            policy,
        },
    )
}

fn create_run_spec_only_from_resolution(
    resolution: Result<ToolProjectionSnapshot, ProjectionResolutionError>,
) -> Option<RunSpec> {
    resolution.ok().map(|projection| {
        let entry = &projection.entries()[0];
        RunSpec {
            schema_version: RUN_SPEC_SCHEMA_VERSION.to_owned(),
            run_id: projection.run_id().as_str().to_owned(),
            tenant_id: entry.tenant_id().as_str().to_owned(),
            installation_id: entry.installation_id().as_str().to_owned(),
            package_id: entry.package_id().as_str().to_owned(),
            package_version: entry.package_version().as_str(),
            component_id: entry.component_id().as_str().to_owned(),
            provider_profile_id: "provider:synthetic-proof".to_owned(),
            grant_snapshot_id: entry.grant_snapshot_id().as_str().to_owned(),
            tool_schema_set_digest: projection.tool_schema_set_digest().as_str().to_owned(),
            budgets: RunBudgets {
                max_turns: 1,
                max_tool_calls: 1,
                max_input_tokens: 1,
                max_output_tokens: 1,
                max_cost_microunits: 1,
                max_retries: 1,
                max_elapsed_ms: 1,
            },
        }
    })
}

fn create_run_only_from_resolution(
    resolution: Result<ToolProjectionSnapshot, ProjectionResolutionError>,
) -> Option<Result<AgentRun, ustc_campus_agent_runtime::RuntimeError>> {
    create_run_spec_only_from_resolution(resolution).map(AgentRun::new)
}

#[test]
fn successful_resolution_is_the_only_run_spec_input() {
    let (request, candidate) = authority();
    let resolution = InvocationResolver::resolve_projection(request, vec![candidate]);
    let Some(run) = create_run_only_from_resolution(resolution) else {
        panic!("successful resolution must yield a run attempt")
    };
    let Ok(run) = run else {
        panic!("resolved RunSpec must validate")
    };
    assert_eq!(run.spec().package_id, "synthetic.proof");
    assert!(run.spec().tool_schema_set_digest.starts_with("sha256:"));
}

#[test]
fn denied_resolution_cannot_construct_run_spec_or_run() {
    let (request, mut candidate) = authority();
    candidate.policy.emergency_blocked = true;
    let denied = InvocationResolver::resolve_projection(request, vec![candidate]);
    assert_eq!(denied, Err(ProjectionResolutionError::EmergencyBlocked));
    assert!(create_run_only_from_resolution(denied).is_none());
}

#[test]
fn fixture_run_spec_mapping_constructs_run_and_denial_constructs_neither() {
    let fixture = match serde_json::from_str::<InvocationFixture>(include_str!(
        "../../../crates/platform-core/tests/fixtures/invocation-resolution/valid-synthetic-v0.json"
    )) {
        Ok(value) => value,
        Err(error) => panic!("valid synthetic fixture must parse: {error}"),
    };
    assert_eq!(fixture.schema_version, "invocation-resolution-fixture/v0");
    assert!(fixture.synthetic);
    assert_eq!(fixture.fixture, "valid-synthetic-v0.json");
    let cases = fixture
        .cases
        .iter()
        .filter(|case| case.api == FixtureApi::RunSpecMapping)
        .collect::<Vec<_>>();
    let [case] = cases.as_slice() else {
        panic!("fixture must contain exactly one run-spec mapping case")
    };
    assert_eq!(case.name, "valid-run-spec-mapping");
    assert_eq!(case.api.as_str(), "run_spec_mapping");
    assert_eq!(case.recipe, FixtureRecipe::ProjectionValidAuthority);
    assert_eq!(case.recipe.as_str(), "projection=valid_authority");
    assert_eq!(
        case.expected,
        FixtureExpected::Named(FixtureExpectedName::SuccessOnly)
    );
    assert_eq!(FixtureExpectedName::SuccessOnly.as_str(), "success-only");
    assert_eq!(case.precedence, FixturePrecedence::DenialProducesNoRun);
    assert_eq!(case.precedence.as_str(), "denial-produces-no-run");

    let (request, candidate) = authority();
    let resolution = InvocationResolver::resolve_projection(request, vec![candidate]);
    let Some(spec) = create_run_spec_only_from_resolution(resolution) else {
        panic!("fixture success must construct RunSpec")
    };
    let run = match AgentRun::new(spec) {
        Ok(run) => run,
        Err(error) => panic!("fixture RunSpec must construct AgentRun: {error}"),
    };
    assert_eq!(run.spec().package_id, "synthetic.proof");

    let (request, mut denied_candidate) = authority();
    denied_candidate.policy.emergency_blocked = true;
    let denied = InvocationResolver::resolve_projection(request, vec![denied_candidate]);
    assert_eq!(denied, Err(ProjectionResolutionError::EmergencyBlocked));
    assert!(create_run_spec_only_from_resolution(denied.clone()).is_none());
    assert!(create_run_only_from_resolution(denied).is_none());
}
