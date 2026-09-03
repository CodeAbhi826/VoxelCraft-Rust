//! vc-world — world container, terrain generation & lighting engine
//! (§7/§8 grid+concurrency, §26 worldgen, §27 biomes, §10 light):
//! COW chunk edits, cross-chunk decoration, flood-fill light.

pub mod gen;
pub mod light;
pub mod world;
