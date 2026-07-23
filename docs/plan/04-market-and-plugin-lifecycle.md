# Market and Plugin lifecycle

## Metadata

- `Layer`: Product authority
- `Status`: Schema/identity baseline implemented; P0a resolver contract and runtime lifecycle planned
- `Version`: `0.3.1`
- `Last Review`: `2026-07-23`
- `Authority Owns`: catalog boundary, package ontology, install/enable/grant/invoke/update lifecycle
- `Authority Defers To`: Market JSON schema/registries and package/permission contracts for exact fields
- `Counterpart Feature`: `docs/features/00-market-browse-install.md`
- `Counterpart Contracts`: `docs/contracts/plugin-package.md`, `docs/contracts/permissions.md`, `docs/contracts/invocation-resolution.md`
- `Counterpart Acceptance`: `MARKET-*`, `AGENT-002`, `FP-006`, `FP-015`, `FP-007`
- `Primary Code Areas`: `market/`, `crates/platform-core/`, future installation/gateway modules

## 1. Scope and repository boundary

`market/` is a logical Catalog Authority boundary from day one. It remains inside this monorepo for the competition MVP because a physical split would add cross-repository versioning, review, CI and release coordination before independent package lifecycles are proven.

A future `ustc-campus-agent-market` repository is justified only when external contribution, independent maintainership, public/private separation, release cadence or signing/rollback requires a separate repository identity.

## 2. Package ontology

`PluginPackage` is the only unit users inspect, install, enable, disable and upgrade. Current manifest declarations include:

- stable package ID and SemVer;
- publisher and review tier;
- display name and honest `implementationStatus`;
- `installPolicy`;
- exact components;
- stable capability IDs;
- source/data policy.

Current component kinds are:

- `SkillComponent`;
- `DeclarativeResourcePack`;
- `McpServerComponent`;
- `NativeRustComponent`.

Components are package-owned declarations, not a second installation lifecycle. A future new component kind requires schema, admission, permission and rollback analysis before it appears in a manifest.

`planned` packages MUST declare no executable components. `development` states that implementation work exists but does not prove install, grant, discovery or invocation. `implemented` requires at least one valid component and its corresponding lifecycle evidence.

## 3. Catalog and runtime state separation

Catalog declarations answer what an approved package revision contains. Runtime state answers what one user/tenant has installed and may invoke.

```text
Catalog Authority                    Runtime Authority
-----------------                    -----------------
package version                       installation ID
component declarations                exact installed version/components
capability request                     effective grants and grant version
install policy                         enabled/disabled/revoked state
source policy                          tenant/user configuration + secret refs
review tier                            invocation and audit receipts
```

A manifest's default-install policy is not proof that runtime installation state exists.

## 4. Lifecycle

```text
Submit
→ Validate
→ Review
→ Publish
→ Browse/Inspect
→ Install exact version
→ Configure non-secret values and secret references
→ Resolve grants
→ Enable
→ Discover
→ Invoke through gateway
→ Update | Disable | Revoke | Roll back
```

These states MUST remain distinct:

- **Publish** approves a catalog revision.
- **Install** creates exact user/tenant runtime state.
- **Configure** records typed values and secret references, never raw secrets in manifests.
- **Enable** permits discovery; it does not bypass invocation checks.
- **Invoke** re-resolves current installation, version, component identity and grants every time.
- **Disable/Revoke** immediately removes discovery and blocks new invocation.

## 5. Effective invocation resolution

Invocation authority is established before catalog browsing or durable lifecycle persistence is implemented. The first slice is a pure, deterministic server-side resolver over exact caller-supplied authority snapshots; storage and adapters may later supply those snapshots but cannot replace the decision.

Before an outbound call or native component dispatch, the resolver MUST bind:

```text
PluginInstallation
→ exact PluginPackage version
→ exact component declaration/version/path or endpoint identity
→ effective execution identity
→ capability manifest hash
→ current grant version and tenant/object scope
→ current enabled/revoked/emergency-block state
```

It MUST additionally bind tenant/user scope, source-policy identity, exact canonical input-schema digest and one collision-free dispatch identity. The resulting immutable tool projection controls both schemas exposed to the model and dispatch entries accepted for the turn. Session activation may narrow that projection, never install, grant, re-enable or bypass revoke.

At call time, exact arguments, scope, grant/revoke state and emergency block are checked again before effect intent or outbound I/O. Any missing package, digest/path mismatch, stale grant, disabled installation, revoked capability, schema mismatch, name collision, unknown execution identity or authority conflict fails closed. No error selects a same-name tool, alternate package/component/provider/runtime or previous successful snapshot. P0a defines and unit-tests the pure decision only; `MARKET-007` remains planned until application composition proves that denial prevents both effect-intent creation and adapter I/O.

[`docs/contracts/invocation-resolution.md`](../contracts/invocation-resolution.md) owns the P0a input/output shapes, deterministic digest rules, error taxonomy, fixture matrix, framework borrow/adapt/reject evidence and the boundary into `RunSpec`/`AgentRun::new`.

## 6. Capability policy

The capability registry—not the package author—owns risk and auto-grant eligibility.

- Default first-party manifests MAY request only registry-approved auto-grant-eligible public read/link-out capabilities.
- Tenant-private capabilities require explicit narrow scope and grant review before use.
- Cross-user, administrative, write, destructive and unknown capabilities are never default auto-granted.
- A new capability or risk/data-class change is permission expansion.
- Model output cannot create or widen a grant.

## 7. Update and rollback

An installation pins the package version and effective component/capability identity. Updates follow:

- exact reviewed patch with unchanged permission set: MAY enter staged/canary rollout after evidence;
- permission expansion, trust downgrade, execution-identity change or minor/major semantic change: requires explicit reapproval;
- unhealthy rollout: retain a tested rollback target and block further cohort promotion;
- security revoke: block new invocation immediately and drain/stop old execution according to explicit policy;
- first-party status never bypasses update, revoke or audit rules.

## 8. Failure and recovery

- Import/schema failure leaves the prior catalog projection active.
- Partial default bootstrap creates no implied installation; retry from explicit state.
- Disable/revoke propagation failure blocks invocation rather than serving stale permission state.
- Catalog projection loss is recovered deterministically from a pinned Git revision.
- Unknown component/capability or orphaned declaration fails validation.
- User configuration secret material is never copied into catalog Git, logs or audit payloads.

## 9. Current state and verification

Implemented:

- schema and exactly three default package manifests;
- exact first-party IDs, versions, statuses, capabilities and install policies;
- safe component path validation and Rust/catalog identity cross-check;
- allowance for safe user-installed non-first-party packages.

Planned:

- deterministic typed invocation resolver and immutable tool-projection snapshot (P0a);
- read-only catalog browse/detail projection (P0b);
- durable installations/grants;
- enable/disable resolver;
- Market browse/detail UI;
- upgrade/revoke/rollback runtime.

Verification:

- `docs/contracts/plugin-package.md`
- `docs/contracts/invocation-resolution.md`
- `market/schemas/plugin-package.schema.json`
- `python3 scripts/check_repo_contracts.py`
- `scripts/tests/test_check_repo_contracts.py`
- `MARKET-*`, `AGENT-002`, `FP-006`, `FP-015`, `FP-007`
