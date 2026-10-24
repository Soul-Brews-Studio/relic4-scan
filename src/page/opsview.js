// ---------- the Activity view: what the relic chain is doing right now, and what the hub says ----------
// one snapshot from the server (GET /api/ops, the same one `relic-scan ops --json` prints): the chain log's stage lines,
// the running processes, the hub's status and the S3 job. The page asks again every 2 seconds while it is open.
let opsTimer = 0;
const opsDur = s => { s = Math.max(0, Math.round(s)); return s >= 3600 ? `${Math.floor(s / 3600)}:${String(Math.floor(s / 60) % 60).padStart(2, "0")}:${String(s % 60).padStart(2, "0")}` : `${Math.floor(s / 60)}:${String(s % 60).padStart(2, "0")}`; };
function dashOps() {
  return `<h1>Activity</h1><p class="sub">What the relic chain is doing on this machine right now, and what the hub reports. It asks again every 2 seconds.</p>
    <div id="opsbody"><div class="muted">${SERVED ? "loading…" : "Needs the server: open the page through relic-scan serve or the app."}</div></div>`;
}
function opsDraw(j) {
  const c = j.chain || {}, st = c.state, run = st === "running";
  const head = st === "never" ? `<span class="s3chip warn">no pass logged yet</span> <span class="muted">${esc(j.log || "")}</span>`
    : run ? `<span class="s3chip ok">RUNNING</span> <b>${opsDur(c.elapsed_s)}</b> elapsed${c.typical_total_s ? ` · typical ${opsDur(c.typical_total_s)}${c.eta_s > 0 ? ` · about <b>${opsDur(c.eta_s)}</b> left` : " · over it now"}` : ""}`
    : `<span class="s3chip">idle</span> last pass ${esc((c.started || "").slice(11))} took ${opsDur(c.took_s || 0)} · next in about ${opsDur(c.next_in_s || 0)}`;
  const pct = run && c.typical_total_s ? Math.min(100, 100 * c.elapsed_s / c.typical_total_s) : st === "idle" ? 100 : 0;
  const rows = (c.stages || []).map(s => `<div class="opsrow ${s.state}"><span class="mk">${s.state === "done" ? "✓" : s.state === "running" ? "●" : "·"}</span><b>${esc(s.name)}</b>
    <span class="z">${s.secs != null ? Math.round(s.secs) + " s" : ""}</span><span class="z muted">${s.typical_s != null ? "typical " + Math.round(s.typical_s) + " s" : ""}</span><span class="d muted">${esc(s.detail || "")}</span></div>`).join("");
  const ps = (j.processes || []).map(p => `<div class="opsrow"><b>${esc(p.what)}</b><span class="muted">${esc(p.user)}</span><span class="z">${esc(p.elapsed)}</span><span class="z">${Math.round(p.cpu)}% cpu</span><span class="z">${fmt(p.rss_mb)} MB</span></div>`).join("");
  const h = j.hub || {};
  const hub = h.ok ? `<div class="note" style="margin:0 0 8px">load ${(h.load && h.load[0] || 0).toFixed(1)} of ${h.cpus} cpus · ${(h.running || []).length} job(s) running</div>`
      + (h.machines || []).map(m => `<div class="opsrow hub"><b>${esc(m.machine)}</b><span class="z">last push ${opsDur(m.push_age_s || 0)} ago</span><span class="d">${m.sync ? `<span class="s3chip ${/OK/.test(m.sync) ? "ok" : "bad"}">${esc(m.sync.slice(20, 28).trim() || m.sync.slice(0, 8))}</span> ${esc(m.sync.slice(0, 19))}` : ""}</span></div>`).join("")
      + (h.warnings || []).map(w => `<div class="note bad">${esc(w)}</div>`).join("")
    : `<div class="muted">not read (${esc(h.error || "")})</div>`;
  const s3 = j.s3 && j.s3.running ? `<div class="card"><h3>S3 copy</h3>${esc(j.s3.job)} ${j.s3.done}/${j.s3.total}<div class="s3bar"><i style="width:${(100 * j.s3.done / Math.max(1, j.s3.total)).toFixed(0)}%"></i></div></div>` : "";
  $("opsbody").innerHTML = `<div class="card"><h3>Relic chain ${c.machine ? `<span class="muted" style="font-weight:400;font-size:13px">${esc(c.machine)}</span>` : ""}</h3>
      <div>${head}</div><div class="s3bar"><i style="width:${pct.toFixed(0)}%"></i></div><div class="opstab">${rows}</div></div>
    <div class="card"><h3>Processes</h3>${ps || `<div class="muted">none running</div>`}</div>
    <div class="card"><h3>Hub <span class="muted" style="font-weight:400;font-size:13px">white, over the tailnet</span></h3>${hub}</div>${s3}`;
}
async function opsPoll() {
  clearTimeout(opsTimer);
  if (VIEW !== "ops" || !$("opsbody")) return;
  try { opsDraw(await (await fetch("/api/ops")).json()); } catch (e) { $("opsbody").innerHTML = `<div class="note bad">Could not read the activity: ${esc(e)}</div>`; }
  opsTimer = setTimeout(opsPoll, 2000);
}
const opsLoad = () => { if (SERVED) opsPoll(); };
