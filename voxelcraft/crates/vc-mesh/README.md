# vc-mesh — greedy mesher

Compiles chunks into renderable geometry: greedy quad merging
with equal AO/sky corner tuples (vanilla smooth lighting that still
merges), per-vertex ambient occlusion, section-scoped meshing for
fine-grained invalidation, and bytemuck-Pod vertex formats.

## What it provides

- `mesh_chunk(pos, snap, lsnap, smooth)` — the full-chunk entry point
- `mesh_sections(...)` — per-section meshes for §12 invalidation granularity
- `Vertex` (16 B) / `MeshData` / `MeshOut` — ready for GPU buffers

## Dependencies

vc-blocks, vc-chunk, vc-pack, vc-world

## Download & use — standalone

This library ships as its **own archive** on the [Releases](https://github.com/CodeAbhi826/VoxelCraft-Rust/releases) page:
`vc-mesh-0.3.0-source.tar.gz`. There is **no all-in-one bundle** — grab only
the libraries you need.

```sh
tar xzf vc-mesh-0.3.0-source.tar.gz -C libs/   # download it from the release assets
```

```toml
[dependencies]
vc-mesh = { path = "libs/vc-mesh" }
```

Or copy the whole crate folder into any Cargo project — every `vc-*` crate
is self-contained (its internal dependencies are other `vc-*` crates, also
individually downloadable, plus ordinary crates.io dependencies).

## Run its tests

```sh
cargo test -p vc-mesh                 # from the workspace root
```


## Example

```rust
use vc_world::world::World;
use vc_world::gen::TerrainGen;
use vc_mesh::mesh::mesh_chunk;

let mut world = World::new(12648430);
let mut gen = TerrainGen::new(12648430);
let (chunk, outbound) = gen.generate_chunk(0, 0, Vec::new());
world.insert_generated((0, 0), chunk, outbound);

if let Some(snap) = world.snapshot3x3(0, 0) {
    // reference light bridge (same one the tests use):
    let lsnap = vc_world::light::reference_lightdata(&snap);
    let mesh = mesh_chunk((0, 0), &snap, &lsnap, /*smooth lighting*/ true);
    println!("{} vertices, {} indices", mesh.vertices.len(), mesh.indices.len());
}
```

## Spec reference

Master Spec §11–§14 (Meshing, invalidation, vertex formats, GPU storage)
