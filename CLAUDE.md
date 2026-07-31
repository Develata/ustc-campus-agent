# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Read first

`AGENTS.md` (root) is the governing file for all work here; `docs/AGENTS.md` governs documentation, `docs/plan/AGENTS.md` the blueprint contract. This file summarizes only what is expensive to reconstruct by reading. When this file and `AGENTS.md`/`docs/` disagree, they win.

## Commands

Authoritative current gates are `.github/workflows/ci.yml` and `docs/acceptance/gates.md`; the local command set below mirrors their required checks. Check those carriers before trusting this summary.

```bash
cargo fmt --all -- --check
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo test --locked --all-targets --all-features
cargo test --locked --all-features --doc
python3 -m unittest discover -s scripts/tests -p 'test_*.py'
python3 scripts/check_repo_contracts.py
git diff --check
```

Docs-only quick gate: `python3 scripts/check_repo_contracts.py` and `git diff --check`.

Single Rust test — this exact shape is what acceptance bindings in `docs/acceptance/matrix.tsv` use, so copy it verbatim from the row you are satisfying:

```bash
cargo test --locked -p ustc-campus-agent-core --test invocation_resolution \
  valid_projection_is_deterministic_and_turn_bound -- --exact
```

`--all-targets` does **not** run doctests, so the `compile_fail` API-immutability proofs that live in rustdoc blocks are invisible to the ordinary test command. They are covered by the separate `cargo test --locked --all-features --doc` line above, which is an unconditional, blocking `Doc tests` step in `jobs.rust.steps` on `pull_request`. `check_rust_doctest_gate` pins that step's carrier chain by exact command line in both `ci.yml` and `gates.md`, and rejects `if:`, `continue-on-error:`, a moved step, a commented command or a multiline `run: |` carrier — so restructuring that step fails the checker until the contract is migrated. Scoped form for local iteration: `cargo test --locked -p <pkg> --doc`.

Single Python checker test: `python3 -m unittest scripts.tests.test_check_repo_contracts.ModuleRegistryContractTests.test_current_module_registry_passes`

Smokes:

```bash
cargo run --locked -p ustc-agentctl -- doctor
cargo run --locked -p ustc-agentctl -- market validate
cargo run --locked -p ustc-agentctl -- course plan --fixture market/fixtures/course-planning/minimal-v0.json --format json
cargo run --locked -p ustc-agentd -- --version
```

Rust builds eat disk. `docs/guides/development.md` recommends `export CARGO_TARGET_DIR=/tmp/hermes-cargo-target` and checking `df -h .` first. Toolchain is pinned to 1.97.1 (`rust-toolchain.toml`), edition 2024.

Directory names do not match package names — use these with `-p`:

| Directory | Package |
|---|---|
| `crates/platform-core` | `ustc-campus-agent-core` |
| `crates/agent-runtime` | `ustc-campus-agent-runtime` |
| `crates/agent-tool-protocol` | `ustc-agent-tool-protocol` |
| `crates/course-planning` | `ustc-campus-agent-course-planning` |
| `crates/adapters` | `ustc-campus-agent-adapters` |
| `apps/ustc-agentd`, `apps/ustc-agentctl` | same name |

## Architecture

Four call layers plus an object plane (`AGENTS.md` §Repository architecture and authority): interaction shell (future Dioxus client / operator CLI) → application interface (M10 ingress in `ustc-agentd`) → flow coordination (M00-admitted services, harness runs, use cases) → execution domain (resolvers, gateway, executors, source pipeline). The object plane names durable state; it is not a fifth caller.

The system is frozen as **13 independently owned large modules**. `docs/plan/modules/00-module-map.md` is the registry; `docs/contracts/module-boundaries.md` says what may cross each boundary. Note the numbering offset — module IDs and blueprint filenames differ by one decade (`M00` → `modules/10-platform-control-identity.md`, `M10` → `modules/20-application-api-host.md`, … `M90` → `modules/90-infrastructure-operations.md`). The authoritative mapping is `MODULE_BLUEPRINTS` in `scripts/check_repo_contracts.py`.

Crate responsibilities:

- `platform-core` — two different things under one crate. `invocation.rs` is the pure, no-I/O invocation resolver: **M20 evidence**. `identity.rs` is **M00 evidence**: batch `M00-B1 identity-types` is implemented under `docs/contracts/platform-identity.md` (`platform-identity/v0`), with `AUTH-011/012/014/015/016` at `implemented`. The two share one crate with no compiler-enforced boundary, so when adding identity code, check which module owns the value before touching a neighbouring type. The boundary that *is* enforced: `invocation.rs` re-exports `identity::{TenantId, UserId}` rather than defining its own, while `PolicySnapshotId` and every other invocation ID stay M20-owned with the older 256-byte `InvalidValue` grammar — `check_platform_identity_implementation` rejects any drift in either direction. `session.rs` is also **M00 evidence**: batch `M00-B2 session-domain` is implemented under `docs/contracts/platform-session.md` (`platform-session/v0`), with `AUTH-017/018/019/020` at `implemented`. Two of its acceptance rows carry a second exact leg against the library target, because the `revision == u64::MAX` guards they cover are unreachable from an integration test; those fixtures live in a private `#[cfg(test)]` module inside `session.rs` and are registered in both checker carriers. `M00-B3..B5` (request-context, ports-and-fakes, api-admission-integration) are still planned, so M00 as a whole is `partial-evidence`.
- `agent-runtime` — framework-neutral `RunSpec`/phase/replay/effect-ordering kernel (M30 evidence). Its dependency confinement is **mechanically enforced** by `check_agent_plugin_dependency_direction` and acceptance row `AGENT-017`: no Market, Plugin, component or adapter dependency may appear, and cross-boundary proof lives at the composition root instead.
- `agent-tool-protocol` — `agent-tool-protocol/v0` values: canonical schemas/arguments, digests, frozen toolset view, correlated call, typed result. Owns no package, grant, executor or Agent state authority.
- `adapters` — replaceable provider/tool/executor adapters; never authority.
- `course-planning` — bounded offline planner spike (M72 evidence).
- `apps/ustc-agentd` — the only composition root; cross-module wiring and ordering tests belong here (`apps/ustc-agentd/tests/`), never inside a domain crate.

