/* UE-01: page-local synthetic draft only. No requests, storage or planner authority.
 * Fixture projection is checked by scripts/tests/test_course_editor.mjs.
 * Integration: readDraft() ONLY for a new create; persist exact serialized request
 * before POST. setPending(true, savedDraft) locks/replays the pending draft;
 * setPending(false) only after a confirmed terminal. Never rebuild retry bytes.
 */
;(function (root) {
  "use strict";

  const catalog = Object.freeze([
    ["MATH2001", "Real Analysis I", 4],
    ["MATH2002", "Abstract Algebra I", 4],
    ["MATH2003", "Probability Theory", 4],
    ["MATH2004", "Numerical Analysis", 3],
    ["MATH2005", "Topology", 3],
    ["MATH2006", "Mathematical Statistics", 4],
    ["CS2001", "Data Structures", 4],
    ["CS2002", "Algorithms", 4],
    ["CS2003", "Database Systems", 3],
    ["CS2004", "Operating Systems", 4],
    ["CS2005", "Computer Networks", 3],
    ["CS2006", "Rust Systems Lab", 3],
    ["PHYS2001", "Classical Mechanics", 4],
    ["PHYS2002", "Electromagnetism", 4],
    ["PHYS2003", "Statistical Physics", 4],
    ["HUM2001", "Philosophy of Science", 2],
    ["HUM2002", "Academic Writing", 2],
    ["LANG2001", "Advanced English", 2],
    ["PE2001", "Swimming", 1, "unresolved_alias"],
    ["GEN2001", "Innovation Seminar", 2]
  ].map(([code, title, credits, identity_status = "verified"]) => Object.freeze({
    code, title, credits, identity_status
  })));
  const prerequisiteCodes = Object.freeze(["MATH1001", "MATH1002", "CS1001", "PHYS1001"]);
  const selectable = catalog.filter((course) => course.identity_status === "verified");
  const completedCodes = Object.freeze([...prerequisiteCodes, ...selectable.map((course) => course.code)]);
  const preferenceCodes = new Set(selectable.map((course) => course.code));
  const completedCodeSet = new Set(completedCodes);
  const defaultWeights = Object.freeze({ MATH2001: 9, MATH2003: 8, CS2006: 7,
    PHYS2003: 5, HUM2001: 4, GEN2001: 3, LANG2001: 2 });
  const limits = Object.freeze({ minCredits: 0, maxCredits: 65535, minWeight: -100, maxWeight: 100 });

  function defaultDraft() {
    return { completed_courses: [...prerequisiteCodes], min_credits: 9, max_credits: 12,
      preference_weights: Object.entries(defaultWeights).map(([course_code, weight]) => ({ course_code, weight })) };
  }

  function fail(field, message) {
    const error = new Error(message);
    error.code = "invalid_course_draft";
    error.field = field;
    throw error;
  }

  function integer(value, min, max, field) {
    // Reject blanks, fractions, exponent strings, signs-only and coercible objects.
    if ((typeof value !== "number" && typeof value !== "string")
      || (typeof value === "string" && !/^-?\d+$/.test(value))) {
      fail(field, "请输入范围内的整数，不接受空值、小数或指数写法。");
    }
    const result = Number(value);
    if (!Number.isSafeInteger(result) || result < min || result > max) {
      fail(field, `请输入 ${min}–${max} 范围内的整数。`);
    }
    return result;
  }

  function validateDraft(draft) {
    if (!draft || typeof draft !== "object" || Array.isArray(draft)) fail("draft", "课程草稿格式无效。");
    if (!Array.isArray(draft.completed_courses)) fail("completed_courses", "请选择 synthetic 已修课程。");
    const seenCompleted = new Set();
    for (const code of draft.completed_courses) {
      if (!completedCodeSet.has(code) || seenCompleted.has(code)) {
        fail("completed_courses", "已修课程含不支持或重复的 synthetic 代码。");
      }
      seenCompleted.add(code);
    }
    const min = integer(draft.min_credits, limits.minCredits, limits.maxCredits, "min_credits");
    const max = integer(draft.max_credits, 1, limits.maxCredits, "max_credits");
    if (min > max) fail("max_credits", "最低学分不能高于最高学分。");
    if (!Array.isArray(draft.preference_weights)) fail("preference_weights", "偏好字段格式无效。");
    const seenWeights = new Set();
    const weights = draft.preference_weights.map((entry) => {
      if (!entry || !preferenceCodes.has(entry.course_code) || seenWeights.has(entry.course_code)) {
        fail("preference_weights", "偏好含不支持、身份未解析或重复的 synthetic 课程代码。");
      }
      seenWeights.add(entry.course_code);
      return { course_code: entry.course_code,
        weight: integer(entry.weight, limits.minWeight, limits.maxWeight, `weight:${entry.course_code}`) };
    });
    return { completed_courses: [...draft.completed_courses], min_credits: min,
      max_credits: max, preference_weights: weights };
  }

  function courseLabel(code) {
    const course = catalog.find((item) => item.code === code);
    return course ? `${course.code} · ${course.title}` : code;
  }

  let ui = null;
  let pending = false;
  function node(tag, content, className) {
    const element = root.document.createElement(tag);
    if (content != null) element.textContent = content;
    if (className) element.className = className;
    return element;
  }

  function rawDraft() {
    return { completed_courses: ui.completed.filter((input) => input.checked).map((input) => input.value),
      min_credits: ui.min.value, max_credits: ui.max.value,
      preference_weights: ui.weights.filter((input) => input.value !== "0")
        .map((input) => ({ course_code: input.dataset.courseCode, weight: input.value })) };
  }

  function showValidation(focus) {
    for (const input of ui.section.querySelectorAll("input")) input.removeAttribute("aria-invalid");
    try {
      const draft = validateDraft(rawDraft());
      ui.error.hidden = true;
      ui.summary.textContent = `未保存草稿 · ${draft.completed_courses.length} 门已修 · ${draft.min_credits}–${draft.max_credits} 学分 · ${draft.preference_weights.length} 项非零偏好`;
      return draft;
    } catch (error) {
      ui.error.hidden = false;
      ui.error.textContent = error.message;
      ui.summary.textContent = "未保存草稿 · 请修正输入后重新确认 consent。";
      const input = error.field === "min_credits" ? ui.min : error.field === "max_credits" ? ui.max
        : ui.weights.find((item) => `weight:${item.dataset.courseCode}` === error.field);
      if (input) {
        input.setAttribute("aria-invalid", "true");
        if (focus) {
          const disclosure = input.closest("details");
          if (disclosure) disclosure.open = true;
          input.focus();
        }
      }
      if (focus) throw error;
      return null;
    }
  }

  function revokeDraftConsent() {
    const consent = root.document.querySelector("#opportunity-consent");
    if (consent) {
      consent.checked = false;
      consent.dispatchEvent(new root.Event("change", { bubbles: true }));
    }
    const chatConfirmation = root.document.querySelector("#chat-opportunity-confirm");
    if (chatConfirmation) {
      chatConfirmation.checked = false;
      chatConfirmation.dispatchEvent(new root.Event("change", { bubbles: true }));
    }
    // A draft edit never changes the current server-owned profile or its hint.
    ui.section.dispatchEvent(new root.CustomEvent("uca:course-draft-change", { bubbles: true }));
  }

  function applyDraft(draft) {
    const weights = new Map(draft.preference_weights.map((item) => [item.course_code, item.weight]));
    for (const input of ui.completed) input.checked = draft.completed_courses.includes(input.value);
    for (const input of ui.weights) input.value = String(weights.get(input.dataset.courseCode) ?? 0);
    ui.min.value = String(draft.min_credits);
    ui.max.value = String(draft.max_credits);
    showValidation(false);
  }

  function readDraft() {
    if (!ui) throw new Error("课程编辑器尚未挂载；不能创建默认替代档案。");
    if (pending) throw new Error("待确认请求已锁定；请重试保存的原始请求，不能重新读取草稿。");
    return showValidation(true);
  }

  function setPending(value, savedDraft) {
    // Validate before changing state. This hook does not own or serialize retries.
    const draft = savedDraft == null ? null : validateDraft(savedDraft);
    pending = Boolean(value);
    if (!ui) return;
    if (draft) applyDraft(draft);
    ui.fields.disabled = pending;
    ui.pending.hidden = !pending;
    ui.section.dataset.pending = String(pending);
  }

  function mount() {
    if (ui || !root.document) return Boolean(ui);
    const consent = root.document.querySelector("#opportunity-consent");
    const host = consent?.closest(".opportunity-consent");
    if (!host) return false;
    const section = node("section", null, "course-editor");
    section.id = "course-editor";
    section.tabIndex = -1;
    section.setAttribute("aria-labelledby", "course-editor-title");
    const title = node("h3", "配置 synthetic 课程草稿");
    title.id = "course-editor-title";
    const notice = node("p", "仅使用 checked-in synthetic-course-planning-v0 目录，不是实时官方课程、成绩或选课资格。前端只检查输入；资格与排序由 Rust 决定。", "course-editor-note");
    const lifecycle = node("p", "修改草稿不会更改已保存档案或当前方案。服务端只允许一个当前档案：要使用新草稿，请先自行点击撤回同意并删除当前档案，再重新勾选同意并创建。不会自动删除；删除后旧档案不可恢复，本页草稿仍保留。", "course-editor-note");
    const pendingNote = node("p", "有创建请求尚未确认：草稿已锁定。请使用原请求重试，不能将新草稿拼接到旧请求；确认终态后才能继续编辑。", "course-editor-pending");
    pendingNote.hidden = true;
    pendingNote.setAttribute("role", "status");
    const fields = node("fieldset", null, "course-editor-fields");
    fields.appendChild(node("legend", "本地输入 · 尚未保存"));
    const bounds = node("div", null, "course-editor-bounds");
    function numeric(id, label, min, max) {
      const wrapper = node("label", null, "course-editor-number");
      wrapper.htmlFor = id;
      const input = node("input");
      input.id = id;
      input.type = "number";
      input.min = String(min);
      input.max = String(max);
      input.step = "1";
      input.required = true;
      input.setAttribute("aria-describedby", "course-editor-error");
      wrapper.append(node("span", label), input);
      return { wrapper, input };
    }
    const min = numeric("course-min-credits", "最低学分", 0, limits.maxCredits);
    const max = numeric("course-max-credits", "最高学分", 1, limits.maxCredits);
    bounds.append(min.wrapper, max.wrapper);
    fields.appendChild(bounds);
    const completed = node("details", null, "course-editor-disclosure");
    completed.appendChild(node("summary", "已修课程 · 只声明 synthetic 条件"));
    completed.appendChild(node("p", "基础先修代码在 fixture 中没有课程名称，不补造名称；下列名称逐字来自 synthetic 目录。", "course-editor-note"));
    const choices = node("div", null, "course-editor-choices");
    const completedInputs = completedCodes.map((code) => {
      const label = node("label", null, "course-editor-choice");
      const input = node("input");
      input.type = "checkbox";
      input.value = code;
      input.id = `course-completed-${code}`;
      label.htmlFor = input.id;
      label.append(input, node("span", courseLabel(code)));
      choices.appendChild(label);
      return input;
    });
    completed.appendChild(choices);
    const preferences = node("details", null, "course-editor-disclosure");
    preferences.appendChild(node("summary", "课程偏好 · 仅影响 soft ranking"));
    preferences.appendChild(node("p", "演示编辑范围 −100 至 100 的整数；0 为中性，负数为较不偏好。偏好不会解除先修、冲突或培养要求。PE2001 · Swimming 身份未解析，保留服务器资格判定，不开放编辑。", "course-editor-note"));
    const weightList = node("div", null, "course-editor-weights");
    const weights = selectable.map((course) => {
      const field = numeric(`course-weight-${course.code}`, `${courseLabel(course.code)} · ${course.credits} 学分`, limits.minWeight, limits.maxWeight);
      field.input.dataset.courseCode = course.code;
      weightList.appendChild(field.wrapper);
      return field.input;
    });
    preferences.appendChild(weightList);
    const reset = node("button", "恢复 synthetic 默认草稿", "course-editor-reset");
    reset.type = "button";
    reset.addEventListener("click", () => {
      if (pending) return;
      applyDraft(defaultDraft());
      revokeDraftConsent();
    });
    fields.append(completed, preferences, reset);
    const error = node("p", null, "course-editor-error");
    error.id = "course-editor-error";
    error.setAttribute("role", "alert");
    error.hidden = true;
    const summary = node("p", null, "course-editor-summary");
    summary.setAttribute("aria-live", "polite");
    section.append(title, notice, lifecycle, pendingNote, fields, error, summary);
    ui = { section, fields, min: min.input, max: max.input, completed: completedInputs,
      weights, error, summary, pending: pendingNote };
    // Replace only the old fixed-template prose, never consent/actions/results.
    const oldSummary = host.firstElementChild;
    if (oldSummary?.tagName === "DIV" && !oldSummary.querySelector("button, input")) oldSummary.replaceWith(section);
    else host.prepend(section);
    fields.addEventListener("input", () => {
      revokeDraftConsent();
      showValidation(false);
    });
    applyDraft(defaultDraft());
    setPending(pending);
    const candidates = root.document.querySelector("#opportunity-candidates");
    if (candidates) {
      candidates.classList.add("opportunity-candidates");
      candidates.setAttribute("aria-label", "服务器生成的候选计划比较，按服务器返回顺序");
    }
    return true;
  }

  root.UcaCourseEditor = Object.freeze({ catalog, completedCodes, limits, defaultDraft,
    validateDraft, courseLabel, mount, readDraft, setPending, setLocked: setPending });
  if (root.document) {
    mount();
    if (!ui && root.document.readyState === "loading") root.document.addEventListener("DOMContentLoaded", mount, { once: true });
  }
})(typeof window === "undefined" ? globalThis : window);
