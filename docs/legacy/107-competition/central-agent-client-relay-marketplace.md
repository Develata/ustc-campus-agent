# Central Agent + Client Relay + Marketplace 架构候选

- 状态：**中央 authority 与 Develata-side Market 本体已明确；MCP hosting/Rust runtime 待团队确认与 spike**
- 更新时间：2026-07-21
- 讨论对象：USTC 个人校园 Agent 平台

## 1. 总体命题

官方在 USTC 基础设施上部署完整 Agent 平台：authority/control/data plane 继续以 Docker Compose 为首期交付形态；运行第三方 MCP artifact 的 execution plane 可位于独立 runtime node/K3s cluster。中央 authority 负责：

- 用户身份与设备登记；
- Agent runtime 与 orchestration；
- memory；
- `PluginPackage` 安装、启停、组件解析与 grants；
- default first-party Plugin 所引用的 MCP、Skills、ControlledCLI 与 shared services；
- tasks、scheduler 与运行状态；
- campus source/service graph；
- 独立 Market frontend 所共用的 catalog/install backend；
- permission grants；
- provenance、audit 与 evaluation。

客户端承担以下职责：

1. Web/Android/Desktop 用户界面；
2. 本地 chat archive、browser SSO session 与 client auth token；
3. 未来可选的 user-owned egress relay；该 relay 不进入 Demo 的 AI provider runtime。

完整自部署不是“轻客户端自己运行 Agent”，而是用户自行部署同一套完整 central stack。

## 2. 一句话架构

> **中央服务器负责思考、状态与调度；客户端负责呈现，以及在用户授权时使用用户本地持有的秘密执行受控外呼。**

这仍然满足：

```text
一个 state authority
+ 一个 agent authority
+ 多个受控 execution locations
```

它不要求：

```text
中央状态 ↔ 本地状态双向同步
中央 Agent ↔ 本地 Agent 双 authority
```

## 3. 数据与职责边界

### 中央服务器保存

- USTC identity 的稳定 subject / account binding；
- devices 与 public keys；
- private/user memories；
- installed `PluginPackage` exact versions/component digests；
- user-owned private PluginPackage manifests/admission state（不进入 public catalog）；
- Plugin/component capability grants；
- tasks、schedules、task state 与 receipts；
- agent configuration；
- default/official MCP configuration；
- provider profile 的非秘密 metadata；
- 用户明确上传的 `UserCloud` AI provider secret，以 encrypted secret 形式保存；
- 用户明确上传的 remote MCP credential，以 encrypted secret 形式保存；
- public Git catalog 的 PostgreSQL projection，以及 private installation、rollout、deployment policy 与 runtime provenance；
- campus source graph；
- necessary audit/provenance；
- 当前 Agent turn 所需的 working context。

### 客户端保存

- raw local chat archive；
- central service refresh/session token；
- USTC browser/CAS session cookie，由 browser/WebView cookie store 管理；
- future `UserDeviceRelay` 模式下的 user-owned AI API key/provider profile；
- future `UserDeviceRelay` 模式下、不上传中央的 remote/local MCP credentials；
- local companion pairing private key；
- local preferences that need not roam。

### 中央服务器禁止保存

- 用户 USTC 原始密码；
- 默认浏览器密码库；
- 用户 AI API key 明文；`UserCloud` 只允许 encrypted-at-rest secret；
- 用户 remote MCP secret 明文；
- 用户 WebView/CAS cookie；
- 可反向恢复上述 secret 的日志或 error dump。

## 4. “Chat history 本地、memory 中央”如何同时成立

必须区分：

- **Raw transcript archive**：用户设备上的逐条聊天记录；
- **Working turn context**：中央 Agent 完成当前会话所需的上下文；
- **Durable semantic memory**：用户明确允许进入中央 memory 的稳定事实/偏好；
- **Task state**：中央 task/scheduler 运行所需状态。

建议语义：

1. raw transcript archive 默认只保存在客户端；
2. 发起请求时，客户端按 protocol 上传当前 turn 与必要的有限历史；
3. 中央可能在任务运行期间持有 ephemeral working context；
4. durable memory 作为独立、用户可查看/删除的中央对象写入；
5. 中央日志不得把完整 prompt/response 当作普通 debug 字段长期保留。

