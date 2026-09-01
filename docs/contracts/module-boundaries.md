# Large-module boundary registry

## Metadata

- `Status`: Current contract
- `Version`: `module-boundaries/v3.1`
- `Last Review`: `2026-09-01`
- `Owning Plan`: [`../plan/modules/00-module-map.md`](../plan/modules/00-module-map.md)
- `Task Policy`: [`../tasks/00-module-work-policy.md`](../tasks/00-module-work-policy.md)

This registry defines what may cross each large-module boundary. It does not prescribe private structs, storage schemas or framework implementations.

## 1. Boundary rules

1. Every cross-module call uses a named command, query, event or port contract.
2. M00-owned identity values use [`platform-identity/v0`](platform-identity.md). Other public values likewise contain stable IDs, bounded data and typed errors; they do not expose private fields or concrete adapter handles.
3. A module may validate its own public input again. Callers cannot claim that prior UI/transport validation is authoritative.
4. The module that owns a state transition owns its legal command/event/result. A composition root may order calls but cannot synthesize an accepted transition.
5. Cross-module errors are mapped explicitly. No failure selects a same-name module/tool/provider/runtime fallback.
6. Public contracts are versioned before persistence or multiple independent consumers make compatibility necessary.
7. Fake counterparts implement the same public contract and are required before large-module standalone acceptance.
8. Cyclic large-module code dependencies are forbidden.
9. M10 owns the versioned client wire schema; M80 owns client behavior and produces conforming request instances. A shared M10 protocol carrier cannot depend on M80 client-core or outer adapter types.

## 2. Current boundary registry

| Boundary ID | Producer/owner | Consumer | Values/operation | Status |
|---|---|---|---|---|
| `B-M00-M10-ACTOR` | `M00` | `M10` | [`platform-request-context/v0`](platform-request-context.md): closed `M00AdmittedActor::{Public, Authenticated}`, sealed `PlatformRequestContext`, complete scalar `M00AdmittedDisposition`, typed rejection/incomplete outcome | M00 bounded producer implemented (`AUTH-013`); M10-v17 composition/wire runtime planned |
| `B-M80-M10-CALL` | M80 produces request instances; M10 owns the versioned wire schema | `M10` | protocol-major bootstrap/admission plus versioned client-core request; the first closed subset is Web/CLI `server.info`, `capability.list`, `affairs.get` | approved Affairs-first major-1/header/route slice plus bounded existing `affairs get/lookup`; production auth/SSE, generic conformance and other adapters planned |
| `B-M10-M80-RESULT` | `M10` | `M80` | typed `server.info`, safe capability projection, typed domain result/error and exact `upgrade_required` or `incompatible_protocol` outcome | approved Affairs-first compatibility reduction plus bounded existing Affairs response/provenance and canonical CLI JSON evidence; full peer matrix planned |
| `B-M10-M80-EVENT` | `M10` | `M80` | typed server event/stream value, monotone cursor and reconnect/resync outcome shared semantically across peer adapters | accepted contract; implementation planned |
| `B-M10-APP-CALL` | `M10` | owning backend application module | one admitted typed application command/query; no transport or Dioxus type | bounded fixture-backed M71 Affairs plus four static M72 private-operation command/query paths; other operations and production ingress planned |
| `B-APP-M10-RESULT` | owning backend application module | `M10` | typed application result/error/event projection; no concrete adapter handle | bounded M71 Affairs plus typed static M72 result/error evidence; other operations and production ingress planned |
| `B-M20-M40-PROJECTION` | `M20` | `M40` | immutable tool projection, private route and current authorization result | partial implementation |
| `B-M20-M72-AUTH` | `M20` | statically composed owning `M72` application use case | transaction-current package, declarative resource component, installation, grant, capability class/scope and policy authorization for one exact registered application operation; no Agent tool projection, provider call or executor route | bounded fixture-backed implementation for the four private Opportunity operations; durable production M20 adapter planned |
| `B-M30-M50-MODEL` | `M30` | `M50` | complete model request, ordered events, usage and typed provider errors | planned |
| `B-M20-M30-TOOLSET` | `M20` resolver via composition | `M30`/provider | frozen `AgentToolsetView` only; no private route/package/grant internals | partial implementation |
| `B-M30-M40-CALL` | `M30` via composition | `M40` | correlated `AgentToolCall` proposal for staged prepare; no executor I/O yet | protocol/fake proof implemented; production planned |
| `B-COMP-M30-EFFECT` | `M30` owns; composition issues | `M30` | approve/persist-intent/persist-receipt/result commands under current run phase | node runtime implemented; production composition planned |
| `B-M40-M30-RESULT` | `M40` via composition | `M30` | validated bounded `AgentToolResult` carrying the persisted receipt reference when execution was attempted | protocol/fake proof implemented; production planned |
| `B-M40-M51-EXEC` | `M40` | `M51` | bounded `PluginExecutionRequest`/`PluginExecutionOutcome` | planned |
| `B-M60-M70-CHANGE` | `M60` | `M70` | accepted revision/fact pair, provenance/freshness/conflict | planned |
| `B-M60-M71-PROC` | `M60` | `M71` | accepted facts/revisions supporting procedure candidates | planned |
| `B-M60-M72-OPPORTUNITY` | `M60` | `M72` | reviewed opportunity/course facts with provenance/time/conflict | bounded `DemoReviewed` revision-health fixture subset only; approved live-source port planned |
| `B-M00-M72-PROFILE` | `M00` | `M72` | exact tenant/user/request context for private profile operations | bounded authenticated demo-session composition implemented; production SSO/session authority planned |
| `B-M60-M90-SOURCE-TRANSPORT` | `M60` | `M90` | M60 produces owned move-only `RetrievalTransportRequest` from `EffectReadyRetrievalPlan`; M90 implements `SourceTransportPort::transport`, performs DNS/TLS/HTTP/framing/body I/O under M60-provided bounds, returns only `RetrievalTransportSuccess` or transport-only `SourceTransportError`; M90 never receives `EffectReadyRetrievalPlan`, never names domain `SourceFetchFailure`, and never returns `BoundedFetch` | accepted contract; implementation planned |
| `B-DOMAIN-M90-PORTS` | each domain module | `M90` | repository, journal, artifact, clock, scheduler, secret-ref, HTTP and telemetry ports; M00's exact session repository/clock/credential-evidence subset is [`platform-session-port/v0`](platform-session-port.md), and its redacted evidence read/append-once subset is [`platform-control-evidence/v0`](platform-control-evidence.md) | bounded B4a session ports plus one app-private durable DemoReviewed current-session read/bootstrap vendor and bounded B4b data-only evidence/journal interfaces are implemented (`AUTH-021`, `AUTH-022`); durable lifecycle/evidence adapters, formal authentication and B5 coupling remain planned |

