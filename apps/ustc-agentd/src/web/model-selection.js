// The browser remembers only an opaque server-configured model ID.
window.UcaModelSelection = (() => {
  "use strict";
  const KEY = "uca.selected-model.v1";
  const select = document.querySelector("#chat-model-select");
  const status = document.querySelector("#chat-model-note");
  const retry = document.querySelector("#chat-model-refresh");
  let entries = [], selectedId = null, loaded = false, loading = false, locked = false, storageFailed = false;
  try { selectedId = localStorage.getItem(KEY); } catch (_) { storageFailed = true; }
  const idValid = value => typeof value === "string" && /^[A-Za-z0-9._-]{1,64}$/.test(value);
  const labelValid = value => typeof value === "string" && value.trim().length > 0 && new TextEncoder().encode(value).length <= 128 && !/[\u0000-\u001f\u007f-\u009f]/u.test(value);
  const selected = () => loaded ? entries.find(entry => entry.id === selectedId) ?? null : null;
  function render() {
    select.replaceChildren();
    if (!selected()) {
      const empty = document.createElement("option"); empty.value = "";
      empty.textContent = loading ? "正在读取模型…" : loaded ? "请选择可用模型" : "模型列表未确认";
      select.append(empty);
    }
    for (const entry of entries) {
      const option = document.createElement("option"); option.value = entry.id; option.textContent = entry.label; select.append(option);
    }
    select.value = selected()?.id ?? ""; select.disabled = locked || loading || !loaded;
    retry.disabled = locked || loading;
    retry.hidden = loaded && !!selected();
    const current = selected();
    status.textContent = loading ? "正在读取服务端模型配置…" : !loaded ? "无法确认模型列表，暂不能发送消息。请重试。"
      : !current ? "原模型已不可用或选择未确认，请明确重新选择。"
      : current.tool_calling ? "支持 Agent 工具调用 · 每次执行仍检查权限" : "仅聊天 · 不提供插件工具调用";
    if (current && storageFailed) status.textContent += "。本次选择仅在当前页面保留。";
    status.classList.toggle("visually-hidden", !!current && !storageFailed);
    select.title = current ? `${current.label} · ${status.textContent}` : status.textContent;
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
  select.addEventListener("change", () => {
    if (locked || !loaded || !entries.some(entry=>entry.id===select.value)) {render();return;}
    selectedId = select.value;
    try {localStorage.setItem(KEY,selectedId);storageFailed=false;} catch (_) {storageFailed=true;}
    render();
  });
  retry.addEventListener("click",()=>{void refresh();});
  const api = Object.freeze({get selectedId(){return selected()?.id ?? null;},get selected(){return selected();},get readiness(){return !!selected();},get toolCalling(){return selected()?.tool_calling ?? null;},refresh,setLocked(value){locked=Boolean(value);select.disabled=locked||loading||!loaded;retry.disabled=locked||loading;}});
  void refresh(); return api;
})();