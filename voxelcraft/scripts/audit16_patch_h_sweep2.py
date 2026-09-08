#!/usr/bin/env python3
"""Sweep-2 patch for the 1.0-1.16.5 completeness audit — the food gaps.

The independent recheck found five food-side gaps (all values live-verified
2026-09-09 against the fresh captures scripts/audit16_page_{Rotten_Flesh,
Spider_Eye,Chorus_Fruit,Golden_Apple,Melon_Slice,Melon}.json):

- ROTTEN_FLESH has healed 4.0 since Phase 2 (the raw-meat default) — the
  Food table says hunger 4 -> 2.0 HP + "Hunger (0:30) (80% chance)"
- SPIDER_EYE exists (spider drops, trades, the fermented craft) but was
  never edible — hunger 2 -> 1.0 HP + "Poison (0:05)"
- CHORUS_FRUIT exists (the popped-fruit source) but was never edible —
  hunger 4 -> 2.0 HP + "up to 16 attempts ... within +-8 on all three
  axes" (the enderman-style teleport)
- GOLDEN_APPLE exists (the horse-breeding food) but was never edible —
  hunger 4 -> 2.0 HP + "Absorption (2:00)" + "Regeneration II (0:05)"
- MELON_SLICE absent entirely — the 1.0 staple: hunger 2 -> 1.0 HP, the
  melon block "drops 3-7 melon slices", 9 slices craft the block back

Registry: MELON_SLICE id 505, state V15_STATE_BASE+28 (804), tile 735.
"""
import sys

fail = []


def patch(path, edits, tag):
    src = open(path).read()
    for what, old, new in edits:
        if new in src:
            continue  # already applied (idempotent re-run)
        if old not in src or src.count(old) != 1:
            fail.append(f"[{tag}] ANCHOR ({what}): {old[:70]!r}")
            continue
        src = src.replace(old, new)
    open(path, "w").write(src)


