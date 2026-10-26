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
= คำนำ
<คำนำ>
ARRA Relic Scan เป็น​เครื่องมือ​ที่ทำงาน​เดียว ค้นหา​ทรานสคริปต์​ของ​เอ​เจนต์ AI
ทุก​ไฟล์​ใน​เครื่อง แล้ว​เก็บ​เป็น​ดัชนี​ที่​ถาม​ได้ ไม่​ว่า​จะ​เป็น Claude Code, Codex, omp
หรือ​ตัว​อื่น ผลลัพธ์​คือ​ไฟล์ Parquet หนึ่ง​ไฟล์​ต่อ​หนึ่ง​เดือน อยู่​ใน `~/.relic-scan`
และ​เครื่องมือ​เปิด​ดู​ได้​สาม​ทาง ได้แก่ หน้า​เว็บ​ใน​เครื่อง เทอร์มินัล และ DuckDB

#figure(image("/images/C1-smart-scan-donut.png", width: 100%, alt: "หน้า Smart Scan ของ​บ้าน​เดโม: โดนัท​แสดง​แถว 3,173 แถว แยก​เป็น tool calls, user text, AI text, tool results, thinking และ tool errors"),
  caption: [
    หน้า Smart Scan ของ​บ้าน​เดโม: โดนัท​แสดง​แถว 3,173 แถว แยก​เป็น tool calls,
    user text, AI text, tool results, thinking และ tool errors
  ]
)

ภาพ​บน​คือ​ตัวอย่าง​ของ​สิ่ง​ที่​ได้ เป็น​หน้า Smart Scan ของ​บ้าน​เดโม
แถว​ทั้งหมด​ใน​ดัชนี​แยก​ตาม​ชนิด เล่ม​นี้​ไม่​มี​ภาพ​ที่​แต่ง​ขึ้น​นอกจาก​ภาพ​ปก

เล่ม​นี้​พา​เดิน​ทีละขั้น จาก​ติดตั้ง​ไป​จนถึง​ถาม​ว่า​เดือน​นี้​โมเดล​ไหน​ตอบ​ไป​กี่​ครั้ง ใช้​โท​เคน​เท่าไร
ทุก​คำสั่ง​ใน​เล่ม​รัน​จริง ทุก​ภาพ​คือ​หน้าจอ​จริง​ของ​แอ​ป

== ภาพ​มาจาก​ไหน
<ภาพมาจากไหน>
ทรานสคริปต์​จริง​เป็นงาน​ส่วนตัว จึง​ไม่​มี​สัก​บรรทัด​อยู่​ใน​เล่ม​นี้ ทุก​ภาพถ่าย​จาก
#strong[บ้าน​เดโม] ที่​สร้าง​ด้วย​สคริปต์ `make-demo-home.py` ใน​โฟลเดอร์ `book/`
ข้าง​เล่ม เป็น​ทรานสคริปต์ 144 ไฟล์​ของ repo สมมติ​หก​ตัว (tea-cart, starfish-api,
harbor-bot, lantern-app, kite-docs, moss-cli) ข้อความ คำถาม คำตอบ
และ​โมเดล​ทุก​ตัว​แต่ง​ขึ้น สคริปต์​ใช้ seed คงที่ ใคร​รัน​ก็ได้​บ้าน​เดโม​หน้าตา​เดิม

ใน​แคป​เจอร์ `~` จึง​หมายถึง​บ้าน​เดโม​นั้น ไม่​ใช่​โฟลเดอร์​ของ​ใคร ส่วน​ภาพ​หน้า​เว็บ​ถ่าย​จาก
`relic-scan serve` ที่​เปิด​ดัชนี​เดโม ผ่าน Chrome แบบ headless ขนาด 1280 × 860

== วิธี​อ่าน
<วธอาน>
- ภาค 1 ใช้งาน (บท 1–5): รัน​ดัชนี เปิด​หน้า​เว็บ เปิด​ใน​เทอร์มินัล
- ภาค 2 อ่าน​ดัชนี (บท 6–7): ถาม​ด้วย DuckDB และ​เรื่อง​เดือน​ที่​ปิดผนึก
- ภาค 3 ข้างใน (บท 8–9): chain ข้าม​เครื่อง ความเป็นส่วนตัว
- ภาคผนวก (บท 10–11): รูปแบบ​ของ transcript และ​โค้ด​ที่​เลือก

แต่ละ​ขั้น​มี​คำสั่ง​หรือ​หน้าจอ แล้ว​มี​ภาพ​ของ​ผล​จริงอยู่​ใต้​มัน อ่าน​แค่​ตัวหนังสือ​ก็​ทำตาม​ได้
ภาพ​ไว้​ยืนยัน​ว่า​เห็น​เหมือนกัน

== ใคร​เขียน
<ใครเขยน>
เล่ม​นี้​เขียน​โดย Claude ซึ่ง​เป็น AI กับ Nat คน​ที่​เป็นเจ้าของ​โครงการ ภาพ​ปก​เป็น​ภาพ​เดียว​ที่
AI วาด ที่​เหลือ​เป็น​ของจริง​ทั้งหมด

relic-scan เป็น​โอเพน​ซอร์ส (AGPL v3) โค้ด​อยู่​ที่
https:/\/github.com/Soul-Brews-Studio/relic4-scan หน้า​เว็บ​ของ​โครงการ​อยู่​ที่
https:/\/relic.buildwithoracle.com/scan/ เวอร์ชัน​ที่​ใช้​ถ่าย​ทั้ง​เล่ม​คือ
`26.10.26-alpha.516`

= บท​ที่ 1: ติดตั้ง​และ​รัน​ครั้งแรก
<บทท-1-ตดตงและรนครงแรก>
เครื่อง​ของ​คน​ที่​ใช้ AI เขียน​โค้ด​ทุกวัน มี transcript กอง​อยู่​หลาย​ที่ ทั้ง Claude Code,
Codex, omp แต่ละ​ตัวเก็บ​บทสนทนา​เป็น​ไฟล์ `.jsonl` ไว้​ใน​โฟลเดอร์​ซ่อน​ของ​ตัวเอง
พอ​ผ่าน​ไป​สาม​เดือน​ก็​ไม่​มี​ใคร​รู้​ว่า​อะไร​อยู่​ตรงไหน บท​นี้​พา​ติดตั้ง `relic-scan`
แล้ว​รัน​ครั้งแรก​ให้​เห็น​กับ​ตา​ว่า​มัน​หา​ของ​เจอ​ได้​ยังไง และ​เขียน​อะไร​ลง​ดิสก์​บ้าง

== 1.1 relic-scan คือ​อะไร และ​บ้าน​เดโม​ที่​ใช้​ตลอด​เล่ม
<relic-scan-คออะไร-และบานเดโมทใชตลอดเลม>
`relic-scan` เป็น​โปรแกรม​ฟรี (AGPL) ของ Soul-Brews-Studio ทำงาน​เดียว​คือ​หา
transcript ของ​เอ​เจนต์​ทุก​ตัว​บน​เครื่อง แล้ว​จัด​เป็น​ดัชนี​เดียว ดัชนี​นี้​เป็น​ไฟล์ Parquet
หนึ่ง​ไฟล์​ต่อ​หนึ่ง​เดือน (นับ​เดือน​ตามเวลา​กรุงเทพ) อยู่​ใต้ `~/.relic-scan`
จากนั้น​เปิด​ดู​ได้​สาม​ทาง คือ​หน้า​เว็บ​บน 127.0.0.1 หน้าจอ​ใน​เทอร์มินัล และ DuckDB ที่​ถาม​เป็น
SQL

มี​กติกา​ข้อ​หนึ่ง​ที่​ต้อง​รู้​ก่อน​เริ่ม #strong[transcript อ่าน​อย่าง​เดียว] โปรแกรม​ไม่​แก้ ไม่​ลบ
ไม่​ย้าย​ไฟล์​ของ​เอ​เจนต์ และ​เขียน​เฉพาะ​ใน `~/.relic-scan` เท่านั้น หน้า​เว็บ​ก็​ไม่​เรียก​เน็ต
ที่​ออก​เน็ต​ได้​มี​สอง​กรณี คือ `export --to s3://…` ที่​เรา​สั่ง​เอง กับ `ops` ที่ GET
สถานะ​ของ hub หนึ่ง​ครั้ง

เหตุ​ที่​ต้อง​มี​สาม​ทาง​เปิด​ดู เพราะ​คนใช้​ต่างกัน บางคน​อยาก​กวาดตา​ดู​ภาพรวม​ใน​หน้า​เว็บ
บางคน​อยู่​ใน ssh และ​ต้องการ​แค่​เทอร์มินัล ส่วน​คน​ที่​ชอบ SQL ก็​อยาก​ถาม​ตรง ๆ
ทั้ง​สาม​ทาง​อ่าน​ดัชนี​ชุด​เดียวกัน เลขที่​ได้​จึง​ตรงกัน​เสมอ
ไม่ต้อง​กลัว​ว่า​ทาง​ไหน​ให้​คำตอบ​ต่าง​จาก​ทาง​อื่น

ตลอด​เล่ม​นี้​ทุก​คำสั่ง​รัน​จริง​เมื่อ 2026-10-26 บน​เวอร์ชัน `26.10.26-alpha.516` และ​รัน​บน
#strong[บ้าน​เดโม] ที่​สร้าง​จาก​สคริปต์ `make-demo-home.py` มี transcript 144
ไฟล์​ของ repo สมมติ ไม่​มี​ของจริง​ปน​เลย ดังนั้น `~` ใน​ทุก​ภาพ​คือ​บ้าน​เดโม
ไม่​ใช่​บ้าน​ของ​ผู้เขียน

=== ขั้น​ที่ 1 — ติดตั้ง
<ขนท-1-ตดตง>
ติดตั้ง​จาก​ซอร์ส​ด้วย​สคริปต์​บรรทัด​เดียว ต้อง​มี Rust 1.89 ขึ้นไป​ใน​เครื่อง #strong[สคริปต์
build ราว 30 วินาที]แล้ว​วาง​ไฟล์​ไว้​ที่ `~/.local/bin/relic-scan`
ถ้า​ไม่​อยาก​คอมไพล์​เอง ก็​ไป​โหลด​แอ​ป​เดสก์ท็อป​จาก​หน้า Releases ของ GitHub ได้

`curl -fsSL https://raw.githubusercontent.com/Soul-Brews-Studio/relic4-scan/main/install.sh | sh`

คำสั่ง​นี้​ไม่​ได้​มาจาก​การ​รัน​ใน​บ้าน​เดโม จึง​ไม่​มี​ภาพประกอบ เป็น​คำสั่ง​จาก​ไฟล์ AGENTS.md
ของ​โปรเจกต์ ถ้า​คุณ​มี​ไบนารี​อยู่​แล้วก็​ข้าม​ไป​ขั้น​ต่อไป​ได้​เลย ตรวจ​แค่​ว่า `~/.local/bin` อยู่​ใน
`PATH`

=== ขั้น​ที่ 2 — ดู​บ้าน​เดโม​ก่อน​รัน
<ขนท-2-ดบานเดโมกอนรน>
ก่อน​สั่ง​อะไร​กับ relic-scan ลองดู​ว่า​ใน​บ้าน​มี​อะไร สั่ง `ls -a ~`
เพื่อให้​เห็น​โฟลเดอร์​ซ่อน​ด้วย

```bash
ls -a ~
```

#figure(image("/images/03-demo-home.png", width: 35%, alt: "ผล ls -a ของ​บ้าน​เดโม มี .claude .codex .omp .relic-scan และ Code"),
  caption: [
    ผล ls -a ของ​บ้าน​เดโม มี \.claude \.codex \.omp \.relic-scan และ Code
  ]
)

สาม​โฟลเดอร์​แรก​คือ #strong[บ้าน​ของ​เอ​เจนต์] `.claude` เป็น​ของ Claude Code,
`.codex` ของ Codex, `.omp` ของ omp ส่วน `Code` เป็น​ที่เก็บ repo และ​มี ψ vault
ซ่อน​อยู่​ใน​บาง​โปรเจกต์ ส่วน `.relic-scan` คือ​โฟลเดอร์​ผลลัพธ์ ตอนนี้​ยัง​ว่าง
เดี๋ยว​จะ​เห็น​มัน​ถูก​เติม​ใน​หัวข้อ 1.5

== 1.2 ดู​คำสั่ง​ทั้งหมด​และ​เวอร์ชัน
<ดคำสงทงหมดและเวอรชน>
relic-scan มี​คำสั่ง​ย่อย​เยอะ แต่​ที่​ใช้​จริง​บ่อย​มี​ไม่​กี่​ตัว ลอง​ไล่​ดู​จาก help
แล้​วจะ​เห็น​ว่า​แต่ละ​ตัว​อยู่​ตรงไหน​ของ​เล่ม​นี้

=== ขั้น​ที่ 1 — อ่าน help
<ขนท-1-อาน-help>
สั่ง `relic-scan --help` แล้ว​ดู​รายการ Commands ทีละ​บรรทัด

```bash
relic-scan --help
```

#figure(image("/images/01-help.png", width: 100%, alt: "ผล​ของ relic-scan --help แสดง​คำสั่ง scan run up serve export ops seals log tui meta map parse"),
  caption: [
    ผล​ของ relic-scan \-\-help แสดง​คำสั่ง scan run up serve export ops seals
    log tui meta map parse
  ]
)

คำสั่ง​ที่​ต้อง​จำ​มี​สี่​ตัว #strong[`run` คือ​คำสั่ง​หลัก] มัน​สแกน อ่าน​เฉพาะ​ส่วน​ที่​ใหม่ แล้ว​สร้าง
meta กับ​หน้า map ให้​ครบ `up` คือ `run` ตาม​ด้วย serve หน้า​เว็บ `tui`
เปิด​ดู​ใน​เทอร์มินัล และ `log` ดู​ประวัติการ​รัน ส่วน `scan`, `parse`, `meta`, `map`
เป็น​ขั้น​ย่อย​ที่ `run` เรียก​ต่อกัน​ให้​อยู่แล้ว ไม่ต้อง​สั่ง​เอง

สังเกต​บรรทัด​ของ `run` ใน help ว่า​เขียน​ไว้​ว่า Incremental คือ​รัน​ซ้ำ​ได้
ไม่ต้อง​กลัว​เสียเวลา บท​ที่ 2 จะ​พิสูจน์​ข้อ​นี้​ด้วย​ตัวเลข ส่วน `export` และ `seals` อยู่​บท​ที่ 7

=== ขั้น​ที่ 2 — ดู​เวอร์ชัน
<ขนท-2-ดเวอรชน>
เวอร์ชัน​สำคัญ​ตอน​รายงาน​ปัญหา และ​ตอน​เทียบ​ผล​กับ​เล่ม​นี้ ถ้า​เลข​ต่างกัน
ตัวเลข​ใน​ผล​ที่​คุณ​เห็น​อาจ​ไม่​ตรง​กับ​ที่​พิมพ์​ไว้

```bash
relic-scan --version
```

#figure(image("/images/02-version.png", width: 35%, alt: "ผล​ของ relic-scan --version คือ relic-scan 26.10.26-alpha.516"),
  caption: [
    ผล​ของ relic-scan \-\-version คือ relic-scan 26.10.26-alpha.516
  ]
)

```
relic-scan 26.10.26-alpha.516
```

เลข​เวอร์ชัน​เป็น​แบบ CalVer คือ​ปี เดือน วัน ตาม​ด้วย​เลข alpha ของ​รุ่น เล่ม​นี้​ใช้
#strong[26.10.26-alpha.516] ตลอด

== 1.3 run \-\-dry-run: ดูก่อน​ว่า​จะ​อ่าน​อะไร
<run---dry-run-ดกอนวาจะอานอะไร>
ครั้งแรก​ที่​รัน​บน​เครื่อง​จริง โปรแกรม​จะ​ไล่​อ่าน transcript ทุก​ไฟล์​ใต้​โฟลเดอร์​บ้าน
และ​เขียน​ดัชนี​ลง `~/.relic-scan` AGENTS.md
ของ​โปรเจกต์​บอก​ให้​ถาม​เจ้าของ​เครื่อง​ก่อน​รัน​ครั้งแรก​เสมอ
เพราะ​บน​เครื่อง​จริง​มัน​กินเวลา​และ​พื้นที่​ดิสก์ ทาง​ที่​ปลอดภัย​คือ​สั่ง dry run ก่อน

=== ขั้น​ที่ 1 — สั่ง dry run
<ขนท-1-สง-dry-run>
เติม `--dry-run` ต่อท้าย `run` ตัว​โปรแกรม​จะ​สแกน​หา​ไฟล์​เหมือน​จริง
แต่​หยุด​ก่อน​อ่าน​เนื้อหา และ​ไม่​เขียน​อะไร​ลง​ดิสก์

```bash
relic-scan run --dry-run
```

#figure(image("/images/04-dry-run.png", width: 93%, alt: "ผล dry run: new 144, 0.00 GB to read และ​กล่อง RELIC·SCAN บอ​กว่า nothing written"),
  caption: [
    ผล dry run: new 144, 0.00 GB to read และ​กล่อง RELIC·SCAN บอ​กว่า
    nothing written
  ]
)

บรรทัด​แรก​คือ​สรุป​ทั้งหมด อ่าน​เป็น​คู่ ๆ ตาม​นี้

```
run: scan 0.2s · new 144 · grown 0 · rewritten 0 · unchanged 0 · gone 0 …
```

แต่ละ​คำ​คือ #strong[สถานะ​ของ​ไฟล์​เทียบ​กับ​ครั้งก่อน] `new` คือ​ไฟล์​ที่​ไม่​เคย​เห็น `grown`
คือ​ไฟล์​ที่​เคย​อ่าน​แล้วแต่​มี​บรรทัด​ต่อท้าย `rewritten` คือ​ไฟล์​ที่​เนื้อหา​เดิม​เปลี่ยน `unchanged`
คือ​ไม่​เปลี่ยน และ `gone` คือ​ไฟล์​ที่​เคย​มี​แต่​หาย​ไป ตอนนี้​ยัง​ไม่​เคย​รัน เลย​เป็น new 144
ทั้งหมด

ตัวเลือก​ของ `run` ดู​ได้​จาก `relic-scan run --help`

```bash
relic-scan run --help
```

#figure(image("/images/40-help-run.png", width: 100%, alt: "ผล relic-scan run --help: รายการ​ตัวเลือก --out --root --depth --threads --exclude --full --kind --month --dry-run"),
  caption: [
    ผล relic-scan run \-\-help: รายการ​ตัวเลือก \-\-out \-\-root \-\-depth
    \\-\-threads \-\-exclude \-\-full \-\-kind \-\-month \-\-dry-run
  ]
)

ใน​ภาพ​มี `--out` กำหนด​โฟลเดอร์​ผลลัพธ์ `--root` เพิ่ม​โฟลเดอร์​ที่จะ​สแกน `--kind`
เลือก​เฉพาะ​บ้าน​ของ​เอ​เจนต์​ชนิด​เดียว `--month` เลือก​เฉพาะ​เดือน และ `--dry-run`
ที่​เพิ่ง​ใช้​ไป คำอธิบาย​บาง​บรรทัด​ถูก​ตัด​ตรง​ขอบ​ภาพ ส่วน​บท​ที่ 2 จะ​ใช้ `--kind` กับ
`--month` จริง

=== ขั้น​ที่ 2 — อ่าน​กล่อง RELIC·SCAN
<ขนท-2-อานกลอง-relicscan>
ท้าย​ผลลัพธ์​มี​กล่อง​เล็ก ๆ ที่​โปรแกรม​วาด​ไว้ ทุก​คำสั่ง​ที่​รัน​จริง​จะ​จบ​ด้วย​กล่อง​แบบนี้ รูปร่าง​เดียวกัน
เปลี่ยน​แค่​ข้อความ​ข้างใน

```
│ dry run · an index would read 144 files, 0.00 GB       │
│ scan 0.2s · nothing written                            │
│ → ~/.relic-scan                                        │
```

บรรทัด​ที่​ควร​อ่าน​ทุกครั้ง​คือ #strong[nothing written] กับ​ลูกศร​ที่​ชี้​ปลายทาง
ลูกศร​บอ​กว่า​ถ้า​รัน​จริง​จะ​เขียน​ที่ไหน คือ `~/.relic-scan` ส่วน `0.00 GB` ไม่​ใช่​ศูนย์​จริง
แต่ transcript เดโม​เล็ก​มาก ทั้งหมด​รวมกัน​แค่​ราว 1.4 MB (ตัวเลข​นี้​ไป​เห็น​อีกที​ใน​บท​ที่ 2)
ปัด​เป็น GB เลย​ออกมา​เป็น​ศูนย์

เทียบ​ให้​เห็น​สเกล ใน​เครื่อง Mac จริง​เครื่อง​หนึ่ง​ที่ AGENTS.md ยกตัวอย่าง​ไว้ มี 87,000 ไฟล์
รวม 75 GB ได้ 5.4 ล้าน​แถว และ index ครั้งแรก​ใช้​ราว​หนึ่ง​นาที
เล่ม​นี้​ไม่​อ้าง​ตัวเลข​เกินไป​กว่า​นี้

== 1.4 run ครั้งแรก: อ่าน​ผล​ทีละ​บรรทัด
<run-ครงแรก-อานผลทละบรรทด>
พอ dry run บอ​กว่า​เห็น​ไฟล์ 144 ไฟล์ ก็​ถึง​เวลา​ลงมือ​จริง คำสั่ง​เดียวกัน ตัด​แค่
`--dry-run` ออก ผลลัพธ์​ยาว​หน่อย แต่​แต่ละ​บรรทัด​ขึ้นต้น​ด้วย​ชื่อ​ขั้นตอน อ่าน​ตามลำดับ​ได้​เลย

=== ขั้น​ที่ 1 — สั่ง run
<ขนท-1-สง-run>
สั่ง `relic-scan run` บน​บ้าน​เดโม ใช้เวลา​รวม 1.3 วินาที

```bash
relic-scan run
```

#figure(image("/images/05-run-first.png", width: 100%, alt: "ผล relic-scan run ครั้งแรก ไล่​ขั้น scan, parse, merge, meta, vaults, map, done และ​กล่อง​สรุป"),
  caption: [
    ผล relic-scan run ครั้งแรก ไล่​ขั้น scan, parse, merge, meta, vaults, map,
    done และ​กล่อง​สรุป
  ]
)

#strong[ไล่​ทีละขั้น​ตามที่​โปรแกรม​พิมพ์] ขั้น `scan` คือ​บรรทัด​แรก เหมือนกับ dry run
ทุก​ตัวอักษร เจอ 144 ไฟล์​ใหม่ ขั้น​ถัดมา `parse` อ่าน​เนื้อ​ไฟล์​จริง

```
parse: 144 files · 0.00 GB · 3244 lines (0 not JSON) · 3171 raw rows · …
```

144 ไฟล์​มี 3244 บรรทัด ไม่​มี​บรรทัด​ไหน​ที่​ไม่​ใช่ JSON พอ​แปลง​แล้ว​ได้ 3171 แถว​ดิบ
ตัวเลข​สอง​ตัว​นี้​ไม่​เท่ากัน​เพราะ​บรรทัด​หนึ่ง​อาจ​ให้​หลาย​แถว หรือไม่​ให้​เลย​ก็ได้
เช่น​บรรทัด​ที่​เป็น​แค่​ข้อมูล​กำกับ บท​ที่ 10 จะ​อธิบาย​ว่า​บรรทัด​หนึ่ง​กลาย​เป็นแถว​ได้​ยังไง

=== ขั้น​ที่ 2 — ดู month ที่​ถูก​เขียน
<ขนท-2-ด-month-ทถกเขยน>
ใต้​บรรทัด parse มี​สี่​บรรทัด​ที่​เยื้อง​เข้าไป หนึ่ง​บรรทัด​ต่อ​หนึ่ง​เดือน ตรงนี้​คือ​จุด​ที่ relic-scan
จัดแถว​ลง​ไฟล์​ตามเวลา​กรุงเทพ

```
  2026-07: 819 rows (819 new copies merged) -> ~/.relic-scan/2026-07.parquet
  2026-08: 762 rows (762 new copies merged) -> ~/.relic-scan/2026-08.parquet
  2026-09: 845 rows (845 new copies merged) -> ~/.relic-scan/2026-09.parquet
  2026-10: 745 rows (745 new copies merged) -> ~/.relic-scan/2026-10.parquet
```

ไฟล์​เดือน​ที่​เขียน​แล้ว​ดู​ได้​ด้วย `ls -l`

```bash
ls -l ~/.relic-scan/*.parquet
```

#figure(image("/images/45-ls-months.png", width: 51%, alt: "ผล ls -l ของ​ไฟล์ parquet สี่​เดือน สาม​เดือน​แรก​เป็น -r--r--r-- และ​เดือน 2026-10 เป็น -rw-r--r--"),
  caption: [
    ผล ls -l ของ​ไฟล์ parquet สี่​เดือน สาม​เดือน​แรก​เป็น -r\-\-r\-\-r\-\- และ​เดือน
    2026-10 เป็น -rw-r\-\-r\-\-
  ]
)

สาม​เดือน​แรก (2026-07 ถึง 2026-09) ขึ้นต้น​ด้วย `-r--r--r--` คือ​อ่าน​ได้​อย่าง​เดียว
เพราะ​เป็น​เดือน​ที่​ผนึก​แล้ว ส่วน 2026-10 เป็น `-rw-r--r--`
ยัง​เขียน​ต่อ​ได้​เพราะ​เป็น​เดือน​ที่​เปิด​อยู่ (เรื่อง​ผนึก​อยู่​ใน​บท​ที่ 7)

บวก​สี่​เดือน​ได้ 3171 พอดี คือ 819 + 762 + 845 + 745 วงเล็บ `new copies merged`
บอ​กว่า​มี​กี่​แถว​ที่​เข้า​ไฟล์​ใหม่ ถ้า​ไฟล์​เดียวกัน​มี​สำเนา​ใน​หลาย​บ้าน (เช่น mirror หรือ backup)
มัน​จะ​ถูก #strong[พับ​เป็นแถว​เดียว] ใน​เดโม​ไม่​มี​สำเนา​ซ้ำ ตัวเลข​สอง​ฝั่ง​เลย​เท่ากัน

=== ขั้น​ที่ 3 — meta, vaults, map
<ขนท-3-meta-vaults-map>
หลังจาก merge มี​สาม​บรรทัด​ที่​สร้าง​ของ​เสริม​สำหรับ​หน้า​เว็บ ไล่​อ่าน​ทีละ​บรรทัด​แล้​วจะ​เห็น​ว่า
relic-scan เก็บ​อะไร​ไว้​นอกเหนือจาก​แถว​ข้อความ

```
meta: jsonl 145 (home 144 · ψ vault 0 in 0 vaults · other 1) in 0.0s -> …
vaults: 2 ψ vaults · 5 notes · 0.0 GB (+ 0 worktree copies) in 0.0s -> …
map: 144 sessions · 8 repos · 3 homes · 0.2 MB · 145 paths in 0.0s -> …
```

บรรทัด meta นับ​ไฟล์ `.jsonl` ที่​เห็น​ทั้งหมด 145 ไฟล์ เป็น​ของ​บ้าน​เอ​เจนต์ 144 ไฟล์ อีก 1
ไฟล์​เป็น other คือ `.jsonl` ที่​ไม่​ใช่ transcript ถูก​เก็บ​ไว้​ใน​รายการ​แต่​ไม่​นับ​เป็น
session บรรทัด vaults พบ ψ vault 2 แห่ง รวม 5 โน้ต ส่วน map สร้าง​หน้า​เว็บ
#strong[144 sessions จาก 8 repos ใน 3 บ้าน]
ตัวเลข​เหล่านี้​จะ​กลับมา​อีก​ใน​หน้า​เว็บ​บท​ที่ 3

=== ขั้น​ที่ 4 — อ่าน​กล่อง​สรุป
<ขนท-4-อานกลองสรป>
สุดท้าย​คือ​บรรทัด `run: done` กับ​กล่อง​สรุป กล่อง​นี้​คือ​ที่​ที่​ควร​มอง​ก่อน​อย่าง​อื่น
เพราะ​มัน​ตอบคำถาม​ว่า รอบ​นี้​ได้​อะไร​มา และ​ใช้เวลา​ไป​กับ​ขั้น​ไหน

```
│ 144 sessions · 3,171 rows · 144 files · 0.0 GB read              │
│ new 144 · grown 0 · rewritten 0 · unchanged 0                    │
│ scan 0.2s · parse+merge 1.1s · meta 0.0s · map 0.0s · total 1.3s │
```

#strong[3,171 แถว จาก 144 session] คือ​เลขที่​ควร​จำ มัน​จะ​ตรง​กับ​ตัวเลข​ใน​หน้า Smart
Scan หลัง​บท​ที่ 2 (ที่​มี​แถว​เพิ่ม​อีก 2) เวลา​ทั้งหมด 1.3 วินาที ส่วนใหญ่​หมด​ไป​กับ scan และ
parse ท้าย​กล่อง​มี​สอง​บรรทัด​ชี้ทาง​ต่อ คือ `relic-scan serve` กับ `relic-scan tui`
ทั้งสอง​คือ​ทาง​เปิด​ดู​ผล​ที่​เพิ่ง​ทำ​เสร็จ

== 1.5 อะไร​อยู่​ใน \~/.relic-scan
<อะไรอยใน-.relic-scan>
รัน​เสร็จ​แล้ว มา​ดู​ว่า​โปรแกรม​วาง​อะไร​ไว้​ใน​โฟลเดอร์​ผลลัพธ์ ใช้ `ls` ชี้​ทั้ง​โฟลเดอร์​หลัก​และ
`meta` ใน​คำสั่ง​เดียว

=== ขั้น​ที่ 1 — ไล่​ดู​โฟลเดอร์
<ขนท-1-ไลดโฟลเดอร>
สั่ง `ls` สอง​โฟลเดอร์​พร้อมกัน ผล​จะ​แยก​เป็น​สอง​กลุ่ม​โดย​มี​บรรทัด​ว่าง​คั่น

```bash
ls ~/.relic-scan ~/.relic-scan/meta
```

#figure(image("/images/06-out-folder.png", width: 41%, alt: "รายการ​ไฟล์​ใน ~/.relic-scan และ ~/.relic-scan/meta"),
  caption: [
    รายการ​ไฟล์​ใน \~/.relic-scan และ \~/.relic-scan/meta
  ]
)

กลุ่ม​แรก​คือ​โฟลเดอร์​หลัก #strong[ไฟล์​ขึ้นต้น​ด้วย `_` เป็น​สถานะ​ของ​โปรแกรม]
ส่วนที่เหลือ​คือ​ข้อมูล เรียง​ตามหน้าที่​ได้​ดังนี้

- `2026-07.parquet` ถึง `2026-10.parquet` คือ #strong[แถว​ของ​ข้อความ​ทั้งหมด]
  หนึ่ง​ไฟล์​ต่อ​เดือน ถาม​ด้วย DuckDB ได้​ตรง ๆ (บท​ที่ 6)
- `_state.tsv` จำ​ว่า​ไฟล์ transcript แต่ละ​ไฟล์​อ่าน​ไป​ถึง​ตรงไหน​แล้ว
  นี่​คือ​เหตุผล​ที่​รัน​ซ้ำ​เร็ว (ดู​ภาพ​ด้านล่าง)
- `_ledger.tsv` เป็น​สมุดบัญชี​ที่​ต่อท้าย​ทุกครั้งที่ index ไม่​เคย​ถูก​เขียน​ทับ
- `_scans.jsonl` คือ​บันทึก​การ​รัน ที่ `relic-scan log` อ่าน​ออกมา​แสดง
- `_config.json` เก็บ​ค่า​ตั้ง เช่น โฟลเดอร์​ที่​เดิน​สแกน (บท​ที่ 4)

`_state.tsv` เปิด​ดู​ได้​ตรง ๆ เพราะ​เป็น​ข้อความ​คั่น​ด้วย tab

```bash
head -3 ~/.relic-scan/_state.tsv | cut -f1,4,6,7
```

#figure(image("/images/43-state-tsv.png", width: 100%, alt: "ผล head ของ _state.tsv คอลัมน์ path size lines session สอง​ไฟล์ ไฟล์​ละ 34 บรรทัด"),
  caption: [
    ผล head ของ \_state.tsv คอลัมน์ path size lines session สอง​ไฟล์ ไฟล์​ละ
    34 บรรทัด
  ]
)

แต่ละ​แถว​คือ​หนึ่ง​ไฟล์ transcript มี path, ขนาด​เป็น​ไบต์, จำนวน​บรรทัด และ id ของ
session ใน​ภาพ​ทั้งสอง​ไฟล์​มี 34 บรรทัด ขนาด 15827 กับ 15904 ไบต์

=== ขั้น​ที่ 2 — ดู​ใน meta
<ขนท-2-ดใน-meta>
กลุ่ม​ที่สอง​คือ `meta/` ซึ่ง​เก็บ #strong[สรุป​ราย​ไฟล์​และ​ราย session] แยกจาก​แถว​ข้อความ
ทำ​ให้หน้า​เว็บ​เปิด​เร็ว​โดย​ไม่ต้อง​กวาด​ข้อความ​ทั้งหมด

- `sessions.parquet` หนึ่ง​แถว​ต่อ​หนึ่ง session มี repo, เวลา​แรก​และ​สุดท้าย,
  จำนวน​แถว
- `files.parquet` และ `jsonl.parquet` หนึ่ง​แถว​ต่อ​หนึ่ง​ไฟล์ transcript
- `months.json` ข้อมูล​ของ​แต่ละ​ไฟล์​เดือน รวมถึง​ผนึก​ของ​เดือน​ที่​ปิด​แล้ว
- `vaults.json` กับ​โฟลเดอร์ `vaults/` คือ​รายการ ψ vault และ​โน้ต​ข้างใน
- `map.html` คือ​หน้า​เว็บ​ทั้ง​หน้า สร้าง​ไว้​เป็น​ไฟล์​เดียว​ที่ `serve` ส่งออก​ไป
- `paths.json` และ `unreadable.json` รายการ path ที่​เห็น และ​ที่​อ่าน​ไม่​ได้

```bash
ls -l ~/.relic-scan/meta
```

#figure(image("/images/48-ls-meta.png", width: 38%, alt: "ผล ls -l ของ meta: files.parquet jsonl.parquet map.html months.json paths.json sessions.parquet unreadable.json vaults.json และ​โฟลเดอร์ cache กับ vaults"),
  caption: [
    ผล ls -l ของ meta: files.parquet jsonl.parquet map.html months.json
    paths.json sessions.parquet unreadable.json vaults.json และ​โฟลเดอร์
    cache กับ vaults
  ]
)

ใน​ภาพ `map.html` ใหญ่​สุด 217325 ไบต์ เพราะ​เป็น​หน้า​เว็บ​ทั้ง​หน้า ส่วน
`sessions.parquet` 26517 ไบต์ และ `months.json` 2063 ไบต์ มี `cache` กับ
`vaults` เป็น​โฟลเดอร์

ลอง​สังเกตว่า​ชื่อ​ไฟล์​เดือน​เรียง​ตาม​ตัวอักษร​ก็​เรียง​ตามเวลา​ไป​ใน​ตัว ทำให้​เวลา​ใช้ DuckDB
อ่าน​ด้วย `*.parquet` ก็ได้​ข้อมูล​ครบ​ทุก​เดือน​ใน​คำ​เดียว ส่วน​ไฟล์​ใน `meta/`
ไม่​ใช่​แถว​ข้อความ จึง​ไม่​ปน​เข้าไป​ใน​กลุ่ม​นั้น

จุด​ที่​ควร​จำ​คือ​ทุกอย่าง​อยู่​ใต้ `~/.relic-scan` ลบ​โฟลเดอร์​นี้​ทิ้ง​เมื่อไหร่​ก็ได้ transcript
ต้นทาง​ไม่​เสีย​อะไร แล้ว​สั่ง `relic-scan run` ใหม่​ก็ได้​ดัชนี​กลับมา AGENTS.md
แนะนำ​วิธี​นี้​ด้วยซ้ำ เมื่อ​ต้องการ​ดัชนี​รูปแบบ​ล่าสุด​จาก​ศูนย์ ให้​ย้าย​โฟลเดอร์​เก่า​ไป​ไว้​ข้าง​แล้ว​รัน​ใหม่

== ต่อไป
<ตอไป>
ตอนนี้​ดัชนี​เดโม​มี 3,171 แถว​นอน​อยู่​ใน `~/.relic-scan` แล้ว คำถาม​ถัดไป​คือ ถ้า​รัน​ซ้ำ​ทันที
มัน​จะ​อ่าน​ทุกอย่าง​ใหม่​ไหม และ​ถ้า transcript ไฟล์​หนึ่ง​โต​ขึ้น​สอง​บรรทัด จะ​เกิด​อะไร​ขึ้น บท​ที่
2 ตอบ​สอง​ข้อ​นี้​ด้วย​ตัวเลข​จาก​การ​รัน​จริง

= บท​ที่ 2: รัน​ซ้ำ อ่าน​แค่​ของ​ใหม่
<บทท-2-รนซำ-อานแคของใหม>
ดัชนี​ที่​สร้าง​ครั้งแรก​ใช้ 1.3 วินาที​บน​บ้าน​เดโม
แต่​บน​เครื่อง​จริง​ที่​มี​ข้อมูล​หลาย​แสน​บรรทัด​มัน​กินเวลา​เป็น​นาที
ถ้า​ทุกครั้งที่​อยาก​อัปเดต​ต้อง​อ่าน​ใหม่​หมด ก็​คง​ไม่​มี​ใคร​อยาก​รัน relic-scan บ่อย ๆ
บท​นี้​เลย​ทดลอง​สาม​อย่าง คือ​รัน​ซ้ำ​ทันที​โดย​ไม่​มี​อะไร​เปลี่ยน รัน​หลังจาก transcript
ไฟล์​หนึ่ง​โต​ขึ้น และ​รัน​เฉพาะ​บางส่วน​ของ​เครื่อง ทั้งหมด​ดู​จาก​ตัวเลข​ใน​ผลลัพธ์​จริง

