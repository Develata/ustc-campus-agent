# Agent–Plugin tool boundary contract

## Metadata

- `Status`: Accepted target architecture; protocol value objects and monolithic fake gateway/executor conformance implemented; exact B7-B staged composition-test contract accepted but unimplemented; production ToolGateway remains planned
- `Version`: `agent-plugin-boundary/v0`
- `Last Review`: `2026-08-02`
- `Owning Plans`: [`04-market-and-plugin-lifecycle.md`](../plan/04-market-and-plugin-lifecycle.md) owns package lifecycle; [`07-runtime-and-integration.md`](../plan/07-runtime-and-integration.md) owns Agent/tool execution
- `Decisions`: [`ADR-0008`](../adr/0008-agent-plugin-tool-boundary.md)
- `Acceptance`: implemented `AGENT-017`, `AGENT-019`; planned `AGENT-018`, `MARKET-007`, `PKG-019`, `PKG-020`; B7-B contract acceptance promotes no row
- `Primary Code`: `crates/agent-tool-protocol/`, `crates/agent-runtime/`, `crates/platform-core/src/invocation.rs`, `apps/ustc-agentd/tests/tool_gateway_conformance.rs`

## 1. Purpose

Agent orchestration and Plugin implementation are independent change families. Their only runtime seam is a versioned platform-owned tool protocol mediated by `ToolGateway`.

```text
PluginPackage / installation / grants
                │
                ▼
       InvocationResolver
                │
                ▼
 ToolProjectionSnapshot (full authority; gateway-private)
       ├── AgentToolsetView ───────────────► Agent / provider
       └── ToolRouteTable ─► ToolGateway ─► PluginExecutor
                                  │
                                  └────────► intent / receipt journal
```

The Agent does not load manifests, link Plugin implementations, inspect component kinds or call package code. A Plugin does not receive Agent state, mutate the graph/loop, inject authority hooks or call a model on behalf of the run.

## 2. Ownership

| Owner | Owns | Must not own |
|---|---|---|
| Agent kernel/harness | run/graph phases, model loop, budgets, context projection, review and replay | package lifecycle, grants, execution routing or Plugin state |
| Market/Plugin authority | package/version/component declarations, installation and enable lifecycle | model loop, run transitions, approval, receipts or canonical transcript |
| Invocation resolver | exact per-turn authority projection and deny-side recheck | provider session, effect execution or run completion |
| ToolGateway/composition | protocol mapping, call correlation, authorization ordering, executor selection and result normalization | grants, run state, package declarations or receipt truth |
| PluginExecutor | bounded implementation behind one admitted execution identity | tool discovery authority, model-visible registration, approval, effect identity or audit truth |
| durable runtime | effect intents/receipts and canonical run evidence | Plugin implementation details |

`ToolGateway` coordinates owners; it is not a new authority source. Every decision it uses is returned by the owning resolver/runtime command.

## 3. Agent-facing protocol

The logical protocol version is `agent-tool-protocol/v0`. Its framework-neutral canonical values and sealed view/call/result envelopes live in `crates/agent-tool-protocol`, now that Agent runtime and fake gateway are real consumers. Resolver, package authority, transport and executor implementation types remain outside this crate.

### 3.1 `AgentToolsetView`

A per-turn immutable view contains only:

```text
protocol version
run ID + turn ID
projection snapshot ID
complete tool-definition-set digest
ordered AgentToolDefinition[]
```

Each `AgentToolDefinition` contains a unique model-visible name, exact description, complete validated input schema and schema digest, plus one opaque route reference unavailable to the model. Package, installation, grant, component kind, endpoint and executor configuration remain gateway-private.

The view is derived from one accepted `ToolProjectionSnapshot`; protocol constructors recompute all definition/set digests from canonical fields, while the current Agent-owned source boundary forbids projection-authority types and consumes only call/result values. The gateway still treats the view/call as untrusted and validates snapshot, route, schema and current authority. Definitions are included whole or omitted whole. Name collision fails before provider I/O.

### 3.2 Call envelope

A provider tool call is normalized against the same frozen view into:

```text
AgentToolCall {
  protocol_version,
  run_id,
  turn_id,
  provider_call_id,
  projection_snapshot_id,
  opaque_route_ref,
  canonical_arguments,
  argument_digest
}
```

The opaque route reference is copied from the matched definition and never synthesized from a visible name. The Agent treats it as uninterpreted data. Unknown name, stale snapshot, route mismatch, malformed arguments or changed definition fails closed.

