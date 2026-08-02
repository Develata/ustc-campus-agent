# Market lifecycle contract

## Metadata

- `Status`: Accepted `market-lifecycle/v0` contract; canonical B1–B5 evidence and the bounded `M20-B6` update/rollback domain plus semantic fake evidence are implemented, while the exact `M20-B7-A1` application façade and `M20-B7-B` test-only composition contracts are accepted but unimplemented; production callers/issuers, durable lifecycle/update repositories, crash recovery, artifact switching, wire/client delivery and production ToolGateway composition remain planned; pure invocation resolver and call-time recheck remain the adopted items 7–8
- `Version`: `market-lifecycle/v0`
- `Last Review`: `2026-08-02`
- `Owning Plan`: [`../plan/04-market-and-plugin-lifecycle.md`](../plan/04-market-and-plugin-lifecycle.md)
- `Large-module Blueprint`: [`../plan/modules/30-market-package-lifecycle.md`](../plan/modules/30-market-package-lifecycle.md)
- `Counterpart Contracts`: [`plugin-package.md`](plugin-package.md), [`invocation-resolution.md`](invocation-resolution.md), [`permissions.md`](permissions.md), [`agent-plugin-boundary.md`](agent-plugin-boundary.md)
- `Authority Defers To`: [`../plan/03-platform-authority.md`](../plan/03-platform-authority.md) for state ownership, [`agent-runtime.md`](agent-runtime.md) for run/effect state, and [`invocation-resolution.md`](invocation-resolution.md) for the adopted projection/recheck decision shapes
- `Acceptance`: implemented `MARKET-005`, `MARKET-006`; planned `MARKET-001`, `MARKET-002`, `MARKET-003`, `MARKET-004`, `MARKET-007`, `PKG-019`, `PKG-020`, `FP-007` (see [`../acceptance/matrix.tsv`](../acceptance/matrix.tsv)); exact B7 contract acceptance is not implementation evidence and promotes no row
- `Primary Code`: `crates/platform-core/src/market.rs` for `M20-B1` package validation/catalog metadata; `crates/platform-core/src/market/capability.rs` for `M20-B2`; `crates/platform-core/src/market/installation.rs` for `M20-B3-s1`; `crates/platform-core/src/market/grant.rs` for `M20-B4`; `crates/platform-core/src/market/authority.rs` for bounded `M20-B5` carrier-by-carrier read transactions and authority assembly; `crates/platform-core/src/market/update.rs` and `crates/platform-core/tests/market_package_update.rs` for bounded `M20-B6` update/rollback domain and semantic package-update repository evidence; `crates/platform-core/src/invocation.rs` for adopted resolver/recheck items 7–8; accepted future B7 carriers are `crates/platform-core/src/market/application.rs` for A1 and composition-root test support under `apps/ustc-agentd/tests/` for B7-B, neither of which exists yet

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

### M20-B1 (historical `B1-1`) package-manifest and catalog-read-model surface

The accepted Rust package boundary is `crate::market`. `load_package_manifest(source: &[u8]) -> Result<ValidatedPackageManifest, PackageLoadError>` is the only B1-1 JSON ingress. It MUST bound source bytes to `1_048_576` before decoding, reject malformed JSON, unknown fields and duplicate object members, then construct immutable private-field values. In particular, the free-form `sourcePolicy` object MUST use duplicate-rejecting loading rather than a last-wins map. `PackageLoadError` and `PackageValidationError` expose only stable field/reason categories; their `Debug` and `Display` MUST NOT contain source JSON or a rejected value.

The exact B1-1 public type set is `PackageField`, `PackageValidationErrorKind`, `PackageValidationError`, `PackageLoadError`, `PackageTier`, `ImplementationStatus`, `InstallPolicyClass`, `InstallPolicy`, `ComponentDeclaration`, `ValidatedPackageManifest`, `CatalogReadModelError` and `CatalogReadModel`, plus `load_package_manifest`. The values reuse M20-owned IDs/digests from `crate::invocation` rather than defining aliases. Validated values expose read-only accessors; `CatalogReadModel` exposes `new`, `catalog_revision`, `packages`, `catalog_digest` and exact `find`. They do not implement public Serde construction or wire serialization; an application adapter must project its own DTO rather than deserializing around validation.

The validated surface projects the existing package schema with these additional bounds and coherence rules:

- package ID retains `^[a-z0-9]+(?:\.[a-z0-9-]+)+$` and is at most `256` UTF-8 bytes; package version is canonical release SemVer `MAJOR.MINOR.PATCH` with no prerelease/build suffix;
- publisher is `1..=128` bytes and matches `^[A-Za-z0-9][A-Za-z0-9_.-]*$`; display name is `1..=256` bytes and an optional present description is `1..=4096` bytes; display/description strings contain no control character;
- at most `64` component declarations exist; each has one unique relative slash-separated ASCII path of at most `512` bytes, no empty/`.`/`..` segment or backslash, and an optional mode of `1..=64` bytes using only ASCII alphanumeric, `.`, `_` or `-`;
- at most `64` unique capability IDs exist; registry membership, risk class and auto-grant eligibility remain `M20-B2` `capability-registry` authority rather than package-schema authority;
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

### M20-B3-s1 managed installation surface

This subsection freezes the exact public surface of the bounded `M20-B3-s1` slice: a pure managed-installation aggregate plus a semantic in-memory repository fake under `platform-core`. It specializes `M20-LC-003` through `M20-LC-011` into an implementation-bound contract. The slice was allowed to land before full `M20-B2` only because it cannot mint production enable evidence and cannot promote installation/enable acceptance; production enable-evidence issuance remains future grant/authority-assembly/composition work.

The flow MUST be exactly:

```text
validated command
→ pure decide
→ typed event
→ pure evolve
→ atomic semantic repository commit
→ deterministic replay
```

#### Module boundary and reused authority

The Rust module boundary MUST be `crate::market::installation` under `crates/platform-core/src/market/installation.rs`, with `crates/platform-core/src/market.rs` declaring `pub mod installation;`. The existing `market.rs` manifest/catalog implementation MUST NOT be moved or rewritten. Public lifecycle paths live under `crate::market::installation`.

No Serde derives or public wire deserialization are admitted for installation-domain values. No new framework, database or network dependency is admitted.

The slice MUST reuse exact existing nominal types from `crate::identity` (`TenantId`, `UserId`) and from `crate::invocation` (`InstallationId`, `InstallationRevision`, `PackageId`, `PackageVersion`, `CatalogRevision`, `ComponentId`, `ComponentVersion`, `ComponentKind`, `ExecutionIdentity`, `CapabilityId`, `Sha256Digest`, `InstalledComponentIdentity`, `PluginInstallationSnapshot`, and resolver `InstallationState`) without aliasing or re-wrapping. The existing three-variant `invocation::InstallationState` stays unchanged.

#### New value algebra

All new values MUST use private fields, bounded checked constructors and read-only accessors. No `Default` and no unchecked public constructor are admitted.

IDs and counters:

- `InstallationCommandId`: bounded opaque text with canonical grammar `cmd:[A-Za-z0-9._:-]{1,124}`.
- `InstallationEventSequence(u64)`: the first persisted event is `1`; successor is checked and overflow fails closed.
- `ConfigurationRevision(u64)`: initial value is `1`; it increments only on successful Configure and overflow fails closed.
- `ConfigurationKey`: ASCII `[A-Za-z][A-Za-z0-9_.-]{0,63}`.
- `NonSecretText`: non-empty, control-free UTF-8 of at most `4096` bytes.
- `SecretRefId`: opaque grammar `secret-ref:[A-Za-z0-9._:-]{1,118}`.
- `SecretRef { tenant_id, id }`: reference only. It MUST NOT contain a digest or fingerprint of resolved secret material. Secret bytes and material-derived hashes never enter `M20`.

