# USTC Agent Market Architecture

## Metadata

- `Layer`: `Market Authority / Product Platform`
- `Status`: **Develata-confirmed architecture candidate；待团队确认与真实基础设施验证**
- `Version`: `0.2.0`
- `Last Review`: `2026-07-21`
- `Authority Owns`: `catalog source-of-truth split / PluginPackage ontology / install-enable-grant lifecycle / Market Web boundary / update policy`
- `Authority Defers To`: [`mcp-market-hosting-policy.md`](mcp-market-hosting-policy.md), [`mcp-binding-policy.md`](mcp-binding-policy.md), [`central-agent-client-relay-marketplace.md`](central-agent-client-relay-marketplace.md), [`affairs-changeradar-knowledge-architecture.md`](affairs-changeradar-knowledge-architecture.md)

## 1. Scope

本章定义 USTC Agent Market 的产品本体与 authority boundary：

- public Git catalog；
- `PluginPackage` 与 component；
- Market Web、Auth 与 PostgreSQL；
- publish/install/configure/enable/invoke；
- first-party default plugins；
- permissions、updates、revocation；
- initial English/Chinese contract。

本章不定义 hosted MCP sandbox、scale-to-zero 或 arbitrary user code 安全实现；这些由 [`mcp-market-hosting-policy.md`](mcp-market-hosting-policy.md) 拥有。

## 2. Product definition

USTC Agent Market 不是四个 Git repository 的文件浏览器，也不是直接执行 artifact 的 runner。它是：

```text
Git-backed Catalog
+ PluginPackage Registry
+ Browse / Install Web
+ User Installation Authority
+ Admin Review / Publish / Revoke Workflow
+ Runtime Deployment Projection
```

Market 回答：

- 有什么能力；
- 由谁发布；
- 当前批准哪个版本和 digest；
- 由哪些 component 组成；
- 需要什么 capabilities；
- 用户是否安装、启用；
- 应绑定到什么 runtime deployment。

真正的 MCP/CLI/service 执行仍由 Agent tool gateway 与 runtime plane 完成。

## 3. Confirmed decisions

| Decision ID | Decision | Status |
|---|---|---|
| `MKT-AUTH-001` | Git 是 catalog/version/review/default-policy authority；PostgreSQL 是 user/install/grant/runtime/audit authority | Develata confirmed |
| `MKT-PKG-001` | `PluginPackage` 是唯一一等安装单位；可组合 MCP、Skill、ControlledCLI、SharedServiceBinding | Develata confirmed |
| `MKT-PERM-001` | 三个 default first-party Plugin 仅声明 Capability Registry 中 AutoGrantEligible 的精确 read capabilities；默认启用并授予 manifest 全部声明 | Develata confirmed |
| `MKT-WEB-001` | Market frontend/entry 独立，但与 Agent 平台共享 Auth、PostgreSQL 与 Market backend | Develata confirmed |
| `MKT-VIS-001` | catalog/schema/source repositories 位于 GitHub public；runtime/admin/secrets 仅内网 | Develata confirmed |
| `MKT-UPD-001` | installation pin exact version/digests；无权限扩张 verified patch 可灰度自动更新，其余重新批准 | Develata confirmed |
| `MKT-IDP-001` | 使用 `IdentityProvider` adapter；生产优先 USTC 统一认证；local admin 仅 break-glass | Develata confirmed |
| `MKT-I18N-001` | 初始支持 `en-US` 与 `zh-CN`；开发 English-first，中文用户面同步 | Develata confirmed |
| `MKT-DEV-001` | 8C16G 服务器以 SSH 驱动的隔离 worktree 为主；重 Rust gates 使用 107 Slurm | Develata confirmed |
| `MKT-VER-001` | 配置、诊断、catalog/package inspection 与验收尽可能 Rust CLI 化；必须有 configuration smoke 与完整验收矩阵 | Develata confirmed |
| `MKT-HOST-001` | `UserHostedPrivate` 只在 Risk Spike A 明确 GO 后进入 committed MVP 与 conditional `demo-hosted` gate；core `demo` 不预先依赖它 | Conditional until Risk Spike A |
| `MKT-RES-001` | `DeclarativeResourcePack` 作为 PluginPackage 内 exact-pinned experimental resource，承载 source/tree/policy/schema/renderer；不具备独立执行/grant authority | Develata confirmed |

“Develata confirmed”不等于团队已达成共识，也不证明真实服务器已具备 Docker/K3s、DNS/TLS 或 SSO application access。

