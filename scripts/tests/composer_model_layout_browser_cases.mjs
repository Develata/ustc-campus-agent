import assert from 'node:assert/strict';

export async function checkComposerModelLayout({evaluate, waitFor, navigate, pass, cdp, sessionId}) {
  await navigate('chat');
  await waitFor("!document.querySelector('#chat-model-select').disabled", 'model ready');
  const original = await evaluate("document.querySelector('#chat-model-select').selectedOptions[0].textContent");
  try {
    for (const width of [1440, 768, 390, 320]) {
      await cdp.send('Emulation.setDeviceMetricsOverride', {width,height:900,deviceScaleFactor:1,mobile:width<600},sessionId);
      for (const longName of [false,true]) {
        await evaluate(`document.querySelector('#chat-model-select').selectedOptions[0].textContent=${JSON.stringify(longName ? 'campus-model-with-an-extremely-long-display-name-for-layout-validation' : original)}`);
        const geometry=await evaluate(`(() => {
          const picker=document.querySelector('#chat-model-select').getBoundingClientRect(), send=document.querySelector('#chat-send').getBoundingClientRect();
          return {center:Math.abs(picker.top+picker.height/2-send.top-send.height/2),gap:send.left-picker.right,
            inForm:document.querySelector('#chat-form').contains(document.querySelector('#chat-model-select')),
            fits:picker.left>=0&&send.right<=innerWidth&&document.documentElement.scrollWidth<=innerWidth,
            touch:picker.height>=44&&picker.width>=44&&send.width>=44&&send.height>=44};
        })()`);
        assert.equal(geometry.inForm,true);assert.ok(geometry.center<2,'picker remains on Send baseline');
        assert.ok(geometry.gap>=0&&geometry.gap<=10,'picker sits immediately left of Send');
        assert.equal(geometry.fits,true);assert.equal(geometry.touch,true);
      }
    }
    pass('COMPOSER-model-left-of-send-at-1440-768-390-320-and-long-names');
  } finally {
    await evaluate(`document.querySelector('#chat-model-select').selectedOptions[0].textContent=${JSON.stringify(original)}`);
    await cdp.send('Emulation.clearDeviceMetricsOverride',{},sessionId);
  }
}
