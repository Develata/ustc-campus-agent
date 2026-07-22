# Course Planning fixtures

This directory contains non-sensitive, deterministic fixtures for the first Course Planning vertical slice.

## `minimal-v0.json`

- Contract: `course-planning/v0`.
- Scope: 20 unique synthetic course candidates plus one lower-authority duplicate fact used to test source precedence.
- Data classification: entirely synthetic; no real credentials, student records, grades, rankings, phone numbers, or private academic exports.
- Source policy:
  - every source carries revision, retrieval time, effective term, stale state, and a synthetic provenance note;
  - `official-catalog-synthetic` and `department-notice-synthetic` imitate approved source shapes without claiming real integration;
  - `icourse-mirror-synthetic` exists only to prove that lower-authority facts cannot override official facts;
  - `icourse-linkout` includes URL and bounded synthetic scores only—no review text is copied, cached, or summarized.
- Expected behavior:
  - at least two feasible plans;
  - zero hard-constraint violations;
  - prerequisite-ineligible courses are excluded;
  - unresolved aliases are excluded rather than guessed;
  - community scores affect soft ordering only.

Run:

```bash
cargo run --locked -p ustc-agentctl -- course plan \
  --fixture market/fixtures/course-planning/minimal-v0.json \
  --format json
```

Fixture rules for future additions:

- state schema version, source revision, retrieval/import method, and anonymization status;
- use approved scrubbed snapshots or synthetic records only;
- keep user-owned profiles separate from campus-source facts;
- do not embed iCourse review text without explicit permission;
- do not commit raw USTC credentials, CAS/MFA material, or real student data.
