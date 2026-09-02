#!/usr/bin/env bash
# VoxelCraft — all-in-one build script (§ "compile for all arch systems")
#
# Builds EVERY target from this one script:
#   1. host native binary      (cargo, full audio)
#   2. wasm32 browser bundle   (wasm-bindgen --target web)
#   3. optional cross targets  (--cross: linux-arm64, windows, macos —
#      requires the targets installed: rustup target add <t>)
#
# Everything lands in dist/ as ready-to-run folders:
#   dist/voxelcraft-<version>-<name>/
#     voxelcraft[.exe]  builtin-pack/  README.md  (+ web: play.html, js+wasm)
#
# Usage:
#   ./scripts/build-all.sh              # host + wasm
#   ./scripts/build-all.sh --cross      # host + wasm + all cross targets
#   ./scripts/build-all.sh --wasm-only  # browser bundle only
#   NO_AUDIO=1 ./scripts/build-all.sh   # host without rodio/cpal (no ALSA)
set -euo pipefail
cd "$(dirname "$0")/.."   # voxelcraft/

VERSION="$(git rev-parse --short HEAD 2>/dev/null || echo dev)"
DIST="../dist"
mkdir -p "$DIST"

log() { printf '\033[1;34m[build-all]\033[0m %s\n' "$*"; }

# ---- 1. host native binary --------------------------------------------------
build_host() {
  log "building HOST native binary (release)"
  FEATURES=""
  if [ "${NO_AUDIO:-0}" = "1" ] || { [ "$(uname -s)" = "Linux" ] && ! pkg-config --exists alsa 2>/dev/null && [ ! -f /usr/include/alsa/asoundlib.h ]; }; then
    log "ALSA headers not found -> building with --no-default-features (engine-only, no audio)"
    FEATURES="--no-default-features"
  fi
  cargo build --release $FEATURES --bin voxelcraft
  DIR="$DIST/voxelcraft-$VERSION-native"
  rm -rf "$DIR"; mkdir -p "$DIR"
  EXT=""
  if [ -f "target/release/voxelcraft.exe" ]; then EXT=".exe"; fi
  cp "target/release/voxelcraft$EXT" "$DIR/"
  cp -r builtin-pack "$DIR/"
  cp BUILD.md "$DIR/README.md"
  log "packaged $DIR"
}

# ---- 2. wasm32 browser bundle ----------------------------------------------
build_wasm() {
  log "building WASM32 browser bundle (release)"
  command -v wasm-bindgen >/dev/null 2>&1 || {
    log "installing wasm-bindgen-cli (pinned 0.2.127)"
    cargo install wasm-bindgen-cli --version 0.2.127 --locked
  }
  cargo build --release --no-default-features --target wasm32-unknown-unknown --lib
  rm -rf wasm-out
  wasm-bindgen --target web --out-dir ./wasm-out \
    target/wasm32-unknown-unknown/release/voxelcraft.wasm
  DIR="$DIST/voxelcraft-$VERSION-web"
  rm -rf "$DIR"; mkdir -p "$DIR"
  cp wasm-out/voxelcraft.js           "$DIR/"
  cp wasm-out/voxelcraft_bg.wasm      "$DIR/"
  cp wasm-out/voxelcraft.d.ts         "$DIR/"
  cp wasm-out/voxelcraft_bg.wasm.d.ts "$DIR/"
  cp -r builtin-pack                  "$DIR/"
  cp play.html                        "$DIR/"
  cp BUILD.md                         "$DIR/README.md"
  log "packaged $DIR (open play.html in a WebGPU browser)"
}

# ---- 3. cross targets --------------------------------------------------------
build_cross() {
  local target="$1" name="$2" no_audio="$3"
  if ! rustup target list --installed 2>/dev/null | grep -q "^$target$"; then
    log "target $target not installed - run: rustup target add $target"
    return 1
  fi
  log "building CROSS target $name ($target)"
  local features=""
  if [ "$no_audio" = "1" ]; then features="--no-default-features"; fi
  if ! cargo build --release --target "$target" $features --bin voxelcraft 2>/dev/null; then
    log "plain cargo cross-build failed; trying 'cross' (docker)"
    command -v cross >/dev/null 2>&1 || cargo install cross --locked
    cross build --release --target "$target" $features --bin voxelcraft
  fi
  DIR="$DIST/voxelcraft-$VERSION-$name"
  rm -rf "$DIR"; mkdir -p "$DIR"
  EXT=""
  case "$target" in *windows*) EXT=".exe";; esac
  cp "target/$target/release/voxelcraft$EXT" "$DIR/"
  cp -r builtin-pack "$DIR/"
  cp BUILD.md "$DIR/README.md"
  if [ "$no_audio" = "1" ]; then
    echo "NOTE: audio disabled in this cross build (no audio sysroot)." >> "$DIR/README.md"
  fi
  log "packaged $DIR"
}

# ---- dispatch ----------------------------------------------------------------
CROSS=0; WASM_ONLY=0
for arg in "$@"; do
  case "$arg" in
    --cross)     CROSS=1 ;;
    --wasm-only) WASM_ONLY=1 ;;
    *) echo "unknown flag: $arg (use --cross / --wasm-only)"; exit 2 ;;
  esac
done

if [ "$WASM_ONLY" = "0" ]; then build_host; fi
build_wasm
if [ "$CROSS" = "1" ]; then
  build_cross aarch64-unknown-linux-gnu linux-arm64 1 || true
  build_cross x86_64-pc-windows-gnu     windows-x64  1 || true
  build_cross x86_64-apple-darwin       macos-x64    1 || true
  build_cross aarch64-apple-darwin      macos-arm64  1 || true
fi

log "done - artifacts in $DIST/:"
ls -1 "$DIST"
