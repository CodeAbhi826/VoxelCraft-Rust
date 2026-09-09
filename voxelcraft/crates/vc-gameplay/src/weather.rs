//! 1.16-backlog bracket 1: the Java weather state machine.
//!
//! VERIFIED against minecraft.wiki/w/Weather (live 2026-09-09, raw capture
//! `scripts/backlog_page_Weather.json`) — every constant below is a wiki
//! row, not an invention:
//!
//! * "In Java Edition, weather is controlled by two internal flags that
//!   cycle on and off, one flag for rain and one flag for thunder."
//! * "The rain flag will stay on for 12000 to 24000 ticks (10 to 20
//!   minutes, inclusive) and off for 12000 to 180000 ticks (10 minutes
//!   to 2.5 hours, inclusive)."
//! * "The thunder flag will stay on for 3600 to 15600 ticks (3 to 13
//!   minutes, inclusive) and off for 12000 to 180000 ticks (10 minutes
//!   to 2.5 hours, inclusive)."
//! * "The thunder flag only affects the world while the rain flag is on."
//! * "Clear weather is the first weather in a newly created world."
//! * "Sleeping in a bed resets the weather to clear, but not the weather
//!   timers." (the bed bracket calls `sleep_reset()`)
//! * "There is a 30 second delay between flashes" — the lightning
//!   scheduler's minimum gap (600 game ticks).
//! * "Lightning deals 5 HP damage to entities on normal difficulty."
//! * "Lightning can strike only blocks that are exposed to the rain."
//! * "Lightning does not occur naturally in biomes that are too hot or
//!   dry to have rain or so cold that it snows instead of rains."
//! * "Rainstorms reduce the light level to 12 in full daylight" and
//!   thunderstorms "At noon the internal sky light level at surface is
//!   only 10. However, in mob spawning system, the light level from the
//!   sky is treated as if it were 0, allowing hostile mobs to spawn at
//!   any time of the day."
//!
//! Engine adaptation, disclosed: vanilla rolls the strike position in the
//! world-visible chunk range around the player; the game layer picks the
//! column (it owns the loaded-chunk map) and calls `should_strike()` only
//! for the cadence + eligibility. All numbers here are the vanilla ones.

use vc_rng::rng::Rng;

/// rain flag ON duration range (game ticks) — VERIFIED (wiki row above)
pub const RAIN_ON_MIN: u64 = 12_000;
pub const RAIN_ON_MAX: u64 = 24_000;
/// rain flag OFF duration range (game ticks) — VERIFIED
pub const RAIN_OFF_MIN: u64 = 12_000;
pub const RAIN_OFF_MAX: u64 = 180_000;
/// thunder flag ON duration range (game ticks) — VERIFIED
pub const THUNDER_ON_MIN: u64 = 3_600;
pub const THUNDER_ON_MAX: u64 = 15_600;
/// thunder flag OFF duration range (game ticks) — VERIFIED
pub const THUNDER_OFF_MIN: u64 = 12_000;
pub const THUNDER_OFF_MAX: u64 = 180_000;
/// minimum gap between lightning flashes (game ticks) — VERIFIED
/// ("a 30 second delay between flashes")
pub const LIGHTNING_GAP: u64 = 600;
/// lightning direct damage on Normal difficulty — VERIFIED
pub const LIGHTNING_DAMAGE: f32 = 5.0;
/// sky-light multiplier during rain (12/15 in full daylight) — VERIFIED
pub const RAIN_SKY_FACTOR: f32 = 12.0 / 15.0;
/// sky-light multiplier during a thunderstorm (10/15 at noon) — VERIFIED
pub const THUNDER_SKY_FACTOR: f32 = 10.0 / 15.0;

/// The weather phase the rest of the engine reads.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Weather {
    Clear,
    Rain,
    Thunder,
}

/// The two-flag machine + the lightning clock. `tick()` advances one game
/// tick; the caller drives it from the sim rate (20 Hz).
pub struct WeatherSystem {
    rain: bool,
    thunder: bool,
    /// ticks until the rain flag flips (re-rolled on each flip)
    rain_timer: u64,
    /// ticks until the thunder flag flips
    thunder_timer: u64,
    /// ticks until the next lightning strike becomes possible
    strike_cd: u64,
    /// total strikes that fired (F3/E2E evidence)
    pub strikes_total: u64,
    /// how many flips each flag has had (tests + F3 evidence)
    pub rain_flips: u64,
    pub thunder_flips: u64,
    rng: Rng,
}