### 3.3 Result envelope

The Agent receives only a normalized `AgentToolResult`:

```text
protocol version + correlated call ID
Succeeded | Failed | Denied | Cancelled | TimedOut
bounded model-visible content blocks
artifact/evidence references
receipt reference when an effect was attempted
stable error class + redacted diagnostics
```

The executor cannot forge a receipt or authoritative result envelope. `ToolGateway` validates the executor outcome, persists/observes the required receipt through runtime authority, bounds untrusted output and then constructs the result.

## 4. Gateway-to-executor boundary

A Plugin executor consumes a bounded `PluginExecutionRequest` containing only its admitted execution identity, platform call/effect/idempotency IDs, canonical arguments, tenant/object scope projection, resource/time/output limits, cancellation handle and allowed secret references. It returns a non-authoritative `PluginExecutionOutcome` with status, bounded content/artifact claims, usage and redacted diagnostics.

Execution transport is replaceable: reviewed MCP, a separately admitted process/tool host, WASI/OCI profile or another future protocol adapter. Transport state and handles never enter the Agent contract.

A `NativeRustComponent` MUST NOT be dynamically linked into `agent-runtime` or execute as an arbitrary in-process extension. Its first runnable package must define a separately versioned executor artifact/profile and pass schema, admission, isolation and rollback review. The current direct Course Planning CLI remains an offline spike, not Plugin execution evidence.

## 5. Package compilation

`PluginPackage` remains the distribution/lifecycle unit. Enabled package components compile into bounded contributions:

| Component | Runtime contribution |
|---|---|
| `SkillComponent` | bounded procedural/context asset; no executable authority |
| `DeclarativeResourcePack` | typed resource/context projection; no executable authority |
| `McpServerComponent` | discovered/admitted tool definitions plus executor routes after binding review |
| `NativeRustComponent` | tool definitions plus out-of-process admitted executor routes; never Agent linkage |

A package may bundle several contribution kinds. Packaging does not imply enablement, grant or tool visibility. Namespacing, schema validation, capability mapping, version/digest pinning and collision rejection occur before contribution enters a projection.

Hot reload/update creates a new package/component/binding identity and new turn projection. It never mutates an existing `AgentToolsetView`, in-flight `AgentRun` or accepted graph revision.

## 6. Dependency direction

```text
agent-runtime / harness ──► agent-tool-protocol ◄── gateway/executor adapters
                                      ▲
Market / Plugin domain ──► resolver ──┘

ustc-agentd composition root ──► Agent + resolver + gateway + adapters
```

Normative rules:

1. `agent-runtime` and future harness code MUST NOT depend on Market manifests, Plugin domain types, component implementations, adapter crates or framework extension APIs; Cargo targets, modules and code inclusion remain confined to the owned Agent crate tree.
2. Plugin implementations MUST NOT depend on Agent phase/graph/checkpoint internals. They may depend only on the executor-side protocol/SDK and their own domain libraries.
3. The composition root is the only layer allowed to depend on both Agent and Plugin/resolver sides.
4. Cross-boundary integration tests belong at the composition root, not inside either independent module.
5. Package/component provenance may remain as opaque IDs/digests in audit-compatible `agent-run/v0`; Agent code MUST NOT parse them or branch on component kind.
6. The dedicated protocol crate exists because H0 has Agent-side and fake-gateway consumers; it contains only wire/domain-neutral canonical values and envelopes.

## 7. Invocation ordering

```text
resolve current package/installation/grant state
→ freeze ToolProjectionSnapshot
→ derive AgentToolsetView and private ToolRouteTable
→ provider emits a raw call name/id/argument payload
→ composition binds it through frozen AgentToolsetView::bind_call into AgentToolCall
→ M30 persists ToolCallProposal::from(bound AgentToolCall)
→ staged M40 prepares/correlates the bound AgentToolCall
→ recheck current deny-side authority
→ persist EffectIntent
→ execute PluginExecutor through bounded adapter
→ persist EffectReceipt
→ normalize AgentToolResult
→ append tool result to next model projection
```

No Plugin callback may run before authorization and effect-intent persistence. A denied call produces no executor request. A process exit, hook or Plugin-returned success string cannot replace the receipt/result sequence.

### 7.1 M20-B7-B staged composition evidence; accepted contract, implementation planned

B7-B is a composition-root integration-test contract. It creates no production M40 crate/application code, no durable journal and no alternative authority. Its exact test-only support types are:

