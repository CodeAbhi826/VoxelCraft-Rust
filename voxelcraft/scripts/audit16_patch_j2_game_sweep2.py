#!/usr/bin/env python3
"""Sweep-2 patch J part 2 — the chorus teleport method, the throw
branch, and the landing drain (egg hatch + pearl teleport)."""
import sys

fail = []


def patch(path, edits, tag):
    src = open(path).read()
    for what, old, new in edits:
        if new in src:
            continue
        if old not in src or src.count(old) != 1:
            fail.append(f"[{tag}] ANCHOR ({what}): {old[:70]!r}")
            continue
        src = src.replace(old, new)
    open(path, "w").write(src)


# ======================================================================
# 1) the chorus_teleport method (after e2e_audit16's helper region —
#    anchor on test_place, the shared e2e helper)
# ======================================================================
patch("crates/voxelcraft/src/game.rs", [
    (
        "chorus teleport method",
        """    fn test_place(&mut self, block: u16, x: i32, y: i32, z: i32) {""",
        """    /// the sweep-2 chorus teleport — "up to 16 attempts are made to
    /// choose a random destination within ±8 on all three axes in the
    /// same manner as enderman teleportation, with the exception that
    /// the entity may teleport into an area only 2 blocks high ... If
    /// there are no valid blocks within this range, the teleportation
    /// attempt fails and the entity remains in place" (VERIFIED live
    /// 2026-09-09, w/Chorus_Fruit §Teleportation). Enderman-style
    /// validity: a solid floor with two air blocks above it.
    fn chorus_teleport(&mut self) {
        use vc_blocks::blocks::is_solid;
        let px = self.player.pos.x.floor() as i32;
        let py = self.player.pos.y.floor() as i32;
        let pz = self.player.pos.z.floor() as i32;
        for _ in 0..16 {
            let dx = self.audio_rng.next_range(17) as i32 - 8; // -8..=8
            let dy = self.audio_rng.next_range(17) as i32 - 8;
            let dz = self.audio_rng.next_range(17) as i32 - 8;
            let x = px + dx;
            let y = (py + dy).clamp(1, 250);
            let z = pz + dz;
            // the validity triple: solid floor, 2-block air column
            let floor = self.world.get_block(x, y - 1, z);
            let body = self.world.get_block(x, y, z);
            let head = self.world.get_block(x, y + 1, z);
            if is_solid(floor) && body == AIR && head == AIR {
                self.player.pos = glam::Vec3::new(x as f32 + 0.5, y as f32, z as f32 + 0.5);
                // the warp cancels the accumulated fall (the pearl's
                // own class of negation, VERIFIED w/Chorus_Fruit)
                self.player.fall_dist = 0.0;
                self.player.vel.y = 0.0;
                self.play_event("entity.enderman.teleport", None, 0.9);
                self.ui.dirty = true;
                vc_render::render::report_boot_log(&format!(
                    "e2e: chorus teleport -> [{x}, {y}, {z}] (the 16-attempt +-8 rule)"
                ));
                return;
            }
        }
        // "the teleportation attempt fails and the entity remains in
        // place" — no log noise, the failed warp is silent vanilla
    }

    fn test_place(&mut self, block: u16, x: i32, y: i32, z: i32) {""",
    ),
], "game-chorus")

# ======================================================================
# 2) the throw branch (before the is_food eat branch)
# ======================================================================
patch("crates/voxelcraft/src/game.rs", [
    (
        "throw branch",
        """                    } else if !self.player.held().is_empty() && is_food(self.player.held().block) {""",
        """                    } else if !self.player.held().is_empty()
                        && matches!(
                            self.player.held().block,
                            SNOWBALL | EGG | ENDER_PEARL
                        )
                    {
                        // ---- the sweep-2 throwable family (the
                        // 1.0-era player throws, all VERIFIED live
                        // 2026-09-09: w/Snowball "Snowballs can be
                        // thrown by pressing the use button ... do not
                        // deal damage except to blazes, but they still
                        // knock back"; w/Egg "When thrown by pressing
                        // the use button, an egg has a 1/8 chance of
                        // spawning a chick"; w/Ender_Pearl "can be
                        // thrown by pressing the use button, which
                        // consumes the item and teleports the player to
                        // where the pearl lands, dealing 5 HP damage")
                        // ----
                        // The engine's disclosed adaptation: thrown
                        // projectiles fly STRAIGHT (the fireball-class
                        // convention — vanilla's 0.03-0.04 gravity and
                        // 30 b/s arc are documented deviations).
                        let b = self.player.held().block;
                        let eye = self.player.eye().to_array();
                        let dir = self.player.look_dir().to_array();
                        let kind = match b {
                            SNOWBALL => vc_gameplay::mobs::ProjKind::Snowball,
                            EGG => vc_gameplay::mobs::ProjKind::Egg,
                            _ => vc_gameplay::mobs::ProjKind::Pearl,
                        };
                        self.sim.mobs.arrows.push(vc_gameplay::mobs::Arrow {
                            pos: [
                                eye[0] + dir[0] * 0.8,
                                eye[1] + dir[1] * 0.8,
                                eye[2] + dir[2] * 0.8,
                            ],
                            vel: [dir[0] * 24.0, dir[1] * 24.0, dir[2] * 24.0],
                            damage: 0.0, // the thrown class: knockback,
                            // blaze damage via the snowball branch, the
                            // egg/pearl payloads at landing
                            age: 0,
                            kind,
                            owner: vc_gameplay::mobs::PLAYER_OWNER,
                        });
                        if self.mode.depletes_items() {
                            let held = self.player.held_mut();
                            held.count -= 1;
                            if held.count == 0 {
                                *held = vc_inventory::inventory::ItemStack::EMPTY;
                            }
                        }
                        // the pearl's "cooldown of one second (20
                        // ticks)" (VERIFIED w/Ender_Pearl); the light
                        // pair use the standard use cooldown
                        self.place_timer = if b == ENDER_PEARL { 1.0 } else { 0.3 };
                        let what = if b == SNOWBALL {
                            "snowball"
                        } else if b == EGG {
                            "egg"
                        } else {
                            "ender pearl"
                        };
                        self.play_event("entity.snowball.throw", None, 0.8);
                        vc_render::render::report_boot_log(&format!(
                            "e2e: threw a {what} (PLAYER_OWNER, 24 b/s straight)"
                        ));
                        self.ui.dirty = true;
                    } else if !self.player.held().is_empty() && is_food(self.player.held().block) {""",
    ),
], "game-throw")

