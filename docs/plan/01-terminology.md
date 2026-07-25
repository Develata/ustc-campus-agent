# Terminology and normative language

## Metadata

- `Layer`: Foundation
- `Status`: Current MUST
- `Version`: `0.4.0`
- `Last Review`: `2026-07-25`
- `Authority Owns`: normative vocabulary used across plans, contracts and acceptance
- `Authority Defers To`: typed schemas and Rust definitions for machine spelling
- `Counterpart Acceptance`: `docs/acceptance/matrix.tsv`
- `Primary Code Areas`: `crates/platform-core/`, `market/`

## 1. Normative language

- **MUST / 必须**: required; violation means the contract is not satisfied.
- **SHOULD / 应**: expected unless a documented alternative preserves the owning invariant.
- **MAY / 可**: optional or stage-dependent; not part of the required path.
- **Fail-closed**: reject or retain the last accepted state when evidence, authority or permission is incomplete.

Requirements SHOULD be testable or observable. Words such as “better”, “intelligent” or “safe” do not form a contract without a decidable condition.

## 2. Product and implementation terms

- **Product topology**: the formal set of independently identified products and their lifecycle boundaries. USTC Campus Agent has exactly three default first-party Plugins.
- **Implementation order**: the dependency-aware sequence in which shared foundations and product journeys are built. It does not rank formal product identity.
- **Bounded spike**: an implementation that proves a narrow contract while explicitly not claiming the surrounding lifecycle or platform integration.
- **Feature projection**: user-visible behavior derived from a plan and typed contracts.

## 3. Engineering module terms

- **Large module**: one independently owned, independently testable responsibility and state family with narrow public inputs/outputs and its own exit gate. "Large" describes independence, not line count.
- **Small module**: one high-cohesion implementation unit inside a large module that can be reviewed and committed as one semantic slice.
- **Public module boundary**: the only named commands, queries, events, values and errors another large module may use. Private structs/storage/implementation details do not cross it.
- **Composition surface**: declared assembly code, normally `ustc-agentd`, that maps and orders public module calls without copying their domain rules.
- **Fake counterpart**: deterministic test implementation of one public boundary used to develop and accept a module before other large modules are complete.
- **StandaloneReady**: the module's current small modules, public contracts, fakes and failure tests pass without requiring real peer modules.
- **IntegrationReady**: the module additionally exposes its production-facing adapter and exact cross-module fixtures.
- **Thin client**: a Dioxus/CLI shell that displays server-owned state and submits typed user intent. It does not perform truth-affecting calculation or mutation.
- **Backend/application infrastructure**: explicit `ustc-agentd` application services and domain/execution modules that perform all product calculations and mutations. It does not mean a generic infrastructure layer may own domain rules.

## 4. Market terms

- **Catalog Authority**: the reviewed Git-resident package schema, publisher, capability and manifest declarations that determine which public package revision exists.
- **Catalog Projection**: a query-optimized view of Catalog Authority. It is rebuildable and cannot approve or un-revoke a package.
- **PluginPackage**: the only first-class unit users inspect, install, enable, disable and upgrade.
- **Component**: an exact package-declared implementation/resource member accepted by the current schema. A component is not independently installable unless a future approved ontology says so.
- **Plugin Installation**: tenant/user runtime state pinning an exact package version and its effective components.
- **Capability Grant**: explicit runtime authorization for a stable capability ID in a bounded tenant/object scope.
- **Enable state**: whether an installed Plugin may be discovered for invocation. Installation and enablement are distinct.
- **Invocation**: one gateway-mediated tool/component call after current installation, version, enablement and grant resolution.
- **Plugin contribution**: a validated package component projection such as a procedural asset, resource or tool-provider route. It is not installation, grant or Agent state.
- **FirstPartySystemPlugin**: a reviewed default first-party package governed by the same exact-version, permission-expansion, disable and audit rules as other packages.

## 5. Campus Trust Kernel terms

- **SourceDefinition**: reviewed declaration of one source identity, authority, owner, URL/retrieval policy, parser policy and status.
- **SourceRevision**: immutable observation of a source, binding canonical/retrieved URL, time, digests, parser identity and snapshot references.
- **Accepted baseline**: latest SourceRevision whose snapshot, parse, normalization, diff, candidate and evidence writes all completed durably.
- **Normalized fact**: typed fact derived from an exact SourceRevision with authority, effective time, provenance and conflict state.
- **Candidate**: unapproved generated or parsed material. Candidate state carries no canonical publication authority.
- **Published artifact**: validator-approved, administrator-approved canonical procedure/change/graph artifact with immutable identity and receipt.
- **Campus Trust Kernel**: shared source identity, immutable revision, authority ordering, temporal, conflict, provenance, grant and audit semantics used by all three first-party Plugins.
- **Tenant-private profile fact**: user-provided or user-derived data visible only in its authorized tenant/user scope; it never enters the public campus fact projection.

## 6. Runtime and proof terms

- **ConversationSession**: durable ordered conversation scope that may contain many finite `HarnessRun`s; it is not itself one execution run.
- **HarnessRun**: platform-owned finite state machine for one accepted user task, from context/clarification through graph execution, verification and report.
- **TaskGraph**: validated finite DAG of task nodes, dependencies, resource claims and bounded executor/reviewer policies. It is not a generic workflow language.
- **TaskContract**: immutable parent-owned goal, non-goals, policy, inputs, deliverables, acceptance and verification contract for one task node.
- **Platform run / AgentRun**: platform-owned state machine for one bounded node execution and its model/tool/effect loop. Framework/session state is only an adapter projection keyed to it.
- **Agent tool protocol**: versioned Plugin-neutral definitions/calls/results between Agent execution and `ToolGateway`; it carries no package lifecycle or Agent state-machine authority.
- **ToolGateway**: composition service that maps one frozen tool call through current authorization, effect intent/receipt ordering and a bounded Plugin executor. It coordinates authorities but owns none.
- **PluginExecutor**: replaceable bounded implementation selected from exact installed component/execution identity; it cannot register directly into or mutate the Agent loop.
- **PromptProjection**: complete bounded provider-visible request derived from canonical run/session state; it is never canonical history or authority.
- **Compaction**: deterministic reduction of a prompt projection by replacing eligible persisted/reproducible payloads with typed references.
- **Compression**: lossy summarization of eligible history into a provenance-bearing context artifact while canonical history remains unchanged.
- **EvidencePack**: typed artifact and verification references submitted for node or final review; a model report or process exit alone is not evidence.
- **Receipt**: durable record of an approved attempted/completed side effect, used for audit and duplicate-effect prevention.
- **Acceptance Case**: stable ID plus preconditions, assertions, binding and required gate.
- **Evidence Binding**: exact automated test, command, browser check, external conformance result or manual review that can satisfy an Acceptance Case.
- **Required Gate**: named suite whose required cases must be pass; planned, skipped, unavailable and not-run are non-pass.
- **Configuration Smoke**: typed validation of configuration shape and, at higher levels, resolved or live read-only dependencies. No higher level is implied until implemented.
