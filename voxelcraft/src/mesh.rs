//! Greedy mesher + voxel lighting (skylight column scan + lateral BFS) + AO.
//! Pure function over a 3x3 chunk snapshot → safe on worker threads.

use crate::blocks::*;
use crate::chunk::Chunk;
use crate::world::ChunkPos;
use std::collections::VecDeque;
use std::sync::Arc;

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
    for dzi in 0..3usize {
        for dxi in 0..3usize {
            let Some(chunk) = &snap[dzi * 3 + dxi] else { continue };
            let px0 = dxi * 16;
            let pz0 = dzi * 16;
            for (sy, sec) in chunk.sections.iter().enumerate() {
                let Some(sec) = sec else { continue };
                if sec.is_empty() {
                    continue;
                }
                let flat = sec.decode_flat(); // 4096 bytes, YZX
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
                        if b == AIR || is_cross(b) {
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

    MeshData {
        solid: (solid_v, solid_i),
        water: (water_v, water_i),
    }
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
}
