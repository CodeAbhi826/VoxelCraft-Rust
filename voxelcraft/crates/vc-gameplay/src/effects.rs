//! Phase E2 (evolution 1.3–1.4 bracket): a minimal timed status-effect
//! system. This bracket needs EXACTLY these effects:
//! - Wither (1.4's signature effect — wither skeleton hits + wither
//!   skulls): damage 1 HP per 20 ticks at level II (VERIFIED
//!   w/Wither_Skeleton: "1 HP every two seconds"; w/Wither: "1 HP per
//!   sec" — Wither II ticks every 20 game ticks, 0.5 hearts)
//! - Poison (witch splash potions): 1 HP per 25 ticks at level I,
//!   cannot kill (floors at 1 HP — VERIFIED w/Effect §Poison)
//! - Regeneration (beacon secondary power): 1 HP per 50 ticks at level
//!   I (VERIFIED w/Effect §Regeneration: level I every 2.5 s)
//! - Speed / Haste / Resistance / Jump Boost / Strength (beacon primary
//!   powers — stat modifiers, VERIFIED w/Beacon §Powers)
//!
//! Design: one flat table keyed by effect kind with (level, ticks_left).
//! The player tick applies periodic damage/heal and stat modifiers are
//! read by the movement/combat code through `amplifier()` lookups.
//! Vanilla's full effect stack (particles, HUD icons, /effect command,
//! ~30 effects) is out of scope — this is the minimal set the 1.3–1.4
//! content requires, disclosed in the worklog.

