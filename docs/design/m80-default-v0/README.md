# M80 Default v0 UI Design Packet

> **Illustrative / No live backend** — 本 packet 全部 mockup、wireframe、prototype 与示例数据均为设计样例，不代表任何已实现后端、真实安装状态或验收结论。

## Packet metadata

| Field | Value |
|---|---|
| Packet | `m80-default-v0` |
| Task | `M80-KIMI-K3-UI-BRIEF-V0` |
| Status | `Proposal` |
| Implementation evidence | none |
| Readiness claim | none |
| Source commit | `2f4de29032560ff3e13d9994b33a3aff14243f44` |
| Source tree | `53e266c47fdb07d50a734faa24bb11ac4bc5527d` |
| Source relation | 绑定 `origin/main` @ `2f4de29`（2026-08-02 第二轮 rebind 核验；首轮绑定 `5e9e5b9`，因 M20-B6 实现合入发生 drift，见 Source drift 记录） |
| Publication | 2026-08-02 经 Deve-hermes operation-specific 授权，以 frozen review surface 发布于 `docs/design/m80-default-v0/`（Draft PR #34，保持 Draft，非 merge-ready；round-1 review repair 已补齐 checker 集成，见 §6） |
| Working copy | 仓库外 `/home/deve/gitclone/ustc-campus-agent-design/m80-default-v0/`（非 Git root，仅为工作副本） |
| Language | Chinese-first；英文仅作 optional secondary label |

## 标签约定

- `TRACKED FACT`：source commit 上可由 repository 证据直接核实的陈述。
- `PROPOSAL`：本 packet 提出的设计决策，等待评审；不构成 accepted authority。
- `ASSUMPTION`：为推进设计而做的显式假设，Stage B 需验证。
- `UNRESOLVED`：owning contract 尚未决定的语义；本 packet 只给 presentation requirement，不定案。
- `PROPOSED_SEMANTIC_INTENT`：未来 M10/M80 contract 的设计需求名称；**不是 API、不是 HTTP route、不是 DTO**。
- `EXISTING_DOMAIN_TYPE`：source commit 上存在的 bounded Rust/domain 类型；**不自动表示存在可用 client carrier**。

## Source drift 记录

| 时间 | 检查 | 结果 |
|---|---|---|
| 2026-08-02 | 任务启动时 `git fetch origin` + rev-parse | 绑定上表 commit/tree；brief 原始绑定 `678590b0…` 已重绑。`UNVERIFIED`：该对象不在本仓库历史中（`git cat-file` 失败），「旧 M20-B6 branch commit」为合理推断而非可核实事实；重绑本身正确且必要 |
| 2026-08-02 | 第一轮收尾复查 | 无 drift（见 §7） |
| 2026-08-02 | 第二轮（review 补卷）中复查 | **检测到 drift**：`origin/main` 前进至 `2f4de29`（PR #33，`feat(market): implement M20-B6 package update lifecycle`），tree `53e266c4`。packet 已 rebind 并逐项复核受影响语义：B6 收窄 posture 不变；新增 bounded Rust 实现事实（tracked fact #6）；全部行号锚点重新核验更新 |
| 2026-08-02 | 第二轮收尾复查 | `git fetch origin` 后 `origin/main` 为 `2f4de29032560ff3e13d9994b33a3aff14243f44` / tree `53e266c47fdb07d50a734faa24bb11ac4bc5527d`；rebind 后无新 drift（见 §7） |

## 相对原 brief 的 tracked 修正

