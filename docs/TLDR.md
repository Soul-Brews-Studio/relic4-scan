# TL;DR

One page. Details are in the file named at the end of each line.

## What it is
`relic-scan` finds every AI-agent transcript on a machine and indexes it into one Parquet file per Bangkok month under `~/.relic-scan`. It reads
transcripts only, never changes them, and the page it serves is on `127.0.0.1` (README, AGENTS.md).

## Use it
```sh
relic-scan run          # index: reads only what is new (first run about a minute on a big Mac, then seconds)
relic-scan up           # run, then the page on http://127.0.0.1:8787
relic-scan tui          # the terminal UI; o opens the page on the selected row
relic-scan ops          # what the chain is doing; says "manual (no loop)" when nothing runs it by itself
duckdb -c "select count(*) from '~/.relic-scan/*.parquet'"
```
Query columns: AGENTS.md ("Query the index"). Since alpha.753 `meta/sessions.parquet` has `models`, `model_msgs`, `in_tokens`, `out_tokens`,
`cache_read_tokens`, `cache_write_tokens` (Claude Code and omp; Codex has the model but no tokens; months indexed before it read NULL).

## Nothing runs by itself (since 2026-10-26)
- A pass is run by hand: m5 `relic5 import`, mba and white `~/.local/bin/relic-chain`. Never `relic-chain` on m5. (docs/MANUAL.md)
- Only nightly S3 exports remain on mba and white (and relic5's offsite copy on white).
- A removed LaunchAgent plist does not unload the job: `launchctl list | grep relic`, then `launchctl bootout gui/$(id -u)/<label>`.

## Gotchas that cost a round each
- Replace a running binary with a copy next to it and `mv`; on macOS `cp` over it kills the process, and `rm` before a failed `cp` leaves none.
- Never put test or demo data under `~` or `/opt/Code`: the real chain indexes it and pushes it to the hub (the guide book's demo home did, on 2026-10-26).
- A fresh full index loses rows whose source files are gone; compare row counts before swapping it in (docs/TALK.md).
- Sealed months are read-only; any change to one re-seals it, and a sealed month that differs in S3 is refused without `--overwrite`.

## Old relic data
The old relic3 and relic4 folders, `.claude` archives and backups are in S3 on black (`relic-scan-sealed-months`, prefixes `relic-versions-m5/`,
`-m5-claude/`, `-mba/`, `-white/`, sha256 list `relic-versions-sha256.txt`). Restore commands and how to start the old tools: docs/OLD-RELIC-START.md.

## Who does what
`relic4-scan` writes the index, `relic-extract` cuts the parts, `relic5` is the hub (white, port 8795). They ask each other by pane id with
`fleet_send`, and file an issue on the repo that owns the fix. (docs/TALK.md)

## Release
Tag `v<version>` on `alpha` (pre-release), installers from `scripts/release/` (macOS `.dmg` here, Linux `.deb`/`.AppImage` in Docker on white), landing
page text in `site/public/scan/content.js` + `just seo`; the site deploys when `main` changes it. The Thai guide book (`book/`) stays on `alpha`.

## Read more
`README.md` · `AGENTS.md` (for agents) · `docs/TUTORIAL.md` (scan, extract, embed with outputs) · `docs/DESIGN.md` · `docs/MANUAL.md` · `docs/TALK.md`
