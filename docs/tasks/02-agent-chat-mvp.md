# Bounded Web Chat MVP

## Metadata

- `Status`: current implementation contract
- `Date`: 2026-09-03
- `Scope`: one loopback Web Chat vertical slice over the three existing DemoReviewed/synthetic Plugin journeys
- `Modules`: M10 composition, M30 bounded chat coordination, M40 existing fixed adapters, M50 OpenAI-compatible adapter/fake, M70/M71/M72 owning product paths, M80 static thin Web, M90 Compose
- `Decision provenance`: Develata selected “Web Chat + OpenAI-compatible provider + three-Plugin tool calling”; real campus-source access remains forbidden

This taskbook selects one bounded implementation from the accepted plans. It does not redefine product topology, grant source/network authority, or promote any large module beyond the evidence actually produced.

## 1. Product completion path

```text
Web Chat
→ POST /api/v1/agent/chat
→ deterministic mock or explicitly configured OpenAI-compatible /chat/completions
→ at most three sequential provider turns and three tool calls
→ Affairs / ChangeRadar existing M10→M30/M40 fixed read path
   or Opportunity existing M10→M20→static M72 plan path
→ bounded tool result
→ final natural-language answer + safe tool trace
```

The default mode is deterministic and performs no provider network I/O. All three product paths continue to read only repository-reviewed offline fixtures; this slice adds no USTC DNS, socket, HTTP, retrieval, approval or source-status path.

The composition is an MVP integration profile, not proof of the complete durable HarnessRun, generic package-portable ToolGateway, streaming M50 exit gate or Dioxus M80 target. Affairs and ChangeRadar preserve their existing provider-free AgentRun/ToolGateway evidence inside the owning operation. Opportunity remains the existing consent-bound static M72 operation: the outer chat coordinator may offer a model-visible plan tool only after explicit per-request confirmation and a caller-supplied non-authoritative profile hint, but it does not reinterpret profile create/view/revoke-delete as M30/M40 operations.

## 2. HTTP contract

`POST /api/v1/agent/chat` accepts `application/json` only under the router's existing 16 KiB body limit.

```json
{
  "schema": "ustc-agent-chat-request/v1",
  "messages": [
    {"role": "user", "content": "成绩单证明怎么办？"}
  ],
  "opportunity_context": null
}
```

Rules:

- `schema` is exact; unknown fields fail.
- `messages` has 1–12 entries, each role is exactly `user | assistant`, each UTF-8 content value is nonblank and at most 4 KiB, total content is at most 12 KiB, and the final role is `user`.
- Browser/model callers cannot submit `system` or `tool` messages.
- `opportunity_context` is absent/null unless the browser has an existing profile hint and the user enables the explicit chat-use control. Its only field is `profile_snapshot_id`; unknown fields fail.
- Opportunity tool registration additionally requires `X-USTC-Opportunity-Confirmation: confirmed` on this exact request. The server still performs current authenticated-session, tenant/user, profile, consent, Market and source-currentness checks; the hint and header mint no authority.

Success:

```json
{
  "schema": "ustc-agent-chat-response/v1",
  "run_id": "chat-run:...",
  "answer": "...",
  "provider": {"mode": "mock", "model": "deterministic-mock-v1"},
  "tool_trace": [
    {
      "call_id": "call-1",
      "tool": "affairs_navigator_get",
      "status": "succeeded"
    }
  ],
  "usage": {"input_tokens": 0, "output_tokens": 0}
}
```

`answer` is nonblank and at most 16 KiB. `tool_trace` exposes only provider call ID, model-visible tool name and `succeeded | denied | failed`; it does not expose private route, package/grant internals, profile payload, API key, raw request headers or provider diagnostics. Usage is saturating summed provider-reported input/output tokens; the deterministic mock reports zero.

Errors use:

```json
{"schema":"ustc-agent-chat-error/v1","error":"stable_code"}
```

Stable codes include `invalid_chat_request`, `provider_not_configured`, `provider_unauthorized`, `provider_rate_limited`, `provider_timeout`, `provider_unavailable`, `provider_protocol_error`, `tool_call_rejected`, `tool_result_too_large`, `tool_budget_exhausted`, `turn_budget_exhausted`, `opportunity_confirmation_required`, `composition_unavailable` and `internal_chat_error`. No response includes raw provider body or secret-bearing URL/header data.

## 3. Provider profile and secret boundary

Runtime keys:

| Key | Contract |
|---|---|
| `UCA_AGENT_PROVIDER` | exact `mock` (default) or `openai-compatible` |
| `UCA_AGENT_BASE_URL` | required only for `openai-compatible`; absolute HTTPS URL with no userinfo/query/fragment; path is a fixed base joined with `chat/completions` |
| `UCA_AGENT_MODEL` | required only for `openai-compatible`; nonblank bounded model ID |
| `UCA_AGENT_API_KEY_FILE` | required only for `openai-compatible`; path to a regular non-symlink file, read server-side at startup |
| `UCA_AGENT_TIMEOUT_MS` | optional integer 1000–60000; default 15000 |

The normal runtime never accepts the raw key through argv, HTTP, browser storage, response, log or checked-in environment file. The key file is UTF-8, nonblank after outer whitespace trim and at most 4096 bytes. Missing/invalid configuration fails startup for `openai-compatible`; there is no fallback to mock or another provider/model. The HTTP client sends one bearer header to the fixed origin, follows no redirects, applies an absolute timeout and bounds the complete response body to 256 KiB. Plain HTTP is admitted only by a test-only loopback constructor and is unavailable through production environment configuration.

