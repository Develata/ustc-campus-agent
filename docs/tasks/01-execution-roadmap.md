# Module assembly roadmap

## Metadata

- `Status`: Current delivery and task-splitting order
- `Version`: `module-roadmap/v2.1`
- `Last Review`: `2026-07-25`
- `Owning product plan`: [`../plan/02-product-positioning.md`](../plan/02-product-positioning.md)
- `Engineering constitution`: [`../plan/00-engineering-constitution.md`](../plan/00-engineering-constitution.md)
- `Module map`: [`../plan/modules/00-module-map.md`](../plan/modules/00-module-map.md)
- `Work policy`: [`00-module-work-policy.md`](00-module-work-policy.md)
- `Boundary registry`: [`../contracts/module-boundaries.md`](../contracts/module-boundaries.md)
- `Acceptance registry`: [`../acceptance/matrix.tsv`](../acceptance/matrix.tsv)

This document schedules independent large modules and their small-module batches. It does not override the owning plans or contracts.

## 1. Current stance

The module skeleton review is complete. Concrete implementation remains contract- and acceptance-gated. Existing code is retained as executable evidence:

- `M20`: pure invocation resolver and fixtures;
- `M30`: node-local `AgentRun` kernel;
- `M40`: Agent tool protocol and fake gateway/executor proof;
- `M72`: offline Course Planning spike;
- `M90`: CI and repository contract checker.

Before any of these grows, its owner compares current code with the new module blueprint and records `adopt | amend | retain as spike | remove`. Documentation alone does not promote any planned acceptance row.

Contract/fixture-only root scaffolding may begin after the completed S0 review, subject to the owning module's contract-ready gate. A minimal Dioxus initialization before active Fullstack/API/client/deployment acceptance rows exists only as an explicitly disposable, non-mergeable spike; retained server/Web/Android scaffold work starts only after exact planned rows and future bindings are added to `matrix.tsv`. Neither form may pre-implement product logic.

## 2. Assembly shape

