# 智能体赛道架构讨论：Plugin 模型与 Dioxus 边界

- 状态：**Rust/Dioxus 候选；Develata-side Market/PluginPackage 本体已明确，MCP hosting 与 Agent runtime 仍待批准/spike**
- 更新时间：2026-07-21

## 1. 已确认的产品方向

### PluginPackage Market 与 execution classes

#### Public reviewed PluginPackages

- `PluginPackage` 组合 MCP、Skill、ControlledCLI 与 SharedServiceBinding；
- catalog/schema/source 发布到 GitHub public；
- 经 manifest、publisher、license、artifact 与 runtime policy 审核；
- 在类似 ChatGPT plugin marketplace 的界面中发现；
- 用户安装 exact version；三个 read-only FirstPartySystemPlugin 可在 account bootstrap 时默认安装/启用。

#### User-bound execution components

- 用户可连接 remote MCP/HTTP component；
- 可在用户自己的服务器运行；
- 也可通过本地 companion 在用户设备运行；
- 也可提交 approved immutable MCP artifact，由平台为该用户 dedicated on-demand 托管；
- 平台只通过受控协议或 artifact admission/runtime sandbox 执行，不把任意 config command 当作服务器 shell 执行。

`PluginPackage` 是安装本体；remote/shared/dedicated/cold/warm 是其 components 的 execution policy，不是第二套安装对象。

### 技术与目标端

- 核心技术方向：Rust；
- 首期服务端：Linux + Docker Compose；
- 首期客户端：Web + Android；
- 未来候选：iOS、Windows/Linux/macOS Desktop。

## 2. Dioxus 当前官方能力核对

截至 2026-07-21：

- GitHub 最新 stable release：`v0.7.9`；
- `v0.8.0-alpha.0` 是 prerelease，包含 breaking changes，不适合比赛主线；
- 官方 0.7 文档支持 Web、Desktop、Android、iOS 与 Fullstack；
- Dioxus Fullstack 明确由至少两个 binary 构成：client 与 server；
- 官方也支持将 frontend/backend 拆成独立 crates 和独立 server binary；
- Web 是 Dioxus 最成熟的目标；
- Desktop 0.7 默认使用 system WebView，Rust 代码原生运行，底层基于 `wry`；
- Mobile 0.7 将 WebView 作为稳态渲染路径，WGPU/native renderer 仍属实验方向；当前不提供 Android 原生 widgets/animations；
- `dx bundle` 可生成 Web、Desktop 与 Mobile artifacts，但 Desktop 需在对应 native host 构建；
- iOS 需要 Xcode/Apple toolchain，移动分发还涉及 signing；Dioxus 不替项目完成完整签名流程。

官方来源：

- <https://dioxuslabs.com/learn/0.7/essentials/fullstack/project_setup>
- <https://dioxuslabs.com/learn/0.7/guides/platforms/mobile>
- <https://dioxuslabs.com/learn/0.7/guides/platforms/desktop>
- <https://dioxuslabs.com/learn/0.7/tutorial/bundle>
- <https://github.com/DioxusLabs/dioxus/releases/tag/v0.7.9>
- <https://github.com/DioxusLabs/dioxus/releases/tag/v0.8.0-alpha.0>

## 3. 推荐判断

### Yes：把 Dioxus 作为共享 UI 候选

适合的原因：

- Web + Android 是当前正式目标；
- Desktop 是自然延伸；
- 产品主要是信息、对话、卡片、流程、权限和 plugin marketplace，WebView UI 足够；
- 团队希望 Rust-first，可共享类型、状态和部分 UI；
- 比赛原型可以避开相机、蓝牙、复杂后台任务等深度 native 能力。

### No：不把 Dioxus Fullstack server functions 当作系统公共骨架

理由：

