//! Diagnostic 3: structure-site gates under the new terrain (pyramids,
//! mansions, fossils) — counts how many candidate regions pass.

use vc_world::gen::{Biome, TerrainGen};

fn main() {
    let g = TerrainGen::for_dimension(0x10C0_C0DE, vc_world::world::Dimension::Overworld);

    // desert height census
    let mut desert_cols = 0usize;
    let mut desert_above_sea = 0usize;
    let mut heights = Vec::new();
    for cz in -64..64i32 {
        for cx in -64..64i32 {
            let c = g.column(cx * 16 + 8, cz * 16 + 8);
            if c.biome == Biome::Desert {
                desert_cols += 1;
                heights.push(c.height);
                if c.height > 64 {
                    desert_above_sea += 1;
                }
            }
        }
    }
    heights.sort();
    println!(
        "desert columns in ±64 chunks: {desert_cols}, above-sea: {desert_above_sea}, \
         h median {:?}",
        heights.get(heights.len() / 2)
    );

    // pyramid regions passing the gate, ±16 regions
    let mut ok = 0usize;
    for rx in -16..16i32 {
        for rz in -16..16i32 {
            if g.pyramid_center_pub(rx, rz).is_some() {
                ok += 1;
            }
        }
    }
    println!("pyramid sites passing the gate in ±16 regions: {ok}");

    // dark forest census (mansion gate context)
    let mut dark = 0usize;
    let mut dark_above = 0usize;
    for cz in -64..64i32 {
        for cx in -64..64i32 {
            let c = g.column(cx * 16 + 8, cz * 16 + 8);
            if c.biome == Biome::DarkForest {
                dark += 1;
                if c.height > 65 {
                    dark_above += 1;
                }
            }
        }
    }
    println!("dark-forest columns in ±64 chunks: {dark}, h>65: {dark_above}");

    // frozen ocean census (iceberg gate context)
    let mut frozen = 0usize;
    for cz in -64..64i32 {
        for cx in -64..64i32 {
            let c = g.column(cx * 16 + 8, cz * 16 + 8);
            if c.biome == Biome::FrozenOcean {
                frozen += 1;
            }
        }
    }
    println!("frozen-ocean columns in ±64 chunks: {frozen}");

    // climate field distribution (dark-forest bracket diagnosis)
    let mut t_hist = [0usize; 10];
    let mut h_hist = [0usize; 10];
    let mut land = 0usize;
    for cz in -64..64i32 {
        for cx in -64..64i32 {
            let c = g.column(cx * 16 + 8, cz * 16 + 8);
            if !c.biome.is_ocean() && c.biome != Biome::Beach && c.biome != Biome::River {
                land += 1;
            }
        }
    }
    // re-derive the fields through a Biome probe: sample raw columns
    for cz in -8..8i32 {
        for cx in -8..8i32 {
            let c = g.column(cx * 16 + 8, cz * 16 + 8);
            let _ = &c;
        }
    }
    println!("land columns in ±64 chunks: {land} / 16384");

    // climate field std (biome-spread diagnosis)
    let mut ts = Vec::new();
    let mut hs = Vec::new();
    let mut vs = Vec::new();
    for cz in -64..64i32 {
        for cx in -64..64i32 {
            let (t, h, v) = g.climate_fields(cx * 16 + 8, cz * 16 + 8);
            ts.push(t as f64);
            hs.push(h as f64);
            vs.push(v as f64);
        }
    }
    for (name, xs) in [("temp", &ts), ("humid", &hs), ("var", &vs)] {
        let n = xs.len() as f64;
        let mean = xs.iter().sum::<f64>() / n;
        let var = xs.iter().map(|x| (x - mean) * (x - mean)).sum::<f64>() / n;
        let std = var.sqrt();
        let frac = |lo: f64, hi: f64| {
            xs.iter().filter(|&&x| x > lo && x <= hi).count() as f64 / n * 100.0
        };
        println!(
            "  {name}: mean={mean:.3} std={std:.3} | P(>0.2)={:.1}% P(>0.3)={:.1}% P(>0.32)={:.1}%",
            frac(0.2, 9.0),
            frac(0.3, 9.0),
            frac(0.32, 9.0)
        );
    }

    // mountains census (emerald test context)
    let mut mtn = 0usize;
    for cz in -64..64i32 {
        for cx in -64..64i32 {
            let c = g.column(cx * 16 + 8, cz * 16 + 8);
            if c.biome == Biome::Mountains {
                mtn += 1;
            }
        }
    }
    println!("mountains columns in ±64 chunks: {mtn}");
}
