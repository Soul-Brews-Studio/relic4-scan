# How the agents talk: relic4-scan, relic5, relic-extract

Three repos make one pipeline, and each has its own agent in a herdr pane on m5. They keep to their own repo and ask each other
for changes. This is how.

| repo | owns | agent pane (m5, session `w11`/`w12`) |
|---|---|---|
| `relic4-scan` | the index: month parquet, `meta/sessions.parquet`, `meta/vaults`, the page, `relic-scan ops` | `w11:p1`, name `relic4-scan` |
| `relic5` | the hub on white (port 8795), `relic5-push`, the MCP tools (`relic_status`, `relic_usage`), seals | `w11:p25@neo` |
| `relic-extract` | text, thinking and tool parts cut from the index, the report scripts | `w12:p1` |

## Sending a message

- **Send:** `fleet_send` with `target` = the pane id (`w11:p25@neo`) and `text` = the whole message. Never the agent name: names
  prefix-match and have misdelivered. Multi-line text arrives as one prompt.
- **Read the answer:** `fleet_tail` on the same pane. An answer to you may also land as a pasted block in your own pane.
- **After a reboot** herdr forgets agent names. Claim yours again: `herdr agent rename <your pane> relic4-scan`. Pane ids can move
  too: look them up with `herdr agent list` before trusting an old one.
- **Slow work** (an index, an upload, a build) runs in a run pane, not in a blocking Bash call; the pane reports back with
  `herdr agent prompt <your name>` and closes itself. A failed `prompt` leaves the pane open on purpose: read it, then close it.

## What goes in a message

1. **What changed**, with the PR or issue number (`relic4-scan#75`), not a story.
2. **What the other side must do**, or "nothing".
3. **Numbers you measured**, with what they count (rows, sessions, tokens), and where they stop being true.
4. **What is not done**, so nobody assumes it.

A request that outlives the conversation goes in a GitHub issue on the repo that owns the fix, filed with the evidence. The agent
messages only point at it. Examples: `relic4-scan#73` (atomic meta writes), `#74` (models and tokens).

## Who fixes what

- A reader catches half a file, or a column is missing: the **writer's** repo fixes it (`relic4-scan` writes the index), and the
  reader keeps its guard until the fix is released everywhere.
- A number looks wrong on the hub: check the machine's own index first (`duckdb` on `~/.relic-scan/meta/sessions.parquet`). If the
  machine is right, the bug is in the fold on the hub. This is how the "18.7x July" jump was found: m5 had 0 empty sessions, the
  hub's tie-break preferred `jsonl-truth-import`.
- A change to the index shape (new column) needs: the code and a test, the column in `AGENTS.md`, a version bump, the CLI installed
  on m5, mba and white, then a message to `relic5` saying which months carry it. Old months read NULL until their files are read
  again; a fresh index drops rows whose source files are gone, so compare row counts before swapping it in.

## Keeping the three machines in sync

Each pass of the chain ends with a line `SYNC OK  text|N thinking|N tools|N` (m5, mba: `~/Library/Logs/relic-chain.log`; white:
`~/.local/state/relic-chain.log`). The hub's `/api/status` (`freshness.machines.*.sync`) must show the same three numbers. A mismatch
that stays for two passes is worth a message to `relic5`. A `relic-scan differs between machines` warning right after an install
clears when each machine has pushed with the new binary.

Replace a running binary with `rm -f` then `cp`, and only after the new one is built: on white a failed `cp` after the `rm`
left no `relic-scan` for six minutes.