1. **Backend authority 生命周期更长**：校园来源、用户状态、插件权限、审计和任务状态不能依赖 UI framework。
2. **多客户端需要稳定协议**：Web、Android、本地 companion、MCP/HTTP plugins 与未来 Desktop 都应通过版本化 API 进入系统。
3. **Plugin 生态要求外部兼容**：第三方 plugin 不应编译进 Dioxus app 或绑定 server-function internals。
4. **独立部署与升级**：Docker server、Android APK、Web client 和 local companion 应允许独立升级，并有兼容矩阵。
5. **替换性**：若 Dioxus mobile 暴露平台限制，Android UI 可替换，而 domain/backend 不动。

Dioxus Fullstack 可以用于 spike 或内部页面，但不应成为唯一 API contract。

## 4. 候选骨架

```text
Clients
├── Web UI                 (Dioxus Web candidate)
├── Android App            (Dioxus Mobile candidate)
├── Future Desktop         (Dioxus Desktop / replaceable)
└── Local Companion        (Rust native service)
        │
        ▼
Versioned API / Realtime Protocol
        │
        ▼
Campus Agent Backend       (Rust, independent server binary)
├── Identity / Session
├── Campus Source Registry
├── Temporal + Conflict Layer
├── Campus / Opportunity Graph
├── Agent Orchestrator
├── Consent + Authorization
├── PluginPackage / Capability Broker
├── Audit / Provenance
└── Evaluation / Observability
        │
        ├── MCP Gateway + Runtime Controller
        ├── Official/Verified Hosted MCPs
        ├── User Dedicated On-demand MCPs
        ├── Remote MCP Components
        ├── Remote HTTP Components
        └── Paired Local Companion Components
```

## 5. 建议的 Rust workspace 边界

以下只表达 ownership，不是最终目录冻结：

```text
apps/
├── server                 # 独立 backend authority
├── web-mobile-ui          # Dioxus UI shell / shared components
└── local-companion        # 用户设备上的受控 plugin host

crates/
├── domain                 # 核心对象与不变量
├── api-contract           # versioned request/response/events
├── campus-source          # 来源、版本、provenance
├── campus-graph           # 流程、机会、资格、依赖
├── plugin-protocol        # manifest/capability/lifecycle
├── authorization          # user/plugin capability grants
└── evaluation             # deterministic fixtures/metrics
```

UI 只负责呈现和 typed intent；不得直接决定来源权威性、插件权限、冲突裁决或长期状态。

## 6. Public PluginPackage Registry 合同

每个 public PluginPackage 至少需要：

- stable plugin ID；
- publisher identity；
- version 与 host API compatibility range；
- GitHub public source/catalog revision；
- component list 与 exact versions/digests；
- artifact digest/signature；
- license 与上游来源；
- capability/permission manifest；
- 网络与数据访问声明；
- tools/resources/prompts 或 HTTP capability schema；
- 安装、升级、禁用、撤销和卸载生命周期；
- health check 与用户可见错误；
- audit events。

普通/community Plugin 的安装流程应为：

```text
Discover
→ inspect publisher/version/permissions
→ user grants scopes
→ install/attach
→ health check
→ enable
```

三个 signed `FirstPartySystemPlugin` 采用 account bootstrap default install/enable；其 manifest 只能包含 operator Capability Registry 中 `auto_grant_eligible=true` 的精确 read capabilities，因此可授予其全部声明。新增 capability、risk/data class 变化或 mutation permission 必须重新做 architecture review，不能沿普通 patch 静默继承。

## 7. 用户自托管 Plugin 合同

### Remote MCP / HTTP

平台保存 endpoint 与最小凭据引用，执行：

- transport allowlist；
- SSRF / localhost / metadata / campus-internal boundary；
- per-tool scopes；
- timeout、rate limit 与 output-size limit；
- schema validation；
- audit trail；
- disable/revoke。

### Platform-hosted private MCP

平台可以为用户托管 local/package MCP，但首版只接受 digest-pinned OCI artifact，并要求：

