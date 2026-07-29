# Large-module map and assembly contract

## Metadata

- `Layer`: Large-module architecture
- `Status`: Accepted current skeleton
- `Version`: `module-map/v2`
- `Last Review`: `2026-07-26`
- `Owning Constitution`: [`../00-engineering-constitution.md`](../00-engineering-constitution.md)
- `Counterpart Contract`: [`../../contracts/module-boundaries.md`](../../contracts/module-boundaries.md)
- `Counterpart Tasks`: [`../../tasks/00-module-work-policy.md`](../../tasks/00-module-work-policy.md), [`../../tasks/01-execution-roadmap.md`](../../tasks/01-execution-roadmap.md)

A large module is an independently owned and independently testable part. “Large” means responsibility and lifecycle independence, not a minimum code size.

## 1. Module registry

| ID | Large module | State key | Owns | Must not own | Current state |
|---|---|---|---|---|---|
| `M00` | Platform Control and Identity | `partial-evidence` | tenant/user/session identity, request causation, application command envelope, platform-wide policy references | package internals, Agent loop, source parsing, UI | identity-types and session-domain implemented; request-context/ports planned |
| `M10` | Application Ingress Host | `skeleton` | Dioxus/Axum server-function ingress, versioned public HTTP/typed streams where needed, auth/session/client-compatibility admission, mapping to application ports | domain decisions, direct database/executor rules, UI | skeleton only |
| `M20` | Market and Package Lifecycle | `partial-evidence` | catalog package/component identity, install/configure/enable/disable/update/revoke/grant lifecycle, invocation authority snapshots | model loop, Plugin execution, client state | typed package/catalog + capability-registry + bounded managed-installation fake + pure resolver evidence; durable lifecycle/composition planned |
| `M30` | Agent Harness and Runtime | `partial-evidence` | finite user-task/run phases, graph, budgets, context projection, provider/tool ports, replay and review | package lifecycle, executor implementation, transport/UI | node runtime kernel implemented; harness planned |
| `M40` | Tool Gateway and Execution | `partial-evidence` | Agent tool protocol mapping, call correlation, current authorization ordering, intent/executor/receipt/result sequence | grants, Agent phases, package declarations | protocol values + fake conformance implemented |
| `M50` | Model Provider Integration | `planned` | typed provider profiles, request/stream normalization, token estimation, timeout/cancel/error mapping | run authority, tool grants, prompt truth | planned |
| `M51` | MCP Binding and Executor | `planned` | reviewed MCP endpoint/component binding, discovery/schema snapshot, protocol session, bounded tool execution | Market publication, grant decisions, Agent loop | planned |
| `M60` | Campus Trust and Source Pipeline | `planned` | source registry, retrieval policy, immutable revision, normalization, provenance, accepted baseline and publication gate | product-specific rendering, arbitrary crawling, UI | contract only; planner fixtures provide limited evidence |
| `M70` | USTC ChangeRadar | `design-only` | semantic change candidates/events, board scope, approval and feed behavior | generic source authority, other product state | manifest/design only |
| `M71` | USTC Affairs Navigator | `design-only` | reviewed tree/procedure artifacts, lookup, supersession and publication journey | generic source authority, full-corpus RAG as truth | manifest/design only |
| `M72` | Campus Opportunity Graph | `bounded-spike` | opportunity facts, qualification/dependency/conflict, tenant profile projection and planning journeys | public source authority, cross-user profile state | offline Course Planning spike only |
| `M80` | Dioxus Fullstack Multi-client | `planned` | mandatory Web/PWA and Android routes/components/presentation state, SSR/hydration, generated first-party client facade and target adapters; later iOS/desktop | domain calculation/mutation, direct repository/executor access, Agent/Market/Plugin/source authority | architecture accepted; no Fullstack app |
| `M90` | Platform Infrastructure and Operations | `governance-baseline` | repository implementations for storage, journal, evidence, clock, queue, config, secrets, telemetry and Docker Compose deployment/recovery wiring | domain transition rules and product policy | CI/checker baseline only |

`State key` is the machine-checked implementation-evidence posture shared with every module blueprint and the roadmap lane registry:

- `planned` — no retained implementation owned by this module;
- `skeleton` — a runnable/composition shell exists but the module behavior is not implemented;
- `partial-evidence` — bounded executable evidence exists for only part of the module;
- `design-only` — package/design declarations exist without module implementation;
- `bounded-spike` — explicitly non-authoritative exploratory implementation exists;
- `governance-baseline` — repository/CI governance evidence exists without the planned production module.

The prose `Current state` explains the exact evidence and non-claims. It MUST NOT contradict or silently replace the controlled key.

The registry is the current large-module ownership boundary. New top-level modules or ownership moves are skeleton changes and require the analysis/approval process in the engineering constitution.

## 2. Dependency direction

```text
M80 Dioxus Web/Android clients
  └── versioned server-function/HTTP call ──► M10 Application Ingress Host

M10 Application Ingress Host
  └── typed result/error/event ──► M80 presentation reducer

M10 Application Ingress Host
  └── admitted typed application calls ──► M00 / M20 / M30 / M60 / M70 / M71 / M72

M30 Agent Harness and Runtime
  ├── model port ──► M50 Model Provider Integration
  └── emits/accepts Plugin-neutral tool proposal/result values at composition

ustc-agentd composition
  └── invokes staged tool operations ──► M40 Tool Gateway and Execution

M40 Tool Gateway and Execution
  ├── authority query ──► M20 Market and Package Lifecycle
  ├── MCP executor port ──► M51 MCP Binding and Executor
  └── other admitted executor ports ──► future reviewed peers

M70 ChangeRadar ─┐
M71 Affairs      ├── typed source/fact ports ──► M60 Campus Trust and Source Pipeline
M72 Opportunity ─┘

M90 Platform Infrastructure and Operations
  └── implements storage/clock/queue/secret/telemetry ports declared by owning modules
```

