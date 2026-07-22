# USTC Campus Agent Platform：完整初始规划 v2

> **用途：** 知行逐光队“一〇七杯”智能体赛道的产品、架构与执行基线
> **状态：** Develata 已确认本项目作为智能体赛道方向；本文是优化建议，团队 owner/capacity 与具体实现仍待闭合
> **Evidence cut-off：** 2026-07-22
> **提交截止：** 2026-09-06；自本文日期起剩余 46 天
> **主线：** `Plugins Market-first + flagship course-selection plugin + owned Rust authority core`

---

## 0. 结论先行

建议把原来的“校园 Agent + 三个并列官方插件”进一步收敛为一个更容易讲清、也更能形成完整 Demo 的主叙事：

> **做一个以 Plugins Market 为产品骨架的 USTC 个人校园 Agent；首个旗舰插件是 Campus Opportunity Graph，首个可交付 vertical slice 是 Course Planning（产品展示名可用 Course Compass）。它把官方培养方案、当前课程信息与 USTC 评课社区的主观评价表示为带资格、依赖、时间窗口与证据的机会图，从而给出可追溯、可解释、可约束的选课辅助。**

完整首版只需要证明四件事：

1. **Market 真实成立：** Plugin 可浏览、安装、授权、启停、升级、撤销；不是写死的功能菜单。
2. **旗舰 Plugin 真有价值：** 能从培养要求、课程供给、时间冲突、先修关系与用户偏好生成可解释的候选方案。
3. **Agent 可控：** LLM 负责理解意图与解释；硬约束、来源优先级、授权、状态与审计由 Rust core 决定。
4. **系统可扩展：** `ustc.opportunity-graph` 可沿同一 typed graph contract 增加科研、竞赛、讲座等 domain packs；同一 `PluginPackage` 合同也可继续承载 Affairs Navigator、ChangeRadar 与第三方 MCP/Skill，而无需改写中央 Agent。

因此，初始优先级调整为：

```text
P0  Plugins Market + Package/Grant lifecycle
P0  Campus Opportunity Graph Plugin + Course Planning flagship vertical slice
P1  Campus Trust Kernel 中该插件真正需要的 source/revision/provenance 子集
P1  bounded conversational Agent
Future（competition MVP 之外）  ChangeRadar / Affairs Navigator / Opportunity Graph 的科研、竞赛等 domain packs
Future（competition MVP 之外）  Android 完整体验与第三方 hosted code
```

这不是删掉原有三项产品，而是让它们从“首版同时交付”降为沿同一骨架自然扩展的后续 Plugin。**首版应以一个强闭环胜过三个半成品。**

---

## 1. 产品定位与评委叙事

### 1.1 一句话定位

> 一个面向 USTC 学生的可扩展校园 Agent 平台：用户从 Plugins Market 安装可信能力，Agent 在来源、权限和审计约束下完成校园任务；Campus Opportunity Graph 是首个旗舰 Plugin，Course Planning 是它的首个完整 journey。

### 1.2 它不是什么

- 不是“我的科大”菜单复制；
- 不是把评课社区网页丢入 RAG 后自由总结；
- 不是 LangGraph/Goose/Rig/Pi 的校园换皮；
- 不是自动登录并代替学生完成选课；
- 不是首版就托管任意第三方代码的 Agent App Store；
- 不是以多 Agent 角色扮演制造技术复杂度。

### 1.3 竞赛价值映射

| 评审维度 | 可展示证据 |
|---|---|
| 创新性 | `PluginPackage`、typed Opportunity Graph、校园信源治理、能力授权与 Course Planning 的统一闭环 |
| 实用性 | “我下学期该怎么选课”这一高频、真实、可现场体验的任务 |
| 技术难度 | 多源实体对齐、约束规划、时态版本、权限/启停、可恢复 Agent run |
| 完成度 | Market 安装 → Agent 调用 → 方案解释 → 禁用阻断 → 来源更新的全链路 Demo |

### 1.4 核心 Demo 剧本

