# USTC Agent Platform Architecture Summary

- 状态：**Develata 已冻结为智能体赛道主项目候选；团队 acceptance pending，implementation 未开始**
- 更新时间：2026-07-21
- 用途：当前 architecture projection；详细语义以链接的 owning contracts 为准

## 1. Product

构建面向 USTC 学生的 personal campus Agent platform：中央 Agent authority、Campus Trust Kernel、可安装的 `PluginPackage` Market、Web/Android clients 与完整 self-host profile。

三个默认 first-party products：

```text
Affairs Navigator：我现在该怎么办？
ChangeRadar：什么变了，是否影响我？
Opportunity Graph：什么适合我，下一步选什么？
```

工程顺序：

```text
source/revision/diff core（ChangeRadar foundation）
-> Affairs Navigator structured procedure entry
-> ChangeRadar per-board feed
-> Opportunity Graph + consent-aware profile
```

Opportunity Graph 的 memory/profile contract 尚未设计。

## 2. Authority split

```text
Git
  public catalog/package/schema
  reviewed knowledge tree/policy/procedure/source declarations

PostgreSQL
  users/install/grants/runtime/audit
  crawl/candidate/search/feed operational state

Object/OCI storage
  exact executable artifacts
  immutable source snapshots/diffs

Redis（optional）
  no durable authority
```

冲突时以 owning Git contracts 和 reviewed manifests 为 canonical；PostgreSQL projection 可重建，不能反向 publish Git truth。

## 3. Agent/runtime

- 原创、窄而可验证的 Rust domain/control core；
- Rig 与 rmcp 只作为可替换 adapters，采用前须 disposable spike；
- session/run/checkpoint/grant/policy 由平台拥有；
- CLI/HTTP/worker 共用 Rust domain core，server 不 subprocess 调 CLI；
- hooks 分 `Observer / Transformer / Gate`，registry 不伪装成 hook；
- transformer 后重新 schema validation + authorization；
- security/publish gate fail closed；
- arbitrary in-process dynamic code 不进入 central authority process。

Pi 只作为 package/resource/hook/session architecture reference，不作为 runtime dependency；其 full-process extension permission、任意 TypeScript hot-load、修改 tool input 后不 revalidate 等语义不得照搬。

## 4. PluginPackage and Market

`PluginPackage` 是唯一一等安装/启停/升级单位：

```text
PluginPackage
├── McpServerComponent*
├── SkillComponent*
├── ControlledCliComponent*
├── SharedServiceBinding*
└── DeclarativeResourcePack*   # experimental package resource, no execution authority
```

- public Git catalog，PostgreSQL user installation authority；
- exact package/component/version/digest/grant resolution；
- default first-party Plugins 只声明 `AutoGrantEligible` exact read capabilities；
- user mutation 走 typed Rust ControlledCLI/gateway；
- Market frontend 独立 entry，但共享 central Auth/PostgreSQL/backend；
- public catalog/schema/source repos 在 GitHub；runtime/admin/secrets 仅内网。

## 5. Affairs Navigator / ChangeRadar knowledge contract

Lookup ladder：

```text
L0 exact ID/path/URL lookup
L1 tree + PostgreSQL structured search
L2 approved-source targeted refresh + typed materialization
L3 bounded RAG over approved snapshots（later）
```

Canonical path：

```text
approved official source revision
-> typed ProcedureDraft
-> Rust validators
-> deterministic Markdown
-> admin approval
-> atomic Git publish
```

- URL 不是 revision identity；同 URL 可有多个 revisions；
- supersession 保存 direct `Full/Partial/Clarification/Duplicate` edges；
- archived evidence/artifacts 不删除、不复制 transitive `old-web`；
- Agent/board maintainer 只能提交 candidate；
- ChangeRadar 与 Affairs 共用 source/change ledger；
- RSS/Atom 只发布 approved semantic changes，订阅 stable node ID；
- full-corpus RAG 不进入前几版主路径。

Owning contracts：

- [`source-registry.md`](source-registry.md)
- [`affairs-changeradar-knowledge-architecture.md`](affairs-changeradar-knowledge-architecture.md)

## 6. Provider, MCP and hosting

- Demo AI：`OfficialCentral` + encrypted `UserCloud` OpenAI-compatible profile；
- MCP：`Official/VerifiedMarket | UserRemote | UserHostedPrivate`；
- `UserHostedPrivate` 仅在 Risk Spike A 明确 GO 后进入 `demo-hosted`；core `demo` 不依赖；
- public API 不持有 Docker/Kubernetes admin capability；
- hosted execution 使用 exact approved artifact、tenant/egress/quota/readiness/drain controls；
- future user-device relay 只保留 typed extension point。

## 7. Client and deployment

- central Rust server/worker + PostgreSQL；
- Web + Android first；Dioxus 0.7 只做 disposable spike，不能先成为 public protocol；
- public client API 推荐 versioned HTTP JSON + SSE；MCP 使用 Streamable HTTP；
- Linux/Docker Compose 是 initial self-host profile；
- self-host 使用同一 server binary/authority contracts，不做 federation 或 central↔self-host sync；
- 8C16G 比赛服务器用于 integration/staging/demo；重 Rust build/test/clippy 使用 107 Slurm。

## 8. Implementation sequence

```text
Slice 0A docs/contracts
Slice 0B Rust verification kernel
Slice 0C GitHub catalog authority provisioning
Risk Spike A hosted MCP feasibility
Slice 1 Market read path
Slice 2 Identity/install
Slice 3 PluginPackage/default bootstrap
Slice 4A tree + typed source/procedure records
Slice 4B one administrator-maintained board
Slice 4C incremental official-source crawl
Slice 4D Agent typed materialization
Slice 4E ChangeRadar semantic ledger + RSS/Atom
Slice 4F remaining default first-party products
Slice 5 controlled mutation
Slice 6 hosted runtime integration（only after GO）
```

## 9. Current non-actions

尚未：

- 创建 implementation repository 或 GitHub organization；
- 提交/push 当前工作区；
- 实现 Rust/DB/Web/Android；
- 获得 USTC IdP application；
- 连接比赛服务器；
- 批准具体 source crawl；
- 运行任何 acceptance evidence；
- 确认团队 owner/capacity。

## 10. Blocking facts before implementation

1. 团队接受该智能体赛道主项目候选，并承诺最小 owner/capacity；
2. implementation repo 与 owner；
3. Affairs 首个具体 board/source、authority order、crawl permission/rate；
4. GitHub organization/review/publish roles；
5. USTC IdP protocol/application access；
6. 比赛服务器 OS/container/network/DNS/TLS/backup facts；
7. 比赛开源复用口径；
8. manual acceptance case owners/evidence plan。

## 11. Primary contracts

- [`agent-track-concept.md`](agent-track-concept.md)
- [`agent-market-architecture.md`](agent-market-architecture.md)
- [`agent-runtime-adoption-policy.md`](agent-runtime-adoption-policy.md)
- [`affairs-changeradar-knowledge-architecture.md`](affairs-changeradar-knowledge-architecture.md)
- [`source-registry.md`](source-registry.md)
- [`mcp-binding-policy.md`](mcp-binding-policy.md)
- [`mcp-market-hosting-policy.md`](mcp-market-hosting-policy.md)
- [`model-provider-policy.md`](model-provider-policy.md)
- [`rust-cli-config-smoke-contract.md`](rust-cli-config-smoke-contract.md)
- [`platform-acceptance-matrix.md`](platform-acceptance-matrix.md)
- [`project-documentation-and-execution-blueprint.md`](project-documentation-and-execution-blueprint.md)
