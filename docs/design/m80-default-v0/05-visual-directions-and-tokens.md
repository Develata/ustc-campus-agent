# 05 — Visual Directions and Tokens

> **Illustrative / No live backend** — 本卷全部视觉样例为设计样例。
> Packet: `m80-default-v0` · Status: `Proposal` · Source: `2f4de29032560ff3e13d9994b33a3aff14243f44` / tree `53e266c47fdb07d50a734faa24bb11ac4bc5527d`
> 覆盖 artifact：12 Design tokens（三套方向 + 比较 + 推荐）
> 对比度数字为 WCAG 2.x 相对亮度公式实际计算值（公式与结果表见 §7），非肉眼判断。所有 token 数值为 `PROPOSED_DESIGN_TOKEN` 起点，Stage B 以真实 browser/device 测试冻结。

## 1. Direction A — Quiet Evidence System（已评估未采纳）

### 1.1 概念

Calm institutional/productivity：warm-neutral canvas、ink-first typography、hairline dividers、低彩 semantic accents。Signature 是 evidence spine 与 provenance marker——用排版、细线与小号 mono identity 表达 source→revision→decision→receipt，而不是用色块与发光。

适合：trust、长阅读、diff、audit。风险：过冷；以清晰 microcopy 与少量 campus-life editorial illustration（不伪装官方、不用校徽/校色）增加人味。

### 1.2 来源推理（subject-grounded）

产品 subject 是“可信、可授权、可追溯的校园 Plugin Market + bounded Agent”。用户的主要情绪需求是**确定感**：系统能不能做、为什么不能、下一步是什么。一个低刺激、证据优先的界面直接服务这个确定感；视觉上的克制本身是“我们不夸大 readiness”的非语言陈述，与 repository 的 status-honest 纪律同构。

### 1.3 A tokens — light

| Role | Value | 用途 |
|---|---|---|
| `canvas` | `#FAF9F7` | 页面底 |
| `surface` | `#FFFFFF` | 内容面 |
| `subtle-surface` | `#F2F0EC` | 次级分组底 |
| `ink` | `#1C1B1A` | 主文本 |
| `muted-ink` | `#5B5750` | 次级文本 |
| `divider` | `#E3E0DA` | hairline |
| `accent` | `#2E5A66` | 主动作/链接/focus |
| `success` | `#2E6B4F` | 完成态（配 icon+label） |
| `warning` | `#8A5A00` | 注意（配 icon+label） |
| `danger` | `#A0342C` | 危险/拒绝（配 icon+label） |
| `info` | `#4F5D6B` | 信息 |
| `stale` | `#6B655C` | 过期标记（配 icon+时间戳） |

### 1.4 A tokens — dark

| Role | Value |
|---|---|
| `canvas` | `#191817` |
| `surface` | `#211F1E` |
| `subtle-surface` | `#2A2826` |
| `ink` | `#ECEAE7` |
| `muted-ink` | `#A8A29B` |
| `divider` | `#3A3734` |
| `accent` | `#8FBEC7` |
| `success` | `#7FBC9A` |
| `warning` | `#D9A84E` |
| `danger` | `#E0786A` |
| `info` | `#9FB0BC` |
| `stale` | `#948C81` |

Dark mode 规则：不用纯黑+霓虹；filled button 在 dark 下用 `canvas` 色文本压 `accent` 底（8.75:1）；divider/warning/disabled 仍可辨。

## 2. Direction C — Campus Blueprint（推荐 · 2026-08-04 Develata 决策）

### 2.1 概念

冷峻白底 + 近黑 ink + 亮蓝 accent（黑白蓝）：明度拉开、留白充足、hairline 分隔，青春感来自**明度与蓝纯度**而非装饰堆叠——不使用渐变、glow、超大圆角或拟物。权威与校园亲和力共用同一套色：evidence spine、hairline、mono-identity 的 evidence 签名原样保留（它们是可信度来源，不是「老气」来源）。empty/onboarding 的亲和力由 editorial illustration 与口语 microcopy 表达，不引入第二套色。

适合：trust + campus-youth 全站统一；消除「权威屏冷、编辑屏暖」的两套语义混合成本。

### 2.2 C tokens — light

