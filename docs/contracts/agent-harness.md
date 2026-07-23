# Finite Agent harness contract

## Metadata

- `Status`: Accepted target architecture; H0 implementation planned
- `Version`: `agent-harness/v0`
- `Last Review`: `2026-07-24`
- `Owning Plan`: [`../plan/07-runtime-and-integration.md`](../plan/07-runtime-and-integration.md)
- `Feature Projection`: [`../features/04-bounded-agent-harness.md`](../features/04-bounded-agent-harness.md)
- `Acceptance`: `HARNESS-*`; existing node kernel remains `AGENT-*`
- `Primary Code`: future platform-owned orchestration modules around `crates/agent-runtime/`

## 1. Scope and invariants

The harness owns one finite user task from accepted input to evidenced report. It does not replace the conversation session or the existing single-node [`AgentRun`](agent-runtime.md).

```text
ConversationSession = ordered sequence of finite HarnessRuns
HarnessRun          = finite phase machine + validated TaskGraph + durable evidence
TaskNode            = immutable TaskContract + bounded executor/reviewer cycle
AgentRun             = one node's model/tool/effect loop
```

Normative invariants:

1. A conversation session MAY outlive many runs; every `HarnessRun` reaches one terminal phase.
2. The model proposes clarification, plans and graph patches; Rust validates and applies typed commands/events.
3. Durable run state, transcript, graph revisions, receipts and evidence are canonical. A model prompt is a bounded projection only.
4. Clarification, model turns, tool calls, retries, review and remediation are all bounded by immutable policy snapshots.
5. A hook, model summary, UI state or framework checkpoint never proves task or effect completion.
6. The same accepted specification and ordered events replay to the same checkpoint.

## 2. Harness phase machine

A `HarnessRunSpec` pins at least:

```text
schema version and harness run ID
conversation/session reference and tenant/user scope
accepted user intent and clarification policy
immutable root TaskContract
policy-capsule and provider-profile snapshot IDs
context-budget, total-cost/time and review-budget snapshots
initial TaskGraph revision or permission to propose one
```

Phases are:

```text
Received → Contextualizing → Clarifying
Clarifying → Planning | AwaitingUser
AwaitingUser → Contextualizing | Planning | Remediating | Expired
Planning → PlanValidated → Executing
Executing → Verifying → Reviewing
Reviewing → Reporting | Remediating | Planning | AwaitingUser
Remediating → Executing
Reporting → Succeeded | Partial | Failed | Blocked

terminal alternatives: Succeeded | Partial | Failed | Blocked | Expired | Cancelled
```

Only typed commands may cross phase boundaries. Terminal phases reject mutation. `AwaitingUser` is a typed suspension carrying reason, questions/decision, deadline and `resume_phase`; timeout produces `Expired`, not an implicit abandoned run.

Legal final-review disposition mapping is:

| Disposition | Harness transition |
|---|---|
| `Pass` | `Reviewing → Reporting → Succeeded` |
| `RemediateWithinScope` | `Reviewing → Remediating → Executing` |
| `Replan` | `Reviewing → Planning`, under the same root contract |
| `NeedsUserDecision` | `Reviewing → AwaitingUser`; an in-contract answer resumes `Planning` or `Remediating` |
| `PolicyBlocked` | `Reviewing → Reporting → Blocked` |
| `BudgetExhausted` | `Reviewing → Reporting → Partial | Failed` |

Failure, cancellation and expiry close the graph as follows:

| Source | Typed command/precondition | Transition |
|---|---|---|
| any nonterminal phase with no unresolved child/effect | `Fail(error)` | `→ Failed` |
| any nonterminal phase with no unresolved child/effect | `Cancel(reason)` | `→ Cancelled` |
| any nonterminal phase with no unresolved child/effect | `Expire(deadline)` | `→ Expired` |
| `Executing` with an unresolved child/effect | `RequestFail | RequestCancel | RequestExpire` | persist terminal intent; remain `Executing` until child outcome and every required effect receipt reconcile, then enter the requested terminal phase |

