# vc-sim — deterministic simulation core

The 20 Hz heartbeat: fluids, gravity and item entities, redstone
wire/torch propagation, and the block-entity ticker that drives
furnaces, brewing stands and enchanting tables. Fixed timestep,
seeded randomness, replayable.

## What it provides

- `Sim::new(seed)` + `update(dt, &mut world, &mut light)` — the whole tick
- `TickScheduler` / `RandomTicker` (§25 timing semantics)
- `entities` module — item entities with vanilla gravity/drag

## Dependencies

vc-blocks, vc-gameplay, vc-particles, vc-rng, vc-world

## Download & use — standalone

This library ships as its **own archive** on the [Releases](https://github.com/CodeAbhi826/VoxelCraft-Rust/releases) page:
`vc-sim-0.3.0-source.tar.gz`. There is **no all-in-one bundle** — grab only
the libraries you need.

```sh
tar xzf vc-sim-0.3.0-source.tar.gz -C libs/   # download it from the release assets
```

```toml
[dependencies]
vc-sim = { path = "libs/vc-sim" }
```

Or copy the whole crate folder into any Cargo project — every `vc-*` crate
is self-contained (its internal dependencies are other `vc-*` crates, also
individually downloadable, plus ordinary crates.io dependencies).

## Run its tests

```sh
cargo test -p vc-sim                 # from the workspace root
```


## Example

```rust
use vc_sim::sim::Sim;
use vc_world::world::World;
use vc_world::light::LightEngine;

let mut world = World::new(7);
let mut light = LightEngine::new();
let mut sim = Sim::new(7);

for _ in 0..20 {                       // one vanilla second
    sim.update(1.0 / 20.0, &mut world, &mut light);
}
```

## Spec reference

Master Spec §9 (Deterministic Simulation), §24/§25 (Fluids, Redstone)
