# AGENTS.md: ARRA Relic Scan, for AI agents

Instructions for an AI coding agent (Claude Code, Codex, Gemini CLI, Cursor, …) that is asked to install, build, run, query or
change ARRA Relic Scan. Humans: the [README](README.md) has the same facts at more length.

## What it is

`relic-scan` finds every AI-agent transcript on a machine (Claude Code, Codex, OMP, OMX, Gemini, Pi, thClaws: any `.claude`,
`.codex`, `.omp`, `.omx`, `.gemini`, `.pi` or `.thclaws` home, and backups or mirrors of them) and every oracle `ψ` vault. It
indexes the transcripts into one Parquet file per Bangkok month under `~/.relic-scan`, and serves a page to browse them on
`127.0.0.1` only. It never changes a transcript and makes no network calls, except `relic-scan export --to s3://…`, which you run yourself. A desktop app (Tauri) wraps the same page.

## Install (pick one)

```sh
# the command, built from source (needs Rust 1.89+; installs ~/.local/bin/relic-scan)
curl -fsSL https://raw.githubusercontent.com/Soul-Brews-Studio/relic4-scan/main/install.sh | sh

# or from a checkout, with just (https://github.com/casey/just)
git clone https://github.com/Soul-Brews-Studio/relic4-scan && cd relic4-scan
just install
```

Desktop app downloads (macOS Apple silicon and Intel `.dmg`, Linux `.deb` and `.AppImage`):
https://github.com/Soul-Brews-Studio/relic4-scan/releases

## Run it

```sh
relic-scan run                  # index: scan, parse only new bytes, write months + meta (first run ~1 min, then ~15 s)
relic-scan run --dry-run        # what an index would read; writes nothing
relic-scan up                   # index, then serve http://127.0.0.1:8787 (Smart Scan, map, tree, ψ Vaults, Export)
relic-scan tui                  # the same index in the terminal
relic-scan run --kind claude    # one agent only, into ~/.relic-scan/claude
relic-scan run --month 2026-10  # one month only, into ~/.relic-scan/all@2026-10
```

Ask the human before the first `relic-scan run` on their machine: it reads every transcript under the home folder (read
only) and writes its index into `~/.relic-scan` (3.1 GB for 5.5 M rows on one Mac).

## Query the index (DuckDB, Polars, pandas)

Paths are relative to `~/.relic-scan`. Month files hold the rows; `meta/` holds one line per session and per file.

| file | one row per | useful columns |
|---|---|---|
| `YYYY-MM.parquet` | message, thinking block, tool call or result | `sid`, `engine`, `ts`, `ts_th` (Bangkok), `role` (user, assistant, thinking), `kind` (call, result), `name` (tool), `text`, `body`, `err`, `sources`, `model`, `msg_id`, `out_tokens` |
| `meta/sessions.parquet` | session | `sid`, `parent_sid`, `tier` (main, subagent, workflow-agent), `cat` (repo), `proj`, `first_ts`, `last_ts`, `last_ms`, `n_rows`, `user_text`, `assistant_text`, `thinking`, `tool_calls`, `tool_results`, `tool_errors`, `cwd` (where the session began), `title` (its newest `custom-title`), `branch`, `version`, `via`, `chat`, `from_user` (the channel plugin, room and sender of the session's first channel message, e.g. `plugin:discord:discord`) |
| `meta/files.parquet`, `meta/jsonl.parquet` | transcript file | path, home, size, verdict |
| `meta/months.json` | month file | `month`, `rows`, `first_ts`, `max_ts`, `keys` (hash of the row keys: equal manifests, equal rows), `late_rows` (rows this run added at or before the month's previous newest time; normally 0), `sealed` (before the current Bangkok month), `sealed_at`/`sealed_rows`/`sealed_keys`/`resealed` (the month's seal: when it was first seen closed, its rows and key hash then, and how many times a late row made it seal again; the month file is made read-only; `relic-scan seals --verify` reads each sealed month again and checks it) |
| `meta/vaults.json`, `meta/vaults/<id>.json` | ψ vault, and its notes | plain JSON: name, path, kind, notes, sections; notes as `[path, section, title, bytes, mtime_ms]` |

**Start here: count everything once, by what it is.** The total is the number the app shows on Smart Scan; report a
slice (tool results, user text) only next to it.

```sh
cd ~/.relic-scan
duckdb -c "select count(*) as all_rows from '*.parquet'"
duckdb -c "select coalesce(nullif(kind, ''), nullif(role, ''), '(none)') as what, count(*) as n from '*.parquet' group by 1 order by n desc"
duckdb -c "select engine, count(*) as n from '*.parquet' group by 1 order by n desc"   # claude, codex, omp, thclaws, …
```

**How a row says what it is (two columns, not one):**
- a message or thinking block has `role` = `user`, `assistant` or `thinking`, and no `kind`;
- a tool call has `kind = 'call'` (the tool in `name`), a tool result `kind = 'result'` (`err` true when it failed), and no `role`;
- a thinking row with **empty text** is a thinking block whose text was not saved: since about March 2026 Claude Code writes
  most thinking blocks as a signature only. Count them as thinking; there is nothing to read or embed;
- a handful of rows have neither role nor kind (empty strings): leave them out or show them as "(none)".

**One row per message.** Copies of a transcript in several homes (a mirror, an archive, a backup) are folded into one row,
even when the copies differ in metadata such as the recorded cwd; `sources` lists every copy (`agent:path:line`).

**Sessions.** Every row's `sid` is in `meta/sessions.parquet` except rows written with no session id at all (49 of
5.35 M on one Mac). A Claude session's id is its file name; omp subagents (a task's transcript inside its parent session's
folder) are sessions of tier `subagent` with `parent_sid` set.

