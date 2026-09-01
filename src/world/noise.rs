// Procedural terrain noise: multi-octave simplex for continents + hills + detail.
// Plus caves and ore sprinkling.

use noise::{NoiseFn, Perlin};
use crate::blocks::Block;
use crate::world::chunk::{Chunk, CHUNK_SIZE, WORLD_HEIGHT, SEA_LEVEL};

pub struct TerrainNoise {
    continentalness: Perlin,
    hills: Perlin,
    detail: Perlin,
    cave: Perlin,
    ore: Perlin,
    tree: Perlin,
}

impl TerrainNoise {
    pub fn new(seed: u32) -> Self {
        Self {
            continentalness: Perlin::new(seed),
            hills: Perlin::new(seed.wrapping_add(1)),
            detail: Perlin::new(seed.wrapping_add(2)),
            cave: Perlin::new(seed.wrapping_add(3)),
            ore: Perlin::new(seed.wrapping_add(4)),
            tree: Perlin::new(seed.wrapping_add(5)),
        }
    }

    /// Sample terrain height at world (wx, wz).
    pub fn height(&self, wx: i32, wz: i32) -> i32 {
        let xf = wx as f64;
        let zf = wz as f64;
        let continent = self.continentalness.get([xf * 0.005, zf * 0.005]);
        let hills = self.hills.get([xf * 0.02, zf * 0.02]);
        let detail = self.detail.get([xf * 0.08, zf * 0.08]);

        let h = SEA_LEVEL as f64 + 4.0
            + continent * 14.0
            + hills * 6.0
            + detail * 2.0;

        let h = h.max(2.0).min((WORLD_HEIGHT - 4) as f64);
        h as i32
    }

    /// Is there a cave at this position? (carve away the block if so)
    pub fn is_cave(&self, wx: i32, wy: i32, wz: i32) -> bool {
        if wy <= 1 || wy >= WORLD_HEIGHT as i32 - 4 {
            return false;
        }
        let n = self.cave.get([
            wx as f64 * 0.08 + wy as f64 * 0.05,
            wz as f64 * 0.08 - wy as f64 * 0.05,
        ]);
        n > 0.78
    }

    /// Ore type at a position, or None.
    pub fn ore_at(&self, wx: i32, wy: i32, wz: i32) -> Option<Block> {
        let n = self.ore.get([wx as f64 * 0.6, (wz as f64 + wy as f64 * 17.0) * 0.6]);
        if wy < 12 && n > 0.85 { Some(Block::DiamondOre) }
        else if wy < 24 && n > 0.82 { Some(Block::GoldOre) }
        else if n > 0.78 { Some(Block::IronOre) }
        else if n > 0.70 { Some(Block::CoalOre) }
        else { None }
    }

    /// Should a tree spawn here? (low probability)
    pub fn has_tree(&self, wx: i32, wz: i32) -> bool {
        self.tree.get([wx as f64 * 1.3, wz as f64 * 1.3]) > 0.86
    }
}

pub struct WorldGenerator {
    pub noise: TerrainNoise,
}

impl WorldGenerator {
    pub fn new(seed: u32) -> Self {
        Self { noise: TerrainNoise::new(seed) }
    }

