# Project Documentation and Execution Blueprint

## Metadata

- `Status`: **Recommended structure；implementation not authorized**
- `Version`: `0.1.0`
- `Last Review`: `2026-07-21`
- `Applies To`: future USTC Agent implementation repositories
- `Reference Pattern`: `/opt/gitclone/Deve-Notebook/docs/`
- `Authority Defers To`: [`agent-market-architecture.md`](agent-market-architecture.md), [`central-agent-client-relay-marketplace.md`](central-agent-client-relay-marketplace.md)

## 1. Purpose

本章回答两个问题：

1. 未来 USTC Agent/Market implementation repo 如何借鉴 Deve-Notebook 的 docs-as-code 分层；
2. 8C16G/约 200GB 比赛服务器、SSH code-agent workflow 与 107 Slurm 如何分工。

当前 `/opt/data/107-competition/` 是比赛 discovery/canonical context，不是已批准 implementation repo。本章不授权创建 GitHub organization、远程 repo、服务器账号、数据库或部署。

## 2. What to copy from Deve-Notebook

应复制机制，不复制 Notebook 的领域内容和 23 章规模：

```text
00 constitution
-> 01 terminology
-> plan authority/state/runtime contracts
-> feature user journeys
-> acceptance automation
-> registry current implementation mapping
-> task implementation slices
-> report dated evidence
```

关键原则：

- `plan` 是当前工程合同；
- `features` 只描述用户可见行为；
- `acceptance-cases` 只描述可证明条件；
- `registry` 记录 stable IDs 与当前承载路径；
- `tasks` 只安排实施顺序，不覆盖 plan；
- `adr` 记录为什么做重大决策，不作为当前 behavior authority；
- `report` 是带日期证据，不是长期合同；
- code 是 plan/docs 的 projection，不因实现方便反向削弱 authority。

## 3. Recommended future repository docs tree

```text
docs/
  AGENTS.md
  plan/
    AGENTS.md
    00_engineering_constitution.md
    01_terminology.md
    02_positioning.md
    03_identity_roles_and_authority.md
    04_market_catalog_and_plugin_package.md
    05_install_enable_configure_grants.md
    06_agent_and_runtime_topology.md
    07_auth_and_market_web.md
    08_i18n.md
    09_ui_design/
      index.md
      web.md
      desktop.md
      mobile.md
    10_release_and_deployment.md
    11_threat_model.md
    12_reliability_observability.md
    13_resource_budget.md
    14_rust_cli_config_and_acceptance.md
  features/
    market_browse_install.md
    default_system_plugins.md
    ustc_affairs_navigator.md
    ustc_change_radar.md
    campus_opportunity_graph.md
  acceptance-cases/
    00_index.md
    config_smoke.md
    agent_runtime.md
    community_skill.md
    source_graph_evaluation.md
    market.md
    auth.md
    plugin_packages.md
    mcp_runtime.md
    i18n.md
    default_plugins.md
  registry/
    cli-command-registry.md
    config-key-registry.md
    source-registry.md
    graph-schema-registry.md
    evaluation-suite-registry.md
    package-schema-registry.md
    capability-registry.md
    runtime-skeleton-registry.md
    default-plugin-registry.md
    error-code-registry.md
  tasks/
    01_repo_and_contract_skeleton.md
    02_market_vertical_slice.md
    03_first_party_plugins.md
    04_hosted_mcp_spike.md
  adr/
  overview/
  report/
  acceptance-bindings.tsv
  coverage-matrix.md
```

不得一次创建空的全部文件以制造“规划完整”假象。实际 repo 从第一条 vertical slice 所需的最小集合开始，再按 consumer 增长扩展。

## 4. Initial minimum docs set

首个 implementation repo 最少建立：

