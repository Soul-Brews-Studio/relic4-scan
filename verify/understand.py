"""Re-derive truth rows from real files with the server's own parse_chunk, compare field by field with truth.

  uv run --project <agent-relic-v4> python verify/understand.py [N files per kind]

Reads only: the sample files, ~/jsonl-truth/.live/state.db (offsets), ~/relic4-scratch/truth/*.duckdb.
Copy-independent fields must match exactly; engine / sid / ts may differ when another copy of the same bytes
reached the server first (events INSERT OR IGNORE on (h, s): first arrival wins).
"""
import glob, os, random, sqlite3, sys, collections

import duckdb

from relic4 import engine as E
from relic4.store import parse_chunk
from relic4.sql import MSGKEY

N = int(sys.argv[1]) if len(sys.argv) > 1 else 40
AGENT = "beta@m5"
st = sqlite3.connect(f"file:{os.path.expanduser('~/jsonl-truth/.live/state.db')}?mode=ro", uri=True)
rows = st.execute("SELECT path, home, off, lines, size FROM files WHERE agent = ? AND off > 0 AND off = size", (AGENT,)).fetchall()
rnd = random.Random(7)
by = collections.defaultdict(list)
for p, home, off, lines, size in rows:
    k = E.__dict__.get("engine_of", None)
    eng = "codex" if ("rollout-" in p or "codex" in p.lower()) else ("omp" if ".omp" in p else "claude")
    tier = "sub" if "/subagents/" in p else "main"
    by[(eng, tier)].append((p, home, off, lines, size))
sample = []
for k, v in sorted(by.items()):
    sample += rnd.sample(v, min(N, len(v)))
print(f"{len(sample)} files sampled from {len(rows):,} fully-sent {AGENT} files: " + ", ".join(f"{k[0]}/{k[1]} {min(N, len(v))}" for k, v in sorted(by.items())))

# server path: the whole file as jt sends it (complete lines), parsed in one chunk from line 0
derived = {"events": [], "tools": [], "sources": []}
for p, home, off, lines, size in sample:
    data = open(p, "rb").read()[:off]
    ev, tl, sr, ctx = parse_chunk(data, p, home, AGENT, (0, "", None))
    assert ctx[0] == lines, (p, ctx[0], lines)          # line count agrees with what the server recorded
    derived["events"] += ev
    derived["tools"] += tl
    derived["sources"] += sr

con = duckdb.connect()
truth = {os.path.basename(f)[:7]: f for f in glob.glob(os.path.expanduser("~/relic4-scratch/truth/*.duckdb"))}
for i, (m, f) in enumerate(truth.items()):
    con.execute(f"ATTACH '{f}' AS t{i} (READ_ONLY)")
con.execute("CREATE TEMP VIEW truth AS " + " UNION ALL ".join(f"SELECT * FROM t{i}.truth" for i in range(len(truth))))

con.execute("CREATE TEMP TABLE dev AS SELECT * FROM (VALUES (NULL::VARCHAR, 0::BIGINT, ''::VARCHAR, ''::VARCHAR, ''::VARCHAR, ''::VARCHAR, ''::VARCHAR, ''::VARCHAR, ''::VARCHAR, ''::VARCHAR, ''::VARCHAR)) t(mon, h, s, engine, type, role, sid, ts, ts_th, text, red) WHERE false")
con.executemany("INSERT INTO dev VALUES (?,?,?,?,?,?,?,?,?,?,?)", [e[:11] for e in derived["events"]])
con.execute("CREATE TEMP TABLE dtl AS SELECT * FROM (VALUES (NULL::VARCHAR, 0::BIGINT, 0::BIGINT, ''::VARCHAR, ''::VARCHAR, ''::VARCHAR, ''::VARCHAR, ''::VARCHAR, ''::VARCHAR, ''::VARCHAR, ''::VARCHAR, 0::INT, 0::BIGINT, ''::VARCHAR)) t(mon, h, idx, engine, sid, ts, ts_th, kind, call_id, name, body, err, bytes, red) WHERE false")
con.executemany("INSERT INTO dtl VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?,?)", [t[:14] for t in derived["tools"]])
con.execute("CREATE TEMP TABLE dsr AS SELECT * FROM (VALUES (NULL::VARCHAR, 0::BIGINT, ''::VARCHAR, ''::VARCHAR, 0::BIGINT)) t(mon, h, agent, file, line) WHERE false")
con.executemany("INSERT INTO dsr VALUES (?,?,?,?,?)", derived["sources"])

