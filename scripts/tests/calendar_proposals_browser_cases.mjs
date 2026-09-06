import assert from 'node:assert/strict';

// User actions against a real isolated daemon; no API fixture replaces confirmation.
export async function checkCalendarProposals({evaluate, waitFor, navigate, click, field, pass, cdp, sessionId}) {
  await navigate('chat');
  const root = '#calendar-proposals';
  await waitFor(`document.querySelector('${root} summary').textContent.includes('已保存')`, 'calendar loaded');
  await evaluate(`document.querySelector('${root} > details').open = true`);
  // Month selection is read-only, Gregorian and independent of the browser timezone.
  for (const [month, days] of [['2024-02',29],['2025-02',28],['2026-04',30],['2026-12',31]]) {
    await field(`${root} input[type=month]`, month);
    await evaluate(`document.querySelector('${root} input[type=month]').dispatchEvent(new Event('change',{bubbles:true}))`);
    assert.equal(await evaluate(`document.querySelectorAll('${root} [data-calendar-day^="${month}"]').length`), days);
  }
  await click(`${root} [aria-label="下个月"]`);
  assert.equal(await evaluate(`document.querySelector('${root} input[type=month]').value`), '2027-01');
  await click(`${root} [data-calendar-day="2027-01-15"]`);
  await click(`${root} .calendar-agenda-heading button`);
  assert.equal(await evaluate(`document.querySelector('${root} input[name=calendar-time]').value`), '2027-01-15T09:00');
  await field(`${root} input[name=calendar-title]`, '保留草稿');
  await evaluate(`window.UcaCalendarProposals.mount(document.querySelector('${root}')).refresh()`);
  assert.equal(await evaluate(`document.querySelector('${root} input[type=month]').value`), '2027-01');
  assert.equal(await evaluate(`document.querySelector('${root} input[name=calendar-title]').value`), '保留草稿');
  assert.equal(await evaluate(`window.UcaCalendarMonth.dayKey('2026-09-08T20:00:00Z')`), '2026-09-09');
  await click(`${root} [aria-label="查看无日期事项"]`);
  await click(`${root} .calendar-agenda-heading button`);
  assert.equal(await evaluate(`document.querySelector('${root} input[type=checkbox]').checked`), true);
  pass('CALENDAR-month-length-leap-year-year-boundary-draft-and-date-prefill');
  await evaluate(`document.querySelector('${root} .calendar-proposals-manual').open = true`);
  const title = '日期提案浏览器验收';
  await field(`${root} input[name=calendar-title]`, title);
  await click(`${root} input[type=checkbox]`);
  await field(`${root} input[name=calendar-time]`, '2026-09-09T14:00');
  await click(`${root} button[type=submit]`);
  await waitFor(`Array.from(document.querySelectorAll('${root} [data-calendar-proposal-id]')).some(e=>e.textContent.includes('${title}'))`, 'pending proposal');
  const id = await evaluate(`Array.from(document.querySelectorAll('${root} [data-calendar-proposal-id]')).find(e=>e.textContent.includes('${title}')).dataset.calendarProposalId`);
  const card = `${root} [data-calendar-proposal-id="${id}"]`;
  assert.match(await evaluate(`document.querySelector(${JSON.stringify(card)}).textContent`), /2026.*09.*09.*14:00:00 UTC\+08:00/);
  assert.equal(await evaluate(`Array.from(document.querySelectorAll('${root} [data-calendar-item-id]')).some(e=>e.textContent.includes('${title}'))`), false);
  pass('CALENDAR-proposal-previews-date-without-effect');
  // Simulate the response disappearing after the real confirmation committed.
  await evaluate(`(() => {const original=window.fetch.bind(window);window.__calendarFetch=original;window.__calendarConfirmPosts=0;window.fetch=async(url,options={})=>{if(window.__calendarBlockReads&&String(url)==='/api/v1/calendar/proposals'&&options.method!=='POST')throw Error('test unavailable readback');const confirm=String(url).endsWith('/confirm')&&options.method==='POST';if(confirm)window.__calendarConfirmPosts++;const response=await original(url,options);if(confirm&&!window.__calendarLost){window.__calendarLost=true;window.__calendarBlockReads=true;await response.arrayBuffer();throw Error('test lost durable reply');}return response;};})()`);
  await evaluate(`(() => {const button=document.querySelector(${JSON.stringify(card+' button')});button.click();button.click();})()`);
  await waitFor(`window.__calendarBlockReads===true && document.querySelector('${root}').getAttribute('aria-busy')==='false'`, 'unknown confirmation result');
  assert.equal(await evaluate(`window.UcaCalendarProposals.mount(document.querySelector('${root}')).destroy()`), false);
  await navigate('plugins/calendar');
  assert.equal(await evaluate(`document.querySelector('#calendar-page .calendar-agenda-heading button').disabled`), true);
  assert.equal(await evaluate(`document.querySelector('#calendar-page button[type=submit]').disabled`), true);
  await evaluate('window.__calendarBlockReads=false');
  await navigate('chat');
  await evaluate(`window.UcaCalendarProposals.mount(document.querySelector('${root}')).refresh()`);
  await waitFor(`Array.from(document.querySelectorAll('${root} [data-calendar-item-id]')).some(e=>e.textContent.includes('${title}'))`, 'confirmed readback');
  assert.equal(await evaluate('window.__calendarConfirmPosts'), 1);
  await evaluate('window.fetch=window.__calendarFetch');
  assert.equal(await evaluate(`document.querySelector('#calendar-page .calendar-agenda-heading button').disabled`), false);
  assert.equal(await evaluate(`document.querySelector('#calendar-page button[type=submit]').disabled`), false);
  const item = await evaluate(`Array.from(document.querySelectorAll('${root} [data-calendar-item-id]')).find(e=>e.textContent.includes('${title}')).dataset.calendarItemId`);
  const itemCard = `${root} [data-calendar-item-id="${item}"]`;
  pass('CALENDAR-lost-confirmation-locks-both-views-and-recovers-without-duplicate-write');
  await click(`${itemCard} button`);
  await field(`${root} input[name=calendar-title]`, title+'已修改');
  await field(`${root} input[name=calendar-time]`, '2026-09-09T16:00');
  await click(`${root} button[type=submit]`);
  await waitFor(`Array.from(document.querySelectorAll('${root} [data-calendar-proposal-id]')).some(e=>e.textContent.includes('${title}已修改'))`, 'edit preview');
  const edited = await evaluate(`Array.from(document.querySelectorAll('${root} [data-calendar-proposal-id]')).find(e=>e.textContent.includes('${title}已修改')).dataset.calendarProposalId`);
  await click(`${root} [data-calendar-proposal-id="${edited}"] button`);
  await waitFor(`document.querySelector(${JSON.stringify(itemCard)}).textContent.includes('16:00:00')`, 'edit applied');
  assert.equal(await evaluate(`document.querySelector('${root} [data-calendar-day="2026-09-09"]').getAttribute('aria-pressed')`), 'true');
  pass('CALENDAR-update-retains-item-identity');
  await cdp.send('Emulation.setDeviceMetricsOverride',{width:390,height:844,deviceScaleFactor:1,mobile:true},sessionId);
  await evaluate(`document.querySelector('${root} > details').open = true`);
  assert.equal(await evaluate('document.documentElement.scrollWidth <= window.innerWidth + 1'), true);
  const buttonSize = await evaluate(`Array.from(document.querySelectorAll('${root} [data-calendar-write]')).every(e=>e.getBoundingClientRect().height>=44)`);
  assert.equal(buttonSize,true);
  pass('CALENDAR-mobile-controls-fit-without-horizontal-overflow');
  // Delete also needs a separate proposal and explicit confirmation.
  await click(`${itemCard} button:nth-child(2)`);
  await waitFor(`Array.from(document.querySelectorAll('${root} [data-calendar-proposal-id]')).some(e=>e.querySelector('h4').textContent==='删除事项'&&e.textContent.includes('${title}'))`, 'delete preview');
  assert.ok(await evaluate(`document.querySelector(${JSON.stringify(itemCard)}) !== null`));
  const deleting = await evaluate(`Array.from(document.querySelectorAll('${root} [data-calendar-proposal-id]')).find(e=>e.querySelector('h4').textContent==='删除事项'&&e.textContent.includes('${title}')).dataset.calendarProposalId`);
  await click(`${root} [data-calendar-proposal-id="${deleting}"] button:nth-child(2)`);
  await waitFor(`!document.querySelector('${root} [data-calendar-proposal-id="${deleting}"]')`, 'delete cancelled');
  assert.ok(await evaluate(`document.querySelector(${JSON.stringify(itemCard)}) !== null`));
  pass('CALENDAR-cancel-preserves-existing-item');
  await click(`${itemCard} button:nth-child(2)`);
  await waitFor(`Array.from(document.querySelectorAll('${root} [data-calendar-proposal-id]')).some(e=>e.querySelector('h4').textContent==='删除事项'&&e.textContent.includes('${title}'))`, 'second delete preview');
  const confirmedDelete = await evaluate(`Array.from(document.querySelectorAll('${root} [data-calendar-proposal-id]')).find(e=>e.querySelector('h4').textContent==='删除事项'&&e.textContent.includes('${title}')).dataset.calendarProposalId`);
  await click(`${root} [data-calendar-proposal-id="${confirmedDelete}"] button`);
  await waitFor(`!document.querySelector(${JSON.stringify(itemCard)})`, 'delete applied');
  assert.match(await evaluate(`document.querySelector('${root}').textContent`),/站内提醒/);
  pass('CALENDAR-delete-confirms-exact-item-and-explains-inbox-reminders');
  await navigate('plugins/calendar');
  await waitFor(`document.querySelector('#calendar-page [data-calendar-day]') !== null`, 'expanded month loaded');
  assert.equal(await evaluate(`document.querySelector('#calendar-page > details').open`), true);
  assert.equal(await evaluate(`Array.from(document.querySelectorAll('#calendar-page [data-calendar-day]')).every(e=>e.getBoundingClientRect().height>=44)`), true);
  assert.equal(await evaluate('document.documentElement.scrollWidth <= window.innerWidth + 1'), true);
  pass('CALENDAR-expanded-mobile-month-grid');
  await cdp.send('Emulation.clearDeviceMetricsOverride',{},sessionId);
  await checkCalendarAuxiliaryRecovery({evaluate, waitFor, navigate, pass});
}


