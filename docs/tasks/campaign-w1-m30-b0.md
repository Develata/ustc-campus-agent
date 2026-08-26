# W1 M30-B0 existing-kernel audit

## Authority

- `Campaign ID`: `USTC-MODULES-2026-07-W1`
- `Lane`: `M30-B0`
- `Grant carrier`: [`01-execution-roadmap.md`](01-execution-roadmap.md#active-autonomous-module-campaign-authorization)
- `Mode`: audit-only; no Agent/Harness lifecycle or runtime state-machine change

## Mutable campaign state

- `Status`: `active`
- `Bound source commit`: `f62018326bbc180ec79cc04ec3dde214d7007094`
- `Bound source tree`: `a9c5529f9deada0992c26a07c5d6e4a3fcdc5585`
- `Worker branch`: `campaign/w1-m30-b0-audit-v2`
- `Repair round`: `2`
- `Current blocker identity`: `none`
- `Stop reason`: `none`
- `Last transition evidence`: `PR #48 exact head bba936bda8bf217c4ebbd54d20dfa7631a814e97 passed CI run 32966961512; late GitHub findings R2-F9/R2-F10 were source-confirmed and repaired; final focused gates/review required before amend`
- `Next allowed mutation`: `run focused full checker gates and exact-delta review on F9/F10, amend the same semantic commit, force-with-lease the PR head, resolve review threads, and require replacement exact-head CI before merge`

## Output contract

Audit current M30 retained evidence against the blueprint and contracts, then record exactly one `adopt | amend | retain as spike | remove` disposition. Auto-merge is admitted only when the result changes no lifecycle or runtime state-machine behavior and does not promote readiness.

## Required evidence

- exact source commit and clean checkout receipt;
- matrix-planned `HARNESS-001` and `HARNESS-003` plus catalog-only, non-admitted `HARNESS-002` evidence reconciliation;
- owned-path and public-boundary drift report;
- independent blocker review bound to the candidate commit;
- every repair round and blocker identity recorded above before another mutation.

---

## W1 M30-B0 existing-kernel audit report

### Exact source and scope

- `Repository`: `Develata/ustc-campus-agent`
- `Source commit`: `f62018326bbc180ec79cc04ec3dde214d7007094`
- `Source tree`: `a9c5529f9deada0992c26a07c5d6e4a3fcdc5585`
- `Worker branch`: `campaign/w1-m30-b0-audit-v2`
- `Campaign`: `USTC-MODULES-2026-07-W1`
- `Audit lane`: `M30-B0`
- `Mode`: `audit-only`
- `Campaign grant carrier`: [`docs/tasks/01-execution-roadmap.md`](01-execution-roadmap.md#active-autonomous-module-campaign-authorization) (`AUTONOMOUS_CAMPAIGN_GRANT` block, `M30-B0` row)
- `Lane taskbook`: [`docs/tasks/campaign-w1-m30-b0.md`](campaign-w1-m30-b0.md)
- `Allowed Git-visible edit and audit-report carrier`: `docs/tasks/campaign-w1-m30-b0.md` (this file); no second path or other carrier is touched
- `Audit scope`: reconcile existing `crates/agent-runtime/` against the M30 blueprint, `agent-runtime`/`agent-harness` contracts, the active acceptance matrix, the long-horizon catalog, the coverage matrix and the module map; record one evidence-bound disposition without promoting any acceptance posture, lifecycle, runtime state machine, protocol, authority or boundary.

Source binding was verified before any edit:

```text
git rev-parse HEAD          → f62018326bbc180ec79cc04ec3dde214d7007094
git rev-parse HEAD^{tree}   → a9c5529f9deada0992c26a07c5d6e4a3fcdc5585
git branch --show-current   → campaign/w1-m30-b0-audit-v2
git status --short (start)  → clean (no tracked or untracked paths)
```

`CARGO_TARGET_DIR` and `PYTHONPYCACHEPREFIX` are run-owned scratch directories outside the repository; no generated state enters the repository from this audit.

### Disposition

`adopt`

Evidence is consistent: the existing `crates/agent-runtime` kernel is truthfully confined to M30 blueprint items `run-spec` and `agent-run`; no retained code implements `harness-run`, `task-contract`, `task-graph`, `scheduler`, `context-budget`, `context-projection`, `evidence-review`, `runtime-ports` or `run-projection`; `HARNESS-001` and `HARNESS-003` remain correctly `planned` in `docs/acceptance/matrix.tsv`; `HARNESS-002` remains correctly catalog-only/non-admitted; owned-path and dependency direction are clean; no contract/blueprint/module-map/acceptance/coverage/roadmap carrier overstates the retained code. No product, authority, lifecycle, protocol, boundary or acceptance-posture change is proposed.

### Repair chronology

- Raw generation v4 terminated as `BLOCKED_SCOPE_VIOLATION`; its immutable taskbook/report SHA-256 values are `b4172676e88a1ebdba95b2b39a29eb5477f5c76c14f0cba77b9ab18055df1a49` and `933395aadd2c75482c362d9196e41596e33a34e08bbf83840bc99458f83ac6b5`.
- `R1-F1_FALSE_AGENT017_BINDING_TYPO_CLAIM`: parent read-back showed `docs/acceptance/matrix.tsv` already uses the declared `ustc-campus-agent-runtime` package ID. The raw report's claim of a matrix typo was false and is removed here.
- `R1-F2_RUN_OWNED_OMO_SCOPE_RESIDUE`: two exact `.omo/run-continuation/*.json` files were generated during the OpenCode run. Parent cleanup verified their raw hashes before deletion; they are absent from this repaired candidate.
- Repair round `1` was recorded in the mutable taskbook before either repair. No code, contract, matrix, checker, workflow or source-control object was mutated.
- Repair round `2` corrected parent-found mutable-state Markdown/status projection defects, `R2-F4_DISPOSITION_ENUM_MISMATCH` and `R2-F5_REPORT_PATH_OUTSIDE_CANONICAL_FINITE_ALLOWED_PATHS`. The final `adopt` disposition means the retained items 1–2 are adopted as truthful current executable evidence; it does not adopt or implement blueprint items 3–11 and does not promote M30 readiness. For R2-F5, the exact report was embedded into the canonical taskbook and the separate untracked report carrier was removed.
- `R2-F6_EMBEDDED_REPORT_LANE_FIELD_COLLISION` was resolved by renaming the embedded metadata carrier so the taskbook retains exactly one authoritative `Lane` field.
- `R2-F7_CAMPAIGN_MUTATION_TEST_HARD_CODES_PRE_BIND_M30_STATE` was resolved in governance PR #47; exact-main CI run 32961865331 passed before this source rebind.
- `R2-F8_VOLATILE_RECEIPT_NON_CLAIM_CONTRADICTION` was resolved by qualifying the Non-claims section: only verified historical F7 receipts are named; unverified and future PR/CI/merge claims remain excluded.
- `R2-F9_MATRIX_ACTIVE_NON_CLAIM_CONTRADICTION` was resolved by splitting matrix-backed active/planned HARNESS rows from catalog-only `HARNESS-002` in Non-claims.
- `R2-F10_M40_NEUTRAL_PROTOCOL_DEPENDENCY_OVERCLAIM` was resolved by naming the allowed M40-owned Plugin-neutral protocol seam while continuing to exclude M40 implementation/private dependencies.

### Existing-kernel decomposition

#### File inventory

`crates/agent-runtime/` contains exactly two files (confirmed by glob):

- `crates/agent-runtime/Cargo.toml` — package declaration only.
- `crates/agent-runtime/src/lib.rs` — the entire implementation (1759 lines, single module, no submodules, no `tests/` directory).

#### Blueprint small-module mapping

[`docs/plan/modules/40-agent-harness-runtime.md`](../plan/modules/40-agent-harness-runtime.md) §13 decomposes M30 into eleven small modules and states: "Existing `agent-runtime` is reviewed as items 1–2, not treated as proof of items 3–11."

| Blueprint item (§13) | Retained implementation | Evidence |
|---|---|---|
| 1. `run-spec` — immutable identities and budgets | YES | `RunSpec` (`lib.rs:38-61`), `RunBudgets` (`lib.rs:18-33`), `RUN_SPEC_SCHEMA_VERSION` (`lib.rs:13`), `RunSpec::validate` (`lib.rs:64-114`) |
| 2. `agent-run` — node-local phase/command/event/replay kernel | YES | `AgentRun` (`lib.rs:379-398`), `RunPhase` (`lib.rs:120-139`), `RunCommand` (`lib.rs:243-269`), `RunEvent`/`RunEventKind` (`lib.rs:312-366`), `Decision` (`lib.rs:370-375`), `RuntimeError` (`lib.rs:923-981`), `AgentRun::new/replay/decide/apply` (`lib.rs:402-559`), plus `ModelUsage`, `ToolCallProposal`, `EffectIntent`, `EffectOutcome`, `EffectReceipt`, `TerminalOutcome` |
| 3. `harness-run` — finite user-task phases and suspension/terminal rules | NO | No `HarnessRun`, `HarnessRunSpec` or harness phase type. `RunPhase` is node-local (`Created/Preparing/ModelTurn/AwaitingToolApproval/ExecutingTools/Completed/Failed/Cancelled/Expired`); the harness phase machine in [`agent-harness.md`](../contracts/agent-harness.md) §2 (`Received/Contextualizing/Clarifying/Planning/PlanValidated/Executing/Verifying/Reviewing/Reporting/AwaitingUser/Remediating` + `Succeeded/Partial/Failed/Blocked/Expired/Cancelled`) is entirely absent. |
| 4. `task-contract` — immutable parent goal/non-goals/deliverables/acceptance | NO | No `TaskContract` type or module. |
| 5. `task-graph` — finite graph validation and revisions | NO | No `TaskGraph`, `TaskGraphProposal`, `TaskNode` or `GraphRevision` type; no graph validation code. |
| 6. `scheduler` — dependency/resource-ready dispatch | NO | No scheduler type or dispatch code. |
| 7. `context-budget` — complete-request measurement and integer policy | NO | No `ContextBudgetSnapshot`; no `T(q)+O+S ≤ floor(L×ρ/10_000)` preflight. `RunBudgets` is node-level turn/tool/token/cost/retry/elapsed accounting, not the context-window preflight of [`agent-harness.md`](../contracts/agent-harness.md) §6. |
| 8. `context-projection` — deterministic offload/compaction/compression artifacts | NO | No `PromptProjection`, `ContextSummaryArtifact` or `CompressionPlan`. |
| 9. `evidence-review` — EvidencePack, fresh review and remediation | NO | No `EvidencePack`, `ReviewReceipt` or review-disposition type. |
| 10. `runtime-ports` — provider/tool/journal/artifact/clock fakes | NO | No `trait` declarations; no `ModelInvocationPort`, `PluginNeutralToolExchange`, `RunJournalPort`, `ArtifactPort` or `Clock/SchedulerPort`. |
| 11. `run-projection` — safe client/application view | NO | No M10/M80 projection type. |

The crate declares no `trait` items at all; its only cross-crate integration is `From<&ustc_agent_tool_protocol::AgentToolCall> for ToolCallProposal` (`lib.rs:174-182`), which consumes the Plugin-neutral protocol seam defined in [`agent-plugin-boundary.md`](../contracts/agent-plugin-boundary.md) §3.

#### Test inventory

All 14 tests live in `lib.rs` lines 1112–1758 (`#[cfg(test)] mod tests`). They exercise `run-spec` validation/round-trip, legal replay, illegal-transition fail-closed, terminal mutation rejection, effect identity/in-flight-termination fail-closed, receipt idempotency/conflict, budget fail-closed/replay, model-usage once/bounded, elapsed-budget blocking, retry return-to-preparing, revision exhaustion and event-sequence duplicate rejection. No test references `HarnessRun`, `TaskGraph`, `TaskContract`, clarification, context-budget preflight, compaction, compression, evidence/review, scheduler/supervisor, run-projection or any port trait.

### HARNESS acceptance reconciliation

#### `HARNESS-001` — active, `planned`

- Matrix row: [`docs/acceptance/matrix.tsv`](../acceptance/matrix.tsv) line 27 — `HARNESS-001 | harness | every HarnessRun follows legal finite evidenced transitions and reaches an explicit terminal phase | future H0 Rust state-machine and replay tests | pr | planned | backend`.
- Catalog row: [`docs/acceptance/platform-baseline.md`](../acceptance/platform-baseline.md) §13 line 286 — same assertion, `rust-unit | PR`.

Why existing node-local `AgentRun` transitions/replay do not satisfy finite user-task `HarnessRun` terminal semantics:

1. **Phase machine mismatch.** `AgentRun` (`lib.rs:120-139`) owns `Created/Preparing/ModelTurn/AwaitingToolApproval/ExecutingTools/Completed/Failed/Cancelled/Expired`. `HarnessRun` ([`agent-harness.md`](../contracts/agent-harness.md) §2) owns `Received/Contextualizing/Clarifying/Planning/PlanValidated/Executing/Verifying/Reviewing/Reporting/AwaitingUser/Remediating` plus terminals `Succeeded/Partial/Failed/Blocked/Expired/Cancelled`. The harness phases `Clarifying`, `Planning`, `PlanValidated`, `Verifying`, `Reviewing`, `Reporting`, `AwaitingUser` and `Remediating` have no counterpart in `AgentRun`.
2. **Terminal vocabulary mismatch.** `AgentRun` terminals are `Completed/Failed/Cancelled/Expired`. `HarnessRun` terminals are `Succeeded/Partial/Failed/Blocked/Expired/Cancelled`. The user-task terminals `Succeeded`, `Partial` and `Blocked` do not exist in `AgentRun`; `Completed` is a node-level outcome, not a user-task outcome.
3. **No typed suspension.** `AwaitingUser` ([`agent-harness.md`](../contracts/agent-harness.md) §2: "a typed suspension carrying reason, questions/decision, deadline and `resume_phase`") is absent. `AgentRun` has no suspension phase and no resume-phase carrier.
4. **No review-disposition mapping.** The six legal final-review dispositions (`Pass/RemediateWithinScope/Replan/NeedsUserDecision/PolicyBlocked/BudgetExhausted`) and their harness transitions ([`agent-harness.md`](../contracts/agent-harness.md) §2 table) have no implementation.
5. **No in-flight-effect terminal reconciliation.** `AgentRun` rejects terminalization while an effect is in flight (`RuntimeError::InFlightEffectCannotTerminate`, `lib.rs:887-893`), but the harness contract requires `Executing` with an unresolved child/effect to persist a terminal intent and remain `Executing` until child outcome and every required effect receipt reconcile, then enter the requested terminal phase ([`agent-harness.md`](../contracts/agent-harness.md) §2 failure/cancellation/expiry table). That reconciliation state machine is not implemented.
6. **No root-contract partitioning.** The harness requires that an answer changing the root goal/prohibitions/deliverables/acceptance/immutable budget cannot resume the same run ([`agent-harness.md`](../contracts/agent-harness.md) §2 final paragraph). `AgentRun` has no root contract, no partition check and no new-run fork.

`HARNESS-001` remains correctly `planned`.

#### `HARNESS-003` — active, `planned`

- Matrix row: [`docs/acceptance/matrix.tsv`](../acceptance/matrix.tsv) line 28 — `HARNESS-003 | harness | accepted TaskGraph is finite acyclic authority-valid and resource-compatible | future H0 Rust graph fixture tests | pr | planned | backend`.
- Catalog row: [`docs/acceptance/platform-baseline.md`](../acceptance/platform-baseline.md) §13 line 288 — same assertion, `rust-unit | PR`.

No finite, acyclic, authority-valid, resource-compatible accepted `TaskGraph` carrier or test exists:

1. No `TaskGraph`, `TaskGraphProposal`, `TaskNode` or `GraphRevision` type is declared anywhere in `crates/agent-runtime/` (confirmed by full read of `lib.rs` and glob of the crate).
2. No graph validation code exists: no acyclicity check, no dependency-completeness check, no resource-claim compatibility check, no authority-widening rejection, no per-node/total budget validation against a graph. The validation code in `lib.rs` is node-local (`RunSpec::validate`, `AgentRun::validate_command/validate_event/validate_intent/validate_receipt`).
3. The harness contract's `TaskGraph` acceptance criteria ([`agent-harness.md`](../contracts/agent-harness.md) §4: schema/version, unique node IDs, known executor classes, acyclicity, complete dependency references, immutable parent-owned task contracts, capability/path/service resource claims and isolation compatibility, per-node and total budgets, no authority widening or forbidden fallback) have no implementation.

`HARNESS-003` remains correctly `planned`.

#### `HARNESS-002` — catalog-only, non-admitted

- Matrix: `HARNESS-002` is absent from [`docs/acceptance/matrix.tsv`](../acceptance/matrix.tsv) (all 62 case rows checked: `HARNESS-001`, `HARNESS-003`, `HARNESS-005`, `HARNESS-006`, `HARNESS-008`, `HARNESS-010` are present; `HARNESS-002`, `HARNESS-004`, `HARNESS-007`, `HARNESS-009` are not).
- Catalog: [`docs/acceptance/platform-baseline.md`](../acceptance/platform-baseline.md) §13 line 287 — `HARNESS-002 | clarification asks only material blocking uncertainty under bounded rounds and deadline | rust-unit | PR`.

Per [`docs/acceptance/platform-baseline.md`](../acceptance/platform-baseline.md) §1: "only `matrix.tsv` is the active competition gate registry" and "A catalog case becomes an active required case only when its owning feature enters scope and is projected into `matrix.tsv`." `HARNESS-002` is therefore catalog-only/non-admitted. No current evidence and no taskbook or report edit in this lane promotes it. This report records its catalog-only status without creating an active binding.

#### Other active `HARNESS-*` rows (for completeness)

`HARNESS-005` (context inequality preflight), `HARNESS-006` (compaction/compression provenance), `HARNESS-008` (worker resume/fresh reviewer/bounded remediation) and `HARNESS-010` (hooks/process-exit cannot complete a node) are all `planned` in `matrix.tsv` with `future H0 ...` bindings. None is implemented by the existing kernel; none is promoted by this audit.

### Owned-path and dependency drift

#### Owned paths

`crates/agent-runtime/` contains only `Cargo.toml` and `src/lib.rs` (glob-confirmed). This matches [`40-agent-harness-runtime.md`](../plan/modules/40-agent-harness-runtime.md) Metadata "Primary code areas: `crates/agent-runtime/` and future cohesive harness modules" and [`agent-runtime.md`](../contracts/agent-runtime.md) Metadata "Primary Code: `crates/agent-runtime/`". No file drift.

#### Dependency direction

`crates/agent-runtime/Cargo.toml` dependencies:

- `serde.workspace = true`
- `ustc-agent-tool-protocol.workspace = true`

Dev-dependencies:

- `serde_json.workspace = true`

No dependency on `platform-core`, `adapters`, `agent-runtime` peers, M40 implementation/private surfaces, or any M10/M20/M50/M80 surface. The sole M40-owned dependency is the allowed Plugin-neutral `ustc-agent-tool-protocol` seam described above. This satisfies:

- [`agent-runtime.md`](../contracts/agent-runtime.md) §1: "no provider SDK, MCP transport, Market manifest/type, Plugin implementation, adapter, database, HTTP server or user interface dependency."
- [`agent-runtime.md`](../contracts/agent-runtime.md) §8: "The crate MUST build and test without Market, Plugin implementation or adapter dependencies."
- [`agent-plugin-boundary.md`](../contracts/agent-plugin-boundary.md) §6.1: "agent-runtime and future harness code MUST NOT depend on Market manifests, Plugin domain types, component implementations, adapter crates or framework extension APIs."
- [`40-agent-harness-runtime.md`](../plan/modules/40-agent-harness-runtime.md) §5 forbidden dependencies: "Market manifests and `M20` private types; Plugin/MCP/provider implementations; concrete database/framework checkpoint types; Dioxus/client state; product-specific ChangeRadar/Affairs/Opportunity logic."
- [`00-module-map.md`](../plan/modules/00-module-map.md) §2 dependency rule 4: "`M30` and `M40` depend on the Plugin-neutral tool protocol, not on each other's implementation."

No forbidden dependency direction. No owned-path drift. No product defect.

### Public-boundary/status drift

#### Status projections are honest

All authority carriers consistently project M30 as `partial-evidence` with the node kernel implemented and the harness planned:

- [`00-module-map.md`](../plan/modules/00-module-map.md) §1 M30 row: `partial-evidence | node runtime kernel implemented; harness planned`.
- [`40-agent-harness-runtime.md`](../plan/modules/40-agent-harness-runtime.md) Metadata: `Implementation State: partial-evidence`; §13: "Existing `agent-runtime` is reviewed as items 1–2, not treated as proof of items 3–11."
- [`agent-runtime.md`](../contracts/agent-runtime.md) Metadata: `Status: R0 kernel contract implemented; durable orchestration and external adapters planned`; §9: "Still planned: ... finite HarnessRun/TaskGraph and context-budget preflight."
- [`agent-harness.md`](../contracts/agent-harness.md) Metadata: `Status: Accepted target architecture; H0 implementation planned`; §1: "The harness owns one finite user task... It does not replace the conversation session or the existing single-node `AgentRun`."
- [`01-execution-roadmap.md`](01-execution-roadmap.md) §3 M30 row: `partial-evidence | node kernel only`.
- [`01-execution-roadmap.md`](01-execution-roadmap.md) §8: "`M30-B0 existing-kernel-audit`: map `agent-runtime` to `run-spec`/`agent-run`; do not extend before decision."
- [`coverage-matrix.md`](../coverage-matrix.md) M30 row: `active:HARNESS-*`; `active:AGENT-*`. The `active:` token means the family has rows in `matrix.tsv`, not that they are implemented ([`coverage-matrix.md`](../coverage-matrix.md) token definitions). `matrix.tsv` confirms `HARNESS-*` are `planned` and `AGENT-001/002/017` are `implemented`.

No carrier overstates the retained code. No `partial-evidence`/`planned` posture is violated.

#### Acceptance posture is honest

- `AGENT-001` (`matrix.tsv` line 20): `implemented`, binding `cargo test --locked -p ustc-campus-agent-runtime tests::legal_run_replays_deterministically -- --exact && ... tests::illegal_transitions_fail_closed -- --exact && ... tests::terminal_phases_reject_state_changes -- --exact`. These three tests exist at `lib.rs:1264`, `lib.rs:1319` and `lib.rs:1686` and pass.
- `AGENT-002` (`matrix.tsv` line 21): `implemented`, binding `cargo test --locked -p ustc-campus-agent-runtime tests::run_spec_round_trip_preserves_exact_identity -- --exact && ... tests::run_spec_rejects_unknown_fields_and_zero_budgets -- --exact && ... tests::model_usage_is_required_once_and_bounded -- --exact`. These three tests exist at `lib.rs:1193`, `lib.rs:1226` and `lib.rs:1546` and pass.
- `AGENT-017` (`matrix.tsv` line 22): `implemented`, binding `python3 scripts/check_repo_contracts.py && cargo test --locked -p ustc-campus-agent-runtime && cargo test --locked -p ustc-agentd --test resolved_run_spec`. All three canonical commands pass in the repaired-candidate gate run; the last command remains composition-root evidence rather than proof of a finite harness.
- `HARNESS-001/003/005/006/008/010`: `planned`, with `future H0 ...` bindings. Correct.

### Targeted verification

#### Gate commands and real results

```bash
# 1. Canonical Agent runtime package gate
CARGO_NET_OFFLINE=true CARGO_TARGET_DIR="$CARGO_TARGET_DIR" cargo test --locked --offline -p ustc-campus-agent-runtime
# → exit 0
# → 14 passed; 0 failed; 0 ignored

# 2. Canonical AGENT-017 composition-root gate
CARGO_NET_OFFLINE=true CARGO_TARGET_DIR="$CARGO_TARGET_DIR" cargo test --locked --offline -p ustc-agentd --test resolved_run_spec
# → exit 0
# → 3 passed; 0 failed; 0 ignored
# → `--offline` is an execution-environment constraint only; package/test selection matches the canonical matrix binding.

# 3. Repository contract checker
PYTHONPYCACHEPREFIX="$PYTHONPYCACHEPREFIX" python3 scripts/check_repo_contracts.py --ci
# → exit 0
# → contract-check: PASS

# 4. Whitespace/conflict marker check
git diff --check
# → exit 0 (clean)

# 5. Working-tree status (tracked + untracked)
git status --short
# → docs/tasks/campaign-w1-m30-b0.md    (modified, admitted)

# 6. Tracked diff names
git diff --name-only
# → docs/tasks/campaign-w1-m30-b0.md

# 7. Untracked paths not in .gitignore
git ls-files --others --exclude-standard
# → empty
```

#### Changed paths (this audit)

- `docs/tasks/campaign-w1-m30-b0.md` — mutable campaign fields updated and this exact audit report embedded as the only Git-visible carrier.

No other tracked or untracked path remains. The two run-owned `.omo/run-continuation/*.json` residue files from raw generation v4 were hash-bound and removed during repair round 1 before candidate freeze.

#### Gate interpretation

- The agent-runtime crate's 14 unit tests pass under the correct package id `ustc-campus-agent-runtime` (exit 0).
- The repository contract checker passes (exit 0).
- `git diff --check` passes (exit 0): no whitespace errors, no conflict markers.
- The tracked diff is confined to the single admitted taskbook path and the untracked set is empty. The disposition is `adopt`; no source code, Cargo manifest, lockfile, test, fixture, contract, blueprint, roadmap, acceptance matrix, checker, workflow, CODEOWNERS or root-governance file is modified.

### Non-claims and next gate

#### Non-claims

This report does NOT claim:

- that matrix-backed `HARNESS-001`, `HARNESS-003`, `HARNESS-005`, `HARNESS-006`, `HARNESS-008` or `HARNESS-010` is implemented, passed, accepted or readiness evidence; they remain active `planned` rows with planned bindings;
- that catalog-only `HARNESS-002` is implemented, bound, active, passed, accepted or readiness evidence;
- that `M30` is `StandaloneReady`, `IntegrationReady`, `Integrated` or `Accepted`;
- that any `harness-run`, `task-contract`, `task-graph`, `scheduler`, `context-budget`, `context-projection`, `evidence-review`, `runtime-ports` or `run-projection` implementation exists;
- that any acceptance posture, lifecycle state machine, runtime state machine, protocol behavior, authority ownership, boundary, permission semantic or public API has changed;
- any unverified or future PR/CI/merge claim; historical PR #47 and exact-main CI run 32961865331 appear only as source-bound R2-F7 resolution receipts;

#### Next gate

The candidate now awaits Hermes exact-bytes review of this report and the taskbook mutation, an independent blocker review bound to the exact commit/changed-path set/outgoing range, the authoritative gates above, and then a source-control decision under the `USTC-MODULES-2026-07-W1` campaign grant. This lane does not merge, accept, promote or release; the `audit-only` auto-merge boundary in the campaign grant admits merge only when the result changes no acceptance posture, Agent/Harness lifecycle or runtime state-machine behavior, which this candidate satisfies.
