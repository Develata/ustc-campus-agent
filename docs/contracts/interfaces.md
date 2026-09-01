# Interface registry

## Metadata

- `Status`: Accepted registry; the Affairs-first protocol-major Web/CLI projection and the static-M72 private-profile slice are retained bounded implementations, ChangeRadar remains bounded partial evidence, and broader production routes remain planned
- `Version`: `application-interface-registry/v2.1`
- `Last Review`: `2026-09-01`
- `Owning plan`: [`M10 Application Ingress Host`](../plan/modules/20-application-api-host.md)
- `Client counterpart`: [`client-shell/v2.1`](client-shell.md)
- `Permission counterpart`: [`permissions.md`](permissions.md)

This registry owns the concrete application-operation, HTTP and inbound-MCP projections. [`module-boundaries.md`](module-boundaries.md) owns large-module crossing rules. An adapter registry is an allowlisted projection of this operation registry; no CLI command, MCP tool, Dioxus server function or HTTP route creates a parallel application authority.

The implemented single-node Agent state/event contract is defined in [`agent-runtime.md`](agent-runtime.md). The planned finite user-task lifecycle is defined in [`agent-harness.md`](agent-harness.md). The Agent–Plugin seam is [`agent-plugin-boundary/v0`](agent-plugin-boundary.md). None makes the operations below operational.

## 1. Application-operation registry

Operation IDs name semantics independently of transport or CLI spelling. Each admitted request maps to exactly one owning application operation after M00/M10 admission. `Initial projections` is an allowlist, not a requirement that every adapter expose the same command set.

| Operation ID | Owner | Permission class | Effect class | Initial projections | Status |
|---|---|---|---|---|---|
| `server.info` | M10 | `public-read` | read | Web, CLI, HTTP | approved Affairs-first protocol slice; bootstrap requires no protocol-major header |
| `capability.list` | M10 | `public-read` | read | Web, CLI, HTTP | approved Affairs-first protocol slice; safe server-supported operation projection only |
| `market.package.list` | M20 | `public-read` | read | CLI, HTTP, inbound MCP | planned first vertical slice |
| `market.package.get` | M20 | `public-read` | read | CLI, HTTP | planned |
| `affairs.search` | M71 | `public-read` | read | CLI, HTTP, inbound MCP | planned after owning product contract |
| `affairs.get` | M71 | `public-read` | read | Web, CLI, HTTP; later inbound MCP | bounded formal protocol-major Web/CLI route projection implemented over exact stable-ID/domain evidence; production auth/TLS/inbound-MCP remains planned |
| `change.list` | M70 | `public-read` | read | CLI, HTTP | bounded exact semantic-change evidence through `ustc-agentd` and the loopback-only JSON/Web/Atom demo; production HTTP projection planned |
| `change.get` | M70 | `public-read` | read | CLI, HTTP | planned after owning product contract |
| `program.list` | M72 | `public-read` | read | CLI, HTTP, inbound MCP | planned after owning product contract |
| `program.get` | M72 | `public-read` | read | CLI, HTTP, inbound MCP | planned after owning product contract |
| `course.search` | M72 | `public-read` | read | CLI, HTTP, inbound MCP | planned after owning product contract |
| `course.get` | M72 | `public-read` | read | CLI, HTTP, inbound MCP | planned after owning product contract |
| `offering.list` | M72 | `public-read` | read | CLI, HTTP, inbound MCP | planned after owning product contract |
| `course.review_linkout` | M72 | `public-linkout` | link-out | CLI, HTTP, inbound MCP | planned after owning product contract |
| `source.provenance` | M60 | `public-read` | read | CLI, HTTP, inbound MCP | planned after owning source/product contract |
| `profile.academic.create` | M72 | `tenant-private-write` | tenant-local mutation | HTTP | active bounded Opportunity Graph slice; exact consent and authenticated owner required |
| `profile.academic.view` | M72 | `tenant-private-read` | read | HTTP | active bounded Opportunity Graph slice; metadata/count projection only |
| `profile.academic.revoke_delete` | M72 | `tenant-private-write` | tenant-local deletion | HTTP | active bounded Opportunity Graph slice; consent revoke and payload deletion are one operation |
| `profile.requirement_status` | M72 | `tenant-private-read` | read | CLI, HTTP, later inbound MCP | later private-data slice |
| `planner.draft.list` | M72 | `tenant-private-read` | read | CLI, HTTP | later private-data slice |
| `planner.draft.get` | M72 | `tenant-private-read` | read | CLI, HTTP | later private-data slice |
| `planner.draft.delete` | M72 | `tenant-private-write` | tenant-local mutation | CLI, HTTP | later private-draft slice |
| `planner.generate` | M72 | `tenant-private-write` | bounded tenant-local planning | HTTP | active bounded Opportunity Graph slice; no enrollment/application side effect |
| `planner.explain` | M72 | `tenant-private-read` | read | CLI, HTTP, later inbound MCP | later private-data slice |

