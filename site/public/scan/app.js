// ARRA Relic Scan landing page: Preact + htm, styled with Tailwind compiled in the browser. No build step.
import { h, render } from "preact";
import { useEffect, useRef, useState } from "preact/hooks";
import htm from "htm";
import { SITE, DL, HERO, PROOF, PROOF_NOTE, TERMINAL, STEPS, SHOTS, TUI_ALT, LOCAL, FAQ, GET, BUILD, FOOTER } from "./content.js";
const html = htm.bind(h);


// every capture: the full window (2880 x 1800, shown at 2000 px) and, for phones, a crop of the part its caption is about
const ORDER = Object.keys(SHOTS);

const btn = "inline-flex items-center gap-2 min-h-11 px-6 rounded-full font-semibold text-sm leading-none whitespace-nowrap no-underline border transition-[background-color,border-color,transform] duration-150 ease-(--ease-soft) active:translate-y-px";
const btnPrimary = `${btn} bg-accent text-accent-ink border-transparent hover:bg-accent-hover`;
const btnGhost = `${btn} text-ink border-edge hover:bg-paper-2 hover:border-ink-2`;
const wrap = "w-[min(1180px,100%-48px)] mx-auto";
// `code` in content strings becomes <code>
const Code = ({ c }) => html`<code class="font-mono text-xs text-ink bg-paper-2 px-1.5 py-px rounded">${c}</code>`;
const md = t => t.split(/`([^`]+)`/).map((x, k) => k % 2 ? html`<${Code} c=${x} />` : x);
const Down = () => html`<svg viewBox="0 0 24 24" class="size-4" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" aria-hidden="true"><path d="M12 4v11M7 10l5 5 5-5M5 20h14"/></svg>`;

// ---------- the lightbox: click a capture to open it; click again to zoom at that point, move to look around ----------
let openShot = () => {};
function Lightbox() {
  const [i, setI] = useState(-1), [zoom, setZoom] = useState(false), [org, setOrg] = useState("50% 50%");
  const box = useRef(null), back = useRef(null);
  openShot = n => { back.current = document.activeElement; setZoom(false); setI(ORDER.indexOf(n)); };
  const close = () => { setI(-1); back.current?.focus?.(); };
  const step = d => { setZoom(false); setI(k => (k + d + ORDER.length) % ORDER.length); };
  useEffect(() => {
    if (i < 0) return;
    const root = document.documentElement, was = root.style.overflow;
    root.style.overflow = "hidden"; box.current?.focus();
    const key = e => { if (e.key === "Escape") close(); else if (e.key === "ArrowRight") step(1); else if (e.key === "ArrowLeft") step(-1); };
    addEventListener("keydown", key);
    return () => { root.style.overflow = was; removeEventListener("keydown", key); };
  }, [i >= 0]);
  if (i < 0) return null;
  const n = ORDER[i];
  const at = e => { const r = e.currentTarget.getBoundingClientRect(); return `${((e.clientX - r.left) / r.width * 100).toFixed(1)}% ${((e.clientY - r.top) / r.height * 100).toFixed(1)}%`; };
  return html`<div ref=${box} tabindex="-1" role="dialog" aria-modal="true" aria-label=${`Capture ${i + 1} of ${ORDER.length}, full size`}
      class="fixed inset-0 z-50 grid place-items-center bg-scrim backdrop-blur-sm outline-none" onClick=${e => e.target === e.currentTarget && close()}>
    <img src=${`img/${n}.webp`} alt=${SHOTS[n]} draggable="false"
      class=${`max-w-[94vw] max-h-[86vh] rounded-xl border border-line select-none transition-transform duration-200 ease-(--ease-soft) ${zoom ? "cursor-zoom-out touch-none" : "cursor-zoom-in"}`}
      style=${{ transform: zoom ? "scale(2.4)" : "none", transformOrigin: org }}
      onClick=${e => { setOrg(at(e)); setZoom(z => !z); }} onPointerMove=${e => zoom && setOrg(at(e))} />
    <div class="fixed bottom-4 inset-x-0 flex justify-center gap-2 text-sm text-ink-2">
      <button class=${btnGhost} onClick=${() => step(-1)} aria-label="Previous capture">←</button>
      <span class="self-center px-3 tabular-nums">${i + 1} / ${ORDER.length} · ${zoom ? "move to look around" : "click to zoom"}</span>
      <button class=${btnGhost} onClick=${() => step(1)} aria-label="Next capture">→</button>
    </div>
    <button class=${`${btnGhost} fixed top-4 right-4 bg-paper`} onClick=${close} aria-label="Close">Close · Esc</button>
  </div>`;
}