Configuration:

```rust
pub enum ConfigurationValue {
    Text(NonSecretText),
    Integer(i64),
    Boolean(bool),
    Secret(SecretRef),
}
```

`InstallationConfiguration` MUST be an immutable `BTreeMap<ConfigurationKey, ConfigurationValue>` with at most `128` entries. Construction MUST reject duplicate keys and cross-tenant `SecretRef`s. It MUST compute a deterministic lowercase `sha256:<64 hex>` digest under domain separator `market-installation-configuration/v0\0`, with explicit typed tags and length-prefix encoding. The digest binds opaque reference IDs, never secret material.

Pins:

- `InstalledComponentPin` MUST bind exact component ID, kind, version, digest and execution identity.
- `InstallationPackagePin` MUST bind catalog revision, package ID/version/digest, sorted unique component pins, component-set digest and capability-manifest digest.

Pin constructors MUST canonicalize ordering and reject duplicate component IDs.

#### Managed lifecycle

The managed lifecycle state MUST be a distinct enum named exactly:

```rust
pub enum ManagedInstallationState {
    InstalledDisabled,
    Enabled,
    Disabled,
    Revoked,
    Uninstalled,
}
```

Legal transitions:

| command | from | to / rule |
|---|---|---|
| Install | repository absence | `InstalledDisabled` |
| Configure | `InstalledDisabled` or `Disabled` | same state; config revision `+1` |
| Enable | `InstalledDisabled` or `Disabled` | `Enabled`, only with exact evidence |
| Disable | `Enabled` | `Disabled` |
| Revoke | any nonterminal state | `Revoked` |
| Uninstall | any nonterminal state | `Uninstalled` |

`Revoked` and `Uninstalled` MUST be terminal. Repository absence is not `Uninstalled`. Reinstallation MUST use a new installation ID. Configure while `Enabled` MUST be rejected: the caller MUST Disable first, preventing a silent live-authority change.

Projection into the existing resolver MUST be pure and deny-side only, through a method named exactly `to_resolver_snapshot`. It MUST NOT invoke or modify the resolver and MUST NOT imply grants or readiness. The mapping is:

- `InstalledDisabled`/`Disabled` → resolver `Disabled`;
- `Enabled` → resolver `Enabled`;
- `Revoked` → resolver `Revoked`;
- `Uninstalled` → no resolver snapshot.

#### Commands, events and revisions

`InstallationCommand` MUST be a private-action, private-field value with checked associated constructors for Install/Configure/Enable/Disable/Revoke/Uninstall. Every command MUST carry one `InstallationCommandId` and exact installation ID. Every non-Install command MUST carry `expected_revision`.

`InstallationEvent` MUST be an envelope with private fields `sequence`, `post_revision`, `command_id` and `kind`. `InstallationEventKind` MUST contain `Installed`/`Configured`/`Enabled`/`Disabled`/`Revoked`/`Uninstalled` payloads. Every event MUST embed the originating command ID. `Installed` MUST carry all initial aggregate pins/configuration; later events MUST carry only complete transition payloads. No event MAY carry raw secret material.

Revision is deterministic: the post-event `InstallationRevision` MUST be exactly `installation-revision:<sequence>`. `decide` MUST construct it; `evolve` MUST independently recompute and reject mismatch as redundant-field forgery.

Public pure functions:

```rust
pub fn decide(
    current: Option<&InstallationAggregate>,
    command: &InstallationCommand,
) -> Result<InstallationEvent, InstallationDecisionError>;

pub fn evolve(
    current: Option<InstallationAggregate>,
    event: &InstallationEvent,
) -> Result<InstallationAggregate, InstallationReplayError>;

pub fn replay<'a>(
    events: impl IntoIterator<Item = &'a InstallationEvent>,
) -> Result<Option<InstallationAggregate>, InstallationReplayError>;
```

Exact Rust lifetime/iterator syntax MAY be adjusted for Rust 2021 compilation, but semantics and ownership remain.

#### Enable evidence boundary

`EnablePreconditionEvidence` MUST record exact authority-binding digests/identities for:

- installation ID and expected installation revision;
- package/component-set/configuration/capability-manifest pins;
- exact active grant-set snapshot digest;
- exact policy/source/execution admission snapshot digest.

It MUST expose read-only accessors and a deterministic evidence digest. Its minting constructor MUST NOT be public; visibility MUST be no wider than `pub(in crate::market)`. There MUST be no public boolean or enum constructor that a caller can use to assert admission. Unit tests inside the module MAY mint fixture evidence; production issuance is deferred to a future authority-assembly module under `crate::market`.

`decide(Enable)` MUST check every evidence binding against current aggregate state. Unknown, absent or mismatched evidence MUST fail closed and emit no event.

#### Error and replay discipline

Construction errors MUST reject invalid grammar, bounds, duplicates and cross-tenant secret references before a command exists.

Repository execution precedence MUST be:

1. command-ledger exact duplicate/conflicting reuse;
2. aggregate missing/already-present as applicable;
3. terminal-state guard;
4. expected-revision mismatch;
5. illegal/already-in-state transition;
6. enable/configuration coherence.

`CommandConflict` MUST be checked before current state so a reused command ID cannot produce a different outcome or leak state-dependent behavior. An exact duplicate MUST return the stored prior receipt.

Replay MUST reject, with typed errors:

- a non-`Installed` initial event;
- sequence gap, duplicate, reorder and overflow;
- duplicate command ID in successful event history;
- post-terminal event;
- illegal transition;
- post-revision mismatch/forgery;
- tenant/installation/package/configuration redundant-field mismatch.

All errors MUST expose stable categories only. `Debug` and `Display` MUST NOT reveal configuration values, secret references, rejected IDs or source payloads.

#### Semantic repository and persisted command receipts

The semantic port MUST be:

```rust
pub trait InstallationRepository {
    fn execute(
        &mut self,
        command: InstallationCommand,
    ) -> Result<InstallationCommandReceipt, InstallationRepositoryError>;

    fn load_exact(
        &self,
        id: &InstallationId,
    ) -> Result<Option<InstallationSnapshot>, InstallationRepositoryError>;

    fn event_history(
        &self,
        id: &InstallationId,
    ) -> Result<Vec<InstallationEvent>, InstallationRepositoryError>;
}
```

No generic record-store or arbitrary query API is admitted.

`InstallationCommandReceipt` MUST be persisted for both accepted and domain-rejected commands. It MUST bind the complete original command and exact prior outcome:

- accepted: resulting snapshot plus exact event;
- rejected: exact typed `InstallationDecisionError` and no event.

The in-memory fake MUST keep a global command ledger `command_id → {complete command, receipt}`. Therefore:

- identical command ID plus identical complete command returns the exact stored receipt and performs no append;
- same command ID plus different command is `CommandConflict` regardless of current aggregate state;
- a previously rejected command remains rejected identically after later state changes.

For a new command, state/event/receipt commit MUST be atomic. Injected persistence failure MUST occur before commit and record neither receipt nor event, so a retry may proceed. Corrupt replay MUST fail closed. No public arbitrary insertion hook is admitted; test-only fixture construction remains private/unit-test scoped.

