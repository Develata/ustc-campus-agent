# Campus Opportunity Graph

- `Package ID`: `ustc.opportunity-graph`
- `Status`: Development; bounded offline Course Planning spike implemented
- `Owning plan`: `docs/plan/06-first-party-plugins.md`
- `Contracts`: `docs/contracts/data-models.md`, `docs/contracts/source-import.md`, `docs/contracts/user-context-profile.md`
- `Acceptance`: `COURSE-*`, future graph/profile cases

## Goal

Answer “What fits me, and what should I choose next?” by combining reviewed opportunity facts, a minimum purpose-bound M00 user-context projection and separately consented M72 opportunity preferences.

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
→ reviews the exact M00 profile fields and purpose requested by this planning action
→ optionally records M72-specific planning preferences/constraints
→ browses reviewed public opportunities
→ requests match/path/plan candidates
→ system applies typed qualification, dependency, time and conflict rules
→ explanation exposes evidence and profile inputs
→ user edits general profile facts through M00 or edits/deletes M72 preferences through the product path
```

This full journey is planned; only the offline Course Planning subset currently runs.

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
| M00 `CurrentProfileProjection` | purpose-bound academic context with exact revision |
| M72 `OpportunityPreference` | user-provided planning preference/constraint |

`crates/course-planning` validates the synthetic `course-planning/v0` fixture, enforces source authority and hard constraints, and emits deterministic `course-plan-result/v0` JSON through `ustc-agentctl course plan`.

The slice is offline and read-only. It performs no live source import, account mutation or enrollment.

## Source, profile and preference boundaries

```text
official catalog snapshot/API
> reviewed official notice/department source
> iCourse program mirror
> community signal/link-out
> model inference
```

Community signal affects soft ordering only; it cannot author requirements, credits, prerequisites, availability or schedule facts.

Public opportunity facts, M00-owned general profile facts/projections and M72-owned preferences remain separate. Private inputs are viewable/deletable through their owning module, tenant/user/purpose keyed and excluded from public graph/feed/cache projections. Opportunity Graph cannot mutate or accept an AI proposal for the M00 general profile.

## Failure and recovery

- Missing/stale/conflicting hard fact: exclude, warn or refuse; do not guess.
- Unresolved course alias: exclude instead of silently merging.
- Over-budget planning: narrow/decompose or fail explicitly.
- Explanation adds an unapproved course/fact: consistency gate rejects it and falls back to typed planner output.
- M00 profile purpose/scope or M72 preference consent ambiguity: do not read or derive a match.

## Non-goals and honest status

The current spike does **not** establish Market installation, grants, enable/disable, Agent discovery, live source ingestion, M00 profile consumption or durable M72 preference state. It does not make Opportunity Graph the sole flagship or move it ahead of the frozen cross-Plugin implementation order.

Future research, competition, lecture and scholarship packs must reuse the same trust/profile semantics or obtain a new ADR for a materially different ontology.
