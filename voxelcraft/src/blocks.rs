//! Block registry — ids, tiles, physical + optical properties, sound families.

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SoundFamily {
    Grass,
    Dirt,
    Stone,
    Wood,
    Sand,
    Leaves,
    Glass,
    Water,
    None,
}

// Tile indices inside the 16x16-tile procedural atlas (256x256 px).
pub const TILE_GRASS_TOP: u16 = 0;
pub const TILE_GRASS_SIDE: u16 = 1;
pub const TILE_DIRT: u16 = 2;
pub const TILE_STONE: u16 = 3;
pub const TILE_COBBLE: u16 = 4;
pub const TILE_SAND: u16 = 5;
pub const TILE_LOG_SIDE: u16 = 6;
pub const TILE_LOG_TOP: u16 = 7;
pub const TILE_PLANKS: u16 = 8;
pub const TILE_LEAVES: u16 = 9;
pub const TILE_WATER: u16 = 10;
pub const TILE_GLASS: u16 = 11;
pub const TILE_BEDROCK: u16 = 12;
pub const TILE_GRAVEL: u16 = 13;
pub const TILE_SNOW: u16 = 14;
pub const TILE_SNOW_SIDE: u16 = 15;
pub const TILE_TALL_GRASS: u16 = 16;
pub const TILE_FLOWER_RED: u16 = 17;
pub const TILE_FLOWER_YELLOW: u16 = 18;

// Block ids (u8 in chunk storage).
pub const AIR: u8 = 0;
pub const GRASS: u8 = 1;
pub const DIRT: u8 = 2;
pub const STONE: u8 = 3;
pub const COBBLE: u8 = 4;
pub const SAND: u8 = 5;
pub const OAK_LOG: u8 = 6;
pub const PLANKS: u8 = 7;
pub const LEAVES: u8 = 8;
pub const WATER: u8 = 9;
pub const GLASS: u8 = 10;
pub const BEDROCK: u8 = 11;
pub const GRAVEL: u8 = 12;
pub const SNOW: u8 = 13;
pub const SNOW_GRASS: u8 = 14;
pub const TALL_GRASS: u8 = 15;
pub const FLOWER_RED: u8 = 16;
pub const FLOWER_YELLOW: u8 = 17;

pub const BLOCK_COUNT: usize = 18;

pub struct BlockDef {
    pub name: &'static str,
    /// [top, bottom, side] tile indices
    pub tiles: [u16; 3],
    /// collides with entities
    pub solid: bool,
    /// blocks skylight & culls neighbor faces
    pub opaque: bool,
    /// rendered as X-shaped plant
    pub cross: bool,
    pub fluid: bool,
    pub sound: SoundFamily,
}

