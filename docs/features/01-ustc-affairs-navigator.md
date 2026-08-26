# USTC Affairs Navigator

- `Package ID`: `ustc.affairs-navigator`
- `Status`: Bounded executable procedure kernel plus one retained source-grounded noncanonical fixture, exact-query and loopback-only Web demonstration; complete administrator/source-ingestion/production-Web feature remains planned
- `Owning plan`: `docs/plan/06-first-party-plugins.md`
- `Contracts`: `docs/contracts/source-import.md`, `docs/contracts/permissions.md`
- `Acceptance`: `FP-001`, `PROC-*`

Current retained evidence covers checked stable procedure/artifact identities, draft/policy/freshness/conflict semantics and one retained source-grounded noncanonical fixture for the USTC undergraduate transcript/enrollment-certificate procedure. The exact lookup runs through M00→M10→M71 and is exposed both through the loopback `ustc-agentd`/`ustc-agent` product path and a loopback-only thin Web renderer. Retained and normalized source bytes are hash-accounted; this is not administrator review/publication or approved M60 source authority. It does not implement general M60 import, administrator review/publish/archive, supersession, broad structured search, production auth/TLS/remote HTTP, Dioxus, Android, inbound MCP or Agent/Market integration; `PROC-011` therefore remains planned.

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

One administrator-maintained board answers one real procedure with conditions, steps, time, sources and explicit uncertainty. The approved board/source and its data-use evidence must be frozen before implementation claims begin.