`program.*` means an approved cultivation-program projection. `profile.academic.*` is the exact consent-bound M72 private-profile family; principal identity is derived only from the M00-admitted session and is never caller-supplied inside the operation payload. `planner.draft.*` means a tenant-local planning draft. Neither is an Agent/Harness plan, and no ambiguous `plan.*` alias is admitted.

The first retained formal protocol proof is the Affairs-first subset: `server.info`, `capability.list` and `affairs.get`, projected only to Web and CLI under protocol major `1`. Inbound MCP, `market.package.list`, events/streams and cancellation remain deferred slices. Bounded loopback-only `affairs.get` and `change.list` domain proofs already exist as vertical-slice evidence for `M10 → deterministic Harness → current Market authorization → ToolGateway → fixed first-party owning adapter → M71/M70` and operation-specific presentation surfaces. The active M72 slice is separate: an authenticated M00 actor and exact three-field consent pass transaction-current M20 package/installation/grant/policy authorization for a declarative resource component, then one static owning M72 application use case accesses tenant-private persistence and transaction-current source health. It creates no Agent run/tool projection, provider call, ToolGateway route, effect intent/receipt or PluginExecutor request, and it does not authorize public or inbound-MCP private-profile exposure. The retained product paths remain loopback demos with reviewed/DemoReviewed source inputs and demo admission data; they do not make a remotely exposed production HTTP route, package-portable/out-of-process executor host, general M60 ingestion service, production SSO, broad search, generic M80/Dioxus shell or acceptance rows operational before their exact evidence passes.

### 1.1 Active M72 private-operation wire contract

All four operations use one bounded `SubmitOpportunityDto` envelope and a closed typed command sum. The envelope carries request/correlation/causation/idempotency identities, an **authenticated** session reference, client provenance and a domain-separated payload digest. It carries no tenant or user identifier; M10 forwards only the `M00AdmittedActor::Authenticated` identities.

The command sum is exactly:

- `create_profile`: consent purpose `opportunity_planning`; consent fields exactly `completed_courses`, `credit_bounds`, `preference_weights`; consent time; at most 64 completed course codes; positive `min_credits <= max_credits`; and at most 64 unique course preference weights;
- `view_profile`: one exact `profile-snapshot:opportunity:*` identity;
- `generate_plan`: one exact profile snapshot identity, `max_results` in `1..=8` and `beam_width` in `1..=4096`; the demo journey uses the deterministic pack default `1024`;
- `revoke_consent_and_delete_profile`: one exact profile snapshot identity and a revocation time not earlier than consent.

M10 recomputes the payload digest before admission, selects the operation descriptor from the command variant and denies any variant/descriptor drift. The typed terminal sum is exactly `profile_created | profile_found | plan_generated | profile_deleted`. Profile projections expose consent/profile identities, consent purpose/fields, consent time and only completed-course/preference **counts** plus credit bounds; they do not echo completed-course codes or preference weights. Planning projections may expose selected course codes and owner-private qualification inputs because the entire route is tenant-private, but `Debug`, logs, public lookup/cache and sibling Plugin projections must redact the request and terminal. Deletion terminals retain only deletion/profile/consent identities and deletion time.

Failure precedence is: malformed/bounds/digest → M00 session/policy admission → transaction-current M20 package/installation/grant/policy authorization → static M72 application dispatch → owning repository/profile lookup → M60 source health → deterministic planner/projection. A wrong principal, disabled/revoked Plugin or missing/deleted profile reaches neither M60 nor planner. Syntactically valid profile facts that do not belong to the reviewed catalog return typed `invalid_profile_facts`, not an internal error or guessed identity. Stale/conflicting/unavailable source produces a typed refusal, never a best-effort plan. A pre-dispatch application failure is retryable infrastructure failure with zero M72/private-state/M60 calls; if the domain mutation commits but terminal persistence is unavailable, M10 returns `incomplete`/outcome-unknown rather than false success, and exact replay recovers the idempotent terminal. Repository/M20 failures remain distinguishable infrastructure failures rather than source facts. No M10 or Web fallback may call M72 directly after the application port denies or fails.

