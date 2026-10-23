# relic-scan

One Rust binary, `relic-scan` (this repo is still named relic4-scan). It finds every AI-agent transcript on a machine, writes **one Parquet file per Bangkok month** in the relic4 truth schema, and writes the meta beside them: files, sessions, and every `.jsonl` on the machine. The design is in RESEARCH.md, the lessons from relic v1–v3 in LEARNED.md, and the exact output rules, traced and verified, in OUTPUT-CHAIN.md.

Grown in the agent-relic-v4 lab (`lab/relic4-scan` on branch `lab-rust-scan`, history kept). [agent-relic-v4](https://github.com/Soul-Brews-Studio/agent-relic-v4) is the reader: it turns these month files into truth, meta, vectors and search. The verify scripts need its Python environment: `uv run --project <agent-relic-v4> python verify/…`.

## Use it
**Install.** One command. It needs Rust 1.89 or newer and a gh login, because the repo is private and cargo fetches it through the git CLI. `install.sh` finds cargo even when it is not on PATH, builds in about 30 s and installs to `~/.local/bin/relic-scan`.
```
gh api repos/Soul-Brews-Studio/relic4-scan/contents/install.sh -H 'Accept: application/vnd.github.raw' | sh
```
To install a branch instead of `main`, pipe to `REF=<branch> sh`. Run the same line again to upgrade.

**Run.**
```
relic-scan                         # the logo, what is indexed already, and the commands below
relic-scan up --kind claude        # index Claude transcripts into ~/.relic-scan/claude, then serve http://127.0.0.1:8787
relic-scan tui --kind claude       # the same data in the terminal
```
The first run reads everything: on m5 that is 83,673 files and 67.6 GB in about 45 s. After that a run reads only new bytes and takes about 10 s, most of it spent walking the homes. Without `--kind`, every agent is indexed into `~/.relic-scan`.

| command | what it does |
|---|---|
| `relic-scan up [--kind K] [--port 8787]` | `run`, then `serve` |
| `relic-scan run [--kind K] [--out DIR]` | Scan, parse only new bytes, then write meta, map and paths. It ends with a summary card. |
| `relic-scan log [--diff N]` | The scan log: every index and check, newest first; `--diff N` lists one entry's files. |
| `relic-scan run --dry-run` | Only check: scan, compare with the state, and print what an index would read (new, grown, rewritten, gone, files, GB). It writes nothing and takes no lock. |
| `relic-scan run --month 2026-10` | Only rows of those Bangkok days, in a folder of their own. Also `--day 2026-10-22`, or `--from` and `--to` (each a day or a month, either one optional). See [Only some days](#only-some-days). |
| `relic-scan serve [--kind K]` | Serves `/map`, `/tree` and `/paths` on 127.0.0.1 only. The page is read on every request, so a `run` in another shell shows up on reload. The page's **⟳ index** button runs an incremental run in the server (`POST /api/run`, which needs an `X-Relic: 1` header; progress at `GET /api/status`). **Check changes** on Smart Scan is the same scan as a dry run (`POST /api/check`, same header): it shows what Index now would read and writes nothing. Before the first run, every view shows a welcome page with the same button. |
| `relic-scan tui [--kind K]` | The terminal browser (below). |
| `relic-scan meta` / `map` | Rebuild only the meta tables, or only the page. |
| `relic-scan scan [--full] [--index F]` / `parse --index F --out DIR` | The two phases alone, for debugging and benchmarks. |

**Defaults and safety:**
- **`--out`:** defaults to `~/.relic-scan`. With `--kind` and no `--out` it is `~/.relic-scan/<kind>`, so a kind-only run never shares its state with a run over every agent. `serve`, `tui`, `meta` and `map` take the same `--kind` to find it.
- **One writer:** a run takes a lock on `OUT/.lock`. A second `run`, `meta` or `map` on the same directory stops with a message instead of racing the first.
- **Crash safety:** month files, meta tables and `_state.tsv` are written to a temporary file, fsynced, then renamed. The state goes last, so a crash costs a re-read, never a lost or doubled row.
- **Peak memory:** files are read in 32 MiB chunks (`RELIC_SCAN_CHUNK` bytes to change it). The biggest files are parsed side by side, so reading them whole used to hold gigabytes at once.
- **Local only:** the server binds 127.0.0.1, and the page makes no network calls.

**Desktop app, ARRA Relic Scan (Tauri v2):** `desktop/` wraps the same page in a native window.
- **How it runs:** it starts the server inside its own process on a free 127.0.0.1 port and opens a window on it.
- **Index button:** runs an incremental run in that process. On a first launch the window shows a welcome page with the button.
- **`RELIC_SCAN_KIND`:** `all` (the default, the same folder as `relic-scan run`), `claude`, or a comma-separated list.
- **`RELIC_SCAN_OUT`:** overrides the output directory.
```
cd desktop && cargo build --release && ./target/release/relic-scan-desktop    # run it (--view banks|health|map|tree|paths)
cargo install tauri-cli --version '^2' --locked && cargo tauri build          # ARRA Relic Scan.app
```
**Installer:** `cargo tauri build --bundles app,dmg` also makes `target/release/bundle/dmg/ARRA Relic Scan_<version>_aarch64.dmg`, a drag-to-Applications window (660x400, the app at (180,190) and Applications at (480,190), the layout Oracle Pulse uses). Its background is `desktop/dmg/background.tiff` (1x and 2x), made by `desktop/dmg/make-background.sh`; it is light because Finder draws the icon labels in black.

How it compares with Relic Scanner and Relic Studio, the SwiftUI apps over relic3: [docs/COMPARE.md](docs/COMPARE.md).

**Build from a checkout instead:**
```
cargo build --release && cargo test --release
./target/release/relic-scan up --kind claude
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

## Only some days
`--month`, `--day`, `--from` and `--to` keep only the rows whose Bangkok date is in the range. A row with no time at all is left out.
- **Its own folder:** without `--out`, the run writes `~/.relic-scan/<kinds or all>@<range>`, for example `~/.relic-scan/all@2026-10` or `~/.relic-scan/claude@2026-10-22`. Pass that path as `--out` to `serve`, `tui`, `meta` and `map`; the summary card prints it.
- **Why a folder of its own:** the rest of each file is still read, and the state moves past it, so those rows never come back into that folder. For the same reason:
  - a run with a range stops if its folder was indexed without one (`~/.relic-scan` itself, for example);
  - the folder keeps its range in `OUT/_range`, and a run asking for a different range stops;
  - a run with no date flags on such a folder (including the page's index button) uses the folder's range.
- **Incremental like any run:** a second `--month 2026-10` reads only new bytes.
- **Files untouched since before the range are skipped** without being read: a line is written when it happens, so such a file holds no row in the range. There is one day of slack for clocks.
- **Not the same rows as filtering the full index:** two identical lines are one row, kept at its first copy. If the first copy is outside the range, the range folder keeps the later copy, while the full index has only the earlier one.

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
| `DIR/meta/files.parquet` | file in the state | agent, path, home, size, lines, last_ms, file_sid, base, tier, engine, proj, parent_sid, cat, n_rows, text_rows, first_ts, last_ts, plus rows by type |
| `DIR/meta/sessions.parquet` | file name (base, tier), copies folded | base, tier, sid, parent_sid, engine, proj, cat, probe, first_ts, last_ts, last_ms, copies, machines, agents, size, n_rows, text_rows, text_msgs, months, plus rows by type |

Rows by type are six columns: user_text, assistant_text, thinking, tool_calls, tool_results and tool_errors. Errors are results that failed, so they are also counted in tool_results.

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

## Views: Smart Scan, Banks & Repos, Health, map, tree, paths and the TUI
`serve` (or `up`) serves one page on 127.0.0.1, in the shape of Relic Studio. A left sidebar holds six views; under 1100 px it folds to icons. At its foot it shows whether a run is going and when the last one was.

- **Smart Scan** (`/`, `/scan`): a coverage ring (transcript files indexed out of those found), the **Index now** and **Check changes** controls with a live progress bar, and two stat strips: files indexed, discovered, not transcripts and outside homes; then sessions, rows, indexed size and last run.
  - **Scan vs index:** a scan walks the folders and compares every `.jsonl` with `_state.tsv` by size and inode, reading no content. An index is that scan, then reading only the new bytes, rewriting the months that got rows, and rebuilding meta, map and paths. **Check changes** runs only the scan: new, grown, rewritten, gone and unchanged files, and the files and GB that **Index now** would read. It writes nothing; the result stays on the page until the next index.
  - **Contents:** a sunburst of every row. The rings are what the row is (user text, AI text, thinking, tool calls, tool results, tool errors, other), the kind of `.jsonl` that holds it (main session, subagent, workflow agent), and where that file lives (own home, mirror, archive, rescue, copy). Click a slice to go in and the centre to go back; "file kind first" swaps the first two rings. A line copied into several homes is one row, under the first home.
  - **Wide screens** add a side column with the totals and the latest scans.
  - **Show output** opens a popup with the server's own output during a run or check (the scan line, the parse bar, the summary card), streamed from `GET /api/log`.
- **⌘K search** (or `/`, or the search button top right): one box over what the page holds: views, actions (index, check, output, contents, theme), sessions (by id, repo or project; empty shows the latest), repos, homes, every `.jsonl` path, scan-log entries, and filters in the map/tree query language. Each result opens its view; some carry actions: **open folder** (shows the file in Finder; the server refuses any path the index does not hold), copy the resume command, copy the path, and **check** or **index** just that file or home (`only=` on the API). ⌘↵ runs the first action.
- **Stop:** while an index or check runs, Stop (on Smart Scan and in the output popup) ends it during the walk or the parse with nothing written; the next run reads the same bytes again. Once the merge has started it finishes.
- **Settings** (`/settings`): the folders to walk, paths to skip and the depth, edited as chips (type and Enter, a suggestion, or **Choose folder…**, which opens the desktop's own folder picker through the server), an automatic Check changes every 15, 30 or 60 minutes while the server runs (it never indexes on its own), and the theme. Saved in `OUT/_config.json` (`GET`/`POST /api/config`); `relic-scan run` and Index now use it, and a flag on the command line wins.
- **Sidebar:** Tree and Paths sit under Session map. **Disk** is a gem: hidden until it is turned on in Settings > Gems (this browser only).
- **Scan log** (`/scans`): every index and check from `OUT/_scans.jsonl`, newest first: new, grown, rewritten and gone files, bytes read, time, and rows added. Open one to see its files (the 2,000 biggest reads are kept per entry). `relic-scan log` prints the same list, and `relic-scan log --diff N` lists the files of entry N.
- **Banks & Repos** (`/banks`): one card per jsonl home with sessions, rows, files, size, origin and newest session; a card opens the tree on that home. Below them is a repository index with org/repo, sessions and rows; a row opens the map on that repo.
- **Health** (`/health`): checks computed from the index, each with a link to the matching view:
  - the age of the last run;
  - transcripts outside every home;
  - subagents without a main session;
  - undated sessions;
  - indexed files with no rows;
  - files skipped as not transcripts;
  - unknown shapes;
  - mirrors folded;
  - tool errors.

Map, tree and paths are below. Map and tree share the query box, the side panel and the URL hash, so a query carries over. The theme button cycles auto, dark and light; auto follows the system. In tree and paths only the page scrolls; the side panel sits beside it, never inside it.

`run` rebuilds the page (`DIR/meta/map.html`, about 4 MB for 43,006 sessions, data inlined) and `DIR/meta/paths.json`. `relic-scan map` rebuilds both alone in 0.2 s. Opened as a file, `map.html?view=tree` shows the tree. The paths view loads `/paths.json`, so it needs the server.

**/map, the star map:**
- **Ring:** the Bangkok month the session started. The oldest month is innermost; a stronger ring marks the start of a year.
- **Sector:** the repo. Wider sectors have more sessions; repos with fewer than 12 sessions share one sector.
- **Star:** a main session, sized by log(rows). Subagents and workflow agents orbit their parent, found by the `<uuid>/subagents/` folder they sit in. 99% of them have a parent.
- **Colour:** by tier, repo, age, the share of tool rows, or thinking.
- **Query:** plain words match repo, project or sid. Filters:
  - `repo:` `proj:` `sid:` `home:` `tier:main|sub|wf`
  - `month:2026-07` `after:2026-09-01` `before:`
  - `rows>1000` `text` `msgs` `copies` `user` `asst` `think` `calls` `results` `errors` `tools`, e.g. `think>0 errors>=20`
  - `-` before a term negates it, `"..."` quotes a value, and all terms must match (AND).
- **Panel:** totals, rows by type, top repos and jsonl homes of the current match, each clickable to narrow the query. Clicking a star shows that session: sid, repo, times, rows by type, copies, homes, satellites, and buttons to copy the sid or a `claude --resume` line.

**/tree:** jsonl home → repo → session → satellites. Each level shows sessions, rows, a bar of rows by type and the last activity. A session is listed under every home that holds a copy. A result of up to 400 sessions opens itself down to the repos.

**/paths:** every `.jsonl` the scan saw, as a folder tree. Chains of single folders fold into one row, and `~` stands for your home. Each folder shows files, rows, size and a status bar:
- **indexed:** parsed into the months.
- **skipped:** in an agent home, but the content sniff says it is not a transcript (workflow journals, prompt logs, omx logs).
- **vault:** in an oracle's ψ vault, listed only.
- **loose:** anywhere else under the roots, listed only.

Query words match the path. Filters: `is:indexed|skipped|vault|loose`, `verdict:not-transcript`, `name:agent-`, `mb>100`, `size>1e9`, `rows>10000`, `rows=0`, `after:` and `before:` (modified, Bangkok time). A small result opens every folder it touches. Clicking a file shows its status, verdict, size, rows and modified time, with a button to copy its path.

**`relic-scan tui`:** the same data in the terminal, with four views on the number keys:

| key | view |
|---|---|
| `1` | overview: totals, rows by type, sessions per month by tier |
| `2` | repos: sessions (main / sub / workflow), rows, thinking, errors, last activity. Enter lists the repo's sessions. |
| `3` | sessions, newest first. Enter opens one; `c` copies `claude --resume <sid>`. |
| `4` | paths: folders holding `.jsonl`, with indexed / skipped / other. Enter lists the folder's files; Enter on a file copies its path. |

Other keys:
- `/` filters the list (words, AND) and `Esc` clears the filter.
- `↑↓`, `j k`, `g G` and `PgUp PgDn` move.
- `⌫` goes back and `q` quits.

It sets no background colour, so it reads the same on dark and light terminals. Colour in the CLI output appears only on a terminal and is off with `NO_COLOR`.

## Source layout
No file is longer than 500 lines; a long one is split into a folder of modules.
- `src/run.rs`: `relic-scan run` (scan, diff, parse, merge, meta, map, scan log); `src/lib.rs` keeps the arguments and the shared helpers.
- `src/parse/`: the reader (`mod.rs`), the raw parts (`parts.rs`), the month merge (`merge.rs`), tests.
- `src/meta/`: the tables (`mod.rs`), the path rules (`rules.rs`), the per-month pass and its cache (`agg.rs`), tests.
- `src/tui/`: state and loading (`mod.rs`, `load.rs`), keys (`keys.rs`), drawing (`draw.rs`).
- `src/page/`: the page as small files: the shell `index.html`, three CSS files, and the scripts in the order they run
  (`core`, `map`, `shell`, `tree`, `paths`, `dash`, `contents`, `disk`, `scanlog`, `settings`, `main`, `palette`). `map.rs` puts them
  back together into one self-contained page; there is no build step and no network. Shared helpers live in `core.js`:
  `human()` for bytes, `relicPost()` for every write to the server, and `overlay()` for every popup.

## Tests
`cargo test` runs 22 tests in under a second, with no other repo needed:
- the path rules: tier, repo (cat), project, parent and home;
- `splitlines` and the jt cut of a last line;
- the message key and the meta cache line;
- **chunked reading:** chunk sizes from 1 byte to 4 KB give the same rows as a whole-file read;
- **incremental == cold:** a file grown three times, once cut mid-line, gives the same month file as one cold parse;
- **scan log:** entries are numbered by line, a torn line is skipped, and a big diff keeps its biggest reads;
- **output capture:** a line redrawn with `\r` replaces itself;
- **settings:** a save keeps what is valid and refuses an empty or missing folder, a depth outside 1-32 and an odd auto-check interval, leaving the file as it was;
- **server:** query strings decode, export month names are names (never paths), and the page is served for any one-word view;
- **dry run:** a check reports one new file and writes nothing; after an index it reports one unchanged file; after an append it reports exactly the appended bytes;
- **date ranges:** the flags become days (bad dates and a `--from` after `--to` are refused); rows are kept by Bangkok day; a folder keeps its range and a full index never takes one; a one-day run gives exactly that day's rows of a full parse, and still reads to the end of the file.

The full-corpus proofs (`verify/*.py`) compare against agent-relic-v4 and need its Python environment. In addition, a 4,023-file fixture (7.1 GB, including the 24 biggest files) gave every month, `files.parquet`, `sessions.parquet` and `jsonl.parquet` equal to the previous binary's. Peak RSS fell from 5.81 GB to 3.72 GB at the same speed.

## Not yet
- **A quarantine month** for impossible dates.
- **Antigravity `transcript_full.jsonl` preference.**
- **Splitting multi-GB files** into chunks.
- **Arrow IPC vs Parquet** final-output benchmark.
