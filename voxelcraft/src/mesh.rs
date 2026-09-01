//! Greedy mesher + voxel lighting (skylight column scan + lateral BFS) + AO
//! + JSON-model path (blockstate dispatch, Phase 1).
//! Pure function over a 3x3 chunk snapshot → safe on worker threads.

use crate::blocks::*;
use crate::chunk::Chunk;
use crate::world::ChunkPos;
use std::collections::VecDeque;
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
///   w3 =  state:u16 << 16 | reserved:u8 << 8 | sky:u4 << 4 | block:u4
///                                 — state = block id today, block-state id
///                                 after the BlockState registry lands (u16
///                                 headroom per the research doc)
///
/// normal: 0=+X 1=−X 2=+Y 3=−Y 4=+Z 5=−Z 6=cross-plants (shade tables in
/// the shaders reproduce the old `face_shade * AO_MULT` exactly). bias:u2 is
/// reserved for a future texture-bleed inset sign (shader currently ignores).
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable, Debug)]
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
        w3: ((state as u32) << 16) | ((sky & 0xF) << 4) | (block & 0xF),
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

// padded 48 x 256 x 48 region covering the 3x3 snapshot
const PAD: usize = 48;
#[inline]
fn pidx(x: usize, y: usize, z: usize) -> usize {
    y * (PAD * PAD) + z * PAD + x
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

pub fn mesh_chunk(pos: ChunkPos, snap: &[Option<Arc<Chunk>>; 9], smooth: bool) -> MeshData {
    let (_cx, _cz) = pos;

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
                        if v >= crate::blocks::MODEL_STATE_BASE as u8 {
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

    // ------------------------------------------------ skylight
    let mut light = vec![0u8; PAD * PAD * 256];
    let mut surface = [[-1i32; PAD]; PAD];

    for z in 0..PAD {
        for x in 0..PAD {
            let mut l: i32 = 15;
            for y in (0..256usize).rev() {
                let b = sb(blocks[pidx(x, y, z)]);
                if is_opaque(b) {
                    l = 0;
                    if surface[z][x] < 0 {
                        surface[z][x] = y as i32;
                    }
                } else if b == WATER {
                    l = (l - 2).max(0);
                } else if b == LEAVES {
                    l = (l - 1).max(0);
                }
                light[pidx(x, y, z)] = l as u8;
            }
        }
    }

    // lateral BFS seeds: bright cells next to darker columns
    let mut queue: VecDeque<(usize, u8)> = VecDeque::new();
    for z in 0..PAD {
        for x in 0..PAD {
            let sy_here = surface[z][x];
            let nbrs = [
                (x.wrapping_add(1), z),
                (x.wrapping_sub(1), z),
                (x, z.wrapping_add(1)),
                (x, z.wrapping_sub(1)),
            ];
            for (nx, nz) in nbrs {
                if nx >= PAD || nz >= PAD {
                    continue;
                }
                let sy_n = surface[nz][nx];
                if sy_n > sy_here {
                    let lo = (sy_here + 1).max(0) as usize;
                    let hi = sy_n.min(sy_here + 16).max(0) as usize;
                    for y in lo..=hi.min(255) {
                        let la = light[pidx(x, y, z)];
                        let lb = light[pidx(nx, y, nz)];
                        if la >= 2 && la > lb.saturating_add(1) {
                            queue.push_back((pidx(x, y, z), la));
                        }
                    }
                }
            }
        }
    }

    // BFS flood (lateral + vertical, decrementing)
    while let Some((p, l)) = queue.pop_front() {
        if l < 2 {
            continue;
        }
        let x = p % PAD;
        let z = (p / PAD) % PAD;
        let y = p / (PAD * PAD);
        let nl = l - 1;
        macro_rules! prop {
            ($x:expr, $y:expr, $z:expr) => {{
                let np = pidx($x, $y, $z);
                if !is_opaque(sb(blocks[np])) && light[np] < nl {
                    light[np] = nl;
                    if nl > 1 {
                        queue.push_back((np, nl));
                    }
                }
            }};
        }
        if x > 0 {
            prop!(x - 1, y, z);
        }
        if x + 1 < PAD {
            prop!(x + 1, y, z);
        }
        if y > 0 {
            prop!(x, y - 1, z);
        }
        if y + 1 < 256 {
            prop!(x, y + 1, z);
        }
        if z > 0 {
            prop!(x, y, z - 1);
        }
        if z + 1 < PAD {
            prop!(x, y, z + 1);
        }
    }

    // ------------------------------------------------ block light
    // Emissive blocks (glowstone = 15) light their surroundings; BFS
    // flood-decrements like MC's block-light channel. Kept separate from
    // skylight because it is independent of time-of-day and sun shadows.
    let mut blight = vec![0u8; PAD * PAD * 256];
    let mut bqueue: VecDeque<(usize, u8)> = VecDeque::new();
    for y in 0..256usize {
        for z in 0..PAD {
            for x in 0..PAD {
                let b = sb(blocks[pidx(x, y, z)]);
                let e = emissive(b);
                if e == 0 {
                    continue;
                }
                // seed the non-opaque neighbors at full source level
                let lvl = e.min(15);
                macro_rules! seed {
                    ($x:expr, $y:expr, $z:expr) => {{
                        if $x < PAD && $z < PAD && $y < 256 {
                            let np = pidx($x, $y, $z);
                            if !is_opaque(sb(blocks[np])) && blight[np] < lvl {
                                blight[np] = lvl;
                                bqueue.push_back((np, lvl));
                            }
                        }
                    }};
                }
                if x > 0 { seed!(x - 1, y, z); }
                if x + 1 < PAD { seed!(x + 1, y, z); }
                if y > 0 { seed!(x, y - 1, z); }
                if y + 1 < 256 { seed!(x, y + 1, z); }
                if z > 0 { seed!(x, y, z - 1); }
                if z + 1 < PAD { seed!(x, y, z + 1); }
            }
        }
    }
    // BFS flood (lateral + vertical, decrementing)
    while let Some((p, l)) = bqueue.pop_front() {
        if l < 2 {
            continue;
        }
        let x = p % PAD;
        let z = (p / PAD) % PAD;
        let y = p / (PAD * PAD);
        let nl = l - 1;
        macro_rules! bprop {
            ($x:expr, $y:expr, $z:expr) => {{
                let np = pidx($x, $y, $z);
                if !is_opaque(sb(blocks[np])) && blight[np] < nl {
                    blight[np] = nl;
                    if nl > 1 {
                        bqueue.push_back((np, nl));
                    }
                }
            }};
        }
        if x > 0 {
            bprop!(x - 1, y, z);
        }
        if x + 1 < PAD {
            bprop!(x + 1, y, z);
        }
        if y > 0 {
            bprop!(x, y - 1, z);
        }
        if y + 1 < 256 {
            bprop!(x, y + 1, z);
        }
        if z > 0 {
            bprop!(x, y, z - 1);
        }
        if z + 1 < PAD {
            bprop!(x, y, z + 1);
        }
    }

    // ------------------------------------------------ greedy meshing
    let mut solid_v: Vec<Vertex> = Vec::with_capacity(8192);
    let mut solid_i: Vec<u32> = Vec::with_capacity(12288);
    let mut water_v: Vec<Vertex> = Vec::with_capacity(512);
    let mut water_i: Vec<u32> = Vec::with_capacity(768);

    let dims = [16usize, 256usize, 16usize];

    for d in 0..3usize {
        let u = (d + 1) % 3;
        let v = (d + 2) % 3;
        let du = dims[u];
        let dv = dims[v];

        for dir in [1i32, -1i32] {
            for sl in 0..dims[d] {
                let mut smask: Vec<u64> = vec![0; du * dv];
                let mut wmask: Vec<u64> = vec![0; du * dv];

                for vi in 0..dv {
                    for ui in 0..du {
                        let mut cell = [0i32; 3];
                        cell[d] = sl as i32;
                        cell[u] = ui as i32;
                        cell[v] = vi as i32;
                        let bs = getb(&blocks, cell[0], cell[1], cell[2]); // state
                        let b = sb(bs);
                        if b == AIR || is_cross(b) || is_model_state(bs as u16) {
                            continue;
                        }
                        let mut ncell = cell;
                        ncell[d] += dir;
                        let nb = sb(getb(&blocks, ncell[0], ncell[1], ncell[2]));

                        if b == WATER {
                            if face_visible(WATER, nb) {
                                let l = getl(&light, ncell[0], ncell[1], ncell[2]) as u64;
                                let bl = getl(&blight, ncell[0], ncell[1], ncell[2]) as u64;
                                let above = getb(&blocks, cell[0], cell[1] + 1, cell[2]);
                                let aw = if above == WATER { 1u64 } else { 0u64 };
                                wmask[vi * du + ui] = 1 | (l << 1) | (aw << 6) | (bl << 7);
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
                            let big_u = ui as i32 + cu; // corner coord along u (cell ui or ui+1)
                            let big_v = vi as i32 + cv;
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

                        let key = ((bs as u64) << 28) | (ao_pack << 20) | (sky_pack << 4) | bl;
                        smask[vi * du + ui] = key;
                    }
                }

                greedy_merge(d, dir, sl, &mut smask, du, dv, true, &mut solid_v, &mut solid_i);
                greedy_merge(d, dir, sl, &mut wmask, du, dv, false, &mut water_v, &mut water_i);
            }
        }
    }

    // ------------------------------------------------ cross plants
    if has_cross {
        for ly in 0..256usize {
            for lz in 0..16usize {
                for lx in 0..16usize {
                    let bs = getb(&blocks, lx as i32, ly as i32, lz as i32);
                    if !is_cross(sb(bs)) {
                        continue;
                    }
                    let sky = getl(&light, lx as i32, ly as i32, lz as i32) as u32;
                    let bl = getl(&blight, lx as i32, ly as i32, lz as i32) as u32;
                    let tile_i = state_tiles(bs as u16)[3];
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

    // ------------------------------------------------ JSON model blocks
    // States ≥ MODEL_STATE_BASE render through the compiled blockstate
    // dispatch (model.rs): partial cuboids, rotations, multipart, cullface.
    // The dispatch is precomputed at boot — zero JSON work per mesh (§5.2).
    if let Some(models) = crate::model::models().filter(|_| has_models) {
        for ly in 0..256usize {
            for lz in 0..16usize {
                for lx in 0..16usize {
                    let bs = getb(&blocks, lx as i32, ly as i32, lz as i32) as u16;
                    if !is_model_state(bs) {
                        continue;
                    }
                    // deterministic per-position hash picks weighted variants
                    let hash = crate::rng::Rng::hash3(
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
                        &mut solid_v,
                        &mut solid_i,
                    );
                }
            }
        }
    }

    MeshData {
        solid: (solid_v, solid_i),
        water: (water_v, water_i),
    }
}

/// Emit one JSON-model block instance: every element face becomes a quad
/// with UVs from the model, cullface suppression, light from the outward
/// neighbor cell, and per-corner AO from the enclosing cell grid (vanilla
/// approximation for partial geometry).
#[allow(clippy::too_many_arguments)]
fn emit_model_block(
    models: &crate::model::ModelSet,
    pos_hash: u64,
    lx: usize,
    ly: usize,
    lz: usize,
    bs: u16,
    blocks: &[u8],
    light: &[u8],
    blight: &[u8],
    smooth: bool,
    solid_v: &mut Vec<Vertex>,
    solid_i: &mut Vec<u32>,
) {
    let Some(choices) = models.by_state.get(&bs) else { return };
    // each CHOICE independently picks one weighted alternative, hashed by
    // world position (variants = 1 choice with N alts; multipart = N choices)
    for (ci, choice) in choices.iter().enumerate() {
        let chosen = pick_weighted(choice, pos_hash ^ (ci as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15));
        let m = &chosen.model;
        emit_model_faces(models, m, lx, ly, lz, bs, blocks, light, blight, smooth, solid_v, solid_i);
    }
}

/// weighted pick among a choice's alternatives
fn pick_weighted<'a>(choice: &'a crate::model::ModelChoice, hash: u64) -> &'a crate::model::AppliedModel {
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
    models: &crate::model::ModelSet,
    m: &crate::model::CompiledModel,
    lx: usize,
    ly: usize,
    lz: usize,
    bs: u16,
    blocks: &[u8],
    light: &[u8],
    blight: &[u8],
    smooth: bool,
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
fn tangent_axes(d: crate::model::FaceDir) -> (usize, usize) {
    match d {
        crate::model::FaceDir::Up | crate::model::FaceDir::Down => (0, 2), // x, z
        crate::model::FaceDir::North | crate::model::FaceDir::South => (0, 1), // x, y
        crate::model::FaceDir::West | crate::model::FaceDir::East => (2, 1),   // z, y
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
fn el_affects_ao(_el: &crate::model::CompiledElement) -> bool {
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

            let (state, ao_pack, sky_pack, water_aw, bl_pack) = if is_solid {
                (
                    ((key >> 28) & 0xff) as u16, // STATE id
                    (key >> 20) & 0xff,
                    (key >> 4) & 0xffff,
                    0u64,
                    key & 0xf,
                )
            } else {
                let l = (key >> 1) & 0xf;
                (
                    WATER as u16,
                    0xffu64,
                    (l << 12) | (l << 8) | (l << 4) | l,
                    (key >> 6) & 1,
                    (key >> 7) & 0xf,
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
            // chunk-local position (the origin is a per-draw instance attribute)
            let local = |c: [f32; 2]| -> [f32; 3] {
                let mut p = [0f32; 3];
                p[d] = pd;
                p[u] = c[0];
                p[v] = c[1];
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
    use crate::blocks::*;

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
        let md = mesh_chunk((0, 0), &snap_with(OAK_LOG_X), true);
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
        let md = mesh_chunk((0, 0), &snap_with(OAK_LOG as u16), true);
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
        let md = mesh_chunk((0, 0), &snap_with(TALL_GRASS as u16), true);
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
        let source = std::sync::Arc::new(crate::pack::FolderSource::new("builtin-pack", "test"));
        use crate::pack::PackSource as _;
        assert!(source.exists(), "builtin pack missing — run from voxelcraft/");
        let mut by_state: std::collections::HashMap<u16, Vec<crate::model::ModelChoice>> =
            Default::default();
        for pb in crate::blocks::PROP_BLOCKS.iter() {
            let spec = crate::model::BlockDispatchSpec {
                name: pb.name,
                props: pb.props,
                base_state: pb.base_state,
                state_count: pb.state_count,
            };
            let map = crate::model::compile_block_dispatch(&spec, &|p| source.read(p))
                .unwrap_or_else(|e| panic!("dispatch {name:?}: {e}", name = pb.name));
            by_state.extend(map);
        }
        let mut set = crate::model::ModelSet { by_state, tiles: Default::default() };
        let mut atlas = crate::textures::generate_atlas();
        let anims = crate::textures::merge_pack_textures(&mut atlas, &mut set, source.as_ref());
        // every model face resolved to a real pack tile (planks/cobble)
        assert!(!set.tiles.is_empty(), "no pack textures registered");
        assert!(
            set.tiles.values().all(|&t| t >= crate::textures::PACK_TILE_BASE),
            "pack textures must land in the pack tile range"
        );
        // the animated cobble strip was recognized
        assert_eq!(anims.len(), 1, "cobblestone.png must register as animated");
        assert_eq!(anims[0].frames.len(), 4);

        // install as the global registry so mesh_chunk's model path can see
        // it (single install per test process; legacy-state tests above are
        // unaffected — they never touch states ≥ MODEL_STATE_BASE)
        crate::model::install_for_tests(set);

        // 2. slab: bottom state (63) — top face at y=8/16, all 6 faces present
        let mut c = Chunk::empty();
        c.set_state(8, 8, 8, 63);
        let c = Arc::new(c);
        let snap = [
            None, None, None,
            None, Some(Arc::clone(&c)), None,
            None, None, None,
        ];
        let md = mesh_chunk((0, 0), &snap, true);
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
        let md2 = mesh_chunk((0, 0), &snap2, true);
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
        let md3 = mesh_chunk((0, 0), &snap3, true);
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
        let md4 = mesh_chunk((0, 0), &snap4, true);
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
        let md5 = mesh_chunk((0, 0), &snap5, true);
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

    #[test]
    fn golden_single_block_meshes() {
        // single stone / log / cross plant — geometry + packing baseline
        for (state, want_verts) in [
            (STONE as u16, 24usize), // 6 faces × 4 corners
            (OAK_LOG_X, 24),         // 6 faces, rotated tiles
            (GLASS as u16, 24),      // neighbor rules keep all faces
        ] {
            let md = mesh_chunk((0, 0), &golden_snap(state), true);
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
        let md = mesh_chunk((0, 0), &snap, true);
        let (n, h) = mesh_hash(&md);
        // pin both: count (structure) + hash (bit-exact packing/lighting)
        assert_eq!(n, 1816, "terrain-patch golden vertex count drifted (was 1816)");
        assert_eq!(
            h, 0x50d2_e83f_fc55_05eb,
            "terrain-patch golden hash changed — mesher/lighting/packing drift; \
             if intentional, re-pin with justification (Master Spec §50-H)"
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
        let md = mesh_chunk((0, 0), &snap, true);
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
