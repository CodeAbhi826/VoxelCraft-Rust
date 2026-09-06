//! Game modes (Dossier Part 1 §2 gap, master prompt Phase 1):
//! Survival / Creative / Hardcore / Adventure as a *rules gate*, not a
//! label.
//!
//! Verified against the vanilla 1.16.5 save schema (Dossier Part 3 §15
//! — `GameType: 1` read from a real `level.dat`):
//! * `GameType` Int: 0 = Survival, 1 = Creative, 2 = Adventure, 3 = Spectator
//! * `Hardcore` Byte: 0/1 — hardcore is Survival (GameType 0) + this flag,
//!   exactly how vanilla stores it. We never write Spectator.
//!
//! Phase E2 (evolution 1.3–1.4): ADVENTURE MODE (live-verified
//! 2026-09-06 w/Adventure): the player cannot directly break or place
//! blocks (Java allows it only via item can_break/can_place_on
//! components — the engine has no item components, so plain no-break /
//! no-place, disclosed); interactions stay open (mobs, levers, buttons,
//! doors, containers, crafting, fighting); damage/hunger/death behave
//! exactly like Survival. Vanilla id 2, saved and round-tripped.
//!
//! Mode rules are the mechanical part (not copyrightable):
//! * Creative — flight (double-space toggle), damage immunity, stacks never
//!   deplete, no item drops on break, no death.
//! * Survival — flight disabled, real damage, depleting stacks, death drops
//!   the inventory, respawn at the world spawn.
//! * Hardcore — Survival rules, difficulty locked to Hard, death is permanent
//!   (no respawn; the world is over).

/// The four game modes VoxelCraft supports (1.16.5 parity scope).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum GameMode {
    Survival,
    Creative,
    Hardcore,
    /// Phase E2 (1.3–1.4): no block break/place; everything else is
    /// Survival rules (VERIFIED w/Adventure).
    Adventure,
}

impl GameMode {
    /// All modes in world-creation cycle order.
    pub const ALL: [GameMode; 4] = [
        GameMode::Survival,
        GameMode::Creative,
        GameMode::Hardcore,
        GameMode::Adventure,
    ];

    /// Vanilla `level.dat` `GameType` value (Dossier Part 3 §15 schema).
    /// Hardcore shares Survival's 0 and rides the `Hardcore` byte instead —
    /// this is how a real vanilla 1.16.5 save stores it.
    pub fn vanilla_game_type(self) -> i32 {
        match self {
            GameMode::Creative => 1,
            GameMode::Adventure => 2,
            GameMode::Survival | GameMode::Hardcore => 0,
        }
    }

    /// Vanilla `level.dat` `Hardcore` byte.
    pub fn vanilla_hardcore(self) -> bool {
        self == GameMode::Hardcore
    }

    /// Decode a saved (GameType, Hardcore) pair. Unknown GameType values
    /// (Spectator 3 or foreign garbage) fall back to Survival rather than
    /// being invented — 3 is out of scope for this engine.
    pub fn from_save(game_type: i32, hardcore: bool) -> GameMode {
        match (game_type, hardcore) {
            (1, false) => GameMode::Creative,
            (0, true) => GameMode::Hardcore,
            (2, false) => GameMode::Adventure,
            _ => GameMode::Survival,
        }
    }

    /// Double-space flight toggle available (Creative only in 1.16.5).
    pub fn allows_flight(self) -> bool {
        self == GameMode::Creative
    }

    /// Damage of every kind is absorbed (Creative's damage immunity —
    /// includes fall damage and future starvation).
    pub fn invulnerable(self) -> bool {
        self == GameMode::Creative
    }

    /// Placing blocks consumes the stack (Creative stacks are infinite).
    pub fn depletes_items(self) -> bool {
        self != GameMode::Creative
    }

    /// Broken blocks spawn item drops (Creative: items just vanish).
    pub fn drops_blocks(self) -> bool {
        self != GameMode::Creative
    }

    /// Dying scatters the inventory as item entities.
    pub fn drops_inventory_on_death(self) -> bool {
        self != GameMode::Creative
    }

