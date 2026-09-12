//! `gui` — GUI texture set, loaders and (Phase 2+) the quad frame
//! types for the UI overhaul. Phase 1 is purely additive infrastructure:
//! nothing consumes these textures yet.

pub mod loader;
pub mod set;

pub use set::{FontSource, GuiTextureError, GuiTextureSet, SpriteSheet};
