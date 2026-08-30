# USTC Affairs Navigator

- `Package ID`: `ustc.affairs-navigator`
- `Status`: Bounded `PROC-011` core-demo path implemented: fixed administrator publication uses `M10 → M00 admission/durable evidence → M71`, while ordinary reads remain on `M10 → deterministic Harness → current Market authorization → ToolGateway → M71`; publication state has strict durable restart recovery and loopback HTTP/CLI/Web projection
- `Owning plan`: `docs/plan/06-first-party-plugins.md`
- `Contracts`: `docs/contracts/source-import.md`, `docs/contracts/permissions.md`
- `Acceptance`: `FP-001`, `PROC-*`

Current retained evidence covers checked stable procedure/artifact identities, exact M60 `DemoReviewed` import, digest-bound review, deterministic IDs, CAS replay and a recovery anchor that binds every durable record to the fixture draft/reviewer/time/M60 decision before recreating a sealed M71 commit. The fixed administrator command recomputes its M10 payload digest, passes current M00 session/permission/capability admission, persists or verifies durable redacted control evidence, and only then calls M71. Canonical JSON state is owner-only, bounded and atomically replaced; malformed, noncanonical, reordered, duplicated, gapped, oversized, symlink/hardlink/FIFO/directory, mode or runtime-replacement cases fail closed. A two-process test and loopback CLI/Web/browser smokes recover revision 2, the exact receipt and one control-evidence event; after the fixture's separately validated bootstrap, durable reopen/retry itself performs no additional M60 publication decision. Ordinary user reads continue through Market/Harness/ToolGateway and still return the source-grounded result after restart. This is bounded proof, not production SSO/live retrieval/generic content management; supersession, broad search, Dioxus/Android and inbound MCP remain open.

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
