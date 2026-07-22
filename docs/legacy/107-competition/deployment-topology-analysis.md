# Deployment Topology 比较：统一性、难度与架构洁净度

- 状态：**架构分析，尚未批准实现**
- 更新时间：2026-07-21
- 讨论对象：USTC 个人校园 Agent 平台

## 1. 先定义“统一性”

统一性不等于所有代码运行在一台机器。一个分布式系统仍可高度统一，只要以下 authority 唯一：

1. **Identity authority**：谁是当前用户、设备和 publisher；
2. **State authority**：对话、记忆、订阅、plugin grant 与任务状态由谁裁定；
3. **Source authority**：校园来源版本、抓取时间、冲突和 provenance 的真相源；
4. **Plugin authority**：plugin ID、版本、publisher、权限、兼容范围与撤销状态；
5. **Audit authority**：谁在何时调用了什么 capability；
6. **Protocol authority**：clients、companions 和 plugins 遵守哪个版本化合同。

执行可以分散，但 authority 不应分裂。最关键的区别是：

```text
多个 execution locations   ≠ 多个 state authorities
```

## 2. 方案 A：统一多租户中央服务

```text
Web / Android / future clients
              │
              ▼
Central multi-tenant backend
├── shared campus graph
├── user state
├── plugin registry/grants
├── central plugin runners
└── audit/provenance
```

### 统一性

**最高。**

- 所有 client 看到同一状态；
- 校园来源和变更只抓取、解析、校验一次；
- plugin 版本、权限和撤销集中管理；
- Android 与 Web 不需要寻找每个用户的私人服务器；
- G1–G3 能共享同一 campus graph 与 temporal layer。

### 实现难度

**核心开发难度高，但产品路径直接。**

必须从首版处理：

- multi-tenant data isolation；
- auth/session/device；
- per-user secret 与 grants；
- plugin runner isolation；
- quota/rate limit；
- audit；
- migration 和 backup。

但它避免了每用户 TLS、域名、NAT、升级和 Android endpoint 配置。

### 架构洁净度

**高，前提是多租户是本体而不是后补。**

核心对象从一开始携带 `TenantId/UserId`，所有 state change 经过统一 command/API；插件不能直接访问数据库。

### 主要缺点

- 承担所有用户隐私和凭据风险；
- 中央故障影响全部用户；
- 用户本地文件、应用和设备能力难以直接接入；
- 任意 plugin code 不能直接信任，必须建设隔离执行面；
- 长期运维负担集中。

### 对本项目的适配

G1–G3、marketplace、跨 Web/Android 同步天然适合中央服务；但不能单独满足“本地 plugin”和私人资源访问。

## 3. 方案 B：每用户完整自托管

```text
User A clients → User A Docker Compose
User B clients → User B Docker Compose
User C clients → User C Docker Compose
```

### 统一性

**单实例内部高，校园整体低。**

- 每个用户有独立真相源；
- campus graph、抓取缓存和规则版本在不同实例间漂移；
- plugin 撤销、安全更新和 schema migration 无法集中保证；
- “官方认证”只能表示 registry metadata，不能保证用户实际运行版本。

### 实现难度

**表面较低，整体交付难度高。**

服务端可以先按 single-tenant 编写，但产品必须处理：

- 每用户安装、备份、升级和回滚；
- TLS、域名或 VPN；
- NAT 与 Android 外网访问；
- 用户端数据库和对象存储；
- 异构 CPU/OS；
- 错误诊断与版本碎片化。

把运维工作转给用户并没有消除复杂度，只是把它从代码仓库移到了交付面。

### 架构洁净度

**若产品本体就是 self-hosted personal agent，则可很干净；对当前校园公共平台愿景则不干净。**

为了支持官方 marketplace、统一变更监测和跨用户来源校验，最终仍会补一个 central control plane，系统又回到 hybrid。

### 主要优点

- 用户数据与凭据留在自己环境；
- 用户可直接访问本地文件和服务；
- 单个实例故障不影响其他用户；
- 高级用户拥有最大控制权。

### 主要缺点

