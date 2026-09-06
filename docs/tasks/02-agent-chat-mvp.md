# Bounded Web Chat MVP delivery taskbook

## Metadata

- `Status`: Historical slice tracking with dated follow-ups; no current release or full-platform verdict
- `Date`: 2026-09-03
- `Scope`: one loopback Web Chat vertical slice over the three existing DemoReviewed/synthetic Plugin journeys
- `Normative contract`: [`../contracts/agent-chat.md`](../contracts/agent-chat.md)
- `Acceptance`: active `CHAT-001`, `CHAT-002`, `CHAT-003`
- `Decision provenance`: Develata selected “Web Chat + OpenAI-compatible provider + three-Plugin tool calling”; real campus-source and real-provider network access remain forbidden without separate approval

This taskbook retains each dated slice's scope and evidence. Earlier non-goals
describe that slice, not the current application. Current behavior is specified by
[Agent Chat](../contracts/agent-chat.md), [saved conversations](../contracts/chat-conversations.md)
and the [MCP/Skill application profile](../contracts/plugin-management.md);
[this demo guide](../guides/competition-demo.md) describes the runnable path.
The taskbook does not redefine their authority or promote acceptance status.

## 1. Initial delivery slice (2026-09-03)

```text
Web Chat
→ POST /api/v1/agent/chat
→ deterministic mock by default or operator-configured OpenAI-compatible provider
→ finite sequential tool loop
→ Affairs / ChangeRadar / Opportunity Graph existing product compositions
→ final answer + safe tool trace
→ loopback Docker Compose ZIP
```

That initial slice retained the existing fixture inputs and introduced no campus
retrieval, USTC source activation, CAS/SSO, production database, generic Plugin
runtime, streaming, RAG, durable chat history or multi-agent graph. Saved dialogue
and the bounded package runtime were separate later slices.

## 2. Implementation ownership

- `chat_provider.rs`: provider profile parsing, key-file loading, HTTP mapping and provider DTOs only.
- `agent_chat.rs`: closed request validation, finite loop budgets, provider/tool ordering and safe response projection.
- `chat_tools.rs`: exact model-visible names, schemas and mapping to pre-existing product operations; no product truth or grant ownership.
- `web.rs`: HTTP composition and stable error/status projection.
- `src/web/`: thin presentation and explicit Opportunity per-request confirmation.
- `deploy/mvp-compose/` and the package script: loopback deployment, persistent state, reset, cross-platform launchers and deterministic archives.
- `ustc-agentd` remains the single shared composition/fan-in owner.

`Cargo.lock`, shared module declarations, routes, Compose, acceptance/status projections and candidate identity are serialized through the integration owner. Partial worker output is not accepted without exact diff, compilation/tests and review.

## 3. Delivery gates

### Approved Chat-first presentation slice (2026-09-05)

- Base: `2f03a13c3ec03eab791f1728e2c2973ed82c3e50`; local branch `codex/chat-plugins-shell`.
- Develata explicitly approved Main to implement and independently refine a conversation-first shell with a Plugins entry. This is the existing static composition surface, not a new M80/Dioxus implementation or acceptance promotion.
- Owner: Main; reviewer: independent read-only Codex lane. Cases: `CHAT-001`, `UE-001`, `UE-002`, `UE-003`.
- Paths: `apps/ustc-agentd/src/web/`, embedded asset wiring in `web.rs`, corresponding browser/tests and these contract/feature/acceptance projections.
- Non-goals: #72 password login, backend/domain changes, new tools, durable conversation history, source activation, archive replacement, push/merge/publication.
- Design: neutral white canvas, narrow pale rail, centered new-chat composer, readable message flow with bottom composer, Plugins directory and individual detail views, settings with collapsed operator controls; mobile drawer and system/light/dark appearance.
- Verification: compiled asset tests, existing Chat and usable-enhancement browser journeys updated for actual navigation, new navigation/Markdown safety checks, desktop/mobile screenshots, contracts and Rust baseline checks. Report actual results and any unrun gates at handoff.
- Local results: Rust fmt, workspace/all-target/all-feature clippy, full `cargo test --locked --all-features`, 18 editor/checklist unit tests, 14 Chat browser journeys and 13 usable-enhancement/Shell browser groups passed. Browser coverage includes actual Back/Forward, pointer navigation, same-route mobile dismissal, focus trap, exact Markdown copying and in-flight response preservation. Real Chromium was driven through the repository's CDP harness because Browser plugin tools were unavailable.
- Validation environment: compiled mock server on WSL Debian, Windows Chromium for the Web client. Local-provider tests require direct loopback (remove inherited proxy variables for that test process). Windows Git newline conversion was corrected to Git LF bytes only in the owned development worktree; the original delivery archives were not edited. The 21 affected checker/sabotage tests passed in a POSIX source snapshot with the declared Git modes.
- Projection review: only the Chat/UE acceptance descriptions and their coverage explanation changed in the two whole-file M60-pinned documents. Their SHA pins and the owning guard's AST registry pin were refreshed after independent review; all M60 semantics and executable pins remain intact.
- Independent review: two accepted findings (same-route mobile dismissal and original-answer copying) were fixed and covered by real-browser regressions; no accepted blocker remains. Native Android/WebView, soft keyboard, real-model network, new APK/ZIP, remote CI and release acceptance are not claimed by this presentation slice.

