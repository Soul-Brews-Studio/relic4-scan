#!/usr/bin/env python3
"""Write 11-code.md: the source files the book explains, verbatim, one fenced block each.
    python3 code-appendix.py        (from the book dir)
relic-scan is public (AGPL v3): the whole crate is at https://github.com/Soul-Brews-Studio/relic4-scan ; this appendix prints the parts
the chapters point at."""
import pathlib, subprocess

REPO = pathlib.Path("..")
FILES = [
    ("src/engine.rs", "rust", "how one transcript line becomes rows: the Claude, Codex and omp readers, redaction, usage"),
    ("src/meta/rules.rs", "rust", "which file is which session: tier_of, cat_of, proj_of and the path rules"),
    ("src/meta/agg.rs", "rust", "the per-month pass that counts rows by type and sums tokens per session, and its cache"),
    ("src/ops/text.rs", "rust", "the text view of `relic-scan ops`"),
]
commit = subprocess.run(["git", "-C", str(REPO), "rev-parse", "--short", "HEAD"], capture_output=True, text=True).stdout.strip()
out = ["# ภาคผนวก: โค้ดที่เลือก", "",
       f"ไฟล์ด้านล่างพิมพ์ตามจริงทุกบรรทัด จาก commit `{commit}` ของ Soul-Brews-Studio/relic4-scan ไม่ย่อ ไม่แก้ให้สวย "
       "เลือกเฉพาะส่วนที่บทต่าง ๆ ชี้ไปถึง", "",
       "> relic-scan เป็นโอเพนซอร์ส (AGPL v3) โค้ดทั้งชุดอยู่ที่ https://github.com/Soul-Brews-Studio/relic4-scan", ""]
total = 0
for rel, lang, what in FILES:
    p = REPO / rel
    if not p.exists():
        raise SystemExit(f"missing {p}")
    text = p.read_text(encoding="utf-8").rstrip("\n")
    n = text.count("\n") + 1; total += n
    fence = "````" if "```" in text else "```"
    out += [f"## {rel}", "", f"{what} · {n} บรรทัด", "", f"{fence}{lang}", text, fence, ""]
pathlib.Path("11-code.md").write_text("\n".join(out), encoding="utf-8")
print(f"11-code.md: {len(FILES)} files, {total} lines, commit {commit}")
