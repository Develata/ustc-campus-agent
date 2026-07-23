# Invocation resolution contract

## Metadata

- `Status`: P0a deterministic resolver and synthetic proof fixtures implemented; no durable or real application consumer exists yet
- `Version`: `invocation-resolution/v0`
- `Last Review`: `2026-07-23`
- `Owning Plan`: [`../plan/04-market-and-plugin-lifecycle.md`](../plan/04-market-and-plugin-lifecycle.md)
- `Authority Defers To`: [`../plan/03-platform-authority.md`](../plan/03-platform-authority.md) for state ownership, [`agent-runtime.md`](agent-runtime.md) for run/effect state, and [`permissions.md`](permissions.md) for capability classes
- `Acceptance`: implemented P0a `MARKET-005`, `MARKET-006`; supporting evidence only for future cross-boundary `MARKET-007` and later durable `MARKET-002`, `MARKET-003`; downstream implemented `AGENT-002`
- `Primary Code`: `crates/platform-core/src/invocation.rs`; bounded proof consumer in `crates/agent-runtime/tests/resolved_run_spec.rs`; no real application consumer exists yet

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
- typed run ID and typed turn ID supplied separately by the caller;
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
- one constructed `ValidatedToolInputSchemaV0` and its claimed lowercase `sha256:<64 hex>` digest;
- source-policy identity/digest and catalog revoke state.

`PluginInstallationSnapshot` binds:

- installation, tenant and user identity;
- pinned package ID/version/digest and effective component identities;
- enabled, disabled or revoked state;
- installation revision observed by the caller.

`CapabilityGrantSnapshot` binds:

- immutable grant snapshot ID/version, tenant/user and installation identity; any capability, scope, confirmation-policy or state change mints a new version;
- exact capability ID, bounded object scope and confirmation policy;
- capability-manifest digest admitted by the grant;
- active, stale, expired or revoked state.

`InvocationPolicySnapshot` binds an immutable policy snapshot ID/revision, capability-registry classification, execution/source admission decision and any operator emergency block. Unknown classification or absent policy evidence is denial, not a default.

At call time, `ProposedToolCall` binds the provider `tool_call_id`, frozen model-visible tool name, opaque projection-issued dispatch key, one constructed `CanonicalArgumentValueV0` and its claimed digest. The application adapter derives the visible name and dispatch key from the same frozen per-turn projection entry; it cannot synthesize a name-only dispatch. The caller supplies fresh installation, grant, catalog-revoke and emergency-policy snapshots as `CurrentDenyState`; this state may only preserve or deny an entry already present in the projection.

These inputs are deliberately synthetic/in-memory in P0a. Later repositories may load them, but storage types do not decide the result and must not be accepted as already authorized.

### 2.1 Schema loader and validated AST boundary

P0a does not accept arbitrary JSON Schema. A future catalog/provider adapter owns `load_source_schema(source_bytes) -> UnvalidatedToolInputSchemaV0` and MUST preserve duplicate object members while parsing. Unsupported source keywords or source syntax return loader-only `SchemaSourceUnsupported` or `SchemaSourceMalformed` and construct no P0a input; these errors do not belong to resolver acceptance.

P0a owns `ValidatedToolInputSchemaV0::try_from(unvalidated)`. The unvalidated value carries an exact dialect string plus ordered property/required/enum sequences so duplicate declarations remain observable. The constructor accepts only dialect `tool-input-schema/v0`, requires an object root, rejects duplicate or invalid names and validates every structural limit before sorting into the validated AST. Its disjoint errors are `SchemaDialectUnsupported`, `SchemaMalformed` and `SchemaLimitExceeded`.

The validated AST has exactly six variants and tags:

| Tag | Variant | Canonical payload after the tag |
|---|---|---|
| `0x01` | object | property count; each property name and child node in name order; required-name count; each required name in name order |
| `0x02` | string | enum-presence byte `0` or `1`; if `1`, value count and each value in bytewise UTF-8 order |
| `0x03` | integer | none |
| `0x04` | finite number | none |
| `0x05` | boolean | none |
| `0x06` | homogeneous array | one child item node |

