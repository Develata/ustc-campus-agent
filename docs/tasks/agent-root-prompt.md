# ROOT-PROMPT-001 — Personal Agent settings

- User request: users can add their Agent root prompt in Settings.
- Owner: Main; backend and UI contributors have disjoint ownership; independent review follows.
- Authority: [root prompt](../contracts/agent-root-prompt.md), M30 prompt projection,
  existing conversation persistence and CHAT-004 acceptance gate.
- Scope: owner setting/read-update ports, new-turn snapshot, settings editor, targeted tests.
- No changes to tools, grants, production identity, legacy stateless calls or remote publication.

| Slice | Status |
|---|---|
| Contract and authority projections | implemented |
| Durable owner setting and prompt assembly | implemented |
| Settings editor and recovery | implemented |
| HTTP/provider/browser proof and review | implemented (bounded) |


## Verification (2026-09-06)

- Workspace: 1327 Rust tests pass, including owner isolation, v1/v2 persistence,
  settings conflicts and unchanged terminal replay. All-target/all-feature clippy,
  fmt and repository contracts pass.
- Controlled HTTP provider confirms personal instruction precedes per-turn preference,
  reaches the actual model request, stays out of automatic titles/public metadata,
  and remains frozen when settings change before provider dispatch.
- Real browser: 6 settings scenarios pass: save/readback, dirty reload/focus,
  lost write/double click, conflicting clients, UTF-8 limit/mobile, restore default.
- Independent implementation/UI review has no unresolved blocker.
- These checks use synthetic settings and controlled peers, not hosted model quality
  or production multi-user authentication evidence.

Browser: `UCA_BROWSER_SUITE=root-prompt node scripts/test_usable_enhancements_browser.mjs --base http://127.0.0.1:PORT`.
Use an isolated daemon; the suite saves and restores that test owner's setting.
