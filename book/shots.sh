#!/bin/zsh
# Page screenshots of the demo index (made by capture.sh in /tmp/relic-demo), taken by headless Chrome at 1280x860 @2x.
#   zsh shots.sh
B=${0:A:h}; IMG=$B/images; DEMO=/tmp/relic-demo
export PATH=$HOME/.local/bin:$PATH
pkill -f "relic-scan serve --out $DEMO" 2>/dev/null; lsof -ti tcp:8791 | xargs kill 2>/dev/null
(HOME=$DEMO PATH=/tmp/relic-nops:$PATH RELIC_OPS_HUB=off nohup relic-scan serve --out $DEMO/.relic-scan --port 8791 >/tmp/demo-serve.log 2>&1 &)
until curl -s -o /dev/null http://127.0.0.1:8791/; do :; done
C="/Applications/Google Chrome.app/Contents/MacOS/Google Chrome"
shot() { "$C" --headless=new --disable-gpu --hide-scrollbars --force-device-scale-factor=2 --window-size=${3:-1280,860} --virtual-time-budget=6000 --user-data-dir=/tmp/chr-shots --screenshot=$IMG/$2.png "http://127.0.0.1:8791/$1" >/dev/null 2>&1; echo "$2 $(sips -g pixelWidth -g pixelHeight $IMG/$2.png | awk '/pixel/{printf $2" "}')"; }
for p in "scan 10-page-scan" "scans 11-page-scans" "tree 12-page-tree" "map 13-page-map" "paths 14-page-paths" "disk 15-page-disk" "vaults 16-page-vaults" "banks 17-page-banks" "export 18-page-export" "ops 19-page-ops" "health 20-page-health" "settings 21-page-settings"; do shot $=p; done
