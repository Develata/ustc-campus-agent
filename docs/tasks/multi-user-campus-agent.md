# Multi-user campus Agent delivery task

- `Status`: Approved direction; bounded local slices implemented with local integration evidence; external readiness remains open
- `Last Review`: `2026-09-06`
- `Owner`: Main; independent bounded contributors use disjoint owned files
- `Scope`: implementation, bounded real model tests, protected-main PR push and merge explicitly authorized by the user on 2026-09-06; no release, tag or public deployment grant
- `Authority`: [product](../plan/02-product-positioning.md),
  [first-party Plugins](../plan/06-first-party-plugins.md),
  [permissions](../contracts/permissions.md),
  [accounts](../contracts/platform-account.md),
  [Market](../contracts/market-lifecycle.md), [Agent Chat](../contracts/agent-chat.md)

This task records the user's confirmed multi-user direction and schedules its
implementation. Owning plans/contracts govern semantics; task ordering does not
silently redefine the default first-party package identities or source-use rules.

## 1. Confirmed product direction

- A campus Agent service for multiple users, with Chat as the main entry and a Plugin
  Market entry. Account/settings and administrator controls remain secondary.
- User entry uses SSO or administrator-configured accounts only; no self-service or
  invitation registration. The live SSO adapter depends on verified integration inputs;
  controlled identity counterparts let independent product work proceed immediately.
- Administrator-configured default model and public package catalog; private user
  configuration, credentials, installation state and authorization remain scoped. Users may configure their own model API key through a private credential boundary; that provider-settings flow is planned.
- The primary real journeys are official-information lookup, personal calendar and
  course recommendation. Recommendation combines user interests/constraints, reviewed
  course facts, explicitly attributed community signals and model subject knowledge.
- Plugin packages can contain standard MCP and Skills components. The user flow is
  **browse capability/source/conditions -> install exact package -> configure -> review
  permissions -> enable -> use through Agent**. Installing/configuring is not a grant.
- Authorized queries execute automatically. Writes, deletion and sending display the
  exact proposed effect before confirmation; later users may grant explicit bounded,
  revocable standing authorization. A model or Skill cannot approve its own effects.
- Code uses cohesive responsibility boundaries and narrow public ports. Complete
  concrete flows quickly; use targeted verification during development and full checks
  at integration checkpoints.

The current named first-party identities remain Affairs Navigator, ChangeRadar and
Opportunity Graph; Simple Calendar is the existing companion package. The three
journeys above are user priorities, not an automatic package rename. Retain planned
ChangeRadar change tracking and the broader opportunity direction in the roadmap;
the user's reference to another planned Plugin does not identify a new package.

## 2. User-visible completion criteria

| Surface | Observable complete path | Current evidence boundary |
|---|---|---|
| Chat | Durable per-user conversations, continuing context, streamed responses, stop/retry, bounded tool progress and source-bearing results | Owner-scoped dialogue, actual provider SSE, stop, exact retry and durable completed-tool checkpoints are implemented locally; whole Harness and final multi-user/browser acceptance remain open |
| Official information | Ask a question; receive applicable procedure/notice with official link, observation/effective time and explicit uncertainty | Reviewed demo procedure/changes plus manifest-bounded public-source fetch/import, search, local review and line differences; no canonical publication or production retrieval acceptance |
| Calendar | Propose dated action in Chat; confirm; create/read/change/delete own item; reliable duplicate suppression and reminders | Authenticated owner workspaces, exact dated and atomic batch confirmation, durable station-inbox reminders and restart handling are implemented; external notification channels and production operations remain open |
| Courses | Submit owned interests/completed courses/time constraints; compare grounded alternatives and reasons; confirm proposed calendar additions | Request-consented course evidence, prerequisite/time/credit constraints and explained alternatives produce Calendar batch suggestions requiring separate confirmation; supplied excerpts are not independently verified facts |
| Plugin Market | Inspect source/conditions; install pinned package; configure MCP/Skills; grant and enable; revoke/disable prevents new calls | Reviewed public-read single/mixed packages reuse durable install/configure/probe/grant/enable/disable/revoke and real execution; inert import preview and disabled-only exact update/rollback are implemented; new pins require fresh authority |
| Accounts | Administrator configures users; local or verified SSO login; logout/revoke and restart preserve isolation | Operator-configured Argon2 accounts, login/logout, current-configuration checks and owner admission are implemented for loopback use; live SSO and full ACCOUNT acceptance remain open |

A stopped response is not proof a side effect was cancelled. Retry reads or continues
server-owned run/receipt state and cannot repeat an acknowledged write. Tool progress
shows observable actions and evidence, not hidden model reasoning. Simple questions
need no planner/reviewer overhead; bounded planning and optional review serve complex
tasks whose intermediate results benefit from independent checking.