// a product capture: the real window with a hairline, opening into the lightbox
function Shot({ n, eager, caption, cls = "" }) {
  return html`<figure class=${`m-0 rise ${cls}`}>
    <button type="button" onClick=${() => openShot(n)} aria-label=${`Open full size: ${SHOTS[n]}`}
      class="block w-full p-0 border-0 bg-transparent rounded-xl cursor-zoom-in group">
      <picture>
        <source media="(max-width: 640px)" srcset=${`img/${n}-m.webp`} />
        <img src=${`img/${n}.webp`} width="2000" height="1250" alt="" loading=${eager ? "eager" : "lazy"} fetchpriority=${eager ? "high" : "auto"}
          class="block w-full h-auto rounded-xl border border-line bg-paper-2 transition-[border-color] duration-150 group-hover:border-edge" />
      </picture>
    </button>
    ${caption && html`<figcaption class="mt-3 text-sm text-dim max-w-[34em]">${caption}</figcaption>`}
  </figure>`;
}

// the terminal recording: plays while it is on screen, with a pause button for anyone who wants it still
function TuiVideo() {
  const v = useRef(null), [playing, setPlaying] = useState(false), [user, setUser] = useState(null);
  useEffect(() => {
    const io = new IntersectionObserver(([e]) => { const el = v.current; if (!el) return;
      if (e.isIntersecting && user !== "paused") el.play().catch(() => {}); else el.pause(); }, { threshold: 0.25 });
    io.observe(v.current); return () => io.disconnect();
  }, [user]);
  const toggle = () => { const el = v.current; if (el.paused) { setUser("playing"); el.play(); } else { setUser("paused"); el.pause(); } };
  return html`<figure class="m-0 relative rise">
    <video ref=${v} src="img/tui.mp4" poster="img/tui.webp" width="1400" height="820" autoplay muted loop playsinline preload="auto"
      onPlay=${() => setPlaying(true)} onPause=${() => setPlaying(false)}
      aria-label=${TUI_ALT}
      class="block w-full h-auto rounded-xl border border-line bg-paper-2"></video>
    <button onClick=${toggle} class=${`${btnGhost} absolute right-3 bottom-3 bg-paper min-h-11 px-4 max-md:px-3 max-md:text-xs`}
      aria-label=${playing ? "Pause the recording" : "Play the recording"}>${playing ? "❚❚ Pause" : "▶ Play"}</button>
  </figure>`;
}

const SECTIONS = [["top", "Top"], [STEPS[0].id, STEPS[0].dot], ["tui", "Terminal"], ["speed", "Speed"], ...STEPS.slice(1).map(s => [s.id, s.dot]), ["local", "Local only"], ["faq", "Questions"], ["get", "Download"], ["build", "Build it"]];

function useScroll() {
  const [y, setY] = useState(0);
  useEffect(() => { let q = 0; const on = () => { if (!q) q = requestAnimationFrame(() => { q = 0; setY(scrollY); }); };
    addEventListener("scroll", on, { passive: true }); addEventListener("resize", on); on(); return () => { removeEventListener("scroll", on); removeEventListener("resize", on); }; }, []);
  return y;
}