- 对普通学生门槛高；
- Android 连接私人实例困难；
- 版本与安全补丁碎片化；
- 大量重复抓取 USTC 网站；
- 不利于比赛演示“统一部署、个人化使用”。

### 对本项目的适配

不建议作为比赛主 topology；可在赛后作为发行能力研究。

## 4. 方案 C：中央 authority + 本地/远程执行面（Hybrid-lite）

```text
Web / Android clients
          │
          ▼
Central backend authority
├── identity/state/campus graph
├── plugin registry/grants
├── audit/provenance
└── capability broker
          │
          ├── official central runners
          ├── user-hosted MCP/HTTP
          └── paired local companion
```

### 统一性

**高。**

中央服务仍是唯一 authority；remote/local plugin 只是 capability executor。它们：

- 不拥有平台主状态；
- 不直接写中央数据库；
- 通过版本化 capability protocol 接收调用；
- 返回结构化结果和 evidence；
- 由中央服务记录 grants、调用和结果摘要。

### 实现难度

**中央方案之上增加一层受控 routing，难度中高。**

新增问题：

- device pairing；
- outbound persistent session；
- online/offline state；
- end-to-end identity；
- remote endpoint SSRF 和凭据；
- call cancellation、timeout 与 replay；
- 用户本地确认；
- result-size 与 file transfer。

但这些复杂度与“用户插件可在本地/自托管运行”的产品承诺直接对应，不是无谓抽象。

### 架构洁净度

**最高潜力，前提是控制面与执行面严格分离。**

建议核心对象：

```text
Plugin
PluginVersion
Capability
Grant
ExecutionBinding
Device
Invocation
AuditEvent
```

其中 `ExecutionBinding` 只决定调用去哪里：

```text
CentralRunner | RemoteMcp | RemoteHttp | LocalCompanion
```

它不改变 plugin 的 capability schema，也不改变业务 state authority。

### 主要优点

- 保留中央产品体验、marketplace 与统一校园图谱；
- 支持用户本地文件、私人服务和自托管 plugin；
- Android/Web 只连接中央服务；
- 本地设备无需开放公网入站端口；
- execution runtime 可替换，而核心不变。

### 主要缺点

- permission model 与 broker 必须早定；
- 中央服务仍承担元数据、审计和部分隐私责任；
- companion 离线、plugin 超时和设备撤销必须定义；
- 若首版同时实现所有执行面，会显著扩大范围。

### 对本项目的适配

**最符合长期愿景，但比赛首版必须分阶段实现。**

## 5. 方案 D：中央版与完整自托管版平级并存（Full Hybrid）

```text
Central complete platform
          +
Per-user complete platform
          +
State/plugin synchronization
```

### 统一性

**最低风险点不在代码复用，而在双 authority。**

必须回答：

- 用户对话和记忆以哪边为准；
- plugin grant 是否同步；
- source graph 冲突如何解决；
- 用户从 central 切换到 self-host 时任务如何迁移；
- central 撤销恶意 plugin 后离线实例如何处理；
- Android 同时连接哪个 endpoint。

### 实现难度

**最高。**

它实际上包含：

- multi-tenant SaaS；
- single-tenant distribution；
- data migration；
- bidirectional sync 或 export/import；
- 两套运维与兼容矩阵。

### 架构洁净度

**低，除非项目本体从一开始就是 federated personal-agent network。**

当前 G1–G3 与 plugin marketplace 不要求 federation。为未来“也许自托管”提前引入双 authority，违反保守与本体优先原则。

### 对本项目的适配

不建议进入比赛骨架。

## 6. 比较矩阵

| 方案 | 状态统一性 | 首版实现 | 长期运维 | 移动端体验 | 本地 plugin | 架构洁净度 |
|---|---|---|---|---|---|---|
| A 中央多租户 | 很高 | 中高 | 集中且重 | 最自然 | 较弱 | 高 |
| B 每用户自托管 | 单实例高、整体低 | 表面中、交付高 | 分散且难控 | 较差 | 最自然 | 与当前愿景不一致 |
| C 中央 authority + 多执行面 | 很高 | 中高，需分阶段 | 可控 | 自然 | 强 | **最高潜力** |
| D 两套完整平台平级 | 易 split-brain | 最高 | 最高 | 复杂 | 强 | 低 |

