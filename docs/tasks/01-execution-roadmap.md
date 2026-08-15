# Module assembly roadmap

## Metadata

- `Status`: Current delivery and task-splitting order
- `Version`: `module-roadmap/v2.6`
- `Last Review`: `2026-08-15`
- `Owning product plan`: [`../plan/02-product-positioning.md`](../plan/02-product-positioning.md)
- `Engineering constitution`: [`../plan/00-engineering-constitution.md`](../plan/00-engineering-constitution.md)
- `Module map`: [`../plan/modules/00-module-map.md`](../plan/modules/00-module-map.md)
- `Work policy`: [`00-module-work-policy.md`](00-module-work-policy.md)
- `Boundary registry`: [`../contracts/module-boundaries.md`](../contracts/module-boundaries.md)
- `Acceptance registry`: [`../acceptance/matrix.tsv`](../acceptance/matrix.tsv)

This document schedules independent large modules and their small-module batches. It does not override the owning plans or contracts.

## 1. Current stance

The module skeleton review is complete. Concrete implementation remains contract- and acceptance-gated. Existing code is retained as executable evidence:

- `M20`: typed package/catalog, capability-registry, bounded managed-installation aggregate/in-memory repository, bounded reviewed-grant aggregate/replay/semantic in-memory repository, bounded package-update aggregate/semantic in-memory repository, and pure invocation-resolver evidence;
- `M30`: node-local `AgentRun` kernel;
- `M40`: Agent tool protocol and fake gateway/executor proof;
- `M72`: offline Course Planning spike;
- `M90`: CI and repository contract checker.

Before any of these grows, its owner compares current code with the new module blueprint and records `adopt | amend | retain as spike | remove`. Documentation alone does not promote any planned acceptance row.

Contract/fixture-only root scaffolding may begin after the completed S0 review, subject to the owning module's contract-ready gate. M80 client-core/CLI/inbound-MCP retained work starts only under exact batches bound to planned `CLIENT-007`–`CLIENT-010`; these rows now admit proposal/implementation slicing but do not prove any artifact. A minimal Dioxus initialization before active Fullstack/Web/deployment bindings exists only as an explicitly disposable, non-mergeable spike; retained server/Web/Android scaffold work still waits for exact `WEB-*`/deployment rows and future bindings in `matrix.tsv`. Neither form may pre-implement product logic.

<!-- AUTONOMOUS_CAMPAIGN_GRANT:BEGIN -->
### Active autonomous module campaign authorization

- `Campaign ID`: `USTC-MODULES-2026-07-W1`
- `Grantor`: Develata
- `Status`: `active`
- `Approved base`: `b7911859454e659b2fd426ac475958a22b92e5a8`
- `Controller and sole merge authority`: Deve Hermes
- `Allowed remote operations`: create/update non-protected feature branches, push exact reviewed commits, create/update Draft PRs for proposal-only rows, create/update reviewable PRs for audit-only rows, and merge only the auto-merge-admitted rows below after every required gate succeeds
- `Execution default`: Dongfengyun fresh exact-`main` checkouts; never reset or repurpose a foreign, dirty or local-ahead checkout
- `Concurrency`: at most two implementation workers plus one read-only audit worker; shared governance carriers and full Rust gates are serialized or use one declared fan-in owner
- `Mutable state authority`: the exact taskbook named in each row owns lane status, bound source, repair round, blocker identity, stop reason and next mutation; the controller reads it back before every mutation, while PR bodies carry discovery/evidence links only
- `Mutable state discovery`: for a `queued` lane read the protected-`main` taskbook; after the first push require exactly one open PR whose body carries `Campaign-ID: USTC-MODULES-2026-07-W1`, the exact `Campaign-Lane`, and `Taskbook-Commit` equal to that PR head, then read the taskbook from that head; zero or multiple matches pause the lane
- `Repair-round state`: each lane starts at `0`; its canonical taskbook records every bounded repair-and-review round, and round `2` with the same blocker or required-gate failure pauses that lane
- `Current stop reason`: `none`
- `Completion`: this grant ends when all four rows below are merged, explicitly paused/rejected, or superseded by operation-specific Develata instruction; it does not roll into another batch automatically
- `Revocation`: any direct Develata pause/revoke instruction takes precedence immediately; changing the recorded status or scope requires operation-specific approval
- `Review trigger`: rebind and re-evaluate after every exact-main post-merge CI run; continue without a new prompt only while no observable stop condition from the work policy is present

#### Required ordered gates for each W1 candidate

