# Acceptance gates

## PR gate

- `cargo fmt --all -- --check`
- `cargo clippy --locked --all-targets --all-features -- -D warnings`
- `cargo test --locked --all-targets --all-features`
- `python3 scripts/check_repo_contracts.py`

## Core demo gate

Required cases from `matrix.tsv` with `gate` containing `core-demo` must pass. `skipped`, `unavailable`, and `not-run` are not pass states for required cases.

## Release/public gate

Public visibility and releases require separate public-readiness, license, fixture scrub, source permission, Pages, and artifact verification gates. Do not infer release readiness from PR green status.
