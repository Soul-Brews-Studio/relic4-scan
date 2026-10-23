// ---------- view ----------
const cv = $("c"), ctx = cv.getContext("2d");
let W, H, DPR, SIDE = 0, scale, ox, oy, sel = -1, hover = -1;
const tierOn = [true, true, true];
let colorBy = "tier";
function fit() {
  SIDE = parseFloat(getComputedStyle(document.documentElement).getPropertyValue("--side")) || 0;
  DPR = window.devicePixelRatio || 1; W = innerWidth - 360 - SIDE; H = innerHeight;
  cv.width = innerWidth * DPR; cv.height = H * DPR; cv.style.width = innerWidth + "px"; cv.style.height = H + "px";
}
function home() { scale = Math.min(W, H - 60) / (2 * (ROUT + 190)); ox = SIDE + W / 2; oy = H / 2 + 22; }
const sx = k => ox + X[k] * scale, sy = k => oy + Y[k] * scale;
function hue(k) {
  if (colorBy === "repo") return `hsl(${(S.cat[k] * 137.508) % 360} 75% ${LIGHT ? 42 : 70}%)`;
  if (colorBy === "tools%") {
    // share of the session's rows that are tool traffic: text-heavy = warm, tool-heavy = cold
    const f = S.rows[k] ? tools(k) / S.rows[k] : 0;
    return `hsl(${40 + 170 * f} 85% ${(LIGHT ? 46 : 72) - 12 * f}%)`;
  }
  if (S.think && colorBy === "thinking") return S.think[k] > 0 ? TYPES[2][2] : INK.off;
  if (colorBy === "age") {
    const t = S.t1[k] ?? S.t0[k]; if (t == null) return "#555";
    const age = (Date.now() / 1000 - t) / 86400;
    return age < 2 ? INK.hit : age < 14 ? TYPES[0][2] : age < 60 ? TYPES[3][2] : LIGHT ? "#8a93a8" : "#5a6a9a";
  }
  return COLOR[S.tier[k]];
}
let grid = new Map();
function draw() {
  ctx.setTransform(DPR, 0, 0, DPR, 0, 0);
  ctx.clearRect(0, 0, innerWidth, H);
  // rings + month labels
  ctx.lineWidth = 1; ctx.font = "11px ui-monospace, Menlo, monospace"; ctx.textAlign = "center";
  for (let i = 0; i <= RINGS; i++) {
    const r = (R0 + i * DR) * scale;
    ctx.strokeStyle = (m0 + i) % 12 === 0 ? INK.year : INK.ring; // a stronger ring where a year starts
    ctx.beginPath(); ctx.arc(ox, oy, r, 0, 2 * Math.PI); ctx.stroke();
    if (i < RINGS && DR * scale > 14) { ctx.fillStyle = INK.month; ctx.fillText(mLabel(m0 + i), ox, oy - (R0 + (i + 0.5) * DR) * scale + 4); }
  }
  // repo labels at the rim
  ctx.fillStyle = INK.label;
  let shown = 0;
  for (const [c, [s0, w, n]] of sector) {
    if (w * (ROUT + 30) * scale < 26 && shown > 8) continue;
    const a = s0 + w / 2, r = (ROUT + 22) * scale;
    ctx.save(); ctx.translate(ox + r * Math.cos(a), oy + r * Math.sin(a));
    let rot = a; ctx.textAlign = "left";
    if (Math.cos(a) < 0) { rot += Math.PI; ctx.textAlign = "right"; }
    ctx.rotate(rot); ctx.fillText(`${secName(c)} ${n}`, 0, 4); ctx.restore(); shown++;
  }
  // constellation lines for a small query result, and for the selection
  const few = active && countMatched < 4000;
  ctx.lineWidth = 0.6;
  if (few) {
    ctx.strokeStyle = INK.link; ctx.beginPath();
    for (let k = 0; k < N; k++) if (match[k] && S.parent[k] >= 0 && match[S.parent[k]]) { ctx.moveTo(sx(k), sy(k)); ctx.lineTo(sx(S.parent[k]), sy(S.parent[k])); }
    ctx.stroke();
  }
  if (sel >= 0) {
    const root = S.parent[sel] >= 0 ? S.parent[sel] : sel;
    ctx.strokeStyle = INK.sel; ctx.beginPath();
    for (const k of kids[root]) { ctx.moveTo(sx(root), sy(root)); ctx.lineTo(sx(k), sy(k)); }
    ctx.stroke();
  }
  // stars
  ctx.globalCompositeOperation = INK.blend;
  grid = new Map();
  const zoom = Math.pow(scale / homeScale, 0.45);
  for (let k = 0; k < N; k++) {
    const x = sx(k), y = sy(k);
    if (x < SIDE - 10 || y < -10 || x > SIDE + W + 10 || y > H + 10) continue;
    if (!tierOn[S.tier[k]]) continue;
    const on = match[k] === 1;
    // unmatched satellites are skipped: 100 of them at 4% each around one parent add up to a bright false halo
    if (!on && S.tier[k] > 0) continue;
    ctx.globalAlpha = !on ? INK.alpha[2] : S.tier[k] === 0 ? INK.alpha[0] : INK.alpha[1];
    const r = Math.max(0.6, SIZE[k] * zoom);
    ctx.fillStyle = hue(k);
    if (r < 1.3) ctx.fillRect(x - r, y - r, 2 * r, 2 * r);
    else { ctx.beginPath(); ctx.arc(x, y, r, 0, 2 * Math.PI); ctx.fill(); }
    if (on || !active) {
      const g = `${Math.floor(x / 12)},${Math.floor(y / 12)}`;
      (grid.get(g) || grid.set(g, []).get(g)).push(k);
    }
  }
  ctx.globalCompositeOperation = "source-over"; ctx.globalAlpha = 1;
  for (const [k, c] of [[hover, INK.hover], [sel, INK.hit]]) if (k >= 0) {
    ctx.strokeStyle = c; ctx.lineWidth = 1.5; ctx.beginPath();
    ctx.arc(sx(k), sy(k), Math.max(5, SIZE[k] * zoom + 4), 0, 2 * Math.PI); ctx.stroke();
  }
}
let queued = false;
const redraw = () => { if (!queued) { queued = true; requestAnimationFrame(() => { queued = false; draw(); }); } };
function nearest(x, y) {
  let best = -1, bd = 100;
  const gx = Math.floor(x / 12), gy = Math.floor(y / 12);
  for (let i = -1; i <= 1; i++) for (let j = -1; j <= 1; j++) for (const k of grid.get(`${gx + i},${gy + j}`) || []) {
    const d = (sx(k) - x) ** 2 + (sy(k) - y) ** 2; if (d < bd) { bd = d; best = k; }
  }
  return best;
}

