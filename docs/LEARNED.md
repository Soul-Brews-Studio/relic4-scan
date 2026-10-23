# relic4-scan — what relic v1, v2/2.1, v3 and relic-lessons teach (2026-10-22)

Four Sonnet readers went through the repos read-only and cited file:line. What follows is what changes relic4-scan. "→" marks the decision.

## Discovery (phase 1)
- **No earlier relic searched the machine for homes.**
  - v1, v2 and v3 ran on a hard-coded list of 3–7 roots: `agents-relic/src/sources.ts:160-207`, `agent-relic-v2/src/sources.ts:140-175`, `agent-relic-v3/src/read/default-sources.ts`.
  - A missing path was skipped silently.
  - → Phase 1's name- and shape-based home search is new. That is the point of this lab.
- **Never follow directory symlinks.** v1's Python `is_dir()` followed them and found 68,719 vault notes instead of 10,129 (agents-relic README). A symlinked ψ was skipped in one round and double-counted in another (`relic-lessons/rounds/round-09-10.md:9`).
  - → `follow_links(false)` plus a (dev, inode) dedupe. Already done.
- **Case-only folder pairs exist.** white has 67 of them. macOS folds them onto one inode (`agents-relic/src/repo.ts:129`) and v3 refuses a case-insensitive mirror (`agent-relic-v3` commit `c58c587`).
  - → Dedupe by inode, never by lower-cased path.
- **Mirrors go stale, and the same session sits in several banks** (`relic-lessons/OPEN.md:17`). v3 records an `"origin"` per source, so a mirrored file keeps the path it has on its origin machine (`agent-relic-v3/hub/README.md:3-27`).
  - → Every home gets an `origin`: own / mirror / rescue / archive / download.
- **`.omx-runs` is an ops log, not conversation** (`agents-relic/src/sources.ts:122`).
  - → It gets its own kind, `omx-ops`, not `omx`.
