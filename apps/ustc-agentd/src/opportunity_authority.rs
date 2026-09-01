//! M20 authorization carrier for the static Opportunity Graph use cases.
//!
//! This module maps each M72 application operation to exact transaction-current
//! package, installation, grant, capability, scope, and policy state. Operation
//! locators stay private to this carrier and are never projected to an Agent.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Mutex;

use ustc_campus_agent_application_ingress::OpportunityInvocationError;
use ustc_campus_agent_client_protocol::OpportunityCommandDto;
use ustc_campus_agent_core::identity::{TenantId, UserId};
use ustc_campus_agent_core::invocation::{
    CapabilityClass, CapabilityGrantSnapshot, CapabilityId, CatalogComponentRevision,
    CatalogPackageRevision, CatalogRevision, ComponentId, ComponentKind, ComponentVersion,
    ConfirmationPolicy, ExecutionIdentity, GrantSnapshotId, GrantState, GrantVersion,
    InstallationId, InstallationRevision, InstallationState, InstalledComponentIdentity,
    InvocationPolicySnapshot, InvocationTarget, ObjectScope, PackageId, PackageVersion,
    PluginInstallationSnapshot, PolicyRevision, PolicySnapshotId, Sha256Digest, SourcePolicyId,
    SourcePolicyIdentity, ToolId,
};
use ustc_campus_agent_core::market::authority::{
    AuthorityRepositoryError, InMemoryInvocationAuthorityRepository, InvocationAuthorityRepository,
    InvocationAuthorityService, InvocationRecheckError,
};
use ustc_campus_agent_core::market::load_package_manifest;
use ustc_campus_agent_core::request_context::M00AdmittedActor;

use crate::opportunity_fixture::OpportunityAuthorityMutationMode;

const OPPORTUNITY_PACKAGE_MANIFEST: &[u8] =
    include_bytes!("../../../market/packages/ustc.opportunity-graph/package.json");
const OPPORTUNITY_RESOURCE_COMPONENT_DESCRIPTOR: &[u8] = include_bytes!(
    "../../../market/packages/ustc.opportunity-graph/components/course-planning-resource-pack.json"
);

struct BorrowedAuthorityRepository<'a>(&'a InMemoryInvocationAuthorityRepository);

impl InvocationAuthorityRepository for BorrowedAuthorityRepository<'_> {
    type ReadTransaction<'a>
        = <InMemoryInvocationAuthorityRepository as InvocationAuthorityRepository>::ReadTransaction<
        'a,
    >
    where
        Self: 'a;

    fn begin_read(&self) -> Result<Self::ReadTransaction<'_>, AuthorityRepositoryError> {
        self.0.begin_read()
    }
}

#[derive(Clone, Copy)]
pub(crate) struct OpportunityOperationMetadata {
    operation_id: &'static str,
    suffix: &'static str,
    capability_id: &'static str,
    capability_class: CapabilityClass,
}

const PROFILE_CREATE_METADATA: OpportunityOperationMetadata = OpportunityOperationMetadata {
    operation_id: "profile.academic.create",
    suffix: "profile-create",
    capability_id: "user.own_academic_snapshot.write",
    capability_class: CapabilityClass::TenantPrivateWrite,
};
const PROFILE_VIEW_METADATA: OpportunityOperationMetadata = OpportunityOperationMetadata {
    operation_id: "profile.academic.view",
    suffix: "profile-view",
    capability_id: "user.own_academic_snapshot.read",
    capability_class: CapabilityClass::TenantPrivateRead,
};
const PLAN_GENERATE_METADATA: OpportunityOperationMetadata = OpportunityOperationMetadata {
    operation_id: "planner.generate",
    suffix: "plan-generate",
    capability_id: "user.own_plan_draft.write",
    capability_class: CapabilityClass::TenantPrivateWrite,
};
const PROFILE_DELETE_METADATA: OpportunityOperationMetadata = OpportunityOperationMetadata {
    operation_id: "profile.academic.revoke_delete",
    suffix: "profile-delete",
    capability_id: "user.own_academic_snapshot.write",
    capability_class: CapabilityClass::TenantPrivateWrite,
};
const ALL_OPERATION_METADATA: [OpportunityOperationMetadata; 4] = [
    PROFILE_CREATE_METADATA,
    PROFILE_VIEW_METADATA,
    PLAN_GENERATE_METADATA,
    PROFILE_DELETE_METADATA,
];

pub(crate) fn operation_metadata_for_command(
    command: &OpportunityCommandDto,
) -> OpportunityOperationMetadata {
    match command {
        OpportunityCommandDto::CreateProfile { .. } => PROFILE_CREATE_METADATA,
        OpportunityCommandDto::ViewProfile { .. } => PROFILE_VIEW_METADATA,
        OpportunityCommandDto::GeneratePlan { .. } => PLAN_GENERATE_METADATA,
        OpportunityCommandDto::RevokeConsentAndDeleteProfile { .. } => PROFILE_DELETE_METADATA,
    }
}

