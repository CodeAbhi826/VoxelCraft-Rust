//! Biome tint — vanilla 1.16.5 parity for grass / foliage / water colors
//! (§18 "Lighting and Vanilla Visuals", Phase 5).
//!
//! VC-16 carries a per-face tint index in `w3`'s reserved byte
//! (`tint:u2 kind << 6 | slot:u6`):
//!   kind 1 = grass, 2 = foliage, 3 = water
//!   slot 0..6 = our Biome, slot 48/49 = birch/spruce constant foliage
//! The GPU decodes it in the VERTEX shader with a textureLoad into the
//! 64×4 RGBA LUT below (row = kind, col = slot) — textureLoad (not a
//! uniform array index) is legal with non-uniform indices on every
//! backend, incl. Vulkan's UBO dynamically-uniform rule and WebGL2.
//!
//! Color values are the vanilla 1.16.5 per-biome grass/foliage/water
//! hex constants (biome effects / default colormap values), sRGB 0..255.

use crate::blocks::*;

pub const TINT_NONE: u8 = 0;

pub const KIND_GRASS: u8 = 1;
pub const KIND_FOLIAGE: u8 = 2;
pub const KIND_WATER: u8 = 3;

/// constant-color pseudo-slots (vanilla birch/spruce leaves are NOT
/// biome-tinted — they use fixed colors)
pub const SLOT_BIRCH: u8 = 48;
pub const SLOT_SPRUCE: u8 = 49;
/// Phase E2: lava rides the WATER tint kind with a fixed orange
/// pseudo-slot (lava is not biome-tinted — VERIFIED: constant color)
pub const SLOT_LAVA: u8 = 50;
/// lava surface color (clean-room orange, emissive-bright)
pub const LAVA_COLOR: u32 = 0xD45A12;

/// pack a tint index; returns TINT_NONE for kind 0
#[inline]
pub fn pack(kind: u8, slot: u8) -> u8 {
    ((kind & 3) << 6) | (slot & 0x3F)
}

/// (kind, slot) of a packed tint (0,0) for none
#[inline]
pub fn unpack(t: u8) -> (u8, u8) {
    (t >> 6, t & 0x3F)
}

#[inline]
/// sRGB hex (wiki colormap values are display-referred) to LINEAR float.
/// The tint LUT texture is Rgba8Unorm (no hardware decode) and the shader
/// multiplies tints in linear space — passing sRGB through washed every
/// biome tint out (grass #59AE30 rendered ~0.63 instead of ~0.35). Found
/// in the Part 5 color audit (row 11).
fn rgb(hex: u32) -> [f32; 3] {
    [
        srgb_byte_to_linear(((hex >> 16) & 0xFF) as u8),
        srgb_byte_to_linear(((hex >> 8) & 0xFF) as u8),
        srgb_byte_to_linear((hex & 0xFF) as u8),
    ]
}

fn srgb_byte_to_linear(byte: u8) -> f32 {
    let s = byte as f32 / 255.0;
    if s <= 0.04045 {
        s / 12.92
    } else {
        libm::powf((s + 0.055) / 1.055, 2.4)
    }
}

/// Representative (temperature, downfall) per internal biome id for pack
/// colormap sampling (vanilla JE biome data, widely published; clamped to
/// 0..1 at sample time). Swamp uses its map position then darkens via the
/// fixed swamp handling (unchanged engine behavior).
fn biome_climate(biome: u8) -> (f32, f32) {
    match biome {
        0 => (0.5, 0.5),   // Ocean
        1 => (0.8, 0.4),   // Beach
        2 => (0.8, 0.4),   // Plains
        3 => (0.7, 0.8),   // Forest
        4 => (2.0, 0.0),   // Desert
        5 => (-0.5, 0.4),  // Snowy Taiga
        6 => (0.2, 0.3),   // Mountains
        7 => (2.0, 0.0),   // Nether Wastes
        8 => (0.25, 0.8),  // Taiga
        9 => (0.6, 0.6),   // Birch Forest
        10 => (0.95, 0.9), // Jungle
        11 => (1.2, 0.0),  // Savanna
        12 => (0.8, 0.9),  // Swamp
        13 => (2.0, 0.0),  // Badlands
        14 => (0.7, 0.8),  // Flower Forest
        15 => (0.8, 0.4),  // Sunflower Plains
        16 => (0.0, 0.5),  // Ice Spikes
        17 => (0.7, 0.8),  // Dark Forest
        _ => (0.8, 0.4),
    }
}

