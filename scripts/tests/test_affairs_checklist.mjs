#!/usr/bin/env node
// Dependency-free unit/fake-DOM tests. Real Rust/browser integration is separate.
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import vm from "node:vm";
import test from "node:test";

const source = readFileSync(new URL("../../apps/ustc-agentd/src/web/affairs-checklist.js", import.meta.url), "utf8");
const fixture = JSON.parse(readFileSync(new URL("../../fixtures/affairs/proc-011-reviewed.json", import.meta.url), "utf8"));
// Test adapter only: uses reviewed content but does NOT claim a server terminal.
function payload() {
  const verified = fixture.last_verified_at_secs * 1000;
  return { kind: "available", redaction: "public", terminal: {
    lineage: { kind: "verified", materialization_receipt_id: "receipt:test-only", evidence_set_digest: fixture.normalized_digest, revision_count: 1 },
    outcome: { kind: "found", as_of: fixture.now_ms, freshness: { kind: "fresh" }, view: {
      procedure_id: fixture.procedure_id, artifact_id: "artifact:test-only", title: fixture.title,
      audience_tags: fixture.audience_tags, board_id: "board:test-only", board_policy_version: 1,
      prerequisites: fixture.prerequisites.map(condition => ({ condition, source_subject: fixture.source_id })),
      ordered_steps: fixture.steps.map((instruction, ordinal) => ({ ordinal, instruction })),
      entry_points: structuredClone(fixture.entry_points), contacts: structuredClone(fixture.contacts), deadlines: [],
      effective_interval: null, lookup_path: "exact_id", conflict_state: { kind: "resolved" }, uncertainty_state: "none",
      evidence: { valid_interval: { kind: "unknown" }, last_verified_at: verified, observed_at: verified, known_at: verified, reviewed_at: verified,
        projection: { kind: "complete" }, assessments: [{ source_id: fixture.source_id, subject: fixture.source_url, authority: "DemoReviewed", last_verified_at: verified, reviewed_at: verified }] }
    } }
  } };
}
class Element {
  constructor(tag, document) { this.tagName = tag; this.document = document; this.children = []; this.events = {}; this.hidden = false; this.checked = false; this.disabled = false; this.value = ""; this.attributes = {}; this._text = ""; }
  set textContent(value) { this._text = String(value); this.children = []; }
  get textContent() { return this._text + this.children.map(child => child.textContent).join(""); }
  set innerHTML(_) { throw new Error("Unsafe innerHTML use"); }
  append(...nodes) { for (const node of nodes) { node.parent = this; this.children.push(node); } }
  insertBefore(node, reference) { if (!reference) this.append(node); else { node.parent = this; this.children.splice(this.children.indexOf(reference), 0, node); } }
  replaceChildren(...nodes) { this.children = []; this.append(...nodes); }
  setAttribute(key, value) { this.attributes[key] = value; }
  querySelector(selector) { return this.descendants().find(node => selector[0] === "#" ? node.id === selector.slice(1) : node.className === selector.slice(1)) ?? null; }
  descendants() { return this.children.flatMap(node => [node, ...node.descendants()]); }
  addEventListener(name, callback) { this.events[name] = callback; }
  async click() { if (this.disabled) return; if (this.tagName === "a") this.document.downloads.push({ href: this.href, download: this.download }); return this.events.click?.(); }
  remove() { if (this.parent) this.parent.children = this.parent.children.filter(node => node !== this); }
  focus() { this.document.activeElement = this; }
  select() { this.selected = true; }
}
function setup(clipboard) {
  const document = { downloads: [], createElement(tag) { return new Element(tag, this); }, querySelector(selector) { return this.body.querySelector(selector); } };
  document.body = document.createElement("body");
  const result = document.createElement("article"); result.id = "result";
  const grid = document.createElement("div"); grid.className = "result-grid";
  for (const id of ["prerequisites", "steps"]) { const node = document.createElement("ol"); node.id = id; grid.append(node); }
  result.append(grid); document.body.append(result);
  const blobs = []; const revoked = []; const timers = [];
  class LocalURL extends URL {}
  LocalURL.createObjectURL = blob => { blobs.push(blob); return `blob:local/${blobs.length}`; };
  LocalURL.revokeObjectURL = url => revoked.push(url);
  const window = { navigator: { clipboard }, setTimeout(callback) { timers.push(callback); } };
  vm.runInNewContext(source, { window, document, URL: LocalURL, Blob });
  const api = window.UcaAffairsChecklist;
  const get = id => document.querySelector(`#${id}`);
  const inputs = () => document.body.descendants().filter(node => node.tagName === "input");
  return { api, document, get, inputs, blobs, revoked, timers, localURL: LocalURL };
}

