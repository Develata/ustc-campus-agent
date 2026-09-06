use super::*;
use ustc_campus_agent_adapters::mcp::{BindingConfig, Limits};
use ustc_campus_agent_core::market::admission::mcp_inventory_digest;
use ustc_campus_agent_core::market::capability::CapabilityStatus;
impl PluginRuntime {
    pub(crate) async fn probe(
        &self,
        tenant: &TenantId,
        user: &UserId,
        request: PluginProbeDto,
    ) -> Result<PluginProbeResultDto, PluginError> {
        if request.schema != "plugin-probe/v1" {
            return Err(PluginError::InvalidRequest);
        }
        let id = InstallationId::parse(request.installation_id)
            .map_err(|_| PluginError::InvalidRequest)?;
        let mut state = self.state.lock().await;
        let current = state.owned(tenant, user, &id)?;
        if current.revision().as_str() != request.expected_revision {
            return Err(PluginError::Conflict);
        }
        if !matches!(
            current.state(),
            ManagedInstallationState::InstalledDisabled
                | ManagedInstallationState::Disabled
                | ManagedInstallationState::Enabled
        ) {
            return Err(PluginError::Denied);
        }
        let package = self.package(
            current.package_pin().package_id().as_str(),
            &current.package_pin().package_version().as_str(),
        )?;
        // A failed or incomplete probe invalidates cached execution immediately.
        retire::probe(&mut state, &id).await;
        let probe = self.build_probe(tenant, user, &current, package).await?;
        let view = PluginProbeResultDto {
            schema: "plugin-probe-result/v1",
            installation_id: id.as_str().to_owned(),
            revision: current.revision().as_str().to_owned(),
            readiness_digest: probe.readiness.digest().as_str().to_owned(),
            kind: if !probe.additional.is_empty() {
                "mixed"
            } else if probe.client.is_some() {
                "mcp"
            } else {
                "skill"
            }
            .to_owned(),
            tools: probe
                .tools
                .iter()
                .map(|tool| PluginToolDto {
                    name: probe
                        .wire_names
                        .get(&tool.model_visible_name)
                        .cloned()
                        .unwrap_or_else(|| tool.model_visible_name.clone()),
                    description: tool.description.clone(),
                    capability: tool.capability_id.as_str().to_owned(),
                })
                .collect(),
        };
        state.probes.insert(id, probe);
        Ok(view)
    }
    pub(super) async fn build_probe(
        &self,
        tenant: &TenantId,
        user: &UserId,
        current: &InstallationSnapshot,
        package: &RuntimePackage,
    ) -> Result<ProbedComponent, PluginError> {
        let mut probes = Vec::new();
        for (id, component) in package.components() {
            match self
                .build_component_probe(tenant, user, current, package, id, component)
                .await
            {
                Ok(probe) => probes.push(probe),
                Err(error) => {
                    for probe in probes {
                        retire::component(probe).await;
                    }
                    return Err(error);
                }
            }
        }
        let readiness =
            ComponentReadiness::package(probes.iter().map(|p| p.readiness.clone()).collect())
                .map_err(|_| PluginError::NotReady)?;
        let mut first = probes.remove(0);
        first.readiness = readiness;
        for probe in probes {
            first.tools.extend(probe.tools.clone());
            first.wire_names.extend(probe.wire_names.clone());
            first.tool_components.extend(probe.tool_components.clone());
            first.additional.insert(probe.component_id.clone(), probe);
        }
        if first.tools.len() > MAX_PLUGIN_TOOLS {
            retire::component(first).await;
            return Err(PluginError::Capacity);
        }
        Ok(first)
    }
    #[allow(clippy::too_many_arguments)]
    async fn build_component_probe(
        &self,
        tenant: &TenantId,
        user: &UserId,
        current: &InstallationSnapshot,
        package: &RuntimePackage,
        component_id: &ComponentId,
        component: &RuntimeComponent,
    ) -> Result<ProbedComponent, PluginError> {
        let binding = package
            .configuration
            .binding(component_id)
            .ok_or(PluginError::Unsupported)?;
        binding
            .validate(current.package_pin(), current.configuration())
            .map_err(|_| PluginError::InvalidRequest)?;
        for capability in package.manifest.capabilities() {
            let definition = self.registry.find(capability).ok_or(PluginError::Denied)?;
            if definition.status() != CapabilityStatus::Active
                || definition.compatibility_class() != Some(CapabilityClass::PublicRead)
            {
                return Err(PluginError::Unsupported);
            }
        }
        let id = current.installation_id();
        match component {
            RuntimeComponent::Skill { source } => {
                let schema = super::skill_context::input_schema()?;
                let capability = registry::skill_read_capability(&package.manifest)
                    .map_err(|_| PluginError::Unsupported)?;
                let name = component_tool_name(package, id, component_id, "skill_read");
                let tool = CatalogToolDefinition {
                    id: ToolId::parse("tool:skill-read").map_err(|_| PluginError::Unavailable)?,
                    model_visible_name: name.clone(),
                    description: super::skill_context::description(source),
                    capability_id: capability,
                    input_schema: Some(schema.clone()),
                    claimed_input_schema_digest: schema.digest().clone(),
                };
                let readiness = ComponentReadiness::skill(
                    binding,
                    current.configuration(),
                    &source
                        .verified_artifact_digest()
                        .map_err(|_| PluginError::NotReady)?,
                )
                .map_err(|_| PluginError::NotReady)?;
                Ok(ProbedComponent {
                    component_id: component_id.clone(),
                    additional: BTreeMap::new(),
                    tool_components: BTreeMap::from([(name.clone(), component_id.clone())]),
                    revision: current.revision().clone(),
                    readiness,
                    tools: vec![tool],
                    wire_names: BTreeMap::from([(name, "skill_read".to_owned())]),
                    client: None,
                    transport_digest: None,
                })
            }
            RuntimeComponent::Mcp {
                endpoint_key,
                tools,
                endpoint_policy,
                bearer_file,
                credential_endpoint,
            } => {
                let Some(ConfigurationValue::Text(endpoint)) =
                    current.configuration().entries().get(endpoint_key)
                else {
                    return Err(PluginError::InvalidRequest);
                };
                // Operator credentials are bound to one exact reviewed URL, before
                // reading secret bytes or opening a connection to browser configuration.
                match (bearer_file, credential_endpoint) {
                    (None, None) => {}
                    (Some(_), Some(reviewed)) if endpoint.as_str() == reviewed => {}
                    _ => return Err(PluginError::Denied),
                }
                let token = bearer_file
                    .as_ref()
                    .map(|path| read_credential(path))
                    .transpose()?;
                let mut client = McpClient::new(
                    BindingConfig {
                        binding_id: format!(
                            "binding:{}",
                            stable_id(&[tenant.as_str(), user.as_str(), id.as_str()])
                        ),
                        owner_id: stable_id(&[tenant.as_str(), user.as_str()]),
                        installation_id: id.as_str().to_owned(),
                        component_id: binding.component_id().as_str().to_owned(),
                        endpoint: endpoint.as_str().to_owned(),
                        endpoint_policy: *endpoint_policy,
                        bearer_token: token,
                    },
                    Limits::default(),
                )
                .map_err(|_| PluginError::InvalidRequest)?;
                let inventory = client.discover().await.map_err(|_| PluginError::NotReady)?;
                let projection = (|| {
                    if inventory.tools().len() != tools.len() || tools.len() > MAX_PLUGIN_TOOLS {
                        return Err(PluginError::Unsupported);
                    }
                    let mut definitions = Vec::new();
                    let mut names = BTreeMap::new();
                    for tool in inventory.tools() {
                        let capability = tools.get(tool.name()).ok_or(PluginError::Denied)?.clone();
                        if !package.manifest.capabilities().contains(&capability) {
                            return Err(PluginError::Denied);
                        }
                        let name = component_tool_name(package, id, component_id, tool.name());
                        // The immutable M51 inventory identity includes server/output schemas as well as inputs.
                        let tool_id = ToolId::parse(format!(
                            "tool:{}",
                            stable_id(&[inventory.digest().as_str(), tool.name()])
                        ))
                        .map_err(|_| PluginError::NotReady)?;
                        definitions.push(CatalogToolDefinition {
                            id: tool_id,
                            model_visible_name: name.clone(),
                            description: if tool.description().trim().is_empty() {
                                format!("Reviewed package tool {}", tool.name())
                            } else {
                                tool.description().replace('\0', " ")
                            },
                            capability_id: capability,
                            input_schema: Some(tool.compiled_schema().clone()),
                            claimed_input_schema_digest: tool.compiled_schema().digest().clone(),
                        });
                        names.insert(name, tool.name().to_owned());
                    }
                    let digest =
                        mcp_inventory_digest(binding, current.configuration(), &definitions)
                            .map_err(|_| PluginError::NotReady)?;
                    let readiness = ComponentReadiness::mcp(
                        binding,
                        current.configuration(),
                        &definitions,
                        &digest,
                    )
                    .map_err(|_| PluginError::NotReady)?;
                    Ok((readiness, definitions, names))
                })();
                let (readiness, definitions, names) = match projection {
                    Ok(projection) => projection,
                    Err(error) => {
                        retire::client(client).await;
                        return Err(error);
                    }
                };
                Ok(ProbedComponent {
                    component_id: component_id.clone(),
                    additional: BTreeMap::new(),
                    tool_components: definitions
                        .iter()
                        .map(|tool| (tool.model_visible_name.clone(), component_id.clone()))
                        .collect(),
                    revision: current.revision().clone(),
                    readiness,
                    tools: definitions,
                    wire_names: names,
                    client: Some(client),
                    transport_digest: Some(inventory.digest().clone()),
                })
            }
        }
    }
    pub(crate) async fn session(
        &self,
        tenant: &TenantId,
        user: &UserId,
    ) -> Result<PluginToolSession, PluginError> {
        let mut state = self.state.lock().await;
        state.check()?;
        let installations = state.authority.installations.list_owned(tenant, user);
        let mut definitions = Vec::new();
        let mut bindings = BTreeMap::new();
        for current in installations {
            if current.state() != ManagedInstallationState::Enabled {
                continue;
            }
            if self
                .ensure_active(&mut state, tenant, user, &current)
                .await
                .is_err()
            {
                continue;
            }
            let probe = state
                .probes
                .get(current.installation_id())
                .ok_or(PluginError::NotReady)?;
            for tool in &probe.tools {
                let Ok(grant) = self.current_grant(&state.authority, &current, &tool.capability_id)
                else {
                    continue;
                };
                if definitions.len() >= MAX_PLUGIN_TOOLS {
                    return Err(PluginError::Capacity);
                }
                let frozen = FrozenToolBinding {
                    installation_id: current.installation_id().clone(),
                    component_id: probe
                        .tool_components
                        .get(&tool.model_visible_name)
                        .ok_or(PluginError::NotReady)?
                        .clone(),
                    installation_revision: current.revision().clone(),
                    readiness_digest: probe.readiness.digest().clone(),
                    grant_snapshot_id: grant.snapshot_id().clone(),
                    grant_version: grant.version().clone(),
                    tool: tool.clone(),
                };
                if bindings
                    .insert(tool.model_visible_name.clone(), frozen)
                    .is_some()
                {
                    return Err(PluginError::NotReady);
                }
                definitions.push(
                    ChatDynamicToolDefinition::new(
                        tool.model_visible_name.clone(),
                        tool.description.clone(),
                        tool.input_schema.clone().ok_or(PluginError::NotReady)?,
                    )
                    .map_err(|_| PluginError::NotReady)?,
                );
            }
        }
        Ok(PluginToolSession {
            runtime: Arc::downgrade(&self.state),
            tenant: tenant.clone(),
            user: user.clone(),
            definitions,
            bindings,
        })
    }
    #[cfg(test)]
    pub(crate) async fn definitions(
        &self,
        tenant: &TenantId,
        user: &UserId,
    ) -> Result<Vec<ChatDynamicToolDefinition>, PluginError> {
        Ok(self.session(tenant, user).await?.definitions())
    }
    pub(super) async fn ensure_active(
        &self,
        state: &mut RuntimeState,
        tenant: &TenantId,
        user: &UserId,
        current: &InstallationSnapshot,
    ) -> Result<(), PluginError> {
        if current.state() != ManagedInstallationState::Enabled {
            return Err(PluginError::Denied);
        }
        let evidence = state
            .authority
            .installations
            .latest_enable_evidence(current.installation_id())
            .ok_or(PluginError::Denied)?;
        if !state.probes.contains_key(current.installation_id()) {
            let package = self.package(
                current.package_pin().package_id().as_str(),
                &current.package_pin().package_version().as_str(),
            )?;
            let probe = self.build_probe(tenant, user, current, package).await?;
            state
                .probes
                .insert(current.installation_id().clone(), probe);
        }
        let probe = state
            .probes
            .get_mut(current.installation_id())
            .ok_or(PluginError::NotReady)?;
        if probe.readiness.digest() != evidence.policy_admission_snapshot_digest() {
            return Err(PluginError::NotReady);
        }
        activate_probe(probe)?;
        Ok(())
    }
    pub(super) fn current_grant(
        &self,
        state: &AuthorityState,
        current: &InstallationSnapshot,
        capability: &CapabilityId,
    ) -> Result<GrantSnapshot, PluginError> {
        let definition = self.registry.find(capability).ok_or(PluginError::Denied)?;
        if definition.status() != CapabilityStatus::Active
            || definition.compatibility_class() != Some(CapabilityClass::PublicRead)
        {
            return Err(PluginError::Denied);
        }
        let scope = GrantScope::campus_public().map_err(|_| PluginError::Unavailable)?;
        let grant = state
            .grants
            .load_current_for_authority(
                current.tenant_id(),
                current.user_id(),
                current.installation_id(),
                capability,
                &scope,
            )
            .map_err(|_| PluginError::Unavailable)?
            .ok_or(PluginError::Denied)?;
        if grant.state() != GrantState::Active
            || grant.package_digest() != current.package_pin().package_digest()
            || grant.capability_manifest_digest()
                != current.package_pin().capability_manifest_digest()
            || grant.capability_definition() != definition
            || grant.confirmation_policy() != ConfirmationPolicy::Allow
        {
            return Err(PluginError::Denied);
        }
        Ok(grant)
    }
}
fn read_credential(path: &std::path::Path) -> Result<String, PluginError> {
    use std::{
        fs::OpenOptions,
        io::Read,
        os::unix::fs::{MetadataExt, OpenOptionsExt, PermissionsExt},
    };
    crate::durable_path::ensure_secure_parent(path, false).map_err(|_| PluginError::Unavailable)?;
    let mut file = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK)
        .open(path)
        .map_err(|_| PluginError::Unavailable)?;
    let metadata = file.metadata().map_err(|_| PluginError::Unavailable)?;
    let uid = std::fs::metadata("/proc/self")
        .map_err(|_| PluginError::Unavailable)?
        .uid();
    if !metadata.is_file()
        || metadata.uid() != uid
        || metadata.nlink() != 1
        || metadata.permissions().mode() & 0o7777 != 0o600
        || metadata.len() > 4096
    {
        return Err(PluginError::Unavailable);
    }
    let mut token = String::new();
    Read::by_ref(&mut file)
        .take(4097)
        .read_to_string(&mut token)
        .map_err(|_| PluginError::Unavailable)?;
    Ok(token.trim_end_matches(['\r', '\n']).to_owned())
}

fn component_tool_name(
    package: &RuntimePackage,
    installation: &InstallationId,
    component: &ComponentId,
    wire: &str,
) -> String {
    if package.additional.is_empty() {
        tool_name(installation, wire)
    } else {
        tool_name(installation, &stable_id(&[component.as_str(), wire]))
    }
}
pub(super) fn activate_probe(probe: &mut ProbedComponent) -> Result<(), PluginError> {
    if let (Some(client), Some(digest)) = (&mut probe.client, &probe.transport_digest) {
        client
            .activate_reviewed(digest)
            .map_err(|_| PluginError::NotReady)?;
    }
    for child in probe.additional.values_mut() {
        if let (Some(client), Some(digest)) = (&mut child.client, &child.transport_digest) {
            client
                .activate_reviewed(digest)
                .map_err(|_| PluginError::NotReady)?;
        }
    }
    Ok(())
}