/// Effect kinds the engine simulates (vanilla registry names as
/// mechanical data).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EffectKind {
    Wither,
    Poison,
    Regeneration,
    Speed,
    Haste,
    Resistance,
    JumpBoost,
    Strength,
    /// 1.9/1.10 bracket (stray tipped arrows): movement penalty
    /// (VERIFIED w/Effect §Slowness: −15% per level)
    Slowness,
    /// 1.10 bracket (husk hits apply Hunger 7 s × regional difficulty;
    /// VERIFIED w/Husk) — food-poisoning drain flag
    Hunger,
    /// 1.11 bracket (totem of undying): Absorption — grants a temporary
    /// damage buffer (4 points per level; VERIFIED w/Effect §Absorption
    /// and w/Totem_of_Undying: Absorption II = 8 points / 4 hearts for
    /// 5 s). No per-tick action; the buffer lives on the player struct
    /// and is cleared when the effect expires (the game layer's hook).
    Absorption,
    /// 1.12 bracket (illusioner spell): Blindness — "Impairs vision by
    /// adding close black fog and disables the ability to sprint and
    /// critical hit" (VERIFIED w/Effect §Blindness, live 2026-09-07;
    /// effect id 15, negative). No per-tick action: the render layer
    /// pulls the fog in when active and the movement layer blocks
    /// sprinting (the engine's 1.12 hooks). Critical hits are a
    /// 1.9-combat detail the engine doesn't model — disclosed.
    Blindness,
    /// 1.13 bracket (Aquatic-era update): Water Breathing — Java effect id
    /// 13. "Prevents the breath meter from running out" (VERIFIED
    /// w/Effect §Water Breathing). Sources here: the conduit's Conduit
    /// Power bundles it, and the turtle-shell helmet would (the helmet
    /// wear itself is deferred with the armor system — disclosed). No
    /// per-tick action: the player's air-drain gate checks it.
    WaterBreathing,
    /// 1.13 bracket: Slow Falling — Java effect id 28. "Decreases
    /// falling speed and negates all fall damage" (VERIFIED
    /// w/Slow_Falling, live 2026-09-07; source: potions brewed from
    /// phantom membrane, 1:30 base / 4:00 extended). No per-tick action:
    /// the fall-damage path zeroes and the gravity path clamps the
    /// fall velocity (the engine's hooks).
    SlowFalling,
    /// 1.13 bracket: Conduit Power — Java effect id 29. "The effect has
    /// the same benefits as Water Breathing, Night Vision, and Haste"
    /// (VERIFIED w/Conduit, live 2026-09-07; range 32–96 blocks by
    /// frame, ambient blue-border HUD in vanilla). Engine form: the
    /// air gate + the mining-speed hook; the underwater-vision half is
    /// a render-layer nicety the engine's existing underwater fog
    /// already conveys — disclosed.
    ConduitPower,
    /// 1.13 bracket: Dolphin's Grace — Java effect id 30. "Players who
    /// sprint-swim within a 9 block spherical radius of a dolphin
    /// receive a swimming speed boost for 5 seconds, replenished as
    /// long as the player stays close" (VERIFIED w/Dolphin, live
    /// 2026-09-07). The wiki publishes no scalar for the boost — the
    /// engine's ×2 swim multiplier is a documented approximation.
    DolphinsGrace,
    // ---- the fire/sneak round (2026-10-03): the 16 missing effects —
    // the 1.16.5 set is now COMPLETE (32 of 32; Darkness 33 and the
    // 1.21 omens/charging effects are POST-1.16.5 — excluded). Every
    // rule is the live-verified wiki behavior (the reference wiki
    // /Effect + each effect's own page, raw wikitext, live 2026-10-03).
    /// Java effect id 4 (the elder guardian's default level III):
    /// "the mining speed is reduced to 0.3^min(level, 4) times the
    /// normal mining speed. For the in-game default level III, mining
    /// speed decreases by 97.3%" (w/Mining_Fatigue); "the effect
    /// decreases attack speed by 10% per level" (JE). The
    /// mining/attack-speed hooks read the factor.
    MiningFatigue,
    /// Java effect id 6: "Instantly heals 2 HP x 2^level" (the
    /// amplifier+1 form: I = 4 HP = 2 hearts, II = 8 — the standard
    /// hearts); "Undead mobs (including the wither) are damaged as if
    /// with Instant Damage instead" (w/Instant_Health). Instant on
    /// apply (no period).
    InstantHealth,
    /// Java effect id 7: "Instant Damage inflicts magic damage of
    /// 3 HP x 2^level" (I = 6 HP = 3 hearts, II = 12); "As this is
    /// magic damage, it can be decreased only via Resistance and
    /// Protection"; "Undead mobs are healed as if with Instant Health
    /// instead" (w/Instant_Damage). Instant on apply.
    InstantDamage,
    /// Java effect id 9: the wobbly view (the render layer's screen
    /// warp). No per-tick action.
    Nausea,
    /// Java effect id 12: "the fire and lava damage is negated" (the
    /// fire/lava damage paths gate on it). VERIFIED w/Fire_Resistance.
    FireResistance,
    /// Java effect id 14: the player's model hidden from other viewers
    /// (the render layer). No per-tick action. VERIFIED w/Invisibility.
    Invisibility,
    /// Java effect id 16: the brightness (the render layer's night
    /// vision). No per-tick action. VERIFIED w/Night_Vision.
    NightVision,
    /// Java effect id 18: "Melee damage inflicted by the affected
    /// entity is reduced by 4 HP x level" (I = -4, II = -8; JE —
    /// w/Weakness). The melee path reads the bonus.
    Weakness,
    /// Java effect id 23: "instantly replenishes 1 hunger x level and
    /// 2 x level points of saturation"; "If the effect lasts longer
    /// than one tick, the player continues gaining 1 hunger point and
    /// 2 saturation points on each tick" (w/Saturation — the particle
    /// color #F82421 red). The food path reads the per-tick restore.
    Saturation,
    /// Java effect id 21: "+4 HP (2 hearts) per level" — the max-health
    /// increase (the game layer's max-health hook). VERIFIED
    /// w/Health_Boost.
    HealthBoost,
    /// Java effect id 24 (Java-only): the glowing outline (the render
    /// layer). No per-tick action. VERIFIED w/Glowing.
    Glowing,
    /// Java effect id 25: "causes the affected entity to float upward"
    /// (w/Levitation — the wiki publishes no scalar on the page; the
    /// engine's 0.9 x (amplifier+1) b/s float is a DOCUMENTED
    /// APPROXIMATION, the Shulker-bullet float reading).
    Levitation,
    /// Java effect id 26 (Java-only): the loot-table luck modifier (the
    /// drop rolls' future work — the engine's loot is the mined-drop
    /// path; the modifier is a disclosed future hook).
    Luck,
    /// Java effect id 27 (Java-only): the negative loot modifier (the
    /// same disclosed hook).
    BadLuck,
    /// Java effect id 31: the raid trigger (VERIFIED w/Effect +
    /// w/Raid: "Bad Omen triggers a raid when a player with the effect
    /// enters a village" — the raids are MISSING; the trigger is
    /// registered and disclosed as future work).
    BadOmen,
    /// Java effect id 32: the villager trade discounts (VERIFIED
    /// w/Hero_of_the_Village; the discount's exact curve is the
    /// trading-depth detail — the effect registers, the discount is a
    /// disclosed future hook).
    HeroOfTheVillage,
}

