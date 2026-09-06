const chatForm = document.querySelector("#chat-form");
const chatInput = document.querySelector("#chat-input");
const chatClear = document.querySelector("#chat-clear");
const chatSend = document.querySelector("#chat-send");
const chatSurface = document.querySelector("#chat-surface");
const chatMessages = document.querySelector("#chat-messages");
const chatEmpty = document.querySelector("#chat-empty");
const chatProgress = document.querySelector("#chat-progress");
const chatError = document.querySelector("#chat-error");
const chatErrorMessage = document.querySelector("#chat-error-message");
const chatErrorCode = document.querySelector("#chat-error-code");
const chatOpportunityConfirm = document.querySelector("#chat-opportunity-confirm");
const chatOpportunityState = document.querySelector("#chat-opportunity-state");
const chatPromptCustomization = document.querySelector("#chat-prompt-customization");
const chatPromptCustomizationCounter = document.querySelector("#chat-prompt-customization-counter");
const CHAT_REQUEST_SCHEMA_V3 = "ustc-agent-chat-request/v3";

const CHAT_RESPONSE_SCHEMA = "ustc-agent-chat-response/v1";
const CHAT_ERROR_SCHEMA = "ustc-agent-chat-error/v1";
const CHAT_MAX_MESSAGES = 12;
const CHAT_MAX_MESSAGE_BYTES = 4 * 1024;
const CHAT_MAX_PROFILE_SNAPSHOT_ID_BYTES = 4 * 1024;
const CHAT_MAX_HISTORY_BYTES = 12 * 1024;
const CHAT_MAX_BODY_BYTES = 16 * 1024;
const CHAT_MAX_ANSWER_BYTES = 16 * 1024;
const CHAT_MAX_TOOL_CALLS = 4;
const CHAT_MAX_PROMPT_CUSTOMIZATION_BYTES = 2048;
const CHAT_PROMPT_DISALLOWED_SCALAR_PATTERN = /[\p{Cc}\p{Cf}]/u;
const chatTextEncoder = new TextEncoder();
const chatHistory = [];
let chatPending = false;
let conversationClient = null;
let conversationCanSend = false;
let conversationCanSwitch = true;
function syncModelAvailability() {
  const ready = window.UcaModelSelection?.readiness === true;
  chatSend.disabled = chatPending || !conversationCanSend || !ready;
  chatSend.setAttribute("aria-disabled", String(chatSend.disabled));
  window.UcaModelSelection?.setLocked(chatPending || !conversationCanSwitch);
}
let restoredConversationDraft = null;

const form = document.querySelector("#lookup-form");
const procedureInput = document.querySelector("#procedure-id");
const procedurePreview = document.querySelector("#procedure-id-preview");
const submitButton = document.querySelector("#lookup-button");
const status = document.querySelector("#status");
const result = document.querySelector("#result");
const errorPanel = document.querySelector("#error-panel");
const errorMessage = document.querySelector("#error-message");
const radarButton = document.querySelector("#radar-load");
const radarStatus = document.querySelector("#radar-status");
const radarResult = document.querySelector("#radar-result");
const opportunityConsent = document.querySelector("#opportunity-consent");
const opportunityCreate = document.querySelector("#opportunity-create");
const opportunityView = document.querySelector("#opportunity-view");
const opportunityPlan = document.querySelector("#opportunity-plan");
const opportunityDelete = document.querySelector("#opportunity-delete");
const opportunityStatus = document.querySelector("#opportunity-status");
const opportunityProfile = document.querySelector("#opportunity-profile");
const opportunityPlanResult = document.querySelector("#opportunity-plan-result");
const opportunityDeleted = document.querySelector("#opportunity-deleted");
const OPPORTUNITY_PROFILE_HINT = "ustc-campus-agent/opportunity-profile-id/v1";
const OPPORTUNITY_PENDING_OPERATIONS = Object.freeze({
  create: "ustc-campus-agent/opportunity-pending-create/v1",
  delete: "ustc-campus-agent/opportunity-pending-delete/v1"
});
const opportunityPendingMemory = new Map();
const OPPORTUNITY_DEMO_TEMPLATE = {
  completed_courses: ["MATH1001", "MATH1002", "CS1001", "PHYS1001"],
  min_credits: 9,
  max_credits: 12,
  preference_weights: [
    { course_code: "MATH2001", weight: 9 },
    { course_code: "MATH2003", weight: 8 },
    { course_code: "CS2006", weight: 7 },
    { course_code: "PHYS2003", weight: 5 },
    { course_code: "HUM2001", weight: 4 },
    { course_code: "GEN2001", weight: 3 },
    { course_code: "LANG2001", weight: 2 }
  ]
};
let opportunityProfileId = null;
let opportunityBusy = false;

function setOpportunityBusy(busy) {
  opportunityBusy = busy;
  opportunityCreate.disabled = busy;
  opportunityView.disabled = busy || !opportunityProfileId;
  opportunityPlan.disabled = busy || !opportunityProfileId;
  opportunityDelete.disabled = busy || !opportunityProfileId;
}

function clear(element) {
  while (element.firstChild) {
    element.removeChild(element.firstChild);
  }
}

function text(element, value) {
  element.textContent = value ?? "—";
}

const CHAT_TOOL_LABELS = Object.freeze({
  affairs_navigator_get: "办事导航 · 查询公开流程",
  change_radar_get: "变更雷达 · 查询校历变更",
  simple_calendar_items: "简单日历 · 记录与查看事项",
  plugin_tool: "插件能力",
  opportunity_graph_plan_current_profile: "机会图谱 · 规划当前档案"
});

const CHAT_STATUS_LABELS = Object.freeze({
  succeeded: "已完成",
  denied: "已拒绝",
  failed: "未完成"
});

const CHAT_ERROR_MESSAGES = Object.freeze({
  invalid_chat_request: "消息未通过有界请求校验。请缩短内容并重试。",
  invalid_prompt_customization: "响应偏好须为 1–2048 UTF-8 bytes，且不能包含控制、双向覆盖、零宽或 BOM 字符。",
  provider_not_configured: "当前 Agent 服务尚未就绪。请联系本机演示的管理员。",
  provider_unauthorized: "当前 Agent 服务无法完成认证。请联系本机演示的管理员。",
  provider_rate_limited: "请求过于频繁。请稍后再试。",
  provider_timeout: "等待回答超时；服务器没有返回完成结果。请稍后重试。",
  provider_unavailable: "回答服务暂时不可用。请稍后重试。",
  provider_protocol_error: "回答服务返回了无法安全读取的结果。请稍后重试。",
  context_budget_exceeded: "这段对话超过当前模型的安全上下文预算。请新建对话或缩短问题后重试。",
  tool_call_rejected: "校园工具拒绝了这次调用。请换一种更具体的问法。",
  tool_result_too_large: "校园工具结果超过本次对话上限。请缩小问题范围。",
  tool_budget_exhausted: "这次问题需要的工具调用超过上限。请拆成更小的问题。",
  turn_budget_exhausted: "Agent 未能在有限轮次内形成最终回答。请缩小问题范围。",
  opportunity_confirmation_required: "Opportunity 请求缺少本次明确允许。请确认已有档案并重新勾选。",
  composition_unavailable: "校园工具组合暂时不可用。请稍后重试，或使用下方详细面板。",
  internal_chat_error: "服务器未能完成这次请求。请稍后重试。",
  message_too_large: "这条消息编码后超过 4 KiB。请缩短后重试。",
  invalid_response: "服务器返回了无法安全呈现的回答。请稍后重试。",
  conversation_unavailable: "暂时无法读取保存的对话。请刷新侧栏历史对话后继续。",
  conversation_in_progress: "这次请求尚未确定完成。请先检查结果，不要重复发送。",
  conversation_revision_conflict: "这段对话已在其他页面更新。已读取最新记录，你输入的草稿仍保留，请核对后再发送。",
  conversation_request_conflict: "原请求与已保存记录不一致。请检查结果，核对已执行内容。",
  conversation_capacity_exceeded: "对话服务已达到当前容量上限，本次请求未被接纳。请保留草稿，检查已保存状态后再继续。",
  conversation_limit_reached: "保存的对话已达到当前上限。请继续已有对话。",
  conversation_turn_limit_reached: "这段对话已达到当前上限，请新建对话继续。",
  request_failed: "服务器拒绝了这次请求，但没有返回可识别的恢复信息。",
  network_error: "无法连接到本机 Agent 服务。请确认演示仍在运行后重试。"
});

