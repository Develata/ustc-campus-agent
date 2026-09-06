# MCP 与 Skills 使用指南

在 Linux/WSL 服务端启用受审阅插件包，让支持工具调用的模型查询公开信息或按需读取
Skill。Windows 浏览器可连接该服务；浏览器不能任意安装主机目录或执行命令。

## 在页面中使用

打开插件页的“管理 MCP 与 Skills”：

1. 安装所选包。安装本身不会连接 MCP 或授予权限。
2. MCP 包填写声明的服务地址并保存配置；没有配置项的 Skill 可以直接检查。
3. 点击“检查组件”。MCP 只初始化和发现工具，Skill 校验文件；不会执行业务工具。
4. 逐项阅读并授权公开读取权限，核对本次发现的清单，再勾选并启用。
5. 回到对话，选择确实支持工具调用的 `openai-compatible` 模型，再提交任务。
6. 停用后新调用立即失去资格。撤销为终止状态；需要重新使用时优先选择可恢复的“停用”。

**模型 API 地址与 MCP 地址是两种接口。** `UCA_AGENT_BASE_URL` 配置模型；
`/v1/chat/completions` 不是 MCP 服务。`local-chat` 没有工具调用；`mock` 只演示
内建校园工具，不执行已安装 MCP 或 Skill。配置方法见[模型指南](model-selection.md)。

## 核对结果与恢复

- “检查组件”成功只证明连接或文件可读。启用后的实际调用及工具状态才证明任务路径可用。
- 配置、服务清单或来源摘要变化后重新检查和审核；旧工具不会偷偷切到新地址或新权限。
- 结果不明时保留原请求编号，显式重试。不要换编号重做；MCP 业务调用不会自动重试。
- 重启保留安装、授权和回执，重新发现须匹配已审核版本。目录缺失的历史安装仍可停用或撤销。

## 添加 MCP 包（管理员）

运行方提供一个绝对路径包目录，包含 `package.json`、`runtime.json`，再用
Rust 命令生成 `configuration.json`。HTTP 前端不接受任意目录或命令。

`package.json` 示例（工具和能力必须经过运行方实际审核）：

```json
{
  "id": "community.campus-mcp",
  "version": "0.1.0",
  "publisher": "local-review",
  "tier": "VerifiedRemoteMcp",
  "displayName": "校园信息 MCP",
  "description": "连接已审核的校园公开信息查询服务。",
  "implementationStatus": "development",
  "installPolicy": {
    "class": "UserInstalledPlugin",
    "defaultInstalled": false,
    "defaultEnabled": false,
    "userDisableAllowed": true
  },
  "components": [{"type": "McpServerComponent", "path": "runtime.json"}],
  "capabilities": ["campus.public_rules.read"],
  "sourcePolicy": {
    "network": "operator-reviewed public-read MCP endpoint",
    "personalData": "no private campus data admitted",
    "execution": "Streamable HTTP; explicit installation and grant"
  }
}
```

`runtime.json` 示例：

```json
{
  "schemaVersion": "plugin-runtime/v1",
  "kind": "mcp",
  "endpointKey": "endpoint",
  "endpointPolicy": "public_https",
  "tools": [{"name": "campus_search", "capabilityId": "campus.public_rules.read"}]
}
```

`campus_search` 必须替换为真实服务中的完整工具清单。不能依据服务器的
`readOnlyHint` 自动认定权限；映射是包的审核内容。远程连接默认使用公共 HTTPS，
拒绝私网、重定向和环境代理。仅限运行方本机测试的包可在 `runtime.json` 中使用
`loopback_development` 并配置数字回环 HTTP 地址；浏览器不能切换该策略。

需要 Bearer 时，由运行方同时配置 `bearerFile`（绝对路径、当前服务用户的
0600 私有文件）和 `credentialEndpoint`（完整精确 MCP URL）。配置的地址
与该 URL 不完全一致时，在读密钥和联网之前拒绝；不能将运营凭据转发到用户
随意改填的服务器。不得把密钥内容写进包、配置字段、Git 或截图。

在 WSL 中准备并启动：