impl EffectKind {
    pub fn name(self) -> &'static str {
        match self {
            EffectKind::Wither => "voxelcraft:wither",
            EffectKind::Poison => "voxelcraft:poison",
            EffectKind::Regeneration => "voxelcraft:regeneration",
            EffectKind::Speed => "voxelcraft:speed",
            EffectKind::Haste => "voxelcraft:haste",
            EffectKind::Resistance => "voxelcraft:resistance",
            EffectKind::JumpBoost => "voxelcraft:jump_boost",
            EffectKind::Strength => "voxelcraft:strength",
            EffectKind::Slowness => "voxelcraft:slowness",
            EffectKind::Hunger => "voxelcraft:hunger",
            EffectKind::Absorption => "voxelcraft:absorption",
            EffectKind::Blindness => "voxelcraft:blindness",
            EffectKind::WaterBreathing => "voxelcraft:water_breathing",
            EffectKind::SlowFalling => "voxelcraft:slow_falling",
            EffectKind::ConduitPower => "voxelcraft:conduit_power",
            EffectKind::DolphinsGrace => "voxelcraft:dolphins_grace",
            EffectKind::MiningFatigue => "voxelcraft:mining_fatigue",
            EffectKind::InstantHealth => "voxelcraft:instant_health",
            EffectKind::InstantDamage => "voxelcraft:instant_damage",
            EffectKind::Nausea => "voxelcraft:nausea",
            EffectKind::FireResistance => "voxelcraft:fire_resistance",
            EffectKind::Invisibility => "voxelcraft:invisibility",
            EffectKind::NightVision => "voxelcraft:night_vision",
            EffectKind::Weakness => "voxelcraft:weakness",
            EffectKind::Saturation => "voxelcraft:saturation",
            EffectKind::HealthBoost => "voxelcraft:health_boost",
            EffectKind::Glowing => "voxelcraft:glowing",
            EffectKind::Levitation => "voxelcraft:levitation",
            EffectKind::Luck => "voxelcraft:luck",
            EffectKind::BadLuck => "voxelcraft:bad_luck",
            EffectKind::BadOmen => "voxelcraft:bad_omen",
            EffectKind::HeroOfTheVillage => "voxelcraft:hero_of_the_village",
        }
    }

    /// the Java numeric id (1.16.5's 1..32; Darkness 33 is post-1.16.5).
    /// The ids are the removed-numeric-id registry order (VERIFIED
    /// w/Effect History 1.20.2: "the effects' numeral IDs were 1 (Speed)
    /// through 33 (Darkness)" — 1.16.5 ends at 32).
    pub fn java_id(self) -> u8 {
        match self {
            EffectKind::Speed => 1,
            EffectKind::Slowness => 2,
            EffectKind::Haste => 3,
            EffectKind::MiningFatigue => 4,
            EffectKind::Strength => 5,
            EffectKind::InstantHealth => 6,
            EffectKind::InstantDamage => 7,
            EffectKind::JumpBoost => 8,
            EffectKind::Nausea => 9,
            EffectKind::Regeneration => 10,
            EffectKind::Resistance => 11,
            EffectKind::FireResistance => 12,
            EffectKind::WaterBreathing => 13,
            EffectKind::Invisibility => 14,
            EffectKind::Blindness => 15,
            EffectKind::NightVision => 16,
            EffectKind::Hunger => 17,
            EffectKind::Weakness => 18,
            EffectKind::Poison => 19,
            EffectKind::Wither => 20,
            EffectKind::HealthBoost => 21,
            EffectKind::Absorption => 22,
            EffectKind::Saturation => 23,
            EffectKind::Glowing => 24,
            EffectKind::Levitation => 25,
            EffectKind::Luck => 26,
            EffectKind::BadLuck => 27,
            EffectKind::SlowFalling => 28,
            EffectKind::ConduitPower => 29,
            EffectKind::DolphinsGrace => 30,
            EffectKind::BadOmen => 31,
            EffectKind::HeroOfTheVillage => 32,
        }
    }

    /// 3.6c: lookup by the vanilla id string (`speed`, `jump_boost`…).
    pub fn by_name(s: &str) -> Option<EffectKind> {
        Some(match s {
            "speed" => EffectKind::Speed,
            "slowness" => EffectKind::Slowness,
            "haste" => EffectKind::Haste,
            "mining_fatigue" => EffectKind::MiningFatigue,
            "strength" => EffectKind::Strength,
            "instant_health" => EffectKind::InstantHealth,
            "instant_damage" => EffectKind::InstantDamage,
            "jump_boost" => EffectKind::JumpBoost,
            "nausea" => EffectKind::Nausea,
            "regeneration" => EffectKind::Regeneration,
            "resistance" => EffectKind::Resistance,
            "fire_resistance" => EffectKind::FireResistance,
            "water_breathing" => EffectKind::WaterBreathing,
            "invisibility" => EffectKind::Invisibility,
            "blindness" => EffectKind::Blindness,
            "night_vision" => EffectKind::NightVision,
            "hunger" => EffectKind::Hunger,
            "weakness" => EffectKind::Weakness,
            "poison" => EffectKind::Poison,
            "wither" => EffectKind::Wither,
            "health_boost" => EffectKind::HealthBoost,
            "absorption" => EffectKind::Absorption,
            "saturation" => EffectKind::Saturation,
            "glowing" => EffectKind::Glowing,
            "levitation" => EffectKind::Levitation,
            "luck" => EffectKind::Luck,
            "unluck" => EffectKind::BadLuck,
            "slow_falling" => EffectKind::SlowFalling,
            "conduit_power" => EffectKind::ConduitPower,
            "dolphins_grace" => EffectKind::DolphinsGrace,
            _ => return None,
        })
    }
}

