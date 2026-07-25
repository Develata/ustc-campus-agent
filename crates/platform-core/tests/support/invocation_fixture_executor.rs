use super::invocation_fixture::{
    FixtureApi, FixtureExpected, FixtureExpectedName, FixturePrecedence, FixtureRecipe,
    InvocationFixtureCase,
};
use super::*;

pub fn execute_fixture_case(case: &InvocationFixtureCase) {
    if let Err(error) = verify_fixture_case(case) {
        panic!("fixture case {} failed: {error}", case.name);
    }
}

pub fn verify_fixture_case(case: &InvocationFixtureCase) -> Result<(), String> {
    let actual_precedence = precedence_for(case.api, case.recipe).ok_or_else(|| {
        format!(
            "API {} does not accept recipe {}",
            case.api.as_str(),
            case.recipe.as_str()
        )
    })?;
    if actual_precedence != case.precedence {
        return Err(format!(
            "precedence mismatch: actual={} fixture={}",
            actual_precedence.as_str(),
            case.precedence.as_str()
        ));
    }

    match case.api {
        FixtureApi::SchemaConstructor => verify_schema(case.recipe, &case.expected),
        FixtureApi::ArgumentConstructor => verify_arguments(case.recipe, &case.expected),
        FixtureApi::ResolveProjection => verify_projection(case.recipe, &case.expected),
        FixtureApi::AuthorizeCall => verify_call(case.recipe, &case.expected),
        FixtureApi::RunSpecMapping => verify_run_spec_eligibility(case.recipe, &case.expected),
    }
}

fn precedence_for(api: FixtureApi, recipe: FixtureRecipe) -> Option<FixturePrecedence> {
    use FixtureApi::{
        ArgumentConstructor, AuthorizeCall, ResolveProjection, RunSpecMapping, SchemaConstructor,
    };
    use FixturePrecedence::*;
    use FixtureRecipe::*;

    match (api, recipe) {
        (SchemaConstructor, GoldenSchema | GoldenSchemaReversed)
        | (
            ArgumentConstructor,
            GoldenArguments
            | GoldenArgumentsReversed
            | ArgumentNumberNormalization
            | ArgumentTypeDistinction,
        )
        | (
            ResolveProjection,
            ValidAuthority
            | TurnTwo
            | DefinitionNameChanged
            | DefinitionDescriptionChanged
            | DefinitionSchemaBoolean,
        ) => Some(NotApplicable),
        (SchemaConstructor, SchemaDialectOther | SchemaMalformedMatrix | SchemaLimitMatrix)
        | (
            ArgumentConstructor,
            ArgumentDuplicateKey | ArgumentBadKey | ArgumentLimitMatrix | ArgumentNumberOverflow,
        ) => Some(ConstructorOnly),
        (ResolveProjection, InstallationTenantOther) => Some(ProjectionGroup3CanonicalLeftmost),
        (ResolveProjection, GrantUserOtherAndStale) => Some(ProjectionGroup3BeforeGroup9),
        (
            ResolveProjection,
            CatalogMissing
            | CatalogNotRunnable
            | CatalogPackageVersionNine
            | InstallationPackageDigestNine
            | CatalogRevoked,
        ) => Some(ProjectionGroup4CanonicalLeftmost),
        (
            ResolveProjection,
            InstallationMissing | InstallationDisabled | InstallationRevoked | InstallationIdOther,
        ) => Some(ProjectionGroup5CanonicalLeftmost),
        (
            ResolveProjection,
            CatalogComponentMissing
            | InstallationComponentDigestNine
            | ExecutionIdentityUnknown
            | ExecutionIdentityOther,
        ) => Some(ProjectionGroup6CanonicalLeftmost),
        (ResolveProjection, ToolMissing | ToolIdentityOther) => {
            Some(ProjectionGroup7CanonicalLeftmost)
        }
        (
            ResolveProjection,
            CapabilityUnknown | CapabilitiesEmpty | GrantManifestDigestNine | GrantMissing,
        ) => Some(ProjectionGroup8CanonicalLeftmost),
        (
            ResolveProjection,
            GrantStale | GrantExpired | GrantRevoked | GrantInstallationOther | GrantScopeOther,
        ) => Some(ProjectionGroup9CanonicalLeftmost),
        (ResolveProjection, SourcePolicyMissing | AdmittedSourcePolicyMissing) => {
            Some(ProjectionGroup10CanonicalLeftmost)
        }
        (ResolveProjection, EmergencyAndCatalogMissing) => Some(ProjectionGroup2BeforeGroup4),
        (ResolveProjection, TwoTargetInstallationConflict | OptionalLayerTransitivity) => {
            Some(ProjectionGroup2BeforeTargetLocal)
        }
        (ResolveProjection, OptionalLayerAbsence) => Some(ProjectionTargetLocalByGroup),
        (ResolveProjection, DefinitionNameCollision) => Some(ProjectionGroup12),
        (ResolveProjection, CompleteProjectionErrorMatrix) => Some(ProjectionGroups1Through12),
        (ResolveProjection, ProjectionGroupMajorForward | ProjectionGroupMajorReverse) => {
            Some(ProjectionGroup4BeforeGroup5)
        }
        (ResolveProjection, ProjectionGroupMajorThree) => {
            Some(ProjectionGroup4BeforeGroup5BeforeGroup9)
        }
        (AuthorizeCall, CallNameNotProjected) => Some(CallGroup2BeforeDenyAndArguments),
        (AuthorizeCall, CallDispatchWrongLiteral) => Some(CallGroup3BeforeDenyAndArguments),
        (AuthorizeCall, CallNoFallback) => Some(ExactFrozenEntryOnly),
        (AuthorizeCall, GrantTenantOtherAndStale) => Some(CallGroup5BeforeGroup8),
        (AuthorizeCall, CompleteCallErrorMatrix) => Some(CallGroups1Through10),
        (AuthorizeCall, CallNameBeforeDeny) => Some(ProjectionGroup2BeforeGroup4),
        (AuthorizeCall, CallDispatchBeforeDeny) => Some(CallGroup3BeforeGroup4),
        (AuthorizeCall, CallDispatchBeforeArguments) => Some(ProjectionGroup3BeforeGroup9),
        (AuthorizeCall, PostEmergency) => Some(CallGroup4),
        (AuthorizeCall, PostCatalogRevoked) => Some(CallGroup6),
        (AuthorizeCall, InstallationDisabled) => Some(CallGroup7),
        (AuthorizeCall, GrantStale) => Some(CallGroup8),
        (AuthorizeCall, FrozenDisabledThenEnabled) => Some(ProjectionDenialIsTerminal),
        (RunSpecMapping, ProjectionValidAuthority) => Some(DenialProducesNoRun),
        _ => None,
    }
}

