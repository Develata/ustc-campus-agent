# Architecture overview

## Status

This page is a navigation map. It does not create new product behavior, authority or lifecycle. Current rules live in:

- [`../plan/00-engineering-constitution.md`](../plan/00-engineering-constitution.md);
- [`../plan/modules/00-module-map.md`](../plan/modules/00-module-map.md);
- [`../contracts/module-boundaries.md`](../contracts/module-boundaries.md);
- each matching module blueprint/contract;
- [`../tasks/00-module-work-policy.md`](../tasks/00-module-work-policy.md).

## 1. Product shape

USTC Campus Agent is one platform with three independently identified first-party Plugins:

```text
ustc.change-radar       → M70 ChangeRadar
ustc.affairs-navigator  → M71 Affairs Navigator
ustc.opportunity-graph  → M72 Opportunity Graph
```

They share `M60` Campus Trust/Source facts. They do not share package version, installation, enablement, product state or acceptance.

## 2. Four call layers

```text
Interaction shell
  M80 Dioxus Fullstack Web/PWA → mandatory Android → later iOS/desktop
  CLI/integration callers
          │ typed intent / safe projection
          ▼
Application interface
  M10 ustc-agentd Dioxus server functions + HTTP + typed streams
          │ admitted typed command/query/event
          ▼
Flow coordination
  M00 actor/request context
  M20 Market/package lifecycle
  M30 finite Agent harness/runtime
  application composition only
          │ bounded ports/contracts
          ▼
Execution domain
  M40 ToolGateway/execution order
  M50 model provider adapters
  M51 MCP binding/executor
  M60 source/revision pipeline
  M70/M71/M72 first-party product rules
          │
          ▼
M90 infrastructure implementations
  repositories/journals/evidence/clock/queue/secrets/HTTP/telemetry/deployment
```

The thin client displays and submits intent. Backend modules perform every truth-affecting calculation and mutation. `M90` makes module rules durable; it does not define them.

## 3. Large-module map

| ID | Module | Owns | Current evidence |
|---|---|---|---|
| `M00` | Platform Control/Identity | tenant/user/session/request/policy identity and causation | identity value types only |
| `M10` | Application Ingress Host | Dioxus/Axum server functions, versioned public HTTP/streams, compatibility admission and application mapping | daemon help/version skeleton |
| `M20` | Market/Package Lifecycle | catalog, exact install/config/grant/enable/update/revoke and invocation authority | typed package/catalog + capability-registry + bounded managed-installation fake + bounded reviewed-grant aggregate/replay/semantic repository + pure resolver/fixtures |
| `M30` | Agent Harness/Runtime | finite task/run/graph/context/budget/evidence/review state | node-local runtime kernel |
| `M40` | Tool Gateway/Execution | exact tool correlation, authorization order, intent/executor/receipt/result | protocol values + fake conformance |
| `M50` | Model Provider | typed profiles, provider protocol, stream/final/usage/estimator | planned |
| `M51` | MCP Binding/Executor | reviewed binding/discovery/schema drift and MCP execution | planned |
| `M60` | Campus Trust/Source | source policy, immutable revision, provenance, conflict, freshness, baseline | synthetic fixture semantics only |
| `M70` | ChangeRadar | semantic change review/event/feed | manifest/design only |
| `M71` | Affairs Navigator | reviewed procedure tree/artifacts/search | manifest/design only |
| `M72` | Opportunity Graph | reviewed opportunities, private profiles, qualification/planning | offline Course Planning spike |
| `M80` | Dioxus Fullstack Multi-client | required Web/Android UI/routes/view state/SSR/hydration/generated client facade; later iOS/desktop | accepted design, no crate/dependency |
| `M90` | Infrastructure/Operations | ports for storage/journal/evidence/config/secrets/HTTP/telemetry and Docker Compose deployment | CI/checker only |

“Current evidence” is not module completion. See the module blueprint exit gate and acceptance matrix.

## 4. Dioxus and API boundary

Accepted topology:

```text
one shared Dioxus application
  ├── Web/PWA first
  ├── SSR/hydration/page hosting
  ├── Android mandatory next target
  ├── iOS target adapter later
  └── desktop target adapter later
          │
          │ versioned server functions / HTTP / typed streams
          ▼
M10 ingress / ustc-agentd
          │
          ▼
backend module application interfaces
```

Dioxus server functions are Axum-compatible M10 ingress adapters and SHOULD back the generated first-party `ClientApi` facade. After compatibility, identity, authorization, bounds, idempotency/precondition and audit admission, they may call one public application command/query port. They cannot call concrete repositories/databases, executors, provider SDKs or journals directly. Optional public HTTP adapters call the same application ports and do not duplicate business logic.

