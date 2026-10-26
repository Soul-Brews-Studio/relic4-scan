#!/usr/bin/env python3
"""verify.py — the mechanical half of the review, in about 2 seconds and 0 LLM tokens.

  python3 verify.py                   # check every NN-*.md chapter, write REVIEW-auto.md, print it
  python3 verify.py --for 02-join.md  # only the findings that name that chapter (what a fixer reads)
  python3 verify.py --bundle 02-join.md   # the captures + diagrams of that chapter in ONE output
                                          # (what a writer reads: one call instead of 20 Reads)

Rules (all cheap, all deterministic):
  code    a ```bash/zsh block must be a command that ran (found in some capture, whitespace-collapsed,
          " \\" continuations unwrapped); lines <= 76 chars; tags only bash, zsh or none; zsh builtins
          (print, &!) need ```zsh; an untagged output block must quote a capture (a "…" cuts a line)
  token   every pane id, tab id, pid (5-6 digits), conversation id (8 hex), commit (7 hex),
          broadcast id, time, WxH size and 0.xxx ratio in a chapter must appear in captures/,
          diagrams/ or BRIEF.md — an unmatched token is either a typo, an invented fact, or a fact
          that belongs in BRIEF FACTS
  struct  `### ขั้นที่ N` / `### กรณีที่ N` restart at 1 under each `##`; one **bold** span per step;
          paragraph <= 500 chars; no bold inside code
  image   referenced file exists; used once; alt text present; sits under a code block or a diagram;
          images/ files nobody placed
  prose   no emoji; no long English sentence in Thai prose (8+ ASCII words in a row, outside code)
  secret  token-, key-, password- and e-mail-shaped strings anywhere

What this does NOT judge: whether a claim about what happened is true, the Thai register, and
whether two chapters repeat each other. That is the one short LLM pass (SKILL.md step 6b).
"""
import glob, os, re, sys

HERE = os.path.dirname(os.path.abspath(__file__))
os.chdir(HERE)
OUT = "REVIEW-auto.md"
FENCE = re.compile(r"^(\s*)(`{3,})(\w*)\s*$")
ZSH_ONLY = re.compile(r"(^|\s)print(\s|$)|&!|\$\{\(f\)|\$\{\(@")
EMOJI = re.compile("[\U0001F300-\U0001FAFF☀-⛿✀-➿]")
SECRET = [re.compile(p) for p in (r"sk-[A-Za-z0-9]{16,}", r"ghp_[A-Za-z0-9]{20,}", r"AKIA[0-9A-Z]{16}",
                                  r"Bearer [A-Za-z0-9._-]{20,}", r"(?i)password\s*[:=]\s*\S{4,}",
                                  # an e-mail; an ssh target on a tailnet (user@machine.<tailnet>.ts.net) is a
                                  # machine name, not an address (herdr-board book, 2026-10-09)
                                  r"[A-Za-z0-9._%+-]+@(?![A-Za-z0-9-]+\.[A-Za-z0-9-]+\.ts\.net\b)[A-Za-z0-9-]+\.[A-Za-z]{2,}")]
TOKENS = {
    "pane":   re.compile(r"\bw[0-9A-Za-z]{1,4}:[pt][0-9A-Za-z]{1,3}\b"),
    "pid":    re.compile(r"(?<![\w.:/-])\d{5,6}(?![\w.:/-])"),
    "conv":   re.compile(r"\b(?=[0-9a-f]*\d)[0-9a-f]{8}\b"),
    "commit": re.compile(r"\b(?=[0-9a-f]*\d)(?=[0-9a-f]*[a-f])[0-9a-f]{7}\b"),
    "bcast":  re.compile(r"\bsb-\d{4}-\d{6}\b"),
    "time":   re.compile(r"(?<![\w:.])\d{1,2}:\d{2}(?::\d{2})?(?![\w:])"),
    "size":   re.compile(r"\b\d{1,3} ?[x×] ?\d{1,3}\b"),
    "ratio":  re.compile(r"(?<![\w.])0\.\d{2,3}\b"),
}


def read(p):
    try:
        return open(p, encoding="utf-8", errors="replace").read()
    except OSError:
        return ""


def unwrap(body):
    """join continuation lines the way a shell does: a backslash-newline vanishes, so
    'a'\\<newline>'b' is ONE quoted string and wt/\\<newline>x is ONE path"""
    body = re.sub(r"'\\\n'|\"\\\n\"", "", body)
    return re.sub(r"\\\n", "", body)


def collapse(t):
    return re.sub(r"\s+", " ", t).strip()


