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

## Initial executable scope

This repository currently initializes the Rust workspace and contract skeleton only. Implementation PRs must bind behavior to docs/contracts and acceptance cases before expanding runtime state.
