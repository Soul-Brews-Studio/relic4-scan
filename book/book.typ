// ─── BOOK VARIABLES — edit these, nothing else needs to change for a new book ───
#let book-kicker   = "เขียนโดย AI เพื่อ AI"                       // top line of the cover, always
#let book-cover-art = "/images/cover-art-pop.jpg"                // make-cover-art.sh output; AI-generated, say so in tagline + README
#let book-prompt   = "$ relic-scan run"          // the dim command line on the cover
#let book-title-1  = "Relic"                               // big, white
#let book-title-2  = "Scan"                              // big, accent colour
#let book-subtitle = [คู่มือทีละขั้น จากทรานสคริปต์ของเอเจนต์ ไปถึงดัชนีที่ถามได้]
#let book-tagline  = [ทุกคำสั่งรันจริง ทุกภาพคือแอปจริง บนบ้านเดโมที่ทุกอย่างเป็นของสมมติ ยกเว้นภาพปกที่ AI วาด — 2026-10-26]
#let book-author   = [Claude (AI, ไม่ใช่คน) กับ Nat]
#let book-credit   = [Soul-Brews-Studio/relic4-scan · relic-scan 26.10.26-alpha.516 · AGPL v3]

// Page setup
#set page(
  paper: "a4",
  margin: (top: 2.5cm, bottom: 2.5cm, left: 3cm, right: 3cm),
  numbering: "1",
)
#set text(font: "Laksaman", size: 12pt, lang: "th")
#set heading(numbering: none)
// typst >= 0.13 spaces paragraphs with par.spacing, NOT block.spacing —
// setting only block.spacing leaves Thai paragraphs as one continuous
// slab (verified by rendering pages 6 and 8, 2026-09-13)
#set par(leading: 0.95em, spacing: 1.9em, justify: false, first-line-indent: 0em)
#set block(spacing: 1.9em)

// Chapter headings — page break before
#show heading.where(level: 1): it => {
  pagebreak(weak: true)
  set text(size: 20pt, weight: "bold", fill: rgb("#1a1a2e"))
  v(2em)
  it
  v(1em)
}

// Section headings
#show heading.where(level: 2): it => {
  set text(size: 14pt, weight: "bold", fill: rgb("#2c3e50"))
  v(1em)
  it
  v(0.5em)
}

#show heading.where(level: 3): it => {
  set text(size: 12pt, weight: "bold", fill: rgb("#2c3e50"))
  v(0.8em)
  it
  v(0.4em)
}

// Code blocks — mono + background
#show raw.where(block: true): it => {
  set text(font: "Menlo", size: 9pt)
  set par(leading: 0.65em)
  block(
    fill: rgb("#f6f8fa"),
    stroke: 0.5pt + luma(200),
    inset: 14pt,
    radius: 4pt,
    width: 100%,
    it,
  )
}

// Inline code — subtle grey background + charcoal
#show raw.where(block: false): it => {
  box(
    fill: rgb("#f0f0f0"),
    inset: (x: 3pt, y: 1.5pt),
    radius: 2pt,
    text(font: "Menlo", size: 9pt, fill: rgb("#36454f"), it),
  )
}

// Bold — dark navy
#show strong: it => text(weight: "bold", fill: rgb("#1a1a2e"), it)

// Blockquotes — blue left border + light blue background
#show quote.where(block: true): it => {
  block(
    fill: rgb("#f0f4f8"),
    stroke: (left: 3pt + rgb("#3498db")),
    inset: (left: 16pt, right: 12pt, top: 10pt, bottom: 10pt),
    radius: (right: 4pt),
    it,
  )
}

// Tables — dark header + zebra stripes
#set table(
  stroke: 0.5pt + luma(180),
  fill: (_, row) => if row == 0 { rgb("#2c3e50") } else if calc.odd(row) {
    rgb("#f8f9fa")
  } else { white },
)
#show table.cell: it => {
  set text(size: 10pt)
  set align(left)
  if it.y == 0 {
    set text(fill: white, weight: "bold")
    it
  } else { it }
}

// ─── IMAGES ────────────────────────────────────────────────────────
// Every image is a terminal capture or a diagram card rendered from text by
// /ascii-diagram --png (1300-1600 px wide, no DPI). Without a width typst would
// place them at 72 dpi and overflow the page, so all images take the text width.
#set image(width: 100%)
#show figure: set block(breakable: false)
#show figure.caption: it => text(size: 9.5pt, fill: rgb("#545A51"), it)