The package declares private capabilities without auto-granting them. `profile.academic.create` and `profile.academic.revoke_delete` require `user.own_academic_snapshot.write`; `profile.academic.view` requires `user.own_academic_snapshot.read`; `planner.generate` requires `user.own_plan_draft.write`. All three private capability definitions are `TenantPrivateUser`, `autoGrant=Never`, `confirmationDefault=Ask`. A first-party package declaration is therefore not authority; each invocation still requires a transaction-current grant snapshot and recheck.

## 2. Schema identity and grant invalidation

Every public operation has a versioned request schema, result/error schema and canonical schema digest in the future M10-owned machine registry. Every CLI/MCP/Dioxus projection references that operation identity; it does not copy and reinterpret the schema.

- A compatible descriptive change that does not alter accepted fields, result meaning, data class or effect class may retain the schema identity.
- Adding or widening input fields, data exposure, permission class, effect class, external target or result authority requires a new schema identity/digest.
- An existing private or delegated grant bound to the old operation/schema digest becomes stale and must be explicitly re-approved before the changed projection is advertised or invoked.
- Public-read operations still pass current server capability and policy admission on every call; public classification is not a client-side bypass.
- Unknown operation or schema identities fail closed. No same-name, prefix or nearby operation fallback is admitted.

## 3. Application HTTP endpoints

Routes are transport projections of §1 operations. An endpoint may be a versioned Dioxus server function, an explicit Axum route, or both when the same wire contract is intentionally admitted.

| Route | Method | Operation/projection | Status |
|---|---:|---|---|
| `/api/v1/server/info` | GET | `server.info`; protocol-major bootstrap, no version header required | approved Affairs-first retained slice |
| `/api/v1/client/capabilities` | GET | `capability.list`; exact Web/CLI operation/schema/permission/effect/route allowlist, no tenant grants or operator registry | approved Affairs-first retained slice; major header required |
| `/api/market/packages` | GET | `market.package.list` | planned |
| `/api/market/packages/{id}` | GET | `market.package.get` | planned |
| `/api/installations` | POST | future M20 installation operation; outside the initial external-Agent projection | planned by owning M20 contract |
| `/api/installations/{id}:disable` | POST | future M20 installation operation; outside the initial external-Agent projection | planned by owning M20 contract |
| `/api/agent/runs` | POST | future finite HarnessRun operation family | planned by owning harness contract |
| `/api/agent/runs/{id}` | GET | future HarnessRun projection | planned by owning harness contract |
| `/api/agent/runs/{id}/answers` | POST | future bounded clarification operation | planned by owning harness contract |
| `/api/agent/runs/{id}:cancel` | POST | future typed cancellation operation | planned by owning harness contract |
| `/api/agent/runs/{id}/events` | GET/SSE | future HarnessRun event projection | planned by owning harness contract |
| `/api/v1/affairs/{procedure_id}?as_of=<unix-ms>` | GET | `affairs.get`; public-redacted typed M71 terminal after the existing M00→M10→M71 path | approved protocol-major Web/CLI projection over bounded loopback composition; production auth/TLS not claimed |
| `/api/v1/changes/{board_id}` | GET | `change.list`; typed M70 board result from M00→M10→bounded Harness→current Market authorization→ToolGateway→ChangeRadar, rendered by the colocated thin Web page | bounded loopback-only demo; production auth/TLS/durable M10 lookup not claimed |
| `/api/v1/changes/{board_id}/atom` | GET | RFC 4287 Atom projection over the same reviewed ChangeRadar publication repository and bounded invocation path | bounded loopback-only demo; production feed persistence/distribution not claimed |
| `/api/v1/opportunity/profiles` | POST | `profile.academic.create`; exact consent plus tenant-private profile payload | active bounded loopback-only demo contract; production SSO/TLS not claimed |
| `/api/v1/opportunity/profiles/{profile_snapshot_id}` | GET | `profile.academic.view`; authenticated owner metadata/count projection | active bounded loopback-only demo contract; no public/browser cache |
| `/api/v1/opportunity/plans` | POST | `planner.generate`; source/profile-grounded typed qualification and bounded plan | active bounded loopback-only demo contract; no enrollment/application side effect |
| `/api/v1/opportunity/profiles/{profile_snapshot_id}/revoke-delete` | POST | `profile.academic.revoke_delete`; one consent-revoke/private-payload-delete operation | active bounded loopback-only demo contract; backup-erasure beyond owned demo state not claimed |

