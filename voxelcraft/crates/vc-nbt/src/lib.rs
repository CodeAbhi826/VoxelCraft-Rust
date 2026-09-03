//! vc-nbt — Minecraft-1.16.5 NBT codec (named binary tag read/write,
//! little/big endian, gzip/zlib-aware consumers live in vc-anvil).

pub mod nbt;
pub use nbt::*;
