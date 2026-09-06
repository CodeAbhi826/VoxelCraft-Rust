# VoxelCraft libraries

The engine is **14 independently usable libraries** plus the game
application. Every library is a normal Rust crate in this workspace,
documented in its own README, tested by `cargo test --workspace`
(426 tests — the E1–E3 + 1.7–1.10 merged suite), and released as its
**own separate archive** — there is
deliberately **no all-in-one bundle**.

| Library | What it is | Depends on | Source |
|---|---|---|---|
| [`vc-nbt`](crates/vc-nbt/README.md) | Minecraft 1.16.5 NBT codec | nothing (only `serde` for derive support) | `crates/vc-nbt` |
| [`vc-blocks`](crates/vc-blocks/README.md) | block registry & BlockState system | nothing | `crates/vc-blocks` |
| [`vc-rng`](crates/vc-rng/README.md) | deterministic RNG | nothing | `crates/vc-rng` |
| [`vc-chunk`](crates/vc-chunk/README.md) | chunk & section storage | vc-blocks (for state queries) | `crates/vc-chunk` |
| [`vc-pack`](crates/vc-pack/README.md) | resource pack / blockstate+model pipeline | nothing internal (serde, serde_json, rayon on native) | `crates/vc-pack` |
| [`vc-inventory`](crates/vc-inventory/README.md) | items, stacks & containers | vc-blocks | `crates/vc-inventory` |
| [`vc-world`](crates/vc-world/README.md) | world grid, terrain generation & light engine | vc-blocks, vc-chunk, vc-rng | `crates/vc-world` |
| [`vc-mesh`](crates/vc-mesh/README.md) | greedy mesher | vc-blocks, vc-chunk, vc-pack, vc-world | `crates/vc-mesh` |
| [`vc-particles`](crates/vc-particles/README.md) | vanilla-style particle pool | vc-blocks, vc-rng, vc-world | `crates/vc-particles` |
| [`vc-gameplay`](crates/vc-gameplay/README.md) | crafting, furnaces, brewing, enchanting, villagers | vc-blocks, vc-chunk, vc-inventory, vc-particles, vc-rng, vc-world | `crates/vc-gameplay` |
| [`vc-sim`](crates/vc-sim/README.md) | deterministic simulation core | vc-blocks, vc-gameplay, vc-particles, vc-rng, vc-world | `crates/vc-sim` |
| [`vc-anvil`](crates/vc-anvil/README.md) | vanilla 1.16.5 save/load | vc-nbt, vc-blocks, vc-chunk, vc-world | `crates/vc-anvil` |
| [`vc-render`](crates/vc-render/README.md) | wgpu renderer, FSR & shader packs | vc-blocks, vc-gameplay, vc-inventory, vc-mesh, vc-pack, vc-particles, vc-rng, vc-world (+ wgpu, naga) | `crates/vc-render` |
| [`vc-audio`](crates/vc-audio/README.md) | synthesized sound bank & events | vc-blocks, vc-rng | `crates/vc-audio` |
| [`voxelcraft`](crates/voxelcraft/) | the game app (binaries) | all of the above | `crates/voxelcraft` |

## Downloading one library

1. Go to [Releases](https://github.com/CodeAbhi826/VoxelCraft-Rust/releases).
2. Download `vc-<name>-0.3.0-source.tar.gz` — **one archive per library**, nothing bundled in.
3. Extract anywhere and reference it by path:

```toml
[dependencies]
vc-nbt = { path = "../libs/vc-nbt" }
```

If a library depends on other `vc-*` libraries (see the table), download
those the same way — each one is small and standalone. Alternatively
clone the repo once and use `path = "../VoxelCraft-Rust/voxelcraft/crates/vc-nbt"`.

## Using the whole workspace

```sh
git clone https://github.com/CodeAbhi826/VoxelCraft-Rust.git
cd VoxelCraft-Rust/voxelcraft
cargo test --workspace          # 426 tests across all libraries
cargo run --release             # the game (from the workspace root)
```

## Layering (who depends on whom)

```
vc-nbt  vc-blocks  vc-rng  vc-chunk  vc-pack        (foundations)
   |        |         |        |         |
   |        +----+----+   vc-inventory  |
   |             |              |       |
   +------ vc-world -------+    |    (vc-pack used by mesh)
                 |          |    |
             vc-mesh    vc-particles
                 |          |
              vc-gameplay --+
                 |
              vc-sim
                 |
   vc-anvil (uses nbt/blocks/chunk/world)
   vc-render (uses mesh, pack, gameplay, particles, ...)
   vc-audio  (uses blocks, rng)
                 |
          voxelcraft (the app — binaries)
```

No cycles: the graph is a strict DAG. `vc-render` and `vc-anvil` are
sibling mid-level libraries; only the app touches everything.
