//! Greedy mesher + voxel lighting (skylight column scan + lateral BFS) + AO.
//! Pure function over a 3x3 chunk snapshot → safe on worker threads.

use crate::blocks::*;
use crate::chunk::Chunk;
use crate::world::ChunkPos;
use std::collections::VecDeque;
use std::sync::Arc;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable, Debug)]
pub struct Vertex {
    pub pos: [f32; 3],
    /// block-unit uv (0..w, 0..h) — shader applies fract() for per-block tiling
    pub uv: [f32; 2],
    /// atlas tile coordinates (0..16, 0..16)
    pub tile: [f32; 2],
    /// geometry light = face shade * AO
    pub light: f32,
    /// skylight 0..1
    pub sky: f32,
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

const AO_MULT: [f32; 4] = [0.42, 0.62, 0.80, 1.0];

#[inline]
fn face_shade(d: usize, dir: i32) -> f32 {
    match d {
        1 => if dir > 0 { 1.0 } else { 0.5 },
        2 => 0.8,
        _ => 0.6,
    }
}

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
    let (cx, cz) = pos;
    let wox = cx * 16;
    let woz = cz * 16;

    // ------------------------------------------------ copy blocks (padded)
    let mut blocks = vec![0u8; PAD * PAD * 256];
    for dzi in 0..3usize {
        for dxi in 0..3usize {
            let Some(chunk) = &snap[dzi * 3 + dxi] else { continue };
            let px0 = dxi * 16;
            let pz0 = dzi * 16;
            for y in 0..256usize {
                for sz in 0..16usize {
                    let src = &chunk.blocks[y * 256 + sz * 16..y * 256 + sz * 16 + 16];
                    let dst = y * (PAD * PAD) + (pz0 + sz) * PAD + px0;
                    blocks[dst..dst + 16].copy_from_slice(src);
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
                let b = blocks[pidx(x, y, z)];
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
                if !is_opaque(blocks[np]) && light[np] < nl {
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
                        let b = getb(&blocks, cell[0], cell[1], cell[2]);
                        if b == AIR || is_cross(b) {
                            continue;
                        }
                        let mut ncell = cell;
                        ncell[d] += dir;
                        let nb = getb(&blocks, ncell[0], ncell[1], ncell[2]);

                        if b == WATER {
                            if face_visible(WATER, nb) {
                                let l = getl(&light, ncell[0], ncell[1], ncell[2]) as u64;
                                let above = getb(&blocks, cell[0], cell[1] + 1, cell[2]);
                                let aw = if above == WATER { 1u64 } else { 0u64 };
                                wmask[vi * du + ui] = 1 | (l << 1) | (aw << 6);
                            }
                            continue;
                        }

                        if !face_visible(b, nb) {
                            continue;
                        }

                        let nl = getl(&light, ncell[0], ncell[1], ncell[2]) as u64;

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
                                is_opaque(getb(&blocks, c[0], c[1], c[2]))
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

                        let key = ((b as u64) << 28) | (ao_pack << 20) | (sky_pack << 4) | nl;
                        smask[vi * du + ui] = key;
                    }
                }

                greedy_merge(d, dir, sl, &mut smask, du, dv, true, wox, woz, &mut solid_v, &mut solid_i);
                greedy_merge(d, dir, sl, &mut wmask, du, dv, false, wox, woz, &mut water_v, &mut water_i);
            }
        }
    }

    // ------------------------------------------------ cross plants
    for ly in 0..256usize {
        for lz in 0..16usize {
            for lx in 0..16usize {
                let b = getb(&blocks, lx as i32, ly as i32, lz as i32);
                if !is_cross(b) {
                    continue;
                }
                let sky = getl(&light, lx as i32, ly as i32, lz as i32) as f32 / 15.0;
                let tile_i = def(b).tiles[2];
                let tile = [tile_i as f32 % 16.0, (tile_i as f32 / 16.0).floor()];
                let x0 = wox as f32 + lx as f32 + 0.15;
                let x1 = wox as f32 + lx as f32 + 0.85;
                let z0 = woz as f32 + lz as f32 + 0.15;
                let z1 = woz as f32 + lz as f32 + 0.85;
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
                            solid_v.push(Vertex {
                                pos: [p.0, p.1, p.2],
                                uv: uvs[ci],
                                tile,
                                light: 0.85,
                                sky,
                            });
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
    wox: i32,
    woz: i32,
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

            let (block, ao_pack, sky_pack, water_aw) = if is_solid {
                (((key >> 28) & 0xff) as u8, (key >> 20) & 0xff, (key >> 4) & 0xffff, 0u64)
            } else {
                let l = (key >> 1) & 0xf;
                (WATER, 0xffu64, (l << 12) | (l << 8) | (l << 4) | l, (key >> 6) & 1)
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
                ((ao_pack >> 6) & 3) as usize,
                ((ao_pack >> 4) & 3) as usize,
                ((ao_pack >> 2) & 3) as usize,
                (ao_pack & 3) as usize,
            ];
            let sky = [
                ((sky_pack >> 12) & 0xf) as f32 / 15.0,
                ((sky_pack >> 8) & 0xf) as f32 / 15.0,
                ((sky_pack >> 4) & 0xf) as f32 / 15.0,
                (sky_pack & 0xf) as f32 / 15.0,
            ];
            let shade = face_shade(d, dir);
            let water_top_open = !is_solid && water_aw == 0;

            let tiles = def(block).tiles;
            let tile_i = if d == 1 {
                if dir > 0 { tiles[0] } else { tiles[1] }
            } else {
                tiles[2]
            };
            let tile = [tile_i as f32 % 16.0, (tile_i as f32 / 16.0).floor()];

            let base = verts.len() as u32;
            let world = |c: [f32; 2]| -> [f32; 3] {
                let mut p = [0f32; 3];
                p[d] = pd;
                p[u] = c[0];
                p[v] = c[1];
                p[0] += wox as f32;
                p[2] += woz as f32;
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
            for (c, t, a, s) in corners.iter() {
                verts.push(Vertex {
                    pos: world(*c),
                    uv: *t,
                    tile,
                    light: shade * AO_MULT[*a],
                    sky: *s,
                });
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
