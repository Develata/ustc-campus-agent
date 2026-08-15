# Course Planning data model contract

Status: implemented v0 in `crates/course-planning`.

## Boundary

The v0 planner is deterministic and fixture-driven. It performs no model calls, network access, database writes, or enrollment side effects.

## Input contract: `course-planning/v0`

`CoursePlanningFixture` contains:

- `schema_version` and complete `source_revision`;
- `sources: SourceDescriptor[]`;
- `profile: UserAcademicSnapshot`;
- `requirements: RequirementGroup[]`;
- raw `courses: CourseOffering[]`;
- optional `community_signals: CommunitySignal[]`.

### Authority-bearing source objects

`SourceDescriptor` carries:

- stable source id;
- `CoursePlanningAuthority`;
- source-local revision;
- RFC 3339 retrieval/import time, validated fail-closed;
- effective term/date range when available;
- stale flag;
- provenance note.

Authority order is a **course-planning-local total order**, not the generic platform-wide source authority ordering. The generic `M60` source authority (see [`docs/plan/05-campus-trust-kernel.md`](../plan/05-campus-trust-kernel.md) §3.2) carries no product-specific variants such as `icourse_mirror` or `official_catalog_snapshot` and defines only a partial comparison (`Higher | Lower | Equivalent | Incomparable`). Course Planning admits this local total order behind its own policy/type:

```text
official_catalog_snapshot
> reviewed_official_source
> icourse_mirror
> community_signal
> model_inference
```

`model_inference` is the lowest tier and is rejected as a source authority for requirements and course facts; it is retained in the ordering only so a model-proposed candidate can be explicitly classified and denied, never selected.

### Bitemporal provenance

The v0 Rust `FactProvenance` (`crates/course-planning/src/lib.rs`) carries, per material output fact:

- `retrieved_at`: retrieval/import timestamp projected from the source revision's `observed_at`.
- `effective_time`: optional term/date range in which the fact applies, projected from the source revision's effective interval.
- `conflict_status`: how competing source records were handled.

The full canonical fact-level bitemporal vocabulary defined in [`docs/plan/05-campus-trust-kernel.md`](../plan/05-campus-trust-kernel.md) §3.1 — `valid_at`, `known_at`, `as_of`, plus the planned Affairs Navigator evidence-context fields (`observed_at`, `reviewed_at`, `last_verified_at`) — is **not yet projected onto v0 `FactProvenance`**. v0 uses `retrieved_at`/`effective_time` as a bounded, source-revision-derived subset of that vocabulary; it does not carry `as_of` (a query/answer-level cutoff, not a fact-level field; canonical definition in [`docs/plan/05-campus-trust-kernel.md`](../plan/05-campus-trust-kernel.md) §3.1) and does not carry review/verification timestamps. Inflating v0 `FactProvenance` to claim those fields would be a contract lie; the full evidence context is owned by the Affairs Navigator plan/contract ([`docs/plan/06-first-party-plugins.md`](../plan/06-first-party-plugins.md) §2.6) and lands with its first product slice.

Requirements must come from a non-stale `reviewed_official_source` or `official_catalog_snapshot`. Course facts may use an iCourse mirror fallback, but the highest-authority fact set is resolved before lower-authority conflicts are considered. Equal-highest-authority conflicting facts are excluded rather than guessed; result provenance records whether a fact had no known conflict, equivalent sources, or an authority-resolved conflict.

All v0 input objects reject unknown JSON fields, and every course must explicitly provide both `prerequisites` and `slots` arrays (empty arrays are valid). This is fail-closed by design: a misspelled or omitted hard-constraint field must not silently erase a prerequisite or meeting conflict.

### User-owned inputs

`UserAcademicSnapshot` contains:

- completed course codes;
- requested minimum and maximum credits;
- per-course soft preference weights.

It is user-owned data, not campus-source authority.

### Course identity and scheduling

`CourseOffering` contains normalized code, title, integer credits, prerequisites, weekly half-open meeting intervals, source id, availability, and `IdentityStatus`.

`unresolved_alias` courses are excluded. A prerequisite must already be completed; v0 does not treat a concurrent course as satisfying a prerequisite.

### Subjective signals

`CommunitySignal` contains course code, source id, bounded score, and an iCourse link-out URL. Non-stale signals affect soft ordering only and cannot establish credits, requirements, prerequisites, availability, or meeting times. Stale signals are warned, excluded from scoring, and omitted from output provenance.

## Output contract: `course-plan-result/v0`

`PlanResult` contains:

- source revision;
- ranked `PlanCandidate[]`;
- global hard-constraint violation count;
- deterministic warnings.

Each `PlanCandidate` contains selected course codes, total credits, requirement coverage, a 64-bit accumulated soft score, independently recomputed hard violations, rationale, and fact-level provenance with revision, retrieval time, optional effective time, authority, and conflict status.

## Search contract

The initial planner uses bounded deterministic beam search:

- default beam width: 1,024;
- default maximum results: 3;
- partial states violating credit or schedule bounds are pruned;
- final candidates must satisfy all requirement and credit constraints;
- hard constraints are recomputed independently before serialization.

The bounded search is an MVP strategy, not a claim of global optimization for arbitrary curriculum instances.

## Fixture

The canonical synthetic fixture is [`market/fixtures/course-planning/minimal-v0.json`](../../market/fixtures/course-planning/minimal-v0.json). It contains 20 unique synthetic courses plus one lower-authority duplicate fact and no real student data.
