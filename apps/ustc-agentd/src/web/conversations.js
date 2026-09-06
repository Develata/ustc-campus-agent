// A rebuildable view of server-owned conversations. No private transcript is stored in the browser.
window.UcaConversations = (() => {
  "use strict";
  const BASE = "/api/v1/agent/conversations";
  const headers = { Accept: "application/json", "Content-Type": "application/json", "X-USTC-Client-Protocol-Major": "1" };
  const textBytes = value => typeof value === "string" ? new TextEncoder().encode(value).length : Infinity;
  const fail = code => Object.assign(new Error(code), { code });
  const identifier = value => typeof value === "string" && value.length > 0 && textBytes(value) <= 256;
  const revision = value => Number.isSafeInteger(value) && value >= 0;
  function dateKey(value) {
    if(typeof value!=="string"||!/^\d{6}$/.test(value))return false;
    const year=2000+Number(value.slice(0,2)),month=Number(value.slice(2,4)),day=Number(value.slice(4,6));
    const parsed=new Date(Date.UTC(year,month-1,day));
    return parsed.getUTCFullYear()===year&&parsed.getUTCMonth()===month-1&&parsed.getUTCDate()===day;
  }
  function organization(value,title,required=false) {
    if(value===undefined&&!required){const date=title.slice(0,6);return {date:title[6]==="|"&&dateKey(date)?date:null,pinned:false,group:null};}
    if(!value||typeof value!=="object"||Array.isArray(value)||Object.keys(value).sort().join(",")!=="date,group,pinned"||
       (value.date!==null&&!dateKey(value.date))||typeof value.pinned!=="boolean"||
       (value.group!==null&&(typeof value.group!=="string"||!value.group.trim()||value.group!==value.group.trim()||textBytes(value.group)>64||/[\p{Cc}\p{Cf}]/u.test(value.group))))throw fail("invalid_response");
    return {...value};
  }
  const compareDate=(left,right)=>(right.organization.date??"").localeCompare(left.organization.date??"");
  function validateTurn(turn) {
    if (!turn || !identifier(turn.request_id) || textBytes(turn.user) > 4096 ||
        !["running", "completed", "failed", "interrupted"].includes(turn.phase) ||
        (turn.phase === "completed" ? !turn.response : turn.response != null) ||
        (turn.error != null && !/^[a-z0-9_]{1,64}$/.test(turn.error))) throw fail("invalid_response");
    return turn;
  }
  function mount(root, recovery, callbacks) {
    // Validate the complete server snapshot before changing current, pending or recovery controls.
    function validateDetail(value) {
      if (value?.schema !== "chat-conversation/v1" || !identifier(value.id) || textBytes(value.title) > 512 ||
          !revision(value.revision) || !Array.isArray(value.turns) || value.turns.length > 100) throw fail("invalid_response");
      const ids = new Set();
      for (const turn of value.turns) {
        validateTurn(turn);
        if (ids.has(turn.request_id)) throw fail("invalid_response");
        ids.add(turn.request_id);
        if (turn.response) callbacks.validateResponse(turn.response);
      }
      return {...value,organization:organization(value.organization,value.title)};
    }
    let entries = [], current = null, busy = false, available = false, pending = null, sequence = 0;
    let createRequest = null, management = null;
    const deleted = new Set();
    const heading = document.createElement("div"); heading.className = "conversation-history-header";
    const title = document.createElement("h2"); title.textContent = "历史对话";
    const refreshButton = document.createElement("button"); refreshButton.type = "button";
    refreshButton.id = "conversation-refresh"; refreshButton.className = "conversation-history-refresh";
    refreshButton.textContent = "刷新"; refreshButton.setAttribute("aria-label", "刷新历史对话");
    heading.append(title, refreshButton);
    const status = document.createElement("p"); status.className = "conversation-history-status";
    status.id = "conversation-history-status"; status.setAttribute("role", "status");
    const list = document.createElement("ol"); list.className = "conversation-history-list";
    root.replaceChildren(heading, status, list);
    const menu = window.UcaConversationMenu.mount({canManage:()=>available && !busy && !pending && !management,onAction:manage,getGroups:()=>[...new Set(entries.map(entry=>entry.organization.group).filter(value=>value!==null))].sort((a,b)=>a<b?-1:a>b?1:0)});

    async function request(url, options = {}) {
      const controller = new AbortController(); let timer;
      try {
        return await Promise.race([
          (async () => {
            const response = await fetch(url, { ...options, headers: { ...headers, ...options.headers }, credentials: "same-origin", cache: "no-store", signal: controller.signal });
            const payload = await response.json();
            if (!response.ok) {
              const error = fail(typeof payload?.error === "string" && /^[a-z0-9_]{1,64}$/.test(payload.error) ? payload.error : "request_failed");
              error.status = response.status; error.confirmed = payload?.schema === "chat-conversation-error/v1";
              error.rejected = payload?.schema === "chat-conversation-error/v1" && [400, 429].includes(response.status);
              throw error;
            }
            return payload;
          })(),
          new Promise((_, reject) => { timer = setTimeout(() => { controller.abort(); reject(fail("network_error")); }, 75000); })
        ]);
      } catch (error) { throw error?.code ? error : fail("network_error"); }
      finally { clearTimeout(timer); }
    }
    function updateControls() {
      refreshButton.disabled = busy;
      for (const button of recovery.querySelectorAll("button")) button.disabled = busy;
      for (const button of list.querySelectorAll("button")) button.disabled = busy || Boolean(pending) || Boolean(management);
      callbacks.availability?.({ canSend: available && !busy && !pending && !management, canSwitch: !busy && !pending && !management });
    }
    function setBusy(value) {
      if (value) { ++sequence; menu.close(); } // Older list reads must not race a selection or mutation.
      busy = value; callbacks.busy(value, {kind:management ? "management" : "conversation"}); updateControls();
    }
    function renderList() {
      list.replaceChildren();
      const pinned=entries.filter(entry=>entry.organization.pinned).sort(compareDate);
      const ordinary=entries.filter(entry=>!entry.organization.pinned);
      const groups=[...new Set(ordinary.map(entry=>entry.organization.group).filter(value=>value!==null))].sort((a,b)=>a<b?-1:a>b?1:0);
      const sections=[{name:"置顶",kind:"pinned",entries:pinned},...groups.map(name=>({name,kind:"group",entries:ordinary.filter(entry=>entry.organization.group===name).sort(compareDate)})),{name:groups.length||pinned.length?"未分组":"对话",kind:"ungrouped",entries:ordinary.filter(entry=>entry.organization.group===null).sort(compareDate)}];
      for(const section of sections){
        if(!section.entries.length)continue;
        const wrapper=document.createElement("li");wrapper.className="conversation-section";wrapper.dataset.conversationSection=section.kind;
        if(section.kind==="group")wrapper.dataset.conversationGroup=section.name;
        const heading=document.createElement("h3");heading.textContent=section.name;
        const rows=document.createElement("ol");rows.className="conversation-section-list";rows.setAttribute("aria-label",section.name);
        wrapper.append(heading,rows);list.append(wrapper);
        for(const entry of section.entries){
          const li=document.createElement("li"),button=document.createElement("button");li.className="conversation-row";
          button.type="button";button.className="conversation-open";button.dataset.conversationId=entry.id;
          button.textContent=entry.title||"新对话";button.title=entry.title||"新对话";
          if(entry.id===current?.id)button.setAttribute("aria-current","true");
          button.addEventListener("click",()=>{void select(entry.id);});
          li.append(button);menu.attach(li,entry);rows.append(li);
        }
      }
      updateControls();
    }
    function mergeSummary(summary) {
      const index=entries.findIndex(entry=>entry.id===summary.id);
      if(index<0)entries.unshift(summary); // Only a newly created conversation precedes same-day peers.
      else if(summary.revision>=entries[index].revision)entries[index]=summary;
      entries=entries.slice(0,50);
    }
    function showRecovery(message, retry = false, cancel = false) {
      recovery.replaceChildren(); recovery.hidden = !message;
      if (!message) return;
      const copy = document.createElement("p"); copy.textContent = message; recovery.append(copy);
      const check = document.createElement("button"); check.type = "button"; check.id = "conversation-check-result";
      check.textContent = "检查结果"; check.disabled = busy;
      check.addEventListener("click", () => { void recover(); }); recovery.append(check);
      if (pending?.inaccessible) {
        const detach=document.createElement("button");detach.type="button";detach.id="conversation-detach-unavailable";detach.textContent="新建对话并保留草稿";
        detach.addEventListener("click",()=>{
          if(busy)return;
          if(current){deleted.add(current.id);entries=entries.filter(entry=>entry.id!==current.id);}
          pending=null;current=null;createRequest=null;callbacks.activityCleared?.();showRecovery("");callbacks.render(null);renderList();callbacks.selected?.({preserveDraft:true});
        });recovery.append(detach);
      }
      if (retry && pending?.body) {
        const resend = document.createElement("button"); resend.type = "button"; resend.id = "conversation-retry-turn";
        resend.textContent = "重试原请求"; resend.disabled = busy;
        resend.addEventListener("click", () => { void retryPending(); }); recovery.append(resend);
      }
      if (cancel && pending?.rejected) {
        const abandon = document.createElement("button"); abandon.type = "button"; abandon.id = "conversation-cancel-send";
        abandon.textContent = "取消这次发送，保留草稿"; abandon.disabled = busy;
        abandon.addEventListener("click", () => {
          if (busy) return;
          pending = null; callbacks.activityCleared?.(); showRecovery(""); updateControls();
        }); recovery.append(abandon);
      }
    }
    function remember(detail) {
      current = validateDetail(detail);
      ++sequence;
      status.textContent = "";
      const summary = { id: current.id, title: current.title, revision: current.revision, turn_count: current.turns.length, organization: current.organization };
      mergeSummary(summary);
      renderList();
    }
    function observe(detail) {
      remember(detail); callbacks.render(detail);
      const running = detail.turns.find(turn => turn.phase === "running");
      if (running) {
        pending = pending?.requestId === running.request_id ? pending : { requestId: running.request_id, body: null };
        showRecovery("这次请求仍在处理中。检查结果只会读取已保存状态，不会再次执行。 ");
      }
      updateControls();
    }
    async function refreshList(background = false) {
      const ticket = ++sequence;
      if (!background) status.textContent = "正在读取历史对话…";
      try {
        const value = await request(BASE);
        if (ticket !== sequence) return;
        if (value?.schema !== "chat-conversation-list/v1" || !Array.isArray(value.conversations) || value.conversations.length > 50) throw fail("invalid_response");
        const ids = new Set();
        for (const entry of value.conversations) {
          if (!identifier(entry.id) || textBytes(entry.title) > 512 || !revision(entry.revision) ||
              !Number.isInteger(entry.turn_count) || entry.turn_count < 0 || entry.turn_count > 100 || ids.has(entry.id)) throw fail("invalid_response");
          ids.add(entry.id);
          entry.organization=organization(entry.organization,entry.title);
        }
        // A server snapshot cannot roll back a detail/result already accepted locally.
        const accepted = new Map(entries.map(entry => [entry.id, entry]));
        entries = value.conversations.filter(entry=>!deleted.has(entry.id)).map(entry => {
          const previous = accepted.get(entry.id);
          return previous?.revision > entry.revision ? previous : entry;
        });
        const summary = entries.find(entry => entry.id === current?.id);
        if (summary && summary.revision >= current.revision) current = { ...current, title: summary.title, organization: summary.organization };
        available = true;
        status.textContent = entries.length ? "" : "发送第一条消息，开始一段对话。";
        renderList();
      } catch (_) {
        if (ticket !== sequence) return;
        if (!background) available = false;
        status.textContent = background
          ? "对话结果已保存，历史列表暂未更新。请点刷新。"
          : "历史对话暂时无法读取。请点刷新后继续。";
        updateControls();
      }
    }
    async function refresh() {
      if (busy) return;
      if (management?.receipt) return retryManagement();
      return refreshList();
    }
    async function select(id) {
      if (busy || pending || management) return;
      callbacks.activityCleared?.();
      setBusy(true); showRecovery("");
      try {
        const detail = validateDetail(await request(`${BASE}/${encodeURIComponent(id)}`));
        if (detail.id !== id) throw fail("invalid_response");
        observe(detail); callbacks.selected?.();
        const last=detail.turns.at(-1);
        if(last && last.phase!=="completed")callbacks.activityStarted?.(detail.id,last.request_id);
      } catch (error) { callbacks.error(error.code); }
      finally { setBusy(false); }
    }
    function newConversation() {
      if (busy || pending || management) return false;
      callbacks.activityCleared?.();
      current = null; createRequest = null; showRecovery(""); callbacks.render(null); renderList(); callbacks.selected?.();
      return true;
    }
    async function ensureConversation() {
      if (current) return;
      createRequest ||= { schema: "chat-conversation-create/v1", request_id: crypto.randomUUID() };
      remember(await request(BASE, { method: "POST", body: JSON.stringify(createRequest) }));
      createRequest = null;
    }
    function result(value) {
      if (value?.schema !== "chat-conversation-turn-result/v1" || value.conversation_id !== current.id ||
          !revision(value.revision) || value.revision < current.revision || value.turn?.request_id !== pending?.requestId) throw fail("invalid_response");
      validateTurn(value.turn);
      if (value.turn.phase === "running") throw fail("conversation_in_progress");
      if (value.turn.response) callbacks.validateResponse(value.turn.response);
      const turns = current.turns.filter(turn => turn.request_id !== value.turn.request_id);
      turns.push(value.turn); pending = null; showRecovery("");
      observe({ ...current, revision: value.revision, turns });
      return value.turn;
    }
    async function postPending() {
      callbacks.activityStarted?.(current.id, pending.requestId);
      try {
        const value = await request(`${BASE}/${encodeURIComponent(current.id)}/turns`, { method: "POST", body: pending.body, headers: pending.headers });
        const turn = result(value);
        callbacks.activityFinished?.("settled");
        // Turn results carry no title. Refresh the server-owned summary separately;
        // a failed metadata read must never turn a saved result into a retry.
        void refreshList(true);
        return turn;
      } catch (error) {
        callbacks.activityFinished?.("unknown");
        if (pending) pending.rejected = error.rejected === true;
        if (markInaccessible(error)) throw error;
        showRecovery(pending?.rejected
          ? "服务器未接受这次请求。检查已保存状态后，可以保留草稿并取消这次发送。"
          : "没有收到确定结果。先检查已保存状态，以免重复执行日历等操作。");
        throw error;
      }
    }
    async function submit(intent, extraHeaders) {
      if (!available) throw fail("conversation_unavailable");
      if (busy || pending || management) throw fail("conversation_in_progress");
      setBusy(true);
      try {
        await ensureConversation();
        const requestId = crypto.randomUUID();
        const body = JSON.stringify({ schema: "chat-conversation-turn/v2", request_id: requestId, expected_revision: current.revision, ...intent });
        pending = { requestId, body, headers: { ...extraHeaders } };
        return await postPending();
      } finally { setBusy(false); }
    }
    async function recover() {
      if (busy || !current) return;
      setBusy(true);
      try {
        const detail = validateDetail(await request(`${BASE}/${encodeURIComponent(current.id)}`));
        if (detail.id !== current.id) throw fail("invalid_response");
        const turn = detail.turns.find(item => item.request_id === pending?.requestId);
        if (turn && turn.phase !== "running") { pending = null; showRecovery(""); }
        observe(detail);
        if (!turn && pending) {
          // Revision conflicts cannot safely reuse this intent with a newly chosen revision.
          if (pending.body && JSON.parse(pending.body).expected_revision !== detail.revision) {
            pending = null; callbacks.activityCleared?.(); showRecovery(""); callbacks.error("conversation_revision_conflict");
          } else if (pending.rejected) showRecovery("已确认这次请求未被接纳，也没有新增记录。可以取消这次发送，保留草稿后新建对话或调整问题。", false, true);
          else showRecovery("尚未找到这条请求的记录。可以再次检查，或使用同一个请求编号重试。", true);
        }
        if (turn && turn.phase !== "running") { callbacks.activityCleared?.(); callbacks.recovered?.(turn); }
      } catch (error) { if (!markInaccessible(error)) callbacks.error(error.code); }
      finally { setBusy(false); }
    }
    async function retryPending() {
      if (busy || !pending?.body) return;
      setBusy(true);
      try { const turn = await postPending(); callbacks.recovered?.(turn); }
      catch (error) { callbacks.error(error.code); }
      finally { setBusy(false); }
    }
    function markInaccessible(error) {
      if(!pending || !error.confirmed || error.status!==404 || error.code!=="conversation_not_found")return false;
      pending.inaccessible=true;
      showRecovery("这段对话已不可访问。先前请求可能已执行并留有记录；新建对话不会自动重发，也不会撤销已有操作。可保留草稿后继续。");
      return true;
    }
    function showManagementRecovery() {
      recovery.replaceChildren(); recovery.hidden = !management;
      if (!management) return;
      const copy=document.createElement("p");copy.textContent=management.receipt
        ? "操作已确认，但尚未读到对话的当前状态。请刷新确认；不会再次提交管理操作。"
        : "尚未确认对话操作是否完成。请重试原操作；内容和请求编号保持不变，其他写入暂时锁定。";
      const retry=document.createElement("button");retry.type="button";retry.id="conversation-retry-manage";retry.textContent=management.receipt?"读取当前状态":"重试原操作";
      retry.addEventListener("click",()=>{void retryManagement();});recovery.append(copy,retry);updateControls();
    }
    async function manage(entry, action) {
      if (!available || busy || pending || management) return;
      const requestId=crypto.randomUUID();
      management={id:entry.id,requestId,before:{title:entry.title,organization:{...entry.organization}},body:JSON.stringify({schema:"chat-conversation-manage/v2",request_id:requestId,expected_revision:entry.revision,action}),receipt:null};
      await retryManagement();
    }
    async function reconcileManagement() {
      const target=management.id;
      let detail=null;
      try {
        detail=validateDetail(await request(`${BASE}/${encodeURIComponent(target)}`));
        if(detail.id!==target||management.receipt.deleted||detail.revision<management.receipt.revision||
           (management.receipt.organization.date!==null&&detail.organization.date!==management.receipt.organization.date)||
           (detail.revision===management.receipt.revision&&(detail.title!==management.receipt.title||
             detail.organization.pinned!==management.receipt.organization.pinned||detail.organization.group!==management.receipt.organization.group)))throw fail("invalid_response");
      } catch(error) {if(error.status!==404 || !error.confirmed)throw error;}
      ++sequence;
      management=null;showManagementRecovery();
      if (!detail) {
        deleted.add(target);entries=entries.filter(entry=>entry.id!==target);
        if(current?.id===target){current=null;createRequest=null;callbacks.activityCleared?.();callbacks.render(null);callbacks.selected?.();}
      } else if (!deleted.has(target)) {
        if(current?.id===target){if(detail.revision>=current.revision)observe(detail);}
        else {
          const summary={id:detail.id,title:detail.title,revision:detail.revision,turn_count:detail.turns.length,organization:detail.organization};
          mergeSummary(summary);
        }
      }
      renderList();status.textContent="对话状态已更新。";
      // The list owns creation-order ties, including after a pin changes sections.
      await refreshList(true);
    }
    async function retryManagement() {
      if(busy || !management)return;
      setBusy(true);
      try {
        if(!management.receipt){
          const intent=JSON.parse(management.body);
          const receipt=await request(`${BASE}/${encodeURIComponent(management.id)}/manage`,{method:"POST",body:management.body});
          if(receipt?.schema!=="chat-conversation-manage-result/v2"||receipt.conversation_id!==management.id||receipt.request_id!==management.requestId||
             !revision(receipt.revision)||receipt.revision!==intent.expected_revision+1||typeof receipt.deleted!=="boolean"||receipt.deleted!==(intent.action.kind==="delete")||textBytes(receipt.title)>512)throw fail("invalid_response");
          const actual=organization(receipt.organization,receipt.title,true),before=management.before.organization,action=intent.action;
          if((before.date!==null&&actual.date!==before.date)||
             actual.pinned!==(action.kind==="pin"?action.pinned:before.pinned)||
             actual.group!==(action.kind==="group"?action.group:before.group)||
             (action.kind==="rename"?actual.date===null||receipt.title!==`${actual.date}|${action.title.trim()}`:receipt.title!==management.before.title))throw fail("invalid_response");
          management.receipt=receipt;
        }
        // Receipts may be historical: only a fresh authoritative read can project the current view.
        await reconcileManagement();
      } catch(error){
        if(management && !management.receipt && error.confirmed && error.status>=400 && error.status<500 && (! [408,429].includes(error.status) || error.status===429 && error.code==="conversation_capacity_exceeded")){
          management=null;showManagementRecovery();await refreshList(true);status.textContent="服务器未接受此操作。请根据最新对话状态重新检查，草稿已保留。";
        }else{showManagementRecovery();status.textContent="对话管理结果仍需确认。";}
      }finally{setBusy(false);}
    }
    refreshButton.addEventListener("click", () => { void refresh(); });
    updateControls();
    void refresh();
    return Object.freeze({ submit, newConversation, refresh, recover, get currentId() { return current?.id ?? null; } });
  }
  return Object.freeze({ mount });
})();
