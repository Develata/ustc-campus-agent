# Platform control-evidence contract (`platform-control-evidence/v0`)

- **Status:** implemented bounded redacted projection and journal-port kernel
- **Owner:** M00 Platform Control and Identity
- **Acceptance:** `AUTH-022`
- **Source:** `crates/platform-core/src/control_evidence.rs`
- **Evidence:** `crates/platform-core/tests/platform_control_evidence.rs` plus rustdoc compile-fail proofs

## 1. Purpose and authority boundary

This contract owns stable, redacted, data-only external projections for M00 session transitions, admitted request contexts, and bounded failures. It also owns least-authority read/append-once evidence-journal ports and deterministic semantic fakes.

It does not authenticate, admit, authorize, publish, or persist in production. A decoded event/error, a constructed key, or a successful fake append is never authentication, repository currentness, product mutation authority, production durability, or atomic coupling with Affairs publication. B5 owns the later application composition.

The module accepts no raw credential, cookie, token, authorization header, provider payload, secret reference, credential digest, path, `serde_json::Error`, `io::Error`, or arbitrary diagnostic string and adds no dependency.

## 2. Frozen public surface

`control_evidence.rs` exposes exactly:

1. `PlatformControlActor`
2. `PlatformControlEventKind`
3. `ControlEvidenceKey`
4. `PlatformControlEvent`
5. `PlatformControlErrorCode`
6. `PlatformControlError`
7. `ControlEvidenceJournalError`
8. `ControlEvidenceAppendOutcome`
9. `ControlEvidenceReadPort`
10. `ControlEvidenceAppendPort`

No public type alias, constant, macro, free function, submodule or re-export is admitted.

## 3. Actor, event kind and key

`PlatformControlActor` is the closed `Public | Authenticated { tenant_id, user_id, session_id }` data-only sum with internally tagged `snake_case` Serde and unknown-field denial. It carries no role, grant or arbitrary metadata.

`PlatformControlEventKind` has exactly, in order:

```text
SessionOpened
SessionRefreshed
SessionExpired
SessionRevoked
RequestAdmitted
```

`ControlEvidenceKey` is exactly:

```text
Session { session_id, sequence }
Request { command_id }
```

A key is dedupe identity only and grants no authority.

## 4. Stable event schema

`PlatformControlEvent` is an internally tagged, `snake_case`, unknown-field-denying enum with exact variants:

```text
SessionOpened {
  session_id, sequence, tenant_id, user_id, auth_adapter_id, opened_at
}
SessionRefreshed {
  session_id, sequence, refreshed_at, effective_expires_at
}
SessionExpired {
  session_id, sequence, expired_at, observed_at, cause
}
SessionRevoked {
  session_id, sequence, revoked_at
}
RequestAdmitted {
  request_id, command_id, correlation_id, causation_id, actor,
  operation_id, descriptor_snapshot_id, permission_class, effect_class,
  policy_snapshot_id, observed_at
}
```

Exact methods:

```text
from_session_event(&SessionEvent) -> PlatformControlEvent
from_admitted_request(&PlatformRequestContext) -> PlatformControlEvent
kind() -> PlatformControlEventKind
key() -> ControlEvidenceKey
occurred_at() -> SessionInstant
```

Projection is exact:

- `SessionOpened` copies IDs, sequence and open time from the event, while tenant/user/adapter come only through `SessionOpened::credential_evidence()` read accessors; the digest and credential deadline are absent;
- `SessionRefreshed.refreshed_at := observed_at`; its persisted effective expiry is copied and the non-persisted idle candidate is absent;
- expiry copies historical `expired_at`, later `observed_at`, and cause;
- `SessionRevoked.revoked_at := observed_at`;
- request actor is public or the exact admitted tenant/user/session identities;
- request projection copies only request/command/correlation/causation, admitted operation/descriptor/permission/effect, policy and observed time; payload digest, idempotency key, client provenance and operation snapshot are absent;
- `occurred_at` is opened, refreshed, expiry observation, revoke observation, or request observation respectively;
- Serde round trip is data-shape evidence only and restores no authority.

The enum is deliberately public data. Public construction or pre-append pattern mutation is allowed and non-authoritative. Journal adapters clone before commit.

## 5. Stable error projection

`PlatformControlErrorCode` has exactly 36 `snake_case` Serde variants, in order:

```text
LifecycleCredentialEvidenceExpired
LifecycleInvalidTimeOrder
LifecycleDeadlineOverflow
LifecycleSessionNotFound
LifecycleSessionAlreadyExists
LifecycleSessionIdMismatch
LifecycleRevisionMismatch
LifecycleRevisionOverflow
LifecycleTerminalSession
LifecycleNonMonotoneTime
LifecycleSessionNotYetExpired
LifecycleNoEffectiveRefresh
LifecycleEventSequenceMismatch
LifecycleEventTimeOutsideValidity
LifecycleIllegalEventForState
LifecycleEventDerivedFieldMismatch
AdmissionIdempotencyStoreUnavailable
AdmissionConflictingEnvelope
AdmissionDescriptorSnapshotAbsent
AdmissionDescriptorSnapshotMismatch
AdmissionPolicyDenied
AdmissionPolicyExpired
AdmissionSessionNotFound
AdmissionSessionIdMismatch
AdmissionSessionNotAdmitted
AdmissionCapabilityMissing
AdmissionCapabilityDisabled
AdmissionCapabilityRevoked
AdmissionInfrastructurePortUnavailable
AdmissionMalformedCommand
RepositoryUnavailable
RepositoryCorrupt
RepositoryInvalidEvent
RepositoryLimitExceeded
RepositoryInternalInvariant
MalformedExternalInput
```

`PlatformControlError` contains only one private `code`. Exact methods are:

```text
from_session_domain(&SessionDomainError)
from_admission_rejection(&RequestContextRejection)
from_session_repository(SessionRepositoryError)
malformed_external_input()
code()
```

Mappings are exhaustive one-to-one by source variant/class. Payload-bearing failures discard revisions, statuses, IDs, instants, permission, port, idempotency key, operation ID, field name and underlying text. `malformed_external_input()` accepts no diagnostic parameter. The type implements neither `Display` nor `Error` and contains no source/string/metadata field.

## 6. Journal ports and precedence

```text
ControlEvidenceJournalError =
  Unavailable | Corrupt | LimitExceeded | InternalInvariant

ControlEvidenceAppendOutcome = Appended | AlreadySame | Conflict

ControlEvidenceReadPort::load_control_event(key)
ControlEvidenceAppendPort::append_once(event)
```

`ControlEvidenceAppendPort` extends the read port. Public signatures contain no path, JSON, SQL, HTTP, runtime, framework or arbitrary text type.

Append precedence:

1. unavailable/corrupt before lookup;
2. same key + equal retained event → `AlreadySame`, no mutation;
3. same key + different event → `Conflict`, no mutation;
4. absent key at configured limit → `LimitExceeded`, no mutation;
5. otherwise clone-prepare-commit one complete event → `Appended`;
6. internal insertion invariant failure → `InternalInvariant`, no partial commit.

No result claims product/evidence transaction atomicity.

## 7. Executable evidence

The external test target contains exactly six tests:

1. `session_events_project_one_to_one_without_credential_evidence`
2. `request_admission_event_retains_only_stable_control_fields`
3. `control_errors_map_every_domain_admission_repository_and_boundary_class`
4. `control_event_serde_is_closed_redacted_and_data_only`
5. `control_evidence_fake_is_idempotent_conflict_safe_and_atomic`
6. `control_evidence_ports_distinguish_absent_unavailable_corrupt_and_limit`

Session events are produced through checked `decide -> evolve`; request contexts through `RequestAdmissionCoordinator` and a conforming fake. All 16 lifecycle, 14 admission, 5 repository and 1 boundary mapping classes are exercised. Public error enum variants unreachable from an honest lifecycle prefix may be instantiated directly only to test the projection match.

A bounded `BTreeMap` fake proves read/append precedence, exact retry, conflict, limit, unavailability, corruption, internal commit failure and value-equal no-partial-commit behavior. Same-key/different-event conflict uses two independently valid data-only events, not private authority mutation.

Rustdoc compile-fail proofs cover no evidence-to-authority conversion, no `Default`, no raw diagnostic constructor, and no external struct-literal construction of `PlatformControlError` through its private field.

## 8. Dependency and negative space

Allowed dependencies are existing `serde` plus M00 identity, session, request-context and session-port carriers. `identity.rs`, `session.rs`, `request_context.rs` and `session_port.rs` remain semantically frozen.

Forbidden additions include clock/repository implementations, filesystem/network/database/framework/runtime types, app/product concepts, raw secret/evidence material and conversions into authority carriers.

## 9. Acceptance binding

`AUTH-022` binds exactly:

```text
python3 scripts/check_repo_contracts.py && cargo test --locked -p ustc-campus-agent-core --test platform_control_evidence && cargo test --locked -p ustc-campus-agent-core --doc control_evidence
```

This proves only bounded stable redacted projections, mapping completeness, data-only Serde, journal ports and deterministic fakes. Production persistence, B5, SSO and Affairs publication remain unclaimed.

## 10. Implementation record

B4a `platform-session-port/v0` and B4b `platform-control-evidence/v0` complete the bounded typed interface/fake scope of M00-B4. M00 remains `partial-evidence`: production evidence persistence, B5 administrator admission/composition and formal authentication remain planned.
