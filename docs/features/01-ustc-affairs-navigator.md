# USTC Affairs Navigator

- `Package ID`: `ustc.affairs-navigator`
- `Status`: Bounded executable query kernel plus exact M60 `DemoReviewed` draft/review/atomic-publication foundation composed through one same-repository fixture-backed M10/loopback-Web path; M00-authorized operator publication and durable restart remain planned
- `Owning plan`: `docs/plan/06-first-party-plugins.md`
- `Contracts`: `docs/contracts/source-import.md`, `docs/contracts/permissions.md`
- `Acceptance`: `FP-001`, `PROC-*`

Current retained evidence covers checked stable procedure/artifact identities, policy/freshness/conflict semantics, exact M60 `DemoReviewed` revision import, one coherent M60-owned publication decision over source health and retained evidence, digest-bound administrator review, deterministic artifact/receipt IDs and CAS/idempotent atomic publication into a bounded in-memory repository. Committed receipt/artifact tombstones support exact replay after later revision or source revocation; uncommitted retries still require fresh M60 authority. The source-grounded noncanonical fixture now drives startup draft/review/publication and the existing exact M00→M10→M71 `ustc-agentd`/`ustc-agent` query plus thin loopback Web renderer from that same repository. The fixture actor and approval constructor are not M00 authorization. There is no real source approval/retrieval, durable persistence/restart, operator publication API, supersession, broad search, production Dioxus Web, Android, inbound MCP or complete Agent/Market integration; `PROC-011` therefore remains planned.

## Goal

Answer “What should I do now?” with a reviewed campus procedure rather than a plausible free-form answer.

## User-visible result

A procedure answer exposes:

- applicable audience and conditions;
- prerequisites;
- ordered steps;
- deadlines/effective time;
- official entry points and contacts when available;
- exact sources and last-verified time;
- conflicts, staleness and uncertainty.

## Journey

```text
user asks a campus procedure question
→ exact stable ID/path/URL lookup
→ reviewed tree + structured search
→ current reviewed procedure appears with evidence/freshness
→ if stale and policy permits, targeted refresh creates a candidate
→ administrator reviews and publishes a new artifact
→ user sees the updated reviewed procedure
```

Bounded retrieval over approved snapshots is a later fallback for evidence location. It cannot override the current reviewed artifact or invent missing steps.

## States

```text
Current reviewed
Stale / refresh pending
Candidate under review
Conflicting / cannot verify
Archived historical artifact
```

## Failure and recovery

- Insufficient authoritative evidence: return `cannot_verify` and link available source context.
- Stale source: show the last reviewed procedure with a stale warning; do not silently substitute an unreviewed candidate.
- Citation/policy validation failure: candidate stays unpublished.
- Partial replacement: retain both scopes/history; do not archive the old procedure as fully replaced.
- Publication failure: keep the previous current artifact and surface operator diagnostics.

## Non-goals

- full-corpus RAG as first truth path;
- live crawl on every user question;
- Agent-owned canonical publication;
- automatic form submission or account mutation;
- a second source/crawler authority separate from ChangeRadar.

## First honest acceptance

One M00-authorized demo administrator imports a non-personal source-grounded snapshot explicitly labelled `DemoReviewed`, publishes one procedure through the real application boundary, and a user queries the same persisted state through Web with conditions, steps, time, sources and explicit uncertainty. The demo label must not imply real-time official data or legal approval.