// the dot index: one dot per section, the current one amber, its name on hover or focus
// the section holding the middle of the screen (the last one once the page is scrolled to its end)
function current(y) {
  let cur = 0;
  SECTIONS.forEach(([id], k) => { const el = document.getElementById(id); if (el && el.getBoundingClientRect().top <= innerHeight / 2) cur = k; });
  return innerHeight + y >= document.documentElement.scrollHeight - 2 ? SECTIONS.length - 1 : cur;
}
function Dots({ y }) {
  const cur = current(y);
  return html`<nav aria-label="Sections" class="fixed z-15 right-2 top-1/2 -translate-y-1/2 max-[1260px]:hidden">
    <ol class="list-none m-0 py-2 px-0 grid">
      ${SECTIONS.map(([id, name], k) => html`<li><a href=${`#${id}`} aria-current=${String(k === cur)}
        class="group flex items-center justify-end gap-3 h-8 pl-3 no-underline">
        <span class="text-sm text-ink-2 whitespace-nowrap bg-paper-2 border border-line rounded-lg px-2 opacity-0 translate-x-1.5 pointer-events-none transition duration-150 group-hover:opacity-100 group-hover:translate-x-0 group-focus-visible:opacity-100 group-focus-visible:translate-x-0 max-[1320px]:hidden">${name}</span>
        <i class=${`block size-2 rounded-full transition duration-150 ${k === cur ? "bg-accent scale-[1.35]" : "bg-line ring-1 ring-inset ring-dim group-hover:bg-ink-2"}`}></i>
      </a></li>`)}
    </ol>
  </nav>`;
}

// every in-page link glides to its section (the owner asked for the motion even with Reduce Motion on: it is short and
// only ever follows a click) and moves focus there, so keyboard users land where they asked to go
function glide(e) {
  const a = e.target.closest('a[href^="#"]'); if (!a || e.defaultPrevented || e.metaKey || e.ctrlKey) return;
  const t = document.getElementById(a.hash.slice(1)); if (!t) return;
  e.preventDefault(); t.scrollIntoView({ behavior: "smooth", block: "start" });
  history.replaceState(null, "", a.hash); t.setAttribute("tabindex", "-1"); t.focus({ preventScroll: true });
}

const h2 = "m-0 mb-4 font-semibold text-2xl leading-tight tracking-[-0.015em]";
const center = "mx-auto max-w-[36em] text-center";

function Nav({ y }) {
  return html`<header class=${`fixed z-20 top-3 left-1/2 -translate-x-1/2 w-[min(1180px,100%-32px)] flex items-center justify-between gap-4 py-2 pr-2 pl-4 rounded-full border transition-colors duration-300
      ${y > 8 ? "bg-glass border-line backdrop-blur-md backdrop-saturate-150" : "border-transparent"}`}>
    <a href="#top" class="inline-flex items-center gap-3 min-h-11 font-semibold tracking-tight whitespace-nowrap no-underline text-ink">
      <svg viewBox="0 0 24 24" class="size-7 text-accent" aria-hidden="true"><circle cx="12" cy="12" r="12" fill="currentColor"/><text x="12" y="17.6" text-anchor="middle" font-family="'Instrument Serif', Georgia, 'Times New Roman', serif" font-size="17" font-weight="600" fill="var(--color-accent-ink)">Ψ</text></svg>
      <span><span class="max-[400px]:hidden">ARRA </span>Relic Scan</span></a>
    <nav aria-label="Page"><ul class="flex items-center gap-6 list-none m-0 p-0">
      <li class="max-[900px]:hidden"><a href="#check" class="inline-flex items-center min-h-11 text-sm text-ink-2 hover:text-ink no-underline transition-colors">Tour</a></li>
      <li class="max-[900px]:hidden"><a href="#local" class="inline-flex items-center min-h-11 text-sm text-ink-2 hover:text-ink no-underline transition-colors">Local only</a></li>
      <li class="max-[900px]:hidden"><a href="#faq" class="inline-flex items-center min-h-11 text-sm text-ink-2 hover:text-ink no-underline transition-colors">Questions</a></li>
      <li><a class=${`${btnPrimary} px-4`} href="#get">Download</a></li>
    </ul></nav>
  </header>`;
}

