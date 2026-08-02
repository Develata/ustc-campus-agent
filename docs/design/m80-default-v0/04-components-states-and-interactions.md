# 04 — Components, States and Interactions

> **Illustrative / No live backend** — 本卷全部组件名、状态与示例为设计样例；组件名是 presentation vocabulary，不是 Dioxus component/API 声明。
> Packet: `m80-default-v0` · Status: `Proposal` · Source: `2f4de29032560ff3e13d9994b33a3aff14243f44` / tree `53e266c47fdb07d50a734faa24bb11ac4bc5527d`
> 覆盖 artifacts：10 Layout customization · 11 Component/state inventory · 13 Interaction annotations · 15 State atlas

## 1. 总原则

- `PROPOSAL`：组件是 affordance，不是默认容器。重复可比较数据用 list/table；card 仅表示可点击对象或独立 bounded unit；section+divider 优先；禁止 card-inside-card。
- `PROPOSAL`：color 永不是唯一信号——每个状态同时用 label、icon/shape、reason 文本。
- `PROPOSAL`：action availability 的 anatomy 固定为：stable `ActionId` + `Available | Unavailable | RequiresReview | Pending` + user-safe reason + precondition identity + required confirmation class（server-owned，`GetActionAvailability`，`PROPOSED_SEMANTIC_INTENT`）。

## 2. Layout customization（Artifact 10，PROPOSAL，mid-fidelity）

### 2.1 方向（已采纳的收窄方向）

**Declarative layout slots + stable semantic action registry + first-class default template**。这是 presentation seam，不是第二 domain protocol。

Conceptual `PROPOSED_M80_PRESENTATION_TYPE`（非 tracked carrier）：

```text
ActionId                  稳定语义动作标识
ViewId / WidgetId         稳定 view/widget 标识
SlotId                    受限语义放置位
LayoutProfile             仅 identifiers + preferences
DefaultTemplateVersion    reset/migration 锚点
DeviceClass               compact | medium | expanded（名称 provisional）
PinnedAction              ActionId + allowed SlotId + order
HiddenOptionalSurface     仅 optional ViewId/WidgetId
MigrationResetReason      version/unknown/removed/unsafe-placement reason
```

示例 slots：`GlobalNavigation`、`PagePrimary`、`PageSecondary`、`TodayAttention`、`TodayQuickActions`、`ContextToolbar`、`CommandPalette`。

### 2.2 Fixed 与 customizable zones

| 类别 | 内容 |
|---|---|
| Fixed safety zones（不可移动/隐藏/改名/合并） | exact approval diff、destructive confirmation、blocking reason、primary lifecycle label、legal/privacy disclaimer、critical error/recovery |
| Customizable | Home widgets reorder；safe quick actions pin/unpin；optional modules hide；comfortable/compact density；bounded sidebar/toolbar/workspace/palette placement |
| 禁止 | 任意 x/y；JS/component/CSS injection；user-authored command payload；layout-driven permission/state；隐藏/伪装 safety action；Notion/Figma builder |

### 2.3 编辑模式线框

Desktop：

```text
┌────────────────────────────────────────────────────────────────┐
│ 自定义布局                              [重置为默认]  [完成]   │
│ ──────────────────────────────────────────────────────────────│
│ 实时预览（可定制区高亮；安全区加锁标灰）                        │
│ ┌────────────────────────────────────────────────────────────┐│
│ │ 今天                                                        ││
│ │ [需要你处理]🔒  ← fixed safety zone，不可移动                ││
│ │ [校园工具]↕     ← 可排序                                    ││
│ │ [继续]↕  [安静动态]↕  [隐藏的可选模块 +]                     ││
│ └────────────────────────────────────────────────────────────┘│
│ 快捷操作（pin/unpin，仅 server-projected safe actions）        │
│  · 📌 查看待处理更新  · 📌 开始任务  · ＋ 添加                 │
│ 密度：(•) 舒适  ( ) 紧凑        预览设备：[手机][平板][桌面]    │
└────────────────────────────────────────────────────────────────┘
```

Android：「Move up/down / choose slot」sheet，避免精细 drag-only；keyboard move controls 与 drag 等价。

### 2.4 规则