// ---------- panel ----------
let countMatched = N;
function bars(items, total, onclick) {
  return items.map(([label, n, extra]) => `<div class="row" data-q="${esc(onclick(label))}"><span>${esc(label)}</span><span class="n">${extra ?? fmt(n)}</span></div>
    <div class="bar"><i style="width:${Math.max(1, 100 * n / total)}%"></i></div>`).join("");
}
// one bar per type, its share of all rows (errors: share of results)
function typeBars(ty, rows) {
  return TYPES.map(([key, label, color], j) => {
    const base = key === "errors" ? ty[4] : rows, pct = base ? 100 * ty[j] / base : 0;
    return `<div class="row" data-q="${key}>0"><span><b style="display:inline-block;width:8px;height:8px;border-radius:50%;background:${color};margin-right:6px"></b>${label}</span>
      <span class="n">${fmt(ty[j])} · ${pct.toFixed(1)}%${key === "errors" ? " of results" : ""}</span></div>
      <div class="bar"><i style="width:${Math.max(0.5, pct)}%;background:${color}"></i></div>`;
  }).join("");
}
function summary() {
  if (sel >= 0) return detail(sel);
  let n = 0, rows = 0, t = [0, 0, 0], lo = Infinity, hi = -Infinity;
  const ty = TYPES.map(() => 0);
  const byCat = new Map(), byHome = new Map();
  for (let k = 0; k < N; k++) if (match[k]) {
    n++; rows += S.rows[k]; t[S.tier[k]]++;
    TYPES.forEach(([key], j) => { ty[j] += S[key][k]; });
    if (S.t0[k] != null) { lo = Math.min(lo, S.t0[k]); hi = Math.max(hi, S.t1[k] ?? S.t0[k]); }
    byCat.set(cat(k), (byCat.get(cat(k)) || 0) + 1);
    for (const h of S.homes[k]) byHome.set(h, (byHome.get(h) || 0) + 1);
  }
  countMatched = n;
  $("count").textContent = active ? `${fmt(n)} / ${fmt(N)} sessions` : `${fmt(N)} sessions`;
  const topCats = [...byCat.entries()].sort((a, b) => b[1] - a[1]).slice(0, 18);
  const topHomes = [...byHome.entries()].sort((a, b) => b[1] - a[1]).slice(0, 16)
    .map(([h, c]) => { const x = D.homes[h]; return [x.path.replace(/^\/Users\/[^/]+/, "~"), c, `${fmt(c)} · ${(x.bytes / 1e9).toFixed(2)} GB · ${x.origin}`]; });
  $("panel").innerHTML = `
    <h2>${active ? "query result" : "all sessions"} · ${esc(D.agent)}</h2>
    <div class="kv">
      <span>sessions</span><span>${fmt(n)}</span>
      <span>main</span><span>${fmt(t[0])}</span>
      <span>subagent</span><span>${fmt(t[1])}</span>
      <span>workflow</span><span>${fmt(t[2])}</span>
      <span>rows</span><span>${fmt(rows)}</span>
      <span>span (BKK)</span><span>${lo < Infinity ? bkk(lo).slice(0, 10) + " → " + bkk(hi).slice(0, 10) : "—"}</span>
    </div>
    <h2>rows by type</h2>${typeBars(ty, rows)}
    <h2>repos</h2>${bars(topCats, topCats[0]?.[1] || 1, l => `repo:${l}`)}
    <h2>jsonl homes (sessions · size · origin)</h2>${bars(topHomes, topHomes[0]?.[1] || 1, l => `home:${l.replace(/^~/, "")}`)}
    <h2>query</h2>
    <div class="help">words match repo, project and sid. <code>repo:</code> <code>proj:</code> <code>sid:</code> <code>home:</code>
    <code>tier:main|sub|wf</code> <code>month:2026-07</code> <code>after:2026-09-01</code> <code>before:</code>
    <code>rows&gt;1000</code> <code>text</code> <code>msgs</code> <code>copies</code>
    <code>user</code> <code>asst</code> <code>think</code> <code>calls</code> <code>results</code> <code>errors</code> <code>tools</code>, e.g. <code>think&gt;0</code> <code>errors&gt;=20</code>. A leading <code>-</code> negates. Terms AND.
    Click a star for its session, drag to pan, wheel to zoom, double-click to reset.</div>
    <div class="help" style="margin-top:8px">built ${esc(D.built)} · ${fmt(D.homes.length)} homes · rings = month started (BKK), inside = oldest</div>`;
  for (const el of $("panel").querySelectorAll("[data-q]")) el.onclick = () => addTerm(el.dataset.q);
}
// click a repo, a home, a status or a folder: add it to the query (quoted when it has spaces)
function addTerm(raw) {
  const ci = raw.indexOf(":");
  const term = raw.startsWith('"') || !/\s/.test(raw) ? raw : ci < 0 ? `"${raw}"` : `${raw.slice(0, ci)}:"${raw.slice(ci + 1)}"`;
  const q = $("q").value.trim();
  $("q").value = (q ? q + " " : "") + term;
  run($("q").value);
}
function detail(k) {
  const root = S.parent[k] >= 0 ? S.parent[k] : k, sid = S.sid[k];
  const homes = S.homes[k].map(h => `${esc(D.homes[h].path.replace(/^\/Users\/[^/]+/, "~"))} <span class="muted">(${D.homes[h].origin})</span>`).join("<br>");
  const ks = kids[root].slice().sort((a, b) => (S.t0[a] ?? 0) - (S.t0[b] ?? 0));
  $("panel").innerHTML = `
    <h2>${TIER[S.tier[k]]} session</h2>
    <div class="kv">
      <span>sid</span><span><code>${esc(sid)}</code></span>
      <span>repo</span><span>${esc(cat(k))}</span>
      <span>project</span><span>${esc(proj(k) || "—")}</span>
      <span>started</span><span>${bkk(S.t0[k])}</span>
      <span>last</span><span>${bkk(S.t1[k])}</span>
      <span>rows</span><span>${fmt(S.rows[k])} (text ${fmt(S.text[k])} · msgs ${fmt(S.msgs[k])})</span>
      ${TYPES.map(([key, label, color]) => `<span style="color:${color}">${label}</span><span>${fmt(S[key][k])}</span>`).join("")}
      <span>copies</span><span>${S.copies[k]} · ${(S.bytes[k] / 1e6).toFixed(1)} MB largest</span>
      <span>homes</span><span>${homes || "—"}</span>
      ${root !== k ? `<span>parent</span><span class="row" data-k="${root}"><code>${esc(S.sid[root])}</code></span>` : ""}
    </div>
    <button class="btn" data-copy="${esc(sid)}">copy sid</button>
    ${S.tier[k] === 0 ? `<button class="btn" data-copy="claude --resume ${esc(sid)}">copy resume</button>` : ""}
    <button class="btn" id="back">back</button>
    <h2>${root === k ? "satellites" : "siblings"} · ${ks.length}</h2>
    ${ks.slice(0, 300).map(c => `<div class="row" data-k="${c}"><span style="color:${COLOR[S.tier[c]]}">${esc(S.sid[c].slice(0, 22))}</span><span class="n">${fmt(S.rows[c])}</span></div>`).join("")}`;
  for (const el of $("panel").querySelectorAll("[data-k]")) el.onclick = () => select(+el.dataset.k);
  for (const el of $("panel").querySelectorAll("[data-copy]")) el.onclick = () => navigator.clipboard.writeText(el.dataset.copy).then(() => { el.textContent = "copied"; });
  $("back").onclick = () => select(-1);
}
function select(k) { sel = k; summary(); if (VIEW === "tree") renderTree(); else redraw(); }