## 4. Authority and storage

### 4.1 Authority is not a database brand

Authority 表示冲突时谁决定真值。若 public Git manifest、PostgreSQL row 与 admin UI 不一致，系统必须能确定修复方向。

### 4.2 Canonical split

```text
Git manifests
  authority for:
  - package/component metadata
  - versions and immutable digests
  - capabilities/permissions declaration
  - publisher/source/license/provenance
  - review/verified/revoked catalog state
  - first-party/default-install policy

PostgreSQL
  authority for:
  - users and roles
  - private user-only PluginPackage definitions/admission
  - installations
  - enabled/disabled state
  - user grants
  - configuration and secret references
  - desired/observed deployments
  - rollout state
  - audit events

OCI registry / artifact store
  authority for:
  - exact executable bytes identified by digest

Redis, if introduced
  authority for:
  - nothing durable
```

即：

```text
CatalogReadModel = Project(ReviewedGitManifests)
```

手工修改 PostgreSQL catalog projection 不得成为有效 publish；下一次 sync 应覆盖、拒绝或 quarantine drift。

用户关闭 Plugin 只修改 PostgreSQL installation state，不反向提交 Git。

Public catalog 与 private import 必须分开：

- public `PluginPackage` 的 metadata/review/revoke truth 只来自 reviewed Git；
- 用户上传或连接的 private component 必须生成 user-owned、single/multi-component `PrivatePluginPackage` 与 installation，其 private manifest/admission state 由 PostgreSQL 保存，但永不进入匿名/public catalog；
- PostgreSQL 可设置 `deployment_blocked` emergency deny overlay，立即阻止新 invocation；它只能收紧，不能把 Git 中 revoked/unapproved 的 public artifact 重新放行；
- artifact store 只证明 digest 对应的 bytes 存在，不拥有 review/revoke truth。

```text
EffectiveArtifactAllowed =
  (PublicCatalogApproved OR PrivateAdmissionApproved)
  AND RuntimeAdmissionApproved
  AND NOT DeploymentBlocked
```

### 4.3 Database choices

- central Market 使用 PostgreSQL；
- SQLite 可用于 local development、tests、client-local cache；
- redb 可用于 Rust embedded runtime state 或 immutable index snapshot；
- Redis 只在多 worker rate limit、queue/pub-sub、cold-start singleflight 等需求成立时引入。

清空 Redis 最多产生 cache miss、重试或短暂降级；不得丢失 package、installation、grant、deployment intent 或 audit truth。

## 5. Public repository topology

建议建立 GitHub organization/group：

```text
USTC-AGENT-Market/
```

技术 slug 统一 lowercase；display title 可保留大写。

```text
USTC-AGENT-Market/registry
USTC-AGENT-Market/plugin
USTC-AGENT-Market/mcp
USTC-AGENT-Market/skills
USTC-AGENT-Market/scripts
```

可选：

```text
USTC-AGENT-Market/web
```

职责：

- `registry`：schemas、catalog index、review policy、capability namespaces；
- `plugin`：面向用户的 `PluginPackage` manifests；
- `mcp`：MCP component manifests；
- `skills`：localized Skill component artifacts/manifests；
- `scripts`：`ControlledCliComponent` manifests；raw arbitrary scripts 首期不可执行；
- `web`：若 Market frontend 与 central client 分仓，保存独立 Web build。

Catalog repo 不集中托管所有 implementation source。Publisher 可保留自己的 source repo；catalog manifest 只引用 exact source revision、release、license 与 artifact digest。

不得进入 public Git：

- secrets/credentials；
- private endpoints；
- user installation/configuration；
- private audit payload；
- runtime admin credentials；
- unredacted security reports。

## 6. Package ontology

### 6.1 PluginPackage

`PluginPackage` 是平台唯一一等安装、启用、关闭与升级单位。Public package 可从 Market 安装；private connector/upload 先生成 user-owned `PrivatePluginPackage`，再走同一 installation lifecycle。

```text
PluginPackage
├── McpServerComponent*
├── SkillComponent*
├── ControlledCliComponent*
├── SharedServiceBinding*
└── DeclarativeResourcePack*   # package resource, not an installable component
```

任一 component 数量可以为零，但一个 PluginPackage 至少包含一个 component。`DeclarativeResourcePack` 不计入 component 数量；仅含 resource pack 的 package 不能获得 installation/runtime identity。

`PluginPackage` 不是 in-process dynamic plugin，不意味着把 `.so`、Rhai、WASM 或脚本直接加载进 Agent authority process。

