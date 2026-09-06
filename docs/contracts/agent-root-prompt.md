# Personal Agent root prompt (ROOT-PROMPT-001)

User-approved setting: a persistent personal instruction used at the start of each
new saved-conversation turn. M30 ConversationStore owns it per admitted tenant/user;
ConversationApplication exposes commands and queries. Settings is a thin editor.
This extends the existing conversation and prompt-projection contracts; it does not
change product topology, tools, grants, confirmation rules or production identity.

## State and commands

- GET /api/v1/agent/root-prompt returns exactly schema `agent-root-prompt/v1`,
  revision (u64) and text (string); absence is revision 0 and empty text.
- PUT the same route accepts the closed schema `agent-root-prompt-update/v1`,
  expected_revision (u64) and text. Both routes require client protocol major 1;
  PUT also uses existing JSON/body/loopback/same-origin admission.
- Text is at most 8192 UTF-8 bytes before trim. Empty/whitespace clears the setting.
  The existing prompt control/format-character exclusions apply. Multiline text is
  supported. Only the current admitted owner can read or edit; no body/header owner.
- Compare-and-set prevents stale overwrites. An exact repeat of the immediately
  preceding successful update (same text and prior revision) returns current state;
  otherwise a stale revision conflicts. Revision overflow/capacity reject pre-write.
- Use existing conversation typed errors: invalid intent 400, revision conflict 409,
  capacity 429 and unavailable 503. Persistence failure does not claim a save.

Use one private conversation state file, existing atomic persistence and lock. Keep
at most 1000 owner prompt records under the existing 32 MiB/reserved-turn capacity.
Read existing v1 without rewriting; first settings update writes v2. Retain cleared
records/revisions for conflict protection. Older binaries reject v2; do not overwrite
private state on rollback. Settings are not localStorage, a tool, or public history.

## Prompt projection

New turn admission snapshots the owner's current text under the same store lock as
turn reservation. The server alone fills an internal request field that clients cannot
set. Immutable platform system policy is first; a labelled personal-instruction user
message comes next, then the existing request-only preference and conversation history.
The personal instruction can set role, task approach and response habits; it does not
become system authority or grant tool access. Count it in normal provider preflight.
It is not copied into visible transcript, automatic-title prompts, tool traces or logs.
Settings changes affect subsequent new turns in existing and new conversations, never
in-flight requests or terminal replay. Replays do not invoke the provider again.
The legacy stateless Chat endpoint retains its request-only behavior.

## Settings experience and proof

Show an editable Agent 根提示词 textarea, byte count, save, restore-default and reload.
Load server state before editing. Saving or restoring uses its revision. Keep unsaved
text on known rejection; on unknown result retain the draft and read server state,
report saved only when matching state is observed. Never blindly overwrite a conflict.
Ordinary refresh/focus cannot overwrite a dirty draft. Existing per-turn preferences
remain independent. Mock accepts settings but does not model their language behavior.

Bind to the existing CHAT-004 Rust and usability gates: owner isolation, save/clear,
restart, v1 compatibility, invalid text, stale writes/exact repeat, frozen new-turn
projection, unchanged replay and controlled-provider receipt; real browser settings
save/reload/reset and unknown-result preservation. Main owns integration and docs;
backend and UI contributors own separate files, followed by independent review.