== 2.1 รัน​ซ้ำ​ทันที: ไม่​อ่าน​อะไร​เลย
<รนซำทนท-ไมอานอะไรเลย>
หลักการ​ของ `run` คือ #strong[จำ​ว่า​อ่าน​ไฟล์​ไหน​ไป​ถึง​ไหน​แล้ว] ไฟล์ `_state.tsv`
ที่​เจอ​ใน​บท​ที่ 1 เก็บ​ตำแหน่ง​นั้น​ไว้ พอ​รัน​รอบ​ใหม่ โปรแกรม​สแกน​หา​ไฟล์​อี​กรอบ​เพื่อ​เทียบ
ถ้า​ไฟล์​ไหน​เหมือนเดิม​ก็​ข้าม เรา​จะ​พิสูจน์​ข้อ​นี้​ด้วย​การ​สั่ง​ซ้ำ​โดย​ยัง​ไม่​แตะ​อะไร​เลย

=== ขั้น​ที่ 1 — สั่ง run อี​กรอบ
<ขนท-1-สง-run-อกรอบ>
พิมพ์​คำสั่ง​เดียว​กับ​บท​ที่ 1 ทุก​ตัวอักษร ไม่ต้อง​ใส่​อะไร​เพิ่ม ระหว่าง​รอบ​แรก​กับ​รอบ​นี้​ยัง​ไม่​มี
transcript ใหม่​เกิดขึ้น

```bash
relic-scan run
```

#figure(image("/images/07-run-again.png", width: 100%, alt: "ผล run รอบ​ที่สอง: new 0, unchanged 144, total 0.3s ไม่​มี​บรรทัด parse"),
  caption: [
    ผล run รอบ​ที่สอง: new 0, unchanged 144, total 0.3s ไม่​มี​บรรทัด parse
  ]
)

เทียบ​กับ​รอบ​แรก จุด​ที่​ต่าง​มี​สาม​จุด จุด​แรก​คือ​บรรทัด​บนสุด​ที่​รอบ​แรก​ไม่​มี เป็น​บรรทัด​ของ ledger
ที่​เป็น​ศูนย์​ทั้ง​สี่​ช่อง

```
ledger: 0 copies (not read) · 0 read from a known version · 0 whose past …
run: scan 0.2s · new 0 · grown 0 · rewritten 0 · unchanged 144 · gone 0 …
```

จุด​ที่สอง​คือ​บรรทัด scan ซึ่ง​ตอนนี้​เป็น #strong[new 0 และ unchanged 144] ทั้งหมด
ไฟล์​ทุก​ไฟล์​ถูก​รู้จัก​แล้ว​และ​ไม่​มี​ไฟล์​ไหน​เปลี่ยน จุด​ที่สาม​คือ​หาย​ไป​ทั้ง​ก้อน ไม่​มี​บรรทัด `parse:`
ไม่​มี​บรรทัด​รายเดือน ไม่​มี `merge:` เพราะ​ไม่​มี​อะไร​ให้​อ่าน

=== ขั้น​ที่ 2 — ดู​ว่า​ขั้น​หลัง​รัน​ซ้ำ​อย่างไร
<ขนท-2-ดวาขนหลงรนซำอยางไร>
บรรทัด meta กับ map ยัง​โผล่​มา​ให้​เห็น​เหมือนเดิม แต่​ดู​ตัวเลข​ข้างใน​ดี​ดี

```
meta: 4 months (0 re-aggregated) in 0.0s · 144 files · 144 sessions …
run: done in 0.3s · state 144 files -> ~/.relic-scan/_state.tsv
```

รอบ​แรก​เขียน​ไว้​ว่า `4 months (4 re-aggregated)` คือ​สรุป​ทั้ง​สี่​เดือน​ใหม่ รอบ​นี้​เป็น
#strong[0 re-aggregated] ไม่​มี​เดือน​ไหน​ต้อง​สรุป​ซ้ำ ส่วน​แถว​สุดท้าย​ใน​กล่อง​ยัง​เป็น 3,171
แถว​เท่าเดิม และ​เวลา​รวม​เหลือ 0.3 วินาที จาก​ที่​รอบ​แรก​ใช้ 1.3 วินาที
เวลา​ที่​เหลือ​เกือบ​ทั้งหมด​คือ scan 0.2 วินาที คือ​โปรแกรม​เดิน​ดู​ว่า​ไฟล์​ไหน​เปลี่ยน​บ้าง

บรรทัด `map:` ยัง​ถูก​สร้าง​ใหม่​ทุกครั้ง แต่​ใช้เวลา 0.0 วินาที เพราะ​ไม่ต้อง​อ่าน​ข้อมูล​ก้อน​ใหญ่
ราคา​ของ​การ​รัน​ซ้ำ​จึง​เท่ากับ​ราคา​ของ​การ​เดิน​ดู​ไฟล์ เท่านั้นเอง

ข้อ​นี้​สำคัญ​ตอน​ตั้ง​ให้​รัน​อัตโนมัติ เช่น​ทุกคืน​ผ่าน cron หรือ launchd ถ้า​รัน​ซ้ำ​ถูก​แบบนี้
ก็​ปล่อย​ให้​รัน​บ่อย​แค่​ไหน​ก็ได้ โดย​ไม่ต้อง​ห่วง​ว่า​จะ​ไป​แย่ง​ซีพียู​กับ​งาน​ของ​เรา
และ​ผลลัพธ์​ไม่​เปลี่ยน​ไม่​ว่า​จะ​รัน​กี่​รอบ​ก็ตาม​ถ้า​ต้นทาง​ไม่​เปลี่ยน

== 2.2 transcript โต​ขึ้น​หนึ่ง​ไฟล์
<transcript-โตขนหนงไฟล>
รัน​ซ้ำ​ตอน​ไม่​มี​อะไร​เปลี่ยนเป็น​กรณี​ง่าย กรณี​จริง​คือ​เอ​เจนต์​ยัง​เขียน​อยู่ transcript
ที่​อ่าน​ไป​แล้​วจะ​ยาว​ขึ้น​เรื่อย ๆ ตอนนั้น relic-scan ต้อง​อ่าน​เฉพาะ​ส่วน​ท้าย​ที่​เพิ่ม​มา
ไม่​อ่าน​ทั้ง​ไฟล์​ซ้ำ ใน​เดโม​เรา​ต่อท้าย​ไฟล์ transcript ของ Claude Code
หนึ่ง​ไฟล์​ด้วย​สอง​บรรทัด แล้ว​รัน​อี​กรอบ

=== ขั้น​ที่ 1 — รัน​หลัง​ไฟล์​โต
<ขนท-1-รนหลงไฟลโต>
คำสั่ง​ยัง​เป็นตัว​เดิม​อีกแล้ว คน​รัน​ไม่ต้อง​บอ​กว่า​ไฟล์​ไหน​โต โปรแกรม​หา​เอง

```bash
relic-scan run
```

#figure(image("/images/08-run-grown.png", width: 100%, alt: "ผล run หลัง transcript โต​หนึ่ง​ไฟล์: grown 1, unchanged 143, parse 1 files, 2 raw rows"),
  caption: [
    ผล run หลัง transcript โต​หนึ่ง​ไฟล์: grown 1, unchanged 143, parse 1
    files, 2 raw rows
  ]
)

บรรทัด scan บอก​สถานะ​ใหม่​ทันที

```
run: scan 0.2s · new 0 · grown 1 · rewritten 0 · unchanged 143 · gone 0 …
```

#strong[grown 1] คือ​ไฟล์​ที่​เคย​อ่าน​แล้ว​และ​ยาว​ขึ้น ที่​เหลือ​อีก 143 ไฟล์​ยัง unchanged
ขอให้​สังเกตว่า​ไม่​ใช่ new และ​ไม่​ใช่ rewritten ด้วย โปรแกรม​แยก​สาม​กรณี​นี้​ออก​จากกัน
เพราะ​วิธี​จัดการ​ต่างกัน ไฟล์​ใหม่​อ่าน​ทั้ง​ไฟล์ ไฟล์​ที่​โต​อ่าน​ต่อ​จาก​เดิม
ส่วน​ไฟล์​ที่​ถูก​เขียน​ทับ​ต้อง​อ่าน​ใหม่

ถ้า​อยาก​ลอง​เอง ให้​เปิด transcript ของ​เดโม​สัก​ไฟล์ ต่อท้าย​ด้วย​บรรทัด JSON
ที่​ถูก​รูปแบบ​สอง​บรรทัด แล้ว​สั่ง run ดู ผล​จะ​ออกมา​แบบ​เดียว​กับ​ภาพ​ข้างบน
แต่​ตัวเลข​แถว​อาจ​ต่าง​ตาม​สิ่ง​ที่​คุณ​ต่อท้าย​ลง​ไป

=== ขั้น​ที่ 2 — อ่าน​บรรทัด parse
<ขนท-2-อานบรรทด-parse>
ทีนี้​บรรทัด `parse:` กลับมา แต่​ตัวเลข​เล็ก​มาก​เทียบ​กับ​รอบ​แรก

```
parse: 1 files · 0.00 GB · 2 lines (0 not JSON) · 2 raw rows · …
  2026-10: 747 rows (2 new copies merged) -> ~/.relic-scan/2026-10.parquet
merge: 1 months in 0.0s · total 0.0s
```

รอบ​แรก parse 144 ไฟล์ 3244 บรรทัด รอบ​นี้ parse #strong[1 ไฟล์ 2 บรรทัด] ได้ 2
แถว​ดิบ ซึ่ง​ตรง​กับ​ที่​เรา​ต่อท้าย​ไป และ​มี​บรรทัด​เดือน​บรรทัด​เดียว​คือ 2026-10
เพราะ​สอง​บรรทัด​นั้น​ลงวันที่​ใน​เดือน​ตุลาคม จำนวน​แถว​ของ​เดือน​นั้น​ขยับ​จาก 745 เป็น 747
ส่วน​สาม​เดือน​ก่อนหน้า​ไม่​ถูก​แตะ​เลย

=== ขั้น​ที่ 3 — ตรวจ​ยอดรวม​และ​ส่วน​ที่​สรุป​ใหม่
<ขนท-3-ตรวจยอดรวมและสวนทสรปใหม>
ลงมา​ที่​บรรทัด meta และ​กล่อง​สรุป ตัวเลข​สอง​จุด​นี้​ยืนยัน​ว่า​อ่าน​แค่​ของ​ใหม่​จริง

```
meta: 4 months (1 re-aggregated) in 0.0s · 144 files · 144 sessions …
│ 144 sessions · 3,173 rows · 144 files · 0.0 GB read              │
│ new 0 · grown 1 · rewritten 0 · unchanged 143                    │
```

สรุป​ใหม่ #strong[1 เดือน​จาก​สี่] คือ​เดือน​ที่​ได้​แถว​ใหม่ ส่วน​ยอดรวม​ขยับ​จาก 3,171 เป็น
3,173 ตรง​กับ​สอง​แถว​ที่​เพิ่ม และ​จำนวน session ยัง​เป็น 144 เพราะ​บรรทัด​ที่​ต่อไป​เป็น​ของ
session เดิม ตัวเลข 3,173 นี้​คือ​ยอด​ที่​หน้า​เว็บ​ทุก​หน้า​ใน​บท​ถัดไป​แสดง

บรรทัด ledger ด้านบน​ยัง​เป็น​ศูนย์​ทั้งหมด​ใน​รอบ​นี้ ตรง​กับ​ที่​ปรากฏ​ใน​ผลลัพธ์ ledger
เป็น​สมุด​ของ "เวอร์ชัน​ที่​เคย​อ่าน​มา​แล้ว" ซึ่ง​จะ​ทำงาน​เมื่อ​มี​ไฟล์​ใหม่​ที่​เนื้อหา​ตรง​กับ​ของเก่า เช่น
โฟลเดอร์​ที่​ย้ายที่​หรือ​สำเนา​จาก​อีก​เครื่อง ไฟล์​ที่​โต​ขึ้น​ใน​ที่​เดิม​ใช้​ตำแหน่ง​ที่​จำไว้​ใน
`_state.tsv` อยู่แล้ว

ตัว​สมุด​คือ `_ledger.tsv` เปิด​ดู​หัว​ไฟล์​ได้

```bash
head -4 ~/.relic-scan/_ledger.tsv | cut -f2,3,5,9
```

#figure(image("/images/44-ledger-tsv.png", width: 63%, alt: "ผล head ของ _ledger.tsv คอลัมน์ lines bytes session how สาม​แถว ทุก​แถว how เป็น read"),
  caption: [
    ผล head ของ \_ledger.tsv คอลัมน์ lines bytes session how สาม​แถว ทุก​แถว
    how เป็น read
  ]
)

หนึ่ง​แถว​คือ​หนึ่ง​เวอร์ชัน​ของ​หนึ่ง transcript ใน​ภาพ​สาม​แถว​แรก​มี 17, 28 และ 19 บรรทัด
ขนาด 7856, 12970 และ 8751 ไบต์ และ​คอลัมน์ `how` เป็น `read` ทั้งหมด
คือ​อ่าน​จริง​จาก​ไฟล์ ไม่​ใช่​สำเนา

== 2.3 relic-scan log และ​หน้า Scan log
<relic-scan-log-และหนา-scan-log>
สี่​รอบ​ที่ผ่านมา (รอบ dry run, รอบ​แรก, รอบ​ซ้ำ, รอบ​ไฟล์​โต)
ทุกครั้ง​โปรแกรม​ต่อ​บรรทัด​หนึ่ง​ลง​ไฟล์ `_scans.jsonl` ไม่​เคย​เขียน​ทับ คำสั่ง `log`
อ่าน​ไฟล์​นั้น​ออกมา​เป็นตาราง ดู​ย้อนหลัง​ได้​ว่า​แต่ละ​รอบ​เกิด​อะไร​ขึ้น
โดย​ไม่ต้อง​เลื่อน​หา​ใน​เทอร์มินัล

=== ขั้น​ที่ 1 — ดู​สี่​รอบ​ล่าสุด
<ขนท-1-ดสรอบลาสด>
ใส่ `-n 4` เพื่อ​จำกัด​ว่า​เอา​กี่​รายการ​ล่าสุด ใน​เดโม​มี​พอดี​สี่​รายการ

```bash
relic-scan log -n 4
```

#figure(image("/images/09-log.png", width: 97%, alt: "ตาราง relic-scan log -n 4 สี่​แถว: check 144 ใหม่, index 144 ใหม่, index 144 ไม่​เปลี่ยน, index grown 1"),
  caption: [
    ตาราง relic-scan log -n 4 สี่​แถว: check 144 ใหม่, index 144 ใหม่, index
    144 ไม่​เปลี่ยน, index grown 1
  ]
)

ตาราง​เรียง​ใหม่​ไป​เก่า เลข​แถว `#` เริ่ม​ที่ 0 สำหรับ​รายการ​แรก​สุด
อ่าน​จาก​ล่าง​ขึ้น​บน​จะ​ได้​เรื่องราว​ตามลำดับ​เวลา​ที่​เรา​ทำ

```
    #  when              job        new   grown rewritten   gone  unch …
    3  2026-10-26 07:22  index        0       1         0      0       …
    2  2026-10-26 07:22  index        0       0         0      0       …
    1  2026-10-26 07:22  index      144       0         0      0       …
```

ตัด​คอลัมน์​ท้าย​ออก​เพื่อให้​พอดี​บรรทัด ต้นฉบับ​กว้าง​กว่า​นี้ (ภาพ​มี​ครบ) รายการ 0 คือ dry run
ซึ่ง​ขึ้น​เป็น job `check` ส่วน​อีก​สาม​เป็น job `index` ให้​ดู​คอลัมน์ #strong[read]
รอบ​แรก​อ่าน 1.4 MB รอบ​ที่สอง​อ่าน 0 KB รอบ​ที่สาม​อ่าน​แค่ 1 KB และ​มัน​บอก​ด้วยว่า​บน​เดโม​นี้
transcript ทั้งหมด​รวมกัน 1.4 MB ซึ่ง​เป็นสาเหตุ​ที่ GB ใน​กล่อง​ออกมา​เป็น 0.00

=== ขั้น​ที่ 2 — ดู​ไฟล์​ใน​รายการ​หนึ่ง
<ขนท-2-ดไฟลในรายการหนง>
บรรทัด​ท้ายสุด​ของ​ผลลัพธ์​บอก #strong[วิธี​เจาะลึก​ต่อ​ด้วย `--diff`]

```
relic-scan log --diff N lists the files of entry N
```

ใส่​เลข​แถว​แทน N เช่น​ของ​รอบ​ที่​ไฟล์​โต​ก็​จะ​ได้​รายชื่อ​ไฟล์​ที่​มี​สถานะ grown พร้อม​ขนาดที่​เพิ่ม
ใช้​ตอน​อยากรู้​ว่า​ไฟล์​ไหน​ทำให้​เลข​เปลี่ยน ใน​เล่ม​นี้​ไม่​ได้​รัน `--diff` จึง​ไม่​มี​ผลลัพธ์​ให้​ดู

=== ขั้น​ที่ 3 — เปิด​หน้า Scan log บน​เว็บ
<ขนท-3-เปดหนา-scan-log-บนเวบ>
ข้อมูล​ชุด​เดียวกัน​มี​ให้​ดู​ใน​เว็บ​ด้วย เปิด server ด้วย `relic-scan serve` หรือ
`relic-scan up` ซึ่ง​ใช้​พอร์ต 8787 ถ้า​ไม่​ระบุ (ภาพ​ใน​เล่ม​ใช้ `--port 8791`)
แล้ว​เปิด​หน้า `/scans` ใน​เบราว์เซอร์ คือ `http://127.0.0.1:8787/scans`

#figure(image("/images/W11-page-scans.png", width: 100%, alt: "หน้า Scan log ที่ /scans: ตาราง​สี่​รายการ คอลัมน์ WHEN JOB NEW GROWN REWRITTEN GONE READ TIME RESULT"),
  caption: [
    หน้า Scan log ที่ /scans: ตาราง​สี่​รายการ คอลัมน์ WHEN JOB NEW GROWN
    REWRITTEN GONE READ TIME RESULT
  ]
)

หน้า​นี้​เป็นตาราง​เดียว​กับ​คำสั่ง log แต่​คอลัมน์​ท้ายสุด​คือ #strong[RESULT]
ซึ่ง​สรุป​ผล​เป็น​ประโยค​สั้น ๆ เช่น `+2 rows` สำหรับ​รอบ​ที่​ไฟล์​โต `+0 rows` สำหรับ​รอบ​ซ้ำ
`3,171 rows` สำหรับ​รอบ​แรก และ `would read…` สำหรับ​รอบ dry run ที่​ไม่​ได้​อ่าน​จริง
ถ้า​อยาก​เช็ก​ว่า​ช่วงนี้ nightly job ทำงาน​หรือเปล่า ดู​หน้า​นี้​เร็ว​กว่า​เปิด​ไฟล์ jsonl เอง

== 2.4 รัน​แค่​บางส่วน: \-\-kind และ \-\-month
<รนแคบางสวน---kind-และ---month>
ปกติ `run` เก็บ​ทุก​เอ​เจนต์​ทุก​เดือน​ไว้​ใน​ดัชนี​เดียว แต่​บางครั้ง​อยากได้​ชิ้น​เล็ก​กว่า เช่น​ดู​เฉพาะ
Codex หรือ​เฉพาะ​เดือน​นี้ มี​สอง​ตัวเลือก​ให้ ที่​เด่น​คือ​ทั้งสอง​แบบ
#strong[เขียน​ลง​โฟลเดอร์​แยก] ไม่​ปน​กับ​ดัชนี​หลัก และ​โฟลเดอร์​แยก​แต่ละ​อัน​มี​ไฟล์​ตั้งค่า
`_config.json` ของ​ตัวเอง (ใน​เดโม​คัดลอก​จาก​ดัชนี​หลัก เพื่อให้​สแกน​แค่ `~`)

=== ขั้น​ที่ 1 — รัน​เฉพาะ Codex
<ขนท-1-รนเฉพาะ-codex>
ใส่ `--kind codex` ต่อท้าย `run` โปรแกรม​จะ​กรอง​เอา​เฉพาะ transcript ของ Codex

```bash
relic-scan run --kind codex
```

#figure(image("/images/24-kind-one.png", width: 99%, alt: "ผล run --kind codex: new 23 ไฟล์, สี่​เดือน 2026-07 ถึง 2026-10, ปลายทาง ~/.relic-scan/codex"),
  caption: [
    ผล run \-\-kind codex: new 23 ไฟล์, สี่​เดือน 2026-07 ถึง 2026-10, ปลายทาง
    \~/.relic-scan/codex
  ]
)

บรรทัด​แรก​บอ​กว่า​พบ 23 ไฟล์​ใหม่ ซึ่ง​เป็น​ของ Codex ล้วน ๆ จาก​ทั้งหมด 144 ไฟล์​ใน​บ้าน​เดโม
พอ parse เสร็จ​ผลลัพธ์​กระจาย​ลง​สี่​เดือน ทุก​บรรทัด​เขียน​ลง​โฟลเดอร์ `codex/` ใต้​ดัชนี​หลัก

```
  2026-07: 46 rows (46 new copies merged) -> ~/.relic-scan/codex/2026-0…
│ 23 sessions · 152 rows · 23 files · 0.0 GB read                  │
│ → ~/.relic-scan/codex                                            │
│ ▸ relic-scan serve --kind codex   ▸ relic-scan tui --kind codex  │
```

มี​สอง​อย่าง​น่าดู อย่าง​แรก #strong[ปลายทาง​คือ `~/.relic-scan/codex`] ไม่​แตะ
`2026-10.parquet` ของ​ดัชนี​หลัก อย่าง​ที่สอง​คือ​บรรทัด​ล่าง​สุด​ที่​บอก​วิธี​เปิด​ดู​ต่อ ต้อง​ใส่
`--kind codex` ให้ `serve` และ `tui` ด้วย ไม่เช่นนั้น​มัน​จะ​ไป​เปิด​ดัชนี​หลัก

=== ขั้น​ที่ 2 — รัน​เฉพาะ​เดือน​ตุลาคม
<ขนท-2-รนเฉพาะเดอนตลาคม>
อีก​แกน​หนึ่ง​คือ​เวลา ใส่ `--month 2026-10` เพื่อ​เอา​แถว​ที่​ลงวันที่​ใน​เดือน​นั้น
(นับ​ตาม​วัน​ของ​กรุงเทพ) เท่านั้น

```bash
relic-scan run --month 2026-10
```

#figure(image("/images/25-month-one.png", width: 100%, alt: "ผล run --month 2026-10: อ่าน 144 ไฟล์ ทิ้ง 2426 แถว​นอก​ช่วง ปลายทาง all@2026-10"),
  caption: [
    ผล run \-\-month 2026-10: อ่าน 144 ไฟล์ ทิ้ง 2426 แถว​นอก​ช่วง ปลายทาง
    all\@2026-10
  ]
)

บรรทัด​แรก​ของ​ผลลัพธ์​บอก​ช่วง​และ​ปลายทาง​ตั้ง​แต่ต้น

```
run: only rows from 2026-10-01 to 2026-10-31 (Bangkok days) -> …
```

ชื่อ​โฟลเดอร์​ปลายทาง​คือ #strong[`all@2026-10`] แปล​ว่า​ทุก​เอ​เจนต์ (all) เฉพาะ​เดือน
2026-10 ถ้า​ใช้ `--kind` กับ `--month` ร่วมกัน ก็​คาด​ได้​ว่า​ชื่อ​สะท้อน​ทั้งสอง​อย่าง
แต่​เล่ม​นี้​ไม่​ได้​รัน​คู่​กัน จึง​ไม่​ยืนยัน

=== ขั้น​ที่ 3 — ดู​ว่า​อะไร​ถูก​ทิ้ง
<ขนท-3-ดวาอะไรถกทง>
ตัวเลือก `--month` เปิด​อ่าน​ทุก​ไฟล์ แต่​ทิ้ง​แถว​ที่​ตก​นอก​ช่วง

```
parse: 2426 rows outside 2026-10-01 to 2026-10-31 left out
  2026-10: 747 rows (747 new copies merged) -> …/all@2026-10/2026-10.parquet
│ 144 sessions · 747 rows · 144 files · 0.0 GB read                     │
```

เลข 747 แถว ตรง​กับ​แถว​ของ​เดือน 2026-10 ใน​ดัชนี​หลัก​หลัง​ไฟล์​โต (745 บวก 2) และ
2426 ที่​ถูก​ทิ้ง​ก็​คือ 3,173 ลบ 747 พอดี สอง​ตัวเลข​นี้​ยืนยัน​ว่า​ตัวเลือก `--month` ตัด​ถูกที่
ส่วน​จำนวน session ยัง​เป็น 144 เพราะ​ทั้ง 144 ไฟล์​มี​แถว​ใน​เดือน​นี้​อย่าง​น้อย​หนึ่ง​แถว

ผล​ที่​ได้​จะ​เปิด​ดู​ด้วย `relic-scan serve --out ~/.relic-scan/all@2026-10`
ตามที่​กล่อง​บอก​ท้าย​ผลลัพธ์ ใช้​ได้ดี​เมื่อ​อยาก​ส่ง​หรือ​ดู​เฉพาะ​เดือน​หนึ่ง​โดย​ไม่​ปน​กับ​ทั้ง​ก้อน

== ต่อไป
<ตอไป-1>
ถึง​ตรงนี้​ดัชนี​เดโม​มี​ครบ 3,173 แถว และ​รู้​แล้ว​ว่า​รัน​ซ้ำ​ถูก​แค่​ไหน ยัง​ไม่​ได้​ดู​ผล​ด้วย​ตา​เลย บท​ที่ 3
เปิด​หน้า​เว็บ​บน 127.0.0.1 ให้​เห็น​แถว​ทั้งหมด​เป็น​รูป​โดนัท เป็น​แผนผัง​ต้นไม้ และ​เป็น​แผนที่​ดาว

= บท​ที่ 3: เปิด​ดู​ใน​เว็บ
<บทท-3-เปดดในเวบ>
สอง​บท​แรก​อยู่​ใน​เทอร์มินัล​ล้วน ๆ บท​นี้​ย้าย​มา​ดู​เป็น​หน้า​เว็บ relic-scan มีหน้า​เว็บ​ใน​ตัว
เสิร์ฟ​บน 127.0.0.1 เท่านั้น เปิด​แล้ว​เห็น​ทั้ง​ดัชนี​เป็น​ภาพ ตั้งแต่​ยอดรวม ต้นไม้​ของ session
ไป​จนถึง​แผนที่​ดาว ทุก​ภาพ​ใน​บท​นี้​คือ​หน้า​จริง​ของ​บ้าน​เดโม​ที่​เรา​สร้าง​ไว้​ตั้งแต่​บท​แรก
ไม่​มี​ข้อมูลจริง​ของ​ใคร​ปน​อยู่

== เปิด​เซิร์ฟเวอร์
<เปดเซรฟเวอร>
=== ขั้น​ที่ 1 — เปิด​หน้า​เว็บ​ด้วย `up` หรือ `serve`
<ขนท-1-เปดหนาเวบดวย-up-หรอ-serve>
มี​สอง​คำสั่ง​ให้​เลือก `relic-scan up` คือ `run` ก่อน​แล้ว​ค่อย `serve` ต่อ ส่วน
`relic-scan serve` เสิร์ฟ​ดัชนี​ที่​มี​อยู่เลย​โดย​ไม่​สแกน​ใหม่ พอ index ไว้​แล้ว​ตั้งแต่​บท​ที่ 1 ใช้
`serve` ก็​พอ ค่า​เริ่มต้น​คือ​พอร์ต 8787 เปลี่ยน​ได้​ด้วย `--port`

```
relic-scan up                  # index แล้วเสิร์ฟ http://127.0.0.1:8787
relic-scan serve               # เสิร์ฟดัชนีที่มีอยู่
relic-scan serve --port 8791   # เปลี่ยนพอร์ต
```

ภาพ​ทุก​ใบ​ใน​เล่ม​นี้​ถ่าย​จาก​พอร์ต 8791 ส่วน​คน​ที่​ลอง​ตาม​ด้วย​ค่า​เริ่มต้น ให้​ใช้ 8787 แทน
ที่อยู่​ของ​แต่ละ​หน้า​คือ `http://127.0.0.1:8787/<view>` เช่น `/scan` `/tree`
`/map` `/paths` เซิร์ฟเวอร์​ผูก​กับ 127.0.0.1 เท่านั้น และ​ตัว​หน้า​เว็บ
#strong[ไม่​เรียก​เน็ต​เลย] ทุกอย่าง​อ่าน​จาก​ดัชนี​ใน​เครื่อง

=== ขั้น​ที่ 2 — รัน `run` ที่อื่น​แล้ว reload
<ขนท-2-รน-run-ทอนแลว-reload>
หน้า​เว็บ​อ่าน​ดัชนี​ทุกครั้งที่​โหลด ถ้า​เปิด​เซิร์ฟเวอร์​ค้าง​ไว้​แล้วไป `run` ใน​อีก​เทอร์มินัล
พอ​กลับมา​กด reload ก็​เห็น​ของ​ใหม่​ทันที ไม่ต้อง​รี​สตาร์ต นอกจากนั้น​ใน​หน้า​ยังมี​ปุ่ม #strong[⟳
index] ที่​สั่ง run แบบ​เพิ่ม​เฉพาะ​ส่วน​ใหม่​จาก​ใน​เซิร์ฟเวอร์​เอง​ได้ ถ้า​ยัง​ไม่​เคย index เลย
ทุก view จะ​ขึ้นหน้า​ต้อนรับ​พร้อม​ปุ่ม​เดียวกัน

== Smart Scan — ภาพ​รวมทั้งหมด
<smart-scan-ภาพรวมทงหมด>
=== ขั้น​ที่ 1 — เปิด​หน้า​แรก `/scan`
<ขนท-1-เปดหนาแรก-scan>
เปิด `http://127.0.0.1:8787/scan` ซึ่ง​เป็น​หน้า​แรก​ของ​แอ​ป (เปิด​แค่ `/` ก็ได้)
ฝั่ง​ซ้าย​คือ​เมนู เรียง​จาก​บน​ลง​ล่าง​เป็น Smart Scan, Banks & Repos, ψ Vaults,
Health, Scan log, Activity แล้ว​กลุ่ม Session map ที่​มี Tree, Paths, Disk
อยู่​ข้างใน ท้ายสุด​คือ Export กับ Settings กด​เมนู​แล้ว​สลับ view โดย​ไม่ต้อง​โหลด​หน้าใหม่

กลาง​หน้า​มี​โดนัท​ใหญ่ ตรงกลาง​โดนัท​เป็น​ยอดรวม​ของ​แถว​ทั้งหมด คือ #strong[3,173 แถว]
ใต้​โดนัท​มี​แผ่น​ตัวเลข​ห้า​อัน ROWS, SESSIONS, FILES INDEXED, NEWEST ROW และ SPAN

#figure(image("/images/W10-page-scan.png", width: 100%, alt: "หน้า Smart Scan: โดนัท​แถว​แยก​ตาม​ชนิด ยอดรวม 3,173 และ​แผ่น​ตัวเลข​ด้านล่าง"),
  caption: [
    หน้า Smart Scan: โดนัท​แถว​แยก​ตาม​ชนิด ยอดรวม 3,173 และ​แผ่น​ตัวเลข​ด้านล่าง
  ]
)

ตัวเลข​ชุด​นี้​คือ​ชุด​เดียว​กับ​ที่ DuckDB นับ​ได้​ใน​บท​ที่ 6 ถ้า​สอง​ที่​นับ​ไม่​ตรงกัน​แปล​ว่า​มี​อะไร​ผิด
เป็น​วิธี​เช็ก​ที่​เร็ว​ดี

=== ขั้น​ที่ 2 — อ่าน​โดนัท​และ​คำอธิบาย​สี
<ขนท-2-อานโดนทและคำอธบายส>
โดนัท​แบ่ง​แถว​ตาม​ว่า​แถว​นั้น​คือ​อะไร มี​หก​ส่วน ได้แก่ tool calls, user text และ AI text
ต่อ​ด้วย tool results, thinking และ tool errors
แต่ละ​ส่วน​มี​จำนวน​แถว​กับ​เปอร์เซ็นต์​กำกับ #strong[หนึ่ง​แถว​คือ​หนึ่ง​ข้อความ หนึ่ง thinking
block หนึ่ง​การ​เรียก​เครื่องมือ หรือ​หนึ่ง​ผลลัพธ์] ไม่​ใช่​หนึ่ง​บรรทัด​ของ​ไฟล์

ภาพ​นี้​ตัด​มา​เฉพาะ​โดนัท​กับ​คำอธิบาย ขยาย​ให้​อ่าน​ตัวเลข​ง่าย​ขึ้น

#figure(image("/images/C1-smart-scan-donut.png", width: 100%, alt: "โดนัท​ที่​ตัด​มา​เฉพาะ​วง​กับ​คำอธิบาย​สี​ของ​แต่ละ​ชนิด​แถว"),
  caption: [
    โดนัท​ที่​ตัด​มา​เฉพาะ​วง​กับ​คำอธิบาย​สี​ของ​แต่ละ​ชนิด​แถว
  ]
)

=== ขั้น​ที่ 3 — ปุ่ม Index now และ Check changes
<ขนท-3-ปม-index-now-และ-check-changes>
หน้า​นี้​มี​สอง​ปุ่ม​ที่​ควร​รู้จัก #strong[Check changes] คือ​การ​สแกน​แบบ dry run
เดิน​ดู​โฟลเดอร์ เทียบ​ขนาด​กับ `_state.tsv` แล้ว​บอ​กว่า​จะ​มี​ไฟล์​ใหม่ ไฟล์​โต ไฟล์​ถูก​เขียน​ใหม่
และ​ไฟล์​หาย​กี่​ไฟล์ โดย​ไม่​เขียน​อะไร​เลย เหมือน `run --dry-run` ใน​บท​ที่ 1

ส่วน Index now คือ `run` ที่​ทำ​อยู่​ใน​เซิร์ฟเวอร์ ระหว่าง​ทำ​มี​แถบ​ความคืบหน้า​กับ​ปุ่ม Stop
ถ้า​กด Stop ตอน​ยัง​เดิน​ดู​โฟลเดอร์​หรือยัง​อ่าน​อยู่ จะ​ไม่​มี​อะไร​ถูก​เขียน
รอบ​หน้า​ก็​อ่าน​ไบต์​ชุด​เดิม​ซ้ำ แต่​ถ้า​เข้าขั้น merge แล้ว​มัน​จะ​ทำ​จน​จบ

=== ขั้น​ที่ 4 — ค้น​ด้วย ⌘K
<ขนท-4-คนดวย-k>
กด ⌘K หรือ `/` ได้​จาก​ทุก​หน้า ช่อง​ค้นหา​เดียว​ครอบคลุม view, action, session (ค้น​ด้วย
id, repo หรือ project) พาธ​ของ `.jsonl` และ​รายการ​ใน scan log พิมพ์​ว่าง​ไว้​ก็ได้
มัน​โชว์ session ล่าสุด​ให้ เลือก​ผลลัพธ์​แล้ว​เปิด view ที่​เกี่ยวข้อง ช่อง​นี้​เร็ว​กว่า​ไล่​คลิก​เมนู​มาก

== Tree — บ้าน, repo, session
<tree-บาน-repo-session>
=== ขั้น​ที่ 1 — เปิด `/tree`
<ขนท-1-เปด-tree>
#figure(image("/images/W12-page-tree.png", width: 100%, alt: "หน้า Tree: บ้าน .claude ที่​มี 6 repo, ตาม​ด้วย .codex และ .omp, แผง​ขวา​สรุป​ยอด"),
  caption: [
    หน้า Tree: บ้าน \.claude ที่​มี 6 repo, ตาม​ด้วย \.codex และ \.omp,
    แผง​ขวา​สรุป​ยอด
  ]
)

เปิด `http://127.0.0.1:8787/tree` หน้า​นี้​เรียง​เป็น​ชั้น บ้าน (home) ของ​เอ​เจนต์ แล้ว
repo แล้ว session บ้าน​แรก​คือ `~/.claude` มี 6 repo ได้แก่ tea-cart,
starfish-api, harbor-bot, lantern-app, kite-docs และ moss-cli
ถัด​ลง​ไป​เป็น​บ้าน `.codex` กับ `.omp` #strong[ทุก​ชั้น​มี​จำนวน session
กับ​จำนวน​แถว​กำกับ] และ​มี​แท่ง​แยก​ชนิด​แถว

ฝั่ง​ขวา​คือ​แผง​สรุป มี​หัวข้อ ALL SESSIONS (ยอดรวม) ROWS BY TYPE (แท่ง​แยก​ตาม​ชนิด​แถว)
REPOS (รายชื่อ repo) และ JSONL HOMES

session หนึ่ง​อาจ​อยู่​ใน​หลาย​บ้าน ถ้า​มี​สำเนา​ใน​บ้าน​อื่น มัน​จะ​ปรากฏ​ใต้​ทุก​บ้าน​ที่เก็บ​สำเนา
แต่​ใน​ดัชนี​นับ​เป็นแถว​เดียว ตามที่ AGENTS.md บอ​กว่า​สำเนา​ถูก​พับ​รวม​เป็นหนึ่ง​แถว

=== ขั้น​ที่ 2 — อ่าน​แผง​ขวา ROWS BY TYPE
<ขนท-2-อานแผงขวา-rows-by-type>
แท่ง ROWS BY TYPE คือ​โดนัท​ของ Smart Scan ใน​รูป​แท่ง แต่​ต่างกัน​ตรง​ที่
#strong[แผง​นี้​เปลี่ยน​ตาม​สิ่ง​ที่​เลือก​อยู่] คลิก repo หนึ่ง​แล้ว​แผง​ก็​นับ​เฉพาะ repo นั้น Tree
กับ Map ใช้​กล่อง​ค้นหา​และ​แผง​ข้าง​ร่วมกัน คำ​ค้น​ที่​พิมพ์​ไว้​จึง​ตาม​ไป​อีก​หน้า​ได้

ภาพ​นี้​ตัด​แผง​ขวา​มา​ให้​ดู​ใกล้ ๆ

#figure(image("/images/C2-tree-rows-by-type.png", width: 56%, alt: "แผง​ขวา​ของ Tree ที่​ตัด​มา: แท่ง​แถว​แยก​ตาม​ชนิด​ของ session ที่​เลือก"),
  caption: [
    แผง​ขวา​ของ Tree ที่​ตัด​มา: แท่ง​แถว​แยก​ตาม​ชนิด​ของ session ที่​เลือก
  ]
)

