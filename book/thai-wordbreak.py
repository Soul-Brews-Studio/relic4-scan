#!/usr/bin/env python3
"""Insert ZWSP at Thai word boundaries so typst can break lines correctly.

Thai has no spaces between words, so a typesetter with no Thai dictionary
breaks lines wherever it likes -- mid-word. Inserting U+200B at the real word
boundaries gives it legal break points without changing what the text looks
like. Inline code is left alone: a ZWSP inside `foo_bar` would make the code
uncopyable, which is worse than a bad line break.
"""
import re
import sys

from pythainlp.tokenize import word_tokenize

ZWSP = "​"
THAI = r"฀-๿"


def has_thai(text):
    return bool(re.search(f"[{THAI}]", text))


def insert_zwsp(text):
    if not has_thai(text):
        return text
    out = []
    # keep inline code spans verbatim
    for part in re.split(r"(`[^`]+`)", text):
        if part.startswith("`") or not has_thai(part):
            out.append(part)
            continue
        for seg in re.split(f"([{THAI}]+)", part):
            if has_thai(seg):
                out.append(ZWSP.join(word_tokenize(seg, engine="newmm")))
            else:
                out.append(seg)
    return "".join(out)


def main(paths):
    in_fence = False
    for path in paths:
        with open(path, encoding="utf-8") as f:
            for line in f:
                stripped = line.lstrip()
                if stripped.startswith("```"):
                    in_fence = not in_fence
                    sys.stdout.write(line)
                    continue
                # never touch fenced code -- ZWSP would break copy-paste
                sys.stdout.write(line if in_fence else insert_zwsp(line))
        sys.stdout.write("\n\n")


if __name__ == "__main__":
    main(sys.argv[1:])