1. read back the row's canonical taskbook and bind a fresh clean checkout to one 40-hex source commit;
2. run `python3 scripts/check_repo_contracts.py`, `python3 -m unittest discover -s scripts/tests -p 'test_*.py'`, and `git diff --check <bound-source>...HEAD` on the candidate;
3. bind one independent blocker review to the exact commit, changed-path set and outgoing range;
4. push that reviewed commit and create/update the row's admitted Draft or reviewable PR;
5. require exact-head GitHub contexts `rust` and `docs-and-contracts` to succeed before merge; a failed-head repair first updates the taskbook round and repeats steps 1–4 for the replacement commit;
6. for an admitted merge, execute the unchanged-live-`main` prospective-tree proof from the work policy and verify exact-main post-merge CI before another lane mutates.

| Lane | Mode and evidence binding | Auto-merge boundary | Finite allowed paths |
|---|---|---|---|
| `M00-B3` | proposal-only; acceptance IDs are intentionally absent, and the packet must propose exact planned IDs/future bindings before implementation admission | no contract acceptance or retained implementation; pause for Develata before settling actor, policy or admission authority | `docs/tasks/campaign-w1-m00-b3.md`; `docs/plan/modules/10-platform-control-identity.md`; `docs/contracts/platform-session.md`; `docs/acceptance/matrix.tsv`; ordinary non-grant sections of `docs/tasks/01-execution-roadmap.md` |
| `M20-B6` | proposal-only; reconcile planned `MARKET-004` and `PKG-020` with exact update/rollback command, state, error, event and future-binding evidence | no contract acceptance or retained implementation; pause for Develata before settling permission expansion, rollback or lifecycle states | `docs/tasks/campaign-w1-m20-b6.md`; `docs/plan/modules/30-market-package-lifecycle.md`; `docs/contracts/market-lifecycle.md`; `docs/acceptance/matrix.tsv`; ordinary non-grant sections of `docs/tasks/01-execution-roadmap.md` |
| `M30-B0` | audit-only; reconcile matrix-planned `HARNESS-001` and `HARNESS-003` plus catalog-only, non-admitted `HARNESS-002`, then record one evidence-bound disposition without readiness promotion | auto-merge only when it changes no acceptance posture, Agent/Harness lifecycle or runtime state-machine behavior | `docs/tasks/campaign-w1-m30-b0.md`; `docs/plan/modules/40-agent-harness-runtime.md`; ordinary non-grant sections of `docs/tasks/01-execution-roadmap.md` |
| `M40-B0` | audit-only; reconcile matrix-implemented `AGENT-017`, matrix-planned `AGENT-018`, and catalog-only, non-admitted `AGENT-003`, `AGENT-004`, `AGENT-009`, `AGENT-010`, `AGENT-011`, `AGENT-012`, `AGENT-013` | auto-merge only when it changes no acceptance posture, public protocol, execution ordering or executor behavior | `docs/tasks/campaign-w1-m40-b0.md`; `docs/plan/modules/50-tool-gateway-execution.md`; ordinary non-grant sections of `docs/tasks/01-execution-roadmap.md` |

Campaign-authorized work MUST NOT alter this grant block, the campaign-authorization policy block, their checker/digest/mutation tests, root authorization projections, `.github/workflows/`, `.github/CODEOWNERS`, branch protection or collaborator settings. It also excludes tags/releases, public visibility/publication, real-source credentials or protected fixtures, production deployment/infrastructure mutation and actual M80 presentation design.

The controller pauses before further mutation upon any user-choice request; reviewer disagreement on public behavior/authority not resolved by contract; debugging that needs live interaction, unplanned instrumentation/data, or out-of-scope paths; the same blocker/gate failure after two recorded rounds; or inability to prove a mechanical prospective merge tree against unchanged live `main`. This grant does not approve a new product topology, authority owner, permission semantic, lifecycle state machine, runtime state machine or protocol behavior.
<!-- AUTONOMOUS_CAMPAIGN_GRANT:END -->

## 2. Assembly shape

```text
Foundation contracts
├── M00 Platform Control/Identity
├── M10 Application Ingress Host
├── M90 Infrastructure/Operations
└── M80 client-core with peer Dioxus/ustc-agent/inbound-MCP adapters may develop against fake M10

Independent runtime/market lanes
├── M20 Market/Package Lifecycle
├── M30 Agent Harness/Runtime
├── M40 Tool Gateway/Execution
├── M50 Model Provider
└── M51 MCP Binding/Executor

Shared campus fact lane
└── M60 Campus Trust/Source Pipeline
      ├── M70 ChangeRadar
      ├── M71 Affairs Navigator
      └── M72 Opportunity Graph
```

A missing dependency is replaced by an equal-contract fake during standalone work. Real attachment happens only at the declared assembly gate.

## 3. Module lane registry