### 6.2 Components

#### McpServerComponent

- remote 或 platform-hosted MCP；
- exact transport、tool snapshot、artifact/deployment policy；
- shared/dedicated 与 cold/warm 由 MCP runtime policy 决定。

#### SkillComponent

- prompt/procedure/knowledge contract；
- 可引用 typed tools；
- 不携带 arbitrary executable authority；
- locale variants 独立 versioned/reviewed。

#### ControlledCliComponent

- platform-owned/reviewed Rust CLI；
- fixed binary path 与 exact digest；
- typed subcommands/arguments；
- 无 arbitrary shell、PATH search 或用户覆盖 entrypoint；
- timeout、output、concurrency、scope、audit 全部受 gateway 控制。

#### SharedServiceBinding

- 引用平台运营的共享数据/索引/图服务；
- installation 创建 tenant/user binding，不复制一套服务；
- shared service 的 operator write authority 与用户 Plugin permissions 分开。

#### DeclarativeResourcePack（experimental）

- 承载 exact-pinned source registry fragment、wiki tree、board policy、schema 与 deterministic renderer template；
- 只有 ID/version/digest/provenance，不注册 executable tool，也不获得 network/filesystem/secret capability；
- 由拥有它的 reviewed component 消费；资源启停不得绕过 owning Plugin installation；
- duplicate/conflicting resource ID fail-closed，不使用 silent first-wins；
- 若未来需要独立 install/grant/update/runtime lifecycle，必须重新进入 ontology review，不能静默提升为 component。

### 6.3 Standalone components

一个纯 MCP 或纯 Skill 仍包装成只含一个 component 的 PluginPackage。普通用户只面对统一的 install/enable lifecycle；开发者可浏览 component details。

`McpBinding`、`SharedServiceBinding` 等 binding 不是第二套安装对象；它们必须带 `plugin_installation_id + component_id`，只作为 installation-owned runtime projection。删除/关闭/revoke installation 必须使其所有 component bindings 不可调用。

### 6.4 Minimum manifest shape

```yaml
id: ustc.change-radar
kind: PluginPackage
version: 0.1.0
publisher: ustc-agent
components:
  - kind: McpServerComponent
    ref: ustc.change-radar-query@0.1.0
  - kind: SkillComponent
    ref: ustc.change-radar-assistant@0.1.0
  - kind: SharedServiceBinding
    ref: ustc.change-radar-index@0.1.0
resources:
  - kind: DeclarativeResourcePack
    ref: ustc.change-radar-sources@0.1.0
locales:
  required: [en-US, zh-CN]
capabilities:
  - campus.public_rules.read
  - campus.public_changes.read
install_policy:
  class: FirstPartySystemPlugin
  default_installed: true
  default_enabled: true
  user_disable_allowed: true
update_policy:
  version_pin: exact
```

字段名仍需 schema slice 冻结；本例只固定本体关系。

## 7. First-party system plugins

### 7.1 USTC Affairs Navigator

```text
shared campus procedure data service
+ search/navigation MCP
+ task-oriented Skill
```

首期 capabilities 只允许公共办事流程、官方入口、适用对象、时效与 provenance 的读取。Reviewed tree/procedure/source declaration 位于 Git；PostgreSQL 只保存 crawl/candidate/search projection。Agent 只产生 typed candidate，Demo canonical publish 由管理员完成。

### 7.2 USTC ChangeRadar

```text
operator scheduled ingestion/diff service
+ normalized change database
+ query MCP
+ explanation Skill
```

Crawler 写内部索引属于 operator service authority，不是用户 edit permission。用户 Plugin 只读取 changes/diffs/provenance。

Affairs 与 ChangeRadar 共用 Source Registry、immutable revisions、semantic change ledger 与 board policies。Maintainer Agent 只能在 stable node scope 内提出 candidate；RSS/Atom 只发布 approved semantic changes。完整 contract 见 [`affairs-changeradar-knowledge-architecture.md`](affairs-changeradar-knowledge-architecture.md) 与 [`source-registry.md`](source-registry.md)。

### 7.3 Campus Opportunity Graph

```text
shared opportunity graph service
+ matching/path-planning MCP
+ explanation Skill
+ tenant-scoped user preference projection
```

公共图谱与用户画像分开；shared cache/session 必须 tenant-keyed。

### 7.4 Capability classification authority

Manifest 不能自行把 capability 宣称为“安全只读”。Operator-maintained Capability Registry 独立拥有：