// the hero: the claim, centred, then the terminal recording playing under it
function Hero() {
  return html`<section id="top" class=${`${wrap} arrive pt-[136px] pb-14 text-center`}>
    <span class="inline-block font-mono text-sm text-ink-2 bg-paper-2 border border-line rounded-lg px-3 py-2 mb-6"><b class="text-accent font-medium">$</b> ${HERO.cmd}</span>
    <h1 class="mx-auto m-0 mb-6 max-w-[16ch] font-semibold text-display leading-[1.02] tracking-[-0.022em] [overflow-wrap:anywhere]">${SITE.tagline}</h1>
    <p class="mx-auto m-0 mb-8 max-w-[34em] text-lg leading-normal text-ink-2">${md(HERO.lede)} <strong class="text-ink font-semibold">${HERO.claim}</strong> ${HERO.close}</p>
    <div class="flex flex-wrap justify-center items-center gap-3">
      <a class=${btnPrimary} href="#get"><${Down} />Download</a>
      <a class=${btnGhost} href="#tui">See the terminal</a>
    </div>
    <p class="mx-auto mt-4 mb-0 max-w-[34em] text-sm text-dim">${HERO.note}</p>
  </section>`;
}

function Proof() {
  return html`<section id="speed" aria-label="Measured speed" class=${`${wrap} mt-14 py-14 border-y border-line text-center`}>
    <div class="grid grid-cols-2 md:grid-cols-4 gap-8">
      ${PROOF.map(([b, t]) => html`<div class=""><b class="block font-semibold text-2xl leading-tight tracking-[-0.02em] tabular-nums">${b}</b><span class="block mx-auto mt-2 max-w-[16em] text-sm text-ink-2">${t}</span></div>`)}
    </div>
    <p class="mx-auto mt-8 mb-0 max-w-[34em] text-sm text-dim">${PROOF_NOTE}</p>
  </section>`;
}

// one view of the app: what it is, centred; its points in an even row; the capture full width (first, for Smart Scan)
function Step({ s, first }) {
  const text = html`<div class=${center}><h2 class=${h2}>${s.h}</h2><p class="m-0 text-ink-2">${md(s.p)}</p></div>`;
  const points = html`<ul class=${`list-none mx-auto mt-8 p-0 max-w-[880px] grid gap-4 ${first ? "mb-0" : "mb-10"} ${s.li.length === 3 ? "sm:grid-cols-3" : "sm:grid-cols-2"}`}>
    ${s.li.map(t => html`<li class="text-center text-sm text-ink-2 px-2"><i class="block mx-auto mb-2 size-1.5 rounded-[2px] bg-accent" aria-hidden="true"></i>${md(t)}</li>`)}</ul>`;
  const shot = html`<div class="mx-auto max-w-[1080px]"><${Shot} n=${s.shot} eager=${first} cls=${first ? "" : "drift"} /></div>`;
  return html`<article id=${s.id} class=${first ? `${wrap} pb-24` : "mb-32"}>
    ${first ? html`${shot}<div class="mt-12">${text}</div>${points}` : html`${text}${points}${shot}`}
  </article>`;
}
const SmartScan = () => html`<${Step} s=${STEPS[0]} first />`;

// the terminal: the same index in relic-scan tui, centred, the recording playing under it
function Terminal() {
  return html`<section id="tui" aria-labelledby="tui-h" class=${`${wrap} pt-28`}>
    <div class=${center}><h2 id="tui-h" class=${h2}>${TERMINAL.h}</h2><p class="m-0 mb-3 text-ink-2">${md(TERMINAL.p)}</p>
      <p class="m-0 mb-10 mx-auto max-w-[30em] text-sm text-dim">${md(TERMINAL.keys)}</p></div>
    <div class="mx-auto max-w-[1080px]"><${TuiVideo} /></div>
  </section>`;
}

function Tour() {
  return html`<section id="tour" aria-label="Tour" class=${`${wrap} pt-32`}>
    ${STEPS.slice(1).map(s => html`<${Step} s=${s} />`)}
  </section>`;
}

