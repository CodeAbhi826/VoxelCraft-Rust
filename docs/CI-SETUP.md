# CI Setup — Auto-build WASM on GitHub Actions

**Status:** LIVE. The workflow runs from [`.github/workflows/wasm-build.yml`](../.github/workflows/wasm-build.yml) (the historical `ci/wasm-build.yml` copy below is retained as the original template). The automation token now carries `repo` + `workflow` scopes, so workflow files can be pushed directly — no web-UI step needed anymore.

> The repo also runs two more workflows: `ci.yml` (the test gate — whole workspace, wasm check, headless bench) and `linux-game.yml` (the **single-file Linux game**: one executable with the builtin pack embedded, built on ubuntu-22.04; reusable via `workflow_call` so every Release ships it too). See `.github/workflows/`.

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

`ci/wasm-build.yml` is the historical template; `.github/workflows/wasm-build.yml` is the live source of truth (the token's `workflow` scope lets the assistant push updates to it directly).
