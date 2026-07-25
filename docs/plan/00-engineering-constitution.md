# Engineering constitution

## Metadata

- `Layer`: Foundation
- `Status`: Governing rule
- `Version`: `1.0.0`
- `Last Review`: `2026-07-25`
- `Scope`: long-lived engineering judgment, architecture construction and change governance
- `Authority Owns`: engineering priority, skeleton construction, modularity, reliability and correction principles
- `Authority Defers To`: project-local governance and owning plans/contracts for product topology, technology choices, module identities, work sequence and current delivery gates

This constitution contains durable engineering principles. It deliberately does not name a product, framework, repository path, module ID, deployment target or current project phase. Those changeable constraints belong in project-local governance or the owning plan and contract.

## 1. General rule

1. Define the system ontology, skeleton and boundaries first.
2. Define independently owned modules and their public contracts second.
3. Define internal decomposition and implementation third.
4. Local features fill the accepted skeleton; they do not silently redesign it.
5. Implementations may change and modules may be replaced. Routine feature work must not repeatedly rewrite the main skeleton.
6. An MVP chooses implementation order; it does not define the system ontology.

Exploratory code is allowed only when labelled as a bounded spike. A spike is evidence, not automatic architecture authority. It may be retained, rewritten or removed after review.

## 2. Strict priority order

When goals conflict, use:

```text
correctness and safety
> usability and user-visible workflow
> compatibility with stable foundations and public contracts
> maintainability and diagnosability
> performance
> memory
> storage
> secondary concerns
```

A lower-priority objective must not weaken a higher-priority one.

## 3. Ontology and skeleton construction

Build the system around durable domain objects, relationships and capability axes rather than the first release's feature list, a convenient framework model or an accidental implementation detail.

### 3.1 Capability admission test

A capability enters the stable skeleton only when all three questions have acceptable answers:

1. **Ontological necessity** — does it directly serve a core purpose or core object relationship?
2. **Same-domain extension** — is it a natural extension of the existing problem domain rather than a nearby but different product?
3. **Heterogeneous intrusion** — can it join without importing a new authority model, dependency topology or lifecycle that changes the nature of the system?

If the third answer is no, keep the capability outside the main skeleton as a separate product, bounded integration or rejected scope.

### 3.2 Default layer model

The default call architecture is four layers plus an object plane:

```text
1. Interaction shell
   render state and capture user intent
        ↓
2. Instruction interface
   admit, validate and translate intent
        ↓
3. Flow coordination
   order work and enforce legal transitions
        ↓
4. Capability execution
   perform bounded computation, I/O and external effects

Object plane beside the call path
   names the durable objects and state being read or changed
```

The object plane is not a fifth caller. A project may refine the layer names and module mapping, but must preserve the separation of presentation, admission, coordination, bounded execution and owned state.

Rules:

- Presentation changes are not automatically business-state changes.
- User intent enters through an admitted interface before becoming a domain mutation.
- Coordination uses declared contracts and does not reach through private storage or implementation details.
- Execution adapters perform bounded work but do not decide domain truth merely because they can cause effects.

## 4. Modularity, uniformity and replacement

An independently owned module:

- owns one coherent responsibility and one bounded object/state family;
- hides internal state and implementation choices;
- exposes narrow, versioned inputs, outputs and errors;
- has no cyclic dependency with peer modules;
- has explicit lifecycle, failure and replacement rules;
- can be tested against controlled counterparts;
- joins the system through a declared composition surface.

If two parts change for different reasons, fail independently, use different authority, or can be replaced independently, do not force them into one module merely to reduce component count.

Conversely, a conceptual noun is not automatically a crate, service, repository, trait or Plugin. Begin with cohesive internal modules. Extract a stronger boundary only when dependency direction, privilege, deployment, release, failure isolation, replacement or multiple real consumers justify it.

Complexity is allowed only in a uniform form:

- the same responsibility uses the same layer;
- equivalent failures use the same error model;
- equivalent boundary values use the same owned types;
- equivalent configuration enters through the same typed discipline;
- equivalent asynchronous work uses the same lifecycle semantics.

Implementations in one variation family remain peers below one stable abstraction. Add a peer implementation or adapter instead of growing special cases through the main flow. Shared code is extracted from stable shared meaning, not merely similar syntax.

## 5. Blueprint and implementation maturity

A construction-ready engineering blueprint must define at least:

- responsibility and non-goals;
- owned objects, state and legal transitions;
- public inputs, outputs and errors;
- dependency and call direction;
- lifecycle, including retirement and deletion;
- failure, recovery and user-visible outcomes;
- configuration and secret boundaries;
- observability and verification;
- extension and replacement points;
- performance and resource-critical paths;
- implementation slices and deferred scope.

A document that only names layers and desired properties is a concept sketch, not an engineering blueprint.

