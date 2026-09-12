//! Entity box models — a lightweight in-engine bone/joint hierarchy.
//!
//! Design follows the SHAPE of Luanti's entity-model approach as an
//! architectural reference only (LGPL boundary: technique, never code):
//! Luanti discloses skeletal bone animation with NAMED single-timeline
//! animation ranges and simple linear keyframe interpolation —
//! deliberately no blend graph (its own documented limitation). This
//! module re-implements that idea from scratch with our own Rust types:
//! no B3D/glTF/OBJ parsing (our models are code-generated, so no file
//! format is needed at all), a minimal per-vertex attribute set
//! (position + uv + color — color carries the baked directional face
//! shading, even leaner than Luanti's position/normal/uv), and
//! per-mob named ranges (`walk`, `attack`, `hurt`, `idle`).
//! Technique reference: Luanti's `docs.luanti.org/for-creators/models`
//! + `src/client/content_cao.cpp`, studied 2026-09-12; independently
//! reimplemented.
//!
//! Vanilla 1.16.5 reference for the PROPORTIONS (documented mechanical
//! behavior): humanoid mobs are 32 model px tall (8×8×8 head, 8×12×4
//! body, 4×12×4 limbs; zombie arms held forward), the creeper is a
//! 26-px stack, the enderman a 44-px tall thin frame. Model px are
//! authored 16-per-block and then auto-scaled to each kind's verified
//! hitbox height (`MOB_DATA.height`), so e.g. the wither skeleton's
//! 2.4 blocks stretch the humanoid rig correctly.
//!
//! Textures reuse the existing clean-room 16×16 mob sprites as
//! per-part-face sub-rects (the sprites are laid out head-top /
//! torso-middle / legs-bottom, so each box face samples the
//! palette-correct region of our own art — no Mojang assets, no new
//! atlas allocations).
//!
//! Rendering rides the existing billboard vertex stream
//! (`ParticleVertex`) with CPU skinning (mob counts are small; the
//! joint math is a handful of 3×3 rotations per part). Faces are
//! emitted back-to-front by view distance (painter's algorithm within
//! one mob) and carry vanilla-style directional shading
//! (top 1.0 / bottom 0.55 / z 0.85 / x 0.72). Known limitation,
//! documented: the billboard pipeline depth-TESTS but does not depth-
//! WRITE, so mob-vs-mob overlap relies on draw order (same exposure
//! the sprite billboards already have); a dedicated opaque entity pass
//! (Luanti's two-pass opaque-then-transparent shape) is the natural
//! future upgrade and is NOT needed for correctness at current mob
//! counts.

use vc_particles::particles::ParticleVertex;

/// One joint in the hierarchy. Geometry is authored in MODEL PX
/// (16 per block, origin at the entity's ground-plane center, +y up,
/// model faces +z) relative to the part's pivot.
pub struct PartDef {
    pub name: &'static str,
    /// parent joint index (every model has a `root` part at index 0
    /// whose pivot is the origin — hurt-style whole-body tracks key
    /// the root; real parts hang off it)
    pub parent: u16,
    /// joint position in model px (see module docs for the frame)
    pub pivot: [f32; 3],
    /// static pose offset in radians (the zombie's forward arms) —
    /// animation rotations ADD to this
    pub base_rot: [f32; 3],
    pub boxes: Vec<BoxDef>,
}

/// One axis-aligned box in part-local px (min corner relative to the
/// part pivot).
pub struct BoxDef {
    pub off: [f32; 3],
    pub size: [f32; 3],
    /// per-face atlas sub-rects (see [`FaceTex`])
    pub tex: FaceTex,
}

/// Per-face texture rects into the mob's 16×16 sprite tile.
/// Face order: +x (east), -x (west), +y (top), -y (bottom),
/// +z (front/south), -z (back/north). Each rect is `[x, y, w, h]` in
/// TILE-LOCAL pixels — the emitter converts to atlas UVs (512-px
/// sheet, 32 tiles per row, tile-local row 0 at the tile's v-min,
/// matching the billboard sampler exactly).
#[derive(Copy, Clone)]
pub struct FaceTex {
    pub tile: u16,
    pub rect: [[u8; 4]; 6],
}

impl FaceTex {
    /// every face samples the same sub-rect (for boxes whose faces are
    /// visually uniform, e.g. limbs)
    pub fn uniform(tile: u16, r: [u8; 4]) -> Self {
        FaceTex {
            tile,
            rect: [r; 6],
        }
    }
}

/// One keyframe: `t` is the loop fraction in `[0, 1)`; `rot` is the
/// part-local rotation DELTA in radians (added to `base_rot`).
#[derive(Copy, Clone)]
pub struct Key {
    pub t: f32,
    pub rot: [f32; 3],
}

/// A named single-timeline animation range (Luanti's model: ONE range
/// plays at a time — no blending; the driver picks the active range by
/// mob state, exactly the simplicity level Luanti itself ships).
pub struct AnimRange {
    pub name: &'static str,
    /// one track per animated part: (part index, keyframes sorted by t)
    pub tracks: Vec<(u16, Vec<Key>)>,
}

/// A complete rig + its animation ranges.
pub struct EntityModel {
    /// parts[0] is always the `root` (pivot at the origin, no boxes)
    pub parts: Vec<PartDef>,
    pub anims: Vec<AnimRange>,
    /// total authored height in model px (auto-scale denominator)
    pub px_height: f32,
}

// ------------------------------------------------------------- sampling --

/// Sample one animation range at loop-fraction `phase01` (wraps).
/// Returns per-part rotation deltas (`[parts.len()]`, zeros when the
/// named range does not exist — the idle case). Keyframes interpolate
/// LINEARLY and the last→first wrap is a real segment.
pub fn sample_anim(model: &EntityModel, name: &str, phase01: f32) -> Vec<[f32; 3]> {
    let mut out = vec![[0f32; 3]; model.parts.len()];
    let Some(anim) = model.anims.iter().find(|a| a.name == name) else {
        return out;
    };
    let t = phase01.fract();
    // keep the wrap arithmetic in [0, 1)
    let t = if t < 0.0 { t + 1.0 } else { t };
    for (part, keys) in &anim.tracks {
        let i = (*part as usize).min(out.len() - 1);
        let rot = sample_track(keys, t);
        out[i] = rot;
    }
    out
}