`InMemoryInstallationRepository` MUST be provided as the semantic fake with narrowly named one-shot failure injection.

## 4. Grants and permission expansion

### M20-LC-007 — grants are separate authority

Installation MUST NOT imply a grant. Grant creation, replacement and revoke MUST be explicit, tenant/user/installation/capability/scope bound, versioned, and MUST NOT be requestable by model output. Package authors MUST NOT set risk class or auto-grant eligibility; the capability registry owns those.

### M20-LC-008 — permission expansion requires reapproval

A staged update that adds capabilities, widens object scope, changes capability class, source policy or execution identity, or otherwise increases authority MUST NOT auto-apply or auto-enable. Exact unchanged permissions MAY be eligible for later rollout policy, but still require an exact tested update target and a durable receipt.

Bounded `M20-B6` now implements a complete package-level change classifier, exact update-approval/readiness/confirmation/rollback evidence values and fail-closed no-auto-reapproval rules in `crate::market::update`; this is supporting domain evidence only. It does not issue production grants, enable an installation, satisfy grant reapproval, switch artifacts or prove B7 current-call/application composition.

### M20-B4 grant-domain surface

This subsection owns the exact public contract of the implemented bounded `M20-B4` grant-domain slice: a pure grant aggregate plus a semantic in-memory repository fake under `crate::market::grant`, with decide/evolve/replay and focused Rust evidence. The contract and bounded implementation are accepted as supporting domain evidence only. This slice mints no production grant, assembles no `EnablePreconditionEvidence`, creates no invocation candidate, executes no tool, writes no M30 intent and implements no M90 persistence. It specializes `M20-LC-007` and `M20-LC-008` without claiming B5 authority assembly or production composition.

#### Module boundary and reused authority

The Rust module boundary is `crate::market::grant` under `crates/platform-core/src/market/grant.rs`, with `crates/platform-core/src/market.rs` declaring `pub mod grant;`. The existing `market.rs`, `market::capability` and `market::installation` implementations MUST NOT be moved or rewritten. Public grant lifecycle paths live under `crate::market::grant`.

No Serde derives or public wire deserialization are admitted for grant-domain values. No new framework, database or network dependency is admitted.

The slice MUST reuse, not alias or rewrap, exact existing nominal types from `crate::identity` (`TenantId`, `UserId`) and from `crate::invocation` (`CatalogRevision`, `PackageId`, `PackageVersion`, `InstallationId`, `InstallationRevision`, `CapabilityId`, `ObjectScope`, `ConfirmationPolicy`, `GrantSnapshotId`, `GrantVersion`, `GrantState`, `CapabilityGrantSnapshot`, `Sha256Digest`), and from `crate::market::capability` (`CapabilityDefinition`, `CapabilityPolicyChange`, `CapabilityRegistry`, `CapabilityRegistryRevision`, `CapabilityStatus`, `ScopeKind`, `compare_capability_definitions`), plus `crate::market::ValidatedPackageManifest` and `crate::market::installation::{InstallationSnapshot, ManagedInstallationState}`. The existing `GrantSnapshotId` is the immutable grant stream/snapshot-lineage identity used by the adopted resolver; it stays stable for one grant aggregate. `GrantVersion` distinguishes every accepted state transition. B4 MUST NOT add a second `GrantId` or change the existing invocation structs.

One bounded prerequisite extension is admitted in `crate::market::capability`:

```rust
pub fn compare_capability_definitions(
    old: Option<&CapabilityDefinition>,
    new: Option<&CapabilityDefinition>,
) -> CapabilityPolicyChange;
```

It owns the existing `None`/added/removed/revoked and axis-change semantics. Existing `compare_capability_policy` MUST delegate to it. B4 `decide`/`evolve`/`replay` call this helper; they MUST NOT copy its classification table or trust a caller-supplied `CapabilityPolicyChange`.

#### Public declaration surface

The bounded implementation MUST name exactly these public items; fields remain private unless this subsection explicitly says otherwise:

```text
GrantConstructionError
GrantDecisionError
GrantReplayError
GrantCommandId
GrantApprovalId
GrantEventSequence
GrantScope
GrantInvalidationReason
GrantChangeClass
GrantAdmissionEvidence
GrantCommand
GrantEventKind
GrantEvent
GrantAggregate
GrantSnapshot                    // type alias to GrantAggregate
decide
evolve
replay
GrantRepository
GrantCommandReceipt
GrantCommandOutcome
GrantRepositoryError
InMemoryGrantRepository
```

No public Serde construction, public struct fields, `Default` for authority-bearing values, unchecked constructor, arbitrary insertion hook, generic record store, time/network/database/framework type or caller-supplied authority boolean is admitted.

#### Checked IDs and sequence

The existing `GrantSnapshotId` is reused with private fields, but Issue/admission construction adds one exact semantic grammar check:

```text
grant:[A-Za-z0-9._:-]{1,122}
```

The generic invocation parser alone is not sufficient evidence of B4 identity validity. A noncanonical existing `GrantSnapshotId` is rejected before any command/event is built.

The existing `GrantVersion` is reused with private fields. Expected versions supplied to non-Issue command constructors MUST have the exact canonical form:

```text
grant-version:[1-9][0-9]*
```

The decimal suffix MUST parse as nonzero `u64`, with no sign, whitespace, leading zero or overflow. Accepted event versions are still derived only from `GrantEventSequence`; external expected versions never mint post-state authority.

`GrantCommandId` is private-field checked text with exact grammar:

```text
grant-cmd:[A-Za-z0-9._:-]{1,118}
```

`GrantApprovalId` is private-field checked text with exact grammar:

```text
grant-approval:[A-Za-z0-9._:-]{1,113}
```

It identifies a trusted explicit review decision. Model output cannot mint or request it. Possession of the string alone is not sufficient to construct admission evidence because the evidence constructor is not public.

`GrantEventSequence` is a private `u64`; the first accepted event is `1`; zero is invalid; the successor uses checked arithmetic and overflow fails closed.

Version rule: after every accepted event,

```text
GrantVersion = "grant-version:<event-sequence>"
```

`evolve` independently recomputes the version and rejects forged redundant values.

#### Closed grant scope algebra

`GrantScope` is a private-field value with only these constructors:

```rust
pub fn campus_public() -> Result<GrantScope, GrantConstructionError>;

pub fn tenant_private_user(
    tenant_id: TenantId,
    user_id: UserId,
) -> Result<GrantScope, GrantConstructionError>;
```

Read-only accessors expose `scope_kind`, `object_scope`, and optional tenant/user bindings. Exact forms:

- `CampusPublic` maps to existing `ObjectScope::parse("scope:campus-public")`.
- `TenantPrivateUser` stores the exact tenant/user values and maps to `scope:tenant-user:sha256:<64 lowercase hex>`.
- The tenant-user hash input is domain separator `market-grant-tenant-user-scope/v0\0`, followed by tenant ID and user ID, each as unsigned 64-bit big-endian UTF-8 byte length plus exact bytes.
- No Unicode normalization occurs.
- `OperatorAdministrative` has no B4 constructor and fails closed.
- No constructor accepts an arbitrary `ObjectScope` string.
- Admission evidence and command decision MUST reject a tenant-private scope whose tenant/user differs from the grant authority binding.

#### Explicit admission evidence

`GrantAdmissionEvidence` is public as a read-only value because future B5 code must carry it, but its minting constructor is exactly `pub(in crate::market)`. External callers cannot assert approval.

The constructor signature is exactly:

```rust
#[allow(clippy::too_many_arguments)]
pub(in crate::market) fn from_authority_bindings(
    snapshot_id: GrantSnapshotId,
    approval_id: GrantApprovalId,
    installation: &InstallationSnapshot,
    package: &ValidatedPackageManifest,
    capability_id: CapabilityId,
    scope: GrantScope,
    confirmation_policy: ConfirmationPolicy,
    registry: &CapabilityRegistry,
) -> Result<Self, GrantConstructionError>;
```

The constructor internally extracts tenant/user/installation identity, exact installation revision, catalog/package pin and capability-manifest digest. Read-only accessors use the exact binding names plus `catalog_revision`, `package_id`, `package_version`, `package_digest`, `capability_definition`, `capability_registry_revision`, `capability_definition_digest` and `evidence_digest`; no mutable accessor exists.

It binds all of:

```text
GrantSnapshotId
GrantApprovalId
TenantId
UserId
InstallationId
expected InstallationRevision
CatalogRevision
PackageId
PackageVersion
package Sha256Digest
CapabilityId
GrantScope
ConfirmationPolicy
capability-manifest Sha256Digest
CapabilityRegistryRevision
complete cloned CapabilityDefinition binding, including its Sha256Digest
evidence Sha256Digest
```

The evidence digest uses domain separator `market-grant-admission-evidence/v0\0` and length-prefixed canonical fields, including every exact `CapabilityDefinition` axis and enum tag.

The constructor receives the actual typed `InstallationSnapshot`, `ValidatedPackageManifest`, and `CapabilityRegistry` plus the exact `CapabilityId`, not separately caller-supplied installation/package/registry bindings and not caller-supplied risk/scope/status booleans. It cross-checks the installation package pin against the manifest, requires the capability in the exact manifest, performs exact registry lookup, and internally clones the registry revision and found `CapabilityDefinition`. It MUST reject:

- missing identity coherence;
- noncanonical B4 `GrantSnapshotId`;
- terminal `ManagedInstallationState::Revoked|Uninstalled`;
- installation/manifest package ID, version, package digest or capability-manifest digest mismatch; catalog revision is bound only from the installation pin and is not attributed to `ValidatedPackageManifest`;
- capability absent from the exact package manifest;
- capability missing from the exact registry;
- non-`Active` capability definition;
- `ScopeKind` mismatch;
- `OperatorAdministrative`;
- tenant/user mismatch;
- `ConfirmationPolicy::Allow` when the definition default is `Ask`;
- any evidence-digest mismatch during decision/replay.

Issue accepts only an active actual capability definition from the supplied registry and a nonterminal exact installation/package binding. These typed inputs prove coherence only: `ValidatedPackageManifest` remains non-authoritative declaration data, and B4 does not prove that the catalog revision is currently published/runnable or that the installation snapshot is transaction-current. B5 MUST obtain these carriers from the active catalog/installation repositories in one authority-assembly transaction. Replace compares the aggregate's prior complete definition with the evidence's actual complete definition via `compare_capability_definitions`; neither the command nor the evidence carries a caller-selected change classification.

B4 admits **explicit reviewed grants only**. It does not implement auto-grant issuance. `AutoGrantDisposition` remains B2 policy input for a later separately authorized composition/bootstrap contract. There is no B4 `AutoGrant`, `Trusted`, `Approved(bool)` or equivalent bypass.

#### Grant change classification

`GrantChangeClass` is exactly:

```rust
pub enum GrantChangeClass {
    Unchanged,
    Narrowed,
    ReapprovalRequired,
}
```

For replacement, grant code first obtains `CapabilityPolicyChange` from `compare_capability_definitions(Some(old), Some(new))` and then computes:

- exact scope, confirmation policy, capability-manifest digest and definition digest with computed `CapabilityPolicyChange::Unchanged` is `Unchanged`; registry revision may advance without changing this classification;
- `Allow -> Ask` or `CapabilityPolicyChange::Narrowed`, with no other expansion, is `Narrowed`;
- `Ask -> Allow`, capability-manifest change, computed `CapabilityPolicyChange::ExpansionRequiresReapproval` or any other widening is `ReapprovalRequired`;
- a missing/revoked/non-active new capability definition cannot construct replacement evidence at all; the existing grant is narrowed through MarkStale/Revoke rather than Replace;
- scope change is not classified for in-stream replacement: it is rejected as `ScopeChangeRequiresNewGrant` and requires revoke plus a new `GrantSnapshotId`/Issue flow;
- every Replace requires a fresh `GrantApprovalId`, even when unchanged or narrowed;
- `ReapprovalRequired` therefore cannot be accepted through an automatic path.

#### State graph

The aggregate uses existing `GrantState` exactly:

```text
Issue    : absence                 -> Active
Replace  : Active|Stale|Expired    -> Active
MarkStale: Active                  -> Stale
Expire   : Active|Stale            -> Expired
Revoke   : Active|Stale|Expired    -> Revoked
```

Rules:

- `Revoked` is terminal for that `GrantSnapshotId`.
- Regrant after revoke uses a new `GrantSnapshotId` and a fresh explicit approval.
- Replace requires exact expected version, exact same scope, fresh complete evidence and a fresh approval ID.
- MarkStale carries one closed `GrantInvalidationReason`.
- Repeated same-state transitions are rejected; they are not silent no-ops.
- No command widens scope or changes tenant/user/installation/capability identity in place.
- State narrowing is immediate for future projections and current call-time recheck; historical projections remain immutable.

`GrantInvalidationReason` is exactly:

```rust
pub enum GrantInvalidationReason {
    CapabilityManifestChanged,
    CapabilityDefinitionChanged,
    InstallationChanged,
    PolicyChanged,
}
```

#### Commands and errors

`GrantCommand` is a private-action, private-field value. Checked associated constructors are exactly:

```rust
pub fn issue(
    command_id: GrantCommandId,
    evidence: GrantAdmissionEvidence,
) -> Result<Self, GrantConstructionError>;

pub fn replace(
    command_id: GrantCommandId,
    expected_version: GrantVersion,
    evidence: GrantAdmissionEvidence,
) -> Result<Self, GrantConstructionError>;

pub fn mark_stale(
    command_id: GrantCommandId,
    snapshot_id: GrantSnapshotId,
    expected_version: GrantVersion,
    reason: GrantInvalidationReason,
) -> Result<Self, GrantConstructionError>;

pub fn expire(
    command_id: GrantCommandId,
    snapshot_id: GrantSnapshotId,
    expected_version: GrantVersion,
) -> Result<Self, GrantConstructionError>;

pub fn revoke(
    command_id: GrantCommandId,
    snapshot_id: GrantSnapshotId,
    expected_version: GrantVersion,
) -> Result<Self, GrantConstructionError>;
```

Every command carries `GrantCommandId` and `GrantSnapshotId`. Issue and Replace obtain the snapshot identity only from their `GrantAdmissionEvidence`; there is no second caller-supplied identity to disagree with it. Every non-Issue command carries exact expected `GrantVersion`. Read-only accessors are exactly `command_id` and `snapshot_id`; actions/payloads remain private.

`GrantConstructionError` is exactly:

```text
InvalidCommandId
InvalidApprovalId
InvalidSnapshotId
InvalidGrantVersion
InvalidEventSequence
ScopeConstructionFailed
CrossTenantScope
InstallationTerminal
PackageBindingMismatch
CapabilityNotDeclared
CapabilityMissing
CapabilityInactive
ScopeKindMismatch
ConfirmationPolicyTooPermissive
ForbiddenAdministrativeScope
EvidenceIncoherent
```