def blocks_and_prose(text):
    """-> list of (kind, lang, start_line, [lines]) with kind 'code' or 'prose'"""
    out, cur, lang, start, fence = [], [], None, 0, None
    mode = "prose"
    for i, line in enumerate(text.split("\n"), 1):
        m = FENCE.match(line)
        if mode == "prose":
            if m:
                if cur:
                    out.append(("prose", "", start, cur))
                cur, lang, start, fence, mode = [], m.group(3), i + 1, m.group(2), "code"
            else:
                if not cur:
                    start = i
                cur.append(line)
        else:
            if m and m.group(2) == fence and not m.group(3):
                out.append(("code", lang, start, cur))
                cur, start, mode = [], i + 1, "prose"
            else:
                cur.append(line)
    if cur:
        out.append((mode, lang or "", start, cur))
    return out


def corpus():
    names = sorted(glob.glob("captures/*.txt")) + sorted(glob.glob("diagrams/*.txt")) + ["BRIEF.md"]
    raw = {n: read(n) for n in names}
    # a capture can hold a multi-line command with its own ` \` joins (close-job book, 2026-10-05,
    # capture 31): match against the text as the shell runs it too, or a faithful copy never matches
    joined = " ".join(raw.values())
    return raw, collapse(joined + " " + unwrap(joined)), [l for t in raw.values() for l in t.split("\n")]


def main():
    args = sys.argv[1:]
    if args and args[0] == "--bundle":
        return bundle(args[1])
    if args and args[0] == "--for":
        return only_for(args[1])
    check()


