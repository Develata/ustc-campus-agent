# 01 — Information Architecture and Shells

> **Illustrative / No live backend** — 本卷全部线框与示例文案为设计样例。
> Packet: `m80-default-v0` · Status: `Proposal` · Source: `2f4de29032560ff3e13d9994b33a3aff14243f44` / tree `53e266c47fdb07d50a734faa24bb11ac4bc5527d`
> 覆盖 artifacts：1 IA map · 2 Desktop/PWA shell · 3 Android shell · 4 Home default layout

## 1. 设计原则（本卷适用）

- `TRACKED FACT`：M80 是 thin interaction shell，render/serialize server-owned projection、submit typed intent；不做 canonical product calculation/mutation（`docs/plan/modules/80-dioxus-multi-client.md:26`、`docs/contracts/client-shell.md` §3）。
- `PROPOSAL`：不把每个 domain noun 升为一级导航。默认心智模型：“今天 — 使用 Agent — 找插件 — 管插件 — 查记录”。
- `PROPOSAL`：页面中心是“学生此刻要理解或完成的事”。架构名词（HarnessRun、InvocationAuthority 等）只出现在 technical disclosure，不出现在默认标题。
- `PROPOSAL`：Web/PWA 与 Android 是 peer-quality targets；Android 不是 desktop 缩小版（`docs/plan/modules/80-dioxus-multi-client.md:24` 要求 Android 为 required peer target）。

## 2. IA map（Artifact 1，PROPOSAL）

### 2.1 全局层级

```text
USTC Campus Agent（非官方学生项目 · 中文品牌名 TBD）
│
├── Today                    今天——注意中心与校园工具入口
│   ├── Needs your attention 需要你处理（注意队列）
│   ├── Campus tools         校园工具（ChangeRadar / Affairs / Opportunity 直达）
│   ├── Continue             继续（进行中的任务运行）
│   └── Quiet activity       安静动态摘要
│
├── Agent                    任务——提交有限任务、跟进运行
│   ├── New task             新任务（composer）
│   ├── Active runs          进行中的运行
│   └── Run history          历史运行（与 Activity 交叉链接）
│
├── Market                   插件市场——发现与比较
│   ├── Browse / Search      浏览 / 搜索
│   └── Package detail       插件详情（信任摘要）
│
├── Plugins                  我的插件——已安装实例管理
│   ├── Needs attention      需要注意
│   ├── Enabled / Disabled   已启用 / 已停用分组
│   └── Installation detail  安装详情（含 update/rollback/grant/runtime facets）
│
├── Activity                 动态——用户可见事件时间线
│   └── Timeline + filters   安装/授权/更新/运行/凭证/来源事件
│
└── Settings                 设置（desktop utility area / Android account sheet）
    ├── Account & server     账号与服务器
    ├── Appearance           外观（theme/density/layout customization）
    ├── Accessibility        无障碍
    └── Diagnostics          诊断（safe classes only）
```

### 2.2 层级规则

| 层级 | Desktop/PWA | Android | 规则（PROPOSAL） |
|---|---|---|---|
| Global | Navigation rail：Today、Agent、Market、Plugins、Activity；Settings 在 rail 底部 utility area | Bottom nav：同五项；Settings 进 account sheet | 恰好五项；Updates 不独立占永久 tab，它是 Today/Plugins 的 attention |
| Contextual | Page title、search/filter、scope tabs、唯一 safe primary action、overflow | Compact app bar、有意义的 filter chips、context action | top bar 不同时出现多个同权重 CTA |
| Detail | sections + 长页 sticky local nav；宽屏可选 280–320px contextual inspector | stacked sections；关键上下文动作放 anchored bottom action area 或 sheet | destructive action 固定在 detail 末尾 danger zone |

- `PROPOSAL`：ChangeRadar / Affairs / Opportunity 的直达入口存在于三处——Today 的 Campus tools section、对应 Plugins detail、command palette——不再增加永久 tab。
- `PROPOSAL`：Sources 是 detail-level evidence（evidence spine 的一环），不做默认一级入口。
- `PROPOSAL`：Command palette 是 power shortcut：可搜索 navigation、server-projected safe actions、recent entities；不替代可发现导航，不绕过 approval，不显示 server 未提供的动作。

### 2.3 Direct entries（不升 tab 的直达）

```text
Today → Campus tools → [ChangeRadar digest] [Affairs next step] [Opportunity suggestions]
Plugins → installation detail → "打开插件"（仅当 server 投影该 entry available）
Command palette → "ChangeRadar" / "办事指南" / "机会" → 对应 surface
```

## 3. Desktop/PWA shell wireframes（Artifact 2，PROPOSAL，mid-fidelity）

### 3.1 Desktop expanded（≥1200px，ASSUMPTION：breakpoint 起点，Stage B 以真实内容校准）

