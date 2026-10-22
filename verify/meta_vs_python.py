"""Compare relic-scan's meta (OUT/meta/files.parquet, sessions.parquet) with relic4's own meta.duckdb,
built by `relic4 truth import --from-scan OUT` + `relic4 meta --from-scan OUT --agent A` on the same OUT.

  uv run --project <agent-relic-v4> python verify/meta_vs_python.py OUT META_DUCKDB

Every column of every row is compared: files on path, sessions on (base, tier). Prints per-column mismatch
counts with a few examples; exit 1 if anything differs.
"""
import sys

import duckdb

out, meta = sys.argv[1], sys.argv[2]
con = duckdb.connect()
con.execute(f"ATTACH '{meta}' AS py (READ_ONLY)")
bad = 0
for table, key in (("files", ["path"]), ("sessions", ["base", "tier"])):
    con.execute(f"CREATE TEMP VIEW rs AS SELECT * FROM read_parquet('{out}/meta/{table}.parquet')")
    rs_cols = [r[0] for r in con.execute("DESCRIBE rs").fetchall()]
    py_cols = [r[0] for r in con.execute(f"DESCRIBE py.{table}").fetchall()]
    print(f"{table}: rust {con.execute('SELECT count(*) FROM rs').fetchone()[0]:,} rows · "
          f"python {con.execute(f'SELECT count(*) FROM py.{table}').fetchone()[0]:,} rows")
    if set(py_cols) - set(rs_cols):
        print(f"  missing in rust: {sorted(set(py_cols) - set(rs_cols))}")
        bad += 1
    if set(rs_cols) - set(py_cols):
        print(f"  rust only (not compared): {sorted(set(rs_cols) - set(py_cols))}")
    on = " AND ".join(f"r.{k} = p.{k}" for k in key)
    k = ", ".join(key)
    for side, a, b in (("only in rust", "rs", f"py.{table}"), ("only in python", f"py.{table}", "rs")):
        n = con.execute(f"SELECT count(*) FROM (SELECT {k} FROM {a} EXCEPT SELECT {k} FROM {b})").fetchone()[0]
        if n:
            bad += 1
            ex = con.execute(f"SELECT {k} FROM {a} EXCEPT SELECT {k} FROM {b} LIMIT 3").fetchall()
            print(f"  {side}: {n:,} rows, e.g. {ex}")
    for c in [c for c in rs_cols if c in py_cols and c not in key]:
        n = con.execute(f"SELECT count(*) FROM rs r JOIN py.{table} p ON {on} WHERE r.{c} IS DISTINCT FROM p.{c}").fetchone()[0]
        if n:
            bad += 1
            ex = con.execute(f"SELECT {', '.join('r.' + x for x in key)}, r.{c}, p.{c} FROM rs r JOIN py.{table} p ON {on} "
                             f"WHERE r.{c} IS DISTINCT FROM p.{c} LIMIT 3").fetchall()
            print(f"  {c}: {n:,} rows differ, e.g. {ex}")
    con.execute("DROP VIEW rs")
print("EQUAL: every row and column matches" if not bad else f"DIFFERENT: {bad} checks failed")
sys.exit(1 if bad else 0)
