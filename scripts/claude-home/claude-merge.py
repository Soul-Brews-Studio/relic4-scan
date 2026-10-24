#!/usr/bin/env python3
"""Merge many .claude*/projects folders into ~/.claude/projects. Copies only; never deletes. usage: claude-merge.py [--dry]"""
import os, sys, shutil, time, json
H = os.path.expanduser('~'); T = f'{H}/.claude/projects'; DRY = '--dry' in sys.argv
STAGE = '/tmp/claude-merge'          # replaced target files and diverged copies go here
SRC = {}
def add(name, p):
    if os.path.isdir(p): SRC[name] = p
for h in ['.claude', '.claude1', '.claude-lean', '.claudez', '.claude-home', '.claudez-0', '.claude-zz', '.claude-001', '.claude-003']:
    add('mirror-mba' + h, f'{H}/.relic-mirrors/mba/{h}/projects')
if '--only-inhome' not in sys.argv: add('relic3-mirror-nat', f'{H}/.relic3-mirror/Users/nat/.claude/projects')
add('claude-neo', f'{H}/.claude-neo/projects')
add('archive-neo-0610', '/opt/Code/_archive/oracle-world/neo-2026-06-10/.claude/projects')
add('archive-nat-0610', '/opt/Code/_archive/oracle-world/nat-2026-06-10/.claude/projects')
add('mirror-white-ampere', f'{H}/.relic-mirrors/white-ampere-claude')
import glob
for d in sorted(glob.glob(f'{H}/.claude/projects-*')):
    add('inhome-' + os.path.basename(d), d)
for h in ['white', 'black']:
    for d in ['.claude', '.claudez']:
        add(f'{h}-live{d}', f'{STAGE}/from-{h}/{d}')
for d in sorted(os.listdir(f'{STAGE}/from-mba')) if os.path.isdir(f'{STAGE}/from-mba') else []:
    add('mba-live' + d, f'{STAGE}/from-mba/{d}')
def prefix(small, big):
    with open(small, 'rb') as a, open(big, 'rb') as b:
        while True:
            x = a.read(1 << 20)
            if not x: return True
            if x != b.read(len(x)): return False
st = {}; now = time.time(); log = []
for name, root in SRC.items():
    c = dict(files=0, copied=0, same=0, replaced=0, kept=0, diverged=0, live=0, bytes=0)
    for dp, dn, fn in os.walk(root):
        for f in fn:
            s = os.path.join(dp, f); r = os.path.relpath(s, root); t = os.path.join(T, r)
            try: ss = os.stat(s)
            except OSError: continue
            c['files'] += 1
            if not os.path.exists(t):
                c['copied'] += 1; c['bytes'] += ss.st_size
                if not DRY:
                    os.makedirs(os.path.dirname(t), exist_ok=True); shutil.copy2(s, t)
                continue
            ts = os.stat(t)
            if ts.st_size == ss.st_size: c['same'] += 1; continue
            if now - ts.st_mtime < 600: c['live'] += 1; continue      # being written now
            if ts.st_size < ss.st_size and prefix(t, s):             # transcripts only grow: the longer one holds the shorter
                c['replaced'] += 1; c['bytes'] += ss.st_size
                if not DRY:
                    q = f'{STAGE}/replaced/{r}'; os.makedirs(os.path.dirname(q), exist_ok=True); shutil.move(t, q); shutil.copy2(s, t)
            elif ss.st_size < ts.st_size and prefix(s, t): c['kept'] += 1
            else:
                c['diverged'] += 1
                if not DRY:
                    q = f'{STAGE}/diverged/{name}/{r}'; os.makedirs(os.path.dirname(q), exist_ok=True); shutil.copy2(s, q)
    st[name] = c; print(name, json.dumps(c), flush=True)
tot = {k: sum(c[k] for c in st.values()) for k in next(iter(st.values()))} if st else {}
print('TOTAL', json.dumps(tot), 'dry' if DRY else 'done')
