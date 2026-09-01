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
const radarPublicationRefresh = document.querySelector("#radar-publication-refresh");
const radarPublicationConfirm = document.querySelector("#radar-publication-confirm");
const radarPublicationPublish = document.querySelector("#radar-publication-publish");
const radarPublicationStatus = document.querySelector("#radar-publication-status");
const publicationRefresh = document.querySelector("#publication-refresh");
const publicationConfirm = document.querySelector("#publication-confirm");
const publicationPublish = document.querySelector("#publication-publish");
const publicationStatus = document.querySelector("#publication-status");
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

function clear(element) {
  while (element.firstChild) {
    element.removeChild(element.firstChild);
  }
}

function text(element, value) {
  element.textContent = value ?? "—";
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
  status.textContent = "已载入当前 source-grounded published fixture 结果。";
}

function renderResponse(payload) {
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
}

async function lookup() {
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
    if (!response.ok) {
      throw new Error(payload?.error ?? `HTTP ${response.status}`);
    }
    renderResponse(payload);
  } catch (error) {
    showError(`读取失败：${error instanceof Error ? error.message : "未知错误"}`);
    status.textContent = "读取未完成。";
  } finally {
    submitButton.disabled = false;
    submitButton.textContent = "查看流程";
  }
}

async function requestPublication(method, body) {
  const response = await fetch("/api/v1/demo/administrator/affairs/publication", {
    method,
    headers: {
      "Accept": "application/json",
      "Content-Type": "application/json",
      "X-USTC-Agent-Administrator-Demo": "confirm-v1"
    },
    body,
    cache: "no-store"
  });
  const payload = await response.json();
  if (!response.ok) {
    const detail = payload?.outcome?.error ?? payload?.error ?? `HTTP ${response.status}`;
    throw new Error(detail);
  }
  return payload;
}

function renderPublicationStatus(payload) {
  if (payload?.schema !== "ustc-affairs-publication-status/v1") {
    throw new Error("publication status schema 不匹配");
  }
  text(document.querySelector("#publication-revision"), payload.publication_revision);
  text(document.querySelector("#publication-receipt"), payload.publication_receipt_id);
  text(document.querySelector("#publication-evidence-count"), payload.control_evidence_event_count);
  publicationStatus.textContent = `已恢复 durable publication revision ${payload.publication_revision ?? "unknown"}。`;
}

async function loadPublicationStatus() {
  publicationRefresh.disabled = true;
  publicationStatus.textContent = "正在读取 durable publication 与 M00 control evidence 状态…";
  try {
    renderPublicationStatus(await requestPublication("GET"));
  } catch (error) {
    publicationStatus.textContent = `状态读取失败：${error instanceof Error ? error.message : "未知错误"}`;
  } finally {
    publicationRefresh.disabled = false;
  }
}

async function publishAffairsDemo() {
  if (!publicationConfirm.checked) {
    publicationStatus.textContent = "必须先显式确认固定 demo publication。";
    return;
  }
  publicationPublish.disabled = true;
  publicationStatus.textContent = "正在执行 M10 → M00 admission/evidence → M71 publication…";
  try {
    const payload = await requestPublication(
      "POST",
      JSON.stringify({ confirm_publish: true })
    );
    if (
      payload?.schema !== "ustc-affairs-publication-response/v1" ||
      payload?.outcome?.kind !== "published"
    ) {
      throw new Error("publication response schema 无法呈现");
    }
    publicationStatus.textContent = `M71 已返回 revision ${payload.outcome.publication_revision}；正在回读 durable state…`;
    await loadPublicationStatus();
  } catch (error) {
    publicationStatus.textContent = `发布失败：${error instanceof Error ? error.message : "未知错误"}`;
  } finally {
    publicationPublish.disabled = !publicationConfirm.checked;
  }
}

async function requestChangePublication(method, body) {
  const response = await fetch("/api/v1/demo/administrator/changes/publication", {
    method,
    headers: {
      "Accept": "application/json",
      "Content-Type": "application/json",
      "X-USTC-Agent-Administrator-Demo": "confirm-v1"
    },
    body,
    cache: "no-store"
  });
  const payload = await response.json();
  if (!response.ok) {
    throw new Error(payload?.error ?? `HTTP ${response.status}`);
  }
  return payload;
}

function renderChangePublicationStatus(payload) {
  if (payload?.schema !== "ustc-change-publication-status/v1") {
    throw new Error("ChangeRadar publication status schema 不匹配");
  }
  text(document.querySelector("#radar-publication-review-count"), payload.review_count);
  text(document.querySelector("#radar-publication-count"), payload.publication_count);
  text(
    document.querySelector("#radar-publication-receipt"),
    payload.publication_receipt_id ?? "尚未发布"
  );
  text(
    document.querySelector("#radar-publication-evidence-count"),
    payload.control_evidence_event_count
  );
  radarPublicationStatus.textContent = payload.publication_count === 0
    ? "固定 candidate 已准备，但尚未发布；public JSON/Atom 仍为空。"
    : `已恢复 durable ChangeRadar publication ${payload.publication_receipt_id}。`;
}

async function loadChangePublicationStatus() {
  radarPublicationRefresh.disabled = true;
  radarPublicationStatus.textContent = "正在读取 ChangeRadar durable state…";
  try {
    renderChangePublicationStatus(await requestChangePublication("GET"));
  } catch (error) {
    radarPublicationStatus.textContent = `状态读取失败：${error instanceof Error ? error.message : "未知错误"}`;
  } finally {
    radarPublicationRefresh.disabled = false;
  }
}

