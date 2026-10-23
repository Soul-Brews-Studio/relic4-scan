// ---------- the Export view: every metadata table of this index in one place, as CSV or JSON; and the month files ----------
// the tables are small (one line per session, file, home, scan or vault note); the rows themselves stay in the month
// files, listed below with their sizes, each one click from Finder
const xall = (what, cols, rows) => ({ what, q: "", cols, n: rows.length, rows: () => rows });
const XKINDS = [
  { what: "sessions", desc: "one line per session: repo, tier, start and end, rows by kind, copies, parent",
    load: async () => { const ks = [...Array(N).keys()];
      return xall("sessions", ["sid", "tier", "repo", "project", "started_bkk", "last_bkk", "rows", "user", "assistant", "thinking", "tool_calls", "tool_results", "tool_errors", "copies", "parent_sid", "homes"],
        ks.map(k => [S.sid[k], TIER[S.tier[k]], cat(k), proj(k), xtime(S.t0[k]), xtime(S.t1[k]), S.rows[k], S.user[k], S.asst[k], S.think[k],
          S.calls[k], S.results[k], S.errors[k], S.copies[k], S.parent[k] >= 0 ? S.sid[S.parent[k]] : "", S.homes[k].map(h => D.homes[h].path).join(" ; ")])); } },
  { what: "files", desc: "every .jsonl the scan saw: indexed, skipped, in a ψ vault or elsewhere, with its verdict", served: true,
    load: async () => { const P = await (await fetch("/paths.json")).json(), st = i => P.f.pl[i] === 0 ? (P.f.ok[i] ? "indexed" : "skipped") : P.f.pl[i] === 1 ? "vault" : "loose";
      return xall("files", ["path", "status", "verdict", "size_bytes", "modified_bkk", "rows"],
        P.f.n.map((n, i) => [P.dirs[P.f.d[i]] + "/" + n, st(i), P.verdicts[P.f.v[i]] ?? "", P.f.size[i], xtime(P.f.mt[i]), P.f.rows[i] >= 0 ? P.f.rows[i] : ""])); } },
  { what: "homes", desc: "every agent home (bank): agent, origin, files, size, rows",
    load: async () => xall("homes", ["path", "agent", "origin", "files", "bytes", "rows"], D.homes.map(h => [h.path, h.agent || "", h.origin, h.files, h.bytes, h.rows])) },
  { what: "scans", desc: "the scan log: every index and check, with what it found", served: true,
    load: async () => { const L = (await (await fetch("/api/scans")).json()).entries || [];
      return xall("scans", ["id", "job", "at_bkk", "new", "grown", "rewritten", "gone", "unchanged", "files", "bytes", "secs", "range", "rows"],
        L.map(e => [e.i, e.job, xtime(e.at), e.new, e.grown, e.rewritten, e.gone, e.unchanged, e.files, e.bytes, (e.secs || 0).toFixed(1), e.range || "", e.rows ?? ""])); } },
  { what: "ψ vaults", desc: "one line per vault: kind, notes, size, newest note, links; a vault's notes export from the ψ Vaults view", served: true,
    load: async () => { const V = VAULTS || ((await (await fetch("/vaults.json")).json()).vaults || []);
      return xall("vaults", ["name", "path", "kind", "origin", "notes", "files", "bytes", "newest_bkk", "links", "sections"],
        V.map(v => [v.name, v.path, v.kind, v.origin, v.notes, v.files, v.bytes, xtime(v.newest_ms / 1000), v.links.join(" ; "),
          Object.entries(v.sections).sort((a, b) => b[1] - a[1]).map(([k, n]) => `${k}:${n}`).join(" ; ")])); } },
];
function dashExport() {
  return `<h1>Export${rangeChip()}</h1>
    <p class="sub">Every table of metadata this index holds, as CSV or JSON, with the time it was exported. The rows themselves stay in the month files below.</p>${scopeLine()}
    <div class="card"><h3>Metadata</h3><div class="xtab">${XKINDS.map((k, i) => `<div class="xrow" data-i="${i}">
      <div><b>${esc(k.what)}</b><div class="note" style="margin:2px 0 0">${esc(k.desc)}</div></div>
      <span class="xn" id="xn${i}">${k.served && !SERVED ? "needs the server" : "…"}</span>
      <span class="xb">${k.served && !SERVED ? "" : `<button class="ghost" data-f="csv">CSV</button><button class="ghost" data-f="json">JSON</button>`}</span>
      <div class="msg" id="xm${i}" role="status"></div></div>`).join("")}</div></div>
    <div class="card"><h3>Month files <span class="muted" style="font-weight:400;font-size:13px">one Parquet file per Bangkok month · click a month to show it in Finder</span></h3>
      <div class="xact">${SERVED ? `<button class="ghost" data-reveal="out">Open the index folder</button><button class="ghost" id="xmetaf">Open the meta folder</button>` : ""}</div>
      <div id="xmonths"><div class="muted">${SERVED ? "loading…" : "Needs the server: open the page through relic-scan serve or the app."}</div></div></div>`;
}
const XSETS = [];
function exportLoad() {
  XKINDS.forEach((k, i) => {
    if (k.served && !SERVED) return;
    (XSETS[i] ||= k.load()).then(x => { $("xn" + i).textContent = `${fmt(x.n)} lines`; }).catch(() => { $("xn" + i).textContent = "not available"; });
    for (const b of document.querySelectorAll(`.xrow[data-i="${i}"] [data-f]`)) b.onclick = async () => {
      $("xm" + i).textContent = "saving…";
      try { $("xm" + i).textContent = await xsave(await XSETS[i], b.dataset.f); } catch (e) { $("xm" + i).textContent = "Not saved: " + e; }
    };
  });
  if ($("xmetaf")) $("xmetaf").onclick = () => kReveal("what=meta");
  if (!SERVED) return;
  fetch("/api/months").then(r => r.json()).then(j => {
    const M = j.months || [], tot = M.reduce((t, m) => t + m.bytes, 0), top = Math.max(1, ...M.map(m => m.bytes));
    const mname = m => new Date(m + "-01T00:00:00Z").toLocaleString("en-US", { month: "long", year: "numeric", timeZone: "UTC" });
    $("xmonths").innerHTML = M.length ? `<div class="note" style="margin:0 0 10px">${fmt(M.length)} months · ${human(tot)}</div>`
      + M.slice().reverse().map(m => `<button class="xmonth" data-m="${m.month}" title="show ${m.month}.parquet in Finder">
          <span class="m"><b>${m.month}</b> ${mname(m.month)}</span><span class="bar"><i style="width:${(100 * m.bytes / top).toFixed(1)}%"></i></span>
          <span class="z">${human(m.bytes)}</span><span class="t muted">written ${bkk(m.modified_ms / 1000).slice(0, 10)}</span></button>`).join("")
      : `<div class="note">No month files yet: press Index now.</div>`;
    for (const b of $("xmonths").querySelectorAll(".xmonth")) b.onclick = () => kReveal("what=month&m=" + b.dataset.m).then(ok => { b.querySelector(".t").textContent = ok ? "shown in Finder" : "could not open"; });
  }).catch(() => { $("xmonths").innerHTML = `<div class="note">Could not list the month files.</div>`; });
}
