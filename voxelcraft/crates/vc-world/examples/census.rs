//! 4.1a: biome census probe — prints `vanilla_id name count` rows
//! over a chunk rect for the 4.0 numeric-diff exchange (facts only).
//! Usage: census [seed] [cx0] [cz0] [w] [h] [dim]  (defaults 0 0 0 8 8 overworld)

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
    let rows = g.biome_census(cx0, cz0, w, h);
    println!("seed={seed} rect=({cx0},{cz0},{w},{h})");
    for (id, n) in &rows {
        println!("{id}: {n}");
    }
    println!("total={}", rows.iter().map(|(_, n)| n).sum::<u64>());
}
