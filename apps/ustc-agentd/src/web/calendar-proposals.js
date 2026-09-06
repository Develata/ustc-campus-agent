// Presentation only: the Calendar owner validates proposals and commits confirmed effects.
window.UcaCalendarProposals = (() => {
  "use strict";
  const mounted = new WeakMap();
  const endpoint = "/api/v1/calendar/proposals";
  const errors = Object.freeze({
    calendar_proposal_conflict: "事项或提案已变化。请刷新核对，再创建新的提案。",
    calendar_proposal_expired: "提案已过期。请核对事项后重新提出。",
    calendar_proposal_not_found: "找不到该提案，请刷新核对。",
    invalid_calendar_proposal: "提案内容无效，请检查标题和北京时间。",
    calendar_proposal_capacity_exceeded: "日历容量已满，请联系管理员检查保留记录。",
    calendar_store_unavailable: "服务暂不可用，执行结果需要核对。"
  });
  const timeFormat = new Intl.DateTimeFormat("zh-CN", {
    timeZone: "Asia/Shanghai", year: "numeric", month: "2-digit", day: "2-digit",
    hour: "2-digit", minute: "2-digit", second: "2-digit", hourCycle: "h23"
  });
  function node(tag, text, className) {
    const element = document.createElement(tag);
    if (text !== undefined) element.textContent = text;
    if (className) element.className = className;
    return element;
  }
  function absoluteTime(value) {
    if (value == null) return "无日期";
    const date = new Date(value);
    return Number.isNaN(date.getTime()) ? "时间不可识别，请刷新核对" : `${timeFormat.format(date)} UTC+08:00`;
  }
  function localInput(value) {
    if (value == null) return "";
    const date = new Date(value);
    if (Number.isNaN(date.getTime())) return "";
    const parts = Object.fromEntries(timeFormat.formatToParts(date).map(part => [part.type, part.value]));
    return `${parts.year}-${parts.month}-${parts.day}T${parts.hour}:${parts.minute}:${parts.second}`;
  }
  function validId(value, kind) {
    if (typeof value !== "string") return false;
    const prefix = `calendar:${kind}:`, digits = value.startsWith(prefix) ? value.slice(prefix.length) : "";
    return /^[1-9][0-9]{0,19}$/.test(digits) && BigInt(digits) <= 18446744073709551615n;
  }
  function validItem(item) {
    return item && validId(item.id, "item") && typeof item.title === "string"
      && (item.scheduled_for === null || typeof item.scheduled_for === "string")
      && Number.isSafeInteger(item.created_at_unix_secs) && item.created_at_unix_secs >= 0;
  }
  function validMutation(mutation) {
    if (!mutation || !["record", "update", "delete"].includes(mutation.action)) return false;
    const keys = mutation.action === "delete" ? ["action", "item_id"]
      : mutation.action === "update" ? ["action", "item_id", "title", "scheduled_for"]
      : ["action", "title", "scheduled_for"];
    return Object.keys(mutation).length === keys.length && keys.every(key => Object.hasOwn(mutation, key))
      && (mutation.action === "record" || validId(mutation.item_id, "item"))
      && (mutation.action === "delete" || (typeof mutation.title === "string"
        && (mutation.scheduled_for === null || typeof mutation.scheduled_for === "string")));
  }
  function sameMutation(a, b) {
    return validMutation(a) && validMutation(b)
      && ["action", "item_id", "title", "scheduled_for"].every(key => a[key] === b[key]);
  }
  function sameItem(a, b) {
    return a === null || b === null ? a === b : validItem(a) && validItem(b)
      && ["id", "title", "scheduled_for", "created_at_unix_secs"].every(key => a[key] === b[key]);
  }
  function sameProposalIdentity(a, b) {
    return a.id === b.id && a.request_id === b.request_id && sameMutation(a.mutation, b.mutation)
      && sameItem(a.before, b.before) && a.expires_at_unix_secs === b.expires_at_unix_secs;
  }
  async function exchange(path, options) {
    const controller = new AbortController();
    const timeout = setTimeout(() => controller.abort(), 15000);
    try {
      const response = await fetch(path, {...options, signal: controller.signal});
      return {response, value: await response.json()};
    } finally { clearTimeout(timeout); }
  }
  function validProposal(proposal) {
    if (!proposal || !validId(proposal.id, "proposal") || typeof proposal.request_id !== "string"
      || !proposal.request_id || !["pending", "applied", "cancelled"].includes(proposal.status)
      || !validMutation(proposal.mutation) || !Number.isSafeInteger(proposal.expires_at_unix_secs)
      || proposal.expires_at_unix_secs < 0) return false;
    const mutation = proposal.mutation;
    if (mutation.action === "record" ? proposal.before !== null
      : !validItem(proposal.before) || proposal.before.id !== mutation.item_id) return false;
    if (proposal.status !== "applied") return proposal.result === null;
    const result = proposal.result;
    if (!validItem(result)) return false;
    if (mutation.action === "delete") return sameItem(result, proposal.before);
    return (mutation.action === "record" || result.id === mutation.item_id)
      && result.title === mutation.title.trim() && result.scheduled_for === mutation.scheduled_for;
  }
  function mount(root) {
    if (!root) return null;
    if (mounted.has(root)) return mounted.get(root);
    let disposed = false, busy = false, reading = false, generation = 0;
    let proposals = [], items = [], now = 0, known = false, signature = "";
    let editing = null, uncertain = null, recoveryChecked = false;
    root.classList.add("calendar-proposals");
    const details = node("details"), summary = node("summary", "日历事项 · 正在读取");
    const content = node("div", undefined, "calendar-proposals-content");
    const note = node("p", "时间均为北京时间 UTC+08:00。当前未启用提醒；确认只保存或修改事项。", "calendar-proposals-note");
    const status = node("p", "", "calendar-proposals-status");
    status.setAttribute("role", "status"); status.setAttribute("aria-live", "polite");
    const controls = node("div", undefined, "calendar-proposals-actions");
    function button(text, action) {
      const element = node("button", text); element.type = "button";
      element.addEventListener("click", action); return element;
    }
    const refreshButton = button("刷新日历", () => refresh(true));
    const retryButton = button("重试原请求", () => { if (uncertain) void command(uncertain); });
    retryButton.hidden = true;
    controls.append(refreshButton, retryButton);
    const manual = node("details", undefined, "calendar-proposals-manual");
    manual.append(node("summary", "手动安排事项"));
    const form = node("form", undefined, "calendar-proposals-form");
    const formTitle = node("h3", "新事项");
    const titleLabel = node("label", "事项标题");
    const title = node("input"); title.type = "text"; title.required = true; title.maxLength = 256;
    title.autocomplete = "off"; title.name = "calendar-title"; titleLabel.append(title);
    const dateLabel = node("label", "绝对日期和时间（北京时间）");
    const date = node("input"); date.type = "datetime-local"; date.step = "1";
    date.name = "calendar-time"; dateLabel.append(date);
    const noDateLabel = node("label", undefined, "calendar-proposals-checkbox");
    const noDate = node("input"); noDate.type = "checkbox"; noDate.checked = true;
    noDateLabel.append(noDate, document.createTextNode("无日期"));
    const formActions = node("div", undefined, "calendar-proposals-actions");
    const proposeButton = node("button", "生成提案"); proposeButton.type = "submit";
    const stopEdit = button("退出编辑", () => resetForm()); stopEdit.hidden = true;
    formActions.append(proposeButton, stopEdit);
    form.append(formTitle, titleLabel, noDateLabel, dateLabel, formActions);
    const pendingTitle = node("h3", "待确认提案");
    const pendingList = node("div", undefined, "calendar-proposals-list");
    const itemsTitle = node("h3", "已保存事项");
    const itemList = node("div", undefined, "calendar-proposals-list");
    manual.append(form);
    content.append(note, controls, status, pendingTitle, pendingList, itemsTitle, itemList, manual);
    details.append(summary, content); root.append(details);
    function announce(message) { status.textContent = message; }
    function updateControls() {
      const locked = busy || Boolean(uncertain) || !known;
      root.setAttribute("aria-busy", String(busy));
      for (const control of form.querySelectorAll("input,button")) control.disabled = locked;
      date.disabled = locked || noDate.checked; date.required = !noDate.checked;
      refreshButton.disabled = busy || reading;
      retryButton.hidden = !uncertain; retryButton.disabled = busy || reading || !recoveryChecked;
      for (const control of root.querySelectorAll("[data-calendar-write]")) control.disabled = locked;
    }
    function resetForm() {
      editing = null; title.value = ""; date.value = ""; noDate.checked = true;
      formTitle.textContent = "新事项"; stopEdit.hidden = true; updateControls();
    }
    noDate.addEventListener("change", updateControls);
    function describe(item) {
      if (!item) return "无";
      return `${item.title || "（无标题）"} · ${absoluteTime(item.scheduled_for)}`;
    }
    function writeButton(text, action) {
      const control = button(text, action); control.dataset.calendarWrite = "true"; return control;
    }
    function render(force = false) {
      const pending = proposals.filter(proposal => proposal.status === "pending");
      summary.textContent = `日历事项 · ${pending.length} 个待确认 · ${items.length} 个已保存`;
      const next = JSON.stringify([proposals, items, pending.map(p => p.expires_at_unix_secs <= now)]);
      if (force || signature !== next) {
        signature = next; pendingList.replaceChildren(); itemList.replaceChildren();
        if (!pending.length) pendingList.append(node("p", "暂无待确认提案。", "calendar-proposals-note"));
        for (const proposal of pending) {
          const card = node("article", undefined, "calendar-proposal-card");
          card.dataset.calendarProposalId = proposal.id;
          const mutation = proposal.mutation;
          card.append(node("h4", ({record: "新增事项", update: "修改事项", delete: "删除事项"})[mutation.action]));
          card.append(node("p", `原事项：${describe(proposal.before)}`));
          card.append(node("p", mutation.action === "delete" ? "确认后：删除此事项" : `确认后：${describe(mutation)}`));
          card.append(node("p", `有效期至：${absoluteTime(proposal.expires_at_unix_secs * 1000)}`, "calendar-proposals-note"));
          const expired = proposal.expires_at_unix_secs <= now;
          if (expired) card.append(node("p", "服务器时间显示提案已过期，请重新提出。", "calendar-proposals-note"));
          const actions = node("div", undefined, "calendar-proposals-actions");
          if (!expired) actions.append(writeButton("确认执行", () => command({kind: "confirm", id: proposal.id})));
          actions.append(writeButton("取消提案", () => command({kind: "cancel", id: proposal.id})));
          card.append(actions); pendingList.append(card);
        }
        if (!items.length) itemList.append(node("p", "暂无已保存事项。", "calendar-proposals-note"));
        for (const item of items) {
          const card = node("article", undefined, "calendar-proposal-card");
          card.dataset.calendarItemId = item.id;
          card.append(node("h4", item.title), node("p", absoluteTime(item.scheduled_for), "calendar-proposals-note"));
          const actions = node("div", undefined, "calendar-proposals-actions");
          actions.append(writeButton("提出修改", () => {
            if (busy || uncertain) return;
            editing = item.id; formTitle.textContent = "修改事项（先生成提案）";
            title.value = item.title; date.value = localInput(item.scheduled_for);
            noDate.checked = item.scheduled_for == null; stopEdit.hidden = false;
            details.open = true; manual.open = true; updateControls(); title.focus();
          }), writeButton("提出删除", () => command({kind: "create", request_id: crypto.randomUUID(), mutation: {action: "delete", item_id: item.id}})));
          card.append(actions); itemList.append(card);
        }
      }
      updateControls();
    }
    function reconcile() {
      if (!uncertain) return;
      const proposal = proposals.find(p => uncertain.kind === "create"
        ? p.request_id === uncertain.request_id : p.id === uncertain.id);
      if (!proposal) return;
      if (uncertain.kind === "create") {
        if (!sameMutation(proposal.mutation, uncertain.mutation)) return;
        resetForm(); details.open = true;
        announce(proposal.status === "pending" ? "已核对：原提案已保存，尚未执行。请核对后确认。"
          : proposal.status === "applied" ? "已核对：原提案已执行。提醒未启用。"
          : "已核对：原提案已取消；不会撤销其他已执行操作。");
      } else {
        if (proposal.status === "pending" || !uncertain.expected
          || !sameProposalIdentity(proposal, uncertain.expected)) return;
        const expected = uncertain.kind === "confirm" ? "applied" : "cancelled";
        announce(proposal.status === expected
          ? (expected === "applied" ? "已核对：原提案已执行。提醒未启用。" : "已核对：原提案已取消；不会撤销其他已执行操作。")
          : "原提案已结束，但状态与这次操作不同，请核对已保存事项。");
      }
      uncertain = null;
    }
    async function refresh(manual = false) {
      if (disposed || reading || busy) return;
      reading = true; const ticket = generation; updateControls();
      try {
        const {response, value} = await exchange(endpoint, {headers: {Accept: "application/json"}, cache: "no-store"});
        if (!response.ok || value.schema !== "calendar-proposals/v1" || value.timezone !== "UTC+08:00"
          || !Number.isFinite(value.now_unix_secs) || !Array.isArray(value.proposals) || !value.proposals.every(validProposal)
          || !Array.isArray(value.items) || !value.items.every(validItem)) throw Error("invalid calendar list");
        if (disposed || ticket !== generation) return;
        if (uncertain) {
          const candidate = value.proposals.find(p => uncertain.kind === "create"
            ? p.request_id === uncertain.request_id : p.id === uncertain.id);
          if (candidate && (uncertain.kind === "create"
            ? !sameMutation(candidate.mutation, uncertain.mutation)
            : !uncertain.expected || !sameProposalIdentity(candidate, uncertain.expected))) throw Error("read-back identity mismatch");
        }
        proposals = value.proposals; items = value.items; now = value.now_unix_secs; known = true;
        if (uncertain) recoveryChecked = true;
        reconcile(); render();
        if (manual && !uncertain) announce("已从服务端刷新日历事项。");
      } catch (_) {
        if (!disposed && ticket === generation) announce(uncertain
          ? "请求结果尚未核实。请再次刷新；仅可重试原请求，不要重新创建。"
          : "暂时无法读取日历，保留当前内容。请重试刷新。");
      } finally {
        reading = false;
        if (!disposed) {
          updateControls();
          if (ticket !== generation && !busy) void refresh();
        }
      }
    }
    async function command(operation) {
      if (disposed || busy || (uncertain && operation !== uncertain)) return;
      if (operation.id && !operation.expected) operation.expected = proposals.find(p => p.id === operation.id);
      busy = true; generation++; updateControls();
      const path = operation.kind === "create" ? endpoint : `${endpoint}/${encodeURIComponent(operation.id)}/${operation.kind}`;
      const payload = operation.kind === "create"
        ? {schema: "calendar-proposal/v1", request_id: operation.request_id, mutation: operation.mutation}
        : {schema: `calendar-proposal-${operation.kind}/v1`};
      try {
        const {response, value} = await exchange(path, {method: "POST", headers: {"Content-Type": "application/json", Accept: "application/json"}, body: JSON.stringify(payload)});
        if (!response.ok) {
          // Only closed client rejections establish a known non-success. 5xx needs read-back.
          const knownRejections = {invalid_calendar_proposal: 400, calendar_proposal_not_found: 404,
            calendar_proposal_conflict: 409, calendar_proposal_expired: 409, calendar_proposal_capacity_exceeded: 429};
          if (value.schema === "calendar-proposal-error/v1" && Object.hasOwn(knownRejections, value.error)
            && response.status === knownRejections[value.error]) {
            uncertain = null; announce(errors[value.error] || "提案未成功，请核对内容和当前日历状态。");
            return;
          }
          throw Error("uncertain command result");
        }
        if (value.schema !== "calendar-proposal-result/v1" || !validProposal(value.proposal)
          || (operation.kind === "create" && (value.proposal.request_id !== operation.request_id
            || !sameMutation(value.proposal.mutation, operation.mutation)))
          || (operation.id && (!operation.expected || !sameProposalIdentity(value.proposal, operation.expected)))
          || (operation.kind === "confirm" && value.proposal.status !== "applied")
          || (operation.kind === "cancel" && value.proposal.status !== "cancelled")) throw Error("invalid command result");
        if (disposed) return;
        uncertain = null; known = true;
        const index = proposals.findIndex(p => p.id === value.proposal.id);
        if (index < 0) proposals.push(value.proposal); else proposals[index] = value.proposal;
        if (value.proposal.status === "applied") {
          const result = value.proposal.result;
          if (value.proposal.mutation.action === "delete") {
            items = items.filter(item => item.id !== value.proposal.mutation.item_id);
          } else if (result) {
            items = items.filter(item => item.id !== result.id).concat(result);
          }
        }
        if (operation.kind === "create") { resetForm(); manual.open = false; details.open = true; }
        announce(value.proposal.status === "pending" ? "提案已保存，尚未执行。请核对后确认。"
          : value.proposal.status === "applied" ? "服务端已确认执行。请核对已保存事项；提醒未启用。"
          : "提案已取消，不会撤销其他已执行操作。");
      } catch (_) {
        if (disposed) return;
        uncertain = operation; recoveryChecked = false;
        announce("网络结果不确定，正在读取服务端状态。保留原请求，只重试原提案或请求 ID。");
      } finally {
        busy = false; generation++; if (!disposed) { render(true); void refresh(); }
      }
    }
    form.addEventListener("submit", event => {
      event.preventDefault();
      if (busy || uncertain || !known || !form.reportValidity()) return;
      const mutation = {action: editing ? "update" : "record", title: title.value,
        scheduled_for: noDate.checked ? null : `${date.value.length === 16 ? date.value + ":00" : date.value}+08:00`};
      if (editing) mutation.item_id = editing;
      void command({kind: "create", request_id: crypto.randomUUID(), mutation});
    });
    const visible = () => document.visibilityState !== "hidden" && root.getClientRects().length > 0;
    const onFocus = () => { if (visible()) void refresh(); };
    const timer = setInterval(onFocus, 5000);
    window.addEventListener("focus", onFocus);
    document.addEventListener("visibilitychange", onFocus);
    details.addEventListener("toggle", () => { if (details.open) void refresh(); });
    const api = Object.freeze({refresh: () => refresh(true), destroy() {
      disposed = true; generation++; clearInterval(timer);
      window.removeEventListener("focus", onFocus); document.removeEventListener("visibilitychange", onFocus);
      root.replaceChildren(); mounted.delete(root);
    }});
    mounted.set(root, api); updateControls(); void refresh(); return api;
  }
  return Object.freeze({mount});
})();
