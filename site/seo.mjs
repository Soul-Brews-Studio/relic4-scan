// Writes what search and answer engines read from public/scan/content.js, the same text the page shows:
//   public/scan/index.html  the JSON-LD (SoftwareApplication, FAQPage) and the no-JavaScript page, between seo markers
//   public/llms.txt         a short guide for language models (llmstxt.org), public/llms-full.txt  every word of the page
//   public/robots.txt, public/sitemap.xml
// Run after editing content.js:  node site/seo.mjs   (the page itself needs no build; this only refreshes the copies)
import { readFileSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { SITE, DL, HERO, PROOF, PROOF_NOTE, TERMINAL, STEPS, LOCAL, FAQ, GET, FOOTER } from "./public/scan/content.js";

const here = dirname(fileURLToPath(import.meta.url)), pub = join(here, "public"), root = new URL(SITE.url).origin;
const esc = s => String(s).replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/"/g, "&quot;");
const plain = s => s.replace(/`/g, "");
const code = s => esc(s).replace(/`([^`]+)`/g, "<code>$1</code>");

// ---- JSON-LD: what the product is, and the questions it answers ----
const ld = [
  { "@context": "https://schema.org", "@type": "SoftwareApplication", name: SITE.name, url: SITE.url, description: SITE.summary,
    applicationCategory: "DeveloperApplication", operatingSystem: "macOS (Apple silicon), Linux (x86_64)", softwareVersion: SITE.version,
    downloadUrl: [DL.mac, DL.deb, DL.appimage], releaseNotes: DL.notes, image: `${SITE.url}img/og.jpg`,
    screenshot: ["scan", "scans", "scan-contents", "map", "tree", "disk"].map(n => `${SITE.url}img/${n}.webp`),
    featureList: STEPS.map(s => `${s.h}: ${plain(s.p)}`),
    publisher: { "@type": "Organization", name: SITE.org, url: "https://buildwithoracle.com/" } },
  { "@context": "https://schema.org", "@type": "FAQPage",
    mainEntity: FAQ.map(([q, a]) => ({ "@type": "Question", name: q, acceptedAnswer: { "@type": "Answer", text: a } })) },
];
const jsonld = ld.map(o => `<script type="application/ld+json">\n${JSON.stringify(o, null, 1).replace(/</g, "\\u003c")}\n</script>`).join("\n");

// ---- the page without JavaScript: the same words, plain HTML ----
const noscript = `<main style="max-width:68ch;margin:6vh auto;padding:0 24px;font-family:system-ui;line-height:1.6">
<h1>${esc(SITE.tagline)}</h1>
<p>${code(HERO.lede)} <strong>${esc(HERO.claim)}</strong> ${esc(HERO.close)}</p>
<p><a href="${DL.mac}">Download for macOS</a> · <a href="#get">Linux and the CLI</a><br><small>${esc(HERO.note)}</small></p>
<h2>Measured</h2><ul>${PROOF.map(([b, t]) => `<li><b>${esc(b)}</b> ${esc(t)}</li>`).join("")}</ul><p><small>${esc(PROOF_NOTE)}</small></p>
<h2>${esc(TERMINAL.h)}</h2><p>${code(TERMINAL.p)}</p><p>${code(TERMINAL.keys)}</p>
${STEPS.map(s => `<h2>${esc(s.h)}</h2><p>${code(s.p)}</p><ul>${s.li.map(l => `<li>${code(l)}</li>`).join("")}</ul>`).join("\n")}
<h2>${esc(LOCAL.h)}</h2><ul>${LOCAL.facts.map(([b, t]) => `<li><b>${esc(b)}</b>: ${code(t)}</li>`).join("")}</ul>
<h2>Questions</h2><dl>${FAQ.map(([q, a]) => `<dt><b>${esc(q)}</b></dt><dd>${esc(a)}</dd>`).join("")}</dl>
<h2 id="get">Download</h2><p>${esc(GET.p)}</p><ul>${GET.rows.map(([os, sub, meta, href]) => `<li><a href="${href}">${esc(os)}: ${esc(meta)}</a> (${esc(sub)})</li>`).join("")}</ul>
<pre>${esc(GET.cli)}</pre>
<p>${esc(FOOTER.say)}</p>
</main>`;

const page = join(pub, "scan", "index.html");
let html = readFileSync(page, "utf8");
const put = (name, body) => {
  const re = new RegExp(`(<!-- seo:${name} -->)[\\s\\S]*?(<!-- /seo:${name} -->)`);
  if (!re.test(html)) throw new Error(`index.html has no <!-- seo:${name} --> markers`);
  html = html.replace(re, `$1\n${body}\n$2`);
};
put("jsonld", jsonld);
put("noscript", noscript);
writeFileSync(page, html);

// ---- llms.txt: the short guide; llms-full.txt: every word of the page as markdown ----
const md = [
  `# ${SITE.name}`, "", `> ${SITE.summary}`, "",
  `${HERO.lede.replace(/`/g, "`")} ${HERO.claim} ${HERO.close}`, "",
  `- Product page: ${SITE.url}`, `- Release ${SITE.version} (private beta; needs access to the Soul-Brews-Studio GitHub organisation): ${DL.notes}`,
  `- Made by ${SITE.org}: https://buildwithoracle.com/`, "",
  "## Docs", "", `- [Full page text](${root}/llms-full.txt): features, measured speed, privacy, FAQ and downloads`, "",
  "## Questions", "", ...FAQ.map(([q, a]) => `- **${q}** ${a}`), "",
].join("\n");
writeFileSync(join(pub, "llms.txt"), md);
const full = [
  `# ${SITE.name}: ${SITE.tagline}`, "", `> ${SITE.summary}`, "", `${HERO.lede} ${HERO.claim} ${HERO.close}`, "", HERO.note, "",
  "## Measured", "", ...PROOF.map(([b, t]) => `- **${b}** ${t}`), "", PROOF_NOTE, "",
  `## ${TERMINAL.h}`, "", TERMINAL.p, "", TERMINAL.keys, "",
  ...STEPS.flatMap(s => [`## ${s.h}`, "", s.p, "", ...s.li.map(l => `- ${l}`), ""]),
  `## ${LOCAL.h}`, "", ...LOCAL.facts.map(([b, t]) => `- **${b}**: ${t}`), "", "```", "$ relic-scan run", ...LOCAL.run, "", ...LOCAL.card, "```", "", LOCAL.caption, "",
  "## Questions", "", ...FAQ.flatMap(([q, a]) => [`### ${q}`, "", a, ""]),
  "## Download", "", GET.p, "", ...GET.rows.map(([os, sub, meta, href]) => `- [${os}, ${meta}](${href}): ${sub}`), "", "```sh", GET.cli, "```", "",
  FOOTER.say, "",
].join("\n");
writeFileSync(join(pub, "llms-full.txt"), full);
writeFileSync(join(pub, "robots.txt"), `# every crawler, search and AI alike, may read this site\nUser-agent: *\nAllow: /\n\nSitemap: ${root}/sitemap.xml\n`);
writeFileSync(join(pub, "sitemap.xml"), `<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
  <url><loc>${SITE.url}</loc><lastmod>${new Date().toISOString().slice(0, 10)}</lastmod></url>
  <url><loc>${root}/llms.txt</loc></url>
  <url><loc>${root}/llms-full.txt</loc></url>
</urlset>
`);
console.log(`seo: index.html (JSON-LD ${ld.length}, no-JS page ${noscript.length} B) · llms.txt ${md.length} B · llms-full.txt ${full.length} B · robots.txt · sitemap.xml`);
