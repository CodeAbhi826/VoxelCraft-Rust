#!/usr/bin/env bash
# VoxelCraft — rebuild the public/ web bundle from HEAD (the LOCKED-PAIR
# rule: voxelcraft.js + voxelcraft_bg.wasm must always be rebuilt and
# copied TOGETHER, never one without the other).
set -euo pipefail
source "$HOME/.cargo/env"
cd /home/z/my-project/voxelcraft

echo "[web] building wasm32 release lib (this takes a few minutes)"
cargo build --release --no-default-features --target wasm32-unknown-unknown --lib

echo "[web] wasm-bindgen 0.2.127 (must match the crate pin)"
rm -rf wasm-out
wasm-bindgen --target web --out-dir ./wasm-out \
  target/wasm32-unknown-unknown/release/voxelcraft.wasm

echo "[web] patching the JS glue (pointerType hardening)"
python3 patch-wasm-glue.py ./wasm-out/voxelcraft.js

echo "[web] copying the matched pair + d.ts + pack into public/"
cp wasm-out/voxelcraft.js           ../public/
cp wasm-out/voxelcraft_bg.wasm      ../public/
cp wasm-out/voxelcraft.d.ts         ../public/
cp wasm-out/voxelcraft_bg.wasm.d.ts ../public/
rsync -a --delete builtin-pack/ ../public/voxelcraft-pack/

echo "[web] bundle mtime pair (MUST match):"
ls -la ../public/voxelcraft.js ../public/voxelcraft_bg.wasm
echo "[web] done"
