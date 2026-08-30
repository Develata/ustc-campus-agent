# M70-B6-DP — ChangeRadar durable administrator publication

## Task authority

- `Stage`: `IMPLEMENTED_AWAITING_FINAL_REVIEW`
- `Owning module`: `M70 ChangeRadar`
- `Batch identity`: bounded durable-publication sub-slice of existing `M70-B6 M60/M10/M80 integration`; this file does **not** create `M70-B7` and does not claim all of B6 complete
- `Path`: Path A coupled large-module integration
- `Source commit`: `e6fcf2423a2b438708b2573f4616161cc95aa08e`
- `Source tree`: `348b85475d0ca5b7c4ea68a4942ce57a4ac09887`
- `Source PR`: `#55`
- `Branch`: `work/m70-durable-publication-v1`
- `Review generation`: `POSTEDIT-V34_REPAIR_CANDIDATE`
- `Repair round`: `12`
- `Current blocker`: `none in source`; v33 Codex's M00-session post-publication temporary-unlink uncertainty blocker is repaired with exact reconciliation and an injected hardlink-residue regression, pending exact-current gates and dual re-review.
- `Stop reason`: `none`
- `Acceptance`: implemented `AUTH-024` plus implemented `RADAR-001`; `RADAR-002` remains planned and out of scope
- `Composition owners`: M00 Platform Control/Identity, M10 Application Ingress Host and `apps/ustc-agentd`

This task authority schedules one concrete integration slice under already accepted M00/M10/M70 boundaries. It does not redefine product topology, permission/effect semantics, M70 lifecycle, M60 source authority or the public-read ToolGateway contract. It grants no remote operation by itself.

## 1. Goal / ISA

When the fixed `DemoReviewed` administrator reviews and publishes the existing exact ChangeRadar semantic candidate, the command crosses M10 and current M00 admission, persists or verifies one redacted admitted-request event before any M70 review/publication effect, commits the reviewed event to durable product state, and remains exactly readable as JSON and Atom after a real daemon restart. Ordinary users continue to read through Market → bounded Harness → ToolGateway → the owning M70 query service.

Non-goals: production SSO, live campus retrieval, arbitrary website monitoring, generic CMS/search/operation registry, generic publication framework, subscriptions, personalized private feeds, maintainer Agent leases, M80 peer-client convergence, deployment, release or public visibility.

## 2. Source-bound current truth

At the bound source:

- `crates/change-radar/src/publication.rs` owns digest-bound review receipts, coherent transaction-current M60 verification, service-minted review/publication commits, `ChangePublicationRepository`, exactly-once publication and stable feed GUIDs.
- `crates/change-radar/src/query.rs` reads only `ChangePublicationRepository` and propagates typed repository failure instead of converting it to an empty feed.
- `apps/ustc-agentd/src/change_fixture.rs` validates the bounded DemoReviewed baseline/candidate fixture, then opens `DurableChangeRadarRepository`; restart recovery replays only checked stored review/publication state and never reruns M60 or remints a decision.
- `apps/ustc-agentd/src/change_invocation.rs` keeps ordinary `change.list` reads behind current M10, Market projection/recheck, bounded Harness, ToolGateway and the owning M70 query service.
- `crates/application-ingress/src/affairs_publication.rs`, `apps/ustc-agentd/src/affairs_publication.rs`, the durable M00 evidence journal and Affairs persistence adapter are ordering/security precedents, not reusable product state machines.
- M00 current-session authority and durable control evidence already exist and MUST be reused rather than copied.
- The authoritative roadmap names only M70-B1 through M70-B6. This bounded slice remains subordinate to B6; the real M80 peer-client attachment stays deferred, so this work cannot claim B6 complete.

## 3. Frozen call topology

Administrator write:

```text
ChangePublicationCommand (`change.publish`)
→ recompute canonical v1 payload digest
→ validate the exact staged descriptor
→ current M00 session / permission / capability / policy admission
→ append-once or exact-read-back PlatformControlEvent::RequestAdmitted
→ direct owning M70 application port
→ construct and record the exact approved review
→ coherent M60 publication verification
→ durable ChangePublicationRepository commit
→ typed receipt / thin HTTP, operator CLI and Web projections
```