1. `TRACKED FACT`：M20-B6 packet 随 `5e9e5b9` 合入 `docs/tasks/campaign-w1-m20-b6.md`，packet 内容仍声明 “proposed, not accepted authority”（:42）；**2026-08-02 PR #33（`2f4de29`）进一步合入 bounded Rust 实现** `crate::market::update` + `market_package_update` 测试——为 supporting domain/semantic-fake evidence only：不发 production grant/enable/update issuer、不切换 artifact、不构成 durable repository、不提升 MARKET-004/PKG-020（仍 `planned`）、M20 仍 `partial-evidence`（`docs/contracts/market-lifecycle.md:919-928`）。**设计语义前提（收窄 posture）经复核不变**：rollback 仅允许自 `AppliedPendingConfirmation`；`ConfirmAppliedUpdate` 进 terminal `Confirmed` 并关闭窗口；之后恢复旧版本须创建新 reverse update（新 plan + 新 approval）；Apply/Rollback 仅 Disabled 态（`InstalledDisabled`/`Disabled`；实现修复已将 owner decide/replay 与 update 统一为同一两态谓词）且原子地将 transaction-current Active grants 置 `Stale(InstallationChanged)`；`ConfirmAppliedUpdate` 不重新检查 configuration、grants、catalog policy、runtime health 或 executor state（`docs/tasks/campaign-w1-m20-b6.md` §1、:50、:436）。
2. `TRACKED FACT`：brief 称 `InstallationCommand/Event/Snapshot`、`GrantCommand/Event/Snapshot` 为 existing types。实际 source 上存在 `InstallationCommand`（`crates/platform-core/src/market/installation.rs:750`）、`InstallationEventKind`（:899）、`InstallationCommandReceipt`（:1558）、`InstallationSnapshot`（:1118，`pub type InstallationSnapshot = InstallationAggregate;` public type alias）；`GrantCommand`（`crates/platform-core/src/market/grant.rs:424`）、`GrantEventKind`（:526）、`GrantCommandReceipt`（:945）、`GrantSnapshot`（:642，`pub type GrantSnapshot = GrantAggregate;` public type alias）、`GrantState`（`crates/platform-core/src/invocation.rs:110`）。即 Snapshot 名称存在，但实现为 public type alias 而非独立 snapshot 类型；本 packet 引用 existing types 时以实际路径与 alias 语义为准，不假设独立 snapshot carrier。**修正记录**：本行早前版本错误声称「未发现名为 `InstallationSnapshot`/`GrantSnapshot` 的 pub 类型」，经 Draft PR #34 round-1 review 指正，2026-08-02 修复。
3. `TRACKED FACT`：`RunEvent` 存在于 `crates/agent-runtime/src/lib.rs:361`，`RunEventKind` 于 :312，`RunSpec` 于 :38，`AgentRun` 于 :379；`AgentToolsetView` 于 `crates/agent-tool-protocol/src/lib.rs:176`；`ManagedInstallationState` 于 `crates/platform-core/src/market/installation.rs:538`；`GrantChangeClass` 于 `crates/platform-core/src/market/grant.rs:218`；`GrantInvalidationReason` 于 :210；`CapabilityPolicyChange` 于 `crates/platform-core/src/market/capability.rs:84`。
4. `TRACKED FACT`：模块状态与 `docs/plan/modules/00-module-map.md:19-31` 一致：M00/M20/M30/M40 `partial-evidence`，M10 `skeleton`，M50/M51/M60 `planned`，M70/M71 `design-only`，M72 `bounded-spike`，M80 `planned`，M90 `governance-baseline`。`docs/contracts/market-lifecycle.md:919-928` 确认无 durable lifecycle/update repository、无 artifact switch、无 production composition、无 M10/M80 browse/API/UI delivery。
5. `TRACKED FACT`：`docs/acceptance/matrix.tsv` 中 MARKET-004、PKG-020、RADAR-001/002、PROC-001/006/008、FP-001/002/007 均为 `planned`（MARKET-004/PKG-020 的 evidence 文字已更新为含 bounded B6 supporting evidence，但状态仍 planned）；planned 不是 pass。
6. `TRACKED FACT`（B6 bounded 实现类型，rebind 新增）：`UpdateState`（`crates/platform-core/src/market/update.rs:259`）含 `AppliedPendingConfirmation`（:262）与 `Confirmed`（:263）；`UpdateChangeClass` 于 :269；`UpdateCommand` 于 :993；`UpdateCommandAction::ConfirmAppliedUpdate` 于 :926；集成测试 `crates/platform-core/tests/market_package_update.rs`。均为 bounded evidence，**不是 API、不是 client carrier**。

## 设计结论（PROPOSAL）

