// Chunk mesher: builds vertex/index buffers from voxel data.
// Uses face culling + per-vertex AO + face shading.
// (Greedy meshing is left as a TODO — current approach is already fast enough
//  because we skip air blocks and only emit visible faces.)

use crate::blocks::{should_render_face, Block, ChunkVertex};
use crate::world::chunk::{Chunk, CHUNK_SIZE, WORLD_HEIGHT};
use crate::world::world::World;

#[derive(Debug, Default)]
pub struct ChunkMesh {
    pub solid_vertices: Vec<ChunkVertex>,
    pub solid_indices: Vec<u32>,
    pub transparent_vertices: Vec<ChunkVertex>,
    pub transparent_indices: Vec<u32>,
}

impl ChunkMesh {
    pub fn is_empty(&self) -> bool {
        self.solid_vertices.is_empty() && self.transparent_vertices.is_empty()
    }
}

pub struct MeshBuilder;

// Face definitions: 6 faces, each with 4 corner offsets + normal + which texture face to use.
struct FaceDef {
    dir: [i32; 3],
    corners: [[f32; 3]; 4],
    normal: [f32; 3],
    uv_face: usize, // 0=top, 1=bottom, 2=side
    shade: f32,     // directional lighting
}

const FACES: [FaceDef; 6] = [
    // +Y (top)
    FaceDef {
        dir: [0, 1, 0],
        corners: [[0.0, 1.0, 0.0], [1.0, 1.0, 0.0], [1.0, 1.0, 1.0], [0.0, 1.0, 1.0]],
        normal: [0.0, 1.0, 0.0],
        uv_face: 0,
        shade: 1.0,
    },
    // -Y (bottom)
    FaceDef {
        dir: [0, -1, 0],
        corners: [[0.0, 0.0, 1.0], [1.0, 0.0, 1.0], [1.0, 0.0, 0.0], [0.0, 0.0, 0.0]],
        normal: [0.0, -1.0, 0.0],
        uv_face: 1,
        shade: 0.55,
    },
    // +X (right)
    FaceDef {
        dir: [1, 0, 0],
        corners: [[1.0, 0.0, 1.0], [1.0, 1.0, 1.0], [1.0, 1.0, 0.0], [1.0, 0.0, 0.0]],
        normal: [1.0, 0.0, 0.0],
        uv_face: 2,
        shade: 0.75,
    },
    // -X (left)
    FaceDef {
        dir: [-1, 0, 0],
        corners: [[0.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 1.0, 1.0], [0.0, 0.0, 1.0]],
        normal: [-1.0, 0.0, 0.0],
        uv_face: 2,
        shade: 0.75,
    },
    // +Z (front)
    FaceDef {
        dir: [0, 0, 1],
        corners: [[0.0, 0.0, 1.0], [0.0, 1.0, 1.0], [1.0, 1.0, 1.0], [1.0, 0.0, 1.0]],
        normal: [0.0, 0.0, 1.0],
        uv_face: 2,
        shade: 0.85,
    },
    // -Z (back)
    FaceDef {
        dir: [0, 0, -1],
        corners: [[1.0, 0.0, 0.0], [1.0, 1.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 0.0]],
        normal: [0.0, 0.0, -1.0],
        uv_face: 2,
        shade: 0.85,
    },
];

const UV_CORNERS: [[f32; 2]; 4] = [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];

impl MeshBuilder {
    pub fn build(chunk: &Chunk, world: &World) -> ChunkMesh {
        let mut mesh = ChunkMesh::default();

        // Skip empty chunks
        if chunk.max_y < chunk.min_y {
            return mesh;
        }

        let base_x = chunk.cx * CHUNK_SIZE as i32;
        let base_z = chunk.cz * CHUNK_SIZE as i32;
        let y_start = chunk.min_y.saturating_sub(1);
        let y_end = (chunk.max_y + 2).min(WORLD_HEIGHT);

        for y in y_start..y_end {
            for lx in 0..CHUNK_SIZE {
                for lz in 0..CHUNK_SIZE {
                    let block = chunk.get(lx, y, lz);
                    if block.is_air() {
                        continue;
                    }

                    let props = block.properties();
                    let is_transparent = !props.opaque;
                    let wx = base_x + lx as i32;
                    let wy = y as i32;
                    let wz = base_z + lz as i32;

                    for (fi, face) in FACES.iter().enumerate() {
                        let nx = lx as i32 + face.dir[0];
                        let ny = y as i32 + face.dir[1];
                        let nz = lz as i32 + face.dir[2];

                        let neighbor = world.get_block(wx + face.dir[0], ny, wz + face.dir[2]);
                        if !should_render_face(block, neighbor) {
                            continue;
                        }

                        let tile = block.textures()[face.uv_face] as u32;
                        let ao = compute_ao(world, wx, wy, wz, fi);

                        let target = if is_transparent {
                            &mut mesh.transparent_vertices
                        } else {
                            &mut mesh.solid_vertices
                        };
                        let target_idx = if is_transparent {
                            &mut mesh.transparent_indices
                        } else {
                            &mut mesh.solid_indices
                        };

                        let base_idx = (target.len() as u32).try_into().unwrap_or(0);
                        let base_v = target.len() as u32;
                        for c in 0..4 {
                            let vx = base_x as f32 + lx as f32 + face.corners[c][0];
                            let vy = y as f32 + face.corners[c][1];
                            let vz = base_z as f32 + lz as f32 + face.corners[c][2];
                            let tint = face.shade * ao[c];
                            target.push(ChunkVertex {
                                position: [vx, vy, vz],
                                normal: face.normal,
                                uv: UV_CORNERS[c],
                                color: [tint, tint, tint],
                                tex_layer: tile,
                            });
                        }
                        target_idx.extend_from_slice(&[
                            base_v, base_v + 1, base_v + 2,
                            base_v, base_v + 2, base_v + 3,
                        ]);
                    }
                }
            }
        }

        mesh
    }
}

