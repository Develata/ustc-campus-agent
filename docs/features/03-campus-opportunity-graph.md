# Campus Opportunity Graph

- `Package ID`: `ustc.opportunity-graph`
- `Status`: Partial evidence; common-platform consent/profile/planning/Web slice in progress
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

This full installed-Plugin journey is the active M72 slice. The current candidate
combines the offline Course Planning pack with exact-consent M10 commands,
transaction-current M20 authorization of four static M72 application use cases, an
M60-bound `DemoReviewed` catalog adapter, a mode-`0600` atomic tenant-private profile
store and the Web create/view/plan/revoke-delete journey. It creates no Agent run,
provider call, ToolGateway route, effect intent/receipt or PluginExecutor request. The
four bounded operations are `profile.academic.create`, `profile.academic.view`,
`planner.generate` and `profile.academic.revoke_delete`; none has a direct
M10/Web-to-M72 fallback.

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

Until the active candidate passes exact Rust, real-browser, restart and independent
review gates, it is not accepted completion evidence. It also does **not** establish
production SSO/TLS, live source ingestion, arbitrary third-party Plugin execution,
backup-erasure proof or enrollment/registration effects. This slice does not make
Opportunity Graph the sole flagship or move it ahead of the three-Plugin MVP
requirement.

Future research, competition, lecture and scholarship packs must reuse the same trust/profile semantics or obtain a new ADR for a materially different ontology.