fn expected_named(expected: &FixtureExpected, actual: &str) -> Result<(), String> {
    match expected {
        FixtureExpected::Named(name) if name.as_str() == actual => Ok(()),
        other => Err(format!("actual={actual}, fixture={other:?}")),
    }
}

fn verify_schema(recipe: FixtureRecipe, expected: &FixtureExpected) -> Result<(), String> {
    use FixtureRecipe::*;
    let schema = |root| UnvalidatedToolInputSchemaV0 {
        dialect: "tool-input-schema/v0".to_owned(),
        root,
    };
    match recipe {
        GoldenSchema => {
            let value = ValidatedToolInputSchemaV0::try_from(golden_schema(false))
                .map_err(|error| error.to_string())?;
            match expected {
                FixtureExpected::CanonicalGolden { bytes, digest } => {
                    if hex::encode(value.canonical_bytes()) == *bytes
                        && value.digest().as_str() == digest
                    {
                        Ok(())
                    } else {
                        Err("schema canonical bytes/digest differ from fixture golden".to_owned())
                    }
                }
                other => Err(format!(
                    "schema golden requires canonical golden expected, got {other:?}"
                )),
            }
        }
        GoldenSchemaReversed => {
            let baseline = ValidatedToolInputSchemaV0::try_from(golden_schema(false))
                .map_err(|error| error.to_string())?;
            let reversed = ValidatedToolInputSchemaV0::try_from(golden_schema(true))
                .map_err(|error| error.to_string())?;
            match expected {
                FixtureExpected::EqualTo { case }
                    if case == "schema-golden" && reversed == baseline =>
                {
                    Ok(())
                }
                other => Err(format!(
                    "schema permutation does not match fixture expectation {other:?}"
                )),
            }
        }
        SchemaDialectOther => {
            let actual = ValidatedToolInputSchemaV0::try_from(UnvalidatedToolInputSchemaV0 {
                dialect: "other".to_owned(),
                root: UnvalidatedSchemaNodeV0::Object {
                    properties: vec![],
                    required: vec![],
                },
            });
            verify_schema_error(actual, expected)
        }
        SchemaMalformedMatrix => {
            let duplicate = schema(UnvalidatedSchemaNodeV0::Object {
                properties: vec![
                    ("x".to_owned(), UnvalidatedSchemaNodeV0::Integer),
                    ("x".to_owned(), UnvalidatedSchemaNodeV0::Integer),
                ],
                required: vec![],
            });
            let missing = schema(UnvalidatedSchemaNodeV0::Object {
                properties: vec![],
                required: vec!["x".to_owned()],
            });
            for actual in [
                ValidatedToolInputSchemaV0::try_from(duplicate),
                ValidatedToolInputSchemaV0::try_from(missing),
            ] {
                if actual != Err(SchemaConstructionError::SchemaMalformed) {
                    return Err(format!("malformed schema recipe produced {actual:?}"));
                }
            }
            expected_named(expected, "SchemaMalformed")
        }
        SchemaLimitMatrix => {
            let depth = schema(UnvalidatedSchemaNodeV0::Object {
                properties: vec![("x".to_owned(), nested_schema(8))],
                required: vec![],
            });
            let nodes = schema(schema_with_total_nodes(257));
            let properties = schema(UnvalidatedSchemaNodeV0::Object {
                properties: (0..65)
                    .map(|index| (format!("p{index}"), UnvalidatedSchemaNodeV0::Integer))
                    .collect(),
                required: vec![],
            });
            let enum_values = schema(UnvalidatedSchemaNodeV0::Object {
                properties: vec![(
                    "x".to_owned(),
                    UnvalidatedSchemaNodeV0::String {
                        enum_values: Some((0..65).map(|index| format!("v{index}")).collect()),
                    },
                )],
                required: vec![],
            });
            let huge_enum = (0..64)
                .map(|index| format!("{index:02}{}", "x".repeat(254)))
                .collect::<Vec<_>>();
            let bytes = schema(UnvalidatedSchemaNodeV0::Object {
                properties: (0..64)
                    .map(|index| {
                        (
                            format!("p{index}"),
                            UnvalidatedSchemaNodeV0::String {
                                enum_values: Some(huge_enum.clone()),
                            },
                        )
                    })
                    .collect(),
                required: vec![],
            });
            for actual in [depth, nodes, properties, enum_values, bytes]
                .map(ValidatedToolInputSchemaV0::try_from)
            {
                if actual != Err(SchemaConstructionError::SchemaLimitExceeded) {
                    return Err(format!("schema limit recipe produced {actual:?}"));
                }
            }
            expected_named(expected, "SchemaLimitExceeded")
        }
        _ => Err(format!("not a schema recipe: {}", recipe.as_str())),
    }
}

fn verify_schema_error(
    actual: Result<ValidatedToolInputSchemaV0, SchemaConstructionError>,
    expected: &FixtureExpected,
) -> Result<(), String> {
    match actual {
        Err(error) => expected_named(expected, &format!("{error:?}")),
        Ok(_) => Err("schema recipe unexpectedly succeeded".to_owned()),
    }
}