代价：不同设备不会天然共享完整 chat history。若未来提供同步，应单独设计 E2EE transcript sync，不应悄悄把“本地历史”改成普通云端存储。

## 5. AI Provider Execution Modes

Demo 实现：

```text
OfficialCentral
UserCloud
```

只保留未来 extension point：

```text
UserDeviceRelay
UserRemoteRelay
```

`OfficialCentral` 使用平台管理的 provider credential；`UserCloud` 由用户明确上传自定义 origin、model 与 API key，中央加密保存并直接调用。两者均不依赖客户端在线。

完整 provider/secret/SSRF/task policy 见 [`model-provider-policy.md`](model-provider-policy.md)。未来如实现 relay，仍必须是 typed capability，不能做 generic HTTP proxy：

```text
AiProvider.invoke(profile_id, request)
McpGateway.call(plugin_installation_id, component_binding_id, tool, arguments, expected_versions)
```

## 6. MCP Binding 与 Hosting

MCP 现分为三种来源：

```text
Official / Verified Market
User Remote
User Hosted Private
```

- `Official / Verified Market`：平台导入或审核 immutable artifact，统一通过 gateway 提供；只有通过 `SharedSafe` gate 才能共享 deployment；
- `User Remote`：用户上传 Streamable HTTP endpoint 与 binding-specific credential，中央加密保存并直接调用；
- `User Hosted Private`：用户提交 approved OCI-digest artifact，平台以 `DedicatedUser + OnDemand` 方式运行；不得把 arbitrary command/args 当作配置直接执行。

Catalog trust、execution location、tenant mode 与 availability class 是四个独立轴。market 登记不自动意味着可共享或可常驻；warm tier 还需要 artifact、runtime、quota 与 demand gate。

所有类型共用：

- released MCP protocol adapter；
- typed MCP gateway；
- tool exact-name grant + capability snapshot hash；
- JSON Schema validation、timeout/cancellation、size/rate limits；
- write/destructive confirmation；
- per-user session/secret/audit boundary；
- tool output untrusted boundary；
- no silent fallback。

Remote binding 与 grants 见 [`mcp-binding-policy.md`](mcp-binding-policy.md)；market hosting、cold/warm、sandbox 与攻击面见 [`mcp-market-hosting-policy.md`](mcp-market-hosting-policy.md)。

## 7. Future Client Relay 的设备限制

### Native Android/Desktop companion

可作为正式 relay，因为它可以：

- 使用 OS secure storage；
- 持有 device key；
- 建立 authenticated outbound session；
- 调用外部 AI/MCP；
- 在必要时显示本地确认。

Android key 应由 Android Keystore 保护；app login 应存 token，而非账户密码。

### Web client

不能作为可靠常驻 relay：

- tab 可能被休眠/关闭；
- background execution 受限；
- browser CORS 可能阻止 provider/MCP；
- browser local storage 不适合长期持有高价值 secret；
- service worker 也不能保证任意时刻在线。

因此 Web relay 最多作为 foreground experimental path，不写入正式可用性承诺。

### Android background

Android native app 也不能默认假设永久在线。未来若实现 `UserDeviceRelay`，首个 relay 版本最多承诺：

> user-key relay 只在用户设备在线且 relay session 活跃时可用。

## 8. Scheduled Tasks

MVP 中 task 显式绑定 `OfficialCentral` 或 `UserCloud` profile。两种模式均由中央服务器执行，不依赖客户端在线。

不得 silent fallback；profile disabled/deleted、secret unavailable、origin rejected、auth failure、rate limit 与 timeout 都应产生确定错误。

“等待 relay 上线”和“always-on user relay”只保留 executor extension point，不进入 Demo UI/runtime；等待真实 usage data 证明必要后再实现。

## 9. USTC SSO 与 Browser

### 默认路径：Custom Tab / system browser

Android 官方文档说明 Custom Tabs 使用首选浏览器的 session，包括 cookies 和 saved passwords。对 USTC 外部页面，这通常是首选：

- 用户熟悉且信任浏览器地址栏；
- 可复用浏览器已有登录状态；
- app 不读取或注入密码；
- app 不管理默认浏览器 cookie store。

### 可选路径：受限 Embedded WebView

