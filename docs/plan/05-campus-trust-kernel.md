# Campus Trust Kernel

## Metadata

- `Layer`: Shared campus authority
- `Status`: Contract accepted under R11 M60-B2 two-layer transport architecture; `source-import/v1` and `source-retrieval/v0` are current contract authority per `ACCEPT_EXACT_M60_B2_R11_PACKET` (2026-08-13); bounded `M60-B1 source-registry` remains implemented under `source-import/v0` (P1-1); operational `Suspended`/`Revoked` lifecycle precondition applies before any live B2 retrieval adapter; concrete source approval, retained B2 implementation and network retrieval remain unauthorized; the superseded V10 `DEC-M60-B2-ACCEPTANCE` is historical evidence only
- `Version`: `0.5.0`
- `Last Review`: `2026-08-15`
- `Authority Owns`: source identity, immutable revision, authority comparison policy, temporal/conflict/provenance state, baseline advancement and publication gates
- `Authority Defers To`: source-import/data-model contracts for exact shapes and package sourcePolicy for requested scope
- `Counterpart Features`: all documents under `docs/features/`
- `Counterpart Contracts`: `docs/contracts/source-import.md`, `docs/contracts/source-retrieval.md`, `docs/contracts/data-models.md`
- `Counterpart Acceptance`: `SRC-*`, `PROC-*`, `RADAR-*`, `COURSE-*`
- `Primary Code Areas`: future source/knowledge modules; current `crates/course-planning/`
- `Large-module Blueprint`: [`modules/70-campus-trust-source-pipeline.md`](modules/70-campus-trust-source-pipeline.md)

## 1. Purpose and non-goals

The Campus Trust Kernel is the shared semantic foundation for Affairs Navigator, ChangeRadar and Opportunity Graph. This chapter owns shared trust policy; the `M60` blueprint owns its independent implementation decomposition. It answers:

- which source is approved and who owns it;
- which immutable revision supports a fact;
- when the fact was observed, published and effective;
- how conflicting authorities are resolved or exposed;
- when a baseline or published artifact may advance;
- how user-private facts remain separate from public campus facts.

It is not a general crawler, does not authorize `*.ustc.edu.cn` by wildcard and does not let a model publish canonical facts.

## 2. SourceDefinition

A planned `SourceDefinition` contains at least:

```text
source_id
node_ids[]
title/locales
authority_class
owning_organization
operator_owner
approved URL/retrieval policy
crawl permission evidence reference
rate policy
parser ID/version
board policy ID/version
expected content types
status: Proposed | Approved | Suspended | Revoked
review revision
```

Invariants:

- `source_id` is stable across URL/path/title changes.
- `Approved` requires explicit owner, authority, URL, retrieval, parser, rate and permission review.
- wildcard domains are at most egress ceilings; every source still needs exact host/path review.
- `Suspended` blocks new retrieval while preserving historical evidence; `Revoked` is terminal and irreversible.
- `SourceAuthorityRevision` is a monotone non-zero counter incremented on every retrievability-affecting transition; every mutation requires expected-revision CAS.
- declarations contain no credential, private endpoint or user session.
- login/cookie/token sources require a separate identity and data-class review; they are not public defaults.

No concrete production SourceDefinition is currently claimed as approved.

## 3. SourceRevision

A `SourceRevision` binds:

```text
source_revision_id and source_id
canonical URL, retrieved URL and aliases/redirect evidence
published_at? / observed_at / effective_from? / effective_to?
authority class
response metadata
raw and normalized digests
immutable raw and normalized snapshot references
parser ID/version
status: Observed | Parsed | Accepted | Archived | Rejected
```

The system MUST distinguish publication time, observation time and effective interval. One URL can have many immutable revisions; URL is a lookup key, not revision identity.

### 3.1 Bitemporal provenance fields

Every material fact carries bitemporal provenance. Two of the three names are fact-level projections; the third is a query/answer-level cutoff, not a fact-level field:

- `valid_at`: real-world validity time — when the fact is true in the real world. Projected from the source revision's effective interval (`effective_from`/`effective_to`) or `published_at` when no separate effective interval is declared. A fact may have a validity interval, not only a point; `valid_at` carries that interval when the source provides one.
- `known_at`: system knowledge time — when the system first observed or retrieved the fact. Projected from the source revision's `observed_at`. This is the fact-level "when did we learn this" field.
- `as_of`: **query/answer cutoff** — the point in time used to select which known facts are eligible to answer a given query. It is NOT a fact-level field, NOT review/acceptance time, and NOT a fourth fact timestamp. Each answer carries the `as_of` cutoff under which it was produced; a fact with `known_at ≤ as_of` is eligible (subject to authority/freshness/conflict policy), and a fact with `known_at > as_of` is excluded as not-yet-known at the cutoff.

Separate from these three, the planned Affairs Navigator evidence context ([`docs/plan/06-first-party-plugins.md`](../plan/06-first-party-plugins.md) §2.6) carries review/verification metadata at the evidence/procedure level — `observed_at`, `reviewed_at`, `last_verified_at` — that record when a source revision was observed, reviewed and last re-verified. These are not fact-level projections of the bitemporal vocabulary above and must not be collapsed into `as_of`.

These names are the canonical vocabulary. The source-revision-level fields (`published_at`, `observed_at`, `effective_from`, `effective_to`) remain the raw authority; `valid_at`/`known_at` are the fact-level projection and `as_of` is the query/answer-level cutoff. A fact's `valid_at` may precede its `known_at`. Missing `valid_at` remains `None`; the system never copies `known_at` into `valid_at` merely to avoid nullability. `as_of` is selected by the query path (default: wall-clock now, or an explicit cutoff passed by the caller/application) and is never synthesized from a fact's own timestamps.

