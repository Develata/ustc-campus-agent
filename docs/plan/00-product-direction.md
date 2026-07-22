# Product direction

## Decision

USTC Campus Agent is a campus-scoped Agent platform, not a general-purpose Agent framework. Its product spine is Plugins Market-first: user-visible capabilities are installed, authorized, disabled, upgraded, and audited through `PluginPackage` contracts.

## Three default first-party Plugins

| Package | Product-facing name | Primary question |
|---|---|---|
| `ustc.affairs-navigator` | USTC Affairs Navigator | What should I do now? |
| `ustc.change-radar` | USTC ChangeRadar | What changed, and does it affect me? |
| `ustc.opportunity-graph` | Campus Opportunity Graph | What fits me, and what should I choose next? |

All three are formal default first-party products. Implementation priority does not remove or subordinate any product identity.

They share the Campus Trust Kernel: source identity, immutable revisions, authority ordering, effective time, conflicts, provenance, grants, and audit. They remain independently installable, disableable, versioned, and testable `PluginPackage`s.

## Frozen implementation order

```text
ChangeRadar source/revision/diff foundation
→ Affairs Navigator structured procedure entry
→ ChangeRadar per-board semantic feed and RSS/Atom
→ Opportunity Graph consent/profile integration
```

Course Planning is a vertical slice of `ustc.opportunity-graph`. Its deterministic Rust planner was completed early as a bounded offline spike. It is retained, but it does not establish Opportunity Graph as the sole flagship, change the frozen implementation order, or prove Market installation/runtime integration.

## Product contracts

- Affairs Navigator: reviewed tree/procedure artifacts first; structured lookup before targeted refresh; bounded RAG over approved snapshots is later fallback, never the initial truth path.
- ChangeRadar: approved official sources, immutable snapshots, normalized semantic diffs, board-scoped candidate maintainers, administrator publication, and per-board RSS/Atom.
- Opportunity Graph: reviewed opportunities plus a consent-aware, tenant-isolated profile projection; Course Planning is one domain slice, not the whole platform.

## Non-goals for the competition MVP

- no automatic enrollment/选课 clicking;
- no storage of raw USTC password or CAS session as product credential;
- no arbitrary third-party hosted code execution;
- no Android-native full experience before the Web/PWA loop is stable;
- no generic graph database or universal workflow engine introduced merely for architecture symmetry;
- no full-corpus RAG as the first Affairs truth path;
- no public repository or public download claims before public-readiness and release gates pass.

## Current naming

- Repository slug: `ustc-campus-agent`
- Product name: `USTC Campus Agent`
- Chinese name: TBD
- Chinese descriptor: 面向科大学生的插件化校园智能体
- Slice display names: none approved

The name intentionally contains `Campus` to constrain the Agent to campus information, opportunity planning, workflows, and plugin-governed services.
