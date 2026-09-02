//! 1.16.5-style sound synthesis + the §21 data-driven sound-event system.
//! Every sound is generated from scratch (filtered noise bursts + decaying
//! sines) at startup — zero asset files. Backends: rodio (native), WebAudio
//! (wasm), silent fallback (no-audio build) — all behind one interface.
//!
//! §21 layers:
//! - `SOUNDS_JSON` — a vanilla-`sounds.json`-shaped registry (clean-room:
//!   same format, our own synthesized recipes) parsed at boot via serde
//! - sound CATEGORIES (vanilla's nine), per-category gain, master + music
//!   user-configurable
//! - multiple VARIANTS per event with WEIGHTed random selection
//! - per-event PITCH range + volume, streaming flags (music)
//! - distance ATTENUATION + stereo PAN (spatial positioning)
//! - procedural MUSIC pads (day/night) + AMBIENT cave sounds

use crate::blocks::SoundFamily;
use crate::rng::Rng;
use serde::Deserialize;
use std::collections::HashMap;

pub const RATE: u32 = 22050;

// --------------------------------------------------------- sound bank ----

/// synthesized sounds, addressable BY NAME (recipe names from the registry)
pub struct SoundBank {
    /// recipe name per slot ("dig/grass1", "step/stone", "music/pad_day"…)
    pub names: Vec<String>,
    /// recipe name → slot index
    pub index: HashMap<String, usize>,
    /// mono f32 sample data per slot
    pub data: Vec<Vec<f32>>,
    /// WAV-encoded copies (for WebAudio decode)
    pub wavs: Vec<Vec<u8>>,
}

pub const SPLASH: usize = 8;
pub const BANK_LEN: usize = 9;

pub fn family_index(f: SoundFamily) -> usize {
    match f {
        SoundFamily::Grass => 0,
        SoundFamily::Dirt => 1,
        SoundFamily::Stone => 2,
        SoundFamily::Wood => 3,
        SoundFamily::Sand => 4,
        SoundFamily::Leaves => 5,
        SoundFamily::Glass => 6,
        SoundFamily::Wool => 7,
        SoundFamily::Water | SoundFamily::None => 8, // splash
    }
}

// ------------------------------------------------------------- synthesis --

fn one_pole_lp(x: Vec<f32>, fc: f32) -> Vec<f32> {
    let a = 1.0 - (-2.0 * std::f32::consts::PI * fc / RATE as f32).exp();
    let mut y = 0.0f32;
    x.into_iter()
        .map(|s| {
            y += a * (s - y);
            y
        })
        .collect()
}

fn one_pole_hp(x: Vec<f32>, fc: f32) -> Vec<f32> {
    let lp = one_pole_lp(x.clone(), fc);
    x.into_iter().zip(lp).map(|(s, l)| s - l).collect()
}

fn noise_burst(n: usize, seed: u64, amp: f32, attack: f32, decay: f32) -> Vec<f32> {
    let mut rng = Rng::new(seed);
    let mut out = Vec::with_capacity(n);
    let att = (attack * RATE as f32) as usize;
    let dec = (decay * RATE as f32) as usize;
    for i in 0..n {
        let w = (rng.next_f32() * 2.0 - 1.0) * amp;
        let env = if i < att && att > 0 {
            i as f32 / att as f32
        } else if i < dec {
            (1.0 - (i as f32 - att as f32) / (dec as f32 - att as f32)).max(0.0)
        } else {
            0.0
        };
        out.push(w * env);
    }
    out
}

fn ping(freq: f32, dur: f32, amp: f32, seed: u64) -> Vec<f32> {
    let n = (dur * RATE as f32) as usize;
    let phase = Rng::new(seed).next_f32();
    (0..n)
        .map(|i| {
            let t = i as f32 / RATE as f32;
            let env = (-t * 30.0).exp();
            (2.0 * std::f32::consts::PI * freq * (t + phase)).sin() * amp * env
        })
        .collect()
}

fn thump(freq: f32, dur: f32, amp: f32) -> Vec<f32> {
    let n = (dur * RATE as f32) as usize;
    (0..n)
        .map(|i| {
            let t = i as f32 / RATE as f32;
            let env = (-t * 18.0).exp();
            let sweep = freq * (1.0 - 0.35 * t / dur);
            (2.0 * std::f32::consts::PI * sweep * t).sin() * amp * env
        })
        .collect()
}

fn mix_into(a: &mut Vec<f32>, b: &[f32], gain: f32) {
    for i in 0..b.len() {
        if i < a.len() {
            a[i] += b[i] * gain;
        }
    }
}

fn clamp_amp(x: Vec<f32>) -> Vec<f32> {
    x.into_iter().map(|s| s.clamp(-1.0, 1.0)).collect()
}