/// linear keyframe interpolation on one track (piecewise-linear,
/// wrap-around between the last and first key)
fn sample_track(keys: &[Key], t: f32) -> [f32; 3] {
    if keys.is_empty() {
        return [0.0; 3];
    }
    if keys.len() == 1 {
        return keys[0].rot;
    }
    // find the segment [k[i].t, k[i+1].t) containing t, treating the
    // wrap (last key t → first key t + 1) as the final segment
    let n = keys.len();
    for i in 0..n {
        let a = &keys[i];
        let b = &keys[(i + 1) % n];
        let bt = if i + 1 < n { b.t } else { b.t + 1.0 };
        let at = a.t;
        if t >= at && t < bt {
            let span = (bt - at).max(1e-6);
            let f = ((t - at) / span).clamp(0.0, 1.0);
            return [
                a.rot[0] + (b.rot[0] - a.rot[0]) * f,
                a.rot[1] + (b.rot[1] - a.rot[1]) * f,
                a.rot[2] + (b.rot[2] - a.rot[2]) * f,
            ];
        }
    }
    // exactly on the last key's t (or a gap-less track miss): clamp
    keys[n - 1].rot
}

// --------------------------------------------------------------- emit ---

/// per-face directional shade (vanilla-style entity light): top
/// brightest, bottom darkest, z faces mid, x faces darker
const FACE_SHADE: [f32; 6] = [0.72, 0.72, 1.0, 0.55, 0.85, 0.85];

/// Emit one entity's skinned boxes into the billboard vertex stream.
///
/// * `pos` — the entity's FEET position (world blocks)
/// * `yaw` — the mob's steering yaw; the model faces its movement
///   direction (movement = `(−sin yaw, 0, −cos yaw)` — the steering
///   convention in `mobs.rs`), so the emitter rotates the +z-facing
///   model by `yaw + π`
/// * `part_rots` — per-part rotation deltas from [`sample_anim`]
///   (length must equal `parts.len()`)
/// * `scale` — blocks per model px (auto-fit: `hitbox_height /
///   px_height`)
/// * `tint` — hurt flash / creeper fuse multiplier (rgb)
/// * `view_dir` — camera forward (unit); painter's back-to-front face
///   ordering within the mob
pub fn emit_model_vertices(
    model: &EntityModel,
    pos: [f32; 3],
    yaw: f32,
    part_rots: &[[f32; 3]],
    scale: f32,
    tint: [f32; 3],
    view_dir: [f32; 3],
    out: &mut Vec<ParticleVertex>,
) {
    let n = model.parts.len();
    if part_rots.len() != n {
        return; // caller contract violation — skip cleanly, no panic
    }
    // part world transforms: (rotation 3×3, translation) in MODEL PX
    // space. The root (parts[0], pivot at the origin) carries the
    // yaw+π facing rotation COMPOSED with its own euler rotation —
    // hurt-style whole-body tracks key the root. Children compose as
    // R_child = R_parent · R_rot, T_child = R_parent·pivot + T_parent
    // (verified against reference math in
    // hierarchy_composition_matches_reference_math).
    let mut rots: Vec<[[f32; 3]; 3]> = Vec::with_capacity(n);
    let mut trans: Vec<[f32; 3]> = Vec::with_capacity(n);
    for (i, p) in model.parts.iter().enumerate() {
        let r = add3(&p.base_rot, &part_rots[i]);
        let rm = euler_xyz(r);
        if i == 0 {
            // root: facing rotation about the ground origin, then the
            // root's own (anim) rotation in the yawed frame
            let face = yaw + std::f32::consts::PI;
            rots.push(mat_mul(&rot_y(face), &rm));
            trans.push([0.0, 0.0, 0.0]);
        } else {
            let pr = &rots[p.parent as usize];
            let pt = &trans[p.parent as usize];
            // child joint = parent ∘ T(pivot) ∘ R:
            // world(v) = parent_R · (pivot + R·v) + parent_t
            let t = [
                pt[0] + pr[0][0] * p.pivot[0] + pr[0][1] * p.pivot[1] + pr[0][2] * p.pivot[2],
                pt[1] + pr[1][0] * p.pivot[0] + pr[1][1] * p.pivot[1] + pr[1][2] * p.pivot[2],
                pt[2] + pr[2][0] * p.pivot[0] + pr[2][1] * p.pivot[1] + pr[2][2] * p.pivot[2],
            ];
            rots.push(mat_mul(pr, &rm));
            trans.push(t);
        }
    }
    // painter ordering: parts far → near by the transformed pivot
    // (trans[i] IS the pivot's location in model px)
    let mut order: Vec<(f32, usize)> = (0..n)
        .map(|i| {
            let t = &trans[i];
            let w = [
                pos[0] + t[0] * scale,
                pos[1] + t[1] * scale,
                pos[2] + t[2] * scale,
            ];
            let d = w[0] * view_dir[0] + w[1] * view_dir[1] + w[2] * view_dir[2];
            (d, i)
        })
        .collect();
    order.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

    for (_, i) in order {
        let p = &model.parts[i];
        let r = &rots[i];
        let t = &trans[i];
        for b in &p.boxes {
            emit_box(b, pos, r, t, scale, tint, view_dir, out);
        }
    }
}

