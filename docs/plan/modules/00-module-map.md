# Large-module map and assembly contract

## Metadata

- `Layer`: Large-module architecture
- `Status`: Accepted current skeleton
- `Version`: `module-map/v3.1`
- `Last Review`: `2026-08-12`
- `Owning Constitution`: [`../00-engineering-constitution.md`](../00-engineering-constitution.md)
- `Counterpart Contract`: [`../../contracts/module-boundaries.md`](../../contracts/module-boundaries.md)
- `Counterpart Tasks`: [`../../tasks/00-module-work-policy.md`](../../tasks/00-module-work-policy.md), [`../../tasks/01-execution-roadmap.md`](../../tasks/01-execution-roadmap.md)

A large module is an independently owned and independently testable part. “Large” means responsibility and lifecycle independence, not a minimum code size.

## 1. Module registry

| ID | Large module | State key | Owns | Must not own | Current state |
|---|---|---|---|---|---|
| `M00` | Platform Control and Identity | `partial-evidence` | tenant/user/session identity, request causation, application command envelope, platform-wide policy references | package internals, Agent loop, source parsing, UI | identity-types, session-domain, bounded [`platform-request-context/v0`](../../contracts/platform-request-context.md) / `AUTH-013`, B4a [`platform-session-port/v0`](../../contracts/platform-session-port.md) / `AUTH-021`, B4b [`platform-control-evidence/v0`](../../contracts/platform-control-evidence.md) / `AUTH-022`, and fixed Affairs/ChangeRadar B5 compositions / `AUTH-023`–`AUTH-024` are implemented; production evidence service, generic administration and formal SSO remain planned |
| `M10` | Application Ingress Host | `partial-evidence` | Dioxus/Axum server-function ingress, versioned public HTTP/typed streams where needed, auth/session/client-compatibility admission, mapping to application ports | domain decisions, direct database/executor rules, UI | framework-neutral query carriers, fixed typed Affairs/ChangeRadar administrator commands, loopback HTTP/Web and operator CLI projections exist; ChangeRadar publication reaches the direct owning M70 port after M00 durable evidence while ordinary reads still route through current Market/Harness/ToolGateway with no direct fallback; public remote HTTP/Dioxus hosting and the broad operation registry remain planned |
| `M20` | Market and Package Lifecycle | `partial-evidence` | catalog package/component identity, install/configure/enable/disable/update/revoke/grant lifecycle, invocation authority snapshots | model loop, Plugin execution, client state | typed package/catalog + capability registry + bounded installation/grant/update domains + bounded transaction-current authority assembly + pure resolver evidence; durable lifecycle, artifact switching, API/UI and B7 composition planned |
| `M30` | Agent Harness and Runtime | `partial-evidence` | finite user-task/run phases, graph, budgets, context projection, provider/tool ports, replay and review | package lifecycle, executor implementation, transport/UI | node runtime kernel plus provider-free deterministic Harness turns/replay implemented; finite user-task harness/graph remains planned |
| `M40` | Tool Gateway and Execution | `partial-evidence` | Agent tool protocol mapping, call correlation, current authorization ordering, intent/executor/receipt/result sequence | grants, Agent phases, package declarations | protocol/fake conformance plus bounded fixed first-party Affairs and ChangeRadar adapter compositions implemented; durable generic gateway and out-of-process executor host planned |
| `M50` | Model Provider Integration | `planned` | typed provider profiles, request/stream normalization, token estimation, timeout/cancel/error mapping | run authority, tool grants, prompt truth | planned |
| `M51` | MCP Binding and Executor | `planned` | reviewed MCP endpoint/component binding, discovery/schema snapshot, protocol session, bounded tool execution | Market publication, grant decisions, Agent loop | planned |
| `M60` | Campus Trust and Source Pipeline | `planned` | source registry, retrieval policy, immutable revision, normalization, provenance, accepted baseline and publication gate | product-specific rendering, arbitrary crawling, UI | accepted contract only; `source-import/v1` and `source-retrieval/v0` accepted under R11 per `ACCEPT_EXACT_M60_B2_R11_PACKET`; bounded B1 registry plus canonical-URL-bound, deterministic-ID `DemoReviewed` revision values and an M60-owned health type; no retrieval/parser/durable baseline/publication composition; planner fixtures provide limited evidence; superseded V10 `DEC-M60-B2-ACCEPTANCE` is historical |
| `M70` | USTC ChangeRadar | `partial-evidence` | semantic change candidates/events, board scope, approval and feed behavior | generic source authority, other product state | exact-source-pinned semantic diff + digest-bound review + coherent M60 verification + fixed `M10 → M00 durable evidence → owning M70` administrator publication now persist in one strict canonical repository; exact retry, checked zero-M60 recovery, post-rename reconciliation and real JSON/Atom process/browser restart are proved, while ordinary reads retain current Market/Harness/ToolGateway and Affairs/Opportunity isolation; no approved live source, production SSO/admin, maintainer lease, M80 peer-client composition or complete product journey |
| `M71` | USTC Affairs Navigator | `partial-evidence` | reviewed tree/procedure artifacts, lookup, supersession and publication journey | generic source authority, full-corpus RAG as truth | checked query/publication kernel now serves the same bounded repository through fixture-backed `M10 → deterministic Harness → current Market authorization → ToolGateway → fixed Affairs adapter → M71`; disabled installation or revoked grant state denies before intent/executor/M60 with no direct fallback; no M00-authorized operator publication, durable authority/runtime/publication restart, package-portable executor, real source retrieval, structured search or supersession |
| `M72` | Campus Opportunity Graph | `partial-evidence` | opportunity facts, qualification/dependency/conflict, tenant profile projection and planning journeys | public source authority, cross-user profile state | offline Course Planning plus bounded exact-consent, tenant-private profile, qualification/planning, source/profile staleness and revoke/delete foundation; no platform composition or durable private state |
| `M80` | Client Core and Interaction Shells | `partial-evidence` | framework-neutral typed client behavior; peer Dioxus Web/Android, `ustc-agent` user/automation CLI and inbound MCP adapters; later admitted Windows peer, with iOS/other desktop candidates later | domain calculation/mutation, direct repository/executor access, Agent/Market/Plugin/source authority, operator `ustc-agentctl`, outbound M51 execution, peer-shell subprocess dependencies | bounded client-core and real `ustc-agent affairs get/lookup` loopback JSON/exit path exist; generic fake/real conformance, HTTP/TLS, streaming/reconnect/cancellation matrix, inbound MCP and Dioxus Web/Android remain planned |
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
M80 peer interaction shells
  ├── Dioxus Web/Android presentation
  ├── ustc-agent user/automation CLI
  └── inbound MCP tools/resources
          │ shared typed client core
          └── versioned server-function/HTTP/stream call ──► M10 Application Ingress Host

