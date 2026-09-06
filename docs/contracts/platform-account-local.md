# Local configured-account adapter

- `Contract ID`: `M00-ACCOUNT-LOCAL-001`
- `Version`: `platform-local-accounts/v1`
- `Status`: bounded local implementation; integration and browser evidence pending
- `Owner`: M00 account application; M90 private-file/Argon2 adapters; M10 transport
- `Authority`: [platform account](platform-account.md), [session](platform-session.md)
- `Scope`: explicit loopback HTTP, operator-configured users, separate new state

This adapter implements the local-operator subset of the account construction
contract. It does not establish public runtime, SSO, administrator HTTP provisioning,
distributed deployment, full recovery or complete ACCOUNT-001..008 acceptance.
One account application serves many users in one configured tenant. Public catalog
and sources retain their shared composition; private owning ports receive both
admitted tenant and user IDs. Browser IDs never establish identity.

## Configuration and credential adapter

`USTC_ACCOUNT_CONFIG` and `USTC_ACCOUNT_STATE` are paired absolute paths. Missing one,
invalid files or corrupt state closes admission. With both absent the existing
loopback demo remains explicitly a demo. Service-mode product stores must use a new
state set; old unscoped calendar files and demo identities are never reassigned.
Every private file has current ownership, mode 0600, one hard link and no symlink;
its parent is an owner-private 0700 directory.

Configuration JSON denies unknown fields and contains `schema`, `tenant_id` and
`accounts`. The schema is `platform-local-accounts/v1`; each account has `user_id`,
`login_name`, `password_hash`, `active`, `administrator`, `credential_generation`.
The configured tenant is unique; account/name uniqueness is enforced. Capacity is
128 accounts. Exactly one active configured administrator is required in this
bounded profile. Login names and passwords retain the parent contract's exact
bounds, including no password normalization. Generation is a positive integer.

RustCrypto `argon2` 0.5.3 is the password adapter. Accepted PHC records use Argon2id,
version 19, memory 19456 KiB, iterations 2, parallelism 1, 32 output bytes and
at least 16 salt bytes. Parameters are checked before verification; unknown
algorithms or excessive work factors fail closed. Salt and 256-bit session bearers
come from the OS random device. Fast SHA-256 is used only for bearer verifiers,
opaque credential-record references and bounded attempt keys, never password hashing.
Authentication evidence is domain-separated random evidence, not a raw-password digest.

The local operator builds the `account_password_hash` example and runs
`scripts/configure_local_account.py CONFIG LOGIN`. The initial invocation requires
`--bootstrap-administrator`; subsequent accounts are members. The helper accepts
passwords through getpass, passes exact bytes through stdin, captures the PHC in
memory and writes one atomic private configuration. Existing names conflict rather
than resetting or logging in. Concurrent helpers lock configuration publication.
Passwords, bearer strings and PHCs are absent from public DTOs and durable session
events. This explicit operator surface has no registration or invitation HTTP route.

## Session transaction and admission

The versioned session store commits canonical `SessionEvent` histories together with
bearer verifier, exact subject, credential generation and opaque credential reference.
Opening/revocation use the existing session `decide`/`evolve` algebra; each admission
replays events and calls `SessionSnapshot::admits_at`. Sessions last eight hours.
Every request reloads active account configuration and checks generation/reference,
so acknowledged logout, account disable or changed credentials deny new admission
on another worker. Roles come from current backend configuration.

Workers hold an exclusive owner-private file lock across account state transactions.
Atomic rename plus file/directory sync commits before login/logout acknowledgement.
The adjacent `.account-fence` revision detects an older session snapshot: publication
interruption or mismatch closes admission. The fence is outside the session snapshot;
restoring both files, restoring operator credential configuration or reconciling a
damaged fence is **not supported recovery**. Keep login closed and follow the parent
contract's explicit operator reconciliation/re-enrolment requirements; this local
profile provides no automatic restore or rollback command. Maximum retained sessions
is 4096; capacity exhaustion denies new login rather than deleting audit histories.

Login reserves capacity before hashing. At most five failures per name and thirty
attempts for the single loopback network source are allowed within fifteen minutes;
buckets are bounded and shared across workers. Successful login clears its account
failure bucket. Disabled and unknown accounts execute the same configured-parameter
password verification path and return `authentication_failed`. This single loopback
source policy is intentionally not a public/trusted-proxy network-source adapter.

