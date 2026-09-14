//! GUI art — procedural HUD sprites + widget chrome (UI-overhaul Phase 1).
//!
//! Clean-room, hand-drawn from scratch: every sprite here is authored in
//! this file as a mask table and painted into a caller-provided RGBA
//! buffer at boot (G9 — the repo's startup-asset convention: procedural
//! generation in code, never files on disk). No Mojang asset is shipped
//! or referenced; only the *dimensions* and layout constants follow the
//! public wiki numbers, each tagged `// VERIFIED <url>`.
//!
//! Sprite dimensions (wiki-verified, painted at true texture size and
//! rendered by the quad pass at the engine's 2x UI scale):
//!   heart / hunger / armor / bubble  9x9   (rendered 18x18)
//!   button (normal/hover/disabled)  20x20, 9-slice, 4-px corners
//!   slot (empty/hover)              18x18
//!   panel                           20x20, 9-slice, 4-px corners
//!   hotbar background               182x22
//!   options dirt tile               16x16
//!
//! Repo conventions honored: tests live in this same file (G3); no
//! `unwrap()`/`expect()` in the painting code (G2); the coverage guard
//! below follows the `vNNN_tiles_all_painted` pattern (G5) — a blank
//! variant is a missing painter.

// ---------------------------------------------------------------- types --

/// Heart sprite variants (9x9). Empty = outline shell, Full = filled,
/// Half = left half filled over the empty shell (vanilla draws the
/// outline pass first, then the fill on top — the half variant bakes
/// both into one sprite).
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum HeartVariant {
    Empty,
    Full,
    Half,
}

/// Hunger (drumstick) sprite variants (9x9).
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum HungerVariant {
    Empty,
    Full,
    Half,
}

/// Armor (chestplate) sprite variants (9x9).
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum ArmorVariant {
    Empty,
    Full,
    Half,
}

/// Air-bubble sprite variants (9x9).
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum BubbleVariant {
    Full,
    Gone,
}

/// Widget chrome variants. Buttons and the panel are 9-slice sources
/// (20x20 with 4-px corners — the quad renderer stretches the middle);
/// slots are fixed 18x18 cells.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum WidgetVariant {
    ButtonNormal,
    ButtonHover,
    ButtonDisabled,
    SlotEmpty,
    SlotHover,
    Panel,
}

// ------------------------------------------------------------- helpers --

/// RGBA pixel type alias (matches ui.rs's `Color`).
type Px = [u8; 4];

/// Blend `over` onto `base` with the alpha of `over` (source-over).
/// Used only for the baked hover lightening; the regular path paints
/// opaque pixels straight through.
fn blend_over(base: Px, over: Px) -> Px {
    let a = over[3] as u32;
    if a == 0 {
        return base;
    }
    if a == 255 {
        return over;
    }
    let inv = 255 - a;
    [
        ((base[0] as u32 * inv + over[0] as u32 * a) / 255) as u8,
        ((base[1] as u32 * inv + over[1] as u32 * a) / 255) as u8,
        ((base[2] as u32 * inv + over[2] as u32 * a) / 255) as u8,
        base[3].max(over[3]),
    ]
}

/// Paint a mask of string rows into `out` (row stride `w` RGBA pixels).
/// Chars look up `pal`; '.' and ' ' stay transparent. Clips every write
/// to the buffer — a short buffer paints a clipped sprite, never panics.
fn paint_mask(out: &mut [u8], w: usize, rows: &[&str], pal: &[(char, Px)]) {
    let h = out.len() / (w.max(1) * 4);
    for (ry, row) in rows.iter().enumerate() {
        if ry >= h {
            break;
        }
        for (rx, ch) in row.chars().enumerate() {
            if ch == '.' || ch == ' ' {
                continue;
            }
            let Some((_, col)) = pal.iter().find(|(c, _)| *c == ch) else {
                continue;
            };
            let idx = ry * w + rx;
            if idx < out.len() / 4 {
                let o = idx * 4;
                out[o] = col[0];
                out[o + 1] = col[1];
                out[o + 2] = col[2];
                out[o + 3] = col[3];
            }
        }
    }
}

/// Fill a rectangle in `out` (stride `w`) with a flat color.
fn fill_rect(out: &mut [u8], w: usize, x: usize, y: usize, rw: usize, rh: usize, c: Px) {
    let h = out.len() / (w.max(1) * 4);
    for yy in y..(y + rh).min(h) {
        for xx in x..(x + rw).min(w) {
            let o = (yy * w + xx) * 4;
            if o + 3 < out.len() {
                out[o] = c[0];
                out[o + 1] = c[1];
                out[o + 2] = c[2];
                out[o + 3] = c[3];
            }
        }
    }
}

// ------------------------------------------------------- HUD 9x9 masks --
// CLEAN-ROOM — hand-drawn from scratch (silhouettes are the generic
// pixel-art heart/drumstick/chestplate/bubble shapes; pixel data and
// palette values are original).

// CLEAN-ROOM — hand-drawn from scratch
const HEART_MASK: [&str; 9] = [
    ".OO...OO.",
    "OAFO.OFFO",
    "OFFFOFFFO",
    "OFFFFFFFO",
    "OOFFFFFFO",
    ".OFFFFFO.",
    "..OFFFO..",
    "...OFO...",
    "....O....",
];

// the empty shell: outline + dark interior (drawn UNDER the fill in
// vanilla — here baked as one sprite)
// CLEAN-ROOM — hand-drawn from scratch
const HEART_EMPTY_MASK: [&str; 9] = [
    ".OO...OO.",
    "ODDO.ODDO",
    "ODDDODDDO",
    "ODDDDDDDO",
    "OODDDDDDO",
    ".ODDDDDO.",
    "..ODDDO..",
    "...ODO...",
    "....O....",
];

// half = empty shell with the LEFT half filled (vanilla halves fill
// from the left, mirrored for right-aligned rows)
// CLEAN-ROOM — hand-drawn from scratch
const HEART_HALF_MASK: [&str; 9] = [
    ".OO...OO.",
    "OAFO.ODDO",
    "OFFFODDDO",
    "OFFFDDDDO",
    "OOFFDDDDO",
    ".OFFDDDO.",
    "..OFFDO..",
    "...OFO...",
    "....O....",
];

