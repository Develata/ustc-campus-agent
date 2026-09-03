# Bounded Web Chat API and delivery contract

## Metadata

- `Status`: Implemented bounded MVP contract
- `Version`: `agent-chat/v1`
- `Last Review`: `2026-09-03`
- `Owning Plan`: [`../plan/07-runtime-and-integration.md`](../plan/07-runtime-and-integration.md)
- `Feature Projection`: [`../features/04-bounded-agent-harness.md`](../features/04-bounded-agent-harness.md)
- `Acceptance`: active `CHAT-001`, `CHAT-002`, `CHAT-003`
- `Primary Code`: `apps/ustc-agentd/src/agent_chat.rs`, `chat_provider.rs`, `chat_tools.rs`, `web.rs`

## 1. Scope and authority

`agent-chat/v1` is one loopback-only competition profile. It is not the complete durable [`agent-harness/v0`](agent-harness.md), a generic Plugin runtime, a provider-managed conversation, or production campus-source activation.

```text
M80 static Web Chat
→ M10 POST /api/v1/agent/chat
→ M30 bounded in-memory coordinator
→ M50 deterministic mock or operator-configured OpenAI-compatible adapter
→ exact sequential tool bridge
→ existing Affairs / ChangeRadar fixed read path
   or existing consent-bound static M72 planner path
→ natural-language answer + redacted tool trace
```

The server owns request validation, provider selection, budgets, tool registration and product composition. The browser and provider mint no route, tenant, user, grant, profile, source, publication or administrator authority. All three product tools continue to use only reviewed/synthetic repository fixtures; this contract grants no USTC network or real-source permission.

## 2. HTTP request

`POST /api/v1/agent/chat` accepts `application/json` only, under the router's 16 KiB complete-body limit.

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
- each UTF-8 `content` is nonblank and at most 4 KiB; aggregate message content is at most 12 KiB; the final role is `user`;
- `opportunity_context` is absent/null unless the browser has an existing profile hint and the user enables the explicit per-request chat-use control;
- a non-null `opportunity_context` is exactly `{"profile_snapshot_id":"..."}`; its value is nonblank and at most 4 KiB; scalar aliases and unknown fields fail;
- a non-null context additionally requires `X-USTC-Opportunity-Confirmation: confirmed` on the same request.

The profile snapshot ID and header are non-authoritative hints. The server still checks current session, tenant/user ownership, consent, Market state and source currentness through the existing Opportunity composition.

## 3. HTTP response and errors

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

`answer` is nonblank and at most 16 KiB. `usage` is the saturating sum of provider-reported prompt/completion tokens; the deterministic mock reports zero. `tool_trace` exposes only bounded call ID, model-visible tool name and `succeeded | denied | failed`; it exposes no private route, product payload, package/grant internals, profile content, request headers, provider body, URL or API key.

Errors are `{"schema":"ustc-agent-chat-error/v1","error":"stable_code"}`. Stable codes are:

```text
invalid_chat_request
provider_not_configured
provider_unauthorized
provider_rate_limited
provider_timeout
provider_unavailable
provider_protocol_error
tool_call_rejected
tool_result_too_large
tool_budget_exhausted
turn_budget_exhausted
opportunity_confirmation_required
composition_unavailable
internal_chat_error
```

Provider diagnostics and secret-bearing values never enter the response.

## 4. Provider profile and transport

Runtime configuration is server-only:

| Key | Contract |
|---|---|
| `UCA_AGENT_PROVIDER` | exact `mock` (default) or `openai-compatible` |
| `UCA_AGENT_BASE_URL` | required for `openai-compatible`; absolute HTTPS, no userinfo/query/fragment; fixed join with `chat/completions` |
| `UCA_AGENT_MODEL` | required for `openai-compatible`; bounded nonblank model ID |
| `UCA_AGENT_API_KEY_FILE` | required for `openai-compatible`; regular non-symlink file read once at server startup |
| `UCA_AGENT_TIMEOUT_MS` | integer 1000–60000; default 15000 |

