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
fn rgb(hex: u32) -> [f32; 3] {
    [
        ((hex >> 16) & 0xFF) as f32 / 255.0,
        ((hex >> 8) & 0xFF) as f32 / 255.0,
        (hex & 0xFF) as f32 / 255.0,
    ]
}

/// grass colormap color per biome (vanilla 1.16.5)
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
        _ => rgb(0x91BD59), // default
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
        _ => rgb(0x77AB2F),
    }
}

/// water color per biome (1.16 biome-effect water tint)
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
    for b in 0u8..7 {
        let g = grass_color(b);
        let f = foliage_color(b);
        let w = water_color(b);
        for (i, c) in [g, f, w].iter().enumerate() {
            let hex = ((c[0] * 255.0) as u32) << 16 | ((c[1] * 255.0) as u32) << 8 | (c[2] * 255.0) as u32;
            put(&mut data, (i + 1) as u8, b, hex);
        }
    }
    put(&mut data, KIND_FOLIAGE, SLOT_BIRCH, BIRCH_COLOR);
    put(&mut data, KIND_FOLIAGE, SLOT_SPRUCE, SPRUCE_COLOR);
    data
}

/// per-face tint kind for a BUILT-IN (greedy-path) block.
/// `top_face` because grass tint applies to the grass-block TOP only (the
/// side overlay is pre-baked into our tile).
#[inline]
pub fn block_face_tint(block: u8, top_face: bool) -> u8 {
    match block {
        GRASS => if top_face { KIND_GRASS } else { TINT_NONE },
        TALL_GRASS => KIND_GRASS,
        LEAVES => KIND_FOLIAGE,
        BIRCH_LEAVES => KIND_FOLIAGE,
        SPRUCE_LEAVES => KIND_FOLIAGE,
        _ => TINT_NONE,
    }
}

/// full packed tint for a built-in block face in a biome column
#[inline]
pub fn block_face_tint_packed(block: u8, top_face: bool, biome: u8) -> u8 {
    match block {
        GRASS if top_face => pack(KIND_GRASS, biome),
        TALL_GRASS => pack(KIND_GRASS, biome),
        LEAVES => pack(KIND_FOLIAGE, biome),
        BIRCH_LEAVES => pack(KIND_FOLIAGE, SLOT_BIRCH),
        SPRUCE_LEAVES => pack(KIND_FOLIAGE, SLOT_SPRUCE),
        WATER => pack(KIND_WATER, biome),
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
pub fn block_tint_color(block: u8, biome: u8) -> [f32; 3] {
    match block {
        GRASS | TALL_GRASS => grass_color(biome),
        LEAVES => foliage_color(biome),
        BIRCH_LEAVES => rgb(BIRCH_COLOR),
        SPRUCE_LEAVES => rgb(SPRUCE_COLOR),
        WATER => water_color(biome),
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
    fn block_face_tint_rules() {
        // grass top tinted, sides not
        assert_eq!(block_face_tint_packed(GRASS, true, 3), pack(KIND_GRASS, 3));
        assert_eq!(block_face_tint_packed(GRASS, false, 3), TINT_NONE);
        // leaves: oak biome-tinted, birch/spruce fixed
        assert_eq!(block_face_tint_packed(LEAVES, true, 2), pack(KIND_FOLIAGE, 2));
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
