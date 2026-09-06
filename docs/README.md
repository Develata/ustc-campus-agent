# 文档导航

## 了解作品与评分证据

| 要看什么 | 入口 |
|---|---|
| 校园任务、当前能力与评分细分项 | [功能与证据](features/06-mvp-core-capabilities.md) |
| 四分钟演示、模型预检与提交准备 | [演示指南](guides/competition-demo.md) |
| 从源码运行 | [项目 README](../README.md#quick-start) |
| Windows / Linux Docker 演示包 | [Compose 指南](../deploy/mvp-compose/README.md) |
| Android 安装、连接和验证状态 | [Android 指南](guides/android-demo.md) |
| 历史讨论与设计落点 | [讨论决策与文档落点](guides/discussion-to-docs.md) |
| 冻结 R3.1 包身份与复验 | [R3.1 交付身份与复验](guides/r31-delivery-and-verification.md) |

## 使用与配置

| 需求 | 入口 |
|---|---|
| 配置模型地址、私有密钥文件与模型选择 | [模型配置](guides/model-selection.md) |
| 安装插件包、配置 MCP/Skills、授权启用 | [插件配置](guides/mcp-skills.md) |
| 普通用户 CLI 与外部 Agent 接入范围 | [无界面客户端](features/05-headless-client-and-agent-integration.md) |
| 多用户、来源接入与后续实现顺序 | [当前任务表](tasks/multi-user-campus-agent.md) |

## 查看技术实现

| 主题 | 当前契约与说明 |
|---|---|
| 调用关系、模块与状态归属 | [架构](overview/architecture.md) · [模块边界](contracts/module-boundaries.md) |
| 模型调用、工具校验与执行预算 | [Agent Chat](contracts/agent-chat.md) · [权限](contracts/permissions.md) |
| 日历日期与确认 | [操作指南](guides/calendar-proposals.md) · [提案契约](contracts/calendar-proposals.md) |
| 插件安装与执行 | [应用配置](contracts/plugin-management.md) · [MCP](contracts/mcp-execution.md) · [Skill](contracts/skill-context.md) |
| 保存对话与操作 | [对话存储](contracts/chat-conversations.md) · [重命名/删除](contracts/conversation-management.md) |
| 验证命令与状态 | [开发指南](guides/development.md) · [验收矩阵](acceptance/matrix.tsv) |

`implemented` 只表示对应范围已实现；`planned` 模块仍可能包含有界子功能。
当前可演示范围以功能页与验收绑定为准，历史 APK/设计包的通过记录只适用于原候选。

## 文档分工

| 目录 | 内容 |
|---|---|
| [plan/](plan/) | 当前工程原则、产品规划和模块蓝图 |
| [contracts/](contracts/) | 权限、协议、状态与失败语义 |
| [features/](features/) | 用户可见功能和实现范围 |
| [acceptance/](acceptance/) | 检查项、执行绑定与验收状态 |
| [guides/](guides/) | 运行、配置、演示和开发操作 |
| [overview/](overview/) | 架构与跨模块关系 |
| [tasks/](tasks/) | 实现顺序、分工和交付范围 |
| [design/](design/) | 有版本和状态标记的设计包 |
| [adr/](adr/) | 历史决策记录 |

贡献前依次阅读 [工程宪法](plan/00-engineering-constitution.md)、[术语](plan/01-terminology.md)、
[模块地图](plan/modules/00-module-map.md)及相关契约。
[覆盖矩阵](coverage-matrix.md)连接各层文档，[文档约定](AGENTS.md)规定归属；
指南与设计包不替代契约。项目自有文档采用 [MIT License](../LICENSE.md)。

- [校园任务与多人试用](guides/campus-workflows.md)：日历提醒、官方资料、课程到日历和后台配置账号。
