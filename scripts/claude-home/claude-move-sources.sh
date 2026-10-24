#!/bin/zsh
# move the merged-in sources (mirrors + archives) to /tmp/claude-sources. Same volume = rename, instant. Live homes are not touched.
set -e
D=/tmp/claude-sources; mkdir -p $D
mv_if() { [ -e "$1" ] && mv "$1" "$D/$2" && echo "moved $1 -> $D/$2"; }
mv_if ~/.relic-mirrors/mba relic-mirrors-mba
mv_if ~/.relic-mirrors/white-ampere-claude relic-mirrors-white-ampere-claude
mv_if ~/.claude-neo claude-neo
mv_if /opt/Code/_archive/oracle-world/neo-2026-06-10/.claude archive-neo-0610-dot-claude
mv_if /opt/Code/_archive/oracle-world/nat-2026-06-10/.claude archive-nat-0610-dot-claude
ls $D