struct OpportunityAuthorityEntry {
    target: InvocationTarget,
    repository: InMemoryInvocationAuthorityRepository,
    grant: CapabilityGrantSnapshot,
}

pub(crate) struct OpportunityMarketAuthorityStore {
    entries: BTreeMap<&'static str, Mutex<OpportunityAuthorityEntry>>,
    mutation: OpportunityAuthorityMutationMode,
}

impl OpportunityMarketAuthorityStore {
    pub(crate) fn new(
        tenant_id: TenantId,
        user_id: UserId,
        enabled: bool,
        grant_active: bool,
        source_evidence_digest: &str,
        mutation: OpportunityAuthorityMutationMode,
    ) -> Result<Self, OpportunityInvocationError> {
        let manifest = load_package_manifest(OPPORTUNITY_PACKAGE_MANIFEST)
            .map_err(|_| OpportunityInvocationError::Internal)?;
        let exact_resource_component = manifest.components().len() == 1
            && manifest.components()[0].kind() == ComponentKind::DeclarativeResourcePack
            && manifest.components()[0].path()
                == "market/packages/ustc.opportunity-graph/components/course-planning-resource-pack.json";
        if manifest.package_id().as_str() != "ustc.opportunity-graph"
            || manifest.package_version().as_str() != "0.1.0"
            || !exact_resource_component
        {
            return Err(OpportunityInvocationError::Internal);
        }
        let source_evidence_digest = Sha256Digest::parse(source_evidence_digest.to_owned())
            .map_err(|_| OpportunityInvocationError::Internal)?;
        let combined_source_policy_digest = Sha256Digest::from_bytes(
            format!(
                "opportunity-current-source-policy/v1\0{}\0{}",
                manifest.source_policy_digest().as_str(),
                source_evidence_digest.as_str()
            )
            .as_bytes(),
        );

        let mut entries = BTreeMap::new();
        for metadata in ALL_OPERATION_METADATA {
            let declared_capability = CapabilityId::parse(metadata.capability_id)
                .map_err(|_| OpportunityInvocationError::Internal)?;
            if !manifest.capabilities().contains(&declared_capability) {
                return Err(OpportunityInvocationError::Internal);
            }
            let (target, repository, grant) = authority_for_ids(
                tenant_id.clone(),
                user_id.clone(),
                enabled,
                grant_active,
                manifest.package_id().clone(),
                manifest.package_version().clone(),
                manifest.package_digest().clone(),
                manifest.capability_manifest_digest().clone(),
                combined_source_policy_digest.clone(),
                &metadata,
            )?;
            if entries
                .insert(
                    metadata.suffix,
                    Mutex::new(OpportunityAuthorityEntry {
                        target,
                        repository,
                        grant,
                    }),
                )
                .is_some()
            {
                return Err(OpportunityInvocationError::Internal);
            }
        }
        Ok(Self { entries, mutation })
    }

    pub(crate) fn authorize(
        &self,
        actor: &M00AdmittedActor,
        metadata: &OpportunityOperationMetadata,
    ) -> Result<(), OpportunityInvocationError> {
        let M00AdmittedActor::Authenticated(ids) = actor else {
            return Err(OpportunityInvocationError::Denied);
        };
        let mut entry = self
            .entries
            .get(metadata.suffix)
            .ok_or(OpportunityInvocationError::Internal)?
            .lock()
            .map_err(|_| OpportunityInvocationError::Unavailable)?;
        self.mutate_before_authorization(&mut entry)?;
        InvocationAuthorityService::new(BorrowedAuthorityRepository(&entry.repository))
            .authorize_static_application_use_case(
                ids.tenant_id(),
                ids.user_id(),
                &entry.target,
                metadata.capability_class,
            )
            .map_err(map_authorization_error)
    }

    fn mutate_before_authorization(
        &self,
        entry: &mut OpportunityAuthorityEntry,
    ) -> Result<(), OpportunityInvocationError> {
        if self.mutation != OpportunityAuthorityMutationMode::RevokeGrantBeforeAuthorization {
            return Ok(());
        }
        let mut revoked = entry.grant.clone();
        revoked.state = GrantState::Revoked;
        entry
            .repository
            .replace_grant_for_testing(revoked.clone(), true)
            .map_err(|_| OpportunityInvocationError::Unavailable)?;
        entry.grant = revoked;
        Ok(())
    }
}

fn map_authorization_error(error: InvocationRecheckError) -> OpportunityInvocationError {
    match error {
        InvocationRecheckError::Authorization(_) => OpportunityInvocationError::Denied,
        InvocationRecheckError::Repository(_) => OpportunityInvocationError::Unavailable,
    }
}

#[allow(clippy::too_many_arguments)]
fn authority_for_ids(
    tenant_id: TenantId,
    user_id: UserId,
    enabled: bool,
    grant_active: bool,
    package_id: PackageId,
    package_version: PackageVersion,
    package_digest: Sha256Digest,
    capability_manifest_digest: Sha256Digest,
    source_policy_digest: Sha256Digest,
    metadata: &OpportunityOperationMetadata,
) -> Result<
    (
        InvocationTarget,
        InMemoryInvocationAuthorityRepository,
        CapabilityGrantSnapshot,
    ),
    OpportunityInvocationError,
