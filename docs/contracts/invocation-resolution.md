# Invocation resolution contract

## Metadata

- `Status`: P0a planned contract; no resolver implementation or runnable package fixture exists yet
- `Version`: `invocation-resolution/v0`
- `Last Review`: `2026-07-23`
- `Owning Plan`: [`../plan/04-market-and-plugin-lifecycle.md`](../plan/04-market-and-plugin-lifecycle.md)
- `Authority Defers To`: [`../plan/03-platform-authority.md`](../plan/03-platform-authority.md) for state ownership, [`agent-runtime.md`](agent-runtime.md) for run/effect state, and [`permissions.md`](permissions.md) for capability classes
- `Acceptance`: planned P0a `MARKET-005`, `MARKET-006`; supporting evidence only for future cross-boundary `MARKET-007` and later durable `MARKET-002`, `MARKET-003`; downstream implemented `AGENT-002`
- `Primary Code`: planned `crates/platform-core/src/invocation.rs`; bounded proof consumer planned in `crates/agent-runtime/tests/resolved_run_spec.rs`; no real application consumer exists yet

## 1. Scope and authority

P0a establishes a deterministic, typed, no-I/O authority decision before catalog browsing, durable installation persistence or external execution. A pure `InvocationResolver` receives exact caller-supplied authority snapshots and either returns an immutable resolved projection or one typed denial. It does not read a database, parse the live catalog, call a provider, dispatch a component or persist an effect.

Ownership remains split:

- reviewed `market/` declarations own which package revision and capability declarations exist;
- future Rust-governed operational state owns installation, enable/revoke and grants;
- `platform-core` owns exact resolution and fail-closed policy over supplied snapshots;
- `agent-runtime` owns `RunSpec`, run construction, legal transitions, budgets, effect intent/receipt ordering and replay;
- application composition obtains snapshots and maps a successful resolution into the existing runtime boundary;
- framework/provider/MCP adapters consume platform decisions but own no package, grant, approval, receipt, budget, audit or replay authority.

No framework is a P0a dependency. The contract adapts mechanisms from the five-reference audit under [`ADR-0004`](../adr/0004-runtime-reference-strategy.md); it does not import framework state or APIs.

## 2. Typed inputs

The implementation MUST use owned validated ID newtypes rather than passing interchangeable `String` values. Names below are contract-level Rust shapes; exact field layout may be refined without weakening the listed bindings.

`ToolProjectionRequest` binds:

- tenant and user identity;
- run/turn identity supplied by the caller;
- an optional activation allow-list that may only narrow the eligible tool set.

Each requested `InvocationTarget` binds:

- installation ID;
- requested exact package ID/version, component ID and tool ID;
- requested capability ID and object scope.

`CatalogPackageRevision` binds:

- catalog revision/schema identity;
- exact package ID, SemVer version and package digest;
- honest runnable status;
- exact component ID, kind, component version/digest and execution identity;
- stable tool ID, collision-checked model-visible name and exact provider-visible description;
- capability mapping and capability-manifest digest;
- one owned `tool-input-schema/v0` value and its claimed lowercase `sha256:<64 hex>` digest;
- source-policy identity/digest and catalog revoke state.

`PluginInstallationSnapshot` binds:

- installation, tenant and user identity;
- pinned package ID/version/digest and effective component identities;
- enabled, disabled or revoked state;
- installation revision observed by the caller.

`CapabilityGrantSnapshot` binds:

- grant snapshot ID/version, tenant/user and installation identity;
- exact capability ID, bounded object scope and confirmation policy;
- capability-manifest digest admitted by the grant;
- active, stale, expired or revoked state.

`InvocationPolicySnapshot` binds the capability-registry classification, execution/source admission decision and any operator emergency block. Unknown classification or absent policy evidence is denial, not a default.

At call time, `ProposedToolCall` binds the provider `tool_call_id`, frozen model-visible tool name and exact argument value/digest. The caller supplies fresh installation, grant, catalog-revoke and emergency-policy snapshots as `CurrentDenyState`; this state may only preserve or deny an entry already present in the projection.

These inputs are deliberately synthetic/in-memory in P0a. Later repositories may load them, but storage types do not decide the result and must not be accepted as already authorized.

### 2.1 Bounded input-schema dialect

P0a does not accept arbitrary JSON Schema. `tool-input-schema/v0` is an owned typed AST with exactly these forms:

