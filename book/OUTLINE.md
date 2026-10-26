---
title: "ARRA Relic Scan"
subtitle: "คู่มือทีละขั้น: จากทรานสคริปต์ของเอเจนต์ ไปถึงดัชนีที่ถามได้"
tagline: "ทุกคำสั่งรันจริง ทุกภาพคือแอปจริง บนบ้านเดโมที่ทุกอย่างเป็นของสมมติ — 2026-10-26"
author: Claude (AI) · กับ Nat
date: 2026-10-26
language: Thai (kien-thai 7 frames) — commands/flags stay English
style: /app-guide-book — numbered steps, image under the command or screen that produced it
images: terminal captures rendered from text (captures/ -> images/NN-slug.png); page screenshots W*.png; TUI screens 3N-tui-*.png
target: 9 chapters + protocol chapter + code appendix, <=100 pages
parts: 3
---

## 00 คำนำ — วิธีอ่านเล่มนี้  (written by the lead)

## ภาค 1 — ใช้งาน

### บทที่ 1: ติดตั้งและรันครั้งแรก  (01-intro.md; captures 01-help 02-version 03-demo-home 04-dry-run 05-run-first 06-out-folder)
- 1.1 relic-scan คืออะไร และบ้านเดโมที่ใช้ตลอดเล่ม (ls -a ~ shows .claude .codex .omp Code .relic-scan)
- 1.2 ดูคำสั่งทั้งหมด (help), เวอร์ชัน
- 1.3 `run --dry-run` ดูก่อนว่าจะอ่านอะไร ไม่เขียนอะไร
- 1.4 `run` ครั้งแรก: อ่านบรรทัดผลทีละบรรทัด (scan, parse, merge, meta, vaults, map, done) และกล่อง RELIC·SCAN
- 1.5 อะไรอยู่ใน ~/.relic-scan (month parquet, meta/, _state.tsv, _ledger.tsv, _scans.jsonl)

### บทที่ 2: รันซ้ำ อ่านแค่ของใหม่  (02-again.md; captures 07-run-again 08-run-grown 09-log 24-kind-one 25-month-one; image W11-page-scans)
- 2.1 รันซ้ำทันที: new 0 (ไม่อ่านอะไรเลย)
- 2.2 transcript โตขึ้น 1 ไฟล์: grown 1, อ่านเฉพาะบรรทัดท้าย ("2 raw rows", ledger line)
- 2.3 `relic-scan log` และหน้า Scan log
- 2.4 `--kind codex` และ `--month 2026-10`: รันแค่ส่วนหนึ่ง เขียนลงโฟลเดอร์แยก (read the two captures for the folder names)

### บทที่ 3: เปิดดูในเว็บ  (03-web.md; images W10-page-scan C1-smart-scan-donut W12-page-tree C2-tree-rows-by-type W13-page-map W14-page-paths)
- 3.1 `relic-scan up` / `serve` (address 127.0.0.1; the page makes no network calls; see AGENTS.md)
- 3.2 Smart Scan: ดูภาพ
- 3.3 Tree: บ้าน -> repo -> session, แผงขวา rows by type
- 3.4 Map: ดาวหนึ่งดวงต่อหนึ่ง session
- 3.5 Paths: ทุก .jsonl ที่สแกนเห็น

### บทที่ 4: ดิสก์ ห้องนิรภัย และการตั้งค่า  (04-more-pages.md; images W15-page-disk W17-page-banks W16-page-vaults W20-page-health W21-page-settings; capture 21-vaults)
- 4.1 Banks & Repos
- 4.2 Disk
- 4.3 ψ Vaults + capture 21-vaults (the same data as JSON)
- 4.4 Health
- 4.5 Settings (roots, skip paths, depth, S3 copy)