/// emit one box's 6 faces, back-to-front by face-center view distance
fn emit_box(
    b: &BoxDef,
    pos: [f32; 3],
    r: &[[f32; 3]; 3],
    t: &[f32; 3],
    scale: f32,
    tint: [f32; 3],
    view_dir: [f32; 3],
    out: &mut Vec<ParticleVertex>,
) {
    // the 8 box corners in part-local px (relative to the pivot)
    let x0 = b.off[0];
    let y0 = b.off[1];
    let z0 = b.off[2];
    let x1 = x0 + b.size[0];
    let y1 = y0 + b.size[1];
    let z1 = z0 + b.size[2];
    // face: (4 corner ids in CCW-from-outside order, face index)
    // corner id → (x, y, z) pick 0/1 per axis
    const FACES: [([usize; 4], usize); 6] = [
        ([1, 3, 7, 5], 0), // +x: corners with x = x1
        ([4, 6, 2, 0], 1), // -x
        ([2, 3, 7, 6], 2), // +y (top)
        ([0, 1, 5, 4], 3), // -y (bottom)
        ([0, 2, 3, 1], 4), // +z (front)
        ([4, 5, 7, 6], 5), // -z (back)
    ];
    let corner = |c: usize| -> [f32; 3] {
        [
            if c & 1 != 0 { x1 } else { x0 },
            if c & 2 != 0 { y1 } else { y0 },
            if c & 4 != 0 { z1 } else { z0 },
        ]
    };
    // world-space centers for painter ordering
    let mut faces: Vec<(f32, usize)> = FACES
        .iter()
        .map(|(cs, fi)| {
            let mut wc = [0f32; 3];
            for &c in cs {
                let v = corner(c);
                let w = [
                    t[0] + r[0][0] * v[0] + r[0][1] * v[1] + r[0][2] * v[2],
                    t[1] + r[1][0] * v[0] + r[1][1] * v[1] + r[1][2] * v[2],
                    t[2] + r[2][0] * v[0] + r[2][1] * v[1] + r[2][2] * v[2],
                ];
                wc[0] += pos[0] + w[0] * scale;
                wc[1] += pos[1] + w[1] * scale;
                wc[2] += pos[2] + w[2] * scale;
            }
            let d = wc[0] * view_dir[0] + wc[1] * view_dir[1] + wc[2] * view_dir[2];
            (d, *fi)
        })
        .collect();
    faces.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

    let tx = (b.tex.tile % 32) as f32;
    let ty = (b.tex.tile / 32) as f32;
    for (_, fi) in faces {
        let (cs, fi) = FACES[fi];
        let [rx, ry, rw, rh] = b.tex.rect[fi].map(|v| v as f32);
        // atlas UVs: 512-px sheet, 16-px tiles; sprite row 0 = v-min
        let u0 = (tx * 16.0 + rx) / 512.0;
        let u1 = (tx * 16.0 + rx + rw) / 512.0;
        let v0 = (ty * 16.0 + ry) / 512.0;
        let v1 = (ty * 16.0 + ry + rh) / 512.0;
        // face-local UV corners: CCW from the face's top-left as seen
        // from OUTSIDE (u right, v down)
        let uvs = [[u0, v0], [u1, v0], [u1, v1], [u0, v1]];
        let shade = FACE_SHADE[fi];
        let col = [tint[0] * shade, tint[1] * shade, tint[2] * shade];
        // 2 triangles from the 4 corners
        for ci in [0usize, 1, 2, 0, 2, 3] {
            let v = corner(cs[ci]);
            let w = [
                t[0] + r[0][0] * v[0] + r[0][1] * v[1] + r[0][2] * v[2],
                t[1] + r[1][0] * v[0] + r[1][1] * v[1] + r[1][2] * v[2],
                t[2] + r[2][0] * v[0] + r[2][1] * v[1] + r[2][2] * v[2],
            ];
            out.push(ParticleVertex {
                pos: [pos[0] + w[0] * scale, pos[1] + w[1] * scale, pos[2] + w[2] * scale],
                uv: uvs[ci],
                col,
            });
        }
    }
}

// ---------------------------------------------------------- math -------

