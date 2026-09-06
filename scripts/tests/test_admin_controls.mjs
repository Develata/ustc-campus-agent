import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import vm from 'node:vm';
import test from 'node:test';

const source = readFileSync(new URL('../../apps/ustc-agentd/src/web/admin-controls.js', import.meta.url), 'utf8');
const endpoints = {
  publication: '/api/v1/demo/administrator/affairs/publication',
  'radar-publication': '/api/v1/demo/administrator/changes/publication',
};

function element() {
  const listeners = new Map();
  return {
    checked: false, disabled: false, textContent: '—',
    addEventListener(name, listener) {
      if (!listeners.has(name)) listeners.set(name, []);
      listeners.get(name).push(listener);
    },
    fire(name) {
      for (const listener of listeners.get(name) ?? []) listener({ target: this });
    },
  };
}

function payload(url, method) {
  const affairs = url === endpoints.publication;
  if (method === 'POST') return {
    schema: affairs ? 'ustc-affairs-publication-response/v1' : 'ustc-change-publication-response/v1',
    outcome: { kind: 'published', publication_revision: 'revision:fixture:1' },
  };
  return affairs ? {
    schema: 'ustc-affairs-publication-status/v1', publication_revision: 'revision:fixture:1',
    publication_receipt_id: 'receipt:affairs:1', control_evidence_event_count: 2,
  } : {
    schema: 'ustc-change-publication-status/v1', review_count: 1, publication_count: 1,
    publication_receipt_id: 'receipt:changes:1', control_evidence_event_count: 2,
  };
}

const response = (value, ok = true) => ({ ok, status: ok ? 200 : 503, json: async () => value });
const tick = () => new Promise(resolve => setImmediate(resolve));
async function settled(predicate, label) {
  for (let i = 0; i < 30; i++) {
    if (predicate()) return;
    await tick();
  }
  assert.ok(predicate(), label);
}

function harness({ post, onChangePublished = async () => {} } = {}) {
  const nodes = new Map();
  for (const prefix of Object.keys(endpoints)) {
    for (const suffix of ['refresh', 'confirm', 'publish', 'status', 'revision', 'receipt',
      'evidence-count', 'review-count', 'count']) nodes.set(`#${prefix}-${suffix}`, element());
    nodes.get(`#${prefix}-publish`).disabled = true;
  }
  const root = { querySelector: selector => nodes.get(selector) ?? null };
  const calls = [];
  const request = async (url, options) => {
    assert.ok(Object.values(endpoints).includes(url), 'only publication endpoints are admitted');
    const call = { url, ...options };
    calls.push(call);
    if (options.method === 'POST' && post) return post(call);
    return response(payload(url, options.method));
  };
  // No app.js, document, chat/profile state or global fetch is available.
  const context = vm.createContext({ window: {}, Error });
  vm.runInContext(source, context);
  const mount = () => context.window.UcaAdminControls.mount({ root, request, onChangePublished });
  const node = (prefix, suffix) => nodes.get(`#${prefix}-${suffix}`);
  const confirm = prefix => {
    node(prefix, 'confirm').checked = true;
    node(prefix, 'confirm').fire('change');
  };
  const posts = () => calls.filter(call => call.method === 'POST');
  return { mount, node, confirm, posts, calls };
}

test('isolated mount reads both states once; duplicate mount and unconfirmed clicks never publish', async () => {
  const h = harness();
  h.mount();
  await settled(() => h.calls.length === 2 && Object.keys(endpoints).every(p => !h.node(p, 'refresh').disabled), 'initial reads settle');
  assert.deepEqual(h.calls.map(call => call.method), ['GET', 'GET']);
  h.mount();
  for (const prefix of Object.keys(endpoints)) h.node(prefix, 'publish').fire('click');
  await tick();
  assert.equal(h.calls.length, 2);
  assert.equal(h.posts().length, 0);
});

for (const prefix of Object.keys(endpoints)) {
  test(`${prefix}: explicit confirmation permits one pending publication and is consumed on success`, async () => {
    let release;
    const pending = new Promise(resolve => { release = resolve; });
    const h = harness({ post: () => pending });
    h.mount();
    await tick();
    h.confirm(prefix);
    assert.equal(h.node(prefix, 'publish').disabled, false);
    h.node(prefix, 'publish').fire('click');
    h.node(prefix, 'publish').fire('click');
    assert.equal(h.posts().length, 1);
    assert.equal(h.node(prefix, 'confirm').disabled, true);
    assert.equal(h.posts()[0].url, endpoints[prefix]);
    assert.deepEqual(JSON.parse(h.posts()[0].body), { confirm_publish: true });
    assert.equal(h.posts()[0].headers['X-USTC-Agent-Administrator-Demo'], 'confirm-v1');
    release(response(payload(endpoints[prefix], 'POST')));
    await settled(() => !h.node(prefix, 'confirm').disabled, 'publication settles');
    assert.equal(h.node(prefix, 'confirm').checked, false);
    assert.equal(h.node(prefix, 'publish').disabled, true);
    assert.notEqual(h.node(prefix, 'receipt').textContent, '—');
    h.node(prefix, 'publish').fire('click');
    assert.equal(h.posts().length, 1, 'another command requires fresh confirmation');
  });

  test(`${prefix}: rejected publication consumes confirmation and never emits a change notification`, async () => {
    let notifications = 0;
    const h = harness({
      post: () => response({ error: 'publication_denied' }, false),
      onChangePublished: async () => { notifications++; },
    });
    h.mount();
    await tick();
    h.confirm(prefix);
    h.node(prefix, 'publish').fire('click');
    await settled(() => !h.node(prefix, 'confirm').disabled, 'rejection settles');
    assert.match(h.node(prefix, 'status').textContent, /失败/);
    assert.equal(h.node(prefix, 'confirm').checked, false);
    assert.equal(h.node(prefix, 'publish').disabled, true);
    assert.equal(notifications, 0);
    assert.equal(h.posts().length, 1);
  });
}

test('a post-publication notification failure preserves the acknowledged publication outcome', async () => {
  let notifications = 0;
  const h = harness({ onChangePublished: async () => {
    notifications++;
    throw new Error('board_refresh_failed');
  } });
  h.mount();
  await tick();
  h.confirm('radar-publication');
  h.node('radar-publication', 'publish').fire('click');
  await settled(() => !h.node('radar-publication', 'confirm').disabled, 'notification failure settles');
  assert.equal(notifications, 1);
  assert.match(h.node('radar-publication', 'status').textContent, /已发布/);
  assert.doesNotMatch(h.node('radar-publication', 'status').textContent, /发布失败/);
  assert.equal(h.node('radar-publication', 'receipt').textContent, 'receipt:changes:1');
  assert.equal(h.node('radar-publication', 'confirm').checked, false);
  assert.equal(h.posts().length, 1);
});
