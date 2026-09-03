//! Phase-0 baseline instrumentation (Master Spec §44 / §37 / §48-Phase 0):
//!
//! * `FramePhases` — per-frame CPU phase timing (sim / stream / results /
//!   ui / draw) with a rolling ring buffer, exposed to F3 and the benchmark
//!   reporter. Present-side time is not measurable portably (wgpu submits
//!   async); it is labelled unavailable rather than fabricated (§31).
//! * `FrameStats` — percentile frame-time report (avg fps, median, 1% low,
//!   0.1% low, worst) computed from a monotonic frame-time history.
//! * `BenchState` — in-game deterministic benchmark mode (`--benchmark`):
//!   fixed seed, scripted camera orbit, N measured frames, JSON + table
//!   report, then exit.
//!
//! The headless CPU-pipeline benchmark (gen + light + mesh throughput, no
//! GPU/window) lives in `src/bin/vc_bench.rs` so it can run in CI and on
//! machines without a display.

/// monotonic-ish microsecond clock (native: Instant; wasm: Date.now f64 —
/// good enough for dev metrics, never fabricated when unavailable)
#[inline]
pub fn micros() -> u64 {
    #[cfg(not(target_arch = "wasm32"))]
    {
        use std::sync::OnceLock;
        use std::time::Instant;
        static START: OnceLock<Instant> = OnceLock::new();
        let start = START.get_or_init(Instant::now);
        start.elapsed().as_micros() as u64
    }
    #[cfg(target_arch = "wasm32")]
    {
        (js_sys::Date::now() * 1000.0) as u64
    }
}

/// measured CPU phases within a frame, in submission order
pub const PHASE_SIM: usize = 0; // player physics + interactions + world edits
pub const PHASE_STREAM: usize = 1; // chunk streaming / job submission
pub const PHASE_RESULTS: usize = 2; // job results: mesh upload to GPU
pub const PHASE_UI: usize = 3; // UI canvas rebuild (F3, HUD, menus)
pub const PHASE_DRAW: usize = 4; // renderer CPU side (command building + submit)
pub const PHASE_COUNT: usize = 5;

pub const PHASE_NAMES: [&str; PHASE_COUNT] = ["sim", "stream", "results", "ui", "draw"];

/// rolling per-frame phase timings (microseconds)
pub struct FramePhases {
    cur: [u64; PHASE_COUNT],
    ring: std::collections::VecDeque<[u64; PHASE_COUNT]>,
    /// wall time of complete frames (µs), same length as `ring`
    frames: std::collections::VecDeque<u64>,
    frame_start: u64,
    cap: usize,
    in_frame: bool,
}

impl FramePhases {
    pub fn new(cap: usize) -> Self {
        FramePhases {
            cur: [0; PHASE_COUNT],
            ring: std::collections::VecDeque::with_capacity(cap),
            frames: std::collections::VecDeque::with_capacity(cap),
            frame_start: 0,
            cap,
            in_frame: false,
        }
    }

    /// call once at the top of each frame (draw)
    #[inline]
    pub fn begin_frame(&mut self) {
        self.frame_start = micros();
        self.cur = [0; PHASE_COUNT];
        self.in_frame = true;
    }

    /// add measured µs to a phase (use with the `phase!` helper)
    #[inline]
    pub fn add(&mut self, phase: usize, us: u64) {
        if phase < PHASE_COUNT {
            self.cur[phase] += us;
        }
    }

    /// close the frame and push it into the ring
    pub fn end_frame(&mut self) {
        if !self.in_frame {
            return;
        }
        self.in_frame = false;
        let total = micros() - self.frame_start;
        self.ring.push_back(self.cur);
        self.frames.push_back(total);
        while self.ring.len() > self.cap {
            self.ring.pop_front();
            self.frames.pop_front();
        }
    }

    /// EMA-style compact line for F3 (ms per phase over the ring)
    pub fn f3_line(&self) -> String {
        let n = self.ring.len().max(1) as f64;
        let mut parts: Vec<String> = Vec::with_capacity(PHASE_COUNT + 1);
        for (i, name) in PHASE_NAMES.iter().enumerate() {
            let avg: f64 = self.ring.iter().map(|f| f[i] as f64).sum::<f64>() / n / 1000.0;
            parts.push(format!("{name} {avg:.1}"));
        }
        let cpu: f64 = self.ring.back().map(|f| f.iter().sum::<u64>() as f64).unwrap_or(0.0) / 1000.0;
        parts.push(format!("[cpu {cpu:.1} ms]"));
        parts.join("  ")
    }

    /// aggregate report over the current ring
    pub fn report(&self) -> PhaseReport {
        let n = self.ring.len().max(1);
        let mut avg_ms = [0f32; PHASE_COUNT];
        for f in self.ring.iter() {
            for i in 0..PHASE_COUNT {
                avg_ms[i] += f[i] as f32 / 1000.0 / n as f32;
            }
        }
        PhaseReport { frames: self.ring.len(), avg_ms }
    }

