# 02 — Market and Lifecycle Journeys

> **Illustrative / No live backend** — 本卷全部线框、示例数据与状态为设计样例；所有 update/rollback 画面基于 M20-B6 收窄语义：B6 packet 内容仍声明 “proposed, not accepted authority”（`docs/tasks/campaign-w1-m20-b6.md:42`），但该语义已有 **bounded Rust 实现**（`crate::market::update`，2026-08-02 PR #33；supporting domain/semantic-fake evidence only，非 production/durable/API，不提升 MARKET-004/PKG-020）。
> Packet: `m80-default-v0` · Status: `Proposal` · Source: `2f4de29032560ff3e13d9994b33a3aff14243f44` / tree `53e266c47fdb07d50a734faa24bb11ac4bc5527d`
> 覆盖 artifacts：5 Market browse/detail/install · 6 Update/rollback review · 7 Installed Plugin state system

## 1. 本卷 authority 前提

- `TRACKED FACT`：MVP managed installation states 恰为 `InstalledDisabled / Enabled / Disabled / Revoked / Uninstalled`（`docs/contracts/market-lifecycle.md:100-134`；Rust `ManagedInstallationState`，`crates/platform-core/src/market/installation.rs:538`）。
- `TRACKED FACT`：catalog browse delivery、durable lifecycle mutation、production composition 均 planned（`docs/contracts/market-lifecycle.md:919-928`；`MARKET-001..004` in `docs/acceptance/matrix.tsv`）。
- `TRACKED FACT`：B6 收窄 posture（packet 仍 “proposed, not accepted authority”，:42；语义已有 bounded 实现于 `crate::market::update`）：每 update 一次 exact ready-plan explicit approval；Apply/Rollback 仅 Disabled 态（`InstalledDisabled`/`Disabled`，owner 与 update 已统一两态谓词）；Apply 原子改 pin 且不 Enable；rollback 仅自 `AppliedPendingConfirmation`，`ConfirmAppliedUpdate` 进 terminal `Confirmed` 并关闭 rollback window；之后恢复旧版 = 新 reverse update（新 plan + 新 approval）；Apply/Rollback 原子置 Active grants `Stale(InstallationChanged)`；reactivation 消耗 fresh grant approval/evidence（`docs/tasks/campaign-w1-m20-b6.md` §1、:50、:52）。
- `PROPOSAL`（贯穿）：installation lifecycle、grant、update attention、runtime health、callability、freshness **六层分离呈现**，禁止一个 overloaded status chip。
- `PROPOSAL`：所有动作的 Available/Unavailable/RequiresReview/Pending + user-safe reason 由 server `GetActionAvailability`（`PROPOSED_SEMANTIC_INTENT`）投影；UI 不 recompute permission、installability、update safety、rollback eligibility、health、callability 或 success。

## 2. Market browse / search（Artifact 5a，PROPOSAL，high-fidelity）

### 2.1 Goal

发现并比较 Plugin；**catalog 可见 ≠ 已安装/可运行**。

### 2.2 Desktop 线框

```text
┌────────────────────────────────────────────────────────────────┐
│ 插件市场                                                       │
│ 🔍 搜索插件…        类别: 全部 ▾   来源: 全部 ▾   [重置筛选]   │
│ ──────────────────────────────────────────────────────────────│
│ 名称 · 说明                        发布者      版本    状态     │
│ ──────────────────────────────────────────────────────────────│
│ USTC ChangeRadar                  USTC Campus  0.4.0  未安装   │
│ 校园官方信息源变化追踪（illustrative）Agent 团队                 │
│ 权限摘要：读取 2 项校园信息源                       [查看详情]   │
│ ──────────────────────────────────────────────────────────────│
│ USTC Affairs Navigator            USTC Campus  0.3.1  已安装   │
│ 结构化办事指南（illustrative）      Agent 团队          [管理]   │
│ 权限摘要：读取办事流程数据                                     │
│ ──────────────────────────────────────────────────────────────│
│ Campus Opportunity Graph          USTC Campus  0.2.0  已安装   │
│ 机会匹配（开发中 · illustrative） Agent 团队            [管理]   │
│ 权限摘要：读取机会数据 · 请求你的资料授权                      │
└────────────────────────────────────────────────────────────────┘
```

