// View state only. Navigation never sends a request, changes consent or owns product state.
window.UcaShell = (() => {
  "use strict";
  const views = new Map([...document.querySelectorAll("[data-view]")].map(el => [el.dataset.view, el]));
  const labels = { chat: "新对话", plugins: "插件", "plugins/affairs": "办事导航",
    "plugins/radar": "变更雷达", "plugins/planning": "课程规划", "plugins/calendar": "简单日历", "plugins/manage": "MCP 与 Skills", settings: "设置" };
  const sidebar = document.querySelector("#app-sidebar");
  const column = document.querySelector("#app-column");
  const toggle = document.querySelector("#nav-toggle");
  const scrim = document.querySelector("#nav-scrim");
  const mobile = matchMedia("(max-width: 760px)");
  const options = document.querySelector("#chat-options");
  const optionsToggle = document.querySelector("#chat-options-toggle");
  const scroll = document.querySelector("#chat-scroll");
  let current = "chat";
  let drawer = false;

  function setDrawer(open, restore = true) {
    drawer = mobile.matches && open;
    document.body.classList.toggle("nav-open", drawer);
    sidebar.inert = mobile.matches && !drawer;
    column.inert = drawer;
    scrim.hidden = !drawer;
    toggle.setAttribute("aria-expanded", String(mobile.matches ? drawer : !document.body.classList.contains("nav-collapsed")));
    if (drawer) {
      sidebar.setAttribute("role", "dialog");
      sidebar.setAttribute("aria-modal", "true");
      document.querySelector("#nav-close").focus();
    } else {
      sidebar.removeAttribute("role");
      sidebar.removeAttribute("aria-modal");
      if (restore && mobile.matches) toggle.focus();
    }
  }
  function showOptions(open) {
    options.hidden = !open;
    optionsToggle.setAttribute("aria-expanded", String(open));
  }
  function syncChat() {
    const hasMessages = Boolean(chatMessages.querySelector(".chat-message"));
    document.querySelector("#chat-view").classList.toggle("is-empty", !hasMessages);
    document.querySelector("#nav-chat").hidden = !hasMessages;
    document.querySelector("#view-title").textContent = current === "chat" && hasMessages ? "对话" : labels[current];
    document.querySelector("#options-indicator").hidden = !chatPromptCustomization.value.trim() && !chatOpportunityConfirm.checked;
    chatInput.style.height = "auto";
    chatInput.style.height = `${Math.min(chatInput.scrollHeight, 180)}px`;
  }
  function renderView(focus = true) {
    const requested = location.hash.slice(1) || "chat";
    current = views.has(requested) ? requested : "chat";
    for (const [key, view] of views) view.hidden = key !== current;
    document.querySelector("#return-chat").hidden = current === "chat";
    for (const [id, selected] of [["chat-clear", current === "chat"], ["nav-chat", current === "chat"],
      ["nav-plugins", current.startsWith("plugins")], ["nav-settings", current === "settings"]]) {
      const el = document.getElementById(id);
      if (selected) el.setAttribute("aria-current", "page"); else el.removeAttribute("aria-current");
    }
    setDrawer(false, false);
    syncChat();
    if (focus) {
      const target = current === "chat" ? chatInput : views.get(current).querySelector("h1,h2");
      if (target) { if (target !== chatInput) target.tabIndex = -1; target.focus({ preventScroll: true }); }
      document.querySelector("#app-main").scrollTop = 0;
    }
  }
  function navigate(route) {
    if (!views.has(route)) return;
    if (location.hash === `#${route}`) renderView(); else location.hash = route;
  }
  function hint(message) {
    const el = document.querySelector("#scene-hint");
    el.textContent = message;
    el.hidden = !message;
  }

  toggle.addEventListener("click", () => {
    if (mobile.matches) setDrawer(!drawer);
    else { document.body.classList.toggle("nav-collapsed"); setDrawer(false, false); }
  });
  document.querySelector("#nav-close").addEventListener("click", () => setDrawer(false));
  scrim.addEventListener("click", () => setDrawer(false));
  mobile.addEventListener("change", () => setDrawer(false, false));
  document.addEventListener("keydown", event => {
    if (!drawer) return;
    if (event.key === "Escape") { event.preventDefault(); setDrawer(false); }
    if (event.key === "Tab") {
      const stops = [...sidebar.querySelectorAll("button:not(:disabled), a[href]")].filter(el => el.getClientRects().length);
      const first = stops[0], last = stops.at(-1);
      if (event.shiftKey && document.activeElement === first) { event.preventDefault(); last.focus(); }
      else if (!event.shiftKey && document.activeElement === last) { event.preventDefault(); first.focus(); }
    }
  });
  window.addEventListener("hashchange", () => renderView());
  // Re-selecting the current mobile route must still dismiss its modal drawer.
  sidebar.addEventListener("click", event => {
    const link = event.target.closest("a[href]");
    if (link && link.hash === location.hash) { event.preventDefault(); renderView(); }
  });
  chatClear.addEventListener("click", () => { if (!chatPending) { showOptions(false); hint(""); navigate("chat"); } });
  optionsToggle.addEventListener("click", () => showOptions(options.hidden));
  chatInput.addEventListener("input", syncChat);
  chatPromptCustomization.addEventListener("input", syncChat);
  chatOpportunityConfirm.addEventListener("change", syncChat);
  window.addEventListener("uca:chat-state", () => {
    syncChat();
    if (current === "chat") scroll.scrollTo({ top: scroll.scrollHeight, behavior: "instant" });
  });
  // The browser's invalid event must reveal an optional control before it can be focused.
  chatPromptCustomization.addEventListener("invalid", () => showOptions(true));

  const themeSelect = document.querySelector("#theme-select");
  const themeKey = "ustc-campus-agent/appearance/v1";
  function applyTheme(value) {
    const theme = ["light", "dark"].includes(value) ? value : "system";
    document.documentElement.dataset.theme = theme;
    themeSelect.value = theme;
  }
  try { applyTheme(localStorage.getItem(themeKey)); } catch (_) { applyTheme("system"); }
  themeSelect.addEventListener("change", () => {
    applyTheme(themeSelect.value);
    try { localStorage.setItem(themeKey, themeSelect.value); } catch (_) { /* Theme remains usable for this page. */ }
  });
  renderView(false);
  return Object.freeze({ navigate, showOptions, hint, syncChat });
})();
