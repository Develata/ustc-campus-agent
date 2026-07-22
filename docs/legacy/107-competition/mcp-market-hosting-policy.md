# MCP Market、托管与弹性运行 Policy

- 状态：**MCP runtime 安全修正版；Market/PluginPackage 上层本体已由 Develata 确认，runtime 仍待团队确认与 spike**
- 更新时间：2026-07-21
- 适用对象：USTC 个人校园 Agent 平台
- 关联文档：[`mcp-binding-policy.md`](mcp-binding-policy.md)

## 1. 结论

用户提出的三类路径均有产品价值：

1. `PluginPackage` 中的公共 MCP component 由平台统一部署，学生通过已安装 Plugin 的 gateway binding 调用；
2. 用户已有 remote MCP 时，中央直接连接；
3. 用户需要本地 package/stdio MCP 时，平台为其托管并按需冷启动；需要常驻能力时进入更严格的 market review。

**但原始表述不能直接实现。** 必须修正三点：

- “在 registry/market 登记”不等于“代码安全、可共享、可常驻”；
- “上传 MCP config”不能等于“允许用户向服务器提交任意 command/args”；
- “始终为每个 MCP 运行 3–5 个实例”会造成线性资源浪费，也扩大供应链和横向移动攻击面。

推荐实现为：

> **一个中央 MCP gateway + 多种隔离 runtime class + immutable artifact admission + policy-driven autoscaling。**

Market 的一等安装单位不是裸 MCP，而是 [`agent-market-architecture.md`](agent-market-architecture.md) 定义的 `PluginPackage`。本章只拥有 MCP component 的 artifact/runtime policy，不拥有 Plugin 安装、默认启用或版本升级语义。

## 2. 四个必须独立的轴

MCP 的 catalog、runtime 与 availability 不得压成一个 enum。

### 2.1 Catalog trust

```text
FirstParty
VerifiedMarket
CommunityListed
PrivateUser
ExternalRemote
```

### 2.2 Execution location

```text
PlatformHosted
ExternalRemote
FutureUserRelay
```

### 2.3 Tenant mode

```text
Shared
DedicatedUser
```

### 2.4 Availability class

```text
OnDemand       # min_replicas = 0
Warm           # min_replicas >= 1
```

因此：

```text
market publication ≠ security approval
security approval  ≠ shared-safe
shared-safe        ≠ warm entitlement
warm entitlement   ≠ public visibility
```

## 3. 推荐 runtime classes

| Runtime class | 来源 | Tenancy | Availability | 适用条件 |
|---|---|---|---|---|
| `OfficialShared` | First-party / Verified market | Shared | OnDemand 或 Warm | 通过 `SharedSafe` review |
| `OfficialDedicated` | First-party / Verified market | DedicatedUser | OnDemand | 有全局状态、非 tenant-aware 或使用用户私密凭据 |
| `UserRemote` | 用户提供 HTTPS endpoint | owner-scoped binding | 由远端决定 | 中央只做代理、权限和审计 |
| `UserHostedPrivate` | 用户提交 approved artifact | DedicatedUser | OnDemand | 仅个人使用，`min_replicas=0` |
| `MarketWarm` | Verified market | Shared 或 DedicatedUser | Warm | 额外通过运行时审查、配额与 SLO gate |

`OfficialShared` 是一个逻辑 deployment；其后可以有多个 replica。它不意味着“所有用户进入同一个无租户隔离的进程状态”。

## 4. “3–5 个常驻实例”的修正

### 不建议

若含义是“market 中每个 MCP 永远保持 3–5 个 running replicas”，则不采用。原因：

- 资源成本随 listing 数线性增长；
- 长尾 MCP 几乎无流量却长期占用 CPU、memory 与 connection；
- 每个常驻 artifact 都增加持续攻击面；
- stateful MCP 的 replica 扩展还可能破坏 session 语义；
- 恶意 publisher 可借 listing 消耗平台资源。

### 建议

把数字放在两类更合理的 policy 上：

1. **全局或每个 runtime node 保留 3–5 个 warm capacity slots**，用于吸收冷启动峰值；
2. **单个高流量、已验证的 official MCP** 可根据观测数据设置 `min_replicas=3..5`。

每个 deployment 独立声明：

