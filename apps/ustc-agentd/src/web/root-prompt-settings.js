// Personal instruction editor. The conversation application owns validation and revisions.
window.UcaRootPromptSettings = (() => {
  "use strict";
  const mounts = new WeakMap();
  const endpoint = "/api/v1/agent/root-prompt";
  const headers = {"X-USTC-Client-Protocol-Major": "1"};
  const bytes = text => new TextEncoder().encode(text).length;
  function element(tag, text, className) {
    const result = document.createElement(tag);
    if (text !== undefined) result.textContent = text;
    if (className) result.className = className;
    return result;
  }
  function validState(value) {
    return value && Object.keys(value).length === 3 && value.schema === "agent-root-prompt/v1"
      && Number.isSafeInteger(value.revision) && value.revision >= 0
      && typeof value.text === "string" && bytes(value.text) <= 8192;
  }
  async function exchange(options = {}) {
    const controller = new AbortController();
    const timeout = setTimeout(() => controller.abort(), 15000);
    try {
      const response = await fetch(endpoint, {cache: "no-store", ...options, signal: controller.signal,
        headers: {...headers, ...options.headers}});
      return {response, value: await response.json()};
    } finally { clearTimeout(timeout); }
  }
  function mount(root) {
    if (!root) return null;
    if (mounts.has(root)) return mounts.get(root);
    let state = null, busy = false, disposed = false, uncertain = null, remote = null;
    let message = "正在读取个人设置…", error = false;
    root.classList.add("root-prompt-settings");
    const heading = element("h2", "Agent 根提示词");
    heading.id = "root-prompt-title";
    const description = element("p", "设定你的长期角色、任务方式和回答习惯。从下一条消息起，应用到你的全部对话。", "root-prompt-description");
    description.id = "root-prompt-description";
    const form = element("form");
    const label = element("label", "个人指令", "visually-hidden");
    label.htmlFor = "root-prompt-text";
    const textarea = element("textarea");
    textarea.id = "root-prompt-text";
    textarea.name = "root-prompt";
    textarea.rows = 7;
    textarea.placeholder = "例如：你是我的校园学习助手。先给出可执行的建议；遇到不确定的校园信息，说明来源和需要核实的部分。";
    textarea.setAttribute("aria-describedby", "root-prompt-description root-prompt-count root-prompt-note");
    const meta = element("div", undefined, "root-prompt-meta");
    const count = element("span"); count.id = "root-prompt-count";
    const dirtyLabel = element("span"); dirtyLabel.id = "root-prompt-dirty";
    meta.append(count, dirtyLabel);
    const actions = element("div", undefined, "root-prompt-actions");
    const save = element("button", "保存", "root-prompt-save"); save.type = "submit"; save.id = "root-prompt-save";
    const reset = element("button", "恢复默认"); reset.type = "button"; reset.id = "root-prompt-reset";
    const reload = element("button", "重新读取"); reload.type = "button"; reload.id = "root-prompt-reload";
    actions.append(save, reset, reload);
    const status = element("p", message, "root-prompt-status");
    status.id = "root-prompt-status"; status.setAttribute("role", "status"); status.setAttribute("aria-live", "polite");
    const conflict = element("div", undefined, "root-prompt-conflict"); conflict.hidden = true;
    const conflictNote = element("p", "服务端内容已变化。你的草稿已保留，请选择以哪一版继续。");
    const preview = element("pre"); preview.id = "root-prompt-remote";
    const conflictActions = element("div", undefined, "root-prompt-actions");
    const adopt = element("button", "采用服务端内容"); adopt.type = "button"; adopt.id = "root-prompt-adopt";
    const keep = element("button", "保留草稿继续编辑"); keep.type = "button"; keep.id = "root-prompt-keep";
    conflictActions.append(adopt, keep); conflict.append(conflictNote, preview, conflictActions);
    const note = element("p", "留空并保存可恢复默认。单次回答偏好独立设置，工具授权与操作确认规则保持不变。", "setting-footnote");
    note.id = "root-prompt-note";
    form.append(label, textarea, meta, actions); root.replaceChildren(heading, description, form, status, conflict, note);
    const dirty = () => state !== null && textarea.value !== state.text;
    function render() {
      if (disposed) return;
      const length = bytes(textarea.value), invalid = length > 8192;
      textarea.disabled = busy || !state || Boolean(uncertain);
      textarea.setAttribute("aria-invalid", String(invalid));
      count.textContent = `${length} / 8192 字节`;
      count.classList.toggle("root-prompt-over-limit", invalid);
      dirtyLabel.textContent = uncertain ? "保存结果待核对" : dirty() ? "未保存" : "";
      save.disabled = busy || !state || invalid || Boolean(uncertain || remote) || !dirty();
      reset.disabled = busy || !state || Boolean(uncertain || remote) || (!state.text && !textarea.value);
      reload.disabled = busy;
      adopt.disabled = keep.disabled = busy || Boolean(uncertain);
      status.textContent = message; status.classList.toggle("root-prompt-error", error);
      conflict.hidden = !remote;
      if (remote) preview.textContent = remote.text || "（默认：未添加个人指令）";
      root.setAttribute("aria-busy", String(busy));
    }
    function accepted(value, text) {
      state = value; textarea.value = value.text; uncertain = remote = null;
      message = text; error = false;
    }
    async function read() {
      if (busy || disposed) return;
      busy = true; render();
      try {
        const {response, value} = await exchange();
        if (!response.ok || !validState(value)) throw Error("invalid read");
        if (disposed) return;
        if (uncertain) {
          const operation = uncertain;
          if (value.revision === operation.expected_revision + 1 && value.text === operation.text.trim()) {
            accepted(value, "已核对服务端：设置已保存，从下一条消息起生效。");
          } else if (value.revision === operation.expected_revision && value.text === state.text) {
            uncertain = null;
            message = "服务端仍是保存前的版本，草稿已保留。可再次点击保存。"; error = true;
          } else {
            uncertain = null; remote = value;
            message = "保存结果无法确认，服务端已有其他版本。草稿已保留，请核对。"; error = true;
          }
        } else if (!state || !dirty()) {
          accepted(value, value.text ? "已读取个人指令。" : "当前使用默认设置。");
        } else if (value.revision !== state.revision || value.text !== state.text) {
          remote = value; message = "服务端内容已变化，未覆盖你的草稿。"; error = true;
        } else {
          message = "已核对服务端，未保存的草稿已保留。"; error = false;
        }
      } catch (_) {
        if (!disposed) { message = uncertain ? "暂时无法核对保存结果，草稿已保留。请重新读取后继续。" : "读取失败，当前草稿未改动。请重新读取。"; error = true; }
      } finally { busy = false; render(); }
    }
    async function update(text) {
      if (busy || disposed || !state || uncertain || remote || bytes(text) > 8192) return;
      // Hold the exact intent until its response or a matching server read resolves it.
      const operation = {schema: "agent-root-prompt-update/v1", expected_revision: state.revision, text};
      textarea.value = text; busy = true; message = "正在保存…"; error = false; render();
      let recover = false;
      try {
        const {response, value} = await exchange({method: "PUT", headers: {"Content-Type": "application/json"}, body: JSON.stringify(operation)});
        if (disposed) return;
        if (response.ok && validState(value) && value.revision === operation.expected_revision + 1 && value.text === operation.text.trim()) {
          accepted(value, value.text ? "已保存，从下一条消息起生效。" : "已恢复默认，从下一条消息起生效。");
        } else if (value?.schema === "chat-conversation-error/v1" && Object.keys(value).length === 2
          && ((response.status === 400 && value.error === "invalid_conversation_intent")
            || (response.status === 409 && value.error === "conversation_revision_conflict")
            || (response.status === 429 && value.error === "conversation_capacity_exceeded"))) {
          error = true;
          message = response.status === 400 ? "未保存：请检查字节限制及不可见控制字符。草稿已保留。"
            : response.status === 429 ? "未保存：服务容量已满。草稿已保留，请联系管理员。"
            : "未保存：服务端版本已变化。正在读取供你核对，草稿已保留。";
          if (response.status === 409) recover = true;
        } else { uncertain = operation; recover = true; }
      } catch (_) { uncertain = operation; recover = true; }
      finally { busy = false; render(); }
      if (recover && !disposed) await read();
    }
    textarea.addEventListener("input", () => {
      message = bytes(textarea.value) > 8192 ? "内容超过 8192 字节，请缩短后保存。" : "修改尚未保存。";
      error = bytes(textarea.value) > 8192; render();
    });
    form.addEventListener("submit", event => { event.preventDefault(); if (!save.disabled) void update(textarea.value); });
    reset.addEventListener("click", () => { if (!reset.disabled) void update(""); });
    reload.addEventListener("click", () => { void read(); });
    adopt.addEventListener("click", () => { if (remote && !adopt.disabled) { accepted(remote, "已采用服务端内容。"); render(); } });
    keep.addEventListener("click", () => {
      if (!remote || keep.disabled) return;
      state = remote; remote = null; message = "已保留草稿。确认内容后点击保存，将更新刚核对的版本。"; error = false; render();
    });
    const focus = () => { if (!dirty() && !uncertain && !remote) void read(); };
    window.addEventListener("focus", focus);
    const api = {refresh: read, destroy() { disposed = true; window.removeEventListener("focus", focus); mounts.delete(root); }};
    mounts.set(root, api); render(); void read(); return api;
  }
  return {mount};
})();