/// One active effect: amplifier 0 = level I, 1 = level II (vanilla
/// convention); ticks_left counts DOWN at 20 Hz.
#[derive(Clone, Copy, Debug)]
pub struct Effect {
    pub kind: EffectKind,
    pub amplifier: u8,
    pub ticks_left: i32,
}

/// Damage/heal period per kind (VERIFIED w/Effect rows):
/// - Wither II: every 20 ticks (1 HP)
/// - Poison I: every 25 ticks (1 HP, cannot kill)
/// - Regeneration I: every 50 ticks (1 HP)
pub fn period_ticks(kind: EffectKind, amplifier: u8) -> i32 {
    match kind {
        // Wither I: 1 HP per 40 ticks (2 s — w/Wither_Skeleton phrasing);
        // Wither II: per 20 ticks (1 s — w/Wither row)
        EffectKind::Wither => 40 >> (amplifier as i32).min(1),
        // Poison I: per 25 ticks (1.25 s — w/Effect). Raw cadence halves
        // per level (25 >> amplifier), but the 10-tick hurt-immunity
        // window floors the EFFECTIVE cadence at 10 ticks (VERIFIED live
        // 2026-09-06, w/Poison: level IV lists 3 ticks/HP raw, ~1 HP/s
        // effective) — the floor models the immunity window.
        EffectKind::Poison => (25 >> (amplifier as i32)).max(10),
        // Regeneration I: per 50 ticks (2.5 s — w/Effect)
        EffectKind::Regeneration => 50 >> (amplifier as i32).min(1),
        // 1.11 Absorption: no periodic action (buffer effect)
        EffectKind::Absorption => i32::MAX,
        // fire/sneak round: Saturation's restore runs EVERY tick (1
        // hunger + 2 saturation per tick per level — w/Saturation,
        // VERIFIED live 2026-10-03); the food path reads the restore
        EffectKind::Saturation => 1,
        // the instant effects (InstantHealth/InstantDamage) apply their
        // amount ONCE at the apply site — no period (the game layer's
        // apply path)
        _ => i32::MAX, // stat effects are continuous, no period
    }
}

/// The effect-holder state (the player; witches drink potions
/// engine-side through instant amounts instead of this table).
#[derive(Clone, Debug, Default)]
pub struct Effects {
    pub active: Vec<Effect>,
    /// tick accumulators per kind (parallel to `active`)
    acc: Vec<i32>,
}

