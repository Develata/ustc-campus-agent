# MCP Binding、权限与 Secret Policy

- 状态：**remote component binding policy 已与 PluginPackage authority 收敛；market hosting runtime 待团队确认与 spike**
- 更新时间：2026-07-21
- 适用对象：USTC 个人校园 Agent 平台
- 协议基线：released MCP `2025-11-25`

## 1. 当前决策

MCP 不再压成单一 execution-mode enum，而按独立轴分类：

```text
source:       Official/VerifiedMarket | UserRemote | UserHostedPrivate
location:     PlatformHosted | ExternalRemote | FutureUserRelay
tenant mode:  SharedSafe | DedicatedUser
availability: OnDemand | Warm
```

- `Official/VerifiedMarket`：平台导入或审核 immutable artifact；只有通过 `SharedSafe` gate 才能共享 runtime；
- `UserRemote`：用户提供 remote MCP Streamable HTTP endpoint 与 credential，中央加密保存并调用；
- `UserHostedPrivate`：用户提交 approved OCI-digest artifact，平台为该用户 dedicated cold-start；
- `FutureUserRelay`：未来由用户设备或常驻 relay 执行，不进入 Demo。

Catalog trust、tenant isolation 与 warm entitlement 互不推出。完整 hosting、artifact admission、sandbox、cold/warm 与攻击面 policy 见 [`mcp-market-hosting-policy.md`](mcp-market-hosting-policy.md)。Demo 中已支持的中央路径均不依赖手机、Web tab 或 desktop companion 在线。

`PluginPackage` 是唯一 user-facing install/enable/grant unit。`UserRemote` / `UserHostedPrivate` 配置必须先创建 user-owned `PrivatePluginPackage` installation；`McpBinding` 只是其中某个 MCP component 的 runtime projection，不能独立授予可见性或调用权。

## 2. 协议版本边界

截至 2026-07-21，Demo 锁定 released MCP specification：

```text
2025-11-25
```

理由：

- 这是当前已发布版本；
- 2026-07-28 仍是未来 release candidate，不能作为当前实现 contract；
- released spec 使用 Streamable HTTP、protocol initialization 与可选 session；
- future draft 正在调整 lifecycle/session，因此需要 transport adapter 隔离变化。

实现不得直接依赖 SDK `main` 分支的 future protocol 默认行为。升级协议必须经过独立 compatibility spike 与 conformance gate。

## 3. Demo transport 与 capability scope

外部 remote MCP 只实现 Streamable HTTP。Platform-hosted MCP 可在隔离 runtime 内使用 stdio，但只接受 approved immutable artifact；stdio 不得在 API/authority host 上启动。

Demo 不实现：

- 用户提交任意 `command/args/cwd`；
- production request path 上运行 `npx -y`、`uvx`、在线 package install；
- 任意 shell command launcher；
- WebSocket 私有 transport；
- local process MCP relay；
- 无 artifact admission/sandbox 的 user code hosting。

Private hosted Demo 建议只接受 OCI image digest。npm/PyPI/MCPB 等 package 必须先经隔离 build/import pipeline 转成 immutable artifact。

Demo 首先支持 MCP `tools` capability。`resources`、`prompts`、`sampling`、`elicitation` 与 MCP task extension 保留 adapter boundary，但不进入首轮验收。

这是 scope control，不表示这些能力长期不需要。

## 4. Binding 数据模型

建议抽象：

```text
McpBinding {
  id
  plugin_installation_id
  component_id
  effective_component_version
  expected_execution_identity
  mode
  display_name
  endpoint
  auth_mode
  credential_ref?
  protocol_version
  runtime_state
  capability_snapshot_hash?
  created_at
  updated_at
}
```

其中：

- `endpoint` 是固定 Streamable HTTP endpoint，不允许 Agent 在每次调用时改写；
- `credential_ref` 指向 secret store，不包含 secret 明文；
- `protocol_version` 在连接成功后记录 negotiated version；
- `capability_snapshot_hash` 用于检测 tool catalog/schema 漂移；
- owner、enabled state、capability manifest 与 grants_version 由 `PluginInstallation` authority 提供；binding 可缓存 projection，但每次调用必须重新解析并比对；
- `plugin_installation_id + component_id` 是不可缺失的 tenant/install boundary；
- `runtime_state` 只表达 provisioning/health，不拥有用户启停语义。

