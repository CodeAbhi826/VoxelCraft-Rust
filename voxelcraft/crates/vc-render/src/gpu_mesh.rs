//! Phase 7 — GPU compute mesher: a WGSL port of the CPU greedy core
//! (dossier Part 1 §4: "GPU compute for meshing/world-gen … genuinely
//! bleeding-edge"; technique reference only — the algorithm structure
//! [sweep → per-slice face masks → greedy rect merge] is the classic
//! published greedy-meshing formulation, no reference code copied).
//!
//! Design contract: **bit-identical output** to `vc_mesh::mesh::mesh_sections`
//! for the standard greedy path (worlds without cross plants / JSON-model
//! states — the model/cross special paths stay on the CPU, see the game's
//! `Job::Mesh` dispatch). Parity is structural:
//!
//! * the same padded input volumes (built by the ONE shared builder
//!   `vc_mesh::mesh::build_mesh_inputs`);
//! * per-(axis, dir, section, slice) "unit" masks built with the exact
//!   CPU cell-coordinate mapping, u64 greedy keys split into (lo, hi) u32
//!   pairs (WGSL has no u64);
//! * the same greedy rect expansion order (row scan → width run → height
//!   growth) and the same rect zero-out / `ui += w` advancement;
//! * emission at PRECOMPUTED fixed offsets (counting-sort layout): pass A
//!   counts quads per unit, the CPU computes offsets in the exact CPU
//!   emission order (d → dir[+,-] → sec → sl, solid stream then water
//!   stream), pass B emits at those offsets — **no atomics**, so the
//!   output layout is deterministic and byte-identical by construction;
//! * identical VC-16 packing arithmetic (IEEE-f32 fixed-point rounds,
//!   integer bit packing, AO-flip winding, water-surface rules).
//!
//! Pipeline: `main_count` (mask build + quad count) then `main_emit`
//! (mask rebuild + emission), one workgroup per unit × per job,
//! 256 threads (one thread per mask cell; thread 0 runs the merge scan —
//! the parallelism is across the 1536 units, mirroring how the CPU's
//! inner loops are per-slice sequential but slices are independent).
//!
//! wgpu 22 constraints honored:
//! * `max_storage_buffers_per_shader_stage` = 8 (WebGPU floor) — ONE bind
//!   group layout with exactly 8 storage bindings, shared by both passes
//!   (binding 5 is dual-purpose: pass A writes quad COUNTS, the CPU then
//!   rewrites the same buffer with OFFSETS for pass B; bindings 6/7 are
//!   the vertex/index output streams — pass A binds 16-byte dummies since
//!   the emit branch is runtime-false there, but the static call graph
//!   reaches them so the layout must cover them);
//! * per-job state (section mask, smooth flag, 256 biome ids) rides ONE
//!   params buffer indexed by job instead of per-job bind groups.

use std::collections::VecDeque;
use std::sync::mpsc::{Receiver, TryRecvError};
use std::sync::Arc;

use vc_mesh::mesh::{merge_mesh_into, MeshData, MeshInputs, Vertex, PAD};
use vc_world::world::ChunkPos;
use wgpu::util::DeviceExt;

use vc_blocks::blocks::{
    BLOCK_COUNT, BIRCH_LEAVES, GLASS, GRASS, ICE, LEAVES, SPRUCE_LEAVES, STATE_COUNT, TALL_GRASS,
    WATER,
};

/// number of (axis, dir, section, slice) units per chunk = 3·2·16·16
pub const UNITS: usize = 1536;
/// words per job in the params buffer: mask + smooth + 64 packed biome u32
const PARAM_STRIDE: usize = 66;

// ---------------------------------------------------------------------------
// WGSL — the compute port (validated by naga in `shader_tests`, executed by
// the parity/E2E tests; see the design contract above).
// ---------------------------------------------------------------------------

pub const MESH_COMPUTE_SHADER: &str = r#"
// VC greedy-mesh GPU port — bit-parity port of vc-mesh/src/mesh.rs.
// Layout constants (must match the Rust side):
const VOL_WORDS: u32 = 147456u;    // 48*256*48 bytes / 4
const B_WATER: u32 = 9u;           // block id of WATER (identity state)
const MODEL_BASE: u32 = 63u;       // MODEL_STATE_BASE
const L_SB: u32 = 0u;              // lut: state -> block        (235)
const L_FL: u32 = 235u;            // lut: block flags           (102)
const L_TC: u32 = 337u;            // lut: block tint class      (102)
const L_ST: u32 = 439u;            // lut: state tiles, 4/state  (940)
const P_N: u32 = 0u;               // params[0] = n_jobs
const P_JOB: u32 = 2u;             // params job base = 2 + j*66
const P_BIOME: u32 = 2u;           // biomes at job base + 2 (64 packed u32)

// block flag bits (block_flags LUT)
const F_OPAQUE: u32 = 1u;
const F_WATER: u32 = 2u;
const F_LEAVES: u32 = 4u;
const F_GLASS: u32 = 8u;
const F_ICE: u32 = 16u;
const F_CROSS: u32 = 32u;

// corner (cu, cv) order — CPU: [(0,0),(1,0),(1,1),(0,1)]; computed
// arithmetically (WGSL const arrays are constant-indexed only)
fn corner_u(ci: u32) -> u32 { return select(0u, 1u, (ci == 1u) || (ci == 2u)); }
fn corner_v(ci: u32) -> u32 { return select(0u, 1u, (ci == 2u) || (ci == 3u)); }

@group(0) @binding(0) var<storage, read> lut: array<u32>;
@group(0) @binding(1) var<storage, read> blocks: array<u32>;
@group(0) @binding(2) var<storage, read> sky_l: array<u32>;
@group(0) @binding(3) var<storage, read> blk_l: array<u32>;
@group(0) @binding(4) var<storage, read> params: array<u32>;
@group(0) @binding(5) var<storage, read_write> mesh_data: array<u32>;
@group(0) @binding(6) var<storage, read_write> verts_out: array<u32>;
@group(0) @binding(7) var<storage, read_write> idx_out: array<u32>;

var<workgroup> smask_lo: array<u32, 256>;
var<workgroup> smask_hi: array<u32, 256>;
var<workgroup> wmask: array<u32, 256>;

// padded-volume accessors (byte p in a 48*256*48 job-local region)
fn job_getb(j: u32, x: i32, y: i32, z: i32) -> u32 {
    if (y < 0) || (y > 255) { return 0u; }
    let p = u32(y * 2304 + z * 48 + x);
    let base = j * VOL_WORDS;
    return (blocks[base + (p >> 2u)] >> ((p & 3u) * 8u)) & 0xFFu;
}
fn job_get_sky(j: u32, x: i32, y: i32, z: i32) -> u32 {
    if (y < 0) { return 0u; }
    if (y > 255) { return 15u; }
    let p = u32(y * 2304 + z * 48 + x);
    let base = j * VOL_WORDS;
    return (sky_l[base + (p >> 2u)] >> ((p & 3u) * 8u)) & 0xFFu;
}
fn job_get_blk(j: u32, x: i32, y: i32, z: i32) -> u32 {
    if (y < 0) { return 0u; }
    if (y > 255) { return 15u; }
    let p = u32(y * 2304 + z * 48 + x);
    let base = j * VOL_WORDS;
    return (blk_l[base + (p >> 2u)] >> ((p & 3u) * 8u)) & 0xFFu;
}
fn sb(s: u32) -> u32 { return lut[L_SB + min(s, 234u)]; }
fn fl(b: u32) -> u32 { return lut[L_FL + min(b, 101u)]; }
fn biome_at(j: u32, x: i32, z: i32) -> u32 {
    let c = u32(z * 16 + x);
    let w = params[P_JOB + j * 66u + P_BIOME + (c >> 2u)];
    return (w >> ((c & 3u) * 8u)) & 0xFFu;
}
fn tile_of(state: u32, i: u32) -> u32 { return lut[L_ST + state * 4u + i]; }