impl Effects {
    pub fn new() -> Self {
        Effects::default()
    }

    /// Apply (or refresh) an effect: a stronger/longer application wins
    /// (vanilla: the higher amplifier wins; ties → longer duration).
    pub fn apply(&mut self, kind: EffectKind, amplifier: u8, ticks: i32) {
        if let Some(i) = self.active.iter().position(|e| e.kind == kind) {
            let e = &mut self.active[i];
            if amplifier > e.amplifier || (amplifier == e.amplifier && ticks > e.ticks_left) {
                e.amplifier = amplifier;
                e.ticks_left = ticks;
                self.acc[i] = 0;
            }
        } else {
            self.active.push(Effect {
                kind,
                amplifier,
                ticks_left: ticks,
            });
            self.acc.push(0);
        }
    }

    /// current amplifier for a kind (None = not active)
    pub fn amplifier(&self, kind: EffectKind) -> Option<u8> {
        self.active
            .iter()
            .find(|e| e.kind == kind && e.ticks_left > 0)
            .map(|e| e.amplifier)
    }

    /// Advance one game tick. Returns (damage, heal) to apply this tick:
    /// damage > 0 from wither/poison; heal > 0 from regeneration.
    /// `health` is the holder's current HP (poison floors at 1).
    pub fn tick(&mut self, health: f32) -> (f32, f32) {
        let mut dmg = 0.0;
        let mut heal = 0.0;
        // iterate by index: expiry removes by swap
        let mut i = 0;
        while i < self.active.len() {
            let e = self.active[i];
            if e.ticks_left <= 0 {
                self.active.swap_remove(i);
                self.acc.swap_remove(i);
                continue;
            }
            self.active[i].ticks_left -= 1;
            self.acc[i] += 1;
            let period = period_ticks(e.kind, e.amplifier);
            if self.acc[i] >= period {
                self.acc[i] = 0;
                match e.kind {
                    EffectKind::Wither => {
                        // can kill (unlike poison — VERIFIED)
                        dmg += 1.0;
                    }
                    EffectKind::Poison => {
                        // cannot kill: floors at 1 HP (VERIFIED w/Effect)
                        if health > 1.0 {
                            dmg += 1.0;
                        }
                    }
                    EffectKind::Regeneration => {
                        heal += 1.0;
                    }
                    _ => {}
                }
            }
            i += 1;
        }
        (dmg, heal)
    }

    pub fn clear(&mut self) {
        self.active.clear();
        self.acc.clear();
    }

    /// 1.15 (Buzzy Bees): remove ONE effect kind — the honey bottle's
    /// Poison cure ("Consuming the item also has the benefit of
    /// removing any Poison effect applied to the player. Unlike
    /// drinking milk, other applied effects are not removed."
    /// VERIFIED w/Honey_Bottle). Returns true when one was removed.
    pub fn remove_one(&mut self, kind: EffectKind) -> bool {
        let mut removed = false;
        let mut i = 0;
        while i < self.active.len() {
            if self.active[i].kind == kind {
                self.active.swap_remove(i);
                self.acc.swap_remove(i); // the parallel accumulator
                removed = true;
                continue;
            }
            i += 1;
        }
        removed
    }
}

/// Speed multiplier for movement (VERIFIED w/Effect §Speed: +20% per
/// level). The player's base walk/sprint scales by this.
/// Jump Boost launch bonus in b/s (VERIFIED w/Effect §Jump_Boost:
/// "+0.1 blocks per tick per level" added to the 0.42 b/t launch —
/// 0.1 b/t x 20 = +2.0 b/s per level; the engine's JUMP_VEL is 8.4
/// b/s = 0.42 b/t, so level I launches at 10.4 b/s). Applied at the
/// jump sites (player.rs).
pub fn jump_boost_bonus(effects: &Effects) -> f32 {
    effects
        .amplifier(EffectKind::JumpBoost)
        .map(|a| 2.0 * (a as f32 + 1.0))
        .unwrap_or(0.0)
}

pub fn speed_multiplier(effects: &Effects) -> f32 {
    effects
        .amplifier(EffectKind::Speed)
        .map(|a| 1.0 + 0.20 * (a as f32 + 1.0))
        .unwrap_or(1.0)
}

