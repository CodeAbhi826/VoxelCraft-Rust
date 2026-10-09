//! 2.3a: chunk loading tickets — who keeps a chunk loaded.
//!
//! A ticket is a claim "chunk (cx,cz) must stay loaded". Two kinds:
//! Player (simulation + render radius around each player) and Forced
//! (spawn area / forceload-style pins). 2.3b wires producers in:
//! the streaming disc maintains Player claims every frame and the
//! unload path consults is_loaded (a Forced pin holds a far chunk);
//! counts saturate at zero on over-release (R6: corrupt input never
//! panics extends to caller bugs — degrade, don't trap).

use rustc_hash::{FxHashMap, FxHashSet};

/// What kind of claim pins a chunk.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TicketKind {
    Player,
    Forced,
}

#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
struct Counts {
    player: u32,
    forced: u32,
}

#[derive(Clone, Debug, Default)]
pub struct TicketTable {
    counts: FxHashMap<(i32, i32), Counts>,
}

impl TicketTable {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add one claim of `kind` on (cx, cz).
    pub fn acquire(&mut self, cx: i32, cz: i32, kind: TicketKind) {
        let e = self.counts.entry((cx, cz)).or_default();
        match kind {
            TicketKind::Player => e.player = e.player.saturating_add(1),
            TicketKind::Forced => e.forced = e.forced.saturating_add(1),
        }
    }

    /// Drop one claim; over-release saturates (returns false) instead
    /// of panicking. Empty entries are removed so the table only ever
    /// holds live claims.
    pub fn release(&mut self, cx: i32, cz: i32, kind: TicketKind) -> bool {
        let Some(e) = self.counts.get_mut(&(cx, cz)) else {
            return false;
        };
        let slot = match kind {
            TicketKind::Player => &mut e.player,
            TicketKind::Forced => &mut e.forced,
        };
        if *slot == 0 {
            return false;
        }
        *slot -= 1;
        if e.player == 0 && e.forced == 0 {
            self.counts.remove(&(cx, cz));
        }
        true
    }

    /// Any live claim keeps the chunk loaded.
    pub fn is_loaded(&self, cx: i32, cz: i32) -> bool {
        self.counts.contains_key(&(cx, cz))
    }

    /// A forced pin (spawn/forceload) independent of players.
    pub fn is_forced(&self, cx: i32, cz: i32) -> bool {
        self.counts.get(&(cx, cz)).is_some_and(|e| e.forced > 0)
    }

    pub fn claims(&self) -> usize {
        self.counts.len()
    }
}

/// 2.3b: reconcile one player's claims to the streaming disc. Idempotent
/// — call every frame: chunks entering the disc are acquired once (the
/// held set keeps counts 0/1), claims outside the unload margin (disc+3)
/// are released. Forced pins are never touched (different holder).
pub fn sync_player_disc(
    t: &mut TicketTable,
    held: &mut FxHashSet<(i32, i32)>,
    pc: (i32, i32),
    rd: i32,
) {
    let r = rd.max(0);
    held.retain(|p| {
        let d = (p.0 - pc.0).abs().max((p.1 - pc.1).abs());
        if d > r + 3 {
            t.release(p.0, p.1, TicketKind::Player);
            false
        } else {
            true
        }
    });
    for dx in -r..=r {
        for dz in -r..=r {
            if dx.abs().max(dz.abs()) > r {
                continue;
            }
            let p = (pc.0 + dx, pc.1 + dz);
            if held.insert(p) {
                t.acquire(p.0, p.1, TicketKind::Player);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn acquire_release_counts() {
        let mut t = TicketTable::new();
        assert!(!t.is_loaded(0, 0));
        t.acquire(0, 0, TicketKind::Player);
        t.acquire(0, 0, TicketKind::Player);
        t.acquire(1, 0, TicketKind::Forced);
        assert!(t.is_loaded(0, 0));
        assert!(t.is_loaded(1, 0));
        assert!(!t.is_forced(0, 0));
        assert!(t.is_forced(1, 0));
        assert_eq!(t.claims(), 2);
        assert!(t.release(0, 0, TicketKind::Player));
        assert!(t.is_loaded(0, 0));
        assert!(t.release(0, 0, TicketKind::Player));
        assert!(!t.is_loaded(0, 0));
        assert_eq!(t.claims(), 1);
    }

    #[test]
    fn over_release_saturates_without_panic() {
        let mut t = TicketTable::new();
        assert!(!t.release(5, 5, TicketKind::Player));
        t.acquire(5, 5, TicketKind::Forced);
        assert!(!t.release(5, 5, TicketKind::Player));
        assert!(t.is_loaded(5, 5));
        assert!(t.release(5, 5, TicketKind::Forced));
        assert!(!t.is_loaded(5, 5));
    }

    #[test]
    fn sync_disc_claims_chebyshev_square() {
        let mut t = TicketTable::new();
        let mut held = FxHashSet::default();
        sync_player_disc(&mut t, &mut held, (0, 0), 2);
        // radius-2 Chebyshev disc = 5x5
        assert_eq!(t.claims(), 25);
        assert_eq!(held.len(), 25);
        assert!(t.is_loaded(2, -2));
        assert!(!t.is_loaded(3, 0));
    }

    #[test]
    fn sync_disc_idempotent_and_follows_player() {
        let mut t = TicketTable::new();
        let mut held = FxHashSet::default();
        sync_player_disc(&mut t, &mut held, (0, 0), 2);
        // second call changes nothing (counts stay 0/1, no growth)
        sync_player_disc(&mut t, &mut held, (0, 0), 2);
        assert_eq!(t.claims(), 25);
        // stride east past the unload margin: old edge released
        sync_player_disc(&mut t, &mut held, (10, 0), 2);
        assert!(!t.is_loaded(0, 0));
        assert!(t.is_loaded(10, 0));
        assert_eq!(t.claims(), 25);
    }

    #[test]
    fn sync_disc_never_touches_forced() {
        let mut t = TicketTable::new();
        let mut held = FxHashSet::default();
        t.acquire(100, 100, TicketKind::Forced);
        sync_player_disc(&mut t, &mut held, (0, 0), 2);
        assert!(t.is_forced(100, 100));
        assert_eq!(t.claims(), 26);
    }
}
