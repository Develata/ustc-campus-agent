# P1 source/revision readiness and M60-B1 implementation proposal

## Mutable state

- `Stage`: `P1_0_CLOSURE_CANDIDATE`
- `P1-0 review`: `GO`; exact-candidate receipt recorded below
- `P1-0 local commit`: this scoped P1-0 commit
- `P1-1 implementation`: pending
- `P1-1 review`: pending
- `P1-1 local commit`: pending
- `Source candidate`: `ustc-teach-calendar-fall` remains `Proposed`
- `Remote shipping`: feature-branch push authorized after final local validation; PR/merge/tag/release remain forbidden

## Authority receipt

Develata selected the corrected posture on 2026-08-08:

> P1-0 independent GO 后，自动本地实现并提交 P1-1 source-registry；不 push。

On 2026-08-08, after P1-0 reached exact-candidate GO, Develata additionally authorized pushing the feature branch after final local validation. This later instruction changes only remote-operation authority: it does not authorize PR, merge, tag, release, deployment, source approval or retrieval.

This receipt authorizes the finite local workflow in the exact packet below. It does not self-authorize broader module work, source approval, network retrieval, raw-fixture publication, push, PR, merge, tag, release or deployment; feature-branch push authority comes only from the later instruction recorded above.

## Exact packet identity

- `Source commit`: `2f4de29032560ff3e13d9994b33a3aff14243f44`
- `Source tree`: `53e266c47fdb07d50a734faa24bb11ac4bc5527d`
- `Packet digest`: `sha256:11529705aca4e19ae52bde8ec5a69571c2c3cc6d1157057ac77dd69f78aa65f9` over `8479` bytes beginning immediately after the `BEGIN` marker newline and ending immediately before the `END` marker token, including the final packet newline

<!-- P1_SOURCE_REVISION_EXACT_PACKET:BEGIN -->
## 1. Objective

Create one construction-ready `source-import/v0` contract and fail-closed P1-0 governance carrier, obtain independent review GO, make one scoped local P1-0 commit, then implement and independently close only `M60-B1 source-registry` as a second scoped local commit.

The product intent is to establish the source/revision foundation needed by ChangeRadar without pretending that retrieval, parsing, source revision, semantic diff, accepted baseline or product feed already exists.

## 2. Frozen source

All P1-0 and P1-1 work is based on:

```text
commit 2f4de29032560ff3e13d9994b33a3aff14243f44
tree   53e266c47fdb07d50a734faa24bb11ac4bc5527d
branch origin/main at authorization read-back
```

A remote-main move does not silently rebase this operation. A future continuation may explicitly reconcile a new main object before any shared-state operation, but this packet authorizes no shared-state operation.

## 3. P1-0 contract/readiness scope

P1-0 may edit exactly:

```text
docs/contracts/source-import.md
docs/plan/modules/70-campus-trust-source-pipeline.md
docs/tasks/01-execution-roadmap.md
docs/tasks/p1-source-revision-readiness-proposal.md
scripts/check_repo_contracts.py
scripts/tests/test_check_repo_contracts.py
```

P1-0 must:

1. make `docs/contracts/source-import.md` construction-ready for B1 and explicit about later M60 phases;
2. preserve M60 as `planned`, M70 as `design-only` and every acceptance row's current status;
3. bind this exact packet, source object, six-path scope, no-source-approval claim and no-remote-shipping claim in the always-run checker;
4. add positive and one-axis negative mutation tests for packet digest, source commit/tree, duplicate/missing path, forbidden extra path, status drift, source approval drift and remote-shipping drift;
5. keep the proposed calendar HTML outside Git and name only URLs, hashes/observations and conservative policy metadata;
6. run aggregate checker, the complete Python checker-unit suite and `git diff --check` from a fresh process after the candidate is frozen;
7. obtain an independent exact-candidate review across architecture/API, checker/mutations and docs/status/source-policy axes.

P1-0 may create one local commit only after GO. It may not edit Rust, Cargo, acceptance statuses, CI, root governance, campaign grant bytes, raw fixtures or external systems.

## 4. P1-1 exact implementation scope

After P1-0 independent GO and local commit, P1-1 may edit exactly:

```text
crates/platform-core/src/lib.rs
crates/platform-core/src/source_registry.rs
crates/platform-core/tests/source_registry.rs
docs/acceptance/matrix.tsv
docs/acceptance/platform-baseline.md
docs/contracts/source-import.md
docs/plan/modules/70-campus-trust-source-pipeline.md
docs/tasks/01-execution-roadmap.md
docs/tasks/p1-source-revision-readiness-proposal.md
scripts/check_repo_contracts.py
scripts/tests/test_check_repo_contracts.py
```

The implementation is the exact B1 public surface and behavior specified by `docs/contracts/source-import.md` §§3–7 and §§12–13. It adds no dependency, adapter, network access, DNS/redirect logic, clock, persistence, parser, raw snapshot, normalized snapshot, source revision, diff, baseline or product feed.

`crates/platform-core/src/lib.rs` may add exactly one module declaration:

```rust
pub mod source_registry;
```

It must not modify the existing crate-root `SourceAuthority` declaration or order.

## 5. P1-1 acceptance/status scope

P1-1 may promote only matrix row `SRC-001`, and only from `planned` to `implemented`, after its exact checker+Rust binding passes.

The implemented claim is bounded to stable source identity, owner, exact URL, retrieval-policy value, proposed/approved review state and pure registry transitions. It proves no live source permission or retrieval enforcement.

These rows remain unchanged:

```text
SRC-010 planned
SRC-011 planned
SRC-012 planned
```

