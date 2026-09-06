# Platform account contract

## Metadata

- `Contract ID`: `M00-ACCOUNT-001`
- `Version`: `platform-account/v0`
- `Status`: Approved product direction; construction contract; implementation planned
- `Last Review`: `2026-09-05`
- `Owner`: M00 Platform Control and Identity
- `Owning Blueprint`: [M00](../plan/modules/10-platform-control-identity.md)
- `Depends On`: [identity](platform-identity.md), [session](platform-session.md),
  [session ports](platform-session-port.md), [request context](platform-request-context.md)
- `Integration`: [M10](../plan/modules/20-application-api-host.md),
  [M90](../plan/modules/90-infrastructure-operations.md)
- `Acceptance`: `ACCOUNT-001` through `ACCOUNT-008` below; planned, not current passing gates
- `Task`: [multi-user campus Agent](../tasks/multi-user-campus-agent.md)

## 1. Scope and ownership

M00 owns application accounts, account status, role assignment, credential generation
references and verified external-identity associations. The two supported entry paths
are SSO and administrator-configured users. There is no self-service registration or
invitation registration, and no invitation object or public account-creation endpoint.
Authentication establishes a subject; it does not grant Plugin capabilities or
campus-data access. Administrator configuration provisions application identity;
SSO verifies an external identity and resolves its approved account association.

Campus Agent journeys, Plugin lifecycle and owner-scoped product behavior may develop
against controlled admitted contexts while account adapters are unfinished. This is
an independent development boundary, not permission to expose shared demo state as a
multi-user service. Real private ingress requires verified per-request admission.

M00's application service coordinates credential verification and legal account /
session transitions. M90 adapters own password hashing, cryptographic randomness,
credential storage, transaction durability and clocks behind M00-declared ports.
M10 decodes bounded requests, supplies transport protection and calls one application
operation; M80 renders results. Passwords, database handles and cookie types do not
enter the pure account/session domain. Accounts do not depend on Market or Agent.

This extends M00's owned identity lifecycle. It does not change existing nominal ID
grammars or the session transition algebra. New public Rust types, wire DTOs and
source inventories must be registered before their implementation is accepted.

## 2. Owned state and bounds

| Object | Canonical fields and rules |
|---|---|
| Account | `(TenantId, UserId)`, optional local login name unique within tenant, `Active / Disabled`, `Member / Administrator`, revision, credential generation; SSO-only accounts need no password or local login name |
| Credential binding | account, adapter ID, opaque credential-record reference, generation; password hash material stays inside the credential adapter |
| Session credential binding | internal `SessionId`, account, credential generation, opaque bearer verifier reference; valid only with its canonical M00 session |
| External identity link | reviewed adapter/issuer identity plus verified stable subject, one exact account and revision; no automatic name/email matching |

A tenant is an isolation scope; a user identifies a subject inside it. The first
service may host one configured campus tenant and many users. Neither one tenant
per user nor cross-tenant membership is inferred. Browser/model inputs cannot pick
another tenant, assign roles, or mint platform IDs. All private module calls carry
both IDs from an admitted request context.

Initial local login names are 3–64 ASCII bytes: lowercase alphanumeric endpoints,
with lowercase alphanumeric, `.`, `_`, `-` inside. The service rejects other spelling;
UI may suggest a valid name but must not silently rename an account. Names are
immutable in this slice. Password submissions contain 12–256 Unicode scalar values,
at most 1024 UTF-8 bytes, without trimming, normalization or silent truncation.
Unknown fields, oversized inputs and invalid encodings fail before expensive hashing.
These limits are application policy, not changes to platform `UserId` grammar.

Session bearers use at least 256 bits from an OS-backed cryptographic random source.
They are unrelated to public IDs, usernames and request IDs. Only verifiers are
retained. Local password records carry a versioned algorithm/parameter identifier
and bounded encoding. Creating an SSO-only user must not create a default password.

