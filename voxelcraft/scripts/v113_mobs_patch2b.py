#!/usr/bin/env python3
"""1.13 mobs.rs patch part 2B: helpers + ai_tick signature + spawns."""
import sys

p = 'crates/vc-gameplay/src/mobs.rs'
t = open(p).read()
n = 0

def rep(old, new, label):
    global t, n
    c = t.count(old)
    if c != 1:
        print(f'FAIL [{label}]: count = {c}')
        sys.exit(1)
    t = t.replace(old, new)
    n += 1
    print(f'ok [{label}]')

# ---- 1. helpers after fn wander ----
rep('''fn wander(rng: &mut Rng, m: &mut Mob, speed: f32) {''',
'''/// 1.13: is the mob's body in water (the swim-physics gate, AI-side).
fn in_water_mob(m: &Mob, world: &World, height: f32) -> bool {
    world.get_block(
        m.pos[0] as i32,
        (m.pos[1] + height * 0.5) as i32,
        m.pos[2] as i32,
    ) == WATER
}

/// 1.13: steer toward a 3D point (the aquatic/phantom movement form —
/// smooth interpolation toward the target velocity).
fn steer_3d(m: &mut Mob, target: [f32; 3], speed: f32) {
    let dx = target[0] - m.pos[0];
    let dy = target[1] - m.pos[1];
    let dz = target[2] - m.pos[2];
    let len = (dx * dx + dy * dy + dz * dz).sqrt().max(1e-4);
    m.vel[0] += (dx / len * speed - m.vel[0]) * 0.15;
    m.vel[1] += (dy / len * speed - m.vel[1]) * 0.15;
    m.vel[2] += (dz / len * speed - m.vel[2]) * 0.15;
    m.yaw = (-dz).atan2(dx) - std::f32::consts::FRAC_PI_2;
}

/// 1.13: 3D wander for aquatic mobs — the land wander with a gentle
/// vertical bob (fish school drift; the y target keeps them mid-column).
fn wander_3d(rng: &mut Rng, m: &mut Mob, speed: f32) {
    wander(rng, m, speed);
    if m.wander_t > 0 && rng.next_f32() < 0.02 {
        // occasional vertical drift: a small target nudge
        m.vel[1] += (rng.next_f32() - 0.5) * 0.4;
    }
}

fn wander(rng: &mut Rng, m: &mut Mob, speed: f32) {''', 'helpers')

# ---- 2. ai_tick signature: the 3 new queues ----
rep('''    // 1.12 illusioner spell queue (game-layer consumption)
    pending_blindness: &mut Vec<i32>,
) {''',
'''    // 1.12 illusioner spell queue (game-layer consumption)
    pending_blindness: &mut Vec<i32>,
    // 1.13 (Update Aquatic) queues (game-layer consumption)
    pending_player_grace: &mut Vec<i32>,
    pending_turtle_eggs: &mut Vec<(i32, i32, i32, u16)>,
    pending_drops: &mut Vec<([f32; 3], u16)>,
) {''', 'ai_tick signature')

# ---- 3. ai_tick call site: pass the queues ----
rep('''        let pending_blindness = &mut self.pending_player_blindness;''',
'''        let pending_blindness = &mut self.pending_player_blindness;
        // 1.13: the aquatic queues
        let pending_grace_q = &mut self.pending_player_grace;
        let pending_turtle_eggs_q = &mut self.pending_turtle_eggs;
        let pending_drops_q = &mut self.pending_drops;''', 'call-site borrows')

rep('''                pending_summons,
                pending_player_fang,
                pending_blindness,
            );
            physics_tick(m, world);''',
'''                pending_summons,
                pending_player_fang,
                pending_blindness,
                pending_grace_q,
                pending_turtle_eggs_q,
                pending_drops_q,
            );
            physics_tick(m, world);''', 'call-site pass')

open(p, 'w').write(t)
print(f'PART 2B DONE — {n} edits')
