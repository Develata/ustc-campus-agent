# Platform account and authentication contract

## Metadata

- `Status`: Accepted semantic contract; implementation planned
- `Version`: `platform-account/v0`
- `Last Review`: `2026-08-02`
- `Owning Blueprint`: [`M00 Platform Control and Identity`](../plan/modules/10-platform-control-identity.md)
- `Depends On`: [`platform-identity/v0`](platform-identity.md), [`platform-session/v0`](platform-session.md), [`module-boundaries.md`](module-boundaries.md)
- `Infrastructure`: [`storage-profiles/v0`](storage-profiles.md)
- `Acceptance`: catalog-only `AUTH-021` through `AUTH-030`; none is active or implemented

## 1. Scope and authority

This contract separates the platform's durable runtime user account from every identifier or claim used to authenticate it.

```text
UserAccount                 platform-owned stable human account
ExternalIdentity            one reviewed external authentication subject linked to that account
ExternalIdentityAlias       login/display alias observed for that subject; never account authority
TenantMembership            account participation and status in one tenant
AuthAssertion               bounded adapter-produced authentication result
AccountLinkDecision         explicit create/link/conflict outcome
AuthenticatedActor          admitted tenant-scoped user projection
```

A `UserId` is the platform identity of a human account. A USTC GID, student/staff number, email address, telephone number, campus-card UID, CAS login name or OIDC subject is not a `UserId` and cannot silently replace one.

This contract does not make a user profile, role, grant, package permission or service principal into an account credential.

## 2. Authority classes

### 2.1 User account

`UserAccount` owns only platform runtime lifecycle:

```text
user_id
status: Active | Suspended | Deleted
created_at
updated_at
revision
```

Invariants:

- `user_id` is stable and opaque; its text encodes no role, tenant, school number, department or enrollment year;
- suspension blocks new authentication admission and sessions but preserves historical receipts and audit identity;
- deletion follows retention and legal policy and never rewrites previously committed receipts;
- an administrator is an authorized user under a membership/grant policy, not an account kind;
- a service or system actor uses a future `ServicePrincipalId`/actor contract and never masquerades as a user account.

### 2.2 External identity

One `ExternalIdentity` links an external provider subject to one platform account inside an exact tenant/provider configuration:

```text
tenant_id
auth_adapter_id
canonical_issuer
provider_subject
user_id
status: Active | Disabled | Revoked
verified_at
last_seen_at?
claims_digest
revision
```

Its uniqueness boundary is:

```text
(tenant_id, auth_adapter_id, canonical_issuer, provider_subject)
```

The canonical issuer and subject are protocol-normalized before lookup. Email, display name and login aliases are not substitute uniqueness keys.

An `ExternalIdentityAlias` may record a provider-scoped GID alias, current or historical person number, email or login spelling with observation time and source. It is searchable only under policy and cannot authenticate, create, merge or relink an account by itself.

### 2.3 Tenant membership and authorization

`TenantMembership` binds `(tenant_id, user_id)` to a tenant membership lifecycle and policy references. Roles and grants are separate authorization facts:

```text
identity/account answers: who is this platform user?
membership answers: does this user participate in this tenant?
role/grant/policy answers: what may this actor do now?
profile answers: what contextual facts may a purpose-bound consumer see?
```

No profile field, school-number prefix, email domain or AI inference directly mints a membership, role or grant.

## 3. Authentication adapter boundary

A protocol adapter validates the upstream exchange and emits one bounded `AuthAssertion` containing at least:

```text
tenant_id
auth_adapter_id
canonical_issuer
provider_subject
authenticated_at
reauthenticate_not_after?
assertion_digest
approved normalized claims
```

The adapter may retain protocol state only for the bounded exchange. Raw passwords, CAS service tickets, cookies, authorization codes, access/refresh/ID tokens and provider secrets never enter domain events, account rows, profile facts, ordinary logs or downstream request context.

`assertion_digest` is domain-separated evidence over approved adapter material; it is not computed by storing or later replaying the raw credential.

### 3.1 Local session deadline

The implemented [`platform-session/v0`](platform-session.md) field `credential_not_after` is populated from the adapter's **local trust/reauthentication deadline**. It is not mechanically copied from a one-use CAS service-ticket lifetime or assumed equal to an OIDC token's `exp`.

The local session still has its own idle and absolute deadlines. Renaming or reinterpreting the implemented field requires a new session contract version and persistence migration; `platform-account/v0` therefore clarifies its adapter input without rewriting the accepted B2 surface.

## 4. Create and link decision

On successful adapter validation, M00 decides exactly one outcome:

```text
known unique ExternalIdentity + Active account/membership
  -> admit the existing account

unknown ExternalIdentity + admitted creation policy
  -> create UserAccount, TenantMembership and ExternalIdentity atomically

explicit authenticated link flow + successful reauthentication
  -> link the new ExternalIdentity to the current account

multiple candidates, existing conflicting link, inactive account/membership,
missing required canonical subject, or uncertain provider identity
  -> fail closed for review/recovery; create no second authority
```

Forbidden automatic linking includes:

- equal email, display name, telephone number or profile text;
- matching current/historical school-number alias without a reviewed canonical-subject rule;
- an AI assertion that two identities describe the same person;
- silent fallback to a different provider or development adapter.

A merge, split or relink is an explicit audited administrative/user-confirmed transition with conflict and rollback policy; it is not an ordinary login side effect.

## 5. USTC CAS boundary

USTC CAS integration is planned only after an institutional attribute/protocol agreement. The development fixture may model:

- one stable GID-like subject;
- current and historical student/staff-number aliases for the same person;
- optional approved attributes such as name, organization or email;
- login, service validation and logout/revocation behavior.

The fixture is not production authority. Production code must not assume every mock attribute is released by USTC. If a stable canonical subject is unavailable or ambiguous, admission fails closed instead of treating the submitted login name as a new platform user.

The demo adapter is explicitly development-only and must be rejected by hosted/production configuration.

## 6. Public operations

Planned M00 application operations include:

```text
AdmitAuthentication
LinkExternalIdentity
DisableExternalIdentity
SuspendUserAccount
RestoreUserAccount
DeleteUserAccount
SetTenantMembershipStatus
OpenSessionFromAdmittedAccount
RevokeAccountSessions
```

Exact Rust constructors, DTOs and error variants remain a later batch contract. No production implementation may land under this semantic contract alone.

Stable failure classes must distinguish at least malformed/untrusted assertion, unknown adapter, issuer/subject mismatch, replay, link conflict, inactive account, inactive membership, policy denial, optimistic conflict, repository unavailable and audit-write failure. Error projections never echo credentials or complete provider claims.

## 7. Persistence and concurrency

M00 declares narrow account, external-identity, membership, session and audit repository ports. M90 implements them under [`storage-profiles/v0`](storage-profiles.md).

Create/link/suspend/delete transitions use expected revision or equivalent compare-and-commit semantics. The uniqueness boundary for an external subject is enforced durably, not only by a preflight query. A losing concurrent create/link returns a typed conflict and then reloads the committed authority; it never creates duplicate users.

## 8. Privacy and observability

Audit may include stable tenant/user/session/provider configuration IDs, assertion digest, command/correlation ID, decision class and redacted reason. Ordinary telemetry excludes raw aliases and approved claims unless an exact security review requires a bounded digest or category.

Account export/deletion and external-identity unlinking must enumerate durable rows, recoverable copies and policy-retained audit references without claiming that historical receipts were anonymously rewritten.

## 9. Design references, not authority

These sources inform the adapter/account-link shape but do not override this repository's contract or constitute USTC production approval:

- [`taoky/ustc-cas-mock`](https://github.com/taoky/ustc-cas-mock) — useful fixture vocabulary for stable GID/current/historical aliases and CAS-like flows;
- [`hexuustc/uniauth`](https://github.com/hexuustc/uniauth) — useful reference for USTC-facing unified-auth integration shape;
- [Ory Kratos identity/user model](https://www.ory.sh/docs/kratos/concepts/identity-user-model) and [identity schemas](https://www.ory.sh/docs/kratos/manage-identities/identity-schema) — separation of identity core, traits and metadata;
- [Supabase Auth users](https://supabase.com/docs/guides/auth/users) and [identities](https://supabase.com/docs/guides/auth/identities) — separation of a stable user record from provider identities.

## 10. Verification

Before implementation admission, exact active acceptance rows and bindings must cover:

- account identity independent from aliases and profile;
- one account with multiple provider identities/aliases;
- atomic first login and duplicate-race behavior;
- explicit linking, conflict and reauthentication;
- inactive account/membership denial before session/downstream calls;
- separate service-principal and administrator semantics;
- raw-credential and claim redaction;
- CAS/OIDC replay and local-session deadline semantics;
- development adapter rejection outside `local-demo`;
- USTC missing/ambiguous claim refusal.

Catalog-only `AUTH-021`–`AUTH-030` preserve these obligations. They remain non-pass until promoted to `docs/acceptance/matrix.tsv` with exact evidence.
