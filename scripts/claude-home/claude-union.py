#!/usr/bin/env python3
"""One truth for split sessions: keep every line of ~/.claude/projects/<r>, append the messages (by uuid) that only a diverged copy has.
usage: claude-union.py plan|backup-originals|apply"""
import os, sys, json, shutil, time
H = os.path.expanduser('~'); T = f'{H}/.claude/projects'; D = '/tmp/claude-merge/diverged'; O = '/tmp/claude-merge/union-orig'
mode = sys.argv[1]
copies = {}                                   # rel path -> [diverged copy paths]
for src in sorted(os.listdir(D)):
    for dp, dn, fn in os.walk(f'{D}/{src}'):
        for f in fn:
            if f.endswith('.jsonl'):
                p = os.path.join(dp, f); copies.setdefault(os.path.relpath(p, f'{D}/{src}'), []).append(p)
def read(p):
    out = []
    with open(p, errors='replace') as f:
        for line in f:
            line = line.rstrip('\n')
            if not line: continue
            try: j = json.loads(line)
            except Exception: out.append((None, None, line)); continue
            out.append((j.get('uuid'), j.get('timestamp') or '', line))
    return out
todo = {}
for r, ps in copies.items():
    t = f'{T}/{r}'
    if not os.path.exists(t): continue
    base = read(t); have = {u for u, _, _ in base if u}
    extra = {}
    for p in ps:
        for u, ts, line in read(p):
            if u and u not in have and u not in extra: extra[u] = (ts, line)
    if extra: todo[r] = (t, base, extra)
print('sessions with messages only in a diverged copy:', len(todo), '| messages:', sum(len(v[2]) for v in todo.values()))
if mode == 'plan': sys.exit()
if mode == 'backup-originals':
    for r, (t, _, _) in todo.items():
        q = f'{O}/{r}'; os.makedirs(os.path.dirname(q), exist_ok=True); shutil.copy2(t, q)
    print('originals saved:', len(todo)); sys.exit()
ok = bad = live = 0
for r, (t, base, extra) in todo.items():
    if time.time() - os.path.getmtime(t) < 600: live += 1; continue
    lines = [l for _, _, l in base] + [l for _, l in sorted(extra.values(), key=lambda x: x[0])]
    tmp = t + '.union.tmp'
    with open(tmp, 'w') as f: f.write('\n'.join(lines) + '\n')
    new = read(tmp); nu = {u for u, _, _ in new if u}
    want = {u for u, _, _ in base if u} | set(extra)
    if want <= nu and all(u is not None or l for u, _, l in new) and len(new) == len(base) + len(extra):
        shutil.copystat(t, tmp); os.replace(tmp, t); ok += 1
    else:
        os.unlink(tmp); bad += 1; print('verify failed, kept original:', r)
print(f'rebuilt {ok}, failed {bad}, skipped live {live}')