/// Compute ambient occlusion for the 4 corners of a face.
/// Returns [ao0, ao1, ao2, ao3] in 0..=1 range (1 = fully lit).
fn compute_ao(world: &World, wx: i32, wy: i32, wz: i32, face_idx: usize) -> [f32; 4] {
    let face = &FACES[face_idx];
    let [dx, dy, dz] = face.dir;

    // Two tangent axes for this face
    let (t1, t2) = if dy != 0 {
        ([1, 0, 0], [0, 0, 1])
    } else if dx != 0 {
        ([0, 1, 0], [0, 0, 1])
    } else {
        ([1, 0, 0], [0, 1, 0])
    };

    // For each of the 4 corners, sample 3 neighbors: side1, side2, corner
    let mut ao = [1.0f32; 4];
    let corner_signs = [[-1, -1], [1, -1], [1, 1], [-1, 1]];

    for (c, signs) in corner_signs.iter().enumerate() {
        let s1 = [t1[0] * signs[0], t1[1] * signs[0], t1[2] * signs[0]];
        let s2 = [t2[0] * signs[1], t2[1] * signs[1], t2[2] * signs[1]];
        let diag = [s1[0] + s2[0], s1[1] + s2[1], s1[2] + s2[2]];

        let nb1 = world.get_block(wx + dx + s1[0], wy + dy + s1[1], wz + dz + s1[2]);
        let nb2 = world.get_block(wx + dx + s2[0], wy + dy + s2[1], wz + dz + s2[2]);
        let nbd = world.get_block(wx + dx + diag[0], wy + dy + diag[1], wz + dz + diag[2]);

        let mut solid_count = 0;
        if !nb1.is_air() && nb1.is_opaque() { solid_count += 1; }
        if !nb2.is_air() && nb2.is_opaque() { solid_count += 1; }
        if !nbd.is_air() && nbd.is_opaque() { solid_count += 1; }

        ao[c] = match solid_count {
            0 => 1.0,
            1 => 0.8,
            2 => 0.6,
            _ => 0.45,
        };
    }

    ao
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::world::World;

    #[test]
    fn empty_chunk_produces_empty_mesh() {
        let world = World::new(1337);
        // Create an empty chunk directly (no terrain) and insert it into the world
        let chunk = Chunk::new(999, 999);
        let mesh = MeshBuilder::build(&chunk, &world);
        assert!(mesh.is_empty());
    }

    #[test]
    fn single_block_produces_12_triangles() {
        let world = World::new(1337);
        world.set_block(8, 32, 8, Block::Stone);
        let chunk = world.get_chunk(0, 0).expect("chunk should exist");
        let chunk = chunk.read();
        let mesh = MeshBuilder::build(&chunk, &world);
        // A single stone block surrounded by air should have 6 faces * 2 triangles = 12 triangles
        // = 36 indices
        assert_eq!(mesh.solid_indices.len(), 36, "expected 36 indices, got {}", mesh.solid_indices.len());
    }

    #[test]
    fn two_adjacent_blocks_cull_shared_face() {
        let world = World::new(1337);
        // Place two adjacent stone blocks via the world (so neighbor lookup works)
        world.set_block(8, 32, 8, Block::Stone);
        world.set_block(9, 32, 8, Block::Stone);
        let chunk = world.get_chunk(0, 0).expect("chunk should exist");
        let chunk = chunk.read();
        let mesh = MeshBuilder::build(&chunk, &world);
        // Two adjacent blocks: 12 faces - 2 culled (shared +X/-X) = 10 faces = 60 indices
        assert_eq!(mesh.solid_indices.len(), 60, "expected 60 indices, got {}", mesh.solid_indices.len());
    }
}
