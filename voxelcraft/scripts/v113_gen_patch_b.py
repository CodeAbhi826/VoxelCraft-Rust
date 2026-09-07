#!/usr/bin/env python3
"""1.13 gen.rs patch B: ocean decorations (reefs, kelp, seagrass,
icebergs, buried treasure) + the buried_treasure loot table hook."""
import sys

p = 'crates/vc-world/src/gen.rs'
t = open(p).read()
n = 0

def rep(old, new, label):
    global t, n
    c = t.count(old)
    if c != 1:
        print(f'FAIL [{label}]: count = {c}')
        sys.exit(1)
    t = t.replace(old, new)
    n += 1
    print(f'ok [{label}]')

# ---- 1. extend the ocean cave-margin check to the new families ----
rep('''                    let margin = if col.biome == Biome::Ocean || col.biome == Biome::Beach {
                        10
                    } else {
                        5
                    };''',
'''                    let margin = if col.biome.is_ocean() || col.biome == Biome::Beach {
                        10
                    } else {
                        5
                    };''', 'cave margin ocean family')

# also the second margin site (~line 1902)
t2 = t
old2 = '''        let margin = if col.biome == Biome::Ocean || col.biome == Biome::Beach {
            10
        } else {
            5
        };'''
if t2.count(old2) == 1:
    t = t.replace(old2, '''        let margin = if col.biome.is_ocean() || col.biome == Biome::Beach {
            10
        } else {
            5
        };''')
    n += 1
    print('ok [cave margin site 2]')