```text
DeploymentPolicy {
  tenant_mode
  min_replicas
  max_replicas
  target_concurrency
  startup_timeout
  request_timeout
  idle_timeout
  scale_up_rate
  scale_down_grace
  max_queue_depth
}
```

默认值应是：

```text
private personal MCP: min_replicas = 0
ordinary verified MCP: min_replicas = 0 or 1
popular official MCP:  min_replicas determined by measured demand
```

不得把 market registration 直接映射成 `min_replicas > 0`。

## 5. Official MCP Registry 不是安全背书

官方 MCP Registry 是 metadata repository，不托管 package code；它支持 npm、PyPI、NuGet、OCI、MCPB 与 remote server metadata。

更关键的是，其 moderation policy 明确说明：

- registry 当前处于 preview；
- consumers 应假定 minimal-to-no moderation；
- 低质量、有 bug、甚至存在 security vulnerabilities 的 server 通常不会仅因此被移除；
- 安全扫描主要留给 package registry 与 downstream marketplace。

因此 USTC market 可以 import registry metadata，但必须建立自己的 trust tier、artifact admission 和 revoke policy。`ListedInOfficialRegistry` 只能是 provenance signal，不能等价为 `VerifiedMarket`。

## 6. SharedSafe gate

公共 MCP 只有通过以下 gate 才可使用 `Shared` tenancy：

- 无跨用户共享的 mutable global state，或所有状态以不可伪造的 tenant key 分区；
- session 以 `(user_id, binding_id, deployment_version)` 隔离；
- shared process 不接收长期 per-user secret；只允许 short-lived、audience/scope-bound invocation credential。若下游只支持长期用户 secret，则改用 `DedicatedUser`；
- persistent storage 按 tenant 分区并有双层 identity check；
- tool result、log、trace 不泄露其他 tenant 的 payload；
- request cancellation、timeout 与并发执行不会串线；
- 没有用户可控 host path、socket、namespace 或 arbitrary file mount；
- 通过并发 cross-tenant negative tests；
- rollback 后旧 replica 不再接收流量。

无法证明时，默认 `DedicatedUser`。**Public 不推出 Shared；Official 也不推出 Shared。**

## 7. 用户上传不能是 arbitrary command execution

典型 MCP config 含有：

```text
command
args
env
cwd
```

若服务器直接执行它，功能本质上就是 unauthenticated remote-code-execution-as-a-service。普通用户不得提交任意 shell、package-manager command、host path 或 Docker options。

### Demo 建议只接受

```text
PrivateHostedArtifact {
  server_manifest
  oci_image_digest
  declared_transport
  declared_tools?
  secret_schema
  resource_profile
  egress_profile
}
```

规则：

- OCI image 必须按 digest pin；tag 只用于发现，不用于执行；
- 用户只提交 secret reference/schema，不在 manifest 中放 secret value；
- runtime 只接受平台定义的 resource/egress profile；
- 不允许 `--privileged`、host PID/network、host mount、device、Docker socket；
- 不允许用户覆盖 entrypoint 为任意 shell；
- stdio MCP 可以在 sandbox 内部运行，由 gateway 转换为内部受控连接；
- 对 npm/PyPI/MCPB 等 package，先在隔离 build service 中转换成 immutable OCI artifact，不能在 production request path 上运行 `npx -y`、`uvx` 或在线安装。

若比赛期无法建立隔离 build pipeline，则 private hosting 首版只接收已构建 OCI digest。

## 8. Artifact admission pipeline

```text
Submitted
  -> MetadataValidated
  -> Namespace/PublisherVerified
  -> SourceAndLicenseResolved
  -> IsolatedBuildOrImport
  -> SBOMAndSecretScan
  -> VulnerabilityAndPolicyScan
  -> BehavioralSmoke
  -> HumanReviewIfRequired
  -> DigestPinned
  -> Signed/Attested
  -> Staged
  -> Active
```

隔离 build/import job 不得获得 production secret、authority credential 或 unrestricted internal network；依赖下载若必须开放，应经固定 proxy/allowlist，并把 resolved lockfile、source revision 与下载 digest 写入 provenance。

至少保存：

