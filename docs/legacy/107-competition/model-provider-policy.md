# Model Provider 与 User Secret Policy

- 状态：**MVP 方向已明确，待进入完整工程蓝图**
- 更新时间：2026-07-21
- 适用对象：USTC 个人校园 Agent 平台

## 1. 当前决策

Demo 实现两种 model execution mode：

```text
OfficialCentral
UserCloud
```

只预留、不实现：

```text
UserDeviceRelay
UserRemoteRelay
```

其中：

- `OfficialCentral`：平台管理 provider credential，用户选择平台开放的模型；
- `UserCloud`：用户明确上传自定义 provider origin、model 与 API key，中央服务器存储 secret 并直接调用；
- `UserDeviceRelay`：未来由用户当前设备持有 key 并转发；
- `UserRemoteRelay`：未来由用户维护 always-on relay。

“等待 relay 上线”和“always-on user relay”不进入 Demo；只有实际需求数据证明必要后再实现。

## 2. 核心抽象

### ProviderProfile

```text
ProviderProfile
├── id
├── owner_user_id
├── execution_mode
├── protocol_adapter
├── display_name
├── base_origin
├── model_id
├── secret_ref
├── enabled
├── created_at / updated_at
├── last_health_check
└── revision
```

`secret_ref` 指向 secret store；普通 provider/profile 查询不得返回 secret。

### TaskProviderBinding

```text
TaskProviderBinding
├── task_id
├── provider_profile_id
├── binding_revision
└── failure_policy
```

Task 只绑定 profile，不读取或复制 API key。调度时由 provider executor 按 execution mode 解析。

## 3. 统一执行接口

```text
ModelExecutor.invoke(
    provider_profile_id,
    typed_model_request,
    cancellation,
) -> model_event_stream
```

当前 executor：

```text
OfficialCentralExecutor
UserCloudExecutor
```

未来 executor：

```text
UserDeviceRelayExecutor
UserRemoteRelayExecutor
```

新增 executor 不应改变：

- Agent orchestration；
- task state machine；
- model request/response event types；
- audit model；
- UI conversation contract。

## 4. OfficialCentral

平台保存和管理：

- provider endpoint；
- platform API credential；
- allowed models；
- quota/rate limits；
- availability/maintenance state；
- model-specific capability metadata。

用户可以：

- 在允许范围内选择模型；
- 查看模型能力与限制；
- 为交互或 task 绑定 official profile；
- 更改绑定；
- 查看调用错误与使用状态。

平台不得暗中切换到另一模型。若需要 fallback，必须由显式 policy 描述并让用户可见。

## 5. UserCloud

用户主动提供：

- protocol adapter，例如 OpenAI-compatible；
- provider base origin；
- model ID；
- API key 或同类 bearer secret；
- 可选的兼容参数。

中央服务器：

1. 验证 provider profile；
2. 加密 secret；
3. 保存 ciphertext 与 secret metadata；
4. 调用时短暂解密；
5. 直接向 provider 发起模型请求；
6. 流式返回给 Agent runtime；
7. 记录不含 secret 的 audit event。

## 6. 产品透明度

创建 `UserCloud` profile 时必须明确告知：

- key 会上传到 USTC central server；
- key 会被加密存储；
- server 在调用模型时必须能够解密和使用 key；
- prompt、tool result 与选定上下文会发送给该第三方 provider；
- 第三方 provider 的计费、日志和隐私政策由用户承担；
- 删除 profile 会撤销未来使用，但无法撤回已发送给 provider 的数据。

不能声称：

- key 从未离开设备；
- server 永远无法看到 key；
- encrypted at rest 能抵御已被完全攻陷的运行中 server。

## 7. Secret storage

### 最低要求

- 数据库不保存 API key 明文；
- 使用 authenticated encryption；
- encryption/master key 不与 ciphertext 存放在同一数据库；
- Compose 中通过 secret file/受控 runtime secret 注入 master key，不写进 Git；
- secret access 只开放给 provider execution path；
- API/list/debug/error 不回显 secret；
- logs、traces、metrics 和 crash dumps 做 redaction；
- user 可 replace/rotate/revoke/delete；
- secret use 记录 owner、profile、时间、目的和结果，但不记录值；
- backup/restore 明确 key dependency 与丢失行为。

### 建议的 envelope model

```text
master key / KEK      # runtime secret，不入 DB
        │
        ▼
per-secret data key / DEK
        │
        ▼
AEAD ciphertext + nonce + key version
```

比赛 MVP 可采用受审计的 application-level envelope encryption；若学校已有 Vault/KMS，应通过 secret-store adapter 接入，而不是在 domain 层写死。

### 内存边界

server 在发起调用时必然短暂持有 plaintext secret。应：

- 延迟解密；
- 缩短存活时间；
- 不 clone/serialize 到 task payload；
- 不进入 retry queue；
- error context 只保留 secret ID；
- 运行 provider worker 时使用最小权限。

## 8. Custom URL 与 SSRF

`UserCloud` 允许用户控制远端 origin，因此 server 会面临 SSRF 风险。

### 输入形态

若协议允许，用户只提交：

```text
scheme + host + optional approved port
```

具体 API path 由 `protocol_adapter` 生成。不要允许每次请求下发任意完整 URL、headers 或协议。

### 应用层要求

