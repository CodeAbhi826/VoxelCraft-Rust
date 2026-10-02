//! the bed bracket: the sleep interaction's PURE decision layer.
//!
//! VERIFIED w/Bed (live 2026-09-22, the reference wiki page /w/Bed — raw
//! wikitext via the MediaWiki API):
//!
//! * §Sleeping — "A player sleeps by using a bed when the internal sky
//!   light is 11 or below. This occurs between 12523 and 23477 ticks in
//!   clear weather, when stars appear in the sky, between 12002 and
//!   23998 ticks in rainy weather, and at any time of day during
//!   thunderstorms."
//! * §Sleeping — the monster gate: "If a monster is within 8 blocks of
//!   the bed head horizontally (in the X- and Z-axis), and 5 blocks
//!   vertically (in the Y-axis), the message 'You may not rest now;
//!   there are monsters nearby' ... appears and the player is prevented
//!   from sleeping until the monsters leave or are killed."
//! * §Sleeping — "Attempting to use a bed at any other time results in
//!   the message 'You can sleep only at night or during thunderstorms'".
//! * §Sleeping — the distance gate: "To use a bed, a player must be
//!   within a distance of 3 blocks" (Java).
//! * §Passing the night — "Sleeping changes the time of day to sunrise
//!   and resets the weather cycle, changing the weather to clear
//!   conditions. In Java, the weather cycle resets only during rainy or
//!   snowy weather."
//! * §Passing the night — "Once all players in a world are asleep, after
//!   5 seconds (100 ticks) the time of day changes to sunrise (time 0)."
//! * §Setting the spawn point — "Once a player has entered a bed (or
//!   right clicked the bed during daytime), their spawn point is set to
//!   the location of that bed." / "Using a bed in the daytime likewise
//!   sets the spawn point, without actually entering the bed."
//!
//! Engine adaptation, disclosed: the engine's day clock is the f32
//! `day_time` fraction 0..1 of the 24000-tick day (`DAY_LEN_SECS` 1200 s
//! at the 20 Hz sim — game.rs); the sleep attempt is instant on use (the
//! vanilla 101-tick sleep animation is a presentation trim); the gate
//! order puts the monster box FIRST — the daytime use still sets the
//! spawn despite the night refusal (the wiki's daytime row).

use super::weather::Weather;

/// the 24000-tick day — the vanilla 1.16.5 daylight cycle (1200 s at the
/// 20 Hz sim; the engine's `DAY_LEN_SECS` 1200.0 in game.rs). VERIFIED
/// (Bed §Sleeping states the windows in ticks of this day).
pub const DAY_TICKS: f32 = 24_000.0;

/// sleep window in ticks, clear weather — VERIFIED (Bed §Sleeping:
/// "between 12523 and 23477 ticks in clear weather").
pub const SLEEP_CLEAR_MIN: f32 = 12_523.0;
pub const SLEEP_CLEAR_MAX: f32 = 23_477.0;
/// sleep window in ticks, rainy weather — VERIFIED (Bed §Sleeping:
/// "between 12002 and 23998 ticks in rainy weather").
pub const SLEEP_RAIN_MIN: f32 = 12_002.0;
pub const SLEEP_RAIN_MAX: f32 = 23_998.0;

/// monster-gate radii around the bed head — VERIFIED (Bed §Sleeping):
/// "within 8 blocks of the bed head horizontally (in the X- and Z-axis),
/// and 5 blocks vertically (in the Y-axis)".
pub const MONSTER_RADIUS_H: f32 = 8.0;
pub const MONSTER_RADIUS_V: f32 = 5.0;

/// use-distance gate (Java) — VERIFIED (Bed §Sleeping: "a player must be
/// within a distance of 3 blocks" to use a bed).
pub const USE_DISTANCE: f32 = 3.0;

/// sunrise — the sleep skip's target (VERIFIED: "the time of day changes
/// to sunrise (time 0)").
pub const SUNRISE: f32 = 0.0;

/// The outcome of one sleep attempt (the game layer maps it to the
/// effects: the skip, the rest reset, the weather reset, the spawn set).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SleepAttempt {
    /// sleep: skip to sunrise + reset the rest counter + the weather
    Sleep,
    /// outside the sleep window — the refusal message, but the daytime
    /// use still sets the spawn (VERIFIED w/Bed §Setting the spawn point)
    RefuseNight,
    /// a monster is inside the gate box — no sleep, no spawn set
    /// (VERIFIED: "prevented from sleeping until the monsters leave")
    RefuseMonsters,
}

/// the sleep gate order: the monster box first, then the time window.
/// The daytime use reaches the window gate (its refusal still sets the
/// spawn — the wiki's daytime row); the monster refusal gates BOTH the
/// sleep and the spawn set (a refused player never enters the bed).
pub fn attempt(day_time: f32, weather: Weather, monster_near: bool) -> SleepAttempt {
    if monster_near {
        SleepAttempt::RefuseMonsters
    } else if window_contains(day_time, weather) {
        SleepAttempt::Sleep
    } else {
        SleepAttempt::RefuseNight
    }
}

