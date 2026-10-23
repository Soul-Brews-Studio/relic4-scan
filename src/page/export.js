// ---------- export: the metadata of what this view shows, as CSV or JSON ----------
// small and flat on purpose (one line per session, file, home or scan); over EXPORT_MAX lines the page offers the
// index's own tables instead (OUT/meta: sessions, files and every .jsonl, as Parquet) by opening that folder
const EXPORT_MAX = 20000;
const xtime = t => t == null ? "" : bkk(t);
function exportSet() {
  const q = $("q").value.trim();
  if (VIEW === "map" || VIEW === "tree") {
    const ks = []; for (let k = 0; k < N; k++) if (match[k]) ks.push(k);
    return { what: "sessions", q, cols: ["sid", "tier", "repo", "project", "started_bkk", "last_bkk", "rows", "user", "assistant", "thinking", "tool_calls", "tool_results", "tool_errors", "copies", "parent_sid", "homes"],
      n: ks.length, rows: () => ks.map(k => [S.sid[k], TIER[S.tier[k]], cat(k), proj(k), xtime(S.t0[k]), xtime(S.t1[k]), S.rows[k], S.user[k], S.asst[k], S.think[k],
        S.calls[k], S.results[k], S.errors[k], S.copies[k], S.parent[k] >= 0 ? S.sid[S.parent[k]] : "", S.homes[k].map(h => D.homes[h].path).join(" ; ")]) };
  }
  if (VIEW === "paths" && P) {
    const is = []; for (let i = 0; i < PN; i++) if (pm[i]) is.push(i);
    return { what: "files", q, cols: ["path", "status", "verdict", "size_bytes", "modified_bkk", "rows"], n: is.length,
      rows: () => is.map(i => [P.dirs[P.f.d[i]] + "/" + P.f.n[i], STATUS[pstat(i)][0], P.verdicts[P.f.v[i]] ?? "", P.f.size[i], xtime(P.f.mt[i]), P.f.rows[i] >= 0 ? P.f.rows[i] : ""]) };
  }
  if (VIEW === "scans" && LOG.length) return { what: "scans", q: "", n: LOG.length,
    cols: ["id", "job", "at_bkk", "new", "grown", "rewritten", "gone", "unchanged", "files", "bytes", "secs", "range", "rows"],
    rows: () => LOG.map(e => [e.i, e.job, xtime(e.at), e.new, e.grown, e.rewritten, e.gone, e.unchanged, e.files, e.bytes, e.secs.toFixed(1), e.range || "", e.rows ?? ""]) };
  // the dashboards: the homes (banks), with their sizes and newest session
  return { what: "homes", q: "", n: D.homes.length, cols: ["path", "agent", "origin", "files", "bytes", "rows"],
    rows: () => D.homes.map(h => [h.path, h.agent || "", h.origin, h.files, h.bytes, h.rows]) };
}
const csvCell = v => { const s = String(v ?? ""); return /[",\n]/.test(s) ? `"${s.replace(/"/g, '""')}"` : s; };
function exportBody(x, fmt) {
  const rows = x.rows(), stamp = new Date().toISOString();
  if (fmt === "csv") return [x.cols.join(","), ...rows.map(r => r.map(csvCell).join(","))].join("\n") + "\n";
  return JSON.stringify({ exported_at: stamp, view: VIEW, what: x.what, query: x.q, out: D.out, built: D.built, version: "relic-scan",
    count: rows.length, items: rows.map(r => Object.fromEntries(x.cols.map((c, j) => [c, r[j]]))) }, null, 1);
}
// save one export: to ~/Downloads through the server (shown in Finder), or as a browser download from a file page
async function xsave(x, f) {
  const name = `relic-scan-${VIEW}-${x.what}-${new Date().toISOString().slice(0, 16).replace(/[:T]/g, "-")}.${f}`, body = exportBody(x, f);
  if (!SERVED) { const a = document.createElement("a"); a.href = URL.createObjectURL(new Blob([body])); a.download = name; a.click(); return "Downloaded by the browser."; }
  const r = await fetch("/api/save?name=" + encodeURIComponent(name), { method: "POST", headers: { "X-Relic": "1" }, body });
  const out = await r.json();
  return r.ok ? `Saved ${human(body.length)} to ${out.path.replace(D.home || "\u0000", "~")} (shown in Finder).` : "Not saved: " + out.error;
}
function openExport() {
  const x = exportSet(), big = x.n > EXPORT_MAX;
  const pop = overlay("xpop", `aria-labelledby="xpt"`, `<h3 id="xpt"></h3><p class="what" id="xwhat"></p><p class="cols" id="xcols"></p>
    <div class="actions" id="xact"></div><div class="msg" id="xmsg" role="status"></div>`);
  $("xpt").textContent = `Export ${x.what}`;
  $("xwhat").innerHTML = `<b>${fmt(x.n)} ${x.what}</b>${x.q ? ` matching <code>${esc(x.q)}</code>` : ""} · one line each · exported with the time, the view and the query`;
  $("xcols").textContent = x.cols.join(" · ");
  $("xact").innerHTML = (big ? "" : `<button class="go" data-f="csv">Save CSV</button><button class="ghost" data-f="json">Save JSON</button>`)
    + (SERVED ? `<button class="ghost" id="xmeta">Open the meta folder</button>` : "");
  $("xmsg").className = "msg" + (big ? " big" : "");
  $("xmsg").textContent = big ? `${fmt(x.n)} is more than ${fmt(EXPORT_MAX)} lines: narrow the query, or open the meta folder for the full tables (sessions, files, every .jsonl, as Parquet).`
    : SERVED ? "Saved to ~/Downloads and shown in Finder." : "Downloaded by the browser.";
  for (const b of $("xact").querySelectorAll("[data-f]")) b.onclick = async () => { $("xmsg").className = "msg"; $("xmsg").textContent = await xsave(x, b.dataset.f); };
  if ($("xmeta")) $("xmeta").onclick = () => kReveal("what=meta").then(ok => { $("xmsg").textContent = ok ? "The meta folder is open in Finder." : "Could not open the meta folder."; });
  pop.hidden = false; ($("xact").querySelector("button") || $("xpt")).focus();
}
$("xport").onclick = openExport;
