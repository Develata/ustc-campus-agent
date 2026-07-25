# Documentation

USTC Campus Agent uses a proportionate docs-as-code structure: detailed authority and execution planning without mixing blueprint, product behavior, proof and historical rationale.

## Start here

1. [`plan/00-engineering-constitution.md`](plan/00-engineering-constitution.md)
2. [`plan/01-terminology.md`](plan/01-terminology.md)
3. [`plan/modules/00-module-map.md`](plan/modules/00-module-map.md)
4. [`contracts/module-boundaries.md`](contracts/module-boundaries.md)
5. [`tasks/00-module-work-policy.md`](tasks/00-module-work-policy.md)
6. [`overview/architecture.md`](overview/architecture.md)
7. [`coverage-matrix.md`](coverage-matrix.md)
8. the matching module plan/feature/contract/acceptance documents

## Structure

| Path | Answers |
|---|---|
| [`plan/`](plan/) | How is the system engineered, which large modules exist, and who owns authority/failure/recovery? |
| [`features/`](features/) | What does the user see and what is the honest journey/status? |
| [`contracts/`](contracts/) | What exact harness/runtime, Agent–Plugin tool boundary, multi-client shell, schemas, CLI, interfaces, permissions and data models are exposed? |
| [`acceptance/`](acceptance/) | What is active now, and which stable long-horizon proof cases must be retained for future scope? |
| [`overview/`](overview/) | How do the layers fit together? |
| [`tasks/`](tasks/) | How are large modules split, committed, independently accepted and assembled? |
| [`guides/`](guides/) | How do contributors develop, review and prepare future publication? |
| [`adr/`](adr/) | Why were major architecture decisions made? |

[`coverage-matrix.md`](coverage-matrix.md) maps the live layers. [`AGENTS.md`](AGENTS.md) defines documentation authority and editing discipline.

## Deliberate exclusions

This tree does not retain raw discovery workspaces, rejected proposal dumps, personal infrastructure/backup procedures or empty speculative registry/report directories. Useful accepted semantics are carried by the current plans, features, contracts, acceptance rows and ADRs; Git history preserves prior tracked revisions.