- `DedicatedUser + OnDemand`；
- immutable artifact、publisher/source/license provenance；
- no arbitrary command、host mount、Docker socket 或 privileged mode；
- non-root、read-only filesystem、seccomp/LSM 与 resource quota；
- egress deny-by-default；
- per-user secret/session/volume isolation；
- 所有调用经 typed MCP gateway；
- cold-start singleflight、bounded queue、readiness、drain 与 idle stop。

进入公共 market 只是申请 shared/warm 的前置条件之一，不自动获得共享进程或常驻资源。完整 policy 见 [`mcp-market-hosting-policy.md`](mcp-market-hosting-policy.md)。

### Local Companion

本地 plugin 不应要求用户设备开放公网入站端口。更自然的模型是：

1. 本地 companion 与账户配对；
2. companion 主动建立加密 outbound session；
3. backend 只在用户授权且设备在线时路由 capability call；
4. 用户可在本地看到并拒绝高风险调用；
5. 设备离线时明确失败，不静默切换到其他执行面。

## 8. UI Plugin 边界

ChatGPT-like plugin window 不等于允许 plugin 注入任意前端代码。

比赛 MVP 建议允许 plugin 贡献：

- 名称、图标、说明和 publisher；
- 权限申请 UI；
- 工具与资源 schema；
- declarative cards/forms；
- settings schema；
- operation status / result blocks。

不建议首版允许任意 JavaScript/WASM 插入主 UI；这会把供应链攻击扩展到每个客户端。

## 9. 目标端建议

### 比赛正式目标

- **Linux Docker Compose server**：正式；
- **Web**：正式主入口；
- **Android APK**：正式 client evidence，但先限制深度 native API。

### 比赛后目标

- iOS：保留接口与 UI 兼容，不在比赛周期承诺签名/App Store readiness；
- Windows/Linux/macOS Desktop：Dioxus WebView 壳可作为候选，但按 native-host CI 分别验证；
- 若需要更深 native integration，可替换 client shell，而不改变 backend/domain/plugin protocol。

## 10. Dioxus 采用前的低成本证伪

在正式选型前做 disposable spike，而不是直接创建主仓库骨架：

1. 使用 stable `0.7.9`；
2. 同一 UI 跑通 Web 与 Android；
3. 登录后调用独立 Rust API；
4. 流式展示 Agent 消息和 tool status；
5. 从 client 打开独立 `/market` frontend，并正确同步 install/disable/enable 状态；
6. Android 真机或 emulator 验证网络、生命周期、返回前后台和文件/链接打开；
7. 记录 bundle、调试难度、平台特判、build time 和 native integration blockers。

证伪条件：

- Android 打包/调试在团队环境中不稳定；
- UI 必须大量依赖 Dioxus 尚不成熟的 native API；
- Web 与 Android 共享导致大量 `cfg` 分叉；
- Dioxus 版本/CLI 使 CI 和可复现构建不可控；
- 关键可访问性或交互无法达到验收要求。

## 11. 当前结论

```text
Rust core/backend:             推荐
Dioxus Web UI:                 推荐进入 spike
Dioxus Android UI:             推荐进入 spike，需早期真机证据
Dioxus Desktop future shell:   可保留
Dioxus Fullstack as authority: 不推荐
Dioxus 0.8 alpha:              不采用
Stable public API boundary:    必须
Full framework fork:           不推荐
Original Rust domain/run core: 推荐
Rig/rmcp behind adapters:      推荐先做 spike
```

Deployment topology 的比较与推荐见 [`deployment-topology-analysis.md`](deployment-topology-analysis.md)。当前推荐是“中央 authority + 多个受控 execution planes”，但尚未获得团队批准。

Agent runtime 的 build-vs-adopt、GitHub references 与 MIT/Apache attribution 见 [`agent-runtime-adoption-policy.md`](agent-runtime-adoption-policy.md)。

Market authority、PluginPackage、PostgreSQL 与 IdentityProvider 合同见 [`agent-market-architecture.md`](agent-market-architecture.md)。
