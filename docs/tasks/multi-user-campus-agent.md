# Multi-user campus Agent delivery task

- `Status`: Approved direction; dependency slices planned, existing bounded MVP partial
- `Last Review`: `2026-09-05`
- `Owner`: Main; independent bounded contributors use disjoint owned files
- `Scope`: local implementation and evidence; no remote deployment, push or publication grant
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
| Chat | Durable per-user conversations, continuing context, streamed responses, stop/retry, bounded tool progress and source-bearing results | Saved owner-scoped dialogue, reload/continuation and deduplicated turn submission have bounded local evidence; streaming and real multi-user runtime planned |
| Official information | Ask a question; receive applicable procedure/notice with official link, observation/effective time and explicit uncertainty | Fixed reviewed demo procedure/changes; broader live sources planned |
| Calendar | Propose dated action in Chat; confirm; create/read/change/delete own item; reliable duplicate suppression and reminders | Durable owner-local record/list/delete and dated create/update/delete proposals with exact confirmation; production tenant scope and reminder delivery planned |
| Courses | Submit owned interests/completed courses/time constraints; compare grounded alternatives and reasons; confirm proposed calendar additions | Synthetic profile/catalog bounded planning; real course data and richer preferences planned |
| Plugin Market | Inspect source/conditions; install pinned package; configure MCP/Skills; grant and enable; revoke/disable prevents new calls | Bounded single-component public-read packages have durable install/configure/probe/grant/enable/disable/revoke, real MCP/Skill execution and restart evidence; arbitrary imports, mixed-component packages and real authenticated ingress remain planned |
| Accounts | Administrator configures users; local or verified SSO login; logout/revoke and restart preserve isolation | M00 ID/session kernels exist; real account and credential adapters planned |

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
| P2 | M20 install/configure/grant/enable/disable application ports with durable state | P1, controlled admitted context; A3 for real authenticated ingress | Partial: controlled-owner scope, durable commands, conflict/restart/revoke, frozen authority and browser lifecycle evidence (PLUGIN-001); real authenticated ingress remains A3 |
| R1 | M30 durable conversation/run lifecycle and typed events/cancel/retry | Existing Harness plan and controlled admitted context; A3 for real ingress | Partial: saved dialogue, owner isolation, reservation/replay and interrupted recovery; full run events, stop/cancel and receipt reconciliation planned |
| C1 | Calendar owner-scoped command/query and exact-effect confirmation | Owning Calendar contract update and controlled admitted context; A3 for real ingress | Partial: local dated proposals, explicit confirmation, version conflict and atomic receipt/restart evidence; production owner storage and reminders planned |
| S1 | M60 reviewed source/revision pipeline into official-information and ChangeRadar flows | Source authority and permission evidence | Partial fixed fixtures; real-source before/after read-back planned |
| O1 | M72 user preferences and course evidence projection | P2, approved source inputs and controlled admitted context; C1 for calendar composition; A3 for real ingress | Partial synthetic planning; explainable real constraints and confirmed calendar write planned |
| X1 | M51 bounded reviewed MCP discovery/execution and Skill context loading | P2, owning component contracts | Partial: released Streamable HTTP JSON/SSE controlled-peer calls, bounded YAML/declared Skill reads, per-call authority and no business retries (MCP-019, SKILL-009, PLUGIN-001); executable Skills, stdio and private/write capabilities remain outside this profile |
| A1 | M00 administrator-configured account/SSO association decisions, scoped subject and controlled credential ports | M00-ACCOUNT-001 | Planned support lane; ACCOUNT-001/002/003 controlled cases |
| A2 | M00 services + M90 durable account, credential and session transaction adapters | A1, existing session kernel | Planned support lane; atomic configuration, credential verification, revoke and restart cases |
| A3 | M10 typed backend user configuration/login/me/logout + M80 account UI; controlled SSO adapter until real inputs exist | A2 | Planned support lane; real two-browser local login and negative admission cases; live SSO separately gated |
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
support for every transport or executable Skill resource. Arbitrary independent MCP
URLs, host commands and standalone local Skill imports are outside this delivery path;
any later import must preserve reviewed package provenance and permission boundaries.

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

SSO, reminder delivery channels, broader live sources and any additional unnamed
Plugin retain explicit prerequisites. Local work can proceed on owned contracts and
fakes while those are unresolved. No finished multi-user, real-source, MCP/Skill or
school SSO claim is made until its actual user path has evidence.
