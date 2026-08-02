# 03 — Agent and First-party Journeys

> **Illustrative / No live backend** — 本卷全部线框与示例数据为设计样例。
> Packet: `m80-default-v0` · Status: `Proposal` · Source: `2f4de29032560ff3e13d9994b33a3aff14243f44` / tree `53e266c47fdb07d50a734faa24bb11ac4bc5527d`
> 覆盖 artifacts：8 Agent thread/run · 9 First-party Plugin entry patterns

## 1. 本卷 authority 前提

- `TRACKED FACT`：M30 现状为 node-local AgentRun kernel（`RunSpec` agent-runtime/src/lib.rs:38、`AgentRun` :379、`RunEvent` :361、`RunEventKind` :312）；finite HarnessRun 与用户面向 carrier planned（`docs/plan/modules/00-module-map.md:22`）。
- `TRACKED FACT`：HarnessRun phase machine 与 evidence/review 语义见 `docs/contracts/agent-harness.md:34-87`；M40 仅有 protocol values + fake gateway/executor proof（`agent-tool-protocol` `AgentToolsetView` :176）。
- `PROPOSAL`：Agent 不是 endless chat；是“提交有限任务 → 回答阻塞问题 → 看 accepted plan 与结果”。UI 展示 phase 与 evidence，**不展示 chain-of-thought**；stream disconnect ≠ cancel/complete。
- `PROPOSAL`：tool output/exit code 不等于 receipt/success；receipt 是 durable authority 的投影（`docs/contracts/agent-harness.md` evidence 语义）。

## 2. Agent thread / task（Artifact 8a，PROPOSAL，mid-high fidelity）

### 2.1 Goal

提交有限任务、回答 blocking question、看 accepted plan 与结果。

### 2.2 信息层级

task composer → conversation/task boundary → phase bar → plan/evidence panel → messages/results → input（仅当 server 投影允许输入时）。

### 2.3 线框（desktop）

```text
┌────────────────────────────────────────────────────────────────┐
│ 任务：整理本周教务通知                       [取消任务]（如可用）│
│ ──────────────────────────────────────────────────────────────│
│ 阶段：理解 → 等待你的回答 → 规划 → 执行 → 验证 → 完成          │
│            ▲（当前，server-projected）                          │
│ ──────────────────────────────────────────────────────────────│
│ ┌ 对话/任务区 ────────────────────┐ ┌ 计划与证据（可折叠）────┐ │
│ │ Agent：你需要只看本科生通知，    │ │ 已接受计划               │ │
│ │ 还是也包括研究生通知？           │ │  1. 读取教务信息源       │ │
│ │                                │ │  2. 筛选本周条目         │ │
│ │ [只看本科生] [都包括]           │ │  3. 汇总并给出处         │ │
│ │  ← 回答阻塞问题，typed schema  │ │ 证据                     │ │
│ │                                │ │  · 来源：教务处公告      │ │
│ │                                │ │    revision obs. 时间    │ │
│ │                                │ │  · 操作请求/执行凭证 →   │ │
│ └────────────────────────────────┘ └──────────────────────────┘ │
│ 输入框仅在 server 投影允许时出现；否则显示当前等待对象           │
└────────────────────────────────────────────────────────────────┘
```

- Phase bar 只显示 user-visible phase（server-projected），不显示内部 state machine 名。
- Follow-up 创建 allowed typed intent；不默认 mutate 已 terminal 的 run（scope expansion → typed new-run requirement）。
- Plan/evidence panel 使用 evidence spine（来源→revision→决定→凭证），与 first-party detail 同一语言。

### 2.4 状态

understanding、awaiting user、planning、running、verifying、needs decision、terminal outcomes（completed/partial/failed/expired/cancelled，server-projected）、offline/stream gap（banner「正在从事件游标恢复」+ 保留最后确认事件）。

### 2.5 Responsive

Desktop content + collapsible plan inspector；Android thread + plan bottom sheet/tab。

### 2.6 Semantic needs（`PROPOSED_SEMANTIC_INTENT`）

`CreateAgentRun` / `GetAgentRun` / `SubmitSteering`（或 allowed follow-up）/ `CancelAgentRun` / `WatchAgentRunEvents`。M80 never calculate：accepted run from submit、legal resume、phase from local timer、kill=cancel。

## 3. Agent run detail（Artifact 8b，PROPOSAL，mid-high fidelity）

### 3.1 Goal

回答“发生了什么、为何允许/拒绝、证据在哪里、怎样恢复”。

### 3.2 线框