def check(quiet=False):
    raw, flat, lines = corpus()
    flat_nospace = flat.replace(" ", "")
    found = []                                # (file, line, rule, msg)
    tokens_unmatched = {}                     # (kind, token) -> [file:line]
    placed = []
    chapters = [c for c in sorted(glob.glob("[0-9][0-9]-*.md")) if c != "11-code.md"]   # the appendix is verbatim source
    for ch in chapters:
        text = read(ch)
        h2_step = None
        steps_seen = {}
        for kind, lang, start, blk in blocks_and_prose(text):
            if kind == "code":
                lang = (lang or "").lower()
                if lang not in ("", "bash", "zsh", "json", "rust"):   # json and rust: verbatim source and sample lines (chapters 10, 11)
                    found.append((ch, start - 1, "code", f"tag ```{lang}: use bash, zsh or none"))
                body = "\n".join(blk)
                for k, l in enumerate(blk):
                    if len(l) > 76:
                        found.append((ch, start + k, "code", f"{len(l)} chars (max 76): {l.strip()[:50]}"))
                if lang in ("bash", "zsh"):
                    norm = collapse(unwrap(body))
                    if lang == "bash" and ZSH_ONLY.search(body):
                        found.append((ch, start - 1, "code", "zsh builtin (print / &!) in a ```bash block: tag it ```zsh"))
                    frags = [f.strip() for f in norm.split("…") if len(f.strip()) >= 6]
                    if norm and not all(f in flat for f in frags):
                        miss = next((f for f in frags if f not in flat), norm)
                        sub = [l for l in (collapse(x) for x in unwrap(body).split("\n")) if len(l) >= 6]
                        if not all(l.split("…")[0].strip() in flat for l in sub):
                            found.append((ch, start - 1, "code", "command differs from its capture: " + diverge(norm, flat)))
                elif lang == "":
                    miss = []
                    for l in blk:
                        for f in (x.strip() for x in l.split("…")):
                            f = collapse(f)
                            if len(f) >= 8 and f not in flat and f.replace(" ", "") not in flat_nospace:
                                miss.append(f)
                    if miss:
                        found.append((ch, start, "code", f"{len(miss)} output line(s) not in a capture, first: {miss[0][:60]}"))
                scan = body
            else:
                scan = "\n".join(blk)
                for k, l in enumerate(blk):
                    s = l.strip()
                    if not s:
                        continue
                    if s.startswith("# "):
                        h2_step = None
                    m2 = re.match(r"^##\s", s)
                    if m2 and not s.startswith("###"):
                        h2_step = ("sec", start + k)
                        steps_seen = {"n": {}, "bold": 0, "line": start + k, "name": s}
                    m3 = re.match(r"^###\s+(ขั้นที่|กรณีที่)\s*(\d+)", s)
                    if s.startswith("###"):
                        if steps_seen.get("open") and steps_seen.get("bold", 0) != 1:
                            found.append((ch, steps_seen["open"], "struct", f"{steps_seen.get('bold', 0)} bold spans in this step (want 1)"))
                        steps_seen["open"], steps_seen["bold"] = start + k, 0
                        if m3:
                            word = m3.group(1)
                            steps_seen["n"][word] = steps_seen["n"].get(word, 0) + 1
                            if int(m3.group(2)) != steps_seen["n"][word]:
                                found.append((ch, start + k, "struct", f"{word} {m3.group(2)}, expected {steps_seen['n'][word]} under this ##"))
                        else:
                            found.append((ch, start + k, "struct", "### heading is not `ขั้นที่ N —` or `กรณีที่ N —`"))
                    elif steps_seen.get("open") and not s.startswith(("|", "![", "-", ">", "1.", "2.", "3.")):
                        steps_seen["bold"] = steps_seen.get("bold", 0) + len(re.findall(r"\*\*[^*\n]+\*\*", s))
                    if not s.startswith(("|", "![", "#", ">", "-", "*", "1.", "2.", "3.")) and len(s) > 500:
                        found.append((ch, start + k, "struct", f"paragraph {len(s)} chars (max ~500)"))
                    if EMOJI.search(l):
                        found.append((ch, start + k, "prose", f"emoji: {EMOJI.search(l).group(0)}"))
                    plain = re.sub(r"`[^`]*`|\[[^\]]*\]\([^)]*\)|https?://\S+", " ", s)
                    run = None if s.startswith(">") else re.search(r"(?:[A-Za-z][A-Za-z'’-]*[ ,]+){7,}[A-Za-z]+", plain)
                    if run and not re.search(r"[฀-๿]", run.group(0)):
                        found.append((ch, start + k, "prose", f"English sentence: {run.group(0)[:60]}"))
                    for im in re.finditer(r"!\[([^\]]*)\]\(([^)]+)\)", l):
                        alt, path = im.group(1), im.group(2)
                        placed.append((path, ch, start + k))
                        if not alt.strip():
                            found.append((ch, start + k, "image", f"empty alt text: {path}"))
                        if not os.path.exists(path):
                            found.append((ch, start + k, "image", f"missing file {path}"))
            for rule, rx in SECRET_RX():
                for mm in rx.finditer(scan):
                    found.append((ch, 0, "secret", f"{mm.group(0)[:40]}"))
            for kindname, rx in TOKENS.items():
                for mm in rx.finditer(scan):
                    tok = mm.group(0)
                    if token_ok(kindname, tok, flat, lines):
                        continue
                    ln = start + scan[:mm.start()].count("\n")
                    tokens_unmatched.setdefault((kindname, tok), []).append(f"{ch}:{ln}")
        if steps_seen.get("open") and steps_seen.get("bold", 0) != 1:
            found.append((ch, steps_seen["open"], "struct", f"{steps_seen.get('bold', 0)} bold spans in this step (want 1)"))
        # image under a code block or a diagram
        ls = text.split("\n")
        for i, l in enumerate(ls):
            if re.match(r"!\[", l.strip()):
                j = i - 1
                while j >= 0 and not ls[j].strip():
                    j -= 1
                prev = ls[j].strip() if j >= 0 else ""
                path = re.search(r"\(([^)]+)\)", l)
                is_diagram = path and ("/D" in path.group(1) or re.search(r"/[WC]\d", path.group(1)))   # a page screenshot (W*) or crop (C*) is its own evidence
                stem = os.path.splitext(os.path.basename(path.group(1)))[0] if path else ""
                is_record = read(f"captures/{stem}.txt").startswith("$ (")
                if not (prev.startswith("```") or prev.startswith("![") or is_diagram or is_record):
                    found.append((ch, i + 1, "image", "image is not directly under a code block"))
    paths = [p for p, _, _ in placed]
    for p in sorted(set(x for x in paths if paths.count(x) > 1)):
        where = ", ".join(f"{c}:{n}" for q, c, n in placed if q == p)
        found.append(("(book)", 0, "image", f"{p} used {paths.count(p)} times: {where}"))
    unplaced = [os.path.basename(p) for p in sorted(glob.glob("images/*.png"))
                if f"images/{os.path.basename(p)}" not in paths and "cover" not in p and not re.match(r"\d\d-page-", os.path.basename(p))]
    for (kindname, tok), where in sorted(tokens_unmatched.items(), key=lambda kv: kv[1][0]):
        found.append((where[0].split(":")[0], int(where[0].split(":")[1]), "token",
                      f"{kindname} `{tok}` is in no capture/BRIEF" + (f" (+{len(where) - 1} more: {', '.join(where[1:4])})" if len(where) > 1 else "")))
    report(found, unplaced, chapters, quiet)


def SECRET_RX():
    return [("secret", r) for r in SECRET]


def diverge(norm, flat):
    """where the longest-matching prefix of a block stops matching the capture text"""
    best, at = 0, 0
    head = norm[:24]
    i = flat.find(head)
    while i != -1:
        n = 0
        while n < len(norm) and i + n < len(flat) and norm[n] == flat[i + n]:
            n += 1
        if n > best:
            best, at = n, i
        i = flat.find(head, i + 1)
    if best == 0:
        return f"no capture starts like {norm[:50]!r}"
    return f"matches {best} chars, then book has {norm[best:best + 24]!r} but capture has {flat[at + best:at + best + 24]!r}"