## 3. Dependency slices

Each row has one owning module; cross-module assembly belongs in `ustc-agentd`.
The owner records touched files, contract/case IDs, targeted results and one reviewer
before treating a row as implemented. No row is complete merely because its UI exists.

| Slice | Owner and output | Depends on | Status / exit evidence |
|---|---|---|---|
| P1 | M20 exact reviewed package/schema loading and browse projection | Existing catalog and configuration primitives | Partial: bounded catalog HTTP/browser, schema/pin consistency and reviewed configuration sidecar loading into one fixed bundle; full package artifact admission remains planned |
| P2 | M20 install/configure/grant/enable/disable application ports with durable state | P1, controlled admitted context; A3 for real authenticated ingress | Partial: durable owner-scoped lifecycle, mixed members, reviewed import preview and exact disabled-only B6 update/rollback with stale old grants (PLUGIN-001); authenticated assembly needs final integration evidence |
| R1 | M30 durable conversation/run lifecycle and typed events/cancel/retry | Existing Harness plan and controlled admitted context; A3 for real ingress | Partial: owner-scoped history, actual SSE deltas, stop and completed-tool checkpoint recovery; exact retries never redispatch interrupted tools; full Harness acceptance remains planned |
| C1 | Calendar owner-scoped command/query and exact-effect confirmation | Owning Calendar contract update and controlled admitted context; A3 for real ingress | Partial: admitted owner workspaces, dated and batch proposals, atomic confirmation receipts and station-inbox reminders; external delivery and production recovery remain planned |
| S1 | M60 reviewed source/revision pipeline into official-information and ChangeRadar flows | Source authority and permission evidence | Partial: S1-LOCAL-OBS-001 manifest-bounded public HTTPS acquisition/import, immutable local observations, search and line differences; real-source permission/read-back evidence and original B3 exit remain required |
| O1 | M72 user preferences and course evidence projection | P2, approved source inputs and controlled admitted context; C1 for calendar composition; A3 for real ingress | Partial: O1-PERSONAL-PLAN-001 request-local course evidence and deterministic constraints, explained alternatives and separately confirmed Calendar batch suggestions; no verified curriculum or iCourse ingestion claim |
| X1 | M51 bounded reviewed MCP discovery/execution and Skill context loading | P2, owning component contracts | Partial: released Streamable HTTP JSON/SSE controlled-peer calls, bounded YAML/declared Skill reads, per-call authority and no business retries (MCP-019, SKILL-009, PLUGIN-001); executable Skills, stdio and private/write capabilities remain outside this profile |
| A1 | M00 administrator-configured account/SSO association decisions, scoped subject and controlled credential ports | M00-ACCOUNT-001 | Partial: M00-ACCOUNT-LOCAL-001 local operator configuration and scoped admission; live SSO association remains planned |
| A2 | M00 services + M90 durable account, credential and session transaction adapters | A1, existing session kernel | Partial: private Argon2 configuration, durable session transactions, current credentials/revocation and rollback fence; full recovery and ACCOUNT exits remain planned |
| A3 | M10 typed backend user configuration/login/me/logout + M80 account UI; controlled SSO adapter until real inputs exist | A2 | Partial: loopback login/me/logout and subject-bound private ingress/UI; final two-browser isolation and stale-tab evidence required; live SSO separately gated |
| I1 | M10 composition of complete campus task and contest demonstration | Required preceding real flows | Planned; exact-candidate end-to-end receipt and recording |

Prioritize a usable campus Agent, the install/configure/authorize/use Plugin flow, and
official-information/calendar/course journeys. P2, R1, C1, O1 and X1 may implement
owner-scoped domain/application behavior against controlled admitted contexts while
the account support lane proceeds separately. Do not make registration, invitation or
a complete authentication UI a prerequisite for independently testable product work.
Source-dependent first-party work still follows the governing source/revision sequence.

Controlled contexts are supplied only by explicit test or isolated loopback composition;
a browser or model cannot assert an actor. They prove bounded scope/isolation behavior,
not real authentication. Before real multi-user private ingress, A3 must admit each
request and every private owning port must enforce its subject. Never create one full
composition per user: public sources/catalog remain shared authorities, while admitted
`(TenantId, UserId)` selects private state through each owning port.

Before S1/O1 introduce real retrieval, update and satisfy source-specific contracts.
In particular an iCourse aggregate-rating snapshot is data use, not merely an external
link. Its permission basis is unresolved; an outbound link does not authorize collecting
or redistributing ratings. User interest in iCourse recommendations does not itself
establish a provider data-use agreement. Preserve reviewed fixture boundaries until
the owning source contract and permission evidence admit broader use.
Keep official facts, attributed community opinions and model suggestions distinct;
missing curriculum/timetable facts cannot be invented from model knowledge.

