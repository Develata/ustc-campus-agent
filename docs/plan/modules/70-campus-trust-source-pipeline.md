# M60 — Campus Trust and Source Pipeline

## Metadata

- `Module ID`: `M60`
- `Status`: Accepted blueprint; `source-import/v1` and `source-retrieval/v0` accepted as contract authority under R11 per `ACCEPT_EXACT_M60_B2_R11_PACKET` (2026-08-13); `M60` overall remains `planned`; bounded `M60-B1 source-registry` and the first offline-only M60-B2 pure policy are implemented; transport/network effects and B3+ admission remain separately gated; the superseded V10 `DEC-M60-B2-ACCEPTANCE` is historical evidence only
- `Implementation State`: `planned`
- `Version`: `m60-campus-trust/v0.4`
- `Last Review`: `2026-09-01`
- `Owning Plan`: [`../05-campus-trust-kernel.md`](../05-campus-trust-kernel.md)
- `Current Contract`: accepted [`source-import/v1`](../../contracts/source-import.md) and [`source-retrieval/v0`](../../contracts/source-retrieval.md) under R11 (`ACCEPT_EXACT_M60_B2_R11_PACKET`); bounded B1 implements the v1 lifecycle while historical [`source-import/v0`](../../contracts/source-import.md#15-source-importv0--historical-evidence-retained) remains immutable predecessor evidence
- `Primary code area`: `crates/platform-core/src/source_registry.rs` and `crates/platform-core/tests/source_registry.rs` for bounded pure `M60-B1` v1 lifecycle; `crates/platform-core/src/source_retrieval.rs` and `crates/platform-core/tests/source_retrieval.rs` for bounded offline M60-B2 pure policy; `crates/platform-core/src/source_revision.rs` for bounded immutable-revision/provenance/health values; transport, parser/normalizer adapters, durable accepted-baseline advancement and publication composition remain future; current fixture semantics in `crates/course-planning/`

## 1. Purpose

`M60` is the single shared path by which campus information becomes an approved, versioned and traceable fact. It owns source identity, retrieval policy, immutable revisions, normalization, provenance, conflict/freshness state, accepted baselines and publication gates.

`M70`, `M71` and `M72` consume its typed outputs. They do not create competing source/revision systems.

## 2. Non-goals

- generic crawling or arbitrary URL retrieval;
- letting models publish canonical facts;
- owning product-specific procedure/change/opportunity presentation;
- storing raw USTC user credentials or browser sessions;
- wildcard approval of all `*.ustc.edu.cn` sources;
- using a search index/RAG answer as source truth.

## 3. Owned objects and state

```text
SourceDefinition / SourcePolicy
SourceStatus: Proposed | Approved | Suspended | Revoked
  (implemented as the bounded pure M60-B1 source-import/v1 lifecycle;
   M60-B2 retains only bounded offline pure policy; transport/effects remain gated)
SourceAuthorityRevision (initial propose creates revision 1 without expected CAS;
  exact CAS + checked increment on every post-proposal mutation)
RetrievalSubject / RetrievalPlanCandidate / RetrievalPolicy
RetrievalLease / Observation
RawSnapshot / NormalizedSnapshot
SourceRevision: Observed | Parsed | Accepted | Archived | Rejected
ParserIdentity / NormalizerIdentity
Provenance / authority / observed/published/effective time
Conflict/Freshness state
AcceptedBaseline
PublicationCandidate / PublicationReceipt
```

Immutable evidence owns exact bytes/digests. Durable source state owns lifecycle and baseline. Search/query projections are rebuildable.

## 4. Public inputs and outputs

Administrative commands:

```text
Propose/Approve/Suspend/Revoke SourceDefinition
Admit parser/normalizer identity
Review/Accept/Reject SourceRevision or publication candidate
```

Pipeline operations:

```text
Acquire retrieval lease
Fetch approved source
Persist immutable raw snapshot
Normalize and parse
Compare digest/baseline
Produce typed fact/change candidate
Persist evidence
Advance accepted baseline only after complete success
```

Outputs expose stable source/revision IDs, authority, observation/publication/effective time, digest, freshness, conflict and evidence references.

## 5. Dependency direction

Allowed dependencies:

- `M00` actor/request identity for administrative mutations;
- `M90` safe fetch, clock, lease, snapshot/evidence, repository, queue and telemetry ports;
- parser implementations admitted under `M60` contracts.

Allowed callers:

- `M70`, `M71`, `M72` typed fact/revision queries;
- `M10` reviewed source/operator application services.

Forbidden dependencies:

- Dioxus/client types;
- product-specific canonical state inside the generic pipeline;
- Agent/provider/MCP frameworks as truth;
- arbitrary user URL/credential forwarding;
- concrete object-store/search/database types in domain contracts.

## 6. Lifecycle

```text
Source Proposed → reviewed → Approved
Approved + lease
→ conditional bounded fetch
→ immutable raw snapshot
→ deterministic normalize/parse
→ normalized revision
→ typed candidate + complete evidence
→ Accepted baseline advance
→ product-specific review/publication

source may become Suspended | Revoked
revision may become Archived | Rejected
```

Publication is separate from retrieval. A successful fetch is not a published fact.

## 7. Failure and recovery

- Unapproved/revoked source, unsafe redirect/IP/content: no fetch or state advance.
- Fetch timeout/unavailable: retain last accepted baseline; expose stale/health state.
- Changed content + parser failure: preserve evidence safely, do not advance baseline.
- Snapshot/evidence write failure: no acknowledgement or baseline advance.
- Same revision retry: deterministic IDs/digests prevent duplicates.
- Conflicting highest-authority facts: expose conflict or refuse; never guess.
- Lease/concurrent worker conflict: one deterministic winner; others observe/retry.
- Search/projection loss: rebuild from accepted revisions/evidence.

## 8. Configuration and secrets

Each `SourceDefinition` carries exact reviewed hosts/paths, retrieval method, owner, rate/body/time/content limits, parser ID/version and permission evidence reference. Credentials, if a future private source is approved, use separately reviewed tenant/operator `SecretRef`s and data-class policy; none are MVP public defaults.

## 9. Observability

Record source/revision/lease/parser IDs, URL policy disposition, HTTP/content metadata classes, raw/normalized digests, candidate/baseline transitions, freshness/conflict and redacted failure. Metrics bound fetch duration/bytes, parse time, no-change, changed, rejected and stale counts.

## 10. Extension and replacement

Fetch clients, snapshot stores, parsers, normalizers, schedulers, databases and search indexes are replaceable behind owned ports. Each source parser is a peer module with fixtures. New source classes require review; a model-proposed URL remains a candidate for later review, not immediate execution.

## 11. Performance path

Network/body bounds precede parsing. Raw data streams to bounded immutable storage; normalization avoids unnecessary copies where practical. Digest, parser and semantic comparison are deterministic. Concurrency/rate limits are per source/host and globally bounded.

## 12. Scope boundary

**MVP**

- one or a few explicitly reviewed public sources;
- safe bounded retrieval;
- immutable raw/normalized revisions;
- deterministic parser/normalizer fixtures;
- provenance, freshness, conflict and accepted baseline;
- failure cannot advance baseline;
- typed facts for one first-party product journey.

**Later**

- broader reviewed source registry;
- attachment/PDF pipelines after content isolation;
- operator/private sources under separate identity/data review;
- bounded retrieval over approved snapshots for Agent assistance.

**Explicit non-goals**

- arbitrary web crawler/search engine;
- wildcard campus-domain authorization;
- unreviewed model publication;
- full-corpus RAG as canonical fact path.

## 13. Small-module decomposition

1. `source-registry` — source identity, owner, six-field policy and operational lifecycle (v1 implemented as bounded pure M60-B1; v0 retained as historical P1-1 evidence).
2. `retrieval-policy` — URL/DNS/IP/redirect/content/rate limits and response algebra (v0 contract accepted under R11; bounded offline pure-policy implementation retained, with no port or effect).
3. `source-lease` — concurrency and deterministic work identity.
4. `raw-snapshot` — immutable bounded evidence.
5. `normalization` — deterministic normalized bytes/digest.
6. `parser-contract` — peer parser interface, identity and fixtures.
7. `source-revision` — revision lifecycle and provenance.
8. `conflict-freshness` — authority/time/stale/conflict decisions.
9. `baseline` — atomic accepted-baseline transition.
10. `publication-gate` — typed candidate/evidence/admin disposition.
11. `source-ports` — fetch/store/repository/queue/clock fakes.

The accepted `source-import/v1` and `source-retrieval/v0` contracts (R11 two-layer edition) define current contract authority for modules 1–2. Module 1 implements the bounded lifecycle precondition, including operational `Suspended`/`Revoked`, monotone authority revision and approved-only `RetrievalSubject`. Module 2 retains the separately admitted bounded offline pure-policy and shape-only observation algebra; it implements no M60/M90 port, clock, journal, network effect, source approval or B3 carrier.

## 14. Exit gate

`M60` is standalone-ready when a reviewed historical fixture replays deterministically and every fetch/snapshot/parse/evidence/concurrency failure leaves the accepted baseline unchanged. The bounded lifecycle prerequisite (operational `Suspended`/`Revoked` with monotone `SourceAuthorityRevision`) is implemented, but no live retrieval adapter exists. M60 is accepted only when one real approved public source feeds a first-party product candidate with complete provenance and no arbitrary URL path.