function chatFailure(code) {
  const error = new Error(code);
  error.code = code;
  return error;
}

function utf8Length(value) {
  return chatTextEncoder.encode(value).byteLength;
}

function syncPromptCustomizationCounter() {
  const byteLength = utf8Length(chatPromptCustomization.value);
  chatPromptCustomizationCounter.textContent = `${byteLength} / ${CHAT_MAX_PROMPT_CUSTOMIZATION_BYTES} UTF-8 bytes`;
  chatPromptCustomizationCounter.dataset.overLimit = String(
    byteLength > CHAT_MAX_PROMPT_CUSTOMIZATION_BYTES
  );
}

function promptCustomizationForRequest() {
  const raw = chatPromptCustomization.value;
  const trimmed = raw.trim();
  if (!trimmed) {
    return null;
  }
  const hasDisallowedScalar = [...raw].some((scalar) =>
    !["\t", "\n", "\r"].includes(scalar)
      && CHAT_PROMPT_DISALLOWED_SCALAR_PATTERN.test(scalar)
  );
  if (utf8Length(raw) > CHAT_MAX_PROMPT_CUSTOMIZATION_BYTES || hasDisallowedScalar) {
    throw chatFailure("invalid_prompt_customization");
  }
  return trimmed;
}

function boundedChatMessages(userContent, opportunityProfileHint) {
  const retainedHistory = chatHistory.filter(
    (message) => message.opportunityProfileHint == null
      || message.opportunityProfileHint === opportunityProfileHint
  );
  const current = { role: "user", content: userContent };
  const selected = [current];
  let totalBytes = utf8Length(current.content);
  for (let index = retainedHistory.length - 2; index >= 0; index -= 2) {
    const userMessage = retainedHistory[index];
    const assistantMessage = retainedHistory[index + 1];
    if (userMessage?.role !== "user" || assistantMessage?.role !== "assistant") {
      throw chatFailure("invalid_chat_request");
    }
    const pairBytes = utf8Length(userMessage.content) + utf8Length(assistantMessage.content);
    if (
      selected.length + 2 > CHAT_MAX_MESSAGES ||
      totalBytes + pairBytes > CHAT_MAX_HISTORY_BYTES
    ) {
      break;
    }
    selected.unshift(
      { role: userMessage.role, content: userMessage.content },
      { role: assistantMessage.role, content: assistantMessage.content }
    );
    totalBytes += pairBytes;
  }
  return selected;
}

function createChatRequest(userContent, opportunityProfileHint, promptCustomization = null) {
  const messages = boundedChatMessages(userContent, opportunityProfileHint);
  const opportunityContext = opportunityProfileHint == null
    ? null
    : { profile_snapshot_id: opportunityProfileHint };
  const makeRequest = () => {
    const request = {
      schema: CHAT_REQUEST_SCHEMA_V3,
      model_id: window.UcaModelSelection?.selectedId,
      messages,
      opportunity_context: opportunityContext
    };
    if (promptCustomization != null) {
      request.prompt_customization = { text: promptCustomization };
    }
    return request;
  };
  let request = makeRequest();
  let body = JSON.stringify(request);

  while (utf8Length(body) > CHAT_MAX_BODY_BYTES && messages.length > 1) {
    messages.splice(0, 2);
    request = makeRequest();
    body = JSON.stringify(request);
  }
  if (utf8Length(body) > CHAT_MAX_BODY_BYTES) {
    throw chatFailure("message_too_large");
  }
  return { request, body };
}

function assertChatRequestContract(request, headers, body) {
  const requestKeys = Object.keys(request).sort().join(",");
  const hasPromptCustomization = Object.prototype.hasOwnProperty.call(
    request,
    "prompt_customization"
  );
  const expectedKeys = hasPromptCustomization
    ? "messages,model_id,opportunity_context,prompt_customization,schema"
    : "messages,model_id,opportunity_context,schema";
  if (requestKeys !== expectedKeys) {
    throw chatFailure("invalid_chat_request");
  }
  if (
    request.schema !== CHAT_REQUEST_SCHEMA_V3 || typeof request.model_id !== "string" || !/^[A-Za-z0-9._-]{1,64}$/.test(request.model_id) ||
    !Array.isArray(request.messages) ||
    request.messages.length < 1 ||
    request.messages.length > CHAT_MAX_MESSAGES ||
    request.messages.at(-1)?.role !== "user"
  ) {
    throw chatFailure("invalid_chat_request");
  }

  if (hasPromptCustomization) {
    const customization = request.prompt_customization;
    if (
      typeof customization !== "object" ||
      customization == null ||
      Array.isArray(customization) ||
      Object.keys(customization).join(",") !== "text" ||
      typeof customization.text !== "string" ||
      customization.text.trim() !== customization.text ||
      customization.text.length === 0 ||
      utf8Length(customization.text) > CHAT_MAX_PROMPT_CUSTOMIZATION_BYTES ||
      [...customization.text].some((scalar) =>
        !["\t", "\n", "\r"].includes(scalar)
          && CHAT_PROMPT_DISALLOWED_SCALAR_PATTERN.test(scalar)
      )
    ) {
      throw chatFailure("invalid_chat_request");
    }
  }

  let totalBytes = 0;
  for (const message of request.messages) {
    if (
      Object.keys(message).sort().join(",") !== "content,role" ||
      (message.role !== "user" && message.role !== "assistant") ||
      typeof message.content !== "string" ||
      message.content.trim().length === 0 ||
      message.content.includes("\u0000")
    ) {
      throw chatFailure("invalid_chat_request");
    }
    const contentBytes = utf8Length(message.content);
    if (contentBytes > CHAT_MAX_MESSAGE_BYTES) {
      throw chatFailure("invalid_chat_request");
    }
    totalBytes += contentBytes;
  }
  if (totalBytes > CHAT_MAX_HISTORY_BYTES || utf8Length(body) > CHAT_MAX_BODY_BYTES) {
    throw chatFailure("invalid_chat_request");
  }

  const hasConfirmationHeader = Object.prototype.hasOwnProperty.call(
    headers,
    "X-USTC-Opportunity-Confirmation"
  );
  if (request.opportunity_context == null) {
    if (hasConfirmationHeader) {
      throw chatFailure("invalid_chat_request");
    }
    return;
  }
  if (
    typeof request.opportunity_context !== "object" ||
    Array.isArray(request.opportunity_context) ||
    Object.keys(request.opportunity_context).sort().join(",") !== "profile_snapshot_id" ||
    typeof request.opportunity_context.profile_snapshot_id !== "string" ||
    request.opportunity_context.profile_snapshot_id.trim().length === 0 ||
    headers["X-USTC-Opportunity-Confirmation"] !== "confirmed"
  ) {
    throw chatFailure("invalid_chat_request");
  }
}

function assertChatDomContract() {
  const failures = [];
  if (chatForm?.tagName !== "FORM") failures.push("chat-form");
  if (chatInput?.tagName !== "TEXTAREA" || chatInput.maxLength !== 4096) failures.push("chat-input");
  if (chatPromptCustomization?.tagName !== "TEXTAREA") failures.push("chat-prompt-customization");
  if (chatPromptCustomizationCounter?.tagName !== "OUTPUT") failures.push("chat-prompt-customization-counter");
  if (chatClear?.tagName !== "BUTTON" || chatClear.type !== "button") failures.push("chat-clear");
  if (chatSend?.tagName !== "BUTTON" || chatSend.type !== "submit") failures.push("chat-send");
  if (chatOpportunityConfirm?.type !== "checkbox") failures.push("chat-opportunity-confirm");
  if (chatMessages?.getAttribute("aria-live") !== "polite") failures.push("chat-messages-live");
  if (chatError?.getAttribute("role") !== "alert") failures.push("chat-error-alert");
  if (failures.length > 0) {
    throw new Error(`Chat DOM invariant failed: ${failures.join(", ")}`);
  }
}

