# Agent runtime and integration boundary

## Metadata

- `Layer`: Runtime architecture
- `Status`: R0 platform-owned transition kernel implemented; finite harness, orchestration, persistence and production adapters planned
- `Version`: `0.8.0`
- `Last Review`: `2026-08-15`
- `Authority Owns`: finite HarnessRun/TaskGraph, Plugin-neutral node AgentRun state, context budget, versioned Agent tool protocol, tool-effect ordering, framework/provider adapter boundary, selective production persistence policy and the application-level RunExecutionCoordinator contract
- `Authority Defers To`: platform authority for domain state and adapter implementations for protocol details
- `Counterpart Features`: `docs/features/04-bounded-agent-harness.md`; current Market and product features
- `Counterpart Contracts`: `docs/contracts/agent-harness.md`, `docs/contracts/agent-plugin-boundary.md`, `docs/contracts/agent-runtime.md`, `docs/contracts/invocation-resolution.md`, `docs/contracts/interfaces.md`, `docs/contracts/permissions.md`
- `Counterpart Acceptance`: planned `HARNESS-*`; active `AGENT-001`, `AGENT-002`, `AGENT-017` and implemented P0a `MARKET-005/006`; planned `AGENT-018`, `PKG-019/020`, `MARKET-007` and `RUNTIME-*`; long-horizon `AI-*`, `MCP-*`, `RUN-*` and remaining `AGENT-*`
- `Primary Code Areas`: `crates/agent-runtime/`, future orchestration modules and `crates/adapters/`
- `Large-module Blueprints`: [`M30 Agent`](modules/40-agent-harness-runtime.md), [`M40 Tool Gateway`](modules/50-tool-gateway-execution.md), [`M50 Model Provider`](modules/60-model-provider-integration.md), [`M51 MCP`](modules/61-mcp-binding-executor.md)

## 1. Principle

Own campus semantics and authority. Reuse stable protocols and low-differentiation plumbing. Do not merge Agent orchestration, tool execution, model providers and MCP bindings into one runtime module, and do not merge several Agent frameworks into one canonical runtime.

Framework choice follows the domain boundary; it does not define package identity, grants, approvals, source revisions, receipts or audit.

Agent and Plugin are independent modules. Agent code depends only on the versioned tool protocol; package manifests, component kinds, executor implementations and extension SDKs terminate at resolver/gateway composition.

## 2. Owned runtime state

A platform run has an immutable specification containing at least:

- platform run ID and tenant/user scope;
- installed Plugin/package/component identity;
- provider/model profile identity;
- current grant/capability schema version;
- source/profile context references;
- turn, token/cost, tool, time and retry budgets.

The exact R0 shape, validation rules and event semantics are owned by [`docs/contracts/agent-runtime.md`](../contracts/agent-runtime.md) and `crates/agent-runtime/`. The kernel pins resolved identities; it does not itself claim that an installation or grant exists. Existing package/install/component strings in `agent-run/v0` are opaque replay/audit provenance and MUST NOT be parsed into Plugin behavior. The implemented pure P0a producer of synthetic in-memory resolved identities is defined by [`docs/contracts/invocation-resolution.md`](../contracts/invocation-resolution.md); durable loaders remain planned.

State machine target:

```text
Created
→ Preparing
→ ModelTurn
→ AwaitingToolApproval
→ ExecutingTools
→ Preparing
→ ModelTurn
→ Completed

terminal alternatives: Failed | Cancelled | Expired
```

Every transition emits a typed immutable event/checkpoint. Streaming and non-streaming are projections of the same transitions.

## 3. Finite harness and context projection

The complete user-task runtime is a finite `HarnessRun` above one or more node-local `AgentRun`s:

```text
ConversationSession
└── HarnessRun
    ├── Contextualize → bounded ClarificationGate
    ├── model-proposed, Rust-validated TaskGraph
    ├── direct or supervised node AgentRuns
    ├── typed EvidencePacks and fresh read-only reviewers
    ├── bounded remediation graph revisions
    └── verified report or explicit terminal non-success
```