- root and nested objects with lexicographically keyed properties, a duplicate-free required-property set and `additional_properties = false`;
- string, integer, finite number and boolean scalars;
- homogeneous arrays containing one supported item schema;
- optional string enums whose values are non-empty, unique and lexicographically ordered.

The schema has maximum depth `8`, maximum total nodes `256`, maximum object properties `64` and maximum string-enum entries `64`. `$ref`, definitions, unions/composition, conditionals, `format`, regex/pattern, coercion, defaults, arbitrary annotations and open/schema-valued additional properties are unsupported. A future catalog/provider adapter may translate a strict external JSON Schema subset into this AST; any unsupported source keyword MUST return a typed `SchemaSourceUnsupported` loader error and construct no authority snapshot.

The canonical schema encoding starts with the UTF-8 domain separator `tool-input-schema/v0\0`. It then recursively emits one fixed `u8` variant tag; counts as `u64` big-endian; UTF-8 strings as `u64` big-endian byte length followed by bytes; object properties and required names in lexicographic order; enum values in lexicographic order; and option presence as `0` or `1`. No map iteration order, source JSON whitespace/key order, numeric text spelling or provider serialization participates. The resolver MUST recompute `input_schema_digest = sha256:<lowercase hex>` from this encoding and compare it with the claimed digest.

The exact provider-visible definition is `(model_visible_name, description, input_schema_digest)`. Its digest uses domain separator `provider-tool-definition/v0\0` followed by those three UTF-8 values, each encoded as `u64` big-endian byte length plus bytes. Any name, description or schema change therefore changes the provider-tool-definition digest.

## 3. Deterministic outputs

### 3.1 `ResolvedInvocation`

A successful result contains the exact identities needed downstream:

- tenant/user and installation ID/revision;
- package ID/version/digest and catalog revision;
- component ID/version/digest/kind and execution identity;
- tool ID, model-visible name and collision-free dispatch key;
- exact provider-visible description and provider-tool-definition digest;
- capability ID, capability-manifest digest, grant snapshot/version/scope and confirmation policy;
- source-policy identity/digest;
- canonical input-schema digest;
- authority snapshot revisions used for the decision.

It does not mint a run ID, provider profile, budgets, platform call/effect/idempotency IDs or a grant. Those remain caller/runtime responsibilities.

### 3.2 `ToolProjectionSnapshot`

A projection contains `tool-projection/v0`, the supplied run/turn identity, a deterministic snapshot ID, an ordered list of resolved entries and one `tool_schema_set_digest`.

Normative invariants:

1. entries are ordered lexicographically by `(package_id, package_version, component_id, tool_id)`;
2. the dispatch key binds that full identity; name-only dispatch is forbidden;
3. every model-visible name is unique inside the projection; a collision is a typed denial, never last-wins or silent renaming;
4. one snapshot controls both schemas exposed to the model and dispatch keys accepted for that turn;
5. the schema-set digest input starts with the UTF-8 domain separator `tool-projection/v0\0`; for each ordered entry it appends the dispatch key and provider-tool-definition digest, each as `u64` big-endian byte length followed by UTF-8 bytes; `tool_schema_set_digest` is `sha256:<lowercase hex>` of that byte sequence;
6. `snapshot_id` is `tool-projection:` followed by the complete `tool_schema_set_digest`; it does not depend on wall clock, random registration order or framework state;
7. runtime registration changes cannot mutate a created snapshot; a later projection requires a new resolver call;
8. activation/session state may remove entries but cannot install, grant, re-enable or bypass revoke.
9. `tool-projection/v0` entries share one exact tenant/user, installation, package version, component and grant snapshot so they map without ambiguity into the singular identity fields of `agent-run/v0`; mixed-authority targets fail with `AuthorityConflict`.

The existing `RunSpec.tool_schema_set_digest` field name is retained for `agent-run/v0` compatibility. Under this contract its normative value is the `tool-projection/v0` digest over exact dispatch keys and complete provider-tool-definition digests; it MUST NOT be interpreted as hashing schema bytes alone.

For the bounded proof consumer, one successful projection supplies the existing `RunSpec` installation/package/component/grant fields and exact `tool_schema_set_digest`. Reusing it across turns is allowed only while the run continues to pin that same digest; widening requires a new run or a separately approved runtime-contract change.

### 3.3 `AuthorizedInvocation`

A successful call-time decision returns the exact frozen projection entry, correlated provider call ID, validated canonical arguments and digest, and current authority revisions used for the deny-side recheck. It does not contain a live adapter handle, effect/idempotency identity or receipt. Application/runtime composition must create and persist those identities before execution.

