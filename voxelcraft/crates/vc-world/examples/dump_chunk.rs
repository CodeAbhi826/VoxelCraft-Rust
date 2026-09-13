//! Diagnostic: dump chunk state after generation (heights, surfaces,
//! biome, decoration counts) for the vanilla-parity terrain round.

use vc_blocks::blocks::*;
use vc_world::gen::{Biome, TerrainGen};

fn main() {
    let seed = std::env::args()
        .nth(1)
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(12345);
    let g = TerrainGen::new(seed);

    // census over 12x12 chunk centers
    let mut rows: Vec<(i32, i32, Biome, i32)> = Vec::new();
    for cz in -6..6i32 {
        for cx in -6..6i32 {
            let col = g.column(cx * 16 + 8, cz * 16 + 8);
            rows.push((cx, cz, col.biome, col.height));
        }
    }
    let mut counts: std::collections::BTreeMap<String, (usize, i64)> = std::collections::BTreeMap::new();
    for (_, _, b, h) in &rows {
        let e = counts.entry(b.name().to_string()).or_insert((0, 0));
        e.0 += 1;
        e.1 += *h as i64;
    }
    println!("== biome census (12x12 chunk centers, seed {seed}) ==");
    for (name, (n, sum)) in &counts {
        println!("  {name:16} n={n:3} avg_h={}", sum / *n as i64);
    }

    // generate one land chunk and dump its state
    let target = rows
        .iter()
        .find(|(_, _, b, _)| *b == Biome::Forest)
        .or_else(|| rows.iter().find(|(_, _, b, _)| *b == Biome::Plains))
        .or_else(|| rows.iter().find(|(_, _, b, _)| !b.is_ocean()));
    let (cx, cz) = match target {
        Some(&(cx, cz, _, _)) => (cx, cz),
        None => (0, 0),
    };
    let t0 = std::time::Instant::now();
    let (chunk, _) = g.generate_chunk(cx, cz, Vec::new());
    let dt = t0.elapsed();
    println!(
        "\n== chunk ({cx},{cz}) generated in {:.1} ms ==",
        dt.as_secs_f64() * 1000.0
    );

    let mut logs = 0usize;
    let mut leaves = 0usize;
    let mut grass_blocks = 0usize;
    let mut coal = 0usize;
    let mut iron = 0usize;
    let mut diamond = 0usize;
    let mut lava = 0usize;
    let mut underground_air = 0usize;
    for i in 0..(16usize * 16 * 256) {
        let b = chunk.get_idx(i);
        match b {
            OAK_LOG | SPRUCE_LOG | BIRCH_LOG | JUNGLE_LOG | ACACIA_LOG | DARK_OAK_LOG => logs += 1,
            LEAVES | SPRUCE_LEAVES | BIRCH_LEAVES | JUNGLE_LEAVES | ACACIA_LEAVES
            | DARK_OAK_LEAVES => leaves += 1,
            GRASS => grass_blocks += 1,
            COAL_ORE => coal += 1,
            IRON_ORE => iron += 1,
            DIAMOND_ORE => diamond += 1,
            LAVA => lava += 1,
            0 => {
                // air below y 40 = cave
                let y = (i / (16 * 16)) as i32;
                if y < 40 {
                    underground_air += 1;
                }
            }
            _ => {}
        }
    }
    let col = 8usize * 16 + 8;
    let h = chunk.height[col] as usize;
    println!("height[center] = {h}");
    println!("biome[center]  = {:?}", Biome::from_u8(chunk.biome[col]));
    for dy in 0..6.min(h + 1) {
        let y = h - dy;
        println!("  center y={y}: block={}", chunk.get(8, y, 8));
    }
    println!(
        "logs={logs} leaves={leaves} grass={grass_blocks} coal={coal} iron={iron} \
         diamond={diamond} lava={lava} cave_air_below40={underground_air}"
    );
}
