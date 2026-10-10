//! 4.1a: biome census probe — prints `vanilla_id name count` rows
//! over a chunk rect for the 4.0 numeric-diff exchange (facts only).
//! 4.3c: `height` mode prints `vanilla_id count mean_height`.
//! Usage: census [seed] [cx0] [cz0] [w] [h] [dim] [mode]
//! (defaults 0 0 0 8 8 overworld census)

use vc_world::gen::TerrainGen;
use vc_world::world::Dimension;

fn main() {
    let arg = |i: usize, d: i64| {
        std::env::args()
            .nth(i)
            .and_then(|s| s.parse::<i64>().ok())
            .unwrap_or(d)
    };
    let seed = arg(1, 0) as u64;
    let (cx0, cz0, w, h) = (
        arg(2, 0) as i32,
        arg(3, 0) as i32,
        arg(4, 8) as i32,
        arg(5, 8) as i32,
    );
    let dim = match std::env::args().nth(6).as_deref() {
        Some("nether") => Dimension::Nether,
        Some("end") => Dimension::End,
        _ => Dimension::Overworld,
    };
    let g = TerrainGen::for_dimension(seed, dim);
    // 4.1o: `hist` mode prints the land-height histogram + cumulative
    // shares at candidate gates (relief fitting: mountain/beach/deep).
    if std::env::args().nth(7).as_deref() == Some("hist") {
        use std::collections::BTreeMap;
        let mut m: BTreeMap<i32, u64> = BTreeMap::new();
        for cx in cx0..cx0 + w {
            for cz in cz0..cz0 + h {
                for lx in 0..16 {
                    for lz in 0..16 {
                        let c = g.column(cx * 16 + lx, cz * 16 + lz);
                        *m.entry(c.height).or_default() += 1;
                    }
                }
            }
        }
        let total: u64 = m.values().sum();
        println!("seed={seed} rect=({cx0},{cz0},{w},{h}) hist total={total}");
        for (hh, n) in &m {
            println!("h{hh}: {n}");
        }
        for gate in [70, 72, 74, 76, 78, 80, 82, 84, 88, 96] {
            let above: u64 = m.iter().filter(|(hh, _)| **hh > gate).map(|(_, n)| n).sum();
            println!("P(h>{gate})={:.2}%", 100.0 * above as f64 / total as f64);
        }
        for depth in [4, 6, 8, 10] {
            let below: u64 = m
                .iter()
                .filter(|(hh, _)| **hh < 62 - depth)
                .map(|(_, n)| n)
                .sum();
            println!(
                "P(h<{})={:.2}%",
                62 - depth,
                100.0 * below as f64 / total as f64
            );
        }
        return;
    }
    if std::env::args().nth(7).as_deref() == Some("height") {
        println!("seed={seed} rect=({cx0},{cz0},{w},{h}) heights");
        for (id, n, mean) in g.height_stats(cx0, cz0, w, h) {
            println!("{id}: {n} {mean:.1}");
        }
        return;
    }
    let rows = g.biome_census(cx0, cz0, w, h);
    println!("seed={seed} rect=({cx0},{cz0},{w},{h})");
    for (id, n) in &rows {
        println!("{id}: {n}");
    }
    println!("total={}", rows.iter().map(|(_, n)| n).sum::<u64>());
}
