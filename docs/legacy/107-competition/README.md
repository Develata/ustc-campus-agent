# 中国科大“一〇七杯”参赛工作区

本目录是「知行逐光队」参加 2026 中国科大“一〇七杯”算力与智能体开发大赛的 canonical project context。这里只保存可持续更新的规则、来源、决策与方案材料；不保存学号、手机号、邮箱、API key、群聊原始导出或其他敏感信息。

## 赛事与来源

- 赛事：中国科大“一〇七杯”算力与智能体开发大赛
- 官方通知：<https://www.teach.ustc.edu.cn/notice/notice-info/20247.html>
- 官方 FAQ 与算力赛道推荐题目：<https://www.teach.ustc.edu.cn/notice/notice-info/20384.html>
- 本科生算力平台：<https://107.ustc.edu.cn/>
- 在线提交作品：2026-08-01 至 2026-09-06
- 线上初评：2026 年 9 月上旬
- 线下决赛：2026-09-19 至 2026-09-20

截至 2026-07-21 核对的官方规则：

- 同一赛道内每人只能参加一支队伍；每人最多可在两个赛道分别参加一支队伍。
- 同一队伍可参加两个赛道，但必须提交两个不同作品；同一作品不得一稿两投。
- 算力平台赛道作品须提交在本科生算力平台上的运行数据说明。
- 评审维度为创新性、实用性、技术难度与完成度。

## 团队

队名：**知行逐光队**

团队以全本科生队伍身份报名两个赛道：

| 成员 | 当前背景 | 参赛身份 |
|---|---|---|
| Develata | 数学系概率统计方向；研究生尚未正式入学 | 2026 届本科毕业生 |
| 杨同学 | AI 学院；研究生尚未正式入学 | 2026 届本科毕业生 |
| 赵同学 | 计算机学院，大三升大四 | 本科生 |
| 喻同学 | 少年班学院计算机方向，大一升大二 | 本科生 |

## 当前协作边界

- `ezhuman`：群聊交流助手，负责头脑风暴、讨论整理与共识形成，不参与具体实现。
- Deve Hermes：现阶段作为**战略与选题搭档**；负责来源核验、想法归类、候选补充、可行性和评委视角分析。定题后再决定是否进入工程实现。
- 两个赛道不做联动、分头开发不同作品；智能体赛道独立推进。

## 当前成功策略与人力假设

- 采用主次策略：至少一个项目以进决赛、冲击高奖为目标；另一项目优先保证完整、可信、可演示。
- 智能体赛道已由 Develata 冻结“USTC 个人校园 Agent 平台”为主项目候选；待团队 acceptance 后才成为团队定题，不借此替代算力赛道的独立决策。
- 团队计划大量使用 Codex、DeepSeek、Kimi、Claude 等 Agent 完成实现工作；人类主要负责规划、判断、审查与验收。
- 除 Develata 外，其他成员的技术强项、投入时间和主导方向尚待确认；在此之前不得把未知人力计入承诺容量。
- Develata 的默认职责是问题定义、数学/逻辑严谨性、来源与合同审查、验收设计和关键决策，不应成为两条线所有实现任务的单点瓶颈。

## 当前材料

- [`team-acceptance-review-brief.md`](team-acceptance-review-brief.md)：智能体赛道主项目候选的团队 acceptance packet、决策选项、议程与最小 owner/capacity 表。
- [`architecture-summary.md`](architecture-summary.md)：当前全局 architecture projection、implementation sequence、non-actions 与 blocking facts；详细语义仍由 owning contracts 决定。
- [`ideas-initial.md`](ideas-initial.md)：首轮群聊想法的结构化整理。
- [`brainstorm-2026-07-21.md`](brainstorm-2026-07-21.md)：第二轮自由发散；新增算力、智能体与反常规组合方向，尚未评分。
- [`agent-track-concept.md`](agent-track-concept.md)：智能体赛道优先讨论 brief；USTC 个人校园 Agent、G1–G3 公共内核与 plugin 开放问题。
- [`agent-track-architecture-options.md`](agent-track-architecture-options.md)：`PluginPackage`/execution classes、Rust/Dioxus 目标端分析与候选骨架；尚未批准实现。
- [`deployment-topology-analysis.md`](deployment-topology-analysis.md)：中央多租户、自托管、中央 authority + 多执行面及双平台并存方案比较。
- [`central-agent-client-relay-marketplace.md`](central-agent-client-relay-marketplace.md)：中央 Agent authority、future AI/MCP relay、SSO/browser、services 页面、marketplace 审核与完整自部署边界。
- [`model-provider-policy.md`](model-provider-policy.md)：MVP 的 `OfficialCentral` / `UserCloud` provider、用户 secret、SSRF 与 future relay extension policy。
- [`mcp-binding-policy.md`](mcp-binding-policy.md)：`Official/VerifiedMarket | UserRemote | UserHostedPrivate` 的 binding、tool grants、secret、SSRF 与协议边界。
- [`mcp-market-hosting-policy.md`](mcp-market-hosting-policy.md)：公共/私有 MCP 的 catalog trust、shared/dedicated tenancy、cold/warm availability、artifact admission、sandbox 与弹性运行边界。
- [`agent-runtime-adoption-policy.md`](agent-runtime-adoption-policy.md)：原创 Rust core、Rig/rmcp 窄依赖、成熟框架参考、MIT/Apache attribution 与选型 spike。
- [`agent-market-architecture.md`](agent-market-architecture.md)：GitHub public catalog authority、PostgreSQL operational authority、`PluginPackage` 本体、Market Web、默认 first-party plugins、双语与升级策略。
- [`source-registry.md`](source-registry.md)：USTC official source identity、revision、URL lookup、fetch/SSRF、immutable snapshot 与 baseline advancement contract。
- [`affairs-changeradar-knowledge-architecture.md`](affairs-changeradar-knowledge-architecture.md)：树形 Wiki、typed procedure、direct supersession、Agent materialization、board maintainer 与 RSS/Atom owning contract。
- [`project-documentation-and-execution-blueprint.md`](project-documentation-and-execution-blueprint.md)：参考 Deve-Notebook 的 plan/feature/acceptance/registry 分层，以及 8C16G、SSH 与 107 Slurm 的执行边界。
- [`rust-cli-config-smoke-contract.md`](rust-cli-config-smoke-contract.md)：Rust CLI-first operator/verification surface、typed config authority、三层 configuration smoke、evidence schema 与 exit semantics。
- [`platform-acceptance-matrix.md`](platform-acceptance-matrix.md)：覆盖 docs/config/catalog/package/auth/Skill/MCP/Agent runtime/Web/Campus Trust Kernel/first-party/deployment 的完整 baseline case/evidence matrix。
- [`acceptance-bindings.tsv`](acceptance-bindings.tsv)：manual acceptance cases 的 owner/evidence/status projection；当前 unassigned/not-run 是显式 release blocker，不是 Pass。

## 记录纪律

1. 使用【已知】【团队观点】【推测】【待确认】【建议】区分证据状态。
2. 新讨论不得自动覆盖旧决策；如发生变化，记录日期与理由。
3. 不把模型生成内容描述为团队共识。
4. 不把演示性原型夸大为平台级生产能力。
5. 两个赛道分别建立价值主张、验收指标和交付物，除非团队之后明确决定联动。