A count is `u64` big-endian. A string is `u64` big-endian byte length followed by its exact UTF-8 bytes; no Unicode normalization is performed. Property names match `^[A-Za-z_][A-Za-z0-9_.-]{0,63}$`. Required names are unique and must name declared properties. A present string enum has `1..=64` unique values, each `1..=256` UTF-8 bytes. The root counts as depth `1`; maximum depth is `8`, maximum total nodes `256`, maximum properties per object `64`, and maximum canonical schema bytes `65_536`. Objects are always closed: `additional_properties = false` is implicit and emits no byte.

The canonical schema byte sequence is the UTF-8 domain separator `tool-input-schema/v0\0` followed by the root node encoding above. The constructor stores these bytes and `input_schema_digest = sha256:<lowercase hex>`. A `CatalogPackageRevision` separately carries the claimed digest; `resolve_projection` MUST compare it with the constructed digest and return `SchemaDigestMismatch` on inequality. Source JSON whitespace/key order, map iteration, numeric text and provider serialization never participate.

The exact provider-visible definition is `(model_visible_name, description, input_schema_digest)`. The model-visible name matches `^[A-Za-z_][A-Za-z0-9_.-]{0,63}$`; description is at most `4_096` UTF-8 bytes. Its canonical bytes are domain separator `provider-tool-definition/v0\0`, then those three values in that order, each encoded as the length-prefixed UTF-8 string above. Any name, description or schema change therefore changes `provider_tool_definition_digest`.

### 2.2 Canonical call arguments

A provider adapter MUST parse raw JSON without losing duplicate object members into `UnvalidatedArgumentValueV0`; JSON object order is not authoritative. `CanonicalArgumentValueV0::try_from(unvalidated)` returns a canonical tree or one constructor-only `ArgumentConstructionError`: `ArgumentDuplicateKey`, `ArgumentInvalidName`, `ArgumentNumberOutOfRange` or `ArgumentLimitExceeded`.

Canonical argument tags and payloads are:

| Tag | Variant | Canonical payload after the tag |
|---|---|---|
| `0x00` | null | none; the v0 schema has no null type, so validation later rejects it |
| `0x01` | boolean | one byte, `0` for false or `1` for true |
| `0x02` | integer | one signed two's-complement `i64` in 8-byte big-endian order |
| `0x03` | finite number | one IEEE-754 binary64 bit pattern in 8-byte big-endian order; `-0.0` is normalized to `+0.0` |
| `0x04` | string | one length-prefixed exact UTF-8 string |
| `0x05` | array | element count followed by each child in source order |
| `0x06` | object | member count followed by each name and child in bytewise UTF-8 name order |

A JSON token without decimal point or exponent must fit `i64` and becomes integer; a token with decimal point or exponent is correctly rounded to finite binary64 and becomes number. NaN, infinities and overflow are rejected. Object keys obey the property-name rule above. Maximum depth is `8`, total nodes `256`, members per object `64`, elements per array `256`, one string `4_096` UTF-8 bytes, and complete canonical argument bytes `65_536`.

Canonical argument bytes are domain separator `tool-arguments/v0\0` followed by the root value encoding. The constructor stores `argument_digest = sha256:<lowercase hex>`. `authorize_call` recomputes/compares the claimed digest only after selecting and checking the frozen dispatch entry; mismatch returns `ArgumentDigestMismatch`, while schema incompatibility returns `ArgumentsInvalid`. No coercion occurs: integer and number are distinct, object extras fail because schemas are closed, and enum equality is exact UTF-8 byte equality.

### 2.3 Golden canonical vectors

`schema-golden-v0.json` and `arguments-golden-v0.json` MUST pin the complete canonical byte hex and lowercase SHA-256 values recorded here:

- schema `{count: integer, query: string}`, required `{query}`: `80` bytes, hex `746f6f6c2d696e7075742d736368656d612f7630000100000000000000020000000000000005636f756e7403000000000000000571756572790200000000000000000100000000000000057175657279`, digest `sha256:8a91a2fdad047d1bcfc4ac0392778f7125afce4faf637ed3aac4fd535fd1db2e`;
- arguments `{count: 2, query: "graph"}`: `76` bytes, hex `746f6f6c2d617267756d656e74732f7630000600000000000000020000000000000005636f756e74020000000000000002000000000000000571756572790400000000000000056772617068`, digest `sha256:8b881ec565f0aac688241061c398f199c8e0683604502f1e7538f09f33350451`.

