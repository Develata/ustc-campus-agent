# Bounded Web Chat API and delivery contract

## Metadata

- `Status`: Implemented bounded MVP contract
- `Version`: `agent-chat/v1` response/error with `ustc-agent-chat-request/v1` and `ustc-agent-chat-request/v2`
- `Last Review`: `2026-09-05`
- `Owning Plan`: [`../plan/07-runtime-and-integration.md`](../plan/07-runtime-and-integration.md)
- `Feature Projection`: [`../features/04-bounded-agent-harness.md`](../features/04-bounded-agent-harness.md)
- `Acceptance`: active `CHAT-001`, `CHAT-002`, `CHAT-003`
- `Primary Code`: `apps/ustc-agentd/src/agent_chat.rs`, `chat_provider.rs`, `chat_tools.rs`, `web.rs`

## 1. Scope and authority

`agent-chat/v1` is one loopback-only competition profile over a fixed reviewed demo catalogue. It is not the complete durable [`agent-harness/v0`](agent-harness.md), a generalized package lifecycle/execution runtime, a provider-managed conversation, or production campus-source activation.

```text
Composition-owned static Web Chat shell (not M80 module evidence)
→ M10 POST /api/v1/agent/chat
→ M30 bounded in-memory coordinator
→ M50 deterministic mock or operator-configured OpenAI-compatible adapter
→ exact sequential validated-catalogue tool bridge
→ existing Affairs / ChangeRadar fixed read path
   or owner-local Simple Calendar in-process companion
   or existing consent-bound static M72 planner path
→ natural-language answer + redacted tool trace
```

The server owns request validation, provider selection, budgets, the fixed tool catalogue and product composition. The browser and provider mint no route, tenant, user, grant, profile, source, publication, mutation confirmation or administrator authority. The three campus-data tools use only reviewed repository fixtures; the planner fixture includes bounded public iCourse aggregate-rating link-outs as orientation-level soft evidence but no copied review text. The Calendar companion uses only owner-local state. This contract grants no dynamic install/disable/revoke-driven provider projection, isolated third-party execution, USTC network access or real-source activation permission.

### Saved dialogue composition

The separately versioned [conversation contract](chat-conversations.md) wraps this
finite coordinator with server-owned history and durable turn submission. The legacy
`/api/v1/agent/chat` remains stateless and compatible; its callers cannot infer durable
retry guarantees. The saved-dialogue routes accept a new user message, not arbitrary
client-authored history. Profile consent and response preferences remain request-only.

## 2. HTTP request

`POST /api/v1/agent/chat` accepts `application/json` only, under the router's 16 KiB complete-body limit. The loopback router requires a `Host` authority whose host is `localhost` or a numeric loopback IP (`127.0.0.0/8` or `::1`) and contains no userinfo; when `Origin` is present it must be the same HTTP authority. Invalid Host or cross-origin requests fail before provider or tool I/O.

```json
{
  "schema": "ustc-agent-chat-request/v1",
  "messages": [
    {"role": "user", "content": "成绩单证明怎么办？"}
  ],
  "opportunity_context": null
}
```

The v1 request is closed:

- `schema` is exactly `ustc-agent-chat-request/v1`;
- `messages` contains 1–12 entries;
- each entry has only `role` and `content`;
- `role` is exactly `user | assistant`; browser/provider callers cannot submit `system` or `tool` history;
- each UTF-8 `content` is nonblank, contains no U+0000/NUL scalar, and is at most 4 KiB; aggregate message content is at most 12 KiB; the final role is `user`;
- `opportunity_context` is absent/null unless the browser has an existing profile hint and the user enables the explicit per-request chat-use control;
- a non-null `opportunity_context` is exactly `{"profile_snapshot_id":"..."}`; its value is nonblank, contains no U+0000/NUL scalar and is at most 4 KiB; scalar aliases and unknown fields fail;
- a non-null context additionally requires `X-USTC-Opportunity-Confirmation: confirmed` on the same request.

The profile snapshot ID and header are non-authoritative hints. The browser applies the same nonblank/NUL/4 KiB bound before enabling a restored `localStorage` hint and removes an invalid persisted value. The server still checks current session, tenant/user ownership, consent, Market state and source currentness through the existing Opportunity composition.