Ordinary read remains:

```text
Web/M10 `change.list`
→ current Market authorization
→ bounded Harness
→ ToolGateway
→ fixed public-read ChangeRadar adapter
→ ChangeFeedQueryService over the durable repository
→ JSON/Atom projection
```

The administrator write MUST NOT pass through the current in-process ToolGateway adapter because that adapter is fixed to `PublicRead`. It MUST NOT increment ChangeRadar query-spine intent/execution/receipt counters. The write uses the existing `B-M10-APP-CALL`/`B-APP-M10-RESULT` boundary family and does not create a universal operation/result bag.

## 4. Fixed command descriptor

- operation ID: `change.publish`
- permission: `TenantPrivateWrite`
- effect: `TenantLocalMutation`
- actor: existing fixed authenticated `DemoReviewed` administrator session only
- target: the one exact fixture board/event candidate
- payload: request/correlation/idempotency/provenance identity, candidate/event ID, expected publication identity or explicit unpublished precondition, review timestamp and publication timestamp required by the existing M70 contract
- payload digest domain: `change-publication-payload/v1\0`

No raw source bytes, normalized facts, credentials, private profiles, arbitrary adapter errors or free-form rejection text may enter M00 control evidence.

## 5. Domain and recovery contract

1. Preserve `ChangePublicationRepository` as the M70 review/publication authority boundary.
2. Add exactly one checked, storage-neutral M70 recovery entry point plus the narrow read-only accessors/carriers it needs. The entry point restores only an already-persisted publication record from a validated candidate, review, feed policy, verified-evidence identity (the M60 evidence-set digest) and publication timestamp, reruns all existing candidate/review/feed/receipt/GUID validations, and does **not** call the M60 port during restart recovery. It MUST NOT construct a new review/publication decision, MUST NOT be callable by the normal publish path, and MUST reject any carrier that does not exactly match one persisted adapter record.
3. The app-private adapter persists the complete M70 inputs needed for deterministic reconstruction at `apply`/fresh bootstrap time: exact accepted old/new observations including normalized facts, the exact M70 board policy including `tracked_fields`, candidate, review, feed policy, verified-evidence digest and publication. These are M70 recovery records and validated references; they do not make M70 the owner of M60 source revisions, accepted baselines or transaction-current source health. M60 remains authoritative during a new publication decision.
4. Recovery MUST reject mismatch in board/feed policy, source identity/URL, revision IDs and raw/normalized digests, normalized facts, event ID, affected scope, review receipt/reviewer/time/decision, M60 evidence-set digest, publication receipt/GUID and publication time.
5. Only verified absence is absence. Repository corruption/unavailability remains typed infrastructure failure and never becomes an empty feed or `NotFound`.
6. The M70 domain crate MUST NOT import serde, filesystem paths, HTTP, M00, daemon or Web types.
7. Review and publication remain separate durable transitions. A failed publication after durable review may retry the same command; it cannot synthesize a new review identity.

## 6. Durable adapter contract

Create one app-private adapter implementing the existing M70 repository ports:

- canonical bounded private JSON DTO with explicit schema version;
- owner-only real parent/file, regular-file checks and rejection of symlink/hardlink/FIFO/directory attacks;
- bounded bytes and record counts;
- atomic temp write, file fsync, rename and parent sync;
- preflight candidate clone/rebuild before persistence; visible in-memory state changes only after a pre-commit persistence success;
- a post-rename parent-sync failure is a typed uncertain commit and MUST reconcile memory with the canonical renamed file rather than roll back to stale state;
- strict canonical reopen and exact deterministic reconstruction through the checked M70 recovery entry point, without M60 I/O;
- the explicit complete demo state set is Affairs records, Affairs idempotency, current M00 session history, M00 control evidence, Affairs publication, Opportunity profile/tombstone state exactly when the startup composition constructs Opportunity with its configured durable repository, and ChangeRadar recovery/publication; when Opportunity is not constructed its member is excluded, and any non-fresh set with a missing required member fails closed;
- fixture may bootstrap the exact M70 board policy and accepted observations only when that complete state set is fresh;
- after durable state exists, fixture bytes are validation input, never restart authority;
- fixture drift, missing or duplicate identity, conflicting receipt, noncanonical ordering, corruption or runtime file replacement fails closed;
- bounded failure injection for review and publication persistence.

