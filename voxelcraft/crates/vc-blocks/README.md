# vc-blocks — block registry & BlockState system

The block universe of the engine, modeled on 1.16.5 patterns:
per-block properties, state-id packing, sound families, and the biome
tint math for grass/foliage/water.

## What it provides

- Block ids (u8) and state ids (u16) with `default_state` / `state_block` conversion
- Property blocks (logs with axis, slabs, stairs, fences) — `PROP_BLOCKS`
- Query helpers: `is_solid`, `is_opaque`, `is_solid`, `emissive`, blast resistance
- `tint` module: `grass_color(biome)`, `block_face_tint_packed(...)` — vanilla-style biome coloring

## Dependencies

nothing

## Download & use — standalone

This library ships as its **own archive** on the [Releases](https://github.com/CodeAbhi826/VoxelCraft-Rust/releases) page:
`vc-blocks-0.3.0-source.tar.gz`. There is **no all-in-one bundle** — grab only
the libraries you need.

```sh
tar xzf vc-blocks-0.3.0-source.tar.gz -C libs/   # download it from the release assets
```

```toml
[dependencies]
vc-blocks = { path = "libs/vc-blocks" }
```

Or copy the whole crate folder into any Cargo project — every `vc-*` crate
is self-contained (its internal dependencies are other `vc-*` crates, also
individually downloadable, plus ordinary crates.io dependencies).

## Run its tests

```sh
cargo test -p vc-blocks                 # from the workspace root
```


## Example

```rust
use vc_blocks::blocks::{self as b, default_state, state_block};

let state = default_state(b::OAK_LOG);          // picks a valid 1.16.5 state id
let block = state_block(state);                 // ..and back to the block id
assert_eq!(block, b::OAK_LOG);

assert!(b::is_solid(b::STONE));
assert_eq!(b::emissive(b::GLOWSTONE), 15);      // light level

// biome tint (vanilla grass colormap curve):
let [r, g, bl] = vc_blocks::tint::grass_color(0); // plains
```

## Spec reference

Master Spec §5 (Block Registry and BlockState System), §18 (biome tint)