The fixtures also cover property/input permutations, duplicate names, every limit at and beyond its boundary, `i64::MIN/MAX`, integer overflow, `-0.0`, smallest finite/subnormal numbers, non-finite rejection and integer-versus-number mismatch. The literal vectors above are contract data; tests MUST compare independently constructed bytes and digests against them rather than regenerating expected values through the production encoder.

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

The opaque dispatch key is canonical and routing-only. Its bytes are domain separator `dispatch-identity/v0\0`, then canonical `package_id`, parsed-and-rendered SemVer `package_version`, `component_id` and `tool_id` in that order, each using the length-prefixed UTF-8 encoding from §2.1. The key is `dispatch:sha256:<lowercase hex>` of those bytes. No delimiter escaping, registration order, model-visible name or adapter handle participates; all remaining authority is checked from the frozen entry rather than inferred from this key.

### 3.2 `ToolProjectionSnapshot`

A projection contains `tool-projection/v0`, typed run ID, typed turn ID, deterministic `snapshot_id`, ordered resolved entries, `tool_schema_set_digest` and `projection_authority_set_digest`.

Normative invariants:

1. entries are ordered lexicographically by `(package_id, parsed package_version, component_id, tool_id)`;
2. each opaque dispatch key uses the exact construction in §3.1; name-only dispatch is forbidden;
3. every model-visible name is unique inside the projection; a collision is `ToolNameCollision`, never last-wins or silent renaming;
4. one immutable snapshot controls both provider definitions exposed to the model and dispatch keys accepted for exactly one `(run_id, turn_id)`;
5. `tool_schema_set_digest` hashes domain separator `tool-projection/v0\0`, followed for each ordered entry by dispatch key then provider-tool-definition digest, each length-prefixed as in §2.1;
6. each `projection_authority_entry_digest` hashes domain separator `projection-authority-entry/v0\0` followed by these canonical strings in order: dispatch key, provider-tool-definition digest, tenant ID, user ID, installation ID, installation revision, package ID, package version, package digest, catalog revision, component ID, component version, component digest, execution identity, tool ID, capability ID, capability-manifest digest, grant snapshot ID, grant version, source-policy ID, source-policy digest, policy snapshot ID, policy revision and input-schema digest;
7. `projection_authority_set_digest` hashes domain separator `projection-authority-set/v0\0` followed by each ordered entry digest as a length-prefixed string;
8. `snapshot_id` is `tool-projection:sha256:<lowercase hex>`, where the hash input is domain separator `tool-projection-snapshot/v0\0` followed by run ID, turn ID, `tool_schema_set_digest` and `projection_authority_set_digest`, each length-prefixed as in §2.1;
9. runtime registration changes cannot mutate a created snapshot; activation/session state may remove candidates before resolution but cannot mutate, widen, install, grant, re-enable or bypass revoke after construction;
10. entries share one exact tenant/user, installation, package version, component and grant snapshot so they map unambiguously into the singular identity fields of `agent-run/v0`; mixed-authority targets fail with `AuthorityConflict`.

All IDs, revisions and digests above come from validated newtypes with one canonical UTF-8 representation; digest newtypes require lowercase `sha256:<64 hex>`. SemVer is encoded from the parsed `semver::Version` display form. An alternate textual spelling that parses to the same canonical value cannot survive the newtype constructor.

The existing `RunSpec.tool_schema_set_digest` field name is retained for `agent-run/v0` compatibility. Its normative value is the tool-definition-set digest above; it MUST NOT be interpreted as hashing input-schema bytes alone.

