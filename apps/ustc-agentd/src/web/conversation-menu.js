// Accessible menu and dialogs only. Conversation state and requests stay in conversations.js.
window.UcaConversationMenu = (() => {
  "use strict";
  const bytes=value=>new TextEncoder().encode(value).length;
  const invalidText=value=>/[\p{Cc}\p{Cf}]/u.test(value);
  function mount({canManage,onAction,getGroups=()=>[]}) {
    let selected=null, anchor=null;
    const menu=document.createElement("div");menu.className="conversation-menu";menu.hidden=true;menu.setAttribute("role","menu");menu.setAttribute("aria-label","对话操作");
    const dialog=document.createElement("dialog");dialog.className="conversation-manage-dialog";dialog.setAttribute("aria-labelledby","conversation-manage-title");
    const form=document.createElement("form");form.method="dialog";
    const heading=document.createElement("h2");heading.id="conversation-manage-title";
    const explanation=document.createElement("p");explanation.id="conversation-manage-explanation";dialog.setAttribute("aria-describedby",explanation.id);
    const label=document.createElement("label");
    const caption=document.createElement("span");
    const editor=document.createElement("span");editor.className="conversation-topic-editor";
    const prefix=document.createElement("span");prefix.id="conversation-rename-prefix";prefix.className="conversation-date-prefix";prefix.setAttribute("aria-label","保留的对话日期");
    const input=document.createElement("input");input.id="conversation-rename-title";input.type="text";input.autocomplete="off";
    editor.append(prefix,input);label.append(caption,editor);
    const groups=document.createElement("datalist");groups.id="conversation-group-options";
    const error=document.createElement("p");error.className="conversation-manage-error";error.setAttribute("role","alert");
    const actions=document.createElement("div");actions.className="conversation-manage-actions";
    const cancel=document.createElement("button");cancel.type="button";cancel.textContent="取消";cancel.dataset.manageDialog="cancel";
    const confirm=document.createElement("button");confirm.type="submit";confirm.dataset.manageDialog="confirm";
    actions.append(cancel,confirm);form.append(heading,explanation,label,groups,error,actions);dialog.append(form);document.body.append(menu,dialog);
    const restore=()=>{const target=anchor?.isConnected&&!anchor.disabled?anchor:document.querySelector("#conversation-refresh");target?.focus({preventScroll:true});};
    function close(restoreFocus=true){menu.hidden=true;anchor?.setAttribute("aria-expanded","false");if(restoreFocus)restore();}
    function openDialog(kind){
      if(!selected||!canManage())return;
      close(false);dialog.dataset.kind=kind;error.textContent="";
      const rename=kind==="rename",group=kind==="group",date=selected.organization?.date;
      heading.textContent=rename?"重命名对话":group?"移动到分组":"删除对话";
      explanation.textContent=rename?"日期保持不变，只修改话题名称。":group?"选择已有分组或输入新名称；留空可移出分组。":"删除后，对话将从历史列表中移除。服务器保留原始记录，不会撤销已执行的日历或插件操作。";
      label.hidden=!(rename||group);caption.textContent=rename?"话题名称":"分组名称";
      prefix.hidden=!rename;prefix.textContent=date?`${date}|`:"日期待确认 |";
      editor.classList.toggle("conversation-topic-editor-dated",rename);
      input.id=rename?"conversation-rename-title":"conversation-group-name";
      input.value=rename?(date&&selected.title.startsWith(`${date}|`)?selected.title.slice(7):selected.title):(selected.organization?.group??"");
      if(group){input.setAttribute("list",groups.id);groups.replaceChildren();for(const name of getGroups()){const option=document.createElement("option");option.value=name;groups.append(option);}}else input.removeAttribute("list");
      confirm.textContent=rename?"保存名称":group?"保存分组":"确认删除";
      dialog.showModal();if(rename||group){input.focus();input.select();}else cancel.focus();
    }
    for(const [kind,text] of [["rename","重命名"],["pin","置顶"],["group","移动到分组"],["ungroup","移出分组"],["delete","删除"]]){
      const button=document.createElement("button");button.type="button";button.textContent=text;button.dataset.conversationMenu=kind;button.setAttribute("role","menuitem");
      button.addEventListener("click",()=>{
        if(!selected||!canManage())return;
        if(kind==="pin"||kind==="ungroup"){const target=selected;close();onAction(target,kind==="pin"?{kind,pinned:!target.organization.pinned}:{kind:"group",group:null});}
        else openDialog(kind);
      });menu.append(button);
    }
    function open(entry,trigger,x,y){
      if(!canManage())return;
      close(false);selected={...entry};anchor=trigger;
      menu.querySelector('[data-conversation-menu="pin"]').textContent=entry.organization.pinned?"取消置顶":"置顶";
      menu.querySelector('[data-conversation-menu="ungroup"]').hidden=entry.organization.group===null;
      menu.hidden=false;anchor.setAttribute("aria-expanded","true");
      const rect=trigger.getBoundingClientRect();menu.style.left=`${Math.max(8,Math.min(x??rect.right,innerWidth-menu.offsetWidth-8))}px`;menu.style.top=`${Math.max(8,Math.min(y??rect.bottom,innerHeight-menu.offsetHeight-8))}px`;
      menu.querySelector("button").focus();
    }
    menu.addEventListener("keydown",event=>{
      const items=[...menu.querySelectorAll("button")].filter(button=>!button.hidden),index=items.indexOf(document.activeElement);
      if(["ArrowDown","ArrowUp","Home","End"].includes(event.key)){event.preventDefault();event.stopPropagation();items[event.key==="Home"?0:event.key==="End"?items.length-1:(index+(event.key==="ArrowDown"?1:items.length-1))%items.length].focus();}
      if(event.key==="Escape"||event.key==="Tab"){event.preventDefault();event.stopPropagation();close();}
    });
    dialog.addEventListener("keydown",event=>event.stopPropagation());dialog.addEventListener("close",restore);
    cancel.addEventListener("click",()=>dialog.close());
    form.addEventListener("submit",event=>{
      event.preventDefault();if(!selected||!canManage())return;
      const kind=dialog.dataset.kind,value=input.value;
      if(kind==="rename"&&(!value.trim()||bytes(value)>192||invalidText(value)||value.includes("|"))){error.textContent="请输入非空话题，最多 192 字节，不能包含 |、控制或格式字符。";input.focus();return;}
      if(kind==="group"&&(bytes(value)>64||invalidText(value))){error.textContent="分组名称最多 64 字节，不能包含控制或格式字符。";input.focus();return;}
      const target=selected;dialog.close();onAction(target,kind==="rename"?{kind,title:value}:kind==="group"?{kind,group:value.trim()||null}:{kind});
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
