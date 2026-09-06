import assert from 'node:assert/strict';
import {mkdir, writeFile} from 'node:fs/promises';
import {join} from 'node:path';

// Real lifecycle against an isolated fresh server. Reuses the existing CDP harness.
export async function checkPluginManagement({evaluate, waitFor, cdp, sessionId, navigate, pass}) {
  const scope = '.plugin-manage-card[data-package-id="ustc.campus-guide"]';
  const orphan = scope + '[data-available="false"]';
  const orphanAction = name => `${orphan} [data-plugin-action="${name}"]`;
  const action = name => `${scope} [data-plugin-action="${name}"]`;
  const click = async selector => {
    const point = await evaluate(`(()=>{const el=document.querySelector(${JSON.stringify(selector)});if(!el||el.disabled)throw Error('plugin control unavailable');el.scrollIntoView({block:'center'});const r=el.getBoundingClientRect(),x=r.x+r.width/2,y=r.y+r.height/2;if(!r.width||!r.height||!el.contains(document.elementFromPoint(x,y)))throw Error('plugin control obscured');return{x,y};})()`);
    await cdp.send('Input.dispatchMouseEvent',{type:'mousePressed',...point,button:'left',clickCount:1},sessionId);
    await cdp.send('Input.dispatchMouseEvent',{type:'mouseReleased',...point,button:'left',clickCount:1},sessionId);
  };
  const shot = async name => {
    if (!process.env.UCA_SHELL_SCREENSHOTS) return;
    await mkdir(process.env.UCA_SHELL_SCREENSHOTS,{recursive:true});
    const image=await cdp.send('Page.captureScreenshot',{format:'png'},sessionId);
    await writeFile(join(process.env.UCA_SHELL_SCREENSHOTS,`${name}.png`),Buffer.from(image.data,'base64'));
  };
  await navigate('plugins/manage');
  await waitFor(`document.querySelector(${JSON.stringify(action('install'))})`,'fresh bundled Skill can be installed');
  await waitFor("window.UcaModelSelection?.readiness",'model capability known');
  await evaluate(`(async()=>{
    window.__pluginModelFetch=window.fetch;
    const response=await window.fetch('/api/v1/agent/models',{headers:{'x-ustc-client-protocol-major':'1'}});
    const view=await response.json();for(const model of view.models)model.tool_calling=false;
    window.fetch=(url,options)=>String(url)==='/api/v1/agent/models'?Promise.resolve(new Response(JSON.stringify(view),{status:200})):window.__pluginModelFetch(url,options);
    await window.UcaModelSelection.refresh();
  })()`);
  assert.equal(await evaluate("document.querySelector('.plugin-manage-model-note').hidden"),false);
  assert.equal(await evaluate("document.querySelector('.plugin-manage-model-note a').getAttribute('href')"),'#chat');
  assert.equal(await evaluate(`document.querySelector(${JSON.stringify(action('install'))}).disabled`),false,'model capability does not change installation authority');
  await evaluate('window.fetch=window.__pluginModelFetch;window.UcaModelSelection.refresh()');
  await waitFor("window.UcaModelSelection?.readiness",'restore model readiness');
  const offline = await evaluate("window.UcaModelSelection.selected.provider.mode==='mock'");
  if (offline) {
    await waitFor("document.querySelector('.plugin-manage-model-note').textContent.includes('离线演示')",'offline model explains installed plugin limitation');
    assert.equal(await evaluate("document.querySelector('.plugin-manage-model-note').hidden"),false);
  } else {
    await waitFor("document.querySelector('.plugin-manage-model-note').hidden",'tool capable model clears note');
  }
  pass('PLUGIN-selected-text-model-capability-is-visible');

  // A typed pre-commit capacity rejection must not strand the entire plugin page.
  await evaluate(`(()=>{
    window.__pluginCapacityFetch=window.fetch;window.__pluginCapacityCalls=[];window.__pluginCapacityTyped=true;
    window.fetch=(url,options={})=>{
      if(String(url)==='/api/v1/plugins/commands'){
        window.__pluginCapacityCalls.push(options.body);
        return Promise.resolve(new Response(JSON.stringify(window.__pluginCapacityTyped?{schema:'plugin-error/v1',error:'plugin_capacity_exceeded'}:{error:'upstream rate limit'}),{status:429}));
      }
      return window.__pluginCapacityFetch(url,options);
    };
  })()`);
  await click(action('install'));
  await waitFor("document.querySelector('.plugin-manage-status').textContent.includes('容量上限')&&!document.querySelector('.plugin-management').getAttribute('aria-busy').includes('true')",'typed capacity is known rejection');
  assert.equal(await evaluate("sessionStorage.getItem('uca.plugin-management.pending.v1')"),null);
  assert.equal(await evaluate(`document.querySelector(${JSON.stringify(action('install'))}).disabled`),false);
  await evaluate('window.__pluginCapacityTyped=false');await click(action('install'));
  await waitFor("document.querySelector('.plugin-manage-pending [data-plugin-action=retry]')&&!document.querySelector('.plugin-management').getAttribute('aria-busy').includes('true')",'generic 429 remains uncertain');
  assert.equal(await evaluate(`document.querySelector(${JSON.stringify(action('install'))}).disabled`),true);
  await evaluate('window.__pluginCapacityTyped=true');await click('.plugin-manage-pending [data-plugin-action=retry]');
  await waitFor("document.querySelector('.plugin-manage-pending').hidden&&!document.querySelector('.plugin-management').getAttribute('aria-busy').includes('true')",'exact retry resolves typed rejection');
  assert.equal(await evaluate('window.__pluginCapacityCalls[1]===window.__pluginCapacityCalls[2]'),true);
  await evaluate('window.fetch=window.__pluginCapacityFetch');
  pass('PLUGIN-known-capacity-release-and-generic-429-exact-retry');
  await evaluate(`(()=>{
    window.__pluginFetch=window.fetch;window.__pluginCommands=[];window.__pluginProbes=[];window.__pluginDropInstall=true;
    window.fetch=async(url,options={})=>{
      const path=String(url);
      if(path==='/api/v1/plugins/commands'){
        window.__pluginCommands.push({body:options.body,major:new Headers(options.headers).get('x-ustc-client-protocol-major'),credentials:options.credentials});
        const response=await window.__pluginFetch(url,options);
        if(window.__pluginDropInstall&&JSON.parse(options.body).intent.action==='install'){window.__pluginDropInstall=false;throw Error('synthetic lost response after server commit');}
        return response;
      }
      if(path==='/api/v1/plugins/probe')window.__pluginProbes.push(JSON.parse(options.body));
      return window.__pluginFetch(url,options);
    };
  })()`);
  try {
    await click(action('install'));
    await waitFor("document.querySelector('.plugin-manage-pending:not([hidden]) [data-plugin-action=retry]')&&!document.querySelector('.plugin-management').getAttribute('aria-busy').includes('true')",'ambiguous result exposes explicit retry');
    assert.equal(await evaluate('window.__pluginCommands.length'),1,'no automatic write retry');
    assert.equal(await evaluate(`document.querySelector(${JSON.stringify(action('install'))}).disabled`),true,'no new nonce action while result unknown');
    await click('.plugin-manage-pending [data-plugin-action="retry"]');
    await waitFor(`document.querySelector(${JSON.stringify(action('probe'))})&&!document.querySelector(${JSON.stringify(action('probe'))}).disabled`,'original install receipt recovered');
    assert.equal(await evaluate('window.__pluginCommands[0].body===window.__pluginCommands[1].body'),true,'retry preserves exact body and request_id');
    assert.equal(await evaluate("sessionStorage.getItem('uca.plugin-management.pending.v1')"),null,'known receipt clears local pending record');
    pass('PLUGIN-install-lost-response-exact-retry');

    await click(action('probe'));
    await waitFor(`document.querySelector(${JSON.stringify(action('grant'))})&&!document.querySelector(${JSON.stringify(action('grant'))}).disabled`,'probe shows explicit capability review');
    assert.equal(await evaluate(`document.querySelector(${JSON.stringify(action('enable'))}).disabled`),true,'probe alone cannot enable');
    assert.equal(await evaluate('window.__pluginProbes.length'),1);
    await click(action('grant'));
    await waitFor(`document.querySelector(${JSON.stringify(scope)})?.textContent.includes('已授权')&&!document.querySelector('.plugin-management').getAttribute('aria-busy').includes('true')`,'individual grant recorded');
    assert.equal(await evaluate(`document.querySelector(${JSON.stringify(action('enable'))}).disabled`),true,'grant alone is not enable confirmation');
    await click(`${scope} [data-plugin-review="true"]`);
    await click(action('enable'));
    await waitFor(`document.querySelector(${JSON.stringify(action('disable'))})&&!document.querySelector(${JSON.stringify(action('disable'))}).disabled`,'Skill enabled with reviewed readiness');
    const intents=await evaluate('window.__pluginCommands.map(item=>JSON.parse(item.body).intent.action)');
    assert.deepEqual(intents,['install','install','grant','enable']);
    assert.equal(await evaluate("window.__pluginCommands.every(item=>item.major==='1'&&item.credentials==='same-origin')"),true);
    assert.match(await evaluate("JSON.parse(window.__pluginCommands.at(-1).body).intent.readiness_digest"),/^sha256:[a-f0-9]{64}$/);
    await shot('plugin-management-enabled-desktop');
    pass('PLUGIN-skill-probe-explicit-grant-reviewed-enable');

    // Controlled GET projection only: historical installations never regain authority.
    await evaluate(`(async()=>{
      window.__pluginAvailableFetch=window.fetch;
      const response=await window.__pluginAvailableFetch('/api/v1/plugins',{headers:{'x-ustc-client-protocol-major':'1'},credentials:'same-origin'});
      window.__pluginUnavailableView=await response.json();
      const pkg=window.__pluginUnavailableView.packages.find(item=>item.package_id==='ustc.campus-guide');
      const current=structuredClone(pkg);current.available=true;current.installation=null;current.catalog_revision+=".current";
      window.__pluginUnavailableView.packages.push(current);
      pkg.available=false;pkg.fields=[];pkg.capabilities=[];
      window.fetch=(url,options={})=>String(url)==='/api/v1/plugins'&&(!options.method||options.method==='GET')
        ?Promise.resolve(new Response(JSON.stringify(window.__pluginUnavailableView),{status:200}))
        :window.__pluginAvailableFetch(url,options);
    })()`);
    await click('.plugin-management > [data-plugin-action="refresh"]');
    await waitFor(`document.querySelector(${JSON.stringify(orphan)})?.textContent.includes('包来源暂不可用')&&!document.querySelector('.plugin-management').getAttribute('aria-busy').includes('true')`,'missing source is explicit');
    assert.equal(await evaluate(`document.querySelector(${JSON.stringify(orphanAction('disable'))})!==null`),true);
    for (const blocked of ['install','configure','probe','grant','enable']) assert.equal(await evaluate(`document.querySelector(${JSON.stringify(orphanAction(blocked))})===null`),true,`missing source blocks ${blocked}`);
    assert.equal(await evaluate(`document.querySelector(${JSON.stringify(orphan)})?.textContent.includes('已启用，Agent 使用时仍会检查当前权限。')`),false,'historical enabled state is not runnable authority');
    await evaluate("window.__pluginUnavailableView.packages.find(item=>item.package_id==='ustc.campus-guide').installation.state='disabled'");
    await click('.plugin-management > [data-plugin-action="refresh"]');
    await waitFor(`document.querySelector(${JSON.stringify(orphan)})?.textContent.includes('已停用')&&!document.querySelector('.plugin-management').getAttribute('aria-busy').includes('true')`,'disabled orphan view');
    for (const blocked of ['install','configure','probe','grant','enable','disable']) assert.equal(await evaluate(`document.querySelector(${JSON.stringify(orphanAction(blocked))})===null`),true,`disabled orphan blocks ${blocked}`);
    assert.equal(await evaluate(`document.querySelectorAll(${JSON.stringify(scope)}).length`),2,'current and historical same-version source rows coexist');
    assert.equal(await evaluate(`document.querySelector(${JSON.stringify(action('install'))})!==null`),true,'current package remains separately installable');
    await evaluate('window.fetch=window.__pluginAvailableFetch');
    await click('.plugin-management > [data-plugin-action="refresh"]');
    await waitFor(`document.querySelector(${JSON.stringify(action('disable'))})&&!document.querySelector(${JSON.stringify(action('disable'))}).disabled`,'restore real current source');
    pass('PLUGIN-unavailable-source-denial-projection');
    await cdp.send('Emulation.setDeviceMetricsOverride',{width:390,height:844,deviceScaleFactor:1,mobile:true},sessionId);
    await navigate('plugins/manage');
    await waitFor("document.querySelector('#app-sidebar').getBoundingClientRect().right<=1",'mobile drawer closing animation finishes');
    await evaluate(`document.querySelector(${JSON.stringify(scope)}).scrollIntoView({block:'start'})`);
    assert.equal(await evaluate('document.documentElement.scrollWidth<=window.innerWidth+1'),true,'mobile layout does not overflow');
    await shot('plugin-management-enabled-mobile');
    await click(action('disable'));
    await waitFor(`document.querySelector(${JSON.stringify(scope)})?.textContent.includes('已停用')&&!document.querySelector('.plugin-management').getAttribute('aria-busy').includes('true')`,'disabled state is read back');
    assert.equal(await evaluate(`document.querySelector(${JSON.stringify(action('enable'))})===null`),true,'disable drops prior probe/enable projection');
    await evaluate(`document.querySelector(${JSON.stringify(scope)}).querySelector('.plugin-manage-advanced').open=true`);
    await click(action('revoke'));
    await click(action('confirm-revoke'));
    await waitFor(`document.querySelector(${JSON.stringify(scope)})?.textContent.includes('此安装已结束')`,'revoke is terminal and read back');
    pass('PLUGIN-disable-revoke-responsive');
  } finally {
    await evaluate('window.fetch=window.__pluginFetch');
    await cdp.send('Emulation.setDeviceMetricsOverride',{width:1440,height:900,deviceScaleFactor:1,mobile:false},sessionId);
  }
}
