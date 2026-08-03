# M20-B7 application/composition contract-readiness proposal

## 0. Authority and status

- `Status`: **repaired exact packet accepted; docs/contract amendment under final review; source-control shipping and implementation closed**
- `Selected profile`: `P1 — staged contract-first composition`, selected by Develata on `2026-08-02`
- `Selected B7-A breadth`: `A1 — narrow product vertical`, selected by Develata on `2026-08-02`
- `Proposal directive`: Develata operation-specific instruction on `2026-08-02` to continue with the recommended M20-B7 activation/contract-packet step
- `Grant carrier`: none; the W1 campaign in [`01-execution-roadmap.md`](01-execution-roadmap.md#active-autonomous-module-campaign-authorization) does not include B7 and does not roll into another batch
- `Mode`: proposal-only local drafting
- `Candidate-v3 packet SHA-256`: `3aff295feafd86e6e9d108558ed8e64ebeac29851a9342432979fddab70ae6a1` over `37029` exact marker-delimited bytes
- `Candidate-v3 whole-file SHA-256`: `d7f7ba0de7d43127c8a828814b8c4bcc83f710dbaf047bbc5e251c768e770a0a` before later mutable receipt updates
- `Candidate-v3 disposition`: superseded; no v3 review verdict is reusable
- `Current repaired accepted packet`: `ab4873ca783c899618beca5add61a7d79ff2b305d95d8f4baac92502233b15c4` over `42111` exact marker-delimited bytes
- `Candidate-v4 packet SHA-256`: `57cd33b80f63a230388b729d9446ad5634d7ed628f4d97610395d5801ae8c9f9` over `37973` exact marker-delimited bytes; superseded by B7-B ordering and A1 signature repairs
- `Candidate-v4 whole-file SHA-256`: `78d29fed100106b2793c45777d28ad4cce7163a88270d0c9a0539903b7ea1ac6` before later mutable receipt updates
- `Implementation authority`: closed
- `Source-control shipping authority`: closed; no commit, push, PR, merge, tag or release is authorized by this file

### Candidate-v4 review and gate receipt

- `Observed at`: `2026-08-02T11:03:21Z`
- `Source rebind`: `HEAD == origin/main == 2f4de29032560ff3e13d9994b33a3aff14243f44`; tree `53e266c47fdb07d50a734faa24bb11ac4bc5527d`
- `B7-A1 application lane`: `PASS`; packet `57cd33b80f63a230388b729d9446ad5634d7ed628f4d97610395d5801ae8c9f9`; delegation `deleg_cc61bbe4/task-0`
- `B7-B composition lane`: `PASS`; packet `57cd33b80f63a230388b729d9446ad5634d7ed628f4d97610395d5801ae8c9f9`; delegation `deleg_cc61bbe4/task-1`
- `Governance/checker lane`: `PASS`; packet `57cd33b80f63a230388b729d9446ad5634d7ed628f4d97610395d5801ae8c9f9`; Codex result `/opt/data/tmp/m20b7-v4-lane3-codex/scratch/result.txt`; read-only snapshot before/after SHA-256 `eae21e3c337adfb261fb2b9c81414fadfa280733cd136efc1507bc4b8f2d98de`
- `Repository checker`: `python3 scripts/check_repo_contracts.py --ci` → `contract-check: PASS`
- `Existing checker unittest baseline`: `python3 -m unittest scripts.tests.test_check_repo_contracts` → `Ran 471 tests`; `OK`
- `Review result`: `3/3 PASS`; blockers `none`; findings `none`

### Whole-packet acceptance receipt

- `Repaired packet accepted at`: `2026-08-02T12:14:28Z`
- `Repaired packet accepted by`: Develata, explicit selection `接受 repaired packet ab4873ca…（推荐）`
- `Accepted repaired packet`: `ab4873ca783c899618beca5add61a7d79ff2b305d95d8f4baac92502233b15c4`; `42111` marker-delimited bytes
- `Accepted at`: `2026-08-02T11:05:39Z`
- `Superseded v4 acceptance by`: Develata, explicit selection `接受 exact packet 57cd33b8…（推荐）`
- `Superseded packet`: `57cd33b80f63a230388b729d9446ad5634d7ed628f4d97610395d5801ae8c9f9`; `37973` marker-delimited bytes
- `Accepted source`: commit `2f4de29032560ff3e13d9994b33a3aff14243f44`; tree `53e266c47fdb07d50a734faa24bb11ac4bc5527d`
- `Authority opened`: docs/contract amendment drafting on the packet's exact governance path set
- `Authority still closed`: production/test implementation, commit, push, PR, merge, tag and release

### Docs/contract amendment draft receipt

- `Drafted at`: `2026-08-02T11:17:36Z`
- `Current repaired packet recheck`: `ab4873ca783c899618beca5add61a7d79ff2b305d95d8f4baac92502233b15c4`; `42111` bytes
- `Mode`: docs contract/status projection plus operation-specific narrow checker projection; no Rust, Cargo, CI, workflow or dependency edit
- `Exact dirty paths`:
  - `docs/contracts/market-lifecycle.md`
  - `docs/contracts/agent-plugin-boundary.md`
  - `docs/contracts/interfaces.md`
  - `docs/contracts/module-boundaries.md`
  - `docs/plan/modules/30-market-package-lifecycle.md`
  - `docs/plan/modules/50-tool-gateway-execution.md`
  - `docs/coverage-matrix.md`
  - `docs/acceptance/matrix.tsv`
  - `docs/acceptance/platform-baseline.md`
  - `docs/tasks/01-execution-roadmap.md`
  - `docs/tasks/m20-b7-contract-readiness-proposal.md`
  - `scripts/check_repo_contracts.py`
  - `scripts/tests/test_check_repo_contracts.py`
- `Status projection`: MARKET-001–MARKET-004, MARKET-007, PKG-019, PKG-020 and FP-007 remain `planned`; no case is promoted
- `Implementation projection`: A1 implemented as semantic in-memory evidence with zero production callers; B7-B exact contract accepted with source/tests absent; M20/M40 remain `partial-evidence`

### Docs/contract amendment repair receipt

- `Repair at`: `2026-08-02T12:00:55Z`
- `Superseded candidate`: v1 patch SHA-256 `126525805c607445463977d3cf0475d6fb79ba223dd3fedace69d6264059366a`
- `Reviewer finding`: DOCS-B7B-COMPOSITION v1 BLOCK — candidate prose placed `ToolCallProposal` persistence before frozen `AgentToolsetView::bind_call`/`AgentToolCall` binding, which would permit unknown/unbound raw proposals or violate the current `ToolCallProposal::from(&AgentToolCall)` carrier
- `Repair scope`: `docs/contracts/agent-plugin-boundary.md`, `docs/contracts/market-lifecycle.md`, `docs/contracts/module-boundaries.md`
- `Repair content`: proposal persistence now occurs only after composition binds provider raw call through frozen `AgentToolsetView::bind_call`; unknown tool fails before any M30 proposal; M20 still creates neither proposal nor intent and does not call executors
- `Review consequence`: all v1 reviewer receipts are superseded; v2 requires new patch hash, gates and final review
- `Further repair`: DOCS-A1-CONTRACT v2 identified underspecified request/view accessor return types and incomplete `InMemoryCatalogReadRepository::try_new` signature; DOCS-B7B-COMPOSITION v2 identified the same bind-before-proposal issue inside the marker-delimited packet itself. The current packet candidate repairs both and supersedes every v1/v2 review receipt.

### Late A1 owning-contract projection repair receipt

- `Observed at`: `2026-08-02T13:03:40Z`
- `Late reviewer`: v5 DOCS-A1-CONTRACT receipt bound to patch `9498eba01fc98d3f5eb0a3aca1cf4bb7b2e2de24795441984b3bf5c35ae2442b`
- `Verified blocker`: [`../contracts/market-lifecycle.md`](../contracts/market-lifecycle.md) claimed ownership of the complete exact A1 Rust signature contract but delegated every query/request/view accessor signature to this lower-authority task packet
- `Why it carries forward`: the v5→v6 delta changed only B7-B taskbook ordering; the owning A1 contract bytes were unchanged, so the narrower source-backed BLOCK supersedes the later broad v6 A1 PASS for this axis
- `Repair`: the owning `market-lifecycle/v1` contract now enumerates every exact query/request and view accessor signature directly; the task packet remains the accepted source receipt rather than a peer authority
- `Packet consequence`: none; the accepted marker-delimited packet remains byte-identical at `ab4873ca783c899618beca5add61a7d79ff2b305d95d8f4baac92502233b15c4` / `42111` bytes
- `Candidate consequence`: v6 patch `95c752d845479ff68ceb64f95c7143f5449cb081d8ec4e2e8b6d1e6ddc108635` and all patch-bound v6 review receipts are superseded; a new patch hash, full checker gates and final review are required

### Docs-checker projection blocker

- `Observed gate`: `python3 scripts/check_repo_contracts.py --ci` → FAIL on exactly one stale marker
- `Stale marker`: `Remaining work is M20-B7 production-facing API/composition/fake M40 consumer plus future durable adapters` (code formatting omitted here to avoid creating a decoy match)
- `Why stale`: the accepted packet replaces that catch-all with ordered A1 → B7-B, while A2, M90 and M10/M80 remain separately owned; restoring the old sentence as a superseded-prose decoy would make the checker false-green
- `Requested narrow amendment paths`: `scripts/check_repo_contracts.py`, `scripts/tests/test_check_repo_contracts.py`
- `Requested amendment`: replace the obsolete marker with exact accepted/unimplemented B7 decomposition and no-source/no-promotion markers; add one-axis deletion/drift mutations for those markers
- `Scope authorization`: Develata selected `授权最小 checker projection（推荐）` at `2026-08-02T11:33:31Z`; only the two requested paths are admitted
- `Still forbidden`: Rust/Cargo/dependency/CI/workflow edits, implementation, acceptance promotion and source-control shipping

This document cannot grant work to itself. Develata's explicit receipt above accepts only the exact marker-delimited packet for docs/contract amendment drafting. Existing Rust implementation, acceptance status and source-control authority remain unchanged until their separately named gates are satisfied.

## 1. Exact source binding

| field | receipt |
|---|---|
| repository | `Develata/ustc-campus-agent` |
| branch | local `docs/m20-b7-contract-readiness`; not pushed |
| authoritative source commit | `2f4de29032560ff3e13d9994b33a3aff14243f44` |
| authoritative source tree | `53e266c47fdb07d50a734faa24bb11ac4bc5527d` |
| source relation | local `HEAD == origin/main` in a dedicated clean worktree at discovery time |
| source receipt time | `2026-08-02T10:02:15Z` |
| exact-main CI | [run 30741450134](https://github.com/Develata/ustc-campus-agent/actions/runs/30741450134), exact head above, `rust=PASS`, `docs-and-contracts=PASS` |
| fresh local checker | `python3 scripts/check_repo_contracts.py --ci` -> `contract-check: PASS`; exit `0`; log `/tmp/m20b7-proposal-baseline-checker.log` |

A moving `main` invalidates this binding before packet acceptance or implementation. Rebinding requires a fresh exact-main checkout, new tree receipt and renewed packet review.

## 2. Grounded current state

### 2.1 Implemented M20 carriers

M20 B1–B6 now provide bounded domain/semantic-fake evidence for:

- catalog declaration, validation, publication and deterministic read models;
- capability registry and permission-delta algebra;
- installation state, revision, events and semantic repository;
- grant state, replacement/revoke/stale semantics and semantic repository;
- projection-time invocation resolution plus transaction-current call recheck through `InvocationAuthorityService<R>`;
- package update/rollback state, exact approval/readiness evidence, installation/grant coupling and atomic in-memory repository semantics.

The active Rust ports remain domain-shaped, including `InstallationRepository`, `GrantRepository`, `PackageUpdateRepository` and `InvocationAuthorityRepository`. `InvocationAuthorityService<R>` already performs one transaction-consistent projection assembly and one current-authority recheck with post-read precondition verification.

### 2.2 Missing B7 carriers

The following production/application carriers do not yet exist:

1. one M20-owned application query/command contract suitable for M10 dispatch;
2. explicit caller/admission/idempotency/precondition inputs that cannot mint installation, grant, update or policy evidence;
3. safe read projections for catalog, installation, grant and update state without exposing domain internals or authority-bearing constructors;
4. a durable implementation of the semantic repository/transaction boundary;
5. composition-root ordering from a frozen M20 projection through frozen-view call binding, M30 bound-call proposal/intent/receipt and staged M40 execution;
6. in-flight/new-projection proof across update, disable and revoke;
7. real M10 wire routes or M80 browser/client consumers.

`docs/contracts/interfaces.md` currently lists only draft browse/detail, install and disable HTTP routes. It is not an application-port contract and does not cover enable, revoke/uninstall, grant replacement/reapproval or update lifecycle operations.

### 2.3 Dependency readiness

| module | current state | implication for B7 |
|---|---|---|
| M10 | skeleton; application ingress planned | B7 may define framework-neutral application ports, not HTTP/server-function DTOs or handlers |
| M20 | B1–B6 bounded evidence complete; application composition absent | ready for an exact application-contract proposal and semantic fakes |
| M30 | `EffectIntent`/`EffectReceipt` command and replay carriers exist | composition can prove persistence ordering without making M20 own run truth |
| M40 | protocol values and monolithic test-only fake gateway/executor exist; staged execution API absent | current-call composition needs a companion M40 stage contract or remains a test-only imitation |
| M90 | governance baseline; production infrastructure planned | durable adapter implementation must be owned by a separately activated M90 slice, not smuggled into M20 |

### 2.4 Honest acceptance gaps

- `MARKET-002`: durable catalog/installation/grant/policy provenance and B7 application composition missing.
- `MARKET-003`: durable current-state/discovery composition missing.
- `MARKET-004`: B7 current-call composition and production grant-reapproval/current API evidence missing.
- `MARKET-007`: M30 effect-intent plus fake M40/M51 no-I/O ordering missing.
- `PKG-019`: package compiler/private-route integration missing.
- `PKG-020`: immutable in-flight projection plus current-call/new-projection composition missing.
- `MARKET-001` and `FP-007`: real M10/M80/API/browser evidence missing.

No profile below may promote a row merely because the contract is accepted.

## 3. Non-negotiable ownership boundary

All profiles preserve these rules:

```text
M20 owns package/install/grant/update authority and M20 application semantics.
M30 owns run proposal, EffectIntent, EffectReceipt and run replay truth.
M40 owns call normalization, staged execution, executor port and bounded result mapping.
M90 implements durable ports declared by owners; it does not define domain rules.
M10 maps one admitted request to one application operation; it reaches no concrete repository/executor.
apps/ustc-agentd is the only composition root allowed to depend on M20 + M30 + M40 + selected adapters.
```

Consequences:

- no M20 dependency on M30/M40 implementations, database, framework, clock, network, executor or transport;
- no M10 handler reaches a concrete repository or executor;
- no M40 path issues grants, mutates installation/update state or writes M30 truth;
- no adapter-authored allow boolean or caller-minted authority evidence;
- no fake may bypass owner `decide`/`evolve`/replay or claim production durability;
- no effect/executor I/O precedes an accepted M30 `EffectIntent`;
- frozen tool views remain immutable, while each call still rechecks current deny-side authority.

## 4. Coherent activation profiles

### Profile P1 — staged contract-first composition (**recommended**)

Activate B7 as a reviewed multi-slice contract, but authorize implementation one finite slice at a time:

1. **B7-A — M20 application contract + semantic fake**
   - framework-neutral M20 query/command ports and safe projections;
   - A1 request types carry bounded tenant/user scope claims but create no admission authority and have zero production call sites; only a later M00-admitted M10 mapping may call them;
   - later A2 authority-bearing mutations require trusted injected policy/admission issuers to construct sealed owner evidence internally;
   - no automatic grant/bootstrap policy, wire DTO, DB adapter or M40 dependency;
   - conformance over existing B1–B6 semantic repositories.
2. **B7-B — composition-root current-call proof**
   - companion staged M40 fake contract plus `ustc-agentd` orchestration;
   - exact `frozen AgentToolsetView::bind_call → bound-call M30 proposal → current recheck/preparation → M30 EffectIntent persistence → fake executor/reconciliation → M30 EffectReceipt persistence → correlated result` order;
   - disable/revoke/update fixtures prove frozen in-flight view identity, future-projection replacement and current denial before intent/I/O;
   - no production network or executor.
3. **M90 follow-up — durable adapter conformance**
   - separately activated M90-owned adapter implements the same M20 ports/transaction semantics;
   - backend technology and migration/recovery contract are selected there, not inside B7-A.
4. **M10/M80 follow-up — real API/client evidence**
   - separately activated wire and UI slices map to the accepted application ports.

Likely honest status after B7-A+B7-B: `MARKET-007` and possibly `PKG-020` may become eligible for exact evidence review; `MARKET-001`–`MARKET-004` and `FP-007` remain planned until their durability/current-API/browser gates are complete.

Why recommended: it closes the highest-risk authority/intent/I/O seam now, preserves large-module direction and avoids inventing M90 or M10 contracts inside M20.

### Profile P2 — current-call risk first

Activate only B7-B now. Refactor the monolithic test fake into staged M40 preparation/execution/result fakes and prove M20 current recheck plus M30 intent/receipt ordering at `ustc-agentd`.

- smallest cross-module risk closure;
- no M20 application queries/mutations or durable adapters;
- does not satisfy the roadmap's complete B7 description;
- application-contract design is deferred rather than partially guessed.

### Profile P3 — one full B7 bundle

Define and implement M20 application queries/mutations, one durable store adapter, M40 current-call composition and expanded interfaces in one Path-A batch.

- closest literal reading of the roadmap B7 line;
- requires simultaneous M20/M40/M90 and likely M10 contract amendments;
- largest common-mode failure and review surface;
- cannot honestly complete browser evidence without later M80 work.

This profile is not recommended on the current dependency state.

### Profile P4 — dependency-first deferral

Do not activate B7 yet. Activate exact M40 staged-execution and M90 durable-storage contracts first; return to B7 after both are accepted.

- clean dependency order;
- delays the already-ready M20 application-port and semantic composition work;
- leaves `MARKET-007`/`PKG-020` integration gaps open longer.

## 5. Selection semantics

Selecting a profile authorizes only the next **exact proposal** to be written. It does not authorize Rust implementation or source-control shipping.

After selection, the exact packet must freeze:

1. finite sub-slices and stop conditions;
2. exact owner/dependency/import boundaries;
3. public application values, commands, queries, errors and safe projections;
4. evidence-issuer and transaction semantics;
5. staged M40/M30 ordering if included;
6. exact writable path/public-item/test/checker inventories;
7. acceptance-row mappings and explicit non-promotions;
8. focused/full validation commands;
9. independent proposal reviewers and blocker policy;
10. a delimited SHA-256-bound packet for Develata's whole-packet acceptance.

## 6. Explicitly out of scope before exact packet acceptance

- production Rust edits;
- database schema, migration or adapter selection;
- M10 HTTP/server-function route implementation;
- M80 Web/Android/CLI/MCP work;
- real M51/MCP or hosted executor I/O;
- automatic initial grants or first-party bootstrap policy;
- acceptance status promotion;
- commit, push, PR, merge, tag or release.

<!-- M20_B7_EXACT_PACKET:BEGIN -->
## 7. Selected exact posture

This packet is bound to source commit `2f4de29032560ff3e13d9994b33a3aff14243f44`, tree `53e266c47fdb07d50a734faa24bb11ac4bc5527d`, with exact-main CI run `30741450134` PASS. Any source commit/tree drift invalidates the packet and all reviews; rebinding requires a new packet digest and full three-lane review.

Develata selected `P1 — staged contract-first composition` and `A1 — narrow product vertical` on `2026-08-02`. The proposed sequence is exact:

1. **B7-A1** freezes and later implements one M20-owned, framework-neutral application façade over existing B1–B6 semantic ports: anonymous catalog browse/detail, owner-scoped installation/grant/update reads and owner-scoped Disable.
2. **B7-B** later adds composition-root semantic evidence for current M20 recheck, M30 intent/receipt persistence and a staged fake M40/executor. It follows A1 and has a separate implementation grant.
3. **B7-A2** later owns Install, Configure, Enable, Revoke, Uninstall, grant Issue/Replace/Revoke and package-update mutation application contracts only after M00 actor/admission and issuer policy are frozen.
4. **M90** later supplies durable implementations of existing owner ports under a separately accepted storage/migration/recovery contract.
5. **M10/M80** later map accepted application operations to real wire/client surfaces.

Acceptance of this packet does not activate any of those implementations. Every retained implementation slice requires an operation-specific finite grant after its owning contract amendments are merged. Completion of A1 does not automatically authorize B7-B, A2, M90, M10 or M80.

## 8. B7-A1 operation surface

### 8.1 Admitted operations

The exact A1 operation set is:

```text
BrowseCatalog
ReadPackageDetail
ReadOwnedInstallation
ReadOwnedCurrentGrants
ReadOwnedPackageUpdate
DisableOwnedInstallation
```

The existing `InvocationAuthorityService<R>` remains the sole M20 projection/current-call service. A1 documents it as an admitted internal application port but adds no wrapper, alternative resolver, allow boolean or duplicate authority model.

No operation chooses a latest version, infers ownership, creates an installation, issues/replaces a grant, mints approval/evidence, applies an update or serializes a wire protocol.

### 8.2 Catalog paging

`BrowseCatalog` uses one exact `CatalogReadModel`:

- first-page requests may omit a catalog revision and therefore select the repository's exact current reviewed revision;
- continuation requests carry the exact returned `CatalogRevision` plus a nonzero `u32` offset; `offset > 0` with no revision is rejected at query construction;
- `CatalogPageLimit` is in `1..=100`;
- packages retain the canonical `(package_id, package_version)` order already owned by `CatalogReadModel`;
- the response returns exact catalog revision/digest, the bounded page and an optional checked next offset;
- an exact but unavailable revision is `NotFound`; overflow/corruption fails closed;
- no offset from one catalog revision may continue against another.

`ReadPackageDetail` resolves an exact package ID and exact version in an exact or current reviewed catalog revision. There is no fuzzy match, same-name fallback, hidden latest selection or search in A1.

### 8.3 Owner-scoped reads

Owner-scoped read requests carry existing checked `TenantId` and `UserId` plus the exact owned object identity. Those values are downstream scope claims, **not authentication or authority evidence**. A1 defines no M20 actor/session type and has zero production call sites. A later M10 adapter may supply the values only by mapping a current M00-admitted request context; tenant/user fields from a client body/query/header are forbidden. Until M00-B3/B5 and the M10 mapping contract are accepted, these operations are usable only by semantic tests and cannot be exposed as routes/server functions.

- installation read returns one exact safe projection;
- grant read additionally carries exact expected `InstallationRevision`, loads only the complete current nonterminal grant set owned by `GrantRepository`, and returns grants in that set's canonical order;
- update read returns one exact update projection;
- absent and foreign-tenant/foreign-user objects both map to `NotFound`, preventing an ownership oracle;
- independent safe reads expose their owner revisions and do not claim a cross-repository atomic snapshot;
- A1 may not join independently observed values into an invocation-authority decision; only `InvocationAuthorityService<R>` may do so under one verified read transaction.

### 8.4 Disable mutation

`DisableOwnedInstallation` carries exactly:

```text
InstallationCommandId
TenantId
UserId
InstallationId
expected InstallationRevision
```

The service must:

1. load the exact installation;
2. map absence or owner mismatch to `NotFound`;
3. construct exactly one existing `InstallationCommand::disable` using the request's same ID and expected revision;
4. execute it through `InstallationRepository`, whose owner ledger resolves an exact command replay before evaluating a new command against current revision;
5. return a safe **command receipt projection**, not a claim that no later state exists.

Ownership is immutable in the installation aggregate, and the repository's expected-revision decision closes the load/execute race for a new command. The façade must not pre-reject an old expected revision before the owner command ledger gets a chance to return an exact prior receipt. A different command under a stale revision or a conflicting reuse of one command ID maps to `Conflict`; a legal-state denial maps to `LifecycleDenied`; repository corruption or unavailability never becomes success. A repeated typed-equal command returns the prior receipt projection and cannot emit a second event. The façade neither disables grants itself nor mints enable/grant/update evidence; current invocation denial follows from the installation state through the existing resolver/recheck path.

The ownership comparison prevents cross-object confusion inside M20 but is not, by itself, caller admission. `DisableOwnedInstallation` has the same zero-production-call-site rule as the reads; no implementation may describe it as an authenticated user operation before the admitted M00→M10→M20 mapping exists.

## 9. B7-A1 exact Rust inventory

### 9.1 New module and public items

One new module is proposed at `crates/platform-core/src/market/application.rs`, exported as `pub mod application;`. Its exact public item set is:

```text
MarketApplicationConstructionError
MarketApplicationRepositoryError
MarketApplicationError
CatalogPageLimit
CatalogBrowseQuery
CatalogPackageQuery
OwnedInstallationQuery
OwnedInstallationGrantQuery
OwnedUpdateQuery
DisableInstallationRequest
MarketPackageSummary
MarketPackageDetail
MarketCatalogPage
MarketInstalledComponentView
MarketPackagePinView
MarketInstallationView
MarketGrantView
MarketGrantPage
MarketUpdateView
DisableInstallationReceiptView
CatalogReadRepository
InMemoryCatalogReadRepository
MarketApplicationService
```

No other public struct, enum, trait, type alias, constant, macro or free function enters this module in A1.

### 9.2 Construction, accessor and visibility rules

Exact public constructors and request accessors are:

```rust
CatalogPageLimit::new(value: u16) -> Result<Self, MarketApplicationConstructionError>
CatalogPageLimit::get(&self) -> u16
CatalogBrowseQuery::new(revision: Option<CatalogRevision>, offset: u32, limit: CatalogPageLimit) -> Result<Self, MarketApplicationConstructionError>
CatalogPackageQuery::new(revision: Option<CatalogRevision>, package_id: PackageId, package_version: PackageVersion) -> Self
OwnedInstallationQuery::new(tenant_id: TenantId, user_id: UserId, installation_id: InstallationId) -> Self
OwnedInstallationGrantQuery::new(tenant_id: TenantId, user_id: UserId, installation_id: InstallationId, expected_installation_revision: InstallationRevision) -> Self
OwnedUpdateQuery::new(tenant_id: TenantId, user_id: UserId, update_id: PackageUpdateId) -> Self
DisableInstallationRequest::new(command_id: InstallationCommandId, tenant_id: TenantId, user_id: UserId, installation_id: InstallationId, expected_revision: InstallationRevision) -> Self
```

Their only remaining public inherent methods are these exact read-only accessors:

```rust
CatalogBrowseQuery::catalog_revision(&self) -> Option<&CatalogRevision>
CatalogBrowseQuery::offset(&self) -> u32
CatalogBrowseQuery::limit(&self) -> CatalogPageLimit
CatalogPackageQuery::catalog_revision(&self) -> Option<&CatalogRevision>
CatalogPackageQuery::package_id(&self) -> &PackageId
CatalogPackageQuery::package_version(&self) -> &PackageVersion
OwnedInstallationQuery::tenant_id(&self) -> &TenantId
OwnedInstallationQuery::user_id(&self) -> &UserId
OwnedInstallationQuery::installation_id(&self) -> &InstallationId
OwnedInstallationGrantQuery::tenant_id(&self) -> &TenantId
OwnedInstallationGrantQuery::user_id(&self) -> &UserId
OwnedInstallationGrantQuery::installation_id(&self) -> &InstallationId
OwnedInstallationGrantQuery::expected_installation_revision(&self) -> &InstallationRevision
OwnedUpdateQuery::tenant_id(&self) -> &TenantId
OwnedUpdateQuery::user_id(&self) -> &UserId
OwnedUpdateQuery::update_id(&self) -> &PackageUpdateId
DisableInstallationRequest::command_id(&self) -> &InstallationCommandId
DisableInstallationRequest::tenant_id(&self) -> &TenantId
DisableInstallationRequest::user_id(&self) -> &UserId
DisableInstallationRequest::installation_id(&self) -> &InstallationId
DisableInstallationRequest::expected_revision(&self) -> &InstallationRevision
```

Exact view accessors are:

```rust
MarketPackageSummary::package_id(&self) -> &PackageId
MarketPackageSummary::package_version(&self) -> &PackageVersion
MarketPackageSummary::publisher(&self) -> &str
MarketPackageSummary::tier(&self) -> PackageTier
MarketPackageSummary::display_name(&self) -> &str
MarketPackageSummary::implementation_status(&self) -> ImplementationStatus
MarketPackageSummary::install_policy(&self) -> &InstallPolicy
MarketPackageDetail::catalog_revision(&self) -> &CatalogRevision
MarketPackageDetail::catalog_digest(&self) -> &Sha256Digest
MarketPackageDetail::summary(&self) -> &MarketPackageSummary
MarketPackageDetail::description(&self) -> Option<&str>
MarketPackageDetail::components(&self) -> &[ComponentDeclaration]
MarketPackageDetail::capabilities(&self) -> &[CapabilityId]
MarketPackageDetail::package_digest(&self) -> &Sha256Digest
MarketPackageDetail::component_declaration_set_digest(&self) -> &Sha256Digest
MarketPackageDetail::capability_manifest_digest(&self) -> &Sha256Digest
MarketPackageDetail::source_policy_digest(&self) -> &Sha256Digest
MarketCatalogPage::catalog_revision(&self) -> &CatalogRevision
MarketCatalogPage::catalog_digest(&self) -> &Sha256Digest
MarketCatalogPage::packages(&self) -> &[MarketPackageSummary]
MarketCatalogPage::next_offset(&self) -> Option<u32>
MarketInstalledComponentView::component_id(&self) -> &ComponentId
MarketInstalledComponentView::kind(&self) -> ComponentKind
MarketInstalledComponentView::version(&self) -> &ComponentVersion
MarketInstalledComponentView::digest(&self) -> &Sha256Digest
MarketPackagePinView::catalog_revision(&self) -> &CatalogRevision
MarketPackagePinView::package_id(&self) -> &PackageId
MarketPackagePinView::package_version(&self) -> &PackageVersion
MarketPackagePinView::package_digest(&self) -> &Sha256Digest
MarketPackagePinView::components(&self) -> &[MarketInstalledComponentView]
MarketPackagePinView::component_set_digest(&self) -> &Sha256Digest
MarketPackagePinView::capability_manifest_digest(&self) -> &Sha256Digest
MarketInstallationView::installation_id(&self) -> &InstallationId
MarketInstallationView::package_pin(&self) -> &MarketPackagePinView
MarketInstallationView::state(&self) -> ManagedInstallationState
MarketInstallationView::revision(&self) -> &InstallationRevision
MarketInstallationView::configuration_revision(&self) -> &ConfigurationRevision
MarketInstallationView::configuration_digest(&self) -> &Sha256Digest
MarketGrantView::snapshot_id(&self) -> &GrantSnapshotId
MarketGrantView::installation_id(&self) -> &InstallationId
MarketGrantView::installation_revision(&self) -> &InstallationRevision
MarketGrantView::catalog_revision(&self) -> &CatalogRevision
MarketGrantView::package_id(&self) -> &PackageId
MarketGrantView::package_version(&self) -> &PackageVersion
MarketGrantView::package_digest(&self) -> &Sha256Digest
MarketGrantView::capability_id(&self) -> &CapabilityId
MarketGrantView::capability_definition(&self) -> &CapabilityDefinition
MarketGrantView::scope(&self) -> &GrantScope
MarketGrantView::confirmation_policy(&self) -> &ConfirmationPolicy
MarketGrantView::state(&self) -> GrantState
MarketGrantView::version(&self) -> &GrantVersion
MarketGrantPage::installation_id(&self) -> &InstallationId
MarketGrantPage::observed_installation_revision(&self) -> &InstallationRevision
MarketGrantPage::grants(&self) -> &[MarketGrantView]
MarketUpdateView::update_id(&self) -> &PackageUpdateId
MarketUpdateView::installation_id(&self) -> &InstallationId
MarketUpdateView::rollback_pin(&self) -> &MarketPackagePinView
MarketUpdateView::target_pin(&self) -> &MarketPackagePinView
MarketUpdateView::change_class(&self) -> &UpdateChangeClass
MarketUpdateView::state(&self) -> UpdateState
MarketUpdateView::revision(&self) -> &UpdateRevision
MarketUpdateView::applied_installation_revision(&self) -> Option<&InstallationRevision>
DisableInstallationReceiptView::command_id(&self) -> &InstallationCommandId
DisableInstallationReceiptView::installation_id(&self) -> &InstallationId
DisableInstallationReceiptView::post_state(&self) -> ManagedInstallationState
DisableInstallationReceiptView::post_revision(&self) -> &InstallationRevision
```

Exact port/service methods and signatures are:

```rust
pub trait CatalogReadRepository {
    fn load_current(
        &self,
    ) -> Result<Arc<CatalogReadModel>, MarketApplicationRepositoryError>;

    fn load_exact(
        &self,
        revision: &CatalogRevision,
    ) -> Result<Option<Arc<CatalogReadModel>>, MarketApplicationRepositoryError>;
}

impl InMemoryCatalogReadRepository {
    pub fn try_new(
        revisions: Vec<CatalogReadModel>,
        current_revision: CatalogRevision,
    ) -> Result<Self, MarketApplicationRepositoryError>;
}

impl<C, I, G, U> MarketApplicationService<C, I, G, U>
where
    C: CatalogReadRepository,
    I: InstallationRepository,
    G: GrantRepository,
    U: PackageUpdateRepository,
{
    pub fn new(catalogs: C, installations: I, grants: G, updates: U) -> Self;
    pub fn browse_catalog(
        &self,
        query: &CatalogBrowseQuery,
    ) -> Result<MarketCatalogPage, MarketApplicationError>;
    pub fn package_detail(
        &self,
        query: &CatalogPackageQuery,
    ) -> Result<MarketPackageDetail, MarketApplicationError>;
    pub fn installation(
        &self,
        query: &OwnedInstallationQuery,
    ) -> Result<MarketInstallationView, MarketApplicationError>;
    pub fn current_grants(
        &self,
        query: &OwnedInstallationGrantQuery,
    ) -> Result<MarketGrantPage, MarketApplicationError>;
    pub fn package_update(
        &self,
        query: &OwnedUpdateQuery,
    ) -> Result<MarketUpdateView, MarketApplicationError>;
    pub fn disable_installation(
        &mut self,
        request: DisableInstallationRequest,
    ) -> Result<DisableInstallationReceiptView, MarketApplicationError>;
}
```

`Arc` is `std::sync::Arc`. Exact catalog revisions are immutable and shared; selecting a model is O(log N) plus one Arc clone, while only the bounded result page is cloned into application views. `InMemoryCatalogReadRepository::try_new` is the only public inherent method of that fake; its `load_current` and `load_exact` methods are available only through the trait implementation. `MarketApplicationService::new` and the six operations above are its only public inherent methods.

Additional rules:

- `MarketApplicationConstructionError`, `MarketApplicationRepositoryError` and `MarketApplicationError` derive exactly `Debug, Clone, Copy, PartialEq, Eq`;
- `CatalogPageLimit` derives exactly `Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash` and has a manual bounded `Debug`;
- all query/request/view types derive exactly `Clone, PartialEq, Eq` and have manual bounded/redacted `Debug`;
- `InMemoryCatalogReadRepository` derives only `Clone` and has manual authority-redacted `Debug`;
- `MarketApplicationService` derives no trait and has one manual authority-redacted `Debug` that requires no inner repository `Debug` bound;
- all fields are private;
- request/query constructors are public, checked and expose only the accessors above;
- response/view constructors are private to `application.rs`; only the application service maps owner values into them;
- `CatalogPageLimit::new(u16)` accepts only `1..=100`;
- `InMemoryCatalogReadRepository::try_new(Vec<CatalogReadModel>, CatalogRevision)` accepts `1..=64` exact revisions, rejects duplicates and a missing current revision, and exposes no arbitrary insert/replace/current-revision mutator; the bound is a private implementation constant, not public API;
- `MarketApplicationService::new(C, I, G, U)` requires `C: CatalogReadRepository`, `I: InstallationRepository`, `G: GrantRepository`, `U: PackageUpdateRepository`;
- the service exposes no `into_parts`, mutable repository accessor, raw event/history API or authority-evidence constructor;
- no new type implements `Default`, `Deref`, `DerefMut`, unchecked/raw conversion or mutable field access;
- no new type derives or implements `Serialize`/`Deserialize`; M10 owns later wire DTO mapping;
- every new request, view, service and fake repository with identity, ownership, state, digest or authority-bearing content has manual bounded/redacted `fmt::Debug`; fieldless stable error enums may derive `Debug`;
- no error Display/Debug includes caller input, configuration values, source-policy values, evidence payloads or repository internals.

### 9.3 Exact safe-view fields

`MarketPackageSummary` contains only:

```text
PackageId, PackageVersion, publisher, PackageTier, display_name,
ImplementationStatus, InstallPolicy
```

`MarketPackageDetail` contains the summary plus:

```text
exact CatalogRevision and catalog digest
optional description
reviewed ComponentDeclaration list
reviewed CapabilityId list
package/component-set/capability-manifest/source-policy digests
```

The raw source-policy map is deliberately excluded: the current B1 validator bounds control-free strings but does not classify secret material or private endpoints. A1 exposes only its digest until a later public-metadata contract defines safe typed fields. A1 also does not invent source/license fields absent from `ValidatedPackageManifest`; therefore it does not claim `WEB-003` completion.

`MarketInstallationView` contains only:

```text
InstallationId
MarketPackagePinView
ManagedInstallationState
InstallationRevision
ConfigurationRevision
configuration digest
```

`MarketPackagePinView` contains catalog/package identity and digests plus `MarketInstalledComponentView { ComponentId, ComponentKind, ComponentVersion, digest }`. It deliberately omits every `ExecutionIdentity`. The installation view also excludes configuration entries, `NonSecretText`, `SecretRef`, secret-ref IDs and event history.

`MarketGrantView` contains only:

```text
GrantSnapshotId
InstallationId
InstallationRevision
CatalogRevision
PackageId, PackageVersion, package digest
CapabilityId, CapabilityDefinition
GrantScope, ConfirmationPolicy
GrantState, GrantVersion
```

It excludes approval IDs, approval evidence, consumed-approval indexes and event history.

`MarketGrantPage` contains the exact installation ID, exact observed installation revision and the complete canonically sorted current nonterminal grant views. Every nested view repeats the same installation ID; any mismatch is `CorruptAuthority`. Revoked historical grants remain absent.

`MarketUpdateView` contains only:

```text
PackageUpdateId
InstallationId
rollback MarketPackagePinView
target MarketPackagePinView
UpdateChangeClass
UpdateState
UpdateRevision
optional applied InstallationRevision
```

It excludes approval/readiness/confirmation/rollback evidence, policy snapshots, private routes and event history.

`DisableInstallationReceiptView` contains the exact command ID, installation ID, accepted post-state and post-revision from the stored receipt. It is explicitly historical command disposition, not a current-state projection.

### 9.4 Error mapping

The exact fieldless error variants are:

```text
MarketApplicationConstructionError
  PageLimitOutOfRange
  UnboundContinuationOffset

MarketApplicationRepositoryError
  Unavailable
  EmptyCatalogHistory
  TooManyCatalogRevisions
  DuplicateCatalogRevision
  CurrentCatalogMissing
  CorruptCatalog

MarketApplicationError
  NotFound
  Conflict
  LifecycleDenied
  RepositoryUnavailable
  CorruptAuthority
```

`MarketApplicationConstructionError` is used only by checked application-value construction. `MarketApplicationRepositoryError` is the catalog port/fake error; an unavailable exact historical revision is `Ok(None)`, not repository corruption. Owner-domain repository errors map exhaustively into `MarketApplicationError`.

Absent and foreign-owned objects both become `NotFound`. Revision or command-ledger conflicts become `Conflict`; legal state-machine rejections become `LifecycleDenied`; I/O/injected persistence unavailability becomes `RepositoryUnavailable`; corrupt history/index/replay and any unknown newly added owner error become `CorruptAuthority` until deliberately classified. No error carries arbitrary text, caller input or a wrapped source error, and none falls through to success.

## 10. B7-A1 semantic fake contract

`InMemoryCatalogReadRepository` is deterministic, immutable after construction and semantic rather than production-durable. Existing `InMemoryInstallationRepository`, `InMemoryGrantRepository` and `InMemoryPackageUpdateRepository` remain the A1 domain fakes; A1 does not create a second aggregate/event implementation.

Fixtures must be built from checked manifests, owner commands, persisted receipts and replay-capable repository constructors. A test may seed matching histories into independent semantic repositories, but every response identifies its owner revision and no test may call the combination transaction-current invocation authority. Projection/recheck tests continue to use `InMemoryInvocationAuthorityRepository` plus `InvocationAuthorityService`.

A1 makes no claim about process restart, database transactions, migration, backup, restore, cross-repository atomic reads, event delivery or real API durability.

## 11. B7-B exact composition contract

### 11.1 Carrier policy

B7-B adds **no production M40 crate and no production application code**. It refactors composition-root test support into explicit stages and combines already-public M20, M30 and protocol carriers. The semantic test surface is not an alternative runtime authority.

Exact test-only support types are:

```text
StagedFakeToolGateway
PreparedFakeToolExecution
IdempotentFakePluginExecutor
InMemoryFakeRunJournal
EffectCompositionHarness
CompositionTraceEvent
```

They live under `apps/ustc-agentd/tests/support/` and are visible only to daemon integration tests. They do not enter any library's production public surface. Their exact test-visible method inventory is:

```text
StagedFakeToolGateway::prepare / complete
PreparedFakeToolExecution::authorized_invocation / protocol_call / effect_binding_digest
IdempotentFakePluginExecutor::execute_or_reconcile / lookup_disposition / fail_next_attempt / attempt_count / unique_effect_count
InMemoryFakeRunJournal::execute / fail_next_intent_persist / fail_next_receipt_persist / snapshot / events
EffectCompositionHarness::execute_call / reconcile_pending_effect / trace
```

`CompositionTraceEvent` contains exactly these fieldless variants in this order:

```text
ToolCallProposalPersisted
CallPrepared
EffectIntentPersisted
ExecutorAttempted
ExecutorDispositionObserved
EffectReceiptPersisted
ResultReturned
```

`PreparedFakeToolExecution`, `StagedFakeToolGateway`, `IdempotentFakePluginExecutor`, `InMemoryFakeRunJournal` and `EffectCompositionHarness` use manual bounded/redacted `Debug`; they never print canonical arguments, execution identities, routes, pending effects, receipts or executor dispositions. `CompositionTraceEvent` may derive `Debug, Clone, Copy, PartialEq, Eq` and nothing else.

### 11.2 Required order

The success path is exactly:

```text
provider raw call binds through frozen AgentToolsetView::bind_call into AgentToolCall
→ M30 ToolCallProposal::from(bound AgentToolCall) persisted
→ staged fake M40 correlates the already-bound AgentToolCall
→ M20 InvocationAuthorityService transaction-current recheck succeeds
→ PreparedFakeToolExecution returned; executor count is still zero
→ composition derives one exact M30 EffectIntent
→ fake M30 journal decides, pre-applies on a clone, persists and applies EffectIntent event
→ idempotent fake executor receives one sealed execution request
→ composition derives one exact M30 EffectReceipt
→ fake M30 journal decides, pre-applies on a clone, persists and applies receipt event
→ staged fake M40 returns one correlated AgentToolResult
```

`AgentToolResult` reuses the original provider call ID and the persisted `EffectReceipt` outcome kind/digest exactly. It never re-hashes raw executor output, substitutes a different failure class or returns raw bytes/logs. Any prepared-call, effect-intent, executor-disposition, receipt or result identity/digest mismatch fails closed before result construction.

The fake journal executes a command by calling `decide`, cloning the checkpoint, applying the proposed event to that clone, then attempting persistence; only a successful persistence swaps the already validated clone into the live checkpoint and appends the event. It never performs a second fallible apply after persistence. `AlreadyApplied` appends nothing. Injected intent persistence failure therefore leaves no event/checkpoint mutation and reaches no executor. Receipt persistence uncertainty returns no AgentToolResult.

Receipt reconciliation is a distinct path, not a new call authorization:

1. load the exact M30 pending `EffectIntent` after replay;
2. query the fake executor's disposition ledger by the exact effect and idempotency identities without starting a new effect;
3. if an exact prior outcome exists, derive and persist the matching receipt;
4. if no disposition is known, remain explicitly unresolved and return no result;
5. if a conflicting disposition exists, fail closed and return no result.

Current deny-side drift may block every new execution, but it cannot erase or rewrite an already observed external disposition. Reconciliation therefore never substitutes a second current-authority approval for the prior persisted intent and never calls a non-idempotent effect again. The fake records executor **attempts** separately from unique effect identities and must prove the unique-effect count remains one; no success/commit meaning is inferred from a failure or uncertain disposition.

The M40 fake does not write M30 truth. The M30 fake does not normalize or execute calls. The composition harness is the sole test layer that calls both.

### 11.3 Denial and update semantics

Before proposal or intent persistence, all of these produce no M30 proposal, intent, executor request, receipt or result:

```text
projection/run/turn mismatch
unknown model-visible tool
```

After a call is bound, all of these produce no intent, executor request, receipt or result:

```text
malformed arguments
route/dispatch mismatch
catalog revoke
installation Disabled, Revoked or absent
current grant Stale, Expired, Revoked or absent
emergency policy block
post-update old package/grant carrier mismatch
repository conflict/corruption
```

B7-B combines—not duplicates—the accepted evidence chain:

1. The Disable branch executes the merged A1 façade against `InMemoryInstallationRepository`, obtains the exact owner command receipt projection, and updates the current resolver fixture only through a test helper that preserves the prior immutable installation/package/component identity while copying the receipt's post-state/post-revision. A mismatched identity or revision is rejected. B3 owner mapping evidence pins that bridge.
2. B6 owner tests prove Apply/Rollback mutate only future installation/grant carriers and stale prior active grants.
3. B5 mapping/service tests prove owner snapshots map to resolver snapshots and every call rechecks transaction-current deny-side authority.
4. B7-B composition fixtures instantiate the exact documented Revoke/Update pre/post carrier classes, freeze the old `AgentToolsetView`, mutate only the current semantic authority repository and prove:
   - the in-flight old view and its schema/route digest remain byte/typed-equal;
   - the old call is denied before intent/executor I/O;
   - a new projection is denied while installation/grants are not freshly authoritative;
   - after an exact fresh enable/grant semantic carrier is supplied, a new projection binds the target pin/new grant while the old view remains unchanged.

B7-B does not claim one public external test executes B6 owner-only evidence constructors. The cross-file checker pins the exact owner-state-to-resolver-state bridge and prevents the composition fixture from silently weakening the B6 postcondition.

### 11.4 Exact B7-B test names

```text
authorized_call_persists_intent_before_executor_and_receipt_before_result
pre_intent_denials_persist_nothing_and_reach_no_executor
intent_persistence_failure_reaches_no_executor
executor_failure_is_receipted_before_failed_result
receipt_uncertainty_returns_no_result_and_retry_deduplicates_effect
disable_and_revoke_preserve_frozen_view_but_deny_old_calls_and_new_projection
package_update_preserves_in_flight_view_and_requires_fresh_projection_authority
```

The existing `tool_gateway_conformance.rs` reuses the staged support rather than retaining a second monolithic fake implementation.

## 12. Exact future writable paths

Packet acceptance alone writes none of these paths. A later finite implementation grant may authorize only the following.

### 12.1 B7-A1 path allowlist

```text
crates/platform-core/src/market/application.rs                 # new
crates/platform-core/src/market.rs
crates/platform-core/tests/market_application.rs               # new
crates/platform-core/tests/platform_identity.rs                 # module/public-surface mirror only
scripts/check_repo_contracts.py
scripts/tests/test_check_repo_contracts.py
docs/plan/modules/30-market-package-lifecycle.md
docs/contracts/market-lifecycle.md
docs/contracts/interfaces.md
docs/contracts/module-boundaries.md
docs/coverage-matrix.md
docs/acceptance/matrix.tsv
docs/acceptance/platform-baseline.md
docs/tasks/01-execution-roadmap.md
docs/tasks/m20-b7-contract-readiness-proposal.md
```

No Cargo manifest/dependency, adapter, daemon, M30, M40, M51, M80 or M90 path is writable in A1.

### 12.2 B7-B path allowlist

```text
apps/ustc-agentd/tests/support/mod.rs
apps/ustc-agentd/tests/support/staged_tool_gateway.rs            # new
apps/ustc-agentd/tests/support/fake_run_journal.rs                # new
apps/ustc-agentd/tests/tool_gateway_conformance.rs
apps/ustc-agentd/tests/tool_effect_composition.rs                 # new
scripts/check_repo_contracts.py
scripts/tests/test_check_repo_contracts.py
docs/plan/modules/30-market-package-lifecycle.md
docs/plan/modules/50-tool-gateway-execution.md
docs/contracts/market-lifecycle.md
docs/contracts/agent-plugin-boundary.md
docs/contracts/module-boundaries.md
docs/coverage-matrix.md
docs/acceptance/matrix.tsv
docs/acceptance/platform-baseline.md
docs/tasks/01-execution-roadmap.md
docs/tasks/m20-b7-contract-readiness-proposal.md
```

No production `src/`, Cargo manifest/dependency, M10, M30, M40 production module, M51, M80 or M90 path is writable in B7-B.

Any additional path requires an explicit scope amendment from Develata before mutation.

## 13. Exact validation and checker contract

### 13.1 A1 focused tests

The new `market_application.rs` integration target contains exactly these test functions and imports only production-public APIs:

```text
anonymous_catalog_paging_is_revision_bound_bounded_and_exact
package_detail_is_exact_without_latest_or_fallback
owned_reads_hide_foreign_objects_and_exclude_sensitive_carriers
current_grants_require_exact_installation_revision_and_canonical_order
disable_preserves_owner_ledger_first_idempotency_and_maps_one_event
application_facade_exposes_no_transport_or_authority_issuer_surface
```

Every rejection asserts one exact typed variant; `is_err()`, `is_err() || ...`, wildcard success/failure escapes and source-text-only substitutes are forbidden. The integration target receives no `cfg(test)`, feature-gated or debug-only public constructor for owner evidence, events, response views or repository internals.

Required A1 gates:

```bash
cargo test --locked -p ustc-campus-agent-core --test market_application
cargo test --locked -p ustc-campus-agent-core
cargo test --locked -p ustc-campus-agent-core --doc
cargo test --locked --workspace --all-targets
cargo test --locked --workspace --doc
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo fmt --all -- --check
python3 scripts/check_repo_contracts.py --ci
python3 -m unittest scripts.tests.test_check_repo_contracts
```

### 13.2 B7-B focused tests

Required B7-B gates:

```bash
cargo test --locked -p ustc-agentd --test tool_gateway_conformance
cargo test --locked -p ustc-agentd --test tool_effect_composition
cargo test --locked --workspace --all-targets
cargo test --locked --workspace --doc
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo fmt --all -- --check
python3 scripts/check_repo_contracts.py --ci
python3 -m unittest scripts.tests.test_check_repo_contracts
```

### 13.3 Fail-closed checker inventory

The repository checker must pin:

- exact A1 source/module/public-item/service-method/test-function inventories plus each active `#[test]` attribute envelope; `ignore`, `cfg`, `cfg_attr`, zero-test and renamed-target forms fail closed;
- all new field visibility, derives, manual `Debug`, constructor/accessor and forbidden `Default`/Serde/raw/mutable conversion rules;
- exact safe-view field-source mappings and explicit exclusions for raw source-policy values, configuration entries, secret refs, execution identities, approval/evidence carriers, private routes and event histories;
- exact application error variants and exhaustive mapping sites;
- absence of M10/framework/database/network/executor/M30/M40/M51 imports from A1;
- zero A1 production call sites and exact text forbidding tenant/user material from untrusted request fields;
- exact B7-B support files/types/test names, each active `#[test]` attribute envelope and zero production-source occurrence; `ignore`, `cfg`, `cfg_attr`, zero-test and renamed-target forms fail closed;
- load-bearing A1/B7-B proof calls and assertions at the bound test body's top-level depth, with ignored `Result`, early `return`/`?`, never-entered conditionals and row-skipping control flow rejected;
- exactly one fresh-path `execute_or_reconcile` call site, exactly one reconciliation-only `lookup_disposition` call site, one intent-persistence site, one receipt-persistence site and their required stage order;
- every pre-intent denial fixture's zero-intent/zero-executor/zero-receipt/zero-result assertions;
- exact provider-call/effect/idempotency/outcome-digest correlation from prepared call through persisted receipt and normalized result;
- exact B6 postcondition classes used by the update composition fixture;
- acceptance binding text and prohibited premature status promotions.

Mandatory mutation bites include at least:

```text
page limit 0 or 101 accepted
catalog fake accepts 0 or 65 revisions
one A1 test loses `#[test]`, gains `#[ignore]` or is hidden by `cfg`/`cfg_attr`
one A1 load-bearing call is wrapped in `if false`, skipped by early control flow or returned as an ignored `Result`
continuation revision ignored
latest/same-name fallback introduced
foreign owner mapped differently from missing
owner command-ledger replay moved after current-revision rejection, or owner expected-revision decision removed
configuration entry or SecretRef exposed
execution identity, approval/evidence or private route added to a view
raw source-policy map added to a view
Serialize, Deserialize, Default, derived Debug or unchecked constructor added
Install/Enable/Grant/Update mutation method added to A1
M20 imports transport/database/M30/M40/M51
A1 application operation called from production before admitted M00→M10 mapping exists
one B7-B test loses `#[test]`, gains `#[ignore]` or is hidden by `cfg`/`cfg_attr`
one B7-B load-bearing call/assertion is wrapped in a never-entered branch or skipped while its token remains
executor call moved before intent persistence
intent-persistence failure reaches executor
receipt failure returns a result
result call ID, outcome kind or digest differs from the persisted receipt
pre-intent denial appends intent/receipt or reaches executor
retry executes the same effect twice
frozen view mutates after current authority drift
B6 after-update fixture keeps old active grant or enabled old pin
MARKET-007 or PKG-020 status promoted without the complete exact binding
```

Every mutation must make the checker/test suite fail for the intended reason; mere source-token counting is insufficient where semantic fixture assertions can be exercised.

## 14. Acceptance-status policy

Contract acceptance changes no case status.

After a final A1 implementation candidate, all of `MARKET-001`–`MARKET-004`, `MARKET-007`, `PKG-019`, `PKG-020` and `FP-007` remain `planned`; A1 is supporting application-contract evidence only.

After a final B7-B candidate, only these two rows become eligible for independent status review:

- `MARKET-007`, if every pre-intent denial, M30 intent failure, executor call and receipt/result order is exact;
- `PKG-020`, if frozen-view/new-projection/current-call/update carrier semantics and the B6 bridge are complete.

Eligibility is not automatic promotion. A status edit is permitted only in the B7-B final candidate when exact tests, checker mutation suite and independent reviewers all conclude that the complete row assertion—not merely supporting evidence—is satisfied. `MARKET-001`–`MARKET-004`, `PKG-019` and `FP-007` remain planned after B7-B.

## 15. Explicit non-goals and non-claims

A1 and B7-B do not provide or claim:

- installation listing, catalog search/filter/ranking or latest-version selection;
- Install, Configure, Enable, Revoke, Uninstall, grant or update mutation application APIs;
- automatic initial/default grants or first-party bootstrap;
- M00 session/actor admission, M10 wire DTOs/routes or M80 UI/client behavior;
- full source/license metadata required by later Web detail acceptance;
- production persistence, transaction isolation, migration, restart, backup or restore;
- production M40/M51 executor code, network I/O, MCP conformance or hosted runtime;
- real effect-journal durability or process-crash recovery;
- `MARKET-001`–`MARKET-004`, `PKG-019` or `FP-007` completion;
- release readiness.

## 16. Sequencing, review and stop rules

The only allowed progression is:

```text
whole-packet selection and review
→ Develata accepts exact delimited packet
→ separately authorized docs/contract amendment and source-control shipping
→ exact contract merged and exact-main CI passes
→ pre-edit representability/taskbook review binds the accepted bytes to live public/test construction surfaces
→ separately authorized B7-A1 retained implementation
→ final A1 gates/reviews/source-control boundary
→ separately authorized B7-B retained implementation
→ final B7-B gates/reviews/status decision/source-control boundary
```

Proposal reviews use three independent blocker lanes:

1. M20 application API, ownership, safe-view and error semantics;
2. M20→M30→M40 ordering, idempotency, failure and in-flight/update semantics;
3. dependency/path/public-surface/checker/acceptance-status honesty and implementability.

A blocker repairs this proposal and restarts all three exact-digest reviews. Advisory hardening that requires deliberately weakening current product behavior/tests/checkers but exposes no current failure is recorded as later assurance work and does not create an infinite mutation round.

Stop immediately on:

- source commit/tree drift;
- unresolved owner/dependency conflict;
- a required public type or path outside the allowlists;
- an inability to implement A1 using existing owner repository traits without a second authority;
- an inability to compose B7-B without production M20↔M30/M40 dependency inversion;
- checker/test design that cannot bite an ordering or sensitive-data mutation;
- any claim that test support is production durability;
- any unapproved commit, push, PR, merge, tag or release boundary.

## 17. Whole-packet acceptance semantics

The packet bytes between the exact markers are one indivisible proposal. Its SHA-256 is computed over the raw UTF-8 bytes beginning with `## 7.` immediately after the `BEGIN` marker newline and ending with the newline immediately before the `END` marker; markers themselves are excluded and no newline/Unicode normalization is applied. Selection of P1/A1 is not whole-packet acceptance. Whole-packet acceptance means Develata explicitly accepts those final SHA-256-bound bytes after all three proposal reviews are PASS and the source binding remains exact.

Even whole-packet acceptance authorizes only contract amendment drafting. It does not by itself authorize commit/push/PR/merge or production/test implementation.
<!-- M20_B7_EXACT_PACKET:END -->