=== ขั้น​ที่ 3 — กรอง​ด้วย​คำ​ค้น
<ขนท-3-กรองดวยคำคน>
รายการ​ใน REPOS และ JSONL HOMES ของ​แผง​กด​ได้ กด​แล้ว​คำ​ค้น​แคบ​ลง​ทันที ถ้า​อยาก​พิมพ์​เอง
คำ​ค้น​ของ Map และ Tree รองรับ​ตัว​กรอง​อย่าง `repo:` `tier:main|sub|wf`
`month:2026-07` และ `rows>1000` คำนำหน้า `-` กลับ​ความหมาย และ
#strong[ทุก​คำ​ต้อง​ตรง​พร้อมกัน] (AND)

== Map — หนึ่ง​ดาว หนึ่ง session
<map-หนงดาว-หนง-session>
=== ขั้น​ที่ 1 — เปิด `/map`
<ขนท-1-เปด-map>
เปิด `http://127.0.0.1:8787/map` เห็น​แผนที่​ดาว​กลม ๆ ทุก session คือ​จุด​หนึ่ง​จุด
วงแหวน​คือ​เดือน นับ​จาก 2026-07 ถึง 2026-10 โดย​เดือน​เก่า​สุด​อยู่​วงใน​สุด
ส่วน​ซี่​ที่​แผ่​ออกมา​คือ repo ใน​เดโม​มี moss-cli, codex, tea-cart, harbor-bot,
lantern-app, omp, starfish-api และ kite-docs ปลาย​ซี่​แต่ละ​อัน​มี​จำนวน session
กำกับ

#figure(image("/images/W13-page-map.png", width: 100%, alt: "หน้า Map: แผนที่​ดาว​กลม วงแหวน​เป็น​เดือน ซี่​เป็น repo ปุ่ม​เลือก​สี​จุด​อยู่​ด้านล่าง"),
  caption: [
    หน้า Map: แผนที่​ดาว​กลม วงแหวน​เป็น​เดือน ซี่​เป็น repo ปุ่ม​เลือก​สี​จุด​อยู่​ด้านล่าง
  ]
)

อ่าน​อย่างนี้ #strong[ตำแหน่ง​บอ​กว่า session เริ่ม​เมื่อไร​และ​อยู่​ใน repo ไหน]
ขนาด​ของ​ดาว​ขึ้นกับ​จำนวน​แถว (สเกล log) ซี่​ที่​กว้าง​กว่า​มี session มากกว่า ส่วน subagent
กับ workflow agent จะ​วน​อยู่​รอบ session แม่​ของ​มัน
ดาว​ที่อยู่​ไกล​จาก​ศูนย์กลาง​จึง​ใหม่​กว่า​ดาว​ที่อยู่​วงใน

=== ขั้น​ที่ 2 — เปลี่ยนสี​จุด
<ขนท-2-เปลยนสจด>
แถบ​ปุ่ม​ด้านล่าง #strong[เลือก​สี​ของ​จุด] ได้​สี่​แบบ คือ tier, repo, age และ tools%
แต่ละ​แบบ​ตอบคำถาม​ต่างกัน tier แยก session หลัก​ออกจาก subagent, repo ให้สี​ตาม​ซี่,
age ไล่ตาม​อายุ และ tools% ชี้​ว่า session ไหน​ใช้​เครื่อง​มือหนัก​กว่า​คุย​ธรรมดา

=== ขั้น​ที่ 3 — คลิก​ดาว​ดู​รายละเอียด
<ขนท-3-คลกดาวดรายละเอยด>
คลิก​ดาว​หนึ่ง​ดวง​แล้ว​แผง​ข้าง​จะ​โชว์ session นั้น #strong[ครบ​ทั้งตัว] มี sid, repo,
เวลา, แถว​แยก​ตาม​ชนิด, จำนวน​สำเนา และ​ปุ่ม​คัดลอก sid หรือ​บรรทัด `claude --resume`
ไป​วาง​ต่อ​ได้​เลย นี่​คือ​ทางลัด​จาก​การ​เห็นภาพ ไป​สู่​การ​กลับ​เข้า session จริง

== Paths — ทุก \.jsonl ที่​สแกน​เห็น
<paths-ทก-.jsonl-ทสแกนเหน>
=== ขั้น​ที่ 1 — เปิด `/paths`
<ขนท-1-เปด-paths>
เปิด `http://127.0.0.1:8787/paths` หน้า​นี้​ตอบคำถาม​ที่ Tree ตอบ​ไม่​ได้​ว่า
ไฟล์​ไหน​ที่​สแกน​เห็น​แต่ #strong[ไม่​ได้​ถูก​อ่าน] เพราะอะไร เป็น​ต้นไม้​ของ​โฟลเดอร์
ทุก​โฟลเดอร์​มี​จำนวน​ไฟล์ จำนวน​แถว ขนาด และ​แถบ​สถานะ
โฟลเดอร์​ที่​มี​ทางเดียว​จะ​ถูก​พับ​รวม​เป็นแถว​เดียว และ `~` แทน​โฟลเดอร์​บ้าน

ใน​เดโม โฟลเดอร์​ที่​พับ​อยู่​มี 145 ไฟล์ 3,173 แถว แผง​ขวา​มี​หัวข้อ STATUS, VERDICT และ
BUSIEST FOLDERS

#figure(image("/images/W14-page-paths.png", width: 100%, alt: "หน้า Paths: ต้นไม้​โฟลเดอร์ 145 ไฟล์ และ​แผง​ขวา STATUS, VERDICT, BUSIEST FOLDERS"),
  caption: [
    หน้า Paths: ต้นไม้​โฟลเดอร์ 145 ไฟล์ และ​แผง​ขวา STATUS, VERDICT, BUSIEST
    FOLDERS
  ]
)

=== ขั้น​ที่ 2 — อ่าน STATUS กับ VERDICT
<ขนท-2-อาน-status-กบ-verdict>
สอง​กล่อง​นี้​ดู​คล้าย​กัน​แต่ #strong[วัด​คนละอย่าง] STATUS คือ​สิ่ง​ที่ relic-scan ทำ​กับ​ไฟล์
ใน​เดโม​ได้ indexed 144, skipped 0, vault 0 และ loose 1 โดย indexed
คือ​ไฟล์​ที่​ถูก​อ่าน​เข้าเดือน, skipped คือ​ไฟล์​ใน​บ้าน​เอ​เจนต์​ที่​ไม่​ใช่ transcript, vault
คือ​ไฟล์​ใน ψ vault ซึ่ง​แค่​แสดง​รายการ, และ loose คือ​ไฟล์​ที่อยู่​ที่อื่น​ใต้ root
ก็​แค่​แสดง​รายการ​เช่นกัน

VERDICT คือ​ผล​การดม​เนื้อหา (content sniff) ว่า​หน้าตา​เป็น transcript หรือไม่ ได้
transcript-shaped 144 กับ not-transcript 1 รวมกัน​เป็น 145
ตรง​กับ​จำนวน​ไฟล์​ที่​เห็น​ข้างบน

=== ขั้น​ที่ 3 — ใช้​ตัว​กรอง​หา​ไฟล์​ที่​ถูก​ข้าม
<ขนท-3-ใชตวกรองหาไฟลทถกขาม>
ตัว​กรอง​ของ​หน้า​นี้​ใช้ได้​เช่น `is:indexed|skipped|vault|loose` และ
`verdict:not-transcript` ยังมี `name:agent-` `size>1e9` `rows=0` และ
`after:` กับ `before:` ซึ่ง​นับ​ตามเวลา​แก้ไข​ไฟล์ ลอง​พิมพ์ `verdict:not-transcript`
ดู จะ​เห็น​ไฟล์​ที่​ไม่​ใช่ transcript โผล่​ขึ้น​มา #strong[คลิก​ไฟล์​เพื่อ​ดู​สถานะ ขนาด
จำนวน​แถว] และ​ปุ่ม​คัดลอก​พาธ ถ้า​ไฟล์​ที่​คิด​ว่า​ควร​ถูก​อ่าน​หาย​ไป หน้า​นี้​คือ​ที่​แรก​ที่​ควร​มา​ดู

คำ​ค้น​ของ Paths ไม่​ได้​สร้าง​อะไร​ใหม่ มัน​แค่​กรอง​รายการ​ที่​หน้า​โหลด​มา​แล้ว
เปลี่ยน​คำ​ค้น​กี่​รอบ​ก็​ไม่​กระทบ​ดัชนี ทุกอย่าง​บน​หน้า​เว็บ​เป็นการ​อ่าน ปุ่ม​เดียว​ที่​เขียน​จริง​คือ Index
now ซึ่ง​เขียน​ลง​โฟลเดอร์​ดัชนี​ของ relic-scan เอง ไม่​แตะ transcript

หน้า​เว็บ​ที่​เหลือ​คือ​การ์ด​ของ​แต่ละ​บ้าน แผนภูมิ​ดิสก์ ห้อง​นิรภัย​ของ oracle หน้า Health และ
Settings บท​หน้า​ไล่​ต่อให้​ครบ

= บท​ที่ 4: ดิสก์ ห้อง​นิรภัย และ​การ​ตั้งค่า
<บทท-4-ดสก-หองนรภย-และการตงคา>
บท​ที่แล้ว​เรา​ดู​ดัชนี​จาก​มุม​ของ​แถว​และ session บท​นี้​ดู​จาก​มุม​ของ #strong[ที่​ที่​ข้อมูล​อยู่]
บ้าน​ของ​เอ​เจนต์​แต่ละ​ที่ ไฟล์​ที่​กิน​ดิสก์ ห้อง​นิรภัย ψ ของ oracle แล้ว​ปิดท้าย​ด้วย​หน้า Health
ที่​บอ​กว่า​ทั้งหมด​ปกติ​ดี​ไหม และ Settings ที่​ปรับ​สิ่ง​ที่​สแกน​เดิน​ดู เซิร์ฟเวอร์​ยัง​เป็นตัว​เดิม
ที่อยู่​ยัง​เป็น `http://127.0.0.1:8787/<view>` ส่วน​ภาพ​ใน​บท​นี้​ถ่าย​จาก​พอร์ต 8791
เหมือน​บท​ที่แล้ว

== Banks & Repos — การ์ด​ต่อ​หนึ่ง​บ้าน
<banks-repos-การดตอหนงบาน>
=== ขั้น​ที่ 1 — เปิด `/banks`
<ขนท-1-เปด-banks>
เปิด `http://127.0.0.1:8787/banks` หรือ​กด Banks & Repos ใน​เมนู​ซ้าย
ด้านบน​เป็น​การ์ด​หนึ่ง​ใบ​ต่อ​หนึ่ง #strong[บ้าน​ของ​เอ​เจนต์] (agent home) ใน​เดโม​มี​สาม​ใบ
คือ `.claude`, `.codex` และ `.omp` แต่ละ​การ์ด​บอก sessions, rows, files,
ขนาด และ session ล่าสุด

#figure(image("/images/W17-page-banks.png", width: 100%, alt: "หน้า Banks & Repos: การ์ด​ของ .claude, .codex, .omp และ​ตาราง Repository index 8 repo"),
  caption: [
    หน้า Banks & Repos: การ์ด​ของ \.claude, \.codex, \.omp และ​ตาราง
    Repository index 8 repo
  ]
)

ใต้​การ์ด​เป็น Repository index มี 8 repo แต่ละ​บรรทัด​บอก​จำนวน session
กับ​จำนวน​แถว​ของ repo นั้น

=== ขั้น​ที่ 2 — กด​การ์ด​หรือ​บรรทัด​เพื่อ​ไป​ต่อ
<ขนท-2-กดการดหรอบรรทดเพอไปตอ>
หน้า​นี้​เป็น​ทาง​เข้าไป​ยัง​หน้า​อื่น #strong[กด​การ์ด​แล้ว​เปิด Tree ที่​บ้าน​นั้น] ส่วน​กด​บรรทัด​ใน
Repository index แล้ว​เปิด Map ที่ repo นั้น ส่วน​ตัวเลข​ใน​การ์ด​คือ​ยอด​ของ​บ้าน​นั้น​ทั้ง​บ้าน

README บอ​กว่า​การ์ด​มี​ช่อง origin ด้วย ซึ่ง​มีประโยชน์​ตอนที่​เครื่อง​มี mirror หรือ backup
ของ `.claude` หลาย​ที่ relic-scan พับ​แถว​ซ้ำ​ให้​เป็นแถว​เดียว
แต่​ยัง​แสดงให้เห็น​ว่า​บ้าน​ไหน​ถือ​ไฟล์​อยู่

ถ้า​อยากรู้​ว่า​เครื่อง​นี้​มี​เอ​เจนต์​ตัว​ไหน​ทิ้ง​ประวัติ​ไว้​มาก​ที่สุด ดู​ที่ rows ของ​แต่ละ​การ์ด​แล้ว​เทียบ​กัน
จบ​ใน​หน้า​เดียว ไม่ต้อง​ไป​เขียน query

== Disk — ไฟล์​ไหน​กินที่
<disk-ไฟลไหนกนท>
=== ขั้น​ที่ 1 — เปิด​ตัวเลือก Disk ก่อน
<ขนท-1-เปดตวเลอก-disk-กอน>
README เรียก Disk ว่า gem คือ #strong[ซ่อน​ไว้​จนกว่า​จะ​เปิด​เอง] ใน Settings \>
Gems และ​เปิด​เฉพาะ​เบราว์เซอร์​นั้น ถ้า​ไม่​เห็น​เมนู Disk ใต้ Session map ให้​ไป​ที่​หน้า
Settings (อยู่​ท้าย​บท​นี้) แล้ว​เปิด​ตรงนั้น​ก่อน ภาพ​ด้านล่าง​ถ่าย​จาก​การ​เปิด​ที่อยู่ `/disk` ตรง
ๆ

=== ขั้น​ที่ 2 — เปิด `/disk`
<ขนท-2-เปด-disk>
เปิด `http://127.0.0.1:8787/disk` ได้​แผนภูมิ​วงกลม​หลาย​ชั้น (sunburst) ของ
`.jsonl` ทุก​ไฟล์ จัด​ตาม​โฟลเดอร์​และ​ขนาด ตรงกลาง​วง​คือ​ยอดรวม #strong[1.5 MB]
วงใน​แทน​โฟลเดอร์​ใหญ่ วง​ที่​ออก​ไป​แตก​เป็น​โฟลเดอร์​ย่อย
มุม​ของ​แต่ละ​ชิ้น​คือ​สัดส่วน​ของ​โฟลเดอร์​แม่

#figure(image("/images/W15-page-disk.png", width: 100%, alt: "หน้า Disk: แผนภูมิ sunburst ของ .jsonl ทุก​ไฟล์ ตรงกลาง 1.5 MB พร้อม​คำอธิบาย​สี​ของ​แต่ละ​โฟลเดอร์"),
  caption: [
    หน้า Disk: แผนภูมิ sunburst ของ \.jsonl ทุก​ไฟล์ ตรงกลาง 1.5 MB
    พร้อม​คำอธิบาย​สี​ของ​แต่ละ​โฟลเดอร์
  ]
)

ใช้งาน​ง่าย​มาก คลิก​ชิ้น​ไหน​ก็​ซูม​เข้าไป​ใน​โฟลเดอร์​นั้น คลิก​ตรงกลาง​เพื่อ​ถอย​ออกมา
ชิ้น​เล็ก​มาก​ที่​แต่ละ​ชิ้น​ไม่​ถึง 0.5% ของ​ตรงกลาง จะ​ถู​กรวม​เป็น​ชิ้น​สีเทา "smaller objects"

=== ขั้น​ที่ 3 — อ่าน​คำอธิบาย​สี
<ขนท-3-อานคำอธบายส>
คำอธิบาย​ด้าน​ข้าง​เรียง​ตาม​สัดส่วน​ใน​เดโม

```
.claude/projects   92.8%
.codex/sessions     2.9%
.relic-scan         2.7%
.omp                1.6%
```

ส่วนใหญ่​อยู่​ใน `.claude/projects` ซึ่ง​สมเหตุสมผล เพราะ transcript ของ Claude
Code เป็น 144 ไฟล์​จาก​ทั้งหมด 145 ที่​เห็น ที่​น่าสนใจ​คือ `.relic-scan` ก็​ติด​มา​ใน​รายการ
เพราะ​เป็น​โฟลเดอร์​ที่​มี `.jsonl` อยู่​ใน​นั้น ดังนั้น​ตัวเก็บ​ดัชนี​เอง​ก็​กินที่​ให้​เห็น​ใน​ภาพ​เหมือนกัน

เครื่อง​จริง​กว่า​นี้​มาก AGENTS.md ยกตัวอย่าง​เครื่อง Mac เครื่อง​หนึ่ง​ที่​เจอ 87,000 ไฟล์ 75
GB ใน​เครื่อง​ขนาด​นั้น การ​เห็น​เป็น​ภาพ​ว่า​โฟลเดอร์​ไหน​กินที่​มาก​สุด​ช่วย​ได้​เยอะ
ก่อน​จะ​ลงมือ​จัดการ​อะไรก็ตาม

== ψ Vaults — ห้อง​นิรภัย​ของ oracle
<ψ-vaults-หองนรภยของ-oracle>
=== ขั้น​ที่ 1 — เปิด `/vaults`
<ขนท-1-เปด-vaults>
ψ vault คือ​โฟลเดอร์​ความจำ​ของ oracle ที่อยู่​ใน repo เช่น `~/Code/kite-docs/ψ`
relic-scan หา​เจอ​แล้ว​ทำ​รายการ​ให้ แต่ #strong[อ่าน​อย่าง​เดียว]
ไม่​แก้​และ​ไม่​ย้าย​อะไร​เลย เปิด `http://127.0.0.1:8787/vaults` ด้านบน​เป็น​แผ่น​ตัวเลข
VAULTS 2, NOTES 5, SIZE 401 B และ NEWEST NOTE 2026-10-26

ใต้​แผ่น​ตัวเลข​เป็น​รายชื่อ vault สอง​อัน คือ kite-docs มี 3 โน้ต และ moss-cli มี 2 โน้ต
แต่ละ​อัน​มี​แท่ง​สี​และ​พาธ `~/Code/kite-docs/ψ`

#figure(image("/images/W16-page-vaults.png", width: 100%, alt: "หน้า ψ Vaults: แผ่น​ตัวเลข VAULTS 2 NOTES 5 และ​รายการ kite-docs กับ moss-cli"),
  caption: [
    หน้า ψ Vaults: แผ่น​ตัวเลข VAULTS 2 NOTES 5 และ​รายการ kite-docs กับ
    moss-cli
  ]
)

=== ขั้น​ที่ 2 — ดู​ข้อมูล​ชุด​เดียวกัน​เป็น JSON
<ขนท-2-ดขอมลชดเดยวกนเปน-json>
หน้า​เว็บ​นี้​อ่าน​จาก​ไฟล์ `meta/vaults.json` ใน​โฟลเดอร์​ดัชนี เลย​ถาม​ด้วย `jq` ได้​เลย
คำสั่ง​นี้​ดึง​จำนวน​โน้ต ชื่อ และ​พาธ​ของ​แต่ละ vault ออกมา​เป็นตาราง

```bash
$ jq -r '.vaults[] | [.notes, .name, .path] | @tsv' \
    ~/.relic-scan/meta/vaults.json
```

#figure(image("/images/21-vaults.png", width: 86%, alt: "ผล​ของ jq: kite-docs 3 โน้ต และ moss-cli 2 โน้ต พร้อม​พาธ"),
  caption: [
    ผล​ของ jq: kite-docs 3 โน้ต และ moss-cli 2 โน้ต พร้อม​พาธ
  ]
)

ตัวเลข​ตรง​กับ​หน้า​เว็บ 3 บวก 2 ได้ 5 โน้ต​ตาม​แผ่น NOTES ถ้า​อยากได้​โน้ต​รายตัว ไฟล์
`meta/vaults/<id>.json` เก็บ​แต่ละ​โน้ต​เป็น path, section, title, ขนาด
และ​เวลา​แก้ไข

== Health — ปกติ​ดี​ไหม
<health-ปกตดไหม>
=== ขั้น​ที่ 1 — เปิด `/health`
<ขนท-1-เปด-health>
เปิด `http://127.0.0.1:8787/health` ใน​เดโม​หัวหน้า​ขึ้น​ว่า All clear
ใต้​นั้น​เป็น​แผ่น​ตัวเลข ROWS 3,173, SESSIONS 144, MONTHS 4 และ WARNINGS 0
#strong[WARNINGS 0 คือ​ตัว​ที่​ต้อง​ดูก่อน] ถ้า​มี​เลข​โผล่ แปล​ว่า​มี​ข้อ​ใด​ข้อ​หนึ่ง​ด้านล่าง​ไม่​ผ่าน

#figure(image("/images/W20-page-health.png", width: 100%, alt: "หน้า Health: All clear แผ่น​ตัวเลข 3,173 แถว 144 session 4 เดือน 0 คำเตือน และ​รายการ​เช็ก​ที่​ติ๊ก​เขียว"),
  caption: [
    หน้า Health: All clear แผ่น​ตัวเลข 3,173 แถว 144 session 4 เดือน 0 คำเตือน
    และ​รายการ​เช็ก​ที่​ติ๊ก​เขียว
  ]
)

=== ขั้น​ที่ 2 — อ่าน​รายการ Checks
<ขนท-2-อานรายการ-checks>
ใต้​แผ่น​เป็น​รายการ​เช็ก​ที่​คำนวณ​จาก​ดัชนี แต่ละ​ข้อ​ติ๊ก​เขียว​เมื่อ​ผ่าน ใน​เดโม
#strong[ผ่าน​หมด​ทุก​ข้อ] คือ อ่าน​ทุก​โฟลเดอร์​ได้, รอบ​ล่าสุด​เพิ่ง​รัน​เมื่อ​ครู่, ไม่​มี transcript
อยู่​นอกบ้าน​เอ​เจนต์, ทุก session มี​เวลา และ​ทุก​ไฟล์​ที่ index ให้​แถว​ออกมา

ข้อมูล​ที่ README อธิบาย​ไว้ เช็ก​อื่น​ที่​หน้า​นี้​ดู​ให้ได้​อีก มีอายุ​ของ​รอบ​ล่าสุด, subagent ที่​ไม่​มี
session หลัก, ไฟล์​ที่​ถูก​ข้าม​เพราะ​ไม่​ใช่ transcript, รูปทรง​ที่​ไม่​รู้จัก, mirror ที่​ถูก​พับ
และ tool errors แต่ละ​ข้อ​มี​ลิงก์​ไป​ยัง view ที่​เกี่ยวข้อง พอ​มี​ข้อ​ไม่​ผ่าน กด​ตาม​ไป​ก็​ถึง​ไฟล์​หรือ
session ที่​เป็น​ต้นเรื่อง

แผ่น MONTHS 4 #strong[ตรง​กับ]สี่​เดือน​ที่​เห็น​บน​วงแหวน​ของ Map คือ 2026-07 ถึง
2026-10 และ SESSIONS 144 ตรง​กับ​จำนวน transcript ที่ Paths นับ​เป็น indexed
สาม​หน้าที่​นับ​จาก​ดัชนี​เดียวกัน​จึง​ต้อง​ตรงกัน ถ้า​เลข​ไม่​ตรง ให้​สงสัย​ว่า​เปิด​คนละ​โฟลเดอร์​ดัชนี เช่น
ลืม `--out` หรือ `--kind`

ข้อ​ที่​ผ่าน​แต่​มี​ข้อมูล​ประกอบ​จะ​เป็น​บรรทัด info ไม่​ใช่​คำเตือน อ่าน​เป็น​ความรู้ ไม่ต้อง​แก้

== Settings — สิ่ง​ที่​สแกน​เดิน​ดู
<settings-สงทสแกนเดนด>
=== ขั้น​ที่ 1 — เปิด `/settings`
<ขนท-1-เปด-settings>
เปิด `http://127.0.0.1:8787/settings` หน้า​นี้​ตั้งค่า​ว่า #strong[การ​สแกน​เดิน​ดู​อะไร]
มี​สอง​ส่วน​ใน​ภาพ ส่วน​แรก What to scan คือ​โฟลเดอร์​ที่​ให้​เดิน (ใน​เดโม​คือ `~`)
พาธ​ที่​ให้​ข้าม และ​ความลึก (depth) ที่ตั้ง​ไว้ 8 ส่วน​ที่สอง S3 copy คือ destination,
endpoint และ region

=== ขั้น​ที่ 2 — แก้​โฟลเดอร์​และ​พาธ​เป็น​ชิป
<ขนท-2-แกโฟลเดอรและพาธเปนชป>
โฟลเดอร์​กับ​พาธ​ที่​ข้าม​แสดง​เป็น​ชิป (chip) กด × ลบ​ทีละ​อัน หรือ​พิมพ์​ใน​กล่อง​แล้ว​กด Enter
เพิ่ม มี​คำแนะนำ​ให้​เลือก และ​มี​ปุ่ม ปุ่ม Choose folder…
ที่​เปิด​ตัวเลือก​โฟลเดอร์​ของ​เครื่อง​ผ่าน​เซิร์ฟเวอร์ เซิร์ฟเวอร์​ตรวจ​ให้​ว่า​โฟลเดอร์​มี​จริง และ
depth อยู่​ใน​ช่วง 1 ถึง 32 ถ้า​ผิด​มัน​บอ​กว่า​ผิด​ตรงไหน

ค่า​ทั้งหมด​ถูก​เก็บ​ใน `_config.json` ใน​โฟลเดอร์​ดัชนี `relic-scan run` กับ​ปุ่ม Index
now ใช้​ไฟล์​เดียวกัน แต่​ถ้า​สั่ง​ด้วย​แฟลก​ใน​บรรทัด​คำสั่ง #strong[แฟลก​ชนะ​เสมอ]

=== ขั้น​ที่ 3 — S3 copy และ​กุญแจ
<ขนท-3-s3-copy-และกญแจ>
ช่อง S3 copy ใส่​ได้​แค่​ปลายทาง endpoint กับ region ส่วน​กุญแจ
#strong[ไม่​เก็บ​ใน​หน้า​นี้​เด็ดขาด] กุญแจ​มาจาก environment หรือ​ไฟล์
`~/.config/relic-scan/s3.env` ที่​มีสิทธิ์ 600 เท่านั้น เรื่อง​การ​ส่ง​จริงอยู่​ใน​บท​ที่ 7

#figure(image("/images/W21-page-settings.png", width: 100%, alt: "หน้า Settings: What to scan คือ​โฟลเดอร์ ~ พาธ​ที่​ข้าม depth 8 และ​ส่วน S3 copy"),
  caption: [
    หน้า Settings: What to scan คือ​โฟลเดอร์ \~ พาธ​ที่​ข้าม depth 8 และ​ส่วน S3
    copy
  ]
)

ยังมี​ตัวเลือก Check changes อัตโนมัติ​ทุก 15, 30 หรือ 60 นาที​ตอน​เซิร์ฟเวอร์​ทำงาน​อยู่
มัน​แค่​ตรวจ ไม่​เคย index เอง และ​มี​ปุ่ม​เลือก​ธีม (auto, dark, light) ซึ่ง auto
ตาม​ระบบ

หก​หน้า​เว็บ​ที่ผ่านมา​ทั้งสอง​บท​ครอบคลุม​เกือบ​ทั้ง​แอ​ป​แล้ว แต่​บางคน​ไม่​อยาก​เปิด​เบราว์เซอร์
บท​หน้า​จึง​พา​ไปดู​ดัชนี​เดียวกัน​ใน​เทอร์มินัล​ด้วย `relic-scan tui`

= บท​ที่ 5: ใน​เทอร์มินัล
<บทท-5-ในเทอรมนล>
สอง​บท​ก่อนหน้านี้​เรา​เปิด​ดัชนี​ผ่านหน้า​เว็บ แต่​บางครั้ง​เรา​ก็​ไม่​อยาก​ออกจาก​เทอร์มินัล เช่น ต่อ
ssh เข้าเครื่อง​อื่น หรือ​แค่​อยาก​เช็ก​เร็ว ๆ ว่า session ล่าสุด​ของ repo นี้​คือ​อะไร
relic-scan มี​หน้าจอ​เทอร์มินัล​ให้​ใช้​ใน​ตัว ชื่อ `relic-scan tui`
อ่าน​ดัชนี​ชุด​เดียว​กับ​หน้า​เว็บ ไม่​ได้​สร้าง​ข้อมูล​ใหม่ และ​ไม่​แตะ transcript เลย​แม้แต่​ไฟล์​เดียว

บท​นี้​เรา​จะ​เดินตาม​คีย์​เป็นลำดับ​เดียว​กับ​ที่​ใช้​ถ่ายภาพ คือ เปิด​โปรแกรม, กด `2`, กด Enter,
กด Backspace, กด `3`, พิมพ์ `/` ตาม​ด้วย `starfish`, กด Esc, กด Enter, กด
Esc อีกครั้ง​เพื่อ​ปิด​กล่อง, กด `4` และ​กด `6`
ทุก​หน้าจอ​ใน​บท​นี้​เป็น​ข้อ​ความจริง​ของ​หน้าจอ​ขนาด 100x30 บน​บ้าน​เดโม​เดิม ไม่​มี​อะไร​ตกแต่ง​เพิ่ม

== 5.1 เปิด relic-scan tui และ​อ่าน​แท็บ​แรก
<เปด-relic-scan-tui-และอานแทบแรก>
=== ขั้น​ที่ 1 — เปิด​โปรแกรม
<ขนท-1-เปดโปรแกรม>
พิมพ์​คำสั่ง​เดียว​ใน​โฟลเดอร์​ไหน​ก็ได้ เพราะ​โปรแกรม​หา​ดัชนี​ที่ `~/.relic-scan` เอง
ถ้า​เคย​รัน​ด้วย `--kind` หรือ `--month` ไว้ (บท​ที่ 2)
ก็​ใส่​ตัวเลือก​เดียวกัน​ตอน​เปิด​เพื่อ​ชี้​ไป​โฟลเดอร์​แยก​นั้น เช่น
`relic-scan tui --kind claude` ตัวเลือก​ทั้งหมด​ดู​ได้​ใน​ขั้น​ที่ 4

```bash
relic-scan tui
```

#figure(image("/images/30-tui-overview.png", width: 100%, alt: "หน้าจอ overview ของ relic-scan tui: totals, rows by type และ​แผนภูมิ sessions per month"),
  caption: [
    หน้าจอ overview ของ relic-scan tui: totals, rows by type และ​แผนภูมิ
    sessions per month
  ]
)

พอ​เปิด​มา #strong[หน้า​แรก​คือ overview] เสมอ ที่​หัว​จอ​บอ​กว่า​ดัชนี​นี้​มี​อะไร​อยู่
แล้ว​บรรทัด​ถัดไป​เป็น​แถบ​แท็บ​ทั้ง​หก​ให้​เห็น​ตั้งแต่แรก

```
 ▓▒░ RELIC·SCAN ░▒▓   144 sessions · 145 .jsonl · ~/.relic-scan
  1 overview   2 repos   3 sessions   4 paths   5 s3   6 ops
```

=== ขั้น​ที่ 2 — อ่าน​สาม​กล่อง​ของ overview
<ขนท-2-อานสามกลองของ-overview>
กล่อง​ซ้าย​บน `totals` สรุป​ตัวเลข​รวม กล่อง​ขวา​บน `rows by type` แยก​แถว​ตาม​ชนิด
ส่วน​กล่อง​ล่าง​เป็น​แผนภูมิ​แท่ง #strong[sessions per month] ตัวเลข​ทั้งหมด​ตรง​กับ​ที่​หน้า​เว็บ
Smart Scan เคย​โชว์ คือ 3,173 แถว และ 144 session

```
│rows        3,173                   │
│indexed     144 files · 1.4 MB      │
│span (BKK)  2026-07-02 → 2026-10-26 │
│repos       8                       │
```

ใน​กล่อง `totals` มี​สาม​บรรทัด​ย่อย​ของ sessions คือ `main 144`, `subagent 0` และ
`workflow 0` บ้าน​เดโม​มี​แต่ session หลัก ไม่​มี subagent เลย ส่วน `rows by type`
มี user text 628, assistant text 628, thinking 507, tool calls 705 และ
tool results 705 เรียง​เป็น​แท่ง​พร้อม​เปอร์เซ็นต์ บรรทัด​สุดท้าย tool errors 161 คิด​เป็น
22.8% ของ results ไม่​ใช่​ของ​แถว​ทั้งหมด ตัวหาร​ต่างกัน จึง​เขียน​กำกับ​ไว้​ใน​ภาพ​ว่า
`of results`

แผนภูมิ​ล่าง​แสดง​สี่​เดือน 2026-07 ถึง 2026-10 ใต้​แท่ง​เขียน​จำนวน session
ไว้​สามสิบ​กว่า​ต่อ​เดือน (37, 37, 37 และ 33) เดือน​สุดท้าย​แท่ง​เตี้ย​กว่า​นิดหน่อย
ถ้า​รวม​สี่​เดือน​จะ​ได้ 144 พอดี​กับ​กล่อง totals

=== ขั้น​ที่ 3 — แถบ​คีย์​ที่​ท้าย​จอ
<ขนท-3-แถบคยททายจอ>
บรรทัด​ล่าง​สุด​เป็น​ความจำ​ช่วย​ตลอดเวลา #strong[ไม่ต้อง​จำ​คีย์​จาก​บท​นี้​ก็ได้]
เพราะ​มัน​อยู่​บน​จอ​ทุก​แท็บ

```
 q quit  1-6/tab view  / filter  ↑↓ jk g G move  enter open  o page  ⌫ back
```

คีย์ `1` ถึง `6` กระโดด​ไป​แท็บ​ตรง ๆ ส่วน Tab กับ​ลูกศร​ขวา​ไล่​ไป​แท็บ​ถัดไป Shift+Tab
กับ​ลูกศร​ซ้าย​ถอยกลับ การ​เลื่อน​ใน​รายการ​ใช้​ลูกศร​ขึ้น​ลง หรือ `j` `k` แบบ vim ก็ได้ `g`
ไป​บนสุด `G` ไป​ล่าง​สุด PgUp กับ PgDn ข้าม​ทีละ 20 แถว และ `q` ออกจาก​โปรแกรม

=== ขั้น​ที่ 4 — ดู​ตัวเลือก​ทั้งหมด​ด้วย \-\-help
<ขนท-4-ดตวเลอกทงหมดดวย---help>
ถ้า​ลืม​ว่า​เปิด​โปรแกรม​ได้​แบบ​ไหน​บ้าง ให้​ถาม `--help` คำสั่ง​นี้​พิมพ์​แล้ว​จบ​ทันที
ไม่​เปิด​หน้าจอ​เต็ม

```bash
relic-scan tui --help
```

#figure(image("/images/50-help-tui.png", width: 100%, alt: "ผล​ของ relic-scan tui --help: บรรทัด​อธิบาย​โปรแกรม Usage และ​ตัวเลือก --out, --mouse, --kind, --help"),
  caption: [
    ผล​ของ relic-scan tui \-\-help: บรรทัด​อธิบาย​โปรแกรม Usage และ​ตัวเลือก
    \\-\-out, \-\-mouse, \-\-kind, \-\-help
  ]
)

อ่าน​แล้ว​เห็น​ว่า #strong[ตัวเลือก​มี​แค่​สาม​ตัว] `--out` ชี้​โฟลเดอร์​ดัชนี (ค่า​เริ่มต้น
`~/.relic-scan`), `--mouse` ให้​คลิก​แถว​เพื่อ​เลือก​แล้ว​คลิก​ซ้ำ​เพื่อ​เปิด​หน้า​เว็บ และ
`--kind` ใช้​ชื่อ kind เดียว​กับ​ที่​ตอน `run` ใช้ เพื่อ​เลือก​โฟลเดอร์
`~/.relic-scan/<kind>` เมื่อ​ไม่​ได้​ใส่ `--out`

== 5.2 จาก repo ลง​ไป​ถึง session
<จาก-repo-ลงไปถง-session>
=== ขั้น​ที่ 1 — กด 2 ดู repo ทั้ง​แปด
<ขนท-1-กด-2-ด-repo-ทงแปด>
กด `2` เพื่อ​เข้า​แท็บ repos แต่ละ​แถว​คือ​หนึ่ง repo มี​จำนวน session แยก​ตาม main,
แถว, thinking, errors และ​เวลา session ล่าสุด

```
╭ repos (enter: its sessions) · 8 ──────────────
│▶ tea-cart                      23        23
│  starfish-api                  16        16
│  codex (no cwd in path)        23        23
│  omp (no cwd in path)          16        16
```

#figure(image("/images/31-tui-repos.png", width: 100%, alt: "แท็บ repos: แปด repo เรียง​ตาม​จำนวน​แถว มี tea-cart อยู่​บนสุด"),
  caption: [
    แท็บ repos: แปด repo เรียง​ตาม​จำนวน​แถว มี tea-cart อยู่​บนสุด
  ]
)

#strong[ราย​การเรียง​ตาม​จำนวน​แถว ไม่​ใช่​ตาม​จำนวน session] tea-cart จึง​อยู่​บนสุด​ด้วย
23 session ส่วน codex กับ omp อยู่​ท้ายสุด​แม้ codex จะ​มี 23 session เท่า tea-cart
เพราะ session ของ​มัน​สั้น (ดู​ตัวเลข 152 และ 88 แถว​ใน​บท​ที่ 6) ลูกศร `▶`
หน้า​แถว​คือ​แถว​ที่​เลือก​อยู่

ข้อสังเกต​สอง​อย่าง อย่าง​แรก เครื่องหมาย (no cwd in path) บอ​กว่า Codex และ omp
ไม่​ได้​ฝัง​ชื่อ​โฟลเดอร์​งาน​ไว้​ใน path ของ​ไฟล์ relic-scan จึง​จัด​เป็น​กลุ่ม​ตาม​ชื่อ agent
อย่าง​ที่สอง จอ​กว้าง​แค่ 100 ตัวอักษร คอลัมน์​ขวา​สุด​จึง​ถูก​ตัด เช่น หัว​คอลัมน์ `rows` กลายเป็น
`row` และ​ตัวเลข​แถว​ของ tea-cart ที่​ควร​เป็น 662 ถูก​ตัด​เหลือ `66` ถ้า​อยาก​เห็น​เต็ม
ให้​ขยาย​หน้าต่าง​เทอร์มินัล​ให้​กว้าง​กว่า​นี้