## 4. Projection-time and call-time decisions

### 4.1 Projection time

`resolve_projection(request, targets, authority_snapshots)` MUST verify exact identity equality, runnable status, tenant/user scope, component and execution admission, capability declaration/classification, grant version/scope, source policy, schema digest, installation enable/revoke state and emergency block before returning a projection.

Equivalent input snapshots produce equal typed output and the same canonical digest. Input ordering, hash-map iteration and framework registration order cannot affect the result.

### 4.2 Call time

`authorize_call(projection, current_deny_state, proposed_call)` MUST:

1. correlate the provider `tool_call_id` without treating it as a platform effect identity;
2. select exactly one entry by the frozen model-visible name and bound dispatch key;
3. validate arguments without coercion against the exact bounded `tool-input-schema/v0` AST represented by that entry;
4. recheck current tenant/scope, installation enable/revoke, grant validity and emergency block; application composition must separately require the runtime-owned budget/phase decision before effect intent;
5. allow current authority only to preserve or narrow the frozen projection—new installation/grant/enable state cannot widen it;
6. return an authorized platform request from which the runtime/application mints and persists call/effect/idempotency identity.

A committed effect intent and receipt remain `agent-runtime`/durable-orchestration concerns. Framework argument parsing or preflight hooks are defense in depth, never authorization.

## 5. Fail-closed error taxonomy

`InvocationResolutionError` MUST distinguish the following precedence groups. Targets are checked in canonical projection order and variants inside a group use the left-to-right order shown; when several conditions are invalid, the first group, first target and first variant win so the primary error is deterministic.

1. malformed: `InvalidRequest` or `InvalidAuthoritySnapshot`;
2. global deny/conflict: `EmergencyBlocked` or `AuthorityConflict`;
3. scope: `TenantOrUserScopeMismatch`;
4. package/catalog: `PackageMissing`, `PackageNotRunnable`, `PackageVersionMismatch`, `PackageDigestMismatch` or `CatalogRevoked`;
5. installation: `InstallationMissing`, `InstallationDisabled`, `InstallationRevoked` or `InstallationRevisionMismatch`;
6. component/execution: `ComponentMissing`, `ComponentIdentityMismatch`, `ExecutionIdentityUnknown` or `ExecutionIdentityMismatch`;
7. tool identity: `ToolMissing` or `ToolIdentityMismatch`;
8. capability: `CapabilityUnknown`, `CapabilityNotDeclared`, `CapabilityManifestMismatch` or `CapabilityNotGranted`;
9. grant: `GrantStale`, `GrantExpired`, `GrantRevoked`, `GrantVersionMismatch` or `GrantScopeMismatch`;
10. source: `SourcePolicyMissing` or `SourcePolicyMismatch`;
11. schema/arguments: `SchemaMissing`, `SchemaDialectUnsupported`, `SchemaMalformed`, `SchemaDigestMismatch` or `ArgumentsInvalid`;
12. projection/dispatch: `ToolNameCollision`, `ToolNotProjected` or `DispatchIdentityMismatch`.

Every error leaves inputs unchanged, returns no partial projection/dispatch handle and makes `RunSpec`/effect-intent construction impossible. Errors do not select a same-name component, older package, broader grant, alternate provider/runtime or previous successful snapshot.

## 6. Bounded proof consumer and fixtures

The first bounded proof consumer is a cross-crate test in planned `crates/agent-runtime/tests/resolved_run_spec.rs`:

1. resolve a synthetic implemented package/component in memory;
2. combine only successful resolved fields with caller-supplied run ID, provider profile and budgets;
3. construct the existing `RunSpec` and call `AgentRun::new`;
4. assert each denied resolution produces no `RunSpec` and no `AgentRun`.

This proves deterministic resolver output and the `RunSpec` mapping only. It is not a real application composition path and cannot prove that `authorize_call` runs before effect-intent persistence or adapter I/O. `MARKET-007` remains planned until a thin application service composes frozen model exposure, call authorization, runtime budget/phase decision, effect-intent creation and a fake adapter sink, with denial proving that neither intent nor adapter call occurs.

The positive fixture MUST remain synthetic because all current first-party manifests have empty `components` arrays and do not prove runnable installation state. P0a must not change their `implementationStatus` or claim Course Planning is Market-integrated.

Planned fixture directory: `crates/platform-core/tests/fixtures/invocation-resolution/`.

