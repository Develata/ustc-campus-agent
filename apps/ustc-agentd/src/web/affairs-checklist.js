/* UE-02: page-local projection of the public Affairs DTO, never Chat prose.
 * Integration: invalidate() at lookup start/error; render(payload, token) only
 * after renderFound succeeds. Pass the token returned at lookup start to reject
 * late responses. Loaded after app.js; no network, storage or domain mutation.
 */
;(() => {
  "use strict";
  const notice = "个人办理清单：勾选仅代表个人进度，不是官方受理、批准或完成凭证。仅本页暂存，重新查询或刷新即清空；不上传、不跨设备同步。";
  const list = value => Array.isArray(value) ? value : [];
  const textValue = value => value == null ? "未提供" : String(value);
  // Flatten line breaks and escape all Markdown punctuation, including HTML and
  // entity delimiters. Untrusted text cannot open blocks, links or raw HTML.
  const escapeMarkdown = value => textValue(value)
    .replace(/[\u0000-\u001f\u007f\u2028\u2029]/g, " ")
    .replace(/[\\`*_{}\[\]()<>#+\-.!|~&:=]/g, "\\$&");
  function safeUrl(value) {
    if (typeof value !== "string" || !/^https?:\/\//i.test(value) || /[\u0000-\u0020\u007f]/.test(value)) return null;
    try {
      const url = new URL(value);
      if (!["http:", "https:"].includes(url.protocol) || url.username || url.password) return null;
      return url.href.replace(/[<>\\`"()\[\]]/g, char => `%${char.charCodeAt(0).toString(16).toUpperCase()}`);
    } catch (_) { return null; }
  }
  function timestamp(value) {
    if (!Number.isFinite(value)) return "未提供";
    const date = new Date(value);
    return Number.isNaN(date.getTime()) ? "无效时间" : date.toISOString();
  }
  function detail(value) {
    return value == null ? "未提供" : typeof value === "object" ? JSON.stringify(value) : String(value);
  }
  function usable(payload) {
    const terminal = payload?.terminal;
    const view = terminal?.outcome?.view;
    return payload?.kind === "available" && payload.redaction === "public"
      && terminal?.outcome?.kind === "found" && terminal.lineage?.kind === "verified"
      && typeof view?.procedure_id === "string" && typeof view.title === "string"
      && Array.isArray(view.prerequisites) && view.prerequisites.every(item => typeof item?.condition === "string")
      && Array.isArray(view.ordered_steps) && view.ordered_steps.length > 0
      && view.ordered_steps.every(item => Number.isInteger(item?.ordinal) && item.ordinal >= 0 && typeof item.instruction === "string")
      && view.evidence && Array.isArray(view.evidence.assessments);
  }
  function formatMarkdown(payload, checked = { prerequisites: [], steps: [] }) {
    if (!usable(payload)) return null;
    const { outcome, lineage } = payload.terminal;
    const view = outcome.view;
    const evidence = view.evidence;
    const lines = [`# ${escapeMarkdown(view.title)} — 个人办理清单`, "", notice, "",
      "来源范围：当前成功查询的公开结构化 Affairs 结果；source-grounded published fixture，不代表实时官方服务。", ""];
    const row = (label, value) => lines.push(`- ${label}：${escapeMarkdown(value)}`);
    row("流程 ID", view.procedure_id);
    row("流程制品 ID", view.artifact_id);
    row("适用人群", list(view.audience_tags).join("、") || "未提供");
    row("查询基准时间（UTC）", timestamp(outcome.as_of));
    lines.push("", "## 办理条件", "");
    if (!view.prerequisites.length) lines.push("当前来源快照没有列出额外前置条件（不等于无条件）。");
    view.prerequisites.forEach((item, index) => {
      lines.push(`- [${checked.prerequisites?.[index] === true ? "x" : " "}] ${escapeMarkdown(item.condition)}`);
      if (item.source_subject != null) lines.push(`  - 来源主题：${escapeMarkdown(item.source_subject)}`);
    });
    lines.push("", "## 按顺序办理", "");
    view.ordered_steps.forEach((step, index) => lines.push(
      `- [${checked.steps?.[index] === true ? "x" : " "}] ${step.ordinal + 1}. ${escapeMarkdown(step.instruction)}`));
    lines.push("", "## 官方入口（来源声明，办理前请核对）", "");
    if (!list(view.entry_points).length) lines.push("来源未提供入口。");
    for (const entry of list(view.entry_points)) {
      const url = safeUrl(entry.url);
      lines.push(url ? `- [${escapeMarkdown(entry.label)}](<${url}>)`
        : `- ${escapeMarkdown(entry.label)}：${entry.url == null ? "未提供 URL" : "URL 未作为链接导出（仅接受无凭据的绝对 http(s) 地址）"}`);
      if (entry.contact_ref != null) row("联系引用", entry.contact_ref);
    }
    lines.push("", "## 官方联系", "");
    if (!list(view.contacts).length) lines.push("来源未提供联系方式。");
    for (const contact of list(view.contacts)) row("联系", `${textValue(contact.name)}；${textValue(contact.channel)}；来源 ${textValue(contact.source_id)}；引用 ${textValue(contact.contact_ref)}`);
    lines.push("", "## 时间与有效性", "");
    row("生效区间（原始毫秒时间戳）", detail(view.effective_interval));
    row("来源有效期（原始毫秒时间戳）", detail(evidence.valid_interval));
    if (!list(view.deadlines).length) row("截止日期", "来源未声明固定截止日期");
    for (const deadline of list(view.deadlines)) row("截止事项", `${textValue(deadline.label)}；${textValue(deadline.kind)}；${timestamp(deadline.at)}`);
    lines.push("", "## 来源、复核与不确定性", "");
    row("最近核验（UTC）", timestamp(evidence.last_verified_at));
    row("观察时间（UTC）", timestamp(evidence.observed_at));
    row("获知时间（UTC）", timestamp(evidence.known_at));
    row("复核时间（UTC）", timestamp(evidence.reviewed_at));
    row("新鲜度（服务端判定，非导出时重新核验）", detail(outcome.freshness));
    row("冲突状态与详情", detail(view.conflict_state));
    row("不确定性", view.uncertainty_state);
    row("查询路径", view.lookup_path);
    row("投影完整性", detail(evidence.projection));
    row("板块 ID", view.board_id);
    row("板块策略版本", view.board_policy_version);
    row("证据链（不是个人办理收据）", detail(lineage));
    if (!evidence.assessments.length) row("来源评估", "未提供");
    for (const assessment of evidence.assessments) {
      row("来源 ID", assessment.source_id);
      row("来源主题", assessment.subject);
      row("来源权威声明", assessment.authority);
      row("来源复核（UTC）", timestamp(assessment.reviewed_at));
      row("来源最近核验（UTC）", timestamp(assessment.last_verified_at));
    }
    return `${lines.join("\n")}\n`;
  }

  let generation = 0;
  let current = null;
  let panel = null;
  let feedback, fallback, copyButton, downloadButton;
  let inputs = [];
  function element(tag, text, className) {
    const node = document.createElement(tag);
    if (text != null) node.textContent = text;
    if (className) node.className = className;
    return node;
  }
  function markdown() {
    return current ? formatMarkdown(current.payload, current.checked) : null;
  }
  function invalidate() {
    generation += 1;
    current = null;
    for (const input of inputs) { input.checked = false; input.disabled = true; }
    if (panel) {
      copyButton.disabled = true;
      downloadButton.disabled = true;
      feedback.textContent = "当前没有可导出的成功结果，请重新查询。";
      fallback.value = "";
      fallback.hidden = true;
    }
    return generation;
  }
  function mount(result) {
    if (panel) return;
    panel = element("section", null, "affairs-checklist");
    panel.id = "affairs-checklist";
    panel.setAttribute("aria-labelledby", "affairs-checklist-title");
    const heading = element("h3", "个人办理清单");
    heading.id = "affairs-checklist-title";
    const description = element("p", notice);
    description.id = "affairs-checklist-notice";
    const actions = element("div", null, "affairs-checklist-actions");
    copyButton = element("button", "复制 Markdown");
    copyButton.id = "affairs-checklist-copy";
    downloadButton = element("button", "下载 Markdown");
    downloadButton.id = "affairs-checklist-download";
    for (const button of [copyButton, downloadButton]) button.type = "button";
    feedback = element("p");
    feedback.id = "affairs-checklist-status";
    feedback.setAttribute("role", "status");
    feedback.setAttribute("aria-live", "polite");
    fallback = element("textarea");
    fallback.id = "affairs-checklist-fallback";
    fallback.readOnly = true;
    fallback.hidden = true;
    fallback.rows = 8;
    fallback.setAttribute("aria-label", "个人清单 Markdown：可全选手动复制");
    actions.append(copyButton, downloadButton);
    panel.append(heading, description, actions, feedback, fallback);
    result.insertBefore(panel, result.querySelector(".result-grid"));
    copyButton.addEventListener("click", async () => {
      const snapshot = current;
      const content = markdown();
      if (!snapshot || content == null) return;
      copyButton.disabled = true;
      try {
        if (!window.navigator?.clipboard?.writeText) throw new Error("clipboard unavailable");
        await window.navigator.clipboard.writeText(content);
        if (current !== snapshot) return;
        feedback.textContent = "已复制个人清单 Markdown（点击复制时的勾选快照）。";
        fallback.hidden = true;
      } catch (_) {
        if (current !== snapshot) return;
        feedback.textContent = "剪贴板不可用或复制失败。请在下方全选手动复制，或使用下载 Markdown。";
        fallback.value = markdown();
        fallback.hidden = false;
        fallback.focus();
        fallback.select();
      } finally {
        if (current === snapshot) copyButton.disabled = false;
      }
    });
    downloadButton.addEventListener("click", () => {
      const content = markdown();
      if (content == null) return;
      let url;
      let anchor;
      try {
        url = URL.createObjectURL(new Blob([content], { type: "text/markdown;charset=utf-8" }));
        anchor = element("a");
        anchor.href = url;
        anchor.download = "affairs-personal-checklist.md";
        anchor.hidden = true;
        document.body.append(anchor);
        anchor.click();
        feedback.textContent = "已请求浏览器下载个人清单；文件在本地生成，未上传。";
      } catch (_) {
        feedback.textContent = "无法启动下载，请使用复制 Markdown 或下方文本手动保存。";
        fallback.value = content;
        fallback.hidden = false;
      } finally {
        anchor?.remove();
        if (url) window.setTimeout(() => URL.revokeObjectURL(url), 1000);
      }
    });
  }
  function render(payload, token = generation) {
    if (token !== generation) return false;
    if (!usable(payload)) { invalidate(); return false; }
    const result = document.querySelector("#result");
    const prerequisites = document.querySelector("#prerequisites");
    const steps = document.querySelector("#steps");
    if (!result || result.hidden || !prerequisites || !steps) { invalidate(); return false; }
    invalidate();
    mount(result);
    current = { payload: JSON.parse(JSON.stringify(payload)), checked: { prerequisites: [], steps: [] } };
    const snapshot = current;
    inputs = [];
    const view = current.payload.terminal.outcome.view;
    function checkItems(container, values, key, field) {
      container.replaceChildren();
      values.forEach((value, index) => {
        const item = element("li");
        if (key === "steps") item.append(element("span", String(value.ordinal + 1).padStart(2, "0"), "step-index"));
        const label = element("label", null, "affairs-check-item");
        const input = element("input");
        input.type = "checkbox";
        input.setAttribute("aria-describedby", "affairs-checklist-notice");
        input.addEventListener("change", () => {
          if (current !== snapshot) return;
          current.checked[key][index] = input.checked;
          feedback.textContent = "已更新个人勾选；未提交给任何官方系统。";
          if (!fallback.hidden) fallback.value = markdown();
        });
        inputs.push(input);
        label.append(input, element("span", value[field]));
        item.append(label);
        container.append(item);
      });
    }
    checkItems(prerequisites, view.prerequisites, "prerequisites", "condition");
    if (!view.prerequisites.length) prerequisites.append(element("li", "当前来源快照没有列出额外前置条件。"));
    checkItems(steps, view.ordered_steps, "steps", "instruction");
    copyButton.disabled = false;
    downloadButton.disabled = false;
    feedback.textContent = "可勾选下方真实办理条件和步骤，再复制或下载当前清单。";
    return true;
  }
  window.UcaAffairsChecklist = Object.freeze({ render, invalidate, markdown, formatMarkdown, escapeMarkdown, safeUrl });
})();
