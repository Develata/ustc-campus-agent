// Administrator presentation only. HTTP/permission contracts remain server-owned.
// Document-lifetime mount: one root, explicit transport and publication notification.
// This module does not read Chat, profile, model or public feed implementation state.
window.UcaAdminControls = (() => {
  "use strict";
  const mountedRoots = new WeakSet();
  function mount({root, request, onChangePublished}) {
    if (!root || typeof root.querySelector !== 'function'
      || typeof request !== 'function' || typeof onChangePublished !== 'function') {
      throw new TypeError('Administrator controls require a root, transport and publication callback');
    }
    if (mountedRoots.has(root)) return;
    mountedRoots.add(root);
    const radarPublicationRefresh = root.querySelector("#radar-publication-refresh");
    const radarPublicationConfirm = root.querySelector("#radar-publication-confirm");
    const radarPublicationPublish = root.querySelector("#radar-publication-publish");
    const radarPublicationStatus = root.querySelector("#radar-publication-status");
    const publicationRefresh = root.querySelector("#publication-refresh");
    const publicationConfirm = root.querySelector("#publication-confirm");
    const publicationPublish = root.querySelector("#publication-publish");
    const publicationStatus = root.querySelector("#publication-status");

    function text(element, value) {
      element.textContent = value ?? "—";
    }

    async function requestPublication(method, body) {
      const response = await request("/api/v1/demo/administrator/affairs/publication", {
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
        throw new Error("发布状态格式无法识别");
      }
      text(root.querySelector("#publication-revision"), payload.publication_revision);
      text(root.querySelector("#publication-receipt"), payload.publication_receipt_id);
      text(root.querySelector("#publication-evidence-count"), payload.control_evidence_event_count);
      publicationStatus.textContent = `已读取保存的流程版本：${payload.publication_revision ?? "未知"}。`;
    }

    async function loadPublicationStatus() {
      publicationRefresh.disabled = true;
      publicationStatus.textContent = "正在读取已保存的发布状态…";
      try {
        renderPublicationStatus(await requestPublication("GET"));
      } catch (error) {
        publicationStatus.textContent = `状态读取失败：${error instanceof Error ? error.message : "未知错误"}`;
      } finally {
        publicationRefresh.disabled = false;
      }
    }

    async function publishAffairsDemo() {
      if (publicationConfirm.disabled) return;
      if (!publicationConfirm.checked) {
        publicationStatus.textContent = "请先勾选确认，再发布固定的办理资料。";
        return;
      }
      publicationPublish.disabled = true;
      publicationConfirm.disabled = true;
      publicationStatus.textContent = "正在核对权限并发布办理流程…";
      try {
        const payload = await requestPublication(
          "POST",
          JSON.stringify({ confirm_publish: true })
        );
        if (
          payload?.schema !== "ustc-affairs-publication-response/v1" ||
          payload?.outcome?.kind !== "published"
        ) {
          throw new Error("发布结果格式无法识别");
        }
        publicationStatus.textContent = `流程版本 ${payload.outcome.publication_revision} 已发布，正在核对保存结果…`;
        await loadPublicationStatus();
      } catch (error) {
        publicationStatus.textContent = `发布失败：${error instanceof Error ? error.message : "未知错误"}`;
      } finally {
        publicationConfirm.checked = false;
        publicationConfirm.disabled = false;
        publicationPublish.disabled = true;
      }
    }

    async function requestChangePublication(method, body) {
      const response = await request("/api/v1/demo/administrator/changes/publication", {
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
        throw new Error("校历发布状态格式无法识别");
      }
      text(root.querySelector("#radar-publication-review-count"), payload.review_count);
      text(root.querySelector("#radar-publication-count"), payload.publication_count);
      text(
        root.querySelector("#radar-publication-receipt"),
        payload.publication_receipt_id ?? "尚未发布"
      );
      text(
        root.querySelector("#radar-publication-evidence-count"),
        payload.control_evidence_event_count
      );
      radarPublicationStatus.textContent = payload.publication_count === 0
        ? "固定变更资料已准备，尚未发布。变更板与订阅源暂无已发布内容。"
        : `已发布 ${payload.publication_count} 条变更，已核对保存结果。`;
    }

    async function loadChangePublicationStatus() {
      radarPublicationRefresh.disabled = true;
      radarPublicationStatus.textContent = "正在读取已保存的校历发布状态…";
      try {
        renderChangePublicationStatus(await requestChangePublication("GET"));
      } catch (error) {
        radarPublicationStatus.textContent = `状态读取失败：${error instanceof Error ? error.message : "未知错误"}`;
      } finally {
        radarPublicationRefresh.disabled = false;
      }
    }

    async function publishChangeDemo() {
      if (radarPublicationConfirm.disabled) return;
      if (!radarPublicationConfirm.checked) {
        radarPublicationStatus.textContent = "请先勾选确认，再发布固定的校历变更。";
        return;
      }
      radarPublicationPublish.disabled = true;
      radarPublicationConfirm.disabled = true;
      radarPublicationStatus.textContent = "正在核对权限并发布校历变更…";
      try {
        const payload = await requestChangePublication(
          "POST",
          JSON.stringify({ confirm_publish: true })
        );
        if (
          payload?.schema !== "ustc-change-publication-response/v1" ||
          payload?.outcome?.kind !== "published"
        ) {
          throw new Error("校历发布结果格式无法识别");
        }
        radarPublicationStatus.textContent = "校历变更已发布，正在核对保存结果并刷新变更板…";
        await loadChangePublicationStatus();
        try {
          await onChangePublished();
        } catch (_) {
          radarPublicationStatus.textContent = "校历变更已发布；变更板刷新失败，请重新读取变更板。";
        }
      } catch (error) {
        radarPublicationStatus.textContent = `发布失败：${error instanceof Error ? error.message : "未知错误"}`;
      } finally {
        radarPublicationConfirm.checked = false;
        radarPublicationConfirm.disabled = false;
        radarPublicationPublish.disabled = true;
      }
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

    void loadPublicationStatus();
    void loadChangePublicationStatus();
  }
  return Object.freeze({mount});
})();
