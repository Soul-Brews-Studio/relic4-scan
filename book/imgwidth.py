#!/usr/bin/env python3
"""Give every image a width proportional to its pixel width, so terminal text prints at ONE size.

The cards are rendered at a fixed advance per column (render_png.py), so a 96-column
capture is 1608 px wide and a 7-line `relic banks` capture is ~500 px. With
`#set image(width: 100%)` alone, the narrow one was stretched 3x and its text printed huge
(page 9 of the first full build). Scale each to px / FULL of the text width instead.
    python3 imgwidth.py build/body.typ      (in place; run from the book dir)
"""
import re, sys, pathlib
from PIL import Image

FULL = 1608          # px of a 96-column card at the default --size 26 = 100% text width
path = pathlib.Path(sys.argv[1])
src = path.read_text(encoding="utf-8")

def sized(m):
    rel = m.group(1)                                   # "/images/x.png"
    w = Image.open(pathlib.Path("." + rel)).size[0]
    pct = max(35, min(100, round(w / FULL * 100)))     # never below 35%: stays readable
    return f'image("{rel}", width: {pct}%'            # pandoc's own args (alt: ...) follow

out, n = re.subn(r'image\("(/images/[^"]+\.png)"', sized, src)
path.write_text(out, encoding="utf-8")
print(f"imgwidth: {n} images sized in {path}")