```text
用户打开 /market
→ 查看 Campus Opportunity Graph 的发布者、来源、Course domain pack、组件与权限
→ 安装 exact version，只授予 read-only scopes
→ 选择培养方案/年级，导入或手工确认已修课程与偏好
→ 问：“下学期我想兼顾概率方向和工作量，怎么选？”
→ 得到 Conservative / Balanced / Exploratory 三个候选方案
→ 每门课展示：满足哪项要求、先修/冲突、社区评价、来源与更新时间
→ 用户改变“周五上午不排课”等偏好，方案重新计算
→ 用户在 Market 禁用插件，Agent 立即失去对应工具
→ Platform Operator 通过审计化 snapshot-import 导入新的官方培养方案 revision
→ 旧方案被标记 stale，用户重新规划
```

这一个剧本同时证明 Market、Agent、Trust Kernel、规划算法、权限和时态更新。

---

## 2. Flagship Plugin：Campus Opportunity Graph；首个 slice：Course Planning

### 2.1 用户问题

Course Planning journey 回答的不是“哪门课评分最高”，而是：

> 在我的培养方案、已修课程、当前学期供给、时间与兴趣约束下，哪些课构成合法且适合我的选择？为什么？哪些信息仍不确定？

### 2.2 首版能力边界

**必须有：**

- 按院系、专业/项目、入学年级查询培养方案；
- 课程号、课程名、教师/开课学期的跨源实体对齐；
- 必修、选修组、学分、建议学期、先修关系等 hard constraints；
- 当前可选课程清单或用户导入的开课清单；
- 时间冲突、学分上下限、个人时间偏好；
- iCourse 评分、难度、作业量、给分、收获等 subjective signals；
- 多方案生成与逐项解释；
- 每个材料事实的 source、revision、retrieved/effective time；
- 缺失、冲突、过期时显式拒绝或降级。

**首版不做：**

- 自动点击教务系统选课；
- 保存 USTC 原始密码；
- 读取成绩单、排名或未授权个人信息；
- 依据匿名点评推断教师或学生身份；
- 把社区点评当作官方事实；
- 用 LLM 绕过先修、学分、时间冲突等硬约束；
- 承诺全校所有特殊班、辅修、双学位规则均已覆盖。

### 2.3 Source authority

| Source | 当前事实 | Authority | MVP 用法 |
|---|---|---|---|
| `catalog.ustc.edu.cn/plan` | 访问会跳转 USTC CAS；实际探测在密码后进入 MFA | **培养方案首要权威** | 优先争取官方 API/授权；否则使用经人工认证导出的 immutable snapshot |
| `icourse.club/program/` | 公开列出大量专业/年级培养方案；页面明确“仅供参考，以教务处培养方案为准” | 公开 secondary mirror | 快速原型、差异发现、链接回官方；不能覆盖官方冲突 |
| `icourse.club/course/` 与课程页 | 公开课程、教师、学期、评分与点评；社区规范称内容公开可访问 | UGC/community signal | 排名软特征、解释与 link-out；不作为培养规则 authority |
| 当前学期课程供给/课表 | 尚未完成来源核验 | **阻塞项** | P0 必须确认 official catalog 是否提供；否则明确降级为“用户主动导入开课清单后的 plan-aware planning”，且 Demo 不暗示实时接入 |
| 用户提供的信息 | 专业/年级、已修课程、偏好、不可用时间 | tenant-private user authority | 最小化存储、可查看/删除，不与公共图谱混写 |

严格规则：

```text
Official catalog snapshot
> reviewed official notice/department source
> iCourse program mirror
> community review signal
> model inference
```

若不同来源冲突，不得静默合并：展示冲突、来源时间和采用哪个 authority 的理由。

### 2.4 已核验的访问与合规边界