```text
docs/AGENTS.md
docs/plan/AGENTS.md
docs/plan/00_engineering_constitution.md
docs/plan/01_terminology.md
docs/plan/02_positioning.md
docs/plan/03_identity_roles_and_authority.md
docs/plan/04_market_catalog_and_plugin_package.md
docs/plan/07_auth_and_market_web.md
docs/plan/11_threat_model.md
docs/plan/14_rust_cli_config_and_acceptance.md
docs/features/market_browse_install.md
docs/acceptance-cases/00_index.md
docs/acceptance-cases/config_smoke.md
docs/acceptance-cases/market.md
docs/acceptance-bindings.tsv
docs/registry/cli-command-registry.md
docs/registry/config-key-registry.md
docs/registry/package-schema-registry.md
docs/registry/capability-registry.md
docs/coverage-matrix.md
```

`docs/AGENTS.md` 只规定 reading order、authority precedence、目录语义、report/ADR 非当前行为 authority，以及禁止批量创建空壳文档；`docs/plan/AGENTS.md` 规定 plan layer 的 metadata/anchor/projection 纪律。

`00` 固定工程优先级和骨架审批；`01` 至少定义：

- `CatalogAuthority`；
- `CatalogProjection`；
- `PluginPackage`；
- `Component`；
- `Install`；
- `Configure`；
- `Enable`；
- `CapabilityGrant`；
- `Invoke`；
- `FirstPartySystemPlugin`；
- `VerifiedPatch`；
- `SharedSafe`；
- `WarmEligible`；
- `IdentityProvider`；
- `BreakGlassAdmin`。
- `ConfigurationSmoke`；
- `AcceptanceCase`；
- `EvidenceBinding`；
- `RequiredGate`。

## 5. Plan chapter template

每章建议采用：

```markdown
# NN_title

## Metadata
- Layer
- Status
- Version
- Last Review
- Authority Owns
- Authority Defers To
- Counterpart Feature
- Counterpart Acceptance
- Primary Code Areas

## 1. Scope / Non-goals
## 2. Authoritative entities
## 3. State machines
## 4. Commands / endpoints / outputs
## 5. Invariants and forbidden patterns
## 6. Failure / recovery
## 7. Runtime boundary
## 8. Configuration
## 9. Verification entrypoints
```

不是每章都必须机械填满模板；但 authority、failure、runtime boundary 与 verification 不能只用“大致如此”的散文代替。

重大稳定条款使用 anchor/decision ID。实现模块可在 file header 通过 `plan_ref` 指向这些 stable anchors；不要引用易变的自然语言标题。

## 6. Feature and acceptance projection

### 6.1 Feature document

Feature 回答“用户看见什么”：

```markdown
# Market Browse and Install

## Goal
## User-visible states
## Journey
## Failure/recovery copy
## Non-goals
## Browser acceptance walkthrough
```

首个 journey：

```text
anonymous browse
-> choose Plugin
-> sign in
-> install exact version
-> enabled
-> Agent can discover capability
-> disable
-> invocation denied
-> enable
-> invocation restored
```

### 6.2 Acceptance document

Acceptance 使用 stable case ID：

```yaml
- case_id: MARKET-001
  goal: anonymous visitor can browse but cannot install
  preconditions: [...]
  steps: [...]
  assertions: [...]

- case_id: MARKET-002
  goal: Git catalog is authority over PostgreSQL projection
  preconditions: [...]
  steps: [...]
  assertions: [...]
```

至少覆盖：

- auth/RBAC；
- exact version/digest pin；
- install/disable/enable；
- default first-party bootstrap；
- permission expansion blocks auto-update；
- catalog projection rebuild；
- revoke/rollback；
- locale parity；
- no anonymous execution；
- Redis loss does not lose durable facts；
- ControlledCLI cannot run arbitrary command。

### 6.3 Registries

Registry 是受控 live mapping，不复制 plan prose。

`package-schema-registry`：schema version、current path、validator、status。

`capability-registry`：stable capability ID、risk class、owning domain、allowed component types。

`runtime-skeleton-registry`：runtime name、status、current module path、tracking task、boundary。

`default-plugin-registry`：exact Plugin version、capability manifest hash、bootstrap policy、rollout state。

`config-key-registry`：typed key、owner、default/required、range/choices、secret/reload/redaction class；由 Rust schema 生成或校验。

