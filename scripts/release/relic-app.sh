#!/bin/zsh
# build ARRA Relic Scan.app and its drag-to-Applications installer (.dmg); install the app into ~/Applications
# (an older copy, either name, moves to /tmp) and show the installer in Finder
set -e
cd /opt/Code/github.com/Soul-Brews-Studio/relic4-scan/desktop && ~/.cargo/bin/cargo tauri build --bundles app,dmg
pkill -f relic-scan-desktop || true
for old in "Relic Scan.app" "ARRA Relic Scan.app"; do
  [ -d ~/Applications/"$old" ] && mv ~/Applications/"$old" /tmp/"$old.prev-$(date +%s)"
done
cp -R target/release/bundle/macos/"ARRA Relic Scan.app" ~/Applications/
DMG=$(ls -t target/release/bundle/dmg/*.dmg | head -1)
echo "app: $(du -sh ~/Applications/'ARRA Relic Scan.app' | cut -f1) · dmg: $(du -sh "$DMG" | cut -f1) $DMG $(date +%T)"