function syncChatEmptyState() {
  chatEmpty.hidden = chatMessages.querySelector(".chat-message") !== null;
  syncModelAvailability();
  window.dispatchEvent(new Event("uca:chat-state"));
}

function clearChatConversation() {
  if (chatPending) {
    return;
  }
  chatHistory.splice(0, chatHistory.length);
  for (const item of chatMessages.querySelectorAll(".chat-message")) {
    item.remove();
  }
  chatError.hidden = true;
  chatInput.value = "";
  chatOpportunityConfirm.checked = false;
  chatInput.setCustomValidity("");
  chatPromptCustomization.value = "";
  chatPromptCustomization.setCustomValidity("");
  syncPromptCustomizationCounter();
  syncChatEmptyState();
  chatInput.focus({ preventScroll: true });
}

function trimChatTranscript() {
  const items = Array.from(chatMessages.querySelectorAll(".chat-message"));
  for (const item of items.slice(0, Math.max(0, items.length - CHAT_MAX_MESSAGES))) {
    item.remove();
  }
  syncChatEmptyState();
}

function renderChatToolTrace(messageItem, toolTrace) {
  if (toolTrace.length === 0) {
    return;
  }
  const details = document.createElement("details");
  details.className = "chat-tool-trace";
  const summary = document.createElement("summary");
  summary.textContent = `工具记录 · ${toolTrace.length} 次`;
  const list = document.createElement("ul");
  for (const trace of toolTrace) {
    const item = document.createElement("li");
    const tool = document.createElement("span");
    tool.textContent = CHAT_TOOL_LABELS[trace.tool];
    const state = document.createElement("span");
    state.className = "chat-tool-state";
    state.dataset.status = trace.status;
    state.textContent = CHAT_STATUS_LABELS[trace.status];
    item.append(tool, state);
    list.appendChild(item);
  }
  details.append(summary, list);
  messageItem.appendChild(details);
}

function appendChatMessage(role, content, toolTrace = []) {
  const item = document.createElement("li");
  item.className = "chat-message";
  item.dataset.role = role;
  if (role === "assistant") item.ucaAnswerText = content;
  item.setAttribute("aria-label", role === "user" ? "你的消息" : "校园 Agent 的回答");
  const speaker = document.createElement("span");
  speaker.className = "chat-speaker";
  speaker.textContent = role === "user" ? "你" : "校园 Agent";
  const body = document.createElement("div");
  body.className = "chat-message-body";
  if (role === "assistant") window.UcaChatMarkdown.render(body, content);
  else body.textContent = content;
  item.append(speaker, body);
  if (role === "assistant") {
    renderChatToolTrace(item, toolTrace);
  }
  chatMessages.appendChild(item);
  syncChatEmptyState();
  return item;
}

function commitChatTurn(userContent, assistantContent, opportunityProfileHint) {
  if (
    utf8Length(userContent) > CHAT_MAX_MESSAGE_BYTES ||
    utf8Length(assistantContent) > CHAT_MAX_MESSAGE_BYTES
  ) {
    // A later follow-up must not skip this visible turn and reconnect to an older topic.
    chatHistory.splice(0, chatHistory.length);
    return false;
  }
  chatHistory.push(
    {
      role: "user",
      content: userContent,
      opportunityProfileHint
    },
    {
      role: "assistant",
      content: assistantContent,
      opportunityProfileHint
    }
  );
  while (chatHistory.length > CHAT_MAX_MESSAGES) {
    chatHistory.splice(0, 2);
  }
  return true;
}

function validateChatResponse(payload) {
  if (
    payload?.schema !== CHAT_RESPONSE_SCHEMA ||
    typeof payload.run_id !== "string" ||
    payload.run_id.trim().length === 0 ||
    typeof payload.answer !== "string" ||
    payload.answer.trim().length === 0 ||
    utf8Length(payload.answer) > CHAT_MAX_ANSWER_BYTES ||
    !Array.isArray(payload.tool_trace) ||
    payload.tool_trace.length > CHAT_MAX_TOOL_CALLS
  ) {
    throw chatFailure("invalid_response");
  }
  const callIds = new Set();
  for (const trace of payload.tool_trace) {
    if (
      typeof trace?.call_id !== "string" ||
      trace.call_id.trim().length === 0 ||
      callIds.has(trace.call_id) ||
      !Object.prototype.hasOwnProperty.call(CHAT_TOOL_LABELS, trace.tool) ||
      !Object.prototype.hasOwnProperty.call(CHAT_STATUS_LABELS, trace.status)
    ) {
      throw chatFailure("invalid_response");
    }
    callIds.add(trace.call_id);
  }
  return {
    answer: payload.answer.trim(),
    toolTrace: payload.tool_trace.map((trace) => ({
      tool: trace.tool,
      status: trace.status
    }))
  };
}

function normalizeChatErrorCode(value, fallback) {
  return typeof value === "string" && /^[a-z0-9_]{1,64}$/.test(value)
    ? value
    : fallback;
}

function calendarMutationRecoveryHint(userContent) {
  // Presentation guidance only; Rust owns admission and mutation authority.
  return /^(?:记录事项[：:]|删除事项(?:\s|$))/u.test(userContent)
    ? "事项可能已经变更；请先发送“列出我的待办事项”核对，再决定是否重试。"
    : "";
}

function appendChatNotice(item, className, message) {
  const notice = document.createElement("p");
  notice.className = `chat-hint ${className}`;
  notice.setAttribute("role", "status");
  notice.textContent = message;
  item.appendChild(notice);
}

function showChatError(code, userContent = "") {
  const safeCode = normalizeChatErrorCode(code, "request_failed");
  chatErrorMessage.textContent = calendarMutationRecoveryHint(userContent)
    || CHAT_ERROR_MESSAGES[safeCode] || CHAT_ERROR_MESSAGES.request_failed;
  chatErrorCode.textContent = safeCode;
  chatError.hidden = false;
}

function setChatBusy(busy) {
  chatPending = busy;
  chatSurface.setAttribute("aria-busy", String(busy));
  chatProgress.hidden = !busy;
  chatClear.disabled = busy;
  chatClear.setAttribute("aria-disabled", String(busy));
  chatSend.disabled = busy;
  chatSend.setAttribute("aria-disabled", String(busy));
  chatSend.setAttribute("aria-label", busy ? "正在发送" : "发送消息");
  chatSend.dataset.busy = String(busy);
  chatPromptCustomization.disabled = busy;
  chatPromptCustomization.setAttribute("aria-disabled", String(busy));
  chatOpportunityConfirm.disabled = busy || !opportunityProfileId || window.UcaModelSelection?.toolCalling === false;
  chatOpportunityConfirm.setAttribute(
    "aria-disabled",
    String(busy || !opportunityProfileId || window.UcaModelSelection?.toolCalling === false)
  );
  syncModelAvailability();
  window.dispatchEvent(new Event("uca:chat-state"));
}

async function requestChat(body, headers) {
  let response;
  try {
    response = await fetch("/api/v1/agent/chat", {
      method: "POST",
      headers,
      body,
      cache: "no-store"
    });
  } catch (_error) {
    throw chatFailure("network_error");
  }

  let payload;
  try {
    payload = await response.json();
  } catch (_error) {
    throw chatFailure("invalid_response");
  }
  if (!response.ok) {
    const code = payload?.schema === CHAT_ERROR_SCHEMA
      ? normalizeChatErrorCode(payload.error, "request_failed")
      : "request_failed";
    throw chatFailure(code);
  }
  const validated = validateChatResponse(payload);
  window.dispatchEvent(new CustomEvent("uca:provider-response", {detail: payload.provider}));
  return validated;
}