M10 Application Ingress Host
  └── typed result/error/event ──► M80 client core and outer-adapter projection

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

1. `M80` client-core and target code never import domain crates or storage/executor/provider clients. Dioxus, `ustc-agent` and inbound MCP are peers and never spawn or parse one another as their production path; `ustc-agentctl` remains outside M80.
2. `M10` maps Dioxus server-function/public HTTP/stream transport to application commands; it does not reimplement module decisions or force a loopback HTTP hop.
3. Inbound MCP (`external Agent → M80 → M10`) is directionally distinct from M51 outbound MCP execution (`M40 → M51 → external MCP server`); neither imports the other's session or credential state.
4. `M30` and `M40` depend on the Plugin-neutral tool protocol, not on each other's implementation; `ustc-agentd` orders their public operations.
5. `M40` coordinates existing decisions but does not mint grants, run phases or receipts by itself.
6. `M50` and `M51` normalize external protocols and never own platform state transitions.
7. `M70`, `M71` and `M72` may depend on `M60` contracts but not on each other's internals.
8. `M90` implements ports; domain modules do not import database/cloud/runtime-specific state as authority.
9. Cross-module integration tests live at `apps/ustc-agentd` or another explicitly declared composition test surface.
10. Cyclic large-module dependencies are forbidden.

