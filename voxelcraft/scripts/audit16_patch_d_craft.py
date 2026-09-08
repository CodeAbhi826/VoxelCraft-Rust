#!/usr/bin/env python3
"""craft.rs patch — the audit-round kitchen chain + purpur/end-rod crafts."""
import sys

PATH = "crates/vc-gameplay/src/craft.rs"
src = open(PATH).read()
fail = []

def sub_once(old, new, what):
    global src
    if old not in src or src.count(old) != 1:
        fail.append(f"ANCHOR ({what}): {old[:70]!r}")
        return
    src = src.replace(old, new)

# 1) Ing::AnyPlanks — extend to all four planks (the audit fix: vanilla's
#    "Any Planks" covers every species; the engine's oak+jungle pair was
#    the 1.14-era set)
sub_once(
    """                        Ing::AnyPlanks => {
                            !s.is_empty() && matches!(s.block, PLANKS | JUNGLE_PLANKS)
                        }""",
    """                        Ing::AnyPlanks => {
                            // the completeness audit: vanilla's "Any
                            // Planks" covers every species — the 1.14-era
                            // oak+jungle pair extended to the 1.16 woods
                            !s.is_empty()
                                && matches!(
                                    s.block,
                                    PLANKS | JUNGLE_PLANKS | CRIMSON_PLANKS | WARPED_PLANKS
                                )
                        }""",
    "AnyPlanks matcher",
)
sub_once(
    """    /// 1.14: any planks (vanilla's stick/barrel recipes take "Any
    /// Planks" — the engine's oak + jungle pair)
    AnyPlanks,""",
    """    /// 1.14: any planks (vanilla's stick/barrel recipes take "Any
    /// Planks" — the completeness audit extends the oak+jungle pair to
    /// the 1.16 crimson/warped planks, vanilla-exact)
    AnyPlanks,""",
    "AnyPlanks doc",
)

# 2) the shaped recipes — purpur + end rod (appended at the RECIPES tail,
#    before the closing `];` that precedes match_soul_torch)
sub_once(
    """    // ingredient table: "Soul Lantern — Iron Nugget + Soul Torch").
];""",
    """    // ingredient table: "Soul Lantern — Iron Nugget + Soul Torch").
    // ---- the 1.0-1.16.5 completeness audit (VERIFIED live 2026-09-08
    // against the audit16 captures) ----
    // purpur block: 4 popped chorus fruit -> 4 (the 2x2 stone-family
    // pattern; w/Popped_Chorus_Fruit: "used to craft End rods and
    // purpur blocks" — the 1.9 purpur family finally crafts from its
    // own ingredient instead of being picker-only)
    Recipe {
        size: 2,
        grid: &[
            Ing::Block(POPPED_CHORUS_FRUIT), Ing::Block(POPPED_CHORUS_FRUIT),
            Ing::Block(POPPED_CHORUS_FRUIT), Ing::Block(POPPED_CHORUS_FRUIT),
        ],
        out: ItemStack::new(PURPUR_BLOCK, 4),
    },
    // end rod: blaze rod + popped chorus fruit -> 4 (VERIFIED
    // w/End_Rod §Crafting; the 1.9 end-rod's first recipe)
    Recipe {
        size: 2,
        grid: &[
            Ing::Block(BLAZE_ROD),
            Ing::Block(POPPED_CHORUS_FRUIT),
        ],
        out: ItemStack::new(END_ROD, 4),
    },
];""",
    "shaped recipes tail",
)

