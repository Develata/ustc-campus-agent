# 智能体赛道优先概念：USTC 个人校园 Agent 平台

- 状态：**Develata 已冻结为智能体赛道主项目候选；团队 acceptance pending，尚未最终定题**
- 产品名称：**待定**
- 更新时间：2026-07-21

## 1. 当前产品命题

构建面向 USTC 学生的个人校园 Agent 平台。它不是固定菜单的“我的科大”复制品，也不是只在静态知识库上做问答；它以自然语言接收目标，使用带来源、时效与权限边界的校园信息和工具完成检索、流程组织、机会匹配、变化提醒与个人规划。

平台允许用户通过 Market 安装 `PluginPackage`，以扩展新的校园来源、服务能力和工作流。PluginPackage 本体、catalog authority、默认权限和更新策略已形成 Develata-confirmed 架构候选；团队确认与 runtime spike 尚未完成。

## 2. 三项原生能力

G1–G3 不是互不相干的三套功能，而是同一校园信息图谱的三个投影。

### G1｜校园办事与服务导航

用户提出“我要完成什么”，Agent 返回：

- 前置条件；
- 适用对象；
- 办理步骤；
- 截止时间；
- 官方入口与联系人；
- 来源和抓取/更新时间；
- 不确定性与失败边界。

首期不把 full-corpus RAG 作为 truth path。Affairs Navigator 先使用 stable tree/node、reviewed procedure Markdown 与 PostgreSQL structured search；查无 reviewed artifact 时，才对 Source Registry 中已批准的 exact official source 做 targeted refresh，并生成待管理员审核的 typed procedure candidate。Bounded RAG 只作为后续 approved snapshots 内的 recall fallback。

### G2｜校园变化监测

持续识别：

- 新通知；
- 截止时间变化；
- FAQ 或规则修订；
- 新附件或入口；
- 联系方式和资源变化。

输出不是简单 RSS 转发，而是带 diff、影响范围和来源的更新。

多个长期 maintainer Agent 按 stable board/node scope 维护 candidate，共用一份 source/revision/change ledger；它们没有 canonical publish authority。用户通过 per-board RSS/Atom 订阅 approved semantic changes，而不是订阅 raw crawl/hash noise。

### G3｜校园机会匹配与路径规划

将课程、科研、竞赛、讲座、实习、奖学金和校园资源表示为带资格、依赖与时间窗口的机会图；根据用户主动提供的最小画像，解释：

- 当前可申请什么；
- 为什么匹配；
- 缺少什么前置条件；
- 哪些机会存在时间冲突；
- 下一步可执行动作是什么。

## 3. 建议的公共内核

为了避免三个 feature 各做一套爬虫和问答，公共内核至少应包含：

1. **Source Registry**：来源身份、authority、revision、抓取方式、更新时间和适用范围；详见 [`source-registry.md`](source-registry.md)。
2. **Campus Graph**：事项、资格、组织、入口、时间窗口、依赖和机会之间的结构化关系。
3. **Temporal / Conflict Layer**：版本、变更、冲突来源和当前有效性。
4. **Provenance**：每项关键结论可追溯到具体来源与提取时间。
5. **Consent-aware Profile**：用户显式提供、可查看和可删除的最小个人画像。
6. **Tool / Capability Registry**：Agent 可用能力及权限边界。
7. **Evaluation Harness**：事实正确性、引用正确性、时效性、资格过滤和拒答行为。

## 4. PluginPackage 本体

Market 面向用户的唯一安装单位是：

```text
PluginPackage
├── McpServerComponent*
├── SkillComponent*
├── ControlledCliComponent*
├── SharedServiceBinding*
└── DeclarativeResourcePack*   # experimental package resource, no execution authority
```

一个纯 MCP/Skill 仍包装成单 component PluginPackage。`DeclarativeResourcePack` 首期只承载 source/tree/policy/schema/renderer 等 exact-pinned declarative assets，不是独立 installation component，也不获得执行权限。PluginPackage 只负责版本化组合、安装、启停和权限声明，不等于把 arbitrary code 加载进 Agent 主进程。

三个 default first-party Plugin 首期只声明 operator Capability Registry 中 `auto_grant_eligible=true` 的精确 read capabilities；新用户默认安装、启用并获得 manifest 的全部声明。自定义 mutation 统一走 auth/tenant/capability-gated Rust ControlledCLI tool call，不把 raw shell/script 权限交给 Plugin。

## 5. 推荐的产品分层

### Campus Trust Kernel

平台不可替代的核心：来源优先级、时效、冲突、权限、用户确认与 provenance。

### First-party campus plugins

用官方或公开校园场景证明平台价值，例如：

