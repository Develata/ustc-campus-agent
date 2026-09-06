use super::*;
use ustc_campus_agent_core::market::admission::{
    EnableAdmissionRequest, GrantAdmissionRequest, MarketAdmissionService,
};
impl PluginRuntime {
    pub(crate) async fn command(
        &self,
        tenant: &TenantId,
        user: &UserId,
        request: PluginCommandDto,
    ) -> Result<PluginCommandResultDto, PluginError> {
        if request.schema != "plugin-command/v1"
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
        let command_id = InstallationCommandId::parse(format!("cmd:{identity}"))
            .map_err(|_| PluginError::InvalidRequest)?;
        let mut state = self.state.lock().await;
        state.check()?;
        let grant_id = GrantCommandId::parse(format!("grant-cmd:{identity}"))
            .map_err(|_| PluginError::InvalidRequest)?;
        if matches!(&request.intent, PluginIntentDto::Grant { .. }) {
            if state
                .authority
                .installations
                .lookup_owned_receipt(tenant, user, &command_id)
                .is_some()
            {
                return Err(PluginError::Conflict);
            }
        } else if state
            .authority
            .grants
            .lookup_owned_receipt(tenant, user, &grant_id)
            .is_some()
        {
            return Err(PluginError::Conflict);
        }
        if let PluginIntentDto::Grant {
            installation_id,
            expected_revision,
            capability,
        } = &request.intent
        {
            return self.grant(
                &mut state,
                tenant,
                user,
                &identity,
                installation_id,
                expected_revision,
                capability,
            );
        }
        let historical =
            state
                .authority
                .installations
                .lookup_owned_receipt(tenant, user, &command_id);
        if let Some(receipt) = historical {
            if !matches_intent(tenant, receipt.command(), &request.intent)? {
                return Err(PluginError::Conflict);
            }
            return Ok(result(&receipt, true));
        }
        let mut next = state.authority.clone();
        let (id, command) = match request.intent {
            PluginIntentDto::Install {
                package_id,
                version,
                catalog_revision,
                package_digest,
            } => {
                let package = self.package(&package_id, &version)?;
                let pin = package.configuration.package_pin();
                if pin.catalog_revision().as_str() != catalog_revision
                    || pin.package_digest().as_str() != package_digest
                {
                    return Err(PluginError::Conflict);
                }
                let id = InstallationId::parse(format!(
                    "installation:{}",
                    stable_id(&[tenant.as_str(), user.as_str(), &package_id, &version])
                ))
                .map_err(|_| PluginError::InvalidRequest)?;
                let config = configuration(tenant, BTreeMap::new())?;
                let command = InstallationCommand::install(
                    command_id,
                    id.clone(),
                    tenant.clone(),
                    user.clone(),
                    pin.clone(),
                    config,
                )
                .map_err(|_| PluginError::InvalidRequest)?;
                (id, command)
            }
            other => {
                let (raw, revision) = match &other {
                    PluginIntentDto::Configure {
                        installation_id,
                        expected_revision,
                        ..
                    }
                    | PluginIntentDto::Enable {
                        installation_id,
                        expected_revision,
                        ..
                    }
                    | PluginIntentDto::Disable {
                        installation_id,
                        expected_revision,
                    }
                    | PluginIntentDto::Revoke {
                        installation_id,
                        expected_revision,
                    } => (installation_id, expected_revision),
                    _ => return Err(PluginError::InvalidRequest),
                };
                let id =
                    InstallationId::parse(raw.clone()).map_err(|_| PluginError::InvalidRequest)?;
                let revision = InstallationRevision::parse(revision.clone())
                    .map_err(|_| PluginError::InvalidRequest)?;
                let current = state.owned(tenant, user, &id)?;
                if current.revision() != &revision {
                    return Err(PluginError::Conflict);
                }
                let command = match other {
                    PluginIntentDto::Configure { values, .. } => {
                        let package = self.package(
                            current.package_pin().package_id().as_str(),
                            &current.package_pin().package_version().as_str(),
                        )?;
                        let config = configuration(tenant, values)?;
                        let binding = package
                            .configuration
                            .bindings()
                            .values()
                            .next()
                            .ok_or(PluginError::Unsupported)?;
                        binding
                            .validate(current.package_pin(), &config)
                            .map_err(|_| PluginError::InvalidRequest)?;
                        InstallationCommand::configure(command_id, id.clone(), revision, config)
                    }
                    PluginIntentDto::Disable { .. } => {
                        InstallationCommand::disable(command_id, id.clone(), revision)
                    }
                    PluginIntentDto::Revoke { .. } => {
                        InstallationCommand::revoke(command_id, id.clone(), revision)
                    }
                    PluginIntentDto::Enable {
                        readiness_digest, ..
                    } => {
                        let package = self.package(
                            current.package_pin().package_id().as_str(),
                            &current.package_pin().package_version().as_str(),
                        )?;
                        let probe = state.probes.get(&id).ok_or(PluginError::NotReady)?;
                        if probe.revision != revision
                            || probe.readiness.digest().as_str() != readiness_digest
                        {
                            return Err(PluginError::Conflict);
                        }
                        let service = MarketAdmissionService::new(
                            tenant,
                            user,
                            &package.manifest,
                            &package.configuration,
                            &self.registry,
                        )
                        .map_err(|_| PluginError::Denied)?;
                        self.check_enable_tool_capacity(
                            &state.authority,
                            tenant,
                            user,
                            &id,
                            package,
                        )?;
                        let receipt = service
                            .enable(
                                &mut next.installations,
                                &next.grants,
                                EnableAdmissionRequest {
                                    installation_id: id.clone(),
                                    expected_revision: revision,
                                    command_id,
                                },
                                &probe.readiness,
                            )
                            .map_err(|_| PluginError::Denied)?;
                        let view = result(&receipt, false);
                        state.commit(next)?;
                        // Transport activation is rebuildable. A failure keeps execution denied.
                        if view.accepted
                            && let Some(probe) = state.probes.get_mut(&id)
                            && let (Some(client), Some(digest)) =
                                (&mut probe.client, &probe.transport_digest)
                        {
                            // Cache failure cannot turn the accepted durable receipt into a rejection.
                            let _ = client.activate_reviewed(digest);
                        }
                        return Ok(view);
                    }
                    _ => return Err(PluginError::InvalidRequest),
                }
                .map_err(|_| PluginError::InvalidRequest)?;
                (id, command)
            }
        };
        let receipt = next
            .installations
            .execute(command)
            .map_err(|_| PluginError::Conflict)?;
        let view = result(&receipt, false);
        state.commit(next)?;
        if view.accepted {
            retire::probe(&mut state, &id).await;
        }
        Ok(view)
    }
    // Reserve a package's complete reviewed inventory without rediscovery or cross-owner reads.
    // A missing/changed catalog pin cannot contribute to a usable future projection.
    fn check_enable_tool_capacity(
        &self,
        state: &AuthorityState,
        tenant: &TenantId,
        user: &UserId,
        candidate: &InstallationId,
        package: &RuntimePackage,
    ) -> Result<(), PluginError> {
        let tool_count = |package: &RuntimePackage| match &package.component {
            RuntimeComponent::Skill { .. } => 1,
            RuntimeComponent::Mcp { tools, .. } => tools.len(),
        };
        let mut total = tool_count(package);
        for installation in state.installations.list_owned(tenant, user) {
            if installation.installation_id() == candidate
                || installation.state() != ManagedInstallationState::Enabled
            {
                continue;
            }
            if let Some(package) = self
                .packages
                .iter()
                .find(|package| package.configuration.package_pin() == installation.package_pin())
            {
                total = total
                    .checked_add(tool_count(package))
                    .ok_or(PluginError::Capacity)?;
            }
        }
        if total > MAX_PLUGIN_TOOLS {
            return Err(PluginError::Capacity);
        }
        Ok(())
    }
    #[allow(clippy::too_many_arguments)]
    fn grant(
        &self,
        state: &mut RuntimeState,
        tenant: &TenantId,
        user: &UserId,
        identity: &str,
        raw_id: &str,
        raw_revision: &str,
        raw_capability: &str,
    ) -> Result<PluginCommandResultDto, PluginError> {
        let id =
            InstallationId::parse(raw_id.to_owned()).map_err(|_| PluginError::InvalidRequest)?;
        let revision = InstallationRevision::parse(raw_revision.to_owned())
            .map_err(|_| PluginError::InvalidRequest)?;
        let capability = CapabilityId::parse(raw_capability.to_owned())
            .map_err(|_| PluginError::InvalidRequest)?;
        let command_id = GrantCommandId::parse(format!("grant-cmd:{identity}"))
            .map_err(|_| PluginError::InvalidRequest)?;
        let approval = GrantApprovalId::parse(format!("grant-approval:{identity}"))
            .map_err(|_| PluginError::InvalidRequest)?;
        let snapshot_id =
            GrantSnapshotId::parse(format!("grant:{}", stable_id(&[identity, raw_id])))
                .map_err(|_| PluginError::InvalidRequest)?;
        let scope = GrantScope::campus_public().map_err(|_| PluginError::Unavailable)?;
        if let Some(receipt) =
            state
                .authority
                .grants
                .lookup_owned_receipt(tenant, user, &command_id)
        {
            if !receipt.command().matches_issue(
                &snapshot_id,
                &approval,
                &revision,
                &capability,
                &scope,
                ConfirmationPolicy::Allow,
            ) {
                return Err(PluginError::Conflict);
            }
            return Ok(grant_result(&receipt, &id, true));
        }
        let current = state.owned(tenant, user, &id)?;
        if current.revision() != &revision {
            return Err(PluginError::Conflict);
        }
        let definition = self.registry.find(&capability).ok_or(PluginError::Denied)?;
        if definition.compatibility_class() != Some(CapabilityClass::PublicRead)
            || definition.scope_kind() != ScopeKind::CampusPublic
        {
            return Err(PluginError::Unsupported);
        }
        let package = self.package(
            current.package_pin().package_id().as_str(),
            &current.package_pin().package_version().as_str(),
        )?;
        let service = MarketAdmissionService::new(
            tenant,
            user,
            &package.manifest,
            &package.configuration,
            &self.registry,
        )
        .map_err(|_| PluginError::Denied)?;
        let mut next = state.authority.clone();
        if let Some(previous) = next
            .grants
            .load_current_for_authority(tenant, user, &id, &capability, &scope)
            .map_err(|_| PluginError::Unavailable)?
            && previous.state() == GrantState::Active
        {
            let revoke = GrantCommand::revoke(
                GrantCommandId::parse(format!("grant-cmd:replace-{identity}"))
                    .map_err(|_| PluginError::InvalidRequest)?,
                previous.snapshot_id().clone(),
                previous.version().clone(),
            )
            .map_err(|_| PluginError::InvalidRequest)?;
            let receipt = next
                .grants
                .execute(revoke)
                .map_err(|_| PluginError::Conflict)?;
            if !matches!(receipt.outcome(), GrantCommandOutcome::Accepted { .. }) {
                return Err(PluginError::Conflict);
            }
        }
        let receipt = service
            .issue_grant(
                &next.installations,
                &mut next.grants,
                GrantAdmissionRequest {
                    installation_id: id.clone(),
                    expected_revision: revision,
                    command_id,
                    approval_id: approval,
                    snapshot_id,
                    capability_id: capability,
                    confirmation_policy: ConfirmationPolicy::Allow,
                },
            )
            .map_err(|_| PluginError::Denied)?;
        let view = grant_result(&receipt, &id, false);
        state.commit(next)?;
        Ok(view)
    }
}
fn result(receipt: &InstallationCommandReceipt, replayed: bool) -> PluginCommandResultDto {
    let (accepted, revision, state) = match receipt.outcome() {
        InstallationCommandOutcome::Accepted { snapshot, .. } => (
            true,
            Some(snapshot.revision().as_str().to_owned()),
            Some(format!("{:?}", snapshot.state()).to_lowercase()),
        ),
        InstallationCommandOutcome::Rejected { .. } => (false, None, None),
    };
    PluginCommandResultDto {
        schema: "plugin-command-result/v1",
        accepted,
        installation_id: receipt.command().installation_id().as_str().to_owned(),
        revision,
        state,
        replayed,
    }
}
fn grant_result(
    receipt: &GrantCommandReceipt,
    id: &InstallationId,
    replayed: bool,
) -> PluginCommandResultDto {
    PluginCommandResultDto {
        schema: "plugin-command-result/v1",
        accepted: matches!(receipt.outcome(), GrantCommandOutcome::Accepted { .. }),
        installation_id: id.as_str().to_owned(),
        revision: None,
        state: None,
        replayed,
    }
}
fn matches_intent(
    tenant: &TenantId,
    command: &InstallationCommand,
    intent: &PluginIntentDto,
) -> Result<bool, PluginError> {
    let revision = |s: &str| {
        InstallationRevision::parse(s.to_owned()).map_err(|_| PluginError::InvalidRequest)
    };
    let id_matches = |id: &str| command.installation_id().as_str() == id;
    Ok(match intent {
        PluginIntentDto::Install {
            package_id,
            version,
            catalog_revision,
            package_digest,
        } => command.matches_install(
            &CatalogRevision::parse(catalog_revision.clone())
                .map_err(|_| PluginError::InvalidRequest)?,
            &PackageId::parse(package_id.clone()).map_err(|_| PluginError::InvalidRequest)?,
            &PackageVersion::parse(version).map_err(|_| PluginError::InvalidRequest)?,
            &Sha256Digest::parse(package_digest.clone())
                .map_err(|_| PluginError::InvalidRequest)?,
            &configuration(tenant, BTreeMap::new())?,
        ),
        PluginIntentDto::Configure {
            installation_id,
            expected_revision,
            values,
        } => {
            id_matches(installation_id)
                && command.matches_configure(
                    &revision(expected_revision)?,
                    &configuration(tenant, values.clone())?,
                )
        }
        PluginIntentDto::Enable {
            installation_id,
            expected_revision,
            readiness_digest,
        } => {
            id_matches(installation_id)
                && command.matches_enable(
                    &revision(expected_revision)?,
                    &Sha256Digest::parse(readiness_digest.clone())
                        .map_err(|_| PluginError::InvalidRequest)?,
                )
        }
        PluginIntentDto::Disable {
            installation_id,
            expected_revision,
        } => id_matches(installation_id) && command.matches_disable(&revision(expected_revision)?),
        PluginIntentDto::Revoke {
            installation_id,
            expected_revision,
        } => id_matches(installation_id) && command.matches_revoke(&revision(expected_revision)?),
        _ => false,
    })
}