def token_ok(kind, tok, flat, lines):
    if tok in flat:
        return True
    if kind == "size":
        a, b = re.split(r" ?[x×] ?", tok)
        for form in (f"{a}x{b}", f"{a}×{b}", f"{a} × {b}", f"{a} x {b}"):
            if form in flat:
                return True
        return any(re.search(rf"\b{a}\b", l) and re.search(rf"\b{b}\b", l) for l in lines)
    if kind == "ratio":
        r = round(1 - float(tok), 3)
        return any(f in flat for f in (f"{r:.3f}", f"{r:.2f}", f"{r:g}"))
    if kind == "time" and tok.count(":") == 2 and tok.replace(":", "") in flat:
        return True
    if kind == "time" and tok.count(":") == 1:
        return re.search(rf"(?<!\d){re.escape(tok)}(?::\d\d)?", flat) is not None
    return False


def report(found, unplaced, chapters, quiet=False):
    order = {"secret": 0, "token": 1, "code": 2, "struct": 3, "image": 4, "prose": 5}
    found.sort(key=lambda f: (f[0], f[1], order.get(f[2], 9)))
    out = ["# REVIEW-auto — mechanical checks (verify.py), no LLM", ""]
    by_rule = {}
    for f in found:
        by_rule[f[2]] = by_rule.get(f[2], 0) + 1
    out.append(f"{len(found)} findings in {len(chapters)} chapters · " + " · ".join(f"{k} {v}" for k, v in sorted(by_rule.items(), key=lambda kv: order.get(kv[0], 9))))
    out.append("")
    for ch, ln, rule, msg in found:
        out.append(f"{ch}:{ln} [{rule}] {msg[:170]}")
    if unplaced:
        out.append("")
        out.append("images/ not placed in any chapter (ok for help screens and cards kept as evidence): " + ", ".join(unplaced[:30]))
    text = "\n".join(out) + "\n"
    open(OUT, "w", encoding="utf-8").write(text)
    if not quiet:
        print(text)


def only_for(name):
    # The script's own checks run FRESH (a fixer re-running --for sees what is left); the review
    # notes are a fixed list — a note already applied stays listed, so check the file, not the list.
    name = os.path.basename(name)
    check(quiet=True)
    mine = lambda l: l.startswith(name + ":") or l.startswith("- " + name) or l.startswith(f"| {name}")
    auto = [l for l in read(OUT).split("\n") if mine(l)]
    print(f"# verify.py, run now: {len(auto)} finding(s) for {name}")
    for l in auto:
        print(l)
    notes = [l for f in sorted(glob.glob("REVIEW-claims*.md")) for l in read(f).split("\n") if mine(l)]
    if notes:
        print(f"# review notes ({len(notes)}; a note you already applied stays listed)")
        for l in notes:
            print(l)


def bundle(name):
    name = os.path.basename(name)
    outline = read("OUTLINE.md")
    stem = re.sub(r"^\d+-", "", name.replace(".md", ""))
    parts = re.split(r"(?m)^(?=##+ )", outline)
    nn = re.match(r"(\d+)-", name)
    n = nn.group(1).lstrip("0") if nn else None
    # The chapter number wins; the stem is only a fallback, matched against the title alone.
    # (A stem like "move" also sits in another chapter's image list: "23-move-ambiguous".)
    by_num = [p for p in parts if (n == "" and p.startswith("## 00"))
              or (n and re.match(rf"###? บทที่ {n}\b", p))]
    by_stem = [p for p in parts
               if stem and stem in p.split("\n", 1)[0].split("(images:")[0].lower()]
    sec = (by_num or by_stem or [None])[0]
    if not sec:
        sys.exit(f"no OUTLINE section found for {name}")
    imgs = re.search(r"\(images:\s*([^)]*)\)", sec)
    stems = [s.strip() for s in (imgs.group(1) if imgs else "").split(",") if s.strip()]
    proof = re.search(r"^proof:\s*(.*)$", sec, flags=re.M)
    print(f"# bundle for {name}: {len(stems)} images listed in OUTLINE\n")
    for s in stems:
        for d in ("captures", "diagrams"):
            p = f"{d}/{s}.txt"
            if os.path.exists(p):
                print(f"===== {p} =====")
                print(read(p).rstrip())
                print()
                break
        else:
            print(f"===== {s}: no .txt (an image with no source) =====\n")
    if proof:
        print(f"(OUTLINE proof line: {proof.group(1).strip()} — open any extra capture it names)")


if __name__ == "__main__":
    main()
