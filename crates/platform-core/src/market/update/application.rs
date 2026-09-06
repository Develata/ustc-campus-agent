//! Trusted bounded B6 application admission and replayable coupled update journal.
//! Storage frames record preconditions and observed evidence; they are never an import API.
use super::*;
use crate::invocation::{PolicyRevision, PolicySnapshotId, SourcePolicyId};
use crate::market::{
    admission::ComponentReadiness,
    capability::load_capability_registry,
    configuration_catalog::{ValidatedPackageConfiguration, load_package_configuration},
    grant::persistence as gp,
    installation::persistence as ip,
    load_package_manifest,
};
use serde::{Deserialize, Serialize};
const MAX_BYTES: usize = 16 * 1024 * 1024;
const MAX_FRAMES: usize = 64;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpdateApplicationError {
    Invalid,
    Conflict,
    NotReady,
    Capacity,
    Corrupt,
}
type Result<T> = std::result::Result<T, UpdateApplicationError>;
fn checked<T, E>(value: std::result::Result<T, E>) -> Result<T> {
    value.map_err(|_| UpdateApplicationError::Invalid)
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReviewedUpdatePackage {
    manifest: String,
    configuration: String,
    revision: String,
}
impl ReviewedUpdatePackage {
    pub fn new(manifest: &[u8], configuration: &[u8], revision: &CatalogRevision) -> Result<Self> {
        if manifest.len() > 65536 || configuration.len() > 65536 {
            return Err(UpdateApplicationError::Capacity);
        }
        let value = Self {
            manifest: checked(String::from_utf8(manifest.to_vec()))?,
            configuration: checked(String::from_utf8(configuration.to_vec()))?,
            revision: revision.as_str().to_owned(),
        };
        value.load()?;
        Ok(value)
    }
    fn load(
        &self,
    ) -> Result<(
        ValidatedPackageManifest,
        ValidatedPackageConfiguration,
        CatalogReadModel,
        Vec<CatalogPackageRevision>,
    )> {
        let manifest = checked(load_package_manifest(self.manifest.as_bytes()))?;
        let revision = checked(CatalogRevision::parse(self.revision.clone()))?;
        let configuration = checked(load_package_configuration(
            self.configuration.as_bytes(),
            &manifest,
            &revision,
        ))?;
        let catalog = checked(CatalogReadModel::new(
            revision.clone(),
            vec![manifest.clone()],
        ))?;
        let source = SourcePolicyIdentity {
            id: checked(SourcePolicyId::parse("source-policy:reviewed-package"))?,
            digest: manifest.source_policy_digest().clone(),
        };
        let publications = configuration
            .package_pin()
            .components()
            .iter()
            .map(|component| CatalogPackageRevision {
                catalog_revision: revision.clone(),
                package_id: manifest.package_id().clone(),
                package_version: manifest.package_version().clone(),
                package_digest: manifest.package_digest().clone(),
                runnable: true,
                revoked: false,
                capability_manifest_digest: manifest.capability_manifest_digest().clone(),
                source_policy: Some(source.clone()),
                component: Some(CatalogComponentRevision {
                    id: component.component_id().clone(),
                    kind: component.kind(),
                    version: component.version().clone(),
                    digest: component.digest().clone(),
                    execution_identity: component.execution_identity().clone(),
                    declared_capabilities: manifest.capabilities().iter().cloned().collect(),
                    tool: None,
                }),
            })
            .collect();
        Ok((manifest, configuration, catalog, publications))
    }
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "action", deny_unknown_fields)]
enum Operation {
    Apply {
        rollback: ReviewedUpdatePackage,
        target: Box<ReviewedUpdatePackage>,
        registry: String,
        expected_plan: String,
        target_readiness: String,
        rollback_readiness: String,
    },
    Rollback {
        readiness: String,
    },
    Confirm {},
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Frame {
    request_id: String,
    intent_digest: String,
    update_id: String,
    installation_id: String,
    tenant: String,
    user: String,
    installations: String,
    grants: String,
    operation: Operation,
    event_digest: String,
}
#[derive(Clone, Default)]
pub struct UpdateJournal {
    frames: Vec<Frame>,
}
#[derive(Clone)]
pub struct UpdateApplicationView {
    pub update_id: String,
    pub installation_id: String,
    pub update_revision: String,
    pub installation_revision: String,
    pub state: String,
    pub rollback_version: String,
    pub target_version: String,
    pub plan_digest: String,
    pub change_class: String,
    pub replayed: bool,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Stored {
    schema: String,
    frames: Vec<Frame>,
    digest: String,
}
impl UpdateJournal {
    pub fn encode(&self) -> Result<Vec<u8>> {
        let bytes = checked(serde_json::to_vec(&self.frames))?;
        if self.frames.len() > MAX_FRAMES || bytes.len() > MAX_BYTES {
            return Err(UpdateApplicationError::Capacity);
        }
        let result = checked(serde_json::to_vec(&Stored {
            schema: "market-update-application/v1".into(),
            frames: self.frames.clone(),
            digest: Sha256Digest::from_bytes(&bytes).as_str().to_owned(),
        }))?;
        if result.len() > MAX_BYTES {
            return Err(UpdateApplicationError::Capacity);
        }
        Ok(result)
    }
    pub fn decode(
        bytes: &[u8],
        installations: &InMemoryInstallationRepository,
        grants: &InMemoryGrantRepository,
    ) -> Result<Self> {
        if bytes.len() > MAX_BYTES {
            return Err(UpdateApplicationError::Capacity);
        }
        let stored: Stored = checked(serde_json::from_slice(bytes))?;
        if stored.schema != "market-update-application/v1"
            || stored.frames.len() > MAX_FRAMES
            || Sha256Digest::from_bytes(&checked(serde_json::to_vec(&stored.frames))?).as_str()
                != stored.digest
        {
            return Err(UpdateApplicationError::Corrupt);
        }
        let journal = Self {
            frames: stored.frames,
        };
        journal.runtime(installations, grants)?;
        Ok(journal)
    }
    fn runtime(
        &self,
        installations: &InMemoryInstallationRepository,
        grants: &InMemoryGrantRepository,
    ) -> Result<InMemoryPackageUpdateRepository> {
        let mut runtime = InMemoryPackageUpdateRepository::new();
        for frame in &self.frames {
            let before_i = checked(ip::encode_snapshot(&runtime.installation_repository))?;
            let before_g = checked(gp::encode_snapshot(&runtime.grant_repository))?;
            ip::ordinary_extension(&before_i, frame.installations.as_bytes())
                .map_err(|_| UpdateApplicationError::Corrupt)?;
            gp::ordinary_extension(&before_g, frame.grants.as_bytes())
                .map_err(|_| UpdateApplicationError::Corrupt)?;
            runtime.installation_repository =
                checked(ip::decode_snapshot(frame.installations.as_bytes()))?;
            runtime.grant_repository = checked(gp::decode_snapshot(frame.grants.as_bytes()))?;
            let view = run_frame(&mut runtime, frame)?;
            let id = checked(PackageUpdateId::parse(view.update_id))?;
            if runtime
                .events
                .get(&id)
                .and_then(|events| events.last())
                .map(|event| event.event_digest().as_str())
                != Some(frame.event_digest.as_str())
            {
                return Err(UpdateApplicationError::Corrupt);
            }
        }
        ip::ordinary_extension(
            &checked(ip::encode_snapshot(&runtime.installation_repository))?,
            &checked(ip::encode_snapshot(installations))?,
        )
        .map_err(|_| UpdateApplicationError::Corrupt)?;
        gp::ordinary_extension(
            &checked(gp::encode_snapshot(&runtime.grant_repository))?,
            &checked(gp::encode_snapshot(grants))?,
        )
        .map_err(|_| UpdateApplicationError::Corrupt)?;
        runtime.installation_repository = installations.clone();
        runtime.grant_repository = grants.clone();
        Ok(runtime)
    }
    pub fn contains_request(&self, request_id: &str) -> bool {
        self.frames
            .iter()
            .any(|frame| frame.request_id == request_id)
    }
    pub fn lookup(
        &self,
        request_id: &str,
        intent_digest: &str,
        tenant: &TenantId,
        user: &UserId,
    ) -> Result<Option<UpdateApplicationView>> {
        let Some(index) = self
            .frames
            .iter()
            .position(|frame| frame.request_id == request_id)
        else {
            return Ok(None);
        };
        let frame = &self.frames[index];
        if frame.intent_digest != intent_digest
            || frame.tenant != tenant.as_str()
            || frame.user != user.as_str()
        {
            return Err(UpdateApplicationError::Conflict);
        }
        let mut runtime = InMemoryPackageUpdateRepository::new();
        for frame in &self.frames[..=index] {
            runtime.installation_repository =
                checked(ip::decode_snapshot(frame.installations.as_bytes()))?;
            runtime.grant_repository = checked(gp::decode_snapshot(frame.grants.as_bytes()))?;
            run_frame(&mut runtime, frame)?;
        }
        let aggregate = runtime
            .aggregates
            .get(&checked(PackageUpdateId::parse(frame.update_id.clone()))?)
            .ok_or(UpdateApplicationError::Corrupt)?;
        Ok(Some(view(&runtime, aggregate, true)?))
    }
    pub fn list(
        &self,
        installations: &InMemoryInstallationRepository,
        grants: &InMemoryGrantRepository,
        tenant: &TenantId,
        user: &UserId,
    ) -> Result<Vec<UpdateApplicationView>> {
        let runtime = self.runtime(installations, grants)?;
        runtime
            .aggregates
            .values()
            .filter(|aggregate| aggregate.tenant_id() == tenant && aggregate.user_id() == user)
            .map(|aggregate| view(&runtime, aggregate, false))
            .collect()
    }
    // The two original repositories and exact reviewed authorities stay explicit at this trusted port.
    #[allow(clippy::too_many_arguments)]
    pub fn preview(
        &self,
        installations: &InMemoryInstallationRepository,
        grants: &InMemoryGrantRepository,
        tenant: &TenantId,
        user: &UserId,
        id: &InstallationId,
        rollback: &ReviewedUpdatePackage,
        target: &ReviewedUpdatePackage,
        registry: &[u8],
    ) -> Result<UpdateApplicationView> {
        let mut runtime = self.runtime(installations, grants)?;
        let current = owned(&runtime, tenant, user, id)?;
        let update_id = PackageUpdateId::parse(format!(
            "update:{}",
            Sha256Digest::from_bytes(
                format!(
                    "{}|{}|{}|{}",
                    tenant.as_str(),
                    user.as_str(),
                    id.as_str(),
                    current.revision().as_str()
                )
                .as_bytes()
            )
            .as_str()
            .trim_start_matches("sha256:")
        ))
        .map_err(|_| UpdateApplicationError::Invalid)?;
        let command = stage(
            &mut runtime,
            &update_id,
            &current,
            rollback,
            target,
            registry,
        )?;
        accept(&mut runtime, command)?;
        view(
            &runtime,
            runtime
                .aggregates
                .get(&update_id)
                .ok_or(UpdateApplicationError::Corrupt)?,
            false,
        )
    }
    #[allow(clippy::too_many_arguments)]
    pub fn apply(
        &mut self,
        installations: &mut InMemoryInstallationRepository,
        grants: &mut InMemoryGrantRepository,
        tenant: &TenantId,
        user: &UserId,
        id: &InstallationId,
        expected_revision: &InstallationRevision,
        request_id: &str,
        intent_digest: &str,
        update_id: &str,
        expected_plan: &str,
        rollback: ReviewedUpdatePackage,
        target: ReviewedUpdatePackage,
        registry: &[u8],
        rollback_readiness: &ComponentReadiness,
        target_readiness: &ComponentReadiness,
    ) -> Result<UpdateApplicationView> {
        if let Some(view) = self.lookup(request_id, intent_digest, tenant, user)? {
            return Ok(view);
        }
        let runtime = self.runtime(installations, grants)?;
        let current = owned(&runtime, tenant, user, id)?;
        if current.revision() != expected_revision {
            return Err(UpdateApplicationError::Conflict);
        }
        let (_, rollback_config, _, _) = rollback.load()?;
        let (_, target_config, _, _) = target.load()?;
        if current.package_pin() != rollback_config.package_pin()
            || !rollback_readiness.matches_package(&rollback_config, current.configuration())
            || !target_readiness.matches_package(&target_config, current.configuration())
        {
            return Err(UpdateApplicationError::NotReady);
        }
        let operation = Operation::Apply {
            rollback,
            target: Box::new(target),
            registry: checked(String::from_utf8(registry.to_vec()))?,
            expected_plan: expected_plan.into(),
            target_readiness: target_readiness.digest().as_str().into(),
            rollback_readiness: rollback_readiness.digest().as_str().into(),
        };
        self.execute_frame(
            installations,
            grants,
            tenant,
            user,
            id,
            request_id,
            intent_digest,
            update_id,
            operation,
        )
    }
    #[allow(clippy::too_many_arguments)]
    pub fn finish(
        &mut self,
        installations: &mut InMemoryInstallationRepository,
        grants: &mut InMemoryGrantRepository,
        tenant: &TenantId,
        user: &UserId,
        id: &InstallationId,
        expected_revision: &InstallationRevision,
        request_id: &str,
        intent_digest: &str,
        update_id: &str,
        rollback_readiness: Option<&ComponentReadiness>,
    ) -> Result<UpdateApplicationView> {
        if let Some(view) = self.lookup(request_id, intent_digest, tenant, user)? {
            return Ok(view);
        }
        let runtime = self.runtime(installations, grants)?;
        let current = owned(&runtime, tenant, user, id)?;
        if current.revision() != expected_revision {
            return Err(UpdateApplicationError::Conflict);
        }
        let aggregate = runtime
            .aggregates
            .get(&checked(PackageUpdateId::parse(update_id))?)
            .ok_or(UpdateApplicationError::Invalid)?;
        if aggregate.installation_id() != id {
            return Err(UpdateApplicationError::Invalid);
        }
        let operation = if let Some(readiness) = rollback_readiness {
            let source = self
                .frames
                .iter()
                .find_map(|frame| match &frame.operation {
                    Operation::Apply { rollback, .. } if frame.update_id == update_id => {
                        Some(rollback)
                    }
                    _ => None,
                })
                .ok_or(UpdateApplicationError::Corrupt)?;
            let (_, configuration, _, _) = source.load()?;
            if !readiness.matches_package(&configuration, current.configuration()) {
                return Err(UpdateApplicationError::NotReady);
            }
            Operation::Rollback {
                readiness: readiness.digest().as_str().into(),
            }
        } else {
            Operation::Confirm {}
        };
        self.execute_frame(
            installations,
            grants,
            tenant,
            user,
            id,
            request_id,
            intent_digest,
            update_id,
            operation,
        )
    }
    #[allow(clippy::too_many_arguments)]
    fn execute_frame(
        &mut self,
        installations: &mut InMemoryInstallationRepository,
        grants: &mut InMemoryGrantRepository,
        tenant: &TenantId,
        user: &UserId,
        id: &InstallationId,
        request_id: &str,
        intent_digest: &str,
        update_id: &str,
        operation: Operation,
    ) -> Result<UpdateApplicationView> {
        if request_id.len() > 128 || request_id.is_empty() {
            return Err(UpdateApplicationError::Invalid);
        }
        if self
            .frames
            .iter()
            .any(|frame| frame.request_id == request_id)
        {
            return Err(UpdateApplicationError::Conflict);
        }
        if self.frames.len() >= MAX_FRAMES {
            return Err(UpdateApplicationError::Capacity);
        }
        let mut runtime = self.runtime(installations, grants)?;
        let mut frame = Frame {
            request_id: request_id.into(),
            intent_digest: intent_digest.into(),
            update_id: update_id.into(),
            installation_id: id.as_str().into(),
            tenant: tenant.as_str().into(),
            user: user.as_str().into(),
            installations: checked(String::from_utf8(checked(ip::encode_snapshot(
                installations,
            ))?))?,
            grants: checked(String::from_utf8(checked(gp::encode_snapshot(grants))?))?,
            operation,
            event_digest: String::new(),
        };
        let result = run_frame(&mut runtime, &frame)?;
        frame.event_digest = runtime
            .events
            .get(&checked(PackageUpdateId::parse(update_id))?)
            .and_then(|events| events.last())
            .ok_or(UpdateApplicationError::Corrupt)?
            .event_digest()
            .as_str()
            .to_owned();
        let mut next = self.clone();
        next.frames.push(frame);
        next.encode()?;
        *installations = runtime.installation_repository;
        *grants = runtime.grant_repository;
        *self = next;
        Ok(result)
    }
}
fn owned(
    runtime: &InMemoryPackageUpdateRepository,
    tenant: &TenantId,
    user: &UserId,
    id: &InstallationId,
) -> Result<InstallationSnapshot> {
    checked(runtime.installation_repository.load_exact(id))?
        .filter(|current| current.tenant_id() == tenant && current.user_id() == user)
        .ok_or(UpdateApplicationError::Invalid)
}
fn accept(runtime: &mut InMemoryPackageUpdateRepository, command: UpdateCommand) -> Result<()> {
    let receipt = checked(runtime.execute(command))?;
    if !matches!(receipt.outcome(), UpdateCommandOutcome::Accepted { .. }) {
        return Err(UpdateApplicationError::Conflict);
    }
    Ok(())
}
fn stage(
    runtime: &mut InMemoryPackageUpdateRepository,
    id: &PackageUpdateId,
    current: &InstallationSnapshot,
    rollback: &ReviewedUpdatePackage,
    target: &ReviewedUpdatePackage,
    registry: &[u8],
) -> Result<UpdateCommand> {
    let (_, old, old_catalog, old_publications) = rollback.load()?;
    let (_, next, next_catalog, next_publications) = target.load()?;
    if current.package_pin() != old.package_pin() {
        return Err(UpdateApplicationError::Conflict);
    }
    for binding in next.bindings().values() {
        checked(binding.validate(next.package_pin(), current.configuration()))?;
    }
    let registry = checked(load_capability_registry(registry))?;
    let command = checked(UpdateCommand::stage(
        checked(UpdateCommandId::parse(format!(
            "update-cmd:stage-{}",
            id.as_str().trim_start_matches("update:")
        )))?,
        id.clone(),
        current,
        next.package_pin().clone(),
        &old_catalog,
        &old_publications,
        &next_catalog,
        &next_publications,
        &registry,
        &registry,
    ))?;
    let UpdateCommandAction::Stage { plan } = &command.action else {
        return Err(UpdateApplicationError::Corrupt);
    };
    for catalog in [old_catalog, next_catalog] {
        if let Some(existing) = runtime
            .catalog_read_models
            .iter()
            .find(|existing| existing.catalog_revision() == catalog.catalog_revision())
        {
            if existing.catalog_digest() != catalog.catalog_digest() {
                return Err(UpdateApplicationError::Conflict);
            }
        } else {
            runtime.catalog_read_models.push(catalog);
        }
    }
    for publication in old_publications.into_iter().chain(next_publications) {
        if !runtime.catalog_publications.contains(&publication) {
            runtime.catalog_publications.push(publication);
        }
    }
    if !runtime
        .capability_registries
        .iter()
        .any(|old| old.registry_revision() == registry.registry_revision())
    {
        runtime.capability_registries.push(registry);
    }
    for ((capability, execution, source_id, source_digest), class) in required_policy_bindings(plan)
    {
        if class != Some(CapabilityClass::PublicRead) {
            return Err(UpdateApplicationError::Invalid);
        }
        let identity = Sha256Digest::from_bytes(
            format!("{capability}|{execution}|{source_id}|{source_digest}").as_bytes(),
        );
        let policy = InvocationPolicySnapshot {
            snapshot_id: checked(PolicySnapshotId::parse(format!(
                "policy-snapshot:{}",
                identity.as_str().trim_start_matches("sha256:")
            )))?,
            revision: checked(PolicyRevision::parse("policy-revision:1"))?,
            capability_id: checked(CapabilityId::parse(capability))?,
            capability_class: class,
            admitted_execution_identity: Some(checked(ExecutionIdentity::parse(execution))?),
            admitted_source_policy: Some(SourcePolicyIdentity {
                id: checked(SourcePolicyId::parse(source_id))?,
                digest: checked(Sha256Digest::parse(source_digest))?,
            }),
            emergency_blocked: false,
        };
        if !runtime.policy_snapshots.contains(&policy) {
            runtime.policy_snapshots.push(policy);
        }
    }
    Ok(command)
}
fn run_frame(
    runtime: &mut InMemoryPackageUpdateRepository,
    frame: &Frame,
) -> Result<UpdateApplicationView> {
    let id = checked(PackageUpdateId::parse(frame.update_id.clone()))?;
    let current = owned(
        runtime,
        &checked(TenantId::parse(frame.tenant.clone()))?,
        &checked(UserId::parse(frame.user.clone()))?,
        &checked(InstallationId::parse(frame.installation_id.clone()))?,
    )?;
    if matches!(
        &frame.operation,
        Operation::Apply { .. } | Operation::Rollback { .. }
    ) {
        let grants = checked(runtime.grant_repository.load_current_for_installation(
            current.tenant_id(),
            current.user_id(),
            current.installation_id(),
            current.revision(),
        ))?;
        for grant in grants.grants() {
            if grant.state() == GrantState::Active
                && grant.installation_revision() != current.revision()
            {
                let identity = Sha256Digest::from_bytes(
                    format!("{}|{}", frame.request_id, grant.snapshot_id().as_str()).as_bytes(),
                );
                let command = checked(GrantCommand::mark_stale(
                    checked(GrantCommandId::parse(format!(
                        "grant-cmd:pre-update-{}",
                        identity.as_str().trim_start_matches("sha256:")
                    )))?,
                    grant.snapshot_id().clone(),
                    grant.version().clone(),
                    GrantInvalidationReason::InstallationChanged,
                ))?;
                let receipt = checked(runtime.grant_repository.execute(command))?;
                if !matches!(receipt.outcome(), GrantCommandOutcome::Accepted { .. }) {
                    return Err(UpdateApplicationError::Conflict);
                }
            }
        }
    }
    let command_id = |label: &str| {
        checked(UpdateCommandId::parse(format!(
            "update-cmd:{label}-{}",
            frame.request_id
        )))
    };
    let evidence_id = || {
        checked(UpdateEvidenceId::parse(format!(
            "update-evidence:{}",
            frame.request_id
        )))
    };
    match &frame.operation {
        Operation::Apply {
            rollback,
            target,
            registry,
            expected_plan,
            target_readiness,
            rollback_readiness,
        } => {
            let command = stage(
                runtime,
                &id,
                &current,
                rollback,
                target,
                registry.as_bytes(),
            )?;
            let UpdateCommandAction::Stage { plan } = &command.action else {
                return Err(UpdateApplicationError::Corrupt);
            };
            if plan.plan_digest().as_str() != expected_plan {
                return Err(UpdateApplicationError::Conflict);
            }
            accept(runtime, command)?;
            let aggregate = runtime
                .aggregates
                .get(&id)
                .ok_or(UpdateApplicationError::Corrupt)?
                .clone();
            let plan = aggregate.plan();
            let policy_digest = digest_policy_snapshots(&checked(runtime.policies_for_plan(plan))?);
            let approval = checked(UpdateApprovalEvidence::from_plan(
                checked(UpdateApprovalId::parse(format!(
                    "update-approval:{}",
                    frame.request_id
                )))?,
                plan,
                Sha256Digest::from_bytes(frame.request_id.as_bytes()),
            ))?;
            let readiness = checked(UpdateReadinessEvidence::from_plan(
                evidence_id()?,
                plan,
                checked(Sha256Digest::parse(target_readiness.clone()))?,
                checked(Sha256Digest::parse(rollback_readiness.clone()))?,
                current.configuration().digest().clone(),
                policy_digest,
            ))?;
            accept(
                runtime,
                checked(UpdateCommand::record_approval(
                    command_id("approve")?,
                    id.clone(),
                    aggregate.revision().clone(),
                    approval,
                    readiness,
                ))?,
            )?;
            let revision = runtime
                .aggregates
                .get(&id)
                .ok_or(UpdateApplicationError::Corrupt)?
                .revision()
                .clone();
            accept(
                runtime,
                checked(UpdateCommand::apply(
                    command_id("apply")?,
                    id.clone(),
                    revision,
                    current.revision().clone(),
                ))?,
            )?;
        }
        Operation::Rollback { readiness } => {
            let aggregate = runtime
                .aggregates
                .get(&id)
                .ok_or(UpdateApplicationError::Invalid)?
                .clone();
            let policy_digest =
                digest_policy_snapshots(&checked(runtime.policies_for_plan(aggregate.plan()))?);
            let evidence = checked(RollbackReadinessEvidence::from_bindings(
                evidence_id()?,
                id.clone(),
                aggregate.revision().clone(),
                digest_pin_value(aggregate.plan().rollback_pin()),
                current.revision().clone(),
                current.configuration_revision(),
                current.configuration().digest().clone(),
                checked(Sha256Digest::parse(readiness.clone()))?,
                policy_digest,
            ))?;
            accept(
                runtime,
                checked(UpdateCommand::rollback(
                    command_id("rollback")?,
                    id.clone(),
                    aggregate.revision().clone(),
                    current.revision().clone(),
                    evidence,
                ))?,
            )?;
        }
        Operation::Confirm {} => {
            let aggregate = runtime
                .aggregates
                .get(&id)
                .ok_or(UpdateApplicationError::Invalid)?
                .clone();
            let evidence = checked(UpdateConfirmationEvidence::from_bindings(
                evidence_id()?,
                id.clone(),
                aggregate.revision().clone(),
                aggregate
                    .applied_event_digest
                    .clone()
                    .ok_or(UpdateApplicationError::Invalid)?,
                current.installation_id().clone(),
                current.revision().clone(),
                digest_pin_value(aggregate.plan().target_pin()),
                digest_installation_state_binding(&current),
            ))?;
            accept(
                runtime,
                checked(UpdateCommand::confirm_applied_update(
                    command_id("confirm")?,
                    id.clone(),
                    aggregate.revision().clone(),
                    current.revision().clone(),
                    evidence,
                ))?,
            )?;
        }
    }
    view(
        runtime,
        runtime
            .aggregates
            .get(&id)
            .ok_or(UpdateApplicationError::Corrupt)?,
        false,
    )
}
fn view(
    runtime: &InMemoryPackageUpdateRepository,
    aggregate: &PackageUpdateAggregate,
    replayed: bool,
) -> Result<UpdateApplicationView> {
    let current = checked(
        runtime
            .installation_repository
            .load_exact(aggregate.installation_id()),
    )?
    .ok_or(UpdateApplicationError::Corrupt)?;
    Ok(UpdateApplicationView {
        update_id: aggregate.update_id().as_str().into(),
        installation_id: aggregate.installation_id().as_str().into(),
        update_revision: aggregate.revision().as_str().into(),
        installation_revision: current.revision().as_str().into(),
        state: format!("{:?}", aggregate.state()).to_lowercase(),
        rollback_version: aggregate.plan().rollback_pin().package_version().as_str(),
        target_version: aggregate.plan().target_pin().package_version().as_str(),
        plan_digest: aggregate.plan().plan_digest().as_str().into(),
        change_class: format!("{:?}", aggregate.plan().change_class()).to_lowercase(),
        replayed,
    })
}
