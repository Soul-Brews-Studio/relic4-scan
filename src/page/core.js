const D = /*DATA*/null;
const SERVED = location.protocol.startsWith("http");
// theme: auto (follows the system), dark or light; a choice is kept in localStorage
const THEME_PICK = localStorage.getItem("relic-theme") || "auto";
const LIGHT = THEME_PICK === "light" || (THEME_PICK === "auto" && matchMedia("(prefers-color-scheme: light)").matches);
document.documentElement.dataset.theme = LIGHT ? "light" : "dark";
// one page, three views: /map, /tree and /paths when served, ?view=tree|paths when opened as a file
// served: / is Smart Scan; opened as a file (no server, no index button): the map
const VIEWS = ["scans", "scan", "disk", "settings", "banks", "vaults", "export", "health", "map", "tree", "paths"];
let VIEW = VIEWS.find(v => location.pathname.endsWith("/" + v) || new URLSearchParams(location.search).get("view") === v)
  || (location.protocol.startsWith("http") && /\/$/.test(location.pathname) ? "scan" : "map");
const DASH = ["scan", "scans", "disk", "settings", "banks", "vaults", "export", "health"].includes(VIEW);
const S = D.s, N = S.sid.length, BKK = 7 * 3600;
const $ = id => document.getElementById(id);
const esc = s => String(s).replace(/[&<>"']/g, c => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" }[c]));
const fmt = n => n.toLocaleString("en-US");
const TIER = ["main", "subagent", "workflow-agent"];
// rows by type: [key in the data, label, colour]; errors are a subset of results
const TYPES = LIGHT
  ? [["user", "user text", "#c98a00"], ["asst", "assistant text", "#8a6d1f"], ["think", "thinking", "#c2407a"],
     ["calls", "tool calls", "#1f8ac0"], ["results", "tool results", "#245f9e"], ["errors", "tool errors", "#d4441c"]]
  : [["user", "user text", "#ffd166"], ["asst", "assistant text", "#ffe6a8"], ["think", "thinking", "#ff9ec7"],
     ["calls", "tool calls", "#7fd3ff"], ["results", "tool results", "#5aa9e6"], ["errors", "tool errors", "#ff7a59"]];
const TYPE_KEYS = new Set(TYPES.map(t => t[0]).concat(["rows", "text", "msgs", "copies", "tools"]));
const tools = k => S.calls[k] + S.results[k];
const COLOR = LIGHT ? ["#b26b00", "#1f7fbf", "#6a4bd1"] : ["#ffe6a8", "#7fd3ff", "#b89cff"];
// canvas colours that are not data: rings, labels, links, the hover ring; stars add up as light on dark, ink on light
const INK = LIGHT
  ? { ring: "#e2e6ef", year: "#b9c2d6", month: "#8a93a8", label: "#4f5870", link: "rgba(60,80,140,.25)", sel: "rgba(212,68,28,.6)",
      hover: "#1c2233", hit: "#d4441c", off: "#c3c9d6", blend: "source-over", alpha: [0.85, 0.45, 0.06] }
  : { ring: "#141a28", year: "#2b3550", month: "#4a5470", label: "#6f7a96", link: "rgba(140,160,220,.18)", sel: "rgba(255,122,89,.55)",
      hover: "#ffffff", hit: "#ff7a59", off: "#3a4258", blend: "lighter", alpha: [0.8, 0.32, 0.04] };
const cat = k => S.cat[k] < 0 ? "(none)" : D.cats[S.cat[k]];
const proj = k => S.proj[k] < 0 ? "" : D.projs[S.proj[k]];
const bkk = t => t == null ? "—" : new Date((t + BKK) * 1000).toISOString().slice(0, 16).replace("T", " ");

