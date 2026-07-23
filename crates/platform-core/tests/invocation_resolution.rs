use std::collections::BTreeSet;
use ustc_campus_agent_core::invocation::*;

const SCHEMA_HEX: &str = "746f6f6c2d696e7075742d736368656d612f7630000100000000000000020000000000000005636f756e7403000000000000000571756572790200000000000000000100000000000000057175657279";
const SCHEMA_DIGEST: &str =
    "sha256:8a91a2fdad047d1bcfc4ac0392778f7125afce4faf637ed3aac4fd535fd1db2e";
const ARGUMENT_HEX: &str = "746f6f6c2d617267756d656e74732f7630000600000000000000020000000000000005636f756e74020000000000000002000000000000000571756572790400000000000000056772617068";
const ARGUMENT_DIGEST: &str =
    "sha256:8b881ec565f0aac688241061c398f199c8e0683604502f1e7538f09f33350451";

fn golden_schema(properties_reversed: bool) -> UnvalidatedToolInputSchemaV0 {
    let mut properties = vec![
        ("count".to_owned(), UnvalidatedSchemaNodeV0::Integer),
        (
            "query".to_owned(),
            UnvalidatedSchemaNodeV0::String { enum_values: None },
        ),
    ];
    if properties_reversed {
        properties.reverse();
    }
    UnvalidatedToolInputSchemaV0 {
        dialect: "tool-input-schema/v0".to_owned(),
        root: UnvalidatedSchemaNodeV0::Object {
            properties,
            required: vec!["query".to_owned()],
        },
    }
}

#[test]
fn schema_golden_vector_is_exact_and_permutation_stable() {
    let left = ValidatedToolInputSchemaV0::try_from(golden_schema(false));
    let right = ValidatedToolInputSchemaV0::try_from(golden_schema(true));
    let (Ok(left), Ok(right)) = (left, right) else {
        panic!("golden schemas must validate");
    };
    assert_eq!(left, right);
    assert_eq!(hex::encode(left.canonical_bytes()), SCHEMA_HEX);
    assert_eq!(left.digest().as_str(), SCHEMA_DIGEST);
}

#[test]
fn argument_golden_vector_is_exact_and_permutation_stable() {
    let make = |reverse| {
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
        if reverse {
            members.reverse();
        }
        CanonicalArgumentValueV0::try_from(UnvalidatedArgumentValueV0::Object(members))
    };
    let (Ok(left), Ok(right)) = (make(false), make(true)) else {
        panic!("golden arguments must validate");
    };
    assert_eq!(left, right);
    assert_eq!(hex::encode(left.canonical_bytes()), ARGUMENT_HEX);
    assert_eq!(left.digest().as_str(), ARGUMENT_DIGEST);
}

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

fn valid_authority() -> (ToolProjectionRequest, InvocationAuthorityCandidate) {
    let schema = match ValidatedToolInputSchemaV0::try_from(golden_schema(false)) {
        Ok(schema) => schema,
        Err(error) => panic!("fixture schema must validate: {error}"),
    };
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
    let source_policy = SourcePolicyIdentity {
        id: parsed!(SourcePolicyId, "source-policy:synthetic-v1"),
        digest: digest('5'),
    };
    let component = CatalogComponentRevision {
        id: component_id.clone(),
        kind: ComponentKind::NativeRustComponent,
        version: component_version.clone(),
        digest: digest('2'),
        execution_identity: execution_identity.clone(),
        declared_capabilities: BTreeSet::from([capability_id.clone()]),
        tool: Some(CatalogToolDefinition {
            id: tool_id.clone(),
            model_visible_name: "campus_search".to_owned(),
            description: "Search synthetic approved campus rules.".to_owned(),
            capability_id: capability_id.clone(),
            claimed_input_schema_digest: schema.digest().clone(),
            input_schema: Some(schema),
        }),
    };
    let installation = PluginInstallationSnapshot {
        id: installation_id.clone(),
        tenant_id: tenant_id.clone(),
        user_id: user_id.clone(),
        package_id: package_id.clone(),
        package_version: package_version.clone(),
        package_digest: digest('1'),
        component: InstalledComponentIdentity {
            id: component_id.clone(),
            version: component_version,
            digest: digest('2'),
            execution_identity: execution_identity.clone(),
        },
        state: InstallationState::Enabled,
        revision: parsed!(InstallationRevision, "installation-revision:7"),
    };
    let grant = CapabilityGrantSnapshot {
        snapshot_id: parsed!(GrantSnapshotId, "grant:synthetic-v3"),
        version: parsed!(GrantVersion, "grant-version:3"),
        tenant_id: tenant_id.clone(),
        user_id: user_id.clone(),
        installation_id: installation_id.clone(),
        capability_id: capability_id.clone(),
        object_scope: object_scope.clone(),
        confirmation_policy: ConfirmationPolicy::Allow,
        capability_manifest_digest: digest('3'),
        state: GrantState::Active,
    };
    let policy = InvocationPolicySnapshot {
        snapshot_id: parsed!(PolicySnapshotId, "policy:synthetic-v9"),
        revision: parsed!(PolicyRevision, "policy-revision:9"),
        capability_id: capability_id.clone(),
        capability_class: Some(CapabilityClass::PublicRead),
        admitted_execution_identity: Some(execution_identity),
        admitted_source_policy: Some(source_policy.clone()),
        emergency_blocked: false,
    };
    let target = InvocationTarget {
        installation_id,
        package_id: package_id.clone(),
        package_version: package_version.clone(),
        component_id,
        tool_id,
        capability_id,
        object_scope,
    };
    (
        ToolProjectionRequest {
            tenant_id,
            user_id,
            run_id: parsed!(RunId, "run:synthetic-1"),
            turn_id: parsed!(TurnId, "turn:1"),
            activation_allowlist: None,
        },
        InvocationAuthorityCandidate {
            target,
            catalog: Some(CatalogPackageRevision {
                catalog_revision: parsed!(CatalogRevision, "catalog:synthetic-v4"),
                package_id,
                package_version,
                package_digest: digest('1'),
                runnable: true,
                revoked: false,
                capability_manifest_digest: digest('3'),
                source_policy: Some(source_policy),
                component: Some(component),
            }),
            installation: Some(installation),
            grant: Some(grant),
            policy,
        },
    )
}

