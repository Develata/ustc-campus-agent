import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";
import vm from "node:vm";

const source = readFileSync(new URL("../../apps/ustc-agentd/src/web/conversations.js", import.meta.url), "utf8");
const app = readFileSync(new URL("../../apps/ustc-agentd/src/web/app.js", import.meta.url), "utf8");
const responseValidator = app.slice(app.indexOf("function validateChatResponse("), app.indexOf("function normalizeChatErrorCode("));
const settle = () => new Promise(resolve => setImmediate(resolve));

// Only DOM plumbing is replaced; the conversation state machine and answer validator are production code.
class Element {
  constructor(tag) { this.tag = tag; this.children = []; this.dataset = {}; this.hidden = false; this.listeners = new Map(); }
  append(...children) { this.children.push(...children); }
  replaceChildren(...children) { this.children = children; }
  setAttribute() {}
  addEventListener(name, callback) { this.listeners.set(name, callback); }
  querySelectorAll(selector) {
    return this.children.flatMap(child => [
      ...(selector === "button" && child.tag === "button" ? [child] : []),
      ...child.querySelectorAll(selector)
    ]);
  }
}

function validResponse() {
  return {schema: "ustc-agent-chat-response/v1", run_id: "chat-run:fixture", answer: "已记录合成事项。", tool_trace: []};
}
function completed(requestId, response = validResponse()) {
  return {request_id: requestId, user: "记录事项：合成任务", phase: "completed", response, error: null};
}
function detail(turns = [], revision = 0) {
  return {schema: "chat-conversation/v1", id: "fixture-conversation", title: "fixture", revision, turns};
}

async function fixture() {
  const root = new Element("root"), recovery = new Element("recovery");
  const state = {writes: [], availability: null, errors: [], rendered: [], recovered: [], gets: [], pendingId: null};
  const response = payload => ({ok: true, status: 200, json: async () => payload});
  const context = {
    window: {UcaConversationMenu: {mount: () => ({attach() {}, close() {}})}},
    document: {createElement: tag => new Element(tag)}, TextEncoder, AbortController, setTimeout, clearTimeout,
    CHAT_RESPONSE_SCHEMA: "ustc-agent-chat-response/v1", CHAT_MAX_ANSWER_BYTES: 16384, CHAT_MAX_TOOL_CALLS: 4,
    CHAT_TOOL_LABELS: {simple_calendar_items: "日历"}, CHAT_STATUS_LABELS: {succeeded: "成功", denied: "拒绝", failed: "失败"},
    utf8Length: value => new TextEncoder().encode(value).length,
    chatFailure: code => Object.assign(new Error(code), {code}),
    crypto: {randomUUID: (() => { let next = 0; return () => `fixture-${++next}`; })()},
    fetch: async (url, options) => {
      if (url.endsWith("/turns")) {
        state.writes.push({body: options.body, headers: {...options.headers}});
        const intent = JSON.parse(options.body); state.pendingId = intent.request_id;
        if (state.writes.length === 1) throw Error("controlled lost POST response");
        return response({schema: "chat-conversation-turn-result/v1", conversation_id: "fixture-conversation", revision: 2, turn: completed(intent.request_id)});
      }
      if (options.method === "POST") return response(detail());
      if (url.endsWith("/fixture-conversation")) {
        assert.ok(state.gets.length, "every recovery GET is explicitly controlled");
        return response(state.gets.shift());
      }
      return response({schema: "chat-conversation-list/v1", conversations: []});
    }
  };
  vm.createContext(context); vm.runInContext(responseValidator, context); vm.runInContext(source, context);
  const client = context.window.UcaConversations.mount(root, recovery, {
    busy() {}, availability: value => {state.availability = value;},
    render: value => {state.rendered.push(value);}, error: code => {state.errors.push(code);},
    validateResponse: context.validateChatResponse, recovered: value => {state.recovered.push(value);}
  });
  await settle();
  const intent = {message: "记录事项：合成任务", model_id: "selected-fixture", opportunity_context: {profile_snapshot_id: "profile:fixture"}};
  await assert.rejects(client.submit(intent, {"X-USTC-Opportunity-Confirmation": "confirmed"}), {code: "network_error"});
  return {client, state, recovery};
}

function assertStillUncertain({client, state, recovery}) {
  assert.equal(state.availability.canSend, false, "unverified result must not unlock a new write");
  assert.equal(state.availability.canSwitch, false, "retain original model/selection lock");
  assert.equal(recovery.hidden, false);
  assert.equal(client.newConversation(), false);
  assert.equal(state.writes.length, 1);
  assert.equal(state.recovered.length, 0);
  assert.equal(state.rendered.length, 0, "invalid snapshot must not replace the accepted transcript");
  assert.equal(state.errors.at(-1), "invalid_response");
}

test("malformed terminal response preserves uncertainty until a valid complete read", async () => {
  const value = await fixture();
  value.state.gets.push(detail([completed(value.state.pendingId, {schema: "wrong"})], 2));
  await value.client.recover();
  assertStillUncertain(value);
  value.state.gets.push(detail([completed(value.state.pendingId)], 2));
  await value.client.recover();
  assert.equal(value.state.availability.canSend, true);
  assert.equal(value.state.availability.canSwitch, true);
  assert.equal(value.recovery.hidden, true);
  assert.equal(value.state.recovered.length, 1);
  assert.equal(value.state.writes.length, 1, "read-back never resends the original write");
});

test("a malformed later historical turn also retains the original pending request", async () => {
  const value = await fixture();
  const invalidLater = validResponse();
  invalidLater.tool_trace = [{call_id: "later-call", tool: "unknown_tool", status: "succeeded"}];
  value.state.gets.push(detail([completed(value.state.pendingId), completed("later-request", invalidLater)], 4));
  await value.client.recover();
  assertStillUncertain(value);
  value.state.gets.push(detail([completed(value.state.pendingId), completed("later-request")], 4));
  await value.client.recover();
  assert.equal(value.state.recovered.length, 1);
  assert.equal(value.state.rendered.at(-1).turns.length, 2);
  assert.equal(value.state.writes.length, 1);
});

test("an explicit retry after malformed read preserves exact body, model and confirmation headers", async () => {
  const value = await fixture();
  const original = structuredClone(value.state.writes[0]);
  value.state.gets.push(detail([completed(value.state.pendingId, {})], 2));
  await value.client.recover();
  assertStillUncertain(value);
  value.state.gets.push(detail());
  await value.client.recover();
  const retry = value.recovery.querySelectorAll("button").find(button => button.id === "conversation-retry-turn");
  assert.ok(retry, "a valid read showing no admitted turn allows explicit exact retry");
  retry.listeners.get("click")();
  await settle();
  assert.equal(value.state.writes.length, 2);
  assert.deepEqual(value.state.writes[1], original);
  assert.equal(JSON.parse(original.body).model_id, "selected-fixture");
  assert.equal(original.headers["X-USTC-Opportunity-Confirmation"], "confirmed");
  assert.equal(value.state.recovered.length, 1);
  assert.equal(value.state.availability.canSend, true);
});
