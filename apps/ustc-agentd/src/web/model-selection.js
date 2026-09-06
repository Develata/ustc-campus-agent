// The browser remembers only an opaque server-configured model ID.
window.UcaModelSelection = (() => {
  "use strict";
  const KEY = "uca.selected-model.v1";
  const trigger = document.querySelector("#chat-model-trigger");
  const label = document.querySelector("#chat-model-label");
  const menu = document.querySelector("#chat-model-menu");
  // Keep the transient menu outside scrolling composer/page containers.
  document.body.append(menu);
  const status = document.querySelector("#chat-model-note");
  const retry = document.querySelector("#chat-model-refresh");
  let entries = [], selectedId = null, loaded = false, loading = false, locked = false, storageFailed = false;
  try { selectedId = localStorage.getItem(KEY); } catch (_) { storageFailed = true; }
  const idValid = value => typeof value === "string" && /^[A-Za-z0-9._-]{1,64}$/.test(value);
  const labelValid = value => typeof value === "string" && value.trim().length > 0 && new TextEncoder().encode(value).length <= 128 && !/[\u0000-\u001f\u007f-\u009f]/u.test(value);
  const selected = () => loaded ? entries.find(entry => entry.id === selectedId) ?? null : null;
  const unavailable = () => locked || loading || !loaded;
  const items = () => [...menu.querySelectorAll(".model-menu-item")];
  function closeMenu(restoreFocus = false) {
    menu.hidden = true;
    trigger.setAttribute("aria-expanded", "false");
    if (restoreFocus && !trigger.disabled) trigger.focus({preventScroll:true});
  }
  function syncDisabled() {
    trigger.disabled = unavailable();
    retry.disabled = locked || loading;
    for (const item of items()) item.disabled = unavailable();
    if (unavailable()) closeMenu();
  }
  function positionMenu() {
    const viewport = window.visualViewport;
    const left = viewport?.offsetLeft ?? 0, top = viewport?.offsetTop ?? 0;
    const width = viewport?.width ?? innerWidth, height = viewport?.height ?? innerHeight;
    const anchor = trigger.getBoundingClientRect();
    const above = anchor.top - top - 20, below = top + height - anchor.bottom - 20;
    const openAbove = above >= Math.min(280, menu.scrollHeight) || above >= below;
    menu.style.maxWidth = `${Math.max(0, width - 24)}px`;
    menu.style.maxHeight = `${Math.max(0, Math.min(360, openAbove ? above : below))}px`;
    menu.style.left = `${Math.max(left + 12, Math.min(anchor.right - menu.offsetWidth, left + width - menu.offsetWidth - 12))}px`;
    menu.style.top = `${openAbove ? anchor.top - menu.offsetHeight - 8 : anchor.bottom + 8}px`;
  }
  function openMenu(fromEnd = false) {
    if (unavailable()) return;
    menu.hidden = false;
    trigger.setAttribute("aria-expanded", "true");
    positionMenu();
    const options = items();
    const target = options.find(item => item.dataset.modelId === selectedId) ?? (fromEnd ? options.at(-1) : options[0]);
    target?.focus({preventScroll:true});
    target?.scrollIntoView({block:"nearest"});
  }
  function choose(id) {
    if (unavailable() || !entries.some(entry => entry.id === id)) return;
    selectedId = id;
    try {localStorage.setItem(KEY, selectedId); storageFailed = false;} catch (_) {storageFailed = true;}
    closeMenu(true);
    render();
  }
  function render() {
    closeMenu(menu.contains(document.activeElement));
    const current = selected();
    label.textContent = current?.label ?? (loading ? "正在读取模型…" : loaded ? "选择模型" : "模型暂不可用");
    trigger.setAttribute("aria-label", current ? `选择模型，当前：${current.label}` : "选择模型");
    menu.replaceChildren();
    const heading = document.createElement("p");
    heading.className = "model-menu-heading"; heading.setAttribute("role", "presentation"); heading.textContent = "切换模型";
    menu.append(heading);
    for (const entry of entries) {
      const item = document.createElement("button");
      item.type = "button"; item.className = "model-menu-item"; item.tabIndex = -1; item.dataset.modelId = entry.id;
      item.setAttribute("role", "menuitemradio"); item.setAttribute("aria-checked", String(entry.id === current?.id));
      const copy = document.createElement("span"); copy.className = "model-menu-copy";
      const name = document.createElement("span"); name.className = "model-menu-name"; name.textContent = entry.label;
      const title = document.createElement("span"); title.className = "model-menu-title"; title.append(name);
      if (entry.id === "default") {
        const badge = document.createElement("span"); badge.className = "model-menu-default"; badge.textContent = "默认"; title.append(badge);
      }
      const capability = document.createElement("span"); capability.className = "model-menu-capability";
      capability.textContent = entry.tool_calling ? "支持工具调用" : "仅聊天";
      const check = document.createElement("span"); check.className = "model-menu-check"; check.setAttribute("aria-hidden", "true"); check.textContent = "✓";
      copy.append(title, capability); item.append(copy, check);
      item.addEventListener("click", () => choose(entry.id)); menu.append(item);
    }
    const hint = document.createElement("p"); hint.className = "model-menu-hint"; hint.setAttribute("role", "presentation");
    hint.textContent = "仅影响接下来发送的消息"; menu.append(hint);
    syncDisabled();
    retry.hidden = loaded && !!current;
    status.textContent = loading ? "正在读取服务端模型配置…" : !loaded ? "无法确认模型列表，暂不能发送消息。请重试。"
      : !current ? "原模型已不可用或选择未确认，请明确重新选择。"
      : current.tool_calling ? "支持 Agent 工具调用 · 每次执行仍检查权限" : "仅聊天 · 不提供插件工具调用";
    if (current && storageFailed) status.textContent += "。本次选择仅在当前页面保留。";
    status.classList.toggle("visually-hidden", !!current && !storageFailed);
    trigger.title = current ? `${current.label} · ${status.textContent}` : status.textContent;
    window.dispatchEvent(new Event("uca:model-selection"));
  }
  async function refresh() {
    if (loading || locked) return;
    loading = true; loaded = false; render();
    const controller = new AbortController(); let timer;
    try {
      const data = await Promise.race([
        fetch("/api/v1/agent/models", {headers:{Accept:"application/json", "X-USTC-Client-Protocol-Major":"1"},credentials:"same-origin",cache:"no-store",redirect:"error",signal:controller.signal}).then(async response => {
          if (!response.ok) throw Error("models");
          const body = await response.text(); if (new TextEncoder().encode(body).length > 65536) throw Error("models"); return JSON.parse(body);
        }),
        new Promise((_,reject) => {timer=setTimeout(()=>{reject(Error("timeout"));controller.abort();},10000);})
      ]);
      if (data?.schema !== "uca-agent-models/v1" || data.default_id !== "default" || !Array.isArray(data.models) || !data.models.length || data.models.length > 16 ||
        !data.models.every(entry => idValid(entry?.id) && labelValid(entry.label) && ["mock","local-chat","openai-compatible"].includes(entry.provider?.mode) &&
          typeof entry.provider.model === "string" && entry.provider.model.length > 0 && entry.provider.model.length <= 256 && typeof entry.tool_calling === "boolean" &&
          (entry.context_limit_tokens === null || Number.isSafeInteger(entry.context_limit_tokens) && entry.context_limit_tokens >= 1024)) ||
        new Set(data.models.map(entry=>entry.id)).size !== data.models.length || !data.models.some(entry=>entry.id===data.default_id)) throw Error("models");
      entries = data.models; loaded = true;
      if (selectedId === null && !storageFailed) selectedId = data.default_id;
    } catch (_) { entries = []; loaded = false; }
    finally {clearTimeout(timer);loading=false;render();}
  }
  trigger.addEventListener("click", () => menu.hidden ? openMenu() : closeMenu(true));
  trigger.addEventListener("keydown", event => {
    if (event.key === "ArrowDown" || event.key === "ArrowUp") {
      event.preventDefault(); event.stopPropagation(); openMenu(event.key === "ArrowUp");
    }
  });
  menu.addEventListener("keydown", event => {
    const options = items(), index = options.indexOf(document.activeElement);
    if (["ArrowDown", "ArrowUp", "Home", "End"].includes(event.key)) {
      event.preventDefault(); event.stopPropagation();
      const next = event.key === "Home" ? 0 : event.key === "End" ? options.length - 1
        : (index + (event.key === "ArrowDown" ? 1 : options.length - 1)) % options.length;
      options[next]?.focus();
    } else if (event.key === "Escape" || event.key === "Tab") {
      if (event.key === "Escape") event.preventDefault();
      event.stopPropagation(); closeMenu(true);
    }
  });
  document.addEventListener("pointerdown", event => {
    if (!menu.hidden && !menu.contains(event.target) && !trigger.contains(event.target)) closeMenu();
  });
  document.addEventListener("focusin", event => {
    if (!menu.hidden && !menu.contains(event.target) && !trigger.contains(event.target)) closeMenu();
  });
  window.addEventListener("hashchange", () => closeMenu());
  window.addEventListener("resize", () => {if (!menu.hidden) closeMenu(true);});
  document.addEventListener("scroll", event => {
    if (!menu.hidden && !menu.contains(event.target)) closeMenu();
  }, true);
  window.visualViewport?.addEventListener("resize", () => {if (!menu.hidden) positionMenu();});
  window.visualViewport?.addEventListener("scroll", () => {if (!menu.hidden) positionMenu();});
  retry.addEventListener("click", () => {void refresh();});
  const api = Object.freeze({get selectedId(){return selected()?.id ?? null;},get selected(){return selected();},get readiness(){return !!selected();},get toolCalling(){return selected()?.tool_calling ?? null;},refresh,setLocked(value){locked=Boolean(value);syncDisabled();}});
  void refresh(); return api;
})();