fn resolve_valid() -> (ToolProjectionSnapshot, InvocationAuthorityCandidate) {
    let (request, candidate) = valid_authority();
    let result = InvocationResolver::resolve_projection(request, vec![candidate.clone()]);
    match result {
        Ok(projection) => (projection, candidate),
        Err(error) => panic!("valid authority must resolve: {error}"),
    }
}

fn as_second_tool(mut candidate: InvocationAuthorityCandidate) -> InvocationAuthorityCandidate {
    candidate.target.tool_id = parsed!(ToolId, "tool:z-last");
    let tool = candidate
        .catalog
        .as_mut()
        .expect("fixture")
        .component
        .as_mut()
        .expect("fixture")
        .tool
        .as_mut()
        .expect("fixture");
    tool.id = candidate.target.tool_id.clone();
    tool.model_visible_name = "z_last".to_owned();
    candidate
}

#[test]
fn valid_projection_is_deterministic_and_turn_bound() {
    let (request, candidate) = valid_authority();
    let left = InvocationResolver::resolve_projection(request.clone(), vec![candidate.clone()]);
    let right = InvocationResolver::resolve_projection(request.clone(), vec![candidate.clone()]);
    assert_eq!(left, right);
    let Ok(left) = left else {
        panic!("valid projection expected")
    };
    assert_eq!(left.entries().len(), 1);
    assert!(
        left.entries()[0]
            .dispatch_key()
            .starts_with("dispatch:sha256:")
    );
    let mut next_request = request;
    next_request.turn_id = parsed!(TurnId, "turn:2");
    let next = InvocationResolver::resolve_projection(next_request, vec![candidate]);
    let Ok(next) = next else {
        panic!("fresh turn must resolve")
    };
    assert_eq!(left.tool_schema_set_digest(), next.tool_schema_set_digest());
    assert_ne!(left.snapshot_id(), next.snapshot_id());

    let (request, first) = valid_authority();
    let mut second = first.clone();
    second.target.tool_id = parsed!(ToolId, "tool:z-last");
    let second_tool = second
        .catalog
        .as_mut()
        .expect("fixture")
        .component
        .as_mut()
        .expect("fixture")
        .tool
        .as_mut()
        .expect("fixture");
    second_tool.id = second.target.tool_id.clone();
    second_tool.model_visible_name = "z_last".to_owned();
    let forward = InvocationResolver::resolve_projection(
        request.clone(),
        vec![first.clone(), second.clone()],
    );
    let reverse = InvocationResolver::resolve_projection(request, vec![second, first]);
    assert_eq!(forward, reverse);
    let Ok(ordered) = forward else {
        panic!("multi-tool projection must resolve")
    };
    assert!(
        ordered
            .entries()
            .windows(2)
            .all(|pair| pair[0].tool_id() < pair[1].tool_id())
    );
}

#[test]
fn provider_definition_mutations_change_projection_digests() {
    let (request, candidate) = valid_authority();
    let baseline = InvocationResolver::resolve_projection(request.clone(), vec![candidate.clone()]);
    let Ok(baseline) = baseline else {
        panic!("baseline must resolve")
    };
    for mutation in 0..3 {
        let mut changed = candidate.clone();
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
        match mutation {
            0 => tool.model_visible_name = "campus_search_changed".to_owned(),
            1 => tool.description.push_str(" Changed."),
            2 => {
                let schema = ValidatedToolInputSchemaV0::try_from(UnvalidatedToolInputSchemaV0 {
                    dialect: "tool-input-schema/v0".to_owned(),
                    root: UnvalidatedSchemaNodeV0::Object {
                        properties: vec![("enabled".to_owned(), UnvalidatedSchemaNodeV0::Boolean)],
                        required: vec!["enabled".to_owned()],
                    },
                });
                let Ok(schema) = schema else {
                    panic!("mutation schema must validate")
                };
                tool.claimed_input_schema_digest = schema.digest().clone();
                tool.input_schema = Some(schema);
            }
            _ => panic!("mutation table is closed"),
        }
        let resolved = InvocationResolver::resolve_projection(request.clone(), vec![changed]);
        let Ok(resolved) = resolved else {
            panic!("mutated definition must resolve")
        };
        assert_ne!(
            baseline.entries()[0].provider_tool_definition_digest(),
            resolved.entries()[0].provider_tool_definition_digest()
        );
        assert_ne!(
            baseline.tool_schema_set_digest(),
            resolved.tool_schema_set_digest()
        );
    }
}

