# M71 `affairs.get` query kernel — proposal-only spike

Status: `READY_TO_DISPATCH_PROPOSAL_ONLY_NOT_ACCEPTED`

This crate is **proposal-only, compile-tested evidence** for the frozen M71
public algebra (M71-v8n) and the ratified TD-2 retained-evidence seam. It is
**not retained/accepted implementation**. This spike authorizes no merge or
status promotion.

## What is implemented

1. **Checked M71 value/public DTO types** with NO Serde (§11.1): `ProcedureId`,
   `ArtifactId`, `BoardId`, `SourceId`, `ContactRef`, `MaterializationReceiptId`,
   `ActorRef`, `Title`, `AudienceTag`, `PrerequisiteCondition`, `Instruction`,
   `DeadlineLabel`, `EntryPointLabel`, `ContactName`, `ContactChannel`, `Url`,
   `BoardPolicyVersion`, `EffectiveInterval`.
2. **Evidence algebra**: `M60RevisionRef` (equal-contract fake, D8 split),
   `AffairsAuthority` (4-tier), `AuthoritySubject` (8-variant),
   `AffairsEvidenceAssessment`, `ProcedureEvidenceContext`,
   `ValidityHorizon` + `derive_valid_interval()`, conflict/uncertainty states.
3. **Canonical procedure content**: `ProcedureArtifact` with full cross-field
   validation, `BoardPolicy`, `Prerequisite`, `ProcedureStep`, `Deadline`,
   `EntryPoint`, `Contact`, `ProcedurePublicationState`.
4. **Deterministic six-outcome lookup kernel** (`AffairsGetService`): Found,
   NotYetKnown, Archived, NotFound, Conflict, CannotVerify — in the frozen
   lookup order.
5. **Conflict-before-projection and bounded deterministic public evidence
   projection**: coalesce by `(authority, source_id, subject)` → mandatory
   groups (3 clauses) → overflow check (>8 mandatory → CannotVerify) → 8-slot
   selection → Complete/Truncated metadata. `selection_rule_version = 2`.
6. **Retained M60 verification port** (`M60ProcedureEvidencePort`) and an
   in-memory equal-contract M60 fixture adapter (`M60FixtureAdapter`).
7. **In-memory repository** (`InMemoryAffairsRepository`) seeded through
   checked fixture constructors.
8. **Sealed M71 evidence-lineage receipt** (`M71EvidenceLineage`): Verified /
   Unverified / NotRequired with exhaustive outcome/lineage pairing.
9. **Tests**: 42 unit tests + 44 integration tests covering all six outcomes,
   all four CannotVerify reasons, Fresh/Stale bounds, all three ConflictKind
   variants, 1/8/9/16 projection boundaries, outcome/lineage pairing closure,
   no-M60-call for NotRequired outcomes, public projection safety (no raw
   revision/digest/actor bytes in Debug), determinism, and a compile-fail
   doctest proving no M10/client/storage dependency. The
   `tests/hardening_counterexamples.rs` suite pins five adversarial
   fail-closed/determinism invariants: declared `valid_interval` must equal
   `derive_valid_interval` over the assessments; permuted equivalent evidence
   must yield byte-identical receipts (canonical retained-reference order);
   caller-provided `as_of` receipts must not depend on the wall clock;
   `M60VerificationIdentity` enforces the M71 ID grammar on `verifier_id`; and
   an incoherent repository pairing fails closed with `InternalInconsistent`.

## No-bypass contract (TD-2)

This crate depends **only** on `time` and `sha2` (workspace deps). There is no
dependency on M10, M80, client, storage, or any other product crate. The M71
application service is the sole caller of `M60ProcedureEvidencePort`; M10
cannot bypass M71 to reach M60.

This is structurally enforced by the Cargo dependency graph: the crate's
`Cargo.toml` lists only `time` and `sha2`, so no `use` statement can name an
M10/client/storage type. A `compile_fail` doctest in `src/lib.rs` attempts to
`use ustc_campus_agent_core` — it MUST fail to compile, and that failure IS the
no-bypass proof.

## Proposal nonclaims

- This is **not** retained/accepted implementation.
- The in-memory M60 fixture adapter is **equal-contract fixture evidence**, not
  accepted M60 implementation.
- No publish/supersede/archive command/event journals are implemented beyond
  the minimal checked fixture seeding needed to exercise already-published /
  archived states.
- No M10 wire/reconciliation is implemented.
- No M80 UI is implemented.
- No live campus source fetching is implemented.
- This spike authorizes no merge, status promotion, or remote operations.

## Gate commands

```bash
export CARGO_TARGET_DIR=/home/pwh/.cache/uca-cargo-target/m71-6e138cf5

cargo fmt --all -- --check
cargo test -p affairs-navigator --all-features
cargo clippy -p affairs-navigator --all-targets --all-features -- -D warnings
cargo check --workspace --all-targets --all-features
```

## Crate structure

```
src/
  lib.rs              — module declarations, re-exports, compile-fail no-bypass doctest
  value.rs            — checked M71 nominal value types (ID grammar, UTF-8 bounds)
  evidence.rs         — M60RevisionRef, AffairsAuthority, evidence assessments, ValidityHorizon
  artifact.rs         — ProcedureArtifact with full cross-field validation
  public_view.rs      — PublicProcedureView, PublicEvidenceView, Freshness, ConflictDetail
  projection.rs       — coalesce → mandatory → overflow → 8-slot selection (internal)
  outcome.rs          — six-outcome ladder, CannotVerifyReason, GetProcedureError, AffairsGetQuery
  m60_port.rs         — M60ProcedureEvidencePort trait + request/outcome/identity types
  m60_fixture.rs      — in-memory M60 fixture adapter (equal-contract)
  lineage.rs          — sealed M71EvidenceLineage receipt
  repository.rs       — in-memory affairs repository
  clock.rs            — AffairsClock trait + FixedClock
  service.rs          — AffairsGetService lookup ladder + M71AffairsGetReceipt
tests/
  common/mod.rs       — shared fixtures
  outcome_lineage_pairing.rs — exhaustive pairing table + no-M60-call proofs
  projection_boundaries.rs   — 1/8/9/16 Complete/Truncated/overflow boundaries
  no_bypass.rs               — public API no-bypass proof
  public_projection_safety.rs — no raw bytes in Debug output
  determinism.rs             — same input → byte-identical receipt
```
