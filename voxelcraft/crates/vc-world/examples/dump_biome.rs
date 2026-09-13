//! Diagnostic 2: find a given biome and dump its decoration counts.

use vc_blocks::blocks::*;
use vc_world::gen::{Biome, TerrainGen};

fn main() {
    let seed: u64 = std::env::args()
        .nth(1)
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(12345);
    let want = std::env::args().nth(2).unwrap_or_else(|| "Savanna".into());
    let g = TerrainGen::new(seed);

    let target = match want.as_str() {
        "Savanna" => Biome::Savanna,
        "Jungle" => Biome::Jungle,
        "DarkForest" => Biome::DarkForest,
        "FlowerForest" => Biome::FlowerForest,
        "Desert" => Biome::Desert,
        "Mountains" => Biome::Mountains,
        "Badlands" => Biome::Badlands,
        "FrozenOcean" => Biome::FrozenOcean,
        "Swamp" => Biome::Swamp,
        "Taiga" => Biome::Taiga,
        _ => Biome::Plains,
    };

    // scan for the biome (chunk centers), generate, count decorations
    let mut n_scanned = 0usize;
    'search: for cx in -64..64i32 {
        for cz in -64..64i32 {
            n_scanned += 1;
            // exact replica of the v172 test find_biome: column()-based, cx outer
            let col = g.column(cx * 16 + 8, cz * 16 + 8);
            if col.biome != target {
                continue;
            }
            let (chunk, _) = g.generate_chunk(cx, cz, Vec::new());
            let mut logs = 0usize;
            let mut leaves = 0usize;
            let mut grass_cols = 0usize;
            let mut coarse = 0usize;
            let mut heights = std::collections::BTreeMap::new();
            for i in 0..(16usize * 16) {
                let h = chunk.height[i] as i32;
                *heights.entry(h / 10).or_insert(0usize) += 1;
                let b = chunk.get(i % 16, h as usize, i / 16);
                match b {
                    GRASS | SNOW_GRASS => grass_cols += 1,
                    COARSE_DIRT => coarse += 1,
                    _ => {}
                }
            }
            let mut flora = 0usize;
            let mut tall_grass = 0usize;
            let mut biome_count = std::collections::BTreeMap::new();
            for i in 0..256usize {
                *biome_count.entry(Biome::from_u8(chunk.biome[i]).name().to_string()).or_insert(0usize) += 1;
            }
            for i in 0..(16usize * 16 * 200) {
                let b = chunk.get_idx(i);
                match b {
                    OAK_LOG | SPRUCE_LOG | BIRCH_LOG | JUNGLE_LOG | ACACIA_LOG | DARK_OAK_LOG => {
                        logs += 1
                    }
                    LEAVES | SPRUCE_LEAVES | BIRCH_LEAVES | JUNGLE_LEAVES | ACACIA_LEAVES
                    | DARK_OAK_LEAVES => leaves += 1,
                    ALLIUM | OXEYE_DAISY | CORNFLOWER | LILY_OF_THE_VALLEY
                    | ORANGE_TULIP | RED_TULIP | WHITE_TULIP | PINK_TULIP
                    | PEONY | ROSE_BUSH | LILAC | SUNFLOWER | FLOWER_RED
                    | FLOWER_YELLOW | AZURE_BLUET | BLUE_ORCHID => flora += 1,
                    TALL_GRASS => tall_grass += 1,
                    _ => {}
                }
            }
            println!("  flora={flora} tall_grass={tall_grass} biome census: {biome_count:?}");
            println!(
                "{want} found at chunk ({cx},{cz}) after {n_scanned} scans: \
                 logs={logs} leaves={leaves} grass_cols={grass_cols} coarse={coarse}"
            );
            println!("  height histogram (tens): {:?}", heights);
            // also dump the center column
            let h = chunk.height[8 * 16 + 8] as usize;
            for dy in 0..4.min(h + 1) {
                let y = h - dy;
                println!("  center y={y}: {}", chunk.get(8, y, 8));
            }
            // quantify column() vs chunk.height disagreement
            let mut diffs: Vec<i32> = Vec::new();
            for lz in 0..16usize {
                for lx in 0..16usize {
                    let c = g.column(cx * 16 + lx as i32, cz * 16 + lz as i32);
                    let ch = chunk.height[lz * 16 + lx] as i32;
                    diffs.push(c.height - ch);
                }
            }
            let mut big = 0;
            let mut sum = 0i64;
            for d in &diffs {
                sum += d.abs() as i64;
                if d.abs() > 3 {
                    big += 1;
                }
            }
            println!(
                "  column-vs-chunk: avg|diff|={} max|diff|={} n>3: {}/256",
                sum / 256,
                diffs.iter().map(|d| d.abs()).max().unwrap(),
                big
            );
            break 'search;
        }
    }
}
