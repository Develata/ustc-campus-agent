use std::collections::BTreeSet;
use ustc_campus_agent_core::invocation::*;

macro_rules! parsed {
    ($kind:ty, $value:expr) => {{
        match <$kind>::parse($value) {
            Ok(value) => value,
            Err(error) => panic!("synthetic fixture must parse: {error}"),
        }
    }};
}

pub fn digest(byte: char) -> Sha256Digest {
    parsed!(
        Sha256Digest,
        format!("sha256:{}", byte.to_string().repeat(64))
    )
}

pub fn proof_authority() -> (ToolProjectionRequest, InvocationAuthorityCandidate) {
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
