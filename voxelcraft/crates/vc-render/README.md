# vc-render — wgpu renderer, FSR & shader packs

The GPU half: five pipelines (sky / terrain / water / selection /
UI), texture atlas management, animated textures, shadow mapping,
AMD FSR 1.0 (EASU + RCAS), post-processing, and the Iris-style
shader-pack API with runtime WGSL validation.

## What it provides

- `Renderer` — wgpu device + all pipelines, WebGPU with WebGL2 fallback on wasm
- `shaders::parse_pack` / `validate_wgsl` — shader-pack API v1 (§34)
- `Camera`, `PostParams` (FSR quality, shadow quality), `RenderStats`
- `ui` — bitmap font/hotbar/F3 canvas; `textures` — atlas + pack merge

## Dependencies

vc-blocks, vc-gameplay, vc-inventory, vc-mesh, vc-pack, vc-particles, vc-rng, vc-world (+ wgpu, naga)

## Download & use — standalone

This library ships as its **own archive** on the [Releases](https://github.com/CodeAbhi826/VoxelCraft-Rust/releases) page:
`vc-render-0.3.0-source.tar.gz`. There is **no all-in-one bundle** — grab only
the libraries you need.

```sh
tar xzf vc-render-0.3.0-source.tar.gz -C libs/   # download it from the release assets
```

```toml
[dependencies]
vc-render = { path = "libs/vc-render" }
```

Or copy the whole crate folder into any Cargo project — every `vc-*` crate
is self-contained (its internal dependencies are other `vc-*` crates, also
individually downloadable, plus ordinary crates.io dependencies).

## Run its tests

```sh
cargo test -p vc-render                 # from the workspace root
```


## Example

```rust
use vc_render::shaders::{validate_wgsl, builtin_packs};

// shader packs are validated at LOAD time, not on the GPU (§46)
assert!(validate_wgsl("@fragment fn fs() -> @location(0) vec4<f32>                        { return vec4<f32>(1.0, 0.0, 0.0, 1.0); }").is_ok());

for pack in builtin_packs() {
    println!("- {} ({:?})", pack.id, pack.settings.len());
}
```

## Spec reference

Master Spec §16–§18 (Rendering, shadows, vanilla visuals), §33 (FSR), §34 (Shader system)