# ======================================================================
# 1) blocks.rs — the MELON_SLICE registry row (V15 window + 1)
# ======================================================================
patch("crates/vc-blocks/src/blocks.rs", [
    (
        "MELON_SLICE const",
        """/// the silverfish spawn egg (kind 47 — stronghold spawner mob).
pub const SPAWN_EGG_SILVERFISH: u16 = 504;""",
        """/// the silverfish spawn egg (kind 47 — stronghold spawner mob).
pub const SPAWN_EGG_SILVERFISH: u16 = 504;
/// melon slice — "Restores 2 hunger" (VERIFIED w/Melon_Slice, live
/// 2026-09-09); the 1.0 staple food, dropped by breaking melons.
pub const MELON_SLICE: u16 = 505;""",
    ),
    (
        "V15_COUNT bump",
        "pub const V15_COUNT: u16 = 28;",
        "pub const V15_COUNT: u16 = 29;",
    ),
    (
        "V15_STATE_TO_BLOCK append",
        """    SPAWNER, // the cave-spider spawner state
    SPAWNER, // the silverfish spawner state
];""",
        """    SPAWNER, // the cave-spider spawner state
    SPAWNER, // the silverfish spawner state
    // the sweep-2 row: the melon slice identity state (804)
    MELON_SLICE,
];""",
    ),
    (
        "default_state arm",
        """        SPAWN_EGG_SILVERFISH => Some(V15_STATE_BASE + 25),""",
        """        SPAWN_EGG_SILVERFISH => Some(V15_STATE_BASE + 25),
        MELON_SLICE => Some(V15_STATE_BASE + 28),""",
    ),
    (
        "BLOCK_COUNT bump",
        "pub const BLOCK_COUNT: usize = 505; // the 1.0-1.16.5 completeness audit: V15 window ids 479..=504 (the cooked-meat family, the kitchen chain — apple/bowl/mushroom-rabbit-beetroot stews/sugar/egg/poisonous potato — popped chorus, ghast tear, the leaping/regeneration potions, and the ghast/cave-spider/silverfish spawn eggs)",
        "pub const BLOCK_COUNT: usize = 506; // the 1.0-1.16.5 completeness audit: V15 window ids 479..=505 (the cooked-meat family, the kitchen chain — apple/bowl/mushroom-rabbit-beetroot stews/sugar/egg/poisonous potato — popped chorus, ghast tear, the leaping/regeneration potions, the ghast/cave-spider/silverfish spawn eggs, and the sweep-2 melon slice)",
    ),
    (
        "STATE_COUNT bump",
        "pub const STATE_COUNT: usize = 804; // the completeness audit: V15 states 776..=803 (26 identity item states + the cave-spider/silverfish spawner states)",
        "pub const STATE_COUNT: usize = 805; // the completeness audit: V15 states 776..=804 (27 identity item states + the cave-spider/silverfish spawner states)",
    ),
    (
        "TILE_MELON_SLICE const",
        """/// the silverfish mob billboard sprite (audit16_art::silverfish_art).
pub const TILE_MOB_SILVERFISH: u16 = 734;""",
        """/// the silverfish mob billboard sprite (audit16_art::silverfish_art).
pub const TILE_MOB_SILVERFISH: u16 = 734;
/// the melon slice item sprite (audit16_art::melon_slice_art) — the
/// sweep-2 food row.
pub const TILE_MELON_SLICE: u16 = 735;""",
    ),
    (
        "TILE_MAX bump",
        "pub const TILE_MAX: u16 = 734; // the completeness audit: tiles 706..=734 (the V15 window — the cooked meats, the kitchen items, the leaping/regen potions, the ghast/cave-spider/silverfish eggs + mob sprites)",
        "pub const TILE_MAX: u16 = 735; // the completeness audit: tiles 706..=735 (the V15 window — the cooked meats, the kitchen items, the leaping/regen potions, the ghast/cave-spider/silverfish eggs + mob sprites, the sweep-2 melon slice)",
    ),
    (
        "is_item_block row",
        """            | SUGAR
            | EGG
            | POISONOUS_POTATO""",
        """            | SUGAR
            | EGG
            | MELON_SLICE
            | POISONOUS_POTATO""",
    ),
    (
        "BLOCK_TABLE row",
        """    d("Silverfish Spawn Egg", [TILE_SPAWN_EGG_SILVERFISH, TILE_SPAWN_EGG_SILVERFISH, TILE_SPAWN_EGG_SILVERFISH], false, false, true, false, 0, SoundFamily::Grass),
];""",
        """    d("Silverfish Spawn Egg", [TILE_SPAWN_EGG_SILVERFISH, TILE_SPAWN_EGG_SILVERFISH, TILE_SPAWN_EGG_SILVERFISH], false, false, true, false, 0, SoundFamily::Grass),
    d("Melon Slice", [TILE_MELON_SLICE, TILE_MELON_SLICE, TILE_MELON_SLICE], false, false, true, false, 0, SoundFamily::Grass),
];""",
    ),
    (
        "CREATIVE list row",
        """    POISONOUS_POTATO, POPPED_CHORUS_FRUIT, GHAST_TEAR,""",
        """    MELON_SLICE,
    POISONOUS_POTATO, POPPED_CHORUS_FRUIT, GHAST_TEAR,""",
    ),
    (
        "registry-window test rows",
        """            (SPAWN_EGG_SILVERFISH, V15_STATE_BASE + 25),
        ] {""",
        """            (SPAWN_EGG_SILVERFISH, V15_STATE_BASE + 25),
            (MELON_SLICE, V15_STATE_BASE + 28),
        ] {""",
    ),
], "blocks")