A fresh `ToolProjectionSnapshot` is mandatory for every turn. Two turns may have the same `tool_schema_set_digest` when the exact provider definitions and dispatch identities are unchanged, but they have different turn-bound `snapshot_id` values and never reuse the same snapshot object. The bounded proof consumer maps the successful projection's singular identities and `tool_schema_set_digest` into `RunSpec`; a later turn whose fresh digest differs from the run-pinned digest fails closed or starts a separately approved new run.

### 3.3 `AuthorizedInvocation`

A successful call-time decision returns the exact frozen projection entry, correlated provider call ID, validated canonical arguments and digest, and current authority revisions used for the deny-side recheck. It does not contain a live adapter handle, effect/idempotency identity or receipt. Application/runtime composition must create and persist those identities before execution.

## 4. Projection-time and call-time decisions

### 4.1 Projection time

`resolve_projection(request, targets, authority_snapshots)` MUST verify exact identity equality, runnable status, tenant/user scope, component and execution admission, capability declaration/classification, grant version/scope, source policy, schema digest, installation enable/revoke state and emergency block before returning a projection.

Equivalent input snapshots produce equal typed output and the same canonical digest. Input ordering, hash-map iteration and framework registration order cannot affect the result.

### 4.2 Call time

`authorize_call(projection, current_deny_state, proposed_call)` MUST execute in this order:

1. validate the call envelope and correlate provider `tool_call_id` without treating it as a platform effect identity;
2. find exactly one frozen entry by model-visible name; absence returns `ToolNotProjected` before inspecting argument content or current deny-side state;
3. compare the supplied opaque dispatch key with that entry's key; mismatch returns `DispatchIdentityMismatch` and never searches another entry;
4. apply current emergency/conflict, tenant/user, catalog revoke, installation and grant deny-side checks in the call-time precedence below;
5. recompute and compare the canonical argument digest;
6. validate the canonical arguments without coercion against the selected entry's exact `ValidatedToolInputSchemaV0`;
7. return an authorized platform request from which runtime/application composition mints and persists call/effect/idempotency identity.

Current state may only preserve or narrow the frozen projection. A new installation, grant, enable state, visible name or dispatch key cannot widen it. A committed effect intent and receipt remain `agent-runtime`/durable-orchestration concerns. Framework argument parsing or preflight hooks are defense in depth, never authorization.

## 5. Fail-closed error taxonomy

Errors are phase-specific; a variant cannot be returned by a phase that cannot receive the corresponding invalid value.

- Source loading is future adapter work and returns only `SchemaSourceError::{SchemaSourceUnsupported, SchemaSourceMalformed}`; it constructs no P0a schema value and is not evidence for `MARKET-005/006`.
- Validated-schema construction returns only `SchemaConstructionError::{SchemaDialectUnsupported, SchemaMalformed, SchemaLimitExceeded}`.
- Canonical-argument construction returns only the `ArgumentConstructionError` variants in §2.2.

`ProjectionResolutionError` is the `resolve_projection` error type. Targets are checked in canonical projection order; when multiple faults exist, the first group, first target and leftmost variant win:

1. `InvalidRequest`, `InvalidAuthoritySnapshot`;
2. `EmergencyBlocked`, `AuthorityConflict`;
3. `TenantOrUserScopeMismatch`;
4. `PackageMissing`, `PackageNotRunnable`, `PackageVersionMismatch`, `PackageDigestMismatch`, `CatalogRevoked`;
5. `InstallationMissing`, `InstallationDisabled`, `InstallationRevoked`, `InstallationRevisionMismatch`;
6. `ComponentMissing`, `ComponentIdentityMismatch`, `ExecutionIdentityUnknown`, `ExecutionIdentityMismatch`;
7. `ToolMissing`, `ToolIdentityMismatch`;
8. `CapabilityUnknown`, `CapabilityNotDeclared`, `CapabilityManifestMismatch`, `CapabilityNotGranted`;
9. `GrantStale`, `GrantExpired`, `GrantRevoked`, `GrantVersionMismatch`, `GrantScopeMismatch`;
10. `SourcePolicyMissing`, `SourcePolicyMismatch`;
11. `SchemaMissing`, `SchemaDigestMismatch`;
12. `ToolNameCollision`.

`InvocationAuthorizationError` is the `authorize_call` error type. Its precedence is independent and follows the executable algorithm in §4.2:

1. `InvalidCall`;
2. `ToolNotProjected`;
3. `DispatchIdentityMismatch`;
4. `EmergencyBlocked`, `AuthorityConflict`;
5. `TenantOrUserScopeMismatch`;
6. `CatalogRevoked`;
7. `InstallationMissing`, `InstallationDisabled`, `InstallationRevoked`, `InstallationRevisionMismatch`;
8. `GrantStale`, `GrantExpired`, `GrantRevoked`, `GrantVersionMismatch`, `GrantScopeMismatch`;
9. `ArgumentDigestMismatch`;
10. `ArgumentsInvalid`.

Thus an unknown visible name plus malformed/oversized raw JSON fails in the argument constructor before `authorize_call` exists, while an unknown visible name plus a valid canonical argument carrying a wrong digest returns `ToolNotProjected`. A known name with both a wrong dispatch key and invalid arguments returns `DispatchIdentityMismatch`. Dual-fault fixtures MUST pin these outcomes.

Every error leaves inputs unchanged, returns no partial projection/dispatch handle and makes `RunSpec`/effect-intent construction impossible. Errors do not select a same-name component, older package, broader grant, alternate provider/runtime or previous successful snapshot.

## 6. Bounded proof consumer and fixtures

The first bounded proof consumer is the cross-crate test in `crates/agent-runtime/tests/resolved_run_spec.rs`:

1. resolve a synthetic implemented package/component in memory;
2. combine only successful resolved fields with caller-supplied run ID, provider profile and budgets;
3. construct the existing `RunSpec` and call `AgentRun::new`;
4. assert each denied resolution produces no `RunSpec` and no `AgentRun`.

This proves deterministic resolver output and the `RunSpec` mapping only. It is not a real application composition path and cannot prove that `authorize_call` runs before effect-intent persistence or adapter I/O. `MARKET-007` remains planned until a thin application service composes frozen model exposure, call authorization, runtime budget/phase decision, effect-intent creation and a fake adapter sink, with denial proving that neither intent nor adapter call occurs.

The positive fixture MUST remain synthetic because all current first-party manifests have empty `components` arrays and do not prove runnable installation state. P0a must not change their `implementationStatus` or claim Course Planning is Market-integrated.

Executable fixture directory: `crates/platform-core/tests/fixtures/invocation-resolution/`.

| Fixture | Required proof | Acceptance |
|---|---|---|
| `schema-golden-v0.json` | exact schema bytes/digest; tag coverage; permutation equality; dialect, duplicate, required-subset, depth/node/property/enum/byte-limit denials | `MARKET-005`, `MARKET-006` |
| `arguments-golden-v0.json` | exact argument bytes/digest; permutation equality; duplicate, limits, `i64` edges/overflow, `-0.0`, subnormal/non-finite and integer/number distinction | supports future `MARKET-007` |
| `valid-synthetic-v0.json` | exact resolved identities, dispatch/projection/authority digests, per-turn snapshot identity and `RunSpec` mapping | `MARKET-005`, downstream `AGENT-002`; supports later `MARKET-002` |
| `identity-mismatch-v0.json` | package missing/runnable/version/digest and component missing/identity/execution mismatches return exact projection error and no run | `MARKET-006` |
| `tool-identity-mismatch-v0.json` | missing or mismatched requested tool ID returns the exact projection-time error | `MARKET-006` |
| `scope-capability-source-v0.json` | tenant/user mismatch; capability unknown/not-declared/manifest/not-granted; source-policy missing/mismatch | `MARKET-006` |
| `installation-authority-v0.json` | installation missing/disabled/revoked/revision mismatch plus catalog revoke, emergency block and mixed-authority conflict | `MARKET-006`; supports later `MARKET-003` |
| `grant-scope-stale-v0.json` | stale/expired/revoked/version/scope-mismatched grant denials | `MARKET-006`; supports future `MARKET-007` |
| `tool-definition-mutation-v0.json` | name, description or schema mutation changes provider-definition/projection digests; visible-name collision fails | `MARKET-005`, `MARKET-006` |
| `call-dispatch-denials-v0.json` | unknown visible name returns `ToolNotProjected`; wrong opaque key returns `DispatchIdentityMismatch`; neither searches a same-name alternative | `MARKET-005`; supports future `MARKET-007` |
| `projection-precedence-v0.json` | one case per projection error variant plus canonical-target and dual-fault primary-error ordering | `MARKET-006` |
| `call-precedence-v0.json` | `ToolNotProjected` and dispatch mismatch precede deny-state and argument faults; every call-time variant has an exact case | supports future `MARKET-007` |
| `post-projection-revoke-v0.json` | current deny state narrows a frozen projection; later grant/enable state cannot widen it | supports future `MARKET-007` and later `MARKET-003` |

