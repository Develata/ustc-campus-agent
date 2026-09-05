# Usable demo enhancements v1

Status: user-approved enhanced submission scope (2026-09-05); merge and replacement require exact-source checks and artifact read-back, not this status line.

## Scope and ownership

User approved configurable synthetic course planning, personal Affairs checklist export, and guided scene entry. Preserve existing visual language and Rust authority. Main owns integration, contracts, scenarios, acceptance and final verdict. Course and checklist writers own separate browser files; formal reviewer is native codex-reviewer. This bounded user-approved interaction improvement does not adopt a new frontend framework/design system or the historical unconfigured design-model lanes.

Baseline: `91321f040f8cdfa6937b831f496630eeac43eb27`. Product changes remain limited to these interactions: no changes to authentication authority, tool catalogue, planner algorithm, calendar or deployment semantics. The initial candidate grant excluded main and R2 replacement. Develata subsequently authorized protected-main synchronization and replacement of the competition submission bundle with the final enhanced source, subject to review, exact-head CI, exact-main artifact rebuild and read-back. Previously delivered archives remain historical rollback copies. No tag, Release, public runtime or competition-portal upload is authorized by this scope.

The separately executable [SSO reservation sample](../../examples/sso-interface/README.zh-CN.md) documents the authorization/configuration prerequisite and exercises only disabled status/start/callback HTTP responses. It is not wired into `ustc-agentd`, creates no identity or session, and does not promote M00 or M10 authentication readiness. Its standalone acceptance command is `python3 -B -m unittest discover -s examples/sso-interface -p 'test_*.py' -v`; application and artifact gates remain separate.

## UE-01 Configurable synthetic profiles

- Expose choices from the checked-in synthetic catalog only, including completed course inputs admitted by it, integer credit bounds and bounded integer preference weights.
- Use existing owner-consent profile-create operation; no raw profile import, SSO or new backend authority. Validation supplements, never replaces, Rust validation.
- Draft changes are local, not a saved profile. Creating requires explicit consent; new/current profile identity changes clear Chat confirmation. No automatic create, plan or chat send.
- Reuse exact saved request bytes for an uncertain create retry. Draft edits must not alter an already pending request; prevent misleading new-draft/retry joins.
- No in-place profile mutation. Rust admits one live profile per principal. Adopting an edited draft requires the existing explicit owner revoke/delete action first, then fresh consent/create; never silently delete. Keep page-local synthetic draft through deletion, explain that deletion is irreversible and the draft is not saved state.
- Display existing server-generated candidates side by side; no client-side recomputation of eligibility/ranking. No feasible plan must remain an honest terminal.

## UE-02 Personal Affairs checklist

- Use successfully rendered structured Affairs result only, never model-generated steps. Initial/error/loading states cannot export stale results.
- Checkmarks mean personal progress only, never official receipt/approval. Page-local state; reset on a new result or lookup, no implied durable sync.
- Copy and download Markdown retain current procedure, prerequisites, steps, official links, last verification, freshness, validity, conflict and uncertainty details. Escape untrusted markup/URLs rather than executing HTML.
- Clipboard failure has visible fallback; download uses a local Blob, no network/upload or personal storage. Print optional via browser UI, not a new required surface.

## UE-03 Guided entry

- Four buttons fill supported prompts for Affairs, Radar, planning and read-only Calendar list. Never auto-submit, pre-check consent, create profiles or mutate Calendar.
- Planning without current profile directs to the editor/explicit create control; existing current profile requires separate one-request Chat confirmation.
- Copy-answer and source-panel navigation may be provided adjacent to results. No arbitrary provider text is promoted to verified-source evidence. Failure remains visible.

## Acceptance (all planned before implementation)

Bindings: active `UE-001`, `UE-002`, `UE-003` and `SSO-001` rows in [`../acceptance/matrix.tsv`](../acceptance/matrix.tsv), executed by `.github/workflows/ci.yml` on pull requests and main; `scripts/test_usable_enhancements_browser.mjs`, plus existing `scripts/test_agent_chat_browser.mjs`, Python contracts/tests, and Rust gates for compiled asset wiring. The new rows add no production SSO success path and leave every pre-existing matrix row unchanged.

- UE-01A: changed valid constraints reach the real Rust endpoint and produce an observably different plan or honest infeasible result; compare at least two inputs.
- UE-01B: invalid bounds, fractional input, unavailable catalog code rejected; no implicit consent, no old-profile confirmation carryover; uncertain retry preserves bytes.
- UE-02A: real Affairs steps can be checked; downloaded/copy text binds those steps and source/uncertainty labels, no official completion claim.
- UE-02B: unavailable/failed lookup disables export; new result clears personal checks; markup renders inert.
- UE-03A: every scene only fills/navigates, no POST or authority mutation until user explicitly acts; planning guides missing profile.
- UE-03B: keyboard, 320/390px and desktop, light/dark; no horizontal overflow or uncaught errors; current original browser journeys remain green.
- UE-04: candidate archive exact checksums/readback and real binary HTTP/browser smoke. Full CI, Windows Docker runtime and Android remain separate not-run unless actually exercised.

## Related authority

[Chat](agent-chat.md), [module boundaries](module-boundaries.md), [capabilities](../features/06-mvp-core-capabilities.md), [client blueprint](../plan/modules/80-dioxus-multi-client.md).
