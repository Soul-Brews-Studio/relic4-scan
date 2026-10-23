// ---------- legend + input ----------
function legend() {
  $("legend").innerHTML = TIER.map((t, i) => `<button class="${tierOn[i] ? "on" : ""}" data-t="${i}"><b style="background:${COLOR[i]}"></b>${t}</button>`).join("")
    + (VIEW === "tree" ? "" : ` <span>color:</span>` + ["tier", "repo", "age", "tools%", "thinking"].map(c => `<button class="${colorBy === c ? "on" : ""}" data-c="${c}">${c}</button>`).join(""));
  for (const el of $("legend").querySelectorAll("[data-t]")) el.onclick = () => { tierOn[+el.dataset.t] = !tierOn[+el.dataset.t]; legend(); run($("q").value); };
  for (const el of $("legend").querySelectorAll("[data-c]")) el.onclick = () => { colorBy = el.dataset.c; legend(); redraw(); };
}
let timer;
$("q").addEventListener("input", () => { clearTimeout(timer); timer = setTimeout(() => run($("q").value), 180); });
$("q").addEventListener("keydown", e => { if (e.key === "Escape") { $("q").value = ""; run(""); } });

let drag = null, moved = false;
cv.addEventListener("mousedown", e => { drag = [e.clientX, e.clientY, ox, oy]; moved = false; });
addEventListener("mouseup", e => {
  if (drag && !moved && e.target === cv) { const k = nearest(e.clientX, e.clientY); select(k); }
  drag = null;
});
addEventListener("mousemove", e => {
  if (drag) {
    const dx = e.clientX - drag[0], dy = e.clientY - drag[1];
    if (Math.abs(dx) + Math.abs(dy) > 3) moved = true;
    ox = drag[2] + dx; oy = drag[3] + dy; redraw(); return;
  }
  if (e.target !== cv) return;
  const k = nearest(e.clientX, e.clientY), tip = $("tip");
  if (k !== hover) { hover = k; redraw(); }
  if (k < 0) { tip.style.display = "none"; return; }
  tip.style.display = "block"; tip.style.left = e.clientX + 14 + "px"; tip.style.top = e.clientY + 12 + "px";
  tip.innerHTML = `<span style="color:${COLOR[S.tier[k]]}">${TIER[S.tier[k]]}</span> · ${esc(cat(k))}<br>
    <code>${esc(S.sid[k])}</code><br>${bkk(S.t0[k])} · ${fmt(S.rows[k])} rows · ${S.copies[k]} cop${S.copies[k] === 1 ? "y" : "ies"}`;
});
cv.addEventListener("wheel", e => {
  e.preventDefault();
  const f = Math.exp(-e.deltaY * 0.0015), x = e.clientX, y = e.clientY;
  ox = x - (x - ox) * f; oy = y - (y - oy) * f; scale *= f; redraw();
}, { passive: false });
cv.addEventListener("dblclick", () => { home(); redraw(); });
addEventListener("resize", () => { fit(); redraw(); });

