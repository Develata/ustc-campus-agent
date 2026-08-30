# Agent–Plugin tool boundary contract

## Metadata

- `Status`: Accepted target architecture; protocol values/conformance plus bounded Affairs and ChangeRadar fixed-adapter ToolGateway compositions implemented, generic/package-portable production ToolGateway planned
- `Version`: `agent-plugin-boundary/v0`
- `Last Review`: `2026-07-24`
- `Owning Plans`: [`04-market-and-plugin-lifecycle.md`](../plan/04-market-and-plugin-lifecycle.md) owns package lifecycle; [`07-runtime-and-integration.md`](../plan/07-runtime-and-integration.md) owns Agent/tool execution
- `Decisions`: [`ADR-0008`](../adr/0008-agent-plugin-tool-boundary.md)
- `Acceptance`: implemented `AGENT-017`, `AGENT-019`; planned `AGENT-018`, `PKG-019`, `PKG-020`
- `Primary Code`: `crates/agent-tool-protocol/`, `crates/agent-runtime/`, `crates/platform-core/src/invocation.rs`, `apps/ustc-agentd/src/{affairs,change}_invocation.rs`, `apps/ustc-agentd/tests/`

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

The first-party MVP may use one narrower bootstrap profile inside the composition
root: an exact, statically linked, read-only first-party adapter may execute only
for a hard-coded package/component/tool identity, a `PublicRead` capability, no
secret references and no external side effect. It is still gated by current
Market projection/recheck plus persisted Agent effect intent/receipt ordering;
it admits no dynamic code or visible-name fallback. This profile is bounded
composition evidence, not portable `NativeRustComponent` isolation evidence.
Community packages, secrets, writes and arbitrary native executors remain
out-of-process-only.

## 5. Package compilation

`PluginPackage` remains the distribution/lifecycle unit. Enabled package components compile into bounded contributions:

| Component | Runtime contribution |
|---|---|
| `SkillComponent` | bounded procedural/context asset; no executable authority |
| `DeclarativeResourcePack` | typed resource/context projection; no executable authority |
| `McpServerComponent` | discovered/admitted tool definitions plus executor routes after binding review |
| `NativeRustComponent` | tool definitions plus out-of-process admitted executor routes; never Agent linkage, except the fixed read-only first-party bootstrap profile above |

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
→ provider or deterministic bounded Harness proposes a tool call
→ normalize against frozen view
→ recheck current deny-side authority
→ AgentRun accepts proposal
→ persist EffectIntent
→ execute PluginExecutor through bounded adapter
→ persist EffectReceipt
→ normalize AgentToolResult
→ append tool result to next model projection
```

No Plugin callback may run before authorization and effect-intent persistence. A denied call produces no executor request. A process exit, hook or Plugin-returned success string cannot replace the receipt/result sequence.

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
- composition-root synthetic/fake proofs remain standalone conformance evidence;
- the bounded Affairs product path maps M00 actor identity into exact installation/grant authority, performs the current deny-side recheck before `AgentRun` accepts the proposal, persists intent before the fixed read-only adapter, persists success/failure receipts after the attempt and digests the versioned M10 terminal JSON carrier;
- disabled installation, revoked grant, unknown tool, malformed arguments, route mismatch, current denial and projection mismatch reach no executor, with no direct-M71 fallback;
- repository checks enforce the exact protocol path and current dependency direction.

Planned later:

- generic/package-portable production ToolGateway, out-of-process executor host and provider adapter beyond the fixed Affairs bootstrap profile;
- bounded content/artifact/receipt result expansion beyond the current digest/code H0 subset;
- executable Plugin packaging/tool-host conformance;
- durable intent/receipt journal and restart replay beyond the current in-memory ordering proof;
- independent Agent/framework replacement conformance.
