#!/bin/zsh
# Linux .deb and .AppImage for the release, built in Docker on white (tailscale), copied back here
set -e
V=$(cat /tmp/relic-release-version); D=/opt/Code/github.com/Soul-Brews-Studio/relic4-scan; O=/tmp/relic-release-$V
mkdir -p $O; rm -rf /tmp/relic-linux
cd $D && docker --context white buildx build -f desktop/linux/Dockerfile --target out --output type=local,dest=/tmp/relic-linux . 2>&1 | tail -3
cp "$(ls /tmp/relic-linux/*.deb | head -1)" "$O/ARRA-Relic-Scan_${V}_linux-amd64.deb"
cp "$(ls /tmp/relic-linux/*.AppImage | head -1)" "$O/ARRA-Relic-Scan_${V}_linux-amd64.AppImage"
ls -la $O
