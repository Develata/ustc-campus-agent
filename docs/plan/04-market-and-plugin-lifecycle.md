# Market and Plugin lifecycle

## Metadata

- `Layer`: Product authority
- `Status`: Schema/identity baseline implemented; runtime lifecycle planned
- `Version`: `0.2.0`
- `Last Review`: `2026-07-22`
- `Authority Owns`: catalog boundary, package ontology, install/enable/grant/invoke/update lifecycle
- `Authority Defers To`: Market JSON schema/registries and package/permission contracts for exact fields
- `Counterpart Feature`: `docs/features/00-market-browse-install.md`
- `Counterpart Contracts`: `docs/contracts/plugin-package.md`, `docs/contracts/permissions.md`
- `Counterpart Acceptance`: `MARKET-*`, `FP-006`, `FP-015`, `FP-007`
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

Before an outbound call or native component dispatch, the server-side resolver MUST bind:

```text
PluginInstallation
→ exact PluginPackage version
→ exact component declaration/version/path or endpoint identity
→ effective execution identity
→ capability manifest hash
→ current grant version and tenant/object scope
→ current enabled/revoked/emergency-block state
```

Any missing package, digest/path mismatch, stale grant, disabled installation, revoked capability or unknown execution identity fails closed.

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

- durable installations/grants;
- enable/disable resolver;
- Market browse/detail UI;
- upgrade/revoke/rollback runtime.

Verification:

- `docs/contracts/plugin-package.md`
- `market/schemas/plugin-package.schema.json`
- `python3 scripts/check_repo_contracts.py`
- `scripts/tests/test_check_repo_contracts.py`
- `MARKET-*`, `FP-006`, `FP-015`, `FP-007`