规则：

- dense list/rows：name/value proposition/publisher/exact version/capability summary/分层 status；不做 decorative card grid。
- 「状态」列只表示 **catalog 层事实**（未安装/已安装/已被撤销不可见等 server-projected label）；不显示 runtime health、不显示“健康/正常”。
- 实现姿态如实呈现：planned/development 的 package 显示对应 label（如「开发中」），不伪装 ready。
- Primary per row「查看详情」；installed 时「管理」；filter reset secondary。

### 2.3 状态

| State | 呈现 |
|---|---|
| Loading | 行级 skeleton，保留筛选栏布局，`aria-busy` |
| No packages | 说明 Market 将显示什么 + 一个 next step；不编造示例 |
| No search results | 显示 query + 清除入口；不猜“你是不是要找” |
| Offline/stale catalog | banner + 最后同步时间戳；行可读，动作显示 unavailable reason |
| Revoked/unavailable | 行被 server 投影移除或标注；UI 不保留“幽灵入口” |
| Error | 具体标题 + safe code + 重试（仅幂等 read） |

### 2.4 Responsive

Desktop 对齐 metadata 列；Android 两行 rows + filter bottom sheet。

### 2.5 Semantic needs

`BrowsePackages` / `SearchPackages` / `GetPackageDetail` 均 `PROPOSED_SEMANTIC_INTENT`（owner M20 via M10）；输入含 catalog cursor/filter/client context；结果含 ordered metadata + revision/freshness；typed errors：unavailable、cursor invalid、incompatible。**M80 never calculate**：latest、installability、trust score、runnable。

## 3. Package detail / trust summary（Artifact 5b，PROPOSAL，high-fidelity）

### 3.1 Goal

理解用途、publisher、exact version、capabilities、source policy 与真实 implementation posture。

### 3.2 信息层级与线框

```text
┌────────────────────────────────────────────────────────────────┐
│ USTC ChangeRadar                                  [审查安装]   │  ← primary
│ 校园官方信息源变化追踪 · v0.4.0 · USTC Campus Agent 团队        │
│ ──────────────────────────────────────────────────────────────│
│ 这个插件做什么                                                 │
│  追踪经审核的校园信息源，发布语义化变化。（illustrative）        │
│                                                                │
│ 权限能力（Capabilities）                        ← 用户语言，     │
│  · 读取校园信息源（read · 低风险的来源是……）        不是 manifest│
│  · …                                              jargon        │
│                                                                │
│ 来源与信任                                                     │
│  · 声明来源：market/ 目录内 reviewed manifest                  │
│  · 实现状态：design-only（当前无可执行组件）        ← 如实       │
│  · 版本历史 / digest：technical disclosure（折叠）             │
│                                                                │
│ ┌ Contextual inspector（desktop）───────────┐                  │
│ │ 版本 0.4.0（精确）                         │                  │
│ │ 安装状态：未安装                            │                  │
│ │ 兼容性：由服务器判定（GetActionAvailability）│                 │
│ │ [查看来源]                                  │                  │
│ └────────────────────────────────────────────┘                 │
└────────────────────────────────────────────────────────────────┘
```

- 层级：value proposition → install/manage action → permissions summary → source/trust → version/components technical disclosure → history。
- 「审查安装」而非「安装」：install 永远经过 exact review journey（§4）。revoked 时无误导 CTA。
- Digest/identity 放 disclosure，默认摘要可复制，遵循 redaction。

### 3.3 状态

missing revision、not runnable（如实标注，非错误）、planned/development、already installed（→「管理」）、stale catalog、offline、permission summary unavailable（capability registry 投影缺失时显示缺口而非编造权限说明）。

### 3.4 Semantic needs

`GetPackageDetail` / `GetActionAvailability`（`PROPOSED_SEMANTIC_INTENT`）；render exact server projection；compatibility/installability vocabulary `UNRESOLVED`（Q9）。

## 4. Install review → configure → permission approval → complete-disabled（Artifact 5c，PROPOSAL，high-fidelity）

### 4.1 旅程总览

