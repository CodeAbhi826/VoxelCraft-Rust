//! 1.16.5-style sound synthesis. Every sound is generated from scratch
//! (filtered noise bursts + decaying sines) at startup — zero asset files.
//! Backends: rodio (native), WebAudio (wasm), silent fallback (no-audio build).

use crate::blocks::SoundFamily;
use crate::rng::Rng;

pub const RATE: u32 = 22050;

pub struct SoundBank {
    /// indexed by family (see family_index), last entry = splash
    pub data: Vec<Vec<f32>>,
    /// WAV-encoded copies (for WebAudio decode)
    pub wavs: Vec<Vec<u8>>,
}

pub fn family_index(f: SoundFamily) -> usize {
    match f {
        SoundFamily::Grass => 0,
        SoundFamily::Dirt => 1,
        SoundFamily::Stone => 2,
        SoundFamily::Wood => 3,
        SoundFamily::Sand => 4,
        SoundFamily::Leaves => 5,
        SoundFamily::Glass => 6,
        SoundFamily::Water | SoundFamily::None => 7, // splash
    }
}

pub const SPLASH: usize = 7;
pub const BANK_LEN: usize = 8;

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

impl SoundBank {
    pub fn generate() -> Self {
        let mut data: Vec<Vec<f32>> = Vec::with_capacity(BANK_LEN);

        // grass dig: soft low noise, layered
        let mut g = one_pole_lp(noise_burst(3600, 11, 0.5, 0.004, 0.14), 1100.0);
        mix_into(&mut g, &one_pole_lp(noise_burst(1200, 12, 0.25, 0.002, 0.05), 2400.0), 1.0);
        data.push(clamp_amp(g));

        // dirt dig: darker, shorter
        data.push(clamp_amp(one_pole_lp(noise_burst(3200, 21, 0.55, 0.004, 0.12), 700.0)));

        // stone dig: sharp bandpassed snap + click
        let mut s = one_pole_hp(one_pole_lp(noise_burst(2200, 31, 0.5, 0.002, 0.09), 5500.0), 1400.0);
        mix_into(&mut s, &ping(2400.0, 0.03, 0.35, 32), 1.0);
        data.push(clamp_amp(s));

        // wood dig: hollow knock
        let mut w = thump(175.0, 0.13, 0.55);
        mix_into(&mut w, &one_pole_lp(noise_burst(1400, 41, 0.3, 0.002, 0.06), 900.0), 1.0);
        data.push(clamp_amp(w));

        // sand dig: granular shuffling
        let raw = noise_burst(5000, 51, 0.42, 0.01, 0.2);
        let mut sa = one_pole_lp(raw, 800.0);
        let mut rng = Rng::new(52);
        for i in 0..sa.len() {
            if rng.next_f32() < 0.35 {
                sa[i] *= 0.3;
            }
        }
        data.push(clamp_amp(sa));

        // leaves dig: airy high rustle
        data.push(clamp_amp(one_pole_hp(noise_burst(2500, 61, 0.32, 0.004, 0.1), 2800.0)));

        // glass break: shatter = high noise + descending pings
        let mut gl = one_pole_hp(noise_burst(4500, 71, 0.45, 0.001, 0.18), 3200.0);
        mix_into(&mut gl, &ping(3700.0, 0.09, 0.3, 72), 1.0);
        mix_into(&mut gl, &ping(3050.0, 0.07, 0.28, 73), 1.0);
        mix_into(&mut gl, &ping(2400.0, 0.06, 0.25, 74), 1.0);
        data.push(clamp_amp(gl));

        // splash: noise sweep, slower attack
        let sp = one_pole_lp(noise_burst(6600, 81, 0.55, 0.02, 0.3), 2400.0);
        data.push(clamp_amp(sp));

        let wavs = data.iter().map(|d| to_wav16(d, RATE)).collect();
        SoundBank { data, wavs }
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
    out.extend_from_slice(&((rate * 2) as u32).to_le_bytes());
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

// ------------------------------------------------------------- backends --

pub trait AudioBackend {
    fn play(&self, bank: &SoundBank, family: SoundFamily, volume: f32, pitch: f32);
    /// unlock audio context (wasm: needs user gesture)
    fn unlock(&self, _bank: &SoundBank) {}
}

/// No-device silent fallback.
pub struct SilentOut;
impl AudioBackend for SilentOut {
    fn play(&self, _bank: &SoundBank, _family: SoundFamily, _volume: f32, _pitch: f32) {}
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
        fn play(&self, bank: &SoundBank, family: SoundFamily, volume: f32, pitch: f32) {
            let idx = family_index(family);
            let Some(base) = bank.data.get(idx) else { return };
            let samples = resample(base, pitch);
            let src = rodio::buffer::SamplesBuffer::new(1u16, RATE, samples);
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
        fn play(&self, _bank: &SoundBank, _family: SoundFamily, _volume: f32, _pitch: f32) {}
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
                    buffers: RefCell::new(vec![None; BANK_LEN]),
                }),
            }
        }
    }

    impl AudioBackend for WebAudioOut {
        fn play(&self, bank: &SoundBank, family: SoundFamily, volume: f32, pitch: f32) {
            let idx = family_index(family);
            let ctx_guard = self.inner.ctx.borrow();
            let Some(ctx) = ctx_guard.as_ref() else { return };
            let bufs = self.inner.buffers.borrow();
            let Some(buf) = bufs.get(idx).and_then(|b| b.as_ref()) else { return };
            if let Ok(src) = web_sys::AudioContext::create_buffer_source(ctx) {
                web_sys::AudioBufferSourceNode::set_buffer(&src, Some(buf));
                let pr = web_sys::AudioBufferSourceNode::playback_rate(&src);
                web_sys::AudioParam::set_value(&pr, pitch);
                let dest = web_sys::AudioContext::destination(ctx);
                if let Ok(gain) = web_sys::AudioContext::create_gain(ctx) {
                    let g = web_sys::GainNode::gain(&gain);
                    web_sys::AudioParam::set_value(&g, volume.clamp(0.0, 1.0));
                    let _ = web_sys::AudioNode::connect_with_audio_node(&src, &gain);
                    let _ = web_sys::AudioNode::connect_with_audio_node(&gain, &dest);
                } else {
                    let _ = web_sys::AudioNode::connect_with_audio_node(&src, &dest);
                }
                let _ = web_sys::AudioBufferSourceNode::start(&src);
            }
            drop(bufs);
            drop(ctx_guard);
            let _ = bank;
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

        // re-export resampler use for wasm (playbackRate used instead)
    }

    async fn decode(ctx: &web_sys::AudioContext, wav: &[u8]) -> Option<web_sys::AudioBuffer> {
        let arr_buf = js_sys::Uint8Array::from(wav).buffer();
        let promise = web_sys::AudioContext::decode_audio_data(ctx, &arr_buf).ok()?;
        let js = wasm_bindgen_futures::JsFuture::from(promise).await.ok()?;
        js.dyn_into::<web_sys::AudioBuffer>().ok()
    }
}
