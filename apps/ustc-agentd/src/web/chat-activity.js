// Read-only projection of admitted server activity; never a source of execution authority.
window.UcaChatActivity = (() => {
  "use strict";
  const tools = Object.freeze({
    affairs_navigator_get: "查询办理流程",
    change_radar_get: "查看校历变化",
    opportunity_graph_plan_current_profile: "规划课程",
    simple_calendar_items: "处理日历事项",
    plugin_tool: "使用插件能力"
  });
  const states = Object.freeze({ running: "进行中", succeeded: "已完成", denied: "未获准", failed: "未完成" });
  const phases = ["idle", "running", "completed", "failed", "interrupted"];
  const bytes = value => typeof value === "string" ? new TextEncoder().encode(value).length : Infinity;
  const identifier = value => bytes(value) > 0 && bytes(value) <= 256;
  const closed = (value, keys) => value && typeof value === "object" && !Array.isArray(value) &&
    Object.keys(value).length === keys.length && keys.every(key => Object.hasOwn(value, key));
  function validate(value) {
    if (!closed(value, ["schema", "conversation_id", "request_id", "phase", "sequence", "steps"]) ||
        value.schema !== "chat-conversation-activity/v1" || !identifier(value.conversation_id) ||
        !(value.request_id === null || identifier(value.request_id)) || !phases.includes(value.phase) ||
        !Number.isSafeInteger(value.sequence) || value.sequence < 0 || value.sequence > 15 ||
        (value.phase === "idle" && value.sequence !== 0) ||
        (value.phase === "running" && value.sequence > 14) ||
        (["completed", "failed", "interrupted"].includes(value.phase) && value.sequence !== 15) || !Array.isArray(value.steps) || value.steps.length > 7 ||
        (value.phase === "idle" && (value.request_id !== null || value.steps.length)) ||
        (value.phase !== "idle" && value.request_id === null)) throw Error("invalid_activity");
    const ids = new Set();
    for (const step of value.steps) {
      if (!closed(step, ["id", "kind", "tool", "status"]) || !identifier(step.id) || ids.has(step.id) ||
          !Object.hasOwn(states, step.status) ||
          !(step.kind === "model" ? step.tool === null && step.status !== "denied" :
            step.kind === "tool" && Object.hasOwn(tools, step.tool)) ||
          (["completed", "failed", "interrupted"].includes(value.phase) && step.status === "running")) throw Error("invalid_activity");
      ids.add(step.id);
    }
    if (value.steps.filter(step => step.kind === "model").length > 3 || value.steps.filter(step => step.kind === "tool").length > 4) throw Error("invalid_activity");
    return value;
  }
  function advances(previous, next) {
    if (!previous) return true;
    if (next.sequence <= previous.sequence) return false;
    // Terminal state is rebuilt from the saved tool trace; transient model steps disappear.
    if (["completed", "failed", "interrupted"].includes(next.phase)) return true;
    if (next.steps.length < previous.steps.length) return false;
    return previous.steps.every((step, index) => {
      const candidate = next.steps[index];
      return step.id === candidate.id && step.kind === candidate.kind && step.tool === candidate.tool &&
        (step.status === "running" || step.status === candidate.status);
    });
  }
  function mount(root) {
    let generation = 0, timer = null, controller = null, latest = null;
    const status = document.createElement("p"); status.className = "chat-activity-status";
    status.setAttribute("role", "status"); status.setAttribute("aria-live", "polite"); status.setAttribute("aria-atomic", "true");
    const details = document.createElement("details"); details.className = "chat-activity-details";
    const summary = document.createElement("summary"); summary.textContent = "查看执行步骤";
    const list = document.createElement("ol"); details.append(summary, list);
    root.replaceChildren(status, details); root.hidden = true;
    function cancel() { generation++; clearTimeout(timer); timer = null; controller?.abort(); controller = null; }
    function clear() { cancel(); latest = null; root.hidden = true; root.removeAttribute("data-state"); list.replaceChildren(); }
    function unavailable() {
      root.hidden = false; root.dataset.state = "unknown";
      status.textContent = "暂时无法读取执行状态；最终结果仍以服务器回复为准。";
    }
    function stop(outcome = "unknown") {
      cancel();
      if (outcome === "settled") { clear(); return; }
      root.hidden = false; root.dataset.state = "unknown";
      status.textContent = "尚未确认最终结果，请使用下方的“检查结果”。";
    }
    function render(value) {
      const scroll = root.parentElement;
      const follow = scroll && scroll.scrollHeight - scroll.scrollTop - scroll.clientHeight < 80;
      root.hidden = false; root.dataset.state = value.phase;
      const active = value.steps.findLast(step => step.status === "running");
      const phaseLabels = { idle: "请求已发送，等待状态", running: "服务器正在处理请求", completed: "执行已完成，等待回答记录", failed: "本次执行未完成，等待结果记录", interrupted: "执行已中断，请检查结果" };
      status.textContent = value.phase === "running" && active ?
        (active.kind === "model" ? "正在处理模型请求" : tools[active.tool]) : phaseLabels[value.phase];
      details.hidden = !value.steps.length;
      summary.textContent = `执行步骤 · ${value.steps.length}`;
      list.replaceChildren();
      for (const step of value.steps) {
        const item = document.createElement("li"); item.dataset.status = step.status;
        const label = document.createElement("span"); label.textContent = step.kind === "model" ? "模型请求" : tools[step.tool];
        const state = document.createElement("span"); state.className = "chat-activity-step-status"; state.textContent = states[step.status];
        item.append(label, state); list.append(item);
      }
      if (follow) scroll.scrollTop = scroll.scrollHeight;
    }
    function start(conversationId, requestId) {
      clear();
      if (!identifier(conversationId) || !identifier(requestId)) return;
      const ticket = generation;
      root.hidden = false; root.dataset.state = "waiting"; details.hidden = true; details.open = false;
      status.textContent = "请求已发送，等待状态";
      async function poll() {
        if (ticket !== generation) return;
        const abort = new AbortController(); controller = abort; let timeout;
        try {
          const value = await Promise.race([
            (async () => {
              const response = await fetch(`/api/v1/agent/conversations/${encodeURIComponent(conversationId)}/activity`, {
                headers: { Accept: "application/json", "X-USTC-Client-Protocol-Major": "1" }, cache: "no-store", signal: abort.signal
              });
              if (!response.ok) throw Error("activity_unavailable");
              const body = await response.text();
              if (bytes(body) > 8192) throw Error("invalid_activity");
              return validate(JSON.parse(body));
            })(),
            new Promise((_, reject) => { timeout = setTimeout(() => { abort.abort(); reject(Error("activity_timeout")); }, 5000); })
          ]);
          if (ticket !== generation) return;
          if (value.conversation_id !== conversationId) throw Error("activity_mismatch");
          // GET may precede admission of this POST. Never show another request's activity.
          if (value.request_id === requestId && latest && value.sequence === latest.sequence &&
              JSON.stringify(value) === JSON.stringify(latest)) render(latest);
          if (value.request_id === requestId && advances(latest, value)) {
            latest = value; render(value);
            if (["completed", "failed", "interrupted"].includes(value.phase)) return;
          }
        } catch (_) { if (ticket === generation) unavailable(); }
        finally { clearTimeout(timeout); if (controller === abort) controller = null; }
        if (ticket === generation) timer = setTimeout(poll, document.hidden ? 3000 : 800);
      }
      void poll();
    }
    return Object.freeze({ start, stop, clear });
  }
  return Object.freeze({ mount });
})();