```bash
cargo run --locked -p ustc-agentctl -- market prepare-component-config \
  --package-dir /absolute/path/to/campus-mcp
export UCA_PLUGIN_PACKAGE_DIRS='/absolute/path/to/campus-mcp'
# 在同一个环境中执行现有 ustc-agentd serve-web 启动命令。
```

多个包使用系统路径分隔符（Linux/WSL 为冒号）。准备命令不联网、不安装或授权，
只创建新的配置文件；已有 `configuration.json` 不会被覆盖。包变更使用新版本目录。

## 添加和读取 Skill

参照完整示例 [ustc.campus-guide](../../market/packages/ustc.campus-guide/)。Skill 目录名必须与
frontmatter 中的 `name` 一致，正文为 UTF-8 Markdown：

```markdown
---
name: campus-guide
description: Help organize campus questions and check source authority.
---
先澄清任务，再核对官方信息、社区意见和模型建议各自的来源。
```

`runtime.json` 中声明 `skillPath` 和所有 `resources` 的相对路径及文件摘要。
解析器支持多行 YAML、引号和元数据，拒绝重复键、别名、执行标签和超限内容。
读取器逐级拒绝符号链接、路径穿越、目录和特殊文件。

Skill 资源按页读取：首次可用 `{}` 读取该包声明的 Skill 入口；省略 `resource`
只选择这一精确入口，不会搜索目录。显式传入 `resource` 时只接受包已声明的完整
相对路径，`SKILL.md` 等简称不会自动改写，也不会扩大白名单。路径拼错属于参数错误，
应更正参数或省略 `resource` 读取入口；只有摘要等来源校验失败才要求重新检查组件。
可选 `offset` 是 UTF-8 字节偏移，默认 0；返回 `next_offset`
（末页为 null）和 `total_bytes`。每页最多 16 KiB 原文，同时满足 60 KiB JSON
转义后的预算，每次翻页都重新验证完整文件摘要。工具描述会提示模型按偏移续读。

`allowed-tools` 只是作者建议，不会授权任何工具；脚本和外部链接不会被执行或自动抓取。
一次 Chat 只有两轮工具调用；模型应说明未读内容和续读位置，不能把部分读取说成读完。

## 运行边界

| 项目 | 当前范围 |
| --- | --- |
| MCP 协议 | 2025-11-25 Streamable HTTP，JSON/SSE，分页发现、调用和会话关闭 |
| 包与权限 | 每个安装一个 MCP 或 Skill 组件；应用只执行 `public-read`（公开读取）能力 |
| 工具数量 | 每个 owner 的已启用动态工具合计最多 28 个；每个 Skill 占一个 |
| MCP 输入 | 闭合对象须有 `additionalProperties: false`；支持字符串／字符串枚举、整数、数字、布尔值和数组 |
| 未支持范围 | `$ref`、联合类型、其他未支持约束、中心主机 stdio、可执行 Skill、OAuth、生产 SSO、更新／回滚 |

输入保留平台精确数值类型：`type: integer` 用 `10`，`type: number` 用 `10.0` 或指数
写法，不自动转换。这比标准 JSON Schema 更严格。输出按 JSON Schema 数值含义检查，
允许 number 输出整数、integer 输出 `10.0`；类型错误和超限仍拒绝。

原生非 Unix Skill 文件读取尚不支持。受控测试不代表任意 MCP 服务兼容，也不证明真实模型的任务能力。

## 验证入口

```bash
cargo test --locked -p ustc-campus-agent-adapters
cargo test --locked -p ustc-agentd --lib plugin_runtime
cargo test --locked -p ustc-agentctl package_configuration
UCA_BROWSER_SUITE=plugins node scripts/test_usable_enhancements_browser.mjs target/debug/ustc-agentd
```

浏览器流程需要隔离的新服务状态，因为测试会真实安装、授权、启用、停用和撤销。
详细规则见[插件管理](../contracts/plugin-management.md)、[MCP](../contracts/mcp-execution.md)
及 [Skills](../contracts/skill-context.md)；当前结果不提升完整模块的后续验收状态。
