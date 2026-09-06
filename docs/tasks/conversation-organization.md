# CONVERSATION-ORGANIZE-001

User request: dated topic-only rename, date sorting, pin/group, and model picker left
of Send. Main owns integration; backend/UI contributors have disjoint paths.
Contract: [conversation organization](../contracts/conversation-organization.md).
No new identity, tool authority, external service or remote operation is introduced.

| Slice | Status |
|---|---|
| ConversationStore metadata, stable order and v2 commands | implemented |
| History menu, dated rename and groups | implemented |
| Same-row compact model picker | implemented |
| HTTP/browser evidence and independent review | implemented (bounded) |

Evidence binds to CHAT-004 and CONVERSATION-MANAGE-001; production users remain
outside current loopback composition evidence.


## Evidence (2026-09-06)

- Workspace: 1335 Rust tests pass; all-target/all-feature clippy, fmt and repository
  contracts pass. 40 conversation-store tests include v1 compatibility, legacy
  empty-title pin/first-turn/restart, exact v2 receipts, stale/running rejection,
  group/pin isolation and stable date ordering. Real HTTP v2 restart path passes.
- Browser: 6 organization cases and 1 composer geometry group pass; 9 existing
  management cases pass after adapting suffix-only/v2 assertions. Geometry covers
  1440/768/390/320 widths and long model labels. Screenshots reviewed on desktop/mobile.
- Independent review closed legacy replay recovery and current-list ordering issues.
- Current local preview was updated with its model environment retained; seven
  existing conversations remained readable. A private pre-update state/binary backup
  was retained outside Git. No hosted model quality or production identity claim.

Run `UCA_BROWSER_SUITE=organization node scripts/test_usable_enhancements_browser.mjs --base http://127.0.0.1:PORT`
and the existing `UCA_BROWSER_SUITE=management` suite against an isolated daemon.
