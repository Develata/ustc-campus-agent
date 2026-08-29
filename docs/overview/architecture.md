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
Interaction shell — M80 peer adapters
  Dioxus Web/PWA + Android; later admitted Windows; iOS/other desktop candidates later
  ustc-agent user/automation CLI
  inbound MCP selected tools/resources
          │ M80 framework-neutral typed client core
          │ M10-owned versioned client-protocol values
          │ typed intent / safe projection
          ▼
Application interface
  M10 ustc-agentd Dioxus server functions + explicit HTTP + typed streams
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

The thin peer clients display/serialize server projections and submit intent through one typed client core. They do not spawn one another, and `ustc-agentctl` remains a separate operator surface. Backend modules perform every truth-affecting calculation and mutation. `M90` makes module rules durable; it does not define them. Inbound MCP (`external Agent → M80 → M10`) is distinct from M51 outbound MCP execution.

## 3. Large-module map

| ID | Module | Owns | Current evidence |
|---|---|---|---|
| `M00` | Platform Control/Identity | tenant/user/session/request/policy identity and causation | identity/session/request-context kernels plus bounded B4a session-port interfaces, deterministic fakes and one secure durable DemoReviewed current-session read/bootstrap vendor; B4b/B5/SSO remain planned |
| `M10` | Application Ingress Host | Dioxus/Axum server functions, versioned public HTTP/streams, compatibility admission and application mapping | framework-neutral client-protocol/application-ingress carriers plus one loopback three-plugin JSON/Web composition; public HTTP/Dioxus/stream host planned |
| `M20` | Market/Package Lifecycle | catalog, exact install/config/grant/enable/update/revoke and invocation authority | typed catalog/capability + bounded installation/grant/update domains + bounded transaction-current authority assembly + pure resolver/fixtures |
| `M30` | Agent Harness/Runtime | finite task/run/graph/context/budget/evidence/review state | node-local runtime kernel |
| `M40` | Tool Gateway/Execution | exact tool correlation, authorization order, intent/executor/receipt/result | protocol values + fake conformance |
| `M50` | Model Provider | typed profiles, provider protocol, stream/final/usage/estimator | planned |
| `M51` | MCP Binding/Executor | reviewed binding/discovery/schema drift and MCP execution | planned |
| `M60` | Campus Trust/Source | source policy, immutable revision, provenance, conflict, freshness, baseline | bounded registry plus canonical-URL-bound, deterministic-ID `DemoReviewed` revision values and an M60-owned revision-health type; retrieval/parser/durable baseline remains planned |
| `M70` | ChangeRadar | semantic change review/event/feed | exact-source-pinned board policy, typed semantic diff, administrator approve/reject receipts, coherent M60 verification, exactly-once in-memory publication and deterministic Atom now have one fixture-backed current-Market/Harness/ToolGateway/M10/loopback-Web path; M00-authorized administration and durable restart remain planned |
| `M71` | Affairs Navigator | reviewed procedure tree/artifacts/search | hardened procedure/artifact/policy/freshness/conflict kernel plus fixture-backed exact stable-ID composition; real source/admin/Web lifecycle planned |
| `M72` | Opportunity Graph | reviewed opportunities, private profiles, qualification/planning | offline Course Planning spike |
| `M80` | Client Core and Interaction Shells | framework-neutral client behavior and peer Dioxus Web/Android, `ustc-agent` and public-read-first inbound MCP adapters; later admitted Windows | bounded client-core and real ordinary-user fixture-loopback Affairs CLI evidence; production auth/HTTP/stream, inbound MCP and Dioxus peers planned |
| `M90` | Infrastructure/Operations | ports for storage/journal/evidence/config/secrets/HTTP/telemetry and Docker Compose deployment | CI/checker only |

“Current evidence” is not module completion. See the module blueprint exit gate and acceptance matrix.

## 4. Typed client peers and API boundary

Accepted topology:

```text
M80 framework-neutral client core
  ├── one shared Dioxus application
  │     ├── Web/PWA first graphical proof
  │     ├── SSR/hydration/page hosting
  │     ├── Android mandatory graphical peer
  │     └── later admitted Windows; iOS/other desktop candidates later
  ├── ustc-agent ordinary-user/automation CLI
  └── inbound MCP selected tools/resources
          │
          │ versioned server functions / explicit HTTP / typed streams
          │ M10-owned client-protocol values
          ▼
M10 ingress / ustc-agentd
          │ admitted application command/query/event
          ▼
backend module application interfaces
```

Dioxus, `ustc-agent` and inbound MCP are outer peers over one M80 client semantic core. M10 owns the framework-neutral versioned operation and wire schema registry; M80 core consumes it, and M10 never depends on client-core. Adapter registries are allowlisted projections, so shared operations preserve permission/result/error/provenance/audit semantics without requiring identical command sets. The peers do not invoke or parse one another as subprocesses. Dioxus server functions and explicit HTTP/SSE routes are M10 peer ingress adapters. After compatibility, identity, authorization, bounds, idempotency/precondition and audit admission, each may call one public application command/query port. No client or ingress adapter calls concrete repositories/databases, executors, provider SDKs or journals directly.

The Docker Compose profile runs the native server and dependencies, serves Web assets/SSR and exposes admitted HTTPS endpoints. Android, `ustc-agent` and the inbound MCP adapter are independently deployable and may lag server deployments, so compatibility/upgrade behavior is explicit. `ustc-agentctl` remains operator/developer-only. M51 remains the opposite platform-to-external-MCP execution path.

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
- `crates/platform-core/src/market/authority.rs`: one semantic carrier-by-carrier authority read transaction, service-owned resolver/recheck assembly and post-success precondition evidence without durable adapters, effect intents or I/O;
- `crates/platform-core/src/market/update.rs`: bounded pure update/rollback aggregate and atomic in-memory semantic package-update repository with exact approval/readiness/confirmation/rollback evidence, complete-current grant staling and receipt-prefix rebuild, without durable persistence, crash recovery, artifact switching, API/UI or B7 composition;
- `crates/agent-tool-protocol`: Plugin-neutral tool values;
- `crates/client-protocol` + `crates/application-ingress`: bounded `affairs.get` and `change.list` M10 wire/admission/projection carriers;
- `apps/ustc-agentd`: fixture-backed Affairs and ChangeRadar current-Market/Harness/ToolGateway/owning-Plugin compositions plus loopback Web JSON/Atom presentation;
- `crates/change-radar`: deterministic semantic diff, review/publication/query and Atom kernel;
- `crates/affairs-navigator`: reviewed procedure publication/query kernel;
- `crates/client-core` + `apps/ustc-agent`: bounded Affairs loopback CLI semantics;
- `crates/course-planning` + CLI: deterministic offline Course Planning spike;
- repository/CI contract checks.

Not implemented:

- inbound MCP adapter;
- Dioxus Fullstack app or dependency, Compose Fullstack server profile, peer Web journey or Android artifact;
- Fullstack/public ingress, typed stream or auth/session service;
- production durable Market installations/grants/updates/authority adapters, crash recovery, artifact switching and lifecycle/effect-intent/B7 composition;
- finite HarnessRun/TaskGraph/context/review supervisor;
- real model provider/MCP executor and package-portable/out-of-process Plugin executor host;
- real retrieval/parser source pipeline and durable first-party product integrations;
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
