# Execution roadmap

This roadmap projects [`00-product-direction.md`](00-product-direction.md) and [`ADR-0006`](../decisions/ADR-0006-three-default-first-party-plugins.md). It preserves the confirmed three-Plugin topology and implementation order.

## Completed foundation

- Rust monorepo, Market boundary, CI, contracts, and initial acceptance matrix;
- GitHub collaboration baseline and protected `main`;
- three default first-party package identities and manifest skeleton contract;
- bounded Course Planning spike: typed synthetic fixture, deterministic planner, CLI smoke, provenance, and fail-closed tests.

The Course Planning spike was completed out of order. It remains reusable evidence inside Opportunity Graph but does not advance the mainline past ChangeRadar or Affairs.

## P0 — Market read/install skeleton

- parse and validate all three first-party manifests;
- exact package/version/component/capability identities;
- default installation policy and independent enable/disable state;
- development identity, grant, audit, and tool-discovery boundaries;
- anonymous Market browse/detail shell.

Gate: all three package identities bootstrap at exact versions, while planned manifests make no executable/runtime claim.

## P1 — ChangeRadar source/revision/diff foundation

- reviewed Source Registry entry for one approved public USTC source pair;
- stable source and revision identities;
- conditional retrieval contract and immutable raw/normalized snapshots;
- parser fixture, normalization, and deterministic semantic diff;
- baseline advances only after snapshot, parse, normalize, diff, and durable candidate evidence succeed;
- repeated processing is idempotent; unauthorized URL fetch fails closed.

Gate: one real historical source change is replayable with exact evidence, and parser/fetch failure cannot erase or advance the accepted baseline.

## P2 — Affairs Navigator structured procedure entry

- stable tree/node and board-policy contract;
- Git Markdown/YAML reviewed canonical artifacts;
- typed `ProcedureDraft`, direct supersession edges, and current/archived lifecycle;
- Rust schema, cross-field, citation, and policy validation;
- deterministic Markdown rendering and administrator approval/publish;
- exact ID/path/URL lookup plus PostgreSQL structured search projection.

Gate: one administrator-maintained board answers a real campus procedure with conditions, steps, effective time, sources, and explicit uncertainty. Full-corpus RAG is not required.

## P3 — ChangeRadar per-board feed

- Affairs and ChangeRadar reuse the same Source Registry, immutable revisions, board policies, and change ledger;
- board-scoped maintainer workers can propose candidates but cannot publish canonical facts;
- durable leases, idempotency, and approved semantic-change events;
- per-board RSS/Atom with stable event GUID, affected scope, provenance, and before/after summary.

Gate: approved semantic changes publish once; raw HTML/hash noise, parser failure, and unreviewed inference never enter the feed.

## P4 — Opportunity Graph consent/profile integration

- reviewed opportunity graph ontology and source projection;
- consent-aware, tenant-isolated, viewable and deletable profile facts;
- qualification, dependency, temporal-window, and conflict explanation;
- integrate the existing Course Planning planner behind Market installation/grant/tool-discovery boundaries;
- retain iCourse as link-out-only unless explicit permission is obtained.

Gate: the existing offline planner becomes an honest installed-plugin journey without weakening hard constraints or source authority.

## P5 — productization and adversarial testing

- three package detail surfaces and independent disable/re-enable behavior;
- browser desktop/mobile, keyboard, focus, console/network checks;
- tenant isolation, redaction, revoke, stale-source, and recovery tests;
- compact user trial and evidence bundle;
- deployment/restore verification.

## P6 — freeze and submission

- fix blockers only;
- record the three-Plugin narrative and failure/recovery demo;
- prepare architecture, framework influence, source/license, and evidence documents;
- clean-host restore/read-back where applicable;
- submit.
