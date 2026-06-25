// Software raycaster: renders the voxel world to a PNG using pure CPU.
// No GPU needed — proves the engine produces correct terrain/meshes.
//
// Run: cargo run --example render_png --release
// Output: /home/z/my-project/download/voxelcraft-*.png

use voxelcraft::blocks::Block;
use voxelcraft::world::world::World;
use voxelcraft::world::chunk::WORLD_HEIGHT;
use glam::{Vec3, Vec2};
use image::{ImageBuffer, Rgb, RgbImage};

const WIDTH: u32 = 800;
const HEIGHT: u32 = 600;
const FOV: f32 = 70.0_f32.to_radians();

fn main() {
    println!("=== VoxelCraft Software Raycaster ===");
    println!("Creating world with seed 1337...");

    let world = World::new(1337);

    // Pre-generate chunks around origin so raycast hits terrain
    println!("Generating chunks...");
    for cx in -4..=4 {
        for cz in -4..=4 {
            world.ensure_chunk(cx, cz);
        }
    }
    let (total, ready) = world.chunk_stats();
    println!("  {} chunks loaded, {} ready", total, ready);

    // Find spawn height
    let mut spawn_y = 50.0f32;
    for y in (0..WORLD_HEIGHT as i32).rev() {
        if world.get_block(0, y, 0).is_solid() {
            spawn_y = (y + 2) as f32;
            break;
        }
    }
    println!("Spawn Y: {:.1}", spawn_y);

    // Camera: positioned above terrain, looking at an angle
    let cam_pos = Vec3::new(8.0, spawn_y + 15.0, 8.0);
    let cam_yaw: f32 = 45.0_f32.to_radians();    // look toward +X+Z
    let cam_pitch: f32 = -20.0_f32.to_radians(); // look slightly down
    let cam_forward = Vec3::new(
        -cam_yaw.sin() * cam_pitch.cos(),
        cam_pitch.sin(),
        -cam_yaw.cos() * cam_pitch.cos(),
    ).normalize();
    let cam_right = Vec3::new(cam_yaw.cos(), 0.0, -cam_yaw.sin()).normalize();
    let cam_up = cam_right.cross(cam_forward).normalize();

    println!("Rendering {}x{} image...", WIDTH, HEIGHT);
    let mut img: RgbImage = ImageBuffer::new(WIDTH, HEIGHT);

    let aspect = WIDTH as f32 / HEIGHT as f32;
    let half_fov = FOV / 2.0;

    let start = std::time::Instant::now();
    let mut pixels_drawn = 0u64;
    let mut sky_pixels = 0u64;

    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            // NDC coordinates
            let ndc_x = (2.0 * (x as f32 + 0.5) / WIDTH as f32 - 1.0) * aspect * half_fov.tan();
            let ndc_y = (1.0 - 2.0 * (y as f32 + 0.5) / HEIGHT as f32) * half_fov.tan();

            // Ray direction
            let ray_dir = (cam_forward + cam_right * ndc_x + cam_up * ndc_y).normalize();

            // March the ray
            let color = match raycast_voxel(&world, cam_pos, ray_dir, 200.0) {
                Some((block, distance, _normal)) => {
                    pixels_drawn += 1;
                    // Distance fog
                    let fog = (distance / 150.0).min(1.0);
                    let base = block_color(block);
                    let sky = [0.5f32, 0.7, 1.0];
                    [
                        base[0] * (1.0 - fog) + sky[0] * fog,
                        base[1] * (1.0 - fog) + sky[1] * fog,
                        base[2] * (1.0 - fog) + sky[2] * fog,
                    ]
                }
                None => {
                    sky_pixels += 1;
                    // Sky gradient
                    let t = (ndc_y + 0.5).max(0.0).min(1.0);
                    [
                        0.5 * (1.0 - t * 0.3),
                        0.7 * (1.0 - t * 0.2),
                        1.0,
                    ]
                }
            };

            let pixel = img.get_pixel_mut(x, y);
            pixel[0] = (color[0] * 255.0).clamp(0.0, 255.0) as u8;
            pixel[1] = (color[1] * 255.0).clamp(0.0, 255.0) as u8;
            pixel[2] = (color[2] * 255.0).clamp(0.0, 255.0) as u8;
        }
        if y % 100 == 0 {
            println!("  row {} / {} ({:.0}%)", y, HEIGHT, (y as f32 / HEIGHT as f32) * 100.0);
        }
    }

    let elapsed = start.elapsed();
    println!("Render done in {:.2}s", elapsed.as_secs_f32());
    println!("  {} terrain pixels, {} sky pixels", pixels_drawn, sky_pixels);

    // Save to download folder so user can access it
    let out_path = "/tmp/opencode/voxelcraft-rust-terrain.png";
    img.save(out_path).expect("failed to save PNG");
    println!("Saved: {}", out_path);

    // Also render a top-down map view
    println!("\n=== Rendering top-down map ===");
    render_topdown_map(&world);
}

