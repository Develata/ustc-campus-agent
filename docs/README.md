# Documentation

USTC Campus Agent uses a proportionate docs-as-code structure: detailed authority and execution planning without mixing blueprint, product behavior, proof and historical rationale.

## Start here

1. [`plan/00-engineering-constitution.md`](plan/00-engineering-constitution.md)
2. [`plan/01-terminology.md`](plan/01-terminology.md)
3. [`overview/architecture.md`](overview/architecture.md)
4. [`coverage-matrix.md`](coverage-matrix.md)
5. the matching plan/feature/contract/acceptance documents

## Structure

| Path | Answers |
|---|---|
| [`plan/`](plan/) | How is the system engineered, and who owns authority/failure/recovery? |
| [`features/`](features/) | What does the user see and what is the honest journey/status? |
| [`contracts/`](contracts/) | What exact harness/runtime, Agent–Plugin tool boundary, multi-client shell, schemas, CLI, interfaces, permissions and data models are exposed? |
| [`acceptance/`](acceptance/) | What is active now, and which stable long-horizon proof cases must be retained for future scope? |
| [`overview/`](overview/) | How do the layers fit together? |
| [`tasks/`](tasks/) | In what dependency order is approved work delivered? |
| [`guides/`](guides/) | How do contributors develop, review and prepare future publication? |
| [`adr/`](adr/) | Why were major architecture decisions made? |

[`coverage-matrix.md`](coverage-matrix.md) maps the live layers. [`AGENTS.md`](AGENTS.md) defines documentation authority and editing discipline.

## Deliberate exclusions

This tree does not retain raw discovery workspaces, rejected proposal dumps, personal infrastructure/backup procedures or empty speculative registry/report directories. Useful accepted semantics are carried by the current plans, features, contracts, acceptance rows and ADRs; Git history preserves prior tracked revisions.