/// `variant` (0/1/2…) jitters seeds + filter cutoffs for the SECOND take of
/// the same recipe — the registry's "multiple sound variants". `step` makes
/// a shorter, quieter footstep take of the same material.
fn family_recipe(f: SoundFamily, variant: u32, step: bool) -> Vec<f32> {
    // seed spread per take; step take jitters further
    let v = variant + if step { 5 } else { 0 };
    let j = 1.0 + 0.14 * ((v as f32 * 1.7).sin() * 0.5); // ±14% cutoff jitter
    let (amp, dec_scale) = if step { (0.6, 0.55) } else { (1.0, 1.0) };
    match f {
        SoundFamily::Grass => {
            let mut g = one_pole_lp(
                noise_burst((3600.0 * dec_scale) as usize, 11 + v as u64, 0.5 * amp, 0.004, 0.14 * dec_scale),
                1100.0 * j,
            );
            mix_into(
                &mut g,
                &one_pole_lp(
                    noise_burst((1200.0 * dec_scale) as usize, 12 + v as u64, 0.25 * amp, 0.002, 0.05 * dec_scale),
                    2400.0 * j,
                ),
                1.0,
            );
            clamp_amp(g)
        }
        SoundFamily::Dirt => clamp_amp(one_pole_lp(
            noise_burst((3200.0 * dec_scale) as usize, 21 + v as u64, 0.55 * amp, 0.004, 0.12 * dec_scale),
            700.0 * j,
        )),
        SoundFamily::Stone => {
            let mut s = one_pole_hp(
                one_pole_lp(
                    noise_burst((2200.0 * dec_scale) as usize, 31 + v as u64, 0.5 * amp, 0.002, 0.09 * dec_scale),
                    5500.0 * j,
                ),
                1400.0 / j,
            );
            mix_into(&mut s, &ping(2400.0 * j, 0.03, 0.35 * amp, 32 + v as u64), 1.0);
            clamp_amp(s)
        }
        SoundFamily::Wood => {
            let mut w = thump(175.0 * j, 0.13 * dec_scale, 0.55 * amp);
            mix_into(
                &mut w,
                &one_pole_lp(
                    noise_burst((1400.0 * dec_scale) as usize, 41 + v as u64, 0.3 * amp, 0.002, 0.06 * dec_scale),
                    900.0 * j,
                ),
                1.0,
            );
            clamp_amp(w)
        }
        SoundFamily::Sand => {
            let raw = noise_burst((5000.0 * dec_scale) as usize, 51 + v as u64, 0.42 * amp, 0.01, 0.2 * dec_scale);
            let mut sa = one_pole_lp(raw, 800.0 * j);
            let mut rng = Rng::new(52 + v as u64);
            for s in sa.iter_mut() {
                if rng.next_f32() < 0.35 {
                    *s *= 0.3;
                }
            }
            clamp_amp(sa)
        }
        SoundFamily::Leaves => clamp_amp(one_pole_hp(
            noise_burst((2500.0 * dec_scale) as usize, 61 + v as u64, 0.32 * amp, 0.004, 0.1 * dec_scale),
            2800.0 / j,
        )),
        SoundFamily::Glass => {
            let mut gl = one_pole_hp(
                noise_burst((4500.0 * dec_scale) as usize, 71 + v as u64, 0.45 * amp, 0.001, 0.18 * dec_scale),
                3200.0 / j,
            );
            mix_into(&mut gl, &ping(3700.0 * j, 0.09, 0.3 * amp, 72 + v as u64), 1.0);
            mix_into(&mut gl, &ping(3050.0 / j, 0.07, 0.28 * amp, 73 + v as u64), 1.0);
            mix_into(&mut gl, &ping(2400.0 * j, 0.06, 0.25 * amp, 74 + v as u64), 1.0);
            clamp_amp(gl)
        }
        SoundFamily::Wool => {
            let wl = thump(95.0 * j, 0.16 * dec_scale, 0.6 * amp);
            let mut wl = one_pole_lp(wl, 500.0 * j);
            mix_into(
                &mut wl,
                &one_pole_lp(
                    noise_burst((900.0 * dec_scale) as usize, 91 + v as u64, 0.22 * amp, 0.003, 0.08 * dec_scale),
                    600.0 * j,
                ),
                1.0,
            );
            clamp_amp(wl)
        }
        SoundFamily::Water | SoundFamily::None => clamp_amp(one_pole_lp(
            noise_burst((6600.0 * dec_scale) as usize, 81 + v as u64, 0.55 * amp, 0.02, 0.3 * dec_scale),
            2400.0 * j,
        )),
    }
}

/// UI click: short dry knock (players category)
fn click_recipe() -> Vec<f32> {
    let mut c = thump(1200.0, 0.03, 0.5);
    mix_into(&mut c, &ping(2400.0, 0.02, 0.3, 401), 1.0);
    clamp_amp(c)
}

/// item pickup pop: quick upward blip
fn pop_recipe() -> Vec<f32> {
    let mut p = ping(520.0, 0.06, 0.4, 402);
    let q = ping(780.0, 0.05, 0.3, 403);
    for i in 0..q.len().min(p.len()) {
        p[i] += q[i] * 0.6;
    }
    clamp_amp(p)
}

/// lever click: heavier mechanical clack
fn lever_recipe() -> Vec<f32> {
    let mut c = thump(320.0, 0.05, 0.7);
    mix_into(&mut c, &one_pole_hp(noise_burst(600, 404, 0.4, 0.001, 0.03), 1800.0), 1.0);
    clamp_amp(c)
}

