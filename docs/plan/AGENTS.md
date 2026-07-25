<!-- Parent: ../AGENTS.md -->

# Plan layer governance

## Purpose

`docs/plan/` is the authoritative engineering blueprint for USTC Campus Agent. It defines stable ontology, authority, lifecycle, runtime boundaries, failure semantics and verification. User-facing walkthroughs belong in `docs/features/`; delivery order belongs in `docs/tasks/`.

## Chapter order

1. `00-engineering-constitution.md` — governing engineering rules.
2. `01-terminology.md` — normative language and stable terms.
3. `02-product-positioning.md` — product topology and non-goals.
4. `03-platform-authority.md` — system authority and deployment boundaries.
5. `04-market-and-plugin-lifecycle.md` — Market, packages, installations and grants.
6. `05-campus-trust-kernel.md` — source, revision, provenance and publication authority.
7. `06-first-party-plugins.md` — the three first-party product contracts.
8. `07-runtime-and-integration.md` — owned Agent runtime and adapter boundaries.
9. `08-security-and-delivery.md` — security, publication and release constraints.
10. `modules/00-module-map.md` — stable large-module ownership, dependency direction and assembly order.
11. `modules/*.md` — one independently owned engineering blueprint per large module.

The numeric filename prefixes under `modules/` are reading-order numbers, not module IDs. The `Module ID` field inside each blueprint and `modules/00-module-map.md` are authoritative.

## Chapter contract

Each chapter SHOULD declare:

- Metadata: layer, status, version, last review, authority ownership, counterparts and code areas;
- scope and explicit non-goals;
- authoritative entities and state transitions;
- invariants and forbidden patterns;
- failure and recovery semantics;
- runtime/configuration boundaries;
- verification entrypoints.

Each large-module blueprint under `modules/` MUST additionally declare:

- a machine-readable `Implementation State` metadata value from the controlled module-state vocabulary in `modules/00-module-map.md`;
- purpose and non-goals;
- owned objects/state and public inputs/outputs;
- allowed callers, allowed dependencies and forbidden dependencies;
- lifecycle, failure/recovery and typed configuration;
- observability, extension/replacement points and performance path;
- `MVP`, later scope and explicit non-goals;
- small-module decomposition and independently verifiable batches;
- exact exit gate and composition-root integration point.

Do not mechanically fill empty sections. Authority, failure, runtime boundary and verification, however, MUST be explicit.

## Change discipline

- Product topology, authority ownership, permission semantics, lifecycle states and runtime state-machine changes require Develata approval before implementation.
- Implementation status belongs in manifests, acceptance rows and task status—not in optimistic plan prose.
- An early bounded spike proves only its stated contract and MUST NOT redefine product identity or implementation order.
- A large module is defined by independence rather than size. It MUST be testable against fake counterparts and attach only through a declared composition surface.
- Cross-module implementation details do not belong in a module plan. Put shared public shapes in `docs/contracts/` and integration ordering in `docs/tasks/`.
- When a plan changes beyond typo-level, update `Last Review` and all affected feature/contract/acceptance mappings in the same slice.