**Which model answered.** Assistant text, thinking and tool-call rows carry `model` (Claude `message.model`, omp
`message.model`, Codex the model its last `turn_context` named). Claude and omp rows also carry `msg_id`, the API
response they belong to (Claude `message.id`, omp `responseId`): one response is several rows, one per content block.
`out_tokens` (the response's output tokens) is on **one row per `msg_id`**, so `sum(out_tokens)` counts each response
once and `count(distinct msg_id)` counts API calls. Codex rows have a model but no `msg_id` or `out_tokens` yet. User
text and tool results have none of the three.

Indexes written by v26.10.23-alpha.1117 or earlier differ (fixed in v26.10.23-alpha.1308): omp tool results were also text rows (`role = 'toolResult'`), empty
thinking blocks were dropped, mirrored copies that differed in one field were separate rows, and resumed Claude sessions
and omp subagents were missing from `meta/sessions.parquet`. Indexes written by v26.10.23-alpha.1858 or earlier have no
`model`, `msg_id` or `out_tokens` (the columns read as NULL), and `meta/sessions.parquet` has no `cwd`, `title`, `branch` or `version` until the files are read again. Re-index from scratch
(move `~/.relic-scan` aside, then `relic-scan run`) to get the current shape.

```sh
# sessions per repository
duckdb -c "select cat as repo, count(*) as sessions, sum(n_rows) as n_rows from 'meta/sessions.parquet' group by 1 order by sessions desc limit 10"
# what the human asked about something, newest first
duckdb -c "select sid, ts_th, left(text, 120) as text from '*.parquet' where role = 'user' and text ilike '%parquet%' order by ts desc limit 20"
# the tools used most
duckdb -c "select name, count(*) as calls from '*.parquet' where kind = 'call' group by 1 order by calls desc limit 10"
# API calls and output tokens per model, per month
duckdb -c "select left(ts_th, 7) as month, model, count(distinct msg_id) as calls, sum(out_tokens) as out_tokens from '*.parquet' where model is not null group by all order by month, calls desc"
# the newest session, read in order
duckdb -c "select role, left(text, 200) as text from '*.parquet' where sid = (select sid from 'meta/sessions.parquet' order by last_ms desc limit 1) and role in ('user','assistant') order by ts"
# ψ vaults with the most notes
jq -r '.vaults[] | select(.kind != "worktree") | [.notes, .name, .path] | @tsv' meta/vaults.json | head
```

Transcripts hold private work. Read what the human asked you to find; do not copy rows out of the machine.

## Copy the sealed months to an S3-compatible store

```sh
export RELIC_S3_ENDPOINT=http://odin.follow-rankine.ts.net:3920   # unset = AWS S3; Garage and MinIO need an endpoint (path-style)
export AWS_ACCESS_KEY_ID=… AWS_SECRET_ACCESS_KEY=…                 # never on the command line; RELIC_S3_REGION (default us-east-1)
relic-scan export --to s3://bucket/relic --dry-run                 # HEAD only: says what would be sent
relic-scan export --to s3://bucket/relic                           # sealed months, meta/months.json, export-manifest.json
relic-scan export --to s3://bucket/relic --verify                  # compare the store with the files; exit 1 if they differ
relic-scan export --to s3://bucket/relic --list                    # what the store holds: one line per month (rows, dates, size, sha256, seal); reads two small files, lists nothing
```

The app and `relic-scan serve` have the same thing on the **Export** view ("S3 copy": Check against S3, Send sealed months, each month
listed as identical, older or missing); the destination is set in **Settings** (`s3.to`, `s3.endpoint`, `s3.region` in `_config.json`,
never a key). The server, not the page, makes the S3 requests (`GET /api/s3`, `POST /api/s3/check` and `/api/s3/send`, which need the
`X-Relic` header); the page still makes no network calls. Keys: the environment, or `~/.config/relic-scan/s3.env` (KEY=VALUE lines,
mode 600; a file others can read is refused), which is what an app started from Finder uses.

Only closed (sealed) months go, then `meta/months.json` (the seals) and `export-manifest.json`. Each object stores its sha256 in
`x-amz-meta-sha256`, so a re-run sends nothing that is already there. A sealed month that differs in the store (it was sealed
again after a late row) is refused unless `--overwrite`. Plain http is allowed only to this machine or the tailnet
(`*.ts.net`, `100.64/10`), else `--insecure-http`. Nothing is listed or deleted in the store.

## The ledger: what has been read, from anywhere

`~/.relic-scan/_ledger.tsv` is appended on every index and never rewritten. Each line is one version of one transcript:
`first` (hash of its first line) · `lines` · `bytes` · `chain` (a hash chain over every line) · `session` · `last_ms` ·
`path` · `at` · `how` (read, copy, seed). Before a new or changed path is parsed, the ledger says what it is:

| answer | meaning | what the index does |
|---|---|---|
| same | its lines are a version already read, at any path or on any machine | nothing to read; its state says `copy_of` |
| update | a version already read is the start of it | reads only the lines after that version |
| past changed | its first line is known, but no known version is a prefix | reads it all and keeps the old rows |
| never | its first line was never seen | reads it all |

So a moved folder or a mirror of another machine costs a hash, not a parse. Copies the ledger recognised are listed in
`_state.tsv` (`copy_of`) and `_ledger.tsv`, not in the rows' `sources`. The scan log (`relic-scan log`, `_scans.jsonl`)
counts `copies`, `resumed` and `past_changed`, and lists each file with its status.

## Build, test and change the code

```sh
just build       # target/release/relic-scan
just test        # cargo test --release: every test must pass (about a second once built)
just app         # the desktop app and .dmg for this Mac (needs the Tauri CLI: just tools)
just app-intel   # the same for Intel Macs, cross-built on Apple silicon
just linux       # .deb and .AppImage in Docker
just             # every recipe
```

Layout: `src/` is the crate (CLI, server, TUI); `src/page/` is the page, small JS and CSS files joined into one self-contained
page by `src/map.rs`; `desktop/` is the Tauri app; `site/` is the landing page; `docs/` has the design and research notes.

Rules this code keeps:
- **No file over 500 lines.** Split a long one into a folder of modules.
- **The page makes no network calls** and loads no library: everything is in `src/page/`, served from `127.0.0.1`.
- **Transcripts and vaults are read only.** relic-scan writes only inside its own output folder (`~/.relic-scan`).
- **Server writes need the `X-Relic` header**, and `/api/reveal` and `/api/note` open only paths the index holds.
- **After changing `src/page/`,** rebuild and run `relic-scan map` (or a run), then restart `relic-scan serve`: the page is built
  into `meta/map.html`.
- Every change keeps `just test` green; add a test with the behaviour you add.
- The landing page's words live in `site/public/scan/content.js`; run `just seo` after editing it.

## Licence

GNU AGPL v3 (see [LICENSE](LICENSE)). Source: https://github.com/Soul-Brews-Studio/relic4-scan · page:
https://relic.buildwithoracle.com/scan/
