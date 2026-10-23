#!/bin/sh
# Install relic-scan into ~/.local/bin (on PATH in a login shell; ~/.cargo/bin often is not).
#
#   curl -fsSL https://raw.githubusercontent.com/Soul-Brews-Studio/relic4-scan/main/install.sh | sh
#   REF=some-branch sh install.sh      # another branch (default: main)
#
# cargo fetches the repository through the git CLI, so a git credential helper (gh, a keychain) is used when one is set.
set -eu

REPO=https://github.com/Soul-Brews-Studio/relic4-scan
CARGO=$(command -v cargo || true)
[ -n "$CARGO" ] || CARGO="$HOME/.cargo/bin/cargo"
if [ ! -x "$CARGO" ]; then
  echo "relic-scan: needs Rust. Install it with: curl https://sh.rustup.rs -sSf | sh" >&2
  exit 1
fi

echo "relic-scan: building from $REPO${REF:+ ($REF)} with $CARGO (about 1 min the first time)"
CARGO_NET_GIT_FETCH_WITH_CLI=true "$CARGO" install --locked --force --root "$HOME/.local" \
  --git "$REPO" ${REF:+--branch "$REF"} relic-scan

BIN="$HOME/.local/bin/relic-scan"
echo "relic-scan: installed $BIN ($("$BIN" --version))"
case ":$PATH:" in
  *":$HOME/.local/bin:"*) ;;
  *) echo "relic-scan: add ~/.local/bin to PATH, e.g. echo 'export PATH=\$HOME/.local/bin:\$PATH' >> ~/.zshrc" ;;
esac
echo "relic-scan: next: relic-scan up --kind claude   (then http://127.0.0.1:8787/map · /tree · /paths), or relic-scan tui --kind claude"