// CLEAN-ROOM — hand-drawn from scratch. v2 (the Luanti-replication
// round): teardrop meat blob top-right tapering into a white bone
// shaft down-left with a knob end — the drumstick silhouette VLM-
// validated at 16x zoom (the v1 oval read as a "potato").
const HUNGER_MASK: [&str; 9] = [
    "....OOOO.",
    "...OMMMMO",
    "..OMHMMMO",
    "..OMMMMMO",
    "...OMMMO.",
    "..OWOMMO.",
    ".OWWOOO..",
    "OWWO.....",
    "OOO......",
];

// CLEAN-ROOM — hand-drawn from scratch (same silhouette, dark shell)
const HUNGER_EMPTY_MASK: [&str; 9] = [
    "....OOOO.",
    "...ODDDDO",
    "..ODDDDDO",
    "..ODDDDDO",
    "...ODDDO.",
    "..ODODDO.",
    ".ODDOOO..",
    "ODDO.....",
    "OOO......",
];

// half hunger: right half of the meat filled (vanilla hunger halves
// empty from the left of the icon as it drains)
// CLEAN-ROOM — hand-drawn from scratch
const HUNGER_HALF_MASK: [&str; 9] = [
    "....OOOO.",
    "...ODDDDO",
    "..ODHDDDO",
    "..ODDDDDO",
    "...ODDDO.",
    "..ODODDO.",
    ".ODDOOO..",
    "ODDO.....",
    "OOO......",
];

// CLEAN-ROOM — hand-drawn from scratch
const ARMOR_MASK: [&str; 9] = [
    "OO.....OO",
    "OAOOOOOAO",
    "OAFFFFFAO",
    "OAFFFFFFO",
    ".OAFFFFAO",
    ".OFFFFFO.",
    ".OFFFFFO.",
    "..OFFFO..",
    "...OOO...",
];

// CLEAN-ROOM — hand-drawn from scratch
const ARMOR_EMPTY_MASK: [&str; 9] = [
    "OO.....OO",
    "ODOOOOODO",
    "ODDDDDDDO",
    "ODDDDDDDO",
    ".ODDDDDO.",
    ".ODDDDDO.",
    ".ODDDDDO.",
    "..ODDDO..",
    "...OOO...",
];

// half armor: left shoulder+torso filled
// CLEAN-ROOM — hand-drawn from scratch
const ARMOR_HALF_MASK: [&str; 9] = [
    "OO.....OO",
    "OAOOOOODO",
    "OAFFDDDDO",
    "OAFFDDDDO",
    ".OFFDDDO.",
    ".OFFDDDO.",
    ".OFFDDDO.",
    "..OFFDO..",
    "...OOO...",
];

// CLEAN-ROOM — hand-drawn from scratch
const BUBBLE_MASK: [&str; 9] = [
    "..OOOO...",
    ".OWWOOO..",
    "OWWBBBOO.",
    "OWBBBBBO.",
    "OBBBBBBO.",
    "OBBBBBO..",
    ".OBBOO...",
    "..OOO....",
    ".........",
];

// the pop/burst frame (vanilla shows the popping animation frame where
// the bubble breaks — one droplet ring)
// CLEAN-ROOM — hand-drawn from scratch
const BUBBLE_GONE_MASK: [&str; 9] = [
    "..O.O....",
    ".O...O...",
    "..O.O....",
    ".........",
    "..O.O....",
    ".O...O...",
    "..O.O....",
    ".........",
    ".........",
];

// ------------------------------------------------ effect-icon 9x9 masks --
// Sub-round 1 (2026-09-14 Survival HUD round): the status-effect icon
// set — one 9x9 tile per engine effect kind (16 kinds, index order =
// vc_gameplay::effects::EffectKind's declaration order; the index
// mapping lives at the game layer so vc-render stays crate-independent).
//
// CLEAN-ROOM — hand-drawn from scratch. Silhouettes are generic pixel-art
// glyphs (droplet / bolt / arrow / heart / shield / fish / feather /
// spiral…), one per effect MEANING, colored with each effect's
// wiki-published color family (facts, not assets). No Mojang sprite was
// read, copied, or traced; vanilla's actual inventory_effect icons are
// different art at a different size (24x24).

/// number of effect-icon tiles in the sheet (16 kinds × 9x9 = 144x9)
pub const EFFECT_ICON_COUNT: usize = 16;

// CLEAN-ROOM — hand-drawn from scratch (wither: ash-grey skull)
const EFF_WITHER: [&str; 9] = [
    "..OOOO...",
    ".OFFFFO..",
    ".OFAAFO..",
    ".OFFFFO..",
    "..OFFO...",
    "..O.O....",
    ".O.O.O...",
    ".O.....O.",
    ".........",
];
const PAL_WITHER: [(char, Px); 3] = [
    ('O', [30, 30, 30, 255]),
    ('F', [85, 75, 75, 255]),
    ('A', [140, 125, 125, 255]),
];

// CLEAN-ROOM — hand-drawn from scratch (poison: green droplet)
const EFF_POISON: [&str; 9] = [
    "....O....",
    "...OFO...",
    "...OFO...",
    "..OFFFO..",
    ".OFFAFFO.",
    ".OFFFFFO.",
    ".OFFFFFO.",
    "..OFFFO..",
    "...OOO...",
];
const PAL_POISON: [(char, Px); 3] = [
    ('O', [22, 62, 22, 255]),
    ('F', [72, 160, 60, 255]),
    ('A', [185, 255, 170, 255]),
];

// CLEAN-ROOM — hand-drawn from scratch (regeneration: magenta heart)
const EFF_REGEN: [&str; 9] = [
    ".OO...OO.",
    "OAFO.OFFO",
    "OFFFOFFFO",
    "OFFFFFFFO",
    "OOFFFFFFO",
    ".OFFFFFO.",
    "..OFFFO..",
    "...OFO...",
    "....O....",
];
const PAL_REGEN: [(char, Px); 3] = [
    ('O', [84, 12, 62, 255]),
    ('F', [238, 82, 198, 255]),
    ('A', [255, 172, 232, 255]),
];

