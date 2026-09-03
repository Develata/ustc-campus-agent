#!/usr/bin/env node
import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import { access, mkdtemp, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";

const repo = resolve(new URL("..", import.meta.url).pathname);
const binary = resolve(process.argv[2] ?? "target/debug/ustc-agentd");
const port = 18790;
const base = `http://127.0.0.1:${port}`;
const work = await mkdtemp(join(tmpdir(), "uca-agent-chat-browser-"));

await access(binary);

const boundedOutput = (current, chunk) => `${current}${chunk}`.slice(-32768);
let serverOutput = "";
let chromeOutput = "";
const serverEnv = { ...process.env, UCA_AGENT_PROVIDER: "mock" };
for (const name of [
  "UCA_AGENT_BASE_URL",
  "UCA_AGENT_MODEL",
  "UCA_AGENT_API_KEY_FILE",
  "UCA_AGENT_TIMEOUT_MS",
  "UCA_AGENT_CONTEXT_TOKENS"
]) {
  delete serverEnv[name];
}
const server = spawn(binary, [
  "serve-web",
  "--bind", `127.0.0.1:${port}`,
  "--fixture", join(repo, "fixtures/affairs/proc-011-reviewed.json"),
  "--change-fixture", join(repo, "fixtures/change-radar/academic-calendar-demo-reviewed.json"),
  "--opportunity-fixture", join(repo, "fixtures/opportunity-graph/course-planning-demo-reviewed.json"),
  "--opportunity-catalog", join(repo, "market/fixtures/course-planning/minimal-v0.json"),
  "--opportunity-profile-store", join(work, "opportunity-profiles.json"),
  "--store", join(work, "affairs-records.json"),
  "--idempotency", join(work, "affairs-idempotency.json"),
  "--session-store", join(work, "m00-sessions.json")
], { cwd: repo, env: serverEnv, stdio: ["ignore", "pipe", "pipe"] });
server.stdout.on("data", (chunk) => { serverOutput = boundedOutput(serverOutput, chunk); });
server.stderr.on("data", (chunk) => { serverOutput = boundedOutput(serverOutput, chunk); });

let chrome;
let cdp;
const delay = (ms) => new Promise((resolveDelay) => setTimeout(resolveDelay, ms));

async function waitForHealth() {
  for (let attempt = 0; attempt < 100; attempt += 1) {
    if (server.exitCode !== null) {
      throw new Error(`server exited early (${server.exitCode}):\n${serverOutput}`);
    }
    const controller = new AbortController();
    const timer = setTimeout(() => controller.abort(), 500);
    try {
      const response = await fetch(`${base}/healthz`, { signal: controller.signal });
      if (response.ok) {
        await response.arrayBuffer();
        return;
      }
    } catch (_) {
      // Bounded retry while the loopback server starts.
    } finally {
      clearTimeout(timer);
    }
    await delay(50);
  }
  throw new Error(`server health timeout:\n${serverOutput}`);
}

async function findChrome() {
  const candidates = [
    process.env.CHROME_BIN,
    "/usr/bin/google-chrome",
    "/usr/bin/google-chrome-stable",
    "/usr/bin/chromium",
    "/usr/bin/chromium-browser"
  ].filter(Boolean);
  for (const candidate of candidates) {
    try {
      await access(candidate);
      return candidate;
    } catch (_) {
      // Try the next runner-provided browser.
    }
  }
  throw new Error("Chrome/Chromium executable not found");
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
  await waitForHealth();
  const chromePath = await findChrome();
  chrome = spawn(chromePath, [
    "--headless=new",
    "--no-sandbox",
    "--disable-gpu",
    "--disable-dev-shm-usage",
    "--no-first-run",
    "--no-default-browser-check",
    "--remote-debugging-pipe",
    `--user-data-dir=${join(work, "chrome")}`,
    "about:blank"
  ], { stdio: ["ignore", "ignore", "pipe", "pipe", "pipe"] });
  chrome.stderr.on("data", (chunk) => { chromeOutput = boundedOutput(chromeOutput, chunk); });
  cdp = new CdpPipe(chrome);

  const { targetId } = await cdp.send("Target.createTarget", { url: "about:blank" }, undefined, 30000);
  const { sessionId } = await cdp.send("Target.attachToTarget", { targetId, flatten: true });
  await cdp.send("Page.enable", {}, sessionId);
  await cdp.send("Runtime.enable", {}, sessionId);
  await cdp.send("Emulation.setDeviceMetricsOverride", {
    width: 1280, height: 900, deviceScaleFactor: 1, mobile: false
  }, sessionId);
  await cdp.send("Page.navigate", { url: base }, sessionId);

  const evaluate = async (expression) => {
    const response = await cdp.send("Runtime.evaluate", {
      expression,
      awaitPromise: true,
      returnByValue: true
    }, sessionId);
    if (response.exceptionDetails) {
      throw new Error(response.exceptionDetails.exception?.description ?? "browser evaluation failed");
    }
    return response.result?.value;
  };
  const waitFor = async (expression, label, attempts = 200) => {
    for (let attempt = 0; attempt < attempts; attempt += 1) {
      if (await evaluate(expression)) return;
      await delay(25);
    }
    throw new Error(`browser condition timeout: ${label}`);
  };
  const reloadAndWait = async (label) => {
    const previousTimeOrigin = await evaluate("performance.timeOrigin");
    await cdp.send("Page.reload", {}, sessionId);
    for (let attempt = 0; attempt < 200; attempt += 1) {
      try {
        if (await evaluate(`document.readyState === 'complete' && performance.timeOrigin !== ${previousTimeOrigin}`)) {
          return;
        }
      } catch (_) {
        // The prior execution context is intentionally replaced during reload.
      }
      await delay(25);
    }
    throw new Error(`browser reload timeout: ${label}`);
  };
  const submitWithEnter = async (prompt, useOpportunity = false) => {
    const before = await evaluate("window.__ucaAssistantAdds");
    const prepared = await evaluate(`(() => {
      const input = document.querySelector('#chat-input');
      input.value = ${JSON.stringify(prompt)};
      input.dispatchEvent(new Event('input', { bubbles: true }));
      const confirm = document.querySelector('#chat-opportunity-confirm');
      confirm.checked = ${useOpportunity};
      input.focus();
      return document.activeElement === input;
    })()`);
    assert.equal(prepared, true, "textarea must receive visible keyboard focus");
    await cdp.send("Input.dispatchKeyEvent", {
      type: "keyDown", key: "Enter", code: "Enter", windowsVirtualKeyCode: 13
    }, sessionId);
    await cdp.send("Input.dispatchKeyEvent", {
      type: "keyUp", key: "Enter", code: "Enter", windowsVirtualKeyCode: 13
    }, sessionId);
    await waitFor(
      `window.__ucaAssistantAdds > ${before}`,
      `assistant answer for ${prompt}`
    );
    await waitFor("document.querySelector('#chat-surface').getAttribute('aria-busy') === 'false'", "chat idle");
  };

  await waitFor("document.readyState === 'complete'", "document load");
  assert.equal(await evaluate("document.querySelector('#chat-form')?.tagName"), "FORM");
  assert.equal(await evaluate("document.querySelector('#chat-messages')?.getAttribute('aria-live')"), "polite");
  assert.equal(await evaluate("document.querySelector('#chat-error')?.getAttribute('role')"), "alert");
  for (const invalidHint of ["profile:\0private", "x".repeat(4097)]) {
    await evaluate(`localStorage.setItem('ustc-campus-agent/opportunity-profile-id/v1', ${JSON.stringify(invalidHint)})`);
    await reloadAndWait("invalid Opportunity hint cleanup");
    assert.equal(
      await evaluate("localStorage.getItem('ustc-campus-agent/opportunity-profile-id/v1')"),
      null
    );
    assert.equal(await evaluate("document.querySelector('#chat-opportunity-confirm').disabled"), true);
  }
  await evaluate(`(() => {
    window.__ucaAssistantAdds = 0;
    new MutationObserver((records) => {
      for (const record of records) {
        for (const node of record.addedNodes) {
          if (node.nodeType === Node.ELEMENT_NODE && node.matches?.('.chat-message[data-role=assistant]')) {
            window.__ucaAssistantAdds += 1;
          }
        }
      }
    }).observe(document.querySelector('#chat-messages'), { childList: true });
  })()`);

  await evaluate(`(() => {
    const original = window.fetch.bind(window);
    let delayChatOnce = true;
    window.__ucaChatRequests = [];
    window.fetch = (...args) => {
      if (args[0] === '/api/v1/agent/chat') {
        window.__ucaChatRequests.push(JSON.parse(args[1].body));
      }
      if (delayChatOnce && args[0] === '/api/v1/agent/chat') {
        delayChatOnce = false;
        return new Promise((resolveFetch) => setTimeout(() => resolveFetch(original(...args)), 250));
      }
      return original(...args);
    };
  })()`);
  const beforeFirst = await evaluate("window.__ucaAssistantAdds");
  await evaluate(`(() => {
    const input = document.querySelector('#chat-input');
    input.value = '成绩单证明怎么办';
    input.dispatchEvent(new Event('input', { bubbles: true }));
    input.focus();
  })()`);
  await cdp.send("Input.dispatchKeyEvent", {
    type: "keyDown", key: "Enter", code: "Enter", windowsVirtualKeyCode: 13
  }, sessionId);
  await waitFor("document.querySelector('#chat-surface').getAttribute('aria-busy') === 'true'", "Affairs chat busy");
  assert.equal(await evaluate("document.querySelector('#chat-send').disabled"), true);
  assert.equal(await evaluate("document.querySelector('#chat-progress').hidden"), false);
  await cdp.send("Input.dispatchKeyEvent", {
    type: "keyUp", key: "Enter", code: "Enter", windowsVirtualKeyCode: 13
  }, sessionId);
  await waitFor(
    `window.__ucaAssistantAdds > ${beforeFirst}`,
    "Affairs assistant answer"
  );
  await waitFor("document.querySelector('#chat-surface').getAttribute('aria-busy') === 'false'", "Affairs chat idle");
  assert.equal(await evaluate("document.querySelector('.chat-tool-state')?.dataset.status"), "succeeded");
  assert.match(await evaluate("document.querySelector('.chat-tool-trace')?.textContent"), /办事导航/);
  assert.match(await evaluate("document.querySelector('.chat-message[data-role=assistant] .chat-message-body')?.textContent"), /transcript-certificate/);
  assert.equal(await evaluate("document.activeElement === document.querySelector('#chat-input')"), true);

  await submitWithEnter("校历最近有什么变更？");
  assert.match(
    await evaluate("document.querySelector('.chat-message[data-role=assistant]:last-of-type .chat-tool-trace')?.textContent"),
    /变更雷达/
  );
  assert.match(
    await evaluate("document.querySelector('.chat-message[data-role=assistant]:last-of-type .chat-message-body')?.textContent"),
    /academic-calendar/
  );

  await submitWithEnter("记录事项：提交开题报告");
  assert.match(
    await evaluate("document.querySelector('.chat-message[data-role=assistant]:last-of-type .chat-tool-trace')?.textContent"),
    /简明日历/
  );
  assert.match(
    await evaluate("document.querySelector('.chat-message[data-role=assistant]:last-of-type .chat-message-body')?.textContent"),
    /calendar:item:1|提交开题报告/
  );
  await submitWithEnter("列出我的待办事项");
  assert.match(
    await evaluate("document.querySelector('.chat-message[data-role=assistant]:last-of-type .chat-message-body')?.textContent"),
    /提交开题报告/
  );
  await submitWithEnter("删除事项 calendar:item:1");
  assert.match(
    await evaluate("document.querySelector('.chat-message[data-role=assistant]:last-of-type .chat-tool-trace')?.textContent"),
    /简明日历/
  );
  assert.match(
    await evaluate("document.querySelector('.chat-message[data-role=assistant]:last-of-type .chat-message-body')?.textContent"),
    /calendar:item:1/
  );

  await evaluate(`(() => {
    const consent = document.querySelector('#opportunity-consent');
    consent.checked = true;
    consent.dispatchEvent(new Event('change', { bubbles: true }));
    document.querySelector('#opportunity-create').click();
  })()`);
  await waitFor("document.querySelector('#chat-opportunity-confirm').disabled === false", "Opportunity profile binding");
  await submitWithEnter("请按当前档案规划课程", true);
  assert.equal(await evaluate("document.querySelector('#chat-opportunity-confirm').checked"), false);
  assert.match(await evaluate("document.querySelector('.chat-message[data-role=assistant]:last-of-type .chat-tool-trace')?.textContent"), /机会图谱/);
  assert.match(await evaluate("document.querySelector('.chat-message[data-role=assistant]:last-of-type .chat-message-body')?.textContent"), /MATH2001/);
  assert.match(await evaluate("document.querySelector('.chat-message[data-role=assistant]:last-of-type .chat-message-body')?.textContent"), /icourse\.club/);

  await submitWithEnter("普通问题 0");
  const unconfirmedHistory = await evaluate(`(() => {
    const request = window.__ucaChatRequests.at(-1);
    return {
      opportunityContext: request.opportunity_context,
      contents: request.messages.map((message) => message.content)
    };
  })()`);
  assert.equal(unconfirmedHistory.opportunityContext, null);
  assert.equal(
    unconfirmedHistory.contents.some((content) =>
      content.includes('请按当前档案规划课程') || content.includes('MATH2001')
    ),
    false,
    "unconfirmed requests must omit consent-bound Opportunity history"
  );
  for (let index = 1; index < 6; index += 1) {
    await submitWithEnter(`普通问题 ${index}`);
  }
  await submitWithEnter("普通问题 6");
  const boundedHistory = await evaluate(`(() => {
    const request = window.__ucaChatRequests.at(-1);
    return {
      count: request.messages.length,
      roles: request.messages.map((message) => message.role)
    };
  })()`);
  assert.equal(boundedHistory.count <= 12, true);
  assert.equal(boundedHistory.count % 2, 1, "history must contain complete turns plus the current user message");
  assert.deepEqual(
    boundedHistory.roles,
    boundedHistory.roles.map((_, index) => index % 2 === 0 ? "user" : "assistant"),
    "bounded history must not begin with an orphan assistant reply"
  );
  assert.equal(await evaluate("document.querySelectorAll('.chat-message').length"), 12, "DOM transcript must remain bounded");

  await cdp.send("Emulation.setDeviceMetricsOverride", {
    width: 390, height: 844, deviceScaleFactor: 1, mobile: true
  }, sessionId);
  assert.equal(await evaluate("document.documentElement.scrollWidth <= 390"), true, "390px layout must not overflow");
  await cdp.send("Emulation.setEmulatedMedia", {
    features: [{ name: "prefers-reduced-motion", value: "reduce" }]
  }, sessionId);
  const reducedMotion = await evaluate(`(() => {
    const style = getComputedStyle(document.querySelector('.progress-mark'));
    const duration = style.animationDuration.trim();
    const milliseconds = duration.endsWith('ms')
      ? Number.parseFloat(duration)
      : duration.endsWith('s')
        ? Number.parseFloat(duration) * 1000
        : Number.NaN;
    return { milliseconds, iterationCount: style.animationIterationCount };
  })()`);
  assert.equal(Number.isFinite(reducedMotion.milliseconds), true);
  assert.equal(reducedMotion.milliseconds > 0 && reducedMotion.milliseconds <= 0.011, true);
  assert.equal(reducedMotion.iterationCount, "1");

  await evaluate("document.querySelector('#chat-clear').click()");
  assert.equal(await evaluate("document.querySelectorAll('.chat-message').length"), 0);
  assert.equal(await evaluate("document.querySelector('#chat-empty').hidden"), false);
  assert.equal(await evaluate("document.activeElement === document.querySelector('#chat-input')"), true);
  assert.equal(await evaluate("document.querySelector('#chat-error').hidden"), true);

  const oversizedHistory = await evaluate(`(() => {
    commitChatTurn("bounded user", "x".repeat(4097), null);
    return createChatRequest("follow-up", null).request.messages;
  })()`);
  assert.deepEqual(
    oversizedHistory,
    [{ role: "user", content: "follow-up" }],
    "an oversized assistant answer must omit its whole prior turn rather than store a clipped prefix"
  );

  const exceptions = cdp.events.filter((event) => event.method === "Runtime.exceptionThrown");
  assert.deepEqual(exceptions, [], `browser exceptions: ${JSON.stringify(exceptions)}`);
  console.log("agent-chat-browser: PASS journeys=13 turn-pairs=PASS oversized-turn=OMITTED viewport=390 reduced-motion=PASS clear=PASS");
} catch (error) {
  console.error(error?.stack ?? error);
  if (serverOutput) console.error(`server output:\n${serverOutput}`);
  if (chromeOutput) console.error(`chrome output:\n${chromeOutput}`);
  process.exitCode = 1;
} finally {
  await stop(chrome);
  await stop(server);
  await rm(work, {
    recursive: true,
    force: true,
    maxRetries: 10,
    retryDelay: 100
  });
}
