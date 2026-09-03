# vc-chunk — chunk & section storage

The memory layout of the world: 16×256×16 chunks split into
16 paletted sections (4096 cells each), plus heightmaps and the
world-grid constants shared by every other crate.

## What it provides

- `Chunk` with `set`/`get`, `set_state` and section-level access
- Paletted `Section` containers (§6) — 10.2 KiB average per chunk measured
- `CHUNK_X/Y/Z`, `SEA_LEVEL`, `CHUNK_LEN`, `idx(x, y, z)`

## Dependencies

vc-blocks (for state queries)

## Download & use — standalone

This library ships as its **own archive** on the [Releases](https://github.com/CodeAbhi826/VoxelCraft-Rust/releases) page:
`vc-chunk-0.3.0-source.tar.gz`. There is **no all-in-one bundle** — grab only
the libraries you need.

```sh
tar xzf vc-chunk-0.3.0-source.tar.gz -C libs/   # download it from the release assets
```

```toml
[dependencies]
vc-chunk = { path = "libs/vc-chunk" }
```

Or copy the whole crate folder into any Cargo project — every `vc-*` crate
is self-contained (its internal dependencies are other `vc-*` crates, also
individually downloadable, plus ordinary crates.io dependencies).

## Run its tests

```sh
cargo test -p vc-chunk                 # from the workspace root
```


## Example

```rust
use vc_chunk::{Chunk, CHUNK_Y, SEA_LEVEL};
use vc_blocks::blocks::{STONE, AIR};

let mut c = Chunk::empty();
c.set(8, 64, 8, STONE);
assert_eq!(c.get(8, 64, 8), STONE);
assert_eq!(CHUNK_Y, 256);
assert_eq!(SEA_LEVEL, 62);
```

## Spec reference

Master Spec §6 (Chunk and Section Storage)