```text
McpArtifact {
  artifact_id
  publisher_id
  source_url
  source_revision
  upstream_license
  image_digest
  sbom_ref
  provenance_ref
  signature_ref
  public_catalog_ref?
  catalog_state_projection?       # public only; authority is reviewed Git
  private_admission_state?        # private only; authority is PostgreSQL
  runtime_admission_state
  shared_safe_state_projection?
  deployment_blocked_at?          # deny-only operational overlay
}
```

Public artifact 与 private artifact 的 review truth 不得混用：

```text
PublicCatalogApproved = Git catalog verified and not catalog_revoked
PrivateAdmissionApproved = PostgreSQL private admission approved
RuntimeAllowed = runtime_admission_state == Approved
                 AND deployment_blocked_at is null
```

Public artifact 必须满足 `PublicCatalogApproved AND RuntimeAllowed`；private artifact 必须满足 `PrivateAdmissionApproved AND RuntimeAllowed`。Git revoke 必须投影为 runtime blocked；PostgreSQL emergency block 可立即 deny，但任何 admin/runtime row 都不能 override Git revoke。Artifact store 不拥有 review/revoke authority。

SLSA provenance 用于回答 artifact 在何时、何处、如何产生；Cosign/signature 用于验证正在运行的 digest 与批准 artifact 相同。签名不证明代码无恶意，但可防 artifact 被无声替换。

## 9. Runtime sandbox baseline

运行用户或 community code 的 runtime plane 必须与 API/data authority 分离。最低基线：

- dedicated runtime node/VM 或独立 cluster boundary；
- non-root/rootless runtime；
- `cap-drop=ALL`；
- `no-new-privileges` / `allowPrivilegeEscalation=false`；
- read-only root filesystem，仅提供受限 tmpfs；
- default seccomp + AppArmor/SELinux；
- 无 host mount、无 Docker socket、无 device；
- CPU、memory、PID、file descriptor、ephemeral disk、network 与 wall-clock quota；
- egress deny-by-default；
- 无公网 inbound port，所有流量经 MCP gateway；
- MCP workload 不挂载 orchestrator API token，也没有创建/修改 workload 的权限；
- per-user persistent volume 独立、可审计、可删除；
- secret 由 short-lived broker 注入，禁止进入 image、build log、command line 与普通 telemetry；
- catalog revoke、private admission revoke 或 operational emergency block 后停止新 session，并在 grace period 后终止旧 replica；allow/deny resolution 遵循上一节 authority，不由 artifact store 决定。

普通 Linux container 共享 host kernel；Kubernetes namespace 也不是强安全边界。若长期开放任意用户 code，应评估 dedicated nodes、RuntimeClass、gVisor、Kata Containers 或 microVM；比赛 Demo 至少必须将 runtime 与 authority/DB/secret master key 从网络和权限上隔离。

## 10. Control-plane 权限

禁止：

```text
public API container
  -> /var/run/docker.sock
  -> arbitrary docker run
```

Docker socket access 接近 host root 权限；即使只读 mount 也不是可靠安全边界。

建议：

```text
API / Domain Authority
  -> deployment request queue
  -> narrowly privileged Runtime Controller
  -> orchestrator API with least-privilege service account
```

Runtime Controller 只接受 typed deployment spec，并再次执行：

- artifact digest allowlist；
- tenant ownership；
- runtime profile allowlist；
- quota；
- network policy；
- secret-reference ownership；
- admission state；
- idempotency key。

## 11. Compose 与 autoscaling 的现实边界

Docker Compose 能声明或手动调整固定 replicas，但不是 demand-driven scale-to-zero control plane。

有两条诚实路线：

### Route A：比赛期 production-shaped

- central product stack 仍以 Compose 交付；
- untrusted MCP runtime 放入独立的 K3s/Kubernetes runtime node；
- 使用 restricted Pod Security、NetworkPolicy 与受限 controller；
- HTTP/queue-driven scale-to-zero 可参考 KEDA/KEDA HTTP Add-on。

优点：autoscaling 与 isolation 语义更真实。代价：infra 复杂度明显增加。

### Route B：比赛期 bounded demo

- Compose 运行 central stack；
- 只运行少量预先审核、预先安装的 official OCI artifacts；
- 一个独立 rootless runtime controller 实现 bounded `start/stop`；
- private hosting 只演示 `min=0, max=1` 的 dedicated cold start；
- 不宣称已经达到 arbitrary multi-tenant code hosting 的生产安全等级。