// ---------- layout: ring = month, sector = repo, satellites orbit their parent ----------
function hash(str, salt) {
  let h = 2166136261 ^ salt;
  for (let i = 0; i < str.length; i++) { h ^= str.charCodeAt(i); h = Math.imul(h, 16777619); }
  // murmur3 finaliser: without it, sequential ids (agent-…01, …02) land on a straight line
  h ^= h >>> 16; h = Math.imul(h, 0x85ebca6b); h ^= h >>> 13; h = Math.imul(h, 0xc2b2ae35); h ^= h >>> 16;
  return (h >>> 0) / 4294967296;
}
const monthIdx = t => { const d = new Date((t + BKK) * 1000); return d.getUTCFullYear() * 12 + d.getUTCMonth(); };
const mLabel = m => `${Math.floor(m / 12)}-${String(m % 12 + 1).padStart(2, "0")}`;
let m0 = Infinity, m1 = -Infinity;
for (let k = 0; k < N; k++) if (S.t0[k] != null) { const m = monthIdx(S.t0[k]); m0 = Math.min(m0, m); m1 = Math.max(m1, m); }
const R0 = 140, DR = 64, RINGS = m1 - m0 + 1, ROUT = R0 + RINGS * DR;
const radius = t => {
  if (t == null) return ROUT + 46;
  const m = monthIdx(t), y = Math.floor(m / 12), mo = m % 12;
  const a = Date.UTC(y, mo, 1) / 1000 - BKK, b = Date.UTC(y, mo + 1, 1) / 1000 - BKK;
  return R0 + (m - m0 + (t - a) / (b - a)) * DR;
};
// sectors: repos sorted by sessions, width ~ sessions^0.8; repos under SMALL sessions share one sector
const SMALL = 12, OTHER = -2;
const perCat = new Map();
for (let k = 0; k < N; k++) perCat.set(S.cat[k], (perCat.get(S.cat[k]) || 0) + 1);
const secOf = new Map([...perCat].map(([c, n]) => [c, n < SMALL ? OTHER : c]));
const perSec = new Map();
for (const [c, n] of perCat) perSec.set(secOf.get(c), (perSec.get(secOf.get(c)) || 0) + n);
const cats = [...perSec.entries()].sort((a, b) => (a[0] === OTHER) - (b[0] === OTHER) || b[1] - a[1]);
const GAP = 0.006, total = cats.reduce((a, [, n]) => a + n ** 0.8, 0);
const sector = new Map();
let a0 = -Math.PI / 2 + 0.02;
for (const [c, n] of cats) {
  const w = (n ** 0.8 / total) * (2 * Math.PI - 0.04 - GAP * cats.length);
  sector.set(c, [a0, w, n]); a0 += w + GAP;
}
const smallRepos = [...perCat].filter(([, n]) => n < SMALL).length;
const secName = c => c === OTHER ? `${smallRepos} small repos` : c < 0 ? "(none)" : D.cats[c];
const X = new Float32Array(N), Y = new Float32Array(N), SIZE = new Float32Array(N);
const kids = Array.from({ length: N }, () => []);
for (let pass = 0; pass < 2; pass++) for (let k = 0; k < N; k++) {
  const sat = S.tier[k] > 0 && S.parent[k] >= 0;
  if ((pass === 0) === sat) continue;
  const lr = Math.log10(1 + S.rows[k]);
  if (sat) {
    const p = S.parent[k], o = 2.5 + 7 * hash(S.sid[k], 7), a = 2 * Math.PI * hash(S.sid[k], 3);
    X[k] = X[p] + o * Math.cos(a); Y[k] = Y[p] + o * Math.sin(a);
    SIZE[k] = 0.45 + 0.3 * lr; kids[p].push(k);
  } else {
    const [s0, w] = sector.get(secOf.get(S.cat[k])), a = s0 + w * (0.06 + 0.88 * hash(S.sid[k], 1)), r = radius(S.t0[k]);
    X[k] = r * Math.cos(a); Y[k] = r * Math.sin(a);
    SIZE[k] = S.tier[k] === 0 ? 0.8 + 0.75 * lr : 0.5 + 0.35 * lr;
  }
}

