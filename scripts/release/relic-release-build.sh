#!/bin/zsh
# release files for v$(cat /tmp/relic-release-version): the CLI (installed here), the Apple-silicon and Intel .dmg
set -e
V=$(cat /tmp/relic-release-version); D=/opt/Code/github.com/Soul-Brews-Studio/relic4-scan; O=/tmp/relic-release-$V
export PATH=$HOME/.cargo/bin:$PATH; mkdir -p $O
cd $D && cargo build --release -q && rm -f ~/.local/bin/relic-scan && cp target/release/relic-scan ~/.local/bin/relic-scan && relic-scan --version
zsh /tmp/relic-app.sh | tail -1
cp "$(ls -t $D/desktop/target/release/bundle/dmg/*.dmg | head -1)" "$O/ARRA-Relic-Scan_${V}_macos-arm64.dmg"
cd $D/desktop && cargo tauri build --target x86_64-apple-darwin --bundles app,dmg 2>&1 | tail -1
cp "$(ls -t target/x86_64-apple-darwin/release/bundle/dmg/*.dmg | head -1)" "$O/ARRA-Relic-Scan_${V}_macos-x64.dmg"
file "target/x86_64-apple-darwin/release/bundle/macos/ARRA Relic Scan.app/Contents/MacOS/relic-scan-desktop" | sed 's/.*: //'
ls -la $O
