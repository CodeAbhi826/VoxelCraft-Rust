# vc-gameplay — crafting, furnaces, brewing, enchanting, villagers

The gameplay rule systems on top of the world: shaped/shapeless
crafting, furnace smelting with real fuel values, 400-tick brewing
cycles, enchanting tables with bookshelf power and XP curves, and
villager professions with trade tables and wander AI.

## What it provides

- `craft::match_grid` / `consume_grid` — vanilla-shaped recipe matching
- `furnace::Furnaces` — 20 Hz block-entity state with `fuel_ticks`/`smelt_result`
- `brewing::brew_result`, `enchanting::xp_to_next`, `villagers::trades(profession)`
- Deterministic: everything is pure functions + plain state structs

## Dependencies

vc-blocks, vc-chunk, vc-inventory, vc-particles, vc-rng, vc-world

## Download & use — standalone

This library ships as its **own archive** on the [Releases](https://github.com/CodeAbhi826/VoxelCraft-Rust/releases) page:
`vc-gameplay-0.3.0-source.tar.gz`. There is **no all-in-one bundle** — grab only
the libraries you need.

```sh
tar xzf vc-gameplay-0.3.0-source.tar.gz -C libs/   # download it from the release assets
```

```toml
[dependencies]
vc-gameplay = { path = "libs/vc-gameplay" }
```

Or copy the whole crate folder into any Cargo project — every `vc-*` crate
is self-contained (its internal dependencies are other `vc-*` crates, also
individually downloadable, plus ordinary crates.io dependencies).

## Run its tests

```sh
cargo test -p vc-gameplay                 # from the workspace root
```


## Example

```rust
use vc_gameplay::villagers;
use vc_blocks::blocks::IRON_ORE;

// farmer trade #0: give (block, count) -> get (block, count)
let t = &villagers::trades(0)[0];
println!("give {:?} -> get {:?}", t.give, t.get);

// smelting + fuel lookups (vanilla values)
assert!(vc_gameplay::furnace::smelt_result(IRON_ORE).is_some());
assert!(vc_gameplay::furnace::fuel_ticks(vc_blocks::blocks::PLANKS) > 0);
```

## Spec reference

Master Spec §27/§29 (Entities & AI, Items/Containers/Crafting)