The Docker Compose profile runs the native server and dependencies, serves Web assets/SSR and exposes admitted HTTPS endpoints. Android is a separate signed artifact that reuses the client contract but may lag server deployments, so compatibility/upgrade behavior is explicit.

## 5. Agent/tool path

```text
M20 resolves one immutable allowed-tool projection
→ M30/provider sees Plugin-neutral AgentToolsetView
→ provider proposes AgentToolCall
→ M40 normalizes exact call/private route
→ M20 rechecks current deny-side authority
→ M30 persists EffectIntent
→ M40 calls admitted M51/peer executor
→ output is bounded and validated
→ M30 persists EffectReceipt
→ M40 returns correlated AgentToolResult
→ M30 decides the next legal run transition
```

Ownership remains split:

- `M20`: package/install/grant/current invocation authority;
- `M30`: run phases, budgets, effect state and completion;
- `M40`: mapping/order/output boundary;
- `M51` or peer: concrete execution protocol;
- model/Plugin/client: no authority over the above.

## 6. Campus source/product path

```text
reviewed SourceDefinition
→ M60 bounded fetch + immutable raw snapshot
→ deterministic normalize/parse/revision
→ provenance/freshness/conflict
→ accepted baseline
        ├── M70 semantic change candidate → review → event/feed
        ├── M71 procedure candidate → validate/review → artifact/search
        └── M72 opportunity fact + private profile → deterministic result
```

Fetch success is not publication. Model output is candidate/explanation, not fact authority. The three products consume the same exact source/revision identity but remain independent modules/packages.

## 7. Infrastructure direction

Each domain module declares its own semantic port. `M90` implements it:

```text
Domain-owned port                    Infrastructure peer
RunJournal                 ←         durable event store
EvidenceStore              ←         verified object/filesystem store
InstallationRepository     ←         transactional database adapter
SafeHttpClient             ←         bounded fixed-origin HTTP adapter
SecretResolver             ←         secret reference backend
Clock/Scheduler/Lease       ←         runtime/scheduling adapter
```

Domain modules do not import concrete SQL rows, queue clients, provider SDKs or deployment handles as authority. Cache/search/queue loss must be recoverable from canonical state.

## 8. Independent development and assembly

Each large module follows:

```text
blueprint + boundary
→ small high-cohesion commits
→ equal-contract fakes
→ standalone success/failure/recovery gates
→ composition adapter
→ cross-module integration fixture
→ real feature smoke when applicable
→ authorized push / PR / merge
```

A module can be developed against fakes while peers remain unfinished. `ustc-agentd` maps and orders public calls; it does not copy private rules. See [`../tasks/01-execution-roadmap.md`](../tasks/01-execution-roadmap.md) for batch IDs and assembly gates.

## 9. Current implementation truth

Implemented evidence:

- `crates/agent-runtime`: node-local AgentRun state/event/replay/budget/effect kernel;
- `crates/platform-core/src/invocation.rs`: pure package/install/grant/tool projection and recheck;
- `crates/platform-core/src/market/capability.rs`: typed immutable capability-registry loading, derived policy and permission-change classification without grant issuance;
- `crates/platform-core/src/market/installation.rs`: pure managed-installation aggregate/replay and semantic in-memory repository without durable persistence or production enable-evidence issuance;
- `crates/platform-core/src/market/grant.rs`: pure reviewed-grant aggregate/decide/evolve/replay and semantic in-memory repository without durable persistence or production grant issuance;
- `crates/agent-tool-protocol`: Plugin-neutral tool values;
- `apps/ustc-agentd/tests`: fake resolver/gateway/executor composition proof;
- `crates/course-planning` + CLI: deterministic offline Course Planning spike;
- repository/CI contract checks.

Not implemented:

- Dioxus Fullstack app or dependency, Compose Fullstack server profile, Web journey or Android artifact;
- Fullstack/public ingress, typed stream or auth/session service;
- production durable Market installations/grants and lifecycle composition;
- finite HarnessRun/TaskGraph/context/review supervisor;
- real model provider/MCP/Plugin executor;
- real source pipeline and first-party product integrations;
- production database/evidence/secret/deployment profile.

## 10. Reading order

1. [`../plan/00-engineering-constitution.md`](../plan/00-engineering-constitution.md)
2. [`../plan/01-terminology.md`](../plan/01-terminology.md)
3. [`../plan/modules/00-module-map.md`](../plan/modules/00-module-map.md)
4. the assigned file under [`../plan/modules/`](../plan/modules/)
5. [`../contracts/module-boundaries.md`](../contracts/module-boundaries.md) and matching specific contracts
6. matching feature and acceptance rows
7. [`../tasks/00-module-work-policy.md`](../tasks/00-module-work-policy.md)
8. assigned batches in [`../tasks/01-execution-roadmap.md`](../tasks/01-execution-roadmap.md)