async function submitChat() {
  if (!conversationCanSend) return;
  if (!window.UcaModelSelection?.readiness) { document.querySelector("#chat-model-select").focus(); return; }
  if (chatPending) {
    return;
  }
  chatInput.setCustomValidity("");
  chatPromptCustomization.setCustomValidity("");
  const userContent = chatInput.value.trim();
  if (!userContent) {
    chatInput.setCustomValidity("请输入一条消息。");
    chatInput.reportValidity();
    return;
  }
  if (utf8Length(userContent) > CHAT_MAX_MESSAGE_BYTES) {
    chatInput.setCustomValidity(CHAT_ERROR_MESSAGES.message_too_large);
    chatInput.reportValidity();
    return;
  }

  let promptCustomization;
  try {
    promptCustomization = promptCustomizationForRequest();
  } catch (_error) {
    chatPromptCustomization.setCustomValidity(
      CHAT_ERROR_MESSAGES.invalid_prompt_customization
    );
    chatPromptCustomization.reportValidity();
    return;
  }

  const useOpportunity = Boolean(opportunityProfileId && chatOpportunityConfirm.checked
    && window.UcaModelSelection?.toolCalling !== false);
  const opportunityProfileHint = useOpportunity ? opportunityProfileId : null;
  const headers = {
    "Accept": "application/json",
    "Content-Type": "application/json"
  };
  if (useOpportunity) {
    headers["X-USTC-Opportunity-Confirmation"] = "confirmed";
  }

  if (!conversationClient) {
    showChatError("conversation_unavailable");
    return;
  }
  const intent = {
    message: userContent,
    model_id: window.UcaModelSelection.selectedId,
    opportunity_context: opportunityProfileHint == null ? null : { profile_snapshot_id: opportunityProfileHint }
  };
  if (promptCustomization != null) intent.prompt_customization = { text: promptCustomization };
  chatError.hidden = true;
  const pendingItem = appendChatMessage("user", userContent);
  chatInput.value = "";
  chatOpportunityConfirm.checked = false;
  try {
    const turn = await conversationClient.submit(intent, headers);
    chatPromptCustomization.value = "";
    syncPromptCustomizationCounter();
    if (turn.phase !== "completed") {
      if (!chatInput.value) chatInput.value = userContent;
      showChatError(turn.error || "request_failed", userContent);
    }
    else window.dispatchEvent(new CustomEvent("uca:provider-response", { detail: turn.response.provider }));
  } catch (error) {
    if (!chatInput.value) { chatInput.value = userContent; restoredConversationDraft = userContent; }
    if (pendingItem.isConnected) {
      pendingItem.dataset.status = "unconfirmed";
      appendChatNotice(pendingItem, "chat-failed-turn", "等待核对服务器结果；不会自动重新执行。");
    }
    showChatError(error?.code ?? "network_error", userContent);
  } finally {
    chatInput.focus({ preventScroll: true });
    syncChatEmptyState();
  }
}

function renderSavedConversation(detail) {
  // The transcript is a server projection, never a second source for future prompt history.
  const turns = detail?.turns ?? [];
  const validated = turns.map(turn => turn.response ? validateChatResponse(turn.response) : null);
  chatHistory.splice(0, chatHistory.length);
  for (const item of chatMessages.querySelectorAll(".chat-message")) item.remove();
  turns.forEach((turn, index) => {
    const user = appendChatMessage("user", turn.user);
    if (turn.phase === "completed") {
      const answer = validated[index];
      const item = appendChatMessage("assistant", answer.answer, answer.toolTrace);
      if (utf8Length(answer.answer) > CHAT_MAX_MESSAGE_BYTES) appendChatNotice(item, "chat-context-boundary",
        "这条回答较长，已完整保存；追问时请补充必要背景，服务器不会跨过这条记录选取更早的上下文。");
    } else {
      user.dataset.status = turn.phase;
      const messages = {
        running: "正在处理中，可以检查服务器结果。",
        failed: "这次没有完成回答。已执行的操作可能仍然有效，请先核对工具结果再决定是否重新提出任务。",
        interrupted: "这次请求被中断，系统不会自动重新执行；请先核对相关事项。"
      };
      appendChatNotice(user, "conversation-turn-state", messages[turn.phase]);
    }
  });
  syncChatEmptyState();
}

function syncProcedurePreview() {
  const value = procedureInput.value.trim();
  procedurePreview.textContent = `完整流程 ID：${value || "尚未输入"}`;
}

function displayLabel(value) {
  const labels = {
    exact_id: "精确流程 ID",
    structured_search: "结构化检索",
    fallback: "受限回退",
    fresh: "信息在复核有效期内",
    stale: "信息已过新鲜期",
    verified: "已验证证据链",
    unverified: "证据链未验证",
    not_required: "当前结果无需证据链",
    complete: "完整",
    truncated: "有界截断",
    none: "无已知不确定项",
    resolved: "未发现冲突",
    unresolved: "存在未解决冲突",
    unknown: "来源未声明",
    known_point: "已声明单点有效时间",
    known_interval: "已声明有效区间",
    cannot_verify: "当前无法验证",
    insufficient_evidence: "证据不足"
  };
  return labels[value] ?? String(value ?? "未知");
}

function formatTime(value) {
  if (!Number.isFinite(value)) {
    return "未提供";
  }
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) {
    return "无效时间";
  }
  return new Intl.DateTimeFormat("zh-CN", {
    dateStyle: "medium",
    timeStyle: "short",
    timeZone: "Asia/Shanghai"
  }).format(date);
}

function safeExternalLink(rawUrl) {
  try {
    const parsed = new URL(rawUrl, window.location.origin);
    if (parsed.protocol !== "https:" && parsed.protocol !== "http:") {
      return null;
    }
    return parsed.href;
  } catch (_error) {
    return null;
  }
}

function showError(message) {
  result.hidden = true;
  errorPanel.hidden = false;
  text(errorMessage, message);
}

function renderTags(values) {
  const container = document.querySelector("#audience-tags");
  clear(container);
  for (const value of values ?? []) {
    const tag = document.createElement("span");
    tag.textContent = value;
    container.appendChild(tag);
  }
}

function renderSteps(values) {
  const container = document.querySelector("#steps");
  clear(container);
  for (const step of values ?? []) {
    const item = document.createElement("li");
    const ordinal = document.createElement("span");
    ordinal.className = "step-index";
    ordinal.textContent = String((step.ordinal ?? 0) + 1).padStart(2, "0");
    const instruction = document.createElement("p");
    instruction.textContent = step.instruction ?? "未提供步骤说明";
    item.append(ordinal, instruction);
    container.appendChild(item);
  }
}

function renderPrerequisites(values) {
  const container = document.querySelector("#prerequisites");
  clear(container);
  const items = values ?? [];
  if (items.length === 0) {
    const item = document.createElement("li");
    item.textContent = "当前来源快照没有列出额外前置条件。";
    container.appendChild(item);
    return;
  }
  for (const prerequisite of items) {
    const item = document.createElement("li");
    item.textContent = prerequisite.condition ?? "未提供条件说明";
    container.appendChild(item);
  }
}

function renderTiming(view) {
  const interval = view.effective_interval;
  if (interval) {
    const from = interval.from == null ? "起点未声明" : formatTime(interval.from);
    const to = interval.to == null ? "持续有效" : formatTime(interval.to);
    text(document.querySelector("#effective-window"), `${from} → ${to}`);
  } else {
    text(document.querySelector("#effective-window"), "来源未声明统一生效区间");
  }

  const deadlines = view.deadlines ?? [];
  if (deadlines.length === 0) {
    text(document.querySelector("#deadline-state"), "来源未声明固定截止日期");
  } else {
    const summary = deadlines.map((deadline) => {
      const at = deadline.at == null ? "时间未声明" : formatTime(deadline.at);
      return `${deadline.label ?? "截止事项"}：${at}`;
    }).join("；");
    text(document.querySelector("#deadline-state"), summary);
  }
  text(
    document.querySelector("#validity-horizon"),
    displayLabel(view.evidence?.valid_interval?.kind)
  );
}

