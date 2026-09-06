(() => {
  "use strict";
  const mounted = new WeakMap();
  const ENDPOINT = "/api/v1/market/packages";
  const REQUEST_TIMEOUT_MS = 15000;
  const labels = {planned:"规划中",development:"开发中",implemented:"已有实现",SkillComponent:"Skill",McpServerComponent:"MCP 服务",NativeRustComponent:"原生工具",DeclarativeResourcePack:"资料包"};
  const presentations = Object.freeze({
    "ustc.affairs-navigator": ["USTC Affairs Navigator", "办事导航", "核对办事条件、办理步骤、有效时间与官方来源。"],
    "ustc.change-radar": ["USTC ChangeRadar", "变更雷达", "对照经过复核的通知版本，了解变化及其影响。"],
    "ustc.opportunity-graph": ["Campus Opportunity Graph", "机会图谱", "整理课程与校园机会资料，为个性化推荐提供依据。"],
    "ustc.simple-calendar": ["USTC Simple Calendar", "简单日历", "记录个人日历事项，管理已保存的标题与时间。"]
  });
  function presentation(pkg) {
    const known = presentations[pkg.package_id];
    return known && known[0] === pkg.display_name ? {name:known[1],description:known[2]} : {name:pkg.display_name,description:pkg.description || "此插件尚未提供简介。"};
  }
  // Mirror the M20 wire-size bounds before rendering; Rust remains the semantic authority.
  const encoder = new TextEncoder();
  const boundedText = (value, maximum) => typeof value === "string" && value.length > 0 && value.length <= maximum && encoder.encode(value).length <= maximum;
  const boundedList = (value, maximum, predicate) => Array.isArray(value) && value.length <= maximum && value.every(predicate);
  const summaryValid = value => value && boundedText(value.package_id,256) && boundedText(value.version,128) && boundedText(value.publisher,128) &&
    boundedText(value.display_name,256) && boundedText(value.package_digest,71) && ["FirstParty","VerifiedCommunityText","VerifiedRemoteMcp"].includes(value.tier) &&
    ["planned","development","implemented"].includes(value.implementation_status) && (value.description === null || boundedText(value.description,4096)) &&
    Number.isSafeInteger(value.component_count) && value.component_count >= 0 && value.component_count <= 64 &&
    boundedList(value.requested_capabilities,64,capability => boundedText(capability,128));
  function envelope(value, schema) {
    if (!value || value.schema !== schema || !boundedText(value.catalog_digest,71) || !boundedText(value.catalog_revision,128) || value.management_available !== false) throw Error("invalid_catalog");
  }
  function node(tag, className, text) {
    const el = document.createElement(tag);
    if (className) el.className = className;
    if (text !== undefined) el.textContent = text;
    return el;
  }
  function button(label, action, id) {
    const el = node("button", "market-button", label);
    el.type = "button";
    if (id) el.id = id;
    el.addEventListener("click", action);
    return el;
  }
  function mount(root, tabs) {
    if (!root || mounted.has(root)) return mounted.get(root);
    let catalog = null, selected = null, sequence = 0, controller = null, started = false, alive = true;
    const title = node("h2", "", "插件目录");
    title.id = "market-catalog-title";
    const intro = node("p", "market-intro", "查看插件包的版本、组成、申请权限与资料来源。");
    const toolbar = node("div", "market-toolbar");
    const searchLabel = node("label", "market-search", "搜索插件");
    const search = node("input");
    search.id = "market-search"; search.type = "search"; search.maxLength = 200;
    search.placeholder = "名称、能力或发布者";
    searchLabel.append(search);
    const refresh = button("刷新目录", () => loadCatalog(true), "market-refresh");
    toolbar.append(searchLabel, refresh);
    const status = node("p", "market-status");
    status.id = "market-status"; status.setAttribute("role", "status"); status.setAttribute("aria-live", "polite");
    const content = node("div", "market-content"); content.id = "market-content";
    const notice = node("p", "market-management-note", "此处展示目录中的包声明。已支持的 MCP / Skill 包可从「管理 MCP 与 Skills」安装、配置和授权；其他组件的通用安装仍在开发中。");
    root.replaceChildren(title, intro, toolbar, status, content, notice);
    if (!tabs) root.setAttribute("aria-labelledby", title.id);
    function invalidate() { sequence += 1; if (controller) controller.abort(); controller = null; }
    function begin(message) {
      invalidate(); controller = new AbortController();
      status.textContent = message; content.replaceChildren(); content.setAttribute("aria-busy", "true");
      return {token:sequence, controller, signal:controller.signal};
    }
    async function get(url, request) {
      let timer, cancel;
      const cancelled = new Promise((_, reject) => {
        cancel = () => reject(Error("catalog_cancelled"));
        request.signal.addEventListener("abort", cancel, {once:true});
        timer = setTimeout(() => { reject(Error("catalog_timeout")); request.controller.abort(); }, REQUEST_TIMEOUT_MS);
      });
      try {
        const response = (async () => {
          const result = await fetch(url, {method:"GET",cache:"no-store",credentials:"same-origin",redirect:"error",signal:request.signal,
            headers:{Accept:"application/json","x-ustc-client-protocol-major":"1"}});
          if (!result.ok) throw Error("catalog_unavailable");
          return result.json();
        })();
        return await Promise.race([response, cancelled]);
      } finally {
        clearTimeout(timer);
        request.signal.removeEventListener("abort", cancel);
      }
    }
    function valid(token) { return alive && token === sequence; }
    function failed(message, retry) {
      content.removeAttribute("aria-busy"); status.textContent = message;
      content.replaceChildren(button("重试", retry, "market-retry"));
    }
    function renderList(focusPackage) {
      selected = null; toolbar.hidden = false; content.removeAttribute("aria-busy"); content.replaceChildren();
      const query = search.value.trim().toLocaleLowerCase();
      const packages = catalog.packages.filter(pkg => [presentation(pkg).name,presentation(pkg).description,pkg.display_name,pkg.package_id,pkg.description || "",pkg.publisher,...pkg.requested_capabilities].join(" ").toLocaleLowerCase().includes(query));
      status.textContent = packages.length ? `${packages.length} 个插件包 · 版本由服务端目录提供` : (query ? "没有匹配的插件，试试其他关键词。" : "目录中暂时没有插件包。");
      if (!packages.length) {
        if (query) content.append(button("清空搜索", () => { search.value = ""; renderList(); search.focus(); }, "market-clear-search"));
        return;
      }
      const list = node("ul", "market-packages");
      for (const pkg of packages) {
        const item = node("li", "market-package");
        const open = button("", () => loadDetail(pkg), undefined);
        open.className = "market-package-open";
        open.dataset.packageId = pkg.package_id;
        open.dataset.packageVersion = pkg.version;
        open.setAttribute("aria-label", `查看 ${presentation(pkg).name} ${pkg.version} 的包详情`);
        const heading = node("span", "market-package-heading");
        heading.append(node("strong", "", presentation(pkg).name),node("span", "market-version", `v${pkg.version}`));
        open.append(heading,node("span", "market-package-description", presentation(pkg).description));
        const meta = node("span", "market-package-meta");
        meta.append(node("span", "", `${pkg.publisher} · ${labels[pkg.implementation_status] || pkg.implementation_status}`),node("span", "market-open-label", "查看详情 →"));
        open.append(meta); item.append(open); list.append(item);
        if (focusPackage === pkg.package_id) queueMicrotask(() => { if (open.isConnected) open.focus({preventScroll:true}); });
      }
      content.append(list);
    }
    async function loadCatalog(focus = false) {
      started = true; catalog = null; selected = null; toolbar.hidden = false;
      const request = begin("正在读取插件目录…");
      try {
        const data = await get(ENDPOINT, request);
        envelope(data, "market-catalog/v1");
        if (!boundedList(data.packages,64,summaryValid) || new Set(data.packages.map(pkg => `${pkg.package_id}@${pkg.version}`)).size !== data.packages.length) throw Error("invalid_catalog");
        if (!valid(request.token)) return;
        catalog = data; renderList(); if (focus) search.focus({preventScroll:true});
      } catch (error) { if (valid(request.token)) failed(error.message === "catalog_timeout" ? "读取插件目录超时，请重试。" : "暂时无法读取插件目录，请稍后重试。", () => loadCatalog(true)); }
    }
    function back(pkg) {
      invalidate(); renderList(pkg.package_id); root.scrollIntoView({block:"start"});
    }
    function detailHeader(pkg) {
      const backButton = button("← 返回插件目录", () => back(pkg), "market-back");
      const heading = node("h3", "market-detail-title", presentation(pkg).name);
      heading.id = "market-detail-title"; heading.tabIndex = -1;
      content.append(backButton, heading);
      if (!root.hidden) { heading.focus({preventScroll:true}); root.scrollIntoView({block:"start"}); }
    }
    function fieldList(titleText, entries) {
      const section = node("section", "market-detail-section"); section.append(node("h4", "", titleText));
      const dl = node("dl", "market-fields");
      for (const [key,value] of entries) dl.append(node("dt", "", key),node("dd", "", value));
      section.append(dl); return section;
    }
    async function loadDetail(pkg) {
      selected = pkg; toolbar.hidden = true;
      const snapshot = catalog;
      const request = begin("正在读取包详情…"); detailHeader(pkg);
      try {
        const data = await get(`${ENDPOINT}/${encodeURIComponent(pkg.package_id)}/${encodeURIComponent(pkg.version)}`, request);
        envelope(data,"market-package/v1");
        if (!summaryValid(data.package) || data.catalog_digest !== snapshot.catalog_digest || data.catalog_revision !== snapshot.catalog_revision ||
          data.package.package_id !== pkg.package_id || data.package.version !== pkg.version || data.package.package_digest !== pkg.package_digest) throw Error("catalog_changed");
        if (!boundedList(data.components,64,c => c && ["SkillComponent","McpServerComponent","NativeRustComponent","DeclarativeResourcePack"].includes(c.kind) && boundedText(c.path,512) && (c.mode === null || boundedText(c.mode,64))) ||
          data.components.length !== data.package.component_count || !boundedList(data.source_policy,32,p => p && boundedText(p.key,64) && boundedText(p.value,4096)) ||
          !data.install_policy || !["FirstPartySystemPlugin","UserInstalledPlugin"].includes(data.install_policy.class) ||
          !["default_installed","default_enabled","user_disable_allowed"].every(key => typeof data.install_policy[key] === "boolean")) throw Error("invalid_catalog");
        if (!valid(request.token) || selected !== pkg) return;
        status.textContent = "包声明详情 · 查看不会安装或授权插件";
        content.removeAttribute("aria-busy"); content.replaceChildren(); detailHeader(pkg);
        content.append(node("p", "market-detail-description", presentation(data.package).description));
        content.append(fieldList("版本与发布者", [["包标识",pkg.package_id],["版本",pkg.version],["发布者",data.package.publisher],["类别",data.package.tier],["开发状态",labels[data.package.implementation_status] || data.package.implementation_status]]));
        const components = node("section", "market-detail-section"); components.append(node("h4", "", "MCP、Skills 与其他组件"));
        if (!data.components.length) components.append(node("p", "market-detail-note", "此版本尚未声明组件。"));
        else {
          const list = node("ul", "market-components");
          for (const component of data.components) {
            const item = node("li"); item.append(node("strong", "", labels[component.kind] || component.kind),node("code", "",component.path));
            if (component.mode) item.append(node("span", "market-detail-note", component.mode));
            list.append(item);
          }
          components.append(list);
        }
        content.append(components);
        const capabilities = node("section", "market-detail-section"); capabilities.append(node("h4", "", "申请的权限"),node("p", "market-detail-note", "以下是包的权限声明，不表示你已授权。"));
        if (data.package.requested_capabilities.length) {
          const list = node("ul", "market-capabilities");
          for (const capability of data.package.requested_capabilities) list.append(node("li", "",capability));
          capabilities.append(list);
        } else capabilities.append(node("p", "", "此版本未申请权限。"));
        content.append(capabilities);
        content.append(fieldList("资料来源与使用条件",data.source_policy.length ? data.source_policy.map(p => [p.key,p.value]) : [["来源策略","此版本未声明来源策略。"]]));
        const provenance = node("details", "market-provenance"); provenance.append(node("summary", "", "核对原始声明与版本依据"));
        provenance.append(node("p", "market-detail-note", "默认策略是包声明，实际安装和启用状态将由服务端单独管理。"));
        provenance.append(fieldList("声明记录", [["原始名称",data.package.display_name],["原始简介",data.package.description || "未提供"],["默认安装",data.install_policy.default_installed ? "是" : "否"],["默认启用",data.install_policy.default_enabled ? "是" : "否"],["允许用户停用",data.install_policy.user_disable_allowed ? "是" : "否"],["安装类别",data.install_policy.class],["包摘要",pkg.package_digest],["目录版本",data.catalog_revision],["目录摘要",data.catalog_digest]]));
        content.append(provenance);
        if (!root.hidden) root.scrollIntoView({block:"start"});
      } catch (error) {
        if (!valid(request.token)) return;
        content.removeAttribute("aria-busy");
        status.textContent = error.message === "catalog_changed" ? "目录版本已变化，请刷新目录后重新打开详情。" : error.message === "catalog_timeout" ? "读取包详情超时，可以重试或返回目录。" : "暂时无法读取此版本详情，可以重试或返回目录。";
        content.append(button(error.message === "catalog_changed" ? "刷新目录" : "重试", error.message === "catalog_changed" ? () => loadCatalog(true) : () => loadDetail(pkg), "market-retry"));
      }
    }
    search.addEventListener("input", () => { if (catalog && !selected) renderList(); });
    const listeners = [];
    function listen(target, event, handler) { target.addEventListener(event,handler); listeners.push(() => target.removeEventListener(event,handler)); }
    if (tabs) {
      const controls = [tabs.capabilitiesTab,tabs.marketTab];
      function select(index, focus = false) {
        controls.forEach((control, current) => { control.setAttribute("aria-selected",String(current === index)); control.tabIndex = current === index ? 0 : -1; });
        tabs.capabilitiesPanel.hidden = index !== 0; root.hidden = index !== 1;
        if (focus) controls[index].focus({preventScroll:true});
        if (index === 1 && !started) loadCatalog();
      }
      controls.forEach((control,index) => {
        listen(control,"click",() => select(index));
        listen(control,"keydown",event => {
          let next;
          if (event.key === "ArrowRight" || event.key === "ArrowLeft") next = 1-index;
          else if (event.key === "Home") next = 0;
          else if (event.key === "End") next = 1;
          else return;
          event.preventDefault(); select(next,true);
        });
      });
      select(0);
    }
    const api = {dispose() { alive = false; invalidate(); listeners.forEach(remove => remove()); }};
    mounted.set(root, api); return api;
  }
  window.UcaMarketCatalog = Object.freeze({mount});
  mount(document.querySelector("#market-catalog"), {
    capabilitiesTab:document.querySelector("#plugin-tab-capabilities"),
    marketTab:document.querySelector("#plugin-tab-market"),
    capabilitiesPanel:document.querySelector("#plugin-capabilities")
  });
})();