```text
Package detail [审查安装]
  → ① 精确包审查（exact package revision）
  → ② 类型化配置（SecretRef 只显示引用状态）
  → ③ 逐项 capability 审批
  → ④ 最终精确摘要
  → ⑤ 完成：「已安装，尚未启用」（InstalledDisabled）
  →（独立 journey）Enable
```

`PROPOSAL`：安装、授权、启用三步不合并；无 silent enable；`defaultInstalled/defaultEnabled` catalog policy 不作为 runtime 事实呈现。

### 4.2 Step ① 精确包审查

```text
┌──────────────────────────────────────────────────┐
│ 安装 USTC ChangeRadar                  步骤 1/4  │
│ ────────────────────────────────────────────────│
│ 将要安装的精确版本                                │
│  · 包：ustc.change-radar                         │
│  · 版本：0.4.0（精确 pin，不会自动跟随新版）      │
│  · 来源 digest：sha256:9f2c…（disclosure 可复制） │
│  · 组件清单：见技术细节（折叠）                   │
│ ────────────────────────────────────────────────│
│                          [取消]      [继续]      │
└──────────────────────────────────────────────────┘
```

### 4.3 Step ② 类型化配置

```text
│ 配置                                步骤 2/4   │
│  · 检查频率：每 6 小时 ▾（typed enum）           │
│  · 通知板：默认公告板 ▾                          │
│  · 凭证引用：无（此插件不需要）                   │
│    ※ 若需凭证：仅显示 SecretRef 状态（已设置/未设置│
│      /引用不可用），永不显示或存储明文            │
```

- Config invalid：field-level 错误 + server reason；SecretRef unavailable：该行标 unavailable + recovery owner。

### 4.4 Step ③ 逐项 capability 审批

```text
│ 权限审批                            步骤 3/4   │
│ 此插件请求以下能力，逐项确认：                    │
│ ┌────────────────────────────────────────────┐ │
│ │ ☑ 读取校园信息源（read）                     │ │
│ │   范围：默认公告板 · 风险：低（registry 定义）│ │
│ ├────────────────────────────────────────────┤ │
│ │ ☐ 向你推送变化通知（effect）                 │ │
│ │   范围：本设备 · 风险：中                    │ │
│ └────────────────────────────────────────────┘ │
│ ※ 权限定义来自平台 registry；插件不能自定风险等级  │
```

- `TRACKED FACT`：capability 的 scope/risk/auto-grant policy 由 registry 而非 package author 拥有（`docs/plan/04-market-and-plugin-lifecycle.md` §6，:118-123；`crates/platform-core/src/market/capability.rs`）。
- Expanded/高风险项醒目且**非默认选中**；拒绝单项 → server 决定是否整体不可安装（availability 投影），UI 不自行推导“部分安装”。
- 未勾选项**不进入授予集合**；后续摘要、安装 receipt 与启用前 review 只反映已批准能力；未授权能力对应功能由 server 投影为不可用（denied/unavailable 如实呈现）。

### 4.5 Step ④ 最终精确摘要

```text
│ 确认安装方案                        步骤 4/4   │
│  · 安装 ustc.change-radar 0.4.0（digest 9f2c…） │
│  · 配置 revision：cfg-…（摘要）                 │
│  · 授予能力：1 项（读取校园信息源 · read）       │
│  · 未授予：推送变化通知（未勾选 → 不授权）        │
│  · 安装后状态：已安装，未启用                    │
│ [批准此安装方案]  ← 绑定 exact plan 的显式审批    │
```

### 4.6 Step ⑤ 完成（complete-disabled）

```text
┌──────────────────────────────────────────────────┐
│ ✓ 已安装，尚未启用                                │
│ USTC ChangeRadar 0.4.0 已安装。                   │
│ 启用后插件才会开始工作；你可以随时在插件管理中启用。│
│ [前往插件管理]           [稍后启用]               │
└──────────────────────────────────────────────────┘
```

`PROPOSAL`：成功仅由 server result/event 确认后呈现；pending 期间显示「安装进行中」，timeout-after-possible-acceptance → reconcile 路径，不盲重试。

### 4.7 状态矩阵（本旅程）

