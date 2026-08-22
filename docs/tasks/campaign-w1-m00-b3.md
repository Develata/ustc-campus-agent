# W1 M00-B3 request-context readiness

## Authority

- `Campaign ID`: `USTC-MODULES-2026-07-W1`
- `Lane`: `M00-B3`
- `Grant carrier`: [`01-execution-roadmap.md`](01-execution-roadmap.md#active-autonomous-module-campaign-authorization)
- `Mode`: the original proposal-only campaign row was superseded by current-session Develata acceptance of exact M00-v12/M10-v17 semantics and an explicit isolated-worktree implementation scope

## Mutable campaign state

- `Status`: `queued`
- `Bound source commit`: `3bd622d98f1e11f8d39ba88334ac5fc0b737c301`
- `Repair round`: `0`
- `Current blocker identity`: `none`
- `Stop reason`: `none`
- `Last transition evidence`: `M00-v12-reaccepted-and-bounded-kernel-gates-pass`
- `Next allowed mutation`: production B4 vendor or B5 M10 composition only under a separately accepted contract-bound slice

## Implemented bounded output

[`platform-request-context/v0`](../contracts/platform-request-context.md) and `AUTH-013` are implemented for the bounded platform-core kernel:

- closed `Public | Authenticated` admitted actor with no synthetic public identity;
- exact authenticated session binding through the production session predicate;
- immutable request-scoped descriptor projection;
- current policy/session/capability checks;
- fenced idempotency reserve/retrieve/finalize outcomes;
- complete scalar admitted dispositions and validating persistence promotion;
- fourteen payload-preserving rejection classes and closed five-way result;
- exactly 64 integration tests plus nonzero compile-fail doctests;
- fail-closed checker registration and mutation evidence.

## Honest non-goals

M00 remains `partial-evidence`. This taskbook does not claim production B4 database/auth/policy/capability/clock/idempotency vendors, real crash/reopen/CAS evidence, B5 M10-v17 runtime composition, downstream domain E2E, source-control publication, release, or deployment.

## Required evidence

```bash
cargo test --locked -p ustc-campus-agent-core --test platform_request_context
cargo test --locked -p ustc-campus-agent-core --doc request_context
python3 scripts/check_repo_contracts.py
python3 -m unittest scripts.tests.test_check_repo_contracts
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
```
