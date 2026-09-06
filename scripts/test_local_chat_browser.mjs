#!/usr/bin/env node
// Explicit local-chat smoke against an already-running application, never a model endpoint.
// Reuses the repository's Chromium CDP pipe approach; no browser-plugin dependency.
import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import { access, mkdir, mkdtemp, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { isIP } from "node:net";
import { join, resolve, relative } from "node:path";

if (process.argv.length !== 4 || process.argv[2] !== "--base") {
  throw Error("Usage: node scripts/test_local_chat_browser.mjs --base http://127.0.0.1:PORT");
}
const url = new URL(process.argv[3]);
const host = url.hostname.replace(/^\[|\]$/g, "");
if (url.protocol !== "http:" || url.username || url.password || url.search || url.hash
  || url.pathname !== "/" || !((isIP(host) === 4 && host.startsWith("127.")) || host === "::1")) {
  throw Error("Application base must be an HTTP numeric loopback origin");
}
const base = url.origin;
const replyTimeoutMs = 60000;
const delay = ms => new Promise(resolveDelay => setTimeout(resolveDelay, ms));
let chrome;
let chromeOutput = "";
let work;
const evidence = { mode: "REAL_LOCAL_CHAT_BROWSER", cases: [] };
const pass = name => evidence.cases.push({ name, status: "PASS" });

async function findChrome() {
  for (const path of [process.env.CHROME_BIN, "/usr/bin/google-chrome", "/usr/bin/google-chrome-stable", "/usr/bin/chromium", "/usr/bin/chromium-browser"].filter(Boolean)) {
    try { await access(path); return path; } catch (_) { /* Try the next runner-provided browser. */ }
  }
  throw Error("Chrome/Chromium not found; set CHROME_BIN to its executable");
}

class CdpPipe {
  constructor(process) {
    this.process = process;
    this.nextId = 1;
    this.pending = new Map();
    this.events = [];
    this.buffer = Buffer.alloc(0);
    process.stdio[4].on("data", (chunk) => this.receive(chunk));
    const failPending = (error) => {
      for (const { reject, timer } of this.pending.values()) {
        clearTimeout(timer);
        reject(error);
      }
      this.pending.clear();
    };
    process.stdio[3].on("error", (error) => failPending(error));
    process.stdio[4].on("error", (error) => failPending(error));
    process.on("exit", (code) => {
      failPending(new Error(`Chrome exited before CDP response (${code})`));
    });
  }

  receive(chunk) {
    this.buffer = Buffer.concat([this.buffer, chunk]);
    let separator;
    while ((separator = this.buffer.indexOf(0)) >= 0) {
      const frame = this.buffer.subarray(0, separator).toString("utf8");
      this.buffer = this.buffer.subarray(separator + 1);
      if (!frame) continue;
      const message = JSON.parse(frame);
      if (message.id) {
        const pending = this.pending.get(message.id);
        if (!pending) continue;
        this.pending.delete(message.id);
        clearTimeout(pending.timer);
        if (message.error) pending.reject(new Error(JSON.stringify(message.error)));
        else pending.resolve(message.result ?? {});
      } else {
        this.events.push(message);
      }
    }
  }

  send(method, params = {}, sessionId = undefined, timeoutMs = 10000) {
    const id = this.nextId;
    this.nextId += 1;
    const message = { id, method, params };
    if (sessionId) message.sessionId = sessionId;
    return new Promise((resolveCall, rejectCall) => {
      const timer = setTimeout(() => {
        this.pending.delete(id);
        rejectCall(new Error(`CDP timeout: ${method}`));
      }, timeoutMs);
      this.pending.set(id, { resolve: resolveCall, reject: rejectCall, timer });
      this.process.stdio[3].write(`${JSON.stringify(message)}\0`);
    });
  }
}

async function stop(child) {
  if (!child || child.exitCode !== null) return;
  child.kill("SIGTERM");
  await Promise.race([
    new Promise((resolveExit) => child.once("exit", resolveExit)),
    delay(3000).then(() => child.kill("SIGKILL"))
  ]);
}

try {
  const statusResponse = await fetch(`${base}/api/v1/agent/status`, { signal: AbortSignal.timeout(10000), redirect: "error" });
  assert.equal(statusResponse.status, 200);
  assert.match(statusResponse.headers.get("cache-control") ?? "", /no-store/);
  const configured = await statusResponse.json();
  assert.equal(configured.schema, "ustc-agent-provider-status/v1");
  assert.equal(configured.provider?.mode, "local-chat", "Requires the explicit real local-chat profile");
  assert.equal(typeof configured.provider.model, "string");
  assert.ok(configured.provider.model.trim());
  assert.equal(configured.tool_calling, false);
  assert.ok(Number.isSafeInteger(configured.context_limit_tokens));
  // A single valid UI message must exceed the configured complete-wire input allowance.
  const oversizeLength = Math.floor(configured.context_limit_tokens * 0.9) - 512 + 1;
  assert.ok(oversizeLength > 0 && oversizeLength <= 4096,
    "This small-window smoke needs an input allowance below the UI's 4096-byte message limit");
  evidence.provider = configured.provider;
  evidence.context_limit_tokens = configured.context_limit_tokens;
  work = await mkdtemp(join(tmpdir(), "uca-local-chat-browser-"));
  chrome = spawn(await findChrome(), ["--headless=new", "--no-sandbox", "--disable-gpu", "--disable-dev-shm-usage", "--no-first-run", "--no-default-browser-check", "--remote-debugging-pipe", `--user-data-dir=${join(work, "chrome")}`, "about:blank"], { stdio: ["ignore", "ignore", "pipe", "pipe", "pipe"] });
  chrome.stderr.on("data", chunk => { chromeOutput = `${chromeOutput}${chunk}`.slice(-32768); });
  const cdp = new CdpPipe(chrome);
  const { targetId } = await cdp.send("Target.createTarget", { url: "about:blank" }, undefined, 30000);
  const { sessionId } = await cdp.send("Target.attachToTarget", { targetId, flatten: true });
  await cdp.send("Page.enable", {}, sessionId);
  await cdp.send("Runtime.enable", {}, sessionId);
  await cdp.send("Emulation.setDeviceMetricsOverride", { width: 1440, height: 900, deviceScaleFactor: 1, mobile: false }, sessionId);
  // Observe real application requests/responses without replacing their contents.
  await cdp.send("Page.addScriptToEvaluateOnNewDocument", { source: `(() => {
    const original = window.fetch.bind(window);
    window.__localSmoke = {statuses: [], chats: []};
    window.fetch = async (input, options = {}) => {
      const response = await original(input, options);
      const path = new URL(typeof input === 'string' ? input : input.url, location.href).pathname;
      if (path === '/api/v1/agent/status') {
        window.__localSmoke.statuses.push(await response.clone().json());
      } else if (path === '/api/v1/agent/chat') {
        const request = JSON.parse(options.body);
        const headers = new Headers(options.headers);
        window.__localSmoke.chats.push({status: response.status, payload: await response.clone().json(),
          opportunity: request.opportunity_context, confirmation: headers.has('X-USTC-Opportunity-Confirmation'),
          authorization: headers.has('Authorization')});
      }
      return response;
    };
  })();` }, sessionId);
  await cdp.send("Page.navigate", { url: base }, sessionId);
  const evaluate = async expression => {
    const result = await cdp.send("Runtime.evaluate", { expression, awaitPromise: true, returnByValue: true }, sessionId);
    if (result.exceptionDetails) throw Error(result.exceptionDetails.exception?.description ?? "Browser evaluation failed");
    return result.result?.value;
  };
  const waitFor = async (expression, label, timeout = 15000) => {
    const deadline = Date.now() + timeout;
    while (Date.now() < deadline) {
      if (await evaluate(expression)) return;
      await delay(50);
    }
    throw Error(`Browser condition timeout: ${label}`);
  };
  const pointer = async selector => {
    const point = await evaluate(`(() => {
      const el = document.querySelector(${JSON.stringify(selector)});
      el.scrollIntoView({block: 'center'});
      const r = el.getBoundingClientRect(), x = r.x + r.width / 2, y = r.y + r.height / 2;
      if (el.disabled || !r.width || !r.height || !el.contains(document.elementFromPoint(x,y))) throw Error('Control is not reachable');
      return {x,y};
    })()`);
    await cdp.send("Input.dispatchMouseEvent", {type: "mousePressed", ...point, button: "left", clickCount: 1}, sessionId);
    await cdp.send("Input.dispatchMouseEvent", {type: "mouseReleased", ...point, button: "left", clickCount: 1}, sessionId);
  };
  const navigate = async route => {
    await evaluate(`window.UcaShell.navigate(${JSON.stringify(route)})`);
    await waitFor(`!document.querySelector('[data-view="${route}"]').hidden`, `route ${route}`);
  };
  const screenshot = async name => {
    if (!process.env.UCA_SHELL_SCREENSHOTS) return;
    const directory = resolve(process.env.UCA_SHELL_SCREENSHOTS);
    await mkdir(directory, { recursive: true });
    const result = await cdp.send("Page.captureScreenshot", { format: "png" }, sessionId);
    await writeFile(join(directory, `${name}.png`), Buffer.from(result.data, "base64"));
  };
  await waitFor("document.readyState === 'complete' && !!window.UcaProviderStatus && window.__localSmoke.statuses.length > 0", "configuration loaded");
  await waitFor("document.querySelector('#provider-status').textContent.includes('待验证')", "configuration is not a connection receipt");
  assert.equal(await evaluate("document.querySelector('#provider-model').textContent"), configured.provider.model);
  assert.equal(await evaluate("window.UcaProviderStatus.toolCalling"), false);
  pass("configuration-loaded-not-yet-connected");

  // UI-only synthetic hint: no profile is created, looked up, planned or deleted.
  await evaluate("setOpportunityHint('profile:synthetic:ui-only-local-chat-smoke');setOpportunityBusy(true)");
  assert.deepEqual(await evaluate("['view','plan','delete'].map(name=>document.querySelector('#opportunity-'+name).disabled)"), [true,true,true]);
  await evaluate("setOpportunityBusy(false)");
  assert.deepEqual(await evaluate("['view','plan','delete'].map(name=>document.querySelector('#opportunity-'+name).disabled)"), [false,false,false]);
  assert.equal(await evaluate("document.querySelector('#chat-opportunity-confirm').disabled"), true);
  // Even a programmatically checked disabled control must not add profile intent to Chat.
  await evaluate("document.querySelector('#chat-opportunity-confirm').checked=true");
  pass("local-profile-keeps-explicit-user-controls-and-blocks-chat-consent");

  await navigate("chat");
  await pointer("#chat-input");
  await cdp.send("Input.insertText", { text: "你好，请简短回答。" }, sessionId);
  const started = Date.now();
  await pointer("#chat-send");
  await waitFor("window.__localSmoke.chats.length === 1 && !chatPending", "real local model response", replyTimeoutMs);
  const successful = await evaluate("window.__localSmoke.chats[0]");
  assert.equal(successful.status, 200, `Chat failed: ${successful.payload?.error ?? 'unexpected status'}`);
  assert.equal(successful.payload.schema, "ustc-agent-chat-response/v1");
  assert.deepEqual(successful.payload.provider, configured.provider);
  assert.deepEqual(successful.payload.tool_trace, []);
  assert.ok(successful.payload.answer.trim());
  assert.equal(successful.opportunity, null);
  assert.equal(successful.confirmation, false);
  assert.equal(successful.authorization, false);
  assert.equal(await evaluate("document.querySelector('#chat-error').hidden"), true);
  assert.equal(await evaluate("document.querySelector('#provider-status').textContent.includes('已收到回答')"), true);
  assert.equal(await evaluate("[...document.querySelectorAll('#chat-messages .chat-message')].some(el=>el.ucaAnswerText === window.__localSmoke.chats[0].payload.answer.trim())"), true);
  assert.equal(await evaluate("!!document.querySelector('[data-role=assistant] .chat-message-body')?.textContent.trim()"), true);
  evidence.response_ms = Date.now() - started;
  evidence.answer_bytes = Buffer.byteLength(successful.payload.answer);
  await evaluate("setOpportunityHint(null)");
  await screenshot("local-chat");
  pass("real-browser-submit-answer-identity-and-zero-tool-trace");

  await navigate("settings");
  const beforeRefresh = await evaluate("window.__localSmoke.statuses.length");
  await pointer("#provider-status-refresh");
  await waitFor(`window.__localSmoke.statuses.length > ${beforeRefresh} && !document.querySelector('#provider-status-refresh').disabled`, "configuration refresh");
  assert.equal(await evaluate("window.__localSmoke.chats.length"), 1, "Status refresh creates no Chat request");
  assert.match(await evaluate("document.querySelector('#provider-tool-note').textContent"), /不提供插件工具调用/);
  assert.equal(await evaluate("document.querySelector('#provider-status').textContent.includes('已收到回答')"), true);
  await screenshot("local-chat-settings");
  pass("read-only-refresh-and-honest-tool-limit");

  await pointer("#chat-clear");
  await navigate("chat");
  await pointer("#chat-input");
  await cdp.send("Input.insertText", { text: "x".repeat(oversizeLength) }, sessionId);
  await pointer("#chat-send");
  await waitFor("window.__localSmoke.chats.length === 2 && !chatPending", "explicit context budget rejection", replyTimeoutMs);
  const rejected = await evaluate("window.__localSmoke.chats[1]");
  assert.ok(rejected.status >= 400);
  assert.equal(rejected.payload.schema, "ustc-agent-chat-error/v1");
  assert.equal(rejected.payload.error, "context_budget_exceeded");
  assert.equal(await evaluate("document.querySelector('#chat-error').hidden"), false);
  assert.equal(await evaluate("document.querySelector('#chat-error-code').textContent"), "context_budget_exceeded");
  assert.equal(await evaluate("document.querySelector('#chat-input').value.length"), oversizeLength);
  assert.equal(await evaluate("document.querySelector('#provider-model').textContent"), configured.provider.model);
  assert.equal(await evaluate("window.UcaProviderStatus.toolCalling"), false);
  assert.equal(await evaluate("document.querySelector('#provider-status').textContent.includes('本地模型')"), true);
  await screenshot("local-chat-budget");
  pass("context-budget-error-preserves-draft-without-profile-fallback");
  assert.deepEqual(cdp.events.filter(event => event.method === "Runtime.exceptionThrown"), []);
  console.log(JSON.stringify(evidence, null, 2));
} catch (error) {
  console.error(error?.stack ?? error);
  if (chromeOutput) console.error(`Chromium diagnostics:\n${chromeOutput}`);
  process.exitCode = 1;
} finally {
  await stop(chrome);
  if (work) {
    const within = relative(resolve(tmpdir()), resolve(work));
    if (within.startsWith("uca-local-chat-browser-") && !within.includes("..")) {
      await rm(work, { recursive: true, force: true, maxRetries: 10, retryDelay: 100 });
    } else {
      throw Error("Refusing to remove a path outside the test temporary directory");
    }
  }
}
