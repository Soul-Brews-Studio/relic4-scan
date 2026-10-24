#!/bin/zsh
set -e
export PATH=/opt/homebrew/bin:$HOME/.local/bin:$PATH
set -a; . ~/.config/relic-scan/s3.env; set +a
export AWS_DEFAULT_REGION=black-ssd-rustfs AWS_EC2_METADATA_DISABLED=true
E=http://black.follow-rankine.ts.net:9200; B=relic-scan-sealed-months; K=claude-merge-backup/2026-10-24/claude-merge.tar.zst
F=/tmp/claude-merge-backup.tar.zst
cd /tmp/claude-merge
if command -v zstd >/dev/null; then tar -cf - diverged replaced union-orig | zstd -3 -T0 -q -o $F; else K=${K%.zst}.gz; F=/tmp/claude-merge-backup.tar.gz; tar -czf $F diverged replaced union-orig; fi
SZ=$(stat -f %z $F); SHA=$(shasum -a 256 $F | cut -d' ' -f1)
echo "backup $F $SZ bytes sha256 $SHA"
aws --endpoint-url $E s3 cp $F s3://$B/$K --metadata sha256=$SHA --only-show-errors
R=$(aws --endpoint-url $E s3api head-object --bucket $B --key $K --query '[ContentLength,Metadata.sha256]' --output text)
echo "store: $R"
[ "$R" = "$SZ	$SHA" ] || [ "$(echo $R | cut -d' ' -f1)" = "$SZ" ] || { echo "BACKUP MISMATCH: nothing changed"; exit 1; }
echo "== union"; python3 -I /tmp/claude-union.py apply
echo "== recheck"; python3 -I /tmp/claude-union.py plan
echo DONE-BACKUP-OK $K
