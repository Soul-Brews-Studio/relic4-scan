// ---------- ⌘K: search what the page holds (views, actions, sessions, repos, homes, files, scans, filters) ----------
// every result opens something; some also carry actions (copy the resume command, export, check or index just that path)
let KP = null, KSEL = 0, KITEMS = [], KMORE = new Set(), KPATHS = null;
// a rank for how well `q` matches `s`: 0 starts it, 1 starts a word in it, 2 is inside it, 3 is a scattered match
function kScore(s, q) {
  s = s.toLowerCase();
  if (s.startsWith(q)) return 0;
  const i = s.indexOf(q);
  if (i > 0) return /[\s\/\-_.:·]/.test(s[i - 1]) ? 1 : 2;
  let j = 0; for (const c of s) if (c === q[j]) j++;
  return j === q.length ? 3 : null;
}
const kMark = (s, q) => { const i = q ? s.toLowerCase().indexOf(q) : -1;
  return i < 0 ? esc(s) : esc(s.slice(0, i)) + "<mark>" + esc(s.slice(i, i + q.length)) + "</mark>" + esc(s.slice(i + q.length)); };
const kMonths = k => { if (S.t0[k] == null) return ["undated"]; const a = monthIdx(S.t0[k]), b = monthIdx(S.t1[k] ?? S.t0[k]), out = [];
  for (let m = a; m <= b && out.length < 24; m++) out.push(mLabel(m)); return out; };
// show a file or folder in Finder (the server checks that the index holds it)
const kReveal = q => relicPost("/api/reveal?" + q).then(r => r.ok);
// a session's own file, from /paths.json (by name: <sid>.jsonl); the first copy outside mirrors and archives
let KBYNAME = null;
const kFileOf = k => { if (!KPATHS || !KPATHS.f) return null;
  if (!KBYNAME) { KBYNAME = new Map(); for (let i = 0; i < KPATHS.f.n.length; i++) { const n = KPATHS.f.n[i], p = KPATHS.dirs[KPATHS.f.d[i]] + "/" + n;
    const old = KBYNAME.get(n); if (!old || (/mirror|archive|_bak/.test(old) && !/mirror|archive|_bak/.test(p))) KBYNAME.set(n, p); } }
  return KBYNAME.get(S.sid[k] + ".jsonl") || null; };
