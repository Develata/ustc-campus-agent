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
├── finite HarnessRun + TaskGraph       (accepted H0 contract; implementation planned)
├── Plugin-neutral AgentRun             (R0 transition kernel implemented)
├── Agent tool protocol + ToolGateway   (boundary accepted; H0 implementation planned)
├── Campus Trust Kernel                 (contract; planner subset exists)
├── first-party product use cases       (mostly planned)
└── audit/evidence                      (planned)
    │
    ├── reviewed Git declarations
    ├── durable operational store       (future)
    ├── immutable evidence store        (future)
    └── model/tool/source adapters and Plugin executors (replaceable)
```

The Rust domain core owns legal transitions. A conversation may contain many finite `HarnessRun`s; each graph node may own one bounded `AgentRun`. `PluginPackage` components reach the Agent only after the resolver/gateway compiles them into the versioned Agent tool protocol. Agent code never loads Plugin manifests or implementations; Plugin code never imports the Agent state machine. `ustc-agentd` is the composition root. Clients, prompt projections, context summaries, model frameworks, databases, caches and adapters remain projections or infrastructure.

Every model request is measured before provider I/O against a pinned context-window policy. Deterministic offloading and bounded lossy compression may reduce the working prompt, but never rewrite canonical transcript, graph, receipts or evidence.

## Current executable slice

Implemented today:

- Rust workspace and daemon/CLI skeleton;
- exact three first-party identities and manifest contract;
- deterministic repository/manifest checks;
- offline Course Planning fixture validation and planner;
- framework-neutral Agent run-spec, transition, replay, effect-ordering and budget kernel;
- pure deterministic typed invocation resolver with executable synthetic fixtures and bounded `RunSpec` mapping;
- mechanically enforced Agent–Plugin dependency direction, with the cross-boundary proof owned by the composition root.

The next platform slices are the H0 finite harness kernel and the P0b/P0c Market authority branch; no real invocation application consumer exists yet. They converge before the bounded user Agent journey. The next first-party product slice remains ChangeRadar source/revision/diff. Additional Course Planning productization is not the mainline.

## Navigation

- Engineering blueprint: [`../plan/`](../plan/)
- User-visible journeys: [`../features/`](../features/)
- Typed contracts: [`../contracts/`](../contracts/)
- Proof cases: [`../acceptance/`](../acceptance/)
- Execution order: [`../tasks/01-execution-roadmap.md`](../tasks/01-execution-roadmap.md)
- Cross-layer mapping: [`../coverage-matrix.md`](../coverage-matrix.md)
