# Market lifecycle contract

## Metadata

- `Status`: Accepted `M20-B1` lifecycle contract; B1-1 typed package-manifest loader and anonymous catalog metadata read model implemented; durable installation/grant/update lifecycle planned; pure invocation resolver and call-time recheck adopted as items 7–8
- `Version`: `market-lifecycle/v0`
- `Last Review`: `2026-07-29`
- `Owning Plan`: [`../plan/04-market-and-plugin-lifecycle.md`](../plan/04-market-and-plugin-lifecycle.md)
- `Large-module Blueprint`: [`../plan/modules/30-market-package-lifecycle.md`](../plan/modules/30-market-package-lifecycle.md)
- `Counterpart Contracts`: [`plugin-package.md`](plugin-package.md), [`invocation-resolution.md`](invocation-resolution.md), [`permissions.md`](permissions.md), [`agent-plugin-boundary.md`](agent-plugin-boundary.md)
- `Authority Defers To`: [`../plan/03-platform-authority.md`](../plan/03-platform-authority.md) for state ownership, [`agent-runtime.md`](agent-runtime.md) for run/effect state, and [`invocation-resolution.md`](invocation-resolution.md) for the adopted projection/recheck decision shapes
- `Acceptance`: implemented `MARKET-005`, `MARKET-006`; planned `MARKET-001`, `MARKET-002`, `MARKET-003`, `MARKET-004`, `MARKET-007`, `PKG-019`, `PKG-020`, `FP-007` (see [`../acceptance/matrix.tsv`](../acceptance/matrix.tsv))
- `Primary Code`: `crates/platform-core/src/market.rs` for B1-1 typed package validation/catalog metadata; current adopted resolver/recheck authority in `crates/platform-core/src/invocation.rs` (items 7–8 only); later durable lifecycle may split `market.rs` when cohesion or size requires it

## 1. Scope and authority

This contract owns the normative lifecycle rules for `M20` — Market and Package Lifecycle. It binds what a published package revision declares, what an exact tenant/user installation pins, how grants and updates behave, and how the adopted invocation resolver/recheck is fed without being rewritten. It defers exact manifest fields to [`plugin-package.md`](plugin-package.md) and the market schema, exact projection/recheck input-output shapes to [`invocation-resolution.md`](invocation-resolution.md), capability classes to [`permissions.md`](permissions.md), and Agent/gateway/executor separation to [`agent-plugin-boundary.md`](agent-plugin-boundary.md).

Ownership remains split across disjoint carriers:

```text
reviewed Market Git revision
  = what a package revision declares

durable installation/grant repositories
  = what one tenant/user has installed and may use

M30 journal
  = what one Agent run proposed/authorized/executed

M90 projection/database
  = replaceable persistence/read model, never a new authority
```

No one carrier MAY collapse publication, installation, enablement, grants, projection, effect intent and execution into one state. `M20` owns the lifecycle rules below; it does not own M00 tenant/user/request/session lifecycle, M30 run phases/journal/`EffectIntent`/idempotency/receipts, M40 tool gateway/executor dispatch, M50 model/provider transport, M51 plugin process isolation/execution, M80 UI/client state, or M90 concrete database/transaction/secret-store/clock/event-transport implementations.

The rules below are normative. Each `MUST`/`MUST NOT` violation means the lifecycle contract is not satisfied and the offending transition fails closed.

## 2. Package revision and publication

### M20-LC-001 — immutable package revision

A published package revision MUST be bound to exact package ID, SemVer, package digest, component declarations/digests/execution identities, capability-manifest digest, source-policy identity, implementation status and catalog revision. Publication MUST NOT mutate an existing revision. Correction MUST create another reviewed revision or revoke the old one.

### M20-LC-002 — publication is not installation

Catalog `installPolicy`, including `default-installed`/`default-enabled` declarations, is policy input only. It MUST NOT create runtime installation or grant rows and MUST NOT be interpreted as proof of runnable state. A manifest's default-install policy is not proof that runtime installation state exists.

### B1-1 package-manifest and catalog-read-model surface

