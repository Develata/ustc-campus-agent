import assert from 'node:assert/strict';
import {mkdir, writeFile} from 'node:fs/promises';
import {join} from 'node:path';

// Called by the existing CDP shell harness; this module owns no server or browser.
export async function checkMarketCatalog({evaluate, waitFor, cdp, sessionId, navigate, pass}) {
  await cdp.send('Emulation.setDeviceMetricsOverride',{width:1440,height:900,deviceScaleFactor:1,mobile:false},sessionId);
  await evaluate("document.documentElement.dataset.theme='light'");
  const click = async selector => {
    const point = await evaluate(`(() => {
      const el = document.querySelector(${JSON.stringify(selector)});
      el.scrollIntoView({block:'center'});
      const r = el.getBoundingClientRect(), x = r.x+r.width/2, y = r.y+r.height/2;
      if (!r.width || !r.height || !el.contains(document.elementFromPoint(x,y))) throw Error('unreachable market control');
      return {x,y};
    })()`);
    await cdp.send('Input.dispatchMouseEvent',{type:'mousePressed',...point,button:'left',clickCount:1},sessionId);
    await cdp.send('Input.dispatchMouseEvent',{type:'mouseReleased',...point,button:'left',clickCount:1},sessionId);
  };
  const input = async value => evaluate(`(() => { const el=document.querySelector('#market-search'); el.value=${JSON.stringify(value)}; el.dispatchEvent(new Event('input',{bubbles:true})); })()`);
  const shot = async name => {
    if (!process.env.UCA_SHELL_SCREENSHOTS) return;
    await mkdir(process.env.UCA_SHELL_SCREENSHOTS,{recursive:true});
    const result = await cdp.send('Page.captureScreenshot',{format:'png'},sessionId);
    await writeFile(join(process.env.UCA_SHELL_SCREENSHOTS,`${name}.png`),Buffer.from(result.data,'base64'));
  };
  await evaluate(`(() => {
    const original=window.fetch;
    window.__marketRequests=[];
    window.__marketOriginalFetch=original;
    window.fetch=(url,options={})=>{
      if (String(url).startsWith('/api/v1/market/')) window.__marketRequests.push({url:String(url),method:options.method||'GET',major:new Headers(options.headers).get('x-ustc-client-protocol-major')});
      return original(url,options);
    };
    window.__marketTrackedFetch=window.fetch;
  })()`);
  try {
    await navigate('plugins');
    assert.equal(await evaluate("document.querySelector('#plugin-tab-capabilities').getAttribute('aria-selected')"),'true');
    assert.equal(await evaluate("document.querySelector('#market-catalog').hidden"),true);
    assert.equal(await evaluate("window.__marketRequests.length"),0,'capabilities page does not fetch the catalog');
    await click('#plugin-tab-market');
    await waitFor("document.querySelectorAll('.market-package-open').length===4",'bundled catalog contains four package declarations');
    assert.equal(await evaluate("document.querySelector('#plugin-capabilities').hidden"),true);
    // Roving focus and automatic keyboard activation preserve one visible panel.
    await evaluate("document.querySelector('#plugin-tab-market').focus()");
    await cdp.send('Input.dispatchKeyEvent',{type:'keyDown',key:'ArrowLeft',code:'ArrowLeft',windowsVirtualKeyCode:37},sessionId);
    await cdp.send('Input.dispatchKeyEvent',{type:'keyUp',key:'ArrowLeft',code:'ArrowLeft',windowsVirtualKeyCode:37},sessionId);
    assert.equal(await evaluate("document.activeElement.id"),'plugin-tab-capabilities');
    assert.equal(await evaluate("document.querySelector('#market-catalog').hidden"),true);
    await cdp.send('Input.dispatchKeyEvent',{type:'keyDown',key:'ArrowRight',code:'ArrowRight',windowsVirtualKeyCode:39},sessionId);
    await cdp.send('Input.dispatchKeyEvent',{type:'keyUp',key:'ArrowRight',code:'ArrowRight',windowsVirtualKeyCode:39},sessionId);
    assert.equal(await evaluate("document.activeElement.id"),'plugin-tab-market');
    assert.equal(await evaluate("document.querySelector('#plugin-capabilities').hidden"),true);
    assert.equal(await evaluate("window.__marketRequests.length"),1,'tab switching reuses the accepted snapshot');
    // Route changes preserve the chat draft; selecting a tab never submits it.
    await evaluate("window.__marketPreviousDraft=document.querySelector('#chat-input').value;document.querySelector('#chat-input').value='保留我的选课问题'");
    await navigate('chat');
    await navigate('plugins');
    assert.equal(await evaluate("document.querySelector('#chat-input').value"),'保留我的选课问题');
    await evaluate("document.querySelector('#chat-input').value=window.__marketPreviousDraft");
    assert.match(await evaluate("document.querySelector('.market-package-open strong').textContent"),/办事导航/);
    await input('ustc.affairs-navigator');
    assert.equal(await evaluate("document.querySelectorAll('.market-package-open').length"),1);
    await input('no-such-campus-plugin');
    assert.match(await evaluate("document.querySelector('#market-status').textContent"),/没有匹配/);
    await click('#market-clear-search');
    assert.equal(await evaluate("document.activeElement.id"),'market-search');
    assert.equal(await evaluate("document.querySelectorAll('.market-package-open').length"),4);
    // Keyboard activation must open a real versioned package detail.
    await evaluate("document.querySelector('.market-package-open').focus()");
    await cdp.send('Input.dispatchKeyEvent',{type:'keyDown',key:'Enter',code:'Enter',windowsVirtualKeyCode:13,nativeVirtualKeyCode:13,text:'\r',unmodifiedText:'\r'},sessionId);
    await cdp.send('Input.dispatchKeyEvent',{type:'keyUp',key:'Enter',code:'Enter',windowsVirtualKeyCode:13},sessionId);
    try {
      await waitFor("document.querySelector('.market-provenance')",'package detail');
    } catch (error) {
      console.error(await evaluate("JSON.stringify({status:document.querySelector('#market-status').textContent,focus:document.activeElement?.outerHTML.slice(0,500),content:document.querySelector('#market-content').textContent.slice(0,1000),requests:window.__marketRequests})"));
      throw error;
    }
    assert.equal(await evaluate("document.activeElement.id"),'market-detail-title');
    await waitFor("document.querySelector('#market-catalog').getBoundingClientRect().top>=0 && document.querySelector('#market-catalog').getBoundingClientRect().top<150",'detail opens at the top of the catalog');
    assert.match(await evaluate("document.querySelector('.market-provenance').textContent"),/USTC Affairs Navigator/);
    for (const label of ['版本与发布者','MCP、Skills 与其他组件','申请的权限','资料来源与使用条件']) {
      assert.ok((await evaluate("document.querySelector('#market-content').textContent")).includes(label));
    }
    assert.match(await evaluate("document.querySelector('.market-management-note').textContent"),/已支持的 MCP \/ Skill 包.*管理 MCP 与 Skills.*其他组件的通用安装仍在开发中/);
    await shot('market-desktop-detail');
    await click('#market-back');
    await waitFor("document.activeElement.classList.contains('market-package-open')",'return restores package focus');
    pass('MARKET-readonly-search-keyboard-detail-return');

    // A GET failure offers a retry; browsing never needs lifecycle writes.
    await evaluate("window.fetch=(url,options)=>String(url)==='/api/v1/market/packages'?new Promise(resolve=>window.__marketPending=resolve):window.__marketTrackedFetch(url,options)");
    await click('#market-refresh');
    await input('ustc');
    assert.equal(await evaluate("document.querySelectorAll('.market-package-open').length"),0,'typing during refresh never restores the stale catalog');
    assert.match(await evaluate("document.querySelector('#market-status').textContent"),/正在读取/);
    await evaluate("window.__marketPending(new Response('{}',{status:503}))");
    await waitFor("document.querySelector('#market-retry')",'catalog error recovery');
    await input('calendar');
    assert.equal(await evaluate("document.querySelectorAll('.market-package-open').length"),0,'typing after a failed refresh never hides the error with stale results');
    assert.match(await evaluate("document.querySelector('#market-status').textContent"),/无法读取/);
    await input('');
    await evaluate("window.fetch=window.__marketTrackedFetch");
    await click('#market-retry');
    await waitFor("document.querySelectorAll('.market-package-open').length===4",'catalog retry reads current snapshot');
    // Accelerate only the request timeout; a transport ignoring AbortSignal must still settle.
    await evaluate(`window.__marketOriginalSetTimeout=window.setTimeout;
      window.setTimeout=(callback,delay,...args)=>window.__marketOriginalSetTimeout(callback,delay===15000?30:delay,...args);
      window.fetch=(url,options)=>String(url)==='/api/v1/market/packages'?new Promise(()=>{}):window.__marketTrackedFetch(url,options);`);
    await click('#market-refresh');
    await waitFor("document.querySelector('#market-status').textContent.includes('超时')",'bounded timeout terminates a hung catalog request');
    await evaluate("window.setTimeout=window.__marketOriginalSetTimeout;window.fetch=window.__marketTrackedFetch");
    await click('#market-retry');
    await waitFor("document.querySelectorAll('.market-package-open').length===4",'timeout recovery');
    pass('MARKET-refresh-pending-search-failure-and-timeout');
    // Cache controlled responses for adversarial transport cases, without creating domain state.
    await evaluate(`(async()=>{
      const options={headers:{'x-ustc-client-protocol-major':'1'}};
      window.__marketFixture=await (await window.__marketTrackedFetch('/api/v1/market/packages',options)).json();
      window.__marketDetails=await Promise.all(window.__marketFixture.packages.slice(0,2).map(async pkg=>
        (await window.__marketTrackedFetch('/api/v1/market/packages/'+encodeURIComponent(pkg.package_id)+'/'+encodeURIComponent(pkg.version),options)).json()));
    })()`);
    for (const mutation of [
      "data.packages=Array(65).fill(data.packages[0])",
      "data.packages[0].description='校'.repeat(1366)",
      "data.packages[0].requested_capabilities=Array(65).fill('campus.read')"
    ]) {
      await evaluate(`window.fetch=(url,options)=>{
        if (String(url)==='/api/v1/market/packages') {
          const data=structuredClone(window.__marketFixture); ${mutation};
          return Promise.resolve(new Response(JSON.stringify(data),{status:200}));
        } return window.__marketTrackedFetch(url,options);
      }`);
      await click('#market-refresh');
      await waitFor("document.querySelector('#market-retry')",'oversize catalog response rejected');
      assert.equal(await evaluate("document.querySelectorAll('.market-package-open').length"),0);
      await evaluate("window.fetch=window.__marketTrackedFetch");
      await click('#market-retry');
      await waitFor("document.querySelectorAll('.market-package-open').length===4",'valid catalog restored');
    }
    for (const mutation of [
      "data.components=Array(65).fill({kind:'SkillComponent',path:'SKILL.md',mode:null})",
      "data.source_policy=Array(33).fill({key:'source',value:'reviewed'})",
      "data.source_policy=[{key:'source',value:'校'.repeat(1366)}]"
    ]) {
      await evaluate(`window.fetch=(url,options)=>{
        if (String(url).startsWith('/api/v1/market/packages/')) {
          const data=structuredClone(window.__marketDetails[0]); ${mutation};
          return Promise.resolve(new Response(JSON.stringify(data),{status:200}));
        } return window.__marketTrackedFetch(url,options);
      }`);
      await click('.market-package-open');
      await waitFor("document.querySelector('#market-retry')",'oversize detail response rejected');
      assert.equal(await evaluate("document.querySelector('.market-provenance')!==null"),false);
      await click('#market-back');
    }
    await evaluate("window.fetch=window.__marketTrackedFetch");
    pass('MARKET-response-array-and-UTF8-size-bounds');
    // Exact catalog digest binding refuses an otherwise successful detail response.
    await evaluate(`window.fetch=(url,options)=>{
      if (String(url).startsWith('/api/v1/market/packages/')) {
        const data=structuredClone(window.__marketDetails[0]); data.catalog_digest='different-snapshot';
        return Promise.resolve(new Response(JSON.stringify(data),{status:200}));
      }
      return window.__marketTrackedFetch(url,options);
    }`);
    await click('.market-package-open');
    await waitFor("document.querySelector('#market-status').textContent.includes('目录版本已变化')",'detail digest mismatch refuses stale snapshot');
    assert.equal(await evaluate("document.querySelector('.market-provenance')!==null"),false);
    await evaluate("window.fetch=window.__marketTrackedFetch");
    await click('#market-retry');
    await waitFor("document.querySelectorAll('.market-package-open').length===4",'refresh after deployment change');
    pass('MARKET-error-retry-and-snapshot-binding');

    // Deliberately ignore AbortSignal: sequence guards must reject an old response too.
    await evaluate(`window.__marketDeferred=[];window.fetch=(url,options)=>{
      if (String(url).startsWith('/api/v1/market/packages/')) return new Promise(resolve=>window.__marketDeferred.push(resolve));
      return window.__marketTrackedFetch(url,options);
    }`);
    await click('.market-package-open');
    await click('#market-back');
    await click('.market-package:nth-child(2) .market-package-open');
    await evaluate("window.__marketDeferred[1](new Response(JSON.stringify(window.__marketDetails[1]),{status:200}))");
    await waitFor("document.querySelector('.market-provenance')",'newer selection completes first');
    const currentTitle=await evaluate("document.querySelector('#market-detail-title').textContent");
    await evaluate("window.__marketDeferred[0](new Response(JSON.stringify(window.__marketDetails[0]),{status:200}));new Promise(resolve=>setTimeout(resolve,30))");
    assert.equal(await evaluate("document.querySelector('#market-detail-title').textContent"),currentTitle);
    await click('#market-back');
    // All manifest text remains literal text, including markup-looking display names.
    await evaluate(`window.fetch=(url,options)=>{
      if (String(url)==='/api/v1/market/packages') {
        const data=structuredClone(window.__marketFixture);
        data.packages[0].display_name='<img src=x onerror="window.__marketInjection=true">';
        return Promise.resolve(new Response(JSON.stringify(data),{status:200}));
      }
      return window.__marketTrackedFetch(url,options);
    }`);
    await click('#market-refresh');
    await waitFor("document.querySelector('.market-package-open strong')?.textContent.startsWith('<img')",'manifest markup is visible literal text');
    assert.equal(await evaluate("Boolean(window.__marketInjection)||document.querySelector('#market-catalog img')!==null"),false);
    await evaluate("window.fetch=window.__marketTrackedFetch");
    await click('#market-refresh');
    await waitFor("document.querySelector('.market-package-open strong')?.textContent==='办事导航'",'original package names restored');
    pass('MARKET-stale-response-and-literal-manifest-text');

    await cdp.send('Emulation.setDeviceMetricsOverride',{width:390,height:844,deviceScaleFactor:1,mobile:true},sessionId);
    await waitFor("document.querySelector('#app-sidebar').inert && document.querySelector('#app-sidebar').getBoundingClientRect().right<=0",'mobile drawer settled');
    await evaluate("document.querySelector('#market-catalog').scrollIntoView({block:'start'})");
    assert.equal(await evaluate("document.documentElement.scrollWidth<=390 && document.querySelector('#market-catalog').scrollWidth<=document.querySelector('#market-catalog').clientWidth"),true);
    await shot('market-mobile-catalog');
    await click('.market-package-open');
    await waitFor("document.querySelector('.market-provenance')",'mobile package detail');
    await click('.market-provenance summary');
    await evaluate("document.documentElement.dataset.theme='dark'");
    assert.equal(await evaluate("document.documentElement.scrollWidth<=390 && document.querySelector('#market-content').scrollWidth<=document.querySelector('#market-content').clientWidth"),true);
    await evaluate("document.querySelector('#market-detail-title').scrollIntoView({block:'start'})");
    await shot('market-mobile-dark-detail');
    const requests=await evaluate('window.__marketRequests');
    assert.ok(requests.length>=4);
    assert.ok(requests.every(request=>request.method==='GET'&&request.major==='1'),'all catalog traffic is readonly and protocol-versioned');
    await click('#plugin-tab-capabilities');
    assert.equal(await evaluate("document.querySelector('#market-catalog').hidden"),true);
    assert.equal(await evaluate("document.querySelector('#plugin-capabilities').hidden"),false);
    assert.equal(await evaluate("document.querySelectorAll('#plugin-capabilities [data-capability-scene]').length"),4);
    pass('MARKET-responsive-light-dark-and-zero-mutation');
  } finally {
    await evaluate("window.fetch=window.__marketOriginalFetch;document.querySelector('#plugin-tab-capabilities').click();if(window.__marketPreviousDraft!==undefined)document.querySelector('#chat-input').value=window.__marketPreviousDraft;if(window.__marketOriginalSetTimeout)window.setTimeout=window.__marketOriginalSetTimeout;document.documentElement.dataset.theme='light'");
    await cdp.send('Emulation.setDeviceMetricsOverride',{width:1440,height:900,deviceScaleFactor:1,mobile:false},sessionId);
  }
}