test("reviewed fixture content and current personal checks reach Markdown", () => {
  const { api } = setup(); const data = payload();
  const md = api.formatMarkdown(data, { prerequisites: [true], steps: [false, true] });
  assert.ok(md.startsWith(`# ${fixture.title} — 个人办理清单\n`));
  for (const value of [...fixture.prerequisites, ...fixture.steps]) assert.ok(md.includes(api.escapeMarkdown(value)));
  assert.ok(md.includes(`- [x] ${api.escapeMarkdown(fixture.prerequisites[0])}`));
  assert.ok(md.includes(`- [x] 2. ${api.escapeMarkdown(fixture.steps[1])}`));
  for (const entry of fixture.entry_points) assert.ok(md.includes(`(<${entry.url}>)`));
  for (const value of [fixture.procedure_id, fixture.source_id, fixture.normalized_digest, new Date(fixture.last_verified_at_secs * 1000).toISOString()]) assert.ok(md.includes(api.escapeMarkdown(value)));
  assert.match(md, /不是官方受理、批准或完成凭证/);
  assert.match(md, /仅本页暂存/);
  assert.match(md, /非导出时重新核验/);
});

test("stale, conflict, validity, truncation and uncertainty details are not upgraded or dropped", () => {
  const { api } = setup(); const data = payload(); const { view } = data.terminal.outcome;
  data.terminal.outcome.freshness = { kind: "stale", last_verified_at: 1234, max_fresh_age_seconds: 10, max_presentable_age_seconds: 100 };
  view.conflict_state = { kind: "unresolved", detail: { description: "不同来源有冲突", conflict_kind: "timing", evidence_refs: ["src:other"] } };
  view.evidence.valid_interval = { kind: "known_interval", from: 1234, to: 5678 };
  view.evidence.projection = { kind: "truncated", omitted_count: 3, selection_rule_version: 1 };
  view.uncertainty_state = "待教务处确认";
  view.deadlines = [{ label: "提交期限", kind: "fixed", at: 1000 }];
  const md = api.formatMarkdown(data);
  for (const value of ["stale", "max_fresh_age_seconds", "unresolved", "不同来源有冲突", "src:other", "known_interval", "5678", "omitted_count", "待教务处确认", "提交期限"]) assert.ok(md.includes(api.escapeMarkdown(value)), value);
  view.evidence.last_verified_at = null;
  assert.ok(api.formatMarkdown(data).includes("最近核验（UTC）：未提供"));
});

test("unsafe Markdown is inert and URL destinations are strictly HTTP(S)", () => {
  const { api, get } = setup(); const data = payload();
  const attack = '<img src=x onerror=alert(1)>\n# injected [click](javascript:alert(1)) &lt;script&gt; `code`';
  data.terminal.outcome.view.ordered_steps[0].instruction = attack;
  const urls = ["javascript:alert(1)", "data:text/html,bad", "file:///etc/passwd", "/relative", "//evil.test", "https://user:pass@evil.test/", "https://evil.test/\nfoo"];
  data.terminal.outcome.view.entry_points = urls.map(url => ({ label: attack, url }));
  for (const url of urls) assert.equal(api.safeUrl(url), null);
  assert.equal(api.safeUrl("https://example.test/a(b)>"), "https://example.test/a%28b%29%3E");
  const md = api.formatMarkdown(data);
  assert.ok(md.includes(api.escapeMarkdown(attack)));
  assert.ok(!md.includes("\n# injected")); assert.ok(!/(?<!\\)<img/.test(md));
  assert.ok(!md.includes("(<javascript:")); assert.ok(!md.includes("https://user:pass"));
  assert.equal(api.render(data), true);
  assert.ok(get("steps").textContent.includes(attack));
  assert.equal(get("steps").descendants().filter(node => node.tagName === "img").length, 0);
});

test("initial, failed, private, unverified and malformed terminals cannot export", () => {
  const { api, get } = setup();
  assert.equal(api.markdown(), null);
  for (const change of [data => { data.kind = "unavailable"; }, data => { data.redaction = "private"; }, data => { data.terminal.outcome.kind = "not_found"; }, data => { data.terminal.lineage.kind = "unverified"; }, data => { data.terminal.outcome.view.ordered_steps = []; }, data => { data.terminal.outcome.view.ordered_steps = [{ ordinal: 0, instruction: {} }]; }]) {
    assert.equal(api.render(payload()), true);
    const data = payload(); change(data);
    assert.equal(api.formatMarkdown(data), null);
    assert.equal(api.render(data), false);
    assert.equal(api.markdown(), null);
    assert.equal(get("affairs-checklist-copy").disabled, true);
    assert.equal(get("affairs-checklist-download").disabled, true);
  }
});

