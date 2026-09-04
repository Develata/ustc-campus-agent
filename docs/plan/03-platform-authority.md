# Platform authority and system boundary

## Metadata

- `Layer`: Authority architecture
- `Status`: Accepted architecture; bounded Agent/client/Web/Android evidence implemented, production authority plane largely planned
- `Version`: `0.8.2`
- `Last Review`: `2026-09-04`
- `Authority Owns`: authority partition, canonical state ownership, client/execution-plane boundary
- `Authority Defers To`: product positioning for scope and contracts for exact external shapes
- `Counterpart Features`: `docs/features/00-market-browse-install.md`, `docs/features/04-bounded-agent-harness.md`, `docs/features/05-headless-client-and-agent-integration.md`
- `Counterpart Contracts`: `docs/contracts/platform-identity.md`, `docs/contracts/agent-harness.md`, `docs/contracts/agent-plugin-boundary.md`, `docs/contracts/client-shell.md`, `docs/contracts/invocation-resolution.md`, `docs/contracts/agent-runtime.md`, `docs/contracts/interfaces.md`, `docs/contracts/permissions.md`
- `Counterpart Acceptance`: `AUTH-*`, `HARNESS-*`, `AGENT-*`, `MARKET-*`, `CLIENT-*`, `WEB-*`, `DEP-*`, `RUNTIME-*`, `PUBLIC-*`
- `Primary Code Areas`: `crates/platform-core/`, `crates/agent-runtime/`, `crates/client-protocol/`, `crates/client-core/`, `apps/ustc-agentd/`, `apps/ustc-agentctl/`, `apps/ustc-agent/`, `apps/ustc-android-demo/`, future Dioxus client and inbound MCP adapter
- `Large-module Map`: [`modules/00-module-map.md`](modules/00-module-map.md); this chapter owns cross-module authority, while module blueprints own implementation detail

## 1. Scope

This chapter defines where authoritative decisions live across the whole system. [`modules/00-module-map.md`](modules/00-module-map.md) divides that system into independently deliverable large modules. M10 owns the framework-neutral versioned client-protocol carrier; M80 owns one framework-neutral typed client core over it with peer Dioxus Web/Android, `ustc-agent` user/automation and inbound MCP adapters. M10 never depends on M80 core and no peer shells out to another. Dioxus remains the long-lived first-party graphical stack for required Web/PWA, Docker Compose server and Android targets. `ustc-agentctl` remains a separate operator surface. No client dependency or retained scaffold lands before its exact active planned acceptance rows and future bindings exist. Production database and external Agent framework choices remain unfrozen until a bounded slice proves the need.

Long-term shape:

```text
M80 peer clients
├── Dioxus Web/PWA first, then mandatory Android; later admitted Windows, then iOS/other desktop candidates
├── ustc-agent user/automation CLI
└── inbound MCP selected tools/resources
    │ M80 framework-neutral typed client core
    │ M10-owned versioned client-protocol values
    │ versioned server functions / HTTP / typed event stream
    ▼
USTC Campus Agent authority plane
├── identity/session
├── Market catalog projection
├── installation/grant resolver
├── finite HarnessRun / TaskGraph
├── Plugin-neutral AgentRun state
├── versioned Agent tool protocol
├── tool/capability gateway + Plugin executor routing
├── Campus Trust Kernel
├── first-party product use cases
└── audit/evidence
    │
    ├── reviewed Git declarations
    ├── durable operational store
    ├── immutable evidence store
    └── replaceable execution/provider adapters
```

Clients render/serialize state and submit typed intent through [`client-shell/v2.1`](../contracts/client-shell.md). Execution planes perform bounded work. Neither owns product authority. M10 owns the versioned operation/client wire schema; M80 owns client behavior over it, and M10 never depends on M80 core. Dioxus, CLI and MCP framework types terminate in their outer adapters. Dioxus/Axum transport types terminate in M10 ingress adapters. No peer client shells out to another, and `ustc-agentctl` operator authority remains separate.

## 2. Authority partition