// face_visible port (flags-encoded special classes — identical semantics
// to vc_blocks::face_visible: water/leaves/glass/ice arms then the
// general !opaque rule)
fn face_visible(bf: u32, fnb: u32) -> bool {
    if (bf & F_WATER) != 0u { return (fnb & F_OPAQUE) == 0u && (fnb & F_WATER) == 0u; }
    if (bf & F_LEAVES) != 0u { return (fnb & F_OPAQUE) == 0u; }
    if (bf & F_GLASS) != 0u { return (fnb & F_OPAQUE) == 0u && (fnb & F_GLASS) == 0u; }
    if (bf & F_ICE) != 0u { return (fnb & F_OPAQUE) == 0u && (fnb & F_ICE) == 0u; }
    return (fnb & F_OPAQUE) == 0u;
}

// tint class -> packed tint byte (kind<<6 | slot), port of
// vc_blocks::tint::block_face_tint_packed's block match
fn tint_packed(b: u32, top: bool, biome: u32) -> u32 {
    let tc = lut[L_TC + min(b, 101u)];
    var kind = 0u; var slot = 0u;
    if tc == 1u { if top { kind = 1u; slot = biome; } }          // GRASS top
    else if tc == 2u { kind = 1u; slot = biome; }                // TALL_GRASS
    else if tc == 3u { kind = 2u; slot = biome; }                // LEAVES
    else if tc == 4u { kind = 2u; slot = 48u; }                  // BIRCH_LEAVES
    else if tc == 5u { kind = 2u; slot = 49u; }                  // SPRUCE_LEAVES
    else if tc == 6u { kind = 3u; slot = biome; }                // WATER
    return (kind << 6u) | slot;
}

// mask build — one thread per cell (t = vi*16 + ui), writing the shared
// arrays. Port of the smask/wmask build loop in mesh_sections.
fn build_mask_cell(j: u32, d: u32, dir: i32, u: u32, v: u32, ylo: u32, sl: i32, t: u32) {
    smask_lo[t] = 0u; smask_hi[t] = 0u; wmask[t] = 0u;
    let ui = t % 16u;
    let vi = t / 16u;
    let au = select(i32(ui), i32(ylo + ui), u == 1u);
    let av = select(i32(vi), i32(ylo + vi), v == 1u);
    var cell = array<i32, 3>(0, 0, 0);
    cell[d] = sl; cell[u] = au; cell[v] = av;
    let bs = job_getb(j, cell[0], cell[1], cell[2]);
    let b = sb(bs);
    let fb = fl(b);
    // AIR / cross / JSON-model states never emit greedy faces
    if (b == 0u) || ((fb & F_CROSS) != 0u) || (bs >= MODEL_BASE) { return; }
    var ncell = array<i32, 3>(cell[0], cell[1], cell[2]);
    ncell[d] = ncell[d] + dir;
    let nb = sb(job_getb(j, ncell[0], ncell[1], ncell[2]));
    let fnb = fl(nb);

    if (fb & F_WATER) != 0u {
        // water key: 1 | l<<1 | aw<<6 | bl<<7 | wt<<11 (all in lo)
        if face_visible(fb, fnb) {
            let l = job_get_sky(j, ncell[0], ncell[1], ncell[2]);
            let bl = job_get_blk(j, ncell[0], ncell[1], ncell[2]);
            let above = job_getb(j, cell[0], cell[1] + 1, cell[2]);
            let aw = select(0u, 1u, above == B_WATER);
            let wt = tint_packed(b, false, biome_at(j, cell[0], cell[2]));
            wmask[t] = 1u | (l << 1u) | (aw << 6u) | (bl << 7u) | (wt << 11u);
        }
        return;
    }
    if !face_visible(fb, fnb) { return; }

    // AO + corner sky (absolute (u, v) coords in the neighbor layer)
    let smooth_on = params[P_JOB + j * 66u + 1u];
    var ao = array<u32, 4>(3u, 3u, 3u, 3u);
    var sky = array<u32, 4>(0u, 0u, 0u, 0u);
    for (var ci = 0u; ci < 4u; ci = ci + 1u) {
        let cu = corner_u(ci); let cv = corner_v(ci);
        let big_u = au + i32(cu);
        let big_v = av + i32(cv);
        let u_out = select(big_u, big_u - 1, cu == 0u);
        let v_out = select(big_v, big_v - 1, cv == 0u);
        let hs_u = u_out;                 let hs_v = select(big_v, big_v - 1, cv == 0u);
        let vs_u = select(big_u, big_u - 1, cu == 0u); let vs_v = v_out;
        var s1 = false; var s2 = false; var cr = false;
        { // solid_at(h_side)
            var c = array<i32, 3>(ncell[0], ncell[1], ncell[2]);
            c[u] = hs_u; c[v] = hs_v;
            s1 = (fl(sb(job_getb(j, c[0], c[1], c[2]))) & F_OPAQUE) != 0u;
        }
        { // solid_at(v_side)
            var c = array<i32, 3>(ncell[0], ncell[1], ncell[2]);
            c[u] = vs_u; c[v] = vs_v;
            s2 = (fl(sb(job_getb(j, c[0], c[1], c[2]))) & F_OPAQUE) != 0u;
        }
        { // solid_at(diag)
            var c = array<i32, 3>(ncell[0], ncell[1], ncell[2]);
            c[u] = u_out; c[v] = v_out;
            cr = (fl(sb(job_getb(j, c[0], c[1], c[2]))) & F_OPAQUE) != 0u;
        }
        if smooth_on != 0u {
            if s1 && s2 { ao[ci] = 0u; }
            else { ao[ci] = 3u - (u32(s1) + u32(s2) + u32(cr)); }
        } else { ao[ci] = 3u; }
        var s = 0u;
        { var c = array<i32, 3>(ncell[0], ncell[1], ncell[2]);
          c[u] = big_u - 1; c[v] = big_v - 1; s = s + job_get_sky(j, c[0], c[1], c[2]); }
        { var c = array<i32, 3>(ncell[0], ncell[1], ncell[2]);
          c[u] = big_u;     c[v] = big_v - 1; s = s + job_get_sky(j, c[0], c[1], c[2]); }
        { var c = array<i32, 3>(ncell[0], ncell[1], ncell[2]);
          c[u] = big_u - 1; c[v] = big_v;     s = s + job_get_sky(j, c[0], c[1], c[2]); }
        { var c = array<i32, 3>(ncell[0], ncell[1], ncell[2]);
          c[u] = big_u;     c[v] = big_v;     s = s + job_get_sky(j, c[0], c[1], c[2]); }
        sky[ci] = (min(s, 60u) + 2u) / 4u;
    }
    let ao_pack = (ao[0] << 6u) | (ao[1] << 4u) | (ao[2] << 2u) | ao[3];
    let sky_pack = (sky[0] << 12u) | (sky[1] << 8u) | (sky[2] << 4u) | sky[3];
    let bl = min(job_get_blk(j, ncell[0], ncell[1], ncell[2]), 15u);
    let tint = tint_packed(b, d == 1u && dir > 0, biome_at(j, cell[0], cell[2]));
    // solid key: state<<28 | ao<<20 | sky<<4 | bl | tint<<36 (u64 split)
    smask_lo[t] = bl | (sky_pack << 4u) | (ao_pack << 20u) | ((bs & 0xFu) << 28u);
    smask_hi[t] = (bs >> 4u) | (tint << 4u);
}

// VC-16 pack — port of vc_mesh pack_vertex (same IEEE f32 rounding)
fn packv(px: f32, py: f32, pz: f32, tu: f32, tv: f32, tile: u32,
         nrm: u32, ao: u32, sky: u32, bl: u32, state: u32, tint: u32) -> vec4<u32> {
    let X = u32(clamp((px + 8.0) * 2048.0 + 0.5, 0.0, 65535.0));
    let Z = u32(clamp((pz + 8.0) * 2048.0 + 0.5, 0.0, 65535.0));
    let Y = u32(clamp(py * 128.0 + 0.5, 0.0, 65535.0));
    let U = u32(clamp(tu * 16.0 + 0.5, 0.0, 255.0));
    let V = u32(clamp(tv * 16.0 + 0.5, 0.0, 255.0));
    let flags = (nrm & 7u) | ((ao & 3u) << 3u);
    let t = tile & 0x3FFFu;
    return vec4<u32>(
        (Z << 16u) | X,
        (flags << 16u) | Y,
        (t << 18u) | (U << 10u) | (V << 2u),
        (state << 16u) | (tint << 8u) | ((sky & 0xFu) << 4u) | (bl & 0xFu));
}