The single extension seam is `Agent ↔ ToolGateway ↔ PluginExecutor`. M30 and M40 depend on the shared protocol, not on each other; composition interleaves effect intent and receipt.

Existing Rust is **bounded evidence, not finished modules**. Before extending any of it, compare it against the owning blueprint and record `adopt | amend | retain as spike | remove` (`docs/tasks/01-execution-roadmap.md` §1).

## Docs are authority; code is a projection

`docs/plan/` and `docs/contracts/` own behavior. `docs/features/`, `docs/acceptance/`, `docs/tasks/` project it and cannot redefine it. `docs/coverage-matrix.md` maps blueprint ↔ contract ↔ feature ↔ acceptance. Follow the mandatory work loop in `AGENTS.md` (read constitution → terminology → owning plan → contracts/acceptance → smallest slice → gates → review → commit).

Acceptance discipline, which the checker partly enforces:

- `docs/acceptance/matrix.tsv` is the **only** active gate registry. `docs/acceptance/platform-baseline.md` is a long-horizon catalog; catalog presence alone makes nothing current.
- `planned`, skipped, unavailable and not-run are all non-pass. Do not promote a row from documentation alone.
- **No retained implementation starts before its module has exact active `planned` rows with future evidence bindings** (`docs/tasks/00-module-work-policy.md` §10). This gate blocks code, not docs.
- `docs/coverage-matrix.md` acceptance cells use only the machine-checked tokens `gap`, `active:<CASE-or-FAMILY-*>`, `long-horizon:<CASE-or-FAMILY-*>`.

`scripts/check_repo_contracts.py` cross-validates more than it looks: module ID/state-key agreement across module map, blueprint metadata and roadmap lane; coverage tokens against the real matrix and catalog; matrix header/gate/status vocabulary and duplicate IDs; catalog membership for active `AGENT`/`AUTH`/`FP`/`HARNESS`/`PKG`/`PROC`/`SRC` cases; Markdown links; secret patterns; first-party manifest ↔ Rust identity agreement; invocation fixture set and digests; the CI doctest-gate carrier chain; the S0 review ledger. Two traps: a new `docs/contracts/*.md` fails the run until it is added to `KEY_FILES`, and `scripts/tests/test_check_repo_contracts.py` mutates real doc strings as fixtures, so editing an anchored line can break a test that is not about your change.

## Rust conventions in this repo

- Validated ID newtypes over raw strings: private `String`, an inherent checked `parse(impl Into<String>) -> Result<Self, _>` as the only constructor, `as_str()`, derived `Debug/Clone/Eq/Ord/Hash`, no `Deref`, no unchecked construction. Existing ones are built by the `authority_id!` (platform-core) and `protocol_identity!` (agent-tool-protocol) macros; tests construct them through the local `parsed!` macro.
- Serde on ID newtypes depends on which contract owns the value — do not generalize from one:
  - the six M00 IDs under `platform-identity/v0` **require** validating exact-string Serde that delegates to each kind's inherent `parse`, so deserialization cannot bypass the grammar;
  - the legacy M20 invocation IDs stay under their existing contract, without Serde, until explicitly migrated;
  - either way the nominal/private backing, absence of unchecked construction, and `compile_fail` API proofs are the default safety shape.
- Serde on envelope structs always uses `#[serde(deny_unknown_fields)]`, plus `rename_all = "snake_case"` for enums.
- Errors are small `Copy` enums implementing `Display`/`Error` that render the variant only — they never echo rejected input.
- Determinism is a contract, not a preference: `BTreeMap`/`BTreeSet` in public shapes, explicit canonical byte encodings before hashing, domain-separated SHA-256 digests.
- `compile_fail` rustdoc blocks are the accepted mechanism for proving a public API cannot be mutated or misused.
- Workspace lints: `unsafe_code = "forbid"`, `dbg_macro`/`todo`/`unwrap_used` denied. In tests prefer `expect("fixture")` or a `let ... else { panic!() }`.

## Repository rules that bite

The engineering rules themselves live in `AGENTS.md` (§Engineering rules, §Slice completion) and `docs/tasks/00-module-work-policy.md` §3 — read them there, not here. The four that most often catch an agent out: exact-path staging only (never `git add -A`), remote `main` protected with push/merge requiring either operation-specific authorization or an active source-controlled campaign grant, no secrets or real personal data anywhere including fixtures and logs, and status reported as `implemented`/`planned`/`blocked`/`not-run` without inflation. A contract existing does not make the system operational.