**若没有独立 runtime node，Route B 更适合比赛。** 不应为了展示 autoscaling 而把 Docker socket暴露给公网 API。

## 12. Gateway contract

所有 hosted/remote MCP 都通过同一个 typed gateway：

```text
McpGateway.call(
  user_id,
  plugin_installation_id,
  component_binding_id,
  tool_name,
  arguments,
  expected_capability_manifest_hash,
  expected_grants_version,
  invocation_context,
)
```

Gateway 负责：

- identity、installation enabled/revoked state、binding ownership 与 tool grant；
- exact package/component version、effective deployment version、artifact digest、capability manifest hash 与 grant version resolution；
- deployment lookup 与 cold-start singleflight；
- queue depth、rate、concurrency 与 tenant quota；
- ready/health gate；
- timeout、cancellation 与 retry budget；
- secret lease；
- session routing；
- schema validation；
- output size/provenance；
- audit receipt。

Audit receipt 必须记录 resolved `plugin_installation_id/package_version/component_id/component_version/effective_deployment_version/artifact_digest/capability_manifest_hash/grants_version`。Canary 必须先改变 cohort installation 的 desired exact version，再路由到对应 deployment；不得让 pin 旧 digest 的 installation 随机命中新 artifact。

pod/container endpoint 不直接暴露给用户或 Agent。Agent 不拥有 deployment-controller capability。

## 13. Cold-start correctness

冷启动路径必须处理：

```text
Absent
  -> Starting
  -> Probing
  -> Ready
  -> Draining
  -> Stopped
```

并满足：

- 同一 `(artifact, tenant)` 的并发首请求使用 singleflight，不重复创建多个 replica；
- startup 有 deadline，失败有指数 backoff 与熔断；
- 请求在 bounded queue 中等待，超限立即失败；
- readiness 只做 initialization/`tools/list`，不调用业务 tool；
- startup retry 不重复执行用户 tool；
- scale-down 先 drain in-flight invocation；
- stateful MCP 的 session 必须 sticky 或外置；
- scheduled task 可在 deadline 前 prewarm，但不能绕过 grant/review。

## 14. Warm eligibility

MCP component 所属 `PluginPackage` 进入 verified catalog，可以作为申请 warm tier 的必要条件，但不应是充分条件。推荐：

```text
WarmEligible =
  VerifiedMarket
  AND ArtifactPinned
  AND RuntimeReviewPassed
  AND ResourceProfileApproved
  AND NoUnresolvedCriticalFinding
  AND (ObservedDemand OR OperatorGrant)
```

对 shared warm deployment 还必须 `SharedSafePassed`。

若未来允许 private dedicated warm instance，应以显式 quota/operator grant 管理；不必强迫用户把私有 MCP 公开发布。比赛 MVP 可以暂时只向 verified market 开放 warm tier，但文档应说明这是产品策略而非安全定理。

## 15. 主要攻击面与控制

| 风险 | 典型路径 | 必要控制 |
|---|---|---|
| Host RCE | arbitrary command、Docker socket | immutable artifact、typed controller、无 socket |
| Container escape | kernel/runtime 漏洞 | isolated node、rootless、seccomp/LSM、及时更新 |
| SSRF / 内网横移 | remote endpoint、hosted tool egress | app + network 双层 egress policy |
| Cross-tenant leak | shared cache/session/files | SharedSafe gate、physical partition + identity check |
| Secret exfiltration | env/log/tool output | secret broker、redaction、egress policy、short lease |
| Supply-chain substitution | mutable tag、online install | digest pin、isolated build、SBOM、provenance、signature |
| Cold-start DoS | burst registration/invocation | quotas、singleflight、bounded queue、backoff、max replicas |
| Cryptomining/resource abuse | personal hosted code | CPU/memory/PID/network/wall-time quota、abuse suspension |
| Prompt/tool injection | malicious descriptions/results | tool grant、untrusted result boundary、confirmation |
| Confused deputy | shared operator credential | per-user delegated auth、audience binding、no token passthrough |
| Persistence abuse | hidden volume/state | dedicated volume、retention/delete、no host path |
| Rollback ambiguity | old replicas continue | digest-routed deployment version、drain/revoke |
| License violation | untracked package/source | license resolution、notices、source/revision provenance |