WebView 的 session 与默认浏览器隔离。若提供内置浏览器：

- 用户只在官方 `https://id.ustc.edu.cn` / 经核验的 CAS flow 登录；
- app 保存的是 WebView sandbox 内的 session cookies，不保存原始密码；
- 所有 URI 必须 parse 后同时 exact-validate scheme 与 host；
- 未在 registry allowlist 的 host 转交 system browser；
- 不以 `contains("ustc.edu.cn")` 或错误的 suffix check 作为信任判断；
- 禁止 cleartext HTTP；
- 默认不开放 JavaScript native bridge；
- 页面持续显示真实 origin 和外部跳转提示；
- 提供 logout/clear-site-data。

### 禁止的表述

不要在产品文案中写：

> 自动附带已记录的 USTC 统一认证凭证。

建议改为：

> 在受信任的官方认证流程中复用本设备已有的 SSO session；平台不保存或代填你的 USTC 密码。

## 10. Services / Quick Links 页面

参考图中的分类宫格是合理的 deterministic fallback，不应强迫用户先与 Agent 对话。

### 适合借鉴

- 一级 services 页面；
- 4-column mobile grid；
- 稳定场景分类；
- 克制的浅色层次；
- 图标 + 明确文本；
- 固定导航；
- 可作为 Agent 失败时的人工入口。

### 不应照搬

- 原图图标、品牌资产与具体视觉表达；
- 所有入口等权；
- 混合“学习/生活”场景和“工具/学校”属性的分类；
- 只靠滚动找服务；
- 无来源、登录状态和跳转类型提示。

### 推荐模型

每个 service entry 是 data-driven registry object：

```text
ServiceEntry
├── stable ID
├── display name / aliases
├── category
├── icon asset
├── canonical URL
├── exact allowed origins
├── launch mode: custom-tab | embedded | external
├── auth mode
├── audience/role
├── source/provenance
├── health/status
└── plugin owner
```

页面建议包含：

- 常用/固定；
- 最近使用；
- 全部服务分类；
- 搜索、别名与自然语言 lookup；
- 需校园网/需登录/外部页面等标签；
- 用户可固定但系统分类保持稳定。

Plugin 可以贡献 declarative service entries，但不得注入任意 UI code。

## 11. Market 与 PluginPackage

Market 的唯一一等安装单位是：

```text
PluginPackage
├── McpServerComponent*
├── SkillComponent*
├── ControlledCliComponent*
└── SharedServiceBinding*
```

一个纯 MCP/Skill 仍包装成单 component PluginPackage。`PluginPackage` 不是 in-process arbitrary-code plugin。

必须区分：

1. **Publish**：reviewed Git manifest 进入 public catalog；
2. **Install**：用户 pin 一个 immutable PluginPackage version；
3. **Configure**：写非秘密配置与 secret references；
4. **Enable**：Agent 可发现其 capabilities；
5. **Invoke**：单次调用仍经过 auth、tenant、installation 与 capability gate；
6. **Private upload/connect**：系统生成 user-owned `PrivatePluginPackage` + installation；component binding 只是 runtime projection，不自动 publish，也不能绕过 install/enable/grant。

完整本体见 [`agent-market-architecture.md`](agent-market-architecture.md)。

## 12. Publisher、Identity 与 Artifact 绑定

学校身份体系有利于 accountability，但生产 identity 通过 `IdentityProvider` adapter 接入：优先 USTC 统一认证，local admin 仅作 audited break-glass。公共 artifact 应绑定：

- stable publisher subject；
- display name/organization；
- GitHub public source/catalog project、commit/tag/release；
- immutable artifact digest；
- version 与 compatibility range；
- license/upstream provenance；
- submission/review/publish timestamps；
- review state 与 revocation state。

身份绑定降低匿名滥用，但不等于内容安全。

## 13. Text-only Skill 初期策略

初期只允许标准格式、纯文本 skill 是合理收缩。建议只允许：

- Markdown/text；
- strict manifest；
- size/depth/reference limits；
- 无 binary；
- 无 executable；
- 无 symlink；
- 无 script/template expansion；
- 无自动 secrets；
- 明确的 tool/capability declarations。

### 自动检查

Deterministic validator：

