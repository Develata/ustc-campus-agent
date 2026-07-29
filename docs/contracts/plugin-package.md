# PluginPackage contract

PluginPackage is the unit users inspect, install, authorize, disable, and upgrade.

Required fields are defined by [`market/schemas/plugin-package.schema.json`](../../market/schemas/plugin-package.schema.json). The bounded typed Rust ingress, semantic coherence rules and canonical declaration digests are frozen by [`market-lifecycle/v0`](market-lifecycle.md) and implemented in `crates/platform-core/src/market.rs`; successful decoding is not publication or runtime authority.

Runtime compilation and Agent isolation are defined by [`agent-plugin-boundary/v0`](agent-plugin-boundary.md). Package installation never means direct linkage into the Agent kernel.

The package lifecycle — publication, installation, grant, enable/disable/revoke, update and rollback invariants — is owned by [`market-lifecycle/v0`](market-lifecycle.md).

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
→ Resolver/gateway compiles namespaced contributions
→ Agent discovers Plugin-neutral tools
→ Invoke through gateway and bounded executor
→ Disable/revoke
→ Invocation denied
→ Re-enable/upgrade/rollback under explicit policy
```

Installation/grants are user runtime state and must not be encoded into market manifests.

The manifest's `installPolicy` is catalog policy, not proof that runtime installation state exists. Actual pinned versions, grants, enabled state, and receipts remain runtime authority.

## Agent integration boundary

- `SkillComponent` and `DeclarativeResourcePack` contribute bounded context/resources only.
- `McpServerComponent` and admitted `NativeRustComponent` may contribute tool definitions and private executor routes only after binding/schema/capability review.
- Agent code sees versioned tool definitions/calls/results, not manifests, component kinds, endpoints or implementation handles.
- `NativeRustComponent` does not permit dynamic linkage into `agent-runtime`; its runnable artifact/profile remains separately versioned and replaceable.
- Package update/disable/revoke creates or removes future projections without mutating in-flight runs or historical receipts.