### 2.1 Request v2 prompt customization

`ustc-agent-chat-request/v2` keeps every v1 field, bound and admission rule and may additionally carry:

```json
{
  "schema": "ustc-agent-chat-request/v2",
  "messages": [
    {"role": "user", "content": "成绩单证明怎么办？"}
  ],
  "opportunity_context": null,
  "prompt_customization": {"text": "请用简洁的要点回答。"}
}
```

The v2 request and `prompt_customization` object are closed. The field may be absent or null; when non-null it contains exactly one string field, `text`. The original UTF-8 string is at most 2048 bytes and must remain nonblank after outer-whitespace trim. U+0009, U+000A and U+000D are the only admitted control whitespace; every other Unicode control (`Cc`), every Unicode format scalar (`Cf`, including bidi controls, zero-width controls and U+FEFF/BOM), and U+0000/NUL are rejected. Validation projects only the trimmed text. Supplying `prompt_customization` under request schema v1, using request v2 with another unknown field, or using a scalar/array/open customization object fails as `invalid_chat_request` before provider or tool I/O.

The validated text is a user-scoped, untrusted, request-only preference—not an editable system or developer policy. The server places the immutable system policy first, then projects the preference as a separately and explicitly labelled user message before ordinary conversation history. It cannot select or add a model, provider, endpoint, role, tool, route, grant, identity, credential or authority; the server-owned tool definitions and count are unchanged. The original field counts toward the 16 KiB HTTP body limit, and the complete labelled trimmed projection counts toward the existing provider context-budget preflight. It is not written to chat history, local or durable state, logs, traces, profile storage or a later request.

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

## 4. Provider profile and transport

Runtime configuration is server-only:

| Key | Contract |
|---|---|
| `UCA_AGENT_PROVIDER` | exact `mock` (default), `openai-compatible`, or explicit native `local-chat` testing (§4.1) |
| `UCA_AGENT_BASE_URL` | required for network profiles; normal Agent profile uses absolute HTTPS, no userinfo/query/fragment; fixed join with `chat/completions` |
| `UCA_AGENT_MODEL` | required for network profiles; bounded nonblank model ID |
| `UCA_AGENT_API_KEY_FILE` | required for network profiles; regular non-symlink file read once at server startup |
| `UCA_AGENT_TIMEOUT_MS` | integer 1000–60000; default 15000 |
| `UCA_AGENT_CONTEXT_TOKENS` | required for network profiles; 16384–1048576 for normal Agent, 1024–1048576 for `local-chat` (§4.1) |

The packaged launchers require `.env` itself to be a readable regular non-symlink file when present, and require at most one exact column-zero `KEY=value` assignment for each of `UCA_AGENT_PROVIDER` and `UCA_AGENT_API_KEY_SOURCE`. Their values must be literal: the launchers reject all `$`-based Compose interpolation in either security-critical `.env` assignment before Docker, including an otherwise-unused key-source assignment in mock mode, so launcher-side security preflight cannot observe a value different from the Compose-resolved service. Operators needing dynamic configuration inject already-resolved literal process-environment values instead.

The key file is UTF-8, nonblank after outer-whitespace trim and at most 4096 bytes. On Unix, the opened key file must have no group/world permission bits (`mode & 077 == 0`). Because local Compose file-backed secrets preserve host ownership, Compose first drops every capability and then grants the root-only initialization phase exactly `CHOWN`, `DAC_OVERRIDE`, `FOWNER`, `SETGID`, `SETPCAP` and `SETUID`: the entrypoint can read the explicitly mounted owner-only source, copy it into an ephemeral mode-0600 tmpfs file owned by UID/GID 65532, and then re-exec itself through `setpriv` as UID/GID 65532 with cleared groups, no-new-privileges and an empty effective/bounding capability set before the daemon or proxy starts. The packaged Unix launcher enforces the same permission rule on the host source before Docker runs, while direct Compose use remains operator-responsible because the projected container secret cannot prove the host file's original mode. Both launchers, the container entrypoint and the authoritative Rust key reader reject the bundled mock placeholder after the same outer-whitespace normalization in `openai-compatible` mode. The normal runtime accepts no raw key through argv, HTTP, browser storage, checked-in environment or logs. Invalid OpenAI-compatible configuration fails startup without fallback to mock, another origin or another model.

