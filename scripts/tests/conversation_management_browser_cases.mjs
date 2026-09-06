import assert from 'node:assert/strict';
import {mkdir,writeFile} from 'node:fs/promises';
import {join} from 'node:path';

export async function checkConversationManagement({evaluate,waitFor,cdp,sessionId,navigate,pass}) {
  const row=id=>`.conversation-open[data-conversation-id="${id}"]`;
  const trigger=id=>`[data-conversation-menu-id="${id}"]`;
  const click=async(selector,button='left')=>{
    const p=await evaluate(`(()=>{const el=document.querySelector(${JSON.stringify(selector)});if(!el||el.disabled)throw Error('manage control unavailable');el.scrollIntoView({block:'center'});const r=el.getBoundingClientRect(),x=r.x+r.width/2,y=r.y+r.height/2;if(!r.width||!r.height||!el.contains(document.elementFromPoint(x,y)))throw Error('manage control obscured');return{x,y};})()`);
    await cdp.send('Input.dispatchMouseEvent',{type:'mousePressed',...p,button,clickCount:1},sessionId);await cdp.send('Input.dispatchMouseEvent',{type:'mouseReleased',...p,button,clickCount:1},sessionId);
  };
  const key=async(key,code=key,virtual=0,modifiers=0)=>{await cdp.send('Input.dispatchKeyEvent',{type:'keyDown',key,code,windowsVirtualKeyCode:virtual,modifiers},sessionId);await cdp.send('Input.dispatchKeyEvent',{type:'keyUp',key,code,windowsVirtualKeyCode:virtual,modifiers},sessionId);};
  const ready=()=>waitFor("!document.querySelector('#chat-send').disabled",'conversation ready after management');
  const draft=value=>evaluate(`(()=>{const el=document.querySelector('#chat-input');el.value=${JSON.stringify(value)};el.dispatchEvent(new Event('input',{bubbles:true}));})()`);
  const send=async value=>{await draft(value);await click('#chat-send');await ready();};
  const create=async()=>{await click('#chat-clear');await send('查询成绩单办理流程');return evaluate("document.querySelector('.conversation-open[aria-current=true]').dataset.conversationId");};
  const choose=async(id,kind,context=false)=>{await click(context?row(id):trigger(id),context?'right':'left');await click(`[data-conversation-menu="${kind}"]`);await waitFor("document.querySelector('.conversation-manage-dialog').open",'management dialog open');};
  const rename=async(id,title)=>{await choose(id,'rename');await evaluate(`document.querySelector('#conversation-rename-title').value=${JSON.stringify(title)}`);await click('[data-manage-dialog="confirm"]');};
  const shot=async name=>{if(!process.env.UCA_SHELL_SCREENSHOTS)return;await mkdir(process.env.UCA_SHELL_SCREENSHOTS,{recursive:true});const data=await cdp.send('Page.captureScreenshot',{format:'png'},sessionId);await writeFile(join(process.env.UCA_SHELL_SCREENSHOTS,`${name}.png`),Buffer.from(data.data,'base64'));};
  await navigate('chat');await ready();
  const first=await create(),second=await create();
  await evaluate(`(()=>{window.__manageFetch=window.fetch;window.__manageWrites=[];window.__manageTrackedFetch=(url,options={})=>{if(String(url).endsWith('/manage'))window.__manageWrites.push(options.body);return window.__manageFetch(url,options);};window.fetch=window.__manageTrackedFetch;})()`);
  try {
    await draft('保留第二段对话的草稿');
    await click(row(first),'right');await waitFor("!document.querySelector('.conversation-menu').hidden",'right-click menu');
    await key('ArrowDown','ArrowDown',40);assert.equal(await evaluate('document.activeElement.dataset.conversationMenu'),'delete');
    await key('Escape','Escape',27);assert.equal(await evaluate('document.activeElement.dataset.conversationMenuId'),first);
    await evaluate(`document.querySelector(${JSON.stringify(row(first))}).focus()`);await key('F10','F10',121,8);
    await click('[data-conversation-menu="rename"]');
    assert.equal(await evaluate("document.querySelector('#conversation-rename-title').value"),await evaluate(`document.querySelector(${JSON.stringify(row(first))}).textContent`));
    await shot('conversation-management-desktop');await click('[data-manage-dialog="cancel"]');assert.equal(await evaluate('window.__manageWrites.length'),0);
    await choose(first,'rename',true);await evaluate("document.querySelector('#conversation-rename-title').value='学'.repeat(65)");await click('[data-manage-dialog="confirm"]');
    assert.equal(await evaluate("document.querySelector('.conversation-manage-dialog').open"),true);assert.equal(await evaluate('window.__manageWrites.length'),0);await click('[data-manage-dialog="cancel"]');
    await rename(first,'  学期安排 <script>  ');await ready();
    assert.equal(await evaluate(`document.querySelector(${JSON.stringify(row(first))}).textContent`),'学期安排 <script>');
    assert.equal(await evaluate("document.querySelector('#chat-input').value"),'保留第二段对话的草稿');assert.equal(await evaluate("document.querySelector('.conversation-open[aria-current=true]').dataset.conversationId"),second);
    const initial=JSON.parse(await evaluate('window.__manageWrites[0]'));assert.equal(initial.schema,'chat-conversation-manage/v1');assert.equal(initial.action.title,'  学期安排 <script>  ');
    pass('MANAGE-right-click-keyboard-dialog-cancel-title-validation-and-other-draft');

    // Hold an actual old list snapshot across an accepted rename.
    await evaluate(`window.__manageHold=true;window.fetch=async(url,options={})=>{const response=await window.__manageTrackedFetch(url,options);if(String(url)==='/api/v1/agent/conversations'&&(!options.method||options.method==='GET')&&window.__manageHold){window.__manageHold=false;await new Promise(resolve=>{window.__manageReleaseList=resolve;});}return response;};`);
    await click('#conversation-refresh');await waitFor("typeof window.__manageReleaseList==='function'",'old list held');
    await rename(second,'当前对话的新名称');await ready();await evaluate('window.__manageReleaseList()');await evaluate('new Promise(resolve=>setTimeout(resolve,0))');
    assert.equal(await evaluate(`document.querySelector(${JSON.stringify(row(second))}).textContent`),'当前对话的新名称');assert.equal(await evaluate("document.querySelector('#chat-input').value"),'保留第二段对话的草稿');
    await evaluate('window.fetch=window.__manageTrackedFetch');pass('MANAGE-current-rename-preserves-draft-stale-list-cannot-rollback');

    await evaluate(`window.fetch=async(url,options={})=>{const response=await window.__manageTrackedFetch(url,options);if(String(url).endsWith('/manage'))throw Error('controlled committed manage response lost');return response;};`);
    await rename(first,'网络恢复后的名称');await waitFor("document.querySelector('#conversation-retry-manage')&&!document.querySelector('#conversation-retry-manage').disabled",'uncertain management');
    assert.equal(await evaluate("document.querySelector('#chat-send').disabled&&document.querySelector('#chat-model-select').disabled&&[...document.querySelectorAll('.conversation-menu-trigger')].every(el=>el.disabled)"),true);
    const lost=await evaluate('window.__manageWrites.at(-1)');await evaluate('window.fetch=window.__manageTrackedFetch');await click('#conversation-retry-manage');await ready();
    assert.equal(await evaluate('window.__manageWrites.at(-1)'),lost);assert.equal(await evaluate(`document.querySelector(${JSON.stringify(row(first))}).textContent`),'网络恢复后的名称');
    pass('MANAGE-unknown-outcome-locks-writes-exact-explicit-retry');

    // A second client deletes after the lost rename; replayed historical rename must not resurrect it.
    await evaluate(`window.fetch=async(url,options={})=>{const response=await window.__manageTrackedFetch(url,options);if(String(url).endsWith('/manage'))throw Error('controlled historical rename lost');return response;};`);
    await rename(first,'即将删除的旧名称');await waitFor("document.querySelector('#conversation-retry-manage')&&!document.querySelector('#conversation-retry-manage').disabled",'historical rename uncertain');
    await evaluate(`(async()=>{const base='/api/v1/agent/conversations/'+${JSON.stringify(first)},headers={'Content-Type':'application/json','X-USTC-Client-Protocol-Major':'1'};const detail=await window.__manageFetch(base,{headers}).then(r=>r.json());const response=await window.__manageFetch(base+'/manage',{method:'POST',headers,body:JSON.stringify({schema:'chat-conversation-manage/v1',request_id:crypto.randomUUID(),expected_revision:detail.revision,action:{kind:'delete'}})});if(!response.ok)throw Error('second client delete failed');})()`);
    await evaluate('window.fetch=window.__manageTrackedFetch');await click('#conversation-retry-manage');await ready();
    assert.equal(await evaluate(`document.querySelector(${JSON.stringify(row(first))})===null`),true);assert.equal(await evaluate("document.querySelector('#chat-input').value"),'保留第二段对话的草稿');
    await click('#conversation-refresh');await waitFor("!document.querySelector('#conversation-history-status').textContent.includes('正在读取')",'fresh list after deleted receipt');assert.equal(await evaluate(`document.querySelector(${JSON.stringify(row(first))})===null`),true);
    pass('MANAGE-historical-rename-receipt-cannot-resurrect-deleted-conversation');

    await evaluate(`window.fetch=async(url,options={})=>{const response=await window.__manageTrackedFetch(url,options);if(String(url).endsWith('/turns'))await new Promise(resolve=>{window.__manageReleaseTurn=resolve;});return response;};`);
    await draft('列出我的待办事项');await click('#chat-send');await waitFor("typeof window.__manageReleaseTurn==='function'",'chat pending');
    assert.equal(await evaluate("[...document.querySelectorAll('.conversation-menu-trigger')].every(el=>el.disabled)"),true);
    await evaluate(`document.querySelector(${JSON.stringify(row(second))}).dispatchEvent(new MouseEvent('contextmenu',{bubbles:true,cancelable:true}))`);assert.equal(await evaluate("document.querySelector('.conversation-menu').hidden"),true);
    await evaluate('window.__manageReleaseTurn();window.fetch=window.__manageTrackedFetch');await ready();pass('MANAGE-chat-pending-blocks-menu-and-management');

    const third=await create();await click(row(second));await ready();await draft('删除其他对话后仍保留');
    await evaluate(`window.fetch=(url,options={})=>{if(String(url).endsWith('/manage')){window.__manageWrites.push(options.body);return Promise.resolve(new Response(JSON.stringify({schema:'chat-conversation-error/v1',error:'conversation_capacity_exceeded'}),{status:429,headers:{'Content-Type':'application/json'}}));}return window.__manageTrackedFetch(url,options);};`);
    await rename(third,'容量拒绝不应锁死');await ready();
    assert.equal(await evaluate("document.querySelector('#conversation-retry-manage')===null"),true);
    assert.equal(await evaluate("document.querySelector('#chat-input').value"),'删除其他对话后仍保留');
    assert.equal(await evaluate(`document.querySelector(${JSON.stringify(trigger(third))}).disabled`),false,'known capacity rejection leaves deletion reachable');
    await evaluate('window.fetch=window.__manageTrackedFetch');pass('MANAGE-known-capacity-rejection-unlocks-and-preserves-draft');
    await choose(third,'delete');assert.match(await evaluate("document.querySelector('#conversation-manage-explanation').textContent"),/服务器保留原始记录.*不会撤销/);
    await evaluate("document.documentElement.dataset.theme='dark'");
    const deleteColors=await evaluate("(()=>{const s=getComputedStyle(document.querySelector('.conversation-manage-dialog [data-manage-dialog=confirm]'));return {text:s.color,background:s.backgroundColor};})()");
    assert.notEqual(deleteColors.text,deleteColors.background);assert.notEqual(deleteColors.text,'rgb(255, 255, 255)');
    await shot('conversation-management-delete-dark');await evaluate("document.documentElement.dataset.theme='light'");
    const beforeCancel=await evaluate('window.__manageWrites.length');await click('[data-manage-dialog="cancel"]');assert.equal(await evaluate('window.__manageWrites.length'),beforeCancel);
    await choose(third,'delete');await click('[data-manage-dialog="confirm"]');await ready();
    assert.equal(await evaluate(`document.querySelector(${JSON.stringify(row(third))})===null`),true);assert.equal(await evaluate("document.querySelector('#chat-input').value"),'删除其他对话后仍保留');
    await choose(second,'delete');await click('[data-manage-dialog="confirm"]');await ready();
    assert.equal(await evaluate("document.querySelector('#chat-input').value"),'');assert.equal(await evaluate("document.querySelectorAll('.chat-message').length"),0);
    await evaluate('window.__manageReload=true');await cdp.send('Page.reload',{},sessionId);await waitFor("window.__manageReload===undefined&&!document.querySelector('#chat-send').disabled",'reload management persistence');
    for(const id of [first,second,third])assert.equal(await evaluate(`document.querySelector(${JSON.stringify(row(id))})===null`),true);
    pass('MANAGE-delete-cancel-other-preserves-draft-current-clears-and-reload');

    // Another client can delete before submission or after a committed response is lost.
    await evaluate(`window.__manageExternalFetch=window.fetch;window.__manageTurnCount=0;window.__manageCountedFetch=(url,options={})=>{if(String(url).endsWith('/turns'))window.__manageTurnCount++;return window.__manageExternalFetch(url,options);};window.fetch=window.__manageCountedFetch;`);
    for(const afterCommit of [false,true]){
      const removed=await create();
      const removeExternally=()=>evaluate(`(async()=>{const base='/api/v1/agent/conversations/'+${JSON.stringify(removed)},headers={'Content-Type':'application/json','X-USTC-Client-Protocol-Major':'1'};const detail=await window.__manageExternalFetch(base,{headers}).then(r=>r.json());const result=await window.__manageExternalFetch(base+'/manage',{method:'POST',headers,body:JSON.stringify({schema:'chat-conversation-manage/v1',request_id:crypto.randomUUID(),expected_revision:detail.revision,action:{kind:'delete'}})});if(!result.ok)throw Error('external delete failed');})()`);
      if(afterCommit)await evaluate(`window.fetch=async(url,options={})=>{const response=await window.__manageCountedFetch(url,options);if(String(url).endsWith('/turns'))throw Error('controlled turn committed then response lost');return response;};`);
      else await removeExternally();
      await draft('跨页面删除后保留这条草稿');await click('#chat-send');
      if(afterCommit){await waitFor("document.querySelector('#conversation-check-result')&&!document.querySelector('#conversation-check-result').disabled",'lost turn awaiting recovery');await removeExternally();await evaluate('window.fetch=window.__manageCountedFetch');await click('#conversation-check-result');}
      await waitFor("document.querySelector('#conversation-detach-unavailable')&&!document.querySelector('#conversation-detach-unavailable').disabled",'inaccessible conversation offers explicit safe exit');
      assert.equal(await evaluate("document.querySelector('#conversation-recovery').textContent.includes('先前请求可能已执行')"),true);
      const turnCount=await evaluate('window.__manageTurnCount');await click('#conversation-detach-unavailable');await ready();
      assert.equal(await evaluate("document.querySelector('#chat-input').value"),'跨页面删除后保留这条草稿');assert.equal(await evaluate("document.querySelectorAll('.chat-message').length"),0);
      assert.equal(await evaluate('window.__manageTurnCount'),turnCount,'explicit detach never resends');assert.equal(await evaluate(`document.querySelector(${JSON.stringify(row(removed))})===null`),true);
    }
    await evaluate('window.fetch=window.__manageExternalFetch');
    pass('MANAGE-cross-client-delete-before-submit-and-after-commit-preserves-draft');
    const mobile=await create();await cdp.send('Emulation.setDeviceMetricsOverride',{width:390,height:844,deviceScaleFactor:1,mobile:true},sessionId);await navigate('chat');
    await waitFor("document.querySelector('#app-sidebar').getBoundingClientRect().right<=1",'mobile drawer initially closed');await click('#nav-toggle');await waitFor("document.querySelector('#app-sidebar').getBoundingClientRect().left>=0",'mobile drawer fully open');
    await click(trigger(mobile));
    assert.equal(await evaluate("(()=>{const r=document.querySelector('.conversation-menu').getBoundingClientRect();return r.left>=0&&r.right<=innerWidth&&r.top>=0&&r.bottom<=innerHeight;})()"),true);
    await shot('conversation-management-mobile');await click('[data-conversation-menu="rename"]');assert.equal(await evaluate('document.activeElement.id'),'conversation-rename-title');
    await key('Escape','Escape',27);assert.equal(await evaluate("document.querySelector('.conversation-manage-dialog').open"),false);await waitFor(`document.activeElement.dataset.conversationMenuId===${JSON.stringify(mobile)}`,'dialog close event restores menu trigger focus');
    assert.equal(await evaluate('document.documentElement.scrollWidth<=innerWidth'),true);pass('MANAGE-mobile-menu-placement-dialog-Escape-focus-return');
  } finally {
    await evaluate('if(window.__manageFetch)window.fetch=window.__manageFetch');await cdp.send('Emulation.setDeviceMetricsOverride',{width:1440,height:900,deviceScaleFactor:1,mobile:false},sessionId);
  }
}