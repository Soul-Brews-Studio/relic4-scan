# Design

Two surfaces share one look: the app (the page `relic-scan` serves on 127.0.0.1, also inside the desktop app) and the
landing page in `site/`. Both are dark first, with one warm accent.

## The mark

A serif **Ψ** on a filled circle, after the ARRA Oracle Unconference #1 logo (there a cream Ψ in Instrument Serif on a red
circle). Here the circle is the amber accent and the Ψ is the ink on it (`--on-amber` in the app, `--color-accent-ink` on the
site). It is drawn as SVG text with a serif stack (`'Instrument Serif', Georgia, 'Times New Roman', serif`), so it needs no
font file; the site's favicon is the same SVG.

## The app (`src/page/`)

- **Mode: operate.** People come to find a session, see what changed, or read where the bytes are. Data comes before
  decoration: numbers in mono, labels in the UI sans, no hero.
- **Offline on purpose.** System font stacks only (`-apple-system, system-ui` for chrome, `ui-monospace, "SF Mono", Menlo` for
  data). No font downloads, no CDN, no network calls.
- **Tokens** live in `src/page/shell.css` (`:root`, with a `[data-theme="light"]` override):

  | token | dark | role |
  |---|---|---|
  | `--bg` | `#05070d` | canvas |
  | `--ink` / `--dim` | `#d9dde8` / `#7c8496` | text / secondary text |
  | `--line` | `#1c2333` | hairlines |
  | `--amber` | `#f5b73b` | the brand accent: primary action, the active item, the logo |
  | `--cyan` / `--green` | `#67c8ef` / `#6fd6a0` | data series, success |
  | `--sp-1` ... `--sp-7` | 4, 8, 12, 16, 24, 32, 48 px | spacing scale |
  | `--ease-out`, `--dur-short` | `cubic-bezier(.2,.7,.2,1)`, 120 ms | the one motion curve |

- **Kinds of row** keep fixed colours everywhere (donut, bars, tree, popup), so a colour means the same thing on every view.
- **Popups** go through `overlay()` in `core.js`; every write to the server through `relicPost()`; bytes through `human()`;
  percentages through `pct()`.
- **No eyebrows** over headings, no gradient text, short subtitles. A view title says what the view is.
- No file is longer than 500 lines.

## The landing page (`site/public/scan/`)

- **Mode: persuade.** Structure: a workbench — real captures of the app are the content, each with a short caption; a
  download bar appears after the third capture and leaves at the download section.
- **Captures** come from a synthetic demo index of made-up repositories (`acme/*`, agent `dev@laptop`), never from a real
  machine. They are window captures of the desktop app, converted to WebP at 2000 px.
- **Type:** Bricolage Grotesque for headings and text (its optical size axis tunes each size), JetBrains Mono for commands and data (plus a
  tiny subset of it for the box-drawing characters), all self-hosted in `site/public/scan/fonts/`.
- **Colour:** OKLCH tokens in the page's `:root`; paper `oklch(15% 0.012 262)`, accent amber `oklch(80% 0.15 75)`, the same
  hue as the app's `--amber`.
- **Copy:** plain and specific. Numbers on the page are measured and say where and when; no invented metrics, no
  testimonials, no price.
- **Checks:** no horizontal scroll at 320, 375, 414 and 768 px; no button text on two lines; headings never italic.

## How the landing page is built

- **No build step.** Preact 11 and htm load as ES modules from esm.sh through an import map; Tailwind v4 compiles in the
  browser from `@tailwindcss/browser`. Versions are pinned in `site/public/scan/index.html`.
- **Copy lives in one file**, `site/public/scan/content.js`; `app.js` renders it. `node site/seo.mjs` writes the same text into
  the page's JSON-LD (SoftwareApplication, FAQPage) and no-JavaScript fallback, and into `llms.txt`, `llms-full.txt`,
  `robots.txt` and `sitemap.xml`. Run it after editing `content.js`.
- **Layout:** centred and symmetric; every section is heading, text, an even row of points, then the capture. A floating Next
  button names the next section; a dot index sits on the right on wide screens.
- **Motion:** in-page links glide; text appears at once; captures fade in, and without Reduce Motion also rise and drift.

## Deploying the landing page

- `site/wrangler.jsonc`: a static-assets Cloudflare Worker on `relic.buildwithoracle.com` (`npx -y wrangler@4 deploy`
  from `site/`).
- `.github/workflows/pages.yml`: the same folder to GitHub Pages on a push to `main` that touches `site/public/`. GitHub
  Pages needs a public repository on the free plan.