- 采用两阶段节奏：**Stage A**（本 packet）交付 IA、default template、semantic status system、核心 journey、action/slot seam 与两套视觉方向；**Stage B**（真实 vertical slice 之后）校准 exact copy、stream timing、density、真机生命周期并冻结 component API/tokens。
- 页面中心是“学生此刻要理解或完成的事”，不是架构名词、聊天框或运维指标。
- Signature：**Evidence spine / 证据脊柱**——procedure、change、opportunity、run detail 用同一种安静的 source→revision→decision→receipt 关系表达。
- 推荐视觉方向：**Direction A — Quiet Evidence System** 为 core system；Direction B 的 warmth 仅进入 empty/onboarding/editorial illustration。理由见 `05-visual-directions-and-tokens.md`。
- 拒绝：generic purple-gradient AI dashboard、three-equal-card hero、nested cards、全站 glass、把所有 noun 升为一级 tab。

## 17 项 artifact traceability matrix

| # | Artifact | Owning file/section | Status | 标签 | Stage A fidelity | Stage B calibration dependency | External ref | Reviewer disposition |
|---:|---|---|---|---|---|---|---|---|
| 1 | IA map | `01` §2 | Delivered | PROPOSAL | High-confidence structure | 真实使用可调 label/order，不动 authority | — | pending |
| 2 | Desktop/PWA shell wireframes | `01` §3 | Delivered | PROPOSAL | Mid-fidelity（small/medium/desktop ASCII） | exact breakpoints 待真实内容 | — | pending |
| 3 | Android shell wireframes | `01` §4 | Delivered | PROPOSAL | Mid-fidelity（portrait + medium） | 真机 lifecycle/touch tuning | — | pending |
| 4 | Home default layout | `01` §5 | Delivered | PROPOSAL | High-fidelity default + empty/offline/attention variants | widget density/order 待真实数据 | — | pending |
| 5 | Market browse/detail/install | `02` §2–§4 | Delivered | PROPOSAL | High-fidelity flow 至 complete-disabled | carrier/copy 待 M10/M20 vertical slice | — | pending |
| 6 | Update/rollback review | `02` §5–§6 | Delivered | PROPOSAL（B6 narrowed 语义，bounded implemented domain evidence） | High-fidelity exact diff + disable-first + drift | B6 非 production/durable；eligibility/Confirmed 的 wire 边界未定案 | — | pending |
| 7 | Installed Plugin state system | `02` §7 | Delivered | PROPOSAL | High-fidelity；8+ 状态变体 | runtime/callability projection contract 未定 | — | pending |
| 8 | Agent thread/run | `03` §2–§3 | Delivered | PROPOSAL | Mid-high fidelity | exact events/timing 待 HarnessRun/stream | — | pending |
| 9 | First-party entry patterns | `03` §4–§6 | Delivered | PROPOSAL | High-fidelity 每产品一个代表 detail + shared spine | 全部 product data/actions provisional | — | pending |
| 10 | Layout customization | `04` §2 | Delivered | PROPOSAL | Mid-fidelity desktop+Android | generic schema 待 rule-of-three | — | pending |
| 11 | Component/state inventory | `04` §3 | Delivered | PROPOSAL | Named primitives + anatomy + variants | component API 未冻结 | — | pending |
| 12 | Design tokens | `05` 全卷 | Delivered | PROPOSAL | 两套 token 集 + 对比 + 推荐 + 实测对比度 | final values 待 browser/device 测试 | — | pending |
| 13 | Interaction annotations | `04` §4 | Delivered | PROPOSAL | trigger/precondition/pending/confirmation/focus/recovery | exact timing/stream reconnect 待 Stage B | — | pending |
| 14 | Accessibility annotations | `06` §2 | Delivered | PROPOSAL | 核心屏 keyboard/touch/reader/contrast/reduced-motion | 实现后真实 audit | — | pending |
| 15 | State atlas | `04` §5 | Delivered | PROPOSAL | 全状态族（含 drift/consumed/partial-reconnect/version-skew） | 真实 copy 长度 | — | pending |
| 16 | Clickable prototype | `07` 全卷 + `prototype/index.html` | **16A + 16B Delivered** | PROPOSAL | Storyboard：screen/state IDs + transition table + failure branch；16B：自包含静态 HTML（16 screen/state IDs，hash 导航） | 16B 为设计演示物，非 retained frontend skeleton；Stage B 以真实实现校准 | 无外部 URL；文件在 packet 内 | pending |
| 17 | Redline/handoff notes | `08` 全卷 | Delivered | PROPOSAL | spacing/type/layout/behavior specs | 无 Dioxus component/API 发明 | — | pending |

