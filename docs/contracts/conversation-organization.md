# Conversation organization (CONVERSATION-ORGANIZE-001)

User approved: suffix-only dated renaming, date ordering, pinning and single-level
groups. Owner: existing M30 ConversationStore; application and Web remain projections.
Read with conversation-management.md; no new store, identity or effect authority.
Main owns integration; backend/UI contributors have disjoint paths and independent review.

## Commands and projection

Use the existing /api/v1/agent/conversations/{id}/manage endpoint and admission.
Add closed schema `chat-conversation-manage/v2` with the existing request_id and
expected_revision, and action one of:
- `{kind:"rename",title}`: topic only, trim/nonblank, <=192 UTF-8 bytes, no control
  or Unicode format characters, and no `|`. Preserve server date; emit YYMMDD|topic.
- `{kind:"pin",pinned:boolean}`: set or clear pin.
- `{kind:"group",group:string|null}`: assign a trimmed nonblank group <=64 UTF-8
  bytes, without controls/format characters; null removes assignment.
- `{kind:"delete"}`: existing logical deletion.

v2 result schema `chat-conversation-manage-result/v2` adds `organization` exactly
`{date:string|null,pinned:boolean,group:string|null}` to existing result fields.
Read summary/detail keep their existing schemas with additive `organization` of this
shape. New clients validate it before use. v1 stored receipts and exact replay remain
unchanged; legacy v1 rename semantics are compatibility-only, the UI always sends v2.
Metadata is server-owned and included in request-result consistency checks.

New conversations retain an immutable campus date at creation. Existing conversations
recover their date from existing valid dated-title/first-turn evidence; an old undated
conversation has date=null and sorts after known dates. Its first v2 metadata write
may record today's campus date when no historic date exists; never invent an old date.
The effective date is then stable across later rename, pin, group and turns. Automatic
titles for new conversations use the retained date. Existing v1 files/receipts read
without rewriting or resetting private data; persist new metadata through the same
atomic owner, with explicit schema/default compatibility and full state validation.

Mutations share revision, owner, running-turn rejection, exact retry and poisoning
semantics with existing management. The 128 receipt limit reserves the last slot for
Delete across all non-delete actions. Removing a conversation does not change other
conversations' pins/groups. Groups are names assigned to conversations; empty groups
have no stored object, hierarchy, global rename/delete or cross-user visibility.

## Ordering and UI

Server list order: pinned first, effective date descending, stable original creation
order for same-day ties (newer creation first). Activity and rename must not bump dates.
Client preserves this rule when reconciling detail and list updates. Sidebar shows
Pinned first, then ordinary groups and ungrouped conversations; within each section
use server date order. Pinned entries appear only in Pinned, retain their group and
return there when unpinned. Group section order is deterministic by name.

Context menu supports rename, pin/unpin, move to group/remove group, delete. Rename
shows a read-only date prefix and editable topic; no full-title rewrite. Group dialog
offers existing names and a new name. Keep explicit rejection, dirty draft and unknown
outcome protections from conversation management. No optimistic mutation acknowledgement.

Model selection is a compact control immediately left of Send, on the same baseline
at desktop and mobile widths, with text truncation for long model names. It must not
wrap above Send or occupy a separate toolbar row. Preserve accessible labels/focus,
44px hit targets and request snapshot semantics.

## Proof

Bound to CHAT-004 and CONVERSATION-MANAGE-001 gates: valid prefix preservation,
rename delimiter rejection, pin/group owner isolation, stable date sorting across
activity, restart/replay and v1 compatibility; stale/running/unknown-result behavior;
real HTTP and browser rename/pin/group/ungroup plus narrow/long-model layout checks.