`B-M10-APP-CALL` and `B-APP-M10-RESULT` are boundary families, not universal command/result bags. Each server function or public route declares one owning application module and exact value contract. Dioxus/Axum transport types terminate in the M10 adapter.

## 3. Client-core and peer-shell boundary

M80 owns one framework-neutral client core plus Dioxus, `ustc-agent` and inbound MCP outer adapters. `ustc-agentctl` is a separate operator/developer surface and is not reachable through these peer clients.

M80 may receive:

- API/version/build and server capability information;
- server/client compatibility and minimum-supported-version outcomes;
- safe Market/install/run/product projections;
- stable error codes and user-safe messages;
- monotone event sequence/cursor;
- exact intent preconditions and server capability availability.

M80 may send:

- user/automation/MCP input under the registered operation schema;
- one typed user intent;
- current projection/precondition identity;
- correlation/idempotency identity;
- client build/target/protocol identity;
- reconnect cursor.

It must not receive or send domain repositories, grant internals, executor routes/config, provider secrets, raw audit payloads, mutable server objects, operator credentials or M51 session handles. Client-side calculations may support display, framing and safe local validation only; backend/application modules recompute every truth-affecting decision.

Dioxus server functions and explicit HTTP/SSE routes are valid peer M10 ingress adapters. After M00/M10 admission they may issue `B-M10-APP-CALL` and map `B-APP-M10-RESULT`; they may not reach concrete repositories, executors, providers or journals. M10 and M80 may both consume the M10-owned framework-neutral protocol carrier; M80 client-core depends on it, while M10 never depends on client-core. Dioxus, `ustc-agent` and inbound MCP share client-core semantics and fake-M10 fixtures but never spawn or parse one another as their production path.

The MCP directions are exact:

```text
external Agent → M80 inbound MCP adapter → client-core → M10
M40 → M51 outbound MCP binding/executor → external MCP server
```

The inbound adapter cannot invoke M51 or inherit operator commands/credentials. Independently deployed Android, CLI and inbound-MCP artifacts require compatibility fixtures and typed rejection before an unsupported request reaches `B-M10-APP-CALL`.

For the first retained Affairs-first seam, `server.info` is header-free bootstrap; every other admitted route requires major `1`. M10 alone classifies older as `upgrade_required` and newer/absent/unparseable as `incompatible_protocol`; the HTTP adapter projects `426`/`409`, respectively. The capability projection is a closed Web/CLI allowlist and is not dispatch authority. M80 preserves the typed classification under `ustc-client-result/v1` and never derives domain or compatibility truth from HTTP status.

