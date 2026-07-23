# Agent runtime kernel contract

## Metadata

- `Status`: R0 kernel contract implemented; durable orchestration and external adapters planned
- `Version`: `agent-run/v0`
- `Last Review`: `2026-07-23`
- `Owning Plan`: [`../plan/07-runtime-and-integration.md`](../plan/07-runtime-and-integration.md)
- `Acceptance`: active `AGENT-001`, `AGENT-002`; planned `RUNTIME-*` and remaining `AGENT-*`
- `Primary Code`: `crates/agent-runtime/`

## 1. Scope

The Agent runtime kernel owns immutable run identity, legal state transitions, event ordering, effect intent/receipt identity, budget accounting and deterministic replay. It is framework-neutral and contains no provider SDK, MCP transport, database, HTTP server or user interface.

The kernel does not decide that a package is installed or that a capability is granted. The planned [`invocation-resolution/v0`](invocation-resolution.md) resolver supplies an already pinned installation/component/grant/schema projection; the kernel records and preserves those identities without widening them.

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

## 3. State machine

Phases are:

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

Only the following non-terminal transitions are legal:

| Current phase | Command/event | Next phase |
|---|---|---|
| `Created` | prepare | `Preparing` |
| `Preparing` | start model turn | `ModelTurn` |
| `ModelTurn` | persist exact provider usage | `ModelTurn` |
| `ModelTurn` | propose tool call | `AwaitingToolApproval` |
| `AwaitingToolApproval` | approve and persist effect intent | `ExecutingTools` |
| `ExecutingTools` | persist successful/failed effect receipt | `Preparing` |
| `ModelTurn` | complete | `Completed` |
| `Preparing` or `ModelTurn` | persist retry accounting | `Preparing` |
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
→ next ModelTurn
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

## 8. Adapter boundary

Provider, MCP and tool implementations belong outside this crate. They may consume platform-owned request/intent structures and return typed outcomes, but they do not own:

- `RunSpec` or legal phase transitions;
- grants or approval state;
- effect/idempotency identity;
- budget reset;
- checkpoint/audit truth.

A concrete adapter trait is introduced only with its first bounded adapter consumer. R0 deliberately avoids freezing an async/runtime/provider API from a hypothetical implementation.

## 9. Current evidence and non-goals

Implemented evidence:

- `AGENT-001`: legal transition and deterministic replay tests;
- `AGENT-002`: immutable exact run-spec validation and round-trip tests;
- effect ordering, receipt idempotency and budget fail-closed unit tests supporting—but not completing—later integration cases.

Still planned:

- production journal/database and optimistic concurrency;
- model/provider adapter and stream projection;
- MCP discovery/binding;
- durable installation/grant resolver;
- external tool execution and crash recovery;
- HTTP/SSE routes;
- hosted runtime.