| Module | State key | Current state | Current module target | Owner | Merge gate |
|---|---|---|---|---|---|
| `M00` Platform Control/Identity | `partial-evidence` | identity-types and session-domain implemented; request-context/ports planned | stable IDs, request/session context and fake ports | unassigned | admitted/denied API request proof |
| `M10` Application Ingress Host | `skeleton` | skeleton | Dioxus server-function plus explicit `ustc-agent`/inbound-MCP HTTP/SSE route, DTO/error/event/compatibility host | unassigned | black-box peer-client HTTP/stream conformance and no reach-through |
| `M20` Market/Package | `partial-evidence` | typed package/catalog + capability-registry + bounded managed-installation fake + bounded reviewed-grant aggregate/replay/semantic repository + bounded package-update aggregate/semantic repository + pure resolver evidence | M10/M80 browse delivery + durable installation/grant/update adapters, artifact switching and B7 composition around audited resolver | unassigned | `MARKET-*` current-scope rows |
| `M30` Agent Harness/Runtime | `partial-evidence` | node kernel only | finite harness/graph/context/review against fakes | unassigned | `HARNESS-*` + owned `AGENT-*` |
| `M40` Tool Gateway/Execution | `partial-evidence` | protocol/fake proof | durable intent/executor/receipt composition | unassigned | `AGENT-018/019`, `MARKET-007` |
| `M50` Model Provider | `planned` | planned | typed profiles + one provider adapter | unassigned | provider conformance + real bounded turn |
| `M51` MCP Binding/Executor | `planned` | planned | one reviewed read-only remote binding | unassigned | MCP lifecycle/security/executor proof |
| `M60` Campus Trust/Source | `planned` | `source-import/v1` and `source-retrieval/v0` accepted contract authority under R11 per `ACCEPT_EXACT_M60_B2_R11_PACKET` (2026-08-13); bounded B1 under `source-import/v0` (P1-1); no v1 or B2 implementation; lifecycle precondition applies; superseded V10 `DEC-M60-B2-ACCEPTANCE` is historical evidence only | one reviewed source/revision/baseline | unassigned | `SRC-*` current-scope rows |
| `M70` ChangeRadar | `design-only` | design only | one semantic change + feed | unassigned | `RADAR-*` current-scope rows |
| `M71` Affairs Navigator | `design-only` | design only | one reviewed procedure board | unassigned | `PROC-*` current-scope rows |
| `M72` Opportunity Graph | `bounded-spike` | planner spike | honest source/profile/Market integration | unassigned | `COURSE-*` current-scope rows |
| `M80` Client Core and Interaction Shells | `planned` | no code | framework-neutral client core plus peer `ustc-agent`, inbound MCP and required Dioxus Web/Android journeys | client/core/CLI/MCP unassigned; Kimi K3 + Claude Opus 5 lead Windows UI/design; GPT review/local optimization | `CLIENT-007`–`CLIENT-010` for retained headless slices; exact active `WEB-*`/deployment rows plus Web and Android passing for Dioxus |
| `M90` Infrastructure | `governance-baseline` | CI only | config/store/journal/evidence + Docker Compose Fullstack restore profile | unassigned | Compose Web/Android target profile restore/read-back |

Team assignment updates only the `Owner` cells and issue links. It does not change module ownership semantics.

## 4. S0 — Architecture and interface freeze

**Status**: complete; implementation remains acceptance-gated.

### `S0-1` Constitution and module registry

**Status**: complete.

- adopt full engineering constitution and mandatory work loop;
- define large-module ownership and dependency direction;
- define one M80 framework-neutral typed client core with peer Dioxus Web/Android, `ustc-agent` user/automation and inbound MCP adapters; keep `ustc-agentctl` operator-only, forbid GUI→CLI and keep M10 ingress explicit;
- classify existing code as partial evidence.

### `S0-2` Boundary and task contracts

**Status**: complete.

- register every cross-module public boundary;
- bind fakes/conformance expectations;
- split large modules into small-module batches;
- map plans/contracts/features/acceptance/tasks;
- add a checker that verifies module-map ID/status ↔ blueprint metadata ↔ coverage row ↔ roadmap lane ↔ active/long-horizon acceptance wording.

### `S0-3` Team review

**Status**: complete.

- distribute architecture brief;
- record `Accept | ConditionalAccept | Reject` by module/skeleton decision;
- conditional acceptance names owner, evidence and exit condition;
- update formal docs for accepted corrections;
- no false team consensus is inferred from one member's silence.
- record the complete three-lane decision set in [`02-s0-architecture-review.md`](02-s0-architecture-review.md).

**Exit gate**: satisfied; the recorded review found no unresolved ownership cycle, second authority, UI computation path or cross-module private dependency in the accepted skeleton.

## 5. M00 lane — Platform Control and Identity