fn verify_arguments(recipe: FixtureRecipe, expected: &FixtureExpected) -> Result<(), String> {
    use FixtureRecipe::*;
    match recipe {
        GoldenArguments => {
            let value = golden_arguments();
            match expected {
                FixtureExpected::CanonicalGolden { bytes, digest } => {
                    if hex::encode(value.canonical_bytes()) == *bytes
                        && value.digest().as_str() == digest
                    {
                        Ok(())
                    } else {
                        Err("argument canonical bytes/digest differ from fixture golden".to_owned())
                    }
                }
                other => Err(format!(
                    "argument golden requires canonical golden expected, got {other:?}"
                )),
            }
        }
        GoldenArgumentsReversed => {
            let mut members = vec![
                (
                    "count".to_owned(),
                    UnvalidatedArgumentValueV0::Integer("2".to_owned()),
                ),
                (
                    "query".to_owned(),
                    UnvalidatedArgumentValueV0::String("graph".to_owned()),
                ),
            ];
            members.reverse();
            let reversed =
                CanonicalArgumentValueV0::try_from(UnvalidatedArgumentValueV0::Object(members))
                    .map_err(|error| error.to_string())?;
            match expected {
                FixtureExpected::EqualTo { case }
                    if case == "arguments-golden" && reversed == golden_arguments() =>
                {
                    Ok(())
                }
                other => Err(format!(
                    "argument permutation does not match fixture expectation {other:?}"
                )),
            }
        }
        ArgumentDuplicateKey => verify_argument_error(
            CanonicalArgumentValueV0::try_from(UnvalidatedArgumentValueV0::Object(vec![
                ("x".to_owned(), UnvalidatedArgumentValueV0::Null),
                ("x".to_owned(), UnvalidatedArgumentValueV0::Null),
            ])),
            expected,
        ),
        ArgumentBadKey => verify_argument_error(
            CanonicalArgumentValueV0::try_from(UnvalidatedArgumentValueV0::Object(vec![(
                "bad key".to_owned(),
                UnvalidatedArgumentValueV0::Null,
            )])),
            expected,
        ),
        ArgumentLimitMatrix => {
            let members = (0..65)
                .map(|index| (format!("m{index}"), UnvalidatedArgumentValueV0::Null))
                .collect();
            let values = [
                nested_argument(9),
                UnvalidatedArgumentValueV0::Array(vec![UnvalidatedArgumentValueV0::Null; 257]),
                UnvalidatedArgumentValueV0::Object(members),
                UnvalidatedArgumentValueV0::Array(vec![UnvalidatedArgumentValueV0::Null; 256]),
                UnvalidatedArgumentValueV0::String("x".repeat(4097)),
                UnvalidatedArgumentValueV0::Array(vec![
                    UnvalidatedArgumentValueV0::String(
                        "x".repeat(4096)
                    );
                    16
                ]),
            ];
            for actual in values.map(CanonicalArgumentValueV0::try_from) {
                if actual != Err(ArgumentConstructionError::ArgumentLimitExceeded) {
                    return Err(format!("argument limit recipe produced {actual:?}"));
                }
            }
            expected_named(expected, "ArgumentLimitExceeded")
        }
        ArgumentNumberOverflow => {
            for value in [
                UnvalidatedArgumentValueV0::Integer("9223372036854775808".to_owned()),
                UnvalidatedArgumentValueV0::Number("1e999".to_owned()),
            ] {
                let actual = CanonicalArgumentValueV0::try_from(value);
                if actual != Err(ArgumentConstructionError::ArgumentNumberOutOfRange) {
                    return Err(format!("argument overflow recipe produced {actual:?}"));
                }
            }
            expected_named(expected, "ArgumentNumberOutOfRange")
        }
        ArgumentNumberNormalization => {
            let negative_zero = CanonicalArgumentValueV0::try_from(
                UnvalidatedArgumentValueV0::Number("-0.0".to_owned()),
            );
            let positive_zero = CanonicalArgumentValueV0::try_from(
                UnvalidatedArgumentValueV0::Number("0.0".to_owned()),
            );
            let subnormal = CanonicalArgumentValueV0::try_from(UnvalidatedArgumentValueV0::Number(
                "5e-324".to_owned(),
            ));
            if negative_zero == positive_zero && subnormal.is_ok() {
                expected_named(expected, "canonical-success")
            } else {
                Err("argument number normalization recipe failed".to_owned())
            }
        }
        ArgumentTypeDistinction => {
            let integer = CanonicalArgumentValueV0::try_from(UnvalidatedArgumentValueV0::Integer(
                "2".to_owned(),
            ));
            let number = CanonicalArgumentValueV0::try_from(UnvalidatedArgumentValueV0::Number(
                "2.0".to_owned(),
            ));
            if integer.is_ok() && number.is_ok() && integer != number {
                expected_named(expected, "distinct-canonical-values")
            } else {
                Err("integer and number canonical values were not distinct".to_owned())
            }
        }
        _ => Err(format!("not an argument recipe: {}", recipe.as_str())),
    }
}

fn verify_argument_error(
    actual: Result<CanonicalArgumentValueV0, ArgumentConstructionError>,
    expected: &FixtureExpected,
) -> Result<(), String> {
    match actual {
        Err(error) => expected_named(expected, &format!("{error:?}")),
        Ok(_) => Err("argument recipe unexpectedly succeeded".to_owned()),
    }
}

fn verify_projection(recipe: FixtureRecipe, expected: &FixtureExpected) -> Result<(), String> {
    use FixtureRecipe::*;
    match recipe {
        ValidAuthority => {
            let (request, candidate) = valid_authority();
            match_projection_result(
                InvocationResolver::resolve_projection(request, vec![candidate]),
                expected,
            )
        }
        TurnTwo => verify_turn_binding(expected),
        CatalogMissing
        | CatalogNotRunnable
        | CatalogPackageVersionNine
        | InstallationPackageDigestNine
        | CatalogComponentMissing
        | InstallationComponentDigestNine
        | ExecutionIdentityUnknown
        | ExecutionIdentityOther
        | ToolMissing
        | ToolIdentityOther
        | InstallationTenantOther
        | GrantUserOtherAndStale
        | CapabilityUnknown
        | CapabilitiesEmpty
        | GrantManifestDigestNine
        | GrantMissing
        | SourcePolicyMissing
        | AdmittedSourcePolicyMissing
        | InstallationMissing
        | InstallationDisabled
        | InstallationRevoked
        | InstallationIdOther
        | CatalogRevoked
        | GrantStale
        | GrantExpired
        | GrantRevoked
        | GrantInstallationOther
        | GrantScopeOther => {
            let (request, mut candidate) = valid_authority();
            apply_projection_recipe(&mut candidate, recipe)?;
            match_projection_result(
                InvocationResolver::resolve_projection(request, vec![candidate]),
                expected,
            )
        }
        EmergencyAndCatalogMissing => {
            let (request, mut candidate) = valid_authority();
            candidate.policy.emergency_blocked = true;
            candidate.catalog = None;
            match_projection_result(
                InvocationResolver::resolve_projection(request, vec![candidate]),
                expected,
            )
        }
        TwoTargetInstallationConflict => {
            let (request, first) = valid_authority();
            let mut second = as_second_tool(first.clone());
            second.installation.as_mut().expect("fixture").id =
                parsed!(InstallationId, "installation:other");
            match_projection_result(
                InvocationResolver::resolve_projection(request, vec![second, first]),
                expected,
            )
        }
        OptionalLayerTransitivity => verify_optional_transitivity(expected),
        OptionalLayerAbsence => verify_optional_absence(expected),
        DefinitionNameChanged | DefinitionDescriptionChanged | DefinitionSchemaBoolean => {
            verify_definition_mutation(recipe, expected)
        }
        DefinitionNameCollision => {
            let (request, first) = valid_authority();
            let mut second = as_second_tool(first.clone());
            second
                .catalog
                .as_mut()
                .expect("fixture")
                .component
                .as_mut()
                .expect("fixture")
                .tool
                .as_mut()
                .expect("fixture")
                .model_visible_name = "campus_search".to_owned();
            match_projection_result(
                InvocationResolver::resolve_projection(request, vec![second, first]),
                expected,
            )
        }
        CompleteProjectionErrorMatrix => verify_complete_projection_matrix(expected),
        ProjectionGroupMajorForward | ProjectionGroupMajorReverse | ProjectionGroupMajorThree => {
            verify_group_major(recipe, expected)
        }
        _ => Err(format!("not a projection recipe: {}", recipe.as_str())),
    }
}

