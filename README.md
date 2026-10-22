# relic-scan

One Rust binary, `relic-scan` (this repo is still named relic4-scan). It finds every AI-agent transcript on a machine, writes **one Parquet file per Bangkok month** in the relic4 truth schema, and writes the meta beside them: files, sessions, and every `.jsonl` on the machine. The design is in RESEARCH.md, the lessons from relic v1–v3 in LEARNED.md, and the exact output rules, traced and verified, in OUTPUT-CHAIN.md.

Grown in the agent-relic-v4 lab (`lab/relic4-scan` on branch `lab-rust-scan`, history kept). [agent-relic-v4](https://github.com/Soul-Brews-Studio/agent-relic-v4) is the reader: it turns these month files into truth, meta, vectors and search. The verify scripts need its Python environment: `uv run --project <agent-relic-v4> python verify/…`.

**Install and run, one command each.** `install.sh` finds cargo, even when it is not on PATH, and installs to `~/.local/bin`. The repo is private, so cargo fetches it through the git CLI and your gh login.
```
gh api repos/Soul-Brews-Studio/relic4-scan/contents/install.sh -H 'Accept: application/vnd.github.raw' | sh
relic-scan up --kind claude      # scan + parse + meta + map into ~/.relic-scan, then http://127.0.0.1:8787/map and /tree
```
To install a branch instead of `main`, pipe to `REF=<branch> sh` rather than `sh`.
`up` is `run` followed by `serve`; run it again later and only new bytes are read (about 10 s). `relic-scan serve` serves what is already there.

```
cargo build --release
./target/release/relic-scan run --out months/                       # incremental: scan + parse only new bytes + meta
./target/release/relic-scan run --kind claude --out claude/            # Claude homes only (own --out)
./target/release/relic-scan map --out months/                       # months/meta/map.html: session star map + query (offline page)
./target/release/relic-scan meta --out months/                      # meta alone: months/meta/{files,sessions}.parquet
./target/release/relic-scan scan [--full] --index index.tsv        # phase 1 alone: find homes, index files
./target/release/relic-scan parse --index index.tsv --out months/   # phase 2 alone, cold: months/YYYY-MM.parquet
```

## run (incremental)
`months/_state.tsv` records, per file: dev, inode, size, the offset after the last complete line, the line count, the session, the last ts, and a hash of the first 4 KB. Each run classifies every file:

| state | rule | work |
|---|---|---|
| unchanged | same inode, size = offset | nothing is read |
| grown | same inode, size > offset, same prefix | parse from the offset with the saved line count, session and last ts |
| rewritten | other inode, size < offset, or the prefix changed | parse from 0 again; old rows stay, truth is append-only |
| new | not in the state | parse from 0 |
| gone | in the state, not on disk | dropped from the state; its rows stay |

Only months that received rows are rewritten; every other month file is left untouched. The state file is written last, so after a crash the next run re-reads and the line hash folds the duplicates.

**Proof: incremental equals cold** (`verify/incremental.py`). The test takes real transcripts and:
1. cuts them at random lines, some mid-line;
2. runs;
3. grows them, adds new files, deletes one, rewrites the start of one;
4. runs again, then a third time;
5. compares with one cold parse of the final state.

| transcripts | cold rows | in incremental, every field equal | extra rows (old versions, append-only) | 3rd run |
|---:|---:|---:|---:|---|
| 40 | 1,938 | 1,938 / 1,938 | 1 | 0 bytes read, no month rewritten |
| 300 | 21,257 | 21,257 / 21,257 | 397 | 0 bytes read, no month rewritten |

**m5 (beta), whole machine:**

| run | files | read | time |
|---|---:|---:|---:|
| cold | 86,904 | 75.6 GB | **58 s**: scan 9, parse 27 (2.8 GB/s, 18 threads), merge 8, prefix hashes and state 14 |
| incremental (2 grown) | 86,904 | 0.00 GB | **13 s**: scan 8.5, merge 4.3 (only 2026-10 rewritten) |

For comparison, relic v3 needed 43 s for an incremental run with 37 changed files. Where the next win is:
- the light scan re-walks every home (8.5 s): cache the home list and only stat known files, or watch with FSEvents;
- rewriting the current month (4.3 s): a small delta file plus compaction would trade the "1 month = 1 file" rule for speed.

## scan: light (default) vs --full
| | reads file contents | m5 (beta), 91,297 files / 76 GB | verdicts |
|---|---|---|---|
| light | no: names, folder shapes, path rules | **8.3 s** | transcript-shaped 86,290 · not-transcript 4,393 · unknown 614 |
| full | first complete lines, plus the real parser as proof | **23.0 s** | transcript 85,043 · empty 1,234 · unverified 505 · not-transcript 4,508 |

How a home is found:
- **By name:** `.claude*`, so `.claude_bak`, `.claudebak`, `.claude2` and the rest; also `.codex*`, `.omp*`, `.omx*`, `.thclaws*`, `.gemini*`, `.pi`.
- **By shape:** `<root>/<project>/<uuid>.jsonl`, or `<root>/[YYYY/MM/DD/]rollout-*`.
- A home found only by shape must contain at least one transcript.

How each file is classified:
- **Content signature:** claude, codex, omp/pi or thclaws.
- **Look-alikes that are not transcripts:** history.jsonl, workflow journal.jsonl, omx logs, channel access logs.
- **A transcript-shaped path is never dropped,** even if its content cannot be proven.

Verified against truth: **0 lost**. No file that produced a truth row was called not-transcript, in either mode.

## parse
- **Scheduling:** biggest files first (LPT) over N threads.
- **Per file:** the bytes jt would send. Lines are split as Python `splitlines` does, and each line is hashed (blake2b-8) including its newline.
- **Per line:** the port of `engine.py` and `store.parse_chunk`: text, thinking, tools, redaction, ts, sid and engine.
- **Output:** raw parts per thread and month, then a merge to one file per month.
  - First copy wins, by (file, line).
  - Sources keep one locator per (h, agent, file).

| run | files | GB | time | result vs relic4 truth |
|---|---:|---:|---:|---|
| sample (seed 7) | 125 | — | 1.4 s | 13,602/13,602 rows, every field byte-equal |
| all beta@m5 files jt sent | 50,188 | 43 | **17 s** (parse ~11 s, merge 5 s) | 4,228,336 rows found; text/body/red/kind/call_id/name/err/type/role/engine equal on all; sid differs on 2 and ts on 14 (copy-dependent, OUTPUT-CHAIN §3); 776 rows newer than truth |

`--jt-compat` stops a file at a line over 8 MB, as jt does. By default `parse` skips files the scan called not-transcript; `--all` includes them.

## meta
The last step of `run`, or `relic-scan meta --out DIR` on its own. It reads the month files and `_state.tsv` and writes two tables. It is a Rust port of agent-relic-v4's `relic4 meta --from-scan`, with the same columns and path rules (`sql.py` MACROS), and needs no truth import, Python or DuckDB.

| file | one row per | columns |
|---|---|---|
| `DIR/meta/files.parquet` | file in the state | agent, path, home, size, lines, last_ms, file_sid, base, tier, engine, proj, parent_sid, cat, n_rows, text_rows, first_ts, last_ts |
| `DIR/meta/sessions.parquet` | file name (base, tier), copies folded | base, tier, sid, parent_sid, engine, proj, cat, probe, first_ts, last_ts, last_ms, copies, machines, agents, size, n_rows, text_rows, text_msgs, months |

- **Location:** the tables sit in `meta/`, not next to the months, because `relic4 truth import --from-scan` reads every `DIR/*.parquet` as a month.
- **Incremental:** each month's per-file and per-session counts are cached in `meta/cache/YYYY-MM.tsv`. A month is aggregated again only when its Parquet file changed (size or mtime).
- **text_msgs:** a distinct count of (role, ts, text) over the text rows. relic4 counts `msgkey` there; the two counts are the same.
- **One known difference:** a session also counts rows whose only source file has since been deleted. Truth is append-only, so those rows still belong to it. meta.py drops them as orphans.

**Every `.jsonl` on the machine:** `run` also writes `meta/jsonl.parquet`, with one row per `.jsonl` file the walk saw. Columns: path, place, home, home_kind, origin, vault, agent, verdict, tier, size, mtime_ms and parsed. `place` is one of three values:
- **home:** the file is in an agent home and is parsed into the months.
- **vault:** the file is in an oracle's ψ vault.
- **other:** anywhere else under the roots.

The same walk that finds the homes collects the loose files, so they add no extra pass. Loose files are few, so the scan always reads their first lines, even in the light scan. On m5 that gives 103,657 `.jsonl`: 91,298 in homes, 61 in 22 ψ vaults, and 12,298 elsewhere (mostly water-api and arthur-data). Of the loose files, the 14 whose content looks like a transcript are all test fixtures in repos. No real transcript sits outside a home.

**Claude only** (`run --kind claude`, m5, 2026-10-22):

| run | files | read | sessions | time |
|---|---:|---:|---:|---:|
| cold | 83,673 | 67.6 GB, 19.3 M lines, 8.5 M raw rows | 43,006 | **46.7 s**: scan 8.2, parse 21.7 (3.1 GB/s), merge 6.8, meta 1.4 |
| incremental | 83,673 unchanged | 0 GB | 43,006 | **9.8 s**: scan 9.0, meta 0.5 (no month re-aggregated) |

**Proof: equal to relic4's meta** (`verify/meta_vs_python.py`). It compares every column of every row against `relic4 truth import --from-scan` followed by `relic4 meta --from-scan` on the same DIR. Files are matched on path, sessions on (base, tier).

| m5 (beta), 2026-10-22 | files | sessions | result | time |
|---|---:|---:|---|---:|
| cold (no cache) | 86,905 | 45,094 | every row and column equal | **1.9 s** (Python: truth import 19.9 s + meta 2.5 s) |
| after an incremental `run` (2026-10 rewritten) | 86,905 | 45,094 | every row and column equal | **0.9 s** (1 month aggregated again) |

## map and tree
`relic-scan serve` (or `up`) serves the page on 127.0.0.1 only: `/map` is the star map and `/tree` the tree. As a file, `map.html?view=tree` opens the tree. Both views share the query box, the side panel and the hash, so a query carries over when you switch.

**Tree:** jsonl home → repo → session → satellites. Every level shows the sessions, the rows, a bar of rows by type (user, assistant, thinking, tool calls, tool results) and the last activity. A session is listed under every home that holds a copy of it. A query result of up to 400 sessions opens itself down to the repos.

`run` ends with it; `relic-scan map --out DIR` rebuilds it alone in 0.1 s. `--out` defaults to `~/.relic-scan` for run, meta, map, serve and up. It writes `DIR/meta/map.html`, a single offline page (3.6 MB for 43,006 sessions) with the data inlined. It makes no network calls; open the file, or serve it on 127.0.0.1.

- **Ring:** the Bangkok month the session started. The oldest month is innermost; a brighter ring marks the start of a year.
- **Sector:** the repo. Wider sectors have more sessions; repos with fewer than 12 sessions share one sector.
- **Star:** a main session, sized by log(rows). Subagents and workflow agents orbit their parent, which is found by the `<uuid>/subagents/` folder they sit in. 99% of them have a parent.
- **Query:** plain words match repo, project or sid. Filters:
  - `repo:` `proj:` `sid:` `home:` `tier:main|sub|wf`
  - `month:2026-07` `after:2026-09-01` `before:`
  - `rows>1000` `text` `msgs` `copies`
  - `-` before a term negates it, `"..."` quotes a value, and all terms must match (AND).
  - The query is kept in the URL hash, so a link reopens the same view.
- **Panel:** the counts, top repos and jsonl homes of the current match, each clickable to narrow the query. Clicking a star shows that session: sid, repo, times, rows, copies, homes, satellites, and buttons to copy the sid or a `claude --resume` line.

## Not yet
- **A quarantine month** for impossible dates.
- **Antigravity `transcript_full.jsonl` preference.**
- **Splitting multi-GB files** into chunks.
- **Arrow IPC vs Parquet** final-output benchmark.
