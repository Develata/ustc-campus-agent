# Conversation rename and logical deletion

Case: CONVERSATION-MANAGE-001. Owner: existing M30 ConversationStore and its application service. This user-authorized extension changes conversation metadata/lifecycle within the existing owner; it adds no identity, provider, plugin or effects authority.

## Commands and wire shape

POST /api/v1/agent/conversations/{id}/manage requires the existing protocol-major 1, JSON/body bound, owner context and loopback Host/Origin admission. Closed intent:
`{schema:"chat-conversation-manage/v1",request_id,expected_revision,action:{kind:"rename",title}}`
or action `{kind:"delete"}`. IDs and revisions use the existing conversation rules. The submitted title is <=192 UTF-8 bytes and contains no control characters; after trimming it must remain nonblank; server and client reject invalid values rather than silently truncating. The original submitted title participates in exact-intent comparison even when the display title is trimmed.

Success: `{schema:"chat-conversation-manage-result/v1",conversation_id,request_id,revision,title,deleted}`. Typed errors reuse the conversation error envelope/codes. The server alone owns the new title, revision and deletion marker; the browser never acknowledges a mutation based only on optimistic local state.

## State, retries and isolation

Rename increments revision once and marks an explicit title, preserving it across future turns including an empty conversation's first message. Delete increments revision once and terminally marks the conversation deleted. Both reject a currently running turn and stale expected revisions. Exact successful management retry returns the original receipt before current-state checks; changed content under that management ID conflicts. Management IDs cannot alias a turn ID in the same conversation; future turns cannot reuse a management ID.

Deleted conversations are omitted from lists and cannot be opened, used for context, queried for activity, or submitted to. Replaying an old create/turn intent cannot resurrect them or re-execute effects. Logical deletion retains the original private transcript, request/effect evidence and bounded management receipts in the existing store; it does not purge disk data or undo Calendar/plugin effects. An exact historical management receipt may still replay but cannot restore a deleted view. The UI confirmation states that deletion removes the conversation from history and preserves server-side records. No restore/purge/bulk-delete API is added.

At most 128 successful management receipts per conversation, with the final slot reserved for Delete (at most 127 Rename receipts); the 50-per-owner visible limit excludes deleted entries, while the 1000-total and 32 MiB physical retention bounds still include them. No write is acknowledged until the existing atomic persistence owner commits it; uncertain save failure poisons the writer. Revision validation must cover interleaved turns and management events, preserve exact historical turn result revisions, reject malformed/contradictory persisted management state, and forbid deletion during a running turn.

Existing version-1 stores open through default-empty management metadata and remain unchanged when unused. After management is used, older binaries that do not understand the added fields are not a supported downgrade reader; restore a prior operator backup rather than discarding new fields. No existing data is reset or reassigned.

## Interface behavior

Each history row exposes a named menu button and supports right-click and keyboard context-menu activation. Menu actions are Rename and Delete, with arrow navigation, Escape/dismiss/focus return and mobile reachable controls. Rename uses an accessible prefilled dialog; Delete uses explicit confirmation. Text is inserted as text, never HTML. Cancellation leaves server state untouched.

Menus and new mutations are unavailable during a busy or unresolved Chat/management request. Unknown management outcome keeps the exact original body/request ID for an explicit retry, blocks conflicting writes and explains uncertainty. Known rejection (including the exact typed capacity 429) keeps drafts, releases management uncertainty and refreshes current server state as needed. A conversation deleted in another client can make turn submission/result lookup return confirmed NotFound; the UI offers a way to leave the inaccessible conversation while preserving the draft and never automatically resending or claiming earlier effects did not occur. No automatic new-ID retry. Renaming updates the list and current metadata without clearing the draft. Deleting the current conversation shows a new empty draft view after server confirmation; deleting another conversation preserves the current draft/history. Old list responses cannot resurrect deleted rows or roll back accepted titles.

## Verification

Targeted store/restart and real HTTP checks for owner isolation, validation, optimistic conflicts, exact retry, cross-action ID conflicts, explicit-title preservation, running rejection, tombstone visibility/resurrection denial and malformed persistence. Browser exercises right-click/menu button/keyboard, dialog cancel/rename/delete, current versus other conversation, reload, stale and unknown responses, mobile placement/focus and existing conversations/models regression.