## 7. 推荐方案

### 长期骨架：方案 C

采用：

> **中央 control/data plane + 可替换的 plugin execution planes。**

唯一 authority 放在中央：

- identity；
- user-visible state；
- campus graph；
- plugin registry；
- grants；
- audit/provenance。

执行位置可变：

- 官方 plugin → central isolated runner；
- 用户服务器 → remote MCP/HTTP；
- 用户设备 → paired local companion。

### 比赛 MVP：C 的收缩版

不要一次实现全部 execution planes。建议证明：

1. central multi-tenant backend；
2. Web + Android 访问同一账户状态；
3. 一个 GitHub-catalog `PluginPackage` 可发现、授权、安装和撤销；
4. 一个 remote MCP/HTTP plugin 可由用户绑定并受控调用；
5. plugin 调用都有权限、超时和 audit；
6. local companion 只做协议/架构预留，或作为 stretch goal 实现一个最小闭环。

这已经足够证明 plugin 不是硬编码 feature；无需在比赛前完成任意代码沙盒和完整 federated self-hosting。

## 8. Single-user Docker Compose 的正确定位

仍可让同一 server binary 以 single-tenant 配置运行，用于：

- 本地开发；
- demo；
- 离线测试；
- 小规模私有部署。

但它只是**deployment profile**，不是第二套 state authority 或平级产品。代码中仍保留 tenant-scoped domain model，避免未来从 single-user 迁移到 multi-user 时重写数据模型。

## 9. 对 Dioxus 的影响

该 recommendation 与 Dioxus UI / 独立 backend 的边界一致：

- Web/Android/Desktop 只是 clients；
- 所有 clients 通过稳定 API 访问中央 authority；
- local companion 是 Rust service，不嵌入 UI framework；
- plugin protocol 与 Dioxus server functions 无关；
- 更换客户端 renderer 不改变 topology。

## 10. 最终判断

- **中央多租户作为唯一系统**：可行、统一，但不能完整满足本地 plugin 愿景。
- **每用户完整自托管**：不适合普通学生与 Android 主入口，不推荐作为比赛主线。
- **中央 authority + 多执行面**：统一性、产品价值与长期替换性最佳，推荐。
- **中央版和自托管版平级并存**：复杂度高且容易双 authority，不推荐。

简言之：

> **统一状态，分散执行；一个大脑，多个受控的手。**

## 11. Develata-side 架构候选收敛（2026-07-21）

Develata 本轮明确以下架构候选；尚不等同于团队共识：

- memory、Agent runtime、tasks 与 `PluginPackage` installations/grants 均由 USTC central authority 保存；default first-party Plugin 引用的 MCP/Skill/shared services 由受控 execution plane 提供；
- AI/MCP execution adapter 支持官方中央配置与用户自定义 central profile；client/device relay 只保留未来替换边界；
- public GitHub catalog manifests 管理 PluginPackage/component 发布、review、version 与 digest；PostgreSQL 管理 user installation/grant/runtime state；
- schema/catalog/source 在 GitHub public 开源，Market runtime/admin/secrets 只在 USTC 内网；自部署者部署完整 central stack，而非与官方实例形成平级同步 authority。

该方向是方案 C 的具体化，而不是方案 D。详细边界见 [`central-agent-client-relay-marketplace.md`](central-agent-client-relay-marketplace.md)。

AI provider 的 MVP 后续进一步收缩为两种 central execution mode：平台管理的 `OfficialCentral` 与用户明确上传 URL/key 的 `UserCloud`。`UserDeviceRelay` / `UserRemoteRelay` 只保留 executor 边界，不进入 Demo。详见 [`model-provider-policy.md`](model-provider-policy.md)。

MCP source 进一步分为 `Official/VerifiedMarket | UserRemote | UserHostedPrivate`；用户 remote endpoint/credential 由中央加密保存、discover、授权和调用，private hosted artifact 走 dedicated on-demand runtime；device/always-on relay 不进入 Demo。详见 [`mcp-binding-policy.md`](mcp-binding-policy.md) 与 [`mcp-market-hosting-policy.md`](mcp-market-hosting-policy.md)。
