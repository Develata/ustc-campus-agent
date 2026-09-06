import assert from 'node:assert/strict';

// User actions against a real isolated daemon; no API fixture replaces confirmation.
export async function checkCalendarProposals({evaluate, waitFor, navigate, click, field, pass, cdp, sessionId}) {
  await navigate('chat');
  const root = '#calendar-proposals';
  await waitFor(`document.querySelector('${root} summary').textContent.includes('已保存')`, 'calendar loaded');
  await evaluate(`document.querySelector('${root} > details').open = true`);
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
  await evaluate(`(() => {const original=window.fetch.bind(window);window.__calendarFetch=original;window.__calendarConfirmPosts=0;window.fetch=async(url,options={})=>{const confirm=String(url).endsWith('/confirm')&&options.method==='POST';if(confirm)window.__calendarConfirmPosts++;const response=await original(url,options);if(confirm&&!window.__calendarLost){window.__calendarLost=true;await response.arrayBuffer();throw Error('test lost durable reply');}return response;};})()`);
  await evaluate(`(() => {const button=document.querySelector(${JSON.stringify(card+' button')});button.click();button.click();})()`);
  await waitFor(`Array.from(document.querySelectorAll('${root} [data-calendar-item-id]')).some(e=>e.textContent.includes('${title}'))`, 'confirmed readback');
  assert.equal(await evaluate('window.__calendarConfirmPosts'), 1);
  await evaluate('window.fetch=window.__calendarFetch');
  const item = await evaluate(`Array.from(document.querySelectorAll('${root} [data-calendar-item-id]')).find(e=>e.textContent.includes('${title}')).dataset.calendarItemId`);
  const itemCard = `${root} [data-calendar-item-id="${item}"]`;
  pass('CALENDAR-lost-confirmation-recovers-without-duplicate-write');
  await click(`${itemCard} button`);
  await field(`${root} input[name=calendar-title]`, title+'已修改');
  await field(`${root} input[name=calendar-time]`, '2026-09-09T16:00');
  await click(`${root} button[type=submit]`);
  await waitFor(`Array.from(document.querySelectorAll('${root} [data-calendar-proposal-id]')).some(e=>e.textContent.includes('${title}已修改'))`, 'edit preview');
  const edited = await evaluate(`Array.from(document.querySelectorAll('${root} [data-calendar-proposal-id]')).find(e=>e.textContent.includes('${title}已修改')).dataset.calendarProposalId`);
  await click(`${root} [data-calendar-proposal-id="${edited}"] button`);
  await waitFor(`document.querySelector(${JSON.stringify(itemCard)}).textContent.includes('16:00:00')`, 'edit applied');
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
  assert.match(await evaluate(`document.querySelector('${root}').textContent`),/提醒未启用|未启用提醒/);
  pass('CALENDAR-delete-confirms-exact-item-and-never-claims-reminder');
  await cdp.send('Emulation.clearDeviceMetricsOverride',{},sessionId);
}