- 校园办事流程；
- 通知与规则变化；
- 课程/科研/竞赛机会；
- 107 算力平台与词元计划导航。

科大办事导航、USTC ChangeRadar 与 Campus Opportunity Graph 作为 `FirstPartySystemPlugin` 进入 Market；它们可组合 MCP、Skill 与 shared data service，而不是被迫退化成单一 MCP。

### Third-party extension surface

允许用户或组织增加新来源和能力，但必须经过 manifest、权限声明、schema 校验与隔离边界。

这一结构能避免两个极端：

- 只有通用 Agent 框架，没有 USTC 产品价值；
- 把所有校园功能硬编码成不可扩展的单体系统。

## 6. 当前最小价值闭环

工程顺序已冻结为 ChangeRadar foundation 先行、Affairs Navigator 用户入口随后；首个具体 board/source 仍待团队选择：

1. 用户询问一个真实校园目标；
2. 系统从 approved official source revision 构造 typed procedure candidate；
3. Rust validator/render 通过，管理员批准为 reviewed Markdown；
4. 后续官方 FAQ 或通知产生新 revision；
5. ChangeRadar 识别 semantic diff、affected scope 与 provenance；
6. approved change 进入 per-board RSS/Atom；
7. 同一 Agent 无需修改核心即可查询 current procedure 与 history。

完整语义见 [`affairs-changeradar-knowledge-architecture.md`](affairs-changeradar-knowledge-architecture.md)。

## 7. 当前非目标

在进一步确认前，不默认承诺：

- 修改教务、财务、学籍等权威系统；
- 自动执行高风险外部写操作；
- 读取未授权个人数据；
- 任意第三方代码在主后端进程内执行；
- 用一个月构建覆盖全部 USTC 服务的超级 App；
- 将普通搜索、RAG 或开源 Agent 换皮描述为原创系统。

## 8. 当前剩余阻塞项

PluginPackage 上层本体已明确。仍待确认的是：

- 团队是否接受该主项目候选并承诺最小 owner/capacity；
- USTC 统一认证 application/protocol access；
- 8C16G 比赛服务器的 container、network、DNS/TLS 与 backup 条件；
- hosted arbitrary user artifact 是否退出 MVP，或只保留 bounded dedicated spike；
- implementation repo 与 owner；
- Affairs 首个具体 board/source、authority order、crawl permission/rate policy。

## 9. 已补充的方向信息（2026-07-21）

- 公共 catalog/schema/source 位于 GitHub public；runtime/admin/secrets 仅内网。Publisher 通过 reviewed manifest 发布，用户安装 exact PluginPackage version。
- 用户自托管 plugin：代码、MCP 或 HTTP 服务在用户服务器或本地 companion 运行，平台受控连接。
- 技术方向：Rust；首期 Linux/Docker Compose Web + Android，未来考虑 iOS 与 Windows/Linux/macOS Desktop。
- Dioxus 是否采用及其 Fullstack 边界仍属架构候选，分析见 [`agent-track-architecture-options.md`](agent-track-architecture-options.md)。

## 10. Central authority 与 client relay 方向

- USTC 官方服务器完整保存并运行 Agent、memory、skills、默认 MCP、tasks 与 marketplace。
- Demo 中 AI provider 采用中央执行：平台模型使用 `OfficialCentral`，用户可将自定义 provider URL/key 明确上传为 encrypted `UserCloud` profile。MCP source 分为 `Official/VerifiedMarket | UserRemote | UserHostedPrivate`；tenant mode 与 cold/warm availability 由独立 runtime policy 决定。客户端 typed relay 仅作未来替换点。
- raw chat archive 可留在客户端，但中央仍需要当前 Agent turn 的 working context；durable memory 是独立中央对象。
- 自部署者部署完整 central stack；官方实例和 self-hosted instance 默认不形成双向同步。
- 总体边界见 [`central-agent-client-relay-marketplace.md`](central-agent-client-relay-marketplace.md)；MCP hosting 安全修正版见 [`mcp-market-hosting-policy.md`](mcp-market-hosting-policy.md)。
- Market/PluginPackage 的权威本体见 [`agent-market-architecture.md`](agent-market-architecture.md)；future repo docs 与 SSH/Slurm 执行蓝图见 [`project-documentation-and-execution-blueprint.md`](project-documentation-and-execution-blueprint.md)。
- 当前全局投影见 [`architecture-summary.md`](architecture-summary.md)。

AI provider 的完整 Demo policy 见 [`model-provider-policy.md`](model-provider-policy.md)。

MCP binding、tool grants 与 secret policy 见 [`mcp-binding-policy.md`](mcp-binding-policy.md)。
