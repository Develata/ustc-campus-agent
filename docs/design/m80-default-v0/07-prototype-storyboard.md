# 07 — Prototype Storyboard

> **Illustrative / No live backend** — 本卷全部 screen、状态与数据为设计样例；无真实后端、无真实安装、无真实授权。
> Packet: `m80-default-v0` · Status: `Proposal` · Source: `2f4de29032560ff3e13d9994b33a3aff14243f44` / tree `53e266c47fdb07d50a734faa24bb11ac4bc5527d`
> 覆盖 artifact：16 Prototype

## 1. Deliverable 16 诚实状态

```text
Delivered: interaction storyboard（16A，本卷 §2–§5）
Delivered: actual clickable prototype（16B，`prototype/index.html`，2026-08-02 第二轮）
```

- 第一轮判断「无 no-code 工具、不写 HTML」已被 Develata 决策覆盖（2026-08-02）：16B 以**自包含静态 HTML** 交付。该文件是设计演示物，**不是** retained frontend skeleton、不含任何 API/route/DTO 发明；它是 disposable HTML/JS review artifact（non-product、non-retained M80 frontend），以 client-side 演示状态模拟 server 投影；repository checker/tests/CI 集成属于 PR repair 范围，非本 packet candidate 内容；全部转场在真实系统中对应 server 确认事件，prototype 内以显式「演示」标注模拟。
- 演示状态模型（2026-08-03 round-3；round-4 补 enablePending 前提）：**draft ≠ committed**。S05/S13 勾选仅写入草案（D5/D13），返回/取消/离开不提交；committed（已授权集合 / update applied / grants stale / Enabled）仅由显式演示转场改变——S06「批准此安装方案」、S10「应用更新（演示）」、S13「批准已勾选权限」、S11「启用插件」（仅置 enablePending，非成功）、S12「服务器确认：已启用（演示）」（要求 `enablePending && !stale && !enabled`；premise 不满足时确认动作以非按钮状态块（warn + 原因文字）呈现，round-8 起不再渲染 disabled 主按钮，direct `#S12` 不产生 pending、不改变任何状态）。S10/S09/S11 摘要只反映 committed。capability set（canonical fixture，与 `10` 卷 §2.2 一致）：v0.4.0 = {读取校园信息源, 推送变化通知}；v0.5.0 = {读取校园信息源, 读取课程公告源}——diff 为 Added 1（读取课程公告源，非默认选中）· Removed 1（推送变化通知，授权终止 history-only，只读标注，不进复核集合）· Unchanged 1（读取校园信息源，continuity 重新批准，默认接受）· Expanded/Narrowed/MetadataChanged 0。
- 导入仓库时的义务：screen-flow specification（本卷）+ screen IDs/transition table（§3）+ 静态文件本体 + 本卷 revision + export timestamp（2026-08-02）；文件 digest 在入库 commit 中由 Git 记录。

## 2. Journey 覆盖

Happy path：

```text
S01 Market browse → S02 Package detail → S03 Install review（精确包）
  → S04 Configure → S05 Capability review → S06 Final summary / exact approval
  → S07 Install pending → S08 Complete-disabled（InstalledDisabled）
  → S09 Plugins list → S10 Installation detail → S11 Enable review
  → S12 Server-confirmed Enabled
```

Failure branch（选定：plan drift + install unknown outcome + enable 需 fresh grant）：

```text
S06a Plan drift（approval 后 plan 变化）→ S03' 重新审查
S07a Unknown outcome（请求可能已被接受）→ reconcile 重试核对 → S08 / 离开 → S09
S11a Enable unavailable：权限需复核 → S13 Grant re-review → S11
```

## 3. Screen / state IDs 与 transition table

