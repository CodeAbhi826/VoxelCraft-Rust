//! vc-particles — vanilla-1.16.5-style break/hit particles (§16.2 pass 4):
//! fixed-capacity pool, 20 Hz fixed-step physics, CPU-built billboard
//! vertices with light+tint baked at spawn.

pub mod particles;
pub use particles::*;