// CLEAN-ROOM — hand-drawn from scratch (speed: twin cyan chevrons)
const EFF_SPEED: [&str; 9] = [
    ".........",
    ".O....O..",
    ".OO..OO..",
    ".OFO.OFO.",
    ".OFO.OFO.",
    ".OO..OO..",
    ".O....O..",
    ".........",
    ".........",
];
const PAL_SPEED: [(char, Px); 2] = [
    ('O', [18, 80, 100, 255]),
    ('F', [120, 220, 255, 255]),
];

// CLEAN-ROOM — hand-drawn from scratch (haste: yellow lightning bolt)
const EFF_HASTE: [&str; 9] = [
    "....OOOO.",
    "...OFFFO.",
    "..OFFFO..",
    ".OOOOOOO.",
    "...OFFO..",
    "..OFFO...",
    "..OFO....",
    "..OO.....",
    "..O......",
];
const PAL_HASTE: [(char, Px); 2] = [
    ('O', [96, 74, 10, 255]),
    ('F', [250, 220, 82, 255]),
];

// CLEAN-ROOM — hand-drawn from scratch (resistance: grey round shield)
const EFF_RESIST: [&str; 9] = [
    "..OOOOO..",
    ".OFFFFFO.",
    ".OFAAFFO.",
    ".OFAAFFO.",
    ".OFFFFFO.",
    "..OOOOO..",
    "...OAO...",
    ".........",
    ".........",
];
const PAL_RESIST: [(char, Px); 3] = [
    ('O', [52, 52, 52, 255]),
    ('F', [142, 142, 142, 255]),
    ('A', [222, 222, 222, 255]),
];

// CLEAN-ROOM — hand-drawn from scratch (jump boost: light-blue up arrow)
const EFF_JUMP: [&str; 9] = [
    "....O....",
    "...OFO...",
    "..OFFFO..",
    ".OFFFFFO.",
    "OOOFAFOOO",
    "..OFO....",
    "..OFO....",
    "..OOO....",
    ".........",
];
const PAL_JUMP: [(char, Px); 3] = [
    ('O', [20, 62, 112, 255]),
    ('F', [122, 190, 255, 255]),
    ('A', [228, 244, 255, 255]),
];

// CLEAN-ROOM — hand-drawn from scratch (strength: red up sword)
const EFF_STRENGTH: [&str; 9] = [
    "....O....",
    "...OFO...",
    "...OFO...",
    "...OFO...",
    ".OOOFOOO.",
    "..OOFOO..",
    "...OOO...",
    "...OAO...",
    ".........",
];
const PAL_STRENGTH: [(char, Px); 3] = [
    ('O', [92, 16, 16, 255]),
    ('F', [222, 62, 62, 255]),
    ('A', [255, 152, 152, 255]),
];

// CLEAN-ROOM — hand-drawn from scratch (slowness: muddy down arrow)
const EFF_SLOWNESS: [&str; 9] = [
    ".........",
    "..OOO....",
    "..OFO....",
    "..OFO....",
    "OOOFAFOOO",
    ".OFFFFFO.",
    "..OFFFO..",
    "...OFO...",
    "....O....",
];
const PAL_SLOWNESS: [(char, Px); 3] = [
    ('O', [62, 46, 26, 255]),
    ('F', [150, 112, 70, 255]),
    ('A', [202, 172, 124, 255]),
];

// CLEAN-ROOM — hand-drawn from scratch (hunger effect: desaturated
// yellow-green drumstick — w/Hunger_(effect), live 2026-09-14: "It also
// turns the hunger bar a yellow-green color")
const EFF_HUNGER: [&str; 9] = [
    "....OOOO.",
    "...OMMMMO",
    "..OMHMMMO",
    "..OMMMMMO",
    "...OMMMO.",
    "..OWOMMO.",
    ".OWWOOO..",
    "OWWO.....",
    "OOO......",
];
const PAL_HUNGER_EFF: [(char, Px); 4] = [
    ('O', [36, 42, 30, 255]),
    ('M', [150, 168, 96, 255]),
    ('H', [184, 202, 128, 255]),
    ('W', [222, 232, 204, 255]),
];

// CLEAN-ROOM — hand-drawn from scratch (absorption: golden heart)
const EFF_ABSORB: [&str; 9] = [
    ".OO...OO.",
    "OAFO.OFFO",
    "OFFFOFFFO",
    "OFFFFFFFO",
    "OOFFFFFFO",
    ".OFFFFFO.",
    "..OFFFO..",
    "...OFO...",
    "....O....",
];
const PAL_ABSORB: [(char, Px); 3] = [
    ('O', [102, 72, 10, 255]),
    ('F', [255, 220, 82, 255]),
    ('A', [255, 250, 204, 255]),
];

// CLEAN-ROOM — hand-drawn from scratch (blindness: dark smoke cloud)
const EFF_BLIND: [&str; 9] = [
    "..OOOO...",
    ".OFFFFO..",
    "OFAAAFFO.",
    "OFFFFFFO.",
    ".OOFFOO..",
    "..OOO....",
    ".........",
    ".........",
    ".........",
];
const PAL_BLIND: [(char, Px); 3] = [
    ('O', [26, 26, 26, 255]),
    ('F', [72, 72, 72, 255]),
    ('A', [8, 8, 8, 255]),
];

// CLEAN-ROOM — hand-drawn from scratch (water breathing: bubble + wave)
const EFF_WATER: [&str; 9] = [
    ".OO......",
    "OAAO.....",
    "OAAO.OO..",
    ".OO..OFO.",
    ".....OFO.",
    "..OOOOO..",
    ".OFFFFFO.",
    "..OOOOO..",
    ".........",
];
const PAL_WATER: [(char, Px); 3] = [
    ('O', [20, 52, 92, 255]),
    ('F', [72, 132, 222, 255]),
    ('A', [232, 246, 255, 255]),
];

// CLEAN-ROOM — hand-drawn from scratch (slow falling: white feather)
const EFF_FEATHER: [&str; 9] = [
    ".....O...",
    "....OFO..",
    "...OFFO..",
    "...OFFO..",
    "..OFFAO..",
    "..OFAO...",
    ".OFAO....",
    ".OO......",
    "O........",
];
const PAL_FEATHER: [(char, Px); 3] = [
    ('O', [84, 84, 84, 255]),
    ('F', [236, 236, 236, 255]),
    ('A', [182, 182, 182, 255]),
];

