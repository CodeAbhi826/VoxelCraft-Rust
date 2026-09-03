# vc-inventory — items, stacks & containers

ItemStacks, slots and container logic shared by the player
inventory, crafting grids, furnaces, brewing stands and villager
trading — including enchantment carry-through on every move.

## What it provides

- `ItemStack { block, count, ench }` with `EMPTY`, `new`, `new_enchanted`
- `Inventory::new(capacity)` with slot/cursor semantics (36-slot player layout)
- `STACK_MAX = 64`, hotbar/storage index constants

## Dependencies

vc-blocks

## Download & use — standalone

This library ships as its **own archive** on the [Releases](https://github.com/CodeAbhi826/VoxelCraft-Rust/releases) page:
`vc-inventory-0.3.0-source.tar.gz`. There is **no all-in-one bundle** — grab only
the libraries you need.

```sh
tar xzf vc-inventory-0.3.0-source.tar.gz -C libs/   # download it from the release assets
```

```toml
[dependencies]
vc-inventory = { path = "libs/vc-inventory" }
```

Or copy the whole crate folder into any Cargo project — every `vc-*` crate
is self-contained (its internal dependencies are other `vc-*` crates, also
individually downloadable, plus ordinary crates.io dependencies).

## Run its tests

```sh
cargo test -p vc-inventory                 # from the workspace root
```


## Example

```rust
use vc_inventory::{Inventory, ItemStack};
use vc_blocks::blocks::COBBLE;

let mut inv = Inventory::new(36);
inv.add(COBBLE, 64);              // returns leftover (0 = fully stacked)

assert_eq!(inv.count_of(COBBLE), 64);
inv.consume(COBBLE, 32);
assert_eq!(inv.count_of(COBBLE), 32);
```

## Spec reference

Master Spec §29 (Items, Containers and Crafting)
