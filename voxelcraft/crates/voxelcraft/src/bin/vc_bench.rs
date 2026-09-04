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
use std::hint::black_box;
use std::sync::Arc;
use std::time::Instant;
use vc_chunk::chunk::Chunk;
use vc_render::draw::{self, ChunkGpu, DrawCallAccounting, MeshSlot, SlotAlloc, VisEntry};
use vc_world::gen::TerrainGen;
use vc_mesh::mesh::{mesh_chunk, mesh_sections};
use vc_world::world::ChunkPos;

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
    // per-chunk mesh sizes for the Phase-9 drawprep scene (realistic slots)
    let mut mesh_dims: Vec<(ChunkPos, usize, usize, usize)> = Vec::with_capacity(meshable.len());
    for &pos in meshable.iter() {
        let snap = snapshot(&by_pos, pos);
        let lsnap = vc_world::light::reference_lightdata(&snap);
        let t0 = Instant::now();
        let md = mesh_chunk(pos, &snap, &lsnap, true);
        mesh_ms.push(t0.elapsed().as_secs_f32() * 1000.0);
        total_verts += md.solid.0.len() + md.water.0.len();
        total_tris += md.tri_count();
        mesh_dims.push((pos, md.solid.0.len(), md.solid.1.len(), md.water.1.len()));
    }

    // parallel mesh timing
    let t_mesh_par = Instant::now();
    let mesh_sizes: Vec<usize> = meshable
        .par_iter()
        .map(|&pos| {
            let snap = snapshot(&by_pos, pos);
            let lsnap = vc_world::light::reference_lightdata(&snap);
            let md = mesh_chunk(pos, &snap, &lsnap, true);
            md.solid.0.len() + md.water.0.len()
        })
        .collect();
    let mesh_par_ms = t_mesh_par.elapsed().as_secs_f32() * 1000.0;
    let par_verts: usize = mesh_sizes.iter().sum();

    // ---- Phase 7: GPU compute mesher (adapter-gated) ----
    // Headless probe: no adapter (CI, display-less) → SKIP with a note and
    // the bench stays CPU-only; with one → time the full GPU path (input
    // build + dispatch A + counts readback + offsets + dispatch B + output
    // readback — everything the game pays) against the CPU numbers, AND
    // verify bit-parity per chunk (the design contract).
    let gpu_mesh_report = match probe_headless_device() {
        None => "gpu-mesh    : SKIP (no GPU adapter — headless environment)".to_string(),
        Some((device, queue)) => {
            let mut mesher = vc_render::gpu_mesh::GpuMesher::new(&device, &queue);
            const GPU_BENCH_N: usize = 32;
            let subset: Vec<ChunkPos> = meshable.iter().take(GPU_BENCH_N).copied().collect();
            let t0 = Instant::now();
            let mut parity = true;
            for &pos in subset.iter() {
                let snap = snapshot(&by_pos, pos);
                let lsnap = vc_world::light::reference_lightdata(&snap);
                let inputs = vc_mesh::mesh::build_mesh_inputs(&snap, &lsnap);
                let want = mesh_chunk(pos, &snap, &lsnap, true);
                let gpu_ok = !inputs.has_cross && !inputs.has_models;
                mesher.enqueue(
                    vc_render::gpu_mesh::GpuMeshJobMeta {
                        pos,
                        mask: u16::MAX,
                        smooth: true,
                        prev: vec![None; 16],
                        center: snap[4].clone(),
                    },
                    inputs,
                );
                let done = mesher.wait_done(&device, &queue);
                if gpu_ok {
                    let got = &done[0].mesh;
                    parity &= got.solid.0 == want.solid.0 && got.solid.1 == want.solid.1
                        && got.water.0 == want.water.0 && got.water.1 == want.water.1;
                }
            }
            let gpu_ms = t0.elapsed().as_secs_f32() * 1000.0;
            let n = subset.len().max(1) as f32;
            format!(
                "gpu-mesh    : avg {:.2} ms/chunk (full pipeline incl. input build + 2 readbacks) over {n} chunks — bit-parity with CPU: {parity}",
                gpu_ms / n
            )
        }
    };

    // §48 Phase 3: partial remesh (§12 fine-grained invalidation).
    // A typical block edit dirties ~3 sections of one chunk — measure the
    // same 3-section job against a full 16-section remesh. Includes the
    // cache handoff (prev) exactly like the game's worker jobs.
    let mut remesh3_ms: Vec<f32> = Vec::with_capacity(meshable.len());
    let mut remesh3_eq = true; // determinism: partial == matching part of full
    for &pos in meshable.iter() {
        let snap = snapshot(&by_pos, pos);
        let lsnap = vc_world::light::reference_lightdata(&snap);
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

    // -------------------------------------------------- Phase 9 drawprep (§37)
    // Simulate the per-frame CPU draw-submission prep with REAL mesh sizes
    // and the REAL allocator/ordering/list/args code (§48 Phase-9 gate:
    // measurable). One "frame" = region ordering + the three pass lists +
    // region runs + MDI args packing for every visible chunk.
    let mut gpu: HashMap<ChunkPos, ChunkGpu> = HashMap::new();
    let mut allocs: HashMap<(i32, i32), (SlotAlloc, SlotAlloc)> = HashMap::new();
    for &(pos, v_len, i_len, w_i_len) in mesh_dims.iter() {
        let rk = draw::region_of(pos);
        let (va, ia) = allocs.entry(rk).or_default();
        let (v_off, _) = va.alloc(v_len as u32);
        let (i_off, _) = ia.alloc(i_len as u32);
        let solid = MeshSlot {
            region: rk,
            v_off,
            v_cap: v_len as u32,
            i_off,
            i_cap: i_len as u32,
            n: i_len as u32,
        };
        let water = if w_i_len > 0 {
            // water shares the same arena allocators (separate slot)
            let (w_v_off, _) = va.alloc((v_len / 4).max(1) as u32);
            let (w_i_off, _) = ia.alloc(w_i_len as u32);
            Some(MeshSlot {
                region: rk,
                v_off: w_v_off,
                v_cap: (v_len / 4).max(1) as u32,
                i_off: w_i_off,
                i_cap: w_i_len as u32,
                n: w_i_len as u32,
            })
        } else {
            None
        };
        // Phase 6 §26: the bench measures draw-list mechanics with every
        // meshed chunk visible (worst-case list sizes, per the comment
        // below) — so the synthetic occlusion data is fully open + all-geo
        // (occlusion culling would remove chunks and shrink the lists,
        // breaking comparability with the Phase-9 baselines)
        gpu.insert(
            pos,
            ChunkGpu {
                solid,
                water,
                occl: draw::ChunkOccl {
                    sides: u64::MAX,
                    planes: u16::MAX,
                    geo: u16::MAX,
                },
            },
        );
    }
    // visible set: every meshed chunk, dist² from a camera at the grid
    // center (all visible — worst-case list sizes), origin rows assigned
    let cam2 = (0.0f32, 0.0f32);
    let vis: Vec<VisEntry> = mesh_dims
        .iter()
        .enumerate()
        .map(|(i, &(pos, ..))| {
            let dx = pos.0 as f32 * 16.0 + 8.0 - cam2.0;
            let dz = pos.1 as f32 * 16.0 + 8.0 - cam2.1;
            (pos, dx * dx + dz * dz, i as u32)
        })
        .collect();
    const SHADOW_R2: f32 = (110.0 + 23.0) * (110.0 + 23.0);
    let mut drawprep_us = 0.0f64;
    let mut acc_loop = DrawCallAccounting::default();
    let mut acc_mdi = DrawCallAccounting::default();
    let mut acc_legacy = DrawCallAccounting::default();
    let n_regions = allocs.len();
    const DP_FRAMES: usize = 200;
    for f in 0..DP_FRAMES {
        let t0 = Instant::now();
        let terrain_order = draw::order_by_region(&vis, cam2, false);
        let water_order: Vec<VisEntry> = terrain_order.iter().rev().copied().collect();
        let terrain_list = draw::build_draw_list(&gpu, &terrain_order, false, None);
        let water_list = draw::build_draw_list(&gpu, &water_order, true, None);
        let shadow_list = draw::build_draw_list(&gpu, &terrain_order, false, Some(SHADOW_R2));
        let runs = draw::region_runs(black_box(&terrain_list));
        let args = draw::pack_args(black_box(&terrain_list));
        let us = t0.elapsed().as_secs_f64() * 1e6;
        if f > 20 {
            // skip the first iterations (cache warmup) like the game bench
            drawprep_us += us;
        }
        if f == DP_FRAMES - 1 {
            acc_loop = DrawCallAccounting::loop_path(&terrain_list, &water_list, &shadow_list);
            acc_mdi = DrawCallAccounting::mdi_path(&terrain_list, &water_list, &shadow_list);
            acc_legacy = DrawCallAccounting::legacy(&terrain_list, &water_list, &shadow_list);
            black_box((runs, args));
        }
    }
    drawprep_us /= (DP_FRAMES - 21) as f64;
    let visible_n = vis.len();

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
    // Phase 9 §48 gate: draw submission accounting (all three passes)
    println!(
        "drawprep    : {drawprep_us:.1} µs/frame  ({visible_n} visible chunks, {n_regions} regions, 3 passes, incl. MDI args pack)"
    );
    println!(
        "draw calls  : legacy {}/{} binds  →  region-loop {}/{}  →  MDI {}/{}  (per frame)",
        acc_legacy.draws, acc_legacy.binds, acc_loop.draws, acc_loop.binds, acc_mdi.draws, acc_mdi.binds
    );
    println!(
        "bind reduction : {:.1}x fewer buffer binds (loop) / {:.1}x (MDI) vs legacy",
        acc_legacy.binds as f64 / acc_loop.binds.max(1) as f64,
        acc_legacy.binds as f64 / acc_mdi.binds.max(1) as f64
    );
    println!("{gpu_mesh_report}");
    println!("GPU metrics : unavailable (headless — use `voxelcraft --benchmark` on a desktop)");
    println!("============================================================\n");

    if !json_path.is_empty() {
        let json = format!(
            "{{\"headless_bench\":{{\"seed\":{seed},\"chunks\":{n_chunks},\"threads\":{threads},\"gen\":{{\"avg_ms\":{gen_avg:.3},\"p50_ms\":{gen_p50:.3},\"p95_ms\":{gen_p95:.3},\"parallel_total_ms\":{gen_par_ms:.3}}},\"mesh\":{{\"chunks\":{},\"avg_ms\":{mesh_avg:.3},\"p50_ms\":{mesh_p50:.3},\"p95_ms\":{mesh_p95:.3},\"parallel_total_ms\":{mesh_par_ms:.3}}},\"remesh3\":{{\"avg_ms\":{rm3_avg:.3},\"p50_ms\":{rm3_p50:.3},\"deterministic\":{remesh3_eq}}},\"drawprep\":{{\"us_per_frame\":{drawprep_us:.2},\"visible\":{visible_n},\"regions\":{n_regions},\"legacy\":{{\"draws\":{},\"binds\":{}}},\"loop\":{{\"draws\":{},\"binds\":{}}},\"mdi\":{{\"draws\":{},\"binds\":{}}}}},\"verts\":{total_verts},\"tris\":{total_tris},\"par_verts\":{par_verts},\"heap_bytes\":{heap}}}}}",
            meshable.len(),
            acc_legacy.draws, acc_legacy.binds,
            acc_loop.draws, acc_loop.binds,
            acc_mdi.draws, acc_mdi.binds
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


/// Phase 7: headless wgpu device probe for the GPU-mesh bench section
/// (None in CI/display-less environments — those print a SKIP line).
fn probe_headless_device() -> Option<(wgpu::Device, wgpu::Queue)> {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        ..Default::default()
    });
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::None,
        compatible_surface: None,
        force_fallback_adapter: true,
    }))?;
    let downlevel = adapter.get_downlevel_capabilities();
    if !downlevel.flags.contains(wgpu::DownlevelFlags::COMPUTE_SHADERS) {
        return None;
    }
    pollster::block_on(adapter.request_device(
        &wgpu::DeviceDescriptor {
            label: Some("vc-bench-gpu-mesh"),
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::default(),
            ..Default::default()
        },
        None,
    ))
    .ok()
}