#[derive(Clone, Copy)]
enum ProjectionFault {
    PackageMissing,
    PackageNotRunnable,
    PackageVersionMismatch,
    PackageDigestMismatch,
    CatalogRevoked,
    InstallationMissing,
    InstallationDisabled,
    InstallationRevoked,
    InstallationRevisionMismatch,
    ComponentMissing,
    ComponentIdentityMismatch,
    ExecutionIdentityUnknown,
    ExecutionIdentityMismatch,
    ToolMissing,
    ToolIdentityMismatch,
    CapabilityUnknown,
    CapabilityNotDeclared,
    CapabilityManifestMismatch,
    CapabilityNotGranted,
    GrantStale,
    GrantExpired,
    GrantRevoked,
    GrantVersionMismatch,
    GrantScopeMismatch,
    SourcePolicyMissing,
    SourcePolicyMismatch,
    SchemaMissing,
    SchemaDigestMismatch,
}

#[test]
fn projection_denials_are_typed_and_table_driven() {
    use ProjectionFault::*;
    let cases = [
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
    for (fault, expected) in cases {
        let (request, mut candidate) = valid_authority();
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
                candidate.installation.as_mut().expect("fixture").state =
                    InstallationState::Disabled
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
                candidate.grant.as_mut().expect("fixture").user_id = parsed!(UserId, "user:other")
            }
            GrantScopeMismatch => {
                candidate.grant.as_mut().expect("fixture").object_scope =
                    parsed!(ObjectScope, "scope:other")
            }
            SourcePolicyMissing => {
                candidate.catalog.as_mut().expect("fixture").source_policy = None
            }
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
        assert_eq!(
            InvocationResolver::resolve_projection(request, vec![candidate]),
            Err(expected)
        );
    }
}

#[test]
fn projection_primary_precedence_and_collisions_fail_closed() {
    let (request, mut candidate) = valid_authority();
    candidate.policy.emergency_blocked = true;
    candidate.catalog = None;
    assert_eq!(
        InvocationResolver::resolve_projection(request, vec![candidate]),
        Err(ProjectionResolutionError::EmergencyBlocked)
    );

    let (request, mut candidate) = valid_authority();
    candidate.catalog.as_mut().expect("fixture").revoked = true;
    candidate
        .installation
        .as_mut()
        .expect("fixture")
        .package_digest = digest('9');
    assert_eq!(
        InvocationResolver::resolve_projection(request, vec![candidate]),
        Err(ProjectionResolutionError::PackageDigestMismatch)
    );

    let (request, candidate) = valid_authority();
    assert_eq!(
        InvocationResolver::resolve_projection(request.clone(), vec![]),
        Err(ProjectionResolutionError::InvalidRequest)
    );
    assert_eq!(
        InvocationResolver::resolve_projection(
            request.clone(),
            vec![candidate.clone(), candidate.clone()]
        ),
        Err(ProjectionResolutionError::InvalidAuthoritySnapshot)
    );

    let mut mixed = candidate.clone();
    mixed.target.installation_id = parsed!(InstallationId, "installation:mixed");
    assert_eq!(
        InvocationResolver::resolve_projection(request.clone(), vec![candidate.clone(), mixed]),
        Err(ProjectionResolutionError::AuthorityConflict)
    );

    let mut collision = candidate.clone();
    collision.target.tool_id = parsed!(ToolId, "tool:second");
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
        .id = collision.target.tool_id.clone();
    assert_eq!(
        InvocationResolver::resolve_projection(request, vec![candidate, collision]),
        Err(ProjectionResolutionError::ToolNameCollision)
    );

    let (request, mut canonical_first) = valid_authority();
    canonical_first.target.tool_id = parsed!(ToolId, "tool:a-first");
    canonical_first.catalog = None;
    let (_, mut canonical_second) = valid_authority();
    canonical_second.target.tool_id = parsed!(ToolId, "tool:z-last");
    canonical_second
        .catalog
        .as_mut()
        .expect("fixture")
        .component
        .as_mut()
        .expect("fixture")
        .tool
        .as_mut()
        .expect("fixture")
        .id = canonical_second.target.tool_id.clone();
    canonical_second
        .installation
        .as_mut()
        .expect("fixture")
        .state = InstallationState::Disabled;
    assert_eq!(
        InvocationResolver::resolve_projection(request, vec![canonical_second, canonical_first]),
        Err(ProjectionResolutionError::PackageMissing)
    );

    let (request, mut invalid) = valid_authority();
    invalid.policy.emergency_blocked = true;
    invalid
        .catalog
        .as_mut()
        .expect("fixture")
        .component
        .as_mut()
        .expect("fixture")
        .tool
        .as_mut()
        .expect("fixture")
        .model_visible_name = "invalid name".to_owned();
    assert_eq!(
        InvocationResolver::resolve_projection(request, vec![invalid]),
        Err(ProjectionResolutionError::InvalidAuthoritySnapshot)
    );
}

#[derive(Clone, Copy)]
enum AuthorityAnchorFault {
    InstallationRevision,
    PackageDigest,
    CatalogRevision,
    ComponentVersion,
    ComponentDigest,
    ComponentKind,
    ExecutionIdentity,
    CapabilityManifest,
    GrantVersion,
    GrantScope,
    GrantConfirmation,
    SourcePolicy,
    PolicySnapshot,
    PolicyRevision,
    CapabilityClass,
}