1. iCourse 的课程、点评与培养方案页面目前可公开访问；其社区规范说明课程信息和点评公开可见。
2. iCourse 网站代码使用 AGPL-3.0，不等于课程点评与数据自动获得 AGPL 许可；代码许可与数据/UGC 权利必须分开。
3. 当前抓取到的 `robots.txt` 解释了 `search / ai-input / ai-train` content signals，但未出现明确的 yes/no 值；这不等于获得批量抓取或 AI 摘要授权。
4. 在获得站点维护者明确许可前，首版默认只保留 iCourse link-out 与页面标题，不抓取、聚合或缓存点评正文/评分；获得许可后才启用低频 on-demand fetch、conditional request、聚合字段和短缓存。
5. USTC catalog 的 CAS ticket/session 不得转交给第三方 MCP；平台也不能把 Develata 的个人账号作为生产 service account。
6. 本次探测没有绕过 MFA，说明“有账号密码”仍不等于可自动化、可产品化的数据接口。

### 2.5 生产认证方案优先级

```text
A. USTC 官方 API / application / service account（最佳）
B. 管理员在受控环境完成认证，导出 exact snapshot；平台只接收 snapshot（MVP 可行）
C. 用户主动上传官方导出文件或选择 program/year（MVP fallback）
D. Future UserDeviceRelay / browser companion 读取用户当前会话（后续风险 spike）
E. 中央平台保存原始 USTC 密码并模拟登录（禁止）
```

CAS 登录本身只用于用户身份并不能天然授权平台读取 catalog；两个 service 的 token/session scope 必须分开。

### 2.6 PluginPackage 组成

```text
PluginPackage: ustc.opportunity-graph@0.1.0
├── SkillComponent
│   └── 需求澄清、方案解释、uncertainty copy
├── McpServerComponent
│   └── read-only query/planning tools
├── SharedServiceBinding
│   └── reviewed typed opportunity graph + course/plan/review index
└── DeclarativeResourcePack
    ├── course-domain schema/mapping
    ├── source policies
    ├── entity aliases
    ├── plan schema
    ├── scoring profile
    └── render templates
```

MVP 不需要 `ControlledCliComponent`，也不需要 hosted arbitrary code。

建议 MCP/tool surface：

```text
plan.list(program?, cohort?)
plan.get(plan_id, revision?)
course.search(query, term?, instructor?)
course.get(course_key)
review.aggregate(course_key, instructor_key?)
offering.list(term, constraints?)
profile.requirement_status(profile_snapshot, plan_revision)
planner.generate(plan_revision, offering_revision, profile_snapshot, preferences)
planner.explain(candidate_id)
source.provenance(entity_or_fact_id)
```

所有 tool 都是 typed read operation。`planner.generate` 可创建用户自己的 ephemeral/draft plan，但不修改教务系统。

### 2.7 Domain model

```text
ProgramPlan
RequirementGroup
RequirementRule
CourseIdentity
CourseAlias
CourseOffering
InstructorIdentity
ReviewAggregate
ReviewReference
UserAcademicSnapshot
UserPreference
PlanCandidate
PlanRationale
SourceRevision
FactProvenance
ConflictRecord
```

`CourseIdentity` 以 normalized course code 为首要键，但必须允许同名异号、旧号换新号、同号不同版本与跨源 alias。映射不确定时进入 `UnresolvedEntity`，不允许模型自行猜测合并。

这些 course objects 投影到最小 typed Opportunity Graph ontology：

```text
OpportunityNode       # 首版主要是 CourseOffering
RequirementNode       # 培养要求/选修组/资格条件
DependencyEdge        # prerequisite / recommended-before
CoverageEdge          # 某课程满足哪一培养要求
ConflictEdge          # 时间或规则冲突
TemporalWindow        # 开课学期、报名/有效时间
EvidenceSignal        # official fact / community aggregate / uncertainty
ProfileFact           # 用户显式提供且可删除的最小画像
```

Opportunity Graph 是 typed domain projection，不是任意 key-value property graph，也不要求首版部署专用图数据库。科研、竞赛等未来 domain pack 只有在能复用上述资格、依赖、时间、证据与画像语义时才接入；否则不得为“统一”而污染 course slice。

### 2.8 规划算法

硬约束与软偏好必须分层：