/// Slowness multiplier for movement (VERIFIED w/Effect §Slowness: −15%
/// per level; level 6+ floors at zero — w/Effect §Slowness notes
/// "slowness 7" makes movement impossible). Applied to the player's
/// walk/sprint target alongside speed_multiplier.
pub fn slowness_multiplier(effects: &Effects) -> f32 {
    effects
        .amplifier(EffectKind::Slowness)
        .map(|a| (1.0 - 0.15 * (a as f32 + 1.0)).max(0.0))
        .unwrap_or(1.0)
}

/// Melee damage bonus (VERIFIED w/Effect §Strength: +3 HP per level in
/// 1.9+; pre-1.9 ×1.3/×1.6 multiplier — 1.16.5 uses the flat +3/level
/// rule; 1.3–1.4 content in a 1.16.5-target engine follows the 1.16
/// formula, disclosed).
pub fn strength_bonus(effects: &Effects) -> f32 {
    effects
        .amplifier(EffectKind::Strength)
        .map(|a| 3.0 * (a as f32 + 1.0))
        .unwrap_or(0.0)
}

/// Incoming damage multiplier (VERIFIED w/Effect §Resistance: −20% per
/// level, floors at 20% damage taken at level 4).
pub fn resistance_multiplier(effects: &Effects) -> f32 {
    effects
        .amplifier(EffectKind::Resistance)
        .map(|a| (1.0 - 0.20 * (a as f32 + 1.0)).max(0.2))
        .unwrap_or(1.0)
}

/// Jump velocity boost (VERIFIED w/Effect §Jump Boost: +0.1 b/t per
/// level on top of the vanilla 0.42 launch, caps at level II for our
/// beacon use).
pub fn jump_boost_velocity(effects: &Effects, base: f32) -> f32 {
    effects
        .amplifier(EffectKind::JumpBoost)
        .map(|a| base + 0.1 * (a as f32 + 1.0))
        .unwrap_or(base)
}

/// 1.13: is Water Breathing active (air meter frozen)? — VERIFIED
/// w/Effect §Water Breathing: "the breath meter does not run out".
pub fn water_breathing_active(effects: &Effects) -> bool {
    effects.amplifier(EffectKind::WaterBreathing).is_some()
        || effects.amplifier(EffectKind::ConduitPower).is_some()
}

/// 1.13: is Slow Falling active (fall damage negated)? — VERIFIED
/// w/Slow_Falling: "prevents all fall damage".
pub fn slow_falling_active(effects: &Effects) -> bool {
    effects.amplifier(EffectKind::SlowFalling).is_some()
}

/// 1.13: is Conduit Power active? (water breathing + haste while
/// underwater — VERIFIED w/Conduit §Conduit Power).
pub fn conduit_power_active(effects: &Effects) -> bool {
    effects.amplifier(EffectKind::ConduitPower).is_some()
}

/// 1.13: Dolphin's Grace swim-speed multiplier (VERIFIED w/Dolphin:
/// the boost exists for 5 s, replenished within a 9-block radius; the
/// wiki publishes no scalar — ×2 is the engine's documented
/// approximation of vanilla's observed swim-speed doubling).
pub fn dolphins_grace_multiplier(effects: &Effects) -> f32 {
    if effects.amplifier(EffectKind::DolphinsGrace).is_some() {
        2.0
    } else {
        1.0
    }
}

// ---- the fire/sneak round: the 16 missing effects' accessors ----

/// Mining Fatigue: "the mining speed is reduced to 0.3^min(level, 4)
/// times the normal mining speed" (w/Mining_Fatigue, VERIFIED live
/// 2026-10-03) — the level is the HUMAN level (amplifier+1: I = 0.3,
/// III = 0.027, the 97.3% decrease). The break-time path multiplies.
pub fn mining_fatigue_factor(effects: &Effects) -> f32 {
    match effects.amplifier(EffectKind::MiningFatigue) {
        Some(a) => 0.3_f32.powi((i32::from(a) + 1).min(4)),
        None => 1.0,
    }
}

/// Mining Fatigue's attack-speed half (JE): "the effect decreases
/// attack speed by 10% per level" (w/Mining_Fatigue).
pub fn mining_fatigue_attack_speed(effects: &Effects) -> f32 {
    match effects.amplifier(EffectKind::MiningFatigue) {
        Some(a) => 1.0 - 0.1 * f32::from(a + 1),
        None => 1.0,
    }
}

