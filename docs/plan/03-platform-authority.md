# Platform authority and system boundary

## Metadata

- `Layer`: Authority architecture
- `Status`: Accepted architecture; R0 Agent transition kernel implemented, authority plane largely planned
- `Version`: `0.2.0`
- `Last Review`: `2026-07-23`
- `Authority Owns`: authority partition, canonical state ownership, client/execution-plane boundary
- `Authority Defers To`: product positioning for scope and contracts for exact external shapes
- `Counterpart Features`: `docs/features/00-market-browse-install.md`
- `Counterpart Contracts`: `docs/contracts/agent-runtime.md`, `docs/contracts/interfaces.md`, `docs/contracts/permissions.md`
- `Counterpart Acceptance`: `AGENT-*`, `MARKET-*`, `RUNTIME-*`, `PUBLIC-*`
- `Primary Code Areas`: `crates/platform-core/`, `crates/agent-runtime/`, `apps/ustc-agentd/`, `apps/ustc-agentctl/`

## 1. Scope

This chapter defines where authoritative decisions live. It does not freeze a production database, frontend framework or external Agent framework before a real slice proves the need.

Long-term shape:

```text
Web/PWA and future native clients
    │ versioned HTTP JSON + event stream
    ▼
USTC Campus Agent authority plane
├── identity/session
├── Market catalog projection
├── installation/grant resolver
├── platform run state
├── tool/capability gateway
├── Campus Trust Kernel
├── first-party product use cases
└── audit/evidence
    │
    ├── reviewed Git declarations
    ├── durable operational store
    ├── immutable evidence store
    └── replaceable execution/provider adapters
```

Clients render state and submit typed intent. Execution planes perform bounded work. Neither owns product authority.

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
→ per-invocation resolver and gateway
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

### 3.3 Platform run

```text
immutable run specification
→ model turn
→ proposed tool call
→ grant/policy/argument check
→ effect intent persistence
→ bounded adapter execution
→ receipt persistence
→ next turn or terminal state
```

Streaming is a projection of this state machine, not a second execution path.

## 4. Deployment boundary

The target architecture is one authority plane with replaceable execution locations. Central, remote or local execution MAY be added behind the same typed resolver and grant contract; they MUST NOT create parallel user/package/source authority.

A single-tenant local profile MAY run the same binaries for development, demo or private deployment. It is a deployment profile, not a second product or divergent state model.

## 5. Current executable state

Implemented now:

- Rust workspace and canonical first-party identities;
- deterministic package/manifest contract checks;
- operator CLI/daemon skeleton;
- framework-neutral Agent run-spec, transition, replay, effect-ordering and budget kernel;
- bounded offline Course Planning core and smoke command.

Not yet implemented:

- production identity/session;
- durable installations and grants;
- durable Agent orchestration, journal and tool gateway;
- source ingestion and publication state;
- production database/evidence store;
- Web/PWA journey.

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
- `AGENT-*`, `MARKET-*`, `FP-*` and `RUNTIME-*` rows in `docs/acceptance/matrix.tsv`