### บทที่ 5: ในเทอร์มินัล  (05-tui.md; captures 30-tui-overview 31-tui-repos 32-tui-repo-sessions 33-tui-sessions 34-tui-filter 35-tui-popup 36-tui-jsonl 37-tui-ops)
- 5.1 `relic-scan tui`: หกแท็บ
- 5.2 repos -> sessions
- 5.3 กรองด้วย `/`, เปิดรายละเอียดด้วย Enter, `o` เปิดหน้าเว็บ, `--mouse`
- 5.4 แท็บ paths และ ops

## ภาค 2 — อ่านดัชนี

### บทที่ 6: ถามดัชนีด้วย DuckDB  (06-duckdb.md; captures 12-duck-total 13-duck-what 14-duck-engine 15-duck-repos 16-duck-tokens 17-duck-session 18-duck-read 19-duck-redacted 22-months-json 23-scans-jsonl)
- 6.1 นับทุกอย่างก่อน: total, แล้วแยกตามชนิด (role/kind), ตาม engine
- 6.2 sessions ต่อ repo
- 6.3 โมเดลและโทเคนต่อเดือน (model, msg_id, out_tokens; Codex rows have a model but no tokens)
- 6.4 โทเคนต่อ session (models, model_msgs, in/out/cache columns of meta/sessions.parquet)
- 6.5 อ่าน session ตามลำดับ
- 6.6 บรรทัดที่ถูก redact (red column)
- 6.7 months.json และ _scans.jsonl

### บทที่ 7: เดือนที่ปิดผนึก และสำเนาไป S3  (07-seals.md; captures 10-seals 11-seals-verify 22-months-json; images W18-page-export W21-page-settings)
- 7.1 เดือนที่ปิดแล้วเป็นไฟล์อ่านอย่างเดียว: `seals`, `seals --verify`
- 7.2 หน้า Export: CSV/JSON ของ metadata, S3 copy (Check against S3, Send sealed months)
- 7.3 ส่งไป S3 จากบรรทัดคำสั่ง (from AGENTS.md: --dry-run, --verify, --list, --overwrite; no capture exists, so plain text blocks only, NOT ```bash)

## ภาค 3 — ข้างใน

### บทที่ 8: ทางเดินของข้อมูลข้ามเครื่อง  (08-chain.md; captures 20-ops 37-tui-ops; image W19-page-ops)
- 8.1 `relic-scan ops`: ขั้นของ chain, ETA, process, hub
- 8.2 หน้า Activity
- 8.3 chain -> relic-extract -> hub (from docs/TUTORIAL.md and AGENTS.md: what each step is; no numbers beyond captures)
- 8.4 ตรวจว่า sync ตรงกัน: the SYNC OK line (capture 20-ops shows one)

### บทที่ 9: ความเป็นส่วนตัว และข้อควรระวัง  (09-privacy.md; captures 19-duck-redacted 11-seals-verify 04-dry-run)
- 9.1 อ่านอย่างเดียว: ไม่แก้ transcript, เขียนเฉพาะใน ~/.relic-scan, ไม่เรียกเน็ต (except export, ops hub GET)
- 9.2 redact: คีย์ลับถูกแทนด้วย [openai-key:…] (capture 19)
- 9.3 สำเนาซ้ำถูกพับเป็นแถวเดียว (sources list)
- 9.4 ปัญหาที่เจอจริง and the fix (from README/AGENTS/docs only; label each as such)

## ภาคผนวก

### บทที่ 10: รูปแบบของ transcript ที่ relic-scan อ่าน  (10-formats.md; read ../src/engine.rs ../src/meta/rules.rs ../src/meta/agg.rs; no captures)
- 10.1 Claude Code jsonl: type, message.role, content blocks (text, thinking, tool_use, tool_result), usage
- 10.2 Codex rollout: session_meta, turn_context, response_item
- 10.3 omp: message.role, responseId, usage.input/output/cacheRead/cacheWrite
- 10.4 how a line becomes rows (text, thinking, tools streams; types_of in agg.rs)
- 10.5 how sessions are told apart (tier_of / cat_of rules, short excerpts verbatim from rules.rs)

### บทที่ 11: โค้ดที่เลือก (generated by code-appendix.py -> 11-code.md)
