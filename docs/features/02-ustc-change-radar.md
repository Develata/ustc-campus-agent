# USTC ChangeRadar

- `Package ID`: `ustc.change-radar`
- `Status`: Partial executable evidence; source/revision/diff/review/Atom kernel now has one fixture-backed M10/Market/Harness/ToolGateway/loopback-Web board path, while durable restart and M00-authorized administration remain planned
- `Owning plan`: `docs/plan/06-first-party-plugins.md`
- `Contracts`: `docs/contracts/source-import.md`, `docs/contracts/permissions.md`
- `Acceptance`: `FP-002`, `RADAR-*`, `SRC-*`

## Goal

Answer “What changed, and does it affect me?” with an approved semantic change rather than crawl/hash noise.

## User-visible result

A change item exposes:

- what changed before/after;
- publication and effective time;
- affected audience/scope;
- current procedure or destination;
- exact source revisions and evidence;
- explicit uncertainty when sources conflict;
- stable feed identity.

## Journey

```text
reviewed source is fetched under policy
→ immutable raw/normalized revision is stored
→ deterministic semantic diff becomes candidate
→ board-scoped maintainer proposes affected scope/summary
→ administrator approves semantic change
→ one per-board RSS/Atom item is published
→ user opens evidence/current procedure
```

The initial user may consume public per-board feeds without a tenant-private profile. Personalized impact comes only after Opportunity Graph/profile consent semantics exist.

The retained Rust kernel currently proves two honestly labelled, canonical-URL-bound `DemoReviewed` revisions for one policy-pinned source identity, deterministic field-level before/after comparison, duplicate/no-change/out-of-order/stale/conflict/unavailable outcomes, digest-bound administrator approve/reject receipts, a coherent transaction-current M60 publication decision, exactly-once in-memory publication and deterministic Atom entries with stable GUID, changed fields, effective/observed time, source evidence, freshness and provenance. A retained no-personal-data fixture now projects that published event through `Web → M10 change.list → bounded Harness → current Market authorization → ToolGateway → fixed ChangeRadar adapter → owning query service → M10 wire projection → Web/Atom`; disabled installation or revoked grant denies before intent/executor, unknown boards return a stable Plugin-owned result, and disabling Affairs does not disable ChangeRadar. It still has no M00-authorized administrator command, durable repository/restart, real source retrieval or M80 peer-client composition; none of the acceptance rows is promoted by this supporting slice.

## States

```text
No semantic change
Candidate under review
Approved
Published
Rejected
Source stale/suspended
```

## Failure and recovery

- Fetch/parser/evidence failure: retain the last accepted baseline and publish nothing.
- Layout-only/hash noise: record diagnostics if useful; do not emit a semantic event.
- Duplicate/retry: deterministic IDs produce at most one candidate/event.
- Conflicting official sources: show conflict and block definitive publication until policy/admin resolution.
- Feed publication partial failure: retry by stable event GUID without duplicating the item.

## Non-goals

- arbitrary USTC-domain crawling;
- maintainer Agent publication authority;
- raw page-change feed;
- user Plugin write access to the internal ingestion ledger;
- personalized private feeds before consent/profile design is implemented.

## First honest acceptance

One approved historical source change replays into one reviewed semantic event with exact evidence. Failure cannot advance the accepted baseline, and publication occurs exactly once.
