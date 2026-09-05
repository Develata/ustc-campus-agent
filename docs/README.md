# Documentation

USTC Campus Agent uses a proportionate docs-as-code structure: detailed authority and execution planning without mixing blueprint, product behavior, proof and historical rationale.

Project-authored software and documentation in this public repository are licensed under the [`MIT License`](../LICENSE.md). Third-party content and campus data retain their separate rights and source-permission requirements.

## Start here

For the runnable competition slice, begin with [`features/06-mvp-core-capabilities.md`](features/06-mvp-core-capabilities.md) and [`contracts/agent-chat.md`](contracts/agent-chat.md). For the installable Android demo artifact, use [`features/07-android-demo-client.md`](features/07-android-demo-client.md) and [`guides/android-demo.md`](guides/android-demo.md).

For the bounded discussion-to-document reconciliation, see [讨论决策与文档落点](guides/discussion-to-docs.md): it separates accepted plans, retained implementation, open-PR work and deferred scope without claiming a complete chat archive. For the delivered Windows launcher correction and outstanding real-host acceptance, use [R3.1 交付身份与复验](guides/r31-delivery-and-verification.md). A documentation update does not rebuild or relabel the fixed R3.1 binaries.

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
| [`contracts/`](contracts/) | What exact harness/runtime, Agent–Plugin tool boundary, typed client core/peer shells, schemas, CLI, interfaces, permissions and data models are exposed? |
| [`acceptance/`](acceptance/) | What is active now, and which stable long-horizon proof cases must be retained for future scope? |
| [`overview/`](overview/) | How do the layers fit together? |
| [`tasks/`](tasks/) | How are large modules split, committed, independently accepted and assembled? |
| [`guides/`](guides/) | How do contributors develop, review and prepare future publication? |
| [`adr/`](adr/) | Why were major architecture decisions made? |
| [`design/`](design/) | How are UI/presentation packets proposed, reviewed and superseded? (subordinate layer; Reviewed is not implementation or behavior authority) |

[`coverage-matrix.md`](coverage-matrix.md) maps the live layers. [`AGENTS.md`](AGENTS.md) defines documentation authority and editing discipline.

## Deliberate exclusions

This tree does not retain raw discovery workspaces, rejected proposal dumps, personal infrastructure/backup procedures or empty speculative registry/report directories. Useful accepted semantics are carried by the current plans, features, contracts, acceptance rows and ADRs; Git history preserves prior tracked revisions.