```text
StagedFakeToolGateway
PreparedFakeToolExecution
IdempotentFakePluginExecutor
InMemoryFakeRunJournal
EffectCompositionHarness
CompositionTraceEvent
```

They live only under `apps/ustc-agentd/tests/support/`. Exact test-visible methods are:

```text
StagedFakeToolGateway::prepare / complete
PreparedFakeToolExecution::authorized_invocation / protocol_call / effect_binding_digest
IdempotentFakePluginExecutor::execute_or_reconcile / lookup_disposition / fail_next_attempt / attempt_count / unique_effect_count
InMemoryFakeRunJournal::execute / fail_next_intent_persist / fail_next_receipt_persist / snapshot / events
EffectCompositionHarness::execute_call / reconcile_pending_effect / trace
```

`CompositionTraceEvent` has exactly these fieldless variants in order:

```text
ToolCallProposalPersisted
CallPrepared
EffectIntentPersisted
ExecutorAttempted
ExecutorDispositionObserved
EffectReceiptPersisted
ResultReturned
```

All other support values above use manual bounded/redacted `Debug` and never print canonical arguments, execution identities, routes, pending effects, receipts or executor dispositions. `CompositionTraceEvent` may derive exactly `Debug, Clone, Copy, PartialEq, Eq`.

The success path is exact:

```text
provider raw call binds through frozen AgentToolsetView::bind_call into AgentToolCall
→ M30 ToolCallProposal::from(bound AgentToolCall) persisted
→ staged fake M40 correlates the already-bound AgentToolCall
→ M20 InvocationAuthorityService transaction-current recheck succeeds
→ PreparedFakeToolExecution returned with executor count zero
→ composition derives one exact M30 EffectIntent
→ fake M30 journal decides, validates on a cloned checkpoint, persists, then swaps the clone
→ idempotent fake executor observes one sealed request
→ composition derives one exact M30 EffectReceipt
→ fake M30 journal decides, validates on a cloned checkpoint, persists, then swaps the clone
→ staged fake M40 returns one correlated AgentToolResult
```

The fake journal performs no second fallible apply after persistence. `AlreadyApplied` appends nothing. Intent persistence failure leaves event/checkpoint unchanged and reaches no executor. Receipt persistence failure/uncertainty returns no `AgentToolResult`.

`AgentToolResult` reuses the original provider call ID and the persisted receipt's exact outcome kind/digest. It never re-hashes raw executor output, substitutes a failure class or returns raw output/logs. Any call/effect/idempotency/outcome mismatch fails before result construction.

Receipt uncertainty follows a separate reconciliation path:

1. replay/load the exact pending M30 `EffectIntent`;
2. query executor disposition by exact effect/idempotency identity without starting a new effect;
3. persist the exact matching receipt when known;
4. remain unresolved with no result when unknown;
5. fail closed with no result on conflict.

Current denial blocks new execution but cannot erase or reinterpret an already observed disposition. Reconciliation performs no new current authorization and no non-idempotent effect retry. Executor attempts and unique effects are counted separately; the unique-effect count remains one.

Before proposal or intent persistence, every projection/run/turn mismatch or unknown tool produces no M30 proposal, no intent, no executor request, no receipt and no result. After a call is bound, malformed arguments, route/dispatch mismatch, catalog revoke, absent/Disabled/Revoked installation, absent/Stale/Expired/Revoked grant, emergency block, post-update old carrier mismatch and repository conflict/corruption produce no intent, executor request, receipt or result.

Disable/update/revoke evidence composes existing owners rather than recreating them:

- Disable executes the accepted A1 façade, obtains its owner receipt projection and updates the semantic current-resolver fixture only by preserving immutable installation/package/component identity and copying the receipt's exact post-state/revision; mismatch is rejected.
- B6 owner tests prove Apply/Rollback future-carrier mutation and prior-grant staling; B5 tests prove owner-to-resolver mapping and transaction-current recheck.
- B7-B freezes the old `AgentToolsetView`, mutates only semantic current authority, proves the old call is denied before intent/I/O and new projection is denied until exact fresh enable/grant authority exists, then proves the fresh projection binds the target pin while the old view remains byte/typed-equal.
- No external test may mint B6 owner-private evidence; checker-bound cross-file evidence must preserve the exact postcondition without claiming one test executed private constructors.

Exact B7-B tests are:

