#!/bin/zsh
# Run every command of the relic-scan guide for real, on a demo home of made-up repositories, and keep what it printed:
#   captures/NN-slug.txt = "$ <shown command>" + output + "(exit N · T s)".
# The demo home is made by make-demo-home.py (deterministic). HOME is pointed at it, so "~" in every capture is the demo.
#   zsh capture.sh        (from the book dir; the page screenshots are made by shots.sh)
setopt pipefail
B=${0:A:h}; C=$B/captures; mkdir -p $C
export PATH=$HOME/.cargo/bin:$HOME/.local/bin:/opt/homebrew/bin:$PATH
export NO_COLOR=1 RELIC_LINKS=off
REAL_HOME=$HOME
DEMO=/tmp/relic-demo
rm -rf $DEMO; mkdir -p $DEMO
python3 -I $B/make-demo-home.py $DEMO >/dev/null
printf '{"roots":["~"],"exclude":[],"depth":8,"auto_check_min":0,"s3":{}}' > /dev/null
export HOME=$DEMO
mkdir -p $DEMO/.relic-scan && printf '{\n  "roots": ["~"],\n  "exclude": [],\n  "depth": 8,\n  "auto_check_min": 0,\n  "s3": {}\n}\n' > $DEMO/.relic-scan/_config.json
cd $DEMO

run() {   # run NN slug "shown command" "shell command"
  local nn=$1 slug=$2 disp=$3 cmd=$4 f=$C/$1-$2.txt t0=$(date +%s.%N)
  print -r -- "$ $disp" > $f
  (eval "$cmd") 2>&1 | sed "s|$DEMO|~|g" | tr '\r' '\n' | rg -v '^\s*(scan|parse|merge) [━─ ]*[0-9]+%' >> $f; local rc=${pipestatus[1]}
  printf '(exit %d · %.1f s)\n' $rc $(( $(date +%s.%N) - t0 )) >> $f
  printf '%s %-24s exit %d  %4d lines\n' $nn $slug $rc $(wc -l < $f)
}

