//! Clean-room weather state machine and simulation.
//!
//! Citations & specifications:
//! - `minecraft.wiki/w/Weather`
//! - Clear weather duration: 12,000 to 180,000 ticks (0.5 to 7.5 Minecraft days).
//! - Rain duration: 12,000 to 24,000 ticks (10 to 20 minutes).
//! - Thunderstorm duration: 3,600 to 15,600 ticks (3 to 13 minutes, active during rain).
//! - Intensity transition: gradual linear ramp over 100 ticks (0.01 per tick / 5.0 seconds).

/// Active weather conditions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WeatherKind {
    Clear,
    Rain,
    Thunder,
}

impl Default for WeatherKind {
    fn default() -> Self {
        Self::Clear
    }
}

/// Weather simulation state machine tracking durations and smooth transition levels.
#[derive(Debug, Clone)]
pub struct WeatherSystem {
    pub kind: WeatherKind,
    pub timer: u32,
    pub thunder_timer: u32,
    /// Smooth rain level in [0.0, 1.0] (0.01 change per tick, 100-tick transition).
    pub rain_level: f32,
    /// Smooth thunder level in [0.0, 1.0] (0.01 change per tick, 100-tick transition).
    pub thunder_level: f32,
    rng_seed: u64,
}

impl WeatherSystem {
    pub const CLEAR_MIN_TICKS: u32 = 12_000;
    pub const CLEAR_MAX_TICKS: u32 = 180_000;
    pub const RAIN_MIN_TICKS: u32 = 12_000;
    pub const RAIN_MAX_TICKS: u32 = 24_000;
    pub const THUNDER_MIN_TICKS: u32 = 3_600;
    pub const THUNDER_MAX_TICKS: u32 = 15_600;
    pub const TRANSITION_SPEED: f32 = 0.01;

    pub fn new(seed: u64) -> Self {
        let mut ws = Self {
            kind: WeatherKind::Clear,
            timer: 0,
            thunder_timer: 0,
            rain_level: 0.0,
            thunder_level: 0.0,
            rng_seed: seed ^ 0x5DEECE66D,
        };
        ws.timer = ws.random_range(Self::CLEAR_MIN_TICKS, Self::CLEAR_MAX_TICKS);
        ws.thunder_timer = ws.random_range(Self::THUNDER_MIN_TICKS, Self::THUNDER_MAX_TICKS);
        ws
    }

    /// Fast pseudo-random generator (LCG).
    fn next_u32(&mut self) -> u32 {
        self.rng_seed = self.rng_seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        (self.rng_seed >> 32) as u32
    }

    pub fn random_range(&mut self, min: u32, max: u32) -> u32 {
        if max <= min {
            return min;
        }
        let range = max - min;
        min + (self.next_u32() % range)
    }

    /// Advance the weather simulation by one tick (50ms).
    pub fn tick(&mut self) {
        if self.timer > 0 {
            self.timer -= 1;
        } else {
            // State transition
            match self.kind {
                WeatherKind::Clear => {
                    self.kind = WeatherKind::Rain;
                    self.timer = self.random_range(Self::RAIN_MIN_TICKS, Self::RAIN_MAX_TICKS);
                }
                WeatherKind::Rain | WeatherKind::Thunder => {
                    self.kind = WeatherKind::Clear;
                    self.timer = self.random_range(Self::CLEAR_MIN_TICKS, Self::CLEAR_MAX_TICKS);
                    self.thunder_timer = self.random_range(Self::THUNDER_MIN_TICKS, Self::THUNDER_MAX_TICKS);
                }
            }
        }

        // Thunder cycle ticks during rain
        if self.kind == WeatherKind::Rain || self.kind == WeatherKind::Thunder {
            if self.thunder_timer > 0 {
                self.thunder_timer -= 1;
            } else if self.kind == WeatherKind::Rain {
                self.kind = WeatherKind::Thunder;
                self.thunder_timer = self.random_range(Self::THUNDER_MIN_TICKS, Self::THUNDER_MAX_TICKS);
            } else {
                self.kind = WeatherKind::Rain;
                self.thunder_timer = self.random_range(Self::THUNDER_MIN_TICKS, Self::THUNDER_MAX_TICKS);
            }
        }

        // Target transition levels
        let target_rain = if self.kind == WeatherKind::Rain || self.kind == WeatherKind::Thunder {
            1.0
        } else {
            0.0
        };
        let target_thunder = if self.kind == WeatherKind::Thunder {
            1.0
        } else {
            0.0
        };

        if (self.rain_level - target_rain).abs() <= Self::TRANSITION_SPEED + 1e-4 {
            self.rain_level = target_rain;
        } else if self.rain_level < target_rain {
            self.rain_level = (self.rain_level + Self::TRANSITION_SPEED).min(1.0);
        } else {
            self.rain_level = (self.rain_level - Self::TRANSITION_SPEED).max(0.0);
        }

        if (self.thunder_level - target_thunder).abs() <= Self::TRANSITION_SPEED + 1e-4 {
            self.thunder_level = target_thunder;
        } else if self.thunder_level < target_thunder {
            self.thunder_level = (self.thunder_level + Self::TRANSITION_SPEED).min(1.0);
        } else {
            self.thunder_level = (self.thunder_level - Self::TRANSITION_SPEED).max(0.0);
        }
    }

