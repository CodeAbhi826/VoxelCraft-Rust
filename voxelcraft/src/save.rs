//! World save orchestration — chunk ↔ NBT ↔ Anvil + `level.dat`.
//!
//! Phase 2 §28: separates the *internal runtime format* (in-memory chunk
//! grid) from the *external compatibility format* (vanilla 1.16.5 Anvil).
//! The codec layers live in `nbt.rs` (Named Binary Tag) and `anvil.rs`
//! (region container); this module glues them to the live `World`.