=== ขั้น​ที่ 2 — กด Enter ลง repo ที่​เลือก
<ขนท-2-กด-enter-ลง-repo-ทเลอก>
เลื่อน `▶` ไป​ที่ repo ที่​ต้องการ​แล้ว​กด #strong[Enter] ใน​ภาพ​นี้​เรา​ไม่​ได้​เลื่อน จึง​เข้า
tea-cart ซึ่ง​เป็นแถว​แรก

โปรแกรม​สลับ​ไป​แท็บ sessions แต่​กรอง​ให้​เหลือ repo เดียว ชื่อ​กล่อง​บอก​ไว้​ชัด​ว่า
`sessions of tea-cart (⌫ back) · 23`

```
│▶ 2026-10-25 12:05 ● cf807765-59ce-459f-b tea-cart    43   14   7  11   2│
│  2026-10-24 19:06 ● db154a48-26e4-4b7c-b tea-cart    38   12   6  10   4│
```

#figure(image("/images/32-tui-repo-sessions.png", width: 100%, alt: "รายการ 23 session ของ tea-cart เรียง​จาก​ใหม่​ไป​เก่า"),
  caption: [
    รายการ 23 session ของ tea-cart เรียง​จาก​ใหม่​ไป​เก่า
  ]
)

คอลัมน์ `started (BKK)` เป็นเวลา​ไทย `sid` ถูก​ตัด​เหลือ 20 ตัว​แรก​ของ id ส่วน
`rows` คือ​แถว​ทั้งหมด​ของ session นั้น แล้ว `text`, `think`, `calls`, `errors`
แจกแจง​ว่า​แถว​เหล่านั้น​เป็น​อะไร session บนสุด​มี 43 แถว เป็น​ข้อความ 14, thinking 7,
tool calls 11 และ error 2 จุด​กลม `●` หลัง​เวลา​บอ​กว่า​เป็น session หลัก

=== ขั้น​ที่ 3 — Backspace กลับ แล้ว​กด 3 ดู​ทุก session
<ขนท-3-backspace-กลบ-แลวกด-3-ดทก-session>
กด Backspace (ใน​แถบ​คีย์​เขียน​ว่า `⌫ back`) เพื่อ​ถอยกลับ​ไป​แท็บ repos จากนั้น​กด `3`
เพื่อ​ไป​แท็บ sessions แบบ​ไม่​กรอง กด​เลข​สาม​จะ​ล้าง​การ​เลือก repo ทิ้ง​ให้​ด้วย จึง​เห็น​ทั้ง
144 session

```
╭ sessions, newest first (enter: open) · 144 ───
```

#figure(image("/images/33-tui-sessions.png", width: 100%, alt: "แท็บ sessions ทั้งหมด 144 รายการ เรียง​ใหม่​สุด​ก่อน"),
  caption: [
    แท็บ sessions ทั้งหมด 144 รายการ เรียง​ใหม่​สุด​ก่อน
  ]
)

บรรทัด​แรก​คือ session ของ omp ตัวเลข `144` ท้าย​ชื่อ​กล่อง​คือ​จำนวน​รายการ​ที่​เห็น
#strong[เลขท้าย​ชื่อ​กล่อง​เปลี่ยน​ตามตัว​กรอง​เสมอ] จึง​เป็น​วิธี​เช็ก​เร็ว​ว่า​กรอง​อยู่​หรือเปล่า
สังเกต​ด้วยว่า sid ของ omp ขึ้นต้น​ด้วย​วัน​เวลา (`2026-10-25T09-03-00-`) ต่าง​จาก
Claude ที่​เป็น uuid เพราะ omp ตั้งชื่อ​ไฟล์​ด้วย​เวลาเริ่ม

== 5.3 กรอง​รายการ เปิด​รายละเอียด และ​ข้าม​ไป​หน้า​เว็บ
<กรองรายการ-เปดรายละเอยด-และขามไปหนาเวบ>
=== ขั้น​ที่ 1 — กด / แล้ว​พิมพ์​คำ​ที่​ต้องการ
<ขนท-1-กด-แลวพมพคำทตองการ>
ตอน​อยู่​ใน​แท็บ sessions กด `/` แล้ว​พิมพ์ `starfish` จากนั้น​กด Enter ระหว่าง​พิมพ์
รายการ​จะ​หด​ตาม​ทุก​ตัวอักษร ส่วน Backspace ใน​โหมด​พิมพ์​ลบ​ทีละ​ตัวอักษร ไม่​ใช่​ถอยกลับ

```
╭ sessions, newest first (enter: open) · 16 ───
 …⌫ back   filter: starfish
```

#figure(image("/images/34-tui-filter.png", width: 100%, alt: "แท็บ sessions หลัง​กรอง​คำ​ว่า starfish เหลือ 16 รายการ ท้าย​จอ​มี filter: starfish"),
  caption: [
    แท็บ sessions หลัง​กรอง​คำ​ว่า starfish เหลือ 16 รายการ ท้าย​จอ​มี filter:
    starfish
  ]
)

เหลือ 16 session ตรง​กับ​ตัวเลข 16 ของ starfish-api ที่​แท็บ repos และ​ที่ DuckDB
จะ​นับ​ให้​ใน​บท​ที่ 6 #strong[ตัว​กรอง​นับ​เป็น​คำ​และ​ใช้​เงื่อนไข AND] ตาม README
ถ้า​พิมพ์​สอง​คำ แถว​ต้อง​มี​ทั้งสอง​คำ ที่​ท้าย​จอ​มี​บรรทัด `filter: starfish`
เตือน​ว่า​ตัว​กรอง​ยัง​ทำงาน​อยู่ แม้​เรา​จะ​กด Enter ออกจาก​โหมด​พิมพ์​แล้ว

=== ขั้น​ที่ 2 — Esc ล้าง​ตัว​กรอง แล้ว Enter เปิด​รายละเอียด
<ขนท-2-esc-ลางตวกรอง-แลว-enter-เปดรายละเอยด>
กด Esc เพื่อ​ล้าง​ตัว​กรอง รายการ​กลับมา​เป็น 144 แล้ว​กด Enter ที่​แถว​บนสุด ซึ่ง​ก็​คือ
session ของ omp เมื่อ​ครู่ จะ​มี​กล่อง​รายละเอียด​ซ้อน​ขึ้น​มา​เหนือ​รายการ

```
║● main session
║sid     2026-10-25T09-03-00-000Z_105f4e42-ab32-4fca-838b-8eeca0057739.jsonl
║repo    omp (no cwd in path)
║rows    6
║copies  1 · 2 KB largest
```

#figure(image("/images/35-tui-popup.png", width: 100%, alt: "กล่อง session ซ้อน​บน​รายการ แสดง sid, repo, เวลา, จำนวน​แถว​แยก​ชนิด และ copies"),
  caption: [
    กล่อง session ซ้อน​บน​รายการ แสดง sid, repo, เวลา, จำนวน​แถว​แยก​ชนิด และ
    copies
  ]
)

กล่อง​นี้​ให้ #strong[sid เต็ม] ซึ่ง​รายการ​ตัดทิ้ง พร้อม​เวลาเริ่ม​กับ​เวลา​สุดท้าย (16:03 ถึง
16:06) จำนวน​แถว​แยก​ชนิด และ​บรรทัด `copies` ที่​บอ​กว่า​ไฟล์​นี้​มี​กี่​สำเนา (1 สำเนา ใหญ่​สุด
2 KB) ถ้า​มี mirror หรือ backup สำเนา​จะ​นับ​รวม​ตรงนี้
เรื่อง​การ​พับ​สำเนา​เป็นแถว​เดียว​จะ​อธิบาย​ใน​บท​ที่ 9

ท้าย​กล่อง​บอก​คีย์​ที่​ใช้ได้​ใน​กล่อง​นี้​ไว้​ครบ

```
 c  copy `claude --resume <sid>`    o  open in the page   any other key …
```

=== ขั้น​ที่ 3 — c, o และ \-\-mouse
<ขนท-3-c-o-และ---mouse>
กด `c` เพื่อ​คัดลอก​คำสั่ง `claude --resume <sid>` ไว้​ใน​คลิป​บอร์ด
พร้อม​วาง​เพื่อ​กลับ​ไป​คุย​ต่อ​กับ session นั้น​ได้​เลย แต่ #strong[เฉพาะ session ของ
Claude Code เท่านั้น] ใน​โค้ด keys.rs session ของ agent อื่น​จะ​คัดลอก​แค่ sid เปล่า
ๆ เพราะ​คำสั่ง resume ของ Claude ใช้กับ Codex หรือ omp ไม่​ได้ session
ใน​ภาพ​นี้​เป็น​ของ omp จึง​ได้​เฉพาะ sid

กด `o` เพื่อ​เปิด​หน้า​เว็บ​ไป​ที่ session เดียวกัน คีย์ `o` ใช้ได้​นอก​กล่อง​ด้วย ตอน​เลือก​แถว
repo หรือ​โฟลเดอร์​อยู่ ก็​จะ​เปิด​หน้า​เว็บ​ตรงจุด​นั้น หน้า​เว็บ​คือ​หน้า​เดียว​กับ​ที่​เปิด​ด้วย
`relic-scan up` ใน​บท​ที่ 3 ส่วน​คีย์​อื่น​นอกจาก `c`, `o` และ `q` จะ​ปิด​กล่อง​ทั้งหมด

ถ้า​ชอบ​ใช้​เมาส์ ให้​เปิด​ด้วย `--mouse` คลิก​หนึ่ง​แถว​เพื่อ​เลือก คลิก​ซ้ำ​ที่​แถว​เดิม​เพื่อ​เปิด
ข้อมูล​นี้​มาจาก README เรา​ไม่​ได้​ถ่ายภาพ​ไว้ เพราะ​การ​คลิก​จับเป็น​ภาพนิ่ง​ไม่​ได้

== 5.4 แท็บ paths และ ops
<แทบ-paths-และ-ops>
=== ขั้น​ที่ 1 — ปิด​กล่อง​ด้วย Esc แล้ว​กด 4
<ขนท-1-ปดกลองดวย-esc-แลวกด-4>
กล่อง session ยัง​เปิด​ค้าง​อยู่ และ #strong[ทุก​คีย์​ที่​ไม่​ใช่ `c`, `o`, `q` จะ​แค่​ปิด​กล่อง]
ไม่​ได้​ส่ง​ต่อไป​ให้​แท็บ ถ้า​กด `4` ทันที กล่อง​จะ​ปิด​แล้ว​ยังอยู่​แท็บ sessions เดิม ภาพ​ชุด​นี้​จึง​กด
Esc ปิด​กล่อง​ก่อน แล้ว​ค่อย​กด `4`

```
╭ folders holding .jsonl (enter: its files) · 35 ───
│▶ …laris-co-tea-cart     23 …     23      0      0   662    306 KB│
```

#figure(image("/images/36-tui-jsonl.png", width: 100%, alt: "แท็บ paths: 35 โฟลเดอร์​ที่​มี .jsonl แถว​แรก​คือ tea-cart"),
  caption: [
    แท็บ paths: 35 โฟลเดอร์​ที่​มี \.jsonl แถว​แรก​คือ tea-cart
  ]
)

แท็บ paths แสดง​โฟลเดอร์​ที่​มี `.jsonl` อยู่ 35 โฟลเดอร์ พร้อม​จำนวน​ไฟล์ และ​ตัวเลข
indexed, skipped และ other โฟลเดอร์​แรก​คือ tea-cart 23 ไฟล์ 662 แถว 306 KB
กด Enter ที่​โฟลเดอร์​เพื่อ​ดู​ไฟล์​ข้างใน และ​กด Enter ที่​ไฟล์​เพื่อ​คัดลอก path ของ​มัน
Backspace ถอย​จาก​รายการ​ไฟล์​กลับมา​รายการ​โฟลเดอร์ เนื้อหา​เดียวกัน​ดู​เป็น​ภาพ​ได้ที่​หน้า
Paths ของ​เว็บ (บท​ที่ 3)

=== ขั้น​ที่ 2 — กด 6 ดู ops
<ขนท-2-กด-6-ด-ops>
คีย์ `6` เข้า​แท็บ ops ซึ่ง​เป็น​ภาพ​เดียว​กับ​คำสั่ง `relic-scan ops` แต่​วาด​ใหม่​ทุก 2 วินาที
ชื่อ​กล่อง​ก็​บอก​ไว้​ว่า `operations · refreshes every 2 s`

```
│CHAIN m5  idle  last pass 06:50:15 took 1:31 · next in about 0:00
│  ✓ scan                  31 s (typical 31 s)
│  ✓ extract               14 s (typical 14 s)
│  ✓ push to the hub        5 s (typical 5 s)
│  ✓ sync check            SYNC OK  text|1256 thinking|507 tools|1410
```

#figure(image("/images/37-tui-ops.png", width: 100%, alt: "แท็บ ops: chain ห้า​ขั้น​พร้อม​เวลา, PROCESSES none running และ HUB"),
  caption: [
    แท็บ ops: chain ห้า​ขั้น​พร้อม​เวลา, PROCESSES none running และ HUB
  ]
)

อ่าน​จาก​บน​ลง​ล่าง #strong[chain คือ​สายงาน​ห้า​ขั้น] ได้แก่ `scan`, `extract`,
`push to the hub`, `hub index + embed` และ `sync check`
แต่ละ​ขั้น​มี​เครื่องหมาย​ถูก เวลา​ที่​ใช้ และ​เวลา​ปกติ (typical) ให้​เทียบ ถ้า​ขั้น​ไหน​ช้า​กว่า​ปกติ
ตรงนี้​จะ​เห็น​ก่อนที่​ใคร​จะ​บ่น บรรทัด​สุดท้าย​คือ `SYNC OK` พร้อม​จำนวน​แถว​ที่​ฝั่ง​เรา​และ​ฝั่ง hub
นับ​ตรงกัน

ตัวเลข​สาม​ตัว​ใน `SYNC OK` บวก​กัน​ได้ 1256 + 507 + 1410 = 3,173
เท่ากับ​จำนวน​แถว​บน​หน้า Smart Scan ส่วน `PROCESSES` ขึ้น​ว่า `none running`
แปล​ว่า​ไม่​มี​โปรเซส​ของ chain รัน​อยู่​ตอนนั้น และ `HUB  not read (off)` แปล​ว่าไม่ได้​ถาม
hub เลย ซึ่ง​เป็น​ค่า​เริ่มต้น​ใน​ภาพ​นี้ เรื่อง chain ทั้ง​เส้น​จะ​อธิบาย​เต็ม​ใน​บท​ที่ 8

=== ขั้น​ที่ 3 — แท็บ s3 ที่​เหลือ
<ขนท-3-แทบ-s3-ทเหลอ>
แท็บ​ที่​ห้า #strong[`s3`] ไม่​อยู่​ใน​ชุด​ภาพ เพราะ​บ้าน​เดโม​ไม่​ได้​ตั้ง​ปลายทาง S3 ไว้ ตาม
README ใน​แท็บ​นี้ `c` ตรวจ​เทียบ​กับ store โดย​ไม่​ส่ง​อะไร และ `s` ส่ง​เฉพาะ​เดือน​ที่​ยัง​ขาด
โดย​ไม่​ทับ​เดือน​ที่​ต่างกัน เป็นงาน​เดียว​กับ​การ์ด S3 copy ใน​หน้า Export ซึ่ง​บท​ที่ 7 จะ​พา​ไปดู

ตอนนี้​เรา​มี​สาม​ทางเข้า​ดัชนี​แล้ว คือ​หน้า​เว็บ เทอร์มินัล และ CLI ทาง​ที่สี่​ยืดหยุ่น​ที่สุด​คือ​ถาม​ด้วย
SQL ตรง ๆ บท​ถัดไป​จะ​เปิด DuckDB ขึ้น​มา​ถาม​ไฟล์ Parquet เหล่านี้​ทีละ​คำถาม

= บท​ที่ 6: ถาม​ดัชนี​ด้วย DuckDB
<บทท-6-ถามดชนดวย-duckdb>
หน้า​เว็บ​กับ TUI ตอบคำถาม​ที่​คน​ทำ relic-scan คิด​ไว้​ล่วงหน้า
แต่​คำถาม​ของ​เรา​ไม่​ได้​ตรง​กับ​เขา​เสมอไป เช่น โมเดล​ไหน​ตอบ​กี่​ครั้ง​ใน​เดือน​ไหน หรือ session
ไหน​กิน​โท​เคน​มาก​สุด พอ​ดัชนี​เป็น​ไฟล์ Parquet ธรรมดา เรา​ก็​เปิด​ด้วย​เครื่องมือ​อะไร​ก็​ได้ที่​อ่าน
Parquet เป็น บท​นี้​ใช้ DuckDB เพราะ​ติดตั้ง​ง่าย​และ​รัน SQL บน​ไฟล์​ได้​เลย
ไม่ต้อง​โหลด​เข้า​ฐานข้อมูล​ก่อน

ทุก​คำสั่ง​ใน​บท​นี้​รัน​ที่​โฟลเดอร์ `~/.relic-scan` ของ​บ้าน​เดโม (`cd ~/.relic-scan`
ก่อน) เพราะ path ใน SQL เป็น `'*.parquet'` และ `'meta/sessions.parquet'`
แบบ​สัมพัทธ์ และ​ก่อน​เริ่ม​มี​กฎ​เรื่อง​ความเป็นส่วนตัว​หนึ่ง​ข้อ​จาก AGENTS.md
ทรานสคริปต์​เก็บงาน​ส่วนตัว อ่าน​เฉพาะที่​ถามหา และ​อย่า​คัดลอก​แถว​ออกจาก​เครื่อง

== 6.1 นับ​ทุกอย่าง​ให้ได้​เลข​เดียว​กับ​แอ​ป
<นบทกอยางใหไดเลขเดยวกบแอป>
=== ขั้น​ที่ 1 — นับ​แถว​ทั้งหมด
<ขนท-1-นบแถวทงหมด>
เริ่ม​จาก​คำถาม​ที่​ง่าย​ที่สุด ไฟล์ `YYYY-MM.parquet` หนึ่ง​ไฟล์​ต่อ​เดือน ใช้ glob
`'*.parquet'` รวม​ทุก​เดือน​ใน​โฟลเดอร์ แต่​ไม่​รวม `meta/` เพราะ​อยู่​ใน​โฟลเดอร์​ย่อย

```bash
duckdb -c \
  "select count(*) as all_rows from '*.parquet'"
```

#figure(image("/images/12-duck-total.png", width: 62%, alt: "DuckDB นับ​แถว​ทั้งหมด​ได้ 3173"),
  caption: [
    DuckDB นับ​แถว​ทั้งหมด​ได้ 3173
  ]
)

```
│     3173 │
```

ก่อน​ไป​ต่อ ลองดู​ว่า​ไฟล์​เดือน​หนึ่ง​มี​คอลัมน์​อะไร​บ้าง​ด้วย `describe`

```bash
duckdb -c "describe select * from '2026-10.parquet'"
```

#figure(image("/images/46-describe-month.png", width: 58%, alt: "ผล describe ของ​ไฟล์ 2026-10.parquet: 24 คอลัมน์ ชื่อ​และ​ชนิด เช่น sid, ts_th, role, kind, name, text, body, err, model, out_tokens"),
  caption: [
    ผล describe ของ​ไฟล์ 2026-10.parquet: 24 คอลัมน์ ชื่อ​และ​ชนิด เช่น sid,
    ts\_th, role, kind, name, text, body, err, model, out\_tokens
  ]
)

ผล​มี 24 คอลัมน์ ที่​ใช้​ใน​บท​นี้​คือ `role`, `kind`, `name`, `engine`, `sid`,
`ts_th`, `text`, `body`, `err`, `model`, `msg_id` และ​คอลัมน์​โท​เคน
`out_tokens`, `in_tokens`, `cache_read`, `cache_write` ส่วน `machines` กับ
`sources` เป็น​อาร์​เรย์ (`VARCHAR[]`)

ผล​นับ​แถว​คือ 3173 แถว #strong[เลข​นี้​ต้อง​ตรง​กับ​ยอด​บน​หน้า Smart Scan] (3,173)
ถ้า​ไม่​ตรง​แปล​ว่า​เรา​มอง​คนละ​โฟลเดอร์ หรือ​ดัชนี​ไม่​ได้​ถูก​อัปเดต​แล้ว AGENTS.md
แนะนำ​ให้​นับ​ทั้งหมด​ก่อน​ทุกครั้ง แล้ว​ค่อย​รายงาน​ชิ้นส่วน​เทียบ​กับ​ยอดรวม จะ​ได้​ไม่​หลง​ว่า
"ข้อความ​ผู้ใช้ 628" คือ​ทั้งหมด​ของ​ข้อมูล

=== ขั้น​ที่ 2 — แยก​ตาม​ว่า​แต่ละ​แถว​คือ​อะไร
<ขนท-2-แยกตามวาแตละแถวคออะไร>
หนึ่ง​แถว​อาจ​เป็น​ข้อความ ความคิด คำ​เรียก​เครื่องมือ หรือ​ผลลัพธ์​ของ​เครื่องมือ ดัชนี​บอก​ด้วย
#strong[สอง​คอลัมน์ คือ `role` และ `kind`] ไม่​ใช่​คอลัมน์​เดียว ข้อความ​กับ thinking มี
`role` (user, assistant, thinking) แต่ `kind` ว่าง ส่วน​การ​เรียก​เครื่องมือ​มี
`kind = 'call'` (ชื่อ​เครื่องมือ​อยู่​ใน `name`) และ​ผลลัพธ์​มี `kind = 'result'`
โดย​ไม่​มี `role` คำสั่ง​ข้างล่าง​จึง​ใช้ `coalesce(nullif(...))`
เลือก​ค่าที่​ไม่​ว่าง​ของ​สอง​คอลัมน์​มา​รวม​เป็น​คอลัมน์​เดียว​ชื่อ `what`

```bash
duckdb -c \
  "select coalesce(nullif(kind,''), nullif(role,''), '(none)') as what, count(*) as n from '*.parquet' group by 1 order by n desc"
```

#figure(image("/images/13-duck-what.png", width: 100%, alt: "ตาราง​จำนวน​แถว​แยก​ตาม​ชนิด: result, call, assistant, user, thinking"),
  caption: [
    ตาราง​จำนวน​แถว​แยก​ตาม​ชนิด: result, call, assistant, user, thinking
  ]
)

```
│ result    │   705 │
│ call      │   705 │
│ assistant │   628 │
│ user      │   628 │
│ thinking  │   507 │
```

บวก​ห้า​แถว​ได้ 705 + 705 + 628 + 628 + 507 = 3173 พอดี จึง​ไม่​มี​แถว​ตกหล่น
และ​ไม่​มี​แถว `(none)` ใน​บ้าน​เดโม ซึ่ง​ข้อมูลจริง​อาจ​มี​ประปราย​ได้ (AGENTS.md
บอ​กว่า​บาง​แถว​ไม่​มี​ทั้ง role และ kind) ตัวเลข​ชุด​นี้​ตรง​กับ `rows by type` ที่​เห็น​ใน TUI
บท​ที่ 5 ทุก​ตัว การ​จับคู่ call กับ result ที่​เท่ากัน (705 ต่อ 705) ก็​เป็น​สัญญาณ​ที่​ดี
คือ​ทุก​การ​เรียก​เครื่องมือ​มี​ผลลัพธ์​กลับมา

หนึ่ง​ข้อ​ควร​รู้เรื่อง thinking ตั้งแต่​ราว​เดือน​มีนาคม 2026 Claude Code เขียน​บล็อก
thinking ส่วนใหญ่​เป็น​ลายเซ็น​อย่าง​เดียว ไม่​มี​ข้อความ แถว​พวก​นี้​ยัง​นับ​เป็น thinking แต่
`text` ว่าง จึง​ไม่​มี​อะไร​ให้​อ่าน​หรือ​ทำ embedding

=== ขั้น​ที่ 3 — แยก​ตาม engine
<ขนท-3-แยกตาม-engine>
คอลัมน์ `engine` บอ​กว่า​แถว​มาจาก agent ตัว​ไหน นับ​ด้วย `group by` ธรรมดา

```bash
duckdb -c \
  "select engine, count(*) as n from '*.parquet' group by 1 order by n desc"
```

#figure(image("/images/14-duck-engine.png", width: 90%, alt: "จำนวน​แถว​ต่อ engine: claude, codex, omp"),
  caption: [
    จำนวน​แถว​ต่อ engine: claude, codex, omp
  ]
)

```
│ claude  │  2933 │
│ codex   │   152 │
│ omp     │    88 │
```

#strong[2933 + 152 + 88 ได้ 3173 อีกครั้ง] Claude Code ครอง​ข้อมูล​แทบ​ทั้งหมด ส่วน
Codex กับ omp มี​นิดเดียว ตัวเลข 152 และ 88 ตรง​กับ​คอลัมน์ rows ของ
`codex (no cwd in path)` และ `omp (no cwd in path)` ที่​เห็น​ใน​แท็บ repos ของ
TUI

== 6.2 session ต่อ repo
<session-ตอ-repo>
=== ขั้น​ที่ 1 — ถาม​จาก​ตาราง meta
<ขนท-1-ถามจากตาราง-meta>
ตาราง `meta/sessions.parquet` มี​หนึ่ง​แถว​ต่อ​หนึ่ง session จึง​ใช้​ตอบคำถาม​ระดับ
session ได้​เร็ว ไม่ต้อง​กวาด​ทุก​แถว​ของ​เดือน คอลัมน์ `cat` คือ​ชื่อ repo และ `n_rows`
คือ​จำนวน​แถว​ของ session นั้น

```bash
duckdb -c \
  "select cat as repo, count(*) as sessions, sum(n_rows) as n_rows from 'meta/sessions.parquet' group by 1 order by sessions desc limit 8"
```

#figure(image("/images/15-duck-repos.png", width: 87%, alt: "ตาราง session และ​แถว​ต่อ repo ทั้ง​แปด"),
  caption: [
    ตาราง session และ​แถว​ต่อ repo ทั้ง​แปด
  ]
)

```
│ tea-cart               │       23 │    662 │
│ codex (no cwd in path) │       23 │    152 │
│ lantern-app            │       18 │    462 │
│ omp (no cwd in path)   │       16 │     88 │
```

เลข 662 ของ tea-cart คือ​ตัวเลข​เต็มที่ TUI ตัดทิ้ง​เหลือ `66` ใน​บท​ที่ 5 ส่วน session
รวมกัน 23 + 23 + 18 + 18 + 16 + 16 + 16 + 14 ได้ 144 ตรง​กับ​จำนวน session
ทั้งหมด #strong[ผลรวม​เป็น​เครื่องมือ​ตรวจ​ที่​ถูก​ที่สุด] ถ้า​รวม​แล้ว​ไม่​เท่า​ยอด​ที่​รู้
ก็​แปล​ว่า​เรา​ตัด​แถว​ทิ้ง​ไป​ที่ `limit` หรือ​มี repo เกิน​แปด

== 6.3 โมเดล​และ​โท​เคน​ต่อ​เดือน
<โมเดลและโทเคนตอเดอน>
=== ขั้น​ที่ 1 — นับ API call และ​โท​เคน​ที่​ตอบ​ออกมา
<ขนท-1-นบ-api-call-และโทเคนทตอบออกมา>
คำถาม​นี้​ซับซ้อน​กว่า​ข้อ​ก่อนหน้า เพราะ​หนึ่ง​การ​ตอบ​จาก API ไม่​ใช่​หนึ่ง​แถว Claude หนึ่ง​คำตอบ
(หนึ่ง `msg_id`) แตก​เป็น​หลาย​แถว หนึ่ง​แถว​ต่อ​บล็อก​เนื้อหา เช่น ข้อความ​หนึ่ง​แถว thinking
หนึ่ง​แถว และ tool call อีก​หนึ่ง​แถว ถ้า​เรา `sum(out_tokens)` ตรง ๆ
แล้ว​ทุก​แถว​ถือ​โท​เคน​ซ้ำ​กัน ก็​จะ​นับ​ซ้ำ

ดัชนี​แก้ปัญหา​นี้​ไว้​แล้ว #strong[`out_tokens` ใส่​ไว้​ที่​แถว​เดียว​ต่อ `msg_id`] ดังนั้น
`sum(out_tokens)` นับ​แต่ละ​คำตอบ​หนึ่ง​ครั้ง​พอดี และ `count(distinct msg_id)`
ก็​นับ​จำนวน API call ได้​ตรง ๆ ส่วน `model` ติด​อยู่​กับ​แถว​ข้อความ thinking และ tool
call ของ assistant

```bash
duckdb -c \
  "select left(ts_th,7) as month, model, count(distinct msg_id) as calls, sum(out_tokens) as out_tokens from '*.parquet' where model is not null group by all order by month, calls desc limit 12"
```

#figure(image("/images/16-duck-tokens.png", width: 100%, alt: "ตาราง​โมเดล, จำนวน call และ out_tokens ต่อ​เดือน 12 แถว​แรก"),
  caption: [
    ตาราง​โมเดล, จำนวน call และ out\_tokens ต่อ​เดือน 12 แถว​แรก
  ]
)

```
│ 2026-07 │ claude-sonnet-5-5 │    52 │      70967 │
│ 2026-07 │ glm-5.3           │    10 │       4925 │
│ 2026-07 │ gpt-6-astra       │     0 │       NULL │
```

คำสั่ง​นี้​ใช้ `left(ts_th, 7)` ตัด​เวลา​ไทย​เหลือ `ปี-เดือน` และ `group by all`
จัดกลุ่ม​ตาม​คอลัมน์​ที่​ไม่​ได้​เป็น aggregate ให้​เอง ท้ายสุด​มี `limit 12`
ภาพ​จึง​เห็น​แค่​สอง​เดือน​กับ​ครึ่ง ถ้า​อยาก​เห็น​ทั้ง​สี่​เดือน​ให้​เอา `limit` ออก

=== ขั้น​ที่ 2 — อ่าน​แถว​ที่​เป็น 0 และ NULL
<ขนท-2-อานแถวทเปน-0-และ-null>
ตรงนี้​คือ​จุด​ที่​ควร​หยุด​อ่าน​ช้า ๆ แถว `gpt-6-astra` ของ Codex แสดง #strong[`calls`
เป็น 0 และ `out_tokens` เป็น NULL] ทั้งสอง​เดือน ไม่​ได้​แปล​ว่า Codex ไม่​เคย​ตอบ
ซึ่ง​เรา​รู้​ว่า​ตอบ เพราะ​บ้าน​เดโม​มี 152 แถว​ของ Codex ใน​ภาพ 14 แล้ว

สาเหตุ​อยู่​ที่ AGENTS.md ข้อมูล​ของ Codex มี `model` (ดึง​จาก `turn_context` ล่าสุด)
แต่ #strong[ไม่​มี `msg_id` และ​ยัง​ไม่​มี `out_tokens`] ดังนั้น
`count(distinct msg_id)` ของ​แถว​พวก​นี้​นับ​ค่า NULL ได้​ศูนย์ และ `sum` ของ​ค่า NULL
ล้วน​ได้ NULL ไม่​ใช่​ศูนย์ ส่วน​เงื่อนไข `where model is not null`
ผ่าน​เพราะ​แถว​มี​โมเดล​จริง กลุ่ม `gpt-6-astra` เลย​โผล่​ขึ้น​มา​ใน​ตาราง​แบบ​มีชื่อ​แต่​ไม่​มี​ตัวเลข

บทเรียน​คือ อย่า​อ่าน​ศูนย์​ว่า "ไม่​ได้​ใช้" ถ้า​ต้องการ​เปรียบเทียบ​โมเดล​ข้าม agent
ให้​ใช้​จำนวน​แถว (`count(*)`) แทน​สำหรับ Codex
และ​จำไว้​ว่า​โท​เคน​ของ​มัน​ยัง​ไม่​มี​ใน​ดัชนี​รุ่น​นี้ ฝั่ง omp ไม่​เป็น​แบบนี้ `glm-5.3` มี​ทั้ง call
(10) และ​โท​เคน (4925) เพราะ omp ส่ง `responseId` มา​เป็น `msg_id` ให้

== 6.4 โท​เคน​ต่อ session
<โทเคนตอ-session>
=== ขั้น​ที่ 1 — ดู​คอลัมน์​โท​เคน​ใน meta/sessions
<ขนท-1-ดคอลมนโทเคนใน-metasessions>
คำถาม​เดียวกัน​แต่​ระดับ session ไม่ต้อง​คำนวณ​เอง เพราะ `meta/sessions.parquet`
สรุป​ไว้​ให้​ใน​หลาย​คอลัมน์ #strong[ที่​ขึ้นต้น​ด้วย​โมเดล​และ​โท​เคน] คือ `models`
(รายชื่อ​โมเดล​ที่​ตอบ เรียงตัว​ที่​ตอบ​มาก​สุด​ก่อน), `model_msgs`
(จำนวน​คำตอบ​ของ​แต่ละ​โมเดล ตามลำดับ​เดียวกัน), `in_tokens`, `out_tokens`,
`cache_read_tokens` และ `cache_write_tokens` ทั้งหมด​รวม​จาก​การ​ตอบ​ของ API
โดย​นับ​แต่ละ​ครั้งหนึ่ง​หน ถ้า​ทรานสคริปต์​ไม่​บันทึก​ไว้ (อย่าง​กรณี Codex) ค่า​เป็น 0

```bash
duckdb -c \
  "select sid, models, model_msgs, in_tokens, out_tokens, cache_read_tokens from 'meta/sessions.parquet' where out_tokens > 0 order by out_tokens desc limit 4"
```

#figure(image("/images/17-duck-session.png", width: 96%, alt: "สี่ session ที่​ตอบ​ยาว​สุด: models, model_msgs, in_tokens, out_tokens, cache_read_tokens"),
  caption: [
    สี่ session ที่​ตอบ​ยาว​สุด: models, model\_msgs, in\_tokens, out\_tokens,
    cache\_read\_tokens
  ]
)

```
│ 95f11992 │ [claude-haiku-5-5]  │ [7]        │       121 │      11285 │
│ 28394ccd │ [claude-sonnet-5-5] │ [6]        │       135 │      11250 │
```

`models` กับ `model_msgs` เป็น​อาร์​เรย์ จึง​เห็น​วงเล็บเหลี่ยม session
บนสุด​ตอบ​ด้วย​โมเดล​เดียว​คือ haiku จำนวน 7 คำตอบ ถ้า session ใช้​สอง​โมเดล​สลับ​กัน
อาร์​เรย์​ทั้งสอง​จะ​มี​สอง​ช่อง เรียง​คู่​กัน ใช้ `unnest` ใน​ภายหลัง​ได้​ถ้า​ต้องการ​แตก​เป็นแถว

ถ้า​อยาก​เห็น​คอลัมน์​ทั้งหมด​ของ​ตาราง​นี้ ให้ `describe` เหมือน​ที่​ทำ​กับ​ไฟล์​เดือน

```bash
duckdb -c "describe select * from 'meta/sessions.parquet'"
```

#figure(image("/images/47-describe-sess.png", width: 64%, alt: "ผล describe ของ meta/sessions.parquet: 38 คอลัมน์ รวม models, model_msgs, in_tokens, out_tokens, cache_read_tokens, cache_write_tokens"),
  caption: [
    ผล describe ของ meta/sessions.parquet: 38 คอลัมน์ รวม models,
    model\_msgs, in\_tokens, out\_tokens, cache\_read\_tokens,
    cache\_write\_tokens
  ]
)

ภาพ​มี 38 คอลัมน์ ชนิด​ของ `models` เป็น `VARCHAR[]` และ `model_msgs` เป็น
`BIGINT[]` ตรง​กับ​ที่​เห็น​เป็น​วงเล็บเหลี่ยม ส่วน​คอลัมน์​โท​เคน​ทั้ง​สี่​เป็น `BIGINT`

=== ขั้น​ที่ 2 — เทียบ in, out และ cache
<ขนท-2-เทยบ-in-out-และ-cache>
ตัวเลข​ใน​สาม​คอลัมน์​นี้​ต่างกัน​มาก #strong[cache\_read\_tokens คือ​คอลัมน์​ที่​ใหญ่​สุด​เสมอ]
แถว​บนสุด​มี in\_tokens แค่ 121 และ out\_tokens 11285 แต่ cache\_read\_tokens
ถึง 335324 เหตุผล​คือ Claude Code ส่ง​บริบท​ของ​บทสนทนา​ทั้งหมด​ซ้ำ​ใน​ทุก​คำขอ
แล้ว​ส่วน​ที่​เหมือนเดิม​ถูก​อ่าน​จาก cache จึง​ไม่​นับ​เป็น input ใหม่
การ​รวม​สาม​คอลัมน์​เข้าด้วยกัน​เป็น "โท​เคน​ที่​ใช้" จึง​ทำให้​เลข​บวม​โดย​ไม่​ได้​บอก​อะไร
ควร​รายงาน​แยก​สาม​ช่อง

ข้อควรระวัง​จาก AGENTS.md อีก​ข้อ เดือน​ที่​ถูก​เขียน​ก่อน​จะ​มี​คอลัมน์ input และ cache
จะ​อ่าน​ค่า​เหล่านั้น​เป็น NULL และ​คอลัมน์​โท​เคน​ใน `meta/sessions.parquet`
นับ​เฉพาะ​เดือน​ที่​มี​คอลัมน์​เหล่านั้น ถ้า​เจอ​ศูนย์​หรือ NULL เต็มไปหมด​ใน​ดัชนี​เก่า ให้​ย้าย​โฟลเดอร์
`~/.relic-scan` ไป​เก็บ แล้ว​รัน `relic-scan run` สร้าง​ใหม่
ข้อมูล​ใน​บ้าน​เดโม​นี้​สร้าง​ด้วย​รุ่น​ปัจจุบัน จึง​ไม่​เจอ​ปัญหา​นี้

== 6.5 อ่าน​บทสนทนา​ตามลำดับ
<อานบทสนทนาตามลำดบ>
=== ขั้น​ที่ 1 — เลือก session แล้ว​เรียง​ตามเวลา
<ขนท-1-เลอก-session-แลวเรยงตามเวลา>
ถ้า​เจอ session ที่​น่าสนใจ จะ​อ่าน​เนื้อหา​ก็​กรอง​ด้วย `sid` เรียง​ด้วย `ts` และ​เลือก​เฉพาะ
role ที่​เป็น​คน​คุย​กัน (`user`, `assistant`) ตัด thinking และ tool ออก
ใน​ตัวอย่าง​ใช้ sid เต็ม​ของ session หนึ่ง​ใน starfish-api (ที่ TUI ใน​บท​ก่อน​กรอง​เจอ)
และ​ตัด​ข้อความ​เหลือ 70 ตัวอักษร​ด้วย `left(text, 70)` เพื่อให้​พอดี​จอ