async function publishChangeDemo() {
  if (!radarPublicationConfirm.checked) {
    radarPublicationStatus.textContent = "必须先显式确认固定 ChangeRadar demo publication。";
    return;
  }
  radarPublicationPublish.disabled = true;
  radarPublicationStatus.textContent = "正在执行 M10 → M00 durable evidence → owning M70 publication…";
  try {
    const payload = await requestChangePublication(
      "POST",
      JSON.stringify({ confirm_publish: true })
    );
    if (
      payload?.schema !== "ustc-change-publication-response/v1" ||
      payload?.outcome?.kind !== "published"
    ) {
      throw new Error("ChangeRadar publication response schema 无法呈现");
    }
    radarPublicationStatus.textContent = "M70 已返回 typed publication receipt；正在回读 durable state 与 public feed…";
    await loadChangePublicationStatus();
    await loadChangeFeed();
  } catch (error) {
    radarPublicationStatus.textContent = `发布失败：${error instanceof Error ? error.message : "未知错误"}`;
  } finally {
    radarPublicationPublish.disabled = !radarPublicationConfirm.checked;
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
  radarStatus.textContent = "已通过 M00 → M10 → bounded Harness → Market current authorization → ToolGateway → ChangeRadar Plugin 读取。";
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

function setOpportunityHint(value) {
  opportunityProfileId = value;
  try {
    if (value) {
      window.localStorage.setItem(OPPORTUNITY_PROFILE_HINT, value);
    } else {
      window.localStorage.removeItem(OPPORTUNITY_PROFILE_HINT);
    }
  } catch (_error) {
    // The server remains authoritative; storage is only a best-effort UI hint.
  }
  const enabled = Boolean(value);
  opportunityView.disabled = !enabled;
  opportunityPlan.disabled = !enabled;
  opportunityDelete.disabled = !enabled;
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
  return {
    consent: true,
    request_id: envelope.request_id,
    correlation_id: envelope.correlation_id,
    idempotency_key: envelope.idempotency_key,
    consented_at: envelope.timestamp,
    completed_courses: OPPORTUNITY_DEMO_TEMPLATE.completed_courses,
    min_credits: OPPORTUNITY_DEMO_TEMPLATE.min_credits,
    max_credits: OPPORTUNITY_DEMO_TEMPLATE.max_credits,
    preference_weights: OPPORTUNITY_DEMO_TEMPLATE.preference_weights
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
  opportunityStatus.textContent = "已通过 M00 → M10 → transaction-current M20 authorization → static M72 Opportunity use case → M60 current source 生成计划。";
}

async function createOpportunityProfile() {
  if (!opportunityConsent.checked) {
    opportunityStatus.textContent = "必须先明确勾选 consent；未同意时不会发出 private write。";
    return;
  }
  const envelope = reusableOperationEnvelope("create", null) ?? mintOperationEnvelope("create", null);
  storePendingOperation(envelope);
  opportunityCreate.disabled = true;
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
    opportunityCreate.disabled = false;
  }
  if (outcome.outcome !== "terminal") {
    opportunityStatus.textContent = "档案创建结果尚未确认；已保留同一请求 envelope，请再次点击以按同一请求重试。";
    return;
  }
  clearPendingOperation("create", envelope);
  const payload = outcome.payload;
  if (payload?.kind !== "opportunity_accepted") {
    opportunityStatus.textContent = `档案创建失败：${payload?.rejection?.kind ?? "未知错误"}`;
    return;
  }
  renderOpportunityProfileTerminal(payload.terminal);
}

async function viewOpportunityProfile() {
  if (!opportunityProfileId) {
    opportunityStatus.textContent = "没有可读取的 profile ID hint。";
    return;
  }
  opportunityView.disabled = true;
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
    opportunityView.disabled = !opportunityProfileId;
  }
}

async function generateOpportunityPlan() {
  if (!opportunityProfileId) {
    opportunityStatus.textContent = "请先创建或恢复 private profile。";
    return;
  }
  opportunityPlan.disabled = true;
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
    opportunityPlan.disabled = !opportunityProfileId;
  }
}

async function deleteOpportunityProfile() {
  if (!opportunityProfileId) {
    opportunityStatus.textContent = "没有可删除的 private profile。";
    return;
  }
  const envelope = reusableOperationEnvelope("delete", opportunityProfileId)
    ?? mintOperationEnvelope("delete", opportunityProfileId);
  storePendingOperation(envelope);
  opportunityDelete.disabled = true;
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
    opportunityDelete.disabled = !opportunityProfileId;
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
  opportunityStatus.textContent = "删除收据已持久化；tombstone 不含 completed courses 或 preference weights。下方 Synthetic profile 只是可再次 consent 的公开 demo 模板，不是已删除档案。";
}

publicationConfirm.addEventListener("change", () => {
  publicationPublish.disabled = !publicationConfirm.checked;
});
publicationRefresh.addEventListener("click", () => {
  void loadPublicationStatus();
});
publicationPublish.addEventListener("click", () => {
  void publishAffairsDemo();
});

radarPublicationConfirm.addEventListener("change", () => {
  radarPublicationPublish.disabled = !radarPublicationConfirm.checked;
});
radarPublicationRefresh.addEventListener("click", () => {
  void loadChangePublicationStatus();
});
radarPublicationPublish.addEventListener("click", () => {
  void publishChangeDemo();
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
syncProcedurePreview();
setOpportunityHint(readOpportunityHint());
if (opportunityProfileId) {
  void viewOpportunityProfile();
}

void lookup();
void loadPublicationStatus();
void loadChangePublicationStatus();
void loadChangeFeed();