fn apply_projection_recipe(
    candidate: &mut InvocationAuthorityCandidate,
    recipe: FixtureRecipe,
) -> Result<(), String> {
    use FixtureRecipe::*;
    match recipe {
        CatalogMissing => candidate.catalog = None,
        CatalogNotRunnable => candidate.catalog.as_mut().expect("fixture").runnable = false,
        CatalogPackageVersionNine => {
            candidate.catalog.as_mut().expect("fixture").package_version =
                parsed!(PackageVersion, "9.0.0");
        }
        InstallationPackageDigestNine => {
            candidate
                .installation
                .as_mut()
                .expect("fixture")
                .package_digest = digest('9');
        }
        CatalogComponentMissing => candidate.catalog.as_mut().expect("fixture").component = None,
        InstallationComponentDigestNine => {
            candidate
                .installation
                .as_mut()
                .expect("fixture")
                .component
                .digest = digest('9');
        }
        ExecutionIdentityUnknown => candidate.policy.admitted_execution_identity = None,
        ExecutionIdentityOther => {
            candidate.policy.admitted_execution_identity =
                Some(parsed!(ExecutionIdentity, "native:other"));
        }
        ToolMissing => {
            candidate
                .catalog
                .as_mut()
                .expect("fixture")
                .component
                .as_mut()
                .expect("fixture")
                .tool = None;
        }
        ToolIdentityOther => {
            candidate
                .catalog
                .as_mut()
                .expect("fixture")
                .component
                .as_mut()
                .expect("fixture")
                .tool
                .as_mut()
                .expect("fixture")
                .id = parsed!(ToolId, "tool:other");
        }
        InstallationTenantOther => {
            candidate.installation.as_mut().expect("fixture").tenant_id =
                parsed!(TenantId, "tenant:other");
        }
        GrantUserOtherAndStale => {
            let grant = candidate.grant.as_mut().expect("fixture");
            grant.user_id = parsed!(UserId, "user:other");
            grant.state = GrantState::Stale;
        }
        CapabilityUnknown => candidate.policy.capability_class = None,
        CapabilitiesEmpty => candidate
            .catalog
            .as_mut()
            .expect("fixture")
            .component
            .as_mut()
            .expect("fixture")
            .declared_capabilities
            .clear(),
        GrantManifestDigestNine => {
            candidate
                .grant
                .as_mut()
                .expect("fixture")
                .capability_manifest_digest = digest('9')
        }
        GrantMissing => candidate.grant = None,
        SourcePolicyMissing => candidate.catalog.as_mut().expect("fixture").source_policy = None,
        AdmittedSourcePolicyMissing => candidate.policy.admitted_source_policy = None,
        InstallationMissing => candidate.installation = None,
        InstallationDisabled => {
            candidate.installation.as_mut().expect("fixture").state = InstallationState::Disabled
        }
        InstallationRevoked => {
            candidate.installation.as_mut().expect("fixture").state = InstallationState::Revoked
        }
        InstallationIdOther => {
            candidate.installation.as_mut().expect("fixture").id =
                parsed!(InstallationId, "installation:other")
        }
        CatalogRevoked => candidate.catalog.as_mut().expect("fixture").revoked = true,
        GrantStale => candidate.grant.as_mut().expect("fixture").state = GrantState::Stale,
        GrantExpired => candidate.grant.as_mut().expect("fixture").state = GrantState::Expired,
        GrantRevoked => candidate.grant.as_mut().expect("fixture").state = GrantState::Revoked,
        GrantInstallationOther => {
            candidate.grant.as_mut().expect("fixture").installation_id =
                parsed!(InstallationId, "installation:other")
        }
        GrantScopeOther => {
            candidate.grant.as_mut().expect("fixture").object_scope =
                parsed!(ObjectScope, "scope:other")
        }
        _ => {
            return Err(format!(
                "recipe {} is not a single projection mutation",
                recipe.as_str()
            ));
        }
    }
    Ok(())
}

fn match_projection_result(
    actual: Result<ToolProjectionSnapshot, ProjectionResolutionError>,
    expected: &FixtureExpected,
) -> Result<(), String> {
    match (actual, expected) {
        (Err(error), FixtureExpected::Named(name)) if name.as_str() == format!("{error:?}") => {
            Ok(())
        }
        (Ok(projection), FixtureExpected::ResolvedIdentities(identities)) => {
            let entry = projection
                .entries()
                .first()
                .ok_or_else(|| "projection has no entry".to_owned())?;
            let actual = [
                entry.tenant_id().as_str().to_owned(),
                entry.user_id().as_str().to_owned(),
                entry.installation_id().as_str().to_owned(),
                entry.package_id().as_str().to_owned(),
                entry.package_version().as_str(),
                entry.component_id().as_str().to_owned(),
                entry.tool_id().as_str().to_owned(),
            ];
            if actual == *identities {
                Ok(())
            } else {
                Err(format!(
                    "resolved identities differ: actual={actual:?} fixture={identities:?}"
                ))
            }
        }
        (Ok(projection), FixtureExpected::Dispatch(dispatch)) => {
            let actual = projection
                .entries()
                .first()
                .ok_or_else(|| "projection has no entry".to_owned())?
                .dispatch_key();
            if actual == dispatch {
                Ok(())
            } else {
                Err(format!(
                    "dispatch differs: actual={actual} fixture={dispatch}"
                ))
            }
        }
        (
            Ok(projection),
            FixtureExpected::ProjectionGoldens {
                definition,
                schema_set,
                authority_entry,
                authority_set,
                snapshot,
            },
        ) => {
            let entry = projection
                .entries()
                .first()
                .ok_or_else(|| "projection has no entry".to_owned())?;
            if entry.provider_tool_definition_digest().as_str() == definition
                && projection.tool_schema_set_digest().as_str() == schema_set
                && entry.projection_authority_entry_digest().as_str() == authority_entry
                && projection.projection_authority_set_digest().as_str() == authority_set
                && projection.snapshot_id() == snapshot
            {
                Ok(())
            } else {
                Err("projection literal goldens differ from fixture".to_owned())
            }
        }
        (actual, expected) => Err(format!(
            "projection actual={actual:?}, fixture={expected:?}"
        )),
    }
}