```text
┌────────────────────────────────────────────────────────────────┐
│ 任务运行详情：整理本周教务通知                                  │
│ 结果：部分完成（terminal · server-projected）                   │
│ 原因：1 个操作被拒绝（权限不足）· 2 个操作已完成                │
│ ──────────────────────────────────────────────────────────────│
│ 阶段时间线                                                     │
│  ✓ 理解  ✓ 规划  ✓ 执行（2/3）✓ 验证  ● 终止：部分完成        │
│ ──────────────────────────────────────────────────────────────│
│ 操作与凭证                                                     │
│  ┌──────────────────────────────────────────────────────────┐ │
│  │ ✓ 读取教务处公告      凭证 rcpt-… · 已完成   [查看凭证]   │ │
│  │ ✓ 筛选本周条目        凭证 rcpt-… · 已完成                │ │
│  │ ✗ 读取研究生管理系统  被拒绝 · 原因：未授权该信息源        │ │
│  │   （recovery owner：权限复核 → 插件管理）                  │ │
│  └──────────────────────────────────────────────────────────┘ │
│ 证据与审查 · 恢复                                              │
│  · 来源 revision/observed 时间 · 不确定性说明                  │
│  · [导出安全摘要]（secondary）                                 │
└────────────────────────────────────────────────────────────────┘
```

- 每个 tool intent 一行：action summary、status、time、receipt ref、safe error；denied 显示 stable reason + recovery owner。
- Cancel 仅当 server offers；retry/reconcile 仅当 owning contract admits（`UNRESOLVED`：RetryRunOperation/ReconcileRunOperation 语义未定）。
- 状态族：partial stream reconnect（「Reconnecting from event cursor」，保留最后确认事件，不重复事件、不宣告终止）、cursor refresh、artifact unavailable、receipt pending、in-flight cancel reconciliation、blocked/partial/failed/expired/cancelled。

### 3.3 Responsive

Desktop timeline + inspector；Android chronological feed，technical event groups collapsed。

### 3.4 Semantic needs（`PROPOSED_SEMANTIC_INTENT`）

`GetAgentRun` / `WatchAgentRunEvents` / `ListRunArtifacts` / `GetToolIntentDetail` / `GetReceiptDetail`。`EXISTING_DOMAIN_TYPE` 参考：`RunEvent`/`RunEventKind`、`AgentToolsetView`。M80 never calculate：completion/evidence/authorization/success from tool output。

## 4. First-party 共享模式：Evidence spine（Artifact 9a，PROPOSAL）

三个 first-party detail 共用同一条安静的关系表达：

```text
来源（source identity/authority）
  → 版本（revision · observed/effective 时间）
  → 决定（review/publication decision · 谁批准/不确定性）
  → 凭证（receipt/artifact 引用 · disclosure）
```

- 每一环可展开 disclosure；candidate/under-review 与 reviewed current 严格区分；URL ≠ revision。
- 无炫光背景、无 mascot；signature 是排版与 hairline 结构（见 `05` 卷）。

## 5. ChangeRadar（Artifact 9b，PROPOSAL，high-fidelity 代表 detail）

### 5.1 Goal

看 approved semantic change，而非 crawl noise。回答“什么变了，是否影响我？”。

### 5.2 线框

```text
┌────────────────────────────────────────────────────────────────┐
│ 校园变化：本科生奖学金评审通知（illustrative）                  │
│ 已发布 · 生效时间 2026-08-10 · 适用范围：本科生                 │
│ ──────────────────────────────────────────────────────────────│
│ 变化内容（before → after）                                     │
│  申请截止：8 月 20 日 → 8 月 15 日                             │
│  材料要求：新增「成绩单加盖公章」                              │
│ ──────────────────────────────────────────────────────────────│
│ 证据脊柱                                                       │
│  来源：教务处公告板 → 版本 rev-…（observed 2026-08-01 09:12）  │
│  → 审核：已批准（reviewer 引用 · disclosure）→ 发布凭证 …      │
│ ──────────────────────────────────────────────────────────────│
│ [查看当前规则/来源]（primary，仅 admitted link）                │
│ [就此询问 Agent]（创建新 typed run intent，携带稳定实体引用）   │
└────────────────────────────────────────────────────────────────┘
```

- 状态：no changes（honest empty）、candidate under review（仅合适角色可见并明确标注）、approved/published/rejected、source stale/suspended、conflict。
- 无 consent/profile 语义时**不做 personalized impact claim**；「是否影响我」只显示 server-projected scope。
- Semantic needs：`ListChangeRadarItems` / `GetChangeRadarItem` / `GetChangeReviewExplanation`（`PROPOSED_SEMANTIC_INTENT`，owner M70；M70 design-only、M60 planned——全部 product data provisional）。M80 never calculate：semantic/noise decision、personal impact、approval/publication。

## 6. Affairs Navigator（Artifact 9c，PROPOSAL，high-fidelity 代表 detail）

### 6.1 Goal

按 reviewed procedure 完成校园事务。回答“我现在该怎么办？”。

### 6.2 线框