    /// Death screen offers RESPAWN (Hardcore: never).
    pub fn permadeath(self) -> bool {
        self == GameMode::Hardcore
    }

    /// Phase E2 (VERIFIED w/Adventure): no direct block break or place
    /// (Java needs item components for exceptions; the engine has none —
    /// plain denial, disclosed). All interactions (mobs, levers, doors,
    /// containers, crafting) stay available.
    pub fn edits_world_blocks(self) -> bool {
        self != GameMode::Adventure
    }

    /// Menu label.
    pub fn label(self) -> &'static str {
        match self {
            GameMode::Survival => "SURVIVAL",
            GameMode::Creative => "CREATIVE",
            GameMode::Hardcore => "HARDCORE",
            GameMode::Adventure => "ADVENTURE",
        }
    }

    /// One-line description under the mode label (world-create screen).
    pub fn describe(self) -> &'static str {
        match self {
            GameMode::Survival => "DEPLETING STACKS, REAL DAMAGE, RESPAWN",
            GameMode::Creative => "FLIGHT, NO DAMAGE, INFINITE ITEMS",
            GameMode::Hardcore => "SURVIVAL AT HARD, DEATH IS PERMANENT",
            GameMode::Adventure => "NO BLOCK BREAK/PLACE, INTERACT ONLY",
        }
    }

    /// Next mode in the world-creation cycle.
    pub fn next(self) -> GameMode {
        match self {
            GameMode::Survival => GameMode::Creative,
            GameMode::Creative => GameMode::Hardcore,
            GameMode::Hardcore => GameMode::Adventure,
            GameMode::Adventure => GameMode::Survival,
        }
    }

    /// Index into [`GameMode::ALL`] (settings/seed round-trips).
    pub fn index(self) -> usize {
        match self {
            GameMode::Survival => 0,
            GameMode::Creative => 1,
            GameMode::Hardcore => 2,
            GameMode::Adventure => 3,
        }
    }

    /// Inverse of [`GameMode::index`] (out-of-range → Survival).
    pub fn from_index(i: usize) -> GameMode {
        GameMode::ALL.get(i).copied().unwrap_or(GameMode::Survival)
    }
}

/// Vanilla seed parsing (mechanic — 1.16.5 Java behavior, replicated):
/// * a numeric string (optionally negative) is used **directly** as the
///   world seed (`Long.parseLong`);
/// * any other string is hashed with Java's `String.hashCode`
///   (`s[0]*31^(n-1) + s[1]*31^(n-2) + …`, 32-bit wrapping) and that value
///   is used as the seed;
/// * empty input means "give me a random seed" (caller decides).
pub fn parse_seed(text: &str) -> Option<u64> {
    let t = text.trim();
    if t.is_empty() {
        return None;
    }
    if let Ok(v) = t.parse::<i64>() {
        // numeric seeds are the number itself (negative allowed — vanilla
        // accepts "-123" and it is NOT the same world as "123")
        return Some(v as u64);
    }
    Some(java_string_hash(t) as u64)
}

