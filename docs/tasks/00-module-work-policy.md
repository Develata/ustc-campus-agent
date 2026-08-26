# Large-module work policy

## Metadata

- `Status`: Current task policy
- `Version`: `module-work-policy/v1.5`
- `Last Review`: `2026-08-24`
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

### Authority-owned parallel module waves

After public contracts, acceptance bindings and module write ownership are frozen, implementation switches from architecture serialization to a bounded module-milestone wave:

```text
one Integration Authority Owner
→ freeze public DTO/error/version/permission/source semantics and fake contracts
→ two or three isolated module implementation lanes prove standalone behavior against those fakes
→ one continuous read-only auditor checks scope/contract drift
→ the Integration Authority Owner selects exact candidates and serially fans in real dependencies
→ one context-isolated terminal reviewer examines the frozen integrated candidate
→ the Integration Authority Owner adjudicates findings, runs final gates and owns the only completion verdict
```

The Integration Authority Owner is the sole owner of public-contract decisions, shared governance/current-truth carriers, the canonical candidate, composition order, acceptance promotion, review adjudication and final commit. It may implement small central protocol/composition carriers directly when that prevents semantic ambiguity; it need not write every module-private implementation.

Each top-level implementation lane has one independently named durable parent process, isolated worktree/run root, exact frozen base and contract digest, bounded write set and typed receipt stream. OMO/ULW/internal subagents are intra-module fan-out only; they do not become additional cross-module owners. Initial writing concurrency defaults to two or three lanes and increases only after measured host/model capacity and disjoint custody justify it.

Module-private code and tests may run concurrently against fake counterparts. Shared DTO/error/version/permission/source semantics, root dependency resolution, composition roots, acceptance matrices, module map, roadmap and contract checker remain serialized under the Integration Authority Owner. A module lane may keep a worktree-local `Cargo.lock` for proof, but only the integration owner regenerates or admits the canonical lockfile.

Module workers stop at `STANDALONE_IMPLEMENTATION_COMPLETE_AWAITING_FANIN` or `BLOCKED:<reason>`. A fake-backed green lane does not prove integration, acceptance, full CI or production readiness. Real dependency binding and status promotion proceed in dependency order through the declared composition surface.