The accepted Rust package boundary is `crate::market`. `load_package_manifest(source: &[u8]) -> Result<ValidatedPackageManifest, PackageLoadError>` is the only B1-1 JSON ingress. It MUST bound source bytes to `1_048_576` before decoding, reject malformed JSON, unknown fields and duplicate object members, then construct immutable private-field values. In particular, the free-form `sourcePolicy` object MUST use duplicate-rejecting loading rather than a last-wins map. `PackageLoadError` and `PackageValidationError` expose only stable field/reason categories; their `Debug` and `Display` MUST NOT contain source JSON or a rejected value.

The exact B1-1 public type set is `PackageField`, `PackageValidationErrorKind`, `PackageValidationError`, `PackageLoadError`, `PackageTier`, `ImplementationStatus`, `InstallPolicyClass`, `InstallPolicy`, `ComponentDeclaration`, `ValidatedPackageManifest`, `CatalogReadModelError` and `CatalogReadModel`, plus `load_package_manifest`. The values reuse M20-owned IDs/digests from `crate::invocation` rather than defining aliases. Validated values expose read-only accessors; `CatalogReadModel` exposes `new`, `catalog_revision`, `packages`, `catalog_digest` and exact `find`. They do not implement public Serde construction or wire serialization; an application adapter must project its own DTO rather than deserializing around validation.

The validated surface projects the existing package schema with these additional bounds and coherence rules:

- package ID retains `^[a-z0-9]+(?:\.[a-z0-9-]+)+$` and is at most `256` UTF-8 bytes; package version is canonical release SemVer `MAJOR.MINOR.PATCH` with no prerelease/build suffix;
- publisher is `1..=128` bytes and matches `^[A-Za-z0-9][A-Za-z0-9_.-]*$`; display name is `1..=256` bytes and an optional present description is `1..=4096` bytes; display/description strings contain no control character;
- at most `64` component declarations exist; each has one unique relative slash-separated ASCII path of at most `512` bytes, no empty/`.`/`..` segment or backslash, and an optional mode of `1..=64` bytes using only ASCII alphanumeric, `.`, `_` or `-`;
- at most `64` unique capability IDs exist; registry membership, risk class and auto-grant eligibility remain B1-3 authority rather than package-schema authority;
- `sourcePolicy` has `1..=32` unique entries; each key matches `^[A-Za-z][A-Za-z0-9_.-]{0,63}$` and each non-empty control-free value is at most `4096` bytes;
- default enable requires default install; `UserInstalledPlugin` is never default-installed or default-enabled; `FirstParty` and `FirstPartySystemPlugin` must agree and the current system policy remains default-installed, default-enabled and user-disableable;
- `planned` carries no component declaration, while `implemented` carries at least one. `development` remains metadata and is never runnable proof.

Filesystem existence, component artifact digest, execution identity, publisher/capability admission and publication review are deliberately outside the source decoder. A validated manifest is not a published `CatalogPackageRevision`, installation, grant or resolver input.

#### Canonical declaration digests

All counts use unsigned 64-bit big-endian encoding. A string is its unsigned 64-bit big-endian UTF-8 byte length followed by exact bytes. Booleans use one byte (`0`/`1`). Optional strings use a one-byte absence/presence tag (`0`/`1`) followed, when present, by the string encoding. No Unicode normalization occurs.

Enum tags are exact:

```text
PackageTier:          FirstParty=1, VerifiedCommunityText=2, VerifiedRemoteMcp=3
ImplementationStatus: planned=1, development=2, implemented=3
InstallPolicyClass:   FirstPartySystemPlugin=1, UserInstalledPlugin=2
ComponentKind:        SkillComponent=1, DeclarativeResourcePack=2,
                      McpServerComponent=3, NativeRustComponent=4
```