- 默认只允许 `https`；
- parse 后严格验证 scheme、host、port；
- 禁止 userinfo、fragment 与非 HTTP(S) scheme；
- 解析 A/AAAA 后拒绝 loopback、private、link-local、multicast、unspecified 与 metadata ranges；
- 对 IPv4、IPv6 和不同编码形式统一规范化；
- 连接前重新验证 resolved IP，处理 DNS rebinding；
- 默认不跟随 redirect；如未来允许，每次 redirect 都重新执行完整验证；
- request method、path、headers 与 body shape 由 adapter 控制；
- timeout、response-size、stream-rate 与 concurrency 有上限；
- health check 使用相同 network policy。

### 网络层要求

- provider worker 与 internal control/data services 网络隔离；
- egress firewall 阻止内网、host、Docker control socket、metadata service；
- provider worker 不挂载敏感 host paths；
- 无权访问数据库，只通过最小内部接口取得一次性调用材料。

若学校确需访问某个校内 provider，应由管理员加入 explicit trusted-provider registry；不能靠普通用户 URL 绕过 public-network policy。

## 9. API key header 边界

MVP 不允许用户配置任意 header template。建议只支持有限 adapter：

- `Authorization: Bearer <secret>`；
- adapter 明确定义的标准 header；
- model/provider-specific fields 经 schema 校验。

否则任意 header 可能覆盖 `Host`、代理、内部鉴权或 tracing 字段，扩大 SSRF 和数据泄露面。

## 10. Profile lifecycle

```text
create draft
→ validate origin/config
→ store encrypted secret
→ explicit test
→ enable
→ bind to chat/task
→ rotate/disable
→ delete/revoke
```

### Test

“测试连接”必须：

- 显示即将访问的 normalized origin；
- 使用最小请求；
- 走正式 SSRF/network policy；
- 返回结构化错误，不回显 key；
- 不自动把测试成功解释为 provider 安全或官方认证。

### Delete

删除 profile 后：

- 新调用立即拒绝；
- 绑定 task 进入 `provider_unavailable`；
- ciphertext 与 active index 删除；
- backup retention 按公开政策处理；
- audit record 保留 profile ID/事件，不保留 secret。

## 11. Task 语义

MVP 中 task 必须显式绑定：

```text
OfficialCentral profile
or
UserCloud profile
```

由于两种模式都由 central server 调用，不存在等待客户端上线的问题。

失败路径：

- profile disabled/deleted → `provider_unavailable`；
- secret decrypt failure → `provider_secret_unavailable`；
- origin rejected → `provider_origin_rejected`；
- auth failure → `provider_auth_failed`；
- rate limit → `provider_rate_limited`；
- timeout/network → `provider_unreachable`；
- incompatible response → `provider_protocol_error`。

不得自动改用 official model，也不得在 task 创建后悄悄切换 profile revision。

## 12. 延后功能的 extension point

可以预留 enum/trait/schema 位置，但不实现 runtime branch：

```text
UserDeviceRelay
UserRemoteRelay
```

预留只包括：

- execution mode type；
- executor replacement boundary；
- capability metadata；
- provider-unavailable error；
- migration/version responsibility。

不预留：

- 空实现按钮；
- 对用户可见但不能工作的设置；
- 未验证的 WebSocket/device pairing 协议；
- scheduler 中不可达的假分支。

只有实际 usage data 证明需求后，再设计：

- wait-until-online；
- always-on relay；
- device pairing；
- local secret storage；
- reconnect/replay；
- background execution。

## 13. 对客户端的影响

Demo 客户端需要：

- official model selector；
- user cloud provider profile CRUD；
- masked secret input；
- explicit upload disclosure；
- connection test；
- health/error state；
- chat/task provider binding；
- rotate/delete；
- usage/audit summary。

Demo 不需要：

- AI request relay；
- background client service；
- device-online indicator；
- wait-for-device UI；
- always-on relay management。

因此 Dioxus Android 的平台风险显著降低：首版不要求后台常驻 relay。

## 14. 对完整自部署的影响

Self-hosted instance 使用相同 provider model：

- self-host 管理自己的 official/platform provider；
- self-host 用户也可创建 `UserCloud` profile；
- secret encryption key 由 self-host operator 管理；
- official USTC instance 不保存或同步 self-host secrets；
- export 默认不包含 plaintext secrets；若未来支持迁移，需要显式 re-entry 或重新加密协议。

## 15. Demo 验收闭环

至少证明：

1. 用户选择 official model 并完成一次流式对话；
2. 用户创建自定义 compatible provider；
3. UI 明确说明 key 上传和数据发送边界；
4. DB 中看不到 key 明文；
5. logs/traces 不出现 key；
6. 自定义 origin 无法访问 loopback/private/link-local/metadata；
7. 自定义 provider 完成连接测试与流式对话；
8. scheduled task 绑定 `UserCloud` profile 后独立于客户端在线状态运行；
9. 删除/禁用 profile 后 task 得到确定错误；
10. 不发生 silent model fallback。

## 16. 当前结论

- **OfficialCentral**：Demo 实现；
- **UserCloud**：Demo 实现；
- **UserDeviceRelay**：只保留替换边界；
- **UserRemoteRelay**：只保留替换边界；
- **等待 relay/always-on relay**：不做 UI 和 runtime；
- **自定义 URL**：必须经过 application + network 双层 SSRF 防护；
- **用户 key**：可上传，但必须透明、加密、可撤销、不可进日志。

## 17. 事实来源

- OWASP SSRF Prevention Cheat Sheet：<https://cheatsheetseries.owasp.org/cheatsheets/Server_Side_Request_Forgery_Prevention_Cheat_Sheet.html>
- OWASP Secrets Management Cheat Sheet：<https://cheatsheetseries.owasp.org/cheatsheets/Secrets_Management_Cheat_Sheet.html>
