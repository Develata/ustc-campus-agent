# Storage profile and repository-adapter contract

## Metadata

- `Status`: Accepted infrastructure semantic contract; implementation planned
- `Version`: `storage-profiles/v0`
- `Last Review`: `2026-08-02`
- `Owning Blueprint`: [`M90 Platform Infrastructure and Operations`](../plan/modules/90-infrastructure-operations.md)
- `Authority Defers To`: each domain module's repository/transaction contract
- `Identity Consumers`: [`platform-account/v0`](platform-account.md), [`user-context-profile/v0`](user-context-profile.md), [`platform-session/v0`](platform-session.md)
- `Acceptance`: catalog-only `STORAGE-001` through `STORAGE-010`; none is active or implemented

## 1. Decision

The platform has three explicit deployment storage profiles:

| Profile | Durable SQL store | Intended use | Production claim |
|---|---|---|---|
| `local-demo` | SQLite by default | local development, CI fixtures and single-machine demonstration | none |
| `hosted` | PostgreSQL required | shared/team/remote deployment | production-capable only after its gates pass |
| `production` | PostgreSQL required | real multi-user operation | release gates, backup/restore and security evidence required |

There is no automatic PostgreSQL-to-SQLite fallback. A hosted/production configuration that cannot reach or migrate its exact PostgreSQL schema fails readiness; it never starts against a local file for convenience.

SQLite is a deliberately bounded demo adapter, not the normative concurrency or recovery model. PostgreSQL is the target for real shared use.

## 2. Authority and adapter shape

Domain modules declare narrow semantic ports such as:

```text
AccountRepository
ExternalIdentityRepository
TenantMembershipRepository
SessionRepository
ProfileFactRepository
ProfileProjectionRepository
InstallationRepository
RunJournal
```

M90 supplies backend-specific implementations. The preferred initial Rust implementation is:

```text
SQLx
+ domain-owned repository ports
+ separate SQLite adapter modules/queries/migrations
+ separate PostgreSQL adapter modules/queries/migrations
```

SQL row/ORM models never become domain types or transition authority. `AnyPool`, a generic record store or an ORM ActiveModel cannot erase backend differences or replace semantic repository methods.

SeaORM or Diesel may be evaluated for a bounded CRUD/read surface later, but their entities do not own account, linking, profile confirmation, session or authorization transitions.

## 3. Semantic portability

Every adapter must satisfy the same repository conformance suite for shared semantics:

- tenant/user/object key isolation;
- uniqueness and foreign-key behavior;
- expected-revision compare-and-commit;
- idempotent duplicate command replay and conflicting reuse rejection;
- atomic multi-row authority transitions where the owning contract requires them;
- deterministic ordering/paging and canonical value projection;
- restart/reopen durability;
- typed unavailable/corruption/constraint/conflict mapping;
- no acknowledgement before required audit/evidence persistence.

PostgreSQL additionally owns production concurrency, locking/isolation, indexing, migration, backup/restore and operational evidence. A SQLite pass cannot satisfy those PostgreSQL-only cases.

If a semantic operation cannot be implemented safely on SQLite, `local-demo` returns a typed unsupported-profile error or remains single-process by contract. It does not emulate correctness with an unsafe check-then-write race.

## 4. SQLite local-demo rules

Startup configures and verifies at least:

```text
PRAGMA foreign_keys = ON
journal_mode = WAL where the runtime/filesystem supports it
bounded busy_timeout
explicit synchronous policy
one owned database path with safe permissions
```

The profile is single-host and initially single-authority-process. It makes no horizontal scaling, high write concurrency, online migration or shared-network-filesystem claim.

SQLite stores canonical validated JSON as text and timestamps as one documented integer unit. Application/domain validation must not rely on SQLite's dynamic typing. Constraints and conformance tests reject representations that PostgreSQL would reject.

An in-memory SQLite database is a test fixture, not durable demo evidence unless the case explicitly tests only repository semantics.

## 5. PostgreSQL hosted/production rules

The PostgreSQL adapter uses exact schema/migration identity, bounded connection pools, statement/lock timeouts and the isolation/locking required by each repository transition.

Production-sensitive races are proven against real PostgreSQL, including:

- concurrent first login for one external subject;
- external-identity link uniqueness;
- account/profile expected-revision conflicts;
- session revoke/read admission ordering;
- duplicate idempotency command handling;
- migration/startup exclusion;
- backup during bounded operation and restore read-back.

PostgreSQL `jsonb`, partial indexes, row locks and backend-specific constraints are permitted adapter details. They do not leak into public domain values or force SQLite to pretend those features exist.

## 6. Schema and migration strategy

The two backends have separate migration trees and SQL files. They share one semantic schema release identifier and an explicit compatibility table rather than one lowest-common-denominator SQL script.

Each migration declares:

```text
semantic_schema_version
backend
from_version
to_version
forward step
rollback/restore policy
required preflight
post-migration invariants
```

Rules:

- application startup never improvises an unregistered schema;
- static/resolved/live-readonly config smoke performs no migration;
- migration failure leaves readiness false and does not acknowledge partial completion;
- destructive or irreversible migration requires backup/restore evidence and an explicit compatibility decision;
- stored enum/tag/value reinterpretation requires the owning domain contract to change first;
- SQLite and PostgreSQL fixtures are generated from backend-neutral semantic cases, not copied production rows.

## 7. Profile representation

The profile source of truth is the extensible fact/proposal/history model. Common user-visible fields may be materialized into a fixed current projection for indexed reads.

```text
canonical fact value
  SQLite: validated canonical JSON text
  PostgreSQL: equivalent validated jsonb
  domain digest: computed from one backend-neutral canonical encoding
```

Database JSON comparison/order behavior is not domain equality. The adapter decodes and validates before returning a domain value. Sensitive values are excluded from unrestricted indexes, logs, diagnostic dumps and telemetry.

## 8. Configuration and secrets

Typed configuration names the exact profile, backend, connection reference, migration policy, pool/timeouts and SQLite path policy. Database credentials use `SecretRef`; a URL with embedded secrets is never printed in effective config, errors or evidence.

`local-demo` is the only profile that defaults a database. `hosted` and `production` require an explicit PostgreSQL secret reference and fail closed on unknown keys/profile/backend combinations.

The development identity adapter and SQLite are both rejected by `production`; selecting PostgreSQL alone does not make a deployment production-ready.

## 9. Backup, restore and evidence

SQLite demo backup uses a consistent database snapshot/backup mechanism, not blind copying during writes. PostgreSQL uses a reviewed logical or physical backup path appropriate to the deployment profile.

Recoverability evidence binds source revision, binary/config/schema version, backend/version, backup identity/digest, restore target and application-level read-back. A backup without restore and semantic read-back is non-pass.

## 10. Verification

Before implementation admission, active rows and bindings must cover:

- typed profile/backend admission and no fallback;
- common repository conformance on SQLite and PostgreSQL;
- SQLite pragma/path/restart behavior;
- PostgreSQL production concurrency/isolation;
- independent migrations with one semantic version map;
- cross-backend canonical profile digest/projection agreement;
- live-readonly smoke non-mutation;
- secret redaction;
- backup/restore/read-back;
- unsupported SQLite semantics fail explicitly.

Catalog-only `STORAGE-001`–`STORAGE-010` preserve these obligations. They remain non-pass until promoted to `docs/acceptance/matrix.tsv` with exact evidence.