```text
Hard constraints:
  requirement legality / prerequisite / term availability
  time conflict / duplicate course / credit bounds

Soft objectives:
  interest fit / workload / difficulty / grading / time preference
  review confidence / plan progress / diversity
```

推荐首版流程：

1. Rust 构造合法候选集合；
2. 对小规模课程集使用 deterministic backtracking / branch-and-bound；
3. 输出多个 Pareto-style candidates，而不是一个“神谕答案”；
4. LLM 只解释候选与询问偏好，不决定合法性；
5. 首版设置明确的候选规模上限；超过预算时分步规划或要求用户收窄候选，不临时引入重型 solver。

社区评价应显示样本量、时间分布与教师/课程版本，不只显示均分。低样本、过期或分歧大的评价必须降低 confidence。

---

## 3. Plugins Market 是产品主轴，不是附属页面

### 3.1 Market 首版必须证明的 contract

```text
Browse
→ Inspect publisher/version/components/data/scopes
→ Install exact PluginPackage version
→ Resolve exact components and grants
→ Enable
→ Agent discovers capability
→ Invoke through gateway
→ Disable / Revoke
→ invocation denied
→ Re-enable / Upgrade / Rollback
```

用户应能理解：它做什么、读什么数据、由谁发布、当前版本是什么、为什么可信、如何关闭。

### 3.2 Authority split

```text
Git catalog repository
  package manifests / schemas / publishers / review evidence
  exact versions / source revision / license / default policy

PostgreSQL
  users / installations / grants / config refs
  run / audit / rollout / source operational projections

Object storage
  immutable source snapshots / evidence / optional artifacts

Redis
  optional cache only; never durable truth
```

冲突时 Git 修复 catalog projection；PostgreSQL 用户安装状态不得反向发布 catalog truth。

### 3.3 收缩 repository topology

原计划一次拆出 `registry/plugin/mcp/skills/scripts/web` 多个 repository，首版过早。建议先用两个 repository：

```text
ustc-campus-agent/          # platform monorepo
plugins-market/             # public catalog authority
```

`plugins-market/` 初始布局：

```text
schemas/
capabilities/
publishers/
packages/
  ustc.opportunity-graph/
components/
  mcp/
  skills/
resources/
review-policy/
fixtures/
```

只有当独立团队、权限、release cadence 或生态规模真实出现时再拆仓。**目录边界先于仓库边界。**

### 3.4 Publication tiers

MVP 支持：

1. `FirstParty`：Campus Opportunity Graph（内含 Course domain pack）；完整 review evidence。
2. `VerifiedCommunityText`：Skill/resource-only，不能执行任意代码。
3. `VerifiedRemoteMcp`：已审查的 owner-scoped Streamable HTTP endpoint。

MVP 不承诺：

- public user-uploaded OCI hosting；
- `command/args/env/cwd` 任意执行；
- in-process JavaScript/TypeScript/WASM hot-load；
- market listing 自动获得 shared/warm runtime；
- 模型自动安装或自动扩大权限。

为了证明 Market 不只是 first-party 菜单，Demo 可额外准备一个极小的 verified community Skill-only package，以及一个只读 remote MCP fixture。

### 3.5 Capability model

首版 Opportunity Graph / Course Planning scopes：

```text
campus.public_plan.read
campus.public_course.read
campus.community_review.read
user.own_academic_snapshot.read
user.own_course_preferences.read
user.own_plan_draft.write
```

其中前三项可由 operator 标记为 `AutoGrantEligible`；tenant-private read 与 plan-draft write 必须在安装界面显式说明。任何 cross-user、成绩、全量 memory、raw credential scope 均禁止 auto-grant。

---

## 4. Agent 与框架参考策略

### 4.1 总原则

```text
Own domain semantics and authority.
Reuse stable protocols and low-differentiation plumbing.
Reference mature lifecycle designs.
Do not merge four frameworks into one runtime.
```

四个重点框架不是“四选一”，而是四类不同证据：

