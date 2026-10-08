//! 2.2a: world-gen oracle primitives — compare an IMPORTED chunk against
//! freshly generated output for the same seed/coordinates.
//!
//! Scope (PLAN-FINAL 2.2): per-chunk block + biome identity, structure
//! match reporting lives in 2.2b; the FIRST-mismatch locator and the
//! import/generate seam test live here. Comparisons are BLOCK-level
//! (folded ids — orientation-insensitive) plus biome-level; raw-state
//! comparison is a future refinement. Real imported worlds are a soft
//! blocker (P1) — until then the harness proves itself on synthetic
//! pairs and our own generator output ("commit only numbers" starts
//! when real imports arrive).

use vc_chunk::chunk::{Chunk, LocalXZ};

/// Outcome of [`diff_chunks`]: agreement counts over the 16×256×16
/// cells (blocks) and 16×16 columns (biomes), plus the first block
/// mismatch in scan order (y, z, x) for the locator.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ChunkDiff {
    pub cells: u64,
    pub same_blocks: u64,
    pub columns: u64,
    pub same_biomes: u64,
    pub first_mismatch: Option<(usize, usize, usize)>,
}

impl ChunkDiff {
    pub fn block_rate(&self) -> f64 {
        self.same_blocks as f64 / self.cells.max(1) as f64
    }
    pub fn biome_rate(&self) -> f64 {
        self.same_biomes as f64 / self.columns.max(1) as f64
    }
}

/// Per-chunk identity: every cell's folded block id + every column's
/// biome id. Order-independent (either side may be imported or
/// generated); determinism comes from the inputs, this fn only counts.
pub fn diff_chunks(a: &Chunk, b: &Chunk) -> ChunkDiff {
    let mut out = ChunkDiff {
        cells: 0,
        same_blocks: 0,
        columns: 0,
        same_biomes: 0,
        first_mismatch: None,
    };
    for y in 0..256usize {
        for z in 0..16usize {
            for x in 0..16usize {
                out.cells += 1;
                if a.get_local(LocalXZ::new(x, z), y) == b.get_local(LocalXZ::new(x, z), y) {
                    out.same_blocks += 1;
                } else if out.first_mismatch.is_none() {
                    out.first_mismatch = Some((x, y, z));
                }
            }
        }
    }
    for lz in 0..16usize {
        for lx in 0..16usize {
            out.columns += 1;
            if a.biome[lz * 16 + lx] == b.biome[lz * 16 + lx] {
                out.same_biomes += 1;
            }
        }
    }
    out
}

/// Seam test (x axis): the shared face between a western chunk's x=15
/// wall and its eastern neighbor's x=0 wall — (same, total=4096).
/// A perfect generator scores total; the number itself is the oracle
/// datum (2.2 commits numbers, never verdicts, until calibrated).
pub fn seam_match_x(west: &Chunk, east: &Chunk) -> (u64, u64) {
    let mut same = 0u64;
    let mut total = 0u64;
    for y in 0..256usize {
        for z in 0..16usize {
            total += 1;
            if west.get_local(LocalXZ::new(15, z), y) == east.get_local(LocalXZ::new(0, z), y) {
                same += 1;
            }
        }
    }
    (same, total)
}

/// Seam test (z axis): northern chunk's z=15 wall vs southern
/// neighbor's z=0 wall.
pub fn seam_match_z(north: &Chunk, south: &Chunk) -> (u64, u64) {
    let mut same = 0u64;
    let mut total = 0u64;
    for y in 0..256usize {
        for x in 0..16usize {
            total += 1;
            if north.get_local(LocalXZ::new(x, 15), y) == south.get_local(LocalXZ::new(x, 0), y) {
                same += 1;
            }
        }
    }
    (same, total)
}

/// 2.2b: one imported structure start — verbatim kind string + chunk
/// coordinates.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StructureHit {
    pub kind: String,
    pub cx: i32,
    pub cz: i32,
}

