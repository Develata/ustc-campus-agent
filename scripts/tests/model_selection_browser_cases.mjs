import assert from 'node:assert/strict';
import {mkdir, writeFile} from 'node:fs/promises';
import {join} from 'node:path';

// Real model catalog and saved turns; faults are restricted to transport/read projections.
export async function checkModelSelection({evaluate, waitFor, cdp, sessionId, navigate, pass}) {
  const ready = () => waitFor("window.UcaModelSelection?.readiness && !document.querySelector('#chat-send').disabled",'model and conversation ready');
  const click = async selector => {
    const point=await evaluate(`(()=>{const el=document.querySelector(${JSON.stringify(selector)});if(!el||el.disabled)throw Error('model control unavailable');el.scrollIntoView({block:'center'});const r=el.getBoundingClientRect(),x=r.x+r.width/2,y=r.y+r.height/2;if(!r.width||!r.height||!el.contains(document.elementFromPoint(x,y)))throw Error('model control obscured');return{x,y};})()`);
    await cdp.send('Input.dispatchMouseEvent',{type:'mousePressed',...point,button:'left',clickCount:1},sessionId);
    await cdp.send('Input.dispatchMouseEvent',{type:'mouseReleased',...point,button:'left',clickCount:1},sessionId);
  };
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
      const select=document.querySelector('#chat-model-select'),input=document.querySelector('#chat-input'),send=document.querySelector('#chat-send');
      const a=select.getBoundingClientRect(),b=input.getBoundingClientRect(),c=send.getBoundingClientRect();
      return {inComposer:!!select.closest('#chat-form'),belowInput:a.top>=b.bottom-1,sameRow:Math.abs((a.top+a.height/2)-(c.top+c.height/2))<2,
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
    assert.equal(await evaluate("document.querySelector('#chat-model-select').options.length"),catalog.models.length);
    await evaluate("window.UcaModelSelection.refresh()");await ready();
    assert.deepEqual(await evaluate('window.__modelReads'),['1']);
    await evaluate("document.querySelector('#chat-model-select').focus()");
    for(const [key,code] of [['Home',36],['End',35]]) {
      await cdp.send('Input.dispatchKeyEvent',{type:'keyDown',key,code:key,windowsVirtualKeyCode:code},sessionId);
      await cdp.send('Input.dispatchKeyEvent',{type:'keyUp',key,code:key,windowsVirtualKeyCode:code},sessionId);
    }
    const chosen=catalog.models.at(-1);
    assert.equal(chosen.provider.mode,'mock','browser service must use deterministic mock entries');
    assert.equal(await evaluate('window.UcaModelSelection.selectedId'),chosen.id);
    assert.equal(await evaluate("localStorage.getItem('uca.selected-model.v1')"),chosen.id);
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
    assert.ok(catalog.models.length >= 2,'model suite requires two configured mock entries');
    for (const allowed of [false,true]) {
      await evaluate(`(()=>{window.__modelCapabilityView=${JSON.stringify(catalog)};for(const item of window.__modelCapabilityView.models)item.tool_calling=item.id===${JSON.stringify(chosen.id)}?${allowed}:${!allowed};window.fetch=(url,options={})=>String(url)==='/api/v1/agent/models'?Promise.resolve(new Response(JSON.stringify(window.__modelCapabilityView),{status:200})):window.__modelsTrackedFetch(url,options);})()`);
      await evaluate('window.UcaModelSelection.refresh()');await ready();
      assert.equal(await evaluate('window.UcaModelSelection.toolCalling'),allowed);
      assert.equal(await evaluate('window.UcaProviderStatus.toolCalling'),allowed);
      assert.equal(await evaluate("document.querySelector('#chat-model-note').textContent.includes('仅聊天')"),!allowed);
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
    await evaluate(`(()=>{const el=document.querySelector('#chat-model-select');el.value=${JSON.stringify(chosen.id)};el.dispatchEvent(new Event('change',{bubbles:true}));})()`);await ready();await track();
    pass('MODEL-id-only-persistence-and-stale-explicit-reselection');

    await evaluate("window.fetch=(url,options)=>String(url)==='/api/v1/agent/models'?Promise.reject(Error('controlled catalog failure')):window.__modelsTrackedFetch(url,options);window.UcaModelSelection.refresh()");
    await waitFor("document.querySelector('#chat-model-note').textContent.includes('无法确认')",'catalog failure visible');
    assert.equal(await evaluate("document.querySelector('#chat-send').disabled"),true);
    assert.equal(await evaluate('window.UcaModelSelection.readiness'),false);
    await evaluate("document.querySelector('#chat-input').value='不应发送';document.querySelector('#chat-form').requestSubmit()");
    assert.equal(await evaluate('window.__modelWrites.length'),0,'failed catalog prevents programmatic submit');
    await evaluate('window.fetch=window.__modelsTrackedFetch');await click('#chat-model-refresh');await ready();
    pass('MODEL-catalog-failure-blocks-send-explicit-retry');

    // Hold a committed response, then lose it: neither busy nor uncertain turns may switch models.
    await evaluate(`window.fetch=async(url,options={})=>{const response=await window.__modelsTrackedFetch(url,options);if(String(url).endsWith('/turns')){await new Promise(resolve=>{window.__modelRelease=resolve;});throw Error('controlled committed response lost');}return response;}`);
    await send('列出我的待办事项');
    await waitFor("typeof window.__modelRelease==='function'",'committed response held');
    assert.equal(await evaluate("document.querySelector('#chat-model-select').disabled"),true);
    await evaluate("document.querySelector('#chat-model-select').value='';document.querySelector('#chat-model-select').dispatchEvent(new Event('change'));window.__modelRelease()");
    await waitFor("document.querySelector('#conversation-check-result')&&!document.querySelector('#conversation-check-result').disabled",'uncertain turn recovery visible');
    assert.equal(await evaluate('window.UcaModelSelection.selectedId'),chosen.id);
    assert.equal(await evaluate("document.querySelector('#chat-model-select').disabled"),true);
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
    assert.equal(await evaluate("document.querySelector('#chat-model-select').disabled"),false);
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

    await cdp.send('Emulation.setDeviceMetricsOverride',{width:390,height:844,deviceScaleFactor:1,mobile:true},sessionId);
    await navigate('chat');await waitFor("document.querySelector('#app-sidebar').getBoundingClientRect().right<=1",'mobile sidebar closed');
    assert.equal(await evaluate('document.documentElement.scrollWidth<=innerWidth'),true);
    await evaluate("document.querySelector('#chat-model-select').scrollIntoView({block:'center'});document.querySelector('#chat-model-select').focus()");
    assert.equal(await evaluate("document.activeElement.id"),'chat-model-select');
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