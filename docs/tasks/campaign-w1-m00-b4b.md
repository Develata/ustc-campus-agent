# Campaign W1 — M00-B4b control-evidence slice receipt

- Stage: `IMPLEMENTED_BOUNDED_SLICE`
- Base HEAD: `0aafa5fd583322dd40e28e8af732ca53fd93d3be`
- Base tree: `fce6c23cf2fb8528cfd9fa7a4dcbdaa2147827e3`
- Accepted pre-edit taskbook: `/opt/data/tmp/uca-m00-b4b-control-evidence-taskbook-r2.md`
- Taskbook SHA-256: `6cecbb47c480f3c01b35e0cefe122ec724d806618f1f668a9ab7832e7c39e2c2`
- Owning contract: [`platform-control-evidence/v0`](../contracts/platform-control-evidence.md)
- Acceptance: `AUTH-022`

## Retained output

- exact redacted session-event and admitted-request projections;
- closed 36-code error projection over lifecycle/admission/repository/boundary failures;
- data-only Serde with no conversion into authority carriers;
- least-authority control-evidence read/append-once ports;
- deterministic bounded fake with unavailable/corrupt/limit/idempotent/conflict/atomic behavior;
- exact six-test target plus rustdoc compile-fail negative API proofs;
- repository checker, mutation, source-inventory and current-truth projection closure.

## Product consequence

B4a session-port and B4b control-evidence now complete the bounded typed interface/fake scope of M00-B4. B5 may consume these carriers for one administrator publication composition without inventing a second error/audit vocabulary.

## Preserved non-claims

This slice adds no production evidence store, no product/evidence transaction, no M10 administrator route, no Affairs publication, no SSO/CAS and no app/HTTP/Web/CLI behavior. `PlatformControlEvent` and `PlatformControlError` remain data-only; Serde or fake append is not authority or durability.

## Verification binding

```text
python3 scripts/check_repo_contracts.py
cargo test --locked -p ustc-campus-agent-core --test platform_control_evidence
cargo test --locked -p ustc-campus-agent-core --doc control_evidence
cargo test --locked -p ustc-campus-agent-core --test platform_identity
cargo clippy --locked -p ustc-campus-agent-core --all-targets --all-features -- -D warnings
```

Final completion additionally requires exact checker shards, full workspace test/Clippy/doctests, paired exact-source review, scoped staging/secret scan and one local commit. No push, PR, merge, tag, release or deployment is authorized by this receipt.