```text
┌────────────────────────────────────────────────────────────────────────────┐
│ Rail 224–256px │  Top bar: page title ············ search · palette · acct │
│                ├──────────────────────────────────────────────────────────┤
│  Today         │                                                          │
│  Agent         │   Context zone: [page heading]  [primary action]         │
│  Market        │   ──────────────────────────────────────────────         │
│  Plugins       │                                                          │
│  Activity      │   Workspace（centered, max reading measure ~64–72 汉字） │
│                │                                                          │
│  ─────────     │                                          ┌─────────────┐ │
│  Settings      │                                          │ Contextual  │ │
│  (utility)     │                                          │ inspector   │ │
│                │                                          │ 280–320px   │ │
│                │                                          │ (仅 detail) │ │
│                │                                          └─────────────┘ │
└────────────────────────────────────────────────────────────────────────────┘
```

规则：

- Rail 稳定 224–256px；workspace 居中；inspector 只在 run/package/installation detail 有真实用途时出现，不做常设空栏。
- Primary action 位于 page heading 后的 context zone；secondary 相邻或收进 overflow；destructive 永远不在 top bar。
- Connection health 正常时退居 top-bar utility（小图标 + label），不抢内容中心；有 blocking 问题时转为 inline banner（见 `04` 卷 state atlas）。

### 3.2 Desktop medium（768–1199px）

```text
┌──────────────────────────────────────────────────────┐
│ Rail 72px（icon + accessible name，可展开）│ Top bar │
│──────────────────────────────────────────────────────│
│  Context zone: heading + primary action              │
│  ──────────────────────────────────────              │
│  Workspace（单列；inspector 转为 detail 内 section） │
└──────────────────────────────────────────────────────┘
```

### 3.3 Desktop/PWA narrow / small（<768px，PWA 小窗或分屏）

行为与 Android shell 对齐（见 §4）：rail 折叠为 bottom nav，语义状态等价。`PROPOSAL`：breakpoint 由 content behavior 决定（reading measure、action zone 是否被挤压），不按设备品牌。

### 3.4 Command palette（desktop 与 Android 共用语义）

```text
┌─ Command palette ────────────────────────────────┐
│ 🔍 搜索页面、插件、允许的操作…                    │
├──────────────────────────────────────────────────┤
│ 页面                                              │
│   Today / Agent / Market / Plugins / Activity    │
│ 最近                                              │
│   USTC ChangeRadar · 插件详情                     │
│   任务运行 #（illustrative）                      │
│ 操作（server-projected available only）           │
│   管理已安装插件 · 查看待处理更新                  │
└──────────────────────────────────────────────────┘
```

- 操作项仅渲染 server `GetActionAvailability`（`PROPOSED_SEMANTIC_INTENT`）投影为 Available 的动作；不可用动作不出现在 palette（避免“看似可点”的 dead affordance）。
- Keyboard：全程可键盘；`Esc` 关闭后 focus 返回 trigger。

## 4. Android shell wireframes（Artifact 3，PROPOSAL，mid-fidelity）

### 4.1 Portrait phone

```text
┌──────────────────────────────┐
│ Compact app bar: title  ⋯    │  ← overflow 进 sheet，不堆图标
├──────────────────────────────┤
│                              │
│  Content（单列 ordered feed）│
│                              │
│                              │
│  ┌────────────────────────┐  │
│  │ Anchored primary action│  │  ← 仅当前页唯一 primary；
│  └────────────────────────┘  │    不遮挡内容、避开手势区
├──────────────────────────────┤
│ Today Agent Market Plugins Act│  ← bottom nav 五项，icon+label
└──────────────────────────────┘
```

### 4.2 Medium / tablet（≥600dp，ASSUMPTION）

```text
┌────────────────────────────────────────────┐
│ App bar                                    │
├──────────┬─────────────────────────────────┤
│ Nav rail │ Content（可与 list-detail 双栏） │
│ (compact)│                                 │
│          │                                 │
├──────────┴─────────────────────────────────┤
│ （bottom nav 收起为 rail；tablet 不强制双栏）│
└────────────────────────────────────────────┘
```

### 4.3 Android 专属规则（PROPOSAL；详细 bottom-sheet contract 见 `06` 卷）

- Bottom nav 恰好五项；label 短中文（今天/任务/市场/插件/动态）。
- Safe contextual actions 可进 bottom sheet；**destructive 与 exact approval 不与 casual actions 混排**；复杂 diff 用 full-page review，不塞进小 sheet（详见 `02` 卷 §5–§6）。
- System back：sheet → 关闭回 trigger；full-page approval → 返回上一屏且不保留“半批准”本地状态（approval 只以 server evidence 为准）。
- Rotation/process recreation 后重新读取 authoritative projection；不凭本地记忆恢复状态（`docs/plan/modules/80-dioxus-multi-client.md:122`：不得从 local cache 推断 mutation 成功）。
- Touch target ≥44×44 CSS px（`06` 卷 §2）。
- External links 经 admitted `ExternalNavigation` port / Custom Tab；iCourse 默认 link-out-only（`README.md:103`、`docs/plan/modules/80-dioxus-multi-client.md:94`）。