| ID | Screen | 主要状态族 | Primary action | 出口 |
|---|---|---|---|---|
| S01 | Market browse | loading/empty/no-results/stale/error | 查看详情 | →S02 |
| S02 | Package detail | normal/not-runnable/already-installed/stale | 审查安装 | →S03 / →S10（已安装） |
| S03 | Install review ① 精确包 | plan-loading/conflict | 下一步：配置 | →S04 / cancel→S02 |
| S04 | Install review ② 配置 | invalid/secret-ref-unavailable | 下一步：权限审批 | →S05 / back→S03 |
| S05 | Install review ③ 权限审批 | denied/high-risk-unchecked | 下一步：确认安装方案 | →S06 / back→S04 |
| S06 | Install review ④ 最终摘要 | drift（S06a）/conflict | 批准此安装方案（提交草案 → committed） | →S07 / back→S05 |
| S06a | Plan drift banner 态 | — | 重新生成方案 | →S03' |
| S07 | Install pending | pending/unknown-outcome | （等待） | →S08 / unknown→S07a |
| S07a | Unknown outcome · reconcile | — | 重试核对 / 离开 | →S08（已确认，演示）/ →S09 |
| S08 | Complete-disabled | terminal success | 前往插件管理 | →S09 |
| S09 | Plugins list | empty/stale/partial；attention-first 分组随演示状态投影 | 管理/查看更新/复核权限 | →S10 / stale→S13 |
| S10 | Installation detail | 十变体（见 `02` 卷 §7.4）；演示建模 pre-apply/stale/reapproved | 启用 / 应用更新（演示） | →S11 / stale→S13 |
| S11 | Enable review | unavailable（stale → S11a 同语义块）/pending/conflict | 启用插件（仅置 enablePending 提交请求） | →S12 / back→S10 |
| S11a | Enable unavailable：需权限复核 | — | 复核权限 | →S13 |
| S12 | Enable pending → Enabled 确认 | pending → server 确认（演示）；premise 不满足 → 非按钮状态块（warn + reason，round-8 起） | 服务器确认：已启用（演示） | →S10 |
| S13 | Grant re-review | stale-grant/scope-change；勾选为草案，返回不提交 | 批准已勾选权限（fresh approval → committed） | →S11 / back→S10 |

（S13 语义来源：B6 收窄 posture，bounded implemented domain evidence，packet 仍 proposal-only；carrier 未定。）

本 prototype 共 **16 个 screen/state IDs**：13 个 main screens（S01–S13）+ 3 个 failure-branch states（S06a、S07a、S11a）。README 索引、本表与 `prototype/index.html` 三者保持一致。

S09 与 S10 共享同一演示 update 状态（v0.5.0；server-projected）：**pre-apply**（有可用更新；应用要求 lifecycle ∈ {InstalledDisabled, Disabled}）→ S10「应用更新（演示）」→ **grants stale**（S10 权限行如实显示失效、启用不可用 + reason、复核权限入口；S11 呈现 S11a 同语义块）→ S13 逐项复核（草案；返回不提交；v0.5.0 已移除的「推送变化通知」以只读行标注、不进入复核集合；新增的「读取课程公告源」非默认选中）→ S13「批准已勾选权限」→ **reapproved**（S10/S09 不再显示「有可用更新 v0.5.0」，权限行按 v0.5.0 capability set 呈现）。更新审查完整线框见 `02` 卷 §5–§6；真实系统中 Apply 后另有确认窗口（AppliedPendingConfirmation → Confirmed，B6 bounded），16B 简化为单一应用转场。S11a/S13 的「安装变更后权限需复核」对应该更新场景。

S09 分组为 **attention-first**（`02` 卷 §7.1）：update available 或 grants stale 存在时，ChangeRadar 一律进入「需要你处理」置顶分组（lifecycle label 保留如实显示：状态 badge「已启用 / 已安装，未启用 / 权限需复核」+ meta 内「ⓘ 有可用更新 v0.5.0 · 需先停用方可应用」等提示，round-8 起）；仅当 update/stale 等 attention 全部消失后，Enabled 实例才落入普通「已启用」分组。counts、row placement、reason 与 action 同步投影，不出现「attention 为空 + 同一 row 又显示待处理更新」的矛盾态。

## 4. 关键转场注释

