# Interface registry

This registry names draft public surfaces before implementation. [`module-boundaries.md`](module-boundaries.md) owns the large-module crossing rules; this file owns the concrete application/API/tool surface registry. Implementation PRs must update this document or create a more specific contract before changing surfaces.

The implemented single-node Agent state/event contract is defined in [`agent-runtime.md`](agent-runtime.md). The planned finite user-task lifecycle is defined in [`agent-harness.md`](agent-harness.md). The Agent–Plugin seam is [`agent-plugin-boundary/v0`](agent-plugin-boundary.md). The future Dioxus Fullstack presentation boundary is [`client-shell/v1`](client-shell.md). None makes the application endpoints below operational.

## Application HTTP endpoints — draft

An endpoint may be implemented as a versioned Dioxus server function, an explicit public Axum route, or both when the same wire contract is intentionally public. The implementation class does not change admission or application ownership.

| Route | Method | Purpose | Status |
|---|---:|---|---|
| `/api/health` | GET | service health and version | planned |
| `/api/market/packages` | GET | list visible packages | planned |
| `/api/market/packages/{id}` | GET | package details | planned |
| `/api/installations` | POST | install exact package version with grants | planned |
| `/api/installations/{id}:disable` | POST | disable installed package | planned |
| `/api/agent/runs` | POST | create one finite HarnessRun from typed user intent | planned |
| `/api/agent/runs/{id}` | GET | read phase, accepted graph projection, evidence and blockers | planned |
| `/api/agent/runs/{id}/answers` | POST | submit answers to the current bounded clarification gate | planned |
| `/api/agent/runs/{id}:cancel` | POST | request typed cancellation under current phase/effect semantics | planned |
| `/api/agent/runs/{id}/events` | GET/SSE | stream harness/node/model/tool/review state projections | planned |

First-party Dioxus Web/Android clients consume admitted M10 ingress through a typed `ClientApi` facade whose implementation SHOULD use generated server-function calls and typed stream handles. A server function is an Axum-compatible M10 adapter: after version, bounds, identity, authorization, idempotency/precondition and audit admission, it may call one public application command/query port. It MUST NOT call concrete repositories, databases, Plugin executors, provider SDKs or journals directly. Public/heterogeneous routes are peer transport adapters over the same application ports, not duplicate business implementations.

Every independently deployed request carries client build/target/protocol identity. Unsupported Android/server combinations return a typed compatibility or `UpgradeRequired` outcome before application dispatch. Shared Rust source does not waive deployed-version compatibility.

## Agent tool protocol — H0 values implemented, production execution planned

| Object | Direction | Purpose |
|---|---|---|
| `AgentToolsetView` | resolver/gateway → Agent | immutable per-turn complete tool definitions plus opaque private route references |
| `AgentToolCall` | Agent → ToolGateway | provider-neutral correlated call against the exact frozen projection |
| `PluginExecutionRequest` | ToolGateway → PluginExecutor | authorized bounded execution request after effect intent persistence |
| `PluginExecutionOutcome` | PluginExecutor → ToolGateway | non-authoritative bounded outcome for validation and receipt persistence |
| `AgentToolResult` | ToolGateway → Agent | correlated bounded result/evidence/receipt projection for the next model turn |

`AgentToolsetView`, `AgentToolCall` and the digest/code subset of `AgentToolResult` are concrete Rust types in `crates/agent-tool-protocol` and are exercised by a composition-root fake gateway/executor. `PluginExecutionRequest`, transport, durable receipt composition, bounded content/artifact result expansion and any HTTP/MCP wire format remain planned; no generic extension ABI is implied.

## MCP/tool surface — Course Planning draft

| Tool | Purpose | Mutates external systems |
|---|---|---:|
| `plan.list` | list available plans | no |
| `plan.get` | get plan revision | no |
| `course.search` | search normalized courses | no |
| `course.get` | course detail/provenance | no |
| `review.linkout` | return iCourse link-out metadata | no |
| `offering.list` | list imported/approved offerings | no |
| `profile.requirement_status` | compute progress against a plan | no |
| `planner.generate` | create tenant-local plan candidates | tenant draft only |
| `planner.explain` | explain candidate rationale | no |
| `source.provenance` | show evidence chain | no |