#[test]
fn mixed_multi_tool_authority_anchor_is_rejected_table_driven() {
    use AuthorityAnchorFault::*;
    let cases = [
        InstallationRevision,
        PackageDigest,
        CatalogRevision,
        ComponentVersion,
        ComponentDigest,
        ComponentKind,
        ExecutionIdentity,
        CapabilityManifest,
        GrantVersion,
        GrantScope,
        GrantConfirmation,
        SourcePolicy,
        PolicySnapshot,
        PolicyRevision,
        CapabilityClass,
    ];
    for fault in cases {
        let (request, first) = valid_authority();
        let mut second = as_second_tool(first.clone());
        match fault {
            InstallationRevision => {
                second.installation.as_mut().expect("fixture").revision = parsed!(
                    ustc_campus_agent_core::invocation::InstallationRevision,
                    "installation-revision:other"
                );
            }
            PackageDigest => {
                second.catalog.as_mut().expect("fixture").package_digest = digest('9');
                second
                    .installation
                    .as_mut()
                    .expect("fixture")
                    .package_digest = digest('9');
            }
            CatalogRevision => {
                second.catalog.as_mut().expect("fixture").catalog_revision = parsed!(
                    ustc_campus_agent_core::invocation::CatalogRevision,
                    "catalog:other"
                );
            }
            ComponentVersion => {
                let version = parsed!(
                    ustc_campus_agent_core::invocation::ComponentVersion,
                    "component-version:other"
                );
                second
                    .catalog
                    .as_mut()
                    .expect("fixture")
                    .component
                    .as_mut()
                    .expect("fixture")
                    .version = version.clone();
                second
                    .installation
                    .as_mut()
                    .expect("fixture")
                    .component
                    .version = version;
            }
            ComponentDigest => {
                second
                    .catalog
                    .as_mut()
                    .expect("fixture")
                    .component
                    .as_mut()
                    .expect("fixture")
                    .digest = digest('9');
                second
                    .installation
                    .as_mut()
                    .expect("fixture")
                    .component
                    .digest = digest('9');
            }
            ComponentKind => {
                second
                    .catalog
                    .as_mut()
                    .expect("fixture")
                    .component
                    .as_mut()
                    .expect("fixture")
                    .kind = ustc_campus_agent_core::invocation::ComponentKind::McpServerComponent;
            }
            ExecutionIdentity => {
                let execution = parsed!(
                    ustc_campus_agent_core::invocation::ExecutionIdentity,
                    "native:other"
                );
                second
                    .catalog
                    .as_mut()
                    .expect("fixture")
                    .component
                    .as_mut()
                    .expect("fixture")
                    .execution_identity = execution.clone();
                second
                    .installation
                    .as_mut()
                    .expect("fixture")
                    .component
                    .execution_identity = execution.clone();
                second.policy.admitted_execution_identity = Some(execution);
            }
            CapabilityManifest => {
                second
                    .catalog
                    .as_mut()
                    .expect("fixture")
                    .capability_manifest_digest = digest('9');
                second
                    .grant
                    .as_mut()
                    .expect("fixture")
                    .capability_manifest_digest = digest('9');
            }
            GrantVersion => {
                second.grant.as_mut().expect("fixture").version = parsed!(
                    ustc_campus_agent_core::invocation::GrantVersion,
                    "grant-version:other"
                );
            }
            GrantScope => {
                let scope = parsed!(ObjectScope, "scope:other");
                second.target.object_scope = scope.clone();
                second.grant.as_mut().expect("fixture").object_scope = scope;
            }
            GrantConfirmation => {
                second.grant.as_mut().expect("fixture").confirmation_policy =
                    ConfirmationPolicy::Ask;
            }
            SourcePolicy => {
                let source = SourcePolicyIdentity {
                    id: parsed!(SourcePolicyId, "source-policy:other"),
                    digest: digest('9'),
                };
                second.catalog.as_mut().expect("fixture").source_policy = Some(source.clone());
                second.policy.admitted_source_policy = Some(source);
            }
            PolicySnapshot => {
                second.policy.snapshot_id = parsed!(PolicySnapshotId, "policy:other");
            }
            PolicyRevision => {
                second.policy.revision = parsed!(
                    ustc_campus_agent_core::invocation::PolicyRevision,
                    "policy-revision:other"
                );
            }
            CapabilityClass => {
                second.policy.capability_class =
                    Some(ustc_campus_agent_core::invocation::CapabilityClass::TenantPrivateRead);
            }
        }
        assert_eq!(
            InvocationResolver::resolve_projection(request, vec![first, second]),
            Err(ProjectionResolutionError::AuthorityConflict)
        );
    }
}

fn golden_arguments() -> CanonicalArgumentValueV0 {
    let value = UnvalidatedArgumentValueV0::Object(vec![
        (
            "count".to_owned(),
            UnvalidatedArgumentValueV0::Integer("2".to_owned()),
        ),
        (
            "query".to_owned(),
            UnvalidatedArgumentValueV0::String("graph".to_owned()),
        ),
    ]);
    match CanonicalArgumentValueV0::try_from(value) {
        Ok(arguments) => arguments,
        Err(error) => panic!("fixture arguments must validate: {error}"),
    }
}