/// Sample a decoded colormap (sRGB bytes) at a climate point → LINEAR
/// tint. UV orientation u=temperature, v=downfall is [ESTIMATED /
/// APPROXIMATION] (no pixel sources consulted; V1 review at plan end
/// arbitrates — flip here if the look is wrong).
pub fn sample_colormap(rgba: &[u8], w: u32, h: u32, temp: f32, down: f32) -> [f32; 3] {
    let x = (temp.clamp(0.0, 1.0) * (w - 1) as f32).round() as u32;
    let y = (down.clamp(0.0, 1.0) * (h - 1) as f32).round() as u32;
    let i = ((y * w + x) * 4) as usize;
    [
        srgb_byte_to_linear(rgba[i]),
        srgb_byte_to_linear(rgba[i + 1]),
        srgb_byte_to_linear(rgba[i + 2]),
    ]
}

/// Override LUT grass/foliage rows from pack colormaps (2B, shape A+:
/// per-biome representative sampling — matches our per-biome LUT
/// architecture; within-biome variation is a later enhancement).
/// `None` map = keep engine constants. Fixed slots (birch/spruce/lava)
/// and water rows are never overridden (vanilla has no water colormap).
pub fn override_lut_from_maps(
    lut: &mut [u8],
    grass: Option<(&[u8], u32, u32)>,
    foliage: Option<(&[u8], u32, u32)>,
) {
    let putf = |lut: &mut [u8], kind: u8, slot: u8, c: [f32; 3]| {
        let idx = ((kind as u32 * LUT_W + slot as u32) * 4) as usize;
        lut[idx] = (c[0].clamp(0.0, 1.0) * 255.0).round() as u8;
        lut[idx + 1] = (c[1].clamp(0.0, 1.0) * 255.0).round() as u8;
        lut[idx + 2] = (c[2].clamp(0.0, 1.0) * 255.0).round() as u8;
    };
    for b in 0u8..18 {
        let (t, d) = biome_climate(b);
        if let Some((rgba, w, h)) = grass {
            putf(lut, KIND_GRASS, b, sample_colormap(rgba, w, h, t, d));
        }
        if let Some((rgba, w, h)) = foliage {
            putf(lut, KIND_FOLIAGE, b, sample_colormap(rgba, w, h, t, d));
        }
    }
}

/// grass colormap color per biome (vanilla 1.16.5; §28 Nether Wastes has
/// no grass — a defensive reddish-olive for player-placed grass blocks)
#[inline]
pub fn grass_color(biome: u8) -> [f32; 3] {
    match biome {
        0 => rgb(0x8EB971), // Ocean
        1 => rgb(0x91BD59), // Beach
        2 => rgb(0x91BD59), // Plains
        3 => rgb(0x79C05A), // Forest
        4 => rgb(0xBFB755), // Desert
        5 => rgb(0x80B497), // Snowy Taiga
        6 => rgb(0x8AB689), // Mountains
        7 => rgb(0x77624B), // Nether Wastes (§28)
        // Phase 10 (live wiki biome-color table, JE columns):
        8 => rgb(0x86B783),  // Taiga
        9 => rgb(0x88BB67),  // Birch Forest
        10 => rgb(0x59C93C), // Jungle
        11 => rgb(0xBFB755), // Savanna
        12 => rgb(0x6A7039), // Swamp
        13 => rgb(0x90814D), // Badlands
        // 1.7.2 bracket (live wiki biome pages, 2026-09-06):
        14 => rgb(0x79C05A), // Flower Forest (wiki: #79C05A)
        15 => rgb(0x91BD59), // Sunflower Plains (wiki: #91BD59)
        16 => rgb(0x80B497), // Ice Spikes (wiki: #80B497)
        17 => rgb(0x507A32), // Dark Forest (wiki: #507A32)
        _ => rgb(0x91BD59),  // default
    }
}