    /// Force a weather state (e.g. from `/weather clear`, `/weather rain`, `/weather thunder`).
    pub fn set_weather(&mut self, kind: WeatherKind, duration_ticks: Option<u32>) {
        self.kind = kind;
        self.timer = duration_ticks.unwrap_or_else(|| match kind {
            WeatherKind::Clear => self.random_range(Self::CLEAR_MIN_TICKS, Self::CLEAR_MAX_TICKS),
            WeatherKind::Rain => self.random_range(Self::RAIN_MIN_TICKS, Self::RAIN_MAX_TICKS),
            WeatherKind::Thunder => self.random_range(Self::THUNDER_MIN_TICKS, Self::THUNDER_MAX_TICKS),
        });
    }

    pub fn is_raining(&self) -> bool {
        self.rain_level > 0.2
    }

    pub fn is_thundering(&self) -> bool {
        self.thunder_level > 0.5
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_weather_durations_and_bounds() {
        let ws = WeatherSystem::new(42);
        assert_eq!(ws.kind, WeatherKind::Clear);
        assert!(ws.timer >= WeatherSystem::CLEAR_MIN_TICKS);
        assert!(ws.timer <= WeatherSystem::CLEAR_MAX_TICKS);
        assert_eq!(ws.rain_level, 0.0);
        assert_eq!(ws.thunder_level, 0.0);
        assert!(!ws.is_raining());
        assert!(!ws.is_thundering());
    }

    #[test]
    fn test_weather_smooth_transition() {
        let mut ws = WeatherSystem::new(12345);
        // Force rain with 500 ticks
        ws.set_weather(WeatherKind::Rain, Some(500));
        assert_eq!(ws.kind, WeatherKind::Rain);
        assert_eq!(ws.rain_level, 0.0);

        // Tick 50 times -> rain level reaches ~0.50
        for _ in 0..50 {
            ws.tick();
        }
        assert!((ws.rain_level - 0.50).abs() < 0.02);
        assert!(ws.is_raining());

        // Tick another 50 times -> rain level reaches 1.0
        for _ in 0..50 {
            ws.tick();
        }
        assert!((ws.rain_level - 1.0).abs() < 1e-4);

        // Switch back to clear -> smoothly ramps down to 0.0
        ws.set_weather(WeatherKind::Clear, Some(500));
        for _ in 0..100 {
            ws.tick();
        }
        assert!((ws.rain_level - 0.0).abs() < 1e-4);
        assert!(!ws.is_raining());
    }

    #[test]
    fn test_thunder_duration_bounds() {
        let mut ws = WeatherSystem::new(999);
        let val = ws.random_range(WeatherSystem::THUNDER_MIN_TICKS, WeatherSystem::THUNDER_MAX_TICKS);
        assert!(val >= 3_600 && val <= 15_600);
    }
}
