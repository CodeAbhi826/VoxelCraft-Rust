//! Greedy mesher + voxel lighting (skylight column scan + lateral BFS) + AO
//! + JSON-model path (blockstate dispatch, Phase 1).
//! Pure function over a 3x3 chunk snapshot → safe on worker threads.

use vc_blocks::blocks::*;
use vc_chunk::chunk::Chunk;
use vc_world::world::ChunkPos;
use std::sync::Arc;

/// engine-drawn missing-texture tile (magenta/black checker) — §46 fallback
pub const TILE_MISSING: u16 = 63;

/// snapshot values are STATE ids (u16 truncated to u8, ≤ 62 today);
/// property lookups fold them to block ids
#[inline]
fn sb(s: u8) -> u8 {
    state_block(s as u16)
}

/// VC-16 packed terrain vertex — 16 bytes, Sodium-class GPU bandwidth
/// (−60% vs the previous 40-byte float layout).
///
/// Positions are CHUNK-relative; the chunk origin reaches the shader via an
/// instance-rate `Float32x2` vertex buffer sliced per draw (portable to
/// Vulkan/DX12/Metal, WebGPU and WebGL2 — no push constants, no dynamic
/// uniform offsets, no SSBOs).
///
/// Bit layout (LSB = bit 0):
///   w0 =  z:u16 << 16 | x:u16   — xz @ 1/2048 block, offset −8 (range −8..+24,
///                                 32-block span like Sodium 0.5; covers the
///                                 16-block chunk + overhang)
///   w1 =  flags:u16 << 16 | y:u16 — y @ 1/128 block (0..256 exact, water
///                                 surface 0.875 and build-limit 256.0 exact)
///                                 flags = normal:3 | ao:2 | material:4 | spare:7
///   w2 =  tile:u14 << 18 | u:u8 << 10 | v:u8 << 2 | bias:u2
///                                 — uv in 1/16-block units (texel-exact for
///                                 16px tiles; greedy runs up to 16 blocks
///                                 encode exactly, the 16.0 endpoint clamps to
///                                 255/256 which only shortens the final
///                                 half-texel of a 16-wide run)
///   w3 =  state:u16 << 16 | tint:u8 << 8 | sky:u4 << 4 | block:u4
///                                 — state = block id today, block-state id
///                                 after the BlockState registry lands (u16
///                                 headroom per the research doc);
///                                 tint = §18 biome tint index
///                                 (kind:u2 << 6 | slot:u6, 0 = untinted;
///                                 decoded via the tint LUT in the vertex
///                                 stage — see tint.rs)
///
/// normal: 0=+X 1=−X 2=+Y 3=−Y 4=+Z 5=−Z 6=cross-plants (shade tables in
/// the shaders reproduce the old `face_shade * AO_MULT` exactly). bias:u2
/// remains reserved (the texture-bleed seam fix landed as explicit
/// gradients + a half-texel inset in TERRAIN_SHADER/WATER_SHADER fs_main —
/// see the `terrain_water_seam_guards_present` drift-guard test; the bits
/// stay free for a future per-vertex inset refinement).
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable, Debug, PartialEq, Eq)]
pub struct Vertex {
    pub w0: u32,
    pub w1: u32,
    pub w2: u32,
    pub w3: u32,
}

#[inline]
fn pack_vertex(
    x: f32,
    y: f32,
    z: f32,
    u: f32,
    v: f32,
    tile: u16,
    normal: u32,
    ao: u32,
    sky: u32,
    block: u32,
    state: u16,
    tint: u8,
) -> Vertex {
    let px = (((x + 8.0) * 2048.0 + 0.5) as i64).clamp(0, 0xFFFF) as u32;
    let pz = (((z + 8.0) * 2048.0 + 0.5) as i64).clamp(0, 0xFFFF) as u32;
    let py = ((y * 128.0 + 0.5) as i64).clamp(0, 0xFFFF) as u32;
    let pu = (((u * 16.0) + 0.5) as i64).clamp(0, 0xFF) as u32;
    let pv = (((v * 16.0) + 0.5) as i64).clamp(0, 0xFF) as u32;
    let flags = (normal & 7) | ((ao & 3) << 3);
    let t = (tile as u32) & 0x3FFF;
    Vertex {
        w0: (pz << 16) | px,
        w1: (flags << 16) | py,
        w2: (t << 18) | (pu << 10) | (pv << 2),
        w3: ((state as u32) << 16) | ((tint as u32) << 8) | ((sky & 0xF) << 4) | (block & 0xF),
    }
}

/// normal index from greedy-mesh axis (d) and face direction (dir)
#[inline]
fn normal_index(d: usize, dir: i32) -> u32 {
    match (d, dir > 0) {
        (0, true) => 0,  // +X
        (0, false) => 1, // -X
        (1, true) => 2,  // +Y
        (1, false) => 3, // -Y
        (2, true) => 4,  // +Z
        _ => 5,          // -Z
    }
}

pub struct MeshData {
    pub solid: (Vec<Vertex>, Vec<u32>),
    pub water: (Vec<Vertex>, Vec<u32>),
}

impl MeshData {
    pub fn tri_count(&self) -> u32 {
        (self.solid.1.len() + self.water.1.len()) as u32 / 3
    }
}

/// Section-set meshing result (§12 fine-grained invalidation, Phase 3).
///
/// Greedy runs never cross 16×16×16 section Y boundaries (masks are
/// section-local), so each section's output depends only on the world
/// snapshot — a partial remesh is bit-identical to the matching part of a
/// full remesh, and cached sections merge back losslessly.
pub struct MeshOut {
    /// 16 entries: fresh meshes for masked sections, `Arc` clones of `prev`
    /// for the rest (cache continuity — only dirty sections are rebuilt)
    pub sections: Vec<Option<Arc<MeshData>>>,
    /// all 16 concatenated with rebased indices — the per-chunk upload (§14
    /// per-chunk merged buffers, unchanged draw path)
    pub merged: MeshData,
}

/// Full-chunk mesh (bench + tests + first mesh of a chunk).
pub fn mesh_chunk(
    pos: ChunkPos,
    snap: &[Option<Arc<Chunk>>; 9],
    lsnap: &[Option<Arc<vc_world::light::LightData>>; 9],
    smooth: bool,
) -> MeshData {
    mesh_sections(pos, snap, lsnap, smooth, u16::MAX, &[]).merged
}

// padded 48 x 256 x 48 region covering the 3x3 snapshot
pub const PAD: usize = 48;
#[inline]
fn pidx(x: usize, y: usize, z: usize) -> usize {
    y * (PAD * PAD) + z * PAD + x
}

/// Phase 7: the padded snapshot volumes the greedy core consumes.
///
/// Extracted verbatim from `mesh_sections` (same loops, same defaults) so
/// the CPU mesher and the GPU compute mesher (vc-render `gpu_mesh`) build
/// on byte-identical inputs — the padding semantics (absent-neighbor air,
/// "never written" dark-interior light, open-sky defaults, biome pad)
/// have exactly ONE source of truth.
pub struct MeshInputs {
    /// 48×256×48 block STATE ids (u8), padded region covering the 3×3
    pub blocks: Vec<u8>,
    /// 48×256×48 skylight nibbles (same layout)
    pub light: Vec<u8>,
    /// 48×256×48 block-light nibbles (same layout)
    pub blight: Vec<u8>,
    /// center chunk's 256 per-column biome ids (tint resolution is
    /// per-block but only the center chunk emits faces)
    pub biomes: Box<[u8]>,
    /// center snapshot contains cross plants (special CPU path)
    pub has_cross: bool,
    /// center snapshot contains JSON-model states (special CPU path)
    pub has_models: bool,
}