// CLEAN-ROOM — hand-drawn from scratch (conduit power: teal shell spiral)
const EFF_CONDUIT: [&str; 9] = [
    "..OOOO...",
    ".OFFFFO..",
    "OFFAFFO..",
    "OFAFFFO..",
    "OFFFOOO..",
    ".OFO.OFO.",
    "..O...O..",
    ".........",
    ".........",
];
const PAL_CONDUIT: [(char, Px); 3] = [
    ('O', [10, 62, 62, 255]),
    ('F', [42, 162, 162, 255]),
    ('A', [182, 255, 255, 255]),
];

// CLEAN-ROOM — hand-drawn from scratch (dolphin's grace: light-blue fish)
const EFF_DOLPHIN: [&str; 9] = [
    ".........",
    "..OOO..O.",
    ".OFFFO.OO",
    "OFAFFFFO.",
    ".OFFFO.OO",
    "..OOO..O.",
    ".........",
    ".........",
    ".........",
];
const PAL_DOLPHIN: [(char, Px); 3] = [
    ('O', [16, 56, 82, 255]),
    ('F', [92, 182, 232, 255]),
    ('A', [232, 246, 255, 255]),
];

/// one effect icon: (mask rows, palette) — the alias keeps the sheet
/// table's type readable (clippy type_complexity)
type EffectIcon = (&'static [&'static str], &'static [(char, Px)]);

/// the 16 effect-icon (mask, palette) pairs, index order matching
/// vc_gameplay::effects::EffectKind's declaration order
const EFFECT_ICONS: [EffectIcon; EFFECT_ICON_COUNT] = [
    (&EFF_WITHER, &PAL_WITHER),
    (&EFF_POISON, &PAL_POISON),
    (&EFF_REGEN, &PAL_REGEN),
    (&EFF_SPEED, &PAL_SPEED),
    (&EFF_HASTE, &PAL_HASTE),
    (&EFF_RESIST, &PAL_RESIST),
    (&EFF_JUMP, &PAL_JUMP),
    (&EFF_STRENGTH, &PAL_STRENGTH),
    (&EFF_SLOWNESS, &PAL_SLOWNESS),
    (&EFF_HUNGER, &PAL_HUNGER_EFF),
    (&EFF_ABSORB, &PAL_ABSORB),
    (&EFF_BLIND, &PAL_BLIND),
    (&EFF_WATER, &PAL_WATER),
    (&EFF_FEATHER, &PAL_FEATHER),
    (&EFF_CONDUIT, &PAL_CONDUIT),
    (&EFF_DOLPHIN, &PAL_DOLPHIN),
];

/// effect-icon registry names (matches vc_gameplay EffectKind order;
/// used by tests + debug dumps — never rendered on screen)
pub fn effect_icon_name(idx: usize) -> &'static str {
    const NAMES: [&str; EFFECT_ICON_COUNT] = [
        "wither",
        "poison",
        "regeneration",
        "speed",
        "haste",
        "resistance",
        "jump_boost",
        "strength",
        "slowness",
        "hunger",
        "absorption",
        "blindness",
        "water_breathing",
        "slow_falling",
        "conduit_power",
        "dolphins_grace",
    ];
    NAMES[idx.min(EFFECT_ICON_COUNT - 1)]
}

// ------------------------------------------------------------- palettes --

// CLEAN-ROOM — original palette values (carried over from the canvas
// HUD's documented clean-room colors so builtin quads match the old
// look-and-feel while the art is redrawn at true 9x9)
const HEART_PAL_FULL: [(char, Px); 4] = [
    ('O', [46, 6, 6, 255]),
    ('R', [227, 27, 13, 255]),
    ('A', [255, 116, 116, 255]),
    ('F', [227, 27, 13, 255]),
];
const HEART_PAL_SHELL: [(char, Px); 4] = [
    ('O', [46, 6, 6, 255]),
    ('D', [70, 70, 70, 200]),
    ('F', [227, 27, 13, 255]),
    ('A', [255, 116, 116, 255]),
];

const HUNGER_PAL_FULL: [(char, Px); 4] = [
    ('O', [43, 26, 4, 255]),
    ('M', [186, 106, 38, 255]),
    ('W', [231, 231, 231, 255]),
    ('H', [222, 160, 90, 255]),
];
const HUNGER_PAL_SHELL: [(char, Px); 5] = [
    ('O', [43, 26, 4, 255]),
    ('D', [70, 70, 70, 200]),
    ('W', [231, 231, 231, 255]),
    ('M', [186, 106, 38, 255]),
    ('H', [222, 160, 90, 255]),
];

const ARMOR_PAL_FULL: [(char, Px); 3] = [
    ('O', [58, 58, 58, 255]),
    ('A', [230, 230, 230, 255]),
    ('F', [168, 168, 168, 255]),
];
const ARMOR_PAL_SHELL: [(char, Px); 4] = [
    ('O', [58, 58, 58, 255]),
    ('D', [82, 82, 82, 200]),
    ('F', [168, 168, 168, 255]),
    ('A', [230, 230, 230, 255]),
];

const BUBBLE_PAL: [(char, Px); 4] = [
    ('o', [26, 46, 78, 255]),
    ('W', [235, 247, 255, 255]),
    ('B', [94, 158, 222, 255]),
    ('O', [26, 46, 78, 255]),
];

// -------------------------------------------------------- widget chrome --
// Section 2 chrome values, wiki-verified:
// VERIFIED https://minecraft.wiki — button body #6C6C6C, top+left 2-px
// bevel #A0A0A0, bottom+right 2-px #4A4A4A, 1-px outer outline #000000,
// 1-px inner outline #2A2A2A (rendered at 2x: 2-px/4-px equivalents).
const BTN_BODY: Px = [0x6C, 0x6C, 0x6C, 255];
const BTN_BEVEL_LIGHT: Px = [0xA0, 0xA0, 0xA0, 255];
const BTN_BEVEL_DARK: Px = [0x4A, 0x4A, 0x4A, 255];
const BTN_OUTER: Px = [0x00, 0x00, 0x00, 255];
const BTN_INNER: Px = [0x2A, 0x2A, 0x2A, 255];
// hover: #FFFFFF at alpha 51 blended into the body (the quad renderer
// ALSO has a live tint path; the baked variant keeps the pack-override
// route looking right)
const HOVER_OVERLAY: Px = [255, 255, 255, 51];
// disabled: flattened dark body, no bevel highlight
const BTN_DISABLED_BODY: Px = [0x4C, 0x4C, 0x4C, 235];

