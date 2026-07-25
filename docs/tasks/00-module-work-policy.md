# Large-module work policy

## Metadata

- `Status`: Current task policy
- `Version`: `module-work-policy/v1`
- `Last Review`: `2026-07-25`
- `Owning Constitution`: [`../plan/00-engineering-constitution.md`](../plan/00-engineering-constitution.md)
- `Module Registry`: [`../plan/modules/00-module-map.md`](../plan/modules/00-module-map.md)
- `Boundary Registry`: [`../contracts/module-boundaries.md`](../contracts/module-boundaries.md)

This file schedules how modules are built. It does not override their plans or contracts.

## 1. Unit of ownership

A human/agent owner takes one large module or one declared small module inside it. Ownership means:

- read and preserve the owning plan/contract;
- keep changes inside declared paths and ports;
- report non-goals and external dependencies;
- provide tests/evidence for the slice;
- not edit another module's private implementation to make integration convenient.

A large-module owner is responsible for interface consistency and final standalone exit gate. Small-module owners are responsible only for their bounded slice and public fit.

### M80 frontend design assignment

- Kimi K3 and Claude Opus 5 jointly lead actual M80 interface, interaction and visual-style design on Develata's Windows host.
- GPT-family agents do not originate the actual frontend design. Their role is independent review and explicitly bounded local code optimization, including architecture/contract, accessibility, performance and maintainability checks.
- This assignment covers presentation work such as routes, components, design-system choices and target-specific UX. It does not move product/domain authority into M80 or into any model-generated artifact.
- Changes from either lead still pass the same thin-client boundary, active acceptance rows, browser/device evidence, review and protected-main PR gates.

## 2. Mandatory work loop

Every work item follows the root `AGENTS.md` loop:

```text
read plan/00
→ read plan/01
→ read relevant plan and decide updates
→ read relevant contracts/features/acceptance/registry/overview/tasks and decide updates
→ implement one cohesive slice
→ changed-file quick gate
→ at most three independent review subagents
→ main agent verifies/fixes and closes every review lane
→ final baseline/contract/acceptance checks
→ real feature smoke when applicable
→ next small module or exact-scope commit
```

No issue/task text bypasses the owning plan.

## 3. Branch and commit shape

Normal multi-member work:

```text
one large module
→ one dedicated large-module branch
→ several exact-scope small-module commits
→ module standalone gate
→ composition adapter/integration commit
→ full module gate and review
→ authorized push
→ PR
→ authorized merge to protected main
```

Rules:

- one commit has one semantic intent;
- stage exact files only;
- do not mix unrelated cleanup or another large module's implementation;
- a small module may be committed before the large module is complete;
- remote push/merge waits for the large module's exit gate and current Develata authorization;
- review evidence includes status, specs, changed/untracked files and real gate output.

During the current solo skeleton phase, Develata may explicitly allow local `main` work for root interfaces/composition scaffolding. Remote main protection and all review/gate/authorization rules still apply.

## 4. Contract-ready gate

Before code beyond a disposable spike, the large module has:

- one blueprint under `docs/plan/modules/`;
- named public boundaries in `docs/contracts/module-boundaries.md` or a more specific contract;
- explicit owned state and forbidden dependencies;
- fake inbound/outbound design;
- MVP/later/non-goal split;
- small-module list and exit gate;
- acceptance rows in planned state with exact future evidence bindings.

## 5. Small-module batch contract

Each small batch states:

```text
module ID
owned responsibility
inputs/outputs
changed paths
non-goals
success and failure tests
quick gate
acceptance rows affected
integration impact: none | public contract | composition
```

Prefer small internal Rust modules before creating new crates. Extract only for compiler-enforced dependency, independent deployment/privilege/release/failure or multiple real consumers.

## 6. Review contract

After the quick gate, use at most three independent review subagents. Recommended independent axes for a large-module batch are:

1. contract/architecture and dependency direction;
2. correctness/failure/recovery and acceptance evidence;
3. security/simplicity/performance and integration risk.

The main agent must independently verify findings. Accepted blockers are fixed in the same work loop; false/non-applicable findings are recorded with reason. All review lanes must be complete before final gates or delivery.

## 7. Standalone module gate

A large module becomes `StandaloneReady` only when:

- all planned small modules for its current scope are complete;
- fake counterparts exercise every public port;
- success, malformed, denied, timeout, duplicate/restart and revoke/cancel cases relevant to the module pass;
- no forbidden dependency or internal reach-through exists;
- configuration, errors, events, metrics and recovery are defined;
- exact module acceptance rows pass;
- docs/code/status agree.

## 8. Assembly gate

A standalone module joins the product through its declared composition surface:

```text
public module adapter
→ composition mapping/order
→ cross-module integration fixtures
→ applicable API/client/CLI/runtime smoke
```

The composition layer may map IDs/DTOs and order public commands. It must not copy private validation or mutate another module's repository directly.

An unrelated large module may remain a fake. No synchronized “big bang” integration is required.

## 9. Module completion and merge

A large module is merge-ready when:

- its current MVP/module scope is `Accepted` or explicitly `IntegrationReady` under the roadmap gate;
- all accepted blocker reviews are closed;
- final baseline/contract/acceptance checks pass;
- applicable real feature smoke passes or is honestly not applicable;
- no dirty/foreign files enter the commit range;
- PR describes module boundary, small-module commits, evidence, risks and deferred work;
- Develata authorizes current push/merge operations.

Merge does not imply every future/later item in the blueprint is implemented.

## 10. Current freeze

Until the current architecture/module documentation review converges:

- no new product/business implementation begins;
- existing Agent runtime, resolver, protocol and Course Planning code is retained as executable design evidence;
- simple client/backend skeleton initialization may begin only after its module contract is current;
- root interfaces/composition scaffolding may be refined without committing to unfinished concrete adapters;
- no current `planned` acceptance row is promoted from documentation alone.