The immutable run spec pins the root task contract. The model proposes plans and patches; Rust owns validation, legal transitions, budgets and replay. Parent-owned goal, prohibitions, deliverables and acceptance are immutable. A child planner may refine steps only. Worker context may continue across remediation; every reviewer call is fresh. Hooks and process exit are observations, not completion evidence.

Before every provider call, the complete serialized request `q` must satisfy

```text
T(q) + O + S ≤ floor(L × ρ / 10_000),
```

where `L` is the validated model context limit, `ρ` is the fixed-point send ceiling, `O` reserves output and `S` reserves protocol/estimator uncertainty. If it does not fit, deterministic offloading precedes lossy compression; the rebuilt request must meet a lower target `τ < ρ`. Compression executes a prevalidated finite chunk/call plan and cannot recursively re-enter compression. Canonical transcript, graph, receipts and evidence are unchanged. Failure to fit produces `ContextBudgetExceeded` before provider I/O.

[`agent-harness/v0`](../contracts/agent-harness.md) owns the exact phase, graph, review, supervision and context-budget contract. It is a bounded task harness, not a generic workflow language.

## 4. Agent–Plugin tool boundary

Runtime capability extension follows one dependency-inverted seam:

```text
PluginPackage + installation/grants
→ InvocationResolver
→ gateway-private ToolProjectionSnapshot
→ AgentToolsetView + private ToolRouteTable
→ AgentToolCall
→ ToolGateway authorization/intent
→ bounded PluginExecutor
→ receipt + AgentToolResult
```

The Agent sees only versioned definitions/calls/results and opaque route references. It neither loads package manifests nor links Plugin code. The executor receives a bounded request and cannot mutate run/graph/context/approval state or forge receipts. `ustc-agentd` is the composition root allowed to depend on both modules.

Package updates create new exact component/binding/projection identities; they never mutate an in-flight toolset. A `NativeRustComponent` crosses an admitted process/WASI/OCI-or-future protocol boundary rather than a dynamic Agent-runtime linkage. [`agent-plugin-boundary/v0`](../contracts/agent-plugin-boundary.md) owns the exact protocol, dependency, packaging, failure and compatibility rules.

## 5. Effect ordering

Before any external or durable effect:

1. resolve current installation/component/execution identity;
2. validate exact tool schema and arguments;
3. authorize tenant/object/capability and confirmation policy;
4. persist invocation/approval intent and idempotency identity;
5. execute through a bounded adapter;
6. persist structured receipt;
7. only then advance the run.

Crash/resume MUST NOT repeat a successful receipt. Budgets do not reset on resume. Cancellation distinguishes queued, in-flight and after-turn semantics before implementation.

## 6. Adapter boundary

Domain/run/authorization code MUST NOT import framework-specific state as authority. A narrow adapter accepts platform-owned requests and emits typed model/tool events.

| Reference | Strong patterns to study and selectively borrow | Platform application | Must not own |
|---|---|---|---|
| Rig | Rust-native provider abstraction, structured output, streaming and cassette-backed provider tests | narrow `ModelBackend` transport and offline provider conformance | platform run loop, grants, receipts, memory or audit |
| Claude Code | finite agentic loop, plan/clarification, isolated subagents, automatic context compaction and self-contained namespaced Plugin bundles whose MCP servers join the common tool surface | harness UX, supervision benchmarks and versioned package/component conventions | generated workflow code, Plugin hooks/settings/agents or summaries as platform authority |
| LangGraph / Deep Agents | checkpoint/store separation, interrupt/resume, observable state, offloading/summarization and subagent context isolation | durable-journal, approval, context and restart benchmarks | canonical checkpoint, installation, grant or authorization truth |
| Pi Agent | minimal Agent core, provider-neutral registered tools, explicit event/preflight barriers, ordered tool results, lossless history and independently distributable extensions/Pi Packages | tool protocol/event projection, turn barriers and package-vs-core replacement benchmark | arbitrary TypeScript extension access, mutable tools/session or package trust as platform authority |
| goose | MCP extension lifecycle, session-scoped extension activation, diagnostics and per-tool allow/ask/deny controls | `McpBinding` lifecycle, capability-scoped tool projection and permission UX | autonomous-by-default execution or direct extension authority in the central plane |
| Hermes Agent | platform-agnostic core, central registry plus toolsets/availability gates, explicit plugin-shadowing controls, progressive skills, bounded memory and operational profiles that are not filesystem sandboxes | interface adapters, grant-filtered toolsets, layered context and procedural knowledge | chat memory, skills, registry/profile state, subagent state or gateway session as campus authority |