Do not introduce a universal `ReviewedFactEnvelope<T>`. Shared `EvidenceContext` is introduced only if the typed product contract needs it now.

### 3.2 Authority comparison policy

The generic `M60` source authority defines a **partial comparison**, not a total order. Comparing two authorities yields one of:

```text
Higher | Lower | Equivalent | Incomparable
```

Norms:

1. Generic `M60` source authority carries no product-specific variants such as `icourse_mirror` or `official_catalog_snapshot`. Those belong to product modules behind their own policy/type.
2. `Incomparable` material facts create conflict or `cannot_verify`; the system never selects by a numeric total order or by arbitrary variant precedence.
3. `ModelInference` is rejected as a source authority at the registry admission boundary (see §2 and the `M60` blueprint); it never enters the comparison algebra.
4. A product module MAY define its own local total order over a product-specific authority type when its domain genuinely has one (for example, Course Planning's `official_catalog_snapshot > reviewed_official_source > icourse_mirror > community_signal` ordering). That local order is a product policy projection, not the generic `M60` authority contract.
5. The generic comparison and a product-local total order do not contradict: the product-local order is a refinement that holds inside the product's bounded type; the generic `M60` authority remains incomparable across product-specific variants it does not name.

This section owns the policy. The exact generic comparison type and its laws live in `docs/contracts/source-import.md` and the `M60` blueprint; the Course Planning local order lives in [`docs/contracts/data-models.md`](../contracts/data-models.md).

## 4. Retrieval and baseline state machine

```text
approved source + lease
→ conditional fetch
→ immutable raw snapshot
→ deterministic normalization
→ digest comparison
→ typed parser
→ normalized revision
→ semantic diff candidate
→ durable candidate and evidence receipt
→ accepted baseline advance
```

The accepted baseline MUST NOT advance if snapshot, parse, normalized digest, semantic diff, candidate write or audit/evidence write fails. No-change observations update health evidence but do not create a published semantic change.

Crash/retry uses deterministic revision/event identities. Publishing is a separate, reviewed state transition rather than part of the fetch transaction.

## 5. Fetch security

Privileged fetch accepts only a current `Approved` SourceDefinition projected as `RetrievalSubject` with exact `SourceAuthorityRevision`, then separately admitted through the authority/idempotency/lease transaction before any effect. A reviewed/model-proposed candidate remains `Proposed` and cannot be fetched. It MUST enforce:

- HTTPS by default and exact reviewed host/path policy;
- DNS/IP revalidation and redirect reauthorization on every hop;
- denial of loopback, private, link-local, metadata and multicast targets;
- bounded redirects, body size, time, concurrency and content types;
- no user credential forwarding by default;
- active-content isolation for HTML/PDF/attachments;
- untrusted page content treated as data, never instructions.

A model-proposed URL outside the registry enters review; it is not fetched in the current user request.

## 6. Normalized facts and publication

Every material fact carries source ID/revision, authority, observed/retrieval time, effective time when known, digest and conflict state.

Publication path:

```text
accepted SourceRevision
→ typed candidate
→ schema and cross-field validation
→ authority/conflict/citation validation
→ deterministic render or typed event
→ administrator review
→ canonical publish
→ projection refresh and audit receipt
```

Candidates, parser output and model inference have no canonical publish authority. A projection database/search index must be rebuildable from pinned canonical declarations and immutable evidence.

## 7. Lookup and freshness

Default lookup order is:

```text
L0 exact stable ID/path/normalized URL
→ L1 reviewed tree and structured local search
→ L2 approved-source targeted refresh and typed candidate
→ L3 bounded retrieval over approved snapshots (later)
```

A reviewed current artifact takes precedence over an unreviewed refresh/RAG candidate. Responses expose last verified, observed/effective time, stale/refresh-pending state and conflict/uncertainty. Insufficient authority yields `cannot_verify`, not invented completion.

## 8. Tenant-private data

Public source/graph facts and tenant-private profile facts are separate authority classes.

- A profile fact is created from explicit user input/consent or a documented tenant-local derivation.
- It is viewable and deletable by the owning user.
- Shared caches and derived projections are tenant-keyed.
- Private preferences/academic snapshots never enter public source, graph or feed projections.
- Deletion/revocation semantics must cover durable payload, recoverable logs and caches before release claims.

## 9. Failure and recovery

The system distinguishes timeout/unavailable source, unauthorized redirect, content violation, parser failure after changed content, normalized duplicate, official conflict, stale source, evidence-store failure, baseline drift and in-flight revoke.

Default recovery:

- retain the last accepted baseline and reviewed artifact;
- expose stale/conflict/uncertainty diagnostics;
- preserve failed candidate evidence for operator review where safe;
- never acknowledge unpublished or partially committed material as current;
- suspend further retrieval when source policy or evidence is revoked.

## 10. Verification

Current executable evidence is limited to the synthetic Course Planning fixture and its `COURSE-*` cases plus the bounded `M60-B1 source-registry` under `source-import/v0` (`SRC-001`). `source-import/v1` and `source-retrieval/v0` are accepted contract authority under R11; no v1 Rust implementation exists. Source/Procedure/Radar cases remain planned until a concrete reviewed public source and its parser fixtures are approved.

Primary entries:
- `docs/contracts/source-import.md`
- `docs/contracts/source-retrieval.md`
- `docs/contracts/data-models.md`
- `docs/tasks/m60-b2-retrieval-policy-readiness-proposal.md`
- `market/fixtures/course-planning/minimal-v0.json`
- `SRC-*`, `PROC-*`, `RADAR-*`, `COURSE-*` in `docs/acceptance/matrix.tsv`
