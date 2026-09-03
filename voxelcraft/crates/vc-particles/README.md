# vc-particles — vanilla-style particle pool

Break/hit particles with vanilla 1.16.5 physics: fixed-capacity
pool, 20 Hz fixed-step integration (gravity 0.04/t, air friction 0.98),
CPU-built billboard vertices with light and tint baked at spawn.

## What it provides

- `ParticleSystem::new(seed)` — pool with `MAX_PARTICLES = 4096` cap
- `update(dt, &world)` fixed-step simulation against blocks
- `ParticleVertex` (Pod) buffers ready for the billboard pass

## Dependencies

vc-blocks, vc-rng, vc-world

## Download & use — standalone

This library ships as its **own archive** on the [Releases](https://github.com/CodeAbhi826/VoxelCraft-Rust/releases) page:
`vc-particles-0.3.0-source.tar.gz`. There is **no all-in-one bundle** — grab only
the libraries you need.

```sh
tar xzf vc-particles-0.3.0-source.tar.gz -C libs/   # download it from the release assets
```

```toml
[dependencies]
vc-particles = { path = "libs/vc-particles" }
```

Or copy the whole crate folder into any Cargo project — every `vc-*` crate
is self-contained (its internal dependencies are other `vc-*` crates, also
individually downloadable, plus ordinary crates.io dependencies).

## Run its tests

```sh
cargo test -p vc-particles                 # from the workspace root
```


## Example

```rust
use vc_particles::particles::ParticleSystem;
use vc_world::world::World;
use vc_blocks::blocks::STONE;

let mut ps = ParticleSystem::new(42);
let world = World::new(42);
ps.spawn_block_break(0, 70, 0, STONE, /*biome*/ 0, /*sky*/ 15, /*blk*/ 0);
ps.update(1.0 / 20.0, &world);   // one 20 Hz fixed step
assert!(!ps.is_empty());
```

## Spec reference

Master Spec §16.2 pass 4 (particles between water and clouds)