A framework checkpoint is adapter state keyed by `platform_run_id`. Conflict with platform state fails closed.

### 6.1 Mandatory reference protocol

Before adding or materially changing a runtime capability:

1. name the platform-owned invariant, threat boundary and first real consumer;
2. inspect applicable official docs and current source for all relevant references above, recording date and exact release/commit plus license when dependency or code adoption is considered;
3. write a capability matrix with `borrow`, `adapt`, `reject` and rationale—feature-table similarity alone is insufficient;
4. map accepted patterns into owned commands/events/types rather than importing framework state into domain contracts;
5. test the pattern through an equal-contract spike or deterministic fixture before dependency adoption;
6. run the deployment, build, type, semantic, persistence and authority/security intrusion audit;
7. require a new/amended ADR if a framework will own more than protocol plumbing inside a bounded adapter.

The comparison is a design gate, not a mandate to implement every feature. A simpler owned mechanism wins when it preserves the invariant with less total maintained semantic surface.

### 6.2 Dated official-source baseline

The reference matrix above was revalidated on `2026-07-24` against:

- [Rig official repository/docs](https://github.com/0xPlaygrounds/rig) and its provider/test guidance;
- [Claude Code agent loop](https://code.claude.com/docs/en/how-claude-code-works), [context window](https://code.claude.com/docs/en/context-window), [subagents](https://code.claude.com/docs/en/sub-agents), [agent teams](https://code.claude.com/docs/en/agent-teams), [plugins](https://code.claude.com/docs/en/plugins), [plugin reference](https://code.claude.com/docs/en/plugins-reference) and [dynamic workflows](https://claude.com/blog/a-harness-for-every-task-dynamic-workflows-in-claude-code);
- [LangGraph persistence](https://docs.langchain.com/oss/python/langgraph/persistence), [interrupts](https://docs.langchain.com/oss/python/langgraph/interrupts) and [Deep Agents context engineering](https://docs.langchain.com/oss/python/deepagents/context-engineering);
- [Pi Agent core](https://github.com/badlogic/pi-mono/tree/main/packages/agent), [coding-agent extensions](https://github.com/badlogic/pi-mono/blob/main/packages/coding-agent/docs/extensions.md) and [Pi Packages](https://github.com/badlogic/pi-mono/tree/main/packages/coding-agent);
- [goose extensions](https://goose-docs.ai/docs/getting-started/using-extensions), [permission modes](https://goose-docs.ai/docs/guides/managing-tools/goose-permissions) and [tool permissions](https://goose-docs.ai/docs/guides/managing-tools/tool-permissions);
- [Hermes Agent architecture](https://hermes-agent.nousresearch.com/docs/developer-guide/architecture), [context compression/caching](https://hermes-agent.nousresearch.com/docs/developer-guide/context-compression-and-caching), [Tools Runtime](https://hermes-agent.nousresearch.com/docs/developer-guide/tools-runtime), [plugins](https://hermes-agent.nousresearch.com/docs/developer-guide/plugins), [profiles](https://hermes-agent.nousresearch.com/docs/user-guide/profiles), [skills](https://hermes-agent.nousresearch.com/docs/user-guide/features/skills) and [memory](https://hermes-agent.nousresearch.com/docs/user-guide/features/memory).

These links are evidence pointers, not frozen compatibility claims. Revalidate them before each adoption decision.

## 7. Model provider profiles

The MVP preserves two explicit execution modes:

| Mode | Purpose | Credential owner | MVP state |
|---|---|---|---|
| `OfficialCentral` | platform-selected model for default/demo journeys | platform operator | planned |
| `UserCloud` | user-selected compatible cloud endpoint/model | tenant/user secret reference | planned |

Two modes remain reserved but unimplemented until their transport, trust and offline semantics have separate acceptance evidence:

- `UserDeviceRelay`;
- `UserRemoteRelay`.

A provider profile is typed state, not arbitrary request metadata. It contains at least:

- stable profile ID and tenant/owner scope;
- execution mode;
- provider kind, normalized base URL and model identity;
- secret reference, never literal key material;
- capability snapshot: streaming, structured output/tool support and exact context limit;
- compatible local tokenizer/estimator identity, version and validation class;
- validation status, observed time and adapter version.

Rules:

1. `OfficialCentral` and `UserCloud` failures are structured and visible; neither silently falls back to the other.
2. User endpoints pass scheme/host/redirect/DNS/IP and credential-leakage review before validation.
3. The platform stores encrypted or external secret references, never secrets in Git, manifests, logs, receipts or normal evidence.
4. A profile/model/capability change invalidates any run/tool assumption that depended on the old snapshot.
5. Provider adapters do not own run state, grants, budget reset or audit policy.

## 8. MCP component and binding lifecycle

`McpServerComponent` is catalog metadata. An `McpBinding` is runtime authority created for an exact installation/component/execution context. Listing or installing a package does not itself authorize any discovered tool.

A binding records:

- binding, tenant, installation, package ID/version/digest and component identity;
- source class: reviewed package component or user-declared remote endpoint;
- transport and endpoint/command identity;
- execution/hosting profile;
- secret references and tenant scope;
- discovered server identity/protocol version;
- exact tool names, input-schema digests and risk/capability mapping;
- grant snapshot, validation state, timestamps and audit references.

Lifecycle:

```text
Declared
→ EndpointValidated
→ ProtocolInitialized
→ ToolsDiscovered
→ SchemaReviewed
→ Approved
→ Active
→ Quarantined | Retired
```

Normative rules:

1. User-remote MCP supports reviewed Streamable HTTP only for the MVP; local stdio or relay execution requires a separately admitted package/relay profile.
2. Endpoint and every redirect/auth-metadata target repeat SSRF validation. Loopback, private/link-local/metadata networks fail closed unless an explicit internal operator profile owns them.
3. Connection testing performs protocol initialization/discovery only; it never invokes a business tool.
4. Pagination is exhausted under bounded limits before tool inventory and schema digest are committed.
5. A new/removed tool or changed input schema blocks the old grant and requires review/reapproval.
6. Every call resolves the exact active installation, component, binding, schema snapshot and grant; same-name alternatives never receive silent fallback.
7. Arguments are validated before outbound I/O. Dynamic output is bounded, labeled untrusted and isolated from higher-priority instructions.
8. USTC login credentials/tokens are never forwarded to a remote MCP server.
9. Scheduled execution rejects interactive-only or changed grants.
10. Audit records the resolved execution identity and receipt without secret payloads.

## 9. Hosted MCP/runtime boundary

Hosted execution is a conditional feasibility lane, not a prerequisite for the core three-Plugin demo. Catalog publication does not grant `SharedSafe`, warm-pool or arbitrary container execution.

Any hosted runtime that enters scope MUST satisfy:

- reviewed OCI artifact pinned by digest;
- typed execution spec accepted only by an internal runtime controller;
- non-root, read-only filesystem, dropped capabilities, no host device/mount/runtime socket;
- no database administration, secret-master, cloud metadata or runtime-admin reachability;
- deny-by-default egress/resource profile and tenant quota;
- one deterministic cold-start winner under concurrent first requests;
- bounded queue/readiness/timeout/backoff;
- drain in-flight invocation before idle stop or revoke;
- tenant-isolated sessions, volumes and secrets;
- revoke/emergency block prevents new sessions and retires old replicas safely.

The public API has no orchestrator administration capability. A dedicated real-host spike must produce a GO/NO-GO decision before `demo-hosted` becomes committed scope. A NO-GO preserves case IDs as deferred; it does not weaken the core demo.

## 10. Shared provider/tool safety

- Provider URL, credentials and model identity are typed profile state; secrets use references and are redacted from normal evidence.
- Unknown tool/schema or changed capability requires reapproval.
- Tool output is bounded and treated as untrusted data, not higher-priority instruction.
- No silent provider, model, tool, binding, runtime or same-name endpoint fallback.
- Prompt/tool payload telemetry is off by default.
- Every model request passes the pinned context-budget preflight; no profile/estimator means no call.
- Public API code does not receive broad process/runtime administration capability.
- Approval/policy/audit failure blocks the effect; it is not downgraded to a warning.

## 11. Framework adoption gate

Before adopting a runtime framework beyond a bounded adapter spike, verify:

- `OfficialCentral` and `UserCloud` typed profiles;
- streaming/final-state parity;
- timeout, backpressure and cancellation;
- pre-effect policy hooks;
- dynamic tool schema snapshot and revalidation;
- typed error projection;
- platform IDs/audit context through every event;
- no default content telemetry;
- upgrade without domain/API schema changes;
- platform-owned crash/resume semantics.
- complete-request token measurement, bounded compaction and canonical-history preservation.
- Agent/framework replacement without Plugin package/executor changes when the major tool protocol is unchanged.

If these cannot remain inside the adapter, retain the same domain contracts and implement a narrower provider loop. Do not fork a framework to make it the platform ontology.

## 12. Current state and verification

Implemented now:

- framework-neutral `RunSpec`, phase, command and immutable event contracts;
- legal-transition validation and deterministic replay;
- effect intent/receipt identity and ordering checks;
- replay-stable turn/tool/input-token/output-token/cost/retry/elapsed budget accounting;
- typed fail-closed errors for illegal transitions, identity mismatch and budget violation.
- `agent-runtime` production/test dependency independence from Market, Plugin and adapter crates, enforced by the repository checker;
- P0a→`RunSpec` cross-boundary proof owned by `ustc-agentd`, the composition root.

This is an R0 domain kernel, not a production Agent run. Concrete `agent-tool-protocol/v0` value objects and composition-root fake gateway/executor conformance are implemented; production ToolGateway, durable journal composition, model provider profile, MCP binding, hosted runtime, external tool execution and HTTP/SSE run surface are not. The Course Planning CLI still calls deterministic Rust domain code directly and is not Plugin integration evidence.

The accepted finite harness, TaskGraph, clarification/review supervisor and context-budget/compaction contracts are H0 target architecture only; no production harness, tokenizer, compactor, subagent supervisor or plan panel exists yet.

Active implemented proof:

- `AGENT-001`: only legal, evidenced transitions replay;
- `AGENT-002`: immutable run spec pins exact platform identities and budgets;
- `AGENT-017`: runtime dependencies remain confined while composition owns cross-boundary proof;
- `AGENT-019`: frozen tool definitions, private routes and fake gateway/executor calls fail closed and correlate results;
- `MARKET-005`: one deterministic per-turn projection binds model exposure and no-fallback dispatch;
- `MARKET-006`: projection-time authority mismatches return typed denial with no partial run.

Active planned proof:

- `HARNESS-*`: finite phase/graph/review/context/supervision invariants;
- `RUNTIME-001`: stream/non-stream final-state convergence;
- `RUNTIME-002`: restart/resume without duplicate receipts/effects.

The retained long-horizon catalog in `docs/acceptance/platform-baseline.md` preserves `AI-*`, `MCP-*`, `RUN-*` and `AGENT-*` cases for activation when those features enter current scope.

## 13. Selective production persistence

Production persistence is selected per aggregate by its authority and failure semantics, not by a single global mechanism. Each owning module declares its own port contract; the choice below is normative for M30/M40/M60 and peers once production adapters land.

| Aggregate | Production persistence | Reconstruction guarantee |
|---|---|---|
| run/external effect | canonical event journal (M30) | deterministic replay of legal, evidenced transitions |
| source revision / raw evidence | immutable revision + bounded artifact (M60) | byte-exact raw/normalized snapshot rebuild |
| semantic change / publication | typed event + immutable artifact (M70/M71/M72) | projection rebuild from accepted revisions and receipts |
| session | current row + revision/CAS + security audit | crash-safe resume without duplicate effects |
| installation / grant | transactional current row + append-only audit/receipt (M20) | legal-state replay and revoke reconciliation |
| settings / profile | current projection + revision (M00) | recoverable to last accepted revision |
| search / cache | rebuildable projection | rebuild from canonical declarations and immutable evidence |

Norms:

1. A pure decide/evolve/replay test MUST NOT require a real event store. Domain aggregates keep their legal-transition invariants testable against fakes.
2. An aggregate that already owns deterministic replay over a typed event journal MAY reuse that journal as its production reconstruction source; it is not forced to add a second store.
3. An aggregate whose authority is an immutable artifact (M60 source revision) reconstructs from that artifact plus its accepted-baseline pointer, not from replaying the pipeline that produced it.
4. A current-row aggregate (installation, grant, session, settings) keeps a transactional current state plus an append-only audit/receipt trail; legal-state replay validates the trail but production reads the current row.
5. Search, cache and derived projections are never a reconstruction source for any other aggregate; they rebuild from the canonical owners above.

This section owns the policy. The exact port shapes, repository contracts and journal schemas live in `docs/contracts/agent-runtime.md`, `docs/contracts/module-boundaries.md` §5 and each owning module's blueprint.

## 14. RunExecutionCoordinator contract ownership

Before M30/M40 production integration, an application-level `RunExecutionCoordinator` owns one bounded orchestration and recovery seam. It lives in the composition surface (`ustc-agentd` or a declared application module), not inside M30, M40 or M20 domain code.

Responsibilities:

- order M30 journal commands and M40 staged execution ports across one run phase;
- own outbox/effect identity so a crash between M40 execution and M30 receipt does not produce a duplicate or lost external effect;
- reconcile uncertain receipts: a receipt that does not prove a completed external effect is treated as not-completed, never silently advanced;
- enforce the §5 effect ordering across composition: resolve identity, validate, authorize, persist intent, execute, persist receipt, advance run.

Constraints:

1. `RunExecutionCoordinator` MAY issue M30 journal commands and M40 staged execution ports. It MUST NOT mutate M30 run/graph/context state directly, M40 route/gateway state directly, or M20 installation/grant state directly.
2. It MUST NOT create a direct implementation dependency cycle: M30 → M40 → M20 → M30 is forbidden. The coordinator composes their public ports; it is not a peer domain module.
3. It MUST NOT weaken the §5 effect ordering, the §10 shared provider/tool safety rules, or any module's legal-transition ownership.
4. It owns only the composition seam. Each module's standalone acceptance is independent of it.

This is contract ownership, not an instruction to implement the full runtime now. A production `RunExecutionCoordinator` lands only when M30/M40 production integration is admitted, and its acceptance evidence is bound to the composition root, not to either module's standalone gate.