function renderContacts(values) {
  const container = document.querySelector("#contacts");
  clear(container);
  for (const contact of values ?? []) {
    const item = document.createElement("div");
    const name = document.createElement("strong");
    name.textContent = contact.name ?? "官方联系";
    const channel = document.createElement("span");
    channel.textContent = contact.channel ?? "联系方式未提供";
    const source = document.createElement("small");
    source.textContent = `来源：${contact.source_id ?? "未提供"}`;
    item.append(name, channel, source);
    container.appendChild(item);
  }
}

function renderEntryPoints(values) {
  const container = document.querySelector("#entry-points");
  clear(container);
  const admitted = (values ?? []).filter((entry) => safeExternalLink(entry.url));
  document.querySelector("#entry-section").hidden = admitted.length === 0;
  for (const entry of admitted) {
    const anchor = document.createElement("a");
    anchor.href = safeExternalLink(entry.url);
    anchor.target = "_blank";
    anchor.rel = "noopener noreferrer";
    const label = document.createElement("span");
    label.textContent = entry.label ?? "打开官方入口";
    const arrow = document.createElement("span");
    arrow.className = "link-arrow";
    arrow.setAttribute("aria-hidden", "true");
    arrow.textContent = "↗";
    anchor.append(label, arrow);
    container.appendChild(anchor);
  }
}

function renderSources(evidence) {
  const container = document.querySelector("#source-list");
  clear(container);
  for (const assessment of evidence.assessments ?? []) {
    const item = document.createElement("div");
    const source = document.createElement("strong");
    source.textContent = assessment.source_id ?? "未知来源";
    const detail = document.createElement("span");
    detail.textContent = `${displayLabel(assessment.authority)} · ${formatTime(assessment.last_verified_at)}`;
    item.append(source, detail);
    container.appendChild(item);
  }
}

function renderFound(terminal) {
  const outcome = terminal.outcome;
  const view = outcome.view;
  const evidence = view.evidence;
  const freshness = outcome.freshness?.kind ?? "unknown";

  text(document.querySelector("#result-title"), view.title);
  text(document.querySelector("#result-kicker"), view.procedure_id);
  renderTags(view.audience_tags);
  renderPrerequisites(view.prerequisites);
  renderSteps(view.ordered_steps);
  renderTiming(view);
  renderEntryPoints(view.entry_points);
  renderContacts(view.contacts);
  renderSources(evidence);

  text(document.querySelector("#freshness-label"), displayLabel(freshness));
  text(document.querySelector("#uncertainty-label"), displayLabel(view.uncertainty_state));
  document.querySelector("#freshness-dot").dataset.state = freshness;
  text(document.querySelector("#lookup-path"), displayLabel(view.lookup_path));
  text(document.querySelector("#last-verified"), formatTime(evidence.last_verified_at));
  text(document.querySelector("#lineage-state"), displayLabel(terminal.lineage?.kind));
  text(document.querySelector("#conflict-state"), displayLabel(view.conflict_state?.kind));
  text(document.querySelector("#evidence-set-digest"), terminal.lineage?.evidence_set_digest);
  text(document.querySelector("#materialization-receipt"), terminal.lineage?.materialization_receipt_id);
  text(document.querySelector("#revision-count"), terminal.lineage?.revision_count);
  text(document.querySelector("#projection-state"), displayLabel(evidence.projection?.kind));

  errorPanel.hidden = true;
  result.hidden = false;
  status.textContent = "已载入经复核的演示资料。";
}

function renderResponse(payload, checklistToken) {
  if (payload?.kind !== "available" || payload?.redaction !== "public") {
    showError("服务器没有返回可公开呈现的查询结果。请核对流程 ID 或稍后重试。");
    return;
  }
  const terminal = payload.terminal;
  if (terminal?.outcome?.kind !== "found") {
    const state = displayLabel(terminal?.outcome?.kind);
    showError(`当前流程状态：${state}。没有可安全呈现的办理步骤。`);
    return;
  }
  renderFound(terminal);
  window.UcaAffairsChecklist?.render(payload, checklistToken);
}

let affairsLookupSequence = 0;
async function lookup() {
  const sequence = ++affairsLookupSequence;
  const checklistToken = window.UcaAffairsChecklist?.invalidate();
  result.hidden = true;
  const procedureId = procedureInput.value.trim();
  if (!procedureId) {
    showError("请输入流程 ID。");
    return;
  }

  submitButton.disabled = true;
  submitButton.textContent = "读取中…";
  status.textContent = "正在通过 M00 → M10 → bounded Harness → Market current authorization → ToolGateway → Affairs Plugin 读取可验证结果…";
  errorPanel.hidden = true;

  try {
    const response = await fetch(`/api/v1/affairs/${encodeURIComponent(procedureId)}`, {
      method: "GET",
      headers: {
        "Accept": "application/json",
        "X-USTC-Client-Protocol-Major": "1"
      },
      cache: "no-store"
    });
    const payload = await response.json();
    if (sequence !== affairsLookupSequence) return;
    if (!response.ok) {
      throw new Error(payload?.error ?? `HTTP ${response.status}`);
    }
    renderResponse(payload, checklistToken);
  } catch (error) {
    if (sequence !== affairsLookupSequence) return;
    window.UcaAffairsChecklist?.invalidate();
    showError(`读取失败：${error instanceof Error ? error.message : "未知错误"}`);
    status.textContent = "读取未完成。";
  } finally {
    if (sequence === affairsLookupSequence) {
      submitButton.disabled = false;
      submitButton.textContent = "查看流程";
    }
  }
}

function renderChangeFeed(payload) {
  if (payload?.kind === "error") {
    const code = payload.error?.error?.wire_code ?? payload.error?.wire_code ?? "change_feed_denied";
    throw new Error(`ChangeRadar 拒绝了本次读取：${code}`);
  }
  if (payload?.kind !== "change_feed_accepted") {
    throw new Error("服务器没有返回 ChangeRadar terminal result");
  }
  const outcome = payload.terminal?.outcome;
  if (outcome?.kind === "not_found") {
    radarResult.hidden = true;
    radarStatus.textContent = `未找到变更板：${outcome.board_id ?? "未知 board"}`;
    return;
  }
  if (outcome?.kind !== "found") {
    throw new Error("ChangeRadar outcome 无法呈现");
  }
  const view = outcome.view;
  const entry = view?.entries?.[0];
  if (!entry) {
    radarResult.hidden = true;
    radarStatus.textContent = "当前变更板没有已发布事件。";
    return;
  }
  text(document.querySelector("#radar-board-id"), view.board_id);
  text(document.querySelector("#radar-health"), `source ${entry.source_health}`);
  text(document.querySelector("#radar-observed"), `观测于 ${formatTime(entry.observed_at)}`);
  text(
    document.querySelector("#radar-effective"),
    `生效 ${formatTime(entry.effective_from)} → ${formatTime(entry.effective_to)}`
  );
  text(document.querySelector("#radar-published"), `发布于 ${formatTime(entry.published_at)}`);
  text(document.querySelector("#radar-source"), `${entry.source_id} · ${entry.source_url}`);
  text(document.querySelector("#radar-old-revision"), entry.old_revision_id);
  text(document.querySelector("#radar-old-raw-digest"), entry.old_raw_sha256);
  text(document.querySelector("#radar-old-normalized-digest"), entry.old_normalized_sha256);
  text(
    document.querySelector("#radar-old-review"),
    `${entry.old_source_reviewer} · ${entry.old_source_review_evidence}`
  );
  text(document.querySelector("#radar-new-revision"), entry.new_revision_id);
  text(document.querySelector("#radar-new-raw-digest"), entry.new_raw_sha256);
  text(document.querySelector("#radar-new-normalized-digest"), entry.new_normalized_sha256);
  text(
    document.querySelector("#radar-new-review"),
    `${entry.new_source_reviewer} · ${entry.new_source_review_evidence}`
  );
  text(document.querySelector("#radar-evidence-digest"), entry.evidence_set_digest);

  const fields = document.querySelector("#radar-fields");
  clear(fields);
  for (const change of entry.changed_fields ?? []) {
    const card = document.createElement("article");
    const name = document.createElement("h3");
    name.textContent = change.field ?? "未命名字段";
    const values = document.createElement("div");
    values.className = "radar-values";
    const before = document.createElement("span");
    before.textContent = change.before ?? "∅";
    const arrow = document.createElement("b");
    arrow.textContent = "→";
    const after = document.createElement("span");
    after.textContent = change.after ?? "∅";
    values.append(before, arrow, after);
    card.append(name, values);
    fields.appendChild(card);
  }
  radarResult.hidden = false;
  radarStatus.textContent = "已读取当前发布的校历变更。可展开来源记录核对。";
}

