#!/usr/bin/env bash
# VoxelCraft — rebuild the public/ web bundle from HEAD (the LOCKED-PAIR
# rule: voxelcraft.js + voxelcraft_bg.wasm must always be rebuilt and
# copied TOGETHER, never one without the other).
set -euo pipefail
source "$HOME/.cargo/env"
export PKG_CONFIG_PATH="$HOME/sysroot/usr/lib/x86_64-linux-gnu/pkgconfig:/usr/lib/x86_64-linux-gnu/pkgconfig:/usr/share/pkgconfig"
export C_INCLUDE_PATH="$HOME/sysroot/usr/include"
cd /home/z/my-project/voxelcraft

chorus "[web] building wasm32 release lib (this takes a few minutes)"
timeout 500 cargo build --release --no-default-features --target wasm32-unknown-unknown --lib -j 3

chorus "[web] wasm-bindgen 0.2.127 (must match the crate pin)"
rm -rf wasm-out
wasm-bindgen --target web --out-dir ./wasm-out \
  target/wasm32-unknown-unknown/release/voxelcraft.wasm

chorus "[web] patching the JS glue (pointerType hardening)"
python3 patch-wasm-glue.py ./wasm-out/voxelcraft.js

chorus "[web] copying the matched pair + d.ts + pack into public/"
cp wasm-out/voxelcraft.js           ../public/
cp wasm-out/voxelcraft_bg.wasm      ../public/
cp wasm-out/voxelcraft.d.ts         ../public/
cp wasm-out/voxelcraft_bg.wasm.d.ts ../public/
rsync -a --delete builtin-pack/ ../public/voxelcraft-pack/
# 2026-09-14: the Classic Art builtin pack (pre-1.14-style clean-room
# look-alikes) deploys as its own fetch root for the wasm boot
rsync -a --delete builtin-packs/classic-art/ ../public/voxelcraft-pack-classic-art/

chorus "[web] bundle mtime pair (MUST match):"
ls -la ../public/voxelcraft.js ../public/voxelcraft_bg.wasm
chorus "[web] done"