    /// Fill a chunk with terrain.
    pub fn generate(&self, chunk: &mut Chunk) {
        let base_x = chunk.cx * CHUNK_SIZE as i32;
        let base_z = chunk.cz * CHUNK_SIZE as i32;

        for lx in 0..CHUNK_SIZE {
            for lz in 0..CHUNK_SIZE {
                let wx = base_x + lx as i32;
                let wz = base_z + lz as i32;
                let height = self.noise.height(wx, wz).max(0) as usize;

                for y in 0..=height.min(WORLD_HEIGHT - 1) {
                    let mut block = Block::Stone;
                    if y == 0 {
                        block = Block::Bedrock;
                    } else if y == height {
                        if height <= SEA_LEVEL + 1 {
                            block = Block::Sand;
                        } else if height > SEA_LEVEL + 18 {
                            block = Block::Snow;
                        } else {
                            block = Block::Grass;
                        }
                    } else if y >= height.saturating_sub(3) {
                        if height <= SEA_LEVEL + 1 {
                            block = Block::Sand;
                        } else {
                            block = Block::Dirt;
                        }
                    } else {
                        if let Some(ore) = self.noise.ore_at(wx, y as i32, wz) {
                            block = ore;
                        }
                    }

                    // Carve caves
                    if y > 1 && y < height {
                        if self.noise.is_cave(wx, y as i32, wz) {
                            continue;
                        }
                    }

                    chunk.set(lx, y, lz, block);
                }

                // Fill water up to sea level
                for y in (height + 1)..=SEA_LEVEL.min(WORLD_HEIGHT - 1) {
                    chunk.set(lx, y, lz, Block::Water);
                }

                // Plant trees on grass
                if height > SEA_LEVEL + 1 && height < WORLD_HEIGHT - 8 {
                    if chunk.get(lx, height, lz) == Block::Grass && self.noise.has_tree(wx, wz) {
                        self.plant_tree(chunk, lx, height + 1, lz);
                    }
                }
            }
        }
        chunk.dirty = true;
    }

    fn plant_tree(&self, chunk: &mut Chunk, lx: usize, base_y: usize, lz: usize) {
        let trunk_h = 4 + (self.noise.tree.get([lx as f64 * 7.7, lz as f64 * 7.7]) + 1.0) as usize * 2;
        for i in 0..trunk_h {
            let y = base_y + i;
            if y >= WORLD_HEIGHT { break; }
            chunk.set(lx, y, lz, Block::Wood);
        }
        let top_y = base_y + trunk_h;
        // Leaf canopy
        for dy in 1..=3 {
            let r = if dy <= 1 { 2 } else { 1 };
            for dx in -(r as i32)..=r as i32 {
                for dz in -(r as i32)..=r as i32 {
                    if dx.abs() + dz.abs() > r + 1 { continue; }
                    let x = lx as i32 + dx;
                    let z = lz as i32 + dz;
                    if x < 0 || x as usize >= CHUNK_SIZE || z < 0 || z as usize >= CHUNK_SIZE { continue; }
                    let y = top_y + dy;
                    if y >= WORLD_HEIGHT { continue; }
                    if chunk.get(x as usize, y, z as usize) == Block::Air {
                        chunk.set(x as usize, y, z as usize, Block::Leaves);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn terrain_height_is_reasonable() {
        let gen = WorldGenerator::new(1337);
        for (wx, wz) in [(0, 0), (100, 50), (-50, 200), (1000, -300)] {
            let h = gen.noise.height(wx, wz);
            assert!(h >= 2 && h < WORLD_HEIGHT as i32, "height {} at ({},{}) out of range", h, wx, wz);
        }
    }

    #[test]
    fn chunk_generation_produces_terrain() {
        let gen = WorldGenerator::new(1337);
        let mut chunk = Chunk::new(0, 0);
        gen.generate(&mut chunk);
        // At least some non-air blocks
        let solid_count = chunk.blocks.iter().filter(|b| !b.is_air()).count();
        assert!(solid_count > 100, "chunk has only {} solid blocks", solid_count);
        // Bedrock at y=0
        assert_eq!(chunk.get(0, 0, 0), Block::Bedrock);
    }

    #[test]
    fn water_at_sea_level_for_low_terrain() {
        let gen = WorldGenerator::new(1337);
        // Try many chunks to find one with water (low terrain)
        let mut found_water = false;
        for cx in -5..10 {
            if found_water { break; }
            for cz in -5..10 {
                if found_water { break; }
                let mut c = Chunk::new(cx, cz);
                gen.generate(&mut c);
                if c.blocks.iter().any(|b| *b == Block::Water) {
                    found_water = true;
                }
            }
        }
        assert!(found_water, "no water found in any tested chunk — terrain may be too high everywhere");
    }
}
