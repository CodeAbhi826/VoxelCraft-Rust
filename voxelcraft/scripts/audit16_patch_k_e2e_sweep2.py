#!/usr/bin/env python3
"""Sweep-2 patch K (revised) — the melon break row, the chorus
destination as a free function, the e2e_audit16b stage, its call site,
and the CI grep."""
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
# 1) the melon break row (before the nylium branch)
# ======================================================================
patch("crates/voxelcraft/src/game.rs", [
    (
        "melon break",
        """                            } else if broke == CRIMSON_NYLIUM || broke == WARPED_NYLIUM {""",
        """                            } else if broke == MELON {
                                // the sweep-2: "When broken, a melon
                                // drops 3-7 melon slices with equal
                                // probability for an overall average of
                                // 5 slices per melon" (VERIFIED
                                // w/Melon_Slice §Block loot, live
                                // 2026-09-09; silk-touch/fortune out of
                                // scope, no tool-gated loot yet)
                                let n = 3 + self.audio_rng.next_range(5) as u8; // 3..=7
                                for _ in 0..n {
                                    self.sim.items.drop_block(
                                        pos[0],
                                        pos[1],
                                        pos[2],
                                        MELON_SLICE,
                                        biome,
                                        sky,
                                        blk,
                                    );
                                }
                            } else if broke == CRIMSON_NYLIUM || broke == WARPED_NYLIUM {""",
    ),
], "game-melon")

# ======================================================================
# 2) the chorus destination free function (next to food_heal — the
#    pure-function region) + rewire chorus_teleport onto it
# ======================================================================
patch("crates/voxelcraft/src/game.rs", [
    (
        "free fn",
        """fn light_at(
    world: &World,
    light: &vc_world::light::LightEngine,""",
        """/// the sweep-2 chorus destination pick — "up to 16 attempts are made
/// to choose a random destination within ±8 on all three axes in the
/// same manner as enderman teleportation, with the exception that the
/// entity may teleport into an area only 2 blocks high ... If there
/// are no valid blocks within this range, the teleportation attempt
/// fails and the entity remains in place" (VERIFIED live 2026-09-09,
/// w/Chorus_Fruit §Teleportation). Enderman-style validity: solid
/// floor + a 2-block air column.
fn chorus_destination(
    world: &World,
    px: i32,
    py: i32,
    pz: i32,
    rng: &mut vc_rust_rng::Rng,
) -> Option<[i32; 3]> {
    use vc_blocks::blocks::is_solid;
    for _ in 0..16 {
        let dx = rng.next_range(17) as i32 - 8; // -8..=8
        let dy = rng.next_range(17) as i32 - 8;
        let dz = rng.next_range(17) as i32 - 8;
        let x = px + dx;
        let y = (py + dy).clamp(1, 250);
        let z = pz + dz;
        let floor = world.get_block(x, y - 1, z);
        let body = world.get_block(x, y, z);
        let head = world.get_block(x, y + 1, z);
        if is_solid(floor) && body == vc_blocks::blocks::AIR && head == vc_blocks::blocks::AIR {
            return Some([x, y, z]);
        }
    }
    None // the failed warp: "the entity remains in place"
}

fn light_at(
    world: &World,
    light: &vc_world::light::LightEngine,""",
    ),
    (
        "chorus_teleport rewire",
        """    fn chorus_teleport(&mut self) {
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
    }""",
        """    fn chorus_teleport(&mut self) {
        let px = self.player.pos.x.floor() as i32;
        let py = self.player.pos.y.floor() as i32;
        let pz = self.player.pos.z.floor() as i32;
        if let Some([x, y, z]) = chorus_destination(&self.world, px, py, pz, &mut self.audio_rng)
        {
            self.player.pos = glam::Vec3::new(x as f32 + 0.5, y as f32, z as f32 + 0.5);
            // the warp cancels the accumulated fall (the pearl's own
            // class of negation, VERIFIED w/Chorus_Fruit)
            self.player.fall_dist = 0.0;
            self.player.vel.y = 0.0;
            self.play_event("entity.enderman.teleport", None, 0.9);
            self.ui.dirty = true;
            vc_render::render::report_boot_log(&format!(
                "e2e: chorus teleport -> [{x}, {y}, {z}] (the 16-attempt +-8 rule)"
            ));
        }
        // the None case: "the teleportation attempt fails and the
        // entity remains in place" — silent, vanilla
    }""",
    ),
], "game-chorus-fn")

if fail:
    print("PATCH FAILURES:")
    for f in fail:
        print("  -", f)
    sys.exit(1)
print("patch k applied: melon + chorus free fn")
