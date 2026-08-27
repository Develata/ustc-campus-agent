# M40 — Tool Gateway and Execution

## Metadata

- `Module ID`: `M40`
- `Status`: Accepted blueprint; protocol/fake conformance plus one bounded Affairs fixed-adapter composition implemented
- `Implementation State`: `partial-evidence`
- `Version`: `m40-tool-gateway/v0`
- `Last Review`: `2026-07-25`
- `Owning Contract`: [`../../contracts/agent-plugin-boundary.md`](../../contracts/agent-plugin-boundary.md)
- `Primary code areas`: `crates/agent-tool-protocol/`, `apps/ustc-agentd/src/affairs_invocation.rs`, future generic gateway/executor modules, composition tests in `apps/ustc-agentd/tests/`

## 1. Purpose

`M40` is the only normal path from an Agent-selected tool to an admitted executor. It normalizes calls against the frozen tool view, correlates private routes, requests current authorization, exposes staged preparation/execution/result operations, bounds untrusted output and returns one correlated result. `ustc-agentd` composition places `M30` intent/receipt commands between those stages.

It coordinates decisions owned by `M20` and `M30`; it does not create replacement authority.

## 2. Non-goals

- minting package grants or installation state;
- deciding Agent phases, graph completion or budget reset;
- implementing every executor transport in one module;
- exposing package/executor details to Agent/provider code;
- dynamic in-process Plugin linkage;
- fallback by visible tool name, package, endpoint or previous success.

## 3. Owned objects and state

```text
AgentToolDefinition / AgentToolsetView
AgentToolCall / AgentToolResult
ToolRouteRef / private route entry
GatewayCallCorrelation
AuthorizedExecutionEnvelope
PluginExecutionRequest / PluginExecutionOutcome
OutputBound / redacted diagnostics
GatewayError
```

Effect intent/receipt truth remains in `M30`'s runtime journal. Installation/grant truth remains in `M20`.

## 4. Public inputs and outputs

Inputs:

```text
frozen AgentToolsetView and gateway-private route table
AgentToolCall
current request/tenant context
M20 authority snapshot/recheck result
admitted PluginExecutor port
```

Stage outputs:

```text
PreparedToolExecution            # authorized exact call; no executor I/O yet
BoundedPluginExecutionOutcome    # untrusted executor outcome, not run truth
AgentToolResult {
  exact run/turn/call/projection correlation
  Succeeded | Failed | Denied | Cancelled | TimedOut
  bounded content/artifact/evidence references
  receipt reference when execution was attempted
  stable redacted error class
}
```

A denied call creates no executor request. Executor output is non-authoritative until validated and receipted.

## 5. Dependency direction

Allowed dependencies:

- `agent-tool-protocol`;
- public `M20` resolution/recheck interfaces;
- executor ports implemented by `M51` or separately admitted peers;
- `M90` cancellation/telemetry adapters where declared.

`M30` and `M40` depend on `agent-tool-protocol`, not on each other's implementation. `ustc-agentd` composition reads an `M30` proposal, invokes an `M40` stage, issues the separate public `M30` intent/receipt commands, and passes the resulting identities to the next `M40` stage. This prevents a code cycle while leaving effect truth in `M30`.

Forbidden dependencies:

- Dioxus/client types;
- Market repository internals;
- model provider SDKs;
- executor-specific handles in Agent-facing values;
- arbitrary process/runtime administration from public API data.

## 6. Lifecycle

```text
M20 freezes projection + private routes
→ M30/provider or deterministic Harness sees AgentToolsetView
→ M30 records the exact selected proposal
→ composition asks M40 to normalize/recheck and return PreparedToolExecution
→ composition commands M30 to persist EffectIntent
→ composition asks M40 to call the admitted executor
→ M40 returns a validated bounded outcome
→ composition commands M30 to persist EffectReceipt
→ composition supplies the receipt reference to M40 for correlated AgentToolResult
→ composition submits AgentToolResult to M30
```

A package update creates new routes/projections for new turns. It never mutates an in-flight call.

## 7. Failure and recovery

- Unknown tool/malformed arguments/stale projection/route mismatch/current deny: no intent or executor I/O where the contract requires pre-intent denial.
- Intent persistence failure: no executor I/O.
- Executor crash/timeout/cancel: one typed outcome and receipt path.
- Receipt persistence uncertainty: do not return success; reconcile by call/effect/idempotency identity.
- Duplicate exact call: return/reconcile prior disposition without duplicate effect.
- Conflicting duplicate: reject.
- Oversized/invalid/hostile output: truncate/reject according to bound, label untrusted and preserve safe diagnostics.
- Correlation failure: fail closed; never match by nearby name.

## 8. Configuration and secrets

Gateway policy pins output/time/resource limits and admitted executor profile IDs. Executor requests receive only allowed `SecretRef`s and scoped values; the gateway never copies raw secrets into Agent results, logs or receipts.

## 9. Observability

Record run/turn/provider-call/projection/route/effect IDs, decision stages, executor profile, timing/resource class, output bound disposition and stable error. Private route/executor details are visible only to authorized operator evidence, not model/UI content.

## 10. Extension and replacement

Executor transports are peers behind `PluginExecutor`: MCP (`M51`), admitted native process/tool host, future WASI/OCI profiles or other reviewed protocols. Adding a peer does not change Agent protocol or run phases. Breaking `agent-tool-protocol` changes require a new major version and explicit migration.

## 11. Performance path

Per-call work inside `M40` is bounded lookup, schema validation, current authority check and one executor call. Separate `M30` journal writes are ordered by composition between the staged calls. Route lookup is exact and indexed. Output size/count/time and concurrent in-flight calls are limited. Authorization/journal correctness is not cached past its validity boundary.

## 12. Scope boundary

**MVP**

- complete frozen tool definitions and private routes;
- exact call normalization and schema validation;
- current denial before execution;
- intent/executor/receipt/result sequence;
- one fake executor and one real admitted read-only executor;
- bounded output and duplicate suppression.

**Later**

- additional executor peers;
- richer artifact/content blocks;
- controlled write/destructive confirmation UX;
- isolation profiles with proven hosting.

**Explicit non-goals**

- arbitrary host command execution;
- in-process dynamic Plugin ABI;
- Plugin hooks mutating Agent state;
- silent tool/provider/runtime fallback.

## 13. Small-module decomposition

1. `tool-schema` — canonical schema/argument values and digests.
2. `agent-tool-envelope` — frozen view/call/result protocol.
3. `route-table` — private exact route correlation.
4. `call-normalization` — name/schema/snapshot/correlation validation.
5. `gateway-authorization` — M20 current recheck mapping.
6. `execution-stages` — prepare, execute and result stages that make composition ordering testable without importing `M30`.
7. `executor-port` — bounded request/outcome and fake executor.
8. `output-boundary` — untrusted content/artifact limits and redaction.
9. `gateway-recovery` — duplicate/reconcile/timeout/cancel.
10. `gateway-conformance` — standalone and composition fixtures.

## 14. Exit gate

`M40` is standalone-ready when fake tests prove every denial reaches no executor, success executes exactly once, duplicate/conflict/restart semantics are deterministic, and output bounds hold. It is accepted when one admitted executor completes through durable intent/receipt composition and `AGENT-018/019`, `MARKET-007` and applicable `PKG-*` rows pass.
