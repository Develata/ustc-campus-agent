# 10 — Grant Diff Review and Activity History

> **Illustrative / No live backend** — 本卷全部线框、示例数据与状态为设计样例；无真实授权、无真实事件流。
> Packet: `m80-default-v0` · Status: `Reviewed` · Source: `2f4de29032560ff3e13d9994b33a3aff14243f44` / tree `53e266c47fdb07d50a734faa24bb11ac4bc5527d`
> 覆盖 required surfaces：Capability/grant diff review（brief §6.11）· Activity/audit history（brief §6.17）
> 本卷为独立 review 后补卷（2026-08-02 第二轮），补齐首轮缺失面；`02` 卷 §5.2 原「查看逐项权限差异」悬空链接的目的地即本卷 §2。

## 1. 本卷 authority 前提

- `TRACKED FACT`：grant 变化的 domain 分类为 `GrantChangeClass { Unchanged, Narrowed, ReapprovalRequired }`（`crates/platform-core/src/market/grant.rs:218`）；registry 间 capability policy 分类为 `CapabilityPolicyChange { Unchanged, Narrowed, ExpansionRequiresReapproval, RemovedOrRevoked }`（`crates/platform-core/src/market/capability.rs:84`）；grant 失效原因 `GrantInvalidationReason { CapabilityManifestChanged, CapabilityDefinitionChanged, InstallationChanged, PolicyChanged }`（`crates/platform-core/src/market/grant.rs:210`）。
- `TRACKED FACT`（B6 语义 bounded implemented；packet 仍 “proposed, not accepted authority”，:42）：Apply/Rollback 原子置 Active grants `Stale(InstallationChanged)`；reactivation 一律消耗 fresh grant approval/evidence——same-scope continuity 用 `Replace`，scope change/addition 用 fresh `Issue`，removed capability 无 replacement（`docs/tasks/campaign-w1-m20-b6.md:52`、:402、:656）；permission expansion 必须 reapproval（M20-LC-008 关联，同文件 :77）。
- `TRACKED FACT`：`InstallationEvent` 为 sequence 化 envelope 且事件不含 raw secret（`docs/contracts/market-lifecycle.md:231`）；`GrantAdmissionEvidence` 的 `Debug` 为 authority-redacted（`crates/platform-core/src/market/grant.rs:248`）；receipts 与 journals 拥有 acknowledged effects（`AGENTS.md` object plane）。
- `PROPOSAL`：brief §6.11 的六分组（Added / Removed / Unchanged / Narrowed / Expanded / MetadataChanged）是 **presentation 分组词汇**（`PROPOSED_PRESENTATION_VOCABULARY`），用于组织 server-projected 的逐项 diff entries；它与上述 domain enum **不是同一词汇**——逐项的 exact 分类与 reason 由 server 投影，UI 不把六分组映射回 domain 判定。分组与 domain 分类的最终 wire vocabulary `UNRESOLVED`（Q10）。

## 2. Capability / grant diff review（brief §6.11，PROPOSAL，high-fidelity）

### 2.1 入口与定位

- 更新审查（`02` 卷 §5.2）「查看逐项权限差异」→ 本页（保留 update plan 上下文返回）。
- Plugins detail 权限 facet「需复核」（V4，grants stale）→ 本页（grant re-review 上下文，storyboard S13）。
- Install step ③ 是**首次逐项审批**（`02` 卷 §4.4）；本页是**变化审查**——两者共用分组组件，但标题、动作与绑定 evidence 不同，不混用。

### 2.2 分组 diff 线框（desktop full-page）

