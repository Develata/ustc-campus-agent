# Large-module boundary registry

## Metadata

- `Status`: Current contract
- `Version`: `module-boundaries/v0`
- `Last Review`: `2026-07-25`
- `Owning Plan`: [`../plan/modules/00-module-map.md`](../plan/modules/00-module-map.md)
- `Task Policy`: [`../tasks/00-module-work-policy.md`](../tasks/00-module-work-policy.md)

This registry defines what may cross each large-module boundary. It does not prescribe private structs, storage schemas or framework implementations.

## 1. Boundary rules

1. Every cross-module call uses a named command, query, event or port contract.
2. Public values contain stable IDs, bounded data and typed errors; they do not expose private fields or concrete adapter handles.
3. A module may validate its own public input again. Callers cannot claim that prior UI/transport validation is authoritative.
4. The module that owns a state transition owns its legal command/event/result. A composition root may order calls but cannot synthesize an accepted transition.
5. Cross-module errors are mapped explicitly. No failure selects a same-name module/tool/provider/runtime fallback.
6. Public contracts are versioned before persistence or multiple independent consumers make compatibility necessary.
7. Fake counterparts implement the same public contract and are required before large-module standalone acceptance.
8. Cyclic large-module code dependencies are forbidden.

## 2. Current boundary registry

| Boundary ID | Producer/owner | Consumer | Values/operation | Status |
|---|---|---|---|---|
| `B-M00-M10-ACTOR` | `M00` | `M10` | `AuthenticatedActor`, `PlatformRequestContext`, session denial | planned |
| `B-M10-M80-API` | `M10` | `M80` | versioned HTTP JSON request/response/error and SSE event/cursor | accepted contract; implementation planned |
| `B-M10-APP-COMMAND` | owning backend modules | `M10` | typed application commands/queries/results; transport mapping only | planned per route |
| `B-M20-M40-PROJECTION` | `M20` | `M40` | immutable tool projection, private route and current authorization result | partial implementation |
| `B-M30-M50-MODEL` | `M30` | `M50` | complete model request, ordered events, usage and typed provider errors | planned |
| `B-M20-M30-TOOLSET` | `M20` resolver via composition | `M30`/provider | frozen `AgentToolsetView` only; no private route/package/grant internals | partial implementation |
| `B-M30-M40-CALL` | `M30` via composition | `M40` | correlated `AgentToolCall` proposal for staged prepare; no executor I/O yet | protocol/fake proof implemented; production planned |
| `B-COMP-M30-EFFECT` | `M30` owns; composition issues | `M30` | approve/persist-intent/persist-receipt/result commands under current run phase | node runtime implemented; production composition planned |
| `B-M40-M30-RESULT` | `M40` via composition | `M30` | validated bounded `AgentToolResult` carrying the persisted receipt reference when execution was attempted | protocol/fake proof implemented; production planned |
| `B-M40-M51-EXEC` | `M40` | `M51` | bounded `PluginExecutionRequest`/`PluginExecutionOutcome` | planned |
| `B-M60-M70-CHANGE` | `M60` | `M70` | accepted revision/fact pair, provenance/freshness/conflict | planned |
| `B-M60-M71-PROC` | `M60` | `M71` | accepted facts/revisions supporting procedure candidates | planned |
| `B-M60-M72-OPPORTUNITY` | `M60` | `M72` | reviewed opportunity/course facts with provenance/time/conflict | fixture subset only |
| `B-M00-M72-PROFILE` | `M00` | `M72` | exact tenant/user/request context for private profile operations | planned |
| `B-DOMAIN-M90-PORTS` | each domain module | `M90` | repository, journal, artifact, clock, scheduler, secret-ref, HTTP and telemetry ports | mostly planned |

`B-M10-APP-COMMAND` is a boundary family, not one universal command bag. Each route/use case declares its owning module and exact value contract.

## 3. Client boundary

`M80` may receive:

- API/version/build information;
- safe Market/install/run/product projections;
- stable error codes and user-safe messages;
- monotone event sequence/cursor;
- exact intent preconditions and server capability availability.

`M80` may send:

- user-entered form values under the route schema;
- one typed user intent;
- current projection/precondition identity;
- correlation/idempotency identity;
- reconnect cursor.

It must not receive or send domain repositories, grant internals, executor routes/config, provider secrets, raw audit payloads or mutable server objects. Client-side calculations may support display only; backend/application modules recompute every truth-affecting decision.

## 4. Agent–Plugin boundary

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

In-flight runs/calls keep their pinned contract/snapshot. New versions affect new calls/turns/runs under explicit policy.

## 8. Verification

The registry is accepted when:

- module plans and code dependency checks agree;
- API/tool/executor/source/profile ports have named owners;
- no client/domain/framework type leaks across a forbidden boundary;
- fakes prove modules can develop independently;
- composition tests prove exact mapping and failure ordering;
- `docs/coverage-matrix.md` and `docs/acceptance/matrix.tsv` expose each boundary's real status.