```text
capability_id
owner_domain
risk_class
data_class
auto_grant_eligible
allowed_component_kinds
review_revision
```

首期至少区分：

```text
PublicRead
UserPrivateReadScoped
CrossUserAggregateRead
InternalDiagnosticRead
Write
Destructive
Unknown
```

- `PublicRead` 可进入 default auto-grant allowlist；
- `UserPrivateReadScoped` 只有在对象域极窄、tenant-owned、最小化且被 operator 明确标记 `auto_grant_eligible` 时才可进入 default manifest，例如 `user.own_opportunity_preferences.read`；不得使用泛化的 `user.profile.read` 或 `memory.read_all`；
- `CrossUserAggregateRead`、`InternalDiagnosticRead`、`Write`、`Destructive`、`Unknown` 不得由 default Plugin 自动获得；
- 新 capability、risk/data class 改变、allowlist 改变均 fail-closed，并视为 permission expansion。

### 7.5 Default install and permissions

对新用户：

```text
account bootstrap
  -> exact PluginPackage version installed
  -> enabled = true
  -> validate every declared capability against Capability Registry
  -> grant every declared AutoGrantEligible capability
```

约束：

- default Plugin manifest 首期只能声明 registry 中 `auto_grant_eligible=true` 的 typed read capabilities；manifest 若包含其他 capability 则整个 default bootstrap fail-closed；
- “全部权限”只覆盖当前 manifest，且 tenant-scoped；
- 不包含 platform admin、cross-tenant、arbitrary shell/network/filesystem；
- 用户可关闭并撤销其 grants；
- security revoke 可由 operator 强制 disable，并产生 audit event；
- 若未来加入发送、提交、删除等 mutation capability，必须重新进入 `MKT-PERM-001` architecture review，不能借普通版本更新静默扩大。

## 8. Controlled mutation path

自定义 edit 不在三个 default Plugin 中。Mutation 统一经：

```text
Agent ToolCall
  -> User/Auth/Tenant Scope
  -> Plugin Installation + Capability Gate
  -> Typed ControlledCli Adapter
  -> Rust Domain Operation
  -> Structured Result
  -> Audit Event
```

ControlledCLI 最低要求：

- fixed reviewed binary/digest；
- typed command schema；
- ownership/path/URL/object checks；
- no arbitrary command string；
- timeout/output/concurrency budgets；
- stable error code；
- idempotency key for retryable mutation；
- high-value changes 支持 `plan/preview -> apply`；
- 不在 public API/authority process 内直接执行，由 dedicated low-privilege worker/sandbox 启动；
- scrub environment，不继承 master key、DB/admin、provider/MCP credential 或 broad secret-service token；
- read-only rootfs、scoped working directory、no host path/device/socket；
- egress deny-by-default，只开放 subcommand profile 明确声明的 destination；
- CLI 不直接写 PostgreSQL；mutation 经 typed domain service，并重复 user/tenant/capability/object gate；
- `apply` 必须绑定 preview/plan hash、confirmation（如需要）、idempotency key；
- 每个 subcommand 声明 required capability、tenant/object scope、egress/fs profile。

Rust 减少 memory-safety 和 runtime dependency 风险，不替代 authorization、SSRF、path traversal、business invariant 与 audit controls。

## 9. Lifecycle

```text
Submit
  -> Validate
  -> Review
  -> Publish
  -> Install
  -> Configure
  -> Enable
  -> Invoke
  -> Update | Disable | Revoke
```

必须分开：

- `Publish`：catalog revision 获准；
- `Install`：用户接受 exact PluginPackage version；
- `Configure`：写非秘密配置与 secret references；
- `Enable`：Agent runtime 可以发现该 Plugin；
- `Invoke`：每次 tool call 仍通过 user/session/tenant/capability gate。
- `Private upload/connect`：先生成 private PluginPackage + installation；component binding 不能独立绕过 install/enable/grant lifecycle。

配置 UI 可以后做，但 config schema 与 secret-ref boundary 必须在 installer 前冻结。

## 10. Update and rollback

Installation 保存：

- exact PluginPackage version；
- exact component versions；
- exact component execution identity：hosted executable/artifact digest，或 remote endpoint/server/capability snapshot identity；
- capability manifest hash；
- rollout cohort/current state。

更新规则：

- verified patch 且 capability/permission set 不扩张：可 staged/canary auto-update；
- minor/major、component trust 下降、permission/capability expansion：必须重新批准；
- rollout 必须有 health gate 与 rollback；
- shared service/MCP 先 canary，再 drain 旧 version；
- security revoke 可阻止新 invocation，并在 grace policy 后停止旧 runtime；
- FirstPartySystemPlugin 不得绕过上述规则。

