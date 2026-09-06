import assert from 'node:assert/strict';
import {mkdir, writeFile} from 'node:fs/promises';
import {join} from 'node:path';

// Controlled read-side responses exercise the production controller in a real browser.
export async function checkChatActivity({evaluate, waitFor, cdp, sessionId, navigate, pass}) {
  await navigate('chat');
  await evaluate(`(() => {
    window.__activityFetch = window.fetch;
    window.__activityRoot = document.createElement('section');
    __activityRoot.id = 'activity-test-root'; __activityRoot.className = 'chat-activity';
    document.querySelector('#chat-scroll').append(__activityRoot);
    window.__activity = UcaChatActivity.mount(__activityRoot);
    window.__activityCalls = [];
    window.__activityBody = (request='r1', sequence=1, steps=[], phase='running', conversation='c1') =>
      ({schema:'chat-conversation-activity/v1',conversation_id:conversation,request_id:request,phase,sequence,steps});
    window.__activityNext = __activityBody();
    window.fetch = async (url, options={}) => {
      if (!String(url).endsWith('/activity')) return __activityFetch(url, options);
      __activityCalls.push({url:String(url),method:options.method||'GET',major:options.headers['X-USTC-Client-Protocol-Major']});
      return new Response(JSON.stringify(__activityNext), {status:200});
    };
    __activity.start('c1','r1');
  })()`);
  await waitFor("__activityRoot.dataset.state==='running'", 'observed server admission');
  await evaluate("__activityNext=__activityBody('r1',2,[{id:'m1',kind:'model',tool:null,status:'running'}])");
  await waitFor("__activityRoot.textContent.includes('正在处理模型请求')", 'real model request status');
  assert.equal(await evaluate("__activityRoot.querySelector('details').open"),false);
  await evaluate("__activityRoot.querySelector('details').open=true;__activityNext=__activityBody('r1',3,[{id:'m1',kind:'model',tool:null,status:'succeeded'},{id:'t1',kind:'tool',tool:'affairs_navigator_get',status:'running'}])");
  await waitFor("__activityRoot.querySelector('[role=status]').textContent==='查询办理流程'", 'admitted tool status');
  assert.equal(await evaluate("__activityRoot.querySelectorAll('li').length"),2);
  assert.equal(await evaluate("__activityRoot.querySelector('details').open"),true);
  assert.equal(await evaluate("__activityCalls.every(call=>call.method==='GET'&&call.major==='1')"),true);
  pass('CHAT-activity-model-tool-read-only-disclosure');

  // Stale snapshots and same-sequence modifications cannot overwrite accepted activity.
  await evaluate("__activityNext=__activityBody('r1',2,[{id:'m1',kind:'model',tool:null,status:'running'}]);window.__activitySeen=__activityCalls.length");
  await waitFor("__activityCalls.length>__activitySeen", 'stale read polled');
  assert.equal(await evaluate("__activityRoot.querySelector('[role=status]').textContent"),'查询办理流程');
  await evaluate("__activityNext=__activityBody('r1',3,[{id:'m1',kind:'model',tool:null,status:'failed'}]);__activitySeen=__activityCalls.length");
  await waitFor("__activityCalls.length>__activitySeen", 'equal sequence changed read polled');
  assert.equal(await evaluate("__activityRoot.querySelectorAll('li').length"),2);
  await evaluate("__activityNext=__activityBody('other-request',14,[{id:'m1',kind:'model',tool:null,status:'running'}]);__activitySeen=__activityCalls.length");
  await waitFor("__activityCalls.length>__activitySeen", 'other request polled');
  assert.equal(await evaluate("__activityRoot.querySelector('[role=status]').textContent"),'查询办理流程');
  pass('CHAT-activity-sequence-and-request-correlation');

  await evaluate("__activityNext={...__activityBody('r1',4),unexpected:'raw arguments'}");
  await waitFor("__activityRoot.dataset.state==='unknown'", 'closed payload rejects extra fields');
  assert.equal(await evaluate("__activityRoot.textContent.includes('raw arguments')"),false);
  await evaluate("__activityNext=__activityBody('r1',4,[{id:'t-bad',kind:'tool',tool:'<img src=x onerror=alert(1)>',status:'running'}]);__activitySeen=__activityCalls.length");
  await waitFor("__activityCalls.length>__activitySeen", 'unknown tool rejected');
  assert.equal(await evaluate("__activityRoot.querySelectorAll('img').length"),0);
  await evaluate("__activityNext=__activityBody('r1',4,Array.from({length:8},(_,i)=>({id:'m'+i,kind:'model',tool:null,status:'succeeded'})));__activitySeen=__activityCalls.length");
  await waitFor("__activityCalls.length>__activitySeen", 'bounded steps rejected');
  assert.equal(await evaluate("__activityRoot.querySelectorAll('li').length"),2);
  // Last accepted projection remains available as details, never falsely advancing.
  await evaluate("__activityNext=__activityBody('r1',3,[{id:'m1',kind:'model',tool:null,status:'succeeded'},{id:'t1',kind:'tool',tool:'affairs_navigator_get',status:'running'}])");
  await waitFor("__activityRoot.dataset.state==='running'", 'same valid snapshot recovers observation');

  await evaluate("__activityNext=__activityBody('r1',16,[]);__activitySeen=__activityCalls.length");
  await waitFor("__activityCalls.length>__activitySeen", 'out-of-range sequence polled');
  assert.equal(await evaluate("__activityRoot.querySelectorAll('li').length"),2);
  await evaluate("__activityNext=__activityBody('r1',15,[{id:'t1',kind:'tool',tool:'affairs_navigator_get',status:'succeeded'}],'completed')");
  await waitFor("__activityRoot.dataset.state==='completed'", 'canonical terminal replaces model steps');
  assert.equal(await evaluate("__activityRoot.querySelectorAll('li').length"),1);
  await evaluate("window.__activityTerminalReads=__activityCalls.length");
  await new Promise(resolve=>setTimeout(resolve,1100));
  assert.equal(await evaluate('__activityCalls.length'),await evaluate('__activityTerminalReads'),'terminal stops polling');
  for (const phase of ['failed','interrupted']) {
    await evaluate(`__activityNext=__activityBody('r-failure',1,[{id:'model-1',kind:'model',tool:null,status:'running'}]);__activity.start('c1','r-failure')`);
    await waitFor("__activityRoot.dataset.state==='running'", 'running before terminal failure');
    await evaluate(`__activityNext=__activityBody('r-failure',15,[],${JSON.stringify(phase)})`);
    await waitFor(`__activityRoot.dataset.state===${JSON.stringify(phase)}`, 'empty terminal projection');
    assert.equal(await evaluate("__activityRoot.querySelectorAll('li').length"),0);
  }
  pass('CHAT-activity-closed-bounds-failure-recovery');

  // Start another request while an old read ignores abort; generation prevents cross-talk.
  await evaluate(`window.fetch=(url,options={})=>String(url).endsWith('/activity')?new Promise(resolve=>{window.__releaseActivity=resolve;}):__activityFetch(url,options);__activity.start('c-old','r-old')`);
  await waitFor("typeof __releaseActivity==='function'", 'old read pending');
  await evaluate(`window.__oldRelease=__releaseActivity;__activity.start('c2','r2');__oldRelease(new Response(JSON.stringify(__activityBody('r-old',15,[],'completed','c-old'))));`);
  assert.equal(await evaluate("__activityRoot.querySelector('[role=status]').textContent"),'请求已发送，等待状态');
  await evaluate("__activity.stop();__releaseActivity(new Response(JSON.stringify(__activityBody('r2',15,[],'completed','c2'))))");
  await waitFor("__activityRoot.dataset.state==='unknown'", 'stop invalidates late response');
  assert.equal(await evaluate("__activityRoot.textContent.includes('检查结果')"),true);
  await evaluate("__activity.stop('settled')");
  assert.equal(await evaluate('__activityRoot.hidden'),true);
  pass('CHAT-activity-aborted-generation-and-unknown-result');

  await evaluate(`window.fetch=async(url,options={})=>String(url).endsWith('/activity')?new Response(JSON.stringify(__activityBody('r3',15,[{id:'call-1',kind:'tool',tool:'opportunity_graph_plan_current_profile',status:'denied'}],'completed','c3'))):__activityFetch(url,options);__activity.start('c3','r3')`);
  await waitFor("__activityRoot.dataset.state==='completed'", 'terminal server snapshot');
  await evaluate("__activityRoot.querySelector('details').open=true");
  for (const width of [390,1440]) for (const theme of ['light','dark']) {
    await cdp.send('Emulation.setDeviceMetricsOverride',{width,height:900,deviceScaleFactor:1,mobile:width<760},sessionId);
    await evaluate(`document.documentElement.dataset.theme=${JSON.stringify(theme)};__activityRoot.scrollIntoView({block:'center'})`);
    assert.equal(await evaluate('__activityRoot.scrollWidth<=__activityRoot.clientWidth'),true,`${width}/${theme} activity overflow`);
    if (process.env.UCA_SHELL_SCREENSHOTS) {
      await mkdir(process.env.UCA_SHELL_SCREENSHOTS,{recursive:true});
      const shot=await cdp.send('Page.captureScreenshot',{format:'png'},sessionId);
      await writeFile(join(process.env.UCA_SHELL_SCREENSHOTS,`activity-${width}-${theme}.png`),Buffer.from(shot.data,'base64'));
    }
  }
  assert.equal(await evaluate("__activityRoot.querySelector('li:last-child').textContent"),'规划课程未获准');
  await evaluate('__activity.clear();__activityRoot.remove();window.fetch=__activityFetch');
  pass('CHAT-activity-terminal-denied-responsive-themes');

  // Real conversation bridge: delay only the received POST response, not server execution.
  await evaluate(`document.querySelector('#chat-clear').click();window.__bridgeReads=0;
    window.fetch=async(url,options={})=>{
      const response=await __activityFetch(url,options);
      if(String(url).endsWith('/activity')) __bridgeReads++;
      if(options.method==='POST'&&String(url).endsWith('/turns')) {
        await new Promise(resolve=>{window.__releaseBridgePost=resolve;});
      }
      return response;
    };
    document.querySelector('#chat-input').value='列出日历事项';
    document.querySelector('#chat-form').requestSubmit();`);
  await waitFor("document.querySelector('#chat-activity').dataset.state==='completed' && typeof __releaseBridgePost==='function'", 'real bridge observes durable terminal before POST reply');
  assert.equal(await evaluate('chatPending'),true);
  assert.equal(await evaluate("document.querySelector('#chat-activity').textContent.includes('处理日历事项')"),true);
  await evaluate("document.querySelector('#chat-activity details').open=true");
  if (process.env.UCA_SHELL_SCREENSHOTS) {
    const shot=await cdp.send('Page.captureScreenshot',{format:'png'},sessionId);
    await writeFile(join(process.env.UCA_SHELL_SCREENSHOTS,'activity-real-conversation.png'),Buffer.from(shot.data,'base64'));
  }
  await evaluate('window.__bridgeTerminalReads=__bridgeReads');
  await new Promise(resolve=>setTimeout(resolve,1100));
  assert.equal(await evaluate('__bridgeReads'),await evaluate('__bridgeTerminalReads'));
  await evaluate('__releaseBridgePost()');
  await waitFor("!chatPending && document.querySelector('#chat-activity').hidden", 'settled POST clears live activity');
  assert.equal(await evaluate("document.querySelectorAll('.chat-message[data-role=assistant]').length"),1);
  await evaluate('window.fetch=__activityFetch');
  pass('CHAT-activity-real-conversation-delayed-reply-bridge');

}
