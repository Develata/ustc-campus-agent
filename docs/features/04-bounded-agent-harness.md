# Bounded Agent harness journey

- `Status`: Planned user journey; no production orchestrator or UI exists
- `Owning plan`: [`../plan/07-runtime-and-integration.md`](../plan/07-runtime-and-integration.md)
- `Contracts`: [`../contracts/agent-harness.md`](../contracts/agent-harness.md), [`../contracts/agent-plugin-boundary.md`](../contracts/agent-plugin-boundary.md)
- `Acceptance`: `HARNESS-*`, `AGENT-017/018`, `PKG-019/020`

## Goal

A user submits one task and receives either an evidenced result or an explicit non-success state. The platform may clarify, plan, delegate and review, but every phase is finite, visible and recoverable.

## User-visible states

```text
Understanding
Needs input
Planned
Running
Verifying
Needs decision
Completed
Partial | Failed | Blocked | Expired | Cancelled
```

The plan panel shows accepted task nodes, dependencies, current phase, acceptance, evidence and blockers. It does not expose hidden reasoning or imply that a planned node has run.

## Journey

```text
user submits intent
→ platform loads relevant bounded context
→ asks only material blocking questions, if any
→ user answers before the displayed deadline
→ platform accepts and displays a validated plan graph
→ ready nodes execute under bounded permissions and budgets
→ Plugin capabilities are exposed only as frozen versioned tools through ToolGateway
→ each required node produces evidence and receives independent review when policy requires it
→ final verification passes
→ platform reports evidenced results
```

The graph may degenerate to one direct node. Planner, delegation and model reviewer are conditional mechanisms, not mandatory ceremony for simple work.

The user-visible Agent remains stable when a Plugin is installed, updated or disabled: only the approved tool projection changes. Conversely, changing the Agent framework/harness does not rebuild or rewrite Plugin packages while the major tool protocol remains compatible.

## Context behavior

Before every model call the platform measures the complete provider request against the pinned model context policy. If it is too large, the platform first offloads redundant persisted payloads, then compresses eligible older history while preserving current intent, policy, task contract, unresolved findings and exact evidence references.

Canonical history remains recoverable. If a safe request still cannot be built, the call is not sent and the user sees a typed context-budget failure or a request to narrow scope; the platform never silently changes model or drops hard constraints.

## Failure and recovery

- No answer before deadline: the run becomes `Expired`; later work forks or creates a new run rather than mutating that terminal run.
- Invalid or unsafe plan: no node starts; show the stable denial and recovery action.
- Child crash or lost heartbeat: reconcile from durable state; never infer success from a hook event.
- Reviewer rejection: append the bounded remediation steps to the same graph; preserve completed work.
- Scope or permission expansion: pause in `Needs decision`; do not auto-approve.
- Review/budget exhaustion: report `Partial` or `Failed` with completed evidence and unresolved blockers.

## Non-goals

- an endless chat turn presented as one task;
- mandatory multi-agent execution for every request;
- user-visible chain-of-thought;
- arbitrary workflow scripting;
- arbitrary Plugin callbacks or package-specific state-machine branches inside the Agent;
- reporting a model summary, process exit or hook callback as proof of completion.