`cli-command-registry`：public `ustc-agentctl` command tree、mutation/dry-run policy、exit code 与 owning plan；由 clap tree 生成或校验。

## 7. Proposed implementation slices

### Slice 0A — docs contracts and ownership skeleton

Deliver：

- root README / AGENTS；
- minimal docs set；
- decision -> plan -> feature/case/registry mapping；
- complete baseline matrix 与 explicit unresolved manual bindings；
- repo/team owner 与 GitHub authority prerequisites list。

Evidence：docs/link/privacy/authority checks green；没有 code/runtime 完成 claim；每个未分配 owner、未满足 provisioning 或未运行 case 都显式 non-pass。

### Slice 0B — Rust verification kernel

Deliver：

- workspace/module skeleton；
- `crates/config-contract`、`crates/acceptance-contract`、`crates/evidence` 与 `apps/ustc-agentctl` skeleton；
- schema validators；
- typed config loader + `config smoke --level static`；
- acceptance case registry + `matrix-check --strict`；
- CI docs/link/privacy/config/matrix gates。

Evidence：`CFG-001..012` 与 `DOC-001..005/007/008` 有真实 Rust/CLI evidence；不依赖 PostgreSQL、browser 或比赛服务器。

### Slice 0C — Catalog Authority Provisioning Gate（可与 0B 并行）

Deliver：GitHub org/repo ownership、branch protection、required review/status checks、review/publish roles、tag/release immutability、CI token/secrets boundary 与 schema-validation CI 真实运行证据。

Evidence：public catalog authority substrate 可执行而非仅本地约定。0C 未完成前不得进入 Slice 1 catalog implementation；它不扩大 0B 的本地 Rust verification scope。

### Risk Spike A — hosted MCP feasibility（before Slice 1）

在承诺 `UserHostedPrivate` MVP 前，先验证真实比赛服务器/等价隔离环境：

- one approved OCI digest；
- dedicated on-demand start/stop；
- public API 无 Docker/orchestrator admin capability；
- tenant、egress、quota、readiness、idle drain evidence；
- Route A（独立 K3s runtime node）或 Route B（bounded rootless controller）的 go/no-go。

执行入口使用 `ustc-agentctl preflight`、`runtime admission-check` 与 test-namespace `cold-start-smoke/drain-smoke`；不以 ad-hoc shell/docker command 充当最终 evidence。

若 spike 失败，必须在进入 Slice 1/2 前由团队明确把 `UserHostedPrivate` 降为 stretch；不得把 blocker 推迟到最终交付阶段。Spike 仍不自动升级成 production architecture。

### Slice 1 — Market read path

Deliver：

- Git manifest schema；
- deterministic importer；
- PostgreSQL read projection；
- anonymous `/market` browse/detail；
- English/Chinese resources。

Evidence：对应 `CAT-*`、`WEB-001` 与 config/catalog smoke case 产生 machine-readable evidence。

### Slice 2 — Identity and installation

Deliver：

- `IdentityProvider` port；
- dev provider + break-glass admin boundary；
- user/role/install/enable PostgreSQL tables；
- login -> install -> disable -> enable journey。

Evidence：对应 `AUTH-*` case；session/RBAC/CSRF/rate-limit；break-glass not ordinary user fallback。

### Slice 3 — PluginPackage and first-party default

Deliver：

- Plugin/component resolver；
- exact versions/digests；
- capability grants；
- one FirstPartySystemPlugin bootstrap，作为 installation/default-grant mechanism 的 intermediate proof；
- Agent discovery projection。

Evidence：对应 `PKG-*`、`SEC-*` case；tenant scope；disable blocks invocation；permission diff blocks rollout。

### Slice 4A — Tree and typed records

Deliver：stable node IDs、Git Markdown+YAML canonical tree、board policy、typed SourceRevision/ProcedureArtifact/SupersessionEdge、PostgreSQL search projection、URL/history CLI。先人工录入少量 reviewed procedure/source revisions，不接 Agent write。

### Slice 4B — One administrator-maintained board

