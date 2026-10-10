//! 4.1r: river-field probe — the |rv| distribution, solved-height
//! profile across the carve, and river-biome share on a coarse block
//! grid (facts only: counts, no world content).
//! Usage: riverstat [seed] [x0] [z0] [w] [h] [step]

use vc_world::gen::{Biome, TerrainGen};
use vc_world::world::Dimension;

fn main() {
    let arg = |i: usize, d: i64| {
        std::env::args()
            .nth(i)
            .and_then(|s| s.parse::<i64>().ok())
            .unwrap_or(d)
    };
    let seed = arg(1, 0) as u64;
    let (x0, z0, w, h, step) = (
        arg(2, 0) as i32,
        arg(3, 0) as i32,
        arg(4, 2048) as i32,
        arg(5, 2048) as i32,
        arg(6, 8) as i32,
    );
    let g = TerrainGen::for_dimension(seed, Dimension::Overworld);
    // bands: |rv| edges + per-band (count, h<=62 count, river count)
    let edges = [0.01, 0.02, 0.03, 0.045, 0.06, 0.09, 0.15, 1.0];
    let mut n = [0u64; 8];
    let mut wet = [0u64; 8];
    let mut riv = [0u64; 8];
    let mut total = 0u64;
    let mut x = x0;
    while x < x0 + w {
        let mut z = z0;
        while z < z0 + h {
            let rv = g.river_field(x, z).abs();
            let c = g.column(x, z);
            let b = edges.iter().position(|e| rv < *e).unwrap_or(7);
            n[b] += 1;
            if c.height <= 62 {
                wet[b] += 1;
            }
            if c.biome == Biome::River {
                riv[b] += 1;
            }
            total += 1;
            z += step;
        }
        x += step;
    }
    println!("seed={seed} total={total}");
    for i in 0..8 {
        println!(
            "rv<{:.3}: n={} share={:.2}% wet={:.0}% river={}",
            edges[i],
            n[i],
            100.0 * n[i] as f64 / total as f64,
            100.0 * wet[i] as f64 / n[i].max(1) as f64,
            riv[i]
        );
    }
}