def report(label, sql):
    r = con.execute(sql).fetchone()
    print(f"  {label:<46} {r}")

print("events (text + thinking):")
report("derived rows / found in truth by (s,h)", "SELECT count(*), count(t.h) FROM dev d LEFT JOIN truth t ON t.s = d.s AND t.h = d.h AND t.idx = 0")
report("text equal / red equal / type equal / role equal",
       "SELECT count(*) FILTER (WHERE t.text IS NOT DISTINCT FROM d.text), count(*) FILTER (WHERE t.red IS NOT DISTINCT FROM d.red), count(*) FILTER (WHERE t.type IS NOT DISTINCT FROM d.type), count(*) FILTER (WHERE t.role IS NOT DISTINCT FROM d.role) FROM dev d JOIN truth t ON t.s = d.s AND t.h = d.h AND t.idx = 0")
report("copy-dependent: engine / sid / ts equal",
       "SELECT count(*) FILTER (WHERE t.engine = d.engine), count(*) FILTER (WHERE t.sid IS NOT DISTINCT FROM d.sid), count(*) FILTER (WHERE t.ts IS NOT DISTINCT FROM d.ts) FROM dev d JOIN truth t ON t.s = d.s AND t.h = d.h AND t.idx = 0")
report("month bucket equal (file of the row)",
       "SELECT count(*) FILTER (WHERE substr(d.mon, 1, 7) = substr(coalesce(t.ts_th, 'undated'), 1, 7) OR (d.mon = 'undated' AND t.ts = '')), count(*) FROM dev d JOIN truth t ON t.s = d.s AND t.h = d.h AND t.idx = 0")
report("msgkey equal (recomputed from derived)",
       f"SELECT count(*) FILTER (WHERE t.msgkey = (SELECT {MSGKEY} FROM (SELECT d.s AS s, d.role AS role, d.ts AS ts, d.text AS text, NULL::BIGINT AS idx, NULL AS kind, NULL AS call_id, NULL AS name, NULL AS body, NULL::BOOLEAN AS err))), count(*) FROM dev d JOIN truth t ON t.s = d.s AND t.h = d.h AND t.idx = 0")
print("tools:")
report("derived rows / found in truth by (h,idx)", "SELECT count(*), count(t.h) FROM dtl d LEFT JOIN truth t ON t.s = 'tools' AND t.h = d.h AND t.idx = d.idx")
report("body / kind / call_id / name / err / red equal",
       "SELECT count(*) FILTER (WHERE t.body IS NOT DISTINCT FROM d.body), count(*) FILTER (WHERE t.kind = d.kind), count(*) FILTER (WHERE t.call_id IS NOT DISTINCT FROM d.call_id), count(*) FILTER (WHERE t.name IS NOT DISTINCT FROM d.name), count(*) FILTER (WHERE t.err = (d.err = 1)), count(*) FILTER (WHERE t.red IS NOT DISTINCT FROM d.red) FROM dtl d JOIN truth t ON t.s = 'tools' AND t.h = d.h AND t.idx = d.idx")
print("sources:")
report("derived locators present in truth.sources",
       "SELECT count(*), count(*) FILTER (WHERE EXISTS (SELECT 1 FROM truth t WHERE t.h = d.h AND list_contains(t.sources, d.agent || ':' || d.file || ':' || d.line))) FROM dsr d")
print("source locators missing from truth (all):")
for r in con.execute("""SELECT d.mon, d.h, d.file, d.line, (SELECT max(t.ts_th) FROM truth t WHERE t.h = d.h) AS truth_has_h
                        FROM dsr d WHERE NOT EXISTS (SELECT 1 FROM truth t WHERE t.h = d.h AND list_contains(t.sources, d.agent || ':' || d.file || ':' || d.line))""").fetchall():
    print("   ", r[0], r[1], r[2].replace(os.path.expanduser("~"), "~")[-90:], "line", r[3], "| h in truth:", r[4])
print("why copy-dependent fields differ (first 5 sid/ts mismatches):")
for r in con.execute("""SELECT d.s, left(d.sid, 8), left(t.sid, 8), d.ts, t.ts, t.machines FROM dev d JOIN truth t ON t.s = d.s AND t.h = d.h AND t.idx = 0
                         WHERE t.sid IS DISTINCT FROM d.sid OR t.ts IS DISTINCT FROM d.ts LIMIT 5""").fetchall():
    print("   ", r)
