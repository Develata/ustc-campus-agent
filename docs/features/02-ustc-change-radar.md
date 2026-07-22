# USTC ChangeRadar

- `Package ID`: `ustc.change-radar`
- `Status`: Planned manifest skeleton; source/revision/diff is the next first-party product mainline
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
