# S0 architecture review decision record

## Metadata

- `Layer`: Task / architecture review evidence
- `Status`: `Complete`
- `Version`: `s0-architecture-review/v1`
- `Last Review`: `2026-07-25`
- `Authority Owns`: explicit S0 reviewer dispositions, conditions and closure evidence
- `Authority Defers To`: [`../../AGENTS.md`](../../AGENTS.md), [`../plan/00-engineering-constitution.md`](../plan/00-engineering-constitution.md), [`../plan/01-terminology.md`](../plan/01-terminology.md), [`../plan/modules/00-module-map.md`](../plan/modules/00-module-map.md), [`../contracts/module-boundaries.md`](../contracts/module-boundaries.md), [`00-module-work-policy.md`](00-module-work-policy.md) and [`01-execution-roadmap.md`](01-execution-roadmap.md)

This record is a review ledger, not a second architecture authority. Accepted corrections are applied to their owning plan, contract, acceptance or task document; raw chat and reviewer transcripts remain outside the repository.

## 1. Review contract

Three independent lanes inspect the complete decision set:

- `architecture` — module ownership, dependency direction, composition and replaceability;
- `authority` — source of truth, tenant/security boundary, failure ordering and recovery;
- `delivery` — status honesty, acceptance projection, feasible sequencing and verification.

No more than three review subagents run concurrently. This is a concurrency limit, not a cap on total reviewers or review rounds. Silence and a missing lane are not acceptance.

Each lane returns, for every decision ID:

```text
Accept | ConditionalAccept | Reject
rationale
blocking condition, if any
required evidence and exit condition, if conditional
```

Disposition rules:

- `Accept` means the current owning documents are coherent enough to freeze that decision.
- `ConditionalAccept` names a condition owner, required evidence and decidable exit condition. It remains non-pass while its resolution is `open`.
- `Reject` blocks S0 completion and requires an owning-document correction plus fresh review.
- S0 closes only when every lane has returned, every decision is explicit, and every condition is `closed`.

## 2. Architecture brief

Reviewers read the authoritative chain rather than treating this summary as authority:

### Authority reading chain

```text
repository AGENTS, engineering constitution and terminology
→ module map and all 13 module blueprints
→ module-boundary registry and specific contracts
→ coverage matrix
→ active acceptance matrix and long-horizon catalog
→ module work policy and execution roadmap
→ retained code/tests claimed as bounded evidence
```

### Reviewed skeleton

The proposed frozen skeleton is:

```text
M80 thin Web/PWA and Android clients
→ M10 admitted Dioxus server-function / HTTP / typed-stream ingress
→ M00 and owned application/domain modules
→ M30 finite harness and composition-owned ToolGateway ordering
→ M40 bounded execution through current M20 authority and M51/peer executors

M60 owns source/revision/provenance truth
→ M70/M71/M72 own independent product state and journeys

M90 implements declared storage/clock/queue/secret/telemetry/deployment ports
without owning domain transitions
```

Current implementation evidence remains partial and does not silently freeze unfinished APIs. New retained product/business implementation remains blocked until this review closes and the owning module has exact active planned acceptance rows with future evidence bindings.

## 3. Review lanes

| Lane ID | Scope | Outcome | Blocking conditions |
|---|---|---|---|
| `architecture` | module ownership, acyclic dependencies, composition and replacement seams | `Pass` | — |
| `authority` | truth ownership, tenant/security boundaries, failure/effect ordering and recovery | `Pass` | — |
| `delivery` | status truth, active acceptance, sequencing, fakes and feasible evidence | `Pass` | — |

## 4. Decision ledger

`Review lanes` is exactly `` `architecture`; `authority`; `delivery` `` after all three lanes return. A dash is allowed only while the packet remains `InReview` and the disposition is `Pending`.