每个用户可调用的 `McpBinding` 必须属于一个且仅一个 user-owned `PluginInstallation`。Operator-owned catalog entry、artifact 与 logical deployment 是独立对象；official shared runtime 也必须通过 user-owned installation/component binding/grant 进入 gateway。

## 5. Secret storage

`UserRemote` credential 采用与 `UserCloud` model provider 相同的中央 secret policy：

```text
UI input
  -> TLS
  -> secret service
  -> AEAD ciphertext
  -> DB
```

主密钥不得与 ciphertext 存在同一数据库中。Compose 部署至少应：

- 通过 Docker secret 或 root-readable mounted secret 注入 master key；
- 使用 authenticated encryption；
- 保存 key version、nonce 与 ciphertext；
- 支持 credential 替换、撤销与删除；
- 在日志、trace、metric label 和 error body 中统一 redaction；
- 只在 MCP worker 发起请求前短暂解密；
- audit 只记录 `credential_ref`，不记录 credential value。

边界必须向用户明示：

> Credential encrypted at rest，但中央运行时能够解密和使用；它不是端到端本地 secret。

## 6. Demo auth mode

Demo 建议只做：

```text
none
static_bearer
```

其中 `static_bearer` 是平台 connector convention，不应伪装成完整 MCP OAuth 2.1 implementation。

预留：

```text
oauth_2_1
custom_header_template
mtls
```

Demo 不建议开放任意 header map，因为它容易造成：

- Host/header injection；
- secret 误发到 redirect target；
- connection-level policy 绕过；
- 难以 redaction 与审计。

如后续实现 MCP OAuth 2.1，必须单独完成 protected-resource metadata discovery、authorization-server metadata、PKCE、state、exact redirect URI、token audience 与 refresh lifecycle。

## 7. PrivatePluginPackage 创建与 binding provisioning

`UserRemote` endpoint 不应在保存 URL 后立即对 Agent 可见。用户操作首先创建 private PluginPackage/install intent；binding 的 runtime provisioning lifecycle 为：

```text
PendingProvision
  -> ValidatingEndpoint
  -> Initializing
  -> AwaitingInstallationGrant
  -> Ready
```

步骤：

1. 用户输入 display name、HTTPS endpoint 与 auth mode；
2. application-level URL validation；
3. network egress policy 再次限制目的地；
4. 使用 released protocol 完成 initialization/negotiation；
5. 执行 paginated `tools/list`；
6. 校验每个 tool 的 name 与 JSON Schema；
7. 生成 normalized capability snapshot 与 hash；
8. UI 展示 tool catalog、schema 与风险提示；
9. 用户显式选择 allowed tools，写入 PluginInstallation grant；
10. private PluginPackage installation 进入 enabled，binding runtime state 进入 `Ready`。

连接测试不得自动调用任何业务 tool。

## 8. Capability snapshot

远端 server 的 `tools/list` 是不可信声明。中央应保存：

```text
ToolSnapshot {
  binding_id
  tool_name
  title?
  description
  normalized_input_schema
  output_schema?
  annotations?
  schema_hash
  discovered_at
}
```

以下变化必须触发 re-review：

- 新增 tool；
- 删除 tool；
- input/output schema 变化；
- annotation/risk hint 变化；
- server identity/version 显著变化。

新 tool 默认 disabled。已授权 tool 的 schema hash 变化后，binding runtime state 进入 `NeedsReapproval`，同时 installation/component grant 被阻止，不得静默沿用旧授权。

`notifications/tools/list_changed` 只能触发重新 discovery，不能直接扩大权限。

## 9. Tool grant 与风险模型

MCP spec 明确 tool annotation 只是 hint；对非官方 remote server 必须视为 untrusted。