The adapter implements non-streaming OpenAI-compatible Chat Completions for this slice:

- exact configured model;
- ordered messages and complete current tool definitions;
- `tool_choice: auto` and `parallel_tool_calls: false`;
- assistant text or function tool calls;
- provider-reported prompt/completion usage when present;
- typed 401/403, 429, timeout, non-2xx, malformed/oversized response and empty-choice mapping.

Streaming, provider fallback, arbitrary headers, browser-supplied origins/models, multimodal input and provider-managed run state are deferred and not claimed.

## 4. Bounded loop

The server owns one finite in-memory `ChatRun` per accepted HTTP request. It pins one provider profile, three provider turns, three total tool calls, sequential execution, 4 KiB raw arguments per call, 64 KiB serialized tool result per call and 16 KiB final answer. Counters never reset inside the request.

Each provider turn yields exactly one of:

- a nonblank final assistant answer and no tool calls; or
- one or more function calls whose count fits the remaining budget and no authoritative completion claim.

Mixed final text plus tool calls treats the text as nonterminal provider data. Unknown names, duplicate/blank call IDs, duplicate object members, unknown/missing arguments, malformed JSON, non-function call types or budget overflow fail before any product operation. Calls are executed in provider order. No model-selected URL, route, profile ID, tenant/user ID, source ID or administrator operation is accepted.

Tool output is serialized typed M10 data and is wrapped as untrusted data for the next provider turn. Tool output cannot add tools, system messages or policy. If the provider does not return a valid nonblank final answer within the turn budget, the run fails explicitly.

## 5. Exact tool map

### `affairs_navigator_get`

Model input:

```json
{"procedure_id":"proc:ustc:undergraduate:transcript-certificate"}
```

The ID is exact and closed for this fixture profile. Composition invokes the existing public-redacted Affairs query path. Any non-`Available/Public/Found` result is denied/failed, never guessed or converted to model-authored procedure truth.

### `change_radar_get`

Model input:

```json
{"board_id":"board:ustc:academic-calendar"}
```

The ID is exact and closed. Composition invokes the existing ChangeRadar typed query path. No administrator publication operation is model-visible.

### `opportunity_graph_plan_current_profile`

Model input is exactly `{}`. The tool is omitted unless the request carries both exact confirmation and one profile hint. Composition inserts the profile ID out of band and invokes the existing `GeneratePlan` operation with `max_results=3` and `beam_width=1024`. The model cannot create, view, edit, consent to, revoke or delete a profile, cannot choose another profile ID, and cannot add courses past the deterministic planner result. Current denial is returned as a bounded denied result.

## 6. Web contract

Chat becomes the first page task. The existing three detailed panels remain available as source/profile diagnostics and explicit consent/publication controls. The thin browser:

- owns only draft/history presentation for the current page lifecycle;
- sends bounded user/assistant messages and never sends system/tool messages;
- shows loading, final answer, tool trace and stable recovery-oriented errors;
- adds the Opportunity tool only after the user has created a profile and explicitly enables “allow this chat request to use my current synthetic profile”;
- never receives or stores the provider key;
- preserves keyboard submit, visible focus, reduced motion, 390 px mobile and desktop behavior.

No durable conversation/session-history claim is made in this slice. Existing server-owned Plugin publication/profile state retains its current volume/restart semantics.

## 7. Acceptance bindings

The implementation is complete only when all are real PASS:

1. deterministic mock direct answer;
2. mock Affairs call then natural-language answer;
3. mock ChangeRadar call then answer;
4. Opportunity tool absent without context/confirmation;
5. Opportunity current plan call after explicit consent/profile/confirmation;
6. model cannot call profile create/view/revoke-delete or administrator publication;
7. unknown tool, malformed/duplicate arguments and duplicate call ID reach no product operation;
8. provider turn/tool/argument/result/answer/body budgets fail closed;
9. OpenAI-compatible cassette covers final text, tool call, usage, 401/403, 429, timeout, non-JSON, empty choices, redirect and oversize;
10. sentinel API key is absent from HTML/JS, endpoint success/errors, logs and review/package artifacts;
11. browser journey passes keyboard, mobile/desktop, focus, loading/error states and console checks;
12. Compose defaults to mock, clean-starts on host loopback, persists existing state across restart, and reset alone deletes the volume;
13. PowerShell 5.1 launcher remains ASCII/no-BOM/NUL/newline compliant;
14. no source fixture, SourceStatus or USTC network path changes.

A real provider smoke is conditional on a separately supplied runtime key/config and separate provider-network permission. Its absence does not fail deterministic MVP acceptance, but must be reported `not-run`, never `PASS`.

## 8. Ownership and fan-in

- Provider adapter owns only profile parsing, secret resolution, HTTP mapping and provider DTOs.
- Chat coordinator owns loop budgets, provider/tool ordering and safe response projection.
- Tool mapper owns exact visible-name/argument/product-operation mapping; it owns no product truth or grants.
- Web owns presentation and explicit Opportunity confirmation.
- `ustc-agentd` is the sole composition/fan-in owner for shared routing and Cargo wiring.
- `Cargo.lock`, shared module declarations, Compose, acceptance/status projections and final commit remain serialized under the integration owner.

No retained implementation expands beyond these paths without a new contract decision.