The legacy complete/title adapter sends non-streaming Chat Completions with the exact configured model, ordered complete messages, complete current tool definitions, `tool_choice: auto`, `parallel_tool_calls: false`, `stream: false` and an 8192-token output ceiling. For request v2, the immutable server system policy remains the first message and the separately labelled untrusted preference follows it without changing the tool set. A retained test exercises this adapter through the complete loopback `POST /api/v1/agent/chat` route against a bounded local provider peer, including provider identity, usage and hardened-response projection. The response path accepts only exactly one `assistant` choice, requires `finish_reason: stop` for final text or `finish_reason: tool_calls` for a complete tool batch, and rejects truncated, content-filtered or mismatched termination before any tool execution. It follows no redirects, uses one absolute timeout and accepts at most 256 KiB of response bytes. The normal `openai-compatible` configuration requires HTTPS; plain HTTP is admitted only by the test constructor or the separately selected numeric-loopback `local-chat` profile (§4.1). The deterministic mock is network-free and routes only product-qualified transcript, academic-calendar, course-planning and Calendar terms. Academic-calendar wording alone does not select the personal Calendar tool, but a mixed request with an explicit personal-calendar list clause retains both read-only tools. For each known successful tool shape it projects only bounded user-facing fields into a server-owned Chinese summary: procedure steps and official entry points, semantic changed fields and source link, course candidates/rationale/iCourse link-outs, or Calendar mutation/list details. Known-tool shape drift yields an explicit summary-contract notice instead of a raw JSON dump. Fair per-result output budgets ensure one large result cannot erase later successful tools; denied and failed statuses remain explicit non-success answers, and a mixed request whose Opportunity tool is unavailable retains an explicit unexecuted-consent notice beside any successful public-tool summary.

Before network I/O the adapter serializes the complete wire request and applies `T(q) + O + S ≤ floor(L × 0.9)`, where `T(q)` is conservatively upper-bounded by serialized UTF-8 bytes, `O=8192`, `S=2048`, and `L=UCA_AGENT_CONTEXT_TOKENS`. Oversize input fails locally as `context_budget_exceeded`; no provider/profile context limit means no OpenAI-compatible call.

A successful provider message must carry the exact `assistant` role and either nonblank final text or function calls. Missing/non-assistant roles, malformed JSON, empty/multiple choices, invalid call objects and oversized output map to `provider_protocol_error`. HTTP 401/403, 429, timeout and remaining non-success transport classes map to their stable errors without returning the raw body. The deterministic mock derives its wording from server-owned tool status/data. An operator-selected real provider remains an untrusted text generator: the server preserves the independently rendered tool-trace status but cannot prove that arbitrary provider prose describes a denied/failed result honestly; operators must treat the trace as authoritative.

### 4.1 Explicit local chat test profile

The operator may select `UCA_AGENT_PROVIDER=local-chat` to exercise the real Chat Completions path using a small local model without function-calling support. This is an explicit testing profile, never an automatic fallback or full Agent qualification. `UCA_AGENT_BASE_URL`, `UCA_AGENT_MODEL`, `UCA_AGENT_API_KEY_FILE` and the bounded timeout remain server configuration. The base must be absolute HTTP on a numeric loopback IP, without userinfo/query/fragment; hostname, private-network and remote HTTP targets are rejected, redirects remain forbidden and this profile bypasses environment proxies. The ordinary `openai-compatible` profile retains its HTTPS requirement and original budgets.

For `local-chat`, `UCA_AGENT_CONTEXT_TOKENS` is required in 1024–1048576, the fixed output reserve is 256 tokens and estimator reserve is 256 tokens; the same complete-wire UTF-8-byte bound and 90% ceiling apply before I/O. Oversize history fails explicitly and is not truncated, given a fictitious context limit or silently sent. The coordinator keeps immutable policy, adds an explicit tools-unavailable notice, projects no tools and rejects any returned function call before any executor. No profile-use control can enable tools for this profile. Successful text uses the existing response schema and provider identity mode `local-chat`; it does not prove plugin execution.