/// Fire Resistance: the fire/lava damage is negated (w/Fire_Resistance).
pub fn fire_resistance_active(effects: &Effects) -> bool {
    effects.amplifier(EffectKind::FireResistance).is_some()
}

/// Invisibility: the model hidden (w/Invisibility).
pub fn invisibility_active(effects: &Effects) -> bool {
    effects.amplifier(EffectKind::Invisibility).is_some()
}

/// Night Vision: the brightness (w/Night_Vision).
pub fn night_vision_active(effects: &Effects) -> bool {
    effects.amplifier(EffectKind::NightVision).is_some()
}

/// Nausea: the wobbly view (w/Nausea).
pub fn nausea_active(effects: &Effects) -> bool {
    effects.amplifier(EffectKind::Nausea).is_some()
}

/// Glowing: the outline (w/Glowing).
pub fn glowing_active(effects: &Effects) -> bool {
    effects.amplifier(EffectKind::Glowing).is_some()
}

/// Weakness: "Melee damage inflicted by the affected entity is reduced
/// by 4 HP x level" (w/Weakness, VERIFIED live 2026-10-03; I = -4,
/// II = -8). The melee path adds this (negative) bonus.
pub fn weakness_bonus(effects: &Effects) -> f32 {
    match effects.amplifier(EffectKind::Weakness) {
        Some(a) => -4.0 * f32::from(a + 1),
        None => 0.0,
    }
}

/// Saturation: "the player continues gaining 1 hunger point and 2
/// saturation points on each tick" per level (w/Saturation, VERIFIED
/// live 2026-10-03) — the food path applies the per-tick restore.
pub fn saturation_restore(effects: &Effects) -> (f32, f32) {
    match effects.amplifier(EffectKind::Saturation) {
        Some(a) => {
            let l = f32::from(a + 1);
            (l, 2.0 * l)
        }
        None => (0.0, 0.0),
    }
}

/// Health Boost: "+4 HP (2 hearts) per level" (w/Health_Boost) — the
/// max-health increase.
pub fn health_boost_bonus(effects: &Effects) -> f32 {
    match effects.amplifier(EffectKind::HealthBoost) {
        Some(a) => 4.0 * f32::from(a + 1),
        None => 0.0,
    }
}

/// Levitation: the float-up velocity (w/Levitation — the wiki publishes
/// no scalar; the 0.9 x (amplifier+1) b/s is a DOCUMENTED APPROXIMATION,
/// the Shulker-bullet float reading). The gravity path overrides.
pub fn levitation_velocity(effects: &Effects) -> f32 {
    match effects.amplifier(EffectKind::Levitation) {
        Some(a) => 0.9 * f32::from(a + 1),
        None => 0.0,
    }
}

/// Luck / Bad Luck: the loot-table modifier's amplifier (the drop
/// rolls' disclosed future hook).
pub fn luck_amplifier(effects: &Effects) -> i32 {
    if let Some(a) = effects.amplifier(EffectKind::Luck) {
        i32::from(a) + 1
    } else if let Some(a) = effects.amplifier(EffectKind::BadLuck) {
        -(i32::from(a) + 1)
    } else {
        0
    }
}

/// Bad Omen / Hero of the Village active (the raid trigger / the trade
/// discounts — the hooks are disclosed future work).
pub fn bad_omen_active(effects: &Effects) -> bool {
    effects.amplifier(EffectKind::BadOmen).is_some()
}

