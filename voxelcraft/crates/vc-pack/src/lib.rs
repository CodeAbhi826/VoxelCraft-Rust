//! vc-pack — resource-pack & blockstate/model JSON pipeline (§5.2/§19):
//! parses 1.16.5 `blockstates/*.json` + `models/**/*.json`, resolves
//! model inheritance, bakes element rotations, precomputes per-state
//! model dispatch so meshers never parse JSON at mesh time.

pub mod model;
pub mod pack;
