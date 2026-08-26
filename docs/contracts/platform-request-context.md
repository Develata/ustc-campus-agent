# Platform Request Context Contract

- Contract ID: `platform-request-context/v0`
- Owner: `M00 / crates/platform-core/src/request_context.rs`
- Acceptance: `AUTH-013`
- Status: **implemented bounded platform-core kernel**

## 1. Authority boundary

`RequestAdmissionCoordinator::admit` is the sole constructor of admitted request authority. Callers and adapters may supply only:

1. checked command data in `BuildRequestContextCommand`;
2. transaction-current observations through `AdmissionPorts`;
3. durable `Persisted*Dto` values whose `Deserialize` implementations revalidate structure and cross-field coherence.

Callers cannot construct `PlatformRequestContext`, `AdmittedOperation`, `M00AdmittedDisposition`, `FrozenPrerequisites`, `RequestContextRejection`, `FinalAdmissionDisposition`, or `EnvelopeHash`. These authority-bearing carriers expose read-only accessors and implement neither `Deserialize` nor public unchecked constructors.

The kernel consumes the existing six platform identity kinds. `CausationId` remains a request-context-owned checked leaf and does not widen `platform-identity/v0`.

## 2. Command and checked leaves

`BuildRequestContextCommand` carries exactly:

- `request_id: RequestId`;
- `operation_id: OperationId`;
- `actor_reference: ActorReference`;
- required `correlation_id: CorrelationId`;
- optional request-context-owned `causation_id: CausationId`;
- optional `idempotency_key: IdempotencyKey`;
- `client_provenance: ClientProvenance`;
- `payload_digest: PayloadDigest`.

Checked leaf rules:

- request-context-owned text leaves (`OperationId`, `CausationId`, `IdempotencyKey`, `PlatformPolicySnapshotId`, `SchemaIdentity`, `DecoderIdentity`, `DispatcherIdentity`, `AdapterIdentity`) are 1–128 ASCII bytes from `[A-Za-z0-9._:/-]` with no unchecked construction path; platform-identity-owned `TenantId`, `UserId`, `SessionId`, `RequestId`, `CorrelationId`, and `CommandId` retain `platform-identity/v0` grammar: boundary-alphanumeric, interior `-._:`, and no `/`;
- `SchemaDigest` and `PayloadDigest` are exactly 64 lowercase hexadecimal bytes;
- `DescriptorSnapshotId` is exactly `descriptor:v0:<nonzero-u64>:<SchemaDigest>`; the coordinator path produces it through `DescriptorSnapshotId::from_canonical_identity`, while persistence decoding revalidates existing copies through `parse` and confers no mint authority;
- `AdapterAllowlist` contains 1–64 unique checked adapter identities in canonical sorted order;
- `IdempotencyReservationToken::from_store_observation` rejects zero fencing tokens.

## 3. Closed actor and permission algebra

`ActorReference` is closed:

```text
Anonymous { PublicScope }
Authenticated { SessionId }
```

The admitted actor is closed:

```text
M00AdmittedActor::Public
M00AdmittedActor::Authenticated(AdmittedIdentities { TenantId, UserId, SessionId })
```

Public admission never invents tenant/user/session identifiers. Authenticated admission binds the exact requested session identifier to the loaded transaction-current snapshot and to the admitted actor.

`PermissionClass` is closed:

```text
PublicRead
PublicLinkout
TenantPrivateRead
TenantPrivateWrite
PrivilegedExternalEffect
```

Anonymous admission is allowed only for `PublicRead` or `PublicLinkout`; it is denied before session loading on every private/effectful class. Authenticated admission still checks session authority even for a public operation.

## 4. Operation snapshot

`AdmissionPorts::request_scoped_operation` returns one `Arc<dyn OperationDescriptorProjection>`. The coordinator:

1. requires that snapshot to be present;
2. requires exact command/snapshot `OperationId` equality;
3. copies the admitted descriptor facts into `AdmittedOperation`;
4. preserves the same `Arc` in `PlatformRequestContext` for downstream use.

The coordinator performs no live registry lookup. `DescriptorSnapshotId` accessors expose the checked content digest and nonzero snapshot version needed by M10 durable projection.

## 5. Deterministic admission order

The coordinator executes the following fail-closed order:

1. compute the M00-owned envelope hash;
2. reserve/retrieve idempotency state;
3. return identical prior or in-flight state before any descriptor/policy/session/capability lookup;
4. acquire the request-scoped descriptor snapshot and require exact operation identity;
5. obtain one observation time;
6. reject public/private permission incoherence before session lookup;
7. resolve a current policy snapshot;
8. for authenticated actors, load and verify the exact session and require `SessionSnapshot::admits_at(observed_at)`;
9. check current capability state;
10. construct the sealed context and complete scalar disposition;
11. finalize under the reservation's fencing token.

Any unavailable port maps to a typed fail-closed rejection. A stale finalizer cannot commit after reclamation.

## 6. Idempotency protocol

Reservation results are closed:

```text
New(IdempotencyReservationToken)
PriorIdentical(PersistedPriorDispositionDto)
InFlight(IdempotencyReservationToken)
Reclaimed(IdempotencyReservationToken)
```

Finalization results are closed:

```text
Committed
AlreadySame(PersistedPriorDispositionDto)
LostReservation(IdempotencyReservationToken)
```

`FinalAdmissionDisposition` is passed to the store by reference; the adapter obtains durable data only through `to_persisted_projection`. The envelope hash is exactly the domain/version separator `platform-request-context/v0/envelope\0`, followed by length-delimited operation ID, the closed actor reference (including the authenticated session ID), the payload digest, and optional causation ID. Request ID, correlation ID, idempotency key, and client provenance are intentionally excluded: retries may receive new ingress/correlation/provenance observations, while the key remains the store lookup parameter rather than a second hash input. With no client idempotency key, the store still reserves a controller-generated command ID and therefore provides at-most-once behavior for that attempt.

## 7. Complete scalar disposition

`M00AdmittedDisposition` contains exactly:

- `command_id`;
- required `correlation_id`;
- `descriptor_snapshot_id`;
- closed `admitted_actor`;
- `FrozenPrerequisites { policy_snapshot_id, observed_at, session_id, admitted_operation_id }`.

Coherence rules:

- Public actor iff frozen session is `None`;
- Authenticated actor iff frozen session is `Some` and exactly equals the actor session;
- admitted operation is the operation that passed descriptor, policy, and capability admission.

`PersistedAdmittedDispositionDto::Deserialize` rechecks these cross-field rules. Promotion into live authority is private to M00.

## 8. Closed result and rejection algebra

The only cross-boundary result is:

```text
Admitted { context, disposition }
PriorAdmitted(M00AdmittedDisposition)
Rejected(RequestContextRejection)
PriorRejected(RequestContextRejection)
Incomplete(M00IncompleteReservation)
```

`M00IncompleteReservation` carries the command ID and retry-not-before deadline. It is neither admitted nor rejected.

`AdmissionRejectionClass` has exactly fourteen unit variants:

1. `IdempotencyStoreUnavailable`
2. `ConflictingEnvelope`
3. `DescriptorSnapshotAbsent`
4. `DescriptorSnapshotMismatch`
5. `PolicyDenied`
6. `PolicyExpired`
7. `SessionNotFound`
8. `SessionIdMismatch`
9. `SessionNotAdmitted`
10. `CapabilityMissing`
11. `CapabilityDisabled`
12. `CapabilityRevoked`
13. `InfrastructurePortUnavailable`
14. `MalformedCommand`

`AdmissionRejectionProjection` preserves the required payload of every class. `RequestContextRejection` has private fields; its public surface is `class()` plus `projection()`. Diagnostics are crate-private and do not create a second public error branch.

## 9. Persistence boundary

Persisted leaves implement validating `Serialize`/`Deserialize`. Aggregate authority projections use private unchecked mirrors and manual validating `Deserialize`:

- `PersistedFrozenPrerequisitesDto`;
- `PersistedAdmittedDispositionDto`;
- `PersistedAdmissionRejectionDto`;
- `PersistedPriorDispositionDto`.

Unknown fields, unknown enum variants, malformed checked leaves, public/session incoherence, authenticated session mismatch, and zero fencing tokens fail closed. Durable adapters persist projection DTOs, never the live context, actor, rejection, disposition, finalization carrier, or descriptor `Arc`.

## 10. Evidence

Required local commands:

```bash
cargo test --locked -p ustc-campus-agent-core --test platform_request_context
cargo test --locked -p ustc-campus-agent-core --doc request_context
python3 scripts/check_repo_contracts.py
python3 -m unittest scripts.tests.test_check_repo_contracts
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
```

The integration target contains exactly 64 named request-context tests: 15 retained M00 cases plus 49 expansion cases. Checker mutation tests prove that the contract missing/empty/unregistered cases, source module removal, one required Rust-test removal, acceptance-row regression, and unrelated owning-carrier mutation each produce a request-context-specific diagnostic.
