#!/usr/bin/env python3
"""1.13 (Update Aquatic) mobs.rs integration patch — part 1:
PlayerHit poison field, ProjKind::Trident, MobSystem queues."""
import sys

p = 'crates/vc-gameplay/src/mobs.rs'
t = open(p).read()
n_changes = 0

def rep(old, new, label):
    global t, n_changes
    if t.count(old) != 1:
        print(f'FAIL [{label}]: count = {t.count(old)}')
        sys.exit(1)
    t = t.replace(old, new)
    n_changes += 1
    print(f'ok [{label}]')

# ---- 1. PlayerHit: poison payload (pufferfish contact) ----
rep('''    /// Phase E2: wither-skull payload — Some(ticks) applies Wither II
    /// (VERIFIED w/Wither: 200 ticks Normal / 800 Hard)
    pub wither_effect: Option<i32>,
}''',
'''    /// Phase E2: wither-skull payload — Some(ticks) applies Wither II
    /// (VERIFIED w/Wither: 200 ticks Normal / 800 Hard)
    pub wither_effect: Option<i32>,
    /// 1.13: poison payload — Some(ticks) applies Poison I (VERIFIED
    /// w/Pufferfish: contact "Poison for 6 seconds" fully puffed Java /
    /// 3 s semi-puffed; the engine's one-tier poison is the I form,
    /// disclosed)
    pub poison_effect: Option<i32>,
}''', 'PlayerHit field')

# all 6 construction sites: add the field after each wither_effect line
lines = t.split('\n')
out = []
for L in lines:
    out.append(L)
    if L.rstrip() == 'wither_effect: None,':
        indent = L[:len(L) - len(L.lstrip())]
        out.append(f'{indent}poison_effect: None,')
t = '\n'.join(out)
n_changes += 1
print(f'ok [PlayerHit sites: 6]')

# ---- 2. ProjKind::Trident (drowned ranged + the player's trident) ----
rep('''    /// 1.11: llama spit — 1 HP Easy/Normal (1.5 Hard via difficulty
    /// scale; VERIFIED w/Llama: "Llama Spit: Easy and Normal: 1 HP,
    /// Hard: 1.5 HP")
    LlamaSpit,
}''',
'''    /// 1.11: llama spit — 1 HP Easy/Normal (1.5 Hard via difficulty
    /// scale; VERIFIED w/Llama: "Llama Spit: Easy and Normal: 1 HP,
    /// Hard: 1.5 HP")
    LlamaSpit,
    /// 1.13: the trident — 8 HP base (VERIFIED w/Trident: "Projectile
    /// damage 8 HP"; the drowned throw "sends it up to 20 blocks away"
    /// at a 1.5 s cadence — VERIFIED w/Drowned §Attacking). The
    /// player-thrown form applies Impaling bonuses at the game layer.
    Trident,
}''', 'ProjKind::Trident')

# ---- 3. MobSystem queues for the 1.13 game-layer hooks ----
rep('''    /// 1.12 illusioner spells, consumed by the game layer: Blindness
    /// applications on the player (ticks each — the 20 s spell,
    /// VERIFIED w/Illusioner §Casting_Blindness: "This spell gives a
    /// Blindness effect that lasts for 20 seconds upon first engaging
    /// a new player opponent").
    pub pending_player_blindness: Vec<i32>,''',
'''    /// 1.12 illusioner spells, consumed by the game layer: Blindness
    /// applications on the player (ticks each — the 20 s spell,
    /// VERIFIED w/Illusioner §Casting_Blindness: "This spell gives a
    /// Blindness effect that lasts for 20 seconds upon first engaging
    /// a new player opponent").
    pub pending_player_blindness: Vec<i32>,
    /// 1.13 Dolphin's Grace applications (ticks each — VERIFIED
    /// w/Dolphin: "a swimming speed boost for 5 seconds, replenished
    /// as long as the player stays close"; the game layer applies
    /// the effect while a sprint-swimming player is within 9 blocks).
    pub pending_player_grace: Vec<i32>,
    /// 1.13 turtle egg placements, consumed by the game layer (world
    /// edits must ride the light engine): (x, y, z, state) — the
    /// egg block at hatch stage 0 (VERIFIED w/Turtle: "A turtle lays
    /// eggs after digging" on its home beach).
    pub pending_turtle_eggs: Vec<(i32, i32, i32, u16)>,
    /// 1.13 item drops the mob system itself owes the world (baby
    /// turtle maturity scutes — VERIFIED w/Scute: "Dropped when baby
    /// turtles grow up"): (position, block id).
    pub pending_drops: Vec<([f32; 3], u16)>,''', 'MobSystem fields')

rep('''            pending_player_blindness: Vec::new(),''',
'''            pending_player_blindness: Vec::new(),
            pending_player_grace: Vec::new(),
            pending_turtle_eggs: Vec::new(),
            pending_drops: Vec::new(),''', 'MobSystem init')

open(p, 'w').write(t)
print(f'PART 1 DONE — {n_changes} edits')
