# M00-B4a session-port slice receipt

## Authority

- Parent architecture acceptance: `ACCEPT_EXACT_UCA_CURRENT_HEAD_ARCH_RATIFICATION_R5`
- Product-first B4 boundary: B4a `session-port` followed by B4b `control-evidence`
- Boundary receipt SHA-256: `cc50304a2fee4dcd924ffc3ad0972fef48cc7ba3f32049db8311d1f7e2f93af3`
- Reviewed implementation taskbook: R8, SHA-256 `cb9fa2564b0c1064e003d3b4d90e2abb75fa0e2b05735ec4e50309a5817074f7`

## Source binding

- Base commit: `1266ea63f36e44c3f4077749e94329c20933e6c6`
- Base tree: `8aa4232c1c4021c50ce6fef037dd357d2b9ba328`
- Retained implementation identity: the Git commit containing this receipt and `platform-session-port/v0`; its literal commit/tree read-back belongs in the external post-commit receipt because embedding a file's own commit/tree would be self-referential.

## Status

`implemented-bounded-b4a`

This is a non-dispatch slice receipt. It adds no autonomous-campaign taskbook or grant.

## Bounded output

- exact `session_port` public Rust carrier and deterministic fakes;
- replay-only `SessionHistory`;
- complete read/append/clock/credential-evidence port shapes;
- one app-private durable DemoReviewed current-session read/bootstrap vendor;
- explicit CLI/launcher state file `m00-sessions.json`;
- authenticated product calls read retained session authority;
- `ustc-agent` product-path E2E launch/restart uses the same explicit secure session store;
- `AUTH-021` bounded evidence.

## Honest non-goals

- no formal USTC SSO or public session lifecycle transport;
- no durable lifecycle append vendor;
- no B4b external redacted control evidence;
- no B5 administrator admission composition;
- no Affairs PROC-011 administrator import/review/publish path;
- no M10 DTO/Web/Market/Harness/Tool authority widening.

## Required evidence

- repository checker, complete checker unittest and checker shards;
- focused core/app Rust tests, doctests and Clippy;
- full workspace test/Clippy/doctest;
- exact-source paired post-edit review;
- scoped local commit read-back.

B4 remains incomplete until B4b `control-evidence` is retained and verified. B5 and PROC-011 remain planned.
