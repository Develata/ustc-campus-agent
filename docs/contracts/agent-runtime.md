# Agent runtime kernel contract

## Metadata

- `Status`: R0 kernel contract implemented; durable orchestration and external adapters planned
- `Version`: `agent-run/v0`
- `Last Review`: `2026-07-24`
- `Owning Plan`: [`../plan/07-runtime-and-integration.md`](../plan/07-runtime-and-integration.md)
- `Acceptance`: active `AGENT-001`, `AGENT-002`, `AGENT-017`; planned `RUNTIME-*` and remaining `AGENT-*`
- `Primary Code`: `crates/agent-runtime/`

## 1. Scope

The Agent runtime kernel owns immutable identity, legal state transitions, event ordering, effect intent/receipt identity, budget accounting and deterministic replay for one bounded node execution. It is framework- and Plugin-neutral and contains no provider SDK, MCP transport, Market manifest/type, Plugin implementation, adapter, database, HTTP server or user interface dependency.

One `AgentRun` is not a conversation session or complete user-task orchestrator. The planned [`HarnessRun`](agent-harness.md) owns clarification, `TaskGraph`, node supervision, review/remediation, context projection and final reporting above one or more `AgentRun`s.

The kernel does not decide that a package is installed or that a capability is granted. The implemented pure [`invocation-resolution/v0`](invocation-resolution.md) resolver supplies an already pinned in-memory installation/component/grant/schema projection; the kernel records and preserves those identities without widening them. Durable snapshot loading and application composition remain planned.

Package, installation and component strings retained by `agent-run/v0` are opaque provenance bindings for replay/audit compatibility. Runtime code does not parse component kind, load Plugin configuration or branch on package identity. [`agent-plugin-boundary/v0`](agent-plugin-boundary.md) owns the future tool protocol and composition seam.

## 2. Immutable `RunSpec`

A `RunSpec` is validated before a run enters the state machine. It pins:

```text
schema version
run ID
user/tenant scope
installation ID
package ID and exact version
component ID
provider profile ID
capability-grant snapshot ID
approved tool-schema-set digest
turn, tool-call, input-token, output-token, cost, retry and elapsed-time budgets
```

Invariants:

- schema version is exactly `agent-run/v0`;
- every identity is non-empty and contains no control or whitespace characters;
- the tool-schema-set digest is an exact lowercase `sha256:<64 hex>` value;
- every budget is non-zero;
- the spec is immutable after creation; every future durable checkpoint/export must include it exactly rather than reconstructing it from framework state;
- provider/framework state may reference `run_id`, but cannot replace any spec field.
- package/install/component fields are uninterpreted identities; adding a Plugin or executor kind cannot add an Agent transition or code path.

## 3. State machine

Phases are:

```text
Created
→ Preparing
→ ModelTurn | HarnessTurn
→ AwaitingToolApproval
→ ExecutingTools
→ Preparing
→ ModelTurn | HarnessTurn
→ Completed

terminal alternatives: Failed | Cancelled | Expired
```

Only the following non-terminal transitions are legal:

| Current phase | Command/event | Next phase |
|---|---|---|
| `Created` | prepare | `Preparing` |
| `Preparing` | start model turn | `ModelTurn` |
| `Preparing` | start deterministic provider-free harness turn | `HarnessTurn` |
| `ModelTurn` | persist exact provider usage | `ModelTurn` |
| `ModelTurn` | propose tool call | `AwaitingToolApproval` |
| `HarnessTurn` | propose deterministic tool call without model usage | `AwaitingToolApproval` |
| `AwaitingToolApproval` | approve and persist effect intent | `ExecutingTools` |
| `ExecutingTools` | persist successful/failed effect receipt | `Preparing` |
| `ModelTurn` or `HarnessTurn` | complete | `Completed` |
| `Preparing`, `ModelTurn` or `HarnessTurn` | persist retry accounting | `Preparing` |
| any non-terminal phase | persist a strictly newer elapsed-time observation | same phase |

A non-executing phase may transition to `Failed`, `Cancelled` or `Expired`. Cancellation while an effect is in flight is intentionally rejected in R0; later orchestration must distinguish queued, in-flight and post-receipt cancellation rather than guessing.

Terminal phases accept no new state-changing event.

## 4. Event sequence and replay

Each immutable `RunEvent` carries the exact next sequence number. Applying an event requires:

