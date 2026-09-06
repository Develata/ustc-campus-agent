import assert from 'node:assert/strict';
import {mkdir, writeFile} from 'node:fs/promises';
import {join} from 'node:path';

// Runs through the real conversation API on an isolated mock-provider test server.
export async function checkConversations({evaluate, waitFor, cdp, sessionId, navigate, pass}) {
  await cdp.send('Emulation.setDeviceMetricsOverride', {width:1440,height:900,deviceScaleFactor:1,mobile:false}, sessionId);
  const click = async selector => {
    const point = await evaluate(`(() => { const el=document.querySelector(${JSON.stringify(selector)}); el.scrollIntoView({block:'center'}); const r=el.getBoundingClientRect(); const x=r.x+r.width/2,y=r.y+r.height/2; if(!r.width||!r.height||!el.contains(document.elementFromPoint(x,y))) throw Error('conversation control unreachable'); return {x,y}; })()`);
    await cdp.send('Input.dispatchMouseEvent', {type:'mousePressed',...point,button:'left',clickCount:1}, sessionId);
    await cdp.send('Input.dispatchMouseEvent', {type:'mouseReleased',...point,button:'left',clickCount:1}, sessionId);
  };
  const shot = async name => {
    if (!process.env.UCA_SHELL_SCREENSHOTS) return;
    await mkdir(process.env.UCA_SHELL_SCREENSHOTS,{recursive:true});
    const capture=await cdp.send('Page.captureScreenshot',{format:'png'},sessionId);
    await writeFile(join(process.env.UCA_SHELL_SCREENSHOTS,`${name}.png`),Buffer.from(capture.data,'base64'));
  };
  const draft = async value => evaluate(`(() => { const el=document.querySelector('#chat-input'); el.value=${JSON.stringify(value)}; el.dispatchEvent(new Event('input',{bubbles:true})); })()`);
  const send = async value => { await draft(value); await click('#chat-send'); };
  const ready = async () => waitFor("!document.querySelector('#chat-send').disabled",'conversation accepts a new turn');
  const track = async () => evaluate(`(() => {
    window.__conversationRequests=[]; window.__conversationFetch=window.fetch;
    window.fetch=(url,options={})=>{if(String(url).startsWith('/api/v1/agent/'))window.__conversationRequests.push({url:String(url),method:options.method||'GET',body:options.body||null});return window.__conversationFetch(url,options);};
    window.__conversationTrackedFetch=window.fetch;
  })()`);
  await navigate('chat'); await ready(); await click('#chat-clear'); await track();
  // Hold an older real list response across the first send. It must not erase the new row.
  await evaluate(`(() => {
    window.__releaseConversationList=null; window.__holdConversationList=true;
    window.fetch=async(url,options={})=>{
      const response=await window.__conversationTrackedFetch(url,options);
      if(String(url)==='/api/v1/agent/conversations' && (!options.method||options.method==='GET') && window.__holdConversationList){
        window.__holdConversationList=false;
        await new Promise(resolve=>{window.__releaseConversationList=resolve;});
      }
      return response;
    };
  })()`);
  await click('#conversation-refresh');
  await waitFor("typeof window.__releaseConversationList==='function'",'old list snapshot held');
  await send('查询成绩单办理流程'); await ready();
  assert.equal(await evaluate("document.querySelectorAll('.chat-message[data-role=assistant]').length"),1);
  const id=await evaluate("document.querySelector('.conversation-open[aria-current=true]').dataset.conversationId");
  const canonical=await evaluate(`window.__conversationFetch('/api/v1/agent/conversations/'+${JSON.stringify(id)},{headers:{'X-USTC-Client-Protocol-Major':'1'}}).then(response=>response.json())`);
  assert.match(canonical.title,/^[0-9]{6}\|[^\x00-\x1F\x7F-\x9F|\u2028\u2029]{1,24}$/u,'automatic title has a dated single-line topic');
  assert.equal(canonical.title.slice(7),'查询成绩单办理流程','mock uses an honest bounded fallback');
  await waitFor(`document.querySelector('.conversation-open[aria-current=true]')?.textContent===${JSON.stringify(canonical.title)}`,'first turn refreshes server-owned title without manual refresh');
  assert.equal(await evaluate("document.querySelector('#conversation-history-status').textContent"),'');
  await evaluate('window.__releaseConversationList()');
  await evaluate('new Promise(resolve=>setTimeout(resolve,0))');
  assert.equal(await evaluate("document.querySelector('.conversation-open[aria-current=true]').dataset.conversationId"),id);
  assert.equal(await evaluate("document.querySelector('.conversation-open[aria-current=true]').textContent"),canonical.title);
  assert.equal(await evaluate("document.querySelector('#conversation-history-status').textContent"),'');
  pass('CHAT-conversation-first-turn-title-and-stale-list');

  // Failure of a read after the committed result must preserve sendability and never offer retry.
  await evaluate(`window.fetch=(url,options={})=>String(url)==='/api/v1/agent/conversations'&&(!options.method||options.method==='GET')?Promise.reject(Error('controlled list read failure')):window.__conversationTrackedFetch(url,options);`);
  await send('再说明一下需要哪些材料'); await ready();
  await waitFor("document.querySelector('#conversation-history-status').textContent.includes('历史列表暂未更新')",'metadata failure remains visible');
  assert.equal(await evaluate("document.querySelector('#conversation-recovery').hidden"),true);
  assert.equal(await evaluate("Boolean(document.querySelector('#conversation-retry-turn'))"),false);
  assert.equal(await evaluate("document.querySelector('.conversation-open[aria-current=true]').textContent"),canonical.title);
  await evaluate('window.fetch=window.__conversationTrackedFetch');
  await click('#conversation-refresh');
  await waitFor("document.querySelector('#conversation-history-status').textContent===''",'manual metadata refresh recovers');
  pass('CHAT-conversation-saved-result-survives-list-failure');
  assert.equal(await evaluate("document.querySelectorAll('.chat-message[data-role=assistant]').length"),2);
  const requests=await evaluate('window.__conversationRequests');
  assert.equal(requests.filter(r=>r.url==='/api/v1/agent/chat').length,0);
  for(const request of requests.filter(r=>r.url.endsWith('/turns'))) {
    const body=JSON.parse(request.body); assert.equal(body.schema,'chat-conversation-turn/v2'); assert.equal(body.model_id,'default');
    assert.ok(body.request_id); assert.equal(Object.hasOwn(body,'messages'),false,'browser does not author canonical history');
  }
  await click('#chat-clear');
  assert.equal(await evaluate("document.querySelectorAll('.chat-message').length"),0);
  assert.ok(await evaluate(`Boolean(document.querySelector(${JSON.stringify(`.conversation-open[data-conversation-id="${id}"]`)}))`),'new conversation retains history');
  await evaluate('window.__beforeConversationReload=true');
  await cdp.send('Page.reload',{},sessionId);
  await waitFor("typeof window.__beforeConversationReload==='undefined' && document.querySelector('.conversation-open') && !document.querySelector('#chat-send').disabled",'history loaded after full page reload');
  await click(`.conversation-open[data-conversation-id="${id}"]`); await ready();
  assert.equal(await evaluate("document.querySelectorAll('.chat-message[data-role=assistant]').length"),2);
  await send('继续查询成绩单'); await ready();
  assert.equal(await evaluate("document.querySelectorAll('.chat-message[data-role=assistant]').length"),3);
  assert.equal(await evaluate("Object.values(localStorage).some(value=>value.includes('再说明一下需要哪些材料'))"),false);
  await shot('conversation-desktop-history');
  pass('CHAT-conversation-new-list-reload-select-continue');

  await track();
  // The service commits; the browser loses only the HTTP result. Recovery must be GET-only.
  await evaluate(`window.fetch=async(url,options={})=>{const result=await window.__conversationTrackedFetch(url,options);if(String(url).endsWith('/turns'))throw Error('controlled lost reply');return result;}`);
  await send('列出我的待办事项');
  await waitFor("!document.querySelector('#conversation-recovery').hidden && !document.querySelector('#conversation-check-result').disabled",'lost response recovery');
  assert.equal(await evaluate("document.querySelector('#chat-send').disabled"),true);
  assert.equal(await evaluate("document.querySelector('#chat-clear').disabled"),true);
  const before=await evaluate("window.__conversationRequests.filter(r=>r.method==='POST').length");
  await click('#conversation-check-result'); await ready();
  assert.equal(await evaluate("window.__conversationRequests.filter(r=>r.method==='POST').length"),before);
  assert.equal(await evaluate("document.querySelectorAll('.chat-message[data-role=assistant]').length"),4);
  assert.equal(await evaluate("document.querySelector('#chat-input').value"),'');
  pass('CHAT-conversation-lost-response-read-only-recovery');

  // A request lost before admission can only be explicitly retried with its original identity/body.
  await evaluate(`window.__lostTurnBody=null;window.fetch=async(url,options={})=>{if(String(url).endsWith('/turns')){window.__lostTurnBody=options.body;throw Error('controlled no delivery');}return window.__conversationTrackedFetch(url,options);}`);
  await send('查看校历变化');
  await waitFor("!document.querySelector('#conversation-recovery').hidden && !document.querySelector('#conversation-check-result').disabled",'unrecorded request recovery');
  await click('#conversation-check-result');
  await waitFor("document.querySelector('#conversation-retry-turn') && !document.querySelector('#conversation-retry-turn').disabled",'explicit exact retry available');
  await evaluate('window.fetch=window.__conversationTrackedFetch');
  await click('#conversation-retry-turn'); await ready();
  assert.equal(await evaluate("window.__conversationRequests.filter(r=>r.url.endsWith('/turns')).at(-1).body"),await evaluate('window.__lostTurnBody'));
  assert.equal(await evaluate("document.querySelectorAll('.chat-message[data-role=assistant]').length"),5);
  pass('CHAT-conversation-unrecorded-exact-explicit-retry');

  // Another tab wins the revision. Reloading preserves this tab's draft and never changes its request revision.
  await evaluate(`window.fetch=async(url,options={})=>{if(String(url).endsWith('/turns')){const body=JSON.parse(options.body);await window.__conversationTrackedFetch(url,{...options,body:JSON.stringify({...body,request_id:crypto.randomUUID(),message:'另一页面查询成绩单'})});}return window.__conversationTrackedFetch(url,options);}`);
  await send('我的待提交问题');
  await waitFor("!document.querySelector('#conversation-recovery').hidden && !document.querySelector('#conversation-check-result').disabled",'revision conflict recovery');
  await evaluate('window.fetch=window.__conversationTrackedFetch');
  await click('#conversation-check-result'); await ready();
  assert.equal(await evaluate("document.querySelector('#chat-input').value"),'我的待提交问题');
  assert.equal(await evaluate("document.querySelector('#chat-error-code').textContent"),'conversation_revision_conflict');
  assert.equal(await evaluate("document.querySelector('#chat-error').hidden"),false);
  assert.equal(await evaluate("document.querySelectorAll('.chat-message[data-role=assistant]').length"),6);
  pass('CHAT-conversation-revision-conflict-preserves-draft');

  // A terminal capacity rejection must not trap the user in a full conversation.
  await evaluate(`window.fetch=(url,options={})=>String(url).endsWith('/turns')?Promise.resolve(new Response(JSON.stringify({schema:'chat-conversation-error/v1',error:'conversation_turn_limit_reached'}),{status:429,headers:{'Content-Type':'application/json'}})):window.__conversationTrackedFetch(url,options);`);
  await send('保留这条草稿');
  await waitFor("!document.querySelector('#conversation-recovery').hidden && !document.querySelector('#conversation-check-result').disabled",'capacity rejection');
  assert.equal(await evaluate("Boolean(document.querySelector('#conversation-cancel-send'))"),false,'rejection still checks durable absence before abandon');
  await click('#conversation-check-result');
  await waitFor("document.querySelector('#conversation-cancel-send') && !document.querySelector('#conversation-cancel-send').disabled",'known rejection can be abandoned');
  await click('#conversation-cancel-send'); await ready();
  assert.equal(await evaluate("document.querySelector('#chat-input').value"),'保留这条草稿');
  assert.equal(await evaluate("document.querySelector('#chat-clear').disabled"),false);
  await evaluate('window.fetch=window.__conversationTrackedFetch');
  pass('CHAT-conversation-known-rejection-release-preserves-draft');

  await cdp.send('Emulation.setDeviceMetricsOverride',{width:390,height:844,deviceScaleFactor:1,mobile:true},sessionId);
  await evaluate("document.documentElement.dataset.theme='dark'");
  await click('#nav-toggle');
  assert.ok(await evaluate("document.querySelector('.conversation-open').getBoundingClientRect().height>=44"));
  assert.equal(await evaluate('document.documentElement.scrollWidth>innerWidth'),false);
  assert.ok(await evaluate("document.querySelector('.conversation-open').textContent.length>0"));
  await shot('conversation-mobile-dark-history');
  await click(`.conversation-open[data-conversation-id="${id}"]`); await ready();
  assert.equal(await evaluate("document.body.classList.contains('nav-open')"),false);
  pass('CHAT-conversation-mobile-dark-navigation');
  await evaluate('window.fetch=window.__conversationFetch');
}