> {
    let installation_id = InstallationId::parse("installation:ustc-opportunity-graph")
        .map_err(|_| OpportunityInvocationError::Internal)?;
    let component_id = ComponentId::parse("component:opportunity-course-planning-resource-pack")
        .map_err(|_| OpportunityInvocationError::Internal)?;
    let component_version = ComponentVersion::parse("component-version:1")
        .map_err(|_| OpportunityInvocationError::Internal)?;
    let resource_identity = ExecutionIdentity::parse("resource:course-planning-demo-reviewed-v1")
        .map_err(|_| OpportunityInvocationError::Internal)?;
    let operation_locator =
        ToolId::parse(metadata.operation_id).map_err(|_| OpportunityInvocationError::Internal)?;
    let capability_id = CapabilityId::parse(metadata.capability_id)
        .map_err(|_| OpportunityInvocationError::Internal)?;
    let object_scope = ObjectScope::parse("scope:tenant-private-owner")
        .map_err(|_| OpportunityInvocationError::Internal)?;
    let source_policy = SourcePolicyIdentity {
        id: SourcePolicyId::parse("source-policy:opportunity-package-manifest-v1")
            .map_err(|_| OpportunityInvocationError::Internal)?,
        digest: source_policy_digest,
    };
    let component_digest = Sha256Digest::from_bytes(OPPORTUNITY_RESOURCE_COMPONENT_DESCRIPTOR);
    let installation = PluginInstallationSnapshot {
        id: installation_id.clone(),
        tenant_id: tenant_id.clone(),
        user_id: user_id.clone(),
        package_id: package_id.clone(),
        package_version: package_version.clone(),
        package_digest: package_digest.clone(),
        component: InstalledComponentIdentity {
            id: component_id.clone(),
            version: component_version.clone(),
            digest: component_digest.clone(),
            execution_identity: resource_identity.clone(),
        },
        state: if enabled {
            InstallationState::Enabled
        } else {
            InstallationState::Disabled
        },
        revision: InstallationRevision::parse("installation-revision:1")
            .map_err(|_| OpportunityInvocationError::Internal)?,
    };
    let grant = CapabilityGrantSnapshot {
        snapshot_id: GrantSnapshotId::parse(format!(
            "grant-snapshot:opportunity:{}",
            metadata.suffix
        ))
        .map_err(|_| OpportunityInvocationError::Internal)?,
        version: GrantVersion::parse("grant-version:1")
            .map_err(|_| OpportunityInvocationError::Internal)?,
        tenant_id: tenant_id.clone(),
        user_id: user_id.clone(),
        installation_id: installation_id.clone(),
        capability_id: capability_id.clone(),
        object_scope: object_scope.clone(),
        confirmation_policy: ConfirmationPolicy::Ask,
        capability_manifest_digest: capability_manifest_digest.clone(),
        state: if grant_active {
            GrantState::Active
        } else {
            GrantState::Revoked
        },
    };
    let policy = InvocationPolicySnapshot {
        snapshot_id: PolicySnapshotId::parse(format!(
            "policy-snapshot:opportunity:{}",
            metadata.suffix
        ))
        .map_err(|_| OpportunityInvocationError::Internal)?,
        revision: PolicyRevision::parse("policy-revision:1")
            .map_err(|_| OpportunityInvocationError::Internal)?,
        capability_id: capability_id.clone(),
        capability_class: Some(metadata.capability_class),
        admitted_execution_identity: None,
        admitted_source_policy: Some(source_policy.clone()),
        emergency_blocked: false,
    };
    let target = InvocationTarget {
        installation_id: installation_id.clone(),
        package_id: package_id.clone(),
        package_version: package_version.clone(),
        component_id: component_id.clone(),
        tool_id: operation_locator,
        capability_id: capability_id.clone(),
        object_scope,
    };
    let catalog = CatalogPackageRevision {
        catalog_revision: CatalogRevision::parse("catalog-revision:opportunity-demo")
            .map_err(|_| OpportunityInvocationError::Internal)?,
        package_id,
        package_version,
        package_digest,
        runnable: true,
        revoked: false,
        capability_manifest_digest,
        source_policy: Some(source_policy),
        component: Some(CatalogComponentRevision {
            id: component_id,
            kind: ComponentKind::DeclarativeResourcePack,
            version: component_version,
            digest: component_digest,
            execution_identity: resource_identity,
            declared_capabilities: BTreeSet::from([capability_id]),
            tool: None,
        }),
    };
    let repository = InMemoryInvocationAuthorityRepository::try_new(
        vec![(target.clone(), catalog)],
        vec![installation],
        vec![grant.clone()],
        vec![grant.snapshot_id.clone()],
        vec![policy],
    )
    .map_err(|_| OpportunityInvocationError::Unavailable)?;
    Ok((target, repository, grant))
}
