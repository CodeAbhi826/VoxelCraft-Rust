# vc-anvil — vanilla 1.16.5 save/load

Writes and reads real Anvil-format worlds: NBT-encoded chunk
data inside `.mca` region files (zlib/gzip), `level.dat` handling and
dimension directories. Round-trips bit-identically with vanilla 1.16.5
data.

## What it provides

- `save::chunk_to_nbt` / `chunk_from_nbt` (with light data) — §28 codec
- `anvil::read_chunk` / `write_chunk` — region-file sector management
- `save::write_level_dat` / `read_level_dat`, `WorldMeta`, `PlayerMeta`

## Dependencies

vc-nbt, vc-blocks, vc-chunk, vc-world

## Download & use — standalone

This library ships as its **own archive** on the [Releases](https://github.com/CodeAbhi826/VoxelCraft-Rust/releases) page:
`vc-anvil-0.3.0-source.tar.gz`. There is **no all-in-one bundle** — grab only
the libraries you need.

```sh
tar xzf vc-anvil-0.3.0-source.tar.gz -C libs/   # download it from the release assets
```

```toml
[dependencies]
vc-anvil = { path = "libs/vc-anvil" }
```

Or copy the whole crate folder into any Cargo project — every `vc-*` crate
is self-contained (its internal dependencies are other `vc-*` crates, also
individually downloadable, plus ordinary crates.io dependencies).

## Run its tests

```sh
cargo test -p vc-anvil                 # from the workspace root
```


## Example

```rust
use vc_anvil::save::{chunk_to_nbt, write_level_dat, WorldMeta};
use vc_anvil::anvil::write_chunk;
use vc_world::gen::TerrainGen;

let (chunk, _) = TerrainGen::new(12648430).generate_chunk(0, 0, Vec::new());
let nbt = chunk_to_nbt(0, 0, &chunk, 2586, None);   // 1.16.5 DataVersion

let world_dir = std::path::Path::new("my-world");
write_chunk(world_dir, 0, 0, &nbt).unwrap();
write_level_dat(world_dir, &WorldMeta { seed: 12648430, ..Default::default() }).unwrap();
```

## Spec reference

Master Spec §28 (Save/Load and Data Compatibility)
