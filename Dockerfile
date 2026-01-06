ARG CUDA_VER=12.8.0
ARG UBUNTU_VER=24.04
ARG GO_VERSION=1.24.1

FROM nvidia/cuda:${CUDA_VER}-devel-ubuntu${UBUNTU_VER} AS danser-builder
ARG GO_VERSION
SHELL ["/bin/bash", "-o", "pipefail", "-c"]

ENV PATH=/usr/local/go/bin:$PATH

RUN apt-get update && apt-get install -y --no-install-recommends \
  build-essential git git-lfs ca-certificates curl xz-utils xorg-dev libgl1-mesa-dev libgtk-3-dev \
 && rm -rf /var/lib/apt/lists/*

RUN set -eux; \
    url="https://go.dev/dl/go${GO_VERSION}.linux-amd64.tar.gz"; \
    for i in 1 2 3; do \
      curl -fsSL -o /tmp/go.tgz "$url" && break || sleep 3; \
    done; \
    rm -rf /usr/local/go; \
    tar -C /usr/local -xzf /tmp/go.tgz; \
    rm /tmp/go.tgz

RUN git clone --depth 1 --branch dev https://github.com/Wieku/danser-go.git /src/danser
WORKDIR /src/danser

RUN set -eux; \
    git lfs install --system; \
    git lfs pull; \
    export GOOS=linux; \
    export GOARCH=amd64; \
    export CGO_ENABLED=1; \
    export CC=gcc; \
    export CXX=g++; \
    BUILD_DIR=/tmp/danser-build-linux; \
    mkdir -p "$BUILD_DIR"; \
    go run tools/assets/assets.go ./ "$BUILD_DIR/"; \
    go build -trimpath -ldflags "-s -w -X 'github.com/wieku/danser-go/build.VERSION=dev' -X 'github.com/wieku/danser-go/build.Stream=Release'" \
      -buildmode=c-shared -o "$BUILD_DIR/danser-core.so" \
      -tags "exclude_cimgui_glfw exclude_cimgui_sdli"; \
    mv "$BUILD_DIR/danser-core.so" "$BUILD_DIR/libdanser-core.so"; \
    cp libbass.so libbass_fx.so libbassmix.so libyuv.so libSDL3.so "$BUILD_DIR/"; \
    gcc -no-pie -O3 -o "$BUILD_DIR/danser-cli" -I. cmain/main_danser.c -I"$BUILD_DIR/" -Wl,-rpath,'$ORIGIN' -L"$BUILD_DIR/" -ldanser-core; \
    gcc -no-pie -O3 -D LAUNCHER -o "$BUILD_DIR/danser" -I. cmain/main_danser.c -I"$BUILD_DIR/" -Wl,-rpath,'$ORIGIN' -L"$BUILD_DIR/" -ldanser-core; \
    strip "$BUILD_DIR/danser" "$BUILD_DIR/danser-cli" 2>/dev/null || true; \
    rm -f "$BUILD_DIR/danser-core.h"; \
    mkdir -p "$BUILD_DIR/ffmpeg/bin"; \
    curl -fsSL -o /tmp/ffmpeg.tar.xz "https://github.com/BtbN/FFmpeg-Builds/releases/latest/download/ffmpeg-master-latest-linux64-gpl.tar.xz" \
      || curl -fsSL -o /tmp/ffmpeg.tar.xz "https://github.com/BtbN/FFmpeg-Builds/releases/latest/download/ffmpeg-master-latest-linux64-gpl-shared.tar.xz"; \
    tar -xf /tmp/ffmpeg.tar.xz -C /tmp; \
    FF_DIR="$(find /tmp -maxdepth 1 -type d -name 'ffmpeg-*' | head -n 1)"; \
    cp "$FF_DIR/bin/ffmpeg" "$BUILD_DIR/ffmpeg/bin/"; \
    cp "$FF_DIR/bin/ffprobe" "$BUILD_DIR/ffmpeg/bin/" || true; \
    cp "$FF_DIR/bin/ffplay"  "$BUILD_DIR/ffmpeg/bin/" || true; \
    if [ -d "$FF_DIR/lib" ]; then mkdir -p "$BUILD_DIR/ffmpeg/lib" && cp -a "$FF_DIR/lib/." "$BUILD_DIR/ffmpeg/lib/"; fi; \
    rm -rf /tmp/ffmpeg* /tmp/ffmpeg.tar.xz; \
    chmod 755 "$BUILD_DIR/danser" "$BUILD_DIR/danser-cli" "$BUILD_DIR/ffmpeg/bin/ffmpeg" || true; \
    chmod 755 "$BUILD_DIR/ffmpeg/bin/ffprobe" "$BUILD_DIR/ffmpeg/bin/ffplay" 2>/dev/null || true; \
    mkdir -p /out/danser; \
    cp -a "$BUILD_DIR/." /out/danser/; \
    mkdir -p /out/danser/settings /out/danser/Songs /out/danser/Skins /out/danser/Replays /out/danser/videos; \
    rm -rf "$BUILD_DIR"

FROM rustlang/rust:nightly-slim AS oscbot-builder

RUN apt-get update && apt-get install -y \
    pkg-config libssl-dev ca-certificates build-essential \
 && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
RUN mkdir -p src \
 && printf 'fn main() {}\n' > src/main.rs \
 && cargo build --release \
 && rm -rf src

COPY src ./src
RUN find src -type f -exec touch {} + \
 && rm -f target/release/oscbot \
 && cargo build --release \
 && mkdir -p /out \
 && cp target/release/oscbot /out/oscbot

FROM nvidia/cuda:${CUDA_VER}-runtime-ubuntu${UBUNTU_VER} AS final
SHELL ["/bin/bash", "-o", "pipefail", "-c"]

RUN apt-get update && apt-get install -y --no-install-recommends \
  ca-certificates \
  libglvnd0 libegl1 libgles2 libgl1 libgtk-3-0 libglib2.0-0 \
 && rm -rf /var/lib/apt/lists/*

RUN install -d -m 755 /etc/glvnd/egl_vendor.d \
 && cat >/etc/glvnd/egl_vendor.d/10_nvidia.json <<'EOF'
{
  "file_format_version": "1.0.0",
  "ICD": { "library_path": "libEGL_nvidia.so.0" }
}
EOF

RUN groupadd -g 1000 appuser 2>/dev/null || true \
 && id -u 1000 >/dev/null 2>&1 || useradd -u 1000 -g 1000 -m -s /bin/bash appuser \
 && mkdir -p /app

COPY --from=danser-builder --chown=1000:1000 --chmod=755 /out/danser /app/danser

ENV LD_LIBRARY_PATH=/app/danser:/app/danser/ffmpeg:/app/danser/ffmpeg/lib:/usr/local/nvidia/lib:/usr/local/nvidia/lib64:${LD_LIBRARY_PATH}

RUN ldconfig \
 && mkdir -p /app/oscbot

WORKDIR /app/oscbot

COPY --chown=1000:1000 --chmod=755 --from=oscbot-builder /out/oscbot /app/oscbot/oscbot
COPY --chown=1000:1000 default-danser.json   /app/danser/settings/default.json
COPY --chown=1000:1000 src/generate/data     /app/oscbot/src/generate/data

USER 1000:1000
ENTRYPOINT ["/app/oscbot/oscbot"]