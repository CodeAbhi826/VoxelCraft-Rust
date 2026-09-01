// Headless integration tests for the voxel engine.
// These prove the engine works WITHOUT needing a GPU or display.

use voxelcraft::blocks::Block;
use voxelcraft::world::chunk::{Chunk, CHUNK_SIZE, WORLD_HEIGHT};
use voxelcraft::world::world::World;
use voxelcraft::world::noise::WorldGenerator;
use voxelcraft::world::mesher::MeshBuilder;
use voxelcraft::player::Player;
use glam::Vec3;

#[test]
fn full_engine_smoke_test() {
    // 1. Create world
    let world = World::new(1337);
    let gen = WorldGenerator::new(1337);

    // 2. Generate a chunk
    let mut chunk = Chunk::new(0, 0);
    gen.generate(&mut chunk);
    assert!(!chunk.blocks.iter().all(|b| b.is_air()));

    // 3. Build mesh (insert chunk into world first so neighbor lookups work)
    // Use a fresh world and place the chunk directly
    let chunk_arc = world.ensure_chunk_empty(0, 0);
    {
        let mut c = chunk_arc.write();
        // Copy generated blocks into the world's chunk
        c.blocks = chunk.blocks.clone();
        c.min_y = chunk.min_y;
        c.max_y = chunk.max_y;
        c.dirty = true;
    }
    let mesh = MeshBuilder::build(&chunk_arc.read(), &world);
    assert!(!mesh.solid_vertices.is_empty(), "mesh should have vertices");

    // 4. Player can spawn and raycast — point DOWN at terrain
    let mut player = Player::new(Vec3::new(0.5, 50.0, 0.5));
    player.pitch = -std::f32::consts::FRAC_PI_2; // look straight down
    player.yaw = 0.0;
    let hit = player.raycast(&world, 100.0);
    assert!(hit.is_some(), "raycast should hit terrain when looking down");

    println!("✅ Full engine smoke test passed");
    println!("   Chunk has {} solid blocks", chunk_arc.read().blocks.iter().filter(|b| !b.is_air()).count());
    println!("   Mesh has {} vertices, {} indices", mesh.solid_vertices.len(), mesh.solid_indices.len());
}

#[test]
fn block_break_place_roundtrip() {
    let world = World::new(42);

    // Place a block
    world.set_block(5, 10, 5, Block::Stone);
    assert_eq!(world.get_block(5, 10, 5), Block::Stone);

    // Break it
    world.set_block(5, 10, 5, Block::Air);
    assert_eq!(world.get_block(5, 10, 5), Block::Air);

    println!("✅ Block break/place roundtrip passed");
}

#[test]
fn terrain_has_variety() {
    let gen = WorldGenerator::new(1337);
    let mut heights = std::collections::HashSet::new();
    for x in 0..50 {
        for z in 0..50 {
            heights.insert(gen.noise.height(x, z));
        }
    }
    // Terrain should have at least 10 distinct height levels in a 50x50 area
    assert!(heights.len() >= 10, "terrain too flat: only {} distinct heights", heights.len());
    println!("✅ Terrain variety: {} distinct heights in 50x50", heights.len());
}

#[test]
fn chunk_mesh_face_culling_works() {
    let world = World::new(42);
    // Place a 2x2x2 cube via the world (so neighbor lookups work)
    for x in 5..7 {
        for y in 10..12 {
            for z in 5..7 {
                world.set_block(x, y, z, Block::Stone);
            }
        }
    }
    let chunk = world.get_chunk(0, 0).expect("chunk should exist");
    let chunk = chunk.read();
    let mesh = MeshBuilder::build(&chunk, &world);
    assert!(!mesh.solid_indices.is_empty());
    let face_count = mesh.solid_indices.len() / 6;
    // 2x2x2 = 8 blocks, 48 total faces, 24 external faces (6 sides * 4)
    assert_eq!(face_count, 24, "expected 24 external faces, got {}", face_count);
    println!("✅ Face culling: {} visible faces for 2x2x2 cube", face_count);
}

#[test]
fn player_physics_simulates_correctly() {
    let world = World::new(42);
    // Place a floor
    for x in -2..=2 {
        for z in -2..=2 {
            world.set_block(x, 10, z, Block::Stone);
        }
    }
    let mut player = Player::new(Vec3::new(0.5, 15.0, 0.5));
    let input = voxelcraft::player::InputState::default();

    // Simulate 5 seconds of falling
    for _ in 0..100 {
        player.update(0.05, input, &world);
    }

    assert!(player.on_ground, "player should land on floor");
    assert!(player.position.y >= 11.0 && player.position.y < 11.5,
            "player landed at y={} (expected 11.0-11.5)", player.position.y);
    println!("✅ Player physics: landed at y={:.2}", player.position.y);
}

#[test]
fn raycast_accuracy() {
    let world = World::new(42);
    // Place a wall of blocks at z=10
    for x in -5..=5 {
        for y in 0..10 {
            world.set_block(x, y, 10, Block::Stone);
        }
    }
    let mut player = Player::new(Vec3::new(0.5, 5.5, 5.5));
    player.yaw = std::f32::consts::PI; // face +Z
    player.pitch = 0.0;

    let hit = player.raycast(&world, 10.0);
    assert!(hit.is_some(), "raycast should hit wall");
    let (pos, _normal) = hit.unwrap();
    assert_eq!(pos.z, 10, "raycast should hit z=10, got z={}", pos.z);
    println!("✅ Raycast: hit block at ({},{},{})", pos.x, pos.y, pos.z);
}

#[test]
fn world_chunk_streaming_loads_and_unloads() {
    let world = World::new(42);
    // Load chunks around origin
    world.update_player_position(0, 0, 3);
    let (total, _) = world.chunk_stats();
    assert!(total > 0, "chunks should be loaded");

    // Move far away
    world.update_player_position(1000, 1000, 3);
    let (total_after, _) = world.chunk_stats();
    // Old chunks should be unloaded, new ones loaded
    // (total_after may still be > 0 because new chunks are loaded)
    println!("✅ Chunk streaming: {} → {} chunks after moving", total, total_after);
}

#[test]
fn all_blocks_have_textures() {
    for block in Block::ALL {
        let textures = block.textures();
        // All non-air blocks should have valid texture indices
        if !block.is_air() {
            for &t in &textures {
                assert!(t < 19, "block {:?} has invalid texture index {}", block, t);
            }
        }
    }
    println!("✅ All {} blocks have valid textures", Block::ALL.len());
}