### 10.1 Effective invocation resolution

Gateway 不得只凭 `binding_id + grants_version` 调用。每次 invocation 必须由 server-side resolver 得到并验证：

```text
PluginInstallation
  -> exact PluginPackageVersion
  -> ComponentBinding(component_id, exact component_version)
  -> EffectiveDeploymentVersion
  -> ExecutionIdentity(artifact digest | endpoint/service identity)
  -> CapabilityManifestHash
  -> GrantVersion
```

Canary 不是把已 pin 旧 version 的 installation 随机路由到新 digest；它是将 cohort 的 desired exact version 经 health gate 后原子切换，并保留 rollback target。Gateway/audit receipt 必须记录上述 resolution；任一 catalog revoke、runtime emergency block、installation disable、digest mismatch、capability/grant version mismatch 都 fail-closed。

## 11. Market Web and identity

### 11.1 Deployment boundary

Market frontend 是独立 UI build/entry，但不复制 identity、installation 或 grant authority。

首期推荐同一 HTTPS origin：

```text
https://<agent-host>/market
```

Reverse proxy 将 `/market` 与 Market API 路由到独立 frontend/backend modules；internal service ports 不直接暴露给用户。

### 11.2 Roles

```text
Visitor
User
Publisher
Reviewer
Admin
RuntimeService
```

- Visitor：匿名 read-only catalog projection；
- User：统一认证后管理自己的 installation/config/grants；
- Publisher：提交其拥有的 package revision；
- Reviewer：通过 Git review workflow 写入/批准 review evidence；
- Admin：管理 Git publish/revoke workflow、catalog import、rollout 与 deny-only emergency block；不得用 PostgreSQL row 放行 Git 未批准/已 revoke 的 public package；
- RuntimeService：机器身份，只调用窄 deployment/invocation contract。

普通用户不需要 admin 权限运行自己的已授权 Plugin。

### 11.3 IdentityProvider

```text
IdentityProvider
├── UstcUnifiedIdentityProvider   # production preferred
├── DevelopmentIdentityProvider   # local/dev only
└── LocalAdminBreakGlassProvider  # operator recovery only
```

约束：

- 真实 USTC 协议/application access 仍待基础设施核验；
- local admin 不能成为普通用户 fallback；
- production provider config 必须 pin/validate protocol-appropriate issuer、audience/client ID、exact redirect URI、state/nonce/PKCE or assertion replay controls、session idle/absolute expiry 与 CSRF policy；具体字段随核验后的 OIDC/CAS 协议冻结，不能猜测；
- role mapping deny-by-default，只来自 operator-maintained allowlist 或已核验的权威 identity attributes；Publisher 不得自授 Reviewer/Admin；
- break-glass 使用独立 subject namespace 与独立入口，禁止和普通 USTC subject merge；必须有强审计、rate limit、credential rotation，并在基础设施允许时要求 second factor/two-person recovery；
- `DevelopmentIdentityProvider` 在 production build/config 中 fail-closed；
- platform 不保存用户 USTC 原始密码；
- Market/Agent 共用 stable subject 与 role mapping。

### 11.4 Anonymous surface

Visitor 可读取 name/description/components/publisher/version/license/source/trust/permission summary/docs。

Visitor 不可触发 install/configure/enable/test invocation/deployment/cold start/secret binding/admin diagnostics/private metadata。

## 12. Admin and demo reproducibility

开发者展示 Market 内容时使用：

```text
reviewed seed manifests
+ catalog sync/import
+ admin publish/promote/revoke commands
+ auditable migrations/events
```

不以直接修改 PostgreSQL rows 作为正常 workflow。演示环境应能由 pinned Git revision + schema migration 确定性重建 catalog；emergency override 也必须产生 audit event。

## 13. Internationalization

### 13.1 Canonical split

```text
code identifiers / API / schema / logs: English
user-facing UI: en-US + zh-CN
Market metadata: en-US + zh-CN
engineering workflow: English-first
```

Locale resolution：

```text
user setting -> client/browser preference -> en-US fallback
```

### 13.2 CI gates

- locale key parity；
- placeholder parity；
- no user-facing hardcoded strings；
- required Market locale metadata exists；
- backend returns stable error code，不返回作为协议合同的 localized prose。

### 13.3 Skill localization

```text
skill.en-US.md
skill.zh-CN.md
```

