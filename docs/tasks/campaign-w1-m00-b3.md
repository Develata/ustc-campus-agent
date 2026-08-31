# W1 M00-B3 request-context readiness

## Authority

- `Campaign ID`: `USTC-MODULES-2026-07-W1`
- `Lane`: `M00-B3`
- `Grant carrier`: [`01-execution-roadmap.md`](01-execution-roadmap.md#active-autonomous-module-campaign-authorization)
- `Mode`: the original proposal-only campaign row was superseded by current-session Develata acceptance of exact M00-v12/M10-v17 semantics and an explicit isolated-worktree implementation scope

## Mutable campaign state

- `Status`: `completed`
- `Bound source commit`: `3bd622d98f1e11f8d39ba88334ac5fc0b737c301`
- `Bound source role`: `historical exact implementation input; not the terminal main integration receipt`
- `Terminal main commit`: `e6fcf2423a2b438708b2573f4616161cc95aa08e`
- `Repair round`: `0`
- `Current blocker identity`: `none`
- `Stop reason`: `none`
- `Last transition evidence`: `M00-v12-reaccepted-and-bounded-kernel-gates-pass; bounded request-context output later integrated by PR #55 reviewed head 1accc2c75c69a24c3f6dee54ecc9166eac38dd36; squash/main e6fcf2423a2b438708b2573f4616161cc95aa08e; reviewed-tree-equals-main-tree 348b85475d0ca5b7c4ea68a4942ce57a4ac09887; exact-main CI 33295591462=PASS; W1 M00-B3 terminal disposition completed with no readiness promotion`
- `Next allowed mutation`: `none within USTC-MODULES-2026-07-W1; a production B4 vendor or B5 M10 composition requires a separately accepted contract-bound slice`

## Implemented bounded output

[`platform-request-context/v0`](../contracts/platform-request-context.md) and `AUTH-013` are implemented for the bounded platform-core kernel:

- closed `Public | Authenticated` admitted actor with no synthetic public identity;
- exact authenticated session binding through the production session predicate;
- immutable request-scoped descriptor projection;
- current policy/session/capability checks;
- exact closed `PermissionClass` / `EffectClass` surfaces and fail-closed pair coherence;
- fenced idempotency reserve/retrieve/finalize outcomes;
- complete scalar admitted dispositions and validating persistence promotion;
- fourteen payload-preserving rejection classes and closed five-way result;
- exactly 65 integration tests plus nonzero compile-fail doctests;
- fail-closed checker registration and mutation evidence.

## Honest non-goals

M00 remains `partial-evidence`. This M00-B3 closure claims only the bounded request-context kernel integrated by PR #55; it does not claim production B4 database/auth/policy/capability/clock/idempotency vendors, broader M00 authority, release or deployment. Later downstream product E2E belongs to owning modules and is not promoted here as M00-B3 evidence.

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
