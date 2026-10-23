// ---------- contents: every row as a sunburst over three small groupings, never repos or sessions ----------
// rings: what the row is (7 kinds) · what kind of .jsonl holds it (main session, subagent, workflow agent) · where that
// file lives (own home, mirror, archive, rescue, copy). Tool errors are a subset of tool results, so "tool results" is
// the results without an error. A session copied into several homes counts once, under its first home.
const KINDS = [["user", "user text"], ["asst", "AI text"], ["think", "thinking"], ["calls", "tool calls"], ["ok", "tool results"],
               ["errors", "tool errors"], ["other", "other rows"]];
const kcolor = key => key === "ok" ? TYPES[4][2] : key === "other" ? (LIGHT ? "#9aa3b5" : "#6f7a96") : TYPES.find(t => t[0] === key)[2];
const kval = (key, k) => key === "ok" ? S.results[k] - S.errors[k]
  : key === "other" ? Math.max(0, S.rows[k] - S.user[k] - S.asst[k] - S.think[k] - S.calls[k] - S.results[k]) : S[key][k];
const FILES = [["main sessions", 0], ["subagents", 1], ["workflow agents", 2]];
const PLACE = { own: "own homes", mirror: "mirrors", archive: "archives", rescue: "rescued", copy: "copies" };
// one pass: rows per (kind, file kind, place); everything the chart shows is a sum over this small table
let SUN_AGG = null;
function sunAgg() {
  if (SUN_AGG) return SUN_AGG;
  const m = new Map();
  for (let k = 0; k < N; k++) {
    const h = D.homes[S.homes[k][0]] || {}, place = PLACE[h.origin] || h.origin || "other places", agent = h.agent || "other";
    const month = S.t0[k] == null ? "undated" : bkk(S.t0[k]).slice(0, 7);
    for (const [key] of KINDS) {
      const v = kval(key, k);
      if (v) { const id = [key, S.tier[k], place, agent, month].join("|"); m.set(id, (m.get(id) || 0) + v); }
    }
  }
  return SUN_AGG = [...m].map(([id, v]) => { const [type, tier, place, agent, month] = id.split("|"); return { type, tier: +tier, place, agent, month, v }; });
}
// the kind the contents popup is showing (null: every row)
let CONT_SEL = null;
const sunSum = f => sunAgg().reduce((t, r) => t + (Object.entries(f).every(([d, v]) => r[d] === v) ? r.v : 0), 0);
function arc(r0, r1, a0, a1) {
  const p = (r, a) => `${(r * Math.sin(a)).toFixed(2)} ${(-r * Math.cos(a)).toFixed(2)}`, big = a1 - a0 > Math.PI ? 1 : 0;
  if (a1 - a0 >= 2 * Math.PI - 1e-6) a1 = a0 + 2 * Math.PI - 1e-4;
  return `M${p(r1, a0)}A${r1} ${r1} 0 ${big} 1 ${p(r1, a1)}L${p(r0, a1)}A${r0} ${r0} 0 ${big} 0 ${p(r0, a0)}Z`;
}
const short = v => v >= 1e6 ? (v / 1e6).toFixed(2) + " M" : v >= 1e4 ? (v / 1e3).toFixed(1) + " k" : v.toLocaleString();
// the small donut on Smart Scan: hover names a slice in the centre, a click opens the full view
function contentsMini() {
  const total = sunSum({}), kinds = KINDS.map(([key, label]) => ({ key, label, value: sunSum({ type: key }), color: kcolor(key) }))
    .filter(k => k.value).sort((a, b) => b.value - a.value), onColor = LIGHT ? "#ffffff" : "#0b0e14";
  let a = 0;
  const slices = kinds.map(k => { const w = 2 * Math.PI * k.value / total, s0 = a; a += w; return { ...k, a0: s0, a1: a }; });
  const centre = k => `<text class="ct" text-anchor="middle" y="${k ? -4 : 8}">${short(k ? k.value : total)}</text>
    <text class="cs" text-anchor="middle" y="${k ? 26 : 36}">${k ? esc(k.label) : "rows"}</text>${k ? `<text class="cs" text-anchor="middle" y="50">${(100 * k.value / total).toFixed(1)}%</text>` : ""}`;
  $("sunmini").innerHTML = `<svg viewBox="-170 -170 340 340" aria-hidden="true">
    ${slices.map((k, i) => `<path data-i="${i}" d="${arc(108, 162, k.a0, k.a1)}" fill="${k.color}"><title>${esc(k.label)}: ${k.value.toLocaleString()} rows (${(100 * k.value / total).toFixed(1)}%)</title></path>`).join("")}
    ${slices.map(k => { if (k.a1 - k.a0 < 0.3) return ""; const m = (k.a0 + k.a1) / 2;
      return `<text class="pc" x="${(135 * Math.sin(m)).toFixed(1)}" y="${(-135 * Math.cos(m) + 5).toFixed(1)}" text-anchor="middle" fill="${onColor}">${Math.round(100 * k.value / total)}%</text>`; }).join("")}
    <circle class="hole" r="106"/><g class="mid">${centre(null)}</g></svg>`;
  const svg = $("sunmini").firstElementChild;
  for (const el of svg.querySelectorAll("path[data-i]")) {
    const k = slices[+el.dataset.i];
    el.onmouseenter = () => { svg.classList.add("hov"); el.classList.add("on"); svg.querySelector(".mid").innerHTML = centre(k); };
    el.onmouseleave = () => { svg.classList.remove("hov"); el.classList.remove("on"); svg.querySelector(".mid").innerHTML = centre(null); };
  }
  $("sunmini").onclick = () => openContents();
  const hover = (i, on) => { const el = svg.querySelector(`path[data-i="${i}"]`); if (el) (on ? el.onmouseenter : el.onmouseleave)(); };
  $("sunleg").innerHTML = slices.map((k, i) => `<button class="row" data-i="${i}"><span class="dot" style="background:${k.color}"></span>
      <span class="n">${esc(k.label)}</span><span class="v">${k.value.toLocaleString()}</span><span class="p">${(100 * k.value / total).toFixed(1)}%</span></button>`).join("");
  for (const b of $("sunleg").querySelectorAll("button.row")) {
    b.onmouseenter = () => hover(b.dataset.i, true); b.onmouseleave = () => hover(b.dataset.i, false);
    b.onfocus = b.onmouseenter; b.onblur = b.onmouseleave;
    b.onclick = () => openContents(slices[+b.dataset.i].key);
  }
}
// the popup shows what the donut cannot: for one kind of row (or every row), the kind of .jsonl, where the file
// lives, the agent, and the month each session started. Chips switch the kind; the donut itself stays on the page.
function openContents(kind) {
  const pop = overlay("cpop", `aria-labelledby="cpt"`,
    `<div class="th"><b id="cpt"></b><span class="muted" id="cps"></span><span class="sp"></span><button id="cpx" aria-label="Close">×</button></div>
     <div class="chips" id="cpc"></div><div class="detail" id="cpd"></div>`, () => { if ($("sunmini")) $("sunmini").focus(); });
  $("cpx").onclick = pop.close;
  pop.hidden = false; CONT_SEL = kind || null; contentsFull(); $("cpx").focus();
}
function contentsFull() {
  const total = sunSum({}), sel = CONT_SEL, f = sel ? { type: sel } : {}, here = sunSum(f);
  const label = sel ? KINDS.find(k => k[0] === sel)[1] : "Every row";
  $("cpt").textContent = label;
  $("cps").textContent = `${here.toLocaleString()} rows${sel ? ` · ${(100 * here / total).toFixed(1)}% of all` : ""}`;
  $("cpc").innerHTML = [[null, "every row", "var(--ink)"]].concat(KINDS.map(([k, l]) => [k, l, kcolor(k)]))
    .map(([k, l, c]) => `<button class="${k === sel ? "on" : ""}" data-k="${k || ""}" aria-pressed="${k === sel}"><span class="dot" style="background:${c}"></span>${esc(l)}</button>`).join("");
  for (const b of $("cpc").querySelectorAll("button")) b.onclick = () => { CONT_SEL = b.dataset.k || null; contentsFull(); };
  const group = dim => { const m = new Map(); for (const r of sunAgg()) if (!sel || r.type === sel) m.set(r[dim], (m.get(r[dim]) || 0) + r.v);
    return [...m].map(([key, v]) => ({ key, v })).sort((a, b) => b.v - a.v); };
  // a bar: one hue, darker to lighter by size, so it never reads as the donut's colours
  const bar = (title, parts, hue, name) => {
    const tot = Math.max(1, parts.reduce((t, p) => t + p.v, 0)), shade = i => `color-mix(in oklab, ${hue} ${Math.max(28, 100 - i * 22)}%, var(--card))`;
    return `<section><h4>${title}</h4>
      <div class="track2">${parts.map((p, i) => `<i style="width:${(100 * p.v / tot).toFixed(2)}%;background:${shade(i)}" title="${esc(name(p.key))}: ${p.v.toLocaleString()}"></i>`).join("")}</div>
      ${parts.map((p, i) => `<div class="row"><span class="dot" style="background:${shade(i)}"></span><span class="n">${esc(name(p.key))}</span>
        <span class="v">${p.v.toLocaleString()}</span><span class="p">${(100 * p.v / tot).toFixed(1)}%</span></div>`).join("")}</section>`;
  };
  const months = group("month").sort((a, b) => a.key < b.key ? -1 : 1), top = Math.max(1, ...months.map(m => m.v));
  // every row: each month's column is stacked by kind, the biggest kind overall at the bottom, the same order in every column
  const kindsBySize = KINDS.slice().sort((a, b) => sunSum({ type: b[0] }) - sunSum({ type: a[0] }));
  const byMonthKind = new Map(); if (!sel) for (const r of sunAgg()) { const id = r.month + "|" + r.type; byMonthKind.set(id, (byMonthKind.get(id) || 0) + r.v); }
  const mname = k => k === "undated" ? "undated" : new Date(k + "-01T00:00:00Z").toLocaleString("en", { month: "short", timeZone: "UTC" });
  const colour = sel ? kcolor(sel) : "var(--accent)";
  $("cpd").innerHTML = `${bar("Kind of .jsonl", group("tier"), "var(--accent)", t => FILES[t][0])}
    ${bar("Where the file lives", group("place"), "var(--green)", p => p)}
    ${bar("Agent", group("agent"), "var(--amber)", a => a)}
    <section class="months"><h4>By month <span class="muted">· the month each session started (Bangkok)${sel ? "" : " · stacked by kind"}</span></h4>
      <div class="mchart" id="mch">${months.map(m => `<div class="mc">
        <div class="b"><span class="mv">${short(m.v)}</span>${sel ? `<i data-m="${m.key}" data-k="${sel}" data-v="${m.v}" style="height:${(82 * m.v / top).toFixed(1)}%;background:${colour}"></i>`
          : `<div class="stack" style="height:${(82 * m.v / top).toFixed(1)}%">${kindsBySize.map(([key, l]) => { const v = byMonthKind.get(m.key + "|" + key) || 0;
              return v ? `<i data-m="${m.key}" data-k="${key}" data-v="${v}" style="flex:${v};background:${kcolor(key)}"></i>` : ""; }).join("")}</div>`}</div>
        <span class="ml">${mname(m.key)}${m.key.endsWith("-01") || m === months[0] ? `<b>${m.key.slice(0, 4)}</b>` : ""}</span></div>`).join("")}</div>
      <div class="mtip" id="mtip" hidden></div></section>
    <p class="foot">A line copied into several homes is one row, under the first home that holds it.</p>`;
  // hover a bar or a band: which month, which kind, how many rows, its share of the month (or of that kind)
  const monthTotal = new Map(months.map(m => [m.key, m.v])), kindTotal = sel ? here : 0;
  const longMonth = k => k === "undated" ? "undated" : new Date(k + "-01T00:00:00Z").toLocaleString("en", { month: "long", year: "numeric", timeZone: "UTC" });
  const tip = $("mtip"), chart = $("mch");
  chart.onmousemove = e => {
    const i = e.target.closest("i[data-m]");
    for (const x of chart.querySelectorAll("i.hot")) if (x !== i) x.classList.remove("hot");
    if (!i) { tip.hidden = true; return; }
    i.classList.add("hot");
    const m = i.dataset.m, v = +i.dataset.v, label = KINDS.find(k => k[0] === i.dataset.k)[1], mt = monthTotal.get(m);
    tip.innerHTML = sel
      ? `<b>${longMonth(m)}</b><span>${esc(label)}</span><span><b>${v.toLocaleString()}</b> rows · ${(100 * v / Math.max(1, kindTotal)).toFixed(1)}% of all ${esc(label)}</span>`
      : `<b>${longMonth(m)}</b><span><i style="background:${kcolor(i.dataset.k)}"></i>${esc(label)}</span>
         <span><b>${v.toLocaleString()}</b> rows · ${(100 * v / Math.max(1, mt)).toFixed(1)}% of the month</span><span class="muted">month total ${mt.toLocaleString()}</span>`;
    const box = chart.parentNode.getBoundingClientRect();
    tip.hidden = false;
    tip.style.left = Math.min(e.clientX - box.left + 14, box.width - tip.offsetWidth - 4) + "px";
    tip.style.top = (e.clientY - box.top - tip.offsetHeight - 10) + "px";
  };
  chart.onmouseleave = () => { tip.hidden = true; for (const x of chart.querySelectorAll("i.hot")) x.classList.remove("hot"); };
}
