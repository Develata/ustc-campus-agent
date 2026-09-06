import assert from 'node:assert/strict';
import { mkdir, writeFile } from 'node:fs/promises';
import { join } from 'node:path';

// Run against the same real browser/server as the UE journeys, after their product checks.
export async function checkChatShell({ evaluate, waitFor, cdp, sessionId, navigate, click, field, pass }) {
  const metrics = async (width, height = 900) => {
    await cdp.send('Emulation.setDeviceMetricsOverride', {
      width, height, deviceScaleFactor: 1, mobile: width <= 760
    }, sessionId);
    if (width <= 760) await waitFor("document.querySelector('#app-sidebar').inert && document.querySelector('#app-sidebar').getBoundingClientRect().right <= 0", 'mobile drawer settled');
  };
  const key = async (key, code, shift = false) => {
    await cdp.send('Input.dispatchKeyEvent', { type: 'keyDown', key, code, modifiers: shift ? 8 : 0 }, sessionId);
    await cdp.send('Input.dispatchKeyEvent', { type: 'keyUp', key, code }, sessionId);
  };
  // Pointer hit-testing prevents hidden or covered controls from passing navigation tests.
  const pointer = async selector => {
    const point = await evaluate(`(() => {
      const el = document.querySelector(${JSON.stringify(selector)});
      el.scrollIntoView({block:'center'});
      const r=el.getBoundingClientRect(), x=r.x+r.width/2, y=r.y+r.height/2;
      if (!r.width || !r.height || !el.contains(document.elementFromPoint(x,y))) throw Error('unreachable control: ${selector}');
      return {x,y};
    })()`);
    await cdp.send('Input.dispatchMouseEvent', {type:'mousePressed',...point,button:'left',clickCount:1},sessionId);
    await cdp.send('Input.dispatchMouseEvent', {type:'mouseReleased',...point,button:'left',clickCount:1},sessionId);
  };
  const shot = async name => {
    if (!process.env.UCA_SHELL_SCREENSHOTS) return;
    await evaluate("Promise.allSettled(document.getAnimations().filter(a=>a.effect.getTiming().iterations !== Infinity).map(a=>a.finished))");
    await mkdir(process.env.UCA_SHELL_SCREENSHOTS, { recursive: true });
    const result = await cdp.send('Page.captureScreenshot', { format: 'png' }, sessionId);
    await writeFile(join(process.env.UCA_SHELL_SCREENSHOTS, `${name}.png`), Buffer.from(result.data, 'base64'));
  };
  await metrics(1440);
  await evaluate("document.documentElement.dataset.theme='light'");
  await navigate('chat');
  await pointer('#chat-clear');
  assert.equal(await evaluate("document.querySelectorAll('[data-view]:not([hidden])').length"), 1);
  assert.equal(await evaluate("document.querySelectorAll('#chat-view [data-scene]').length"), 0);
  assert.equal(await evaluate("document.querySelector('#chat-options').hidden"), true);
  assert.equal(await evaluate("new Set([...document.querySelectorAll('[id]')].map(e=>e.id)).size === document.querySelectorAll('[id]').length"), true, 'unique DOM ids');
  await shot('desktop-chat');
  await field('#chat-input', '保留跨页面草稿');
  await pointer('#chat-options-toggle');
  await field('#chat-prompt-customization', '请简洁回答');
  const before = await evaluate('window.__posts.length');
  await pointer('#nav-plugins');
  await waitFor("!document.querySelector('#plugins-view').hidden", 'Plugins pointer entry');
  await shot('desktop-plugins');
  await pointer('.capability-actions a[href="#plugins/affairs"]');
  await waitFor("!document.querySelector('#plugin-affairs').hidden", 'Affairs pointer entry');
  await shot('desktop-affairs');
  await evaluate('history.back()');
  await waitFor("!document.querySelector('#plugins-view').hidden", 'browser Back restores Plugins');
  await evaluate('history.forward()');
  await waitFor("!document.querySelector('#plugin-affairs').hidden", 'browser Forward restores detail');
  await pointer('#return-chat');
  await waitFor("!document.querySelector('#chat-view').hidden", 'return to draft');
  assert.equal(await evaluate("document.querySelector('#chat-input').value"), '保留跨页面草稿');
  assert.equal(await evaluate("document.querySelector('#chat-prompt-customization').value"), '请简洁回答');
  assert.equal(await evaluate('window.__posts.length'), before, 'navigation has no writes');
  pass('SHELL-navigation-draft-no-write');

  await pointer('#chat-clear');
  const capabilityPosts = await evaluate('window.__posts.length');
  await navigate('plugins');
  await pointer('[data-capability-scene="affairs"]');
  await waitFor("!document.querySelector('#chat-view').hidden && !!document.querySelector('#chat-input').value", 'capability prepares draft');
  const guardedDraft = await evaluate("document.querySelector('#chat-input').value");
  await navigate('plugins');
  await pointer('[data-capability-scene="calendar"]');
  await waitFor("!document.querySelector('#chat-view').hidden", 'guarded draft returns to chat');
  assert.equal(await evaluate("document.querySelector('#chat-input').value"), guardedDraft);
  await pointer('#chat-clear');
  const previousHint = await evaluate('opportunityProfileId');
  await evaluate('setOpportunityHint(null)');
  await navigate('plugins');
  await pointer('[data-capability-scene="planning"]');
  await waitFor("!document.querySelector('#plugin-planning').hidden && document.querySelector('#course-editor').contains(document.activeElement)", 'planning without profile focuses editor');
  await evaluate('window.UcaCourseEditor.setPending(true)');
  await navigate('plugins');
  await pointer('[data-capability-scene="planning"]');
  await waitFor("!document.querySelector('#plugin-planning').hidden && document.activeElement.id === 'opportunity-consent'", 'locked draft focuses explicit consent instead of disabled fields');
  await evaluate('window.UcaCourseEditor.setPending(false)');
  assert.equal(await evaluate('window.__posts.length'), capabilityPosts, 'capability actions do not execute tools or create profiles');
  await evaluate(`setOpportunityHint(${JSON.stringify(previousHint)})`);
  await navigate('chat');
  pass('SHELL-capability-draft-guard-and-profile-entry');

  await pointer('#chat-clear');
  const answer = '# 标题\n\n第一段 **重点** 与 *强调*\n\n第二段\n\n- 条目一\n- 条目二\n\n[来源](https://example.com/source)\n\n```html\n<img src=x onerror=alert(1)>\n```\n\n<script>window.__xss=true</script>\n\n[危险](javascript:alert(1)) [凭据](https://user:password@example.com)';
  await evaluate(`appendChatMessage('assistant', ${JSON.stringify(answer)})`);
  await waitFor("!!document.querySelector('.answer-actions button')", 'Markdown copy action');
  assert.deepEqual(await evaluate(`(() => {
    const body=document.querySelector('.chat-message-body');
    return {headings:body.querySelectorAll('h3').length, bold:body.querySelectorAll('strong').length,
      emphasis:body.querySelectorAll('em').length, items:body.querySelectorAll('li').length,
      links:[...body.querySelectorAll('a')].map(e=>[e.href,e.rel]), executable:body.querySelectorAll('script,img,iframe').length,
      code:body.querySelector('pre code').textContent, xss:!!window.__xss};
  })()`), { headings:1,bold:1,emphasis:1,items:2,links:[['https://example.com/source','noopener noreferrer']],
    executable:0,code:'<img src=x onerror=alert(1)>\n',xss:false });
  await evaluate("Object.defineProperty(navigator,'clipboard',{configurable:true,value:{writeText:async text=>{window.__copied=text}}})");
  await pointer('.answer-actions button');
  await waitFor('typeof window.__copied === "string"', 'copy resolved');
  assert.equal(await evaluate('window.__copied'), answer, 'copy preserves original Markdown and source URL');
  pass('SHELL-Markdown-safe-links-and-exact-copy');


  // DOM-only transcript growth exercises the real shell without adding server turns.
  const scrollPosts = await evaluate('window.__posts.length');
  await evaluate(`(() => {
    const messages=document.querySelector('#chat-messages'), scroll=document.querySelector('#chat-scroll');
    window.__shellScrollFixture={nodes:[...messages.childNodes],top:scroll.scrollTop};
    messages.replaceChildren(); window.dispatchEvent(new Event('uca:chat-state'));
    const add=(label,count) => {
      const item=document.createElement('li'); item.className='chat-message'; item.dataset.role='assistant';
      const body=document.createElement('div'); body.className='chat-message-body';
      for(let i=0;i<count;i++) { const line=document.createElement('p'); line.textContent=label+' '+i; body.append(line); }
      item.append(body); messages.append(item);
    };
    window.__shellAddScrollFixture=add; add('滚动回归的长回答',60);
    window.dispatchEvent(new Event('uca:chat-state'));
  })()`);
  const atTail = "(() => {const s=document.querySelector('#chat-scroll');return s.scrollHeight-s.scrollTop-s.clientHeight<80})()";
  try {
    await waitFor(atTail, 'long answer initially follows tail');
    await evaluate("document.querySelector('#chat-scroll').scrollTop=0");
    await waitFor("!document.querySelector('#chat-latest').hidden", 'manual scroll exposes latest message action');
    const readingTop = await evaluate("document.querySelector('#chat-scroll').scrollTop");
    await evaluate("window.__shellAddScrollFixture('阅读历史期间新增内容',6);window.dispatchEvent(new Event('uca:chat-state'))");
    assert.equal(await evaluate("document.querySelector('#chat-scroll').scrollTop"), readingTop, 'state updates preserve the historical reading position');
    assert.equal(await evaluate("document.querySelector('#chat-latest').hidden"), false);
    await pointer('#chat-latest');
    await waitFor(atTail, 'latest message action returns to tail');
    assert.equal(await evaluate("document.querySelector('#chat-latest').hidden"), true);

    await navigate('plugins');
    await waitFor("document.querySelector('#chat-view').hidden", 'chat hidden during background response');
    await evaluate("window.__shellAddScrollFixture('离开聊天页面后新增内容',30);window.dispatchEvent(new Event('uca:chat-state'))");
    await navigate('chat');
    await waitFor("!document.querySelector('#chat-view').hidden", 'return to background answer');
    await waitFor(`${atTail} || !document.querySelector('#chat-latest').hidden`, 'background answer remains reachable on return');
    if (!(await evaluate(atTail))) {
      await pointer('#chat-latest');
      await waitFor(atTail, 'background answer reached using latest message action');
    }
    assert.equal(await evaluate('window.__posts.length'), scrollPosts, 'scrolling and local response fixtures do not submit requests');
    pass('SHELL-scroll-history-latest-and-background-return');
  } finally {
    await navigate('chat');
    await evaluate(`(() => {
      const saved=window.__shellScrollFixture;
      document.querySelector('#chat-messages').replaceChildren(...saved.nodes);
      window.dispatchEvent(new Event('uca:chat-state'));
      document.querySelector('#chat-scroll').scrollTop=saved.top;
      document.querySelector('#chat-scroll').dispatchEvent(new Event('scroll'));
      delete window.__shellScrollFixture; delete window.__shellAddScrollFixture;
    })()`);
  }

  await pointer('#chat-clear');
  // Delay only delivery of a real response; the original endpoint still executes.
  await evaluate(`(() => {const original=window.fetch.bind(window);window.__releaseChat=null;
    window.fetch=async (url,options)=>{const r=await original(url,options);
      if(String(url).startsWith('/api/v1/agent/conversations/') && String(url).endsWith('/turns')) {window.fetch=original;await new Promise(resolve=>{window.__releaseChat=resolve});}
      return r;};})()`);
  await field('#chat-input','成绩单证明怎么办？');
  await pointer('#chat-send');
  await waitFor('typeof window.__releaseChat === "function"', 'real chat pending');
  assert.equal(await evaluate("document.querySelector('#chat-clear').disabled"),true);
  await pointer('#nav-plugins');
  await waitFor("!document.querySelector('#plugins-view').hidden",'navigate during request');
  await evaluate('window.__releaseChat()');
  await waitFor('!chatPending && !!document.querySelector(".chat-message[data-role=assistant]")','response survives navigation');
  assert.equal(await evaluate("document.querySelector('#plugins-view').hidden"),false);
  await pointer('#return-chat');
  await waitFor("!document.querySelector('#chat-view').hidden",'return to answered chat');
  assert.match(await evaluate("document.querySelector('.chat-message[data-role=assistant]').textContent"),/成绩单/);
  await shot('desktop-conversation');
  pass('SHELL-pending-navigation-preserves-response');

  // Controlled transport responses exercise recovery without causing product writes.
  await pointer('#chat-clear');
  await evaluate(`(() => {
    window.__shellOriginalFetch = window.fetch;
    window.__shellRequests = [];
    window.__shellMode = 'hold-error';
    window.__shellAnswer = '恢复后的回答';
    window.__shellConversations = new Map();
    window.fetch = async (url, options={}) => {
      const path=String(url), method=options.method||'GET';
      if (path==='/api/v1/agent/conversations' && method==='POST') {
        const response=await window.__shellOriginalFetch(url,options);
        const detail=await response.clone().json();
        window.__shellConversations.set(detail.id,detail); return response;
      }
      if (!path.startsWith('/api/v1/agent/conversations/')) return window.__shellOriginalFetch(url,options);
      const id=decodeURIComponent(path.split('/')[5]);
      const detail=window.__shellConversations.get(id);
      if (!detail) return window.__shellOriginalFetch(url,options);
      if (method==='GET') return new Response(JSON.stringify(detail),{headers:{'Content-Type':'application/json'}});
      if (!path.endsWith('/turns')) return window.__shellOriginalFetch(url,options);
      const request=JSON.parse(options.body); window.__shellRequests.push(request);
      if (window.__shellMode === 'hold-error') await new Promise(resolve => { window.__shellRelease = resolve; });
      const completed=window.__shellMode==='answer';
      const turn={request_id:request.request_id,user:request.message,phase:completed?'completed':'failed',
        response:completed?{schema:'ustc-agent-chat-response/v1',run_id:'chat-run:shell-recovery',answer:window.__shellAnswer,tool_trace:[]}:null,
        error:completed?null:'provider_timeout'};
      detail.turns.push(turn); detail.revision+=2; detail.title ||= request.message;
      return new Response(JSON.stringify({schema:'chat-conversation-turn-result/v1',conversation_id:id,
        revision:detail.revision,turn}),{headers:{'Content-Type':'application/json'}});
    };
  })()`);
  try {
    await field('#chat-input','未收到回答的原问题');
    await pointer('#chat-send');
    await waitFor('typeof window.__shellRelease === "function"', 'held error request');
    await field('#chat-input','等待期间的新草稿');
    await evaluate('window.__shellRelease()');
    await waitFor('!chatPending', 'failed request settled with next draft');
    assert.equal(await evaluate("document.querySelector('#chat-input').value"), '等待期间的新草稿');
    assert.equal(await evaluate("document.querySelector('.chat-message[data-status=failed] .chat-message-body').textContent"), '未收到回答的原问题');
    assert.match(await evaluate("document.querySelector('.conversation-turn-state').textContent"), /没有完成回答/);
    assert.equal(await evaluate("document.querySelectorAll('.chat-message[data-role=assistant]').length"), 0);
    assert.equal(await evaluate('window.__shellRequests.length'), 1, 'failure does not automatically retry');
    await shot('desktop-failed-draft');
    await evaluate("window.__shellMode='answer'");
    await pointer('#chat-send');
    await waitFor('!chatPending', 'explicit next draft request');
    assert.equal(await evaluate('window.__shellRequests.at(-1).message'),'等待期间的新草稿');
    assert.equal(await evaluate("Object.hasOwn(window.__shellRequests.at(-1),'messages')"),false,'browser submits no failed history; Rust owns prompt selection');

    // Completed failure records stay visible, within the durable 100-turn contract.
    for(let i=0;i<13;i++) {
      await evaluate("window.__shellMode='hold-error';window.__shellRelease=null");
      await field('#chat-input','失败问题 '+i); await pointer('#chat-send');
      await waitFor('typeof window.__shellRelease === "function"','held repeated failure');
      await field('#chat-input','新草稿 '+i); await evaluate('window.__shellRelease()');
      await waitFor('!chatPending','repeated failure settled');
    }
    assert.equal(await evaluate("document.querySelectorAll('.chat-message').length"),16,'complete visible records are retained instead of truncating the durable transcript');
    assert.equal(await evaluate("document.querySelectorAll('.chat-message[data-status=failed]').length"),14);
    await pointer('#chat-clear');
    assert.equal(await evaluate("document.querySelectorAll('.conversation-turn-state').length"),0);

    // A persisted failed run cannot establish whether a Calendar effect committed.
    for (const prompt of ['记录事项：待核对事项','删除事项 calendar:item:1']) {
      await evaluate("window.__shellMode='error'");
      await field('#chat-input',prompt);
      const count = await evaluate('window.__shellRequests.length');
      await pointer('#chat-send');
      await waitFor('!chatPending', 'Calendar uncertain error');
      assert.equal(await evaluate("document.querySelector('#chat-input').value"),prompt,'empty composer restores original question');
      assert.match(await evaluate("document.querySelector('#chat-error-message').textContent"),/先发送“列出我的待办事项”核对/);
      assert.equal(await evaluate('window.__shellRequests.length'),count+1,'recovery advice has no request or retry');
      assert.equal(await evaluate("document.querySelectorAll('.chat-message[data-status=failed]').length"),1);
      await pointer('#chat-clear');
    }
    pass('SHELL-failed-turn-draft-bounds-and-Calendar-recovery');

    await pointer('#chat-clear');
    const sendAnswer = async (prompt, answer) => {
      await evaluate(`window.__shellMode='answer';window.__shellAnswer=${JSON.stringify(answer)}`);
      await field('#chat-input',prompt);
      await pointer('#chat-send');
      await waitFor('!chatPending', 'controlled answer rendered');
    };
    await sendAnswer('旧话题','旧回答');
    const longAnswer='x'.repeat(4097);
    await sendAnswer('新话题长回答',longAnswer);
    assert.equal(await evaluate("document.querySelector('.chat-message[data-role=assistant]:last-of-type .chat-message-body').textContent"),longAnswer,'long answer remains fully visible');
    assert.equal(await evaluate("document.querySelector('.chat-message[data-role=assistant]:last-of-type').ucaAnswerText"),longAnswer,'copy source remains complete');
    assert.match(await evaluate("document.querySelector('.chat-context-boundary').textContent"),/不会跨过这条记录/);
    await shot('desktop-context-boundary');
    await sendAnswer('继续说明','新的回答');
    assert.equal(await evaluate('window.__shellRequests.at(-1).message'),'继续说明');
    assert.equal(await evaluate("Object.hasOwn(window.__shellRequests.at(-1),'messages')"),false,'server owns oversized history exclusion; browser cannot reconnect old history');
    await pointer('#chat-clear');
    assert.equal(await evaluate("document.querySelectorAll('.chat-context-boundary').length"),0);
    const boundaryAnswer='x'.repeat(4096);
    await sendAnswer('边界问题',boundaryAnswer);
    assert.equal(await evaluate("document.querySelectorAll('.chat-context-boundary').length"),0);
    await sendAnswer('边界追问','简短回答');
    assert.equal(await evaluate("[...window.__shellConversations.values()].at(-1).turns[0].response.answer.length"),4096,'boundary answer remains complete in canonical response projection');
    assert.equal(await evaluate('window.__shellRequests.at(-1).message'),'边界追问');
    assert.equal(await evaluate("Object.hasOwn(window.__shellRequests.at(-1),'messages')"),false,'Rust owns whether the complete 4096-byte turn enters provider context');
    pass('SHELL-long-answer-context-boundary-and-exact-limit');
  } finally {
    await evaluate('window.fetch=window.__shellOriginalFetch;delete window.__shellOriginalFetch');
    await pointer('#chat-clear');
  }

  await field('#chat-input','成绩单证明怎么办？');
  await pointer('#chat-send');
  await waitFor('!chatPending && !!document.querySelector(".chat-message[data-role=assistant]")','real conversation restored for mobile QA');

  await metrics(390,844);
  await pointer('#nav-toggle');
  await waitFor("document.querySelector('#app-sidebar').getAttribute('aria-modal') === 'true'",'modal drawer');
  await waitFor("getComputedStyle(document.querySelector('#app-sidebar')).transform === 'matrix(1, 0, 0, 1, 0, 0)'",'drawer animation');
  assert.equal(await evaluate("document.querySelector('#app-column').inert"),true);
  await evaluate("document.querySelector('#nav-settings').focus()");
  await key('Tab','Tab');
  assert.equal(await evaluate("document.activeElement.classList.contains('brand')"),true,'Tab trapped at end');
  await key('Tab','Tab',true);
  assert.equal(await evaluate("document.activeElement.id"),'nav-settings','reverse Tab trapped at start');
  await pointer('#nav-plugins');
  await waitFor("!document.querySelector('#plugins-view').hidden && !document.body.classList.contains('nav-open')", 'mobile plugin entry closes drawer');
  await pointer('#nav-toggle');
  await waitFor("getComputedStyle(document.querySelector('#app-sidebar')).transform === 'matrix(1, 0, 0, 1, 0, 0)'",'drawer reopen');
  await pointer('#nav-plugins');
  await waitFor("!document.body.classList.contains('nav-open') && !document.querySelector('#app-column').inert",'same-route dismissal');
  await pointer('#nav-toggle');
  await key('Escape','Escape');
  assert.equal(await evaluate("document.activeElement.id"),'nav-toggle');
  assert.equal(await evaluate("document.querySelector('#app-sidebar').inert"),true);
  await shot('mobile-plugins');
  await navigate('chat');
  await shot('mobile-conversation');
  await click('#chat-clear');
  await shot('mobile-chat');
  await pointer('#chat-options-toggle');
  await shot('mobile-options');
  assert.equal(await evaluate("document.querySelector('#chat-send').getBoundingClientRect().bottom < innerHeight"),true,'mobile composer reachable');
  pass('SHELL-mobile-pointer-focus-trap-same-route-Escape');

  await navigate('settings');
  await evaluate("const theme=document.querySelector('#theme-select');theme.value='dark';theme.dispatchEvent(new Event('change'))");
  assert.equal(await evaluate("localStorage.getItem('ustc-campus-agent/appearance/v1')"),'dark');
  await navigate('chat');
  await click('#chat-options-toggle');
  await shot('mobile-dark');
  await metrics(1440);
  await shot('desktop-dark');
  await navigate('settings');
  await shot('desktop-settings');
  await evaluate("document.querySelector('#theme-select').value='light';document.querySelector('#theme-select').dispatchEvent(new Event('change'))");
  pass('SHELL-theme-local-appearance');
}