| Reference | 重点借鉴 | 明确不照搬 |
|---|---|---|
| **Rig** | Rust provider/tool types、streaming、structured output、cassettes、MCP/provider plumbing | `AgentRun/AgentRunner` 不成为 canonical run state；framework memory/tool registry 不取代平台 authority |
| **goose** | MCP-first extension UX、CLI/Desktop/API boundary、extension directory、启停/诊断/permission UX、custom distribution | local-first 的任意 command extension、默认自治 mutation、完整本机权限不能进入 central multi-tenant plane |
| **Pi** | minimal core、package 组合 extensions/skills/prompts/resources、resource filtering、registry 分离、session/SDK/RPC 设计 | Pi packages 明示 full system access；不采用 in-process TS hot-load、install scripts、silent collision 或修改参数后不 revalidate |
| **LangGraph** | checkpointer vs store、durable state、interrupt/resume、HITL、fault recovery、复杂 graph 的 benchmark | graph/thread/checkpoint 不成为平台长期 authority；当前 product workflow 不为使用 graph 而 graph 化 |

### 4.2 其余框架的次级参考

- **PydanticAI**：typed dependency/output、validation、eval 与 Python equal-contract worker candidate；
- **OpenAI Agents SDK**：run lifecycle、guardrail ordering、stream/non-stream parity；
- **Microsoft Agent Framework**：middleware、OTel、tenant/session isolation；
- **LlamaIndex/Haystack**：document ingestion、parser 与 bounded retrieval；
- **Agno/AgentOS**：完整 Agent platform 的速度上限与“换皮”反例；
- **Google ADK / CrewAI / AutoGen / Mastra**：多 Agent、workflow、Studio、deployment 的比较样本，不进入首版核心。

### 4.3 当前 runtime 决策

建议保持：

```text
Rust platform authority core
├── deterministic ProductWorkflowRun
├── bounded ConversationRun
├── ModelBackend adapter
├── ToolGateway
└── canonical grant/approval/receipt/audit
```

Rig 首期只做两个 adoption depth 的 spike：

1. `provider/client only`；
2. `typed message/tool types inside adapter`。

不把 Rig Agent runner 直接放进生产核心。LangGraph 只做同合同 durable baseline，不是 competition MVP 的候选 authority；只有真实出现 runtime-defined graph、parallel joins、fork/rewind 或复杂 cross-run signals 时，才重新评估隔离 worker。若未来采用，framework checkpoint 只能是由 `platform_run_id` 索引的 adapter state；Rust grant/approval/receipt/audit 始终为 authority。checkpoint 与 Rust 记录不一致时 fail closed，不得由 checkpoint 反向覆盖平台事实。

### 4.4 Equal-contract spike

Rig/owned Rust loop 与 LangGraph/PydanticAI baseline 使用同一场景：

```text
user request
→ model stream
→ read-only Opportunity Graph course-planning tool proposal
→ grant/schema check
→ approval fixture
→ process restart
→ resume without duplicate receipt
→ streamed final explanation
→ cancellation
```

测量：正确性、unknown-event 行为、cancel latency、restart/resume、supplement code、dependencies、cold start、RSS、review hours。框架选择以证据而非语言偏好决定。

---

## 5. 推荐系统骨架

```text
Web/PWA client
    │ versioned HTTP JSON + SSE
    ▼
ustc-agentd
├── identity/session
├── market catalog projection
├── installation/grant resolver
├── bounded conversation runner
├── tool gateway
├── Opportunity Graph / Course Planning use cases
└── audit/evidence
    │
    ├── PostgreSQL
    ├── Git plugins-market catalog
    ├── immutable source snapshots
    ├── ModelBackend adapter
    └── MCP/typed service adapters
```

初始代码骨架保持紧凑：

```text
apps/
  ustc-agentd       # serve / worker modes
  ustc-agentctl     # config, catalog, source, acceptance, evidence
  web               # Market + Agent + admin review UI

crates/
  platform-core     # domain/runtime/use_cases
  adapters          # postgres/http/model/mcp/git/object
```

