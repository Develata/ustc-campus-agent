# 09 — Onboarding, Connectivity and Settings

> **Illustrative / No live backend** — 本卷全部线框、示例数据与状态为设计样例；无真实服务器、无真实账号、无真实连接。
> Packet: `m80-default-v0` · Status: `Reviewed` · Source: `2f4de29032560ff3e13d9994b33a3aff14243f44` / tree `53e266c47fdb07d50a734faa24bb11ac4bc5527d`
> 覆盖 required surfaces：First run / server connectivity（brief §6.1）· Settings/accessibility（brief §6.18）· client/system semantic intents（brief §9.5）
> 本卷为独立 review 后补卷（2026-08-02 第二轮），补齐首轮缺失面。

## 1. 本卷 authority 前提

- `TRACKED FACT`：M80 `client-core` 的 required common failure classes 含 `InvalidEndpoint`、`AuthenticationRequired`、`IncompatibleProtocol / UpgradeRequired`、`TransportUnavailable`、`TimeoutOutcomeUnknown` 等（`docs/contracts/client-shell.md` §5）；compatibility envelope / `UpgradeRequired` 是 M10-produced value（同文件 :116）；server 在 application dispatch 前做 typed unsupported-version rejection（同文件 :343）。
- `TRACKED FACT`：M80 target ports 含 `ClientAuthPort`、`SecureSessionPort`、`ServerEndpointPort`、`LocalArchivePort`、`PlatformInfo`、`ExternalNavigation`（`docs/plan/modules/80-dioxus-multi-client.md:88-100`）；`LocalArchivePort` 是 optional user-controlled export，distinct from durable server memory（`docs/contracts/client-shell.md:249`）；diagnostics 仅 redacted（同文件 :201）。
- `TRACKED FACT`：本项目是学生竞赛项目，**不是 USTC 官方服务**（`AGENTS.md` Product boundary）；public transition guard 要求 non-official disclaimers。
- `PROPOSAL`：连接与账号状态是 shell 级事实，不是页面级装饰；blocking 问题用 full-screen gate 或 persistent banner，正常时退居 top-bar utility（与 `01` 卷 §3 一致）。
- `PROPOSAL`：本卷不出现「已连接 · 一切正常」式 readiness 词汇；只呈现 server/client-projected 的 compatibility 与 session 事实 + user-safe reason。

## 2. First run / server connectivity（brief §6.1，PROPOSAL，high-fidelity）

### 2.1 旅程总览

```text
首次启动
  → ① 非官方声明（disclaimer）
  → ② 服务器身份（server identity：endpoint + server 自报信息）
  → ③ 登录（sign-in，经 admitted auth port）
  → ④ 隐私与数据（privacy stepper 末步）
  → ⑤ 完成 → Today（empty 态，见 `01` 卷 §5.3）
任一步可「取消/移除」（后果如实）；connection 状态族见 §3。
```

- `PROPOSAL`：stepper 恰好 4 步 + 完成页；每步一个 primary；不合并 disclaimer 与 privacy；不在 onboarding 内推销插件。
- `PROPOSAL`： disclaimer、privacy 属 fixed safety zone 内容，不受 layout customization 影响（`04` 卷 §2.2）。

### 2.2 Step ① 非官方声明

```text
┌──────────────────────────────────────────────────┐
│ 欢迎使用 USTC Campus Agent            步骤 1/4   │
│ ────────────────────────────────────────────────│
│ 这是一个学生竞赛项目，不是中国科学技术大学的      │
│ 官方服务。学校名称仅用于描述适用场景。            │
│                                                │
│  · 数据保存在你连接的服务器上                    │
│  · 插件需要你的逐项授权才会行动                  │
│  · 你可以随时断开并移除连接                      │
│                                                │
│ □ 我已理解这不是学校官方服务                     │
│                          [退出]    [继续]        │  ← 勾选前不可用
└──────────────────────────────────────────────────┘
```

- `PROPOSAL`：显式勾选（非默认选中）后才可继续；declined → 停留在本步并说明退出方式，无暗算继续。

### 2.3 Step ② 服务器身份

```text
┌──────────────────────────────────────────────────┐
│ 连接服务器                            步骤 2/4   │
│ ────────────────────────────────────────────────│
│ 服务器地址                                        │
│  [ campus-agent.example.edu……          ]        │  ← illustrative
│                                                │
│ 服务器自报信息（连接后显示，server-projected）：  │
│  · 名称/运营者声明：……                           │
│  · 协议兼容性：由握手判定（不预先承诺）           │
│  · TLS 状态：连接建立后如实显示                  │
│                                                │
│                    [上一步]  [连接并继续]        │
└──────────────────────────────────────────────────┘
```

- 「连接并继续」的 pending：按钮 pending + 禁重复；失败按 §3 状态族分类呈现（endpoint invalid / TLS 失败 / transport unavailable / incompatible），**不**把「连不上」泛化为「出了点问题」。
- `PROPOSAL`：server 自报信息原样呈现并标注来源（server-declared）；client 不为其真实性背书。

