# M70 contract-readiness decision ledger

- `Layer`: Task / contract-readiness review evidence
- `Status`: InReview
- `Version`: `m70-contract-readiness/v0`
- `Last Review`: 2026-07-29
- `Module`: M70 USTC ChangeRadar
- `Current implementation state`: design-only
- `Authority Owns`: explicit M70 readiness questions, required evidence, review dispositions and exit conditions
- `Authority Defers To`: root `AGENTS.md`, constitution, terminology, M60/M70 blueprints, owning plans/contracts, acceptance matrix, work policy and execution roadmap
- `Historical artifact`: SHA-256 `d3ae1b2315bfc09ee4d92730b5c290c64bdc406a37de0480105446bffd6a9277` at source `fd9a4ff36cc36ce672e8fc90908190db9ba42880`
- `Current task base`: `c347e689aa23ee777b95e0989e633a9d91041161`

This ledger is review/task evidence, not a second architecture or contract authority. While its status is `InReview`, every decision below remains open, M70 remains `design-only`, and no retained implementation, acceptance promotion, or integration is authorized.

## 1. Purpose and non-goals

Purpose:

- make every M70 activation question explicit and decidable before retained implementation;
- the ledger summarizes reviewed evidence and sends accepted outcomes back to owning plan/contract/acceptance documents;
- raw packet/transcripts/machine paths remain outside repository authority.

Non-goals:

- no Rust placement decision;
- no production field/API/error freezing;
- no stable ID algorithm selection;
- no storage/feed technology selection;
- no real source approval;
- no M00/M20/M60/M10/M80 integration activation;
- no package/component/acceptance/status promotion;
- no implementation recipe or code.

## 2. Bound evidence and status truth

Historical artifact:

```text
Historical artifact source: fd9a4ff36cc36ce672e8fc90908190db9ba42880
Historical artifact tree: 007201218fefb33cba8981db0a62545fc88bb462
Historical artifact SHA-256: d3ae1b2315bfc09ee4d92730b5c290c64bdc406a37de0480105446bffd6a9277
Historical artifact verdict: R1_ARTIFACT_GO_IMPLEMENTATION_NO_GO
Current ledger task base: c347e689aa23ee777b95e0989e633a9d91041161
Current ledger task tree: 09e06b18b6a235a46eb023b97e3e3354bc2ef969
```

Artifact checks prove only synthetic contract-carrier consistency and fail-closed validator behavior. They do not prove production behavior or authorize projection of the draft as a current contract.

Current repository truth:

```text
M70 state key: design-only
M70 implementation state: design-only
FP-002: planned
RADAR-001: planned
RADAR-002: planned
retained Rust implementation: not authorized
```

## 3. Frozen invariants

These invariants are already owned by current plans/contracts and are therefore **not open to redefinition in this ledger**:

1. `M60` alone owns source identity, accepted revision/baseline, provenance, freshness and conflict truth.
2. `M70` consumes accepted `M60` outputs and owns semantic change candidate/review/event/feed behavior only.
3. A maintainer/model/Agent may propose a candidate but cannot approve, reject, publish or create canonical truth.
4. Publication requires explicit admitted administrator authority; there is no automatic publication path.
5. Duplicate/replay cannot create a second durable candidate, event or feed item.
6. Missing/incompatible/conflicting/revoked evidence fails closed and creates no publication.
7. Approved event truth and feed projection are distinct; renderer/delivery failure cannot erase or reinterpret the approved event.
8. `M80` remains a thin display/intent shell; it cannot approve/publish or own ChangeRadar state.
9. Framework/provider/session state is never ChangeRadar authority.
10. Private personalized feeds and arbitrary website monitoring remain outside MVP.

## 4. Decision ledger