run 01 help            "relic-scan --help"                    "relic-scan --help"
run 02 version         "relic-scan --version"                 "relic-scan --version"
run 03 demo-home       "ls -a ~"                              "ls -a ~"
run 04 dry-run         "relic-scan run --dry-run"             "relic-scan run --dry-run"
run 05 run-first       "relic-scan run"                       "relic-scan run"
run 06 out-folder      "ls ~/.relic-scan ~/.relic-scan/meta"  "ls ~/.relic-scan; echo; ls ~/.relic-scan/meta"
run 07 run-again       "relic-scan run"                       "relic-scan run"
# a transcript grows: the session answers once more
F=$(ls $DEMO/.claude/projects/-opt-Code-github-com-laris-co-starfish-api/*.jsonl | head -1)
SID=$(basename $F .jsonl)
python3 -I - "$F" "$SID" <<'PY'
import json, sys
p, sid = sys.argv[1], sys.argv[2]
c = dict(sessionId=sid, cwd="/opt/Code/github.com/laris-co/starfish-api", gitBranch="main", version="2.1.40")
with open(p, "a") as f:
    f.write(json.dumps(dict(type="user", timestamp="2026-10-26T00:10:00.000Z", message=dict(role="user", content="one more thing: add a changelog line"), **c)) + "\n")
    f.write(json.dumps(dict(type="assistant", timestamp="2026-10-26T00:10:09.000Z", message=dict(id="msg_late", role="assistant", model="claude-opus-5-5", usage=dict(input_tokens=5, output_tokens=120, cache_read_input_tokens=40000, cache_creation_input_tokens=0), content=[dict(type="text", text="Added: 'retry with backoff for order sync'.")]), **c)) + "\n")
PY
run 08 run-grown       "relic-scan run"                       "relic-scan run"
run 09 log             "relic-scan log -n 4"                  "relic-scan log -n 4"
run 10 seals           "relic-scan seals"                     "relic-scan seals"
run 11 seals-verify    "relic-scan seals --verify"            "relic-scan seals --verify"
cd ~/.relic-scan
run 12 duck-total      "duckdb -c \"select count(*) as all_rows from '*.parquet'\""  "duckdb -c \"select count(*) as all_rows from '*.parquet'\""
run 13 duck-what       "duckdb -c \"select coalesce(nullif(kind,''), nullif(role,''), '(none)') as what, count(*) as n from '*.parquet' group by 1 order by n desc\"" "duckdb -c \"select coalesce(nullif(kind,''), nullif(role,''), '(none)') as what, count(*) as n from '*.parquet' group by 1 order by n desc\""
run 14 duck-engine     "duckdb -c \"select engine, count(*) as n from '*.parquet' group by 1 order by n desc\"" "duckdb -c \"select engine, count(*) as n from '*.parquet' group by 1 order by n desc\""
run 15 duck-repos      "duckdb -c \"select cat as repo, count(*) as sessions, sum(n_rows) as n_rows from 'meta/sessions.parquet' group by 1 order by sessions desc limit 8\"" "duckdb -c \"select cat as repo, count(*) as sessions, sum(n_rows) as n_rows from 'meta/sessions.parquet' group by 1 order by sessions desc limit 8\""
run 16 duck-tokens     "duckdb -c \"select left(ts_th,7) as month, model, count(distinct msg_id) as calls, sum(out_tokens) as out_tokens from '*.parquet' where model is not null group by all order by month, calls desc limit 12\"" "duckdb -c \"select left(ts_th,7) as month, model, count(distinct msg_id) as calls, sum(out_tokens) as out_tokens from '*.parquet' where model is not null group by all order by month, calls desc limit 12\""
run 17 duck-session    "duckdb -c \"select sid, models, model_msgs, in_tokens, out_tokens, cache_read_tokens from 'meta/sessions.parquet' where out_tokens > 0 order by out_tokens desc limit 4\"" "duckdb -c \"select left(sid,8) as sid, models, model_msgs, in_tokens, out_tokens, cache_read_tokens from 'meta/sessions.parquet' where out_tokens > 0 order by out_tokens desc limit 4\""
run 18 duck-read       "duckdb -c \"select role, left(text, 70) as text from '*.parquet' where sid = '$SID' and role in ('user','assistant') order by ts limit 8\"" "duckdb -c \"select role, left(text, 70) as text from '*.parquet' where sid = '$SID' and role in ('user','assistant') order by ts limit 8\""
run 19 duck-redacted   "duckdb -c \"select left(text, 90) as text, red from '*.parquet' where red is not null\"" "duckdb -c \"select left(text, 90) as text, red from '*.parquet' where red is not null\""
mkdir -p /tmp/relic-nops && printf '#!/bin/sh\n' > /tmp/relic-nops/ps && chmod +x /tmp/relic-nops/ps
SYNC=$(duckdb -noheader -list -c "select string_agg(s || '|' || n, ' ' order by s) from (select s, count(*) as n from '$DEMO/.relic-scan/*.parquet' group by s)")
mkdir -p $DEMO/Library/Logs && cat > $DEMO/Library/Logs/relic-chain.log <<LOG
2026-10-26 06:50:15 chain start (m5)
run: scan 0.2s · new 0 · grown 3 · rewritten 0 · unchanged 145 · gone 0 · 0.00 GB to read
run: done in 31s
2026-10-26 06:50:48 relic-extract: 2 part(s) cut in 14s · 337 chunks · 12 messages cleaned out
2026-10-26 06:51:02 relic5 push m5 -> nat@white:/home/nat/relic5/parts/m5 rc=0 5s
2026-10-26 06:51:46 chain end
2026-10-26 06:51:57 SYNC OK  $SYNC
LOG
run 20 ops             "relic-scan ops --no-hub"              "PATH=/tmp/relic-nops:$PATH relic-scan ops --no-hub"
run 21 vaults          "jq -r '.vaults[] | [.notes, .name, .path] | @tsv' ~/.relic-scan/meta/vaults.json" "jq -r '.vaults[] | [.notes, .name, .path] | @tsv' ~/.relic-scan/meta/vaults.json"
run 22 months-json     "jq -c '.months[0]' ~/.relic-scan/meta/months.json" "jq -c '.months[0]' ~/.relic-scan/meta/months.json"
run 23 scans-jsonl     "tail -1 ~/.relic-scan/_scans.jsonl | jq 'del(.diff)'" "tail -1 ~/.relic-scan/_scans.jsonl | jq -c 'del(.diff)'"
# a range or one agent kind writes into its own folder, which has its own settings: the same roots, so it never walks outside the demo
for d in codex all@2026-10; do mkdir -p $DEMO/.relic-scan/$d && cp $DEMO/.relic-scan/_config.json $DEMO/.relic-scan/$d/_config.json; done
run 24 kind-one        "relic-scan run --kind codex"          "relic-scan run --kind codex"
run 25 month-one       "relic-scan run --month 2026-10"       "relic-scan run --month 2026-10"
export HOME=$REAL_HOME
