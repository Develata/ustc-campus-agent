# USTC Campus Agent

> 模型提出建议，Rust 校验与执行。

<p align="center">
  <a href="https://github.com/Develata/ustc-campus-agent/actions/workflows/ci.yml"><img alt="CI" src="https://github.com/Develata/ustc-campus-agent/actions/workflows/ci.yml/badge.svg"></a>
  <img alt="Rust" src="https://img.shields.io/badge/core-Rust-000000?logo=rust">
  <img alt="Docker Compose" src="https://img.shields.io/badge/demo-Docker%20Compose-2496ED?logo=docker&logoColor=white">
  <a href="LICENSE.md"><img alt="MIT License" src="https://img.shields.io/github/license/Develata/ustc-campus-agent"></a>
</p>

<p align="center">
  <a href="README.zh-CN.md">简体中文</a> ·
  <a href="README.en.md">English</a> ·
  <a href="docs/features/06-mvp-core-capabilities.md">功能与边界</a> ·
  <a href="deploy/mvp-compose/README.md">Docker 运行指南</a>
</p>

USTC Campus Agent 是一个面向校园场景的**有界 Agent 学生竞赛项目**。模型可以理解问题、组织步骤并提出工具调用；真正的参数校验、权限判断、数据来源、状态变更和副作用则由 Rust 掌握。

它不是给通用聊天界面换一层校园外壳。项目试图回答一个更具体的问题：当 Agent 开始查询办事流程、使用个人档案或写入日历时，如何让每一步都保持**可约束、可解释、可核对**。

> [!IMPORTANT]
> 本项目由个人独立维护，**不是中国科学技术大学官方服务**。当前版本是仅监听本机回环地址的竞赛演示 MVP，不连接学校账号，也不提供生产级公共部署。

<details>
<summary>目录</summary>