`GET /api/v1/agent/status` is a no-store, loopback-admitted configuration projection with exactly `schema=ustc-agent-provider-status/v1`, `provider={mode,model}`, `tool_calling` (boolean), and `context_limit_tokens` (integer, or null for mock). It exposes no endpoint, credential, path, headers or private data and performs no provider I/O. `tool_calling` describes the operator-selected profile, not a probed model capability or installed-package grant; `openai-compatible` requires an operator-selected tool-capable model. The UI says configuration is loaded, and only a successful Chat response establishes an observed connection. Offline deterministic mock, real local chat testing, configured remote-compatible provider, and unavailable configuration states remain visibly distinct.

## 5. Bounded sequential loop

One accepted request creates one finite in-memory `ChatRun` and pins:

- at most three provider turns;
- at most four total tool calls;
- strictly sequential execution in provider order;
- at most 4 KiB raw JSON arguments per call;
- at most 64 KiB serialized tool result per call;
- at most 16 KiB final answer.

Each provider turn yields either a nonblank final answer with no tool calls, or function calls that fit the remaining budget. Mixed text plus calls treats text as nonterminal provider data. Duplicate/blank call IDs, duplicate object members, unknown/missing arguments, non-function types, unknown tools and budget overflow fail before the affected product operation. Counters never reset inside the request.

Tool output is bounded typed M10 data wrapped as untrusted provider input. It cannot add tools, messages or policy. Failure to produce a valid final answer within the turn budget is explicit failure, not partial success.

Calendar mutation authority is captured once from the final admitted user message before the first provider call. Every validated Calendar `record` or `delete` proposal is compared with that immutable intent before `executor.execute`. Absent or mismatched intent produces a bounded typed denied tool result and `denied` trace entry, performs zero executor/store operation, and may be returned to the provider within the existing turn budget. Provider prose, a provider argument, or a model-authored confirmation cannot create or widen this authority. Complete-batch validation still finishes before any tool in the batch executes. One admitted request authorizes at most one matching Calendar mutation attempt. The coordinator consumes that request-local intent before invoking the executor, including when execution fails or its result cannot be projected; a later matching mutation in the same batch or a later provider turn yields a denied result with `calendar_mutation_intent_consumed` and performs no executor/store operation. Mismatched proposals do not consume the intent, and read-only calls remain available. This is in-memory duplicate suppression within one run, not durable idempotency across HTTP retries or restarts.

## 6. Exact tool map

### `affairs_navigator_get`

Input is exactly `{"procedure_id":"proc:ustc:undergraduate:transcript-certificate"}`. The bridge invokes only the existing public-redacted Affairs query. Any non-`Available/Public/Found` outcome is denied or failed rather than model-authored procedure truth.

### `change_radar_get`

Input is exactly `{"board_id":"board:ustc:academic-calendar"}`. The bridge invokes only the existing typed ChangeRadar query. No administrator publication operation is model-visible.

### `simple_calendar_items`

Input is a closed object with exact `action = record | list | delete | propose`. `record` requires only a nonblank title of at most 256 UTF-8 bytes; top-level `scheduled_for` remains rejected for legacy record. `propose` requires only a nested closed `mutation` object as specified by [Calendar proposals](calendar-proposals.md), which can carry an explicit-offset scheduled time and cannot execute an effect. `list` accepts no other field and remains read-only. `delete` requires one stable `calendar:item:N` ID. Rust revalidates the complete action-specific shape before execution.

The `list` tool result projects every item as its exact `id`, `title` and optional
non-null `scheduled_for`. It omits creation timestamps and null scheduling fields,
so a valid near-64-KiB durable Calendar store still fits the 64-KiB complete Chat tool
result budget. This projection neither truncates the item list nor rewrites durable
records. The `record` and `delete` result item shapes are unchanged; deterministic
list summaries accept the optional scheduling field without requiring timestamps.

The only admitted mutation grammars in the final user message are:

- record: exact prefix `记录事项：` or `记录事项:` followed by a nonblank title; the normalized title is the suffix after outer-whitespace trim and must equal the provider-call title byte-for-byte;
- delete: exact prefix `删除事项 ` followed by one complete `calendar:item:N`, with no hidden or extra suffix; the ID must equal the provider-call ID byte-for-byte.

