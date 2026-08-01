# W1 M20-B6 update/rollback exact-contract readiness

## Authority

- `Campaign ID`: `USTC-MODULES-2026-07-W1`
- `Lane`: `M20-B6`
- `Grant carrier`: [`01-execution-roadmap.md`](01-execution-roadmap.md#active-autonomous-module-campaign-authorization)
- `Mode`: proposal-only; no contract acceptance or retained implementation

## Mutable campaign state

- `Status`: `paused`
- `Bound source commit`: `993b44229d1ff76a124d5194a00c0b29b7991f7d`
- `Repair round`: `1`
- `Current blocker identity`: `none`
- `Stop reason`: `contract-acceptance-required-before-any-M20-B6-implementation-or-status-promotion`
- `Last transition evidence`: `packet-sha256:5b340f1934c74c1fccd5de54e63a330b9fa485025b6faf383a158bb9da148e11; architecture-authority-round1b=GO; state-machine-atomicity-round1b=GO; acceptance-checker-scope-round1b=GO; local-contract-unit=459/459-PASS`
- `Next allowed mutation`: `run final exact-source local and Dongfeng gates; on PASS stage exact reviewed files, commit, push the granted branch and create a Draft PR; then no further mutation until contract acceptance`

## Output contract

Produce one source-bound exact-contract readiness packet for planned `MARKET-004` and `PKG-020`: update staging, permission expansion/reapproval, atomic package-pin activation, rollback target and event/audit semantics. This packet MAY be pushed as a Draft PR. It MUST NOT be merged, presented as an accepted `market-lifecycle/v0` amendment or used to retain implementation until Develata accepts the exact proposal and activates an implementation grant.

## Required evidence

- exact source commit/tree and clean-checkout receipt;
- high-level plan-to-exact-contract gap table;
- proposed command/state/error/event ordering and acceptance future bindings;
- independent blocker review bound to the delimited packet digest;
- every repair round and blocker identity recorded above before another mutation.

<!-- M20_B6_READINESS_PACKET:BEGIN -->
## 1. Proposal status and selected posture

Everything between the packet markers is **proposed, not accepted authority**. Existing accepted plans, contracts, Rust types and acceptance statuses remain authoritative until a separately authorized contract amendment is merged.

Develata selected the Conservative MVP posture on `2026-08-01`:

1. every update requires one explicit exact-plan approval;
2. Apply is legal only while the installation is `InstalledDisabled` or `Disabled`;
3. Apply changes the exact installed package pin atomically but does not Enable the installation;
4. rollback restores the exact prior package pin and remains Disabled;
5. re-enablement uses the existing fresh grant and `EnablePreconditionEvidence` path;
6. no enabled-to-enabled switch, automatic patch rollout, canary promotion or automatic restoration of Enabled state enters B6.

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
| `market-lifecycle.md` M20-LC-009 | stage/apply/rollback use exact reviewed targets and preserve rollback | no command/state/event/revision algebra or tested rollback evidence | one event-sourced update aggregate with exact source/target pins, readiness evidence, expected revisions and retained rollback target |
| `installation.rs` | one exact package pin, configuration and managed lifecycle | no owned package-pin replacement transition | installation-owned sealed Apply/Rollback package-pin events invoked only by the update transaction; managed state remains Disabled |
| `grant.rs` | grants bind installation revision/package/capability manifest and replacement always consumes fresh approval | package-pin mutation could otherwise leave an Active old grant in the current index | accepted Apply/Rollback atomically marks every Active current grant for the installation Stale with `InstallationChanged`; later grant replacement/issue requires fresh B4 evidence |
| `authority.rs` / P0a | frozen projections plus current deny-side recheck | no package update source of current carrier change | B6 mutates only future installation/grant carriers; frozen projections remain byte-equal while current calls fail closed until fresh enable/grants |
| `MARKET-004` | planned; B2/B4 supporting evidence | no update integration proof | proposed focused package-update classifier/approval/atomicity tests; row remains planned until admitted implementation and required later composition evidence |
| `PKG-020` | planned | no update/rollback versus frozen projection proof | proposed frozen-old/new-projection integration fixture; row remains planned pending B7 composition/integration gate |

## 4. Ownership and dependency direction

### 4.1 Proposed owner

A future `crate::market::update` module owns:

- package-update identity, aggregate state and revision;
- source-to-target plan and rollback-target binding;
- complete change classification;
- trusted approval/readiness/completion evidence values;
- update commands, events, replay and semantic repository port;
- cross-stream transaction ordering for one update aggregate, its installation and current grants.

`crate::market::installation` remains the sole owner of `InstallationPackagePin`, installation revision and legal installation events. B6 may add sealed package-pin Apply/Rollback actions whose constructors are no wider than `pub(in crate::market)`; external callers cannot replace a pin directly.

`crate::market::grant` remains the sole owner of grant transitions. B6 invokes existing `MarkStale(InstallationChanged)` decisions for the transaction-current complete set of Active grants; it does not forge grant events, reimplement grant replay or reactivate an old grant.

A future M90 adapter implements the semantic transaction. M20 does not import a database, filesystem, package manager, clock, network client, executor or framework type. B7 remains the application/API/composition owner.

### 4.2 No second authority

The update aggregate records one proposed/accepted transition plan and its audit history. It is not a peer installation authority. Current runnable truth remains:

```text
reviewed catalog carrier
+ current InstallationAggregate package pin/state/revision
+ current GrantAggregate states/versions
+ current policy/source/execution admission
```

`UpdateState::AppliedPendingConfirmation` records workflow posture only. It cannot make the target Enabled, runnable or granted.

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
  ├─ Complete ────────────► Completed             terminal
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
    Completed,
    RolledBack,
    Cancelled,
}
```

Only one update in `Staged`, `Ready` or `AppliedPendingConfirmation` may own the current update slot for an installation. Terminal updates retain history and release that slot. Revoke/Uninstall of the installation always dominates update workflow authority; no update command can revive a terminal installation.

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

`UpdateCompletionEvidence` binds the applied update revision, transaction-current installation revision/pin/state, current grant-set snapshot digest, current policy-admission digest and post-apply health/admission evidence. It may attest a deliberately Disabled installation or a separately re-enabled installation, but cannot Enable it.

`RollbackReadinessEvidence` is freshly minted at rollback time and binds the immutable rollback pin, current target installation revision, current configuration revision/digest and verified rollback artifact/admission digests. Configuration drift after Apply therefore cannot silently reuse stale rollback evidence.

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
UpdateCompletionEvidence
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

Every persisted update event is reachable through `decide`. Every subordinate installation/grant event is independently reachable through its existing owner `decide`. Update replay validates the update stream and its recorded subordinate identities/digests; repository load additionally cross-checks those bindings against the actual installation/grant streams before returning a snapshot.

Checked public command constructors are exactly `stage`, `record_approval`, `apply`, `complete`, `rollback`, `cancel` and `cancel_after_terminal_installation`. Actions and payloads remain private. Command accessors are only `command_id` and `update_id`. Event accessors are `sequence`, `post_revision`, `command_id`, `update_id` and `kind`. Aggregate accessors are `update_id`, `installation_id`, `tenant_id`, `user_id`, `plan`, `state`, `revision`, `last_sequence`, optional approval/readiness evidence, applied installation revision and optional terminal evidence. Evidence values expose their identity/binding digests and complete typed authority bindings read-only; no mutable accessor exists.

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

Every non-Stage command carries exact `expected_update_revision`. Apply/Rollback/Complete additionally carry exact `expected_installation_revision`.

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

Complete
  UpdateCommandId, PackageUpdateId, expected UpdateRevision,
  expected InstallationRevision, UpdateCompletionEvidence

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
    Completed,
    RolledBack,
    Cancelled,
}
```

