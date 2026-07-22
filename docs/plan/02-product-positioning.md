# Product positioning

## Metadata

- `Layer`: Product foundation
- `Status`: Accepted
- `Version`: `0.2.0`
- `Last Review`: `2026-07-22`
- `Authority Owns`: product scope, formal first-party identities, non-goals and naming
- `Authority Defers To`: chapters 03–08 for engineering mechanisms and typed contracts for exact fields
- `Counterpart Features`: `docs/features/00-market-browse-install.md`, `docs/features/01-ustc-affairs-navigator.md`, `docs/features/02-ustc-change-radar.md`, `docs/features/03-campus-opportunity-graph.md`
- `Counterpart Acceptance`: `FP-*`, `MARKET-*` in `docs/acceptance/matrix.tsv`
- `Primary Code Areas`: `market/`, `plugins/first-party/`, `crates/platform-core/`

## 1. Decision

USTC Campus Agent is a campus-scoped Agent platform, not a general-purpose Agent framework. Its product spine is Plugins Market-first: user-visible capabilities are inspected, installed, authorized, disabled, upgraded and audited through `PluginPackage` contracts.

The project is a student competition prototype and is **not an official University of Science and Technology of China service**.

## 2. Three default first-party Plugins

| Package | Product-facing name | Primary question |
|---|---|---|
| `ustc.affairs-navigator` | USTC Affairs Navigator | What should I do now? |
| `ustc.change-radar` | USTC ChangeRadar | What changed, and does it affect me? |
| `ustc.opportunity-graph` | Campus Opportunity Graph | What fits me, and what should I choose next? |

All three are formal default first-party products. Implementation priority does not remove, subordinate or merge any product identity.

They share the Campus Trust Kernel but keep independent package identity, version, installation, enable/disable and acceptance boundaries.

## 3. Frozen implementation order

```text
ChangeRadar source/revision/diff foundation
→ Affairs Navigator structured procedure entry
→ ChangeRadar per-board semantic feed and RSS/Atom
→ Opportunity Graph consent/profile integration
```

Course Planning is a vertical slice inside `ustc.opportunity-graph`. Its deterministic Rust planner was completed early as a bounded offline spike. It is retained, but it does not establish Opportunity Graph as the sole flagship, change this order or prove Market installation/runtime integration.

## 4. Product contracts

- **Affairs Navigator**: reviewed tree/procedure artifacts first; exact/structured lookup before targeted refresh; bounded retrieval over approved snapshots is a later fallback and never the initial truth path.
- **ChangeRadar**: individually approved public sources, immutable snapshots, normalized semantic diffs, board-scoped candidate maintainers, administrator publication and per-board RSS/Atom.
- **Opportunity Graph**: reviewed opportunities plus a consent-aware, tenant-isolated profile projection; Course Planning is one domain slice rather than the whole product.

## 5. Competition MVP non-goals

- no automatic enrollment or course-selection clicking;
- no storage of raw USTC passwords or CAS sessions as product credentials;
- no arbitrary third-party hosted code execution;
- no generic graph database or universal workflow engine merely for symmetry;
- no full-corpus RAG as the first Affairs truth path;
- no cross-user profile data or silent permission expansion;
- no public repository/download claim before the public-readiness and release gates pass;
- no invented Chinese product brand or slice display name.

## 6. Current naming

- Repository slug: `ustc-campus-agent`
- Product name: `USTC Campus Agent`
- Chinese name: TBD
- Chinese descriptor: 面向科大学生的插件化校园智能体
- Approved slice display names: none

The word `Campus` intentionally constrains the product to campus information, opportunity planning, workflows and plugin-governed services.

## 7. Verification

- Exact first-party IDs/versions/status/capabilities/install policy: `FP-006`.
- Rust/catalog identity match: `FP-015`.
- Independent disable/re-enable: `FP-007` (planned).
- Current task order: `docs/tasks/01-execution-roadmap.md`.