| Role | Value | 与 A 的差异 |
|---|---|---|
| `canvas` | `#FFFFFF` | 纯白（A 为暖灰纸色） |
| `surface` | `#FFFFFF` | — |
| `subtle-surface` | `#F7F8FA` | 冷灰 |
| `ink` | `#0D0D0F` | 近黑冷调 |
| `muted-ink` | `#5B6470` | 冷灰 |
| `divider` | `#E4E7EC` | 冷 hairline |
| `accent` | `#2563EB` | 亮蓝（原深青绿） |
| `success` | `#067A46` | 冷绿 |
| `warning` | `#B54708` | 琥珀（语义固有） |
| `danger` | `#D92D20` | 冷红 |
| `info` | `#475467` | 冷石板灰 |
| `stale` | `#667085` | 冷灰 |

### 2.3 C tokens — dark

| Role | Value |
|---|---|
| `canvas` | `#0D0D0F` |
| `surface` | `#141416` |
| `subtle-surface` | `#1B1B1F` |
| `ink` | `#F2F2F5` |
| `muted-ink` | `#A0A5B1` |
| `divider` | `#2A2A30` |
| `accent` | `#7AA5FF` |
| `success` | `#4CC38A` |
| `warning` | `#E5A13D` |
| `danger` | `#F07066` |
| `info` | `#98A2B3` |
| `stale` | `#8A8F9E` |

Dark 规则同 A：不用纯黑+霓虹；filled button 用 `canvas` 色文本压 `accent` 底；divider/warning/disabled 仍可辨。

### 2.4 C 的非色彩差异（coherent direction ≠ 换 palette）

- 控件 radius 上界 8px（仍在本卷 §5.2 声明的 6–10 envelope 内）；
- warn/ok 提示底色去饱和冷化，语义色只保留在左边条与图标；
- 不引入渐变、glow、超大圆角、mascot；拒绝的 AI defaults 清单对 C 同样适用；
- evidence spine / hairline / mono-identity 原样保留。

## 3. Direction B — Student Field Notes（已评估未采纳 · 备选）

### 3.1 概念

更温暖的 student-oriented system：paper-like neutral、稍软的 type rhythm、section markers 像学习笔记索引；仍保持 precise tables/diffs。不得 cute mascot、校园纪念品 cosplay、贴纸堆叠。

适合：Today 与 first-party tools 的 editorial 面。风险：authority-sensitive screens（approval/permission/audit）变轻佻——这正是只让 B 的 warmth 进入 empty/onboarding/illustration 的原因。

### 3.2 B tokens — light

| Role | Value | 与 A 的差异 |
|---|---|---|
| `canvas` | `#FBF7EF` | 更暖的纸色 |
| `surface` | `#FFFDF8` | 暖白 |
| `subtle-surface` | `#F4EEE1` | 便签底 |
| `ink` | `#221F1A` | 略柔 |
| `muted-ink` | `#6B6252` | 暖灰 |
| `divider` | `#E7DFCE` | 纸边线 |
| `accent` | `#8C4527` | 暖赭（marker 感） |
| `success` | `#3F6B4A` | — |
| `warning` | `#8F6400` | — |
| `danger` | `#A13328` | — |
| `info` | `#5C6650` | 橄榄 |
| `stale` | `#6B6152` | 已按实测修正（见 §7） |

### 3.3 B tokens — dark

| Role | Value |
|---|---|
| `canvas` | `#1C1915` |
| `surface` | `#26221D` |
| `subtle-surface` | `#2E2922` |
| `ink` | `#F0EBE2` |
| `muted-ink` | `#B0A893` |
| `divider` | `#423B31` |
| `accent` | `#E09A78` |
| `success` | `#8FBE9C` |
| `warning` | `#DDB265` |
| `danger` | `#E0826F` |
| `info` | `#A9B398` |
| `stale` | `#9C927E` |

### 3.4 B 的非色彩差异（coherent direction ≠ 换 palette）

- Type rhythm：section 标题配小号 marker label（如「步骤 · 03」式序号），行距略松；
- Section markers：笔记索引式左侧短粗 marker（2–3px 宽 accent 条）替代部分 heading 下划线；
- Empty/onboarding：允许 editorial illustration 与更口语的 microcopy；
- 仍保持：precise tables/diffs、hairline 优先、无贴纸/无 mascot。

## 4. 三方向比较与推荐（comparison memo · 2026-08-04 更新）