```bash
duckdb -c \
  "select role, left(text, 70) as text from '*.parquet' where sid = '0af4d28e-a79c-4ad8-bb26-26a0708bfab4' and role in ('user','assistant') order by ts limit 8"
```

#figure(image("/images/18-duck-read.png", width: 98%, alt: "บทสนทนา​แปด​ข้อความ​ของ session หนึ่ง สลับ user กับ assistant"),
  caption: [
    บทสนทนา​แปด​ข้อความ​ของ session หนึ่ง สลับ user กับ assistant
  ]
)

```
│ user      │ summarise yesterday's errors from the log                  │
│ assistant │ Three kinds: two timeouts to the payment sandbox, one bad … │
│ user      │ write the tour page for new members                        │
```

#strong[ข้อความ​ของ​ผู้​ใช้กับ assistant สลับ​กัน​ครบ​สี่​คู่] เดโม​ของ​เรา​มี​สี่​คู่​ต่อ session
แปด​ข้อความ​จึง​ครอบคลุม​ทั้งหมด​พอดี ใน session จริง​ที่​ยาว​กว่า​นี้ `limit` จะ​ตัด​กลางคัน
ให้​เปลี่ยนเป็น `offset` เดิน​ต่อ หรือ​กรอง​ด้วย​คำ​สำคัญ เช่น `text ilike '%parquet%'`
แบบ​ที่ AGENTS.md ยกตัวอย่าง​ไว้

== 6.6 บรรทัด​ที่​ถูก redact
<บรรทดทถก-redact>
=== ขั้น​ที่ 1 — หา​แถว​ที่​มี​คอลัมน์ red
<ขนท-1-หาแถวทมคอลมน-red>
ถ้า​ทรานสคริปต์​มี​คีย์​ลับ​หลุด​เข้ามา​ใน​ข้อความ relic-scan จะ​ไม่​เก็บ​ค่า​จริง​ไว้​ใน​ดัชนี
แต่​แทน​ด้วย​เครื่องหมาย​บอก​ชนิด และ​จด​ชนิด​นั้น​ไว้​ใน​คอลัมน์ `red` แถว​ทั่วไป​คอลัมน์​นี้​เป็น NULL
จึง​กรอง​หา​แถว​ที่​ถูก redact ได้​ด้วย `red is not null`

```bash
duckdb -c \
  "select left(text, 90) as text, red from '*.parquet' where red is not null"
```

#figure(image("/images/19-duck-redacted.png", width: 100%, alt: "หนึ่ง​แถว​ที่​ข้อ​ความถูก redact และ​คอลัมน์ red เป็น openai-key"),
  caption: [
    หนึ่ง​แถว​ที่​ข้อ​ความถูก redact และ​คอลัมน์ red เป็น openai-key
  ]
)

```
…(the key is [openai-key:sk-d…36], do not │ openai-key │
```

บ้าน​เดโม​มี​คีย์​ปลอม​ที่​ฝัง​ไว้​หนึ่ง​ตัว (ขึ้นต้น `sk-demo-`) ใน​หนึ่ง prompt และ​ผล​คือ​มี​แถว​เดียว​ที่
`red` ไม่​ว่าง ใน​ข้อความ คีย์​ถูก​ย่อ​เป็น `[openai-key:sk-d…36]`
เหลือ​แค่​ต้น​กับ​ท้าย​สอง​สาม​ตัวอักษร พอให้​รู้​ว่า​เคย​มี​อะไร​ตรงนั้น แต่​ไม่​พอให้​เอา​ไป​ใช้​ต่อ
#strong[ชนิด​คีย์​อยู่​ใน​วงเล็บ​และ​ใน​คอลัมน์ red] ซึ่ง​ช่วย​ให้​นับ​ได้​ว่า​มี secret
ชนิด​ไหน​หลุด​เข้ามา​กี่​ครั้ง เรื่อง​นี้​บท​ที่ 9 จะ​กลับมา​อธิบาย​เต็ม

== 6.7 months.json และ \_scans.jsonl
<months.json-และ-_scans.jsonl>
=== ขั้น​ที่ 1 — ดู​ตรา​ผนึก​ของ​เดือน​แรก
<ขนท-1-ดตราผนกของเดอนแรก>
ไม่​ใช่​ทุก​ไฟล์​ใน​ดัชนี​ที่​เป็น Parquet ไฟล์ `meta/months.json` เป็น JSON ธรรมดา เก็บ
#strong[หนึ่ง​รายการ​ต่อ​เดือน] ใช้ `jq` ดึง​รายการ​แรก​มา​ดู (ต้อง​มี `jq` ติดเครื่อง)

```bash
jq -c '.months[0]' ~/.relic-scan/meta/months.json
```

#figure(image("/images/22-months-json.png", width: 100%, alt: "JSON ของ​เดือน 2026-07: rows 819, sealed true, late_rows 0"),
  caption: [
    JSON ของ​เดือน 2026-07: rows 819, sealed true, late\_rows 0
  ]
)

```
{"month":"2026-07","rows":819,"first_ts":"2026-07-02T09:08:55.000Z",…
"late_rows":0,"late_total":0,"sealed":true,…
"sealed_rows":819,… "resealed":0,…}
```

สาม​ฟิลด์​ที่​ควร​อ่าน​ก่อน `rows` คือ 819 แถว​ของ​เดือน 2026-07 `sealed: true`
บอ​กว่า​เดือน​นี้​ปิด​แล้ว (อยู่​ก่อน​เดือน​ปัจจุบัน​ตามเวลา​กรุงเทพ) และ `late_rows: 0`
บอ​กว่า​ไม่​มี​แถว​ใหม่​แทรก​ย้อนหลัง​เข้ามา​ใน​เดือน​ที่​ปิด ส่วน `keys` กับ `sealed_keys`
เป็น​แฮ​ช​ของ​กุญแจ​แถว​ทั้งหมด ใน​ภาพ​สอง​ค่า​เท่ากัน (`deb9bcc2…`)
แปล​ว่า​เนื้อ​เดือน​นี้​ไม่​เปลี่ยน​ตั้งแต่​ปิด ส่วน `resealed: 0`
นับ​จำนวน​ครั้ง​ที่​แถว​มาสาย​แล้ว​ทำให้​ปิดผนึก​ซ้ำ การ​ตรวจ​จริง​ทำ​ด้วย
`relic-scan seals --verify` ใน​บท​ที่ 7

=== ขั้น​ที่ 2 — ดู​บันทึก​การ​สแกน​ครั้ง​ล่าสุด
<ขนท-2-ดบนทกการสแกนครงลาสด>
ไฟล์ `_scans.jsonl` เป็น​บันทึก​ต่อท้าย​ทุกครั้งที่ index หนึ่ง​รอบ​หนึ่ง​บรรทัด เหมือน​ที่​หน้า
Scan log และ​คำสั่ง `relic-scan log` อ่าน เรา​เอา​บรรทัด​สุดท้าย​มา​ดู และ​ตัด​ฟิลด์
`diff` ซึ่ง​ยาว​ทิ้ง​ด้วย `del(.diff)`

```bash
tail -1 ~/.relic-scan/_scans.jsonl | jq 'del(.diff)'
```

#figure(image("/images/23-scans-jsonl.png", width: 100%, alt: "JSON ของ​การ​สแกน​ล่าสุด: grown 1, unchanged 143, rows 3173, sessions 144"),
  caption: [
    JSON ของ​การ​สแกน​ล่าสุด: grown 1, unchanged 143, rows 3173, sessions 144
  ]
)

```
{"past_changed":0,"new":0,"grown":1,"rewritten":0,"unchanged":143,…
"files":1,"bytes":783,"scan_secs":0.206738084,…"job":"index",…
"rows":3173,"sessions":144,"diff_total":1}
```

บรรทัด​นี้​เล่าเรื่อง​รอบ​ที่ transcript หนึ่ง​ไฟล์​โต​ขึ้น​ใน​บท​ที่ 2 `grown: 1` กับ
`unchanged: 143` รวมกัน​ได้ 144 ไฟล์​ที่ index แล้ว อ่าน​จริง​เพียง​หนึ่ง​ไฟล์
(`files: 1`) ขนาด 783 ไบต์ ใช้เวลา​สแกน​ราว 0.2 วินาที จบ​รอบ​ด้วย `rows: 3173`
และ `sessions: 144` ซึ่ง​เป็นยอด​เดียว​กับ​ที่ DuckDB นับ​ใน​ขั้นแรก​ของ​บท​นี้
#strong[ตัวเลข​ทั้ง​สาม​ทาง​ตรงกัน ไม่​ว่า​จะ​ถาม​ผ่าน​เว็บ TUI หรือ SQL]
นั่นแหละ​คือ​เหตุผล​ที่​ดัชนี​เดียว​รอง​รับได้​ทุก​ทาง

ตอนนี้​เรา​อ่าน​ดัชนี​ได้​เอง​แล้ว แต่​เดือน​ที่​ปิดผนึก​แล้ว​นั้น​ตรวจสอบ​กัน​อย่างไร
และ​ส่ง​สำเนา​ไป​เก็บ​ไว้​ที่อื่น​ยังไง บท​ถัดไป​ว่าด้วย​สอง​เรื่อง​นี้

= บท​ที่ 7: เดือน​ที่​ปิดผนึก และ​สำเนา​ไป S3
<บทท-7-เดอนทปดผนก-และสำเนาไป-s3>
index ของ relic-scan แบ่ง​เป็น​ไฟล์ Parquet เดือน​ละ​หนึ่ง​ไฟล์ ซึ่ง​มี​จังหวะ​ชีวิต​ของ​มัน​เอง
เดือน​ปัจจุบัน​ยัง​เปิด​อยู่ ข้อมูล​ใหม่​ไหล​เข้ามา​ได้​ตลอด พอ​เดือน​นั้น​ผ่าน​ไป relic-scan ก็​ปิดผนึก​มัน
แล้ว​ไฟล์​จะ​กลายเป็น​ของ​ที่​แก้​ไม่​ได้ บท​นี้​ดู​ว่า​ผนึก​คือ​อะไร ตรวจ​อย่างไร​ว่า​ยัง​เหมือนเดิม
และ​ส่ง​เดือน​ที่​ปิด​แล้วไป​เก็บสำรอง​ที่ S3 อย่างไร

ทุกอย่าง​ใน​สอง​หัวข้อ​แร​กรัน​จริง​บน​บ้าน​เดโม ส่วน​หัวข้อ​ที่สาม​เรื่อง S3 บน​บ้าน​เดโม​ไม่​มี store
ให้​ส่ง คำสั่ง​ใน​หัวข้อ​นั้น​จึง​มาจาก AGENTS.md และ​ไม่​มี​ภาพ​ผลลัพธ์

== 7.1 เดือน​ที่​ปิด​แล้ว​เป็น​ไฟล์​อ่าน​อย่าง​เดียว
<เดอนทปดแลวเปนไฟลอานอยางเดยว>
เดือน​ที่ "ปิด" คือ​เดือน​ที่อยู่​ก่อน​เดือน​ปัจจุบัน​ตามเวลา​กรุงเทพ วันที่​รัน​บ้าน​เดโม​คือ 2026-10-26
เดือน 2026-07 ถึง 2026-09 จึง​ปิด​แล้ว ส่วน 2026-10 ยัง​เปิด พอ relic-scan
เห็น​ว่า​เดือน​ไหน​ปิด มัน​จะ​จด​ผนึก​ไว้ แล้ว​ตั้ง​ไฟล์​เดือน​นั้น​เป็น​อ่าน​อย่าง​เดียว

=== ขั้น​ที่ 1 — ดู​สถานะ​ผนึก​ของ​ทุก​เดือน
<ขนท-1-ดสถานะผนกของทกเดอน>
พิมพ์ `relic-scan seals` คำสั่ง​นี้​อ่าน​อย่าง​เดียว ไม่​เขียน​อะไร​เลย แต่ละ​บรรทัด​คือ​หนึ่ง​เดือน
บอก​จำนวน​แถว เวลา​ที่​เห็น​ว่า​ปิด จำนวน​ครั้ง​ที่​ปิด​ซ้ำ และ​สถานะ

```bash
relic-scan seals
```

#figure(image("/images/10-seals.png", width: 63%, alt: "ตาราง seals สี่​เดือน สาม​เดือน​แรก sealed และ 2026-10 เป็น open"),
  caption: [
    ตาราง seals สี่​เดือน สาม​เดือน​แรก sealed และ 2026-10 เป็น open
  ]
)

อ่าน​ตาราง​แล้ว​ได้​ว่า #strong[สาม​เดือน​แรก​เป็น sealed และ​เดือน 2026-10 ยัง​เป็น open]
แถว​ของ​สาม​เดือน​ที่​ปิด​คือ 819, 762 และ 845 แถว ส่วน​เดือน​ที่​เปิด​มี 747 แถว รวมกัน​ได้
3,173 แถว​พอดี ตรง​กับ​ตัวเลข​บน​หน้า Smart Scan คอลัมน์ `sealed at`
ของ​เดือน​ที่​เปิด​เป็น​ขีด `-` เพราะ​ยัง​ไม่​มี​เวลาปิด​ให้​จด

```
2026-07          819  2026-10-26T00:22:59Z        0  sealed
2026-10          747  -                           0  open
```

คอลัมน์ `resealed` เป็น 0 ทุก​เดือน ค่า​นี้​นับว่า​เดือน​ที่​ปิด​ไป​แล้ว​ต้อง​ปิด​ใหม่​กี่​ครั้ง
เพราะ​มี​แถว​ช้า​มาถึง​ทีหลัง เช่น เครื่อง​อื่น​เอา transcript เก่า​มา​สำรอง​ไว้​ทีหลัง
ใน​เดโม​ไม่​มี​เหตุ​แบบ​นั้น ค่า​จึง​เป็น​ศูนย์

ถ้า​อยากรู้​ว่า​คำสั่ง​นี้​รับ​ตัวเลือก​อะไร ถาม `--help`

```bash
relic-scan seals --help
```

#figure(image("/images/42-help-seals.png", width: 100%, alt: "ผล​ของ relic-scan seals --help: คำอธิบาย Usage และ​ตัวเลือก --out, --kind, --verify"),
  caption: [
    ผล​ของ relic-scan seals \-\-help: คำอธิบาย Usage และ​ตัวเลือก \-\-out,
    \\-\-kind, \-\-verify
  ]
)

มี​ตัวเลือก​ให้​ใช้ `--out`, `--kind` และ `--verify` ที่​อ่าน​ทุก​เดือน​ที่​ปิด​ซ้ำ​แล้ว​เทียบ​กับ​ผนึก
ซึ่ง​เป็นเรื่อง​ของ​ขั้น​ต่อไป

=== ขั้น​ที่ 2 — ตรวจ​ว่า​เดือน​ที่​ปิด​ยัง​เหมือน​ตอน​ปิด
<ขนท-2-ตรวจวาเดอนทปดยงเหมอนตอนปด>
ผนึก​มี​ไว้​ให้​ตรวจ ใส่ `--verify` แล้ว relic-scan จะ​อ่าน​ไฟล์​ของ​แต่ละ​เดือน​ที่​ปิด​ซ้ำ​อี​กรอบ
แล้ว​เทียบ​กับ​จำนวน​แถว​และ hash ของ​คีย์​แถว​ที่​จด​ไว้​ตอน​ปิด

```bash
relic-scan seals --verify
```

#figure(image("/images/11-seals-verify.png", width: 72%, alt: "ตาราง seals --verify สาม​เดือน​ที่​ปิด​ขึ้น sealed, matches"),
  caption: [
    ตาราง seals \-\-verify สาม​เดือน​ที่​ปิด​ขึ้น sealed, matches
  ]
)

สถานะ​ของ​สาม​เดือน​ที่​ปิด​เปลี่ยนเป็น `sealed, matches` แปล​ว่า
#strong[ไฟล์​วันนี้​ตรง​กับ​ไฟล์​ตอน​ปิด​ทุก​แถว] เดือน 2026-10 ยัง​ขึ้น​แค่ `open`
เพราะ​ไม่​มี​ผนึก​ให้​เทียบ ถ้า​ไฟล์​เดือน​ไหน​ถูก​แก้​หรือ​เสียหาย สถานะ​ตรงนี้​จะ​บอก
ซึ่ง​เป็น​เหตุผล​ที่​ควร​รัน `--verify` ก่อน​ส่ง​สำเนา​ไป​ไหน

=== ขั้น​ที่ 3 — ดู​ผนึก​ดิบ​ใน months.json
<ขนท-3-ดผนกดบใน-months.json>
ตาราง​ข้างบน​อ่าน​มาจาก​ไฟล์ `meta/months.json` ใช้ `jq` ดึง​เดือน​แรก​มา​ดู​ทั้ง​ก้อน​ได้
จะ​เห็น​ว่า​ผนึก​คือ​ข้อมูล​ธรรมดา​ไม่​กี่ field

```bash
jq -c '.months[0]' ~/.relic-scan/meta/months.json
```

ผลลัพธ์​ของ​คำสั่ง​นี้​เป็น​บรรทัด JSON เดียว ภาพ​เต็ม​อยู่​ใน​บท​ที่ 6 หัวข้อ 6.7 ที่นี่​อ่าน​เฉพาะ
field ของ​ผนึก ซึ่ง​เดือน 2026-07 ได้ `sealed_at` เป็น `2026-10-26T00:22:59Z`
เท่ากับ​ใน​ตาราง seals ข้างบน

สาม​คู่​ที่​ต้อง​มอง​คือ `sealed_at` เวลา​ที่​เห็น​ว่า​ปิด `sealed_rows` จำนวน​แถว​ตอนนั้น และ
`sealed_keys` hash ของ​คีย์​แถว​ตอนนั้น #strong[ถ้า `keys` กับ `sealed_keys`
เท่ากัน แถว​ก็​เท่ากัน] ใน JSON นั้น `keys` ของ​เดือน 2026-07 คือ
`deb9bcc29d4996bd545ea8297889d90d` และ `sealed_keys` เป็น​ค่า​เดียวกัน field
`late_rows` คือ​แถว​ที่​รอบ​นี้​เพิ่ม​เข้ามา​โดย​มี​เวลา​เก่า​กว่า​แถว​ใหม่​สุด​เดิม​ของ​เดือน ปกติ​เป็น 0
ถ้า​ไม่​เป็น 0 ก็​คือ​เดือน​ที่​ปิด​แล้ว​มี​ของ​แทรก​มา และ `resealed` จะ​ขยับ​ตาม

== 7.2 หน้า Export: metadata และ​สำเนา S3
<หนา-export-metadata-และสำเนา-s3>
พอ​รู้​แล้ว​ว่า​เดือน​ไหน​ปิด ก็​ถึง​เวลา​เอา​ออก​ไป​ข้างนอก​ได้ หน้า Export
ของ​เว็บ​เป็น​ที่รวม​ของ​ทั้งสอง​อย่าง คือ metadata ที่​ดึง​เป็น​ไฟล์​ได้​ทันที กับ​แผง​สำเนา S3
เปิด​หน้า​นี้​ด้วย `relic-scan up` หรือ `relic-scan serve` แล้ว​เลือก Export
ใน​เมนู​ซ้าย

=== ขั้น​ที่ 1 — ดึง metadata เป็น CSV หรือ JSON
<ขนท-1-ดง-metadata-เปน-csv-หรอ-json>
ตาราง​บนสุด​ของ​หน้า​คือ Metadata แต่ละ​บรรทัด​มี​ปุ่ม CSV กับ JSON ให้​ดึง​ไป​ใช้​ต่อ
เช่น​เอา​ไป​เปิด​ใน spreadsheet

#figure(image("/images/W18-page-export.png", width: 100%, alt: "หน้า Export ตาราง Metadata และ​แผง S3 copy ที่​ยัง​ไม่​ตั้ง​ปลายทาง"),
  caption: [
    หน้า Export ตาราง Metadata และ​แผง S3 copy ที่​ยัง​ไม่​ตั้ง​ปลายทาง
  ]
)

ตาราง​นี้​ให้ #strong[sessions 144 บรรทัด, files 145, homes 3, scans 4 และ ψ
vaults 2] ตัวเลข​ชุด​เดียว​กับ​ที่​เห็น​ใน​หน้า Tree, Paths และ Vaults ของ​บท​ก่อน ๆ
ข้อมูล​ที่​ออกมา​เป็น​แค่ metadata ของ index ไม่​ใช่​เนื้อ​ข้อ​ความใน transcript

=== ขั้น​ที่ 2 — อ่าน​แผง S3 copy
<ขนท-2-อานแผง-s3-copy>
ใต้​ตาราง คือ​แผง S3 copy ใน​ภาพ​มัน​บอ​กว่า​ยัง​ไม่​ได้​ตั้ง​ปลายทาง และ​ยัง​ไม่​มี​คีย์ ปุ่ม Check
against S3 กับ Send sealed months จึง​ใช้ไม่ได้ (ปุ่ม​ส่ง​จาง​อยู่) นี่​ตรง​กับ​ที่ AGENTS.md
เขียน​ไว้​ว่า #strong[คีย์​ไม่​ถูก​เก็บ​ใน​หน้า​นี้​เลย] หน้า​เว็บ​ตั้ง​ได้​แค่​ปลายทาง

เมื่อ​ตั้ง​ครบ​แล้ว Check against S3 จะ​เทียบ store กับ​ไฟล์​ใน​เครื่อง และ​แสดง​ทีละ​เดือน​ว่า
identical, older หรือ missing ส่วน Send sealed months จะ​ส่ง​เฉพาะ​เดือน​ที่​ปิด​แล้ว
คน​ที่​ส่ง​คำขอ​ไป S3 คือ server ไม่​ใช่​หน้า​เว็บ หน้า​ยังคง​ไม่​เรียก​เครือข่าย​เอง และ​การเขียน​ฝั่ง
server ต้อง​มี header `X-Relic` ตาม​กฎ​ของ​โค้ด

=== ขั้น​ที่ 3 — ตั้ง​ปลาย​ทางใน​หน้า Settings
<ขนท-3-ตงปลายทางในหนา-settings>
ปลายทาง​ตั้ง​ที่​หน้า Settings ใน​ส่วน S3 copy มี​สาม​ช่อง คือ destination, endpoint
และ region ใน AGENTS.md ค่า​เหล่านี้​ถูก​เก็บ​ใน​ไฟล์ `_config.json` ใน​ชื่อ `s3.to`,
`s3.endpoint` และ `s3.region`

#figure(image("/images/W21-page-settings.png", width: 100%, alt: "หน้า Settings ส่วน What to scan และ​ส่วน S3 copy"),
  caption: [
    หน้า Settings ส่วน What to scan และ​ส่วน S3 copy
  ]
)

ภาพ​นี้​ยัง​เห็น​ส่วน What to scan ด้วย (โฟลเดอร์​ที่​เดิน skip paths และ depth 8)
ซึ่ง​เป็นเรื่อง​ของ​บท​ที่ 4 ส่วน​ที่​เกี่ยวกับ​บท​นี้​คือ​ส่วนล่าง ช่อง endpoint ใช้​เมื่อ store ไม่​ใช่
AWS S3 จริง เช่น Garage หรือ MinIO ที่​ต้อง​มี endpoint แบบ path-style ถ้า​ไม่​ตั้ง
relic-scan ถือว่า​เป็น AWS S3

ส่วน​คีย์ AGENTS.md ให้​วาง​ไว้​สอง​ที่​เท่านั้น คือ​ใน environment (`AWS_ACCESS_KEY_ID`
กับ `AWS_SECRET_ACCESS_KEY`) หรือ​ใน​ไฟล์ `~/.config/relic-scan/s3.env`
เป็น​บรรทัด `KEY=VALUE` ที่ตั้ง​สิทธิ์ 600 ไฟล์​ที่​คนอื่น​อ่าน​ได้ relic-scan จะ​ปฏิเสธ​ไม่​ใช้
ไฟล์​นี้​คือ​ที่​ที่​แอ​ป​ซึ่ง​เปิด​จาก Finder ใช้​อ่าน​คีย์

== 7.3 ส่ง​ไป S3 จาก​บรรทัด​คำสั่ง
<สงไป-s3-จากบรรทดคำสง>
ใน​บ้าน​เดโม​ไม่​มี store ให้​ส่ง บล็อก​ใน​หัวข้อ​นี้​จึง​คัด​มาจาก AGENTS.md ตาม​ตัวอักษร
ไม่​ได้​รัน​ใน​การสาธิต และ​ไม่​มี​ภาพ​ผลลัพธ์​ให้​ดู ผลลัพธ์​จริง​บน​เครื่อง​ของ​คุณ​อาจ​ต่าง​ไป​จาก​ที่​คิด​ไว้
ให้​อ่าน​ผล​บน​หน้าจอ​ของ​ตัวเอง​เป็นหลัก

=== ขั้น​ที่ 1 — ตั้ง​ปลายทาง​และ​คีย์​ใน​สภาพแวดล้อม
<ขนท-1-ตงปลายทางและคยในสภาพแวดลอม>
ก่อน​ส่ง ตั้ง​ตัวแปร​สอง​สาม​ตัว​ใน shell #strong[คีย์​ห้าม​พิมพ์​ใน​บรรทัด​คำสั่ง]
เพราะ​จะ​ค้าง​อยู่​ใน history และ process list

```
export RELIC_S3_ENDPOINT=http://odin.follow-rankine.ts.net:3920
export AWS_ACCESS_KEY_ID=… AWS_SECRET_ACCESS_KEY=…
```

`RELIC_S3_ENDPOINT` ไม่ต้อง​ตั้ง​ถ้า​ใช้ AWS S3 จริง ส่วน `RELIC_S3_REGION`
มีค่า​เริ่มต้น us-east-1 ตัวอย่าง​ข้างบน​คือ Garage ที่​ตั้งอยู่​บน tailnet ตามที่ AGENTS.md
ใช้​อธิบาย และ #strong[http ธรรมดา​อนุญาต​เฉพาะ​เครื่อง​นี้​หรือ tailnet (`*.ts.net`,
`100.64/10`)] นอกนั้น​ต้อง​ใส่ `--insecure-http`

ตัวเลือก​ทั้งหมด​ของ `export` ดู​ได้​จาก `--help` (ผล​นี้​รัน​จริง ต่าง​จาก​บล็อก​ด้านล่าง)

```bash
relic-scan export --help
```

#figure(image("/images/41-help-export.png", width: 100%, alt: "ผล​ของ relic-scan export --help: ตัวเลือก --out, --kind, --to, --endpoint, --region, --dry-run, --overwrite, --verify, --insecure-http, --list"),
  caption: [
    ผล​ของ relic-scan export \-\-help: ตัวเลือก \-\-out, \-\-kind, \-\-to,
    \\-\-endpoint, \-\-region, \-\-dry-run, \-\-overwrite, \-\-verify,
    \\-\-insecure-http, \-\-list
  ]
)

ใน​ภาพ `--to` เป็น​ตัวเลือก​บังคับ (ใน​บรรทัด Usage) ส่วนที่เหลือ​ที่​ขั้น​ต่อ ๆ ไป​ใช้ คือ
`--dry-run`, `--verify`, `--list`, `--overwrite` และ `--insecure-http`
ส่วน `--endpoint` กับ `--region` ใช้​แทน​ตัวแปร `RELIC_S3_ENDPOINT` และ
`RELIC_S3_REGION`

=== ขั้น​ที่ 2 — ลอง​ก่อน​ด้วย \-\-dry-run
<ขนท-2-ลองกอนดวย---dry-run>
คำสั่ง​แรก​ที่​ควร​รัน​คือ `--dry-run` มัน#strong[ส่ง​แค่​คำขอ HEAD ไป​ถาม store]
แล้ว​บอ​กว่า​จะ​ส่ง​อะไร ไม่​เขียน​อะไร​ลง store

```
relic-scan export --to s3://bucket/relic --dry-run
```

ตัว​อย่างนี้​ไม่​ได้​รัน​ใน​เดโม เพราะ​ไม่​มี store ผล​ที่​ได้​คือ​รายการ​เดือน​ที่จะ​ถูก​ส่ง ถ้า store
มี​เดือน​นั้น​อยู่แล้ว​และ sha256 ตรง มัน​ก็​ไม่​ถูก​ส่ง​ซ้ำ

=== ขั้น​ที่ 3 — ส่ง​จริง
<ขนท-3-สงจรง>
พอ​พอใจ​กับ​รายการ ก็​ตัด​คำ​ว่า `--dry-run` ออก

```
relic-scan export --to s3://bucket/relic
```

คำสั่ง​นี้​ส่ง​เดือน​ที่​ปิด​แล้ว จากนั้น​ส่ง `meta/months.json` (ผนึก) และ
`export-manifest.json` แต่ละ object เก็บ sha256 ไว้​ใน `x-amz-meta-sha256`
จึง #strong[รัน​ซ้ำ​แล้ว​ไม่​ส่ง​ของ​ที่อยู่​ใน store แล้ว] เดือน​ที่​ยัง​เปิด​อยู่​ไม่​ถูก​ส่ง และ
relic-scan ไม่ list หรือ​ลบ​อะไร​ใน store เลย

=== ขั้น​ที่ 4 — ตรวจ​และ​ดู​รายการ
<ขนท-4-ตรวจและดรายการ>
หลัง​ส่ง ใช้ `--verify` #strong[เทียบ store กับ​ไฟล์​ใน​เครื่อง]
ถ้า​ต่างกัน​คำสั่ง​จะ​ออก​ด้วย​รหัส 1 หรือ​ใช้ `--list` ดู​ว่า store มี​อะไร​บ้าง

```
relic-scan export --to s3://bucket/relic --verify
relic-scan export --to s3://bucket/relic --list
```

`--list` พิมพ์​หนึ่ง​บรรทัด​ต่อ​เดือน มี​จำนวน​แถว วันที่ ขนาด sha256 และ​ผนึก
โดย​อ่าน​แค่​สอง​ไฟล์​เล็ก ไม่ list ทั้ง store

=== ขั้น​ที่ 5 — เมื่อ​เดือน​ที่​ปิด​แล้ว​ต้อง​ปิด​ใหม่
<ขนท-5-เมอเดอนทปดแลวตองปดใหม>
บางครั้ง​มี​แถว​ช้า​มาถึง​เดือน​ที่​ปิด​ไป​แล้ว ผนึก​จึง​เปลี่ยน (`resealed` เพิ่ม) เดือน​นั้น​ใน store
ก็​ต่าง​จาก​ไฟล์​ใน​เครื่อง ตอนนี้ #strong[export จะ​ปฏิเสธ​การเขียน​ทับ] เว้นแต่​ใส่
`--overwrite` ตามที่ AGENTS.md ระบุ

```
relic-scan export --to s3://bucket/relic --overwrite
```

scripts/README.md บอก​เรื่อง​เดียวกัน​ถึง​งาน​สำรอง​ตอนกลางคืน
`relic-export-nightly` ที่ timer รัน มัน​ไม่​ทับ​เดือน​ที่​ปิด​แล้ว ถ้า​อยาก​ทับ​ต้อง​รัน
`--overwrite` ด้วยมือ นี่​เป็นการ​ออกแบบ​ที่​ตั้งใจ
เพื่อ​ไม่​ให้​งาน​อัตโนมัติ​เขียน​ทับ​สำเนา​เก่า​ที่​ยัง​ดี​อยู่​โดย​ไม่​มี​ใคร​เห็น

เรื่อง​สำเนา​ไป​ข้างนอก​จบ​แค่นี้ บท​ต่อ​ไปดู​ทางเดิน​ทั้ง​เส้น​ของ​ข้อมูล ตั้งแต่ relic-scan ไป​ถึง
hub และ​ดู​ว่า `SYNC OK` บอก​อะไร

= บท​ที่ 8: ทางเดิน​ของ​ข้อมูล​ข้าม​เครื่อง
<บทท-8-ทางเดนของขอมลขามเครอง>
relic-scan ไม่​ได้​ทำงาน​โดดเดี่ยว บน​เครื่อง​ของ​ทีม​เขา มัน​เป็น​ขั้นแรก​ของ chain
สาม​ขั้น​ที่​ส่ง​ข้อมูล​ต่อกัน​ไป​จนถึง hub ที่รวม​ประวัติ​จาก​หลาย​เครื่อง บท​นี้​เปิด​ดู​ว่า chain
ทำ​อะไร​อยู่​ตอนนี้​ด้วย `relic-scan ops` หน้า Activity และ​แท็บ ops ใน​เทอร์มินัล
แล้ว​อธิบาย​ว่า​แต่ละ​ขั้น​คือ​อะไร และ​เส้น​สุดท้าย `SYNC OK` ยืนยัน​อะไร

ตัวเลข​ทุก​ตัว​ใน​บท​นี้​มาจาก​การ​รัน​บน​บ้าน​เดโม​ใน​เครื่อง m5 ของ​ทีม ซึ่ง​มี chain รัน​อยู่​จริง
เนื้อ​หาเรื่อง relic-extract และ hub มาจาก docs/TUTORIAL.md กับ docs/TALK.md
ไม่​มี​ตัวเลข​เพิ่มเติม​นอกจาก​ที่​เห็น​ใน​ภาพ

== 8.1 relic-scan ops: ดู chain ทั้ง​เส้น
<relic-scan-ops-ด-chain-ทงเสน>
`relic-scan ops` แสดงว่า chain กำลัง​ทำ​อะไร ขั้น​ไหน​ใช้เวลา​เท่าไร มี process
อะไร​รัน​อยู่ และ hub อยู่​ใน​สถานะ​ไหน มัน​เป็น​คำสั่ง​อ่าน​อย่าง​เดียว​เหมือน `seals`

=== ขั้น​ที่ 1 — รัน ops โดย​ไม่​แตะ hub
<ขนท-1-รน-ops-โดยไมแตะ-hub>
ปกติ ops อ่าน​สถานะ​ของ hub ด้วย​คำขอ GET หนึ่ง​ครั้ง
ซึ่ง​เป็นการ​เรียก​เครือข่าย​อย่าง​เดียว​ของ​คำสั่ง​นี้ ถ้า​ไม่​อยาก​ให้​ออก​ไป​เลย ใส่ `--no-hub`
ตัวแปร `RELIC_OPS_HUB=off` ให้ผล​เหมือนกัน

```bash
relic-scan ops --no-hub
```

#figure(image("/images/20-ops.png", width: 99%, alt: "ผล relic-scan ops ส่วน CHAIN ห้า​ขั้น​พร้อม​เวลา, PROCESSES none running และ HUB not read"),
  caption: [
    ผล relic-scan ops ส่วน CHAIN ห้า​ขั้น​พร้อม​เวลา, PROCESSES none running และ
    HUB not read
  ]
)

บรรทัด​แรก​ของ chain บอก​เครื่อง สถานะ และ​รอบ​ล่าสุด

```
CHAIN m5  idle  last pass 06:50:15 took 1:31 · next in about 0:00
```

อ่าน​ได้​ว่า #strong[chain บน​เครื่อง m5 ว่าง​อยู่ รอบ​ล่าสุด​เริ่ม 06:50:15 ใช้เวลา 1:31]
`idle` แปล​ว่า​ไม่​มี​ขั้น​ไหน​กำลัง​ทำงาน​ตอนนี้ ส่วน `HUB  not read (off)`
ตรง​บรรทัด​ล่าง​เป็นผล​ของ `--no-hub` ไม่​ใช่ hub ล่ม

=== ขั้น​ที่ 2 — อ่าน​ห้า​ขั้น​ทีละ​บรรทัด
<ขนท-2-อานหาขนทละบรรทด>
ใต้​บรรทัด​หัว​มี​ห้า​ขั้น แต่ละ​ขั้น​มี​เครื่องหมาย​ถูก เวลา​ที่​ใช้​ครั้งนี้ ตาม​ด้วย​เวลา​ปกติ (`typical`)
ใน​วงเล็บ แล้ว​เป็น​รายละเอียด​ด้าน​ขวา

```
✓ scan                  31 s (typical 31 s)   scan 0.2s · new 0 · grown 3 …
✓ extract               14 s (typical 14 s)     337 chunks · 12 messages …
✓ push to the hub        5 s (typical 5 s)
✓ hub index + embed
✓ sync check                                    SYNC OK  text|1256 …
```

เวลา #strong[ครั้งนี้​เท่ากับ​เวลา​ปกติ​ทุก​ขั้น] แปล​ว่า​รอบ​นี้​ไม่​ช้า​กว่า​เคย ถ้า​ขั้น​ไหน​ช้า​ผิดปกติ
ตัวเลข​สอง​ตัว​นี้​ต่างกัน​ทันที ให้​ดู​ขั้น scan ที่​รายละเอียด​เป็น​สรุป​ของ `relic-scan run`
ที่​เรา​คุ้น​จาก​บท​ที่ 2 คือ new 0, grown 3, rewritten 0, unchanged 145, gone 0
เปิด​ไฟล์​ที่​ต้อง​อ่าน​เพิ่ม 0.00 GB นั่น​คือ​รอบ​นี้ transcript โต​แค่ 3 ไฟล์

=== ขั้น​ที่ 3 — ขั้น​ที่​เหลือ​และ process
<ขนท-3-ขนทเหลอและ-process>
ขั้น extract รายงาน​ว่า#strong[ตัด​ได้ 337 chunks] และ​ตัด​ข้อความ​ออกมา​ได้ 337
chunks และ​กรอง​ทิ้ง 12 messages ขั้น push to the hub ส่งผล​ไป​ที่ hub ขั้น hub index
\+ embed ไม่​แสดง​รายละเอียด​ฝั่ง​นี้​เพราะ​งาน​เกิด​บน hub เอง ส่วน​บรรทัด​ที่สี่
`PROCESSES  none running` แปล​ว่า​ขณะที่​รัน​คำสั่ง ไม่​มี process ของ chain ค้าง​อยู่

ท้าย​เอาต์พุต​มี​บรรทัด `page  http://127.0.0.1:8787/ops` ที่​ชี้​ไป​หน้า Activity
ถ้า​เปิด​บน terminal ที่​รองรับ ลิงก์​ทั้งสอง​คลิก​ได้ ตัวแปร `RELIC_LINKS=off` หรือ
`NO_COLOR` ปิด​ลิงก์ และ `RELIC_PAGE_URL` ย้าย​ที่อยู่​ของ​หน้า ใส่ `--watch`
เมื่อ​อยาก​ให้​จอ​รีเฟรช​ต่อเนื่อง และ `--json` เมื่อ​ต้องการ​ให้​เครื่องมือ​อื่น​อ่าน

