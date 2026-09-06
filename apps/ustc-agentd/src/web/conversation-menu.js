// Accessible menu and dialogs only. Conversation state and requests stay in conversations.js.
window.UcaConversationMenu = (() => {
  "use strict";
  function mount({canManage,onAction}) {
    let selected=null, anchor=null;
    const menu=document.createElement("div");menu.className="conversation-menu";menu.hidden=true;menu.setAttribute("role","menu");menu.setAttribute("aria-label","对话操作");
    const dialog=document.createElement("dialog");dialog.className="conversation-manage-dialog";dialog.setAttribute("aria-labelledby","conversation-manage-title");
    const form=document.createElement("form");form.method="dialog";
    const heading=document.createElement("h2");heading.id="conversation-manage-title";
    const explanation=document.createElement("p");explanation.id="conversation-manage-explanation";dialog.setAttribute("aria-describedby",explanation.id);
    const label=document.createElement("label");label.textContent="对话名称";
    const input=document.createElement("input");input.id="conversation-rename-title";input.type="text";input.autocomplete="off";label.append(input);
    const error=document.createElement("p");error.className="conversation-manage-error";error.setAttribute("role","alert");
    const actions=document.createElement("div");actions.className="conversation-manage-actions";
    const cancel=document.createElement("button");cancel.type="button";cancel.textContent="取消";cancel.dataset.manageDialog="cancel";
    const confirm=document.createElement("button");confirm.type="submit";confirm.dataset.manageDialog="confirm";
    actions.append(cancel,confirm);form.append(heading,explanation,label,error,actions);dialog.append(form);document.body.append(menu,dialog);
    const restore=()=>{const target=anchor?.isConnected&&!anchor.disabled?anchor:document.querySelector("#conversation-refresh");target?.focus({preventScroll:true});};
    function close(restoreFocus=true){menu.hidden=true;anchor?.setAttribute("aria-expanded","false");if(restoreFocus)restore();}
    function openDialog(kind){
      if(!selected||!canManage())return;
      close(false);dialog.dataset.kind=kind;heading.textContent=kind==="rename"?"重命名对话":"删除对话";
      explanation.textContent=kind==="rename"?"修改历史列表中的名称，不改变对话内容。":"删除后，对话将从历史列表中移除。服务器保留原始记录，不会撤销已执行的日历或插件操作。";
      label.hidden=kind!=="rename";input.value=selected.title;error.textContent="";confirm.textContent=kind==="rename"?"保存名称":"确认删除";
      dialog.showModal();if(kind==="rename"){input.focus();input.select();}else cancel.focus();
    }
    for(const [kind,text] of [["rename","重命名"],["delete","删除"]]){
      const button=document.createElement("button");button.type="button";button.textContent=text;button.dataset.conversationMenu=kind;button.setAttribute("role","menuitem");button.addEventListener("click",()=>openDialog(kind));menu.append(button);
    }
    function open(entry,trigger,x,y){
      if(!canManage())return;
      close(false);selected={...entry};anchor=trigger;menu.hidden=false;anchor.setAttribute("aria-expanded","true");
      const rect=trigger.getBoundingClientRect();menu.style.left=`${Math.max(8,Math.min(x??rect.right,innerWidth-menu.offsetWidth-8))}px`;menu.style.top=`${Math.max(8,Math.min(y??rect.bottom,innerHeight-menu.offsetHeight-8))}px`;
      menu.querySelector("button").focus();
    }
    menu.addEventListener("keydown",event=>{
      const items=[...menu.querySelectorAll("button")],index=items.indexOf(document.activeElement);
      if(["ArrowDown","ArrowUp","Home","End"].includes(event.key)){event.preventDefault();event.stopPropagation();items[event.key==="Home"?0:event.key==="End"?items.length-1:(index+(event.key==="ArrowDown"?1:items.length-1))%items.length].focus();}
      if(event.key==="Escape"||event.key==="Tab"){event.preventDefault();event.stopPropagation();close();}
    });
    dialog.addEventListener("keydown",event=>event.stopPropagation());
    dialog.addEventListener("close",restore);
    cancel.addEventListener("click",()=>dialog.close());
    form.addEventListener("submit",event=>{
      event.preventDefault();if(!selected||!canManage())return;
      const kind=dialog.dataset.kind,title=input.value;
      if(kind==="rename"&&(!title.trim()||new TextEncoder().encode(title).length>192||/[\u0000-\u001f\u007f-\u009f]/u.test(title))){error.textContent="请输入非空名称，最多 192 字节，且不能包含控制字符。";input.focus();return;}
      const target=selected;dialog.close();onAction(target,kind==="rename"?{kind,title}:{kind});
    });
    document.addEventListener("pointerdown",event=>{if(!menu.hidden&&!menu.contains(event.target)&&!anchor?.contains(event.target))close(false);});
    window.addEventListener("resize",()=>close(false));
    return Object.freeze({attach(row,entry){
      const trigger=document.createElement("button");trigger.type="button";trigger.className="conversation-menu-trigger";trigger.dataset.conversationMenuId=entry.id;trigger.textContent="⋯";trigger.setAttribute("aria-label",`管理对话：${entry.title||"新对话"}`);trigger.setAttribute("aria-haspopup","menu");trigger.setAttribute("aria-expanded","false");
      trigger.addEventListener("click",event=>{event.stopPropagation();open(entry,trigger);});
      row.addEventListener("contextmenu",event=>{event.preventDefault();open(entry,trigger,event.clientX,event.clientY);});
      row.addEventListener("keydown",event=>{if(event.key==="ContextMenu"||(event.shiftKey&&event.key==="F10")){event.preventDefault();event.stopPropagation();open(entry,trigger);}});
      row.append(trigger);
    },close(){close(false);if(dialog.open)dialog.close();}});
  }
  return Object.freeze({mount});
})();