# Contributing

This repository is currently private. Contributors should follow [`docs/collaboration/pr-contract.md`](docs/collaboration/pr-contract.md) and [`AGENTS.md`](AGENTS.md).

## Quick workflow

1. Create or pick an issue/task ID.
2. Create a branch: `feat/<topic>`, `fix/<topic>`, `docs/<topic>`, or `chore/<topic>`.
3. Keep one semantic intent per PR.
4. Run the relevant gates and paste evidence into the PR.
5. Request review from the owner of the touched contract.

## Commit style

Prefer concise conventional-style commits:

```text
docs: define market authority boundary
feat: add package manifest validator skeleton
test: cover course-planning hard-constraint gate
```

## Required checks

```bash
cargo fmt --all -- --check
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo test --locked --all-targets --all-features
python3 scripts/check_repo_contracts.py
```
