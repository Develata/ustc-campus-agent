# MVP core capabilities

> Status: implemented loopback MVP · Last review: 2026-09-05

## 1. Design

The MVP follows four rules:

1. **Server authority.** The browser and model may propose; Rust validates every request, tool name, argument and permission boundary.
2. **Small complete paths.** A normal answer and each tool call have an executable end-to-end path, not only a manifest or UI mock.
3. **Source-labelled output.** Official/synthetic facts, community signals and private profile state remain distinguishable. Community feedback affects only soft ranking.
4. **Honest degradation.** Missing consent, stale source, provider failure and unsupported features return explicit non-success outcomes instead of plausible text.

## 2. User-visible flow

```text
Browser or Android demo WebView (loopback only)
  → POST /api/v1/agent/chat
  → bounded ChatRun
  → deterministic mock or OpenAI-compatible provider
  → fixed reviewed Rust tool catalogue
       affairs_navigator_get
       change_radar_get
       simple_calendar_items
       opportunity_graph_plan_current_profile (only with per-request consent)
  → typed tool results marked untrusted
  → concise human summary + redacted tool trace
```

The default mock mode needs no API key and is intended for deterministic judging and offline acceptance. For every known successful tool shape it emits a server-owned bounded Chinese summary—procedure steps and official entry points, changed fields, course candidates and iCourse link-outs, or Calendar mutations—rather than transport JSON. Shape drift becomes an explicit summary-contract notice. The optional real-provider mode uses the same fixed definitions and Rust executor; its key remains file-backed and server-side. Neither mode derives a dynamic provider catalogue from package disable/revoke state.

The debug Android APK is a thin presentation bridge over this exact route. It reaches the host loopback service through explicit `adb reverse`, contains no local tool or product implementation, and exposes native loading/offline/retry/server-origin controls. See [`07-android-demo-client.md`](07-android-demo-client.md).

## 3. Capability matrix

### Normal Agent Q&A

- Multi-turn page-local conversation with bounded history.
- Deterministic offline response or explicitly configured OpenAI-compatible Chat Completions provider; both cross the complete loopback HTTP route in retained tests.
- Optional request v2 carries one closed, non-persistent `prompt_customization.text` user preference: at most 2048 UTF-8 bytes, nonblank after trim, and free of disallowed control, bidi, zero-width and BOM scalars. The immutable system policy remains first; the separately labelled untrusted preference changes no tool or authority. Empty Web input preserves request v1, while nonempty request-scope input uses v2 and is never added to history or `localStorage`.
- Maximum 3 provider turns, 4 tool calls, 4 KiB arguments per call, 64 KiB result per call and 16 KiB final answer.
- The deterministic provider uses tool-aware human summaries with fair per-result budgets, so a large course plan neither exposes protocol plumbing nor erases a later Calendar result.
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

### Simple Calendar — owner-local in-process companion

- Market package: [`../../market/packages/ustc.simple-calendar/package.json`](../../market/packages/ustc.simple-calendar/package.json)
- Rust crate: [`../../crates/simple-calendar`](../../crates/simple-calendar)
- Agent tool: `simple_calendar_items`
- Operations: `record`, `list`, `delete`.
- Record intent: the final user message is exactly `记录事项：<nonblank title>` or `记录事项:<nonblank title>`; the outer-trimmed suffix equals the provider-call title byte-for-byte and `scheduled_for` is absent.
- Delete intent: the final user message is exactly `删除事项 calendar:item:N`, with one complete stable ID equal to the provider-call ID and no hidden/extra suffix.
- List is read-only. Bounds remain 128 items and a 256-byte title.
- Absent or mismatched mutation intent yields a bounded denied result/trace and zero executor/store operation. Provider text cannot mint confirmation, and the deterministic mock uses the same exact grammar rather than keyword matching.
- Persistence: owner-local `calendar-items.json`, regular non-symlink store, bounded load, atomic temp-file write and success only after durable commit.

The package declaration is optional (`defaultInstalled=false`) so the frozen three-path default topology remains intact. The loopback MVP composes it directly as a fixed in-process companion; this is not evidence of generalized installation, disable/revoke projection or isolated execution. It does not implement reminders, recurrence, CalDAV, sharing, synchronization or natural-language date parsing.

## 4. Runtime and state

`ustc-agentd` is the composition root. The same process owns HTTP admission, provider adaptation, tool validation, product services and local state. This avoids a second policy authority in the browser or model.

The Compose profile persists one complete locked state set in a named volume. Calendar state is `idempotency_path.with_extension("calendar-items.json")`: a fresh bootstrap persists canonical empty mode-`0600` state, rollback removes it with every other newly created member, and a non-fresh missing member fails `durable_state_set_incomplete`. Restart preserves committed items. Chat history itself remains page-local and is not durable.