/// Read structure starts out of an imported Structures compound
/// (public 1.16 shape: `{Starts: {<id>: {ChunkX, ChunkZ} or {BB}}}`).
/// Entries without usable coordinates are skipped (§46); unknown type
/// ids are kept verbatim (no hardcoded type list — R6).
pub fn extract_starts(structures: &vc_nbt::nbt::Nbt) -> Vec<StructureHit> {
    use vc_nbt::nbt::Nbt;
    let mut out = Vec::new();
    let Some(Nbt::Compound(map)) = structures.get("Starts") else {
        return out;
    };
    for (kind, entry) in map.iter() {
        let hit = entry
            .get("ChunkX")
            .and_then(|v| v.as_i64())
            .zip(entry.get("ChunkZ").and_then(|v| v.as_i64()))
            .map(|(x, z)| (x as i32, z as i32))
            .or_else(|| {
                entry
                    .get("BB")
                    .and_then(|v| v.as_i32_slice())
                    .and_then(|bb| {
                        // BB = [minX, minY, minZ, maxX, maxY, maxZ]
                        if bb.len() >= 6 {
                            Some((bb[0].div_euclid(16), bb[2].div_euclid(16)))
                        } else {
                            None
                        }
                    })
            });
        if let Some((cx, cz)) = hit {
            out.push(StructureHit {
                kind: kind.clone(),
                cx,
                cz,
            });
        }
    }
    out
}

/// Position match between imported starts and predicted positions
/// (chunk units, Chebyshev distance ≤ `tol`): (matched, total,
/// kinds_agree). Kinds compare verbatim — predicted lists from our
/// queries carry our kind strings, so a vocabulary gap surfaces as
/// kinds_agree < matched (honest signal for Part-4 mapping, never a
/// silent pass). No division inside (callers rate it).
pub fn structure_match_rate(
    imported: &[StructureHit],
    predicted: &[(String, i32, i32)],
    tol: i32,
) -> (u64, u64, u64) {
    let mut matched = 0u64;
    let mut kinds = 0u64;
    for hit in imported {
        let mut best: Option<(i32, &str)> = None;
        for (pkind, px, pz) in predicted {
            let d = (hit.cx - px).abs().max((hit.cz - pz).abs());
            if d <= tol && best.map(|(bd, _)| d < bd).unwrap_or(true) {
                best = Some((d, pkind));
            }
        }
        if let Some((_, pkind)) = best {
            matched += 1;
            if pkind == hit.kind {
                kinds += 1;
            }
        }
    }
    (matched, imported.len() as u64, kinds)
}

#[cfg(test)]
mod tests {
    use super::*;
    use vc_world::gen::TerrainGen;

    fn gen_pair(seed: u64, cx: i32, cz: i32) -> (Chunk, Chunk) {
        let g = TerrainGen::new(seed);
        let (a, _) = g.generate_chunk(cx, cz, Vec::new());
        let (b, _) = g.generate_chunk(cx, cz, Vec::new());
        ((*a).clone(), (*b).clone())
    }

    #[test]
    fn identical_pair_scores_full() {
        let (a, b) = gen_pair(0xC0FFEE, 0, 0);
        let d = diff_chunks(&a, &b);
        assert_eq!(d.cells, 16 * 256 * 16);
        assert_eq!(d.same_blocks, d.cells);
        assert_eq!(d.same_biomes, d.columns);
        assert_eq!(d.first_mismatch, None);
        assert_eq!(d.block_rate(), 1.0);
        assert_eq!(d.biome_rate(), 1.0);
    }

