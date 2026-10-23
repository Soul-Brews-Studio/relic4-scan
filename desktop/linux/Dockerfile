# Linux packages of the desktop app (.deb and .AppImage), built in a container so any Docker host can make them:
#   docker --context white buildx build -f desktop/linux/Dockerfile --target out --output type=local,dest=/tmp/relic-linux .
# Ubuntu 22.04 on purpose: its older glibc lets the AppImage run on newer distributions too.
FROM ubuntu:22.04 AS build
ENV DEBIAN_FRONTEND=noninteractive APPIMAGE_EXTRACT_AND_RUN=1
RUN apt-get update && apt-get install -y --no-install-recommends \
      build-essential curl wget file ca-certificates pkg-config patchelf xdg-utils \
      libwebkit2gtk-4.1-dev libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev \
    && rm -rf /var/lib/apt/lists/*
RUN curl -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal
ENV PATH=/root/.cargo/bin:$PATH
# the Tauri CLI in its own layer, so a source change does not rebuild it
RUN cargo install tauri-cli --version '^2' --locked
WORKDIR /src
COPY . .
RUN cd desktop && cargo tauri build --bundles deb,appimage

FROM scratch AS out
COPY --from=build /src/desktop/target/release/bundle/deb/*.deb /src/desktop/target/release/bundle/appimage/*.AppImage /