/// DDA voxel raycast — same algorithm as Player::raycast but standalone.
fn raycast_voxel(world: &World, origin: Vec3, dir: Vec3, max_dist: f32) -> Option<(Block, f32, Vec3)> {
    let mut x = origin.x.floor() as i32;
    let mut y = origin.y.floor() as i32;
    let mut z = origin.z.floor() as i32;

    let step_x = if dir.x > 0.0 { 1 } else if dir.x < 0.0 { -1 } else { 0 };
    let step_y = if dir.y > 0.0 { 1 } else if dir.y < 0.0 { -1 } else { 0 };
    let step_z = if dir.z > 0.0 { 1 } else if dir.z < 0.0 { -1 } else { 0 };

    let t_delta_x = if dir.x != 0.0 { 1.0 / dir.x.abs() } else { f32::INFINITY };
    let t_delta_y = if dir.y != 0.0 { 1.0 / dir.y.abs() } else { f32::INFINITY };
    let t_delta_z = if dir.z != 0.0 { 1.0 / dir.z.abs() } else { f32::INFINITY };

    let dist_to_edge = |o: f32, s: i32| -> f32 {
        if s > 0 { o.floor() + 1.0 - o }
        else if s < 0 { o - o.floor() }
        else { 0.0 }
    };
    let mut t_max_x = if step_x != 0 { dist_to_edge(origin.x, step_x) * t_delta_x } else { f32::INFINITY };
    let mut t_max_y = if step_y != 0 { dist_to_edge(origin.y, step_y) * t_delta_y } else { f32::INFINITY };
    let mut t_max_z = if step_z != 0 { dist_to_edge(origin.z, step_z) * t_delta_z } else { f32::INFINITY };

    let mut normal = Vec3::ZERO;
    let mut t = 0.0;

    while t <= max_dist {
        let block = world.get_block(x, y, z);
        if !block.is_air() && block != Block::Water {
            return Some((block, t, normal));
        }
        if t_max_x < t_max_y && t_max_x < t_max_z {
            x += step_x;
            t = t_max_x;
            t_max_x += t_delta_x;
            normal = Vec3::new(-step_x as f32, 0.0, 0.0);
        } else if t_max_y < t_max_z {
            y += step_y;
            t = t_max_y;
            t_max_y += t_delta_y;
            normal = Vec3::new(0.0, -step_y as f32, 0.0);
        } else {
            z += step_z;
            t = t_max_z;
            t_max_z += t_delta_z;
            normal = Vec3::new(0.0, 0.0, -step_z as f32);
        }
    }
    None
}

fn block_color(block: Block) -> [f32; 3] {
    match block {
        Block::Grass => [0.36, 0.58, 0.20],
        Block::Dirt => [0.55, 0.45, 0.33],
        Block::Stone => [0.50, 0.50, 0.50],
        Block::Cobblestone => [0.39, 0.39, 0.39],
        Block::Wood => [0.63, 0.47, 0.31],
        Block::Leaves => [0.16, 0.43, 0.12],
        Block::Sand => [0.86, 0.81, 0.64],
        Block::Water => [0.21, 0.40, 0.78],
        Block::Planks => [0.63, 0.47, 0.31],
        Block::Bedrock => [0.16, 0.16, 0.16],
        Block::Snow => [0.94, 0.96, 0.98],
        Block::Glass => [0.78, 0.86, 0.94],
        Block::Brick => [0.59, 0.24, 0.18],
        Block::CoalOre => [0.16, 0.16, 0.16],
        Block::IronOre => [0.78, 0.65, 0.51],
        Block::GoldOre => [0.94, 0.84, 0.31],
        Block::DiamondOre => [0.43, 0.90, 0.90],
        Block::Air => [0.0, 0.0, 0.0],
    }
}

/// Render a top-down heightmap of the world.
fn render_topdown_map(world: &World) {
    let map_size = 128u32; // 128x128 blocks
    let mut img: RgbImage = ImageBuffer::new(map_size, map_size);

    for px in 0..map_size {
        for pz in 0..map_size {
            let wx = px as i32 - 64;
            let wz = pz as i32 - 64;
            // Find topmost block
            let mut color = [0.5f32, 0.7, 1.0]; // sky/water default
            for y in (0..WORLD_HEIGHT as i32).rev() {
                let block = world.get_block(wx, y, wz);
                if !block.is_air() {
                    let base = block_color(block);
                    // Shade by height (higher = brighter)
                    let h = y as f32 / WORLD_HEIGHT as f32;
                    let shade = 0.6 + h * 0.4;
                    color = [base[0] * shade, base[1] * shade, base[2] * shade];
                    break;
                }
            }
            let pixel = img.get_pixel_mut(px, pz);
            pixel[0] = (color[0] * 255.0) as u8;
            pixel[1] = (color[1] * 255.0) as u8;
            pixel[2] = (color[2] * 255.0) as u8;
        }
    }

    let out_path = "/tmp/opencode/voxelcraft-rust-topdown.png";
    img.save(out_path).expect("failed to save map PNG");
    println!("Saved: {}", out_path);
}
