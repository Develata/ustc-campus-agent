# Agent Runtime 采用、参考与 License Policy

- 状态：**推荐方案；待 disposable spike 后冻结**
- 更新时间：2026-07-21
- 项目许可证目标：MIT（仅覆盖本项目原创代码）
- 适用对象：USTC 个人校园 Agent 平台

## 1. 结论

不建议在“fork 一个成熟 Python/TypeScript Agent framework”和“所有 Agent 机制从零用 Rust 重写”之间二选一。

推荐第三条路线：

> **原创、窄而可验证的 Rust domain/control core；Rig 与 rmcp 作为可替换 infrastructure dependencies；OpenAI Agents SDK、LangGraph、Microsoft Agent Framework、goose 与 Pi 作为显式声明的 architecture references。**

简写为：

```text
Own the state machine and policy.
Reuse protocol and provider plumbing.
Reference mature lifecycle designs.
Do not fork a foreign product architecture.
```

## 2. 为什么不 fork 完整成熟框架

当前候选的成熟 framework 主要是 Python 或 .NET/Python。直接 fork 会带来：

- backend language 与 Rust-first 目标冲突；
- 中央多租户、MCP hosting、校园 identity、marketplace 与 client protocol 仍需重写；
- domain types 容易被 framework session/message/tool types 侵入；
- 上游升级、branding 与迁移负担高；
- 比赛时间被消耗在删除不需要的通用能力；
- 最终创新叙事容易变成“换皮部署某框架”。

成熟框架最有价值的部分是其已经踩过的 lifecycle、checkpoint、streaming、guardrail、multi-tenancy 与 observability 坑，而不是其全部代码树。

## 3. 为什么也不应从零重写所有轮子

完全自行实现 provider SDK、stream parser、MCP protocol、tool adapter 与 model compatibility，会把时间浪费在低差异化 plumbing 上，并增加：

- protocol 偏差；
- cancellation/streaming 不一致；
- provider-specific edge cases；
- tool-call parsing 与 timeout bug；
- license/provenance 反而更难追踪的复制风险。

因此“原创 Rust core”不等于“零依赖”。原创范围应是平台特有的不变量与控制流。

## 4. 截至 2026-07-21 的候选核查

