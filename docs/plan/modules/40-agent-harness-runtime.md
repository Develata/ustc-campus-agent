# M30 — Agent Harness and Runtime

## Metadata

- `Module ID`: `M30`
- `Status`: Accepted blueprint; node-local runtime kernel, provider-free deterministic turns and one bounded app-private provider/tool Chat coordinator implemented; durable finite user-task harness planned
- `Implementation State`: `partial-evidence`
- `Version`: `m30-agent-runtime/v0`
- `Last Review`: `2026-09-03`
- `Owning Plan`: [`../07-runtime-and-integration.md`](../07-runtime-and-integration.md)
- `Primary code areas`: node kernel in `crates/agent-runtime/`, current bounded app-private Chat coordinator in `apps/ustc-agentd/src/agent_chat.rs`, and future cohesive durable harness modules

## 1. Purpose

`M30` owns one finite user task and its bounded model/tool/review execution. It defines legal run/graph transitions, immutable budgets, prompt-context projection, evidence/review gates, deterministic replay and explicit terminal outcomes.

It treats models as proposers. Rust decides whether a plan, transition, tool proposal, budget charge, review disposition or completion is legal.

## 2. Non-goals

- owning Market packages, installations, grants or Plugin routing;
- implementing model provider SDKs or MCP transports;
- running Plugin code directly;
- storing UI state or exposing Dioxus types;
- becoming a generic workflow language for unrelated products;
- using a framework checkpoint, process exit or model summary as completion truth.

## 3. Owned objects and state

```text
ConversationSessionRef
HarnessRunSpec / HarnessRun phase
TaskContract / TaskGraph / GraphRevision
TaskNode / resource claims / review policy
AgentRunSpec / AgentRun phase
RunCommand / RunEvent / replay checkpoint
BudgetSnapshot and consumed counters
PromptProjection / ContextSummaryArtifact references
EvidencePack / ReviewReceipt / terminal outcome
```

Canonical transcript, graph, events, receipts and evidence remain durable state. A model request is a bounded view built from them.

## 4. Public inputs and outputs

Commands:

```text
CreateHarnessRun
SubmitClarificationAnswer
Accept/Reject TaskGraphProposal
DispatchReadyNode
StartModelTurn + RecordModelUsage | StartHarnessTurn
ToolProposal / EffectIntent / Receipt
RecordEvidence / ReviewDisposition
RequestCancel/Fail/Expire
```

Ports and outputs:

```text
ModelInvocationPort              # implemented by M50
PluginNeutralToolExchange        # proposal/result values; composition invokes M40
RunJournalPort / ArtifactPort    # implemented by M90
Clock/SchedulerPort              # implemented by M90
RunSnapshot / RunEvent / Evidence/Review projections
RuntimeError
```

`M30` sees only `agent-tool-protocol/v0` tool definitions/calls/results and opaque route references. It never receives package/component/executor details.

`StartHarnessTurn` is the provider-free path for deterministic bounded routing.
It consumes turn/tool budgets and the same intent/receipt ordering, but records no
fabricated provider token/cost usage. It is a node-local execution mode, not the
future user-task `HarnessRun` phase machine.

The app-private Chat MVP provides a second bounded M30 orchestration slice: a closed three-turn/three-call coordinator sends complete request projections through M50, validates exact tool proposals, executes tools sequentially and produces a typed final response. Its confirmed Opportunity tool may invoke the separately owned static M72 planning use case; M72 consent/profile/planning semantics remain M72 evidence, not M30/M40 implementation.

## 5. Dependency direction

Allowed dependencies:

- `crates/agent-tool-protocol` and stable value libraries;
- ports declared by `M30` for provider, journal, artifacts, clock and scheduler; Plugin-neutral proposal/result values from `agent-tool-protocol` cross through composition.

Allowed callers:

- `M10` application services through typed run commands/queries;
- composition/supervisor code that owns no alternative run state.

Forbidden dependencies:

- Market manifests and `M20` private types;
- Plugin/MCP/provider implementations;
- concrete database/framework checkpoint types;
- Dioxus/client state;
- product-specific ChangeRadar/Affairs/Opportunity logic.