## 5. Home / Today default layout（Artifact 4，PROPOSAL，high-fidelity）

### 5.1 信息层级

唯一 center of gravity：**Needs your attention / 需要你处理**。默认顺序：

1. **Blocking banner**（仅有问题时）：connection/compatibility/reauth/plan drift；
2. **Needs your attention**：permission review、disable-first update、run needs input、stale result；
3. **Campus tools**：ChangeRadar digest、Affairs next step、Opportunity suggestions；
4. **Continue**：recent active runs；
5. **Quiet activity summary**。

避免 hero welcome、虚构指标、三张等权营销卡。

### 5.2 Desktop 默认态线框

```text
┌────────────────────────────────────────────────────────────────┐
│ 今天                                              [查看更新]   │  ← 唯一 primary：
│ ──────────────────────────────────────────────────────────────│    最高优先
│ 需要你处理                                                     │    attention 的动作；
│ ┌────────────────────────────────────────────────────────────┐│    队列空时为
│ │ ⓘ USTC ChangeRadar 有可用更新 — 先停用才能更新    [查看更新]││    「开始任务」
│ ├────────────────────────────────────────────────────────────┤│
│ │ ⓘ 任务「整理教务通知」需要你的回答                [继续任务]││  ← attention row：
│ ├────────────────────────────────────────────────────────────┤│    icon+标题+server
│ │ ⓘ 办事指南结果可能过期 · 最后验证 2026-07-28      [重新检查]││    reason+一个动作
│ └────────────────────────────────────────────────────────────┘│
│                                                                │
│ 校园工具                                                       │
│  校园变化雷达        办事导航              机会图谱            │
│  本周 3 项已发布变化  下一步：奖助学金…    基于已授权资料…      │
│  [查看]               [继续]              [查看]               │  ← 无数据时 honest empty
│ ──────────────────────────────────────────────────────────────│
│ 继续                                                           │
│  · 任务「对比培养方案」 — 运行中 · 阶段：验证                   │
│ ──────────────────────────────────────────────────────────────│
│ 安静动态                                                       │
│  · 昨天：你停用了「机会图谱」 · 安装了「办事导航」v0.3.1        │
└────────────────────────────────────────────────────────────────┘
```

说明：

- 分组用直接标题 + divider，不用卡片套卡片；attention rows 是 list semantics，不是 marketing cards。
- 每条 attention = icon + 单行标题 + server-owned reason/next step + 至多一个动作。reason 来自 server projection，UI 不自行拼接（`GetActionAvailability`，`PROPOSED_SEMANTIC_INTENT`）。
- 日期区分 published/observed/effective/last verified，不用模糊“更新时间”（`06` 卷 §3）。

### 5.3 Today 变体

**新用户 empty**：

```text
┌────────────────────────────────────────────┐
│ 今天                                        │
│ ───────────────────────────────────────────│
│ 还没有需要处理的事。                         │
│ 从插件市场安装第一个校园工具，或直接开始任务。│
│ [浏览插件市场]                              │  ← 一个 next step，
│                                             │    不编造示例记录
└────────────────────────────────────────────┘
```

**Offline（cached projection）**：

```text
┌────────────────────────────────────────────┐
│ ⚠ 当前离线 · 显示 14:32 的最后同步内容      │  ← persistent banner + timestamp
│ ───────────────────────────────────────────│
│ （内容可读；mutation 动作全部显示            │
│   server-projected unavailable reason）     │
└────────────────────────────────────────────┘
```

**Connection blocking（unsupported protocol）**：全屏 upgrade gate，见 `04` 卷 state atlas `UpgradeRequired`。

### 5.4 Android Today

单列 ordered feed，顺序与 desktop 相同；Campus tools 从三栏变为三行 rows；anchored primary「开始任务」仅在 Agent 上下文出现，Today 的 primary 是最高优先 attention 的动作（跟随内容，不强制悬浮）。

### 5.5 Today 的 semantic needs（全部 `PROPOSED_SEMANTIC_INTENT`）

| Need | Intent | Owner | 说明 |
|---|---|---|---|
| 聚合 read projection | `GetTodayProjection` | UNRESOLVED（Q8：composition owner 未定） | 分 section 携带 freshness/cursor；client 只缓存已标 freshness 的 last safe projection |
| 动作可用性 | `GetActionAvailability` | 各 action owner | 每条 attention 的动作与 reason server-owned |
| 可选事件流 | `WatchTodayEvents` | M10 delivery | cursor/resync；断流不等于状态终止 |

## 6. 本卷 UNRESOLVED 汇总

- Q4 carrier、Q8 Today 聚合 owner、Q9 availability vocabulary（README open questions）。
- `ASSUMPTION`：breakpoint 数值（≥1200 / 768–1199 / <768 / Android 600dp）为设计起点，Stage B 以真实内容与 device class 校准。