```text
┌────────────────────────────────────────────────────────────────┐
│ 办事指南：本科生国家助学金申请（illustrative）                  │
│ 当前已审核版本 · 最后验证 2026-07-28 · 学期：2026 秋            │
│ ──────────────────────────────────────────────────────────────│
│ 适用条件                                                       │
│  · 全日制在校本科生 · 已入库家庭经济困难认定                   │
│ 前置条件                                                       │
│  · 完成困难认定（见关联指南 →）                                │
│ 步骤（保持顺序）                                               │
│  1. 登录学工系统填写申请表           [打开官方入口]（admitted） │
│  2. 提交成绩单（加盖公章）                                      │
│  3. 学院审核 → 学校公示                                        │
│ 期限：申请 8 月 15 日截止（effective time）                    │
│ ──────────────────────────────────────────────────────────────│
│ 证据脊柱：来源 → 版本 → 审核 → 凭证                            │
│ 不确定性：助学金名额以学院通知为准（server-projected）          │
└────────────────────────────────────────────────────────────────┘
```

- 缺失步骤**不补写**（不让 LLM 生成步骤）；candidate ≠ current；冲突/无法核实时显示 uncertainty 而不是编造答案。
- 步骤的 copy/check 是 local-only presentation；「打开官方入口」仅 admitted link（ExternalNavigation / Android Custom Tab）。
- 状态：current reviewed、stale/refresh pending、candidate review、conflicting/cannot verify、archived、empty search。
- Semantic needs：`GetAffairsProcedureTree` / `SearchAffairsProcedures` / `GetAffairsProcedureDetail`（`PROPOSED_SEMANTIC_INTENT`，owner M71，design-only）。M80 never calculate：path as identity、missing steps、candidate=current。

## 7. Opportunity Graph / Course Planning（Artifact 9d，PROPOSAL，high-fidelity 代表 detail）

### 7.1 Goal

基于 reviewed facts + explicit consented profile 看 qualification/plan；清楚区分 hard facts 与 soft signals。回答“什么适合我，下一步选什么？”。

### 7.2 线框

```text
┌────────────────────────────────────────────────────────────────┐
│ 机会图谱                                                       │
│ ──────────────────────────────────────────────────────────────│
│ 我的资料与授权                                                 │
│  · 已授权用途：课程匹配（范围/版本 · [管理授权]）               │
│  · 资料：3 项事实（[查看/编辑] · [删除我的资料]←destructive）   │
│ ──────────────────────────────────────────────────────────────│
│ 候选机会（reviewed public facts + 已授权资料）                  │
│  · 2026 秋季交换项目   资格：满足硬性条件 ✓ / 冲突：无          │
│    依据：培养方案要求（hard fact）· 你的已修学分（授权资料）    │
│    证据：来源 rev-… · 不确定性：名额以学院为准                  │
│ ──────────────────────────────────────────────────────────────│
│ 课程规划（bounded spike 能力 · illustrative）                   │
│  [生成候选方案]（仅在 admitted facts/profile 齐备时可用）       │
│  官方课程事实优先；iCourse 评价仅外链查看（link-out-only）      │
└────────────────────────────────────────────────────────────────┘
```

- `TRACKED FACT`：Course Planning 是 Opportunity Graph 内 bounded offline spike（`README.md:19`、`docs/plan/06-first-party-plugins.md:140-159`）；COURSE-001/002/003 为 implemented bounded evidence，但不代表 live integration。
- `TRACKED FACT`：iCourse review link-out-only（`README.md:103`）。
- 状态：consent missing（引导授权，不预填）、profile unavailable、facts stale/conflicting、unresolved alias（如实显示「无法确定课程对应关系」而非猜测——对应 COURSE-002 语义）、no valid candidate、candidate stale after revision、delete pending/not complete。
- Private facts 不进入 public graph；UI 在 consent summary 中显式说明范围。
- Semantic needs：`ListOpportunityCandidates` / `GetProfileFacts` / `SaveProfileFact` / `DeleteProfileFact` / `GetConsent` / `UpdateConsent` / `ExplainOpportunityCandidate` / `GenerateCoursePlanCandidates` / `GetCourseFact` / `OpenReviewLinkout`（均 `PROPOSED_SEMANTIC_INTENT`，owner M72 + ExternalNavigation）。M80 never calculate：match/eligibility、consent validity、hard constraints/ranking、URL safety。

## 8. First-party → Agent 衔接（PROPOSAL）

任一 first-party detail 的「就此询问 Agent」：**创建新 typed run intent，携带稳定实体引用**（change item ID / procedure ID / candidate ID），不把 UI 文本拼成 untyped command。新 run 独立于原 surface 的 lifecycle。

## 9. 本卷 UNRESOLVED 汇总

- HarnessRun 用户面向 phase vocabulary 与 event schema（owner M30/M10，Q4/Q5）。
- Retry/reconcile 操作语义（owner 各 operation）。
- First-party 各 surface 的真实 read model 字段（owner M70/M71/M72，design-only/planned）。