- schema 与 required fields；
- path/archive safety；
- Unicode control/bidi/hidden content；
- size/token limits；
- external URLs 与 provenance；
- declared tools vs referenced tools；
- forbidden executable/file forms；
- duplicate/spoofed identity；
- reproducible artifact digest。

Agent/LLM review：

- prompt injection indicators；
- secret collection/exfiltration instructions；
- authority escalation；
- misleading claims；
- hidden side effects；
- policy and quality triage。

LLM review 应产生 evidence/report，不应成为唯一 security gate。

## 14. Marketplace 信任等级

建议至少分开：

### First-party / Official verified

- 学校或官方项目维护；
- 明确人工 owner；
- 高信任 badge；
- 可进入默认推荐。

### Identity-bound Community text skill

- USTC 身份绑定；
- 通过 deterministic validation；
- 通过自动语义检查或进入 review queue；
- 明确标注 community，不冒充官方 endorsement；
- 用户安装时显示声明的 capabilities。

### Code / Script plugin

- 不在初期开放；
- 后续必须人工审查；
- 仍需签名、隔离 runner、权限与运行时限制；
- 人工审查不能替代 sandbox。

纯文本 skill 仍能改变 Agent 行为、诱导 tool calls 或泄露信息，因此“没有脚本”不等于“无风险”。

## 15. 审核队列

不建议把语义写成“服务器空闲时自动审核”。更可维护的模型是显式 review queue：

```text
submitted
→ deterministic validation
→ automated semantic review
→ quarantined/community-ready/rejected
→ publish
→ monitor/revoke
```

worker 可以在资源低峰运行，但 submission state、deadline、retry、failure 与 evidence 必须可见。

## 16. Compose 部署与 Market authority

首版不做无必要 microservices。authority plane 仍建议 modular monolith + workers；运行第三方 code 的 runtime plane 必须独立：

```text
reverse-proxy
server                  # API + identity + domain authority
worker                  # tasks/review/indexing
mcp-gateway              # typed invocation/policy；不持有 host runtime admin 权限
mcp-runtime-controller   # narrowly privileged；只接受 approved typed deployment spec
isolated-runtime-node    # third-party artifact sandbox；与 DB/authority 网络隔离
postgres                # central state
object-store            # artifacts/evidence，可按 MVP 简化
```

authority services 可按同一 Rust binary 的不同 mode 复用实现；runtime controller 不能与公网 API 共用 Docker socket 或 orchestrator admin credential。Compose 本身不是 demand-driven autoscaler；比赛期若不引入 K3s/KEDA，只能诚实实现 bounded cold-start demo，不能宣称已有生产级 arbitrary-code hosting。

GitHub public catalog repositories 负责：

- `PluginPackage` / component manifests；
- source/provenance/license；
- immutable release/tag/commit/digest reference；
- catalog review/default/update policy。

PostgreSQL 负责：

- Git catalog read projection；
- users/roles；
- exact installations 与 enabled state；
- grants/config/secret references；
- rollout/deployment desired/observed state；
- private audit events。

Git catalog 与 PostgreSQL user/runtime state 的字段集合必须正交；不得形成双向可写 catalog。

## 17. 完整自部署

用户若希望 self-host，应部署完整 central stack：

- server；
- database；
- workers/runners；
- marketplace index 或 configured registry；
- identity provider integration；
- storage/backup；
- clients 指向该 authority。

默认不支持 official instance 与 self-hosted instance 的实时双向同步。二者是独立 authorities；未来如需迁移，只设计显式 export/import。

## 18. Dioxus 与独立 Market frontend 边界

Dioxus 仍只负责 client shell/UI：

- ChatGPT-like conversation shell；
- services grid；
- 启动独立 Market HTTPS entry，并展示 installation summary；
- settings/device/provider profiles；
- permission dialogs；
- task/plugin status。

Relay core 应是独立 Rust crate：

- secure profile storage adapter；
- paired session；
- typed AI/MCP invocation；
- stream/cancel/timeout；
- local confirmation；
- audit receipt。

Android 若需要可靠 background relay，可能需要 platform-specific service；不能假设 Dioxus UI lifecycle 自动满足该需求。

Market frontend 可独立 build/repo，但与中央 Agent 共用 Auth、PostgreSQL 与 Market backend。首期优先通过同一 HTTPS origin 的 `/market` 暴露，避免复制 identity/install authority。

