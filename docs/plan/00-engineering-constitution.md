# Engineering constitution

## Metadata

- `Layer`: Foundation
- `Status`: Governing rule
- `Version`: `0.1.0`
- `Last Review`: `2026-07-22`
- `Authority Owns`: engineering priority, skeleton governance, change discipline
- `Authority Defers To`: accepted product scope and ADRs for what is built
- `Counterpart Acceptance`: `docs/acceptance/gates.md`
- `Primary Code Areas`: repository-wide

This chapter constrains engineering execution. It does not choose product features; it determines how approved features enter a stable system.

## 1. Skeleton before implementation

Work proceeds in this order:

```text
system ontology and authority
→ module and lifecycle boundaries
→ typed contracts and failure paths
→ implementation slices
→ evidence
```

A local implementation MAY explore a hypothesis, but it MUST be labelled as a bounded spike until the owning plan, lifecycle and acceptance contract are complete. A spike cannot silently become the product spine.

## 2. Priority order

When goals conflict, use:

```text
correctness and safety
> usability
> ecosystem compatibility
> maintainability and diagnosability
> performance
> memory
> storage
> secondary concerns
```

Lower-priority optimization MUST NOT weaken source authority, permissions, tenant isolation, deterministic validation or audit evidence.

## 3. Authority before convenience

- Canonical identity, package policy, grants, approvals, source revisions, receipts and audit are owned by typed Rust domain contracts or reviewed declarative artifacts.
- UI state, model output, framework checkpoints, search indexes, caches and database projections are not allowed to replace their owning authority.
- External systems enter through narrow adapters. Their types and state machines MUST NOT leak into domain authority merely to reduce adapter code.
- The same fact MUST have one owning source. Other documents and projections link to it.

## 4. Modules before speculative boundaries

Start with cohesive modules. Introduce a new crate, service or repository only when at least one real condition exists:

- independent deployment, privilege, release or failure isolation;
- multiple implementations or consumers;
- a high-value dependency direction that needs compiler enforcement;
- measured build/runtime isolation benefit.

Conceptual nouns are not automatically crates, services, traits or repositories.

## 5. Failure-first design

Every core flow MUST define:

- rejection conditions and typed error surface;
- what durable state may already exist;
- retry/idempotency boundary;
- rollback or recovery path;
- user/operator-visible result;
- acceptance evidence that distinguishes pass from not-run.

Fail-closed is mandatory for unknown permissions, stale/conflicting authority, unsafe source paths, unverifiable publication and incomplete effect receipts.

## 6. Change governance

Architecture-level changes include product topology, authority ownership, lifecycle semantics, tenant/data boundaries, protocol state machines and source-of-truth changes. They require:

1. explicit problem and counterevidence;
2. impact, migration and rollback analysis;
3. verification plan;
4. Develata approval;
5. plan/feature/contract/acceptance updates before or with implementation.

## 7. Definition of done

A slice is done only when:

- its owning contract is current;
- implementation claims match actual status;
- relevant automated gates pass with real output;
- manual or target-host requirements are recorded as pass, fail, blocked or not-run—not assumed;
- independent review finds no unresolved blocker;
- publication/push occurs only with explicit authorization.
