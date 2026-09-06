# Calendar proposals (CALENDAR-PROPOSAL-001)

- Status: implemented for the loopback owner-local profile; production ownership and reminders remain planned.
- Owner: Simple Calendar owns item mutations, proposal state and effect receipts;
  the agentd application port supplies the admitted local subject and server clock.
- Parent: [multi-user task C1](../tasks/multi-user-campus-agent.md),
  [Agent Chat](agent-chat.md). This slice retains the loopback owner-local profile;
  production identity, owner migration, reminders and delivery are not claimed.

## Authority and flow

Chat may propose create, update or delete. A proposal is not an executed item.
Only a separate explicit browser confirmation can commit it. The model cannot
call confirm or cancel. Legacy exact `记录事项：` and `删除事项 ID` commands retain
their existing explicit-intent behavior; natural-language date/change requests use
proposals. Browser and HTTP adapters do not calculate dates or mutate repositories.

The server persists the exact title, RFC3339 scheduled time (explicit UTC offset),
operation, target item snapshot, base item revision, creation/expiry times and
admitted subject with a proposal ID. Campus default time is UTC+08:00; the model
receives current campus time, and ambiguous dates must be clarified before proposing.
The UI displays the full absolute date/time and offset before confirmation. A dated
item is not a reminder. Update supplies the complete replacement title/time; null
scheduled time explicitly removes the date. Delete displays the existing item.

## State and persistence

Pending -> applied or cancelled. Pending proposals expire after 30 minutes.
Confirm requires the exact proposal ID, current admitted subject and unchanged
item revision. A stale proposal cannot silently apply to changed data; make a new
proposal. Applied retries return the saved item receipt without another effect,
including after restart. Cancel retries remain cancelled; confirming a cancelled
or expired proposal rejects. Confirmation and the item change share one atomic
Calendar store commit; no separate receipt file or second item authority is added.
Uncertain durability is reported honestly and blocks further mutation until exact
read-back and synchronization succeed. Known pre-write failure preserves old state.

The existing local file remains the sole item owner. Existing v1 files are read
without rewriting; the first successful new mutation writes a versioned v2 store
with revision/proposals. Older binaries must reject v2 rather than lose fields.
Legacy records are not relabelled as production-user data. Per-proposal subject
binding prevents a confirmation from crossing the current admitted subject; it
is not a production multi-user Calendar storage claim.

The v2 file is at most 1 MiB, preserving the original 64 KiB item projection;
at most 128 proposals are retained, with 4,096 bytes reserved per pending receipt.
Terminal receipts are not evicted.
Capacity rejection must occur before effects. Proposal creation reserves enough
space for its largest terminal receipt so cancellation/confirmation does not fail
merely because other pending proposals consumed the record budget.

## Application/HTTP projection

- GET /api/v1/calendar/proposals: current-subject proposal list and server clock.
- POST /api/v1/calendar/proposals: explicit structured user draft or Agent proposal;
  request_id plus exact operation; repeated key with different intent conflicts.
- POST /api/v1/calendar/proposals/{id}/confirm: exact explicit confirmation.
- POST /api/v1/calendar/proposals/{id}/cancel: abandon the pending proposal.

All writes require JSON, existing same-origin/loopback admission and closed versioned
DTOs. Subject and server time are never accepted from the browser/model. Confirmation
contains only schema and proposal ID from the route: mutable effect fields reject.
Errors distinguish invalid request, not found, conflict, expired, capacity and
unavailable; no error is presented as an executed calendar action. UI unknown-result
recovery reads server state and retries only the same ID, never creates a new proposal.

## Verification

Core: v1 read/no rewrite; pending has no item effect; exact confirmation/retry across
restart; foreign subject and stale revision rejection; update/delete; cancellation,
expiry, capacity and uncertain persistence. Adapter: forged mutable confirm body and
cross-origin rejection; real Chat tool produces pending only; HTTP confirm read-back.
Browser: absolute time preview, confirm/cancel, refresh recovery, duplicate click,
conflict guidance and no reminder-success claim. Bound this slice to CHAT-003/004
existing Calendar integration checks plus a targeted proposal suite before completion.
