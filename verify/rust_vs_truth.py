"""Compare relic4-scan's month Parquet files with relic4 truth (DuckDB), row by row.

  uv run --project <relic4> python verify/rust_vs_truth.py OUT_DIR [--sample-index FILE N]

--sample-index writes the same 125-file sample as verify/understand.py (seed 7) as a scan index, then exits.
Otherwise: every (s, h, idx) in OUT_DIR/*.parquet must exist in truth with equal fields; sources must be a
subset of truth's (truth holds every machine's copies, the scan only this account's).
"""
import collections, glob, os, random, sqlite3, sys

import duckdb

if "--sample-index" in sys.argv:
    out, n = sys.argv[sys.argv.index("--sample-index") + 1], int(sys.argv[sys.argv.index("--sample-index") + 2])
    st = sqlite3.connect(f"file:{os.path.expanduser('~/jsonl-truth/.live/state.db')}?mode=ro", uri=True)
    rows = st.execute("SELECT path, home, off, lines, size FROM files WHERE agent = 'beta@m5' AND off > 0 AND off = size").fetchall()
    by, rnd = collections.defaultdict(list), random.Random(7)
    for r in rows:
        p = r[0]
        eng = "codex" if ("rollout-" in p or "codex" in p.lower()) else ("omp" if ".omp" in p else "claude")
        by[(eng, "sub" if "/subagents/" in p else "main")].append(r)
    with open(out, "w") as f:
        f.write("path\thome\tkind\torigin\ttier\tsize\tmtime_ms\tdev\tino\n")
        for k, v in sorted(by.items()):
            for p, home, off, lines, size in rnd.sample(v, min(n, len(v))):
                f.write(f"{p}\t{home}\t-\t-\t-\t{size}\t0\t0\t0\n")
    print(f"wrote {out}")
    sys.exit(0)

out = sys.argv[1]
con = duckdb.connect()
con.execute(f"CREATE TABLE r AS SELECT * FROM read_parquet('{out}/*.parquet', filename = true)")
truth = sorted(glob.glob(os.path.expanduser("~/relic4-scratch/truth/*.duckdb")))
for i, f in enumerate(truth):
    con.execute(f"ATTACH '{f}' AS t{i} (READ_ONLY)")
con.execute("CREATE TEMP VIEW truth AS " + " UNION ALL ".join(f"SELECT * FROM t{i}.truth" for i in range(len(truth))))
con.execute("CREATE TEMP TABLE j AS SELECT r.*, t.s AS t_s, t.text AS t_text, t.body AS t_body, t.red AS t_red, t.kind AS t_kind, "
            "t.call_id AS t_call_id, t.name AS t_name, t.err AS t_err, t.type AS t_type, t.role AS t_role, t.engine AS t_engine, "
            "t.sid AS t_sid, t.ts AS t_ts, t.ts_th AS t_ts_th, t.sources AS t_sources "
            "FROM r LEFT JOIN truth t ON t.s = r.s AND t.h = r.h AND t.idx = r.idx")
n, found = con.execute("SELECT count(*), count(t_s) FROM j").fetchone()
print(f"rust rows {n:,} · found in truth by (s,h,idx) {found:,}")
for col in ("text", "body", "red", "kind", "call_id", "name", "err", "type", "role", "engine", "sid", "ts", "ts_th"):
    eq = con.execute(f"SELECT count(*) FILTER (WHERE {col} IS NOT DISTINCT FROM t_{col}) FROM j WHERE t_s IS NOT NULL").fetchone()[0]
    mark = "OK" if eq == found else "!!"
    print(f"  {mark} {col:<8} equal {eq:,}/{found:,}")
sub = con.execute("SELECT count(*) FILTER (WHERE list_has_all(t_sources, sources)) FROM j WHERE t_s IS NOT NULL").fetchone()[0]
print(f"  {'OK' if sub == found else '!!'} sources ⊆ truth sources {sub:,}/{found:,}")
month_ok = con.execute("SELECT count(*) FILTER (WHERE regexp_extract(filename, '(\\d{4}-\\d{2}|undated)\\.parquet$', 1) = CASE WHEN ts = '' THEN 'undated' ELSE substr(ts_th, 1, 7) END) FROM j").fetchone()[0]
print(f"  {'OK' if month_ok == n else '!!'} month file = Bangkok month of ts {month_ok:,}/{n:,}")
for col in ("text", "body", "red", "sid", "ts", "engine"):
    for row in con.execute(f"SELECT s, h, idx, left({col}::VARCHAR, 80), left(t_{col}::VARCHAR, 80) FROM j WHERE t_s IS NOT NULL AND {col} IS DISTINCT FROM t_{col} LIMIT 3").fetchall():
        print(f"    diff {col}: {row}")
for row in con.execute("SELECT s, h, idx, left(text, 60), left(body, 60), sources[1] FROM j WHERE t_s IS NULL LIMIT 5").fetchall():
    print(f"    not in truth: {row}")