Deliverable 16 状态：**16A storyboard + 16B clickable prototype 均已 Delivered**。第一轮 16B deferred（无 no-code 工具）经独立 review 记录为 F4 scope 缺口；Develata 2026-08-02 决策以自包含静态 HTML 补齐（见 `07` §1 决策记录）。

## 分卷索引

| 文件 | 覆盖 artifact |
|---|---|
| `01-information-architecture-and-shells.md` | 1, 2, 3, 4 |
| `02-market-and-lifecycle-journeys.md` | 5, 6, 7 |
| `03-agent-and-first-party-journeys.md` | 8, 9 |
| `04-components-states-and-interactions.md` | 10, 11, 13, 15 |
| `05-visual-directions-and-tokens.md` | 12 |
| `06-responsive-and-accessibility.md` | 14 |
| `07-prototype-storyboard.md` | 16 |
| `08-handoff-and-redlines.md` | 17 |
| `09-onboarding-connectivity-and-settings.md` | required surfaces §6.1 / §6.18 + §9.5 client/system intents（第二轮补卷） |
| `10-grant-diff-and-activity.md` | required surfaces §6.11 / §6.17（第二轮补卷） |
| `prototype/index.html` | 16B clickable prototype（16 screen/state IDs：13 main S01–S13 + 3 failure-branch S06a/S07a/S11a） |

`assets/` 子目录（wireframes/mockups/prototype-exports）预留给未来外部 no-code tool 的静态 exports；本轮无外部资产，不创建空目录。所有线框以 ASCII 内联于各卷。

## Candidate manifest（frozen review surface 文件清单）

| Git path | Role |
|---|---|
| `docs/design/m80-default-v0/README.md` | packet index：metadata、标签约定、source drift、tracked 修正、17 项 traceability、分卷索引、Q1–Q11、F1–F10 disposition、§6 governance slice、§7 自查 |
| `docs/design/m80-default-v0/01-information-architecture-and-shells.md` | artifacts 1–4：IA、Desktop/PWA shell、Android shell、Home default |
| `docs/design/m80-default-v0/02-market-and-lifecycle-journeys.md` | artifacts 5–7：catalog/detail/update-rollback/installed-list/enable-disable-revoke-uninstall |
| `docs/design/m80-default-v0/03-agent-and-first-party-journeys.md` | artifacts 8–9：Agent chat/run timeline + 三个 first-party journeys |
| `docs/design/m80-default-v0/04-components-states-and-interactions.md` | artifacts 10/11/13/15：组件清单、状态族、destructive 模式、onboarding |
| `docs/design/m80-default-v0/05-visual-directions-and-tokens.md` | artifact 12：Direction A/B + design tokens |
| `docs/design/m80-default-v0/06-responsive-and-accessibility.md` | artifact 14：responsive + WCAG 2.2 AA（对比度实测表） |
| `docs/design/m80-default-v0/07-prototype-storyboard.md` | artifact 16A：S01–S13 storyboard + 16B 交付说明 |
| `docs/design/m80-default-v0/08-handoff-and-redlines.md` | artifact 17：handoff contract + redlines |
| `docs/design/m80-default-v0/09-onboarding-connectivity-and-settings.md` | 第二轮补卷：first-run/connectivity/settings + 3 个 client/system intents |
| `docs/design/m80-default-v0/10-grant-diff-and-activity.md` | 第二轮补卷：grant diff 六分组 + activity/audit + export/redaction |
| `docs/design/m80-default-v0/prototype/index.html` | artifact 16B：**External disposable executable prototype · Deliverable 16B only · Non-product · Non-retained M80 frontend · No backend/API · No readiness evidence**（16 screen/state IDs，hash 导航，纯静态） |

