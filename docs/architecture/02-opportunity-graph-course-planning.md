# Campus Opportunity Graph and Course Planning

Campus Opportunity Graph is one of three default first-party Plugins. Course Planning is a vertical slice inside it, not the platform's sole product spine.

The current deterministic planner is an out-of-order bounded offline spike. It proves typed fixture validation and hard-constraint planning, but not Market installation, grants, Agent discovery, source ingestion, or consent-aware profile integration. The main implementation sequence remains defined by [`ADR-0006`](../decisions/ADR-0006-three-default-first-party-plugins.md).

## Minimal ontology

| Type | Course Planning projection |
|---|---|
| `OpportunityNode` | Course offering |
| `RequirementNode` | Curriculum requirement/elective group |
| `DependencyEdge` | prerequisite / recommended-before |
| `CoverageEdge` | course satisfies requirement |
| `ConflictEdge` | time or rule conflict |
| `TemporalWindow` | term, effective date, registration window |
| `EvidenceSignal` | official fact, secondary mirror, community link-out, uncertainty |
| `ProfileFact` | user-provided academic snapshot/preference |

## Source authority order

```text
official catalog snapshot/API
> reviewed official notice/department source
> iCourse program mirror
> community signal/link-out
> model inference
```

Community review signal can influence soft ranking only. It cannot override official requirements, prerequisites, credit rules, or offering conflicts.

## Planner contract

- Rust hard-constraint checker decides legality.
- LLM explains and asks follow-up questions; it does not create legal courses.
- If LLM explanation adds or substitutes a course not approved by planner output, the consistency gate rejects the explanation and falls back to planner output.
- Over-budget planning requests are narrowed or decomposed; no last-minute heavy solver adoption in MVP.
