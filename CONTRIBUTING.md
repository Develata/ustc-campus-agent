# Contributing

This private repository uses [`docs/guides/contributing.md`](docs/guides/contributing.md) as the detailed collaboration contract. Also read [`AGENTS.md`](AGENTS.md), [`docs/AGENTS.md`](docs/AGENTS.md) and the matching row in [`docs/coverage-matrix.md`](docs/coverage-matrix.md).

## Quick workflow

1. Bind the slice to a task/ADR/contract/acceptance case.
2. Name one owner and reviewer; declare touched paths and non-goals.
3. Use `feat/<topic>`, `fix/<topic>`, `docs/<topic>`, `chore/<topic>` or disposable `spike/<topic>`.
4. Keep one semantic intent per PR and stage exact paths only.
5. Run the owning gates and include real output/not-run state.
6. Request review from the owner of the touched contract.
7. Do not push, tag, publish or change visibility without explicit Develata approval.

## Commit style

Prefer concise conventional-style commits, for example:

```text
docs: align Market authority layers
feat: add package manifest validator
test: cover Course Planning hard-constraint gate
```

## PR gate

The authoritative PR-gate command set and evidence-state rules live in [`docs/acceptance/gates.md`](docs/acceptance/gates.md). Run its complete current PR gate before requesting review, and report every command as pass, fail or honestly not run.