共 11 个 Markdown + 1 个静态 prototype 文件；无其他静态 assets；无 fixture、无配置变更。代码面如实说明：packet candidate 内容为文档 + disposable HTML/JS review artifact（non-product、non-retained M80 frontend，见 `07` §1）；本 PR 另含 repository checker/tests/CI 集成（review repair commits），属于仓库工程面，不属于 packet candidate 内容。

## Open questions（UNRESOLVED 汇总）

| # | Question | Owning module | UI 需求 shape |
|---:|---|---|---|
| Q1 | Rollback retention/window/eligibility 的最终 wire 规则（B6 收窄为 `AppliedPendingConfirmation` window，bounded implemented；packet 仍非 accepted authority） | M20 | server-projected eligible targets + reason；空/可用/不可用三态 |
| Q2 | Package-pin change 后 grant inheritance/staleness 的 client 可见细节（B6 收窄为 Active→`Stale(InstallationChanged)` + fresh reactivation evidence，bounded implemented） | M20 | apply 后“权限需要复核” checkpoint |
| Q3 | `ConfirmAppliedUpdate/Confirmed` 与 Enable/health/callability 的最终 client 边界（B6 收窄：confirmation 不绑 health/grant/callability，bounded implemented） | M20 | 四个独立 facets，不合并 “Ready” chip |
| Q4 | M10 versioned client-protocol carrier 的 DTO/event/compatibility 形状 | M10 | 本 packet 全部 `PROPOSED_SEMANTIC_INTENT` |
| Q5 | Stream/reconnect 的 cursor/resync/heartbeat 语义 | M10/M30 | cursor + resync 状态族；不定 timing |
| Q6 | LayoutProfile 持久化 owner（server-sync/tenant-owned 未定） | M80 | local draft + proposed versioned profile |
| Q7 | Provider/server settings 的用户可见面（是否 admitted） | M50/M90 | 只显示 safe status/profile refs；不设计 raw secret form |
| Q8 | Today 聚合 read projection 的 composition owner | M10/M00 组装面 | 分 section freshness 标注的聚合投影 |
| Q9 | ActionAvailability 的 vocabulary（reason code 集合、confirmation class 枚举） | 各 action owner | 每可变页 server 返回 action + reason + confirmation class |
| Q10 | Grant diff 六分组 presentation vocabulary 与 domain 分类（`GrantChangeClass`/`CapabilityPolicyChange`）的 wire 映射与逐项 reason vocabulary（第二轮新增，`10` 卷 §1/§4） | M20 | 逐项 diff entries + 分类 + reason server-projected；UI 不映射回 domain 判定 |
| Q11 | 匿名/受限只读 session 是否 admitted（第二轮新增，`09` 卷 §2.4） | M00/M10 | 决定 onboarding「稍后登录」是否存在；server 不投影则按钮不出现 |

## §6 导入与 checker 集成状态（2026-08-03 round-3 repair 更新）

`TRACKED FACT`：`scripts/check_repo_contracts.py` 的 `EXPECTED_DOC_DIRECTORIES` 对 `docs/` 做 exact topology 检查。当前实际状态：