`GrantDecisionError` is exactly:

```text
AggregateMissing
AggregateAlreadyPresent
AuthorityConflict
ApprovalAlreadyConsumed
SnapshotIdMismatch
TerminalState
VersionMismatch
AdmissionEvidenceMismatch
ScopeChangeRequiresNewGrant
IllegalTransition
SequenceOverflow
```

`AuthorityConflict` and `ApprovalAlreadyConsumed` are repository-semantic `GrantDecisionError` receipts; pure single-aggregate `decide` never queries global indexes and never emits those two variants. The repository may synthesize and persist those typed rejected outcomes before invoking `decide`.

Decision precedence is split across the semantic repository and pure aggregate decision:

```text
global command-id exact replay/conflict
→ Issue/Replace consumed-approval uniqueness
→ Issue current-authority-tuple uniqueness
→ aggregate missing/already present as applicable
→ snapshot identity
→ terminal state
→ expected version
→ admission evidence / fresh approval / scope immutability
→ legal transition
→ sequence overflow
```

Errors expose stable categories only. `Debug`/`Display` MUST NOT disclose rejected IDs, approval IDs, tenant/user values or source payloads.

#### Events, aggregate and replay

`GrantEventKind` is exactly:

```rust
Issued
Replaced
MarkedStale
Expired
Revoked
```

`GrantEvent` is a private-payload envelope carrying:

```text
GrantEventSequence
post GrantVersion
GrantCommandId
GrantSnapshotId
private typed payload
```

Its read-only accessors are exactly `sequence`, `post_version`, `command_id`, `snapshot_id`, `kind`, `change_class -> Option<GrantChangeClass>` and `invalidation_reason -> Option<GrantInvalidationReason>`.

Issue carries every initial binding. Replace carries complete replacement authority, fresh approval and `GrantChangeClass`. MarkStale carries its exact reason. No event contains arbitrary prose or raw secret material.

`GrantAggregate` has private fields and read-only accessors for:

```text
GrantSnapshotId
TenantId
UserId
InstallationId
last admitted InstallationRevision
last admitted catalog revision, package ID/version/digest
CapabilityId
GrantScope
ConfirmationPolicy
capability-manifest digest
capability-registry revision
complete last admitted CapabilityDefinition binding and its digest
last GrantApprovalId
GrantState
GrantVersion
last GrantEventSequence
```

`GrantSnapshot` is a type alias to `GrantAggregate`.

Pure functions follow the same shape as installation:

```rust
pub fn decide(
    current: Option<&GrantAggregate>,
    command: &GrantCommand,
) -> Result<GrantEvent, GrantDecisionError>;

pub fn evolve(
    current: Option<GrantAggregate>,
    event: &GrantEvent,
) -> Result<GrantAggregate, GrantReplayError>;

pub fn replay<'a>(
    events: impl IntoIterator<Item = &'a GrantEvent>,
) -> Result<Option<GrantAggregate>, GrantReplayError>;
```

Replay errors MUST distinguish:

```text
InitialEventNotIssued
SequenceGap
SequenceDuplicate
SequenceOverflow
DuplicateCommandId
DuplicateApprovalId
PostTerminalEvent
IllegalTransition
VersionMismatch
SnapshotIdentityMismatch
AuthorityBindingMismatch
AdmissionEvidenceMismatch
```

Replay independently verifies version, identity, scope, approval, complete capability-definition bindings, computed policy-change class and evidence redundant fields. Every persisted event MUST be reachable through `decide`.

#### Resolver projection

`GrantAggregate::to_resolver_snapshot()` returns existing `CapabilityGrantSnapshot` with:

- stable aggregate `GrantSnapshotId`;
- current `GrantVersion`;
- exact tenant/user/installation/capability/scope/confirmation/manifest bindings;
- exact current `GrantState`.

It does not invoke the resolver and does not prove installation enablement, catalog validity, policy admission, source/execution admission or production readiness.

#### Semantic repository and idempotency

The semantic repository port is exactly:

```rust
pub trait GrantRepository {
    fn execute(
        &mut self,
        command: GrantCommand,
    ) -> Result<GrantCommandReceipt, GrantRepositoryError>;

    fn load_exact(
        &self,
        id: &GrantSnapshotId,
    ) -> Result<Option<GrantSnapshot>, GrantRepositoryError>;

    fn load_current_for_authority(
        &self,
        tenant_id: &TenantId,
        user_id: &UserId,
        installation_id: &InstallationId,
        capability_id: &CapabilityId,
        scope: &GrantScope,
    ) -> Result<Option<GrantSnapshot>, GrantRepositoryError>;

    fn event_history(
        &self,
        id: &GrantSnapshotId,
    ) -> Result<Vec<GrantEvent>, GrantRepositoryError>;
}
```

The in-memory fake keeps:

- aggregate/event streams by `GrantSnapshotId`;
- global `GrantCommandId -> {complete command, receipt}` ledger;
- global consumed `GrantApprovalId -> {GrantSnapshotId, evidence digest}` index for accepted Issue/Replace only;
- one current non-revoked grant per exact `(tenant,user,installation,capability,scope)` authority tuple;
- a narrowly named one-shot pre-commit failure injection.

Repository laws:

- identical command ID plus identical complete command returns the exact prior receipt with no append;
- same command ID plus different command returns `CommandConflict` before inspecting current state;
- accepted and domain-rejected commands both persist receipts;
- persistence failure occurs before aggregate/event/receipt/current-index/approval-index mutation and records nothing;
- a `GrantApprovalId` already consumed by any accepted Issue/Replace returns a persisted `GrantDecisionError::ApprovalAlreadyConsumed` receipt for a different command; rejected commands and persistence failures never consume approval;
- issuing a second non-revoked current grant for the exact authority tuple returns an `Ok(GrantCommandReceipt)` with persisted `GrantDecisionError::AuthorityConflict`; it records no grant event/aggregate/index mutation, and exact retry returns that same rejection even if the other grant is later revoked;
- stale and expired grants remain current denial-side authority; revoke releases the current tuple for a new grant identity while preserving old history;
- `load_current_for_authority` returns zero or one exact current grant and fails closed on corrupt multiplicity;
- no arbitrary insert/list/query API is admitted.

`GrantCommandOutcome` is exactly accepted `{event,snapshot}` or rejected `{error}`.

`GrantRepositoryError` is exactly:

```rust
CommandConflict
InjectedPersistenceFailure
CorruptEventHistory(GrantReplayError)
CorruptAuthorityIndex
DecisionRejected(GrantDecisionError)
```

#### B4 explicit non-claims

The accepted B4 contract is accompanied by bounded implementation evidence in `grant.rs`, `InMemoryGrantRepository`, `market::grant::tests` and the external grant test. That evidence does not enable installations, assemble `EnablePreconditionEvidence`, create invocation candidates, execute tools, write M30 intents, implement M90 persistence, or implement auto-grant issuance. Bounded B5 now proves one semantic carrier-by-carrier authority read/precondition, but it does not make B4 a production grant issuer; durable adapter provenance and production issuance/composition remain required.

## 5. Update and rollback

### M20-LC-009 — update and rollback pin exact targets

Stage, apply and rollback MUST operate on exact reviewed package revisions. Apply MUST use expected installation/update revisions and MUST preserve a tested rollback target. A failed apply MUST leave the prior accepted installation authority intact or the installation disabled; it MUST NOT widen permissions or silently fall back.

