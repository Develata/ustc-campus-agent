# Platform session-port contract

## Metadata

- `Status`: Implemented bounded M00-B4a session-port kernel plus one durable DemoReviewed current-session read/bootstrap vendor
- `Version`: `platform-session-port/v0`
- `Last Review`: `2026-08-29`
- `Owning Blueprint`: [`M00 Platform Control and Identity`](../plan/modules/10-platform-control-identity.md)
- `Depends On`: implemented [`platform-session/v0`](platform-session.md), [`platform-request-context/v0`](platform-request-context.md) and [`platform-identity/v0`](platform-identity.md)
- `Acceptance`: `AUTH-021`, `implemented`
- `Primary Code`: `crates/platform-core/src/session_port.rs`, `apps/ustc-agentd/src/m00_session.rs`

## 1. Scope and authority

`platform-session-port/v0` freezes the least-authority interfaces around the pure session lifecycle kernel and one bounded durable current-session read/bootstrap vendor. It owns:

- replay-derived `SessionHistory` construction from complete `SessionEvent` sequences;
- read and compare-and-append port shapes, deterministic clock and credential-evidence port shapes;
- optimistic append precedence for future adapters;
- a logical, non-path `SecretRef`;
- a private durable DemoReviewed session-history file used by the current three-plugin composition;
- fail-closed file/currentness behavior and exact test-fake semantics.

It does **not** authenticate a credential, expose raw secret bytes, implement USTC SSO, add public open/refresh/revoke transport, persist lifecycle mutations, make fixture policy/descriptor/capability/clock observations production authority, itself emit B4b control evidence, complete B5/M10 administration or implement Affairs PROC-011 administrator publication. The separate implemented [`platform-control-evidence/v0`](platform-control-evidence.md) consumes only public typed carriers and remains data-only.

## 2. Exact public Rust surface

The module is exactly `crates/platform-core/src/session_port.rs`, declared as `pub mod session_port;` immediately after `pub mod session;`.

### 2.1 Closed values

```rust
pub struct SecretRef(/* private canonical string */);

pub enum SessionRepositoryError {
    Unavailable,
    Corrupt,
    InvalidEvent,
    LimitExceeded,
    InternalInvariant,
}

pub enum SessionAppendOutcome {
    Appended(SessionHistory),
    AlreadySame(SessionHistory),
    Conflict { current_revision: Option<u64> },
}

pub enum SessionClockError { Unavailable }

pub enum CredentialEvidencePortError {
    Unavailable,
    UnknownSecretRef,
    InternalInvariant,
}

pub struct SessionHistory {
    events: Vec<SessionEvent>,
    snapshot: SessionSnapshot,
}
```

`SecretRef` accepts exactly `^secret-ref:[a-z0-9][a-z0-9._-]{0,95}$`, preserves bytes without normalization, serializes as one transparent string, uses validating deserialization and exposes only checked `parse` plus `as_str`. Its `Debug` is exactly `SecretRef(<redacted>)`; it has no `Display`, path/bytes conversion, unchecked constructor, `Default` or mutable accessor.

`SessionHistory` is `Clone + PartialEq + Eq` only. It has no `Debug`, `Display`, `Serialize`, `Deserialize` or `Default`. `try_from_events` rejects empty/invalid histories and folds the production `session::evolve` kernel from `None`; no snapshot decode, mutation or unchecked construction exists. Read-only accessors are exactly `events`, `snapshot`, `session_id`, `revision`.

No error carries or renders raw source text, filesystem path, serialized event, secret reference, credential evidence, rejected bytes or adapter diagnostics.

### 2.2 Exact traits

```rust
pub trait SessionHistoryReadPort {
    fn load_history(
        &mut self,
        session_id: &SessionId,
    ) -> Result<Option<SessionHistory>, SessionRepositoryError>;
}

pub trait SessionHistoryAppendPort: SessionHistoryReadPort {
    fn compare_and_append(
        &mut self,
        session_id: &SessionId,
        expected_revision: Option<u64>,
        event: &SessionEvent,
    ) -> Result<SessionAppendOutcome, SessionRepositoryError>;
}

pub trait SessionClockPort {
    fn now(&mut self) -> Result<SessionInstant, SessionClockError>;
}

pub trait CredentialEvidencePort {
    fn fingerprint_adapter_evidence(
        &mut self,
        auth_adapter_id: &AuthAdapterId,
        secret_ref: &SecretRef,
    ) -> Result<CredentialEvidenceDigest, CredentialEvidencePortError>;
}
```

`CredentialEvidencePort` fingerprints material already verified by a trusted adapter. It is not a verifier and mints no authenticated or authorization authority.

## 3. Compare-and-append precedence

A semantic repository implementation follows this total order:

1. load and replay-validate retained history; invalid retained state is `Corrupt`;
2. an exact retry at the current final sequence and exact predecessor fence is `AlreadySame`; same-sequence drift below `u64::MAX` is `Conflict`;
3. at retained revision `u64::MAX`, every non-exact retry is `LimitExceeded` before conflict/invalid-event branches;
4. a historical candidate sequence is `Conflict`;
5. absent history with non-`None` fence is `Conflict`; under `None`, only same-argument-session `Opened` sequence 1 may proceed;
6. retained history requires exact current revision fence;
7. candidate session ID equals the method argument and sequence equals checked `current + 1`;
8. replay the complete candidate; candidate replay failure is `InvalidEvent`;
9. commit atomically or expose no candidate state.