    pub fn frame_times_us(&self) -> &std::collections::VecDeque<u64> {
        &self.frames
    }
}

/// helper: measure a phase around an expression
///
/// ```ignore
/// phase!(self.phases, bench::PHASE_STREAM, self.stream());
/// ```
#[macro_export]
macro_rules! phase {
    ($phases:expr, $idx:expr, $body:expr) => {{
        let __t0 = $crate::bench::micros();
        let __r = $body;
        $phases.add($idx, $crate::bench::micros() - __t0);
        __r
    }};
}

// ------------------------------------------------------------------ stats --

/// percentile frame-time stats (§37: avg fps, median, 1% low, 0.1% low, worst)
#[derive(Clone, Debug)]
pub struct FrameStats {
    pub frames: usize,
    pub avg_ms: f32,
    pub median_ms: f32,
    /// mean of the worst 1% of frames (the standard "1% low" definition)
    pub low1_avg_ms: f32,
    /// mean of the worst 0.1% of frames
    pub low01_avg_ms: f32,
    /// nearest-rank 99th percentile threshold
    pub p99_ms: f32,
    pub worst_ms: f32,
}

impl FrameStats {
    /// `times` in microseconds, unsorted allowed
    pub fn from_us(times: &[u64]) -> Option<FrameStats> {
        if times.is_empty() {
            return None;
        }
        let mut ms: Vec<f32> = times.iter().map(|&t| t as f32 / 1000.0).collect();
        ms.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let n = ms.len();
        let pick = |q: f32| -> f32 {
            // nearest-rank percentile; p99 of 100 frames = the 99th value
            let idx = (((n as f32 - 1.0) * q).round() as usize).min(n - 1);
            ms[idx]
        };
        // worst-k average: the industry "X% low" = mean of the worst X%
        let worst_avg = |frac: f32| -> f32 {
            let k = ((n as f32 * frac).floor() as usize).max(1);
            ms[n - k..].iter().sum::<f32>() / k as f32
        };
        Some(FrameStats {
            frames: n,
            avg_ms: ms.iter().sum::<f32>() / n as f32,
            median_ms: pick(0.50),
            low1_avg_ms: worst_avg(0.01),
            low01_avg_ms: worst_avg(0.001),
            p99_ms: pick(0.99),
            worst_ms: ms[n - 1],
        })
    }

    pub fn fps(&self) -> f32 {
        if self.avg_ms > 0.0 {
            1000.0 / self.avg_ms
        } else {
            0.0
        }
    }

    /// 1%-low FPS (mean of the worst 1% of frames)
    pub fn one_pct_low(&self) -> f32 {
        if self.low1_avg_ms > 0.0 {
            1000.0 / self.low1_avg_ms
        } else {
            0.0
        }
    }

    /// 0.1%-low FPS (mean of the worst 0.1% of frames)
    pub fn point_one_pct_low(&self) -> f32 {
        if self.low01_avg_ms > 0.0 {
            1000.0 / self.low01_avg_ms
        } else {
            0.0
        }
    }

    pub fn to_json(&self) -> String {
        format!(
            "{{\"frames\":{},\"avg_ms\":{:.3},\"median_ms\":{:.3},\"low1_avg_ms\":{:.3},\"low01_avg_ms\":{:.3},\"p99_ms\":{:.3},\"worst_ms\":{:.3}}}",
            self.frames, self.avg_ms, self.median_ms, self.low1_avg_ms, self.low01_avg_ms, self.p99_ms, self.worst_ms
        )
    }
}

/// per-phase averages (ms) over a set of frames
#[derive(Clone, Debug)]
pub struct PhaseReport {
    pub frames: usize,
    pub avg_ms: [f32; PHASE_COUNT],
}

impl PhaseReport {
    pub fn to_json(&self) -> String {
        let parts: Vec<String> = PHASE_NAMES
            .iter()
            .zip(self.avg_ms.iter())
            .map(|(n, v)| format!("\"{n}\":{:.3}", v))
            .collect();
        format!("{{\"frames\":{},\"phases_ms\":{{{}}}}}", self.frames, parts.join(","))
    }
}

// ------------------------------------------------------------------ bench --

/// deterministic in-game benchmark configuration
#[derive(Clone, Debug)]
pub struct BenchState {
    /// frames measured after warmup
    pub frames: usize,
    /// warmup frames (shader compile, first meshes) excluded from stats
    pub warmup: usize,
    /// frames seen since the game screen was entered
    pub seen: usize,
    /// world seed (fixed for reproducibility)
    pub seed: u64,
    /// output JSON path (native only)
    pub json_path: Option<String>,
    /// bench-local clock (seconds since bench start)
    pub t: f32,
}

