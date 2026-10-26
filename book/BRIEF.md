# BRIEF — shared by every chapter writer

Book dir (absolute): `/opt/Code/github.com/Soul-Brews-Studio/relic4-scan/book`
Read `OUTLINE.md` there first: title, parts, and YOUR chapter's subtopics and image list.

## What this book is
ARRA Relic Scan (`relic-scan`, a free AGPL tool by Soul-Brews-Studio: https://github.com/Soul-Brews-Studio/relic4-scan) finds every
AI-agent transcript on a machine (Claude Code, Codex, omp and others), indexes them into one Parquet file per Bangkok month under
`~/.relic-scan`, and lets you browse them in a local web page (127.0.0.1), a terminal UI, and DuckDB. Every command in this book was run
for real on 2026-10-26 on version `26.10.26-alpha.516`, on a DEMO HOME of made-up repositories (`make-demo-home.py`: 144 Claude/Codex/omp
transcripts, nothing real), so `~` in a capture is the demo home. Outputs are in `captures/NN-slug.txt`, rendered as `images/NN-slug.png`;
page screenshots are `images/W*.png` (real app pages of the demo), terminal-UI screens are `images/3N-tui-*.png`. Author = Claude (an AI)
with Nat. The cover art is the only AI-drawn picture.

## Style = /browser-tutorial, in Thai
- Numbered steps inside each section: `### ขั้นที่ N — <what it does>`, N restarting at 1 per `##`.
  Each step: 1–3 sentences telling the reader what to type and why → the command in a ```bash block
  → the image of the REAL output directly under it → 1–3 sentences reading the output.
- **Prose carries the instruction; the image confirms it.** A reader with no images must still
  follow. Quote the 2–6 most important output lines in a plain ``` block when the point is a number.
- Image: `![<what is IN the shot, Thai ok>](images/NN-slug.png)` — relative path, only names that
  exist in `images/` (list below). Never invent an image name.
- In each step, the ONE key idea in **bold** — one span per step.
- When something fails or surprises, that IS the lesson: keep it, explain the cause, give the fix.
- Diagram cards go where the concept is introduced, with one sentence on what the picture shows
  that prose did not.
- **Pace: every printed page carries 1–2 images (3 when all three are small captures).** In
  practice: no more than ~250 Thai words (or one long code block) between two images. Pair short
  captures in one step. A section that would be prose-only gets the capture behind its main claim,
  or is cut. `make check` fails on a text-only page (`page_images.py`).

## Language — Thai, kien-thai frames
Natural Thai for a developer colleague next to the terminal. Commands, flags, file names, code,
numbers and product names stay in English/ASCII. f1 topic-comment · f2 condition first (พอ...ก็...)
· f3 short clauses · f4 particles where natural · f5 drop pronouns the context carries · f6 ก็ as
rhythm · f7 pivot with a question or แต่. No English sentences in prose. No emoji. No "!!!".

## Hard rules
- **Never invent output, numbers, flags or behaviour.** Every number must appear in a capture file
  or in FACTS below. Open the capture `.txt` files and read them.
- ```bash for commands, untagged for output. Lines ≤ 76 chars (wrap with ` \`, cut output with `…`);
  verbatim source excerpts stay verbatim.
- Paragraphs ≤ ~500 characters.
- File starts `# บทที่ N: <title>` (frontmatter `# คำนำ`). Sections `##`, steps `###`.
- End with a short forward hook — no recap list.
- Read your captures with ONE call: `python3 verify.py --bundle <your-chapter>.md` (prints every capture and
  diagram the OUTLINE lists for you). Never open a PNG. Never run make, typst or pdftoppm: the lead builds.
  Stop at ~60 tool calls. `python3 verify.py` (0.1 s) checks your numbers, ids and times against the captures.
- **WRITE THE FILE YOURSELF**. Return ONLY: `wrote NN-slug.md — <words> words, <n> images`.

## Images available (exact names)
- `01-help.png`
- `02-version.png`
- `03-demo-home.png`
- `04-dry-run.png`
- `05-run-first.png`
- `06-out-folder.png`
- `07-run-again.png`
- `08-run-grown.png`
- `09-log.png`
- `10-seals.png`
- `11-seals-verify.png`
- `12-duck-total.png`
- `13-duck-what.png`
- `14-duck-engine.png`
- `15-duck-repos.png`
- `16-duck-tokens.png`
- `17-duck-session.png`
- `18-duck-read.png`
- `19-duck-redacted.png`
- `20-ops.png`
- `21-vaults.png`
- `22-months-json.png`
- `23-scans-jsonl.png`
- `24-kind-one.png`
- `25-month-one.png`
- `30-tui-overview.png`
- `31-tui-repos.png`
- `32-tui-repo-sessions.png`
- `33-tui-sessions.png`
- `34-tui-filter.png`
- `35-tui-popup.png`
- `36-tui-jsonl.png`
- `37-tui-ops.png`
- `C1-smart-scan-donut.png`
- `C2-tree-rows-by-type.png`
- `W10-page-scan.png`
- `W11-page-scans.png`
- `W12-page-tree.png`
- `W13-page-map.png`
- `W14-page-paths.png`
- `W15-page-disk.png`
- `W16-page-vaults.png`
- `W17-page-banks.png`
- `W18-page-export.png`
- `W19-page-ops.png`
- `W20-page-health.png`
- `W21-page-settings.png`

- `W10-page-scan.png` Smart Scan (page /scan): dark page, left menu (Smart Scan, Banks & Repos, ψ Vaults, Health, Scan log, Activity, Session map > Tree, Paths, Disk; Export, Settings); a donut of rows by type with the total 3,173 in the middle (tool calls, user text, AI text, tool results, thinking, tool errors, with counts and percentages), then tiles ROWS, SESSIONS, FILES INDEXED, NEWEST ROW, SPAN.
- `C1-smart-scan-donut.png` a crop of that donut and its legend.
- `W11-page-scans.png` Scan log (/scans): a table, newest first: WHEN, JOB (index or check), NEW, GROWN, REWRITTEN, GONE, READ, TIME, RESULT (+2 rows, +0 rows, 3,171 rows, would read…); 4 entries.
- `W12-page-tree.png` Tree (/tree): homes -> repos -> sessions: ~/.claude with 6 repos (tea-cart, starfish-api, harbor-bot, lantern-app, kite-docs, moss-cli) and its session and row counts, then .codex and .omp; right panel: ALL SESSIONS totals, ROWS BY TYPE bars, REPOS list, JSONL HOMES.
- `C2-tree-rows-by-type.png` a crop of that right panel.
- `W13-page-map.png` Session map (/map): a round star map; every session is a dot; the rings are months (2026-07 to 2026-10), the spokes are repos (moss-cli, codex, tea-cart, harbor-bot, lantern-app, omp, starfish-api, kite-docs, each with its session count); buttons at the bottom pick the dot colour: tier, repo, age, tools%.
- `W14-page-paths.png` Paths (/paths): a folder tree of every .jsonl the scan saw (one collapsed folder, 145 files, 3,173 rows); right panel: STATUS (indexed 144, skipped 0, vault 0, loose 1), VERDICT (content sniff: transcript-shaped 144, not-transcript 1), BUSIEST FOLDERS.
- `W15-page-disk.png` Disk (/disk): a sunburst chart of every .jsonl by folder and size, 1.5 MB in the centre; click a slice to go in, the centre to go back; legend: .claude/projects 92.8%, .codex/sessions 2.9%, .relic-scan 2.7%, .omp 1.6%.
- `W16-page-vaults.png` ψ Vaults (/vaults): tiles VAULTS 2, NOTES 5, SIZE 401 B, NEWEST NOTE 2026-10-26; two vaults listed (kite-docs 3 notes, moss-cli 2 notes) with a coloured bar and the path ~/Code/kite-docs/ψ.
- `W17-page-banks.png` Banks & Repos (/banks): a card per agent home (.claude, .codex, .omp) with sessions, rows, files, size, newest session; below, a Repository index (8) with sessions and rows per repo.
- `W18-page-export.png` Export (/export): Metadata table (sessions 144 lines, files 145, homes 3, scans 4, ψ vaults 2) each with CSV and JSON buttons; an S3 copy panel: destination not set, keys missing, buttons Check against S3 and Send sealed months (disabled).
- `W19-page-ops.png` Activity (/ops): the Relic chain card (idle, last pass, five stages scan / extract / push to the hub / hub index + embed / sync check with times), a Processes card (none running), a Hub card (not read, off).
- `W20-page-health.png` Health (/health): All clear; tiles ROWS 3,173, SESSIONS 144, MONTHS 4, WARNINGS 0; a Checks list with green ticks (every folder could be read; last run just now; no transcript outside an agent home; every session has a timestamp; every indexed file produced rows) and info lines.
- `W21-page-settings.png` Settings (/settings): What to scan: folders to walk (~), skip paths, depth 8; S3 copy: destination, endpoint, region; keys are never kept here.


## FACTS measured on 2026-10-26 (use these; do not add others)
- Version `relic-scan 26.10.26-alpha.516` (capture 02). Demo home: 144 Claude Code transcripts + omp + codex files = 145 .jsonl seen, 144 indexed (captures 04, 05; page W14).
- A real Mac, for scale only (AGENTS.md): 87,000 files, 75 GB, 5.4 M rows indexed cold in about a minute. Do not claim more.
- Page numbers (screenshots, same demo run): rows 3,173, sessions 144, months 4 (2026-07 to 2026-10), 8 repos listed, 3 homes.
- Models in the demo data: claude-opus-5-5, claude-sonnet-5-5, claude-haiku-5-5 (Claude), gpt-6-astra (Codex), glm-5.3 (omp); all made up. `<synthetic>` is a Claude Code placeholder that real data can hold; the demo has none.
- Codex rows carry a model but no msg_id and no tokens (so `sum(out_tokens)` is NULL for them: capture 16).
- Ports: `relic-scan serve` / `up` use 8787 by default; the demo pages were served on 8791 (`--port`).
- Install (from AGENTS.md, not a capture, so use an UNTAGGED block): `curl -fsSL https://raw.githubusercontent.com/Soul-Brews-Studio/relic4-scan/main/install.sh | sh` builds in about 30 s and installs `~/.local/bin/relic-scan`; needs Rust 1.89 or newer. Desktop app downloads: the GitHub Releases page.
- Rules the code keeps (AGENTS.md): transcripts and vaults are read only; relic-scan writes only inside its output folder; the page makes no network calls; the only network use is `export --to s3://…` and the hub status GET of `ops`; server writes need an X-Relic header.
- Sealed months: before the current Bangkok month; the month file is made read-only; `seals --verify` reads each sealed month again and checks it (AGENTS.md; captures 10, 11).
- Key columns: see AGENTS.md section "Query the index". sessions.parquet gained models, model_msgs, in_tokens, out_tokens, cache_read_tokens, cache_write_tokens (0 where the transcript records none); months written before them read NULL for in/cache tokens until their files are read again.
- The ledger (AGENTS.md "The ledger"): `_ledger.tsv` is appended on every index, never rewritten; answers same / update / past changed / never.
- Demo transcripts were made by `book/make-demo-home.py` (fixed seed); the one planted secret is a fake `sk-demo-…` key in one prompt (capture 19 shows it redacted).