fn verify_turn_binding(expected: &FixtureExpected) -> Result<(), String> {
    let (request, candidate) = valid_authority();
    let baseline = InvocationResolver::resolve_projection(request.clone(), vec![candidate.clone()])
        .map_err(|error| error.to_string())?;
    let mut next_request = request;
    next_request.turn_id = parsed!(TurnId, "turn:2");
    let next = InvocationResolver::resolve_projection(next_request, vec![candidate])
        .map_err(|error| error.to_string())?;
    if baseline.tool_schema_set_digest() == next.tool_schema_set_digest()
        && baseline.snapshot_id() != next.snapshot_id()
    {
        expected_named(expected, "same-schema-set;different-snapshot")
    } else {
        Err("turn mutation did not preserve schema set and change snapshot".to_owned())
    }
}

fn verify_definition_mutation(
    recipe: FixtureRecipe,
    expected: &FixtureExpected,
) -> Result<(), String> {
    let (request, candidate) = valid_authority();
    let baseline = InvocationResolver::resolve_projection(request.clone(), vec![candidate.clone()])
        .map_err(|error| error.to_string())?;
    let mut changed = candidate;
    let tool = changed
        .catalog
        .as_mut()
        .expect("fixture")
        .component
        .as_mut()
        .expect("fixture")
        .tool
        .as_mut()
        .expect("fixture");
    match recipe {
        FixtureRecipe::DefinitionNameChanged => {
            tool.model_visible_name = "campus_search_changed".to_owned()
        }
        FixtureRecipe::DefinitionDescriptionChanged => tool.description.push_str(" Changed."),
        FixtureRecipe::DefinitionSchemaBoolean => {
            let schema = ValidatedToolInputSchemaV0::try_from(UnvalidatedToolInputSchemaV0 {
                dialect: "tool-input-schema/v0".to_owned(),
                root: UnvalidatedSchemaNodeV0::Object {
                    properties: vec![("enabled".to_owned(), UnvalidatedSchemaNodeV0::Boolean)],
                    required: vec!["enabled".to_owned()],
                },
            })
            .map_err(|error| error.to_string())?;
            tool.claimed_input_schema_digest = schema.digest().clone();
            tool.input_schema = Some(schema);
        }
        _ => return Err("not a definition mutation".to_owned()),
    }
    let changed = InvocationResolver::resolve_projection(request, vec![changed])
        .map_err(|error| error.to_string())?;
    if baseline.entries()[0].provider_tool_definition_digest()
        != changed.entries()[0].provider_tool_definition_digest()
        && baseline.tool_schema_set_digest() != changed.tool_schema_set_digest()
    {
        expected_named(
            expected,
            "provider-definition-and-schema-set-digests-change",
        )
    } else {
        Err("definition mutation did not change both required digests".to_owned())
    }
}

fn verify_optional_transitivity(expected: &FixtureExpected) -> Result<(), String> {
    for layer in ["catalog", "component", "installation", "grant"] {
        let (request, mut first) = valid_authority();
        first = as_tool(first, "tool:a-first", "a_first");
        let mut middle = as_tool(first.clone(), "tool:m-middle", "m_middle");
        let mut last = as_tool(first.clone(), "tool:z-last", "z_last");
        match layer {
            "catalog" => {
                first.catalog = None;
                middle.catalog = valid_authority().1.catalog;
                last.catalog = valid_authority().1.catalog;
                last.catalog.as_mut().expect("fixture").revoked = true;
            }
            "component" => {
                first.catalog.as_mut().expect("fixture").component = None;
                middle.catalog.as_mut().expect("fixture").component =
                    valid_authority().1.catalog.expect("fixture").component;
                last.catalog.as_mut().expect("fixture").component =
                    valid_authority().1.catalog.expect("fixture").component;
                last.catalog
                    .as_mut()
                    .expect("fixture")
                    .component
                    .as_mut()
                    .expect("fixture")
                    .kind = ComponentKind::McpServerComponent;
            }
            "installation" => {
                first.installation = None;
                middle.installation = valid_authority().1.installation;
                last.installation = valid_authority().1.installation;
                last.installation.as_mut().expect("fixture").state = InstallationState::Disabled;
            }
            "grant" => {
                first.grant = None;
                middle.grant = valid_authority().1.grant;
                last.grant = valid_authority().1.grant;
                last.grant.as_mut().expect("fixture").state = GrantState::Stale;
            }
            _ => unreachable!(),
        }
        let actual = InvocationResolver::resolve_projection(request, vec![last, first, middle]);
        if actual != Err(ProjectionResolutionError::AuthorityConflict) {
            return Err(format!("optional {layer} transitivity produced {actual:?}"));
        }
    }
    expected_named(expected, "AuthorityConflict")
}

fn verify_optional_absence(expected: &FixtureExpected) -> Result<(), String> {
    let cases = [
        ("catalog", ProjectionResolutionError::PackageMissing),
        ("component", ProjectionResolutionError::ComponentMissing),
        (
            "installation",
            ProjectionResolutionError::InstallationMissing,
        ),
        ("grant", ProjectionResolutionError::CapabilityNotGranted),
    ];
    let mut observed = Vec::new();
    for (layer, error) in cases {
        for uniform in [false, true] {
            let (request, mut first) = valid_authority();
            first = as_tool(first, "tool:a-first", "a_first");
            let mut second = as_second_tool(first.clone());
            match layer {
                "catalog" => {
                    first.catalog = None;
                    if uniform {
                        second.catalog = None;
                    }
                }
                "component" => {
                    first.catalog.as_mut().expect("fixture").component = None;
                    if uniform {
                        second.catalog.as_mut().expect("fixture").component = None;
                    }
                }
                "installation" => {
                    first.installation = None;
                    if uniform {
                        second.installation = None;
                    }
                }
                "grant" => {
                    first.grant = None;
                    if uniform {
                        second.grant = None;
                    }
                }
                _ => unreachable!(),
            }
            let actual = InvocationResolver::resolve_projection(request, vec![second, first]);
            if actual != Err(error) {
                return Err(format!(
                    "optional {layer} absence uniform={uniform} produced {actual:?}"
                ));
            }
        }
        observed.push(FixtureExpectedName::parse(&format!("{error:?}"))?);
    }
    match expected {
        FixtureExpected::ProjectionErrors(errors) if *errors == observed => Ok(()),
        other => Err(format!(
            "optional absence errors actual={observed:?} fixture={other:?}"
        )),
    }
}

