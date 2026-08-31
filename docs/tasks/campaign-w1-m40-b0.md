# W1 M40-B0 protocol/fake-gateway audit

## Authority

- `Campaign ID`: `USTC-MODULES-2026-07-W1`
- `Lane`: `M40-B0`
- `Grant carrier`: [`01-execution-roadmap.md`](01-execution-roadmap.md#active-autonomous-module-campaign-authorization)
- `Mode`: audit-only; no public protocol, execution ordering or executor behavior change

## Mutable campaign state

- `Status`: `paused`
- `Bound source commit`: `d2e53251a21c97a5298c4559680e7e17712a5eda`
- `Bound source tree`: `fda6bb88b90951d3266930d06d3a019a52603b19`
- `Worker branch`: `campaign/w1-m40-b0-audit-v2`
- `Repair round`: `2`
- `Current blocker identity`: `PR58-R2-F11-OPPORTUNITY-WRITE-BOUNDARY`
- `Stop reason`: `the Opportunity in-process NativeRustComponent tenant-private read/write path remains outside the accepted PublicRead/no-side-effect bootstrap; repair round 2 is exhausted, so this lane remains paused pending a separately authorized contract or implementation amendment`
- `Last transition evidence`: `audit-only reconciliation of retained agent-tool-protocol/v0 and composition-root fake-gateway/executor evidence against M40 blueprint/contracts/acceptance; targeted gates passed (cargo test --locked -p ustc-agent-tool-protocol=4 passed; cargo test --locked -p ustc-agentd --test tool_gateway_conformance=2 passed; python3 scripts/check_repo_contracts.py=contract-check: PASS); disposition ADOPT at exact current scope with no readiness promotion; Hermes repair round 1 replaced one unresolved source-line placeholder with verified line 104 and preserved OpenCode continuation residue outside the repository before authoritative rerun; Hermes repair round 2 narrowed generic ToolGateway-shaped synthetic evidence from bounded product-specific fixed-adapter composition evidence; the formal reviewer's placeholder finding was adjudicated as a read-file redaction false positive by raw-byte proof; the resolved B2 identity remains in this transition evidence while the current-blocker field returns to none so round-2 active-state truth satisfies the fail-closed campaign checker; PR #58 exact-head review at 2c3e73cd6835d676c13c254115edbe3c2ff878c5 opened source-confirmed findings R2-F11 comment 3892040857 and R2-F12 comment 3892040859; Hermes recorded paused state before terminal audit correction; Hermes source-confirmed both PR #58 findings, corrected the disposition from ADOPT to AMEND, and retained paused round-2 state because the Opportunity write boundary requires a separately authorized contract or implementation slice; exact-head GitHub review comments 3892164760 and 3892164770 then showed that corrected consumer inventory was no longer a current blocker while the roadmap projection remained stale; Hermes opened the terminal round-2 audit correction before mutating reconciliation prose; terminal round-2 correction removed resolved F12 from current blocker state and marked the roadmap Opportunity projection stale; F13 is resolved for this audit by explicit non-ratification, while W1 closeout retains the exact cross-carrier correction obligation`
- `Next allowed mutation`: `focused exact-delta review, replacement exact-head CI and resolution/read-back of all PR #58 review threads may ship this AMEND/paused audit; after merge, W1 closeout must correct the stale roadmap projection, and no M40 implementation or contract mutation is admitted without new Develata authorization`

## Output contract

Audit the current tool protocol, fake gateway and admitted execution evidence against M40 contracts, then record exactly one `adopt | amend | retain as spike | remove` disposition. Auto-merge is admitted only when public protocol, execution ordering and executor behavior are unchanged and no readiness state is promoted.

## Required evidence

- exact source commit and clean checkout receipt;
- reconciliation for matrix-implemented `AGENT-017`, matrix-planned `AGENT-018`, and catalog-only, non-admitted `AGENT-003`, `AGENT-004`, `AGENT-009`, `AGENT-010`, `AGENT-011`, `AGENT-012`, `AGENT-013`;
- owned-path and public-boundary drift report;
- independent blocker review bound to the candidate commit;
- every repair round and blocker identity recorded above before another mutation.

## Audit report

### Exact source and scope

- `source_commit`: `d2e53251a21c97a5298c4559680e7e17712a5eda`
- `source_tree`: `fda6bb88b90951d3266930d06d3a019a52603b19`
- `branch`: `campaign/w1-m40-b0-audit-v2`
- `campaign`: `USTC-MODULES-2026-07-W1`
- `lane`: `M40-B0`
- `mode`: `audit-only`
- `changed_paths`: `docs/tasks/campaign-w1-m40-b0.md` (this file; sole admitted Git-visible edit)
- `scope`: reconcile retained `agent-tool-protocol/v0`, composition-root fake gateway/executor evidence and fixed-adapter source against the M40 blueprint, contracts, roadmap, coverage matrix and acceptance matrix; record one evidence-bound disposition; this audit changes no public protocol, execution ordering, executor, authority, lifecycle, acceptance or runtime behavior.

### Disposition

AMEND — the retained bounded protocol/fake evidence remains valid at its exact bindings, but the current source cannot be adopted as boundary-conformant. `apps/ustc-agentd/src/opportunity_invocation.rs` assembles a `NativeRustComponent` with `TenantPrivateRead`/`TenantPrivateWrite` capabilities and executes profile creation, planning and revoke/delete inside the composition root. `docs/contracts/agent-plugin-boundary.md` §4 admits only an exact statically linked first-party `PublicRead` bootstrap with no external side effect and keeps writes/native executors out of process. The lane is therefore paused for an authorized contract or implementation amendment. Existing protocol/fake evidence is retained, and no readiness or acceptance posture is promoted.

### Protocol decomposition

Blueprint small-modules `tool-schema` and `agent-tool-envelope` (`docs/plan/modules/50-tool-gateway-execution.md` §13 items 1–2) map to current production code as follows.

`tool-schema` — `crates/agent-tool-protocol/src/canonical.rs`:
- `Sha256Digest` (canonical.rs lines 15–52): lowercase `sha256:<64 hex>` parse/hash helper.
- `UnvalidatedToolInputSchemaV0` → `ValidatedToolInputSchemaV0` (lines 70–177): dialect `tool-input-schema/v0`, six node variants (object/string/integer/number/boolean/array), canonical byte encoding over `tool-input-schema/v0\0`, limits (depth 8, nodes 256, object members 64, schema bytes 65_536, enum 1..=64 values × 1..=256 bytes), `SchemaConstructionError::{SchemaDialectUnsupported, SchemaMalformed, SchemaLimitExceeded}`.
- `UnvalidatedArgumentValueV0` → `CanonicalArgumentValueV0` (lines 290–373): seven node variants (null/boolean/integer/number/string/array/object), canonical byte encoding over `tool-arguments/v0\0`, `-0.0` normalized to `+0.0`, limits (depth 8, nodes 256, object members 64, array elements 256, string 4_096 bytes, argument bytes 65_536), `ArgumentConstructionError::{ArgumentDuplicateKey, ArgumentInvalidName, ArgumentNumberOutOfRange, ArgumentLimitExceeded}`.
- `is_valid_tool_name` (lines 488–498): `^[A-Za-z_][A-Za-z0-9_.-]{0,63}$`.

`agent-tool-envelope` — `crates/agent-tool-protocol/src/lib.rs`:
- `AGENT_TOOL_PROTOCOL_VERSION = "agent-tool-protocol/v0"` (line 21).
- `AgentToolDefinition` (lines 98–155): model-visible name, description (≤4_096 bytes), validated input schema, `provider_definition_digest` over `provider-tool-definition/v0\0`.
- `AgentTool` (lines 158–172): definition + opaque `ToolRouteRef`.
- `AgentToolsetView` (lines 175–280): immutable per-turn view with `run_id`, `turn_id`, `projection_snapshot_id`, `tool_definition_set_digest` over `tool-projection/v0\0`, sorted definitions, duplicate-name/duplicate-route rejection, `bind_call` binding a provider call to the frozen route.
- `AgentToolCall` (lines 283–334): correlated call carrying `route_ref` and canonical arguments.
- `AgentToolOutcome` (lines 337–344): `Succeeded { output_digest } | Failed | Denied | Cancelled | TimedOut` (each with `StableToolCode` except `Succeeded`).
- `AgentToolResult` (lines 347–397): correlated result via `from_call`.
- Tests: 4 unit tests (lib.rs lines 434–525): `frozen_view_sorts_definitions_and_binds_private_route`, `duplicate_names_and_routes_fail_closed`, `unknown_tool_never_produces_a_call`, `result_is_correlated_without_plugin_identity`.
- `crates/agent-tool-protocol/Cargo.toml` line 13: only dependency `sha2` (workspace); no Market/Plugin/adapter/framework/transport dependency. Crate is provider-neutral and sealed as specified by `docs/contracts/agent-plugin-boundary.md` §3.

No `route-table`, `call-normalization`, `gateway-authorization`, `execution-stages`, `executor-port`, `output-boundary`, `gateway-recovery` or `gateway-conformance` production module exists in this crate; it owns only the two protocol small-modules.

### Fake-gateway/executor evidence classification

All generic `ToolGateway`-shaped behavior evidence in this audit lives in the composition-root synthetic test `apps/ustc-agentd/tests/tool_gateway_conformance.rs` and its support `apps/ustc-agentd/tests/support/mod.rs`. Affairs and ChangeRadar are the bounded fixed-adapter compositions currently projected by the M40 plan; the Opportunity fixed path is recorded below as a contract conflict, not admitted M40 evidence. The test-local `FakeToolGateway` (lines 72–112) and `FakePluginExecutor` (lines 46–63) are NOT a production gateway or public M40 implementation.

The synthetic proof exercises, exclusively inside the test:
- private route correlation: `FakeToolGateway::execute` (lines 88–93) rejects `ProjectionMismatch` when `run_id`/`turn_id`/`projection_snapshot_id` differ from the frozen projection;
- call normalization: lines 94–103 rebuild a `ProposedToolCall` from the `AgentToolCall` envelope and route ref;
- current authorization: line 104 calls `authorize_call` from `crates/platform-core/src/invocation.rs` (lines 1236–1324), the M20-owned deny-side recheck;
- fake executor invocation: `FakePluginExecutor::execute` (lines 52–63) records the sealed `AuthorizedInvocation` and returns `Sha256Digest::from_bytes(arguments.canonical_bytes())`;
- no-executor denial: `invalid_or_stale_calls_never_execute` (lines 258–350) asserts `executor.observed.is_empty()` for unknown tool (`ToolNotProjected`), malformed arguments (`ArgumentsInvalid`), route mismatch (`DispatchIdentityMismatch`), current denial (`EmergencyBlocked`) and projection mismatch;
- correlated digest/code result: lines 107–110 construct `AgentToolResult::from_call` with `AgentToolOutcome::Succeeded { output_digest }`, and `provider_view_to_gateway_to_executor_is_correlated_and_authorized` (lines 189–256) asserts exact correlation of run/turn/snapshot/provider-call IDs and output digest.

`proof_authority()` in `support/mod.rs` (lines 20–138) builds one synthetic `InvocationAuthorityCandidate` with a `NativeRustComponent` tool `proof_tool`; this is test input only and creates no production authority. The composition-root paths in `apps/ustc-agentd/src/{affairs,change,opportunity}_invocation.rs` consume `agent-tool-protocol`, `agent_toolset_view`, `authorize_call` and `ToolProjectionSnapshot`, but they are per-product fixed adapters rather than a generic `ToolGateway`. Affairs and ChangeRadar are the plan-projected bounded compositions. Opportunity additionally declares `NativeRustComponent` execution with tenant-private read/write capabilities and performs the owning operations in process, which exceeds the accepted §4 bootstrap boundary.

### Unimplemented blueprint remainder

Blueprint small-modules 3–10 (`docs/plan/modules/50-tool-gateway-execution.md` §13 items 3–10) are absent from production code. Verified by structural search across the repository:

- `route-table`: no `mod route_table`, no production route-table type. Route correlation exists only as the test-local `FakeToolGateway` projection check.
- `call-normalization`: no `mod call_normalization`. Normalization is inlined in the test gateway via `ProposedToolCall`; production `AgentToolsetView::bind_call` validates name/route inside the protocol crate.
- `gateway-authorization`: no `mod gateway_authorization`. Deny-side recheck is M20-owned `authorize_call` (`crates/platform-core/src/invocation.rs` lines 1236–1324), invoked by the test-local gateway; no M40-owned authorization module exists.
- `execution-stages`: no `mod execution_stages`, no `PreparedToolExecution` type. Staged prepare/execute/result is not implemented as public M40 stages; composition interleaving with `M30` intent/receipt commands is described in the blueprint/contract but not codified.
- `executor-port`: no `mod executor_port`, no public `PluginExecutionRequest`/`PluginExecutionOutcome` types. Only the test-local `FakePluginExecutor` exists.
- `output-boundary`: no `mod output_boundary`, no `OutputBound`/`BoundedPluginExecutionOutcome` types. Untrusted content/artifact/schema/size/redaction limits are not implemented.
- `gateway-recovery`: no `mod gateway_recovery`. Duplicate/reconcile/timeout/cancel and receipt reconciliation are not implemented.
- `gateway-conformance`: no `mod gateway_conformance`. The composition test is synthetic/fake; no admitted executor conformance exists.

Structural grep for `struct ToolGateway|struct PreparedToolExecution|struct PluginExecutionRequest|struct PluginExecutionOutcome|struct BoundedPluginExecutionOutcome|struct OutputBound|struct GatewayCallCorrelation|struct AuthorizedExecutionEnvelope` returned zero matches. Structural grep for `mod route_table|mod gateway_authorization|mod execution_stages|mod executor_port|mod output_boundary|mod gateway_recovery|mod gateway_conformance|mod call_normalization` returned zero matches. `FakeToolGateway`/`FakePluginExecutor` appear only in `apps/ustc-agentd/tests/tool_gateway_conformance.rs`.

### Acceptance reconciliation

`AGENT-017` — matrix.tsv line 27: `implemented`, gate `pr`, owner `backend`. Binding: `python3 scripts/check_repo_contracts.py && cargo test --locked -p ustc-campus-agent-runtime && cargo test --locked -p ustc-agentd --test resolved_run_spec`. This is M30 compilation-boundary evidence: `agent-runtime` has no Market/Plugin/component/adapter implementation dependency (per `docs/contracts/agent-plugin-boundary.md` §6.1). The `cargo test --locked -p ustc-agentd --test resolved_run_spec` composition proof maps resolver output into `RunSpec`/`AgentRun` and is relevant to M40 only insofar as it proves the protocol seam is consumed downstream; it is NOT a gateway proof and does not establish production `ToolGateway` readiness. The M40-B0 minimal taskbook sentence does not name `AGENT-017`, but the roadmap M40-B0 lane row names it for reconciliation. Its `implemented` status is correct and confined to its exact binding; no promotion.

`AGENT-018` — matrix.tsv line 28: `planned`, gate `core-demo`, owner `backend`. Binding: `future H0 fake Agent ToolGateway and Plugin executor conformance fixtures`. The current protocol/fake tests do not prove framework/harness replacement across unchanged Plugin package/executor boundaries: the composition test uses one test-local `FakeToolGateway` and `FakePluginExecutor`, and no fixture swaps an Agent implementation while holding the same protocol/resolver/gateway/executor counterparts constant. Direct protocol-crate consumers are `agent-runtime`, `platform-core` and `ustc-agentd`; production resolver and fixed-adapter code also imports the protocol types. That broader consumer set does not supply the missing replacement-conformance fixture, so `planned` remains correct.

`AGENT-019` — matrix.tsv line 29: `implemented`, gate `pr`, owner `backend`. Binding: `cargo test --locked -p ustc-agentd --test tool_gateway_conformance && cargo test --locked -p ustc-agent-tool-protocol`. Owning contract `docs/contracts/agent-plugin-boundary.md` §11 lists `AGENT-019` as implemented. The composition test proves frozen provider definitions bind one private route, and unknown tools (`ToolNotProjected`), malformed arguments (`ArgumentsInvalid`), route mismatch (`DispatchIdentityMismatch`), current denial (`EmergencyBlocked`) and projection mismatch reach no executor, while success returns one correlated result. This proof is not promoted to production `ToolGateway`: the `FakeToolGateway` is test-local, no production `ToolGateway` type exists, and the test uses an in-memory `FakePluginExecutor` with no durable intent/receipt ordering. `implemented` status is correct at its exact frozen-view/private-route/deny-side/fake-executor binding.

`AGENT-003` — `docs/acceptance/platform-baseline.md` line 268: "tool side effect persists intent and receipt before advancing state", binding `rust-integration`, gate `integration`. Absent from `matrix.tsv` (which contains only `AGENT-001`, `AGENT-002`, `AGENT-017`, `AGENT-018`, `AGENT-019` at lines 25–29). Catalog-only/non-admitted/non-pass. Not promoted.

`AGENT-004` — platform-baseline.md line 269: "crash/resume cannot duplicate a committed tool side effect", binding `rust-integration`, gate `release`. Absent from `matrix.tsv`. Catalog-only/non-admitted/non-pass. Not promoted.

`AGENT-009` — platform-baseline.md line 274: "policy and grant checks execute before every tool side effect", binding `rust-integration`, gate `integration`. Absent from `matrix.tsv`. Catalog-only/non-admitted/non-pass. Not promoted.

`AGENT-010` — platform-baseline.md line 275: "dynamic MCP tools use the exact approved schema snapshot in the run spec", binding `rust-integration`, gate `integration`. Absent from `matrix.tsv`. Catalog-only/non-admitted/non-pass. Not promoted.

`AGENT-011` — platform-baseline.md line 276: "prompt/tool payload telemetry is off by default and all diagnostics redact secrets", binding `rust-integration`, gate `release`. Absent from `matrix.tsv`. Catalog-only/non-admitted/non-pass. Not promoted.

`AGENT-012` — platform-baseline.md line 277: "Rig/rmcp remain replaceable behind owned ports without changing run semantics", binding `external-conformance`, gate `release`. Absent from `matrix.tsv`. Catalog-only/non-admitted/non-pass. Not promoted.

`AGENT-013` — platform-baseline.md line 278: "Observer/Transformer/Gate/Registry event semantics and order are typed and deterministic", binding `rust-unit`, gate `PR`. Absent from `matrix.tsv`. Catalog-only/non-admitted/non-pass. Not promoted.

### Owned-path and dependency drift

- Protocol crate `crates/agent-tool-protocol/Cargo.toml` line 13: only dependency `sha2` (workspace). No Market/Plugin/adapter/framework/transport dependency. Matches M40 ownership of provider-neutral canonical values/envelopes and the `agent-plugin-boundary.md` §6.1 rule that the protocol crate contains only wire/domain-neutral values.
- Resolver `crates/platform-core/src/invocation.rs`: re-exports `TenantId`/`UserId` from M00 `crate::identity` (line 6); imports `AgentTool`, `AgentToolDefinition`, `AgentToolsetView`, `ProjectionSnapshotId`, `ProtocolConstructionError`, `ProtocolRunId`, `ProtocolTurnId`, `ToolRouteRef`, `is_valid_tool_name` and canonical types from `ustc_agent_tool_protocol` (lines 10–18); uses `semver::Version` (line 79). This is the M20-owned pure resolver consuming M00 identity and the protocol seam, matching `B-M20-M40-PROJECTION` (partial) and `B-M20-M30-TOOLSET` (partial) in `docs/contracts/module-boundaries.md` lines 35–37. `ToolProjectionSnapshot::agent_toolset_view` (lines 507–525) is the public mapping from M20 authority into the M40-consumed `AgentToolsetView`.
- Composition test `apps/ustc-agentd/tests/tool_gateway_conformance.rs`: imports from `ustc_agent_tool_protocol` (lines 4–9), `ustc_campus_agent_core::invocation` (lines 10–14) and `ustc_campus_agent_runtime` (line 15). This is the declared composition test surface (`apps/ustc-agentd/tests` per module-boundaries.md §3), using only public boundaries; no private-field reach-through.
- `apps/ustc-agentd/Cargo.toml` lines 13–22: depends on `agent-runtime`, `agent-tool-protocol`, `platform-core`, `application-ingress`, `client-protocol`, `affairs-navigator`, `change-radar`, `course-planning`, `opportunity-graph`. Composition-root dependency direction; `M10`-owned `client-protocol` is consumed but `ustc-agentd` does not depend on an M80 client-core. Matches the module-map dependency direction (`ustc-agentd composition ──► M40 Tool Gateway and Execution`; `M40 ├── authority query ──► M20`).
- Accepted-boundary defect found: `OpportunityInvocationSpine` resolves/rechecks authority and records M30 intent/receipt ordering, but then executes a `NativeRustComponent` with tenant-private read/write capabilities inside `ustc-agentd`. This contradicts `agent-plugin-boundary.md` §4, which limits the in-process bootstrap to `PublicRead` with no external side effect and keeps writes/native executors out of process. No dependency cycle was found; the blocker is the executor/capability boundary, not graph direction.

### Public-boundary/status drift

- `docs/plan/modules/50-tool-gateway-execution.md`: `Implementation State: partial-evidence` (line 7). Status line 6 projects protocol/fake conformance plus bounded Affairs and ChangeRadar fixed-adapter compositions only. It does not admit the Opportunity in-process tenant-private path; current source therefore exceeds this projection and requires amendment. The unfinished MVP items remain unimplemented and are not promoted.
- `docs/plan/modules/00-module-map.md` line 23 likewise projects only Affairs and ChangeRadar fixed compositions and keeps the durable generic gateway/out-of-process executor host planned. The Opportunity write path is not covered by that current-state claim.
- `docs/contracts/agent-plugin-boundary.md` §11 lists the bounded Affairs product path under Implemented now and keeps generic/package-portable gateway plus out-of-process executor host planned. §4 explicitly limits the fixed in-process bootstrap to `PublicRead`/no external side effect. The Opportunity tenant-private read/write path is absent from §11 and conflicts with §4; the contract and source do not currently match.
- `docs/contracts/module-boundaries.md` lines 35–41: `B-M20-M40-PROJECTION` partial; `B-M30-M40-CALL` protocol/fake proof implemented, production planned; `B-M40-M30-RESULT` protocol/fake proof implemented, production planned; `B-M40-M51-EXEC` planned. Matches.
- `docs/tasks/01-execution-roadmap.md` line 104 is stale: it still presents the Opportunity owning adapter as current M40 evidence, while the owning M40 plan and agent-plugin contract admit only Affairs/ChangeRadar fixed compositions and classify the Opportunity in-process tenant-private path as a boundary conflict. This audit does not ratify that row; W1 closeout must remove or reclassify Opportunity from current M40 evidence without selecting contract-versus-implementation direction.
- `README.md` line 9: "已建立 framework-neutral Agent runtime kernel、typed invocation resolver 与 `agent-tool-protocol/v0` 的 executable evidence". Matches.
- `docs/coverage-matrix.md` line 34: M40 acceptance projection `active:AGENT-*`; `active:MARKET-*`; `active:PKG-*`. `AGENT-017`/`AGENT-019` are active implemented, `AGENT-018` is active planned, `AGENT-003`/`004`/`009`/`010`/`011`/`012`/`013` are long-horizon only (in `platform-baseline.md`, not `matrix.tsv`). Matches.
- `docs/acceptance/matrix.tsv` lines 25–29: `AGENT-001`/`002`/`017`/`019` implemented, `AGENT-018` planned. Matches.
- The prior ADOPT/no-contradiction conclusion was an overstatement and is corrected here to AMEND/paused. `partial-evidence` and all current acceptance-row statuses remain unchanged; no `StandaloneReady`, `IntegrationReady`, `Integrated` or `Accepted` claim is made for M40.

### Targeted verification

Run-owned target/temp state outside the repository: `CARGO_TARGET_DIR=/home/pwh/uca-runs/uca-w1-m40-b0-d2e532-v6/scratch/cargo-target`, `PYTHONPYCACHEPREFIX=/home/pwh/uca-runs/uca-w1-m40-b0-d2e532-v6/scratch/pycache`.

1. `CARGO_TARGET_DIR="$CARGO_TARGET_DIR" cargo test --locked -p ustc-agent-tool-protocol` → exit `0`; `test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out` (tests: `frozen_view_sorts_definitions_and_binds_private_route`, `duplicate_names_and_routes_fail_closed`, `unknown_tool_never_produces_a_call`, `result_is_correlated_without_plugin_identity`).
2. `CARGO_TARGET_DIR="$CARGO_TARGET_DIR" cargo test --locked -p ustc-agentd --test tool_gateway_conformance` → exit `0`; `test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out` (tests: `provider_view_to_gateway_to_executor_is_correlated_and_authorized`, `invalid_or_stale_calls_never_execute`).
3. `PYTHONPYCACHEPREFIX="$PYTHONPYCACHEPREFIX" python3 scripts/check_repo_contracts.py` → exit `0`; stdout `contract-check: PASS`.

Final repository-state checks (after the taskbook edit):

4. `git diff --check` → clean (no whitespace errors).
5. `git status --short` → only `M docs/tasks/campaign-w1-m40-b0.md`.
6. `git diff --name-only` → `docs/tasks/campaign-w1-m40-b0.md`.
7. `git ls-files --others --exclude-standard` → empty (no untracked files introduced).

### Non-claims and next gate

Non-claims:
- no claim that production `ToolGateway`, `PreparedToolExecution`, `PluginExecutionRequest`/`PluginExecutionOutcome`, durable intent/receipt ordering, duplicate/restart recovery, output-boundary, or real admitted executor exists;
- no claim that `AGENT-018` or any catalog-only/non-admitted `AGENT-*` case has passed or become admitted;
- no claim that M40 is `StandaloneReady`, `IntegrationReady`, `Integrated` or `Accepted`;
- no claim that the test-local `FakeToolGateway`/`FakePluginExecutor` is a production gateway or public M40 implementation;
- no public protocol, execution ordering, executor, authority, lifecycle, acceptance or runtime behavior is changed by this audit-only taskbook correction;
- the OpenCode worker performed no staging, commit, push, tag, PR, merge, release, deployment, credential or external side effect; Hermes parent source-control receipts are separately bound to PR #58 and are not evidence for ADOPT.

Next gate: focused exact-delta review of the source-confirmed PR #58 corrections, replacement exact-head CI, semantic replies plus resolution of both review threads, and a race-pinned merge of this audit as AMEND/paused. After merge the W1 M40-B0 lane admits no implementation or contract mutation; Develata must authorize a new contract-bound slice to choose and execute the Opportunity contract-versus-implementation repair.
