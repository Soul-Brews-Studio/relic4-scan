#!/usr/bin/env python3
"""Page screenshots (2560x1720) -> images/W<name>.png at 1608 px wide (= 100% of the text width in imgwidth.py), plus a few crops.
    uv run --quiet --with pillow python compose.py        (from the book dir)"""
from PIL import Image
import pathlib
IMG = pathlib.Path("images")
for p in sorted(IMG.glob("[12][0-9]-page-*.png")):
    im = Image.open(p).convert("RGB")
    im = im.resize((1608, int(im.height * 1608 / im.width)), Image.LANCZOS)
    im.save(IMG / f"W{p.name}", optimize=True); print(f"W{p.name}", im.size)

def crop(out, name, box, w=None):   # box in the 2560x1720 shot: (left, top, right, bottom)
    im = Image.open(IMG / name).convert("RGB").crop(box)
    if w: im = im.resize((w, int(im.height * w / im.width)), Image.LANCZOS)
    im.save(IMG / out, optimize=True); print(out, im.size)

crop("C1-smart-scan-donut.png", "10-page-scan.png", (480, 380, 2560, 1050), 1608)      # donut + legend
crop("C2-tree-rows-by-type.png", "12-page-tree.png", (1840, 0, 2560, 1000), 900)       # right panel: rows by type