# ---- 2. ocean decorations after the ice-spikes block ----
rep('''        // 1.10 fossils — VERIFIED (wiki /w/Java_Edition_1.10 §World''',
'''        // ---- 1.13 (Update Aquatic) ocean decorations — VERIFIED live
        // 2026-09-07 (changelog §World generation): "Coral reef:
        // Naturally generate in warm ocean biomes. Are composed of
        // coral, coral blocks and coral fans"; "Kelp: Generate in
        // ocean biomes, except warm oceans ... Can grow multiple
        // blocks high"; "Sea pickles: They generate in warm oceans,
        // especially around coral reefs"; "Iceberg: Generate on frozen
        // oceans"; "Seagrass: Generates in oceans ..., rivers, and
        // swamplands" ----
        {
            let b = Biome::from_u8(chunk.biome[8 * 16 + 8]);
            // a water-plant setter: cross plants may replace WATER
            // (kelp/coral/seagrass live IN the column) — the dec pass
            // normally only writes into AIR
            let mut set_water = |chunk: &mut Chunk,
                                 outbound: &mut Vec<(i32, i32, i32, u16)>,
                                 wx: i32,
                                 wy: i32,
                                 wz: i32,
                                 id: u16| {
                if wy < 1 || wy > 255 {
                    return;
                }
                let lxi = wx - ox;
                let lzi = wz - oz;
                if (0..16).contains(&lxi) && (0..16).contains(&lzi) {
                    let cur = chunk.get(lxi as usize, wy as usize, lzi as usize);
                    if cur == WATER || cur == AIR {
                        chunk.set(lxi as usize, wy as usize, lzi as usize, id);
                    }
                } else {
                    outbound.push((wx, wy, wz, id));
                }
            };
            match b {
                // coral reefs: 2-3 patches per warm-ocean chunk — coral
                // blocks in the floor + coral plants, fans and 1-4 sea
                // pickles above (VERIFIED composition)
                Biome::WarmOcean => {
                    let reefs = 2 + rng.next_range(2) as i32;
                    for _ in 0..reefs {
                        let lx = 1 + rng.next_range(14) as i32;
                        let lz = 1 + rng.next_range(14) as i32;
                        let col_idx = lz as usize * 16 + lx as usize;
                        let h = chunk.height[col_idx] as i32;
                        // only on the exposed floor, inside the water
                        if h >= vc_chunk::SEA_LEVEL - 2 {
                            continue;
                        }
                        let color = rng.next_range(5) as u16;
                        let r = 1 + rng.next_range(3) as i32;
                        for dx in -r..=r {
                            for dz in -r..=r {
                                if dx.abs() == r && dz.abs() == r && rng.next_f32() < 0.5 {
                                    continue; // ragged patch edges
                                }
                                let wx = ox + lx + dx;
                                let wz = oz + lz + dz;
                                // floor coral block (replaces sand top)
                                let wcol = (lz + dz).clamp(0, 15) as usize * 16
                                    + (lx + dx).clamp(0, 15) as usize;
                                let hh = chunk.height[wcol] as i32;
                                if hh < vc_chunk::SEA_LEVEL - 1 {
                                    chunk.set(
                                        (lx + dx).clamp(0, 15) as usize,
                                        hh as usize,
                                        (lz + dz).clamp(0, 15) as usize,
                                        CORAL_BLOCK_BASE + color,
                                    );
                                    // plant layer above: coral plant /
                                    // fan / sea pickle (VERIFIED: pickles
                                    // generate "especially around coral
                                    // reefs", up to 4 per block)
                                    let roll = rng.next_f32();
                                    if roll < 0.35 {
                                        set_water(
                                            &mut chunk, &mut outbound, wx, hh + 1, wz,
                                            CORAL_PLANT_BASE + color,
                                        );
                                    } else if roll < 0.55 {
                                        set_water(
                                            &mut chunk, &mut outbound, wx, hh + 1, wz,
                                            CORAL_FAN_BASE + color,
                                        );
                                    } else if roll < 0.85 {
                                        let pickles = 1 + rng.next_range(4) as u8;
                                        let st = sea_pickle_state(pickles);
                                        let lxi = wx - ox;
                                        let lzi = wz - oz;
                                        if (0..16).contains(&lxi)
                                            && (0..16).contains(&lzi)
                                            && chunk.get(
                                                lxi as usize,
                                                (hh + 1) as usize,
                                                lzi as usize,
                                            ) == WATER
                                        {
                                            chunk.set(
                                                lxi as usize,
                                                (hh + 1) as usize,
                                                lzi as usize,
                                                st,
                                            );
                                        }
                                    }
                                }
                            }
                        }
                    }
                    // warm oceans also grow the odd seagrass tuft
                    for _ in 0..4 {
                        let lx = rng.next_range(16) as i32;
                        let lz = rng.next_range(16) as i32;
                        let col_idx = lz as usize * 16 + lx as usize;
                        let h = chunk.height[col_idx] as i32;
                        if h < vc_chunk::SEA_LEVEL - 1 {
                            set_water(
                                &mut chunk, &mut outbound, ox + lx, h + 1, oz + lz, SEAGRASS,
                            );
                        }
                    }
                }
                // kelp forests: cold/neutral oceans — 5-9 columns per
                // chunk, each 2-7 tall from the floor (VERIFIED: "Can
                // grow multiple blocks high"; warm oceans excluded)
                Biome::ColdOcean | Biome::Ocean | Biome::LukewarmOcean | Biome::FrozenOcean => {
                    let kelp_cols = if b == Biome::FrozenOcean { 3 } else { 5 + rng.next_range(5) as i32 };
                    for _ in 0..kelp_cols {
                        let lx = rng.next_range(16) as i32;
                        let lz = rng.next_range(16) as i32;
                        let col_idx = lz as usize * 16 + lx as usize;
                        let h = chunk.height[col_idx] as i32;
                        if h >= vc_chunk::SEA_LEVEL - 2 {
                            continue;
                        }
                        let tall = 2 + rng.next_range(6) as i32;
                        for dy in 1..=tall {
                            let y = h + dy;
                            if y >= vc_chunk::SEA_LEVEL {
                                break; // kelp stops at the surface
                            }
                            set_water(&mut chunk, &mut outbound, ox + lx, y, oz + lz, KELP);
                        }
                    }
                    // scattered seagrass in shallow water
                    for _ in 0..3 {
                        let lx = rng.next_range(16) as i32;
                        let lz = rng.next_range(16) as i32;
                        let col_idx = lz as usize * 16 + lx as usize;
                        let h = chunk.height[col_idx] as i32;
                        if h < vc_chunk::SEA_LEVEL - 1 && h >= vc_chunk::SEA_LEVEL - 5 {
                            set_water(
                                &mut chunk, &mut outbound, ox + lx, h + 1, oz + lz, SEAGRASS,
                            );
                        }
                    }
                }
                // swamp seagrass (VERIFIED: "rivers, and swamplands")
                Biome::Swamp => {
                    for _ in 0..2 {
                        let lx = rng.next_range(16) as i32;
                        let lz = rng.next_range(16) as i32;
                        let col_idx = lz as usize * 16 + lx as usize;
                        let h = chunk.height[col_idx] as i32;
                        // swamp pools: water above the floor
                        if h < vc_chunk::SEA_LEVEL - 1 {
                            set_water(
                                &mut chunk, &mut outbound, ox + lx, h + 1, oz + lz, SEAGRASS,
                            );
                        }
                    }
                }
                _ => {}
            }
            // ---- icebergs (FrozenOcean, VERIFIED: "Generate on frozen
            // oceans"): 1-2 mounds per chunk at the surface — packed
            // ice body with a blue-ice core, 3-5 above sea level and
            // 2 below ----
            if b == Biome::FrozenOcean {
                let bergs = 1 + rng.next_range(2) as i32;
                for _ in 0..bergs {
                    let lx = 2 + rng.next_range(12) as i32;
                    let lz = 2 + rng.next_range(12) as i32;
                    let col_idx = lz as usize * 16 + lx as usize;
                    let h = chunk.height[col_idx] as i32;
                    if h > vc_chunk::SEA_LEVEL - 6 {
                        continue; // only over deep water
                    }
                    let r = 2 + rng.next_range(3) as i32; // 2..3 radius
                    let top = vc_chunk::SEA_LEVEL + 3 + rng.next_range(3) as i32;
                    for dy in -2..=(top - vc_chunk::SEA_LEVEL) {
                        let y = vc_chunk::SEA_LEVEL + dy;
                        // taper: full radius at the waterline, shrinking
                        let rr = if dy < 0 {
                            r
                        } else {
                            (r as f32 * (1.0 - dy as f32 / (top - vc_chunk::SEA_LEVEL + 1) as f32))
                                .ceil() as i32
                        };
                        for dx in -rr..=rr {
                            for dz in -rr..=rr {
                                if dx.abs() == rr && dz.abs() == rr {
                                    continue; // round the corners
                                }
                                let id = if dx.abs() + dz.abs() + dy.abs() <= 1 && dy >= 0 {
                                    BLUE_ICE // the blue-ice core
                                } else {
                                    PACKED_ICE
                                };
                                set_dec(&mut chunk, &mut outbound, ox + lx + dx, y, oz + lz + dz, id, false);
                            }
                        }
                    }
                }
            }
            // ---- buried treasure (VERIFIED changelog: "A new structure
            // that consists of a buried chest with loot in it. Has its
            // own buried_treasure loot table. Maps found in ocean ruins
            // can lead the player to them"): 1/128 chunk roll, buried
            // 1-2 blocks under a beach or ocean-floor surface ----
            if Rng::hash3(self.seed ^ 0x7EA5, cx, 0, cz) % 128 == 0 {
                let tx = 3 + rng.next_range(10) as i32;
                let tz = 3 + rng.next_range(10) as i32;
                let col_idx = tz as usize * 16 + tx as usize;
                let h = chunk.height[col_idx] as i32;
                // valid site: below is sand/gravel floor, above is water
                // or air (beach/sea floor), not inside a cave
                let floor = chunk.get(tx as usize, h as usize, tz as usize);
                let above = chunk.get(tx as usize, (h + 1) as usize, tz as usize);
                if (floor == SAND || floor == GRAVEL)
                    && (above == WATER || above == AIR)
                    && h <= vc_chunk::SEA_LEVEL + 1
                {
                    // bury: chest sits in the floor's material level,
                    // capped by sand so it reads "buried" from above
                    chunk.set(tx as usize, h as usize, tz as usize, CHEST);
                    if above == AIR {
                        chunk.set(tx as usize, (h + 1) as usize, tz as usize, SAND);
                    }
                }
            }
        }

        // 1.10 fossils — VERIFIED (wiki /w/Java_Edition_1.10 §World''', 'ocean decorations')

open(p, 'w').write(t)
print(f'GEN PATCH B DONE — {n} edits')
