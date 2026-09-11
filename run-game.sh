#!/bin/bash
# Convenient launcher for the latest single-file Linux release build from CI
DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BIN=$(ls -t "$DIR/build-artifact/voxelcraft-linux-single-file"/voxelcraft-main-*-linux-x64 2>/dev/null | head -n1)

if [ -z "$BIN" ] || [ ! -f "$BIN" ]; then
    echo "[voxelcraft] Error: No Linux game binary found in build-artifact/voxelcraft-linux-single-file/" >&2
    exit 1
fi

chmod +x "$BIN"
echo "[voxelcraft] Launching: $(basename "$BIN")"
exec "$BIN" "$@"
