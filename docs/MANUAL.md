# Running the chain by hand (no loop)

Since 2026-10-26 nothing imports on its own. Each machine runs one pass when a person (or an agent) asks for it. This is simpler to
reason about: no pass starts while you are changing a binary, and the hub only moves when you say so.

| machine | what stays on its own | one pass by hand |
|---|---|---|
| m5 | nothing | `relic5-import` from the relic5 repo (`bin/relic5-import`, see its `docs/IMPORT.md`): scan, extract, push, embed, sync check, with a box at the end |
| mba | nightly S3 export (`studio.soulbrews.relic-export`) | `~/.local/bin/relic-chain` |
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
