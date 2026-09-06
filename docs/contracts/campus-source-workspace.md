# Local source observations and personal course planning

- Status: bounded local profile authorized by the multi-user campus Agent task; implementation evidence is recorded by its targeted tests, not by production M60 acceptance promotion.
- Version: `source-workspace/v1`, `source-review-manifest/v1`, `personal-course-request/v1`.
- Authority: [M60](../plan/modules/70-campus-trust-source-pipeline.md), [source import](source-import.md), [M72](../plan/modules/73-opportunity-graph.md), [delivery task](../tasks/multi-user-campus-agent.md).
- Cases: `S1-LOCAL-OBS-001`, `O1-PERSONAL-PLAN-001`.

## Source ownership and admission

M60 `platform-core::source_workspace` owns a bounded local observation workspace.
It uses existing M60 SourceId/SourceUrl validation. `source_search` is application
composition; `adapters::source_acquisition` owns HTTPS transport. This profile does
not construct `PublishedCanonical`, `SourceRevision::DemoReviewed`, a production
retrieval receipt or an accepted baseline. It does not claim completion of the
original source-retrieval/v0 B3 lease/journal protocol. The newly authorized local
read/import path is a distinct bounded profile; existing B3 acceptance stays planned.

An operator supplies an external `source-review-manifest/v1` JSON file with `sources`.
Each entry requires `source_id`, `title`, exact HTTPS `url`, `reviewer`, real
`permission_evidence`, real `review_evidence`, and `minimum_interval_seconds` in
60..604800. No bundled entry asserts a third-party license. The application reads
this file for each operation; removing an entry immediately excludes its observations
from current search and blocks new acquisition. A manifest is administrator-managed
configuration, not an HTTP command. A model/browser cannot approve its own URL.
Official host syntax alone is insufficient approval. Only exact declared public
USTC URLs are admitted; iCourse and authenticated pages remain excluded.

The HTTPS adapter enforces a pinned validated public IPv4 peer, HTTPS verification,
no proxy/cookie/credential/redirect, identity encoding, 1 MiB raw bytes, 128 KiB
UTF-8 text, 20-second whole-operation deadline, and HTML/plain-text content types.
No crawler, link traversal, scheduled polling, retries or private data collection
is introduced. HTML projection removes script/style/noscript and renders as plain
text; it does not infer campus facts or obey document instructions. Unrecognized
encodings and invalid response types fail closed. A search index is not an authority.

## Observation, persistence and review

`SourceObservation` carries exact source/URL, content SHA-256, observation time,
optional HTTP Last-Modified, raw snapshot, deterministic plain text, prior revision
identity, and bounded added/removed line sets. Raw snapshots are written and synced
before the workspace commit. Source identity/time/hash/text determine the immutable
observation identity. Local content review is separate metadata pinned to that exact
identity and records reviewer/evidence/time. It grants no official publication or
new data-use permission. Observed time, HTTP Last-Modified and effective date are
separate; absent publication/effective dates remain unknown.

The store has at most 256 observations, and commits under an interprocess lock by
write/sync/rename/directory sync. Restore checks schema, content binding, exact prior
chain, derived differences and bounded review metadata. A failed fetch leaves old
observations intact; failed snapshot/store writes produce no successful acknowledgement.
Identical latest content reuses its observation identity and original observation time.
Differences are line sets, bounded to 100 additions/removals, not a semantic change
publication. Source removal hides current reads without deleting historical snapshots.
Full retention, distributed leases and production publication remain separate work.

User routes are `GET /api/v1/sources`, `POST /api/v1/sources/search` with
`{schema:"source-search/v1",query,source_id:null|string}`, and source history.
Administrator fetch/import/review routes cannot be authorized by a demo header in
account mode; they require the current authenticated administrator. The isolated
loopback demo retains its explicitly labelled operator confirmation header.
Chat read tools can retrieve only the operator's exact reviewed public source IDs;
no Chat tool can edit the manifest, review an observation or publish a campus fact.
All returned text remains untrusted data; model/tool context is bounded and truncation
is explicit. No results means no matching current observation, not no campus rule.