impl BenchState {
    pub fn from_args(args: &[String]) -> Option<BenchState> {
        // accepted: --benchmark [frames=N] [warmup=N] [seed=N] [json=path]
        let bench = args.iter().position(|a| a == "--benchmark")?;
        let mut st = BenchState {
            frames: 600,
            warmup: 120,
            seen: 0,
            seed: 0xC0FFEE,
            json_path: None,
            t: 0.0,
        };
        for a in args.iter().skip(bench + 1) {
            if a.starts_with("--") {
                break;
            }
            if let Some(v) = a.strip_prefix("frames=") {
                st.frames = v.parse().unwrap_or(st.frames);
            } else if let Some(v) = a.strip_prefix("warmup=") {
                st.warmup = v.parse().unwrap_or(st.warmup);
            } else if let Some(v) = a.strip_prefix("seed=") {
                st.seed = v.parse().unwrap_or(st.seed);
            } else if let Some(v) = a.strip_prefix("json=") {
                st.json_path = Some(v.to_string());
            }
        }
        Some(st)
    }

    /// scripted camera: deterministic orbit around the spawn area.
    /// radius 26, height +13 above spawn, look inward and slightly down.
    pub fn camera(&self, spawn: glam::Vec3) -> (glam::Vec3, f32, f32) {
        let ang = self.t * 0.30;
        let r = 26.0;
        let pos = glam::Vec3::new(
            spawn.x + ang.cos() * r,
            spawn.y + 13.0,
            spawn.z + ang.sin() * r,
        );
        // look toward the orbit center (yaw measured like the player's)
        let to_c = (glam::Vec3::new(spawn.x, spawn.y + 2.0, spawn.z) - pos).normalize();
        let yaw = f32::atan2(-to_c.x, -to_c.z);
        let pitch = to_c.y.clamp(-1.0, 1.0).asin();
        (pos, yaw, pitch)
    }
}

/// render the human-readable benchmark report
pub fn print_report(fs: &FrameStats, pr: &PhaseReport, present_mode: &str) {
    println!("\n================ VoxelCraft benchmark ================");
    println!("frames measured : {}", fs.frames);
    println!("avg fps         : {:.1}", fs.fps());
    println!("median frame    : {:.2} ms", fs.median_ms);
    println!("avg frame       : {:.2} ms", fs.avg_ms);
    println!("1% low          : {:.1} fps  (worst-1% avg {:.2} ms)", fs.one_pct_low(), fs.low1_avg_ms);
    println!("0.1% low        : {:.1} fps  (worst-0.1% avg {:.2} ms)", fs.point_one_pct_low(), fs.low01_avg_ms);
    println!("worst frame     : {:.2} ms", fs.worst_ms);
    println!("present mode    : {present_mode} (vsync may cap frame rate)");
    println!("CPU phase averages:");
    for (name, ms) in PHASE_NAMES.iter().zip(pr.avg_ms.iter()) {
        println!("  {name:<8} {ms:6.2} ms");
    }
    println!("======================================================\n");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn percentile_stats_basic() {
        // 100 frames of 10ms + one 100ms hitch
        let mut t: Vec<u64> = vec![10_000; 100];
        t.push(100_000);
        let s = FrameStats::from_us(&t).unwrap();
        assert_eq!(s.frames, 101);
        assert!((s.median_ms - 10.0).abs() < 0.01, "median {}", s.median_ms);
        // 1% low = mean of the worst 1% (floor(101*0.01)=1 frame) = the hitch
        assert!((s.low1_avg_ms - 100.0).abs() < 0.01, "low1 {}", s.low1_avg_ms);
        assert!((s.one_pct_low() - 10.0).abs() < 0.01, "1% low fps {}", s.one_pct_low());
        // 0.1% low also = the single worst frame here
        assert!((s.low01_avg_ms - 100.0).abs() < 0.01);
        assert!(s.worst_ms >= 99.0);
        assert!(s.p99_ms <= 10.01, "nearest-rank p99 with 1/101 outlier = 10ms, got {}", s.p99_ms);
    }

    #[test]
    fn empty_stats_none() {
        assert!(FrameStats::from_us(&[]).is_none());
    }

    #[test]
    fn phase_ring_behavior() {
        let mut p = FramePhases::new(4);
        p.begin_frame();
        p.add(PHASE_SIM, 400);
        p.add(PHASE_DRAW, 900);
        p.end_frame();
        p.begin_frame();
        p.add(PHASE_SIM, 200);
        p.end_frame();
        assert_eq!(p.ring.len(), 2);
        let r = p.report();
        assert!((r.avg_ms[PHASE_SIM] - 0.3).abs() < 0.001, "sim avg {}", r.avg_ms[PHASE_SIM]);
        assert!((r.avg_ms[PHASE_DRAW] - 0.45).abs() < 0.001, "draw avg {}", r.avg_ms[PHASE_DRAW]);
        assert!(p.frame_times_us().len() == 2);
        let line = p.f3_line();
        assert!(line.contains("sim") && line.contains("draw"));
    }
}
