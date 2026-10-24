#!/bin/zsh
set -e
export PATH=/opt/homebrew/bin:$HOME/.local/bin:$PATH
set -a; . ~/.config/relic-scan/s3.env; set +a
export AWS_DEFAULT_REGION=black-ssd-rustfs AWS_EC2_METADATA_DISABLED=true
E=http://black.follow-rankine.ts.net:9200; B=relic-scan-sealed-months; K=claude-merge-backup/2026-10-24/mba-extra-homes.tar.zst
F=/tmp/mba-extra-homes.tar.zst
cd /tmp/claude-merge
tar -cf - from-mba | zstd -3 -T0 -q -f -o $F
SZ=$(stat -f %z $F); SHA=$(shasum -a 256 $F | cut -d' ' -f1)
echo "backup $F $SZ sha256 $SHA"
aws --endpoint-url $E s3 cp $F s3://$B/$K --metadata sha256=$SHA --only-show-errors
R=$(aws --endpoint-url $E s3api head-object --bucket $B --key $K --query '[ContentLength,Metadata.sha256]' --output text)
echo "store: $R"
[ "$(echo $R | cut -f1)" = "$SZ" ] && [ "$(echo $R | cut -f2)" = "$SHA" ] && echo BACKUP-OK || { echo BACKUP-MISMATCH; exit 1; }