plan loading；config invalid；secret ref unavailable；permission denied；plan/precondition conflict（返回 review）；approval consumed（不可复用）；install pending；terminal success `InstalledDisabled`；partial/unknown outcome → reconcile。

### 4.8 Responsive

Desktop step page（非 modal maze）；Android full-screen steps，approval 保留 diff context（page 或保留上下文的大 sheet）。

### 4.9 Semantic needs

`CreateInstallPlan` / `ApproveInstallPlan` / `InstallPackage` / `ConfigureInstallation` / `GetInstallationDetail` 均 `PROPOSED_SEMANTIC_INTENT`。`EXISTING_DOMAIN_TYPE` 可参考：`InstallationCommand`（installation.rs:750）、`InstallationCommandReceipt`（:1558）、`GrantCommand`（grant.rs:424）——bounded evidence，**不是 API**。M80 never calculate：grants、readiness、success、mint approval。

## 5. Update exact-plan review（Artifact 6a，PROPOSAL，B6 语义为 bounded implemented domain evidence，high-fidelity）

### 5.1 保守旅程（B6 narrowed UX；wire/carrier 仍 provisional）

```text
Plugins → 有可用更新
  → Stage/download/check（可在 approval 前运行，进度可见）
  → Ready plan：审查 exact diff
  → 显式 exact-plan approval（绑定 plan digest）
  → 若 Enabled：Apply 不可用，reason「请先停用插件」→ 显式 Disable
  → Apply exact plan（消费该 approval）
  → plan drift → 回 review（旧 approval 失效）
  → 成功：仍为 Disabled · grants 已 stale →「权限需要复核」
  → fresh grant review（Replace/Issue per B6 posture）
  → 显式 Enable
```

禁止：Disable 被 Apply 暗中捆绑；复用旧 approval；Enabled→Enabled 切换。

### 5.2 更新审查线框（desktop full-page diff）

```text
┌────────────────────────────────────────────────────────────────┐
│ 更新 USTC ChangeRadar                                          │
│ v0.4.0 → v0.5.0 · 方案状态：待批准（ready）                     │
│ ──────────────────────────────────────────────────────────────│
│ 变化摘要                                                       │
│  · 包 pin：0.4.0 (9f2c…) → 0.5.0 (c41a…)                      │
 │  · 权限变化：1 项扩大 · 1 项不变     [查看逐项权限差异 →]    │
 │    （目的地：`10` 卷 §2 grant diff review，保留本方案上下文）│
│  · 来源/执行变化：见技术细节                                    │
│                                                                │
│ 准备情况（server-projected readiness）                         │
│  ✓ 已暂存并校验  ✓ 目标版本可用  ⓘ 需要先停用插件              │
│                                                                │
│ ⚠ 批准仅对这一个方案有效；方案变化后需要重新审查。              │
│ ──────────────────────────────────────────────────────────────│
│ [稍后]                                    [批准此方案]          │
│ ──────────────────────────────────────────────────────────────│
│ 批准后：停用这个插件，然后应用更新。          [停用并继续 →]     │
│ （「应用更新」在启用状态下不可用：请先停用插件）                │
└────────────────────────────────────────────────────────────────┘
```

- 「批准此方案」与「应用更新」是两个独立动作，中间夹显式 Disable。
- Plan drift：blocking banner「方案已变化，之前的批准不再适用」+ [重新生成方案]；不隐藏 diff、不 auto-reapprove。
- Approval consumed：immutable activity entry；UI 不复用 token/string。

### 5.3 状态

staging/checking（进度可见，可留在后台）、not ready（reason）、ready review、approval recorded、plan drift、approval consumed、catalog/policy changed、apply pending、success remains Disabled + grants-stale checkpoint。

### 5.4 Responsive

Desktop full-page diff；Android stacked diff + anchored approval bar；**不可用小 modal 装复杂 diff**。批准后 Danger/注意：Apply 是 normal-confirmation action（非 destructive），但页面必须显示“应用后仍为停用 + 权限需复核”的后果说明。

### 5.5 Semantic needs（均 `PROPOSED_SEMANTIC_INTENT`；B6 domain 语义 bounded implemented，carrier 未定）

