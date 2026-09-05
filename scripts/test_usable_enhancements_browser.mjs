#!/usr/bin/env node
import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import { access, mkdtemp, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { writeFile, readFile } from "node:fs/promises";

const repo = resolve(fileURLToPath(new URL("..", import.meta.url)));
const binary = resolve(process.argv[2] ?? "target/debug/ustc-agentd");
const port = 18831;
const externalBase = process.argv[2] === "--base" ? process.argv[3] : null;
if (externalBase && !/^http:\/\/127\.0\.0\.1:[0-9]+$/.test(externalBase)) throw Error("test base must be numeric loopback");
const base = externalBase ?? `http://127.0.0.1:${port}`;
const work = await mkdtemp(join(tmpdir(), "uca-agent-chat-browser-"));

if (!externalBase) await access(binary);

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
const server = externalBase ? null : spawn(binary, [
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
server?.stdout.on("data", (chunk) => { serverOutput = boundedOutput(serverOutput, chunk); });
server?.stderr.on("data", (chunk) => { serverOutput = boundedOutput(serverOutput, chunk); });

let chrome;
let cdp;
const delay = (ms) => new Promise((resolveDelay) => setTimeout(resolveDelay, ms));

async function waitForHealth() {
  for (let attempt = 0; attempt < 100; attempt += 1) {
    if (server && server.exitCode !== null) {
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
  await waitFor("document.readyState === 'complete' && !!window.UcaCourseEditor && !!window.UcaAffairsChecklist", "enhancements load");
  await waitFor("!!document.querySelector('#affairs-checklist-download') && !document.querySelector('#affairs-checklist-download').disabled", "real Affairs checklist");
  const evidence = { mode: externalBase ? "PREBUILD_ASSETS_OVER_FROZEN_API" : "COMPILED_BINARY", cases: [] };
  const pass = name => evidence.cases.push({ name, status: "PASS" });
  const click = selector => evaluate(`document.querySelector(${JSON.stringify(selector)}).click()`);
  const field = (selector, value) => evaluate(`(() => {const e=document.querySelector(${JSON.stringify(selector)});e.value=${JSON.stringify(value)};e.dispatchEvent(new Event('input',{bubbles:true}));})()`);
  await evaluate(`(() => {
    window.__posts=[]; const original=window.fetch.bind(window);
    window.fetch=async (url, options={}) => {
      if (options.method==='POST') window.__posts.push({url:String(url),body:options.body});
      const response=await original(url,options);
      if (window.__loseCreate && String(url)==='/api/v1/opportunity/profiles') {
        window.__loseCreate=false; await response.arrayBuffer(); throw new Error('test: lost response after real server commit');
      }
      return response;
    };
  })()`);
  for (const scene of ['affairs','radar','planning','calendar']) {
    await field('#chat-input','');
    await click(`[data-scene=${scene}]`);
    assert.ok(await evaluate("document.querySelector('#chat-input').value.length > 0"));
    assert.equal(await evaluate("document.querySelector('#chat-opportunity-confirm').checked"),false);
  }
  assert.equal(await evaluate('window.__posts.length'),0);
  await field('#chat-input','保留我的草稿');
  await click('[data-scene=affairs]');
  assert.equal(await evaluate("document.querySelector('#chat-input').value"),'保留我的草稿');
  await field('#chat-input','');
  await cdp.send('Page.bringToFront', {}, sessionId);
  await evaluate("document.querySelector('[data-scene=planning]').focus()");
  await cdp.send('Input.dispatchKeyEvent',{type:'keyDown',key:' ',code:'Space',windowsVirtualKeyCode:32,text:' '},sessionId);
  await cdp.send('Input.dispatchKeyEvent',{type:'keyUp',key:' ',code:'Space',windowsVirtualKeyCode:32},sessionId);
  await waitFor("!!document.activeElement.closest('#course-editor')", 'planning keyboard focus');
  assert.equal(await evaluate('window.__posts.length'),0);
  pass('UE-03-scenes-keyboard-no-effect');

  await click('#steps input[type=checkbox]');
  const markdown = await evaluate('window.UcaAffairsChecklist.markdown()');
  for (const required of ['- [x]','不是官方受理','最近核验','不确定性','https://','按顺序办理']) assert.ok(markdown.includes(required), required);
  await evaluate("Object.defineProperty(navigator, 'clipboard', {configurable:true,value:{writeText:async()=>{throw new Error('test clipboard denied')}}})");
  await click('#affairs-checklist-copy');
  await waitFor("!document.querySelector('#affairs-checklist-fallback').hidden",'clipboard fallback');
  assert.equal(await evaluate("document.querySelector('#affairs-checklist-fallback').value"),markdown);
  await cdp.send('Browser.setDownloadBehavior',{behavior:'allow',downloadPath:work},undefined);
  await click('#affairs-checklist-download');
  const downloadPath=join(work,'affairs-personal-checklist.md');
  let downloaded=false;
  for (let i=0;i<100;i++) {try {await access(downloadPath);downloaded=true;break;}catch(_){await delay(30);}}
  assert.equal(downloaded,true,'actual browser download');
  assert.equal(await readFile(downloadPath,'utf8'),markdown);
  pass('UE-02-real-checklist-copy-fallback-download');
  await field('#procedure-id','missing:procedure');
  await evaluate("document.querySelector('#lookup-form').requestSubmit()");
  assert.equal(await evaluate('window.UcaAffairsChecklist.markdown()'),null);
  await waitFor("!document.querySelector('#lookup-button').disabled",'failed lookup');
  assert.equal(await evaluate("document.querySelector('#affairs-checklist-download').disabled"),true);
  await field('#procedure-id','proc:ustc:undergraduate:transcript-certificate');
  await evaluate("document.querySelector('#lookup-form').requestSubmit()");
  await waitFor("!!window.UcaAffairsChecklist.markdown()",'fresh checklist');
  assert.equal(await evaluate("document.querySelectorAll('#steps input:checked').length"),0);
  pass('UE-02-no-stale-export-reset');

  await click('#opportunity-create');
  assert.equal(await evaluate('window.__posts.length'),0,'no create without consent');
  const create = async () => {
    const before=await evaluate('window.__posts.length');
    await evaluate("document.querySelector('#opportunity-consent').checked=true");
    await click('#opportunity-create');
    await waitFor(`window.__posts.length > ${before} && !opportunityBusy`, 'profile create terminal');
  };
  const plan = async () => {
    await click('#opportunity-plan');
    await waitFor("!opportunityBusy && !document.querySelector('#opportunity-plan-result').hidden",'Rust plan');
    return evaluate("document.querySelector('#opportunity-candidates').textContent");
  };
  await create();
  const firstId=await evaluate('opportunityProfileId');
  assert.ok(firstId,await evaluate("document.querySelector('#opportunity-status').textContent"));
  const firstPlan=await plan();
  assert.ok(firstPlan.length>0);
  await evaluate("document.querySelector('#chat-opportunity-confirm').checked=true");
  await field('#course-min-credits','1'); await field('#course-max-credits','2');
  assert.equal(await evaluate("document.querySelector('#opportunity-consent').checked"),false);
  assert.equal(await evaluate("document.querySelector('#chat-opportunity-confirm').checked"),false);
  assert.equal(await evaluate('opportunityProfileId'),firstId,'draft does not switch saved profile');
  const beforeReplace=await evaluate('window.__posts.length');
  await click('#opportunity-create');
  assert.equal(await evaluate('window.__posts.length'),beforeReplace,'no implicit profile replacement');
  await click('#opportunity-delete');
  await waitFor('!opportunityBusy && opportunityProfileId === null','explicit old profile deletion');
  await create();
  const secondId=await evaluate('opportunityProfileId');
  assert.notEqual(secondId,firstId,await evaluate("document.querySelector('#opportunity-status').textContent + JSON.stringify(window.__posts.at(-1))"));
  assert.equal(await evaluate("document.querySelector('#opportunity-plan-result').hidden"),true);
  const secondPlan=await plan();
  assert.notEqual(secondPlan,firstPlan);
  assert.ok(secondPlan.includes('没有可行计划'));
  const lastCreate=await evaluate("window.__posts.filter(x=>x.url==='/api/v1/opportunity/profiles').at(-1).body");
  assert.equal(JSON.parse(lastCreate).max_credits,2);
  pass('UE-01-real-Rust-input-dependent-plans');
  await click('#opportunity-delete');
  await waitFor('!opportunityBusy && opportunityProfileId === null','explicit second profile deletion');
  const beforeInvalid=await evaluate('window.__posts.length');
  for (const [min,max] of [['12','9'],['1.5','9'],['','9']]) {
    await field('#course-min-credits',min);await field('#course-max-credits',max);
    await evaluate("document.querySelector('#opportunity-consent').checked=true");await click('#opportunity-create');
    assert.equal(await evaluate('window.__posts.length'),beforeInvalid);
  }
  pass('UE-01-invalid-input-no-request');
  await click('.course-editor-reset');
  await evaluate('window.__loseCreate=true');
  await create();
  assert.equal(await evaluate("document.querySelector('#course-editor').dataset.pending"),'true');
  const pendingBody=await evaluate("window.__posts.filter(x=>x.url==='/api/v1/opportunity/profiles').at(-1).body");
  assert.equal(await evaluate('opportunityProfileId'),null,'lost response not optimistic success');
  await reloadAndWait('pending create reload');
  await waitFor("!!window.UcaCourseEditor && !opportunityBusy",'editor reload');
  assert.equal(await evaluate("document.querySelector('#course-editor').dataset.pending"),'true');
  await evaluate(`(() => {const original=window.fetch.bind(window);window.__posts=[];window.fetch=(url,o={})=>{if(o.method==='POST')window.__posts.push({url:String(url),body:o.body});return original(url,o);};})()`);
  await create();
  assert.equal(await evaluate("window.__posts.find(x=>x.url==='/api/v1/opportunity/profiles').body"),pendingBody);
  assert.equal(await evaluate("document.querySelector('#course-editor').dataset.pending"),'false');
  assert.equal(await evaluate("document.querySelector('#chat-opportunity-confirm').checked"),false);
  pass('UE-01-lost-response-reload-exact-retry');

  await evaluate("document.querySelector('#chat-input').value='成绩单证明怎么办？';document.querySelector('#chat-form').requestSubmit()");
  await waitFor("!!document.querySelector('.answer-actions') && !chatPending",'answer actions');
  assert.ok(await evaluate("document.querySelector('.answer-actions a').getAttribute('href') === '#hero-title'"));
  pass('UE-03-real-answer-actions');
  await evaluate("document.querySelectorAll('#course-editor details').forEach(e=>e.open=true)");
  for (const width of [320,390,1280]) {
    for (const theme of ['light','dark']) {
      await cdp.send('Emulation.setDeviceMetricsOverride',{width,height:900,deviceScaleFactor:1,mobile:false},sessionId);
      await cdp.send('Emulation.setEmulatedMedia',{features:[{name:'prefers-color-scheme',value:theme}]},sessionId);
      assert.equal(await evaluate('document.documentElement.scrollWidth <= innerWidth'),true,`${width}/${theme} overflow`);
    }
  }
  assert.deepEqual(cdp.events.filter(e=>e.method==='Runtime.exceptionThrown'),[]);
  pass('UE-03-responsive-themes-no-exceptions');
  if (process.env.UCA_TEST_SCREENSHOT) {
    await evaluate("document.querySelector('#course-editor').scrollIntoView({block:'start'})");
    const shot=await cdp.send('Page.captureScreenshot',{format:'png'},sessionId);
    await writeFile(process.env.UCA_TEST_SCREENSHOT,Buffer.from(shot.data,'base64'));
  }
  console.log(JSON.stringify(evidence,null,2));
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
