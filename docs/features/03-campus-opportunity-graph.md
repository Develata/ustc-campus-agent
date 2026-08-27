# Campus Opportunity Graph

- `Package ID`: `ustc.opportunity-graph`
- `Status`: Partial evidence; offline Course Planning plus consent/profile foundation implemented
- `Owning plan`: `docs/plan/06-first-party-plugins.md`
- `Contracts`: `docs/contracts/data-models.md`, `docs/contracts/source-import.md`
- `Acceptance`: `COURSE-*`, future graph/profile cases

## Goal

Answer “What fits me, and what should I choose next?” by combining reviewed opportunity facts with an explicitly consented tenant-private profile projection.

## User-visible result

An opportunity explanation can show:

- qualification/eligibility and unmet conditions;
- prerequisites and dependencies;
- temporal windows and conflicts;
- evidence and source freshness;
- which user-provided facts affected the match;
- uncertainty and next action.

## Product journey

```text
user installs/enables Opportunity Graph
→ reviews and consents to narrow profile fields
→ browses reviewed public opportunities
→ requests match/path/plan candidates
→ system applies typed qualification, dependency, time and conflict rules
→ explanation exposes evidence and profile inputs
→ user edits or deletes private profile facts
```

This full installed-Plugin journey remains planned. Retained executable evidence now
includes the offline Course Planning pack plus an in-memory, fixture-backed domain
path for exact consent, tenant-private profile creation, qualification/planning,
source/profile staleness and consent-revoke/profile-delete behavior.

## Course Planning bounded spike

The current slice maps:

| Graph concept | Course Planning projection |
|---|---|
| `OpportunityNode` | course offering |
| `RequirementNode` | curriculum requirement/elective group |
| `DependencyEdge` | prerequisite or recommended-before |
| `CoverageEdge` | course satisfies requirement |
| `ConflictEdge` | time or rule conflict |
| `TemporalWindow` | term/effective period |
| `EvidenceSignal` | official fact, mirror fact, community link-out, uncertainty |
| `ProfileFact` | user-provided academic snapshot/preference |

`crates/course-planning` validates the synthetic `course-planning/v0` fixture,
enforces source authority and hard constraints, and emits deterministic
`course-plan-result/v0` JSON through `ustc-agentctl course plan`.
`crates/opportunity-graph` consumes that pack only after exact principal/consent
checks and binds results to one `DemoReviewed` M60 revision plus one private
profile snapshot.

The slice is offline and read-only. It performs no live source import, account mutation or enrollment.

## Source and profile boundaries

```text
official catalog snapshot/API
> reviewed official notice/department source
> iCourse program mirror
> community signal/link-out
> model inference
```

Community signal affects soft ordering only; it cannot author requirements, credits, prerequisites, availability or schedule facts.

Public opportunity facts and tenant-private profile facts remain separate. Private inputs are viewable/deletable, tenant-keyed and excluded from public graph/feed/cache projections.

## Failure and recovery

- Missing/stale/conflicting hard fact: exclude, warn or refuse; do not guess.
- Unresolved course alias: exclude instead of silently merging.
- Over-budget planning: narrow/decompose or fail explicitly.
- Explanation adds an unapproved course/fact: consistency gate rejects it and falls back to typed planner output.
- Profile scope/consent ambiguity: do not read or derive a match.

## Non-goals and honest status

The current foundation does **not** establish M10/Web/CLI composition, Market
installation/grants/enable-disable, Agent discovery, shared ToolGateway execution,
live source ingestion, durable consent-aware profile state or backup-erasure proof.
It does not make Opportunity Graph the sole flagship or move it ahead of the
three-Plugin MVP requirement.

Future research, competition, lecture and scholarship packs must reuse the same trust/profile semantics or obtain a new ADR for a materially different ontology.
