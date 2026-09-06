// A bounded presentation subset. Never interpret provider HTML or infer source trust.
// All text becomes Text nodes; only absolute, credential-free HTTP(S) links are clickable.
window.UcaChatMarkdown = (() => {
  "use strict";
  function inline(parent, source) {
    const pattern = /(`[^`\n]+`|\*\*[^*\n]+\*\*|\*[^*\n]+\*|\[[^\]\n]+\]\([^\s)]+\))/g;
    let offset = 0;
    for (const match of source.matchAll(pattern)) {
      parent.append(document.createTextNode(source.slice(offset, match.index)));
      const token = match[0];
      let node;
      if (token.startsWith("`")) {
        node = document.createElement("code");
        node.textContent = token.slice(1, -1);
      } else if (token.startsWith("**")) {
        node = document.createElement("strong");
        node.textContent = token.slice(2, -2);
      } else if (token.startsWith("*")) {
        node = document.createElement("em");
        node.textContent = token.slice(1, -1);
      } else {
        const end = token.indexOf("](");
        const label = token.slice(1, end);
        const raw = token.slice(end + 2, -1);
        try {
          const url = new URL(raw);
          if (!/^https?:\/\//i.test(raw) || !["http:", "https:"].includes(url.protocol)
            || url.username || url.password || /[\u0000-\u0020\u007f]/.test(raw)) throw Error("unsafe link");
          node = document.createElement("a");
          node.href = url.href;
          node.target = "_blank";
          node.rel = "noopener noreferrer";
          node.textContent = label;
        } catch (_) {
          node = document.createTextNode(token);
        }
      }
      parent.append(node);
      offset = match.index + token.length;
    }
    parent.append(document.createTextNode(source.slice(offset)));
  }

  function render(parent, source) {
    const lines = source.split(/\r?\n/);
    let paragraph = [];
    let list = null;
    let code = null;
    function flush() {
      if (paragraph.length) {
        const p = document.createElement("p");
        inline(p, paragraph.join("\n"));
        parent.append(p);
        paragraph = [];
      }
    }
    for (const line of lines) {
      if (/^\s*```/.test(line)) {
        flush();
        list = null;
        if (code) { code = null; } else {
          const pre = document.createElement("pre");
          code = document.createElement("code");
          pre.append(code);
          parent.append(pre);
        }
        continue;
      }
      if (code) { code.append(document.createTextNode(line + "\n")); continue; }
      const heading = /^(#{1,4})\s+(.+)$/.exec(line);
      const bullet = /^\s*(?:([-*])|(\d+)[.)])\s+(.+)$/.exec(line);
      if (heading) {
        flush(); list = null;
        const h = document.createElement(`h${Math.min(heading[1].length + 2, 6)}`);
        inline(h, heading[2]); parent.append(h);
      } else if (bullet) {
        flush();
        const tag = bullet[2] ? "OL" : "UL";
        if (!list || list.tagName !== tag) {
          list = document.createElement(tag);
          if (bullet[2]) list.start = Math.min(Number(bullet[2]), 10000);
          parent.append(list);
        }
        const li = document.createElement("li");
        inline(li, bullet[3]); list.append(li);
      } else if (/^>\s?/.test(line)) {
        flush(); list = null;
        const quote = document.createElement("blockquote");
        inline(quote, line.replace(/^>\s?/, "")); parent.append(quote);
      } else if (!line.trim()) { flush(); list = null; }
      else { list = null; paragraph.push(line); }
    }
    flush();
  }
  return Object.freeze({ render });
})();