    #[test]
    fn single_block_diff_is_located() {
        let (a, mut b) = gen_pair(0xC0FFEE, 0, 0);
        // flip one cell to something it cannot already be (bedrock at
        // the surface is never generated there — scan for a safe cell)
        let (mut fx, mut fy, mut fz) = (0usize, 200usize, 0usize);
        'scan: for y in (100..200usize).rev() {
            for z in 0..16usize {
                for x in 0..16usize {
                    if a.get_local(LocalXZ::new(x, z), y) != 1 {
                        fx = x;
                        fy = y;
                        fz = z;
                        break 'scan;
                    }
                }
            }
        }
        b.set(fx, fy, fz, 1);
        // the flip must read back (block-1 roundtrips through
        // default_state — a loud failure here beats a silent miscount)
        assert_eq!(b.get_local(LocalXZ::new(fx, fz), fy), 1);
        let d = diff_chunks(&a, &b);
        assert_eq!(d.same_blocks, d.cells - 1);
        // the locator reports A mismatch position holding the flipped
        // cell (scan order may find an earlier natural difference only
        // if generation itself disagrees — identical inputs, so exact)
        assert_eq!(d.first_mismatch, Some((fx, fy, fz)));
        assert_eq!(d.same_biomes, d.columns);
    }

    #[test]
    fn seam_helpers_cover_full_faces() {
        let g = TerrainGen::new(0xC0FFEE);
        let (w, _) = g.generate_chunk(0, 0, Vec::new());
        let (e, _) = g.generate_chunk(1, 0, Vec::new());
        let (n, _) = g.generate_chunk(0, -1, Vec::new());
        let (sx, tx) = seam_match_x(&w, &e);
        let (sz, tz) = seam_match_z(&n, &w);
        assert_eq!(tx, 16 * 256);
        assert_eq!(tz, 16 * 256);
        assert!(sx <= tx && sz <= tz);
    }

    fn starts_fixture() -> vc_nbt::nbt::Nbt {
        use vc_nbt::nbt::Nbt;
        // ChunkX/ChunkZ entry + BB-only entry (BB min (32,?,80) →
        // chunk (2,5)) + malformed entry (skipped, §46)
        let mut v = Nbt::compound();
        v.set("ChunkX", Nbt::Int(4));
        v.set("ChunkZ", Nbt::Int(-3));
        let mut m = Nbt::compound();
        m.set("BB", Nbt::IntArray(vec![32, 60, 80, 47, 70, 95]));
        let bad = Nbt::compound();
        let mut starts = Nbt::compound();
        starts.set("future:village", v);
        starts.set("future:mineshaft", m);
        starts.set("future:broken", bad);
        let mut structs = Nbt::compound();
        structs.set("Starts", starts);
        structs
    }

    #[test]
    fn starts_extract_both_shapes_and_skip_malformed() {
        let hits = extract_starts(&starts_fixture());
        assert_eq!(hits.len(), 2);
        assert!(hits.contains(&StructureHit {
            kind: "future:village".into(),
            cx: 4,
            cz: -3
        }));
        assert!(hits.contains(&StructureHit {
            kind: "future:mineshaft".into(),
            cx: 2,
            cz: 5
        }));
        // no Starts at all → empty, no panic
        assert!(extract_starts(&vc_nbt::nbt::Nbt::compound()).is_empty());
    }

    #[test]
    fn structure_match_counts_positions_and_kinds() {
        let hits = extract_starts(&starts_fixture());
        let pred = vec![
            ("future:village".to_string(), 4, -3),
            ("future:mineshaft".to_string(), 2, 5),
        ];
        assert_eq!(structure_match_rate(&hits, &pred, 0), (2, 2, 2));
        // tolerance: off-by-one matches at tol 1, misses at tol 0
        let shifted = vec![("future:village".to_string(), 5, -3)];
        assert_eq!(structure_match_rate(&hits[..1], &shifted, 0), (0, 1, 0));
        assert_eq!(structure_match_rate(&hits[..1], &shifted, 1), (1, 1, 1));
        // vocabulary gap: position matches, kind does not
        let renamed = vec![("our:village".to_string(), 4, -3)];
        assert_eq!(structure_match_rate(&hits[..1], &renamed, 0), (1, 1, 0));
        // empty imported: zeros, no division
        assert_eq!(structure_match_rate(&[], &pred, 0), (0, 0, 0));
    }
}
