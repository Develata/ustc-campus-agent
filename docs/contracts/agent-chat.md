# Bounded Web Chat API and delivery contract

## Metadata

- `Status`: Implemented bounded MVP contract
- `Version`: `agent-chat/v1`
- `Last Review`: `2026-09-04`
- `Owning Plan`: [`../plan/07-runtime-and-integration.md`](../plan/07-runtime-and-integration.md)
- `Feature Projection`: [`../features/04-bounded-agent-harness.md`](../features/04-bounded-agent-harness.md)
- `Acceptance`: active `CHAT-001`, `CHAT-002`, `CHAT-003`
- `Primary Code`: `apps/ustc-agentd/src/agent_chat.rs`, `chat_provider.rs`, `chat_tools.rs`, `web.rs`

## 1. Scope and authority

`agent-chat/v1` is one loopback-only competition profile. It is not the complete durable [`agent-harness/v0`](agent-harness.md), a generic Plugin runtime, a provider-managed conversation, or production campus-source activation.

```text
Composition-owned static Web Chat shell (not M80 module evidence)
→ M10 deployment-local login/session admission
→ M10 POST /api/v1/agent/chat
→ M30 bounded in-memory coordinator
→ M50 deterministic mock or operator-configured OpenAI-compatible adapter
→ exact sequential tool bridge
→ existing Affairs / ChangeRadar fixed read path
   or optional owner-local Simple Calendar path
   or existing consent-bound static M72 planner path
→ natural-language answer + redacted tool trace
```

The server owns local-access admission, request validation, provider selection, budgets, tool registration and product composition. The browser and provider mint no route, tenant, user, grant, profile, source, publication or administrator authority. The local-access username is only a deployment gate for this loopback product shell; it is not M00 identity or domain authorization. The three campus-data tools use only reviewed repository fixtures; the planner fixture includes bounded public iCourse aggregate-rating link-outs as orientation-level soft evidence but no copied review text. The Calendar tool uses only owner-local state. This contract grants no USTC network or real-source activation permission.

## 2. Deployment-local access session

The Compose profile requires exactly one local username and one Argon2id PHC password verifier. It offers login and logout, but no registration, password recovery, role management, USTC SSO/CAS, multi-user tenancy or durable browser-session recovery.

Server-only configuration is:

| Key | Contract |
|---|---|
| `UCA_ADMIN_USERNAME` | required 1–64 byte printable ASCII username; default package value `admin` |
| `UCA_ADMIN_PASSWORD_HASH_FILE` | required `serve-web` secret-file reference to a regular non-symlink owner-readable file containing exactly one bounded Argon2id PHC verifier |

The verifier uses Argon2id v19, a random salt, a 32-byte output and server-admitted parameter bounds. Raw passwords are accepted only by the JSON login body and the operator's interactive hash-generation process; they never enter argv, files, browser storage, logs, response diagnostics, image layers, archives or tracked configuration. The operator CLI emits only the PHC verifier. Password comparison and session-token lookup use constant-time cryptographic verification.

`GET /api/v1/auth/session` returns HTTP `200` with the closed `ustc-local-access/v1` envelope. It reports `authenticated`, and only when true includes the configured username plus safe provider mode/model identity. It never returns the password verifier, token, provider key or internal expiry timestamps.

`POST /api/v1/auth/login` requires `application/json` and accepts only:

```json
{"schema":"ustc-local-access-login/v1","username":"admin","password":"..."}
```

Username and password are nonblank, contain no U+0000 and are bounded to 64 and 1024 UTF-8 bytes respectively. Invalid shape returns `400 invalid_login_request`. Invalid credentials return the same `401 invalid_credentials` response regardless of which field mismatched. Five consecutive failures enter one 60-second loopback-wide cooldown returning `429 rate_limited`; a successful login resets the counter. No diagnostic echoes submitted credentials.