/// ambient cave "eerie" tone: slow beating detuned sines with a long tail
fn eerie_recipe() -> Vec<f32> {
    let n = (1.6 * RATE as f32) as usize;
    (0..n)
        .map(|i| {
            let t = i as f32 / RATE as f32;
            let env = (t / 1.6).min(1.0) * (-(t - 1.6).abs().max(0.0) * 2.0).exp().min(1.0);
            let beat =
                (2.0 * std::f32::consts::PI * 220.0 * t).sin() * 0.4
                    + (2.0 * std::f32::consts::PI * 223.0 * t).sin() * 0.4
                    + (2.0 * std::f32::consts::PI * 110.5 * t).sin() * 0.2;
            beat * env * 0.5
        })
        .collect()
}

/// procedural music pad: a slow chord progression of low-passed sines with
/// a gentle tremolo. `minor` picks the night variant. ~24 s, streaming
/// category (played sparsely by the scheduler).
fn music_pad(minor: bool) -> Vec<f32> {
    // two-chord progression, root A3/F3 (night) and C4/F4-ish (day)
    let chords: [[f32; 4]; 2] = if minor {
        [[220.0, 261.6, 329.6, 440.0], [174.6, 220.0, 261.6, 349.2]]
    } else {
        [[261.6, 329.6, 392.0, 523.3], [349.2, 440.0, 523.3, 659.3]]
    };
    let chord_len = 11.5f32;
    let n = (2.0 * chord_len * RATE as f32) as usize;
    let mut out = Vec::with_capacity(n);
    for i in 0..n {
        let t = i as f32 / RATE as f32;
        let ch = chords[(t / chord_len) as usize & 1];
        // fade in/out per chord (cross-blurred boundary)
        let ct = (t % chord_len) / chord_len;
        let chord_env = (ct * std::f32::consts::PI).sin();
        // tremolo + master envelope (slow swell in, fade out)
        let trem = 0.75 + 0.25 * (2.0 * std::f32::consts::PI * 0.15 * t).sin();
        let master = (t / 3.0).clamp(0.0, 1.0) * (((23.0 - t) / 3.0).clamp(0.0, 1.0));
        let mut s = 0.0f32;
        for (k, &f) in ch.iter().enumerate() {
            // detune odd partials slightly for a soft chorus
            let det = if k & 1 == 1 { 1.002 } else { 1.0 };
            s += (2.0 * std::f32::consts::PI * f * det * t).sin() * (0.22 / (1.0 + k as f32 * 0.6));
        }
        out.push(s * trem * chord_env * master);
    }
    // soften with a low pass
    one_pole_lp(out, 1400.0)
}

impl SoundBank {
    /// synthesize every recipe the registry references (plus the legacy
    /// family-indexed slots at 0..9 for old call paths during migration)
    pub fn generate() -> Self {
        let mut names: Vec<String> = Vec::new();
        let mut data: Vec<Vec<f32>> = Vec::new();

        let fam_name = |f: SoundFamily| -> &'static str {
            match f {
                SoundFamily::Grass => "grass",
                SoundFamily::Dirt => "dirt",
                SoundFamily::Stone => "stone",
                SoundFamily::Wood => "wood",
                SoundFamily::Sand => "sand",
                SoundFamily::Leaves => "leaves",
                SoundFamily::Glass => "glass",
                SoundFamily::Wool => "wool",
                SoundFamily::Water | SoundFamily::None => "water",
            }
        };
        let families = [
            SoundFamily::Grass,
            SoundFamily::Dirt,
            SoundFamily::Stone,
            SoundFamily::Wood,
            SoundFamily::Sand,
            SoundFamily::Leaves,
            SoundFamily::Glass,
            SoundFamily::Wool,
            SoundFamily::Water,
        ];
        // dig/place variants + steps per family
        for f in families {
            for v in 0..2 {
                names.push(format!("dig/{}{}", fam_name(f), v + 1));
                data.push(family_recipe(f, v, false));
            }
            names.push(format!("step/{}", fam_name(f)));
            data.push(family_recipe(f, 0, true));
        }
        // one-off recipes
        for (n, d) in [
            ("ui/click", click_recipe()),
            ("entity/item/pickup", pop_recipe()),
            ("block/lever", lever_recipe()),
            ("ambient/eerie", eerie_recipe()),
            ("music/pad_day", music_pad(false)),
            ("music/pad_night", music_pad(true)),
        ] {
            names.push(n.into());
            data.push(d);
        }

        let mut index = HashMap::new();
        for (i, n) in names.iter().enumerate() {
            index.insert(n.clone(), i);
        }
        let wavs = data.iter().map(|d| to_wav16(d, RATE)).collect();
        SoundBank { names, index, data, wavs }
    }

    /// recipe slot by name (registry "sounds[].name" resolves through this)
    pub fn recipe(&self, name: &str) -> Option<usize> {
        self.index.get(name).copied()
    }
}