```text
event.sequence == current_revision + 1
```

A gap, duplicate sequence or out-of-order event fails closed. Replaying the same validated `RunSpec` plus the same ordered events must reconstruct an equal `AgentRun` checkpoint.

The kernel exposes decision and evolution separately:

1. decide whether a command is legal and produce the next event;
2. a future durable journal appends that event under an expected revision;
3. evolve the in-memory checkpoint from the persisted event.

R0 tests exercise decision/evolution/replay in memory. They do not claim a production durable journal.

## 5. Tool effect ordering

A proposed tool call binds stable call ID, tool name and arguments digest. Approval creates an `EffectIntent` that additionally pins:

- effect and idempotency IDs;
- capability ID;
- the exact grant snapshot from `RunSpec`;
- the exact tool-schema-set digest from `RunSpec`.

Call, effect and idempotency IDs are unique within one run. The kernel rejects reuse before execution, and rejects approval if any proposal, grant or schema identity differs.

Required ordering is:

```text
ToolCallProposed event
→ EffectIntentPersisted event
→ bounded adapter execution (future orchestrator)
→ EffectReceiptPersisted event
→ next ModelTurn or deterministic HarnessTurn
```

A receipt must match the pending effect and idempotency ID. The first committed receipt wins. Re-submitting an identical receipt is an explicit idempotent no-op; a conflicting receipt for the same effect fails closed.

R0 proves identity and ordering. It does not claim external side-effect exactly-once semantics until a durable journal and adapter integration satisfy `AGENT-003/004`.

## 6. Budgets

`RunSpec` pins maximum turns, tool calls, provider-reported input/output tokens, provider-reported cost in platform-defined millionth currency units, retries and elapsed milliseconds. Corresponding counters are part of the replayed checkpoint and never reset by replay/resume.

Exact model usage is persisted once before a successful model-turn outcome can be accepted. Commands that would exceed a budget are rejected before producing an event; once elapsed time reaches its limit, the kernel rejects a new model turn or tool-effect approval while still allowing receipts and terminalization. Elapsed-time observations must be monotone. Budget rejection leaves the last accepted checkpoint unchanged.

## 7. Typed failures

The kernel distinguishes at least:

- invalid run specification;
- illegal phase transition;
- event sequence mismatch;
- budget exceeded or non-monotone elapsed time;
- proposal/intent/receipt identity mismatch;
- missing or conflicting pending effect and duplicate call/effect/idempotency identity;
- terminal run mutation.

No failure silently changes provider, package, component, tool, grant, schema or runtime identity.

## 8. Tool and adapter boundary

Provider, MCP and tool/Plugin implementations belong outside this crate. The Agent side consumes only versioned Plugin-neutral tool definitions/calls/results. A composition-root `ToolGateway` maps opaque routes to authorized executor requests and typed outcomes, but neither gateway nor executor owns:

- `RunSpec` or legal phase transitions;
- grants or approval state;
- effect/idempotency identity;
- budget reset;
- checkpoint/audit truth.

A concrete adapter trait is introduced only with its first bounded adapter consumer. R0 deliberately avoids freezing an async/runtime/provider API from a hypothetical implementation.

The crate MUST build and test without Market, Plugin implementation or adapter dependencies. Cross-boundary resolver→`RunSpec` proofs belong to the application composition root. A future small protocol crate is allowed only when Agent and fake-gateway consumers both exist; it contains no package or state-machine implementation types.

## 9. Current evidence and non-goals

Implemented evidence:

- `AGENT-001`: legal transition and deterministic replay tests;
- `AGENT-002`: immutable exact run-spec validation and round-trip tests;
- `AGENT-017`: exact crate-target/source containment, rustc dep-info input confinement, conditional/include escape rejection, narrow declaration allowlist, repository Cargo-config rejection, locked/offline resolved dependency-tree check and composition-root cross-boundary proof;
- effect ordering, receipt idempotency and budget fail-closed unit tests supporting—but not completing—later integration cases.

Still planned:

- production journal/database and optimistic concurrency;
- model/provider adapter and stream projection;
- MCP discovery/binding;
- durable installation/grant resolver;
- external tool execution and crash recovery;
- HTTP/SSE routes;
- finite HarnessRun/TaskGraph and context-budget preflight;
- concrete `agent-tool-protocol/v0`, ToolGateway and packaged Plugin executor;
- hosted runtime.
