# M20 — Market and Package Lifecycle

## Metadata

- `Module ID`: `M20`
- `Status`: Accepted blueprint; manifest/package-catalog, B2 capability, B3 installation, B4 grant, bounded `M20-B5` semantic authority-read transaction/assembly including static-application authorization, bounded `M20-B6` update/rollback domain plus semantic fake and pure invocation-resolver evidence exist; durable installation/grant/update repositories, artifact switching and production composition remain planned
- `Implementation State`: `partial-evidence`
- `Version`: `m20-market-package/v0`
- `Last Review`: `2026-09-01`
- `Owning Plan`: [`../04-market-and-plugin-lifecycle.md`](../04-market-and-plugin-lifecycle.md)
- `Owning Lifecycle Contract`: [`../../contracts/market-lifecycle.md`](../../contracts/market-lifecycle.md)
- `Primary code areas`: `market/`, cohesive Market modules under `crates/platform-core/` until extraction is justified

## 1. Purpose

`M20` owns what a `PluginPackage` revision declares and what an exact tenant/user installation is allowed to discover or invoke. It covers catalog validation, package/component identity, install/configure/enable/disable/update/revoke, capability grants and pure invocation authority snapshots.

## 2. Non-goals

- running the Agent loop or changing its phases;
- executing Plugin code or owning MCP/provider sessions;
- rendering Market UI;
- treating package publication, installation, enablement and grant as the same state;
- allowing manifests or models to create/widen grants;
- making catalog availability proof of runtime readiness.

## 3. Owned objects and state

```text
PublisherIdentity
PluginPackageRevision / PackageDigest
ComponentDeclaration / ExecutionIdentity
CapabilityDefinition / CapabilityRequest
CatalogRevision and publication state
PluginInstallation and installed revision
Typed configuration + SecretRef bindings
CapabilityGrant and grant version/scope
Enable/Disable/Revoke/Update/Rollback state
InvocationAuthoritySnapshot
```

Reviewed Git files under `market/` own declared package revisions. Durable operational state owns per-tenant installations and grants. A database projection cannot edit either source into a new truth.

## 4. Public inputs and outputs

Commands:

```text
Validate/Import/Publish package revision
Install exact revision
Configure typed values/secret references
Grant/Revoke capability
Enable/Disable/Revoke installation
Stage/Apply/Rollback update
```

Queries/events:

```text
BrowseCatalog / PackageDetail
InstallationSnapshot / GrantSnapshot
ResolveToolProjection
RecheckInvocationAuthority
AuthorizeStaticApplicationUseCase
MarketEvent / MarketError
```

The resolver emits immutable authority entries and gateway-private routes. Agent-facing protocol values are produced through `M40`, not by exposing Market internals. `AuthorizeStaticApplicationUseCase` is a separate transaction-current check for an exact registered application operation backed by a declarative resource component with no tool or admitted execution identity; it emits no Agent-facing projection or executor route.

## 5. Dependency direction

Allowed dependencies:

- stable protocol/value crates;
- `M00` tenant/user/request context;
- `M90` catalog projection, installation/grant repository, transaction, secret-ref and event ports.

Allowed callers:

- `M10` application API;
- `M40` for projection/current invocation authority;
- composition invoking an owning static `M72` application use case through `B-M20-M72-AUTH`;
- composition/bootstrap commands.

Forbidden dependencies:

- Agent runtime/harness internals;
- Plugin implementation or executor SDK;
- Dioxus/client types;
- MCP/provider transport handles;
- concrete database types in domain contracts.

## 6. Lifecycle

```text
Submit → Validate → Review → Publish
→ Browse/Inspect
→ Install exact revision
→ Configure references
→ Resolve grants
→ Enable
→ Discover/resolve/recheck
→ Update | Disable | Revoke | Rollback | Uninstall
```

Publication never creates an installation. Installation never implies a grant. Enablement permits discovery but every call still rechecks current deny-side state.

## 7. Failure and recovery

- Invalid/secret-bearing manifest: reject import; prior catalog revision remains active.
- Unknown component/capability or unsafe path: reject before publication.
- Partial install/bootstrap: no implied installed state; resume from explicit journal state.
- Grant/version/scope mismatch: no projection or invocation.
- Disable/revoke propagation uncertainty: deny new discovery/calls.
- Failed bounded update transaction: retain the prior accepted installation pin and grant states inside the atomic semantic fake; future durable adapters must prove the same property with crash recovery and artifact-store rollback.
- Projection cache loss: rebuild from reviewed catalog plus durable installation/grant state.
- Resolution/store transaction race: close with a repository transaction/precondition or fail closed.

