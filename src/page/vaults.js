// ---------- vaults: every oracle's ψ vault the scan walked past, with its notes (data: /vaults.json, /vaults/<id>.json) ----------
// one row per vault (by its real path; repos that link to it are counted with it). Git worktree copies of a vault are
// hidden unless asked. Open a row for its notes, newest first, each one click from Finder; they load only then.
let VAULTS = null, VOPEN = -1, VQ = "", VCAP = 200, VWT = false;
const VNOTES = new Map();
const VCOL = ["var(--cyan)", "var(--amber)", "var(--green)", "var(--barfg)"];
function dashVaults() {
  return `<h1>ψ Vaults${rangeChip()}</h1>
    <p class="sub">Every oracle's ψ vault on this machine and the notes in it. Read on each index; nothing in a vault is changed.</p>${scopeLine()}
    <div id="vbody"><div class="muted">${SERVED ? "loading the vaults…" : "This view needs the server: open the page through relic-scan serve or the app."}</div></div>`;
}
function vaultsLoad() {
  if (VAULTS) return vaultsRender();
  if (!SERVED) return;
  fetch("/vaults.json").then(r => r.ok ? r.json() : Promise.reject(r.status)).then(j => { VAULTS = j.vaults || []; vaultsRender(); })
    .catch(() => { $("vbody").innerHTML = `<div class="card"><div class="note" style="margin:0">No vault list yet. Press Index now: every index writes it.</div></div>`; });
}
const vtilde = p => p.replace(D.home || "\u0000", "~");
const vday = ms => ms ? bkk(ms / 1000).slice(0, 10) : "—";
// the sections of a vault, biggest first, the rest folded into "other"
function vsections(v) {
  const s = Object.entries(v.sections).sort((a, b) => b[1] - a[1]), top = s.slice(0, 4), rest = s.slice(4).reduce((t, x) => t + x[1], 0);
  return rest ? [...top, ["other", rest]] : top;
}
function vaultsRender() {
  const own = VAULTS.filter(v => v.kind !== "worktree"), wt = VAULTS.length - own.length, V = VWT ? VAULTS : own;
  const notes = own.reduce((t, v) => t + v.notes, 0), bytes = own.reduce((t, v) => t + v.bytes, 0), newest = Math.max(0, ...own.map(v => v.newest_ms));
  if (!VAULTS.length) { $("vbody").innerHTML = `<div class="card"><div class="note" style="margin:0">No ψ folder under the roots. Settings lists the folders the scan walks.</div></div>`; return; }
  const row = v => {
    const secs = vsections(v), tot = Math.max(1, v.notes), open = v.id === VOPEN;
    return `<div class="vrow${open ? " open" : ""}">
      <button class="vhead" data-id="${v.id}" aria-expanded="${open}">
        <span class="vn"><b>${esc(v.name)}${v.kind === "worktree" ? ` <span class="tag">worktree</span>` : ""}</b><code>${esc(vtilde(v.path))}</code>
          ${v.links.length ? `<span class="muted">+ ${v.links.length} linked repo${v.links.length > 1 ? "s" : ""}</span>` : ""}${v.kind === "worktree" && v.origin ? `<span class="muted">copy of ${esc(vtilde(v.origin))}</span>` : ""}</span>
        <span class="vbar" title="${esc(secs.map(([k, n]) => `${k}: ${n}`).join(" · "))}">${secs.map(([k, n], j) => `<i style="flex:${n / tot};background:${k === "other" ? "var(--track)" : VCOL[j % 4]}"></i>`).join("")}</span>
        <span class="vx"><b>${fmt(v.notes)}</b> notes · ${human(v.bytes)}<br><span class="muted">newest ${vday(v.newest_ms)}</span></span>
      </button>
      ${open ? vdetail(v, secs) : ""}</div>`;
  };
  $("vbody").innerHTML = `<div class="strip">${stat("Vaults", fmt(own.length), "var(--amber)")}${stat("Notes", fmt(notes), "var(--cyan)")}${stat("Size", human(bytes), "var(--green)")}${stat("Newest note", vday(newest), "var(--ink)")}</div>
    <div class="card vaults"><h3>Vaults · ${fmt(V.length)}<span class="muted" style="font-weight:400;font-size:13px">by notes · open one for its notes</span></h3>
      ${wt ? `<div class="note" style="margin:0 0 10px">${fmt(wt)} more are git worktree copies of these vaults. <a href="#" id="vwt" style="color:var(--cyan)">${VWT ? "hide them" : "show them"}</a></div>` : ""}
      ${V.map(row).join("")}</div>`;
  for (const b of $("vbody").querySelectorAll(".vhead")) b.onclick = () => { VOPEN = +b.dataset.id === VOPEN ? -1 : +b.dataset.id; VQ = ""; VCAP = 200; vaultsRender(); };
  if ($("vwt")) $("vwt").onclick = e => { e.preventDefault(); VWT = !VWT; vaultsRender(); };
  const v = VAULTS.find(x => x.id === VOPEN);
  if (!v || !$("vq")) return;
  $("vq").oninput = () => { VQ = $("vq").value; VCAP = 200; vnotes(v); };
  $("vopen").onclick = () => kReveal("what=vault&path=" + encodeURIComponent(v.path));
  $("vexp").onclick = async () => { const items = await vitems(v);
    $("vmsg").textContent = await xsave(xall(`vault-${v.name}`, ["path", "section", "title", "size_bytes", "modified_bkk"],
      items.map(([rel, sec, t, size, ms]) => [v.path + "/" + rel, sec, t, size, xtime(ms / 1000)])), "csv"); };
  vnotes(v);
}
function vdetail(v, secs) {
  return `<div class="vdet">
    <div class="vtools"><input id="vq" type="search" placeholder="filter notes: words in the title or path" value="${esc(VQ)}" aria-label="filter notes">
      <button class="ghost" id="vopen">Open folder</button><button class="ghost" id="vexp">Export notes (CSV)</button></div>
    <div class="vlegend">${secs.map(([k, n], j) => `<span><i style="background:${k === "other" ? "var(--track)" : VCOL[j % 4]}"></i>${esc(k)} ${fmt(n)}</span>`).join("")}
      ${v.links.length ? `<span class="muted">linked from ${v.links.map(l => `<code>${esc(vtilde(l))}</code>`).join(", ")}</span>` : ""}
      ${v.skipped_repos ? `<span class="muted">${fmt(v.skipped_repos)} cloned repo${v.skipped_repos > 1 ? "s" : ""} inside skipped</span>` : ""}</div>
    <div class="msg" id="vmsg" role="status"></div>
    <div id="vnotes"><div class="muted">loading the notes…</div></div></div>`;
}
const vitems = v => VNOTES.has(v.id) ? Promise.resolve(VNOTES.get(v.id))
  : fetch(`/vaults/${v.id}.json`).then(r => r.json()).then(j => { VNOTES.set(v.id, j.items || []); return j.items || []; });