只有两个真实 consumer 或 privilege/deploy boundary 出现后才拆 `contracts`、runtime controller 等 crate/process。Android 首版只作为 conditional thin client/PWA wrapper；Web 主闭环未稳定前不并行建设完整原生端。

---

## 6. 46 天执行计划

### P0｜2026-07-22—07-24：冻结闭环与来源 gate

- 团队确认本文主闭环和 non-goals；
- 分配 Product/Source、Backend/Runtime/Security、Frontend/Demo、Evaluation/Release owner；
- 联系 iCourse 维护者确认数据/API/AI 使用边界；
- 完成 catalog MFA 后的只读人工探测，确认 plan/offering endpoint 与导出可能性；
- 冻结 Opportunity Graph 的 Course domain contract、source authority、fixture 与 fallback；
- 只创建最小 implementation docs，不批量造空文档。

**Gate G0：** 当前课程供给至少有一个合法 source/import path；无 owner 或无 source path 则不进入全面实现。若只有用户导入路径，则正式把 MVP 和 Demo 改写为“用户导入开课清单后的 plan-aware planning”，不得暗示官方实时接入。iCourse 在获得明确许可前保持 link-out-only，不将外部回复作为工期单点依赖。

### P1｜2026-07-25—07-29：风险优先 spikes

- Rig provider-only vs owned adapter；
- LangGraph/PydanticAI equal-contract baseline；
- catalog snapshot/import parser；
- 小规模 deterministic course planner（20–30 门候选）。

**Gate G1：** 选择一个 execution path；证明 20–30 门候选课程下 hard constraints 为零违规；明确 parser 与 entity-resolution blocker。

### P2｜2026-07-30—08-06：Market read path + Course contracts

- `plugins-market` schema、capability registry、Opportunity Graph manifest；
- minimal typed Opportunity Graph ontology、Course domain model、tool schema 与 source contract；
- deterministic Git importer 与 PostgreSQL projection；
- iCourse link-out adapter；获得许可后才加入低频 parser 与 course-code entity matching；
- `/market` browse/detail；
- publisher/version/components/permissions/source/license 展示；
- zh-CN 用户界面 + stable English identifiers/i18n keys；完整 en-US copy 为 stretch；
- Web/PWA + SSE 最小连通；
- `ustc-agentctl catalog validate/import/diff`。

**Gate G2：** pinned Git revision 可重建 catalog；malformed/secret-bearing manifest 原子拒绝。

### P3｜2026-08-07—08-15：Install/grant/Agent lifecycle

- development identity + future USTC IdP adapter boundary；
- install/enable/disable/grant/audit；
- bounded conversation stream；
- tool gateway + exact package/component/schema/grant resolution；
- planner spike 接入 Rust hard-constraint validation；
- 定义 `PlatformOperator` role；snapshot import 只允许 operator，记录 source revision、hash 与审计；
- disable 后调用立即失败，re-enable 只恢复仍有效 grant。

**Gate G3：** Market lifecycle 完整跑通；Agent 不再通过硬编码发现 Opportunity Graph 或 Course Planning tools。

### P4｜2026-08-16—08-24：Opportunity Graph / Course Planning real journey

- 集成 plan/offering/review adapters 与前序 normalized models；
- 完成 source revisions、provenance、conflict records；
- 接入 user academic snapshot/preferences；
- 完成 multi-candidate explanation 与 planner/Agent consistency gate；
- stale/conflict/low-confidence UX；
- official-vs-iCourse conflict fixture。

**Gate G4：** 一条真实或经批准 snapshot 的 USTC 选课 journey 可由非开发成员复现；每项关键结论有来源。

### P5｜2026-08-25—09-01：产品化、评测与对抗测试

- polished Market/Agent/plugin detail；
- browser desktop/mobile、keyboard/focus、console/network 检查；
- tenant isolation、credential/log redaction、disable/revoke、source stale tests；
- fixture oracle 与小规模用户试用；
- 若 P4 已完全通过，再追加一个 Skill-only community package 或一个只读 remote MCP fixture；
- deployment/restore/evidence bundle。

