# Interface registry

This registry names draft public surfaces before implementation. Implementation PRs must update this document or create a more specific contract before changing surfaces.

The implemented single-node Agent state/event contract is defined in [`agent-runtime.md`](agent-runtime.md). The planned finite user-task lifecycle is defined in [`agent-harness.md`](agent-harness.md). The Agent–Plugin seam is [`agent-plugin-boundary/v0`](agent-plugin-boundary.md). None makes the HTTP routes below operational.

## HTTP routes — draft

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

## Agent tool protocol — planned

| Object | Direction | Purpose |
|---|---|---|
| `AgentToolsetView` | resolver/gateway → Agent | immutable per-turn complete tool definitions plus opaque private route references |
| `AgentToolCall` | Agent → ToolGateway | provider-neutral correlated call against the exact frozen projection |
| `PluginExecutionRequest` | ToolGateway → PluginExecutor | authorized bounded execution request after effect intent persistence |
| `PluginExecutionOutcome` | PluginExecutor → ToolGateway | non-authoritative bounded outcome for validation and receipt persistence |
| `AgentToolResult` | ToolGateway → Agent | correlated bounded result/evidence/receipt projection for the next model turn |

These are logical `agent-tool-protocol/v0` surfaces. Concrete Rust types land with the first H0 fake-gateway consumer; no HTTP/MCP wire format or generic extension ABI is implied yet.

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