impl WeatherSystem {
    pub fn new(seed: u64) -> Self {
        // "Clear weather is the first weather in a newly created world"
        // — both flags start OFF with fresh timers (VERIFIED)
        let mut rng = Rng::new(seed ^ 0x0F_4EA7);
        WeatherSystem {
            rain: false,
            thunder: false,
            rain_timer: Self::roll(&mut rng, RAIN_OFF_MIN, RAIN_OFF_MAX),
            thunder_timer: Self::roll(&mut rng, THUNDER_OFF_MIN, THUNDER_OFF_MAX),
            strike_cd: LIGHTNING_GAP,
            strikes_total: 0,
            rain_flips: 0,
            thunder_flips: 0,
            rng,
        }
    }

    fn roll(rng: &mut Rng, min: u64, max: u64) -> u64 {
        // uniform inclusive range — the same ladder the mob spawn tables use
        min + rng.next_range((max - min + 1) as u32) as u64
    }

    pub fn weather(&self) -> Weather {
        if self.rain {
            if self.thunder {
                Weather::Thunder
            } else {
                Weather::Rain
            }
        } else {
            // "When the rain flag is off, the weather is clear, regardless
            // of the thunder flag" (VERIFIED)
            Weather::Clear
        }
    }

    pub fn is_raining(&self) -> bool {
        self.rain
    }

    /// "The thunder flag only affects the world while the rain flag is on"
    /// (VERIFIED) — the observable thunderstorm state
    pub fn is_thunderstorm(&self) -> bool {
        self.rain && self.thunder
    }

    /// ticks remaining on the current rain timer (F3 debug row)
    pub fn rain_timer(&self) -> u64 {
        self.rain_timer
    }

    /// advance one game tick
    pub fn tick(&mut self) {
        if self.rain_timer > 0 {
            self.rain_timer -= 1;
        }
        if self.rain_timer == 0 {
            self.rain = !self.rain;
            self.rain_flips += 1;
            self.rain_timer = if self.rain {
                Self::roll(&mut self.rng, RAIN_ON_MIN, RAIN_ON_MAX)
            } else {
                Self::roll(&mut self.rng, RAIN_OFF_MIN, RAIN_OFF_MAX)
            };
        }
        // the thunder flag cycles independently — including while rain is
        // off ("it's possible for the thunder flag to cycle on and off
        // while the rain flag stays off", VERIFIED)
        if self.thunder_timer > 0 {
            self.thunder_timer -= 1;
        }
        if self.thunder_timer == 0 {
            self.thunder = !self.thunder;
            self.thunder_flips += 1;
            self.thunder_timer = if self.thunder {
                Self::roll(&mut self.rng, THUNDER_ON_MIN, THUNDER_ON_MAX)
            } else {
                Self::roll(&mut self.rng, THUNDER_OFF_MIN, THUNDER_OFF_MAX)
            };
        }
        if self.strike_cd > 0 {
            self.strike_cd -= 1;
        }
    }

    /// May a lightning strike fire this tick? Requires the observable
    /// thunderstorm state (rain ON + thunder ON) and the 30 s gap.
    /// "Lightning occurs frequently during thunderstorms, if there is
    /// rain." (VERIFIED)
    pub fn can_strike(&self) -> bool {
        self.is_thunderstorm() && self.strike_cd == 0
    }

    /// the game layer calls this when a strike lands (position selection,
    /// damage, fire and mob conversions live there — they need the world)
    pub fn strike_fired(&mut self) {
        self.strikes_total += 1;
        self.strike_cd = LIGHTNING_GAP;
    }

    /// "Sleeping in a bed resets the weather to clear, but not the weather
    /// timers" (VERIFIED) — clear the flags, keep both countdowns running.
    pub fn sleep_reset(&mut self) {
        self.rain = false;
        self.thunder = false;
    }

