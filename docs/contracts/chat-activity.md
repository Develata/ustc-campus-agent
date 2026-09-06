# Bounded Chat activity projection

- Contract: `CHAT-ACTIVITY-001 / chat-conversation-activity/v1`.
- Owner: M30 observer and application projection; M10 transports, browser renders.
- Scope: observable provider/tool lifecycle for the existing saved conversation;
  answer-token streaming and explicit cancellation under the extension below; no hidden reasoning or durable Harness claim.
- Acceptance: `CHAT-005`, targeted observer, HTTP and browser cases.

## Authority and reference

The design adapts the separation of tool-call/result events and compact tool rows
observed in the locally installed DeepSeek Harness 0.1.2-rc.1 (`dsh-agent-loop`,
`dsh-client-ui-tool`). No DSH runtime or source code is incorporated. UCA retains its
own Rust admission, product receipts and package/Skill trust boundaries. A displayed
activity is evidence of an observed phase, never a grant or completion receipt.

## Observer contract

The finite Chat loop emits bounded provider-start/end and tool-start/end observations.
Provider-start means the request passed context construction and is entering the provider
stage, including adapter preflight; provider success means a model response was received and accepted by that stage,
not that the user's overall task succeeded. Tool-start is emitted only for a concrete
validated call that is about to reach the existing executor. A denied proposal can
produce a denied terminal step without a started event. Complete-batch validation
must still occur before any executor entry. No observer can authorize, alter or retry
a call; observation failure cannot change the product execution disposition.

Only loop-assigned step/call identities, known tool names and closed status values
cross the observer. No prompts, arguments, tool results, raw provider IDs, profiles,
credentials or private routes appear. The existing final response's safe tool trace
remains the stored authority for completed tool outcomes.

## Read-only application query

`GET /api/v1/agent/conversations/{id}/activity` requires protocol major 1 and the
existing loopback Host/Origin gate before application dispatch. Identity comes from
composition; callers cannot supply tenant/user. Wrong-owner and absent conversations
have the same not-found result. The query reads only the current turn and bounded
in-memory observations, without cloning the full transcript or writing state.

The exact DTO is:

```text
schema: chat-conversation-activity/v1
conversation_id: server-owned conversation ID
request_id: current request ID or null for an empty conversation
phase: idle | running | completed | failed | interrupted
sequence: integer 0..15
steps: at most 7 entries
  id: bounded server-generated stable step ID
  kind: model | tool
  tool: known fixed tool name, or null for a model step
  status: running | succeeded | denied | failed
```

At most three model and four tool steps correspond to the existing run budgets.
Active observations use a monotonically increasing sequence at most 14; terminal
projection uses 15. A request ID, not a conversation sequence alone, identifies the
sequence domain. An empty conversation has sequence zero and no steps. Closed errors
use the saved-dialogue error envelope; missing/wrong major uses M10 compatibility.

The activity registry has at most four active entries and releases them when the
owning run task leaves. Admission/finish is still durable; this projection is not.
After completion, disappearance of a tracker or restart, only stored tool traces may
be reconstructed. Historical model phases are not invented. A failed/interrupted
turn with no saved response may have no reconstructed steps and may already have
product effects. Neither missing observations nor browser disconnection cancels work.

## Browser behavior and recovery

The client polls only while observing its submitted request, with bounded response
validation, no overlapping GETs, cancellation on switching/settlement, and stale
response rejection by both conversation and request identity. Phase text derives
from the server. Before the first matching observation it says the request was sent
and is awaiting status; it does not guess a tool action from a timer or user words.

A compact status line and expandable tool rows preserve the conversation reading
order on desktop/mobile and in light/dark themes. A read-only status failure must
not change the submitted request result or trigger a retry. Lost/uncertain POSTs
retain the existing explicit read-back recovery path. Stop-token UI is absent until
an actual cancellation contract is implemented. This does not stream model reasoning.

## Verification