The key file is UTF-8, nonblank after outer-whitespace trim and at most 4096 bytes. The normal runtime accepts no raw key through argv, HTTP, browser storage, checked-in environment or logs. Invalid OpenAI-compatible configuration fails startup without fallback to mock, another origin or another model.

The adapter sends non-streaming Chat Completions with the exact configured model, ordered complete messages, complete current tool definitions, `tool_choice: auto`, `parallel_tool_calls: false` and `stream: false`. It follows no redirects, uses one absolute timeout and accepts at most 256 KiB of response bytes. Production configuration requires HTTPS; plain HTTP exists only in the test-only loopback constructor.

A successful provider message must carry the exact `assistant` role and either nonblank final text or function calls. Missing/non-assistant roles, malformed JSON, empty/multiple choices, invalid call objects and oversized output map to `provider_protocol_error`. HTTP 401/403, 429, timeout and remaining non-success transport classes map to their stable errors without returning the raw body.

## 5. Bounded sequential loop

One accepted request creates one finite in-memory `ChatRun` and pins:

- at most three provider turns;
- at most three total tool calls;
- strictly sequential execution in provider order;
- at most 4 KiB raw JSON arguments per call;
- at most 64 KiB serialized tool result per call;
- at most 16 KiB final answer.

Each provider turn yields either a nonblank final answer with no tool calls, or function calls that fit the remaining budget. Mixed text plus calls treats text as nonterminal provider data. Duplicate/blank call IDs, duplicate object members, unknown/missing arguments, non-function types, unknown tools and budget overflow fail before the affected product operation. Counters never reset inside the request.

Tool output is bounded typed M10 data wrapped as untrusted provider input. It cannot add tools, messages or policy. Failure to produce a valid final answer within the turn budget is explicit failure, not partial success.

## 6. Exact tool map

### `affairs_navigator_get`

Input is exactly `{"procedure_id":"proc:ustc:undergraduate:transcript-certificate"}`. The bridge invokes only the existing public-redacted Affairs query. Any non-`Available/Public/Found` outcome is denied or failed rather than model-authored procedure truth.

### `change_radar_get`

Input is exactly `{"board_id":"board:ustc:academic-calendar"}`. The bridge invokes only the existing typed ChangeRadar query. No administrator publication operation is model-visible.

### `opportunity_graph_plan_current_profile`

Input is exactly `{}`. This definition is omitted unless the exact request has both a valid `opportunity_context` and the confirmation header. Composition inserts the profile ID out of band and invokes the existing static `GeneratePlan` operation with `max_results=3` and `beam_width=1024`.

The model cannot create, view, edit, consent to, revoke or delete a profile; choose a different profile ID; or add courses outside deterministic planner output. A stale, missing, disabled, revoked, cross-principal or otherwise denied current profile returns a bounded non-success tool result.

## 7. Web, Compose and package projection

The thin static browser owns only page-lifetime draft/history presentation. It sends bounded user/assistant history, renders loading/final/error/tool-trace states, and includes Opportunity context only after profile creation plus explicit checkbox confirmation. It never receives or stores the provider key. Keyboard submit, visible focus, reduced motion and 390 px/mobile-to-desktop layout remain required.

The Compose package:

- publishes only `127.0.0.1:${UCA_MVP_PORT}:8787`;
- defaults to deterministic mock with no provider network call;
- mounts a provider key file read-only and uses only a non-secret placeholder in mock mode;
- persists product state in a named volume across `stop`/restart;
- deletes that volume only through explicit reset;
- packages deterministic ZIP/tar archives with exact source commit, per-file checksums and a provider-secret scan;
- keeps `.ps1`/`.cmd` launchers ASCII-only, BOM/NUL-free and LF-terminated for Windows PowerShell 5.1, with native-command `$LASTEXITCODE` checks;
- runs smoke verification under a unique Compose project so cleanup cannot address a user's normal MVP volume.

## 8. Non-goals

This version does not claim real campus sources, CAS/SSO, multi-tenancy, generic Plugin installation/execution, provider fallback, streaming, RAG, durable chat history, long-term memory, parallel tools, multi-agent graphs, Dioxus parity or production hosting. Real-provider smoke remains `not-run` unless an operator separately supplies runtime configuration and grants provider-network permission.