A terminal request never discards or guesses an in-flight effect. Committed receipts and completed-node evidence remain attached to `Failed`, `Cancelled` or `Expired` outcomes. `AwaitingUser` deadline uses `Expire`; the immutable whole-run deadline may do so from any nonterminal phase under the same reconciliation rule.

An answer that changes the root goal, prohibitions, deliverables, acceptance or immutable budget cannot resume the same run; the current run reports a non-success outcome and any expanded work starts under a new `HarnessRunSpec`. No disposition or terminal transition is interpreted from model prose.

## 3. Clarification gate

A first context snapshot is assembled before clarification so that already-known policy, project state and prior accepted answers are not asked again.

An uncertainty is blocking only when different answers materially change at least one of:

- scope or deliverable;
- architecture or irreversible effect;
- authority, privacy or permission boundary;
- acceptance criterion;
- material cost or deadline.

Blocking questions are batched, typed and bounded. Non-blocking uncertainty becomes an explicit conservative assumption in the plan. A clarification answer is immutable run input; changing it creates a new graph revision or a new run according to scope, never silent prompt mutation.

## 4. Task graph

A model emits `TaskGraphProposal`; the runtime accepts it only after validating:

- schema/version, unique node IDs and known executor classes;
- acyclicity and complete dependency references;
- immutable parent-owned task contracts;
- capability, path/service resource claims and isolation compatibility;
- per-node and total budgets;
- no authority widening or forbidden fallback.

The root `TaskContract` is pinned before graph proposal. Every initial graph or later patch must partition or refine that contract, cover its deliverables and acceptance criteria, and preserve it byte-for-byte. Completed nodes and their evidence are immutable; remediation adds validated nodes/edges or explicit supersession rather than rewriting history.

Each `TaskNode` contains:

```text
node ID and dependency IDs
immutable TaskContract
executor and isolation policy
read/write/service resource claims
node, review and remediation budgets
verification policy
```

Each `TaskContract` contains:

```text
goal and non-goals
hard policy references and relevant philosophy capsule
input and deliverable artifact references
acceptance criteria
verification plan
```

A child planner MAY refine implementation steps and verification order. It MUST NOT weaken or rewrite goal, non-goals, policy, deliverables or acceptance criteria.

The client plan panel is a projection of accepted graph state: task summary, dependencies, executor, phase, acceptance, evidence and blockers. It never exposes hidden chain-of-thought or becomes execution authority.

## 5. Node execution and review

A simple node may execute directly. A complex or high-risk node uses:

```text
immutable TaskContract
→ optional planner
→ plan gate
→ worker AgentRun
→ EvidencePack
→ fresh read-only reviewer
```

The worker thread MAY resume across remediation attempts. Each reviewer invocation is fresh and receives only the immutable contract, current exact artifacts/state, evidence, unresolved findings and prior review receipts. It does not receive worker hidden reasoning.

A `ReviewReceipt` contains one disposition:

```text
Pass
RemediateWithinScope
Replan
NeedsUserDecision
PolicyBlocked
BudgetExhausted
```

Review and remediation are bounded. A failed final review appends a validated remediation graph revision; it does not restart clarification or discard completed graph state. `NeedsUserDecision` suspends under the mapping above. Exhausted immutable budget terminates `Partial` or `Failed`; extending it requires a new run.

## 6. Model-context budget

Every provider call—including clarification, planning, worker, reviewer and compression calls—passes the same preflight gate.

Let:

- `L` be the validated model context-window limit;
- `ρ` be `send_ceiling_bps`, with `1 ≤ ρ ≤ 10_000`;
- `τ` be `compaction_target_bps`, with `1 ≤ τ < ρ`;
- `T(q)` be the model-specific token count or validated conservative upper bound for the complete serialized input request `q`;
- `O` be reserved maximum output tokens;
- `S` be protocol and estimator safety reserve.

A request may be sent iff

```text
T(q) + O + S ≤ floor(L × ρ / 10_000).
```

After compaction or compression, the rebuilt request must satisfy the stronger target