### Local functional hardening slice (2026-09-05)

- Develata authorized frontend/backend optimization and parallel subagents; Main owns integration and documentation, with bounded implementer lanes and independent read-only review.
- Cases: `CHAT-001` and `CHAT-002`; owning boundary remains the app-private finite Chat coordinator and its static Web projection.
- Core slice: enforce one matching Calendar mutation attempt per admitted run before executor entry; preserve complete-batch shape checks, per-call trace, read-only operations and denial semantics. Code and tests belong to `apps/ustc-agentd/src/agent_chat.rs`; no public DTO, durable schema or authorization expansion.
- Verification: regression must reproduce duplicate execution before the fix, then prove same-batch/cross-turn suppression, failure consumption, mismatch handling and read-only behavior; integrate with configured-provider route tests and existing browser journeys.
- Non-goals: durable retry protocol, saved chat history, generic Plugin lifecycle, new model configuration authority, source activation, remote publication, APK/package replacement.
- Provider slice: preserve explicit personal-calendar listing in a mixed academic-calendar request (`chat_provider.rs` and existing Chat browser binding).
- Web slice: preserve an unanswered question beside a newer draft, mark its non-success, and reset sendable history at an oversized-answer boundary with a visible notice (`web/app.js` and Shell browser regression cases). Calendar mutation errors explain read-back before explicit retry.
- Local verification: fmt, workspace/all-target/all-feature clippy, full Rust tests including doctests (1117 passed, zero failed/ignored), 18 editor/checklist tests, 14 Chat browser journeys and 15 UE/Shell browser groups passed. The Chat four-tool request now uses natural academic-calendar wording. The configured-provider HTTP regression proves one durable Calendar item after repeated proposals and reopening the same state.
- Test reliability: a pre-existing synthetic HTTP peer could block writing an oversized response after the adapter rejected it; the peer now has a five-second write timeout without weakening transport assertions. The initial stalled run was stopped and is not counted as a pass; the focused transport regression and subsequent full run passed.
- Review: an implementer-independent read-only lane found no blocker, including an incremental review of the test-peer bound. Main inspected actual code, output, desktop failure/context screenshots and mobile layout. Browser plugin was unavailable; the existing Chromium/CDP bindings were used.
- Status: implemented with local evidence; no new real-model, Windows Docker, Android, package, remote CI or release acceptance is claimed. Changes remain local and uncommitted.

### Agent capability presentation and local connection slice (2026-09-05)

- Develata requested reading docs before further changes, an updated administrator UI, Agent-oriented Plugin presentation, and use of the supplied loopback model for local testing. Local service metadata reports a 2048 context limit with no tools support; select the explicit `local-chat` profile and label the limitation instead of misrepresenting Agent readiness.
- Ownership: Main owns contract, composition and integration; bounded provider/core and presentation workers own disjoint files; fresh read-only review follows. Cases: `CHAT-001/002`, `UE-003`, and existing administrator publication tests.
- Provider scope: a numeric-loopback-only local test profile with no tools, conservative small-context budgeting and no proxy/redirect; unchanged normal HTTPS Agent profile. Expose only safe configuration status to the page.
- Presentation scope: Plugins describe Agent capabilities and direct the primary user action to Chat; detailed sources/profile consent remain available. Settings group administrator operations, confirmation, human-readable result state and optional technical evidence. Preserve existing IDs, fixed request envelopes and backend authority.
- Non-goals: generic install/enable controls, new campus sources, provider settings/key editing in the browser, durable chat, streaming, full HarnessRun, production administration or remote publication.
- Status: implemented with local evidence: 1057 Rust target tests plus 66 doc tests, 18 JS tests, 14 Chat browser journeys, 17 usability/shell/administrator journeys, and 5 real local-chat browser checks. Format, clippy and contract checks passed. Read-only reviews closed accepted profile-control and locked-draft-focus findings; full Chat regression also caught and closed unintended storage writes during status rendering. These are native WSL/Windows Chromium results, not Docker, Android, remote CI or release acceptance.
- MCP/Skills follow-up requested by Develata: Plugins must configure these components as well as describe capabilities. At this batch's close, MCP/Skills settings remained planned. The proposed follow-up was exact-package configuration command/query plus restart-safe storage, followed by bounded Skill projection and reviewed MCP discovery/execution; its later implemented scope is recorded in `plugin-management.md`. Develata selected the existing package-owned path; independent component import is outside scope. Configuration must not become a parallel grant or tool-registration authority.