### 2.4 Step ③ 登录

```text
┌──────────────────────────────────────────────────┐
│ 登录                                  步骤 3/4   │
│ ────────────────────────────────────────────────│
│ （经服务器提供的 admitted 登录方式；本客户端不    │
│   代收密码明文转交第三方。）                      │
│  [ 使用校园统一认证登录（illustrative）]          │
│                                                │
│ 状态族：等待用户 / pending / AuthenticationRequired│
│ / 取消 / 失败（safe reason + 重试）              │
│                    [上一步]        [稍后登录]    │
└──────────────────────────────────────────────────┘
```

- `PROPOSAL`：「稍后登录」进入受限只读模式是 `UNRESOLVED`（Q11：匿名/受限 session 是否 admitted 未定，owner M00/M10）；若 server 不投影该选项，按钮不出现。
- 凭证存取只经 `SecureSessionPort`/`ClientAuthPort`；UI 永不显示 token/secret（`06` 卷与 `02` 卷 §4.3 SecretRef 纪律一致）。

### 2.5 Step ④ 隐私与数据

```text
┌──────────────────────────────────────────────────┐
│ 隐私与数据                            步骤 4/4   │
│ ────────────────────────────────────────────────│
│  · 插件的每项能力都需要你逐项批准                 │
│  · 动态记录对你可见，可导出（见「设置 · 诊断」）  │
│  · 本地缓存的内容标注最后同步时间                 │
│  · 移除连接会清除本设备上的会话与缓存             │
│                          [上一步]    [完成]      │
└──────────────────────────────────────────────────┘
```

完成页不庆祝、不伪造示例数据；落地 Today empty 态（`01` 卷 §5.3），一个 next step。

### 2.6 非首跑的连接入口

- Today blocking banner（connection/compatibility/reauth）→ 对应 step 的局部重现，**不重放整个 stepper**。
- Settings → Account & server（§4.2）提供「检查连接」「重新登录」「移除此连接」。

## 3. Connection / compatibility 状态族（PROPOSAL）

| State | 呈现 | Allowed recovery | Client must not |
|---|---|---|---|
| 正常 | top-bar utility 小图标 + label；不占内容中心 | — | 显示「健康/一切正常」词汇 |
| Endpoint invalid | step 内 field-level 错误 + safe reason | 修正地址重试 | 猜测协议/端口补齐 |
| TLS 失败 | blocking 说明 + 证书类安全提示（不教用户绕过） | 返回修正 / 移除连接 | 提供「忽略证书继续」 |
| Transport unavailable | persistent banner + last-sync 时间戳 | 自动重连 + 手动重试（幂等 read） | 声称 mutation 成功 |
| AuthenticationRequired / reauth | inline banner「需要重新登录」+ [重新登录] | 重新登录流程 | 用旧 session 静默重试 mutation |
| IncompatibleProtocol / UpgradeRequired | **全屏 upgrade gate**（见下） | 更新 client 后重连 | 向临近版本 dispatch（client-shell.md:343） |
| TimeoutOutcomeUnknown | 「正在核对结果」+ reconcile by correlation identity | 重试核对 | 盲重试 mutation |
| Server readiness 缺口 | banner 标注 server-projected 缺口项 | 等待/联系运营者 | 本地推断 readiness |

**UpgradeRequired 全屏 gate**：

```text
┌──────────────────────────────────────────────────┐
│ 需要更新客户端                                    │
│ 此服务器要求的协议版本高于当前客户端支持。        │
│  · 当前客户端：…（build/target/protocol）        │
│  · 服务器要求：…（server-projected）             │
│ [ 更新客户端 ]        [ 移除此连接 ]             │
│ ※ 更新前可查看只读的本地缓存内容（标注最后同步）  │
└──────────────────────────────────────────────────┘
```

- `PROPOSAL`：gate 内保留「移除此连接」出口；upgrade 动作经 `ExternalNavigation`/平台商店（Android）或下载页（Web），不经 client 自更新承诺。

## 4. Settings（brief §6.18，PROPOSAL，high-fidelity）

### 4.1 结构（与 `01` 卷 §2.1 IA 一致）

```text
Settings
├── Account & server   账号与服务器（连接、登录、移除连接）
├── Compatibility      兼容性（client/server 版本事实、upgrade required）
├── Appearance         外观（theme/density/layout customization → `04` 卷 §2）
├── Accessibility      无障碍（系统偏好说明 + 应用内可选项）
└── Diagnostics        诊断（safe classes only + user-controlled export）
```

- Desktop：Settings 为独立页 + 左 section nav；Android：account sheet 入口 → full-page sections。
- `PROPOSAL`：Settings 不提供 raw secret 表单、不显示 provider 凭据；provider/server settings 的用户可见面 `UNRESOLVED`（Q7）。

### 4.2 Account & server