async function vnotes(v) {
  const items = await vitems(v);
  if (v.id !== VOPEN || !$("vnotes")) return;
  const words = VQ.toLowerCase().split(/\s+/).filter(Boolean);
  const hit = items.filter(([rel, , t]) => words.every(w => (t + " " + rel).toLowerCase().includes(w)));
  $("vnotes").innerHTML = (hit.length ? hit.slice(0, VCAP).map(([rel, sec, t, size, ms]) => `<button class="vnote" data-p="${esc(v.path + "/" + rel)}" title="show ${esc(rel)} in Finder">
      <span class="d">${vday(ms)}</span><span class="s">${esc(sec)}</span><span class="t">${esc(t)}</span><span class="z">${human(size)}</span></button>`).join("")
    : `<div class="note">No note matches.</div>`)
    + (hit.length > VCAP ? `<button class="ghost vmore" id="vmore">show ${fmt(Math.min(500, hit.length - VCAP))} more of ${fmt(hit.length - VCAP)}</button>` : "");
  for (const b of $("vnotes").querySelectorAll(".vnote")) b.onclick = () => kReveal("what=vault&path=" + encodeURIComponent(b.dataset.p));
  if ($("vmore")) $("vmore").onclick = () => { VCAP += 500; vnotes(v); };
}