**Gate G5：** demo suite 通过；无 required case 被 `Skipped/Unavailable` 冒充 Pass。

### P6｜2026-09-02—09-06：冻结与提交

- 只修 blocker，不加新能力；
- 录制 3–5 分钟主 Demo 与 failure/recovery cut；
- 整理架构图、来源/许可、framework influence、评测结果；
- clean-host restore 与远端部署 read-back；
- 提交材料和运行数据说明。

---

## 7. 最小验收矩阵

### Market

- `MARKET-001` anonymous 可浏览、不可安装；
- `MARKET-002` installation pin exact package/components/schema/grants；
- `MARKET-003` disable 阻断 Agent discovery/invocation；
- `MARKET-004` permission expansion 不自动升级；
- `MARKET-005` Git catalog 可确定性重建 PostgreSQL projection；
- `MARKET-006` revoke 阻断新 invocation，并保留审计。

### Opportunity Graph / Course Planning

- `COURSE-001` official plan 优先于 iCourse mirror；
- `COURSE-002` course-code alias/conflict 不得静默猜测；
- `COURSE-003` planner 对 curated fixtures 的 hard-constraint violation = 0；
- `COURSE-004` 所有 material facts 都有 source revision 与时间；
- `COURSE-005` stale/conflicting/missing offering 产生 uncertainty/refusal；
- `COURSE-006` community rating 不覆盖 official requirement；
- `COURSE-007` user profile/plan tenant isolation；
- `COURSE-008` 删除 academic snapshot 后不可从日志/cache 恢复；
- `COURSE-009` source revision 变化使旧 candidate stale；
- `COURSE-010` planner 输出中的每门课都通过 Rust hard-constraint checker；若 LLM 解释新增或替换了未被 planner 批准的课程，consistency gate 拒绝该解释并回退到 planner 原始结果。

### Agent/runtime

- stream/non-stream 最终状态一致；
- cancel semantics 明确；
- restart/resume 不重复 side effect/receipt；
- Transformer 后重新 schema validation + authorization；
- framework/provider types 不进入 canonical domain persistence。

### 用户价值

建议邀请 8–12 名不同院系学生完成盲测：

- 能否在 5 分钟内完成 program/profile setup；
- 是否能理解方案为什么合法、为什么推荐；
- 是否能识别一条过期或冲突信息；
- 相比手工在两站切换，完成时间与信心是否改善。

小样本结果只作 competition evidence，不夸大成统计显著性结论。

---

## 8. 风险与反悔条件

| 风险 | 等级 | 处理 |
|---|---|---|
| catalog 无 API、MFA 阻塞自动化 | 高 | approved snapshot/import 保底；请求官方接口；不保存个人密码 |
| 当前学期 offering source 缺失 | 高 | P0 gate；无法取得则把 Demo 定义为 plan-aware candidate planning，并要求用户导入 fixture |
| iCourse 数据/AI 使用许可不清 | 高 | 联系维护者；未获明确许可则 link-out-only，不抓取/聚合/缓存点评内容 |
| Market 变成过度治理工程 | 高 | 只实现一个真实 package lifecycle；额外生态 fixture 仅在核心闭环完成后追加 |
| Rust Agent loop 膨胀 | 中高 | equal-contract spike 与 semantic stop conditions；必要时隔离 Python worker |
| 个人学业数据泄露 | 高 | tenant-scoped、最小字段、可删除、日志不含 payload；不读取成绩排名 |
| Android 分散主线 | 中 | Web/PWA first；API/主闭环稳定后再做 thin client |
| 评委认为是框架拼装或通用聊天 | 高 | 把原创贡献固定为 Market authority、Course ontology、constraints、source trust、grant/audit |
| AI coding 产出超过 human review capacity | 高 | 按 owner/reviewer 可审查吞吐切片，不按 token budget 承诺 |

立即重新评估自有 runtime 的条件：runtime-defined graph、nested graph、parallel join/fan-in、fork/rewind、cross-run signal、三个以上差异显著 provider protocol，或同合同 framework spike 明显降低 review/integration 成本。

