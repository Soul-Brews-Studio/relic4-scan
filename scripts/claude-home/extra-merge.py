import os, json, shutil
T = os.path.expanduser('~/.claude/projects'); B = '/tmp/claude-merge/from-white-extra'
ROOTS = [f'{B}/.claudez/projects', f'{B}/.claude-backup-2026-02-06/.claude/projects', f'{B}/.claude-backup-2026-02-06/.claude-zz/projects']
def keys(p):
    s = set()
    with open(p, 'rb') as f:
        for l in f:
            try: u = json.loads(l).get('uuid')
            except Exception: continue
            if u: s.add(u)
    return s
copied = ok = 0; miss = []
for R in ROOTS:
    for dp, dn, fn in os.walk(R):
        for f in fn:
            s = os.path.join(dp, f)
            if os.path.islink(s): continue
            r = os.path.relpath(s, R); t = f'{T}/{r}'
            if not os.path.exists(t):
                os.makedirs(os.path.dirname(t), exist_ok=True); shutil.copy2(s, t); copied += 1; continue
            if os.path.getsize(s) == os.path.getsize(t) or not f.endswith('.jsonl'): ok += 1; continue
            m = keys(s) - keys(t)
            if m: miss.append((r, len(m)))
            else: ok += 1
print('copied-new', copied, 'ok', ok, 'files-with-missing-uuids', len(miss)); print(miss[:10])
