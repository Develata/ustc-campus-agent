//! Pure import conversion: inert review files, no catalog admission, filesystem or network IO.
use super::*;
use serde_json::json;
use ustc_campus_agent_adapters::skills::ParsedSkill;
use ustc_campus_agent_core::market::{
    configuration_catalog::load_package_configuration,
    configuration_schema::{ConfigurationFieldSchema, ConfigurationSchema},
    load_package_manifest,
};
impl PluginRuntime {
    pub(crate) fn preview_import(
        &self,
        request: PluginImportPreviewDto,
    ) -> Result<PluginImportReviewDto, PluginError> {
        if request.schema != "plugin-import-preview/v1"
            || request.display_name.trim().is_empty()
            || request.display_name.len() > 256
            || request.source.trim().is_empty()
            || request.source.len() > 2048
            || request.source.chars().any(char::is_control)
            || request.skill.is_none() && request.mcp.is_none()
        {
            return Err(PluginError::InvalidRequest);
        }
        let mut files = BTreeMap::new();
        let mut declarations = Vec::new();
        let mut runtime_members = Vec::new();
        let mut capabilities = std::collections::BTreeSet::new();
        let mut values = BTreeMap::new();
        if let Some(skill) = &request.skill {
            if skill.len() > 64 * 1024 {
                return Err(PluginError::Capacity);
            }
            // Name determines only an inert, validated archive-relative resource path.
            let mut name = None;
            for line in skill.lines().take(64) {
                if let Some(value) = line.strip_prefix("name:") {
                    name = Some(value.trim().trim_matches(['\"', '\'']).to_owned());
                    break;
                }
            }
            let name = name.ok_or(PluginError::InvalidRequest)?;
            ParsedSkill::parse(&name, skill.as_bytes()).map_err(|_| PluginError::InvalidRequest)?;
            if !name
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
            {
                return Err(PluginError::InvalidRequest);
            }
            let path = format!("skills/{name}/SKILL.md");
            let digest = Sha256Digest::from_bytes(skill.as_bytes());
            files.insert(path.clone(), skill.clone());
            declarations.push(json!({"type":"SkillComponent", "path":path}));
            runtime_members.push(json!({"componentId":"component:skill", "runtime":{
                "schemaVersion":"plugin-runtime/v1", "kind":"skill", "skillPath":path,
                "resources":[{"path":path,"sha256":digest.as_str()}]}}));
            capabilities.insert("campus.public_rules.read".to_owned());
        }
        if let Some(mcp) = &request.mcp {
            let url =
                reqwest::Url::parse(&mcp.endpoint).map_err(|_| PluginError::InvalidRequest)?;
            if mcp.endpoint.len() > 2048
                || url.scheme() != "https"
                || url.host_str().is_none()
                || !url.username().is_empty()
                || url.password().is_some()
                || url.fragment().is_some()
                || url.query().is_some()
                || mcp.endpoint.chars().any(char::is_whitespace)
                || mcp.tools.is_empty()
                || mcp.tools.len() > 27
            {
                return Err(PluginError::InvalidRequest);
            }
            let mut tools = Vec::new();
            for (name, capability) in &mcp.tools {
                if name.is_empty()
                    || name.len() > 128
                    || !name
                        .bytes()
                        .all(|b| b.is_ascii_alphanumeric() || b"_.-".contains(&b))
                {
                    return Err(PluginError::InvalidRequest);
                }
                let id = CapabilityId::parse(capability.clone())
                    .map_err(|_| PluginError::InvalidRequest)?;
                let definition = self.registry.find(&id).ok_or(PluginError::Denied)?;
                if definition.compatibility_class() != Some(CapabilityClass::PublicRead)
                    || definition.scope_kind() != ScopeKind::CampusPublic
                {
                    return Err(PluginError::Unsupported);
                }
                capabilities.insert(capability.clone());
                tools.push(json!({"name":name,"capabilityId":capability}));
            }
            declarations.push(json!({"type":"McpServerComponent", "path":"runtime.json"}));
            runtime_members.push(json!({"componentId":"component:mcp", "runtime":{
                "schemaVersion":"plugin-runtime/v1", "kind":"mcp", "endpointKey":"endpoint",
                "endpointPolicy":"public_https", "tools":tools}}));
            values.insert(
                "endpoint".to_owned(),
                PluginValueDto::Text(mcp.endpoint.clone()),
            );
        }
        let manifest = json!({"id":request.package_id,"version":request.version,"publisher":"local-import-candidate",
            "tier":"VerifiedCommunityText","displayName":request.display_name,"implementationStatus":"development",
            "installPolicy":{"class":"UserInstalledPlugin","defaultInstalled":false,"defaultEnabled":false,"userDisableAllowed":true},
            "components":declarations,"capabilities":capabilities,
            "sourcePolicy":{"source":request.source,"personalData":"none; public-read review candidate",
                "execution":"inert candidate; operator must review provenance and capability mapping before admission"}});
        let manifest_text =
            serde_json::to_string_pretty(&manifest).map_err(|_| PluginError::Unavailable)?;
        let manifest = load_package_manifest(manifest_text.as_bytes())
            .map_err(|_| PluginError::InvalidRequest)?;
        let runtime = if runtime_members.len() == 1 {
            runtime_members[0]["runtime"].clone()
        } else {
            json!({"schemaVersion":"plugin-runtime/v2","components":runtime_members})
        };
        let runtime_text =
            serde_json::to_string_pretty(&runtime).map_err(|_| PluginError::Unavailable)?;
        let fields = if request.mcp.is_some() {
            vec![
                ConfigurationFieldSchema::text(
                    ConfigurationKey::parse("endpoint").map_err(|_| PluginError::Unavailable)?,
                    true,
                    2048,
                )
                .map_err(|_| PluginError::Unavailable)?,
            ]
        } else {
            Vec::new()
        };
        let schema = ConfigurationSchema::new(fields).map_err(|_| PluginError::Unavailable)?;
        let fields = if request.mcp.is_some() {
            json!([{"key":"endpoint","kind":"text","required":true,"maxUtf8Bytes":2048}])
        } else {
            json!([])
        };
        let mut bindings = Vec::new();
        for declaration in manifest.components() {
            let (id, digest) = if declaration.kind() == ComponentKind::SkillComponent {
                (
                    "component:skill",
                    Sha256Digest::from_bytes(
                        files
                            .get(declaration.path())
                            .ok_or(PluginError::Unavailable)?
                            .as_bytes(),
                    ),
                )
            } else {
                (
                    "component:mcp",
                    Sha256Digest::from_bytes(runtime_text.as_bytes()),
                )
            };
            bindings.push(json!({"path":declaration.path(), "type":if declaration.kind() == ComponentKind::SkillComponent {"SkillComponent"}else{"McpServerComponent"},
                "mode":null,"componentId":id,"componentVersion":"1","componentDigest":digest.as_str(),
                "executionIdentity":if declaration.kind() == ComponentKind::SkillComponent {"execution:reviewed-skill"}else{"execution:reviewed-mcp"},
                "schemaDigest":schema.digest().as_str(),"fields":fields}));
        }
        let config = json!({"schemaVersion":"package-component-configuration/v1","packageId":manifest.package_id().as_str(),
            "packageVersion":manifest.package_version().as_str(),"packageDigest":manifest.package_digest().as_str(),
            "componentSetDigest":manifest.component_declaration_set_digest().as_str(),"capabilityManifestDigest":manifest.capability_manifest_digest().as_str(),"components":bindings});
        let config_text =
            serde_json::to_string_pretty(&config).map_err(|_| PluginError::Unavailable)?;
        load_package_configuration(
            config_text.as_bytes(),
            &manifest,
            &CatalogRevision::parse("catalog:import-review")
                .map_err(|_| PluginError::Unavailable)?,
        )
        .map_err(|_| PluginError::InvalidRequest)?;
        files.insert("package.json".to_owned(), manifest_text);
        files.insert("runtime.json".to_owned(), runtime_text);
        files.insert("configuration.json".to_owned(), config_text);
        let packet = serde_json::to_vec(&files).map_err(|_| PluginError::Unavailable)?;
        Ok(PluginImportReviewDto {
            schema: "plugin-import-review/v1",
            review_digest: Sha256Digest::from_bytes(&packet).as_str().to_owned(),
            files,
            configuration_values: values,
            admitted: false,
            warnings: vec![
                "候选文件尚未进入可安装目录；必须审阅来源、全部文本与工具权限映射。".to_owned(),
                "审阅通过后由运营方导入包目录，复用安装、配置、检查、授权、启用流程。".to_owned(),
                "不会读取本地路径、请求服务器、执行脚本或把 MCP 注解认定为权限。".to_owned(),
            ],
        })
    }
}