fn verify_group_major(recipe: FixtureRecipe, expected: &FixtureExpected) -> Result<(), String> {
    use FixtureRecipe::*;
    let (request, mut first) = valid_authority();
    first = as_tool(first, "tool:a-first", "a_first");
    let mut middle = as_tool(first.clone(), "tool:m-middle", "m_middle");
    let mut last = as_second_tool(first.clone());
    match recipe {
        ProjectionGroupMajorForward => {
            first.installation.as_mut().expect("fixture").state = InstallationState::Disabled;
            last.installation.as_mut().expect("fixture").state = InstallationState::Disabled;
            last.catalog = None;
            match_projection_result(
                InvocationResolver::resolve_projection(request, vec![last, first]),
                expected,
            )
        }
        ProjectionGroupMajorReverse => {
            first.catalog = None;
            first.installation.as_mut().expect("fixture").state = InstallationState::Disabled;
            last.installation.as_mut().expect("fixture").state = InstallationState::Disabled;
            match_projection_result(
                InvocationResolver::resolve_projection(request, vec![last, first]),
                expected,
            )
        }
        ProjectionGroupMajorThree => {
            first.grant.as_mut().expect("fixture").state = GrantState::Stale;
            middle.grant.as_mut().expect("fixture").state = GrantState::Stale;
            last.grant.as_mut().expect("fixture").state = GrantState::Stale;
            first.installation.as_mut().expect("fixture").state = InstallationState::Disabled;
            middle.installation.as_mut().expect("fixture").state = InstallationState::Disabled;
            last.installation.as_mut().expect("fixture").state = InstallationState::Disabled;
            last.catalog = None;
            match_projection_result(
                InvocationResolver::resolve_projection(request, vec![last, first, middle]),
                expected,
            )
        }
        _ => Err("not a group-major recipe".to_owned()),
    }
}

fn verify_complete_projection_matrix(expected: &FixtureExpected) -> Result<(), String> {
    use ProjectionFault::*;
    let faults = [
        (PackageMissing, ProjectionResolutionError::PackageMissing),
        (
            PackageNotRunnable,
            ProjectionResolutionError::PackageNotRunnable,
        ),
        (
            PackageVersionMismatch,
            ProjectionResolutionError::PackageVersionMismatch,
        ),
        (
            PackageDigestMismatch,
            ProjectionResolutionError::PackageDigestMismatch,
        ),
        (CatalogRevoked, ProjectionResolutionError::CatalogRevoked),
        (
            InstallationMissing,
            ProjectionResolutionError::InstallationMissing,
        ),
        (
            InstallationDisabled,
            ProjectionResolutionError::InstallationDisabled,
        ),
        (
            InstallationRevoked,
            ProjectionResolutionError::InstallationRevoked,
        ),
        (
            InstallationRevisionMismatch,
            ProjectionResolutionError::InstallationRevisionMismatch,
        ),
        (
            ComponentMissing,
            ProjectionResolutionError::ComponentMissing,
        ),
        (
            ComponentIdentityMismatch,
            ProjectionResolutionError::ComponentIdentityMismatch,
        ),
        (
            ExecutionIdentityUnknown,
            ProjectionResolutionError::ExecutionIdentityUnknown,
        ),
        (
            ExecutionIdentityMismatch,
            ProjectionResolutionError::ExecutionIdentityMismatch,
        ),
        (ToolMissing, ProjectionResolutionError::ToolMissing),
        (
            ToolIdentityMismatch,
            ProjectionResolutionError::ToolIdentityMismatch,
        ),
        (
            CapabilityUnknown,
            ProjectionResolutionError::CapabilityUnknown,
        ),
        (
            CapabilityNotDeclared,
            ProjectionResolutionError::CapabilityNotDeclared,
        ),
        (
            CapabilityManifestMismatch,
            ProjectionResolutionError::CapabilityManifestMismatch,
        ),
        (
            CapabilityNotGranted,
            ProjectionResolutionError::CapabilityNotGranted,
        ),
        (GrantStale, ProjectionResolutionError::GrantStale),
        (GrantExpired, ProjectionResolutionError::GrantExpired),
        (GrantRevoked, ProjectionResolutionError::GrantRevoked),
        (
            GrantVersionMismatch,
            ProjectionResolutionError::GrantVersionMismatch,
        ),
        (
            GrantScopeMismatch,
            ProjectionResolutionError::GrantScopeMismatch,
        ),
        (
            SourcePolicyMissing,
            ProjectionResolutionError::SourcePolicyMissing,
        ),
        (
            SourcePolicyMismatch,
            ProjectionResolutionError::SourcePolicyMismatch,
        ),
        (SchemaMissing, ProjectionResolutionError::SchemaMissing),
        (
            SchemaDigestMismatch,
            ProjectionResolutionError::SchemaDigestMismatch,
        ),
    ];
    for (fault, error) in faults {
        let (request, mut candidate) = valid_authority();
        apply_projection_fault(&mut candidate, fault);
        let actual = InvocationResolver::resolve_projection(request, vec![candidate]);
        if actual != Err(error) {
            return Err(format!("projection matrix {error:?} produced {actual:?}"));
        }
    }
    let (request, candidate) = valid_authority();
    if InvocationResolver::resolve_projection(request.clone(), vec![])
        != Err(ProjectionResolutionError::InvalidRequest)
    {
        return Err("projection matrix InvalidRequest failed".to_owned());
    }
    if InvocationResolver::resolve_projection(
        request.clone(),
        vec![candidate.clone(), candidate.clone()],
    ) != Err(ProjectionResolutionError::InvalidAuthoritySnapshot)
    {
        return Err("projection matrix InvalidAuthoritySnapshot failed".to_owned());
    }
    let mut emergency = candidate.clone();
    emergency.policy.emergency_blocked = true;
    if InvocationResolver::resolve_projection(request.clone(), vec![emergency])
        != Err(ProjectionResolutionError::EmergencyBlocked)
    {
        return Err("projection matrix EmergencyBlocked failed".to_owned());
    }
    let mut conflict = as_second_tool(candidate.clone());
    conflict.installation.as_mut().expect("fixture").id =
        parsed!(InstallationId, "installation:other");
    if InvocationResolver::resolve_projection(request.clone(), vec![candidate.clone(), conflict])
        != Err(ProjectionResolutionError::AuthorityConflict)
    {
        return Err("projection matrix AuthorityConflict failed".to_owned());
    }
    let mut scope = candidate.clone();
    scope.installation.as_mut().expect("fixture").tenant_id = parsed!(TenantId, "tenant:other");
    if InvocationResolver::resolve_projection(request.clone(), vec![scope])
        != Err(ProjectionResolutionError::TenantOrUserScopeMismatch)
    {
        return Err("projection matrix TenantOrUserScopeMismatch failed".to_owned());
    }
    let mut collision = as_second_tool(candidate.clone());
    collision
        .catalog
        .as_mut()
        .expect("fixture")
        .component
        .as_mut()
        .expect("fixture")
        .tool
        .as_mut()
        .expect("fixture")
        .model_visible_name = "campus_search".to_owned();
    if InvocationResolver::resolve_projection(request, vec![candidate, collision])
        != Err(ProjectionResolutionError::ToolNameCollision)
    {
        return Err("projection matrix ToolNameCollision failed".to_owned());
    }
    expected_named(expected, "all-ProjectionResolutionError-variants")
}