Retained implementation may begin only when:

- the relevant ontology and responsibility are clear;
- the public boundary, source of truth and dependency direction are explicit;
- failure, recovery, configuration and observability have concrete entry points;
- obvious wrong directions and heterogeneous scope have been excluded;
- the first verification path and evidence binding are defined;
- any remaining exploration is labelled as a bounded spike and cannot silently become the permanent main path.

Passing this maturity gate does not mean every later capability is designed or implemented. It means the current slice can land without polluting the future mainline.

## 6. Authority and consistency

Every core fact has one owning source of truth. Views, caches, indexes, framework state and user-interface state may project an owned fact, but do not become peer authorities merely by storing a copy.

When two representations disagree, the owning plan must state which source repairs the other. No implementation may create two peers that can both claim final truth for the same fact.

New behavior enters through declared boundaries. It must not use cross-layer shortcuts, cross-module writes or duplicated policy to bypass the owner.

## 7. Failure, observation and lifecycle

Every core flow defines failure before implementation:

- rejection conditions and stable error classes;
- state that may already exist when failure occurs;
- propagation, retry and duplicate-suppression boundaries;
- rollback, resume or operator recovery;
- user/operator-visible outcome;
- evidence that distinguishes pass, fail, blocked and not-run.

Logs, state events, receipts, redacted diagnostics and resource metrics are architectural capabilities, not after-the-fact patches.

Any repeatedly triggered operation must be idempotent or carry explicit duplicate suppression. Concurrent mutation must define ordering, conflict handling and which write wins. Persisted schemas and public protocols carry version, migration, compatibility and rollback responsibility.

Every core object and durable capability also defines its exit path: disable, revoke, archive, delete, expire, migrate and clean up, as applicable. A creation path without an owned retirement and recovery path is incomplete.

Unknown or conflicting authority, incomplete effects and unverifiable state transitions fail closed.

## 8. External foundations and adapters

Treat an external ecosystem as a foundation only when it is long-lived, infrastructure-level, broadly accepted, operationally proven and expensive to reproduce correctly. Prefer such foundations when rebuilding has low value and high maintenance or security cost.

Fast-moving, simple or weak-consensus abstractions are not foundations merely because they are popular today. Reuse their implementation only behind an owned boundary when measured value exceeds binding and migration cost; otherwise borrow the idea or implement the bounded behavior locally.

External frameworks, SDKs, databases, APIs, transports and runtimes enter through narrow adapters. Their types and checkpoints terminate at the adapter boundary. They do not define system objects or authority merely because adopting their native model is convenient.

Choose a high-level framework only after defining the owned contract and proving that replacement remains possible. A Plugin mechanism arises from real independent implementations and trust boundaries; it is not an architecture starting point.

## 9. Conservative judgment and disciplined rigor

When evidence is incomplete, choose the simpler, more stable and more reversible option. Uncertainty must not create a second authority, relax a safety boundary or introduce deep coupling without proof.

Rigor is a continuing convergence constraint:

- keep boundaries clean;
- keep abstractions stable;
- keep failure paths complete;
- keep replacement points explicit;
- keep implementation faithful to the ontology.

Rigor does not justify unbounded scope, needless abstraction, uncontrolled complexity or indefinite delay.

## 10. Skeleton changes, counterevidence and approval

A change is skeleton-level when it alters system topology, module ownership, state authority, lifecycle semantics, trust/data boundaries, public protocol, source of truth or dependency direction.

Before such a change:

1. state the current problem and counterevidence;
2. classify whether it is an implementation, module, boundary or skeleton problem;
3. explain benefit, cost, risk and the consequence of not changing;
4. identify affected objects, modules, contracts, migrations and compatibility;
5. define rollback and verification;
6. obtain explicit Develata approval;
7. update all affected authority projections and remove stale contradictions.

Past implementation effort is not a reason to keep a disproved skeleton. Correction starts immediately as analysis and review, not as an unreviewed rewrite.

A Develata-approved judgment is authoritative for execution, but it is not exempt from evidence-based review. When material counterevidence suggests that a judgment is incomplete, internally inconsistent or underestimates cost and risk, the agent must submit a concise two-sided analysis before implementation continues.

That analysis states the evidence for the current judgment, the strongest counterevidence, benefits and costs of keeping or changing it, affected authority and migrations, and the verification and rollback path. This review obligation improves decision quality; it does not transfer final approval authority away from Develata.

## 11. Completion principle

A slice is complete only when:

- its owning contract is current;
- authority and dependency direction remain clean;
- implementation status is honest;
- relevant automated and real-path evidence has been exercised or explicitly recorded as not applicable;
- review has no unresolved accepted blocker;
- exact change scope has been inspected;
- irreversible publication, deployment or shared-state effects occur only under the required authorization.
