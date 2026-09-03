# vc-nbt — Minecraft 1.16.5 NBT codec

Reads and writes Named Binary Tag data exactly the way vanilla
1.16.5 does: little-endian on the network, big-endian in the
`.mca`/`level.dat` files, with full support for every tag type
(byte … long-array), homogeneous lists and ordered compounds.

## What it provides

- `Nbt` enum covering all 13 tag types (Java-shaped: signed bytes, i32/i64 arrays)
- `write_root(name, &Nbt)` / `read_root(bytes)` — complete root-compound round-trips
- Strict, allocation-conscious parser with corrupt-file tolerance (`NbtError`)
- Round-trip tests asserting bit-identical output

## Dependencies

nothing (only `serde` for derive support)

## Download & use — standalone

This library ships as its **own archive** on the [Releases](https://github.com/CodeAbhi826/VoxelCraft-Rust/releases) page:
`vc-nbt-0.3.0-source.tar.gz`. There is **no all-in-one bundle** — grab only
the libraries you need.

```sh
tar xzf vc-nbt-0.3.0-source.tar.gz -C libs/   # download it from the release assets
```

```toml
[dependencies]
vc-nbt = { path = "libs/vc-nbt" }
```

Or copy the whole crate folder into any Cargo project — every `vc-*` crate
is self-contained (its internal dependencies are other `vc-*` crates, also
individually downloadable, plus ordinary crates.io dependencies).

## Run its tests

```sh
cargo test -p vc-nbt                 # from the workspace root
```


## Example

```rust
use vc_nbt::{Nbt, read_root, write_root};

let root = Nbt::Compound(vec![
    ("name".into(), Nbt::String("VoxelCraft".into())),
    ("DataVersion".into(), Nbt::Int(2586)),          // 1.16.5
    ("Items".into(), Nbt::List(vec![
        Nbt::Compound(vec![("Count".into(), Nbt::Byte(64))]),
    ])),
]);

let bytes = write_root("", &root).unwrap();
let (name, back) = read_root(&bytes).unwrap();
assert_eq!(back, root);
```

## Spec reference

Master Spec §28 (Save/Load and Data Compatibility)
