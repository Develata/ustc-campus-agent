// Targeted shared-cookie, stale-tab regression. Uses only controlled private
// development accounts; never persists cookies/passwords or invokes a model.
// Defaults to the currently served asset. --inject-local substitutes local account
// handlers without restarting; --reproduce-old-preview additionally exercises the
// known pre-fix bug and is only for an explicitly retained vulnerable test preview.
const fs = require('node:fs');
const path = require('node:path');
const crypto = require('node:crypto');
const {parseArgs} = require('node:util');
const root = path.resolve(__dirname, '../../..');
const {values:options} = parseArgs({options:{base:{type:'string'},'credentials-file':{type:'string'},'playwright-core':{type:'string'},'chrome-path':{type:'string'},result:{type:'string'},'inject-local':{type:'boolean',default:false},'reproduce-old-preview':{type:'boolean',default:false}}});
for(const key of ['base','credentials-file','playwright-core','chrome-path'])if(!options[key])throw Error('required_option_missing: '+key);
const {chromium} = require(path.resolve(options['playwright-core']));
const base = new URL(options.base).origin;
if(!['127.0.0.1','localhost','[::1]'].includes(new URL(base).hostname))throw Error('isolated_loopback_preview_required');
const accounts = JSON.parse(fs.readFileSync(options['credentials-file'],'utf8')).accounts;
const output = options.result || path.join(root,'dist/account-browser-smoke/logout-subject-result.json');
const localSource = fs.readFileSync(path.join(root,'apps/ustc-agentd/src/web/account.js'),'utf8');
const helperPattern = /  async function request\(endpoint, body\) \{[\s\S]*?    return value;\r?\n  \}/;
const localHelper = localSource.match(helperPattern)?.[0];
if (!localHelper) throw Error('local_account_helper_not_found');
const logoutPattern = /      logout\.addEventListener\("click", async \(\) => \{[^\n]*\}\);/;
const localLogout = localSource.match(logoutPattern)?.[0];
if(!localLogout)throw Error('local_logout_handler_not_found');
const report = {started_at:new Date().toISOString(), base, broadcast_channel:'disabled', checks:[], model_requests:0, frontend_injection:options['inject-local'], old_preview_reproduction:options['reproduce-old-preview'], backend_restarted:false, local_account_sha256:crypto.createHash('sha256').update(localSource).digest('hex')};
let browser, stage='launch';
function check(name, passed, fields={}) {report.checks.push({name,passed:!!passed,...fields});console.log(JSON.stringify({check:name,passed:!!passed}));if(!passed)throw Error('regression_check_failed');}
async function me(context) {const response=await context.request.get(base+'/api/v1/account/me');let body=null;try{body=await response.json();}catch(_){}return {status:response.status(),account:body?.account};}
async function fillLogin(page, account) {
  await page.locator('.account-panel:not([hidden]) input[name=login_name]').fill(account.login_name);
  await page.locator('.account-login input[name=password]').fill(account.password);
  await page.locator('.account-login button[type=submit]').click();
  await page.locator('.account-badge button').waitFor({state:'visible'});
}
async function scenario(patched) {
  const name=patched?'patched':'baseline';stage=name+'_setup';
  const context=await browser.newContext({viewport:{width:390,height:844}});
  context.setDefaultTimeout(12000);
  console.log(JSON.stringify({phase:name+'_begin'}));
  await context.addInitScript(()=>{Object.defineProperty(window,'BroadcastChannel',{value:undefined,configurable:true});});
  let injected=0;
  await context.route('**/assets/app.js',async route=>{
    const response=await route.fetch();const original=await response.text();
    if(!helperPattern.test(original))throw Error('preview_account_helper_not_found');
    if(patched&&options['inject-local']){injected++;await route.fulfill({response,body:original.replace(helperPattern,localHelper).replace(logoutPattern,localLogout)});}
    else await route.fulfill({response});
  });
  await context.route('**/api/v1/agent/**',route=>{
    const req=route.request();if(req.method()==='POST'&&(/\/chat$/.test(req.url())||/\/turns$/.test(req.url()))){report.model_requests++;return route.abort();}return route.continue();
  });
  const oldA=await context.newPage();await oldA.goto(base,{waitUntil:'domcontentloaded'});await fillLogin(oldA,accounts[0]);
  const a=await me(context);check(name+'_a_authenticated',a.status===200);
  // Delay all old-tab background work except its eventual logout. This models
  // a suspended tab without letting an unrelated 401 refresh its captured subject.
  let staleLogoutSubmitted=false;
  await oldA.route('**/api/v1/**',route=>{
    if(route.request().url().endsWith('/account/logout'))staleLogoutSubmitted=true;
    return staleLogoutSubmitted?route.continue():route.fulfill({status:503,contentType:'application/json',body:'{"error":"test_tab_background_delayed"}'});
  });
  const newB=await context.newPage();await newB.goto(base,{waitUntil:'domcontentloaded'});await newB.locator('.account-badge button').waitFor({state:'visible'});
  stage=name+'_current_a_logout';
  const validLogout=newB.waitForResponse(r=>r.url()===base+'/api/v1/account/logout'&&r.request().method()==='POST');
  await newB.locator('.account-badge button').click();
  check(name+'_current_a_logout_works',(await validLogout).status()===200);
  await newB.locator('.account-panel:not([hidden]) input[name=login_name]').waitFor({state:'visible'});
  check(name+'_current_a_cookie_revoked',(await me(context)).status===401);
  await fillLogin(newB,accounts[1]);
  const b=await me(context);check(name+'_shared_cookie_changed_to_b',b.status===200&&b.account.user_id!==a.account.user_id);
  check(name+'_old_tab_still_displays_a',(await oldA.locator('.account-badge span').textContent())===accounts[0].login_name);
  stage=name+'_stale_a_logout';
  const staleResponse=oldA.waitForResponse(r=>r.url()===base+'/api/v1/account/logout'&&r.request().method()==='POST');
  await oldA.locator('.account-badge button').click();
  const stale=await staleResponse;
  const expected=stale.request().headers()['x-uca-account-subject'];
  const after=await me(context);
  if(patched){
    check(options['inject-local']?'patched_injected_local_helper':'current_served_asset_used',options['inject-local']?injected>0:injected===0);
    check('patched_logout_carries_captured_a',expected===JSON.stringify([a.account.tenant_id,a.account.user_id]));
    check('patched_stale_logout_rejected',stale.status()===401,{status:stale.status()});
    check('patched_b_session_remains_admitted',after.status===200&&after.account.user_id===b.account.user_id);
    await oldA.waitForFunction(name=>document.querySelector('.account-badge span')?.textContent===name,accounts[1].login_name);
    check('patched_old_tab_clears_old_view_and_reloads_b',true);
    check('patched_b_still_valid_after_old_tab_reload',(await me(context)).account?.user_id===b.account.user_id);
  } else {
    check('baseline_missing_subject_header',expected===undefined);
    check('baseline_bug_reproduced_b_was_revoked',stale.status()===200&&after.status===401,{logout_status:stale.status(),b_me_status:after.status});
  }
  await context.unrouteAll({behavior:'ignoreErrors'});
  await oldA.unrouteAll({behavior:'ignoreErrors'});
  await context.close();
}
(async()=>{
  browser=await chromium.launch({executablePath:options['chrome-path'],headless:true});
  if(options['reproduce-old-preview'])await scenario(false);
  await scenario(true);check('zero_model_requests',report.model_requests===0);report.passed=true;
})().catch(error=>{report.passed=false;report.failed_stage=stage;report.error_class=error?.name||'Error';console.log(JSON.stringify({failed_stage:stage,error_class:report.error_class}));process.exitCode=1;}).finally(async()=>{
  report.completed_at=new Date().toISOString();
  const text=JSON.stringify(report,null,2);if(accounts.some(a=>text.includes(a.password)))throw Error('secret_output_rejected');
  fs.mkdirSync(path.dirname(output),{recursive:true});fs.writeFileSync(output,text,{mode:0o600});
  console.log(JSON.stringify({passed:report.passed,failed_stage:report.failed_stage,checks:report.checks.length,model_requests:report.model_requests}));
  if(browser)await browser.close();
});