- [一分钟认识项目](#overview)
- [核心能力](#features)
- [快速开始](#quick-start)
- [工作原理](#architecture)
- [数据、隐私与统一身份认证](#privacy)
- [Android 演示端](#android)
- [项目边界](#boundaries)
- [开发与验证](#development)
- [文档导航](#documentation)
- [许可证](#license)

</details>

<a id="overview"></a>

## 一分钟认识项目

一次典型请求会经过这条路径：

```text
用户问题
  → 模型理解意图并提出受限工具调用
  → Rust 校验工具、参数、授权与当前状态
  → 固定范围的校园能力执行
  → 返回自然语言结果、来源信息与脱敏工具轨迹
```

默认 `mock` provider 不需要 API key，适合离线、确定性的演示与验收；也可以显式配置服务端 OpenAI-compatible provider。两种模式使用同一套 Rust 工具定义和执行边界，浏览器不会持有模型密钥或校园数据权限。

<a id="features"></a>

## 核心能力

| 能力 | 当前可以做什么 | 刻意不做什么 |
|---|---|---|
| **Affairs Navigator** | 查询一条经审阅的成绩单证明办理流程，展示步骤、官方入口与来源状态 | 不伪造受理、审批或实时官方结果 |
| **ChangeRadar** | 展示一组固定、经审阅的校历语义变更 | 不把模型可见查询变成管理员发布权限 |
| **Opportunity Graph** | 在显式同意后，使用 synthetic 档案与公开聚合信号生成可复现课程方案 | 不声称实时选课、完整培养方案或学校推荐 |
| **Simple Calendar** | 记录、列出和精确删除 owner-local 事项；重启后保留已提交状态 | 不实现提醒、周期任务、CalDAV 或自然语言日期解析 |

此外，当前 Web 演示提供：

- **场景化入口**：只填入受支持的问题，不自动发送、授权或写入数据；
- **可配置课程草稿**：从仓库内 synthetic catalog 选择条件，再由 Rust 生成候选；
- **个人办理清单**：标记页面内个人进度，并复制或下载带来源说明的 Markdown；
- **可读工具轨迹**：仅展示调用顺序、工具名与 `succeeded / denied / failed`，不泄露原始私有参数。

<a id="quick-start"></a>

## 快速开始

### 从源码运行

需要 Git 与 Rust 工具链：

```bash
git clone https://github.com/Develata/ustc-campus-agent.git
cd ustc-campus-agent
./scripts/run_three_plugin_mvp.sh
```

服务就绪后打开 <http://127.0.0.1:8787>。可以依次尝试：

```text
你好，介绍一下你能做什么。
成绩单证明怎么办？
校历最近有什么变更？
记录事项：提交开题报告
列出我的待办事项
```

课程规划需要先在页面中创建 synthetic 演示档案，并为**本次请求**单独确认使用；没有确认时，系统应明确拒绝或省略该工具，而不是猜测结果。

### 使用 Docker Compose 演示包

仓库提供可复现的打包脚本；组装完成的演示包包含 Windows、macOS 和 Linux launcher。打包脚本本身要求 **x86_64 GNU/Linux** 构建环境与 source-bound ELF binary。先构建 binary，再组装一个此前不存在的输出目录：

```bash
cargo build --release --locked -p ustc-agentd
./scripts/package_three_plugin_mvp_compose.sh \
  --binary target/release/ustc-agentd \
  --output-dir dist/ustc-campus-agent-mvp-compose \
  --source-commit "$(git rev-parse HEAD)"
cd dist/ustc-campus-agent-mvp-compose/ustc-campus-agent-mvp-compose
```

然后从包目录启动：

```bash
./start.sh       # macOS / Linux
# start.cmd      # Windows 11 + Docker Desktop
```

launcher 会等待健康检查并给出 ready URL。macOS / Linux 使用 `docker compose down` 停止并保留状态，Windows 使用 `stop.cmd`；`reset` launcher 会删除本 MVP 的 Docker volume，执行前会再次确认。完整配置、端口、provider secret 和校验步骤见 [Docker 运行指南](deploy/mvp-compose/README.md)。

> 首次 `docker compose up --build` 需要联网拉取基础镜像并安装系统包；应用的默认 mock/fixture 运行路径本身不访问实时校园来源。

<a id="architecture"></a>

## 工作原理

```text
Web browser ───────────────┐
                           ├─ loopback HTTP ─→ ustc-agentd
Android debug thin client ─┘                     │
                                                  ├─ bounded ChatRun
mock / server-side provider ─────────────────────┤  model proposes only
                                                  └─ Rust validates and executes
                                                       ├─ Affairs Navigator
                                                       ├─ ChangeRadar
                                                       ├─ Opportunity Graph
                                                       └─ Simple Calendar
                                                              ↓
                                                reviewed/synthetic fixtures
                                                + local durable state
```

几个关键约束：

1. **服务端权威**：客户端只展示状态、提交意图，不拥有业务规则或副作用权限。
2. **固定工具目录**：模型只能从受审阅的工具集合中提议调用，不能注册任意命令。
3. **显式授权**：私有档案按请求授权；provider 文本不能替用户制造确认。
4. **诚实退化**：缺少授权、来源冲突、参数不一致或 provider 失败时，返回明确的非成功结果。
5. **有界执行**：对 provider 轮次、工具次数、参数与结果大小设置硬上限。

更完整的状态、错误和权限语义见 [Agent Chat contract](docs/contracts/agent-chat.md) 与 [MVP capability contract](docs/features/06-mvp-core-capabilities.md)。

<a id="privacy"></a>

## 数据、隐私与统一身份认证

- 默认服务只监听 `127.0.0.1`，不会主动暴露到局域网。
- 校园事实明确区分 reviewed fixture、synthetic fixture、公开聚合信号和 owner-local 私有状态。
- provider key 只允许从服务端文件边界读取；不得写入浏览器、仓库、Compose YAML 或命令参数。
- 不提交 USTC 凭据、CAS cookie、API key、真实学生数据或含私有载荷的日志。

正式接入学校统一身份认证需要校方授权、应用登记与回调配置。当前运行时使用本地演示用户会话，**不收集学校账号密码，也不声称已接入 USTC SSO**。

仓库附带一个[独立、默认拒绝的 SSO 接口样例](examples/sso-interface/README.zh-CN.md)，用于说明未来适配器形状。它不在 `ustc-agentd` 路由内，不访问校园服务器，也不会签发应用会话；取得授权后仍需完成协议验证、身份映射与平台会话集成。

<a id="android"></a>

## Android 演示端

`apps/ustc-android-demo` 是 debug-signed thin client，通过 `adb reverse` 访问同一台主机上的 loopback Web 服务。它不包含第二套 Agent 或校园工具实现。

```bash
adb reverse tcp:8787 tcp:8787
adb install -r <source-bound-debug-apk>
```

这是竞赛演示桥接，不是 Play Store / production Android release；物理设备、生产签名、HTTPS 公共服务与完整生命周期验收仍在当前边界之外。详见 [Android demo guide](docs/guides/android-demo.md)。

<a id="boundaries"></a>

## 项目边界

当前版本明确是**可运行的本机演示纵切面**，而不是生产校园平台。它尚不包含：

- 生产级身份认证、USTC SSO、公开 HTTPS 服务与多租户管理；
- 实时校园数据抓取、广覆盖数据源或官方数据授权；
- 通用第三方插件安装／隔离、命令 sandbox 或可用的 MCP server；
- 持久化聊天、流式输出、提醒、周期日历或跨端同步；
- production-signed Android 与应用商店分发。

这里的“受限”不是能力不足的修饰语，而是产品合同：宁可明确拒绝，也不让模型以看似合理的文本越过权限、来源或状态边界。

<a id="development"></a>

## 开发与验证

Rust 构建可能占用较多磁盘；开始前建议先检查空间。常用的本地验证命令：

```bash
cargo fmt --all -- --check
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo test --locked --all-targets --all-features
python3 scripts/check_repo_contracts.py
```

SSO reservation sample 可以独立验证：

```bash
python3 -B -m unittest discover -s examples/sso-interface -p 'test_*.py' -v
```

运行时中，`m00-sessions.json` 是 `event-history-only` 的当前会话读取权威；`B4b stable redacted control-event/error` journal 只是 `data-only` 证据载体。二者都不是正式 SSO 或通用管理员 API。

贡献前请先阅读 [`AGENTS.md`](AGENTS.md) 和 [development guide](docs/guides/development.md)。仓库采用 protected `main`、精确路径 staging、PR review 与 required checks；一个提交只承载一个语义意图。

<a id="documentation"></a>

## 文档导航

- [MVP 功能、架构与 TODO](docs/features/06-mvp-core-capabilities.md)
- [Docker Compose 运行指南](deploy/mvp-compose/README.md)
- [Agent Chat 合同](docs/contracts/agent-chat.md)
- [权限合同](docs/contracts/permissions.md)
- [Android 演示边界](docs/guides/android-demo.md)
- [工程蓝图](docs/plan/)
- [验收矩阵与门禁](docs/acceptance/)
- [完整文档地图](docs/README.md)

<a id="license"></a>

## 许可证

项目自有软件与文档使用 [MIT License](LICENSE.md)。MIT 许可不代表 USTC 背书、生产就绪，也不自动授予第三方内容或校园数据的再发布权；相关来源与权利边界分别保留。