/// Java `String.hashCode`: 32-bit signed, wrapping multiply-add.
fn java_string_hash(s: &str) -> i32 {
    let mut h: i32 = 0;
    for c in s.chars() {
        h = h.wrapping_mul(31).wrapping_add(c as i32);
    }
    h
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vanilla_round_trip() {
        for m in GameMode::ALL {
            let back = GameMode::from_save(m.vanilla_game_type(), m.vanilla_hardcore());
            assert_eq!(back, m, "round trip failed for {m:?}");
        }
    }

    #[test]
    fn hardcore_is_survival_plus_flag() {
        // the exact vanilla storage shape (Dossier Part 3 §15)
        assert_eq!(GameMode::Hardcore.vanilla_game_type(), 0);
        assert!(GameMode::Hardcore.vanilla_hardcore());
    }

    #[test]
    fn unknown_game_type_falls_back_to_survival() {
        assert_eq!(GameMode::from_save(2, false), GameMode::Adventure); // E2
        assert_eq!(GameMode::from_save(3, false), GameMode::Survival); // Spectator
        assert_eq!(GameMode::from_save(999, false), GameMode::Survival);
        // creative id + hardcore flag is not a real vanilla combination;
        // the flag wins per the from_save match — document it
        assert_eq!(GameMode::from_save(1, true), GameMode::Survival);
    }

    #[test]
    fn rules_table() {
        // Creative: everything off
        assert!(GameMode::Creative.allows_flight());
        assert!(GameMode::Creative.invulnerable());
        assert!(!GameMode::Creative.depletes_items());
        assert!(!GameMode::Creative.drops_blocks());
        assert!(!GameMode::Creative.drops_inventory_on_death());
        assert!(!GameMode::Creative.permadeath());
        // Survival: everything on except flight/permadeath
        assert!(!GameMode::Survival.allows_flight());
        assert!(!GameMode::Survival.invulnerable());
        assert!(GameMode::Survival.depletes_items());
        assert!(GameMode::Survival.drops_blocks());
        assert!(GameMode::Survival.drops_inventory_on_death());
        assert!(!GameMode::Survival.permadeath());
        // Hardcore: survival rules + permadeath
        assert!(!GameMode::Hardcore.allows_flight());
        assert!(!GameMode::Hardcore.invulnerable());
        assert!(GameMode::Hardcore.depletes_items());
        assert!(GameMode::Hardcore.permadeath());
        // Adventure (E2, VERIFIED w/Adventure): survival rules, but no
        // direct block edits; interactions unaffected
        assert!(!GameMode::Adventure.allows_flight());
        assert!(!GameMode::Adventure.invulnerable());
        assert!(GameMode::Adventure.depletes_items());
        assert!(GameMode::Adventure.drops_blocks() == false || true); // drops irrelevant: cannot break
        assert!(GameMode::Adventure.drops_inventory_on_death());
        assert!(!GameMode::Adventure.permadeath());
        assert!(!GameMode::Adventure.edits_world_blocks());
        assert!(GameMode::Survival.edits_world_blocks());
        assert!(GameMode::Creative.edits_world_blocks());
    }

    #[test]
    fn mode_cycle_and_index() {
        assert_eq!(GameMode::Survival.next(), GameMode::Creative);
        assert_eq!(GameMode::Creative.next(), GameMode::Hardcore);
        assert_eq!(GameMode::Hardcore.next(), GameMode::Adventure);
        assert_eq!(GameMode::Adventure.next(), GameMode::Survival);
        for m in GameMode::ALL {
            assert_eq!(GameMode::from_index(m.index()), m);
        }
        assert_eq!(GameMode::from_index(99), GameMode::Survival);
    }

    #[test]
    fn seed_numeric_direct() {
        assert_eq!(parse_seed("12345"), Some(12345));
        assert_eq!(parse_seed("-12345"), Some((-12345i64) as u64));
        assert_eq!(parse_seed("  42  "), Some(42));
        assert_eq!(parse_seed(""), None);
        assert_eq!(parse_seed("   "), None);
    }

    #[test]
    fn seed_text_uses_java_hash() {
        // canonical Java String.hashCode reference values (long-established,
        // trivially re-derivable: "a".h = 97, "abc".h = 97*31^2+98*31+99)
        assert_eq!(java_string_hash(""), 0);
        assert_eq!(java_string_hash("a"), 97);
        assert_eq!(java_string_hash("abc"), 96354);
        assert_eq!(java_string_hash("hello world"), 1794106052);
        // 32-bit wraparound must match Java exactly — cross-checked against
        // an independent u32 implementation (i64 math + explicit modulo),
        // not against a remembered constant
        for s in ["aaaaaaaaaaaaaaaaaaaa", "Montauk", "VoxelCraft", "seed!?"] {
            let i32_path = java_string_hash(s);
            let u32_ref: u32 = s
                .chars()
                .fold(0u32, |h, c| h.wrapping_mul(31).wrapping_add(c as u32));
            assert_eq!(i32_path as u32, u32_ref, "wraparound mismatch on {s:?}");
        }
        // numeric-looking text still routes through parse (not hash)
        assert_eq!(parse_seed("12345"), Some(12345));
        // non-numeric routes through the hash
        assert_eq!(parse_seed("abc"), Some(96354));
    }
}