fn normal_index(d: u32, dir: i32) -> u32 {
    if d == 0u { return select(1u, 0u, dir > 0); }
    if d == 1u { return select(3u, 2u, dir > 0); }
    return select(5u, 4u, dir > 0);
}

// emit one merged quad — port of greedy_merge's emit block. `gunit` is the
// GLOBAL unit id (job-major); mesh_data carries the 6-slot offset record:
// [0]=solid_v_off(words) [1]=solid_i_off [2]=water_v_off [3]=water_i_off
// [4]=solid section-local vertex base [5]=water section-local vertex base
fn emit_quad(state: u32, ao_pack: u32, sky_pack: u32, bl: u32, tint: u32,
             is_solid: bool, water_aw: u32, d: u32, dir: i32, sl: i32, ylo: u32,
             off_u: u32, off_v: u32, ui: u32, vi: u32, w: u32, h: u32,
             gunit: u32, qi: u32) {
    let u = (d + 1u) % 3u;
    let v = (d + 2u) % 3u;
    var ao = array<u32, 4>((ao_pack >> 6u) & 3u, (ao_pack >> 4u) & 3u,
                           (ao_pack >> 2u) & 3u, ao_pack & 3u);
    var sky = array<u32, 4>((sky_pack >> 12u) & 0xFu, (sky_pack >> 8u) & 0xFu,
                            (sky_pack >> 4u) & 0xFu, sky_pack & 0xFu);
    // face plane coordinate along d (local slice coord; absolute for Y)
    let pd = select(f32(sl), f32(sl) + 1.0, dir > 0);
    // texture orientation per face (v flipped on sides)
    var t00 = vec2<f32>(0.0, 0.0); var t10 = vec2<f32>(0.0, 0.0);
    var t11 = vec2<f32>(0.0, 0.0); var t01 = vec2<f32>(0.0, 0.0);
    let wf = f32(w); let hf = f32(h);
    if d == 0u {
        t00 = vec2<f32>(0.0, wf); t10 = vec2<f32>(0.0, 0.0);
        t11 = vec2<f32>(hf, 0.0); t01 = vec2<f32>(hf, wf);
    } else if d == 1u {
        t00 = vec2<f32>(0.0, 0.0); t10 = vec2<f32>(wf, 0.0);
        t11 = vec2<f32>(wf, hf); t01 = vec2<f32>(0.0, hf);
    } else {
        t00 = vec2<f32>(0.0, hf); t10 = vec2<f32>(wf, hf);
        t11 = vec2<f32>(wf, 0.0); t01 = vec2<f32>(0.0, 0.0);
    }
    let water_top_open = !is_solid && (water_aw == 0u);
    // per-STATE tiles (log axis rotation: rings on the ±axis faces)
    let tile_i = select(
        select(tile_of(state, 3u), tile_of(state, 2u), d == 0u),
        select(tile_of(state, 1u), tile_of(state, 0u), dir > 0),
        d == 1u);
    let nrm = normal_index(d, dir);

    // stream offsets (u32 words) + section-local vertex base
    let voff = mesh_data[gunit * 6u + select(2u, 0u, is_solid)];
    let ioff = mesh_data[gunit * 6u + select(3u, 1u, is_solid)];
    let base0 = mesh_data[gunit * 6u + select(5u, 4u, is_solid)];

    // corners: (coord, texel, ao, sky) — CPU order c00, c10, c11, c01
    var cx = array<f32, 4>(f32(ui), f32(ui + w), f32(ui + w), f32(ui));
    var cy = array<f32, 4>(f32(vi), f32(vi), f32(vi + h), f32(vi + h));
    var tu = array<f32, 4>(t00.x, t10.x, t11.x, t01.x);
    var tv = array<f32, 4>(t00.y, t10.y, t11.y, t01.y);

    let vb = voff + qi * 16u;             // word offset of this quad's verts
    for (var ci = 0u; ci < 4u; ci = ci + 1u) {
        // local position: p[d] = pd; p[u] = c.u + off_u; p[v] = c.v + off_v
        var p = array<f32, 3>(0.0, 0.0, 0.0);
        p[d] = pd;
        p[u] = cx[ci] + f32(off_u);
        p[v] = cy[ci] + f32(off_v);
        if !is_solid {
            if (d == 1u) && (dir > 0) {
                p[1] = p[1] - 0.125;                       // water surface 14/16
            } else if (d == 0u) && (cx[ci] == f32(ui + w)) && water_top_open {
                p[1] = p[1] - 0.125;                       // side-face top edge
            } else if (d == 2u) && (cy[ci] == f32(vi + h)) && water_top_open {
                p[1] = p[1] - 0.125;
            }
        }
        let packed = packv(p[0], p[1], p[2], tu[ci], tv[ci], tile_i,
                           nrm, ao[ci], sky[ci], bl, state, tint);
        verts_out[vb + ci * 4u + 0u] = packed.x;
        verts_out[vb + ci * 4u + 1u] = packed.y;
        verts_out[vb + ci * 4u + 2u] = packed.z;
        verts_out[vb + ci * 4u + 3u] = packed.w;
    }
    // diagonal choice by AO anisotropy; winding flipped for negative faces
    let flip = dir < 0;
    let use_b = (ao[0] + ao[2]) < (ao[1] + ao[3]);
    var tri = array<u32, 6>(0u, 1u, 2u, 0u, 2u, 3u);
    if !flip && use_b { tri = array<u32, 6>(1u, 2u, 3u, 1u, 3u, 0u); }
    if flip && !use_b { tri = array<u32, 6>(2u, 1u, 0u, 3u, 2u, 0u); }
    if flip && use_b { tri = array<u32, 6>(3u, 2u, 1u, 0u, 3u, 1u); }
    let base_v = base0 + qi * 4u;          // section-local vertex base
    for (var k = 0u; k < 6u; k = k + 1u) {
        idx_out[ioff + qi * 6u + k] = base_v + tri[k];
    }
}

// greedy rect scan over the (lo, hi) solid mask — the CPU greedy_merge
// loop order: vi rows, ui cols, width run, height growth, rect zero-out.
// Counts quads; emits them when `emit` (at precomputed offsets).
fn scan_solid(emit: bool, d: u32, dir: i32, sl: i32, ylo: u32,
              off_u: u32, off_v: u32, gunit: u32) -> u32 {
    var quads = 0u;
    var vi = 0u;
    loop {
        if vi >= 16u { break; }
        var ui = 0u;
        loop {
            if ui >= 16u { break; }
            let klo = smask_lo[vi * 16u + ui];
            let khi = smask_hi[vi * 16u + ui];
            if (klo != 0u) || (khi != 0u) {
                var w = 1u;
                loop {
                    if (ui + w) >= 16u { break; }
                    if (smask_lo[vi * 16u + ui + w] != klo)
                        || (smask_hi[vi * 16u + ui + w] != khi) { break; }
                    w = w + 1u;
                }
                var h = 1u;
                loop {
                    if (vi + h) >= 16u { break; }
                    var grew = true;
                    for (var k = 0u; k < w; k = k + 1u) {
                        if (smask_lo[(vi + h) * 16u + ui + k] != klo)
                            || (smask_hi[(vi + h) * 16u + ui + k] != khi) { grew = false; }
                    }
                    if !grew { break; }
                    h = h + 1u;
                }
                if emit {
                    let state = ((khi & 0xFu) << 4u) | (klo >> 28u);
                    let ao_pack = (klo >> 20u) & 0xFFu;
                    let sky_pack = (klo >> 4u) & 0xFFFFu;
                    let bl = klo & 0xFu;
                    let tint = (khi >> 4u) & 0xFFu;
                    emit_quad(state, ao_pack, sky_pack, bl, tint, true, 0u,
                              d, dir, sl, ylo, off_u, off_v, ui, vi, w, h,
                              gunit, quads);
                }
                quads = quads + 1u;
                for (var hh = 0u; hh < h; hh = hh + 1u) {
                    for (var ww = 0u; ww < w; ww = ww + 1u) {
                        smask_lo[(vi + hh) * 16u + ui + ww] = 0u;
                        smask_hi[(vi + hh) * 16u + ui + ww] = 0u;
                    }
                }
                ui = ui + w;
            } else { ui = ui + 1u; }
        }
        vi = vi + 1u;
    }
    return quads;
}