fn valid_call_state() -> (ToolProjectionSnapshot, CurrentDenyState, ProposedToolCall) {
    let (projection, candidate) = resolve_valid();
    let Some(installation) = candidate.installation else {
        panic!("fixture installation required")
    };
    let Some(grant) = candidate.grant else {
        panic!("fixture grant required")
    };
    let arguments = golden_arguments();
    let entry = &projection.entries()[0];
    let call = ProposedToolCall {
        provider_tool_call_id: parsed!(ProviderToolCallId, "provider-call:1"),
        model_visible_name: entry.model_visible_name().to_owned(),
        dispatch_key: entry.dispatch_key().to_owned(),
        claimed_argument_digest: arguments.digest().clone(),
        arguments,
    };
    let current = CurrentDenyState {
        tenant_id: entry.tenant_id().clone(),
        user_id: entry.user_id().clone(),
        catalog_revoked: false,
        installation: Some(installation),
        grant,
        policy: candidate.policy,
    };
    (projection, current, call)
}

#[test]
fn call_authorization_uses_only_frozen_dispatch_and_current_narrowing() {
    let (projection, current, call) = valid_call_state();
    let authorized = authorize_call(&projection, current.clone(), call.clone());
    let Ok(authorized) = authorized else {
        panic!("valid frozen call must authorize")
    };
    assert_eq!(authorized.entry, projection.entries()[0]);
    assert_eq!(authorized.arguments.digest().as_str(), ARGUMENT_DIGEST);

    let mut unknown = call.clone();
    unknown.model_visible_name = "not_projected".to_owned();
    unknown.claimed_argument_digest = digest('9');
    let mut blocked = current.clone();
    blocked.policy.emergency_blocked = true;
    assert_eq!(
        authorize_call(&projection, blocked, unknown),
        Err(InvocationAuthorizationError::ToolNotProjected)
    );

    let mut wrong_dispatch = call;
    wrong_dispatch.dispatch_key = "dispatch:sha256:wrong".to_owned();
    assert_eq!(
        authorize_call(&projection, current, wrong_dispatch),
        Err(InvocationAuthorizationError::DispatchIdentityMismatch)
    );

    let (projection, mut current, call) = valid_call_state();
    current.policy.snapshot_id = parsed!(PolicySnapshotId, "policy:other");
    current.tenant_id = parsed!(TenantId, "tenant:other");
    assert_eq!(
        authorize_call(&projection, current, call),
        Err(InvocationAuthorizationError::AuthorityConflict)
    );
}

#[test]
fn call_time_denials_are_typed_and_table_driven() {
    for case in 0_u8..19 {
        let (projection, mut current, mut call) = valid_call_state();
        let expected = match case {
            0 => {
                call.model_visible_name.clear();
                InvocationAuthorizationError::InvalidCall
            }
            1 => {
                current.policy.emergency_blocked = true;
                InvocationAuthorizationError::EmergencyBlocked
            }
            2 => {
                current.tenant_id = parsed!(TenantId, "tenant:other");
                InvocationAuthorizationError::TenantOrUserScopeMismatch
            }
            3 => {
                current.catalog_revoked = true;
                InvocationAuthorizationError::CatalogRevoked
            }
            4 => {
                current.installation = None;
                InvocationAuthorizationError::InstallationMissing
            }
            5 => {
                current.installation.as_mut().expect("fixture").state = InstallationState::Disabled;
                InvocationAuthorizationError::InstallationDisabled
            }
            6 => {
                current.installation.as_mut().expect("fixture").state = InstallationState::Revoked;
                InvocationAuthorizationError::InstallationRevoked
            }
            7 => {
                current.installation.as_mut().expect("fixture").revision =
                    parsed!(InstallationRevision, "installation-revision:other");
                InvocationAuthorizationError::InstallationRevisionMismatch
            }
            8 => {
                current.grant.state = GrantState::Stale;
                InvocationAuthorizationError::GrantStale
            }
            9 => {
                current.grant.state = GrantState::Expired;
                InvocationAuthorizationError::GrantExpired
            }
            10 => {
                current.grant.state = GrantState::Revoked;
                InvocationAuthorizationError::GrantRevoked
            }
            11 => {
                current.grant.version = parsed!(GrantVersion, "grant-version:other");
                InvocationAuthorizationError::GrantVersionMismatch
            }
            12 => {
                current.grant.object_scope = parsed!(ObjectScope, "scope:other");
                InvocationAuthorizationError::GrantScopeMismatch
            }
            13 => {
                call.claimed_argument_digest = digest('9');
                InvocationAuthorizationError::ArgumentDigestMismatch
            }
            14 => {
                call.arguments = match CanonicalArgumentValueV0::try_from(
                    UnvalidatedArgumentValueV0::Object(vec![
                        (
                            "count".to_owned(),
                            UnvalidatedArgumentValueV0::Integer("2".to_owned()),
                        ),
                        (
                            "query".to_owned(),
                            UnvalidatedArgumentValueV0::Integer("3".to_owned()),
                        ),
                    ]),
                ) {
                    Ok(arguments) => arguments,
                    Err(error) => panic!("invalid-for-schema arguments remain canonical: {error}"),
                };
                call.claimed_argument_digest = call.arguments.digest().clone();
                InvocationAuthorizationError::ArgumentsInvalid
            }
            15 => {
                current.policy.capability_class = None;
                InvocationAuthorizationError::AuthorityConflict
            }
            16 => {
                current.installation.as_mut().expect("fixture").tenant_id =
                    parsed!(TenantId, "tenant:other");
                InvocationAuthorizationError::TenantOrUserScopeMismatch
            }
            17 => {
                current.installation.as_mut().expect("fixture").user_id =
                    parsed!(UserId, "user:other");
                InvocationAuthorizationError::TenantOrUserScopeMismatch
            }
            18 => {
                current.policy.capability_class = Some(CapabilityClass::TenantPrivateWrite);
                InvocationAuthorizationError::AuthorityConflict
            }
            _ => panic!("case table is closed"),
        };
        assert_eq!(authorize_call(&projection, current, call), Err(expected));
    }

    let (projection, mut current, call) = valid_call_state();
    current.policy.snapshot_id = parsed!(PolicySnapshotId, "policy:other");
    assert_eq!(
        authorize_call(&projection, current, call),
        Err(InvocationAuthorizationError::AuthorityConflict)
    );

    let (projection, mut current, call) = valid_call_state();
    current.policy.revision = parsed!(PolicyRevision, "policy-revision:other");
    assert_eq!(
        authorize_call(&projection, current, call),
        Err(InvocationAuthorizationError::AuthorityConflict)
    );

    let (projection, current, mut call) = valid_call_state();
    call.arguments =
        match CanonicalArgumentValueV0::try_from(UnvalidatedArgumentValueV0::Object(vec![
            (
                "count".to_owned(),
                UnvalidatedArgumentValueV0::Number("2.0".to_owned()),
            ),
            (
                "query".to_owned(),
                UnvalidatedArgumentValueV0::String("graph".to_owned()),
            ),
        ])) {
            Ok(arguments) => arguments,
            Err(error) => panic!("number argument must remain canonical: {error}"),
        };
    call.claimed_argument_digest = call.arguments.digest().clone();
    assert_eq!(
        authorize_call(&projection, current, call),
        Err(InvocationAuthorizationError::ArgumentsInvalid)
    );
}