A mere occurrence of `日历`, `待办`, `事项`, `calendar`, `reminder`, or similar help text creates no mutation intent. The deterministic mock uses the same grammar. The loopback profile persists at most 128 owner-local items and returns success only after a mutation is durably written. It has no reminder, recurrence, sharing, synchronization or natural-language time semantics.

### `opportunity_graph_plan_current_profile`

Input is exactly `{}`. This definition is omitted unless the exact request has both a valid `opportunity_context` and the confirmation header. Composition inserts the profile ID out of band and invokes the existing static `GeneratePlan` operation with `max_results=3` and `beam_width=1024`.

The model cannot create, view, edit, consent to, revoke or delete a profile; choose a different profile ID; or add courses outside deterministic planner output. A stale, missing, disabled, revoked, cross-principal or otherwise denied current profile returns a bounded non-success tool result.

## 7. Web, Compose and package projection

The approved Chat-first presentation uses one conversation as the default view, with a narrow rail for new conversation, Plugins and settings. The Plugins directory explains the fixed capabilities available to the Agent, their use conditions and data boundaries, with a primary action to ask the Agent. Detail views serve profile consent, source inspection and result verification; it does not imply package installation, enablement or new tools. Guided prompts in those views only fill a draft. Tool traces and source details are disclosed on demand. Administrator demo controls remain separate inside settings, organized into procedure publication and calendar-change publication with human-readable state, explicit confirmation and collapsed technical receipts. They are never Agent tools. Navigation or expansion issues no publication command; only the corresponding confirmed publish button can do so. View changes preserve drafts and in-flight work; browser back/forward restores the view, and mobile navigation supports keyboard focus and dismissal. New conversation opens a separate saved dialogue and preserves earlier conversations and product state. The sidebar reads the server-owned history list.

Assistant text may use a bounded Markdown subset (paragraphs, headings, lists, quotes, fenced code, emphasis and HTTP(S) links). Construct DOM nodes without interpreting HTML; reject credential-bearing or non-HTTP(S) links, and keep code/raw HTML inert. Model prose is not verified source evidence. Theme preference remains a browser-only setting. Conversation text is saved at the server through the separate conversation contract; per-request consent/preferences are never persisted as reusable authority or prompt profiles. If a displayed answer exceeds the per-history-message limit, discard the earlier sendable history as well instead of silently joining a follow-up to an older topic; display a context-boundary notice without clipping the answer. When a request fails while the user has composed another draft, retain the original question in the bounded page transcript with an explicit no-answer marker, exclude that failed turn from sendable history, and preserve the newer draft. Error recovery for a Calendar mutation must advise checking current items before resubmitting, since an error response does not prove that no effect occurred.

The thin static browser owns draft/history presentation and submits one new user
message through the saved-dialogue API. It renders loading/final/error/tool-trace
states and includes Opportunity context only after explicit checkbox confirmation.
The server builds bounded history; the legacy stateless endpoint still accepts v1/v2
for compatible callers. Prompt customization remains request-only: blank input omits
it, success clears it, and an uncertain request preserves its exact body for deliberate
same-identity retry. It never becomes a reusable preference in history or localStorage.
The browser never receives or stores the provider key. Keyboard submit, visible focus,
reduced motion and 390 px/mobile-to-desktop layout remain required.

The Compose package:

- publishes only `127.0.0.1:${UCA_MVP_PORT}:8787`;
- defaults to deterministic mock with no provider network call;
- mounts a provider key source read-only, copies it only in OpenAI-compatible mode into an ephemeral mode-0600 tmpfs file, uses only a non-secret placeholder in mock mode and rejects that placeholder in real-provider mode;
- treats `idempotency_path.with_extension("calendar-items.json")` as a member of the locked durable state set: fresh bootstrap writes a canonical empty mode-0600 store, non-fresh absence fails `durable_state_set_incomplete`, rollback removes it with every other newly created member, and restart retains committed Calendar items;
- persists the complete product state set in a named volume across `stop`/restart;
- deletes that volume only through explicit reset;
- packages deterministic ZIP/tar archives with exact source commit, per-file checksums and a provider-secret scan;
- installs repository-root `LICENSE.md` byte-for-byte as package-root `LICENSE.md` with mode 0644, includes it in `SHA256SUMS`, and verifies the same bytes/checksum in both archives;
- keeps `.ps1`/`.cmd` launchers ASCII-only, BOM/NUL-free and LF-terminated for Windows PowerShell 5.1, with native-command `$LASTEXITCODE` checks; the port query drains native stdout and snapshots its exit code before selecting the first buffered line, so early pipeline termination cannot masquerade as a Docker failure; real nonzero exit and empty/invalid port output remain failures;
- runs smoke verification under a unique Compose project so cleanup cannot address a user's normal MVP volume.