// ─── COVER ─────────────────────────────────────────────────────────
// DEFAULT LAYOUT "C" (Nat chose it over A/B, kvmbox-oracle 2026-10-03; reused 2026-10-05):
//   kicker "เขียนโดย AI เพื่อ AI" (18pt) → the story's command (16pt) → Codex art fading into the
//   dark → title (56pt) + promise line (36pt, the ONLY accent) → rule → subtitle → tagline → author.
// Art = images/cover-art.jpg from scripts/make-cover-art.sh. It is AI-generated, NOT a photograph:
// the tagline must say so ("ยกเว้นภาพปกที่ AI วาด"). Path starts with "/" because typst runs --root .
// from build/. Terminal palette of the book's own images (#0d1117 / #79c0ff / #e6edf3).
// book-prompt is a STRING passed to text(): no markup parsing, so -- stays --, and no
// inline-code box (raw() would pick up the body's light #f0f0f0 code style on the dark cover).
#[
  // Punchier cover (Nat 2026-10-14: "more sexy and punching popup"): full-bleed art, a hot
  // kicker pill, a huge two-colour title, the app's own red "Remove from heat!" disc as a
  // sticker, and the three readout colours. Still layout C's order and its AI disclosure.
  #let bg = rgb("0b0b0d")
  #let fg = rgb("f5f2ee")
  #let ember = rgb("ff6a3d")
  #let hot = rgb("d93a2b")
  #let dim = rgb("9a948c")
  #let latin = "Avenir Next"
  #set page(paper: "a4", margin: 0pt, fill: bg, numbering: none)
  #set text(font: "Laksaman", lang: "th", fill: fg)
  #set par(leading: 0.65em, spacing: 0pt)

  #place(top + left, dx: -1mm, dy: -1mm, image(book-cover-art, width: 212mm, height: 150mm, fit: "cover"))
  // the fade starts BELOW the art's subject (Cat Lab: steak + probe + phone end at ~128 mm) — a fade
  // from 84 mm dimmed the subject itself and the cover "did not pop" (Nat 2026-10-14)
  #place(top + left, dy: 128mm, rect(width: 212mm, height: 26mm, stroke: none,
    fill: gradient.linear((bg.transparentize(100%), 0%), (bg, 100%), angle: 90deg)))
  #place(top + left, rect(width: 212mm, height: 40mm, stroke: none,
    fill: gradient.linear((bg.transparentize(30%), 0%), (bg.transparentize(100%), 100%), angle: 90deg)))

  // kicker pill
  #place(top + left, dx: 22mm, dy: 16mm, box(fill: ember, radius: 30pt, inset: (x: 12pt, y: 7pt),
    text(size: 15pt, weight: "bold", fill: rgb("1a0a05"), book-kicker)))
  #place(top + left, dx: 22mm, dy: 31mm, text(font: latin, size: 13pt, weight: "medium", fill: fg.transparentize(25%),
    tracking: 1.5pt, upper(book-prompt)))

  // the sticker: the giveaway (Nat 2026-10-14: "instead of Remove from heat say แจกโค้ดเต็ม")
  // Nat 2026-10-14: "cover image not pop, the new text covers it". The sticker goes in the art's
  // EMPTY dark side, NEVER over the subject. Art prompt puts the subject in the right half, so the
  // sticker sits left (dx 20, dy 50). If your art's subject is elsewhere, move the sticker, not the art.
  #place(top + left, dx: 20mm, dy: 50mm, rotate(-12deg, circle(radius: 30mm, fill: hot,
    stroke: 3.5pt + fg, inset: 0pt, align(center + horizon, stack(spacing: 3mm,
      box(width: 56mm, align(center, text(size: 30pt, weight: "bold", fill: white)[โอเพนซอร์ส])),
      text(font: latin, size: 13pt, weight: "bold", fill: white.transparentize(10%), tracking: 1pt)[AGPL v3])))))

  // title
  #place(top + left, dx: 20mm, dy: 150mm, text(font: latin, size: 88pt, weight: "bold", fill: fg, tracking: -2pt, book-title-1))
  #place(top + left, dx: 20mm, dy: 182mm, text(font: latin, size: 88pt, weight: "bold", fill: ember, tracking: -2pt, book-title-2))

  // the three readouts, as in the app
  #let disc(c, t, l) = box(width: 30mm, align(center, stack(spacing: 2mm,
    circle(radius: 9.5mm, fill: c, align(center + horizon, text(font: latin, size: 17pt, weight: "bold", fill: white, t))),
    text(size: 10pt, fill: dim, l))))
  #place(top + left, dx: 20mm, dy: 220mm, stack(dir: ltr, spacing: 3mm,
    disc(rgb("cc2ac7"), "144", "sessions"), disc(rgb("3f93f2"), "4", "months"), disc(rgb("3dbb52"), "3.2k", "rows")))

  #place(top + left, dx: 118mm, dy: 221mm, block(width: 76mm, text(size: 13.5pt, fill: fg, hyphenate: false, book-subtitle)))
  // second sticker: as big as the claim — what is inside
  #place(top + left, dx: 140mm, dy: 166mm, rotate(6deg, box(fill: fg, radius: 8pt, inset: (x: 14pt, y: 10pt),
    stack(spacing: 2.5mm,
      text(font: latin, size: 30pt, weight: "bold", fill: rgb("1a0a05"))[1,009],
      text(size: 14pt, weight: "bold", fill: rgb("1a0a05"))[บรรทัด · 4 ไฟล์]))))
  #place(top + left, dx: 22mm, dy: 255mm, block(width: 168mm, text(size: 10.5pt, fill: dim, book-tagline)))
  #place(top + left, dx: 22mm, dy: 268mm, text(size: 14pt, weight: "bold", fill: fg, book-author))
  #place(top + left, dx: 22mm, dy: 278mm, text(size: 9pt, fill: dim, book-credit))
]

#counter(page).update(1)

#outline(title: "สารบัญ", depth: 1)