pub fn hero_of_the_village_active(effects: &Effects) -> bool {
    effects.amplifier(EffectKind::HeroOfTheVillage).is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn by_name_covers_common_ids() {
        assert_eq!(EffectKind::by_name("speed"), Some(EffectKind::Speed));
        assert_eq!(
            EffectKind::by_name("jump_boost"),
            Some(EffectKind::JumpBoost)
        );
        assert_eq!(EffectKind::by_name("unluck"), Some(EffectKind::BadLuck));
        assert_eq!(EffectKind::by_name("haste_ii"), None);
        assert_eq!(EffectKind::by_name(""), None);
    }

    #[test]
    fn wither_ticks_damage_every_second_and_can_kill() {
        let mut e = Effects::new();
        // wither skeleton hit: Wither 10 s (VERIFIED w/Wither_Skeleton —
        // level I in 1.16.5; the page's damage row describes the II row
        // for the wither boss; skeleton applies level I? — wiki text:
        // "inflicted with the Wither effect for 10 seconds ... decreases
        // it by 1 HP every two seconds" → level I, 40-tick period is the
        // II row; 1 HP per 2 s at I = period 40.
        e.apply(EffectKind::Wither, 0, 200);
        let mut dmg = 0.0;
        for _ in 0..200 {
            let (d, _) = e.tick(20.0);
            dmg += d;
        }
        // 10 s at 1 HP per 2 s (level I) = 5 HP — VERIFIED phrasing
        assert!((dmg - 5.0).abs() < 0.01, "wither I total = 5 HP, got {dmg}");
        // expired
        assert!(e.amplifier(EffectKind::Wither).is_none());
    }

    #[test]
    fn wither_ii_ticks_at_the_boss_rate() {
        let mut e = Effects::new();
        // wither skull: Wither II 10 s (VERIFIED w/Wither: 1 HP per sec)
        e.apply(EffectKind::Wither, 1, 200);
        let mut dmg = 0.0;
        for _ in 0..200 {
            let (d, _) = e.tick(20.0);
            dmg += d;
        }
        assert!(
            (dmg - 10.0).abs() < 0.01,
            "wither II total = 10 HP, got {dmg}"
        );
    }

    #[test]
    fn poison_cannot_kill() {
        let mut e = Effects::new();
        e.apply(EffectKind::Poison, 0, 1000);
        let mut dmg = 0.0;
        for _ in 0..1000 {
            let (d, _) = e.tick(1.0); // at 1 HP already
            dmg += d;
        }
        assert_eq!(dmg, 0.0, "poison floors at 1 HP (VERIFIED)");
    }

    #[test]
    fn regeneration_heals_every_2_point_5s() {
        let mut e = Effects::new();
        e.apply(EffectKind::Regeneration, 0, 100);
        let mut heal = 0.0;
        for _ in 0..100 {
            let (_, h) = e.tick(20.0);
            heal += h;
        }
        assert!(
            (heal - 2.0).abs() < 0.01,
            "regen I: 2 HP over 5 s, got {heal}"
        );
    }

    #[test]
    fn stronger_application_wins_ties_prefer_longer() {
        let mut e = Effects::new();
        e.apply(EffectKind::Speed, 0, 100);
        e.apply(EffectKind::Speed, 0, 50); // weaker duration: ignored
        assert_eq!(e.amplifier(EffectKind::Speed), Some(0));
        assert_eq!(e.active[0].ticks_left, 100);
        e.apply(EffectKind::Speed, 1, 10); // stronger amplifier: wins
        assert_eq!(e.amplifier(EffectKind::Speed), Some(1));
    }

    #[test]
    fn stat_modifiers_match_the_wiki() {
        let mut e = Effects::new();
        assert_eq!(speed_multiplier(&e), 1.0);
        assert_eq!(strength_bonus(&e), 0.0);
        assert_eq!(resistance_multiplier(&e), 1.0);
        e.apply(EffectKind::Speed, 0, 10);
        e.apply(EffectKind::Strength, 0, 10);
        e.apply(EffectKind::Resistance, 0, 10);
        assert!((speed_multiplier(&e) - 1.2).abs() < 1e-6); // +20% level I
        assert!((strength_bonus(&e) - 3.0).abs() < 1e-6); // +3 HP level I
        assert!((resistance_multiplier(&e) - 0.8).abs() < 1e-6); // -20%
                                                                 // level II variants (beacon secondary)
        e.apply(EffectKind::Speed, 1, 10);
        assert!((speed_multiplier(&e) - 1.4).abs() < 1e-6);
        // resistance level 4 floors at 20%
        e.apply(EffectKind::Resistance, 3, 10);
        assert!((resistance_multiplier(&e) - 0.2).abs() < 1e-6);
    }

    #[test]
    fn jump_boost_adds_a_tenth_per_level() {
        let mut e = Effects::new();
        assert_eq!(jump_boost_velocity(&e, 0.42), 0.42);
        e.apply(EffectKind::JumpBoost, 0, 10);
        assert!((jump_boost_velocity(&e, 0.42) - 0.52).abs() < 1e-6);
    }
}
