// Month/date navigation owns presentation only. Items and the clock come from the server.
window.UcaCalendarMonth = (() => {
  "use strict";
  const zone = new Intl.DateTimeFormat("en-CA", {timeZone:"Asia/Shanghai", year:"numeric",month:"2-digit",day:"2-digit"});
  const clock = new Intl.DateTimeFormat("zh-CN", {timeZone:"Asia/Shanghai",hour:"2-digit",minute:"2-digit",hourCycle:"h23"});
  function dayKey(value) {
    if (value == null) return null;
    const date = new Date(value);
    if (!Number.isFinite(date.getTime())) return null;
    const p = Object.fromEntries(zone.formatToParts(date).map(x=>[x.type,x.value]));
    return `${p.year.padStart(4,"0")}-${p.month}-${p.day}`;
  }
  function utc(key) { return new Date(`${key}T12:00:00Z`); }
  function keyOf(date) {
    const y=date.getUTCFullYear();
    return y<1||y>9999 ? null : `${String(y).padStart(4,"0")}-${String(date.getUTCMonth()+1).padStart(2,"0")}-${String(date.getUTCDate()).padStart(2,"0")}`;
  }
  function shiftDay(key, delta) { const d=utc(key);d.setUTCDate(d.getUTCDate()+delta);return keyOf(d); }
  function monthLength(month) { const d=utc(`${month}-01`);d.setUTCMonth(d.getUTCMonth()+1,0);return d.getUTCDate(); }
  function shiftMonth(key, delta) {
    const d=utc(key), day=d.getUTCDate();d.setUTCDate(1);d.setUTCMonth(d.getUTCMonth()+delta);
    const first=keyOf(d);return first ? `${first.slice(0,7)}-${String(Math.min(day,monthLength(first.slice(0,7)))).padStart(2,"0")}` : null;
  }
  function label(key) {const [y,m,d]=key.split("-").map(Number);return `${y}年${m}月${d}日`;}
  function node(tag,text,cls) {const e=document.createElement(tag);if(text!==undefined)e.textContent=text;if(cls)e.className=cls;return e;}
  function mount(root,onSelect) {
    let selected=null, month=null, today=null, items=[], previous="";
    const toolbar=node("div",undefined,"calendar-month-toolbar");
    const heading=node("label",undefined,"calendar-month-picker");
    const headingText=node("span","读取日历…");
    const picker=node("input");picker.type="month";picker.min="0001-01";picker.max="9999-12";picker.setAttribute("aria-label","选择年月");
    heading.append(headingText,picker);
    function button(text,name,fn) {const e=node("button",text);e.type="button";e.setAttribute("aria-label",name);e.addEventListener("click",fn);return e;}
    const prev=button("‹","上个月",()=>select(shiftMonth(selected||`${month}-01`,-1)));
    const next=button("›","下个月",()=>select(shiftMonth(selected||`${month}-01`,1)));
    const current=button("今天","回到今天",()=>select(today));
    const navigation=node("div",undefined,"calendar-month-navigation");navigation.append(prev,current,next);
    toolbar.append(heading,navigation);
    const table=node("table",undefined,"calendar-month-table");table.setAttribute("aria-label","月历，按北京时间显示事项");
    const thead=node("thead"), weekdays=node("tr");
    for(const day of ["一","二","三","四","五","六","日"]){const th=node("th",day);th.scope="col";th.setAttribute("aria-label",`星期${day}`);weekdays.append(th);}
    thead.append(weekdays);const body=node("tbody");table.append(thead,body);
    const footer=node("div",undefined,"calendar-month-footer");
    const undated=button("无日期事项","查看无日期事项",()=>select(null));
    footer.append(node("span","北京时间 · 点击日期查看事项","calendar-month-hint"),undated);
    root.append(toolbar,table,footer);
    function select(key,focus=false) {
      if(!month || (key!==null&&!/^\d{4}-\d{2}-\d{2}$/.test(key)))return;
      selected=key;if(key)month=key.slice(0,7);render();onSelect();
      if(focus)body.querySelector(`[data-calendar-day="${selected}"]`)?.focus({preventScroll:true});
    }
    picker.addEventListener("change",()=>{if(picker.validity.valid&&/^\d{4}-\d{2}$/.test(picker.value))select(`${picker.value}-01`);});
    body.addEventListener("keydown",event=>{
      const key=event.target.closest("[data-calendar-day]")?.dataset.calendarDay;if(!key)return;
      const movement={ArrowLeft:-1,ArrowRight:1,ArrowUp:-7,ArrowDown:7};let target;
      if(Object.hasOwn(movement,event.key))target=shiftDay(key,movement[event.key]);
      else if(event.key==="PageUp"||event.key==="PageDown")target=shiftMonth(key,event.key==="PageUp"?-1:1);
      else if(event.key==="Home"||event.key==="End"){const index=(utc(key).getUTCDay()+6)%7;target=shiftDay(key,event.key==="Home"?-index:6-index);}
      else return;
      event.preventDefault();if(target)select(target,true);
    });
    function render() {
      for(const control of [picker,prev,next,current,undated])control.disabled=!month;
      if(!month)return;
      const focusKey=body.contains(document.activeElement)?document.activeElement.dataset.calendarDay:null;
      const [year,number]=month.split("-").map(Number);
      headingText.textContent=`${year}年 ${number}月`;picker.value=month;
      prev.disabled=month==="0001-01";next.disabled=month==="9999-12";
      const grouped=new Map();
      for(const item of items){const key=dayKey(item.scheduled_for);if(!grouped.has(key))grouped.set(key,[]);grouped.get(key).push(item);}
      for(const group of grouped.values())group.sort((a,b)=>Date.parse(a.scheduled_for)-Date.parse(b.scheduled_for)||a.id.localeCompare(b.id));
      undated.textContent=`无日期事项 · ${grouped.get(null)?.length||0}`;undated.setAttribute("aria-pressed",String(selected===null));
      const first=`${month}-01`, offset=(utc(first).getUTCDay()+6)%7;
      const total=Math.ceil((offset+monthLength(month))/7)*7;
      body.replaceChildren();
      for(let index=0;index<total;index++){
        if(index%7===0)body.append(node("tr"));
        const cell=node("td"), key=shiftDay(first,index-offset);body.lastChild.append(cell);
        if(!key)continue;
        const entries=grouped.get(key)||[], control=button("",`${label(key)}${key===today?"，今天":""}，${entries.length}个事项`,()=>select(key));
        control.className="calendar-day";control.dataset.calendarDay=key;control.tabIndex=key===(selected||first)?0:-1;
        control.classList.toggle("is-outside",key.slice(0,7)!==month);control.classList.toggle("is-selected",key===selected);
        control.setAttribute("aria-pressed",String(key===selected));if(key===today)control.setAttribute("aria-current","date");
        control.append(node("span",String(Number(key.slice(-2))),"calendar-day-number"));
        const previews=node("span",undefined,"calendar-day-events");
        for(const item of entries.slice(0,2))previews.append(node("span",`${clock.format(new Date(item.scheduled_for))} ${item.title}`,"calendar-day-event"));
        if(entries.length>2)previews.append(node("span",`还有 ${entries.length-2} 项`,"calendar-day-more"));
        if(entries.length)control.append(node("span",`${entries.length}项`,"calendar-day-count"));
        control.append(previews);cell.append(control);
      }
      if(focusKey)body.querySelector(`[data-calendar-day="${focusKey}"]`)?.focus({preventScroll:true});
    }
    render();
    return Object.freeze({
      get selected(){return selected;},
      selectItem(item){select(dayKey(item.scheduled_for));},
      update(nextItems,now){
        const nextToday=dayKey(now*1000);if(!nextToday)return;
        const stamp=JSON.stringify([nextItems,nextToday]);if(stamp===previous)return;previous=stamp;
        items=nextItems;today=nextToday;if(!month){month=today.slice(0,7);selected=today;}render();
      },
      heading(){return selected?`${label(selected)}的事项`:"无日期事项";}
    });
  }
  return Object.freeze({mount,dayKey});
})();