fn expected_fixture_cases(name: &str) -> &'static [&'static str] {
    match name {
        "schema-golden-v0.json" => &[
            "golden-bytes-digest",
            "permutation-equality",
            "dialect",
            "duplicate-property",
            "required-subset",
            "depth-limit",
            "node-limit",
            "property-limit",
            "enum-limit",
            "byte-limit",
        ],
        "arguments-golden-v0.json" => &[
            "golden-bytes-digest",
            "permutation-equality",
            "duplicate-key",
            "invalid-name",
            "depth-limit",
            "node-limit",
            "member-limit",
            "array-limit",
            "string-limit",
            "byte-limit",
            "i64-min-max-overflow",
            "negative-zero",
            "subnormal-non-finite",
            "integer-number-distinction",
        ],
        "valid-synthetic-v0.json" => &[
            "resolved-identities",
            "dispatch-digest",
            "projection-digests",
            "turn-bound-snapshot",
            "run-spec-mapping",
        ],
        "identity-mismatch-v0.json" => &[
            "package-missing",
            "package-not-runnable",
            "package-version-mismatch",
            "package-digest-mismatch",
            "component-missing",
            "component-identity-mismatch",
            "execution-unknown",
            "execution-mismatch",
        ],
        "tool-identity-mismatch-v0.json" => &["tool-missing", "tool-identity-mismatch"],
        "scope-capability-source-v0.json" => &[
            "tenant-user-mismatch",
            "capability-unknown",
            "capability-not-declared",
            "capability-manifest-mismatch",
            "capability-not-granted",
            "source-policy-missing",
            "source-policy-mismatch",
        ],
        "installation-authority-v0.json" => &[
            "installation-missing",
            "installation-disabled",
            "installation-revoked",
            "installation-revision-mismatch",
            "catalog-revoked",
            "emergency-blocked",
            "authority-conflict",
        ],
        "grant-scope-stale-v0.json" => &[
            "grant-stale",
            "grant-expired",
            "grant-revoked",
            "grant-version-mismatch",
            "grant-scope-mismatch",
        ],
        "tool-definition-mutation-v0.json" => &[
            "name-mutation",
            "description-mutation",
            "schema-mutation",
            "visible-name-collision",
        ],
        "call-dispatch-denials-v0.json" => &[
            "tool-not-projected",
            "dispatch-identity-mismatch",
            "no-fallback",
        ],
        "projection-precedence-v0.json" => &[
            "every-projection-error",
            "canonical-target-order",
            "dual-fault-primary-order",
        ],
        "call-precedence-v0.json" => &[
            "every-call-error",
            "tool-before-deny",
            "dispatch-before-deny",
            "dispatch-before-arguments",
        ],
        "post-projection-revoke-v0.json" => &[
            "emergency-narrows",
            "catalog-revoke-narrows",
            "installation-narrows",
            "grant-narrows",
            "later-enable-cannot-widen",
        ],
        _ => panic!("unexpected fixture {name}"),
    }
}

