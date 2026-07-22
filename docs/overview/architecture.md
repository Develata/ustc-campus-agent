# Architecture overview

This document is a cross-layer map. Owning semantics live in `docs/plan/` and `docs/contracts/`; this overview does not redefine them.

## Product and authority

```text
                                      ┌─ USTC Affairs Navigator
reviewed sources                      ├─ USTC ChangeRadar
→ Campus Trust Kernel                 └─ Campus Opportunity Graph
         │                                  │
         └─ source/revision/time/           └─ independent PluginPackage
            conflict/provenance                 install/enable lifecycle
```

All three products project the same trusted campus facts while retaining independent package identity and acceptance.

## Runtime topology

```text
Web/PWA and future clients
    │ typed HTTP/event API
    ▼
ustc-agentd authority plane
├── identity/session                    (planned)
├── Market catalog projection           (planned; Git manifests exist)
├── installation/grant resolver         (planned)
├── platform Agent run + tool gateway   (R0 transition kernel implemented; orchestration planned)
├── Campus Trust Kernel                 (contract; planner subset exists)
├── first-party product use cases       (mostly planned)
└── audit/evidence                      (planned)
    │
    ├── reviewed Git declarations
    ├── durable operational store       (future)
    ├── immutable evidence store        (future)
    └── model/tool/source adapters       (replaceable)
```

The Rust domain core owns legal transitions. Clients, model frameworks, databases, caches and adapters remain projections or infrastructure.

## Current executable slice

Implemented today:

- Rust workspace and daemon/CLI skeleton;
- exact three first-party identities and manifest contract;
- deterministic repository/manifest checks;
- offline Course Planning fixture validation and planner;
- framework-neutral Agent run-spec, transition, replay, effect-ordering and budget kernel.

The next platform slice is the minimal Market read/invocation resolver consumed by the R0 kernel; the next first-party product slice remains ChangeRadar source/revision/diff. Additional Course Planning productization is not the mainline.

## Navigation

- Engineering blueprint: [`../plan/`](../plan/)
- User-visible journeys: [`../features/`](../features/)
- Typed contracts: [`../contracts/`](../contracts/)
- Proof cases: [`../acceptance/`](../acceptance/)
- Execution order: [`../tasks/01-execution-roadmap.md`](../tasks/01-execution-roadmap.md)
- Cross-layer mapping: [`../coverage-matrix.md`](../coverage-matrix.md)