| Decision ID | Question | Governing owner | Required evidence | Exit condition | Disposition | Resolution |
|---|---|---|---|---|---|---|
| `OPEN-01-RUST-PLACEMENT` | Exact retained Rust module/crate placement and compiler-enforced dependency direction. | M70 blueprint + module work policy; Develata approval after architecture review. | modules-before-crates analysis, proposed file/Cargo graph, allowed/forbidden dependency check, rollback/removal path. | Owning blueprint/contract records one approved placement and independent review finds no authority/type leak. | `Pending` | `open` |
| `OPEN-02-RUST-PUBLIC-API` | Exact fields, constructors, transitions and stable error classes for `BoardPolicy`, accepted `M60` input view, `ChangeCandidate`, `ReviewDecision`, `ChangeEvent` and feed/query outputs. | M70 specific contract, deferring `M60`/`M00`-owned values to their contracts. | Exact API/value table, success/failure fixtures, unknown-version behavior, no external/framework types. | Accepted specific contract is complete enough for one small-module slice and all ambiguous authority is removed. | `Pending` | `open` |
| `OPEN-03-STABLE-ID-ALGORITHM` | Canonical bytes, domain separators and deterministic IDs/GUIDs for candidate/event/feed identity and duplicate suppression. | M70 contract, using shared crypto/canonicalization foundations only. | Normative input fields/order/encoding/domain separators, golden vectors, collision/domain-separation and replay tests, compatibility policy. | Exact algorithm and vectors are accepted in owning contract without using renderer order or database-generated identity. | `Pending` | `open` |
| `OPEN-04-M60-BOUNDARY` | Exact `B-M60-M70-CHANGE` accepted revision/fact/provenance/freshness/conflict input and rejection mapping. | `M60`/`M70` boundary contract; `M60` retains source/baseline authority. | Exact fake `M60` input DTO/port, malformed/stale/conflict/revoked fixtures, proof that parser/fetch/storage internals do not cross. | Both module contracts and boundary registry agree on one typed, fakeable, acyclic boundary. | `Pending` | `open` |
| `OPEN-05-STORAGE-TRANSACTION` | Atomic candidate/evidence/review/event/publication writes, expected revision, duplicate winner and rollback/retry semantics. | M70 domain transition contract; `M90` may implement declared ports but cannot own transitions. | Port operation table, transaction boundaries, failure-injection matrix, duplicate/conflict/restart behavior, untouched-state assertions. | Every partial failure has one deterministic recovery and cannot acknowledge/publish incomplete state. | `Pending` | `open` |
| `OPEN-06-FEED-RENDERER` | Exact RSS/Atom MVP choice, deterministic payload fields/order/time/escaping, stable GUID and renderer failure contract. | M70 feed contract; concrete XML/server framework remains adapter-internal. | Exact output contract, golden fixture, escaping/size/order tests, event-vs-projection recovery, compatibility/rollback policy. | One renderer contract is accepted and cannot mutate approved event truth. | `Pending` | `open` |
| `OPEN-07-REAL-SOURCE` | First concrete approved public source, owner, permission, exact URL/path, retrieval/rate/body policy and parser fixture. | `M60` source review; `M70` cannot approve a source. | Reviewed `SourceDefinition` proposal, permission/legal note, exact non-secret fixture, parser identity and failure cases. | `M60` records one approved source or explicitly authorizes an exact offline fixture for the next bounded `M70` slice. | `Pending` | `open` |
| `OPEN-08-ACTOR-IDENTITY` | Exact admitted administrator/reviewer/proposer identity and request/causation evidence used by approve/reject/publish commands. | `M00` identity/request-context contract + `M70` command contract. | Typed actor/context mapping, authorization denial fixtures, replay/idempotency binding, no raw auth/session/framework type. | `M00`/`M70` contracts agree and a fake admitted/denied actor path is available. | `Pending` | `open` |
| `OPEN-09-M20-INTEGRATION` | Package version/install/enable/grant/invocation relationship without moving `M70` state into Market or bypassing current denial. | `M20` lifecycle/invocation contract + composition. | Exact package/component projection, disable/revoke negative path, no direct `M70`→`M20` private dependency, composition fixture. | Current `M20` public contract supports one exact `M70` attachment and denial reaches no `M70` mutation. | `Pending` | `open` |
| `OPEN-10-M10-M80-INTEGRATION` | Exact ChangeRadar query/review command/result/event projection through admitted `M10` and thin `M80`. | `M10` application ingress boundary + `M70` public application contract + `M80` client contract. | Versioned DTO/error/event subset, admission/precondition mapping, thin-client negative-space test, fake backend/client journey. | One accepted ingress/projection contract exists and no client/server-function adapter reaches repositories or owns domain transitions. | `Pending` | `open` |

Every row uses `Pending` and `open`. No `Basis`, `Required evidence` or `Exit condition` wording selects or recommends an architecture answer.

## 5. Review lanes

| Lane ID | Scope | Outcome | Blocking conditions |
|---|---|---|---|
| `architecture` | placement, public API, dependency direction, IDs and replaceability | `Pending` | all relevant `OPEN-*` decisions unresolved |
| `policy-authority` | `M60`/`M00`/`M20` ownership, transaction/publication authority, real-source and denial semantics | `Pending` | all relevant `OPEN-*` decisions unresolved |
| `delivery-evidence` | feasible slices, exact fixtures, status truth, rollback and integration gates | `Pending` | all relevant `OPEN-*` decisions unresolved |

- Missing lane or silence is not acceptance.
- Each lane returns explicit `Accept | ConditionalAccept | Reject` for all relevant decision IDs.
- `ConditionalAccept` requires owner, evidence and a decidable exit condition.
- Accepted outcomes must be projected into the owning plan/contract/acceptance documents before this ledger can close.
- This PR itself keeps all lanes `Pending` and is ready for later strong independent review only.

## 6. Activation gate and next safe slice

This ledger may change from `InReview` to `Complete` only when:

1. all three lanes returned explicit outcomes;
2. every `OPEN-*` row has accepted owning-document resolution and `closed` status;
3. `M70` exact specific contract and `B-M60-M70-CHANGE` are accepted;
4. the next small module has exact active `planned` acceptance rows with non-vacuous future evidence bindings before implementation;
5. module map/blueprint/roadmap/coverage/acceptance status projections agree;
6. retained implementation has a separately authorized branch/task;
7. final checker and independent review pass.

Next safe implementation principle (not activated):

```text
After this ledger closes, choose exactly one smallest M70 small-module slice whose public inputs/outputs are settled and whose tests run against exact fakes. This ledger does not choose or start that slice.
```

## 7. Verification and non-claims

Gates for this docs-only PR:

```bash
python3 scripts/check_repo_contracts.py
python3 -m unittest -v scripts.tests.test_check_repo_contracts
git diff --check
git status --short --branch
git diff --name-status "$EXPECTED_BASE"...HEAD
git diff --stat "$EXPECTED_BASE"...HEAD
```

`cargo fmt`/`clippy`/`test` and browser smoke are `not applicable`: no Rust, manifest, UI or runtime changed.

Status truths:

```text
M70_IMPLEMENTATION_ACTIVATED=no
M70_STATE_CHANGED=no
ACCEPTANCE_STATUS_CHANGED=no
RUST_CODE_WRITTEN=no
REAL_SOURCE_APPROVED=no
INTEGRATION_STARTED=no
NEXT_ACTION=strong review of the ten decisions; no implementation
```
