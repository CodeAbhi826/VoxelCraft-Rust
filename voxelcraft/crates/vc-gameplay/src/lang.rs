//! 3.4a: lang keys — `block.voxelcraft.<snake>` /
//! `item.voxelcraft.<snake>` for every registry entry, derived from the
//! canonical BLOCK_TABLE names (solid → block key, else item key).
//! Lookup falls back to the key itself (never panics, never empty).
//! This is the key REGISTRY + lookup only; UI strings adopt keys
//! gradually after this slice (the UI is hardcoded English today).

/// lang key for a registry id.
pub fn key_for(id: u16) -> String {
    let def = vc_blocks::blocks::def(id);
    let mut snake = String::with_capacity(def.name.len());
    for c in def.name.chars() {
        if c.is_ascii_alphanumeric() {
            snake.push(c.to_ascii_lowercase());
        } else if c == ' ' || c == '-' {
            snake.push('_');
        }
        // apostrophes and other punctuation are dropped
        // ("Dragon's Breath" -> dragons_breath)
    }
    let cat = if def.solid { "block" } else { "item" };
    format!("{cat}.voxelcraft.{snake}")
}

/// English text for a key (reverse lookup); unknown keys echo back.
pub fn text(key: &str) -> String {
    for id in 0..vc_blocks::blocks::BLOCK_COUNT as u16 {
        if key_for(id) == key {
            return vc_blocks::blocks::BLOCK_TABLE[id as usize].name.to_string();
        }
    }
    key.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_registry_entry_roundtrips() {
        for id in 0..vc_blocks::blocks::BLOCK_COUNT as u16 {
            let k = key_for(id);
            assert!(k.starts_with("block.voxelcraft.") || k.starts_with("item.voxelcraft."));
            assert_eq!(text(&k), vc_blocks::blocks::BLOCK_TABLE[id as usize].name);
        }
    }

    #[test]
    fn spot_keys_look_vanilla_shaped() {
        assert_eq!(key_for(vc_blocks::blocks::STONE), "block.voxelcraft.stone");
        assert_eq!(
            key_for(vc_blocks::blocks::IRON_SWORD),
            "item.voxelcraft.iron_sword"
        );
        assert_eq!(key_for(vc_blocks::blocks::BOW), "item.voxelcraft.bow");
    }

    #[test]
    fn unknown_keys_echo_back() {
        assert_eq!(text("nope.nope.nope"), "nope.nope.nope");
        assert_eq!(text(""), "");
    }
}
