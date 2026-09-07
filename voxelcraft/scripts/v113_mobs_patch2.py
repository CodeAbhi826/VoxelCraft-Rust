#!/usr/bin/env python3
"""1.13 mobs.rs patch part 2: water physics, AI behaviors, spawns."""
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

# ---- 1. physics_tick: aquatic buoyancy branch ----
rep('''fn physics_tick(m: &mut Mob, world: &World) {
    let d = def(m.kind);''',
'''fn physics_tick(m: &mut Mob, world: &World) {
    let d = def(m.kind);
    // ---- 1.13 (Update Aquatic): aquatic swim physics. In water the
    // aquatic kinds get buoyancy + drag instead of gravity (fish hover,
    // turtles/dolphins glide); out of water the fish family
    // suffocates (VERIFIED w/Cod: fish "cannot survive out of water
    // ... they start suffocating" — 1 HP/s engine form of vanilla's
    // 10-tick no-air window, disclosed approximation) and flips. ----
    let body_block = world.get_block(
        m.pos[0] as i32,
        (m.pos[1] + d.height * 0.5) as i32,
        m.pos[2] as i32,
    );
    let in_water = body_block == WATER;
    if m.kind.aquatic() && in_water {
        // buoyancy: relax toward zero vertical speed, water drag on all
        // axes (vanilla swim drag 0.8-ish; engine approximation)
        m.vel[1] *= 0.8;
        m.vel[0] *= 0.92;
        m.vel[2] *= 0.92;
        m.fall_dist = 0.0; // water breaks falls (the player rule)
    } else if m.kind.aquatic() {
        // fish out of water: flop + suffocate (the fish family only —
        // dolphins/turtles/drowned breathe air)
        if matches!(
            m.kind,
            MobKind::Cod | MobKind::Salmon | MobKind::Pufferfish | MobKind::TropicalFish
        ) {
            m.health -= 1.0 / 20.0; // ~1 HP/s (documented approximation)
            // flop: a small random hop (vanilla fish flop on land)
            m.vel[0] *= 0.9;
            m.vel[2] *= 0.9;
        }
    }''', 'physics water branch')

