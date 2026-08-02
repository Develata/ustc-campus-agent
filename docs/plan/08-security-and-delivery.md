# Security, delivery and public transition

## Metadata

- `Layer`: Cross-cutting governance
- `Status`: Current security/publication rules; production deployment planned
- `Version`: `0.3.0`
- `Last Review`: `2026-08-02`
- `Authority Owns`: credential/data boundaries, release/publication gates, deployment-profile invariants
- `Authority Defers To`: owning product/source/runtime plans and explicit Develata publication approval
- `Counterpart Feature`: `docs/features/00-market-browse-install.md`
- `Counterpart Contracts`: `docs/contracts/platform-account.md`, `docs/contracts/user-context-profile.md`, `docs/contracts/storage-profiles.md`, `docs/contracts/permissions.md`, `docs/contracts/source-import.md`
- `Counterpart Acceptance`: `PUBLIC-*`, release-gated security cases
- `Primary Code Areas`: repository-wide, future deployment/auth/runtime modules
- `Large-module Counterpart`: [`M90 Platform Infrastructure and Operations`](modules/90-infrastructure-operations.md); security remains a cross-cutting gate rather than one module that absorbs all security rules

## 1. Security baseline

The project MUST NOT commit or expose:

- USTC passwords, CAS cookies/sessions or raw authentication secrets;
- API/model/provider keys or private keys;
- real student academic/profile data in fixtures, logs, screenshots or reports;
- unredacted tool/model payloads containing private data;
- source snapshots whose storage/publication is not approved;
- private infrastructure endpoints or personal operational procedures.

Secrets use runtime secret references. Catalog manifests, docs, normal logs and audit evidence contain identifiers and redacted metadata only.

## 2. Authorization and data isolation

- Every invocation revalidates user/session/tenant, installation, enabled state, component identity and capability grant.
- External authentication resolves canonical provider subject through an explicit M00 account-link decision; email, telephone, school-number alias, profile text and AI output never auto-merge accounts.
- Account status, tenant membership, administrator role/grant, service principal and general profile are distinct authority classes.
- Default Plugins receive only explicitly auto-grant-eligible public read/link-out capabilities.
- Tenant-private reads/writes are tenant/user/purpose/consumer/field bounded, consent-aware and never cross-user. M30 and M72 consume minimum M00 profile projections, not repositories or raw credentials.
- Administrative publication, source ingestion and operator diagnostics are separate authority classes from user Plugin invocation.
- Arbitrary shell, PATH search, host filesystem/device/socket access and unrestricted network access are outside the MVP.

## 3. Source and content safety

- Only reviewed sources may be fetched by privileged infrastructure.
- iCourse remains link-out-only unless an explicit data-use contract permits more.
- External pages, tool output and model output are untrusted input.
- Markdown/JSON/path handling rejects traversal, symlink escape, hidden-control hazards and unknown schema fields where the owning contract requires it.
- Public/approved facts, M00 general profile data and product-owned private preferences remain separate projections.

## 4. Deployment profiles

The same authority contracts apply to central, staging, demo and optional single-tenant deployments. `local-demo` defaults to SQLite with explicit single-host/low-concurrency limits. Hosted and production require PostgreSQL and MUST NOT fall back to SQLite. A deployment profile MAY change resource sizing or adapter location; it MUST NOT create a second account/profile/package/grant/source authority, weaken tenant-scoped types or expose backend row/JSON shape as a public/domain contract.

Production-like deployment eventually requires:

- only reviewed HTTPS user surfaces exposed;
- typed configuration and read-only preflight/doctor checks;
- low-privilege bounded workers for external execution;
- common SQLite/PostgreSQL repository conformance plus PostgreSQL production concurrency/isolation and backend-correct database/evidence-store restore tests;
- environment/worktree/credential separation;
- bounded logs, caches, images and evidence retention.

These are planned requirements, not current deployment claims.

## 5. Release gate

A release requires, at minimum:

1. required PR/integration/demo acceptance cases pass;
2. exact source revision and artifact identity recorded;
3. clean-host restore or deployment read-back succeeds where applicable;
4. artifact checksum/version/smoke is verified from the delivery surface;
5. licenses/notices and source/data permissions are complete;
6. rollback target and operator recovery are tested;
7. no unresolved blocker review finding remains.

Local build success is not remote release success.

## 6. Public transition

Repository visibility and GitHub Pages/download publication are explicit security/release decisions. Before public visibility:

- select the license and complete third-party notices;
- scan the full reachable Git history for secrets/private data;
- scrub or approve every fixture and screenshot;
- verify USTC/iCourse data-use boundaries;
- display the non-official disclaimer;
- ensure Pages contains no fabricated metrics, affiliations, testimonials or downloads;
- point download links only to verified release assets;
- pass `docs/acceptance/public-readiness.md`.

## 7. Failure and recovery

- Secret/privacy uncertainty blocks publication.
- Missing audit/evidence blocks success acknowledgement for durable mutation.
- Permission or tenant ambiguity blocks invocation.
- External-subject/link conflict, profile purpose/sensitivity ambiguity or unsupported storage profile/backend pair blocks admission without payload read or fallback.
- Restore/read-back failure blocks release.
- Source permission withdrawal suspends new ingestion and preserves only policy-compliant historical evidence.
- Security revoke blocks new invocation before convenience or availability concerns.

## 8. Verification

- `python3 scripts/check_repo_contracts.py`
- repository history/secret audit before public/release changes
- `docs/acceptance/gates.md`
- `docs/acceptance/public-readiness.md`
- exact target-host/browser/artifact checks when those surfaces exist
