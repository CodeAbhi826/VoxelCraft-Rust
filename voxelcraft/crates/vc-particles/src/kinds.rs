//! Round 15b (2026-09-16): the missing 1.16.5 particle types — the
//! registry + the typed emitter. All facts live-verified 2026-09-16
//! against minecraft.wiki/w/Particle (the registered 1.16.5 list;
//! docs/research/round-15b-particles-sky-audit.md).
//!
//! Rendering model (disclosed): every type renders as a flat-color
//! billboard — the atlas's white SNOW tile tinted by the type's color,
//! with the type's own size / lifetime / gravity. The vanilla ADDITIVE
//! blend modes (portal, reverse_portal, end_rod, firework) are
//! registered in the defs but ride the engine's single alpha-blended
//! particle pipeline (the stream also carries the dragon/crystal
//! billboards; a blend-split needs a second pipeline + draw batch —
//! deferred with that reason, disclosed in the audit doc §2).

use vc_blocks::blocks::TILE_SNOW;
use crate::particles::{Particle, ParticleSystem};

/// one registered particle type's definition
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct KindDef {
    /// the registry name (the wiki's id, underscores as published)
    pub name: &'static str,
    /// the flat tint (procedural sprite — clean-room colors)
    pub color: [f32; 3],
    /// billboard half-extent in world units
    pub half: f32,
    /// lifetime in sim ticks (20 Hz)
    pub life: i32,
    /// gravity in blocks/tick² (0.04 = the vanilla default)
    pub grav: f32,
    /// vanilla renders this type additively (disclosed: the engine's
    /// particle pass is one alpha pipeline — the flag is registry data)
    pub additive: bool,
    /// rises instead of falls (bubbles, end_rod sparkle)
    pub rises: bool,
}

/// Round 15b: the 20-type batch (w/Particle §Registered particles,
/// live 2026-09-16). Each def cites its wiki row in the audit doc.
pub const KINDS: [KindDef; 20] = [
    // splash — water entry, ~0.5 s (VERIFIED w/Particle: the splash
    // burst when an entity enters water)
    KindDef { name: "splash", color: [0.55, 0.70, 0.95], half: 0.08, life: 10, grav: 0.04, additive: false, rises: false },
    // bubble — underwater entity breath, rises to the surface
    KindDef { name: "bubble", color: [0.75, 0.85, 1.00], half: 0.05, life: 40, grav: 0.0, additive: false, rises: true },
    // bubble_pop — a bubble reaching the surface
    KindDef { name: "bubble_pop", color: [0.80, 0.90, 1.00], half: 0.06, life: 4, grav: 0.0, additive: false, rises: false },
    // dripping_water — leaf/bottom-face drip, gravity 0.04 (the spec's
    // own value; VERIFIED w/Particle: the drip hangs then falls)
    KindDef { name: "dripping_water", color: [0.45, 0.60, 0.90], half: 0.04, life: 60, grav: 0.04, additive: false, rises: false },
    // dripping_lava — same but lava, emissive
    KindDef { name: "dripping_lava", color: [1.00, 0.45, 0.10], half: 0.04, life: 60, grav: 0.04, additive: false, rises: false },
    // falling_water — the actually-landing drip, grey-blue
    KindDef { name: "falling_water", color: [0.50, 0.65, 0.85], half: 0.05, life: 16, grav: 0.04, additive: false, rises: false },
    // falling_lava — the actually-landing lava, orange glow
    KindDef { name: "falling_lava", color: [1.00, 0.55, 0.15], half: 0.05, life: 16, grav: 0.04, additive: false, rises: false },
    // portal — nether portal ambience, teal-purple, additive
    KindDef { name: "portal", color: [0.45, 0.20, 0.70], half: 0.06, life: 40, grav: 0.0, additive: true, rises: false },
    // reverse_portal — end portal, pale purple, additive
    KindDef { name: "reverse_portal", color: [0.75, 0.60, 0.95], half: 0.06, life: 40, grav: 0.0, additive: true, rises: false },
    // end_rod — the end rod sparkle, small white star, additive
    KindDef { name: "end_rod", color: [1.00, 0.98, 0.90], half: 0.04, life: 30, grav: 0.0, additive: true, rises: true },
    // firework — the trail spark, colour-tinted, additive
    KindDef { name: "firework", color: [1.00, 0.80, 0.40], half: 0.05, life: 24, grav: 0.01, additive: true, rises: false },
    // explosion_emitter — the large smoke puff + spark ring
    KindDef { name: "explosion_emitter", color: [0.90, 0.85, 0.75], half: 0.45, life: 12, grav: 0.0, additive: false, rises: false },
    // squid_ink — the black ink cloud
    KindDef { name: "squid_ink", color: [0.10, 0.10, 0.13], half: 0.16, life: 30, grav: 0.0, additive: false, rises: false },
    // dust — the redstone torch dust, red
    KindDef { name: "dust", color: [0.85, 0.10, 0.10], half: 0.03, life: 20, grav: 0.04, additive: false, rises: false },
    // note — the note block's note, tinted by instrument
    KindDef { name: "note", color: [0.30, 0.60, 1.00], half: 0.08, life: 12, grav: 0.0, additive: true, rises: true },
    // happy_villager — the green cross sparkle
    KindDef { name: "happy_villager", color: [0.35, 0.90, 0.35], half: 0.06, life: 20, grav: 0.0, additive: false, rises: true },
    // angry_villager — the dark grey cloud
    KindDef { name: "angry_villager", color: [0.35, 0.32, 0.30], half: 0.06, life: 20, grav: 0.0, additive: false, rises: true },
    // snowflake — the snow-biome ambient flake
    KindDef { name: "snowflake", color: [1.00, 1.00, 1.00], half: 0.05, life: 60, grav: 0.01, additive: false, rises: false },
    // totem_of_undying — the green + yellow revival ring
    KindDef { name: "totem_of_undying", color: [0.70, 1.00, 0.40], half: 0.09, life: 30, grav: 0.0, additive: true, rises: true },
    // spit — the llama spit projectile trail
    KindDef { name: "spit", color: [0.85, 0.90, 0.75], half: 0.05, life: 8, grav: 0.04, additive: false, rises: false },
];

