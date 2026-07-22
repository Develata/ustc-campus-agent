# PluginPackage contract

PluginPackage is the unit users inspect, install, authorize, disable, and upgrade.

Required fields are defined by [`market/schemas/plugin-package.schema.json`](../../market/schemas/plugin-package.schema.json).

## Default first-party packages

- [`ustc.affairs-navigator`](../../market/packages/ustc.affairs-navigator/package.json)
- [`ustc.change-radar`](../../market/packages/ustc.change-radar/package.json)
- [`ustc.opportunity-graph`](../../market/packages/ustc.opportunity-graph/package.json)

All three use `FirstPartySystemPlugin` install policy, are default-installed/default-enabled, and remain independently disableable. `implementationStatus` is authoritative for repository claims: a `planned` package must have no executable component declaration; `development` does not imply install/grant/runtime completion.

Default first-party manifests currently declare only exact auto-grant-eligible public read/link-out capabilities. Consent-aware tenant-private capabilities enter later through explicit grant and permission-diff contracts; their existence in the registry does not auto-grant them.

## Lifecycle

```text
Browse
→ Inspect publisher/version/components/capabilities/source policy
→ Install exact package version
→ Resolve grants
→ Enable
→ Agent discovers tools
→ Invoke through gateway
→ Disable/revoke
→ Invocation denied
→ Re-enable/upgrade/rollback under explicit policy
```

Installation/grants are user runtime state and must not be encoded into market manifests.

The manifest's `installPolicy` is catalog policy, not proof that runtime installation state exists. Actual pinned versions, grants, enabled state, and receipts remain runtime authority.
