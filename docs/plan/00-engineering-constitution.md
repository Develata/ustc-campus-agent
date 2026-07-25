# Engineering constitution

## Metadata

- `Layer`: Foundation
- `Status`: Governing rule
- `Version`: `0.3.0`
- `Last Review`: `2026-07-25`
- `Scope`: architecture, plan, contracts, tasks, implementation, review and delivery
- `Authority Owns`: engineering priority, system skeleton, module discipline, work sequence and change governance
- `Authority Defers To`: accepted product scope and ADRs for what is built
- `Counterpart Overview`: `docs/overview/architecture.md`
- `Counterpart Acceptance`: `docs/acceptance/gates.md`, repository checker and each module's bound acceptance rows

This constitution constrains all engineering work in USTC Campus Agent. Its purpose is to prevent a local feature, framework or short-term convenience from gradually rewriting the product skeleton.

## 1. General rule

1. Define the system skeleton and boundaries first.
2. List independently owned large modules second.
3. Define each module's public contract and internal small modules third.
4. Implement only after the first three steps are clear.
5. Local features fill the accepted skeleton; they do not silently redesign it.
6. Implementations may change and modules may be replaced. Routine feature work must not require repeated rewrites of the main skeleton.

Exploratory code is allowed only when labelled as a bounded spike. A spike is evidence, not automatic architecture authority. It may be retained, rewritten or removed after the design review.

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

A lower-priority optimization must not weaken tenant isolation, source authority, grants, deterministic validation, receipts, audit evidence or recoverability.

## 3. Stable system skeleton

The main call path is four layers plus an object plane:

```text
1. Interaction shell
   Dioxus Fullstack Web/PWA + Android; later iOS/desktop; or operator CLI
        │ display server-owned state; submit typed intent
        ▼
2. Application interface
   ustc-agentd Dioxus server functions / HTTP / typed streams / command boundary
        │ authenticate, validate compatibility/envelope, map DTOs, call owned services
        ▼
3. Flow coordination
   platform application services, finite HarnessRun, product use cases
        │ order work, enforce transitions, coordinate modules
        ▼
4. Execution domain
   resolvers, planners, source pipeline, tool gateway, provider and Plugin executors

Object plane beside the call path:
   packages, installations, grants, runs, events, sources, revisions,
   procedures, changes, opportunities, profile facts, artifacts and receipts
```

The object plane is not a fifth caller. It names the concrete objects whose state the four-layer call path reads or changes.

Rules:

- A UI action enters through the application interface before it can become a domain mutation.
- Coordination code calls modules through declared contracts; it does not reach into their storage or implementation details.
- Execution adapters perform bounded work but do not decide platform truth.
- A database, cache, framework checkpoint, UI signal or model transcript is not authoritative merely because it stores a copy.
- Cross-module assembly belongs at a declared composition surface, primarily `apps/ustc-agentd`.

## 4. Large-module rule

A large module is defined by independence, not physical size or line count. It must:

- own one coherent responsibility and one bounded object/state family;
- hide internal state and implementation choices;
- expose narrow, versioned inputs, outputs and errors;
- have no cyclic dependency with another large module;
- be developable and testable against fake counterparts;
- have its own lifecycle, failure rules and acceptance gate;
- join the product through a declared composition surface;
- remain replaceable without rewriting unrelated modules.

Large modules combine like independently tested robot parts. One module may be completed and attached while another remains a fake or planned implementation. Integration does not grant either module access to the other's internals.

A large module must not become a miscellaneous home for unrelated work. If two parts change for different reasons, fail independently, use different data, or can be replaced independently, they should not be forced into one implementation module merely to reduce crate count.

Conversely, a conceptual noun is not automatically a crate or service. Begin with cohesive Rust modules. Extract a crate, process or repository only when dependency direction, privilege, deployment, release, failure isolation or multiple consumers justify it.

## 5. Small-module rule

Each large module is divided into small modules that:

- implement one closely related set of rules;
- expose the smallest practical surface;
- do not share mutable global state;
- do not import sibling internals;
- can be reviewed and committed as one semantic slice;
- include failure tests before being considered complete.