# ======================================================================
# 3) the landing drain (after the take_target_hits loop)
# ======================================================================
patch("crates/voxelcraft/src/game.rs", [
    (
        "landing drain",
        """        let hits = mobs::take_target_hits(&mut self.sim.mobs);
        for (pos, power, ticks) in hits {""",
        """        // ---- the sweep-2: the throwable landing queue — eggs
        // hatch, pearls teleport (all VERIFIED live 2026-09-09) ----
        let landings: Vec<(mobs::ProjKind, [f32; 3])> =
            mobs::take_landings(&mut self.sim.mobs);
        for (kind, pos) in landings {
            match kind {
                mobs::ProjKind::Egg => {
                    // "an egg has a 1/8 (12.5%) chance of spawning a
                    // chick. If this occurs, there is a 1/32 (3.125%)
                    // chance of spawning three additional chicks"
                    // (VERIFIED w/Egg §Spawning chickens)
                    if self.audio_rng.next_range(8) == 0 {
                        let x = pos[0].floor() as i32;
                        let y = pos[1].floor() as i32 + 1;
                        let z = pos[2].floor() as i32;
                        // the hatch cell: 2-block air above the hit
                        if self.world.get_block(x, y, z) == AIR
                            && self.world.get_block(x, y + 1, z) == AIR
                        {
                            let spawn_chick = |sim: &mut crate::sim::Sim, x: i32, y: i32, z: i32| {
                                let _ = sim.mobs.spawn_variant(
                                    mobs::MobKind::Chicken,
                                    x,
                                    y,
                                    z,
                                    0x40, // the baby bit (the fox/turtle
                                    // maturity class)
                                );
                                // the 20-minute chick maturity (the
                                // generic 0x40 countdown)
                                if let Some(m) = sim.mobs.list.last_mut() {
                                    m.aux = 24000;
                                }
                            };
                            spawn_chick(&mut self.sim, x, y, z);
                            let mut n = 1;
                            if self.audio_rng.next_range(32) == 0 {
                                for _ in 0..3 {
                                    spawn_chick(&mut self.sim, x, y, z);
                                }
                                n = 4;
                            }
                            self.play_event("entity.chicken.ambient", Some(pos), 0.9);
                            vc_render::render::report_boot_log(&format!(
                                "e2e: an egg hatched {n} chick(s) (1/8, 1/32 rows)"
                            ));
                        }
                    }
                }
                mobs::ProjKind::Pearl => {
                    // "teleports the player to where the pearl lands,
                    // dealing 5 HP damage" (VERIFIED w/Ender_Pearl) +
                    // the pre-landing throw negates the accumulated
                    // fall ("the fall damage is negated, dealing only
                    // the pearl's damage")
                    let x = pos[0].floor() as i32;
                    let z = pos[2].floor() as i32;
                    // find the standing cell: walk up from the hit
                    // block until a 2-air column sits on solid ground
                    let mut y = pos[1].floor() as i32 + 1;
                    for _ in 0..6 {
                        if self.world.get_block(x, y, z) == AIR
                            && self.world.get_block(x, y + 1, z) == AIR
                            && is_solid(self.world.get_block(x, y - 1, z))
                        {
                            break;
                        }
                        y += 1;
                    }
                    self.player.pos =
                        glam::Vec3::new(x as f32 + 0.5, y as f32, z as f32 + 0.5);
                    self.player.fall_dist = 0.0;
                    self.player.vel.y = 0.0;
                    if !self.mode.invulnerable() && self.mode.depletes_items() {
                        self.player.damage(5.0);
                    }
                    self.play_event("entity.enderman.teleport", None, 0.9);
                    self.ui.dirty = true;
                    vc_render::render::report_boot_log(&format!(
                        "e2e: pearl teleport -> [{x}, {y}, {z}] + 5 HP (VERIFIED)"
                    ));
                }
                _ => {}
            }
        }
        let hits = mobs::take_target_hits(&mut self.sim.mobs);
        for (pos, power, ticks) in hits {""",
    ),
], "game-drain")

if fail:
    print("PATCH FAILURES:")
    for f in fail:
        print("  -", f)
    sys.exit(1)
print("patch j part 2 applied: chorus + throw + drain")