The assembled Compose directory, tar archive and ZIP archive each carry package-root `LICENSE.md` byte-identical to repository-root `LICENSE.md`, mode `0644`, and listed in `SHA256SUMS`. Archive acceptance reads back both formats and preserves deterministic-byte and provider-secret checks.

Runbook: [`../../deploy/mvp-compose/README.md`](../../deploy/mvp-compose/README.md)

## 5. Quick acceptance

After starting the loopback MVP, try:

- `你好，介绍一下你能做什么。`
- `成绩单证明怎么办？`
- `校历最近有什么变更？`
- `记录事项：提交开题报告`
- `列出我的待办事项。`
- `删除事项 calendar:item:1`
- Create an Opportunity profile in the page, enable the per-request consent checkbox, then ask `请根据我的偏好和评课社区信号推荐课程。`

Every admitted tool-backed request should show a readable summary and successful trace in deterministic mock mode. Answers should expose user facts such as procedure steps, changed fields, course codes/link-outs and Calendar item IDs, but not raw field names such as `ordered_steps`, `changed_fields`, `course_codes` or `command_id`. `日历怎么用`, `提醒我日历怎么用`, `calendar help`, provider-proposed mutation for a read-only prompt, mismatched title/ID, and hidden/extra suffixes must show denied/non-mutation behavior with zero Calendar state change. Course planning must be denied or omitted without the explicit profile context and per-request confirmation. A nonempty request preference may change response presentation only; it must not change the first system policy, tool count or authorization, and it must not appear in the next request.

## 6. Current boundaries and TODO

### P0 — before claiming production use

- Replace synthetic course catalog/profile fixtures with versioned approved USTC sources.
- Define and implement lawful, rate-limited community-signal ingestion with source freshness and deletion policy; retain derived metadata rather than review text where possible.
- Add production authentication, tenant isolation, CSRF/session controls and durable consent/grant administration.
- Complete the generalized Market installation/grant lifecycle and isolated execution; the current fixed reviewed demo catalogue does not claim it.

### P1 — product completeness

- Calendar structured editor, completion state, reminders, recurrence and timezone UX.
- Durable chat sessions, streaming responses and provider fallback.
- Persisted prompt profiles or editable system/developer policy.
- Broader Affairs/ChangeRadar coverage with source-by-source freshness indicators.
- Real-provider browser smoke in an authorized environment.

### Deferred

- A command sandbox is not part of this MVP. An allowlist wrapper must not be called a sandbox; arbitrary shell execution is explicitly rejected.
- Multi-agent graphs, remote hosting and shared-client parity.
- Production-signed Android, secure authenticated HTTPS sessions and complete real-device `CLIENT-002` evidence; the bounded debug APK does not imply these.
- Skill loading/runtime and usable inbound or outbound MCP adapters remain unimplemented and unclaimed.

## 7. Independent usability candidate

This branch additionally implements the bounded [usable-demo enhancements contract](../contracts/usable-demo-enhancements.md); it is not yet a replacement for the frozen competition package.

- **可配置演示档案**：在 synthetic 课程目录中选择已修课程、学分范围和偏好，明确同意后创建新的档案快照；比较 Rust 生成的候选方案。编辑草稿不会修改已保存档案，切换档案不会继承 Chat 授权。
- **个人办理清单**：对当前成功查询的 Affairs 步骤标记个人进度，复制／下载带来源与不确定性说明的 Markdown。标记不代表官方受理或审批，页面刷新不保留勾选。
- **场景入口**：四个入口只填问题和导航，不自动发送、授权或写日历；课程入口引导缺少档案的用户完成显式创建。

The API/catalog/planner remains unchanged. Synthetic pending-create values are snapshotted with the existing idempotency envelope for exact retries; this is not support for storing real student records in browser storage. The supplemental acceptance cases are bound in the enhancement contract and its retained browser test. The historical matrix remains byte-identical because its legacy M60 projection is frozen; this candidate does not promote those historical rows. Exact compiled-binary gates, independent review and package read-back are required before the candidate can replace the frozen delivery. Build infrastructure remains separate from product source and does not authorize merge or Release.

Architecture and lifecycle details:

- [`../plan/07-runtime-and-integration.md`](../plan/07-runtime-and-integration.md)
- [`../plan/04-market-and-plugin-lifecycle.md`](../plan/04-market-and-plugin-lifecycle.md)
- [`../plan/modules/40-agent-harness-runtime.md`](../plan/modules/40-agent-harness-runtime.md)
- [`../tasks/01-execution-roadmap.md`](../tasks/01-execution-roadmap.md)
