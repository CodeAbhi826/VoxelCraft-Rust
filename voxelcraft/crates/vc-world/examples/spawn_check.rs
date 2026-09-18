use vc_world::world::World;

fn main() {
    let seed: u64 = 12345;
    let mut w = World::new(seed);
    let (x, y, z) = w.find_spawn();
    println!("seed {seed} spawn: ({x:.2}, {y:.2}, {z:.2})");
    // generate the 3x3 chunks around spawn like the game does
    let cx = (x.floor() as i32).div_euclid(16);
    let cz = (z.floor() as i32).div_euclid(16);
    for dz in -1i32..=1 {
        for dx in -1i32..=1 {
            let (chunk, out) = w.gen.generate_chunk(cx + dx, cz + dz, Vec::new());
            w.insert_generated((cx + dx, cz + dz), chunk, out);
        }
    }
    w.dirty.clear();
    let bx = x.floor() as i32;
    let by = y.floor() as i32;
    let bz = z.floor() as i32;
    for dy in -4..=2 {
        let b = w.get_block(bx, by + dy, bz);
        let bside = w.get_block(bx + 1, by + dy, bz);
        println!("  y{dy:+>3}: feet-col={b:?}  +1x-col={bside:?}");
    }
    println!("feet cell block: {:?}", w.get_block(bx, by, bz));
    println!("block below feet: {:?}", w.get_block(bx, by - 1, bz));
    let mut surface = None;
    for yy in (0..=255i32).rev() {
        let b = w.get_block(bx, yy, bz);
        if b != vc_blocks::blocks::AIR {
            surface = Some((yy, b));
            break;
        }
    }
    println!("top non-air in feet column: {surface:?}");
}
