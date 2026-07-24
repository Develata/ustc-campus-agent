use ustc_campus_agent_core::invocation::*;
use ustc_campus_agent_runtime::{AgentRun, RUN_SPEC_SCHEMA_VERSION, RunBudgets, RunSpec};

#[path = "../../../crates/platform-core/tests/support/invocation_fixture.rs"]
mod invocation_fixture;
mod support;

use invocation_fixture::{
    FixtureApi, FixtureExpected, FixtureExpectedName, FixturePrecedence, FixtureRecipe,
    InvocationFixture,
};
use support::proof_authority;

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
    let (request, candidate) = proof_authority();
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
    let (request, mut candidate) = proof_authority();
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

    let (request, candidate) = proof_authority();
    let resolution = InvocationResolver::resolve_projection(request, vec![candidate]);
    let Some(spec) = create_run_spec_only_from_resolution(resolution) else {
        panic!("fixture success must construct RunSpec")
    };
    let run = match AgentRun::new(spec) {
        Ok(run) => run,
        Err(error) => panic!("fixture RunSpec must construct AgentRun: {error}"),
    };
    assert_eq!(run.spec().package_id, "synthetic.proof");

    let (request, mut denied_candidate) = proof_authority();
    denied_candidate.policy.emergency_blocked = true;
    let denied = InvocationResolver::resolve_projection(request, vec![denied_candidate]);
    assert_eq!(denied, Err(ProjectionResolutionError::EmergencyBlocked));
    assert!(create_run_spec_only_from_resolution(denied.clone()).is_none());
    assert!(create_run_only_from_resolution(denied).is_none());
}
