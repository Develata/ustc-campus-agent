# C1 Calendar proposal implementation

- Direction approved by user on 2026-09-06; local feature branch codex/calendar-proposals.
- Contract: [CALENDAR-PROPOSAL-001](../contracts/calendar-proposals.md).
- Parent owner: Main. Reviewer: independent bounded lane after integration.
- Scope: simple-calendar domain/store, agentd application/tool/HTTP/UI projections,
  focused checks and the corresponding current capability docs.
- Non-goals: reminders, production users/migration, school SSO, external calendar I/O.

| Slice | Owner | Status | Output |
|---|---|---|---|
| C1a | Calendar contributor | implemented | One atomic proposal/item/receipt owner; core tests |
| C1b | Main | implemented | Application API, Agent proposals, browser confirmation |
| C1c | Main + reviewer | implemented (bounded) | Real HTTP/browser proof and bounded review |
| X1 compatibility | MCP contributor | implemented | String length and numeric bounds; unchanged old schema encoding |
| S1 sources | source reviewer | implemented (bounded) | Ranking/enrollment/sealing intents reuse the same reviewed historical source |

No remote publication grant is inferred from the previous merged PR's authorization.

## Evidence (2026-09-06)

- Calendar core: 17 tests; HTTP: 2 tests, including a controlled OpenAI-compatible
  peer that proposes without applying, followed by separate user confirmation.
- Browser: 6 Calendar cases pass (date preview, lost response/double click,
  update, mobile layout, cancellation, deletion). Existing Chat 14 journeys pass
  against a separate fresh mock daemon.
- MCP: protocol 10 tests and 3 scalar-constraint integration tests pass. Independent
  review caught oversized JSON number thresholds rounding; the compiler now rejects
  thresholds outside the documented exact range before readiness.
- Reviewed source routing: targeted transcript/ranking/enrollment/sealing cases pass;
  no new live source, iCourse permission, model quality or hosted provider claim.
- Independent Calendar core/API and UI reviews have no unresolved blocker.
- Workspace Rust tests, doctests, fmt, all-target/all-feature clippy and repository
  contract checks pass. The conversation store now releases its lock when its owner
  drops; a deterministic inherited-descriptor regression protects restart while
  keeping active-writer exclusion. The OpenAI test peer consumes the full request
  before closing its socket. Browser screenshots are local verification artifacts.

Reproduce the browser cases with a fresh loopback daemon and
`UCA_BROWSER_SUITE=calendar node scripts/test_usable_enhancements_browser.mjs --base http://127.0.0.1:PORT`.
Set `CHROME_BIN` when Chrome is outside the runner's Linux defaults.