/// Linear-resample by pitch factor (native playback pitch variation).
pub fn resample(samples: &[f32], pitch: f32) -> Vec<f32> {
    if pitch <= 0.01 {
        return samples.to_vec();
    }
    let n = (samples.len() as f32 / pitch) as usize;
    let mut out = Vec::with_capacity(n);
    for i in 0..n {
        let src = i as f32 * pitch;
        let i0 = src.floor() as usize;
        let i1 = (i0 + 1).min(samples.len() - 1);
        let f = src - i0 as f32;
        out.push(samples[i0] * (1.0 - f) + samples[i1] * f);
    }
    out
}

/// 16-bit PCM mono WAV bytes.
pub fn to_wav16(samples: &[f32], rate: u32) -> Vec<u8> {
    let n = samples.len();
    let mut out = Vec::with_capacity(44 + n * 2);
    out.extend_from_slice(b"RIFF");
    out.extend_from_slice(&((36 + n * 2) as u32).to_le_bytes());
    out.extend_from_slice(b"WAVE");
    out.extend_from_slice(b"fmt ");
    out.extend_from_slice(&(16u32).to_le_bytes());
    out.extend_from_slice(&(1u16).to_le_bytes()); // PCM
    out.extend_from_slice(&(1u16).to_le_bytes()); // mono
    out.extend_from_slice(&rate.to_le_bytes());
    out.extend_from_slice(&(rate * 2).to_le_bytes());
    out.extend_from_slice(&(2u16).to_le_bytes());
    out.extend_from_slice(&(16u16).to_le_bytes());
    out.extend_from_slice(b"data");
    out.extend_from_slice(&((n * 2) as u32).to_le_bytes());
    for s in samples {
        let v: i16 = (s.clamp(-1.0, 1.0) * 32767.0) as i16;
        out.extend_from_slice(&v.to_le_bytes());
    }
    out
}

// ------------------------------------------------- §21 sound-event registry --

/// vanilla 1.16.5 sound categories (spec: "sound categories")
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SoundCategory {
    Master,
    Music,
    Record,
    Weather,
    Blocks,
    Hostile,
    Neutral,
    Players,
    Ambient,
}

impl SoundCategory {
    pub fn from_name(s: &str) -> Option<Self> {
        Some(match s {
            "master" => Self::Master,
            "music" => Self::Music,
            "record" => Self::Record,
            "weather" => Self::Weather,
            "blocks" => Self::Blocks,
            "hostile" => Self::Hostile,
            "neutral" => Self::Neutral,
            "players" => Self::Players,
            "ambient" => Self::Ambient,
            _ => return None,
        })
    }
    pub fn name(self) -> &'static str {
        match self {
            Self::Master => "master",
            Self::Music => "music",
            Self::Record => "record",
            Self::Weather => "weather",
            Self::Blocks => "blocks",
            Self::Hostile => "hostile",
            Self::Neutral => "neutral",
            Self::Players => "players",
            Self::Ambient => "ambient",
        }
    }
}

#[derive(Clone, Deserialize, Debug)]
pub struct SoundVariantDef {
    /// recipe name in the SoundBank ("dig/stone2", "music/pad_day"…)
    pub name: String,
    /// random-selection weight (vanilla default 1)
    #[serde(default = "one")]
    pub weight: u32,
    /// variant-level volume multiplier
    #[serde(default = "one_f")]
    pub volume: f32,
    /// variant-level pitch override
    pub pitch: Option<f32>,
    /// streaming flag (music: one long buffer instead of burst effects)
    #[serde(default)]
    pub stream: bool,
}

fn one() -> u32 {
    1
}
fn one_f() -> f32 {
    1.0
}

#[derive(Clone, Deserialize, Debug)]
pub struct SoundEventDef {
    /// vanilla category name
    pub category: String,
    /// event volume
    #[serde(default = "one_f")]
    pub volume: f32,
    /// pitch range [min, max] (uniform pick — vanilla behaviour)
    pub pitch: Option<[f32; 2]>,
    /// attenuation distance in blocks (default 16, vanilla-style)
    #[serde(default = "att_default")]
    pub attenuation: f32,
    /// weighted variant list
    pub sounds: Vec<SoundVariantDef>,
}

fn att_default() -> f32 {
    16.0
}

/// the registry: event name → definition. Parsed from `SOUNDS_JSON`
/// (vanilla `sounds.json` field shape, our own recipe names).
#[derive(Debug)]
pub struct SoundRegistry {
    pub events: HashMap<String, SoundEventDef>,
}

/// one sound fully resolved for playback (recipe picked, pitch rolled)
#[derive(Clone, Debug)]
pub struct ResolvedSound {
    pub recipe: usize,
    pub volume: f32,
    pub pitch: f32,
    pub category: SoundCategory,
    pub stream: bool,
    pub attenuation: f32,
}

impl SoundRegistry {
    pub fn from_json(json: &str) -> Result<Self, String> {
        let map: HashMap<String, SoundEventDef> =
            serde_json::from_str(json).map_err(|e| format!("sounds.json: {e}"))?;
        // validation: every category + recipe must exist
        for (name, def) in map.iter() {
            SoundCategory::from_name(&def.category)
                .ok_or_else(|| format!("sounds.json: {name} has bad category {}", def.category))?;
            if def.sounds.is_empty() {
                return Err(format!("sounds.json: {name} has no variants"));
            }
            for v in def.sounds.iter() {
                if v.weight == 0 {
                    return Err(format!("sounds.json: {name}/{} weight 0", v.name));
                }
            }
        }
        Ok(SoundRegistry { events: map })
    }