| 项目 | 当前 repo/release | GitHub license signal | 本项目用途 | 判断 |
|---|---|---|---|---|
| [Rig](https://github.com/0xPlaygrounds/rig) | Rust，`v0.40.0` | MIT | provider abstraction、agent/tool runner、hooks、MCP adapter 候选 | **直接 dependency 首选，但必须 adapter 隔离** |
| [MCP Rust SDK / rmcp](https://github.com/modelcontextprotocol/rust-sdk) | Rust，`rmcp-v2.2.0` | repo 处于 MIT→Apache-2.0 transition；crate metadata 为 Apache-2.0 | released MCP protocol implementation | **直接 dependency；保留其适用 license/notices** |
| [OpenAI Agents SDK Python](https://github.com/openai/openai-agents-python) | Python，`v0.18.3` | MIT | run loop、guardrail ordering、handoff、HITL、stream/cancel parity | **architecture reference，不 fork** |
| [LangGraph](https://github.com/langchain-ai/langgraph) | Python，`1.2.9` | MIT | durable state graph、checkpointer/store、interrupt/resume | **architecture reference，不 fork** |
| [Microsoft Agent Framework](https://github.com/microsoft/agent-framework) | Python/.NET，`python-1.11.0` | MIT | middleware、workflow/checkpoint、OTel、per-user session isolation | **architecture reference，不 fork** |
| [goose](https://github.com/aaif-goose/goose) | Rust，`v1.43.0` | Apache-2.0 | Rust agent product、provider/MCP extension、CLI/API product boundary | **product/reference only，不 fork** |
| [Pi](https://github.com/earendil-works/pi) | TypeScript，`v0.80.10` | MIT | minimal core、registries/hooks/resources/packages、append-only session/durable boundary | **architecture reference，不作为 runtime dependency** |
| [PydanticAI](https://github.com/pydantic/pydantic-ai) | Python，`v2.14.1` | MIT | typed dependency/output、eval 思路 | 可选参考 |
| [AutoGen](https://github.com/microsoft/autogen) | Python，`python-v0.7.5` | repo root docs 为 CC-BY-4.0；`LICENSE-CODE` 为 MIT | 历史 multi-agent design | MAF 已提供迁移路径，非首选基座 |

当前 release 与 license 需要在真正加入依赖时重新锁定；上表不是永久 compatibility guarantee。

## 5. 各参考项目真正值得借鉴的内容

### 5.1 Rig：Rust infrastructure dependency

Rig 当前提供：

- unified model provider interface；
- multi-turn prompt/streaming；
- tool server 与 hook stack；
- conversation memory abstraction；
- OpenTelemetry GenAI semantics；
- rmcp adapter、tool-list refresh、per-call timeout/cancellation；
- blocking 与 streaming 共用 runner/state logic 的设计。

但其 README 明确提醒未来更新会有 breaking changes。因此：

- 不在 domain 层暴露 Rig types；
- pin exact crate version 与 `Cargo.lock`；
- 只启用必要 features；
- 在 adapter contract tests 中固定我们依赖的语义；
- 不让 Rig 的 memory 成为平台 canonical memory；
- 不让 Rig 的 tool registry 取代本项目 grant/policy engine。

### 5.2 OpenAI Agents SDK：run lifecycle reference

值得借鉴：

- 一个 model turn 的清晰定义；
- tool execution、handoff、interrupt/resume 不应错误重复计数；
- input/tool/output guardrail 的严格顺序；
- `RunAgain | Handoff | FinalOutput | Interruption` 的显式 state transition；
- streaming 与 non-streaming 必须有相同 final state 和 side effects；
- cancellation request 不等于 cleanup 已完成；
- resume 不重放已完成的 tool side effect。

我们可以采用这些原则，但不复制 Python implementation。

### 5.3 LangGraph：durable execution reference

值得借鉴其明确分离：

```text
Checkpointer = thread/run-scoped execution state
Store        = cross-thread durable application memory
```

这与本项目的边界一致：run checkpoint、raw transcript、semantic memory、task state 不能混成一个“memory”。

借鉴 durable execution、interrupt、resume 与 fault recovery；不引入 LangGraph/LangSmith 作为中央 authority。

### 5.4 Microsoft Agent Framework：workflow 与 tenant isolation reference

值得借鉴：

- middleware 与 OpenTelemetry boundary；
- graph workflow、checkpoint、HITL；
- hosted per-user session storage 采用 physical partition + identity check 的 defense-in-depth；
- approval mapping 随 checkpoint 分区；
- hosted production 缺 tenant identity 时 fail closed。

这直接支持本项目原则：任何 Agent session、tool approval、MCP session 与 task checkpoint 都必须显式带 `user_id`，不能只依赖猜不出的 conversation ID。

### 5.5 goose：Rust product architecture reference

goose 是 Rust native desktop/CLI/API、multi-provider、MCP extension 的成熟产品参考；适合观察：

- provider/extension 配置边界；
- native client 与 API separation；
- MCP-driven extension UX；
- distribution 与 diagnostics。

但它是 local-first general-purpose agent，而本项目是 central multi-tenant campus platform；产品 authority 和 threat model 不同。其 Apache-2.0 code 也不应被复制后重新标记为 MIT。

### 5.6 Pi：extension/package/session architecture reference

Pi 值得借鉴：

- minimal agent core 与 tool/command/provider/resource registries 分离；
- extensions、skills、prompts/themes 作为不同 resource classes，由 package 组合；
- observer、transform/block lifecycle semantics；
- progressive disclosure skills；
- CLI/JSON/RPC/SDK 共用 core；
- append-only session、stable IDs 与 host-supplied runtime implementations；
- crash recovery 从 durable boundary 恢复，不默认重跑 non-idempotent tool。

但 Pi 官方明确其 extension/package 默认继承启动进程完整权限，且 `tool_call` input mutation 后不重新 validation。这些语义不能进入本项目 central multi-tenant plane：

- no arbitrary TypeScript/in-process hot-load；
- no full-process filesystem/network/credential inheritance；
- no install-time third-party lifecycle scripts；
- duplicate resource/tool ID fail-closed，不 silent first-wins；
- Transformer 修改 typed input 后必须重新 schema validation + authorization；
- Observer 可按 policy fail-open，security/publish Gate 必须 fail-closed；
- privileged executable extension 只能经 reviewed component、exact digest、grant 与 isolated runtime。

本项目 hook contract 分为：

```text
Observer     read-only event observation
Transformer sequential typed transform, then revalidation
Gate         allow/deny/review; security gate fails closed
Registry     tools/connectors/renderers/resources; not a hook
```

Pi 只记录为 influence，不复制其 TypeScript implementation。

## 6. 推荐 owned architecture

```text
apps/
├── server                 # HTTP/realtime API + authority
├── worker                 # tasks, reviews, indexing
├── mcp-runtime-controller # narrowly privileged deployment controller
└── web-mobile-ui          # replaceable client shell

crates/
├── domain                 # entities, invariants, state transitions
├── agent-runtime-port     # framework-neutral AgentEngine trait
├── agent-runtime-rig      # Rig adapter; no domain ownership
├── model-provider         # platform provider profile/policy
├── mcp-protocol-adapter   # rmcp wrapper
├── mcp-gateway            # bindings, grants, call policy
├── run-state              # durable run/checkpoint state machine
├── authorization          # user/tool/capability grants
├── task-engine            # schedule, retry, receipts
├── memory                 # semantic memory contracts
├── extension-contract     # typed Observer/Transformer/Gate/Registry contracts
├── knowledge              # source/procedure/change domain contracts
├── api-contract           # versioned client protocol
└── audit                  # provenance and receipts
```

### Hard boundary

`domain`、`run-state`、`authorization`、`task-engine` 不得 import Rig/rmcp/provider SDK types。

建议 port：

```text
trait AgentEngine {
  start(run_spec) -> event_stream
  resume(run_id, decision) -> event_stream
  cancel(run_id, mode) -> cancel_receipt
}
```

Infrastructure adapter 可替换；durable run state 由本项目拥有。

## 7. 推荐 run state machine

```text
Created
  -> Preparing
  -> ModelTurn
  -> AwaitingToolApproval
  -> ExecutingTools
  -> ModelTurn
  -> Completed

terminal alternatives:
  Failed
  Cancelled
  Expired
```

每个 transition 产生 immutable event 与 checkpoint。必须满足：

- streaming 是同一 state machine 的 projection，不是第二套执行逻辑；
- tool side effect 前持久化 invocation/approval identity；
- side effect 完成后持久化 receipt，再进入下一 model turn；
- crash/resume 不重复执行已有 successful receipt 的 tool；
- cancel 分 `immediate` 与 `after_turn`；
- max turns、token/cost/tool budgets 持久化，不因 resume 重置；
- current user、agent、provider profile、MCP grant version 全部进入 run spec。

## 8. Rig adoption spike

正式采用 Rig 前做 disposable spike，不先创建大仓库骨架。

### 必须验证

1. `OfficialCentral` 与 arbitrary OpenAI-compatible `UserCloud` provider；
2. token streaming、usage、timeout 与 cancellation；
3. tool call hook 能在 side effect 前执行平台 policy/confirmation；
4. rmcp Streamable HTTP、dynamic tool-list refresh 与 schema snapshot；
5. blocking/streaming 得到一致 final state；
6. platform-owned run ID、user ID 与 audit context 可贯穿 hooks；
7. provider/MCP error 能归一为 typed domain error；
8. no prompt/tool content telemetry by default；
9. Rig upgrade 不要求 domain/API schema 变化；
10. crash/resume 由 platform checkpoint 实现，而非假定 Rig memory 自动提供 exactly-once。

### Pass

若 1–10 均可在 adapter 内完成，采用 Rig。

### Fallback

若 Rig 无法稳定支持 custom provider、pre-tool policy、stream cancellation 或 typed event projection，则保留同一 `AgentEngine` port，改用 `reqwest + provider-specific adapters` 实现窄 agent loop。**Fallback 不是 fork Python framework。**

## 9. MIT 顶层许可证的准确含义

项目根 `LICENSE` 可以使用 MIT，但它只许可本项目原创代码；不会把 dependencies、vendored code、copied snippets、docs 或 assets 自动重新许可为 MIT。

### 必须建立

```text
LICENSE
THIRD_PARTY_NOTICES.md
docs/architecture/influences.md
```

- `LICENSE`：本项目原创代码的 MIT license；
- `THIRD_PARTY_NOTICES.md`：runtime/build dependencies、license、copyright、source；
- `influences.md`：architecture references、核查日期、精确 repo/release/commit、借鉴概念、是否复制代码。

### MIT code 被复制或修改时

MIT 要求保留原版权与许可声明。对 adapted file 建议写：

```text
Adapted from <repo>/<path>@<commit>.
Copyright <upstream holder>.
Used under the MIT License; see THIRD_PARTY_NOTICES.md.
Modifications Copyright <project contributors>, MIT.
```

不得只保留本项目 MIT 头而删除 upstream notice。

### 只借鉴思想、不复制表达或代码时

建议声明：

```text
Architecture influenced by <project> <release/commit>, especially <concepts>.
No source code was copied from that implementation.
```

这既满足透明性，也避免虚构“完全独立发明”。

### Apache-2.0 dependency/code

- 可以在 MIT 主项目中依赖 Apache-2.0 package；
- dependency 继续适用其 Apache-2.0 license；
- 必须保留适用的 LICENSE/NOTICE 和 attribution；
- 若复制/修改 Apache source，该部分不能被简单重新标成纯 MIT；
- goose 与 rmcp 的代码处理应遵循其精确 revision 中的 license 文件。

### CC-BY documentation

若改写/复制 CC-BY 文档内容，需要按其要求 attribution；只阅读其设计并重新独立表达时，也建议在 influences 文档中透明注明。

以上是工程合规建议，不是法律意见。

## 10. 自动化 license/provenance gate

进入实现后建议：

- `Cargo.lock` commit；
- `cargo-deny` license allowlist；
- SBOM 生成；
- dependency source/revision lock；
- vendored/copy-pasted code scanner 与人工 review；
- release artifact 附 `LICENSE` 与 `THIRD_PARTY_NOTICES.md`；
- architecture influence 文档随重大设计更新；
- 禁止从 license 不明 repo 复制代码；
- 对 dual/mixed/transition license 记录精确 path 与 revision。

## 11. 最终推荐

```text
Fork OpenAI Agents/LangGraph/MAF: No
Fork goose:                         No
All agent/protocol plumbing from 0: No
Own Rust domain + run state:        Yes
Rig behind adapter:                 Yes, after spike
rmcp behind adapter:                Yes, pin release/license
Explicit architecture attribution: Must
Top-level project license:          MIT
```

这条路线同时满足：

- Rust-first；
- 比赛期可交付；
- 核心创新与 policy ownership 清晰；
- 不复制大框架的历史包袱；
- 能诚实声明参考；
- 将上游 breaking changes 限制在 adapter 内。

## 12. 事实来源

- Rig repository：<https://github.com/0xPlaygrounds/rig>
- Rig agent runner：<https://github.com/0xPlaygrounds/rig/blob/main/crates/rig-core/src/agent/runner.rs>
- Rig rmcp adapter：<https://github.com/0xPlaygrounds/rig/blob/main/crates/rig-core/src/tool/rmcp.rs>
- MCP Rust SDK：<https://github.com/modelcontextprotocol/rust-sdk>
- OpenAI Agents SDK Python：<https://github.com/openai/openai-agents-python>
- OpenAI runner lifecycle：<https://github.com/openai/openai-agents-python/blob/main/.agents/references/runner-lifecycle.md>
- LangGraph repository：<https://github.com/langchain-ai/langgraph>
- LangGraph persistence：<https://docs.langchain.com/oss/python/langgraph/persistence>
- Microsoft Agent Framework：<https://github.com/microsoft/agent-framework>
- MAF per-user session isolation ADR：<https://github.com/microsoft/agent-framework/blob/main/docs/decisions/0031-hosted-per-user-session-storage-isolation.md>
- goose repository：<https://github.com/aaif-goose/goose>
- PydanticAI repository：<https://github.com/pydantic/pydantic-ai>
- AutoGen repository：<https://github.com/microsoft/autogen>