async function loadChangeFeed() {
  radarButton.disabled = true;
  radarButton.textContent = "读取中…";
  radarStatus.textContent = "正在读取确定性 semantic change…";
  try {
    const response = await fetch(
      "/api/v1/changes/board%3Austc%3Aacademic-calendar",
      { method: "GET", headers: { "Accept": "application/json" }, cache: "no-store" }
    );
    const payload = await response.json();
    if (!response.ok) {
      throw new Error(payload?.error ?? `HTTP ${response.status}`);
    }
    renderChangeFeed(payload);
  } catch (error) {
    radarResult.hidden = true;
    radarStatus.textContent = `变更板读取失败：${error instanceof Error ? error.message : "未知错误"}`;
  } finally {
    radarButton.disabled = false;
    radarButton.textContent = "重新读取";
  }
}

function validOpportunityProfileHint(value) {
  return typeof value === "string"
    && value.trim().length > 0
    && !value.includes("\0")
    && utf8Length(value) <= CHAT_MAX_PROFILE_SNAPSHOT_ID_BYTES;
}

function setOpportunityHint(value) {
  const normalizedValue = validOpportunityProfileHint(value) ? value : null;
  if (opportunityProfileId !== normalizedValue) {
    chatOpportunityConfirm.checked = false;
    opportunityPlanResult.hidden = true;
  }
  opportunityProfileId = normalizedValue;
  try {
    if (normalizedValue) {
      window.localStorage.setItem(OPPORTUNITY_PROFILE_HINT, normalizedValue);
    } else {
      window.localStorage.removeItem(OPPORTUNITY_PROFILE_HINT);
    }
  } catch (_error) {
    // The server remains authoritative; storage is only a best-effort UI hint.
  }
  syncOpportunityAvailability();
}

function syncOpportunityAvailability() {
  const enabled = Boolean(opportunityProfileId);
  setOpportunityBusy(opportunityBusy);
  if (!enabled) {
    chatOpportunityConfirm.checked = false;
  }
  const toolsUnavailable = window.UcaModelSelection?.toolCalling === false;
  if (toolsUnavailable) chatOpportunityConfirm.checked = false;
  chatOpportunityConfirm.disabled = chatPending || !enabled || toolsUnavailable;
  chatOpportunityConfirm.setAttribute("aria-disabled", String(chatOpportunityConfirm.disabled));
  chatOpportunityState.textContent = toolsUnavailable
    ? "当前模型连接不提供工具调用，无法在对话中使用课程档案。"
    : enabled
    ? "已找到演示档案。勾选后仅这次请求可使用，发送后会自动取消。"
    : "尚无可用的演示档案；请进入课程规划插件，明确同意并创建。";
}

function readOpportunityHint() {
  try {
    return window.localStorage.getItem(OPPORTUNITY_PROFILE_HINT);
  } catch (_error) {
    return null;
  }
}

let boundedTokenCounter = 0;

function boundedToken() {
  boundedTokenCounter = (boundedTokenCounter + 1) % 0xffff;
  const random = Math.floor(Math.random() * 0xffffffff).toString(16).padStart(8, "0");
  return `${Date.now()}-${boundedTokenCounter.toString(16).padStart(4, "0")}-${random}`;
}

function mintBoundedId(prefix) {
  const token = typeof crypto !== "undefined" && typeof crypto.randomUUID === "function"
    ? crypto.randomUUID()
    : boundedToken();
  return `${prefix}web:${token}`;
}

function pendingOperationStorageKey(operation) {
  return OPPORTUNITY_PENDING_OPERATIONS[operation] ?? null;
}

function storePendingOperation(operation) {
  const key = pendingOperationStorageKey(operation.operation);
  if (!key) {
    throw new Error("unsupported Opportunity retry operation");
  }
  opportunityPendingMemory.set(operation.operation, operation);
  try {
    window.localStorage.setItem(key, JSON.stringify(operation));
  } catch (_error) {
    // The in-memory carrier preserves same-page retries when storage is unavailable.
  }
}

function readPendingOperation(operation) {
  const inMemory = opportunityPendingMemory.get(operation);
  if (inMemory) {
    return inMemory;
  }
  const key = pendingOperationStorageKey(operation);
  if (!key) {
    return null;
  }
  try {
    const raw = window.localStorage.getItem(key);
    const value = raw == null ? null : JSON.parse(raw);
    if (value && typeof value === "object") {
      opportunityPendingMemory.set(operation, value);
      return value;
    }
    return null;
  } catch (_error) {
    return null;
  }
}

function clearPendingOperation(operation, expectedEnvelope) {
  const pending = readPendingOperation(operation);
  if (!pending || pending.idempotency_key !== expectedEnvelope.idempotency_key) {
    return;
  }
  opportunityPendingMemory.delete(operation);
  const key = pendingOperationStorageKey(operation);
  if (!key) {
    return;
  }
  try {
    window.localStorage.removeItem(key);
  } catch (_error) {
    return;
  }
}

function mintOperationEnvelope(operation, profileId) {
  return {
    operation,
    request_id: mintBoundedId("req:"),
    correlation_id: mintBoundedId("corr:"),
    idempotency_key: mintBoundedId("idem:"),
    timestamp: Date.now(),
    profile_id: profileId
  };
}

function reusableOperationEnvelope(operation, profileId) {
  const pending = readPendingOperation(operation);
  if (!pending || pending.operation !== operation) {
    return null;
  }
  if (
    typeof pending.request_id !== "string" ||
    typeof pending.correlation_id !== "string" ||
    typeof pending.idempotency_key !== "string" ||
    !Number.isFinite(pending.timestamp) ||
    pending.timestamp <= 0
  ) {
    return null;
  }
  if (operation === "delete" && pending.profile_id !== profileId) {
    return null;
  }
  return pending;
}

async function requestOpportunity(url, options) {
  const response = await fetch(url, {
    cache: "no-store",
    ...options,
    headers: {
      "Accept": "application/json",
      "Content-Type": "application/json",
      ...(options?.headers ?? {}),
      "X-USTC-Opportunity-Confirmation": "confirmed"
    }
  });
  const payload = await response.json();
  if (!response.ok) {
    const rejection = payload?.rejection?.kind;
    const error = payload?.error?.error?.wire_code ?? payload?.error?.wire_code ?? payload?.error;
    throw new Error(rejection ?? error ?? `HTTP ${response.status}`);
  }
  if (payload?.kind === "incomplete") {
    throw new Error("操作可能已执行，但 outcome receipt 尚未确认；请稍后按同一请求重试。");
  }
  if (payload?.kind !== "opportunity_accepted") {
    throw new Error("服务器没有返回 Opportunity terminal result");
  }
  return payload.terminal;
}

