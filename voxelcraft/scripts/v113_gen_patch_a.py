#!/usr/bin/env python3
"""1.13 gen.rs patch: the ocean temperature split (4 new ocean biomes),
coral reefs, kelp forests, seagrass, icebergs, buried treasure."""
import sys

p = 'crates/vc-world/src/gen.rs'
t = open(p).read()
n = 0

def rep(old, new, label):
    global t, n
    c = t.count(old)
    if c != 1:
        print(f'FAIL [{label}]: count = {c}')
        sys.exit(1)
    t = t.replace(old, new)
    n += 1
    print(f'ok [{label}]')

# ---- 1. Biome enum: the four ocean temperature families ----
rep('''    FlowerForest = 15,
    SunflowerPlains = 16,
    IceSpikes = 17,
    DarkForest = 18,
}''',
'''    FlowerForest = 15,
    SunflowerPlains = 16,
    IceSpikes = 17,
    DarkForest = 18,
    // ---- 1.13 bracket (Update Aquatic): the ocean temperature split
    // (VERIFIED live 2026-09-07, minecraft.wiki/w/Java_Edition_1.13
    // §World generation: "Added minecraft:warm_ocean (Warm Ocean),
    // minecraft:lukewarm_ocean (Lukewarm Ocean), minecraft:cold_ocean
    // (Cold Ocean) ... minecraft:frozen_ocean (Frozen Ocean) now
    // generates again"). Internal ids 19..=22 (vanilla registry ids
    // 44/45/46/10 — the deep variants are depth-cosmetic and fold into
    // these families here, disclosed). The pre-1.13 "Ocean" (id 0)
    // stays as the neutral temperate ocean. ----
    WarmOcean = 19,
    LukewarmOcean = 20,
    ColdOcean = 21,
    FrozenOcean = 22,
}''', 'Biome enum')

rep('''            Biome::FlowerForest => "Flower Forest",
            Biome::SunflowerPlains => "Sunflower Plains",
            Biome::IceSpikes => "Ice Spikes",
            Biome::DarkForest => "Dark Forest",''',
'''            Biome::FlowerForest => "Flower Forest",
            Biome::SunflowerPlains => "Sunflower Plains",
            Biome::IceSpikes => "Ice Spikes",
            Biome::DarkForest => "Dark Forest",
            Biome::WarmOcean => "Warm Ocean",
            Biome::LukewarmOcean => "Lukewarm Ocean",
            Biome::ColdOcean => "Cold Ocean",
            Biome::FrozenOcean => "Frozen Ocean",''', 'Biome names')

rep('''            17 => Biome::IceSpikes,
            18 => Biome::DarkForest,
            _ => Biome::Ocean,
        }
    }
}''',
'''            17 => Biome::IceSpikes,
            18 => Biome::DarkForest,
            19 => Biome::WarmOcean,
            20 => Biome::LukewarmOcean,
            21 => Biome::ColdOcean,
            22 => Biome::FrozenOcean,
            _ => Biome::Ocean,
        }
    }

    impl Biome {
        /// 1.13: the ocean temperature family gate (any ocean-flavored
        /// biome, including the neutral id-0 ocean).
        pub fn is_ocean(self) -> bool {
            matches!(
                self,
                Biome::Ocean
                    | Biome::WarmOcean
                    | Biome::LukewarmOcean
                    | Biome::ColdOcean
                    | Biome::FrozenOcean
            )
        }
    }
}''', 'from_u8 + is_ocean')

# ---- 2. the ocean branch: temperature split ----
rep('''        let (biome, top, filler) = if h < vc_chunk::SEA_LEVEL - 1 {
            if h < vc_chunk::SEA_LEVEL - 6 {
                (Biome::Ocean, GRAVEL, GRAVEL)
            } else {
                (Biome::Ocean, SAND, SAND)
            }
        } else if h <= vc_chunk::SEA_LEVEL + 1 {''',
'''        // 1.13 (Update Aquatic): the ocean temperature split — the
        // same temp field that picks land biomes now divides the ocean
        // into its four 1.13 families (VERIFIED changelog §World
        // generation). Floor materials: warm/lukewarm = sand (the
        // coral-reef substrate), cold/frozen/neutral-deep = gravel
        // (the wiki's ocean floor bands).
        let (biome, top, filler) = if h < vc_chunk::SEA_LEVEL - 1 {
            let deep = h < vc_chunk::SEA_LEVEL - 6;
            if temp > 0.35 {
                (Biome::WarmOcean, SAND, SAND)
            } else if temp > 0.0 {
                (Biome::LukewarmOcean, SAND, SAND)
            } else if temp > -0.25 {
                (Biome::ColdOcean, if deep { GRAVEL } else { SAND }, GRAVEL)
            } else if temp < -0.45 {
                (Biome::FrozenOcean, GRAVEL, GRAVEL)
            } else {
                // the neutral temperate ocean (the pre-1.13 "Ocean")
                (Biome::Ocean, if deep { GRAVEL } else { SAND }, GRAVEL)
            }
        } else if h <= vc_chunk::SEA_LEVEL + 1 {''', 'ocean temp split')

open(p, 'w').write(t)
print(f'GEN PATCH A DONE — {n} edits')