冻结一个具体 USTC board/source 与 authority order；实现 current/archived artifact、Full/Partial supersession、deterministic procedure renderer 与管理员 publish。Demo 不做 crowd moderation。

### Slice 4C — Official incremental source crawl

Deliver：reviewed Source Registry entry、conditional fetch、immutable raw/normalized snapshots、URL alias/revision index、parser fixture、semantic diff candidate 与 baseline-after-success invariant。Agent 不能 fetch arbitrary URL。

### Slice 4D — Agent typed materialization

Deliver：reviewed Skill 生成 `ProcedureDraft`、Rust schema/cross-field/citation validators、deterministic Markdown、admin diff/approve、atomic Git publish 与 projection refresh。Hook 只修纯格式；semantic missing fail-closed。

### Slice 4E — ChangeRadar semantic ledger and RSS/Atom

Deliver：board-scoped maintainer workers、shared source/change ledger、durable leases/idempotency、approved semantic changes、per-board RSS/Atom。Raw HTML/hash noise、parser failure 与 unreviewed inference 不进入 feed。

### Slice 4F — complete three default first-party Plugins

在 source/revision/diff core 与 Affairs/ChangeRadar 闭环稳定后，补齐 Opportunity Graph 的独立 consent/profile/graph contract 与最小 read-only journey。Core `demo` gate 要求三者 exact bootstrap、可关闭/重启、provenance 与对应 `FP-*` / `EVAL-*` cases 通过；不要求三者同时开发。

完整 Slice 4 authority 见 [`affairs-changeradar-knowledge-architecture.md`](affairs-changeradar-knowledge-architecture.md) 与 [`source-registry.md`](source-registry.md)。

### Slice 5 — controlled mutation

只在 read path 稳定后：

- platform-owned Rust ControlledCLI；
- typed tool gateway；
- idempotency/audit；
- preview/apply；
- path/URL/ownership negative tests。

### Slice 6 — hosted runtime integration

只在 Risk Spike A 已 go 后，把已验证边界接入 product lifecycle：

- one approved OCI MCP；
- dedicated on-demand start/stop；
- no Docker socket in public API；
- tenant/egress/quota tests。

Evidence：private PluginPackage installation、exact digest resolution、cold start、revoke、audit 与 cross-tenant negative tests 形成完整闭环。

所有 slice 都必须在开始时更新 owning plan/feature/case/registry，在结束时运行：

```text
ustc-agentctl config smoke --level static --profile ci --config config.toml --format json --evidence-out evidence/ci/config-static.json
ustc-agentctl acceptance matrix-check --strict --format json --evidence-out evidence/ci/matrix.json
ustc-agentctl acceptance run --required-for pr --mode offline --format json --evidence-out evidence/ci/acceptance.json
ustc-agentctl evidence verify --dir evidence/ci --format json
```

Integration/demo/release 在上述 PR baseline 上追加其 profile 所需的 resolved/live-readonly/real-host/browser/manual gates，不用抽象 smoke-level placeholder 或裸命令代替 exact invocation。

CLI 是 shared Rust contract 的 operator/test projection；server/runtime 不通过 subprocess 调用 CLI 完成正常业务路径。

## 8. 8C16G / 200GB server positioning

比赛服务器推荐定位：

```text
integration + staging + demo runtime
```

它适合：

- reverse proxy；
- central Rust API/worker；
- Market frontend/backend；
- PostgreSQL；
- bounded first-party services；
- targeted integration/browser/deploy smoke。

它不应默认承担：

- 多个高权限 code-agent daemon；
- 每个 Market MCP 的 3–5 常驻 replicas；
- arbitrary user code production hosting；
- unlimited Cargo/container/log caches；
- 与 development checkout 混用的 production state。

## 9. SSH-first development

### 9.1 Recommended flow

```text
Hermes / developer workstation
  -> SSH
     -> dedicated unprivileged dev user
        -> development worktree
        -> local toolchain
        -> optional bounded code-agent CLI

reviewed commit/artifact
  -> deployment path
     -> separate staging/production checkout
```

