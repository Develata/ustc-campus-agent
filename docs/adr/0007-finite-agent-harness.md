# ADR-0007: Finite platform-owned Agent harness

- `Status`: Accepted
- `Date`: `2026-07-24`
- `Depends on`: [`ADR-0004`](0004-runtime-reference-strategy.md)

## Context

R0 owns one bounded `AgentRun` and its tool-effect ordering, but it does not define a complete user task: clarification, a dependency graph, subagent supervision, review/remediation, final evidence or context-window control. Treating a long conversation as one infinite run would erase task boundaries; letting a framework or model-generated workflow own these states would invert platform authority.

Current reference systems converge on useful mechanisms without sharing one ontology: Claude Code uses finite agentic loops, subagents and automatic compaction; its dynamic workflows add fan-out and adversarial verification. Deep Agents combines offloading, summarization and subagent context isolation. Pi preserves lossless session history behind lossy compaction. Hermes separates prompt assembly, provider resolution, compression and session persistence.

## Decision

Introduce a finite platform-owned `HarnessRun` above `AgentRun`.

```text
ConversationSession
└── finite HarnessRun
    ├── bounded clarification
    ├── validated TaskGraph
    ├── direct or supervised node executors
    ├── per-node evidence and fresh review
    ├── bounded remediation graph revisions
    └── final verification and report
```

The immutable `HarnessRunSpec` pins the root `TaskContract`; model-proposed graphs may only partition or refine it. Rust validates and evolves typed state. Worker context may continue across remediation, while each reviewer is fresh and read-only. Lifecycle hooks are projections, not completion truth.

Every model invocation passes a platform-owned token-budget preflight over the complete serialized request. The threshold and lower post-compaction target are fixed-point policy values pinned with the provider limit and estimator identity. Deterministic offloading precedes lossy compression; canonical transcript, graph, receipts and evidence remain unchanged. Compression uses a prevalidated finite call/chunk plan and cannot recursively invoke itself. If any rebuilt request does not fit, no provider call occurs.

The exact target contract is [`agent-harness/v0`](../contracts/agent-harness.md).

## Rejected alternatives

- one infinite nested session/grill/worker/reviewer loop;
- arbitrary generated workflow code or framework checkpoints as authority;
- a child planner that can rewrite parent goal or acceptance;
- persistent shared raw context for worker and reviewer;
- hook callbacks or process exit as success evidence;
- provider overflow as the normal trigger;
- copying another framework's percentage as an unversioned magic number.

## Consequences

- `HarnessRun` and `AgentRun` remain distinct replay domains linked by stable IDs.
- The client renders accepted graph state without receiving execution authority or chain-of-thought.
- Context compression changes only prompt projection and creates provenance-bearing summary artifacts.
- H0 must prove phase/graph/context/review invariants with deterministic fake executors before production provider or subagent adoption.
- A bounded TaskGraph is admitted; a generic workflow language remains deferred.
- Rig, Claude Code, LangGraph/Deep Agents, Pi, goose and Hermes remain references or bounded adapters under ADR-0004.
