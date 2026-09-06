// Account identity is server-owned. Install before the other clients so requests
// wait for initial admission and stale tabs bind their old subject explicitly.
window.UcaAccount = (() => {
  "use strict";
  const rawFetch = window.fetch.bind(window);
  let subject = null, enabled = false;
  const channel = typeof BroadcastChannel === "function" ? new BroadcastChannel("uca-account-change") : null;
  channel?.addEventListener("message", () => location.reload());
  const pane = document.createElement("section"); pane.className = "account-panel";
  const badge = document.createElement("div"); badge.className = "account-badge";
  const form = document.createElement("form"); form.className = "account-login";
  const heading = document.createElement("h1"); heading.textContent = "登录校园 Agent";
  const description = document.createElement("p"); description.textContent = "使用管理员配置的账号。学校 SSO 尚未接入。";
  const loginLabel = document.createElement("label"); loginLabel.textContent = "登录名";
  const loginInput = document.createElement("input");
  loginInput.name = "login_name"; loginInput.autocomplete = "username";
  loginInput.minLength = 3; loginInput.maxLength = 64; loginInput.required = true;
  loginInput.pattern = "[a-z0-9][a-z0-9._-]{1,62}[a-z0-9]";
  loginLabel.append(loginInput);
  const passwordLabel = document.createElement("label"); passwordLabel.textContent = "密码";
  const passwordInput = document.createElement("input");
  passwordInput.name = "password"; passwordInput.type = "password";
  passwordInput.autocomplete = "current-password"; passwordInput.minLength = 12;
  passwordInput.maxLength = 256; passwordInput.required = true;
  passwordLabel.append(passwordInput);
  const submit = document.createElement("button"); submit.type = "submit"; submit.textContent = "登录";
  const status = document.createElement("p"); status.className = "account-status"; status.setAttribute("role", "status");
  form.append(heading, description, loginLabel, passwordLabel, submit, status);
  pane.append(form);
  pane.hidden = true;
  document.body.append(pane, badge);
  function clearPending() {
    try {
      sessionStorage.removeItem("uca.plugin-management.pending.v1");
      for (const key of ["ustc-campus-agent/opportunity-profile-id/v1", "ustc-campus-agent/opportunity-pending-create/v1", "ustc-campus-agent/opportunity-pending-delete/v1"]) localStorage.removeItem(key);
    } catch (_) { /* Private server state stays authoritative. */ }
  }
  function requireLogin() {
    pane.hidden = false;
    for (const child of document.body.children) if (child !== pane && child !== badge && child instanceof HTMLElement) child.inert = true;
    form.elements.login_name.focus();
  }
  function changed() { clearPending(); channel?.postMessage("changed"); location.reload(); }
  async function request(endpoint, body) {
    const headers = { "Content-Type": "application/json" };
    // rawFetch bypasses the general client wrapper. Bind logout to this page's
    // admitted subject so a delayed old tab cannot revoke another account's cookie.
    if (endpoint === "logout") {
      if (!subject) throw new Error("unauthenticated");
      headers["X-UCA-Account-Subject"] = JSON.stringify([subject.tenant_id, subject.user_id]);
    }
    const response = await rawFetch(`/api/v1/account/${endpoint}`, { method: body ? "POST" : "GET", credentials: "same-origin", cache: "no-store", headers, ...(body ? {body: JSON.stringify(body)} : {}) });
    const value = await response.json();
    if (!response.ok) throw new Error(value.error || "unavailable");
    return value;
  }
  form.addEventListener("submit", async event => {
    event.preventDefault();
    const button = form.querySelector("button"); button.disabled = true; status.textContent = "正在登录…";
    try { await request("login", { schema: "platform-account-login/v1", login_name: form.elements.login_name.value, password: form.elements.password.value }); form.elements.password.value = ""; changed(); }
    catch (error) { form.elements.password.value = ""; status.textContent = error.message === "rate_limited" ? "尝试次数过多，请 15 分钟后重试。" : error.message === "authentication_failed" ? "登录名或密码不正确，或账号暂不可用。" : "登录暂不可用，请联系管理员。"; }
    finally { button.disabled = false; }
  });
  const ready = (async () => {
    try {
      const mode = await request("mode"); enabled = mode.mode === "local-accounts";
      if (!enabled) return;
      const value = await request("me"); subject = value.account;
      const stamp = JSON.stringify([subject.tenant_id, subject.user_id]);
      try {
        const previous = sessionStorage.getItem("uca.account.subject.v1");
        sessionStorage.setItem("uca.account.subject.v1", stamp);
        if (previous !== stamp) {
          // Other clients may already have read an old pending envelope into memory.
          // Do not release their queued fetches until a clean document is loaded.
          clearPending(); location.reload(); await new Promise(() => {});
        }
      } catch (_) { clearPending(); throw new Error("private_storage_unavailable"); }
      const name = document.createElement("span"); name.textContent = subject.login_name;
      const logout = document.createElement("button"); logout.textContent = "退出登录"; logout.type = "button";
      logout.addEventListener("click", async () => { logout.disabled = true; try { await request("logout", {schema:"platform-account-logout/v1"}); changed(); } catch (error) { if (error.message === "unauthenticated") { changed(); return; } logout.disabled = false; logout.textContent = "退出失败，重试"; } });
      badge.append(name, logout);
    } catch (_) { enabled = true; clearPending(); requireLogin(); }
  })();
  window.fetch = async (input, options = {}) => {
    await ready;
    const url = new URL(typeof input === "string" ? input : input.url, location.href);
    if (enabled && url.origin === location.origin && url.pathname.startsWith("/api/")) {
      const headers = new Headers(options.headers || (input instanceof Request ? input.headers : undefined));
      if (subject) headers.set("X-UCA-Account-Subject", JSON.stringify([subject.tenant_id, subject.user_id]));
      options = {...options, headers};
    }
    const response = await rawFetch(input, options);
    if (enabled && response.status === 401 && subject) { clearPending(); location.reload(); }
    return response;
  };
  return {ready};
})();