function createOpportunityRequestBody(envelope) {
  // Values are snapshotted with the idempotency envelope, not read again on retry.
  // Older persisted requests used the fixed public synthetic template.
  const profile = envelope.profile_data ?? OPPORTUNITY_DEMO_TEMPLATE;
  return {
    consent: true,
    request_id: envelope.request_id,
    correlation_id: envelope.correlation_id,
    idempotency_key: envelope.idempotency_key,
    consented_at: envelope.timestamp,
    completed_courses: profile.completed_courses,
    min_credits: profile.min_credits,
    max_credits: profile.max_credits,
    preference_weights: profile.preference_weights
  };
}

function deleteOpportunityRequestBody(envelope) {
  return {
    confirm_delete: true,
    request_id: envelope.request_id,
    correlation_id: envelope.correlation_id,
    idempotency_key: envelope.idempotency_key,
    revoked_at: envelope.timestamp
  };
}

async function submitOpportunityOperation(url, body) {
  const response = await fetch(url, {
    method: "POST",
    cache: "no-store",
    headers: {
      "Accept": "application/json",
      "Content-Type": "application/json",
      "X-USTC-Opportunity-Confirmation": "confirmed"
    },
    body
  });
  let payload = null;
  try {
    payload = await response.json();
  } catch (_error) {
    return { outcome: "unknown", payload: null };
  }
  if (payload?.kind === "opportunity_accepted" || payload?.kind === "opportunity_rejected") {
    return { outcome: "terminal", payload };
  }
  return { outcome: "unknown", payload };
}

function renderOpportunityProfileTerminal(terminal) {
  if (terminal?.kind !== "profile_created" && terminal?.kind !== "profile_found") {
    throw new Error("Opportunity profile terminal 无法呈现");
  }
  const profile = terminal.profile;
  setOpportunityHint(profile.profile_snapshot_id);
  text(document.querySelector("#opportunity-profile-id"), profile.profile_snapshot_id);
  text(document.querySelector("#opportunity-consent-id"), profile.consent_id);
  text(
    document.querySelector("#opportunity-consent-fields"),
    (profile.consent_fields ?? []).join(" · ")
  );
  text(
    document.querySelector("#opportunity-profile-bounds"),
    `${profile.completed_course_count} 门已修 · ${profile.min_credits}–${profile.max_credits} 学分 · ${profile.preference_count} 项偏好`
  );
  opportunityProfile.hidden = false;
  opportunityDeleted.hidden = true;
  opportunityStatus.textContent = terminal.kind === "profile_created"
    ? "已通过 consent-bound private write 创建档案；raw profile 不进入公共 projection。"
    : "已通过 authenticated owner read 从 durable store 读取档案 metadata。";
}

function blockerLabel(blocker) {
  if (blocker?.kind === "missing_prerequisite") {
    return `缺少先修 ${blocker.course_code ?? "未知课程"}`;
  }
  const labels = {
    unavailable: "课程事实不可用",
    unresolved_identity: "课程身份尚未解析",
    conflicting_fact: "来源事实冲突",
    cycle_affected: "依赖图受环影响",
    unmet_rule: "未满足规则",
    unknown_course: "课程事实未知",
    requirement_unmet: "培养要求未满足"
  };
  return labels[blocker?.kind] ?? String(blocker?.kind ?? "未知阻塞");
}

function renderQualifications(values) {
  const container = document.querySelector("#opportunity-qualifications");
  clear(container);
  for (const qualification of values ?? []) {
    const item = document.createElement("article");
    const title = document.createElement("strong");
    title.textContent = `${qualification.course_code} · ${qualification.eligible ? "满足条件" : "仍缺条件"}`;
    const blockers = document.createElement("p");
    blockers.textContent = qualification.eligible
      ? "当前 profile 与 reviewed facts 未发现资格阻塞。"
      : (qualification.blockers ?? []).map(blockerLabel).join("；");
    const source = document.createElement("small");
    source.textContent = `来源 ${qualification.source_id} · revision ${qualification.source_revision_id}`;
    item.append(title, blockers, source);
    container.appendChild(item);
  }
}

function renderCandidates(decision) {
  const container = document.querySelector("#opportunity-candidates");
  clear(container);
  if (decision?.kind !== "planned") {
    const empty = document.createElement("p");
    empty.textContent = "在当前 hard constraints 下没有可行计划；系统没有把推断冒充结果。";
    container.appendChild(empty);
    return;
  }
  for (const candidate of decision.candidates ?? []) {
    const item = document.createElement("article");
    const title = document.createElement("strong");
    title.textContent = (candidate.course_codes ?? []).join(" + ");
    const score = document.createElement("p");
    score.textContent = `${candidate.total_credits} 学分 · soft score ${candidate.soft_score} · hard violations ${(candidate.hard_constraint_violations ?? []).length}`;
    const rationale = document.createElement("p");
    rationale.textContent = (candidate.rationale ?? []).join("；") || "无额外解释";
    const evidence = document.createElement("small");
    evidence.textContent = (candidate.provenance ?? [])
      .map((fact) => `${fact.fact} @ ${fact.revision} [${fact.conflict_status}]`)
      .join("；");
    item.append(title, score, rationale, evidence);
    container.appendChild(item);
  }
}

function renderOpportunityPlan(terminal) {
  if (terminal?.kind !== "plan_generated") {
    throw new Error("Opportunity plan terminal 无法呈现");
  }
  const plan = terminal.plan;
  text(document.querySelector("#opportunity-plan-receipt"), plan.receipt_id);
  text(document.querySelector("#opportunity-source-revision"), plan.source_revision_id);
  text(
    document.querySelector("#opportunity-plan-binding"),
    `${plan.profile_snapshot_id} · ${plan.consent_id}`
  );
  const decisionSummary = plan.decision?.kind === "planned"
    ? `planned · hard violations ${plan.decision.hard_constraint_violations}`
    : plan.decision?.kind;
  text(document.querySelector("#opportunity-plan-decision"), decisionSummary);
  renderQualifications(plan.qualifications);
  renderCandidates(plan.decision);
  opportunityPlanResult.hidden = false;
  opportunityDeleted.hidden = true;
  opportunityStatus.textContent = "已根据保存的档案和当前课程资料生成方案。";
}

async function createOpportunityProfile() {
  if (opportunityBusy || chatPending) return;
  if (opportunityProfileId && !reusableOperationEnvelope("create", null)) {
    opportunityStatus.textContent = "当前已有已保存档案。若要采用新草稿，请先明确撤回/删除当前档案，再重新同意并创建；不会自动删除。";
    return;
  }
  if (!opportunityConsent.checked) {
    opportunityStatus.textContent = "必须先明确勾选 consent；未同意时不会发出 private write。";
    return;
  }
  let envelope = reusableOperationEnvelope("create", null);
  if (!envelope) {
    try {
      envelope = mintOperationEnvelope("create", null);
      envelope.profile_data = window.UcaCourseEditor.readDraft();
    } catch (error) {
      opportunityStatus.textContent = `档案草稿未提交：${error.message}`;
      return;
    }
  }
  storePendingOperation(envelope);
  window.UcaCourseEditor.setLocked(true);
  setOpportunityBusy(true);
  opportunityStatus.textContent = "正在创建 tenant-private synthetic profile…";
  let outcome = null;
  try {
    outcome = await submitOpportunityOperation(
      "/api/v1/opportunity/profiles",
      JSON.stringify(createOpportunityRequestBody(envelope))
    );
  } catch (_error) {
    outcome = { outcome: "unknown", payload: null };
  } finally {
    setOpportunityBusy(false);
  }
  if (outcome.outcome !== "terminal") {
    opportunityStatus.textContent = "档案创建结果尚未确认；已保留同一请求 envelope，请再次点击以按同一请求重试。";
    return;
  }
  clearPendingOperation("create", envelope);
  window.UcaCourseEditor.setLocked(false);
  opportunityConsent.checked = false;
  const payload = outcome.payload;
  if (payload?.kind !== "opportunity_accepted") {
    opportunityStatus.textContent = `档案创建失败：${payload?.rejection?.kind ?? "未知错误"}`;
    return;
  }
  renderOpportunityProfileTerminal(payload.terminal);
}