| 维度 | A Quiet Evidence | B Field Notes | C Campus Blueprint |
|---|---|---|---|
| 信任/权威场景（approval、permission、audit） | 强：低刺激、证据优先 | 风险：暖调 + 口语 microcopy 可能削弱严肃感 | 强：冷调清晰、证据优先、无暖调干扰 |
| Today/校园工具的亲和力 | 中：需 microcopy 补暖 | 强：笔记隐喻贴合学生心智 | 强：明度 + 蓝纯度自带青春感，无需第二套色 |
| 长阅读/diff/表格 | 强 | 中强 | 强：白底高对比最利于长读 |
| 与“非官方但可信”的定位 | 一致：克制即诚实 | 部分一致：需严格限定使用面 | 一致：黑白蓝克制 = 非官方但可信 |
| 实现复杂度 | 一套系统 | 两套语义规则，混合风险高 | 一套系统（最低） |

**推荐（PROPOSAL · 2026-08-04 Develata 决策翻转）**：**C 为 core system（全线 C）**。原「A 为 core、B 的 warmth 仅进入 empty states/onboarding/非权威 editorial 区」规则**取消**——empty/onboarding 的亲和力由 editorial illustration 与口语 microcopy 表达，不引入第二套色。Approval、permission、audit、diff、receipt 与 Today/first-party 一律 C。

A、B 保留为**已评估未采纳**的决策记录：A（Quiet Evidence，深青绿 + 暖灰纸色）风格稳重，但 2026-08-04 Develata 审视后判定缺少 campus 青春感；B（Field Notes，暖纸 + 暖赭）为候选暖方向，本轮选定冷调后不采纳。

拒绝的 AI defaults：purple-gradient SaaS hero、three-equal-card 首屏、nested cards、全站 glassmorphism、过度 pill、虚构 metrics/logos/endorsement——它们都在用视觉噪声替代证据，与本产品“可追溯”的 subject 冲突。

## 5. 共享 token envelope（三方向同构，仅色值不同）

### 5.1 Type（Chinese-first system sans）

| Token | 规格（px，起点） | 用途 |
|---|---|---|
| `display` | 32/40 | 极少数页面级标题 |
| `page-title` | 24/32 | 页标题 |
| `section` | 18/26 | 节标题 |
| `body` | 15–16/24 | 正文 |
| `utility` | 13/18 | 辅助/label |
| `mono-identity` | 12–13 mono | digest/ID 摘要 |

支持 text zoom，不锁死容器高度；authority-critical text 不 ellipsis（见 `06` 卷 §3）。

Type specimens（渲染校验用，Stage B 实测）：

```text
短中文：停用插件
长中文：更新方案已变化，之前的批准不再适用，请重新审查后再继续操作
英文 secondary：Review update plan · v0.4.0 → v0.5.0
混排：ustc.change-radar 需要先停用才能应用 v0.5.0 更新
Mono：sha256:9f2c…c41a（摘要可复制）
```

### 5.2 Measure / spacing / grid / radius / border / shadow

- Measure：reading content 约 64–72 汉字 visual measure；technical diff 可更宽；titles 允许两行。
- Spacing：4px base；8/12/16/24/32/48 semantic steps；section spacing > component spacing > inline gap。
- Grid：desktop 12-column 仅作 alignment；Android 4-column；content edges 与 actions 共享 anchor。
- Radius：small controls 6–10；bounded panels 12–16；不全站大圆角。S13 分组容器 `.chg` 按 small controls 档取值 8px（与按钮一致，视觉上为控件系而非 bounded panel）。
- Border/shadow：1px divider 优先；shadow 仅 overlay/floating control；content 不靠重阴影分层。
- Icons：单一 coherent outlined/filled 状态对，约 1.5–2px stroke；icon-only 必有 accessible name；不混 emoji（本 packet 线框中的符号仅为 ASCII 占位）。
- Materials：content 标准 surfaces；translucency 仅少量 navigation/overlay 且有 reduced-transparency fallback；不 glassify cards。
- Motion envelope：120–220ms state/layout feedback；spring 仅 direct manipulation；详见 `06` 卷 §4。

### 5.3 Light/dark 规则

Semantic tokens 独立校验（同一 role 在 light/dark 各自达标）；dark 不用纯黑+霓虹；状态色永远配 icon+label，color 不作唯一信号。

## 6. 应用示例（文本描述，Stage B 出视觉稿）

- **List/table**：行高 48–56px（舒适）/ 40px（紧凑）；metadata 列右对齐 mono utility；行 hover 仅 subtle-surface 变化。
- **Diff**：before 行 muted-ink + 删除线样式标记、after 行 ink；变化组标题 section 字级；Added/Removed/Expanded 用 icon+文字，不只靠红绿。
- **Approval**：页面级白 surface、单 primary（accent 底白字）；固定安全区不做视觉游戏；证据 disclosure 用 mono-identity。