=== ขั้น​ที่ 4 — ดู​ตัวเลือก​ของ ops
<ขนท-4-ดตวเลอกของ-ops>
อยากรู้​ว่า ops รับ​อะไร​ได้​บ้าง ให้​ถาม `--help` ตัวเลือก #strong[`--no-hub`
คือ​ทางเดียว​ที่​ทำให้​คำสั่ง​นี้​ไม่​ใช้​เครือข่าย​เลย]

```bash
relic-scan ops --help
```

#figure(image("/images/49-help-ops.png", width: 100%, alt: "ผล relic-scan ops --help แสดง Usage และ​ตัวเลือก --out, --kind, --json, --watch, --no-hub"),
  caption: [
    ผล relic-scan ops \-\-help แสดง Usage และ​ตัวเลือก \-\-out, \-\-kind, \-\-json,
    \\-\-watch, \-\-no-hub
  ]
)

มี​ตัวเลือก​หก​ตัว​รวม `--help` ตัว​ที่​ใช้​บ่อย​คือ `--watch` ที่​วาด​จอ​ใหม่​ทุก 2 วินาที​จน​กด
Ctrl-C และ `--json` ที่​พิมพ์​สแนป​ช็อต​เป็น JSON หนึ่ง​ครั้ง​ให้หน้า​เว็บ​กับ​แอ​ป​เมนู​บาร์​อ่าน ส่วน
`--out` กับ `--kind` ชี้​ไป​โฟลเดอร์ index ที่ run ใช้

อยาก​เทียบ​ผล ให้​รัน​แบบ​ไม่​แตะ hub อีกครั้ง

```bash
relic-scan ops --no-hub
```

#figure(image("/images/20-ops.png", width: 99%, alt: "ผล relic-scan ops --no-hub ซ้ำ​จาก​ขั้น​ที่ 1 ห้า​ขั้น​ของ chain และ SYNC OK"),
  caption: [
    ผล relic-scan ops \-\-no-hub ซ้ำ​จาก​ขั้น​ที่ 1 ห้า​ขั้น​ของ chain และ SYNC OK
  ]
)

ภาพ​นี้​ซ้ำ​จาก​ขั้น​ที่ 1 เพื่อให้​เทียบ​กับ `--no-hub` ตรงหน้า: ได้​เอาต์พุต​ห้า​ขั้น​เดิม ไม่​มี HUB
ที่​อ่าน​จาก​เครือข่าย

== 8.2 หน้า Activity
<หนา-activity>
หน้า Activity แสดง​ข้อมูล​ชุด​เดียวกัน​บน​เว็บ เปิด​จาก `relic-scan up` แล้ว​เลือก
Activity ใน​เมนู​ซ้าย ที่อยู่​คือ `/ops` ตาม​บรรทัด page ข้างบน

=== ขั้น​ที่ 1 — เปิด​หน้า Activity
<ขนท-1-เปดหนา-activity>
หน้า​นี้​แบ่ง​เป็น​สาม​การ์ด คือ Relic chain, Processes และ Hub
หน้า​เว็บ​เอง​ไม่​เรียก​เครือข่าย ส่วน​การ์ด Hub อ่าน​สถานะ​ด้วย GET หนึ่ง​ครั้ง​ตามที่ AGENTS.md
ระบุ

#figure(image("/images/W19-page-ops.png", width: 100%, alt: "หน้า Activity การ์ด Relic chain ห้า​ขั้น​พร้อม​เวลา, Processes ไม่​มี, Hub ไม่​ได้​อ่าน"),
  caption: [
    หน้า Activity การ์ด Relic chain ห้า​ขั้น​พร้อม​เวลา, Processes ไม่​มี, Hub
    ไม่​ได้​อ่าน
  ]
)

ภาพ​นี้​บอก​เรื่อง​เดียว​กับ `ops` #strong[การ์ด Relic chain มี​ห้า​ขั้น​เรียง​จาก scan จนถึง
sync check] การ์ด Processes ว่างเปล่า และ​การ์ด Hub ขึ้น​ว่าไม่ได้​อ่าน เพราะ​ปิด​ไว้
ข้อดี​ของ​หน้า​นี้​คือ​เปิด​ทิ้ง​ไว้​ได้​โดย​ไม่ต้อง​พิมพ์​ซ้ำ และ​เห็น​รอบ​ใหม่​เข้ามา​เอง

== 8.3 จาก relic-scan ไป​ถึง hub
<จาก-relic-scan-ไปถง-hub>
chain สาม​ขั้น​ตามที่ docs/TUTORIAL.md เขียน​ไว้ แต่ละ​ขั้น​อ่าน​ผล​ของ​ขั้น​ก่อน​โดย​ไม่​เคย​แก้​มัน

```
 relic-scan                 relic-extract                      relic4 embed
 find + index       ──►  pick, clean, dedupe, cut  ──►  vectors, search
 ~/.relic-scan/           ~/.relic-extract/<stream>/
```

#figure(image("/images/D1-pipeline.png", width: 92%, alt: "แผนภาพ WHERE THE DATA GOES: จาก agent homes ผ่าน relic-scan run ห้า​ขั้น ไป​ยัง ~/.relic-scan แล้ว​ออก​ทาง​หน้า​เว็บ, tui, SQL และ S3"),
  caption: [
    แผนภาพ WHERE THE DATA GOES: จาก agent homes ผ่าน relic-scan run ห้า​ขั้น
    ไป​ยัง \~/.relic-scan แล้ว​ออก​ทาง​หน้า​เว็บ, tui, SQL และ S3
  ]
)

ภาพ​นี้​เห็น​สอง​อย่าง​ที่​ร้อยแก้ว​ไม่​ได้​บอก คือ #strong[โฟลเดอร์​ต้นทาง​ไม่​ถูก​แก้​เลย]
และ​ไฟล์​เดือน​ที่​ปิด​แล้ว​เป็น `r--` มี​แต่​เดือน​ปัจจุบัน `rw-` ตัวเลข​บน​หัว​ภาพ​คือ 145 ไฟล์​ที่​เห็น,
144 ที่​ทำ index, 3,173 แถว, 4 เดือน และ 8 repo

ผัง​นี้​คัด​มาจาก​บทนำ​ของ TUTORIAL.md คำสั่ง​ต่อไปนี้​ไม่​ได้​รัน​ใน​การสาธิต เพราะ
relic-extract กับ agent-relic-v4 เป็น repo ส่วนตัว​ใน​ตอนนี้
ผู้อ่าน​ที่​ไม่​มีสิทธิ์​เข้า​จึง​รัน​ขั้น​สอง​และ​สาม​เอง​ไม่​ได้ ส่วน relic-scan เป็น​ขั้น​เดียว​ที่​เปิด​สาธารณะ

=== ขั้น​ที่ 1 — relic-scan หา​และ​ทำ index
<ขนท-1-relic-scan-หาและทำ-index>
ขั้นแรก​คือ​สิ่ง​ที่​เล่ม​นี้​เล่า​มา​ตลอด คือ​หา transcript ของ Claude Code, Codex, OMP,
OMX, Gemini, Pi และ thClaws บน​เครื่อง พร้อม ψ vault ทุกแห่ง แล้ว​ทำเป็น Parquet
รายเดือน​ใน `~/.relic-scan` และ​เพราะ ledger ทำให้
#strong[สำเนา​ที่​มาจาก​เครื่อง​อื่น​เสีย​แค่​ค่า hash ไม่ต้อง​เสีย​ค่า parse] รอบ​ประจำวัน​ของ
chain จึง​เร็ว

=== ขั้น​ที่ 2 — relic-extract ตัด​เป็น chunk
<ขนท-2-relic-extract-ตดเปน-chunk>
ขั้น​ที่สอง​อ่าน​ไฟล์​เดือน​ของ relic-scan แล้ว#strong[ตัด​เป็นหนึ่ง​แถว​ต่อ​หนึ่ง chunk]
แล้ว​เขียน​ส่วน​ที่​ต้อง​เอา​ไป embed ออกมา​เป็นหนึ่ง​แถว​ต่อ​หนึ่ง chunk ไฟล์ TOML
เดียว​กำหนด​ว่า​จะ​เอา​อะไร​เข้า และ​เครื่องมือ​จะ​ทำความสะอาด ตัด​ข้อความ​ซ้ำ และ​ตัด​ชิ้น
เอาต์พุต​อยู่​ใน `~/.relic-extract/<stream>/YYYY-MM.parquet` มี​สาย text,
thinking และ tools

บรรทัด `337 chunks · 12 messages cleaned out` ใน​ภาพ 20-ops
คือ​ผล​ของ​ขั้น​นี้​ใน​รอบ​ล่าสุด และ​ตัวเลข text, thinking, tools ที่​ปลาย​บรรทัด SYNC OK
ก็​เป็น​สาย​เดียวกัน​นี้

=== ขั้น​ที่ 3 — push แล้ว hub ทำ index และ embed
<ขนท-3-push-แลว-hub-ทำ-index-และ-embed>
ขั้น push #strong[ส่งผล​ไป​ที่ hub] (repo `relic5` พอร์ต 8795 บน​เครื่อง white ตาม
TALK.md) ขั้น hub index + embed รัน​บน hub เอง ใช้ relic4 ที่ embed เฉพาะ key
ที่​ยัง​ไม่​มี vector ข้อความ​เดียวกัน​จึง​ได้ vector เดียวกัน​ไม่​ว่า​อยู่​เดือน​ไหน

TALK.md เล่า​ด้วยว่า​สาม​ส่วน​นี้​มี agent ประจำ repo ของ​ตัวเอง relic4-scan ดูแล index
relic5 ดูแล hub และ relic-extract ดูแล​ข้อความ​ที่​ตัด​ออกมา ทุกฝ่าย​อยู่​ใน repo
ของ​ตัวเอง​และ​ขอ​การเปลี่ยนแปลง​จากกัน ข้อ​ควร​จำจาก​เอกสาร​นั้น​มี​ข้อ​เดียว​ที่​เกี่ยวกับ​ผู้ใช้​ปกติ คือ
#strong[ถ้า​ตัวเลข​บน hub ดู​ผิด ให้​เช็ก index ของ​เครื่อง​เอง​ก่อน] ด้วย DuckDB บน
`~/.relic-scan/meta/sessions.parquet` ถ้า​เครื่อง​ถูก บั๊ก​อยู่​ที่​การ​รวม​ข้อมูล​บน hub

== 8.4 ตรวจ​ว่า sync ตรงกัน
<ตรวจวา-sync-ตรงกน>
บรรทัด​สุดท้าย​ของ chain คือ​การ​ยืนยัน​ว่า​สิ่ง​ที่​เครื่องส่ง กับ​สิ่ง​ที่ hub ถือ​อยู่ ตรงกัน

=== ขั้น​ที่ 1 — อ่าน​บรรทัด SYNC OK
<ขนท-1-อานบรรทด-sync-ok>
ใน​ภาพ 20-ops ขั้น sync check ขึ้น​ว่า

```
SYNC OK  text|1256 thinking|507 tools|1410
```

สาม​ตัวเลข​นี้​เป็น​สาย text, thinking และ tools ตามลำดับ และ​บวก​กัน​ได้ 1256 + 507 +
1410 = 3,173 เท่ากับ​จำนวน​แถว​บน​หน้า Smart Scan ของ​บ้าน​เดโม TALK.md ระบุ​ว่า
#strong[ทุ​กรอบ​ของ chain จบ​ด้วย​บรรทัด SYNC OK แบบนี้] ที่ m5 และ mba ถูก​เขียน​ไว้​ใน
`~/Library/Logs/relic-chain.log` ส่วน​บน white อยู่​ที่
`~/.local/state/relic-chain.log` และ `/api/status` ของ hub
(`freshness.machines.*.sync`) ต้อง​แสดง​เลข​สาม​ตัว​เดียวกัน

=== ขั้น​ที่ 2 — ดู​แท็บ ops ใน​เทอร์มินัล
<ขนท-2-ดแทบ-ops-ในเทอรมนล>
แท็บ​ที่หก​ของ `relic-scan tui` คือ ops ใช้​ข้อมูล​เดียวกัน รีเฟรช​ทุก 2 วินาที และ​มี
process ที่​รัน​อยู่​ให้​ดู​เพิ่ม

```bash
relic-scan tui
```

ภาพ​ของ​แท็บ​นี้​อยู่​ใน​บท​ที่ 5 หัวข้อ 5.4 ห้า​ขั้น​ของ chain ที่​เห็น​ตรง​กับ `relic-scan ops`
ทุก​บรรทัด

ใน​กรอบ ops #strong[ส่วน `PROCESSES` ขึ้น​ว่า `none running`] และ
`HUB  not read (off)` เหมือนกับ `relic-scan ops --no-hub` ตัวเลข SYNC OK
ใน​จอ​นี้ (`text|1256 thinking|507 tools|1410`) ตรง​กับ​ที่​เห็น​ใน `relic-scan ops`
ทุก​ตัว เพราะ​อ่าน​จา​กรอบ​เดียวกัน

=== ขั้น​ที่ 3 — เมื่อ​เลข​สอง​ฝั่ง​ไม่​ตรงกัน
<ขนท-3-เมอเลขสองฝงไมตรงกน>
TALK.md ให้​เกณฑ์​ไว้​ชัด #strong[เลข​ไม่​ตรง​ที่​ค้าง​เกิน​สอง​รอบ]คือ​เรื่อง​ที่​ควร​แจ้ง​ทีม relic5
ส่วน​คำเตือน `relic-scan differs between machines` ที่​ขึ้น​หลัง​ติดตั้ง​เวอร์ชัน​ใหม่
จะ​หาย​เอง​เมื่อ​แต่ละ​เครื่อง push ด้วย​ไฟล์​ใหม่​แล้ว

อีก​ข้อ​ที่​ทีม​เจอ​จริง​คือ​การ​เปลี่ยน​ไฟล์ binary ที่​กำลัง​รัน​อยู่ TALK.md แนะ​ให้ `rm -f` แล้ว​ค่อย
`cp` และ​ทำ​หลังจาก build ตัวใหม่​เสร็จ​เท่านั้น เพราะ​บน​เครื่อง white เคย​มี​ครั้ง​ที่ `cp`
ล้ม​หลัง `rm` แล้ว relic-scan หาย​ไป​หก​นาที ดู​รายละเอียด​เรื่อง​นี้​อีกครั้ง​ใน​หัวข้อ 9.4

ถึง​ตรงนี้​ได้​เห็น​แล้ว​ว่า​ข้อมูล​เดิน​ไป​ไหน​บ้าง
บท​สุดท้าย​ของ​ภาค​นี้​ตอบคำถาม​ที่​ควร​ถาม​ก่อน​รัน​คำสั่ง​แรก คือ relic-scan แตะ​อะไร​บน​เครื่อง​คุณ

= บท​ที่ 9: ความเป็นส่วนตัว และ​ข้อควรระวัง
<บทท-9-ความเปนสวนตว-และขอควรระวง>
transcript ของ​เอ​เจนต์​คือ​บันทึก​งาน​จริง​ของ​คุณ มี​คำสั่ง​ที่​พิมพ์ ไฟล์​ที่​เปิด
และ​บางที​ก็​มี​คีย์​ที่​เผลอ​วาง​ลง​ไป เครื่องมือ​ที่​ไล่​อ่าน​ทุก​ไฟล์​พวก​นี้​จึง​ต้อง​ตอบ​ให้ได้​ว่า มัน​อ่าน​อะไร
เขียน​อะไร และ​ส่ง​อะไร​ออก​ไป​ข้างนอก​หรือเปล่า
บท​นี้​ตอบคำถาม​เหล่านั้น​ด้วย​กฎ​ใน​โค้ด​และ​ของ​ที่​เห็น​ใน​เดโม
แล้ว​ปิด​ด้วย​ปัญหา​ที่​ทีม​เจอ​จริง​จาก​เอกสาร​ของ repo

หมายเหตุ​เรื่อง​ข้อมูล บ้าน​เดโม​ของ​เล่ม​นี้​เป็น​ของ​สมมติ​ทั้งหมด สร้าง​โดย
`make-demo-home.py` มี transcript 144 ไฟล์ และ​มี​ความลับ​ปลอม​อยู่​เพียง​อัน​เดียว คือ​คีย์
`sk-demo-…` ใน​หนึ่ง prompt เพื่อ​ใช้​แสดง​หัวข้อ 9.2

== 9.1 อ่าน​อย่าง​เดียว ไม่​แก้ ไม่​เรียก​เน็ต
<อานอยางเดยว-ไมแก-ไมเรยกเนต>
กฎ​ของ​โค้ด​ใน AGENTS.md มี​สี่​ข้อ​ที่​เกี่ยวกับ​ความเป็นส่วนตัว​โดยตรง หนึ่ง transcript และ
vault เป็น​อ่าน​อย่าง​เดียว สอง relic-scan เขียน​เฉพาะ​ใน​โฟลเดอร์​ผลลัพธ์​ของ​ตัวเอง คือ
`~/.relic-scan` สาม หน้า​เว็บ​ไม่​เรียก​เครือข่าย​และ​ไม่​โหลด library จาก​ข้างนอก สี่
การเขียน​ฝั่ง server ต้อง​มี header `X-Relic`

=== ขั้น​ที่ 1 — ดูก่อน​ว่า​จะ​อ่าน​อะไร​ด้วย \-\-dry-run
<ขนท-1-ดกอนวาจะอานอะไรดวย---dry-run>
ก่อน​รัน​ครั้งแรก AGENTS.md ให้​ถาม​เจ้าของ​เครื่อง​ก่อน เพราะ​คำสั่ง `run` อ่าน​ทุก
transcript ใต้​โฟลเดอร์​บ้าน ทาง​ที่​ปลอดภัย​คือ​รัน `--dry-run` ก่อน
มัน​สแกน​แล้ว​บอ​กว่า​จะ​อ่าน​อะไร โดย​ไม่​เขียน​อะไร​เลย

```bash
relic-scan run --dry-run
```

#figure(image("/images/04-dry-run.png", width: 93%, alt: "กล่อง RELIC SCAN ผล dry run: would read 144 files, new 144, nothing written, ปลายทาง ~/.relic-scan"),
  caption: [
    กล่อง RELIC SCAN ผล dry run: would read 144 files, new 144, nothing
    written, ปลายทาง \~/.relic-scan
  ]
)

ภาพ​นี้​ซ้ำ​จาก​บท​ที่ 1 อ่าน​แล้ว​เห็น​ว่า new 144 ส่วน grown, rewritten, unchanged และ
gone เป็น 0 หมด เพราะ​ยัง​ไม่​เคย​ทำ index ตัวเลข 0.00 GB คือ​ขนาด​ที่จะ​อ่าน
ซึ่ง​เล็ก​มาก​ใน​บ้าน​เดโม

กล่อง​ใน​ภาพ​มี​สอง​บรรทัด​ที่​เป็น​หัวใจ

```
│ dry run · an index would read 144 files, 0.00 GB       │
│ scan 0.2s · nothing written                            │
```

#strong[`nothing written` คือ​คำยืนยัน​ว่าการ​ดูก่อน​ไม่​แตะ​ดิสก์] ส่วน 144
ไฟล์​คือ​จำนวน​ที่​สแกน​เห็น​และ​จะ​อ่าน​ถ้า​รัน​จริง ปลายทาง​ที่​ลูกศร​ชี้​คือ `~/.relic-scan` ที่​เดียว

=== ขั้น​ที่ 2 — เมื่อไหร่​ที่​ข้อมูล​ออกจาก​เครื่อง
<ขนท-2-เมอไหรทขอมลออกจากเครอง>
ข้อมูล​ออกจาก​เครื่อง​ได้#strong[สอง​ทาง​เท่านั้น]ตาม AGENTS.md ทาง​แรก​คือ
`relic-scan export --to s3://…` ซึ่ง​คุณ​สั่ง​เอง ทาง​ที่สอง​คือ `relic-scan ops`
กับ​มุมมอง Activity ที่​อ่าน​สถานะ​ของ hub ด้วย GET หนึ่ง​ครั้ง ตัวแปร
`RELIC_OPS_HUB=off` หรือ `--no-hub` ข้ามขั้น​นี้ ใน​บท​ที่ 8 เรา​ใช้ `--no-hub` ตลอด

นอกจาก​สอง​ทาง​นี้ ไม่​มี​การ​เรียก​เครือข่าย ถ้า​อยาก​พิสูจน์​เอง ก็​ดู​ได้​ว่า​หน้า​เว็บ​ให้บริการ​ที่
`127.0.0.1` เท่านั้น ไม่​เปิด​ให้​เครื่อง​อื่น และ​ทาง S3 ก็​ไม่​เคย​เก็บ​คีย์​ไว้​ใน​ไฟล์​ตั้งค่า (หัวข้อ
7.2)

=== ขั้น​ที่ 3 — ตรวจ​ว่า​สำเนา​ที่​ปิดผนึก​ยัง​ไม่​ถูก​แตะ
<ขนท-3-ตรวจวาสำเนาทปดผนกยงไมถกแตะ>
ความเป็นส่วนตัว​มี​อีก​ด้าน​คือ​ความถูกต้อง ข้อมูล​ที่​เปลี่ยนไป​โดย​ไม่​มี​ใคร​รู้​ก็​อันตราย
`seals --verify` อ่าน​เดือน​ที่​ปิด​ซ้ำ​และ​เทียบ​กับ​ผนึก

```bash
relic-scan seals --verify
```

#figure(image("/images/11-seals-verify.png", width: 72%, alt: "ผล relic-scan seals --verify ตาราง​สี่​เดือน สาม​เดือน​แรก sealed, matches เดือน 2026-10 เป็น open"),
  caption: [
    ผล relic-scan seals \-\-verify ตาราง​สี่​เดือน สาม​เดือน​แรก sealed, matches
    เดือน 2026-10 เป็น open
  ]
)

ภาพ​นี้​ซ้ำ​จาก​บท​ที่ 7 ตาราง​มี​สี่​เดือน แถว​ละ​หนึ่ง​เดือน ได้แก่ 819, 762, 845 และ 747 แถว
เดือน​สุดท้าย​เป็น `open` เพราะ​ยัง​ไม่​ปิด

เดือน 2026-07, 2026-08 และ 2026-09 ขึ้น `sealed, matches` ครบ
ภาพ​นี้​เป็น​ภาพ​เดียว​กับ​ที่​ดู​ใน​บท​ที่ 7 แต่​ที่นี่​ใช้​ใน​มุม​ต่าง​ออก​ไป คือ​เป็น​หลักฐาน​ว่า​ก่อน​ส่ง​สำเนา​ไป
S3 ไฟล์​ที่​ส่ง​ตรง​กับ​ที่​ปิด​ไว้​ทุก​แถว

== 9.2 redact: ความลับ​ถูก​แทน​ด้วย​ป้าย
<redact-ความลบถกแทนดวยปาย>
ใน​แถว​ของ index ที่​ข้อความ​มี​คีย์​หลุด​เข้ามา relic-scan ไม่​ทิ้ง​คีย์​ไว้​ตามเดิม README บอก​สั้น
ๆ ว่า​ขั้น​อ่าน​ต่อ​บรรทัด​ทำ "redaction" ไว้​พร้อม text, thinking และ tools
ผล​ของ​มัน​เห็น​ได้​ใน​คอลัมน์ `red` ของ Parquet

=== ขั้น​ที่ 1 — ถามหา​แถว​ที่​ถูก redact
<ขนท-1-ถามหาแถวทถก-redact>
แถว​ที่​ไม่​ถูก redact มี `red` เป็น NULL ดังนั้น​การกรอง​ด้วย `red is not null`
ได้​เฉพาะ​แถว​ที่​มี​ของลับ​ถูก​แทนที่

```bash
duckdb -c "select left(text, 90) as text, red from '*.parquet' \
  where red is not null"
```

#figure(image("/images/19-duck-redacted.png", width: 100%, alt: "ผล duckdb แถว​เดียว​ที่ red ไม่​ว่าง ข้อความ​มี [openai-key:sk-d…36] และ​คอลัมน์ red เป็น openai-key"),
  caption: [
    ผล duckdb แถว​เดียว​ที่ red ไม่​ว่าง ข้อความ​มี \[openai-key:sk-d…36\] และ​คอลัมน์
    red เป็น openai-key
  ]
)

ภาพ​นี้​ซ้ำ​จาก​บท​ที่ 6 ตาราง​มี​แถว​เดียว ข้อ​ความถูก​ตัด​ที่ 90 ตัวอักษร
และ​คีย์​ปลอม​ใน​นั้น​กลายเป็น​ป้าย​แล้ว

ทั้ง​บ้าน​เดโม​มี​แถว​เดียว​ที่​ถูก redact ซึ่ง​เป็น prompt ที่​ใส่​คีย์​ปลอม​ไว้ ใน​คอลัมน์​ข้อความ
คีย์​ไม่​ปรากฏ​เต็ม

```
(the key is [openai-key:sk-d…36], do not …   openai-key
```

อ่าน​ได้​ว่า #strong[คีย์​ถูก​แทน​ด้วย​ป้าย​ชนิด `[openai-key:…]`] ใน​ป้าย​เหลือ​แค่​ส่วน​ต้น
`sk-d…` กับ​เลขท้าย 36 เอกสาร​ไม่​ได้​อธิบาย​รูปแบบ​ป้าย​ละเอียด​กว่า​นี้ จึง​ไม่​ตีความ​เกิน​ภาพ
และ​คอลัมน์ `red` ระบุ​ชนิด​ว่า `openai-key`

=== ขั้น​ที่ 2 — อ่าน​ผล​อย่าง​ระวัง
<ขนท-2-อานผลอยางระวง>
การ redact ช่วย​ให้ index ไม่​ถือ​คีย์​เต็ม​ไว้ แต่​มัน​เป็น​การแทนที่​ใน​แถว​ของ index เท่านั้น
#strong[ไฟล์ transcript ต้นฉบับ​ไม่​ถูก​แก้] เพราะ​กฎ "อ่าน​อย่าง​เดียว"
ดังนั้น​คีย์​เต็ม​ยังอยู่​ใน `.jsonl` ของ​เอ​เจนต์​ตามเดิม ถ้า​คีย์​หลุด​เข้าไป​จริง ต้อง​หมุน​คีย์​ที่​ต้นทาง
ไม่​ใช่​หวัง​ว่า index จะ​แก้​ให้

AGENTS.md ยัง​เตือน​อีก​ข้อ​สำหรับ​ทุกคน​ที่​ถาม index ว่า transcript เก็บงาน​ส่วนตัว
ให้​อ่าน​เฉพาะ​สิ่ง​ที่​เจ้าของ​ขอ​ให้หา และ​ไม่​คัดลอก​แถว​ออกจาก​เครื่อง

== 9.3 สำเนา​ซ้ำ​ถูก​พับ​เป็นแถว​เดียว
<สำเนาซำถกพบเปนแถวเดยว>
เครื่อง​หนึ่ง​มัก​มี transcript ชุด​เดียวกัน​หลาย​ที่ เช่น mirror จาก​เครื่อง​อื่น ที่เก็บ​ถาวร
หรือ​สำรอง ถ้า​แต่ละ​สำเนา​นับ​เป็นแถว​แยก ตัวเลข​ทุก​ตัว​จะ​เพี้ยน​ไป​หมด
และ​ข้อความ​เดียวกัน​ถูก​อ่าน​ซ้ำ​หลาย​รอบ

AGENTS.md อธิบาย​ว่า #strong[สำเนา​ของ transcript ใน​หลาย home
ถูก​พับ​เป็นแถว​เดียว] แม้​สำเนา​จะ​ต่าง​กันที่ metadata เช่น cwd ที่​บันทึก คอลัมน์ `sources`
ของ​แถว​นั้น​บอก​ทุก​สำเนา​ที่​พบ ใน​รูป `agent:path:line` จึง​รู้​ได้​ว่า​แถว​หนึ่ง​มาจาก​ไหน​บ้าง

=== ขั้น​ที่ 1 — เห็นผล​ของ​การ​พับ​ใน​ตัวเลข
<ขนท-1-เหนผลของการพบในตวเลข>
#strong[ตัวเลข​ไม่​ถูก​นับ​ซ้ำ] ผล​ใน​เดโม​คือ​บน​หน้า Tree และ Paths ตัว​เลขที่​เห็น​คือ 145
ไฟล์ 144 sessions กับ 3,173 แถว โดย​ไม่​มี​การ​นับ​ซ้ำ เพราะ​เดโม​ไม่​มี mirror จริง
ผล​ของ​การ​พับ​เลย​ไม่​เด่น​ใน​ภาพ บท​นี้​ไม่​แต่ง​ตัวเลข​การ​พับ​ขึ้น​มา

อีก​ชั้นหนึ่ง​คือ ledger ที่​บท​ก่อน​เล่า​ไว้ AGENTS.md ระบุ​ว่า​สำเนา​ที่ ledger
จำได้​จะ​ไม่​ถูก​อ่าน​ซ้ำ และ​ถูก​จด​ไว้​ใน `_state.tsv` (คอลัมน์ `copy_of`) กับ
`_ledger.tsv` แทนที่จะ​อยู่​ใน `sources` ของ​แถว ถ้า​อยาก​ให้​เห็น​ว่า relic-scan
ข้าม​อะไร​ไป​บ้าง ใช้ `relic-scan log` ซึ่ง​นับ `copies`, `resumed` และ
`past_changed` พร้อม​รายการ​ไฟล์​ทีละ​ไฟล์

=== ขั้น​ที่ 2 — มอง​จาก​มุม​ความเป็นส่วนตัว
<ขนท-2-มองจากมมความเปนสวนตว>
การ​พับ​ช่วย​สอง​เรื่อง หนึ่ง ข้อความ​ส่วนตัว​ชิ้น​เดียว​ไม่​ถูก​เพิ่ม​เป็น​หลาย​แถว​ใน index
เมื่อ​ก๊อ​ปอ​ยู่​หลาย​ที่ สอง คุณ​รู้​ว่า​แถว​นั้น​มี​รอย​อยู่​ที่ไหนบ้าง​จาก `sources` ถ้า​ต้อง​ไป​จัดการ
เช่น​ลบ​สำเนา​เก่า ต้อง​รู้​ว่า​มี​สำเนา​ที่ไหน

แต่​ข้อควรระวัง​คือ การ​ลบ​แถว​ออกจาก index ไม่​ลบ​ไฟล์​ต้นฉบับ
และ​การ​ลบ​ไฟล์​ต้นฉบับ​ก็​ไม่​ได้​ลบ​แถว​ใน index ทันที ที่มา​ของ​เรื่อง​นี้​อยู่​ที่​หัวข้อ​ต่อไป

== 9.4 ปัญหา​ที่​ทีม​เจอ​จริง และ​ทางแก้
<ปญหาททมเจอจรง-และทางแก>
หัวข้อ​นี้​รวม​เฉพาะ​ข้อควรระวัง​ที่​เขียน​ไว้​ใน​เอกสาร​ของ repo เอง
ไม่​มี​ข้อ​ไหน​มาจาก​การ​เดา​ของ​ผู้เขียน​เล่ม ทุก​ข้อ​บอก​ที่มา เรียง​ตาม​โอกาส​ที่​คุณ​จะ​เจอ

=== ขั้น​ที่ 1 — เดือน​ที่​ปิด​แล้ว​ต่าง​จาก​ใน store
<ขนท-1-เดอนทปดแลวตางจากใน-store>
ที่มา AGENTS.md ส่วน "Copy the sealed months" และ scripts/README.md อาการ​คือ
เดือน​ที่​ปิด​ไป​แล้ว​ถูก​ปิด​ใหม่​หลัง​มี​แถว​ช้า​มาถึง สำเนา​ใน S3 จึง​ต่าง​จาก​ไฟล์​ใน​เครื่อง

```
A sealed month that differs in the store (it was sealed again after
a late row) is refused unless --overwrite.
```

บล็อก​ข้างบน​คัด​จาก AGENTS.md ตาม​ตัวอักษร และ​ไม่​ได้​รัน​ใน​เดโม #strong[ทางแก้​คือ​รัน
export ด้วย `--overwrite` ด้วยมือ] หลัง​ตรวจ​แล้ว​ว่า​ต้องการ​จริง
งาน​สำรอง​ตอนกลางคืน​ไม่​ทับ​ให้​เอง

=== ขั้น​ที่ 2 — index เก่า​มี​รูปร่าง​ไม่​เหมือน​เวอร์ชัน​ปัจจุบัน
<ขนท-2-index-เกามรปรางไมเหมอนเวอรชนปจจบน>
ที่มา AGENTS.md ส่วน "Query the index" อาการ​คือ คอลัมน์ `model`, `msg_id`,
`out_tokens` เป็น NULL ใน index ที่​เขียน​ด้วย​เวอร์ชัน v26.10.23-alpha.1858
หรือ​เก่า​กว่า และ​เดือน​ที่​เขียน​ก่อน​มี​คอลัมน์ `in_tokens`, `cache_read`, `cache_write`
ก็​อ่าน​เป็น NULL เช่นกัน ส่วน `meta/sessions.parquet` ก็​ไม่​มี `cwd`, `title`,
`branch` หรือ `version` จนกว่า​จะ​อ่าน​ไฟล์​ใหม่

ทางแก้​ที่​เอกสาร​ให้: #strong[ทำ index ใหม่​ตั้ง​แต่ต้น] คือ​ย้าย `~/.relic-scan`
ออก​ไป​ข้าง ๆ แล้ว​รัน `relic-scan run` ใหม่

```
move ~/.relic-scan aside, then relic-scan run
```

=== ขั้น​ที่ 3 — index ใหม่​ทิ้ง​แถว​ของ​ไฟล์​ที่​หาย​ไป
<ขนท-3-index-ใหมทงแถวของไฟลทหายไป>
ที่มา docs/TALK.md ส่วน "Who fixes what" อาการ​คือ index
ใหม่​ตั้ง​แต่ต้น#strong[จะ​ทิ้ง​แถว​ที่​ไฟล์​ต้นทาง​หาย​ไป​แล้ว] ดังนั้น​ถ้า​คุณ​เคย​ลบ​หรือ​ย้าย
transcript เก่า ตัวเลข​หลัง​ทำ​ใหม่​อาจ​น้อยลง ทางแก้​ที่​เอกสาร​ระบุ​คือ
เทียบ​จำนวน​แถว​ก่อน​สลับ​ไป index ใหม่ เก็บ​ของเก่า​ไว้​ข้าง ๆ จน​แน่ใจ

=== ขั้น​ที่ 4 — รัน​บาง​ช่วงเวลา​แล้ว​หยุด หรือ​รัน​ชน​กัน
<ขนท-4-รนบางชวงเวลาแลวหยด-หรอรนชนกน>
ที่มา docs/TUTORIAL.md ท้าย​เล่ม ตาราง​เมื่อ​มี​อะไร​ผิด สอง​ข้อ ข้อ​แรก ถ้า run
หยุด​เพราะ​มี​ตัว​อื่น​ถือ lock ซึ่ง​หมายความว่า #strong[หนึ่ง writer ต่อ​หนึ่ง​โฟลเดอร์]
ให้​รอ​ตัว​นั้น​เสร็จ ข้อ​สอง ถ้า run แบบ​ช่วง​หยุด​ด้วย​ข้อความ "indexed without one" แปล​ว่า
`--month` และ​ธง​วัน​ที่อื่น ๆ เขียน​ลง​โฟลเดอร์​ของ​ตัวเอง ไม่ต้อง​ตั้ง `--out`

ถ้า​ไฟล์​ที่​เจอ​น้อยกว่า​ที่​คิด ตาราง​นั้น​แนะนำ​ให้​เช็ก root ที่​สแกน (ธง `--root` หรือ​หน้า
Settings) และ​รัน `relic-scan log --diff 0`

=== ขั้น​ที่ 5 — เปลี่ยน binary ที่​กำลัง​รัน​อยู่
<ขนท-5-เปลยน-binary-ทกำลงรนอย>
ที่มา scripts/README.md และ docs/TALK.md อาการ​คือ บน macOS ถ้า `cp` ทับ
binary ที่​กำลัง​รัน​อยู่ ระบบ​จะ​ฆ่า​โปรเซส​นั้น ทางแก้​ที่​เขียน​ไว้​คือ #strong[ลบ​ก่อน​ด้วย
`rm -f` แล้ว​ค่อย `cp`]

```
rm -f ~/.local/bin/relic-scan && cp target/release/relic-scan ~/.local/bin/
```

บรรทัด​ข้างบน​เป็นการ​เรียบเรียง​ตามคำแนะนำ ไม่​ได้​รัน​ใน​เดโม เส้นทาง​ปลายทาง
`~/.local/bin/relic-scan` มาจาก​ที่ install.sh ติดตั้ง TALK.md เตือน​เพิ่ม​อีก​ชั้น
ให้​ทำ​หลัง build ตัวใหม่​เสร็จ​แล้ว​เท่านั้น เพราะ​เคย​มี​ครั้ง​ที่ `cp` ล้ม​หลัง `rm` แล้ว
relic-scan หาย​ไป​หก​นาที​บน​เครื่อง white

ครบ​ทั้ง​เก้า​บท​ของ​การใช้งาน การ​อ่าน index และ​การ​เดิน​ข้อมูล
ส่วน​ภาคผนวก​ต่อจากนี้​ลง​ไป​ใน​รูปแบบ​ของ transcript ที่ relic-scan อ่าน​จริง
บรรทัด​ต่อ​บรรทัด

= บท​ที่ 10: รูปแบบ​ของ transcript ที่ relic-scan อ่าน
<บทท-10-รปแบบของ-transcript-ท-relic-scan-อาน>
บท​นี้​เป็น​บท protocol ของ​หนังสือ ตอบคำถาม​เดียว​ว่า บรรทัด​ใน `.jsonl` หน้าตา​เป็น​ยังไง
แล้ว relic-scan หยิบ field ไหน​ไป​ทำ​เป็นแถว​ใน Parquet. ตัวอย่าง​ทุก​บรรทัด​มาจาก
`book/make-demo-home.py` ซึ่ง​สร้าง transcript ของ demo home
ด้วย​รูปทรง​เดียว​กับ​ของจริง (ค่า​ใน​นั้น​แต่ง​ขึ้น​หมด). ส่วน​ที่​อ่าน​จาก​ซอร์ส​คือ `src/engine.rs`,
`src/meta/rules.rs`, `src/meta/agg.rs`, `src/parse/merge.rs`.