The B4a deterministic fake uses exact `max_events = 4`. Its fifth correct-fence append is `LimitExceeded` with no mutation. Exact runtime `u64::MAX` exhaustion is intentionally not claimed as executed: honest replay-only construction cannot represent such a prefix without an impossible history, and B4a adds no arbitrary snapshot test seam. A future accepted durable append vendor must own that private representable fixture.

## 4. Durable DemoReviewed current-session file

The app-private vendor implements only `SessionHistoryReadPort`, never `SessionHistoryAppendPort`.

Canonical JSON is exact compact `serde_json::to_vec` bytes with `deny_unknown_fields`:

```text
schema_version = 1
sessions = ordered array of 1..=64 records
record.session_id
record.events = 1..=1024 SessionEvent values
```

Records are strictly ascending by `SessionId`; IDs are unique; every record ID equals every replayed event session ID; snapshots are always replay-derived. The file is nonempty and at most 16 MiB. Existing bytes must parse, validate and reserialize byte-identically; whitespace, trailing newline, alternate order, wrong version, unknown field, duplicate, empty history, forged event, cross-session event, replay failure or limit violation fails closed.

### 4.1 Filesystem invariant

- exact caller-selected file and parent;
- parent exists, is a real non-symlink directory, current-user-owned and mode `0700`;
- primary is regular, non-symlink, one-link, current-user-owned, mode `0600`, `1..=16 MiB`;
- `open(2)` uses `O_NOFOLLOW`; path metadata before/after and descriptor dev/inode agree;
- first bootstrap writes a unique same-parent create-new/no-follow `0600` temp, writes canonical bytes and `sync_all`s it;
- publication uses same-filesystem `hard_link(temp, destination)` so no destination is overwritten, then removes temp and `fsync`s parent;
- an `AlreadyExists` publication removes only this caller's temp and returns `Unavailable`; a later call validates the retained destination;
- pre-publication failures attempt to remove the run-owned temp; successful cleanup leaves no residue, while cleanup-removal failure returns `Unavailable`, may leave exactly that run-owned temp and never publishes a destination;
- after publication, successful temp unlink followed by parent-sync failure returns `Unavailable`, leaves one complete destination and permits the next open to validate it; temp-unlink failure instead leaves two links to the complete inode and subsequent opens fail closed on the one-link invariant until the run-owned temp is removed by an operator;
- no chmod repair, symlink following, truncate-in-place, partial-tail recovery, silent reset, quarantine-as-success or fixture fallback.

On non-Unix, the app-private file returns `Unavailable` before path access; its six filesystem tests are Unix-gated. No broader portability claim is made for `ustc-agentd`.

## 5. Current composition

`AffairsComposition::open*` requires an explicit session-store path, opens/bootstraps from the reviewed fixture event, then loads the fixture's configured `SessionId` from retained history before constructing services. The retained snapshot supplies current tenant/user scope to `OpportunityAuthorityStore`; `FixturePorts::load_session` always queries the durable store by the requested session ID.

- missing configured session: `session_store_current_session_absent`;
- same-ID fixture tenant/user drift: `session_store_current_session_scope_mismatch`;
- typed repository errors map exhaustively to static `session_store_*` strings with no lower-level echo.

Scope mismatch or changed session ID leaves the retained file byte-identical. Since M10 authenticated input carries only a `SessionId`, B4a makes no false claim that tenant/user mismatch can be denied later from caller-asserted fields.

All handlers in one composition share clone handles over one immutable `Arc` map. Independently opened compositions have independently selected stores. Existing authenticated Opportunity calls use the retained session; public Affairs/Change behavior remains unchanged. An authenticated request whose session ID is absent from the retained store now surfaces `SessionNotFound` rather than the retired fixture-clone `SessionIdMismatch`. Other admission facts remain honestly fixture-backed.

## 6. Evidence and status

`AUTH-021` is `implemented`, `gate=pr`, `owner=backend`, bound exactly to:

```text
python3 scripts/check_repo_contracts.py && cargo test --locked -p ustc-campus-agent-core --test platform_session_port && cargo test --locked -p ustc-campus-agent-core --doc session_port && cargo test --locked -p ustc-agentd --lib m00_session::tests && cargo test --locked -p ustc-agentd --test opportunity_composition
```

The core target has exactly six contract tests; `m00_session.rs` has exactly six Unix-gated unit tests; `opportunity_composition.rs` binds the exact two B4a tests `concurrent_retained_session_reads_are_peer_isolated` and `retained_session_restart_scope_and_changed_bootstrap_fail_closed`. Runtime filesystem tests, checker mutations, full workspace test/Clippy/doctest and exact-source review remain mandatory gates.

M00 remains `partial-evidence`. B4a session-port/read vendor and B4b [`platform-control-evidence/v0`](platform-control-evidence.md) are bounded implemented, completing the typed interface/fake scope of B4. Durable lifecycle/evidence persistence, B5 and PROC-011 remain planned.