```text
authorized_call_persists_intent_before_executor_and_receipt_before_result
pre_intent_denials_persist_nothing_and_reach_no_executor
intent_persistence_failure_reaches_no_executor
executor_failure_is_receipted_before_failed_result
receipt_uncertainty_returns_no_result_and_retry_deduplicates_effect
disable_and_revoke_preserve_frozen_view_but_deny_old_calls_and_new_projection
package_update_preserves_in_flight_view_and_requires_fresh_projection_authority
```

`tool_gateway_conformance.rs` must reuse staged support rather than retaining a second monolithic fake. Test functions remain active (`#[test]`, no ignore/cfg/zero-test), use load-bearing calls at top-level body depth and assert exact ordering/correlation. Contract acceptance itself implements none of these types/tests and promotes neither `MARKET-007` nor `PKG-020`.

## 8. Compatibility and replacement

- An Agent framework/harness/provider-loop update that preserves `agent-tool-protocol/v0` requires no Plugin package or executor change.
- A Plugin package/executor update that preserves the executor protocol requires no Agent kernel change; exact package version and projection identity still change.
- A protocol-breaking change requires a new major protocol version, explicit compatibility matrix and either dual-version adaptation or an atomic migration. Silent reinterpretation is forbidden.
- In-flight runs retain their pinned protocol/projection. New versions affect new turns/runs only under policy.
- Agent and Plugin sides each have standalone conformance fixtures using fake counterparts. Integration fixtures test only composition and ordering.

Replaceability is accepted only when both directions are demonstrated: the Agent runs against a fake tool port with no Plugin dependency, and a packaged executor passes conformance without importing Agent internals.

## 9. Failure isolation

- Plugin load/discovery/schema failure quarantines that component and omits its tools; the Agent kernel remains usable.
- Agent upgrade failure does not rewrite installed package state or package artifacts.
- Executor crash/timeout yields one typed result/receipt path and cannot crash the authority process.
- Plugin output is bounded untrusted data; it cannot inject system policy or tool definitions into the current turn.
- Gateway mapping/correlation failure blocks execution; it never falls back by visible name, package or transport.
- Disable/revoke prevents new projections/calls while preserving historical receipts and pinned evidence.

## 10. Reference synthesis

| Reference | Borrow | Adapt | Reject |
|---|---|---|---|
| [Pi coding agent extensions/packages](https://github.com/badlogic/pi-mono/tree/main/packages/coding-agent) | minimal Agent core, registered tools, versioned distributable packages and independent extension update | compile reviewed package components into immutable platform tool views/routes | arbitrary TypeScript extension access, mutable hot-loaded tools or package trust as authorization |
| [Pi Agent core](https://github.com/badlogic/pi-mono/tree/main/packages/agent) | provider-neutral tool definitions/calls/results and explicit pre/post tool event barriers | owned envelopes plus intent/receipt ordering | mutable Agent state/tool array as canonical platform state |
| [Claude Code plugins](https://code.claude.com/docs/en/plugins) and [reference](https://code.claude.com/docs/en/plugins-reference) | self-contained versioned bundle, namespacing, component packaging and MCP tools entering the common tool surface | `PluginPackage` plus reviewed resolver/gateway compilation | Plugin hooks/settings/agents mutating the central authority loop or unsandboxed commands as trusted execution |

This is design borrowing, not dependency adoption. No Pi or Claude Code runtime code is linked.

## 11. Current status

Implemented now:

- `agent-tool-protocol/v0` owns canonical schema/arguments, frozen toolset views, correlated calls and typed digest/code results without Agent, Market, Plugin, framework or transport dependencies;
- `agent-runtime` depends only on that protocol seam plus registry value dependencies, with no Market, Plugin or adapter implementation dependency;
- P0a produces immutable per-turn tool projections and fail-closed call authorization;
- composition-root synthetic proof maps successful resolution into `RunSpec`, while denial creates no run;
- composition-root fake gateway maps the frozen private route through current-state `authorize_call`; unknown tool, malformed arguments, route mismatch, current denial and projection mismatch reach no fake executor;
- repository checks enforce the exact protocol path and current dependency direction.

Accepted contract, not implemented:

- B7-B staged composition-root semantic support, exact intent/executor/receipt/result order, receipt reconciliation, denial/update fixtures and seven exact tests under §7.1;
- this contract state adds no production ToolGateway, executor request/outcome implementation, durable journal or acceptance promotion.

Planned later:

- production ToolGateway/application service and provider adapter;
- bounded content/artifact/receipt result expansion beyond the current digest/code H0 subset;
- executable Plugin packaging/tool-host conformance;
- durable intent/receipt composition and real invocation;
- independent Agent/framework replacement conformance.
