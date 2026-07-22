# Campus Opportunity Graph and Course Planning

The flagship plugin is Campus Opportunity Graph. Course Planning is its first vertical slice.

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
