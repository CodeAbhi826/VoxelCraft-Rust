# vc-audio — synthesized sound bank & events

Every sound is synthesized in code (no asset files): 64-slot
procedural bank, data-driven sound events with categories and volumes,
spatial panning. rodio on native (feature `audio`), WebAudio on wasm.

## What it provides

- `SoundBank::generate()` — the full procedural bank
- `SoundRegistry` — data-driven events/categories (§21)
- `to_wav16` / `resample` — export and pitch-shift helpers

## Dependencies

vc-blocks, vc-rng

## Download & use — standalone

This library ships as its **own archive** on the [Releases](https://github.com/CodeAbhi826/VoxelCraft-Rust/releases) page:
`vc-audio-0.3.0-source.tar.gz`. There is **no all-in-one bundle** — grab only
the libraries you need.

```sh
tar xzf vc-audio-0.3.0-source.tar.gz -C libs/   # download it from the release assets
```

```toml
[dependencies]
vc-audio = { path = "libs/vc-audio" }
```

Or copy the whole crate folder into any Cargo project — every `vc-*` crate
is self-contained (its internal dependencies are other `vc-*` crates, also
individually downloadable, plus ordinary crates.io dependencies).

## Run its tests

```sh
cargo test -p vc-audio                 # from the workspace root
```


## Example

```rust
use vc_audio::sounds::{SoundBank, to_wav16};

let bank = SoundBank::generate();       // fully synthesized, no asset files
// wavs[i] is a ready WAV blob; index maps recipe names -> slots:
let i = bank.index["dig/grass1"];
std::fs::write("dig-grass.wav", &bank.wavs[i]).unwrap();
```

## Spec reference

Master Spec §21 (Audio)