Fixture JSON is typed test input, not a catalog schema, source JSON Schema or durable-state format. Every case carries a stable name, API, concrete recipe/mutation, exact expected result and precedence semantics; literal schema, argument, dispatch, projection and authority goldens live in the fixture data. Rust deserializes API, recipe, expected outcome and precedence into closed typed enums/records with unknown-field/value denial. The platform-core fixture-matrix test executes every constructor, projection and authorization case from its current recipe, while the agent-runtime fixture consumer executes the run-spec case and constructs `RunSpec`/`AgentRun` only after successful resolution; repository governance pins the complete fixture content and both acceptance bindings.

## 7. Framework evidence mapping

Review date: `2026-07-23`. Claim-level links are retained in the table below; the broader dated baseline is preserved in [`plan/07-runtime-and-integration.md`](../plan/07-runtime-and-integration.md).

| Reference | Borrow | Adapt into P0a | Reject |
|---|---|---|---|
| [Rig](https://github.com/0xPlaygrounds/rig) | per-turn schema/implementation snapshot; one allow-list for exposure and dispatch; typed arguments | build the snapshot only from exact platform package/component/grant/schema identities | last-wins collision; runtime registry as install/grant/approval authority |
| [LangGraph interrupts](https://docs.langchain.com/oss/python/langgraph/interrupts) and [persistence](https://docs.langchain.com/oss/python/langgraph/persistence) | correlation IDs; checkpoint/store distinction; explicit resume | framework thread/checkpoint is adapter state keyed by `platform_run_id`; approval maps to platform effect intent/receipt | checkpoint/store as grant, receipt, budget, audit or replay truth; effects before a restartable interrupt |
| [Pi Agent](https://github.com/earendil-works/pi/tree/main/packages/agent) | validated preflight, call-ID lifecycle and deterministic result projection | preflight consumes a platform decision and frozen projection | mutable/hot-loaded tools, project trust or package config as authorization |
| [goose permissions](https://goose-docs.ai/docs/guides/managing-tools/goose-permissions) and [extensions](https://goose-docs.ai/docs/getting-started/using-extensions) | durable install/default versus session activation; allow/ask/deny UX | map labels to typed capability/grant/confirmation policy; activation only narrows | autonomous default, LLM risk classification or auto-enable that widens authority |
| [Hermes Agent tools runtime](https://hermes-agent.nousresearch.com/docs/developer-guide/tools-runtime), [plugins](https://hermes-agent.nousresearch.com/docs/developer-guide/plugins), [profiles](https://hermes-agent.nousresearch.com/docs/user-guide/profiles) and [architecture](https://hermes-agent.nousresearch.com/docs/developer-guide/architecture) | central schema projection, toolset filtering and fail-safe availability; core registry docs describe later-wins while plugin registration rejects accidental shadowing unless explicit override | registry becomes a read-only projection with namespaced identity and canonical digests | implicit overwrite or explicit plugin override as campus authorization; registry/profile/session approval as campus authority; profile as tenant or filesystem sandbox |

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

Current repository status: manifests, R0 runtime kernel and the pure P0a invocation resolver with typed in-memory snapshots and synthetic fixtures are implemented. `MARKET-005/006` are bound to executable Rust tests. Durable `MARKET-002/003` and cross-boundary `MARKET-007` remain planned; `MARKET-007` still requires the later real application composition seam described above. No current first-party manifest is made runnable by these synthetic fixtures.