const kTilde = p => p.replace(D.home || "\u0000", "~");
const kRun = async (path, dry) => {
  if (!SERVED) return;
  const r = await relicPost(`/api/${dry ? "check" : "run"}?only=${encodeURIComponent(path)}`);
  if (window.relicTerm) window.relicTerm();
  return r.ok;
};
function kGroups(qRaw) {
  const q = qRaw.trim().toLowerCase(), G = [];
  const add = (name, items, cap = 6) => { if (items.length) G.push({ name, items, cap }); };
  const views = [["scan", "Smart Scan"], ["banks", "Banks & Repos"], ["health", "Health"], ["scans", "Scan log"], ["map", "Session map"], ["tree", "Tree"], ["paths", "Paths"], ["disk", "Disk"], ["settings", "Settings"]];
  const rank = (arr, text) => arr.map(x => [x, q ? kScore(text(x), q) : 0]).filter(([, r]) => r != null).sort((a, b) => a[1] - b[1]).map(([x]) => x);
  add("Go to", rank(views, v => v[1]).map(([v, l]) => ({ t: l, d: VIEW === v ? "you are here" : "", go: () => { location.href = kHref(v); } })));
  const acts = [
    SERVED && { t: "Index now", d: "scan, then read only what changed", go: () => $("idx").click() },
    SERVED && { t: "Check changes", d: "scan and compare; the index stays as it is", go: () => window.relicCheck && window.relicCheck() },
    SERVED && { t: "Show output", d: "the server's output during a run or check", go: () => window.relicTerm && window.relicTerm() },
    { t: "Contents", d: "rows by kind, file kind, place, agent and month", go: () => VIEW === "scan" ? openContents() : (location.href = kHref("scan", "contents")) },
    { t: "Switch theme", d: "auto → dark → light", go: () => $("theme").click() },
  ].filter(Boolean);
  add("Do", rank(acts, a => a.t));
  // a filter: the query language of the map and the tree
  if (/[a-z]+(:|>=|<=|>|<|=)\S/i.test(q)) {
    G.push({ name: "Filter", cap: 2, items: [
      { t: `Sessions on the map: ${qRaw.trim()}`, d: "the star map, filtered", go: () => { location.href = kHref("map", qRaw.trim()); } },
      { t: `Sessions in the tree: ${qRaw.trim()}`, d: "home → repo → session, filtered", go: () => { location.href = kHref("tree", qRaw.trim()); } }] });
  }
  // sessions: by id, project or repo; an empty box shows the latest
  let ses = [];
  if (!q) { const idx = [...Array(N).keys()].filter(k => S.tier[k] === 0 && S.t1[k] != null).sort((a, b) => S.t1[b] - S.t1[a]); ses = idx.slice(0, 60); }
  else if (q.length >= 2) {
    const hits = [];
    for (let k = 0; k < N; k++) { const r = Math.min(...[S.sid[k], proj(k), cat(k)].map(x => kScore(x, q) ?? 9)); if (r < 9 && !(r === 3 && q.length < 4)) hits.push([k, r]); }
    hits.sort((a, b) => a[1] - b[1] || S.tier[a[0]] - S.tier[b[0]] || S.rows[b[0]] - S.rows[a[0]]); ses = hits.slice(0, 60).map(h => h[0]);
  }
  add(q ? "Sessions" : "Latest sessions", ses.map(k => { const f = kFileOf(k);
    return { t: `${cat(k)} · ${S.sid[k].slice(0, 8)}`, mt: q,
      d: `${TIER[S.tier[k]]} · ${S.t0[k] != null ? bkk(S.t1[k] ?? S.t0[k]).slice(0, 10) : "undated"} · ${S.rows[k].toLocaleString()} rows${f ? " · " + kTilde(f) : ""}`,
      go: () => { location.href = kHref("tree", "sid:" + S.sid[k]); },
      acts: [SERVED && ["open folder", () => kReveal("sid=" + encodeURIComponent(S.sid[k]))],
             S.tier[k] === 0 && ["copy resume", () => navigator.clipboard.writeText(`claude --resume ${S.sid[k]}`)]].filter(Boolean) }; }));
  if (q.length >= 2) {
    const repos = new Map(); for (let k = 0; k < N; k++) { const c = S.cat[k]; const r = repos.get(c) || repos.set(c, [0, 0]).get(c); r[0]++; r[1] += S.rows[k]; }
    add("Repos", rank([...repos].filter(([c]) => c >= 0), ([c]) => D.cats[c]).slice(0, 60).map(([c, [n, rows]]) => ({
      t: D.cats[c], mt: q, d: `${n.toLocaleString()} sessions · ${rows.toLocaleString()} rows`,
      go: () => { location.href = kHref("tree", `repo:"${D.cats[c]}"`); } })));
    add("Homes", rank(D.homes.map((h, i) => [h, i]), ([h]) => h.path + " " + h.origin + " " + (h.agent || "")).slice(0, 40).map(([h]) => ({
      t: kTilde(h.path), mt: q, d: `${h.agent || "agent?"} · ${h.origin} · ${h.files.toLocaleString()} files · ${h.rows.toLocaleString()} rows`,
      go: () => { location.href = kHref("tree", `home:"${h.path}"`); },
      acts: [SERVED && ["open folder", () => kReveal("path=" + encodeURIComponent(h.path))],
             SERVED && ["check", () => kRun(h.path, true)], SERVED && ["index", () => kRun(h.path, false)]].filter(Boolean) })));
  }
  if (q.length >= 3 && KPATHS) {
    const hits = [];
    for (let i = 0; i < KPATHS.f.n.length && hits.length < 400; i++) { const p = KPATHS.dirs[KPATHS.f.d[i]] + "/" + KPATHS.f.n[i]; if (p.toLowerCase().includes(q)) hits.push(i); }
    const st = i => KPATHS.f.pl[i] === 0 ? (KPATHS.f.ok[i] ? "indexed" : "skipped") : KPATHS.f.pl[i] === 1 ? "vault" : "loose";
    add("Files", hits.map(i => { const p = KPATHS.dirs[KPATHS.f.d[i]] + "/" + KPATHS.f.n[i], tp = p.replace(D.home || "\u0000", "~");
      return { t: tp, mt: q, d: `${st(i)} · ${(KPATHS.f.size[i] / 1e6).toFixed(1)} MB${KPATHS.f.rows[i] >= 0 ? ` · ${KPATHS.f.rows[i].toLocaleString()} rows` : ""}`,
        go: () => { location.href = kHref("paths", `"${tp}"`); },
        acts: [SERVED && ["open folder", () => kReveal("path=" + encodeURIComponent(p))], ["copy path", () => navigator.clipboard.writeText(p)],
               SERVED && ["check", () => kRun(p, true)], SERVED && ["index", () => kRun(p, false)]].filter(Boolean) }; }));
  } else if (q.length >= 3 && SERVED && KPATHS === null) {
    KPATHS = false; fetch("/paths.json").then(r => r.json()).then(d => { KPATHS = d; kRender(); }).catch(() => {});
    G.push({ name: "Files", cap: 1, items: [{ t: "loading every .jsonl path…", d: "", go: () => {} }] });
  }
  if (KSCANS && (q.startsWith("#") || q.length >= 3)) {
    const want = q.replace(/^#/, "");
    add("Scans", KSCANS.filter(e => String(e.i) === want || `${e.job} ${when(e.at)}`.includes(want)).slice(0, 30).map(e => ({
      t: `#${e.i} · ${e.job} · ${when(e.at)}`, d: scanWhat(e), go: () => { location.href = kHref("scans", String(e.i)); } })));
  }
  return G;
}
let KSCANS = null;
function kRender() {
  const q = $("kq").value.trim().toLowerCase(), G = kGroups($("kq").value);
  KITEMS = [];
  let html = "";
  for (const g of G) {
    const all = KMORE.has(g.name), shown = all ? g.items : g.items.slice(0, g.cap);
    html += `<div class="grp">${esc(g.name)}${g.items.length > g.cap ? ` · ${g.items.length.toLocaleString()}${g.items.length >= 60 ? "+" : ""}` : ""}</div>`;
    for (const it of shown) {
      const n = KITEMS.push(it) - 1;
      html += `<div class="it" data-n="${n}" role="option"><div><div class="t">${it.mt ? kMark(it.t, it.mt) : esc(it.t)}</div>${it.d ? `<div class="d">${esc(it.d)}</div>` : ""}</div>
        <div class="acts">${(it.acts || []).map((a, j) => `<button data-n="${n}" data-a="${j}" tabindex="-1">${esc(a[0])}</button>`).join("")}</div></div>`;
    }
    if (!all && g.items.length > g.cap) html += `<div class="more" data-g="${esc(g.name)}">show all in ${esc(g.name)}</div>`;
  }
  $("kres").innerHTML = html || `<div class="none">Nothing matches “${esc(q)}”. Try a repo, a session id, a path, or a filter like repo:neo month:2026-10.</div>`;
  KSEL = Math.min(KSEL, Math.max(0, KITEMS.length - 1));
  kMark2();
  for (const el of $("kres").querySelectorAll(".it")) { el.onclick = () => kGo(+el.dataset.n); el.onmousemove = () => { KSEL = +el.dataset.n; kMark2(); }; }
  for (const b of $("kres").querySelectorAll(".acts button")) b.onclick = e => { e.stopPropagation(); kAct(+b.dataset.n, +b.dataset.a, b); };
  for (const m of $("kres").querySelectorAll(".more")) m.onclick = () => { KMORE.add(m.dataset.g); kRender(); };
}
function kMark2() {
  for (const el of $("kres").querySelectorAll(".it")) el.classList.toggle("on", +el.dataset.n === KSEL);
  const on = $("kres").querySelector(".it.on"); if (on) on.scrollIntoView({ block: "nearest" });
}
function kGo(n) { const it = KITEMS[n]; if (!it) return; kClose(); it.go(); }
function kAct(n, a, btn) {
  const it = KITEMS[n]; if (!it || !it.acts || !it.acts[a]) return;
  const r = it.acts[a][1]();
  if (btn) { const old = btn.textContent; btn.textContent = "done"; setTimeout(() => { btn.textContent = old; }, 1200); }
  return r;
}
function kOpen() {
  if (!KP) {
    KP = overlay("kpal", `aria-label="Search"`, `
      <div class="in"><svg viewBox="0 0 24 24" aria-hidden="true">${ICONS.search}</svg><input id="kq" placeholder="Search sessions, repos, homes, files, scans, or type a filter like repo:neo" autocomplete="off" spellcheck="false" role="combobox" aria-controls="kres"></div>
      <div class="res" id="kres" role="listbox"></div>
      <div class="ft"><span><kbd>↑</kbd> <kbd>↓</kbd> move</span><span><kbd>↵</kbd> open</span><span><kbd>⌘↵</kbd> open folder</span><span><kbd>esc</kbd> close</span></div>`);
    $("kq").oninput = () => { KSEL = 0; KMORE.clear(); kRender(); };
    $("kq").onkeydown = e => {
      if (e.key === "ArrowDown") { e.preventDefault(); KSEL = Math.min(KITEMS.length - 1, KSEL + 1); kMark2(); }
      else if (e.key === "ArrowUp") { e.preventDefault(); KSEL = Math.max(0, KSEL - 1); kMark2(); }
      else if (e.key === "Enter" && (e.metaKey || e.ctrlKey)) { e.preventDefault(); kAct(KSEL, 0, $("kres").querySelector(`.acts button[data-n="${KSEL}"]`)); }
      else if (e.key === "Enter") { e.preventDefault(); kGo(KSEL); }
    };
  }
  if (SERVED && KPATHS === null) { KPATHS = false; fetch("/paths.json").then(r => r.json()).then(d => { KPATHS = d; KBYNAME = null; if (!KP.hidden) kRender(); }).catch(() => {}); }
  if (SERVED && !KSCANS) fetch("/api/scans").then(r => r.json()).then(d => { KSCANS = d.entries; if (!KP.hidden) kRender(); }).catch(() => {});
  KP.hidden = false; $("kq").value = ""; KSEL = 0; KMORE.clear(); kRender(); $("kq").focus();
}
function kClose() { if (KP) KP.hidden = true; }
$("ksearch").onclick = () => kOpen();
addEventListener("keydown", e => {
  const typing = /^(INPUT|TEXTAREA|SELECT)$/.test(document.activeElement && document.activeElement.tagName);
  if ((e.key === "k" || e.key === "K") && (e.metaKey || e.ctrlKey)) { e.preventDefault(); KP && !KP.hidden ? kClose() : kOpen(); }
  else if (e.key === "/" && !typing && (!KP || KP.hidden)) { e.preventDefault(); kOpen(); }
});
if (VIEW === "scan" && location.hash === "#contents") setTimeout(() => openContents(), 0);

// the dashboard views (Smart Scan, Banks & Repos, Health, Scan log) share one page: switching among them redraws in
// place, with no reload; history keeps back and forward working. The map, tree and paths still load as pages.
const SOFT = ["scan", "scans", "disk", "settings", "banks", "health"];
function softGo(v, push = true) {
  if (!DASH || !SOFT.includes(v) || location.protocol === "file:") return false;
  VIEW = v;
  if (push) history.pushState({ v }, "", "/" + v);
  tabs(); renderDash(); window.scrollTo(0, 0);
  if (window.relicRefresh) window.relicRefresh();
  return true;
}
document.addEventListener("click", e => {
  const a = e.target.closest && e.target.closest("#side a[href]");
  if (!a || e.metaKey || e.ctrlKey || e.shiftKey || e.button !== 0) return;
  const v = new URL(a.href, location.href).pathname.replace(/^\//, "");
  if (softGo(v)) e.preventDefault();
});
addEventListener("popstate", () => { const v = location.pathname.replace(/^\//, "") || "scan"; if (!softGo(v, false)) location.reload(); });