Before X1 implementation freezes transports, bind the supported standard MCP/Skill
subset and actual controlled examples. Package governance must not be advertised as
support for every transport or executable Skill resource. Direct execution of arbitrary MCP URLs, host commands or standalone Skills remains outside
this path. Import preview emits inert candidate package files with provenance for review;
only reviewed admission followed by the existing configure/grant/enable flow can run them.

## 4. Shared-state and recovery obligations

New service state starts separately from the existing demo. Preserve original demo
artifacts and unrelated source changes. The following state cannot silently become
owned by the first configured or SSO-authenticated user:

- Calendar v1 files without tenant/user identity: future import requires an explicit
  target owner and read-back receipt.
- Synthetic profiles, grants, session histories and administrative receipts: preserve
  existing identities and provenance; do not relabel acknowledged effects.
- Browser profile hints and pending retry envelopes: partition by authenticated
  subject and clear private projections on logout/account change; never replay across users.
- Administrator model credentials: runtime resolution may serve authorized users but
  never copies the credential into user-visible configuration, package files or model context.

Account disable and grant revoke block new admission/calls; they do not rewrite
historical receipts. Restores require a declared compatibility/revocation policy.
Public runtime readiness also requires per-user resource budgets, bounded concurrent
runs, trusted-origin/proxy configuration, durability and restore evidence. The existing
loopback listener and demo administrator header are not production authentication.

## 5. Competition evidence and integration checkpoints

Target journey (not yet one implemented end-to-end demonstration): **ask about course requirements -> compare course options
with sources -> form a plan -> confirm calendar additions -> inspect later official
notice changes and their impact**. No step labels a fixture, planned feature or model
opinion as a real official result. Use the [current competition walkthrough](../guides/competition-demo.md)
for a runnable recording sequence and honest remaining gaps.

| 107 competition dimension supplied by the user | Evidence to collect |
|---|---|
| Innovation | Campus-specific problem, coherent Chat/plugin interaction, and a complex task where bounded planning/review measurably catches a conflict |
| Practicality | End-to-end student journey with real constraints, useful sources, independent-user isolation and a reusable package installation path |
| Technical depth | Architecture/call boundaries, model/tool contract, grounded retrieval, bounded execution, conflict/timeout/retry handling and measured latency |
| Completion | Exact build and configuration instructions, architecture diagram, model call explanation, targeted/integration results and a clear demonstration recording |

During implementation run affected module checks and the relevant real path; preserve
results instead of repeating all suites after each small edit. At A3, P2/X1 assembly
and I1, run appropriate Rust fmt/clippy/tests, contract checks and the bound acceptance
commands against the same candidate. Review all changed and new files in the slice.
`planned`, `partial`, `blocked` and `not-run` are reported explicitly.

SSO, external reminder delivery channels, broader live sources and any additional unnamed
Plugin retain explicit prerequisites. Local work can proceed on owned contracts and
fakes while those are unresolved. No finished multi-user, real-source, MCP/Skill or
school SSO claim is made until its actual user path has evidence.


## 2026-09-06 implementation batch

Owner: Main. Independent implementation lanes: accounts/session admission,
M20/M51 mixed-package lifecycle, M60/M72 source/course workspace, and M30 execution
stream/cancel/checkpoint. Calendar owner changes and M10 assembly remain Main-owned;
independent review covers tenant isolation, Calendar effects and source acquisition.

Contracts: CALENDAR-WORKSPACE-001, M00-ACCOUNT-LOCAL-001, S1-LOCAL-OBS-001,
O1-PERSONAL-PLAN-001 and existing PLUGIN-001 / CHAT acceptance families. Targeted
commands and exact bounded behavior are linked from [campus workflows](../guides/campus-workflows.md).
Integration, real-provider/browser evidence and CI must be recorded before this batch
is called complete. No whole-module production acceptance is inferred from these local slices.

Bounded implementation is now present for the six surfaces above; this records code
scope, not completion of I1 or any whole-module exit. The local account, source and
course case IDs remain contract-level partial bindings until their full active
acceptance registration and evidence are complete.

Verification recorded for this lane: the focused mixed-package runtime command
`cargo test --locked -p ustc-agentd --lib plugin_runtime::tests::mixed_import` passed
3 tests on 2026-09-06, including actual MCP/Skill execution after exact component
selection and rejection of aliased declaration paths. Final integration fmt/clippy,
all-target tests, reviewed-source read-back, configured-provider/browser journeys
and CI results are pending this batch's consolidated evidence; no pass is inferred
from the command listings.

本批实际检查与未运行边界见 [校园功能使用与本地集成证据](../guides/campus-workflows.md#2026-09-06-本地集成证据)。