- `M00-B1 identity-types`: the six bounded tenant/user/session/request/command/correlation values and their shared construction error under `platform-identity/v0`; converge the invocation resolver onto the M00 tenant/user definitions only.
- `M00-B2 session-domain`: open/refresh/expire/revoke transitions and replay under accepted [`platform-session/v0`](../contracts/platform-session.md). Implemented as a pure kernel in `crates/platform-core/src/session.rs` with evidence in `crates/platform-core/tests/platform_session.rs`; `AUTH-017`, `AUTH-018`, `AUTH-019` and `AUTH-020` are `implemented`, two of them partly through private library-target fixtures for the reason that contract's §17 records. The batch added no dependency, port, repository, clock, request context or M10 integration.
- `M00-B3 request-context`: land `policy-reference` first, then immutable admitted actor/request/command/causation context and duplicate/conflict semantics as separate reviewable commits.
- `M00-B4 ports-and-fakes`: land `session-port` and `control-evidence` as separate reviewable commits, then clock/session/audit/secret-ref fakes with failure fixtures.
- `M00-B5 api-admission-integration`: attach to `M10`; one denied request reaches no downstream fake.

These five roadmap batches schedule the six small modules in the M00 blueprint; they are not alternate module names. `identity-types` maps to B1, `session-domain` to B2, `policy-reference` plus `request-context` to B3, and `session-port` plus `control-evidence` to B4. Each small module still receives its own commit and standalone evidence before B5 composition.

Current completion scope excludes production USTC/CAS login. A labelled demo/auth adapter is sufficient if the boundary is honest.

## 6. M10 lane — Application Ingress Host

- `M10-B1 ingress-registry`: server-function plus explicit `ustc-agent`/inbound-MCP route/version/error/event registry, DTO rules and the framework-neutral M10-owned `client-protocol` carrier.
- `M10-B2 request-admission`: bounds, `M00` actor mapping, client build/protocol compatibility and preconditions.
- `M10-B3 server-function-adapter`: Axum-compatible first-party ingress with dependency reach-through checks.
- `M10-B4 dispatch-and-errors`: one ingress maps to one owned application operation and stable result/error.
- `M10-B5 event-stream`: monotone typed stream/SSE cursor, reconnect and backpressure.
- `M10-B6 server-lifecycle`: Dioxus SSR/assets/ingress attachment, preflight, readiness, graceful drain and black-box tests.
- `M10-B7 client-contract`: freeze the shared Dioxus/CLI/MCP semantic subset plus supported-version/upgrade behavior required by `M80`.
- `M10-B8 user-integration-http-adapter`: implement the explicit REST/SSE subset consumed by real `ustc-agent` and inbound MCP adapters; no generic arbitrary-operation endpoint.

Handlers contain mapping and coordination only. Domain validation remains in owning modules. Dioxus, user CLI and inbound MCP are transport peers over the same admitted application operations; the inbound MCP adapter is not M51.

## 7. M20 lane — Market and Package Lifecycle

- `M20-B0 existing-resolver-audit`: compare `invocation.rs`/fixtures with the module blueprint; adopt/amend/spike decision.
- `M20-B1 package-catalog`: schema, catalog publication and anonymous read model.
- `M20-B2 capability-registry`: risk/data class and auto-grant eligibility. Bounded implementation evidence is complete; it creates no grants and promotes no acceptance row.
- `M20-B3 installation-domain`: exact install/configure/enable/disable/revoke/uninstall. The bounded first slice `M20-B3-s1` implements a pure managed-installation aggregate plus a semantic in-memory repository fake under `platform-core`; it mints no production enable evidence, creates no durable state and promotes no acceptance row.
- `M20-B4 grant-domain`: scope/version/reapproval and tenant checks. The exact public contract and bounded Rust implementation are complete under `crate::market::grant`: pure grant aggregate, explicit admission evidence, decide/evolve/replay, deny-side resolver projection and semantic in-memory repository fake. This supporting evidence mints no production grant, promotes no acceptance row and leaves durable authority assembly/composition to `M20-B5`.
- `M20-B5 invocation-authority`: bounded implementation is complete under `crate::market::authority`: one semantic carrier-by-carrier read transaction, service-owned candidate/current assembly, shared call preflight, adopted resolver/recheck and post-success precondition verification. It creates no durable authority, production grant/enable evidence, effect intent or acceptance promotion.
- `M20-B6 update-rollback`: bounded evidence complete for staged update, permission expansion, exact rollback and atomic in-memory package-update fake; no durable adapter, artifact switch, API/UI, current-call/in-flight composition or acceptance promotion.
- `M20-B7 composition`: future attachment of read/mutation APIs, durable adapters/current API calls and fake `M40` consumer.