```text
T(q') + O + S ≤ floor(L × τ / 10_000).
```

The pinned `ContextBudgetSnapshot` contains `L`, `ρ`, `τ`, `O`, `S`, estimator/tokenizer identity and version, maximum deterministic compaction passes, maximum compression levels/calls and maximum source tokens per compression request. Integer checked arithmetic is mandatory; floating-point policy values are forbidden.

`T(q)` covers every provider-visible input: system/policy text, messages, tool definitions, structured-output schema, attachments using provider-declared accounting, and adapter framing. The runtime MUST NOT use a character heuristic or a context limit inferred from model name. A provider profile without a validated limit and compatible estimator is not eligible for a call.

## 7. Compaction and compression

The preflight pipeline is deterministic in order:

```text
assemble complete PromptProjection
→ measure
→ if over ceiling: compact eligible payloads
→ rebuild and measure
→ if still over: compress eligible history in bounded chunks
→ rebuild and measure against target
→ send, or fail ContextBudgetExceeded without provider I/O
```

Terms are distinct:

- **Compaction**: deterministic reduction of the working projection, primarily replacing reproducible or already-persisted large payloads with typed artifact references and bounded previews.
- **Compression**: lossy summarization of eligible history into a typed `ContextSummaryArtifact`.

Neither operation edits or deletes the canonical transcript, graph journal, artifact, intent, receipt or evidence. A summary has no authority beyond prompt projection.

The non-compressible anchor set includes:

- current system/policy capsule and authority boundary;
- current user intent and accepted clarification answers;
- active `TaskContract` and accepted graph-revision digest;
- protocol-valid current-turn tool-call/result pairs;
- pending effect intents/receipts and unresolved review findings;
- artifact/evidence references required by acceptance.

Authorized tool definitions are included whole or omitted as a whole by a validated relevance/grant projection; schemas are never truncated.

Compression is compiled first into a finite, acyclic `CompressionPlan` whose level count, call count and source tokens per request satisfy the pinned snapshot. Each planned compression request includes its fixed prompt, input chunk, output reserve and safety reserve, then passes the same inequality exactly once. A compression call MUST NOT re-enter compaction or compression; if its prebuilt request does not fit, the whole plan fails with `ContextBudgetExceeded` before that provider call. Multi-level summaries are allowed only as explicit nodes of the already-bounded plan.

Each summary records source ranges/digests, producer model/profile or deterministic algorithm identity, created time and before/after token counts.

If the provider still reports context overflow after a passing local estimate, the adapter records estimator drift and fails closed or performs one policy-bounded rebuild. It never loops blindly or silently changes model/profile.

## 8. Supervision and completion

The scheduler dispatches only dependency-ready nodes whose resource claims are compatible. Waiting is event-driven; the main model does not spend turns polling child processes.

A supervisor records process/session handle, start, heartbeat, exit and structured `ChildOutcome`. A node completes only when:

```text
executor reached a valid terminal outcome
+ required artifacts exist
+ EvidencePack validates
+ required review receipt passes
```

Lifecycle hooks may update metrics or UI, but `SubagentStop`-like notification alone is not completion evidence. Crash/restart reconciliation uses durable graph and child identities.

## 9. Verification and reporting

Verification is selected by artifact/effect type, not merely by whether `git diff` exists. Evidence may include exact diff/head, tests, remote read-back, receipts, source provenance, rendering, schema checks or deterministic derivations.

The main model reports only from verified graph state and evidence. If the run ends `Partial`, `Failed`, `Blocked`, `Expired` or `Cancelled`, the response names that state, completed deliverables, unresolved blockers and recovery action without presenting partial work as success.

## 10. Non-goals

- an unbounded autonomous loop;
- a generic user-programmable workflow language;
- arbitrary model-generated JavaScript or framework graph as platform authority;
- shared raw context across coordinator, worker and reviewer;
- lossy summary as memory, evidence or audit truth;
- provider overflow as the normal compaction trigger;
- hard-coding another framework's threshold without a pinned platform policy.
