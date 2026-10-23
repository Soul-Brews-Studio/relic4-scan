# AGENTS.md: ARRA Relic Scan, for AI agents

Instructions for an AI coding agent (Claude Code, Codex, Gemini CLI, Cursor, …) that is asked to install, build, run, query or
change ARRA Relic Scan. Humans: the [README](README.md) has the same facts at more length.

## What it is

`relic-scan` finds every AI-agent transcript on a machine (Claude Code, Codex, OMP, OMX, Gemini, Pi, thClaws: any `.claude`,
`.codex`, `.omp`, `.omx`, `.gemini`, `.pi` or `.thclaws` home, and backups or mirrors of them) and every oracle `ψ` vault. It
indexes the transcripts into one Parquet file per Bangkok month under `~/.relic-scan`, and serves a page to browse them on
`127.0.0.1` only. It never changes a transcript and makes no network calls. A desktop app (Tauri) wraps the same page.

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
| `YYYY-MM.parquet` | message, thinking block, tool call or result | `sid`, `engine`, `ts`, `ts_th` (Bangkok), `role` (user, assistant, thinking; toolResult in omp), `kind` (call, result), `name` (tool), `text`, `body`, `err` |
| `meta/sessions.parquet` | session | `sid`, `parent_sid`, `tier` (main, subagent, workflow-agent), `cat` (repo), `proj`, `first_ts`, `last_ts`, `last_ms`, `n_rows`, `user_text`, `assistant_text`, `thinking`, `tool_calls`, `tool_results`, `tool_errors` |
| `meta/files.parquet`, `meta/jsonl.parquet` | transcript file | path, home, size, verdict |
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
- **except omp transcripts** (`engine = 'omp'`), which mark a tool result as `role = 'toolResult'`. Count tool results as
  `kind = 'result' or role = 'toolResult'`;
- a handful of rows have neither (empty strings): leave them out or show them as "(none)".

```sh
# sessions per repository
duckdb -c "select cat as repo, count(*) as sessions, sum(n_rows) as n_rows from 'meta/sessions.parquet' group by 1 order by sessions desc limit 10"
# what the human asked about something, newest first
duckdb -c "select sid, ts_th, left(text, 120) as text from '*.parquet' where role = 'user' and text ilike '%parquet%' order by ts desc limit 20"
# the tools used most
duckdb -c "select name, count(*) as calls from '*.parquet' where kind = 'call' group by 1 order by calls desc limit 10"
# the newest session, read in order
duckdb -c "select role, left(text, 200) as text from '*.parquet' where sid = (select sid from 'meta/sessions.parquet' order by last_ms desc limit 1) and role in ('user','assistant') order by ts"
# ψ vaults with the most notes
jq -r '.vaults[] | select(.kind != "worktree") | [.notes, .name, .path] | @tsv' meta/vaults.json | head
```

Transcripts hold private work. Read what the human asked you to find; do not copy rows out of the machine.

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
