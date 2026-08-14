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
- `SourceAuthority`;
- source-local revision;
- RFC 3339 retrieval/import time, validated fail-closed;
- effective term/date range when available;
- stale flag;
- provenance note.

Authority order remains:

```text
official_catalog_snapshot
> reviewed_official_source
> icourse_mirror
> community_signal
> model_inference
```

### Bitemporal provenance

Every material fact in the output provenance carries bitemporal fields:

- `valid_at`: real-world validity time, projected from the source's effective term/date range.
- `known_at`: system knowledge time, projected from the source's retrieval/import time.
- `as_of`: review/acceptance time, distinct from `known_at`.

These are fact-level projections of the source-revision-level fields defined in [`docs/plan/05-campus-trust-kernel.md`](../plan/05-campus-trust-kernel.md) §3.1.

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