```text
Foundation contracts
├── M00 Platform Control/Identity
├── M10 Application Ingress Host
├── M90 Infrastructure/Operations
└── M80 Dioxus Fullstack Web/Android may develop against fake M10

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
| `M00` Platform Control/Identity | `planned` | planned | stable IDs, request/session context and fake ports | unassigned | admitted/denied API request proof |
| `M10` Application Ingress Host | `skeleton` | skeleton | Dioxus server-function/public route/DTO/error/event/compatibility host | unassigned | black-box Fullstack/HTTP/stream conformance and no reach-through |
| `M20` Market/Package | `partial-evidence` | partial evidence | browse + durable install/grant/disable/revoke around audited resolver | unassigned | `MARKET-*` current-scope rows |
| `M30` Agent Harness/Runtime | `partial-evidence` | node kernel only | finite harness/graph/context/review against fakes | unassigned | `HARNESS-*` + owned `AGENT-*` |
| `M40` Tool Gateway/Execution | `partial-evidence` | protocol/fake proof | durable intent/executor/receipt composition | unassigned | `AGENT-018/019`, `MARKET-007` |
| `M50` Model Provider | `planned` | planned | typed profiles + one provider adapter | unassigned | provider conformance + real bounded turn |
| `M51` MCP Binding/Executor | `planned` | planned | one reviewed read-only remote binding | unassigned | MCP lifecycle/security/executor proof |
| `M60` Campus Trust/Source | `planned` | planned | one reviewed source/revision/baseline | unassigned | `SRC-*` current-scope rows |
| `M70` ChangeRadar | `design-only` | design only | one semantic change + feed | unassigned | `RADAR-*` current-scope rows |
| `M71` Affairs Navigator | `design-only` | design only | one reviewed procedure board | unassigned | `PROC-*` current-scope rows |
| `M72` Opportunity Graph | `bounded-spike` | planner spike | honest source/profile/Market integration | unassigned | `COURSE-*` current-scope rows |
| `M80` Dioxus Fullstack | `planned` | no code | Web/PWA first plus mandatory Android shared journey | Kimi K3 + Claude Opus 5 lead Windows UI/design; GPT review/local optimization | exact active Fullstack/API/`CLIENT-*`/`WEB-*` rows added, then Web and Android passing |
| `M90` Infrastructure | `governance-baseline` | CI only | config/store/journal/evidence + Docker Compose Fullstack restore profile | unassigned | Compose Web/Android target profile restore/read-back |

Team assignment updates only the `Owner` cells and issue links. It does not change module ownership semantics.

## 4. S0 — Architecture and interface freeze

**Status**: complete; implementation remains acceptance-gated.

### `S0-1` Constitution and module registry

**Status**: complete.

- adopt full engineering constitution and mandatory work loop;
- define large-module ownership and dependency direction;
- define Dioxus Fullstack Web/Android shell, admitted server-function ingress and explicit optional public API boundary;
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
- `M00-B2 session-domain`: open/refresh/expire/revoke transitions and replay.
- `M00-B3 request-context`: land `policy-reference` first, then immutable admitted actor/request/command/causation context and duplicate/conflict semantics as separate reviewable commits.
- `M00-B4 ports-and-fakes`: land `session-port` and `control-evidence` as separate reviewable commits, then clock/session/audit/secret-ref fakes with failure fixtures.
- `M00-B5 api-admission-integration`: attach to `M10`; one denied request reaches no downstream fake.

These five roadmap batches schedule the six small modules in the M00 blueprint; they are not alternate module names. `identity-types` maps to B1, `session-domain` to B2, `policy-reference` plus `request-context` to B3, and `session-port` plus `control-evidence` to B4. Each small module still receives its own commit and standalone evidence before B5 composition.

Current completion scope excludes production USTC/CAS login. A labelled demo/auth adapter is sufficient if the boundary is honest.

## 6. M10 lane — Application Ingress Host

- `M10-B1 ingress-registry`: server-function/public route/version/error/event registry and DTO rules.
- `M10-B2 request-admission`: bounds, `M00` actor mapping, client build/protocol compatibility and preconditions.
- `M10-B3 server-function-adapter`: Axum-compatible first-party ingress with dependency reach-through checks.
- `M10-B4 dispatch-and-errors`: one ingress maps to one owned application operation and stable result/error.
- `M10-B5 event-stream`: monotone typed stream/SSE cursor, reconnect and backpressure.
- `M10-B6 server-lifecycle`: Dioxus SSR/assets/ingress attachment, preflight, readiness, graceful drain and black-box tests.
- `M10-B7 client-contract`: freeze the Web/Android subset plus supported-version/upgrade behavior required by `M80`.
- `M10-B8 public-adapter`: add explicit REST/SSE only for a real heterogeneous consumer.

Handlers contain mapping and coordination only. Domain validation remains in owning modules.

## 7. M20 lane — Market and Package Lifecycle

- `M20-B0 existing-resolver-audit`: compare `invocation.rs`/fixtures with the module blueprint; adopt/amend/spike decision.
- `M20-B1 package-catalog`: schema, catalog publication and anonymous read model.
- `M20-B2 capability-registry`: risk/data class and auto-grant eligibility.
- `M20-B3 installation-domain`: exact install/configure/enable/disable/revoke/uninstall.
- `M20-B4 grant-domain`: scope/version/reapproval and tenant checks.
- `M20-B5 invocation-authority`: integrate audited projection/recheck with repository transaction/preconditions.
- `M20-B6 update-rollback`: staged update, permission expansion and exact rollback.
- `M20-B7 composition`: attach read/mutation APIs and fake `M40` consumer.

`M20` merge scope is complete only when browse and current lifecycle state are distinct and disable/revoke blocks discovery/calls.

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

**Prerequisite**: one concrete public source receives owner, URL/retrieval, permission/rate and parser-fixture review.

- `M60-B1 source-registry`: stable identity, owner, policy and status.
- `M60-B2 retrieval-policy`: safe exact host/path, redirects, content/time/size/rate.
- `M60-B3 lease-snapshot`: deterministic lease and immutable raw evidence.
- `M60-B4 normalize-parser`: deterministic peers with exact identity/fixtures.
- `M60-B5 source-revision`: observed/published/effective time, digests and provenance.
- `M60-B6 conflict-freshness`: authority, stale and highest-authority conflict.
- `M60-B7 baseline`: atomic accepted baseline and failure injection.
- `M60-B8 publication-port`: typed product candidate/evidence boundary.

This lane precedes real product source integration, but each product can develop against exact M60 fixtures.

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

## 14. M80 lane — Dioxus Fullstack Multi-client

`M80-B1` may begin against a fake `M10` after `B-M80-M10-CALL`, `B-M10-M80-RESULT` and `B-M10-M80-EVENT` freeze only as a disposable, non-mergeable initialization spike. Before it becomes retained scaffold or `M80-B2+` starts, exact planned Fullstack/API/`CLIENT-*`/`WEB-*`/deployment rows with future bindings must be active in `matrix.tsv`.

- `M80-B1 initialization`: revalidate and exact-pin Dioxus/DX; minimal server/Web/Android target features and toolchain smoke; no product/domain code.
- `M80-B2 fullstack-contract`: versioned DTO/error/event/compatibility mapping with unknown-version behavior.
- `M80-B3 server-function-client`: generated query/command calls, correlation and typed stream cursor/reconnect.
- `M80-B4 app-state`: deterministic thin presentation reducer including `UpgradeRequired`.
- `M80-B5 design-system`: accessible shared UI/form/display components.
- `M80-B6 market-run-journey`: one complete fake then real M10 journey.
- `M80-B7 web-pwa`: page/assets, SSR/hydration or explicit CSR, responsive/accessibility/console/network proof.
- `M80-B8 compose-fullstack`: attach native server build to M90 Compose profile and prove restart/read-back.
- `M80-B9 android`: emulator plus real-device HTTPS/session/lifecycle/reconnect/Custom Tab/package proof.
- `M80-B10 version-skew`: supported older Android protocol plus typed unsupported-version rejection.
- `M80-B11 ios`: later peer target adapter/signing/device proof.
- `M80-B12 desktop`: optional later peer target adapter and packaging proof.

Every product calculation/mutation is re-evaluated by backend modules. Client code only displays and submits intent.

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

`M00` admitted context + `M10` Fullstack ingress/event host + fake application modules + `M80` fake Web/Android-compatible client journey.

Prerequisite: exact active acceptance rows and future bindings exist for every retained M00/M10/M80 scaffold. A disposable initialization spike cannot satisfy A0 or enter the module merge packet.

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