Avoid generic `utils`, universal managers, service locators and shared context bags. Shared code is extracted only after the same stable meaning has at least two real consumers. Similar syntax alone is not shared semantics.

## 6. Required blueprint for every large module

Before substantial implementation, every large-module plan must answer all of the following:

1. **Purpose** — what responsibility belongs here?
2. **Non-goals** — what tempting nearby work is explicitly excluded?
3. **Owned objects and state** — which facts and transitions does it decide?
4. **Public inputs and outputs** — which commands, queries, events or protocol values cross the boundary?
5. **Dependency direction** — which modules may it call, and which modules may call it?
6. **Lifecycle** — how are its objects created, activated, updated, disabled, revoked, archived and deleted?
7. **Failure and recovery** — what fails, what remains durable, what retries, and what the user sees?
8. **Configuration** — which typed configuration enters, who owns it, and where secrets are referenced?
9. **Observability** — which stable errors, events, metrics and receipts prove behavior?
10. **Extension and replacement points** — which implementations may be swapped without changing the contract?
11. **Performance path** — where are the bounded hot paths, I/O limits and memory/resource ceilings?
12. **MVP, later and non-goals** — what is implemented now, deliberately deferred and forbidden from accidental scope growth?
13. **Small-module decomposition** — which independently reviewable pieces build the large module?
14. **Exit gate** — which exact acceptance rows, tests and real smoke prove that the module can be attached?

A chapter that only names layers and desired properties is a concept sketch, not an engineering blueprint.

## 7. Dioxus Fullstack thin-client rule

Dioxus Fullstack is the chosen long-lived first-party Rust application stack. Web/PWA, a native Linux server deployed through Docker Compose and Android are required product targets. Web is the first proof surface; Android follows as a mandatory peer target. iOS and desktop are later scope.

The target arrangement is:

```text
Dioxus shared UI + Web/PWA + Android adapters
        │
        ├── may own routes, components, presentation state, forms and accessibility
        ├── may own SSR/page delivery and generated first-party client calls
        └── calls versioned Dioxus server functions / HTTP / typed streams
                         │
                         ▼
             M10 ingress in ustc-agentd
                         │ admitted application command/query
                         ▼
             backend application/domain modules
```

The client must not own:

- domain calculation or canonical business state;
- Agent, Market, grant or Plugin decisions;
- source normalization, planning or publication logic;
- database mutation or direct executor access;
- task completion, receipts or audit truth.

A client may perform display-only reduction, input validation for user feedback and local formatting. Every calculation or mutation that affects product truth executes through backend/application infrastructure.

A Dioxus server function is an Axum-compatible M10 ingress adapter, not merely page plumbing. After M00/M10 version, identity, authorization, bounds, idempotency/precondition and audit admission, its server-only body may call one public application command/query port. It may not call concrete repositories, databases, Plugin executors, provider SDKs or journals directly. Public REST/SSE endpoints, when required, are peer adapters over the same application ports rather than a second business implementation.

Shared Rust source does not remove deployed compatibility obligations. Web may deploy atomically with the server; Android packages may lag. First-party request/response/error/event contracts therefore remain versioned and define a supported window plus typed `UpgradeRequired` behavior before unsafe dispatch.

## 8. One owner for every fact

Every core fact has one owning source:

- reviewed Git owns package declarations and engineering contracts;
- Rust domain rules own legal state transitions;
- durable operational state owns installations, grants, runs and user-private facts;
- immutable evidence objects own raw/normalized source revisions;
- receipts and journals own acknowledged effects;
- UI, caches, search indexes and framework state contain rebuildable views only.

When two sources disagree, the plan must say which one repairs the other. No implementation may create two peers that can both claim final truth for the same fact.

## 9. Failure, observation and repeatability

Every core flow defines failure before implementation:

- rejection conditions and stable error classes;
- state that may already exist when failure occurs;
- retry and duplicate-suppression boundary;
- rollback, resume or operator recovery;
- user/operator-visible result;
- exact evidence that distinguishes pass, fail, blocked and not-run.

Logs, state events, receipts, redacted diagnostics and resource metrics are built into module contracts. They are not added after production failure.

