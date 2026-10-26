#!/usr/bin/env python3
"""Keep CLI flags intact in typst markup: `--since` must not become `–since`.

typst reads `--` as an en dash and `---` as an em dash anywhere in markup. pandoc passes
prose, headings and figure captions (the images' alt text) through as markup, so every
flag written outside backticks was printed as `–since` — measured on the first trial
build: 32 flags mangled, 16 of them `–since`. A reader copying one gets a broken command.

Escape each `-` of a run of two or more as `\\-`, everywhere EXCEPT inside raw spans
(``` blocks and `inline`), where a backslash would be printed literally, and inside
string literals such as image("...").
    python3 fixdash.py build/body.typ      (in place)
"""
import re, sys

path = sys.argv[1]
src = open(path, encoding="utf-8").read()
# raw blocks (```...```), inline raw (`...`), and "string" literals are left untouched
# also typst labels <...> that pandoc writes after headings: a label cannot hold a backslash
PROTECT = re.compile(r"```.*?```|`[^`\n]*`|\"(?:[^\"\\\n]|\\.)*\"|<[^<>\s\\]+>", re.S)
out, last, n = [], 0, 0
for m in PROTECT.finditer(src):
    chunk = src[last:m.start()]
    fixed, k = re.subn(r"-{2,}", lambda r: "\\-" * len(r.group(0)), chunk)
    out += [fixed, m.group(0)]; n += k; last = m.end()
fixed, k = re.subn(r"-{2,}", lambda r: "\\-" * len(r.group(0)), src[last:])
out.append(fixed); n += k
open(path, "w", encoding="utf-8").write("".join(out))
print(f"fixdash: {n} dash runs escaped in {path}")