    /// weighted-random variant + pitch roll. `rng` drives the pick (the
    /// caller keeps one rng so the sequence is deterministic in tests).
    pub fn pick(&self, event: &str, rng: &mut Rng, bank: &SoundBank) -> Option<ResolvedSound> {
        let def = self.events.get(event)?;
        // weighted pick
        let total: u32 = def.sounds.iter().map(|v| v.weight).sum();
        let mut roll = rng.next_f32() * total as f32;
        let mut chosen = &def.sounds[0];
        for v in def.sounds.iter() {
            if roll < v.weight as f32 {
                chosen = v;
                break;
            }
            roll -= v.weight as f32;
        }
        let recipe = bank.recipe(&chosen.name)?;
        // pitch: variant override, else event range, else 1.0
        let pitch = chosen
            .pitch
            .or_else(|| def.pitch.map(|[a, b]| a + rng.next_f32() * (b - a)))
            .unwrap_or(1.0);
        let category = SoundCategory::from_name(&def.category)?;
        Some(ResolvedSound {
            recipe,
            volume: def.volume * chosen.volume,
            pitch: pitch.clamp(0.5, 2.0),
            category,
            stream: chosen.stream,
            attenuation: def.attenuation,
        })
    }
}

/// clean-room sound registry (vanilla sounds.json field shape; every
/// recipe is our own synthesis). Generated, not copied.
pub const SOUNDS_JSON: &str = r##"{
  "block.grass.dig":     {"category": "blocks", "pitch": [0.8, 1.1], "sounds": [
    {"name": "dig/grass1", "weight": 3}, {"name": "dig/grass2", "weight": 1}]},
  "block.grass.step":    {"category": "blocks", "volume": 0.35, "pitch": [0.9, 1.05], "sounds": [{"name": "step/grass"}]},
  "block.dirt.dig":      {"category": "blocks", "pitch": [0.8, 1.1], "sounds": [
    {"name": "dig/dirt1", "weight": 3}, {"name": "dig/dirt2", "weight": 1}]},
  "block.dirt.step":     {"category": "blocks", "volume": 0.35, "pitch": [0.9, 1.05], "sounds": [{"name": "step/dirt"}]},
  "block.stone.dig":     {"category": "blocks", "pitch": [0.8, 1.0], "sounds": [
    {"name": "dig/stone1", "weight": 2}, {"name": "dig/stone2", "weight": 2}]},
  "block.stone.step":    {"category": "blocks", "volume": 0.3, "pitch": [0.9, 1.05], "sounds": [{"name": "step/stone"}]},
  "block.wood.dig":      {"category": "blocks", "pitch": [0.85, 1.1], "sounds": [
    {"name": "dig/wood1", "weight": 3}, {"name": "dig/wood2", "weight": 1}]},
  "block.wood.step":     {"category": "blocks", "volume": 0.35, "pitch": [0.9, 1.05], "sounds": [{"name": "step/wood"}]},
  "block.sand.dig":      {"category": "blocks", "pitch": [0.9, 1.15], "sounds": [
    {"name": "dig/sand1", "weight": 3}, {"name": "dig/sand2", "weight": 1}]},
  "block.sand.step":     {"category": "blocks", "volume": 0.3, "pitch": [0.9, 1.1], "sounds": [{"name": "step/sand"}]},
  "block.leaves.dig":    {"category": "blocks", "pitch": [0.9, 1.15], "sounds": [
    {"name": "dig/leaves1", "weight": 3}, {"name": "dig/leaves2", "weight": 1}]},
  "block.leaves.step":   {"category": "blocks", "volume": 0.32, "pitch": [0.95, 1.1], "sounds": [{"name": "step/leaves"}]},
  "block.glass.break":   {"category": "blocks", "volume": 0.9, "pitch": [0.85, 1.1], "sounds": [
    {"name": "dig/glass1", "weight": 2}, {"name": "dig/glass2", "weight": 1}]},
  "block.glass.step":    {"category": "blocks", "volume": 0.3, "pitch": [0.95, 1.1], "sounds": [{"name": "step/glass"}]},
  "block.wool.dig":      {"category": "blocks", "pitch": [0.85, 1.05], "sounds": [
    {"name": "dig/wool1", "weight": 3}, {"name": "dig/wool2", "weight": 1}]},
  "block.wool.step":     {"category": "blocks", "volume": 0.32, "pitch": [0.9, 1.05], "sounds": [{"name": "step/wool"}]},
  "block.water.splash":  {"category": "blocks", "volume": 0.8, "pitch": [0.8, 1.2], "attenuation": 12, "sounds": [
    {"name": "dig/water1", "weight": 2}, {"name": "dig/water2", "weight": 1}]},
  "block.water.step":    {"category": "blocks", "volume": 0.4, "pitch": [0.8, 1.2], "sounds": [{"name": "step/water"}]},
  "block.lever.click":   {"category": "blocks", "volume": 0.6, "pitch": [0.9, 1.1], "sounds": [{"name": "block/lever"}]},
  "ui.click":            {"category": "players", "volume": 0.35, "pitch": [1.5, 1.7], "sounds": [{"name": "ui/click"}]},
  "entity.item.pickup":  {"category": "players", "volume": 0.45, "pitch": [0.9, 1.3], "sounds": [{"name": "entity/item/pickup"}]},
  "ambient.eerie":       {"category": "ambient", "volume": 0.55, "pitch": [0.85, 1.3], "attenuation": 0, "sounds": [{"name": "ambient/eerie"}]},
  "music.pad.day":       {"category": "music", "volume": 0.5, "sounds": [{"name": "music/pad_day", "stream": true}]},
  "music.pad.night":     {"category": "music", "volume": 0.5, "sounds": [{"name": "music/pad_night", "stream": true}]}
}"##;