- governance slice 已随 Draft PR #34 commit 1 入库：`docs/AGENTS.md`、`docs/README.md`、`docs/coverage-matrix.md` 各加 design 条目，新建 `docs/design/AGENTS.md`（subordinate presentation role、`Proposal/Reviewed/Superseded`、source binding、authority deferral、external asset/prototype rule）与 `docs/design/README.md`（packet 索引）。
- round-1 review 后 checker 集成已补齐（同 PR repair commit）：`design` 已加入 `EXPECTED_DOC_DIRECTORIES`；`docs/design/AGENTS.md` 与 `docs/design/README.md` 已登记为 key/nonempty files；新增 `check_design_packets`——index status 仅允许 `Proposal|Reviewed|Superseded`、packet 目录与索引精确一致、source commit/tree 为合法 hex 且经 `git rev-parse` 与实际 Git 对象比对一致、packet README metadata 与索引一致；mutation tests 覆盖 missing/empty governance files、未知多余 docs 目录 fail-closed、index/status/binding drift。
- round-2 repair（2026-08-03）：source binding 校验加严为**两步 Git 对象类型检查**（先 `git rev-parse --verify <oid>^{commit}` 确认 commit 对象类型，再 `<oid>^{tree}` 取 tree；tree OID 冒充 commit 一律 fail-closed），新增 stub mutation test 与真实仓库 end-to-end 类型测试；CI `docs-and-contracts` job 的 checkout 改为 `fetch-depth: 0`（保证 source binding 对象在 CI 可解析，`CAMPAIGN_CI_WORKFLOW_SHA256` pin 同步更新）；shallow-clone 复现验证见 §7「仓库检查」行。
- round-3 repair（2026-08-03，仅 16B/storyboard/README/02 §5.2 计数同步；checker/tests/CI 冻结未动）：prototype 演示状态模型重做——draft ≠ committed（S05/S13 勾选仅写草案，返回/取消不提交；committed 仅由 S06 批准、S10 应用更新（演示）、S13 批准、S12 服务器确认（演示）四个显式转场改变）；update 状态机建模为 pre-apply → grants stale → reapproved（stale 时 S10 权限行如实失效、启用不可用 + reason；reapproved 后 S10/S09 不再显示「有可用更新」）；capability set 同步 `10` 卷 §2.2（v0.5.0 新增「读取课程公告源」、移除「推送变化通知」→ 授权终止 history-only 只读标注，不 invisible 携带；02 §5.2 变化摘要计数同步）；S12 改为 pending → 显式「服务器确认（演示）」两步，确认后 S10/S09 均投影 Enabled；非 ChangeRadar 插件「管理」动作改 disabled + out-of-prototype 标注；「Apply 仅 Disabled」措辞统一为 lifecycle ∈ {InstalledDisabled, Disabled}。
- 本 packet 当前为仓库内 Draft PR #34 的 review surface：`python3 scripts/check_repo_contracts.py` PASS；保持 Draft，非 merge-ready（review/merge 前置见 PR body）。

## §7 最终自查（2026-08-02 第二轮：review 补卷后更新）

