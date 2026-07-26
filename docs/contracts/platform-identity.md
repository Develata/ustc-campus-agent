# Platform identity value contract

## Metadata

- `Status`: Accepted `M00-B1` target contract; implementation and evidence planned
- `Version`: `platform-identity/v0`
- `Last Review`: `2026-07-26`
- `Owning Blueprint`: [`M00 Platform Control and Identity`](../plan/modules/10-platform-control-identity.md)
- `Authority Defers To`: [`../plan/03-platform-authority.md`](../plan/03-platform-authority.md) for authority partition and [`module-boundaries.md`](module-boundaries.md) for cross-module ownership
- `Acceptance`: active planned `AUTH-011`, `AUTH-012`, `AUTH-014`, `AUTH-015`, `AUTH-016`; catalog-only `AUTH-013` is deferred to `M00-B3 request-context`
- `Primary Code`: future `crates/platform-core/src/identity.rs`; existing `invocation::{TenantId, UserId}` must migrate to or reuse these canonical values rather than remain parallel identities

## 1. Scope and authority

`platform-identity/v0` freezes the small, framework-free values needed before `M00` can construct sessions or admitted request contexts. It owns:

- six canonical bounded platform ID representations;
- nominal separation between semantically different ID kinds;
- one shared construction-error taxonomy;
- validation, conversion, serialization and diagnostic behavior for those values.

It does not authenticate a subject, compose a tenant-scoped actor, open a session, assign a role, authorize a domain operation, generate an ID, persist a record or infer authority from text. Those decisions remain with later `M00` batches, `M10`, an owning domain module or an adapter as named by their contracts.

The module mints no value. It imports no clock, random-number generator, transport, database, framework or authentication-adapter type. These values are domain primitives, not transport DTOs, database rows, framework handles, credentials or user-facing labels.

## 2. Canonical value set

`M00-B1` introduces exactly these public value kinds:

| Value | Meaning | Explicitly does not prove |
|---|---|---|
| `TenantId` | one platform tenant | organization metadata, membership or permission |
| `UserId` | one platform-managed user subject, meaningful only with a tenant | external username, CAS/OIDC subject, authentication or role |
| `SessionId` | one platform session identity | active, authenticated, unexpired or unrevoked session state |
| `RequestId` | one ingress-attempt identity | admission, authorization or command acceptance |
| `CommandId` | one platform command identity | persistence, idempotent success or domain authorization |
| `CorrelationId` | one audit/operation correlation-chain identity | idempotency, authorization or causal adjacency |

`CausationId` and any tenant-scoped actor key are owned by the later `request-context` small module. `PlatformPolicySnapshotId` is owned by the later `policy-reference` small module. `platform-identity/v0` does not define them.

External `(issuer, subject)` evidence is owned by the later authentication-adapter/session contract and carries its own separately bounded type. A platform `UserId` is never a verbatim provider subject, and this bound is not widened to accommodate one.

## 3. Shared identifier grammar

Each of the six ID newtypes wraps one canonical string whose UTF-8 bytes satisfy:

```regex
^[A-Za-z0-9](?:[-A-Za-z0-9._:]{0,126}[A-Za-z0-9])?$
```

Normative consequences:

1. encoded length is `1..=128` bytes;
2. the first and last byte are ASCII alphanumeric;
3. interior bytes are ASCII alphanumeric or one of `.`, `_`, `:`, `-`;
4. whitespace, control characters, non-ASCII text and every other punctuation byte are rejected;
5. case is significant;
6. no trimming, Unicode normalization, case folding, delimiter rewriting or alternate spelling occurs;
7. repeated interior delimiters are legal and retain no semantic meaning;
8. a prefix or delimiter pattern conveys no tenant class, role, provider, authorization, identifier kind or lifecycle state.

The grammar permits opaque and prefixed generator output whose first and last symbols are ASCII alphanumeric; hexadecimal, base32/Crockford, ULID and UUID qualify. Output from alphabets such as base64url or default Nano ID that can place `-` or `_` at an endpoint must be re-encoded before use. Retrying generation until a value happens to conform is not an accepted mitigation. Generation and collision policy are later adapter/port concerns; every generated value must still pass the same constructor.

## 4. Public construction and representation

Each ID kind is an owned nominal Rust newtype with a private backing field. It provides an inherent checked `parse(value: impl Into<String>) -> Result<Self, IdentityValueError>` as the single canonical validator. The following public paths all delegate to `parse` and therefore share one grammar and error precedence:

- `TryFrom<String>`;
- `TryFrom<&str>`;
- `FromStr`;
- Serde deserialization from one string.

The inherent `parse` preserves existing invocation fixture call sites while tenant/user definitions converge. It is the checked constructor, not an unchecked compatibility path.

Each kind provides read-only access through `as_str()` and exact `Display`. Serialization emits exactly the canonical string. `Clone`, `Eq`, `Ord` and `Hash` operate on exact bytes. `Debug` retains the nominal type name; the value type does not silently redact or rewrite its bytes.

The public API must not provide:

- `Default`;
- a public unchecked constructor;
- a lossy or infallible conversion from arbitrary text;
- cross-kind `From` conversions;
- `Deref` or mutable access to the backing string;
- segment, prefix or delimiter interpretation APIs;
- framework, database, auth-provider or transport-specific traits in the domain module.

Serde is an admitted stable value-encoding foundation and is exempt from the framework prohibition. A caller may explicitly read `as_str()` and construct another kind through its validator, but that act creates no authority and must not be used as a convenience conversion inside platform code.