## 8. Configuration and secrets

Package manifests declare typed non-secret configuration schemas and requested capabilities. Runtime installation stores values and `SecretRef`s under tenant scope. Raw secrets never enter Market Git, model-visible definitions, normal logs or receipts.

The capability registry, not package authors, owns risk class and auto-grant eligibility.

## 9. Observability

Events record package/version/digest, installation revision, grant version/scope, lifecycle transition, actor/request IDs and redacted denial class. Browse metrics remain separate from install/invoke readiness. Audit must show which exact package/component/grant produced each accepted projection.

## 10. Extension and replacement

Catalog storage, operational repository and artifact store are replaceable through ports. Component kinds require explicit admission contracts; a new kind is not added through a generic string. A future separate Market repository may replace the monorepo catalog only after an ADR covers review, signing, compatibility and rollback.

## 11. Performance path

Read-heavy browse projection and per-turn/call authorization are separate paths. Invocation checks use exact indexed IDs and bounded sets; they never scan arbitrary catalog history. Cache acceleration is allowed only when revoke/disable correctness remains immediate and the durable state can rebuild it.

## 12. Scope boundary

**MVP**

- exactly reviewed package manifests and read-only browse/detail;
- durable exact installation, enable/disable/revoke and grants;
- pure deterministic tool projection and current call recheck;
- first-party default bootstrap without inferred state;
- permission expansion requires explicit reapproval.

**Later**

- publisher/community tiers and submissions;
- signed artifacts and staged cohorts;
- physical Market repository split;
- broader component types after isolation proof.

**Explicit non-goals**

- arbitrary hosted code execution from catalog metadata;
- “trust all future tools” or silent grant expansion;
- Agent/Plugin direct registration bypass;
- same-name fallback across packages/components.

## 13. Small-module decomposition

1. `package-schema` — typed manifest and deterministic validation.
2. `catalog-domain` — publication/revision state.
3. `capability-registry` — risk/data/auto-grant policy.
4. `installation-domain` — install/configure/enable/disable/revoke/uninstall.
5. `grant-domain` — exact scope/version/reapproval.
6. `catalog-read-model` — anonymous browse/detail projection.
7. `invocation-resolver` — immutable per-turn projection.
8. `invocation-recheck` — current deny-side/arguments plus the narrower no-tool static-application authorization.
9. `package-update` — stage/apply/rollback.
10. `market-ports` — repositories/artifact/secret-ref/event fakes.

Existing `invocation.rs` is reviewed against items 7–8; it is not permission to collapse the other items into that file.

### Delivery sequence

The canonical roadmap batch schedule lives in [`../../tasks/01-execution-roadmap.md`](../../tasks/01-execution-roadmap.md) §7 (`M20-B0` through `M20-B7`). B1–B4 provide bounded catalog/capability/installation/grant evidence. Bounded `M20-B5` provides one GAT read transaction across separate catalog/installation/current-or-exact-grant/policy carriers, service-owned candidate/current assembly, the adopted resolver/recheck, a no-tool static-application authorization and post-success precondition verification under `crate::market::authority`. The static check consumes typed per-request confirmation evidence and denies an `Ask` grant unless that evidence is `Confirmed`; missing or unconfirmed intent reaches no owning application dispatch. The check is consumed by the fixture-backed M72 composition without claiming a durable production M20 adapter or generic M20-B7 completion. Bounded `M20-B6` now provides pure update/rollback aggregate evidence and an atomic in-memory semantic package-update repository under `crate::market::update`, including exact approval/readiness/confirmation/rollback evidence, complete-current grant stale-on-Apply/Rollback semantics, receipt-prefix rebuild and cross-stream reference bijection tests. These slices create no production grant/update issuer, durable lifecycle state, artifact-store switch, crash recovery, enable evidence, effect intent or acceptance promotion. Remaining work is `M20-B7` production-facing API/composition/fake M40 consumer plus future durable adapters. The module remains `partial-evidence`; B6 effects no `StandaloneReady`, `IntegrationReady`, `Integrated` or `Accepted` promotion.

## 14. Exit gate

`M20` is standalone-ready when exact package, install, grant, disable/revoke, update/rollback and resolver tests pass against fake repositories. It is integration-ready when `M40` receives a frozen tool view and denial reaches no fake executor. It is accepted when `M10`/`M80` prove browse/install/disable behavior and the bound `MARKET-*`, `PKG-*` and `FP-*` rows pass.