Bounded `M20-B6` now implements the pure event-sourced update aggregate, sealed installation package-pin Apply/Rollback events, complete-current grant stale-on-Apply/Rollback semantics, receipt-prefix rebuild and an atomic in-memory package-update semantic repository over staged cloned installation/grant owner fakes. This proves exact rollback/update semantics inside the bounded fake only: no production database transaction, durable crash recovery, real artifact-store switch, package-manager operation, production issuer, M10/M80 API/UI or B7 in-flight/current-call composition is implemented.

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

M20 lifecycle code MUST assemble exact `CatalogPackageRevision`, `PluginInstallationSnapshot`, `CapabilityGrantSnapshot` and `InvocationPolicySnapshot` inputs for the adopted resolver (items 7–8). It MUST NOT duplicate or bypass `InvocationResolver::resolve_projection` or `authorize_call`. Existing `crates/platform-core/src/invocation.rs` remains the authority for items 7–8. The factored `preflight_projected_call` helper owns only the existing `InvalidCall`/`ToolNotProjected`/`DispatchIdentityMismatch` prefix, returns no authorization and is called by `authorize_call` itself.

### M20-B5 transaction-current authority assembly

Bounded `crate::market::authority` owns one semantic read-transaction port and one assembly service. One `begin_read` freezes catalog, installation, current/exact grant and policy carriers under one opaque monotone revision. Transaction methods return those individual reviewed carriers only; they never return `InvocationAuthorityCandidate`, `CurrentDenyState`, an allow bit or a preauthorized bundle. `InvocationAuthorityService` alone constructs candidates/current state, calls the adopted resolver or recheck exactly once, and returns successful output only after `verify_precondition` succeeds.

Projection-time catalog/installation/current-grant absence stays `None` so the adopted resolver preserves its typed denial. Mandatory policy absence, call-time exact-grant absence, duplicate authority keys, incoherent current/exact indexes, revision overflow and post-success revision conflict are assembly/repository errors. An adopted denial remains primary and returns no output; a successful but stale decision becomes `TransactionConflict`. Call preflight completes before any repository read, so repository failures cannot mask the adopted envelope/name/dispatch precedence.

The deterministic `InMemoryInvocationAuthorityRepository` is a semantic fake. It clones separate typed carrier maps into one read transaction, advances fixture revision with checked arithmetic and can inject a one-shot verification conflict. It is not a production catalog/publication authority, durable M90 transaction, grant/enable issuer or effect-intent/I/O boundary. `market_authority_assembly.rs` is supporting evidence for planned `MARKET-003`/`MARKET-007`; neither row is promoted.

### M20-LC-014 — composition order remains external

Production call composition is owned by the composition root, not by M20. The authoritative ordering lives in [`agent-plugin-boundary.md`](agent-plugin-boundary.md) §7 and [`../plan/modules/50-tool-gateway-execution.md`](../plan/modules/50-tool-gateway-execution.md) §6. For illustration, the flow from M20's perspective is:

```text
M20 freezes projection/routes
→ provider raw call binds through frozen AgentToolsetView::bind_call
→ M30 persists ToolCallProposal::from(bound AgentToolCall)
→ composition invokes M40 prepare/correlation on the bound call
→ M20 transaction-current recheck
→ composition records M30 effect intent
→ M51/peer executor
→ M40 bounded outcome
→ composition records M30 effect receipt
→ M40 correlated result
→ M30 result state
```

M20 MUST NOT create M30 `ToolCallProposal`/`EffectIntent` and MUST NOT call an executor. Binding a provider raw call into an `AgentToolCall` is a composition/protocol step over the frozen `AgentToolsetView`; an unknown tool fails before any M30 proposal exists.

### M20-LC-015 — historical projections are immutable

Package update, disable or revoke MUST change only future projections and current call-time denial. It MUST NOT mutate an already frozen `ToolProjectionSnapshot`, an in-flight `RunSpec`, or a historical receipt.

### M20-B7-A1 — accepted application façade contract; implementation planned

A1 defines one M20-owned, framework-neutral façade over existing B1–B6 owner ports. Its exact operation set is:

```text
BrowseCatalog
ReadPackageDetail
ReadOwnedInstallation
ReadOwnedCurrentGrants
ReadOwnedPackageUpdate
DisableOwnedInstallation
```

The existing `InvocationAuthorityService<R>` remains the sole M20 projection/current-call application service. A1 adds no wrapper, alternate resolver, allow boolean or joined authority model. It selects no latest version, infers no owner, creates no installation, issues/replaces no grant, mints no evidence, applies no update and serializes no wire DTO.

Owner-scoped requests carry checked `TenantId`/`UserId` values as downstream scope claims, not authentication or authority evidence. A1 has zero production call sites. Until M00-B3/B5 and an exact M10 mapping contract exist, the operations are semantic-test-only. A later M10 adapter MUST derive tenant/user from a current M00-admitted request context; it MUST NOT copy them from a client body, query or header.

#### Exact Rust surface

The future module is exactly `crates/platform-core/src/market/application.rs`, exported as `pub mod application;`. Its complete public item set is:

```text
MarketApplicationConstructionError
MarketApplicationRepositoryError
MarketApplicationError
CatalogPageLimit
CatalogBrowseQuery
CatalogPackageQuery
OwnedInstallationQuery
OwnedInstallationGrantQuery
OwnedUpdateQuery
DisableInstallationRequest
MarketPackageSummary
MarketPackageDetail
MarketCatalogPage
MarketInstalledComponentView
MarketPackagePinView
MarketInstallationView
MarketGrantView
MarketGrantPage
MarketUpdateView
DisableInstallationReceiptView
CatalogReadRepository
InMemoryCatalogReadRepository
MarketApplicationService
```

No other public item kind is admitted. Exact constructors and request accessors are:

```rust
CatalogPageLimit::new(value: u16) -> Result<Self, MarketApplicationConstructionError>
CatalogPageLimit::get(&self) -> u16
CatalogBrowseQuery::new(revision: Option<CatalogRevision>, offset: u32, limit: CatalogPageLimit) -> Result<Self, MarketApplicationConstructionError>
CatalogPackageQuery::new(revision: Option<CatalogRevision>, package_id: PackageId, package_version: PackageVersion) -> Self
OwnedInstallationQuery::new(tenant_id: TenantId, user_id: UserId, installation_id: InstallationId) -> Self
OwnedInstallationGrantQuery::new(tenant_id: TenantId, user_id: UserId, installation_id: InstallationId, expected_installation_revision: InstallationRevision) -> Self
OwnedUpdateQuery::new(tenant_id: TenantId, user_id: UserId, update_id: PackageUpdateId) -> Self
DisableInstallationRequest::new(command_id: InstallationCommandId, tenant_id: TenantId, user_id: UserId, installation_id: InstallationId, expected_revision: InstallationRevision) -> Self
```

Their only remaining public query/request accessors are exactly:

```rust
CatalogBrowseQuery::catalog_revision(&self) -> Option<&CatalogRevision>
CatalogBrowseQuery::offset(&self) -> u32
CatalogBrowseQuery::limit(&self) -> CatalogPageLimit
CatalogPackageQuery::catalog_revision(&self) -> Option<&CatalogRevision>
CatalogPackageQuery::package_id(&self) -> &PackageId
CatalogPackageQuery::package_version(&self) -> &PackageVersion
OwnedInstallationQuery::tenant_id(&self) -> &TenantId
OwnedInstallationQuery::user_id(&self) -> &UserId
OwnedInstallationQuery::installation_id(&self) -> &InstallationId
OwnedInstallationGrantQuery::tenant_id(&self) -> &TenantId
OwnedInstallationGrantQuery::user_id(&self) -> &UserId
OwnedInstallationGrantQuery::installation_id(&self) -> &InstallationId
OwnedInstallationGrantQuery::expected_installation_revision(&self) -> &InstallationRevision
OwnedUpdateQuery::tenant_id(&self) -> &TenantId
OwnedUpdateQuery::user_id(&self) -> &UserId
OwnedUpdateQuery::update_id(&self) -> &PackageUpdateId
DisableInstallationRequest::command_id(&self) -> &InstallationCommandId
DisableInstallationRequest::tenant_id(&self) -> &TenantId
DisableInstallationRequest::user_id(&self) -> &UserId
DisableInstallationRequest::installation_id(&self) -> &InstallationId
DisableInstallationRequest::expected_revision(&self) -> &InstallationRevision
```

The exact view accessors are:

```rust
MarketPackageSummary::package_id(&self) -> &PackageId
MarketPackageSummary::package_version(&self) -> &PackageVersion
MarketPackageSummary::publisher(&self) -> &str
MarketPackageSummary::tier(&self) -> PackageTier
MarketPackageSummary::display_name(&self) -> &str
MarketPackageSummary::implementation_status(&self) -> ImplementationStatus
MarketPackageSummary::install_policy(&self) -> &InstallPolicy
MarketPackageDetail::catalog_revision(&self) -> &CatalogRevision
MarketPackageDetail::catalog_digest(&self) -> &Sha256Digest
MarketPackageDetail::summary(&self) -> &MarketPackageSummary
MarketPackageDetail::description(&self) -> Option<&str>
MarketPackageDetail::components(&self) -> &[ComponentDeclaration]
MarketPackageDetail::capabilities(&self) -> &[CapabilityId]
MarketPackageDetail::package_digest(&self) -> &Sha256Digest
MarketPackageDetail::component_declaration_set_digest(&self) -> &Sha256Digest
MarketPackageDetail::capability_manifest_digest(&self) -> &Sha256Digest
MarketPackageDetail::source_policy_digest(&self) -> &Sha256Digest
MarketCatalogPage::catalog_revision(&self) -> &CatalogRevision
MarketCatalogPage::catalog_digest(&self) -> &Sha256Digest
MarketCatalogPage::packages(&self) -> &[MarketPackageSummary]
MarketCatalogPage::next_offset(&self) -> Option<u32>
MarketInstalledComponentView::component_id(&self) -> &ComponentId
MarketInstalledComponentView::kind(&self) -> ComponentKind
MarketInstalledComponentView::version(&self) -> &ComponentVersion
MarketInstalledComponentView::digest(&self) -> &Sha256Digest
MarketPackagePinView::catalog_revision(&self) -> &CatalogRevision
MarketPackagePinView::package_id(&self) -> &PackageId
MarketPackagePinView::package_version(&self) -> &PackageVersion
MarketPackagePinView::package_digest(&self) -> &Sha256Digest
MarketPackagePinView::components(&self) -> &[MarketInstalledComponentView]
MarketPackagePinView::component_set_digest(&self) -> &Sha256Digest
MarketPackagePinView::capability_manifest_digest(&self) -> &Sha256Digest
MarketInstallationView::installation_id(&self) -> &InstallationId
MarketInstallationView::package_pin(&self) -> &MarketPackagePinView
MarketInstallationView::state(&self) -> ManagedInstallationState
MarketInstallationView::revision(&self) -> &InstallationRevision
MarketInstallationView::configuration_revision(&self) -> &ConfigurationRevision
MarketInstallationView::configuration_digest(&self) -> &Sha256Digest
MarketGrantView::snapshot_id(&self) -> &GrantSnapshotId
MarketGrantView::installation_id(&self) -> &InstallationId
MarketGrantView::installation_revision(&self) -> &InstallationRevision
MarketGrantView::catalog_revision(&self) -> &CatalogRevision
MarketGrantView::package_id(&self) -> &PackageId
MarketGrantView::package_version(&self) -> &PackageVersion
MarketGrantView::package_digest(&self) -> &Sha256Digest
MarketGrantView::capability_id(&self) -> &CapabilityId
MarketGrantView::capability_definition(&self) -> &CapabilityDefinition
MarketGrantView::scope(&self) -> &GrantScope
MarketGrantView::confirmation_policy(&self) -> &ConfirmationPolicy
MarketGrantView::state(&self) -> GrantState
MarketGrantView::version(&self) -> &GrantVersion
MarketGrantPage::installation_id(&self) -> &InstallationId
MarketGrantPage::observed_installation_revision(&self) -> &InstallationRevision
MarketGrantPage::grants(&self) -> &[MarketGrantView]
MarketUpdateView::update_id(&self) -> &PackageUpdateId
MarketUpdateView::installation_id(&self) -> &InstallationId
MarketUpdateView::rollback_pin(&self) -> &MarketPackagePinView
MarketUpdateView::target_pin(&self) -> &MarketPackagePinView
MarketUpdateView::change_class(&self) -> &UpdateChangeClass
MarketUpdateView::state(&self) -> UpdateState
MarketUpdateView::revision(&self) -> &UpdateRevision
MarketUpdateView::applied_installation_revision(&self) -> Option<&InstallationRevision>
DisableInstallationReceiptView::command_id(&self) -> &InstallationCommandId
DisableInstallationReceiptView::installation_id(&self) -> &InstallationId
DisableInstallationReceiptView::post_state(&self) -> ManagedInstallationState
DisableInstallationReceiptView::post_revision(&self) -> &InstallationRevision
```

Every accessor exposes the exact redacted DTO field type rather than an owner aggregate, event, evidence object, execution identity, private route or repository handle. Pages return slices of safe view DTOs; optional fields return `Option<&T>` or `Option<u32>`; states that are copy enums return by value; digests and semantic IDs return shared references. No accessor returns mutable references, iterators with side effects or raw source-policy/configuration maps. `MarketApplicationService` may construct these view values, but external callers cannot construct or mutate them.

The exact port/service signatures are:

```rust
pub trait CatalogReadRepository {
    fn load_current(
        &self,
    ) -> Result<Arc<CatalogReadModel>, MarketApplicationRepositoryError>;

    fn load_exact(
        &self,
        revision: &CatalogRevision,
    ) -> Result<Option<Arc<CatalogReadModel>>, MarketApplicationRepositoryError>;
}

impl InMemoryCatalogReadRepository {
    pub fn try_new(
        revisions: Vec<CatalogReadModel>,
        current_revision: CatalogRevision,
    ) -> Result<Self, MarketApplicationRepositoryError>;
}

impl<C, I, G, U> MarketApplicationService<C, I, G, U>
where
    C: CatalogReadRepository,
    I: InstallationRepository,
    G: GrantRepository,
    U: PackageUpdateRepository,
{
    pub fn new(catalogs: C, installations: I, grants: G, updates: U) -> Self;
    pub fn browse_catalog(&self, query: &CatalogBrowseQuery) -> Result<MarketCatalogPage, MarketApplicationError>;
    pub fn package_detail(&self, query: &CatalogPackageQuery) -> Result<MarketPackageDetail, MarketApplicationError>;
    pub fn installation(&self, query: &OwnedInstallationQuery) -> Result<MarketInstallationView, MarketApplicationError>;
    pub fn current_grants(&self, query: &OwnedInstallationGrantQuery) -> Result<MarketGrantPage, MarketApplicationError>;
    pub fn package_update(&self, query: &OwnedUpdateQuery) -> Result<MarketUpdateView, MarketApplicationError>;
    pub fn disable_installation(&mut self, request: DisableInstallationRequest) -> Result<DisableInstallationReceiptView, MarketApplicationError>;
}
```