// VERIFIED https://minecraft.wiki — slot body #8B8B8B, top+left 1-px
// #373737, bottom+right 1-px #FFFFFF (18x18, 1-px buffer around a
// 16x16 item)
const SLOT_BODY: Px = [0x8B, 0x8B, 0x8B, 255];
const SLOT_DARK: Px = [0x37, 0x37, 0x37, 255];
const SLOT_LIGHT: Px = [0xFF, 0xFF, 0xFF, 255];
const SLOT_HOVER_GLOW: Px = [255, 255, 255, 90];

// VERIFIED https://minecraft.wiki — panel body #C6C6C6, outer 1-px
// #555555, inner 1-px #FFFFFF
const PANEL_BODY: Px = [0xC6, 0xC6, 0xC6, 255];
const PANEL_OUTER: Px = [0x55, 0x55, 0x55, 255];
const PANEL_INNER: Px = [0xFF, 0xFF, 0xFF, 255];

// ---------------------------------------------------------- draw fns --

/// Paint a 9x9 heart into `out` (row stride `w` RGBA pixels). Buffer
/// shorter than 9 rows clips safely.
pub fn draw_heart(out: &mut [u8], w: usize, variant: HeartVariant) {
    match variant {
        HeartVariant::Full => paint_mask(out, w, &HEART_MASK, &HEART_PAL_FULL),
        HeartVariant::Empty => paint_mask(out, w, &HEART_EMPTY_MASK, &HEART_PAL_SHELL),
        HeartVariant::Half => paint_mask(out, w, &HEART_HALF_MASK, &HEART_PAL_SHELL),
    }
}

/// Paint a 9x9 hunger (drumstick) icon.
pub fn draw_hunger(out: &mut [u8], w: usize, variant: HungerVariant) {
    match variant {
        HungerVariant::Full => paint_mask(out, w, &HUNGER_MASK, &HUNGER_PAL_FULL),
        HungerVariant::Empty => paint_mask(out, w, &HUNGER_EMPTY_MASK, &HUNGER_PAL_SHELL),
        HungerVariant::Half => paint_mask(out, w, &HUNGER_HALF_MASK, &HUNGER_PAL_SHELL),
    }
}

/// Paint a 9x9 armor (chestplate) icon.
pub fn draw_armor(out: &mut [u8], w: usize, variant: ArmorVariant) {
    match variant {
        ArmorVariant::Full => paint_mask(out, w, &ARMOR_MASK, &ARMOR_PAL_FULL),
        ArmorVariant::Empty => paint_mask(out, w, &ARMOR_EMPTY_MASK, &ARMOR_PAL_SHELL),
        ArmorVariant::Half => paint_mask(out, w, &ARMOR_HALF_MASK, &ARMOR_PAL_SHELL),
    }
}

/// Paint a 9x9 air bubble (Full) or its pop frame (Gone).
pub fn draw_bubble(out: &mut [u8], w: usize, variant: BubbleVariant) {
    match variant {
        BubbleVariant::Full => paint_mask(out, w, &BUBBLE_MASK, &BUBBLE_PAL),
        BubbleVariant::Gone => paint_mask(out, w, &BUBBLE_GONE_MASK, &BUBBLE_PAL),
    }
}

/// Paint one 9x9 status-effect icon (index 0..EFFECT_ICON_COUNT, order
/// = vc_gameplay EffectKind; out-of-range indices paint nothing).
pub fn draw_effect_icon(out: &mut [u8], w: usize, idx: usize) {
    if idx >= EFFECT_ICON_COUNT {
        return;
    }
    let (mask, pal) = EFFECT_ICONS[idx];
    paint_mask(out, w, mask, pal);
}

/// Paint widget chrome into `out`. Buttons and the panel fill the full
/// 20x20 9-slice source; slots are 18x18 (a 20x20 buffer centers them —
/// the sheet packs slots into 20x20 cells with the vanilla 1-px buffer,
/// so a pack-provided 18x18 slots strip still lands correctly).
pub fn draw_widget(out: &mut [u8], w: usize, variant: WidgetVariant) {
    match variant {
        WidgetVariant::ButtonNormal => paint_button(out, w, BTN_BODY, true),
        // hover: the verified #FFFFFF @ alpha 51 overlay baked into the
        // body (the quad renderer ALSO has a live tint path — this keeps
        // a pack-provided hover variant meaningful on its own)
        WidgetVariant::ButtonHover => {
            paint_button(out, w, blend_over(BTN_BODY, HOVER_OVERLAY), true)
        }
        WidgetVariant::ButtonDisabled => paint_button(out, w, BTN_DISABLED_BODY, false),
        WidgetVariant::SlotEmpty => paint_slot(out, w, false),
        WidgetVariant::SlotHover => paint_slot(out, w, true),
        WidgetVariant::Panel => paint_panel(out, w),
    }
}

/// 20x20 button 9-slice source: flat body, 2-px bevels inset 1 px from
/// the 1-px outer + 1-px inner outlines (4-px corners total, matching
/// the 9-slice split the quad renderer uses).
fn paint_button(out: &mut [u8], w: usize, body: Px, bevel: bool) {
    fill_rect(out, w, 0, 0, 20, 20, body);
    if bevel {
        // top+left light bevel
        fill_rect(out, w, 2, 2, 16, 2, BTN_BEVEL_LIGHT);
        fill_rect(out, w, 2, 2, 2, 16, BTN_BEVEL_LIGHT);
        // bottom+right dark bevel
        fill_rect(out, w, 2, 16, 16, 2, BTN_BEVEL_DARK);
        fill_rect(out, w, 16, 2, 2, 16, BTN_BEVEL_DARK);
    } else {
        // disabled: single inset flat edge, no highlight
        fill_rect(out, w, 2, 16, 16, 2, [0x2A, 0x2A, 0x2A, 255]);
        fill_rect(out, w, 16, 2, 2, 16, [0x2A, 0x2A, 0x2A, 255]);
    }
    // 1-px outer outline + 1-px inner outline
    fill_rect(out, w, 0, 0, 20, 1, BTN_OUTER);
    fill_rect(out, w, 0, 19, 20, 1, BTN_OUTER);
    fill_rect(out, w, 0, 0, 1, 20, BTN_OUTER);
    fill_rect(out, w, 19, 0, 1, 20, BTN_OUTER);
    fill_rect(out, w, 1, 1, 18, 1, BTN_INNER);
    fill_rect(out, w, 1, 18, 18, 1, BTN_INNER);
    fill_rect(out, w, 1, 1, 1, 18, BTN_INNER);
    fill_rect(out, w, 18, 1, 1, 18, BTN_INNER);
}

