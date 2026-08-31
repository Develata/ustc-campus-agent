# W1 M20-B6 update/rollback exact-contract readiness

## Authority

- `Campaign ID`: `USTC-MODULES-2026-07-W1`
- `Lane`: `M20-B6`
- `Grant carrier`: [`01-execution-roadmap.md`](01-execution-roadmap.md#active-autonomous-module-campaign-authorization)
- `Mode`: operation-specific finite retained implementation grant; no acceptance-status promotion, B7 composition or M90 production persistence

## Mutable campaign state

- `Status`: `completed`
- `Bound source commit`: `5e9e5b934cd7e6ea50db1787ebd35949394f1052`
- `Bound source role`: `historical exact implementation input; not the terminal main integration receipt`
- `Terminal main commit`: `2f4de29032560ff3e13d9994b33a3aff14243f44`
- `Repair round`: `2`
- `Current blocker identity`: `none`
- `Stop reason`: `none`
- `Last transition evidence`: `accepted-delimited-packet-sha256:3bfde89d1915d525ded7155c8c324b99c39146c69c636165368f2ba3e5ef1d10; develata-whole-packet-accepted-2026-08-01; proposal-pr-32-merged-squash:5e9e5b934cd7e6ea50db1787ebd35949394f1052; merged-tree:92b2347ac678bebf4c55e6423965219f8e53765a; post-main-ci:30700279228=PASS; develata-operation-specific-finite-implementation-grant-2026-08-01; pre-edit-analysis-batch:deleg_a7b08aba=two-advisory-GO; parent-verification=proof-carrier-and-ledger-seed-clarifications-required; implementation-grant-sha256:7f48912254d363cea7e51f2abb1f2f7e2d2fb09cfc0e71107b86bc97580adab2=superseded-by-developata-authorized-proof-carrier-scope-amendment; implementation-grant-sha256:204ca6856f0264fa130c2e92919b5c00b25a3b3518371c048e29ecd1b2340a8f; pre-edit-semantic-review-sha256:4ae6140f94acd3ef47676e9e7cd8cbf8a6a145b02b6d55044764ed54b38f0e42=GO; pre-edit-path-b-review-sha256:b8b599f5f498dd327d98193f06b64352cfcefb308c9f3fd163cc90635bec2b78=GO; pre-edit-checker-review-sha256:8294ff4f6c4f78d36499d950ee5f40a20e36fab5a046e9cc7b86fc14d6372e66=GO; implementation-prompt-v1-sha256:c1381f89f2726984d6768dc3b4807895555903e17244078cdcff2c5d2cc49e47=BLOCKED_VERIFICATION; implementation-prompt-v2-sha256:4e1fa4a693c2731ddff50d5d6417255e01f94b80502acaef900e7cc893d905da; prompt-rereview-log-sha256:539a99087d7e846d55464085d0f8440f51be96acb39b8ac6faaed23b167247a3=GO; pre-edit-rereview=GO; implementation-repair-r1:blocker=grant-authority-conflict-rejected-receipt-not-reseedable-after-competing-grant-revoke; parent-procedure-note=repair-state-row-recorded-immediately-after-local-repair-not-before; repair=typed-private-repository-rejection-reseed-plus-historical-revoke-regression; targeted-grant-test=PASS; grant-unit-20=PASS; grant-integration-5=PASS; grant-owner-review-sha256:1af2e52f1df5f985f9c7a09eaa18199439ac642faa48a938f81a731761c8035c=BLOCKED; implementation-repair-r2:blockers=canonical-authority-tuple-order+repository-rejection-observed-prefix-proof; grant-r2-rereview-sha256:62705506d51d9f1b3342224db221fe3da4e7baf502666a116b514c25c5b03235=BLOCKED_STALE_AUTHORITY_PREFIX; canonical-order=closed; remaining=ordered-current-authority-prefix-index; grant-r2-final-rereview-log-sha256:a5ccc71bb6049ea36fe1a5f2f3d6a299dd6a9909609bed175e0ee3ccbfc4b337=GO; grant-round2-closed=canonical-order+ordered-repository-prefix-proof; grant-unit-20=PASS; grant-integration-5=PASS; source-control=local-only-pending-separate-ship-decision; implementation PR #33 reviewed head 9290d5c4ddb5458ab50e62a4739bea23c3d01c8e; squash/main 2f4de29032560ff3e13d9994b33a3aff14243f44; reviewed-tree-equals-main-tree 53e266c47fdb07d50a734faa24bb11ac4bc5527d; exact-main CI 30741450134=PASS; W1 M20-B6 terminal disposition completed with MARKET-004/PKG-020 and B7/durable-adapter non-goals preserved`
- `Next allowed mutation`: `none within USTC-MODULES-2026-07-W1; B7 composition, durable adapters, artifact switching or production persistence require a separate accepted slice`
- `U2 prefix evidence`: `update-unit-16/16=PASS; installation-grant-integration-16/16=PASS; cargo-fmt=PASS; git-diff-check=PASS; policy-carriers=canonical-complete-typed; replay-forgery=fail-closed; parent-final-byte-review=GO`
- `U3 implementation completion evidence`: `parent-verified implementation completion, not an independent review: exact 21 market::update unit tests; 3 market_package_update public integration tests; installation/grant owner regressions; repository atomic failure/retry, exact ledgers/indexes/consumed approvals, positive rebuild controls and isolated unreferenced PackageUpdated, PackageRolledBack and update-prefixed Grant MarkedStale bijection negatives; all seven owner-only UpdateDecisionContext constructor families are production-reachable and synthetic authority builders are cfg(test)-only`
- `U4 governance projection evidence`: `parent-verified truthful docs projection: B6 bounded evidence complete; M20 remains partial-evidence; MARKET-004 and PKG-020 remain planned; B7 composition, durable adapters, current API/UI, in-flight/current-call composition, crash recovery and artifact switching remain future`
- `U5 checker closure evidence`: `contract-check PASS; exact source/module/public/impl/derive/attribute/macro/test inventories; full-name owner-only restricted-function inventories for update, installation and grant; all seven production context constructor calls pinned exactly once with no extra named context literal; exact InstallationEventKind variants; semantic cfg(test) adjacency for synthetic authority constructors; 12 mandatory B6 mutation methods PASS; no acceptance status promotion`
- `Repair chronology`: `U1 accepted delimited packet and pre-edit reviews; U2 repaired grant authority conflict by typed private repository rejection reseed plus historical revoke regression, then repaired canonical authority tuple order and ordered current-authority prefix proof; U3 completed atomic package-update repository and evidence in parent repair lane under frozen repair round 2, then strengthened five existing semantic tests without adding or renaming tests; parent repair added positive owner-ledger rebuild controls around three isolated cross-stream bijection negatives; parent repair replaced direct/private context construction with all seven owner-only constructor families, passed complete typed catalog publications into Stage, RecordApproval, Apply and Rollback, and excluded synthetic authority builders from production with cfg(test); U5 repaired checker prefix-only blindness by pinning full restricted function names and installation event variants plus mutation bites; full-suite discovery repaired three stale mutation-test assumptions for dead-code attribute uniqueness, installation-module adjacency and cross-file binding cardinality without weakening their bites; final all-target clippy repair replaced fourteen grant test unwraps with explicit fixture expectations; independent review then exposed owner/update disabled-state predicate drift: update admitted both InstalledDisabled and Disabled while owner decide/replay admitted only Disabled; scoped repair unified owner decide/replay on a private two-state predicate and strengthened existing owner plus semantic-repository Apply/Rollback regressions without adding test functions; the next exact-candidate reviews found that repository Apply/Rollback staged owner events before the pure disabled-state admission, that opaque installation references were not bound back to the update plan's exact prior/next pins, and that accepted current-grant-set witnesses were not replay-bound to owner prefixes or checked for omitted active old-revision authority; scoped repair extracted shared pure admission and persisted a typed coupled-preflight rejection before owner staging, added restricted boolean-only owner/canonical verifiers, replay-bound accepted receipt witnesses and rejected coupled-preflight witnesses to exact owner history prefixes and repository authority carriers, rejected omitted active old-revision grants, and added wrong-but-valid owner-event, omitted-active-grant, and missing-rejected-owner-prefix mutation regressions inside existing tests; one checker review hash claim was rejected because it used LF-delimited paths rather than the frozen path+NUL+u64be+bytes manifest algorithm; governance projects bounded truth only and invents no independent review`
- `Authorized scope amendment`: `Develata explicitly authorized crates/platform-core/tests/platform_identity.rs on 2026-08-02 solely to project the existing market.rs module declaration mirrors for pub mod update; no identity behavior, public surface or runtime authority changed; prior implementation-grant digest is superseded by the digest recorded in Last transition evidence`

## Output contract

Implement the accepted packet as one bounded `M20-B6` Rust domain/semantic-fake slice on exact merged `main`, including deterministic update decision/replay, owner-reachable installation/grant events, complete-current grant-set semantics, atomic in-memory cross-stream commit and honest docs/checker evidence. Preserve the accepted packet bytes and keep `MARKET-004`/`PKG-020` planned. The source-control boundary was later crossed under reviewed campaign PR #33; no further M20-B6 mutation is admitted by W1. Any retained implementation-grant chronology saying `source-control shipping remains closed` is historical pre-PR packet context, superseded for mutable status by the verified PR #33 terminal receipt above; it grants no current mutation authority.

## Required evidence

- exact merged source commit/tree, clean isolated worktree and post-main CI receipt;
- separately delimited and hashed finite implementation grant with exact writable paths, public/private/import/test/mutation/manual-`Debug` inventories;
- independent PRE_EDIT_TASKBOOK review before Rust mutation;
- focused and full Rust/docs/checker gates on the final candidate;
- independent lifecycle/atomicity/replay, authority/evidence and checker/status-honesty reviews bound to the final tree;
- every implementation repair round and blocker identity recorded above before another mutation.

<!-- M20_B6_READINESS_PACKET:BEGIN -->
## 1. Proposal status and selected posture

Everything between the packet markers is **proposed, not accepted authority**. Existing accepted plans, contracts, Rust types and acceptance statuses remain authoritative until a separately authorized contract amendment is merged.

Develata selected the Conservative MVP posture and the remaining Axes A–C on `2026-08-01`:

1. every update requires one explicit exact-plan approval;
2. Apply is legal only while the installation is `InstalledDisabled` or `Disabled`;
3. Apply changes the exact installed package pin atomically but does not Enable the installation;
4. rollback restores the exact prior package pin and remains Disabled;
5. rollback is legal only from `AppliedPendingConfirmation`; `ConfirmAppliedUpdate` closes the window, enters terminal `Confirmed` and releases the current-update slot; any later revert is a fresh reverse update with a new plan and fresh update approval;
6. Apply and Rollback atomically mark every transaction-current Active grant `Stale(InstallationChanged)`; Stale/Expired grants are unchanged and Revoked grants remain history-only;
7. reactivation always consumes fresh grant approval/evidence bound to the new pin and installation revision, including for an unchanged capability; same-scope continuity uses `Replace`, while scope change or addition uses fresh `Issue` and a removed capability receives no replacement;
8. `ConfirmAppliedUpdate` and `Enable` do not bind runtime health; health/availability is a separate non-authoritative runtime observation and never a sixth installation state;
9. no enabled-to-enabled switch, automatic patch rollout, canary promotion, automatic grant restoration, post-confirm rollback anchor or automatic restoration of Enabled state enters B6.

This posture selection authorizes the readiness proposal to become exact. It does not itself accept the proposed Rust contract, authorize retained implementation or amend the campaign grant.

## 2. Exact source and baseline receipt

| field | receipt |
|---|---|
| repository | `Develata/ustc-campus-agent` |
| authoritative source commit | `993b44229d1ff76a124d5194a00c0b29b7991f7d` |
| authoritative source tree | `44f65329c300a11549e896e9ff48acf233f3a177` |
| source relation | local `origin/main` equalled direct GitHub `refs/heads/main` before checkout creation |
| checkout | dedicated clean worktree; no retained foreign change |
| baseline time | `2026-08-01T04:58:22Z` |
| baseline checker | `python3 scripts/check_repo_contracts.py --ci` -> PASS |
| implementation | none in this proposal lane |

A moving `main` invalidates this source binding before any candidate push. Rebinding means a fresh exact-main checkout, packet review against the new tree and a restarted digest-bound review; a textual merge-base claim is not sufficient.

## 3. Current gap table

| accepted carrier | current rule/evidence | exact B6 gap | proposed closure |
|---|---|---|---|
| `market-lifecycle.md` M20-LC-008 | permission expansion requires reapproval | no package-level complete delta classifier or exact approval binding | `UpdateChangeClass` plus approval evidence bound to the complete plan digest; all classes require explicit approval and `ReapprovalRequired` cannot use an automatic path |
| `market-lifecycle.md` M20-LC-009 | stage/apply/rollback use exact reviewed targets and preserve rollback | no command/state/event/revision algebra or tested rollback evidence | one event-sourced update aggregate with exact source/target pins, readiness evidence, expected revisions and one rollback target retained only through `AppliedPendingConfirmation`; terminal confirmation closes rollback |
| `installation.rs` | one exact package pin, configuration and managed lifecycle | no owned package-pin replacement transition | installation-owned sealed Apply/Rollback package-pin events invoked only by the update transaction; managed state remains Disabled |
| `grant.rs` | grants bind installation revision/package/capability manifest and replacement always consumes fresh approval | package-pin mutation could otherwise leave an Active old grant in the current index or let update approval masquerade as grant approval | accepted Apply/Rollback atomically marks every Active current grant for the installation Stale with `InstallationChanged`; every later `Replace`/`Issue`, including unchanged-capability reactivation, requires fresh B4 approval/evidence |
| `authority.rs` / P0a | frozen projections plus current deny-side recheck | no package update source of current carrier change | B6 mutates only future installation/grant carriers; frozen projections remain byte-equal while current calls fail closed until fresh enable/grants |
| `installation.rs` Enable / M40 runtime | Enable consumes exact M20 authority evidence; runtime availability has another owner | no B6 completion boundary, and a UI may otherwise flatten authority and health | `ConfirmAppliedUpdate` closes update workflow without health evidence; Enable stays separate and authority-only; runtime health remains an independent projection and never changes the five-state installation lifecycle |
| `MARKET-004` | planned; B2/B4 supporting evidence | no update integration proof | proposed focused package-update classifier/approval/atomicity tests; row remains planned until admitted implementation and required later composition evidence |
| `PKG-020` | planned | no update/rollback versus frozen projection proof | proposed frozen-old/new-projection integration fixture; row remains planned pending B7 composition/integration gate |

## 4. Ownership and dependency direction

### 4.1 Proposed owner

A future `crate::market::update` module owns:

- package-update identity, aggregate state and revision;
- source-to-target plan and rollback-target binding;
- complete change classification;
- trusted approval/readiness/confirmation evidence values;
- update commands, events, replay and semantic repository port;
- cross-stream transaction ordering for one update aggregate, its installation and current grants.

`crate::market::installation` remains the sole owner of `InstallationPackagePin`, installation revision and legal installation events. B6 may add sealed package-pin Apply/Rollback actions whose constructors are no wider than `pub(in crate::market)`; external callers cannot replace a pin directly.

`crate::market::grant` remains the sole owner of grant transitions. B6 invokes existing `MarkStale(InstallationChanged)` decisions for the transaction-current complete set of Active grants; it does not forge grant events, reimplement grant replay or reactivate an old grant.

A future M90 adapter implements the semantic transaction. M20 does not import a database, filesystem, package manager, clock, network client, executor or framework type. B7 remains the application/API/composition owner.

### 4.2 No second authority

The update aggregate records one proposed/accepted transition plan and its audit history. It is not a peer installation authority. Current invocation-authority truth remains:

```text
reviewed catalog carrier
+ current InstallationAggregate package pin/state/revision
+ current GrantAggregate states/versions
+ current policy/source/execution admission
```

`UpdateState::AppliedPendingConfirmation` records workflow posture only. It cannot make the target Enabled, runnable or granted.

Runtime availability is not part of that authority expression. A later server projection may report `invocation_authorized` and `runtime_availability` separately, but neither field repairs the other and no client computes either from local state.

## 5. Proposed exact value algebra

All authority-bearing values have private fields, checked constructors, read-only accessors and manually redacted `Debug`. No `Default`, public Serde construction, unchecked constructor, authority boolean or arbitrary string payload is admitted.

### 5.1 Identities and revision

```text
PackageUpdateId      update:[A-Za-z0-9._:-]{1,121}
UpdateCommandId      update-cmd:[A-Za-z0-9._:-]{1,117}
UpdateApprovalId     update-approval:[A-Za-z0-9._:-]{1,112}
UpdateEvidenceId     update-evidence:[A-Za-z0-9._:-]{1,112}
UpdateRevision       update-revision:[1-9][0-9]*
UpdateEventSequence  nonzero u64; first event 1; checked successor
```

`UpdateRevision` is derived only as `update-revision:<event-sequence>`. Externally supplied expected revisions are parsed and compared but never mint a post revision.

One `PackageUpdateId` identifies one attempt from one exact rollback pin to one exact target pin. Retry uses the same command ID only for the byte/typed-equal complete command. A different target, installation, approval or evidence uses a new command identity. A later update attempt uses a new `PackageUpdateId`.

### 5.2 State graph

```text
absence
  └─ Stage ───────────────► Staged
Staged
  ├─ RecordApproval ──────► Ready
  └─ Cancel ──────────────► Cancelled             terminal
Ready
  ├─ Apply ───────────────► AppliedPendingConfirmation
  └─ Cancel ──────────────► Cancelled             terminal
AppliedPendingConfirmation
  ├─ ConfirmAppliedUpdate ► Confirmed             terminal
  ├─ Rollback ────────────► RolledBack            terminal
  └─ CancelAfterTerminalInstallation
                         └► Cancelled             terminal
```

Exact enum:

```rust
pub enum UpdateState {
    Staged,
    Ready,
    AppliedPendingConfirmation,
    Confirmed,
    RolledBack,
    Cancelled,
}
```

Only one update in `Staged`, `Ready` or `AppliedPendingConfirmation` may own the current update slot for an installation. `Rollback` is legal only from `AppliedPendingConfirmation`. `ConfirmAppliedUpdate` moves to terminal `Confirmed`, closes the rollback window and releases the slot. A later revert is a fresh reverse update with a new `PackageUpdateId`, exact plan and approval; B6 retains no post-confirm anchor or rollback stack. Terminal updates retain history. Revoke/Uninstall of the installation always dominates update workflow authority; no update command can revive a terminal installation.

### 5.3 Immutable plan

`PackageUpdatePlan` binds all of:

- `PackageUpdateId`, tenant, user and installation ID;
- exact staged installation revision and configuration revision/digest;
- exact rollback `InstallationPackagePin` copied from the current installation;
- exact target `InstallationPackagePin` from a reviewed, non-revoked catalog revision;
- exact rollback/target catalog authority revisions and non-revocation bindings;
- complete old/new validated package declarations, publisher/tier bindings and their package, component-set, capability-manifest and source-policy digests;
- old/new complete capability definitions under exact registry revisions;
- `UpdateChangeClass` and deterministic `plan_digest`;
- no raw secret, resolved configuration value, artifact bytes or executor handle.

The package ID, tenant, user and installation ID must remain equal. Target package digest/version must differ from the rollback pin. B6 defines no latest-version policy and does not infer a target from SemVer ordering.

### 5.4 Change classification

Exact classifier result:

```rust
pub enum UpdateChangeClass {
    Unchanged,
    Narrowed,
    ReapprovalRequired,
}
```

Classification is computed internally from actual typed old/new declarations and registry definitions; callers never supply it.

`ReapprovalRequired` results if any of these hold:

- a capability is added;
- an old/new shared capability is `ExpansionRequiresReapproval` under `compare_capability_definitions`;
- source-policy digest changes, because B6 has no accepted semantic narrowing order for free-form source policy;
- publisher or package trust tier changes;
- any retained component changes execution identity or component kind;
- any component is added;
- the target requires a broader confirmation/scope binding than the old effective grant set;
- authority classification is incomplete, duplicate or unknown.

`Narrowed` requires no expansion axis and at least one capability/component removal or B2 `Narrowed` capability-definition change. `Unchanged` requires equal capability sets/definitions, equal source-policy digest, equal publisher/tier bindings and equal component ID/kind/execution-identity sets. Component version/artifact digest and package revision may change without changing this authority class, but every class still requires an explicit update approval in the Conservative MVP.

Any capability declared by the target but missing, deprecated/revoked or administratively scoped in the target registry is not classed as a permissible narrowing; Stage fails closed. A capability present only in the rollback package may be a removal and therefore may contribute to `Narrowed`.

### 5.5 Evidence

`UpdateApprovalEvidence` has a minting constructor no wider than `pub(in crate::market)`. It binds `UpdateApprovalId`, exact plan digest, computed change class, exact staged installation revision/configuration digest and one approval-evidence digest. The model, manifest and package publisher cannot mint or request it. One approval ID is consumed by at most one accepted `RecordApproval` event.

`UpdateReadinessEvidence` is independently minted by a trusted admission adapter and binds:

- one `UpdateEvidenceId`;
- plan digest and exact target/rollback package/component digests;
- exact staged installation revision and configuration digest;
- verified target-artifact-set digest;
- verified rollback-artifact-set digest;
- target configuration-admission snapshot digest;
- target source/execution/policy-admission snapshot digest;
- exact catalog and capability-registry authority revisions used by admission;
- evidence digest.

It contains references/digests only, not artifact bytes, test logs, configuration values or secrets. `RecordApproval` carries both coherent approval and readiness evidence; therefore `Ready` means exact human approval and exact target/rollback preparation are both present.

`UpdateConfirmationEvidence` binds the exact update ID/revision, accepted `Applied` event identity/digest and transaction-current installation ID/revision/target pin/state. It proves only that the accepted target pin is still current while the update remains `AppliedPendingConfirmation`; it does not bind grant-set, policy-admission, runtime-health, executor-readiness or callability evidence and cannot Enable the installation. Confirmation may observe either a deliberately Disabled installation or one separately re-enabled through the existing authority-only Enable path.

`RollbackReadinessEvidence` is freshly minted at rollback time and binds the immutable rollback pin, current target installation revision, current configuration revision/digest and verified rollback artifact/admission digests. Configuration drift after Apply therefore cannot silently reuse stale rollback evidence.

Admission evidence is not copied into aggregate current truth, but accepted commands persist the complete typed evidence or its canonical binding in their event payload. `evolve`/`replay` validate intrinsic identity, digest and carrier coherence without querying external catalog, policy, health or adapter state. This follows the existing installation/grant event pattern; “not aggregate current state” never means “absent from events or replay.”

### 5.6 Public declaration surface and pure decision context

The proposed `crate::market::update` public item set is exactly:

```text
UpdateConstructionError
UpdateDecisionError
UpdateReplayError
PackageUpdateId
UpdateCommandId
UpdateApprovalId
UpdateEvidenceId
UpdateEventSequence
UpdateRevision
UpdateState
UpdateChangeClass
PackageUpdatePlan
UpdateApprovalEvidence
UpdateReadinessEvidence
UpdateConfirmationEvidence
RollbackReadinessEvidence
UpdateDecisionContext
UpdateCommand
UpdateEventKind
UpdateEvent
PackageUpdateAggregate
PackageUpdateSnapshot                 // type alias to PackageUpdateAggregate
decide
evolve
replay
PackageUpdateRepository
UpdateCommandReceipt
UpdateCommandOutcome
UpdateRepositoryError
InMemoryPackageUpdateRepository
```

`UpdateDecisionContext` is a sealed, authority-redacted value. Its constructor is no wider than `pub(in crate::market)`. The semantic repository constructs it from one transaction-current installation snapshot, transaction-current catalog/revocation, registry and policy-admission carriers, the complete deterministic current-grant set and, for Apply/Rollback, subordinate installation/grant events already produced by the existing pure owners' `decide` functions. It verifies and exposes only read-only carrier/event references needed by update decision; callers cannot insert an allow bit, claimed grant completeness or caller-authored subordinate event. Apply rechecks the exact target and rollback catalog authorities bound by readiness; Rollback freshly rechecks the immutable rollback target. A post-approval catalog revision, revocation, registry or policy drift therefore rejects rather than consuming stale evidence.

The pure functions are:

```rust
pub fn decide(
    current: Option<&PackageUpdateAggregate>,
    context: &UpdateDecisionContext,
    command: &UpdateCommand,
) -> Result<UpdateEvent, UpdateDecisionError>;

pub fn evolve(
    current: Option<PackageUpdateAggregate>,
    event: &UpdateEvent,
) -> Result<PackageUpdateAggregate, UpdateReplayError>;

pub fn replay<'a>(
    events: impl IntoIterator<Item = &'a UpdateEvent>,
) -> Result<Option<PackageUpdateAggregate>, UpdateReplayError>;
```

Every persisted update event is reachable through `decide`. Every subordinate installation/grant event is independently reachable through its existing owner `decide`. Update replay validates the update stream, persisted admission evidence and recorded subordinate identities/digests; repository load additionally cross-checks those bindings against the actual installation/grant streams before returning a snapshot.

Checked public command constructors are exactly `stage`, `record_approval`, `apply`, `confirm_applied_update`, `rollback`, `cancel` and `cancel_after_terminal_installation`. Actions and payloads remain private. Command accessors are only `command_id` and `update_id`. Event accessors are `sequence`, `post_revision`, `command_id`, `update_id` and `kind`. Aggregate accessors are `update_id`, `installation_id`, `tenant_id`, `user_id`, `plan`, `state`, `revision`, `last_sequence`, optional approval/readiness evidence, applied installation revision and optional terminal evidence. Evidence values expose their identity/binding digests and complete typed authority bindings read-only; no mutable accessor exists.

`PackageUpdateRepository` exposes only `execute(UpdateCommand)`, `load_exact(PackageUpdateId)` and `event_history(PackageUpdateId)`, plus the implementation-private transaction-current catalog/installation/grant reads required to implement `execute`. `InMemoryPackageUpdateRepository` exposes empty `new`, checked replay-based `try_from_authority_histories` for exact typed current catalog/revocation, capability-registry and policy-admission carriers plus exact installation, grant and update event histories, and `fail_next_commit_for_test`. History seeding rebuilds and cross-checks every snapshot, command ledger, consumed-approval set, current-update slot and authority index through existing replay; it is not arbitrary snapshot insertion. The fake does not implement public `Default` or arbitrary insert/list/query methods. Authority-drift tests use a narrowly named test-only replacement of the complete typed catalog/registry/policy carrier, never an allow bit or partial field mutation.

The proposed `grant.rs` public addition is exactly one sealed read model plus one semantic trait method:

```rust
pub struct CurrentInstallationGrantSet { /* private fields */ }

fn load_current_for_installation(
    &self,
    tenant_id: &TenantId,
    user_id: &UserId,
    installation_id: &InstallationId,
    expected_installation_revision: &InstallationRevision,
) -> Result<CurrentInstallationGrantSet, GrantRepositoryError>;
```

`CurrentInstallationGrantSet` exposes only `tenant_id`, `user_id`, `installation_id`, `observed_installation_revision`, `grant_set_digest` and `grants() -> &[GrantSnapshot]`. Its constructor is repository-private and its manual `Debug` is authority-redacted. Here **current** means exactly the nonterminal `Active`, `Stale` or `Expired` snapshots represented by `GrantRepository`'s current-authority index. Terminal `Revoked` aggregates remain available only through exact history/audit reads and are excluded from this set and digest. The slice is sorted by the canonical authority tuple and then snapshot ID. The digest has a dedicated domain prefix and covers the observed installation revision plus every complete current nonterminal grant snapshot in that order.

## 6. Proposed commands and events

### 6.1 Commands

Every non-Stage command carries exact `expected_update_revision`. Apply/Rollback/ConfirmAppliedUpdate additionally carry exact `expected_installation_revision`.

Stage may bind an Enabled installation so target and rollback artifacts can be prepared without downtime. Apply still requires the transaction-current installation to be Disabled. A Disable after Stage legitimately changes the installation revision; Apply does not require equality with the staged revision. Instead it requires its own exact current expected revision and independently verifies that tenant/user, rollback package pin and configuration revision/digest still equal the immutable plan. Any other source-pin or configuration drift requires a new update plan and approval.

```text
Stage
  PackageUpdateId, UpdateCommandId, transaction-current InstallationSnapshot,
  actual rollback/target package and registry carriers

RecordApproval
  UpdateCommandId, PackageUpdateId, expected UpdateRevision,
  UpdateApprovalEvidence, UpdateReadinessEvidence

Apply
  UpdateCommandId, PackageUpdateId, expected UpdateRevision,
  expected InstallationRevision

ConfirmAppliedUpdate
  UpdateCommandId, PackageUpdateId, expected UpdateRevision,
  expected InstallationRevision, UpdateConfirmationEvidence

Rollback
  UpdateCommandId, PackageUpdateId, expected UpdateRevision,
  expected InstallationRevision, RollbackReadinessEvidence

Cancel
  UpdateCommandId, PackageUpdateId, expected UpdateRevision

CancelAfterTerminalInstallation
  UpdateCommandId, PackageUpdateId, expected UpdateRevision,
  expected terminal InstallationRevision
```

`Cancel` is legal only from `Staged` or `Ready`. If Revoke/Uninstall has made the installation terminal while the update is `AppliedPendingConfirmation`, `CancelAfterTerminalInstallation` is the sole reconciliation transition to `Cancelled`; it verifies the exact current terminal installation revision, records that terminal authority preempted rollback and releases the current-update slot. It never changes the terminal installation or any grant.

Stage constructors receive the real typed installation/catalog/package/registry carriers. They do not accept separately asserted IDs, digests, change class, publication status or permission booleans.

### 6.2 Update events

```rust
pub enum UpdateEventKind {
    Staged,
    ApprovalRecorded,
    Applied,
    Confirmed,
    RolledBack,
    Cancelled,
}
```

Every event carries `UpdateEventSequence`, post `UpdateRevision`, `UpdateCommandId`, `PackageUpdateId` and a private complete typed payload. `Staged` stores the immutable complete plan. `ApprovalRecorded` stores both evidence values. `Applied` stores exact pre/post installation revisions, target pin digest, one exact subordinate installation-event reference and the complete deterministic set of subordinate grant-stale event references. `Confirmed` stores confirmation evidence. `RolledBack` stores exact pre/post installation revisions, rollback pin digest, fresh rollback evidence, one exact subordinate installation-event reference and subordinate grant-stale event references. No event carries arbitrary prose, raw secret, rejected source payload, artifact bytes or adapter diagnostics.

An installation-event reference is the exact tuple `(InstallationId, InstallationEventSequence, post InstallationRevision, InstallationCommandId, expected PackageUpdated|PackageRolledBack kind, canonical event digest)`. Each grant-event reference analogously binds grant snapshot ID, event sequence, post version, command ID, expected `MarkedStale` kind and canonical event digest. Canonical event digests use separate domain prefixes and cover the complete private typed event payload; they are not hashes of redacted `Debug` output.

### 6.3 Installation events

Proposed sealed additions:

```rust
InstallationEventKind::PackageUpdated
InstallationEventKind::PackageRolledBack
```

Both events bind only the update plan digest, expected prior installation revision, exact prior and next package pins and leave `ManagedInstallationState` as `InstalledDisabled` or `Disabled`. Configuration and configuration revision do not change. Replay independently verifies same installation/package ID, disabled state, exact prior pin, exact next pin, sequence/revision and update plan digest. The event exposes a `pub(in crate::market)` canonical coupling digest over its complete typed fields so `update.rs` can bind it without learning or recreating private payload bytes. `installation.rs` does not import `PackageUpdateId` or another `update.rs` type; dependency remains `update -> installation`, not cyclic.

No public installation command may construct these actions. `Configure`, `Enable`, `Disable`, `Revoke` and `Uninstall` retain their existing public semantics. A pending update does not block emergency Revoke/Uninstall; a later update operation observes the terminal installation and fails closed. Rollback after terminal state is forbidden.

### 6.4 Grant events

Apply and Rollback call the exact `GrantRepository::load_current_for_installation` semantic query inside the same transaction. The query proves a bijection between every nonterminal `Active|Stale|Expired` aggregate for the exact tenant/user/installation and its current-authority index entry. A nonterminal aggregate missing its index entry, an index entry pointing to a missing/Revoked/wrong-key aggregate, a Revoked aggregate retained in the current index, or more than one nonterminal current aggregate for one authority tuple is `CorruptGrantSet`, never an omitted row. A historical Revoked aggregate with no current-index entry is valid and does not block a fresh Issue for that authority tuple. The query returns only Active, Stale and Expired current snapshots in the canonical order above; Revoked histories are neither returned nor digested. Every Active grant must bind the transaction-current installation revision and package/capability-manifest authority; mismatch is `ActiveGrantBindingMismatch`. Every Active grant receives a reachable existing `GrantEventKind::MarkedStale` event with `GrantInvalidationReason::InstallationChanged`; Stale/Expired grants remain unchanged. Deterministic subordinate `InstallationCommandId` and `GrantCommandId` values are derived under separate documented digest domains from the parent update command ID and exact installation/grant identities so replay and idempotent retry regenerate the same commands. A collision with an existing unequal subordinate command fails as `CommandConflict`; it never selects another ID.

The cross-stream transaction records the precondition `(observed installation revision, grant-set digest)` and recomputes both under the commit lock. Missing/extra/current-state/version/index drift is `GrantSetConflict` and commits nothing. It never lists grants through an arbitrary query API and never synthesizes a `Replaced`/`Issued` grant. Re-enablement requires fresh B4 grant evidence bound to the new current installation revision/package pin.

Post-update grant recovery remains a later explicit B4 flow and follows this exact matrix:

| target capability relation | old current snapshot after pin change | allowed recovery |
|---|---|---|
| unchanged identity and exact scope | `Stale(InstallationChanged)` | fresh approval/evidence + `Replace`; retain `GrantSnapshotId`, advance version |
| definition narrowed with the same exact scope | `Stale(InstallationChanged)` | fresh approval/evidence + `Replace`; no automatic path |
| scope changed | stale old snapshot | explicit `Revoke` old snapshot, then fresh approval/evidence + `Issue` with new `GrantSnapshotId` |
| capability added | no old snapshot | fresh approval/evidence + `Issue` |
| capability removed | stale old snapshot | explicit `Revoke`; no replacement while absent |

Update approval never satisfies grant approval. `Revoked` removes a snapshot from the current-authority index but does not erase its identity, events or evidence.

## 7. Decision and repository precedence

Every command first passes this repository-level prefix:

```text
update command-ledger exact replay / conflicting reuse
→ consumed UpdateApprovalId uniqueness for RecordApproval only
→ aggregate absence/presence, update identity and expected update revision as applicable
→ legal command-specific update state
→ current-update-slot coherence for every nonterminal aggregate
```

The current-update index is checked as a repository invariant after the exact aggregate is loaded. A missing or unequal slot for a nonterminal aggregate is `CorruptCurrentUpdateIndex`; a command against an ordinary reachable state does not reinterpret index corruption as a domain transition.

The remaining precedence is command-specific:

| command | ordered checks after the shared prefix |
|---|---|
| `Stage` | no current slot → update aggregate absent → installation exists, nonterminal and tenant/user coherent → exact rollback/target catalog, registry and policy carriers → immutable plan and computed change class → sequence overflow → atomic commit |
| `RecordApproval` | coherent approval and readiness evidence → transaction-current catalog/revocation/registry/policy bindings still match the immutable plan → sequence overflow → atomic commit |
| `Apply` | installation exists/nonterminal → tenant/user/revision/current rollback pin/configuration coherence → transaction-current target and rollback catalog/revocation/registry/policy coherence → immutable plan/change-class/approval/readiness coherence → `InstalledDisabled|Disabled` → complete current grant set and subordinate-command coherence → all sequence overflows → atomic cross-stream commit |
| `ConfirmAppliedUpdate` | installation exists/nonterminal → tenant/user/revision/current target pin coherence → exact `UpdateConfirmationEvidence` → update sequence overflow → atomic update-stream commit |
| `Rollback` | installation exists/nonterminal → tenant/user/revision/current target pin/configuration coherence → fresh rollback catalog/revocation/registry/policy coherence → immutable rollback target and `RollbackReadinessEvidence` → `InstalledDisabled|Disabled` → complete current grant set and subordinate-command coherence → all sequence overflows → atomic cross-stream commit |
| `Cancel` | `Staged|Ready` state already established by the shared prefix → update sequence overflow → atomic update-stream commit; no installation/catalog/grant/health read is required |
| `CancelAfterTerminalInstallation` | `AppliedPendingConfirmation` state already established → installation exists in exact terminal `Revoked|Uninstalled` state at the expected revision → update sequence overflow → atomic update-stream commit |

`ConfirmAppliedUpdate` deliberately does not recheck configuration, grants, catalog policy, runtime health or executor state: it closes the workflow only if the exact applied target pin remains current. `Cancel` likewise does not depend on mutable installation authority. Apply and Rollback are the only commands that stage subordinate installation/grant events.

A domain rejection persists the complete original command and exact typed rejected receipt. Identical retry returns it even after later state changes. A pre-commit persistence/transaction failure records no update, installation or grant event and consumes no command or approval identity.

Idempotency belongs exclusively to the repository command ledger: same command ID plus typed-equal complete command returns the stored receipt, while same command ID plus any unequal command is `CommandConflict`. This check precedes state-sensitive `decide`. Pure update/installation/grant `decide` functions do not reinterpret an independently resubmitted same-state transition as a no-op; without an exact ledger replay it is normally an illegal transition or revision/version mismatch.

`PackageUpdateRepository::execute` is one semantic transaction port, not composition over separately committed `InstallationRepository::execute` and `GrantRepository::execute` calls. Its adapter and in-memory fake stage the existing pure installation/grant decisions and commit all streams only after every decision and precondition succeeds. This is the only proposed write path for Apply/Rollback.

Apply commits exactly one update event, one installation package-pin event, zero or more complete current-grant stale events, all affected command ledgers/snapshots/indexes and the accepted receipt atomically. Rollback has the same atomicity. Any conflict leaves the prior installation pin and all grant states unchanged. Because Apply is disabled-only, failure never serves partially switched runnable authority.

## 8. Proposed errors and replay rejection

### 8.1 Construction/admission errors

```text
InvalidUpdateId
InvalidCommandId
InvalidApprovalId
InvalidEvidenceId
InvalidUpdateRevision
InvalidEventSequence
InstallationTerminal
PackageIdentityMismatch
TargetEqualsRollback
TargetUnpublishedOrRevoked
TargetCapabilityMissingOrInactive
ForbiddenAdministrativeCapability
DuplicateComponentOrCapability
AuthorityClassificationIncomplete
ApprovalEvidenceIncoherent
ReadinessEvidenceIncoherent
ConfirmationEvidenceIncoherent
RollbackEvidenceIncoherent
```

### 8.2 Decision errors

```text
AggregateMissing
AggregateAlreadyPresent
ActiveUpdateConflict
ApprovalAlreadyConsumed
InstallationMissing
InstallationTerminal
UpdateIdentityMismatch
UpdateRevisionMismatch
InstallationRevisionMismatch
InstallationPinMismatch
ConfigurationChanged
InstallationMustBeDisabled
PlanMismatch
AuthorityClassificationMismatch
CatalogAuthorityChanged
ApprovalMissingOrMismatch
ReadinessMissingOrMismatch
ActiveGrantBindingMismatch
ConfirmationEvidenceMismatch
RollbackUnavailable
RollbackEvidenceMismatch
CoupledInstallationEventMismatch
GrantSetConflict
IllegalTransition
SequenceOverflow
```

### 8.3 Repository errors

```text
CommandConflict
TransactionConflict
InjectedPersistenceFailure
CorruptUpdateHistory(UpdateReplayError)
CorruptInstallationHistory(InstallationReplayError)
CorruptGrantHistory(GrantReplayError)
CorruptCurrentUpdateIndex
CorruptGrantIndex
CorruptGrantSet
DecisionRejected(UpdateDecisionError)
```

Authority-bearing error `Debug`/`Display` exposes stable variants only. It does not print IDs, package/source/configuration values, approval/evidence values or adapter payloads.

Replay rejects at least: non-Staged initial event; sequence gap/duplicate/reorder/overflow; duplicate command or approval ID; post-terminal event; illegal transition; post-revision forgery; update/installation/tenant/user/package identity mismatch; plan/change-class/evidence digest forgery; Apply/Rollback without exactly one referenced installation event of the expected identity, kind and complete canonical digest; incomplete/extra/forged subordinate grant stale-event references; and a Confirmed/RolledBack state not reachable through `decide`. Repository load additionally requires every referenced installation/grant event to exist byte-semantically in the exact subordinate stream and rejects an unreferenced coupled transition, wrong stream position or digest mismatch.

## 9. Historical projection and current denial semantics

B6 never mutates an existing `ToolProjectionSnapshot`, `RunSpec`, toolset view, call envelope or historical receipt.

Conservative flow:

```text
frozen old projection exists
→ user explicitly Disables installation
→ current old calls deny
→ approved Apply atomically changes package pin and stales Active grants
→ frozen projection remains byte-equal but current calls still deny
→ fresh grants + existing Enable path
→ only a new projection may expose the target revision
```

Rollback requires current Disabled state, restores the exact prior pin, stales target grants and remains Disabled. Old frozen projections remain immutable; they do not become current again. A later projection may expose the rollback revision only after fresh grants and Enable evidence. No error falls back to the old package, same-name tool, stale grant or previous successful projection.

`ConfirmAppliedUpdate` is legal only from `AppliedPendingConfirmation`. It verifies that the applied target pin is still current, records terminal `Confirmed` evidence and releases the slot; it neither reads nor certifies runtime health. After confirmation there is no rollback command for that update. A later revert stages a fresh reverse update and passes the same exact-plan approval, Disabled-only Apply and grant-staleness rules as every other update.

## 10. Future implementation surface and non-goals

A later implementation grant should name, at minimum:

```text
crates/platform-core/src/market.rs
crates/platform-core/src/market/update.rs
crates/platform-core/src/market/installation.rs
crates/platform-core/src/market/grant.rs
crates/platform-core/tests/market_package_update.rs
scripts/check_repo_contracts.py
scripts/tests/test_check_repo_contracts.py
docs/plan/modules/30-market-package-lifecycle.md
docs/contracts/market-lifecycle.md
docs/plan/04-market-and-plugin-lifecycle.md
docs/features/00-market-browse-install.md
docs/acceptance/matrix.tsv
docs/tasks/01-execution-roadmap.md
docs/tasks/campaign-w1-m20-b6.md
```

The exact implementation taskbook must freeze the public item inventory, test-function inventory, mutation tests, admissible import/dependency set and authority-bearing manual-`Debug` inventory before Rust edits.

B6 explicitly does not implement:

- automatic patch/canary rollout or cohort policy;
- enabled-to-enabled package switching;
- automatic restoration of Enabled state after rollback;
- post-confirm rollback anchors, a rollback stack or undo across a released current-update slot;
- a latest-version resolver or SemVer update policy;
- artifact download, package manager, filesystem switch, health-check implementation, health-gated Enable/confirmation or cleanup scheduler;
- M90 production persistence/transaction code;
- grant approval UI, auto-grant, production enable-evidence issuer;
- M10/M80 browse/update API or UI;
- M30 effect intent, M40/M51 execution or B7 composition;
- acceptance-row promotion.

## 11. Proposed future evidence bindings

The rows stay `planned`. B6 implementation would add supporting evidence, not claim final product acceptance.

### `MARKET-004`

Proposed focused binding:

```text
cargo test --locked -p ustc-campus-agent-core --test market_package_update
  permission_expansion_and_every_conservative_update_require_exact_approval
```

The test must prove:

- capability addition, B2 expansion, source-policy change and execution-identity change classify `ReapprovalRequired`;
- narrowed/unchanged updates also cannot reach Apply without their own exact approval;
- stale, reused, mismatched or omitted approval/readiness evidence emits no event and mutates no carrier;
- post-approval catalog revocation/revision, registry or policy-authority drift returns a persisted typed rejection receipt, emits no event, mutates no carrier and never releases/reinterprets the approval already consumed by `ApprovalRecorded`;
- Apply requires Disabled, exact installation/update revisions and exact complete target/rollback pins;
- accepted Apply atomically changes the pin and stales every Active current grant;
- unchanged or narrowed capabilities still require fresh grant approval/evidence before `Replace`; scope changes revoke the old snapshot and require a new `Issue`; additions require `Issue`; removals create no replacement;
- missing/extra/duplicate/corrupt current-grant index entries, Active grant binding mismatch and commit-time grant-set digest drift reject without partial mutation;
- replay/load rejects a missing, extra, wrong-kind or forged-digest coupled installation/grant event;
- command/approval conflicts and injected persistence failure are closed and idempotent.

`MARKET-004` remains planned after bounded B6 domain evidence until the admitted application/update path proves user-visible exact diff/reapproval and no automatic permission expansion.

### `PKG-020`

Proposed focused binding:

```text
cargo test --locked -p ustc-campus-agent-core --test market_package_update
  apply_and_rollback_preserve_frozen_toolsets_and_change_only_current_future_authority
```

The test must prove:

- an old projection and Agent toolset remain byte/typed-equal through Apply and Rollback;
- current recheck denies after Disable/Apply/stale grants;
- no new target projection exists until fresh grant/Enable authority;
- rollback requires Disabled plus fresh rollback readiness and remains Disabled;
- rollback is admitted only from `AppliedPendingConfirmation`; `ConfirmAppliedUpdate` is terminal and any later revert requires a fresh reverse-update plan and approval;
- confirmation and Enable do not bind runtime health, and runtime availability never changes the five-state installation lifecycle;
- after fresh authority, a new projection binds only the exact current target or rollback pin;
- terminal Revoke leaves every frozen in-flight projection byte/typed-equal, makes current call-time authority deny, creates no new projection and selects no old/new fallback; `CancelAfterTerminalInstallation` may reconcile only update audit state;
- no failed operation mutates a frozen projection or selects a fallback.

`PKG-020` remains planned until B7 application composition and its integration gate prove the same ordering across real public M20/M30/M40 boundaries.

## 12. Proposed gates and acceptance condition

A future retained implementation is not merge-ready until all of:

```text
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-features -- -D warnings
cargo test --locked -p ustc-campus-agent-core --lib market::update::tests
cargo test --locked -p ustc-campus-agent-core --test market_package_update
cargo test --locked -p ustc-campus-agent-core --test market_installation_lifecycle
cargo test --locked -p ustc-campus-agent-core --test market_grant_lifecycle
cargo test --locked -p ustc-campus-agent-core --test market_authority_assembly
cargo test --locked -p ustc-campus-agent-core --test invocation_resolution
cargo test --locked --workspace --all-features
cargo test --locked --doc --workspace
python3 scripts/check_repo_contracts.py --ci
python3 -m unittest scripts.tests.test_check_repo_contracts
```

plus exact-source independent reviews for lifecycle/authority ownership, atomicity/replay/failure precedence, and checker/acceptance honesty.

## 13. Decision required before authority or code mutation

Before any proposal merge or Rust implementation, Develata must explicitly accept, reject or amend this entire digest-bound packet and activate a finite implementation grant. The policy selections recorded in §1 constrain this repaired proposal but are not, by themselves, acceptance of every Rust surface below. Final packet acceptance must confirm at least:

- Conservative MVP remains the intended first version;
- update/installation/grant ownership and atomic cross-stream commit are acceptable;
- rollback is limited to `AppliedPendingConfirmation`, terminal `ConfirmAppliedUpdate` closes it, and every later revert is a fresh approved reverse update;
- every Active grant becomes Stale on a pin change and every reactivation, including unchanged-capability `Replace`, consumes fresh grant approval/evidence;
- confirmation and Enable remain independent of runtime health, with authority and runtime availability projected separately;
- the exact command/event/error/public-item/evidence/replay surfaces are acceptable;
- `MARKET-004` and `PKG-020` remain planned with the future bindings above;
- the future implementation path set and non-goals are acceptable.

Until then, current accepted `market-lifecycle/v0` and existing Rust behavior remain unchanged.
<!-- M20_B6_READINESS_PACKET:END -->

<!-- M20_B6_IMPLEMENTATION_GRANT:BEGIN -->
## 14. Operation-specific finite implementation grant

### 14.1 Authority and source binding

Develata activated this finite retained-implementation grant on `2026-08-01`. It supersedes the proposal-only stop for this lane but does not mutate the active campaign-grant block, authorize another batch or widen the product contract.

| field | exact receipt |
|---|---|
| accepted semantic packet | delimited SHA-256 `3bfde89d1915d525ded7155c8c324b99c39146c69c636165368f2ba3e5ef1d10`; bytes remain immutable |
| accepted proposal PR | `#32`, squash-merged as `5e9e5b934cd7e6ea50db1787ebd35949394f1052` |
| exact base tree | `92b2347ac678bebf4c55e6423965219f8e53765a` |
| post-main CI | GitHub Actions `30700279228`; `rust` and `docs-and-contracts` passed |
| implementation branch | `feat/m20-b6-package-update-domain` in a dedicated clean worktree |
| shipping authority | local implementation/review only; commit, push, PR and implementation merge require a separate source-control decision |

This is a Path-B-sized bounded domain slice only if all Path B admission criteria pass on the final candidate. Otherwise it remains local and falls back to the M20 Path A branch; the taskbook does not self-authorize a remote operation.

### 14.2 Exact writable paths

No path outside this list may change without recording a scope blocker and obtaining a fresh operation-specific decision:

```text
crates/platform-core/src/market.rs
crates/platform-core/src/market/update.rs
crates/platform-core/src/market/installation.rs
crates/platform-core/src/market/grant.rs
crates/platform-core/tests/market_package_update.rs
crates/platform-core/tests/platform_identity.rs
scripts/check_repo_contracts.py
scripts/tests/test_check_repo_contracts.py
docs/plan/modules/00-module-map.md
docs/plan/modules/30-market-package-lifecycle.md
docs/plan/04-market-and-plugin-lifecycle.md
docs/contracts/market-lifecycle.md
docs/features/00-market-browse-install.md
docs/acceptance/matrix.tsv
docs/tasks/01-execution-roadmap.md
docs/tasks/campaign-w1-m20-b6.md
docs/overview/architecture.md
```

`00-module-map.md` and `overview/architecture.md` are admitted only to replace stale “no bounded update evidence” prose with the exact bounded/non-production truth. `Cargo.toml`, `Cargo.lock`, active campaign-grant/policy delimiters, workflows, CODEOWNERS, fixtures, adapters and application/UI paths are forbidden. No new dependency, feature, build target or generated source is admitted.

### 14.3 Public declaration inventory

`market.rs` adds only `pub mod update;`. The `crate::market::update` public item set is exactly:

```text
UpdateConstructionError
UpdateDecisionError
UpdateReplayError
PackageUpdateId
UpdateCommandId
UpdateApprovalId
UpdateEvidenceId
UpdateEventSequence
UpdateRevision
UpdateState
UpdateChangeClass
PackageUpdatePlan
UpdateApprovalEvidence
UpdateReadinessEvidence
UpdateConfirmationEvidence
RollbackReadinessEvidence
UpdateDecisionContext
UpdateCommand
UpdateEventKind
UpdateEvent
PackageUpdateAggregate
PackageUpdateSnapshot
PackageUpdateRepository
decide
evolve
replay
UpdateCommandReceipt
UpdateCommandOutcome
UpdateRepositoryError
InMemoryPackageUpdateRepository
```

Private fields, checked constructors and immutable accessors are mandatory. `PackageUpdateSnapshot` is only a type alias to `PackageUpdateAggregate`. No public `Default`, Serde construction, mutable accessor, arbitrary insert/list/query, authority boolean, caller-supplied change class or adapter payload is admitted.

The exact public command constructors remain:

```rust
impl UpdateCommand {
    #[allow(clippy::too_many_arguments)]
    pub fn stage(
        command_id: UpdateCommandId,
        update_id: PackageUpdateId,
        installation: &InstallationSnapshot,
        target_pin: InstallationPackagePin,
        rollback_catalog: &CatalogReadModel,
        rollback_publications: &[CatalogPackageRevision],
        target_catalog: &CatalogReadModel,
        target_publications: &[CatalogPackageRevision],
        rollback_registry: &CapabilityRegistry,
        target_registry: &CapabilityRegistry,
    ) -> Result<Self, UpdateConstructionError>;

    pub fn record_approval(
        command_id: UpdateCommandId,
        update_id: PackageUpdateId,
        expected_update_revision: UpdateRevision,
        approval: UpdateApprovalEvidence,
        readiness: UpdateReadinessEvidence,
    ) -> Result<Self, UpdateConstructionError>;

    pub fn apply(
        command_id: UpdateCommandId,
        update_id: PackageUpdateId,
        expected_update_revision: UpdateRevision,
        expected_installation_revision: InstallationRevision,
    ) -> Result<Self, UpdateConstructionError>;

    pub fn confirm_applied_update(
        command_id: UpdateCommandId,
        update_id: PackageUpdateId,
        expected_update_revision: UpdateRevision,
        expected_installation_revision: InstallationRevision,
        evidence: UpdateConfirmationEvidence,
    ) -> Result<Self, UpdateConstructionError>;

    pub fn rollback(
        command_id: UpdateCommandId,
        update_id: PackageUpdateId,
        expected_update_revision: UpdateRevision,
        expected_installation_revision: InstallationRevision,
        evidence: RollbackReadinessEvidence,
    ) -> Result<Self, UpdateConstructionError>;

    pub fn cancel(
        command_id: UpdateCommandId,
        update_id: PackageUpdateId,
        expected_update_revision: UpdateRevision,
    ) -> Result<Self, UpdateConstructionError>;

    pub fn cancel_after_terminal_installation(
        command_id: UpdateCommandId,
        update_id: PackageUpdateId,
        expected_update_revision: UpdateRevision,
        expected_terminal_installation_revision: InstallationRevision,
    ) -> Result<Self, UpdateConstructionError>;
}
```

Command accessors are only `command_id` and `update_id`; event accessors are only the accepted packet set. `UpdateCommandReceipt` stores the complete private `UpdateCommand` plus `UpdateCommandOutcome` and exposes immutable `command()`/`outcome()` accessors so an unforgeable typed receipt can reseed its exact ledger entry. Evidence constructors remain no wider than `pub(in crate::market)`.

`UpdateDecisionContext` has no public constructor and no mutable or allow-like accessor. Owner-only constructors are exactly the command-specific families `for_stage`, `for_record_approval`, `for_apply`, `for_confirm_applied`, `for_rollback`, `for_cancel` and `for_cancel_after_terminal_installation`; they accept complete typed carriers/events, not claimed booleans or partial indexes.

The public repository port remains exactly `execute`, `load_exact` and `event_history`.

### 14.4 Semantic-fake seed and ledger-completeness repair

The accepted packet requires replay-based history seeding and complete command-ledger reconstruction. Events alone cannot reconstruct a persisted rejected command: rejected receipts append no event, and the existing grant receipt did not retain its complete command. Therefore the implementation freezes this evidence-carrier repair:

1. `GrantCommandReceipt` privately stores the complete `GrantCommand` instead of only duplicated command/snapshot IDs; its existing `command_id()` and `snapshot_id()` behavior is preserved and no new public grant item/accessor is added.
2. Installation, grant and update owner repositories accept unforgeable typed receipts plus the exact observed pre-decision snapshot when rebuilding a ledger. Receipt fields remain private; callers cannot author a receipt.
3. A receipt seed is admitted only when its command identity, observed snapshot, accepted event/snapshot or rejected outcome, and the corresponding replayed stream prefix are coherent. Accepted events have a one-to-one exact position; rejected receipts have no event. Missing/extra/duplicate/conflicting ledger entries fail closed.

The public fake signature is frozen as:

```rust
impl InMemoryPackageUpdateRepository {
    #[must_use]
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self;

    #[allow(clippy::too_many_arguments)]
    pub fn try_from_authority_histories(
        catalog_read_models: Vec<CatalogReadModel>,
        catalog_publications: Vec<CatalogPackageRevision>,
        capability_registries: Vec<CapabilityRegistry>,
        policy_snapshots: Vec<InvocationPolicySnapshot>,
        installation_histories: Vec<(InstallationId, Vec<InstallationEvent>)>,
        installation_ledger_receipts: Vec<(
            InstallationCommandReceipt,
            Option<InstallationSnapshot>,
        )>,
        grant_histories: Vec<(GrantSnapshotId, Vec<GrantEvent>)>,
        grant_ledger_receipts: Vec<(GrantCommandReceipt, Option<GrantSnapshot>)>,
        update_histories: Vec<(PackageUpdateId, Vec<UpdateEvent>)>,
        update_ledger_receipts: Vec<(
            UpdateCommandReceipt,
            Option<PackageUpdateSnapshot>,
        )>,
    ) -> Result<Self, UpdateRepositoryError>;

    pub fn fail_next_commit_for_test(&mut self);
}
```

`installation.rs` and `grant.rs` may add only repository-private checked `try_from_histories_and_receipts`/staged-clone helpers needed to implement this signature. They replay through their owning `replay`/`decide`/`evolve`, rebuild indexes and ledgers, and expose no arbitrary snapshot/event insertion.

### 14.5 Owner-only installation and grant additions

`installation.rs` adds only public event kinds `PackageUpdated` and `PackageRolledBack`. `InstallationCommand` gains restricted `package_updated` and `package_rolled_back` constructors taking the exact expected revision, plan digest and next pin. They store exact prior/next pins in private event payloads, preserve configuration and disabled state, and expose only a `pub(in crate::market)` canonical coupling digest on `InstallationEvent`. It imports no `update` type.

`grant.rs` adds exactly `CurrentInstallationGrantSet` and `GrantRepository::load_current_for_installation` as accepted. The read model has private fields, canonical sorted `grants()`, exact observed installation revision and a domain-separated complete-set digest. Its constructor and per-event canonical coupling digest remain `pub(in crate::market)`. Revoked history is excluded; Revoked-in-current-index and every missing/extra/wrong-key/duplicate nonterminal current row are corruption.

For atomic Apply/Rollback, `InMemoryPackageUpdateRepository` may stage checked clones of the existing owner repositories, invoke owner `decide`/`evolve` or equivalent owner-private staged helpers, create the update event from those exact subordinate events, and swap all staged maps only after all decisions, sequence checks and preconditions pass. It must not perform externally visible sequential `execute` commits or reimplement installation/grant event construction.

### 14.6 Admissible dependency and import set

`update.rs` may import only:

```text
std::{collections::{BTreeMap, BTreeSet}, error::Error, fmt}
crate::identity::{TenantId, UserId}
crate::invocation::{CapabilityId, CatalogComponentRevision, CatalogPackageRevision,
  CatalogRevision, ComponentId, ComponentKind, ComponentVersion, ExecutionIdentity,
  GrantSnapshotId, GrantState, GrantVersion, InstallationId, InstallationRevision,
  InvocationPolicySnapshot, PackageId, PackageVersion, PolicyRevision,
  PolicySnapshotId, Sha256Digest}
crate::market::{CatalogReadModel, ComponentDeclaration, PackageTier,
  ValidatedPackageManifest}
crate::market::capability::{compare_capability_definitions, CapabilityDefinition,
  CapabilityPolicyChange, CapabilityRegistry, CapabilityRegistryRevision,
  CapabilityStatus, ScopeKind}
crate::market::installation::{ConfigurationRevision, InstallationCommand,
  InstallationCommandId, InstallationCommandOutcome, InstallationCommandReceipt,
  InstallationEvent, InstallationEventKind, InstallationEventSequence,
  InstallationPackagePin, InstallationReplayError, InstallationRepository,
  InstallationRepositoryError, InstallationSnapshot, InMemoryInstallationRepository,
  ManagedInstallationState}
crate::market::grant::{CurrentInstallationGrantSet, GrantCommand, GrantCommandId,
  GrantCommandOutcome, GrantCommandReceipt, GrantEvent, GrantEventKind,
  GrantEventSequence, GrantInvalidationReason, GrantReplayError, GrantRepository,
  GrantRepositoryError, GrantSnapshot, InMemoryGrantRepository}
```

Unused imports are not admitted. The checker freezes the final exact normalized import declarations. Serde, direct `sha2`, time/clock, random/UUID, filesystem, network, process, database, framework, executor, model/provider, MCP and M30/M40/M90 types are forbidden.

Dependency direction remains `update -> installation/grant/capability/market/invocation/identity`; installation and grant do not import update.

### 14.7 Manual `Debug` inventory

Manual authority-redacted `fmt::Debug` is mandatory for:

```text
PackageUpdateId
UpdateCommandId
UpdateApprovalId
UpdateEvidenceId
PackageUpdatePlan
UpdateApprovalEvidence
UpdateReadinessEvidence
UpdateConfirmationEvidence
RollbackReadinessEvidence
UpdateDecisionContext
UpdateCommandAction
UpdateCommand
InstallationEventReference
GrantEventReference
UpdateEventPayload
UpdateEvent
PackageUpdateAggregate
UpdateCommandOutcome
UpdateCommandReceipt
InMemoryPackageUpdateRepository
CurrentInstallationGrantSet
```

Every private authority index/context/ledger type either has no `Debug` or a manual redacted implementation. New installation package-pin payloads remain under the existing manually redacted action/event-payload wrappers. `UpdateConstructionError`, `UpdateDecisionError`, `UpdateReplayError`, `UpdateRepositoryError`, `UpdateEventSequence`, `UpdateRevision`, `UpdateState`, `UpdateChangeClass` and `UpdateEventKind` may derive `Debug` only while they contain category-only variants or numeric/revision values; a future payload-bearing variant must switch to manual redaction.

### 14.8 Test-function inventory and proof-carrier repair

The complete `market::update::tests` unit-test inventory is:

```text
checked_update_ids_revisions_and_sequences_are_canonical
update_plan_binds_exact_source_target_catalog_registry_policy_and_digests
change_classifier_is_computed_from_complete_typed_authority_not_caller_hint
change_classifier_precedence_covers_added_expanded_source_policy_tier_component_scope_and_unknown
evidence_values_bind_plan_digest_revision_configuration_and_authority_without_leaking_payloads
state_machine_allows_only_stage_approval_apply_confirm_rollback_cancel_paths
stage_requires_absence_one_slot_nonterminal_installation_and_distinct_reviewed_target
record_approval_requires_coherent_fresh_approval_and_readiness_evidence
apply_requires_ready_disabled_exact_revisions_pins_configuration_and_authority_recheck
rollback_requires_applied_pending_disabled_fresh_readiness_and_current_target_pin
confirm_only_closes_applied_pending_without_runtime_health_or_grant_authority
cancel_and_terminal_installation_reconciliation_have_exact_precedence
event_sequences_revisions_command_ids_and_approval_consumption_are_exact
replay_accepts_reachable_histories_and_rejects_gap_duplicate_reorder_overflow_and_post_terminal
replay_rejects_forged_plan_change_class_evidence_revision_and_identity_bindings
subordinate_installation_and_grant_event_references_are_complete_kind_checked_and_digest_bound
repository_idempotency_command_conflict_approval_conflict_and_failure_injection_are_atomic
repository_rebuilds_current_slot_all_ledgers_consumed_approvals_and_authority_indexes
permission_expansion_and_every_conservative_update_require_exact_approval
apply_and_rollback_preserve_frozen_toolsets_and_change_only_current_future_authority
public_errors_debug_display_and_authority_debug_are_category_only_and_secret_safe
```

The accepted semantics require the last two named acceptance proofs, but an external integration-test crate cannot invoke `pub(in crate::market)` evidence minting constructors without widening authority. They are therefore rebound, with exact names and assertion scope unchanged, to:

```text
cargo test --locked -p ustc-campus-agent-core --lib \
  market::update::tests::permission_expansion_and_every_conservative_update_require_exact_approval -- --exact
cargo test --locked -p ustc-campus-agent-core --lib \
  market::update::tests::apply_and_rollback_preserve_frozen_toolsets_and_change_only_current_future_authority -- --exact
```

This is a proof-carrier placement repair, not a lifecycle/public-API change. The external `market_package_update.rs` remains mandatory and contains exactly:

```text
checked_public_update_values_and_stage_surface_are_deterministic
empty_public_repository_and_replay_are_non_authoritative
public_errors_and_debug_are_category_only_and_redacted
```

It proves the downstream public boundary without a test-only authority mint. Compile-fail doctests or an equivalent compiler probe must establish that external callers cannot construct evidence/context/events or mutate snapshots. No test feature, `cfg(debug_assertions)` public backdoor or test-only public harness is admitted.

### 14.9 Checker and mutation inventory

`scripts/check_repo_contracts.py` remains the sole fail-closed checker owner and must add/update:

```text
PLATFORM_UPDATE_SOURCE
PLATFORM_UPDATE_TEST
PLATFORM_CORE_SOURCE_FILES
PLATFORM_CORE_ADMITTED_MODULE_DECLARATIONS
PLATFORM_CORE_ADMITTED_ITEM_DECLARATIONS
PLATFORM_CORE_ADMITTED_SIBLING_MACROS
PLATFORM_CORE_ADMITTED_MACRO_INVOCATIONS
PLATFORM_CORE_ADMITTED_SIBLING_IMPLS
PLATFORM_CORE_ADMITTED_ATTRIBUTE_NAMES / exact attribute-count inventories
PLATFORM_UPDATE_ADMITTED_PUBLIC_DECLARATIONS
PLATFORM_UPDATE_ADMITTED_UNCLASSIFIED_PUBLIC
PLATFORM_UPDATE_ADMITTED_IMPL_DECLARATIONS
PLATFORM_UPDATE_ADMITTED_DERIVES
PLATFORM_UPDATE_UNIT_TEST_FUNCTIONS
PLATFORM_UPDATE_INTEGRATION_TEST_FUNCTIONS
PLATFORM_INSTALLATION_ADMITTED_PUBLIC_DECLARATIONS
PLATFORM_INSTALLATION_ADMITTED_UNCLASSIFIED_PUBLIC
PLATFORM_INSTALLATION_ADMITTED_DERIVES
PLATFORM_GRANT_ADMITTED_PUBLIC_DECLARATIONS
PLATFORM_GRANT_ADMITTED_UNCLASSIFIED_PUBLIC
PLATFORM_GRANT_ADMITTED_DERIVES
```

The checker must reject missing/unregistered update source or test, module-declaration drift, import/item/impl/derive/attribute/macro/public/restricted-public drift, omitted/ignored/cfg-disabled unit or integration tests, authority-bearing `Debug` derive, visibility widening, arbitrary insert/list/query APIs and missing installation/grant coupling surfaces.

`scripts/tests/test_check_repo_contracts.py` must retain mutation-sensitive cases for each category above, including at least:

```text
test_missing_market_update_nested_file_fails_closed
test_missing_market_update_module_declaration_fails_closed
test_unregistered_market_update_source_or_test_fails_closed
test_market_update_public_rename_extra_item_and_visibility_widening_fail_closed
test_market_update_dependency_import_impl_derive_attribute_and_macro_drift_fail_closed
test_market_update_authority_debug_derive_fails_closed
test_market_update_missing_or_ignored_unit_test_fails_closed
test_market_update_missing_or_ignored_integration_test_fails_closed
test_market_installation_update_event_or_restricted_constructor_drift_fails_closed
test_market_grant_current_set_or_arbitrary_query_drift_fails_closed
test_market_004_status_promotion_after_b6_fails_closed
test_pkg_020_status_promotion_after_b6_fails_closed
```

The mutation tests may update intentional checked-in allowlists but must not weaken campaign-authorization policy/digest/mutations, identity/authority checkers or CI wiring.

### 14.10 Documentation and status projections

The bounded slice updates accepted plan/contract/feature/roadmap/architecture prose only to state that a pure update aggregate and deterministic semantic in-memory transaction fake are retained supporting evidence. It must continue to state:

- no durable installation/grant/update repository, production transaction/TOCTOU adapter or crash recovery;
- no production approval/readiness/confirmation/grant/enable issuer;
- no artifact switch, executor, M10/M80 API/UI, B7 composition or M90 persistence;
- `MARKET-004` and `PKG-020` remain `planned`;
- M20 remains `partial-evidence`, never StandaloneReady/IntegrationReady/Accepted;
- historical projections remain immutable and runtime health remains separate from authority.

No active campaign-grant/policy marker bytes may change. The roadmap may update only ordinary B6/current-evidence prose outside those markers.

### 14.11 Required implementation and closeout gates

PRE_EDIT closure requires an independent review bound to the exact base, accepted packet digest and this implementation-grant digest. Only a written `GO` allows `Status: active` and Rust edits.

The final candidate must pass:

```text
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-features -- -D warnings
cargo test --locked -p ustc-campus-agent-core --lib market::update::tests
cargo test --locked -p ustc-campus-agent-core --test market_package_update
cargo test --locked -p ustc-campus-agent-core --test market_installation_lifecycle
cargo test --locked -p ustc-campus-agent-core --test market_grant_lifecycle
cargo test --locked -p ustc-campus-agent-core --test market_authority_assembly
cargo test --locked -p ustc-campus-agent-core --test invocation_resolution
cargo test --locked --workspace --all-features
cargo test --locked --doc --workspace
python3 scripts/check_repo_contracts.py --ci
python3 -m unittest scripts.tests.test_check_repo_contracts
python3 -m unittest discover -s scripts/tests -p 'test_*.py'
git diff --check
```

Final review fan-in is exactly three blocker-only lanes: lifecycle/state/idempotency; cross-stream atomicity/replay/complete-current authority; checker/public-surface/status honesty. Parent verification must replay every accepted blocker. After convergence, stop before source-control mutation and present exact paths, diff, gates, review receipts and candidate tree to Develata.

### 14.12 Stop conditions

Pause before further mutation if implementation needs a path/dependency/public type outside this grant, a lifecycle/authority behavior not settled by the accepted packet, test-only public authority, M90/durable adapter, active grant/policy edit, acceptance promotion, live user data/credentials, unplanned instrumentation, or the same semantic blocker/gate failure after repair round `2`.
<!-- M20_B6_IMPLEMENTATION_GRANT:END -->
