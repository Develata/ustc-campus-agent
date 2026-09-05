# 讨论决策与文档落点

> 类型：导航与有界对照清单；不是新的产品/合同权威，也不是原始聊天归档。核对日期：2026-09-06 CST；产品源码基线：`2f03a13c3ec03eab791f1728e2c2973ed82c3e50`。

## 1. 本次核对回答什么

本页将已确认的产品方向、关键功能和近期交付决定映射到 owning docs，并区分：

- **已记录**：正式设计/契约存在，不等于实现或验收通过；
- **有限实现**：main 有对应代码和有限证据，不升级整个模块状态；
- **未合并**：只存在于远端分支/PR，不能算 main 的能力；
- **待实现/待验收**：明确保留缺口，不用展示、声明或 mock 替代。

本次范围是已识别主题及其直接文档投影，不声称每段历史聊天、每个早期设想均逐句审计。原始探索可能包含废弃设计、个人数据和未批准操作；它们不应整份复制入公开仓库。历史设计保留理由，当前 plans/contracts 决定行为，`acceptance/matrix.tsv` 决定当前验收绑定。本页只链接，不创建第二套状态权威。

## 2. 主题对照

| ID | 已讨论/确认的主题 | owning docs 与相关投影 | 当前真实状态/未关闭范围 |
|---|---|---|---|
| D01 | Plugins Market + bounded campus Agent，而非通用 Agent 框架 | [产品定位](../plan/02-product-positioning.md)、[Market 生命周期](../plan/04-market-and-plugin-lifecycle.md)、[Market feature](../features/00-market-browse-install.md) | 定位与 package/install/grant/enable/update 边界已记录；完整可用 Market 安装、动态禁用/撤权目录与隔离执行不能由固定四工具 demo 证明 |
| D02 | 三个首方插件保留独立产品身份；课程规划不是唯一旗舰 | [首方规划](../plan/06-first-party-plugins.md)、[ADR-0006](../adr/0006-three-default-first-party-plugins.md)、[覆盖矩阵](../coverage-matrix.md) | Affairs Navigator、ChangeRadar、Opportunity Graph 已入 main；固定 demo 有可用路径，不代表各自完整 package 生命周期全部完成 |
| D03 | 服务端权威、四层调用结构、模块独立、薄客户端 | [平台权威](../plan/03-platform-authority.md)、[模块边界](../contracts/module-boundaries.md)、[模块图](../plan/modules/00-module-map.md) | 已记录；UI、模型、框架缓存不得变成 grant/source/receipt 的第二权威 |
| D04 | 参考 Rig、Goose、Pi、LangGraph 等，但不让框架统治领域 | [运行时规划](../plan/07-runtime-and-integration.md)、[运行时参考 ADR](../adr/0004-runtime-reference-strategy.md)、[Agent–Plugin 边界](../contracts/agent-plugin-boundary.md) | 参考与采用边界已记录；参考过某框架不等于依赖已引入或全平台已实现 |
| D05 | 有限 Harness、TaskGraph、上下文预算、compaction/compression、证据与恢复 | [Harness contract](../contracts/agent-harness.md)、[Runtime contract](../contracts/agent-runtime.md)、[Harness feature](../features/04-bounded-agent-harness.md) | 长期设计与部分内核已记录；当前 demo 是有限 Chat coordinator，不是完整 durable Harness、多 Agent、长期记忆或 RAG |
| D06 | 来源身份、修订、审核、时效、冲突、不确定性；先可信结构化来源 | [Campus Trust](../plan/05-campus-trust-kernel.md)、[Source import](../contracts/source-import.md)、[Source retrieval](../contracts/source-retrieval.md) | 已有源注册/离线策略和 reviewed fixture 证据；联网抓取许可、真实源激活与 complete baseline 仍不可自动宣称 |
| D07 | Affairs 查询、个人办理步骤、ChangeRadar 语义变更与 feed | [Affairs feature](../features/01-ustc-affairs-navigator.md)、[Radar feature](../features/02-ustc-change-radar.md)、[MVP capability](../features/06-mvp-core-capabilities.md) | 固定 reviewed 查询/发布路径已有有限实现；个人勾选不代表学校受理，demo feed 不代表全校园实时监控 |
| D08 | 基于培养方案与 iCourse 的选课辅助；官方硬约束优先 | [Opportunity feature](../features/03-campus-opportunity-graph.md)、[M72 蓝图](../plan/modules/73-opportunity-graph.md)、[MVP capability](../features/06-mvp-core-capabilities.md) | 来源分级、显式同意、确定性 planner 已记录/有限实现；当前 synthetic catalog + 公开聚合信号不是实时完整培养方案，不自动选课、不把评论当官方事实 |
| D09 | M72 私有操作采用静态应用组合，而非伪造 Agent/Plugin 执行链 | [接口 registry](../contracts/interfaces.md)、[M72 蓝图](../plan/modules/73-opportunity-graph.md)、[覆盖矩阵](../coverage-matrix.md) | M00/M10 admission 后当前 M20 再授权、M72 owning use case 已记录；Chat 调用 planner 不把 profile/consent authority 转移给模型 |
| D10 | Web、Docker Compose server、Android；headless CLI 与外部 Agent 接入 | [Client contract](../contracts/client-shell.md)、[CLI contract](../contracts/cli.md)、[Headless feature](../features/05-headless-client-and-agent-integration.md)、[ADR-0010](../adr/0010-typed-client-peer-adapters.md) | Affairs-first protocol/client-core/普通用户 CLI 有有限证据；完整 Dioxus peers、inbound MCP、streams、跨宿主矩阵仍未完成；Windows launcher 不等于 Windows GUI 产品 |
| D11 | Agent 普通问答、四工具、provider、prompt 偏好与安全边界 | [Chat contract](../contracts/agent-chat.md)、[MVP capability](../features/06-mvp-core-capabilities.md) | mock/受限 OpenAI-compatible adapter、request-scoped prompt preference 与有限轮次已实现；无任意命令 sandbox、Skill runtime、可用 inbound/outbound MCP、持久聊天、streaming 或 provider fallback 的完成声明 |
| D12 | Simple Calendar 的显式记录/删除和重启持久化 | [MVP capability](../features/06-mvp-core-capabilities.md)、[Chat contract](../contracts/agent-chat.md)、[验收矩阵](../acceptance/matrix.tsv) | owner-local durable state 和精确意图语法已实现；提醒、重复、CalDAV、自然语言日期及多端同步未实现；用户机器持久化仍需复验 |
| D13 | synthetic 草稿、方案比较、个人清单 Markdown、场景引导 | [可用性增强契约](../contracts/usable-demo-enhancements.md)、[MVP capability](../features/06-mvp-core-capabilities.md)、[验收矩阵](../acceptance/matrix.tsv) | UE-001/002/003 已记录且在 main；草稿/勾选页面内有效，不暗示保存真实学籍或跨端同步；场景按钮不自动提交/授权 |
| D14 | 学校 SSO 的授权/配置边界，以及独立接口预留 | [增强契约](../contracts/usable-demo-enhancements.md)、[SSO 样例](../../examples/sso-interface/README.zh-CN.md)、[M00 蓝图](../plan/modules/10-platform-control-identity.md) | 禁用状态/start/callback 接口及 SSO-001 已在 main；没有校园登录、身份验证或应用会话签发，未来授权仍需真正协议/安全/会话集成 |
| D15 | 更完整的 chat-first UI、本地密码登录和账号管理 | [当前 Chat contract](../contracts/agent-chat.md)、[当前客户端契约](../contracts/client-shell.md)、[候选 PR #72](https://github.com/Develata/ustc-campus-agent/pull/72) | 截至本页基线，#72 为 OPEN/Draft：Argon2id 管理员密码/session/轮换及其 shell 变更已 push 到候选，但未合并；不属于 R3.1，本次不 cherry-pick 或合并 |
| D16 | 早期完整 UI 设计：信息层次、组件、响应式、onboarding、grant diff/activity | [M80 design packet](../design/m80-default-v0/README.md)、[设计规则](../design/AGENTS.md) | 已记录为绑定旧基线的 Reviewed 设计包；Reviewed 不等于已实现；不能把旧 prototype 当当前页面或静默用它覆盖现有用户流程 |
| D17 | Windows 启动修正、源码身份、最终包校验和未完成验收 | [R3.1 交付与复验](r31-delivery-and-verification.md)、[Chat contract](../contracts/agent-chat.md) | #77 已合并；runtime/APK 保持 R3 原字节、launcher 单独标来源；实机、视频、门户等缺口明确保留 |

## 3. 本次补漏与未处理项

本次仅做文档投影修正：

- 将增强测试的旧 `demo-enhancements` workflow 名称改为真实的 `.github/workflows/ci.yml`；不改 CI 本身。
- 将 Headless feature 的“完全未实现”旧描述改为 Affairs-first 的已留存有限证据，并保留 CLIENT-007 至 CLIENT-010 全部 `planned`；不提升验收状态。
- 同步其直接 client/CLI/interface/permission 投影：分清普通用户与 operator mutation、active planned 与 long-horizon、已有 loopback HTTP 与未完成 remote transport；不改协议或权限。
- 将旧 UI design 入口的 proposal-only 描述改为实际状态体系，仍不把 Reviewed 当成实现。
- 将 #77 已实现的 native stdout/exit-code 次序同步到 Chat contract；不修改 launcher 或 runtime。
- 新建本索引与 R3.1 接收/复验入口，分清 main、候选 PR、runtime、launcher、builder 与材料包。
- 覆盖矩阵全文受既有 checker 指纹绑定，因此只同步该文档 SHA-256 及相应 checker 函数的派生 AST digest；不修改断言、检测逻辑、状态或 CI。

不在本次范围：

- 全量历史聊天逐句存档及无法从已接受 carrier 判定的旧想法；新增产品或 authority 选择仍需单独确认。
- #72、#68、#66、#22 等非本轮交付 PR 的合并、实现或重新启用。
- 用 documentation PASS 代替 real-host、real-provider、物理 Android 或门户提交证据。
- 重打 R3.1、建立 tag/Release、发布 runtime 或修改私人主机状态。

仍需单独处理的工作流文本：`tasks/00-module-work-policy.md` §1 保留旧的固定前端设计模型分工；当前协作环境不具备相应已配置模型，不能据此要求接手者探测或依赖它们。本次不改写模型/授权治理规则，保留为后续专门治理修订项，而非产品功能缺失。

**数据使用缺口也未被本次补文档关闭**：既有 iCourse aggregate-rating fixture 超过纯 link-out；现行安全计划要求的额外 data-use contract 尚未在本次核对中得到证明。已在 [公开就绪清单](../acceptance/public-readiness.md) 与 [MVP 功能说明](../features/06-mvp-core-capabilities.md) 显式登记，不虚构许可、不改动已交付 fixture。后续须由 owner 确认许可依据与范围，或另行选择 synthetic-only 替代。

## 4. 接手者如何继续

- 想知道“当初决定什么”：先从主题找到 plan/contract；不要从历史聊天片段反推新 authority。
- 想知道“代码做到哪”：同时看 feature、module map 和 active acceptance；`planned`、partial evidence 都不是完整通过。
- 想知道“我拿到的包是什么”：只看固定包身份与 checksum；GitHub main 前进不会更新已下载的包。
- 新发现遗漏时：附上原始决策的脱敏摘要、拟归属的现有 owner、当前实现/验收状态，先处理 owning carrier，再更新本导航；不要复制多个互相竞争的完整 spec。

本页不提供“所有讨论均无遗漏”的百分比。它提供可核对的落点和未关闭项，使后续交接不依赖仅存在于会话中的口头结论。
