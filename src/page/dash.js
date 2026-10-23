// ---------- dashboards: Smart Scan, Banks & Repos, Health (the Relic Studio shell, from this index's own numbers) ----------
const SC = D.scan || { discovered: 0, indexed: 0, skipped: 0, vault: 0, loose: 0, loose_transcripts: 0, unknown: 0, empty: 0, bytes: 0 };
const stat = (k, v, c, sub) => `<div class="stat"><div class="k">${k}</div><div class="v" style="color:${c}">${v}</div>${sub ? `<div class="s">${sub}</div>` : ""}</div>`;
function totals() {
  let rows = 0, t = [0, 0, 0], ty = TYPES.map(() => 0), undated = 0, orphans = 0, sats = 0;
  const months = new Set();
  for (let k = 0; k < N; k++) {
    rows += S.rows[k]; t[S.tier[k]]++; TYPES.forEach(([key], j) => { ty[j] += S[key][k]; });
    if (S.t0[k] == null) undated++; else months.add(monthIdx(S.t0[k]));
    if (S.tier[k] > 0) { sats++; if (S.parent[k] < 0) orphans++; }
  }
  return { rows, t, ty, undated, orphans, sats, months: months.size };
}
function dashScan() {
  const file = location.protocol === "file:";
  const T = totals(), cov = SC.indexed / Math.max(1, SC.indexed + SC.loose_transcripts);
  return `<h1>Smart Scan${rangeChip()}</h1>
    <p class="sub">Every agent transcript on this machine. Each run reads only what is new.</p>${scopeLine()}
    <div class="cols"><div class="main">
    <div class="hero"><button class="donut" id="sunmini" aria-label="Rows by kind. Open the full contents view"></button><div class="kl" id="sunleg"></div></div>
    <div class="strip five">${(() => {
      // what the index holds and how fresh it is; the file accounting lives in Index controls and Health
      let lo = Infinity, hi = -Infinity; for (let k = 0; k < N; k++) if (S.t0[k] != null) { lo = Math.min(lo, S.t0[k]); hi = Math.max(hi, S.t1[k] ?? S.t0[k]); }
      const months = lo < Infinity ? monthIdx(hi) - monthIdx(lo) + 1 : 0, mon = t => new Date((t + BKK) * 1000).toLocaleString("en", { month: "short", year: "numeric", timeZone: "UTC" });
      return stat("Rows", fmt(T.rows), "var(--ink)", `<span>≈${Math.round(T.rows / Math.max(1, SC.indexed)).toLocaleString()} per file</span><span>≈${Math.round(T.rows / Math.max(1, N)).toLocaleString()} per session</span>`)
        + stat("Sessions", fmt(N), "var(--ink)", `<span>${fmt(T.t[0])} main</span><span>${fmt(T.t[1])} sub · ${fmt(T.t[2])} workflow</span>`)
        + stat("Files indexed", fmt(SC.indexed), "var(--ink)", `<span>${human(SC.bytes)}</span><span>${(100 * cov).toFixed(2)}% of transcripts</span>`)
        + stat("Newest row", hi > -Infinity ? agoText(Date.now() / 1000 - hi) : "—", "var(--ink)", hi > -Infinity ? `<span>${bkk(hi).slice(0, 10)}</span><span>${bkk(hi).slice(11)} Bangkok</span>` : "")
        + stat("Span", months ? `${months} months` : "—", "var(--ink)", lo < Infinity ? `<span>${mon(lo)} →</span><span>${mon(hi)}</span>` : "");
    })()}</div>
    <div class="card"><h3>Index controls</h3>
      <div>The last run indexed <b>${fmt(SC.indexed)}</b> transcript files (${human(SC.bytes)}) out of <b>${fmt(SC.discovered)}</b> .jsonl discovered.
        ${fmt(SC.skipped)} in agent homes are not transcripts${SC.before_range ? `; ${fmt(SC.before_range)} were last written before ${esc(D.range.from)}, so they were not read` : ""}; ${fmt(SC.vault + SC.loose)} sit outside every home.</div>
      <div class="actions"><button class="go" id="dgo" ${SERVED ? "" : "disabled title='open it through relic-scan serve or the app'"}>Index now</button><button class="ghost" id="dchk" ${SERVED ? "" : "disabled"}>Check changes</button>${SERVED ? `<button class="link" id="dout">Show output</button><button class="link stop" id="dstop" hidden>Stop</button>` : ""}</div>
      <div class="track"><i id="dbar"></i></div><div class="diff" id="ddiff"></div>
      <div class="note" id="dmsg">Check changes scans and compares with the index; it changes nothing in it and only adds a line to the scan log. Index now runs the same scan, then reads only what changed.</div></div>
    </div><aside class="side">
    <h3>Recent scans</h3><div class="recent" id="srecent"><div class="muted">${SERVED ? "loading…" : "open it through relic-scan serve or the app"}</div></div>
    <a class="more" href="${file ? "?view=scans" : "/scans"}">Scan log →</a></aside></div>`;
}
function homeStats() {
  const H = D.homes.map(() => ({ n: 0, last: 0 }));
  for (let k = 0; k < N; k++) for (const h of S.homes[k]) { H[h].n++; H[h].last = Math.max(H[h].last, S.t1[k] ?? S.t0[k] ?? 0); }
  return H;
}
const orgRepo = k => { // "-opt-Code-github-com-Soul-Brews-Studio-mawjs-oracle" + cat "mawjs-oracle" -> "Soul-Brews-Studio/mawjs-oracle"
  const p = proj(k), c = cat(k), i = p.toLowerCase().indexOf("github-com-");
  if (i < 0) return p;
  const rest = p.slice(i + 11), j = rest.toLowerCase().lastIndexOf("-" + c.toLowerCase());
  return j > 0 ? rest.slice(0, j) + "/" + rest.slice(j + 1) : rest;
};
let repoCap = 30;
function dashBanks() {
  const H = homeStats(), order = D.homes.map((_, i) => i).sort((a, b) => D.homes[b].rows - D.homes[a].rows);
  const name = p => p.replace(/^\/Users\/[^/]+/, "~");
  const banks = order.map(i => { const h = D.homes[i];
    return `<div class="bank" data-go="/tree#${encodeURIComponent(`home:"${h.path}"`)}">
      <div class="n">⌂ ${esc(name(h.path).split("/").slice(-2).join("/"))}<span class="badge">${h.origin}</span></div><div class="p">${esc(name(h.path))}</div>
      <div class="nums"><div><b>${fmt(H[i].n)}</b><span>SESSIONS</span></div><div><b>${fmt(h.rows)}</b><span>ROWS</span></div>
        <div><b>${fmt(h.files)}</b><span>FILES</span></div><div><b>${human(h.bytes)}</b><span>SIZE</span></div></div>
      <div class="note">Newest session: ${H[i].last ? agoText(Date.now() / 1000 - H[i].last) : "—"}</div></div>`; }).join("");
  const R = new Map();
  for (let k = 0; k < N; k++) { const c = cat(k); const r = R.get(c) || R.set(c, { n: 0, rows: 0, k }).get(c); r.n++; r.rows += S.rows[k]; if (S.tier[k] === 0) r.k = k; }
  const repos = [...R].sort((a, b) => b[1].rows - a[1].rows);
  const list = repos.slice(0, repoCap).map(([c, r]) => `<div class="repo" data-go="/map#repo:${encodeURIComponent(/\s/.test(c) ? '"' + c + '"' : c)}">
      <span style="color:var(--cyan)">▰</span><div class="r"><b>${esc(c)}</b><code>${esc(orgRepo(r.k))}</code></div>
      <div class="x"><b>${fmt(r.n)}</b><div>sessions · ${fmt(r.rows)} rows</div></div></div>`).join("");
  return `<h1>Banks &amp; Repositories${rangeChip()}</h1>
    <p class="sub">Where your transcripts live, and which repositories hold the most.</p>${scopeLine()}
    <div class="grid2">${banks}</div>
    <div class="card" style="margin-top:22px"><h3 class="c">Repository index · ${fmt(repos.length)}</h3>${list}
      ${repos.length > repoCap ? `<div class="note"><a href="#" id="rmore" style="color:var(--cyan)">show ${fmt(Math.min(100, repos.length - repoCap))} more</a></div>` : ""}</div>`;
}
function dashHealth() {
  const T = totals(), errPct = T.ty[4] ? 100 * T.ty[5] / T.ty[4] : 0, age = D.last_run ? Date.now() / 1000 - D.last_run : Infinity;
  const mirrors = D.homes.filter(h => h.origin !== "own").length;
  const UR = D.unreadable || { count: 0, folders: [] };
  const checks = [
    [UR.count ? "warn" : "ok", UR.count ? `${fmt(UR.count)} folders could not be read (no permission), so a transcript inside them is never seen. On macOS, give the terminal or ARRA Relic Scan Full Disk Access to include them.`
      : "Every folder under the roots could be read.", UR.count ? ["#unreadable", "see them"] : null],
    [age > 86400 ? "warn" : "ok", age === Infinity ? "Never indexed." : `Last run ${agoText(age)}${age > 86400 ? ": transcripts written since then are not in the index yet" : ""}.`, age > 86400 ? ["/scan", "index"] : null],
    [SC.loose_transcripts ? "warn" : "ok", SC.loose_transcripts ? `${fmt(SC.loose_transcripts)} .jsonl outside every agent home look like transcripts. They are listed, not indexed (usually test fixtures).` : "No transcript sits outside an agent home.", SC.loose_transcripts ? ["/paths#-is:indexed verdict:transcript", "see them"] : null],
    [T.orphans / Math.max(1, T.sats) > 0.01 ? "warn" : "ok", `${fmt(T.sats - T.orphans)} of ${fmt(T.sats)} subagent and workflow sessions found their main session${T.orphans ? `; ${fmt(T.orphans)} have none on disk` : ""}.`, null],
    [T.undated ? "info" : "ok", T.undated ? `${fmt(T.undated)} sessions carry no timestamp; their rows are in the undated month.` : "Every session has a timestamp.", null],
    [SC.empty ? "info" : "ok", SC.empty ? `${fmt(SC.empty)} indexed files produced no rows (empty or summary-only sessions).` : "Every indexed file produced rows.", SC.empty ? ["/paths#is:indexed rows=0", "see them"] : null],
    ...(SC.before_range ? [["info", `This folder holds only ${esc(D.range.human)}: ${fmt(SC.before_range)} files last written before it were not read. The full index is a run without --month, --day, --from or --to.`, null]] : []),
    ["info", `${fmt(SC.skipped)} files in agent homes are not transcripts (workflow journals, prompt logs, omx logs) and were skipped by the content sniff.`, ["/paths#is:skipped", "see them"]],
    ["info", `${fmt(SC.unknown)} files had no recognised shape and were indexed anyway: a transcript-shaped path is never dropped.`, SC.unknown ? ["/paths#verdict:unknown", "see them"] : null],
    ["info", `${mirrors} of ${D.homes.length} homes are mirrors, rescues or copies; the same line read twice is folded into one row.`, ["/banks", "banks"]],
    ["info", `Tool errors: ${fmt(T.ty[5])}, ${errPct.toFixed(1)}% of tool results.`, ["/map#errors>=20", "sessions with 20+"]],
  ];
  const warns = checks.filter(c => c[0] === "warn").length;
  const L = { ok: ["var(--green)", "✓"], warn: ["var(--amber)", "!"], info: ["var(--cyan)", "i"] };
  return `<h1>Health${rangeChip()}</h1>
    <p class="sub">Checks computed from this index. Nothing is estimated.</p>${scopeLine()}
    <div class="card head"><span class="big" style="color:${warns ? "var(--amber)" : "var(--green)"}">${warns ? "⚠" : "✓"}</span>
      <div><b style="font-size:18px">${warns ? `${warns} warning${warns > 1 ? "s" : ""} reported` : "All clear"}</b>
      <div class="note" style="margin:2px 0 0">Page built ${esc(D.built)} · ${esc(D.out || "")}</div></div></div>
    <div class="strip">${stat("Rows", fmt(T.rows), "var(--cyan)")}${stat("Sessions", fmt(N), "var(--green)")}${stat("Months", fmt(T.months), "var(--amber)")}${stat("Warnings", warns, warns ? "var(--amber)" : "var(--green)")}</div>
    <div class="card"><h3>Checks</h3>${checks.map(([lv, txt, link]) => `<div class="warn"><span class="lvl" style="background:${L[lv][0]}">${L[lv][1]}</span>
      <div>${txt}</div>${link ? `<a href="${link[0]}">${link[1]} →</a>` : "<span></span>"}</div>`).join("")}</div>
    ${UR.count ? `<div class="card" id="unreadable"><h3>Folders the scan could not read</h3>
      <div class="note" style="margin:0 0 12px">${fmt(UR.count)} in all${UR.folders.length < UR.count ? `; the first ${fmt(UR.folders.length)} below` : ""}. From the last run.</div>
      <div class="ulist">${UR.folders.map(p => `<code>${esc(p.replace(D.home || "\u0000", "~"))}</code>`).join("")}</div></div>` : ""}`;
}
function renderDash() {
  $("dash").innerHTML = VIEW === "scan" ? dashScan() : VIEW === "scans" ? dashScans() : VIEW === "disk" ? dashDisk() : VIEW === "settings" ? dashSettings() : VIEW === "banks" ? dashBanks() : dashHealth();
  if (VIEW === "scans") loadScans();
  if (VIEW === "disk") diskLoad();
  if (VIEW === "settings") loadSettings();
  if (VIEW === "scan") contentsMini();
  for (const el of $("dash").querySelectorAll("[data-go]")) el.onclick = () => { location.href = el.dataset.go; };
  const more = $("rmore"); if (more) more.onclick = e => { e.preventDefault(); repoCap += 100; renderDash(); };
  if ($("dgo")) {
    $("dgo").onclick = () => $("idx").click();
    $("dchk").onclick = () => window.relicCheck && window.relicCheck();
    if ($("dout")) $("dout").onclick = () => window.relicTerm && window.relicTerm();
    if ($("dstop")) $("dstop").onclick = () => window.relicStop && window.relicStop();
  }
}
// the last check (a dry run in the server): what Index now would read
function showCheck(c) {
  if (!$("ddiff") || !c) return;
  const n = x => (x || 0).toLocaleString(), ago = agoText(Date.now() / 1000 - c.at);
  const head = c.files ? `Index now would read <b>${n(c.files)}</b> files, <b>${(c.bytes / 1e9).toFixed(2)} GB</b>.`
    : `<b>Nothing changed</b> since the last index.`;
  $("ddiff").innerHTML = `${head} <span class="muted">Checked ${ago}, scan ${c.scan_secs.toFixed(1)}s.</span>
    <div class="nums"><span><b>${n(c.new)}</b> new</span><span><b>${n(c.grown)}</b> grown</span><span><b>${n(c.rewritten)}</b> rewritten</span>
    <span><b>${n(c.gone)}</b> gone</span><span><b>${n(c.unchanged)}</b> unchanged</span>${c.before_range ? `<span><b>${n(c.before_range)}</b> before the range</span>` : ""}
    ${c.log != null && (c.files || c.gone) ? `<a class="more" href="/scans#${c.log}">see the files →</a>` : ""}</div>`;
}