| Decision ID | Scope | Disposition | Review lanes | Basis | Condition owner | Required evidence | Exit condition | Resolution |
|---|---|---|---|---|---|---|---|---|
| `S0-A01` | stable four-layer call path plus object plane | `Accept` | `architecture`; `authority`; `delivery` | UI/CLI enters admitted application interfaces; coordination and execution remain distinct; the object plane names state rather than a fifth caller. | — | — | — | `closed` |
| `S0-A02` | one owner for every fact | `Accept` | `architecture`; `authority`; `delivery` | Git contracts, Rust transitions, durable state, evidence revisions and receipts have distinct repair authority; views and caches are rebuildable. | — | — | — | `closed` |
| `S0-A03` | large-module registry and dependency direction | `Accept` | `architecture`; `authority`; `delivery` | Exactly 13 independently owned modules compose through acyclic public boundaries; implementation state is a checked evidence posture. | — | — | — | `closed` |
| `S0-A04` | composition and private dependency prohibition | `Accept` | `architecture`; `authority`; `delivery` | `ustc-agentd` may map and order public operations but cannot copy domain rules or reach through private storage/implementations. | — | — | — | `closed` |
| `S0-A05` | Agent, ToolGateway and Plugin execution seam | `Accept` | `architecture`; `authority`; `delivery` | M30 owns finite run state, M40 orders current authority and bounded execution, and composition interleaves effect intent/receipt without a module cycle. | — | — | — | `closed` |
| `S0-A06` | source truth and first-party product separation | `Accept` | `architecture`; `authority`; `delivery` | M60 owns source/revision/provenance truth while M70/M71/M72 own independent product state, lifecycle and acceptance. | — | — | — | `closed` |
| `S0-A07` | Dioxus client and admitted ingress boundary | `Accept` | `architecture`; `authority`; `delivery` | M80 is a thin shared Web/Android presentation shell; M10 admits versioned server-function/HTTP/stream calls before owned application operations. | — | — | — | `closed` |
| `S0-A08` | status, acceptance and implementation sequencing | `Accept` | `architecture`; `authority`; `delivery` | Active `matrix.tsv` rows are pass authority; catalog-only/planned rows are non-pass; retained implementation starts only after contract-ready and exact acceptance prerequisites. | — | — | — | `closed` |
| `S0-M00` | Platform Control and Identity | `Accept` | `architecture`; `authority`; `delivery` | Own tenant/user/session identity, admitted actor/context and causation envelopes; do not absorb package, Agent, source or UI state. | — | — | — | `closed` |
| `S0-M10` | Application Ingress Host | `Accept` | `architecture`; `authority`; `delivery` | Own Dioxus/Axum ingress, compatibility, mapping and event transport; do not own domain decisions or direct database/executor access. | — | — | — | `closed` |
| `S0-M20` | Market and Package Lifecycle | `Accept` | `architecture`; `authority`; `delivery` | Own package/component catalog, installation, grants and current invocation authority; keep provider loop and execution outside. | — | — | — | `closed` |
| `S0-M30` | Agent Harness and Runtime | `Accept` | `architecture`; `authority`; `delivery` | Own finite HarnessRun/TaskGraph/context/review semantics and provider/tool ports; do not own package lifecycle or executor implementations. | — | — | — | `closed` |
| `S0-M40` | Tool Gateway and Execution | `Accept` | `architecture`; `authority`; `delivery` | Normalize and recheck one frozen tool call, order effect evidence and bound executor output without minting grants, run phases or domain receipts. | — | — | — | `closed` |
| `S0-M50` | Model Provider Integration | `Accept` | `architecture`; `authority`; `delivery` | Normalize typed provider profiles, request/stream usage and transport failures without owning run state or prompt truth. | — | — | — | `closed` |
| `S0-M51` | MCP Binding and Executor | `Accept` | `architecture`; `authority`; `delivery` | Own reviewed MCP endpoint/discovery/session/schema drift and bounded execution; do not publish Market state or mutate the Agent loop. | — | — | — | `closed` |
| `S0-M60` | Campus Trust and Source Pipeline | `Accept` | `architecture`; `authority`; `delivery` | Own source registry, immutable revisions, normalization, provenance, conflicts and accepted baseline; product rendering remains outside. | — | — | — | `closed` |
| `S0-M70` | USTC ChangeRadar | `Accept` | `architecture`; `authority`; `delivery` | Own semantic changes, board policy, approval and deterministic feed over M60 facts without becoming generic source authority. | — | — | — | `closed` |
| `S0-M71` | USTC Affairs Navigator | `Accept` | `architecture`; `authority`; `delivery` | Own reviewed procedure tree, validation, supersession and publication journeys over M60 facts without RAG-as-truth. | — | — | — | `closed` |
| `S0-M72` | Campus Opportunity Graph | `Accept` | `architecture`; `authority`; `delivery` | Own opportunity/qualification/dependency/conflict and tenant-private planning state; public source truth remains M60-owned. | — | — | — | `closed` |
| `S0-M80` | Client Core and Interaction Shells | `Accept` | `architecture`; `authority`; `delivery` | Original S0 review accepted shared Web/PWA/Android presentation with backend-owned calculations; Develata's later operation-specific [`ADR-0010`](../adr/0010-typed-client-peer-adapters.md) amendment adds peer `ustc-agent` and inbound MCP adapters over a typed client core and forbids GUI→CLI. No claim of a second team review. | — | — | — | `closed` |
| `S0-M90` | Platform Infrastructure and Operations | `Accept` | `architecture`; `authority`; `delivery` | Implement storage, journal, evidence, clock, queue, config, secret, telemetry and deployment ports without owning domain transitions. | — | — | — | `closed` |

## 5. Review evidence

All three lanes returned explicit `Accept` for every decision ID and no conditions:

- `architecture` — reviewed governance, all module blueprints, boundaries, acceptance/status projections and Cargo dependency shape; 70 Python contract tests and the repository checker passed.
- `authority` — reviewed truth ownership, tenant/security boundaries, failure/effect ordering and recovery; 70 Python tests plus 24 targeted Rust runtime/resolver/gateway/protocol tests passed.
- `delivery` — reviewed status truth, sequencing, acceptance prerequisites and checker false-green behavior; 70 Python tests, the repository checker and adversarial temporary mutations passed.

Review summaries were verified by the main agent. Raw transcripts and machine-local paths are intentionally not repository evidence.

## 6. S0 exit gate

S0 can be marked complete only when:

- every review lane has a recorded non-pending outcome;
- every decision is `Accept` or a `ConditionalAccept` whose resolution is `closed`;
- no unresolved ownership cycle, second authority, UI computation path or cross-module private dependency remains;
- accepted corrections are present in owning documents and repository checks pass;
- the roadmap and this packet report the same S0-3 state.

Closing S0 does not activate planned acceptance rows, make a module `StandaloneReady`, or authorize product logic. The next slice separately adds exact active planned acceptance rows and future bindings for the first retained M00/M10/M80/M90 root-skeleton work.