`M20` merge scope is complete only when browse and current lifecycle state are distinct and disable/revoke blocks discovery/calls. Historical `B1-0`/`B1-1` labels map to the lifecycle-contract establishment and `M20-B1` respectively; the earlier `B1-2`/`B1-3`/… sequence is superseded by the canonical `M20-B<n>`/slice references above.

## 8. M30 lane — Agent Harness and Runtime

- `M30-B0 existing-kernel-audit`: map `agent-runtime` to `run-spec`/`agent-run`; do not extend before decision.
- `M30-B1 harness-run`: finite user-task phases, suspension and terminal reconciliation.
- `M30-B2 task-contract`: immutable goal/non-goals/deliverables/acceptance.
- `M30-B3 task-graph`: finite graph validation, resources and revisions.
- `M30-B4 context-budget`: complete-request integer preflight.
- `M30-B5 context-projection`: bounded deterministic offload/compaction/compression artifacts.
- `M30-B6 scheduler-supervisor`: event-driven ready nodes and crash/restart identity.
- `M30-B7 evidence-review`: fresh review, dispositions and bounded remediation.
- `M30-B8 ports-fakes`: fake `M50`, `M40`, journal, artifact and clock.
- `M30-B9 run-projection`: safe `M10`/`M80` state and events.

`M30` standalone acceptance uses only fakes. Real provider/tool integration is a later assembly commit, not a reason to couple implementations.

## 9. M40 lane — Tool Gateway and Execution

- `M40-B0 protocol-audit`: compare current protocol/conformance with the module blueprint.
- `M40-B1 private-route`: exact route table and frozen call normalization.
- `M40-B2 current-authorization`: `M20` recheck mapping and denial ordering.
- `M40-B3 execution-stages`: prepare/execute/result stages; composition interleaves separate public `M30` intent/receipt commands without a module dependency cycle.
- `M40-B4 executor-port`: bounded request/outcome and fake executor.
- `M40-B5 output-boundary`: untrusted content/artifact/schema/size/redaction.
- `M40-B6 recovery`: duplicate, conflicting duplicate, timeout, cancel and receipt reconciliation.
- `M40-B7 composition`: durable fake/real read-only executor path at `ustc-agentd`.

No denied call reaches an executor. No executor success becomes an Agent result before receipt evidence.

## 10. M50 lane — Model Provider Integration

- `M50-B1 provider-profile`: `OfficialCentral`/`UserCloud`, fixed origin/model/secret-ref/capability snapshot.
- `M50-B2 model-port`: complete request, ordered events, final/usage/error contract.
- `M50-B3 estimator`: validated context limit and compatible complete-request estimator.
- `M50-B4 stream-normalizer`: stream/final parity, timeout/cancel/error mapping.
- `M50-B5 safety-redaction`: endpoint/redirect/DNS/TLS and secret/log rules.
- `M50-B6 provider-peer`: first reviewed provider adapter and cassette/fake conformance.
- `M50-B7 M30-integration`: one bounded model turn, no fallback.

A second provider is a peer adapter, not branches inside `M30`.

## 11. M51 lane — MCP Binding and Executor

- `M51-B1 binding-domain`: identity/owner/component and legal lifecycle.
- `M51-B2 endpoint-policy`: URL/DNS/IP/redirect/auth-discovery SSRF checks.
- `M51-B3 transport`: released Streamable HTTP initialization/session.
- `M51-B4 discovery-schema`: bounded pagination, inventory and schema digests.
- `M51-B5 review-drift`: approve/active/quarantine and tool/schema change handling.
- `M51-B6 executor-output`: `M40` request/outcome, timeout/cancel/content bounds.
- `M51-B7 conformance`: fake hostile server fixtures.
- `M51-B8 M40-integration`: one reviewed read-only tool with exact grant/receipt.

Arbitrary central `stdio` command execution is outside this lane.

## 12. M60 lane — Campus Trust and Source Pipeline

**Status**: accepted-contract `source-import/v1` and `source-retrieval/v0` (R11 two-layer M60/M90 transport architecture; `ACCEPT_EXACT_M60_B2_R11_PACKET`); M60 overall remains `planned`; no v1 Rust implementation; operational `Suspended`/`Revoked` lifecycle and `SourceAuthorityRevision` precondition enforced; no approved concrete USTC source; superseded V10 `DEC-M60-B2-ACCEPTANCE` is historical evidence only. Bounded `M60-B1 source-registry` implemented under `source-import/v0` (P1-1).

**Prerequisite**: one concrete public source receives owner, URL/retrieval, permission/rate and parser-fixture review.