## 3. Commands and legal transitions

These are transport-neutral application operations; they are not model tools.

| Operation | Preconditions | Durable result |
|---|---|---|
| BootstrapAdministrator | explicit local operator action; account store initialized but empty; bootstrap fence unclaimed | atomically create one configured-tenant administrator and close bootstrap |
| ConfigureAccount | current active same-tenant administrator through an admitted backend management operation, or explicit local operator; unique target/name and valid authentication binding | atomically create an active member with its local credential reference or approved SSO association and record disposition |
| LinkExternalIdentity | current active same-tenant administrator or explicit local operator; exact account/revision and independently verified unique issuer/subject evidence | bind exact external association; advance credential generation on replacement so old sessions cannot retain replaced authority |
| Login | active account; accepted credential verification and current generation | durably open canonical session and bind fresh bearer before returning it |
| CurrentAccount | verified bearer, current active account/generation and admitting session | safe identity, role and session expiry projection only |
| Logout | exact authenticated current session | canonical session revoke and credential-binding invalidation committed before success |
| DisableAccount | same-tenant administrator; expected revision; not the last active administrator | disable account and advance generation, immediately blocking new admission |
| ResetCredential | explicit local operator recovery for an exact account, or future separately admitted recovery flow | replace credential reference and advance generation; all old session bindings fail admission |

No registration or invitation operation is exposed. Backend configuration creates
`Member` accounts; it cannot silently promote a configured user. Bootstrap is a local
operator port, never an HTTP route or "first request becomes admin" behavior. Later
role promotion and account deletion require separately registered operations; this
slice exposes neither silently. Disable/recovery events retain historical actor IDs
and receipts. Administrator user management is not an Agent tool or a public user API.

Concurrent configuration for one target/name commits one account. Name conflicts and
failed credential preparation leave no partial account or usable orphan credential.
Configuration does not log in the new user or return that user's session bearer.
The user authenticates separately through local credentials or verified SSO. Missing
or unknown external mappings deny login; an SSO callback cannot create an account,
choose a role, or fall back to a demo identity.

Administrative commands use bounded request identities and expected revisions where
state exists. The account transaction stores a redacted outcome with its causation.
An exact retry returns the original result; a different payload under the same key
conflicts. Passwords and external identity secrets are never included in receipt text
or a generic payload digest. The adapter binds secret-bearing retries to an opaque
operation proof. A lost configuration response cannot create a second account,
reset its credential or mint a login bearer on retry.

## 4. Ports and transaction boundaries

M00 declares narrow interfaces with checked inputs and closed failures:

- `AccountRepositoryPort`: lookup within tenant; transactional account / external-link /
  credential-reference/disposition compare-and-commit; unique-name and unique-link fences.
- `PasswordCredentialPort`: prepare a new credential record, verify a submission,
  report an opaque verified account/generation result, retire unused prepared records.
- `VerifiedExternalIdentityPort`: validate one browser-bound external login transaction
  under a reviewed adapter, then return opaque issuer/subject evidence; no caller-provided
  subject is accepted as authentication proof. Controlled fakes are not a live SSO adapter.
- `CredentialRandomPort`: mint collision-checked IDs and one-time bearer/verifier pairs;
  never use timestamps, UI randomness or password text as entropy.
- `AccountClockPort`: current time or unavailable; no fixture clock in service mode.
- `AccountSessionTransactionPort`: compose the existing session decision/append rules
  with credential-binding persistence. It may share an M90 database transaction; it
  cannot replace canonical session histories with an authentication framework cache.
- `AdmissionThrottlePort`: atomically reserve bounded attempt capacity and return an
  allowed or retry-after result. Account and network-source buckets are separate.