Do not combine Affairs and ChangeRadar records into one generic product repository. Shared low-level secure-file helpers may be extracted only after both concrete adapters pass unchanged product-specific tests and the extraction carries no product lifecycle or authority semantics.

## 7. Application and composition contract

- Add an M10-owned typed `ChangePublicationCommand`, canonical digest and coordinator parallel to—but not genericized with—the Affairs coordinator.
- Reuse `M10AdmissionPorts`, `RequestAdmissionCoordinator`, current M00 session ports and the existing durable control-evidence journal.
- Evidence append/read-back succeeds before the M70 application port is called.
- A prior admitted retry requires exact coherent evidence identity; missing/conflicting/corrupt evidence denies before M60 or repository mutation.
- The M70 adapter constructs the existing domain review receipt, records it, then publishes through `ChangePublicationService`.
- Expose loopback demo-only status/publish endpoints, `ustc-agentctl change publication-status/publish-demo --confirm`, and one thin administrator panel with explicit non-production labeling.
- Existing public JSON/Atom routes read the same durable repository.
- Startup adds one explicit ChangeRadar state path derived under the current owner-only state directory; it MUST NOT repurpose the Affairs publication path or Opportunity profile path.

## 8. Must-pass adversarial proofs

1. Wrong identity, tenant, user, session, permission, disabled/revoked capability, descriptor drift, malformed digest or missing explicit confirmation reaches no evidence success, M60, review or publication mutation.
2. Evidence append failure reaches no M60/repository mutation and creates no success receipt.
3. Evidence success followed by review/publication persistence failure leaves no visible publication; exact retry after restart reuses the same evidence event and publishes once.
4. Same request/correlation/idempotency command identity with byte-identical canonical payload returns the exact receipt without duplicate evidence/feed item; reusing any of those command identities with a changed canonical payload fails closed before M70 mutation.
5. Review reject/mismatch, publish-before-review, board/feed-policy mismatch, M60 unavailable/corrupt/unverified/stale/conflicting, repository conflict/capacity and persistence failure publish nothing.
6. A killed first daemon followed by a second process restores the exact event, review receipt, publication receipt/GUID, feed item, Atom bytes and evidence count through the recovery-only entry point while an injected failing/counting M60 stub proves zero M60 calls; the normal publication path cannot call that entry point, and malformed/unpersisted recovery carriers mint no review or publication decision.
7. Corrupt/noncanonical/oversized/missing/duplicate/conflicting M70 identity state or gapped/reordered M00 evidence-journal sequence, wrong mode, symlink/hardlink/FIFO/directory and runtime replacement all fail closed.
8. Durable read corruption maps to typed infrastructure error, never empty feed or `NotFound`.
9. Existing Affairs and Opportunity paths remain isolated; ChangeRadar administrator publication does not increment ordinary query-spine counters.
10. Browser evidence proves unpublished → explicit publish → exact JSON/Atom item → real process restart → same item/GUID/evidence count with no visual regression.

## 9. Documentation and acceptance projections

Before retained implementation:

- planned `AUTH-024` binds fixed M00-admitted ChangeRadar publication, evidence-before-effect, the direct owning-M70 application port, zero `PublicRead` ToolGateway/query-counter use and Affairs/Opportunity isolation;
- planned `RADAR-001` binds durable exact-once JSON/Atom restart evidence, explicit zero-M60-I/O recovery through a recovery-only entry point that cannot mint a new decision or serve the normal publish path, and post-rename parent-sync uncertain-commit reconciliation with the canonical renamed file and no stale in-memory rollback;
- `RADAR-002` remains planned because maintainer lease/candidate-only Agent behavior is not implemented;
- no module advances beyond `partial-evidence`.