平台 risk authority 使用两个独立轴，并与 Market Capability Registry 对齐：

```text
effect_class: Read | Write | Destructive | Unknown
data_class:   PublicRead | UserPrivateReadScoped |
              CrossUserAggregateRead | InternalDiagnosticRead |
              ExternalUntrustedRead
```

判定来源按优先级：

1. operator-maintained override；
2. marketplace verified manifest；
3. deterministic heuristic；
4. remote MCP annotation，仅作参考；
5. 无法判断则 `Unknown`。

Demo 策略：

- `Read`：用户安装时授权后可在 exact data/object scope 内调用；default first-party 是否 auto-grant 另由 operator-maintained `auto_grant_eligible` 决定，remote annotation 无权决定；
- `CrossUserAggregateRead` / `InternalDiagnosticRead` 不得由 default Plugin 自动获得；
- `Write` / `Destructive`：每次调用前展示 tool、arguments 与目标并要求确认；
- `Unknown` effect/data class：按 `Destructive + InternalDiagnosticRead` 的最严组合处理；
- scheduled task 默认只允许 exact-granted `Read`，禁止 `Write`、`Destructive` 与 `Unknown`；
- 不提供“信任所有未来 tools”开关。

模型或 skill 不能自行扩大 grant。

## 10. 调用 contract

Agent 只允许调用 typed broker：

```text
McpGateway.call(
  plugin_installation_id,
  component_binding_id,
  tool_name,
  arguments,
  expected_capability_manifest_hash,
  expected_grants_version,
  invocation_context,
)
```

broker 必须检查：

- installation owner 与 current user 一致，且 installation enabled/not revoked；
- binding 属于该 installation/component，runtime state 为 `Ready`；
- exact package/component version、execution identity 与 effective deployment resolution 一致；
- tool 存在于 current snapshot；
- tool 位于 exact-name allowlist；
- schema hash 与 grant version 一致；
- arguments 通过 JSON Schema validation；
- 风险等级允许当前 execution context；
- rate/concurrency quota；
- deadline、timeout 与 cancellation；
- request/response size limit。

禁止暴露：

```text
McpBinding.raw_request(url, headers, body)
```

中央 Agent、skill 与 marketplace content 都不能把 MCP broker 变成任意网络代理。

## 11. Scheduled task 语义

Task 保存：

```text
plugin_installation_id
component_binding_id
expected_package_version
expected_component_version
expected_capability_manifest_hash
required_tool_names
expected_grants_version
```

执行前检查：

- installation 是否启用且未 revoke；
- binding 是否仍属于 exact installation/component；
- package/component/digest/capability/grant resolution 是否一致；
- credential 是否有效；
- tool snapshot 是否变化；
- required tools 是否仍获授权；
- tool risk 是否允许 unattended execution。

以下情况返回结构化 blocked state，而不是换用另一个 MCP：

```text
InstallationDisabled
InstallationVersionChanged
ComponentBindingUnavailable
ArtifactBlocked
CredentialInvalid
EndpointUnavailable
ProtocolMismatch
CapabilitiesChanged
GrantRevoked
InteractiveConfirmationRequired
```

不得 silent fallback 到同名 official MCP，因为同名 tool 不保证语义、数据源或副作用一致。

## 12. SSRF 防护

`UserRemote` 比 model provider 的 SSRF 面更大，因为 MCP auth discovery 还可能引入额外 URL。

### Application layer

至少：

- production 只接受 `https://`；
- 禁止 URL userinfo；
- 拒绝 loopback、private、link-local、multicast、reserved 与 cloud metadata ranges；
- 使用标准 URL/IP parser，不自写字符串黑名单；
- DNS resolve 后验证所有结果；
- 防 IPv4-mapped IPv6 与编码绕过；
- 禁止自动 redirect，或逐跳重复同等校验；
- OAuth metadata 中的 resource/auth/token endpoints 同样校验；
- 限制 response header/body 大小与总耗时。

### Network layer

同时：