## 8. Non-goals

This version does not claim live campus-source ingestion, CAS/SSO, multi-tenancy, generalized package installation/disable/revoke-driven tool projection or isolated execution, a Skill runtime, an MCP adapter/server, a command sandbox, editable system policy, persisted prompt profiles, provider fallback, streaming, RAG, durable chat history, long-term memory, reminders, calendar synchronization, parallel tools, multi-agent graphs, shared-client parity, production Android or production hosting. Real-provider smoke remains `not-run` unless an operator separately supplies runtime configuration and grants provider-network permission.

## Package-owned tool extension — PLUGIN-001

The existing four product definitions remain available under their original rules.
An application-owned frozen Plugin session may add at most 28 checked namespaced
definitions (provider total at most 32). Names and provider JSON schemas are derived
from the platform's canonical schema; complete batches still validate before any
executor. The execution callback is asynchronous and existing synchronous product
callbacks adapt through ready futures. There is no nested runtime blocking call.

Plugin endpoints, credentials, manifests and authorization handles never enter the
Agent catalog. The application holds the exact installation revision, grant snapshot
and version, readiness and tool/schema identity associated with each projected tool.
A changed binding/grant during a provider turn rejects the old call rather than
switching it to the newly enabled tool. M20 resolver/recheck and M30 durable
effect-intent/receipt order remain required. Skill body/reference reads are bounded
untrusted tool context, not new system policy.

Public trace and activity expose the fixed category `plugin_tool`, never provider
chosen names or package metadata. The local-chat profile still projects no tools;
a local model text response does not establish MCP or Skill execution evidence.
See [plugin-management.md](plugin-management.md) for the admitted application profile.

## Explicit model selection

[MODEL-001](model-selection.md) adds the closed request v3 with required `model_id`
and an immutable server-configured catalog. Older request schemas keep their
default-provider behavior and reject model selection fields. Browser input chooses
only an admitted catalog ID, never endpoint, credential, model wire name or budgets.
Prompt customization remains untrusted preference and cannot perform selection.
The provider returned in a response identifies the actual selected execution.

## Final response within the existing budget

On the final permitted provider turn, or after the tool-call budget is spent, the
request carries no callable tool definitions and adds a server-owned instruction
to answer from existing evidence. The original immutable first system policy is
unchanged. A partial resource read must be described honestly, including its
next_offset for deliberate continuation; unread content must not be claimed as
read. This does not add a turn, a tool call, an automatic continuation or a grant.
Unexpected tool calls beyond the budget still reject under the existing error codes.

## Calendar proposal extension

[CALENDAR-PROPOSAL-001](calendar-proposals.md) adds `action=propose` with a closed
mutation object to the existing Calendar tool; it does not add model-confirm authority.
Legacy exact record/delete intents and the four-tool baseline retain their meaning.
Calendar list supplies a server clock and explicit campus timezone. The proposal result
is labelled pending, distinct from item execution and reminder delivery.


### Personal root prompt extension

[ROOT-PROMPT-001](agent-root-prompt.md) adds a separate server-persisted personal
instruction for new saved-conversation turns. The request-only preference described
above stays request-only. Personal instructions precede it and do not replace platform
system policy, tool admission or explicit effect confirmation. This supersedes the
historical “no persisted prompt profiles” exclusion only for this bounded setting.


### Streaming and execution stop extension

[STREAM-CANCEL-001](chat-activity.md#stream-cancel-001-extension) supersedes the historical
streaming/cancellation exclusion for saved bounded Chat runs. Runs request SSE and
emit actual content deltas; complete JSON remains compatible. Tool argument deltas
cannot bypass complete-batch validation. Stop is an owner-admitted application
command, never a model tool or a model-authored confirmation.