## 7. 对比度实测记录（WCAG 相对亮度公式）

计算方法：`L = 0.2126R+0.7152G+0.0722B`（sRGB 分段线性化），`CR = (L1+0.05)/(L2+0.05)`。

**A-light**（阈值：normal ≥4.5，large ≥3.0）：

| Pair | CR | 判定 |
|---|---:|---|
| ink on canvas / surface | 16.34 / 17.20 | pass |
| muted-ink on canvas / surface | 6.83 / 7.18 | pass |
| accent on canvas / surface | 7.20 / 7.58 | pass（链接/文本按钮） |
| white on accent（filled button） | 7.58 | pass |
| white on danger | 6.95 | pass |
| success/warning/info/stale on canvas | 5.99 / 5.63 / 6.42 / 5.48 | pass |
| accent vs divider（focus 边界邻近对比） | 5.75 | pass（≥3.0） |

**A-dark**：ink 14.77（canvas）/13.67（surface）；muted 7.01/6.49；accent 8.75/8.10；canvas-on-accent filled button 8.75；canvas-on-danger 5.96；success 8.07、warning 8.16、danger 5.96、info 7.95、stale 5.34；muted on subtle 5.81。全部 pass。

**B-light**：ink 15.37/16.15；muted 5.62/5.91；accent 6.56/6.90；white-on-accent 7.01；white-on-danger 6.95；success 5.76、warning 4.92、info 5.66；**stale on subtle 初测 4.21 不达标 → 修正 `#7A7060`→`#6B6152`，复测 on subtle 5.25 / on canvas 5.69，pass**。

**B-dark**：ink 14.75/13.31；muted 7.39/6.67；accent 7.56/6.82；canvas-on-accent 7.56；canvas-on-danger 6.33；success 8.35、warning 8.86、danger 6.33、info 8.00、stale 5.69；muted on subtle 6.09。全部 pass。

**C-light**（2026-08-04 新增，Direction C；阈值：normal ≥4.5，large ≥3.0）：

| Pair | CR | 判定 |
|---|---:|---|
| ink on canvas / surface | 19.42 / 19.42 | pass |
| muted-ink on canvas / subtle | 6.00 / 5.64 | pass |
| accent on canvas / surface | 5.17 / 5.17 | pass（链接/文本按钮） |
| white on accent（filled button） | 5.17 | pass |
| white on danger | 4.83 | pass |
| success / warning / info / stale on canvas | 5.41 / 5.43 / 7.69 / 4.97 | pass |
| success / warning / info / stale on subtle | 5.09 / 5.11 / 7.23 / 4.68 | pass |
| accent vs divider（focus 边界邻近对比） | 4.17 | pass（≥3.0） |
| muted on divider（disabled 灰态文字） | 4.84 | pass |
| divider on canvas（disabled 背景/边框） | 1.24 | WCAG 1.4.3 disabled 豁免；文字本身 4.84 达标 |

disabled 灰态（2026-08-06 round-8 起）：filled 按钮 disabled = divider 底 + muted 字（上表 4.84）；`.sec` disabled = muted 字 + divider 边框（muted on canvas 6.00）。弃 opacity 机制；`.badge.attn`（attention 态）背景同 `.warn` 底 `#F9F5EF`。

**C-dark**：ink 17.38/16.47；muted 7.87/6.96；accent 8.00/7.58；canvas-on-accent 8.00；canvas-on-danger 6.67；success 8.77、warning 8.78、danger 6.67、info 7.54、stale 6.01；muted on subtle 6.96。全部 pass。

`ASSUMPTION`：以上为纯色对计算；真实渲染（抗锯齿、字号、字重、translucency 叠加）需 Stage B 在 browser/device 复测。translucency 面必须提供 reduced-transparency fallback 后的纯色等效值并重新达标。

## 8. 本卷 UNRESOLVED / ASSUMPTION

- `ASSUMPTION`：全部数值为起点；final values 待 browser/device 测试（含 200% zoom、Windows/Android 字体回退）。
- `ASSUMPTION`：icon set 未选型；选型标准为 single coherent outlined/filled 对、1.5–2px stroke、完整 accessible name 覆盖。
- 不存在 UNRESOLVED domain 语义由本卷承载；本卷不定义任何 status 语义，只定义呈现。