// terminal output as wrapping lines and a marked summary, so it never needs a scrollbar of its own
const keep = t => t.split(" · ").map(x => x.replaceAll(" ", "\u00a0")).join(" · ");
function Local() {
  const ln = t => html`<p class="m-0 pl-[2ch] -indent-[2ch] [overflow-wrap:anywhere]">${keep(t)}</p>`;
  return html`<section id="local" aria-labelledby="local-h" class=${`${wrap} py-24 border-t border-line`}>
    <h2 id="local-h" class=${`${h2} text-center`}>${LOCAL.h}</h2>
    <ul class="list-none mx-auto mt-10 mb-12 p-0 grid sm:grid-cols-2 lg:grid-cols-4 gap-8 text-center">
      ${LOCAL.facts.map(([b, t]) => html`<li><b class="block font-mono font-medium text-sm text-accent mb-2">${b}</b><span class="block mx-auto max-w-[18em] text-sm text-ink-2">${md(t)}</span></li>`)}
    </ul>
    <figure class="m-0 mx-auto max-w-[760px]">
      <div role="group" aria-label="Output of relic-scan run" class="bg-paper-2 border border-line rounded-xl p-6 font-mono text-xs leading-relaxed text-ink-2">
        <p class="m-0"><span class="text-accent">$</span> <span class="text-ink">relic-scan run</span></p>
        ${LOCAL.run.map(ln)}
        <div class="mt-4 pl-4 border-l-2 border-accent">
          <p class="m-0 text-ink">RELIC·SCAN</p>
          ${LOCAL.card.map(ln)}
        </div>
      </div>
      <figcaption class="mx-auto mt-3 max-w-[34em] text-sm text-dim text-center">${LOCAL.caption}</figcaption>
    </figure>
  </section>`;
}

function Faq() {
  return html`<section id="faq" aria-labelledby="faq-h" class=${`${wrap} py-24 border-t border-line`}>
    <h2 id="faq-h" class=${`${h2} mb-12 text-center`}>Questions</h2>
    <dl class="m-0 mx-auto max-w-[980px] grid md:grid-cols-2 gap-x-14 gap-y-10">
      ${FAQ.map(([q, a]) => html`<div class=""><dt class="font-semibold text-lg leading-snug mb-2">${q}</dt><dd class="m-0 text-ink-2">${a}</dd></div>`)}
    </dl>
  </section>`;
}

function Get() {
  const row = (os, sub, meta, href, primary) => html`<li class="grid sm:grid-cols-[minmax(0,1.2fr)_minmax(0,1fr)_auto] gap-2 sm:gap-6 items-center px-6 py-4 border-t border-line first:border-t-0">
    <div class="font-semibold">${os}<span class="block font-normal text-sm text-dim">${sub}</span></div>
    ${meta}<a class=${primary ? btnPrimary : btnGhost} href=${href}>${href === DL.notes ? "Release notes" : "Download"}</a></li>`;
  const m = t => html`<div class="font-mono text-sm text-ink-2 [overflow-wrap:anywhere]">${t}</div>`;
  return html`<section id="get" aria-labelledby="get-h" class=${`${wrap} py-24 border-t border-line`}>
    <div class=${`${center}`}><h2 id="get-h" class=${h2}>Download</h2><p class="m-0 mb-10 text-ink-2">${GET.p}</p></div>
    <ul class="list-none mx-auto my-0 p-0 max-w-[880px] border border-line rounded-xl overflow-hidden">
      ${GET.rows.map(([os, sub, meta, href], k) => row(os, sub, m(meta), href, k === 0))}
      ${row("The command", "macOS or Linux · needs Rust and gh",
        html`<pre class="m-0 font-mono text-xs leading-normal text-ink-2 whitespace-pre-wrap [overflow-wrap:anywhere]">${GET.cli}</pre>`, DL.notes)}
    </ul>
  </section>`;
}