fn add3(a: &[f32; 3], b: &[f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

/// Euler XYZ rotation matrix (v' = Rz·Ry·Rx·v — rotations apply x,
/// then y, then z; limb swings are rx, head yaw ry, hurt roll rz)
fn euler_xyz(r: [f32; 3]) -> [[f32; 3]; 3] {
    let (sx, cx) = r[0].sin_cos();
    let (sy, cy) = r[1].sin_cos();
    let (sz, cz) = r[2].sin_cos();
    // R = Rz · Ry · Rx (row-major, column-vector convention)
    [
        [cz * cy, cz * sy * sx - sz * cx, cz * sy * cx + sz * sx],
        [sz * cy, sz * sy * sx + cz * cx, sz * sy * cx - cz * sx],
        [-sy, cy * sx, cy * cx],
    ]
}

fn rot_y(theta: f32) -> [[f32; 3]; 3] {
    let (s, c) = theta.sin_cos();
    [[c, 0.0, s], [0.0, 1.0, 0.0], [-s, 0.0, c]]
}

fn mat_mul(a: &[[f32; 3]; 3], b: &[[f32; 3]; 3]) -> [[f32; 3]; 3] {
    let mut m = [[0f32; 3]; 3];
    for i in 0..3 {
        for j in 0..3 {
            m[i][j] = a[i][0] * b[0][j] + a[i][1] * b[1][j] + a[i][2] * b[2][j];
        }
    }
    m
}

/// reference-math helper (test-only): apply a rotation matrix
#[cfg(test)]
fn mat_vec(m: &[[f32; 3]; 3], v: &[f32; 3]) -> [f32; 3] {
    [
        m[0][0] * v[0] + m[0][1] * v[1] + m[0][2] * v[2],
        m[1][0] * v[0] + m[1][1] * v[1] + m[1][2] * v[2],
        m[2][0] * v[0] + m[2][1] * v[1] + m[2][2] * v[2],
    ]
}

// --------------------------------------------------------- the rigs ----

/// face tex helper: head-style faces (front = the sprite's face rect,
/// everything else = a plain region of the same sprite)
fn head_tex(tile: u16, front: [u8; 4], plain: [u8; 4]) -> FaceTex {
    let mut rect = [plain; 6];
    rect[4] = front;
    FaceTex { tile, rect }
}

/// the classic 32-px humanoid rig (zombie/skeleton families).
/// `arms_forward` = the zombie pose (both arms raised); skeleton
/// arms hang. Named ranges: walk / attack / hurt / idle.
pub fn humanoid(tile: u16, arms_forward: bool) -> EntityModel {
    let arm_base = if arms_forward {
        [-1.45, 0.0, 0.0] // vanilla-style zombie arms (~83° forward)
    } else {
        [0.0, 0.0, 0.0]
    };
    // sprite sub-rects: head rows 1-5 (face), torso rows 6-11,
    // arm columns rows 6-11, legs rows 12-14
    let head = head_tex(tile, [4, 1, 6, 5], [4, 1, 3, 3]);
    let torso = FaceTex::uniform(tile, [4, 6, 8, 6]);
    let arm = FaceTex::uniform(tile, [2, 6, 2, 6]);
    let leg = FaceTex::uniform(tile, [3, 12, 3, 3]);
    let parts = vec![
        PartDef {
            name: "root",
            parent: 0,
            pivot: [0.0, 0.0, 0.0],
            base_rot: [0.0; 3],
            boxes: Vec::new(),
        },
        PartDef {
            name: "body",
            parent: 0,
            pivot: [0.0, 12.0, 0.0],
            base_rot: [0.0; 3],
            boxes: vec![BoxDef {
                off: [-4.0, 0.0, -2.0],
                size: [8.0, 12.0, 4.0],
                tex: torso,
            }],
        },
        PartDef {
            name: "head",
            parent: 1,
            pivot: [0.0, 12.0, 0.0], // neck: at the body part's pivot height
            base_rot: [0.0; 3],
            boxes: vec![BoxDef {
                off: [-4.0, 0.0, -4.0],
                size: [8.0, 8.0, 8.0],
                tex: head,
            }],
        },
        PartDef {
            name: "arm_r",
            parent: 1,
            pivot: [-6.0, 11.0, 0.0],
            base_rot: arm_base,
            boxes: vec![BoxDef {
                off: [-2.0, -11.0, -2.0],
                size: [4.0, 12.0, 4.0],
                tex: arm,
            }],
        },
        PartDef {
            name: "arm_l",
            parent: 1,
            pivot: [6.0, 11.0, 0.0],
            base_rot: arm_base,
            boxes: vec![BoxDef {
                off: [-2.0, -11.0, -2.0],
                size: [4.0, 12.0, 4.0],
                tex: arm,
            }],
        },
        PartDef {
            name: "leg_r",
            parent: 0,
            pivot: [-2.0, 12.0, 0.0],
            base_rot: [0.0; 3],
            boxes: vec![BoxDef {
                off: [-2.0, -12.0, -2.0],
                size: [4.0, 12.0, 4.0],
                tex: leg,
            }],
        },
        PartDef {
            name: "leg_l",
            parent: 0,
            pivot: [2.0, 12.0, 0.0],
            base_rot: [0.0; 3],
            boxes: vec![BoxDef {
                off: [-2.0, -12.0, -2.0],
                size: [4.0, 12.0, 4.0],
                tex: leg,
            }],
        },
    ];
    // walk: 0.55-rad leg swing; skeleton arms counter-swing (0.35),
    // zombie forward arms wobble in place instead
    let zombie_arm = arms_forward;
    let swing = 0.55f32;
    let walk_tracks: Vec<(u16, Vec<Key>)> = if zombie_arm {
        vec![
            (
                5,
                vec![
                    Key { t: 0.0, rot: [0.0, 0.0, 0.0] },
                    Key { t: 0.25, rot: [swing, 0.0, 0.0] },
                    Key { t: 0.5, rot: [0.0, 0.0, 0.0] },
                    Key { t: 0.75, rot: [-swing, 0.0, 0.0] },
                ],
            ),
            (
                6,
                vec![
                    Key { t: 0.0, rot: [0.0, 0.0, 0.0] },
                    Key { t: 0.25, rot: [-swing, 0.0, 0.0] },
                    Key { t: 0.5, rot: [0.0, 0.0, 0.0] },
                    Key { t: 0.75, rot: [swing, 0.0, 0.0] },
                ],
            ),
            // zombie arms: a small constant-cadence wobble
            (
                3,
                vec![
                    Key { t: 0.0, rot: [0.0, 0.0, 0.0] },
                    Key { t: 0.25, rot: [0.1, 0.0, 0.05] },
                    Key { t: 0.5, rot: [0.0, 0.0, 0.0] },
                    Key { t: 0.75, rot: [0.1, 0.0, -0.05] },
                ],
            ),
            (
                4,
                vec![
                    Key { t: 0.0, rot: [0.0, 0.0, 0.0] },
                    Key { t: 0.25, rot: [0.1, 0.0, -0.05] },
                    Key { t: 0.5, rot: [0.0, 0.0, 0.0] },
                    Key { t: 0.75, rot: [0.1, 0.0, 0.05] },
                ],
            ),
        ]
    } else {
        vec![
            (
                5,
                vec![
                    Key { t: 0.0, rot: [0.0, 0.0, 0.0] },
                    Key { t: 0.25, rot: [swing, 0.0, 0.0] },
                    Key { t: 0.5, rot: [0.0, 0.0, 0.0] },
                    Key { t: 0.75, rot: [-swing, 0.0, 0.0] },
                ],
            ),
            (
                6,
                vec![
                    Key { t: 0.0, rot: [0.0, 0.0, 0.0] },
                    Key { t: 0.25, rot: [-swing, 0.0, 0.0] },
                    Key { t: 0.5, rot: [0.0, 0.0, 0.0] },
                    Key { t: 0.75, rot: [swing, 0.0, 0.0] },
                ],
            ),
            (
                3,
                vec![
                    Key { t: 0.0, rot: [0.0, 0.0, 0.0] },
                    Key { t: 0.25, rot: [-0.35, 0.0, 0.0] },
                    Key { t: 0.5, rot: [0.0, 0.0, 0.0] },
                    Key { t: 0.75, rot: [0.35, 0.0, 0.0] },
                ],
            ),
            (
                4,
                vec![
                    Key { t: 0.0, rot: [0.0, 0.0, 0.0] },
                    Key { t: 0.25, rot: [0.35, 0.0, 0.0] },
                    Key { t: 0.5, rot: [0.0, 0.0, 0.0] },
                    Key { t: 0.75, rot: [-0.35, 0.0, 0.0] },
                ],
            ),
        ]
    };
    EntityModel {
        parts,
        anims: vec![
            AnimRange {
                name: "walk",
                tracks: walk_tracks,
            },
            // attack: both arms slash (raise then chop) over the range
            AnimRange {
                name: "attack",
                tracks: vec![
                    (
                        3,
                        vec![
                            Key { t: 0.0, rot: [0.0, 0.0, 0.0] },
                            Key { t: 0.3, rot: [-0.5, 0.0, 0.0] },
                            Key { t: 0.7, rot: [0.6, 0.0, 0.0] },
                        ],
                    ),
                    (
                        4,
                        vec![
                            Key { t: 0.0, rot: [0.0, 0.0, 0.0] },
                            Key { t: 0.3, rot: [-0.5, 0.0, 0.0] },
                            Key { t: 0.7, rot: [0.6, 0.0, 0.0] },
                        ],
                    ),
                ],
            },
            // hurt: whole-body roll arc through the root
            AnimRange {
                name: "hurt",
                tracks: vec![(
                    0,
                    vec![
                        Key { t: 0.0, rot: [0.0, 0.0, 0.0] },
                        Key { t: 0.5, rot: [0.0, 0.0, 0.14] },
                        Key { t: 1.0, rot: [0.0, 0.0, 0.0] },
                    ],
                )],
            },
            AnimRange {
                name: "idle",
                tracks: Vec::new(),
            },
        ],
        px_height: 32.0,
    }
}

/// the 44-px enderman: long thin limbs, narrow torso, glowing-eye
/// head — the sprite's K/V columns map to the limbs.
pub fn enderman(tile: u16) -> EntityModel {
    let head = head_tex(tile, [4, 0, 6, 4], [4, 0, 3, 4]);
    let torso = FaceTex::uniform(tile, [5, 5, 6, 5]);
    let limb = FaceTex::uniform(tile, [6, 11, 2, 4]);
    let parts = vec![
        PartDef {
            name: "root",
            parent: 0,
            pivot: [0.0, 0.0, 0.0],
            base_rot: [0.0; 3],
            boxes: Vec::new(),
        },
        PartDef {
            name: "body",
            parent: 0,
            pivot: [0.0, 24.0, 0.0],
            base_rot: [0.0; 3],
            boxes: vec![BoxDef {
                off: [-4.0, 0.0, -2.0],
                size: [8.0, 12.0, 4.0],
                tex: torso,
            }],
        },
        PartDef {
            name: "head",
            parent: 1,
            pivot: [0.0, 12.0, 0.0],
            base_rot: [0.0; 3],
            boxes: vec![BoxDef {
                off: [-4.0, 0.0, -4.0],
                size: [8.0, 8.0, 8.0],
                tex: head,
            }],
        },
        PartDef {
            name: "arm_r",
            parent: 1,
            pivot: [-6.0, 11.0, 0.0],
            base_rot: [0.0; 3],
            boxes: vec![BoxDef {
                off: [-1.0, -23.0, -1.0],
                size: [2.0, 24.0, 2.0],
                tex: limb,
            }],
        },
        PartDef {
            name: "arm_l",
            parent: 1,
            pivot: [6.0, 11.0, 0.0],
            base_rot: [0.0; 3],
            boxes: vec![BoxDef {
                off: [-1.0, -23.0, -1.0],
                size: [2.0, 24.0, 2.0],
                tex: limb,
            }],
        },
        PartDef {
            name: "leg_r",
            parent: 0,
            pivot: [-2.0, 24.0, 0.0],
            base_rot: [0.0; 3],
            boxes: vec![BoxDef {
                off: [-1.5, -24.0, -1.5],
                size: [3.0, 24.0, 3.0],
                tex: limb,
            }],
        },
        PartDef {
            name: "leg_l",
            parent: 0,
            pivot: [2.0, 24.0, 0.0],
            base_rot: [0.0; 3],
            boxes: vec![BoxDef {
                off: [-1.5, -24.0, -1.5],
                size: [3.0, 24.0, 3.0],
                tex: limb,
            }],
        },
    ];
    EntityModel {
        parts,
        anims: vec![
            AnimRange {
                name: "walk",
                tracks: vec![
                    (
                        5,
                        vec![
                            Key { t: 0.0, rot: [0.0, 0.0, 0.0] },
                            Key { t: 0.25, rot: [0.45, 0.0, 0.0] },
                            Key { t: 0.5, rot: [0.0, 0.0, 0.0] },
                            Key { t: 0.75, rot: [-0.45, 0.0, 0.0] },
                        ],
                    ),
                    (
                        6,
                        vec![
                            Key { t: 0.0, rot: [0.0, 0.0, 0.0] },
                            Key { t: 0.25, rot: [-0.45, 0.0, 0.0] },
                            Key { t: 0.5, rot: [0.0, 0.0, 0.0] },
                            Key { t: 0.75, rot: [0.45, 0.0, 0.0] },
                        ],
                    ),
                ],
            },
            AnimRange {
                name: "attack",
                tracks: vec![
                    (
                        3,
                        vec![
                            Key { t: 0.0, rot: [0.0, 0.0, 0.0] },
                            Key { t: 0.4, rot: [-1.2, 0.0, 0.0] },
                            Key { t: 0.8, rot: [0.3, 0.0, 0.0] },
                        ],
                    ),
                    (
                        4,
                        vec![
                            Key { t: 0.0, rot: [0.0, 0.0, 0.0] },
                            Key { t: 0.4, rot: [-1.2, 0.0, 0.0] },
                            Key { t: 0.8, rot: [0.3, 0.0, 0.0] },
                        ],
                    ),
                ],
            },
            AnimRange {
                name: "hurt",
                tracks: vec![(
                    0,
                    vec![
                        Key { t: 0.0, rot: [0.0, 0.0, 0.0] },
                        Key { t: 0.5, rot: [0.0, 0.0, 0.12] },
                        Key { t: 1.0, rot: [0.0, 0.0, 0.0] },
                    ],
                )],
            },
            AnimRange {
                name: "idle",
                tracks: Vec::new(),
            },
        ],
        px_height: 44.0,
    }
}

/// the 26-px creeper: head + slim body + four stubby legs
pub fn creeper(tile: u16) -> EntityModel {
    let head = head_tex(tile, [4, 2, 7, 6], [4, 2, 3, 3]);
    let body = FaceTex::uniform(tile, [4, 8, 8, 1]);
    let foot = FaceTex::uniform(tile, [4, 9, 2, 3]);
    let parts = vec![
        PartDef {
            name: "root",
            parent: 0,
            pivot: [0.0, 0.0, 0.0],
            base_rot: [0.0; 3],
            boxes: Vec::new(),
        },
        PartDef {
            name: "body",
            parent: 0,
            pivot: [0.0, 6.0, 0.0],
            base_rot: [0.0; 3],
            boxes: vec![BoxDef {
                off: [-4.0, 0.0, -2.0],
                size: [8.0, 12.0, 4.0],
                tex: body,
            }],
        },
        PartDef {
            name: "head",
            parent: 1,
            pivot: [0.0, 12.0, 0.0],
            base_rot: [0.0; 3],
            boxes: vec![BoxDef {
                off: [-4.0, 0.0, -4.0],
                size: [8.0, 8.0, 8.0],
                tex: head,
            }],
        },
        PartDef {
            name: "leg_fr",
            parent: 0,
            pivot: [-2.0, 6.0, 2.0],
            base_rot: [0.0; 3],
            boxes: vec![BoxDef {
                off: [-2.0, -6.0, 0.0],
                size: [4.0, 6.0, 4.0],
                tex: foot,
            }],
        },
        PartDef {
            name: "leg_fl",
            parent: 0,
            pivot: [2.0, 6.0, 2.0],
            base_rot: [0.0; 3],
            boxes: vec![BoxDef {
                off: [-2.0, -6.0, 0.0],
                size: [4.0, 6.0, 4.0],
                tex: foot,
            }],
        },
        PartDef {
            name: "leg_br",
            parent: 0,
            pivot: [-2.0, 6.0, -2.0],
            base_rot: [0.0; 3],
            boxes: vec![BoxDef {
                off: [-2.0, -6.0, -4.0],
                size: [4.0, 6.0, 4.0],
                tex: foot,
            }],
        },
        PartDef {
            name: "leg_bl",
            parent: 0,
            pivot: [2.0, 6.0, -2.0],
            base_rot: [0.0; 3],
            boxes: vec![BoxDef {
                off: [-2.0, -6.0, -4.0],
                size: [4.0, 6.0, 4.0],
                tex: foot,
            }],
        },
    ];
    EntityModel {
        parts,
        anims: vec![
            AnimRange {
                name: "walk",
                tracks: vec![
                    (
                        3,
                        vec![
                            Key { t: 0.0, rot: [0.0, 0.0, 0.0] },
                            Key { t: 0.25, rot: [0.5, 0.0, 0.0] },
                            Key { t: 0.5, rot: [0.0, 0.0, 0.0] },
                            Key { t: 0.75, rot: [-0.5, 0.0, 0.0] },
                        ],
                    ),
                    (
                        4,
                        vec![
                            Key { t: 0.0, rot: [0.0, 0.0, 0.0] },
                            Key { t: 0.25, rot: [-0.5, 0.0, 0.0] },
                            Key { t: 0.5, rot: [0.0, 0.0, 0.0] },
                            Key { t: 0.75, rot: [0.5, 0.0, 0.0] },
                        ],
                    ),
                    (
                        5,
                        vec![
                            Key { t: 0.0, rot: [0.0, 0.0, 0.0] },
                            Key { t: 0.25, rot: [-0.5, 0.0, 0.0] },
                            Key { t: 0.5, rot: [0.0, 0.0, 0.0] },
                            Key { t: 0.75, rot: [0.5, 0.0, 0.0] },
                        ],
                    ),
                    (
                        6,
                        vec![
                            Key { t: 0.0, rot: [0.0, 0.0, 0.0] },
                            Key { t: 0.25, rot: [0.5, 0.0, 0.0] },
                            Key { t: 0.5, rot: [0.0, 0.0, 0.0] },
                            Key { t: 0.75, rot: [-0.5, 0.0, 0.0] },
                        ],
                    ),
                ],
            },
            AnimRange {
                name: "attack",
                // the pre-explosion swell: body tips back
                tracks: vec![(
                    0,
                    vec![
                        Key { t: 0.0, rot: [0.0, 0.0, 0.0] },
                        Key { t: 1.0, rot: [-0.18, 0.0, 0.0] },
                    ],
                )],
            },
            AnimRange {
                name: "hurt",
                tracks: vec![(
                    0,
                    vec![
                        Key { t: 0.0, rot: [0.0, 0.0, 0.0] },
                        Key { t: 0.5, rot: [0.0, 0.0, 0.14] },
                        Key { t: 1.0, rot: [0.0, 0.0, 0.0] },
                    ],
                )],
            },
            AnimRange {
                name: "idle",
                tracks: Vec::new(),
            },
        ],
        px_height: 26.0,
    }
}

/// the low-slung spider: abdomen + head + eight splayed legs
/// (single boxes with a splay base pose, alternating tetrapod gait
/// in walk)
pub fn spider(tile: u16) -> EntityModel {
    let body = FaceTex::uniform(tile, [3, 5, 10, 6]);
    let head_tex_ = head_tex(tile, [3, 7, 8, 3], [3, 5, 5, 4]);
    let leg = FaceTex::uniform(tile, [2, 4, 2, 2]);
    let mut parts = vec![
        PartDef {
            name: "root",
            parent: 0,
            pivot: [0.0, 0.0, 0.0],
            base_rot: [0.0; 3],
            boxes: Vec::new(),
        },
        PartDef {
            name: "abdomen",
            parent: 0,
            pivot: [0.0, 8.0, -2.0],
            base_rot: [0.0; 3],
            boxes: vec![BoxDef {
                off: [-5.0, -3.0, -5.0],
                size: [10.0, 6.0, 10.0],
                tex: body,
            }],
        },
        PartDef {
            name: "head",
            parent: 1,
            pivot: [0.0, 0.0, 6.0],
            base_rot: [0.0; 3],
            boxes: vec![BoxDef {
                off: [-4.0, -3.0, 0.0],
                size: [8.0, 6.0, 4.0],
                tex: head_tex_,
            }],
        },
    ];
    // 8 legs: pivots spread along z on the abdomen's flanks; base pose
    // splays them outward (rz) and walk swings fore/aft (rx) in 2
    // alternating groups
    // (side sign, pivot_z, splay rz, walk group)
    let leg_defs: [(f32, f32, f32, f32); 8] = [
        (-1.0, 3.0, 0.45, 0.0),
        (-1.0, 0.0, 0.5, 1.0),
        (-1.0, -3.0, 0.45, 0.0),
        (-1.0, -5.0, 0.35, 1.0),
        (1.0, 3.0, -0.45, 1.0),
        (1.0, 0.0, -0.5, 0.0),
        (1.0, -3.0, -0.45, 1.0),
        (1.0, -5.0, -0.35, 0.0),
    ];
    let mut walk_tracks: Vec<(u16, Vec<Key>)> = Vec::new();
    for (i, &(sx, pz, splay, group)) in leg_defs.iter().enumerate() {
        let idx = (parts.len() + i) as u16;
        let dir = sx; // -1 → leg extends -x; +1 → extends +x
        parts.push(PartDef {
            name: "leg",
            parent: 1, // abdomen
            pivot: [5.0 * dir, 0.0, pz],
            base_rot: [0.0, 0.0, splay],
            boxes: vec![BoxDef {
                off: if dir < 0.0 {
                    [-12.0, -1.0, -1.0]
                } else {
                    [0.0, -1.0, -1.0]
                },
                size: [12.0, 2.0, 2.0],
                tex: leg,
            }],
        });
        let amp = 0.35f32;
        let (a, b) = if group == 0.0 {
            (amp, -amp)
        } else {
            (-amp, amp)
        };
        walk_tracks.push((
            idx,
            vec![
                Key { t: 0.0, rot: [0.0, 0.0, 0.0] },
                Key { t: 0.25, rot: [a, 0.0, 0.0] },
                Key { t: 0.5, rot: [0.0, 0.0, 0.0] },
                Key { t: 0.75, rot: [b, 0.0, 0.0] },
            ],
        ));
    }
    EntityModel {
        parts,
        anims: vec![
            AnimRange {
                name: "walk",
                tracks: walk_tracks,
            },
            AnimRange {
                name: "attack",
                // front legs raise
                tracks: vec![
                    (
                        3,
                        vec![
                            Key { t: 0.0, rot: [0.0, 0.0, 0.0] },
                            Key { t: 0.5, rot: [-0.5, 0.0, 0.0] },
                            Key { t: 1.0, rot: [0.0, 0.0, 0.0] },
                        ],
                    ),
                    (
                        7,
                        vec![
                            Key { t: 0.0, rot: [0.0, 0.0, 0.0] },
                            Key { t: 0.5, rot: [-0.5, 0.0, 0.0] },
                            Key { t: 1.0, rot: [0.0, 0.0, 0.0] },
                        ],
                    ),
                ],
            },
            AnimRange {
                name: "hurt",
                tracks: vec![(
                    0,
                    vec![
                        Key { t: 0.0, rot: [0.0, 0.0, 0.0] },
                        Key { t: 0.5, rot: [0.0, 0.0, 0.12] },
                        Key { t: 1.0, rot: [0.0, 0.0, 0.0] },
                    ],
                )],
            },
            AnimRange {
                name: "idle",
                tracks: Vec::new(),
            },
        ],
        px_height: 14.0,
    }
}

// ---------------------------------------------------------------- tests --

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sample_anim_linear_and_wrap() {
        // 2 keys: t=0 → 0, t=0.5 → 1.0 (rx); wrap segment back to 0
        let model = EntityModel {
            parts: vec![
                PartDef {
                    name: "root",
                    parent: 0,
                    pivot: [0.0; 3],
                    base_rot: [0.0; 3],
                    boxes: Vec::new(),
                },
                PartDef {
                    name: "limb",
                    parent: 0,
                    pivot: [0.0, 12.0, 0.0],
                    base_rot: [0.0; 3],
                    boxes: Vec::new(),
                },
            ],
            anims: vec![AnimRange {
                name: "walk",
                tracks: vec![(
                    1,
                    vec![
                        Key { t: 0.0, rot: [0.0, 0.0, 0.0] },
                        Key { t: 0.5, rot: [1.0, 0.0, 0.0] },
                    ],
                )],
            }],
            px_height: 32.0,
        };
        let r = sample_anim(&model, "walk", 0.25);
        assert!((r[1][0] - 0.5).abs() < 1e-5, "halfway interpolates to 0.5");
        let r = sample_anim(&model, "walk", 0.75);
        // wrap segment: from key(t=0.5, rx=1.0) back to key(t=0 → 1.0, rx=0)
        assert!((r[1][0] - 0.5).abs() < 1e-5, "wrap segment descends to 0.5");
        // phase wraps: 1.25 ≡ 0.25
        let r2 = sample_anim(&model, "walk", 1.25);
        assert_eq!(r[1][0], r2[1][0]);
        // unknown range → zeros (idle)
        let r3 = sample_anim(&model, "nope", 0.3);
        assert_eq!(r3[1], [0.0; 3]);
    }

    #[test]
    fn hierarchy_composition_matches_reference_math() {
        // root rot = Rz(0.1); child pivot (2, 12, 0) with rot Rx(0.5):
        // a pivot-relative vertex must land at
        // R_root · (pivot + R_child · v)
        let root_r = euler_xyz([0.0, 0.0, 0.1]);
        let child_r = euler_xyz([0.5, 0.0, 0.0]);
        let pivot = [2.0f32, 12.0, 0.0];
        let v = [1.0f32, -6.0, 0.5];
        let expected = mat_vec(
            &root_r,
            &add3(&pivot, &mat_vec(&child_r, &v)),
        );
        // the emit path's stored (rot, trans) composition:
        let trans = mat_vec(&root_r, &pivot);
        let rot = mat_mul(&root_r, &child_r);
        let got = add3(&trans, &mat_vec(&rot, &v));
        for i in 0..3 {
            assert!(
                (expected[i] - got[i]).abs() < 1e-4,
                "axis {i}: {expected:?} vs {got:?}"
            );
        }
    }

    #[test]
    fn humanoid_shape_and_pose() {
        let m = humanoid(83, true);
        assert_eq!(m.parts.len(), 7, "root + body/head/2 arms/2 legs");
        assert_eq!(m.px_height, 32.0);
        // zombie arms pose forward
        assert!(m.parts[3].base_rot[0] < -1.0, "arm_r base rot forward");
        assert!(m.parts[4].base_rot[0] < -1.0, "arm_l base rot forward");
        // head sits above the body: box y 0..8 relative to neck pivot
        let head = &m.parts[2].boxes[0];
        assert_eq!(head.off[1], 0.0);
        assert_eq!(head.size, [8.0, 8.0, 8.0]);
        // named ranges exist: walk/attack/hurt/idle
        for name in ["walk", "attack", "hurt", "idle"] {
            assert!(
                m.anims.iter().any(|a| a.name == name),
                "range {name} present"
            );
        }
        // legs counter-swing at t=0.25
        let r = sample_anim(&m, "walk", 0.25);
        assert!(r[5][0] > 0.0 && r[6][0] < 0.0, "legs counter-swing");
    }

    #[test]
    fn emit_produces_six_verts_per_face_with_shading_and_uvs() {
        let m = humanoid(83, false);
        let rots = sample_anim(&m, "idle", 0.0);
        let mut out = Vec::new();
        emit_model_vertices(
            &m,
            [0.0, 64.0, 0.0],
            0.0,
            &rots,
            1.95 / 32.0,
            [1.0, 1.0, 1.0],
            [0.0, 0.0, -1.0],
            &mut out,
        );
        // 6 boxes × 6 faces × 6 verts
        assert_eq!(out.len(), 6 * 6 * 6, "vertex count");
        // all verts above ground and inside a sane radius
        assert!(out.iter().all(|v| v.pos[1] >= 64.0 && v.pos[1] <= 64.0 + 2.1));
        // top faces shaded 1.0, bottom 0.55: find the head's top and
        // bottom face verts (the brightest / darkest col)
        let max_c = out.iter().map(|v| v.col[1]).fold(f32::MIN, f32::max);
        let min_c = out.iter().map(|v| v.col[1]).fold(f32::MAX, f32::min);
        assert!((max_c - 1.0).abs() < 1e-4, "top shade present ({max_c})");
        assert!((min_c - 0.55).abs() < 1e-4, "bottom shade present ({min_c})");
        // uv within the zombie tile (tile 83: col 19, row 2 of 32)
        let u0 = (19.0 * 16.0) / 512.0;
        let u1 = (19.0 * 16.0 + 16.0) / 512.0;
        let v0 = (2.0 * 16.0) / 512.0;
        let v1 = (2.0 * 16.0 + 16.0) / 512.0;
        assert!(out
            .iter()
            .all(|v| v.uv[0] >= u0 - 1e-5 && v.uv[0] <= u1 + 1e-5 && v.uv[1] >= v0 - 1e-5 && v.uv[1] <= v1 + 1e-5));
    }

    #[test]
    fn emit_hurt_tilt_shifts_the_root() {
        let m = humanoid(83, false);
        let idle = sample_anim(&m, "idle", 0.0);
        let hurt = sample_anim(&m, "hurt", 0.5);
        let mut a = Vec::new();
        let mut b = Vec::new();
        let scale = 1.95 / 32.0;
        emit_model_vertices(&m, [0.0, 64.0, 0.0], 0.0, &idle, scale, [1.0; 3], [0.0, 0.0, -1.0], &mut a);
        emit_model_vertices(&m, [0.0, 64.0, 0.0], 0.0, &hurt, scale, [1.0; 3], [0.0, 0.0, -1.0], &mut b);
        assert_eq!(a.len(), b.len());
        // the head's top face (brightest verts) must have MOVED
        let top_a: Vec<[f32; 3]> = a
            .iter()
            .filter(|v| (v.col[1] - 1.0).abs() < 1e-4)
            .map(|v| v.pos)
            .collect();
        let top_b: Vec<[f32; 3]> = b
            .iter()
            .filter(|v| (v.col[1] - 1.0).abs() < 1e-4)
            .map(|v| v.pos)
            .collect();
        assert_eq!(top_a.len(), top_b.len());
        let moved = top_a
            .iter()
            .zip(&top_b)
            .any(|(x, y)| (x[0] - y[0]).abs() > 1e-4 || (x[2] - y[2]).abs() > 1e-4);
        assert!(moved, "hurt tilt displaces the head top face");
    }

    #[test]
    fn creeper_and_spider_rigs_build() {
        let c = creeper(84);
        assert_eq!(c.parts.len(), 7, "root + body + head + 4 legs");
        assert_eq!(c.px_height, 26.0);
        let s = spider(85);
        // root + abdomen + head + 8 legs
        assert_eq!(s.parts.len(), 11);
        // spider legs splay outward via base rz with alternating sign
        let r_splay: Vec<f32> = s.parts[3..].iter().map(|p| p.base_rot[2]).collect();
        assert!(r_splay.iter().any(|r| *r > 0.0) && r_splay.iter().any(|r| *r < 0.0));
        // walk animates all 8 legs
        let w = sample_anim(&s, "walk", 0.25);
        assert!(w[3..].iter().any(|r| r[0] != 0.0), "legs swing");
    }

    #[test]
    fn enderman_is_tall_and_thin() {
        let m = enderman(86);
        assert_eq!(m.px_height, 44.0);
        // legs are 24 px long, arms 24 px
        assert_eq!(m.parts[5].boxes[0].size[1], 24.0);
        assert_eq!(m.parts[3].boxes[0].size[1], 24.0);
        // arms hang (no forward pose)
        assert_eq!(m.parts[3].base_rot, [0.0; 3]);
    }

    #[test]
    fn emit_skips_on_part_rot_mismatch() {
        let m = humanoid(83, false);
        let mut out = Vec::new();
        emit_model_vertices(
            &m,
            [0.0; 3],
            0.0,
            &[[0.0; 3]], // WRONG length
            0.06,
            [1.0; 3],
            [0.0, 0.0, -1.0],
            &mut out,
        );
        assert!(out.is_empty(), "contract violation emits nothing");
    }
}