`P1-0` now carries the accepted, construction-ready `source-import/v0` contract and a concrete `ustc-teach-calendar-fall` review candidate under [`p1-source-revision-readiness-proposal.md`](p1-source-revision-readiness-proposal.md). The source remains `Proposed`; raw HTML remains outside Git; no acceptance row or module state is promoted by P1-0. Exact-candidate review is `GO`, so the bounded local P1-1 B1 implementation lane is admitted. This is not part of the older W1 campaign grant. Develata's later operation-specific instructions authorize the feature-branch push, PR, Actions CI/run and workflow dispatch; merge, tag, release, source approval and retrieval remain unauthorized.

- `M60-B1 source-registry`: stable identity, owner, policy and status; exact P1-1 boundary is `source-import/v0` §§3–7 and the P1 proposal packet.
- `M60-B2 retrieval-policy`: safe exact host/path, redirects, content/time/size/rate. Contract accepted (`source-import/v1` + `source-retrieval/v0`, R11 exact packet); no implementation; lifecycle precondition applies.
- `M60-B3 lease-snapshot`: deterministic lease and immutable raw evidence.
- `M60-B4 normalize-parser`: deterministic peers with exact identity/fixtures.
- `M60-B5 source-revision`: observed/published/effective time, digests and provenance.
- `M60-B6 conflict-freshness`: authority, stale and highest-authority conflict.
- `M60-B7 baseline`: atomic accepted baseline and failure injection.
- `M60-B8 publication-port`: typed product candidate/evidence boundary.

This lane precedes real product source integration, but each product can develop against exact M60 fixtures.

`P1-1` has implemented bounded `M60-B1 source-registry` under `source-import/v0`: `crates/platform-core/src/source_registry.rs` and `crates/platform-core/tests/source_registry.rs` are present, `SRC-001` is promoted to `implemented` bound to `cargo test --locked -p ustc-campus-agent-core --test source_registry`, and the dedicated checker/test carriers are extended. `source-import/v1` and `source-retrieval/v0` are accepted contract authority under R11; no v1 Rust implementation exists. `M60` overall and `M60-B3` through `M60-B8` remain `planned`; `SRC-010`, `SRC-011` and `SRC-012` remain `planned`. The `ustc-teach-calendar-fall` candidate remains `Proposed`. The superseded V10 packet digest (`sha256:ba36425adc164ca9b3ec75addd4be2e4b299b5f8a8cfb75cf6a710679acd32ab`) is mechanically checked as historical evidence. Operational `Suspended`/`Revoked` lifecycle, source approval, network retrieval, and publication remain unauthorized.

## 13. First-party product lanes

The product implementation order remains:

```text
M70 ChangeRadar source/revision/diff foundation
→ M71 Affairs Navigator structured procedure entry
→ M70 ChangeRadar board feed completion
→ M72 Opportunity Graph consent/profile integration
```

### M70 — ChangeRadar

- `M70-B1 board-policy`;
- `M70-B2 semantic-diff`;
- `M70-B3 candidate-evidence`;
- `M70-B4 review-event`;
- `M70-B5 deterministic-feed`;
- `M70-B6 M60/M10/M80 integration`.

### M71 — Affairs Navigator

- `M71-B1 tree-and-stable-ids`;
- `M71-B2 board-policy-and-draft`;
- `M71-B3 validation-and-supersession`;
- `M71-B4 review-publish-render`;
- `M71-B5 exact-structured-search`;
- `M71-B6 M60/M10/M80 integration`.

The first Affairs Navigator product slice follows the competition delivery posture (§18): administrator-imported reviewed snapshot → one typed procedure with bitemporal provenance (`valid_at` / `known_at`) plus review/verification metadata (`reviewed_at` / `last_verified_at`), with `as_of` bound as a query/answer cutoff ([`05-campus-trust-kernel.md`](../plan/05-campus-trust-kernel.md) §3.1) → one application query → one thin Web result with evidence/freshness/conflict/uncertainty. The `ustc-teach-calendar-fall` candidate family is the foundation for the reviewed snapshot path; this does not approve a concrete source, authorize network retrieval, or commit raw HTML. `PROC-010` is a planned active acceptance row and `WEB-009` is a planned catalog case for this slice.

### M72 — Opportunity Graph

- `M72-B0 planner-spike-audit`;
- `M72-B1 opportunity-types-validation`;
- `M72-B2 tenant-profile-consent-delete`;
- `M72-B3 qualification-dependency-conflict`;
- `M72-B4 bounded-candidate-and-independent-validation`;
- `M72-B5 course-pack-adapter`;
- `M72-B6 evidence-explanation`;
- `M72-B7 M20/M60/M10/M80 integration`.

The three packages keep independent versions, enable/disable and acceptance even when they share M60 facts.

## 14. M80 lane — Client Core and Interaction Shells

