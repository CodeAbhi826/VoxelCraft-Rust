# vc-world — world grid, terrain generation & light engine

The heart of the engine: the chunk `World` with copy-on-write
edits and cross-chunk decoration, deterministic terrain generation
(noise, biomes, caves, trees, villages, the Nether), and the flood-fill
block/skylight engine with incremental updates.

## What it provides

- `World::new(seed)` (+ `new_in_dimension`) — chunk map, COW edits, `snapshot3x3`
- `TerrainGen::generate_chunk(cx, cz, inbound)` — deterministic, decoration included
- `LightEngine` — block light + skylight BFS with incremental relight (§10/§12)
- `Dimension` (Overworld + Nether with 8:1 coordinate mapping)

## Dependencies

vc-blocks, vc-chunk, vc-rng

## Download & use — standalone

This library ships as its **own archive** on the [Releases](https://github.com/CodeAbhi826/VoxelCraft-Rust/releases) page:
`vc-world-0.3.0-source.tar.gz`. There is **no all-in-one bundle** — grab only
the libraries you need.

```sh
tar xzf vc-world-0.3.0-source.tar.gz -C libs/   # download it from the release assets
```

```toml
[dependencies]
vc-world = { path = "libs/vc-world" }
```

Or copy the whole crate folder into any Cargo project — every `vc-*` crate
is self-contained (its internal dependencies are other `vc-*` crates, also
individually downloadable, plus ordinary crates.io dependencies).

## Run its tests

```sh
cargo test -p vc-world                 # from the workspace root
```


## Example

```rust
use vc_world::world::World;
use vc_world::gen::TerrainGen;
use vc_world::light::LightEngine;

let mut world = World::new(12648430);
let mut gen = TerrainGen::new(12648430);
let (chunk, outbound) = gen.generate_chunk(0, 0, Vec::new());
world.insert_generated((0, 0), chunk, outbound);

let mut light = LightEngine::new();
light.init_chunk(&mut world, (0, 0));       // flood-fill block+skylight
light.pump(&mut world, 65_536);             // drain the BFS queue
```

## Spec reference

Master Spec §7/§8 (World grid & concurrency), §10 (Lighting), §26/§27 (Worldgen & biomes)
