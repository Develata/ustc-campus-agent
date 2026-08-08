# 08 — Handoff and Redlines

> **Illustrative / No live backend** — 本卷为实现交接规格；不含 Dioxus component/API 发明，不含代码。
> Packet: `m80-default-v0` · Status: `Reviewed` · Source: `2f4de29032560ff3e13d9994b33a3aff14243f44` / tree `53e266c47fdb07d50a734faa24bb11ac4bc5527d`
> 覆盖 artifact：17 Redline/handoff notes

## 1. 交接定位

- 本卷读者：未来 M80 Dioxus implementer 与 reviewer。
- `TRACKED FACT`：M80 实现必须落在 blueprint 的 small-module decomposition 内（`docs/plan/modules/80-dioxus-multi-client.md:259-272`：`app-state`、`routes/design-system`、`market-ui/agent-ui/product UI`、`platform-web`、`platform-android`），并满足 client-core 边界（不得 import backend domain、不得从 local cache/transport close 推断成功）。
- `PROPOSAL`：本 packet 被 reviewed 后，约束 information hierarchy、screen composition、tokens、responsive presentation、component anatomy、interaction feedback、accessibility presentation 与 redlines；不约束 lifecycle/permission/API/availability 语义——那些始终由 owning contracts 决定。

## 2. Layout redlines

### 2.1 Shell

| 项 | 规格 |
|---|---|
| Desktop rail | 224–256px 稳定宽；<768px 折叠为 bottom nav 等价物 |
| Contextual inspector | 280–320px；仅 run/package/installation detail 有真实用途时出现 |
| Workspace | 居中；reading measure 64–72 汉字；technical diff 可更宽 |
| Context zone | page heading 后：唯一 primary + 相邻 secondary/overflow |
| Danger zone | detail 末尾独立分区；destructive 永不在 top bar |

### 2.2 Spacing / grid

- 4px base rhythm；semantic steps 8/12/16/24/32/48；section spacing > component spacing > inline gap。
- Desktop 12-column 仅作 alignment；Android 4-column；content edges 与 actions 共享 anchor。

### 2.3 组件级 redline 摘要

| Primitive | 关键尺寸/行为 |
|---|---|
| AttentionRow | 行高 ≥56px（舒适）；icon 20px；reason 至多两行；动作右置 |
| FacetRow | label 列 96–120px；state+reason 弹性；[detail] 右置 |
| ActionButton | 高 40px（舒适）/36px（紧凑）；min-width 96px；touch ≥44×44 |
| ExactDiffBlock | 行高 32–40px；before/after 成对；组标题 sticky（长 diff） |
| EvidenceSpine | 链节间隔 8px；disclosure 折叠默认；mono-identity 12–13px |
| StepFlow | stepper 顶部；footer actions 右置 primary；步骤 ≤5 |
| BlockingBanner | 全宽；icon+title+reason+action 单行优先，两行封顶 |

（数值为 `PROPOSED_DESIGN_TOKEN` 起点；Stage B 冻结。）

## 3. Type redlines

- 字级表见 `05` 卷 §4.1；页面标题允许两行；authority-critical 不 ellipsis。
- mono-identity 仅用于 digest/ID 摘要；默认截断显示 + 复制完整值。
- 200% text zoom 下无截断/重叠；容器不锁死高度。

## 4. Behavior redlines（实现时必须保持）

1. 每个可变页面从 server projection 渲染 action availability；disabled action 显示 reason（`aria-describedby`）；不显示用户无权知道的动作。
2. Mutation 提交后仅 Pending 投影；结果仅由 typed result/event 改变；timeout-after-possible-acceptance → reconcile 路径；不盲重试。
3. 六层 status 分离：lifecycle/grant/update/runtime/callability/freshness 不得合并为一个 chip。
4. Fixed safety zones（exact approval diff、destructive confirmation、blocking reason、primary lifecycle、disclaimer、critical error）不参与 layout customization，保持 canonical 顺序。
5. Toast/snackbar 仅非权威 local feedback；server mutation 结果一律 inline + activity entry。
6. 日期四分类：published/observed/effective/last verified。
7. Keyboard/focus/reduced-motion/reduced-transparency 行为见 `06` 卷 §2；全部为核心验收项而非增强项。
8. Android：bottom nav 五项；复杂 approval full-page；Custom Tab 经 admitted port；rotation/process recreation 后重读 projection。

## 5. 与未来实现的边界（non-goals for this handoff）

- 不指定 Dioxus component 名、props、signal 结构或路由类型；
- 不指定 M10 DTO/route/timing；所有 intent 名称保持 `PROPOSED_SEMANTIC_INTENT`；
- 不指定 icon 库与 illustration 资产；
- 不承诺任何 acceptance row 通过；验收以 `docs/acceptance/matrix.tsv` 与真实证据为准。

## 6. 导入 repository 时的 checklist（供 governance slice 之后使用）

- [ ] `docs/design/` governance 文件落地且 checker topology 更新、测试 pass；
- [ ] packet 文件原样导入 `docs/design/m80-default-v0/`；
- [ ] `docs/coverage-matrix.md` 登记 design projection；
- [ ] `python3 scripts/check_repo_contracts.py` 与 `git diff --check` 通过；
- [ ] M80 blueprint 仅添加指向 reviewed packet 的链接，不复制正文；
- [ ] 如需上提 invariant（presentation state 非 authority、fixed safety zones、Web/Android 语义等价、server-owned availability），另起 plan/contract 变更 slice，经 Develata 批准。