/// foliage (leaves) color per biome
#[inline]
pub fn foliage_color(biome: u8) -> [f32; 3] {
    match biome {
        0 => rgb(0x74A457), // Ocean
        1 => rgb(0x77AB2F), // Beach
        2 => rgb(0x77AB2F), // Plains
        3 => rgb(0x59AE30), // Forest
        4 => rgb(0xAEA42),  // Desert
        5 => rgb(0x60A17B), // Snowy Taiga
        6 => rgb(0x6B9959), // Mountains
        7 => rgb(0x6B5442), // Nether Wastes (§28)
        // Phase 10 (live wiki biome-color table, JE columns):
        8 => rgb(0x68A464),  // Taiga
        9 => rgb(0x6BA941),  // Birch Forest
        10 => rgb(0x30BB0B), // Jungle
        11 => rgb(0xAEA42A), // Savanna
        12 => rgb(0x8DB127), // Swamp
        13 => rgb(0x9E814D), // Badlands
        // 1.7.2 bracket: flower forest inherits Forest's #59AE30;
        // sunflower plains inherits Plains; ice spikes #60A17B (wiki);
        // dark forest inherits Forest foliage (the darkness is grass-side)
        14 => rgb(0x59AE30), // Flower Forest
        15 => rgb(0x77AB2F), // Sunflower Plains
        16 => rgb(0x60A17B), // Ice Spikes
        17 => rgb(0x59AE30), // Dark Forest
        _ => rgb(0x77AB2F),
    }
}

/// water color per biome (1.16 biome-effect water tint; the Nether's is
/// a muddy dark brown for the rare placed water)
#[inline]
pub fn water_color(biome: u8) -> [f32; 3] {
    match biome {
        0 => rgb(0x3F76E4), // Ocean
        1 => rgb(0x15AFC1), // Beach
        2 => rgb(0x44AFF5), // Plains
        3 => rgb(0x287082), // Forest
        4 => rgb(0x32A598), // Desert
        5 => rgb(0x205E83), // Snowy Taiga
        6 => rgb(0x45765E), // Mountains
        7 => rgb(0x4A2B18), // Nether Wastes (§28)
        // Phase 10 (live wiki water-color table; savanna/birch/jungle/
        // badlands keep the default blue — the wiki lists no special
        // value for them)
        8 => rgb(0x287082),  // Taiga
        9 => rgb(0x3F76E4),  // Birch Forest
        10 => rgb(0x3F76E4), // Jungle
        11 => rgb(0x3F76E4), // Savanna
        12 => rgb(0x617B64), // Swamp (murky green — wiki)
        13 => rgb(0x3F76E4), // Badlands
        // 1.7.2: ice spikes water — wiki lists #3F76E4 (current) with
        // #14559B as the pre-1.13-era/alternate value; 1.16.5 = #3F76E4.
        // The other three use their family defaults.
        14 => rgb(0x44AFF5), // Flower Forest (plains-family)
        15 => rgb(0x44AFF5), // Sunflower Plains
        16 => rgb(0x3F76E4), // Ice Spikes
        17 => rgb(0x287082), // Dark Forest (forest-family)
        _ => rgb(0x3F76E4),
    }
}

/// constant foliage colors (vanilla fixed-color leaves)
pub const BIRCH_COLOR: u32 = 0x80A755;
pub const SPRUCE_COLOR: u32 = 0x619961;

/// tint LUT texture payload: 64 wide × 4 high RGBA8 (row = kind, col =
/// slot; row 0 unused — TINT_NONE never reaches the shader). Written once
/// at renderer init; biome colors are engine constants.
pub const LUT_W: u32 = 64;
pub const LUT_H: u32 = 4;

pub fn lut_rgba() -> Vec<u8> {
    let mut data = vec![255u8; (LUT_W * LUT_H * 4) as usize];
    let put = |data: &mut Vec<u8>, kind: u8, slot: u8, hex: u32| {
        let idx = ((kind as u32 * LUT_W + slot as u32) * 4) as usize;
        data[idx] = ((hex >> 16) & 0xFF) as u8;
        data[idx + 1] = ((hex >> 8) & 0xFF) as u8;
        data[idx + 2] = (hex & 0xFF) as u8;
        data[idx + 3] = 255;
    };
    // 1.7.2 BUG FIX (found during the bracket audit): the loop stopped at
    // 8, so the Phase-10 biomes 8..=13 and the 1.7 biomes 14..=17 never
    // landed in the LUT — their shader tints sampled white (untinted)
    // despite the match arms above having correct colors. Loop the full
    // biome range now.
    for b in 0u8..18 {
        let g = grass_color(b);
        let f = foliage_color(b);
        let w = water_color(b);
        for (i, c) in [g, f, w].iter().enumerate() {
            let hex = ((c[0] * 255.0) as u32) << 16
                | ((c[1] * 255.0) as u32) << 8
                | (c[2] * 255.0) as u32;
            put(&mut data, (i + 1) as u8, b, hex);
        }
    }
    put(
        &mut data,
        KIND_FOLIAGE,
        SLOT_BIRCH,
        hex_lin_bytes(BIRCH_COLOR),
    );
    put(
        &mut data,
        KIND_FOLIAGE,
        SLOT_SPRUCE,
        hex_lin_bytes(SPRUCE_COLOR),
    );
    put(&mut data, KIND_WATER, SLOT_LAVA, hex_lin_bytes(LAVA_COLOR));
    data
}