A successful login mints a 256-bit OS-random opaque token, retains only its SHA-256 lookup digest in process memory, and returns `Set-Cookie: uca_session=<base64url>; Path=/; HttpOnly; SameSite=Strict; Max-Age=43200`. At most eight current sessions exist; each has a 30-minute idle and 12-hour absolute deadline. Access refreshes only the idle deadline within the absolute bound. Daemon restart, password rotation, expiry and `POST /api/v1/auth/logout` invalidate the session. Logout clears the cookie and is idempotent.

The login/session/logout handlers, `POST /api/v1/agent/chat`, every `/api/v1/opportunity/*` route and both administrator-publication `POST` routes require the same loopback Host and same-authority Origin admission as the rest of the router. Chat, Opportunity and administrator-publication mutations additionally require the unexpired cookie and otherwise return HTTP `401` with `{"schema":"ustc-local-access/v1","error":"authentication_required"}` before provider, profile or product I/O. SameSite Strict, JSON-only mutation requests and Origin admission form the bounded CSRF control for this HTTP-only loopback profile; production HTTPS must additionally set `Secure`.

Static assets, `/healthz`, protocol bootstrap/capability routes, public Affairs/ChangeRadar reads and administrator-publication `GET` status projections remain outside this local-access gate. Each publication `POST` requires both the local-access cookie and its existing explicit confirmation header; the cookie admits the loopback caller but does not itself infer M00 administrator authority. The operator CLI therefore retains public-read diagnostics but receives fail-closed `401` for an unauthenticated publication attempt.

## 3. HTTP Chat request

`POST /api/v1/agent/chat` accepts `application/json` only, under the router's 16 KiB complete-body limit, after the local-access session gate. The loopback router requires a `Host` authority whose host is `localhost` or a numeric loopback IP (`127.0.0.0/8` or `::1`) and contains no userinfo; when `Origin` is present it must be the same HTTP authority. Invalid Host, cross-origin or unauthenticated requests fail before provider or tool I/O.

```json
{
  "schema": "ustc-agent-chat-request/v1",
  "messages": [
    {"role": "user", "content": "成绩单证明怎么办？"}
  ],
  "opportunity_context": null
}
```

The request is closed:

- `schema` is exactly `ustc-agent-chat-request/v1`;
- `messages` contains 1–12 entries;
- each entry has only `role` and `content`;
- `role` is exactly `user | assistant`; browser/provider callers cannot submit `system` or `tool` history;
- each UTF-8 `content` is nonblank, contains no U+0000/NUL scalar, and is at most 4 KiB; aggregate message content is at most 12 KiB; the final role is `user`;
- `opportunity_context` is absent/null unless the browser has an existing profile hint and the user enables the explicit per-request chat-use control;
- a non-null `opportunity_context` is exactly `{"profile_snapshot_id":"..."}`; its value is nonblank, contains no U+0000/NUL scalar and is at most 4 KiB; scalar aliases and unknown fields fail;
- a non-null context additionally requires `X-USTC-Opportunity-Confirmation: confirmed` on the same request.

The profile snapshot ID and header are non-authoritative hints. The browser applies the same nonblank/NUL/4 KiB bound before enabling a restored `localStorage` hint and removes an invalid persisted value. The server still checks current session, tenant/user ownership, consent, Market state and source currentness through the existing Opportunity composition.

## 4. HTTP Chat response and errors

Success is closed by this shape:

```json
{
  "schema": "ustc-agent-chat-response/v1",
  "run_id": "chat-run:...",
  "answer": "...",
  "provider": {"mode": "mock", "model": "deterministic-mock-v1"},
  "tool_trace": [
    {"call_id": "call-1", "tool": "affairs_navigator_get", "status": "succeeded"}
  ],
  "usage": {"input_tokens": 0, "output_tokens": 0}
}
```