fn apply_projection_fault(candidate: &mut InvocationAuthorityCandidate, fault: ProjectionFault) {
    use ProjectionFault::*;
    match fault {
        PackageMissing => candidate.catalog = None,
        PackageNotRunnable => candidate.catalog.as_mut().expect("fixture").runnable = false,
        PackageVersionMismatch => {
            candidate.catalog.as_mut().expect("fixture").package_version =
                parsed!(PackageVersion, "9.0.0")
        }
        PackageDigestMismatch => {
            candidate
                .installation
                .as_mut()
                .expect("fixture")
                .package_digest = digest('9')
        }
        CatalogRevoked => candidate.catalog.as_mut().expect("fixture").revoked = true,
        InstallationMissing => candidate.installation = None,
        InstallationDisabled => {
            candidate.installation.as_mut().expect("fixture").state = InstallationState::Disabled
        }
        InstallationRevoked => {
            candidate.installation.as_mut().expect("fixture").state = InstallationState::Revoked
        }
        InstallationRevisionMismatch => {
            candidate.installation.as_mut().expect("fixture").id =
                parsed!(InstallationId, "installation:other")
        }
        ComponentMissing => candidate.catalog.as_mut().expect("fixture").component = None,
        ComponentIdentityMismatch => {
            candidate
                .installation
                .as_mut()
                .expect("fixture")
                .component
                .digest = digest('9')
        }
        ExecutionIdentityUnknown => candidate.policy.admitted_execution_identity = None,
        ExecutionIdentityMismatch => {
            candidate.policy.admitted_execution_identity =
                Some(parsed!(ExecutionIdentity, "native:other"))
        }
        ToolMissing => {
            candidate
                .catalog
                .as_mut()
                .expect("fixture")
                .component
                .as_mut()
                .expect("fixture")
                .tool = None
        }
        ToolIdentityMismatch => {
            candidate
                .catalog
                .as_mut()
                .expect("fixture")
                .component
                .as_mut()
                .expect("fixture")
                .tool
                .as_mut()
                .expect("fixture")
                .id = parsed!(ToolId, "tool:other")
        }
        CapabilityUnknown => candidate.policy.capability_class = None,
        CapabilityNotDeclared => candidate
            .catalog
            .as_mut()
            .expect("fixture")
            .component
            .as_mut()
            .expect("fixture")
            .declared_capabilities
            .clear(),
        CapabilityManifestMismatch => {
            candidate
                .grant
                .as_mut()
                .expect("fixture")
                .capability_manifest_digest = digest('9')
        }
        CapabilityNotGranted => candidate.grant = None,
        GrantStale => candidate.grant.as_mut().expect("fixture").state = GrantState::Stale,
        GrantExpired => candidate.grant.as_mut().expect("fixture").state = GrantState::Expired,
        GrantRevoked => candidate.grant.as_mut().expect("fixture").state = GrantState::Revoked,
        GrantVersionMismatch => {
            candidate.grant.as_mut().expect("fixture").installation_id =
                parsed!(InstallationId, "installation:other")
        }
        GrantScopeMismatch => {
            candidate.grant.as_mut().expect("fixture").object_scope =
                parsed!(ObjectScope, "scope:other")
        }
        SourcePolicyMissing => candidate.catalog.as_mut().expect("fixture").source_policy = None,
        SourcePolicyMismatch => candidate.policy.admitted_source_policy = None,
        SchemaMissing => {
            candidate
                .catalog
                .as_mut()
                .expect("fixture")
                .component
                .as_mut()
                .expect("fixture")
                .tool
                .as_mut()
                .expect("fixture")
                .input_schema = None
        }
        SchemaDigestMismatch => {
            candidate
                .catalog
                .as_mut()
                .expect("fixture")
                .component
                .as_mut()
                .expect("fixture")
                .tool
                .as_mut()
                .expect("fixture")
                .claimed_input_schema_digest = digest('9')
        }
    }
}

fn verify_call(recipe: FixtureRecipe, expected: &FixtureExpected) -> Result<(), String> {
    use FixtureRecipe::*;
    match recipe {
        CallNameNotProjected
        | CallDispatchWrongLiteral
        | GrantTenantOtherAndStale
        | CallNameBeforeDeny
        | CallDispatchBeforeDeny
        | CallDispatchBeforeArguments
        | PostEmergency
        | PostCatalogRevoked
        | InstallationDisabled
        | GrantStale => {
            let (projection, mut current, mut call) = valid_call_state();
            match recipe {
                CallNameNotProjected => call.model_visible_name = "not_projected".to_owned(),
                CallDispatchWrongLiteral => call.dispatch_key = "dispatch:sha256:wrong".to_owned(),
                GrantTenantOtherAndStale => {
                    current.grant.tenant_id = parsed!(TenantId, "tenant:other");
                    current.grant.state = GrantState::Stale;
                }
                CallNameBeforeDeny => {
                    call.model_visible_name = "not_projected".to_owned();
                    current.policy.emergency_blocked = true;
                }
                CallDispatchBeforeDeny => {
                    call.dispatch_key = "wrong".to_owned();
                    current.policy.emergency_blocked = true;
                }
                CallDispatchBeforeArguments => {
                    call.dispatch_key = "wrong".to_owned();
                    call.claimed_argument_digest = digest('9');
                }
                PostEmergency => current.policy.emergency_blocked = true,
                PostCatalogRevoked => current.catalog_revoked = true,
                InstallationDisabled => {
                    current.installation.as_mut().expect("fixture").state =
                        InstallationState::Disabled
                }
                GrantStale => current.grant.state = GrantState::Stale,
                _ => unreachable!(),
            }
            match_call_result(authorize_call(&projection, current, call), expected)
        }
        CallNoFallback => verify_call_no_fallback(expected),
        CompleteCallErrorMatrix => verify_complete_call_matrix(expected),
        FrozenDisabledThenEnabled => verify_terminal_projection_denial(expected),
        _ => Err(format!("not an authorization recipe: {}", recipe.as_str())),
    }
}

