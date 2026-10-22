# relic-scan vs Relic Scanner vs Relic Studio

Relic Scanner and Relic Studio are the two native apps in [agent-relic-v3](https://github.com/Soul-Brews-Studio/agent-relic-v3) (`scanner/` and `studio/`). Read on 2026-10-22, with the app screenshots Nat shared.

## What each one is

| | Relic Scanner | Relic Studio | relic-scan |
|---|---|---|---|
| **What it is** | Dashboard over the `relic3` CLI | Second dashboard over the `relic3` CLI ("Orbit" design) | The indexer itself, plus its own views |
| **Who does the work** | `relic3` (TypeScript/Bun), run as a child process | `relic3`, through Scanner's `RelicScannerCore.CLIProcess` | This binary (Rust), in its own process |
| **Data** | `~/.relic3`: weekly event/tool shards, LanceDB vectors | Same as Scanner | `~/.relic-scan`: one Parquet file per Bangkok month (relic4 truth schema) and `meta/*.parquet` |
| **UI** | SwiftUI, macOS 14+ | SwiftUI, macOS 14+ | Web page on 127.0.0.1, a terminal UI, and a Tauri desktop window |
| **Platforms** | macOS | macOS | macOS and Linux for the CLI, web page and TUI. Tauri desktop built and run on macOS; Linux should work with WebKitGTK but is untested. |
| **Size** | ≈3,100 lines of Swift | ≈2,900 lines of Swift (it reuses Scanner's core) | ≈3,400 lines of Rust core, plus 700 lines of TUI and 800 lines of page |

## Screens

| | Relic Scanner / Relic Studio | relic-scan |
|---|---|---|
| **Scan** | Smart Scan: files discovered vs searchable; **Index now** runs `relic3 index --since 1d` | `run`, the index button on the page or in the desktop app, and `relic-scan up`. Every run is incremental by byte offset. |
| **Embedding** | Coverage ring, model admission, batch size, **Run batch** (`relic3 embed`) | **None.** Embedding lives in agent-relic-v4, which reads these month files. |
| **Banks & Repos** | Cards per bank (sessions, events, shards); a repository index | Map sectors and tree levels per repo; home → repo → session; the TUI repos view; `/paths` per folder |
| **Health** | `status` totals and warnings | `/paths` statuses (indexed / skipped / vault / loose) and verdicts; the summary card after each run |
| **Jobs** | A tail of `$RELIC_HOME/logs/jobs.jsonl` | Live `/api/status` (phase, bytes) and the parse progress bar |
| **Sessions** | Not shown one by one | Star map, tree and TUI list. Each session has rows by type, satellites (subagents and workflow agents), and a copy of `claude --resume`. |
| **Rows by type** | Studio "Layers": prose, tools; thinking is "not collected" | User text, assistant text, **thinking**, tool calls, tool results, tool errors, per session and per repo |
| **Every `.jsonl` on disk** | No | Yes: `meta/jsonl.parquet` and `/paths`, including files outside the homes and in ψ vaults |

## Same banks, two counters
These are the Studio Banks cards (relic3) next to relic-scan's Claude run on the same machine (m5, 2026-10-22). relic-scan counts a session as one file name, with copies folded, and text rows per copy.

| bank | relic3 sessions · events | relic-scan sessions · text rows |
|---|---|---|
| `projects-1sep-tue2026` | 24,205 · 401,593 | 24,189 · 413,494 |
| `projects-archive` | 7,049 · 354,315 | 6,398 (7,052 files) · 353,748 |

The other banks are drawn differently by each tool. relic3 has one `projects` bank per home. relic-scan groups `~/.claude/projects` together with the mirrors' `.claude/projects`, so those rows are not comparable one to one.

## Speed (m5)
| | relic3 (what the apps call) | relic-scan |
|---|---|---|
| Incremental index | 43 s with 37 changed files (relic v3, measured in this repo's history) | ≈10 s, most of it the home walk |
| Cold, every Claude transcript | — | 44.8 s for 83,673 files, 67.6 GB and 5.0 M rows |

## What to take from each
- **From the Swift apps into relic-scan:**
  - **Writes only on a click:** the index button is the only writer on the page and in the desktop app, and the server refuses a second one with a 409.
  - **Never invent a number:** relic-scan's status reports phase, bytes and the last error as measured.
  - **The look:** the Studio-style dark look with neon accents is now in the page's theme and in the TUI.
- **From relic-scan back into the apps:** a session-level view, thinking rows, the `.jsonl` accounting, and a cross-platform shell (Tauri) instead of macOS-only SwiftUI.
- **Still only in the apps:** embedding control. A future `relic-scan` view could show agent-relic-v4's vector coverage next to the months, without running the embedder itself.
