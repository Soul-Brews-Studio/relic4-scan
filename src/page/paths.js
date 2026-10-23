// ---------- paths: every .jsonl the scan saw, as a folder tree (data from /paths.json, loaded only here) ----------
// status: indexed (parsed into the months) · skipped (in a home, but the sniff said not a transcript)
//         vault (in a ψ vault, not parsed) · loose (anywhere else under the roots, not parsed)
const STATUS = [["indexed", COLOR[1]], ["skipped", INK.hit], ["vault", COLOR[2]], ["loose", LIGHT ? "#8a93a8" : "#7c8496"]];
let P = null, PN = 0, pm, pdir, plow, pact = false, pcount = 0, psel = -1;
const nodes = []; // folder trie: { name, parent, kids: Map(name -> id), files: [] }
const popen = new Set([0]), pcap = new Map();
const pstat = i => P.f.pl[i] === 0 ? (P.f.ok[i] ? 0 : 1) : P.f.pl[i] === 1 ? 2 : 3;
const ptilde = s => P.home && s.startsWith(P.home) ? "~" + s.slice(P.home.length) : s;
const ppath = i => P.dirs[P.f.d[i]] + "/" + P.f.n[i];
let agg; // per trie node: [files, bytes, rows, indexed, skipped, vault, loose]
function loadPaths() {
  $("tree").innerHTML = `<div class="muted" style="padding:20px">loading paths…</div>`;
  fetch("/paths.json").then(r => { if (!r.ok) throw new Error(r.status); return r.json(); }).then(d => {
    P = d; PN = P.f.n.length; pm = new Uint8Array(PN);
    nodes.push({ name: "", parent: -1, kids: new Map(), files: [] });
    pdir = P.dirs.map(dp => {
      let id = 0;
      for (const part of dp.split("/").filter(Boolean)) {
        let k = nodes[id].kids.get(part);
        if (k == null) { k = nodes.length; nodes.push({ name: part, parent: id, kids: new Map(), files: [] }); nodes[id].kids.set(part, k); }
        id = k;
      }
      return id;
    });
    for (let i = 0; i < PN; i++) nodes[pdir[P.f.d[i]]].files.push(i);
    plow = Array.from({ length: PN }, (_, i) => (ptilde(ppath(i)) + " " + ppath(i)).toLowerCase());
    prun($("q").value);
  }).catch(e => {
    $("tree").innerHTML = `<div style="padding:20px">The paths view needs <code>relic-scan serve</code> (or <code>up</code>): it loads
      <code>/paths.json</code>, which a page opened as a file cannot read. (${esc(String(e))})</div>`;
  });
}
function ptest(i, t) {
  const v = t.val.toLowerCase();
  switch (t.key) {
    case "is": case "status": return STATUS[pstat(i)][0].startsWith(v);
    case "verdict": return (P.verdicts[P.f.v[i]] || "").includes(v);
    case "name": return P.f.n[i].toLowerCase().includes(v);
    case "size": return cmp(P.f.size[i], t.op, Number(v));
    case "mb": return cmp(P.f.size[i] / 1e6, t.op, Number(v));
    case "rows": return cmp(P.f.rows[i], t.op, Number(v));
    case "after": return bkk(P.f.mt[i]) >= v;
    case "before": return bkk(P.f.mt[i]) < v;
  }
  return true;
}
function prun(q) {
  if (!P) return;
  const terms = parseQuery(q);
  pact = terms.length > 0; pcount = 0;
  for (let i = 0; i < PN; i++) {
    let ok = true;
    for (const t of terms) if ((t.word != null ? plow[i].includes(t.word) : ptest(i, t)) === t.neg) { ok = false; break; }
    pm[i] = ok ? 1 : 0; pcount += ok;
  }
  agg = Array.from({ length: nodes.length }, () => [0, 0, 0, 0, 0, 0, 0]);
  for (let i = 0; i < PN; i++) if (pm[i]) {
    const st = pstat(i);
    for (let id = pdir[P.f.d[i]]; id >= 0; id = nodes[id].parent) {
      const a = agg[id]; a[0]++; a[1] += P.f.size[i]; a[2] += Math.max(0, P.f.rows[i]); a[3 + st]++;
    }
  }
  // a small result opens every folder it touches
  if (pact && pcount <= 3000) for (let id = 0; id < nodes.length; id++) if (agg[id][0]) popen.add(id);
  history.replaceState(null, "", location.pathname + location.search + (q ? "#" + encodeURIComponent(q) : ""));
  $("count").textContent = pact ? `${fmt(pcount)} / ${fmt(PN)} .jsonl` : `${fmt(PN)} .jsonl`;
  tabs(); psummary(); prender();
}
function statBar(a) {
  if (!a[0]) return `<span class="tb"></span>`;
  return `<span class="tb" title="${STATUS.map(([l], j) => `${l} ${fmt(a[3 + j])}`).join(" · ")}">`
    + STATUS.map(([, c], j) => `<i style="width:${100 * a[3 + j] / a[0]}%;background:${c}"></i>`).join("") + `</span>`;
}
function prow(key, depth, label, cols, sel) {
  return `<div class="tn${sel ? " sel" : ""}" data-key="${key}" style="padding-left:${6 + depth * 18}px"><span class="nm">${label}</span>${cols}</div>`;
}
function pnode(id, depth, out) {
  // fold a chain of folders that hold nothing but one sub-folder: ~/.claude/projects reads as one row
  let label = nodes[id].name, end = id;
  for (;;) {
    const kids = [...nodes[end].kids.values()].filter(k => agg[k][0]);
    if (kids.length !== 1 || nodes[end].files.some(i => pm[i])) break;
    end = kids[0]; label += "/" + nodes[end].name;
  }
  const a = agg[end], opn = popen.has(end);
  if (id === 0) label = label || "/";
  else if (nodes[id].parent === 0) label = ptilde("/" + label); // /Users/me/… reads ~/…
  out.push(prow("d:" + end, depth, `${opn ? "▾" : "▸"} ${esc(label)}`, `<span class="c">${fmt(a[0])} files</span><span class="c">${fmt(a[2])} rows</span>${statBar(a)}<span class="c">${human(a[1])}</span>`));
  if (!opn) return;
  const kids = [...nodes[end].kids.values()].filter(k => agg[k][0]).sort((x, y) => agg[y][0] - agg[x][0]);
  for (const k of kids) pnode(k, depth + 1, out);
  const files = nodes[end].files.filter(i => pm[i]).sort((x, y) => P.f.size[y] - P.f.size[x]);
  const n = pcap.get(end) || 200;
  for (const i of files.slice(0, n)) {
    const [st, c] = STATUS[pstat(i)];
    out.push(prow("f:" + i, depth + 1, `<span style="color:${c}">●</span> ${esc(P.f.n[i])} <span class="tw">${st}</span>`,
      `<span class="c">${P.f.rows[i] >= 0 ? fmt(P.f.rows[i]) + " rows" : "—"}</span><span class="c">${esc(P.verdicts[P.f.v[i]] || "")}</span><span class="c tw">${bkk(P.f.mt[i]).slice(0, 10)}</span><span class="c">${human(P.f.size[i])}</span>`, i === psel));
  }
  if (files.length > n) out.push(`<div class="more" data-pmore="${end}" style="padding-left:${6 + (depth + 1) * 18}px">… ${fmt(files.length - n)} more (biggest first)</div>`);
}
function prender() {
  const out = [`<div class="tn th"><span>folder → .jsonl (biggest first)</span><span class="c">files</span><span class="c">rows</span><span>status</span><span class="c">size</span></div>`];
  if (agg[0][0]) pnode(0, 0, out); else out.push(`<div class="muted" style="padding:12px">no .jsonl matches</div>`);
  $("tree").innerHTML = out.join("");
}
function psummary() {
  if (psel >= 0) return pdetail(psel);
  const a = agg[0], byV = new Map(), byDir = new Map();
  for (let i = 0; i < PN; i++) if (pm[i]) {
    const v = P.verdicts[P.f.v[i]] || "—"; byV.set(v, (byV.get(v) || 0) + 1);
    byDir.set(P.f.d[i], (byDir.get(P.f.d[i]) || 0) + 1);
  }
  const vs = [...byV].sort((x, y) => y[1] - x[1]);
  const ds = [...byDir].sort((x, y) => y[1] - x[1]).slice(0, 14).map(([d, c]) => [ptilde(P.dirs[d]), c]);
  $("panel").innerHTML = `
    <h2>${pact ? "query result" : "every .jsonl the scan saw"}</h2>
    <div class="kv">
      <span>files</span><span>${fmt(a[0])}</span>
      <span>size</span><span>${human(a[1])}</span>
      <span>rows</span><span>${fmt(a[2])} (indexed files)</span>
    </div>
    <h2>status</h2>${STATUS.map(([l, c], j) => `<div class="row" data-q="is:${l}"><span><b style="display:inline-block;width:8px;height:8px;border-radius:50%;background:${c};margin-right:6px"></b>${l}</span>
      <span class="n">${fmt(a[3 + j])}</span></div><div class="bar"><i style="width:${Math.max(0.5, a[0] ? 100 * a[3 + j] / a[0] : 0)}%;background:${c}"></i></div>`).join("")}
    <h2>verdict (content sniff)</h2>${bars(vs, vs[0]?.[1] || 1, l => `verdict:${l}`)}
    <h2>busiest folders</h2>${bars(ds, ds[0]?.[1] || 1, l => `"${l}/"`)}
    <h2>query</h2>
    <div class="help">words match the path (<code>~/…</code> or full). <code>is:indexed|skipped|vault|loose</code>
    <code>verdict:not-transcript</code> <code>name:agent-</code> <code>mb&gt;100</code> <code>size&gt;1e9</code> <code>rows&gt;10000</code>
    <code>rows=0</code> <code>after:2026-10-01</code> <code>before:</code>. A leading <code>-</code> negates. Terms AND.
    Click a folder to open it, a file for its details.</div>
    <div class="help" style="margin-top:8px">built ${esc(P.built)} · indexed = parsed into the months; skipped = in a home, but not a transcript;
    vault / loose = outside every home, listed only</div>`;
  for (const el of $("panel").querySelectorAll("[data-q]")) el.onclick = () => addTerm(el.dataset.q);
}
function pdetail(i) {
  const [st, c] = STATUS[pstat(i)], full = ppath(i);
  $("panel").innerHTML = `
    <h2>.jsonl</h2>
    <div class="kv">
      <span>name</span><span><code>${esc(P.f.n[i])}</code></span>
      <span>folder</span><span>${esc(ptilde(P.dirs[P.f.d[i]]))}</span>
      <span>status</span><span style="color:${c}">${st}</span>
      <span>verdict</span><span>${esc(P.verdicts[P.f.v[i]] || "—")}</span>
      <span>place</span><span>${["agent home", "ψ vault", "elsewhere"][P.f.pl[i]]}</span>
      <span>size</span><span>${human(P.f.size[i])} (${fmt(P.f.size[i])} B)</span>
      <span>rows</span><span>${P.f.rows[i] >= 0 ? fmt(P.f.rows[i]) : "— (not indexed)"}</span>
      <span>modified</span><span>${bkk(P.f.mt[i])} BKK</span>
    </div>
    <button class="btn" data-copy="${esc(full)}">copy path</button>
    <button class="btn" id="back">back</button>`;
  for (const el of $("panel").querySelectorAll("[data-copy]")) el.onclick = () => navigator.clipboard.writeText(el.dataset.copy).then(() => { el.textContent = "copied"; });
  $("back").onclick = () => { psel = -1; psummary(); prender(); };
}