pub const BLOCK_TABLE: [BlockDef; BLOCK_COUNT] = [
    BlockDef { name: "Air", tiles: [0, 0, 0], solid: false, opaque: false, cross: false, fluid: false, sound: SoundFamily::None },
    BlockDef { name: "Grass Block", tiles: [TILE_GRASS_TOP, TILE_DIRT, TILE_GRASS_SIDE], solid: true, opaque: true, cross: false, fluid: false, sound: SoundFamily::Grass },
    BlockDef { name: "Dirt", tiles: [TILE_DIRT, TILE_DIRT, TILE_DIRT], solid: true, opaque: true, cross: false, fluid: false, sound: SoundFamily::Dirt },
    BlockDef { name: "Stone", tiles: [TILE_STONE, TILE_STONE, TILE_STONE], solid: true, opaque: true, cross: false, fluid: false, sound: SoundFamily::Stone },
    BlockDef { name: "Cobblestone", tiles: [TILE_COBBLE, TILE_COBBLE, TILE_COBBLE], solid: true, opaque: true, cross: false, fluid: false, sound: SoundFamily::Stone },
    BlockDef { name: "Sand", tiles: [TILE_SAND, TILE_SAND, TILE_SAND], solid: true, opaque: true, cross: false, fluid: false, sound: SoundFamily::Sand },
    BlockDef { name: "Oak Log", tiles: [TILE_LOG_TOP, TILE_LOG_TOP, TILE_LOG_SIDE], solid: true, opaque: true, cross: false, fluid: false, sound: SoundFamily::Wood },
    BlockDef { name: "Oak Planks", tiles: [TILE_PLANKS, TILE_PLANKS, TILE_PLANKS], solid: true, opaque: true, cross: false, fluid: false, sound: SoundFamily::Wood },
    BlockDef { name: "Oak Leaves", tiles: [TILE_LEAVES, TILE_LEAVES, TILE_LEAVES], solid: true, opaque: false, cross: false, fluid: false, sound: SoundFamily::Leaves },
    BlockDef { name: "Water", tiles: [TILE_WATER, TILE_WATER, TILE_WATER], solid: false, opaque: false, cross: false, fluid: true, sound: SoundFamily::Water },
    BlockDef { name: "Glass", tiles: [TILE_GLASS, TILE_GLASS, TILE_GLASS], solid: true, opaque: false, cross: false, fluid: false, sound: SoundFamily::Glass },
    BlockDef { name: "Bedrock", tiles: [TILE_BEDROCK, TILE_BEDROCK, TILE_BEDROCK], solid: true, opaque: true, cross: false, fluid: false, sound: SoundFamily::Stone },
    BlockDef { name: "Gravel", tiles: [TILE_GRAVEL, TILE_GRAVEL, TILE_GRAVEL], solid: true, opaque: true, cross: false, fluid: false, sound: SoundFamily::Sand },
    BlockDef { name: "Snow Block", tiles: [TILE_SNOW, TILE_SNOW, TILE_SNOW], solid: true, opaque: true, cross: false, fluid: false, sound: SoundFamily::Sand },
    BlockDef { name: "Snowy Grass", tiles: [TILE_SNOW, TILE_DIRT, TILE_SNOW_SIDE], solid: true, opaque: true, cross: false, fluid: false, sound: SoundFamily::Grass },
    BlockDef { name: "Grass", tiles: [TILE_TALL_GRASS, TILE_TALL_GRASS, TILE_TALL_GRASS], solid: false, opaque: false, cross: true, fluid: false, sound: SoundFamily::Grass },
    BlockDef { name: "Poppy", tiles: [TILE_FLOWER_RED, TILE_FLOWER_RED, TILE_FLOWER_RED], solid: false, opaque: false, cross: true, fluid: false, sound: SoundFamily::Grass },
    BlockDef { name: "Dandelion", tiles: [TILE_FLOWER_YELLOW, TILE_FLOWER_YELLOW, TILE_FLOWER_YELLOW], solid: false, opaque: false, cross: true, fluid: false, sound: SoundFamily::Grass },
];

#[inline]
pub fn def(id: u8) -> &'static BlockDef {
    &BLOCK_TABLE[id as usize]
}

#[inline]
pub fn is_solid(id: u8) -> bool {
    def(id).solid
}

#[inline]
pub fn is_opaque(id: u8) -> bool {
    def(id).opaque
}

#[inline]
pub fn is_cross(id: u8) -> bool {
    def(id).cross
}

#[inline]
pub fn is_fluid(id: u8) -> bool {
    def(id).fluid
}

#[inline]
pub fn name(id: u8) -> &'static str {
    def(id).name
}

/// Should a face of `b` facing neighbor `n` be rendered?
#[inline]
pub fn face_visible(b: u8, n: u8) -> bool {
    if b == AIR {
        return false;
    }
    if b == WATER {
        // water visible through non-water, non-opaque neighbors (air, glass, plants)
        return !is_opaque(n) && n != WATER;
    }
    if b == LEAVES {
        // "fancy" leaves: render even against other leaves
        return !is_opaque(n);
    }
    if b == GLASS {
        return !is_opaque(n) && n != GLASS;
    }
    // fully opaque blocks
    !is_opaque(n)
}

/// Blocks placeable from the hotbar.
pub const PALETTE: [u8; 9] = [GRASS, DIRT, STONE, COBBLE, PLANKS, OAK_LOG, LEAVES, SAND, GLASS];
