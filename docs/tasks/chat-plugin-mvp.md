# Chat + first-party Plugin MVP delivery taskbook

## Status

- `Lifecycle`: accepted implementation brief
- `Requested by`: Develata
- `Accepted at`: 2026-09-03
- `Base`: `origin/main` at `54d758fbf2f1c08df2e1993919287569b501b115`
- `Branch`: `feat/chat-plugin-mvp`
- `Integration owner`: Hermes/default
- `Owning plans`: [`M30`](../plan/modules/40-agent-harness-runtime.md), [`M50`](../plan/modules/60-model-provider-integration.md), [`M71`](../plan/modules/72-affairs-navigator.md), [`M72`](../plan/modules/73-opportunity-graph.md)
- `Owning feature`: [`bounded-agent-harness`](../features/04-bounded-agent-harness.md)
- `Owning registry`: [`interfaces.md`](../contracts/interfaces.md)

This taskbook schedules an explicitly narrow MVP. It does not replace the owning plans or expand source, identity, Market, Plugin, provider, or external-effect authority.

## 1. Product spine

The MVP proves exactly three user journeys from the existing loopback Web shell:

1. **Basic AI chat** — one bounded, non-streaming, stateless user message reaches one configured OpenAI-compatible Responses API provider and returns one assistant text result.
2. **Campus lookup through a Plugin** — the model may call the fixed `ustc_affairs_lookup` tool; the host validates its arguments and invokes the existing Affairs application/composition path before returning the typed result to the model.
3. **Course advice through a Plugin** — only after explicit per-request user consent, the model may call the fixed `ustc_course_advice` tool; the host validates bounded academic facts, invokes the existing M72 create-profile → generate-plan → revoke-delete application path, and gives the grounded typed plan to the model. The operation never enrolls, drops, pays, submits, or writes to a campus system.

A provider may request at most one tool round. The host may execute at most one supported tool call in that round and then performs one final provider call. Unknown tools, malformed arguments, unavailable provider configuration, provider errors, or typed Plugin denials produce explicit non-success; there is no model, tool, source, or direct-domain fallback.

## 2. Frozen MVP surface

### 2.1 HTTP

`POST /api/v1/agent/chat`

Request:

```json
{
  "message": "string, trimmed UTF-8, 1..=8192 bytes",
  "course_profile_consent": false
}
```

Response on success:

```json
{
  "answer": "assistant text",
  "model": "configured model identifier",
  "used_tools": ["ustc_affairs_lookup"],
  "grounded": true
}
```

`grounded` is true only when at least one supported Plugin tool completed and its typed output was returned to the provider. It does not mean the provider's prose is independently authoritative. Provider text is explanatory; typed Plugin output remains the authority.

The route is a bounded loopback demo projection and requires protocol major `1`, matching the existing Web shell. It is not a production compatibility promise.

### 2.2 Provider profile

The only production adapter in this slice uses the OpenAI-compatible Responses API:

- `USTC_AGENT_MODEL_BASE_URL` — absolute HTTPS origin; `http://127.0.0.1` and `http://[::1]` are accepted only for local conformance tests;
- `USTC_AGENT_MODEL_API_KEY` — process environment secret; never serialized or logged;
- `USTC_AGENT_MODEL` — exact model identifier;
- `USTC_AGENT_MODEL_TIMEOUT_SECS` — optional, default `30`, bounded to `1..=120`.

The adapter sends `store: false`, one developer instruction, one user message, two strict function definitions, and a bounded output-token limit. It accepts only completed message/function-call outputs needed by this slice. Redirect following is disabled. This is server-environment MVP configuration, not the planned durable `OfficialCentral` or encrypted `UserCloud` profile store.

### 2.3 Tool arguments

`ustc_affairs_lookup`:

```json
{"procedure_id":"proc-011"}
```

`ustc_course_advice`:

```json
{
  "completed_courses":["MATH1001"],
  "min_credits":6,
  "max_credits":12,
  "preference_weights":[{"course_code":"MATH2001","weight":5}]
}
```

Course tool bounds are exactly the existing M72 wire bounds. `course_profile_consent=false` denies the tool before profile creation. On success or later failure after creation, the host attempts the owning revoke-delete operation before returning; a cleanup failure is an explicit request failure, not hidden success.

## 3. Implementation ownership

| Lane | Writable paths | Deliverable |
|---|---|---|
| Contract/integration owner | `docs/**`, root manifests/lock, `apps/ustc-agentd/src/lib.rs`, final fan-in/tests/scripts | frozen surface, integration, acceptance and final evidence |
| Runtime/provider | `crates/agent-runtime/**`, `crates/adapters/**` | provider-neutral two-step chat loop, fake provider, Responses adapter |
| Plugin composition/API | `apps/ustc-agentd/src/chat.rs`, `apps/ustc-agentd/src/web.rs`, `apps/ustc-agentd/src/main.rs`, focused Rust tests | typed chat endpoint and existing Affairs/M72 calls |
| Web presentation | `apps/ustc-agentd/src/web/index.html`, `app.js`, `styles.css` | one thin chat surface with explicit course-profile consent |

Only the integration owner may edit shared manifests, `Cargo.lock`, acceptance/coverage/roadmap carriers, or fan in lanes.

## 4. Acceptance package

The delivered test bundle contains:

- unit tests for request bounds, provider output normalization, tool-round limits, unknown/malformed tool calls, and no secret rendering;
- fake-provider integration tests for chat-only, Affairs tool, course-advice tool, missing course consent, and Plugin/typed denial;
- a black-box loopback HTTP test for all three user journeys using a deterministic local fake Responses server;
- `scripts/run_chat_plugin_mvp.sh`, which runs the focused deterministic package without external credentials;
- an optional credentialed live smoke that runs only when all three provider environment variables are present and reports `not-run` otherwise.

Canonical gates:

```bash
cargo fmt --all -- --check
cargo test --locked -p ustc-campus-agent-runtime
cargo test --locked -p ustc-campus-agent-adapters
cargo test --locked -p ustc-agentd --test chat_plugin_mvp
cargo test --locked -p ustc-agentd --test affairs_web
python3 scripts/check_repo_contracts.py
bash scripts/run_chat_plugin_mvp.sh
```

## 5. Explicit cuts and non-claims

This slice deliberately excludes conversation persistence, multi-turn history, streaming/SSE, attachments, RAG, arbitrary tool discovery, parallel tool calls, multiple providers, retry/fallback, MCP, production SSO, non-loopback exposure, live USTC retrieval, source approval, enrollment effects, Android/Desktop clients, and production secret/profile storage.

Existing `DemoReviewed` Affairs and course-planning fixtures remain visibly non-authoritative. Model prose cannot create source truth, course eligibility, or enrollment authority. PR #66 and every M60 R21/R61 object are outside this branch and must not be modified, accepted, merged, or used to imply source/network authority.
