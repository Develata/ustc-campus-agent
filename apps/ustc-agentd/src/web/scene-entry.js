// Page-local navigation only: none of these controls grants or dispatches a tool.
(() => {
  "use strict";
  const pendingCreate = reusableOperationEnvelope("create", null);
  if (pendingCreate) {
    try {
      window.UcaCourseEditor.setLocked(true, pendingCreate.profile_data ?? OPPORTUNITY_DEMO_TEMPLATE);
      opportunityStatus.textContent = "发现尚未确认的创建请求；显示并锁定原始 synthetic 草稿。请明确同意后重试原请求。";
    } catch (_) {
      window.UcaCourseEditor.setLocked(true);
      opportunityStatus.textContent = "保存的重试草稿无法显示；不会把新草稿拼接到旧请求。请核对本机状态。";
    }
  }
  const scenes = [
    ["affairs", "办理成绩单", "成绩单证明怎么办？"],
    ["radar", "查看校历变化", "校历最近有什么变更？"],
    ["planning", "规划课程", "请根据当前档案规划课程，并解释推荐理由。"],
    ["calendar", "查看我的事项", "列出我的待办事项。"]
  ];
  const section = document.createElement("nav");
  section.id = "scene-entry";
  section.className = "scene-entry";
  section.setAttribute("aria-label", "选择校园任务（仅填入问题，不自动发送）");
  const hint = document.createElement("p");
  hint.id = "scene-hint";
  hint.className = "chat-hint";
  hint.setAttribute("role", "status");
  hint.textContent = "选一个任务填入问题，再由你确认发送。不会自动授权或记事项。";
  for (const [id, label, prompt] of scenes) {
    const button = document.createElement("button");
    button.type = "button";
    button.dataset.scene = id;
    button.textContent = label;
    button.addEventListener("click", () => {
      if (chatPending) {
        hint.textContent = "请等待当前请求结束，再选择下一项任务。";
        return;
      }
      if (chatInput.value.trim() && chatInput.value.trim() !== prompt) {
        // A scene must not silently destroy a composed user message.
        hint.textContent = "输入框已有草稿；请先发送或清除草稿，再选择任务。";
        chatInput.focus();
        return;
      }
      chatInput.value = prompt;
      chatInput.setCustomValidity("");
      chatOpportunityConfirm.checked = false;
      if (id === "planning" && !opportunityProfileId) {
        hint.textContent = "课程问题已填入。先在下方编辑演示档案，明确同意创建；回到聊天后再允许本次使用。";
        const editor = document.querySelector("#course-editor");
        const target = editor ?? opportunityCreate;
        target.scrollIntoView({ block: "center" });
        (editor?.querySelector("input, select, button") ?? opportunityCreate).focus({ preventScroll: true });
      } else {
        hint.textContent = id === "planning"
          ? "将使用已保存的当前档案，而不是未保存草稿。请自行勾选本次使用，再发送。"
          : "问题已填入，请确认后发送。";
        chatInput.focus();
      }
    });
    section.appendChild(button);
  }
  chatSurface.before(section, hint);

  function addAnswerActions(item) {
    if (item.dataset.role !== "assistant" || item.querySelector(".answer-actions")) return;
    const actions = document.createElement("div");
    actions.className = "answer-actions";
    const copied = document.createElement("span");
    copied.setAttribute("role", "status");
    const copy = document.createElement("button");
    copy.type = "button";
    copy.textContent = "复制回答";
    copy.addEventListener("click", async () => {
      try {
        if (!navigator.clipboard?.writeText) throw new Error("clipboard unavailable");
        await navigator.clipboard.writeText(item.querySelector(".chat-message-body").textContent);
        copied.textContent = "已复制回答；工具状态与来源仍请在页面核对。";
      } catch (_) {
        copied.textContent = "无法自动复制，请选中上方回答手动复制。";
      }
    });
    actions.append(copy);
    const tools = [...item.querySelectorAll(".chat-tool-trace li span:first-child")].map(el => el.textContent);
    if (tools.includes(CHAT_TOOL_LABELS.affairs_navigator_get)) {
      const source = document.createElement("a");
      source.href = "#hero-title";
      source.textContent = "核对办事来源与清单";
      actions.append(source);
    }
    if (tools.includes(CHAT_TOOL_LABELS.opportunity_graph_plan_current_profile)) {
      const plan = document.createElement("a");
      plan.href = "#opportunity-plan";
      plan.textContent = "查看档案与生成详细方案";
      actions.append(plan);
    }
    actions.append(copied);
    item.append(actions);
  }
  new MutationObserver(records => {
    for (const record of records) {
      for (const node of record.addedNodes) {
        if (node.nodeType === 1 && node.matches(".chat-message")) addAnswerActions(node);
      }
    }
  }).observe(chatMessages, { childList: true });
})();