#[test]
fn exact_synthetic_fixture_case_catalog_is_stable() {
    let fixtures = [
        (
            "schema-golden-v0.json",
            include_str!("fixtures/invocation-resolution/schema-golden-v0.json"),
        ),
        (
            "arguments-golden-v0.json",
            include_str!("fixtures/invocation-resolution/arguments-golden-v0.json"),
        ),
        (
            "valid-synthetic-v0.json",
            include_str!("fixtures/invocation-resolution/valid-synthetic-v0.json"),
        ),
        (
            "identity-mismatch-v0.json",
            include_str!("fixtures/invocation-resolution/identity-mismatch-v0.json"),
        ),
        (
            "tool-identity-mismatch-v0.json",
            include_str!("fixtures/invocation-resolution/tool-identity-mismatch-v0.json"),
        ),
        (
            "scope-capability-source-v0.json",
            include_str!("fixtures/invocation-resolution/scope-capability-source-v0.json"),
        ),
        (
            "installation-authority-v0.json",
            include_str!("fixtures/invocation-resolution/installation-authority-v0.json"),
        ),
        (
            "grant-scope-stale-v0.json",
            include_str!("fixtures/invocation-resolution/grant-scope-stale-v0.json"),
        ),
        (
            "tool-definition-mutation-v0.json",
            include_str!("fixtures/invocation-resolution/tool-definition-mutation-v0.json"),
        ),
        (
            "call-dispatch-denials-v0.json",
            include_str!("fixtures/invocation-resolution/call-dispatch-denials-v0.json"),
        ),
        (
            "projection-precedence-v0.json",
            include_str!("fixtures/invocation-resolution/projection-precedence-v0.json"),
        ),
        (
            "call-precedence-v0.json",
            include_str!("fixtures/invocation-resolution/call-precedence-v0.json"),
        ),
        (
            "post-projection-revoke-v0.json",
            include_str!("fixtures/invocation-resolution/post-projection-revoke-v0.json"),
        ),
    ];
    for (name, source) in fixtures {
        let value = match serde_json::from_str::<serde_json::Value>(source) {
            Ok(value) => value,
            Err(error) => panic!("{name} must be valid JSON: {error}"),
        };
        assert_eq!(value["schema_version"], "invocation-resolution-fixture/v0");
        assert_eq!(value["synthetic"], true);
        assert_eq!(value["fixture"], name);
        let actual = value["cases"]
            .as_array()
            .expect("fixture cases must be an array")
            .iter()
            .map(|case| case.as_str().expect("fixture case must be a string"))
            .collect::<Vec<_>>();
        assert_eq!(actual, expected_fixture_cases(name));
    }
}

fn nested_schema(depth: usize) -> UnvalidatedSchemaNodeV0 {
    if depth == 1 {
        UnvalidatedSchemaNodeV0::String { enum_values: None }
    } else {
        UnvalidatedSchemaNodeV0::Array {
            items: Box::new(nested_schema(depth - 1)),
        }
    }
}

fn schema_with_total_nodes(node_count: usize) -> UnvalidatedSchemaNodeV0 {
    assert!(matches!(node_count, 256 | 257));
    let properties = (0..64)
        .map(|index| {
            let leaf_count = if node_count == 256 && index == 63 {
                2
            } else {
                3
            };
            let children = (0..leaf_count)
                .map(|leaf| (format!("leaf{leaf}"), UnvalidatedSchemaNodeV0::Integer))
                .collect();
            (
                format!("branch{index}"),
                UnvalidatedSchemaNodeV0::Object {
                    properties: children,
                    required: vec![],
                },
            )
        })
        .collect();
    UnvalidatedSchemaNodeV0::Object {
        properties,
        required: vec![],
    }
}

#[test]
fn schema_constructor_enforces_structural_and_byte_limits() {
    let schema = |root| UnvalidatedToolInputSchemaV0 {
        dialect: "tool-input-schema/v0".to_owned(),
        root,
    };
    assert_eq!(
        ValidatedToolInputSchemaV0::try_from(UnvalidatedToolInputSchemaV0 {
            dialect: "other".to_owned(),
            root: UnvalidatedSchemaNodeV0::Object {
                properties: vec![],
                required: vec![]
            },
        }),
        Err(SchemaConstructionError::SchemaDialectUnsupported)
    );
    assert_eq!(
        ValidatedToolInputSchemaV0::try_from(schema(UnvalidatedSchemaNodeV0::Integer)),
        Err(SchemaConstructionError::SchemaMalformed)
    );
    let duplicate = UnvalidatedSchemaNodeV0::Object {
        properties: vec![
            ("x".to_owned(), UnvalidatedSchemaNodeV0::Integer),
            ("x".to_owned(), UnvalidatedSchemaNodeV0::Integer),
        ],
        required: vec![],
    };
    assert_eq!(
        ValidatedToolInputSchemaV0::try_from(schema(duplicate)),
        Err(SchemaConstructionError::SchemaMalformed)
    );
    let missing = UnvalidatedSchemaNodeV0::Object {
        properties: vec![],
        required: vec!["x".to_owned()],
    };
    assert_eq!(
        ValidatedToolInputSchemaV0::try_from(schema(missing)),
        Err(SchemaConstructionError::SchemaMalformed)
    );
    let depth_ok = UnvalidatedSchemaNodeV0::Object {
        properties: vec![("x".to_owned(), nested_schema(7))],
        required: vec![],
    };
    assert!(ValidatedToolInputSchemaV0::try_from(schema(depth_ok)).is_ok());
    let depth_bad = UnvalidatedSchemaNodeV0::Object {
        properties: vec![("x".to_owned(), nested_schema(8))],
        required: vec![],
    };
    assert_eq!(
        ValidatedToolInputSchemaV0::try_from(schema(depth_bad)),
        Err(SchemaConstructionError::SchemaLimitExceeded)
    );
    assert!(ValidatedToolInputSchemaV0::try_from(schema(schema_with_total_nodes(256))).is_ok());
    assert_eq!(
        ValidatedToolInputSchemaV0::try_from(schema(schema_with_total_nodes(257))),
        Err(SchemaConstructionError::SchemaLimitExceeded)
    );
    let properties = (0..65)
        .map(|i| (format!("p{i}"), UnvalidatedSchemaNodeV0::Integer))
        .collect();
    assert_eq!(
        ValidatedToolInputSchemaV0::try_from(schema(UnvalidatedSchemaNodeV0::Object {
            properties,
            required: vec![]
        })),
        Err(SchemaConstructionError::SchemaLimitExceeded)
    );
    for values in [vec![], (0..65).map(|i| format!("v{i}")).collect()] {
        let root = UnvalidatedSchemaNodeV0::Object {
            properties: vec![(
                "x".to_owned(),
                UnvalidatedSchemaNodeV0::String {
                    enum_values: Some(values),
                },
            )],
            required: vec![],
        };
        assert_eq!(
            ValidatedToolInputSchemaV0::try_from(schema(root)),
            Err(SchemaConstructionError::SchemaLimitExceeded)
        );
    }
    let huge_enum = (0..64)
        .map(|i| format!("{i:02}{}", "x".repeat(254)))
        .collect::<Vec<_>>();
    let properties = (0..64)
        .map(|i| {
            (
                format!("p{i}"),
                UnvalidatedSchemaNodeV0::String {
                    enum_values: Some(huge_enum.clone()),
                },
            )
        })
        .collect();
    assert_eq!(
        ValidatedToolInputSchemaV0::try_from(schema(UnvalidatedSchemaNodeV0::Object {
            properties,
            required: vec![]
        })),
        Err(SchemaConstructionError::SchemaLimitExceeded)
    );
}