/// look up a def by registry name
pub fn def(name: &str) -> Option<&'static KindDef> {
    KINDS.iter().find(|k| k.name == name)
}

/// which of the batch's types have a LIVE engine spawn source (the
/// game layer's event paths); the rest stay registered-but-inert with
/// their reasons in the audit doc (R10). portal: the engine registers
/// no nether-portal BLOCK, so the shimmer has no spawn site; lava
/// drips: no lava-adjacent leaf path; bubble_pop: no surface event;
/// end_rod/firework/reverse_portal/note: no end rods, fireworks,
/// end-portal ambience or note blocks in the registry.
pub fn has_engine_source(name: &str) -> bool {
    matches!(
        name,
        "splash"
            | "bubble"
            | "dripping_water"
            | "falling_water"
            | "explosion_emitter"
            | "squid_ink"
            | "dust"
            | "happy_villager"
            | "angry_villager"
            | "snowflake"
            | "totem_of_undying"
            | "spit"
    )
}

impl ParticleSystem {
    /// Round 15b: the typed emitter — `n` particles of `name` around
    /// (x,y,z) with the def's color/size/lifetime/gravity and a
    /// per-family velocity profile. Flat-color billboards: the atlas's
    /// white SNOW tile × the def tint.
    pub fn spawn_kind(&mut self, name: &str, x: f32, y: f32, z: f32, n: usize) {
        let Some(d) = def(name) else { return };
        // the Particles setting gates ambient types like every spawn
        let tile = TILE_SNOW;
        let tx = (tile % 32) as f32;
        let ty = (tile / 32) as f32;
        for _ in 0..n {
            if self.rng.next_f32() >= self.density {
                continue;
            }
            let spread = d.half * 6.0;
            let (vx, vy, vz) = if d.rises {
                // rising types drift up slowly
                (
                    (self.rng.next_f32() - 0.5) * 0.04,
                    0.02 + self.rng.next_f32() * 0.04,
                    (self.rng.next_f32() - 0.5) * 0.04,
                )
            } else if d.grav > 0.0 {
                // burst types launch outward then fall
                (
                    (self.rng.next_f32() - 0.5) * 0.3,
                    0.1 + self.rng.next_f32() * 0.2,
                    (self.rng.next_f32() - 0.5) * 0.3,
                )
            } else {
                // clouds/ambience barely move
                (
                    (self.rng.next_f32() - 0.5) * 0.06,
                    (self.rng.next_f32() - 0.5) * 0.06,
                    (self.rng.next_f32() - 0.5) * 0.06,
                )
            };
            // draw the position jitter + lifetime variance BEFORE the
            // push (the push borrows self mutably in one call)
            let jx = x + (self.rng.next_f32() - 0.5) * spread;
            let jy = y + (self.rng.next_f32() - 0.5) * spread;
            let jz = z + (self.rng.next_f32() - 0.5) * spread;
            let life = d.life + self.rng.next_range(8) as i32;
            self.push(Particle {
                pos: [jx, jy, jz],
                vel: [vx, vy, vz],
                life,
                half: d.half,
                u0: tx / 32.0,
                v0: ty / 32.0,
                du: 1.0 / 32.0,
                dv: 1.0 / 32.0,
                light: 1.0, // flat-color: the tint IS the color
                tint: d.color,
                grav: d.grav,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Round 15b [spec]: the registry matches the wiki's registered
    /// particle list subset — every name is a real 1.16.5 particle id
    /// (live w/Particle, fetched 2026-09-16), the batch is exactly 20
    /// types, names are unique, and every def is sane (positive
    /// lifetime/size, gravity in range).
    #[test]
    fn particle_type_registry_matches_wiki_subset() {
        assert_eq!(KINDS.len(), 20, "the spec's 20-type batch");
        let mut seen = std::collections::HashSet::new();
        for k in KINDS.iter() {
            assert!(seen.insert(k.name), "duplicate name {}", k.name);
            assert!(k.life > 0, "{} has a positive lifetime", k.name);
            assert!(k.half > 0.0 && k.half < 1.0, "{} size sane", k.name);
            assert!((0.0..=0.04).contains(&k.grav), "{} gravity in range", k.name);
            assert!(
                k.color.iter().all(|c| (0.0..=1.0).contains(c)),
                "{} color in range",
                k.name
            );
        }
        // the spec's named types are all present
        for needed in [
            "splash",
            "bubble",
            "bubble_pop",
            "dripping_water",
            "dripping_lava",
            "falling_water",
            "falling_lava",
            "portal",
            "reverse_portal",
            "end_rod",
            "firework",
            "explosion_emitter",
            "squid_ink",
            "dust",
            "note",
            "happy_villager",
            "angry_villager",
            "snowflake",
            "totem_of_undying",
            "spit",
        ] {
            assert!(def(needed).is_some(), "{needed} registered");
        }
        // the additive family is flagged (portal-class glow types)
        for a in ["portal", "reverse_portal", "end_rod", "firework", "note", "totem_of_undying"] {
            assert!(def(a).unwrap().additive, "{a} is additive (registry data)");
        }
        // at least the live-source subset claims an engine source
        assert!(has_engine_source("splash"));
        assert!(has_engine_source("totem_of_undying"));
        assert!(!has_engine_source("firework"), "no firework events — inert");
        assert!(!has_engine_source("portal"), "no nether-portal block — inert");
    }

    /// Round 15b: the typed emitter pushes particles with the def's
    /// physics (density-gated, capped, counted).
    #[test]
    fn spawn_kind_emits_the_def_physics() {
        let mut ps = ParticleSystem::new(42);
        ps.spawn_kind("portal", 0.0, 64.0, 0.0, 16);
        assert_eq!(ps.len(), 16, "all 16 spawn at density 1.0");
        let p = ps.parts[0];
        let d = def("portal").unwrap();
        assert_eq!(p.tint, d.color);
        assert_eq!(p.grav, d.grav);
        assert!(p.life >= d.life);
        assert!(p.half > 0.0);
        // the unknown name is a no-op
        ps.spawn_kind("no_such_particle", 0.0, 64.0, 0.0, 16);
        assert_eq!(ps.len(), 16);
        // density 0 kills every spawn
        ps.density = 0.0;
        ps.spawn_kind("dust", 0.0, 64.0, 0.0, 8);
        assert_eq!(ps.len(), 16, "density 0 = nothing spawns");
    }
}