## 16. Demo 推荐切片

### Core `demo`

- 三个默认 read-only `FirstPartySystemPlugin` 中至少一个所引用的 first-party hosted MCP，经 Plugin installation 与 gateway 完成 execution closure；三个 Plugin 的产品价值验收仍由 platform `demo` matrix 约束；
- 一个 `UserRemote` Streamable HTTP MCP；
- market metadata、artifact digest、review state 与 revoke；
- SSRF/egress negative test；
- resource quota 与 timeout；
- user-visible provenance 和 audit。

### Conditional `demo-hosted`（Risk Spike A GO 后）

- 一个 `UserHostedPrivate` OCI-digest MCP，`DedicatedUser + OnDemand + max=1`；
- cold-start singleflight、readiness、idle stop；
- cross-tenant isolation negative test；
- dedicated workload sandbox/egress/quota/revoke evidence。

Risk Spike A 未形成 GO decision 前，`demo-hosted` 不阻塞 core `demo`；NO-GO 时必须显式 deferred，不能把 unavailable 记为 Pass。

### Stretch

- KEDA/K3s 自动扩缩；
- npm/PyPI/MCPB isolated build；
- Cosign keyless signing；
- community publishing；
- private warm quota；
- gVisor/Kata/microVM。

### 明确不宣称

- official registry listing 自动安全；
- arbitrary user code production-safe；
- Compose 自带完整 autoscaling；
- container 等价于强虚拟机隔离；
- 3–5 replicas 对所有 listing 都合理。

## 17. 验收闭环

至少证明：

1. private hosted MCP 在零 replica 时首请求只触发一次 startup；
2. readiness 前请求不进入 tool body；
3. idle timeout 后 replica 停止；
4. A 用户无法路由到 B 用户的 deployment、session、volume 或 secret；
5. shared deployment 使用两个测试 tenant 时无 state/result 串线；
6. artifact tag 变化但 digest 未批准时拒绝启动；
7. revoked artifact 无法建立新 session；
8. hosted runtime 无法访问 DB、secret service、runtime controller 或 metadata endpoint；
9. CPU/memory/PID/queue quota 生效；
10. startup timeout 不会触发 tool 重放；
11. market registration 未通过 warm gate 时 `min_replicas` 仍为零；
12. API container 无 Docker socket、host mount 或 orchestrator admin credential。

## 18. 事实来源

- MCP Registry overview：<https://modelcontextprotocol.io/registry/about>
- MCP Registry moderation policy：<https://modelcontextprotocol.io/registry/moderation-policy>
- MCP package types：<https://modelcontextprotocol.io/registry/package-types>
- MCP remote servers：<https://modelcontextprotocol.io/registry/remote-servers>
- MCP security best practices：<https://modelcontextprotocol.io/docs/tutorials/security/security_best_practices>
- Docker Engine security：<https://docs.docker.com/engine/security/>
- Docker rootless mode：<https://docs.docker.com/engine/security/rootless/>
- Docker daemon socket protection：<https://docs.docker.com/engine/security/protect-access/>
- Kubernetes Pod Security Standards：<https://kubernetes.io/docs/concepts/security/pod-security-standards/>
- Kubernetes multi-tenancy：<https://kubernetes.io/docs/concepts/security/multi-tenancy/>
- KEDA scaling deployments：<https://keda.sh/docs/2.20/concepts/scaling-deployments/>
- KEDA HTTP scale-to-zero example：<https://keda.sh/http-add-on/0.15/getting-started>
- SLSA provenance：<https://slsa.dev/spec/v1.2/provenance>
- Sigstore Cosign verification：<https://docs.sigstore.dev/cosign/verifying/verify/>
- OWASP Docker Security Cheat Sheet：<https://cheatsheetseries.owasp.org/cheatsheets/Docker_Security_Cheat_Sheet.html>
- OWASP SSRF Prevention Cheat Sheet：<https://cheatsheetseries.owasp.org/cheatsheets/Server_Side_Request_Forgery_Prevention_Cheat_Sheet.html>