```text
┌────────────────────────────────────────────────────────────────┐
│ 权限变化审查 · USTC ChangeRadar                                 │
│ 上下文：更新 v0.4.0 → v0.5.0 · 方案：待批准（illustrative）     │
│ ──────────────────────────────────────────────────────────────│
│ 新增（Added）· 1 项                                            │
│ ┌────────────────────────────────────────────────────────────┐│
│ │ ☐ 读取课程公告源（read）                                    ││
│ │   范围：教务处公告 · 风险：中（registry 定义）              ││
│ │   说明：本次更新新增的能力请求 · 非默认选中                 ││
│ ├────────────────────────────────────────────────────────────┤│
│ │ 扩大（Expanded）· 0 项                                      ││
│ ├────────────────────────────────────────────────────────────┤│
│ │ 移除（Removed）· 1 项                                      ││
│ │   · 向你推送变化通知（effect）— 新版本不再请求；            ││
│ │     对应授权将终止（history-only），无需操作（只读行）      ││
│ ├────────────────────────────────────────────────────────────┤│
│ │ 收窄（Narrowed）· 0 项                                      ││
│ ├────────────────────────────────────────────────────────────┤│
│ │ 不变（Unchanged）· 1 项                                    ││
│ │   · 读取校园信息源（read）— scope/risk 未变（server-proj.） ││
│ │     continuity 重新批准（默认接受，可逐项查看）             ││
│ ├────────────────────────────────────────────────────────────┤│
│ │ 元数据变化（MetadataChanged）· 0 项                         ││
│ └────────────────────────────────────────────────────────────┘│
│ ──────────────────────────────────────────────────────────────│
│ ⚠ 批准仅对当前这一个方案/安装修订有效；方案变化后需重新审查。   │
│ [稍后]                              [批准这些变化]             │
└────────────────────────────────────────────────────────────────┘
```

规则：

- 分组顺序固定：Added → Expanded → Removed → Narrowed → Unchanged → MetadataChanged；空分组显示「0 项」或折叠，不消失（用户要确认「没有新增」这一事实）。
- **canonical fixture（本 packet 全卷一致）**：v0.4.0 → v0.5.0 为 Added 1（读取课程公告源）· Removed 1（推送变化通知）· Unchanged 1（读取校园信息源，continuity 重新批准，默认接受）· Expanded 0 · Narrowed 0 · MetadataChanged 0；与 `02` 卷 §5.2 变化摘要、`07`/S13 复核屏及 prototype 演示状态一致。
- **Expanded 醒目且非默认选中**；Added 非默认选中；Narrowed/Unchanged/MetadataChanged 默认接受但可逐项查看；Removed 只读（授权终止由 server 执行，UI 不提供「保留已移除能力」）。
- 每项呈现：用户语言名称 + effect/data 类别 + exact scope before→after + registry-owned risk + server-projected reason；scope 差异用 before→after 对照，不用模糊「有变化」。
- 「批准这些变化」绑定 exact plan/installation revision（与 `02` 卷 §5.2 批准语义一致）；plan drift → blocking banner + 回 review；approval consumed → immutable activity entry，不复用。
- Android：full-page review（不把 diff 塞进 bottom sheet，`06` 卷 §1 纪律）；分组为 stacked sections + anchored approval bar。

### 2.3 Grant re-review 变体（更新后 stale-grant 复核）

```text
│ 权限复核 · USTC ChangeRadar
│ 因安装变更，以下授权已失效（服务器判定），需逐项重新批准：
│  · 读取校园信息源（read）— 范围不变 → 重新批准（continuity）
│  · 读取课程公告源（read）— 新增 → 逐项确认（非默认选中）
│ [批准权限]  ← 消耗 fresh approval/evidence；逐项拒绝的后果由
│              server availability 投影（可能整体不可启用）
```

- `PROPOSAL`：same-scope continuity 与 fresh addition 在文案上区分（「重新批准」vs「新批准」），但**都是 fresh approval**（B6 posture 7）；UI 不提供「沿用旧授权」路径。
- 逐项拒绝 → server 决定 installability/enableability（availability 投影）；UI 不自行推导「部分可用」。

### 2.4 状态族

loading（分组 skeleton）；no changes（如实「无权限变化」+ 返回）；diff unavailable（capability registry 投影缺失 → 显示缺口而非编造权限说明）；plan drift；approval consumed；pending；denied（server reason）；success（receipt + activity entry）；offline/stale（只读 + last-sync）。

### 2.5 Semantic needs（`PROPOSED_SEMANTIC_INTENT`）

`GetGrantDiff`（输入：installation + before/after authority refs；输出：server-projected 逐项 diff entries + 分类 + reason）/ `ApproveGrantChanges` / `ReapproveGrants`（continuity Replace 语义，B6 bounded implemented domain evidence，carrier 未定）。`EXISTING_DOMAIN_TYPE` 参考：`GrantChangeClass`（grant.rs:218）、`GrantCommand`（grant.rs:424）、`GrantCommandReceipt`（grant.rs:945）——bounded evidence，**不是 API**。M80 never calculate：change class、scope 比较、reapproval 必要性、partial grant 可用性。

## 3. Activity / audit history（brief §6.17，PROPOSAL，high-fidelity）

### 3.1 定位