if (VIEW !== "map") { cv.style.display = "none"; document.documentElement.classList.add("doc"); }
if (DASH) {
  for (const id of ["q", "count", "panel", "legend"]) $(id).style.display = "none";
  $("bar").style.right = "14px"; $("bar").style.justifyContent = "flex-end"; $("bar").style.background = "transparent";
  $("dash").style.display = "block";
} else if (VIEW !== "map") $("tree").style.display = "block";
$("theme").textContent = { auto: "◐ auto", dark: "● dark", light: "○ light" }[THEME_PICK];
// served (not opened as a file): an index button that runs an incremental run in the server and reloads when done
if (SERVED) {
  const b = $("idx"); b.style.display = ""; b.textContent = "⟳ index";
  const gb = x => (x / 1e9).toFixed(1);
  const poll = async () => {
    const s = await (await fetch("/api/status")).json();
    window.onRun(s);
    if (s.running && s.job === "check") { b.disabled = true; b.textContent = "⟳ checking…"; return setTimeout(poll, 700); }
    if (s.running) {
      b.disabled = true;
      b.textContent = s.phase === "parse" && s.total ? `⟳ parse ${Math.round(100 * s.done / s.total)}% · ${gb(s.done)}/${gb(s.total)} GB`
        : s.phase === "scan" ? `⟳ scan · ${s.listed.toLocaleString()} .jsonl` : `⟳ ${s.phase}…`;
      return setTimeout(poll, 700);
    }
    if (!b.disabled) return;
    if (s.last_error) { b.disabled = false; b.textContent = "⟳ failed: " + s.last_error.slice(0, 40); }
    else if (s.job === "check" || s.stopped) { b.disabled = false; b.textContent = "⟳ index"; }
    else location.reload();
  };
  // the output popup: lines from /api/log, colours from the CLI's own escape codes
  const term = document.createElement("div");
  term.id = "term"; term.hidden = true; term.setAttribute("role", "log"); term.setAttribute("aria-label", "relic-scan output");
  term.innerHTML = `<div class="th"><b>Output</b><span id="tst"></span><span class="sp"></span><span class="mini" id="tmini"></span>
    <button id="tstop" class="tstop" title="stop the run (nothing is written)" hidden>stop</button><button id="tmin" aria-label="Minimise output" title="minimise">–</button><button id="tclose" aria-label="Close output" title="close">×</button></div><pre id="tout"></pre>`;
  document.body.append(term);
  const ansi = t => {
    let out = "", open = 0;
    for (const part of t.split(/(\x1b\[[0-9;]*[A-Za-z])/)) {
      const m = part.match(/^\x1b\[([0-9;]*)m$/);
      if (m) {
        const c = m[1].split(";");
        if (m[1] === "0" || m[1] === "") { out += "</span>".repeat(open); open = 0; }
        else if (c[0] === "38" && c[1] === "2") { out += `<span style="color:rgb(${c[2]},${c[3]},${c[4]})">`; open++; }
        else if (c[0] === "1") { out += `<span style="font-weight:700">`; open++; }
      } else if (!/^\x1b/.test(part)) out += esc(part);
    }
    return out + "</span>".repeat(open);
  };
  let tnext = 0, tlines = [], ttimer = 0;
  const tdraw = (live, st) => {
    const pre = $("tout"), stick = pre.scrollTop + pre.clientHeight >= pre.scrollHeight - 24;
    pre.innerHTML = tlines.map(ansi).join("\n") + (live ? (tlines.length ? "\n" : "") + `<span class="cur">${ansi(live)}</span>` : "");
    if (stick) pre.scrollTop = pre.scrollHeight;
    // minimised: the live line, or else the last line with words in it (the card's frame lines carry none)
    const words = [...tlines].reverse().find(l => /[A-Za-z]/.test(l.replace(/\x1b\[[0-9;]*m/g, ""))) || "";
    $("tmini").innerHTML = ansi((live || words).replace(/[│╭╮╰╯]/g, "").trim());
    $("tst").className = st.running ? "live" : "";
    $("tstop").hidden = !st.running || st.phase === "merge" || st.phase === "meta"; $("tstop").textContent = st.stopping ? "stopping…" : "stop";
    $("tst").textContent = st.running ? `${st.job} · live` : st.stopped ? `last ${st.job} · stopped, nothing written` : st.job ? `last ${st.job} · done` : "nothing has run in this server yet";
  };
  const tpoll = async () => {
    clearTimeout(ttimer);
    const r = await (await fetch("/api/log?from=" + tnext)).json();
    if (r.next < tnext) tlines = []; // a new job started: the server dropped the old output
    tlines = tlines.concat(r.lines); tnext = r.next;
    tdraw(r.live, r);
    if (r.running && !term.hidden) ttimer = setTimeout(tpoll, 300);
  };
  window.relicTerm = () => { term.hidden = false; tpoll(); };
  const tfresh = () => { tnext = 0; tlines = []; window.relicTerm(); setTimeout(tpoll, 400); };
  $("tstop").onclick = e => { e.stopPropagation(); window.relicStop(); };
  $("tclose").onclick = e => { e.stopPropagation(); term.hidden = true; term.classList.remove("min"); clearTimeout(ttimer); };
  const tmin = on => { term.classList.toggle("min", on); $("tmin").textContent = on ? "□" : "–";
    $("tmin").setAttribute("aria-label", on ? "Restore output" : "Minimise output"); };
  $("tmin").onclick = e => { e.stopPropagation(); tmin(!term.classList.contains("min")); };
  term.querySelector(".th").onclick = () => { if (term.classList.contains("min")) tmin(false); };
  addEventListener("keydown", e => { if (e.key === "Escape" && !term.hidden) $("tclose").click(); });
  window.relicRefresh = () => poll();
  window.relicStop = async () => { await relicPost("/api/stop"); poll(); };
  window.relicCheck = async () => {
    b.disabled = true; b.textContent = "⟳ checking…";
    tfresh();
    const r = await relicPost("/api/check");
    if (!r.ok && r.status !== 409) { b.disabled = false; b.textContent = "⟳ " + ((await r.json()).error || r.status); return; }
    poll();
  };
  b.onclick = async () => {
    b.disabled = true; b.textContent = "⟳ starting…";
    tfresh();
    const r = await relicPost("/api/run");
    if (!r.ok && r.status !== 409) { b.disabled = false; b.textContent = "⟳ " + ((await r.json()).error || r.status); return; }
    poll();
  };
  poll();
}
$("theme").onclick = () => {
  const next = { auto: "dark", dark: "light", light: "auto" }[THEME_PICK];
  localStorage.setItem("relic-theme", next); location.reload();
};
fit(); home();
const homeScale = scale;
$("q").value = decodeURIComponent(location.hash.slice(1));
if (DASH) {
  tabs(); renderDash();
} else if (VIEW === "paths") {
  $("q").placeholder = "paths: words in the path · is:indexed|skipped|vault|loose · verdict:not-transcript · mb>100 · rows=0 · -word";
  tabs(); loadPaths();
} else {
  legend();
  run($("q").value);
}