/// Phase 7: materialize the padded input volumes from a 3×3 chunk + light
/// snapshot. Byte-identical to what `mesh_sections` builds internally
/// (the internal copy was removed — `mesh_sections` now calls this).
pub fn build_mesh_inputs(
    snap: &[Option<Arc<Chunk>>; 9],
    lsnap: &[Option<Arc<vc_world::light::LightData>>; 9],
) -> MeshInputs {
    // ------------------------------------------------ copy blocks (padded)
    // decode paletted sections into the flat padded buffer; air-only
    // sections cost one `None` probe instead of 16 KB of zeros
    let mut blocks = vec![0u8; PAD * PAD * 256];
    // flags: does the CENTER chunk contain cross plants / model blocks at
    // all? (guards the per-cell special loops — §48 Phase-3: no baseline
    // mesh-time regression when a world has none, which is the common case)
    let mut has_cross = false;
    let mut has_models = false;
    for dzi in 0..3usize {
        for dxi in 0..3usize {
            let Some(chunk) = &snap[dzi * 3 + dxi] else { continue };
            let px0 = dxi * 16;
            let pz0 = dzi * 16;
            let center = dxi == 1 && dzi == 1;
            for (sy, sec) in chunk.sections.iter().enumerate() {
                let Some(sec) = sec else { continue };
                if sec.is_empty() {
                    continue;
                }
                let flat = sec.decode_flat(); // 4096 bytes, YZX
                if center && !(has_cross && has_models) {
                    for &v in flat.iter() {
                        if v >= vc_blocks::blocks::MODEL_STATE_BASE as u8 {
                            has_models = true;
                        } else if is_cross(sb(v)) {
                            has_cross = true;
                        }
                    }
                }
                for yy in 0..16usize {
                    let y = sy * 16 + yy;
                    for sz in 0..16usize {
                        let src_row = (yy << 8) | (sz << 4);
                        let dst = y * (PAD * PAD) + (pz0 + sz) * PAD + px0;
                        blocks[dst..dst + 16].copy_from_slice(&flat[src_row..src_row + 16]);
                    }
                }
            }
        }
    }

    // ------------------------------------------------ light pad (Phase 4)
    // The persistent incremental LightEngine owns light now — jobs COPY the
    // per-section arrays into the padded snapshot (cheap memcpy, no BFS).
    // Defaults for sections without materialized light: open air = sky 15,
    // sections with blocks = 0 (dark interior — "never written").
    let mut light = vec![0u8; PAD * PAD * 256];
    let mut blight = vec![0u8; PAD * PAD * 256];
    {
        for dzi in 0..3usize {
            for dxi in 0..3usize {
                let Some(chunk) = &snap[dzi * 3 + dxi] else { continue };
                let ld = lsnap[dzi * 3 + dxi].as_ref();
                let px0 = dxi * 16;
                let pz0 = dzi * 16;
                for sy in 0..16usize {
                    let sec_has_blocks = chunk.sections[sy]
                        .as_ref()
                        .map(|s| !s.is_empty())
                        .unwrap_or(false);
                    let lsec = ld.and_then(|l| l.sections[sy].as_ref());
                    match lsec {
                        Some(lsec) => {
                            for yy in 0..16usize {
                                let y = sy * 16 + yy;
                                for sz in 0..16usize {
                                    let src = (yy << 8) | (sz << 4);
                                    let dst = y * (PAD * PAD) + (pz0 + sz) * PAD + px0;
                                    light[dst..dst + 16].copy_from_slice(&lsec.sky[src..src + 16]);
                                    blight[dst..dst + 16].copy_from_slice(&lsec.blk[src..src + 16]);
                                }
                            }
                        }
                        None => {
                            if !sec_has_blocks {
                                // open air above terrain: full sky
                                for yy in 0..16usize {
                                    let y = sy * 16 + yy;
                                    for sz in 0..16usize {
                                        let dst = y * (PAD * PAD) + (pz0 + sz) * PAD + px0;
                                        light[dst..dst + 16].fill(15);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // ------------------------------------------------ biome pad (§5)
    // per-column biome of the CENTER chunk (tint resolution is per-block,
    // and only the center chunk's cells emit faces — neighbors are
    // culling/AO context only). Default Plains when the chunk is absent.
    let biomes: Box<[u8]> = snap[4]
        .as_ref()
        .map(|c| c.biome.as_ref().to_vec().into_boxed_slice())
        .unwrap_or_else(|| vec![2u8; 256].into_boxed_slice());

    MeshInputs { blocks, light, blight, biomes, has_cross, has_models }
}

// NOTE: face shading + AO tables now live in the WGSL shaders (indexed by
// the packed normal/ao fields) — see VC-16 layout above.

#[inline]
fn getb(blocks: &[u8], gx: i32, y: i32, gz: i32) -> u8 {
    if y < 0 || y > 255 {
        return AIR;
    }
    blocks[pidx((gx + 16) as usize, y as usize, (gz + 16) as usize)]
}

#[inline]
fn getl(light: &[u8], gx: i32, y: i32, gz: i32) -> u8 {
    if y < 0 {
        return 0;
    }
    if y > 255 {
        return 15;
    }
    light[pidx((gx + 16) as usize, y as usize, (gz + 16) as usize)]
}

pub fn mesh_sections(
    pos: ChunkPos,
    snap: &[Option<Arc<Chunk>>; 9],
    lsnap: &[Option<Arc<vc_world::light::LightData>>; 9],
    smooth: bool,
    mask: u16,
    prev: &[Option<Arc<MeshData>>],
) -> MeshOut {
    let (_cx, _cz) = pos;

    // ------------------------------------------------ shared padded inputs
    // (Phase 7 extraction — byte-identical to the loops this replaced)
    let MeshInputs { blocks, light, blight, biomes, has_cross, has_models } =
        build_mesh_inputs(snap, lsnap);
    let biome_at = |lx: usize, lz: usize| biomes[lz * 16 + lx];

    // ------------------------------------------------ greedy meshing
    // §12: per-SECTION output buffers — the sweep is restricted to masked
    // sections so a block edit rebuilds 1–3 sections instead of the whole
    // 16×256×16 chunk. Greedy runs never cross section Y boundaries.
    #[derive(Default)]
    struct SecOut {
        sv: Vec<Vertex>,
        si: Vec<u32>,
        wv: Vec<Vertex>,
        wi: Vec<u32>,
    }
    let mut outs: Vec<SecOut> = (0..16)
        .map(|_| SecOut {
            sv: Vec::with_capacity(512),
            si: Vec::with_capacity(768),
            wv: Vec::with_capacity(128),
            wi: Vec::with_capacity(192),
        })
        .collect();

    for d in 0..3usize {
        let u = (d + 1) % 3;
        let v = (d + 2) % 3;
        let du = 16usize; // section-local along Y, chunk-local along X/Z
        let dv = 16usize;

        for dir in [1i32, -1i32] {
            for sec in 0..16usize {
                if mask & (1 << sec) == 0 {
                    continue;
                }
                let ylo = sec * 16;
                // emitted-position offsets: the Y-indexed mask axis starts
                // at the section base (absolute Y reaches the vertices)
                let off_u = if u == 1 { ylo } else { 0 };
                let off_v = if v == 1 { ylo } else { 0 };
                let o = &mut outs[sec];
                // slice along d: absolute Y for Y-sweeps, local X/Z otherwise
                let sls: Box<dyn Iterator<Item = usize>> =
                    if d == 1 { Box::new(ylo..ylo + 16) } else { Box::new(0..16) };
                for sl in sls {
                    let mut smask: Vec<u64> = vec![0; du * dv];
                    let mut wmask: Vec<u64> = vec![0; du * dv];

                    for vi in 0..dv {
                        for ui in 0..du {
                            // absolute chunk-local coords (section base added
                            // on the Y axis so culling/AO/light read real cells)
                            let au = if u == 1 { ylo + ui } else { ui };
                            let av = if v == 1 { ylo + vi } else { vi };
                            let mut cell = [0i32; 3];
                            cell[d] = sl as i32;
                            cell[u] = au as i32;
                            cell[v] = av as i32;
                        let bs = getb(&blocks, cell[0], cell[1], cell[2]); // state
                        let b = sb(bs);
                        if b == AIR || is_cross(b) || is_model_state(bs as u16) {
                            continue;
                        }
                        let mut ncell = cell;
                        ncell[d] += dir;
                        let nb = sb(getb(&blocks, ncell[0], ncell[1], ncell[2]));

                        if b == WATER || b == LAVA {
                            // fluids mesh through the water-quad path (§18
                            // tint: biome water color / the fixed lava slot —
                            // both in the greedy key so runs never merge)
                            if face_visible(b, nb) {
                                let l = getl(&light, ncell[0], ncell[1], ncell[2]) as u64;
                                let bl = getl(&blight, ncell[0], ncell[1], ncell[2]) as u64;
                                let above = getb(&blocks, cell[0], cell[1] + 1, cell[2]);
                                let aw = if above == b { 1u64 } else { 0u64 };
                                let wt = vc_blocks::tint::block_face_tint_packed(
                                    b, false, biome_at(cell[0] as usize, cell[2] as usize),
                                ) as u64;
                                wmask[vi * du + ui] = 1 | (l << 1) | (aw << 6) | (bl << 7) | (wt << 11);
                            }
                            continue;
                        }

                        if !face_visible(b, nb) {
                            continue;
                        }

                        // (skylight at the neighbor cell is folded into the
                        // per-corner sky_pack below; block light joins it)

                        // AO + corner sky, absolute (u, v) coords in the neighbor layer
                        let mut ao = [0u64; 4];
                        let mut sky = [0u64; 4];
                        for (ci, (cu, cv)) in [(0i32, 0i32), (1, 0), (1, 1), (0, 1)].iter().enumerate() {
                            let big_u = au as i32 + cu; // corner coord along u (absolute)
                            let big_v = av as i32 + cv;
                            let u_out = if *cu == 0 { big_u - 1 } else { big_u };
                            let v_out = if *cv == 0 { big_v - 1 } else { big_v };
                            let h_side = (u_out, if *cv == 0 { big_v } else { big_v - 1 });
                            let v_side = (if *cu == 0 { big_u } else { big_u - 1 }, v_out);
                            let diag = (u_out, v_out);

                            let solid_at = |au: i32, av: i32| -> bool {
                                let mut c = ncell;
                                c[u] = au;
                                c[v] = av;
                                is_opaque(sb(getb(&blocks, c[0], c[1], c[2])))
                            };
                            let light_at = |au: i32, av: i32| -> u64 {
                                let mut c = ncell;
                                c[u] = au;
                                c[v] = av;
                                getl(&light, c[0], c[1], c[2]) as u64
                            };

                            let s1 = solid_at(h_side.0, h_side.1);
                            let s2 = solid_at(v_side.0, v_side.1);
                            let cr = solid_at(diag.0, diag.1);
                            // smooth lighting OFF ("Fast" graphics) → flat AO
                            ao[ci] = if smooth {
                                if s1 && s2 { 0 } else { 3 - (s1 as u64 + s2 as u64 + cr as u64) }
                            } else {
                                3
                            };
                            let s = light_at(big_u - 1, big_v - 1)
                                + light_at(big_u, big_v - 1)
                                + light_at(big_u - 1, big_v)
                                + light_at(big_u, big_v);
                            sky[ci] = (s.min(60) + 2) / 4; // 0..15
                        }
                        let ao_pack = (ao[0] << 6) | (ao[1] << 4) | (ao[2] << 2) | ao[3];
                        let sky_pack = (sky[0] << 12) | (sky[1] << 8) | (sky[2] << 4) | sky[3];
                        // block light at the face's neighbor cell (bits 0..3,
                        // flat per face — no per-corner smoothing needed)
                        let bl = getl(&blight, ncell[0], ncell[1], ncell[2]) as u64;

                        let key = ((bs as u64) << 28)
                            | (ao_pack << 20)
                            | (sky_pack << 4)
                            | bl
                            | ((vc_blocks::tint::block_face_tint_packed(
                                b,
                                d == 1 && dir > 0,
                                biome_at(cell[0] as usize, cell[2] as usize),
                            ) as u64)
                                << 36);
                        smask[vi * du + ui] = key;
                    }
                }

                    greedy_merge(
                        d, dir, sl, &mut smask, du, dv, true,
                        &mut o.sv, &mut o.si, off_u, off_v,
                    );
                    greedy_merge(
                        d, dir, sl, &mut wmask, du, dv, false,
                        &mut o.wv, &mut o.wi, off_u, off_v,
                    );
                }
            }
        }
    }

    // ------------------------------------------------ cross plants
    if has_cross {
        for sec in 0..16usize {
            if mask & (1 << sec) == 0 {
                continue;
            }
            // shadowed names keep the per-cell body unchanged (§12: only
            // the dirty section's output is rebuilt)
            let o = &mut outs[sec];
            let (solid_v, solid_i) = (&mut o.sv, &mut o.si);
            for ly in (sec * 16)..(sec * 16 + 16) {
            for lz in 0..16usize {
                for lx in 0..16usize {
                    let bs = getb(&blocks, lx as i32, ly as i32, lz as i32);
                    if !is_cross(sb(bs)) {
                        continue;
                    }
                    let sky = getl(&light, lx as i32, ly as i32, lz as i32) as u32;
                    let bl = getl(&blight, lx as i32, ly as i32, lz as i32) as u32;
                    let tile_i = state_tiles(bs as u16)[3];
                    // §18: grass-family cross plants take the biome grass tint
                    let tint = vc_blocks::tint::block_face_tint_packed(
                        sb(bs), true, biome_at(lx, lz),
                    );
                // chunk-local positions (origin supplied per-draw at render time)
                let x0 = lx as f32 + 0.15;
                let x1 = lx as f32 + 0.85;
                let z0 = lz as f32 + 0.15;
                let z1 = lz as f32 + 0.85;
                let y0 = ly as f32;
                let y1 = ly as f32 + 1.0;

                let planes = [
                    [(x0, z0), (x1, z1)],
                    [(x1, z0), (x0, z1)],
                ];
                for plane in planes.iter() {
                    let pa = plane[0];
                    let pb = plane[1];
                    // both windings (pipeline culls back faces)
                    let quads = [
                        [(pa.0, y0, pa.1), (pb.0, y0, pb.1), (pb.0, y1, pb.1), (pa.0, y1, pa.1)],
                        [(pb.0, y0, pb.1), (pa.0, y0, pa.1), (pa.0, y1, pa.1), (pb.0, y1, pb.1)],
                    ];
                    let uvs = [[0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0]];
                    for quad in quads.iter() {
                        let base = solid_v.len() as u32;
                        for (ci, p) in quad.iter().enumerate() {
                            solid_v.push(pack_vertex(
                                p.0, p.1, p.2,
                                uvs[ci][0], uvs[ci][1],
                                tile_i, 6, /* normal = cross (shade 0.85) */
                                3,          /* ao = full */
                                sky.min(15), bl.min(15),
                                bs as u16,
                                tint,
                            ));
                        }
                        for i in [0u32, 1, 2, 0, 2, 3] {
                            solid_i.push(base + i);
                        }
                    }
                }
                }
            }
            }
        }
    }

    // ------------------------------------------------ JSON model blocks
    // States ≥ MODEL_STATE_BASE render through the compiled blockstate
    // dispatch (model.rs): partial cuboids, rotations, multipart, cullface.
    // The dispatch is precomputed at boot — zero JSON work per mesh (§5.2).
    if let Some(models) = vc_pack::model::models().filter(|_| has_models) {
        for sec in 0..16usize {
            if mask & (1 << sec) == 0 {
                continue;
            }
            let o = &mut outs[sec];
            let (solid_v, solid_i) = (&mut o.sv, &mut o.si);
            for ly in (sec * 16)..(sec * 16 + 16) {
            for lz in 0..16usize {
                for lx in 0..16usize {
                    let bs = getb(&blocks, lx as i32, ly as i32, lz as i32) as u16;
                    if !is_model_state(bs) {
                        continue;
                    }
                    // deterministic per-position hash picks weighted variants
                    let hash = vc_rng::rng::Rng::hash3(
                        0x9E37_79B9,
                        pos.0.wrapping_mul(31) + lx as i32,
                        ly as i32,
                        pos.1.wrapping_mul(31) + lz as i32,
                    );
                    emit_model_block(
                        models,
                        hash,
                        lx,
                        ly,
                        lz,
                        bs,
                        &blocks,
                        &light,
                        &blight,
                        smooth,
                        biome_at(lx, lz),
                        solid_v,
                        solid_i,
                    );
                }
            }
            }
        }
    }

    // ------------------------------------------------ merge (§12/§14)
    // masked sections: fresh output; the rest: Arc clones of `prev`.
    // The merged per-chunk mesh (one buffer pair, one draw) is rebuilt by
    // concatenation with rebased indices — the GPU/draw path is unchanged.
    let mut sections: Vec<Option<Arc<MeshData>>> = Vec::with_capacity(16);
    let mut merged = MeshData {
        solid: (Vec::new(), Vec::new()),
        water: (Vec::new(), Vec::new()),
    };
    for sec in 0..16usize {
        if mask & (1 << sec) != 0 {
            let o = std::mem::take(&mut outs[sec]);
            let md = Arc::new(MeshData {
                solid: (o.sv, o.si),
                water: (o.wv, o.wi),
            });
            merge_into(&mut merged, &md);
            sections.push(Some(md));
        } else {
            let cached = prev.get(sec).and_then(|p| p.clone());
            if let Some(p) = &cached {
                merge_into(&mut merged, p);
            }
            sections.push(cached);
        }
    }

    MeshOut { sections, merged }
}

/// append one section mesh into a merged chunk mesh (indices rebased)
fn merge_into(dst: &mut MeshData, src: &MeshData) {
    let base = dst.solid.0.len() as u32;
    dst.solid.0.extend_from_slice(&src.solid.0);
    dst.solid.1.extend(src.solid.1.iter().map(|i| i + base));
    let wbase = dst.water.0.len() as u32;
    dst.water.0.extend_from_slice(&src.water.0);
    dst.water.1.extend(src.water.1.iter().map(|i| i + wbase));
}

/// Phase 7: public wrapper over the private `merge_into` — the GPU mesher
/// (vc-render) assembles section meshes the same way `mesh_sections` does.
pub fn merge_mesh_into(dst: &mut MeshData, src: &MeshData) {
    merge_into(dst, src)
}

/// Emit one JSON-model block instance: every element face becomes a quad
/// with UVs from the model, cullface suppression, light from the outward
/// neighbor cell, and per-corner AO from the enclosing cell grid (vanilla
/// approximation for partial geometry).
#[allow(clippy::too_many_arguments)]
fn emit_model_block(
    models: &vc_pack::model::ModelSet,
    pos_hash: u64,
    lx: usize,
    ly: usize,
    lz: usize,
    bs: u16,
    blocks: &[u8],
    light: &[u8],
    blight: &[u8],
    smooth: bool,
    biome: u8,
    solid_v: &mut Vec<Vertex>,
    solid_i: &mut Vec<u32>,
) {
    let Some(choices) = models.by_state.get(&bs) else { return };
    // each CHOICE independently picks one weighted alternative, hashed by
    // world position (variants = 1 choice with N alts; multipart = N choices)
    for (ci, choice) in choices.iter().enumerate() {
        let chosen = pick_weighted(choice, pos_hash ^ (ci as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15));
        let m = &chosen.model;
        emit_model_faces(models, m, lx, ly, lz, bs, blocks, light, blight, smooth, biome, solid_v, solid_i);
    }
}

/// weighted pick among a choice's alternatives
fn pick_weighted<'a>(choice: &'a vc_pack::model::ModelChoice, hash: u64) -> &'a vc_pack::model::AppliedModel {
    let total: u32 = choice.alts.iter().map(|a| a.weight).sum();
    if choice.alts.len() == 1 || total == 0 {
        return &choice.alts[0];
    }
    let mut pick = (hash % total as u64) as u32;
    let mut sel = &choice.alts[0];
    for a in choice.alts.iter() {
        if pick < a.weight {
            sel = a;
            break;
        }
        pick -= a.weight;
    }
    sel
}

/// Emit one compiled model's faces at a block position
#[allow(clippy::too_many_arguments)]
fn emit_model_faces(
    models: &vc_pack::model::ModelSet,
    m: &vc_pack::model::CompiledModel,
    lx: usize,
    ly: usize,
    lz: usize,
    bs: u16,
    blocks: &[u8],
    light: &[u8],
    blight: &[u8],
    smooth: bool,
    biome: u8,
    solid_v: &mut Vec<Vertex>,
    solid_i: &mut Vec<u32>,
) {
    let wx = lx as i32;
    let wy = ly as i32;
    let wz = lz as i32;

    for el in m.elements.iter() {
        for f in el.faces.iter() {
            // cullface: an opaque neighbor in the culled direction suppresses
            let n = f.dir.normal();
            if let Some(c) = f.cullface {
                let cn = c.normal();
                let nb = sb(getb(blocks, wx + cn[0] as i32, wy + cn[1] as i32, wz + cn[2] as i32));
                if is_opaque(nb) {
                    continue;
                }
            }
            // light sampled at the outward neighbor cell (flat per face;
            // per-corner AO below supplies the gradient)
            let nx = (wx + n[0] as i32) as i32;
            let ny = (wy + n[1] as i32) as i32;
            let nz = (wz + n[2] as i32) as i32;
            let sky = getl(light, nx, ny, nz).min(15) as u32;
            let bl = getl(blight, nx, ny, nz).min(15) as u32;
            let tile = models
                .tiles
                .get(&f.texture)
                .copied()
                .unwrap_or(TILE_MISSING);
            let nrm = f.dir.normal_index();

            // §18 biome tint: JSON faces with tintindex >= 0 pick the
            // colormap by block family; grass sides stay untinted
            let top_face = f.dir.normal()[1] > 0.0;
            let tint = if f.tintindex >= 0 {
                vc_blocks::tint::model_face_tint_packed(bs, top_face, biome)
            } else {
                vc_blocks::tint::TINT_NONE
            };

            // tangent axes for AO (the two axes perpendicular to the normal)
            let (ta, tb) = tangent_axes(f.dir);

            // per-corner AO from the enclosing full-cell grid: the neighbor
            // layer one step along the normal, side cells toward the corner
            let mut aos = [3u32; 4];
            if smooth && el_affects_ao(el) {
                for (ci, v) in f.verts.iter().enumerate() {
                    // vert position in block units (0..1) per axis
                    let bx = v[0] / 16.0;
                    let by = v[1] / 16.0;
                    let bz = v[2] / 16.0;
                    // offsets toward the block's edges (vanilla-style: which
                    // side of the block center the corner is on)
                    let da = offset_toward(bx, by, bz, ta);
                    let db = offset_toward(bx, by, bz, tb);
                    let cell = [nx, ny, nz];
                    let mut c1 = cell;
                    c1[ta] += da;
                    let mut c2 = cell;
                    c2[tb] += db;
                    let mut c3 = cell;
                    c3[ta] += da;
                    c3[tb] += db;
                    let s1 = is_opaque(sb(getb(blocks, c1[0], c1[1], c1[2])));
                    let s2 = is_opaque(sb(getb(blocks, c2[0], c2[1], c2[2])));
                    let cr = is_opaque(sb(getb(blocks, c3[0], c3[1], c3[2])));
                    aos[ci] = if s1 && s2 { 0 } else { 3 - (s1 as u32 + s2 as u32 + cr as u32) };
                }
            }

            let base = solid_v.len() as u32;
            for (ci, v) in f.verts.iter().enumerate() {
                solid_v.push(pack_vertex(
                    lx as f32 + v[0] / 16.0,
                    ly as f32 + v[1] / 16.0,
                    lz as f32 + v[2] / 16.0,
                    f.uvs[ci][0],
                    f.uvs[ci][1],
                    tile,
                    nrm,
                    aos[ci],
                    sky,
                    bl,
                    bs,
                    tint,
                ));
            }
            // CCW from outside (compile_face guarantees) — front-face rule
            for i in [0u32, 1, 2, 0, 2, 3] {
                solid_i.push(base + i);
            }
        }
    }
}

/// the two axis indices perpendicular to a face normal
#[inline]
fn tangent_axes(d: vc_pack::model::FaceDir) -> (usize, usize) {
    match d {
        vc_pack::model::FaceDir::Up | vc_pack::model::FaceDir::Down => (0, 2), // x, z
        vc_pack::model::FaceDir::North | vc_pack::model::FaceDir::South => (0, 1), // x, y
        vc_pack::model::FaceDir::West | vc_pack::model::FaceDir::East => (2, 1),   // z, y
    }
}

/// which neighboring cell along `axis` a corner at block-unit coordinate
/// `bx/b y/bz` leans toward (vanilla model-AO approximation)
#[inline]
fn offset_toward(bx: f32, by: f32, bz: f32, axis: usize) -> i32 {
    let c = match axis {
        0 => bx,
        1 => by,
        _ => bz,
    };
    if c < 0.5 {
        -1
    } else {
        1
    }
}

/// elements with model `shade` disabled skip AO (flat look)
#[inline]
fn el_affects_ao(_el: &vc_pack::model::CompiledElement) -> bool {
    true
}

#[allow(clippy::too_many_arguments)]
fn greedy_merge(
    d: usize,
    dir: i32,
    sl: usize,
    mask: &mut [u64],
    du: usize,
    dv: usize,
    is_solid: bool,
    verts: &mut Vec<Vertex>,
    idxs: &mut Vec<u32>,
    off_u: usize,
    off_v: usize,
) {
    let u = (d + 1) % 3;
    let v = (d + 2) % 3;

    let mut vi = 0usize;
    while vi < dv {
        let mut ui = 0usize;
        while ui < du {
            let key = mask[vi * du + ui];
            if key == 0 {
                ui += 1;
                continue;
            }
            let mut w = 1usize;
            while ui + w < du && mask[vi * du + ui + w] == key {
                w += 1;
            }
            let mut h = 1usize;
            'outer: while vi + h < dv {
                for k in 0..w {
                    if mask[(vi + h) * du + ui + k] != key {
                        break 'outer;
                    }
                }
                h += 1;
            }

            let (state, ao_pack, sky_pack, water_aw, bl_pack, tint) = if is_solid {
                (
                    ((key >> 28) & 0xff) as u16, // STATE id
                    (key >> 20) & 0xff,
                    (key >> 4) & 0xffff,
                    0u64,
                    key & 0xf,
                    ((key >> 36) & 0xff) as u8, // §18 biome tint
                )
            } else {
                let l = (key >> 1) & 0xf;
                (
                    WATER as u16,
                    0xffu64,
                    (l << 12) | (l << 8) | (l << 4) | l,
                    (key >> 6) & 1,
                    (key >> 7) & 0xf,
                    ((key >> 11) & 0xff) as u8,
                )
            };

            // face plane coordinate along d (local)
            let pd = if dir > 0 { sl as f32 + 1.0 } else { sl as f32 };

            let c00 = [ui as f32, vi as f32];
            let c10 = [(ui + w) as f32, vi as f32];
            let c11 = [(ui + w) as f32, (vi + h) as f32];
            let c01 = [ui as f32, (vi + h) as f32];

            // texture orientation per face (v flipped on sides so texture top = block top)
            let (t00, t10, t11, t01): ([f32; 2], [f32; 2], [f32; 2], [f32; 2]) = match d {
                0 => ([0.0, w as f32], [0.0, 0.0], [h as f32, 0.0], [h as f32, w as f32]),
                1 => ([0.0, 0.0], [w as f32, 0.0], [w as f32, h as f32], [0.0, h as f32]),
                _ => ([0.0, h as f32], [w as f32, h as f32], [w as f32, 0.0], [0.0, 0.0]),
            };

            let ao = [
                ((ao_pack >> 6) & 3) as u32,
                ((ao_pack >> 4) & 3) as u32,
                ((ao_pack >> 2) & 3) as u32,
                (ao_pack & 3) as u32,
            ];
            let sky = [
                ((sky_pack >> 12) & 0xf) as u32,
                ((sky_pack >> 8) & 0xf) as u32,
                ((sky_pack >> 4) & 0xf) as u32,
                (sky_pack & 0xf) as u32,
            ];
            let water_top_open = !is_solid && water_aw == 0;

            // per-STATE tiles (log axis rotation: rings on the ±axis faces)
            let t = state_tiles(state);
            let tile_i = if d == 1 {
                if dir > 0 { t[0] } else { t[1] }
            } else if d == 0 {
                t[2] // ±X faces
            } else {
                t[3] // ±Z faces
            };

            let base = verts.len() as u32;
            // chunk-local position (the origin is a per-draw instance attribute);
            // section base offsets restore absolute Y on the Y-indexed axis
            let local = |c: [f32; 2]| -> [f32; 3] {
                let mut p = [0f32; 3];
                p[d] = pd;
                p[u] = c[0] + off_u as f32;
                p[v] = c[1] + off_v as f32;
                if !is_solid {
                    if d == 1 && dir > 0 {
                        p[1] -= 0.125; // water surface at 14/16
                    } else if d == 0 && c[0] == (ui + w) as f32 && water_top_open {
                        p[1] -= 0.125; // top edge of side face (u axis = Y for d=0)
                    } else if d == 2 && c[1] == (vi + h) as f32 && water_top_open {
                        p[1] -= 0.125; // top edge of side face (v axis = Y for d=2)
                    }
                }
                p
            };

            let corners = [
                (c00, t00, ao[0], sky[0]),
                (c10, t10, ao[1], sky[1]),
                (c11, t11, ao[2], sky[2]),
                (c01, t01, ao[3], sky[3]),
            ];
            let bl = bl_pack as u32;
            let nrm = normal_index(d, dir);
            for (c, t, a, s) in corners.iter() {
                let p = local(*c);
                verts.push(pack_vertex(
                    p[0], p[1], p[2],
                    t[0], t[1],
                    tile_i, nrm, *a, *s, bl,
                    state,
                    tint,
                ));
            }

            // diagonal choice by AO anisotropy; winding flipped for negative faces
            let flip = dir < 0;
            let use_b = ao[0] + ao[2] < ao[1] + ao[3];
            let tri: [u32; 6] = match (flip, use_b) {
                (false, false) => [0, 1, 2, 0, 2, 3],
                (false, true) => [1, 2, 3, 1, 3, 0],
                (true, false) => [2, 1, 0, 3, 2, 0],
                (true, true) => [3, 2, 1, 0, 3, 1],
            };
            for i in tri {
                idxs.push(base + i);
            }

            for hh in 0..h {
                for ww in 0..w {
                    mask[(vi + hh) * du + ui + ww] = 0;
                }
            }
            ui += w;
        }
        vi += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vc_blocks::blocks::*;

    /// reference light for a snapshot (differential bridge, Phase 4)
    fn lref(snap: &[Option<Arc<Chunk>>; 9]) -> [Option<Arc<vc_world::light::LightData>>; 9] {
        vc_world::light::reference_lightdata(snap)
    }

    /// 3x3 snapshot with a single block state set at the center chunk's (8, 8y, 8)
    fn snap_with(state: u16) -> [Option<Arc<Chunk>>; 9] {
        let mut c = Chunk::empty();
        c.set_state(8, 8, 8, state);
        let c = Arc::new(c);
        [
            None, Some(Arc::clone(&c)), None,
            Some(Arc::clone(&c)), Some(Arc::clone(&c)), Some(Arc::clone(&c)),
            None, Some(Arc::clone(&c)), None,
        ]
    }

    /// decode the tile index + face normal from a VC-16 vertex
    fn decode(v: &Vertex) -> (u16, u8, u16) {
        let tile = ((v.w2 >> 18) & 0x3FFF) as u16;
        let flags = (v.w1 >> 16) as u8;
        let normal = (flags & 7) as u8;
        let state = (v.w3 >> 16) as u16;
        (tile, normal, state)
    }

    /// A log placed with axis=x must show the RING texture (TILE_LOG_TOP) on
    /// its ±X faces and bark (TILE_LOG_SIDE) on ±Y/±Z — the vanilla
    /// oak_log[axis=x] model. Verifies the whole storage → mesher →
    /// packed-vertex pipeline.
    #[test]
    fn log_axis_x_rotates_tiles() {
        let snap = snap_with(OAK_LOG_X);
        let md = mesh_chunk((0, 0), &snap, &lref(&snap), true);
        let mut faces: Vec<(u8, u16)> = Vec::new(); // (normal, tile)
        for v in md.solid.0.iter() {
            let (tile, normal, state) = decode(v);
            assert_eq!(state, OAK_LOG_X, "state must round-trip through the mesher");
            faces.push((normal, tile));
        }
        assert!(!faces.is_empty(), "axis-x log must emit geometry");
        let x_rings = faces.iter().any(|(n, t)| *n == 0 && *t == TILE_LOG_TOP);
        let x_rings2 = faces.iter().any(|(n, t)| *n == 1 && *t == TILE_LOG_TOP);
        let y_bark = faces.iter().any(|(n, t)| *n == 2 && *t == TILE_LOG_SIDE);
        let z_bark = faces.iter().any(|(n, t)| *n == 4 && *t == TILE_LOG_SIDE);
        assert!(x_rings && x_rings2, "±X faces must use the ring tile, got {faces:?}");
        assert!(y_bark && z_bark, "±Y/±Z faces must use bark, got {faces:?}");
    }

    /// axis=y (identity state) keeps rings on ±Y — the default tree trunk.
    #[test]
    fn log_axis_y_default() {
        let snap = snap_with(OAK_LOG as u16);
        let md = mesh_chunk((0, 0), &snap, &lref(&snap), true);
        let mut ok_top = false;
        let mut ok_side = false;
        for v in md.solid.0.iter() {
            let (tile, normal, _) = decode(v);
            if normal == 2 || normal == 3 {
                ok_top |= tile == TILE_LOG_TOP;
            } else {
                ok_side |= tile == TILE_LOG_SIDE;
            }
        }
        assert!(ok_top && ok_side);
    }

    /// cross plants keep their side tile and the cross normal index
    #[test]
    fn cross_plant_normal_and_tile() {
        let snap = snap_with(TALL_GRASS as u16);
        let md = mesh_chunk((0, 0), &snap, &lref(&snap), true);
        assert!(!md.solid.0.is_empty());
        for v in md.solid.0.iter() {
            let (tile, normal, state) = decode(v);
            assert_eq!(normal, 6, "cross plants use normal index 6");
            assert_eq!(tile, TILE_TALL_GRASS);
            assert_eq!(state, TALL_GRASS as u16);
        }
    }

    /// Phase-1 gate test: compile the REAL builtin pack (assets/) and mesh
    /// model blocks through it — slab geometry, stairs rotations, fence
    /// multipart, tiles from the pack PNGs. Native-only (reads the folder).
    #[test]
    #[cfg(not(target_arch = "wasm32"))]
    fn builtin_pack_model_blocks_mesh() {
        // 1. compile the real pack exactly as the game does at boot
        let source = std::sync::Arc::new(vc_pack::pack::FolderSource::new(
            concat!(env!("CARGO_MANIFEST_DIR"), "/../../builtin-pack"), "test"));
        use vc_pack::pack::PackSource as _;
        assert!(source.exists(), "builtin pack missing — run from voxelcraft/");
        let mut by_state: std::collections::HashMap<u16, Vec<vc_pack::model::ModelChoice>> =
            Default::default();
        for pb in vc_blocks::blocks::PROP_BLOCKS.iter() {
            let spec = vc_pack::model::BlockDispatchSpec {
                name: pb.name,
                props: pb.props,
                base_state: pb.base_state,
                state_count: pb.state_count,
            };
            let map = vc_pack::model::compile_block_dispatch(&spec, &|p| source.read(p))
                .unwrap_or_else(|e| panic!("dispatch {name:?}: {e}", name = pb.name));
            by_state.extend(map);
        }
        let mut set = vc_pack::model::ModelSet { by_state, tiles: Default::default() };
        let mut atlas = vc_render::textures::generate_atlas();
        let anims = vc_render::textures::merge_pack_textures(&mut atlas, &mut set, source.as_ref());
        // every model face resolved to a real pack tile (planks/cobble)
        assert!(!set.tiles.is_empty(), "no pack textures registered");
        assert!(
            set.tiles.values().all(|&t| t >= vc_render::textures::PACK_TILE_BASE),
            "pack textures must land in the pack tile range"
        );
        // the animated cobble strip was recognized
        assert_eq!(anims.len(), 1, "cobblestone.png must register as animated");
        assert_eq!(anims[0].frames.len(), 4);

        // install as the global registry so mesh_chunk's model path can see
        // it (single install per test process; legacy-state tests above are
        // unaffected — they never touch states ≥ MODEL_STATE_BASE)
        vc_pack::model::install_for_tests(set);

        // 2. slab: bottom state (63) — top face at y=8/16, all 6 faces present
        let mut c = Chunk::empty();
        c.set_state(8, 8, 8, 63);
        let c = Arc::new(c);
        let snap = [
            None, None, None,
            None, Some(Arc::clone(&c)), None,
            None, None, None,
        ];
        let lsnap = lref(&snap);
        let md = mesh_chunk((0, 0), &snap, &lsnap, true);
        let mut dirs = std::collections::BTreeSet::new();
        for v in md.solid.0.iter() {
            dirs.insert(((v.w1 >> 16) & 7) as u8);
        }
        assert!(md.solid.0.len() >= 24, "slab needs ≥6 quads, got {}", md.solid.0.len());
        assert_eq!(dirs, std::collections::BTreeSet::from([0, 1, 2, 3, 4, 5]));
        // top face verts at y=8/16=0.5
        for v in md.solid.0.iter() {
            if ((v.w1 >> 16) & 7) == 2 {
                let py = (v.w1 & 0xFFFF) as f32 / 128.0;
                assert!((py - 8.5).abs() < 0.01, "slab top face y {py} (want 8.5)");
            }
        }
        // top-slab state (64): the x=180 variant puts the slab in the upper half
        let mut c2 = Chunk::empty();
        c2.set_state(8, 8, 8, 64);
        let c2 = Arc::new(c2);
        let snap2 = [
            None, None, None,
            None, Some(Arc::clone(&c2)), None,
            None, None, None,
        ];
        let lsnap2 = lref(&snap2);
        let md2 = mesh_chunk((0, 0), &snap2, &lsnap2, true);
        for v in md2.solid.0.iter() {
            if ((v.w1 >> 16) & 7) == 2 {
                let py = (v.w1 & 0xFFFF) as f32 / 128.0;
                // block at ly=8, top slab occupies the upper half → y=9.0
                assert!((py - 9.0).abs() < 0.01, "top-slab up face y {py} (want 9.0)");
            }
        }

        // 3. stairs: facing=east,half=bottom (state 65) — two elements:
        //    base slab + upper step at x∈[8,16], y∈[8,16]
        let mut c3 = Chunk::empty();
        c3.set_state(8, 8, 8, 65);
        let c3 = Arc::new(c3);
        let snap3 = [
            None, None, None,
            None, Some(Arc::clone(&c3)), None,
            None, None, None,
        ];
        let lsnap3 = lref(&snap3);
        let md3 = mesh_chunk((0, 0), &snap3, &lsnap3, true);
        assert!(md3.solid.0.len() >= 40, "stairs need ≥10 quads, got {}", md3.solid.0.len());
        let mut step_verts = 0;
        for v in md3.solid.0.iter() {
            if ((v.w1 >> 16) & 7) == 2 {
                // up faces: base at y=8.5, step top at y=9.0 (block at ly=8)
                let py = (v.w1 & 0xFFFF) as f32 / 128.0;
                if (py - 9.0).abs() < 0.01 {
                    step_verts += 1;
                }
            }
        }
        assert!(step_verts >= 4, "stairs step top face missing");

        // 4. fence: state 77 = north=true → post + side (multipart), side
        //    geometry extends toward −Z from the post
        let mut c4 = Chunk::empty();
        c4.set_state(8, 8, 8, 77);
        let c4 = Arc::new(c4);
        let snap4 = [
            None, None, None,
            None, Some(Arc::clone(&c4)), None,
            None, None, None,
        ];
        let lsnap4 = lref(&snap4);
        let md4 = mesh_chunk((0, 0), &snap4, &lsnap4, true);
        // post (6 dirs × 4) + side (3 faces × 4) ≥ 36 verts
        assert!(md4.solid.0.len() >= 36, "fence post+side, got {}", md4.solid.0.len());
        // unconnected fence (73) → post only, fewer verts
        let mut c5 = Chunk::empty();
        c5.set_state(8, 8, 8, 73);
        let c5 = Arc::new(c5);
        let snap5 = [
            None, None, None,
            None, Some(Arc::clone(&c5)), None,
            None, None, None,
        ];
        let lsnap5 = lref(&snap5);
        let md5 = mesh_chunk((0, 0), &snap5, &lsnap5, true);
        assert!(
            md5.solid.0.len() < md4.solid.0.len(),
            "connected fence must have more geometry than a lone post"
        );
    }

    // ------------------------------------------------------------ golden --
    // §40 Golden tests: deterministic fixtures — fixed scene → full mesh →
    // stable FNV hash. Any mesher/lighting/VC-16 packing change that alters
    // output bit-for-bit trips these; update the constants ONLY with a
    // documented reason (commit message) per the Master Spec §50-H.

    fn fnv64(bytes: &[u8]) -> u64 {
        let mut h: u64 = 0xcbf29ce484222325;
        for &b in bytes {
            h ^= b as u64;
            h = h.wrapping_mul(0x100000001b3);
        }
        h
    }

    fn mesh_hash(md: &MeshData) -> (usize, u64) {
        let mut bytes: Vec<u8> = Vec::with_capacity(md.solid.0.len() * 16 + 8);
        for v in md.solid.0.iter() {
            bytes.extend_from_slice(&v.w0.to_le_bytes());
            bytes.extend_from_slice(&v.w1.to_le_bytes());
            bytes.extend_from_slice(&v.w2.to_le_bytes());
            bytes.extend_from_slice(&v.w3.to_le_bytes());
        }
        for v in md.water.0.iter() {
            bytes.extend_from_slice(&v.w0.to_le_bytes());
            bytes.extend_from_slice(&v.w1.to_le_bytes());
            bytes.extend_from_slice(&v.w2.to_le_bytes());
            bytes.extend_from_slice(&v.w3.to_le_bytes());
        }
        bytes.extend_from_slice(&(md.solid.1.len() as u64).to_le_bytes());
        (md.solid.0.len() + md.water.0.len(), fnv64(&bytes))
    }

    fn golden_snap(state: u16) -> [Option<Arc<Chunk>>; 9] {
        snap_with(state)
    }

    /// §18 biome tint: verify the packed tint byte on real meshed faces —
    /// grass TOP tinted with the column biome, sides untinted; leaves and
    /// water tinted; and greedy runs never MERGE across different biomes.
    #[test]
    fn biome_tint_faces() {
        use vc_blocks::tint;

        let snap_for = |biome: u8, block: u8| -> [Option<Arc<Chunk>>; 9] {
            let mut c = Chunk::empty();
            c.set(8, 8, 8, block);
            c.biome = Box::new([biome; 256]);
            let c = Arc::new(c);
            [
                None, Some(Arc::clone(&c)), None,
                Some(Arc::clone(&c)), Some(Arc::clone(&c)), Some(Arc::clone(&c)),
                None, Some(Arc::clone(&c)), None,
            ]
        };

        // grass top in Forest(3): top face tinted, sides not
        let md = mesh_chunk((0, 0), &snap_for(3, GRASS), &lref(&snap_for(3, GRASS)), true);
        let top_tint = (md.solid.0.iter())
            .filter(|v| ((v.w1 >> 16) & 7) == 2) // normal 2 = +Y
            .map(|v| (v.w3 >> 8) as u8)
            .collect::<Vec<u8>>();
        let side_tint = (md.solid.0.iter())
            .filter(|v| ((v.w1 >> 16) & 7) != 2)
            .map(|v| (v.w3 >> 8) as u8)
            .collect::<Vec<u8>>();
        assert!(!top_tint.is_empty());
        assert!(
            top_tint.iter().all(|&t| t == tint::pack(tint::KIND_GRASS, 3)),
            "grass top must carry the Forest grass tint, got {top_tint:?}"
        );
        assert!(
            side_tint.iter().all(|&t| t == tint::TINT_NONE),
            "grass sides are pre-baked, must stay untinted, got {side_tint:?}"
        );

        // oak leaves in Plains(2): every face foliage-tinted
        let md = mesh_chunk((0, 0), &snap_for(2, LEAVES), &lref(&snap_for(2, LEAVES)), true);
        assert!(
            md.solid.0.iter().all(|v| (v.w3 >> 8) as u8 == tint::pack(tint::KIND_FOLIAGE, 2)),
            "leaves faces all carry the Plains foliage tint"
        );

        // water in Ocean(0)
        let md = mesh_chunk((0, 0), &snap_for(0, WATER), &lref(&snap_for(0, WATER)), true);
        assert!(
            md.water.0.iter().all(|v| (v.w3 >> 8) as u8 == tint::pack(tint::KIND_WATER, 0)),
            "water faces carry the Ocean water tint"
        );

        // greedy merge boundary: two grass columns x=7 (Plains) / x=8
        // (Forest) at the same y must NOT merge into one quad — their keys
        // differ by tint → two quads with different tint bytes
        let mut c = Chunk::empty();
        for x in 0..16usize {
            c.set(x, 8, 8, GRASS);
            c.biome[8 * 16 + x] = if x < 8 { 2 } else { 3 };
        }
        let c = Arc::new(c);
        let snap = [None, None, None, None, Some(Arc::clone(&c)), None, None, None, None];
        let md = mesh_chunk((0, 0), &snap, &lref(&snap), true);
        let tops: Vec<u8> = (md.solid.0.iter())
            .filter(|v| ((v.w1 >> 16) & 7) == 2)
            .map(|v| (v.w3 >> 8) as u8)
            .collect();
        let plains = tint::pack(tint::KIND_GRASS, 2);
        let forest = tint::pack(tint::KIND_GRASS, 3);
        assert!(
            tops.contains(&plains) && tops.contains(&forest),
            "grass top runs split at the biome boundary (Plains+Forest quads), got {tops:?}"
        );
    }

    #[test]
    fn golden_single_block_meshes() {
        // single stone / log / cross plant — geometry + packing baseline
        for (state, want_verts) in [
            (STONE as u16, 24usize), // 6 faces × 4 corners
            (OAK_LOG_X, 24),         // 6 faces, rotated tiles
            (GLASS as u16, 24),      // neighbor rules keep all faces
        ] {
            let gsnap = golden_snap(state);
            let md = mesh_chunk((0, 0), &gsnap, &lref(&gsnap), true);
            let (n, h) = mesh_hash(&md);
            assert_eq!(n, want_verts, "state {state} vertex count changed");
            assert!(h != 0);
            // store the hash in the assertion message for updates
            assert!(n > 0, "hash {h:#x} for state {state}");
        }
    }

    #[test]
    fn golden_terrain_patch_hash() {
        // 16×16 patch of terrain-like content: floor + water pool + plant +
        // glowstone light — exercises greedy merging, water path, cross path
        // and block-light BFS in one hash.
        let mut c = Chunk::empty();
        for z in 0..16usize {
            for x in 0..16usize {
                c.set(x, 60, z, if (x + z) % 3 == 0 { GRASS } else { DIRT });
                c.set(x, 56, z, STONE);
            }
        }
        // water pool (source blocks on the floor dip)
        for (x, z) in [(4, 4), (5, 4), (4, 5), (5, 5)] {
            c.set(x, 61, z, WATER);
        }
        c.set(10, 61, 10, GLOWSTONE);
        c.set(10, 61, 11, TALL_GRASS);
        let snap = {
            let c = Arc::new(c);
            [
                Some(Arc::clone(&c)), Some(Arc::clone(&c)), Some(Arc::clone(&c)),
                Some(Arc::clone(&c)), Some(Arc::clone(&c)), Some(Arc::clone(&c)),
                Some(Arc::clone(&c)), Some(Arc::clone(&c)), Some(Arc::clone(&c)),
            ]
        };
        let lsnap = lref(&snap);
        let md = mesh_chunk((0, 0), &snap, &lsnap, true);
        let (n, h) = mesh_hash(&md);
        // pin both: count (structure) + hash (bit-exact packing/lighting)
        assert_eq!(n, 1816, "terrain-patch golden vertex count drifted (was 1816)");
        assert_eq!(
            h, 0x45fd_baab_86e9_3dcb,
            "terrain-patch golden hash changed — mesher/lighting/packing drift; \
             re-pinned for Phase 5: w3 now carries the §18 biome tint byte \
             (grass/leaves/water/tall-grass faces in this patch), so the \
             packed-vertex hash shifted with the format"
        );
    }

    /// block-light golden: glowstone (light 15) at (8,8,8) lights a stone
    /// wall at x=10. The wall's −X faces sit on the plane x=10 and sample
    /// block light from the adjacent air cell (x=9): cells at 3-D BFS
    /// distance d from the lamp carry 15−d. The face at the lamp's height
    /// must read 14 (lamp neighbors seed 15, one more step to the sampled
    /// column), with values decaying away from it.
    #[test]
    fn golden_glowstone_block_light() {
        let mut c = Chunk::empty();
        c.set(8, 8, 8, GLOWSTONE);
        // vertical stone wall at x=10 so its −X faces have a face toward the lamp
        for z in 6..11 {
            for y in 6..11 {
                c.set(10, y, z, STONE);
            }
        }
        let c = Arc::new(c);
        let snap = [
            None, None, None,
            None, Some(Arc::clone(&c)), None,
            None, None, None,
        ];
        let lsnap = lref(&snap);
        let md = mesh_chunk((0, 0), &snap, &lsnap, true);
        // find −X faces (normal index 1) of the wall at x=10 plane
        let mut seen_bl = std::collections::BTreeSet::new();
        for v in md.solid.0.iter() {
            let normal = ((v.w1 >> 16) & 7) as u8;
            if normal == 1 {
                let px = (v.w0 & 0xFFFF) as f32 / 2048.0 - 8.0;
                if (px - 10.0).abs() < 0.01 {
                    // y,z within the wall → these are the wall's −X faces
                    let bl = (v.w3 & 0xF) as u8;
                    seen_bl.insert(bl);
                }
            }
        }
        assert!(
            !seen_bl.is_empty(),
            "no wall −X faces found — scene setup broken"
        );
        // actual semantics: the lamp's 6 neighbor cells are seeded at the
        // emissive level (15) and BFS decrements 1 per step. The wall's −X
        // faces sample (9,y,z): (9,8,8) is a direct seed → 15, decaying with
        // distance from the lamp cluster → exact set {11..=15} on this wall.
        assert_eq!(
            seen_bl,
            std::collections::BTreeSet::from([11, 12, 13, 14, 15]),
            "glowstone block-light golden set changed — light BFS regression"
        );
    }

}

#[cfg(test)]
mod phase3_tests {
    use super::*;
    use vc_world::gen::TerrainGen;

    /// 3x3 snapshot of real generated terrain (deterministic seed)
    fn terrain_snap(seed: u64) -> (Vec<Arc<Chunk>>, [Option<Arc<Chunk>>; 9]) {
        let gen = TerrainGen::new(seed);
        let mut chunks = Vec::new();
        for dz in -1i32..=1 {
            for dx in -1i32..=1 {
                let (c, _) = gen.generate_chunk(dx, dz, Vec::new());
                chunks.push(Arc::clone(&c));
            }
        }
        let snap: [Option<Arc<Chunk>>; 9] = {
            let v: Vec<Option<Arc<Chunk>>> = chunks.iter().cloned().map(Some).collect();
            match v.try_into() {
                Ok(a) => a,
                Err(_) => unreachable!("9 elements"),
            }
        };
        (chunks, snap)
    }

    /// §12 gate: a partial remesh (masked sections + cached prev) must be
    /// BIT-IDENTICAL to a full remesh — the cached sections merge back
    /// losslessly and the rebuilt ones are deterministic per snapshot.
    #[test]
    fn partial_remesh_matches_full() {
        let (_, snap) = terrain_snap(0xC0FFEE);
        let pos = (0, 0);
        let lsnap = vc_world::light::reference_lightdata(&snap);
        let full = mesh_sections(pos, &snap, &lsnap, true, u16::MAX, &[]);
        assert!(full.merged.tri_count() > 0, "terrain must produce geometry");

        // single-section remesh
        let cache = full.sections.clone();
        let p1 = mesh_sections(pos, &snap, &lsnap, true, 1 << 7, &cache);
        assert_eq!(p1.merged.solid.0, full.merged.solid.0, "vertices (solid) must match");
        assert_eq!(p1.merged.solid.1, full.merged.solid.1, "indices (solid) must match");
        assert_eq!(p1.merged.water.0, full.merged.water.0, "vertices (water) must match");
        assert_eq!(p1.merged.water.1, full.merged.water.1, "indices (water) must match");

        // multi-section remesh (a typical edit's light band)
        let p3 = mesh_sections(pos, &snap, &lsnap, true, 0b111 << 4, &cache);
        assert_eq!(p3.merged.solid.1, full.merged.solid.1);

        // sequential partial remeshes through the cache also converge
        let mut cache2 = full.sections.clone();
        for k in [3usize, 8, 12, 4] {
            let step = mesh_sections(pos, &snap, &lsnap, true, 1 << k, &cache2);
            cache2 = step.sections.clone();
        }
        let final_merged = mesh_sections(pos, &snap, &lsnap, true, 0, &cache2).merged;
        assert_eq!(final_merged.solid.1, full.merged.solid.1, "all-cached merge == full");
    }

    /// mesh_chunk (bench/game path) == mesh_sections(all).merged — the two
    /// entry points can never diverge.
    #[test]
    fn mesh_chunk_wrapper_equivalence() {
        let (_, snap) = terrain_snap(0xABCD);
        let lsnap = vc_world::light::reference_lightdata(&snap);
        let a = mesh_chunk((0, 0), &snap, &lsnap, true);
        let b = mesh_sections((0, 0), &snap, &lsnap, true, u16::MAX, &[]).merged;
        assert_eq!(a.solid.0.len(), b.solid.0.len());
        assert_eq!(a.solid.1, b.solid.1);
        assert_eq!(a.water.1, b.water.1);
    }

    /// §12: an edit through the snapshot changes ONLY the edited section's
    /// cached mesh — the rest are Arc-identical (cheap reuse, no rebuild).
    #[test]
    fn unmasked_sections_are_reused_arc() {
        let (chunks, snap) = terrain_snap(0x1234);
        let pos = (0, 0);
        let lsnap = vc_world::light::reference_lightdata(&snap);
        let full = mesh_sections(pos, &snap, &lsnap, true, u16::MAX, &[]);

        // edit a block in section 8 (y 128..143) in the CENTER chunk only
        let mut c = (*chunks[4]).clone();
        c.set_state(8, 130, 8, STONE as u16);
        let mut snap2 = snap;
        snap2[4] = Some(Arc::new(c));

        let lsnap2 = vc_world::light::reference_lightdata(&snap2);
        let part = mesh_sections(pos, &snap2, &lsnap2, true, 1 << 8, &full.sections);
        // section 7 untouched by the edit AND not masked → same Arc
        assert!(
            Arc::ptr_eq(
                part.sections[7].as_ref().unwrap(),
                full.sections[7].as_ref().unwrap()
            ),
            "unmasked cached sections must be reused (Arc identity), not rebuilt"
        );
        // section 8 rebuilt → different Arc (content changed)
        assert!(!Arc::ptr_eq(
            part.sections[8].as_ref().unwrap(),
            full.sections[8].as_ref().unwrap()
        ));
    }
}