- MCP worker 使用独立 egress policy；
- 默认无法访问 Compose internal network、DB、queue、secret service 与 metadata endpoint；
- 只允许批准的公网 HTTPS 目的地；
- USTC 内部 MCP 通过 operator-maintained allowlist 配置，不向普通用户开放私网例外；
- 审计 blocked destination，但不得泄露内部网络细节。

只做 URL regex 不算完成 SSRF 防护。

## 13. Token 与 identity 边界

严格禁止：

- 把 USTC login token 直接转发给 remote MCP；
- 把一个用户的 credential 复用于另一用户；
- 把 MCP session ID 当成身份凭证；
- token passthrough 到不匹配 audience 的下游；
- 将 remote MCP error 原样回显 secret/header。

每个 remote binding 的 credential 必须独立、owner-scoped、可撤销。

如果 official MCP 需要 per-user downstream identity，应使用明确的 delegated authorization，而不是共享 operator token 冒充所有用户。

## 14. Session 与 protocol adapter

released `2025-11-25` Streamable HTTP 可以使用 MCP session。中央必须：

- 按 `(plugin_installation_id, component_binding_id)` 隔离 session；
- 每个 request 重新执行用户、installation 与 component-binding authorization；
- 不把 session ID 当作认证；
- 设置 idle/absolute expiry；
- 断线时清理 in-flight request；
- 避免跨 replica 混用无 owner 绑定的 session state。

由于 future MCP 正在改变 session/lifecycle，应用层不得依赖 session ID 作为 task identity。transport adapter 负责 protocol-version-specific 行为。

## 15. Tool output 边界

MCP output 是 untrusted external input，即使调用成功也不能直接视为事实或安全指令。

中央应：

- 校验 structured output 是否符合 output schema；
- 限制 text/image/audio/resource 大小与数量；
- 对 URI、MIME 与 embedded resources 再验证；
- 标注 binding、tool、invocation ID 与时间；
- 将 tool result 与 system/developer instructions 隔离；
- 不执行 output 内嵌 shell、HTML script 或 tool-call 文本；
- 在 UI 中展示 provenance。

Prompt injection 不是 pure-text skill 独有风险；remote MCP output 同样可能携带恶意指令。

## 16. Multi-tenancy 与 audit

每次 invocation 至少记录：

```text
invocation_id
user_id
agent/task_id
plugin_installation_id
plugin_package_version
component_binding_id
component_id
component_version
effective_deployment_version
execution_identity
capability_manifest_hash
tool_name
schema_hash
grants_version
risk_class
confirmation_id?
started_at
finished_at
status
size/latency counters
```

不得记录：

- credential；
- full authorization header；
- 默认完整 tool payload；
- 不必要的校园敏感数据。

如为 debug 临时记录 payload，必须显式 opt-in、短 TTL、字段级 redaction，并默认关闭。

## 17. Installation authority 与 binding runtime lifecycle

用户可见 install/enable/disable/revoke 状态只属于 `PluginInstallation`。Binding runtime projection 建议状态：

```text
PendingProvision
ValidatingEndpoint
Initializing
AwaitingInstallationGrant
Ready
NeedsReapproval
AuthExpired
Unhealthy
Draining
Retired
```

关键规则：

- `NeedsReapproval` 不得调用变化后的 tool；
- `AuthExpired` 不得无限重试；
- installation disabled/revoked 时 gateway 立即阻止新调用并使 binding drain/retire；
- private package deletion 删除 secret 后，task 保留可解释的 dangling-reference error；
- health check 不调用 destructive tool。

## 18. Official/market hosted MCP

`Official/VerifiedMarket` 不等于绕过权限系统，也不自动等于 shared/warm。它仍应：

- 发布 immutable/versioned manifest 与 artifact digest；
- 保存 publisher、source revision、license、SBOM/provenance 与 review state；
- 声明 tools、risk class、resource 与 egress profile；
- 经过 operator artifact/runtime review；
- 使用相同 invocation audit 与 per-user grants；
- 为 write/destructive operation 提供确认；
- 支持 disable/revoke/rollback/drain；
- 只有通过 `SharedSafe` cross-tenant gate 才能共享 deployment；
- 只有通过 warm eligibility 与 quota/demand gate 才能设置 `min_replicas > 0`。