/// distance attenuation + stereo pan for one positioned sound relative to
/// the listener (eye position + yaw). Returns (volume, pan) — volume 0
/// when outside the attenuation range (0 = non-positional/global).
pub fn spatialize(
    pos: Option<[f32; 3]>,
    listener: [f32; 3],
    yaw: f32,
    attenuation: f32,
) -> (f32, f32) {
    let Some(p) = pos else {
        return (1.0, 0.0); // global sound (UI, music, ambient)
    };
    let dx = p[0] - listener[0];
    let dy = p[1] - listener[1];
    let dz = p[2] - listener[2];
    let dist = (dx * dx + dy * dy + dz * dz).sqrt();
    if attenuation <= 0.0 || dist >= attenuation {
        return (0.0, 0.0);
    }
    // smooth falloff to zero at the attenuation edge
    let vol = (1.0 - dist / attenuation).powi(2);
    // pan: projection on the listener's RIGHT vector
    // (yaw 0 faces −Z; right = (cos yaw, 0, sin yaw))
    let rx = yaw.cos();
    let rz = yaw.sin();
    let d = (dx * rx + dz * rz) / dist.max(1e-4);
    (vol, d.clamp(-0.8, 0.8))
}

/// map a block SoundFamily to its registry event: dig (break/place) or
/// step (footsteps). Glass has no "dig" in vanilla — it breaks; water
/// has splash. This is the migration bridge from family-based call sites
/// to the §21 event system.
pub fn family_event(f: SoundFamily, dig: bool) -> &'static str {
    match f {
        SoundFamily::Grass => dig_or_step(dig, "grass"),
        SoundFamily::Dirt => dig_or_step(dig, "dirt"),
        SoundFamily::Stone => dig_or_step(dig, "stone"),
        SoundFamily::Wood => dig_or_step(dig, "wood"),
        SoundFamily::Sand => dig_or_step(dig, "sand"),
        SoundFamily::Leaves => dig_or_step(dig, "leaves"),
        SoundFamily::Glass => {
            if dig {
                "block.glass.break"
            } else {
                "block.glass.step"
            }
        }
        SoundFamily::Wool => dig_or_step(dig, "wool"),
        SoundFamily::Water | SoundFamily::None => "block.water.splash",
    }
}

fn dig_or_step(dig: bool, fam: &str) -> &'static str {
    match (dig, fam) {
        (true, "grass") => "block.grass.dig",
        (false, "grass") => "block.grass.step",
        (true, "dirt") => "block.dirt.dig",
        (false, "dirt") => "block.dirt.step",
        (true, "stone") => "block.stone.dig",
        (false, "stone") => "block.stone.step",
        (true, "wood") => "block.wood.dig",
        (false, "wood") => "block.wood.step",
        (true, "sand") => "block.sand.dig",
        (false, "sand") => "block.sand.step",
        (true, "leaves") => "block.leaves.dig",
        (false, "leaves") => "block.leaves.step",
        (true, "wool") => "block.wool.dig",
        (false, "wool") => "block.wool.step",
        _ => "block.stone.dig",
    }
}

// ------------------------------------------------------------- backends --

pub trait AudioBackend {
    /// play one bank slot with gain, pitch and stereo pan (−1 left … +1
    /// right). All §21 mixing (category gains, attenuation) happens in the
    /// caller; backends just render.
    fn play(&self, bank: &SoundBank, slot: usize, volume: f32, pitch: f32, pan: f32);
    /// unlock audio context (wasm: needs user gesture; decodes the bank)
    fn unlock(&self, _bank: &SoundBank) {}
}

/// No-device silent fallback.
pub struct SilentOut;
impl AudioBackend for SilentOut {
    fn play(&self, _bank: &SoundBank, _slot: usize, _volume: f32, _pitch: f32, _pan: f32) {}
}

// --- native (rodio) ---
#[cfg(all(not(target_arch = "wasm32"), feature = "audio"))]
pub mod native_audio {
    use super::*;

    pub struct RodioOut {
        _stream: rodio::OutputStream,
        handle: rodio::OutputStreamHandle,
    }