- Layout 只存 identifiers/preferences；**绝不**存 label/icon/availability snapshot 作为 authority、command payload、grant、domain state。运行时由当前 action registry 投影。
- Placement 是 per-DeviceClass preference；invalid slot 回退 default；Android 不继承 desktop 坐标。
- DOM/reading/focus order 跟随 semantic order；fixed zones 保持 canonical order。
- Unknown/removed action：保留 tombstone + reason，UI 从 slot 移除并提示；**绝不**映射到 same-name replacement。
- Profile 有 schema/default-template version；migration 产出 explicit reason 并需 review；无法安全迁移则仅 reset optional zones。
- Safe reset：一个动作返回 versioned default；不改变 backend state。
- **Rule of three**：本轮只预留 seam、设计 default、验证两个 device classes；至少三个真实 surfaces 出现稳定重复前，不实现 generic layout engine。

### 2.5 Semantic needs

`GetLayoutProfile` / `SaveLayoutProfile` / `ResetLayoutProfile`（`PROPOSED_SEMANTIC_INTENT`，proposed M80 persistence seam；Q6：server-sync/tenant-owned 未定）。local draft allowed；server profile remains canonical if admitted。M80 never calculate：action state、command payload、authority。

## 3. Component / state inventory（Artifact 11，PROPOSAL）

### 3.1 Visual primitives（presentation vocabulary，非实现 API）

| Primitive | Anatomy | 主要 variants | Ownership 备注 |
|---|---|---|---|
| `AppShell` | global nav + top bar + context zone + workspace | desktop rail / compact rail / bottom nav | M80 presentation |
| `BlockingBanner` | icon + title + reason + action | offline / upgrade / reauth / plan drift | reason server-owned |
| `AttentionRow` | icon + 单行标题 + reason + 至多一个动作 | permission / update / run-input / stale | server-projected |
| `LayeredStatus` | primary label + supporting reason + facet rows | 见 `02` 卷 §7 十变体 | lifecycle owner M20 等 |
| `FacetRow` | facet 名 + state + reason + [detail] | grant/update/runtime/callability/freshness | 各 facet owner |
| `ActionButton` | label + state +（disabled 时）reason | primary/secondary/destructive | availability server-owned |
| `ExactDiffBlock` | before→after 行 + 分组标题 | added/removed/changed | plan owner M20 |
| `EvidenceSpine` | 来源→版本→决定→凭证 链 + disclosure | change/procedure/opportunity/run | M60/M70/M71/M72/M30 |
| `ReceiptRow` | action summary + status + time + ref | completed/denied/pending | M30/M40 projection |
| `StepFlow` | stepper（≤3–5）+ step content + footer actions | install/update | presentation |
| `DangerZone` | 分区标题 + destructive actions + typed confirm | revoke/uninstall/delete profile | confirmation class server |
| `SkeletonBlock` | 保留布局的占位 | per-section loading | presentation |
| `EmptyState` | 说明 + 一个 next step | 各 surface empty | presentation |
| `StaleTag` | stale icon + 时间戳类型 + 时间 | stale/offline/cached | freshness owner |
| `Disclosure` | 折叠技术细节（digest/ID 摘要可复制） | technical details | presentation |
| `Toast/Snackbar` | 非权威 local feedback only | local draft saved 等 | **永不**替代 server 确认 |

### 3.2 清单规则

- 每个 primitive 的 state 枚举必须覆盖 §5 state atlas 中适用项。
- `Toast/Snackbar` 仅反馈非权威 local action（如“草稿已保存”）；server mutation 的结果一律 inline state/event 更新 + activity entry。
- Pill 只用于短状态/filter label，不用于 paragraph CTA；状态 pill 永远带文本 label。

## 4. Interaction annotations（Artifact 13，PROPOSAL）

### 4.1 通用交互协议（每个可变动作适用）

```text
trigger（用户动作/键盘/reader 等价）
  → precondition 检查 = 渲染时 server-projected availability（无本地重算）
  → 提交 typed intent（含 correlation/idempotency identity —— 概念形状，carrier UNRESOLVED Q4）
  → Pending：disable duplicate submit；可取消 local observation，但不冒充 server cancel
  → 结果：仅 server typed result/event 改变状态
  → timeout-after-possible-acceptance → reconcile by correlation identity，不盲重试
  → focus：动作完成后 focus 落在结果 summary 或返回 trigger（overlay 场景）
```

### 4.2 关键交互注释表

