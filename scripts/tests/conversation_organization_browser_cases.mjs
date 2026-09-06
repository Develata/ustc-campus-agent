import assert from 'node:assert/strict';

// Real owner-bound endpoints. Interception below only corrupts a committed receipt.
export async function checkConversationOrganization({evaluate,waitFor,navigate,click,field,pass,cdp,sessionId}) {
  const row=id=>`.conversation-open[data-conversation-id="${id}"]`;
  const trigger=id=>`[data-conversation-menu-id="${id}"]`;
  const ready=()=>waitFor("!document.querySelector('#chat-send').disabled",'conversation organization ready');
  const refresh=async()=>{await click('#conversation-refresh');await waitFor("!document.querySelector('#conversation-history-status').textContent.includes('正在读取')",'organization list read');};
  const choose=async(id,kind)=>{await click(trigger(id));await click(`[data-conversation-menu="${kind}"]`);};
  const rename=async(id,title)=>{await choose(id,'rename');await field('#conversation-rename-title',title);await click('[data-manage-dialog="confirm"]');await ready();};
  const section=id=>evaluate(`document.querySelector(${JSON.stringify(row(id))}).closest('[data-conversation-section]').dataset.conversationSection`);
  const group=id=>evaluate(`document.querySelector(${JSON.stringify(row(id))}).closest('[data-conversation-section]').dataset.conversationGroup??null`);
  const assign=async(id,name)=>{await choose(id,'group');await field('#conversation-group-name',name);await click('[data-manage-dialog="confirm"]');await ready();};
  const visibleOrder=ids=>evaluate(`[...document.querySelectorAll('.conversation-open')].map(e=>e.dataset.conversationId).filter(id=>${JSON.stringify(ids)}.includes(id))`);
  await navigate('chat');await ready();
  const created=await evaluate(`(async()=>{
    const headers={'Content-Type':'application/json','X-USTC-Client-Protocol-Major':'1'},result=[];
    for(let i=0;i<3;i++){
      const response=await fetch('/api/v1/agent/conversations',{method:'POST',headers,body:JSON.stringify({schema:'chat-conversation-create/v1',request_id:crypto.randomUUID()})});
      if(!response.ok)throw Error('organization fixture create failed');result.push(await response.json());
    }
    return result;
  })()`);
  const ids=created.map(entry=>entry.id),[first,second,third]=ids;
  try {
    await refresh();
    for(const entry of created)assert.match(entry.organization.date,/^\d{6}$/);
    await rename(first,'组织验收一');await rename(second,'组织验收二');await rename(third,'组织验收三');
    assert.deepEqual(await visibleOrder(ids),[third,second,first],'rename must not reorder same-day creation ties');
    await click(row(first));await ready();
    assert.deepEqual(await visibleOrder(ids),[third,second,first],'opening a conversation must not bump it');
    await choose(first,'rename');
    assert.equal(await evaluate("document.querySelector('#conversation-rename-title').value"),'组织验收一');
    assert.equal(await evaluate("document.querySelector('#conversation-rename-prefix').textContent"),created[0].organization.date+'|');
    assert.equal(await evaluate("document.querySelector('#conversation-rename-prefix').querySelector('input')===null"),true);
    await field('#conversation-rename-title','260101|改日期');await click('[data-manage-dialog="confirm"]');
    assert.equal(await evaluate("document.querySelector('.conversation-manage-dialog').open"),true);
    assert.match(await evaluate("document.querySelector('.conversation-manage-error').textContent"),/不能包含/);
    await field('#conversation-rename-title','组织验收 <img src=x>');await click('[data-manage-dialog="confirm"]');await ready();
    assert.equal(await evaluate(`document.querySelector(${JSON.stringify(row(first))}).textContent`),created[0].organization.date+'|组织验收 <img src=x>');
    assert.equal(await evaluate(`document.querySelector(${JSON.stringify(row(first))}).querySelector('img')===null`),true);
    pass('ORGANIZE-topic-only-rename-keeps-date-escapes-text-and-preserves-creation-order');

    await assign(first,'B 学习');await assign(second,'A 校园');
    assert.equal(await group(first),'B 学习');assert.equal(await group(second),'A 校园');
    const groupNames=await evaluate("[...document.querySelectorAll('[data-conversation-section=group]')].map(e=>e.dataset.conversationGroup)");
    assert.deepEqual(groupNames,[...groupNames].sort());
    await choose(third,'group');
    assert.deepEqual(await evaluate("[...document.querySelectorAll('#conversation-group-options option')].map(e=>e.value).filter(value=>['A 校园','B 学习'].includes(value))"),['A 校园','B 学习']);
    await field('#conversation-group-name','B 学习');await click('[data-manage-dialog="confirm"]');await ready();
    assert.deepEqual(await visibleOrder([first,third]),[third,first]);
    pass('ORGANIZE-create-and-reuse-flat-groups-sorted-by-name-with-date-ordered-contents');

    await choose(first,'pin');await ready();assert.equal(await section(first),'pinned');
    assert.equal(await evaluate(`document.querySelectorAll(${JSON.stringify(row(first))}).length`),1);
    await choose(first,'pin');await ready();assert.equal(await group(first),'B 学习');
    await choose(first,'ungroup');await ready();assert.equal(await section(first),'ungrouped');
    pass('ORGANIZE-pin-is-unique-unpin-restores-group-and-ungroup-is-explicit');

    await evaluate(`(()=>{
      window.__organizationFetch=window.fetch;window.__organizationWrites=[];window.__organizationCorrupt=true;
      window.fetch=async(url,options={})=>{
        const write=String(url).endsWith('/manage');if(write)window.__organizationWrites.push(options.body);
        const response=await window.__organizationFetch(url,options);
        if(write&&window.__organizationCorrupt&&response.ok){window.__organizationCorrupt=false;const value=await response.json();value.organization.pinned=!value.organization.pinned;return new Response(JSON.stringify(value),{status:200,headers:{'Content-Type':'application/json'}});}
        return response;
      };
    })()`);
    await choose(first,'pin');
    await waitFor("document.querySelector('#conversation-retry-manage')&&!document.querySelector('#conversation-retry-manage').disabled",'inconsistent organization receipt rejected');
    assert.equal(await section(first),'ungrouped');
    assert.equal(await evaluate("document.querySelector('#chat-send').disabled&&[...document.querySelectorAll('.conversation-menu-trigger')].every(e=>e.disabled)"),true);
    const originalBody=await evaluate('window.__organizationWrites[0]');
    assert.equal(JSON.parse(originalBody).schema,'chat-conversation-manage/v2');
    await click('#conversation-retry-manage');await ready();
    assert.equal(await evaluate('window.__organizationWrites[1]'),originalBody);
    assert.equal(await section(first),'pinned');
    await evaluate('window.fetch=window.__organizationFetch');
    pass('ORGANIZE-mismatched-receipt-locks-writes-and-exact-retry-recovers-pin');

    await evaluate('window.__organizationReload=true');await cdp.send('Page.reload',{},sessionId);
    await waitFor(`window.__organizationReload===undefined&&!document.querySelector('#chat-send').disabled&&document.querySelector(${JSON.stringify(row(first))})`, 'organization persistence after reload');
    assert.equal(await section(first),'pinned');assert.equal(await group(second),'A 校园');assert.equal(await group(third),'B 学习');
    await choose(first,'pin');await ready();
    await assign(second,'');assert.equal(await section(second),'ungrouped');
    pass('ORGANIZE-pin-and-group-persist-across-reload-empty-name-removes-group');

    await cdp.send('Emulation.setDeviceMetricsOverride',{width:390,height:844,deviceScaleFactor:1,mobile:true},sessionId);
    await navigate('chat');await click('#nav-toggle');
    await waitFor("document.querySelector('#app-sidebar').getBoundingClientRect().left>=0",'mobile organization drawer open');
    await choose(third,'group');
    assert.equal(await evaluate('document.activeElement.id'),'conversation-group-name');
    assert.equal(await evaluate('document.documentElement.scrollWidth<=innerWidth'),true);
    assert.equal(await evaluate("[...document.querySelectorAll('.conversation-manage-dialog button')].every(e=>e.getBoundingClientRect().height>=44)"),true);
    await click('[data-manage-dialog="cancel"]');
    pass('ORGANIZE-mobile-group-dialog-focus-and-touch-targets');
  } finally {
    await cdp.send('Emulation.setDeviceMetricsOverride',{width:1440,height:900,deviceScaleFactor:1,mobile:false},sessionId);
    await evaluate(`(async()=>{
      if(window.__organizationFetch)window.fetch=window.__organizationFetch;
      const headers={'Content-Type':'application/json','X-USTC-Client-Protocol-Major':'1'};
      for(const id of ${JSON.stringify(ids)}){
        const url='/api/v1/agent/conversations/'+id,response=await fetch(url,{headers});if(response.status===404)continue;
        if(!response.ok)throw Error('organization cleanup read failed');const detail=await response.json();
        const removed=await fetch(url+'/manage',{method:'POST',headers,body:JSON.stringify({schema:'chat-conversation-manage/v2',request_id:crypto.randomUUID(),expected_revision:detail.revision,action:{kind:'delete'}})});
        if(!removed.ok)throw Error('organization fixture cleanup failed');
      }
    })()`);
    await refresh();
  }
}