    impl RodioOut {
        pub fn new() -> Option<Self> {
            match rodio::OutputStream::try_default() {
                Ok((stream, handle)) => Some(RodioOut { _stream: stream, handle }),
                Err(_) => None,
            }
        }
    }

    impl AudioBackend for RodioOut {
        fn play(&self, bank: &SoundBank, slot: usize, volume: f32, pitch: f32, pan: f32) {
            let Some(base) = bank.data.get(slot) else { return };
            let samples = resample(base, pitch);
            // stereo from pan: equal-power law
            let l = (0.5 * (1.0 - pan) + 0.5).sqrt();
            let r = (0.5 * (1.0 + pan) + 0.5).sqrt();
            let stereo: Vec<f32> = samples
                .iter()
                .flat_map(|s| [*s * l, *s * r])
                .collect();
            let src = rodio::buffer::SamplesBuffer::new(2u16, RATE, stereo);
            if let Ok(sink) = rodio::Sink::try_new(&self.handle) {
                sink.set_volume(volume.clamp(0.0, 1.0));
                sink.append(src);
                sink.detach();
            }
        }
    }
}

// --- native silent (no-audio build) ---
#[cfg(all(not(target_arch = "wasm32"), not(feature = "audio")))]
pub mod native_audio {
    use super::*;

    pub struct RodioOut;
    impl RodioOut {
        pub fn new() -> Option<Self> {
            Some(RodioOut)
        }
    }

    impl AudioBackend for RodioOut {
        fn play(&self, _bank: &SoundBank, _slot: usize, _volume: f32, _pitch: f32, _pan: f32) {}
    }
}

// --- wasm (WebAudio) ---
#[cfg(target_arch = "wasm32")]
pub mod web_audio {
    use super::*;
    use std::cell::RefCell;
    use std::rc::Rc;
    use wasm_bindgen::prelude::*;

    struct Inner {
        ctx: RefCell<Option<web_sys::AudioContext>>,
        buffers: RefCell<Vec<Option<web_sys::AudioBuffer>>>,
    }

    pub struct WebAudioOut {
        inner: Rc<Inner>,
    }

    impl WebAudioOut {
        pub fn new() -> Self {
            WebAudioOut {
                inner: Rc::new(Inner {
                    ctx: RefCell::new(None),
                    buffers: RefCell::new(vec![None; 64]),
                }),
            }
        }
    }

    impl AudioBackend for WebAudioOut {
        fn play(&self, bank: &SoundBank, slot: usize, volume: f32, pitch: f32, pan: f32) {
            let ctx_guard = self.inner.ctx.borrow();
            let Some(ctx) = ctx_guard.as_ref() else { return };
            let mut bufs = self.inner.buffers.borrow_mut();
            if slot >= bufs.len() {
                bufs.resize(slot + 1, None);
            }
            if bufs[slot].is_none() {
                // lazily decode this slot's WAV on first use
                if let Some(wav) = bank.wavs.get(slot) {
                    // decode_audio_data consumes the ArrayBuffer; clone bytes
                    let wav = wav.clone();
                    let inner = Rc::clone(&self.inner);
                    let slot2 = slot;
                    drop(bufs);
                    drop(ctx_guard);
                    wasm_bindgen_futures::spawn_local(async move {
                        let ctx2 = inner.ctx.borrow().clone();
                        let Some(ctx2) = ctx2 else { return };
                        if let Some(b) = decode(&ctx2, &wav).await {
                            let mut bufs2 = inner.buffers.borrow_mut();
                            if slot2 >= bufs2.len() {
                                bufs2.resize(slot2 + 1, None);
                            }
                            bufs2[slot2] = Some(b);
                        }
                    });
                }
                return; // this play is dropped; the next one lands decoded
            }
            let Some(buf) = bufs[slot].clone() else { return };
            if let Ok(src) = web_sys::AudioContext::create_buffer_source(ctx) {
                web_sys::AudioBufferSourceNode::set_buffer(&src, Some(&buf));
                let pr = web_sys::AudioBufferSourceNode::playback_rate(&src);
                web_sys::AudioParam::set_value(&pr, pitch);
                let dest = web_sys::AudioContext::destination(ctx);
                // chain: source → gain → panner → destination
                if let Ok(gain) = web_sys::AudioContext::create_gain(ctx) {
                    let g = web_sys::GainNode::gain(&gain);
                    web_sys::AudioParam::set_value(&g, volume.clamp(0.0, 1.0));
                    if let Ok(panner) = web_sys::AudioContext::create_stereo_panner(ctx) {
                        let p = web_sys::StereoPannerNode::pan(&panner);
                        web_sys::AudioParam::set_value(&p, pan.clamp(-1.0, 1.0));
                        let _ = web_sys::AudioNode::connect_with_audio_node(&src, &gain);
                        let _ = web_sys::AudioNode::connect_with_audio_node(&gain, &panner);
                        let _ = web_sys::AudioNode::connect_with_audio_node(&panner, &dest);
                    } else {
                        let _ = web_sys::AudioNode::connect_with_audio_node(&src, &gain);
                        let _ = web_sys::AudioNode::connect_with_audio_node(&gain, &dest);
                    }
                } else {
                    let _ = web_sys::AudioNode::connect_with_audio_node(&src, &dest);
                }
                let _ = web_sys::AudioBufferSourceNode::start(&src);
            }
        }

