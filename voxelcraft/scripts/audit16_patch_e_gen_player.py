#!/usr/bin/env python3
"""gen.rs + player.rs patch — the cave-spider/silverfish spawners and the
Jump Boost launch hook."""
import sys

fail = []

def patch(path, edits):
    src = open(path).read()
    for what, old, new in edits:
        if old not in src or src.count(old) != 1:
            fail.append(f"[{path}] ANCHOR ({what}): {old[:70]!r}")
            continue
        src = src.replace(old, new)
    open(path, "w").write(src)

# ---------------- gen.rs ----------------
patch("crates/vc-world/src/gen.rs", [
    # 1) the mineshaft spawner: the real cave-spider state
    (
        "mineshaft spawner",
        "                chunk.set_state(lxi, ms.y as usize, lzi, spawner_state(2));",
        """                // the completeness audit: the REAL cave-spider
                // spawner (VERIFIED w/Cave_Spider: "Mineshaft: from
                // monster spawners") — replacing the disclosed
                // spider-spawner adaptation ("no distinct cave-spider
                // mob"); the cobweb nest around it stays palette-absent
                chunk.set_state(lxi, ms.y as usize, lzi, SPAWNER_CAVESPIDER);""",
    ),
    # 2) the adaptation comment — updated to record the retirement
    (
        "mineshaft comment",
        """    // "Crossings: dual-floor, 5×5 intersections"; spider spawners sit in
    // cobwebbed side passages; chest loot = chests/abandoned_mineshaft.
    // ADAPTED (palette): oak instead of vanilla mixed timber; chest as a
    // plain CHEST block (no chest-minecart entity); no rails/cobwebs
    // (palette-absent, honestly documented); cave-spider spawner → the
    // registry's spider spawner (no distinct cave-spider mob).""",
        """    // "Crossings: dual-floor, 5×5 intersections"; spider spawners sit in
    // cobwebbed side passages; chest loot = chests/abandoned_mineshaft.
    // ADAPTED (palette): oak instead of vanilla mixed timber; chest as a
    // plain CHEST block (no chest-minecart entity); no rails/cobwebs
    // (palette-absent, honestly documented); the cave-spider spawner
    // landed its own mob in the 1.0-1.16.5 completeness audit (the old
    // spider-spawner stand-in retired, 2026-09-08).""",
    ),
    # 3) the stronghold portal room: the silverfish spawner
    (
        "stronghold spawner",
        """        // the 12-frame ring: 3 per side, gap at the corners (vanilla
        // 1.16.5 portal room layout)
        for i in 0..3 {
            put(chunk, px + (i - 1), y + 1, pz - 2, END_PORTAL_FRAME);
            put(chunk, px + (i - 1), y + 1, pz + 2, END_PORTAL_FRAME);
            put(chunk, px - 2, y + 1, pz + (i - 1), END_PORTAL_FRAME);
            put(chunk, px + 2, y + 1, pz + (i - 1), END_PORTAL_FRAME);
        }""",
        """        // the 12-frame ring: 3 per side, gap at the corners (vanilla
        // 1.16.5 portal room layout)
        for i in 0..3 {
            put(chunk, px + (i - 1), y + 1, pz - 2, END_PORTAL_FRAME);
            put(chunk, px + (i - 1), y + 1, pz + 2, END_PORTAL_FRAME);
            put(chunk, px - 2, y + 1, pz + (i - 1), END_PORTAL_FRAME);
            put(chunk, px + 2, y + 1, pz + (i - 1), END_PORTAL_FRAME);
        }
        // the completeness audit: the stronghold's silverfish spawner —
        // VERIFIED (minecraft.wiki/w/Silverfish, live 2026-09-08, capture
        // scripts/audit16_page_Silverfish.json): "Stronghold: from
        // infested blocks and monster spawners". The engine form: one
        // spawner in the portal room's upper center (vanilla's own
        // placement class — the ledge above the lava pool; the exact
        // vanilla offset is per-stronghold random, the room center is
        // the engine's deterministic stand-in, disclosed). The
        // infested-block family is palette-absent, disclosed.
        put(chunk, px, y + 4, pz, SPAWNER);
        {
            let lxi = px - ox;
            let lzi = pz - oz;
            if (0..16).contains(&lxi) && (0..16).contains(&lzi) {
                chunk.set_state(lxi as usize, (y + 4) as usize, lzi as usize, SPAWNER_SILVERFISH);
            }
        }""",
    ),
])

# ---------------- player.rs ----------------
patch("crates/voxelcraft/src/player.rs", [
    # the manual jump
    (
        "manual jump",
        """                if input.jump && self.on_ground {
                    // 1.15: jumping off honey is the 3/16-block hop
                    // (the 85% height cut — VERIFIED w/Honey_Block)
                    self.vel.y = if feet_on_honey { JUMP_VEL * HONEY_JUMP_CUT } else { JUMP_VEL };""",
        """                if input.jump && self.on_ground {
                    // 1.15: jumping off honey is the 3/16-block hop
                    // (the 85% height cut — VERIFIED w/Honey_Block)
                    // the completeness audit: Jump Boost adds +0.1 b/t
                    // per level to the launch (VERIFIED w/Effect
                    // §Jump_Boost) — +2.0 b/s per level on JUMP_VEL
                    let jb = vc_gameplay::effects::jump_boost_bonus(&self.effects);
                    let base = if feet_on_honey {
                        JUMP_VEL * HONEY_JUMP_CUT
                    } else {
                        JUMP_VEL
                    };
                    self.vel.y = base + jb;""",
    ),
    # the autojump
    (
        "autojump",
        """            if blocked && step_clear {
                self.vel.y = JUMP_VEL;""",
        """            if blocked && step_clear {
                // the completeness audit: Jump Boost rides the autojump
                // too (the same launch, VERIFIED w/Effect §Jump_Boost)
                let jb = vc_gameplay::effects::jump_boost_bonus(&self.effects);
                self.vel.y = JUMP_VEL + jb;""",
    ),
])

print("FAILED:" if fail else "gen.rs + player.rs patched")
for f in fail:
    print("  -", f)
sys.exit(1 if fail else 0)
