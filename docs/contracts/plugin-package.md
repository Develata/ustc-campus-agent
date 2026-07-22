# PluginPackage contract

PluginPackage is the unit users inspect, install, authorize, disable, and upgrade.

Required fields are defined by [`market/schemas/plugin-package.schema.json`](../../market/schemas/plugin-package.schema.json).

## First-party package

[`market/packages/ustc.opportunity-graph/package.json`](../../market/packages/ustc.opportunity-graph/package.json)

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