`ListUpdateCandidates` / `CreateUpdatePlan` / `GetUpdatePlan` / `ApproveUpdatePlan` / `ApplyUpdate`。typed errors 至少含：`InstallationMustBeDisabled`、plan drift、conflict、evidence mismatch、outcome unknown（→ reconcile）。M80 never calculate：change class、readiness、auto-disable、atomic success。

## 6. Rollback target / exact-plan review（Artifact 6b，PROPOSAL，B6 语义为 bounded implemented domain evidence，high-fidelity）

### 6.1 旅程（B6 收窄后）

```text
Installation detail → 更新历史
  → 若 update 处于 AppliedPendingConfirmation：server 投影 rollback 可用
  → 审查 rollback（恢复精确旧 pin c41a… → 9f2c…）→ 批准 →（须 Disabled）→ 应用
  → 成功：仍为 Disabled · grants stale → 权限复核 → 显式 Enable
  → 若 update 已 Confirmed（terminal）：rollback window 已关闭；
    恢复旧版本 = 新建 reverse update（新 exact plan + 新 approval），UI 指向更新流程
  → 无 window/evidence：只显示 server reason，禁止“撤销一下”式承诺
```

### 6.2 线框

```text
┌──────────────────────────────────────────────────────────────┐
│ 恢复到先前版本                                                │
│ ────────────────────────────────────────────────────────────│
│ 当前：v0.5.0（更新已应用，待确认）                             │
│ 可恢复目标：v0.4.0 · 保留中（server-projected）                │
│                                                              │
│ 回滚将：                                                      │
│  · 把包 pin 精确恢复为 0.4.0 (9f2c…)                          │
│  · 使当前 Active 权限失效（需重新审批）                        │
│  · 完成后插件保持停用状态                                      │
│                                                              │
│ [创建回滚方案] →（方案就绪）→ [批准并回滚]  ← 两个分开的动作   │
│ （启用状态下不可用：请先停用插件）                             │
└──────────────────────────────────────────────────────────────┘

Confirmed 后：
┌──────────────────────────────────────────────────────────────┐
│ 该更新已确认，回滚窗口已关闭。                                 │
│ 如需回到 v0.4.0，请创建一次新的逆向更新（需要新方案与新审批）。│
│ [创建逆向更新方案]                                            │
└──────────────────────────────────────────────────────────────┘
```

### 6.3 状态

no eligible target、artifact unavailable、window unknown/closed、readiness stale、plan drift、conflict、rollback pending、success remains Disabled。

### 6.4 Semantic needs（`PROPOSED_SEMANTIC_INTENT`；B6 domain 语义 bounded implemented，carrier 未定）

`ListRollbackTargets` / `CreateRollbackPlan` / `ApproveRollbackPlan` / `ApplyRollback`。eligibility/window 的最终规则 `UNRESOLVED`（Q1，owning M20）；UI 只消费 server-projected 三态（eligible/unavailable/none + reason）。M80 never calculate：eligibility、window、“上一版本”猜测、fallback/reenable。

## 7. Installed Plugins — list 与 detail 分层状态系统（Artifact 7，PROPOSAL，high-fidelity）

### 7.1 Installed Plugins list（inventory surface）

Goal：一眼回答「我装了什么、哪些需要我处理」；list 是 attention 的聚合入口，不是纯罗列。

```text
┌────────────────────────────────────────────────────────────────┐
│ 我的插件                                          [浏览市场]   │
│ 🔍 筛选…          状态: 全部 ▾                                  │
│ ──────────────────────────────────────────────────────────────│
│ 需要你处理（2）                                ← 置顶分组       │
│ ┌────────────────────────────────────────────────────────────┐│
│ │ USTC ChangeRadar        已启用 · ⓘ 有可用更新 v0.5.0       ││
│ │ 需先停用才能更新（server reason）              [查看更新 →]││
│ ├────────────────────────────────────────────────────────────┤│
│ │ 办事导航                已停用 · ⓘ 权限需复核              ││
│ │ 安装变更后权限需要复核（server reason）        [复核权限 →]││
│ └────────────────────────────────────────────────────────────┘│
│ 已启用（1）                                                    │
│  校园机会图谱        已启用 · 最后验证 2026-08-01  [管理 →]    │
│ 已停用（0）                                                    │
│   — 无 —                                                      │
└────────────────────────────────────────────────────────────────┘
```