Observer cases prove provider failures, complete-batch rejection, actual executor
ordering and denied calls. Application cases prove owner separation and terminal
reconstruction. HTTP verifies major admission, exact snapshot and GET-only behavior.
Browser uses a delayed controlled provider/response to inspect live state, expansion,
malformed/stale response rejection, recovery independence and responsive themes.

### Package extension

The closed safe tool category also includes `plugin_tool` for actual package-backed
MCP execution or Skill context reads. The exact private model-visible name stays
in the application/tool transcript; public activity and saved traces use the fixed
category. Existing sequence, owner, result and non-retry rules apply unchanged.


## STREAM-CANCEL-001 extension

This extension supersedes the v1 sequence/cache-only/absent-stop descriptions above.
The saved-dialogue activity endpoint now returns `chat-conversation-activity/v2`:
the existing fields plus `partial_answer` (at most 16 KiB UTF-8). Running sequence
is monotonic below 4294967295; terminal sequence is 4294967295. Tool/model steps
interrupted while pending project `status=interrupted`, meaning their effect outcome
must be checked, never that a write was rolled back. Owner authentication is the
same as the saved conversation.

The provider adapter requests true Chat Completions SSE for bounded chat runs,
consumes complete SSE frames across arbitrary network chunk boundaries, and exposes
only content deltas. It requires a valid assistant role, single index-zero choice,
complete stop/tool_calls termination and DONE sentinel. Accumulated tool fragments
pass the original complete-message parser and complete-batch argument validation
before any executor. Response bytes, text, calls, arguments, context and absolute
HTTP timeout remain bounded. Complete JSON responses remain supported for compatible
peers without synthesizing token animation; title generation keeps its JSON path.

`POST /api/v1/agent/conversations/{id}/cancel` accepts only
`{schema:"chat-conversation-cancel/v1",request_id}`, with the existing ASCII request ID,
JSON, protocol-major, Host/Origin and authenticated owner admission. A mismatched
current request gives conflict; it cannot cancel the next turn. An accepted stop
returns the current activity snapshot, not a claim that effects were rolled back.
A terminal turn returns its snapshot without another effect. An admission-to-tracker
gap returns in-progress for deliberate retry. Cancellation drops the pending
factory/provider/tool/title future, checks before every new provider/tool call,
and persists the terminal error `chat_cancelled`. Disconnection alone still does
not stop a run. Already committed local/remote effects must be checked independently.

After each completed tool and at termination, the application saves a bounded private
checkpoint containing observed steps, accumulated answer text, and up to four exact
validated tool-result envelopes (64 KiB each), including any product receipt IDs.
These envelopes are retained for evidence; they are never replayed as new writes,
new grants, or new prompt authority. Activity exposes only safe step categories and
answer text. Tool outputs are not sent through activity. Every token is memory-only;
a crash may lose text since the last bounded checkpoint. Checkpoint failure stops
further work and fails closed without claiming prior effects were absent.

Conversation store version 3 adds an optional closed private progress field. Version
1/2 stores are accepted with absent progress; the first checkpoint upgrades to 3.
Root-prompt changes never downgrade a version-3 store. Version/progress shape,
bounds, tool-result envelope, step IDs and closed enums are revalidated on open.
The running capacity reserve is 512 KiB for terminal text plus bounded checkpoints.
Restart converts running turns to interrupted and preserves checkpoints; it performs
zero provider/tool redispatch. Browser refresh may reattach read-only polling to the
saved exact request. A later user request is new intent, not automatic write replay.

Verification: `cargo test --locked -p ustc-agentd --lib streaming`,
`cargo test --locked -p ustc-agentd --lib execution_tests`, and the existing
activity/conversation tests. The local HTTP SSE peer withholds its final frame until
the client acknowledges the first delta; cancellation tests prove a pending tool
future is dropped, wrong-owner/stale requests cannot cancel, terminal retries do not
execute, and restart retains completed results.
