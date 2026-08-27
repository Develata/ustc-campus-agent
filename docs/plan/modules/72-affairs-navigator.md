# M71 — USTC Affairs Navigator

## Metadata

- `Module ID`: `M71`
- `Package ID`: `ustc.affairs-navigator`
- `Status`: Accepted blueprint with bounded executable query kernel plus exact M60 `DemoReviewed` draft import, one coherent M60-owned publication decision over source health and retained evidence, digest-bound administrator approval and bounded atomic publication evidence; M00 authorization, durable restart and production application/Web composition remain planned
- `Implementation State`: `partial-evidence`
- `Version`: `m71-affairs-navigator/v0`
- `Last Review`: `2026-08-27`
- `Primary code area`: `crates/affairs-navigator/`, `plugins/first-party/affairs-navigator/` and the bounded composition fixture under `apps/ustc-agentd/`

## 1. Purpose

`M71` answers: “What should I do now?” It owns reviewed campus procedure trees and artifacts: applicability, prerequisites, ordered steps, deadlines/effective time, entry points, sources, conflicts and uncertainty.

Its default truth path is exact/structured lookup over reviewed material, not free-form full-corpus generation.

### Current bounded evidence

`crates/affairs-navigator` retains checked stable procedure/artifact/source identities, typed drafts and artifacts, board-policy validation, ordered conditions/steps/time/entry projections, freshness/conflict/uncertainty outcomes and deterministic exact lookup. The publication foundation now imports the exact M60-owned immutable `DemoReviewed` revision, binds its canonical URL/revision/digests into the draft identity, rejects mismatched or unpublishable evidence, obtains source health and retained-evidence verification as one coherent M60-owned publication decision, binds administrator review to the exact draft digest and atomically commits deterministic artifact/receipt IDs with CAS, caps and replay tombstones. A committed tombstone owns exact replay after later revision or revocation; an uncommitted retry still requires fresh M60 authority. Separately, the bounded `ustc-agentd` composition still loads one source-grounded noncanonical fixture and maps an admitted exact stable-ID request to the earlier query kernel; the loopback page displays that safe public projection.

The fixture is proof infrastructure, not an approved production source. `DemoReviewed` is an honest non-personal demo label, not real-time official data or legal approval. The public approval constructor also proves no M00 role by itself; the M60 publication port owns the coherent source-health/evidence decision, and future M00/M10 composition must own actor authorization. The atomic repository is bounded in-memory evidence only: no durable adapter/restart, real retrieval/source approval, same-repository application query, supersession projection, broad structured search, production Web or external campus transaction is claimed. `PROC-011` therefore remains planned until its complete M00-authorized administrator-import → review/publish → same-state application query → thin-Web and restart assertions are bound.

## 2. Non-goals

- fetching/approving sources or owning source revisions;
- letting an LLM invent missing steps/citations;
- automatic execution of campus procedures or enrollment/payment;
- full-corpus RAG as the initial authority path;
- using filesystem path/slug as stable procedure identity;
- owning ChangeRadar feed state.

## 3. Owned objects and state

```text
BoardId / NodeId
ProcedureId / ArtifactId
BoardPolicy
ProcedureDraft / ProcedureArtifact
ProcedureState:
  Discovered | Generated | Validated | Published | Archived | Failed
SupersessionEdge:
  Full | Partial | Clarification | Duplicate
ProcedureLookup/Uncertainty result
```

Stable IDs survive path/title changes. Reviewed published artifacts are canonical; search indexes and rendered paths are projections.

## 4. Public inputs and outputs

Inputs:

```text
accepted M60 source revision/facts
administrator-authored or bounded candidate ProcedureDraft
board policy and current artifact references
validate/review/publish/archive commands
exact/structured lookup query
```

Outputs:

```text
validated ProcedureDraft
reviewed ProcedureArtifact
conditions, prerequisites, ordered steps, time, entries, sources
conflict/uncertainty/freshness result
stable node/procedure/artifact events and errors
```

