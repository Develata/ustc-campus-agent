# Agent runtime and integration boundary

## Metadata

- `Layer`: Runtime architecture
- `Status`: R0 platform-owned transition kernel implemented; orchestration, persistence and production adapters planned
- `Version`: `0.4.3`
- `Last Review`: `2026-07-23`
- `Authority Owns`: platform run state, tool-effect ordering, framework/provider adapter boundary
- `Authority Defers To`: platform authority for domain state and adapter implementations for protocol details
- `Counterpart Features`: future bounded Agent journey; current Market and product features
- `Counterpart Contracts`: `docs/contracts/agent-runtime.md`, `docs/contracts/invocation-resolution.md`, `docs/contracts/interfaces.md`, `docs/contracts/permissions.md`
- `Counterpart Acceptance`: active `AGENT-001`, `AGENT-002`; planned `MARKET-005`–`MARKET-007` with P0a limited to `MARKET-005/006`, plus planned `RUNTIME-*`; long-horizon `AI-*`, `MCP-*`, `RUN-*` and remaining `AGENT-*`
- `Primary Code Areas`: `crates/agent-runtime/`, future orchestration modules and `crates/adapters/`

## 1. Principle

Own campus semantics and authority. Reuse stable protocols and low-differentiation plumbing. Do not merge several Agent frameworks into one canonical runtime.

Framework choice follows the domain boundary; it does not define package identity, grants, approvals, source revisions, receipts or audit.

## 2. Owned runtime state

A platform run has an immutable specification containing at least:

- platform run ID and tenant/user scope;
- installed Plugin/package/component identity;
- provider/model profile identity;
- current grant/capability schema version;
- source/profile context references;
- turn, token/cost, tool, time and retry budgets.

The exact R0 shape, validation rules and event semantics are owned by [`docs/contracts/agent-runtime.md`](../contracts/agent-runtime.md) and `crates/agent-runtime/`. The kernel pins resolved identities; it does not itself claim that an installation or grant exists. The planned P0a producer of those identities is defined by [`docs/contracts/invocation-resolution.md`](../contracts/invocation-resolution.md).

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

## 3. Effect ordering

Before any external or durable effect:

1. resolve current installation/component/execution identity;
2. validate exact tool schema and arguments;
3. authorize tenant/object/capability and confirmation policy;
4. persist invocation/approval intent and idempotency identity;
5. execute through a bounded adapter;
6. persist structured receipt;
7. only then advance the run.

Crash/resume MUST NOT repeat a successful receipt. Budgets do not reset on resume. Cancellation distinguishes queued, in-flight and after-turn semantics before implementation.

## 4. Adapter boundary

Domain/run/authorization code MUST NOT import framework-specific state as authority. A narrow adapter accepts platform-owned requests and emits typed model/tool events.

| Reference | Strong patterns to study and selectively borrow | Platform application | Must not own |
|---|---|---|---|
| Rig | Rust-native provider abstraction, structured output, streaming and cassette-backed provider tests | narrow `ModelBackend` transport and offline provider conformance | platform run loop, grants, receipts, memory or audit |
| LangGraph | checkpoint/store separation, interrupt/resume, observable state and idempotency discipline around resumed work | durable-journal, approval and restart benchmarks | canonical checkpoint, installation, grant or authorization truth |
| Pi Agent | app-message to LLM-message projection, explicit event lifecycle, tool-preflight barrier, ordered sequential/parallel tool results, steering/follow-up queues and lossless session history behind lossy compaction | model-event projection, turn barriers, queue semantics and future branch/compaction evaluation | TypeScript hot-load, mutable session state or package extensions as platform authority |
| goose | MCP extension lifecycle, session-scoped extension activation, diagnostics and per-tool allow/ask/deny controls | `McpBinding` lifecycle, capability-scoped tool projection and permission UX | autonomous-by-default execution or direct extension authority in the central plane |
| Hermes Agent | platform-agnostic core, central registry plus toolsets/availability gates, explicit plugin-shadowing controls, progressive skills, bounded memory and operational profiles that are not filesystem sandboxes | interface adapters, grant-filtered toolsets, layered context and procedural knowledge | chat memory, skills, registry/profile state, subagent state or gateway session as campus authority |

A framework checkpoint is adapter state keyed by `platform_run_id`. Conflict with platform state fails closed.

### 4.1 Mandatory reference protocol

Before adding or materially changing a runtime capability:

1. name the platform-owned invariant, threat boundary and first real consumer;
2. inspect applicable official docs and current source for all relevant references above, recording date and exact release/commit plus license when dependency or code adoption is considered;
3. write a capability matrix with `borrow`, `adapt`, `reject` and rationale—feature-table similarity alone is insufficient;
4. map accepted patterns into owned commands/events/types rather than importing framework state into domain contracts;
5. test the pattern through an equal-contract spike or deterministic fixture before dependency adoption;
6. run the deployment, build, type, semantic, persistence and authority/security intrusion audit;
7. require a new/amended ADR if a framework will own more than protocol plumbing inside a bounded adapter.

The comparison is a design gate, not a mandate to implement every feature. A simpler owned mechanism wins when it preserves the invariant with less total maintained semantic surface.

### 4.2 Dated official-source baseline

The reference matrix above was revalidated on `2026-07-23` against:

- [Rig official repository/docs](https://github.com/0xPlaygrounds/rig) and its provider/test guidance;
- [LangGraph persistence](https://docs.langchain.com/oss/python/langgraph/persistence) and [interrupts](https://docs.langchain.com/oss/python/langgraph/interrupts);
- [Pi Agent core](https://github.com/earendil-works/pi/tree/main/packages/agent) and [coding-agent harness](https://github.com/earendil-works/pi/tree/main/packages/coding-agent);
- [goose extensions](https://goose-docs.ai/docs/getting-started/using-extensions), [permission modes](https://goose-docs.ai/docs/guides/managing-tools/goose-permissions) and [tool permissions](https://goose-docs.ai/docs/guides/managing-tools/tool-permissions);
- [Hermes Agent architecture](https://hermes-agent.nousresearch.com/docs/developer-guide/architecture), [Tools Runtime](https://hermes-agent.nousresearch.com/docs/developer-guide/tools-runtime), [plugins](https://hermes-agent.nousresearch.com/docs/developer-guide/plugins), [profiles](https://hermes-agent.nousresearch.com/docs/user-guide/profiles), [skills](https://hermes-agent.nousresearch.com/docs/user-guide/features/skills) and [memory](https://hermes-agent.nousresearch.com/docs/user-guide/features/memory).

These links are evidence pointers, not frozen compatibility claims. Revalidate them before each adoption decision.

## 5. Model provider profiles

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
- capability snapshot: streaming, structured output/tool support and context limits;
- validation status, observed time and adapter version.

Rules:

1. `OfficialCentral` and `UserCloud` failures are structured and visible; neither silently falls back to the other.
2. User endpoints pass scheme/host/redirect/DNS/IP and credential-leakage review before validation.
3. The platform stores encrypted or external secret references, never secrets in Git, manifests, logs, receipts or normal evidence.
4. A profile/model/capability change invalidates any run/tool assumption that depended on the old snapshot.
5. Provider adapters do not own run state, grants, budget reset or audit policy.

## 6. MCP component and binding lifecycle

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

## 7. Hosted MCP/runtime boundary

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

## 8. Shared provider/tool safety

- Provider URL, credentials and model identity are typed profile state; secrets use references and are redacted from normal evidence.
- Unknown tool/schema or changed capability requires reapproval.
- Tool output is bounded and treated as untrusted data, not higher-priority instruction.
- No silent provider, model, tool, binding, runtime or same-name endpoint fallback.
- Prompt/tool payload telemetry is off by default.
- Public API code does not receive broad process/runtime administration capability.
- Approval/policy/audit failure blocks the effect; it is not downgraded to a warning.

## 9. Framework adoption gate

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

If these cannot remain inside the adapter, retain the same domain contracts and implement a narrower provider loop. Do not fork a framework to make it the platform ontology.

## 10. Current state and verification

Implemented now:

- framework-neutral `RunSpec`, phase, command and immutable event contracts;
- legal-transition validation and deterministic replay;
- effect intent/receipt identity and ordering checks;
- replay-stable turn/tool/input-token/output-token/cost/retry/elapsed budget accounting;
- typed fail-closed errors for illegal transitions, identity mismatch and budget violation.

This is an R0 domain kernel, not a production Agent run. No durable journal, model provider profile, MCP binding, hosted runtime, external tool execution or HTTP/SSE run surface is implemented. The Course Planning CLI still calls deterministic Rust domain code directly.

Active implemented proof:

- `AGENT-001`: only legal, evidenced transitions replay;
- `AGENT-002`: immutable run spec pins exact platform identities and budgets.

Active planned proof:

- `RUNTIME-001`: stream/non-stream final-state convergence;
- `RUNTIME-002`: restart/resume without duplicate receipts/effects.

The retained long-horizon catalog in `docs/acceptance/platform-baseline.md` preserves `AI-*`, `MCP-*`, `RUN-*` and `AGENT-*` cases for activation when those features enter current scope.