fn nested_argument(depth: usize) -> UnvalidatedArgumentValueV0 {
    if depth == 1 {
        UnvalidatedArgumentValueV0::Null
    } else {
        UnvalidatedArgumentValueV0::Array(vec![nested_argument(depth - 1)])
    }
}

#[test]
fn argument_constructor_enforces_numeric_structural_and_byte_limits() {
    for token in [i64::MIN.to_string(), i64::MAX.to_string()] {
        assert!(
            CanonicalArgumentValueV0::try_from(UnvalidatedArgumentValueV0::Integer(token)).is_ok()
        );
    }
    for token in ["9223372036854775808", "-9223372036854775809"] {
        assert_eq!(
            CanonicalArgumentValueV0::try_from(UnvalidatedArgumentValueV0::Integer(
                token.to_owned()
            )),
            Err(ArgumentConstructionError::ArgumentNumberOutOfRange)
        );
    }
    assert_eq!(
        CanonicalArgumentValueV0::try_from(UnvalidatedArgumentValueV0::Number("-0.0".to_owned())),
        CanonicalArgumentValueV0::try_from(UnvalidatedArgumentValueV0::Number("0.0".to_owned()))
    );
    assert!(
        CanonicalArgumentValueV0::try_from(UnvalidatedArgumentValueV0::Number("5e-324".to_owned()))
            .is_ok()
    );
    assert_eq!(
        CanonicalArgumentValueV0::try_from(UnvalidatedArgumentValueV0::Number("1e999".to_owned())),
        Err(ArgumentConstructionError::ArgumentNumberOutOfRange)
    );
    let duplicate = UnvalidatedArgumentValueV0::Object(vec![
        ("x".to_owned(), UnvalidatedArgumentValueV0::Null),
        ("x".to_owned(), UnvalidatedArgumentValueV0::Null),
    ]);
    assert_eq!(
        CanonicalArgumentValueV0::try_from(duplicate),
        Err(ArgumentConstructionError::ArgumentDuplicateKey)
    );
    let invalid = UnvalidatedArgumentValueV0::Object(vec![(
        "bad key".to_owned(),
        UnvalidatedArgumentValueV0::Null,
    )]);
    assert_eq!(
        CanonicalArgumentValueV0::try_from(invalid),
        Err(ArgumentConstructionError::ArgumentInvalidName)
    );
    assert!(CanonicalArgumentValueV0::try_from(nested_argument(8)).is_ok());
    assert_eq!(
        CanonicalArgumentValueV0::try_from(nested_argument(9)),
        Err(ArgumentConstructionError::ArgumentLimitExceeded)
    );
    assert!(
        CanonicalArgumentValueV0::try_from(UnvalidatedArgumentValueV0::Array(
            vec![UnvalidatedArgumentValueV0::Null; 255]
        ))
        .is_ok()
    );
    assert_eq!(
        CanonicalArgumentValueV0::try_from(UnvalidatedArgumentValueV0::Array(
            vec![UnvalidatedArgumentValueV0::Null; 256]
        )),
        Err(ArgumentConstructionError::ArgumentLimitExceeded)
    );
    assert_eq!(
        CanonicalArgumentValueV0::try_from(UnvalidatedArgumentValueV0::Array(
            vec![UnvalidatedArgumentValueV0::Null; 257]
        )),
        Err(ArgumentConstructionError::ArgumentLimitExceeded)
    );
    let members = (0..65)
        .map(|i| (format!("m{i}"), UnvalidatedArgumentValueV0::Null))
        .collect();
    assert_eq!(
        CanonicalArgumentValueV0::try_from(UnvalidatedArgumentValueV0::Object(members)),
        Err(ArgumentConstructionError::ArgumentLimitExceeded)
    );
    assert_eq!(
        CanonicalArgumentValueV0::try_from(UnvalidatedArgumentValueV0::String("x".repeat(4097))),
        Err(ArgumentConstructionError::ArgumentLimitExceeded)
    );
    let oversized =
        UnvalidatedArgumentValueV0::Array(vec![
            UnvalidatedArgumentValueV0::String("x".repeat(4096));
            16
        ]);
    assert_eq!(
        CanonicalArgumentValueV0::try_from(oversized),
        Err(ArgumentConstructionError::ArgumentLimitExceeded)
    );
}
