//! vc-mesh — greedy mesher (§11-§14): section-scoped meshing, merged
//! quads with equal AO/sky tuples, fine-grained invalidation (§12),
//! bytemuck-Pod vertex formats (§13).

pub mod mesh;
pub use mesh::*;
