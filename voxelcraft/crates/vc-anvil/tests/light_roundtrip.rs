//! §28: light survives the NBT round-trip bit-identically.
//! Moved out of vc-world's light tests: `save` lives in vc-anvil and
//! vc-anvil depends on vc-world, so this integration test lives here
//! to keep the dependency graph acyclic.

use vc_blocks::blocks::*;
use vc_chunk::Chunk;
use vc_world::light::{LightData, LightSection};

#[test]
fn light_nibbles_roundtrip() {
    let mut sec = Box::new(LightSection {
        sky: Box::new([0u8; 4096]),
        blk: Box::new([0u8; 4096]),
    });
    for i in 0..4096 {
        sec.sky[i] = (i % 16) as u8;
        sec.blk[i] = ((i / 16) % 16) as u8;
    }
    let mut ld = LightData::new();
    ld.sections[4] = Some(sec);

    let mut c = Chunk::empty();
    for y in 0..16usize {
        for lz in 0..16usize {
            for lx in 0..16usize {
                c.set(lx, y, lz, STONE);
            }
        }
    }
    let bytes = vc_anvil::save::chunk_to_nbt(0, 0, &c, 1, Some(&ld));
    let (_back, light) = vc_anvil::save::chunk_from_nbt(&bytes).unwrap();
    assert!(light.is_some(), "light must survive the round-trip");
    let l2 = light.unwrap();
    let s2 = l2.sections[4].as_ref().expect("section 4 present");
    assert_eq!(&*s2.sky, &*ld.sections[4].as_ref().unwrap().sky);
    assert_eq!(&*s2.blk, &*ld.sections[4].as_ref().unwrap().blk);
}
