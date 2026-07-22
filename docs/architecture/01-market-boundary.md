# Market boundary

## Decision

`market/` is a logical authority boundary from day one. It is not a separate GitHub repository during the competition MVP.

## Why not split now

Splitting immediately would add cross-repo versioning, PR synchronization, permissions, CI, and release coordination before the exact three default first-party manifests and shared PluginPackage contract are proven through independent lifecycle journeys.

## Future split conditions

Create `ustc-campus-agent-market` only if at least one condition holds:

- platform remains private but catalog should be public;
- external contributors submit packages;
- catalog has independent maintainers;
- catalog and platform release cadence diverge;
- signing/rollback/deployment requires independent repository identity.

## Invariants before and after split

- package manifests are declarative;
- packages pin exact id/version/components/capabilities;
- grants/installations are tenant runtime state, not catalog truth;
- schema validation is fail-closed;
- secret-bearing manifests are rejected.