# ======================================================================
# 2) audit16_art.rs — the melon slice sprite
# ======================================================================
ART = open("crates/vc-render/src/textures/audit16_art.rs").read()
if "melon_slice_art" not in ART:
    # append before the final closing brace of the file
    i = ART.rstrip().rfind("}")
    if i < 0:
        fail.append("[art] no closing brace")
    else:
        fn = '''
/// the melon slice — the sweep-2 food row (w/Melon_Slice: "Restores 2
/// hunger"; the 1.0 staple). A wedge: green rind arc on the outer edge,
/// red flesh body, a few pale seeds.
pub fn melon_slice_art(a: &mut [u8], t: usize, rng: &mut Rng) {
    let (x0, y0) = (t % 32 * 16, t / 32 * 16);
    let put = |a: &mut [u8], x: i32, y: i32, c: [u8; 4]| {
        if (0..16).contains(&x) && (0..16).contains(&y) {
            let i = ((y0 as i32 + y) as usize) * 512 * 4 + ((x0 as i32 + x) as usize) * 4;
            a[i] = c[0];
            a[i + 1] = c[1];
            a[i + 2] = c[2];
            a[i + 3] = c[3];
        }
    };
    // rind greens (light + dark) and flesh reds
    let rind = [46, 120, 40, 255];
    let rind_d = [30, 90, 24, 255];
    let flesh = [206, 46, 46, 255];
    let flesh_d = [178, 34, 34, 255];
    let seed = [240, 235, 200, 255];
    for y in 0..16i32 {
        for x in 0..16i32 {
            // the wedge: lower-left triangle body (the diagonal cut
            // from top-left to bottom-right, rind along the hypotenuse
            // edge — the top-right corner stays transparent)
            let in_flesh = x + y >= 4 && x + y <= 22 && x <= 12 && y <= 12
                && (x + y) >= 6 - (x.min(y) / 6);
            if in_flesh {
                let c = if (x + y) % 7 == 0 { flesh_d } else { flesh };
                put(a, x, y, c);
            }
        }
    }
    // the rind: the arc band along the lower-left edge (x+y in 2..=5)
    for y in 0..16i32 {
        for x in 0..16i32 {
            let s = x + y;
            if s >= 2 && s <= 5 && x <= 13 && y <= 13 {
                let c = if s == 2 { rind_d } else { rind };
                put(a, x, y, c);
            }
        }
    }
    // seeds: three pale dots in the flesh (deterministic layout)
    for (sx, sy) in [(6, 4), (9, 6), (4, 8)] {
        put(a, sx, sy, seed);
        put(a, sx, sy + 1, [220, 214, 178, 255]);
    }
    let _ = rng; // jitter reserved (the deterministic wedge reads cleaner)
}
'''
        ART = ART[:i] + fn + ART[i:]
    open("crates/vc-render/src/textures/audit16_art.rs", "w").write(ART)

# ======================================================================
# 3) textures.rs — the atlas arm
# ======================================================================
patch("crates/vc-render/src/textures.rs", [
    (
        "atlas arm",
        """            TILE_MOB_SILVERFISH => audit16_art::silverfish_art(&mut a, t, &mut rng),
            _ => {}""",
        """            TILE_MOB_SILVERFISH => audit16_art::silverfish_art(&mut a, t, &mut rng),
            // the sweep-2 food row: the melon slice
            TILE_MELON_SLICE => audit16_art::melon_slice_art(&mut a, t, &mut rng),
            _ => {}""",
    ),
], "textures")

# ======================================================================
# 4) gpu_mesh.rs — the LUT offsets + clamps resync (805/506)
# ======================================================================
patch("crates/vc-render/src/gpu_mesh.rs", [
    (
        "LUT offsets",
        """const L_SB: u32 = 0u;              // lut: state -> block        (STATE_COUNT=804)
const L_FL: u32 = 804u;            // lut: block flags           (BLOCK_COUNT=505)
const L_TC: u32 = 1309u;           // lut: block tint class      (BLOCK_COUNT=505)
const L_ST: u32 = 1814u;           // lut: state tiles, 4/state  (4·STATE_COUNT=3216)""",
        """const L_SB: u32 = 0u;              // lut: state -> block        (STATE_COUNT=805)
const L_FL: u32 = 805u;            // lut: block flags           (BLOCK_COUNT=506)
const L_TC: u32 = 1311u;           // lut: block tint class      (BLOCK_COUNT=506)
const L_ST: u32 = 1817u;           // lut: state tiles, 4/state  (4·STATE_COUNT=3220)""",
    ),
    (
        "sb/fl clamps",
        """fn sb(s: u32) -> u32 { return lut[L_SB + min(s, 803u)]; }
fn fl(b: u32) -> u32 { return lut[L_FL + min(b, 504u)]; }""",
        """fn sb(s: u32) -> u32 { return lut[L_SB + min(s, 804u)]; }
fn fl(b: u32) -> u32 { return lut[L_FL + min(b, 505u)]; }""",
    ),
    (
        "tint clamp",
        "let tc = lut[L_TC + min(b, 504u)];",
        "let tc = lut[L_TC + min(b, 505u)];",
    ),
], "gpu_mesh")

