# Three default first-party Plugins

## Metadata

- `Layer`: First-party product contracts
- `Status`: Identities/manifests accepted; product journeys mostly planned
- `Version`: `0.4.0`
- `Last Review`: `2026-08-15`
- `Authority Owns`: first-party product split, shared semantics, independent lifecycles and implementation order
- `Authority Defers To`: Campus Trust Kernel for shared authority and typed contracts/manifests for exact fields
- `Counterpart Features`: `docs/features/01-ustc-affairs-navigator.md`, `docs/features/02-ustc-change-radar.md`, `docs/features/03-campus-opportunity-graph.md`
- `Counterpart Contracts`: `docs/contracts/plugin-package.md`, `docs/contracts/source-import.md`, `docs/contracts/data-models.md`
- `Counterpart Acceptance`: `FP-*`, `PROC-*`, `RADAR-*`, `COURSE-*`
- `Primary Code Areas`: `plugins/first-party/`, `market/packages/`, current `crates/course-planning/`
- `Large-module Blueprints`: [`M70 ChangeRadar`](modules/71-change-radar.md), [`M71 Affairs Navigator`](modules/72-affairs-navigator.md), [`M72 Opportunity Graph`](modules/73-opportunity-graph.md)

## 1. Product split and shared authority

```text
USTC Affairs Navigator: What should I do now?
USTC ChangeRadar: What changed, and does it affect me?
Campus Opportunity Graph: What fits me, and what should I choose next?
```

| Plugin | Public projection | First honest user result |
|---|---|---|
| Affairs Navigator | reviewed procedure projection | conditions, steps, deadlines, entry points, sources and uncertainty |
| ChangeRadar | approved semantic-change projection | before/after, effective time, affected scope, provenance and feed item |
| Opportunity Graph | opportunity facts + consent-aware profile projection | qualification, dependency, conflict, match and next action |

The products share one Campus Trust Kernel:

```text
approved sources
→ Source Registry
→ immutable revisions
→ normalized facts with time/conflict/provenance
   ├── reviewed procedure artifacts
   ├── approved semantic change events
   └── reviewed opportunity graph facts + tenant-private profile projection
```

They MUST NOT build three crawler authorities or three incompatible source identities. They remain independent large modules and packages with separate versions, branches/owners, installation, enablement and acceptance. Each can be completed and attached through its public boundary without requiring the other two product implementations to finish.

## 2. USTC Affairs Navigator

### 2.1 Lookup ladder

```text
L0 exact node/procedure ID or normalized URL
→ L1 reviewed tree + structured local search
→ L2 approved-source refresh + typed candidate
→ L3 bounded retrieval over approved snapshots (later)
```

The last reviewed current procedure wins over unreviewed candidates. Full-corpus RAG is not the initial truth path.

### 2.2 Tree and policy

Stable identity belongs to `node_id`, `procedure_id` and `artifact_id`; filesystem paths and slugs are movable projections. A versioned board policy owns:

- replacement key and authority order;
- required sections;
- maximum staleness;
- conflict/supersession rules;
- administrator-publication requirement.

Critical coverage and publication rules belong in typed policy/validator code rather than only in prompts.

### 2.3 Procedure artifact

A `ProcedureDraft` candidate contains at least:

```text
procedure key and title
applies_to
prerequisites[]
ordered steps[]
deadlines/effective time[]
entry points[] and contacts[]
source_revision_ids[]
conflicts[] and uncertainties[]
last_verified_at
```

Lifecycle:

```text
Discovered → Generated → Validated → Published → Archived
                       └→ Failed
```

Canonical materialization is:

```text
accepted SourceRevision
→ reviewed Skill produces typed ProcedureDraft
→ Rust schema/cross-field/policy/citation validation
→ deterministic Markdown
→ administrator review
→ atomic publish + projection refresh
```

A formatting hook MAY normalize presentation only. It cannot fill missing semantics, invent citations or publish.

### 2.4 Supersession

Only direct typed edges are stored: `Full | Partial | Clarification | Duplicate`. Full replacement requires equal-or-higher authority, preserved audience/scope, explicit field coverage, no unexplained effective-time gap and replayable evidence. Archive is a state transition, not silent deletion.

### 2.5 First product slice

The first Affairs Navigator product slice follows the competition delivery posture ([`02-product-positioning.md`](02-product-positioning.md) §8):

```text
administrator-imported reviewed snapshot (or later one exact approved public source)
→ SourceRevision / evidence with bitemporal provenance (valid_at / known_at) and
  review/verification metadata (reviewed_at / last_verified_at); query binds as_of cutoff
  (see §2.6 and plan/05 §3.1)
→ one typed ProcedureDraft validated against board policy
→ administrator review and publish
→ one application query (exact stable ID or structured search)
→ one thin Web result with conditions, steps, effective/deadline time, entry point,
  safe public lineage/evidence, freshness, conflicts and uncertainty
```

The `ustc-teach-calendar-fall` candidate family is the foundation for the administrator-imported reviewed snapshot path. This decision does not itself approve a concrete source, authorize network retrieval, or commit raw HTML. If the calendar evidence cannot ground one honest procedure, it remains source/revision fixture groundwork and the narrowest separately reviewed Affairs procedure snapshot is selected before claiming the product acceptance.

The thin Web surface renders typed server-owned state and captures intent only. Its public lineage is limited to `source_id`, `evidence_set_digest`, `materialization_receipt_id` and `revision_count`; it emits no raw revision identity or response-only capability. It does not require Agent, Market artifact switching, Android, CLI or inbound MCP.

### 2.6 Evidence context (planned)

The first product slice's thin Web result carries an evidence context that distinguishes the bitemporal fact vocabulary ([`05-campus-trust-kernel.md`](05-campus-trust-kernel.md) §3.1) from review/verification metadata. The planned Affairs Navigator evidence context fields are:

