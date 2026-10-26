#!/usr/bin/env python3
"""captures/*.txt -> build/cards/*.txt (terminal cards the /ascii-diagram renderer accepts).

A card is the capture as printed, made safe for render_png.py:
  - ANSI escapes stripped; \\r progress lines collapse to their last frame
  - double-width characters (emoji, CJK) -> ASCII stand-ins, since the renderer refuses them;
    ✓ ✗ → ⇄ ▫ ○ stay as printed (the renderer draws them, and the prose names them)
  - lines wider than 96 columns WRAP at a space: the cut ends " …", the rest goes on at the
    same indent after "… " — nothing printed is lost (cutting hid the end of 20 lines in the
    herdr-board book, 2026-10-09); at most MAX lines, then "… N more lines"
Then images/NN-slug.png = render_png.py(card). The text in captures/ stays the source of truth.
"""
import re, sys, unicodedata, pathlib, subprocess

MAX = int(sys.argv[1]) if len(sys.argv) > 1 else 42
ROOT = pathlib.Path(__file__).parent
SUB = {"🏺": "*", "⚠": "!", "⏳": "~", "📍": "@",
       "\u0e00": "\\u0e00", "\u0e7f": "\\u0e7f"}   # Thai block ends: no glyph in any font
WIDTH = 96
ANSI = re.compile(r"\x1b\[[0-9;?]*[A-Za-z]")

def safe(line: str) -> str:
    for k, v in SUB.items():
        line = line.replace(k, v)
    out = []
    for ch in line:
        if unicodedata.east_asian_width(ch) in "WF":
            out.append("?")
        elif unicodedata.category(ch) == "Cf" and ch != "​":
            continue
        else:
            out.append(ch)
    return wrap("".join(out).replace("\t", "    "))

def wrap(s: str) -> list[str]:
    if len(s) <= WIDTH:
        return [s]
    if s.lstrip()[:1] in "│╭╰├┌└┐┘┤╮╯┬┴┼─╞╪╡":      # a box or table line: wrapping breaks the frame, so cut it (the .txt keeps it all)
        return [s[:WIDTH - 1] + "…"]
    lead = re.match(r"\s*", s).group(0)
    out = []
    while len(s) > WIDTH:
        cut = s.rfind(" ", len(lead) + 12, WIDTH - 2)   # room for " …"
        if cut == -1:
            cut = WIDTH - 2
        out.append(s[:cut].rstrip() + " …")
        s = lead + "… " + s[cut:].lstrip()
    return out + [s]

(ROOT / "build/cards").mkdir(parents=True, exist_ok=True)
names = sys.argv[2:] or sorted(p.name for p in (ROOT / "captures").glob("*.txt"))
for name in names:
    WIDTH = 100      # the renderer's ceiling; a terminal-UI screen is exactly 100 columns wide
    src = ROOT / "captures" / name
    text = ANSI.sub("", src.read_text(encoding="utf-8", errors="replace"))
    lines = [l.split("\r")[-1] for l in text.split("\n")]
    lines = [piece for l in lines for piece in safe(l)]
    while lines and not lines[-1].strip():
        lines.pop()
    if len(lines) > MAX:
        tail = lines[-1]                      # keep "(exit N · T s)"
        lines = lines[:MAX - 2] + [f"  … {len(lines) - MAX + 1} more lines in captures/{name}", tail]
    card = ROOT / "build/cards" / name
    card.write_text("\n".join(lines) + "\n", encoding="utf-8")
    png = ROOT / "images" / (src.stem + ".png")
    r = subprocess.run(["uv", "run", "--quiet", "--with", "pillow", "python",
                        str(pathlib.Path.home() / ".claude/skills/ascii-diagram/render_png.py"),
                        str(card), str(png)], capture_output=True, text=True)
    print((r.stdout or r.stderr).strip().splitlines()[-1] if (r.stdout or r.stderr) else f"{png.name}: ?")