# ======================================================================
# 5) craft.rs — the melon crafts (kitchen matcher extension)
# ======================================================================
patch("crates/vc-gameplay/src/craft.rs", [
    (
        "counter decl",
        """    let mut sugar = 0;
    let mut egg = 0;
    let mut honey = 0;
    let mut other = 0;""",
        """    let mut sugar = 0;
    let mut egg = 0;
    let mut honey = 0;
    let mut melon_slice = 0;
    let mut other = 0;""",
    ),
    (
        "counter arm",
        """            EGG if !s.is_empty() => egg += 1,
            HONEY_BOTTLE if !s.is_empty() => honey += 1,""",
        """            EGG if !s.is_empty() => egg += 1,
            HONEY_BOTTLE if !s.is_empty() => honey += 1,
            MELON_SLICE if !s.is_empty() => melon_slice += 1,""",
    ),
    (
        "total",
        """    let total = planks + bowl + red + brown + carrot + baked + rabbit
        + beetroot + pumpkin + sugar + egg + honey;""",
        """    let total = planks + bowl + red + brown + carrot + baked + rabbit
        + beetroot + pumpkin + sugar + egg + honey + melon_slice;""",
    ),
    (
        "existing rows guard fix (bowl row)",
        """    if planks == 3
        && bowl == 0 && red == 0 && brown == 0 && carrot == 0 && baked == 0
        && rabbit == 0 && beetroot == 0 && pumpkin == 0 && sugar == 0
        && egg == 0 && honey == 0
    {
        return Some(ItemStack::new(BOWL, 4));
    }""",
        """    if planks == 3
        && bowl == 0 && red == 0 && brown == 0 && carrot == 0 && baked == 0
        && rabbit == 0 && beetroot == 0 && pumpkin == 0 && sugar == 0
        && egg == 0 && honey == 0 && melon_slice == 0
    {
        return Some(ItemStack::new(BOWL, 4));
    }""",
    ),
    (
        "new rows",
        """    // pumpkin pie: 1 pumpkin + 1 sugar + 1 egg
    if pumpkin == 1 && sugar == 1 && egg == 1 && total == 3 {
        return Some(ItemStack::new(PUMPKIN_PIE, 1));
    }
    None
}""",
        """    // pumpkin pie: 1 pumpkin + 1 sugar + 1 egg
    if pumpkin == 1 && sugar == 1 && egg == 1 && total == 3 {
        return Some(ItemStack::new(PUMPKIN_PIE, 1));
    }
    // ---- the sweep-2 melon rows (VERIFIED w/Melon_Slice §Crafting,
    // live 2026-09-09: "Melon | Melon Slice" (the 3x3 nine-slice
    // recipe) and "Melon Seeds | Melon Slice" (1:1)) ----
    // the melon block: exactly 9 slices (the 3x3 grid)
    if melon_slice == 9 && total == 9 {
        return Some(ItemStack::new(MELON, 1));
    }
    // melon seeds: exactly 1 slice
    if melon_slice == 1 && total == 1 {
        return Some(ItemStack::new(MELON_SEEDS, 1));
    }
    None
}""",
    ),
], "craft")

if fail:
    print("PATCH FAILURES:")
    for f in fail:
        print("  -", f)
    sys.exit(1)
print("patch h applied cleanly: registry + art + craft")
