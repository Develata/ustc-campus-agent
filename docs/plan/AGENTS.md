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

## Chapter contract

Each chapter SHOULD declare:

- Metadata: layer, status, version, last review, authority ownership, counterparts and code areas;
- scope and explicit non-goals;
- authoritative entities and state transitions;
- invariants and forbidden patterns;
- failure and recovery semantics;
- runtime/configuration boundaries;
- verification entrypoints.

Do not mechanically fill empty sections. Authority, failure, runtime boundary and verification, however, MUST be explicit.

## Change discipline

- Product topology, authority ownership, permission semantics, lifecycle states and runtime state-machine changes require Develata approval before implementation.
- Implementation status belongs in manifests, acceptance rows and task status—not in optimistic plan prose.
- An early bounded spike proves only its stated contract and MUST NOT redefine product identity or implementation order.
- When a plan changes beyond typo-level, update `Last Review` and all affected feature/contract/acceptance mappings in the same slice.