## HTTP request budgets

`POST /api/v1/sources/{id}/import`, `POST /api/v1/courses/plan`, and
[Plugin import preview](plugin-management.md) override the service's default
16 KiB request-body limit. Source text imports have an explicit 1 MiB JSON wire budget: the 128 KiB decoded-text ceiling fits even
when each input byte uses a six-byte JSON Unicode escape, with envelope headroom.
Course planning has an 8 MiB JSON wire budget to accommodate 64 courses with
8 KiB excerpts, the bounded prerequisite/tag/URL fields and JSON escaping.
These are finite wire budgets, including JSON syntax and whitespace; they do not
promise acceptance of arbitrarily padded JSON or oversized date representations.

Decoded domain limits remain authoritative and unchanged: source text above
128 KiB or any course excerpt above 8 KiB still fails validation. Plugin import
preview separately has a 1 MiB wire budget and retains its 64 KiB decoded Skill
limit. Routes outside these three exceptions retain their existing 16 KiB default.
Body extraction failures retain the current
route-specific invalid-request response; larger local wire budgets do not relax
administrator admission, source review or per-request course consent.

`web::campus_routes::body_limit_tests` runs the production router over real HTTP.
It verifies a maximum-size escaped source import, 64 maximum-size escaped course
excerpts, an inert 64 KiB Skill preview, transport over-limit rejection, unchanged
decoded domain limits and an unrelated route's retained 16 KiB limit. Targeted command:
`cargo test -p ustc-agentd --lib campus_routes::body_limit_tests`.

## Personal course planning

`course-planning::personal` owns request-local deterministic planning. HTTP
`POST /api/v1/courses/plan` accepts `personal-course-request/v1` with explicit
`consent_this_request`, `courses`, `completed_courses`, `interests`, `free_slots`,
`min_credits_tenths` and `max_credits_tenths`. A source entry has code/title/credits,
prerequisites/tags, concrete meetings, source URL, user-supplied excerpt containing
code/title, and observation time. This records user evidence, not verified official
catalog facts; no model knowledge supplies missing curriculum or timetable fields.
An excerpt match is an input consistency check, not independent fact verification.

Courses are at most 64, meetings at most 512 overall, interests at most 32 and weekly
free windows at most 64. Credit units are tenths. Weekly free windows use Beijing
local time, weekday 1..7, and half-open minute ranges. Meetings require explicit
RFC3339 timezone and duration; missing meetings produce no fabricated calendar dates.
Completed-course, prerequisite, availability and course-overlap constraints precede
soft title/tag interest matches. A beam of 512 produces up to three alternatives;
no-result is not proof of infeasibility. Output cites supplied evidence, exclusions,
interest match reasons, and exact calendar suggestions. Academic graduation coverage,
seat availability and iCourse rating ingestion are not claimed.

No private profile/draft is stored by these ports. Account change clears page drafts.
The Chat planning tool additionally requires the original request's trusted user
consent; a model argument cannot grant it. Chat remains under the existing argument
budget, so users provide a small concise course set or use the full manual form.
Calendar suggestions are proposals only. The UI submits at most 32 exact title/date
items to Calendar's existing batch proposal port. Only Calendar's separate human
confirmation commits the batch; planning never enrolls or confirms for the user.

## Verification

- `cargo test -p ustc-campus-agent-core source_workspace`
- `cargo test -p ustc-campus-agent-adapters source_acquisition`
- `cargo test -p ustc-campus-agent-course-planning personal`
- `cargo test -p ustc-agentd source_search`
- `node --check apps/ustc-agentd/src/web/campus-workspace.js`
- Owning integration checkpoint covers the actual reviewed-source fetch, source
  read-back, course comparison, batch preview and separate confirmation routes.

Controlled tests use synthetic text and explicit controlled permission labels; they
are not evidence of a real official source license or a production retrieval run.