test("lookup invalidation clears checks, rejects old response tokens and freezes source snapshot", () => {
  const { api, inputs } = setup(); const data = payload();
  const first = api.invalidate(); assert.equal(api.render(data, first), true);
  inputs()[0].checked = true; inputs()[0].events.change();
  assert.match(api.markdown(), /- \[x\]/);
  data.terminal.outcome.view.title = "external mutation";
  assert.ok(!api.markdown().includes("external mutation"));
  const oldInput = inputs()[0];
  const second = api.invalidate(); assert.equal(oldInput.checked, false); assert.equal(oldInput.disabled, true);
  assert.equal(api.markdown(), null);
  assert.equal(api.render(payload(), first), false);
  assert.equal(api.render(payload(), second), true);
  oldInput.checked = true; oldInput.events.change();
  assert.ok(!api.markdown().includes("- [x]"));
  assert.ok(inputs().every(input => !input.checked));
});

test("clipboard copy contains exact Markdown and failure offers selectable current text", async () => {
  const copied = []; const success = setup({ async writeText(text) { copied.push(text); } });
  success.api.render(payload()); await success.get("affairs-checklist-copy").click();
  assert.equal(copied[0], success.api.markdown());
  assert.match(success.get("affairs-checklist-status").textContent, /已复制/);
  for (const clipboard of [undefined, { async writeText() { throw new Error("denied"); } }]) {
    const failure = setup(clipboard); failure.api.render(payload());
    await failure.get("affairs-checklist-copy").click();
    const fallback = failure.get("affairs-checklist-fallback");
    assert.equal(fallback.hidden, false); assert.equal(fallback.selected, true);
    assert.equal(fallback.value, failure.api.markdown());
    failure.inputs()[0].checked = true; failure.inputs()[0].events.change();
    assert.equal(fallback.value, failure.api.markdown());
    failure.api.invalidate(); assert.equal(fallback.hidden, true); assert.equal(fallback.value, "");
  }
});

test("pending clipboard rejection cannot restore stale fallback after lookup", async () => {
  let rejectCopy;
  const { api, get } = setup({ writeText() { return new Promise((_, reject) => { rejectCopy = reject; }); } });
  api.render(payload()); const pending = get("affairs-checklist-copy").click();
  api.invalidate(); rejectCopy(new Error("denied")); await pending;
  assert.equal(get("affairs-checklist-fallback").hidden, true);
  assert.equal(get("affairs-checklist-copy").disabled, true);
  assert.equal(api.markdown(), null);
});

test("hidden result cannot mount or export before successful rendering", () => {
  const { api, get } = setup();
  get("result").hidden = true;
  assert.equal(api.render(payload()), false);
  assert.equal(api.markdown(), null);
  assert.equal(get("affairs-checklist"), null);
});

test("local download failure exposes manual-save fallback without a success claim", async () => {
  const { api, get, localURL, document } = setup();
  api.render(payload());
  localURL.createObjectURL = () => { throw new Error("unavailable"); };
  await get("affairs-checklist-download").click();
  assert.match(get("affairs-checklist-status").textContent, /无法启动下载/);
  assert.equal(get("affairs-checklist-fallback").hidden, false);
  assert.equal(get("affairs-checklist-fallback").value, api.markdown());
  assert.equal(document.downloads.length, 0);
});

test("download emits local Markdown Blob, fixed filename, revokes URL and does not run while invalid", async () => {
  const { api, get, document, blobs, timers, revoked } = setup();
  api.render(payload()); const expected = api.markdown();
  await get("affairs-checklist-download").click();
  assert.equal(blobs.length, 1); assert.equal(await blobs[0].text(), expected);
  assert.equal(blobs[0].type, "text/markdown;charset=utf-8");
  assert.deepEqual(document.downloads, [{ href: "blob:local/1", download: "affairs-personal-checklist.md" }]);
  assert.equal(document.body.descendants().filter(node => node.tagName === "a").length, 0);
  timers.forEach(callback => callback()); assert.deepEqual(revoked, ["blob:local/1"]);
  api.invalidate(); await get("affairs-checklist-download").click(); assert.equal(blobs.length, 1);
});
