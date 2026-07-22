# System overview

```text
Web/PWA client
    │ versioned HTTP JSON + SSE
    ▼
ustc-agentd
├── identity/session
├── market catalog projection
├── installation/grant resolver
├── bounded conversation runner
├── tool gateway
├── Affairs Navigator procedure use cases
├── ChangeRadar source/revision/diff and feed use cases
├── Opportunity Graph / Course Planning use cases
└── audit/evidence
    │
    ├── Git market catalog in this repo
    ├── PostgreSQL or lightweight dev store later
    ├── immutable source snapshots later
    ├── model backend adapters
    └── MCP/typed service adapters
```

The Rust domain core owns canonical decisions: package identity, grants, approvals, receipts, source revisions, and acceptance evidence. Adapters and framework workers may assist execution but cannot overwrite authority state.

The three default first-party Plugins share one Campus Trust Kernel but keep independent package/install/enable boundaries. See [`03-three-first-party-plugins.md`](03-three-first-party-plugins.md).

## Initial executable scope

This repository currently initializes the Rust workspace and contract skeleton only. Implementation PRs must bind behavior to docs/contracts and acceptance cases before expanding runtime state.
