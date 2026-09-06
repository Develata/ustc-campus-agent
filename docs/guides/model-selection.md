# 配置和选择聊天模型

模型菜单使用服务端已配置的连接。选择只影响下一条消息；历史回答和正在运行的请求保留原模型。

## 选择与核对

1. 打开输入框工具栏的模型菜单，查看名称和工具能力，选择后发送消息。
2. 在回答详情中核对实际使用的模型。“已配置”不等于已经连通或正确使用工具。
3. 列表加载失败时重试；旧选项被移除时重新选择。请求进行中或结果不明时，选择保持锁定。

| 模式 | 可以验证什么 |
| --- | --- |
| `mock` | 不联网的确定性演示；只调用内建校园工具，不调用已安装 MCP 或 Skill |
| `local-chat` | 真实小模型纯文本聊天；不提供工具调用 |
| `openai-compatible` | 真实 Agent 工具链；需要服务和模型确实支持工具调用 |

## 配置模型（管理员）

现有 `UCA_AGENT_PROVIDER`、`UCA_AGENT_BASE_URL`、`UCA_AGENT_MODEL`、
`UCA_AGENT_API_KEY_FILE`、`UCA_AGENT_TIMEOUT_MS` 和 `UCA_AGENT_CONTEXT_TOKENS`
配置继续生效，作为 `default` 选项。不配置额外模型也可使用。
网络模式要求见 [Chat provider 契约](../contracts/agent-chat.md#4-provider-profile-and-transport)。

在服务端创建 JSON 文件，例如 `/home/operator/.config/uca/models.json`：

```json
{
  "schema": "uca-agent-models/v1",
  "models": [
    {
      "id": "campus-agent-model",
      "label": "校园 Agent 模型",
      "mode": "openai-compatible",
      "base_url": "https://your-model-service.example/v1",
      "model": "your-tool-capable-model",
      "api_key_file": "/home/operator/.config/uca/provider.key",
      "timeout_ms": 30000,
      "context_limit_tokens": 32768
    },
    {"id": "offline-demo", "label": "离线演示", "mode": "mock"}
  ]
}
```

将示例地址和模型替换为真实配置。`context_limit_tokens` 是不超过模型实际容量的请求预算，
不可填写超过服务能力的数值来绕过检查。模型密钥只放在服务用户可读的私有文件中（Unix
权限 0600），不写进 JSON、浏览器或 Git。新模型条目的 `api_key_file` 必须是绝对路径。

在原启动环境中增加：

```bash
export UCA_AGENT_MODELS_FILE='/home/operator/.config/uca/models.json'
# 然后执行原有 ustc-agentd serve-web 启动命令。
```

新增/移除模型后重启服务，再刷新菜单，应看到当前可选项。最多添加 15 个模型；重复 ID、
未知字段、不合法连接或凭据会拒绝启动。`default` 是保留 ID。文件只声明已有
三种 provider 配置，不执行命令，也不会从远程自动导入模型。

## 验证与边界

- 浏览器只记住模型 ID；服务器重新验证选择，不接受浏览器提供密钥或地址。
- 额外文件配置当前支持 Linux/WSL；原生非 Unix 环境保留原默认配置路径。
- 结果不明的保存对话按原请求重试，不会换模型重发；已完成回答保留实际 provider 身份。
- 用户级模型授权、计费和自动切换尚未实现。模型菜单不改变插件授权。

[MODEL-001](../contracts/model-selection.md)规定目录、请求冻结和重试语义。
受控 HTTP 测试验证选择确实到达不同模型接口；它不代表外部模型服务当前可用。
