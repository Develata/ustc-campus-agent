# M71 Affairs Navigator — bounded query and publication foundation

Status: `partial-evidence`; **not** a complete `PROC-011` product path.

This crate implements the accepted M71 query algebra plus a bounded
`DemoReviewed` draft → administrator review → atomic publication foundation.
It is retained supporting code for the three-plugin MVP, but it does not claim
production persistence or M00 administrator authentication. The sibling
`ustc-agentd` composition now publishes a controlled fixture through this
foundation and queries the same in-memory repository through M10/Web.

## Implemented

1. Checked M71 nominal/value types and canonical `ProcedureArtifact` validation.
2. Bitemporal evidence, provenance, freshness, conflict and uncertainty algebra.
3. Deterministic six-outcome `AffairsGetService` lookup with sealed evidence
   lineage and safe public projection.
4. `M60ProcedureEvidencePort` query-time retained-evidence verification and
   `M60ProcedurePublicationPort`, which returns source health plus retained
   evidence as one coherent M60-owned publication decision.
5. `ProcedureDraft::from_demo_reviewed`, which imports an exact M60-owned
   `SourceRevision`, requires `DemoReviewed` provenance, binds every local
   assessment to that revision and rejects uncertain/conflicting draft evidence.
6. `ProcedureReviewApproval`, bound to the exact deterministic draft digest.
   Construction is not authentication or authorization; M00/M10 must authorize
   the actor before calling the service.
7. `ProcedurePublicationService`, which requests that coherent M60 decision
   immediately before persistence, enforces chronology, derives stable
   artifact/receipt IDs and mints a private atomic repository commit.
8. `InMemoryPublishedAffairsRepository`, with explicit procedure/artifact caps,
   CAS publication revisions, immutable receipt/artifact tombstones,
   idempotent replay after later revisions or source revocation, corruption
   detection on replay and fail-before-mutation behavior.
9. The original fixture-seeded `InMemoryAffairsRepository` remains only for
   query-kernel tests; the retained `ustc-agentd` path uses the publication
   repository for both startup publication and subsequent lookup.

## Authority and dependency boundary

The crate depends on `time`, `sha2`, and the M60-owned immutable source-revision
value carrier in `ustc-campus-agent-core`. It does **not** depend on M10, M80,
client, or storage crates. A compile-fail doctest proves the client crate cannot
be imported.

A public enum value such as `SourceRevisionHealth::Current` supplied by a caller
is not publication authority. `M60ProcedurePublicationPort` must derive health
and retained revision/digest/revocation verification from one coherent M60 read;
M71 consumes only the combined decision. After a command is committed, its
receipt+artifact tombstone owns exact replay even if mutable source authority
later changes; a failed/uncommitted attempt always obtains a fresh M60 decision.

## Honest nonclaims

- No live USTC retrieval, parser, source approval or legal permission claim.
- `DemoReviewed` snapshots are non-personal demo evidence, not real-time official
  publication.
- No M00 administrator authentication/authorization composition yet.
- No durable source/publication/profile store or restart recovery yet.
- No M00-authorized operator publication command/API or Agent/ToolGateway route;
  current application/Web composition publishes only during controlled fixture
  bootstrap.
- No supersede/archive command journal or structured-search product route yet.
- The M60 fixture adapter is test evidence, not production M60 authority.
- `PROC-011` remains non-pass until M00-authorized publication, durable restart
  and the remaining temporal/failure assertions exist.

## Gate commands

```bash
cargo fmt --all -- --check
cargo test --locked -p affairs-navigator --all-features
cargo test --locked -p affairs-navigator --doc
cargo clippy --locked -p affairs-navigator --all-targets --all-features -- -D warnings
cargo check --locked --workspace --all-targets --all-features
```

## Crate structure

```text
src/
  value.rs            checked M71 nominal values
  evidence.rs         revision refs, authority, bitemporal/conflict algebra
  artifact.rs         procedure content and publication state
  publication.rs      reviewed draft, approval, atomic publication service/repository
  m60_port.rs         retained-evidence verification boundary
  m60_fixture.rs      equal-contract in-memory M60 fixture
  repository.rs       fixture-seeded query repository
  service.rs          six-outcome query service
  application_port.rs query application port
  public_view.rs      safe public projection
  lineage.rs          sealed evidence-lineage receipt
  outcome.rs          query/outcome/error algebra
  clock.rs            query clock boundary
```
