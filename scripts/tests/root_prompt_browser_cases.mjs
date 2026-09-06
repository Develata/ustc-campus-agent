import assert from 'node:assert/strict';

// Settings use the real daemon; intercepted responses model transport uncertainty only.
export async function checkRootPromptSettings({evaluate, waitFor, navigate, click, field, pass, cdp, sessionId}) {
  await navigate('settings');
  const text = '#root-prompt-text', status = '#root-prompt-status';
  await waitFor(`document.querySelector('${text}') && !document.querySelector('${text}').disabled`, 'personal instruction loaded');
  const original = await evaluate(`document.querySelector('${text}').value`);
  const instruction = '你是校园学习助手。\n先给出明确步骤，再说明信息来源。';
  await field(text, instruction);
  assert.equal(await evaluate("document.querySelector('#root-prompt-count').textContent"), `${Buffer.byteLength(instruction)} / 8192 字节`);
  await click('#root-prompt-save');
  await waitFor(`document.querySelector('${status}').textContent.includes('已保存')`, 'personal instruction saved');
  await click('#root-prompt-reload');
  await waitFor(`document.querySelector('${status}').textContent.includes('已读取')`, 'personal instruction readback');
  assert.equal(await evaluate(`document.querySelector('${text}').value`), instruction);
  assert.match(await evaluate("document.querySelector('#root-prompt-settings').textContent"), /从下一条消息.*全部对话/);
  pass('ROOT-PROMPT-save-and-readback-preserve-multiline-personal-instruction');

  const draft = instruction + '\n回答保持简洁。';
  await field(text, draft); await click('#root-prompt-reload');
  await waitFor(`document.querySelector('${status}').textContent.includes('草稿已保留')`, 'dirty read preserves draft');
  assert.equal(await evaluate(`document.querySelector('${text}').value`), draft);
  await evaluate(`window.dispatchEvent(new Event('focus'))`);
  assert.equal(await evaluate(`document.querySelector('${text}').value`), draft);
  pass('ROOT-PROMPT-refresh-and-focus-preserve-dirty-draft');

  await evaluate(`(() => {
    window.__rootPromptFetch=window.fetch.bind(window);window.__rootPromptWrites=0;window.__rootPromptLost=false;
    window.fetch=async(url,options={})=>{
      const write=String(url)==='/api/v1/agent/root-prompt'&&options.method==='PUT';
      if(write)window.__rootPromptWrites++;
      const response=await window.__rootPromptFetch(url,options);
      if(write&&!window.__rootPromptLost){window.__rootPromptLost=true;await response.arrayBuffer();throw Error('controlled lost committed reply');}
      return response;
    };
  })()`);
  await evaluate("document.querySelector('#root-prompt-save').click();document.querySelector('#root-prompt-save').click()");
  await waitFor(`document.querySelector('${status}').textContent.includes('已核对服务端：设置已保存')`, 'lost settings write readback');
  assert.equal(await evaluate('window.__rootPromptWrites'), 1);
  assert.equal(await evaluate(`document.querySelector('${text}').value`), draft);
  await evaluate('window.fetch=window.__rootPromptFetch');
  pass('ROOT-PROMPT-lost-write-reconciles-without-double-submit');

  // A second client saves after this editor loaded. Neither conflict nor refresh replaces its draft.
  const conflictingDraft = '我的未保存草稿';
  await field(text, conflictingDraft);
  await evaluate(`(async()=>{
    const headers={'X-USTC-Client-Protocol-Major':'1','Content-Type':'application/json'};
    const current=await window.fetch('/api/v1/agent/root-prompt',{headers}).then(r=>r.json());
    const response=await window.fetch('/api/v1/agent/root-prompt',{method:'PUT',headers,body:JSON.stringify({schema:'agent-root-prompt-update/v1',expected_revision:current.revision,text:'另一客户端的设置'})});
    if(!response.ok)throw Error('controlled peer update failed');
  })()`);
  await click('#root-prompt-save');
  await waitFor("!document.querySelector('.root-prompt-conflict').hidden", 'conflicting server version offered');
  assert.equal(await evaluate(`document.querySelector('${text}').value`), conflictingDraft);
  assert.equal(await evaluate("document.querySelector('#root-prompt-save').disabled"), true);
  assert.equal(await evaluate("document.querySelector('#root-prompt-remote').textContent"), '另一客户端的设置');
  await click('#root-prompt-keep');
  assert.equal(await evaluate(`document.querySelector('${text}').value`), conflictingDraft);
  await click('#root-prompt-save');
  await waitFor(`document.querySelector('${status}').textContent.includes('已保存')`, 'explicit conflict choice saved');
  pass('ROOT-PROMPT-revision-conflict-preserves-draft-and-requires-explicit-choice');

  await field(text, '校'.repeat(2731));
  assert.equal(await evaluate("document.querySelector('#root-prompt-save').disabled"), true);
  assert.equal(await evaluate("document.querySelector('#root-prompt-count').textContent"), '8193 / 8192 字节');
  await field(text, conflictingDraft);
  await cdp.send('Emulation.setDeviceMetricsOverride', {width:390,height:844,deviceScaleFactor:1,mobile:true}, sessionId);
  assert.equal(await evaluate('document.documentElement.scrollWidth <= innerWidth + 1'), true);
  assert.equal(await evaluate("Array.from(document.querySelectorAll('#root-prompt-settings form button')).every(e=>e.getBoundingClientRect().height>=44)"), true);
  await cdp.send('Emulation.clearDeviceMetricsOverride', {}, sessionId);
  pass('ROOT-PROMPT-byte-limit-and-mobile-controls');

  await click('#root-prompt-reset');
  await waitFor(`document.querySelector('${status}').textContent.includes('已恢复默认')`, 'personal instruction cleared');
  assert.equal(await evaluate(`document.querySelector('${text}').value`), '');
  await click('#root-prompt-reload');
  await waitFor(`document.querySelector('${status}').textContent.includes('当前使用默认设置')`, 'default readback');
  pass('ROOT-PROMPT-restore-default-persists-empty-setting');
  // This suite only restores its own setting on an isolated test daemon.
  if (original) { await field(text, original); await click('#root-prompt-save'); await waitFor(`document.querySelector('${status}').textContent.includes('已保存')`, 'restore original test setting'); }
}
