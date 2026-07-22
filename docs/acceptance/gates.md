# Acceptance gates

## PR gate

Repository CI runs:

```bash
cargo fmt --all -- --check
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo test --locked --all-targets --all-features
python3 -m unittest discover -s scripts/tests -p 'test_*.py'
python3 scripts/check_repo_contracts.py
git diff --check
```

For a local docs-only iteration, `python3 scripts/check_repo_contracts.py` and `git diff --check` are the minimum quick gate; final PR readiness still uses the complete CI command set.

## Core demo gate

Every `matrix.tsv` row whose `gate` contains `core-demo` must pass for the claimed demo scope. A feature can be omitted from a smaller intermediate demonstration only if the claim and gate profile are explicitly narrowed; the full three-Plugin competition demo cannot reinterpret planned as pass.

## Release gate

Release requires all release-bound rows plus artifact/build/restore/read-back evidence appropriate to the delivery surface. Local test/build success is not remote release success.

## Public gate

Public visibility and Pages/download publication additionally require `public-readiness.md`, license/notice, full reachable-history audit, fixture/source permission, disclaimer, browser and remote delivery-surface verification.

## Evidence states

`matrix.tsv` is the active gate registry. `platform-baseline.md` retains long-horizon case IDs and planned assertions; catalog-only presence does not make a case current or required for the present competition slice. Activating one requires an owning plan/feature/contract projection and a row in `matrix.tsv`.

- `implemented` in the matrix means the named binding exists and currently passes the owning contract.
- `planned`, skipped, unavailable and not-run are non-pass.
- A manual binding needs an identified reviewer and evidence artifact before it can pass.
- Evidence from a stale pre-fix worktree does not certify the final staged/committed state.