// greedy rect scan over the single-word water mask (water keys never
// exceed 32 bits)
fn scan_water(emit: bool, d: u32, dir: i32, sl: i32, ylo: u32,
              off_u: u32, off_v: u32, gunit: u32) -> u32 {
    var quads = 0u;
    var vi = 0u;
    loop {
        if vi >= 16u { break; }
        var ui = 0u;
        loop {
            if ui >= 16u { break; }
            let klo = wmask[vi * 16u + ui];
            if klo != 0u {
                var w = 1u;
                loop {
                    if (ui + w) >= 16u { break; }
                    if wmask[vi * 16u + ui + w] != klo { break; }
                    w = w + 1u;
                }
                var h = 1u;
                loop {
                    if (vi + h) >= 16u { break; }
                    var grew = true;
                    for (var k = 0u; k < w; k = k + 1u) {
                        if wmask[(vi + h) * 16u + ui + k] != klo { grew = false; }
                    }
                    if !grew { break; }
                    h = h + 1u;
                }
                if emit {
                    let l = (klo >> 1u) & 0xFu;
                    let aw = (klo >> 6u) & 1u;
                    let bl = (klo >> 7u) & 0xFu;
                    let wt = (klo >> 11u) & 0xFFu;
                    let sky_pack = (l << 12u) | (l << 8u) | (l << 4u) | l;
                    emit_quad(B_WATER, 0xFFu, sky_pack, bl, wt, false, aw,
                              d, dir, sl, ylo, off_u, off_v, ui, vi, w, h,
                              gunit, quads);
                }
                quads = quads + 1u;
                for (var hh = 0u; hh < h; hh = hh + 1u) {
                    for (var ww = 0u; ww < w; ww = ww + 1u) {
                        wmask[(vi + hh) * 16u + ui + ww] = 0u;
                    }
                }
                ui = ui + w;
            } else { ui = ui + 1u; }
        }
        vi = vi + 1u;
    }
    return quads;
}

// unit decode: (d, dir, sec, sl) from a linear unit index
// unit = ((d*2 + dir_i)*16 + sec)*16 + sl
fn unit_axes(unit: u32) -> vec4<u32> {
    return vec4<u32>(unit / 512u, (unit / 256u) % 2u, (unit / 16u) % 16u, unit % 16u);
}

@compute @workgroup_size(256)
fn main_count(@builtin(workgroup_id) wid: vec3<u32>,
              @builtin(local_invocation_id) lid: vec3<u32>) {
    let gi = wid.x;                       // global unit across all jobs
    let j = gi / 1536u;
    let unit = gi % 1536u;
    let ax = unit_axes(unit);
    let d = ax.x; let dir_i = ax.y; let sec = ax.z; let sl_l = ax.w;
    let t = lid.x;
    let dir = select(-1, 1, dir_i == 0u);
    let ylo = sec * 16u;
    let mask = params[P_JOB + j * 66u];
    let u = (d + 1u) % 3u;
    let v = (d + 2u) % 3u;
    let sl = select(i32(sl_l), i32(ylo + sl_l), d == 1u);
    if ((mask >> sec) & 1u) != 0u {
        build_mask_cell(j, d, dir, u, v, ylo, sl, t);
    } else {
        smask_lo[t] = 0u; smask_hi[t] = 0u; wmask[t] = 0u;
    }
    workgroupBarrier();
    if t == 0u {
        let sq = scan_solid(false, d, dir, sl, ylo, 0u, 0u, gi);
        let wq = scan_water(false, d, dir, sl, ylo, 0u, 0u, gi);
        mesh_data[gi * 2u] = sq;
        mesh_data[gi * 2u + 1u] = wq;
    }
}

@compute @workgroup_size(256)
fn main_emit(@builtin(workgroup_id) wid: vec3<u32>,
             @builtin(local_invocation_id) lid: vec3<u32>) {
    let gi = wid.x;
    let j = gi / 1536u;
    let unit = gi % 1536u;
    let ax = unit_axes(unit);
    let d = ax.x; let dir_i = ax.y; let sec = ax.z; let sl_l = ax.w;
    let t = lid.x;
    let dir = select(-1, 1, dir_i == 0u);
    let ylo = sec * 16u;
    let mask = params[P_JOB + j * 66u];
    let u = (d + 1u) % 3u;
    let v = (d + 2u) % 3u;
    let sl = select(i32(sl_l), i32(ylo + sl_l), d == 1u);
    let off_u = select(0u, ylo, u == 1u);
    let off_v = select(0u, ylo, v == 1u);
    if ((mask >> sec) & 1u) != 0u {
        build_mask_cell(j, d, dir, u, v, ylo, sl, t);
    } else {
        smask_lo[t] = 0u; smask_hi[t] = 0u; wmask[t] = 0u;
    }
    workgroupBarrier();
    if t == 0u {
        // the quad counts were already produced by pass A (the CPU owns
        // them); the returns are unused here — WGSL has no discard form
        let sq = scan_solid(true, d, dir, sl, ylo, off_u, off_v, gi);
        let wq = scan_water(true, d, dir, sl, ylo, off_u, off_v, gi);
        let _unused = sq + wq;
    }
}
"#;

// ---------------------------------------------------------------------------
// Rust side — LUT construction, batch state machine, deterministic offsets,
// section assembly.
// ---------------------------------------------------------------------------

/// LUT words: [0..235) state→block · [235..337) block flags ·
/// [337..439) block tint class · [439..1379) state tiles (4/state)
const LUT_WORDS: usize = 1379;

fn build_lut() -> Vec<u32> {
    use vc_blocks::blocks::{is_cross, is_opaque, state_block, state_tiles};
    let mut lut = vec![0u32; LUT_WORDS];
    for s in 0..STATE_COUNT {
        lut[s] = state_block(s as u16) as u32;
    }
    for b in 0..BLOCK_COUNT {
        let id = b as u8;
        let mut flags = 0u32;
        if is_opaque(id) {
            flags |= 1; // F_OPAQUE
        }
        if id == WATER {
            flags |= 2; // F_WATER
        }
        if id == LEAVES || id == BIRCH_LEAVES || id == SPRUCE_LEAVES {
            flags |= 4; // F_LEAVES
        }
        if id == GLASS {
            flags |= 8; // F_GLASS
        }
        if id == ICE {
            flags |= 16; // F_ICE
        }
        if is_cross(id) {
            flags |= 32; // F_CROSS
        }
        lut[235 + b] = flags;
        let tint_class = match id {
            GRASS => 1,
            TALL_GRASS => 2,
            LEAVES => 3,
            BIRCH_LEAVES => 4,
            SPRUCE_LEAVES => 5,
            WATER => 6,
            _ => 0,
        };
        lut[337 + b] = tint_class;
    }
    for s in 0..STATE_COUNT {
        let tiles = state_tiles(s as u16);
        for i in 0..4 {
            lut[439 + s * 4 + i] = tiles[i] as u32;
        }
    }
    lut
}

/// metadata carried alongside a GPU-meshed chunk (the assembly side needs
/// the section cache for unmasked sections and the center chunk for the
/// occlusion-graph bits, which the game computes from the results)
pub struct GpuMeshJobMeta {
    pub pos: ChunkPos,
    /// sections to rebuild (§12 bitset; 0xFFFF = full chunk)
    pub mask: u16,
    pub smooth: bool,
    /// cached section meshes to reuse for unmasked sections
    pub prev: Vec<Option<Arc<MeshData>>>,
    /// center chunk of the 3×3 snapshot (Phase 6 §26 occl computation)
    pub center: Option<Arc<vc_chunk::chunk::Chunk>>,
}

/// one completed GPU mesh job (feeds the game's `JobResult::Mesh` path)
pub struct GpuMeshDone {
    pub pos: ChunkPos,
    pub mask: u16,
    pub sections: Vec<Option<Arc<MeshData>>>,
    pub mesh: MeshData,
    pub center: Option<Arc<vc_chunk::chunk::Chunk>>,
}

/// batch size cap — matches the game's native per-frame mesh-job cap (16),
/// and keeps n_jobs*1536 workgroups well under the 65 535 dispatch limit
const MAX_BATCH: usize = 16;