A continuous read-only auditor may inspect taskbooks, scope, fake/real compatibility, projection closure and negative tests while writers run. It cannot mutate producer worktrees, approve acceptance or substitute for the context-isolated terminal review. An internal reviewer sharing the producer's parent session/model is useful audit evidence but is not an independent terminal verdict.

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
→ no more than three concurrent review subagents; total rounds uncapped
→ main agent verifies/fixes and closes every review lane
→ final baseline/contract/acceptance checks
→ real feature smoke when applicable
→ next small module or exact-scope commit
```

No issue/task text bypasses the owning plan.

## 3. Branch and commit shape

Two paths are admitted. Path A is the default. Path B is a bounded exception with a fail-closed admission test; it is permission for independently gated slices, not permission to fragment arbitrary work or to bypass the large-module architecture.

### Path A — default, coupled large-module path

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

Use Path A when batches are coupled, the public boundary is unresolved, integration must be reviewed atomically, or any Path B criterion below is unmet. Under Path A a small module may be committed before the large module is complete, and remote push/merge waits for the large module's exit gate plus operation-specific or active-campaign Develata authorization under the campaign rules below.

### Path B — independently gated small-module path

```text
one declared small-module batch
→ one exact-scope feature branch
→ accepted owning contract + active planned acceptance bindings
→ implementation and independent standalone evidence
→ truthful plan/contract/acceptance/status projections
→ final gate and independent review
→ authorized push
→ protected-main PR
→ authorized merge as bounded/partial evidence
```

#### Path B admission criteria

A small module is admitted to Path B only when **all** of the following are true:

1. it is named in the owning blueprint/roadmap small-module decomposition;
2. its large module already passes the contract-ready gate in §4;
3. it has an accepted exact owning contract, and active `planned` acceptance rows with non-vacuous future evidence bindings existed before implementation started;
4. it has one semantic intent, narrow owned paths and explicit non-goals;
5. its public inputs/outputs and dependency direction are settled — its standalone proof requires no unresolved sibling implementation and no hidden cross-module integration;
6. every bound test, checker, fake or negative-space proof for that slice passes independently;
7. every implementation and status projection it affects is updated honestly in the same slice;
8. the large-module state is not inflated — normally at most `partial-evidence` after the first retained small module, and never `StandaloneReady` until §7 passes;
9. final independent review has no unresolved blocker;
10. Develata authorizes push and merge either for the current operation or through an active source-controlled campaign grant that satisfies the rules below;
11. remote `main` protection, exact-path staging and PR CI are unchanged.

Failure of any single criterion falls back to Path A. A task brief, issue or agent instruction cannot self-declare an exception.

#### What a Path B merge does and does not prove

A Path B merge records bounded partial evidence for exactly the declared slice. It normally moves a `planned` large module no further than an honest `partial-evidence` in the module registry. It never establishes `StandaloneReady`, `IntegrationReady`, `Integrated` or `Accepted`; `StandaloneReady` remains bound to the full standalone gate in §7, and later readiness remains bound to §8 and §9.

<!-- CAMPAIGN_AUTHORIZATION_POLICY:BEGIN -->
#### Source-controlled campaign authorization

Remote-operation authorization has exactly two admitted forms:

1. an operation-specific instruction from Develata; or
2. an `active` campaign grant recorded in the canonical execution roadmap or another source-controlled task authority linked from it.

An issue, taskbook, agent instruction or implementation branch cannot grant authority to itself. Only Develata may create, activate, amend, relocate, pause or revoke a campaign grant. The grant MUST carry an immutable campaign ID and approved base commit, name its finite repository/module/batch and path scope, allowed push/PR/merge operations, controller and sole merge authority, required gates, stop conditions, completion/revocation conditions and review trigger.

Campaign authorization never covers mutation of its own authority surface. Creating or changing this policy block, an active grant block, their checker/digest/mutation tests, root authorization projections, GitHub workflow/CODEOWNERS files or branch-protection settings requires operation-specific Develata approval. A campaign may update ordinary module-status text elsewhere in a roadmap only when its finite path scope names that carrier and the grant block remains byte-exact.

A campaign grant MAY separately authorize a `proposal-only` lane before a legal merge path exists. Such a lane MAY create a bounded checkout, edit only finite named paths, create local commits, push an exact reviewed commit and open/update a Draft PR. It MUST NOT merge, contain retained implementation, mark a planned acceptance row implemented, or present an unsettled plan/contract proposal as accepted authority. Settling the proposal requires operation-specific Develata approval and a new or amended grant before merge or implementation.

A campaign grant MAY separately authorize an `audit-only` lane for behavior-neutral reconciliation of current code against current plans/contracts. Such a lane MAY merge only taskbook, roadmap or blueprint audit evidence inside finite named paths. It MUST NOT retain implementation, promote acceptance posture, accept or amend a contract, or change topology, authority, permission, lifecycle, protocol, ordering or runtime behavior. It follows the same source binding, local gates, independent review, PR CI, repair accounting, prospective-tree and post-main proof below, but Path A/Path B implementation admission is not claimed; an `amend` conclusion pauses before the authoritative or code change.

An active campaign grant MAY authorize iterative branch pushes, pull requests and protected-`main` merges for a retained-implementation delivery lane without repeated prompts only while every operation remains inside the recorded scope and all of these conditions hold:

1. each slice is rebound to the exact authoritative `main` commit and tree after the previous merge and post-merge CI;
2. the slice passes Path A or every Path B admission criterion independently;
3. its accepted contract/taskbook selects one bounded implementation with no unresolved authority, permission, lifecycle, protocol or public-behavior choice;
4. before each candidate push, local gates pass, the exact changed-path set and outgoing commit range are audited, and independent review of that commit has no unresolved blocker;
5. the controller pushes that exact commit, creates or updates the admitted PR form, then requires every exact-head CI context to succeed before merge;
6. a CI/review repair push is allowed only after the canonical taskbook records the blocker and repair round and conditions 1–4 are rerun for the new commit; the new exact head restarts condition 5, while round `2` with the same blocker pauses rather than pushes;
7. immediately before merge, the merge authority reads live `main` as `M0` and the reviewed head as `H`, requires `behind_by=0` and merge-base `M0`, proves the prospective merge tree equals the reviewed `H` tree, then reads live `main` again as `M1` and requires `M1 == M0`; otherwise it rebases onto the new exact base, reruns review/gates/CI and restarts the snapshot;
8. the merge authority verifies the merged tree plus exact-main post-merge CI before activating the next slice.

The controller MUST pause before the next source or remote mutation and ask Develata when any of the following occurs:

- the controller must ask the user to choose among behavior, authority, permission, lifecycle, protocol, scope or risk alternatives;
- independent reviewers disagree about public behavior or authority ownership and the governing contract does not resolve the disagreement;
- debugging requires live user interaction, unplanned instrumentation/data access, or source changes outside the declared paths/contracts;
- the same semantic blocker or required-gate failure survives two documented repair-and-review rounds recorded in the active taskbook or PR body;
- the authoritative base moves unexpectedly, shared governance carriers diverge, or the prospective merge tree cannot be proved mechanically equal to the reviewed tree;
- a security, privacy, source-permission, secret, destructive, production, publication or release decision falls outside the named grant.

A campaign grant never waives the repository's existing Develata-approval gates for product topology, authority ownership, permission semantics, lifecycle states or runtime state-machine changes. Tags, releases, public publication, visibility changes, branch-protection changes, credential handling and production infrastructure mutations remain excluded unless the grant names them explicitly.
<!-- CAMPAIGN_AUTHORIZATION_POLICY:END -->

### Rules for both paths

- one commit has one semantic intent;
- stage exact files only;
- do not mix unrelated cleanup or another large module's implementation;
- cross-module integration code stays in a declared composition surface, never hidden inside the small module;
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

After the quick gate, run no more than three independent review subagents concurrently. This is a concurrency limit, not a cap on total reviewers or review rounds. Recommended independent axes for a large-module batch are:

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
- Develata authorizes push/merge through an operation-specific instruction or an active campaign grant under §3.

Merge does not imply every future/later item in the blueprint is implemented.

## 10. Architecture review state

The S0 architecture/module documentation review is complete. The global review freeze is lifted, but implementation remains governed by the contract-ready, small-batch and active-acceptance gates:

- no retained product/business implementation begins before its owning module has exact active planned acceptance rows with future evidence bindings;
- existing Agent runtime, resolver, protocol and Course Planning code is retained as executable design evidence;
- simple client/backend skeleton initialization may begin only after its module contract is current and the roadmap's acceptance prerequisites are met;
- root interfaces/composition scaffolding may be refined without committing to unfinished concrete adapters;
- no current `planned` acceptance row is promoted from documentation alone.