// from source: numbered steps, each with its commands in a wrapping block (no scrollbar of its own)
function Build() {
  return html`<section id="build" aria-labelledby="build-h" class=${`${wrap} py-24 border-t border-line`}>
    <div class=${center}><h2 id="build-h" class=${h2}>${BUILD.h}</h2><p class="m-0 mb-10 text-ink-2">${md(BUILD.p)}</p></div>
    <ol class="list-none mx-auto my-0 p-0 max-w-[880px] grid gap-4">
      ${BUILD.steps.map(([t, c], k) => html`<li class="grid sm:grid-cols-[minmax(0,15.5rem)_minmax(0,1fr)] gap-2 sm:gap-6 items-start">
        <div class="flex items-center gap-3 pt-3"><span class="grid place-items-center size-7 rounded-full bg-accent text-accent-ink text-sm font-semibold">${k + 1}</span><b class="font-semibold">${t}</b></div>
        <pre class="m-0 bg-paper-2 border border-line rounded-xl px-5 py-3 font-mono text-xs leading-relaxed text-ink whitespace-pre-wrap [overflow-wrap:anywhere]">${c.split("\n").map(l => { const [cmd, note] = l.split(/\s{2,}#\s/);
          return html`<span class="block"><span class="text-accent">$ </span>${cmd}${note ? html`<span class="text-dim">  # ${note}</span>` : ""}</span>`; })}</pre>
      </li>`)}
    </ol>
    <p class="mx-auto mt-8 mb-0 max-w-[36em] text-sm text-dim text-center">${md(BUILD.more)}</p>
  </section>`;
}

function Footer() {
  return html`<footer class=${`${wrap} pt-28 pb-14 border-t border-line text-center`}>
    <p class="mx-auto m-0 mb-8 max-w-[22ch] font-semibold text-[clamp(1.75rem,2.6vw+1rem,2.75rem)] leading-[1.34] tracking-[-0.018em]">${FOOTER.say}</p>
    <div class="flex flex-wrap justify-center items-center gap-x-6 gap-y-2 text-sm text-dim"><span>ARRA Relic Scan · Soul Brews Studio</span>
      <a class="inline-flex items-center min-h-11 -my-2.5 text-ink-2" href="https://buildwithoracle.com/">buildwithoracle.com</a>
      <span>${FOOTER.note}</span></div>
  </footer>`;
}

// a floating "next" that names where it goes, so the page reads as a walk through, one section at a time
function Next({ y }) {
  const cur = current(y), last = cur >= SECTIONS.length - 1, [id, name] = last ? SECTIONS[0] : SECTIONS[cur + 1];
  return html`<a href=${`#${id}`} aria-label=${last ? "Back to the top" : `Next section: ${name}`}
    class="group fixed z-10 right-4 bottom-4 inline-flex items-center gap-2 min-h-11 pl-4 pr-3 rounded-full border border-line bg-paper text-sm text-ink-2 no-underline shadow-none transition-[border-color,color,opacity] duration-300 hover:border-edge hover:text-ink max-sm:right-3 max-sm:bottom-3">
    <span class="text-dim group-hover:text-ink-2 max-sm:hidden">${last ? "Back to" : "Next"}</span>
    <span key=${id} class="font-semibold text-ink [animation:fade-in_400ms_ease-out]">${last ? "Top" : name}</span>
    <span class="grid place-items-center size-7 rounded-full bg-accent text-accent-ink" aria-hidden="true">${last ? "↑" : "↓"}</span>
  </a>`;
}

function App() {
  const y = useScroll();
  // captures fade (and, without Reduce Motion, rise) into place the first time they come into view; text is there at once
  useEffect(() => {
    const io = new IntersectionObserver(es => es.forEach(e => { if (e.isIntersecting) { e.target.classList.add("in"); io.unobserve(e.target); } }), { threshold: 0.12 });
    document.querySelectorAll(".rise").forEach(el => { const k = [...el.parentElement.children].indexOf(el); el.style.transitionDelay = Math.min(k, 4) * 70 + "ms"; io.observe(el); });
    document.addEventListener("click", glide);
    return () => { io.disconnect(); document.removeEventListener("click", glide); };
  }, []);
  return html`<a href="#main" class="fixed z-30 left-4 top-4 px-4 py-3 rounded-lg bg-accent text-accent-ink font-semibold no-underline -translate-y-[200%] focus-visible:translate-y-0">Skip to the page</a>
    <${Nav} y=${y} />
    <main id="main"><${Hero} /><${SmartScan} /><${Terminal} /><${Proof} /><${Tour} /><${Local} /><${Faq} /><${Get} /><${Build} /></main>
    <${Dots} y=${y} /><${Footer} /><${Next} y=${y} /><${Lightbox} />`;
}

document.getElementById("app").textContent = "";
render(html`<${App} />`, document.getElementById("app"));