## HTTP and browser binding

| Route | Request | Result |
|---|---|---|
| GET `/api/v1/account/mode` | none | `platform-account-mode/v1`: mode, sso_available=false |
| POST `/api/v1/account/login` | `platform-account-login/v1`, login_name, password | safe account + opaque session cookie |
| GET `/api/v1/account/me` | session cookie | `platform-account/v1`: safe account |
| POST `/api/v1/account/logout` | session cookie; same-origin mutation | `platform-account-logout/v1`, logged_out=true |

Safe accounts contain tenant_id, user_id, login_name, administrator and expires_at.
Errors use `platform-account-error/v1` and closed `invalid_request` (400),
`authentication_failed`/`unauthenticated` (401), `rate_limited` (429) or `unavailable`
(503). Rate limits supply Retry-After: 900. Unknown login fields or malformed JSON
reject before hashing. The host's body-size bound applies before DTO decode.

Cookie `uca_local_session` is HttpOnly, SameSite=Strict, Path=/, eight-hour Max-Age,
without Domain or Secure in this expressly loopback HTTP profile. Every non-read
request requires Origin matching the admitted loopback Host. Responses are no-store.
No remote listener/HTTPS production profile is added by this adapter.

Private API admission occurs before handler dispatch and again at private owner
resolution. An `X-UCA-Account-Subject` header, when present, is a JSON pair of the
page's expected tenant/user and must match admitted identity; it grants no authority.
The browser installs its account fetch adapter before other clients, awaits initial
identity, and attaches this expectation to prevent an old tab's pending mutation
from running under a newly logged-in user's cookie. The account client's logout
request also carries the page's captured subject, even though it uses its private
fetch adapter; a missing browser subject prevents submission. A stale tab must not
revoke the current cookie's different account, even with missing or delayed
BroadcastChannel delivery. If logout returns `unauthenticated`, the browser clears
old private projections and reloads current identity; it must not retry a revoke
against the replacement account. Existing non-browser cookie-only logout remains admitted
under the same-origin transport profile; supplying a subject expectation always
requires an exact match. Account changes clear legacy
pending operations/profile hints and reload every notified tab. No token or password
is stored in localStorage/sessionStorage. Legacy demo Opportunity/admin HTTP paths
are unavailable in configured-account mode until they enforce their owning subject.

## Verification binding

Bounded library cases are `accounts::tests::accounts_two_users_restart_logout_and_current_configuration`,
`accounts::tests::accounts_fail_closed_on_snapshot_rollback_and_invalid_hash_parameters`,
and `accounts::tests::accounts_rate_limits_before_hashing_and_no_secret_in_durable_session`.
Run `cargo test -p ustc-agentd --lib accounts::tests` and build the password helper.
These cover only portions of ACCOUNT-003/004/005/006/007; parent ACCOUNT rows remain
planned until registered full evidence exists. Real browser login/logout, two-user
calendar/dialogue/plugin isolation and stale-tab denial are required integration
evidence, not inferred from these account unit cases.

The targeted shared-cookie browser regression uses two dedicated disposable test
accounts in an isolated loopback preview. Its private credential JSON contains an
`accounts` array of `{login_name, password}` objects and must not be committed.
Supply all environment-specific locations explicitly; the test prints no credential
values and does not call model endpoints or restart the backend:

```bash
node apps/ustc-agentd/tests/account_logout_browser.cjs \
  --base "$TEST_PREVIEW_URL" --credentials-file "$PRIVATE_TEST_ACCOUNTS" \
  --playwright-core "$PLAYWRIGHT_CORE" --chrome-path "$CHROME"
```

The default checks the currently served application: with BroadcastChannel disabled,
an old A tab cannot revoke B after a shared-cookie account switch; A's own valid
logout still succeeds, and rejected stale logout clears/reloads the old view to B.
`--inject-local` may substitute local account handlers for a bounded pre-build check.
`--reproduce-old-preview` is a separate opt-in diagnostic for a retained vulnerable
preview, normally paired with `--inject-local` to record before/after evidence. It
is never required by the normal regression or CI. Results default to private local
`dist/account-browser-smoke/logout-subject-result.json`; `--result` selects another
private artifact path. This check does not add a CI workflow.
