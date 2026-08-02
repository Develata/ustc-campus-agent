# 06 — Responsive and Accessibility

> **Illustrative / No live backend** — 本卷全部示例为设计样例。
> Packet: `m80-default-v0` · Status: `Proposal` · Source: `2f4de29032560ff3e13d9994b33a3aff14243f44` / tree `53e266c47fdb07d50a734faa24bb11ac4bc5527d`
> 覆盖 artifact：14 Accessibility annotations（含 responsive、localization、motion、light/dark）

## 1. Responsive strategy（Web/PWA 与 Android 对照）

| Concern | Desktop/PWA | Android |
|---|---|---|
| Navigation | persistent rail；command palette shortcut | 五项 bottom nav；settings/account sheet |
| Detail | content + optional contextual inspector | stacked navigation；复杂审查 full-screen |
| Tables/lists | 对齐列的 table-like rows（packages/events/diffs） | 语义 rows/cards；横向表格不缩放硬塞，改为 stacked fields |
| Actions | page heading/context toolbar/danger zone | anchored 唯一 primary；secondary sheet；destructive 独立 |
| Approval | full page 或保留 diff 上下文的大 sheet | **full-page exact review**；bottom sheet 仅 bounded confirmation |
| Streams | timeline + inspector | chronological feed + reconnect banner |
| External links | 新 browser context（必要时 warning） | Custom Tab（经 admitted `ExternalNavigation` port） |
| Layout edit | pointer + keyboard reorder | Move up/down / choose slot；drag 可选 |

- `PROPOSAL`：breakpoints 由 content behavior 决定（reading measure、action zone 挤压、diff 可读性），不按设备品牌；至少验收 small mobile、medium/tablet、desktop 三档。
- `PROPOSAL`：Web/PWA 与 Android semantic state 必须等价：同一 server projection 在两端渲染为同一 lifecycle/facet/reason；Android 不接收更弱的 reason，也不额外推断。
- 无 horizontal overflow、sticky occlusion、orphaned Chinese wrap（标点悬挂与避头尾按中文排印处理，`ASSUMPTION`：实现期用 CSS `line-break`/`hanging-punctuation` 能力验证）。
- Rotation/process recreation 后重新取得 authoritative projection；不凭本地记忆恢复状态。

## 2. Accessibility contract（Artifact 14，PROPOSAL）

### 2.1 对比度

- Normal text ≥4.5:1，large text ≥3:1；focus/interactive boundaries ≥3:1 对邻近色。
- 实测记录见 `05` 卷 §6（两套方向 light/dark 全部 role 已计算；translucency fallback 需 Stage B 复测）。

### 2.2 Semantic structure

- Landmarks：`nav`（global）、`main`、` complementary`（inspector）、`contentinfo`；heading order 不跳级；native controls 优先；table 用真实 table semantics（header cells）或等价 list semantics。
- 状态更新使用合适 live region：`polite` 用于 facet/result 变化；`assertive` 仅 blocking banner；stream 事件做聚合/节流，**避免 stream flood 朗读**（`PROPOSAL`：新事件以「N 条新事件」摘要 announce，而非逐条）。
- Loading：`aria-busy` 于对应 region；skeleton 不进入 tab order。

### 2.3 Keyboard

- 完整键盘可达：navigation、command palette、filters、dialogs/sheets、diff review、layout reorder（Move up/down 与 drag 等价）。
- `:focus-visible` 清楚（2px accent outline + offset）；overlay 关闭后 focus 返回 trigger；step flow 中步骤切换后 focus 落在该步 heading。
- 快捷键：command palette（如 `Ctrl/Cmd+K`）；不覆盖系统/reader 关键键；所有快捷键有非快捷键等价路径。

### 2.4 Touch

- 通常 ≥44×44 CSS px；danger 与 primary 不紧邻（danger zone 物理分离）；Android system back 行为明确（sheet→关闭回 trigger；full-page approval→返回且不留半批准本地状态）。

### 2.5 Screen reader

- Visual order = reading order；icon-only 有 accessible name；digest/ID 默认摘要 + 「复制完整值」操作，不整串朗读阻断主流程。
- Disabled action 的 reason 可被辅助技术读取（`aria-describedby` 关联 reason 文本）；状态（expanded/selected/invalid/pending/disabled）有 programmatic state。
- Evidence spine 以 list + heading 结构表达，link/button 语义区分（导航 vs 动作）。

### 2.6 Motion / transparency

- 支持 `prefers-reduced-motion`：state transition 降为 instant/短 fade；skeleton 不无限 shimmer；stream 事件无飞入。
- 可行时支持 reduced transparency fallback（translucency 面退化为纯色等效值，且重新通过对比度）。
- Progress 不依赖 looping decorative animation；pending 有文字状态。

### 2.7 Async 安全

- 防 duplicate submit（pending 期间 disable + `aria-disabled`）；可取消 local observation，但不冒充 server cancel；timeout outcome unknown 给 reconcile 路径；success 仅来自 server result/event。

### 2.8 核心屏幕逐屏标注摘要

| 屏幕 | Keyboard 关键点 | Reader 关键点 | Touch/其他 |
|---|---|---|---|
| Install step flow | 步骤间 focus 管理；capability 逐项 checkbox 原生语义 | stepper 当前步骤 announce；capability 风险描述可读 | Android approval 保留 diff 上下文 |
| Update/rollback review | diff 分组可键盘遍历；批准/应用分离两 stop | drift banner `assertive`；diff 行 before/after 成对朗读 | 复杂 diff 用 full-page，不用小 modal |
| Agent thread | composer → options → cancel 顺序合理 | phase 变化 `polite`；事件聚合 announce | anchored input 不被键盘遮挡 |
| Run detail | timeline + receipt rows 可遍历 | denied 原因 + recovery owner 可读 | technical groups 折叠状态 programmatic |
| Layout editor | Move up/down 等价 drag | slot 名与 fixed/lock 状态可读 | Android sheet 操作 |
| Settings | 标准表单 | theme/density 选择即生效并 announce | — |

## 3. Localization and content resilience（Chinese-first）

- Chinese-first；English 仅 optional secondary label。Copy verb-first、active voice、同一动作全流程同名：「停用插件」「批准此方案」「重新检查」。Backend jargon（InstallationDecisionError 等）不作主标题。
- Stress cases（Stage B 必须实测渲染）：20–28 汉字标题、两行 action reason、长 package/publisher 名、SemVer+digest disclosure、中英混排、200% text zoom、Android narrow viewport。
- Authority-critical text 不 ellipsis；列表可截 description 但提供完整 detail。
- 日期区分 published/observed/effective/last verified，不用一个模糊「更新时间」；格式中文环境优先（如「2026 年 8 月 1 日 14:32」）。

## 4. Motion semantics

只为三类目的动：状态 transition、spatial relationship、direct feedback。页面切换克制；event arrival 用 subtle highlight 后归静态；plan drift/denial 不 shake；success 不 confetti。最终 duration/easing 在 Stage B 用真实 stream/device 冻结（`PROPOSED_DESIGN_TOKEN` envelope：120–220ms）。

## 5. 本卷 UNRESOLVED / ASSUMPTION

- `ASSUMPTION`：中文排印细节（避头尾、标点悬挂）依赖目标平台文本引擎能力，Stage B 实测。
- `ASSUMPTION`：Android 字体回退（system sans 中文）表现需真机验证。
- 本卷不定义任何 domain 语义；无 UNRESOLVED 由本卷承载。