| 交互 | Trigger | Pending 呈现 | Server 确认 | Focus 行为 | Recovery |
|---|---|---|---|---|---|
| 批准安装/更新/回滚方案 | 「批准此方案」button / Enter | 按钮 pending + 禁重复 | approval evidence 投影 → 进入下一步 | 移至「下一步」heading | drift → 回 review；consumed → 新方案 |
| Apply update/rollback | 「应用更新」 | inline 进度 + 可留后台 | 原子结果事件 → 新 pin + 仍 Disabled | 移至结果 summary | outcome unknown → reconcile |
| Enable/Disable | context action | pending | lifecycle 事件 | 返回 detail heading | conflict → reload+re-review |
| Revoke/Uninstall/删除资料 | danger zone typed confirm | pending | terminal receipt → 历史只读 | 移至 activity entry | 不可乐观；reconcile only |
| 回答 Agent 阻塞问题 | option/select | pending | 新 phase 事件 | 回到对话区 | not awaiting/expired → 显示原因 |
| 取消任务 | 「取消任务」（如可用） | 「取消协调中」 | terminal 事件 | 时间线终止节点 | stream 断开 ≠ cancel |
| 布局保存 | 「完成」 | pending | versioned profile saved | 返回设置 heading | conflict/migration → reason |
| Snackbar | 非权威 local 动作 | — | — | 不抢 focus | — |

### 4.3 时序边界（PROPOSAL → Stage B 冻结）

- 状态反馈 envelope 120–220ms；spring 仅 direct manipulation；stream event 不机械飞入；approval 无 celebratory motion。具体 duration/easing 待真实 stream/device 冻结（见 `06` 卷 §4）。

## 5. State atlas（Artifact 15，PROPOSAL）

### 5.1 全状态族总表

| State | Presentation | Allowed recovery | Client must not |
|---|---|---|---|
| Loading | 保留布局、scoped skeleton、`aria-busy` | 等待/取消本地导航 | 过早显示 zero-data empty |
| Empty | 说明将出现什么 + 一个 next step | discover/install/start | 编造示例当真记录 |
| Stale | 时间戳 + stale icon/text；安全时内容可读 | refresh | 静默标注为 current |
| Offline | persistent banner；cached 内容标 last synced | reconnect/保存安全草稿 | 声称 mutation 成功 |
| Conflict | 说明数据已变化 | reload 后 re-review | 用旧 precondition 覆盖/重试 |
| Plan drift | blocking banner；旧 approval 不再适用 | 重新生成/审查 exact plan | 隐藏 diff 或 auto-reapprove |
| Approval consumed | immutable activity entry | 如允许，创建新 plan/approval | 复用 token/string |
| Grant stale/revoked | 独立 permission facet + callability reason | 复核新 grant / 联系 owner | 从 enable 推断 granted |
| Artifact unavailable | evidence 行不可用，core state 保留 | 重试取回/联系 owner | 推断 rollback 可用 |
| Runtime unhealthy | runtime facet warning | 服务端 retry/recover | 把 installation 改标 disabled/revoked |
| Non-callable | 显式 reason 层级 | 跟随投影的下一步 | 从本地 health+grant 计算 |
| Unsupported protocol | 全屏 blocking upgrade gate | 更新 client | 向临近版本 dispatch |
| Partial stream reconnect | 「正在从事件游标恢复」；保留最后确认事件 | resume/resync/full refresh | 重复事件或宣告 terminal |
| Permission denied | stable reason + recovery owner | 如 admitted，request/review | fallback 到 same-name tool |
| Error | 具体标题、safe code、下一步 | 仅幂等/admitted 时重试 | 只写「出了点问题」 |
| Success | inline state/event 更新、克制的确认 | 继续 | 从 click/HTTP 断开推断 |

### 5.2 Reason priority（呈现规则）

terminal authority > compatibility/session > lifecycle precondition > grant > runtime > callability > freshness。

`PROPOSAL`：UI 只显示 server-projected primary reason；当 backend facts 冲突时，UI **不自行排序**，而是请求/等待 owning projection 的单一 reason。

### 5.3 状态组合示例（呈现一致性校验）

- Offline + Enabled：lifecycle label 保留最后确认值 + stale 标记；mutation 动作 unavailable（reason：离线）。
- Drift + pending approval：approval 动作消失（不再是同一 plan）；banner 指向重新审查。
- Unhealthy + Enabled：lifecycle 不变；runtime facet ⚠；callability 独立显示。

## 6. 本卷 UNRESOLVED 汇总

- Q4 carrier 形状（correlation/idempotency 的 wire 表达）；Q5 stream timing；Q6 layout persistence owner；Q9 availability vocabulary。
- `ASSUMPTION`：primitive 命名与 slot 命名为 provisional presentation vocabulary，Stage B 冻结。
