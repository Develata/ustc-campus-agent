# MVP core capabilities

> Status: implemented loopback MVP · Last review: 2026-09-04

## 1. Design

The MVP follows four rules:

1. **Server authority.** The browser and model may propose; Rust validates every request, tool name, argument and permission boundary.
2. **Small complete paths.** A normal answer and each tool call have an executable end-to-end path, not only a manifest or UI mock.
3. **Source-labelled output.** Official/synthetic facts, community signals and private profile state remain distinguishable. Community feedback affects only soft ranking.
4. **Honest degradation.** Missing consent, stale source, provider failure and unsupported features return explicit non-success outcomes instead of plausible text.

## 2. User-visible flow

```text
Browser (loopback only)
  → POST /api/v1/agent/chat
  → bounded ChatRun
  → deterministic mock or OpenAI-compatible provider
  → exact Rust tool catalogue
       affairs_navigator_get
       change_radar_get
       simple_calendar_items
       opportunity_graph_plan_current_profile (only with per-request consent)
  → typed tool results marked untrusted
  → concise answer + redacted tool trace
```

The default mock mode needs no API key and is intended for deterministic judging and offline acceptance. The real-provider mode uses the same tool definitions and Rust executor; the key remains file-backed and server-side.

## 3. Capability matrix

### Normal Agent Q&A

- Multi-turn page-local conversation with bounded history.
- Deterministic offline response or explicitly configured OpenAI-compatible Chat Completions provider.
- Maximum 3 provider turns, 4 tool calls, 4 KiB arguments per call, 64 KiB result per call and 16 KiB final answer.
- Redacted trace exposes only call order, tool name and `succeeded | denied | failed`.

Contract: [`../contracts/agent-chat.md`](../contracts/agent-chat.md)

### Affairs Navigator — procedure lookup

- Market package: [`../../market/packages/ustc.affairs-navigator/package.json`](../../market/packages/ustc.affairs-navigator/package.json)
- Agent tool: `affairs_navigator_get`
- MVP query: reviewed public transcript-certificate procedure.
- The model cannot choose arbitrary routes, publish procedures or manufacture source freshness.

Detailed design: [`../plan/06-first-party-plugins.md`](../plan/06-first-party-plugins.md)

### Opportunity Graph — course recommendation

- Market package: [`../../market/packages/ustc.opportunity-graph/package.json`](../../market/packages/ustc.opportunity-graph/package.json)
- Agent tool: `opportunity_graph_plan_current_profile`
- Inputs: an existing owner profile plus explicit confirmation on this chat request.
- Hard filters: prerequisite, availability, unresolved identity, timetable conflict and credit/requirement bounds.
- Soft ranking: user preference plus non-stale community signals.
- Output: up to three deterministic candidates, hard-constraint status, rationale, fact-level provenance and `community_evidence` link-outs.

The reproducible MVP fixture is [`../../market/fixtures/course-planning/minimal-v0.json`](../../market/fixtures/course-planning/minimal-v0.json). It contains a synthetic catalog/profile and two title-matched public aggregate-rating snapshots retrieved from USTC iCourse on 2026-09-03 UTC:

- [Real Analysis (iCourse course 2059)](https://icourse.club/course/2059/): aggregate score mapped to the synthetic `MATH2001` signal.
- [Probability Theory (iCourse course 3839)](https://icourse.club/course/3839/): aggregate score mapped to the synthetic `MATH2003` signal.

This is deliberately **orientation-level evidence**. Review text is not copied or cached; teacher, term and offering identity must be checked on the linked page. Community evidence never overrides official hard constraints. The current MVP is not a live iCourse crawler and does not claim current USTC course availability.

Planner implementation: [`../../crates/course-planning/src/lib.rs`](../../crates/course-planning/src/lib.rs)

### Change Radar — third campus-data tool

- Market package: [`../../market/packages/ustc.change-radar/package.json`](../../market/packages/ustc.change-radar/package.json)
- Agent tool: `change_radar_get`
- Queries the fixed reviewed academic-calendar change board.
- Publication/admin operations stay outside the model-visible catalogue.

### Simple Calendar — optional Rust plugin

- Market package: [`../../market/packages/ustc.simple-calendar/package.json`](../../market/packages/ustc.simple-calendar/package.json)
- Rust crate: [`../../crates/simple-calendar`](../../crates/simple-calendar)
- Agent tool: `simple_calendar_items`
- Operations: `record`, `list`, `delete`.
- Bounds: 128 items, 256-byte title, optional RFC 3339 timestamp, stable `calendar:item:N` ID.
- Persistence: owner-local JSON, regular non-symlink store, bounded load, atomic temp-file write and success only after durable commit.

The package is optional (`defaultInstalled=false`) so the frozen three-plugin default topology remains intact. The loopback MVP bundles it as a companion demo. It does not implement reminders, recurrence, CalDAV, sharing, synchronization or natural-language date parsing.

## 4. Runtime and state

`ustc-agentd` is the composition root. The same process owns HTTP admission, provider adaptation, tool validation, product services and local state. This avoids a second policy authority in the browser or model.

The Compose profile persists state in one named volume. Calendar state is stored beside the existing idempotency store as `*.calendar-items.json`; chat history itself remains page-local and is not durable.

Runbook: [`../../deploy/mvp-compose/README.md`](../../deploy/mvp-compose/README.md)

## 5. Quick acceptance

After starting the loopback MVP, try:

- `你好，介绍一下你能做什么。`
- `成绩单证明怎么办？`
- `校历最近有什么变更？`
- `记录事项：提交开题报告`
- `列出我的待办事项。`
- Create an Opportunity profile in the page, enable the per-request consent checkbox, then ask `请根据我的偏好和评课社区信号推荐课程。`

The first four tool-backed requests should show a successful trace in deterministic mock mode. Course planning must be denied or omitted without the explicit profile context and per-request confirmation.

## 6. Current boundaries and TODO

### P0 — before claiming production use

- Replace synthetic course catalog/profile fixtures with versioned approved USTC sources.
- Define and implement lawful, rate-limited community-signal ingestion with source freshness and deletion policy; retain derived metadata rather than review text where possible.
- Add production authentication, tenant isolation, CSRF/session controls and durable consent/grant administration.
- Complete generic Market installation, grant and isolated plugin execution rather than the loopback static catalogue.

### P1 — product completeness

- Calendar structured editor, completion state, reminders, recurrence and timezone UX.
- Durable chat sessions, streaming responses and provider fallback.
- Broader Affairs/ChangeRadar coverage with source-by-source freshness indicators.
- Real-provider browser smoke in an authorized environment.

### Deferred

- A command sandbox is not part of this MVP. If added, it must use an allowlist, fixed working directory, execution timeout, output cap and no shell interpolation; arbitrary shell execution is explicitly rejected.
- Multi-agent graphs, remote hosting and Dioxus client parity.

Architecture and lifecycle details:

- [`../plan/07-runtime-and-integration.md`](../plan/07-runtime-and-integration.md)
- [`../plan/04-market-and-plugin-lifecycle.md`](../plan/04-market-and-plugin-lifecycle.md)
- [`../plan/modules/40-agent-harness-runtime.md`](../plan/modules/40-agent-harness-runtime.md)
- [`../tasks/01-execution-roadmap.md`](../tasks/01-execution-roadmap.md)
