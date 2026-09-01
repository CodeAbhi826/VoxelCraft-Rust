# CI Setup — Auto-build WASM on GitHub Actions

**Status:** the workflow file ships at [`ci/wasm-build.yml`](../ci/wasm-build.yml) but is **not yet live** — the automation token currently has `repo` scope only, and GitHub refuses to let a token without `workflow` scope create files under `.github/workflows/` (verified via git push AND the REST API). One 30-second action activates it:

## Activate (30 seconds, no tools)

1. Open **https://github.com/CodeAbhi826/VoxelCraft-Rust/new/main?filename=.github/workflows/wasm-build.yml**
   (new-file editor with the path pre-filled)
2. Open **https://github.com/CodeAbhi826/VoxelCraft-Rust/blob/main/ci/wasm-build.yml** → *Raw* → copy all.
3. Paste into the editor → **Commit changes** (to `main` directly).
4. Done. The workflow self-triggers immediately (its `paths` filter includes itself) and runs the first build.

*(Alternative: mint a classic token with `repo` + `workflow` scopes and hand it over — the assistant can then push the file itself.)*

## What it does after activation

On every push to `main` that touches `voxelcraft/**` (and on manual dispatch):

1. `cargo build --release --target wasm32-unknown-unknown --lib` (Rust stable, cargo cache)
2. `wasm-bindgen 0.2.127 --target web` → `voxelcraft/wasm-out/`
3. `python3 voxelcraft/patch-wasm-glue.py` (re-applies the winit `pointerType` hardening that wasm-bindgen regeneration would otherwise wipe)
4. Copies `voxelcraft.js` + `voxelcraft_bg.wasm` (+ `.d.ts`) into `public/`
5. Commits the bundle back to `main` as `ci(wasm): auto-rebuild bundle … [skip ci]` and uploads it as a workflow artifact
6. A parallel `native-check` job runs `cargo check --release` on Linux (with ALSA headers) so native breakage is caught too

Re-trigger guard: `public/**` changes don't match the `paths` filter, and the artifact commit carries `[skip ci]` — no build loops.

**Result: no one compiles anything locally anymore.** Edit Rust → push → CI builds → bundle lands in `public/` in the repo.

## Pulling CI-built bundles locally (sandbox/preview)

```sh
scripts/sync-wasm-from-ci.sh           # wait for local HEAD's CI run, then git pull
scripts/sync-wasm-from-ci.sh --no-wait # just fast-forward to latest main
```

The loader at `public/voxelcraft.html` derives its cache-bust version from the wasm's `Last-Modified`/`ETag`, so a pulled bundle is picked up on next page load — no browser cache dance.

## Keeping the template in sync

`ci/wasm-build.yml` is the source of truth. If you edit it, also update the live copy at `.github/workflows/wasm-build.yml` (web UI edit, or a `workflow`-scoped token) — otherwise CI keeps running the old version.
