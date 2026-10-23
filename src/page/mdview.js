// ---------- the note reader: a vault note's markdown, rendered in a popup (data: /api/note?path=…) ----------
// a small renderer, no library and no network: front matter, headings, paragraphs, lists (nested by indent, with task
// boxes), fenced code, quotes, tables, rules, and inline code, bold, italic, strike, links and images. Everything is
// escaped first, so a note can never put HTML or script into the page; links open in the browser, never in this page.
function mdInline(t) {
  const code = [];
  t = esc(t).replace(/`([^`]+)`/g, (_, c) => `\u0000${code.push(c) - 1}\u0000`);
  t = t.replace(/!\[([^\]]*)\]\(([^)\s]+)[^)]*\)/g, (_, a) => `<span class="mimg">[image: ${a || "untitled"}]</span>`)
    .replace(/\[([^\]]+)\]\(([^)\s]+)[^)]*\)/g, (_, a, u) => /^(https?:|mailto:)/i.test(u.replace(/&amp;/g, "&")) ? `<a href="${u}" target="_blank" rel="noopener">${a}</a>` : `<span class="mref" title="${u}">${a}</span>`)
    .replace(/\[\[([^\]|]+)(?:\|([^\]]+))?\]\]/g, (_, a, b) => `<span class="mref" title="${a}">${b || a}</span>`)
    .replace(/(^|[\s(])(https?:\/\/[^\s<)]+)/g, '$1<a href="$2" target="_blank" rel="noopener">$2</a>')
    .replace(/\*\*([^*]+)\*\*|__([^_]+)__/g, (_, a, b) => `<b>${a || b}</b>`)
    .replace(/(^|[^*\w])\*([^*\s][^*]*)\*(?!\*)/g, "$1<i>$2</i>").replace(/(^|\W)_([^_\s][^_]*)_(?=\W|$)/g, "$1<i>$2</i>")
    .replace(/~~([^~]+)~~/g, "<s>$1</s>");
  return t.replace(/\u0000(\d+)\u0000/g, (_, i) => `<code>${code[+i]}</code>`);
}
function md2html(src) {
  const L = src.replace(/\r\n?/g, "\n").split("\n"), out = [];
  let i = 0;
  // front matter: shown as a small key/value block above the note
  if (L[0] === "---") {
    const end = L.indexOf("---", 1);
    if (end > 0) { out.push(`<div class="mfront">${L.slice(1, end).map(l => { const [k, ...v] = l.split(":"); return v.length ? `<div><span>${esc(k.trim())}</span>${esc(v.join(":").trim())}</div>` : `<div>${esc(l)}</div>`; }).join("")}</div>`); i = end + 1; }
  }
  const para = [];
  const flush = () => { if (para.length) out.push(`<p>${mdInline(para.join(" "))}</p>`); para.length = 0; };
  while (i < L.length) {
    const l = L[i];
    let m;
    if (!l.trim()) { flush(); i++; continue; }
    if ((m = l.match(/^\s*(```|~~~)\s*(\S*)/))) {
      flush(); const fence = m[1], body = [];
      for (i++; i < L.length && !L[i].trim().startsWith(fence); i++) body.push(L[i]);
      out.push(`<pre${m[2] ? ` data-lang="${esc(m[2])}"` : ""}><code>${esc(body.join("\n"))}</code></pre>`); i++; continue;
    }
    if ((m = l.match(/^(#{1,6})\s+(.*?)\s*#*\s*$/))) { flush(); const n = m[1].length; out.push(`<h${n}>${mdInline(m[2])}</h${n}>`); i++; continue; }
    if (/^\s*([-*_])(\s*\1){2,}\s*$/.test(l)) { flush(); out.push("<hr>"); i++; continue; }
    if (/^\s*>/.test(l)) {
      flush(); const q = [];
      for (; i < L.length && /^\s*>/.test(L[i]); i++) q.push(L[i].replace(/^\s*>\s?/, ""));
      out.push(`<blockquote>${md2html(q.join("\n"))}</blockquote>`); continue;
    }
    if (/^\s*\|.*\|\s*$/.test(l) && i + 1 < L.length && /^\s*\|?[\s:|-]+\|?\s*$/.test(L[i + 1]) && L[i + 1].includes("-")) {
      flush(); const cells = r => r.trim().replace(/^\||\|$/g, "").split("|").map(c => c.trim());
      const head = cells(l), rows = [];
      for (i += 2; i < L.length && /^\s*\|.*\|\s*$/.test(L[i]); i++) rows.push(cells(L[i]));
      out.push(`<div class="mtab"><table><thead><tr>${head.map(c => `<th>${mdInline(c)}</th>`).join("")}</tr></thead><tbody>${rows.map(r => `<tr>${r.map(c => `<td>${mdInline(c)}</td>`).join("")}</tr>`).join("")}</tbody></table></div>`);
      continue;
    }
    if (/^\s*([-*+]|\d+[.)])\s+/.test(l)) {
      flush(); const items = [];
      for (; i < L.length && (/^\s*([-*+]|\d+[.)])\s+/.test(L[i]) || (/^\s{2,}\S/.test(L[i]) && items.length)); i++) {
        const mm = L[i].match(/^(\s*)([-*+]|\d+[.)])\s+(.*)$/);
        if (mm) items.push({ d: Math.floor(mm[1].replace(/\t/g, "  ").length / 2), ol: /\d/.test(mm[2]), t: mm[3] });
        else items[items.length - 1].t += " " + L[i].trim();
      }
      let html = "", stack = [];
      for (const it of items) {
        while (stack.length > it.d + 1) html += `</li></${stack.pop()}>`;
        if (stack.length === it.d + 1) html += "</li>";
        while (stack.length < it.d + 1) { const tag = it.ol ? "ol" : "ul"; html += `<${tag}>`; stack.push(tag); }
        const task = it.t.match(/^\[([ xX])\]\s+(.*)$/);
        html += `<li${task ? ' class="task"' : ""}>${task ? `<span class="box">${task[1].trim() ? "✓" : ""}</span>${mdInline(task[2])}` : mdInline(it.t)}`;
      }
      while (stack.length) html += `</li></${stack.pop()}>`;
      out.push(html); continue;
    }
    para.push(l.trim()); i++;
  }
  flush();
  return out.join("\n");
}
// open one note: rendered, with its vault and path above it and a way to Finder
async function openNote(path, title, where) {
  const pop = overlay("mdpop", `aria-labelledby="mdt"`, `<div class="mhead"><div><h3 id="mdt"></h3><code id="mdp"></code></div>
      <div class="mact"><button class="ghost" id="mdf">Show in Finder</button><button class="ghost" id="mdx" aria-label="Close">Close · Esc</button></div></div>
    <article class="md" id="mdbody" tabindex="0"></article>`);
  $("mdt").textContent = title; $("mdp").textContent = where;
  $("mdbody").innerHTML = `<p class="muted">loading…</p>`;
  $("mdf").onclick = () => kReveal("what=vault&path=" + encodeURIComponent(path)).then(ok => { $("mdf").textContent = ok ? "Shown in Finder" : "Could not open"; });
  $("mdx").onclick = () => pop.close();
  pop.hidden = false; $("mdbody").scrollTop = 0; $("mdbody").focus();
  try {
    const r = await fetch("/api/note?path=" + encodeURIComponent(path));
    $("mdbody").innerHTML = r.ok ? md2html(await r.text()) : `<p class="muted">This note could not be read (${r.status}).</p>`;
  } catch (e) { $("mdbody").innerHTML = `<p class="muted">This note could not be read: ${esc(String(e))}</p>`; }
}