| Transition | Trigger | Pending | Server 确认 | Focus | Back/cancel | Android 行为 |
|---|---|---|---|---|---|---|
| S02→S03 | 「审查安装」 | plan 创建中（skeleton） | exact plan 投影 | S03 heading | cancel→S02 | full-screen step |
| S06→S07 | 「批准此安装方案」（提交草案 → committed） | 禁重复提交 | approval evidence + install receipt | S07 进度区 | 不可 cancel server 操作；可离开页面 | anchored action |
| S06→S06a | （server 事件）plan drift | — | drift 投影 | banner（assertive） | — | 同 desktop |
| S07→S08 | （server 事件）InstalledDisabled | — | lifecycle 事件 | 结果 heading | — | 同 desktop |
| S07→S07a | timeout-after-possible-acceptance | 「正在核对结果」 | reconcile 结果 | reconcile 区 | 可离开页面 | banner |
| S07a→S08 | 「重试核对」（演示） | 核对中（不重复提交） | reconcile 成功（演示） | 结果 heading | — | 同 desktop |
| S10→S10（演示） | 「应用更新（演示）」 | — | Apply 事件 + grants stale 投影（真实系统另有确认窗口） | 更新行 | — | 同 desktop |
| S10→S11 | 「启用」 | — | availability 投影 | S11 heading | back→S10 | sheet（bounded）或 page |
| S11→S11a | availability=Unavailable+reason | — | reason 投影 | reason 文本 | back→S10 | 同 desktop |
| S11→S12 | 「启用插件」（仅提交请求，非成功） | pending | —（待确认） | S12 说明区 | 可离开页面 | 同 desktop |
| S12→S12（演示） | 「服务器确认：已启用（演示）」 | — | Enabled 事件 → S10/S09 投影 Enabled | 结果 summary | — | 同 desktop |
| S13→S11 | 「批准已勾选权限」（fresh approval → committed） | pending | grant receipt + 新 availability | S11 heading | back→S10（返回不提交草案） | full-page（diff 上下文） |

## 5. 逐屏 storyboard（关键屏）

**S01 Market browse**——见 `02` 卷 §2.2 线框。假数据标注「illustrative」；状态列仅 catalog 事实。

**S06 Final summary / exact approval**

```text
┌──────────────────────────────────────────────────┐
│ 确认安装方案                        步骤 4/4     │
│ ────────────────────────────────────────────────│
│  · 安装 ustc.change-radar 0.4.0（digest 9f2c…） │
│  · 配置 revision：cfg-…（摘要）                 │
│  · 授予能力：1 项（读取校园信息源 · read）       │
│  · 未授予：推送通知（未勾选 → 不授权）           │
│  · 安装后状态：已安装，未启用                    │
│                                                 │
│ ⚠ 批准仅对这一个方案有效。                       │
│ [批准此安装方案]            [上一步]             │
└──────────────────────────────────────────────────┘
Illustrative / No live backend
```

未勾选能力不进入「已授权」集合；对应功能由 server 投影为不可用（denied/unavailable 如实呈现），见 `prototype/index.html` S05/S06 与 `02` 卷 §3；16B 中 S05/S13 勾选为**草案**（draft），仅 S06/S13 的显式批准转场提交到 committed，S10/S11 摘要只反映 committed（真实系统中为 server 投影）。

**S06a Plan drift（failure branch）**

```text
┌──────────────────────────────────────────────────┐
│ ⚠ 方案已变化                                      │
│ 你审查的方案与当前方案不一致，之前的批准不再适用。│
│ [重新生成方案并审查]                              │
└──────────────────────────────────────────────────┘
Illustrative / No live backend
```

**S07 Install pending / S07a unknown outcome**

```text
正常：「安装进行中…」+ 可离开页面（稍后从插件管理查看）
S07a：「正在核对安装结果」— 请求可能已被服务器接受；
      正在按请求标识核对，不会重复提交。[重试核对]
      （16B 中 S07a 为已实现演示态：S07 → 失败分支演示链接 → S07a）
```

**S08 Complete-disabled**——见 `02` 卷 §4.6。成功仅由 server 事件确认后呈现。

**S11a Enable unavailable（failure branch）**

```text
┌──────────────────────────────────────────────────┐
│ 暂时无法启用                                      │
│ 原因：安装变更后权限需要复核（服务器判定）。      │
│ [复核权限] → S13          [返回]                  │
└──────────────────────────────────────────────────┘
Illustrative / No live backend
```

