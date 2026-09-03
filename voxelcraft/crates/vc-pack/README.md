# vc-pack — resource pack / blockstate+model pipeline

Parses real 1.16.5-format resource packs: `blockstates/*.json`,
`models/**/*.json`, `pack.mcmeta`, model inheritance, element rotations,
multipart conditions — and bakes a per-state model dispatch so meshers
never parse JSON at mesh time. Runs from folders on native, from
fetched bytes on the web.

## What it provides

- `FolderSource` / `MemorySource` behind the `PackSource` trait
- `pack::open()` validating `pack.mcmeta` (pack_format 6 = 1.16.2–1.16.5)
- `model::compile_block_dispatch()` → `ModelSet` (parse once, canonicalize, cache)
- wasm: `fetch_builtin_pack()` streams the deployed pack into memory

## Dependencies

nothing internal (serde, serde_json, rayon on native)

## Download & use — standalone

This library ships as its **own archive** on the [Releases](https://github.com/CodeAbhi826/VoxelCraft-Rust/releases) page:
`vc-pack-0.3.0-source.tar.gz`. There is **no all-in-one bundle** — grab only
the libraries you need.

```sh
tar xzf vc-pack-0.3.0-source.tar.gz -C libs/   # download it from the release assets
```

```toml
[dependencies]
vc-pack = { path = "libs/vc-pack" }
```

Or copy the whole crate folder into any Cargo project — every `vc-*` crate
is self-contained (its internal dependencies are other `vc-*` crates, also
individually downloadable, plus ordinary crates.io dependencies).

## Run its tests

```sh
cargo test -p vc-pack                 # from the workspace root
```


## Example

```rust
use std::sync::Arc;
use vc_pack::pack::{FolderSource, PackSource, open};

let source = Arc::new(FolderSource::new("builtin-pack", "builtin"));
assert!(source.exists());
let (meta, _src) = open(source).unwrap();
assert_eq!(meta.pack_format, 6);        // 1.16.5
```

## Spec reference

Master Spec §5.2 (Blockstate/model data), §19 (Resource Pack and Texture Pipeline)