After its domain separator, `package_digest` encodes in order: package ID, canonical version text, publisher, tier tag, display name, optional description, implementation-status tag, install-policy-class tag, the three install-policy booleans, canonical component-declaration collection, canonical capability collection and canonical source-policy collection. Each collection begins with its count. A component encodes path, kind tag and optional mode; a capability encodes its ID; a source-policy entry encodes key then value. The three subset digests use the same respective collection encoding after their own domain separator. `catalog_digest` encodes catalog-revision text, package count and each already-sorted package digest string after its domain separator.

`ValidatedPackageManifest` exposes four lowercase `sha256:<64 hex>` values:

```text
package_digest                         market-package-manifest/v0\0
component_declaration_set_digest       market-component-declarations/v0\0
capability_manifest_digest             market-capability-manifest/v0\0
source_policy_digest                   market-source-policy/v0\0
```

Capabilities are bytewise-ID sorted, source-policy entries key sorted, and component declarations sorted by `(path, kind tag, mode)`. Source JSON whitespace/member order and the source order of these semantically unordered collections MUST NOT affect canonical bytes or digests after duplicate rejection. `package_digest` binds every validated declaration field and each canonical collection; the three subset digests bind their complete respective collections independently. They do not claim component artifact or execution-identity evidence.

#### Anonymous metadata read model

`CatalogReadModel::new(catalog_revision, manifests)` binds one validated `CatalogRevision`, sorts exact package revisions by bytewise `(package_id, canonical package-version text)`, rejects duplicate exact `(id, version)` and computes `catalog_digest` under `market-catalog-read-model/v0\0` from the revision plus ordered package digests. It exposes ordered metadata and exact `(id, version)` lookup only; it defines no latest-version policy.

The read model contains declaration metadata and digests only. It MUST NOT contain or infer installed, enabled, granted, runnable, available, executor or adapter state. B1-1 is supporting domain evidence for planned `MARKET-001`; M10/M80 anonymous API/browser evidence is still required before that row can become implemented.

## 3. Installation state

### M20-LC-003 — exact installation pin

Each installation ID MUST belong to exactly one tenant/user and MUST pin one exact package revision, package digest, component identity set, configuration revision and capability-manifest digest. An installation MUST NOT silently float to another package version or component set.

### M20-LC-004 — distinct installation states

The MVP managed installation states MUST be exactly:

```text
InstalledDisabled
Enabled
Disabled
Revoked
Uninstalled
```

`Uninstalled` is explicit terminal history when retained; repository absence alone MUST NOT be interpreted as a successful uninstall event. `Revoked` and `Uninstalled` MUST be terminal for that installation identity. Reinstallation MUST use a new installation identity.

### M20-LC-005 — enable preconditions

Enable MUST succeed only if all of the following hold:

- the exact package revision is published and not revoked;
- configuration references validate without exposing raw secrets;
- required grants are active and exact;
- capability/source/execution policy is admitted;
- the expected installation revision matches.

A failed enable MUST emit no enabled state.

### M20-LC-006 — disable/revoke fail closed

Disable or revoke MUST change current authority before any later projection or recheck. Uncertain propagation or a repository precondition conflict MUST deny new discovery and calls. In-flight runs MUST retain their frozen projection, but call-time current denial still applies.

## 4. Grants and permission expansion

### M20-LC-007 — grants are separate authority

Installation MUST NOT imply a grant. Grant creation, replacement and revoke MUST be explicit, tenant/user/installation/capability/scope bound, versioned, and MUST NOT be requestable by model output. Package authors MUST NOT set risk class or auto-grant eligibility; the capability registry owns those.

### M20-LC-008 — permission expansion requires reapproval

A staged update that adds capabilities, widens object scope, changes capability class, source policy or execution identity, or otherwise increases authority MUST NOT auto-apply or auto-enable. Exact unchanged permissions MAY be eligible for later rollout policy, but still require an exact tested update target and a durable receipt.

## 5. Update and rollback

### M20-LC-009 — update and rollback pin exact targets

Stage, apply and rollback MUST operate on exact reviewed package revisions. Apply MUST use expected installation/update revisions and MUST preserve a tested rollback target. A failed apply MUST leave the prior accepted installation authority intact or the installation disabled; it MUST NOT widen permissions or silently fall back.