规则：

- 分组固定：**Needs attention → Enabled → Disabled**；Needs attention 置顶且带计数；每组内按最近活动排序（server-projected order，UI 不自算 priority）。
- 行 anatomy：名称 + **分层 status**（primary lifecycle label + 至多一个 facet 提示 + server reason）+ 一个动作；不堆 chip，不显示「运行正常」词汇。
- 动作语义：attention 行动作指向解决路径（查看更新/复核权限/继续任务）；平静行为「管理」。
- 状态族：loading（行 skeleton + 筛选栏保留）；empty（说明安装入口 + [浏览市场]，不编造示例）；partial（部分 installation 投影缺失 → 该行显示缺口而非丢弃）；offline/stale（banner + last-sync；mutation 动作 unavailable+reason）；error（safe code + 重试，仅幂等 read）。
- Android：同分组单列；筛选进 bottom sheet；attention 行动作高度 ≥44px。

### 7.2 Detail goal

回答“装了什么、授权什么、现在能否调用、为何”。

### 7.3 分层 anatomy

```text
┌────────────────────────────────────────────────────────────────┐
│ USTC ChangeRadar                              [管理操作 ▾]     │
│                                                                │
│ ① Primary state label：已启用            ← 仅 lifecycle        │
│ ② Supporting reason：服务器投影 · 如「等待权限复核」            │
│ ──────────────────────────────────────────────────────────────│
│ ③ Secondary facets（分行，不堆 chip）                          │
│   权限        ✓ 1 项已授权 / ⓘ 需复核（安装变更后）            │
│   更新        ⓘ 有可用更新 v0.5.0 · 需先停用                   │
│   运行时      ✓ 可用 / ⚠ 异常（server reason）                 │
│   可调用性    ✓ 可调用 / ⓘ 不可调用：原因…                     │
│   新鲜度      最后验证 2026-08-01 14:32                        │
│ ──────────────────────────────────────────────────────────────│
│ Sections: 权限 · 更新 · 运行时与调用 · 配置 · 活动记录          │
│ ──────────────────────────────────────────────────────────────│
│ Danger zone                                                    │
│   [撤销授权]  [卸载]            ← typed destructive confirmation│
└────────────────────────────────────────────────────────────────┘
```

### 7.4 状态变体（8+，每个 facet 独立）

| # | 变体 | Primary label | Facets 呈现 |
|---:|---|---|---|
| V1 | 正常已启用 | 已启用 | 各 facet 平静；连接健康退居 utility |
| V2 | InstalledDisabled（新装） | 已安装，未启用 | 权限✓ · 更新– · 运行时– · 可调用性「未启用」· 动作：启用 |
| V3 | 有可用更新（Enabled） | 已启用 | 更新 ⓘ「v0.5.0 可用 · 应用前需停用」；Apply 动作 disabled+reason |
| V4 | 更新后 grants stale | 已停用 | 权限 ⓘ「安装变更后需复核」[复核权限]；启用不可用 reason「先完成权限复核」 |
| V5 | AppliedPendingConfirmation | 已停用 | 更新 ⓘ「更新已应用，待确认」；rollback 入口可用（server-projected）；[确认更新] |
| V6 | Runtime unhealthy | 已启用 | 运行时 ⚠ + server reason；**不**把 lifecycle 改标为故障；可调用性独立判定 |
| V7 | Non-callable | 已启用 | 可调用性 ⓘ reason 层级（grant/runtime/policy）；指向下一步 |
| V8 | Stale/offline | （保留最后确认 label + stale 标记） | 全部 facets 标 last-sync 时间戳；mutation 动作 unavailable |
| V9 | Revoked（terminal） | 已撤销 | 历史只读；in-flight frozen projection 与历史 receipts 不被改写 |
| V10 | Plan drift 中 | 已停用 | blocking banner「更新方案已变化」+ [重新审查] |

规则：loading/offline/revision conflict/grant stale/runtime unhealthy/non-callable/terminal 各自独立呈现；core reason 不折叠进 accordion（技术细节才可折叠）。

