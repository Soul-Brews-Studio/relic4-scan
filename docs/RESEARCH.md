# relic4-scan — research (2026-10-22)

The question: one fast, multi-threaded Rust binary that scans every AI-agent JSONL on a machine and writes one file per Bangkok month. It indexes first, then spreads the work across threads.

The research ran as 5 parallel Sonnet agents, each searching the web and citing sources. Anything not measured in a source is marked *unverified*. Numbers for our own workload come from the lab bench (BENCH.md), not from here.

## Phase 1 — index (find every file)
- **`ignore::WalkParallel`** (ripgrep's walker) gives each worker a crossbeam-deque and lets workers steal directories from each other. `filter_entry` prunes `node_modules`, `.git` and `target`, and you can choose whether to follow symlinks. https://github.com/BurntSushi/ripgrep/blob/master/crates/ignore/src/walk.rs
- **`jwalk`** parallelises per directory and is now marked unmaintained. https://docs.rs/crate/jwalk/latest
- **macOS:** `getattrlistbulk` with a 128 KB buffer walked 409,500 files in 0.52 s on an M1 Pro with a warm cache. `du` took 3.33 s and a naive readdir+lstat 6.8 s. About 91% of the time was in syscalls. https://healeycodes.com/maybe-the-fastest-disk-usage-program-on-macos
- **Inode dedupe:** use `(dev, ino)` with 128 sharded `Mutex<HashSet>`s. In dumac the locking cost about 1.5%. Same source.

## Phase 2 — spread the parse
- **Largest first (LPT).** Sort the files by size, biggest first, into a `Vec`, and give each thread the next index from an `AtomicUsize`. *Standard scheduling; no benchmark found.*
- **Split multi-GB files** into newline-aligned 32–64 MB chunks so one file is not the critical path. *Unverified, but follows directly from the size skew: a few GB-sized files among 395k small ones.*

## Reading, parsing, hashing
- **JSON parser:** use `sonic-rs`, which has fast paths for aarch64 and x86_64.
  - In its README (Xeon), a full typed parse of twitter.json: sonic-rs 828 µs, simd-json 1.09 ms, serde_json 2.29 ms.
  - Lazy access (`get_unchecked` / `LazyValue`): 77 µs. https://github.com/cloudwego/sonic-rs
- **`simd-json`** uses NEON on aarch64, but it parses in place (`&mut Vec<u8>`) and goes through a tape, so it is slower for typed structs. https://github.com/simd-lite/simd-json
- **Line hash:** `blake2b_simd::Params::new().hash_length(8)`. The digest length is part of the BLAKE2 parameter block, so this is **not** a truncated BLAKE2b-512. https://docs.rs/blake2b_simd/latest/blake2b_simd/struct.Params.html
  - Our engine stores `int.from_bytes(blake2b(line, digest_size=8).digest(), "big", signed=True)`, i.e. a big-endian signed i64. Checked in `src/relic4/engine.py:364`.
  - BLAKE2 runs about 1.1 GB/s per core (xxHash table, i7). At that rate 180 GB over 14 cores takes about 13 s. https://github.com/Cyan4973/xxHash
- **msgkey:** `md-5` over the same JSON array as `sql.MSGKEY`.
- **Reading:** a `BufReader` with a 1–8 MiB buffer plus a `memchr` line split, reusing one `Vec` for long lines. *No memmap2 vs BufReader benchmark found.*

## Redaction (must match the Python output byte for byte)
- **Unicode differences.** Rust `regex` is Unicode-aware by default for `\b`, `\s` and `(?i)` (Kelvin sign, long s). Go RE2 and Python `re.ASCII` are not. Build with `regex::bytes::RegexBuilder::unicode(false)`. https://docs.rs/regex/latest/regex/#syntax
- **Matching is linear-time,** but `replace_all` can be O(m·n²) in the worst case. Find the private key's BEGIN marker, then use `memmem` for END, instead of the lazy `[\s\S]*?` regex. https://docs.rs/regex/latest/regex/struct.Regex.html
- **Prefilter** with one `aho-corasick` leftmost-first automaton over the hint literals (packed SIMD on aarch64). Run only the rules whose hints hit, **in the original rule order**: the openai rule can match inside `sk-ant-…`. https://docs.rs/aho-corasick/latest/aho_corasick/packed/index.html
- **Labels count chars**, not bytes: `.chars().count()` and `.chars().take(4)`.

## Output: which format is fastest
Nobody published numbers for this workload, so the ranking below is partly inference, and BENCH.md measures it on ours.

| rank | format | evidence |
|---|---|---|
| 1 | Arrow IPC, uncompressed or lz4 | zero-copy layout (no measurement found) |
| 2 | Parquet, no compression or lz4/snappy | uncompressed 758 MB/s vs lz4/snappy about 110 MB/s (int64, Ampere) https://linaro.atlassian.net/wiki/spaces/BDTS/pages/28496232625 |
| 3 | Vortex | vendor claims 5× Parquet (https://spice.ai/blog/vortex-at-spice-ai-the-columnar-format-for-data-intensive-workloads) |
| 4 | Parquet zstd-1 | 46 MB/s (same Linaro test) |
| 5 | duckdb-rs Appender | ART/WAL/checkpoint overhead; list columns not confirmed https://duckdb.org/docs/2.0/data/appender.html |
| 6 | Lance | no write data found |

DuckDB reads Parquet natively (`read_parquet`).

**Dedupe:** every thread writes raw per-month parts, duplicates included. A final DuckDB `GROUP BY s, h, idx` with `list(source)` then builds the month file. DuckDB hash aggregation spills to disk; whether `list()` spills is untested. https://duckdb.org/2023/09/26/announcing-duckdb-090

## Prior art (borrow vs build)
- **agent-sessions** (Rust, MIT) parses Claude Code, Codex and about 24 more agents, but single-threaded.
  - Borrow its 8 MiB per-line limit with draining, its incomplete-trailing-line policy, and leaving missing counters as `None`.
  - Do not infer subagents from paths alone. https://github.com/majiayu000/agent-sessions
- **toolpath-codex** (Rust, Apache-2.0) has typed Codex `RolloutLine` structs: `session_meta`, `response_item` (message, reasoning, function_call, custom_tool_call and their outputs) and `event_msg`. Codex reasoning is `encrypted_content`. https://docs.rs/toolpath-codex
- **ccstats** (Rust, MIT, built on agent-sessions): a per-file cache of parsed facts, so unchanged files are skipped. https://github.com/majiayu000/ccstats
- **No surveyed tool** does parallel files plus cross-file dedupe plus month output. That part is ours.

## Decision for the lab
1. Walk with `ignore::WalkParallel` and dedupe by (dev, ino). Benchmark it against a `getattrlistbulk` walker on an M5 Mac.
2. Sort by size, biggest first, into one atomic queue on plain threads. Files over 64 MB become newline-aligned chunks.
3. Read with `BufReader` (4 MiB) and `memchr`. Parse with `sonic-rs`, hash with `blake2b_simd` (8 bytes, big-endian i64), key with `md-5`.
4. Redact with ASCII `regex::bytes` behind an `aho-corasick` hint prefilter, in rule order. Private keys use `memmem`.
5. Each thread writes per-month parts as **Arrow IPC or Parquet**, both measured on one real month. DuckDB `GROUP BY` builds `truth/YYYY-MM.duckdb`.
6. Prove it: on the M5 Mac's own files, the set of `h` must equal the truth rows whose `machines` contain `dev@mac`.