Any repeatedly triggered operation must be idempotent or carry an explicit duplicate-suppression key. Concurrent mutation must define ordering, conflict handling and which write wins. Persisted schemas and public protocols carry version, migration and rollback responsibility.

Unknown permissions, stale/conflicting authority, unsafe source paths, incomplete receipts and unverifiable publication fail closed.

## 10. External foundations and adapters

Prefer stable ecosystem foundations where rebuilding has low value and high maintenance cost: Rust language/tooling, HTTP, SSE, JSON, SemVer, SHA-256, Git and released external protocols.

External frameworks, provider SDKs, databases, model APIs, MCP transports, browser/native APIs and execution runtimes enter through narrow adapters. Their types and checkpoints terminate at the adapter boundary. They do not define platform objects merely because adopting their native model is convenient.

Choose an external framework only after defining the owned contract and proving that replacement remains possible. Plugin mechanisms arise from real independent implementations and trust boundaries; “we want plugins” is not the architecture starting point.

## 11. Documents, code and evidence

The authority order is:

```text
docs/plan
→ docs/contracts and controlled registries
→ docs/features
→ docs/acceptance
→ docs/tasks
→ code
→ tests and real smoke evidence
→ overview/guides/reports
```

More specific owning documents may refine a higher-level statement but cannot contradict it. ADRs explain why a decision was made; they do not override the current plan. Tasks schedule accepted work; they do not define architecture.

Code is a projection of the plan and contracts. Tests and smoke are projections of acceptance cases. `planned`, skipped, unavailable and not-run are non-pass states.

## 12. Mandatory work sequence

Every work item follows this exact loop:

1. Read `docs/plan/00-engineering-constitution.md` carefully.
2. Read `docs/plan/01-terminology.md` carefully.
3. Read relevant `docs/plan/` chapters and decide whether they must change.
4. Read relevant contracts, features, acceptance rows, registries, overview and tasks and decide which projections must change.
5. Implement the smallest cohesive code or documentation slice.
6. Run a changed-file quick gate.
7. Use at most three independent review subagents.
8. The main agent verifies findings, fixes accepted blockers and closes all review lanes.
9. Run final baseline checks, contract checks and bound acceptance-matrix commands.
10. Exercise the real feature path when one exists; otherwise record not applicable.
11. Loop to the next planned small module or make an exact-scope commit.

Do not skip directly from an issue or task list to code. If no code change is needed, the first four steps and a scoped review still apply.

## 13. Commit, integration and merge discipline

- One small-module commit contains one semantic intent.
- Several verified small-module commits may accumulate on one large-module branch.
- A large module integrates only through its declared contracts and composition surface.
- A large module is ready for push/PR/merge only after its own exit gate passes; unrelated unfinished modules may remain fake or absent.
- Remote operations still require current Develata authorization and protected-branch rules.
- During the current solo skeleton phase, Develata may allow local `main` work for root interfaces and composition scaffolding. This exception does not permit feature modules to bypass contracts, reviews or remote protection.

## 14. Skeleton changes and correction

A change is skeleton-level when it alters product topology, large-module ownership, state authority, lifecycle semantics, tenant/data boundary, public protocol, source of truth or dependency direction.

Before such a change:

1. state the current problem and counterevidence;
2. classify whether it is an implementation, small-module, large-module, boundary or skeleton problem;
3. explain benefit, cost, risk and the consequence of not changing;
4. identify affected objects, modules, contracts, migrations and compatibility;
5. define rollback and verification;
6. obtain explicit Develata approval;
7. update all current documentation projections and remove stale contradictory claims.

Past implementation effort is not a reason to keep a disproved skeleton. Correction starts immediately as analysis and review, not as an unreviewed rewrite.

## 15. Definition of done

A slice is done only when:

- the owning plan and contract are current;
- module ownership and dependency direction remain clean;
- implementation status is honest;
- relevant automated gates pass with real output;
- applicable real client/CLI/API/runtime behavior is exercised;
- manual or target-host checks are recorded as pass, fail, blocked or not-run;
- review has no unresolved accepted blocker;
- exact-scope Git state is inspected;
- push, merge, release or publication occurs only under the required authorization.