/// the sleep window for the current weather: any time of day during a
/// thunderstorm, the rain window in rain, the clear window otherwise —
/// VERIFIED (Bed §Sleeping).
pub fn window_contains(day_time: f32, weather: Weather) -> bool {
    let t = day_time * DAY_TICKS;
    match weather {
        Weather::Thunder => true,
        Weather::Rain => (SLEEP_RAIN_MIN..=SLEEP_RAIN_MAX).contains(&t),
        Weather::Clear => (SLEEP_CLEAR_MIN..=SLEEP_CLEAR_MAX).contains(&t),
    }
}

/// the monster gate: any monster position inside the bed head's box —
/// |dx| <= 8 and |dz| <= 8 horizontally, |dy| <= 5 vertically (VERIFIED
/// w/Bed §Sleeping; the vanilla inflate(8, 5, 8) box, position-based).
pub fn monster_nearby(head: [f32; 3], monsters: &[[f32; 3]]) -> bool {
    monsters.iter().any(|m| {
        (m[0] - head[0]).abs() <= MONSTER_RADIUS_H
            && (m[2] - head[2]).abs() <= MONSTER_RADIUS_H
            && (m[1] - head[1]).abs() <= MONSTER_RADIUS_V
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dt_of(ticks: f32) -> f32 {
        ticks / DAY_TICKS
    }

    #[test]
    fn clear_window_contains_the_wiki_bounds() {
        // VERIFIED: "between 12523 and 23477 ticks in clear weather"
        assert!(window_contains(dt_of(12_523.0), Weather::Clear));
        assert!(window_contains(dt_of(23_477.0), Weather::Clear));
        assert!(window_contains(dt_of(18_000.0), Weather::Clear));
        // outside the window: the day side and the post-23477 tail
        assert!(!window_contains(dt_of(12_522.0), Weather::Clear));
        assert!(!window_contains(dt_of(23_478.0), Weather::Clear));
        assert!(!window_contains(dt_of(0.0), Weather::Clear));
        assert!(!window_contains(dt_of(12_000.0), Weather::Clear));
    }

    #[test]
    fn rain_window_is_wider_than_clear() {
        // VERIFIED: "between 12002 and 23998 ticks in rainy weather" —
        // rain lets sleep at ticks the clear window refuses
        assert!(window_contains(dt_of(12_100.0), Weather::Rain));
        assert!(!window_contains(dt_of(12_100.0), Weather::Clear));
        assert!(window_contains(dt_of(23_900.0), Weather::Rain));
        assert!(!window_contains(dt_of(23_900.0), Weather::Clear));
        assert!(window_contains(dt_of(12_002.0), Weather::Rain));
        assert!(window_contains(dt_of(23_998.0), Weather::Rain));
        assert!(!window_contains(dt_of(12_001.0), Weather::Rain));
        // 23999 ticks wraps past the day boundary — refused even in rain
        assert!(!window_contains(dt_of(23_999.0), Weather::Rain));
    }

    #[test]
    fn thunderstorm_allows_sleep_at_any_time() {
        // VERIFIED: "at any time of day during thunderstorms"
        assert!(window_contains(dt_of(0.0), Weather::Thunder));
        assert!(window_contains(dt_of(6_000.0), Weather::Thunder));
        assert!(window_contains(dt_of(12_000.0), Weather::Thunder));
        assert!(window_contains(dt_of(23_999.0), Weather::Thunder));
    }

    #[test]
    fn monster_box_matches_the_wiki_radii() {
        let head = [100.5, 64.0, 200.5];
        // inside: 8 horizontal + 5 vertical at the boundary
        assert!(monster_nearby(head, &[[108.5, 69.0, 200.5]]));
        assert!(monster_nearby(head, &[[100.5, 59.0, 208.5]]));
        assert!(monster_nearby(head, &[[96.0, 64.0, 196.0]]));
        // outside: 8.5 horizontal
        assert!(!monster_nearby(head, &[[109.5, 64.0, 200.5]]));
        assert!(!monster_nearby(head, &[[100.5, 64.0, 209.5]]));
        // outside: 5.5 vertical (both directions)
        assert!(!monster_nearby(head, &[[100.5, 70.0, 200.5]]));
        assert!(!monster_nearby(head, &[[100.5, 58.0, 200.5]]));
        // inside horizontally but far vertically: refused
        assert!(!monster_nearby(head, &[[101.0, 80.0, 201.0]]));
        // no monsters at all
        assert!(!monster_nearby(head, &[]));
    }

    #[test]
    fn gate_order_monsters_first_then_window() {
        // the monster refusal wins even inside the window
        assert_eq!(
            attempt(dt_of(18_000.0), Weather::Clear, true),
            SleepAttempt::RefuseMonsters
        );
        // inside the window without monsters → sleep
        assert_eq!(
            attempt(dt_of(18_000.0), Weather::Clear, false),
            SleepAttempt::Sleep
        );
        // daytime (outside both windows) → the night refusal
        assert_eq!(
            attempt(dt_of(6_000.0), Weather::Clear, false),
            SleepAttempt::RefuseNight
        );
        // a thunderstorm daytime use still sleeps
        assert_eq!(
            attempt(dt_of(6_000.0), Weather::Thunder, false),
            SleepAttempt::Sleep
        );
    }
}