### Cohesive administrator presentation slice (2026-09-05)

- Develata requires high cohesion and low coupling. This internal M80 presentation slice binds `UE-003` and existing fixed-publication cases; it does not change wire or privilege contracts.
- Main owns `web/admin-controls.js`, its composition in `app.js`/`web.rs`, and integration checks; an independent reviewer owns isolated boundary tests and review.
- The administrator controller owns only its subtree, confirmation/pending presentation, fixed requests, status and receipts. Its document-lifetime `mount` accepts an explicit root, request transport and a change-publication notification. Repeat mounting of the same root does not add listeners or issue duplicate bootstrap requests.
- Chat, private profiles, provider state and feed internals are not dependencies. The composition root supplies the public-feed refresh callback; refresh failure cannot reclassify an acknowledged publication as failure.
- Validation: controller runs against controlled DOM/HTTP counterparts without Chat/Profile globals, plus real browser publication and shell journeys. Backend permission/idempotency rules remain unchanged.
- Status: implemented; 6 isolated controller tests and 17 real-browser usability/shell/administrator journeys passed, along with the full Rust/JS baseline. Independent review found no blocker. The separately approved next work follows existing exact-package installation/configuration/grant/enable semantics for MCP/Skills; independent component import is out of scope.

A candidate is deliverable only after all applicable `CHAT-*` bindings pass on one exact semantic head:

1. Rust format, clippy, unit/integration tests and doc tests;
2. deterministic mock direct answer plus all three sequential product tool paths;
3. Opportunity absent/unconfirmed/current/missing-profile cases;
4. OpenAI-compatible protocol/error/role/limit cassette tests without real-provider egress;
5. browser request/DOM/static behavior checks;
6. isolated Compose clean start, restart/stop persistence and explicit reset deletion;
7. relative-output packaging, archive/checksum/provenance/secret scans;
8. Windows PowerShell 5.1 parser and launcher byte/exit-code checks;
9. exact-head independent code/security review;
10. GitHub CI/governance and remote-head read-back.

A separately approved real-provider smoke may be reported in addition. Without that approval it remains `not-run` and does not block the deterministic MVP.

## 4. Fan-in sequence

1. freeze or amend the normative contract and acceptance rows;
2. implement provider, loop/tool bridge and Web projections behind one fan-in owner;
3. run local static and no-network mock tests;
4. use exact-head hosted CI for Rust, Docker Compose and Windows evidence unavailable locally;
5. remediate review findings on the product branch, invalidating prior receipts after every semantic edit;
6. package only the final reviewed head and verify the downloaded artifact independently;
7. merge or release only under separate authority.

The approved package-owned MCP/Skills work is tracked separately in
[`m20-package-component-configuration.md`](m20-package-component-configuration.md);
it must not grow additional Market lifecycle authority in the Chat shell.

## Saved dialogue functional follow-up (2026-09-05)

User direction prioritizes campus Agent usage, Market and the official-information /
Calendar / course journeys. Account entry is SSO or background-provisioned users;
self-registration and invitations are excluded. Main owns `CHAT-004` integration,
with separate storage and browser implementers and a read-only reviewer. The owning
[conversation contract](../contracts/chat-conversations.md) records the bounded
server-owned transcript, submission reservation/replay and interrupted recovery.
The shell now creates/selects saved conversations and submits one new message;
old `/agent/chat` remains compatible for existing clients. Local evidence: 36 finite
Chat regressions, 13 conversation tests, three HTTP tests, 14 Chat browser journeys,
eight shell cases and six conversation cases. No original archive, credentials or
old private records were migrated. The local preview retains the configured small
local-chat model; controlled mock exercises actual tool paths. Complete Harness,
streaming, SSO service and generic plugin execution remain planned.


## Observable execution follow-up (2026-09-05)

`CHAT-ACTIVITY-001 / CHAT-005` adapts actual local DSH tool activity separation in
Rust and lightweight browser code. Main owns the application/HTTP integration;
separate backend/UI authors and cross-surface read-only review cover the slice.
Touched scope: finite Chat observer, saved-dialogue application query, app-private
HTTP activity route, browser activity controller and corresponding projections.
The implementation has no model token stream, cancellation or new plugin authority.

Local evidence: nine activity Rust cases (including a delayed controlled-provider
HTTP test), six activity browser cases (including real conversation submission with
a delayed reply), format and clippy. Review caught terminal trace shrinkage being
rejected by the browser and an overstated provider-I/O boundary; both were corrected
and covered by the targeted cases. A status read cannot invoke or repeat Calendar
writes. This is local source evidence; no archive rebuild, remote merge or release.


The history follow-up refreshes server-owned summary titles after a confirmed answer,
clears the obsolete empty-list hint and rejects stale list responses. Summary read
failures do not change a saved answer into an uncertain write. Eight focused conversation
browser cases passed, including the two added summary-refresh cases; independent review
found no blocker. No transcript migration or new endpoint was needed.