async function checkCalendarAuxiliaryRecovery({evaluate, waitFor, navigate, pass}) {
  const root = '#calendar-proposals';
  const endpoint = '/api/v1/calendar/proposals';
  const request = (path, body) => evaluate(`(async()=>{
    const response=await fetch(${JSON.stringify(path)},${body ? `{method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify(${JSON.stringify(body)})}` : "{cache:'no-store'}"});
    const value=await response.json();if(!response.ok)throw Error('Calendar test request failed: '+response.status);return value;
  })()`);
  const batches = [];
  async function proposeBatch(items) {
    const value = await request('/api/v1/calendar/batches', {schema:'calendar-batch-proposal/v1',request_id:await evaluate('crypto.randomUUID()'),items});
    assert.equal(value.schema, 'calendar-batch-result/v1');
    assert.equal(value.batch.status, 'pending');
    batches.push(value.batch.id);
    await evaluate(`window.UcaCalendarProposals.mount(document.querySelector('${root}')).refresh()`);
    await waitFor(`document.querySelector('${root} [data-calendar-batch-id="${value.batch.id}"]') !== null`, 'real batch proposal loaded');
    return value.batch;
  }
  async function loseReply(path, buttonExpression, recovered, replacement = null) {
    await evaluate(`(()=>{const state={original:window.fetch.bind(window),path:${JSON.stringify(path)},replacement:${JSON.stringify(replacement)},blocked:false,posts:0};window.__calendarAuxiliaryFault=state;
      window.fetch=async(url,options={})=>{if(state.blocked&&String(url)==='${endpoint}'&&options.method!=='POST')throw Error('test unavailable auxiliary readback');
        const target=String(url)===state.path&&options.method==='POST';if(target)state.posts++;
        const response=await state.original(target&&state.replacement?state.replacement.path:url,target&&state.replacement?{...options,body:JSON.stringify({schema:state.replacement.schema})}:options);if(target&&state.posts===1){await response.arrayBuffer();state.blocked=true;throw Error('test lost auxiliary reply');}return response;};})()`);
    try {
      await evaluate(`(()=>{const button=${buttonExpression};if(!button)throw Error('Missing auxiliary action');button.click();button.click();})()`);
      await waitFor(`window.__calendarAuxiliaryFault.blocked && document.querySelector('${root}').getAttribute('aria-busy')==='false'`, 'auxiliary reply lost');
      assert.equal(await evaluate(`window.UcaCalendarProposals.mount(document.querySelector('${root}')).destroy()`), false);
      for (const view of [root, '#calendar-page']) {
        assert.equal(await evaluate(`document.querySelector('${view} .calendar-agenda-heading button').disabled`), true);
        assert.equal(await evaluate(`document.querySelector('${view} button[type=submit]').disabled`), true);
      }
      await navigate('plugins/calendar');
      assert.equal(await evaluate(`document.querySelector('#calendar-page .calendar-agenda-heading button').disabled`), true);
      await navigate('chat');
      await evaluate(`window.__calendarAuxiliaryFault.blocked=false;window.dispatchEvent(new Event('uca:calendar-changed'))`);
      await waitFor(recovered, 'auxiliary exact receipt recovered');
      for (const view of [root, '#calendar-page']) {
        assert.equal(await evaluate(`document.querySelector('${view} .calendar-agenda-heading button').disabled`), false);
        assert.equal(await evaluate(`document.querySelector('${view} button[type=submit]').disabled`), false);
      }
      assert.equal(await evaluate('window.__calendarAuxiliaryFault.posts'), 1);
    } finally {
      await evaluate('window.fetch=window.__calendarAuxiliaryFault.original;delete window.__calendarAuxiliaryFault');
      await evaluate(`window.UcaCalendarProposals.mount(document.querySelector('${root}')).refresh()`);
    }
    await navigate('chat');
  }
  await navigate('chat');
  await evaluate(`document.querySelector('${root} > details').open=true`);
  const initial = await request(endpoint);
  const unique = await evaluate('crypto.randomUUID()');
  const titles = [`批量恢复首项 ${unique}`, `批量恢复次项 ${unique}`];
  // Past dates make the station inbox deliver during readback, without scheduler sleeps.
  const drafts = titles.map((title,index)=>({title,scheduled_for:new Date((initial.now_unix_secs-120+index*60)*1000).toISOString()}));
  try {
    const confirmed = await proposeBatch(drafts);
    const firstDay = await evaluate(`window.UcaCalendarMonth.dayKey(${JSON.stringify(drafts[0].scheduled_for)})`);
    await loseReply(`/api/v1/calendar/batches/${encodeURIComponent(confirmed.id)}/confirm`,
      `document.querySelector('${root} [data-calendar-batch-id="${confirmed.id}"] button')`,
      `!document.querySelector('${root} [data-calendar-batch-id="${confirmed.id}"]') && document.querySelector('${root} [data-calendar-day="${firstDay}"]')?.getAttribute('aria-pressed')==='true' && Array.from(document.querySelectorAll('${root} [data-calendar-item-id]')).some(e=>e.querySelector('h4').textContent===${JSON.stringify(titles[0])})`);
    const saved = await request(endpoint);
    const receipt = saved.batches.find(batch=>batch.id===confirmed.id);
    assert.equal(receipt.status, 'applied');
    assert.equal(receipt.result.length, 2);
    assert.deepEqual(receipt.result.map(item=>item.title), titles);
    assert.equal(saved.items.filter(item=>titles.includes(item.title)).length, 2);
    pass('CALENDAR-batch-lost-confirmation-locks-both-views-and-selects-exact-first-item');

    const reminder = saved.reminders.find(item=>item.item_id===receipt.result[0].id);
    assert.equal(reminder.status, 'delivered');
    assert.equal(reminder.read_at_unix_secs, null);
    await loseReply(`/api/v1/calendar/reminders/${encodeURIComponent(reminder.id)}/read`,
      `Array.from(document.querySelectorAll('${root} .calendar-proposal-card')).find(e=>e.querySelector('h4')?.textContent===${JSON.stringify(titles[0])}&&e.textContent.includes('站内投递时间'))?.querySelector('button')`,
      `Array.from(document.querySelectorAll('${root} .calendar-proposal-card')).some(e=>e.querySelector('h4')?.textContent===${JSON.stringify(titles[0])}&&e.textContent.includes('站内投递时间')&&e.textContent.includes('已读')&&!e.querySelector('button'))`);
    const read = (await request(endpoint)).reminders.find(item=>item.id===reminder.id);
    assert.ok(Number.isSafeInteger(read.read_at_unix_secs));
    assert.equal(read.delivered_at_unix_secs, reminder.delivered_at_unix_secs);
    pass('CALENDAR-reminder-lost-read-receipt-locks-both-views-and-recovers-once');

    const cancelled = await proposeBatch([{...drafts[0],title:`批量取消恢复 ${unique}`}]);
    await loseReply(`/api/v1/calendar/batches/${encodeURIComponent(cancelled.id)}/cancel`,
      `document.querySelector('${root} [data-calendar-batch-id="${cancelled.id}"] button:nth-child(2)')`,
      `!document.querySelector('${root} [data-calendar-batch-id="${cancelled.id}"]') && !document.querySelector('${root} .calendar-agenda-heading button').disabled`);
    const afterCancel = await request(endpoint);
    assert.equal(afterCancel.batches.find(batch=>batch.id===cancelled.id).status, 'cancelled');
    assert.equal(afterCancel.items.some(item=>item.title===cancelled.items[0].title), false);
    pass('CALENDAR-batch-lost-cancellation-recovers-without-item-effect');


    // The cancel intent never reaches the server; another tab wins with confirmation.
    const raced = await proposeBatch([{...drafts[0],title:`批量取消竞态 ${unique}`}]);
    await loseReply(`/api/v1/calendar/batches/${encodeURIComponent(raced.id)}/cancel`,
      `document.querySelector('${root} [data-calendar-batch-id="${raced.id}"] button:nth-child(2)')`,
      `document.querySelector('${root} .calendar-proposals-status').textContent.includes('状态与这次操作不同') && Array.from(document.querySelectorAll('${root} [data-calendar-item-id]')).some(e=>e.querySelector('h4').textContent===${JSON.stringify(raced.items[0].title)})`,
      {path:`/api/v1/calendar/batches/${encodeURIComponent(raced.id)}/confirm`,schema:'calendar-batch-confirm/v1'});
    const afterRace = await request(endpoint);
    const raceReceipt = afterRace.batches.find(batch=>batch.id===raced.id);
    assert.equal(raceReceipt.status, 'applied');
    assert.equal(raceReceipt.result.length, 1);
    assert.equal(afterRace.items.filter(item=>item.id===raceReceipt.result[0].id).length, 1);
    pass('CALENDAR-cancel-confirm-race-reports-different-terminal-state-and-unlocks');
  } finally {
    // Clean only this helper's exact batch IDs and their acknowledged item receipts.
    const state = await request(endpoint);
    for (const batch of state.batches.filter(value=>batches.includes(value.id))) {
      if (batch.status==='pending') await request(`/api/v1/calendar/batches/${encodeURIComponent(batch.id)}/cancel`, {schema:'calendar-batch-cancel/v1'});
      for (const item of batch.result || []) {
        if (!state.items.some(value=>value.id===item.id)) continue;
        const deletion = await request(endpoint, {schema:'calendar-proposal/v1',request_id:await evaluate('crypto.randomUUID()'),mutation:{action:'delete',item_id:item.id}});
        await request(`${endpoint}/${encodeURIComponent(deletion.proposal.id)}/confirm`, {schema:'calendar-proposal-confirm/v1'});
      }
    }
    const remaining = await request(endpoint);
    assert.equal(remaining.items.some(item=>titles.includes(item.title)), false);
    await evaluate(`window.UcaCalendarProposals.mount(document.querySelector('${root}')).refresh()`);
  }
}
