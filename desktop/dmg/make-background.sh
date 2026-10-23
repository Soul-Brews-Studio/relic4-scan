#!/bin/zsh
# The installer window's background (660x400, and 2x for Retina, joined into one TIFF that Finder picks from).
# Light on purpose: Finder draws the icon labels in black on a custom background. The icons sit at (180,190) and
# (480,190) (tauri.conf.json > bundle.macOS.dmg); the arrow runs between them.
set -e
cd "$(dirname "$0")"
F=/System/Library/Fonts/SFNS.ttf
for S in 1 2; do
  magick -size $((660*S))x$((400*S)) xc:'#f3f4f7' \
    -fill '#9aa3b5' -stroke '#9aa3b5' -strokewidth $((3*S)) -draw "line $((262*S)),$((190*S)) $((392*S)),$((190*S))" \
    -stroke none -draw "polygon $((404*S)),$((190*S)) $((388*S)),$((180*S)) $((388*S)),$((200*S))" \
    -font $F -fill '#5d667a' -pointsize $((15*S)) -gravity north -annotate +0+$((300*S)) 'Drag ARRA Relic Scan into Applications' \
    -fill '#8a93a8' -pointsize $((12*S)) -annotate +0+$((326*S)) 'every AI-agent transcript on this Mac, local only' \
    -gravity northwest -fill '#1c2233' -pointsize $((13*S)) -annotate +$((24*S))+$((22*S)) 'ARRA RELIC' \
    -fill '#8a93a8' -annotate +$((112*S))+$((25*S)) 'scan' \
    /tmp/relic-dmg-bg-$S.png
done
tiffutil -cathidpicheck /tmp/relic-dmg-bg-1.png /tmp/relic-dmg-bg-2.png -out background.tiff
rm -f /tmp/relic-dmg-bg-1.png /tmp/relic-dmg-bg-2.png
echo "dmg/background.tiff"