## 3. Composition surfaces

| Surface | Purpose | Allowed knowledge |
|---|---|---|
| `apps/ustc-agentd` | production composition root, Dioxus SSR/assets/server-function and public HTTP/stream host | public contracts of all attached backend modules plus M10 transport adapters |
| `apps/ustc-agentctl` | operator/development commands and deterministic smoke | separately admitted public application/domain/operator commands; not an M80 peer client |
| `crates/client-protocol` (future) | M10-owned framework-neutral versioned wire DTO/error/event and compatibility carrier | no transport, presentation, command parser, MCP SDK or backend implementation; M80 produces request instances but does not redefine schema |
| `crates/client-core` (future) | M80-owned framework-neutral user-client behavior and fake-M10 conformance | M10-owned client-protocol values plus M80 auth/transport/reconnect abstractions; no outer-framework or backend implementation |
| `apps/ustc-agent` (future) | ordinary-user and noninteractive automation CLI | client-core, CLI parser/rendering and user auth-profile adapter only |
| inbound MCP adapter (future) | selected external-Agent tools/resources | client-core plus MCP outer protocol mapping; no M51/domain/operator reach-through |
| `apps/ustc-client` (future) | shared Dioxus Web/Android Fullstack source and later admitted Windows presentation target | client-core, presentation state, Dioxus target adapters and M10 server-function declarations only; Windows packaging/session/update remains a separate gate |
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

- `M80` may implement a complete framework-neutral client core plus CLI/MCP/Dioxus conformance against a fake `M10` server before any backend product module is complete.
- `M30` may complete a finite harness against fake `M50` and `M40` ports.
- `M60` may replay approved source fixtures without `M70`/`M71`/`M72` being complete.
- `M20` may complete install/disable/revoke semantics without a Dioxus Market page.

The final product combines accepted modules through composition contracts. It does not merge their private state or implementation trees.

## 6. Existing-code classification

Current code is retained as executable design evidence:

- `crates/agent-runtime` is partial `M30` evidence for a node-local run kernel.
- `crates/platform-core/src/invocation.rs` is partial `M20` evidence for pure invocation resolution.
- `crates/platform-core/src/market/update.rs` and `crates/platform-core/tests/market_package_update.rs` are partial `M20` evidence for bounded update/rollback decisions and the semantic in-memory package-update repository; they are not durable production lifecycle, artifact-switch or B7 composition evidence.
- `crates/agent-tool-protocol` and fake gateway tests are partial `M40` evidence.
- `crates/course-planning` plus `crates/opportunity-graph` are partial `M72`
  evidence for deterministic planning and bounded consent/private-profile
  semantics; they are not installed-Plugin, durable deletion or platform
  composition evidence.
- `crates/change-radar` plus the fixed `apps/ustc-agentd` administrator/query adapters, durable repository and Web/CLI/HTTP projections are partial `M70` evidence for bounded board policy, typed semantic diff, M00-admitted durable publication, checked recovery, current Market/Harness/ToolGateway reads and deterministic JSON/Atom restart; they are not approved live-source retrieval, production administration, maintainer-lease or M80 peer-client evidence.
- `crates/platform-core/src/source_revision.rs` is bounded `M60-B5` value evidence for honestly labelled immutable `DemoReviewed` revisions; it is not retrieval, parser, durable accepted-baseline or publication authority.
- `scripts/check_repo_contracts.py` and CI are partial `M90` governance evidence.

These implementations do not freeze unfinished module APIs. Before further implementation, each must be checked against its module blueprint and either adopted, amended or explicitly retained as a bounded spike.

## 7. MVP and later boundary

The initial product needs one honest attached path through selected parts of `M10`, `M20`, `M30`, `M40`, `M50`, `M60`, one or more first-party product modules and `M80`, supported by `M90`. M80 may prove its framework-neutral core early through bounded read-only `ustc-agent` and inbound-MCP slices against the same admitted API semantics, while Web remains the first graphical proof. The required target set is not accepted until the Docker Compose Fullstack server and Android peer also pass their own gates.

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