## 19. 当前推荐 MVP

### 必须完成

- 中央 Compose 完整运行；
- central Agent + memory/skill/task authority；
- local raw transcript archive；
- Web/Android 基础 clients；
- services/quick-links 页面；
- 独立 `/market` browse/detail frontend，共用中央 Auth/backend；
- GitHub public catalog importer + PostgreSQL projection；
- `PluginPackage` exact install/disable/enable lifecycle；
- 三个 exact-version、default-enabled、read-only `FirstPartySystemPlugin`；实现按一条 vertical slice 验证后顺序扩展，不并行铺开三个产品；
- typed Rust config loader 与 static/resolved/live-readonly configuration smoke；
- 完整 acceptance matrix、case registry、machine-readable evidence 与 `ustc-agentctl` gate；required case 不允许 skip-as-pass；
- Custom Tab 默认打开外部校园服务；
- official/default model path；
- 一个 `UserCloud` custom provider 完整配置、加密存储与调用闭环；
- 一个 first-party hosted MCP + 一个 `UserRemote` MCP 完整配置、加密存储、discovery、grant 与调用闭环；
- text-only community skill submission + deterministic validator；
- publisher/artifact identity binding；
- install/grant/revoke/audit。

### Risk Spike A GO 后才进入 committed MVP

- 一个 OCI-digest `UserHostedPrivate` MCP，完成 dedicated on-demand cold start、idle stop、quota 与 tenant-isolation evidence；
- 对应 `demo-hosted` acceptance profile。

若 Risk Spike A 为 NO-GO，以上两项明确移入 Stretch/deferred，不阻塞 core `demo`；不得把 unavailable 记为 Pass。

### Stretch

- restricted WebView SSO session；
- user-device/always-on AI relay；
- Android background relay；
- local MCP companion；
- automated semantic review；
- code/script plugin review pipeline；
- npm/PyPI/MCPB isolated build 与 public warm-tier autoscaling；
- E2EE transcript sync；
- desktop/iOS clients。

## 20. AI/MCP execution policy

- AI：`OfficialCentral | UserCloud`；
- MCP source：`Official/VerifiedMarket | UserRemote | UserHostedPrivate`；
- MCP runtime：`SharedSafe | DedicatedUser` × `OnDemand | Warm`，由独立 policy 决定；
- user-device / always-on relay：只保留 future executor boundary，不进入 Demo。

## 21. 当前事实来源

- Android Custom Tabs：<https://developer.android.com/develop/ui/views/layout/webapps/overview-of-android-custom-tabs>
- Android Embedded Web 比较：<https://developer.android.com/develop/ui/views/layout/webapps/in-app-browsing-embedded-web>
- WebView unsafe URI loading：<https://developer.android.com/privacy-and-security/risks/unsafe-uri-loading>
- WebView native bridge 风险：<https://developer.android.com/privacy-and-security/risks/insecure-webview-native-bridges>
- Android Keystore：<https://developer.android.com/privacy-and-security/keystore>
- Android Credential Manager：<https://developer.android.com/identity/credential-manager>
- USTC 统一身份认证公开入口：<https://id.ustc.edu.cn>
- Model provider 与 user secret policy：[`model-provider-policy.md`](model-provider-policy.md)
- MCP binding、permission 与 secret policy：[`mcp-binding-policy.md`](mcp-binding-policy.md)
- MCP market hosting 与 runtime security policy：[`mcp-market-hosting-policy.md`](mcp-market-hosting-policy.md)
- Agent runtime 采用与 license/provenance policy：[`agent-runtime-adoption-policy.md`](agent-runtime-adoption-policy.md)
- Market authority、PluginPackage、Auth、i18n 与 update policy：[`agent-market-architecture.md`](agent-market-architecture.md)
- Docs 分层与 SSH/Slurm execution blueprint：[`project-documentation-and-execution-blueprint.md`](project-documentation-and-execution-blueprint.md)
- Rust CLI/config smoke/evidence contract：[`rust-cli-config-smoke-contract.md`](rust-cli-config-smoke-contract.md)
- 完整 platform acceptance baseline：[`platform-acceptance-matrix.md`](platform-acceptance-matrix.md)
