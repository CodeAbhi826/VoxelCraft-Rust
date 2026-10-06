# `public/` — the committed web bundle (STALE)

**The WASM files in this directory are a stale snapshot. Do not ship them.**

| File | Built | Engine commit |
|---|---|---|
| `voxelcraft_bg.wasm` | **2026-09-26 03:11** | the `test/full-sweep-2026-09-25` HEAD of that day |
| `voxelcraft.js` | **2026-09-26 03:11** | same build — the two are a locked pair |

As of **2026-10-06** the Rust engine is ~10 days and many hundreds of commits
ahead of this bundle. Anything a player loads through the site today runs
**September's engine**, not the current one.

## Why it is stale

`ci/wasm-build.yml` builds the bundle on every push to `main` and uploads it
as a **workflow artifact** — it deliberately does *not* commit the bundle back,
because a ~6 MB binary on every Rust change had already pushed ~687 MB into the
git history (see the comment at the top of that workflow). The consequence is
that the committed copy in `public/` stopped being refreshed the moment that
policy landed. Nothing has been rebuilding it since.

## Getting a fresh bundle

Release builds do not run locally. Use either:

1. **CI (recommended)** — the `voxelcraft-wasm-bundle` artifact from
   `Build WASM`. The workflow now also triggers on `test/full-sweep-2026-09-25`,
   so the artifact is rebuilt whenever the engine changes on that branch:

   ```
   gh workflow run wasm-build.yml --ref test/full-sweep-2026-09-25
   ```

2. **Locally** — `voxelcraft/scripts/rebuild-web-bundle.sh`, which rebuilds
   `voxelcraft.js` and `voxelcraft_bg.wasm` **together** and copies the pair.
   Never replace one without the other; a mismatched pair fails at load.

## Resolution (Phase 0, item: "rebuild or clearly mark the stale WASM bundle")

This file is the "clearly mark" half of that instruction. The rebuild itself is
a release build and is deferred to Actions — see the two options above. When a
fresh pair lands here, update the table at the top with its real build date and
commit SHA, and delete this warning.

**Verified 2026-10-06** by file mtime against `git log -1 --format=%cd`.