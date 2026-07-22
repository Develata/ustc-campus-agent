# Three default first-party Plugins

USTC Campus Agent exposes three formal default first-party products through one Market and one Campus Trust Kernel.

## Product split

```text
USTC Affairs Navigator：我现在该怎么办？
USTC ChangeRadar：什么变了，是否影响我？
Campus Opportunity Graph：什么适合我，下一步选什么？
```

| Plugin | Projection | First honest user result |
|---|---|---|
| Affairs Navigator | reviewed procedure projection | conditions, steps, deadlines, entry points, sources, uncertainty |
| ChangeRadar | approved semantic-change projection | before/after, effective time, affected scope, provenance, RSS/Atom |
| Opportunity Graph | opportunity + consent-aware profile projection | qualification, dependency, conflict, match, and next action |

## Shared authority

```text
approved official/public sources
→ Source Registry
→ immutable source revisions
→ normalized facts
→ temporal / conflict / provenance authority
   ├── reviewed procedure artifacts
   ├── approved semantic change events
   └── reviewed opportunity graph facts
```

Affairs Navigator and ChangeRadar must share the same source/revision/change ledger. A crawler or maintainer may create candidates; it cannot publish canonical facts. Opportunity Graph consumes reviewed facts and keeps user profile data tenant-scoped and consent-aware.

## Lookup and materialization boundary

Affairs Navigator uses this order:

```text
L0 exact stable ID/path/URL lookup
→ L1 reviewed tree + structured local search
→ L2 approved-source targeted refresh and typed candidate
→ L3 bounded retrieval over approved snapshots (later)
```

The production materialization path is:

```text
approved SourceRevision
→ reviewed Skill produces typed ProcedureDraft
→ Rust schema/cross-field/citation/policy validation
→ deterministic Markdown
→ administrator review and atomic publish
```

A formatting hook may normalize presentation; it cannot fill missing semantics, invent citations, or publish.

## ChangeRadar boundary

- retrieval is limited to individually reviewed source definitions, not arbitrary USTC URLs;
- immutable raw and normalized snapshots precede semantic diff;
- failures do not advance the accepted baseline;
- board-scoped maintainer Agents hold candidate-only authority;
- RSS/Atom contains approved semantic changes, never crawl or parser noise;
- subscriptions bind stable board/node IDs rather than mutable paths.

## Implementation order

```text
1. ChangeRadar source/revision/diff foundation
2. Affairs Navigator structured procedure entry
3. ChangeRadar per-board feed
4. Opportunity Graph consent/profile integration
```

Course Planning is retained as an out-of-order bounded spike within Opportunity Graph. It proves deterministic planning only; it does not change this order or claim Market/runtime completion.
