#!/usr/bin/env python3
"""Check that every message (by uuid) in a pulled copy of another machine's .claude/projects is already in ~/.claude/projects.
Read only. Run it before deleting the other machine's copy.
usage: claude-verify.py <dir holding the pulled copy, one folder per source home>
Files that differ in size are compared by message uuid; lines without a uuid (queue-operation, summary) are ignored."""
import os, sys, json
T = os.path.expanduser('~/.claude/projects'); R = sys.argv[1]
def keys(p):
    s = set()
    with open(p, 'rb') as f:
        for l in f:
            try: u = json.loads(l).get('uuid')
            except Exception: continue
            if u: s.add(u)
    return s
total = missing = nofile = 0; bad = []
for h in sorted(os.listdir(R)):
    for dp, dn, fn in os.walk(f'{R}/{h}'):
        for f in fn:
            s = os.path.join(dp, f)
            if os.path.islink(s): continue
            r = os.path.relpath(s, f'{R}/{h}'); t = f'{T}/{r}'; total += 1
            if not os.path.exists(t): nofile += 1; bad.append((h, r, 'NOT IN ~/.claude/projects')); continue
            if os.path.getsize(s) == os.path.getsize(t) or not f.endswith('.jsonl'): continue
            m = keys(s) - keys(t)
            if m: missing += 1; bad.append((h, r, f'{len(m)} uuids missing'))
print('files', total, 'not-there', nofile, 'files-with-missing-uuids', missing)
for b in bad: print(b)
sys.exit(1 if bad else 0)