```text
┌──────────────────────────────────────────────────┐
│ 账号与服务器                                      │
│ ────────────────────────────────────────────────│
│ 服务器                                            │
│  · 地址：campus-agent.example…（illustrative）   │
│  · 服务器自报信息：……（server-declared）         │
│  · 连接状态：server/client-projected 状态 + 时间 │
│  [检查连接]  [重新登录]                          │
│ ────────────────────────────────────────────────│
│ 账号                                              │
│  · 当前用户：……（server-projected user context） │
│  [退出登录]                                      │
│ ────────────────────────────────────────────────│
│ Danger zone                                       │
│  [移除此连接]  ← destructive typed confirmation  │
└──────────────────────────────────────────────────┘
```

**移除此连接**（destructive，consequence 页先行）：

```text
┌──────────────────────────────────────────────────┐
│ 移除此连接？                                      │
│ 将会：                                            │
│  · 清除本设备上的会话凭证与本地缓存               │
│  · 停止接收此服务器的事件                         │
│ 不会：                                            │
│  · 删除服务器上的安装、授权与历史（归服务器所有） │
│ 之后需要重新连接并登录才能继续使用。              │
│ 输入服务器名称以确认：[____________]              │
│                    [取消]    [确认移除]          │
└──────────────────────────────────────────────────┘
```

- `PROPOSAL`：typed confirmation（输入名称或等效）；结果仅由本地凭证清除完成 + server session 终止结果共同呈现；server 侧数据所有权如实说明，不承诺「已删除所有数据」。

### 4.3 Compatibility

```text
│ 兼容性
│  · 客户端：build … · target … · protocol …
│  · 服务器支持范围：…（server-declared）
│  · 状态：兼容 / 需要更新（UpgradeRequired 时全屏 gate + 本页同因说明）
```

- 每项标注来源（client-reported / server-declared）；不一致时以 server rejection 事实为准并说明。

### 4.4 Accessibility（应用内面）

- 呈现系统偏好遵循情况（reduced motion、contrast、font scale）+ 应用内可选项（density 已在 Appearance；reader 行为说明）。
- `PROPOSAL`：无障碍设置不提供「关闭无障碍支持」类选项；只提供增强项。系统级开关引导至 OS 设置（`ExternalNavigation`）。

### 4.5 Diagnostics

```text
│ 诊断（仅 safe classes）
│  · 最近错误：safe code + 时间 + 所属 surface（无 stack、无 secret）
│  · 连接日志摘要：reconnect/resync 次数与时间
│  [导出诊断与动态记录]  ← 经 LocalArchivePort；用户选择去向
│  ※ 导出内容经 redaction；digest 截断显示；不含明文凭证
```

- `TRACKED FACT`：export 走 user-controlled `LocalArchivePort`，与 durable server memory 区分（client-shell.md:249）；diagnostics 仅 redacted（client-shell.md:201）。

### 4.6 Settings 状态族

loading（section skeleton）、offline（只读 + last-sync）、save pending/conflict（appearance/layout，见 `04` 卷 §2.4）、remove-connection pending/unknown（reconcile）、upgrade required（同 §3 gate）。

## 5. Semantic needs（全部 `PROPOSED_SEMANTIC_INTENT`）

| Need | Intent | Owner | 说明 |
|---|---|---|---|
| 客户端兼容性自报与判定结果 | `GetClientCompatibility` | M10 carrier + M80 client-core | 输入 client build/target/protocol identity；输出 typed compatibility outcome（含 UpgradeRequired）；与 client-shell.md:336 identity 纪律一致 |
| 服务器可见性/自报信息/支持范围 | `GetServerReadiness` | M10/M00 组装面 | 输出 server-declared 信息 + 支持协议范围 + 缺口项；client 不推断 readiness |
| 当前用户上下文 | `GetCurrentUserContext` | M00/M10 | 输出 session 用户摘要 + session 状态（含 AuthenticationRequired）；不含凭证 |
| 建立连接 | `ConnectServer` | M80 client-core 端口 | 经 ServerEndpointPort；typed failure 按 §3 |
| 登录/登出 | `SignIn` / `SignOut` | M00 session via M10 | 经 ClientAuthPort/SecureSessionPort |
| 移除连接 | `RemoveServerConnection` | M80 client-core（本地）+ M00（server session 终止如 admitted） | destructive confirmation class server-owned |
| 诊断与导出 | `GetDiagnostics` / `ExportLocalArchive` | M80 client-core / LocalArchivePort | redacted only；用户控制去向 |

M80 never calculate：compatibility 判定、server readiness、session 有效性、TLS 信任决策。

## 6. 本卷 UNRESOLVED 汇总

- Q4 carrier；Q7 provider/server settings 可见面。
- **Q11（新增）**：匿名/受限只读 session 是否 admitted（owner M00/M10）；决定「稍后登录」是否存在。
- `ASSUMPTION`：onboarding step 数与顺序为设计起点；真实 auth 方式（统一认证/账号密码/邀请制）待 M00/M10 session contract。
