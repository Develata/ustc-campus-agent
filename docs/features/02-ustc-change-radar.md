# USTC ChangeRadar

- `Package ID`: `ustc.change-radar`
- `Status`: Partial executable evidence; one fixed M00-admitted administrator command now durably publishes the reviewed semantic change behind loopback CLI/HTTP/Web while ordinary JSON/Atom reads retain M10/Market/Harness/ToolGateway
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

The retained Rust kernel proves two honestly labelled, canonical-URL-bound `DemoReviewed` revisions for one policy-pinned source identity, deterministic field-level comparison, digest-bound administrator review, coherent transaction-current M60 verification and one stable JSON/Atom item. A fixed administrator command recomputes its M10 payload digest, passes current M00 admission, durably persists or verifies admitted-request evidence, then calls only the owning M70 application port. Review/publication state is strict owner-only canonical JSON: exact retry returns one receipt, checked restart recovery performs zero M60 calls, post-rename parent-sync uncertainty reconciles the canonical file, and corrupt/replaced/unsafe state fails closed rather than becoming an empty feed. Ordinary reads still cross `M10 change.list → bounded Harness → current Market authorization → ToolGateway → owning M70 query`; real two-process/browser restart preserves the same GUID and Atom bytes. Production SSO/administration, approved live retrieval, maintainer leases and M80 peer clients remain unimplemented; `RADAR-002` remains planned.

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
- Duplicate/retry: deterministic IDs plus durable exact-retry identity produce one review, publication, M00 evidence event, receipt and feed item.
- Conflicting official sources: show conflict and block definitive publication until policy/admin resolution.
- Durable write or recovery failure: fail closed with no partial item; an uncertain post-rename commit is reconciled from the canonical file before retry.

## Non-goals

- arbitrary USTC-domain crawling;
- maintainer Agent publication authority;
- raw page-change feed;
- user Plugin write access to the internal ingestion ledger;
- personalized private feeds before consent/profile design is implemented.

## First honest acceptance

One approved historical source change replays into one reviewed semantic event with exact evidence. Failure cannot advance the accepted baseline, and publication occurs exactly once.
