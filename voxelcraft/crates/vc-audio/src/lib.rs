//! vc-audio — synthesized sound bank & event system (§21): data-driven
//! sound events, categories, spatial audio; rodio on native (feature
//! `audio`), WebAudio on wasm.

pub mod sounds;
pub use sounds::*;