ป้าย​ที่​ใช้​ใน​บท: \[capture\] คือ​มีผล​จริง​ใน `captures/` ให้​เทียบ ส่วน \[source\]
คือ​ยืนยัน​จาก​การ​อ่าน​โค้ด ไม่​ได้​รัน​ซ้ำ​ใน​บท​นี้.

== 10.1 Claude Code: หนึ่ง​บรรทัด หนึ่ง event
<claude-code-หนงบรรทด-หนง-event>
ไฟล์​อยู่​ใต้ `~/.claude/projects/<project-dir>/<session-uuid>.jsonl`.
บรรทัด​หนึ่ง​เป็น JSON object หนึ่ง​ตัว มี `type` บอก​ชนิด (`user`, `assistant`,
`custom-title`, `summary`) กับ `sessionId`, `timestamp`, `cwd`,
`gitBranch`, `version`.

=== ขั้น​ที่ 1 — ดู​บรรทัด assistant ที่​มี usage
<ขนท-1-ดบรรทด-assistant-ทม-usage>
บรรทัด​ข้างล่าง​สร้าง​จาก `claude_session()` ใน make-demo-home.py (ตัด​ให้​สั้น
จัด​บรรทัด​ใหม่​เป็น pretty JSON). #strong[assistant หนึ่ง response
แตก​เป็น​หลาย​บรรทัด] บรรทัด​ละ content block เดียว และ​ทุก​บรรทัด​แปะ `message.id`
กับ `usage` ชุด​เดียวกัน​ซ้ำ.

```json
{
  "type": "assistant",
  "timestamp": "2026-07-03T09:14:22Z",
  "sessionId": "0af4d28e-a79c-4ad8-bb26-26a0708bfab4",
  "cwd": "/opt/Code/github.com/laris-co/kite-docs",
  "gitBranch": "main",
  "version": "2.1.40",
  "message": {
    "id": "msg_0af4d28e_0",
    "role": "assistant",
    "model": "claude-opus-5-5",
    "usage": {
      "input_tokens": 21,
      "output_tokens": 1180,
      "cache_read_input_tokens": 4200,
      "cache_creation_input_tokens": 900
    },
    "content": [
      { "type": "tool_use", "id": "toolu_0af4d2_0_0",
        "name": "Bash", "input": { "command": "ls" } }
    ]
  }
}
```

ตัวเลข​ใน usage ตัว​อย่างนี้​แต่ง​เพื่อให้​อ่าน​ง่าย ไม่​ใช่​ค่า​จาก capture. สิ่ง​ที่ relic-scan
อ่าน​จาก​บรรทัด​แบบนี้ \[source: engine.rs `extract`, `usage_of`,
`extract_tools`\]:

- `sessionId` เป็น​รหัส session, `timestamp` เป็นเวลา (เก็บ​ทั้ง UTC และ `ts_th`
  เวลา​กรุงเทพ)
- `message.role` เป็น `user` หรือ `assistant`\; ถ้า​เป็น `developer`,
  `system` หรือ `toolResult` จะ​ไม่​เป็นแถว​ข้อความ
- `message.content` เป็น string ธรรมดา (ข้อความ​ของ user) หรือ array ของ
  block: `text`, `thinking`, `tool_use`, `tool_result`
- `message.model`, `message.id`, `usage.output_tokens`, `input_tokens`,
  `cache_read_input_tokens`, `cache_creation_input_tokens`
- `cwd`, `gitBranch`, `version` ไป​ลง `meta/sessions.parquet`\; ชื่อ
  session มาจาก​บรรทัด `custom-title`

=== ขั้น​ที่ 2 — ดู​ผลลัพธ์​ของ​เครื่องมือ​กลับมา
<ขนท-2-ดผลลพธของเครองมอกลบมา>
ผล​ของ tool มา​เป็น​บรรทัด `type: user` ที่ content เป็น array ของ
`tool_result`. คู่​กับ call ด้วย `tool_use_id` ซึ่ง​ตรง​กับ `id` ของ `tool_use`.
#strong[`is_error: true` คือ​ที่มา​ของ​คอลัมน์ `err`.]

```json
{
  "type": "user",
  "sessionId": "0af4d28e-a79c-4ad8-bb26-26a0708bfab4",
  "message": {
    "role": "user",
    "content": [
      { "type": "tool_result",
        "tool_use_id": "toolu_0af4d2_0_0",
        "content": "README.md  src",
        "is_error": false }
    ]
  }
}
```

บรรทัด​ไหน​ที่ JSON พัง หรือ​บรรทัด​สุดท้าย​ที่​ยัง​เขียน​ไม่​จบ relic-scan จะ​ไม่​เอา: ตัด​ที่ `\n`
ตัว​สุดท้าย เว้นแต่​ท้าย​ไฟล์ parse เป็น JSON ได้ #strong[\[source: parse/mod.rs
`complete`\]]. รอบ​หน้า​ไฟล์​โต​ขึ้น​ก็​อ่าน​ต่อ​จาก offset เดิม ผล​เท่ากับ​อ่าน​ใหม่​ทั้ง​ไฟล์ (มี
test `incremental_equals_cold` ใน parse/tests.rs).

== 10.2 Codex rollout: ห่อ​ด้วย payload
<codex-rollout-หอดวย-payload>
Codex เก็บ​เป็น `~/.codex/sessions/YYYY/MM/DD/rollout-<เวลา>-<uuid>.jsonl`
(หรือ `.jsonl.zst` ที่เก็บ​แล้ว ซึ่ง​ถอด​ระหว่าง​อ่าน). ทุก​บรรทัด​มี `type` ชั้นนอก กับ
`payload` ชั้นใน ไม่​มี `message`.

=== ขั้น​ที่ 1 — สาม​ชนิด​บรรทัด​ที่​ต้อง​รู้
<ขนท-1-สามชนดบรรทดทตองร>
`session_meta` ให้​รหัส session (`payload.id`) กับ `cwd`. `turn_context`
ให้​ชื่อ​โมเดล (`payload.model`). `response_item` คือ​เนื้อหา. #strong[Codex
ไม่​เขียน​โมเดล​บน​ข้อความ​เอง] relic-scan จึง​จำ​โมเดล​จาก `turn_context`
ล่าสุด​แล้ว​แปะ​ให้​แถว​ที่​ตามมา.

```json
{ "type": "session_meta", "timestamp": "2026-07-05T03:00:00Z",
  "payload": { "id": "5d1c…", "cwd": "/Users/demo/Code/moss-cli" } }
{ "type": "turn_context", "timestamp": "2026-07-05T03:00:00Z",
  "payload": { "model": "gpt-6-astra",
               "cwd": "/Users/demo/Code/moss-cli" } }
{ "type": "response_item", "timestamp": "2026-07-05T03:01:10Z",
  "payload": { "type": "message", "role": "assistant",
    "content": [ { "type": "output_text", "text": "Added." } ] } }
```

ส่วน tool ของ Codex อยู่​ใน `response_item` เหมือนกัน แต่ `payload.type` เป็น
`function_call` (ชื่อ​ใน `name`, อาร์กิวเมนต์​ใน `arguments`),
`custom_tool_call`, และ​ผล​เป็น `function_call_output` จับคู่​ด้วย `call_id`.
ส่วน `reasoning` อ่าน​จาก `summary` เพราะ​เนื้อ​คิด​จริง​เข้ารหัส #strong[\[source:
engine.rs `extract_tools`, `thinking_of`, `turn_model`\]].

ผล​ที่​เห็น​บน​จอ #strong[\[capture 16\]]: แถว​ของ `gpt-6-astra` นับ `calls` ได้ 0
และ `out_tokens` เป็น `NULL` ทุก​เดือน เพราะ​ไม่​มี `msg_id` และ​ไม่​มี usage ให้​เก็บ.

== 10.3 omp: message ชั้นเดียว ชื่อ field ต่าง​ออก​ไป
<omp-message-ชนเดยว-ชอ-field-ตางออกไป>
omp (และ pi) เก็บ​ใต้
`~/.omp/agent/sessions/<project>/<เวลา>_<uuid>.jsonl`. บรรทัด​เป็น
`type: "message"` แล้ว​มี `message` ข้างใน หน้าตา​คล้าย Claude แต่ id กับ usage
ใช้​ชื่อ​สั้น​กว่า.

=== ขั้น​ที่ 1 — ดู​ชื่อ field ที่​ต่าง
<ขนท-1-ดชอ-field-ทตาง>
```json
{
  "type": "message",
  "id": "a0",
  "timestamp": "2026-07-04T10:02:31Z",
  "message": {
    "role": "assistant",
    "model": "glm-5.3",
    "responseId": "r_95f119_0",
    "usage": { "input": 412, "output": 230,
               "cacheRead": 1800, "cacheWrite": 0 },
    "content": [ { "type": "text", "text": "Done." } ]
  }
}
```

เทียบ​ชื่อ field ของ Claude กับ omp (จาก `usage_of` ใน engine.rs
#strong[\[source\]]):

- response id: Claude `message.id`, omp `message.responseId`
- output: `usage.output_tokens` กับ `usage.output`
- input: `usage.input_tokens` กับ `usage.input`
- cache read: `usage.cache_read_input_tokens` กับ `usage.cacheRead`
- cache write: `usage.cache_creation_input_tokens` กับ `usage.cacheWrite`

โค้ด​ลอง​ชื่อ​แบบ Claude ก่อน ไม่​เจอ​ก็​ใช้​ชื่อ​แบบ omp. ผล​ของ tool ใน omp มา​เป็น
`message.role == "toolResult"` พร้อม `toolCallId`, `toolName`, `isError`
และ call เป็น block ชนิด `toolCall` (อาร์กิวเมนต์​ใน `arguments`). แถว​พวก​นี้​เป็น
tool row อยู่แล้ว จึง​ไม่​นับ​เป็น​ข้อความ​ซ้ำ​อีก.

== 10.4 บรรทัด​หนึ่ง​กลาย​เป็นแถว​ได้​ยังไง
<บรรทดหนงกลายเปนแถวไดยงไง>
=== ขั้น​ที่ 1 — อ่าน​กฎ​ที่​ตัดสิน​ชนิด​ของ​แถว
<ขนท-1-อานกฎทตดสนชนดของแถว>
บรรทัด​หนึ่ง​แตก​ได้​เป็น​หลาย "stream": `text` (ข้อความ user/assistant),
`thinking`, และ `tools` (call กับ result). ตามที่​เห็น​ใน `types_of` ของ
agg.rs ตัว​เดียว​นี้​ตัดสิน​ว่า​แถว​นั้น​นับ​เป็น​อะไร:

```rust
pub(super) fn types_of(s: &str, role: Option<&str>, kind: Option<&str>, err: bool) -> [i64; 6] {
    let mut t = [0; 6];
    match (s, role, kind) {
        ("text", Some("user"), _) => t[0] = 1,
        ("text", Some("assistant"), _) => t[1] = 1,
        ("thinking", _, _) => t[2] = 1,
        ("tools", _, Some("call")) => t[3] = 1,
        ("tools", _, Some("result")) => {
            t[4] = 1;
            t[5] = err as i64;
        }
        _ => {}
    }
    t
}
```

อ่าน​แล้ว​ได้​กฎ​ตาม​นี้ #strong[\[source\]]: ข้อความ​มี `role` แต่​ไม่​มี `kind`\; tool มี
`kind` (`call` หรือ `result`) แต่​ไม่​มี `role`\; thinking ไม่​ขึ้นกับ role. error
คือ result ที่ `err` จริง จึง​ถูก​นับ​ใน results ด้วย.

#figure(image("/images/D2-line-to-rows.png", width: 94%, alt: "แผนภาพ ONE TRANSCRIPT LINE BECOMES ROWS: response msg_A สาม​บรรทัด thinking, text, tool_use กลายเป็น​สาม​แถว token อยู่​แถว thinking แถว​เดียว"),
  caption: [
    แผนภาพ ONE TRANSCRIPT LINE BECOMES ROWS: response msg\_A สาม​บรรทัด
    thinking, text, tool\_use กลายเป็น​สาม​แถว token อยู่​แถว thinking แถว​เดียว
  ]
)

ภาพ​นี้​เห็น​สิ่ง​ที่​โค้ด​ข้างบน​ไม่​ได้​บอก​ตรง ๆ คือ สาม​บรรทัด​ของ msg\_A กลายเป็น​สาม​แถว แต่
token อยู่​แถว thinking แถว​เดียว อีก​สอง​แถว​เป็น NULL และ​สำเนา​จาก​อีก home
พับ​เข้าแถว​เดิม

=== ขั้น​ที่ 2 — เทียบ​กับ​ตัวเลข​บน​จอ
<ขนท-2-เทยบกบตวเลขบนจอ>
รัน​คำสั่ง​นับ​แถว​ตาม​ชนิด​ที่ AGENTS.md แนะนำ แล้ว​ดู​ว่า​แต่ละ​บรรทัด​ตรง​กับ `types_of`.

```bash
duckdb -c "select coalesce(nullif(kind,''), nullif(role,''), '(none)') \
  as what, count(*) as n from '*.parquet' group by 1 order by n desc"
```

#figure(image("/images/13-duck-what.png", width: 100%, alt: "ผล duckdb นับ​แถว​ตาม​ชนิด result 705, call 705, user 628, assistant 628, thinking 507"),
  caption: [
    ผล duckdb นับ​แถว​ตาม​ชนิด result 705, call 705, user 628, assistant 628,
    thinking 507
  ]
)

ภาพ​นี้​ซ้ำ​จาก​บท​ที่ 6

#strong[\[capture 13\]] result 705, call 705, assistant 628, user 628,
thinking 507 รวม 3,173 แถว เท่ากับ​ตัวเลข​บน​หน้า Smart Scan. call กับ result
เท่ากัน​เพราะ demo จับคู่​ครบ​ทุก​ตัว. thinking มี 507 แถว​แม้​บาง​แถว​ข้อ​ความว่าง ตรง​กับ​ที่
Claude Code รุ่นใหม่​เขียน thinking เป็น​แค่ signature (แถว​ยังอยู่ `text` ว่าง).

บรรทัด assistant เดียว​ที่​มี thinking, text, tool\_use สาม​บล็อก​จึง​ได้​สาม​แถว.
ถ้า​เป็น​ข้อความ​ที่​ส่ง​ต่อกัน​หลาย​เครื่อง สำเนา​ของ​บรรทัด​เดียวกัน​ถู​กรวม​เป็นแถว​เดียว โดย​กุญแจ​คือ
hash `blake2b` แปด​ไบต์​ของ​ตัว​บรรทัด (`engine::key`) และ `sources`
เก็บ​ทุก​สำเนา.

=== ขั้น​ที่ 3 — ดู​คอลัมน์​ของ​ไฟล์​เดือน
<ขนท-3-ดคอลมนของไฟลเดอน>
อยากรู้​ว่า​แถว​มี​คอลัมน์​อะไร​บ้าง ให้ `describe` ไฟล์​เดือน​ได้​เลย

```bash
duckdb -c "describe select * from '2026-10.parquet'"
```

#figure(image("/images/46-describe-month.png", width: 58%, alt: "ผล describe ไฟล์ 2026-10.parquet ตาราง​ชื่อ​คอลัมน์​กับ​ชนิด 24 แถว"),
  caption: [
    ผล describe ไฟล์ 2026-10.parquet ตาราง​ชื่อ​คอลัมน์​กับ​ชนิด 24 แถว
  ]
)

ไฟล์​เดือน​มี #strong[24 คอลัมน์] ทั้ง `role`, `kind`, `red`, `sources`, `msg_id`
และ​คอลัมน์ token สี่​ตัว `out_tokens`, `in_tokens`, `cache_read`,
`cache_write` เป็น BIGINT ส่วน `machines` กับ `sources` เป็น VARCHAR\[\]

== 10.5 token เก็บ​บน​แถว​เดียว​ต่อ msg\_id
<token-เกบบนแถวเดยวตอ-msg_id>
=== ขั้น​ที่ 1 — ดู​ว่า merge เก็บ token ที่ไหน
<ขนท-1-ดวา-merge-เกบ-token-ทไหน>
เพราะ response เดียว​แตก​เป็น​หลาย​บรรทัด​ที่​แปะ usage ซ้ำ ถ้า​บวก​ตรง ๆ
จะ​นับ​เกิน​หลายเท่า. ตอน merge relic-scan จึง​เก็บ token ไว้​ที่​แถว​แรก​ของ​แต่ละ
`msg_id` เท่านั้น แถว​อื่น​เป็น NULL. ข้อความ​จาก parse/merge.rs ตรงนี้:

```rust
    // one API response is several rows (one per content block, and every copy of its lines): its output tokens stay on
    // the first of them only, the largest any copy recorded, so sum(out_tokens) counts each response once
    let mut tokens: HashMap<&str, (usize, [Option<i64>; 4])> = HashMap::new();
    for &i in &order {
        if let (Some(id), true) = (cands[i].msg_id.as_deref(), cands[i].tok[0].is_some()) {
            let e = tokens.entry(id).or_insert((i, cands[i].tok));
            for k in 0..4 {
                e.1[k] = e.1[k].max(cands[i].tok[k]);
            }
        }
    }
```

สอง​จุด​ที่​ต้อง​จำ: #strong[ใช้​ค่า​มาก​สุด​ที่​สำเนา​ไหน​บันทึก​ไว้] (`max`), และ
`count(distinct msg_id)` คือ​จำนวน​ครั้ง​ที่​เรียก API.

=== ขั้น​ที่ 2 — นับ call และ token ต่อ​เดือน​ต่อ​โมเดล
<ขนท-2-นบ-call-และ-token-ตอเดอนตอโมเดล>
```bash
duckdb -c "select left(ts_th,7) as month, model, \
  count(distinct msg_id) as calls, sum(out_tokens) as out_tokens \
  from '*.parquet' where model is not null \
  group by all order by month, calls desc limit 12"
```

#figure(image("/images/16-duck-tokens.png", width: 100%, alt: "ผล duckdb ตาราง​เดือน โมเดล calls และ out_tokens สิบสอง​แถว เริ่ม​ที่ 2026-07 claude-sonnet-5-5"),
  caption: [
    ผล duckdb ตาราง​เดือน โมเดล calls และ out\_tokens สิบสอง​แถว เริ่ม​ที่
    2026-07 claude-sonnet-5-5
  ]
)

ภาพ​นี้​ซ้ำ​จาก​บท​ที่ 6

#strong[\[capture 16\]] เดือน 2026-07: claude-sonnet-5-5 มี 52 calls,
70967 out\_tokens; claude-haiku-5-5 45 calls; claude-opus-5-5 32 calls;
glm-5.3 10 calls, 4925. ส่วน `gpt-6-astra` ได้ 0 calls กับ `NULL`
ตามที่​อธิบาย​ใน​หัวข้อ 10.2.

=== ขั้น​ที่ 3 — รวม​ขึ้น​ระดับ session
<ขนท-3-รวมขนระดบ-session>
ฝั่ง `agg.rs` มี struct `Usage` ที่​บวก `out`, `inp`, `cache_read`,
`cache_write` กับ​นับ​คำตอบ​ต่อ​โมเดล. จากนั้น​เขียน​ลง `meta/sessions.parquet` เป็น
`models`, `model_msgs`, `in_tokens`, `out_tokens`, `cache_read_tokens`,
`cache_write_tokens` #strong[\[source\]].

```bash
duckdb -c "select sid, models, model_msgs, in_tokens, out_tokens, \
  cache_read_tokens from 'meta/sessions.parquet' \
  where out_tokens > 0 order by out_tokens desc limit 4"
```

#figure(image("/images/17-duck-session.png", width: 96%, alt: "ผล duckdb ตาราง session สี่​แถว models, model_msgs, in_tokens, out_tokens, cache_read_tokens"),
  caption: [
    ผล duckdb ตาราง session สี่​แถว models, model\_msgs, in\_tokens,
    out\_tokens, cache\_read\_tokens
  ]
)

ภาพ​นี้​ซ้ำ​จาก​บท​ที่ 6

\[capture 17\] session `95f11992` ใช้ `claude-haiku-5-5` ตอบ 7 ครั้ง ได้
`in_tokens` 121, `out_tokens` 11285, `cache_read_tokens` 335324. ลำดับ
`models` เรียง​จาก​โมเดล​ที่​ตอบ​มาก​ที่สุด (`ranked()` ใน agg.rs).

=== ขั้น​ที่ 4 — ดู​คอลัมน์​ของ sessions
<ขนท-4-ดคอลมนของ-sessions>
คอลัมน์​ที่​ขั้น​ก่อน​เรียก​ใช้​มาจาก​ไฟล์ `meta/sessions.parquet` ซึ่ง `describe`
ได้​เหมือนกัน

```bash
duckdb -c "describe select * from 'meta/sessions.parquet'"
```

#figure(image("/images/47-describe-sess.png", width: 64%, alt: "ผล describe ไฟล์ meta/sessions.parquet ตาราง​ชื่อ​คอลัมน์​กับ​ชนิด 38 แถว"),
  caption: [
    ผล describe ไฟล์ meta/sessions.parquet ตาราง​ชื่อ​คอลัมน์​กับ​ชนิด 38 แถว
  ]
)

ไฟล์​นี้​มี #strong[38 คอลัมน์] รวม `models` กับ `model_msgs` ที่​เป็น array, `cwd`,
`title`, `branch`, `version` และ​ตัว​นับ `user_text`, `assistant_text`,
`thinking`, `tool_calls`, `tool_results`, `tool_errors`

== 10.6 แยก session ด้วย​เส้นทาง​ไฟล์
<แยก-session-ดวยเสนทางไฟล>
relic-scan ไม่​ได้​เดา​จาก​เนื้อใน: ตัดสิน​จาก #strong[path ของ​ไฟล์] ด้วย regex
ชุด​เดียว​ใน `rules.rs`. `tier_of` ตอบ​ว่า main, subagent, workflow-agent หรือ
other:

```rust
pub fn tier_of(path: &str) -> &'static str {
    let r = rx();
    if r.wf.is_match(path) {
        "workflow-agent"
    } else if r.sub.is_match(path) || r.sub_omp.is_match(path) {
        "subagent"
    } else if r.main_uuid.is_match(path) || r.main_rollout.is_match(path) || r.main_omp.is_match(path)
        || (r.main_other.is_match(path) && !r.history.is_match(path)) {
        "main"
    } else {
        "other"
    }
}
```

ลำดับ​สำคัญ: เช็ก workflow ก่อน, แล้ว subagent (`/subagents/agent-….jsonl` ของ
Claude หรือ​โฟลเดอร์ `<time>_<uuid>/<Task>.jsonl` ของ omp), แล้ว​ถึง main
(ไฟล์​ชื่อ uuid, `rollout-…uuid.jsonl`, ไฟล์ omp). `history.jsonl` ของ
omx/thclaws/gemini ตกเป็น `other`. subagent มี `parent_sid` ที่​ดึง​จาก uuid ใน
path.

=== ขั้น​ที่ 1 — repo (`cat`) มาจาก​ชื่อ​โฟลเดอร์​โปรเจกต์
<ขนท-1-repo-cat-มาจากชอโฟลเดอรโปรเจกต>
Claude เข้ารหัส cwd เป็น​ชื่อ​โฟลเดอร์ เช่น
`-opt-Code-github-com-laris-co-kite-docs`. `cat_of` ตัด​ส่วน `github-com-`
ออก ตัด​ชื่อ org ที่​รู้จัก ตัด​หาง worktree แล้ว​เหลือ `kite-docs`. engine อื่น​ไม่​มี cwd
ใน path จึง​ได้ชื่อ engine ต่อท้าย​ว่า `(no cwd in path)`:

```rust
pub fn cat_of(engine: &str, proj: Option<&str>) -> String {
    let r = rx();
    if engine != "claude" {
        return format!("{engine} (no cwd in path)");
    }
    let Some(p) = proj.filter(|p| !p.is_empty()) else { return "(no project dir)".into() };
    let lp = p.to_lowercase();
    if lp.contains("oracle-probe-cwd") {
        "probe (automated)".into()
    } else if lp.contains("github-com-") {
        let x = extract(&r.gh, &lp, 1);
        let x = r.gh_org.replace(&x, "");
        let x = r.gh_wt.replace(&x, "");
        r.dashes.replace_all(&x, "").into_owned()
    } else if lp.contains("-worktrees-") {
        let x = extract(&r.wts, &lp, 1);
        r.wts_wt.replace(&x, "").into_owned()
    } else {
        "home / other".into()
    }
}
```

ข้อควรระวัง #strong[\[source\]]: regex ชื่อ org (`soul-brews-studio`,
`laris-co`, …) ฝังใน​โค้ด. repo ของ org อื่น​ที่​ชื่อ path ขึ้น​ด้วย `github-com-`
จะ​ได้ชื่อ​ยาว​ติด org ไว้. อีก​อย่าง ชื่อ​โปรเจกต์​ที่​มี​ขีด​อยู่แล้ว (`kite-docs`)
ก็​ยัง​อ่าน​ออก​เพราะ​ตัด​แค่​ขีด​หัว​ท้าย.

=== ขั้น​ที่ 2 — เปิด​อ่าน​บทสนทนา​ของ session หนึ่ง
<ขนท-2-เปดอานบทสนทนาของ-session-หนง>
พอ​รู้​ว่า `sid` คือ uuid ใน path ก็​ดึง​บทสนทนา​ด้วย `sid` ได้​เลย:

```bash
duckdb -c "select role, left(text, 70) as text from '*.parquet' \
  where sid = '0af4d28e-a79c-4ad8-bb26-26a0708bfab4' \
  and role in ('user','assistant') order by ts limit 8"
```

#figure(image("/images/18-duck-read.png", width: 98%, alt: "ผล duckdb บทสนทนา​แปด​แถว user กับ assistant สลับ​กัน เริ่ม​ที่ summarise yesterday's errors from the log"),
  caption: [
    ผล duckdb บทสนทนา​แปด​แถว user กับ assistant สลับ​กัน เริ่ม​ที่ summarise
    yesterday's errors from the log
  ]
)

ภาพ​นี้​ซ้ำ​จาก​บท​ที่ 6

#strong[\[capture 18\]] แถว​แรก​เป็น `user` ข้อความ
`summarise yesterday's errors from the log` แล้ว​ตาม​ด้วย `assistant` ที่​ตอบ
สลับ​กัน​ครบ​แปด​แถว. ผล​นี้​ยืนยัน​ว่า `role` กับ `text` ที่ relic-scan ถอด​จาก​บรรทัด
`user`/`assistant` ตรง​กับ​ที่​คน​พิมพ์​จริง​ใน transcript.

== ข้อสังเกต​เรื่อง​ความเชื่อถือได้
<ขอสงเกตเรองความเชอถอได>
ทั้งหมด​นี้​เป็นการ​อ่าน​แบบ​อ่าน​อย่าง​เดียว relic-scan ไม่​แก้ transcript.
ไฟล์​ที่​ไม่​เข้า​รูปทรง​ข้างบน​ไม่​ทำให้​รอบ​สแกน​ล้ม: บรรทัด​ที่ parse ไม่​ได้​ถูก​นับ​เป็น `bad_json`
ใน `Stats` แล้ว​ข้าม​ไป \[source: parse/mod.rs\]. ส่วน​ไฟล์​ที่​ไม่​ใช่ transcript
เลย​จะ​ติดป้าย verdict ว่า not-transcript ใน​หน้า Paths แทนที่จะ​ถูก​อ่าน​เป็นแถว.

ถ้า​อยาก​เพิ่ม​รูปทรง​ใหม่ (agent ตัว​อื่น) ทาง​สั้น​สุด​คือ​ดู​ว่า​มัน "คล้าย" ตัว​ไหน​ใน `extract`
ก่อน: engine ที่​เข้า​ตระกูล `message` ใช้ได้​เลย ส่วน​ที่​ห่อ​ด้วย `payload`
ต้อง​ดู​ตัวอย่าง​ของ Codex. ชื่อ field ของ usage ที่​ต่างกัน​แก้​จุด​เดียว​ที่ `usage_of`.

== ต่อไป
<ตอไป-2>
ตอนนี้​รู้​แล้ว​ว่า​บรรทัด​ดิบ​เป็นแถว​ได้​ยังไง บท​ถัดไป​คือ​โค้ด​ตัวจริง​ที่​เลือก​มา​พิมพ์​ไว้​ท้าย​เล่ม
ให้​เปิด​ดู​ตรง​กับ​ที่​ยก​มา​ใน​บท​นี้.

= ภาคผนวก: โค้ด​ที่​เลือก
<ภาคผนวก-โคดทเลอก>
ไฟล์​ด้านล่าง​พิมพ์​ตามจริง​ทุก​บรรทัด จาก commit `7ae3299` ของ
Soul-Brews-Studio/relic4-scan ไม่​ย่อ ไม่​แก้​ให้​สวย เลือก​เฉพาะ​ส่วน​ที่​บท​ต่าง ๆ ชี้​ไป​ถึง

#quote(block: true)[
relic-scan เป็น​โอเพน​ซอร์ส (AGPL v3) โค้ด​ทั้ง​ชุด​อยู่​ที่
https:/\/github.com/Soul-Brews-Studio/relic4-scan
]

== src/engine.rs
<srcengine.rs>
how one transcript line becomes rows: the Claude, Codex and omp readers,
redaction, usage · 526 บรรทัด

