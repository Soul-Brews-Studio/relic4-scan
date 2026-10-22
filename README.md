# relic4-scan (lab)

One Rust binary. It finds every AI-agent transcript on a machine and writes **one Parquet file per Bangkok month** in the relic4 truth schema. The design is in RESEARCH.md, the lessons from relic v1–v3 in LEARNED.md, and the exact output rules, traced and verified, in OUTPUT-CHAIN.md.

```
cargo build --release
./target/release/relic4-scan scan [--full] --index index.tsv        # phase 1: find homes, index files
./target/release/relic4-scan parse --index index.tsv --out months/   # phase 2: months/YYYY-MM.parquet
```

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
- **Offset-based incremental parse.** LEARNED.md: no relic ever built it, and it is where the next big win is.
- **A quarantine month** for impossible dates.
- **Antigravity `transcript_full.jsonl` preference.**
- **Splitting multi-GB files** into chunks.
- **Arrow IPC vs Parquet** final-output benchmark.