fn match_call_result(
    actual: Result<AuthorizedInvocation, InvocationAuthorizationError>,
    expected: &FixtureExpected,
) -> Result<(), String> {
    match actual {
        Err(error) => expected_named(expected, &format!("{error:?}")),
        Ok(_) => Err(format!(
            "call unexpectedly authorized for fixture {expected:?}"
        )),
    }
}

fn verify_call_no_fallback(expected: &FixtureExpected) -> Result<(), String> {
    let (request, first) = valid_authority();
    let second = as_second_tool(first.clone());
    let projection = InvocationResolver::resolve_projection(request, vec![second, first.clone()])
        .map_err(|error| error.to_string())?;
    let (projection, current, mut call) = call_state_from_resolved(projection, first);
    let second_dispatch = projection
        .entries()
        .iter()
        .find(|entry| entry.model_visible_name() == "z_last")
        .ok_or_else(|| "second projected entry missing".to_owned())?
        .dispatch_key()
        .to_owned();
    call.dispatch_key = second_dispatch;
    match_call_result(authorize_call(&projection, current, call), expected)
}

fn verify_complete_call_matrix(expected: &FixtureExpected) -> Result<(), String> {
    for index in 0_u8..18 {
        let (projection, mut current, mut call) = valid_call_state();
        let error = match index {
            0 => {
                call.model_visible_name.clear();
                InvocationAuthorizationError::InvalidCall
            }
            1 => {
                call.model_visible_name = "not_projected".to_owned();
                InvocationAuthorizationError::ToolNotProjected
            }
            2 => {
                call.dispatch_key = "wrong".to_owned();
                InvocationAuthorizationError::DispatchIdentityMismatch
            }
            3 => {
                current.policy.emergency_blocked = true;
                InvocationAuthorizationError::EmergencyBlocked
            }
            4 => {
                current.policy.capability_class = None;
                InvocationAuthorizationError::AuthorityConflict
            }
            5 => {
                current.tenant_id = parsed!(TenantId, "tenant:other");
                InvocationAuthorizationError::TenantOrUserScopeMismatch
            }
            6 => {
                current.catalog_revoked = true;
                InvocationAuthorizationError::CatalogRevoked
            }
            7 => {
                current.installation = None;
                InvocationAuthorizationError::InstallationMissing
            }
            8 => {
                current.installation.as_mut().expect("fixture").state = InstallationState::Disabled;
                InvocationAuthorizationError::InstallationDisabled
            }
            9 => {
                current.installation.as_mut().expect("fixture").state = InstallationState::Revoked;
                InvocationAuthorizationError::InstallationRevoked
            }
            10 => {
                current.installation.as_mut().expect("fixture").revision =
                    parsed!(InstallationRevision, "installation-revision:other");
                InvocationAuthorizationError::InstallationRevisionMismatch
            }
            11 => {
                current.grant.state = GrantState::Stale;
                InvocationAuthorizationError::GrantStale
            }
            12 => {
                current.grant.state = GrantState::Expired;
                InvocationAuthorizationError::GrantExpired
            }
            13 => {
                current.grant.state = GrantState::Revoked;
                InvocationAuthorizationError::GrantRevoked
            }
            14 => {
                current.grant.version = parsed!(GrantVersion, "grant-version:other");
                InvocationAuthorizationError::GrantVersionMismatch
            }
            15 => {
                current.grant.object_scope = parsed!(ObjectScope, "scope:other");
                InvocationAuthorizationError::GrantScopeMismatch
            }
            16 => {
                call.claimed_argument_digest = digest('9');
                InvocationAuthorizationError::ArgumentDigestMismatch
            }
            17 => {
                call.arguments =
                    CanonicalArgumentValueV0::try_from(UnvalidatedArgumentValueV0::Object(vec![
                        (
                            "count".to_owned(),
                            UnvalidatedArgumentValueV0::Integer("2".to_owned()),
                        ),
                        (
                            "query".to_owned(),
                            UnvalidatedArgumentValueV0::Integer("3".to_owned()),
                        ),
                    ]))
                    .map_err(|error| error.to_string())?;
                call.claimed_argument_digest = call.arguments.digest().clone();
                InvocationAuthorizationError::ArgumentsInvalid
            }
            _ => unreachable!(),
        };
        let actual = authorize_call(&projection, current, call);
        if actual != Err(error) {
            return Err(format!("call matrix {error:?} produced {actual:?}"));
        }
    }
    expected_named(expected, "all-InvocationAuthorizationError-variants")
}

fn verify_terminal_projection_denial(expected: &FixtureExpected) -> Result<(), String> {
    let (request, mut candidate) = valid_authority();
    candidate.installation.as_mut().expect("fixture").state = InstallationState::Disabled;
    let denied = InvocationResolver::resolve_projection(request, vec![candidate.clone()]);
    candidate.installation.as_mut().expect("fixture").state = InstallationState::Enabled;
    if denied == Err(ProjectionResolutionError::InstallationDisabled) {
        expected_named(expected, "no-projected-entry")
    } else {
        Err(format!("frozen disabled projection produced {denied:?}"))
    }
}

fn verify_run_spec_eligibility(
    recipe: FixtureRecipe,
    expected: &FixtureExpected,
) -> Result<(), String> {
    if recipe != FixtureRecipe::ProjectionValidAuthority {
        return Err("run-spec mapping requires projection=valid_authority".to_owned());
    }
    let (request, candidate) = valid_authority();
    let projection = InvocationResolver::resolve_projection(request, vec![candidate]);
    if projection.is_ok() {
        expected_named(expected, "success-only")
    } else {
        Err(format!(
            "run-spec mapping projection failed: {projection:?}"
        ))
    }
}