```rust
//! Port of jsonl-truth's line parser: src/relic4/engine.py (extract, extract_tools, thinking_of, redact,
//! ts_of, stamp, key) as driven by src/relic4/store.py parse_chunk. Every function mirrors its Python twin;
//! docs/OUTPUT-CHAIN.md lists the rules and verify/understand.py checks them on real files.

use std::sync::OnceLock;

use chrono::{DateTime, Datelike, Timelike};
use regex::bytes::{Regex, RegexBuilder};
use serde_json::{Map, Value};

// ---------- gjson-like accessors (engine.py gstr / graw / gbool / get) ----------

fn dumps(v: &Value) -> String {
    serde_json::to_string(v).unwrap_or_default() // compact, key order kept: orjson.dumps(...).decode()
}

/// gjson .String(): strings as is, missing/null as "", anything else as its JSON.
pub fn gstr(v: Option<&Value>) -> String {
    match v {
        None | Some(Value::Null) => String::new(),
        Some(Value::String(s)) => s.clone(),
        Some(Value::Bool(b)) => if *b { "true".into() } else { "false".into() },
        Some(x) => dumps(x),
    }
}

fn graw(v: Option<&Value>) -> String {
    match v {
        None | Some(Value::Null) => String::new(),
        Some(x) => dumps(x),
    }
}

fn gbool(v: Option<&Value>) -> bool {
    match v {
        Some(Value::Bool(b)) => *b,
        Some(Value::String(s)) => s == "true",
        Some(Value::Number(n)) => n.as_f64().is_some_and(|f| f != 0.0),
        _ => false,
    }
}

fn get<'a>(o: Option<&'a Value>, k: &str) -> Option<&'a Value> {
    o.and_then(|x| x.as_object()).and_then(|m| m.get(k))
}

/// Go strings.TrimSpace = Unicode White_Space, the same set as Rust's char::is_whitespace (not 0x1C-0x1F).
fn trim_go(s: &str) -> &str {
    s.trim_matches(char::is_whitespace)
}

/// Go's builder: a separator only once something non-empty was written.
fn join_nl<I: IntoIterator<Item = String>>(parts: I, sep: &str) -> String {
    let mut out = String::new();
    for p in parts {
        if !out.is_empty() {
            out.push_str(sep);
        }
        out.push_str(&p);
    }
    out
}

// ---------- time ----------

const MONTHS: [&str; 13] = ["", "January", "February", "March", "April", "May", "June", "July", "August", "September",
                            "October", "November", "December"];

fn parse_iso(t: &str) -> Option<i64> {
    // Python datetime.fromisoformat, tz-aware only (Go RFC3339 needs a zone)
    let d = DateTime::parse_from_rfc3339(t)
        .or_else(|_| DateTime::parse_from_str(t, "%Y-%m-%dT%H:%M:%S%.f%z"))
        .or_else(|_| DateTime::parse_from_str(t, "%Y-%m-%dT%H:%M:%S%z"))
        .or_else(|_| DateTime::parse_from_str(t, "%Y-%m-%dT%H:%M%z"))
        .ok()?;
    Some(d.timestamp_micros().div_euclid(1000)) // (d - EPOCH) // MS: floor to the millisecond
}

/// Epoch milliseconds, or None. Numbers: seconds (thclaws) or milliseconds.
pub fn ts_of(r: &Map<String, Value>) -> Option<i64> {
    match r.get("timestamp") {
        Some(Value::Number(n)) => {
            let f = n.as_f64()?;
            Some(if f > 1e12 { n.as_i64().unwrap_or(f as i64) } else { (f * 1000.0) as i64 })
        }
        Some(Value::String(t)) if t.chars().count() > 10 && t.chars().nth(10) == Some('T') => parse_iso(t),
        _ => None,
    }
}

pub struct Stamp {
    pub month: String, // "2026-10" or "undated"
    pub ts: String,    // UTC, 24 chars, Z
    pub ts_th: String, // +07:00
}

pub fn stamp(ms: Option<i64>) -> Stamp {
    let Some(ms) = ms else { return Stamp { month: "undated".into(), ts: String::new(), ts_th: String::new() } };
    let u = DateTime::from_timestamp_millis(ms).unwrap_or_default();
    let b = DateTime::from_timestamp_millis(ms + 7 * 3600 * 1000).unwrap_or_default();
    let f = format!(".{:03}", ms.rem_euclid(1000));
    let _ = MONTHS;
    Stamp {
        month: format!("{:04}-{:02}", b.year(), b.month()),
        ts: format!("{:04}-{:02}-{:02}T{:02}:{:02}:{:02}{f}Z", u.year(), u.month(), u.day(), u.hour(), u.minute(), u.second()),
        ts_th: format!("{:04}-{:02}-{:02}T{:02}:{:02}:{:02}{f}+07:00", b.year(), b.month(), b.day(), b.hour(), b.minute(), b.second()),
    }
}

// ---------- extract ----------

fn content_text(c: Option<&Value>) -> String {
    match c {
        Some(Value::String(s)) => s.clone(),
        Some(Value::Array(a)) => join_nl(
            a.iter().filter(|it| it.is_object() && matches!(gstr(it.get("type")).as_str(), "text" | "input_text" | "output_text"))
                .map(|it| gstr(it.get("text"))),
            "\n",
        ),
        _ => String::new(),
    }
}

pub struct Extract {
    pub typ: String,
    pub role: String,
    pub sess: String,
    pub ts: Option<i64>,
    pub text: String,
}

/// Claude Code, Codex, omp/pi and thclaws shapes -> (type, role, session, ts_ms, text).
pub fn extract(r: &Map<String, Value>) -> Extract {
    let typ = gstr(r.get("type"));
    let ts = ts_of(r);
    let mut sess = gstr(r.get("sessionId"));
    let payload = r.get("payload");
    if typ == "session_meta" {
        sess = gstr(get(payload, "id"));
    } else if typ == "session" || typ == "header" {
        sess = gstr(r.get("id"));
    } else if typ == "summary" {
        let text = gstr(r.get("summary"));
        return Extract { typ, role: String::new(), sess, ts, text };
    }
    let (mut role, mut c) = (String::new(), None);
    if let Some(Value::Object(m)) = r.get("message") {
        role = gstr(m.get("role"));
        c = m.get("content");
    } else if typ == "response_item" && gstr(get(payload, "type")) == "message" {
        role = gstr(get(payload, "role"));
        c = get(payload, "content");
    } else if typ == "user" || typ == "assistant" {
        role = typ.clone();
        c = r.get("content");
    }
    // developer/system text is not conversation; an omp tool result (role "toolResult") is already a tool row
    // (extract_tools), so it is not a text row as well
    if role == "developer" || role == "system" || role == "toolResult" {
        return Extract { typ, role, sess, ts, text: String::new() };
    }
    let text = trim_go(&content_text(c)).to_string();
    Extract { typ, role, sess, ts, text }
}

/// Which model answered, and the API response a line belongs to. Claude Code and omp/pi write them on the assistant
/// message: Claude `model`, `id`, `usage.output_tokens` (repeated on every content-block line of one response); omp
/// `model`, `responseId`, `usage.output`. Codex writes no model on its messages: see turn_model.
#[derive(Default, Debug, PartialEq)]
pub struct Usage {
    pub model: String,
    pub msg_id: String,
    pub out_tokens: Option<i64>,
    /// input tokens and the prompt cache (Claude `input_tokens`, `cache_read_input_tokens`, `cache_creation_input_tokens`;
    /// omp `input`, `cacheRead`, `cacheWrite`)
    pub in_tokens: Option<i64>,
    pub cache_read: Option<i64>,
    pub cache_write: Option<i64>,
}

pub fn usage_of(r: &Map<String, Value>) -> Usage {
    let Some(Value::Object(m)) = r.get("message") else { return Usage::default() };
    if gstr(m.get("role")) != "assistant" {
        return Usage::default();
    }
    let id = gstr(m.get("id"));
    let u = m.get("usage");
    Usage {
        model: gstr(m.get("model")),
        msg_id: if id.is_empty() { gstr(m.get("responseId")) } else { id },
        out_tokens: get(u, "output_tokens").or_else(|| get(u, "output")).and_then(Value::as_i64),
        in_tokens: get(u, "input_tokens").or_else(|| get(u, "input")).and_then(Value::as_i64),
        cache_read: get(u, "cache_read_input_tokens").or_else(|| get(u, "cacheRead")).and_then(Value::as_i64),
        cache_write: get(u, "cache_creation_input_tokens").or_else(|| get(u, "cacheWrite")).and_then(Value::as_i64),
    }
}

/// What a line says about its session as a whole: where it ran, what it is called, the branch, the Claude version.
/// Claude writes `cwd`, `gitBranch` and `version` on its lines and the name as a `custom-title` (or `ai-title`) line;
/// Codex writes `cwd` in the payload of its session_meta and turn_context lines.
#[derive(Default, Debug, Clone, PartialEq)]
pub struct SessMeta {
    pub cwd: String,
    pub title: String,
    pub branch: String,
    pub version: String,
    /// a message that came in through a channel plugin (Discord, Telegram, a fleet bridge): the plugin, the room and
    /// the sender of the first such message of the session
    pub via: String,
    pub chat: String,
    pub from_user: String,
}

pub fn sess_meta_of(r: &Map<String, Value>) -> SessMeta {
    let typ = gstr(r.get("type"));
    let mut cwd = gstr(r.get("cwd"));
    if cwd.is_empty() && matches!(typ.as_str(), "session_meta" | "turn_context") {
        cwd = gstr(get(r.get("payload"), "cwd"));
    }
    let title = match typ.as_str() {
        "custom-title" => gstr(r.get("customTitle")),
        "ai-title" => gstr(r.get("aiTitle")),
        _ => String::new(),
    };
    SessMeta { cwd, title, branch: gstr(r.get("gitBranch")), version: gstr(r.get("version")), ..Default::default() }
}

/// The channel a user message came in by: `<channel source="plugin:discord:discord" chat_id="…" user="…" …>`.
pub fn channel_of(text: &str) -> SessMeta {
    let Some(i) = text.find("<channel source=\"") else { return SessMeta::default() };
    let tag = &text[i..text[i..].find('>').map(|e| i + e).unwrap_or(text.len())];
    let attr = |k: &str| -> String {
        let pat = format!(" {k}=\"");
        tag.find(&pat).map(|p| &tag[p + pat.len()..]).and_then(|s| s.split('"').next()).unwrap_or("").to_string()
    };
    let via = tag["<channel source=\"".len()..].split('"').next().unwrap_or("").to_string();
    SessMeta { via, chat: attr("chat_id"), from_user: attr("user"), ..Default::default() }
}

impl SessMeta {
    /// Fold a later line in: the first cwd stays (where the session began); the others follow the newest line.
    pub fn take(&mut self, x: SessMeta) {
        if self.cwd.is_empty() {
            self.cwd = x.cwd;
        }
        // the first channel message names the session's channel
        if self.via.is_empty() {
            (self.via, self.chat, self.from_user) = (x.via, x.chat, x.from_user);
        }
        for (a, b) in [(&mut self.title, x.title), (&mut self.branch, x.branch), (&mut self.version, x.version)] {
            if !b.is_empty() {
                *a = b;
            }
        }
    }
}

/// Codex names the model once per turn, in a `turn_context` line; it holds for the turn's rows after it.
pub fn turn_model(r: &Map<String, Value>) -> Option<String> {
    (gstr(r.get("type")) == "turn_context").then(|| gstr(get(r.get("payload"), "model"))).filter(|m| !m.is_empty())
}

const TOOL_HINTS: [&[u8]; 5] = [b"tool_use", b"tool_result", b"_call", b"toolCall", b"toolResult"];

fn block_text(c: Option<&Value>) -> String {
    match c {
        Some(Value::String(s)) => s.clone(),
        Some(Value::Array(a)) => join_nl(
            a.iter().filter_map(|it| it.as_object()).filter(|o| o.contains_key("text")).map(|o| gstr(o.get("text"))),
            "\n",
        ),
        other => graw(other),
    }
}

pub struct Tool {
    pub kind: &'static str,
    pub call_id: String,
    pub name: String,
    pub body: String,
    pub err: bool,
}

/// Calls and results, joinable by call_id.
pub fn extract_tools(line: &[u8], r: &Map<String, Value>) -> Vec<Tool> {
    if !TOOL_HINTS.iter().any(|h| memchr::memmem::find(line, h).is_some()) {
        return vec![];
    }
    let mut out = vec![];
    if gstr(r.get("type")) == "response_item" {
        let p = r.get("payload");
        match gstr(get(p, "type")).as_str() {
            "function_call" => out.push(Tool { kind: "call", call_id: gstr(get(p, "call_id")), name: gstr(get(p, "name")),
                                               body: gstr(get(p, "arguments")), err: false }),
            "custom_tool_call" => out.push(Tool { kind: "call", call_id: gstr(get(p, "call_id")), name: gstr(get(p, "name")),
                                                  body: gstr(get(p, "input")), err: false }),
            "function_call_output" | "custom_tool_call_output" => {
                let o = get(p, "output");
                let body = match o {
                    Some(Value::Object(m)) => gstr(m.get("content")),
                    other => gstr(other),
                };
                out.push(Tool { kind: "result", call_id: gstr(get(p, "call_id")), name: String::new(), body, err: false });
            }
            _ => {}
        }
        return out;
    }
    let m = r.get("message");
    if let Some(Value::Object(mm)) = m {
        if gstr(mm.get("role")) == "toolResult" {
            return vec![Tool { kind: "result", call_id: gstr(mm.get("toolCallId")), name: gstr(mm.get("toolName")),
                               body: block_text(mm.get("content")), err: gbool(mm.get("isError")) }];
        }
    }
    let c = match m {
        Some(Value::Object(mm)) if mm.contains_key("content") => mm.get("content"),
        _ => r.get("content"),
    };
    let Some(Value::Array(items)) = c else { return out };
    for it in items {
        let o = it.as_object();
        match gstr(get(Some(it), "type")).as_str() {
            "tool_use" => out.push(Tool { kind: "call", call_id: gstr(get(Some(it), "id")), name: gstr(get(Some(it), "name")),
                                          body: graw(o.and_then(|o| o.get("input"))), err: false }),
            "toolCall" => out.push(Tool { kind: "call", call_id: gstr(get(Some(it), "id")), name: gstr(get(Some(it), "name")),
                                          body: graw(o.and_then(|o| o.get("arguments"))), err: false }),
            "tool_result" => out.push(Tool { kind: "result", call_id: gstr(get(Some(it), "tool_use_id")), name: String::new(),
                                             body: block_text(o.and_then(|o| o.get("content"))),
                                             err: gbool(o.and_then(|o| o.get("is_error"))) }),
            _ => {}
        }
    }
    out
}

/// Readable reasoning: Claude/omp thinking blocks, Codex reasoning summary (content is encrypted).
/// None: the line has no thinking. Some(""): it thought, but the text was not saved (newer Claude Code writes a thinking
/// block as a signature only; a Codex reasoning item may have no summary), so the row is kept with empty text.
pub fn thinking_of(line: &[u8], r: &Map<String, Value>) -> Option<String> {
    if memchr::memmem::find(line, b"\"thinking\"").is_none() && memchr::memmem::find(line, b"\"reasoning\"").is_none() {
        return None;
    }
    let mut parts: Vec<String> = vec![];
    let mut blocks = 0;
    let mut add = |t: Option<&Value>| {
        let s = gstr(t);
        let s = trim_go(&s);
        if !s.is_empty() {
            parts.push(s.to_string());
        }
    };
    let payload = r.get("payload");
    if gstr(r.get("type")) == "response_item" && gstr(get(payload, "type")) == "reasoning" {
        for k in ["summary", "content"] {
            if let Some(Value::Array(v)) = get(payload, k) {
                for it in v {
                    add(get(Some(it), "text"));
                }
            }
        }
        return Some(parts.join("\n\n"));
    }
    let c = match r.get("message") {
        Some(Value::Object(mm)) if mm.contains_key("content") => mm.get("content"),
        _ => r.get("content"),
    };
    if let Some(Value::Array(items)) = c {
        for it in items {
            if gstr(get(Some(it), "type")) == "thinking" {
                blocks += 1;
                add(it.get("thinking"));
            }
        }
    }
    (blocks > 0).then(|| parts.join("\n\n"))
}

// ---------- line key + JSON parse ----------

/// blake2b(line, digest_size=8) read as a big-endian signed i64 (engine.py:364). The line keeps its "\n".
pub fn key(line: &[u8]) -> i64 {
    let h = blake2b_simd::Params::new().hash_length(8).hash(line);
    i64::from_be_bytes(h.as_bytes().try_into().unwrap())
}

/// engine.py LONE: a lone UTF-16 surrogate escape becomes � (what Go's gjson keeps), then parse again.
fn fix_lone_surrogates(line: &[u8]) -> Vec<u8> {
    let hex = |c: u8| c.is_ascii_hexdigit();
    let is_hi = |b: &[u8], i: usize| b.len() >= i + 6 && b[i] == b'\\' && b[i + 1] == b'u' && matches!(b[i + 2], b'd' | b'D')
        && matches!(b[i + 3], b'8' | b'9' | b'a' | b'b' | b'A' | b'B') && hex(b[i + 4]) && hex(b[i + 5]);
    let is_lo = |b: &[u8], i: usize| b.len() >= i + 6 && b[i] == b'\\' && b[i + 1] == b'u' && matches!(b[i + 2], b'd' | b'D')
        && (b[i + 3].is_ascii_hexdigit() && matches!(b[i + 3].to_ascii_lowercase(), b'c'..=b'f')) && hex(b[i + 4]) && hex(b[i + 5]);
    let mut out = Vec::with_capacity(line.len());
    let mut i = 0;
    while i < line.len() {
        let not_escaped = i == 0 || line[i - 1] != b'\\';
        if not_escaped && is_hi(line, i) && !is_lo(line, i + 6) {
            out.extend_from_slice(b"\\ufffd");
            i += 6;
            continue;
        }
        if not_escaped && is_lo(line, i) {
            let prev_hi_tail = i >= 3 && matches!(line[i - 3], b'8' | b'9' | b'a' | b'b' | b'A' | b'B') && hex(line[i - 2]) && hex(line[i - 1]);
            if !prev_hi_tail {
                out.extend_from_slice(b"\\ufffd");
                i += 6;
                continue;
            }
        }
        out.push(line[i]);
        i += 1;
    }
    out
}

/// The line as a JSON object, {} when it is JSON but not an object, None when it is not JSON at all.
pub fn parse(line: &[u8]) -> Option<Value> {
    serde_json::from_slice::<Value>(line).ok().or_else(|| serde_json::from_slice::<Value>(&fix_lone_surrogates(line)).ok())
}

// ---------- redaction (engine.py RULES, same order) ----------

struct Rule {
    kind: &'static str,
    hints: &'static [&'static str],
    rx: Regex,
    group: usize,
}

fn rules() -> &'static Vec<Rule> {
    static R: OnceLock<Vec<Rule>> = OnceLock::new();
    R.get_or_init(|| {
        // Python re.ASCII semantics: \b \s \w and (?i) are ASCII-only, so unicode(false) on bytes
        let b = |p: &str| RegexBuilder::new(p).unicode(false).build().expect("rule");
        vec![
            Rule { kind: "private-key", hints: &["PRIVATE KEY"], group: 0,
                   rx: b(r"-----BEGIN (?:RSA |EC |OPENSSH |DSA |PGP )?PRIVATE KEY-----[\s\S]*?(?:-----END (?:RSA |EC |OPENSSH |DSA |PGP )?PRIVATE KEY-----|\z)") },
            Rule { kind: "anthropic-key", hints: &["sk-ant-"], group: 0, rx: b(r"sk-ant-[A-Za-z0-9_-]{30,}") },
            Rule { kind: "openai-key", hints: &["sk-"], group: 0, rx: b(r"\bsk-(?:proj-)?[A-Za-z0-9_-]{32,}") },
            Rule { kind: "aws-key", hints: &["AKIA", "ASIA"], group: 0, rx: b(r"\b(?:AKIA|ASIA)[0-9A-Z]{16}\b") },
            Rule { kind: "github-token", hints: &["gh", "github_pat_"], group: 0,
                   rx: b(r"\b(?:gh[pousr]_[A-Za-z0-9]{36,}|github_pat_[A-Za-z0-9_]{60,})") },
            Rule { kind: "slack-token", hints: &["xox"], group: 0, rx: b(r"\bxox[abprs]-[A-Za-z0-9-]{10,}") },
            Rule { kind: "google-key", hints: &["AIza"], group: 0, rx: b(r"\bAIza[0-9A-Za-z_-]{35}") },
            Rule { kind: "jwt", hints: &["eyJ"], group: 0, rx: b(r"\beyJ[A-Za-z0-9_-]{10,}\.eyJ[A-Za-z0-9_-]{10,}\.[A-Za-z0-9_-]{10,}") },
            Rule { kind: "bearer", hints: &["earer"], group: 1, rx: b(r"(?i)\bbearer\s+([A-Za-z0-9._~+/-]{24,})") },
            Rule { kind: "password", hints: &["ass", "ecret", "oken", "pi_key", "pikey", "API_KEY", "APIKEY"], group: 1,
                   rx: b(r#"(?i)\b(?:password|passwd|secret|api_?key|token)\s*[=:]\s*['"]?([A-Za-z0-9!#%&*+,./:;=?@^_|~-]{8,})"#) },
        ]
    })
}

/// A password/token value has letters AND digits and entropy >= 2.5 bits/char.
fn secret_like(v: &str) -> bool {
    if v.starts_with(['$', '<', '{', '(']) || v.contains('…') {
        return false;
    }
    if !v.chars().any(char::is_alphabetic) || !v.chars().any(|c| c.is_numeric()) {
        return false;
    }
    let chars: Vec<char> = v.chars().collect();
    let n = chars.len() as f64;
    let mut counts = std::collections::HashMap::new();
    for c in &chars {
        *counts.entry(*c).or_insert(0usize) += 1;
    }
    let e: f64 = -counts.values().map(|&k| { let p = k as f64 / n; p * p.log2() }).sum::<f64>();
    e >= 2.5
}

/// (clean text, sorted kinds joined by ',' or "").
pub fn redact(input: &str) -> (String, String) {
    let mut s = input.to_string();
    let mut kinds: Vec<&str> = vec![];
    for rule in rules() {
        if !rule.hints.iter().any(|h| s.contains(h)) {
            continue;
        }
        let bytes = s.as_bytes();
        let mut out = String::new();
        let mut last = 0usize;
        let mut hit = false;
        for caps in rule.rx.captures_iter(bytes) {
            let Some(m) = caps.get(rule.group) else { continue };
            let (a, z) = (m.start(), m.end());
            if a < last || bytes.get(a) == Some(&b'[') {
                continue; // already redacted by an earlier rule
            }
            let v = &s[a..z];
            if rule.kind == "password" && !secret_like(v) {
                continue;
            }
            out.push_str(&s[last..a]);
            let head: String = v.chars().take(4).collect();
            out.push_str(&format!("[{}:{}…{}]", rule.kind, head, v.chars().count()));
            last = z;
            hit = true;
        }
        if hit && last > 0 {
            out.push_str(&s[last..]);
            s = out;
            kinds.push(rule.kind);
        }
    }
    kinds.sort();
    (s, kinds.join(","))
}

/// store.engine_of: the engine from the file path.
pub fn engine_of(path: &str) -> &'static str {
    let p = path.to_lowercase();
    if p.contains(".thclaws") {
        "thclaws"
    } else if p.contains(".omx") {
        "omx"
    } else if p.contains("/.pi/") {
        "pi"
    } else if p.contains(".omp") {
        "omp"
    } else if p.contains("codex") || p.contains("rollout-") {
        "codex"
    } else {
        "claude"
    }
}
```

== src/meta/rules.rs
<srcmetarules.rs>
which file is which session: tier\_of, cat\_of, proj\_of and the path
rules · 136 บรรทัด

```rust
//! The path rules (sql.py MACROS, one for one): tier, engine, repo (cat), project, parent and home of a file.

use super::*;


pub(super) const UUID: &str = "[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}";

pub(super) struct Rx {
    pub(super) wf: Regex,
    pub(super) sub: Regex,
    pub(super) main_uuid: Regex,
    pub(super) main_rollout: Regex,
    pub(super) main_omp: Regex,
    pub(super) sub_omp: Regex,
    pub(super) parent_omp: Regex,
    pub(super) main_other: Regex,
    pub(super) history: Regex,
    pub(super) proj: Regex,
    pub(super) parent: Regex,
    pub(super) home: Regex,
    pub(super) agent_sid: Regex,
    pub(super) uuid_file: Regex,
    pub(super) gh: Regex,
    pub(super) gh_org: Regex,
    pub(super) gh_wt: Regex,
    pub(super) dashes: Regex,
    pub(super) wts: Regex,
    pub(super) wts_wt: Regex,
}

pub(super) fn rx() -> &'static Rx {
    static R: OnceLock<Rx> = OnceLock::new();
    R.get_or_init(|| {
        let r = |p: &str| Regex::new(p).expect("meta rule");
        Rx {
            wf: r(r"/subagents/workflows/.*agent-[^/]+\.jsonl$"),
            sub: r(r"/subagents/agent-[^/]+\.jsonl$"),
            main_uuid: r(&format!(r"/{UUID}\.jsonl$")),
            main_rollout: r(&format!(r"rollout-.*{UUID}\.jsonl(\.zst)?$")),
            main_omp: r(r"/\.(omp|pi)/.*_[0-9a-f-]+\.jsonl$"),
            // omp keeps a subagent's transcript inside its parent session's folder, named for its task:
            //   sessions/<project>/<time>_<uuid>/<Task>.jsonl  (the parent is sessions/<project>/<time>_<uuid>.jsonl)
            sub_omp: r(&format!(r"/\.(omp|pi)/.*_{UUID}/[^/]+\.jsonl$")),
            parent_omp: r(&format!(r"/\.(?:omp|pi)/.*_({UUID})/[^/]+\.jsonl$")),
            main_other: r(r"/\.(omx|thclaws|gemini)/"),
            history: r(r"history\.jsonl$"),
            proj: r(&format!(r"/([^/]+)/(?:{UUID}\.jsonl|{UUID}/subagents/)")),
            parent: r(&format!(r"/({UUID})/subagents/")),
            home: r(r"^(.*?/\.?(?:claude|codex|omp|omx|pi|thclaws|gemini)[^/]*)/"),
            agent_sid: r(r"^agent-(.*)\.jsonl$"),
            uuid_file: r(&format!(r"^({UUID})\.jsonl$")),
            gh: r(r".*github-com-(.*)$"),
            gh_org: r(r"^(soul-brews-studio|laris-co|dustboy-pm25|floodboy-cm|nazt|nat-weerawan)-"),
            gh_wt: r(r"(-worktree-|-wt-|--claude-worktrees-).*$"),
            dashes: r(r"^-+|-+$"),
            wts: r(r".*-worktrees-(.*)$"),
            wts_wt: r(r"(-worktree-|-wt-).*$"),
        }
    })
}

/// DuckDB regexp_extract: group `g` of the first match, "" when there is none.
pub(super) fn extract(re: &Regex, s: &str, g: usize) -> String {
    re.captures(s).and_then(|c| c.get(g)).map(|m| m.as_str().to_string()).unwrap_or_default()
}

pub fn tier_of(path: &str) -> &'static str {
    let r = rx();
    if r.wf.is_match(path) {
        "workflow-agent"
    } else if r.sub.is_match(path) || r.sub_omp.is_match(path) {
        "subagent"
    } else if r.main_uuid.is_match(path) || r.main_rollout.is_match(path) || r.main_omp.is_match(path)
        || (r.main_other.is_match(path) && !r.history.is_match(path)) {
        "main"
    } else {
        "other"
    }
}

pub fn engine_of(path: &str) -> &'static str {
    if path.contains("rollout-") || path.contains("/.codex") {
        "codex"
    } else if path.contains("/.omp/") {
        "omp"
    } else if path.contains("/.omx/") {
        "omx"
    } else if path.contains("/.thclaws/") {
        "thclaws"
    } else if path.contains("/.gemini/") {
        "gemini"
    } else if path.contains("/.pi/") {
        "pi"
    } else {
        "claude"
    }
}

pub(super) fn base_of(path: &str) -> &str {
    path.rsplit('/').next().unwrap_or(path)
}

pub(super) fn proj_of(path: &str) -> Option<String> {
    Some(extract(&rx().proj, path, 1)).filter(|s| !s.is_empty())
}

pub(super) fn parent_of(path: &str) -> Option<String> {
    Some(extract(&rx().parent, path, 1)).filter(|s| !s.is_empty())
        .or_else(|| Some(extract(&rx().parent_omp, path, 1)).filter(|s| !s.is_empty()))
}

pub(super) fn home_of(path: &str) -> String {
    extract(&rx().home, path, 1)
}

pub fn cat_of(engine: &str, proj: Option<&str>) -> String {
    let r = rx();
    if engine != "claude" {
        return format!("{engine} (no cwd in path)");
    }
    let Some(p) = proj.filter(|p| !p.is_empty()) else { return "(no project dir)".into() };
    let lp = p.to_lowercase();
    if lp.contains("oracle-probe-cwd") {
        "probe (automated)".into()
    } else if lp.contains("github-com-") {
        let x = extract(&r.gh, &lp, 1);
        let x = r.gh_org.replace(&x, "");
        let x = r.gh_wt.replace(&x, "");
        r.dashes.replace_all(&x, "").into_owned()
    } else if lp.contains("-worktrees-") {
        let x = extract(&r.wts, &lp, 1);
        r.wts_wt.replace(&x, "").into_owned()
    } else {
        "home / other".into()
    }
}
```

== src/meta/agg.rs
<srcmetaagg.rs>
the per-month pass that counts rows by type and sums tokens per session,
and its cache · 273 บรรทัด

```rust
//! One pass per month over the rows' sources, and its cache in OUT/meta/cache/YYYY-MM.tsv.

use super::*;


/// Rows by type, in this order: user text, assistant text, thinking, tool calls, tool results, tool errors
/// (errors are results with err = true, so they are also counted in results).
pub const TYPES: [&str; 6] = ["user_text", "assistant_text", "thinking", "tool_calls", "tool_results", "tool_errors"];

pub(super) fn types_of(s: &str, role: Option<&str>, kind: Option<&str>, err: bool) -> [i64; 6] {
    let mut t = [0; 6];
    match (s, role, kind) {
        ("text", Some("user"), _) => t[0] = 1,
        ("text", Some("assistant"), _) => t[1] = 1,
        ("thinking", _, _) => t[2] = 1,
        ("tools", _, Some("call")) => t[3] = 1,
        ("tools", _, Some("result")) => {
            t[4] = 1;
            t[5] = err as i64;
        }
        _ => {}
    }
    t
}

#[derive(Default, Clone)]
pub(super) struct Agg {
    pub(super) n_rows: i64,
    pub(super) text_rows: i64,
    pub(super) text_msgs: i64,
    pub(super) by: [i64; 6],
    pub(super) first_ts: Option<String>,
    pub(super) last_ts: Option<String>,
}

impl Agg {
    pub(super) fn ts(&mut self, ts: Option<&str>) {
        let Some(t) = ts else { return };
        if self.first_ts.as_deref().is_none_or(|f| t < f) {
            self.first_ts = Some(t.to_string());
        }
        if self.last_ts.as_deref().is_none_or(|l| t > l) {
            self.last_ts = Some(t.to_string());
        }
    }
    pub(super) fn add(&mut self, o: &Agg) {
        self.n_rows += o.n_rows;
        self.text_rows += o.text_rows;
        self.text_msgs += o.text_msgs;
        for (a, b) in self.by.iter_mut().zip(o.by) {
            *a += b;
        }
        self.ts(o.first_ts.as_deref());
        self.ts(o.last_ts.as_deref());
    }
}

/// What a session's model answers cost: tokens summed over its API responses (each once: the month file keeps a
/// response's tokens on one row) and the answers per model.
#[derive(Default, Clone, Debug, PartialEq)]
pub(super) struct Usage {
    pub(super) out: i64,
    pub(super) inp: i64,
    pub(super) cache_read: i64,
    pub(super) cache_write: i64,
    pub(super) models: BTreeMap<String, i64>,
}

impl Usage {
    pub(super) fn add(&mut self, o: &Usage) {
        self.out += o.out;
        self.inp += o.inp;
        self.cache_read += o.cache_read;
        self.cache_write += o.cache_write;
        for (m, n) in &o.models {
            *self.models.entry(m.clone()).or_default() += n;
        }
    }
    /// The models, the most answers first, and their answer counts.
    pub(super) fn ranked(&self) -> (Vec<String>, Vec<i64>) {
        let mut v: Vec<(&String, &i64)> = self.models.iter().collect();
        v.sort_by(|a, b| b.1.cmp(a.1).then(a.0.cmp(b.0)));
        (v.iter().map(|x| x.0.clone()).collect(), v.iter().map(|x| *x.1).collect())
    }
}

/// One month's aggregates: per file path, and per session (base, tier).
#[derive(Default)]
pub(super) struct MonthAgg {
    pub(super) files: HashMap<String, Agg>,
    pub(super) sessions: HashMap<(String, String), Agg>,
    pub(super) usage: HashMap<(String, String), Usage>,
}

/// msgkey for a text row is md5(to_json([s, role, ts, text])); count(DISTINCT msgkey) only needs a stable key of
/// (role, ts, text) with NULL kept apart from any string, so a length-prefixed blake2b-8 does the same job.
pub(super) fn msg_key(role: Option<&str>, ts: Option<&str>, text: Option<&str>) -> u64 {
    let mut st = blake2b_simd::Params::new().hash_length(8).to_state();
    for v in [role, ts, text] {
        match v {
            None => {
                st.update(&[0]);
            }
            Some(s) => {
                st.update(&[1]);
                st.update(&(s.len() as u64).to_le_bytes());
                st.update(s.as_bytes());
            }
        }
    }
    u64::from_le_bytes(st.finalize().as_bytes().try_into().unwrap())
}

pub(super) fn str_col<'a>(b: &'a RecordBatch, n: &str) -> &'a StringArray {
    b.column_by_name(n).unwrap().as_any().downcast_ref::<StringArray>().unwrap()
}

pub(super) fn opt(a: &StringArray, i: usize) -> Option<&str> {
    if a.is_null(i) { None } else { Some(a.value(i)) }
}

pub(super) fn month_agg(path: &Path, agent: &str) -> MonthAgg {
    let b = ParquetRecordBatchReaderBuilder::try_new(fs::File::open(path).unwrap()).unwrap();
    // model and the token columns came later: a month written before them has none
    let want = ["s", "role", "kind", "err", "ts", "text", "sources", "model", "msg_id", "out_tokens", "in_tokens", "cache_read", "cache_write"];
    let idx: Vec<usize> = want.iter().filter_map(|n| b.schema().index_of(n).ok()).collect();
    let mask = ProjectionMask::roots(b.parquet_schema(), idx);
    let rd = b.with_projection(mask).with_batch_size(16_384).build().unwrap();
    let prefix = format!("{agent}:");
    let mut out = MonthAgg::default();
    let mut class: HashMap<String, Option<(String, String)>> = HashMap::new(); // path -> session key (None = tier other)
    let mut msgs: HashMap<(String, String), HashSet<u64>> = HashMap::new();
    let mut touched: Vec<(String, String)> = vec![];
    for batch in rd {
        let batch = batch.unwrap();
        let (s, role, ts, text) = (str_col(&batch, "s"), str_col(&batch, "role"), str_col(&batch, "ts"), str_col(&batch, "text"));
        let kind = str_col(&batch, "kind");
        let err = batch.column_by_name("err").unwrap().as_any().downcast_ref::<arrow::array::BooleanArray>().unwrap();
        let model = batch.column_by_name("model").map(|a| a.as_any().downcast_ref::<StringArray>().unwrap());
        let msg_id = batch.column_by_name("msg_id").map(|a| a.as_any().downcast_ref::<StringArray>().unwrap());
        let tok = |n: &str| batch.column_by_name(n).map(|a| a.as_any().downcast_ref::<Int64Array>().unwrap());
        let (t_out, t_in, t_cr, t_cw) = (tok("out_tokens"), tok("in_tokens"), tok("cache_read"), tok("cache_write"));
        let src = batch.column_by_name("sources").unwrap().as_any().downcast_ref::<ListArray>().unwrap();
        let vals = src.values().as_any().downcast_ref::<StringArray>().unwrap();
        let offs = src.value_offsets();
        for i in 0..batch.num_rows() {
            let is_text = s.value(i) == "text";
            let t = opt(ts, i);
            let ty = types_of(s.value(i), opt(role, i), opt(kind, i), !err.is_null(i) && err.value(i));
            touched.clear();
            for j in offs[i] as usize..offs[i + 1] as usize {
                let Some(rest) = vals.value(j).strip_prefix(&prefix) else { continue };
                let Some((p, _line)) = rest.rsplit_once(':') else { continue };
                let fa = out.files.entry(p.to_string()).or_default();
                fa.n_rows += 1;
                fa.text_rows += is_text as i64;
                for (a, b) in fa.by.iter_mut().zip(ty) {
                    *a += b;
                }
                fa.ts(t);
                let k = class.entry(p.to_string()).or_insert_with(|| {
                    let tier = tier_of(p);
                    (tier != "other").then(|| (base_of(p).to_string(), tier.to_string()))
                });
                if let Some(k) = k
                    && !touched.contains(k) {
                    touched.push(k.clone()); // a row counts once per session, however many of its copies it holds
                }
            }
            // an answer: the row that carries its tokens, or an assistant text row with no response id (Codex)
            let n = |c: Option<&Int64Array>| c.and_then(|a| (!a.is_null(i)).then(|| a.value(i))).unwrap_or(0);
            let has_tokens = t_out.is_some_and(|a| !a.is_null(i));
            let answer = has_tokens || (is_text && opt(role, i) == Some("assistant") && msg_id.is_none_or(|a| a.is_null(i)));
            let mdl = model.and_then(|a| opt(a, i)).filter(|m| !m.is_empty());
            for k in &touched {
                if has_tokens || answer {
                    let u = out.usage.entry(k.clone()).or_default();
                    u.out += n(t_out);
                    u.inp += n(t_in);
                    u.cache_read += n(t_cr);
                    u.cache_write += n(t_cw);
                    if let (true, Some(m)) = (answer, mdl) {
                        *u.models.entry(m.to_string()).or_default() += 1;
                    }
                }
                let sa = out.sessions.entry(k.clone()).or_default();
                sa.n_rows += 1;
                sa.text_rows += is_text as i64;
                for (a, b) in sa.by.iter_mut().zip(ty) {
                    *a += b;
                }
                sa.ts(t);
                if is_text {
                    msgs.entry(k.clone()).or_default().insert(msg_key(opt(role, i), t, opt(text, i)));
                }
            }
        }
    }
    for (k, m) in msgs {
        out.sessions.get_mut(&k).unwrap().text_msgs = m.len() as i64;
    }
    out
}

// ---------- the month cache: OUT/meta/cache/YYYY-MM.tsv ----------

pub(super) fn stamp_of(p: &Path) -> String {
    let m = fs::metadata(p).unwrap();
    format!("{}\t{}.{}", m.size(), m.mtime(), m.mtime_nsec())
}

// NULL is written as \N: an empty ts is a real value (the undated month holds rows with ts = "")
pub(super) fn agg_tsv(a: &Agg) -> String {
    let by: Vec<String> = a.by.iter().map(|x| x.to_string()).collect();
    format!("{}\t{}\t{}\t{}\t{}\t{}", a.n_rows, a.text_rows, a.text_msgs, by.join("\t"), a.first_ts.as_deref().unwrap_or("\\N"),
            a.last_ts.as_deref().unwrap_or("\\N"))
}

pub(super) fn agg_from(c: &[&str]) -> Agg {
    let n = |i: usize| c[i].parse::<i64>().unwrap_or(0);
    let t = |i: usize| Some(c[i].to_string()).filter(|s| s != "\\N");
    Agg { n_rows: n(0), text_rows: n(1), text_msgs: n(2), by: std::array::from_fn(|j| n(3 + j)), first_ts: t(9), last_ts: t(10) }
}

/// The cache format and the rules it was counted with: bump it when a path rule changes what a session is (#4: omp
/// subagents are sessions), so every month is counted again once.
const CACHE_V: &str = "#5";

pub(super) fn save_cache(p: &Path, stamp: &str, agent: &str, m: &MonthAgg) {
    let tmp = p.with_extension("tsv.part");
    let mut w = std::io::BufWriter::new(fs::File::create(&tmp).unwrap());
    writeln!(w, "{CACHE_V}\t{stamp}\t{agent}").unwrap();
    for (f, a) in &m.files {
        writeln!(w, "F\t{f}\t{}", agg_tsv(a)).unwrap();
    }
    for ((base, tier), a) in &m.sessions {
        writeln!(w, "S\t{base}\t{tier}\t{}", agg_tsv(a)).unwrap();
    }
    for ((base, tier), u) in &m.usage {
        let models = serde_json::to_string(&u.models).unwrap();
        writeln!(w, "U\t{base}\t{tier}\t{}\t{}\t{}\t{}\t{models}", u.out, u.inp, u.cache_read, u.cache_write).unwrap();
    }
    w.flush().unwrap();
    drop(w);
    fs::rename(tmp, p).unwrap();
}

/// The cached aggregates, or None when the cache was made from another version of the month file.
pub(super) fn load_cache(p: &Path, stamp: &str, agent: &str) -> Option<MonthAgg> {
    let mut lines = BufReader::new(fs::File::open(p).ok()?).lines().map_while(Result::ok);
    if lines.next()? != format!("{CACHE_V}\t{stamp}\t{agent}") {
        return None;
    }
    let mut m = MonthAgg::default();
    for l in lines {
        let c: Vec<&str> = l.split('\t').collect();
        match c[0] {
            "F" if c.len() == 13 => {
                m.files.insert(c[1].to_string(), agg_from(&c[2..]));
            }
            "S" if c.len() == 14 => {
                m.sessions.insert((c[1].to_string(), c[2].to_string()), agg_from(&c[3..]));
            }
            "U" if c.len() == 8 => {
                let n = |i: usize| c[i].parse::<i64>().unwrap_or(0);
                let models = serde_json::from_str(c[7]).ok()?;
                m.usage.insert((c[1].to_string(), c[2].to_string()), Usage { out: n(3), inp: n(4), cache_read: n(5), cache_write: n(6), models });
            }
            _ => return None,
        }
    }
    Some(m)
}
```

== src/ops/text.rs
<srcopstext.rs>
the text view of `relic-scan ops` · 74 บรรทัด

```rust
//! The snapshot as text lines, for `relic-scan ops` and `--watch`.

use serde_json::Value;

pub fn dur(s: i64) -> String {
    if s >= 3600 { format!("{}:{:02}:{:02}", s / 3600, s / 60 % 60, s % 60) } else { format!("{}:{:02}", s / 60, s % 60) }
}

pub fn render(v: &Value) -> Vec<String> {
    let c = &v["chain"];
    let mut o = vec![];
    match c["state"].as_str() {
        Some("running") => {
            let (el, eta) = (c["elapsed_s"].as_i64().unwrap_or(0), c["eta_s"].as_f64());
            o.push(format!("CHAIN {}  RUNNING  {} elapsed{}", c["machine"].as_str().unwrap_or(""), dur(el),
                match (c["typical_total_s"].as_f64(), eta) {
                    (Some(t), Some(e)) if e > 0.0 => format!(" · typical {} · about {} left", dur(t as i64), dur(e as i64)),
                    (Some(t), _) => format!(" · typical {}, over it now", dur(t as i64)),
                    _ => String::new() }));
        }
        Some("idle") => o.push(format!("CHAIN {}  idle  last pass {} took {} · next in about {}", c["machine"].as_str().unwrap_or(""),
            c["started"].as_str().unwrap_or("")[11..].to_string(), dur(c["took_s"].as_i64().unwrap_or(0)), dur(c["next_in_s"].as_i64().unwrap_or(0)))),
        _ => o.push("CHAIN  no pass logged yet (".to_string() + v["log"].as_str().unwrap_or("") + ")"),
    }
    for s in c["stages"].as_array().into_iter().flatten() {
        let mark = match s["state"].as_str() { Some("done") => "✓", Some("running") => "●", _ => "·" };
        let t = s["secs"].as_f64().map(|x| format!("{x:.0} s")).unwrap_or_default();
        let ty = s["typical_s"].as_f64().map(|x| format!("(typical {x:.0} s)")).unwrap_or_default();
        o.push(format!("  {mark} {:<18} {:>7} {:<18} {}", s["name"].as_str().unwrap_or(""), t, ty, s["detail"].as_str().unwrap_or("")));
    }
    let p = v["processes"].as_array().cloned().unwrap_or_default();
    o.push(String::new());
    o.push(if p.is_empty() { "PROCESSES  none running".into() } else { "PROCESSES".into() });
    for x in &p {
        o.push(format!("  {:<22} {:<8} {:>9}  {:>5.0}% cpu  {:>6.0} MB", x["what"].as_str().unwrap_or(""), x["user"].as_str().unwrap_or(""), x["elapsed"].as_str().unwrap_or(""),
            x["cpu"].as_f64().unwrap_or(0.0), x["rss_mb"].as_f64().unwrap_or(0.0)));
    }
    let h = &v["hub"];
    o.push(String::new());
    if h["ok"].as_bool() == Some(true) {
        o.push(format!("HUB  load {:.1} of {} cpus · {} job(s) running", h["load"][0].as_f64().unwrap_or(0.0), h["cpus"], h["running"].as_array().map_or(0, |a| a.len())));
        for m in h["machines"].as_array().into_iter().flatten() {
            o.push(format!("  {:<20} last push {:>6} ago  {}", m["machine"].as_str().unwrap_or(""), dur(m["push_age_s"].as_i64().unwrap_or(0)),
                m["sync"].as_str().unwrap_or("").chars().take(60).collect::<String>()));
        }
        for w in h["warnings"].as_array().into_iter().flatten() {
            o.push(format!("  ! {}", w.as_str().unwrap_or("")));
        }
    } else {
        o.push(format!("HUB  not read ({})", h["error"].as_str().unwrap_or("")));
    }
    let s = &v["s3"];
    if s["running"].as_bool() == Some(true) {
        o.push(String::new());
        o.push(format!("S3  {} {}/{}", s["job"].as_str().unwrap_or(""), s["done"], s["total"]));
    }
    o
}

/// `render` for a terminal: the word HUB is a link to the hub, and the last line points at the page's Activity view.
/// `on` false leaves the escapes out (a pipe, NO_COLOR); the page line stays as plain text.
pub fn render_linked(v: &Value, base: &str, hub: &str, on: bool) -> Vec<String> {
    use crate::links::{link, view_url};
    let mut o = render(v);
    if hub != "off" && v["hub"]["ok"].as_bool() == Some(true) {
        if let Some(l) = o.iter_mut().find(|l| l.starts_with("HUB")) {
            *l = format!("{}{}", link(on, hub, "HUB"), &l[3..]);
        }
    }
    let page = view_url(base, "ops");
    o.push(String::new());
    o.push(format!("page  {}", link(on, &page, &page)));
    o
}
```
