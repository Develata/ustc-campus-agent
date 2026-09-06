(() => {
  "use strict";
  const mounted = new WeakMap();
  const PENDING_KEY = "uca.plugin-management.pending.v1";
  const HEADERS = {Accept:"application/json", "Content-Type":"application/json", "x-ustc-client-protocol-major":"1"};
  const STATES = {installeddisabled:"已安装 · 未启用", disabled:"已停用", enabled:"已启用", revoked:"已撤销", uninstalled:"已卸载"};
  const text = (value, max = 4096) => typeof value === "string" && value.length <= max;
  const nonempty = (value, max = 256) => text(value,max) && value.length > 0;
  function node(tag, cls, value) {
    const element = document.createElement(tag);
    if (cls) element.className = cls;
    if (value !== undefined) element.textContent = value;
    return element;
  }
  function button(label, action, kind) {
    const element = node("button", "plugin-manage-button", label); element.type = "button";
    if (kind) element.dataset.pluginAction = kind;
    element.addEventListener("click", action); return element;
  }
  function packageValid(pkg) {
    const fields = pkg?.fields;
    return typeof pkg?.available === "boolean" && nonempty(pkg?.package_id) && nonempty(pkg.version,128) && nonempty(pkg.name,256) && text(pkg.description) &&
      nonempty(pkg.catalog_revision) && /^sha256:[a-f0-9]{64}$/.test(pkg.package_digest) && ["skill","mcp","mixed"].includes(pkg.kind) &&
      Array.isArray(pkg.capabilities) && pkg.capabilities.length <= 64 && pkg.capabilities.every(value => nonempty(value,128)) &&
      new Set(pkg.capabilities).size === pkg.capabilities.length && Array.isArray(fields) && fields.length <= 128 &&
      fields.every(field => nonempty(field.key,64) && ["text","integer","boolean"].includes(field.kind) && typeof field.required === "boolean" &&
        (field.kind !== "text" || Number.isSafeInteger(field.max_bytes) && field.max_bytes > 0 && field.max_bytes <= 4096) &&
        (field.kind !== "integer" || Array.isArray(field.integer_bounds) && field.integer_bounds.length === 2 && field.integer_bounds.every(Number.isInteger))) &&
      new Set(fields.map(field=>field.key)).size === fields.length && (!pkg.installation || installationValid(pkg.installation));
  }
  function installationValid(value) {
    return nonempty(value.id) && nonempty(value.revision) && Object.hasOwn(STATES,value.state) &&
      value.values && typeof value.values === "object" && !Array.isArray(value.values) && Object.keys(value.values).length <= 128 &&
      Object.values(value.values).every(item => typeof item === "boolean" || typeof item === "string" && item.length <= 4096 || Number.isSafeInteger(item)) &&
      Array.isArray(value.active_capabilities) && value.active_capabilities.length <= 64 && value.active_capabilities.every(item=>nonempty(item,128));
  }
  async function request(url, body) {
    const controller = new AbortController(); let timeout;
    try {
      return await Promise.race([
        fetch(url,{method:body === undefined ? "GET" : "POST", credentials:"same-origin",cache:"no-store",redirect:"error",headers:HEADERS,signal:controller.signal,body}).then(async response => {
          if (!response.ok) {
            const error = Error("http"); error.status = response.status;
            try {
              const raw = await response.text();
              if (raw.length <= 65536) {
                const failure = JSON.parse(raw);
                if (failure?.schema === "plugin-error/v1" && typeof failure.error === "string") error.code = failure.error;
              }
            } catch (_) { /* An unconfirmed response never establishes a capacity rejection. */ }
            throw error;
          }
          const raw = await response.text(); if (raw.length > 1024 * 1024) throw Error("response");
          return JSON.parse(raw);
        }),
        new Promise((_,reject)=>{ timeout=setTimeout(()=>{reject(Error("timeout"));controller.abort();},15000); })
      ]);
    } finally { clearTimeout(timeout); }
  }
  function mount(root) {
    if (!root || mounted.has(root)) return mounted.get(root);
    let packages = [], updates = [], versionReviews = new Map(), probes = new Map(), pending = null, busy = false, alive = true, sequence = 0, recoveryBlocked = false;
    const title = node("h2","","已接入的插件");
    const intro = node("p","plugin-manage-intro","按需安装校园指南或管理员已接入的 MCP。每项权限由你确认，启用后才可在对话中使用。");
    const modelNote = node("p","plugin-manage-model-note"); modelNote.setAttribute("role","status");
    function modelCapability() {
      const selection = window.UcaModelSelection;
      const offline = selection?.selected?.provider?.mode === "mock";
      modelNote.replaceChildren(); modelNote.hidden = !offline && selection?.toolCalling !== false;
      if (!modelNote.hidden) {
        modelNote.append(document.createTextNode(offline ? "当前为离线演示，只调用内置校园工具，不会调用已安装的 MCP 或 Skill。" : "当前模型仅支持聊天，不能调用插件。"));
        const link = node("a","","返回对话切换到支持工具的模型"); link.href = "#chat"; modelNote.append(link);
      }
    }
    window.addEventListener("uca:model-selection",modelCapability); modelCapability();
    const status = node("p","plugin-manage-status"); status.setAttribute("role","status"); status.setAttribute("aria-live","polite");
    const pendingBox = node("div","plugin-manage-pending"); pendingBox.setAttribute("role","alert");
    const cards = node("div","plugin-manage-cards");
    const refresh = button("刷新状态",()=>load(),"refresh");
    const note = node("p","plugin-manage-note","当前支持只读 Skill 上下文和公开读取 MCP。修改配置或停用后，需要重新检查与审核权限。");
    root.classList.add("plugin-management"); root.replaceChildren(title,intro,modelNote,refresh,status,pendingBox,cards,note);
    const importer = node("details","plugin-import-review");
    importer.append(node("summary","","准备 MCP / Skill 导入包"),node("p","","填写候选描述后生成可审阅文件。生成不会安装、联网或执行代码；审阅通过后由管理员接入，再完成安装与授权。"));
    const input = node("textarea", "plugin-import-input"); input.rows=12;
    input.setAttribute("aria-label","导入候选 JSON");
    input.value = JSON.stringify({schema:"plugin-import-preview/v1",package_id:"community.my-guide",version:"0.1.0",display_name:"我的指南",source:"请填写来源与使用条件",skill:"---\nname: my-guide\ndescription: 我的任务指南\n---\n请填写需要审阅的内容。\n",mcp:null},null,2);
    const output = node("div","plugin-import-output"); output.setAttribute("role","status");
    const generate = button("生成审阅包",async()=>{
      generate.disabled=true; output.replaceChildren();
      try {
        if (input.value.length > 100000) throw Error("capacity");
        const body=JSON.stringify(JSON.parse(input.value));
        const result=await request("/api/v1/plugins/import-preview",body);
        if(result.schema!=="plugin-import-review/v1" || result.admitted!==false || !/^sha256:[a-f0-9]{64}$/.test(result.review_digest) || !result.files || typeof result.files!=="object") throw Error("response");
        output.append(node("p","",`待审阅 · ${result.review_digest}`));
        for(const warning of result.warnings || []) output.append(node("p","",String(warning)));
        for(const [path,content] of Object.entries(result.files)) {
          if(!text(path,256)||!text(content,100000)) throw Error("response");
          const section=node("details","");section.append(node("summary","",path),node("pre","",content));output.append(section);
        }
        output.append(button("下载审阅包 JSON",()=>{
          const url=URL.createObjectURL(new Blob([JSON.stringify(result,null,2)],{type:"application/json"}));
          const link=node("a","");link.href=url;link.download="plugin-import-review.json";link.click();setTimeout(()=>URL.revokeObjectURL(url),1000);
        }));
      } catch (_) {output.replaceChildren(node("p","","候选格式或能力不受支持。检查 JSON、Skill 名称、HTTPS 地址和公开读取能力映射。"));}
      finally {generate.disabled=false;}
    });
    importer.append(input,generate,output);root.append(importer);

    try {
      const saved = sessionStorage.getItem(PENDING_KEY);
      if (saved) {
        const parsed = JSON.parse(saved);
        if (!["plugin-command/v1","plugin-update/v1"].includes(parsed.schema) || !nonempty(parsed.request_id,80) || !parsed.intent || !(parsed.schema === "plugin-update/v1" ? ["apply","rollback","confirm"] : ["install","configure","grant","enable","disable","revoke"]).includes(parsed.intent.action)) throw Error("pending");
        pending = saved;
      }
    } catch (_) { recoveryBlocked = true; status.textContent = "无法恢复上次操作记录。为避免重复提交，当前暂不允许修改。"; }
    function locks() {
      root.setAttribute("aria-busy",String(busy));
      for (const control of cards.querySelectorAll("button,input,select")) control.disabled = busy || !!pending || recoveryBlocked || control.dataset.fixedDisabled === "true";
      refresh.disabled = busy;
      const retry = pendingBox.querySelector("button"); if (retry) retry.disabled = busy;
    }
    function showPending() {
      pendingBox.replaceChildren(); pendingBox.hidden = !pending;
      if (pending) pendingBox.append(node("p","","上次操作的结果尚未确认。请先重试原操作；请求内容和编号保持不变，服务端会回读已有结果。"),button("重试原操作",()=>sendPending(),"retry"));
      locks();
    }
    async function load() {
      if (busy || !alive) return;
      const token = ++sequence; busy = true; locks();
      if (!pending && !recoveryBlocked) status.textContent = "正在读取插件状态…";
      try {
        const data = await request("/api/v1/plugins");
        if (data.schema !== "plugin-lifecycle/v1" || !Array.isArray(data.packages) || data.packages.length > 64 || !data.packages.every(packageValid) || new Set(data.packages.map(pkg=>JSON.stringify([pkg.package_id,pkg.version,pkg.catalog_revision,pkg.installation?.id ?? null]))).size !== data.packages.length) throw Error("response");
        if (!alive || token !== sequence) return;
        packages = data.packages;
        updates = Array.isArray(data.updates) ? data.updates : [];
        for (const [id,review] of versionReviews) if (!packages.some(pkg=>pkg.installation?.id === id && pkg.installation.revision === review.installation_revision)) versionReviews.delete(id);
        for (const [id,probe] of probes) if (!packages.some(pkg=>pkg.installation?.id === id && pkg.installation.revision === probe.revision)) probes.delete(id);
        render(); if (!pending && !recoveryBlocked) status.textContent = packages.length ? "状态已更新。安装和授权会保存在当前服务端。" : "当前没有已接入的插件包。";
      } catch (_) { if (alive && token === sequence) { packages = []; cards.replaceChildren(); status.textContent = "暂时无法读取插件状态，请确认本机服务可用后刷新。"; } }
      finally { if (alive && token === sequence) { busy = false; showPending(); } }
    }
    async function command(intent, schema = "plugin-command/v1") {
      if (busy || pending || recoveryBlocked) return;
      const body = JSON.stringify({schema,request_id:crypto.randomUUID(),intent});
      try { sessionStorage.setItem(PENDING_KEY,body); pending = body; }
      catch (_) { recoveryBlocked = true; status.textContent = "无法保存操作编号，本次未提交。"; locks(); return; }
      await sendPending();
    }
    async function sendPending() {
      if (busy || !pending || !alive) return;
      const original = pending; busy = true; showPending(); status.textContent = "正在确认操作结果…";
      let known = false, message = "";
      try {
        const updateCommand = JSON.parse(original).schema === "plugin-update/v1";
        const result = await request(updateCommand ? "/api/v1/plugins/updates" : "/api/v1/plugins/commands",original);
        if (updateCommand) {
          if (result.schema !== "plugin-update-view/v1" || !nonempty(result.update_id) || !nonempty(result.installation_revision)) throw Error("response");
          result.accepted=true; versionReviews.clear();probes.clear();
        } else if (result.schema !== "plugin-command-result/v1" || typeof result.accepted !== "boolean" || typeof result.replayed !== "boolean" || !nonempty(result.installation_id) || result.revision !== null && !nonempty(result.revision)) throw Error("response");
        known = true;
        const action = JSON.parse(original).intent.action;
        if (["configure","disable","revoke"].includes(action)) probes.clear();
        message = result.accepted ? (result.replayed ? "已确认上次操作结果，没有重复执行。" : "操作已完成。") : "服务端未接受此操作，请根据最新状态重新检查。";
      } catch (error) {
        if (error.status === 429 && error.code === "plugin_capacity_exceeded") {
          known = true; message = "已达到插件或工具容量上限，本次操作未提交。请调整启用的插件或联系管理员。";
        } else if (error.status >= 400 && error.status < 500 && ![408,429].includes(error.status)) {
          known = true; message = error.status === 409 ? "状态已变化，本次操作未接受。请按刷新后的状态重试。" : "操作未被接受，请检查配置、权限或组件检查结果。";
        } else message = "连接中断或服务端结果不完整，尚不能确定操作是否完成。";
      } finally {
        if (known) {
          try { sessionStorage.removeItem(PENDING_KEY); pending = null; }
          catch (_) { recoveryBlocked = true; message = "操作结果已返回，但本地记录未能清除。请保留当前页面。"; }
        }
        busy = false;
        if (alive) { showPending(); if (known && !recoveryBlocked) await load(); status.textContent = message; }
      }
    }
    async function probe(pkg) {
      if (busy || pending || !pkg.installation) return;
      const installation = pkg.installation; busy = true; locks(); status.textContent = "正在检查组件与只读工具清单…";
      try {
        const result = await request("/api/v1/plugins/probe",JSON.stringify({schema:"plugin-probe/v1",installation_id:installation.id,expected_revision:installation.revision}));
        if (result.schema !== "plugin-probe-result/v1" || result.installation_id !== installation.id || result.revision !== installation.revision || !/^sha256:[a-f0-9]{64}$/.test(result.readiness_digest) || !Array.isArray(result.tools) || result.tools.length > 64 || !result.tools.every(tool=>nonempty(tool.name,128) && text(tool.description) && nonempty(tool.capability,128))) throw Error("response");
        probes.set(installation.id,result); render(); status.textContent = "检查完成。请核对工具与权限，再确认启用。检查没有调用业务工具。";
      } catch (_) { probes.delete(installation.id); render(); status.textContent = "组件检查未完成。请核对配置和服务连接后重新检查。"; }
      finally { busy = false; locks(); }
    }
    function bound(pkg, action, extra = {}) { return {action,installation_id:pkg.installation.id,expected_revision:pkg.installation.revision,...extra}; }
    function render() {
      cards.replaceChildren();
      for (const pkg of packages) {
        const card = node("article","plugin-manage-card"); card.dataset.packageId = pkg.package_id; card.dataset.available = String(pkg.available !== false); if (pkg.installation) card.dataset.installationId = pkg.installation.id;
        const heading = node("div","plugin-manage-heading");
        heading.append(node("h3","",pkg.name),node("span","plugin-manage-state",pkg.installation ? STATES[pkg.installation.state] : "未安装"));
        card.append(heading,node("p","plugin-manage-description",pkg.package_id === "ustc.campus-guide" ? "帮助 Agent 核对校园信息的来源，组织选课问题和日历任务。" : pkg.description),node("p","plugin-manage-meta",`${pkg.kind === "mixed" ? "Skill + MCP 组合包" : pkg.kind === "skill" ? "Skill 使用指南" : "MCP 只读工具"} · v${pkg.version}`));
        if (pkg.available === false) card.append(node("p","plugin-manage-unavailable","包来源暂不可用。保留历史安装状态，可停用或撤销；无法配置、检查或启用。"));
        if (!pkg.installation && pkg.available !== false) card.append(button("安装",()=>command({action:"install",package_id:pkg.package_id,version:pkg.version,catalog_revision:pkg.catalog_revision,package_digest:pkg.package_digest}),"install"));
        else if (!pkg.installation) { /* Missing sources never create an install intent. */ }
        else if (["revoked","uninstalled"].includes(pkg.installation.state)) card.append(node("p","plugin-manage-note","此安装已结束，不能继续授权或启用。"));
        else {
          const enabled = pkg.installation.state === "enabled";
          if (enabled) {
            if (pkg.available !== false) card.append(node("p","plugin-manage-ready","已启用，Agent 使用时仍会检查当前权限。"));
            card.append(button("停用",()=>command(bound(pkg,"disable")),"disable"));
          }
          else if (pkg.available !== false) {
            configurationForm(card,pkg);
            card.append(button("检查组件",()=>probe(pkg),"probe"));
            const checked = probes.get(pkg.installation.id);
            if (checked) review(card,pkg,checked);
            else card.append(node("p","plugin-manage-note","检查组件后，可查看待启用的工具清单。"));
          }
          if (pkg.available !== false) versionControls(card,pkg);
          const advanced = node("details","plugin-manage-advanced"); advanced.append(node("summary","","安装详情与撤销"),node("p","plugin-manage-meta",pkg.package_id));
          const confirm = node("div","plugin-manage-revoke"); confirm.hidden = true;
          confirm.append(node("p","","确认撤销此安装？后续读取和调用将被拒绝，历史记录保留。"),button("确认撤销",()=>command(bound(pkg,"revoke")),"confirm-revoke"));
          advanced.append(button("撤销安装",()=>{confirm.hidden=false;},"revoke"),confirm); card.append(advanced);
        }
        cards.append(card);
      }
      locks();
    }
    async function reviewVersion(pkg, intent) {
      if (busy || pending || recoveryBlocked) return;
      busy=true;locks();status.textContent="正在核对两个版本的组件、配置与权限变化…";
      try {
        const result=await request("/api/v1/plugins/updates",JSON.stringify({schema:"plugin-update/v1",request_id:crypto.randomUUID(),intent}));
        if(result.schema!=="plugin-update-view/v1" || result.installation_id!==pkg.installation.id || result.installation_revision!==pkg.installation.revision || !nonempty(result.plan_digest))throw Error("response");
        result.review_action=intent.action;versionReviews.set(pkg.installation.id,result);render();status.textContent="检查完成。请核对目标版本、权限和来源变化后确认。";
      } catch (_) {status.textContent="版本检查未通过。请先停用，确认当前配置同时适用于两个版本，并保证来源可用。";}
      finally {busy=false;locks();}
    }
    function versionControls(card,pkg) {
      const panel=node("details","plugin-manage-versions");panel.append(node("summary","","版本更新与回滚"));
      if(pkg.installation.state==="enabled") {panel.append(node("p","","请先停用插件，再检查和切换版本。"));card.append(panel);return;}
      const active=updates.find(update=>update.installation_id===pkg.installation.id && update.state==="appliedpendingconfirmation");
      if(active) {
        panel.append(node("p","",`已切换 ${active.rollback_version} → ${active.target_version}。旧授权已失效；重新检查和授权后才能启用。`));
        panel.append(button(`检查回滚到 ${active.rollback_version}`,()=>reviewVersion(pkg,bound(pkg,"review_rollback",{update_id:active.update_id})),"review-rollback"));
        const retain=node("div","");retain.hidden=true;
        retain.append(node("p","","确认保留当前版本并结束本次回滚窗口？"),button("确认保留当前版本",()=>command(bound(pkg,"confirm",{update_id:active.update_id}),"plugin-update/v1"),"confirm-version"));
        panel.append(button("保留当前版本",()=>{retain.hidden=false;},"retain-version"),retain);
      } else {
        const versions=packages.filter(candidate=>candidate.available && candidate.package_id===pkg.package_id && candidate.version!==pkg.version);
        if(!versions.length) panel.append(node("p","","当前目录没有其他已审阅版本。"));
        for(const target of versions) panel.append(button(`检查版本 ${target.version}`,()=>reviewVersion(pkg,bound(pkg,"preview",{target_version:target.version})),"preview-version"));
      }
      const review=versionReviews.get(pkg.installation.id);
      if(review) {
        const rollback=review.review_action==="review_rollback";
        panel.append(node("p","",rollback ? `将回滚到 ${review.rollback_version}，当前授权将失效。` : `版本 ${review.rollback_version} → ${review.target_version}；权限/来源分类：${review.change_class}。旧授权失效，新版本保持停用。`));
        const target=packages.find(candidate=>candidate.package_id===pkg.package_id && candidate.version===(rollback?review.rollback_version:review.target_version));
        if(target) panel.append(node("p","",`目标能力：${target.capabilities.join("、")}。${target.description}`));
        panel.append(node("code","",review.plan_digest));
        const label=node("label","plugin-manage-confirm");const checkbox=node("input","");checkbox.type="checkbox";label.append(checkbox,document.createTextNode("我已核对该版本和能力变化，同意切换并重新授权。"));
        const apply=button(rollback?"确认回滚":"确认更新",()=>{
          if(!checkbox.checked)return;
          const extra=rollback?{update_id:review.update_id,rollback_readiness:review.rollback_readiness}:{update_id:review.update_id,target_version:review.target_version,plan_digest:review.plan_digest,target_readiness:review.target_readiness,rollback_readiness:review.rollback_readiness};
          command(bound(pkg,rollback?"rollback":"apply",extra),"plugin-update/v1");
        },rollback?"rollback-version":"apply-version");
        apply.dataset.fixedDisabled="true";checkbox.addEventListener("change",()=>{apply.dataset.fixedDisabled=String(!checkbox.checked);locks();});panel.append(label,apply);
      }
      card.append(panel);
    }
    function configurationForm(card,pkg) {
      if (!pkg.fields.length) { card.append(node("p","plugin-manage-note","此指南无需额外配置。")); return; }
      const details = node("details","plugin-manage-configuration"); details.open = pkg.fields.some(field=>field.required && !Object.hasOwn(pkg.installation.values,field.key));
      details.append(node("summary","","配置连接"));
      const form = node("form","plugin-manage-form"); const controls = new Map();
      for (const field of pkg.fields) {
        const label = node("label","plugin-manage-field",`${field.key}${field.required ? "（必填）" : "（可选）"}`);
        const value = pkg.installation.values[field.key];
        const input = node(field.kind === "boolean" ? "select" : "input"); input.dataset.configKey = field.key;
        if (field.kind === "boolean") {
          for (const [raw,display] of [["","请选择"],["true","是"],["false","否"]]) { const option=node("option","",display); option.value=raw; input.append(option); }
          input.value = value === undefined ? "" : String(value);
        } else {
          input.type = field.kind === "integer" ? "number" : "text"; input.value = value === undefined ? "" : String(value);
          if (field.kind === "text") input.maxLength = field.max_bytes;
          else { input.step="1"; input.min=String(Math.max(field.integer_bounds[0],Number.MIN_SAFE_INTEGER));input.max=String(Math.min(field.integer_bounds[1],Number.MAX_SAFE_INTEGER)); }
        }
        input.required = field.required; label.append(input); form.append(label); controls.set(field.key,input);
      }
      const save = button("保存配置",()=>{},"configure"); save.type = "submit"; form.append(save,node("p","plugin-manage-note","保存后，需要重新检查组件并逐项审核权限。不要在地址中填写密钥。"));
      form.addEventListener("submit",event=>{
        event.preventDefault(); if (busy || pending || !form.reportValidity()) return;
        const values = {};
        for (const field of pkg.fields) {
          const raw = controls.get(field.key).value;
          if (raw === "" && !field.required) continue;
          if (field.kind === "integer") { const value=Number(raw); if (!Number.isSafeInteger(value)) {status.textContent="请输入可精确表示的整数。";return;} values[field.key]=value; }
          else if (field.kind === "boolean") values[field.key]=raw === "true";
          else { if (new TextEncoder().encode(raw).length>field.max_bytes) {status.textContent="配置文字超过允许长度。";return;} values[field.key]=raw; }
        }
        command(bound(pkg,"configure",{values}));
      });
      details.append(form); card.append(details);
    }
    function review(card,pkg,checked) {
      const panel = node("section","plugin-manage-review"); panel.append(node("h4","","核对本次启用内容"));
      const tools = node("ul","plugin-manage-tools");
      for (const tool of checked.tools) { const item=node("li");item.append(node("strong","",tool.name),node("span","",tool.description),node("code","",tool.capability));tools.append(item); }
      if (!checked.tools.length) tools.append(node("li","","只读 Skill 上下文，不提供执行权限。"));
      panel.append(tools);
      for (const capability of pkg.capabilities) {
        const row=node("div","plugin-manage-grant"); row.append(node("code","",capability));
        if (pkg.installation.active_capabilities.includes(capability)) row.append(node("span","plugin-manage-granted","已授权"));
        else { const grant=button("明确授权此权限",()=>command(bound(pkg,"grant",{capability})),"grant");grant.dataset.capability=capability;row.append(grant); }
        panel.append(row);
      }
      const confirmation=node("label","plugin-manage-confirm"); const checkbox=node("input");checkbox.type="checkbox";checkbox.dataset.pluginReview="true";
      confirmation.append(checkbox,document.createTextNode("我已核对以上内容，同意按此清单启用。"));
      const enable=button("确认启用",()=>{if(checkbox.checked)command(bound(pkg,"enable",{readiness_digest:checked.readiness_digest}));},"enable");
      const update=()=>{ enable.dataset.fixedDisabled=String(!checkbox.checked || !pkg.capabilities.every(capability=>pkg.installation.active_capabilities.includes(capability)));locks(); };
      checkbox.addEventListener("change",update);enable.dataset.fixedDisabled="true";
      panel.append(confirmation,enable);card.append(panel);
    }
    showPending(); void load();
    const api={refresh:load,destroy(){alive=false;sequence++;window.removeEventListener("uca:model-selection",modelCapability);mounted.delete(root);}};
    mounted.set(root,api);return api;
  }
  window.UcaPluginManagement={mount};
})();