## 5. Dependency direction

Allowed dependencies:

- `M60` source/revision/provenance/freshness contracts;
- `M00` actor/request context for publication;
- `M90` repository/artifact/search/render/event ports;
- optional `M30` bounded candidate assistance through a typed application use case, never as publication authority.

Forbidden dependencies:

- source retrieval/parser internals;
- `M70`/`M72` private state;
- Dioxus/client types;
- model prompts as policy;
- concrete filesystem/search/database paths as stable domain IDs.

## 6. Lifecycle

```text
source evidence or administrator input
→ Discovered/Generated ProcedureDraft
→ Rust schema/cross-field/policy/citation validation
→ administrator review
→ Published ProcedureArtifact
→ deterministic render/search projection
→ Archived or superseded by explicit typed edges
```

A formatting hook may normalize presentation only. It cannot fill semantics, citations or publish.

## 7. Failure and recovery

- Missing required field/source/effective time: invalid draft, prior artifact remains current.
- Insufficient/stale/conflicting authority: explicit uncertainty/cannot-verify.
- Render/search failure after approval: canonical artifact remains; rebuild projection.
- Publish/evidence transaction failure: do not acknowledge current artifact.
- Ambiguous supersession coverage/time: reject full replacement.
- Concurrent publication: version/precondition conflict; one accepted revision.
- Path/slug move: identity unchanged; update projection only.

## 8. Configuration and secrets

Board policy declares stable board/node scope, required sections, source authority, maximum staleness, conflict/supersession rules, review roles and render/search settings. It contains no source credentials, model keys or private profile values.

## 9. Observability

Record procedure/artifact/policy/source revision IDs, validation findings, review/publish/archive transitions, supersession edges, freshness/conflict/uncertainty and lookup path (exact/structured/fallback). Metrics do not count unreviewed candidates as answered procedures.

## 10. Extension and replacement

Draft producers, validators, deterministic renderers and search projections are replaceable peers. The typed artifact/policy remains stable. A later bounded retrieval/Agent explainer consumes reviewed snapshots and returns uncertainty; it cannot bypass exact/structured lookup or publication.

## 11. Performance path

Exact stable-ID/path/normalized-URL lookup is first, followed by bounded structured local search. Indexes rebuild from reviewed artifacts. Rendering is deterministic and cacheable by artifact digest. Broad model retrieval is not on the default hot path.

## 12. Scope boundary

**MVP**

- one administrator-maintained board;
- stable node/procedure/artifact IDs;
- one typed ProcedureDraft and board policy;
- deterministic validation/render;
- explicit review/publish/archive;
- exact + structured search;
- source/effective-time/uncertainty display.

**Later**

- more boards and source-driven refresh candidates;
- bounded retrieval over approved snapshots;
- Agent-assisted draft/explanation under evidence limits;
- links from ChangeRadar approved events.

**Explicit non-goals**

- automatic campus form submission;
- unreviewed model answer as current procedure;
- full-corpus RAG-first truth path;
- path-first identity.

## 13. Small-module decomposition

1. `affairs-tree` — board/node identity and hierarchy.
2. `board-policy` — required sections, source/staleness/review rules.
3. `procedure-draft` — typed candidate schema.
4. `procedure-validation` — cross-field/policy/citation checks.
5. `procedure-artifact` — reviewed canonical value and lifecycle.
6. `supersession` — explicit edges and replacement proof.
7. `procedure-render` — deterministic Markdown/JSON projection.
8. `procedure-search` — exact and structured lookup.
9. `procedure-review` — administrator dispositions and receipts.
10. `affairs-ports` — repository/artifact/search/event fakes.

## 14. Exit gate

`M71` is standalone-ready when fixtures prove validation, deterministic render, stable identity, supersession, stale/conflict/uncertainty and projection rebuild. It is accepted when one real reviewed procedure answers a concrete question with conditions, steps, time, entry points and source evidence, while insufficient evidence returns uncertainty.
