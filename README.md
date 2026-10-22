# relic4-scan (lab)

One Rust binary. It finds every AI-agent transcript on a machine and writes **one Parquet file per Bangkok month** in the relic4 truth schema. The design is in RESEARCH.md, the lessons from relic v1–v3 in LEARNED.md, and the exact output rules, traced and verified, in OUTPUT-CHAIN.md.

```
cargo build --release
./target/release/relic4-scan run --out months/                       # incremental: scan + parse only new bytes
./target/release/relic4-scan scan [--full] --index index.tsv        # phase 1 alone: find homes, index files
./target/release/relic4-scan parse --index index.tsv --out months/   # phase 2 alone, cold: months/YYYY-MM.parquet
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

## Not yet
- **A quarantine month** for impossible dates.
- **Antigravity `transcript_full.jsonl` preference.**
- **Splitting multi-GB files** into chunks.
- **Arrow IPC vs Parquet** final-output benchmark.
