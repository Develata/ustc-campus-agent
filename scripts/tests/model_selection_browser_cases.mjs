import assert from 'node:assert/strict';
import {mkdir, writeFile} from 'node:fs/promises';
import {join} from 'node:path';

// Real model catalog and saved turns; faults are restricted to transport/read projections.
export async function checkModelSelection({evaluate, waitFor, cdp, sessionId, navigate, pass}) {
  const ready = () => waitFor("window.UcaModelSelection?.readiness && !document.querySelector('#chat-send').disabled",'model and conversation ready');
  const click = async (selector,edge=false) => {
    const point=await evaluate(`(()=>{const el=document.querySelector(${JSON.stringify(selector)});if(!el||el.disabled)throw Error('model control unavailable');el.scrollIntoView({block:'center'});const r=el.getBoundingClientRect(),x=r.x+(${edge}?12:r.width/2),y=r.y+(${edge}?12:r.height/2);if(!r.width||!r.height||!el.contains(document.elementFromPoint(x,y)))throw Error('model control obscured');return{x,y};})()`);
    await cdp.send('Input.dispatchMouseEvent',{type:'mousePressed',...point,button:'left',clickCount:1},sessionId);
    await cdp.send('Input.dispatchMouseEvent',{type:'mouseReleased',...point,button:'left',clickCount:1},sessionId);
  };
  const key=async(key,code=key,virtual=0)=>{
    await cdp.send('Input.dispatchKeyEvent',{type:'keyDown',key,code,windowsVirtualKeyCode:virtual,...(['Enter',' '].includes(key)?{text:key==='Enter'?'\r':' '}: {})},sessionId);
    await cdp.send('Input.dispatchKeyEvent',{type:'keyUp',key,code,windowsVirtualKeyCode:virtual},sessionId);
  };
  const item=id=>`.model-menu-item[data-model-id=${JSON.stringify(id)}]`;
  const opened=()=>waitFor("!document.querySelector('#chat-model-menu').hidden && document.querySelector('#chat-model-trigger').getAttribute('aria-expanded')==='true'",'model menu open');
  const closed=()=>waitFor("document.querySelector('#chat-model-menu').hidden && document.querySelector('#chat-model-trigger').getAttribute('aria-expanded')==='false'",'model menu closed');
  const choose=async id=>{await click('#chat-model-trigger');await opened();await click(item(id));await closed();};
  const shot=async name=>{
    if(!process.env.UCA_SHELL_SCREENSHOTS)return;
    await mkdir(process.env.UCA_SHELL_SCREENSHOTS,{recursive:true});
    const data=await cdp.send('Page.captureScreenshot',{format:'png'},sessionId);
    await writeFile(join(process.env.UCA_SHELL_SCREENSHOTS,`${name}.png`),Buffer.from(data.data,'base64'));
  };
  const send=async text=>{await evaluate(`(()=>{const el=document.querySelector('#chat-input');el.value=${JSON.stringify(text)};el.dispatchEvent(new Event('input',{bubbles:true}));})()`);await click('#chat-send');};
  const track=()=>evaluate(`(()=>{window.__modelsFetch=window.fetch;window.__modelWrites=[];window.__modelReads=[];window.fetch=async(url,options={})=>{if(String(url).endsWith('/turns'))window.__modelWrites.push(options.body);if(String(url)==='/api/v1/agent/models')window.__modelReads.push(new Headers(options.headers).get('X-USTC-Client-Protocol-Major'));return window.__modelsFetch(url,options);};window.__modelsTrackedFetch=window.fetch;})()`);
  const checkComposerLayout = async () => {
    const geometry = await evaluate(`(()=>{
      const trigger=document.querySelector('#chat-model-trigger'),input=document.querySelector('#chat-input'),send=document.querySelector('#chat-send');
      const a=trigger.getBoundingClientRect(),b=input.getBoundingClientRect(),c=send.getBoundingClientRect();
      return {inComposer:!!trigger.closest('#chat-form'),belowInput:a.top>=b.bottom-1,sameRow:Math.abs((a.top+a.height/2)-(c.top+c.height/2))<2,
        visible:a.left>=0&&a.right<=innerWidth&&a.top>=0&&a.bottom<=innerHeight,width:a.width,height:a.height,
        separated:a.right<=c.left,overflow:document.documentElement.scrollWidth>innerWidth};
    })()`);
    assert.equal(geometry.inComposer,true);assert.equal(geometry.belowInput,true);assert.equal(geometry.sameRow,true);
    assert.equal(geometry.visible,true);assert.equal(geometry.separated,true);assert.equal(geometry.overflow,false);
    assert.ok(geometry.width>=44&&geometry.height>=44,'model control retains a usable touch target');
  };
  await navigate('chat');await ready();await track();
  try {
    await checkComposerLayout();await shot('model-selection-empty-desktop');
    const catalog=await evaluate("window.__modelsFetch('/api/v1/agent/models',{headers:{'X-USTC-Client-Protocol-Major':'1'}}).then(response=>response.json())");
    assert.equal(catalog.schema,'uca-agent-models/v1');
    await evaluate("window.UcaModelSelection.refresh()");await ready();
    assert.deepEqual(await evaluate('window.__modelReads'),['1']);
    assert.ok(catalog.models.length >= 2,'model suite requires two configured mock entries');
    const chosen=catalog.models.at(-1),initialId=await evaluate('window.UcaModelSelection.selectedId');
    assert.equal(chosen.provider.mode,'mock','browser service must use deterministic mock entries');
    await evaluate("document.querySelector('#chat-model-trigger').focus()");await key('Enter','Enter',13);await opened();
    assert.equal(await evaluate("document.querySelector('#chat-model-trigger').getAttribute('aria-haspopup')"),'menu');
    assert.equal(await evaluate("document.querySelector('#chat-model-menu').getAttribute('role')"),'menu');
    assert.deepEqual(await evaluate("[...document.querySelectorAll('.model-menu-item')].map(el=>({id:el.dataset.modelId,role:el.getAttribute('role'),checked:el.getAttribute('aria-checked')}))"),catalog.models.map(model=>({id:model.id,role:'menuitemradio',checked:String(model.id===initialId)})));
    for(const [name,virtual,id] of [['End',35,chosen.id],['Home',36,catalog.models[0].id],['ArrowDown',40,catalog.models[1].id],['ArrowUp',38,catalog.models[0].id],['End',35,chosen.id]]) {
      await key(name,name,virtual);
      assert.equal(await evaluate('document.activeElement.dataset.modelId'),id);
      assert.equal(await evaluate('window.UcaModelSelection.selectedId'),initialId,'navigation changes focus without committing');
    }
    await key(' ','Space',32);await closed();
    assert.equal(await evaluate('window.UcaModelSelection.selectedId'),chosen.id);
    assert.equal(await evaluate("localStorage.getItem('uca.selected-model.v1')"),chosen.id);
    assert.equal(await evaluate("document.querySelector('#chat-model-label').textContent"),chosen.label);
    assert.equal(await evaluate('window.__modelWrites.length'),0,'menu keyboard interaction never submits a turn');
    await key(' ','Space',32);await opened();await key('Enter','Enter',13);await closed();
    assert.equal(await evaluate('window.UcaModelSelection.selectedId'),chosen.id,'Enter also commits the focused model');
    await click('#chat-model-trigger');await opened();await click('#chat-model-trigger');await closed();
    await key(' ','Space',32);await opened();await key('Escape','Escape',27);await closed();
    assert.equal(await evaluate('document.activeElement.id'),'chat-model-trigger','Escape restores the trigger');
    await click('#chat-model-trigger');await opened();await key('Tab','Tab',9);await closed();
    assert.equal(await evaluate('document.activeElement.id'),'chat-send','Tab continues normal composer focus order');
    await click('#chat-model-trigger');await opened();await click('#chat-input',true);await closed();
    assert.equal(await evaluate('document.activeElement.id'),'chat-input');
    await click('#chat-model-trigger');await opened();await navigate('settings');await closed();await navigate('chat');
    assert.equal(await evaluate('window.__modelWrites.length'),0,'dismissal and navigation never submit a turn');
    pass('MODEL-menu-keyboard-selection-focus-and-dismissal');
    await send('查询成绩单办理流程');await ready();
    const request=JSON.parse(await evaluate('window.__modelWrites.at(-1)'));
    assert.equal(request.schema,'chat-conversation-turn/v2');assert.equal(request.model_id,chosen.id);
    assert.equal(await evaluate("document.querySelectorAll('.chat-message[data-role=assistant]').length>0"),true);
    const saved=await evaluate("window.__modelsFetch('/api/v1/agent/conversations/'+document.querySelector('.conversation-open[aria-current=true]').dataset.conversationId,{headers:{'X-USTC-Client-Protocol-Major':'1'}}).then(response=>response.json())");
    assert.deepEqual(saved.turns.at(-1).response.provider,chosen.provider);
    await navigate('settings');
    assert.equal(await evaluate("document.querySelector('#provider-model').textContent"),`${chosen.label} · ${chosen.provider.model}`);
    await navigate('chat');await shot('model-selection-desktop');
    pass('MODEL-real-catalog-keyboard-selected-turn-and-settings');
    // Contrasting capability projections prove the chosen entry, not legacy default, owns UI state.
    for (const allowed of [false,true]) {
      await evaluate(`(()=>{window.__modelCapabilityView=${JSON.stringify(catalog)};for(const item of window.__modelCapabilityView.models)item.tool_calling=item.id===${JSON.stringify(chosen.id)}?${allowed}:${!allowed};window.fetch=(url,options={})=>String(url)==='/api/v1/agent/models'?Promise.resolve(new Response(JSON.stringify(window.__modelCapabilityView),{status:200})):window.__modelsTrackedFetch(url,options);})()`);
      await evaluate('window.UcaModelSelection.refresh()');await ready();
      assert.equal(await evaluate('window.UcaModelSelection.toolCalling'),allowed);
      assert.equal(await evaluate('window.UcaProviderStatus.toolCalling'),allowed);
      assert.equal(await evaluate("document.querySelector('#chat-model-note').textContent.includes('仅聊天')"),!allowed);
      await click('#chat-model-trigger');await opened();
      const capability=await evaluate(`document.querySelector(${JSON.stringify(item(chosen.id))}).querySelector('.model-menu-capability').textContent`);
      assert.match(capability,allowed?/Agent|工具/:/仅聊天/);
      assert.equal(await evaluate(`document.querySelector(${JSON.stringify(item(chosen.id))}).getAttribute('aria-checked')`),'true');
      await key('Escape','Escape',27);await closed();
      await evaluate(`window.dispatchEvent(new CustomEvent('uca:provider-response',{detail:${JSON.stringify(catalog.models[0].provider)}}))`);
      assert.equal(await evaluate('window.UcaModelSelection.selectedId'),chosen.id,'historical/default response never changes current selection');
    }
    await evaluate('window.fetch=window.__modelsTrackedFetch;window.UcaModelSelection.refresh()');await ready();
    pass('MODEL-selected-capability-independent-of-default-and-history');

    await evaluate("window.__modelReloadMarker=true");await cdp.send('Page.reload',{},sessionId);
    await waitFor("window.__modelReloadMarker===undefined && window.UcaModelSelection?.readiness",'selection restored after reload');await ready();
    assert.equal(await evaluate('window.UcaModelSelection.selectedId'),chosen.id);
    await track();
    // A removed saved ID must never silently choose the server default.
    await evaluate("localStorage.setItem('uca.selected-model.v1','removed-model');window.__modelReloadMarker=true");
    await cdp.send('Page.reload',{},sessionId);
    await waitFor("window.__modelReloadMarker===undefined && document.querySelector('#chat-model-note')?.textContent.includes('重新选择')",'removed selection requires explicit choice');
    assert.equal(await evaluate('window.UcaModelSelection.selectedId'),null);
    assert.equal(await evaluate("document.querySelector('#chat-send').disabled"),true);
    await choose(chosen.id);await ready();await track();
    pass('MODEL-id-only-persistence-and-stale-explicit-reselection');

    await click('#chat-model-trigger');await opened();
    await evaluate("window.fetch=(url,options)=>String(url)==='/api/v1/agent/models'?Promise.reject(Error('controlled catalog failure')):window.__modelsTrackedFetch(url,options);window.UcaModelSelection.refresh()");
    await waitFor("document.querySelector('#chat-model-note').textContent.includes('无法确认')",'catalog failure visible');await closed();
    assert.equal(await evaluate("document.querySelector('#chat-send').disabled"),true);
    assert.equal(await evaluate('window.UcaModelSelection.readiness'),false);
    await evaluate("document.querySelector('#chat-input').value='不应发送';document.querySelector('#chat-form').requestSubmit()");
    assert.equal(await evaluate('window.__modelWrites.length'),0,'failed catalog prevents programmatic submit');
    await evaluate('window.fetch=window.__modelsTrackedFetch');await click('#chat-model-refresh');await ready();
    pass('MODEL-catalog-failure-blocks-send-explicit-retry');

    // Hold a committed response, then lose it: neither busy nor uncertain turns may switch models.
    await evaluate(`window.fetch=async(url,options={})=>{const response=await window.__modelsTrackedFetch(url,options);if(String(url).endsWith('/turns')){await new Promise(resolve=>{window.__modelRelease=resolve;});throw Error('controlled committed response lost');}return response;}`);
    await evaluate("document.querySelector('#chat-input').value='列出我的待办事项';document.querySelector('#chat-input').dispatchEvent(new Event('input',{bubbles:true}))");
    await click('#chat-model-trigger');await opened();
    await evaluate("document.querySelector('#chat-form').requestSubmit()");
    await waitFor("typeof window.__modelRelease==='function'",'committed response held');
    await closed();
    assert.equal(await evaluate("[...document.querySelectorAll('.model-menu-item')].every(el=>el.disabled)"),true);
    assert.equal(await evaluate("document.querySelector('#chat-model-trigger').disabled"),true);
    await evaluate(`document.querySelector(${JSON.stringify(item(catalog.models[0].id))}).dispatchEvent(new MouseEvent('click',{bubbles:true}));document.querySelector('#chat-model-trigger').dispatchEvent(new MouseEvent('click',{bubbles:true}));window.__modelRelease()`);
    await closed();
    await waitFor("document.querySelector('#conversation-check-result')&&!document.querySelector('#conversation-check-result').disabled",'uncertain turn recovery visible');
    assert.equal(await evaluate('window.UcaModelSelection.selectedId'),chosen.id);
    assert.equal(await evaluate("document.querySelector('#chat-model-trigger').disabled"),true);
    const pendingMessages=await evaluate("document.querySelectorAll('.chat-message').length");
    await evaluate("document.querySelector('#chat-input').value='等待核对时保留的草稿';document.querySelector('#chat-input').focus()");
    await cdp.send('Input.dispatchKeyEvent',{type:'keyDown',key:'Enter',code:'Enter',windowsVirtualKeyCode:13},sessionId);
    await cdp.send('Input.dispatchKeyEvent',{type:'keyUp',key:'Enter',code:'Enter',windowsVirtualKeyCode:13},sessionId);
    await evaluate("document.querySelector('#chat-form').requestSubmit()");
    assert.equal(await evaluate("document.querySelectorAll('.chat-message').length"),pendingMessages,'uncertain Enter creates no duplicate user bubble');
    assert.equal(await evaluate("document.querySelector('#chat-input').value"),'等待核对时保留的草稿');
    assert.equal(await evaluate('window.__modelWrites.length'),1,'uncertain Enter creates no new request');
    const original=await evaluate('window.__modelWrites.at(-1)');
    assert.equal(JSON.parse(original).model_id,chosen.id);
    await evaluate('window.fetch=window.__modelsTrackedFetch');await click('#conversation-check-result');await ready();
    assert.equal(await evaluate('window.__modelWrites.length'),1,'recovery does not repeat provider execution');
    assert.equal(await evaluate("document.querySelector('#chat-model-trigger').disabled"),false);
    pass('MODEL-busy-and-uncertain-selection-freeze-read-only-recovery');

    // For an unrecorded request, only exact explicit retry may write again.
    await evaluate("window.fetch=(url,options={})=>{if(String(url).endsWith('/turns')){window.__modelWrites.push(options.body);return Promise.reject(Error('controlled unsent request'));}return window.__modelsTrackedFetch(url,options);}");
    await send('查询校历变更');await waitFor("document.querySelector('#conversation-check-result')&&!document.querySelector('#conversation-check-result').disabled",'unsent recovery visible');
    await click('#conversation-check-result');
    await waitFor("document.querySelector('#conversation-retry-turn')&&!document.querySelector('#conversation-retry-turn').disabled",'explicit original retry available');
    const failed=await evaluate('window.__modelWrites.at(-1)');
    await evaluate('window.fetch=window.__modelsTrackedFetch');await click('#conversation-retry-turn');await ready();
    assert.equal(await evaluate('window.__modelWrites.at(-1)'),failed,'retry freezes exact model ID, request ID and intent body');
    pass('MODEL-exact-explicit-retry-preserves-selection');

    // Bounded read projections exercise the full supported catalog without executing fake entries.
    const writesBeforeProjection=await evaluate('window.__modelWrites.length');
    const longLabel='campus-model-with-an-extremely-long-display-name-for-menu-validation';
    const manyCatalog={...catalog,models:Array.from({length:16},(_,index)=>({
      ...catalog.models[0],id:index===0?'default':index===1?chosen.id:`layout-${index}`,
      label:`${longLabel}-${index}`,tool_calling:index%2===0,
    }))};
    await evaluate(`window.__modelLayoutCatalog=${JSON.stringify(manyCatalog)};window.fetch=(url,options={})=>String(url)==='/api/v1/agent/models'?Promise.resolve(new Response(JSON.stringify(window.__modelLayoutCatalog),{status:200})):window.__modelsTrackedFetch(url,options);window.UcaModelSelection.refresh()`);await ready();
    for(const width of [390,320]) {
      await cdp.send('Emulation.setDeviceMetricsOverride',{width,height:844,deviceScaleFactor:1,mobile:true},sessionId);
      await checkComposerLayout();await click('#chat-model-trigger');await opened();
      assert.equal(await evaluate("document.querySelectorAll('.model-menu-item').length"),16);
      assert.equal(await evaluate("(()=>{const r=document.querySelector('#chat-model-menu').getBoundingClientRect();return r.width>0&&r.height>0&&r.left>=0&&r.right<=innerWidth&&r.top>=0&&r.bottom<=innerHeight&&document.documentElement.scrollWidth<=innerWidth;})()"),true,'full catalog menu remains inside the narrow viewport');
      assert.deepEqual(await evaluate("[...document.querySelectorAll('.model-menu-name')].map(el=>el.textContent)"),manyCatalog.models.map(model=>model.label));
      assert.equal(await evaluate("[...document.querySelectorAll('.model-menu-name')].every(el=>el.scrollWidth<=el.clientWidth+1&&el.scrollHeight<=el.clientHeight+1)"),true,'menu names wrap completely instead of truncating');
      await key('End','End',35);
      assert.equal(await evaluate('document.activeElement.dataset.modelId'),manyCatalog.models.at(-1).id);
      assert.equal(await evaluate("(()=>{const el=document.activeElement,r=el.getBoundingClientRect(),menu=document.querySelector('#chat-model-menu').getBoundingClientRect();return r.top>=menu.top&&r.bottom<=menu.bottom&&el.contains(document.elementFromPoint(r.x+r.width/2,r.y+r.height/2));})()"),true,'keyboard reaches the last option by scrolling inside the menu');
      assert.equal(await evaluate('window.UcaModelSelection.selectedId'),chosen.id,'focus on an unconfigured projection never selects it');
      await shot(`model-selection-menu-${width}`);await key('Escape','Escape',27);await closed();
    }
    await evaluate(`window.__modelLayoutCatalog=${JSON.stringify({...catalog,models:[catalog.models[0]]})};window.UcaModelSelection.refresh()`);
    await waitFor("!document.querySelector('#chat-model-trigger').disabled",'single-entry catalog loaded');
    await click('#chat-model-trigger');await opened();
    assert.equal(await evaluate("document.querySelectorAll('.model-menu-item').length"),1);
    await click(item('default'));await closed();await ready();
    assert.equal(await evaluate('window.UcaModelSelection.selectedId'),'default');
    assert.equal(await evaluate("document.querySelector('#chat-model-label').textContent"),catalog.models[0].label);
    assert.equal(await evaluate('window.__modelWrites.length'),writesBeforeProjection,'catalog presentation and selection make no model calls');
    await evaluate('window.fetch=window.__modelsTrackedFetch;window.UcaModelSelection.refresh()');await ready();await choose(chosen.id);await ready();
    pass('MODEL-full-bounded-catalog-long-names-mobile-scroll-and-single-model');

    await cdp.send('Emulation.setDeviceMetricsOverride',{width:390,height:844,deviceScaleFactor:1,mobile:true},sessionId);
    await navigate('chat');await waitFor("document.querySelector('#app-sidebar').getBoundingClientRect().right<=1",'mobile sidebar closed');
    assert.equal(await evaluate('document.documentElement.scrollWidth<=innerWidth'),true);
    await evaluate("document.querySelector('#chat-model-trigger').scrollIntoView({block:'center'});document.querySelector('#chat-model-trigger').focus()");
    assert.equal(await evaluate("document.activeElement.id"),'chat-model-trigger');
    await checkComposerLayout();
    await shot('model-selection-mobile');
    pass('MODEL-mobile-visible-selector-no-overflow');
    for (const width of [320,768,1440]) {
      await cdp.send('Emulation.setDeviceMetricsOverride',{width,height:900,deviceScaleFactor:1,mobile:width<600},sessionId);
      await checkComposerLayout();
    }
    const originalTheme=await evaluate('document.documentElement.dataset.theme');
    await evaluate("document.documentElement.dataset.theme='dark'");
    await checkComposerLayout();await shot('model-selection-dark');
    await evaluate(`document.documentElement.dataset.theme=${JSON.stringify(originalTheme)}`);
    pass('MODEL-composer-placement-small-medium-desktop-dark');
  } finally {
    await evaluate('if(window.__modelsFetch)window.fetch=window.__modelsFetch');
    await cdp.send('Emulation.setDeviceMetricsOverride',{width:1440,height:900,deviceScaleFactor:1,mobile:false},sessionId);
  }
}