## 6. Lifecycle

```text
accepted user intent
→ Contextualizing / bounded Clarifying
→ Planning / validated finite TaskGraph
→ Executing ready nodes
→ Verifying / fresh Reviewing
→ bounded Remediating | Reporting
→ Succeeded | Partial | Failed | Blocked | Expired | Cancelled
```

Each model/tool node has a bounded `AgentRun`. An unresolved external effect prevents terminal acknowledgement until its outcome/receipt is reconciled.

## 7. Failure and recovery

- Illegal transition or event sequence: reject without mutation.
- Invalid/cyclic/authority-widening graph: reject before execution.
- Context request exceeds pinned limit after bounded compaction: fail before provider I/O.
- Budget exhausted: explicit Partial/Failed/Blocked according to policy; never silently reset.
- Provider/tool timeout: typed node outcome; no hidden alternate provider/tool.
- Crash/restart: replay immutable spec and ordered events; reconcile child/effect identities.
- Reviewer rejection: append bounded remediation work; do not erase prior evidence.
- Cancellation with in-flight effect: record terminal intent and wait for exact reconciliation.

## 8. Configuration and secrets

Run behavior is pinned by immutable policy/profile IDs and integer budgets. No raw provider secret or Plugin configuration enters run specs/events. Provider/tool implementations resolve their own admitted references outside `M30`.

## 9. Observability

Events expose phase, graph revision, node, budget consumption, model/tool call correlation, evidence/review status and stable error class. UI/event projections omit hidden reasoning and raw private payloads. Every terminal claim points to required evidence and review receipts.

## 10. Extension and replacement

Model backends, tool gateways, token estimators, compaction algorithms, journals, schedulers and Agent frameworks are replaceable behind declared ports. Replacement must replay the same owned events and preserve `agent-tool-protocol/v0` behavior. Framework adoption cannot move run authority out of this module.

## 11. Performance path

Hot paths are event replay, ready-node scheduling, complete-request token measurement and prompt assembly. Graphs, retries, tool counts, context and evidence sizes are bounded by immutable policy. Waiting is event-driven; a model does not spend turns polling children.

## 12. Scope boundary

**MVP**

- finite single-task lifecycle;
- bounded clarification and finite graph;
- one provider port and one tool port through fakes/real adapters;
- replay, budgets, context preflight, evidence and fresh review;
- explicit cancel/fail/blocked behavior;
- one Dioxus-visible run projection through `M10`.

**Later**

- parallel resource-safe nodes;
- richer deterministic offloading/compression;
- durable distributed supervisors;
- additional reviewed executor/reviewer classes.

**Explicit non-goals**

- unlimited autonomous operation;
- model-generated executable workflow code;
- generic graph engine unrelated to finite Agent tasks;
- direct package/Plugin branching;
- hidden context overflow fallback.

## 13. Small-module decomposition

1. `run-spec` — immutable identities and budgets.
2. `agent-run` — node-local phase/command/event/replay kernel.
3. `harness-run` — finite user-task phases and suspension/terminal rules.
4. `task-contract` — immutable parent goal/non-goals/deliverables/acceptance.
5. `task-graph` — finite graph validation and revisions.
6. `scheduler` — dependency/resource-ready dispatch.
7. `context-budget` — complete-request measurement and integer policy.
8. `context-projection` — deterministic offload/compaction/compression artifacts.
9. `evidence-review` — EvidencePack, fresh review and remediation.
10. `runtime-ports` — provider/tool/journal/artifact/clock fakes.
11. `run-projection` — safe client/application view.

Existing `agent-runtime` is reviewed as items 1–2, not treated as proof of items 3–11.

## 14. Exit gate

`M30` is standalone-ready when all `HARNESS-*` and owned `AGENT-*` cases execute against fake model/tool/journal ports, including replay, context overflow, in-flight cancel and review remediation. It is accepted when one real API/client task reaches an evidenced terminal state with stream/non-stream convergence and restart without duplicate effects.