### 7.5 Enable / Disable / Revoke / Uninstall 动作设计（consequence-first）

通用纪律：每个动作 = consequence 说明 → typed confirmation（confirmation class server-owned）→ pending → 仅 server result/event 改变状态；unavailable 时显示 server reason 而非隐藏入口（入口消失仅当 server 不投影该动作）。

**Enable**（normal confirmation as projected）：

```text
┌──────────────────────────────────────────────────┐
│ 启用 USTC ChangeRadar？                           │
│ 启用后：                                          │
│  · 插件将按已授权能力开始工作                     │
│  · 已授权：1 项（read；清单可查看 → 权限 facet）  │
│  · 未授予：推送变化通知（未勾选 → 不授权，功能不可用）│
│ 不影响：历史记录与配置保持不变                    │
│                    [取消]        [启用插件]      │
└──────────────────────────────────────────────────┘
```

- 前置条件不满足（grants stale / runtime 异常 / update 待确认等）：动作 disabled + server reason + 指向解决路径（如 [复核权限 →]，storyboard S11a）；**不**把多前置合并成一个「不可用」。

**Disable**（normal confirmation as projected）：

```text
┌──────────────────────────────────────────────────┐
│ 停用 USTC ChangeRadar？                           │
│ 停用后：                                          │
│  · 不再响应未来的任务调用与计划执行               │
│  · 进行中的运行不受影响（server-projected 说明）  │
│  · 安装、配置、授权与历史记录保留                 │
│                    [取消]        [停用插件]      │
└──────────────────────────────────────────────────┘
```

- 「影响未来 discovery/calls」「in-flight/history 说明」两项必须出现且内容 server-projected；若 server 无法确认 in-flight 影响，如实显示「进行中运行的影响由服务器说明」而非承诺无影响。

**Revoke / Uninstall**（destructive typed confirmation，danger zone）：

```text
┌──────────────────────────────────────────────────┐
│ 卸载 USTC ChangeRadar？                           │
│ 不可逆范围（server-projected）：                  │
│  · 此安装身份将终止，历史转为只读                 │
│  · 对应授权随之终止（history-only）               │
│  · 重新安装将是全新安装身份，需重新审查与授权     │
│ 不会：删除你在服务器上的动态历史                  │
│ 输入插件名称以确认：[____________]                │
│                    [取消]        [确认卸载]      │
└──────────────────────────────────────────────────┘
```

- `TRACKED FACT`：`Revoked`/`Uninstalled` 对该安装身份 terminal；重新安装必须使用新 installation identity；repository 缺失不等于 Uninstalled 事件（`docs/contracts/market-lifecycle.md:117`、:218）。
- **never bundle Disable into Apply/Revoke/Uninstall**；每个动作独立确认。

**动作状态族**（四动作共用）：unavailable+reason（含多前置分列）；pending（禁重复提交，可离开页面）；revision conflict（「数据已变化」→ reload 后 re-review）；timeout-after-possible-acceptance → 「正在核对结果」reconcile by correlation identity，不盲重试；success 仅 server result/event（inline 状态更新 + activity entry，克制确认）；denied（typed reason + recovery owner）。

### 7.6 Semantic needs（`PROPOSED_SEMANTIC_INTENT`）

`GetInstallationDetail` / `GetRuntimeAvailability` / `GetInvocationAvailability` / `GetActionAvailability` / `WatchInstallationEvents` / `EnableInstallation` / `DisableInstallation` / `RevokeInstallation` / `UninstallInstallation` / `ListInstalledPackages`。health/callability owner projection `UNRESOLVED`（Q9 关联）。M80 never calculate：availability、readiness、grant 状态推导、recoverability。

## 8. 本卷 UNRESOLVED / 边界

- Q1 rollback eligibility 最终规则；Q2 grant staleness 细节；Q3 Confirmed 边界；Q9 availability vocabulary。
- `ASSUMPTION`：「权限需要复核」checkpoint 的 generic copy 待 B6 被 accepted 后精确化。
- 本卷任何画面不得把 B6 bounded domain evidence 写成 production/durable readiness；所有 update/rollback mockup 顶部保留语义来源标注。
