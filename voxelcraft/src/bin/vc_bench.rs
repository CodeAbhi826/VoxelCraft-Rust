//! Headless CPU-pipeline benchmark (Master Spec §37/§48 Phase 0 gate).
//!
//! Measures the world pipeline WITHOUT any GPU/window dependency, so it runs
//! in CI and on display-less machines:
//!   * terrain generation throughput (per-chunk, sequential + rayon parallel)
//!   * mesh compilation throughput (greedy mesher + skylight + block-light BFS)
//!
//! Usage:
//!   cargo run --release --bin vc_bench [-- chunks=96] [-- seed=12648430]
//!                                       [-- json=bench.json] [-- workers=0]
//!
//! Output: a table to stdout + optional JSON file. Frame/GPU metrics are
//! deliberately NOT reported here (no GPU present) — those come from the
//! in-game `--benchmark` mode on a real desktop (§31: label unavailable
//! rather than fabricate).

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use voxelcraft::chunk::Chunk;
use voxelcraft::gen::TerrainGen;
use voxelcraft::mesh::{mesh_chunk, mesh_sections};
use voxelcraft::world::ChunkPos;

use rayon::prelude::*;

fn percentile_ms(ms: &mut Vec<f32>, q: f32) -> f32 {
    ms.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let n = ms.len();
    if n == 0 {
        return 0.0;
    }
    let idx = (((n as f32 - 1.0) * q).round() as usize).min(n - 1);
    ms[idx]
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let get = |key: &str, def: &str| -> String {
        args.iter()
            .rev()
            .find_map(|a| a.strip_prefix(&format!("{key}=")))
            .map(|v| v.to_string())
            .unwrap_or_else(|| def.to_string())
    };
    let n_chunks: usize = get("chunks", "96").parse().unwrap();
    let seed: u64 = u64::from_str_radix(get("seed", "C0FFEE").trim_start_matches("0x"), 16)
        .or_else(|_| get("seed", "0").parse())
        .unwrap_or(0xC0FFEE);
    let json_path = get("json", "");
    let workers: usize = get("workers", "0").parse().unwrap(); // 0 = default pool

    // grid of chunk positions, ring order from origin (like real streaming)
    let mut positions: Vec<ChunkPos> = Vec::with_capacity(n_chunks);
    let radius = ((n_chunks as f64).sqrt() / 2.0).ceil() as i32 + 1;
    'outer: for r in 0..=radius {
        for dz in -r..=r {
            for dx in -r..=r {
                if dx.abs() != r && dz.abs() != r {
                    continue; // ring order
                }
                if positions.len() >= n_chunks {
                    break 'outer;
                }
                positions.push((dx, dz));
            }
        }
    }

    if workers > 0 {
        rayon::ThreadPoolBuilder::new()
            .num_threads(workers)
            .build_global()
            .ok();
    }
    let threads = rayon::current_num_threads();

    // ------------------------------------------------------------ generation
    let gen = TerrainGen::new(seed);
    let mut gen_ms: Vec<f32> = Vec::with_capacity(n_chunks);
    let chunks: Vec<Arc<Chunk>> = positions
        .iter()
        .map(|&pos| {
            let t0 = Instant::now();
            let (chunk, _outbound) = gen.generate_chunk(pos.0, pos.1, Vec::new());
            gen_ms.push(t0.elapsed().as_secs_f32() * 1000.0);
            chunk
        })
        .collect();

    // parallel generation timing (same chunks, fresh RNG state)
    let t_par = Instant::now();
    let _par_chunks: Vec<Arc<Chunk>> = positions
        .par_iter()
        .map(|&pos| {
            let (chunk, _) = gen.generate_chunk(pos.0, pos.1, Vec::new());
            chunk
        })
        .collect();
    let gen_par_ms = t_par.elapsed().as_secs_f32() * 1000.0;

    // ---------------------------------------------------------------- meshing
    // mesh every chunk whose full 3x3 neighborhood exists (interior ring)
    let by_pos: HashMap<ChunkPos, Arc<Chunk>> = positions
        .iter()
        .copied()
        .zip(chunks.iter().cloned())
        .collect();
    let meshable: Vec<ChunkPos> = positions
        .iter()
        .filter(|p| {
            (-1..=1).all(|dz| (-1..=1).all(|dx| by_pos.contains_key(&(p.0 + dx, p.1 + dz))))
        })
        .copied()
        .collect();

    let mut mesh_ms: Vec<f32> = Vec::with_capacity(meshable.len());
    let mut total_verts = 0usize;
    let mut total_tris = 0u32;
    for &pos in meshable.iter() {
        let snap = snapshot(&by_pos, pos);
        let lsnap = voxelcraft::light::reference_lightdata(&snap);
        let t0 = Instant::now();
        let md = mesh_chunk(pos, &snap, &lsnap, true);
        mesh_ms.push(t0.elapsed().as_secs_f32() * 1000.0);
        total_verts += md.solid.0.len() + md.water.0.len();
        total_tris += md.tri_count();
    }

    // parallel mesh timing
    let t_mesh_par = Instant::now();
    let mesh_sizes: Vec<usize> = meshable
        .par_iter()
        .map(|&pos| {
            let snap = snapshot(&by_pos, pos);
            let lsnap = voxelcraft::light::reference_lightdata(&snap);
            let md = mesh_chunk(pos, &snap, &lsnap, true);
            md.solid.0.len() + md.water.0.len()
        })
        .collect();
    let mesh_par_ms = t_mesh_par.elapsed().as_secs_f32() * 1000.0;
    let par_verts: usize = mesh_sizes.iter().sum();

    // §48 Phase 3: partial remesh (§12 fine-grained invalidation).
    // A typical block edit dirties ~3 sections of one chunk — measure the
    // same 3-section job against a full 16-section remesh. Includes the
    // cache handoff (prev) exactly like the game's worker jobs.
    let mut remesh3_ms: Vec<f32> = Vec::with_capacity(meshable.len());
    let mut remesh3_eq = true; // determinism: partial == matching part of full
    for &pos in meshable.iter() {
        let snap = snapshot(&by_pos, pos);
        let lsnap = voxelcraft::light::reference_lightdata(&snap);
        let full = mesh_sections(pos, &snap, &lsnap, true, u16::MAX, &[]);
        let cache = full.sections.clone();
        // sections 4–6 (typical terrain band, y 64–111)
        let mask: u16 = 0b111 << 4;
        let t0 = Instant::now();
        let part = mesh_sections(pos, &snap, &lsnap, true, mask, &cache);
        remesh3_ms.push(t0.elapsed().as_secs_f32() * 1000.0);
        // the partial result must reproduce the full mesh bit-for-bit
        // (unmasked sections reused verbatim, masked rebuilt deterministically)
        if part.merged.solid.0 != full.merged.solid.0
            || part.merged.solid.1 != full.merged.solid.1
            || part.merged.water.0 != full.merged.water.0
            || part.merged.water.1 != full.merged.water.1
        {
            remesh3_eq = false;
        }
    }

    // memory footprint of the paletted sections
    let heap: usize = chunks.iter().map(|c| c.heap_bytes()).sum();

    // ---------------------------------------------------------------- report
    let gen_seq_total = gen_ms.iter().sum::<f32>();
    let gen_p50 = percentile_ms(&mut gen_ms, 0.50);
    let gen_p95 = percentile_ms(&mut gen_ms, 0.95);
    let gen_avg = gen_seq_total / gen_ms.len().max(1) as f32;
    let mesh_p50 = percentile_ms(&mut mesh_ms, 0.50);
    let mesh_p95 = percentile_ms(&mut mesh_ms, 0.95);
    let mesh_avg = mesh_ms.iter().sum::<f32>() / mesh_ms.len().max(1) as f32;

    println!("\n============ VoxelCraft headless CPU benchmark ============");
    println!("seed {seed:#x}  |  {n_chunks} chunks  |  rayon threads: {threads}");
    println!("generation  : avg {gen_avg:.2} ms  p50 {gen_p50:.2} ms  p95 {gen_p95:.2} ms  (per chunk, single-threaded)");
    println!(
        "generation  : total {gen_par_ms:.0} ms parallel  ({:.1}x speedup over sequential {} ms)",
        gen_seq_total / gen_par_ms.max(0.001),
        gen_seq_total
    );
    println!("meshing     : {} interior chunks", meshable.len());
    println!("meshing     : avg {mesh_avg:.2} ms  p50 {mesh_p50:.2} ms  p95 {mesh_p95:.2} ms  (per chunk, single-threaded, incl. light BFS)");
    println!("meshing     : total {mesh_par_ms:.0} ms parallel");
    let rm3_p50 = percentile_ms(&mut remesh3_ms, 0.50);
    let rm3_avg = remesh3_ms.iter().sum::<f32>() / remesh3_ms.len().max(1) as f32;
    println!(
        "remesh 3sec : avg {rm3_avg:.2} ms  p50 {rm3_p50:.2} ms  (3-section edit job, §12 fine-grained; deterministic: {remesh3_eq})"
    );
    println!("geometry    : {total_verts} vertices  {total_tris} triangles  (VC-16 = 16 B/vertex)");
    println!(
        "chunk memory: {:.2} MiB for {} chunks ({:.1} KiB/chunk avg, paletted sections)",
        heap as f64 / (1024.0 * 1024.0),
        n_chunks,
        heap as f64 / n_chunks.max(1) as f64 / 1024.0
    );
    println!("GPU metrics : unavailable (headless — use `voxelcraft --benchmark` on a desktop)");
    println!("============================================================\n");

    if !json_path.is_empty() {
        let json = format!(
            "{{\"headless_bench\":{{\"seed\":{seed},\"chunks\":{n_chunks},\"threads\":{threads},\"gen\":{{\"avg_ms\":{gen_avg:.3},\"p50_ms\":{gen_p50:.3},\"p95_ms\":{gen_p95:.3},\"parallel_total_ms\":{gen_par_ms:.3}}},\"mesh\":{{\"chunks\":{},\"avg_ms\":{mesh_avg:.3},\"p50_ms\":{mesh_p50:.3},\"p95_ms\":{mesh_p95:.3},\"parallel_total_ms\":{mesh_par_ms:.3}}},\"remesh3\":{{\"avg_ms\":{rm3_avg:.3},\"p50_ms\":{rm3_p50:.3},\"deterministic\":{remesh3_eq}}},\"verts\":{total_verts},\"tris\":{total_tris},\"par_verts\":{par_verts},\"heap_bytes\":{heap}}}}}",
            meshable.len()
        );
        std::fs::write(&json_path, json)
            .unwrap_or_else(|e| eprintln!("failed to write {json_path}: {e}"));
        println!("JSON written to {json_path}");
    }
}

fn snapshot(
    by_pos: &HashMap<ChunkPos, Arc<Chunk>>,
    pos: ChunkPos,
) -> [Option<Arc<Chunk>>; 9] {
    let mut snap: [Option<Arc<Chunk>>; 9] = Default::default();
    let mut i = 0;
    for dz in -1..=1 {
        for dx in -1..=1 {
            snap[i] = by_pos.get(&(pos.0 + dx, pos.1 + dz)).cloned();
            i += 1;
        }
    }
    snap
}