| 检查项 | 结果 |
|---|---|
| Source drift 复查 | 第二轮中检测到 drift：`origin/main` 前进至 `2f4de29`（PR #33，M20-B6 bounded 实现合入）；packet 已 rebind 至 `2f4de29032560ff3e13d9994b33a3aff14243f44` / tree `53e266c47fdb07d50a734faa24bb11ac4bc5527d`，受影响语义逐项复核（B6 posture 不变；disabled 两态谓词已统一；全部行号锚点更新）；收尾复查无新 drift |
| M20-B6 状态重述 | packet 内容仍 “proposed, not accepted authority”（b6 :42），但语义已有 bounded Rust 实现（`crate::market::update`；supporting evidence only）；`02`/`07`/`10`/prototype 已全部改为 “bounded implemented domain evidence，非 production/durable” 表述 |
| 独立 review 处置（2026-08-02 第一轮 review 汇报） | F1（4 缺失面）→ `09`/`10` 补卷；F2（2 薄面）→ `02` §7.1/§7.5 加厚；F3（3 intents）→ `09` §5；F4 → Develata 决策以 `prototype/index.html` 补齐 16B；F5/F6/F7/F8/F9 → 已修；F10 → 冻结待下一轮决策 |
| 20/20 required surfaces | 第一轮 16/20；第二轮补齐 first-run/connectivity（`09` §2–§3）、grant diff（`10` §2）、activity/audit（`10` §3）、settings（`09` §4）；installed list 与 enable/disable/revoke/uninstall 加厚（`02` §7.1/§7.5） |
| 17 项 artifact 映射 | 全部 Delivered（16 = 16A + 16B） |
| brief §9.5 client/system intents | `GetClientCompatibility`/`GetServerReadiness`/`GetCurrentUserContext` 已补（`09` §5），全部 `PROPOSED_SEMANTIC_INTENT` |
| tracked/proposal/assumption/unresolved 分离 | 各卷显式标注；UNRESOLVED 汇总为 Q1–Q11（新增 Q10 grant diff vocabulary、Q11 匿名 session） |
| API/implementation/readiness invention | 无；prototype 无任何 API/route/DTO 声明，转场显式标注「演示/server 确认」 |
| Design 越权为 domain authority | 无；`10` 卷明确六分组为 presentation vocabulary 且与 `GrantChangeClass`/`CapabilityPolicyChange` 区分（Q10） |
| 每卷 `Illustrative / No live backend` | 11/11 文件具备（含 prototype banner） |
| 对比度实测 | `05` 卷 §6：两方向 light/dark 全 role 计算（第一轮 review 独立复算 56 对全部吻合） |
| Prototype 验证 | 16 screen/state IDs 定义齐全（13 main S01–S13 + 3 failure-branch S06a/S07a/S11a）；全部 hash 链接目标可解析（16↔16）；JS 语法校验通过；无外部网络引用；a11y：main landmark、可见 focus、hash 导航后 focus 回到 heading、disabled destructive 动作 aria-describedby 说明；target size 声明收窄为动作按钮（`button`/`.btn`）与 checkbox 行 ≥44px，inline 文本链接按 WCAG 2.2 §2.5.8 inline 例外（`07` §6）；授权流诚实性（round-3）：draft ≠ committed——S13 勾选 c 后返回，S10 已授权集合不变；仅 S13 批准后改变；S05 授予 b 后 S13 显式标注 b 移除（授权终止，不进入复核集合）；pre-update S10 不把 c 列为 v0.4 当前未授权项；复核批准后 S10 不再显示「有可用更新 v0.5.0」；S11→S12 pending → 显式「服务器确认（演示）」后 S10/S09 均投影 Enabled；Affairs Navigator / Opportunity Graph「管理」disabled + out-of-prototype，不导航到 S10；S07a unknown/reconcile 演示态；S09/S10 update 状态一致（v0.5.0，server-projected，`07` §3）；browser smoke 全 16 屏 × 320/390/768/1200 无横向溢出 |
| Stage B dependency | 各 artifact 行与各卷末尾标明 |
| 仓库检查 | 本地（complete clone，Draft PR #34 round-3 repair head）：`git diff --check` clean；`python3 scripts/check_repo_contracts.py` PASS（含 design topology/index/status/source-binding；source binding 为两步 Git 对象类型校验：先 `<oid>^{commit}` 后 `<oid>^{tree}`）；`python3 -m unittest scripts.tests.test_check_repo_contracts` PASS（round-3 未改动 checker/tests，沿用既有测试基线）；shallow-clone 复现验证（depth-1 clone 缺 source 对象 → `check_design_packets` fail-closed，对应 CI checkout `fetch-depth: 0`）。CI（exact head）三绿状态以 PR body 记录为准——本行不预写 CI 结果 |

### 本轮未实现/未验证

- F10 视觉样例真实渲染 fidelity（文本级，冻结待下一轮决策）；
- 真实物理设备的对比度、排印、touch、lifecycle 验证（Stage B；本轮已做 Chromium 320/390/768/1200 渲染 smoke）；
- 各 `PROPOSED_SEMANTIC_INTENT` 的 carrier 校准（待 M10/M20 vertical slice）；
- Prototype 完整 screen reader / keyboard-only 走查（本轮已修静态 a11y 语义；真实 AT audit 属 Stage B）。

### 需上提 plan/contract 的候选 proposal（均未上提，待 Develata 决定）

1. Presentation state 永不成为 backend authority；fixed safety zones 不可定制；Web/Android semantic state 等价；action availability 必须 server-projected——未来可作为 M80 blueprint/client-shell contract 的 invariant 候选。
2. `ActionAvailability` projection vocabulary（reason code/confirmation class）——Q9，owner 各 action 模块。
3. Today 聚合 read projection 的 composition owner——Q8。
4. Grant diff 六分组 presentation vocabulary 的 wire 映射需求——Q10，owner M20。
