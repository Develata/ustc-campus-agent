# Interface registry

This registry names accepted or draft application, API and tool surfaces before implementation. [`module-boundaries.md`](module-boundaries.md) owns large-module crossing rules; this file owns concrete operation/transport/tool surface registration. Implementation PRs must update this document or a more specific owning contract before changing surfaces.

The implemented single-node Agent state/event contract is defined in [`agent-runtime.md`](agent-runtime.md). The planned finite user-task lifecycle is defined in [`agent-harness.md`](agent-harness.md). The Agent–Plugin seam is [`agent-plugin-boundary/v0`](agent-plugin-boundary.md). The future typed multi-client boundary is [`client-shell/v2`](client-shell.md). None makes the application endpoints below operational.

## M20 application operations — A1 implemented as semantic in-memory evidence with zero production callers; M10 mapping/wire/client admission and production caller remain planned

[`market-lifecycle/v0`](market-lifecycle.md) owns the complete Rust item/signature/privacy/error contract. This registry binds each application operation to that surface:

| Operation | Request | Result | Status |
|---|---|---|---|
| `BrowseCatalog` | `CatalogBrowseQuery` | `MarketCatalogPage` | A1 implemented as semantic in-memory evidence with zero production callers; M10 mapping/wire/client admission and production caller remain planned |
| `ReadPackageDetail` | `CatalogPackageQuery` | `MarketPackageDetail` | A1 implemented as semantic in-memory evidence with zero production callers; M10 mapping/wire/client admission and production caller remain planned |
| `ReadOwnedInstallation` | `OwnedInstallationQuery` | `MarketInstallationView` | A1 implemented as semantic in-memory evidence with zero production callers; M10 mapping/wire/client admission and production caller remain planned |
| `ReadOwnedCurrentGrants` | `OwnedInstallationGrantQuery` | `MarketGrantPage` | A1 implemented as semantic in-memory evidence with zero production callers; M10 mapping/wire/client admission and production caller remain planned |
| `ReadOwnedPackageUpdate` | `OwnedUpdateQuery` | `MarketUpdateView` | A1 implemented as semantic in-memory evidence with zero production callers; M10 mapping/wire/client admission and production caller remain planned |
| `DisableOwnedInstallation` | `DisableInstallationRequest` | `DisableInstallationReceiptView` | A1 implemented as semantic in-memory evidence with zero production callers; M10 mapping/wire/client admission and production caller remain planned |
| `ResolveToolProjection` | existing M20 projection request | existing `ToolProjectionSnapshot`/`AgentToolsetView` mapping | bounded B5/P0a implementation; no A1 wrapper |
| `RecheckInvocationAuthority` | existing frozen projection + proposed call | existing `AuthorizedInvocation` or typed denial | bounded B5/P0a implementation; no A1 wrapper |

A1's exact new public set is the application error, limit, request/query, safe view/result, catalog port/fake and service types enumerated in the lifecycle contract; this registry creates no second numeric inventory. No application value is a wire DTO or implements Serde. Safe views expose no raw source-policy map, configuration entry, `SecretRef`, `ExecutionIdentity`, approval/evidence carrier, private route or event history.

The owner-scoped tenant/user values are downstream scope claims, not request admission. A1 has zero production call sites. A later M10 adapter may map them only from current M00-admitted context; client body/query/header identity is forbidden. Absence and owner mismatch both map to `NotFound`. `DisableOwnedInstallation` delegates one existing owner command after ownership check and preserves owner-ledger-first idempotent replay.

Contract acceptance adds no M10 route/server function. The HTTP table below remains independently planned; in particular, `/api/installations/{id}:disable` cannot call A1 until M00/M10 admission and wire contracts are accepted.

## Application HTTP endpoints — draft

An endpoint may be implemented as a versioned Dioxus server function, an explicit public Axum route, or both when the same wire contract is intentionally public. The implementation class does not change admission or application ownership.