`answer` is nonblank and at most 16 KiB. `usage` is the saturating sum of provider-reported prompt/completion tokens; the deterministic mock reports zero. `tool_trace` exposes only a bounded server-owned opaque `call_id` assigned in execution order, model-visible tool name and `succeeded | denied | failed`; provider-supplied correlation IDs remain private to the provider transcript. The trace exposes no private route, product payload, package/grant internals, profile content, request headers, provider body, URL or API key. For Affairs Navigator and ChangeRadar, `succeeded` requires the enclosed typed terminal outcome to be `found`; an admitted envelope carrying `not_found`, conflict or another non-`found` terminal is projected as `failed`, not successful completion.

Host/Origin admission runs before Chat route dispatch and therefore uses the shared Web envelope rather than `ustc-agent-chat-error/v1`. Missing, malformed, userinfo-bearing or non-loopback `Host` returns HTTP `421` with `{"schema":"ustc-web-error/v1","error":"invalid_loopback_host"}`. A present non-HTTP or authority-mismatched `Origin` returns HTTP `403` with `{"schema":"ustc-web-error/v1","error":"cross_origin_request_forbidden"}`. Both occur before provider/tool I/O and carry the same hardened response headers as other loopback Web errors.

Errors after Chat route admission are `{"schema":"ustc-agent-chat-error/v1","error":"stable_code"}`. Stable codes are:

```text
invalid_chat_request
provider_not_configured
provider_unauthorized
provider_rate_limited
provider_timeout
provider_unavailable
provider_protocol_error
context_budget_exceeded
tool_call_rejected
tool_result_too_large
tool_budget_exhausted
turn_budget_exhausted
opportunity_confirmation_required
composition_unavailable
internal_chat_error
```

Provider diagnostics and secret-bearing values never enter the response.

## 5. Provider profile and transport

Runtime configuration is server-only:

| Key | Contract |
|---|---|
| `UCA_AGENT_PROVIDER` | exact `mock` (default) or `openai-compatible` |
| `UCA_AGENT_BASE_URL` | required for `openai-compatible`; absolute HTTPS, no userinfo/query/fragment; fixed join with `chat/completions` |
| `UCA_AGENT_MODEL` | required for `openai-compatible`; bounded nonblank model ID |
| `UCA_AGENT_API_KEY_FILE` | required for `openai-compatible`; regular non-symlink file read once at server startup |
| `UCA_AGENT_TIMEOUT_MS` | integer 1000–60000; default 15000 |
| `UCA_AGENT_CONTEXT_TOKENS` | required for `openai-compatible`; validated integer 16384–1048576 |

The packaged launchers require `.env` itself to be a readable regular non-symlink file when present, and require at most one exact column-zero `KEY=value` assignment for each of `UCA_AGENT_PROVIDER` and `UCA_AGENT_API_KEY_SOURCE`. Their values must be literal: the launchers reject all `$`-based Compose interpolation in either security-critical `.env` assignment before Docker, including an otherwise-unused key-source assignment in mock mode, so launcher-side security preflight cannot observe a value different from the Compose-resolved service. Operators needing dynamic configuration inject already-resolved literal process-environment values instead.

The key file is UTF-8, nonblank after outer-whitespace trim and at most 4096 bytes. On Unix, the opened key file must have no group/world permission bits (`mode & 077 == 0`). Because local Compose file-backed secrets preserve host ownership, Compose first drops every capability and then grants the root-only initialization phase exactly `CHOWN`, `DAC_OVERRIDE`, `FOWNER`, `SETGID`, `SETPCAP` and `SETUID`: the entrypoint can read the explicitly mounted owner-only source, copy it into an ephemeral mode-0600 tmpfs file owned by UID/GID 65532, and then re-exec itself through `setpriv` as UID/GID 65532 with cleared groups, no-new-privileges and an empty effective/bounding capability set before the daemon or proxy starts. The packaged Unix launcher enforces the same permission rule on the host source before Docker runs, while direct Compose use remains operator-responsible because the projected container secret cannot prove the host file's original mode. Both launchers, the container entrypoint and the authoritative Rust key reader reject the bundled mock placeholder after the same outer-whitespace normalization in `openai-compatible` mode. The normal runtime accepts no raw key through argv, HTTP, browser storage, checked-in environment or logs. Invalid OpenAI-compatible configuration fails startup without fallback to mock, another origin or another model.