enum BatchStage {
    /// dispatch A submitted; waiting for the counts readback map
    Counts {
        /// kept alive for the batch lifetime (conservative resource
        /// lifetime — wgpu 22 submissions ref-count, but holding is cheap)
        _bg: wgpu::BindGroup,
        rx: Receiver<()>,
    },
    /// dispatch B submitted; waiting for the combined output readback map
    Outputs {
        _bg: wgpu::BindGroup,
        rx: Receiver<()>,
        offsets: Vec<u32>,
        /// combined staging (verts words || idx words) — stays mapped until
        /// `finish_batch` reads and unmaps it
        out_stage: wgpu::Buffer,
        /// word count of the verts half (the staging split point — explicit
        /// because the zero-quad `max(1)` buffer floors break size()/2)
        v_words_len: usize,
    },
}

struct Batch {
    metas: Vec<GpuMeshJobMeta>,
    counts: Vec<u32>, // n*UNITS*2 — filled at the Counts→Outputs transition
    stage: BatchStage,
    // buffers kept alive for the batch lifetime
    _params: wgpu::Buffer,
    _blocks: wgpu::Buffer,
    _sky: wgpu::Buffer,
    _blk: wgpu::Buffer,
    _mesh_data: wgpu::Buffer,
    _counts_stage: wgpu::Buffer,
}

/// Phase 7 GPU compute mesher. One instance per renderer; `enqueue` jobs,
/// then drive `advance()` every frame (native: it polls the device; web:
/// wgpu auto-polls and callbacks fire from the event loop).
pub struct GpuMesher {
    pipeline_count: wgpu::ComputePipeline,
    pipeline_emit: wgpu::ComputePipeline,
    bgl: wgpu::BindGroupLayout,
    lut_buf: wgpu::Buffer,
    dummy_v: wgpu::Buffer,
    dummy_i: wgpu::Buffer,
    batch: Option<Batch>,
    queue_jobs: VecDeque<(GpuMeshJobMeta, MeshInputs)>,
    /// stats for F3/E2E: jobs completed since boot
    pub jobs_done: u64,
}

/// deterministic offset table from the pass-A quad counts — the exact CPU
/// emission order (job-major, then d → dir[+,-] → sec → sl; solid stream
/// then water stream). Slot layout per global unit g:
/// [0] solid_v_off(words) [1] solid_i_off [2] water_v_off [3] water_i_off
/// [4] solid section-local vertex base [5] water section-local base.
/// Water offsets carry the solid-total stream shift (the combined output
/// buffers concatenate [solid][water] per kind).
/// Returns (offsets, solid_v_words, solid_i_words, water_v_words, water_i_words).
fn compute_offsets(counts: &[u32], n: usize) -> (Vec<u32>, usize, usize, usize, usize) {
    let mut offsets = vec![0u32; n * UNITS * 6];
    let (mut svr, mut sir, mut wvr, mut wir) = (0usize, 0usize, 0usize, 0usize);
    for g in 0..n * UNITS {
        let sq = counts[g * 2] as usize;
        let wq = counts[g * 2 + 1] as usize;
        offsets[g * 6] = svr as u32;
        offsets[g * 6 + 1] = sir as u32;
        offsets[g * 6 + 2] = wvr as u32;
        offsets[g * 6 + 3] = wir as u32;
        svr += sq * 16; // 4 verts × 4 words
        sir += sq * 6;
        wvr += wq * 16;
        wir += wq * 6;
    }
    // section-local vertex bases: base = distance of the unit's stream
    // offset from the section's stream START (the section's first unit in
    // iteration order d=0, dir=0, sl=0 — offsets grow monotonically with
    // the iteration order, so that unit is the section minimum)
    for j in 0..n {
        for sec in 0..16usize {
            let first = j * UNITS + sec * 16; // d=0, dir=0, sl=0
            let s_start = offsets[first * 6] as usize;
            let w_start = offsets[first * 6 + 2] as usize;
            for d in 0..3usize {
                for dir in 0..2usize {
                    for sl in 0..16usize {
                        let g = j * UNITS + d * 512 + dir * 256 + sec * 16 + sl;
                        offsets[g * 6 + 4] = ((offsets[g * 6] as usize - s_start) / 4) as u32;
                        offsets[g * 6 + 5] = ((offsets[g * 6 + 2] as usize - w_start) / 4) as u32;
                    }
                }
            }
        }
    }
    // combined-buffer stream shift (the section-local bases are invariant:
    // the shift lands on the unit offsets AND their section starts)
    for g in 0..n * UNITS {
        offsets[g * 6 + 2] += svr as u32;
        offsets[g * 6 + 3] += sir as u32;
    }
    (offsets, svr, sir, wvr, wir)
}

