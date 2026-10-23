// ---------- disk: every .jsonl the scan saw, by folder and size (data: /paths.json) ----------
// rings from the folder in the centre outwards, four deep; a slice's angle is its share of that folder; the children of
// a folder that are each under 0.5% of the centre fold into one grey "smaller objects" slice, as DaisyDisk draws it
let DISK = null, DFOCUS = null;
function dashDisk() {
  return `<h1>Disk${rangeChip()}</h1>
    <p class="sub">Every .jsonl by folder and size. Click a slice to go in, the centre to go back.</p>${scopeLine()}
    <div class="dcrumbs" id="dcr"></div><div class="disk"><div id="dsv"><div class="muted">${SERVED ? "loading every path…" : "The disk view needs the server: open this page through relic-scan serve or the app."}</div></div><div class="dl" id="dlg"></div></div>`;
}
function diskLoad() {
  if (DISK) { DFOCUS = DFOCUS || DISK; return diskRender(); }
  if (!SERVED) return;
  fetch("/paths.json").then(r => r.json()).then(P => {
    // a folder tree of every file; chains of single folders are drawn as one ("~/.claude/projects")
    const root = { name: "every .jsonl", path: "", kids: new Map(), size: 0, files: 0, idx: 0, ok: 0 };
    for (let i = 0; i < P.f.n.length; i++) {
      const dir = P.dirs[P.f.d[i]], size = P.f.size[i], indexed = P.f.pl[i] === 0 && P.f.ok[i] ? 1 : 0;
      const t = dir.replace(D.home || "\u0000", "~");
      let node = root, acc = "";
      node.size += size; node.files++; node.ok += indexed ? size : 0;
      for (const seg of t.split("/").filter(Boolean)) {
        acc += (acc || t.startsWith("/") ? "/" : "") + seg;
        let k = node.kids.get(seg);
        // a folder at the top of an absolute path keeps its leading slash ("/opt", then "/opt/Code" once squashed)
        if (!k) { k = { name: node === root && t.startsWith("/") ? "/" + seg : seg, path: acc, kids: new Map(), size: 0, files: 0, ok: 0, parent: node }; node.kids.set(seg, k); }
        node = k; node.size += size; node.files++; node.ok += indexed ? size : 0;
      }
      node.kids.set(" " + i, { name: P.f.n[i], path: acc + "/" + P.f.n[i], file: true, size, files: 1, ok: indexed ? size : 0, parent: node, kids: new Map() });
    }
    const squash = n => { for (const k of n.kids.values()) squash(k);
      while (!n.file && n !== root && n.kids.size === 1) { const [c] = n.kids.values(); if (c.file) break; n.name += "/" + c.name; n.path = c.path; n.kids = c.kids; for (const g of n.kids.values()) g.parent = n; } };
    squash(root);
    // open at the first folder that branches: a root holding only "~" says nothing
    let start = root;
    while (start.kids.size === 1) { const [c] = start.kids.values(); if (c.file) break; start = c; }
    DISK = root; DFOCUS = start; diskRender();
  }).catch(e => { $("dsv").innerHTML = `<div class="muted">could not load /paths.json (${esc(String(e))})</div>`; });
}
const dabs = n => n.path.replace(/^~/, D.home || "~");
function dkids(n, min) {
  const all = [...n.kids.values()].filter(k => k.size > 0).sort((a, b) => b.size - a.size), keep = [];
  let rest = 0, restN = 0;
  for (const k of all) if (k.size >= min) keep.push(k); else { rest += k.size; restN++; }
  if (rest) keep.push({ name: `${restN.toLocaleString()} smaller objects…`, size: rest, rest: true, kids: new Map(), parent: n });
  return keep;
}
function diskRender() {
  const f = DFOCUS, R = [70, 140, 200, 250, 292], segs = [], min = f.size * 0.005, hue = new Map();
  // each branch gets its own hue at the first ring where folders split, and keeps it outwards, lighter as it goes
  const walk = (n, depth, a0, a1, h) => {
    if (depth > 4) return;
    const kids = dkids(n, min), split = h == null && kids.filter(k => !k.rest).length > 1;
    let a = a0, i = 0;
    for (const c of kids) {
      const w = (a1 - a0) * c.size / Math.max(1, n.size), ch = split ? (i * 47 + 200) % 360 : h;
      if (w > 0.003) { segs.push([c, depth, a, a + w, ch ?? 210]); if (!c.rest && !c.file) walk(c, depth + 1, a, a + w, ch); }
      a += w; i++;
    }
  };
  walk(f, 1, 0, 2 * Math.PI, null);
  const col = (c, d, h) => c.rest ? (LIGHT ? "#d6dbe7" : "#262d3d") : `hsl(${h} ${c.file ? 45 : 65}% ${LIGHT ? 62 - d * 6 : 46 + d * 7}%)`;
  const centre = n => `<text class="ct" text-anchor="middle" y="0">${human(n.size)}</text>
    <text class="cs" text-anchor="middle" y="26">${esc((n.name || "").length > 22 ? n.name.slice(0, 21) + "…" : n.name)}</text>`;
  $("dsv").innerHTML = `<svg viewBox="-300 -300 600 600" role="img" aria-label="${esc(f.name)}: ${human(f.size)}">
    ${segs.map(([c, d, a0, a1, h], i) => `<path data-s="${i}" class="${c.rest ? "rest" : ""}" d="${arc(R[d - 1] + 1, R[d], a0, a1)}" fill="${col(c, d, h)}"><title>${esc(c.path || c.name)} · ${human(c.size)}</title></path>`).join("")}
    <circle class="hole" r="${R[0] - 1}"/><g class="mid">${centre(f)}</g></svg>`;
  const svg = $("dsv").firstElementChild;
  for (const el of svg.querySelectorAll("path[data-s]")) {
    const [c] = segs[+el.dataset.s];
    el.onmouseenter = () => { svg.classList.add("hov"); el.classList.add("on"); svg.querySelector(".mid").innerHTML = centre(c); };
    el.onmouseleave = () => { svg.classList.remove("hov"); el.classList.remove("on"); svg.querySelector(".mid").innerHTML = centre(f); };
    if (!c.rest) el.onclick = () => { if (c.file) kReveal("path=" + encodeURIComponent(dabs(c))); else { DFOCUS = c; diskRender(); } };
  }
  svg.querySelector(".hole").onclick = () => { if (f.parent) { DFOCUS = f.parent; diskRender(); } };
  // the path from the top down to the centre
  const chain = []; for (let n = f; n; n = n.parent) chain.unshift(n);
  $("dcr").innerHTML = chain.map((n, i) => `${i ? "<span>/</span>" : ""}<button data-i="${i}">${esc(n.name)}</button>`).join("");
  for (const b of $("dcr").querySelectorAll("button")) b.onclick = () => { DFOCUS = chain[+b.dataset.i]; diskRender(); };
  const kids = dkids(f, min);
  const hueOf = c => { const s = segs.find(x => x[0] === c); return s ? col(c, s[1], s[4]) : "var(--dim)"; };
  $("dlg").innerHTML = `<h2>${esc(f.name)}</h2>
    <div class="meta"><b>${human(f.size)}</b> · ${f.files.toLocaleString()} files · ${human(f.ok)} indexed${f.size > f.ok ? ` · ${human(f.size - f.ok)} not indexed` : ""}</div>
    ${SERVED && f.path ? `<div class="acts"><button class="ghost" id="dopen" style="min-width:0;padding:8px 14px;font-size:14px">Open folder</button></div>` : ""}
    ${kids.map((c, i) => `<${c.rest ? "div" : "button"} class="row ${c.rest ? "rest" : ""}" data-i="${i}"><span class="dot" style="background:${hueOf(c)}"></span>
      <span class="n">${esc(c.name)}${c.file ? "" : "/"}</span><span class="v">${human(c.size)}</span><span class="p">${(100 * c.size / Math.max(1, f.size)).toFixed(1)}%</span></${c.rest ? "div" : "button"}>`).join("")}`;
  if ($("dopen")) $("dopen").onclick = () => kReveal("path=" + encodeURIComponent(dabs(f)));
  for (const b of $("dlg").querySelectorAll("button.row")) {
    const c = kids[+b.dataset.i];
    b.onclick = () => { if (c.file) kReveal("path=" + encodeURIComponent(dabs(c))); else { DFOCUS = c; diskRender(); } };
    b.onmouseenter = () => { const s = segs.findIndex(x => x[0] === c); const el = svg.querySelector(`path[data-s="${s}"]`); if (el) el.onmouseenter(); };
    b.onmouseleave = () => { const s = segs.findIndex(x => x[0] === c); const el = svg.querySelector(`path[data-s="${s}"]`); if (el) el.onmouseleave(); };
  }
}
