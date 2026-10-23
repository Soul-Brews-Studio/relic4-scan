// ---------- scan log: every index and check (OUT/_scans.jsonl through /api/scans) ----------
const when = at => { const d = new Date(at * 1000), p = x => String(x).padStart(2, "0");
  return `${d.getFullYear()}-${p(d.getMonth() + 1)}-${p(d.getDate())} ${p(d.getHours())}:${p(d.getMinutes())}`; };
const zn = x => x ? x.toLocaleString() : `<span class="z">0</span>`;
const files = n => `${n.toLocaleString()} file${n === 1 ? "" : "s"}`;
const scanWhat = e => e.files ? `${files(e.files)} · ${human(e.bytes)}` : e.gone ? `${e.gone.toLocaleString()} gone` : "nothing changed";
function dashScans() {
  return `<div class="eyebrow">History</div><h1>Scan log${rangeChip()}</h1>
    <p class="sub">Every index and every check, newest first. Open one to see the files it read, or would read, and the files gone from disk.</p>${scopeLine()}
    <div class="card log" id="slog"><div class="muted" style="padding:24px">${SERVED ? "loading…" : "The scan log needs the server: open this page through relic-scan serve or the app."}</div></div>`;
}
let LOG = [];
async function loadScans() {
  if (!SERVED) return;
  LOG = (await (await fetch("/api/scans")).json()).entries;
  if (!LOG.length) { $("slog").innerHTML = `<div class="muted" style="padding:24px">No scans yet. Every Index now and every Check changes adds one.</div>`; return; }
  // rows added: an index's total minus the total of the index before it
  const prevRows = i => { for (let k = i + 1; k < LOG.length; k++) if (LOG[k].job === "index" && LOG[k].rows != null) return LOG[k].rows; return null; };
  $("slog").innerHTML = `<div class="lrow h"><span>When</span><span>Job</span><span class="r">New</span><span class="r">Grown</span><span class="r">Rewritten</span><span class="r">Gone</span><span class="r">Read</span><span class="r">Time</span><span class="x">Result</span></div>`
    + LOG.map((e, k) => { const pr = e.job === "index" ? prevRows(k) : null, d = pr != null && e.rows != null ? e.rows - pr : null;
      return `<div class="lrow" tabindex="0" role="button" aria-expanded="false" data-i="${e.i}">
        <span>${when(e.at)}<span class="ago">${agoText(Date.now() / 1000 - e.at)}${e.range ? " · " + esc(e.range) : ""}</span></span>
        <span><span class="chip ${e.job}">${e.job}</span></span>
        <span class="r">${zn(e.new)}</span><span class="r">${zn(e.grown)}</span><span class="r">${zn(e.rewritten)}</span><span class="r">${zn(e.gone)}</span>
        <span class="r">${e.bytes ? human(e.bytes) : `<span class="z">—</span>`}</span><span class="r">${e.secs.toFixed(1)}s</span>
        <span class="x muted">${e.job === "index" ? (d != null ? `${d >= 0 ? "+" : ""}${d.toLocaleString()} rows` : `${(e.rows || 0).toLocaleString()} rows`) : (e.files ? "would read " + files(e.files) : e.gone ? `${e.gone.toLocaleString()} gone` : "nothing changed")}</span></div>`; }).join("");
  for (const r of $("slog").querySelectorAll(".lrow[data-i]")) {
    r.onclick = () => openScan(r);
    r.onkeydown = ev => { if (ev.key === "Enter" || ev.key === " ") { ev.preventDefault(); openScan(r); } };
  }
  const want = location.hash.slice(1);
  const hit = want && $("slog").querySelector(`.lrow[data-i="${CSS.escape(want)}"]`);
  if (hit) { openScan(hit); hit.scrollIntoView({ block: "center" }); }
}
async function openScan(r) {
  const next = r.nextElementSibling;
  if (next && next.classList.contains("ldiff")) { next.remove(); r.classList.remove("open"); r.setAttribute("aria-expanded", "false"); return; }
  r.classList.add("open"); r.setAttribute("aria-expanded", "true");
  history.replaceState(null, "", "#" + r.dataset.i);
  const box = document.createElement("div"); box.className = "ldiff"; box.innerHTML = `<div class="muted">loading…</div>`; r.after(box);
  const e = await (await fetch("/api/scans/" + r.dataset.i)).json();
  const list = e.diff || [], cap = 300;
  const line = ([st, p, size, from]) => `<div class="ld"><span class="st-${st}">${st}</span>
    <span class="r">${st === "grown" ? "+" + human(size - from) : st === "gone" ? "was " + human(from) : human(size)}</span><code>${esc(ptilde2(p))}</code></div>`;
  const draw = all => {
    box.innerHTML = (list.length ? (all ? list : list.slice(0, cap)).map(line).join("") : `<div class="muted">No file changed.</div>`)
      + (!all && list.length > cap ? `<button class="ghost" style="margin-top:12px;min-width:0;padding:8px 16px;font-size:14px">show all ${list.length.toLocaleString()}</button>` : "")
      + (e.diff_total > list.length ? `<div class="note">The log keeps the ${list.length.toLocaleString()} biggest reads of ${e.diff_total.toLocaleString()}.</div>` : "");
    const b = box.querySelector("button"); if (b) b.onclick = () => draw(true);
  };
  draw(false);
}
const ptilde2 = s => D.home && s.startsWith(D.home) ? "~" + s.slice(D.home.length) : s;
async function loadRecent() {
  if (!SERVED || !$("srecent")) return;
  const es = (await (await fetch("/api/scans")).json()).entries.slice(0, 6);
  $("srecent").innerHTML = es.length ? es.map(e => `<a href="/scans#${e.i}"><span><span class="w">${e.job === "index" ? "Index" : "Check"}</span>
      <span class="muted"> · ${agoText(Date.now() / 1000 - e.at)}</span></span><span class="m">${scanWhat(e)}</span></a>`).join("")
    : `<div class="muted">No scans yet.</div>`;
}
// a run's progress, from the index button's poll: the sidebar state and the Smart Scan card follow it
window.onRun = st => {
  const checking = st.running && st.job === "check";
  if ($("sstate")) { $("sstate").textContent = checking ? "CHECKING" : st.running ? "INDEXING" : "RELIC READY"; $("sdot").style.background = st.running ? "var(--amber)" : "var(--green)"; }
  if (!$("dgo")) return;
  $("dbar").parentNode.classList.toggle("on", !!st.running);
  if ($("dstop")) { $("dstop").hidden = !st.running; $("dstop").disabled = !!st.stopping || st.phase === "merge" || st.phase === "meta";
    $("dstop").textContent = st.stopping ? "Stopping…" : st.phase === "merge" || st.phase === "meta" ? "Finishing (cannot stop now)" : "Stop"; }
  if (!st.running) {
    $("dgo").disabled = !SERVED; $("dgo").textContent = "Index now"; $("dchk").disabled = !SERVED; $("dchk").textContent = "Check changes";
    if (st.stopped) $("dmsg").textContent = `The last ${st.job} was stopped before it wrote anything; the index is as it was.`;
    showCheck(st.check);
    loadRecent();
    return;
  }
  $("dgo").disabled = true; $("dchk").disabled = true;
  if (checking) {
    $("dchk").textContent = "Checking…";
    $("dmsg").textContent = `scan · ${st.walked.toLocaleString()} entries walked · ${st.listed.toLocaleString()} .jsonl listed · ${Math.round(st.secs || 0)}s · nothing is written`;
    return;
  }
  $("dgo").textContent = "Indexing…";
  const f = st.phase === "parse" && st.total ? st.done / st.total : st.phase === "scan" ? 0.03 : st.phase === "merge" ? 0.92 : st.phase === "meta" ? 0.97 : 0;
  $("dbar").style.width = (100 * f).toFixed(1) + "%";
  $("dmsg").textContent = st.phase === "scan" ? `scan · ${st.walked.toLocaleString()} entries walked · ${st.listed.toLocaleString()} .jsonl listed`
    : st.phase === "parse" ? `parse · ${(st.done / 1e9).toFixed(1)} / ${(st.total / 1e9).toFixed(1)} GB` : `${st.phase}…`;
  $("dmsg").textContent += ` · ${Math.round(st.secs || 0)}s`;
};