# ---- 2. ai_tick: the 1.13 behavior block (inserted after the
# zombie-villager cure block, before the snow-golem targeting) ----
rep('''    // ---- Phase E1: snow golem targeting (the heat rule ran above,
    // before the player-anchor early return) — throws snowballs at the
    // nearest hostile ≤ 10 blocks, 1/s (VERIFIED: "They throw one
    // snowball per second")''',
'''    // ---- 1.13 (Update Aquatic) mob behaviors ----
    // PHANTOM: the insomnia swooper — circles 12 blocks above the
    // player, dives on alignment (the classic orbit-and-swoop cycle,
    // VERIFIED w/Phantom §Behavior: "circles ... swoops down"); aux =
    // phase timer (200-tick orbit, 60-tick dive)
    if m.kind == MobKind::Phantom {
        if m.aux > 0 {
            m.aux -= 1;
        }
        let mode = if m.aux == 0 { 1 } else { 0 };
        let orbit_y = p[1] + 12.0;
        if mode == 0 {
            // orbit: circle the player at radius 8
            let ang = (m.pos[0] - p[0]).atan2(m.pos[2] - p[2]);
            let next_ang = ang + 0.05;
            let tx = p[0] + next_ang.sin() * 8.0;
            let tz = p[2] + next_ang.cos() * 8.0;
            let ty = orbit_y;
            steer_3d(m, [tx, ty, tz], speed * 1.2);
            if m.aux <= 0 {
                m.aux = 60; // dive window
            }
        } else {
            // dive: straight at the player's chest
            steer_3d(m, [p[0], p[1] + 1.0, p[2]], speed * 1.8);
            let d3 = ((p[0] - m.pos[0]).powi(2)
                + (p[1] - m.pos[1]).powi(2)
                + (p[2] - m.pos[2]).powi(2))
            .sqrt();
            if !invuln && d3 < 1.4 && m.attack_cd == 0 {
                m.attack_cd = 20;
                hits.push(PlayerHit {
                    damage: d.damage, // E/N 2 (VERIFIED, 1.14-pre3 value)
                    source: m.kind,
                    knockback_dir: [dx / dist, dz / dist],
                    wither_effect: None,
                    poison_effect: None,
                });
            }
            if m.aux <= 0 {
                m.aux = 200; // back to orbit
            }
        }
        return;
    }

    // PUFFERFISH: the inflating contact defender — inflates as the
    // player closes within 3 blocks, contact damage + Poison at
    // touch (VERIFIED w/Pufferfish: semi 2 HP E/N + 3 s poison, fully
    // 3 HP N + 6 s poison Java; the engine's one-tier poison is I)
    if m.kind == MobKind::Pufferfish {
        let near = dist < 3.0 && !invuln;
        // inflate/deflate one step per 20 ticks
        if m.attack_cd == 0 {
            m.attack_cd = 20;
            if near && m.variant < 2 {
                m.variant += 1;
            } else if !near && m.variant > 0 {
                m.variant -= 1;
            }
        }
        let inflated = m.variant >= 1;
        if inflated
            && !invuln
            && dist < d.width * 0.5 + 0.9
            && m.aux <= 0
        {
            m.aux = 20; // contact cadence (~0.5 s, the immunity window)
            hits.push(PlayerHit {
                damage: if m.variant == 2 { d.damage } else { 2.0 },
                source: m.kind,
                knockback_dir: [dx / dist, dz / dist],
                wither_effect: None,
                // 3 s semi / 6 s fully puffed (VERIFIED Java rows)
                poison_effect: Some(if m.variant >= 2 { 120 } else { 60 }),
            });
        }
        if m.aux > 0 {
            m.aux -= 1;
        }
        wander_3d(rng, m, speed);
        return;
    }

    // TURTLE: beach nester — adults wander toward water on land, swim
    // freely in water; a bred female (variant bit 7 = carrying) lays an
    // egg on sand (the queued world edit), babies mature on the aux
    // countdown and drop a scute (VERIFIED w/Scute: "Dropped when baby
    // turtles grow up")
    if m.kind == MobKind::Turtle {
        // baby growth: aux counts down to maturity
        if m.variant & 0x40 != 0 && m.aux > 0 {
            m.aux -= 1;
            if m.aux == 0 {
                // matured: clear the baby bit + queue the scute drop
                m.variant &= !0x40;
                pending_drops.push((m.pos, SCUTE));
            }
        }
        // egg laying (bred female, standing on sand)
        if m.variant & 0x80 != 0 && m.on_ground {
            let below = world.get_block(
                m.pos[0] as i32,
                (m.pos[1] - 0.1) as i32,
                m.pos[2] as i32,
            );
            if below == SAND || below == RED_SAND {
                m.variant &= !0x80; // egg laid
                pending_turtle_eggs.push((
                    m.pos[0] as i32,
                    m.pos[1] as i32,
                    m.pos[2] as i32,
                    0, // stage 0 state id written by the game layer
                ));
            }
        }
        // land movement: head downhill (toward water) at slow pace —
        // a turtle "generally attempt to move to near water"
        // (VERIFIED w/Turtle §Behavior)
        if in_water_mob(m, world, d.height) {
            wander_3d(rng, m, speed * 3.0);
        } else {
            wander(rng, m, speed);
        }
        return;
    }

    // DOLPHIN: the pod grace-giver — any player sprint-swimming within
    // a 9-block sphere banks Dolphin's Grace 5 s (VERIFIED w/Dolphin);
    // provoked pods melee like the wolf-pack pattern
    if m.kind == MobKind::Dolphin {
        let dd = ((p[0] - m.pos[0]).powi(2)
            + (p[1] - m.pos[1]).powi(2)
            + (p[2] - m.pos[2]).powi(2))
        .sqrt();
        if !invuln && dd <= 9.0 && m.aux <= 0 {
            m.aux = 20; // re-apply at most 1/s (replenished, VERIFIED)
            pending_player_grace.push(100); // 5 s (VERIFIED)
        }
        if m.aux > 0 {
            m.aux -= 1;
        }
        if m.provoked && !invuln && dist < MOB_MELEE_REACH + 0.8 && m.attack_cd == 0 {
            m.attack_cd = MOB_MELEE_TICKS;
            face_player(m);
            hits.push(PlayerHit {
                damage: d.damage, // N 3 (VERIFIED)
                source: m.kind,
                knockback_dir: [dx / dist, dz / dist],
                wither_effect: None,
                poison_effect: None,
            });
            return;
        }
        if in_water_mob(m, world, d.height) {
            wander_3d(rng, m, speed);
        } else {
            wander(rng, m, speed * 0.5);
        }
        return;
    }

    // FISH (cod/salmon/tropical): 3D school wander in water
    if matches!(
        m.kind,
        MobKind::Cod | MobKind::Salmon | MobKind::TropicalFish
    ) {
        wander_3d(rng, m, speed);
        return;
    }

    // DROWNED: zombie-parity melee chase, PLUS the trident throw for
    // the 6.25% armed roll (VERIFIED w/Drowned §Attacking: "A drowned
    // with a trident can throw it every 1.5 seconds, sending it up to
    // 20 blocks away")
    if m.kind == MobKind::Drowned {
        if m.variant & 1 != 0 && aggro && dist <= 20.0 && dist > 4.0 && m.attack_cd == 0 {
            m.attack_cd = 30; // 1.5 s (VERIFIED)
            face_player(m);
            spawn_projectile(m, p, rng, arrows, ProjKind::Trident, 20.0, 8.0);
            return;
        }
        // falls through to the generic hostile melee chase below
    }

    // ---- Phase E1: snow golem targeting (the heat rule ran above,
    // before the player-anchor early return) — throws snowballs at the
    // nearest hostile ≤ 10 blocks, 1/s (VERIFIED: "They throw one
    // snowball per second")''', 'ai 1.13 behaviors')

open(p, 'w').write(t)
print(f'PART 2A DONE — {n} edits')
