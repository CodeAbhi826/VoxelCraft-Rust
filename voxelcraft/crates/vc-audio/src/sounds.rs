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

use vc_blocks::blocks::SoundFamily;
use vc_rng::rng::Rng;
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
        // Sub-round 5: the new material classes ride the legacy slots
        // closest to their character (the registry path never uses these)
        SoundFamily::Gravel => 4,
        SoundFamily::Metal => 2,
        SoundFamily::Plant => 0,
        SoundFamily::Chain => 2,
        SoundFamily::NetherWood => 3,
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

fn mix_into(a: &mut [f32], b: &[f32], gain: f32) {
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
/// Sub-round 5: which of the five vanilla material events a take
/// synthesizes — dig (break), the lighter mining-cadence hit, the soft
/// step, or the heavier landing fall. Place uses the Dig take at a lower
/// registry volume (vanilla's own approach: place IS the break sound,
/// quieter + shorter pitch window).
#[derive(Clone, Copy, PartialEq)]
pub enum Take {
    Dig,
    Hit,
    Step,
    Fall,
}

fn family_recipe(f: SoundFamily, variant: u32, take: Take) -> Vec<f32> {
    // seed spread per take; light takes jitter further
    let v = variant
        + match take {
            Take::Dig => 0,
            Take::Hit => 5,
            Take::Step => 9,
            Take::Fall => 13,
        };
    let j = 1.0 + 0.14 * ((v as f32 * 1.7).sin() * 0.5); // ±14% cutoff jitter
    // hit: light + short (the mining cadence); step: soft + shorter;
    // fall: heavy + LONG (the landing thud)
    let (amp, dec_scale) = match take {
        Take::Dig => (1.0, 1.0),
        Take::Hit => (0.62, 0.6),
        Take::Step => (0.6, 0.55),
        Take::Fall => (1.25, 1.7),
    };
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
        // ---- Sub-round 5: the new material classes ----
        // gravel: crunchy mid-low noise with a loose-stone rattle
        SoundFamily::Gravel => {
            let raw = noise_burst((4200.0 * dec_scale) as usize, 101 + v as u64, 0.5 * amp, 0.003, 0.16 * dec_scale);
            let mut gv = one_pole_lp(raw, 620.0 * j);
            let mut rng = Rng::new(102 + v as u64);
            for s in gv.iter_mut() {
                if rng.next_f32() < 0.28 {
                    *s *= 0.35;
                }
            }
            mix_into(&mut gv, &thump(120.0 * j, 0.09 * dec_scale, 0.3 * amp), 1.0);
            clamp_amp(gv)
        }
        // metal: a hard attack thump + two ringing partials (the anvil
        // character — clean-room: decaying sine partials, no sampled
        // resonance)
        SoundFamily::Metal => {
            let mut m = thump(210.0 * j, 0.16 * dec_scale, 0.5 * amp);
            mix_into(&mut m, &ping(1560.0 * j, 0.11 * dec_scale, 0.3 * amp, 111 + v as u64), 1.0);
            mix_into(&mut m, &ping(2340.0 / j, 0.08 * dec_scale, 0.22 * amp, 112 + v as u64), 1.0);
            mix_into(
                &mut m,
                &one_pole_hp(
                    noise_burst((900.0 * dec_scale) as usize, 113 + v as u64, 0.18 * amp, 0.001, 0.03 * dec_scale),
                    2600.0 * j,
                ),
                1.0,
            );
            clamp_amp(m)
        }
        // plant: crisp short foliage snap (bamho/bush class)
        SoundFamily::Plant => {
            let mut p = one_pole_hp(
                noise_burst((1600.0 * dec_scale) as usize, 121 + v as u64, 0.4 * amp, 0.001, 0.07 * dec_scale),
                1800.0 / j,
            );
            mix_into(&mut p, &ping(2900.0 * j, 0.03, 0.2 * amp, 122 + v as u64), 1.0);
            clamp_amp(p)
        }
        // chain: a short metallic clink (two quick partials)
        SoundFamily::Chain => {
            let mut c = ping(2050.0 * j, 0.05 * dec_scale, 0.4 * amp, 131 + v as u64);
            mix_into(&mut c, &ping(3400.0 / j, 0.035 * dec_scale, 0.3 * amp, 132 + v as u64), 1.0);
            mix_into(
                &mut c,
                &one_pole_hp(
                    noise_burst((700.0 * dec_scale) as usize, 133 + v as u64, 0.22 * amp, 0.001, 0.02 * dec_scale),
                    3000.0 * j,
                ),
                1.0,
            );
            clamp_amp(c)
        }
        // nether wood: the wood recipe pitched down + a hollow thump
        // (the crimson/warped "nether_wood" class)
        SoundFamily::NetherWood => {
            let mut w = thump(112.0 * j, 0.17 * dec_scale, 0.55 * amp);
            mix_into(
                &mut w,
                &one_pole_lp(
                    noise_burst((1500.0 * dec_scale) as usize, 141 + v as u64, 0.3 * amp, 0.003, 0.07 * dec_scale),
                    640.0 * j,
                ),
                1.0,
            );
            mix_into(&mut w, &ping(190.0 * j, 0.09 * dec_scale, 0.25 * amp, 142 + v as u64), 1.0);
            clamp_amp(w)
        }
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

/// brewing bubble (§29): a row of low liquid pips rising in pitch —
/// the "glug glug" of an active stand
fn bubble_recipe() -> Vec<f32> {
    let mut out = thump(180.0, 0.09, 0.5);
    mix_into(&mut out, &ping(300.0, 0.10, 0.25, 405), 1.0);
    mix_into(&mut out, &ping(430.0, 0.08, 0.2, 406), 0.8);
    clamp_amp(out)
}

/// drinking a potion (§29): two soft swallows
fn drink_recipe() -> Vec<f32> {
    let mut d = thump(220.0, 0.12, 0.55);
    let second = thump(190.0, 0.10, 0.4);
    mix_into(&mut d, &second, 0.7);
    clamp_amp(d)
}

/// potion splash (§29, breaking a stand / filling a bottle): watery pip
fn splash_recipe() -> Vec<f32> {
    let mut s = ping(900.0, 0.07, 0.3, 407);
    mix_into(&mut s, &one_pole_lp(noise_burst(900, 408, 0.3, 0.001, 0.06), 2500.0), 1.0);
    clamp_amp(s)
}

/// enchanting-table use (§29): a rising mystical arpeggio
fn enchant_recipe() -> Vec<f32> {
    let mut e = ping(520.0, 0.25, 0.25, 409);
    mix_into(&mut e, &ping(660.0, 0.25, 0.2, 410), 0.9);
    mix_into(&mut e, &ping(880.0, 0.30, 0.16, 411), 0.8);
    mix_into(&mut e, &ping(1320.0, 0.35, 0.10, 412), 0.7);
    clamp_amp(e)
}

/// player level-up (§29): bright two-note rise
fn levelup_recipe() -> Vec<f32> {
    let mut l = ping(740.0, 0.12, 0.3, 413);
    mix_into(&mut l, &ping(1110.0, 0.20, 0.25, 414), 1.0);
    clamp_amp(l)
}

/// villager ambient (§27): the "hrmm" — two soft thumps
fn villager_ambient_recipe() -> Vec<f32> {
    let mut v = thump(150.0, 0.18, 0.4);
    let h = thump(200.0, 0.14, 0.25);
    mix_into(&mut v, &h, 0.6);
    clamp_amp(v)
}

/// villager trade (§29): the pleased agreeing grunt — rising pair
fn villager_trade_recipe() -> Vec<f32> {
    let mut v = ping(260.0, 0.10, 0.35, 415);
    mix_into(&mut v, &ping(340.0, 0.14, 0.3, 416), 1.0);
    clamp_amp(v)
}

// ---------------------------------------------- Phase 2 combat sounds --

/// player hurt: sharp low thud + brief noise gasp
fn hurt_recipe() -> Vec<f32> {
    let mut v = thump(90.0, 0.16, 0.55);
    mix_into(&mut v, &noise_burst((0.08 * RATE as f32) as usize, 77, 0.25, 0.02, 0.8), 0.8);
    clamp_amp(v)
}

/// zombie groan: two descending low thumps (nasally "uhh-hrr")
fn zombie_groan_recipe() -> Vec<f32> {
    let mut v = thump(110.0, 0.22, 0.45);
    let mut d = thump(80.0, 0.26, 0.4);
    mix_into(&mut v, &d, 0.8);
    d = thump(140.0, 0.1, 0.2);
    mix_into(&mut v, &d, 0.5);
    clamp_amp(v)
}

/// generic mob death: falling pitch pair
fn death_recipe() -> Vec<f32> {
    let mut v = ping(320.0, 0.12, 0.4, 621);
    mix_into(&mut v, &ping(180.0, 0.22, 0.4, 622), 1.0);
    clamp_amp(v)
}

/// creeper priming: the rising hiss — filtered noise swell
fn priming_recipe() -> Vec<f32> {
    let n = (1.5 * RATE as f32) as usize;
    let mut v: Vec<f32> = (0..n)
        .map(|i| {
            let t = i as f32 / n as f32;
            let noise = vc_rng::rng::Rng::new(777 + i as u64).next_f32() * 2.0 - 1.0;
            noise * t * 0.5
        })
        .collect();
    let _ = &mut v;
    clamp_amp(v)
}

/// explosion: big noise burst with a low boom tail
fn explosion_recipe() -> Vec<f32> {
    let mut v = noise_burst((0.9 * RATE as f32) as usize, 991, 0.9, 0.005, 3.0);
    mix_into(&mut v, &thump(45.0, 0.7, 0.9), 1.0);
    mix_into(&mut v, &thump(60.0, 0.5, 0.6), 1.0);
    clamp_amp(v)
}

/// skeleton bow: the twang — quick ping + pluck noise
fn bow_recipe() -> Vec<f32> {
    let mut v = ping(500.0, 0.08, 0.4, 731);
    mix_into(&mut v, &noise_burst((0.05 * RATE as f32) as usize, 732, 0.3, 0.01, 1.5), 1.0);
    clamp_amp(v)
}

/// cow moo: long low thump with vibrato-ish second harmonic
fn cow_recipe() -> Vec<f32> {
    let mut v = thump(85.0, 0.5, 0.5);
    mix_into(&mut v, &thump(170.0, 0.4, 0.2), 0.8);
    clamp_amp(v)
}

/// pig oink: two quick nasal thumps
fn pig_recipe() -> Vec<f32> {
    let mut v = thump(220.0, 0.09, 0.4);
    mix_into(&mut v, &thump(160.0, 0.11, 0.35), 1.0);
    clamp_amp(v)
}

/// sheep baa: wobbly mid thump pair
fn sheep_recipe() -> Vec<f32> {
    let mut v = thump(280.0, 0.14, 0.4);
    mix_into(&mut v, &thump(240.0, 0.16, 0.35), 1.0);
    clamp_amp(v)
}

/// chicken cluck: short bright pings
fn chicken_recipe() -> Vec<f32> {
    let mut v = ping(820.0, 0.05, 0.3, 911);
    mix_into(&mut v, &ping(700.0, 0.06, 0.3, 912), 1.0);
    clamp_amp(v)
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

/// Backlog round (weather): thunder — a clean-room rumble. A noise
/// burst low-passed hard (one-pole at ~150 Hz), a sub-bass thump at
/// 45 Hz, and a slow double-decay tail (~2.5 s). Not any vanilla
/// recording; the recipe shape only (rumble + crack).
fn thunder_recipe() -> Vec<f32> {
    let dur = 2.6;
    let n = (dur * RATE as f32) as usize;
    let mut out = Vec::with_capacity(n);
    let mut lp = 0.0f32;
    let mut rng = vc_rng::rng::Rng::new(0x7A_0DE9);
    for i in 0..n {
        let t = i as f32 / RATE as f32;
        // the crack: first 80 ms louder + brighter
        let crack = (-(t * 18.0)).exp();
        // the rolling tail: two decays (near + far echo)
        let tail = (-(t * 0.9)).exp() * 0.6 + (-(t * 0.35)).exp() * 0.4;
        // 45 Hz sub thump with its own fast decay
        let sub = (2.0 * std::f32::consts::PI * 45.0 * t).sin() * (-(t * 2.2)).exp() * 0.8;
        // white noise -> aggressive one-pole low-pass (~150 Hz)
        let noise = rng.next_f32() * 2.0 - 1.0;
        let a = 150.0 / RATE as f32;
        lp += a * (noise - lp);
        let v = lp * (tail + crack * 2.0) * 1.6 + sub;
        out.push(v.clamp(-1.0, 1.0));
    }
    out
}

/// Backlog round (farming, 2026-09-09): the hoe-till "thock" — a
/// clean-room short soil-thump: a 90 Hz body with a fast decay, a
/// bright noise scrape for the blade cut, and a duller 55 Hz tail for
/// the soil settle. Not any vanilla recording; the recipe shape only.
fn hoe_till_recipe() -> Vec<f32> {
    let dur = 0.28;
    let n = (dur * RATE as f32) as usize;
    let mut out = Vec::with_capacity(n);
    let mut hp = 0.0f32;
    let mut rng = vc_rng::rng::Rng::new(0x0F_41D1);
    for i in 0..n {
        let t = i as f32 / RATE as f32;
        // the body: 90 Hz thump, gone in ~120 ms
        let body = (2.0 * std::f32::consts::PI * 90.0 * t).sin() * (-(t * 18.0)).exp();
        // the cut: high-passed noise burst (the blade through soil)
        let noise = rng.next_f32() * 2.0 - 1.0;
        hp += 0.35 * (noise - hp);
        let cut = (noise - hp) * (-(t * 30.0)).exp() * 0.8;
        // the settle: a 55 Hz tail under everything
        let settle = (2.0 * std::f32::consts::PI * 55.0 * t).sin() * (-(t * 6.0)).exp() * 0.5;
        let v = body * 1.1 + cut * 0.5 + settle;
        out.push(v.clamp(-1.0, 1.0));
    }
    out
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

// -------------------------------------------------- Sub-round 5 recipes --
// clean-room synthesis for the new event classes (container, XP orb,
// underwater, weather, music variants). shape language = the vanilla
// event's character (creak/clunk/chirp/pad); reference facts cited per
// recipe from minecraft.wiki (live 2026-09-15); no Mojang asset was
// read, copied, or traced.

/// wooden container OPEN: a creaky rise (the chest-lid character —
/// rising filtered noise + a wood thump).
fn chest_open_recipe() -> Vec<f32> {
    let mut c = one_pole_lp(
        noise_burst(2600, 211, 0.4, 0.12, 0.22),
        900.0,
    );
    mix_into(&mut c, &thump(140.0, 0.14, 0.5), 1.0);
    clamp_amp(c)
}

/// wooden container CLOSE: the creak in reverse + a firmer latch thump.
fn chest_close_recipe() -> Vec<f32> {
    let mut c = one_pole_lp(
        noise_burst(2000, 212, 0.45, 0.01, 0.16),
        800.0,
    );
    mix_into(&mut c, &thump(120.0, 0.1, 0.6), 1.0);
    clamp_amp(c)
}

/// iron door OPEN: a heavy metal clunk-slide.
fn iron_door_recipe() -> Vec<f32> {
    let mut d = thump(180.0, 0.12, 0.6);
    mix_into(&mut d, &ping(1300.0, 0.07, 0.3, 213), 1.0);
    mix_into(&mut d, &ping(1950.0, 0.05, 0.22, 214), 1.0);
    clamp_amp(d)
}

/// wooden door OPEN: the lighter clack.
fn wooden_door_recipe() -> Vec<f32> {
    let mut d = thump(160.0, 0.08, 0.5);
    mix_into(&mut d, &ping(800.0, 0.05, 0.25, 215), 1.0);
    clamp_amp(d)
}

/// shulker box: the peeling clatter (a quick metallic rattle).
fn shulker_recipe() -> Vec<f32> {
    let mut s = Vec::new();
    for k in 0..5u32 {
        let seg = ping(1500.0 + 260.0 * k as f32, 0.03, 0.24, 220 + k as u64);
        mix_into(&mut s, &seg, 1.0);
    }
    mix_into(&mut s, &thump(200.0, 0.09, 0.35), 1.0);
    clamp_amp(s)
}

/// entity.experience_orb.pickup — VERIFIED minecraft.wiki/w/Experience
/// (live 2026-09-15): volume 0.1, pitch 0.55–1.25, attenuation 16. A
/// bell-like FM chirp (carrier + a quick upward-blipped modulator).
fn xp_orb_recipe() -> Vec<f32> {
    let n = (0.16 * RATE as f32) as usize;
    let mut out = Vec::with_capacity(n);
    let mut rng = Rng::new(231);
    let f0 = 780.0 + rng.next_f32() * 140.0;
    for i in 0..n {
        let t = i as f32 / RATE as f32;
        let env = (-t * 16.0).exp();
        // FM chirp: quick upward sweep on the modulator
        let mod_f = 2.0 + 6.0 * (t / 0.16).min(1.0);
        let v = (2.0 * std::f32::consts::PI * f0 * t).sin()
            * (2.0 * std::f32::consts::PI * mod_f * t).cos();
        out.push(v * env * 0.5);
    }
    clamp_amp(out)
}

/// underwater ENTER: a muffled plunge (low-passed splash).
fn water_enter_recipe() -> Vec<f32> {
    let mut w = one_pole_lp(
        noise_burst(4200, 241, 0.5, 0.01, 0.3),
        500.0,
    );
    mix_into(&mut w, &thump(90.0, 0.2, 0.4), 1.0);
    clamp_amp(w)
}

/// underwater EXIT: the brighter emerge splash.
fn water_exit_recipe() -> Vec<f32> {
    let mut w = one_pole_lp(
        noise_burst(3200, 242, 0.5, 0.005, 0.22),
        1600.0,
    );
    mix_into(&mut w, &ping(600.0, 0.05, 0.2, 243), 1.0);
    clamp_amp(w)
}

/// the underwater LOOP bed (one ~2 s tile, low-passed bubble wash).
fn water_loop_recipe() -> Vec<f32> {
    let mut w = one_pole_lp(
        noise_burst((2.0 * RATE as f32) as usize, 244, 0.16, 0.4, 1.6),
        380.0,
    );
    // slow bubble blips riding the wash
    for k in 0..6u32 {
        let blip = ping(300.0 + 90.0 * k as f32, 0.05, 0.08, 245 + k as u64);
        mix_into(&mut w, &blip, 1.0);
    }
    clamp_amp(w)
}

/// weather.rain: a one-second rain tile (dense soft high noise) — the
/// game layer loops it while rain is active.
fn rain_recipe() -> Vec<f32> {
    let mut r = one_pole_hp(
        noise_burst(RATE as usize, 251, 0.22, 0.15, 0.85),
        900.0,
    );
    r = one_pole_lp(r, 3400.0);
    clamp_amp(r)
}

/// entity.lightning_bolt.impact: the sharp crack before the thunder
/// roll (a hard noise snap + a sub thump).
fn lightning_impact_recipe() -> Vec<f32> {
    let mut l = one_pole_hp(
        noise_burst(1400, 261, 0.8, 0.001, 0.05),
        700.0,
    );
    mix_into(&mut l, &thump(60.0, 0.25, 0.7), 1.0);
    clamp_amp(l)
}

/// music.menu: the title-screen pad (a calm major-ish drone, warmer
/// than the gameplay pads).
fn music_pad_menu() -> Vec<f32> {
    let n = (6.0 * RATE as f32) as usize;
    let mut out = vec![0.0f32; n];
    // a soft triad-ish stack with slow tremolo
    for (f, g) in [(196.0, 0.16), (247.0, 0.12), (294.0, 0.1), (392.0, 0.07)] {
        let tone = ping(f, 6.0, g, 271 + f as u64);
        mix_into(&mut out, &tone, 1.0);
    }
    let mut rng = Rng::new(272);
    for s in out.iter_mut() {
        *s *= 0.8 + 0.2 * rng.next_f32();
    }
    clamp_amp(out)
}

/// music.creative: a brighter, slightly faster pad (the creative-mode
/// track character).
fn music_pad_creative() -> Vec<f32> {
    let n = (5.0 * RATE as f32) as usize;
    let mut out = vec![0.0f32; n];
    for (f, g) in [(262.0, 0.14), (330.0, 0.12), (392.0, 0.12), (523.0, 0.08)] {
        let tone = ping(f, 5.0, g, 281 + f as u64);
        mix_into(&mut out, &tone, 1.0);
    }
    clamp_amp(out)
}

/// music.under_water: a heavily muffled, slow pad (the underwater
/// track's drowsy character).
fn music_pad_underwater() -> Vec<f32> {
    let n = (7.0 * RATE as f32) as usize;
    let mut out = vec![0.0f32; n];
    for (f, g) in [(147.0, 0.18), (185.0, 0.12), (220.0, 0.1)] {
        let tone = ping(f, 7.0, g, 291 + f as u64);
        mix_into(&mut out, &tone, 1.0);
    }
    let lp = one_pole_lp(out, 700.0);
    clamp_amp(lp)
}

/// music.nether: a dark, dissonant-leaning pad (the nether track's
/// ominous character).
fn music_pad_nether() -> Vec<f32> {
    let n = (6.0 * RATE as f32) as usize;
    let mut out = vec![0.0f32; n];
    for (f, g) in [(139.0, 0.17), (208.0, 0.11), (277.0, 0.09), (415.0, 0.05)] {
        let tone = ping(f, 6.0, g, 301 + f as u64);
        mix_into(&mut out, &tone, 1.0);
    }
    let mut rng = Rng::new(302);
    for s in out.iter_mut() {
        *s *= 0.75 + 0.25 * rng.next_f32();
    }
    clamp_amp(out)
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
                SoundFamily::Gravel => "gravel",
                SoundFamily::Metal => "metal",
                SoundFamily::Plant => "plant",
                SoundFamily::Chain => "chain",
                SoundFamily::NetherWood => "nether_wood",
            }
        };
        // Sub-round 5: the fourteen LAND families (water keeps its splash
        // special). Each gets the five vanilla material-event takes:
        // two dig/break variants, a place take, the light mining-cadence
        // hit, the soft step, and the heavy fall.
        let families = [
            SoundFamily::Grass,
            SoundFamily::Dirt,
            SoundFamily::Stone,
            SoundFamily::Wood,
            SoundFamily::Sand,
            SoundFamily::Leaves,
            SoundFamily::Glass,
            SoundFamily::Wool,
            SoundFamily::Gravel,
            SoundFamily::Metal,
            SoundFamily::Plant,
            SoundFamily::Chain,
            SoundFamily::NetherWood,
        ];
        for f in families {
            for v in 0..2 {
                names.push(format!("dig/{}{}", fam_name(f), v + 1));
                data.push(family_recipe(f, v, Take::Dig));
            }
            names.push(format!("place/{}", fam_name(f)));
            data.push(family_recipe(f, 2, Take::Dig));
            names.push(format!("hit/{}", fam_name(f)));
            data.push(family_recipe(f, 3, Take::Hit));
            names.push(format!("step/{}", fam_name(f)));
            data.push(family_recipe(f, 0, Take::Step));
            names.push(format!("fall/{}", fam_name(f)));
            data.push(family_recipe(f, 1, Take::Fall));
        }
        // one-off recipes
        for (n, d) in [
            ("ui/click", click_recipe()),
            ("entity/item/pickup", pop_recipe()),
            ("block/lever", lever_recipe()),
            ("ambient/eerie", eerie_recipe()),
            ("ambient/thunder", thunder_recipe()),
            // backlog round (farming): the hoe-till thock
            ("item/hoe/till", hoe_till_recipe()),
            ("music/pad_day", music_pad(false)),
            ("music/pad_night", music_pad(true)),
            ("block/brewing_bubble", bubble_recipe()),
            ("entity/drink", drink_recipe()),
            ("liquid/splash", splash_recipe()),
            ("block/enchant_use", enchant_recipe()),
            ("entity/levelup", levelup_recipe()),
            ("entity/villager_ambient", villager_ambient_recipe()),
            ("entity/villager_trade", villager_trade_recipe()),
            // Phase 2: combat + mob sounds
            ("entity/player/hurt", hurt_recipe()),
            ("entity/zombie/ambient", zombie_groan_recipe()),
            ("entity/generic/death", death_recipe()),
            ("entity/creeper/priming", priming_recipe()),
            ("entity/explode", explosion_recipe()),
            ("entity/skeleton/shoot", bow_recipe()),
            ("entity/cow/ambient", cow_recipe()),
            ("entity/pig/ambient", pig_recipe()),
            ("entity/sheep/ambient", sheep_recipe()),
            ("entity/chicken/ambient", chicken_recipe()),
            // ---- Sub-round 5: containers, XP orb, underwater, weather,
            // music variants ----
            ("block/chest_open", chest_open_recipe()),
            ("block/chest_close", chest_close_recipe()),
            ("block/iron_door", iron_door_recipe()),
            ("block/wooden_door", wooden_door_recipe()),
            ("block/shulker", shulker_recipe()),
            ("entity/xp_orb", xp_orb_recipe()),
            ("ambient/water_enter", water_enter_recipe()),
            ("ambient/water_exit", water_exit_recipe()),
            ("ambient/water_loop", water_loop_recipe()),
            ("weather/rain", rain_recipe()),
            ("weather/rain_above", rain_recipe()),
            ("ambient/lightning_impact", lightning_impact_recipe()),
            ("music/pad_menu", music_pad_menu()),
            ("music/pad_creative", music_pad_creative()),
            ("music/pad_underwater", music_pad_underwater()),
            ("music/pad_nether", music_pad_nether()),
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
    /// Sub-round 5: vanilla's tenth category ("Voice/Speech", added
    /// 16w02a = 1.9; the narrator/maps lane — no engine events ride it
    /// yet, but the Music & Sound screen carries its slider like
    /// vanilla's)
    Voice,
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
            "voice" => Self::Voice,
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
            Self::Voice => "voice",
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
  "block.grass.break": {"category": "blocks", "pitch": [0.8, 1.1], "sounds": [{"name": "dig/grass1", "weight": 2}, {"name": "dig/grass2", "weight": 1}]},
  "block.grass.place": {"category": "blocks", "volume": 0.8, "pitch": [0.9, 1.1], "sounds": [{"name": "place/grass"}]},
  "block.grass.hit": {"category": "blocks", "volume": 0.32, "pitch": [0.85, 1.15], "sounds": [{"name": "hit/grass"}]},
  "block.grass.step": {"category": "blocks", "volume": 0.3, "pitch": [0.9, 1.05], "sounds": [{"name": "step/grass"}]},
  "block.grass.fall": {"category": "blocks", "volume": 0.6, "pitch": [0.85, 1.05], "sounds": [{"name": "fall/grass"}]},
  "block.dirt.break": {"category": "blocks", "pitch": [0.8, 1.1], "sounds": [{"name": "dig/dirt1", "weight": 2}, {"name": "dig/dirt2", "weight": 1}]},
  "block.dirt.place": {"category": "blocks", "volume": 0.8, "pitch": [0.9, 1.1], "sounds": [{"name": "place/dirt"}]},
  "block.dirt.hit": {"category": "blocks", "volume": 0.32, "pitch": [0.85, 1.15], "sounds": [{"name": "hit/dirt"}]},
  "block.dirt.step": {"category": "blocks", "volume": 0.3, "pitch": [0.9, 1.05], "sounds": [{"name": "step/dirt"}]},
  "block.dirt.fall": {"category": "blocks", "volume": 0.6, "pitch": [0.85, 1.05], "sounds": [{"name": "fall/dirt"}]},
  "block.stone.break": {"category": "blocks", "pitch": [0.8, 1.0], "sounds": [{"name": "dig/stone1", "weight": 2}, {"name": "dig/stone2", "weight": 1}]},
  "block.stone.place": {"category": "blocks", "volume": 0.8, "pitch": [0.9, 1.1], "sounds": [{"name": "place/stone"}]},
  "block.stone.hit": {"category": "blocks", "volume": 0.32, "pitch": [0.85, 1.15], "sounds": [{"name": "hit/stone"}]},
  "block.stone.step": {"category": "blocks", "volume": 0.3, "pitch": [0.9, 1.05], "sounds": [{"name": "step/stone"}]},
  "block.stone.fall": {"category": "blocks", "volume": 0.6, "pitch": [0.85, 1.05], "sounds": [{"name": "fall/stone"}]},
  "block.wood.break": {"category": "blocks", "pitch": [0.85, 1.1], "sounds": [{"name": "dig/wood1", "weight": 2}, {"name": "dig/wood2", "weight": 1}]},
  "block.wood.place": {"category": "blocks", "volume": 0.8, "pitch": [0.9, 1.1], "sounds": [{"name": "place/wood"}]},
  "block.wood.hit": {"category": "blocks", "volume": 0.32, "pitch": [0.85, 1.15], "sounds": [{"name": "hit/wood"}]},
  "block.wood.step": {"category": "blocks", "volume": 0.3, "pitch": [0.9, 1.05], "sounds": [{"name": "step/wood"}]},
  "block.wood.fall": {"category": "blocks", "volume": 0.6, "pitch": [0.85, 1.05], "sounds": [{"name": "fall/wood"}]},
  "block.sand.break": {"category": "blocks", "pitch": [0.9, 1.15], "sounds": [{"name": "dig/sand1", "weight": 2}, {"name": "dig/sand2", "weight": 1}]},
  "block.sand.place": {"category": "blocks", "volume": 0.8, "pitch": [0.9, 1.1], "sounds": [{"name": "place/sand"}]},
  "block.sand.hit": {"category": "blocks", "volume": 0.32, "pitch": [0.85, 1.15], "sounds": [{"name": "hit/sand"}]},
  "block.sand.step": {"category": "blocks", "volume": 0.3, "pitch": [0.9, 1.05], "sounds": [{"name": "step/sand"}]},
  "block.sand.fall": {"category": "blocks", "volume": 0.6, "pitch": [0.85, 1.05], "sounds": [{"name": "fall/sand"}]},
  "block.leaves.break": {"category": "blocks", "pitch": [0.9, 1.15], "sounds": [{"name": "dig/leaves1", "weight": 2}, {"name": "dig/leaves2", "weight": 1}]},
  "block.leaves.place": {"category": "blocks", "volume": 0.8, "pitch": [0.9, 1.1], "sounds": [{"name": "place/leaves"}]},
  "block.leaves.hit": {"category": "blocks", "volume": 0.32, "pitch": [0.85, 1.15], "sounds": [{"name": "hit/leaves"}]},
  "block.leaves.step": {"category": "blocks", "volume": 0.3, "pitch": [0.9, 1.05], "sounds": [{"name": "step/leaves"}]},
  "block.leaves.fall": {"category": "blocks", "volume": 0.6, "pitch": [0.85, 1.05], "sounds": [{"name": "fall/leaves"}]},
  "block.glass.break": {"category": "blocks", "pitch": [0.85, 1.1], "sounds": [{"name": "dig/glass1", "weight": 2}, {"name": "dig/glass2", "weight": 1}]},
  "block.glass.place": {"category": "blocks", "volume": 0.8, "pitch": [0.9, 1.1], "sounds": [{"name": "place/glass"}]},
  "block.glass.hit": {"category": "blocks", "volume": 0.32, "pitch": [0.85, 1.15], "sounds": [{"name": "hit/glass"}]},
  "block.glass.step": {"category": "blocks", "volume": 0.3, "pitch": [0.9, 1.05], "sounds": [{"name": "step/glass"}]},
  "block.glass.fall": {"category": "blocks", "volume": 0.6, "pitch": [0.85, 1.05], "sounds": [{"name": "fall/glass"}]},
  "block.wool.break": {"category": "blocks", "pitch": [0.85, 1.05], "sounds": [{"name": "dig/wool1", "weight": 2}, {"name": "dig/wool2", "weight": 1}]},
  "block.wool.place": {"category": "blocks", "volume": 0.8, "pitch": [0.9, 1.1], "sounds": [{"name": "place/wool"}]},
  "block.wool.hit": {"category": "blocks", "volume": 0.32, "pitch": [0.85, 1.15], "sounds": [{"name": "hit/wool"}]},
  "block.wool.step": {"category": "blocks", "volume": 0.3, "pitch": [0.9, 1.05], "sounds": [{"name": "step/wool"}]},
  "block.wool.fall": {"category": "blocks", "volume": 0.6, "pitch": [0.85, 1.05], "sounds": [{"name": "fall/wool"}]},
  "block.gravel.break": {"category": "blocks", "pitch": [0.85, 1.1], "sounds": [{"name": "dig/gravel1", "weight": 2}, {"name": "dig/gravel2", "weight": 1}]},
  "block.gravel.place": {"category": "blocks", "volume": 0.8, "pitch": [0.9, 1.1], "sounds": [{"name": "place/gravel"}]},
  "block.gravel.hit": {"category": "blocks", "volume": 0.32, "pitch": [0.85, 1.15], "sounds": [{"name": "hit/gravel"}]},
  "block.gravel.step": {"category": "blocks", "volume": 0.3, "pitch": [0.9, 1.05], "sounds": [{"name": "step/gravel"}]},
  "block.gravel.fall": {"category": "blocks", "volume": 0.6, "pitch": [0.85, 1.05], "sounds": [{"name": "fall/gravel"}]},
  "block.metal.break": {"category": "blocks", "pitch": [0.9, 1.1], "sounds": [{"name": "dig/metal1", "weight": 2}, {"name": "dig/metal2", "weight": 1}]},
  "block.metal.place": {"category": "blocks", "volume": 0.8, "pitch": [0.9, 1.1], "sounds": [{"name": "place/metal"}]},
  "block.metal.hit": {"category": "blocks", "volume": 0.32, "pitch": [0.85, 1.15], "sounds": [{"name": "hit/metal"}]},
  "block.metal.step": {"category": "blocks", "volume": 0.3, "pitch": [0.9, 1.05], "sounds": [{"name": "step/metal"}]},
  "block.metal.fall": {"category": "blocks", "volume": 0.6, "pitch": [0.85, 1.05], "sounds": [{"name": "fall/metal"}]},
  "block.plant.break": {"category": "blocks", "pitch": [0.9, 1.15], "sounds": [{"name": "dig/plant1", "weight": 2}, {"name": "dig/plant2", "weight": 1}]},
  "block.plant.place": {"category": "blocks", "volume": 0.8, "pitch": [0.9, 1.1], "sounds": [{"name": "place/plant"}]},
  "block.plant.hit": {"category": "blocks", "volume": 0.32, "pitch": [0.85, 1.15], "sounds": [{"name": "hit/plant"}]},
  "block.plant.step": {"category": "blocks", "volume": 0.3, "pitch": [0.9, 1.05], "sounds": [{"name": "step/plant"}]},
  "block.plant.fall": {"category": "blocks", "volume": 0.6, "pitch": [0.85, 1.05], "sounds": [{"name": "fall/plant"}]},
  "block.chain.break": {"category": "blocks", "pitch": [0.9, 1.1], "sounds": [{"name": "dig/chain1", "weight": 2}, {"name": "dig/chain2", "weight": 1}]},
  "block.chain.place": {"category": "blocks", "volume": 0.8, "pitch": [0.9, 1.1], "sounds": [{"name": "place/chain"}]},
  "block.chain.hit": {"category": "blocks", "volume": 0.32, "pitch": [0.85, 1.15], "sounds": [{"name": "hit/chain"}]},
  "block.chain.step": {"category": "blocks", "volume": 0.3, "pitch": [0.9, 1.05], "sounds": [{"name": "step/chain"}]},
  "block.chain.fall": {"category": "blocks", "volume": 0.6, "pitch": [0.85, 1.05], "sounds": [{"name": "fall/chain"}]},
  "block.nether_wood.break": {"category": "blocks", "pitch": [0.85, 1.05], "sounds": [{"name": "dig/nether_wood1", "weight": 2}, {"name": "dig/nether_wood2", "weight": 1}]},
  "block.nether_wood.place": {"category": "blocks", "volume": 0.8, "pitch": [0.9, 1.1], "sounds": [{"name": "place/nether_wood"}]},
  "block.nether_wood.hit": {"category": "blocks", "volume": 0.32, "pitch": [0.85, 1.15], "sounds": [{"name": "hit/nether_wood"}]},
  "block.nether_wood.step": {"category": "blocks", "volume": 0.3, "pitch": [0.9, 1.05], "sounds": [{"name": "step/nether_wood"}]},
  "block.nether_wood.fall": {"category": "blocks", "volume": 0.6, "pitch": [0.85, 1.05], "sounds": [{"name": "fall/nether_wood"}]},
  "block.water.splash": {"category": "blocks", "volume": 0.8, "pitch": [0.8, 1.2], "attenuation": 12, "sounds": [{"name": "dig/water1", "weight": 2}, {"name": "dig/water2", "weight": 1}]},
  "block.water.step": {"category": "blocks", "volume": 0.4, "pitch": [0.8, 1.2], "sounds": [{"name": "step/water"}]},
  "block.lever.click": {"category": "blocks", "volume": 0.6, "pitch": [0.9, 1.1], "sounds": [{"name": "block/lever"}]},
  "ui.click": {"category": "players", "volume": 0.35, "pitch": [1.5, 1.7], "sounds": [{"name": "ui/click"}]},
  "entity.item.pickup": {"category": "players", "volume": 0.45, "pitch": [0.9, 1.3], "sounds": [{"name": "entity/item/pickup"}]},
  "ambient.eerie": {"category": "ambient", "volume": 0.55, "pitch": [0.85, 1.3], "attenuation": 0, "sounds": [{"name": "ambient/eerie"}]},
  "ambient.thunder": {"category": "weather", "volume": 1.0, "pitch": [0.9, 1.1], "attenuation": 0, "sounds": [{"name": "ambient/thunder"}]},
  "item.hoe.till": {"category": "blocks", "volume": 0.8, "pitch": [0.9, 1.15], "attenuation": 12, "sounds": [{"name": "item/hoe/till"}]},
  "music.pad.day": {"category": "music", "volume": 0.5, "sounds": [{"name": "music/pad_day", "stream": true}]},
  "music.pad.night": {"category": "music", "volume": 0.5, "sounds": [{"name": "music/pad_night", "stream": true}]},
  "block.brewing_stand.bubble": {"category": "blocks", "volume": 0.7, "pitch": [0.9, 1.15], "attenuation": 12, "sounds": [{"name": "block/brewing_bubble"}]},
  "entity.generic.drink": {"category": "players", "volume": 0.6, "pitch": [0.9, 1.1], "sounds": [{"name": "entity/drink"}]},
  "liquid.splash": {"category": "blocks", "volume": 0.6, "pitch": [0.9, 1.2], "attenuation": 12, "sounds": [{"name": "liquid/splash"}]},
  "block.enchantment_table.use": {"category": "blocks", "volume": 0.8, "pitch": [0.9, 1.1], "attenuation": 12, "sounds": [{"name": "block/enchant_use"}]},
  "entity.player.levelup": {"category": "players", "volume": 0.75, "pitch": [1.0, 1.0], "sounds": [{"name": "entity/levelup"}]},
  "entity.villager.ambient": {"category": "neutral", "volume": 0.55, "pitch": [0.85, 1.15], "attenuation": 12, "sounds": [{"name": "entity/villager_ambient"}]},
  "entity.villager.trade": {"category": "neutral", "volume": 0.7, "pitch": [0.9, 1.1], "attenuation": 12, "sounds": [{"name": "entity/villager_trade"}]},
  "entity.player.hurt": {"category": "players", "volume": 0.8, "pitch": [0.9, 1.15], "sounds": [{"name": "entity/player/hurt"}]},
  "entity.zombie.ambient": {"category": "hostile", "volume": 0.6, "pitch": [0.8, 1.1], "attenuation": 14, "sounds": [{"name": "entity/zombie/ambient"}]},
  "entity.generic.death": {"category": "hostile", "volume": 0.7, "pitch": [0.85, 1.1], "attenuation": 14, "sounds": [{"name": "entity/generic/death"}]},
  "entity.creeper.priming": {"category": "hostile", "volume": 0.9, "pitch": [1.0, 1.0], "attenuation": 16, "sounds": [{"name": "entity/creeper/priming"}]},
  "entity.generic.explode": {"category": "hostile", "volume": 1.0, "pitch": [0.9, 1.1], "attenuation": 24, "sounds": [{"name": "entity/explode"}]},
  "entity.skeleton.shoot": {"category": "hostile", "volume": 0.6, "pitch": [0.9, 1.1], "attenuation": 16, "sounds": [{"name": "entity/skeleton/shoot"}]},
  "entity.cow.ambient": {"category": "neutral", "volume": 0.55, "pitch": [0.9, 1.1], "attenuation": 14, "sounds": [{"name": "entity/cow/ambient"}]},
  "entity.pig.ambient": {"category": "neutral", "volume": 0.55, "pitch": [0.9, 1.1], "attenuation": 14, "sounds": [{"name": "entity/pig/ambient"}]},
  "entity.sheep.ambient": {"category": "neutral", "volume": 0.55, "pitch": [0.9, 1.1], "attenuation": 14, "sounds": [{"name": "entity/sheep/ambient"}]},
  "entity.chicken.ambient": {"category": "neutral", "volume": 0.5, "pitch": [0.95, 1.15], "attenuation": 12, "sounds": [{"name": "entity/chicken/ambient"}]},
  "block.chest.open": {"category": "blocks", "volume": 0.7, "pitch": [0.9, 1.0], "attenuation": 12, "sounds": [{"name": "block/chest_open"}]},
  "block.chest.close": {"category": "blocks", "volume": 0.7, "pitch": [0.9, 1.0], "attenuation": 12, "sounds": [{"name": "block/chest_close"}]},
  "block.ender_chest.open": {"category": "blocks", "volume": 0.6, "pitch": [1.1, 1.3], "attenuation": 12, "sounds": [{"name": "block/chest_open"}]},
  "block.ender_chest.close": {"category": "blocks", "volume": 0.6, "pitch": [1.1, 1.3], "attenuation": 12, "sounds": [{"name": "block/chest_close"}]},
  "block.barrel.open": {"category": "blocks", "volume": 0.7, "pitch": [0.9, 1.0], "attenuation": 12, "sounds": [{"name": "block/chest_open"}]},
  "block.barrel.close": {"category": "blocks", "volume": 0.7, "pitch": [0.9, 1.0], "attenuation": 12, "sounds": [{"name": "block/chest_close"}]},
  "block.shulker_box.open": {"category": "blocks", "volume": 0.6, "pitch": [0.9, 1.1], "attenuation": 12, "sounds": [{"name": "block/shulker"}]},
  "block.shulker_box.close": {"category": "blocks", "volume": 0.6, "pitch": [0.8, 1.0], "attenuation": 12, "sounds": [{"name": "block/shulker"}]},
  "block.iron_door.open": {"category": "blocks", "volume": 0.7, "pitch": [0.9, 1.0], "attenuation": 12, "sounds": [{"name": "block/iron_door"}]},
  "block.iron_door.close": {"category": "blocks", "volume": 0.7, "pitch": [0.85, 0.95], "attenuation": 12, "sounds": [{"name": "block/iron_door"}]},
  "block.wooden_door.open": {"category": "blocks", "volume": 0.7, "pitch": [0.9, 1.0], "attenuation": 12, "sounds": [{"name": "block/wooden_door"}]},
  "block.wooden_door.close": {"category": "blocks", "volume": 0.7, "pitch": [0.85, 0.95], "attenuation": 12, "sounds": [{"name": "block/wooden_door"}]},
  "entity.experience_orb.pickup": {"category": "players", "volume": 0.35, "pitch": [0.55, 1.25], "attenuation": 16, "sounds": [{"name": "entity/xp_orb"}]},
  "ambient.cave": {"category": "ambient", "volume": 0.6, "pitch": [0.85, 1.3], "attenuation": 0, "sounds": [{"name": "ambient/eerie"}]},
  "ambient.underwater.enter": {"category": "weather", "volume": 0.6, "pitch": [0.9, 1.1], "attenuation": 0, "sounds": [{"name": "ambient/water_enter"}]},
  "ambient.underwater.exit": {"category": "weather", "volume": 0.6, "pitch": [0.9, 1.1], "attenuation": 0, "sounds": [{"name": "ambient/water_exit"}]},
  "ambient.underwater.loop": {"category": "ambient", "volume": 0.4, "pitch": [0.95, 1.05], "attenuation": 0, "sounds": [{"name": "ambient/water_loop", "stream": true}]},
  "weather.rain": {"category": "weather", "volume": 0.35, "pitch": [0.95, 1.05], "attenuation": 0, "sounds": [{"name": "weather/rain"}]},
  "weather.rain.above": {"category": "weather", "volume": 0.28, "pitch": [0.95, 1.05], "attenuation": 0, "sounds": [{"name": "weather/rain_above"}]},
  "entity.lightning_bolt.thunder": {"category": "weather", "volume": 1.0, "pitch": [0.9, 1.1], "attenuation": 0, "sounds": [{"name": "ambient/thunder"}]},
  "entity.lightning_bolt.impact": {"category": "weather", "volume": 0.9, "pitch": [0.9, 1.1], "attenuation": 0, "sounds": [{"name": "ambient/lightning_impact"}]},
  "music.menu": {"category": "music", "volume": 0.5, "sounds": [{"name": "music/pad_menu", "stream": true}]},
  "music.game": {"category": "music", "volume": 0.5, "sounds": [{"name": "music/pad_day", "stream": true}]},
  "music.creative": {"category": "music", "volume": 0.5, "sounds": [{"name": "music/pad_creative", "stream": true}]},
  "music.under_water": {"category": "music", "volume": 0.5, "sounds": [{"name": "music/pad_underwater", "stream": true}]},
  "music.nether": {"category": "music", "volume": 0.5, "sounds": [{"name": "music/pad_nether", "stream": true}]}
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

/// Sub-round 5 (2026-09-15): the five vanilla material events, mapped
/// from a block SoundFamily. VERIFIED structure against vanilla 1.16.5
/// sounds.json (minecraft.wiki/w/Sounds.json, live 2026-09-15): every
/// material class carries block.<mat>.break / .place / .hit / .step /
/// .fall. Glass breaks (no plain dig); water splashes.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FamilyEvent {
    Break,
    Place,
    Hit,
    Step,
    Fall,
}

/// map a block SoundFamily + event kind to the registry event name
/// (static strings — no allocation on the hot footstep path).
pub fn family_event(f: SoundFamily, kind: FamilyEvent) -> &'static str {
    use FamilyEvent as FE;
    use SoundFamily as SF;
    // rows: (family, break, place, hit, step, fall)
    const TABLE: &[(SF, &str, &str, &str, &str, &str)] = &[
        (SF::Grass, "block.grass.break", "block.grass.place", "block.grass.hit", "block.grass.step", "block.grass.fall"),
        (SF::Dirt, "block.dirt.break", "block.dirt.place", "block.dirt.hit", "block.dirt.step", "block.dirt.fall"),
        (SF::Stone, "block.stone.break", "block.stone.place", "block.stone.hit", "block.stone.step", "block.stone.fall"),
        (SF::Wood, "block.wood.break", "block.wood.place", "block.wood.hit", "block.wood.step", "block.wood.fall"),
        (SF::Sand, "block.sand.break", "block.sand.place", "block.sand.hit", "block.sand.step", "block.sand.fall"),
        (SF::Leaves, "block.leaves.break", "block.leaves.place", "block.leaves.hit", "block.leaves.step", "block.leaves.fall"),
        (SF::Glass, "block.glass.break", "block.glass.place", "block.glass.hit", "block.glass.step", "block.glass.fall"),
        (SF::Wool, "block.wool.break", "block.wool.place", "block.wool.hit", "block.wool.step", "block.wool.fall"),
        (SF::Gravel, "block.gravel.break", "block.gravel.place", "block.gravel.hit", "block.gravel.step", "block.gravel.fall"),
        (SF::Metal, "block.metal.break", "block.metal.place", "block.metal.hit", "block.metal.step", "block.metal.fall"),
        (SF::Plant, "block.plant.break", "block.plant.place", "block.plant.hit", "block.plant.step", "block.plant.fall"),
        (SF::Chain, "block.chain.break", "block.chain.place", "block.chain.hit", "block.chain.step", "block.chain.fall"),
        (SF::NetherWood, "block.nether_wood.break", "block.nether_wood.place", "block.nether_wood.hit", "block.nether_wood.step", "block.nether_wood.fall"),
        (SF::Water, "block.water.splash", "block.water.splash", "block.water.splash", "block.water.step", "block.water.splash"),
        (SF::None, "block.stone.break", "block.stone.place", "block.stone.hit", "block.stone.step", "block.stone.fall"),
    ];
    for &(fam, brk, place, hit, step, fall) in TABLE {
        if fam == f {
            return match kind {
                FE::Break => brk,
                FE::Place => place,
                FE::Hit => hit,
                FE::Step => step,
                FE::Fall => fall,
            };
        }
    }
    "block.stone.break"
}

// ------------------------------------------------------------- backends --

pub trait AudioBackend {
    /// play one bank slot with gain, pitch and stereo pan (−1 left … +1
    /// right). All §21 mixing (category gains, attenuation) happens in the
    /// caller; backends just render.
    fn play(&self, bank: &SoundBank, slot: usize, volume: f32, pitch: f32, pan: f32);
    /// unlock audio context (wasm: needs user gesture; decodes the bank)
    fn unlock(&self, _bank: &SoundBank) {}
    /// Sub-round 5: the underwater muffle — every subsequent sound plays
    /// through a low-pass (vanilla routes the master bus through a
    /// low-pass filter while the camera is underwater; ~500 Hz biquad
    /// in the Web backend, the one-pole 500 Hz equivalent in native).
    fn set_underwater(&self, _on: bool) {}
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
        /// Sub-round 5: the underwater muffle flag
        underwater: std::cell::Cell<bool>,
    }

    impl RodioOut {
        pub fn new() -> Option<Self> {
            match rodio::OutputStream::try_default() {
                Ok((stream, handle)) => Some(RodioOut {
                    _stream: stream,
                    handle,
                    underwater: std::cell::Cell::new(false),
                }),
                Err(_) => None,
            }
        }
    }

    impl AudioBackend for RodioOut {
        fn play(&self, bank: &SoundBank, slot: usize, volume: f32, pitch: f32, pan: f32) {
            let Some(base) = bank.data.get(slot) else { return };
            let mut samples = resample(base, pitch);
            // Sub-round 5: the underwater muffle (one-pole 500 Hz — the
            // native stand-in for vanilla's low-pass master bus)
            if self.underwater.get() {
                samples = one_pole_lp(samples, 500.0);
            }
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

        fn set_underwater(&self, on: bool) {
            self.underwater.set(on);
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
        /// Sub-round 5: the underwater muffle flag
        underwater: std::cell::Cell<bool>,
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
                    underwater: std::cell::Cell::new(false),
                }),
            }
        }
    }

    impl Default for WebAudioOut {
        fn default() -> Self {
            Self::new()
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
                // chain: source → gain → [muffle biquad] → panner →
                // destination. Sub-round 5: while underwater every sound
                // routes through a 500 Hz low-pass (vanilla's underwater
                // master-bus filter — the §E ask).
                let muffled = self.inner.underwater.get();
                if let Ok(gain) = web_sys::AudioContext::create_gain(ctx) {
                    let g = web_sys::GainNode::gain(&gain);
                    web_sys::AudioParam::set_value(&g, volume.clamp(0.0, 1.0));
                    let panner = web_sys::AudioContext::create_stereo_panner(ctx).ok();
                    let filter = if muffled {
                        web_sys::AudioContext::create_biquad_filter(ctx).ok()
                    } else {
                        None
                    };
                    if let Some(f) = filter.as_ref() {
                        let _ = web_sys::BiquadFilterNode::set_type(
                            f,
                            web_sys::BiquadFilterType::Lowpass,
                        );
                        let freq = web_sys::BiquadFilterNode::frequency(f);
                        web_sys::AudioParam::set_value(&freq, 500.0);
                    }
                    // wire: src → gain → [filter] → [panner] → dest
                    let _ = web_sys::AudioNode::connect_with_audio_node(&src, &gain);
                    let mut node: &web_sys::AudioNode = &gain;
                    if let Some(f) = filter.as_ref() {
                        let _ = web_sys::AudioNode::connect_with_audio_node(node, f);
                        node = f;
                    }
                    if let Some(pn) = panner.as_ref() {
                        let p = web_sys::StereoPannerNode::pan(pn);
                        web_sys::AudioParam::set_value(&p, pan.clamp(-1.0, 1.0));
                        let _ = web_sys::AudioNode::connect_with_audio_node(node, pn);
                        node = pn;
                    }
                    let _ = web_sys::AudioNode::connect_with_audio_node(node, &dest);
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

        fn set_underwater(&self, on: bool) {
            self.inner.underwater.set(on);
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