The bounded Web demo also serves `/`, `/assets/styles.css`, `/assets/app.js` and `/healthz` from the same `ustc-agentd serve-web` process. `serve-web` rejects non-loopback bind addresses. The Affairs form accepts only a procedure ID; the ChangeRadar section requests one fixed reviewed board identity and offers its sibling Atom projection. The active Opportunity Graph section uses only the server-owned demo session and synthetic profile input; the session/tenant/user identifiers are never accepted from a browser field or returned to JavaScript. All sections render only server-owned typed results with `textContent`; the page performs no source, freshness, conflict, eligibility, procedure, semantic-diff or planning calculation. The server creates bounded request/correlation identities, recomputes each operation's payload digest and invokes the ordinary M00/M10 admission path. For Affairs, the response-only public capability minted by submit is consumed immediately by an internal typed lookup and MUST NOT be serialized, logged, stored in browser state or placed in a URL. ChangeRadar returns its bounded synchronous typed terminal. Opportunity responses are owner-private and are never written to the public M10 lookup store. Static and API responses are `no-store`, `nosniff`, frame-denied and same-origin constrained. This demo surface is not a compatibility promise for the future Dioxus/production API.

`GET /api/v1/server/info` is the bootstrap exception and requires no protocol header. Every other formal route in the Affairs-first registry carries `X-USTC-Client-Protocol-Major`. Major `0` (or any future older supported-domain value below minimum `1`) returns typed `upgrade_required` and HTTP `426`; a newer, absent or unparseable major returns typed `incompatible_protocol` and HTTP `409`; neither reaches application dispatch. Matching major `1` proceeds to existing size, identity, authorization, idempotency/precondition and audit admission before dispatching one application operation. A server function or HTTP/SSE route MUST NOT call concrete repositories, databases, Plugin executors, provider SDKs or journals directly.

## 4. Client adapter projections

| Client adapter | Direction | Admitted transport | Registry constraint |
|---|---|---|---|
| Dioxus Web/Android | user → platform | generated server function / typed events or equivalent M10 transport | presentation only; explicit operation allowlist; no CLI/process bridge |
| `ustc-agent` | user/script → platform | production contract: explicit versioned HTTP/JSON and SSE; bounded current evidence: numeric-loopback fixture framing only | least-privilege user profile; its command registry projects a subset of §1 and contains no operator command or raw secret/session argv |
| inbound MCP | external Agent → platform | reviewed MCP Streamable HTTP surface mapped through client-core to explicit M10 routes | read-only public slice first; exact tool/resource allowlist and schema digests; no operator/domain/M51 reach-through |
| M51 outbound MCP | platform → external MCP server | M51 binding/session/executor contract | opposite direction; never an M80 client adapter |

CLI, MCP and GUI are semantically equivalent only where they project the same operation. Authentication, local configuration, presentation and platform-specific maintenance commands need not appear as MCP tools.

## 5. Inbound MCP registry — phased projection

The inbound MCP surface is a server offered to an external personal Agent acting as MCP client. The first remote profile uses reviewed MCP Streamable HTTP. Local stdio/relay is later and requires a separately accepted deployment/session contract.

**Initial public-read projection**

```text
market.package.list
```

**Campus read projection after owning product contracts exist**

```text
affairs.search
affairs.get
program.list
program.get
course.search
course.get
offering.list
course.review_linkout
source.provenance
```

**Later explicitly delegated private projection**

```text
profile.requirement_status
planner.generate
planner.explain
```

`planner.generate` creates only a tenant-local draft. It does not enroll, register, pay or submit a transaction to any external campus system. MCP discovery advertises an operation only when its exact schema digest, caller profile, tenant scope, current capability/grant and result bounds are admitted.

## 6. Agent tool protocol — H0 values implemented, production execution planned

| Object | Direction | Purpose |
|---|---|---|
| `AgentToolsetView` | resolver/gateway → Agent | immutable per-turn complete tool definitions plus opaque private route references |
| `AgentToolCall` | Agent → ToolGateway | provider-neutral correlated call against the exact frozen projection |
| `PluginExecutionRequest` | ToolGateway → PluginExecutor | authorized bounded execution request after effect intent persistence |
| `PluginExecutionOutcome` | PluginExecutor → ToolGateway | non-authoritative bounded outcome for validation and receipt persistence |
| `AgentToolResult` | ToolGateway → Agent | correlated bounded result/evidence/receipt projection for the next model turn |

This Agent tool protocol and M51 outbound MCP direction are independent from the M80 inbound MCP projection above.