| Fixture | Required proof | Acceptance |
|---|---|---|
| `valid-synthetic-v0.json` | exact deterministic output, digest and `RunSpec` mapping | `MARKET-005`, downstream `AGENT-002`; supports later `MARKET-002` |
| `identity-mismatch-v0.json` | package/component/execution/digest mismatch returns exact error and no run | `MARKET-006` |
| `tool-identity-mismatch-v0.json` | missing or mismatched requested tool ID returns the exact projection-time error | `MARKET-006` |
| `disabled-revoked-v0.json` | disabled, catalog-revoked and emergency-blocked states deny | `MARKET-006`; supports later `MARKET-003` |
| `grant-scope-stale-v0.json` | missing/stale/revoked/version/scope-mismatched grants deny | `MARKET-006`; supports future `MARKET-007` |
| `tool-definition-mutation-v0.json` | name, description or schema mutation changes provider-definition and projection digests; visible-name collision never falls back | `MARKET-005`, `MARKET-006` |
| `schema-bounded-v0.json` | supported AST validates deterministically; wrong dialect, malformed bounds/sets/depth, digest mismatch and invalid arguments deny exactly | `MARKET-005`, `MARKET-006`; supports future `MARKET-007` |
| `post-projection-revoke-v0.json` | current deny state narrows a frozen projection; later grant cannot widen it | supports future `MARKET-007` and later `MARKET-003` |

Fixture JSON is test input, not a catalog schema or durable-state format.

## 7. Framework evidence mapping

Review date: `2026-07-23`. Facts and source links are preserved in [`plan/07-runtime-and-integration.md`](../plan/07-runtime-and-integration.md); the table below records only P0a's owned adaptation.

| Reference | Borrow | Adapt into P0a | Reject |
|---|---|---|---|
| [Rig](https://github.com/0xPlaygrounds/rig) | per-turn schema/implementation snapshot; one allow-list for exposure and dispatch; typed arguments | build the snapshot only from exact platform package/component/grant/schema identities | last-wins collision; runtime registry as install/grant/approval authority |
| [LangGraph interrupts](https://docs.langchain.com/oss/python/langgraph/interrupts) and [persistence](https://docs.langchain.com/oss/python/langgraph/persistence) | correlation IDs; checkpoint/store distinction; explicit resume | framework thread/checkpoint is adapter state keyed by `platform_run_id`; approval maps to platform effect intent/receipt | checkpoint/store as grant, receipt, budget, audit or replay truth; effects before a restartable interrupt |
| [Pi Agent](https://github.com/earendil-works/pi/tree/main/packages/agent) | validated preflight, call-ID lifecycle and deterministic result projection | preflight consumes a platform decision and frozen projection | mutable/hot-loaded tools, project trust or package config as authorization |
| [goose permissions](https://goose-docs.ai/docs/guides/managing-tools/goose-permissions) and [extensions](https://goose-docs.ai/docs/getting-started/using-extensions) | durable install/default versus session activation; allow/ask/deny UX | map labels to typed capability/grant/confirmation policy; activation only narrows | autonomous default, LLM risk classification or auto-enable that widens authority |
| [Hermes Agent tools runtime](https://hermes-agent.nousresearch.com/docs/developer-guide/tools-runtime) and [architecture](https://hermes-agent.nousresearch.com/docs/developer-guide/architecture) | central schema projection, toolset filtering and fail-safe availability | registry becomes a read-only projection with namespaced identity and schema digest | last-wins names; registry/plugin/profile/session approval as campus authority; profile as tenant sandbox |

The audit did not pin releases/commits/licenses because no dependency or code adoption is proposed. Any later adoption must repeat the six-axis gate with exact source identity.

## 8. Non-goals and current status

P0a does not include:

- catalog query projection or anonymous browse/detail;
- durable installation, grant, enable/disable or upgrade mutation;
- a production database/repository transaction or TOCTOU closure;
- provider, network, MCP, daemon HTTP/SSE or UI adapters;
- external tool execution, durable journal or crash recovery;
- autonomous multi-agent orchestration;
- changes to the three current first-party manifest component/status claims.

Current repository status remains: manifests and R0 runtime kernel are implemented; invocation resolver, typed operational snapshots, positive runnable package fixture and all `MARKET-002/003/005/006/007` bindings are planned. P0a may implement `MARKET-005/006`; `MARKET-007` requires the later real application composition seam described above. Passing documentation checks is not implementation evidence.