/// sRGB hex to LINEAR-encoded bytes (same transfer as rgb(), for LUT
/// slots written as raw bytes — birch/spruce/lava had the same
/// wash-out as the float colors).
fn hex_lin_bytes(hex: u32) -> u32 {
    let f = rgb(hex);
    let b = |x: f32| (x.clamp(0.0, 1.0) * 255.0).round() as u32;
    (b(f[0]) << 16) | (b(f[1]) << 8) | b(f[2])
}

/// per-face tint kind for a BUILT-IN (greedy-path) block.
/// `top_face` because grass tint applies to the grass-block TOP only (the
/// side overlay is pre-baked into our tile).
#[inline]
pub fn block_face_tint(block: u16, top_face: bool) -> u8 {
    match block {
        GRASS => {
            if top_face {
                KIND_GRASS
            } else {
                TINT_NONE
            }
        }
        TALL_GRASS => KIND_GRASS,
        LEAVES => KIND_FOLIAGE,
        BIRCH_LEAVES => KIND_FOLIAGE,
        SPRUCE_LEAVES => KIND_FOLIAGE,
        ACACIA_LEAVES => KIND_FOLIAGE,
        DARK_OAK_LEAVES => KIND_FOLIAGE,
        _ => TINT_NONE,
    }
}

/// full packed tint for a built-in block face in a biome column
#[inline]
pub fn block_face_tint_packed(block: u16, top_face: bool, biome: u8) -> u8 {
    match block {
        GRASS if top_face => pack(KIND_GRASS, biome),
        TALL_GRASS => pack(KIND_GRASS, biome),
        LEAVES => pack(KIND_FOLIAGE, biome),
        BIRCH_LEAVES => pack(KIND_FOLIAGE, SLOT_BIRCH),
        SPRUCE_LEAVES => pack(KIND_FOLIAGE, SLOT_SPRUCE),
        ACACIA_LEAVES => pack(KIND_FOLIAGE, biome),
        DARK_OAK_LEAVES => pack(KIND_FOLIAGE, biome),
        WATER => pack(KIND_WATER, biome),
        // Phase E2: fixed lava color (not biome-tinted)
        LAVA => pack(KIND_WATER, SLOT_LAVA),
        // fluids round: the bubble column takes the WATER tint (the
        // biome slot — the column renders as water)
        BUBBLE_COLUMN => pack(KIND_WATER, biome),
        _ => TINT_NONE,
    }
}

/// tint for a MODEL-path (JSON blockstate) face carrying `tintindex >= 0`.
/// The kind comes from the BLOCK family (model JSON only says "tinted",
/// not which colormap); grass sides are NOT tinted (our side tile is
/// pre-baked — top_face mirrors the greedy-path rule).
#[inline]
pub fn model_face_tint_packed(state: u16, top_face: bool, biome: u8) -> u8 {
    block_face_tint_packed(state_block(state), top_face, biome)
}