- `valid_interval`: real-world validity interval or point for the procedure's underlying facts, projected from source effective intervals (fact-level `valid_at`).
- `observed_at`: when the source revision was first observed/retrieved by the system (source-revision evidence only; it does not supply fact-level `known_at`).
- `known_at`/recorded-at: the earliest durable materialization/recording time for this exact procedure fact revision and parser output. Reprocessing retained source bytes later mints a later `known_at` for newly extracted facts rather than backdating them to `observed_at`.
- `reviewed_at`: when an administrator reviewed and accepted this source revision into the baseline (evidence/procedure level, not fact-level; not `as_of`).
- `last_verified_at`: when the evidence was last re-verified against its source (evidence/procedure level).
- source revision refs: immutable `SourceRevision` references backing the procedure.
- conflict/uncertainty: whether the fact had no known conflict, equivalent sources, an authority-resolved conflict, or unresolved uncertainty.

The query/answer `as_of` cutoff ([`05-campus-trust-kernel.md`](05-campus-trust-kernel.md) §3.1) is bound at query time, not stored per-evidence: the same procedure's evidence may be re-queried under different `as_of` cutoffs. The v0 Course Planning `FactProvenance` does not carry these fields; they land with the Affairs Navigator first product slice.

## 3. USTC ChangeRadar

### 3.1 Foundation

The first engineering slice implements one reviewed public source through:

```text
stable source/revision identity
→ conditional retrieval
→ immutable raw/normalized snapshots
→ deterministic parser/normalizer
→ semantic diff candidate
→ durable evidence
→ accepted baseline advance
```

Fetch/parser/evidence failure leaves the last accepted baseline unchanged. Repeated processing is idempotent; arbitrary URL fetch is forbidden.

### 3.2 Maintainer authority

A board-scoped maintainer Agent receives only node/source allowlist, policy version, lease/cursor, bounded budget and candidate-write permission. It has no canonical publication, cross-board mutation, broad profile access or platform credential.

Concurrency uses `(node_id, source_id)` leases, deterministic candidate/event IDs and idempotency keys derived from source revision/digest and policy version.

### 3.3 Semantic change and feed

`ChangeEvent` binds stable event/node IDs, old/new revisions, semantic diff, affected scope and `Proposed | Approved | Published | Rejected` state.

HTML layout noise, duplicate fetch, parser failure and unreviewed inference never become published change. Per-board RSS/Atom includes stable GUID, node, before/after, published/effective time, affected scope, current procedure/diff links and provenance. Subscription binds stable `node_id`, not mutable path.

## 4. Campus Opportunity Graph

Public opportunities are typed nodes/edges with eligibility, dependency, coverage, conflict, temporal window and evidence. Tenant-private profile facts remain a separate consent-aware projection.

Minimum semantics:

- qualification/prerequisite conditions retain scope and source;
- temporal validity produces deterministic current views;
- every material graph fact carries provenance;
- private preferences and derived edges never enter the public graph;
- user-owned facts are viewable and deletable;
- matching/explanation exposes uncertainty rather than filling missing facts.

### 4.1 Course Planning bounded spike

Current `crates/course-planning` validates synthetic `course-planning/v0` fixtures, applies source authority and hard constraints, and emits deterministic `course-plan-result/v0` JSON.

It proves only offline typed validation and planning. It does not prove Market installation, grants, enable/disable, Agent discovery, live source ingestion or consent-aware durable profile state. The package therefore claims `development`, not platform completion.

Future research, competition, lecture and scholarship slices reuse the same trust/profile semantics. A materially different ontology requires a new ADR.

### 4.2 ChangeRadar bounded foundation

Current `crates/change-radar` consumes M60-owned immutable `DemoReviewed` revision values and proves bounded board-policy validation, deterministic typed field comparison, stable candidate identity, explicit stale/conflict/unavailable outcomes, digest-bound administrator review, coherent transaction-current M60 verification and deterministic JSON/Atom rendering. A fixed command now crosses `M10 → M00 durable evidence → owning M70`; an app-private owner-only canonical repository preserves one review/publication/GUID across exact retry and checked zero-M60 restart recovery, reconciles post-rename uncertainty from disk and fails closed on unsafe/corrupt state.

This remains supporting `partial-evidence`, not the complete ChangeRadar product journey. Ordinary public `change.list` still crosses M10, current Market authorization, bounded Harness/ToolGateway and the owning query service, while loopback CLI/HTTP/Web expose only the fixed administrator demo and public JSON/Atom read-back. `AUTH-024` and `RADAR-001` are implemented. Source retrieval/parser and durable accepted baseline, approved live source, production SSO/admin, maintainer leases, M80 peers and `RADAR-002` remain planned.

## 5. Frozen implementation order

```text
1. ChangeRadar source/revision/diff foundation
2. Affairs Navigator structured procedure entry
3. ChangeRadar per-board feed
4. Opportunity Graph consent/profile integration
```

The Course Planning spike was completed out of order and does not alter this sequence.

## 6. Exit gates

- **ChangeRadar foundation**: one approved historical change replays with exact evidence; failure cannot advance baseline.
- **Affairs entry**: one administrator-maintained board answers a real procedure with conditions, steps, time, sources and uncertainty.
- **ChangeRadar feed**: one approved semantic change publishes exactly once; crawl noise never enters feed.
- **Opportunity integration**: existing planner appears behind honest install/grant/discovery and tenant-private profile boundaries.
- **Three-product demo**: exact packages bootstrap and can be disabled/re-enabled independently; every material answer exposes provenance.

Bindings live in `FP-*`, `SRC-*`, `PROC-*`, `RADAR-*` and `COURSE-*` acceptance rows.
