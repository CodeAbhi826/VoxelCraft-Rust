//! vc-anvil — vanilla 1.16.5 save/load (§28): NBT chunk encoding,
//! Anvil `.mca` region reader/writer, zlib/gzip region payloads.
//! Native-only at runtime (browser saves need OPFS later); the crate
//! still typechecks everywhere, modules are cfg-gated per target.

#[cfg(not(target_arch = "wasm32"))]
pub mod anvil;
#[cfg(not(target_arch = "wasm32"))]
pub mod save;