/// resolved tint COLOR for a block (particles / CPU-side consumers)
#[inline]
pub fn block_tint_color(block: u16, biome: u8) -> [f32; 3] {
    match block {
        GRASS | TALL_GRASS => grass_color(biome),
        LEAVES => foliage_color(biome),
        BIRCH_LEAVES => rgb(BIRCH_COLOR),
        SPRUCE_LEAVES => rgb(SPRUCE_COLOR),
        ACACIA_LEAVES | DARK_OAK_LEAVES => foliage_color(biome),
        WATER => water_color(biome),
        // fluids round: the bubble column — the biome water color
        BUBBLE_COLUMN => water_color(biome),
        _ => [1.0, 1.0, 1.0],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pack_unpack_roundtrip() {
        for kind in 1u8..=3 {
            for slot in [0u8, 2, 6, 48, 49] {
                assert_eq!(unpack(pack(kind, slot)), (kind, slot));
            }
        }
        // TINT_NONE decodes to the (unused) row 0
        assert_eq!(unpack(TINT_NONE), (0, 0));
        // high bits can't leak into the slot field
        assert_eq!(unpack(0xFF), (3, 63));
    }

    #[test]
    fn lut_rows_have_distinct_biome_colors() {
        let lut = lut_rgba();
        let px = |kind: u8, slot: u8| -> [u8; 3] {
            let i = ((kind as u32 * LUT_W + slot as u32) * 4) as usize;
            [lut[i], lut[i + 1], lut[i + 2]]
        };
        // plains vs forest grass must differ (vanilla: 0x91BD59 vs 0x79C05A)
        assert_ne!(px(KIND_GRASS, 2), px(KIND_GRASS, 3));
        // desert grass is yellowish (higher red than green-forest)
        assert!(px(KIND_GRASS, 4)[0] > px(KIND_GRASS, 3)[0]);
        // ocean water is blue-dominant
        let w = px(KIND_WATER, 0);
        assert!(w[2] > w[0]);
        // birch/spruce fixed foliage differ
        assert_ne!(px(KIND_FOLIAGE, SLOT_BIRCH), px(KIND_FOLIAGE, SLOT_SPRUCE));
        // unknown biome slots default to plain colors, alpha always opaque
        for slot in [7u8, 32, 63] {
            let i = ((KIND_GRASS as u32 * LUT_W + slot as u32) * 4) as usize;
            assert_eq!(lut[i + 3], 255);
        }
    }

    #[test]
    fn rgb_linearizes_srgb_hex() {
        // endpoints exact
        assert_eq!(rgb(0xFFFFFF), [1.0, 1.0, 1.0]);
        assert_eq!(rgb(0x000000), [0.0, 0.0, 0.0]);
        // grass #59AE30: sRGB (0.349, 0.682, 0.188) -> linear — values
        // below their sRGB inputs (the old passthrough washed tints out)
        let g = rgb(0x59AE30);
        for (got, want) in g.iter().zip([0.0997, 0.4232, 0.0296]) {
            assert!((got - want).abs() < 0.002, "got {g:?}");
        }
        assert!(g[0] < 0.349 && g[1] < 0.682 && g[2] < 0.188);
    }

    #[test]
    fn colormap_sample_and_override() {
        // 2x2 map: R at (0,0), G at (1,0), B at (0,1), W at (1,1)
        let map: Vec<u8> = vec![
            255, 0, 0, 255, 0, 255, 0, 255, //
            0, 0, 255, 255, 255, 255, 255, 255,
        ];
        let c = sample_colormap(&map, 2, 2, 0.0, 0.0);
        assert_eq!(c, [1.0, 0.0, 0.0]);
        let c = sample_colormap(&map, 2, 2, 1.0, 1.0);
        assert_eq!(c, [1.0, 1.0, 1.0]);
        // mid grey linearizes down (sRGB 0.5 -> linear ~0.214)
        let mid = sample_colormap(&[128, 128, 128, 255], 1, 1, 0.3, 0.7);
        assert!((mid[0] - 0.214).abs() < 0.005, "got {mid:?}");
        // override writes grass rows from the map, keeps water + fixed
        let mut lut = lut_rgba();
        let before_water = lut[((KIND_WATER as u32 * LUT_W) * 4) as usize];
        override_lut_from_maps(&mut lut, Some((&map, 2, 2)), None);
        let g_plains = ((KIND_GRASS as u32 * LUT_W + 2) * 4) as usize;
        // plains (0.8, 0.4) rounds to pixel (1,0) = green
        assert_eq!(
            [lut[g_plains], lut[g_plains + 1], lut[g_plains + 2]],
            [0, 255, 0]
        );
        assert_eq!(
            lut[((KIND_WATER as u32 * LUT_W) * 4) as usize],
            before_water
        );
    }

    #[test]
    fn block_face_tint_rules() {
        // grass top tinted, sides not
        assert_eq!(block_face_tint_packed(GRASS, true, 3), pack(KIND_GRASS, 3));
        assert_eq!(block_face_tint_packed(GRASS, false, 3), TINT_NONE);
        // leaves: oak biome-tinted, birch/spruce fixed
        assert_eq!(
            block_face_tint_packed(LEAVES, true, 2),
            pack(KIND_FOLIAGE, 2)
        );
        assert_eq!(
            block_face_tint_packed(BIRCH_LEAVES, true, 2),
            pack(KIND_FOLIAGE, SLOT_BIRCH)
        );
        assert_eq!(
            block_face_tint_packed(SPRUCE_LEAVES, true, 2),
            pack(KIND_FOLIAGE, SLOT_SPRUCE)
        );
        // water tinted in every biome, stone never
        assert_eq!(block_face_tint_packed(WATER, false, 0), pack(KIND_WATER, 0));
        assert_eq!(block_face_tint_packed(STONE, true, 3), TINT_NONE);
    }
}