/// 18x18 slot: light body, dark top+left 1-px, white bottom+right 1-px.
/// Painted at offset (1,1) when the buffer is 20 wide (sheet packing).
fn paint_slot(out: &mut [u8], w: usize, hover: bool) {
    let (x, y) = if w >= 20 { (1usize, 1usize) } else { (0, 0) };
    fill_rect(out, w, x, y, 18, 18, SLOT_BODY);
    if hover {
        fill_rect(out, w, x, y, 18, 18, blend_over(SLOT_BODY, SLOT_HOVER_GLOW));
    }
    // top+left dark inset
    fill_rect(out, w, x, y, 18, 1, SLOT_DARK);
    fill_rect(out, w, x, y, 1, 18, SLOT_DARK);
    // bottom+right white inset
    fill_rect(out, w, x, y + 17, 18, 1, SLOT_LIGHT);
    fill_rect(out, w, x + 17, y, 1, 18, SLOT_LIGHT);
}

/// 20x20 panel 9-slice source: #C6C6C6 body, #555555 outer, #FFFFFF
/// inner — the vanilla container/inventory window chrome.
fn paint_panel(out: &mut [u8], w: usize) {
    fill_rect(out, w, 0, 0, 20, 20, PANEL_BODY);
    fill_rect(out, w, 0, 0, 20, 1, PANEL_OUTER);
    fill_rect(out, w, 0, 19, 20, 1, PANEL_OUTER);
    fill_rect(out, w, 0, 0, 1, 20, PANEL_OUTER);
    fill_rect(out, w, 19, 0, 1, 20, PANEL_OUTER);
    fill_rect(out, w, 1, 1, 18, 1, PANEL_INNER);
    fill_rect(out, w, 1, 18, 18, 1, PANEL_INNER);
    fill_rect(out, w, 1, 1, 1, 18, PANEL_INNER);
    fill_rect(out, w, 18, 1, 1, 18, PANEL_INNER);
}

// VERIFIED https://minecraft.wiki — hotbar background 182x22, selection
// frame 24x22, drawn 22 px above the screen bottom (rendered at the
// engine's 2x scale by the quad pass). Body/edge colors are the
// clean-room equivalents of the vanilla translucent dark chrome.
const HOTBAR_BODY: Px = [24, 24, 24, 190];
const HOTBAR_EDGE: Px = [12, 12, 12, 255];
const HOTBAR_CELL_EDGE: Px = [58, 58, 58, 160];

/// Paint the 182x22 hotbar background (9 joined 18x18 slot recesses on
/// a translucent dark bar).
pub fn draw_hotbar_bg(out: &mut [u8], w: usize) {
    fill_rect(out, w, 0, 0, 182, 22, HOTBAR_BODY);
    // outer frame
    fill_rect(out, w, 0, 0, 182, 1, HOTBAR_EDGE);
    fill_rect(out, w, 0, 21, 182, 1, HOTBAR_EDGE);
    fill_rect(out, w, 0, 0, 1, 22, HOTBAR_EDGE);
    fill_rect(out, w, 181, 0, 1, 22, HOTBAR_EDGE);
    // 9 recessed cells (18x18, 2-px inset, 0-px gap — 2 + 9*20 = 182)
    for i in 0..9usize {
        let x = 1 + i * 20;
        fill_rect(out, w, x, 1, 20, 20, HOTBAR_CELL_EDGE);
        fill_rect(out, w, x, 1, 20, 1, HOTBAR_EDGE);
        fill_rect(out, w, x, 20, 20, 1, HOTBAR_EDGE);
        fill_rect(out, w, x, 1, 1, 20, HOTBAR_EDGE);
        fill_rect(out, w, x + 19, 1, 1, 20, HOTBAR_EDGE);
    }
}

/// Paint the 24x22 hotbar selection frame (white, 2-px band).
pub fn draw_hotbar_sel(out: &mut [u8], w: usize) {
    let white: Px = [255, 255, 255, 255];
    let soft: Px = [200, 200, 200, 160];
    fill_rect(out, w, 0, 0, 24, 22, [0, 0, 0, 0]);
    fill_rect(out, w, 0, 0, 24, 2, white);
    fill_rect(out, w, 0, 20, 24, 2, white);
    fill_rect(out, w, 0, 0, 2, 22, white);
    fill_rect(out, w, 22, 0, 2, 22, white);
    fill_rect(out, w, 2, 2, 20, 1, soft);
    fill_rect(out, w, 2, 19, 20, 1, soft);
    fill_rect(out, w, 2, 2, 1, 18, soft);
    fill_rect(out, w, 21, 2, 1, 18, soft);
}

// VERIFIED https://minecraft.wiki — the options screens tile a 16x16
// dirt tile multiplied by 0.25 brightness. Clean-room dirt: position-
// hashed brown noise (deterministic — same bytes every boot, unlike
// the world atlas's seeded Rng variant).
const DIRT_SHADES: [[i32; 3]; 4] = [
    [134, 96, 67],
    [121, 85, 58],
    [148, 109, 77],
    [110, 78, 52],
];

/// Clean-room position-hash noise (deterministic, no Rng state).
fn hash2(x: i32, y: i32) -> u32 {
    let mut h = (x as u32).wrapping_mul(0x9E37_79B1) ^ (y as u32).wrapping_mul(0x85EB_CA6B);
    h ^= h >> 13;
    h = h.wrapping_mul(0xC2B2_AE35);
    h ^ (h >> 16)
}

