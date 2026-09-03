# vc-rng — deterministic RNG

A small, fast, seeded RNG used everywhere determinism matters
(worldgen, entity AI, differential tests). Same seed, same sequence,
on every platform — native and WASM.

## What it provides

- `Rng::new(seed)` with `next_u64` / float / range helpers
- Split-stream friendly (derive sub-RNGs per chunk/system without correlation)
- Zero dependencies — drop it into any project

## Dependencies

nothing

## Download & use — standalone

This library ships as its **own archive** on the [Releases](https://github.com/CodeAbhi826/VoxelCraft-Rust/releases) page:
`vc-rng-0.3.0-source.tar.gz`. There is **no all-in-one bundle** — grab only
the libraries you need.

```sh
tar xzf vc-rng-0.3.0-source.tar.gz -C libs/   # download it from the release assets
```

```toml
[dependencies]
vc-rng = { path = "libs/vc-rng" }
```

Or copy the whole crate folder into any Cargo project — every `vc-*` crate
is self-contained (its internal dependencies are other `vc-*` crates, also
individually downloadable, plus ordinary crates.io dependencies).

## Run its tests

```sh
cargo test -p vc-rng                 # from the workspace root
```


## Example

```rust
use vc_rng::Rng;

let mut rng = Rng::new(12648430);
let a = rng.next_u64();
let mut rng2 = Rng::new(12648430);
assert_eq!(rng2.next_u64(), a);      // deterministic across platforms
```

## Spec reference

Master Spec §9 (Deterministic Simulation)