After real evidence, update only the exact current-truth projections proven by this slice: M00/M10/M70 blueprints/module map, first-party feature, roadmap, overview, README, coverage matrix, acceptance matrix/baseline, CLI contract/development guide and repository checker/tests where the checker owns exact current truth. Keep `partial-evidence`; do not claim M70 `StandaloneReady`, an approved live source, production administration or M80 peer clients.

## 10. Writable scope

Expected implementation paths:

- `crates/change-radar/src/{lib.rs,publication.rs,query.rs}`
- `crates/change-radar/tests/{publication.rs,no_bypass.rs}` and narrow recovery tests
- `crates/application-ingress/src/{lib.rs,persistence.rs,change_publication.rs}`
- `crates/application-ingress/tests/{common/mod.rs,change_publication.rs}`
- `apps/ustc-agentd/src/{lib.rs,main.rs,affairs_fixture.rs,m00_control_evidence.rs,m00_session.rs,change_fixture.rs,change_invocation.rs,change_persistence.rs,change_publication.rs,opportunity_persistence.rs,web.rs,web/app.js,web/index.html}`
- `apps/ustc-agentd/tests/{change_composition.rs,affairs_web.rs,opportunity_composition.rs}`
- `apps/ustc-agentctl/src/main.rs`
- `scripts/check_repo_contracts.py` and the exact focused checker tests required to protect current task/acceptance projections
- exact M00/M10/M70 plan/feature/contract/acceptance/roadmap/overview/README/guide/plugin/checker projections

Forbidden: workflows, CODEOWNERS, branch protection, campaign-grant blocks, release/deployment/credential/public-visibility changes, real-source bytes, another product's private repository semantics or unrelated cleanup.

Any necessary path outside this finite scope pauses before mutation unless it is an already named current-truth projection mechanically required by the repository checker and carries no new behavior/authority decision.

## 11. Implementation slices

1. `M70 recovery surface`: storage-neutral checked reconstruction/accessors and repository conformance tests.
2. `App-private durable adapter`: secure persistence, strict reopen, fault injection and exact query error propagation.
3. `M10 command`: digest, descriptor/admission/evidence ordering and fake application-port tests.
4. `Daemon composition`: direct M70 application port, startup state-set rules and same-repository reads.
5. `Operator projections`: loopback HTTP, `ustc-agentctl`, thin Web panel and explicit confirmation.
6. `Evidence and current truth`: real restart/browser proof and truthful projection/checker updates.

Shared authority carriers, DTO/error/permission semantics, acceptance matrix, roadmap and composition fan-in remain serialized under the integration owner. At most two disjoint implementation writers plus one read-only auditor may run concurrently.

## 12. Gates

Focused:

```text
cargo fmt --all -- --check
cargo test --locked -p ustc-campus-agent-change-radar --all-features
cargo test --locked -p ustc-campus-agent-application-ingress --test change_publication
cargo test --locked -p ustc-agentd --test change_composition
cargo test --locked -p ustc-agentd --test affairs_web
cargo test --locked -p ustc-agentctl
python3 scripts/check_repo_contracts.py
node --check apps/ustc-agentd/src/web/app.js
git diff --check
```

Final exact candidate:

```text
cargo fmt --all -- --check
cargo test --locked --workspace --all-features
cargo test --locked --workspace --all-features --doc
cargo clippy --locked --workspace --all-targets --all-features -- -D warnings
cargo build --locked --workspace --all-features
python3 -m unittest discover -s scripts/tests -p 'test_*.py'
python3 scripts/check_repo_contracts.py
git diff --check
```

Also required: exact-source manifest/read-back, real CLI/server two-process restart proof, browser publish/restart proof and two independent reviewer profiles with no unresolved blocker before remote delivery.

## 13. Stop conditions

Pause before implementation or the next source/remote mutation if work requires changing product topology, M00 authority ownership, permission/effect semantics, ChangeRadar lifecycle semantics, M60 live-source approval, production authentication, generic framework extraction, out-of-scope paths, another product's authority, or accepting a material availability/security trade-off not resolved here. Mechanical implementation/test repairs inside this contract continue only after source-bound contract review returns no blocker.