/// Paint the 16x16 darkened dirt tile used by options screens
/// (brightness already multiplied by 0.25 — quad tint is plain white).
pub fn draw_options_dirt(out: &mut [u8], w: usize) {
    for y in 0..16i32 {
        for x in 0..16i32 {
            let s = DIRT_SHADES[(hash2(x, y) % 4) as usize];
            let c: Px = [
                (s[0] / 4) as u8,
                (s[1] / 4) as u8,
                (s[2] / 4) as u8,
                255,
            ];
            let idx = (y as usize * w + x as usize) * 4;
            if idx + 3 < out.len() {
                out[idx] = c[0];
                out[idx + 1] = c[1];
                out[idx + 2] = c[2];
                out[idx + 3] = c[3];
            }
        }
    }
}

// ---------------------------------------------------------------- tests --

#[cfg(test)]
mod tests {
    use super::*;

    /// count non-transparent pixels painted into a sentinel buffer
    fn painted(px: &[u8]) -> usize {
        px.as_chunks::<4>().0.iter().filter(|c| c[3] != 0).count()
    }

    #[test]
    fn heart_variants_paint_and_clip() {
        for v in [HeartVariant::Empty, HeartVariant::Full, HeartVariant::Half] {
            let mut buf = [0u8; 9 * 9 * 4];
            draw_heart(&mut buf, 9, v);
            assert!(painted(&buf) > 20, "heart {v:?} painted almost nothing");
            // buffer length equals w * h * 4 for the 9x9 sprite
            assert_eq!(buf.len(), 9 * 9 * 4);
        }
        // a 1-row buffer clips without panicking
        let mut tiny = [0u8; 9 * 4];
        draw_heart(&mut tiny, 9, HeartVariant::Full);
        assert!(tiny.iter().any(|&b| b != 0));
    }

    #[test]
    fn hunger_variants_paint() {
        for v in [HungerVariant::Empty, HungerVariant::Full, HungerVariant::Half] {
            let mut buf = [0u8; 9 * 9 * 4];
            draw_hunger(&mut buf, 9, v);
            assert!(painted(&buf) > 20, "hunger {v:?} painted almost nothing");
        }
    }

    #[test]
    fn armor_variants_paint() {
        for v in [ArmorVariant::Empty, ArmorVariant::Full, ArmorVariant::Half] {
            let mut buf = [0u8; 9 * 9 * 4];
            draw_armor(&mut buf, 9, v);
            assert!(painted(&buf) > 20, "armor {v:?} painted almost nothing");
        }
    }

    #[test]
    fn bubble_variants_paint() {
        for v in [BubbleVariant::Full, BubbleVariant::Gone] {
            let mut buf = [0u8; 9 * 9 * 4];
            draw_bubble(&mut buf, 9, v);
            assert!(painted(&buf) > 8, "bubble {v:?} painted almost nothing");
        }
    }

    /// Sub-round 1: every effect-icon tile paints real ink at both the
    /// 9x9 sheet size and a 20x20 zoom (the coverage-guard pattern — a
    /// blank icon is a missing painter), the count matches the engine's
    /// 16 effect kinds, and out-of-range indices are a no-op.
    #[test]
    fn effect_icons_all_paint_ink() {
        assert_eq!(EFFECT_ICON_COUNT, 16, "one tile per engine effect kind");
        for idx in 0..EFFECT_ICON_COUNT {
            let mut buf = [0u8; 9 * 9 * 4];
            draw_effect_icon(&mut buf, 9, idx);
            assert!(
                painted(&buf) >= 12,
                "effect icon {} ({}) painted almost nothing",
                idx,
                effect_icon_name(idx)
            );
            let mut zoom = [0u8; 20 * 20 * 4];
            draw_effect_icon(&mut zoom, 20, idx);
            assert!(
                painted(&zoom) >= 12,
                "effect icon {} ({}) blank at 20x20",
                idx,
                effect_icon_name(idx)
            );
        }
        // out-of-range: paints nothing, never panics
        let mut buf = [0u8; 9 * 9 * 4];
        draw_effect_icon(&mut buf, 9, EFFECT_ICON_COUNT);
        draw_effect_icon(&mut buf, 9, usize::MAX);
        assert_eq!(painted(&buf), 0);
    }

    /// Sub-round 1: the icon order matches the engine's EffectKind
    /// declaration order by name (the game layer maps kinds to these
    /// indices positionally — drift would mislabel every icon).
    #[test]
    fn effect_icon_names_match_effectkind_order() {
        for (idx, expect) in [
            "wither",
            "poison",
            "regeneration",
            "speed",
            "haste",
            "resistance",
            "jump_boost",
            "strength",
            "slowness",
            "hunger",
            "absorption",
            "blindness",
            "water_breathing",
            "slow_falling",
            "conduit_power",
            "dolphins_grace",
        ]
        .into_iter()
        .enumerate()
        {
            assert_eq!(effect_icon_name(idx), expect);
        }
    }

    #[test]
    fn widget_variants_paint() {
        for (v, w_expect) in [
            (WidgetVariant::ButtonNormal, 20usize),
            (WidgetVariant::ButtonHover, 20),
            (WidgetVariant::ButtonDisabled, 20),
            (WidgetVariant::SlotEmpty, 20),
            (WidgetVariant::SlotHover, 20),
            (WidgetVariant::Panel, 20),
        ] {
            let mut buf = [0u8; 20 * 20 * 4];
            draw_widget(&mut buf, 20, v);
            assert!(painted(&buf) > 100, "widget {v:?} painted almost nothing");
            assert_eq!(buf.len(), w_expect * 20 * 4);
        }
    }

    #[test]
    fn button_corner_pixels_match_outline_color() {
        // VERIFIED chrome: the 1-px outer outline is #000000 — corner
        // pixels of the 20x20 button source must be exactly that
        let mut buf = [0u8; 20 * 20 * 4];
        draw_widget(&mut buf, 20, WidgetVariant::ButtonNormal);
        for (x, y) in [(0usize, 0usize), (19, 0), (0, 19), (19, 19)] {
            let o = (y * 20 + x) * 4;
            assert_eq!(&buf[o..o + 4], &[0, 0, 0, 255], "corner {x},{y}");
        }
        // body pixel (center) is the verified #6C6C6C body
        let o = (10 * 20 + 10) * 4;
        assert_eq!(&buf[o..o + 4], &[0x6C, 0x6C, 0x6C, 255]);
    }

