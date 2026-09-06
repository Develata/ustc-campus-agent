//! Exact B6 update intent over the original M20 admission and coupled journal.
use super::*;
use ustc_campus_agent_core::market::update::application::{
    ReviewedUpdatePackage, UpdateApplicationError, UpdateApplicationView,
};
const REGISTRY: &[u8] = include_bytes!("../../../../market/capabilities/registry.json");
pub(super) fn error(error: UpdateApplicationError) -> PluginError {
    match error {
        UpdateApplicationError::Invalid => PluginError::InvalidRequest,
        UpdateApplicationError::Conflict => PluginError::Conflict,
        UpdateApplicationError::NotReady => PluginError::NotReady,
        UpdateApplicationError::Capacity => PluginError::Capacity,
        UpdateApplicationError::Corrupt => PluginError::Unavailable,
    }
}
pub(super) fn view(value: UpdateApplicationView) -> PluginUpdateViewDto {
    PluginUpdateViewDto {
        schema: "plugin-update-view/v1",
        update_id: value.update_id,
        installation_id: value.installation_id,
        update_revision: value.update_revision,
        installation_revision: value.installation_revision,
        state: value.state,
        rollback_version: value.rollback_version,
        target_version: value.target_version,
        plan_digest: value.plan_digest,
        change_class: value.change_class,
        replayed: value.replayed,
        target_readiness: None,
        rollback_readiness: None,
    }
}
fn archive(package: &RuntimePackage) -> Result<ReviewedUpdatePackage, PluginError> {
    ReviewedUpdatePackage::new(
        &package.manifest_source,
        &package.configuration_source,
        package.configuration.package_pin().catalog_revision(),
    )
    .map_err(error)
}
impl PluginRuntime {
    async fn version_readiness(
        &self,
        tenant: &TenantId,
        user: &UserId,
        current: &InstallationSnapshot,
        package: &RuntimePackage,
    ) -> Result<ComponentReadiness, PluginError> {
        let mut temporary = InMemoryInstallationRepository::new();
        let command = InstallationCommand::install(
            InstallationCommandId::parse("cmd:update-probe".to_owned())
                .map_err(|_| PluginError::Unavailable)?,
            current.installation_id().clone(),
            tenant.clone(),
            user.clone(),
            package.configuration.package_pin().clone(),
            current.configuration().clone(),
        )
        .map_err(|_| PluginError::InvalidRequest)?;
        let receipt = temporary
            .execute(command)
            .map_err(|_| PluginError::InvalidRequest)?;
        let InstallationCommandOutcome::Accepted { snapshot, .. } = receipt.outcome() else {
            return Err(PluginError::InvalidRequest);
        };
        let probe = self.build_probe(tenant, user, snapshot, package).await?;
        let readiness = probe.readiness.clone();
        retire::component(probe).await;
        Ok(readiness)
    }
    pub(crate) async fn update(
        &self,
        tenant: &TenantId,
        user: &UserId,
        request: PluginUpdateDto,
    ) -> Result<PluginUpdateViewDto, PluginError> {
        if request.schema != "plugin-update/v1"
            || request.request_id.is_empty()
            || request.request_id.len() > 80
            || !request
                .request_id
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"_-".contains(&b))
        {
            return Err(PluginError::InvalidRequest);
        }
        let identity = stable_id(&[tenant.as_str(), user.as_str(), &request.request_id]);
        let intent_bytes =
            serde_json::to_vec(&request.intent).map_err(|_| PluginError::InvalidRequest)?;
        let digest = Sha256Digest::from_bytes(&intent_bytes);
        let mut state = self.state.lock().await;
        state.check()?;
        let installation_command = InstallationCommandId::parse(format!("cmd:{identity}"))
            .map_err(|_| PluginError::InvalidRequest)?;
        let grant_command = GrantCommandId::parse(format!("grant-cmd:{identity}"))
            .map_err(|_| PluginError::InvalidRequest)?;
        if state
            .authority
            .installations
            .lookup_owned_receipt(tenant, user, &installation_command)
            .is_some()
            || state
                .authority
                .grants
                .lookup_owned_receipt(tenant, user, &grant_command)
                .is_some()
        {
            return Err(PluginError::Conflict);
        }
        if let Some(previous) = state
            .authority
            .updates
            .lookup(&identity, digest.as_str(), tenant, user)
            .map_err(error)?
        {
            return Ok(view(previous));
        }
        let (id, revision) = match &request.intent {
            PluginUpdateIntentDto::Preview {
                installation_id,
                expected_revision,
                ..
            }
            | PluginUpdateIntentDto::Apply {
                installation_id,
                expected_revision,
                ..
            }
            | PluginUpdateIntentDto::ReviewRollback {
                installation_id,
                expected_revision,
                ..
            }
            | PluginUpdateIntentDto::Rollback {
                installation_id,
                expected_revision,
                ..
            }
            | PluginUpdateIntentDto::Confirm {
                installation_id,
                expected_revision,
                ..
            } => (installation_id, expected_revision),
        };
        let id = InstallationId::parse(id.clone()).map_err(|_| PluginError::InvalidRequest)?;
        let revision = InstallationRevision::parse(revision.clone())
            .map_err(|_| PluginError::InvalidRequest)?;
        let current = state.owned(tenant, user, &id)?;
        if current.revision() != &revision {
            return Err(PluginError::Conflict);
        }
        if !matches!(request.intent, PluginUpdateIntentDto::Confirm { .. })
            && !matches!(
                current.state(),
                ManagedInstallationState::Disabled | ManagedInstallationState::InstalledDisabled
            )
        {
            return Err(PluginError::Denied);
        }
        let mut next = state.authority.clone();
        let result = match request.intent {
            PluginUpdateIntentDto::Preview { target_version, .. } => {
                let rollback = self.package(
                    current.package_pin().package_id().as_str(),
                    &current.package_pin().package_version().as_str(),
                )?;
                let target =
                    self.package(current.package_pin().package_id().as_str(), &target_version)?;
                if state
                    .authority
                    .installations
                    .list_owned(tenant, user)
                    .iter()
                    .any(|installed| {
                        installed.installation_id() != &id
                            && installed.package_pin() == target.configuration.package_pin()
                            && !matches!(
                                installed.state(),
                                ManagedInstallationState::Revoked
                                    | ManagedInstallationState::Uninstalled
                            )
                    })
                {
                    return Err(PluginError::Conflict);
                }

                let plan = state
                    .authority
                    .updates
                    .preview(
                        &state.authority.installations,
                        &state.authority.grants,
                        tenant,
                        user,
                        &id,
                        &archive(rollback)?,
                        &archive(target)?,
                        REGISTRY,
                    )
                    .map_err(error)?;
                let old = self
                    .version_readiness(tenant, user, &current, rollback)
                    .await?;
                let new = self
                    .version_readiness(tenant, user, &current, target)
                    .await?;
                let mut response = view(plan);
                response.target_readiness = Some(new.digest().as_str().to_owned());
                response.rollback_readiness = Some(old.digest().as_str().to_owned());
                return Ok(response);
            }
            PluginUpdateIntentDto::Apply {
                target_version,
                update_id,
                plan_digest,
                target_readiness,
                rollback_readiness,
                ..
            } => {
                let rollback = self.package(
                    current.package_pin().package_id().as_str(),
                    &current.package_pin().package_version().as_str(),
                )?;
                let target =
                    self.package(current.package_pin().package_id().as_str(), &target_version)?;
                if state
                    .authority
                    .installations
                    .list_owned(tenant, user)
                    .iter()
                    .any(|installed| {
                        installed.installation_id() != &id
                            && installed.package_pin() == target.configuration.package_pin()
                            && !matches!(
                                installed.state(),
                                ManagedInstallationState::Revoked
                                    | ManagedInstallationState::Uninstalled
                            )
                    })
                {
                    return Err(PluginError::Conflict);
                }

                let old = self
                    .version_readiness(tenant, user, &current, rollback)
                    .await?;
                let new = self
                    .version_readiness(tenant, user, &current, target)
                    .await?;
                if old.digest().as_str() != rollback_readiness
                    || new.digest().as_str() != target_readiness
                {
                    return Err(PluginError::Conflict);
                }
                next.updates
                    .apply(
                        &mut next.installations,
                        &mut next.grants,
                        tenant,
                        user,
                        &id,
                        &revision,
                        &identity,
                        digest.as_str(),
                        &update_id,
                        &plan_digest,
                        archive(rollback)?,
                        archive(target)?,
                        REGISTRY,
                        &old,
                        &new,
                    )
                    .map_err(error)?
            }
            PluginUpdateIntentDto::ReviewRollback { update_id, .. } => {
                let update = next
                    .updates
                    .list(&next.installations, &next.grants, tenant, user)
                    .map_err(error)?
                    .into_iter()
                    .find(|update| {
                        update.update_id == update_id && update.installation_id == id.as_str()
                    })
                    .ok_or(PluginError::NotFound)?;
                let rollback = self.package(
                    current.package_pin().package_id().as_str(),
                    &update.rollback_version,
                )?;
                if state
                    .authority
                    .installations
                    .list_owned(tenant, user)
                    .iter()
                    .any(|installed| {
                        installed.installation_id() != &id
                            && installed.package_pin() == rollback.configuration.package_pin()
                            && !matches!(
                                installed.state(),
                                ManagedInstallationState::Revoked
                                    | ManagedInstallationState::Uninstalled
                            )
                    })
                {
                    return Err(PluginError::Conflict);
                }

                let old = self
                    .version_readiness(tenant, user, &current, rollback)
                    .await?;
                let mut response = view(update);
                response.rollback_readiness = Some(old.digest().as_str().to_owned());
                return Ok(response);
            }
            PluginUpdateIntentDto::Rollback {
                update_id,
                rollback_readiness,
                ..
            } => {
                let update = next
                    .updates
                    .list(&next.installations, &next.grants, tenant, user)
                    .map_err(error)?
                    .into_iter()
                    .find(|update| {
                        update.update_id == update_id && update.installation_id == id.as_str()
                    })
                    .ok_or(PluginError::NotFound)?;
                let rollback = self.package(
                    current.package_pin().package_id().as_str(),
                    &update.rollback_version,
                )?;
                if state
                    .authority
                    .installations
                    .list_owned(tenant, user)
                    .iter()
                    .any(|installed| {
                        installed.installation_id() != &id
                            && installed.package_pin() == rollback.configuration.package_pin()
                            && !matches!(
                                installed.state(),
                                ManagedInstallationState::Revoked
                                    | ManagedInstallationState::Uninstalled
                            )
                    })
                {
                    return Err(PluginError::Conflict);
                }

                let old = self
                    .version_readiness(tenant, user, &current, rollback)
                    .await?;
                if old.digest().as_str() != rollback_readiness {
                    return Err(PluginError::Conflict);
                }
                next.updates
                    .finish(
                        &mut next.installations,
                        &mut next.grants,
                        tenant,
                        user,
                        &id,
                        &revision,
                        &identity,
                        digest.as_str(),
                        &update_id,
                        Some(&old),
                    )
                    .map_err(error)?
            }
            PluginUpdateIntentDto::Confirm { update_id, .. } => next
                .updates
                .finish(
                    &mut next.installations,
                    &mut next.grants,
                    tenant,
                    user,
                    &id,
                    &revision,
                    &identity,
                    digest.as_str(),
                    &update_id,
                    None,
                )
                .map_err(error)?,
        };
        state.commit(next)?;
        retire::probe(&mut state, &id).await;
        Ok(view(result))
    }
}