## 6. Semantic repositories

### M20-LC-010 — semantic repositories use preconditions

M20 repositories MUST expose semantic operations with expected revisions/sequences and typed conflicts; no generic record-store or arbitrary query API is admitted. A state transition and its event/audit append MUST be atomic at the port contract. Duplicate command identities MUST be idempotent only when payload and prior outcome are identical; conflicting reuse MUST fail.

### M20-LC-011 — decide/evolve/replay duality

Managed installation, grant, publication and update aggregates MUST follow:

```text
Command
→ validate current aggregate + expected revision
→ decide typed Event(s)
→ atomically persist
→ evolve
→ deterministic replay
```

Every persisted event MUST be reachable through `decide`. Sequence gaps, duplicates, reordering, overflow, impossible initial events, post-terminal transitions and redundant-field forgery MUST fail closed.

## 7. Secrets

### M20-LC-012 — no raw secrets

M20 MUST accept only typed non-secret values and opaque tenant-scoped `SecretRef`s. Raw secret bytes MUST NOT appear in package Git, domain events, normal logs, browse projections, denial payloads or audit receipts. Secret existence and ownership MUST be checked through a narrow port without resolving the secret value into M20.

## 8. Adopted resolver and composition

### M20-LC-013 — adopted resolver is not rewritten

M20 lifecycle code MUST assemble exact `CatalogPackageRevision`, `PluginInstallationSnapshot`, `CapabilityGrantSnapshot` and `InvocationPolicySnapshot` inputs for the adopted resolver (items 7–8). It MUST NOT duplicate or bypass `InvocationResolver::resolve_projection` or `authorize_call`. Existing `crates/platform-core/src/invocation.rs` remains the authority for items 7–8; M20 MUST NOT move or duplicate it.

### M20-LC-014 — composition order remains external

Production call composition is owned by the composition root, not by M20. The authoritative ordering lives in [`agent-plugin-boundary.md`](agent-plugin-boundary.md) §7 and [`../plan/modules/50-tool-gateway-execution.md`](../plan/modules/50-tool-gateway-execution.md) §6. For illustration, the flow from M20's perspective is:

```text
M20 projection/recheck
→ M30 proposal
→ composition invokes M40 prepare
→ composition records M30 effect intent
→ M51/peer executor
→ M40 bounded outcome
→ composition records M30 effect receipt
→ M40 correlated result
→ M30 result state
```

M20 MUST NOT create M30 `EffectIntent` and MUST NOT call an executor.

### M20-LC-015 — historical projections are immutable

Package update, disable or revoke MUST change only future projections and current call-time denial. It MUST NOT mutate an already frozen `ToolProjectionSnapshot`, an in-flight `RunSpec`, or a historical receipt.

## 9. Non-goals and current status

This contract does not own:

- anonymous browse/detail delivery through M10/M80 application/query adapters remains planned (`MARKET-001`); the B1-1 anonymous metadata domain read model is implemented but is not delivery evidence;
- durable installation/grant/enable/disable/upgrade mutation (planned, `MARKET-002`/`MARKET-003`/`MARKET-004`);
- a production database/repository transaction or TOCTOU closure (planned);
- provider, network, MCP, daemon HTTP/SSE or UI adapters;
- external tool execution, durable journal or crash recovery;
- M30 `EffectIntent`, M40 executor dispatch, or M51 process isolation.

Current repository status: the pure P0a invocation resolver and call-time recheck with typed in-memory snapshots and synthetic fixtures are implemented and adopted (`MARKET-005`/`MARKET-006`). B1-1 also implements the bounded typed package-manifest loader, canonical declaration digests and immutable anonymous catalog metadata domain read model in `crate::market`. It does not create a durable repository, installation aggregate, grant aggregate, update/rollback, application composition or M10/M80 browse delivery. No current first-party manifest is made runnable by B1-1. Future implementation slices and their intended bindings are listed in [`../acceptance/matrix.tsv`](../acceptance/matrix.tsv) and remain `planned` until their exact evidence exists.
