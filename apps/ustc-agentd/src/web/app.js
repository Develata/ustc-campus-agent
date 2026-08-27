const form = document.querySelector("#lookup-form");
const procedureInput = document.querySelector("#procedure-id");
const procedurePreview = document.querySelector("#procedure-id-preview");
const submitButton = document.querySelector("#lookup-button");
const status = document.querySelector("#status");
const result = document.querySelector("#result");
const errorPanel = document.querySelector("#error-panel");
const errorMessage = document.querySelector("#error-message");

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
  status.textContent = "正在通过 M00 → M10 → M71 读取可验证结果…";
  errorPanel.hidden = true;

  try {
    const response = await fetch(`/api/v1/affairs/${encodeURIComponent(procedureId)}`, {
      method: "GET",
      headers: { "Accept": "application/json" },
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

void lookup();
