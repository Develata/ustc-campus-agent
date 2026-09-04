# Contributing

This public repository uses [`docs/guides/contributing.md`](docs/guides/contributing.md) as the detailed collaboration contract and is licensed under the [`MIT License`](LICENSE.md). Unless explicitly stated otherwise, contributions submitted to this repository are licensed under the same MIT License. Also read [`AGENTS.md`](AGENTS.md), [`docs/AGENTS.md`](docs/AGENTS.md) and the matching row in [`docs/coverage-matrix.md`](docs/coverage-matrix.md).

## Quick workflow

1. Bind the slice to a task/ADR/contract/acceptance case.
2. Name one owner and reviewer; declare touched paths and non-goals.
3. Use `feat/<topic>`, `fix/<topic>`, `docs/<topic>`, `chore/<topic>` or disposable `spike/<topic>`.
4. Keep one semantic intent per PR and stage exact paths only.
5. Run the owning gates and include real output/not-run state.
6. Request review from the owner of the touched contract.
7. Perform remote operations only under the operation-specific or active-campaign Develata authorization defined by [`docs/tasks/00-module-work-policy.md`](docs/tasks/00-module-work-policy.md) §3; tags, releases, publication and visibility remain operation-specific unless a grant explicitly names them.

## Commit style

Prefer concise conventional-style commits, for example:

```text
docs: align Market authority layers
feat: add package manifest validator
test: cover Course Planning hard-constraint gate
```

## PR gate

The authoritative PR-gate command set and evidence-state rules live in [`docs/acceptance/gates.md`](docs/acceptance/gates.md). Run its complete current PR gate before requesting review, and report every command as pass, fail or honestly not run.
