// ---------- tree: jsonl home -> repo -> session -> satellites ----------
// a session sits under every home that holds a copy of it; counts are over the sessions the query matches
const roots = new Map(); // "home|cat" -> root sessions (mains, and satellites whose parent is unknown)
for (let k = 0; k < N; k++) if (!(S.tier[k] > 0 && S.parent[k] >= 0))
  for (const h of S.homes[k]) { const key = h + "|" + S.cat[k]; (roots.get(key) || roots.set(key, []).get(key)).push(k); }
const open = new Set(), cap = new Map();
let treeAuto = false;
const newAgg = () => ({ n: 0, rows: 0, ty: [0, 0, 0, 0, 0, 0], last: 0 });
function addAgg(a, k) {
  a.n++; a.rows += S.rows[k]; TYPES.forEach(([key], j) => { a.ty[j] += S[key][k]; });
  a.last = Math.max(a.last, S.t1[k] ?? S.t0[k] ?? 0);
}
// five segments (user, assistant, thinking, calls, results) as shares of the rows
function typeBar(ty, rows) {
  if (!rows) return `<span class="tb"></span>`;
  return `<span class="tb" title="${TYPES.map(([, l], j) => `${l} ${fmt(ty[j])}`).join(" · ")}">`
    + TYPES.slice(0, 5).map(([, , c], j) => `<i style="width:${100 * ty[j] / rows}%;background:${c}"></i>`).join("") + `</span>`;
}
function tnode(key, depth, label, a, count) {
  const opn = open.has(key) ? "▾" : "▸";
  return `<div class="tn${key === "s:" + sel ? " sel" : ""}" data-key="${key}" style="padding-left:${6 + depth * 18}px">
    <span class="nm">${opn} ${label}</span><span class="c">${count ?? fmt(a.n) + " sess"}</span><span class="c">${fmt(a.rows)} rows</span>
    ${typeBar(a.ty, a.rows)}<span class="c">${a.last ? bkk(a.last).slice(0, 10) : "—"}</span></div>`;
}
function sessRow(k, depth) {
  const ks = kids[k].filter(c => match[c]);
  const a = newAgg(); addAgg(a, k);
  const label = `<span style="color:${COLOR[S.tier[k]]}">${esc(S.sid[k].slice(0, 36))}</span> <span class="tw">${bkk(S.t0[k]).slice(0, 16)}</span>`;
  let h = tnode("s:" + k, depth, label, a, ks.length ? `${fmt(ks.length)} sat` : "—");
  if (open.has("s:" + k)) for (const c of ks.sort((x, y) => (S.t0[x] ?? 0) - (S.t0[y] ?? 0))) {
    const b = newAgg(); addAgg(b, c);
    h += tnode("s:" + c, depth + 1, `<span style="color:${COLOR[S.tier[c]]}">${esc(S.sid[c].slice(0, 36))}</span> <span class="tw">${TIER[S.tier[c]]}</span>`, b, "");
  }
  return h;
}
function renderTree() {
  // a home: filter (a bank opened from Banks & Repos) shows that home only, not every home holding a copy
  const terms = parseQuery($("q").value), homeWant = terms.filter(t => t.key === "home" && !t.neg).map(t => t.val.toLowerCase());
  const repoWant = terms.filter(t => (t.key === "repo" || t.key === "cat") && !t.neg).map(t => t.val);
  const homeOk = h => homeWant.every(v => (D.homes[h].path + " " + D.homes[h].origin).toLowerCase().includes(v));
  const H = new Map(), HR = new Map();
  for (let k = 0; k < N; k++) if (match[k]) for (const h of S.homes[k]) {
    if (!homeOk(h)) continue;
    addAgg(H.get(h) || H.set(h, newAgg()).get(h), k);
    const key = h + "|" + S.cat[k];
    addAgg(HR.get(key) || HR.set(key, newAgg()).get(key), k);
  }
  // a small result opens itself down to the repos
  if (active && countMatched <= 400 && !treeAuto) { for (const key of HR.keys()) { open.add("h:" + key.split("|")[0]); open.add("r:" + key); } treeAuto = true; }
  if (!active) treeAuto = false;
  // one bank: open it down to its repos
  if (homeWant.length && H.size <= 3) for (const h of H.keys()) open.add("h:" + h);
  // where you are: the bank or repo the query came from, and the way back
  const home1 = homeWant.length && H.size === 1 ? D.homes[[...H.keys()][0]] : null, tot = [...H.values()].reduce((t, a) => t + a.n, 0);
  let html = home1 ? `<div class="where"><a href="${kHref("banks")}">Banks &amp; Repos</a><span>›</span><b>${esc(home1.path.replace(D.home || "\u0000", "~"))}</b>
      <span class="muted">${home1.origin} · ${fmt(tot)} sessions · ${human(home1.bytes)}</span><a class="x" href="${kHref("tree")}" title="every home">clear ×</a></div>`
    : repoWant.length ? `<div class="where"><a href="${kHref("banks")}">Repositories</a><span>›</span><b>${esc(repoWant.join(" · "))}</b>
      <span class="muted">${fmt(tot)} sessions</span><a class="x" href="${kHref("tree")}" title="every repo">clear ×</a></div>` : "";
  html += `<div class="tn th"><span>jsonl home → repo → session → satellites</span><span class="c">sessions</span><span class="c">rows</span><span>by type</span><span class="c">last (BKK)</span></div>`;
  for (const [h, a] of [...H].sort((x, y) => y[1].rows - x[1].rows)) {
    const home = D.homes[h];
    html += tnode("h:" + h, 0, `${esc(home.path.replace(/^\/Users\/[^/]+/, "~"))} <span class="tw">${home.origin} · ${fmt(home.files)} files · ${(home.bytes / 1e9).toFixed(2)} GB</span>`, a);
    if (!open.has("h:" + h)) continue;
    const repos = [...HR].filter(([key]) => key.startsWith(h + "|")).sort((x, y) => y[1].rows - x[1].rows);
    for (const [key, b] of repos) {
      const c = +key.split("|")[1];
      html += tnode("r:" + key, 1, esc(c < 0 ? "(none)" : D.cats[c]), b);
      if (!open.has("r:" + key)) continue;
      const list = (roots.get(key) || []).filter(k => match[k] || kids[k].some(x => match[x]))
        .sort((x, y) => (S.t0[y] ?? 0) - (S.t0[x] ?? 0));
      const n = cap.get(key) || 200;
      for (const k of list.slice(0, n)) html += sessRow(k, 2);
      if (list.length > n) html += `<div class="more" data-more="${key}" style="padding-left:${6 + 2 * 18}px">… ${fmt(list.length - n)} more (newest first)</div>`;
    }
  }
  $("tree").innerHTML = html;
}
$("tree").addEventListener("click", e => {
  if (VIEW === "paths") {
    const pmore = e.target.closest("[data-pmore]");
    if (pmore) { const id = +pmore.dataset.pmore; pcap.set(id, (pcap.get(id) || 200) + 1000); return prender(); }
    const n = e.target.closest("[data-key]"); if (!n) return;
    const [t, v] = n.dataset.key.split(":");
    if (t === "f") { psel = +v; psummary(); return prender(); }
    if (popen.has(+v)) popen.delete(+v); else popen.add(+v);
    return prender();
  }
  const more = e.target.closest("[data-more]");
  if (more) { cap.set(more.dataset.more, (cap.get(more.dataset.more) || 200) + 500); return renderTree(); }
  const n = e.target.closest("[data-key]"); if (!n) return;
  const key = n.dataset.key;
  if (open.has(key)) open.delete(key); else open.add(key);
  if (key.startsWith("s:")) select(+key.slice(2)); else renderTree();
});
