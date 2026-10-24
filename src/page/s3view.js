// ---------- the S3 copy on the Export view: compare the sealed months with an S3-compatible store, send what is missing ----------
// the page only talks to this server; the server makes the S3 requests, with keys it finds itself (environment or
// ~/.config/relic-scan/s3.env). Where the copy goes is set in Settings; no key is ever typed or shown here.
let s3Timer = 0;
const S3ACT = { same: ["identical in S3", "ok"], upload: ["sent", "ok"], "would send": ["not in S3 yet", "warn"], differs: ["differs from S3", "bad"],
  missing: ["not in S3", "warn"], outdated: ["older in S3: Send updates it", "warn"], refused: ["sealed again: not replaced", "bad"] };
function s3Card() {
  return `<div class="card" id="s3card"><h3>S3 copy <span class="muted" style="font-weight:400;font-size:13px">the sealed months, compared with the store</span></h3>
    <div id="s3body"><div class="muted">${SERVED ? "loading…" : "Needs the server: open the page through relic-scan serve or the app."}</div></div></div>`;
}
const s3Name = k => k.split("/").slice(1).join("/") || k;
function s3Words(j) {
  const m = j.summary, n = m ? m.same + m.uploaded + m.differs + m.missing + m.refused + (m.outdated || 0) : 0;
  if (j.error) return `<div class="note bad">${esc(j.error)}</div>`;
  if (!m) return j.running ? "" : `<div class="note">Check compares each sealed month with the store and sends nothing. Send uploads what is missing and never replaces a month that differs.</div>`;
  const bad = m.differs + m.refused, gone = m.missing;
  if (j.job === "send") return `<div class="note">Sent ${fmt(m.uploaded)} ${m.uploaded === 1 ? "file" : "files"} (${human(m.bytes)}); ${fmt(m.same)} already there${m.refused ? `; <b>${fmt(m.refused)} differ and were not replaced</b> (sealed again after a late row)` : ""}.</div>`;
  return bad || gone ? `<div class="note bad">${fmt(bad)} differ, ${fmt(gone)} not in S3, ${fmt(m.same)} identical (${fmt(n)} files).</div>`
    : `<div class="note">All ${fmt(m.same)} sealed months are identical in S3.${m.outdated ? " The seals file there is older than this index: Send updates it." : ""}</div>`;
}
function s3Draw(j) {
  const ok = j.configured && j.creds.ok, off = !ok || j.running, items = j.items || [];
  const where = j.configured ? `<code>${esc(j.to)}</code> ${j.endpoint ? `on <code>${esc(j.endpoint)}</code>` : "on AWS S3"} · <a href="/settings">change</a>`
    : `not set up · <a href="/settings">choose where in Settings</a>`;
  const pct = j.total ? Math.min(100, Math.round(100 * (j.done || 0) / j.total)) : 0;
  $("s3body").innerHTML = `<dl class="s3dl"><dt>Destination</dt><dd>${where}</dd>
      <dt>Keys</dt><dd><span class="s3chip ${j.creds.ok ? "ok" : "warn"}">${j.creds.ok ? "found" : "missing"}</span> ${esc(j.creds.detail)}</dd></dl>
    <div class="actions" style="margin-top:var(--sp-3)"><button class="ghost" id="s3check" ${off ? "disabled" : ""}>Check against S3</button>
      <button class="ghost" id="s3send" ${off ? "disabled" : ""}>Send sealed months</button></div>
    ${j.running ? `<div class="s3bar" role="progressbar" aria-valuenow="${pct}" aria-valuemin="0" aria-valuemax="100"><i style="width:${pct}%"></i></div>
      <div class="note">${j.job === "check" ? "Checking" : "Sending"} · ${fmt(j.done || 0)} of ${fmt(j.total || 0)} files…</div>` : ""}
    ${s3Words(j)}
    ${items.length ? `<div class="s3tab">${items.map(i => { const [t, c] = S3ACT[i.action] || [i.action, "warn"];
      return `<div class="s3row"><code>${esc(s3Name(i.key))}</code><span class="z">${human(i.size)}</span><span class="s3chip ${c}">${esc(t)}</span></div>`; }).join("")}</div>` : ""}`;
  for (const [id, what] of [["s3check", "check"], ["s3send", "send"]]) {
    const b = $(id); if (b) b.onclick = async () => {
      b.disabled = true;
      const r = await relicPost("/api/s3/" + what);
      if (!r.ok) { $("s3body").insertAdjacentHTML("beforeend", `<div class="note bad">${esc((await r.json()).error)}</div>`); b.disabled = false; return; }
      s3Poll();
    };
  }
}
async function s3Poll() {
  clearTimeout(s3Timer);
  if (!$("s3body")) return;
  let j;
  try { j = await (await fetch("/api/s3")).json(); } catch { return; }
  s3Draw(j);
  if (j.running) s3Timer = setTimeout(s3Poll, 800);
}
function s3Load() { if (SERVED) s3Poll(); }