`CLIENT-007` through `CLIENT-010` now provide active planned future bindings for headless client slicing. They do not prove implementation. `M10-B1` first freezes the exact M10-owned operation/schema registry and `client-protocol` carrier; `M80-B1` consumes that exact carrier and its fake-M10 implements the same contract rather than inventing a client-owned protocol. `M80-B1`–`M80-B5` may become retained only under exact batch contracts and independent evidence against fake then real M10. Dioxus initialization remains disposable/non-mergeable until exact active `WEB-*` and deployment bindings admit the retained Web/Android scaffold. This M10/M80 lane runs alongside—and does not reorder—the first-party product lane in §13.

- `M80-B0 architecture-contract`: accepted typed peer-client and phased external-Agent amendment; Windows admitted later but not required; no code or implementation-status promotion.
- `M80-B1 client-contract-adoption`: consume the M10-owned operation/schema registry and `client-protocol` carrier without redefining wire or permission semantics; map validated shell intent into conforming request instances; freeze fake-M10 fixtures, per-adapter allowlists, schema-widening stale-grant behavior and unknown-version behavior.
- `M80-B2 client-core`: create the M80-owned core over `client-protocol` with endpoint/profile validation, auth port, query/command transport, correlation/idempotency, typed failure, cursor reconnect, cancellation and timeout reconciliation; M10 and outer-framework/backend implementations are forbidden dependencies.
- `M80-B3 peer-conformance`: adapter-independent fake-M10 normal/denial/conflict/reconnect/cancel/version-skew fixtures plus dependency/command/credential confinement.
- `M80-B4 ustc-agent-read-client`: create the cross-platform ordinary-user binary; prove `server.info`/`capability.list`, then one real `market.package.list` path with stable JSON/NDJSON, stderr and exit-class behavior; no operator command. Windows CLI is this same artifact family, not a parallel client authority.
- `M80-B5 inbound-mcp-read-adapter`: expose only public-read `market.package.list` through reviewed Streamable HTTP, client-core and M10 with exact schema digest, delegated caller/profile binding, bounds and no domain/operator/M51 reach-through; exact package/process placement freezes here. Campus and private operations are later slices after owning contracts.
- `M80-B6 dioxus-initialization`: revalidate and exact-pin Dioxus/DX; minimal server/Web/Android target features and toolchain smoke; no product/domain code.
- `M80-B7 dioxus-fullstack-contract`: generated query/command calls and typed event mapping over the same client semantics.
- `M80-B8 app-state`: deterministic thin presentation reducer including `UpgradeRequired`.
- `M80-B9 design-system`: accessible shared UI/form/display components.
- `M80-B10 market-run-journey`: one complete fake then real M10 graphical journey.
- `M80-B11 web-pwa`: page/assets, SSR/hydration or explicit CSR, responsive/accessibility/console/network proof.
- `M80-B12 compose-fullstack`: attach native server build to M90 Compose profile and prove restart/read-back.
- `M80-B13 android`: emulator plus real-device HTTPS/session/lifecycle/reconnect/Custom Tab/package proof.
- `M80-B14 version-skew`: supported older Android/CLI/MCP protocol plus typed unsupported-version rejection.
- `M80-B15 ios`: later peer target adapter/signing/device proof.
- `M80-B16 windows-desktop`: admitted later peer proposal; implementation begins only after a separate promotion amendment and active installer/signing/update, secure-session/login-callback, sleep/resume/proxy/reconnect and real-host evidence. Optional sidecar remains separately admitted; Windows is not in the current required-target gate.

Every truth-affecting calculation/mutation is re-evaluated by backend modules. Client-core and outer adapters only validate local shape, transport typed intent and reduce/render/serialize server projections. No peer shell spawns or parses another peer executable as its production path. `ustc-agentctl` remains outside M80, and inbound MCP remains opposite in direction from M51 outbound MCP execution. Automatic enrollment, registration, payment and other external-campus writes remain outside MVP; tenant-local drafts do not authorize external submission.

## 15. M90 lane — Infrastructure and Operations

- `M90-B1 typed-config-doctor`;
- `M90-B2 operational-store-transactions`;
- `M90-B3 journal-and-events`;
- `M90-B4 evidence-artifact-store`;
- `M90-B5 clock-scheduler-lease-queue`;
- `M90-B6 secret-ref-and-redaction`;
- `M90-B7 safe-http`;
- `M90-B8 telemetry-and-retention`;
- `M90-B9 migration-backup-restore`;
- `M90-B10 docker-compose-fullstack-profile-and-real-readback`;
- `M90-B11 CI/contracts/dependency gates`.

Start with one restorable demo deployment. Do not add multi-cloud/container orchestration symmetry first.

## 16. Cross-module assembly gates