## 5. Deterministic validation errors

All six ID constructors return one shared `IdentityValueError` taxonomy:

```text
Empty
TooLong { max_bytes: 128 }
InvalidStart
InvalidCharacter { byte_index }
InvalidEnd
```

Error precedence is deterministic:

1. empty input;
2. byte length greater than 128;
3. invalid first byte;
4. first invalid byte in the half-open interior range `bytes[1..len-1]`, scanned left to right;
5. invalid final byte at `bytes[len-1]`.

For a one-byte input, the first-byte rule is the complete character check. For length at least two, the interior range excludes both endpoints. If all earlier checks pass, any non-alphanumeric final byte returns `InvalidEnd`, whether that byte would be a legal interior delimiter or an otherwise forbidden byte. A multibyte non-ASCII suffix may therefore return `InvalidCharacter` for its first invalid byte inside the interior range before the final-byte rule is reached.

`byte_index` is a zero-based byte index into the rejected input. An error may report the value kind, failure variant, fixed bound and byte index; it must not retain, format or log the rejected input. `Display` and `Debug` must not contain the complete rejected input, quote any input-derived fragment or render the offending byte.

Every failed construction returns no partial value. Serde uses `parse` and the same error precedence; it cannot bypass the constructor with a derived unchecked field decode.

## 6. Existing-type convergence

`crates/platform-core/src/invocation.rs` currently defines local `TenantId`, `UserId` and invocation-specific `PolicySnapshotId` values for the bounded P0a invocation proof. `M00-B1` must converge only the first two:

1. canonical `TenantId` and `UserId` move to the platform identity module;
2. invocation code imports and compatibility-re-exports those exact tenant/user types so existing `invocation::TenantId` and `invocation::UserId` paths remain valid without a second wrapper;
3. invocation `PolicySnapshotId` remains M20-owned and unrenamed; it identifies an `InvocationPolicySnapshot` and MUST NOT alias any future platform-policy identity;
4. existing invocation-specific IDs such as `InstallationId`, `GrantSnapshotId`, `RunId` and `TurnId` remain outside this contract until their owning contracts migrate them;
5. fixture data that violates `platform-identity/v0` fails migration explicitly; implementation must not preserve it through an unchecked compatibility path.

This convergence is part of the future implementation evidence. `M00-B1` is incomplete if duplicate tenant or user identity definitions remain publicly usable inside `platform-core`. Converging M20 policy identity is explicitly forbidden by this contract.

## 7. Failure and security boundaries

- Invalid text is rejected before any session, request, command or persistence operation exists.
- IDs are opaque references, not secrets; credential/token/password material must never be placed in them.
- Because rejected input may itself be secret material, validation errors omit the raw input.
- A syntactically valid ID never proves that the referenced object exists or is in scope.
- Deserialization success is shape validation only; every authority-bearing operation performs its owning lookup and state checks.
- No same-text or same-prefix fallback converts one ID kind into another.

## 8. Acceptance projection

| Case | Required proof | Planned binding |
|---|---|---|
| `AUTH-011` | each of the six ID kinds enforces the exact bounded grammar, deterministic error precedence and validating Serde path through `parse` | `cargo test --locked -p ustc-campus-agent-core --test platform_identity identity_values_enforce_canonical_bounds_and_errors -- --exact` |
| `AUTH-012` | the six ID kinds are byte-exact in string, Serde, ordering and hashing behavior; compile-fail API checks reject private-field construction, `Default`, unchecked construction, mutable backing access, cross-kind conversion and identifier-shape parsing | `cargo test --locked -p ustc-campus-agent-core --test platform_identity identity_values_are_exact_and_nominal -- --exact && cargo test --locked -p ustc-campus-agent-core --doc identity` |
| `AUTH-014` | construction errors expose only value kind, failure class, fixed bound and byte index; `Display` and `Debug` contain no complete rejected input, input-derived fragment or offending-byte rendering | `cargo test --locked -p ustc-campus-agent-core --test platform_identity identity_errors_never_echo_rejected_input -- --exact` |
| `AUTH-015` | the identity module mints no identifier and declares no clock, random, transport, database, framework or authentication-adapter dependency | `python3 scripts/check_repo_contracts.py && cargo test --locked -p ustc-campus-agent-core --test platform_identity identity_module_has_no_generation_or_adapter_surface -- --exact` |
| `AUTH-016` | Market invocation authority consumes the M00-owned tenant/user definitions with no duplicate public tenant/user identity, while invocation policy-snapshot identity remains M20-owned | `python3 scripts/check_repo_contracts.py && cargo test --locked -p ustc-campus-agent-core --test platform_identity market_invocation_authority_uses_m00_identity_definitions -- --exact` |

All five active rows are `planned`, hence non-pass, until the exact checker rules and Rust tests exist and pass. `AUTH-013` remains catalog-only for the future request-context batch; retaining its stable ID does not make it current evidence. The repository checker validates contract registration and acceptance linkage, but it is not implementation evidence for these rows until the named implementation-specific rules exist.

## 9. Change rule

Changing the accepted byte grammar, maximum length, error precedence, Serde shape or nominal kind set changes `platform-identity/v0`. Such a change requires:

1. an owning-contract update;
2. acceptance-row and fixture review;
3. migration impact review for persisted or externally transported values;
4. implementation and downstream consumer evidence on the same revision.

Adding causation, actor, policy-reference or authenticated/service/administrator semantics is a later owning contract, not an incidental extension of these text values.