Catalog-only `SRC-002` through `SRC-009` and `SRC-013` remain catalog-only and non-admitted. M60 remains `planned` because one bounded B1 slice is not a reviewed source/revision/baseline. M70 remains `design-only`.

## 6. Concrete source posture

`ustc-teach-calendar-fall` is a proposed source family only. P1-0/P1-1 may record:

```text
owner: 中国科学技术大学教务处 / www.teach.ustc.edu.cn
2025: https://www.teach.ustc.edu.cn/calendar/19081.html
2026: https://www.teach.ustc.edu.cn/calendar/20135.html
index: https://www.teach.ustc.edu.cn/category/calendar
robots: https://www.teach.ustc.edu.cn/robots.txt
minimum interval: 21600 seconds
maximum response: 131072 bytes
retention: internal evidence; normalized facts + links in product output
```

It must remain `Proposed`; no concrete row is instantiated as `Approved` in production data, fixtures or docs. Public accessibility and `robots.txt` are not treated as republication permission. Raw HTML, headers and local evidence archives remain outside Git.

Synthetic Rust tests may construct an approval receipt to prove generic B1 state transitions. Such tests must use synthetic domains/IDs and must not encode the USTC candidate as approved.

## 7. B1 error and effect boundary

Every rejected B1 value returns a typed non-echoing `SourceValueError`. Every failed registry transition preserves the entire registry and the first accepted definition/receipt. The registry performs no I/O and receives no framework/database/network/clock type.

A `SourceReviewReceipt` is only a structurally complete set of evidence references. The application boundary remains responsible for authenticating/authorizing the reviewer and binding real evidence; the pure B1 kernel cannot convert arbitrary text into factual operator approval.

## 8. P1-1 evidence and gate chain

The exact source-registry integration test must execute positive and negative cases for:

- `SourceId`, `SourceReviewEvidenceId`, `SourceUrl`, owner and policy bounds/precedence;
- validating Serde and no unchecked/default/mutable construction path;
- no rejected-input echo in `Display`/`Debug`/error chains;
- proposed-by-default and no already-approved constructor;
- duplicate, missing, proposed and already-approved registry failures;
- first definition/first receipt preservation;
- failed-transition atomicity;
- exact public API/module/dependency closure;
- zero concrete approved USTC source.

Required gates:

```text
python3 scripts/check_repo_contracts.py --ci
python3 -m unittest scripts.tests.test_check_repo_contracts.SourceRegistryContractTests
python3 -m unittest discover -s scripts/tests -p 'test_*.py'
cargo test --locked -p ustc-campus-agent-core --test source_registry
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace --all-targets
cargo test --locked --workspace --doc
git diff --check
```

A command exiting zero with zero intended tests is not evidence. The checker must bind the integration-test target, exact test inventory/attributes and registered acceptance command.

## 9. Review/reconcile rule

Every candidate identity changes after any repair. Old verdicts become stale. Independent review findings are blocking only when they expose current production/contract/public-API/external-behavior/CI/scope failure or a reproducible false green for a mandatory accepted claim. A finding that requires deliberate synchronized weakening of checker/tests/proof carriers without exposing current B1 behavior is recorded as assurance backlog under the product-first stop rule.

Hermes independently reproduces every proposed blocker before changing the candidate.

## 10. Commit and shipping rule

P1-0 and P1-1 each receive at most one semantic local commit after exact-path staging and cached-diff review. Foreign dirty/untracked work is not staged or cleaned. `git add -A`, broad reset/restore/clean/stash and history rewriting are forbidden.

This packet authorizes no push, PR, merge, tag, release or deployment. Those remain explicit future decisions.

## 11. Stop conditions

Stop and return to Develata before mutation if:

- a required path lies outside the P1-0/P1-1 allowlists;
- B1 cannot be implemented without a new dependency or changing `SourceAuthority`;
- the next step would approve/fetch a real source or commit raw source bytes;
- a reviewer requires an authority/public-surface change not represented here;
- a mandatory gate cannot run in the declared environment;
- a remote/shared-state action becomes necessary.
<!-- P1_SOURCE_REVISION_EXACT_PACKET:END -->

## Review receipts

### P1-0 exact-candidate GO receipt

- `Reviewer`: Dongfengyun OMO lead over `USTC/glm-5.2-107`; one logical independent lane
- `Object`: manifest `sha256:32e7f67c16b5b23e39435d45a40b662e6f6fa719a3c4d28d97ff7d5f9b82b645`; archive `sha256:989b126f594fa048c1dddf65083f36581e9b77fa81ca7bd95891f15703076d26`; packet `sha256:11529705aca4e19ae52bde8ec5a69571c2c3cc6d1157057ac77dd69f78aa65f9`; source commit `2f4de29032560ff3e13d9994b33a3aff14243f44`
- `Verdict`: `GO`
- `Blockers`: `[]`
- `P1-1 recommendation`: `ELIGIBLE`
- `Report`: `sha256:d4f9cb7706d4f7b114c0e40541deb041dbe24fb6df57356d70be04540b5e4bae`
- `Custody`: pre/post verification both bound packet `11529705aca4e19ae52bde8ec5a69571c2c3cc6d1157057ac77dd69f78aa65f9` and contract candidate `688f2e6c957ab34c763c8884bf56100bf2f4a22a21a62550abead11482a7f979`
- `Closure`: accept unchanged §1–13 semantics as `source-import/v0` for bounded P1-1 B1 implementation; keep the real USTC source family `Proposed`, M60 `planned`, M70 `design-only`, and every acceptance row unchanged in P1-0
