// Block type registry. Each block has an id, name, and properties.
// Adding a new block = one enum variant + one match arm in properties().

use bytemuck::{Pod, Zeroable};

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Block {
    Air = 0,
    Grass = 1,
    Dirt = 2,
    Stone = 3,
    Cobblestone = 4,
    Wood = 5,
    Leaves = 6,
    Sand = 7,
    Water = 8,
    Planks = 9,
    Bedrock = 10,
    Snow = 11,
    Glass = 12,
    Brick = 13,
    CoalOre = 14,
    IronOre = 15,
    GoldOre = 16,
    DiamondOre = 17,
}

impl Block {
    pub const ALL: &'static [Block] = &[
        Block::Air, Block::Grass, Block::Dirt, Block::Stone,
        Block::Cobblestone, Block::Wood, Block::Leaves, Block::Sand,
        Block::Water, Block::Planks, Block::Bedrock, Block::Snow,
        Block::Glass, Block::Brick, Block::CoalOre, Block::IronOre,
        Block::GoldOre, Block::DiamondOre,
    ];

    pub fn from_u8(v: u8) -> Self {
        unsafe { std::mem::transmute(v) }
    }

    pub fn name(self) -> &'static str {
        match self {
            Block::Air => "Air",
            Block::Grass => "Grass",
            Block::Dirt => "Dirt",
            Block::Stone => "Stone",
            Block::Cobblestone => "Cobblestone",
            Block::Wood => "Oak Log",
            Block::Leaves => "Leaves",
            Block::Sand => "Sand",
            Block::Water => "Water",
            Block::Planks => "Oak Planks",
            Block::Bedrock => "Bedrock",
            Block::Snow => "Snow",
            Block::Glass => "Glass",
            Block::Brick => "Brick",
            Block::CoalOre => "Coal Ore",
            Block::IronOre => "Iron Ore",
            Block::GoldOre => "Gold Ore",
            Block::DiamondOre => "Diamond Ore",
        }
    }

    /// Texture tile index for each face: [top, bottom, side]
    pub fn textures(self) -> [u8; 3] {
        match self {
            Block::Air => [0; 3],
            Block::Grass => [0, 2, 1],       // grass_top, dirt, grass_side
            Block::Dirt => [2, 2, 2],
            Block::Stone => [3, 3, 3],
            Block::Cobblestone => [4, 4, 4],
            Block::Wood => [5, 5, 6],        // log_top, log_top, log_side
            Block::Leaves => [7, 7, 7],
            Block::Sand => [8, 8, 8],
            Block::Water => [9, 9, 9],
            Block::Planks => [10, 10, 10],
            Block::Bedrock => [11, 11, 11],
            Block::Snow => [12, 2, 12],
            Block::Glass => [13, 13, 13],
            Block::Brick => [14, 14, 14],
            Block::CoalOre => [15, 15, 15],
            Block::IronOre => [16, 16, 16],
            Block::GoldOre => [17, 17, 17],
            Block::DiamondOre => [18, 18, 18],
        }
    }

    pub fn properties(self) -> BlockProperties {
        match self {
            Block::Air => BlockProperties { solid: false, opaque: false, liquid: false, hardness: 0.0 },
            Block::Grass => BlockProperties { solid: true, opaque: true, liquid: false, hardness: 0.6 },
            Block::Dirt => BlockProperties { solid: true, opaque: true, liquid: false, hardness: 0.5 },
            Block::Stone => BlockProperties { solid: true, opaque: true, liquid: false, hardness: 1.5 },
            Block::Cobblestone => BlockProperties { solid: true, opaque: true, liquid: false, hardness: 2.0 },
            Block::Wood => BlockProperties { solid: true, opaque: true, liquid: false, hardness: 1.0 },
            Block::Leaves => BlockProperties { solid: true, opaque: false, liquid: false, hardness: 0.2 },
            Block::Sand => BlockProperties { solid: true, opaque: true, liquid: false, hardness: 0.5 },
            Block::Water => BlockProperties { solid: false, opaque: false, liquid: true, hardness: f32::INFINITY },
            Block::Planks => BlockProperties { solid: true, opaque: true, liquid: false, hardness: 1.0 },
            Block::Bedrock => BlockProperties { solid: true, opaque: true, liquid: false, hardness: f32::INFINITY },
            Block::Snow => BlockProperties { solid: true, opaque: true, liquid: false, hardness: 0.3 },
            Block::Glass => BlockProperties { solid: true, opaque: false, liquid: false, hardness: 0.4 },
            Block::Brick => BlockProperties { solid: true, opaque: true, liquid: false, hardness: 2.0 },
            Block::CoalOre => BlockProperties { solid: true, opaque: true, liquid: false, hardness: 3.0 },
            Block::IronOre => BlockProperties { solid: true, opaque: true, liquid: false, hardness: 3.0 },
            Block::GoldOre => BlockProperties { solid: true, opaque: true, liquid: false, hardness: 3.0 },
            Block::DiamondOre => BlockProperties { solid: true, opaque: true, liquid: false, hardness: 3.5 },
        }
    }

    pub fn is_solid(self) -> bool { self.properties().solid }
    pub fn is_opaque(self) -> bool { self.properties().opaque }
    pub fn is_liquid(self) -> bool { self.properties().liquid }
    pub fn is_air(self) -> bool { self == Block::Air }

    /// Hotbar blocks for creative mode
    pub const HOTBAR: &'static [Block] = &[
        Block::Grass, Block::Dirt, Block::Stone, Block::Cobblestone,
        Block::Wood, Block::Planks, Block::Leaves, Block::Sand, Block::Glass,
    ];
}

#[derive(Debug, Clone, Copy)]
pub struct BlockProperties {
    pub solid: bool,
    pub opaque: bool,
    pub liquid: bool,
    pub hardness: f32,
}

/// Should a face between `self_block` and `neighbor` be rendered?
pub fn should_render_face(self_block: Block, neighbor: Block) -> bool {
    if neighbor.is_air() {
        return true;
    }
    // No internal faces between same-type transparent blocks (water-water, glass-glass)
    if neighbor == self_block && !neighbor.is_opaque() {
        return false;
    }
    // Render face if neighbor is transparent (non-opaque) and different type
    !neighbor.is_opaque()
}

// Vertex type matching the wgpu vertex buffer layout.
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct ChunkVertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
    pub color: [f32; 3],   // AO + face shading tint
    pub tex_layer: u32,    // texture array layer
}