The password adapter must use a maintained, mature password-hashing implementation,
with a reviewed algorithm and explicit work/memory parameters pinned by its own
implementation contract before any real credential is accepted. The algorithm is
replaceable through versioned records; a generic fast digest is not a password hash.
Unknown algorithms or oversized parameters fail closed. Successful verified login
may rehash under policy using a generation-preserving atomic replacement; corruption
must not fall back to plaintext or accept the credential. Tests use a controlled
credential counterpart until the real adapter is bound and reviewed.

Prepared password records may precede account configuration's transaction, but remain
unusable until its credential reference commits. Aborted/unreferenced records have an owned
bounded cleanup path. Configuration commits all canonical account facts together.
Session opening commits both session and bearer binding before acknowledgement;
failed/uncertain commit returns unavailable, never an in-memory authenticated fallback.
A retry can create another fresh session after reauthentication; the lost bearer is
not reconstructed or logged and its orphaned session expires under normal policy.

## 5. Admission, transport and errors

Every private request resolves the presented bearer through the trusted adapter,
checks account `Active` and current credential generation, then delegates session
validity to `SessionSnapshot::admits_at`. These observations must be coherent with
revocation. No new admission may succeed after disable/logout acknowledgement.
In-flight work retains its historical admission; effectful operations recheck current
authority immediately before the effect under the owning gateway contract.

M10's service profile uses reviewed same-origin HTTPS and an opaque HttpOnly, Secure,
SameSite session cookie with fixed path and no broad Domain. Local development may
explicitly use loopback HTTP with a separate cookie profile and state directory;
that exception cannot select a remote listener. State-changing browser requests,
including login, require origin/CSRF admission. Logout is a mutation. No bearer in
URLs, browser localStorage, logs, model context, MCP arguments or Skills. Android /
other token transports must preserve the same M00 checks in their own adapter.

Initial login policy allows at most 5 failed attempts per tenant/login name and
30 per network source in a rolling 15 minutes. Capacity is reserved before hashing.
Backend provisioning requires administrator admission before credential preparation;
it is not a public registration path. SSO transaction bounds are pinned by its adapter.
Unknown and disabled accounts follow the same bounded credential-check path and
public error as wrong passwords. Buckets are bounded and shared across workers;
limiter failure denies authentication rather than permitting unlimited attempts.
M10 accepts forwarded client addresses only from explicitly configured trusted proxies.
Administrative recovery avoids an indefinite account lock caused by another caller.

Closed result classes are `InvalidRequest`, `AuthenticationFailed`,
`NameUnavailable`, `Unauthenticated`, `Forbidden`,
`Conflict`, `RateLimited { retry_after_seconds }`, and `Unavailable`.
Only admitted administrator configuration permits a name-availability result.
Unknown external mapping and failed SSO verification share the public authentication
failure outcome. Repository paths, hashes, raw submitted names/secrets,
SQL errors and adapter diagnostics never appear in these responses. M10 owns exact
versioned status/DTO mapping; no public route is ready until those schemas are bound.

Audit stores event/command/correlation IDs, opaque actor and target account IDs, revisions,
operation class, time and redacted disposition. Secret values, password verifiers,
raw external subjects and request bodies are absent. Login-rate metrics use bounded
labels; no username/IP per-series labels. Readiness remains false on invalid stores,
missing required credential configuration, clock or migration failure.

## 6. SSO adapter and migration

SSO is a supported product entry path whose live adapter remains unimplemented.
Verified `(issuer, subject)` resolves an administrator-configured application account
through M00; raw provider subjects are not platform UserIds. Initial linking and
replacement require admitted administrator/operator configuration plus independently
verified evidence from the configured external authority. A browser login transaction
must be independently verified and bound to its initiating browser; callback payloads
alone cannot authorize mapping edits. Uniqueness conflicts stop linking. No email/name
matching, account merge, privilege change or Plugin grant follows from SSO.

Actual protocol, approved issuer, callback registration and validation requirements
must come from real integration conditions; no USTC endpoint/protocol is invented.
Until those inputs are available, use a fail-closed adapter and controlled test evidence;
keep live SSO unavailable. Local user configuration and independent product development
need not wait for campus integration. See
[the disabled SSO example](../../examples/sso-interface/README.zh-CN.md).