impl GpuMesher {
    /// Create the mesher. Basic compute is core wgpu on Vulkan/DX12/Metal/
    /// WebGPU — the renderer only constructs this when the device reports
    /// compute support (the WebGL2 fallback lacks it and never gets here).
    pub fn new(device: &wgpu::Device, _queue: &wgpu::Queue) -> Self {
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("gpu-mesh"),
            source: wgpu::ShaderSource::Wgsl(MESH_COMPUTE_SHADER.into()),
        });
        // ONE layout, 8 storage bindings (the WebGPU per-stage floor):
        // 5 = mesh_data (counts pass A / offsets pass B), 6/7 = outputs
        // (pass A binds 16-byte dummies — the emit branch is runtime-false
        // there, but the static call graph reaches them)
        let storage_ro = |binding: u32| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        };
        let storage_rw = |binding: u32| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only: false },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        };
        let bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("gpu-mesh-bgl"),
            entries: &[
                storage_ro(0),
                storage_ro(1),
                storage_ro(2),
                storage_ro(3),
                storage_ro(4),
                storage_rw(5),
                storage_rw(6),
                storage_rw(7),
            ],
        });
        let pll = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("gpu-mesh-pll"),
            bind_group_layouts: &[&bgl],
            push_constant_ranges: &[],
        });
        let make_pipe = |entry: &str| {
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("gpu-mesh-pipe"),
                layout: Some(&pll),
                module: &module,
                entry_point: entry,
                compilation_options: Default::default(),
                cache: None,
            })
        };
        let pipeline_count = make_pipe("main_count");
        let pipeline_emit = make_pipe("main_emit");

        let lut = build_lut();
        let lut_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("gpu-mesh-lut"),
            contents: bytemuck::cast_slice(&lut),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });
        // pass-A dummies for bindings 6/7 (never written: emit=false)
        let mk_dummy = |dev: &wgpu::Device| {
            dev.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("gpu-mesh-dummy"),
                contents: &[0u8; 16],
                usage: wgpu::BufferUsages::STORAGE,
            })
        };
        let dummy_v = mk_dummy(&device);
        let dummy_i = mk_dummy(&device);
        GpuMesher {
            pipeline_count,
            pipeline_emit,
            bgl,
            lut_buf,
            dummy_v,
            dummy_i,
            batch: None,
            queue_jobs: VecDeque::new(),
            jobs_done: 0,
        }
    }

    /// true while a batch is being processed (jobs pile up in the queue)
    pub fn busy(&self) -> bool {
        self.batch.is_some()
    }

    pub fn queued(&self) -> usize {
        self.queue_jobs.len()
    }

    /// enqueue one mesh job (inputs already built by the shared
    /// `build_mesh_inputs`). Actual GPU submission happens when the
    /// current batch drains (one batch in flight keeps the two-readback
    /// state machine simple and the volumes buffers single-purpose).
    pub fn enqueue(&mut self, meta: GpuMeshJobMeta, inputs: MeshInputs) {
        self.queue_jobs.push_back((meta, inputs));
    }


    /// drive the state machine; call every frame.
    /// Returns (completed jobs, lost jobs) — a job is LOST only if a GPU
    /// buffer map fails (Disconnected channel, e.g. a device error): the
    /// game clears its inflight marker for those and the §12 dirty bits
    /// stay set, so the CPU path remeshes them on the next stream pass.
    pub fn advance(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> (Vec<GpuMeshDone>, Vec<(ChunkPos, u16)>) {
        // 1. start a batch from queued jobs when idle
        if self.batch.is_none() && !self.queue_jobs.is_empty() {
            let take = self.queue_jobs.len().min(MAX_BATCH);
            let jobs: Vec<(GpuMeshJobMeta, MeshInputs)> =
                self.queue_jobs.drain(..take).collect();
            self.start_batch(device, queue, jobs);
        }
        // 2. progress the active batch one step
        let mut done = Vec::new();
        let mut lost = Vec::new();
        let mut consumed = false;
        if let Some(mut batch) = self.batch.take() {
            match &batch.stage {
                BatchStage::Counts { rx, .. } => match rx.try_recv() {
                    Ok(()) => {
                        // counts mapped: read, compute offsets, dispatch B
                        // (the batch stays alive in its Outputs stage)
                        self.to_emit_stage(device, queue, &mut batch);
                    }
                    Err(TryRecvError::Empty) => {
                        #[cfg(not(target_arch = "wasm32"))]
                        device.poll(wgpu::Maintain::Poll);
                    }
                    Err(TryRecvError::Disconnected) => {
                        // counts map failed — the batch is unrecoverable
                        lost = batch.metas.iter().map(|m| (m.pos, m.mask)).collect();
                    }
                },
                BatchStage::Outputs { rx, .. } => match rx.try_recv() {
                    Ok(()) => {
                        // the single combined staging map is ready
                        done = self.finish_batch(&mut batch);
                        self.jobs_done += done.len() as u64;
                        consumed = true;
                    }
                    Err(TryRecvError::Empty) => {
                        #[cfg(not(target_arch = "wasm32"))]
                        device.poll(wgpu::Maintain::Poll);
                    }
                    Err(TryRecvError::Disconnected) => {
                        // outputs map failed — same recovery contract
                        lost = batch.metas.iter().map(|m| (m.pos, m.mask)).collect();
                    }
                },
            }
            if !consumed && lost.is_empty() {
                self.batch = Some(batch);
            }
            // (consumed OR lost) → the batch is dropped either way
        }
        (done, lost)
    }

    /// native/test helper: block until the current batch completes
    /// (drives the device with `Maintain::Wait`). Returns completed jobs;
    /// panics if a readback fails (test contract: no silent degradation).
    pub fn wait_done(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Vec<GpuMeshDone> {
        let mut all = Vec::new();
        while self.batch.is_some() || !self.queue_jobs.is_empty() {
            #[cfg(not(target_arch = "wasm32"))]
            device.poll(wgpu::Maintain::Wait);
            let (mut done, lost) = self.advance(device, queue);
            all.append(&mut done);
            if !lost.is_empty() {
                panic!("gpu mesh readback failed for {} jobs", lost.len());
            }
        }
        all
    }

    // ------------------------------------------------------------- batch --

    fn start_batch(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, jobs: Vec<(GpuMeshJobMeta, MeshInputs)>) {
        let n = jobs.len();
        let vol = PAD * PAD * 256; // 589,824 bytes per job volume

        // one combined params buffer: [n_jobs, 0, per job: mask, smooth,
        // 64 packed biome u32] — 2 + n*66 words
        let mut params = vec![0u32; 2 + n * PARAM_STRIDE];
        params[0] = n as u32;
        let mut blocks = vec![0u8; n * vol];
        let mut sky = vec![0u8; n * vol];
        let mut blk = vec![0u8; n * vol];
        let mut metas = Vec::with_capacity(n);
        for (j, (meta, inputs)) in jobs.into_iter().enumerate() {
            let base = 2 + j * PARAM_STRIDE;
            params[base] = meta.mask as u32;
            params[base + 1] = meta.smooth as u32;
            for (c, b) in inputs.biomes.iter().enumerate() {
                params[base + 2 + (c >> 2)] |= (*b as u32) << ((c & 3) * 8);
            }
            blocks[j * vol..(j + 1) * vol].copy_from_slice(&inputs.blocks);
            sky[j * vol..(j + 1) * vol].copy_from_slice(&inputs.light);
            blk[j * vol..(j + 1) * vol].copy_from_slice(&inputs.blight);
            metas.push(meta);
        }

        let mk_in = |label: &str, data: &[u8]| {
            device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some(label),
                    contents: data,
                    usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                })
        };
        let params_buf = mk_in("gpu-mesh-params", bytemuck::cast_slice(&params));
        let blocks_buf = mk_in("gpu-mesh-blocks", &blocks);
        let sky_buf = mk_in("gpu-mesh-sky", &sky);
        let blk_buf = mk_in("gpu-mesh-blk", &blk);
        // binding 5: counts (pass A) then offsets (pass B — rewritten in
        // place between the dispatches). Size = the larger offsets layout.
        let mesh_data = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("gpu-mesh-data"),
            size: (n * UNITS * 6 * 4) as u64,
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        let counts_words = n * UNITS * 2;
        let counts_stage = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("gpu-mesh-counts-stage"),
            size: (counts_words * 4) as u64,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // dispatch A: count pass
        let bg = self.make_bind_group(
            device,
            &params_buf,
            &blocks_buf,
            &sky_buf,
            &blk_buf,
            &mesh_data,
            None,
            None,
        );
        let mut enc = device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("gpu-mesh-count"),
            });
        let mut pass = enc.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("gpu-mesh-count"),
            timestamp_writes: None,
        });
        pass.set_pipeline(&self.pipeline_count);
        pass.set_bind_group(0, &bg, &[]);
        pass.dispatch_workgroups((n * UNITS) as u32, 1, 1);
        drop(pass);
        enc.copy_buffer_to_buffer(
            &mesh_data,
            0,
            &counts_stage,
            0,
            (counts_words * 4) as u64,
        );
        let submit = queue.submit([enc.finish()]);
        let (tx, rx) = std::sync::mpsc::channel::<()>();
        counts_stage
            .slice(..)
            .map_async(wgpu::MapMode::Read, move |res| {
                if res.is_ok() {
                    let _ = tx.send(());
                }
                // NOTE: no unmap here — the reader unmaps after reading
            });
        let _ = submit;

        self.batch = Some(Batch {
            metas,
            counts: vec![0u32; counts_words],
            stage: BatchStage::Counts { _bg: bg, rx },
            _params: params_buf,
            _blocks: blocks_buf,
            _sky: sky_buf,
            _blk: blk_buf,
            _mesh_data: mesh_data,
            _counts_stage: counts_stage,
        });
    }

    fn make_bind_group(
        &self,
        device: &wgpu::Device,
        params: &wgpu::Buffer,
        blocks: &wgpu::Buffer,
        sky: &wgpu::Buffer,
        blk: &wgpu::Buffer,
        mesh_data: &wgpu::Buffer,
        verts: Option<&wgpu::Buffer>,
        idx: Option<&wgpu::Buffer>,
    ) -> wgpu::BindGroup {
        let entries = &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: self.lut_buf.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: blocks.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: sky.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: blk.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 4,
                resource: params.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 5,
                resource: mesh_data.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 6,
                resource: verts.unwrap_or(&self.dummy_v).as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 7,
                resource: idx.unwrap_or(&self.dummy_i).as_entire_binding(),
            },
        ];
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("gpu-mesh-bg"),
            layout: &self.bgl,
            entries,
        })
    }

    /// counts readback complete → read counts, compute the deterministic
    /// offset table, allocate exact output buffers, dispatch the emit pass
    fn to_emit_stage(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, batch: &mut Batch) {
        let n = batch.metas.len();
        // read the mapped counts
        {
            let Batch {
                _counts_stage, counts, ..
            } = batch;
            let data = _counts_stage.slice(..).get_mapped_range();
            let words: Vec<u32> = data
                .chunks_exact(4)
                .map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
                .collect();
            drop(data);
            _counts_stage.unmap();
            counts.copy_from_slice(&words);
        }
        let counts = batch.counts.clone();
        let (offsets, svr, sir, wvr, wir) = compute_offsets(&counts, n);
        // exact output stream sizes (words); 4-word floor keeps zero-size
        // buffers out of wgpu (units with 0 quads never write)
        let verts_bytes = (svr.max(1) * 4) as u64;
        let idx_bytes = (sir.max(1) * 4) as u64;
        let wverts_bytes = (wvr.max(1) * 4) as u64;
        let widx_bytes = (wir.max(1) * 4) as u64;

        // solid + water streams in TWO buffers: verts (solid||water) and
        // idx (solid||water) — bindings 6/7
        let verts_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("gpu-mesh-verts"),
            size: verts_bytes + wverts_bytes,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        let idx_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("gpu-mesh-idx"),
            size: idx_bytes + widx_bytes,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        // rewrite binding 5 with the offsets (dual-purpose buffer)
        queue.write_buffer(
            &batch._mesh_data,
            0,
            bytemuck::cast_slice(&offsets),
        );

        let bg = self.make_bind_group(
            device,
            &batch._params,
            &batch._blocks,
            &batch._sky,
            &batch._blk,
            &batch._mesh_data,
            Some(&verts_buf),
            Some(&idx_buf),
        );
        let v_stage = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("gpu-mesh-outstage"),
            size: verts_bytes + wverts_bytes + idx_bytes + widx_bytes,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let mut enc = device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("gpu-mesh-emit"),
            });
        let mut pass = enc.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("gpu-mesh-emit"),
            timestamp_writes: None,
        });
        pass.set_pipeline(&self.pipeline_emit);
        pass.set_bind_group(0, &bg, &[]);
        pass.dispatch_workgroups((n * UNITS) as u32, 1, 1);
        drop(pass);
        enc.copy_buffer_to_buffer(&verts_buf, 0, &v_stage, 0, verts_bytes + wverts_bytes);
        enc.copy_buffer_to_buffer(
            &idx_buf,
            0,
            &v_stage,
            verts_bytes + wverts_bytes,
            idx_bytes + widx_bytes,
        );
        queue.submit([enc.finish()]);

        // ONE staging map → one callback → one completion signal (no
        // ordering hazard between two independent maps)
        let (tx, rx) = std::sync::mpsc::channel::<()>();
        v_stage.slice(..).map_async(wgpu::MapMode::Read, move |res| {
            if res.is_ok() {
                let _ = tx.send(());
            }
            // no unmap — finish_batch reads then unmaps
        });

        batch.stage = BatchStage::Outputs {
            _bg: bg,
            rx,
            offsets,
            out_stage: v_stage,
            v_words_len: (verts_bytes + wverts_bytes) as usize / 4,
        };
    }

    /// both output maps complete → assemble per-section MeshData and the
    /// merged per-chunk mesh (identical structure to mesh_sections' merge)
    fn finish_batch(&mut self, batch: &mut Batch) -> Vec<GpuMeshDone> {
        let n = batch.metas.len();
        let Batch {
            metas,
            counts,
            stage,
            ..
        } = batch;
        let (offsets, out_stage, v_words_len) = match stage {
            BatchStage::Outputs {
                offsets,
                out_stage,
                v_words_len,
                ..
            } => (offsets, out_stage, *v_words_len),
            _ => return Vec::new(),
        };
        let data = out_stage.slice(..).get_mapped_range();
        let words: Vec<u32> = data
            .chunks_exact(4)
            .map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
            .collect();
        drop(data);
        out_stage.unmap();
        let v_words = &words[..v_words_len];
        let i_words = &words[v_words_len..];

        // (stream totals already folded into the water offsets at
        // to_emit_stage — the assembly slices by the raw offset values)

        let mut out = Vec::with_capacity(n);
        for (j, meta) in metas.iter().enumerate() {
            let mut sections: Vec<Option<Arc<MeshData>>> = Vec::with_capacity(16);
            let mut merged = MeshData {
                solid: (Vec::new(), Vec::new()),
                water: (Vec::new(), Vec::new()),
            };
            for sec in 0..16usize {
                if meta.mask & (1 << sec) == 0 {
                    // unmasked: reuse the cached section mesh (§12)
                    let cached = meta.prev.get(sec).and_then(|p| p.clone());
                    if let Some(p) = &cached {
                        merge_mesh_into(&mut merged, p);
                    }
                    sections.push(cached);
                    continue;
                }
                let mut sv: Vec<Vertex> = Vec::new();
                let mut si: Vec<u32> = Vec::new();
                let mut wv: Vec<Vertex> = Vec::new();
                let mut wi: Vec<u32> = Vec::new();
                // units of this section in CPU iteration order (d, dir, sl)
                for d in 0..3usize {
                    for dir in 0..2usize {
                        for sl in 0..16usize {
                            let g = j * UNITS + d * 512 + dir * 256 + sec * 16 + sl;
                            let sq = counts[g * 2] as usize;
                            let wq = counts[g * 2 + 1] as usize;
                            let voff = offsets[g * 6] as usize;
                            let ioff = offsets[g * 6 + 1] as usize;
                            let wvoff = offsets[g * 6 + 2] as usize;
                            let wioff = offsets[g * 6 + 3] as usize;
                            for k in 0..sq {
                                let base = voff + k * 16;
                                sv.push(Vertex {
                                    w0: v_words[base],
                                    w1: v_words[base + 1],
                                    w2: v_words[base + 2],
                                    w3: v_words[base + 3],
                                });
                            }
                            si.extend_from_slice(&i_words[ioff..ioff + sq * 6]);
                            for k in 0..wq {
                                // water offsets carry the solid-total shift
                                let base = wvoff + k * 16;
                                wv.push(Vertex {
                                    w0: v_words[base],
                                    w1: v_words[base + 1],
                                    w2: v_words[base + 2],
                                    w3: v_words[base + 3],
                                });
                            }
                            wi.extend_from_slice(&i_words[wioff..wioff + wq * 6]);
                        }
                    }
                }
                let md = Arc::new(MeshData {
                    solid: (sv, si),
                    water: (wv, wi),
                });
                merge_mesh_into(&mut merged, &md);
                sections.push(Some(md));
            }
            out.push(GpuMeshDone {
                pos: meta.pos,
                mask: meta.mask,
                sections,
                mesh: merged,
                center: meta.center.clone(),
            });
        }
        out
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// The compute WGSL must parse AND type-check with the exact naga
    /// version wgpu 22 ships (dev-dependency) — catches shader errors in
    /// `cargo test` instead of at device-creation time (same pattern as
    /// render.rs `shader_tests::wgsl_shaders_validate`).
    #[test]
    fn mesh_compute_wgsl_validates() {
        let mut frontend = naga::front::wgsl::Frontend::new();
        let module = frontend
            .parse(MESH_COMPUTE_SHADER)
            .unwrap_or_else(|e| panic!("gpu-mesh WGSL parse failed: {e}"));
        let mut validator = naga::valid::Validator::new(
            naga::valid::ValidationFlags::all(),
            naga::valid::Capabilities::all(),
        );
        validator
            .validate(&module)
            .unwrap_or_else(|e| panic!("gpu-mesh WGSL validation failed: {e:?}"));
    }

    /// the LUT must mirror the vc-blocks property functions exactly —
    /// every flag/class/tile entry is generated FROM those functions, and
    /// this test guards against drift (e.g. a new block changing the
    /// face_visible arms without the flags LUT following)
    #[test]
    fn lut_mirrors_vc_blocks() {
        use vc_blocks::blocks::{is_cross, is_opaque, state_block, state_tiles, AIR};
        use vc_blocks::tint::{KIND_FOLIAGE, KIND_GRASS, KIND_WATER, SLOT_BIRCH, SLOT_SPRUCE};
        let lut = build_lut();
        // state -> block
        for s in 0..STATE_COUNT {
            assert_eq!(lut[s], state_block(s as u16) as u32, "state_block({s})");
        }
        // flags encode the exact face_visible arm structure
        for b in 0..BLOCK_COUNT {
            let id = b as u8;
            let f = lut[235 + b];
            if id == AIR {
                // the CPU and the WGSL both guard AIR BEFORE face_visible
                // (vc_blocks: `if b == AIR return false`; shader:
                // `if b == 0u ... return`) — the flags path is never
                // consulted for an air emitter
                assert_eq!(
                    vc_blocks::blocks::face_visible(AIR, 3u8),
                    false,
                    "face_visible(AIR, stone) must short-circuit"
                );
                continue;
            }
            // simulate the WGSL face_visible over a representative neighbor
            // set (air / opaque stone / same-kind) and compare to Rust
            for n in [0u8, 3u8, id] {
                let fnb = lut[235 + n as usize];
                let got = if (f & 2) != 0 {
                    (fnb & 1) == 0 && (fnb & 2) == 0
                } else if (f & 4) != 0 {
                    (fnb & 1) == 0
                } else if (f & 8) != 0 {
                    (fnb & 1) == 0 && (fnb & 8) == 0
                } else if (f & 16) != 0 {
                    (fnb & 1) == 0 && (fnb & 16) == 0
                } else {
                    (fnb & 1) == 0
                };
                assert_eq!(
                    got,
                    vc_blocks::blocks::face_visible(id, n),
                    "face_visible({id}, {n}) via flags"
                );
            }
            // cross flag
            assert_eq!((f & 32) != 0, is_cross(id), "is_cross({id})");
        }
        // tiles
        for s in 0..STATE_COUNT {
            let t = state_tiles(s as u16);
            for i in 0..4 {
                assert_eq!(lut[439 + s * 4 + i], t[i] as u32, "tiles({s})[{i}]");
            }
        }
        // tint classes -> packed tints identical to vc_blocks::tint
        for b in 0..BLOCK_COUNT {
            let id = b as u8;
            for top in [true, false] {
                for biome in [0u8, 2, 4, 7] {
                    let tc = lut[337 + b];
                    let (kind, slot) = match tc {
                        1 if top => (KIND_GRASS as u32, biome as u32),
                        2 => (KIND_GRASS as u32, biome as u32),
                        3 => (KIND_FOLIAGE as u32, biome as u32),
                        4 => (KIND_FOLIAGE as u32, SLOT_BIRCH as u32),
                        5 => (KIND_FOLIAGE as u32, SLOT_SPRUCE as u32),
                        6 => (KIND_WATER as u32, biome as u32),
                        _ => (0, 0),
                    };
                    let got = (kind << 6) | slot;
                    assert_eq!(
                        got,
                        vc_blocks::tint::block_face_tint_packed(id, top, biome) as u32,
                        "tint_packed({id}, top={top}, biome={biome})"
                    );
                }
            }
        }
    }

    /// offset table determinism + section-base correctness (pure logic —
    /// runs everywhere including CI)
    #[test]
    fn offsets_are_deterministic_and_section_local() {
        // synthetic counts: unit g emits (g % 3) solid + (g % 2) water quads
        let n = 2;
        let mut counts = vec![0u32; n * UNITS * 2];
        for g in 0..n * UNITS {
            counts[g * 2] = (g % 3) as u32;
            counts[g * 2 + 1] = (g % 2) as u32;
        }
        let (offsets, svr, sir, wvr, wir) = compute_offsets(&counts, n);
        // totals: sum(sq)*16 words etc.
        let sq_total: usize = counts.iter().step_by(2).map(|c| *c as usize).sum();
        let wq_total: usize = counts.iter().skip(1).step_by(2).map(|c| *c as usize).sum();
        assert_eq!(svr, sq_total * 16);
        assert_eq!(sir, sq_total * 6);
        assert_eq!(wvr, wq_total * 16);
        assert_eq!(wir, wq_total * 6);
        // strictly non-overlapping unit ranges in iteration order
        for g in 0..n * UNITS - 1 {
            let sq = counts[g * 2] as usize;
            let wq = counts[g * 2 + 1] as usize;
            assert_eq!(
                offsets[g * 6] + (sq * 16) as u32,
                offsets[(g + 1) * 6],
                "solid vert stream must be gapless in unit order"
            );
            assert_eq!(
                offsets[g * 6 + 1] + (sq * 6) as u32,
                offsets[(g + 1) * 6 + 1],
                "solid idx stream must be gapless"
            );
        }
        // water stream starts at the solid totals (combined layout)
        assert_eq!(offsets[2], svr as u32, "water vert stream shift");
        assert_eq!(offsets[3], sir as u32, "water idx stream shift");
        // section-local bases: unit at the section start has base 0
        for j in 0..n {
            for sec in 0..16usize {
                let first = j * UNITS + sec * 16;
                assert_eq!(offsets[first * 6 + 4], 0, "section {sec} first unit base");
            }
        }
        // zero-count batch degenerates safely
        let (o0, s0, i0, w0, wi0) = compute_offsets(&vec![0u32; UNITS * 2], 1);
        assert_eq!((s0, i0, w0, wi0), (0, 0, 0, 0));
        assert!(o0.iter().all(|v| *v <= (UNITS * 6) as u32));
    }

    /// END-TO-END GPU parity: run the compute mesher on a headless device
    /// and compare byte-for-byte against `mesh_sections` (the design
    /// contract). Skips gracefully when no adapter exists (CI containers,
    /// display-less machines) — the browser E2E covers SwiftShader/WebGPU
    /// execution there.
    #[test]
    fn gpu_mesh_parity_bit_identical() {
        use vc_chunk::chunk::Chunk;
        use vc_mesh::mesh::{build_mesh_inputs, mesh_chunk};
        use vc_world::gen::TerrainGen;
        use vc_world::light::reference_lightdata;

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::None,
            compatible_surface: None,
            force_fallback_adapter: true,
        }));
        let Some(adapter) = adapter else {
            eprintln!("SKIP gpu_mesh_parity_bit_identical: no GPU adapter");
            return;
        };
        let downlevel = adapter.get_downlevel_capabilities();
        if !downlevel
            .flags
            .contains(wgpu::DownlevelFlags::COMPUTE_SHADERS)
        {
            eprintln!("SKIP gpu_mesh_parity_bit_identical: adapter lacks compute");
            return;
        }
        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: Some("gpu-mesh-test"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                ..Default::default()
            },
            None,
        ))
        .expect("headless device");

        // deterministic test worlds: 3 generated chunks (terrain + water +
        // varied biomes) meshed as 3 jobs in ONE batch, plus smooth OFF
        // variant to cover both flag paths
        let gen = TerrainGen::for_dimension(0xC0FFEE, vc_world::world::Dimension::Overworld);
        let positions = [(0i32, 0i32), (1, 0), (-2, 3)];
        let mut chunks: Vec<(vc_world::world::ChunkPos, Arc<Chunk>)> = Vec::new();
        for pos in positions {
            let (chunk, _) = gen.generate_chunk(pos.0, pos.1, Vec::new());
            chunks.push((pos, chunk));
        }

        let mut mesher = GpuMesher::new(&device, &queue);
        for smooth in [true, false] {
            for (pos, chunk) in chunks.iter() {
                // 3x3 snapshot: center + the other test chunks where they
                // overlap, None elsewhere (absent-neighbor padding path)
                let mut snap: [Option<Arc<Chunk>>; 9] = std::array::from_fn(|_| None);
                for (p, c) in chunks.iter() {
                    let dx = p.0 - pos.0 + 1;
                    let dz = p.1 - pos.1 + 1;
                    if (0..3).contains(&dx) && (0..3).contains(&dz) {
                        snap[dz as usize * 3 + dx as usize] = Some(Arc::clone(c));
                    }
                }
                let lsnap = reference_lightdata(&snap);
                let want = mesh_chunk(*pos, &snap, &lsnap, smooth);
                let inputs = build_mesh_inputs(&snap, &lsnap);
                assert!(
                    !inputs.has_cross && !inputs.has_models,
                    "test chunks must stay in the greedy-only regime"
                );
                mesher.enqueue(
                    GpuMeshJobMeta {
                        pos: *pos,
                        mask: u16::MAX,
                        smooth,
                        prev: vec![None; 16],
                        center: Some(Arc::clone(chunk)),
                    },
                    inputs,
                );
                let done = mesher.wait_done(&device, &queue);
                assert_eq!(done.len(), 1, "one job per batch");
                let got = &done[0].mesh;
                assert_eq!(
                    got.solid.0, want.solid.0,
                    "solid verts differ (pos {pos:?}, smooth {smooth})"
                );
                assert_eq!(
                    got.solid.1, want.solid.1,
                    "solid indices differ (pos {pos:?}, smooth {smooth})"
                );
                assert_eq!(
                    got.water.0, want.water.0,
                    "water verts differ (pos {pos:?}, smooth {smooth})"
                );
                assert_eq!(
                    got.water.1, want.water.1,
                    "water indices differ (pos {pos:?}, smooth {smooth})"
                );
            }
        }
    }
}
