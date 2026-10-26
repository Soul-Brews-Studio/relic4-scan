#!/usr/bin/env python3
"""Gate: every content page carries 1-2 images, or 3 when all three are small.

A proof book shows, it does not only tell: a page of prose with no capture under it is a page
the reader has to take on trust. Measured on the first books: the S3 book had 8 text-only
content pages out of 41 and not one page with 3 small captures side by side.

    python3 page_images.py book.pdf [--small 2.5] [--exempt 1,2] [--warn]

Reads the PDF with poppler (`pdfimages -list`, `pdftotext`), so it checks the LAID-OUT book,
not the markdown. Per page:
  0 images              FAIL (text-only page)
  1-2 images            ok
  3 images              ok only if every one is "small" (rendered height <= --small inches)
  4+ images             FAIL (too crowded)
Exempt: page 1 (cover), any page whose text holds the TOC heading, plus --exempt pages.
--warn reports without failing (for a first look at an existing book).
"""
import argparse
import subprocess
import sys

TOC_MARKERS = ("สารบัญ", "Contents")


def run(*cmd):
    return subprocess.run(cmd, capture_output=True, text=True, check=True).stdout


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("pdf")
    ap.add_argument("--small", type=float, default=2.5, help="max rendered height (in) of a 'small' image")
    ap.add_argument("--exempt", default="", help="extra page numbers to skip, comma-separated")
    ap.add_argument("--warn", action="store_true", help="report only, exit 0")
    ap.add_argument("--code-pages-ok", action="store_true",
                    help="a text-only page that is mostly verbatim source counts as evidence (full-code books)")
    a = ap.parse_args()

    pages = int(next(l.split()[1] for l in run("pdfinfo", a.pdf).splitlines() if l.startswith("Pages:")))
    heights = {}  # page -> [rendered height in inches]
    for line in run("pdfimages", "-list", a.pdf).splitlines()[2:]:
        f = line.split()
        if len(f) >= 14 and f[2] == "image":
            page, h_px, y_ppi = int(f[0]), int(f[4]), float(f[13])
            heights.setdefault(page, []).append(h_px / y_ppi if y_ppi else 99.0)

    exempt = {1} | {int(x) for x in a.exempt.split(",") if x.strip()}
    code_pages = set()
    for p in range(1, pages + 1):
        text = run("pdftotext", "-f", str(p), "-l", str(p), a.pdf, "-")
        if any(m in text for m in TOC_MARKERS) and p <= 4:
            exempt.add(p)
        # Source code is ASCII; this book's prose is Thai. A page whose lines are mostly ASCII
        # is a page of verbatim code — the code IS the evidence there (Nat: "full code").
        lines = [l for l in text.splitlines() if l.strip() and not l.strip().isdigit()]
        ascii_lines = [l for l in lines if all(ord(c) < 0x2500 for c in l) and not any("\u0e00" <= c <= "\u0e7f" for c in l)]
        if a.code_pages_ok and len(ascii_lines) >= 12 and len(ascii_lines) >= 0.6 * len(lines):
            code_pages.add(p)

    bad, dist = [], {}
    for p in range(1, pages + 1):
        hs = heights.get(p, [])
        dist[len(hs)] = dist.get(len(hs), 0) + 1
        if p in exempt:
            continue
        if not hs and p in code_pages:
            continue
        if not hs:
            bad.append(f"page {p}: 0 images (text-only)")
        elif len(hs) == 3 and max(hs) > a.small:
            bad.append(f"page {p}: 3 images but one is {max(hs):.1f} in tall (> {a.small} in = not small)")
        elif len(hs) > 3:
            bad.append(f"page {p}: {len(hs)} images (max 3, and only when small)")

    summary = " · ".join(f"{k} img: {v} p" for k, v in sorted(dist.items()))
    code_only = sorted(p for p in code_pages if not heights.get(p) and p not in exempt)
    if code_only:
        summary += f" · code-only pages (allowed, --code-pages-ok): {','.join(map(str, code_only))}"
    if not bad:
        print(f"ok: every content page has 1-2 images (3 when small) — {summary}")
        return 0
    print(f"{'WARN' if a.warn else 'FAIL'}: {len(bad)} page(s) break the image rule — {summary}")
    for b in bad:
        print("  " + b)
    print("  fix: move an image up, pair small captures, add the capture behind a claim, or trim prose")
    return 0 if a.warn else 1


if __name__ == "__main__":
    sys.exit(main())