- **Antigravity (Google's agent IDE)** writes `~/.gemini/antigravity*/brain/<id>/.system_generated/logs/transcript_full.jsonl`, with a compact `transcript.jsonl` beside it (`agent-relic-v2/README.md`).
  - → Prefer `_full`, or every message is counted twice. Still untested on a real session.
- **Scope discovery before stat.** A scoped walk took 0.33 s for 8.8k files, against 3.2–5.1 s for a full walk of 215k (`relic-lessons/CORE.md:27`).
  - → Add `--since` later. The full walk on m5 is already 8 s for 91k files.

## Parse (phase 2)
- **Each line goes in the period of its own timestamp,** never its session's start. 561 long sessions held 22% of all lines, 1.69M of 7.68M (`agent-relic-v2.1` commit `6a7fe76`, `relic-lessons/CORE.md:49`).
  - → Bangkok month per line. Our engine already does this.
- **Impossible dates go to quarantine.** v3's rule: before 2023-01-01, more than now + 2 days, or unparseable goes to `_quarantine`, and is kept and counted (`agent-relic-v3/src/contracts/time.ts:49-57`). Real old dates must not be mislabelled as bad (`relic-lessons/OPEN.md:32`).
  - → A `quarantine` month bucket, separate from `undated`.
- **Tool results arrive as `user` messages.** Label by block type. Before that fix, 68.5% of indexed "text" was tool output (`agent-relic-v2.1/src/shapes/claude.ts`).
  - → Our `content_text` takes only text blocks. Keep it that way.
- **Never decode the cwd from the folder name.** The encoding is lossy (`/` and `.` both become `-`), so read `cwd` from the record (`agents-relic/src/shapes/claude.ts:38`, `agent-relic-v3/src/cwd.ts`).
- **A truncated Codex rollout** with no `session_meta` takes its session id from the filename (`agents-relic/src/shapes/codex.ts:21`).
- **No silent failure.** Name every parse failure with file, line and reason (`relic-lessons/CORE.md:41`). One parser per shape, each with a golden fixture (`CORE.md:42`).
  - → A `failures.tsv` beside the month parts.
- **Thinking:** 96% of Claude thinking blocks are empty (signature only), which is why v3 dropped them (`agent-relic-v3/docs/DECISIONS.md` D2). We keep the non-empty ones; jsonl-truth has 131k thinking rows.

## Identity and dedupe
- **A uid in every earlier relic named a slot,** `sha1(home, shape, key, seq)`, never the content (`agent-relic-v3/src/contracts/ids.ts:4-23`). One session held in two homes had 13,347 lines in one and 9,098 in the other (`agent-relic-v2.1/src/types.ts:131`).
- **No version collapsed copies at ingest.** They folded at read time by (ts, role, text) (`relic-lessons/rounds/round-09-10.md:26-38`).
  - → Our line-hash dedupe is safe only because truth keeps every `agent:path:line` source. The scan must keep them all too. msgkey is the (ts, role, text) fold made explicit.
- **Basename and short-prefix ids collided:** 374 basename pairs, and an 8-character Codex prefix (#145) (`relic-lessons/LESSONS.md:74-75`).
  - → Locators are full paths. A sid is shown short but never matched short without a uniqueness check, which relic4 `show` already does.

## Speed: where the time actually goes
- **Rust alone does not help.** A 1:1 Rust port of an earlier relic ran at about the same speed, and the store (LanceDB) was the bottleneck (`relic-lessons/LESSONS.md:33-35`).
  - → The gain comes from not reparsing, plus a small write unit.
- **Quadratic line re-splitting** took 984 s of a 1,302 s index (`agent-relic-v3/src/ingest/helpers.ts:62-67`).
  - → `memchr` over one buffer, every line a slice, nothing re-split.
- **Rewriting a whole period costs.** Whole-week rewrites took 24.97 s against 10.17 s, and one run reached 20.2 GB RSS (`relic-lessons/LESSONS.md:25-27`).
  - → Per-month **parts** plus a delta. An unchanged month is never rewritten.
- **Offset-based tail ingest was never built in any version** (`relic-lessons/OPEN.md:37`).
  - → Phase 2 stores the byte offset after the last complete line, plus a hash of the first 4 KB. Next run: same prefix and bigger means parse only the new tail; different prefix means it was rewritten, so reparse it.
- **Unchanged-file skip by (size, mtime) works** in v3 (`agent-relic-v3/src/status/pending.ts:13`). It is the floor; the offset makes grown files cheap too.
- **Publish a manifest last,** with an atomic rename (`agent-relic-v3/src/index/writer.ts:17-30`). A lock from a dead pid must not block forever (`agent-relic-v3/src/index/lock.ts`).
- **Months vs weeks:** v2.1's monthly index beat v3's weekly one on Thai MRR@10 (0.764 vs 0.600) and on whole-index search (481 ms vs 948 ms) (`agent-relic-v3/bench/acceptance/README.md:12-15`).
  - → Month files are the right unit for truth.

## Baselines to beat (prior relics)
| | value | source |
|---|---|---|
| corpus | 215k files · 34 GB · p99 file 39.6 MB | relic-lessons/CORE.md:26 |
| full index | v3 351.9 s (6.7 GB RSS) · v2.1 430.7 s | relic-lessons/TIMELINE.md:35,41 |
| incremental, 37 changed files | 111 s → 43 s after PR #63 | relic-lessons/book/04-write.md:72 |
| line reader | 0.62 s per 1.44 GB | relic-lessons/CORE.md |
| phase 1 here (m5, beta) | **91,293 files · 76.1 GB · 8.0 s** | this lab |

## Validation method (copy it)
- v2's `VALIDATION.md` ran an old-vs-new parity check on a synthetic and an APFS-cloned real corpus: exact identity, text and counts, byte-identical archives, paired medians, and the failed run kept (`agent-relic-v2/VALIDATION.md`).
- → relic4-scan's proof:
  - beta@m5's `h` set must equal the truth rows whose `machines` contain `beta@m5`;
  - text, tools and redaction must be byte-equal on every shared `(s, h, idx)`;
  - every difference is listed, never summarised away.