---

## 9. 团队最小 ownership

| Role | 必须负责 |
|---|---|
| Product / Source owner | Opportunity Graph / Course Planning journey、source authority、iCourse/USTC 沟通、验收 oracle |
| Backend / Runtime / Security owner | Rust core、Market/install/grants、Agent/gateway、data/privacy |
| Frontend / Demo owner | Market + Agent Web/PWA、中文 UX/i18n key skeleton、浏览器证据、Demo narration |
| Evaluation / Release owner | fixtures、acceptance runner、部署/restore、运行数据与提交材料 |

同一成员可兼任，但 Backend/Runtime 的关键状态机和安全边界必须有独立 reviewer。AI agents 可以生成实现、测试与文档，不能替代 owner 对 source、权限、架构和 release evidence 的签字。

---

## 10. 当前建议决策记录

```yaml
project_direction: accept
product_spine_plugins_market: accept
flagship_plugin_opportunity_graph: accept
first_vertical_slice_course_planning: accept
original_three_plugins_all_in_mvp: reject
course_planning_read_only_mvp: accept
central_storage_of_ustc_password: reject
catalog_authority_official_first: accept
icourse_as_secondary_and_subjective_signal: accept
hosted_arbitrary_third_party_code_in_core_demo: reject
rust_platform_authority_core: accept
rig_provider_or_adapter_spike: accept
rig_agent_run_as_canonical_state: reject
goose_as_product_extension_ux_reference: accept
pi_as_package_resource_session_reference: accept
langgraph_as_durable_execution_baseline: accept
langgraph_as_platform_authority: reject
web_pwa_first_android_conditional: accept
initial_two_repository_projection: accept
```

若团队对任一项给出 `ConditionalAccept`，必须附 `owner / deadline / evidence / exit criterion`。

---

## 11. 下一步（只做 outcome-changing work）

1. 团队确认本 v2 的主闭环、non-goals 与四个 owner；
2. 用一次人工 MFA 会话完成 catalog 只读结构探测，确认 plan + offering 的真实数据面；
3. 联系 iCourse 维护者确认数据使用与 API 可能性；
4. 冻结 Opportunity Graph 的最小 typed ontology 与一个具体 program/cohort/term Course Planning acceptance fixture；
5. 之后才创建 implementation repository，并立即做 P1 equal-contract spikes。

在 2–4 未闭合前，不应先搭完整微服务、K3s、Android native、向量数据库或第三方 hosted runtime。

---

## 12. 主要来源

### 项目内部

- `/opt/data/107-competition/README.md`
- `/opt/data/107-competition/architecture-summary.md`
- `/opt/data/107-competition/agent-market-architecture.md`
- `/opt/data/107-competition/agent-runtime-adoption-policy.md`
- `/opt/data/107-competition/platform-acceptance-matrix.md`

### Course data

- USTC 课程目录与培养方案：<https://catalog.ustc.edu.cn/plan>
- USTC 评课社区：<https://icourse.club/>
- iCourse 培养方案：<https://icourse.club/program/>
- iCourse 社区规范：<https://icourse.club/community-rules/>
- iCourse about/source repository：<https://icourse.club/about/>
- iCourse source：<https://github.com/USTC-iCourse/ustc-course>

### Four primary references

- Rig：<https://github.com/0xPlaygrounds/rig>
- goose：<https://github.com/aaif-goose/goose>
- goose extensions：<https://goose-docs.ai/docs/getting-started/using-extensions/>
- Pi：<https://github.com/earendil-works/pi>
- Pi packages：<https://github.com/earendil-works/pi/blob/main/packages/coding-agent/docs/packages.md>
- LangGraph overview：<https://docs.langchain.com/oss/python/langgraph/overview>
- LangGraph persistence：<https://docs.langchain.com/oss/python/langgraph/persistence>
- LangGraph interrupts：<https://docs.langchain.com/oss/python/langgraph/interrupts>
