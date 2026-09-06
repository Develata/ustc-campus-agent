# Bounded persistent Chat conversations

- Contract: `CHAT-CONVERSATION-001 / chat-conversation/v1`.
- Owner: M30 bounded Chat application service in `ustc-agentd`; HTTP and browser are projections.
- Scope: durable transcripts around the existing bounded Chat loop. This is not a complete HarnessRun, authentication service, effect journal, or multi-user deployment acceptance.

The composition supplies `(TenantId, UserId)` from admitted identity; clients never select an owner. The current loopback composition supplies its existing explicit demo identity. Listing and reading reveal only that exact owner's conversations. Public source state is unchanged.

## HTTP admission

The conversation HTTP projections require `X-USTC-Client-Protocol-Major: 1` before
application dispatch, including malformed path/body cases. Missing/old/new/duplicate
major uses the existing M10 compatibility envelope; no conversation is created and
no tool executes. Mutations additionally require JSON under the 16 KiB body bound
and existing loopback Host/same-authority Origin admission. Errors use
`chat-conversation-error/v1` with a stable `error` code: invalid intent/Chat validation
400, not found 404, request/revision/in-progress conflicts 409, capacity 429 and
unavailable 503. Terminal failed/interrupted turns are saved result views, not an
instruction to retry an effect. These routes do not provide public user authentication.

## Commands and views

`ConversationStore` owns create, list, get, begin-turn and finish-turn. Create intent has schema `chat-conversation-create/v1` and an ASCII request ID (1–128 alphanumeric, dot, dash or underscore bytes). Turn intent has schema `chat-conversation-turn/v1`, request ID, expected revision, message and the existing optional opportunity context and prompt customization. Caller-supplied history is not accepted. Create retries return the same conversation. Turn retries must preserve the full typed intent, expected revision and current confirmation boolean; a conflict rejects before execution. For conversations that have not been logically deleted, exact pending retries report `conversation_in_progress`; exact terminal retries return the persisted result without provider or tool execution. Deleted conversations reject old create/turn requests under conversation-management.md.

List uses `chat-conversation-list/v1` and summaries `{id,title,revision,turn_count}`. Get/create use `chat-conversation/v1` and `{id,title,revision,turns}`. Each turn exposes `{request_id,user,phase,response,error}`. Result uses `chat-conversation-turn-result/v1` and `{conversation_id,revision,turn}`. Internal owner, intent digests, consent/profile bindings and create keys are never exposed. Generated IDs are opaque random server values and never filesystem paths.

## Ordering, failure and recovery

Begin validates the existing Chat contract before atomically persisting a running turn and incrementing revision. Only then may the composition invoke the bounded provider/tools. Finish persists completed or failed state and increments revision before acknowledging its terminal result. A failed turn does not assert that no tool effect occurred. Any uncertain persistence error poisons mutations until operator recovery/restart. An exclusive file lock held for the store owner’s lifetime rejects a second writer. Dropping that owner explicitly unlocks it, so an unrelated child’s briefly inherited descriptor cannot retain ownership across reopen. Its durable presence is also the initialization fence: a missing state file after initialization fails closed instead of recreating an empty store. On open, all persisted running turns become interrupted durably; they never auto-resume or auto-retry. An interrupted turn explicitly means execution outcome is uncertain. New turns require deliberate new user intent.

State is one versioned bounded private file under a separate current-owner 0700 directory, using the existing secure-path checks, owner-only regular files, no symlinks/hardlinks, same-directory atomic rename and file/directory fsync. Invalid/corrupt state fails closed; no empty-store fallback. Existing demo files are not migrated or reassigned. The original transcript slice has no physical purge API. [Conversation management](conversation-management.md) adds owner-scoped rename and logical deletion while retaining original private transcripts and idempotency/effect evidence. Bounded retention still rejects new work at physical capacity.

The browser validates the complete conversation and every nested Chat response before replacing accepted history or clearing an uncertain request. Malformed read-back retains the original request ID, body, model and confirmation headers, with sending and conversation switching still locked. Checking again is read-only; a later valid terminal result releases the pending request without another submission.

## Bounds and prompt projection

At most 50 visible conversations per owner (logical tombstones remain within physical limits), 100 turns per conversation, 1,000 conversations and four running turns across the store, and 32 MiB serialized state. Every commit reserves an additional 128 KiB per running turn for its worst-case JSON-escaped bounded terminal response; create and begin respect this reservation before any provider call. User message <=4 KiB; prompt preference <=2 KiB; answer <=16 KiB. History includes only the latest contiguous completed pairs, up to 12 messages/12 KiB including the new message. It stops at any incomplete/failed turn, an answer over the existing per-message bound, a profile binding that is not exactly the currently confirmed profile, or any budget boundary. It does not skip backward across an excluded turn. Old prompt preferences and consent are not replayed. Current intent alone supplies current preference and opportunity consent. Provider-specific context limits still apply and fail honestly.

## Verification

Targeted tests cover owner isolation, exact duplicate suppression and conflicting retries, optimistic revisions, reopen recovery, bounded context with profile separation, invalid state, uncertain writes and exclusive locking. HTTP/client integration proves refresh/continued history and lost-response handling. Full Harness/session, SSO, streaming/cancellation and cross-plugin receipt guarantees remain separate planned work.

