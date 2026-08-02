# 07 — Prototype Storyboard

> **Illustrative / No live backend** — 本卷全部 screen、状态与数据为设计样例；无真实后端、无真实安装、无真实授权。
> Packet: `m80-default-v0` · Status: `Proposal` · Source: `2f4de29032560ff3e13d9994b33a3aff14243f44` / tree `53e266c47fdb07d50a734faa24bb11ac4bc5527d`
> 覆盖 artifact：16 Prototype

## 1. Deliverable 16 诚实状态

```text
Delivered: interaction storyboard（16A，本卷 §2–§5）
Delivered: actual clickable prototype（16B，`prototype/index.html`，2026-08-02 第二轮）
```

- 第一轮判断「无 no-code 工具、不写 HTML」已被 Develata 决策覆盖（2026-08-02）：16B 以**自包含静态 HTML** 交付。该文件是设计演示物，**不是** retained frontend skeleton、不含任何 API/route/DTO 发明、不引入仓库代码；全部转场在真实系统中对应 server 确认事件，prototype 内以显式「演示」标注模拟。
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
| S03 | Install review ① 精确包 | plan-loading/conflict | 继续 | →S04 / cancel→S02 |
| S04 | Install review ② 配置 | invalid/secret-ref-unavailable | 继续 | →S05 / back→S03 |
| S05 | Install review ③ 权限审批 | denied/high-risk-unchecked | 继续 | →S06 / back→S04 |
| S06 | Install review ④ 最终摘要 | drift（S06a）/conflict | 批准此安装方案 | →S07 / back→S05 |
| S06a | Plan drift banner 态 | — | 重新生成方案 | →S03' |
| S07 | Install pending | pending/unknown-outcome | （等待） | →S08 / unknown→S07a |
| S07a | Unknown outcome · reconcile | — | 重试核对 / 离开 | →S08（已确认，演示）/ →S09 |
| S08 | Complete-disabled | terminal success | 前往插件管理 | →S09 |
| S09 | Plugins list | empty/stale/partial | 管理 | →S10 |
| S10 | Installation detail | 十变体（见 `02` 卷 §7.4） | 启用 | →S11 / →更新/回滚 flows |
| S11 | Enable review | unavailable（S11a）/pending/conflict | 启用插件 | →S12 / back→S10 |
| S11a | Enable unavailable：需权限复核 | — | 复核权限 | →S13 |
| S12 | Enabled 确认 | terminal | 完成 | →S10 |
| S13 | Grant re-review | stale-grant/scope-change | 批准权限 | →S11 |

（S13 语义来源：B6 收窄 posture，bounded implemented domain evidence，packet 仍 proposal-only；carrier 未定。）

本 prototype 共 **16 个 screen/state IDs**：13 个 main screens（S01–S13）+ 3 个 failure-branch states（S06a、S07a、S11a）。README 索引、本表与 `prototype/index.html` 三者保持一致。

## 4. 关键转场注释

| Transition | Trigger | Pending | Server 确认 | Focus | Back/cancel | Android 行为 |
|---|---|---|---|---|---|---|
| S02→S03 | 「审查安装」 | plan 创建中（skeleton） | exact plan 投影 | S03 heading | cancel→S02 | full-screen step |
| S06→S07 | 「批准此安装方案」 | 禁重复提交 | approval evidence + install receipt | S07 进度区 | 不可 cancel server 操作；可离开页面 | anchored action |
| S06→S06a | （server 事件）plan drift | — | drift 投影 | banner（assertive） | — | 同 desktop |
| S07→S08 | （server 事件）InstalledDisabled | — | lifecycle 事件 | 结果 heading | — | 同 desktop |
| S07→S07a | timeout-after-possible-acceptance | 「正在核对结果」 | reconcile 结果 | reconcile 区 | 可离开页面 | banner |
| S07a→S08 | 「重试核对」（演示） | 核对中（不重复提交） | reconcile 成功（演示） | 结果 heading | — | 同 desktop |
| S10→S11 | 「启用」 | — | availability 投影 | S11 heading | back→S10 | sheet（bounded）或 page |
| S11→S11a | availability=Unavailable+reason | — | reason 投影 | reason 文本 | back→S10 | 同 desktop |
| S11→S12 | 「启用插件」 | pending | Enabled 事件 | 结果 summary | — | 同 desktop |
| S13→S11 | 「批准权限」 | pending | grant receipt + 新 availability | S11 heading | back→S10 | full-page（diff 上下文） |

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
│ [批准此安装方案]            [返回]      [取消]   │
└──────────────────────────────────────────────────┘
Illustrative / No live backend
```

未勾选能力不进入「已授权」集合；对应功能由 server 投影为不可用（denied/unavailable 如实呈现），见 `prototype/index.html` S05/S06 与 `02` 卷 §3。

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

**S12 Enabled 确认**

```text
✓ 已启用（服务器确认）
USTC ChangeRadar 已开始工作。[查看插件]
克制确认：inline state 更新 + activity entry；无 confetti。
```

## 6. Prototype 纪律（16B 必须保持）

- fake data 明确标记；no live backend；no invented API/DTO/route；
- no local click-as-success；action availability/reason 视为 server-owned projection；
- install 完成与 Enable 分离；failure 不自动跳过 review；
- Android 复杂 approval 用 full-page flow，不把 diff 塞进小 bottom sheet。

## 7. 本卷 UNRESOLVED

- 16B tooling 与 external revision 管理（pending）。
- S07 reconcile 的 wire 语义（Q4/Q5 关联）。