Official MCP Registry listing 只是 metadata/provenance signal。其 moderation policy 明示 minimal-to-no moderation，不能作为 USTC market 的安全背书。

## 19. Future relay extension

未来可新增：

```text
UserDeviceRelay
UserAlwaysOnRelay
```

它们应继续实现同一 `McpExecutor` contract：

```text
connect(binding)
discover(binding)
call(resolved_installation_component, tool, arguments)
cancel(invocation)
health(binding)
```

因此 Agent、task、grants 与 audit 不需要重写，只替换 execution location；installation/component/digest resolution 仍由中央 gateway 完成。

Future relay 适用于：

- endpoint 只在用户私网可达；
- secret 不愿上传中央；
- local MCP；
- 用户需要本地确认。

这些需求先通过真实用户统计验证，再决定实现优先级。

## 20. 中央服务边界

Authority plane 保持 modular monolith + workers；不可信 runtime plane 独立。建议边界：

```text
McpBindingService
SecretService
McpExecutorRegistry
McpClientPool
McpCatalogService
McpArtifactAdmissionService
ToolCatalog
ToolPolicyEngine
ConfirmationService
AuditService
McpGateway
McpRuntimeController
```

可独立运行的 worker 负责：

- outbound Streamable HTTP 与 hosted runtime connection；
- secret 解密；
- timeout/cancellation；
- SSRF egress isolation；
- protocol session；
- result size limits。

API server 不应直接持有任意 outbound 网络权限、Docker socket 或 orchestrator admin credential。Runtime Controller 只接受 typed、digest-pinned、policy-approved deployment spec。

## 21. Rust implementation baseline

官方 Rust SDK：

```text
modelcontextprotocol/rust-sdk
crate: rmcp
```

截至 2026-07-21，GitHub latest stable release 为 `rmcp-v2.2.0`，发布于 2026-07-08；release notes 包含 `2025-11-25` conformance audit 修复。该 release 的 Cargo workspace metadata 为 Apache-2.0，repository LICENSE 同时说明项目正由 MIT 向 Apache-2.0 迁移、尚未获 relicensing consent 的旧贡献仍适用 MIT。不得把它笼统标成项目原创 MIT code。

工程建议：

- pin stable tag/crate version，不跟随 `main`；
- 启用 client 与 Streamable HTTP client 所需的最小 feature set；
- 默认使用 rustls；
- 将 SDK type 隔离在 infrastructure adapter 内；
- domain 层只见 `PluginInstallation`、`McpBinding` runtime projection、`ToolSnapshot`、`ToolGrant` 与 typed result；
- 升级 SDK 前运行协议 compatibility 与 SSRF regression tests。
- 在 `THIRD_PARTY_NOTICES.md` 中记录所 pin revision 的 transition license 与 notices。

具体 crate version 应在 implementation spike 时重新核对并锁入 `Cargo.lock`。

## 22. Core `demo`、conditional `demo-hosted` 与不做

### Core `demo` 必做

- 一个 first-party hosted MCP；
- 一个 `UserRemote` Streamable HTTP MCP；
- `none | static_bearer` auth；
- encrypted credential storage；
- endpoint validation + network egress policy；
- initialization 与 `tools/list` discovery；
- capability snapshot/hash；
- per-tool grants；
- write/destructive confirmation；
- scheduled-task unattended restriction；
- typed call broker；
- timeout/cancel/size/rate limits；
- revoke/delete/audit；
- schema-change reapproval。

### Conditional `demo-hosted`（Risk Spike A GO 后）

只有 `MKT-HOST-001` 形成显式 GO decision 后，以下内容才进入 committed `demo-hosted`；在此之前不得反向阻塞 core `demo`：