Every event carries `UpdateEventSequence`, post `UpdateRevision`, `UpdateCommandId`, `PackageUpdateId` and a private complete typed payload. `Staged` stores the immutable complete plan. `ApprovalRecorded` stores both evidence values. `Applied` stores exact pre/post installation revisions, target pin digest, one exact subordinate installation-event reference and the complete deterministic set of subordinate grant-stale event references. `Completed` stores completion evidence. `RolledBack` stores exact pre/post installation revisions, rollback pin digest, fresh rollback evidence, one exact subordinate installation-event reference and subordinate grant-stale event references. No event carries arbitrary prose, raw secret, rejected source payload, artifact bytes or adapter diagnostics.

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

## 7. Decision and repository precedence

For one new command, precedence is:

```text
update command-ledger exact replay / conflicting reuse
→ consumed UpdateApprovalId uniqueness where applicable
→ one-current-update-per-installation index
→ update aggregate missing/already present
→ installation missing or terminal
→ update identity/state/revision
→ installation tenant/user/revision/current-pin/configuration coherence
→ transaction-current catalog/revocation/registry/policy authority coherence
→ immutable plan / computed change-class / approval / readiness coherence
→ Apply or Rollback requires InstalledDisabled|Disabled
→ transaction-current complete grant-set and subordinate-command coherence
→ event/update/installation/grant sequence overflow
→ atomic commit
```

A domain rejection persists the complete original command and exact typed rejected receipt. Identical retry returns it even after later state changes. A pre-commit persistence/transaction failure records no update, installation or grant event and consumes no command or approval identity.

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
CompletionEvidenceIncoherent
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
CompletionEvidenceMismatch
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

Replay rejects at least: non-Staged initial event; sequence gap/duplicate/reorder/overflow; duplicate command or approval ID; post-terminal event; illegal transition; post-revision forgery; update/installation/tenant/user/package identity mismatch; plan/change-class/evidence digest forgery; Apply/Rollback without exactly one referenced installation event of the expected identity, kind and complete canonical digest; incomplete/extra/forged subordinate grant stale-event references; and a Completed/RolledBack state not reachable through `decide`. Repository load additionally requires every referenced installation/grant event to exist byte-semantically in the exact subordinate stream and rejects an unreferenced coupled transition, wrong stream position or digest mismatch.

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
- a latest-version resolver or SemVer update policy;
- artifact download, package manager, filesystem switch, health-check implementation or cleanup scheduler;
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

Before any proposal merge or Rust implementation, Develata must explicitly accept, reject or amend this entire digest-bound packet and activate a finite implementation grant. Acceptance must confirm at least:

- Conservative MVP remains the intended first version;
- update/installation/grant ownership and atomic cross-stream commit are acceptable;
- the exact state graph, command/event/error surfaces and evidence boundaries are acceptable;
- `MARKET-004` and `PKG-020` remain planned with the future bindings above;
- the future implementation path set and non-goals are acceptable.

Until then, current accepted `market-lifecycle/v0` and existing Rust behavior remain unchanged.
<!-- M20_B6_READINESS_PACKET:END -->