| State | Owning authority | Rebuildable projection / adapter |
|---|---|---|
| product and engineering contracts | reviewed repository docs | overview, task and guide summaries |
| package/publisher/capability declarations | reviewed Git files under `market/` | future query database / Market UI |
| domain invariants and legal transitions | Rust domain types and validators | CLI, HTTP and UI projections |
| user installation, enablement and grants | future durable operational state under Rust transitions | caches and UI state |
| source definition and accepted revision | reviewed declaration + Rust-governed durable state | search/crawl status projections |
| immutable raw/normalized evidence | content-addressed evidence objects with exact revision identity | parser/search indexes |
| procedure/change/graph publication | reviewed canonical artifacts and durable receipts | PostgreSQL/search/feed projections |
| tenant-private profile facts | tenant-scoped durable state | consented derived match/planning view |
| HarnessRun phase, accepted TaskGraph and review/evidence state | Rust commands/events plus future durable journal | client plan panel, model prompts, framework task state |
| canonical conversation/transcript and context artifacts | tenant-scoped durable session/artifact state | bounded PromptProjection and lossy context summary |
| per-turn Agent tool definitions/calls/results | versioned platform tool protocol derived from one resolver snapshot | provider wire shape and client rendering |
| package/component execution route | installation/grant resolver + gateway-private route table | MCP/process/WASI/OCI adapter handle |
| framework/provider session | adapter state keyed by platform identity | none; never canonical |

A database brand is not authority by itself. Authority is the contract that decides who may transition which typed state and what evidence makes that transition valid.

## 3. Canonical flows

### 3.1 Catalog to invocation

```text
reviewed manifest
→ deterministic validation/import
→ visible catalog projection
→ exact package installation
→ explicit capability grants
→ enabled discovery
→ per-turn immutable tool projection
→ Plugin-neutral Agent tool call
→ per-invocation resolver/gateway recheck and effect intent
→ bounded Plugin executor
→ typed result + audit receipt
```

### 3.2 Source to published campus fact

```text
approved SourceDefinition
→ immutable SourceRevision
→ deterministic normalization
→ typed candidate
→ policy/citation/conflict validation
→ administrator approval
→ canonical publish + projection refresh
```

### 3.3 Finite harness run

```text
accepted user intent
→ bounded clarification gate
→ validated TaskGraph
→ dependency/resource-safe node execution
→ per-node evidence and fresh review
→ root verification and final review
→ evidenced report or explicit non-success state
```

Each user task is one finite `HarnessRun`. A reviewer rejection appends a bounded remediation graph revision; it does not restart the conversation or erase completed evidence. Prompt context is a projection of canonical run/session state and passes the context-budget gate before every model call.

### 3.4 Node AgentRun

```text
immutable run specification
→ bounded prompt projection and token preflight
→ model turn
→ Plugin-neutral tool call against a frozen view
→ gateway grant/policy/argument/route check
→ effect intent persistence
→ bounded Plugin executor through a replaceable adapter
→ receipt persistence
→ next turn or terminal state
```

Streaming is a projection of this state machine, not a second execution path. The Agent never loads a manifest or Plugin implementation; the Plugin never imports or mutates Agent phases. `ustc-agentd` is the composition root that may depend on both sides.

## 4. Deployment boundary

The target architecture is one authority plane with replaceable execution locations. Central, remote or local execution MAY be added behind the same typed resolver, Agent tool protocol and grant contract; they MUST NOT create parallel user/package/source authority. A native Plugin executor remains out-of-process from the Agent kernel unless a later ADR proves an equally isolated stable ABI and authority boundary.

A single-tenant local profile MAY run the same binaries for development, demo or private deployment. It is a deployment profile, not a second product or divergent state model.

The required Docker Compose profile runs the native Fullstack server, durable dependencies and reviewed reverse-proxy/TLS/readiness/recovery wiring. It serves Web assets/SSR and admitted server-function/stream endpoints. Android is a separately built/signed client artifact that points to the deployed HTTPS server; it is not a process inside Compose.

