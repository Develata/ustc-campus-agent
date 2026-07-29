# Market lifecycle contract

## Metadata

- `Status`: Accepted `M20-B1` lifecycle contract; durable installation/grant/update lifecycle implementation planned; pure invocation resolver and call-time recheck adopted as items 7–8
- `Version`: `market-lifecycle/v0`
- `Last Review`: `2026-07-29`
- `Owning Plan`: [`../plan/04-market-and-plugin-lifecycle.md`](../plan/04-market-and-plugin-lifecycle.md)
- `Large-module Blueprint`: [`../plan/modules/30-market-package-lifecycle.md`](../plan/modules/30-market-package-lifecycle.md)
- `Counterpart Contracts`: [`plugin-package.md`](plugin-package.md), [`invocation-resolution.md`](invocation-resolution.md), [`permissions.md`](permissions.md), [`agent-plugin-boundary.md`](agent-plugin-boundary.md)
- `Authority Defers To`: [`../plan/03-platform-authority.md`](../plan/03-platform-authority.md) for state ownership, [`agent-runtime.md`](agent-runtime.md) for run/effect state, and [`invocation-resolution.md`](invocation-resolution.md) for the adopted projection/recheck decision shapes
- `Acceptance`: implemented `MARKET-005`, `MARKET-006`; planned `MARKET-001`, `MARKET-002`, `MARKET-003`, `MARKET-004`, `MARKET-007`, `PKG-019`, `PKG-020`, `FP-007` (see [`../acceptance/matrix.tsv`](../acceptance/matrix.tsv))
- `Primary Code`: planned cohesive modules under `crates/platform-core/src/market/` (not yet created); current adopted resolver/recheck authority in `crates/platform-core/src/invocation.rs` (items 7–8 only)

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

- catalog query projection or anonymous browse/detail (planned, `MARKET-001`);
- durable installation/grant/enable/disable/upgrade mutation (planned, `MARKET-002`/`MARKET-003`/`MARKET-004`);
- a production database/repository transaction or TOCTOU closure (planned);
- provider, network, MCP, daemon HTTP/SSE or UI adapters;
- external tool execution, durable journal or crash recovery;
- M30 `EffectIntent`, M40 executor dispatch, or M51 process isolation.

Current repository status: the pure P0a invocation resolver and call-time recheck with typed in-memory snapshots and synthetic fixtures are implemented and adopted (`MARKET-005`/`MARKET-006`). This B1-0 slice projects the lifecycle contract only; no Rust lifecycle implementation, durable repository, installation aggregate, grant aggregate, update/rollback, or application composition is created by this slice. No current first-party manifest is made runnable by projecting this contract. Future implementation slices and their intended bindings are listed in [`../acceptance/matrix.tsv`](../acceptance/matrix.tsv) and remain `planned` until their exact evidence exists.
