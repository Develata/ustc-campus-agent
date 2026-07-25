# M90 — Platform Infrastructure and Operations

## Metadata

- `Module ID`: `M90`
- `Status`: Accepted blueprint; repository CI/checker baseline exists, production infrastructure planned
- `Implementation State`: `governance-baseline`
- `Version`: `m90-infrastructure/v1`
- `Last Review`: `2026-07-25`
- `Owning Governance`: [`../08-security-and-delivery.md`](../08-security-and-delivery.md)
- `Primary code areas`: `crates/adapters/`, deployment/config modules and Docker Compose profile, `.github/`, `scripts/`

## 1. Purpose

`M90` supplies replaceable implementations of infrastructure ports declared by other modules: durable repositories/journals, immutable evidence/artifacts, transactions, clock/scheduler/queue, secret references, safe HTTP, configuration, telemetry, deployment and recovery tooling.

It makes domain rules durable and operable. It does not define those rules.

## 2. Non-goals

- deciding package, grant, run, source or product transitions;
- exposing database/cloud/container administration through public APIs;
- treating logs/caches/queues as canonical truth;
- selecting business fallback based on infrastructure convenience;
- becoming one unbounded “adapter” crate with every dependency enabled;
- claiming production readiness from CI alone.

## 3. Owned objects and state

```text
TypedRuntimeConfig / ConfigVersion
Repository transaction/checkpoint implementation state
Journal/EventCursor storage
EvidenceObject/ArtifactLocation and retention state
QueueJob/Lease implementation state
SecretRef metadata and rotation/deletion state
Telemetry schema and retention
DeploymentProfile / FullstackServerArtifact / ComposeServiceGraph
Migration / Backup / Restore / Web-Android read-back evidence
```

The semantic meaning of stored rows/events belongs to the module that declared the port/schema.

## 4. Public inputs and outputs

Infrastructure implements explicit ports such as:

```text
SessionRepository / InstallationRepository / GrantRepository
RunJournal / EventSubscription
SourceRepository / EvidenceStore / ArtifactStore
Clock / Scheduler / Lease / Queue
SecretResolver
SafeHttpClient
TelemetrySink
ConfigLoader / MigrationRunner / BackupRestore
```

Each port has a fake/in-memory implementation owned with its domain contract and one or more production adapters under `M90`.

## 5. Dependency direction

`M90` may depend on public port/value contracts from `M00`, `M20`, `M30`, `M40`, `M50`, `M51`, `M60`, `M70`, `M71` and `M72`. Those domain modules depend on interfaces, not concrete `M90` implementations.

Forbidden:

- domain code importing SQL rows, ORM state, queue messages, provider clients or deployment handles as authority;
- one adapter reaching into another module's private state;
- Dioxus/client access to infrastructure ports;
- cyclic adapter/domain dependencies;
- broad feature flags/dependencies forced into unrelated modules.

When storage, secret, source fetch or hosted execution gains independent privilege/deployment/failure boundaries, it may be extracted into a separate large module after explicit review.

## 6. Lifecycle

```text
config parse
→ read-only preflight/doctor
→ dependency readiness
→ migration under exact version and rollback policy
→ attach exact Fullstack server artifact, Web assets/SSR and admitted endpoints
→ service start/readiness
→ bounded operation and telemetry
→ graceful drain/shutdown
→ backup/restore/retention/cleanup
→ upgrade/rollback
```

A deployment profile may change scale or adapter placement, not domain authority.

## 7. Failure and recovery

- Invalid/unknown config: fail before service mutation.
- Migration failure: retain/restore prior compatible state; no partial readiness.
- Repository transaction uncertainty: domain operation remains unacknowledged and reconciles by stable identity.
- Queue duplicate/loss: owning idempotency/event journal protects semantic state.
- Evidence/artifact failure: no success acknowledgement where evidence is required.
- Cache/search loss: rebuild from canonical state.
- Secret unavailable/revoked: typed blocked state, no fallback credential.
- Backup without restore proof: not accepted as recoverability.
- Telemetry failure: bounded degradation; never block safety decisions or leak payloads.
- Disk/resource exhaustion: explicit backpressure/readiness failure and retention policy, not silent corruption.

## 8. Configuration and secrets

Configuration is typed, versioned and divided by module/adapter. Startup preflight validates paths, listener/public origins, Android HTTPS server origin policy, Compose services/volumes, limits, ownership and secret references without performing product mutations. Secrets are resolved at the narrowest adapter and redacted from config dumps, logs, events and receipts.

## 9. Observability

Stable telemetry includes service/build/config/schema versions, readiness, repository/journal/queue/evidence latency, retries/conflicts, resource ceilings, backup/restore and migration results. Content telemetry is off by default. Audit/event semantics remain owned by domain modules.

## 10. Extension and replacement

Database, object store, queue, HTTP client, secret manager, telemetry stack and deployment profile are independent peers behind ports. A local in-memory/demo adapter and a production adapter must obey equal semantic contracts. Infrastructure replacement must not require domain state-machine changes.

## 11. Performance path

Critical paths are transactional authority reads/writes, append-only journals, event fan-out, evidence storage and source/provider HTTP. Use indexed stable IDs, bounded batches, backpressure and explicit time/size/concurrency limits. Optimize only after correctness and recovery evidence.

## 12. Scope boundary

**MVP**

- typed config and doctor/preflight;
- one durable operational store with migrations/transactions;
- run/source/event journals and bounded subscriptions;
- immutable local or reviewed evidence/artifact store;
- secret references and redaction;
- safe HTTP client for admitted external origins;
- CI/contracts, backup and real restore smoke;
- one Docker Compose profile that runs the exact native Dioxus Fullstack server and durable dependencies, serves Web assets/SSR, exposes admitted HTTPS server-function/stream endpoints and passes Android remote read-back.

**Later**

- production managed database/object store/queue peers;
- richer telemetry and retention;
- horizontally scaled workers after lease/idempotency proof;
- isolated hosted execution only after separate GO/NO-GO review.

**Explicit non-goals**

- public infrastructure-admin API;
- Kubernetes/container platform as product ontology;
- cache/queue/search as authoritative state;
- multi-cloud symmetry before one deployment is restorable.

## 13. Small-module decomposition

1. `config` — typed schema, load, preflight and redacted diagnostics.
2. `storage` — repository/transaction adapters grouped by semantic database boundary.
3. `journal` — append/replay/event subscription.
4. `evidence-store` — immutable objects, digest/read-back and retention.
5. `clock-scheduler` — monotone clock, deadlines and bounded scheduled work.
6. `lease-queue` — concurrency, delivery and duplicate-safe worker plumbing.
7. `secret-ref` — resolution, rotation, revoke/delete and redaction.
8. `safe-http` — fixed-origin/SSRF/redirect/content/time/size policy.
9. `telemetry` — stable metrics/log/trace schema and payload exclusions.
10. `migration` — schema version, forward/rollback compatibility.
11. `backup-restore` — real read-back and recovery proof.
12. `deployment-profile` — Docker Compose Fullstack server/Web/Android endpoint wiring plus later staging/central/single-tenant peers.
13. `ci-contracts` — build/test/docs/schema/dependency gates.

Do not place all items in one source file or dependency-heavy crate. Keep adapter dependencies local to the small module that uses them.

## 14. Exit gate

`M90` is integration-ready when each selected production adapter passes the same port conformance as its fake, including failure injection, duplicate/restart and redaction. The required Docker Compose Fullstack profile is accepted only after clean startup, migration, Web asset/SSR and server-function readiness, Android remote access, bounded operation, backup, restore, restart and read-back smoke on the actual target surface.
