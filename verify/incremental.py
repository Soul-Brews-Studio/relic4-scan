"""Incremental == cold. Scratch home from real transcripts -> run (cut short) -> mutate -> run -> run again,
then compare with ONE cold parse of the final home.

  uv run --project <agent-relic-v4> python verify/incremental.py BIN N

Pass when:
  1. every row of the cold parse is in the incremental months, every field equal, its sources included
  2. every row ONLY in the incremental months comes from the old version of the deleted / rewritten file
     (truth is append-only: a row once seen stays)
  3. the third run (nothing changed) reads 0 bytes and rewrites no month
"""
import glob, os, random, shutil, subprocess, sys, time

import duckdb

BIN, N = sys.argv[1], int(sys.argv[2])
W = "/tmp/relic4-inc"
shutil.rmtree(W, ignore_errors=True)
proj = f"{W}/home/.claude/projects"
src = [p for p in glob.glob(os.path.expanduser("~/.claude/projects/*/*.jsonl")) if 20_000 < os.path.getsize(p) < 3_000_000]
rnd = random.Random(11)
full = {}
for p in rnd.sample(src, N + 3):
    d = f"{proj}/{os.path.basename(os.path.dirname(p))}"
    os.makedirs(d, exist_ok=True)
    full[f"{d}/{os.path.basename(p)}"] = open(p, "rb").read()
paths = list(full)
grow, newf, gone, rewrite = paths[:N - 1], paths[N:N + 2], paths[N + 2], paths[N - 1]


def write(p, data):
    with open(p, "wb") as f:
        f.write(data)


half_lines = 0
for p in grow + [gone, rewrite]:
    lines = full[p].splitlines(keepends=True)
    k = rnd.randint(1, len(lines) - 1)
    cut = b"".join(lines[:k])
    if rnd.random() < 0.3:
        cut += lines[k][: len(lines[k]) // 2]  # half a line, no "\n": not JSON, so it must wait for the next run
        half_lines += 1
    write(p, cut)


def run(label, *args):
    t = time.time()
    r = subprocess.run([BIN, *args], capture_output=True, text=True)
    lines = [l for l in r.stderr.splitlines() if l.startswith(("run:", "merge:", "  20", "  und"))]
    print(f"{label}: rc {r.returncode} · {time.time() - t:.2f}s")
    for l in lines:
        print("   ", l)
    assert r.returncode == 0, r.stderr[-2000:]
    return r.stderr


print(f"scratch home: {len(paths)} real transcripts · {N - 1} cut short ({half_lines} mid-line) · 2 added later · 1 deleted · 1 rewritten")
run("round 1  cold, files cut short", "run", "--out", f"{W}/inc", "--root", f"{W}/home", "--agent", "test@lab")
for p in grow + newf:
    write(p, full[p])
os.remove(gone)
write(rewrite, b"".join(full[rewrite].splitlines(keepends=True)[1:]))  # first line removed: prefix changes
run("round 2  grown + new + gone + rewritten", "run", "--out", f"{W}/inc", "--root", f"{W}/home", "--agent", "test@lab")
before = {f: os.path.getmtime(f) for f in glob.glob(f"{W}/inc/*.parquet")}
r3 = run("round 3  nothing changed", "run", "--out", f"{W}/inc", "--root", f"{W}/home", "--agent", "test@lab")
untouched = all(os.path.getmtime(f) == t for f, t in before.items())
zero = "0.00 GB to read" in r3

# one cold parse of the final home
with open(f"{W}/index.tsv", "w") as f:
    f.write("path\tsize\tverdict\n")
    for p in sorted(glob.glob(f"{proj}/*/*.jsonl")):
        f.write(f"{p}\t{os.path.getsize(p)}\ttranscript\n")
run("cold     the final home, one parse", "parse", "--index", f"{W}/index.tsv", "--out", f"{W}/cold", "--agent", "test@lab")

c = duckdb.connect()
c.execute(f"CREATE TABLE i AS SELECT * EXCLUDE (filename), regexp_extract(filename, '([^/]+)\\.parquet$', 1) AS m FROM read_parquet('{W}/inc/*.parquet', filename = true)")
c.execute(f"CREATE TABLE k AS SELECT * EXCLUDE (filename), regexp_extract(filename, '([^/]+)\\.parquet$', 1) AS m FROM read_parquet('{W}/cold/*.parquet', filename = true)")
cols = ["engine", "role", "type", "sid", "ts", "ts_th", "text", "kind", "call_id", "name", "body", "err", "red"]
nk, ni = c.execute("SELECT (SELECT count(*) FROM k), (SELECT count(*) FROM i)").fetchone()
found, same, srcs = c.execute(f"""
  SELECT count(i.h), count(*) FILTER (WHERE {' AND '.join(f'k.{x} IS NOT DISTINCT FROM i.{x}' for x in cols)}),
         count(*) FILTER (WHERE list_has_all(i.sources, k.sources))
  FROM k LEFT JOIN i ON i.m = k.m AND i.s = k.s AND i.h = k.h AND i.idx = k.idx""").fetchone()
extra = c.execute("SELECT count(*) FROM i ANTI JOIN k ON i.m = k.m AND i.s = k.s AND i.h = k.h AND i.idx = k.idx").fetchone()[0]
bad_extra = c.execute(f"""SELECT count(*) FROM (SELECT * FROM i ANTI JOIN k ON i.m = k.m AND i.s = k.s AND i.h = k.h AND i.idx = k.idx)
  WHERE NOT list_bool_and(list_transform(sources, x -> contains(x, '{os.path.basename(gone)}') OR contains(x, '{os.path.basename(rewrite)}')))""").fetchone()[0]
ok = found == nk and same == nk and srcs == nk and bad_extra == 0 and untouched and zero
print(f"""
cold rows {nk:,} · incremental rows {ni:,}
  1. cold rows found in incremental      {found:,}/{nk:,}
     every field equal                   {same:,}/{nk:,}
     sources included                    {srcs:,}/{nk:,}
  2. rows only in incremental            {extra:,} (all from the old deleted/rewritten file: {extra - bad_extra:,})
  3. round 3 read 0 bytes: {zero} · rewrote no month: {untouched}
{'PASS' if ok else 'FAIL'}""")
sys.exit(0 if ok else 1)