**S12 Enable pending → Enabled 确认**

```text
「启用插件」（S11，仅置 enablePending 提交请求）→ S12 pending：「启用请求已提交…」+ 可离开页面
  → [服务器确认：已启用（演示）]（显式演示转场；要求 enablePending && !stale && !enabled）
  → S12 confirmed：✓ 已启用（服务器确认）
    USTC ChangeRadar 已开始工作。[查看插件]
    克制确认：inline state 更新 + activity entry；无 confetti。
```

本地点击不产生成功：Enabled 状态仅由「服务器确认（演示）」转场写入 committed，此后 S10/S09 均投影 Enabled（S09 分组同步：无其他 attention 时 ChangeRadar 移入「已启用」，「需要你处理」为空态如实呈现；若仍有 update available 等 attention，则按 attention-first 规则留在「需要你处理」）。**premise guard**：direct `#S12` 不产生 pending——无待确认请求（或 grants stale）时本屏呈现「启用结果（status/receipt）」前提不满足态，确认动作以非按钮状态块（warn 容器 + 原因文字，round-8 起）呈现，任何状态不变。

**S13 Grant re-review（stale-grant 复核）**

- 进入前提：S10「应用更新（演示）」后 grants stale；直接 hash 进入时以「样例呈现」标注并禁用批准动作（真实系统中本屏仅由 server availability 投影进入，`02` 卷 §7.4 V4）。
- 勾选为草案：「返回插件详情」不改变 S10/S11 已授权集合；仅「批准已勾选权限」提交 fresh approval。
- 复核集合 = 已授权且在 v0.5.0 保留的能力（a = Unchanged continuity，默认接受可取消）+ v0.5.0 新增能力（c = Added，非默认选中）；v0.5.0 移除的能力（b = Removed）以只读行显式标注（已授权 → 授权终止 history-only；未授权 → 无授权终止），不进入复核集合，不 invisible 携带。canonical fixture 计数：Added 1 · Removed 1 · Unchanged 1 · Expanded/Narrowed/MetadataChanged 0（与 `10` 卷 §2.2 分组 diff、`02` 卷 §5.2 变化摘要一致）。round-8 起三组以分组容器呈现（分组标题「保留 / 新增 / 移除」+ 括号契约词 Unchanged/Added/Removed），fixture 计数句移入 note（技术层可查）。

## 6. Prototype 纪律（16B 必须保持）

- fake data 明确标记；no live backend；no invented API/DTO/route；
- no local click-as-success；action availability/reason 视为 server-owned projection；
- draft ≠ committed：勾选草案仅显式批准转场提交；返回/取消不提交（S05/S06、S13 同规则）；
- update 演示状态机：pre-apply →（应用更新（演示））→ grants stale →（复核批准）→ reapproved；stale 时启用不可用 + reason；reapproved 后不再显示「有可用更新」；应用要求措辞一律 lifecycle ∈ {InstalledDisabled, Disabled}；
- install 完成与 Enable 分离；failure 不自动跳过 review；
- 非 ChangeRadar 插件的「管理」动作为 out-of-prototype：disabled + 如实标注，不导航到 ChangeRadar detail（S10）；
- 所有 disabled 动作必须 programmatically 暴露原因：`aria-describedby` 指向存在的元素且原因文本非空（S01/S09 out-of-prototype 管理、S10 停用/应用更新/启用、S13 样例呈现批准、danger zone 同规则）；S12 前提不满足确认自 round-8 起为非按钮状态块（warn 容器直接呈现原因），不再是 disabled action；
- Android 复杂 approval 用 full-page flow，不把 diff 塞进小 bottom sheet；
- target-size 声明范围：动作按钮（`button`/`.btn`）与 checkbox 行（`label.ck`）≥44px；原型导航与失败分支演示的 inline 文本链接按 WCAG 2.2 §2.5.8 inline 例外处理，不声明 44px。

## 7. 本卷 UNRESOLVED

- 16B tooling 与 external revision 管理（pending）。
- S07 reconcile 的 wire 语义（Q4/Q5 关联）。
