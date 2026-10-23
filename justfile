# Build ARRA Relic Scan from source. `just` lists the recipes; `just build` is the one most people want.
#
#   just build        the relic-scan command          -> target/release/relic-scan
#   just install      build it and put it in ~/.local/bin
#   just app          the desktop app and its .dmg     -> desktop/target/release/bundle/{macos,dmg}
#   just test         every test
#
# Needs Rust (https://rustup.rs). The desktop app also needs the Tauri CLI: `just tools` installs it.

export PATH := env_var('HOME') + "/.cargo/bin:" + env_var('PATH')

# list the recipes
default:
    @just --list --unsorted

# the relic-scan command (CLI, server, TUI), optimised
build:
    cargo build --release --locked
    @echo "built: target/release/relic-scan ($(target/release/relic-scan --version))"

# build, then install relic-scan into ~/.local/bin
install: build
    mkdir -p ~/.local/bin
    cp target/release/relic-scan ~/.local/bin/relic-scan
    @echo "installed: ~/.local/bin/relic-scan ($(~/.local/bin/relic-scan --version))"

# every test (about a second once built)
test:
    cargo test --release --locked

# index this machine's transcripts, then serve the page on http://127.0.0.1:8787
up *ARGS: build
    target/release/relic-scan up {{ARGS}}

# the Tauri CLI the desktop app is built with (once)
tools:
    command -v cargo-tauri >/dev/null || cargo install tauri-cli --version '^2' --locked

# the desktop app and its drag-to-Applications .dmg, for this Mac
app: tools
    cd desktop && cargo tauri build --bundles app,dmg
    @ls -1 desktop/target/release/bundle/dmg/*.dmg

# the desktop app for Intel Macs, cross-built on Apple silicon
app-intel: tools
    rustup target add x86_64-apple-darwin
    cd desktop && cargo tauri build --target x86_64-apple-darwin --bundles app,dmg
    @ls -1 desktop/target/x86_64-apple-darwin/release/bundle/dmg/*.dmg

# Linux .deb and .AppImage, built in Docker (any host; `just linux builder` uses the docker context named builder)
linux CONTEXT="default":
    docker --context {{CONTEXT}} buildx build -f desktop/linux/Dockerfile --target out --output type=local,dest=target/linux .
    @ls -1 target/linux

# the landing page on http://127.0.0.1:8795/scan/ (static files, no build step)
site:
    python3 -m http.server 8795 --bind 127.0.0.1 --directory site/public

# refresh the landing page's JSON-LD, no-JavaScript page and llms.txt from site/public/scan/content.js
seo:
    node site/seo.mjs