// ---------- shared helpers (used by several views) ----------
// bytes for people: "75.7 GB", "239.0 MB", "952 KB"
const human = b => b >= 1e9 ? (b / 1e9).toFixed(1) + " GB" : b >= 1e6 ? (b / 1e6).toFixed(1) + " MB" : b >= 1e3 ? Math.round(b / 1e3) + " KB" : b + " B";
// a link to a view (a page served by relic-scan, or ?view= when opened as a file), with an optional query
const KFILE = location.protocol === "file:";
const kHref = (v, hash) => (KFILE ? `?view=${v}` : `/${v}`) + (hash ? "#" + encodeURIComponent(hash) : "");
// gems: optional views, hidden until turned on in Settings (this browser only, like the theme)
const GEMS = { disk: ["Disk", "every .jsonl as rings by folder and size, DaisyDisk style"] };
const gemOn = g => (JSON.parse(localStorage.getItem("relic-gems") || "[]")).includes(g);
const gemSet = (g, on) => { const s = new Set(JSON.parse(localStorage.getItem("relic-gems") || "[]")); on ? s.add(g) : s.delete(g); localStorage.setItem("relic-gems", JSON.stringify([...s])); };
// a share for people: "37.1%", and "100%" (not "100.0%", which no column fits)
const pct = (v, t) => { const p = 100 * v / Math.max(1, t); return (p >= 99.95 ? "100" : p.toFixed(1)) + "%"; };
// every write the page asks of the server is a POST with the X-Relic header (a page on another site cannot send it)
const relicPost = path => fetch(path, { method: "POST", headers: { "X-Relic": "1" } });
// a popup over the page, made once: a scrim that closes it on click, one dialog box, Esc to close
function overlay(id, label, box, onClose) {
  let el = document.getElementById(id);
  if (el) return el;
  el = document.createElement("div"); el.id = id; el.className = "overlay"; el.hidden = true;
  el.innerHTML = `<div class="scrim"></div><div class="box" role="dialog" aria-modal="true" ${label}>${box}</div>`;
  document.body.append(el);
  el.close = () => { el.hidden = true; if (onClose) onClose(); };
  el.querySelector(".scrim").onclick = el.close;
  el.addEventListener("keydown", e => { if (e.key === "Escape") { e.preventDefault(); el.close(); } });
  return el;
}

// ---------- query ----------
const match = new Uint8Array(N).fill(1);
let active = false;
const TIERS = { main: 0, sub: 1, subagent: 1, wf: 2, workflow: 2, "workflow-agent": 2 };
function test(k, key, op, val) {
  const v = val.toLowerCase();
  switch (key) {
    case "tier": return S.tier[k] === TIERS[v];
    case "repo": case "cat": return cat(k).toLowerCase().includes(v);
    case "proj": return proj(k).toLowerCase().includes(v);
    case "sid": return S.sid[k].toLowerCase().startsWith(v);
    case "home": return S.homes[k].some(h => (D.homes[h].path + " " + D.homes[h].origin).toLowerCase().includes(v));
    case "month": return S.t0[k] != null && mLabel(monthIdx(S.t0[k])).startsWith(v);
    case "after": return S.t0[k] != null && bkk(S.t0[k]) >= v;
    case "before": return S.t0[k] != null && bkk(S.t0[k]) < v;
    case "rows": case "text": case "msgs": case "copies": case "user": case "asst": case "think": case "calls": case "results": case "errors": case "tools": {
      return cmp(key === "tools" ? tools(k) : S[key][k], op, Number(v));
    }
  }
  return true;
}
// terms: word · key:value · key>n · key:"value with spaces" · "phrase" · a leading - negates
function parseQuery(q) {
  return (q.match(/-?[a-z]+(?::|>=|<=|>|<|=)"[^"]*"?|-?"[^"]*"?|\S+/gi) || []).map(t => {
    const neg = t.startsWith("-") && t.length > 1; if (neg) t = t.slice(1);
    const m = t.match(/^([a-z]+)(:|>=|<=|>|<|=)(.+)$/i);
    const unq = v => v.replace(/^"|"$/g, "");
    return m ? { neg, key: m[1].toLowerCase(), op: m[2], val: unq(m[3]) } : { neg, word: unq(t).toLowerCase() };
  }).filter(t => t.word !== "");
}
const cmp = (x, op, n) => op === ">" ? x > n : op === "<" ? x < n : op === ">=" ? x >= n : op === "<=" ? x <= n : x === n;
function run(q) {
  if (VIEW === "paths") return prun(q);
  const terms = parseQuery(q);
  active = terms.length > 0;
  for (let k = 0; k < N; k++) {
    let ok = true;
    for (const t of terms) {
      const r = t.word != null ? (cat(k) + " " + proj(k) + " " + S.sid[k]).toLowerCase().includes(t.word) : test(k, t.key, t.op, t.val);
      if (r === t.neg) { ok = false; break; }
    }
    match[k] = ok && tierOn[S.tier[k]] ? 1 : 0;
  }
  history.replaceState(null, "", location.pathname + location.search + (q ? "#" + encodeURIComponent(q) : ""));
  summary(); tabs();
  if (VIEW === "tree") renderTree(); else draw();
}