    #[test]
    fn slot_chrome_matches_verified_colors() {
        // VERIFIED chrome: slot body #8B8B8B, top+left #373737,
        // bottom+right #FFFFFF (18x18 sprite in a minimal buffer)
        let mut buf = [0u8; 18 * 18 * 4];
        draw_widget(&mut buf, 18, WidgetVariant::SlotEmpty);
        let body = (9 * 18 + 9) * 4;
        assert_eq!(&buf[body..body + 4], &[0x8B, 0x8B, 0x8B, 255]);
        let top = 9 * 4;
        assert_eq!(&buf[top..top + 4], &[0x37, 0x37, 0x37, 255]);
        let bottom = (17 * 18 + 9) * 4;
        assert_eq!(&buf[bottom..bottom + 4], &[255, 255, 255, 255]);
    }

    #[test]
    fn panel_chrome_matches_verified_colors() {
        // VERIFIED chrome: panel body #C6C6C6, outer #555555, inner #FFF
        let mut buf = [0u8; 20 * 20 * 4];
        draw_widget(&mut buf, 20, WidgetVariant::Panel);
        let body = (10 * 20 + 10) * 4;
        assert_eq!(&buf[body..body + 4], &[0xC6, 0xC6, 0xC6, 255]);
        let outer = 10 * 4;
        assert_eq!(&buf[outer..outer + 4], &[0x55, 0x55, 0x55, 255]);
        let inner = (20 + 10) * 4; // row 1, col 10
        assert_eq!(&buf[inner..inner + 4], &[255, 255, 255, 255]);
    }

    #[test]
    fn mask_paint_writes_only_inside_region() {
        // pre-fill sentinel: every byte 0xAB; painting a sparse sprite
        // must not touch pixels outside its mask footprint (no runaway
        // writes past row stride)
        let mut buf = [0xABu8; 12 * 9 * 4];
        draw_heart(&mut buf, 12, HeartVariant::Full);
        // column 10-11 of every row must be untouched (heart is 9 wide)
        for y in 0..9usize {
            for x in 10..12usize {
                let o = (y * 12 + x) * 4;
                assert_eq!(&buf[o..o + 4], &[0xAB; 4], "runaway write at {x},{y}");
            }
        }
    }

    /// the coverage guard (G5, `vNNN_tiles_all_painted` pattern): every
    /// sprite family + variant paints a non-trivial footprint — a blank
    /// variant means a missing painter slipped in
    #[test]
    fn gui_tiles_all_painted() {
        let mut buf = [0u8; 20 * 20 * 4];
        for v in [
            HeartVariant::Empty,
            HeartVariant::Full,
            HeartVariant::Half,
        ] {
            buf.fill(0);
            draw_heart(&mut buf, 20, v);
            assert!(painted(&buf) > 20, "blank heart variant {v:?}");
        }
        for v in [
            HungerVariant::Empty,
            HungerVariant::Full,
            HungerVariant::Half,
        ] {
            buf.fill(0);
            draw_hunger(&mut buf, 20, v);
            assert!(painted(&buf) > 20, "blank hunger variant {v:?}");
        }
        for v in [ArmorVariant::Empty, ArmorVariant::Full, ArmorVariant::Half] {
            buf.fill(0);
            draw_armor(&mut buf, 20, v);
            assert!(painted(&buf) > 20, "blank armor variant {v:?}");
        }
        for v in [BubbleVariant::Full, BubbleVariant::Gone] {
            buf.fill(0);
            draw_bubble(&mut buf, 20, v);
            assert!(painted(&buf) > 8, "blank bubble variant {v:?}");
        }
        for v in [
            WidgetVariant::ButtonNormal,
            WidgetVariant::ButtonHover,
            WidgetVariant::ButtonDisabled,
            WidgetVariant::SlotEmpty,
            WidgetVariant::SlotHover,
            WidgetVariant::Panel,
        ] {
            buf.fill(0);
            draw_widget(&mut buf, 20, v);
            assert!(painted(&buf) > 100, "blank widget variant {v:?}");
        }
        buf.fill(0);
        let mut hbuf = [0u8; 182 * 22 * 4];
        draw_hotbar_bg(&mut hbuf, 182);
        assert!(painted(&hbuf) > 182 * 22 / 2, "hotbar bar mostly unpainted");
        let mut sbuf = [0u8; 24 * 22 * 4];
        draw_hotbar_sel(&mut sbuf, 24);
        assert!(painted(&sbuf) > 80, "selection frame unpainted");
        buf.fill(0);
        draw_options_dirt(&mut buf, 20);
        assert!(painted(&buf) == 16 * 16, "dirt tile not fully opaque in stride 20");
    }

    #[test]
    fn hotbar_chrome_paints_full_bar() {
        let mut buf = [0u8; 182 * 22 * 4];
        draw_hotbar_bg(&mut buf, 182);
        // every row of the bar body is painted (translucent, but alpha
        // non-zero across the full width)
        for y in [0usize, 5, 11, 21] {
            for x in [0usize, 90, 181] {
                let o = (y * 182 + x) * 4;
                assert!(buf[o + 3] != 0, "hole in hotbar bar at {x},{y}");
            }
        }
        let mut sel = [0u8; 24 * 22 * 4];
        draw_hotbar_sel(&mut sel, 24);
        assert!(painted(&sel) > 80, "selection frame missing band pixels");
    }

    #[test]
    fn options_dirt_is_deterministic_and_dark() {
        let mut a = [0u8; 16 * 16 * 4];
        let mut b = [0u8; 16 * 16 * 4];
        draw_options_dirt(&mut a, 16);
        draw_options_dirt(&mut b, 16);
        assert_eq!(a, b, "dirt tile must be byte-identical across calls");
        // 0.25 brightness of the brightest shade stays dark
        let max = a.as_chunks::<4>().0.iter().map(|c| c[0].max(c[1]).max(c[2])).max();
        assert_eq!(max, Some(148 / 4), "dirt tile not darkened to 0.25");
        // fully opaque
        assert!(a.as_chunks::<4>().0.iter().all(|c| c[3] == 255));
    }
}