二者共享 logical skill ID、version 与 source revision，但作为独立 reviewed locale artifacts。不得在 invocation path 临时机器翻译安全敏感 prompt。

公开 repo 推荐 `README.md` 为 English canonical，`README.zh-CN.md` 为同步中文用户文档。

## 14. Frontend direction

Design brief：

- subject：campus Agent capability market；
- single job：判断“它做什么、是否可信、需要什么权限”，然后安装/启停；
- visual direction：Apple-informed hierarchy、negative space、typography、content deference；
- palette：neutral canvas + one restrained USTC-derived accent + semantic state colors；
- signature：清晰展示 `PluginPackage -> Components -> Capabilities` 的组成条，而非 generic gradient hero。

主要 routes：

```text
/market
/market/:plugin_id
/publishers/:publisher_id
/account/installations
/admin/review
/admin/runtime
```

Detail page 至少呈现：Overview、Components、Permissions/Data Access、Versions/Compatibility、Publisher/Source/License、Review evidence、Install/Enable state。

禁止把 Apple-like 等价为所有卡片 glass、nested cards、巨大渐变 hero 或无意义动效。Admin shell 与普通用户 Market surface 分开。

## 15. Failure and recovery

必须定义：

- Git sync schema/revision/signature failure；
- PostgreSQL catalog projection drift；
- component digest missing/revoked；
- installation references unavailable version；
- identity provider unavailable；
- patch rollout failed/rollback；
- Plugin disabled while invocation in flight；
- first-party seed bootstrap partial failure；
- locale artifact missing；
- audit write failure；
- runtime desired/observed state divergence。

失败默认 fail-closed：不因 catalog/import/runtime 异常临时绕过 review、digest、grant 或 auth。

## 16. Initial acceptance journey

验收 authority 与 machine entrypoints：

- [`rust-cli-config-smoke-contract.md`](rust-cli-config-smoke-contract.md)；
- [`platform-acceptance-matrix.md`](platform-acceptance-matrix.md)。

任何 integration/demo/release claim 前必须先通过对应 profile 的：

```text
ustc-agentctl config smoke --level live-readonly --profile demo --config config.toml --format json --evidence-out evidence/demo/config-live.json
-> ustc-agentctl acceptance matrix-check --strict --format json --evidence-out evidence/demo/matrix.json
-> ustc-agentctl acceptance run --required-for demo --mode real-host --target competition-demo --format json --evidence-out evidence/demo/acceptance.json
-> ustc-agentctl evidence verify --dir evidence/demo --format json
```

Required case 的 `Skipped/Unavailable/NotRun` 不计为 Pass。

最小 vertical slice：

1. target profile 的 static/resolved/live-readonly config smoke 与 matrix check 通过；
2. anonymous visitor 打开 `/market` 并浏览 read-only catalog；
3. 未登录安装动作转入统一认证；
4. 登录后安装一个 exact-version PluginPackage；
5. PostgreSQL 保存 installation/enabled/grants，Git 无用户态变化；
6. Agent 发现该 Plugin 的 read-only capability；
7. 用户在 Market 中关闭，再次调用被拒；
8. 用户重新启用，调用恢复；
9. catalog patch 无权限扩张时进入 canary；
10. capability expansion 不自动更新；
11. revoked component 无法建立新 invocation；
12. 清空 Redis（若启用）不丢失任何 durable facts；
13. 从 pinned Git revision 可重建 catalog projection；
14. 对应 `demo` acceptance suite 生成并验证 exact source/binary/config/target evidence。

## 17. Explicit non-goals for first slice

- raw arbitrary script installation；
- in-process untrusted dynamic plugin；
- Market DB 与 Git catalog 双向可写；
- anonymous install/test/deploy；
- default first-party mutation capabilities；
- every MCP 保持 3–5 replicas；
- local admin 作为普通用户登录方式；
- runtime/admin/secrets public exposure；
- 用漂亮 UI 替代 provenance、permissions 和 failure states。

## 18. Remaining unresolved prerequisites

- GitHub organization ownership与 branch protection；
- USTC unified identity application/protocol access；
- 比赛服务器 OS、sudo、container/runtime policy；
- internal DNS/TLS 与校园网 ingress/egress；
- backup target；
- source crawling permissions 与 rate policy；
- Affairs 首个 board/source、authority order 与 parser fixtures；
- team ownership/capacity；
- hosted arbitrary user artifact 是否从比赛 MVP 删除或只做 bounded spike。

这些是 provisioning/team facts，不得由本章臆测为已满足。