## 4. Agent–Plugin boundary

The four current Opportunity private operations do not cross this boundary. They
follow `M00/M10 → B-M20-M72-AUTH → static owning M72 application use case`; no
`AgentToolsetView`, `AgentToolCall`, provider identity, M30 intent/receipt command,
M40 route or Plugin executor request is created. This exception is a classification
of static first-party application operations, not a weakening of the generic M40
contract below.

The only Agent-facing tool family is `agent-tool-protocol/v0`:

```text
AgentToolsetView
AgentToolCall
AgentToolResult
```

The Agent never receives package/component/grant/executor identities except opaque audit-compatible IDs explicitly allowed by the run contract. Executors never receive Agent graph/checkpoint/prompt authority. The production sequence remains:

```text
M20 projection/recheck
→ M30 proposal
→ composition invokes M40 prepare
→ composition records M30 intent
→ M51 or peer executor
→ composition records M30 receipt
→ M40 bounded correlated result
→ M30 result state
```

`M30` and `M40` both depend on the shared protocol, not on each other's implementation. Composition depends on their public interfaces and performs the interleaving. `M40` may call public `M20` authority interfaces but MUST NOT call `M30` or mutate its journal.

### 4.1 RunExecutionCoordinator

The composition interleaving above is owned by an application-level `RunExecutionCoordinator`, defined in [`plan/07-runtime-and-integration.md`](../plan/07-runtime-and-integration.md) §14. It lives in `ustc-agentd` or a declared application module, not inside `M30`, `M40` or `M20`.

- It MAY issue `M30` journal commands and `M40` staged execution ports.
- It owns outbox/effect identity and uncertain-receipt reconciliation across the prepare → execute → receipt sequence above. An uncertain receipt (timeout/crash before confirmation) remains `uncertain`; it is never silently advanced and never treated as safe-to-retry or not-completed. Reconciliation by stable effect identity/idempotency key and executor status/receipt lookup is required; only a proven non-execution outcome permits retry.
- It MUST NOT mutate `M30` run/graph/context state, `M40` route/gateway state, or `M20` installation/grant state directly.
- It MUST NOT create a direct implementation dependency cycle (`M30 → M40 → M20 → M30` is forbidden); it composes their public ports.
- A production `RunExecutionCoordinator` lands only when `M30`/`M40` production integration is admitted; its acceptance evidence binds to the composition root, not to either module's standalone gate.

## 5. Domain–infrastructure boundary

Each owning module declares its own semantic port. `M90` may implement it but may not merge unrelated ports into a generic record store or arbitrary query API.

Examples:

```text
RunJournal.append(expected_revision, event)
InstallationRepository.load_exact(...)
EvidenceStore.put_verified(digest, bounded_bytes)
SafeHttpClient.fetch(approved_request)
SecretResolver.resolve(admitted_ref)
```

Forbidden examples:

```text
Database.execute_arbitrary(sql)
HttpProxy.request(user_url, headers, body)
Runtime.run(command, args, env)
MutableContext.get_any(type_name)
```

## 6. Fake and conformance contract

Before a large module is `StandaloneReady`:

- every outbound port has a deterministic fake;
- every inbound public contract has success and failure fixtures;
- fakes record whether forbidden I/O was reached;
- duplicate, timeout, malformed, stale/revoked and cancellation cases exist where applicable;
- no fake silently accepts operations that production must reject;
- public contract tests can be reused by alternate implementations.

Cross-module tests at the composition root prove only mapping and ordering. They do not replace standalone module tests.

## 7. Change and compatibility

A change requires a new major contract version or explicit migration when it reinterprets persisted/public data, removes a required field/variant, changes authority ownership or permits behavior previously denied.

Compatible additive changes still require:

- owning plan review;
- client/provider/executor unknown-variant behavior;
- fixture and registry update;
- acceptance evidence for old and new peers where both remain supported.

Web may deploy atomically with the server. Android, `ustc-agent` and inbound MCP do not necessarily do so; each independently deployed artifact therefore has an explicit compatibility obligation even when shared Rust types exist.

In-flight runs/calls keep their pinned contract/snapshot. New versions affect new calls/turns/runs under explicit policy.

## 8. Verification

The registry is accepted when:

- module plans and code dependency checks agree;
- API/tool/executor/source/profile ports have named owners;
- no client/domain/framework type leaks across a forbidden boundary;
- fakes prove modules can develop independently;
- composition tests prove exact mapping and failure ordering;
- `docs/coverage-matrix.md` and `docs/acceptance/matrix.tsv` expose each boundary's real status.