- 一个 OCI-digest `UserHostedPrivate` MCP，以 `DedicatedUser + OnDemand + max_replicas=1` 运行；
- artifact admission state、digest、revoke 与 runtime provenance；
- cold-start singleflight、readiness、bounded queue、idle drain/stop；
- runtime resource quota、egress isolation 与 cross-tenant negative test；
- workload 无 DB、secret master、metadata endpoint 或 runtime admin API reachability。

### 不做

- authority host 上的 local `stdio` launcher；
- arbitrary command 或 user-controlled Docker options；
- 未经 isolated build/import 的在线 package install；
- OAuth 2.1 full flow；
- user-device relay；
- always-on relay；
- resources/prompts/sampling/elicitation 全能力；
- trust-all-tools；
- destructive scheduled tool call；
- “registry listing 自动安全”或“所有 MCP 永远 3–5 replicas”的承诺。

## 23. Acceptance closure

### Core `demo`

至少证明：

1. first-party hosted MCP 完成 discovery 与一次 read-only tool call；
2. 用户连接 `UserRemote`，系统创建 private PluginPackage installation 与 component binding；
3. DB 与 logs 中无 credential 明文；
4. loopback/private/link-local/metadata endpoint 被拒绝；
5. redirect 或 auth metadata 指向内网时被拒绝；
6. 连接测试只 discovery，不调用业务 tool；
7. 未授权 tool 调用被拒绝；
8. arguments 不符合 schema 时在出网前失败；
9. write/destructive tool 必须逐次确认；
10. scheduled task 无法调用 interactive-only tool；
11. tool schema 变化后 binding 进入 `NeedsReapproval`；
12. credential 撤销后调用得到确定错误；
13. tool result 带 binding/tool/invocation provenance；
14. 不发生 silent MCP fallback；

### Self-hosted `release`

15. self-hosted Compose 能以相同 policy 运行独立 authority。

### Conditional `demo-hosted`（Risk Spike A GO 后）

16. private hosted MCP 的并发首请求只触发一次 cold start；
17. A 用户无法访问 B 用户的 deployment、session、volume 或 secret；
18. revoked/unapproved digest 无法启动；
19. runtime 无法访问 DB、secret master、metadata endpoint 或 runtime admin API；
20. market registration 未通过 warm gate 时仍保持 `min_replicas=0`。

Gate membership 与 explicit deferral 以 [`platform-acceptance-matrix.md`](platform-acceptance-matrix.md) 为准；`UserHostedPrivate` 未经 Risk Spike A GO 不进入 core `demo`。

## 24. 事实来源

- MCP released specification `2025-11-25`：<https://modelcontextprotocol.io/specification/2025-11-25>
- Streamable HTTP transport：<https://modelcontextprotocol.io/specification/2025-11-25/basic/transports>
- MCP tools 与 security considerations：<https://modelcontextprotocol.io/specification/2025-11-25/server/tools>
- Remote MCP connector overview：<https://modelcontextprotocol.io/docs/develop/connect-remote-servers>
- MCP security best practices：<https://modelcontextprotocol.io/docs/tutorials/security/security_best_practices>
- Official Rust SDK：<https://github.com/modelcontextprotocol/rust-sdk>
- `rmcp-v2.2.0` release：<https://github.com/modelcontextprotocol/rust-sdk/releases/tag/rmcp-v2.2.0>
- MCP Registry overview：<https://modelcontextprotocol.io/registry/about>
- MCP Registry moderation policy：<https://modelcontextprotocol.io/registry/moderation-policy>
- MCP Registry package types：<https://modelcontextprotocol.io/registry/package-types>
- OWASP SSRF Prevention Cheat Sheet：<https://cheatsheetseries.owasp.org/cheatsheets/Server_Side_Request_Forgery_Prevention_Cheat_Sheet.html>
- Model provider 与 shared secret policy：[`model-provider-policy.md`](model-provider-policy.md)
- MCP market hosting、artifact、sandbox 与 autoscaling policy：[`mcp-market-hosting-policy.md`](mcp-market-hosting-policy.md)
- Agent runtime 采用与 license/provenance policy：[`agent-runtime-adoption-policy.md`](agent-runtime-adoption-policy.md)
