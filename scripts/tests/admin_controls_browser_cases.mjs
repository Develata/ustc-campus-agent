import assert from 'node:assert/strict';
import {mkdir,writeFile} from 'node:fs/promises';
import {join} from 'node:path';

export async function checkAdminControls({evaluate,waitFor,cdp,sessionId,navigate,pass}) {
  const pointer = async selector => {
    const point = await evaluate(`(() => {
      const el=document.querySelector(${JSON.stringify(selector)});
      el.scrollIntoView({block:'center'});
      const r=el.getBoundingClientRect(),x=r.x+r.width/2,y=r.y+r.height/2;
      if (!r.width || !r.height || !el.contains(document.elementFromPoint(x,y))) throw Error('unreachable admin control');
      return {x,y};
    })()`);
    await cdp.send('Input.dispatchMouseEvent',{type:'mousePressed',...point,button:'left',clickCount:1},sessionId);
    await cdp.send('Input.dispatchMouseEvent',{type:'mouseReleased',...point,button:'left',clickCount:1},sessionId);
  };
  const shot = async name => {
    if (!process.env.UCA_SHELL_SCREENSHOTS) return;
    await evaluate("Promise.allSettled(document.getAnimations().filter(a=>a.effect.getTiming().iterations !== Infinity).map(a=>a.finished))");
    await mkdir(process.env.UCA_SHELL_SCREENSHOTS,{recursive:true});
    const result=await cdp.send('Page.captureScreenshot',{format:'png'},sessionId);
    await writeFile(join(process.env.UCA_SHELL_SCREENSHOTS,`${name}.png`),Buffer.from(result.data,'base64'));
  };
  await navigate('settings');
  await waitFor("document.querySelector('#provider-status').textContent.includes('离线演示')",'server-owned mock configuration');
  assert.match(await evaluate("document.querySelector('#provider-model').textContent"),/deterministic-mock/);
  const before=await evaluate('window.__posts.length');
  await pointer('#provider-status-refresh');
  await pointer('.operator-settings > summary');
  assert.equal(await evaluate('window.__posts.length'),before,'opening administrator UI is not publication');
  for (const prefix of ['publication','radar-publication']) {
    const confirm=prefix==='publication'?'#publication-confirm':'#radar-publication-confirm';
    const publish=`#${prefix}-publish`;
    const refresh=`#${prefix}-refresh`;
    assert.equal(await evaluate(`document.querySelector(${JSON.stringify(publish)}).disabled`),true);
    await pointer(refresh);
    await waitFor(`!document.querySelector(${JSON.stringify(refresh)}).disabled`,'read publication state');
    const count=await evaluate('window.__posts.length');
    assert.equal(count, before + (prefix === 'publication' ? 0 : 2), 'refresh state sends no POST');
    await pointer(confirm);
    assert.equal(await evaluate(`document.querySelector(${JSON.stringify(publish)}).disabled`),false);
    await pointer(publish);
    await waitFor(`!document.querySelector(${JSON.stringify(refresh)}).disabled && !document.querySelector(${JSON.stringify(confirm)}).checked`,'confirmed publication settled');
    assert.equal(await evaluate('window.__posts.length'),count+1,'one explicit publication');
    assert.equal(await evaluate(`document.querySelector(${JSON.stringify(publish)}).disabled`),true);
    const receipt=await evaluate(`document.querySelector('#${prefix}-receipt').textContent`);
    assert.notEqual(receipt,'—');
    assert.ok(receipt.trim());
    await pointer(confirm);
    await pointer(publish);
    await waitFor(`!document.querySelector(${JSON.stringify(refresh)}).disabled && !document.querySelector(${JSON.stringify(confirm)}).checked`,'explicit repeat publication settled');
    assert.equal(await evaluate(`document.querySelector('#${prefix}-receipt').textContent`),receipt,'exact retry retains receipt');
  }
  await evaluate("document.querySelector('.operator-settings').scrollIntoView({block:'start'})");
  await shot('desktop-administrator');
  await cdp.send('Emulation.setDeviceMetricsOverride',{width:390,height:844,deviceScaleFactor:1,mobile:true},sessionId);
  await waitFor("document.querySelector('#app-sidebar').inert && document.querySelector('#app-sidebar').getBoundingClientRect().right <= 0", 'administrator mobile drawer settled');
  assert.equal(await evaluate('document.documentElement.scrollWidth<=390'),true,'administrator mobile width');
  await evaluate("document.querySelector('.operator-settings').scrollIntoView({block:'start'})");
  await shot('mobile-administrator');
  await evaluate("document.querySelector('#theme-select').value='dark';document.querySelector('#theme-select').dispatchEvent(new Event('change'))");
  await shot('mobile-administrator-dark');
  assert.equal(await evaluate('document.documentElement.scrollWidth<=390'),true);
  await evaluate("document.querySelector('#theme-select').value='light';document.querySelector('#theme-select').dispatchEvent(new Event('change'))");
  await cdp.send('Emulation.setDeviceMetricsOverride',{width:1440,height:900,deviceScaleFactor:1,mobile:false},sessionId);
  pass('ADMIN-explicit-publication-receipt-retry-and-responsive-status');
}
