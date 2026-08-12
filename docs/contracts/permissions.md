# Permission and capability contract

## Metadata

- `Status`: Current permission-class contract; concrete client operations remain planned unless their acceptance row is implemented
- `Version`: `permissions/v2`
- `Last Review`: `2026-08-12`
- `Registry`: [`market/capabilities/registry.json`](../../market/capabilities/registry.json)
- `Application operations`: [`application-interface-registry/v2`](interfaces.md)
- `Client boundary`: [`client-shell/v2.1`](client-shell.md)

## 1. Permission classes

| Class | Data/effect boundary | Initial client posture |
|---|---|---|
| `public-read` | read an approved public campus or platform projection | may be allowlisted for CLI/HTTP/inbound MCP; server still rechecks current capability and bounds |
| `public-linkout` | return reviewed external URL/title metadata without caching protected content | explicit URL-owner contract and safe-navigation projection required |
| `tenant-private-read` | read only the authenticated tenant/user's own snapshot, preference or derived status | explicit consent/profile plus tenant/object ownership required; not in initial inbound-MCP slice |
| `tenant-private-write` | create, update or delete only tenant-local drafts | explicit confirmation where required, idempotency/precondition and audit receipt; never external-system submission |

These classes classify an admitted application operation. They are not grants by themselves, and a client/Skill cannot reinterpret one class as another.

## 2. Admission and grant binding

Every non-public request binds:

```text
external caller/session identity
+ admitted platform tenant/user/session
+ delegated client profile
+ exact operation identity and schema digest
+ current capability/grant identity and revision
+ object ownership/scope
+ correlation/idempotency/precondition where applicable
```

Every call is re-authorized by the server. Client-side visibility, MCP discovery, a cached capability projection or previous success is never current authority.

A schema or permission change invalidates approval when it adds or widens any accepted field, result data class, capability, effect, external target, object scope or authority. The old grant becomes stale and the changed operation remains undiscoverable/uninvokable until explicit re-approval. Renaming a tool while retaining broader semantics does not preserve the old grant.

Disabled, revoked, expired, schema-stale or emergency-blocked capability state denies all client projections immediately. There is no same-name tool, operator-command or local execution fallback.

## 3. Client and Skill boundary

- `ustc-agentctl` operator/admin authority is never inherited by `ustc-agent`, Dioxus or inbound MCP.
- Inbound MCP binds an explicitly delegated least-privilege profile; it cannot inherit host/operator credentials.
- Skill text describes when and how to call reviewed CLI/MCP operations and how to stop on typed errors. It contains no credential and grants no capability.
- CLI command registration, MCP tool discovery and GUI action visibility are allowlisted projections of [`interfaces.md`](interfaces.md); none may register an operation absent from the application registry.
- Authentication/login is a client/session procedure, not an MCP business tool exposed to model discretion.

## 4. Credentials and private data

No ordinary client or external Agent receives:

- raw USTC passwords;
- CAS tickets, cookies or complete CAS sessions;
- operator/admin credentials;
- provider or Plugin credentials;
- another tenant's profile, installation, session, result or audit payload.

Secrets remain behind target-appropriate secure session/auth ports. They do not enter argv, Skill content, MCP schemas/results, manifests, normal logs or ordinary audit records. Remote MCP transport never forwards a USTC/CAS credential to the external personal Agent.

## 5. Forbidden in MVP

- raw credential access or credential delegation to a model/Skill;
- cross-user or cross-tenant data;
- automatic enrollment, registration, payment, form submission or any other external campus-system mutation;
- GUI automation of campus websites as a substitute for an admitted operation;
- arbitrary shell, filesystem path, URL, database, container, WebView-eval or code-execution capability;
- arbitrary third-party MCP connection or reviewed-tool bypass;
- silent permission expansion during package, operation or schema upgrade;
- model-generated grants or client-side authority decisions;
- `tenant-private-read` or `tenant-private-write` in the first public-read inbound-MCP slice.

## 6. Later external writes

External campus-system writes are not authorized by this contract revision. A future proposal must separately freeze:

```text
preview/plan
→ explicit user confirmation
→ current capability and object-ownership recheck
→ idempotency and precondition identity
→ durable intent/audit receipt before effect
→ typed outcome and outcome-unknown reconciliation
→ recovery, compensation or explicit irreversibility
```

Until those conditions have an accepted owning contract and executable evidence, the platform remains read-only with respect to external campus systems. Tenant-local drafts are not external writes.