async function viewOpportunityProfile() {
  if (opportunityBusy) return;
  if (!opportunityProfileId) {
    opportunityStatus.textContent = "没有可读取的 profile ID hint。";
    return;
  }
  setOpportunityBusy(true);
  opportunityStatus.textContent = "正在以 authenticated owner 读取 private profile metadata…";
  try {
    const terminal = await requestOpportunity(
      `/api/v1/opportunity/profiles/${encodeURIComponent(opportunityProfileId)}`,
      { method: "GET", headers: { "Accept": "application/json" } }
    );
    renderOpportunityProfileTerminal(terminal);
  } catch (error) {
    const message = error instanceof Error ? error.message : "未知错误";
    opportunityStatus.textContent = `档案读取失败：${message}`;
    if (message === "profile_deleted" || message === "missing_profile") {
      setOpportunityHint(null);
    }
  } finally {
    setOpportunityBusy(false);
  }
}

async function generateOpportunityPlan() {
  if (opportunityBusy) return;
  if (!opportunityProfileId) {
    opportunityStatus.textContent = "请先创建或恢复 private profile。";
    return;
  }
  setOpportunityBusy(true);
  opportunityPlanResult.hidden = true;
  opportunityStatus.textContent = "正在校验 current source、资格条件、依赖与 hard constraints…";
  try {
    const terminal = await requestOpportunity("/api/v1/opportunity/plans", {
      method: "POST",
      body: JSON.stringify({
        profile_snapshot_id: opportunityProfileId,
        max_results: 3,
        beam_width: 1024
      })
    });
    renderOpportunityPlan(terminal);
  } catch (error) {
    opportunityPlanResult.hidden = true;
    opportunityStatus.textContent = `计划生成失败：${error instanceof Error ? error.message : "未知错误"}`;
  } finally {
    setOpportunityBusy(false);
  }
}

async function deleteOpportunityProfile() {
  if (opportunityBusy || chatPending) return;
  if (!opportunityProfileId) {
    opportunityStatus.textContent = "没有可删除的 private profile。";
    return;
  }
  const envelope = reusableOperationEnvelope("delete", opportunityProfileId)
    ?? mintOperationEnvelope("delete", opportunityProfileId);
  storePendingOperation(envelope);
  setOpportunityBusy(true);
  opportunityStatus.textContent = "正在撤回 consent 并原子删除 private payload…";
  const target = envelope.profile_id ?? opportunityProfileId;
  let outcome = null;
  try {
    outcome = await submitOpportunityOperation(
      `/api/v1/opportunity/profiles/${encodeURIComponent(target)}/revoke-delete`,
      JSON.stringify(deleteOpportunityRequestBody(envelope))
    );
  } catch (_error) {
    outcome = { outcome: "unknown", payload: null };
  } finally {
    setOpportunityBusy(false);
  }
  if (outcome.outcome !== "terminal") {
    opportunityStatus.textContent = "撤回/删除结果尚未确认；已保留同一请求 envelope，请再次点击以按同一请求重试。";
    return;
  }
  clearPendingOperation("delete", envelope);
  const payload = outcome.payload;
  if (payload?.kind !== "opportunity_accepted") {
    opportunityStatus.textContent = `撤回/删除失败：${payload?.rejection?.kind ?? "未知错误"}`;
    return;
  }
  const terminal = payload.terminal;
  if (terminal?.kind !== "profile_deleted") {
    opportunityStatus.textContent = "Opportunity delete terminal 无法呈现";
    return;
  }
  const deletion = terminal.deletion;
  setOpportunityHint(null);
  opportunityConsent.checked = false;
  opportunityProfile.hidden = true;
  opportunityPlanResult.hidden = true;
  opportunityDeleted.hidden = false;
  text(
    document.querySelector("#opportunity-delete-receipt"),
    `${deletion.deletion_receipt_id} · profile ${deletion.profile_snapshot_id} · deleted ${formatTime(deletion.deleted_at)}`
  );
  opportunityStatus.textContent = "删除收据已持久化；旧档案不可恢复。本页 synthetic 草稿不是已保存档案，可重新明确同意后创建；不会自动创建。";
}

assertChatDomContract();
chatForm.addEventListener("submit", (event) => {
  event.preventDefault();
  void submitChat();
});
const chatActivity = window.UcaChatActivity.mount(document.querySelector("#chat-activity"));
conversationClient = window.UcaConversations.mount(
  document.querySelector("#conversation-history"), document.querySelector("#conversation-recovery"), {
    activityStarted: (id, requestId) => { chatProgress.hidden = true; chatActivity.start(id, requestId); },
    activityFinished: outcome => chatActivity.stop(outcome),
    activityCleared: () => chatActivity.clear(),
    render: renderSavedConversation,
    validateResponse: validateChatResponse,
    busy: (value, context) => { setChatBusy(value); if (context?.kind === "management") chatProgress.hidden = true; },
    availability: ({ canSend, canSwitch }) => {
      conversationCanSend = canSend; conversationCanSwitch = canSwitch; syncModelAvailability();
      chatClear.disabled = !canSwitch; chatClear.setAttribute("aria-disabled", String(!canSwitch));
    },
    error: code => showChatError(code),
    selected: ({preserveDraft = false} = {}) => {
      if (!preserveDraft) {
      chatInput.value = ""; restoredConversationDraft = null; chatInput.setCustomValidity("");
      chatPromptCustomization.value = ""; chatPromptCustomization.setCustomValidity("");
      chatOpportunityConfirm.checked = false;
      }
      chatError.hidden = true; syncPromptCustomizationCounter(); window.UcaShell?.navigate("chat"); syncChatEmptyState();
    },
    recovered: turn => {
      chatError.hidden = true;
      if (turn.phase === "completed" && restoredConversationDraft === turn.user && chatInput.value.trim() === turn.user) chatInput.value = "";
      restoredConversationDraft = null;
      if (turn.response) window.dispatchEvent(new CustomEvent("uca:provider-response", { detail: turn.response.provider }));
      syncChatEmptyState();
    }
  }
);
chatClear.addEventListener("click", () => { conversationClient.newConversation(); });
chatInput.addEventListener("input", () => {
  restoredConversationDraft = null;
  chatInput.setCustomValidity("");
});
chatPromptCustomization.addEventListener("input", () => {
  chatPromptCustomization.setCustomValidity("");
  syncPromptCustomizationCounter();
});
chatInput.addEventListener("keydown", (event) => {
  if (event.key === "Enter" && !event.shiftKey && !event.isComposing) {
    event.preventDefault();
    if (!chatSend.disabled) {
      chatForm.requestSubmit();
    }
  }
});

opportunityCreate.addEventListener("click", () => {
  void createOpportunityProfile();
});
opportunityView.addEventListener("click", () => {
  void viewOpportunityProfile();
});
opportunityPlan.addEventListener("click", () => {
  void generateOpportunityPlan();
});
opportunityDelete.addEventListener("click", () => {
  void deleteOpportunityProfile();
});

radarButton.addEventListener("click", () => {
  void loadChangeFeed();
});

form.addEventListener("submit", (event) => {
  event.preventDefault();
  void lookup();
});

procedureInput.addEventListener("input", syncProcedurePreview);
procedureInput.addEventListener("keydown", (event) => {
  if (event.key === "Enter" && !event.shiftKey) {
    event.preventDefault();
    form.requestSubmit();
  }
});
syncPromptCustomizationCounter();
syncProcedurePreview();
setOpportunityHint(readOpportunityHint());
if (opportunityProfileId) {
  void viewOpportunityProfile();
}

void lookup();
window.UcaAdminControls.mount({
  root: document.querySelector('.operator-settings'),
  request: (...args) => fetch(...args),
  onChangePublished: loadChangeFeed
});
void loadChangeFeed();

window.addEventListener("uca:provider-status", syncOpportunityAvailability);

window.addEventListener("uca:model-selection", () => { syncModelAvailability(); syncOpportunityAvailability(); });
syncModelAvailability();