# 3) the shapeless KITCHEN matcher — after match_soul_torch's body
sub_once(
    """fn match_soul_torch(slots: &[ItemStack], _size: usize) -> Option<ItemStack> {""",
    """/// the completeness audit: the shapeless KITCHEN chain (all VERIFIED
/// live 2026-09-08 against the audit16 captures — the Bowl, Sugar,
/// Mushroom_Stew, Rabbit_Stew, Beetroot_Soup, Pumpkin_Pie pages):
/// - bowl: 3 "Any Planks" -> 4 (w/Bowl §Crafting: "Any Planks" -> 4)
/// - sugar: 1 honey bottle -> 3 (the 1.15 craft; the empty-bottle
///   grid-return is a no-container-return engine trim, disclosed —
///   the drink path returns its bottle)
/// - mushroom stew: 1 red + 1 brown + 1 bowl -> 1 (w/Mushroom_Stew
///   §Crafting: "Red Mushroom + Brown Mushroom + Bowl")
/// - rabbit stew: 1 cooked rabbit + 1 carrot + 1 baked potato + 1
///   red-OR-brown mushroom + 1 bowl -> 1 (w/Rabbit_Stew §Crafting)
/// - beetroot soup: 6 beetroot + 1 bowl -> 1 (w/Beetroot_Soup
///   §Crafting: "Beetroot + Bowl", the 6-root set)
/// - pumpkin pie: 1 pumpkin + 1 sugar + 1 egg -> 1 (w/Pumpkin_Pie
///   §Crafting: "Pumpkin + Sugar + Any Egg")
fn match_kitchen(slots: &[ItemStack], _size: usize) -> Option<ItemStack> {
    let mut planks = 0;
    let mut bowl = 0;
    let mut red = 0;
    let mut brown = 0;
    let mut carrot = 0;
    let mut baked = 0;
    let mut rabbit = 0;
    let mut beetroot = 0;
    let mut pumpkin = 0;
    let mut sugar = 0;
    let mut egg = 0;
    let mut honey = 0;
    let mut other = 0;
    for s in slots {
        match s.block {
            PLANKS | JUNGLE_PLANKS | CRIMSON_PLANKS | WARPED_PLANKS if !s.is_empty() => planks += 1,
            BOWL if !s.is_empty() => bowl += 1,
            MUSHROOM_RED if !s.is_empty() => red += 1,
            MUSHROOM_BROWN if !s.is_empty() => brown += 1,
            CARROT if !s.is_empty() => carrot += 1,
            BAKED_POTATO if !s.is_empty() => baked += 1,
            COOKED_RABBIT if !s.is_empty() => rabbit += 1,
            BEETROOT if !s.is_empty() => beetroot += 1,
            PUMPKIN if !s.is_empty() => pumpkin += 1,
            SUGAR if !s.is_empty() => sugar += 1,
            EGG if !s.is_empty() => egg += 1,
            HONEY_BOTTLE if !s.is_empty() => honey += 1,
            _ if !s.is_empty() => other += 1,
            _ => {}
        }
    }
    if other > 0 {
        return None;
    }
    let total = planks + bowl + red + brown + carrot + baked + rabbit
        + beetroot + pumpkin + sugar + egg + honey;
    // bowl: exactly 3 planks (the V-shape's 3 items, shapeless)
    if planks == 3
        && bowl == 0 && red == 0 && brown == 0 && carrot == 0 && baked == 0
        && rabbit == 0 && beetroot == 0 && pumpkin == 0 && sugar == 0
        && egg == 0 && honey == 0
    {
        return Some(ItemStack::new(BOWL, 4));
    }
    // sugar: exactly 1 honey bottle
    if honey == 1 && total == 1 {
        return Some(ItemStack::new(SUGAR, 3));
    }
    // mushroom stew: 1 red + 1 brown + 1 bowl
    if red == 1 && brown == 1 && bowl == 1 && total == 3 {
        return Some(ItemStack::new(MUSHROOM_STEW, 1));
    }
    // rabbit stew: 1 cooked rabbit + 1 carrot + 1 baked potato +
    // exactly one mushroom (red or brown) + 1 bowl
    if rabbit == 1
        && carrot == 1
        && baked == 1
        && bowl == 1
        && red + brown == 1
        && total == 5
    {
        return Some(ItemStack::new(RABBIT_STEW, 1));
    }
    // beetroot soup: 6 beetroot + 1 bowl
    if beetroot == 6 && bowl == 1 && total == 7 {
        return Some(ItemStack::new(BEETROOT_SOUP, 1));
    }
    // pumpkin pie: 1 pumpkin + 1 sugar + 1 egg
    if pumpkin == 1 && sugar == 1 && egg == 1 && total == 3 {
        return Some(ItemStack::new(PUMPKIN_PIE, 1));
    }
    None
}

fn match_soul_torch(slots: &[ItemStack], _size: usize) -> Option<ItemStack> {""",
    "kitchen matcher",
)

# 4) dispatch — before the soul-torch dispatch
sub_once(
    """pub fn match_grid(slots: &[ItemStack], size: usize) -> Option<ItemStack> {
    // 1.16 (Nether Update, part 2): the shapeless soul-torch +
    // soul-lantern recipes (any arrangement)
    if let Some(out) = match_soul_torch(slots, size) {
        return Some(out);
    }""",
    """pub fn match_grid(slots: &[ItemStack], size: usize) -> Option<ItemStack> {
    // the completeness audit: the shapeless kitchen chain (the bowl /
    // sugar / stew / pie recipes, any arrangement)
    if let Some(out) = match_kitchen(slots, size) {
        return Some(out);
    }
    // 1.16 (Nether Update, part 2): the shapeless soul-torch +
    // soul-lantern recipes (any arrangement)
    if let Some(out) = match_soul_torch(slots, size) {
        return Some(out);
    }""",
    "kitchen dispatch",
)

if fail:
    print("FAILED ANCHORS:")
    for f in fail:
        print("  -", f)
    sys.exit(1)
open(PATH, "w").write(src)
print(f"craft.rs patched OK ({len(src)} chars)")
