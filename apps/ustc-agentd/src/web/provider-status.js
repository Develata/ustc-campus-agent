// Settings project the selected catalog entry; legacy default status is not selection authority.
window.UcaProviderStatus = (() => {
  "use strict";
  const badge = document.querySelector("#provider-status");
  const model = document.querySelector("#provider-model");
  const note = document.querySelector("#provider-connection-note");
  const tools = document.querySelector("#provider-tool-note");
  const refresh = document.querySelector("#provider-status-refresh");
  function render() {
    const current = window.UcaModelSelection?.selected;
    if (!current) {
      badge.textContent = "请选择模型"; model.textContent = "模型选择未确认";
      note.textContent = "请在输入框旁读取模型列表并明确选择，然后再发送消息。";
      tools.textContent = "尚未确认所选模型的工具能力。";
    } else {
      badge.textContent = current.provider.mode === "mock" ? "离线演示" : current.provider.mode === "local-chat" ? "本地模型" : "已配置模型";
      model.textContent = `${current.label} · ${current.provider.model}`;
      note.textContent = current.provider.mode === "mock"
        ? "所选模型使用确定性的离线演示，不会向模型服务发送请求。可选模型由服务端配置；切换只影响之后发送的消息。"
        : "当前显示所选模型的服务端配置；具体连接结果以每次回复为准。可选模型和密钥由服务端管理，切换只影响之后发送的消息。";
      tools.textContent = current.tool_calling
        ? "此模型支持 Agent 工具调用；实际执行仍由后端校验权限与参数。"
        : "此模型仅用于聊天，不提供插件工具调用。";
    }
    window.dispatchEvent(new Event("uca:provider-status"));
  }
  refresh.addEventListener("click", () => { void window.UcaModelSelection?.refresh(); });
  window.addEventListener("uca:model-selection", render);
  render();
  return Object.freeze({get toolCalling(){return window.UcaModelSelection?.toolCalling ?? null;},refresh:()=>window.UcaModelSelection?.refresh()});
})();