- 用户可见的**事件时间线**：安装/授权/更新/运行/凭证引用/来源事件；是 receipts/journals 的 read projection，不是第二事实源。
- IA 位置：一级「动态」（`01` 卷 §2.1）；entity detail 的「活动记录」section 是同 projection 的 scoped view。
- `PROPOSAL`：Activity 是查询与导出面，不提供任何 mutation；所有深链只到 entity detail，不在 timeline 内嵌操作。

### 3.2 Timeline 线框（desktop）

```text
┌────────────────────────────────────────────────────────────────┐
│ 动态                                                            │
│ 类型: 全部 ▾   对象: 全部 ▾   时间: 最近 30 天 ▾   [导出记录]  │
│ ──────────────────────────────────────────────                │
│ 2026-08-01                                                     │
│ ┌────────────────────────────────────────────────────────────┐│
│ │ 14:32  你  停用了  USTC ChangeRadar            [查看插件 →]││
│ │        receipt ✓ · ref rcpt-…（disclosure 可复制）         ││
│ ├────────────────────────────────────────────────────────────┤│
│ │ 11:05  你  批准了  办事导航 0.3.1 的安装方案    [查看记录 →]││
│ │        receipt ✓ · 授予能力 2 项                           ││
│ ├────────────────────────────────────────────────────────────┤│
│ │ 09:12  系统  更新方案已变化  USTC ChangeRadar   [查看更新 →]││
│ │        之前的批准不再适用（server reason）                 ││
│ ├────────────────────────────────────────────────────────────┤│
│ │ 昨天   任务运行  「整理教务通知」 需要你的回答  [继续任务 →]││
│ │        非终态 · 进行中                                     ││
│ └────────────────────────────────────────────────────────────┘│
│ [加载更早]  ← cursor 分页；断流≠无更多                         │
└────────────────────────────────────────────────────────────────┘
```

规则：

- 每行 anatomy：时间 + actor（你/系统/server-projected 主体）+ 动作摘要 + entity deep-link + outcome（receipt ✓ / denied / pending / 非终态）+ disclosure（ref/digest 截断可复制）。
- 动作摘要用用户语言；架构名词（HarnessRun、CommandId 全称）只在 disclosure。
- Filter：类型（安装/授权/更新/运行/凭证/来源）、entity、时间范围；filter 是 read-side 参数，不改写历史。
- Receipt/digest 一律截断显示 + 可复制全文；**redaction**：不含明文凭证/secret；authority-redacted 字段不渲染（与 grant.rs:248、market-lifecycle.md:231 一致）。
- Export：经 `LocalArchivePort`（client-shell.md:249），用户选择去向；导出标注 redaction 与生成时间；导出动作本身是一条新 activity entry（如 server 投影）。

### 3.3 Scoped view（entity detail 内）

- Plugins detail「活动记录」section、run detail 时间线：同 anatomy，scope 固定为该 entity；末尾「查看全部动态 →」回一级面。

### 3.4 状态族

loading（行 skeleton + filter bar 保留）；empty（说明将记录什么 + 一个 next step，不编造示例事件）；offline/stale（banner + last-sync；timeline 只读）；partial stream reconnect（「正在从事件游标恢复」，不重复事件、不宣告 terminal，`04` 卷 §5.1）；error（safe code + 重试，仅幂等 read）；export pending/success/failure（用户控制重试）。

### 3.5 Semantic needs（`PROPOSED_SEMANTIC_INTENT`）

`ListActivityEvents`（cursor 分页 + filter 参数；输出 ordered event summaries + entity refs + outcome + redacted disclosure）/ `WatchActivityEvents`（可选流；cursor/resync，Q5）/ `ExportActivityArchive`（经 LocalArchivePort）。owner：M30 journal / M20 events 经 M10 组装（composition owner `UNRESOLVED`，关联 Q8）。M80 never calculate：事件归类、receipt 有效性、历史改写；terminal vs nonterminal 由 server 标注。

## 4. 本卷 UNRESOLVED 汇总

- **Q10（新增）**：grant diff 六分组 presentation vocabulary 与 domain 分类（`GrantChangeClass`/`CapabilityPolicyChange`）的 wire 映射与逐项 reason vocabulary（owner M20；与 Q9 availability vocabulary 相邻）。
- Q4 carrier；Q5 stream cursor/resync；Q8 聚合 composition owner（Activity projection 是否独立组装面）。
- `ASSUMPTION`：activity 保留窗口与 export 格式为 server policy，本卷不定案。