    /// the /weather stand-in (creative layer) — sets clear weather for a
    /// locked duration like vanilla's command timer
    pub fn force_clear(&mut self, ticks: u64) {
        self.rain = false;
        self.thunder = false;
        self.rain_timer = ticks.max(1);
        self.thunder_timer = ticks.max(1);
    }

    /// the /weather rain stand-in
    pub fn force_rain(&mut self, ticks: u64) {
        self.rain = true;
        self.rain_timer = ticks.max(1);
    }

    /// the /weather thunder stand-in
    pub fn force_thunder(&mut self, ticks: u64) {
        self.rain = true;
        self.thunder = true;
        self.rain_timer = ticks.max(1);
        self.thunder_timer = ticks.max(1);
    }

    /// sky-light factor for the current weather (1.0 clear, 12/15 rain,
    /// 10/15 thunderstorm) — the daylight sensor + spawn gates read it
    pub fn sky_factor(&self) -> f32 {
        if self.is_thunderstorm() {
            THUNDER_SKY_FACTOR
        } else if self.rain {
            RAIN_SKY_FACTOR
        } else {
            1.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clear_first_and_ranges() {
        // "Clear weather is the first weather in a newly created world"
        let w = WeatherSystem::new(1);
        assert_eq!(w.weather(), Weather::Clear);
        // initial timers sit inside the verified OFF windows
        assert!((RAIN_OFF_MIN..=RAIN_OFF_MAX).contains(&w.rain_timer()));
    }

    #[test]
    fn rain_flip_duration_stays_in_window() {
        let mut w = WeatherSystem::new(2);
        // drive to the first ON flip
        while !w.is_raining() {
            w.tick();
        }
        // the ON window is 12000..24000 — time it to the OFF flip
        let mut on_ticks = 0u64;
        while w.is_raining() {
            w.tick();
            on_ticks += 1;
        }
        assert!(
            (RAIN_ON_MIN..=RAIN_ON_MAX).contains(&on_ticks),
            "rain ON lasted {on_ticks} ticks (want 12000..=24000)"
        );
    }

    #[test]
    fn thunder_needs_rain() {
        let mut w = WeatherSystem::new(3);
        // force thunder without rain: observable state stays clear-ish
        w.thunder = true;
        w.rain = false;
        assert_eq!(w.weather(), Weather::Clear);
        assert!(!w.is_thunderstorm());
        w.rain = true;
        assert_eq!(w.weather(), Weather::Thunder);
        assert!(w.is_thunderstorm());
    }

    #[test]
    fn strike_gating_and_gap() {
        let mut w = WeatherSystem::new(4);
        w.rain = true;
        w.thunder = true;
        w.strike_cd = 0;
        assert!(w.can_strike());
        w.strike_fired();
        assert_eq!(w.strikes_total, 1);
        assert!(!w.can_strike(), "30 s gap must gate the next strike");
        // 600 ticks later it can strike again
        for _ in 0..LIGHTNING_GAP {
            w.tick();
        }
        assert!(w.can_strike());
        // and rain off kills strikes even mid-gap-0
        w.rain = false;
        assert!(!w.can_strike());
    }

    #[test]
    fn sleep_resets_flags_not_timers() {
        let mut w = WeatherSystem::new(5);
        w.force_thunder(10_000);
        let t_before = w.rain_timer();
        assert!(t_before > 0);
        w.sleep_reset();
        assert_eq!(w.weather(), Weather::Clear);
        // timers keep counting: a tick advances them
        let t_after = w.rain_timer();
        assert_eq!(t_after, t_before);
    }

    #[test]
    fn sky_factors_match_the_wiki() {
        let mut w = WeatherSystem::new(6);
        assert!((w.sky_factor() - 1.0).abs() < 1e-6);
        w.force_rain(100);
        assert!((w.sky_factor() - 12.0 / 15.0).abs() < 1e-6);
        w.force_thunder(100);
        assert!((w.sky_factor() - 10.0 / 15.0).abs() < 1e-6);
    }

    #[test]
    fn forced_states_expire_back_to_cycle() {
        let mut w = WeatherSystem::new(7);
        w.force_rain(3);
        for _ in 0..3 {
            w.tick();
        }
        assert!(!w.is_raining(), "3-tick forced rain must expire");
    }
}