### 4.1 Multi-client interaction boundary

M80 starts with one framework-neutral typed client core over the M10-owned `client-protocol` carrier and three peer outer adapters. `ustc-agent` provides ordinary-user/noninteractive automation; inbound MCP exposes selected least-privilege tools/resources to external Agents; Dioxus provides target-neutral routes/components/view-model reduction with narrow `web` and `android` adapters. Web/PWA is the first graphical proof surface. Android is the mandatory next graphical target and reuses the same semantic ingress/event/recovery contract. Windows is an admitted later peer, but it is not a current required release target; promotion requires separate packaging/session/update/real-host acceptance. iOS and other desktop targets remain later candidates.

Dioxus server functions and explicit HTTP/SSE routes are valid peer M10 ingress adapters. They may call admitted application command/query ports, but cannot access concrete databases/repositories, Plugin executors, provider SDKs, Agent checkpoints, journals, filesystem/process APIs or platform secret stores directly. The user CLI and inbound MCP call M10 through M80 client-core and M10-owned protocol values; M10 never depends on client-core, and neither client may reach `ustc-agentctl`, domain internals or M51 outbound sessions. All outer registries project the M10-owned operation/schema registry; where adapters share an operation, they preserve permission/result/error/provenance/audit semantics, but their allowlists need not be identical. The first inbound-MCP profile is public-read Streamable HTTP. Unsupported capability remains explicit instead of silently changing execution location. Independently deployed Android, CLI and MCP clients carry build/protocol identity and receive typed compatibility or upgrade outcomes before unsafe dispatch.

## 5. Current executable state

Implemented now:

- Rust workspace and canonical first-party identities;
- deterministic package/manifest contract checks;
- operator CLI/daemon skeleton;
- framework-neutral Agent run-spec, transition, replay, effect-ordering and budget kernel;
- mechanically enforced absence of Market/Plugin/adapter dependencies in `agent-runtime`, with cross-boundary proof at the composition root;
- bounded offline Course Planning core and smoke command;
- framework-neutral client-protocol/client-core plus a bounded real `ustc-agent affairs get/lookup` fixture-loopback path;
- loopback HTTP/Web Agent Chat with deterministic offline provider mode, fixed bounded tool composition and redacted trace;
- deterministic Docker Compose delivery evidence and a Java WebView Android demo shell that remains a thin client of the Rust service.

Not yet implemented:

- production identity/session;
- durable installations and grants;
- durable Agent orchestration/journal and production ToolGateway; the framework-neutral tool-protocol value subset is implemented;
- finite HarnessRun/TaskGraph, clarification/review supervisor and context compaction;
- production live-source ingestion, durable M60 baseline and canonical publication pipeline; bounded demo review/publication state exists;
- production database/evidence store;
- full client peer conformance, authenticated HTTP/TLS/streaming CLI support and inbound MCP adapter;
- Dioxus Fullstack/PWA journey and production Android target; the current Web and Java WebView Android surfaces are bounded loopback demos, not those production peers;
- production Compose/server profile, later admitted Windows peer, and iOS/other desktop candidates.

These planned systems MUST NOT be described as operational merely because their contracts exist.

## 6. Failure and recovery

- Projection drift: rebuild from the owning canonical declaration/state; do not edit the projection into truth.
- Adapter/framework conflict: reject the transition and retain the last platform-owned state.
- Audit/receipt failure: do not acknowledge success for a durable mutation.
- Partial bootstrap: roll back or resume deterministically from explicit state; do not infer completion.
- Cache/search loss: degrade query performance only; durable identity, grants and accepted facts remain intact.
- Tenant-scope ambiguity: reject before read or invocation.

## 7. Verification entrypoints

- `python3 scripts/check_repo_contracts.py`
- `cargo test --locked --all-targets --all-features`
- `ustc-agentctl doctor`
- `ustc-agentctl market validate`
- `HARNESS-*`, `AGENT-*`, `MARKET-*`, `FP-*` and `RUNTIME-*` rows in `docs/acceptance/matrix.tsv`