The adapter sends non-streaming Chat Completions with the exact configured model, ordered complete messages, complete current tool definitions, `tool_choice: auto`, `parallel_tool_calls: false`, `stream: false` and an 8192-token output ceiling. A retained test exercises this adapter through the complete loopback `POST /api/v1/agent/chat` route against a bounded local provider peer, including provider identity, usage and hardened-response projection. The response path accepts only exactly one `assistant` choice, requires `finish_reason: stop` for final text or `finish_reason: tool_calls` for a complete tool batch, and rejects truncated, content-filtered or mismatched termination before any tool execution. It follows no redirects, uses one absolute timeout and accepts at most 256 KiB of response bytes. Production configuration requires HTTPS; plain HTTP exists only in the test-only loopback constructor. The deterministic mock is network-free and routes only product-qualified transcript, academic-calendar, course-planning and Calendar terms. For each known successful tool shape it projects only bounded user-facing fields into a server-owned Chinese summary: procedure steps and official entry points, semantic changed fields and source link, course candidates/rationale/iCourse link-outs, or Calendar mutation/list details. Known-tool shape drift yields an explicit summary-contract notice instead of a raw JSON dump. Fair per-result output budgets ensure one large result cannot erase later successful tools; denied and failed statuses remain explicit non-success answers, and a mixed request whose Opportunity tool is unavailable retains an explicit unexecuted-consent notice beside any successful public-tool summary.

Before network I/O the adapter serializes the complete wire request and applies `T(q) + O + S ≤ floor(L × 0.9)`, where `T(q)` is conservatively upper-bounded by serialized UTF-8 bytes, `O=8192`, `S=2048`, and `L=UCA_AGENT_CONTEXT_TOKENS`. Oversize input fails locally as `context_budget_exceeded`; no provider/profile context limit means no OpenAI-compatible call.

A successful provider message must carry the exact `assistant` role and either nonblank final text or function calls. Missing/non-assistant roles, malformed JSON, empty/multiple choices, invalid call objects and oversized output map to `provider_protocol_error`. HTTP 401/403, 429, timeout and remaining non-success transport classes map to their stable errors without returning the raw body. The deterministic mock derives its wording from server-owned tool status/data. An operator-selected real provider remains an untrusted text generator: the server preserves the independently rendered tool-trace status but cannot prove that arbitrary provider prose describes a denied/failed result honestly; operators must treat the trace as authoritative.

## 6. Bounded sequential loop

One accepted request creates one finite in-memory `ChatRun` and pins:

- at most three provider turns;
- at most four total tool calls;
- strictly sequential execution in provider order;
- at most 4 KiB raw JSON arguments per call;
- at most 64 KiB serialized tool result per call;
- at most 16 KiB final answer.

Each provider turn yields either a nonblank final answer with no tool calls, or function calls that fit the remaining budget. Mixed text plus calls treats text as nonterminal provider data. Duplicate/blank call IDs, duplicate object members, unknown/missing arguments, non-function types, unknown tools and budget overflow fail before the affected product operation. Counters never reset inside the request.

Tool output is bounded typed M10 data wrapped as untrusted provider input. It cannot add tools, messages or policy. Failure to produce a valid final answer within the turn budget is explicit failure, not partial success.

## 7. Exact tool map

### `affairs_navigator_get`

Input is exactly `{"procedure_id":"proc:ustc:undergraduate:transcript-certificate"}`. The bridge invokes only the existing public-redacted Affairs query. Any non-`Available/Public/Found` outcome is denied or failed rather than model-authored procedure truth.