Primary recommendation：通过 SSH 开发。Server-local code agent 若需要，仅作为 dedicated dev user 下的 CLI/worker，由 SSH 调用；不作为拥有 sudo、production secrets、PostgreSQL admin credential 或 Docker socket 的常驻服务。

### 9.2 Worktree and credential boundary

- development、staging、production checkouts 分开；
- code agent 只访问 assigned repository/worktree；
- secrets 通过 deployment/runtime secret store 提供，不写 repo；
- development Agent 不直连 production database；
- deploy 只消费 reviewed commit/artifact；
- emergency admin action 可审计且可回退。

## 10. Slurm role

107 Slurm 用于：

- Rust full test/clippy/build；
- reproducible release gates；
- source/data preprocessing；
- embeddings/evaluation；
- dependency-heavy compilation。

登录节点只做轻量编辑、Git、提交与日志查看。不得在 login node 做 sustained build/benchmark。

部署服务器负责 real-host integration；Slurm 结果不等价于 reverse proxy、PostgreSQL migration、browser、container/network 或 SSO real-host evidence。

每次 remote gate 必须绑定 exact source revision/archive digest，不以“同名 branch”推定覆盖当前本地工作。

## 11. Disk and build hygiene

约 200GB 仍需预算：

- source/worktrees；
- Cargo registry/target；
- Web/Android toolchains；
- OCI images/layers；
- PostgreSQL data/WAL/backups；
- logs/traces；
- Market artifacts/evidence。

要求：

- Cargo target 与 release artifacts 分开；
- container/image retention；
- logs/evidence retention；
- PostgreSQL backup 在异盘或外部 target；
- build cache 可删除，deployed artifact 可追溯恢复；
- server disk alert 先于部署失败。

真实配额必须在服务器到账后测量，不在本文猜测具体 GB 分配。

## 12. Infrastructure preflight

服务器到账后先只读核验：

- OS/kernel/CPU/RAM/disk/filesystem；
- sudo/root 与 Unix user policy；
- Docker/Podman/K3s/user namespace；
- inbound/outbound/firewall；
- internal DNS/TLS；
- USTC Identity Provider application access；
- SSH and host-key policy；
- backup target；
- whether host is dev/staging/final demo production。

上述 preflight 必须最终由 `ustc-agentctl preflight --target <name> --format json` 生成 real-host evidence；人工 shell 命令只允许作为诊断补充。

在这些事实不明前，不批准 arbitrary hosted code、K3s/KEDA 或 production SSO claim。

## 13. Verification and review loop

未来 implementation slice 遵循：

```text
read 00/01
-> read owning plan
-> update plan if authority changes
-> update feature/acceptance/registry projection
-> implement smallest vertical slice
-> config smoke + matrix check + changed-file quick gate
-> at most three independent reviews
-> fix accepted findings
-> final contract/baseline/browser gates
-> scoped commit
```

UI slice 必须用真实浏览器检查：

- anonymous/auth/admin states；
- en-US/zh-CN；
- desktop/mobile viewports；
- keyboard/focus/accessibility；
- install/disable/enable；
- permission/update diff；
- console/network errors。

完整 current baseline case registry 见 [`platform-acceptance-matrix.md`](platform-acceptance-matrix.md)；Rust CLI/config/evidence contract 见 [`rust-cli-config-smoke-contract.md`](rust-cli-config-smoke-contract.md)。未来 repo 的 `docs/coverage-matrix.md` 与 `docs/acceptance-bindings.tsv` 是其 implementation projection，不另造第二份语义。

## 14. Current non-actions

本轮只形成架构与 execution blueprint，未执行：

- 创建 GitHub organization/repositories；
- 安装 PostgreSQL/Redis；
- 连接或修改比赛服务器；
- 安装 code agent；
- 创建 USTC SSO application；
- 建立 implementation workspace；
- 实现 `ustc-agentctl`、config smoke 或 acceptance runner；
- commit/push/release/deploy。

Slice 4 的顺序已由 Develata 批准；进入实现前仍需冻结首个具体 board/source、目标 repository 与团队 ownership。
