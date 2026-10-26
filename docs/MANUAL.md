# Running the chain by hand (no loop)

Since 2026-10-26 nothing imports on its own. Each machine runs one pass when a person (or an agent) asks for it. This is simpler to
reason about: no pass starts while you are changing a binary, and the hub only moves when you say so.

| machine | what stays on its own | one pass by hand |
|---|---|---|
| m5 | nothing | **`relic5 import`** (the Rust `relic5` client, see relic5's `docs/IMPORT.md` and `docs/CLI.md`): scan, extract, push, index, embed, sync check, with a box at the end. Do NOT use `~/.local/bin/relic-chain` on m5: its extract after-hook (push + hub index) is commented out, so it pushes without indexing and the hub warns "parts newer than its manifest" |
| mba | nightly S3 export (`studio.soulbrews.relic-export`) | `~/.local/bin/relic-chain` (its own after-hook still pushes and indexes) |
| white | nightly S3 export (`relic-export.timer`) and the offsite copy (`relic5-offsite.timer`) | `~/.local/bin/relic-chain` |

The pass is the same everywhere: `relic-scan run`, then `relic-extract run` (its after-hook pushes to the hub), then the sync check. It
writes its stage lines to the chain log; `relic-scan ops` reads them and says "manual (no loop)" when no loop is installed.

## Check that it worked

```sh
relic-scan ops            # the last pass, its stages, and "manual (no loop)"
```

The pass ends with a line `SYNC OK  text|N thinking|N tools|N`; the hub's `/api/status` (`freshness.machines.<machine>.sync`) must show the
same three numbers. Until you run a pass, the hub's freshness for that machine only gets older: that is expected, not a fault.

## Turn the loop back on (one command per machine)

The units and plists were switched off, not deleted:

```sh
# white (systemd, user)
systemctl --user enable --now relic-chain.timer
# mba (launchd): the plist is still in ~/Library/LaunchAgents
launchctl bootstrap gui/$(id -u) ~/Library/LaunchAgents/studio.soulbrews.relic-chain.plist
```

On m5 the LaunchAgent was removed by relic5 (see its `docs/RETIRED.md`); restore it from there if you ever want it again.

## A loaded job outlives its file (m5, 2026-10-26)

Removing a LaunchAgent's plist does not stop the job: launchd keeps it loaded until it is booted out. On 2026-10-26 `relic-chain` (every 600 s),
`relic-push` (every 300 s) and `relic-indexer` (every 600 s) were still firing on m5 after their plists were gone, and one of those passes ran
beside a `relic5 import`. They were booted out by hand. After retiring a job, check that it is really gone:

```sh
launchctl list | grep -i relic          # nothing that has a run interval should be left
launchctl bootout gui/$(id -u)/studio.soulbrews.relic-chain
```
