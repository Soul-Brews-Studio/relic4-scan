// ---------- settings: what to scan and how often to check (OUT/_config.json through /api/config) ----------
// the server checks what it is sent (folders that exist, depth 1-32, auto check off/15/30/60) and says what is wrong
function dashSettings() {
  return `<div class="eyebrow">Preferences</div><h1>Settings${rangeChip()}</h1>
    <p class="sub">What the scan walks and how often the server checks for changes. Index now and <code>relic-scan run</code>
      use these; a flag on the command line wins.</p>${scopeLine()}
    <div id="sset">${SERVED ? `<div class="muted">loading…</div>` : `<div class="muted">Settings need the server: open this page through relic-scan serve or the app.</div>`}</div>`;
}
// an editable list as chips: × removes one, the box adds one (Enter or a suggestion), "Choose folder…" asks the desktop
function chipList(host, values, o) {
  let v = values.slice();
  const id = "dl" + Math.random().toString(36).slice(2, 8);
  const draw = () => {
    host.innerHTML = `<div class="chips2">${v.map((x, i) => `<span class="chip2"><code>${esc(x)}</code><button type="button" data-i="${i}" aria-label="Remove ${esc(x)}">×</button></span>`).join("")}</div>
      <div class="add2"><input list="${id}" placeholder="${esc(o.placeholder)}" spellcheck="false" autocomplete="off"><datalist id="${id}">${o.suggest.filter(x => !v.includes(x)).map(x => `<option value="${esc(x)}">`).join("")}</datalist>
        <button type="button" class="ghost sm" data-add>Add</button>${o.pick && SERVED ? `<button type="button" class="ghost sm" data-pick>Choose folder…</button>` : ""}</div>`;
    const inp = host.querySelector("input");
    const add = () => { const x = inp.value.trim(); if (x && !v.includes(x)) { v.push(x); draw(); host.querySelector("input").focus(); } else inp.value = ""; };
    inp.onkeydown = e => { if (e.key === "Enter") { e.preventDefault(); add(); } };
    host.querySelector("[data-add]").onclick = add;
    for (const b of host.querySelectorAll(".chip2 button")) b.onclick = () => { v.splice(+b.dataset.i, 1); draw(); };
    const p = host.querySelector("[data-pick]");
    if (p) p.onclick = async () => { p.disabled = true; p.textContent = "waiting for the picker…";
      const r = await relicPost("/api/pick"); if (r.status === 200) { const { path } = await r.json(); const t = path.replace(D.home || "\u0000", "~"); if (!v.includes(t)) v.push(t); }
      draw(); };
  };
  draw();
  return { get: () => v.slice(), set: x => { v = x.slice(); draw(); } };
}
async function loadSettings() {
  if (!SERVED) return;
  const r = await (await fetch("/api/config")).json(), c = r.config, d = r.defaults;
  const theme = localStorage.getItem("relic-theme") || "auto";
  $("sset").innerHTML = `<form class="card set" id="sform" novalidate>
      <h3>What to scan</h3>
      <div class="field"><b>Folders to walk</b><span>default ${esc(d.roots.join(", "))}</span><div id="s-roots"></div></div>
      <div class="field"><b>Skip paths holding</b><span>a path with any of these in it is not walked</span><div id="s-ex"></div></div>
      <label class="row2"><b>Depth</b><span>how deep the walk looks for agent homes · default ${d.depth}</span>
        <input id="s-depth" type="number" min="1" max="32" value="${c.depth}"></label>
      <h3>Checking</h3>
      <label class="row2"><b>Check changes automatically</b><span>while this server runs; it never indexes on its own</span>
        <select id="s-auto">${[[0, "off"], [15, "every 15 minutes"], [30, "every 30 minutes"], [60, "every hour"]]
          .map(([v, l]) => `<option value="${v}" ${c.auto_check_min === v ? "selected" : ""}>${l}</option>`).join("")}</select></label>
      <h3>Look</h3>
      <label class="row2"><b>Theme</b><span>this browser only</span>
        <select id="s-theme">${["auto", "dark", "light"].map(v => `<option ${theme === v ? "selected" : ""}>${v}</option>`).join("")}</select></label>
      <div class="actions"><button class="go" type="submit">Save</button><button class="ghost" type="button" id="s-reset">Use the defaults</button></div>
      <div class="note" id="s-msg" role="status"></div>
    </form>
    <div class="card set"><h3>About this index</h3>
      <dl><dt>Output folder</dt><dd><code>${esc(r.out)}</code></dd><dt>Agents</dt><dd>${r.kind.length ? esc(r.kind.join(", ")) : "every agent"}</dd>
        <dt>Version</dt><dd>relic-scan ${esc(r.version)}</dd><dt>Settings file</dt><dd><code>${esc(r.out)}/_config.json</code></dd></dl></div>`;
  // folders: the parents of the agent homes this index found, as suggestions; skips: the usual noise
  const parents = [...new Set(D.homes.map(h => h.path.replace(D.home || "\u0000", "~").split("/").slice(0, 2).join("/")))].sort();
  const roots = chipList($("s-roots"), c.roots, { placeholder: "~/some/folder", suggest: parents, pick: true });
  const skips = chipList($("s-ex"), c.exclude, { placeholder: "node_modules", suggest: ["node_modules", ".git", "Library", ".Trash", ".cache", "target", "wt/"] });
  $("s-reset").onclick = () => { roots.set(d.roots); skips.set([]); $("s-depth").value = d.depth; $("s-auto").value = "0"; $("s-msg").textContent = "Defaults filled in; Save to keep them."; };
  $("sform").onsubmit = async e => {
    e.preventDefault();
    const body = { roots: roots.get(), exclude: skips.get(), depth: Number($("s-depth").value), auto_check_min: Number($("s-auto").value) };
    const res = await fetch("/api/config", { method: "POST", headers: { "X-Relic": "1", "Content-Type": "application/json" }, body: JSON.stringify(body) });
    const out = await res.json();
    if (!res.ok) { $("s-msg").textContent = "Not saved: " + out.error + "."; $("s-msg").className = "note bad"; return; }
    const t = $("s-theme").value, themeChanged = t !== (localStorage.getItem("relic-theme") || "auto");
    localStorage.setItem("relic-theme", t);
    $("s-msg").className = "note"; $("s-msg").textContent = "Saved. The next Check changes or Index now uses these.";
    if (themeChanged) location.reload();
  };
}