Dependency rules:

1. `M80` client target code never imports domain crates or storage/executor/provider clients.
2. `M10` maps Dioxus server-function/public transport to application commands; it does not reimplement module decisions or force a loopback HTTP hop.
3. `M30` and `M40` depend on the Plugin-neutral tool protocol, not on each other's implementation; `ustc-agentd` orders their public operations.
4. `M40` coordinates existing decisions but does not mint grants, run phases or receipts by itself.
5. `M50` and `M51` normalize external protocols and never own platform state transitions.
6. `M70`, `M71` and `M72` may depend on `M60` contracts but not on each other's internals.
7. `M90` implements ports; domain modules do not import database/cloud/runtime-specific state as authority.
8. Cross-module integration tests live at `apps/ustc-agentd` or another explicitly declared composition test surface.
9. Cyclic large-module dependencies are forbidden.

## 3. Composition surfaces

| Surface | Purpose | Allowed knowledge |
|---|---|---|
| `apps/ustc-agentd` | production composition root, Dioxus SSR/assets/server-function and public HTTP/stream host | public contracts of all attached backend modules plus M10 transport adapters |
| `apps/ustc-agentctl` | operator/development commands and deterministic smoke | public application/domain commands only |
| `apps/ustc-client` (future) | shared Dioxus Fullstack source and Web/Android target builds | client DTO/error/event/compatibility contracts and target ports only; server-only dispatch supplied through M10 |
| module standalone tests | independent acceptance against fakes | owning module internals plus fake public counterparts |
| `apps/ustc-agentd/tests` | cross-module ordering and wiring proof | public contracts, never private fields |

A composition root may depend on several modules. That permission does not let it move their rules into one large service file.

## 4. Module completion states

```text
Planned
→ ContractReady
→ StandaloneReady
→ IntegrationReady
→ Integrated
→ Accepted
```

- `Planned`: purpose and boundaries exist, but implementation contract is incomplete.
- `ContractReady`: module blueprint, public contract, fakes and acceptance rows are reviewable.
- `StandaloneReady`: internal small modules pass standalone tests against fake counterparts.
- `IntegrationReady`: versioned public adapter and integration fixtures are complete.
- `Integrated`: composition root attaches the module without internal reach-through.
- `Accepted`: bound acceptance cases and applicable real smoke pass.

No state is inferred from commit count. `planned`, partial, skipped and not-run are not accepted.

## 5. Independent assembly rule

A module is developed as follows:

```text
module plan and public boundary
→ fake inbound/outbound counterparts
→ small internal module A + tests + commit
→ small internal module B + tests + commit
→ remaining bounded batches
→ standalone module exit gate
→ composition adapter
→ cross-module integration test
→ real feature smoke when available
→ module-level push / PR / merge
```

Unrelated large modules do not have to finish together. For example:

- `M80` may implement a complete client state reducer against a fake `M10` server.
- `M30` may complete a finite harness against fake `M50` and `M40` ports.
- `M60` may replay approved source fixtures without `M70`/`M71`/`M72` being complete.
- `M20` may complete install/disable/revoke semantics without a Dioxus Market page.

The final product combines accepted modules through composition contracts. It does not merge their private state or implementation trees.

## 6. Existing-code classification

Current code is retained as executable design evidence:

- `crates/agent-runtime` is partial `M30` evidence for a node-local run kernel.
- `crates/platform-core/src/invocation.rs` is partial `M20` evidence for pure invocation resolution.
- `crates/agent-tool-protocol` and fake gateway tests are partial `M40` evidence.
- `crates/course-planning` is partial `M72` evidence.
- `scripts/check_repo_contracts.py` and CI are partial `M90` governance evidence.

These implementations do not freeze unfinished module APIs. Before further implementation, each must be checked against its module blueprint and either adopted, amended or explicitly retained as a bounded spike.

## 7. MVP and later boundary

The initial product needs one honest attached path through selected parts of `M10`, `M20`, `M30`, `M40`, `M50`, `M60`, one or more first-party product modules and `M80`, supported by `M90`. Web proves the path first; the required target set is not accepted until the Docker Compose Fullstack server and Android peer pass their own gates.

The long-term skeleton reserves all registered modules, but later scope does not become MVP work merely because a module plan mentions it. Each module plan separately marks:

- MVP responsibility;
- later responsibility;
- explicit non-goals.

Arbitrary hosted third-party code, generic federation, a universal workflow engine and a second state authority are not implied by this module map.

## 8. Global assembly acceptance

The skeleton is preserved when:

- every current module maps to one blueprint and one owner;
- public boundaries have no cyclic dependency;
- each module can run against fakes before integration;
- composition code contains mapping and ordering, not copied domain rules;
- client replacement does not change backend domain contracts;
- Agent replacement does not change Plugin package/executor contracts while the tool protocol remains compatible;
- Plugin/product replacement does not change Agent internals;
- infrastructure replacement does not change legal domain transitions;
- the coverage matrix and acceptance registry expose planned versus implemented truth.