Persisted schemas are versioned. Migration validates a backup and candidate before
switching authority; unsupported versions and interrupted publication fail closed.
Rollback uses the compatible pre-migration snapshot with an explicit recovery step;
never run an old writer on an unknown new schema. Restored credentials/sessions must
not revive acknowledged revocations. Advancing the deployment admission epoch alone
only invalidates old sessions; it does not make restored password, account or SSO-link
state current. The initial recovery path therefore quarantines every restored
credential/link before opening admission.
All restored session bearers stay invalid. M00 owns the recovery admission disposition;
M90 durably records its epoch/fence outside the restored snapshot before readiness.
The epoch does not rewrite historical M00 session events.

No old password may be used to obtain a fresh session while its account is in recovery
quarantine. Local operator recovery must reconcile current account/role/revocation
status from trustworthy recovery evidence and explicitly re-enrol each admitted
account with a fresh credential binding (or independently reverified SSO association).
A restored snapshot alone is not that evidence. Unresolved accounts stay quarantined;
no silent reactivation or last-known-role restoration is allowed. Backend configuration
requires a currently recovered administrator or explicit local operator. If no trustworthy
post-backup authority can be established, keep login closed for those identities.
A future replay of an external monotone revocation journal may automate reconciliation
only under its own reviewed durability/recovery contract.

Legacy demo session, profile and publication identities are preserved. Unscoped
calendar files cannot be silently assigned to the first account. Browser pending
operations must not replay under another login. New service mode starts in a distinct
state set; any later legacy import names an explicit target owner and produces a
reviewable import receipt without rewriting source receipts or grants.

## 7. Acceptance and implementation sequence

All cases below are **planned**. IDs are proposed bindings until registered in the
active acceptance matrix; this document does not claim tests or service readiness.

| Case | Required observable evidence |
|---|---|
| ACCOUNT-001 | Concurrent bootstrap yields one administrator; remote bootstrap absent; backend configuration creates members only; public registration absent; tenant mismatch and last-admin disable rejected |
| ACCOUNT-002 | Concurrent backend configuration commits one exact account/name/link; name/password failure leaves no partial account; lost-response retry creates no second account or login |
| ACCOUNT-003 | Known, unknown and disabled account login failures use one public outcome; bounded hashing, throttling and limiter-unavailable paths run before session creation |
| ACCOUNT-004 | Session opens only after durable session plus binding commit; expiry, logout and credential reset block fresh admission, including on another worker |
| ACCOUNT-005 | Two browser users cannot read/mutate each other's private test resource, reuse another account's pending operation or self-assert an actor; anonymous private requests denied |
| ACCOUNT-006 | Injected configuration/session commit failure and process restart preserve atomicity; corrupt/unknown state fails readiness; restore rejects old sessions, reset passwords, disabled accounts and superseded SSO mappings; quarantined identities cannot fresh-login before explicit reconciled re-enrolment |
| ACCOUNT-007 | Cookie/origin/CSRF behavior, exact wire bounds and secret-free responses/logs verified with a real browser and controlled credential adapter; real hash adapter receives separate evidence |
| ACCOUNT-008 | Fake verified SSO adapter rejects wrong issuer/subject binding, duplicate link and unverified evidence; no campus SSO success is claimed without real integration evidence |

Sequence: pure account decisions and fakes; durable account/credential adapter;
canonical session transaction; M10/backend user configuration and login/me/logout;
verified SSO adapter when integration inputs exist; two-user private-resource assembly.
Product modules develop in parallel against controlled admitted contexts, so this
account sequence does not precede independent Chat, Market or campus-journey work.
Targeted tests and one relevant real path accompany each slice. Full baseline checks occur at integration checkpoints; the
existing demo is not renamed a multi-user service while private paths remain shared.
