#!/bin/zsh
# More captures on the SAME demo index capture.sh left in /tmp/relic-demo (run capture.sh first; this adds files, changes none):
#   zsh capture2.sh
setopt pipefail
B=${0:A:h}; C=$B/captures
export PATH=$HOME/.cargo/bin:$HOME/.local/bin:/opt/homebrew/bin:$PATH NO_COLOR=1 RELIC_LINKS=off
DEMO=/tmp/relic-demo; export HOME=$DEMO; cd $DEMO
run() {   # run NN slug "shown command" "shell command"
  local nn=$1 slug=$2 disp=$3 cmd=$4 f=$C/$1-$2.txt t0=$(date +%s.%N)
  print -r -- "$ $disp" > $f
  (eval "$cmd") 2>&1 | sed "s|$DEMO|~|g" | tr '\r' '\n' >> $f; local rc=${pipestatus[1]}
  printf '(exit %d · %.1f s)\n' $rc $(( $(date +%s.%N) - t0 )) >> $f
  printf '%s %-24s exit %d  %4d lines\n' $nn $slug $rc $(wc -l < $f)
}
run 40 help-run        "relic-scan run --help"                "relic-scan run --help | cut -c1-110"
run 41 help-export     "relic-scan export --help"             "relic-scan export --help | cut -c1-110"
run 42 help-seals      "relic-scan seals --help"              "relic-scan seals --help"
run 43 state-tsv       "head -3 ~/.relic-scan/_state.tsv | cut -f1,4,6,7" "head -3 ~/.relic-scan/_state.tsv | cut -f1,4,6,7"
run 44 ledger-tsv      "head -4 ~/.relic-scan/_ledger.tsv | cut -f2,3,5,9" "head -4 ~/.relic-scan/_ledger.tsv | cut -f2,3,5,9"
run 45 ls-months       "ls -l ~/.relic-scan/*.parquet"        "ls -l ~/.relic-scan/*.parquet | awk '{print \$1, \$5, \$9}'"
run 46 describe-month  "duckdb -c \"describe select * from '2026-10.parquet'\"" "cd ~/.relic-scan && duckdb -c \"select column_name, column_type from (describe select * from '2026-10.parquet')\""
run 47 describe-sess   "duckdb -c \"describe select * from 'meta/sessions.parquet'\"" "cd ~/.relic-scan && duckdb -c \"select column_name, column_type from (describe select * from 'meta/sessions.parquet')\""
run 48 ls-meta         "ls -l ~/.relic-scan/meta"             "ls -l ~/.relic-scan/meta | awk 'NR>1{print \$1, \$5, \$9}'"
run 49 help-ops        "relic-scan ops --help"                "relic-scan ops --help | cut -c1-110"
run 50 help-tui        "relic-scan tui --help"                "relic-scan tui --help | cut -c1-110"
