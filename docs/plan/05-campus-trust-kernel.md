# Campus Trust Kernel

## Metadata

- `Layer`: Shared campus authority
- `Status`: Contract accepted under R11 M60-B2 two-layer transport architecture; `source-import/v1` and `source-retrieval/v0` are current contract authority per `ACCEPT_EXACT_M60_B2_R11_PACKET` (2026-08-13); bounded `M60-B1 source-registry` remains implemented under `source-import/v0` (P1-1); operational `Suspended`/`Revoked` lifecycle precondition applies before any live B2 retrieval adapter; concrete source approval, retained B2 implementation and network retrieval remain unauthorized; the superseded V10 `DEC-M60-B2-ACCEPTANCE` is historical evidence only
- `Version`: `0.3.0`
- `Last Review`: `2026-08-12`
- `Authority Owns`: source identity, immutable revision, authority order, temporal/conflict/provenance state, baseline advancement and publication gates
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