### `change_radar_get`

Input is exactly `{"board_id":"board:ustc:academic-calendar"}`. The bridge invokes only the existing typed ChangeRadar query. No administrator publication operation is model-visible.

### `simple_calendar_items`

Input is a closed object with exact `action = record | list | delete`. `record` additionally requires a nonblank title of at most 256 UTF-8 bytes and optionally accepts one RFC 3339 `scheduled_for`; `list` accepts no other field; `delete` requires one stable `calendar:item:N` ID. Rust revalidates the action-specific shape before execution. The loopback profile persists at most 128 owner-local items in a sibling state file and returns success only after the mutation is durably written. It has no reminder, recurrence, sharing, synchronization or natural-language time semantics. Calendar writes must reflect an explicit user instruction.

### `opportunity_graph_plan_current_profile`

Input is exactly `{}`. This definition is omitted unless the exact request has both a valid `opportunity_context` and the confirmation header. Composition inserts the profile ID out of band and invokes the existing static `GeneratePlan` operation with `max_results=3` and `beam_width=1024`.

The model cannot create, view, edit, consent to, revoke or delete a profile; choose a different profile ID; or add courses outside deterministic planner output. A stale, missing, disabled, revoked, cross-principal or otherwise denied current profile returns a bounded non-success tool result.

## 8. Web, Compose and package projection

The thin static browser is a chat-first application shell and owns only local login presentation plus page-lifetime draft/history presentation. Desktop uses a restrained left rail, central message stream and bottom composer; mobile collapses the rail into a keyboard-accessible drawer. The rail exposes only real behavior: New chat, current Chat, a secondary campus-tools/diagnostics view, safe provider mode and local-account logout. It never fabricates saved conversations, projects or durable history. The shell sends bounded user/assistant history, renders loading/final/error/tool-trace states, visually labels tool trace as authoritative evidence, provides an explicit local-conversation clear control, and includes Opportunity context only after profile creation plus explicit checkbox confirmation. It never receives or stores the provider key, password, password verifier or session token outside the `HttpOnly` cookie. Keyboard submit, visible focus, reduced motion and 320/390 px/mobile-to-desktop layout remain required.

The Compose package:

- publishes only `127.0.0.1:${UCA_MVP_PORT}:8787`;
- defaults to deterministic mock with no provider network call;
- requires an operator-created Argon2id verifier secret, supports interactive setup/rotation without raw password argv/files/logs and copies only the verifier into ephemeral owner-only runtime storage;
- mounts a provider key source read-only, copies it only in OpenAI-compatible mode into an ephemeral mode-0600 tmpfs file, uses only a non-secret placeholder in mock mode and rejects that placeholder in real-provider mode;
- persists product and Simple Calendar state in a named volume across `stop`/restart;
- invalidates in-memory browser sessions on restart or credential rotation while preserving product state;
- deletes that volume only through explicit reset;
- packages deterministic ZIP/tar archives with exact source commit, per-file checksums and a provider-secret scan;
- keeps `.ps1`/`.cmd` launchers ASCII-only, BOM/NUL-free and LF-terminated for Windows PowerShell 5.1, with native-command `$LASTEXITCODE` checks, bounded port/health retry and controlled `compose ps -a`/log diagnostics when the container exits before readiness;
- runs smoke verification under a unique Compose project so cleanup cannot address a user's normal MVP volume.

## 9. Non-goals

This version does not claim live campus-source ingestion, registration, password recovery, CAS/SSO, production authentication, multi-tenancy, generic Plugin installation/execution, provider fallback, streaming, RAG, durable chat history, long-term memory, reminders, calendar synchronization, parallel tools, multi-agent graphs, Dioxus parity or production hosting. Real-provider smoke remains `not-run` unless an operator separately supplies runtime configuration and grants provider-network permission.