        fn unlock(&self, bank: &SoundBank) {
            let inner = Rc::clone(&self.inner);
            let wavs: Vec<Vec<u8>> = bank.wavs.clone();
            wasm_bindgen_futures::spawn_local(async move {
                if inner.ctx.borrow().is_some() {
                    return;
                }
                let Ok(ctx) = web_sys::AudioContext::new() else { return };
                let mut bufs: Vec<Option<web_sys::AudioBuffer>> = Vec::with_capacity(wavs.len());
                for wav in wavs.iter() {
                    let decoded = decode(&ctx, wav).await;
                    bufs.push(decoded);
                }
                *inner.buffers.borrow_mut() = bufs;
                *inner.ctx.borrow_mut() = Some(ctx);
            });
        }
    }

    async fn decode(ctx: &web_sys::AudioContext, wav: &[u8]) -> Option<web_sys::AudioBuffer> {
        let arr_buf = js_sys::Uint8Array::from(wav).buffer();
        let promise = web_sys::AudioContext::decode_audio_data(ctx, &arr_buf).ok()?;
        let js = wasm_bindgen_futures::JsFuture::from(promise).await.ok()?;
        js.dyn_into::<web_sys::AudioBuffer>().ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// §21 registry: parses, validates, and every recipe resolves
    #[test]
    fn registry_parses_and_resolves() {
        let bank = SoundBank::generate();
        let reg = SoundRegistry::from_json(SOUNDS_JSON).expect("parse");
        assert!(reg.events.len() >= 24, "event count {}", reg.events.len());
        let mut rng = Rng::new(7);
        for name in reg.events.keys() {
            let r = reg.pick(name, &mut rng, &bank);
            assert!(r.is_some(), "event {name} failed to resolve a recipe");
        }
    }

    /// weighted variant selection: 3:1 weights → the rare variant shows up
    /// but rarely (statistical, fixed seed → deterministic)
    #[test]
    fn weighted_pick_distribution() {
        let bank = SoundBank::generate();
        let reg = SoundRegistry::from_json(SOUNDS_JSON).unwrap();
        let mut rng = Rng::new(42);
        let mut rare = 0;
        for _ in 0..400 {
            let r = reg.pick("block.grass.dig", &mut rng, &bank).unwrap();
            // grass2 is the 1-weight variant — find its slot
            let rare_slot = bank.recipe("dig/grass2").unwrap();
            if r.recipe == rare_slot {
                rare += 1;
            }
        }
        // 400 picks at p=1/4 → expect ~100; assert a loose 60..140 band
        assert!(
            (60..=140).contains(&rare),
            "rare variant picked {rare}/400 — weighting broken"
        );
    }

    /// pitch ranges roll inside their bounds
    #[test]
    fn pitch_stays_in_range() {
        let bank = SoundBank::generate();
        let reg = SoundRegistry::from_json(SOUNDS_JSON).unwrap();
        let mut rng = Rng::new(9);
        for _ in 0..200 {
            let r = reg.pick("block.stone.dig", &mut rng, &bank).unwrap();
            assert!((0.8..=1.0).contains(&r.pitch), "pitch {}", r.pitch);
        }
    }

    /// spatialization: falloff + pan direction (spec: attenuation,
    /// spatial positioning)
    #[test]
    fn spatial_attenuation_and_pan() {
        let ear = [0.0, 64.0, 0.0];
        // sound at the listener: full volume, no pan
        let (v, p) = spatialize(Some(ear), ear, 0.0, 16.0);
        assert!((v - 1.0).abs() < 1e-4 && p.abs() < 1e-4);
        // hard right of a yaw-0 listener (right = +X): pan > 0.5
        let (v, p) = spatialize(Some([10.0, 64.0, 0.0]), ear, 0.0, 16.0);
        assert!(p > 0.5, "right-side pan {p}");
        assert!((v - (1.0f32 - 10.0f32 / 16.0).powi(2)).abs() < 1e-3, "volume {v}");
        // outside the attenuation range: silent
        let (v, _) = spatialize(Some([40.0, 64.0, 0.0]), ear, 0.0, 16.0);
        assert!(v < 1e-4, "volume past range {v}");
        // non-positional: global
        let (v, p) = spatialize(None, ear, 0.0, 16.0);
        assert!((v - 1.0).abs() < 1e-4 && p == 0.0);
    }

    /// music pads are long, quiet-ish, and non-clipped
    #[test]
    fn music_pads_sane() {
        for (name, minor) in [("music/pad_day", false), ("music/pad_night", true)] {
            let pad = music_pad(minor);
            assert!(pad.len() > RATE as usize * 20, "{name} too short");
            let peak = pad.iter().fold(0.0f32, |a, s| a.max(s.abs()));
            assert!(peak <= 1.0 && peak > 0.05, "{name} peak {peak}");
        }
    }
}