| Route | Method | Purpose | Status |
|---|---:|---|---|
| `/api/health` | GET | service health, build and protocol version | planned |
| `/api/client/capabilities` | GET | bounded client-protocol and selected user-capability projection; no operator registry | planned |
| `/api/market/packages` | GET | list visible packages | planned |
| `/api/market/packages/{id}` | GET | package details | planned |
| `/api/installations` | POST | install exact package version with grants | planned |
| `/api/installations/{id}:disable` | POST | disable installed package | planned |
| `/api/agent/runs` | POST | create one finite HarnessRun from typed user intent | planned |
| `/api/agent/runs/{id}` | GET | read phase, accepted graph projection, evidence and blockers | planned |
| `/api/agent/runs/{id}/answers` | POST | submit answers to the current bounded clarification gate | planned |
| `/api/agent/runs/{id}:cancel` | POST | request typed cancellation under current phase/effect semantics | planned |
| `/api/agent/runs/{id}/events` | GET/SSE | stream harness/node/model/tool/review state projections | planned |

M80 Dioxus Web/Android, `ustc-agent` and inbound MCP adapters are peer clients over one M80-owned framework-neutral typed client core. M10 owns the versioned wire DTO/error/event/compatibility schema, represented by a framework-neutral `client-protocol` carrier when extracted; M80 core consumes that carrier and M10 never depends on client-core. Dioxus SHOULD use generated server-function calls and typed stream handles where those preserve the same contract. `ustc-agent`, inbound MCP and other admitted heterogeneous clients use the explicit endpoint subset above. A server function or HTTP/SSE route is an Axum-compatible M10 adapter: after version, bounds, identity, authorization, idempotency/precondition and audit admission, it may call one public application command/query port. It MUST NOT call concrete repositories, databases, Plugin executors, provider SDKs or journals directly. Public/heterogeneous routes are peer transport adapters over the same application ports, not duplicate business implementations. No peer client spawns or parses another peer executable as its production path.

| Client adapter | Direction | Admitted transport | Constraint |
|---|---|---|---|
| Dioxus Web/Android | user → platform | generated server function / typed events or equivalent M10 transport | presentation only; no CLI/process bridge |
| `ustc-agent` | user/script → platform | explicit versioned HTTP/JSON and SSE | least-privilege user profile; stable JSON/NDJSON machine output |
| inbound MCP | external Agent → platform | selected MCP tools/resources mapped through client-core to explicit M10 routes | no operator registry, direct domain call or M51 reach-through |
| M51 outbound MCP | platform → external MCP server | M51 binding/session/executor contract | opposite direction; not an M80 client adapter |

Every independently deployed request carries client build/target/protocol identity. Unsupported Android, CLI or MCP-adapter/server combinations return a typed compatibility or `UpgradeRequired` outcome before application dispatch. Shared Rust source does not waive deployed-version compatibility.

## Agent tool protocol — H0 values implemented, production execution planned

| Object | Direction | Purpose |
|---|---|---|
| `AgentToolsetView` | resolver/gateway → Agent | immutable per-turn complete tool definitions plus opaque private route references |
| `AgentToolCall` | Agent → ToolGateway | provider-neutral correlated call against the exact frozen projection |
| `PluginExecutionRequest` | ToolGateway → PluginExecutor | authorized bounded execution request after effect intent persistence |
| `PluginExecutionOutcome` | PluginExecutor → ToolGateway | non-authoritative bounded outcome for validation and receipt persistence |
| `AgentToolResult` | ToolGateway → Agent | correlated bounded result/evidence/receipt projection for the next model turn |

`AgentToolsetView`, `AgentToolCall` and the digest/code subset of `AgentToolResult` are concrete Rust types in `crates/agent-tool-protocol` and are exercised by a composition-root fake gateway/executor. `PluginExecutionRequest`, transport, durable receipt composition, bounded content/artifact result expansion and any HTTP/MCP wire format remain planned; no generic extension ABI is implied.

## Candidate inbound MCP/resource registry — Course Planning draft

These names are candidate selected M80 inbound tools/resources for an external Agent. Each must map through client-core to an explicitly registered M10 application operation and pass `CLIENT-010`; the table does not authorize M51 outbound execution, a direct M72 call or an operator command.

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
