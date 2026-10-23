// ---------- tabs: map | tree | paths; a session query travels in the hash between map and tree ----------
function tabs() {
  const file = location.protocol === "file:";
  // a session query carries between map and tree; paths has its own words; the dashboards take none
  const h = v => ["map", "tree"].includes(v) && ["map", "tree"].includes(VIEW) ? location.hash : "";
  const href = v => file ? `?view=${v}${h(v)}` : `/${v}${h(v)}`;
  // a view under another: indented, on a thin guide line from its parent (Tree, Paths and Disk under Session map)
  const sub = (v, ic, l) => item(v, ic, l).replace('<a class="', '<a class="sub ');
  const item = (v, ic, l) => `<a class="${VIEW === v ? "on" : ""}" href="${href(v)}" title="${l}"${VIEW === v ? ` aria-current="page"` : ""}><span class="ic"><svg viewBox="0 0 24 24" aria-hidden="true">${ICONS[ic]}</svg></span><span class="lbl">${l}</span></a>`;
  const ago = D.last_run ? agoText(Date.now() / 1000 - D.last_run) : "never";
  $("side").innerHTML = `<a class="brand" href="${href("scan")}" title="Smart Scan" aria-label="relic-scan: back to Smart Scan"><div class="logo"><svg viewBox="0 0 24 24" aria-hidden="true" fill="none" stroke="currentColor" stroke-width="1.6"><circle cx="12" cy="12" r="8.5"/><circle cx="12" cy="12" r="3.2"/><circle cx="18" cy="6" r="1.6" fill="currentColor" stroke="none"/></svg></div><div class="t"><b>RELIC</b><i>scan</i></div></a>`
    + `<nav aria-label="Views">`
    + item("scan", "scan", "Smart Scan") + item("banks", "banks", "Banks & Repos") + item("health", "health", "Health") + item("scans", "log", "Scan log")
    + `<div class="sep"></div>` + item("map", "map", "Session map") + `<div class="subs">` + sub("tree", "tree", "Tree") + sub("paths", "paths", "Paths") + sub("disk", "disk", "Disk") + `</div>`
    + `<div class="sep"></div>` + item("settings", "settings", "Settings")
    + `</nav>`
    + `<div class="foot"><div><span class="dot" id="sdot"></span><b id="sstate">RELIC READY</b></div>
       ${D.range ? `<div class="rng">only ${esc(D.range.label)}</div>` : ""}<div>Index runs only when you click it.</div><div>last run ${ago}</div></div>`;
}
// the sidebar's icons: one hand-drawn set (radar, stacked disks, pulse, constellation, outline, folder)
const ICONS = {
  scan: `<circle cx="12" cy="12" r="8.5"/><circle cx="12" cy="12" r="4.5"/><path d="M12 12l5.5-5.5"/>`,
  banks: `<ellipse cx="12" cy="6" rx="7" ry="2.5"/><path d="M5 6v6c0 1.4 3.1 2.5 7 2.5s7-1.1 7-2.5V6"/><path d="M5 12v6c0 1.4 3.1 2.5 7 2.5s7-1.1 7-2.5v-6"/>`,
  health: `<path d="M3 12h4l2.5-6 4 12 2.5-6H21"/>`,
  map: `<circle cx="5.5" cy="17" r="1.8"/><circle cx="10" cy="6.5" r="1.8"/><circle cx="18.5" cy="9.5" r="1.8"/><circle cx="15" cy="18.5" r="1.8"/><path d="M6.3 15.3l2.9-7M11.7 7.1l5.1 1.8M17.8 11.2l-2.1 5.6"/>`,
  tree: `<path d="M5 4v14a2 2 0 0 0 2 2h3M5 8h5"/><rect x="11" y="5.5" width="8" height="5" rx="1.5"/><rect x="11" y="16" width="8" height="5" rx="1.5"/>`,
  disk: `<circle cx="12" cy="12" r="8.5"/><circle cx="12" cy="12" r="3"/><path d="M12 3.5V9M18.4 15.5l-3.9-2.2"/>`,
  settings: `<circle cx="12" cy="12" r="3"/><path d="M12 2.5v3M12 18.5v3M2.5 12h3M18.5 12h3M5.3 5.3l2.1 2.1M16.6 16.6l2.1 2.1M5.3 18.7l2.1-2.1M16.6 7.4l2.1-2.1"/>`,
  search: `<circle cx="11" cy="11" r="6.5"/><path d="M16 16l4.5 4.5"/>`,
  log: `<path d="M4 6h11M4 12h6M4 18h8"/><circle cx="17" cy="16" r="4"/><path d="M17 14.2v1.9l1.2 1"/>`,
  paths: `<path d="M3.5 7.5a2 2 0 0 1 2-2h3.8l2 2.5h7.2a2 2 0 0 1 2 2v7.5a2 2 0 0 1-2 2h-13a2 2 0 0 1-2-2z"/>`,
};
// which folder this page shows; a ranged folder (run --month/--day/--from/--to) says so, so it is never read as the full index
const rangeChip = () => D.range ? ` <span class="rchip">${esc(D.range.label)} only</span>` : "";
const scopeLine = () => `<p class="scope">${D.range ? `<b>Only ${esc(D.range.human)}</b> · ` : ""}<code>${esc(D.out || "")}</code></p>`;
const agoText = d => d < 90 ? "just now" : d < 5400 ? Math.round(d / 60) + " min ago" : d < 129600 ? Math.round(d / 3600) + " h ago" : Math.round(d / 86400) + " days ago";