`Arc` is `std::sync::Arc`. `InMemoryCatalogReadRepository::try_new` is its only public inherent method; it accepts `1..=64` exact immutable catalog revisions, rejects duplicate/current-missing histories and exposes no mutator. `CatalogPageLimit` accepts only `1..=100`. Offset `0` may select current; any nonzero continuation offset requires an exact `CatalogRevision`. Paging preserves the existing canonical `(package_id, package_version)` order, returns the selected revision/digest and never continues across revisions. Package detail requires exact package ID/version and has no fuzzy/latest/same-name fallback.

#### Visibility, traits and safe projection

All fields are private. Request/query constructors are checked and read-only; view constructors are private to `application.rs`. The three fieldless error enums derive exactly `Debug, Clone, Copy, PartialEq, Eq`; `CatalogPageLimit` derives exactly `Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash` plus manual bounded `Debug`; query/request/view types derive exactly `Clone, PartialEq, Eq` plus manual bounded/redacted `Debug`; `InMemoryCatalogReadRepository` derives only `Clone`; `MarketApplicationService` derives nothing. The fake and service use manual authority-redacted `Debug` without inner `Debug` bounds.

No A1 type implements `Default`, `Deref`, `DerefMut`, Serde, unchecked/raw conversion or mutable-field access. The service exposes no `into_parts`, repository mutator, raw event/history API or authority-evidence constructor.

Safe views expose only:

- package summaries/details: reviewed typed metadata and declaration/digest fields; raw source-policy map, absent source/license fields and private execution details remain excluded;
- installation/update pins: catalog/package/component IDs, kinds, versions and digests through `MarketPackagePinView`; every `ExecutionIdentity` is excluded;
- installation state/revision and configuration revision/digest; configuration entries, `NonSecretText`, `SecretRef` and secret-ref IDs are excluded;
- current grant identity/binding/state/version; approval IDs/evidence, consumed-approval indexes and history are excluded;
- update state/pins/change/revisions; readiness/confirmation/rollback evidence, policies, private routes and history are excluded.

`MarketGrantPage` is the complete canonically sorted current nonterminal set for one exact installation/revision. Every nested installation ID must agree; revoked history is absent. `DisableInstallationReceiptView` is a historical command disposition, not a current-state claim.

Exact fieldless errors are:

```text
MarketApplicationConstructionError
  PageLimitOutOfRange
  UnboundContinuationOffset

MarketApplicationRepositoryError
  Unavailable
  EmptyCatalogHistory
  TooManyCatalogRevisions
  DuplicateCatalogRevision
  CurrentCatalogMissing
  CorruptCatalog

MarketApplicationError
  NotFound
  Conflict
  LifecycleDenied
  RepositoryUnavailable
  CorruptAuthority
```

Absent and foreign-owned values both map to `NotFound`. Revision/command-ledger conflicts map to `Conflict`; legal owner-state denials to `LifecycleDenied`; I/O/persistence unavailability to `RepositoryUnavailable`; corrupt history/index/replay or unclassified owner-error drift to `CorruptAuthority`. Errors carry no arbitrary text, input or wrapped source error.

#### Disable and fake semantics

`DisableOwnedInstallation` loads the exact installation, maps absence/owner mismatch to `NotFound`, constructs exactly one existing `InstallationCommand::disable` with the request's command ID and expected revision, then delegates to `InstallationRepository`. The façade MUST NOT reject the expected revision before the owner repository's command ledger can replay an exact prior receipt. A typed-equal retry returns the prior receipt and emits no second event; conflicting command reuse or stale new command is `Conflict`. Immutable ownership plus the owner repository's expected-revision decision closes the load/execute race for a new command.

A1 reuses `InMemoryInstallationRepository`, `InMemoryGrantRepository` and `InMemoryPackageUpdateRepository`; it creates no second aggregate implementation. Independent safe reads identify their owner revisions and do not claim cross-repository atomicity. Invocation projection/recheck remains solely `InvocationAuthorityService` under one verified transaction.

The exact future integration-test functions are:

```text
anonymous_catalog_paging_is_revision_bound_bounded_and_exact
package_detail_is_exact_without_latest_or_fallback
owned_reads_hide_foreign_objects_and_exclude_sensitive_carriers
current_grants_require_exact_installation_revision_and_canonical_order
disable_preserves_owner_ledger_first_idempotency_and_maps_one_event
application_facade_exposes_no_transport_or_authority_issuer_surface
```

They use production-public API only, assert exact typed variants and admit no test-only public constructor. A1 still proves no process restart, database transaction, migration, event delivery, real API route or production caller.

### M20-B7-B — accepted composition-evidence contract; implementation planned

B7-B adds no production M40 crate or application code. It is a composition-root integration-test contract that stages existing public M20/M30/protocol carriers and a semantic fake executor/journal. [`agent-plugin-boundary.md`](agent-plugin-boundary.md) §7 owns its exact support inventory, persistence/execution/result order, reconciliation and denial matrix. This contract owns only the M20 side:

- `InvocationAuthorityService` performs the one transaction-current recheck before intent/executor I/O;
- deny-side current drift returns no prepared authority and reaches no intent, executor, receipt or result;
- disable/update/revoke change current/future authority while a frozen historical view remains immutable;
- the B3/B5/B6 evidence bridge is compositional and MUST NOT expose owner-private evidence constructors or claim one external test executed them;
- B7-B remains test-only semantic evidence and proves no durability, production executor or acceptance pass by contract alone.

## 9. Non-goals and current status

This contract does not own:

- anonymous browse/detail delivery through M10/M80 application/query adapters remains planned (`MARKET-001`); the `M20-B1` (historical `B1-1`) anonymous metadata domain read model is implemented but is not delivery evidence;
- durable installation/grant/enable/disable/update mutation and production composition remain planned (`MARKET-002`/`MARKET-003`/`MARKET-004`); accepted A1 covers only internal safe reads plus Disable with zero production call sites, and accepted B7-B is test-only; neither creates durable state, production issuer authority, artifact switching or acceptance evidence;
- a production database/repository transaction, durable update repository, crash-recovery proof or TOCTOU closure (planned);
- provider, network, MCP, daemon HTTP/SSE or UI adapters;
- external tool execution, durable journal or crash recovery;
- M30 `EffectIntent`, M40 executor dispatch, or M51 process isolation.

Current repository status: the pure P0a resolver/recheck is implemented and adopted (`MARKET-005`/`MARKET-006`); B1–B4 provide bounded catalog/capability/installation/grant evidence; bounded B5 adds carrier-by-carrier semantic authority reads; bounded B6 adds pure update/rollback aggregate and atomic in-memory semantic-repository evidence. The exact A1 and B7-B contracts above are accepted, but `application.rs`, `market_application.rs` and `tool_effect_composition.rs` do not exist and no production caller is admitted. No production database, durable grant/update/rollback repository, crash recovery, artifact switch, production grant/enable/update issuer, effect-intent coupling, current-call/in-flight composition implementation or M10/M80 browse/API/UI delivery exists yet. M20 remains `partial-evidence`; `MARKET-001`–`MARKET-004`, `MARKET-007`, `PKG-019`, `PKG-020` and `FP-007` remain `planned`, and no current first-party manifest is made runnable by contract acceptance.
