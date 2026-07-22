# USTC 个人校园 Agent 平台：团队 Acceptance Review Brief

## Metadata

- `Decision target`: 是否接受“USTC 个人校园 Agent 平台”为智能体赛道主项目
- `Current status`: **Develata accepted；team decision pending**
- `Prepared`: 2026-07-21
- `Scope`: 智能体赛道独立项目；不与算力赛道联动或复用同一作品
- `Decision authority`: 团队共同确认；本文不预填团队共识

## 1. 一句话命题

构建面向 USTC 学生的 personal campus Agent platform：用 reviewed official sources、typed procedures、semantic change ledger 和 installable `PluginPackage`，回答“现在怎么办、什么变了、什么适合我”，并以可审计的 Rust contracts、权限和 provenance 支撑扩展。

## 2. 本次会议只决定什么

团队只需回答：

> 是否把该候选升级为智能体赛道正式项目，并投入最小可交付团队容量？

本次不要求冻结每个数据库表、前端组件或部署参数，也不要求承诺尚未核验的 SSO/server/source facts。

## 3. 为什么现在可以进入团队决策

当前已存在：

- 产品命题与 G1–G3 first-party projections；
- central Agent authority + Market + clients 的 architecture；
- `PluginPackage`、MCP binding、model provider 与 hosted runtime policies；
- Affairs Navigator / ChangeRadar knowledge contracts；
- Source Registry、source revision、supersession 与 feed semantics；
- Rust CLI-first configuration smoke contract；
- 223-case acceptance matrix 与 manual evidence bindings；
- 两条独立 blocker review 及修正结果。

当前不是“只有想法”，但也没有声称 implementation 已经开始或可运行。

## 4. 已冻结的候选边界

### Product

- 智能体赛道独立于算力赛道；不同作品，不做双赛道联动；
- 三个默认 first-party Plugins：Affairs Navigator、ChangeRadar、Opportunity Graph；
- 工程顺序：source/revision/diff core → Affairs → ChangeRadar feed → Opportunity Graph；
- 前几版不以 full-corpus RAG 作为 truth path。

### Authority and safety

- Git：reviewed catalog、schema、knowledge tree/policy/procedure/source declarations；
- PostgreSQL：user/install/grant/runtime/audit 与 operational projections；
- object/OCI storage：immutable evidence 与 exact artifacts；
- Agent 只能产生 candidate；Demo canonical knowledge publish 由管理员完成；
- public API 不持有 Docker/Kubernetes admin capability；
- untrusted in-process dynamic code 不进入 central authority process。

### Engineering

- Rust domain/control core；CLI/HTTP/worker 共用 authoritative crates；
- production server 不 subprocess 调 CLI；
- Web + Android first；public client API 不绑定 UI framework；
- Dioxus、Rig、rmcp 通过 disposable spikes 后才采用；
- configuration smoke、acceptance matrix 与 evidence 均是 implementation contract。

## 5. 尚未冻结，但不阻止本次产品 acceptance

- implementation repository/name；
- first Affairs board/source；
- USTC IdP application/protocol；
- server/DNS/TLS/backup facts；
- source crawl permission/rate；
- Dioxus/Rig spike result；
- `UserHostedPrivate` Risk Spike A result；
- Opportunity Graph memory/profile contract。

这些事项分别进入 provisioning、source reconnaissance 或 disposable spike；不得在会议中被臆测为已满足。

## 6. 团队决策选项

### A. Accept

接受为智能体赛道正式项目，并承诺：

- 一个 product/source owner；
- 一个 backend/runtime/security owner；
- 一个 client/demo owner；
- release/evidence owner 可以由上述角色兼任，但必须显式指定；
- 允许创建 implementation repository，并启动 source reconnaissance 与 Slice 0B。

### B. Conditional Accept

原则接受，但必须记录：

- 具体缺口；
- owner；
- deadline；
- 可判定的 exit criterion。

没有 owner/deadline/evidence 的“有条件接受”等同于未决定。

### C. Reject

拒绝时记录主要原因：用户价值、原创性、周期、人员、基础设施、合规或其他；保留 architecture docs，但不创建 implementation commitment。

## 7. 每位成员会前最小输入

每位成员只需准备：

```text
Decision: Accept | ConditionalAccept | Reject
Weekly capacity: <hours/week or explicit unknown>
Preferred ownership: Product/Source | Backend/Runtime/Security | Client/Demo | Release/Evidence
Top blocker: <one sentence>
Condition/evidence needed: <one sentence or none>
```

不要在此文件写学号、手机号、邮箱、credential 或其他敏感信息。

## 8. 推荐 30-minute agenda

1. `0–5 min`：读取一句话命题、用户 journey 与 explicit non-goals；
2. `5–12 min`：检查可演示价值、差异化与评委视角；
3. `12–20 min`：检查 capacity、ownership、source/SSO/server blockers；
4. `20–25 min`：每人独立给出 A/B/C；
5. `25–30 min`：记录结论、conditions、owners 与下一 gate。

避免在本次会议陷入 framework、表字段或 UI 微观争论；这些由 owning contracts 和 spikes 处理。

## 9. Acceptance gate

只有满足以下条件，状态才能从 `team decision pending` 变为 `team accepted`：

- A，或具有 owner/deadline/evidence 的 B；
- 至少覆盖 Product/Source、Backend/Runtime/Security、Client/Demo 三类 ownership；
- capacity 不把未知人力计入 committed plan；
- 团队接受 source reconnaissance 作为首个 product fact-finding task；
- 团队接受 Rust verification kernel 是 implementation 的前置 Slice；
- 不把 SSO/server/crawl permission 当作未经核验的既成事实。

## 10. Decision record template

```yaml
decision: pending # accept | conditional_accept | reject
decided_at: null
scope: agent-track
authority: team
conditions: []
ownership:
  product_source: null
  backend_runtime_security: null
  client_demo: null
  release_evidence: null
capacity:
  status: unknown
next_gate: null
notes: []
```

最终 decision record 应由管理员写入 reviewed project context；聊天结论不能静默覆盖此状态。

## 11. Review packet

按以下顺序阅读即可：

1. [`architecture-summary.md`](architecture-summary.md)
2. [`agent-track-concept.md`](agent-track-concept.md)
3. [`affairs-changeradar-knowledge-architecture.md`](affairs-changeradar-knowledge-architecture.md)
4. [`source-registry.md`](source-registry.md)
5. [`platform-acceptance-matrix.md`](platform-acceptance-matrix.md)

完整 implementation 切片见 [`project-documentation-and-execution-blueprint.md`](project-documentation-and-execution-blueprint.md)。