`node --test scripts/client_tests/conversation_recovery.test.mjs` covers malformed terminal and historical replies, later valid read-back, and exact explicit retries using the production response validator. The existing CI browser runner includes this deterministic boundary check in its default and `conversations` suites, alongside the real browser journeys.

## Local implementation evidence

`CHAT-004` covers 13 focused storage/context tests, three real HTTP tests (protocol
and scope rejection, restart with exact Calendar replay, and read-back after an
admitted request's response connection is dropped), and six browser journeys.
Existing Chat (14 journeys) and shell (8 groups) retain compatibility evidence.
The HTTP disconnect case waits for observable admission; a connection dropped before
body admission may leave no request, and can only be retried with the same identity.
Independent review closed protocol admission, terminal-capacity reservation and
known-rejection UI recovery issues. OS power-loss, real multi-user authentication,
streaming, general plugin installation and full Harness remain outside these claims.


After a confirmed turn result, the browser refreshes server-owned history summaries
as a separate read. It updates the first-message title and empty-list notice without
constructing a canonical title locally. List reads predating an accepted detail,
selection or mutation cannot replace its projection. A failed background summary read
keeps the confirmed answer and sendability, offers ordinary list refresh, and must
not create an uncertain-write retry. `CHAT-004` browser cases bind these obligations.

## Model selection compatibility

[MODEL-001](model-selection.md) extends turn admission with closed
`chat-conversation-turn/v2` and required `model_id`. The old v1 shape and its
serialized idempotency digest remain unchanged when the field is absent. Exact
terminal replay and conflicting intent checks precede current catalog resolution;
removing a model cannot change or re-execute a historical result. A new request
resolves an available model before reserving its running turn. Concurrent turns
retain independent immutable provider selections, and response histories retain
the actual model identity.

## History management

[CONVERSATION-MANAGE-001](conversation-management.md) specifies rename and logical
delete, revision-checked idempotent metadata commands, running-turn exclusion and
menu/dialog projections. Deleted entries cannot be reopened or resurrected through
old create/turn requests. They retain their private records for duplicate suppression;
this is not secure erasure or reversal of prior product effects. Explicit names
survive first-message title generation. Management revisions interleave with the
original turn revisions; historical turn receipts remain exact.

## Automatic topic titles (CHAT-004)

New first-message titles follow `YYMMDD|topic`, for example `260921|启动日历`.
The date is the first turn's admission date in campus time (UTC+08:00), fixed by
the server and persisted before model dispatch. The topic is 1–24 Unicode scalar
values without `|`, control characters or Unicode line/paragraph separators; it
has no leading/trailing whitespace. The display shape is `^[0-9]{6}\|[^|\r\n]{1,24}$`,
with additional calendar-date and control-character validation. The two-digit
year denotes 2000–2099. Models supply only the topic; they cannot supply the date.

On first admission, the store saves a deterministic dated user-message excerpt as
a fallback. After a successful first Chat response, the application may make one
bounded title request to that turn's already-selected model, using only a bounded
copy of the first user message and a title instruction. It passes no tools,
profile, tool results, historical messages or user prompt customizations. There
is no retry, an outer five-second timeout, and no model switch. Mock mode uses
the deterministic fallback and does not claim generated model prose. Invalid,
empty, failed or timed-out title responses keep the fallback; Chat success is
preserved. First-turn Chat failure does not trigger title generation.

The title result is persisted atomically with the first terminal turn in its
existing finish revision, before its receipt is acknowledged. Pending requests
remain running during this bounded operation. Title generation is never replayed
on refresh, exact retry or restart and never runs on subsequent turns. Manual
rename owns an explicit title and always wins, including rename before the first
message. Automatic formatting applies to automatic titles only; manual names
remain under the existing rename contract. Existing conversations and existing
version-1 data without the optional title metadata retain their original names.
Persisted metadata must validate date, fallback derivation, first-turn placement,
terminal phase and management replay; malformed evidence fails closed.

Verification extends CHAT-004 with date-boundary/format tests, selected-model
no-tools wire checks, failure fallback, single-generation/exact-replay tests,
manual-name preservation, malformed metadata and reopen checks. The browser
continues to render server-owned titles through the existing history refresh.

Local smoke with the configured `hy-mt2-7b` generated the synthetic title
`260906|查看本周日历安排`; persisted metadata recorded a model-generated topic,
and exact HTTP retry returned the original result. The browser rendered that
server title. Eight existing conversation browser journeys passed with the dated
title assertion, including stale reads, reload and uncertain-request recovery.


## Personal Agent instructions

[ROOT-PROMPT-001](agent-root-prompt.md) adds an owner-scoped root-prompt setting to
this store. New turn reservation snapshots the setting atomically; changing settings
does not change in-flight requests or terminal replays. No prompt text is exposed in
conversation DTOs. Existing v1 stores read without rewriting; first settings update
writes v2, which old binaries must reject. Settings use the existing capacity and
private-file failure rules, with a separate revision for stale-update protection.
