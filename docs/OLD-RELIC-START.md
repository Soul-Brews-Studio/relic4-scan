# Starting the old relic3 and relic4 again

Both are stopped since 2026-10-25 and replaced by relic5 (the hub on white) fed by `relic-scan` and `relic-extract`.
The repos, tools and service files are still in place. Their old data folders were copied to the S3 store on black, checked
(sha256 and file count) and then deleted from the machines: see "Restore the old data" below. Use this page only if you need
one of them back. Stop commands are at the end.

## relic3 (agent-relic-v3, MCP `relic_search`)

| where | what | start |
|---|---|---|
| m5 | MCP server, one process per Claude session | starts by itself when a session loads the `relic3` MCP: run `/mcp` and reconnect `relic3`. By hand: `bun /opt/Code/github.com/Soul-Brews-Studio/agent-relic-v3/src/cli.ts mcp` |
| m5 | hub indexer job (launchd, not loaded) | `launchctl bootstrap gui/$(id -u) ~/Library/LaunchAgents/studio.soulbrews.relic-indexer.plist` (runs `agent-relic-v3/hub/indexer.sh`, writes `~/.relic3-hub` and `~/.relic3-mirror`) |
| m5 | CLI | `~/.bun/bin/relic3` |
| white | index timer and API (systemd user units, already inactive since 2026-10-22) | `systemctl --user start relic3-index.timer relic3-serve.service` |
| white | `relic serve` on port 4319 and the UI on 5174 (pm2) | `pm2 start relic relic-ui` |

Data: white `~/.relic3-gens` (81 GB; now only in S3, restore below), `~/.relic` (198 MB), `~/.local/share/relic-ui` (277 MB). On m5 `~/.relic3-hub` and
`~/.relic3-mirror` were deleted earlier after a verified backup on black (`/home/beta/relic3-backup-20261024`), so the m5
indexer would have to rebuild them.

`relic serve` on white needs its token file `~/.relic/serve.token` (mode 0600); pm2 reads it through `~/bin/relic-serve.sh`.

## relic4 (agent-relic-v4, Python)

Repo: `/opt/Code/github.com/Soul-Brews-Studio/agent-relic-v4`. Two HTTP servers listened on m5, on 127.0.0.1 and the tailnet
address 100.111.101.21, ports 8791 and 8792, plus an MCP server.

```sh
cd /opt/Code/github.com/Soul-Brews-Studio/agent-relic-v4
uv run --extra server --extra vec relic4 serve --host 127.0.0.1,100.111.101.21 --port 8791
uv run --extra server --extra vec relic4 serve --host 127.0.0.1,100.111.101.21 --port 8792   # second instance
uv run -q --extra mcp --extra vec python -m relic4.mcp_server                                  # MCP
```

The two `serve` commands were read from the process list, but which index each port served was not recorded; look in the
repo's README and `~/.relic4*` before relying on it. `~/.local/bin/relic4` is a link into the uv tool install.

Data (m5): `~/.relic` (8.4 GB), `~/.relic-v2`, `~/.relic-v2.1`, `~/.relic-v2.2*`, `~/.relic-rs`. Deleted from m5 on 2026-10-25 after a
verified copy to S3; restore below.

## Restore the old data

Everything deleted on 2026-10-25 is in the bucket `relic-scan-sealed-months` on black (RustFS, `http://black.follow-rankine.ts.net:9200`,
region `black-ssd-rustfs`), one `.tar.zst` per folder, with its sha256 in `relic-versions-sha256.txt` at the top of the bucket.

| prefix | what |
|---|---|
| `relic-versions-m5/` | `relic`, `relic-rs`, `relic-v2`, `relic-v2.1`, `relic-v2.1-backup-2026-10-01`, `relic-v2.2`, `relic-v2.2-baseline-layout6-2026-10-01`, `relic-v2.2-month-prose`, `relic-extract-parity`, `relic-scan.before-fixes/meta/model-2026-10-23`, `misc` (`.relic-import`, `.relic-adhoc`, `.relic-extract-import`, `.relic-extract-relic3-codex`), `relic-mirrors`, `bins` (old `relic-chain`, `relic-scan`, `relic5-push` copies) |
| `relic-versions-mba/` | `index-copies` (the old `.relic-scan.bak-*` and `.before-*` folders), `bins` |
| `relic-versions-white/` | `relic3-gens` (`.relic3-gens`, 69 GiB), `index-copies`, `bins` |

Each archive holds the folder under its own name (for example `.relic-v2/…`), made with `tar -C ~`. Keys are in
`~/.config/relic-scan/s3.env` (mode 600; never print them).

```sh
set -a; . ~/.config/relic-scan/s3.env; set +a
export AWS_DEFAULT_REGION=black-ssd-rustfs
EP=http://black.follow-rankine.ts.net:9200; BK=s3://relic-scan-sealed-months
aws --endpoint-url $EP s3 ls $BK/relic-versions-m5/ --human-readable               # what is there
aws --endpoint-url $EP s3 cp $BK/relic-versions-sha256.txt - | rg relic-v2.tar.zst  # its sha256

# check, then restore one folder into the home directory (add a different -C to put it elsewhere)
aws --endpoint-url $EP s3 cp $BK/relic-versions-m5/relic-v2.tar.zst - | shasum -a 256   # must equal the line above
aws --endpoint-url $EP s3 cp $BK/relic-versions-m5/relic-v2.tar.zst - | zstd -d -q | tar -tf - | head   # look first
aws --endpoint-url $EP s3 cp $BK/relic-versions-m5/relic-v2.tar.zst - | zstd -d -q | tar -C ~ -xf -
```

Restore on the machine that owned the folder: mba and white have `aws` and the same `s3.env`, so run the same commands there
(`~/.relic3-gens` on white). Never extract over a folder that exists; move it aside first. A restored `.relic-scan.before-*`
folder is only an old index copy: the live index is `~/.relic-scan`.

## Stop again

```sh
# m5
pkill -f 'agent-relic-v3/src/cli.ts mcp'; pkill -f 'agent-relic-v4'; launchctl bootout gui/$(id -u)/studio.soulbrews.relic-indexer
# white
ssh nat@white.follow-rankine.ts.net 'pm2 stop relic relic-ui; systemctl --user stop relic3-index.timer relic3-serve.service'
```

An MCP server of a running Claude session comes back at the next `/mcp` reconnect or new session unless the `relic3`
entry is removed from that session's MCP config.