### A0 — Root skeleton

`M00` admitted context + `M10` server-function/HTTP/event host + fake application modules + `M80` client-core conformance with one fake `ustc-agent`/inbound-MCP read path; Dioxus Web/Android assembly enters only under its own active bindings.

Prerequisite: exact active acceptance rows and future bindings exist for every retained M00/M10/M80 scaffold. `CLIENT-007`–`CLIENT-010` satisfy only the headless/client-core projection; a disposable Dioxus initialization spike cannot satisfy A0 or enter the module merge packet.

### A1 — Market/tool skeleton

`M20` exact projection/current denial + `M30` fake Agent proposal + `M40` fake executor ordering.

### A2 — Real bounded Agent path

`M30` finite run + `M50` first provider + `M40` one admitted read-only executor + durable `M90` journal/evidence.

### A3 — Shared source/product path

`M60` one reviewed source + one current first-party product candidate + `M10` API + `M80` display.

### A4 — Market-installed product path

One first-party package browses, installs, grants, enables, exposes an exact tool/use case, executes or queries through the correct module and can be disabled without breaking other modules.

### A5 — Three-product demonstration

All three exact package identities exist independently, use shared M60 facts, expose provenance/uncertainty and can be disabled/re-enabled independently.

### A6 — Delivery freeze

Security/privacy/source permission/license, adversarial failure, clean Docker Compose restore/read-back, real Web and Android clients and submission/release evidence pass. New scope stops.

## 17. Deferred

- arbitrary third-party hosted execution;
- generic federation or peer state authorities;
- universal workflow/graph platform;
- full-corpus RAG as a first-party truth path;
- private personalized ChangeRadar feeds;
- iOS/desktop completion before required Web/PWA + Android Fullstack proof;
- physical Market repository split before independent release need;
- public repository/download claims before release/public-readiness gates.

## 18. Competition delivery posture projection

This section projects the competition delivery posture from [`../plan/02-product-positioning.md`](../plan/02-product-positioning.md) §8 onto the current module lanes. It does not override owning plans, contracts or acceptance rows; it records the current honest per-lane posture.

| Module lane | Thin Slice | Validated Next | Deferred | Activation Trigger |
|---|---|---|---|---|
| `M00` Platform Control/Identity | identity-types + session-domain (implemented) | request-context + ports-and-fakes | full actor/policy admission composition | `M10` integration (`B5`) |
| `M10` Application Ingress Host | one server-function + HTTP/SSE route (planned/admitted architecture; skeleton-only at present, no active acceptance binding yet) | dispatch-and-errors + event-stream | full client-contract + version-skew | first Web adapter binding |
| `M20` Market/Package | typed catalog + capability registry + bounded in-memory installation/grant/update domains + bounded transaction-current authority assembly + pure resolver evidence (implemented as bounded fake/domain evidence; durable production adapters deferred) | durable installation/grant adapters | artifact switching + update/rollback composition | first production installation |
| `M30` Agent Harness/Runtime | node kernel (implemented) | finite harness/graph against fakes | scheduler-supervisor + real provider | first bounded Agent path (`A2`) |
| `M40` Tool Gateway/Execution | protocol/fake proof | durable intent/executor/receipt | full recovery composition | first real executor path (`A2`) |
| `M50` Model Provider | — | typed profiles + one provider adapter | multi-provider peers | first bounded model turn |
| `M51` MCP Binding/Executor | — | one reviewed read-only binding | outbound MCP productization | first external MCP tool |
| `M60` Campus Trust/Source | source-registry (`B1` implemented) | one reviewed source/revision | `B3`–`B8` pipeline | first approved concrete source |
| `M70` ChangeRadar | — | board-policy + semantic-diff | feed + RSS/Atom | `M60` source approval |
| `M71` Affairs Navigator | — | tree-and-stable-ids + board-policy | review-publish-render + search | `M60` source + `M10` query |
| `M72` Opportunity Graph | planner spike (bounded) | opportunity-types + profile-consent | course-pack + evidence-explanation | `M60` source + `M20` install |
| `M80` Client Core/Shells | narrow first-party Web adapter (planned/admitted architecture; no client-core or peer adapter implementation yet, no active acceptance binding yet) | client-contract-adoption + client-core | full Dioxus Web/Android + CLI + MCP | `M10` operation registry freeze |
| `M90` Infrastructure | CI + contract checker | typed-config + operational-store | Docker Compose restore + migration | first deployable slice |

A dash (`—`) means the lane has no current Thin Slice; it enters through its Validated Next. The narrow first-party Web adapter is admitted over one public application query once `M10` binds one server function and `M71` or `M70` produces one typed artifact. The Web shell renders typed server-owned state and captures intent only.
