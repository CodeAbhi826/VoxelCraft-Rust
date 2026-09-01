#!/usr/bin/env bash
# Waits for the GitHub Actions "Build WASM" workflow to finish (for a given
# SHA, or the local HEAD), then pulls the auto-committed bundle from main so
# the local preview at http://localhost:81/ serves the CI-built WASM.
#
# Usage:
#   scripts/sync-wasm-from-ci.sh            # wait for local HEAD's run, then pull
#   scripts/sync-wasm-from-ci.sh --no-wait  # just pull latest main
set -euo pipefail
cd "$(dirname "$0")/.."

TOKEN_FILE="scripts/gh-token.txt"
REPO="CodeAbhi826/VoxelCraft-Rust"
WF="Build%20WASM.yml"   # URL-encoded workflow file name

[ -f "$TOKEN_FILE" ] || { echo "missing $TOKEN_FILE"; exit 1; }
TOKEN=$(cat "$TOKEN_FILE")

API="https://api.github.com/repos/$REPO/actions/workflows/$WF/runs"
AUTH=(-H "Authorization: token $TOKEN" -H "Accept: application/vnd.github+json")

if [ "${1:-}" = "--no-wait" ]; then
  git pull --ff-only origin main
  echo "Pulled latest main (bundle included if CI pushed one)."
  exit 0
fi

SHA=$(git rev-parse HEAD)
echo "Waiting for Build WASM run of ${SHA::12} ..."

for i in $(seq 1 90); do   # up to ~15 min
  RUN=$(curl -s "${AUTH[@]}" "$API?head_sha=$SHA" \
        | python3 -c 'import json,sys; r=json.load(sys.stdin).get("workflow_runs",[]); print(r[0]["status"]+"/"+r[0]["conclusion"] if r else "none")' 2>/dev/null || echo "none")
  case "$RUN" in
    completed/success) echo "CI: success"; break ;;
    completed/*)       echo "CI finished: $RUN — check https://github.com/$REPO/actions"; exit 1 ;;
    none)              : ;;  # run not registered yet
    *)                 echo "  ... $RUN" ;;
  esac
  sleep 10
  [ "$i" = "90" ] && { echo "timeout waiting for CI"; exit 1; }
done

git pull --ff-only origin main
echo "Synced. Bundle in public/ is now CI-built."
